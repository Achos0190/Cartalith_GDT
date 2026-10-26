//! RK-3's cold-build and query-latency bars, and owner answer K9's vault-size measurement
//! (`RAG_KNOWLEDGE_BASE_RESEARCH.md` s.8 RK-3, s.10 K9): a synthetic vault of 2 000, 5 000 and
//! 10 000 notes of about 4 KB, cold-built from disk, with the build time (median, min..max of
//! three), the index memory and the query latency printed.
//!
//! These are MEASUREMENTS and are `#[ignore]`d: a timing assert inside a loaded `cargo test` run
//! proves nothing about the machine. Run them ALONE:
//!
//! `cargo test -p cartalith-knowledge --release --test knowledge_scale -- --ignored --nocapture --test-threads 1`
//!
//! (`--release` is the shipped build; the `dev` profile is `opt-level = 1` and is slower, so the
//! report states which profile each figure came from.)
//!
//! The corpus is synthetic prose built from a 6 000-word invented vocabulary with headings, links and
//! tags, so postings lists and term tables are realistically wide; it contains no real note.
//!
//! What it must never do: assert a time (the milestone's numbers are judged by the report, run alone),
//! or read any directory outside its own scratch vault.

use std::time::Instant;

use cartalith_knowledge::eval::Rng;
use cartalith_knowledge::search::{KnowledgeIndex, Query};
use cartalith_knowledge::vault::{KNOWLEDGE_NOTE_CAP, VaultIndex};
use cartalith_vault::FsVault;

/// A deterministic ~4 KB note: a title, three headed sections of invented-word prose, tags, and links
/// to two other notes.
fn note(i: usize, n: usize, vocab: &[String], rng: &mut Rng) -> (String, String) {
    let title = format!("{} {}", vocab[rng.below(vocab.len())], i);
    let mut t = format!("---\ntype: place\ntags: [{}, {}]\naliases: [{}]\n---\n# {title}\n", vocab[rng.below(vocab.len())], vocab[rng.below(vocab.len())], vocab[rng.below(vocab.len())]);
    for s in 0..3 {
        t.push_str(&format!("\n## {} {}\n", vocab[rng.below(vocab.len())], s));
        for _ in 0..13 {
            let words: Vec<&str> = (0..14).map(|_| vocab[rng.below(vocab.len())].as_str()).collect();
            t.push_str(&words.join(" "));
            t.push_str(". ");
        }
        t.push('\n');
    }
    t.push_str(&format!("\nSee [[Note {}]] and [[Note {}]].\n", rng.below(n), rng.below(n)));
    (format!("Notes/Note {i}.md"), t)
}

fn vocab(rng: &mut Rng) -> Vec<String> {
    let syl = ["ka", "ro", "vel", "mir", "tan", "dor", "sul", "ith", "ben", "or", "ae", "lun", "gar", "shi", "pel", "nu"];
    (0..6000)
        .map(|_| {
            let k = 2 + rng.below(3);
            (0..k).map(|_| syl[rng.below(syl.len())]).collect::<String>()
        })
        .collect()
}

fn build_dir(n: usize) -> (std::path::PathBuf, usize, Vec<String>) {
    let dir = std::env::temp_dir().join(format!("cartalith-knowledge-scale-{n}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("Notes")).unwrap();
    let mut rng = Rng::new(0xC0FFEE ^ n as u64);
    let v = vocab(&mut rng);
    let mut bytes = 0usize;
    for i in 0..n {
        let (p, t) = note(i, n, &v, &mut rng);
        bytes += t.len();
        std::fs::write(dir.join(p), t).unwrap();
    }
    (dir, bytes, v)
}

fn median(v: &mut [f64]) -> f64 {
    v.sort_by(|a, b| a.total_cmp(b));
    v[v.len() / 2]
}

/// Peak working set of THIS process in MB via PowerShell (Windows), or -1 when unavailable.
fn peak_mb() -> f64 {
    let pid = std::process::id();
    std::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", &format!("[math]::Round((Get-Process -Id {pid}).PeakWorkingSet64/1MB,1)")])
        .output()
        .ok()
        .and_then(|o| String::from_utf8_lossy(&o.stdout).trim().parse().ok())
        .unwrap_or(-1.0)
}

/// Measures the cold build (list, read, chunk, tokenize, index) at 2k, 5k and 10k notes, three runs
/// each, and the query latency over 300 queries on the 10k index. Prints one line per size.
#[test]
#[ignore = "a measurement: run alone, release profile (module doc)"]
fn cold_build_and_query_latency_at_2k_5k_10k_notes() {
    let profile = if cfg!(debug_assertions) { "dev(opt-level=1)" } else { "release" };
    println!("profile: {profile}; KNOWLEDGE_NOTE_CAP = {KNOWLEDGE_NOTE_CAP}");
    for n in [2000usize, 5000, 10_000] {
        let (dir, bytes, v) = build_dir(n);
        let mut times: Vec<f64> = Vec::new();
        let mut last: Option<(VaultIndex, KnowledgeIndex)> = None;
        for _ in 0..3 {
            let t0 = Instant::now();
            let mut vi = VaultIndex::new();
            let st = vi.refresh(&FsVault::new(&dir), KNOWLEDGE_NOTE_CAP).unwrap();
            assert_eq!(st.reread, n, "a cold build reads every note");
            let ki = KnowledgeIndex::build(None, Some(&vi));
            times.push(t0.elapsed().as_secs_f64());
            last = Some((vi, ki));
        }
        let (vi, ki) = last.unwrap();
        let (mn, mx) = (times.iter().cloned().fold(f64::MAX, f64::min), times.iter().cloned().fold(0.0, f64::max));
        let med = median(&mut times.clone());
        println!(
            "{n} notes ({:.1} MB on disk, {} chunks, {} docs): cold build median {med:.2} s (min {mn:.2} .. max {mx:.2}); index ~{:.0} MB by field sizes; process peak working set {:.0} MB",
            bytes as f64 / 1e6,
            vi.chunk_count(),
            ki.doc_count(),
            vi.approx_bytes() as f64 / 1e6,
            peak_mb()
        );
        if n == 10_000 {
            let mut rng = Rng::new(99);
            let mut lat: Vec<f64> = Vec::new();
            for _ in 0..300 {
                let q = format!("{} {} {}", v[rng.below(v.len())], v[rng.below(v.len())], v[rng.below(v.len())]);
                let t0 = Instant::now();
                let o = ki.search(&Query::new(&q));
                lat.push(t0.elapsed().as_secs_f64() * 1000.0);
                assert!(o.available);
            }
            lat.sort_by(|a, b| a.total_cmp(b));
            println!("{} docs: query latency over 300 queries: p50 {:.2} ms, p95 {:.2} ms, max {:.2} ms", ki.doc_count(), lat[150], lat[285], lat[299]);
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
}
