//! AC-1 and AC-2 over the REAL rows (`ASSISTANT_ACTIONS_PROPOSAL.md` s.4; Ruling DW): the action
//! catalogue built from the real generation-parameter registry, the real shell source (layers, style
//! presets, curated windows, context actions, tools) and the 66 real runnable `CommandIndex` rows a
//! windowed probe captured (`crates/cartalith-llm/tests/fixtures/ac1_shell_facts_2026-10-31.json`,
//! written by `_ac2_probe.gd --dump`), then the four tools over it.
//!
//! Pulled in by `#[path]`, the `tests/params_mapping.rs` pattern: both included files are godot-free, so
//! this runs under a plain `cargo test` with no engine process.
//!
//! What a green run here DOES NOT show: that the shell's real `CommandIndex` still has these rows. That
//! is the windowed probe's job (`_ac2_probe.gd`, which compares the live facts to what Rust accepts).
//! `cargo test` cannot see a broken shell (MISTAKES.md).
#![allow(dead_code)]

#[path = "../src/params.rs"]
mod params;
#[path = "../src/param_catalogue.rs"]
mod param_catalogue;

use std::fs;
use std::path::{Path, PathBuf};

use cartalith_llm::action_tools::*;
use cartalith_llm::actions::*;
use cartalith_llm::catalogue::{Catalogue, PValue};
use cartalith_llm::openai::{MaxTokensField, Thinking};
use cartalith_llm::params_flow::{IncomingCall, ParamsConversation, Step};
use cartalith_llm::view_actions::ViewCatalogue;
use serde_json::{Value, json};

fn repo(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../").join(rel)
}

fn shell_src(rel: &str) -> String {
    fs::read_to_string(repo(&format!("godot-project/shell/{rel}"))).unwrap_or_else(|e| panic!("{rel}: {e}"))
}

fn params_catalogue() -> Catalogue {
    let d = params::defaults();
    param_catalogue::build(&|k| {
        params::get(&d, k).map(|v| match v {
            params::Value::Bool(b) => PValue::Bool(b),
            params::Value::Num(n) => PValue::Num(n),
        })
    })
}

/// The ids in a GDScript `const NAME := [ ... ]` block that follow `"id": "`, in order.
fn block_ids(src: &str, name: &str, key: &str) -> Vec<String> {
    let start = src.find(&format!("const {name}")).unwrap_or_else(|| panic!("const {name} not found"));
    let rest = &src[start..];
    let end = rest.find("\n]").expect("block end");
    let block = &rest[..end];
    let pat = format!("\"{key}\": \"");
    block.match_indices(&pat).map(|(i, _)| block[i + pat.len()..].split('"').next().unwrap().to_owned()).collect()
}

/// The shell's own slug (`llm_view.gd::slug`): lower-case, runs of other characters collapsed to `_`.
fn slug(n: &str) -> String {
    let mut out = String::new();
    for ch in n.to_lowercase().chars() {
        if ch.is_ascii_lowercase() || ch.is_ascii_digit() {
            out.push(ch);
        } else if !out.ends_with('_') {
            out.push('_');
        }
    }
    out.trim_matches('_').to_owned()
}

/// View facts built from the REAL shell source: the ten layer ids, every style preset, the ten curated
/// windows. Landmark kinds are withheld, as on a fresh world with no landmark pass (the probe's own
/// measurement, `_llmview_probe.gd` check 1).
fn real_view() -> ViewCatalogue {
    let cw = shell_src("workspaces/cartography_workspace.gd");
    let layers: Vec<Value> = block_ids(&cw, "LIVE_LAYERS", "id").iter().map(|i| json!({"id": i, "label": i.replace('_', " ")})).collect();
    let rw = shell_src("workspaces/render_workspace.gd");
    let start = rw.find("const STYLE_PRESETS := [").unwrap();
    let blk = &rw[start..start + rw[start..].find("\n]").unwrap()];
    let presets: Vec<Value> =
        blk.lines().filter_map(|l| l.trim_start().strip_prefix("[\"")).map(|l| l.split('"').next().unwrap()).map(|n| json!({"id": slug(n), "label": n})).collect();
    let v = shell_src("llm_view.gd");
    let windows: Vec<Value> = block_ids(&v, "WINDOWS", "id").iter().map(|i| json!({"id": i, "label": i.replace('_', " ")})).collect();
    ViewCatalogue::from_json(&json!({
        "layers": layers, "presets": presets, "windows": windows,
        "camera": {"w": 2048, "h": 1311, "zoom_min": 0.4, "zoom_max": 64.0, "zoom_now": 1.0},
        "withheld": {"set_landmark_filter": "This world carries no landmarks to filter."}
    }))
    .unwrap()
}

