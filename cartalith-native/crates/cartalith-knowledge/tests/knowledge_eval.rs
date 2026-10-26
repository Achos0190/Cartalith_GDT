//! RK-0 and RK-3 on three real generated worlds (`RAG_KNOWLEDGE_BASE_RESEARCH.md` s.8, s.8.1).
//!
//! The committed fixtures `tests/fixtures/knowledge_eval_<seed>.json` hold a REAL world's input (the
//! engine's own civilisation layer, collected by `cartalith-godot`'s `knowledge_bridge.rs`) and the
//! questions generated from it. This file proves, per world:
//!
//! * the fixture is not stale (the vault hash and the questions equal what the generator now makes);
//! * every gold id exists in the corpus, every paraphrase question truly shares no word with its note,
//!   and the set meets RK-0's size (>= 60 questions, >= 7 categories, >= 200 notes);
//! * the metric can fail: a no-retrieval control and a shuffled-id control score ~0;
//! * the substring baseline and the BM25 retriever are measured and printed (run with `--nocapture`);
//! * RK-3's retrieval bars and the privacy/leak checks hold or are reported.
//!
//! Bars are separate tests so a failure names the bar. They assert the milestone's numbers as
//! written; nothing here is tuned to pass (a failed bar is reported by the build report, not fixed
//! by editing a constant).

use std::collections::BTreeSet;
use std::path::PathBuf;

use cartalith_knowledge::eval::*;
use cartalith_knowledge::model::id_matches;
use cartalith_knowledge::search::{KnowledgeIndex, Query};
use cartalith_knowledge::tokenize::content_tokens;
use cartalith_knowledge::vault::VaultIndex;

fn fixture_path(seed: i64) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("fixtures").join(format!("knowledge_eval_{seed}.json"))
}

fn load(seed: i64) -> Fixture {
    let text = std::fs::read_to_string(fixture_path(seed)).unwrap_or_else(|e| panic!("fixture {seed} missing ({e}); regenerate with `cargo test -p cartalith-godot --lib regenerate_knowledge_fixtures -- --ignored`"));
    serde_json::from_str(&text).expect("fixture parses")
}

/// Everything one world's tests need: the fixture, the written scratch vault and both indexes.
struct Ctx {
    fx: Fixture,
    sv: SyntheticVault,
    dir: PathBuf,
    index: KnowledgeIndex,
    vault: VaultIndex,
}

impl Drop for Ctx {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

fn ctx(seed: i64) -> Ctx {
    let fx = load(seed);
    let sv = build_vault(&fx.world, MIN_NOTES);
    let dir = std::env::temp_dir().join(format!("cartalith-knowledge-eval-{seed}-{}-{:?}", std::process::id(), std::thread::current().id()).replace(['(', ')', ' '], ""));
    let _ = std::fs::remove_dir_all(&dir);
    sv.write_to(&dir).expect("scratch vault is writable");
    let (index, vault) = build_index(&fx.world, &dir).expect("index builds");
    Ctx { fx, sv, dir, index, vault }
}

fn bm25(c: &Ctx) -> Bm25Retriever<'_> {
    Bm25Retriever { index: &c.index, label: "BM25 over both sources".to_string() }
}

/// Protects against a stale fixture: the committed vault hash and question list equal what the
/// generator makes from the committed world today. A generator edit that is not re-committed with its
/// fixtures fails here, by name.
#[test]
fn fixtures_are_current_for_all_three_seeds() {
    for seed in SEEDS {
        let c = ctx(seed);
        assert_eq!(c.fx.seed, seed);
        assert_eq!(c.fx.vault_hash, c.sv.hash(), "seed {seed}: the vault generator changed; regenerate the fixtures");
        assert_eq!(c.fx.vault_notes, c.sv.notes.len(), "seed {seed}");
        assert_eq!(c.fx.questions, build_questions(&c.fx.world, &c.sv), "seed {seed}: the question generator changed; regenerate the fixtures");
    }
}

