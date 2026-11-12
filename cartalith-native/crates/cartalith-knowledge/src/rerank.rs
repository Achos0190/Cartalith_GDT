//! RK-11: a reranker and LLM-written chunk context, each adopted only against a measured lift
//! (`RAG_KNOWLEDGE_BASE_RESEARCH.md` s.7.3, s.8 RK-11).
//!
//! Both are *conditional on RK-9* and both are measured on the same subset RK-9 defines (the
//! paraphrase category plus every answerable question the lexical retriever missed, see
//! [`crate::gate`]). The bar in s.8 is "adopt each only with a measured lift on the RK-9 miss subset that
//! exceeds its latency/cost (stated), and keep the deterministic header as the fallback". This module
//! turns that sentence into code with the numbers named:
//!
//! * a **lift** of at least [`crate::gate::ADOPT_MIN_LIFT_POINTS`] points on the subset, as for RK-9;
//! * **no more than** [`crate::gate::ADOPT_MAX_EXACT_LOSS_POINTS`] point lost on exact-name questions;
//! * a **query-time cost** (reranker only) of at most [`MAX_ADDED_QUERY_MS`] milliseconds added to a
//!   search, measured by the caller (this module reads no clock); an unmeasured cost is
//!   [`RerankVerdict::CostNotMeasured`], never "free";
//! * for LLM-written context, the **one-off index cost** in tokens is *stated* in the report
//!   ([`ContextReport::index_tokens`]) and is not itself a gate, because the context is built once
//!   per chunk and cached by content hash; the rule is that a unit whose writer failed keeps its
//!   deterministic header ([`KnowledgeIndex::build_with_extra_headers`] with no entry for it), so a
//!   writer that fails everywhere is **byte-identical** to no writer (a test pins it).
//!
//! # What is real and what is a control
//!
//! The reranking *models* (Qwen3-Reranker-0.6B, a bge reranker through a llama.cpp server) and the
//! chunk-context *LLM* need a live model, which this build must not load. What is built and tested:
//! the [`Reranker`] and [`ContextWriter`] traits, the reordering and the index hook, the adoption
//! rules, and three controls for each that prove the rules can adopt and can refuse - an identity
//! (must equal the baseline exactly), a cheating oracle (must be adopted) and a harmful one (must
//! not). A live run supplies a real [`Reranker`] through a thin shell adapter and reports through
//! [`evaluate_rerank`] / [`evaluate_context`] unchanged.
//!
//! # What it must never do
//!
//! Read a clock, open a socket, decide on an unmeasured cost, or let a failed writer leave a unit with
//! a blank or partial header.

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use crate::embed::{HybridRetriever, VectorSet, FUSION_DEPTH, HYBRID_TOP};
use crate::eval::{score, Bm25Retriever, Question, Retrieved, Retriever};
use crate::gate::{ADOPT_MAX_EXACT_LOSS_POINTS, ADOPT_MIN_LIFT_POINTS};
use crate::model::{id_matches, Document};
use crate::search::KnowledgeIndex;

/// The most a reranker may add to one search, in milliseconds. An assistant turn already waits on a
/// model call of several seconds (`REPORT.md` s.8: 2.7 to 19 s per answer on this machine), so
/// half a second is under a tenth of a typical turn. Labelled judgement for the owner to revise; the
/// RK-11 bar leaves the figure open ("stated").
pub const MAX_ADDED_QUERY_MS: f64 = 500.0;

/// Scores candidate passages against a query.
pub trait Reranker {
    /// The model id for the report.
    fn name(&self) -> String;
    /// A relevance score for each candidate (higher is better), in the candidates' order.
    fn score(&self, query: &str, candidates: &[(String, String)]) -> Result<Vec<f64>, String>;
}

/// Writes a short situating context for a unit (Anthropic's contextual-retrieval idea [cited, s.1]),
/// which is indexed in the header field.
pub trait ContextWriter {
    /// The model id for the report.
    fn name(&self) -> String;
    /// The context for `d`, or an error (the unit then keeps its deterministic header).
    fn context(&self, d: &Document) -> Result<String, String>;
}