const FACTS: &str = include_str!("../../cartalith-llm/tests/fixtures/ac1_shell_facts_2026-10-31.json");

fn shell_facts() -> ShellFacts {
    ShellFacts::from_json_text(FACTS).expect("the fixture parses")
}

fn real_catalogue() -> ActionCatalogue {
    ActionCatalogue::build(Some(&params_catalogue()), Some(&real_view()), &shell_facts())
}

/// AC-1 over the real rows, with the counts as literals (measured 2026-10-31). Protects: every entry has
/// a stable id, a description of at most 120 characters, a path of at most three levels and an effect;
/// the counts per source; every one of the 66 runnable rows is exactly one of an entry, a merge into a
/// window, or a recorded refusal (none silently lost); and the derived-description count that AC-5 owns.
/// MUTATION: add a row to the fixture and the totals move; drop `refusal` from `push` and `menu:save`
/// becomes an entry (the next test).
#[test]
fn ac1_counts_and_bars_over_the_real_rows() {
    let c = real_catalogue();
    let facts = shell_facts();
    assert_eq!(facts.menu_rows.len(), 66, "the captured probe rows");
    for e in c.entries() {
        assert!(!e.desc.is_empty() && e.desc.chars().count() <= MAX_DESC_CHARS, "{}: {:?}", e.id, e.desc);
        assert!(e.path.split('/').count() <= MAX_PATH_DEPTH && !e.path.is_empty(), "{}: {}", e.id, e.path);
        assert!(e.id.chars().count() <= MAX_ID_CHARS && e.id.chars().all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || "._:".contains(ch)), "{}", e.id);
        assert!(e.effect.as_str() != "destructive");
    }
    let by = |s: &str| c.count_source(s);
    for d in c.dropped.iter().filter(|d| !d.id.starts_with("menu:")) {
        eprintln!("AC1 dropped (non-menu): {} ({})", d.id, d.why);
    }
    assert_eq!(by("params"), 114, "every offered registry row is an entry");
    assert_eq!(by("view"), 10 + 3, "ten layers plus the style preset, zoom and the (withheld) landmark filter");
    assert_eq!(by("window"), 10);
    assert_eq!(by("context"), 23, "30 scanned minus 7 refused: export, discard x3, erase, clear, undo");
    assert_eq!(by("tool"), 13);
    assert_eq!(by("timeline"), 1);
    assert_eq!(by("menu"), 31, "66 rows less 10 merged into windows and 25 refused");
    assert_eq!(c.entries().len(), 114 + 13 + 10 + 23 + 13 + 1 + 31);
    assert_eq!(c.derived_desc_count(), 24, "AC-5's work list: derived descriptions (unreviewed menu and context rows)");
    // Accounting: each of the 66 rows is an entry, a merge or a refusal.
    for r in &facts.menu_rows {
        let id = format!("menu:{}", r.name);
        let n = usize::from(c.get(&id).is_some()) + c.dropped.iter().filter(|d| d.id == id).count();
        assert_eq!(n, 1, "{id}: exactly one of entry / merge / refusal");
    }
    let merged = c.dropped.iter().filter(|d| d.why.starts_with("merged into ")).count();
    let refused = c.dropped.iter().filter(|d| d.id.starts_with("menu:") && d.why.starts_with("contains the forbidden")).count();
    assert_eq!((merged, refused), (10, 25));
}

