//! RK-11's control trials on the three committed worlds: the reranker and the chunk-context rules run
//! against stand-ins that must be adopted (a cheating oracle), must be refused (a harmful one) or must
//! change nothing (an identity, a failing writer). No model, no socket, no clock.
//!
//!   cargo run -p cartalith-knowledge --example rk11_trials
//!
//! A real reranker or context model plugs in through the `Reranker` / `ContextWriter` traits and
//! `evaluate_rerank` / `evaluate_context`; these rows say only that the rules work.

use cartalith_knowledge::embed::{embed_all, VectorCache, PLAIN};
use cartalith_knowledge::eval::{build_index_with, Fixture, SEEDS};
use cartalith_knowledge::gate::{index_for_fixture, HashingEmbedder};
use cartalith_knowledge::rerank::*;

fn main() {
    println!("| World | Trial | Subset | Base R@10 | Trial R@10 | Lift (points) | Exact-name loss | Cost | Verdict |\n|---|---|---:|---:|---:|---:|---:|---|---|");
    for seed in SEEDS {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("fixtures").join(format!("knowledge_eval_{seed}.json"));
        let fx: Fixture = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        let dir = std::env::temp_dir().join(format!("cart-rk11-{seed}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let (index, qs) = index_for_fixture(&fx, &dir).unwrap();
        let mut cache = VectorCache::new("scratch", 32);
        let weak = embed_all(&HashingEmbedder { dim: 32 }, &PLAIN, &index, &qs, &mut cache).unwrap();
        let measured = Cost { added_query_ms: Some(120.0), index_tokens: None };
        let rerankers: Vec<Box<dyn Reranker>> = vec![Box::new(IdentityReranker), Box::new(GoldReranker { questions: qs.clone() }), Box::new(ReverseReranker)];
        for r in &rerankers {
            let rep = evaluate_rerank(&index, &qs, &weak, r.as_ref(), measured);
            println!("| {seed} | rerank {} | {} | {:.3} | {:.3} | {:+.1} | {:+.1} | 120 ms/search (stated) | {:?} |", rep.model, rep.subset, rep.base_r10, rep.reranked_r10, rep.lift_points, rep.exact_loss_points, rep.verdict);
        }
        let writers: Vec<Box<dyn ContextWriter>> = vec![Box::new(FailingWriter), Box::new(QuestionWordsWriter { questions: qs.clone() }), Box::new(BoilerplateWriter)];
        for w in &writers {
            let ctx = write_contexts(&index, w.as_ref());
            let (rebuilt, _) = build_index_with(&fx.world, &dir, &ctx.by_id).unwrap();
            let rep = evaluate_context(&index, &rebuilt, &qs, &w.name(), &ctx, None);
            println!("| {seed} | context {} ({} written, {} kept deterministic) | {} | {:.3} | {:.3} | {:+.1} | {:+.1} | one-off tokens not measured | {:?} |", rep.model, rep.written, rep.fallbacks, rep.subset, rep.base_r10, rep.context_r10, rep.lift_points, rep.exact_loss_points, rep.verdict);
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
}