/// Reorders `ids` (best first) by `scores` (higher first); equal scores keep their incoming order, so
/// a constant scorer is the identity. `ids` and `scores` must be the same length.
pub fn reorder(ids: &[String], scores: &[f64]) -> Vec<String> {
    let mut idx: Vec<usize> = (0..ids.len().min(scores.len())).collect();
    idx.sort_by(|a, b| scores[*b].total_cmp(&scores[*a]).then(a.cmp(b)));
    idx.into_iter().map(|i| ids[i].clone()).collect()
}

/// A retriever that takes the hybrid's (or the lexical list's) top [`FUSION_DEPTH`] candidates,
/// reranks them and returns the top [`HYBRID_TOP`].
pub struct RerankRetriever<'a> {
    /// The candidate source: a hybrid over the index and its vectors.
    pub base: &'a HybridRetriever<'a>,
    /// The reranker.
    pub reranker: &'a dyn Reranker,
    /// A label for the report.
    pub label: String,
}

impl Retriever for RerankRetriever<'_> {
    fn name(&self) -> String {
        self.label.clone()
    }

    fn retrieve(&self, q: &Question) -> Retrieved {
        let cands = self.base.fused_ids(q, FUSION_DEPTH);
        let texts: Vec<(String, String)> = cands.iter().filter_map(|id| self.base.index.get(id).map(|d| (id.clone(), crate::embed::embedding_text(d)))).collect();
        let order = match self.reranker.score(&q.text, &texts) {
            Ok(s) if s.len() == texts.len() => reorder(&texts.iter().map(|t| t.0.clone()).collect::<Vec<_>>(), &s),
            // A reranker that fails or answers the wrong number of scores leaves the fused order: the
            // fallback is the unreranked list, never a partial reorder.
            _ => cands,
        };
        self.base.retrieved_for(q, &order[..order.len().min(HYBRID_TOP)])
    }
}

/// What a reranker or a writer cost, supplied by the caller from a live run.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Cost {
    /// Milliseconds added to one search (reranker), or `None` if not measured.
    pub added_query_ms: Option<f64>,
    /// Tokens spent once per unit at index time (writer), or `None` if not measured.
    pub index_tokens: Option<u64>,
}

/// The reranker's decision.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum RerankVerdict {
    /// Lift, exact-name loss and cost all inside the bars.
    Adopt,
    /// A bar failed.
    NotNeeded {
        /// The failed bars.
        why: Vec<String>,
    },
    /// Quality passed but the cost was not measured, so nothing is decided.
    CostNotMeasured,
}

/// The measured effect of a reranker over a baseline retriever.
#[derive(Debug, Clone, Serialize)]
pub struct RerankReport {
    /// The reranker's id.
    pub model: String,
    /// Questions in the paraphrase / miss subset.
    pub subset: usize,
    /// Baseline recall@10 on the subset.
    pub base_r10: f64,
    /// Reranked recall@10 on the subset.
    pub reranked_r10: f64,
    /// Lift in points.
    pub lift_points: f64,
    /// Exact-name loss in points (positive is a loss).
    pub exact_loss_points: f64,
    /// The cost supplied.
    pub cost: Cost,
    /// The decision.
    pub verdict: RerankVerdict,
}

fn r10_per_question(r: &dyn Retriever, qs: &[Question]) -> BTreeMap<String, f64> {
    qs.iter().filter(|q| !q.gold.is_empty()).map(|q| (q.id.clone(), score(r, std::slice::from_ref(q)).overall_recall10)).collect()
}

fn mean(ids: &BTreeSet<String>, per_q: &BTreeMap<String, f64>) -> f64 {
    let v: Vec<f64> = ids.iter().filter_map(|i| per_q.get(i)).copied().collect();
    if v.is_empty() { 0.0 } else { v.iter().sum::<f64>() / v.len() as f64 }
}

/// The RK-9 subset (paraphrase plus lexical misses), computed against the **lexical** retriever so the
/// same questions are used for every step-2 trial.
fn subset(qs: &[Question], index: &KnowledgeIndex) -> BTreeSet<String> {
    let lex = Bm25Retriever { index, label: "lexical".into() };
    let lq = r10_per_question(&lex, qs);
    qs.iter().filter(|q| !q.gold.is_empty() && (q.category == "paraphrase" || lq.get(&q.id).is_some_and(|r| *r < 1.0))).map(|q| q.id.clone()).collect()
}

