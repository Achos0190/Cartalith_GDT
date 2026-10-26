//! The knowledge crate has no socket (`RAG_KNOWLEDGE_BASE_RESEARCH.md` s.1 hard constraints: "zero
//! sockets when off"; the feature is off by default and this crate can never be the thing that opens
//! one). Mirrors `cartalith-llm`'s `tests/no_network.rs`.
//!
//! Two checks: no source file under `src/` names a network API, and `Cargo.toml` lists no HTTP/TLS/
//! async-runtime dependency. A positive control proves the scanner can fail.

use std::path::PathBuf;

/// Network-capable identifiers. A match in `src/` is a failure.
const FORBIDDEN_SRC: [&str; 9] = ["std::net", "TcpStream", "TcpListener", "UdpSocket", "reqwest", "hyper::", "ureq", "tokio::net", "tungstenite"];

/// Dependencies whose presence in `Cargo.toml` would make a socket one call away.
const FORBIDDEN_DEPS: [&str; 7] = ["reqwest", "hyper", "ureq", "tokio", "async-std", "curl", "native-tls"];

fn scan(text: &str, needles: &[&str]) -> Vec<String> {
    needles.iter().filter(|n| text.contains(**n)).map(|n| n.to_string()).collect()
}

/// Protects the no-socket rule over every file in `src/`.
#[test]
fn no_source_file_names_a_network_api() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = 0;
    for e in std::fs::read_dir(&dir).unwrap() {
        let p = e.unwrap().path();
        if p.extension().is_some_and(|x| x == "rs") {
            files += 1;
            let text = std::fs::read_to_string(&p).unwrap();
            let hits = scan(&text, &FORBIDDEN_SRC);
            assert!(hits.is_empty(), "{} names a network API: {hits:?}", p.display());
        }
    }
    assert!(files >= 7, "the scan must see the crate's files, saw {files}");
}

/// Protects the dependency list.
#[test]
fn cargo_toml_lists_no_network_dependency() {
    let text = std::fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml")).unwrap();
    let hits = scan(&text, &FORBIDDEN_DEPS);
    assert!(hits.is_empty(), "Cargo.toml lists {hits:?}");
}

/// Positive control: the scanner flags a file that does name a socket, so a green run is not vacuous.
#[test]
fn the_scanner_can_fail() {
    assert_eq!(scan("let s = std::net::TcpStream::connect(a);", &FORBIDDEN_SRC), vec!["std::net".to_string(), "TcpStream".to_string()]);
    assert!(scan("fn harmless() {}", &FORBIDDEN_SRC).is_empty());
}
