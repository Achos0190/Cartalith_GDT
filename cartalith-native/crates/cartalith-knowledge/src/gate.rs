//! RK-9: the step-2 decision gate (`RAG_KNOWLEDGE_BASE_RESEARCH.md` s.8 RK-9, owner question K10).
//!
//! Step 1 of the knowledge base is lexical. Embeddings, fusion and a reranker are added **only if
//! a measurement shows a miss they would fix**. This module is that measurement, as a pure function
//! of the RK-0 questions, the real lexical index and a [`VectorSet`]:
//!
//! 1. [`evaluate`] scores the lexical retriever and the lexical-plus-dense RRF hybrid
//!    ([`HybridRetriever`]) on the same questions with RK-0's own scorer, reports the lexical
//!    **misses by category**, the hybrid's recall@10, and the index size the vectors would take.
//! 2. It applies the milestone's rule, with the numbers as named constants: **adopt only if** the hybrid
//!    lifts recall@10 on the *paraphrase / miss subset* by at least [`ADOPT_MIN_LIFT_POINTS`] points
//!    **and** loses no more than [`ADOPT_MAX_EXACT_LOSS_POINTS`] point on exact-name questions;
//!    otherwise the verdict is [`Verdict::NotNeeded`] and the numbers are kept.
//! 3. A set of vectors that does not cover every unit and every question is [`Verdict::Incomplete`],
//!    never a verdict: a half-embedded corpus must not be read as "the hybrid did not help".
//!
//! # Who produces the vectors
//!
//! A live embedder: `godot-project/_llmknow_embed_cli.gd` posts the texts [`inputs`] lists to LM Studio's
//! `/v1/embeddings` or Ollama's `/api/embed` and writes a [`VectorSet`]. That run needs a real model
//! and is the owner's to launch (the instruction for this build: never load or switch a model). What is
//! built and tested here is everything else, and the **controls** that prove the gate can both pass and
//! fail without any model:
//!
//! * [`HashingEmbedder`] - a deterministic character-trigram feature-hashing embedder. A stand-in with
//!   a plausible dense behaviour (it is robust to spelling variants and blind to synonyms); it
//!   exercises the harness and says nothing about Qwen3-Embedding or nomic.
//! * [`GoldBiasedEmbedder`] - the positive control: each question's vector is pulled toward its gold
//!   units (it cheats). The gate **must** adopt this one, or the gate cannot pass.
//! * [`RandomEmbedder`] - the negative control: seeded pseudo-random vectors. The gate **must not**
//!   adopt this one, or the gate adopts noise.
//!
//! # What is not measured here
//!
//! Embedding throughput (tokens per second), VRAM spill next to a chat model and a real model's
//! recall are properties of a live run; [`Timing`] carries the first two when a live file provides
//! them and the report prints `-` (not measured) otherwise.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::embed::{embed_all, embedding_text, Embedder, HybridRetriever, Profile, VectorCache, VectorSet};
use crate::eval::{score, Bm25Retriever, Fixture, Question, SyntheticVault};
use crate::search::KnowledgeIndex;
use crate::tokenize::tokenize;

/// Adopt only if the hybrid lifts recall@10 on the paraphrase / miss subset by at least this many
/// points (RK-9 bar, binding: "at least 10 points").
pub const ADOPT_MIN_LIFT_POINTS: f64 = 10.0;

/// ...and loses no more than this many points of recall@10 on exact-name questions (RK-9 bar,
/// binding: "no more than 1 point").
pub const ADOPT_MAX_EXACT_LOSS_POINTS: f64 = 1.0;

/// What a live embedding run measured about itself.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Timing {
    /// Tokens embedded (as the server reported, or `None` if it did not).
    pub tokens: Option<u64>,
    /// Wall-clock seconds of the embedding pass, per repeat.
    pub seconds: Vec<f64>,
    /// Peak video memory above the idle baseline in MiB, if the run sampled it.
    pub vram_mib: Option<f64>,
}

/// The gate's decision.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Verdict {
    /// Both conditions hold.
    Adopt,
    /// A condition failed; `why` names the one(s) that did. The numbers are kept.
    NotNeeded {
        /// The failed conditions.
        why: Vec<String>,
    },
    /// The vectors do not cover the corpus and the questions; nothing was decided.
    Incomplete {
        /// What is missing.
        why: String,
    },
}