fn exact_name_r10(r: &dyn Retriever, qs: &[Question]) -> f64 {
    let ex: Vec<Question> = qs.iter().filter(|q| q.category == "exact_name").cloned().collect();
    score(r, &ex).by_category.get("exact_name").map_or(0.0, |c| c.recall10)
}

/// Measures `reranker` over `vectors` on `qs` against the unreranked hybrid (module doc).
pub fn evaluate_rerank(index: &KnowledgeIndex, qs: &[Question], vectors: &VectorSet, reranker: &dyn Reranker, cost: Cost) -> RerankReport {
    let base = HybridRetriever::new(index, vectors, "hybrid");
    let reranked = RerankRetriever { base: &base, reranker, label: reranker.name() };
    let sub = subset(qs, index);
    let (bq, rq) = (r10_per_question(&base, qs), r10_per_question(&reranked, qs));
    let (b, r) = (mean(&sub, &bq), mean(&sub, &rq));
    let lift = (r - b) * 100.0;
    let loss = (exact_name_r10(&base, qs) - exact_name_r10(&reranked, qs)) * 100.0;
    let mut why = Vec::new();
    if sub.is_empty() {
        why.push("the lexical retriever missed nothing, so there is nothing to lift".to_string());
    } else if lift < ADOPT_MIN_LIFT_POINTS {
        why.push(format!("lift on the paraphrase/miss subset is {lift:.1} points, below the {ADOPT_MIN_LIFT_POINTS} required"));
    }
    if loss > ADOPT_MAX_EXACT_LOSS_POINTS {
        why.push(format!("exact-name recall@10 falls {loss:.1} points, above the {ADOPT_MAX_EXACT_LOSS_POINTS} allowed"));
    }
    if let Some(ms) = cost.added_query_ms {
        if ms > MAX_ADDED_QUERY_MS {
            why.push(format!("adds {ms:.0} ms per search, above the {MAX_ADDED_QUERY_MS} allowed"));
        }
    }
    let verdict = if !why.is_empty() {
        RerankVerdict::NotNeeded { why }
    } else if cost.added_query_ms.is_none() {
        RerankVerdict::CostNotMeasured
    } else {
        RerankVerdict::Adopt
    };
    RerankReport { model: reranker.name(), subset: sub.len(), base_r10: b, reranked_r10: r, lift_points: lift, exact_loss_points: loss, cost, verdict }
}

/// The contexts a writer produced, and how many units it failed on.
#[derive(Debug, Clone, Default)]
pub struct Contexts {
    /// Unit id to context. Absent for a unit the writer failed on or returned nothing for.
    pub by_id: BTreeMap<String, String>,
    /// Units the writer failed on (they keep their deterministic header).
    pub failures: usize,
}

/// Runs `writer` over every unit of `index`. A failure or an all-whitespace reply leaves the unit out
/// of the map, so [`KnowledgeIndex::build_with_extra_headers`] keeps its deterministic header.
pub fn write_contexts(index: &KnowledgeIndex, writer: &dyn ContextWriter) -> Contexts {
    let mut out = Contexts::default();
    let mut ids: Vec<&str> = index.ids().collect();
    ids.sort_unstable();
    for id in ids {
        let Some(d) = index.get(id) else { continue };
        match writer.context(d) {
            Ok(t) if !t.trim().is_empty() => {
                out.by_id.insert(id.to_string(), t.trim().to_string());
            }
            _ => out.failures += 1,
        }
    }
    out
}

/// The chunk-context decision.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum ContextVerdict {
    /// Lift and exact-name loss inside the bars.
    Adopt,
    /// A bar failed.
    NotNeeded {
        /// The failed bars.
        why: Vec<String>,
    },
}

/// The measured effect of LLM-written chunk context over the deterministic header.
#[derive(Debug, Clone, Serialize)]
pub struct ContextReport {
    /// The writer's id.
    pub model: String,
    /// Units that got a context.
    pub written: usize,
    /// Units that kept the deterministic header.
    pub fallbacks: usize,
    /// Questions in the paraphrase / miss subset.
    pub subset: usize,
    /// Lexical recall@10 on the subset with the deterministic header.
    pub base_r10: f64,
    /// ... with the written context.
    pub context_r10: f64,
    /// Lift in points.
    pub lift_points: f64,
    /// Exact-name loss in points.
    pub exact_loss_points: f64,
    /// Tokens spent once at index time, if the caller measured them (stated, not gated).
    pub index_tokens: Option<u64>,
    /// The decision.
    pub verdict: ContextVerdict,
}

