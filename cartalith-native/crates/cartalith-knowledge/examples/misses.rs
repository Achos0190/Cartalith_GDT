//! Developer aid: prints, per question, the gold ids, whether each expected literal reached the top-5
//! excerpts, and the ranked ids. `cargo run -p cartalith-knowledge --example misses -- <seed> [category]`.
use cartalith_knowledge::eval::*;
use cartalith_knowledge::model::id_matches;
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let seed: i64 = a[1].parse().unwrap();
    let cat = a.get(2).cloned();
    let text = std::fs::read_to_string(format!("{}/tests/fixtures/knowledge_eval_{seed}.json", env!("CARGO_MANIFEST_DIR"))).unwrap();
    let fx: Fixture = serde_json::from_str(&text).unwrap();
    let sv = build_vault(&fx.world, MIN_NOTES);
    let dir = std::env::temp_dir().join(format!("ck-misses-{seed}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    sv.write_to(&dir).unwrap();
    let (idx, _) = build_index(&fx.world, &dir).unwrap();
    let r = Bm25Retriever { index: &idx, label: String::new() };
    for q in fx.questions.iter().filter(|q| cat.as_ref().is_none_or(|c| *c == q.category)) {
        let got = r.retrieve(q);
        let ctx5 = got.texts.iter().take(5).cloned().collect::<Vec<_>>().join("\n");
        let missing: Vec<&String> = q.expect.iter().filter(|e| !contains_literal(&ctx5, e)).collect();
        let ranks: Vec<String> = got.ids.iter().take(6).map(|i| if q.gold.iter().any(|g| id_matches(g, i)) { format!("*{i}") } else { i.clone() }).collect();
        println!("{} [{}] {:?}\n   gold {:?} missing_expect {:?}\n   top {:?}", q.id, q.category, q.text, q.gold, missing, ranks);
    }
    let _ = std::fs::remove_dir_all(&dir);
}