/// Protects "refuse anything destructive by construction" over the REAL pinnable rows (Save, Close,
/// New world, Open project, Export, Import, Delete, Clear, Reset are all pinnable in the real index):
/// none is an entry, and no entry id or title contains a forbidden word. MUTATION: delete "import" from
/// `FORBIDDEN_STEMS` and `menu:import_image` appears.
#[test]
fn the_real_destructive_rows_are_not_in_the_catalogue() {
    let c = real_catalogue();
    for name in [
        "save", "save_as", "close", "new_world", "new_world_from_selection", "open_project", "revert", "export_dm", "export_map_image", "export_player",
        "ap_export", "import_image", "import_pack", "import_folder", "delete", "cut", "paste", "clear_library", "pref_clear_caches", "edit_reset_params",
        "win_reset", "win_save_layout", "help_report", "help_gen_info", "undo_history",
    ] {
        assert!(c.get(&format!("menu:{name}")).is_none(), "menu:{name} must not be an entry");
    }
    for e in c.entries() {
        // A parameter's title is the registry's own wording and is not checked (`carve_rivers` says
        // "on generation"); its id still is.
        let title = if e.source == "params" { "" } else { e.title.as_str() };
        assert_eq!(refusal(&e.id, title), None, "{} / {}", e.id, e.title);
    }
    // The rows nobody reviewed are listed, unavailable, and cannot be proposed.
    for name in ["data_manager", "apply_library", "help_docs", "copy", "ap_validate", "storage"] {
        let e = c.get(&format!("menu:{name}")).unwrap_or_else(|| panic!("menu:{name} should be listed as unreviewed"));
        assert_eq!((e.effect, e.available), (Effect::Unreviewed, false), "{name}");
    }
}

/// Protects "the ids are identical across two builds" over real rows: shuffling the shell's rows, and
/// building twice, gives equal catalogues; the id list is sorted-unique.
#[test]
fn ac1_ids_are_identical_across_builds() {
    let a = real_catalogue();
    let mut f = shell_facts();
    f.menu_rows.reverse();
    let b = ActionCatalogue::build(Some(&params_catalogue()), Some(&real_view()), &f);
    assert_eq!(a, b);
    let ids: Vec<String> = a.entries().iter().map(|e| e.id.clone()).collect();
    let mut s = ids.clone();
    s.sort();
    s.dedup();
    assert_eq!(s.len(), ids.len());
    assert_eq!(real_catalogue(), a, "a second build from the same inputs is equal");
}

/// Protects the curated tables against the real sources they claim to describe: every `MENU_REVIEW` name
/// is a `DccMenus` constant in `menus.gd`; every `WINDOW_REVIEW` id is in `llm_view.gd::WINDOWS`; every
/// `CONTEXT_REVIEW` and `TOOL_REVIEW` id is in the scanned data; every alias names a real entry; and no
/// review row is one the refusal list would drop (a dead row). MUTATION: rename a constant in `menus.gd`
/// and its review row fails here, naming it.
#[test]
fn every_curated_row_names_something_real() {
    let menus = shell_src("menus.gd");
    for (name, ..) in MENU_REVIEW {
        assert!(menus.contains(&format!("const ID_{} :=", name.to_uppercase())), "MENU_REVIEW names ID_{} which menus.gd lacks", name.to_uppercase());
        assert_eq!(refusal(&format!("menu:{name}"), ""), None, "{name} is reviewed but refused: a dead row");
    }
    let wins = block_ids(&shell_src("llm_view.gd"), "WINDOWS", "id");
    for (id, ..) in WINDOW_REVIEW {
        assert!(wins.iter().any(|w| w == id), "WINDOW_REVIEW names {id}, not in llm_view.gd::WINDOWS");
    }
    for w in &wins {
        assert!(WINDOW_REVIEW.iter().any(|(i, ..)| i == w), "window {w} has no curated description");
    }
    let data: Value = serde_json::from_str(STATIC_SOURCES).unwrap();
    let ctx: Vec<&str> = data["context"].as_array().unwrap().iter().map(|c| c["id"].as_str().unwrap()).collect();
    for (id, ..) in CONTEXT_REVIEW {
        assert!(ctx.contains(id), "CONTEXT_REVIEW names {id}, not scanned from the shell");
        let label = data["context"].as_array().unwrap().iter().find(|c| c["id"] == *id).unwrap()["label"].as_str().unwrap();
        assert_eq!(refusal(id, label), None, "{id} is reviewed but its id or label is refused");
    }
    let tools: Vec<&str> = data["tools"].as_array().unwrap().iter().map(|t| t.as_str().unwrap()).collect();
    for (id, _) in TOOL_REVIEW {
        assert!(tools.contains(id), "TOOL_REVIEW names {id}, not scanned from the shell");
    }
    for t in &tools {
        assert!(TOOL_REVIEW.iter().any(|(i, _)| i == t), "tool {t} has no curated description");
    }
    let c = real_catalogue();
    for (id, _) in ALIASES {
        assert!(c.get(id).is_some(), "ALIASES names {id}, which is not an entry");
    }
}