/// Measures a writer: the lexical retriever over the index rebuilt with the contexts against the
/// same retriever over the deterministic index. `rebuilt` is that second index (the caller builds it
/// from the world and the vault with [`KnowledgeIndex::build_with_extra_headers`]).
pub fn evaluate_context(base: &KnowledgeIndex, rebuilt: &KnowledgeIndex, qs: &[Question], writer_name: &str, contexts: &Contexts, index_tokens: Option<u64>) -> ContextReport {
    let (rb, rc) = (Bm25Retriever { index: base, label: "deterministic".into() }, Bm25Retriever { index: rebuilt, label: "contextual".into() });
    let sub = subset(qs, base);
    let (bq, cq) = (r10_per_question(&rb, qs), r10_per_question(&rc, qs));
    let (b, c) = (mean(&sub, &bq), mean(&sub, &cq));
    let lift = (c - b) * 100.0;
    let loss = (exact_name_r10(&rb, qs) - exact_name_r10(&rc, qs)) * 100.0;
    let mut why = Vec::new();
    if sub.is_empty() {
        why.push("the lexical retriever missed nothing, so there is nothing to lift".to_string());
    } else if lift < ADOPT_MIN_LIFT_POINTS {
        why.push(format!("lift on the paraphrase/miss subset is {lift:.1} points, below the {ADOPT_MIN_LIFT_POINTS} required"));
    }
    if loss > ADOPT_MAX_EXACT_LOSS_POINTS {
        why.push(format!("exact-name recall@10 falls {loss:.1} points, above the {ADOPT_MAX_EXACT_LOSS_POINTS} allowed"));
    }
    ContextReport {
        model: writer_name.to_string(),
        written: contexts.by_id.len(),
        fallbacks: contexts.failures,
        subset: sub.len(),
        base_r10: b,
        context_r10: c,
        lift_points: lift,
        exact_loss_points: loss,
        index_tokens,
        verdict: if why.is_empty() { ContextVerdict::Adopt } else { ContextVerdict::NotNeeded { why } },
    }
}

// ----------------------------------------------------------------------------------- controls

/// Identity control: scores by negative incoming rank, so the order must not change.
pub struct IdentityReranker;

impl Reranker for IdentityReranker {
    fn name(&self) -> String {
        "control:identity".into()
    }
    fn score(&self, _q: &str, c: &[(String, String)]) -> Result<Vec<f64>, String> {
        Ok((0..c.len()).map(|i| -(i as f64)).collect())
    }
}

/// Positive control: scores a gold candidate 1 and every other 0 (it cheats from the question list).
pub struct GoldReranker {
    /// The questions whose gold it may read, by text.
    pub questions: Vec<Question>,
}

impl Reranker for GoldReranker {
    fn name(&self) -> String {
        "control:gold".into()
    }
    fn score(&self, query: &str, c: &[(String, String)]) -> Result<Vec<f64>, String> {
        let gold: Vec<&String> = self.questions.iter().filter(|q| q.text == query).flat_map(|q| q.gold.iter()).collect();
        Ok(c.iter().map(|(id, _)| if gold.iter().any(|g| id_matches(g, id)) { 1.0 } else { 0.0 }).collect())
    }
}

/// Negative control: reverses the candidates, so the best are last. Must not be adopted.
pub struct ReverseReranker;

impl Reranker for ReverseReranker {
    fn name(&self) -> String {
        "control:reverse".into()
    }
    fn score(&self, _q: &str, c: &[(String, String)]) -> Result<Vec<f64>, String> {
        Ok((0..c.len()).map(|i| i as f64).collect())
    }
}

/// A writer that always fails: the fallback control.
pub struct FailingWriter;

impl ContextWriter for FailingWriter {
    fn name(&self) -> String {
        "control:failing".into()
    }
    fn context(&self, _d: &Document) -> Result<String, String> {
        Err("the model is not available".into())
    }
}

/// Positive control: writes, for every unit that is the gold of a paraphrase question, that question's
/// words (it cheats). Must be adopted.
pub struct QuestionWordsWriter {
    /// The questions to read.
    pub questions: Vec<Question>,
}