/// Protects RK-0's size bar and the "every gold id exists" bar on all three worlds: >= 60 questions,
/// >= 7 categories each with questions, >= 200 notes, no empty ids, and every gold id resolves to a real
/// world document or a real chunk (a hand-checked gold that is wrong would score every system 0).
#[test]
fn every_world_meets_the_size_bar_and_every_gold_id_exists() {
    for seed in SEEDS {
        let c = ctx(seed);
        let qs = &c.fx.questions;
        assert!(qs.len() >= MIN_QUESTIONS, "seed {seed}: {} questions", qs.len());
        assert!(c.sv.notes.len() >= MIN_NOTES, "seed {seed}: {} notes", c.sv.notes.len());
        let cats: BTreeSet<&str> = qs.iter().map(|q| q.category.as_str()).collect();
        assert!(cats.len() >= 7, "seed {seed}: categories {cats:?}");
        for cat in CATEGORIES {
            assert!(qs.iter().any(|q| q.category == cat), "seed {seed}: no {cat} question");
        }
        let ids: Vec<&str> = c.index.ids().collect();
        for q in qs {
            assert!(!q.text.trim().is_empty() && !q.substring_query.trim().is_empty(), "{}", q.id);
            for g in &q.gold {
                assert!(ids.iter().any(|i| id_matches(g, i)), "seed {seed} {}: gold id {g} is not in the corpus", q.id);
            }
            match q.category.as_str() {
                "unanswerable" | "private" => assert!(q.gold.is_empty(), "{}", q.id),
                _ => assert!(!q.gold.is_empty(), "{}", q.id),
            }
            if q.category == "private" {
                assert!(!q.forbid.is_empty(), "{}", q.id);
            }
        }
        // Question ids are unique.
        let uniq: BTreeSet<&str> = qs.iter().map(|q| q.id.as_str()).collect();
        assert_eq!(uniq.len(), qs.len());
    }
}

/// Protects the paraphrase category's meaning: a paraphrase question shares NO content token with its
/// gold note (title plus text), so a lexical miss on it is a genuine vocabulary miss and not a bug;
/// and all eight hand-written pairs survived the generator's own check.
#[test]
fn paraphrase_questions_share_no_word_with_their_note() {
    for seed in SEEDS {
        let c = ctx(seed);
        assert_eq!(c.sv.truth.paraphrases.len(), 8, "a pair was dropped for sharing a word");
        for p in &c.sv.truth.paraphrases {
            let text = c.sv.notes.iter().find(|(path, _)| *path == p.path).map(|(_, t)| t.clone()).expect("the note exists");
            let note: BTreeSet<String> = content_tokens(&format!("{} {}", p.path, text)).into_iter().collect();
            let shared: Vec<String> = content_tokens(&p.question).into_iter().filter(|t| note.contains(t)).collect();
            assert!(shared.is_empty(), "seed {seed}: question {:?} shares {shared:?} with {}", p.question, p.path);
        }
    }
}

/// Protects the metric's ability to fail (s.8.1 "a metric that cannot fail"): the no-retrieval control
/// scores exactly 0 recall and 0 expected-literal coverage on every answerable category, and the
/// shuffled-id control collapses to a small fraction of the real retriever's recall.
#[test]
fn the_controls_score_zero_and_the_shuffled_index_collapses() {
    for seed in SEEDS {
        let c = ctx(seed);
        let none = score(&NoRetrieval, &c.fx.questions);
        assert_eq!(none.overall_recall10, 0.0, "seed {seed}");
        assert!(none.by_category.values().all(|s| s.expect5 == 0.0 && s.recall10 == 0.0), "seed {seed}");
        let real = score(&bm25(&c), &c.fx.questions);
        let shuffled = score(&ShuffledIndex { index: &c.index }, &c.fx.questions);
        assert!(real.overall_recall10 > 0.5, "seed {seed}: the real retriever must work for the control to mean anything: {}", real.overall_recall10);
        assert!(shuffled.overall_recall10 < real.overall_recall10 * 0.25, "seed {seed}: shuffled {} vs real {}", shuffled.overall_recall10, real.overall_recall10);
        // The unanswerable refusal for no retrieval is trivially 1; the other categories must not be.
        assert_eq!(none.by_category["unanswerable"].refusal, 1.0);
    }
}

/// Measures and prints the baseline and BM25 tables for the report (`--nocapture`), and asserts the
/// substring baseline really ran (non-empty, found something on the exact-name category it can
/// answer) so the comparison is not vacuous.
#[test]
fn baseline_and_bm25_tables_are_measured() {
    for seed in SEEDS {
        let c = ctx(seed);
        let sub = SubstringBaseline::open(&c.dir).expect("the baseline opens the same vault");
        let base = score(&sub, &c.fx.questions);
        let ours = score(&bm25(&c), &c.fx.questions);
        println!("==== seed {seed}: {} questions, {} notes, {} indexed chunks, {} documents ====", c.fx.questions.len(), c.vault.notes_searched(), c.vault.chunk_count(), c.index.doc_count());
        println!("{}", base.table());
        println!("{}", ours.table());
        assert!(base.by_category["exact_name"].hit10 > 0.0, "seed {seed}: the substring baseline found nothing at all; it did not run");
        assert!(ours.n == base.n && ours.n >= MIN_QUESTIONS);
    }
}

