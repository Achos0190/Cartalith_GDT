//! RK-9's decision-gate report on the three committed evaluation worlds.
//!
//!   cargo run -p cartalith-knowledge --example rk9_gate -- standin
//!   cargo run -p cartalith-knowledge --example rk9_gate -- live <dir of <label>__<seed>.json>
//!
//! `standin` runs the three controls of `gate` (trigram-hash, gold-biased, random) with no model: it
//! proves the gate can adopt and can refuse, and says nothing about a real embedding model. `live`
//! reads the vector files `godot-project/_llmknow_embed_cli.gd` wrote from a real embeddings endpoint
//! and decides for that model. Opens no socket, reads no clock.

use cartalith_knowledge::embed::{embed_all, VectorCache, VectorSet, PLAIN};
use cartalith_knowledge::eval::{Fixture, SEEDS};
use cartalith_knowledge::gate::{evaluate, gold_biased_set, index_for_fixture, HashingEmbedder, RandomEmbedder, Timing};

fn fixture(seed: i64) -> Fixture {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("fixtures").join(format!("knowledge_eval_{seed}.json"));
    serde_json::from_str(&std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))).expect("fixture parses")
}

fn scratch(seed: i64) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("cart-rk9-{seed}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    d
}

fn main() {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("standin") => {
            for seed in SEEDS {
                let fx = fixture(seed);
                let dir = scratch(seed);
                let (index, qs) = index_for_fixture(&fx, &dir).expect("index builds");
                let mut cache = VectorCache::new("scratch", 128);
                let trigram = embed_all(&HashingEmbedder { dim: 128 }, &PLAIN, &index, &qs, &mut cache).unwrap();
                let mut cache = VectorCache::new("scratch", 128);
                let random = embed_all(&RandomEmbedder { dim: 128, seed: 7 }, &PLAIN, &index, &qs, &mut cache).unwrap();
                let gold = gold_biased_set(&index, &qs, 128);
                for set in [trigram, random, gold] {
                    println!("{}", evaluate(&index, &qs, &set, None).render(&format!("world {seed}: {}", set.model)));
                }
                let _ = std::fs::remove_dir_all(&dir);
            }
        }
        Some("live") => {
            let Some(dir) = args.next() else {
                eprintln!("usage: rk9_gate live <dir>");
                std::process::exit(2);
            };
            let mut files: Vec<_> = std::fs::read_dir(&dir).expect("readable dir").filter_map(Result::ok).map(|e| e.path()).filter(|p| p.extension().is_some_and(|x| x == "json")).collect();
            files.sort();
            if files.is_empty() {
                eprintln!("no .json vector files in {dir}");
                std::process::exit(2);
            }
            for f in files {
                let stem = f.file_stem().unwrap().to_string_lossy().to_string();
                let Some((_, seed)) = stem.rsplit_once("__") else { continue };
                let seed: i64 = seed.parse().unwrap_or_else(|_| panic!("{stem}: the part after `__` must be the seed"));
                #[derive(serde::Deserialize)]
                struct File {
                    vectors: VectorSet,
                    timing: Option<Timing>,
                }
                let file: File = serde_json::from_str(&std::fs::read_to_string(&f).unwrap()).unwrap_or_else(|e| panic!("{}: {e}", f.display()));
                let fx = fixture(seed);
                let dir = scratch(seed);
                let (index, qs) = index_for_fixture(&fx, &dir).expect("index builds");
                println!("{}", evaluate(&index, &qs, &file.vectors, file.timing).render(&format!("world {seed}: {}", file.vectors.model)));
                let _ = std::fs::remove_dir_all(&dir);
            }
        }
        _ => {
            eprintln!("usage: rk9_gate standin | live <dir>");
            std::process::exit(2);
        }
    }
}