impl ContextWriter for QuestionWordsWriter {
    fn name(&self) -> String {
        "control:question-words".into()
    }
    fn context(&self, d: &Document) -> Result<String, String> {
        let words: Vec<&str> = self.questions.iter().filter(|q| q.category == "paraphrase" && q.gold.iter().any(|g| id_matches(g, &d.id))).map(|q| q.text.as_str()).collect();
        if words.is_empty() { Err("no context for this unit".into()) } else { Ok(words.join(" ")) }
    }
}

/// Negative control: the same boilerplate sentence on every unit. Adds a term to every document, so
/// it lifts nothing. Must not be adopted.
pub struct BoilerplateWriter;

impl ContextWriter for BoilerplateWriter {
    fn name(&self) -> String {
        "control:boilerplate".into()
    }
    fn context(&self, _d: &Document) -> Result<String, String> {
        Ok("This passage belongs to the shared chronicle of the realm.".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embed::{embed_all, VectorCache, PLAIN};
    use crate::gate::{gold_biased_set, index_for_fixture, HashingEmbedder};

    fn fixture(seed: i64) -> crate::eval::Fixture {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("fixtures").join(format!("knowledge_eval_{seed}.json"));
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
    }

    struct Scratch(std::path::PathBuf);
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn prepared(seed: i64) -> (crate::eval::Fixture, KnowledgeIndex, Vec<Question>, Scratch) {
        let fx = fixture(seed);
        let dir = Scratch(std::env::temp_dir().join(format!("cart-rerank-{seed}-{}-{:?}", std::process::id(), std::thread::current().id()).replace(['(', ')', ' '], "")));
        let _ = std::fs::remove_dir_all(&dir.0);
        let (i, q) = index_for_fixture(&fx, &dir.0).unwrap();
        (fx, i, q, dir)
    }

    const MEASURED: Cost = Cost { added_query_ms: Some(120.0), index_tokens: None };

    /// Protects `reorder`: higher score first, equal scores keep incoming order (so a constant scorer is
    /// the identity), and a longer score list than ids cannot index out of range.
    #[test]
    fn reorder_is_stable_and_bounded() {
        let ids: Vec<String> = ["a", "b", "c"].iter().map(|s| s.to_string()).collect();
        assert_eq!(reorder(&ids, &[0.1, 0.9, 0.5]), vec!["b", "c", "a"]);
        assert_eq!(reorder(&ids, &[1.0, 1.0, 1.0]), vec!["a", "b", "c"]);
        assert_eq!(reorder(&ids, &[0.0, 1.0, 2.0, 3.0]), vec!["c", "b", "a"]);
    }

    /// Protects the reranker's fallback: the identity reranker leaves the hybrid's top 10 EXACTLY as it
    /// was (ids and order), on all three worlds.
    #[test]
    fn the_identity_reranker_equals_the_unreranked_hybrid() {
        for seed in crate::eval::SEEDS {
            let (_fx, index, qs, _d) = prepared(seed);
            let set = gold_biased_set(&index, &qs, 32);
            let base = HybridRetriever::new(&index, &set, "hybrid");
            let rr = RerankRetriever { base: &base, reranker: &IdentityReranker, label: "id".into() };
            for q in &qs {
                assert_eq!(base.retrieve(q).ids, rr.retrieve(q).ids, "seed {seed} {}", q.id);
            }
        }
    }

    /// Protects the pass and fail sides of the reranker bar: a cheating reranker over a deliberately
    /// weak dense list is adopted when its cost is measured, the reversing one is not, and an
    /// UNmeasured cost is `CostNotMeasured`, never `Adopt`.
    #[test]
    fn the_gold_reranker_is_adopted_the_reversing_one_is_not_and_an_unmeasured_cost_decides_nothing() {
        let (_fx, index, qs, _d) = prepared(24601);
        let mut cache = VectorCache::new("scratch", 32);
        let weak = embed_all(&HashingEmbedder { dim: 32 }, &PLAIN, &index, &qs, &mut cache).unwrap();
        let gold = GoldReranker { questions: qs.clone() };
        let good = evaluate_rerank(&index, &qs, &weak, &gold, MEASURED);
        assert!(good.lift_points >= ADOPT_MIN_LIFT_POINTS, "lift {}", good.lift_points);
        assert_eq!(good.verdict, RerankVerdict::Adopt, "{good:?}");
        let rev = evaluate_rerank(&index, &qs, &weak, &ReverseReranker, MEASURED);
        assert!(matches!(rev.verdict, RerankVerdict::NotNeeded { .. }), "{rev:?}");
        let unmeasured = evaluate_rerank(&index, &qs, &weak, &gold, Cost { added_query_ms: None, index_tokens: None });
        assert_eq!(unmeasured.verdict, RerankVerdict::CostNotMeasured);
        let slow = evaluate_rerank(&index, &qs, &weak, &gold, Cost { added_query_ms: Some(MAX_ADDED_QUERY_MS + 1.0), index_tokens: None });
        assert!(matches!(slow.verdict, RerankVerdict::NotNeeded { .. }));
    }

    /// Protects the chunk-context fallback (RK-11 bar: "keep the deterministic header as the
    /// fallback"): a writer that fails on every unit leaves the index's results IDENTICAL to the
    /// deterministic index for every question, and the failure count equals the unit count.
    #[test]
    fn a_failing_writer_changes_nothing() {
        let (fx, index, qs, d) = prepared(9137);
        let ctx = write_contexts(&index, &FailingWriter);
        assert_eq!((ctx.by_id.len(), ctx.failures), (0, index.doc_count()));
        let (rebuilt, _) = crate::eval::build_index_with(&fx.world, &d.0, &ctx.by_id).unwrap();
        let (a, b) = (Bm25Retriever { index: &index, label: "a".into() }, Bm25Retriever { index: &rebuilt, label: "b".into() });
        for q in &qs {
            assert_eq!(a.retrieve(q).ids, b.retrieve(q).ids, "{}", q.id);
        }
    }

    /// Protects the context bar on both sides: the cheating writer (paraphrase question words) lifts the
    /// paraphrase subset past the bar on every world and is adopted; the boilerplate writer is not.
    #[test]
    fn the_question_words_writer_is_adopted_and_the_boilerplate_writer_is_not() {
        for seed in crate::eval::SEEDS {
            let (fx, index, qs, d) = prepared(seed);
            let cheat = write_contexts(&index, &QuestionWordsWriter { questions: qs.clone() });
            assert!(cheat.by_id.len() >= 8 && cheat.failures > 0, "seed {seed}: the writer wrote for the paraphrase golds only");
            let (re, _) = crate::eval::build_index_with(&fx.world, &d.0, &cheat.by_id).unwrap();
            let r = evaluate_context(&index, &re, &qs, "cheat", &cheat, Some(0));
            assert!(r.lift_points >= ADOPT_MIN_LIFT_POINTS, "seed {seed}: lift {}", r.lift_points);
            assert_eq!(r.verdict, ContextVerdict::Adopt, "seed {seed}: {r:?}");
            let flat = write_contexts(&index, &BoilerplateWriter);
            let (re2, _) = crate::eval::build_index_with(&fx.world, &d.0, &flat.by_id).unwrap();
            let r2 = evaluate_context(&index, &re2, &qs, "flat", &flat, None);
            assert!(matches!(r2.verdict, ContextVerdict::NotNeeded { .. }), "seed {seed}: {r2:?}");
        }
    }

    /// Protects the empty-map identity of the index hook: `build_with_extra_headers` with no entries
    /// ranks every question exactly as `build` does.
    #[test]
    fn an_empty_extra_header_map_builds_the_same_index() {
        let (fx, index, qs, d) = prepared(71077345);
        let (again, _) = crate::eval::build_index_with(&fx.world, &d.0, &BTreeMap::new()).unwrap();
        let (a, b) = (Bm25Retriever { index: &index, label: "a".into() }, Bm25Retriever { index: &again, label: "b".into() });
        assert_eq!(index.doc_count(), again.doc_count());
        for q in &qs {
            assert_eq!(a.retrieve(q).ids, b.retrieve(q).ids, "{}", q.id);
        }
    }

    /// Protects the cost bar by value: 500 ms.
    #[test]
    fn the_added_latency_bar_is_half_a_second() {
        assert_eq!(MAX_ADDED_QUERY_MS, 500.0);
    }
}