/// RK-3 bar: recall@10 on exact-name questions is at least 0.90 on every world.
#[test]
fn bar_exact_name_recall_at_10_is_at_least_0_90() {
    for seed in SEEDS {
        let c = ctx(seed);
        let r = score(&bm25(&c), &c.fx.questions);
        let v = r.by_category["exact_name"].recall10;
        assert!(v >= 0.90, "seed {seed}: exact-name recall@10 {v:.3} < 0.90");
    }
}

/// RK-3 bar: overall recall@10 is at least 0.75 on every world (paraphrase misses included, as the
/// bar is written over the whole set).
#[test]
fn bar_overall_recall_at_10_is_at_least_0_75() {
    for seed in SEEDS {
        let c = ctx(seed);
        let r = score(&bm25(&c), &c.fx.questions);
        assert!(r.overall_recall10 >= 0.75, "seed {seed}: overall recall@10 {:.3} < 0.75 (MRR {:.3})", r.overall_recall10, r.overall_mrr);
    }
}

/// RK-3 bar: BM25 is strictly above the substring baseline on every category the baseline can answer
/// (any category where the baseline's recall@10 is above 0).
#[test]
fn bar_bm25_beats_the_substring_baseline_on_every_category_it_can_answer() {
    for seed in SEEDS {
        let c = ctx(seed);
        let sub = SubstringBaseline::open(&c.dir).unwrap();
        let base = score(&sub, &c.fx.questions);
        let ours = score(&bm25(&c), &c.fx.questions);
        for cat in CATEGORIES {
            let (b, o) = (&base.by_category[cat], &ours.by_category[cat]);
            if b.recall10 > 0.0 {
                assert!(o.recall10 > b.recall10, "seed {seed} {cat}: BM25 {:.3} is not above substring {:.3}", o.recall10, b.recall10);
            }
        }
    }
}

/// Privacy bar: no private note's canary appears in any retrieved text, id or related excerpt, on any
/// world; AND the control that proves the check can fail: the same vault with the privacy markers
/// stripped leaks the canary.
#[test]
fn private_notes_never_leak_and_the_check_can_fail() {
    for seed in SEEDS {
        let c = ctx(seed);
        let r = score(&bm25(&c), &c.fx.questions);
        assert_eq!(r.leaks, 0, "seed {seed}: a private canary was retrieved");
        // Every private note is excluded and counted out.
        let secrets: Vec<&(String, String)> = c.sv.truth.settlements.iter().filter_map(|s| s.secret.as_ref()).collect();
        assert!(secrets.len() >= 5, "seed {seed}: the fixture must carry private notes, got {}", secrets.len());
        for (path, _) in &secrets {
            assert!(c.vault.is_excluded(path), "seed {seed}: {path} is not excluded");
        }
        // Positive control: strip every marker and the canaries ARE retrievable.
        let dir2 = c.dir.with_extension("leaky");
        let _ = std::fs::remove_dir_all(&dir2);
        let mut leaky = c.sv.clone();
        for (p, t) in leaky.notes.iter_mut() {
            *t = t.replace("dm: true", "dm: no").replace("secret", "plain").replace("#dm-only", "#open");
            let _ = p;
        }
        leaky.notes = leaky.notes.into_iter().map(|(p, t)| (p.replace("DM/", "Open/"), t)).collect();
        leaky.write_to(&dir2).unwrap();
        let (idx2, _) = build_index(&c.fx.world, &dir2).unwrap();
        let r2 = score(&Bm25Retriever { index: &idx2, label: "no markers".into() }, &c.fx.questions);
        assert!(r2.leaks >= 3, "seed {seed}: the control must leak (got {}), or the leak check cannot fail", r2.leaks);
        let _ = std::fs::remove_dir_all(&dir2);
    }
}

