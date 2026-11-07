//! AC-1 (`ASSISTANT_ACTIONS_PROPOSAL.md` s.4): the static half of the action catalogue generator.
//!
//! WHY a scan at all: the context card's dotted actions and the map's armed tools are not in any
//! runtime registry. The context actions are built inside per-workspace functions from a map
//! point; the tools are string literals passed to `app.arm_tool`. So the catalogue's "generated,
//! never hand-copied" rule is met for them by scanning the shell's own source, and committing the
//! result as `data/action_static_sources.json`, which `cartalith_llm::actions` embeds. This file is
//! the scanner AND the drift guard:
//!
//! * `committed_static_sources_match_the_shell_source` re-scans `godot-project/shell` and fails,
//!   naming the difference, when a context action or tool was added, renamed or removed and the
//!   data file was not regenerated. A stale data file is how a model would be offered an action
//!   the app no longer has (or never learn of a new one).
//! * `regenerate_static_sources` (`#[ignore]`) rewrites the data file. Run
//!   `cargo test -p cartalith-llm --test ac1_static_sources regenerate -- --ignored` after a shell
//!   change, then commit the data file with it.
//!
//! The scanner is deliberately std-only (this crate has one dependency by test) and reads only
//! inside the repository. The same scan was prototyped in Python by AC-0
//! (`docs/evidence/assistant-actions-ac0-2026-10-31/tools/actions.py`, `_ctx_actions`, `_tools`).
//!
//! MUST NEVER: read outside the repository, invent an id the source does not contain, or write the
//! data file from a plain `cargo test` (the regenerate test is `#[ignore]`).

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};

/// The shell directory relative to this crate (`cartalith-native/crates/cartalith-llm`).
fn shell_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../godot-project/shell")
}

fn data_file() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data/action_static_sources.json")
}

/// The files whose literals define a context action: `global_tools.gd` and each workspace.
fn context_files() -> Vec<PathBuf> {
    let base = shell_dir();
    let mut out = vec![base.join("global_tools.gd")];
    let mut ws: Vec<PathBuf> = fs::read_dir(base.join("workspaces"))
        .expect("workspaces dir")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "gd"))
        .collect();
    ws.sort();
    out.extend(ws);
    out
}

/// Every `.gd` file under the shell directory, sorted (so the scan is deterministic).
fn all_gd() -> Vec<PathBuf> {
    fn walk(d: &Path, out: &mut Vec<PathBuf>) {
        let mut es: Vec<PathBuf> = fs::read_dir(d).expect("dir").filter_map(|e| e.ok().map(|e| e.path())).collect();
        es.sort();
        for p in es {
            if p.is_dir() {
                walk(&p, out);
            } else if p.extension().is_some_and(|x| x == "gd") {
                out.push(p);
            }
        }
    }
    let mut out = Vec::new();
    walk(&shell_dir(), &mut out);
    out
}

/// `{"id": "civ.start_way", "label": "Start way here", ...}` literals. A label containing `%` is a
/// format string (its text is computed), so it is skipped, as the AC-0 prototype did.
fn scan_context(src: &str, file: &str, out: &mut Vec<(String, String, String)>) {
    let key = "\"id\": \"";
    let mut rest = src;
    while let Some(p) = rest.find(key) {
        rest = &rest[p + key.len()..];
        let Some(end) = rest.find('"') else { break };
        let id = &rest[..end];
        let dom_ok = ["global.", "civ.", "carto.", "world."].iter().any(|d| id.starts_with(d));
        let id_ok = id.chars().all(|c| c.is_ascii_lowercase() || c == '_' || c == '.');
        if !dom_ok || !id_ok {
            continue;
        }
        let after = &rest[end + 1..];
        let lab = "\"label\": \"";
        // The label must follow the id within the same dictionary literal (a short window).
        let Some(lp) = after.find(lab).filter(|lp| *lp <= 4) else { continue };
        let l = &after[lp + lab.len()..];
        let mut label = String::new();
        let mut chars = l.chars();
        while let Some(c) = chars.next() {
            if c == '\\' {
                if let Some(n) = chars.next() {
                    label.push(n);
                }
            } else if c == '"' {
                break;
            } else {
                label.push(c);
            }
        }
        if label.is_empty() || label.contains('%') || label.chars().count() > 80 {
            continue;
        }
        if !out.iter().any(|(i, _, _)| i == id) {
            out.push((id.to_owned(), label, file.to_owned()));
        }
    }
}