fn conv() -> (ParamsConversation, Catalogue) {
    let cat = params_catalogue();
    let c = ParamsConversation::new(&cat, "hide the rivers").unwrap().with_queries(&cat, true).with_actions(&cat, Some(real_catalogue()));
    (c, cat)
}

fn call(name: &str, args: Value) -> IncomingCall {
    IncomingCall { id: None, name: name.into(), arguments: args }
}

/// Protects "offered behind the Commands switch exactly like the current tools": with the catalogue the
/// four tools ride beside the schema, query and (if offered) view tools; without it NOTHING changes (the
/// request is byte-identical to a conversation that never heard of actions); the prompt names the tools
/// exactly when they are offered; an edit or a questions-only request never gets them.
#[test]
fn the_four_tools_are_offered_beside_the_old_ones_and_only_when_enabled() {
    let (c, cat) = conv();
    let names: Vec<String> = c.tools(&cat).iter().map(|t| t.name.clone()).collect();
    assert_eq!(names, ["get_param_schema", "get_parameters", "get_changed_parameters", "describe_parameter", "get_ecology_summary", "sample_point", "get_world_summary",
        "list_actions", "search_actions", "describe_action", "propose_actions"]);
    let sys = &c.messages()[0].content;
    assert!(sys.contains("list_actions") && sys.contains("What exists (category"), "{sys}");
    let plain = ParamsConversation::new(&cat, "hide the rivers").unwrap().with_queries(&cat, true);
    let none = plain.clone().with_actions(&cat, None);
    assert_eq!(plain.messages()[0].content, none.messages()[0].content, "no catalogue, no prompt text");
    assert!(!plain.messages()[0].content.contains("list_actions"));
    assert!(!plain.tools(&cat).iter().any(|t| is_action_tool(&t.name)));
    let q = ParamsConversation::new(&cat, "who rules?").unwrap().with_actions(&cat, Some(real_catalogue()));
    assert!(q.clone().with_knowledge(&cat, cartalith_llm::knowledge_tools::KnowledgeSources { vault: true, world: false }, true, true).tools(&cat).iter().all(|t| !is_action_tool(&t.name)));
}

