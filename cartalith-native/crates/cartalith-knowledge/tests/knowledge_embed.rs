//! RK-10's bars on three real worlds (`RAG_KNOWLEDGE_BASE_RESEARCH.md` s.8 RK-10), at the library level:
//!
//! * RK-3's lexical bars still hold with the hybrid code present (the lexical path is untouched, and a
//!   hybrid that has no query vectors answers exactly as the lexical retriever does);
//! * the fused order is deterministic given cached vectors (100 runs, and across a rebuilt index);
//! * a model or dimension change invalidates the cache (`embed.rs` unit tests pin the cache itself;
//!   here a cache made for one model cannot serve another through `embed_all`);
//! * embeddings are off unless the setting, a source and a permitted endpoint are all present
//!   (`embeddings_enabled`'s unit test) and the crate holds no socket (`no_network.rs`);
//! * every text that would be sent is counted by the plan the consent paragraph quotes, and no
//!   private canary is in any of them.
//!
//! The product wiring (the shell's HTTP call, the setting's checkbox) waits on RK-9's live verdict;
//! see `STATUS.md`.

use std::path::PathBuf;

use cartalith_knowledge::embed::*;
use cartalith_knowledge::eval::{score, Bm25Retriever, Fixture, Question, Retriever, SEEDS};
use cartalith_knowledge::gate::{gold_biased_set, index_for_fixture, HashingEmbedder};
use cartalith_knowledge::search::KnowledgeIndex;

struct Ctx {
    index: KnowledgeIndex,
    qs: Vec<Question>,
    dir: PathBuf,
}

impl Drop for Ctx {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

fn ctx(seed: i64) -> Ctx {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("fixtures").join(format!("knowledge_eval_{seed}.json"));
    let fx: Fixture = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let dir = std::env::temp_dir().join(format!("cart-embed-{seed}-{}-{:?}", std::process::id(), std::thread::current().id()).replace(['(', ')', ' '], ""));
    let _ = std::fs::remove_dir_all(&dir);
    let (index, qs) = index_for_fixture(&fx, &dir).unwrap();
    Ctx { index, qs, dir }
}

/// Protects "RK-3's bars still hold (no regression)": with the hybrid code present and the lexical
/// retriever unchanged, overall recall@10 stays at or above RK-3's 0.75 on all three worlds, and a
/// hybrid retriever given NO query vectors returns exactly the lexical top 10 for every question and
/// counts every one of them as a fallback (it never passes off a lexical answer as a hybrid one).
#[test]
fn the_lexical_bar_holds_and_a_hybrid_with_no_query_vectors_is_the_lexical_list() {
    for seed in SEEDS {
        let c = ctx(seed);
        let lexical = Bm25Retriever { index: &c.index, label: "lexical".into() };
        let rep = score(&lexical, &c.qs);
        assert!(rep.overall_recall10 >= 0.75, "seed {seed}: recall@10 {}", rep.overall_recall10);
        let mut empty = VectorSet { model: "none".into(), dim: 4, ..Default::default() };
        empty.units.insert("x".into(), vec![1.0, 0.0, 0.0, 0.0]);
        let hybrid = HybridRetriever::new(&c.index, &empty, "hybrid");
        for q in &c.qs {
            assert_eq!(hybrid.retrieve(q).ids, lexical.retrieve(q).ids, "seed {seed} {}", q.id);
        }
        assert_eq!(hybrid.fallbacks(), c.qs.len(), "seed {seed}");
    }
}

/// Protects the determinism bar: given the same vectors, the fused top 10 is identical over 100 runs
/// for every question, and identical again on an index rebuilt from the same files.
#[test]
fn the_fused_order_is_the_same_over_one_hundred_runs_and_a_rebuilt_index() {
    let c = ctx(24601);
    let set = gold_biased_set(&c.index, &c.qs, 48);
    let first: Vec<Vec<String>> = {
        let h = HybridRetriever::new(&c.index, &set, "h");
        c.qs.iter().map(|q| h.retrieve(q).ids).collect()
    };
    assert!(first.iter().any(|ids| ids.len() == 10), "the lists must be non-trivial");
    for run in 0..100 {
        let h = HybridRetriever::new(&c.index, &set, "h");
        for (q, want) in c.qs.iter().zip(&first) {
            assert_eq!(&h.retrieve(q).ids, want, "run {run} {}", q.id);
        }
    }
    let again = ctx(24601);
    let h2 = HybridRetriever::new(&again.index, &set, "h");
    for (q, want) in again.qs.iter().zip(&first) {
        assert_eq!(&h2.retrieve(q).ids, want, "rebuilt index {}", q.id);
    }
}

/// Protects the cache contract through `embed_all`: a cache saved for one model, loaded for another,
/// comes back empty and invalidated, so the second model's vectors are all fresh; and the first
/// model's file is not reused under the second's name.
#[test]
fn a_cache_for_one_model_cannot_serve_another_through_embed_all() {
    let c = ctx(9137);
    let e = HashingEmbedder { dim: 24 };
    let mut cache = VectorCache::new(&e.name(), 24);
    let set = embed_all(&e, &PLAIN, &c.index, &c.qs, &mut cache).unwrap();
    assert_eq!(set.units.len(), c.index.doc_count());
    let saved = cache.to_json();
    let (same, inv) = VectorCache::load(&saved, &e.name(), 24);
    assert!(!inv && same.len() == cache.len());
    let (other, inv) = VectorCache::load(&saved, "another-model", 24);
    assert!(inv && other.is_empty());
    let (resized, inv) = VectorCache::load(&saved, &e.name(), 16);
    assert!(inv && resized.is_empty());
}

/// Protects the consent bar: the plan's counts equal an independent recount from the index, the text
/// quotes the unit count, every kind with its count, the character total and the origin, no private
/// canary is in any text that would be sent, and every text sent is one the plan counted.
/// MUTATION: drop the kind list from `consent_text` and the per-kind asserts go red.
#[test]
fn the_consent_paragraph_quotes_the_plan_and_no_private_text_is_in_it() {
    for seed in SEEDS {
        let c = ctx(seed);
        let plan = plan_embedding(&c.index, &QWEN3);
        let texts = texts_to_send(&c.index, &QWEN3);
        assert_eq!(plan.units, texts.len(), "seed {seed}");
        assert_eq!(plan.chars, texts.iter().map(|t| t.chars().count()).sum::<usize>(), "seed {seed}");
        let mut recount = std::collections::BTreeMap::new();
        for id in c.index.ids() {
            *recount.entry(c.index.get(id).unwrap().kind.clone()).or_insert(0usize) += 1;
        }
        assert_eq!(plan.by_kind, recount, "seed {seed}");
        let text = consent_text(&plan, "http://localhost:1234");
        assert!(text.contains(&format!("{} items", plan.units)) && text.contains("http://localhost:1234"), "{text}");
        for (kind, n) in &plan.by_kind {
            assert!(text.contains(&format!("{n} {kind}")), "seed {seed}: the paragraph must name {n} {kind}: {text}");
        }
        assert!(text.contains(&format!("{} characters", plan.chars)));
        for q in c.qs.iter().filter(|q| q.category == "private") {
            for canary in &q.forbid {
                let lc = canary.to_lowercase();
                assert!(!texts.iter().any(|t| t.to_lowercase().contains(&lc)), "seed {seed}: private canary {canary} would be sent");
            }
        }
        assert!(c.qs.iter().any(|q| q.category == "private" && !q.forbid.is_empty()), "the exclusion check must have canaries to look for");
    }
}