/// One category's lexical-versus-hybrid recall@10 and the lexical misses.
#[derive(Debug, Clone, Default, Serialize)]
pub struct CategoryGate {
    /// Questions with gold ids in the category.
    pub with_gold: usize,
    /// Of those, the questions the lexical retriever did not fully answer (recall@10 below 1).
    pub lexical_misses: usize,
    /// Lexical mean recall@10.
    pub lexical_r10: f64,
    /// Hybrid mean recall@10.
    pub hybrid_r10: f64,
}

/// The gate's report for one model on one world.
#[derive(Debug, Clone, Serialize)]
pub struct GateReport {
    /// The model id the vectors came from.
    pub model: String,
    /// Their dimension.
    pub dim: usize,
    /// Units embedded.
    pub units: usize,
    /// Questions scored.
    pub questions: usize,
    /// Questions answered from the lexical list alone for want of a query vector (must be 0 for a verdict).
    pub fallbacks: usize,
    /// Per-category numbers.
    pub by_category: BTreeMap<String, CategoryGate>,
    /// Size of the questions in the paraphrase / miss subset.
    pub miss_subset: usize,
    /// Lexical recall@10 on that subset.
    pub miss_lexical_r10: f64,
    /// Hybrid recall@10 on that subset.
    pub miss_hybrid_r10: f64,
    /// Hybrid minus lexical on that subset, in points (x100).
    pub lift_points: f64,
    /// Lexical minus hybrid on exact-name questions, in points (positive is a loss).
    pub exact_loss_points: f64,
    /// Overall lexical and hybrid recall@10.
    pub overall_lexical_r10: f64,
    /// See above.
    pub overall_hybrid_r10: f64,
    /// Bytes the vectors take as `f32` (`units * dim * 4`): the index size.
    pub index_bytes: usize,
    /// The decision.
    pub verdict: Verdict,
    /// A live run's own measurements, if any.
    pub timing: Option<Timing>,
}

/// The `(id, text)` of every unit of `index` and the `(id, text)` of every question, exactly as they
/// must be embedded under `p` (prefixes applied, text cut). The live CLI embeds these and nothing else.
pub fn inputs(index: &KnowledgeIndex, qs: &[Question], p: &Profile) -> (Vec<(String, String)>, Vec<(String, String)>) {
    let mut units: Vec<(String, String)> = index.ids().filter_map(|id| index.get(id).map(|d| (id.to_string(), p.document_text(&embedding_text(d))))).collect();
    units.sort();
    let queries = qs.iter().map(|q| (q.id.clone(), p.query_text(&q.text))).collect();
    (units, queries)
}

/// The paraphrase category plus every answerable question the lexical retriever missed.
fn miss_subset_ids(qs: &[Question], lex_per_q: &BTreeMap<String, f64>) -> BTreeSet<String> {
    qs.iter().filter(|q| !q.gold.is_empty() && (q.category == "paraphrase" || lex_per_q.get(&q.id).is_some_and(|r| *r < 1.0))).map(|q| q.id.clone()).collect()
}

/// Per-question recall@10 for a retriever (via RK-0's scorer run on one question).
fn per_question_r10(r: &dyn crate::eval::Retriever, qs: &[Question]) -> BTreeMap<String, f64> {
    qs.iter()
        .filter(|q| !q.gold.is_empty())
        .map(|q| {
            let rep = score(r, std::slice::from_ref(q));
            (q.id.clone(), rep.overall_recall10)
        })
        .collect()
}

fn mean_over(ids: &BTreeSet<String>, per_q: &BTreeMap<String, f64>) -> f64 {
    let v: Vec<f64> = ids.iter().filter_map(|i| per_q.get(i)).copied().collect();
    if v.is_empty() { 0.0 } else { v.iter().sum::<f64>() / v.len() as f64 }
}