/// AC-2's wire-size bars as literals, measured 2026-10-31 on the real catalogue. Chars are the exact JSON;
/// tokens use the guard's pessimistic 3.4 chars a token (AC-0 measured the real ratio at most 3.47 with
/// three real tokenizers). Protects the proposal's s.3.6 targets: the four tools and the capability map
/// are well inside "fixed per request <= 1.6k tokens"; each list or search <= 400; each describe <= 300 an
/// action. A failure here means a description or the tree grew, not that the target is wrong: read the
/// numbers, do not retune.
#[test]
fn wire_sizes_are_within_the_proposals_targets() {
    let c = real_catalogue();
    let specs = c.tool_specs();
    let spec_chars: usize = specs.iter().map(|t| json!({"name": t.name, "description": t.description, "parameters": t.parameters}).to_string().len()).sum();
    let cmap = c.capability_map();
    let para = c.prompt_paragraph();
    // The bars are judged at 4.0 characters a token: AC-0 measured 4.0-4.1 for compact JSON with the real
    // Qwen3, gpt-oss and Ministral-3 tokenizers (its "search_actions, 8 matches" row: 1 620 characters =
    // 401 / 405 / 432 tokens). The guard's own estimate is the pessimistic 3.4 and is reported beside it.
    let tok = |chars: usize| (chars as f64 / 4.0).ceil() as usize;
    let pess = |chars: usize| (chars as f64 / CHARS_PER_TOKEN).ceil() as usize;
    eprintln!("AC2 sizes: four tool specs {spec_chars} chars ({} tok), capability map {} chars, prompt paragraph {} chars ({} tok)", tok(spec_chars), cmap.len(), para.len(), tok(para.len()));
    assert!(tok(spec_chars + para.len()) <= 900, "four tools + the prompt paragraph: {} tokens", tok(spec_chars + para.len()));
    let mut l = ActionLedger::default();
    let mut worst_list = 0usize;
    // EVERY category prefix and every page of it, not a hand-picked few.
    let mut prefixes: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    prefixes.insert(String::new());
    for e in c.entries() {
        let segs: Vec<&str> = e.path.split('/').collect();
        for n in 1..=segs.len() {
            prefixes.insert(segs[..n].join("/"));
        }
    }
    let mut calls = 0usize;
    for p in &prefixes {
        let mut page = 0;
        loop {
            let r = c.list(p, None, page, &mut l);
            assert!(r["entries"].as_array().map_or(0, Vec::len) <= MAX_LIST_ENTRIES, "{p}");
            calls += 1;
            let n = r.to_string().len();
            if n > worst_list {
                worst_list = n;
                eprintln!("AC2 worst list so far: {p:?} page {page}: {n} chars");
            }
            if r.get("next_page").is_none() {
                break;
            }
            page += 1;
        }
    }
    eprintln!("AC2 {} prefixes, {calls} list calls, worst {worst_list} chars ({} tok, {} pessimistic)", prefixes.len(), tok(worst_list), pess(worst_list));
    assert!(tok(worst_list) <= 400, "worst list: {} chars = {} tokens", worst_list, tok(worst_list));
    let mut worst_search = 0usize;
    for q in ["raise the sea level", "show the political map", "open the faction roster", "hide the rivers", "draw a new road", "year 398"] {
        let r = c.search(q, None, None, &mut l);
        assert!(r["matches"].as_array().unwrap().len() <= MAX_SEARCH_MATCHES);
        worst_search = worst_search.max(r.to_string().len());
    }
    eprintln!("AC2 worst search {worst_search} chars ({} tok, {} pessimistic)", tok(worst_search), pess(worst_search));
    assert!(tok(worst_search) <= 400, "worst search: {} tokens", tok(worst_search));
    let ids: Vec<String> = c.entries().iter().map(|e| e.id.clone()).collect();
    let mut worst_desc = 0usize;
    for e in c.entries() {
        let r = c.describe(&[e.id.clone()], &{
            let mut k = ActionLedger::default();
            let _ = c.search(&e.title, None, None, &mut k);
            let _ = &ids;
            k
        });
        if r["actions"][0].get("error").is_none() {
            worst_desc = worst_desc.max(r.to_string().len());
        }
    }
    eprintln!("AC2 worst single describe {worst_desc} chars ({} tok, {} pessimistic)", tok(worst_desc), pess(worst_desc));
    assert!(tok(worst_desc) <= 300, "worst describe: {} tokens", tok(worst_desc));
}