/// `arm_tool("<id>")` literals anywhere in the shell.
fn scan_tools(src: &str, out: &mut Vec<String>) {
    let key = "arm_tool(\"";
    let mut rest = src;
    while let Some(p) = rest.find(key) {
        rest = &rest[p + key.len()..];
        let Some(end) = rest.find('"') else { break };
        let id = &rest[..end];
        if !id.is_empty() && id.chars().all(|c| c.is_ascii_lowercase() || c == '_') && !out.iter().any(|t| t == id) {
            out.push(id.to_owned());
        }
    }
}

/// The whole scan as the JSON the data file holds.
fn scan() -> Value {
    let mut ctx: Vec<(String, String, String)> = Vec::new();
    for f in context_files() {
        let name = f.file_name().unwrap().to_string_lossy().to_string();
        scan_context(&fs::read_to_string(&f).expect("read"), &name, &mut ctx);
    }
    ctx.sort();
    let mut tools: Vec<String> = Vec::new();
    for f in all_gd() {
        scan_tools(&fs::read_to_string(&f).expect("read"), &mut tools);
    }
    tools.sort();
    json!({
        "about": "GENERATED by cartalith-llm/tests/ac1_static_sources.rs; do not edit by hand. Regenerate: cargo test -p cartalith-llm --test ac1_static_sources regenerate -- --ignored",
        "context": ctx.iter().map(|(id, label, file)| json!({"id": id, "label": label, "file": file})).collect::<Vec<_>>(),
        "tools": tools,
    })
}

#[test]
#[ignore = "rewrites data/action_static_sources.json; run it after a shell change"]
fn regenerate_static_sources() {
    let text = serde_json::to_string_pretty(&scan()).unwrap() + "\n";
    fs::create_dir_all(data_file().parent().unwrap()).unwrap();
    fs::write(data_file(), text).unwrap();
}

/// Protects the drift rule: the committed data file IS what the shell source scans to. MUTATION:
/// rename `civ.start_way` in `civilization_workspace.gd` and this goes red naming it.
#[test]
fn committed_static_sources_match_the_shell_source() {
    let now = scan();
    let committed: Value = serde_json::from_str(&fs::read_to_string(data_file()).expect("data/action_static_sources.json is missing")).unwrap();
    let ids = |v: &Value, k: &str| -> Vec<String> {
        v[k].as_array().unwrap().iter().map(|x| x.get("id").unwrap_or(x).as_str().unwrap().to_owned()).collect()
    };
    for k in ["context", "tools"] {
        let (a, b) = (ids(&now, k), ids(&committed, k));
        let added: Vec<_> = a.iter().filter(|x| !b.contains(x)).collect();
        let gone: Vec<_> = b.iter().filter(|x| !a.contains(x)).collect();
        assert!(added.is_empty() && gone.is_empty(), "{k}: the shell has {added:?} that the data file lacks, and the data file has {gone:?} the shell no longer has; regenerate it");
    }
    assert_eq!(now["context"], committed["context"], "a context action's label changed; regenerate the data file");
}

/// Protects against a silently empty scan (MISTAKES.md: four subsystems were bitten): the scan must
/// find a plausible number of each, with the AC-0 counts as the floor (30 context actions, 13 tools).
#[test]
fn the_scan_is_not_silently_empty() {
    let now = scan();
    assert!(now["context"].as_array().unwrap().len() >= 30, "context actions found: {}", now["context"].as_array().unwrap().len());
    assert!(now["tools"].as_array().unwrap().len() >= 13, "tools found: {}", now["tools"].as_array().unwrap().len());
    assert!(now["tools"].as_array().unwrap().iter().any(|t| t == "sculpt"));
}