/// Runs the gate (module doc) for `vectors` on `qs` over `index`.
pub fn evaluate(index: &KnowledgeIndex, qs: &[Question], vectors: &VectorSet, timing: Option<Timing>) -> GateReport {
    let units = index.ids().count();
    let incomplete = |why: String| GateReport {
        model: vectors.model.clone(),
        dim: vectors.dim,
        units,
        questions: qs.len(),
        fallbacks: 0,
        by_category: BTreeMap::new(),
        miss_subset: 0,
        miss_lexical_r10: 0.0,
        miss_hybrid_r10: 0.0,
        lift_points: 0.0,
        exact_loss_points: 0.0,
        overall_lexical_r10: 0.0,
        overall_hybrid_r10: 0.0,
        index_bytes: vectors.units.len() * vectors.dim * 4,
        verdict: Verdict::Incomplete { why },
        timing: timing.clone(),
    };
    let missing_units = index.ids().filter(|id| !vectors.units.contains_key(*id)).count();
    let missing_queries = qs.iter().filter(|q| !vectors.queries.contains_key(&q.id)).count();
    if missing_units > 0 || missing_queries > 0 || vectors.dim == 0 {
        return incomplete(format!("{missing_units} of {units} units and {missing_queries} of {} questions have no vector (dimension {})", qs.len(), vectors.dim));
    }
    let lexical = Bm25Retriever { index, label: "lexical".into() };
    let hybrid = HybridRetriever::new(index, vectors, "hybrid");
    let lex_rep = score(&lexical, qs);
    let hyb_rep = score(&hybrid, qs);
    let lex_q = per_question_r10(&lexical, qs);
    let hyb_q = per_question_r10(&hybrid, qs);
    let subset = miss_subset_ids(qs, &lex_q);
    let (ml, mh) = (mean_over(&subset, &lex_q), mean_over(&subset, &hyb_q));
    let lift = (mh - ml) * 100.0;
    let exact_loss = (lex_rep.by_category.get("exact_name").map_or(0.0, |c| c.recall10) - hyb_rep.by_category.get("exact_name").map_or(0.0, |c| c.recall10)) * 100.0;
    let mut by_category = BTreeMap::new();
    for q in qs.iter().filter(|q| !q.gold.is_empty()) {
        let c: &mut CategoryGate = by_category.entry(q.category.clone()).or_default();
        c.with_gold += 1;
        if lex_q.get(&q.id).is_some_and(|r| *r < 1.0) {
            c.lexical_misses += 1;
        }
    }
    for (cat, c) in by_category.iter_mut() {
        c.lexical_r10 = lex_rep.by_category.get(cat).map_or(0.0, |x| x.recall10);
        c.hybrid_r10 = hyb_rep.by_category.get(cat).map_or(0.0, |x| x.recall10);
    }
    let mut why = Vec::new();
    if subset.is_empty() {
        why.push("the lexical retriever missed nothing, so there is nothing to lift".to_string());
    } else if lift < ADOPT_MIN_LIFT_POINTS {
        why.push(format!("lift on the paraphrase/miss subset is {lift:.1} points, below the {ADOPT_MIN_LIFT_POINTS} required"));
    }
    if exact_loss > ADOPT_MAX_EXACT_LOSS_POINTS {
        why.push(format!("exact-name recall@10 falls {exact_loss:.1} points, above the {ADOPT_MAX_EXACT_LOSS_POINTS} allowed"));
    }
    GateReport {
        model: vectors.model.clone(),
        dim: vectors.dim,
        units,
        questions: qs.len(),
        fallbacks: hybrid.fallbacks(),
        by_category,
        miss_subset: subset.len(),
        miss_lexical_r10: ml,
        miss_hybrid_r10: mh,
        lift_points: lift,
        exact_loss_points: exact_loss,
        overall_lexical_r10: lex_rep.overall_recall10,
        overall_hybrid_r10: hyb_rep.overall_recall10,
        index_bytes: units * vectors.dim * 4,
        verdict: if why.is_empty() { Verdict::Adopt } else { Verdict::NotNeeded { why } },
        timing,
    }
}