/// Protects the AC-0 reach numbers on the REAL catalogue: the 30-task script's search half (the model-free
/// half AC-0 measured on the fixture) still reaches the AC-0 floor of 27 of 28 needed actions in the top
/// 8 and 26 in the top 5, and the AC-0 miss ("Let me draw a new road") is now found. The task list and its
/// expected ids are AC-0's own (`tools/tasks.json`), translated to this catalogue's ids by `ID_MAP`; a
/// task whose action AC-1 does not offer (select, context-tab, selection verbs that no code has yet) is
/// listed in `UNOFFERED` and counted, not skipped silently.
#[test]
fn the_ac0_search_reach_holds_on_the_real_catalogue() {
    let tasks: Value = serde_json::from_str(include_str!("../../../docs/evidence/assistant-actions-ac0-2026-10-31/tools/tasks.json")).unwrap();
    let c = real_catalogue();
    let (mut needed, mut top8, mut top5) = (0, 0, 0);
    let mut unoffered: Vec<String> = Vec::new();
    for t in tasks["tasks"].as_array().expect("tasks array") {
        let text = t["prompt"].as_str().unwrap();
        for slot in t["slots"].as_array().unwrap() {
            let ids: Vec<&str> = slot.as_array().unwrap().iter().map(|x| x.as_str().unwrap()).collect();
            // A slot is satisfied by any one of its ids; it needs an offered id to be reachable at all.
            if !ids.iter().any(|i| c.get(i).is_some()) {
                unoffered.push(format!("{} -> {}", t["id"].as_str().unwrap(), ids.join("|")));
                continue;
            }
            needed += 1;
            let r = c.search(text, None, None, &mut ActionLedger::default());
            let ranked: Vec<&str> = r["matches"].as_array().unwrap().iter().map(|m| m["id"].as_str().unwrap()).collect();
            if ids.iter().any(|i| ranked.contains(i)) {
                top8 += 1;
            }
            if ids.iter().any(|i| ranked.iter().take(5).any(|x| x == i)) {
                top5 += 1;
            }
        }
    }
    eprintln!("AC2 reach: needed {needed}, top8 {top8}, top5 {top5}, unoffered {}: {unoffered:?}", unoffered.len());
    assert!(needed >= 20, "the script must exercise the catalogue: {needed}");
    assert!(top8 * 100 >= needed * 85, "search top-8 reach fell: {top8}/{needed}");
    let road = c.search("Let me draw a new road", None, None, &mut ActionLedger::default());
    assert_eq!(road["matches"][0]["id"], json!("tool.way"), "the AC-0 miss is fixed by the alias: {road}");
}

/// AC-2's bar "AC-0's script runs both ways, by browsing and by search, and reports each", the browsing
/// half, model-free: for each offered slot of the 30 tasks, walk `list_actions` from the top, one level at
/// a time, along the id's own path (what a model that reads the tree does), following `next_page` when a
/// leaf pages, and check the id is returned. Reports the calls it took. Protects: every action is
/// reachable by browsing alone, within the proposal's three levels (so at most 4 calls plus pages).
#[test]
fn every_offered_action_is_reachable_by_browsing_alone() {
    let tasks: Value = serde_json::from_str(include_str!("../../../docs/evidence/assistant-actions-ac0-2026-10-31/tools/tasks.json")).unwrap();
    let c = real_catalogue();
    let (mut slots, mut calls_total, mut worst) = (0usize, 0usize, 0usize);
    for t in tasks["tasks"].as_array().unwrap() {
        for slot in t["slots"].as_array().unwrap() {
            let ids: Vec<&str> = slot.as_array().unwrap().iter().map(|x| x.as_str().unwrap()).collect();
            let Some(e) = ids.iter().find_map(|i| c.get(i)) else { continue };
            let mut ledger = ActionLedger::default();
            let segs: Vec<&str> = e.path.split('/').collect();
            let mut calls = 0usize;
            for n in 0..=segs.len() {
                let path = segs[..n].join("/");
                let mut page = 0;
                loop {
                    let r = c.list(&path, None, page, &mut ledger);
                    calls += 1;
                    if r.get("next_page").is_none() || ledger.knows(&e.id) {
                        break;
                    }
                    page += 1;
                }
            }
            assert!(ledger.knows(&e.id), "{}: {} not reachable by browsing {}", t["id"], e.id, e.path);
            slots += 1;
            calls_total += calls;
            worst = worst.max(calls);
        }
    }
    eprintln!("AC2 browse: {slots} offered slots all reached; {calls_total} list calls ({:.1} each, worst {worst})", calls_total as f64 / slots as f64);
    assert_eq!(slots, 25);
    assert!(worst <= 6, "a slot took {worst} list calls");
}

#[derive(Debug)]
struct Row {
    id: String,
    family: String,
    steps: Vec<Vec<IncomingCall>>,
    expect: Value,
}

fn pad(v: &Value) -> Value {
    match v {
        Value::String(s) if s.starts_with("@PAD") => json!("x".repeat(s[4..].parse::<usize>().expect("@PAD<n>"))),
        Value::Array(a) => Value::Array(a.iter().map(pad).collect()),
        Value::Object(m) => Value::Object(m.iter().map(|(k, x)| (k.clone(), pad(x))).collect()),
        other => other.clone(),
    }
}