/// Determinism bar: the ordered top-10 ids of every question, hashed, are identical across 100 repeated
/// runs on one index AND across two independently built indexes (the second built from a freshly
/// written vault), the in-process stand-in for "two processes". (`tests/two_process.rs` runs the real
/// two-process check.)
#[test]
fn rankings_are_identical_across_100_runs_and_two_builds() {
    let c = ctx(SEEDS[0]);
    let digest = |idx: &KnowledgeIndex| -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        for q in &c.fx.questions {
            let mut query = Query::new(&q.text);
            query.limit = 10;
            for id in idx.search(&query).ids() {
                for b in id.bytes().chain([0]) {
                    h ^= u64::from(b);
                    h = h.wrapping_mul(0x0000_0100_0000_01b3);
                }
            }
        }
        h
    };
    let first = digest(&c.index);
    assert_ne!(first, 0xcbf2_9ce4_8422_2325, "an empty digest would prove nothing");
    for _ in 0..100 {
        assert_eq!(digest(&c.index), first);
    }
    let dir2 = c.dir.with_extension("again");
    let _ = std::fs::remove_dir_all(&dir2);
    c.sv.write_to(&dir2).unwrap();
    let (idx2, _) = build_index(&c.fx.world, &dir2).unwrap();
    assert_eq!(digest(&idx2), first, "a second build must rank identically");
    let _ = std::fs::remove_dir_all(&dir2);
}

/// Authority bar (s.5.1, Ruling DM item 3): a conflict question shows BOTH a WORLD DATA and a VAULT NOTE
/// hit with their own figures and never a merged one; a figure question's figure comes from WORLD DATA.
#[test]
fn authority_rules_hold_on_conflict_and_figure_questions() {
    for seed in SEEDS {
        let c = ctx(seed);
        let r = score(&bm25(&c), &c.fx.questions);
        assert!(r.conflict_both_sources >= 0.8, "seed {seed}: conflict shows both sources on {:.2}", r.conflict_both_sources);
        assert!(r.authority_figure_world >= 0.8, "seed {seed}: figure from WORLD DATA on {:.2}", r.authority_figure_world);
        // The merged figure never exists: no excerpt contains both numbers on one line.
        for q in c.fx.questions.iter().filter(|q| q.category == "conflict") {
            let got = bm25(&c).retrieve(q);
            for t in &got.texts {
                for line in t.lines() {
                    let both = q.expect.iter().all(|e| cartalith_knowledge::eval::contains_literal(line, e));
                    assert!(!both, "seed {seed} {}: a line merges both figures: {line}", q.id);
                }
            }
        }
    }
}

/// The digest both the in-process and the two-process checks use: an FNV-1a over every question's
/// ordered top-10 ids and the first hit's score bits, so a ranking AND a score change is caught.
fn digest_of(c: &Ctx) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for q in &c.fx.questions {
        let mut query = Query::new(&q.text);
        query.limit = 10;
        let o = c.index.search(&query);
        for id in o.ids() {
            for b in id.bytes().chain([0]) {
                h ^= u64::from(b);
                h = h.wrapping_mul(0x0000_0100_0000_01b3);
            }
        }
        for hit in &o.hits {
            for b in hit.score.to_bits().to_le_bytes() {
                h ^= u64::from(b);
                h = h.wrapping_mul(0x0000_0100_0000_01b3);
            }
        }
    }
    h
}

/// Helper "test" the parent below re-invokes in a CHILD process (`CK_CHILD=1`); when run normally it
/// does nothing. It prints the digest of seed 0 on one line.
#[test]
fn digest_child() {
    if std::env::var("CK_CHILD").is_err() {
        return;
    }
    let c = ctx(SEEDS[0]);
    println!("CK_DIGEST={:016x}", digest_of(&c));
}

/// Determinism bar, the real two-process half: a child process of this test binary builds its own
/// vault, index and ranking and prints a digest, which must equal this process's. A stale child (the
/// test binary is the one just built) cannot disagree by construction; a hash-order dependence would.
#[test]
fn rankings_are_identical_across_two_processes() {
    let c = ctx(SEEDS[0]);
    let mine = format!("{:016x}", digest_of(&c));
    let exe = std::env::current_exe().expect("the test binary path");
    let out = std::process::Command::new(exe).args(["--exact", "digest_child", "--nocapture", "--test-threads", "1"]).env("CK_CHILD", "1").output().expect("the child runs");
    let text = String::from_utf8_lossy(&out.stdout);
    let theirs = text.split("CK_DIGEST=").nth(1).and_then(|r| r.split_whitespace().next()).unwrap_or_else(|| panic!("the child printed no digest: {text}"));
    assert_eq!(theirs, mine, "two processes ranked differently");
}