impl GateReport {
    /// The report as markdown: the verdict, the per-category miss table and the cost lines. A figure
    /// a stand-in or an absent timing cannot supply prints `-`.
    pub fn render(&self, title: &str) -> String {
        let verdict = match &self.verdict {
            Verdict::Adopt => "ADOPT (both conditions hold)".to_string(),
            Verdict::NotNeeded { why } => format!("NOT NEEDED - {}", why.join("; ")),
            Verdict::Incomplete { why } => format!("INCOMPLETE, no decision - {why}"),
        };
        let mut s = format!("#### {title}\n\nModel `{}` (dimension {}), {} units, {} questions. Verdict: **{}**.\n\n", self.model, self.dim, self.units, self.questions, verdict);
        if matches!(self.verdict, Verdict::Incomplete { .. }) {
            return s;
        }
        s.push_str(&format!(
            "Paraphrase / miss subset ({} questions): lexical recall@10 {:.3}, hybrid {:.3}, lift {:+.1} points (bar: at least {ADOPT_MIN_LIFT_POINTS}). Exact-name loss {:+.1} points (bar: at most {ADOPT_MAX_EXACT_LOSS_POINTS}). Overall recall@10 lexical {:.3}, hybrid {:.3}. Questions without a query vector: {}.\n\n",
            self.miss_subset, self.miss_lexical_r10, self.miss_hybrid_r10, self.lift_points, self.exact_loss_points, self.overall_lexical_r10, self.overall_hybrid_r10, self.fallbacks
        ));
        s.push_str("| Category | With gold | Lexical misses | Lexical R@10 | Hybrid R@10 |\n|---|---:|---:|---:|---:|\n");
        for (cat, c) in &self.by_category {
            s.push_str(&format!("| {} | {} | {} | {:.3} | {:.3} |\n", cat, c.with_gold, c.lexical_misses, c.lexical_r10, c.hybrid_r10));
        }
        s.push_str(&format!("\nIndex size: {} bytes as f32 ({:.2} MiB).", self.index_bytes, self.index_bytes as f64 / 1_048_576.0));
        match &self.timing {
            Some(t) => {
                let tps = t.tokens.and_then(|n| crate::live_eval::Spread::of(&t.seconds).filter(|s| s.median > 0.0).map(|s| n as f64 / s.median));
                s.push_str(&format!(
                    " Embedding throughput {} tokens/s (median of {} passes{}); VRAM above idle {}.\n",
                    tps.map_or("-".to_string(), |v| format!("{v:.0}")),
                    t.seconds.len(),
                    if t.seconds.len() > 1 { format!(", range {:.1}..{:.1} s", t.seconds.iter().cloned().fold(f64::INFINITY, f64::min), t.seconds.iter().cloned().fold(0.0, f64::max)) } else { String::new() },
                    t.vram_mib.map_or("-".to_string(), |v| format!("{v:.0} MiB")),
                ));
            }
            None => s.push_str(" Embedding throughput - and VRAM spill - (not measured: no live embedding run).\n"),
        }
        s
    }
}

// ----------------------------------------------------------------------------------- controls