fn corpus() -> Vec<Row> {
    let doc: Value = serde_json::from_str(include_str!("../../cartalith-llm/tests/fixtures/hostile_corpus_actions_2026-10-31.json")).expect("valid JSON");
    doc["rows"]
        .as_array()
        .expect("rows")
        .iter()
        .map(|r| Row {
            id: r["id"].as_str().unwrap().into(),
            family: r["family"].as_str().unwrap().into(),
            steps: r["steps"]
                .as_array()
                .unwrap()
                .iter()
                .map(|s| s["calls"].as_array().unwrap().iter().map(|c| call(c["name"].as_str().unwrap(), pad(&c["arguments"]))).collect())
                .collect(),
            expect: r["expect"].clone(),
        })
        .collect()
}

fn step_name(s: &Step) -> &'static str {
    match s {
        Step::Continue => "Continue",
        Step::ActionRetry(_) => "ActionRetry",
        Step::ActionProposal(_) => "ActionProposal",
        Step::Answer(_) => "Answer",
        Step::Failed(_) => "Failed",
        _ => "Other",
    }
}

/// The hostile corpus for the four tools (`hostile_corpus_actions_2026-10-31.json`, only grows): every row
/// fed to a real conversation over the real registry and real shell rows. Protects scope V6 and Ruling DO
/// for actions: a guessed, destructive, look-alike or unreturned id is never valid; an oversized, too
/// deep, mistyped or over-long proposal is refused with its rule; markup and control characters survive
/// only as inert text; no tool result ever contains an id with a forbidden word; and the one legitimate
/// row still produces its card. MUTATION: skip the ledger check in `validate` and A01, A02, A04, A18 and
/// A19 go red.
#[test]
fn the_hostile_corpus_for_the_catalogue_tools() {
    let rows = corpus();
    assert!(rows.len() >= 25, "the corpus only grows: {}", rows.len());
    let mut seen = std::collections::BTreeSet::new();
    for r in &rows {
        assert!(seen.insert(r.id.clone()), "duplicate corpus id {}", r.id);
        let (mut c, cat) = conv();
        let mut last: Option<Step> = None;
        for step in &r.steps {
            last = Some(c.on_reply(&cat, "", step.clone()));
        }
        let last = last.unwrap();
        let ex = &r.expect;
        assert_eq!(step_name(&last), ex["step"].as_str().unwrap(), "{} ({}): {last:?}", r.id, r.family);
        let verdict = match &last {
            Step::ActionRetry(v) | Step::ActionProposal(v) => Some(v),
            _ => None,
        };
        if let Some(rule) = ex.get("rule").and_then(Value::as_str) {
            let v = verdict.unwrap_or_else(|| panic!("{}: a rule was expected", r.id));
            let got = v.structural.as_ref().map(|s| s.0.id()).or_else(|| v.rejected.first().map(|x| x.rule.id()));
            assert_eq!(got, Some(rule), "{}: {v:?}", r.id);
        }
        if let Some(valid) = ex.get("valid").and_then(Value::as_array) {
            let want: Vec<&str> = valid.iter().map(|x| x.as_str().unwrap()).collect();
            let got: Vec<&str> = verdict.map(|v| v.valid.iter().map(|x| x.id.as_str()).collect()).unwrap_or_default();
            assert_eq!(got, want, "{}", r.id);
            // A valid action never carries a destructive id, whatever the row.
            for id in &got {
                assert_eq!(refusal(id, ""), None, "{}: {id}", r.id);
            }
        }
        let results: Vec<String> = c.messages().iter().filter(|m| m.tool.is_some() && m.role == cartalith_llm::openai::Role::User).map(|m| m.content.clone()).collect();
        let last_result = results.last().cloned().unwrap_or_default();
        if let Some(s) = ex.get("last_result_has").and_then(Value::as_str) {
            assert!(last_result.contains(s), "{}: {s:?} not in {last_result}", r.id);
        }
        if let Some(s) = ex.get("last_result_lacks").and_then(Value::as_str) {
            assert!(!last_result.contains(s), "{}: {s:?} must not be in {last_result}", r.id);
        }
        if ex.get("no_forbidden_ids_in_results").and_then(Value::as_bool) == Some(true) {
            assert!(!results.is_empty(), "{}: a row asserting clean results must have results", r.id);
            for res in results.iter().filter(|r| !r.contains("\"status\":\"rejected\"")) {
                for id in ids_in(res) {
                    assert_eq!(refusal(&id, ""), None, "{}: the result offered {id}", r.id);
                }
            }
        }
    }
}

