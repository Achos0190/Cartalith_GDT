//! "Zero network use in tests" (LLM-1 bar) and the stronger claim behind it:
//! this crate cannot open a socket, because it has neither a networking
//! dependency nor a use of the standard library's socket module.
//!
//! Protects: LLM_ASSISTANT_SCOPE.md s.2 ("No HTTP or TLS dependency") and s.4
//! rule 1. A reader cannot rely on reviewing the code for this: a future
//! convenience dependency (reqwest, ureq, tokio) would pass every behavioural
//! test, so the check is mechanical.

use std::fs;
use std::path::Path;

/// Needles are assembled from pieces so this file does not contain them
/// literally and match itself.
fn forbidden() -> Vec<String> {
    vec![
        format!("std::{}", "net"),
        format!("Tcp{}", "Stream"),
        format!("Udp{}", "Socket"),
        format!("{}::", "reqwest"),
        format!("{}::", "ureq"),
        format!("{}::", "tokio"),
        format!("{}::", "hyper"),
    ]
}

fn rs_files(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    for e in fs::read_dir(dir).unwrap() {
        let p = e.unwrap().path();
        if p.is_dir() {
            rs_files(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

/// Protects: no source file under `src/` or `tests/` mentions a socket API or
/// a networking crate path.
#[test]
fn no_source_file_touches_the_network() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    rs_files(&root.join("src"), &mut files);
    rs_files(&root.join("tests"), &mut files);
    assert!(files.len() >= 8, "silently-empty guard: only {} files", files.len());
    for f in files {
        let text = fs::read_to_string(&f).unwrap();
        for n in forbidden() {
            assert!(!text.contains(&n), "{} mentions {n}", f.display());
        }
    }
}

/// Protects: the dependency list is exactly `serde_json` plus the workspace's own `cartalith-vault`
/// (Ruling DP: the one shared private-note rule lives there; the Vault crate itself depends only on
/// `serde` and `serde_json`, checked below, so it adds no socket path). Adding any other dependency
/// (or a `[dev-dependencies]` section) fails here and needs a ruling-level reason.
#[test]
fn no_network_dependency() {
    let manifest = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml")).unwrap();
    let deps: Vec<&str> = manifest
        .lines()
        .map(str::trim)
        .skip_while(|l| *l != "[dependencies]")
        .skip(1)
        .take_while(|l| !l.starts_with('['))
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect();
    assert_eq!(deps.len(), 2, "{deps:?}");
    assert!(deps[0].starts_with("serde_json "), "{deps:?}");
    assert_eq!(deps[1], "cartalith-vault = { path = \"../cartalith-vault\" }", "{deps:?}");
    let vault = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("../cartalith-vault/Cargo.toml")).unwrap();
    let vdeps: Vec<&str> = vault
        .lines()
        .map(str::trim)
        .skip_while(|l| *l != "[dependencies]")
        .skip(1)
        .take_while(|l| !l.starts_with('['))
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect();
    assert_eq!(vdeps.len(), 2, "{vdeps:?}");
    assert!(vdeps[0].starts_with("serde = ") && vdeps[1].starts_with("serde_json "), "{vdeps:?}");
    assert!(!manifest.contains("[dev-dependencies]"));
    assert!(!manifest.contains("[build-dependencies]"));
}