/// FNV-1a over a byte slice with a seed folded in: the feature hash of the stand-ins.
fn fnv(seed: u64, bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325 ^ seed.wrapping_mul(0x9e37_79b9_7f4a_7c15);
    for b in bytes {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

fn normalise(v: &mut [f32]) {
    let n = v.iter().map(|x| f64::from(*x).powi(2)).sum::<f64>().sqrt();
    if n > 0.0 {
        for x in v.iter_mut() {
            *x = (f64::from(*x) / n) as f32;
        }
    }
}

/// A deterministic character-trigram feature-hashing embedder: every trigram of every word adds `+1`
/// or `-1` to one of `dim` slots (the signed hashing trick), then the vector is L2-normalised. A
/// stand-in with a dense-model flavour (robust to spelling, blind to synonyms). Not a quality claim
/// about any real model.
pub struct HashingEmbedder {
    /// Output dimension.
    pub dim: usize,
}

impl Embedder for HashingEmbedder {
    fn name(&self) -> String {
        format!("stand-in:trigram-hash-{}", self.dim)
    }
    fn dim(&self) -> usize {
        self.dim
    }
    fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, String> {
        Ok(texts.iter().map(|t| trigram_vector(t, self.dim)).collect())
    }
}

fn trigram_vector(text: &str, dim: usize) -> Vec<f32> {
    let mut v = vec![0.0f32; dim];
    for tok in tokenize(text) {
        let padded: Vec<char> = format!("^{tok}$").chars().collect();
        for w in padded.windows(3) {
            let s: String = w.iter().collect();
            let h = fnv(0, s.as_bytes());
            let slot = (h % dim as u64) as usize;
            v[slot] += if (h >> 40) & 1 == 0 { 1.0 } else { -1.0 };
        }
    }
    normalise(&mut v);
    v
}

/// The negative control: a seeded pseudo-random unit vector per text, unrelated to its content.
pub struct RandomEmbedder {
    /// Output dimension.
    pub dim: usize,
    /// Seed.
    pub seed: u64,
}

impl Embedder for RandomEmbedder {
    fn name(&self) -> String {
        format!("control:random-{}", self.dim)
    }
    fn dim(&self) -> usize {
        self.dim
    }
    fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, String> {
        Ok(texts
            .iter()
            .map(|t| {
                let mut v: Vec<f32> = (0..self.dim).map(|i| (fnv(self.seed.wrapping_add(i as u64 + 1), t.as_bytes()) % 2001) as f32 - 1000.0).collect();
                normalise(&mut v);
                v
            })
            .collect())
    }
}

/// The positive control: builds the [`VectorSet`] of a [`HashingEmbedder`] and then replaces every
/// question's vector with the (normalised) sum of one vector per gold id, so a dense search for it
/// finds its gold. It cheats on purpose: it shows the gate **can** adopt.
pub fn gold_biased_set(index: &KnowledgeIndex, qs: &[Question], dim: usize) -> VectorSet {
    let mut cache = VectorCache::new("scratch", dim);
    let mut set = embed_all(&HashingEmbedder { dim }, &crate::embed::PLAIN, index, qs, &mut cache).expect("the stand-in cannot fail");
    set.model = format!("control:gold-biased-{dim}");
    for q in qs {
        // One unit per gold id: the unit with exactly that id, else (a gold NOTE id names every chunk
        // of the note) the shortest-id chunk, i.e. its first. Summing every chunk of a long note would
        // put the query at the note's centroid, which is near none of its chunks.
        let mut sum = vec![0.0f32; dim];
        for g in &q.gold {
            let pick = set.units.keys().filter(|id| crate::model::id_matches(g, id)).min_by_key(|id| (id.len(), id.to_string()));
            if let Some(v) = pick.and_then(|id| set.units.get(id)) {
                for (s, x) in sum.iter_mut().zip(v) {
                    *s += *x;
                }
            }
        }
        if sum.iter().any(|x| *x != 0.0) {
            normalise(&mut sum);
            set.queries.insert(q.id.clone(), sum);
        }
    }
    set
}

/// Builds the index and questions for a committed fixture (the world input, the synthetic vault
/// written under `root`), the way the RK-0 tests do. The caller removes `root`.
pub fn index_for_fixture(fx: &Fixture, root: &std::path::Path) -> Result<(KnowledgeIndex, Vec<Question>), String> {
    let sv: SyntheticVault = crate::eval::build_vault(&fx.world, crate::eval::MIN_NOTES);
    sv.write_to(root).map_err(|e| e.to_string())?;
    let (index, _vault) = crate::eval::build_index(&fx.world, root)?;
    Ok((index, fx.questions.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embed::PLAIN;

    fn fixture(seed: i64) -> Fixture {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("fixtures").join(format!("knowledge_eval_{seed}.json"));
        serde_json::from_str(&std::fs::read_to_string(path).expect("fixture present")).expect("fixture parses")
    }

    struct Scratch(std::path::PathBuf);
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn prepared(seed: i64) -> (KnowledgeIndex, Vec<Question>, Scratch) {
        let fx = fixture(seed);
        let dir = Scratch(std::env::temp_dir().join(format!("cart-gate-{seed}-{}-{:?}", std::process::id(), std::thread::current().id()).replace(['(', ')', ' '], "")));
        let _ = std::fs::remove_dir_all(&dir.0);
        let (i, q) = index_for_fixture(&fx, &dir.0).expect("index builds");
        (i, q, dir)
    }

    /// Protects the gate's pass side: the gold-biased control (which cheats) must be ADOPTED on every
    /// fixture world, with a lift above the bar and no exact-name loss above the allowance. If the
    /// gate cannot adopt this, it cannot adopt anything and RK-9's verdict means nothing.
    #[test]
    fn the_gold_biased_control_is_adopted_on_all_three_worlds() {
        for seed in crate::eval::SEEDS {
            let (index, qs, _d) = prepared(seed);
            let set = gold_biased_set(&index, &qs, 64);
            let r = evaluate(&index, &qs, &set, None);
            assert_eq!(r.fallbacks, 0, "seed {seed}");
            assert!(r.miss_subset > 0, "seed {seed}: the lexical retriever must miss something for a lift to exist");
            assert!(r.lift_points >= ADOPT_MIN_LIFT_POINTS, "seed {seed}: lift {}", r.lift_points);
            assert_eq!(r.verdict, Verdict::Adopt, "seed {seed}: {r:?}");
        }
    }

    /// Protects the gate's fail side: a random embedder must NOT be adopted on any fixture world (it
    /// lifts nothing and mixes noise into the exact-name list). If this passes by adopting, the gate
    /// adopts noise.
    #[test]
    fn the_random_control_is_not_adopted_on_any_world() {
        for seed in crate::eval::SEEDS {
            let (index, qs, _d) = prepared(seed);
            let mut cache = VectorCache::new("scratch", 64);
            let set = embed_all(&RandomEmbedder { dim: 64, seed: 7 }, &PLAIN, &index, &qs, &mut cache).unwrap();
            let r = evaluate(&index, &qs, &set, None);
            assert!(matches!(r.verdict, Verdict::NotNeeded { .. }), "seed {seed}: {:?} lift {} loss {}", r.verdict, r.lift_points, r.exact_loss_points);
        }
    }

    /// Protects the incompleteness rule: a vector set missing one unit or one question is `Incomplete`
    /// and carries no lift or loss, never "not needed".
    #[test]
    fn a_vector_set_that_misses_a_unit_or_a_question_decides_nothing() {
        let (index, qs, _d) = prepared(9137);
        let mut set = gold_biased_set(&index, &qs, 32);
        let first = set.units.keys().next().unwrap().clone();
        set.units.remove(&first);
        assert!(matches!(evaluate(&index, &qs, &set, None).verdict, Verdict::Incomplete { .. }));
        let mut set = gold_biased_set(&index, &qs, 32);
        set.queries.remove(&qs[0].id);
        let r = evaluate(&index, &qs, &set, None);
        assert!(matches!(r.verdict, Verdict::Incomplete { .. }));
        assert_eq!(r.lift_points, 0.0);
    }

    /// Protects the cache contract in `embed_all`: embedding the same corpus twice through one cache
    /// calls the embedder zero times the second time and returns identical vectors.
    #[test]
    fn a_second_embedding_pass_is_served_entirely_from_the_cache() {
        use std::cell::Cell;
        struct Counting(Cell<usize>, HashingEmbedder);
        impl Embedder for Counting {
            fn name(&self) -> String {
                self.1.name()
            }
            fn dim(&self) -> usize {
                self.1.dim
            }
            fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, String> {
                self.0.set(self.0.get() + texts.len());
                self.1.embed(texts)
            }
        }
        let (index, qs, _d) = prepared(9137);
        let e = Counting(Cell::new(0), HashingEmbedder { dim: 32 });
        let mut cache = VectorCache::new(&e.name(), 32);
        let a = embed_all(&e, &PLAIN, &index, &qs, &mut cache).unwrap();
        let first = e.0.get();
        assert!(first > index.doc_count(), "every unit and question was embedded once ({first})");
        let b = embed_all(&e, &PLAIN, &index, &qs, &mut cache).unwrap();
        assert_eq!(e.0.get(), first, "the second pass embedded nothing");
        assert_eq!(a, b);
    }

    /// Protects the numeric bar by value: the two thresholds are the milestone's, 10 and 1.
    /// MUTATION: change either constant and this goes red.
    #[test]
    fn the_adoption_thresholds_are_the_milestones() {
        assert_eq!(ADOPT_MIN_LIFT_POINTS, 10.0);
        assert_eq!(ADOPT_MAX_EXACT_LOSS_POINTS, 1.0);
    }

    /// Prints the stand-in rows the report quotes (`--nocapture`); asserts only that the report renders
    /// a verdict line and a row per category with gold.
    #[test]
    fn the_report_renders_a_verdict_and_every_category() {
        let (index, qs, _d) = prepared(24601);
        let mut cache = VectorCache::new("scratch", 128);
        let set = embed_all(&HashingEmbedder { dim: 128 }, &PLAIN, &index, &qs, &mut cache).unwrap();
        let r = evaluate(&index, &qs, &set, None);
        let text = r.render("stand-in");
        assert!(text.contains("Verdict:") && text.contains("| paraphrase |") && text.contains("not measured"), "{text}");
        println!("{text}");
    }
}