/// Every string value under an `"id"` key in a tool result.
fn ids_in(text: &str) -> Vec<String> {
    fn walk(v: &Value, out: &mut Vec<String>) {
        match v {
            Value::Object(m) => {
                for (k, x) in m {
                    if k == "id" {
                        if let Some(s) = x.as_str() {
                            out.push(s.to_owned());
                        }
                    }
                    walk(x, out);
                }
            }
            Value::Array(a) => a.iter().for_each(|x| walk(x, out)),
            _ => {}
        }
    }
    let mut out = Vec::new();
    if let Ok(v) = serde_json::from_str::<Value>(text) {
        walk(&v, &mut out);
    }
    out
}

/// The full browse path on the real catalogue, in the order a model takes it, with the shared lookup caps:
/// top, down to a leaf, describe, propose: three model replies of five allowed, and the card is the
/// terminal step. Protects the end-to-end wiring of ledger, caps and verdict through `ParamsConversation`.
#[test]
fn browse_describe_propose_end_to_end_within_the_turn_cap() {
    let (mut c, cat) = conv();
    assert!(matches!(c.on_reply(&cat, "", vec![call("list_actions", json!({})), call("list_actions", json!({"path": "story/time"}))]), Step::Continue));
    assert!(c.action_ledger().knows("timeline.set_year"));
    assert!(matches!(c.on_reply(&cat, "", vec![call("describe_action", json!({"ids": ["timeline.set_year"]}))]), Step::Continue));
    let last = &c.messages().last().unwrap().content;
    assert!(last.contains("\"minimum\":-400") && last.contains("\"maximum\":1200"), "{last}");
    match c.on_reply(&cat, "", vec![call("propose_actions", json!({"summary": "to year 398", "actions": [{"id": "timeline.set_year", "args": {"year": 398}}]}))]) {
        Step::ActionProposal(v) => {
            assert_eq!(v.valid.len(), 1);
            assert_eq!(v.valid[0].args["year"], json!(398));
            assert_eq!(c.turns(), 3);
        }
        s => panic!("{s:?}"),
    }
    let req = c.request(&cat, "m", 1024, MaxTokensField::MaxTokens, false, Thinking::Default);
    assert!(req.tools.iter().any(|t| t.name == "propose_actions"));
    // The 13th lookup of a request is refused: the shared per-request cap.
    let (mut c, cat) = conv();
    for i in 0..3 {
        let calls: Vec<IncomingCall> = (0..6).map(|_| call("list_actions", json!({}))).collect();
        let _ = i;
        let s = c.on_reply(&cat, "", calls);
        if i < 2 {
            assert!(matches!(s, Step::Continue));
        }
    }
    let msgs: Vec<&str> = c.messages().iter().filter(|m| m.tool.is_some() && m.role == cartalith_llm::openai::Role::User).map(|m| m.content.as_str()).collect();
    assert!(msgs.iter().any(|m| m.contains("12 lookups this request may use")), "the 13th call is refused by the shared cap");
}

/// The first request's size on the real catalogue, as the guard measures it, as a literal range. Protects
/// the s.3.6 fixed-request target end to end (system prompt with the map, the old tools, the four new
/// ones) and that the guard passes it at 8 192 and refuses it at 2 048. The live model run records the
/// server's own prompt-token count beside this estimate in the AC-2 report.
#[test]
fn the_first_request_fits_an_8k_window_and_the_guard_says_so() {
    let (c, cat) = conv();
    let req = c.request(&cat, "qwen3-14b", 1024, MaxTokensField::MaxTokens, false, Thinking::Default);
    let est = estimate_prompt_tokens(&req);
    eprintln!("AC2 first Commands request, estimate: {est} tokens (guard, pessimistic)");
    assert!(est < 4500, "the first request with every tool: {est}");
    assert!(check_budget(&req, 8192).is_ok());
    assert!(check_budget(&req, 2048).is_err());
}
