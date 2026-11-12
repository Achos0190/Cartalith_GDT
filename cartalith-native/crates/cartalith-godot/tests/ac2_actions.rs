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
    assert_eq!(by("params"), 115, "every offered registry row is an entry");
    assert_eq!(by("view"), 10 + 3, "ten layers plus the style preset, zoom and the (withheld) landmark filter");
    assert_eq!(by("window"), 10);
    assert_eq!(by("context"), 26, "31 scanned minus 5 refused: discard x3, erase and the sculpt draft's undo (AC-5 allow-listed region_start and river_clear; world.sculpt_here, MS-5, passes unexempted)");
    assert_eq!(by("tool"), 13);
    assert_eq!(by("timeline"), 1);
    assert_eq!(by("menu"), 32, "66 rows less 10 merged into windows and 24 refused (AC-5 allow-listed undo_history)");
    assert_eq!(c.entries().len(), 115 + 13 + 10 + 26 + 13 + 1 + 32);
    assert_eq!(c.derived_desc_count(), 0, "AC-5: every real row carries a written description (the work list was 27 at AC-5's start)");
    // Accounting: each of the 66 rows is an entry, a merge or a refusal.
    for r in &facts.menu_rows {
        let id = format!("menu:{}", r.name);
        let n = usize::from(c.get(&id).is_some()) + c.dropped.iter().filter(|d| d.id == id).count();
        assert_eq!(n, 1, "{id}: exactly one of entry / merge / refusal");
    }
    let merged = c.dropped.iter().filter(|d| d.why.starts_with("merged into ")).count();
    let refused = c.dropped.iter().filter(|d| d.id.starts_with("menu:") && d.why.starts_with("contains the forbidden")).count();
    assert_eq!((merged, refused), (10, 24));
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
        "win_reset", "win_save_layout", "help_report", "help_gen_info",
    ] {
        assert!(c.get(&format!("menu:{name}")).is_none(), "menu:{name} must not be an entry");
    }
    for e in c.entries() {
        // A parameter's title is the registry's own wording and is not checked (`carve_rivers` says
        // "on generation"); its id still is.
        let title = if e.source == "params" { "" } else { e.title.as_str() };
        if REFUSAL_EXEMPT.iter().any(|(i, ..)| *i == e.id) {
            continue; // AC-5's reviewed allow-list, pinned exactly by `the_refusal_allow_list_is_exact_reviewed_and_no_wider`.
        }
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
        // AC-5: a reviewed row the stem check would drop is a dead row UNLESS it is on the exact-id allow-list.
        let exempt = REFUSAL_EXEMPT.iter().any(|(i, ..)| *i == format!("menu:{name}"));
        assert!(refusal(&format!("menu:{name}"), "").is_none() || exempt, "{name} is reviewed but refused: a dead row");
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
        let exempt = REFUSAL_EXEMPT.iter().any(|(i, ..)| i == id);
        assert!(refusal(id, label).is_none() || exempt, "{id} is reviewed but its id or label is refused");
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

// ---------------------------------------------------------------------------------------------
// AC-3: running a card (`cartalith_llm::action_run`, the rules the shell asks before it runs anything)
// ---------------------------------------------------------------------------------------------

use cartalith_llm::action_run::{ExecContext, ExecItem, check_card};

/// The real catalogue with the AC-3 selection facts the 2026-10-31 fixture predates.
fn real_catalogue_with_selection(settlements: Option<i64>, can_back: Option<bool>) -> ActionCatalogue {
    let mut f = shell_facts();
    f.settlements = settlements;
    f.can_select_back = can_back;
    ActionCatalogue::build(Some(&params_catalogue()), Some(&real_view()), &f)
}

/// Protects "a card never offers what Apply would refuse" over the REAL rows: every entry the validator
/// would accept (available, reviewed) has a click path, and the families map to the paths the proposal
/// names (settings -> write_param, views -> LlmView, windows and menu rows -> CommandIndex, the year ->
/// tl_set_year, selection -> DccApp.select, tools -> arm_tool). The counts are literals (measured
/// 2026-11-12 on the 115 registry rows (114 until Ruling ET's `lake_balance`), the 66 captured menu rows and the shell source's own layers,
/// presets, windows, context actions and tools). MUTATION: delete the `window.` arm of
/// `ActionEntry::route` and the window count goes to 0.
#[test]
fn ac3_every_proposable_entry_has_a_click_path_and_the_families_are_counted() {
    let c = real_catalogue_with_selection(Some(40), Some(true));
    let mut by: std::collections::BTreeMap<&'static str, usize> = Default::default();
    for e in c.entries() {
        let proposable = e.available && e.effect != Effect::Unreviewed;
        if proposable {
            assert_ne!(e.route(), Route::None, "{} is proposable but nothing can run it", e.id);
            *by.entry(e.route().as_str()).or_default() += 1;
        }
    }
    eprintln!("AC3 proposable by route: {by:?}");
    assert_eq!(by.get("param"), Some(&115));
    assert_eq!(by.get("window"), Some(&10));
    assert_eq!(by.get("tool"), Some(&13));
    assert_eq!(by.get("year"), Some(&1));
    assert_eq!(by.get("select"), Some(&3));
    assert_eq!(by.get("view"), Some(&12), "ten layers, the style preset and zoom; the landmark filter is withheld on this world");
    assert!(by.get("menu").copied().unwrap_or(0) > 0, "the reviewed menu rows that are available");
    let ctx: Vec<&ActionEntry> = c.entries().iter().filter(|e| e.source == "context").collect();
    assert_eq!(ctx.len(), 26);
    assert!(ctx.iter().all(|e| !e.available), "no context action can be proposed");
    assert!(ctx.iter().filter(|e| e.effect != Effect::Unreviewed).all(|e| e.why == CONTEXT_NOT_RUNNABLE));
}

/// Protects the Session-mode scope with literals: every setting and the three WORLD tools are gated;
/// windows, views, selection, the year and the other tools are not. MUTATION: drop the `world/` arm of
/// `session_gated` and `tool.sculpt` goes red; drop the setting arm and the `param:` assertion goes red.
#[test]
fn ac3_session_mode_gates_settings_and_world_tools_only() {
    let c = real_catalogue_with_selection(Some(40), Some(true));
    let gated: Vec<&str> =
        c.entries().iter().filter(|e| e.session_gated() && e.available && e.effect != Effect::Unreviewed && e.route() != Route::Param).map(|e| e.id.as_str()).collect();
    assert_eq!(gated, ["tool.paint", "tool.region", "tool.sculpt"]);
    assert!(c.entries().iter().filter(|e| e.route() == Route::Param).all(|e| e.session_gated()));
    for id in ["view.layer.rivers", "view.style_preset", "view.zoom_to", "window.factions", "timeline.set_year", "select.settlement", "select.none", "tool.way", "tool.inspect"] {
        assert!(!c.get(id).unwrap_or_else(|| panic!("{id}")).session_gated(), "{id}");
    }
}

/// Protects the selection verbs: offered only when the shell said the counts; the id bound is the count
/// less one (a literal 39); an empty world or no earlier selection makes the entry unavailable WITH a
/// reason, not absent and not offered; `select.none` is not refused as `clear`. MUTATION: change `(n - 1)`
/// to `n` in `selection()` and the bound goes red.
#[test]
fn ac3_selection_verbs_follow_the_shells_counts() {
    let c = real_catalogue_with_selection(Some(40), Some(true));
    let s = c.get("select.settlement").unwrap();
    assert_eq!(s.args_schema()["properties"]["id"]["maximum"], json!(39));
    assert!(s.available && c.get("select.none").unwrap().available && c.get("select.back").unwrap().available);
    assert!(s.check_args(json!({"id": 39}).as_object().unwrap()).is_ok());
    assert!(s.check_args(json!({"id": 40}).as_object().unwrap()).is_err());
    assert!(s.check_args(json!({"id": -1}).as_object().unwrap()).is_err());
    assert!(c.dropped.iter().all(|d| !d.id.starts_with("select.")), "no select verb was refused: {:?}", c.dropped);
    let none = real_catalogue_with_selection(None, None);
    assert!(none.entries().iter().all(|e| e.source != "select"), "not said, not offered");
    let empty = real_catalogue_with_selection(Some(0), Some(false));
    assert!(!empty.get("select.settlement").unwrap().available && empty.get("select.settlement").unwrap().why.contains("no settlements"));
    assert!(!empty.get("select.back").unwrap().available && empty.get("select.back").unwrap().why.contains("earlier selection"));
    assert!(empty.get("select.none").unwrap().available, "deselecting nothing is harmless");
}

type ExecRow = (String, Vec<ExecItem>, ExecContext, Vec<(String, String)>);

fn exec_corpus() -> Vec<ExecRow> {
    let doc: Value = serde_json::from_str(include_str!("../../cartalith-llm/tests/fixtures/hostile_corpus_exec_2026-11-07.json")).expect("valid JSON");
    doc["rows"]
        .as_array()
        .expect("rows")
        .iter()
        .map(|r| {
            let items = r["items"].as_array().unwrap().iter().map(|i| ExecItem::from_wire(i["id"].as_str(), i["route"].as_str(), i["gated"].as_bool())).collect();
            let c = &r["ctx"];
            let ctx = ExecContext {
                session_mode: c["session"].as_bool().unwrap_or(false),
                generating: c["generating"].as_bool().unwrap_or(false),
                finalized: c["finalized"].as_bool().unwrap_or(false),
            };
            let expect = r["expect"].as_array().unwrap().iter().map(|e| (e[0].as_str().unwrap_or("<card>").to_owned(), e[1].as_str().unwrap().to_owned())).collect();
            (r["id"].as_str().unwrap().to_owned(), items, ctx, expect)
        })
        .collect()
}

/// The hostile corpus for running a card (`hostile_corpus_exec_2026-11-07.json`, only grows). Protects the
/// C-rules over every state a card can meet; every rule C1..C7 appears in some row, so a rule that stops
/// firing cannot hide. MUTATION: delete the session test in `check_card` and X01 to X04 and X19 go red.
#[test]
fn ac3_the_hostile_corpus_for_running_a_card() {
    let rows = exec_corpus();
    assert!(rows.len() >= 19, "the corpus only grows: {}", rows.len());
    let mut seen = std::collections::BTreeSet::new();
    let mut rules_seen = std::collections::BTreeSet::new();
    for (id, items, ctx, expect) in &rows {
        assert!(seen.insert(id.clone()), "duplicate corpus id {id}");
        let got: Vec<(String, String)> = check_card(items, ctx).into_iter().map(|r| (r.id.unwrap_or_else(|| "<card>".into()), r.rule.id().to_owned())).collect();
        assert_eq!(&got, expect, "{id}");
        rules_seen.extend(got.into_iter().map(|g| g.1));
    }
    assert_eq!(rules_seen.into_iter().collect::<Vec<_>>(), ["C1", "C2", "C3", "C4", "C5", "C6", "C7"]);
}

/// End to end on the real catalogue: a model browses and proposes a mixed card (a setting, a layer, a
/// window and a selection); the verdict carries the click path and the Session flag of each; `check_card`
/// passes it when idle, refuses exactly the setting in Session mode, and refuses the card while generating.
/// A context action never reaches `valid` (B7, with the right-click reason). Protects that the verdict's
/// fields are the ones the rules read.
#[test]
fn ac3_a_proposed_mixed_card_is_checked_by_the_verdict_it_was_given() {
    let cat = params_catalogue();
    let acts = real_catalogue_with_selection(Some(40), Some(true));
    let mut c = ParamsConversation::new(&cat, "make it so").unwrap().with_queries(&cat, true).with_actions(&cat, Some(acts));
    let browse: Vec<IncomingCall> = vec![
        call("search_actions", json!({"query": "peak altitude"})),
        call("list_actions", json!({"path": "carto/layers"})),
        call("list_actions", json!({"path": "window/records"})),
        call("list_actions", json!({"path": "map/selection"})),
    ];
    assert!(matches!(c.on_reply(&cat, "", browse), Step::Continue));
    let setting = c.action_ledger().knows("param:peak_m");
    assert!(setting, "the search for peak height did not return the setting");
    for id in ["view.layer.rivers", "window.factions", "select.settlement"] {
        assert!(c.action_ledger().knows(id), "{id} was not returned; the test's browse is wrong");
    }
    let actions = json!([
        {"id": "param:peak_m", "args": {"value": 9000}},
        {"id": "view.layer.rivers", "args": {"visible": false}},
        {"id": "window.factions"},
        {"id": "select.settlement", "args": {"id": 7}},
    ]);
    let Step::ActionProposal(v) = c.on_reply(&cat, "", vec![call("propose_actions", json!({"summary": "mixed", "actions": actions}))]) else { panic!("expected a card") };
    assert_eq!(v.valid.len(), 4, "{:?}", v.rejected);
    let items: Vec<ExecItem> = v.valid.iter().map(|a| ExecItem::from_wire(Some(&a.id), Some(a.route.as_str()), Some(a.session_gated))).collect();
    assert_eq!(items.iter().map(|i| i.route.as_str()).collect::<Vec<_>>(), ["param", "view", "window", "select"]);
    let idle = ExecContext { session_mode: false, generating: false, finalized: false };
    assert_eq!(check_card(&items, &idle), vec![]);
    let session: Vec<(String, &str)> = check_card(&items, &ExecContext { session_mode: true, ..idle }).into_iter().map(|r| (r.id.unwrap(), r.rule.id())).collect();
    assert_eq!(session, [("param:peak_m".to_owned(), "C4")]);
    assert_eq!(check_card(&items, &ExecContext { generating: true, ..idle }).len(), 1);
    // A context action is listed but never proposable.
    let (mut c2, cat2) = conv();
    assert!(matches!(c2.on_reply(&cat2, "", vec![call("list_actions", json!({"path": "civil/here"}))]), Step::Continue));
    match c2.on_reply(&cat2, "", vec![call("propose_actions", json!({"actions": [{"id": "civ.start_way", "args": {"x": 1, "y": 1}}]}))]) {
        Step::ActionRetry(v) => {
            assert_eq!(v.rejected[0].rule.id(), "B7", "{v:?}");
            assert!(v.rejected[0].why.contains("right-click"));
        }
        s => panic!("a context action must be refused: {s:?}"),
    }
}

// ---------------------------------------------------------------------------------------------
// AC-4: action packs and pre-retrieval.  AC-5: descriptions, the refusal allow-list, Undo, and the
// hostile corpus for the catalogue itself.
// ---------------------------------------------------------------------------------------------

/// The real catalogue with every fact the live shell reports: 212 settlements and an earlier selection
/// (what the AC-3 probe measured), and an undo step named "Sculpt stamp".
fn full_facts(undo: Option<UndoFact>) -> ShellFacts {
    let mut f = shell_facts();
    f.settlements = Some(212);
    f.can_select_back = Some(true);
    f.undo = undo;
    f
}

fn full_catalogue() -> ActionCatalogue {
    ActionCatalogue::build(Some(&params_catalogue()), Some(&real_view()), &full_facts(Some(UndoFact { available: true, label: "Sculpt stamp".into() })))
}

/// AC-5 bar "every entry has a description, and none is a placeholder", over the real inputs with every
/// fact present. Protects: no entry is `Derived` (the 27 rows that were at AC-5's start are written); every
/// description is a sentence of at least four words, within 120 characters, that is not its title, not its
/// id and not a known placeholder shape ("Context-card action: X.", "X menu: Y.", TODO, TBD, lorem).
/// MUTATION: delete the `menu:copy` row from `DESC_ONLY` and the first assertion goes red; set a
/// description to its title and the shape check goes red.
#[test]
fn ac5_every_entry_has_a_real_description_and_none_is_a_placeholder() {
    let c = full_catalogue();
    let derived: Vec<&str> = c.entries().iter().filter(|e| e.desc_source == DescSource::Derived).map(|e| e.id.as_str()).collect();
    assert!(derived.is_empty(), "derived (placeholder-shaped) descriptions remain: {derived:?}");
    let norm = |s: &str| s.to_lowercase().trim_matches(|ch: char| !ch.is_alphanumeric()).to_owned();
    let shapes = ["context-card action:", " menu: ", "todo", "tbd", "fixme", "lorem", "placeholder", "no description", "xxx"];
    for e in c.entries() {
        let d = e.desc.trim();
        assert!(d.split_whitespace().count() >= 4, "{}: {d:?} is too short to describe anything", e.id);
        assert!(d.chars().count() <= MAX_DESC_CHARS, "{}: {d:?}", e.id);
        assert_ne!(norm(d), norm(&e.title), "{}: the description is the title", e.id);
        assert!(!d.contains(e.id.as_str()), "{}: the description is the id", e.id);
        for s in shapes {
            assert!(!d.to_lowercase().contains(s), "{}: {d:?} looks like a placeholder ({s})", e.id);
        }
    }
    // The curated table is not a dead table: every row names an entry that is really listed and really
    // unreviewed (a reviewed row has its own description and would never read this one).
    for (id, d) in DESC_ONLY {
        let e = c.get(id).unwrap_or_else(|| panic!("DESC_ONLY names {id}, which is not an entry on the real rows"));
        assert_eq!(e.effect, Effect::Unreviewed, "{id} is reviewed, so DESC_ONLY's text is never used");
        assert_eq!(&e.desc, d, "{id}");
    }
}

/// AC-5's allow-list, pinned: the exempt ids are exactly the four reviewed ones; each would be refused by
/// the untouched guard (so the list is not dead); each becomes an entry with the FIXED title, whatever the
/// shell said (a hostile shell title is replaced); and the neighbours the stems exist for are still refused:
/// the draft discards, the paint erase, the sculpt draft's undo, every destructive menu row, a row whose
/// name merely STARTS with an exempt id, and `menu:undo` itself. MUTATION: widen the match from the exact
/// id to `starts_with` and `menu:undo_history_x` appears; remove "undo" from `FORBIDDEN_STEMS` and
/// `refusal("edit.undo", ..)` becomes `None`, failing the first loop.
#[test]
fn the_refusal_allow_list_is_exact_reviewed_and_no_wider() {
    let ids: Vec<&str> = REFUSAL_EXEMPT.iter().map(|(i, ..)| *i).collect();
    assert_eq!(ids, ["edit.undo", "menu:undo_history", "carto.region_start", "world.river_clear"]);
    for (id, title, why) in REFUSAL_EXEMPT {
        assert!(refusal(id, "").is_some() || refusal(id, title).is_some(), "{id} is not refused by the guard, so the allow-list row is dead");
        assert!(why.len() > 20, "{id}: the reason must say why it is safe");
    }
    assert_eq!(refusal("edit.undo", ""), Some("undo"));
    let c = full_catalogue();
    for (id, title, _) in REFUSAL_EXEMPT {
        let e = c.get(id).unwrap_or_else(|| panic!("{id} should be an entry"));
        assert_eq!(&e.title, title, "{id}: the fixed title replaces the shell's");
    }
    for id in [
        "civ.territory_discard", "world.paint_discard", "world.sculpt_discard", "world.paint_erase", "world.sculpt_undo", "menu:undo", "menu:save", "menu:delete",
        "menu:undo_history_x", "edit.undo2", "menu:redo",
    ] {
        assert!(c.get(id).is_none(), "{id} must stay refused");
    }
    // A row that merely starts with an exempt id is not exempt.
    let mut f = full_facts(None);
    f.menu_rows.push(MenuRow { name: "undo_history_x".into(), title: "Undo history x".into(), group: "Edit".into(), window: None, available: true, why: String::new() });
    let c2 = ActionCatalogue::build(Some(&params_catalogue()), Some(&real_view()), &f);
    assert!(c2.get("menu:undo_history_x").is_none());
    assert!(c2.dropped.iter().any(|d| d.id == "menu:undo_history_x" && d.why.contains("forbidden")));
    // A hostile title on an exempt id is replaced, not trusted.
    for r in f.menu_rows.iter_mut().filter(|r| r.name == "undo_history") {
        r.title = "Delete every world".into();
    }
    let c3 = ActionCatalogue::build(Some(&params_catalogue()), Some(&real_view()), &f);
    assert_eq!(c3.get("menu:undo_history").unwrap().title, "Undo history");
}

/// AC-5's safe Undo: offered only when the shell said whether a step exists; `available` follows the
/// shell's answer with a reason; the step's label shows as state; it routes to Undo, is Edit/Undoable,
/// not Session-gated, runs through the validator only after a search returned it, and may not share a card
/// (rule C8). MUTATION: drop the `Route::Undo` arm in `route()` and the proposable assertion goes red;
/// drop the C8 clause in `check_card` and the shared-card assertion goes red.
#[test]
fn ac5_undo_is_offered_from_the_shells_fact_and_runs_alone() {
    let none = ActionCatalogue::build(Some(&params_catalogue()), Some(&real_view()), &full_facts(None));
    assert!(none.get("edit.undo").is_none(), "no fact, no entry: 'not said' is never 'nothing to undo'");
    let c = full_catalogue();
    let e = c.get("edit.undo").expect("edit.undo");
    assert_eq!((e.kind, e.effect, e.available, e.route(), e.session_gated()), (ActionKind::Edit, Effect::Undoable, true, Route::Undo, false));
    assert_eq!(e.state.as_deref(), Some("would undo: Sculpt stamp"));
    assert_eq!(e.path, "edit/history");
    let off = ActionCatalogue::build(Some(&params_catalogue()), Some(&real_view()), &full_facts(Some(UndoFact { available: false, label: "stale".into() })));
    let e = off.get("edit.undo").unwrap();
    assert!(!e.available && e.why == "There is nothing to undo." && e.state.is_none(), "{e:?}");
    // The AC-0 task t24, "Undo that": search returns it, the validator accepts it, check_card lets it run alone.
    let mut ledger = ActionLedger::default();
    let r = c.search("Undo that", None, None, &mut ledger);
    let top: Vec<&str> = r["matches"].as_array().unwrap().iter().map(|m| m["id"].as_str().unwrap()).collect();
    assert_eq!(top.first(), Some(&"edit.undo"), "{top:?}");
    let v = c.validate(&json!({"summary": "undo", "actions": [{"id": "edit.undo"}]}), &ledger);
    assert_eq!(v.valid.len(), 1, "{v:?}");
    let ctx = ExecContext { session_mode: false, generating: false, finalized: false };
    let item = |id: &str, route: &str| ExecItem::from_wire(Some(id), Some(route), Some(false));
    assert!(check_card(&[item("edit.undo", "undo")], &ctx).is_empty());
    let shared = check_card(&[item("view.layer.rivers", "view"), item("edit.undo", "undo")], &ctx);
    assert_eq!(shared.iter().map(|x| x.rule.id()).collect::<Vec<_>>(), ["C8"]);
    // Unavailable Undo is refused by the validator (B7-style availability), never proposed.
    let mut l2 = ActionLedger::default();
    off.search("undo", None, None, &mut l2);
    assert!(off.validate(&json!({"actions": [{"id": "edit.undo"}]}), &l2).valid.is_empty());
}

/// AC-5's hints: `describe_action` tells a small model what `select.settlement`'s `id` and the year
/// MEAN, which the qwen3-14b run lacked (it wrote `number` and `n`). Literal substrings. The validator
/// still rejects the words themselves. MUTATION: remove the hint and the first assertion goes red.
#[test]
fn ac5_describe_says_what_the_ids_and_years_mean() {
    let c = full_catalogue();
    let mut l = ActionLedger::default();
    c.search("select settlement", None, None, &mut l);
    c.search("go to the year", None, None, &mut l);
    let d = c.describe(&["select.settlement".into(), "timeline.set_year".into()], &l);
    let s = d.to_string();
    assert!(s.contains("counting from 0: a whole number such as 17, never a word"), "{s}");
    assert!(s.contains("as a whole number such as 398"), "{s}");
    for bad in ["number", "n", "17", "Vhal Serai"] {
        let v = c.validate(&json!({"actions": [{"id": "select.settlement", "args": {"id": bad}}]}), &l);
        assert!(v.valid.is_empty(), "{bad:?} must be refused: {v:?}");
    }
    // The size bar still holds with the hints: a describe of two entries stays well inside 300 tokens each.
    assert!(s.chars().count() <= 2 * 300 * 4, "{}", s.chars().count());
}

/// The scoped packs against the real catalogue: every pack id is a real, proposable entry, each pack
/// holds three to eight, the block is inside its character and token caps, names its origin, lists each
/// entry's arguments, and contains only available entries. Literal sizes measured 2026-11-08 (printed by
/// the test). MUTATION: add "menu:save" to the pack and the real-entry assertion goes red; raise
/// `MAX_PACK_ACTIONS` and the length assertion goes red.
#[test]
fn every_pack_id_is_a_real_entry_and_the_sizes_hold() {
    let c = full_catalogue();
    assert_eq!(PACKS.len(), 1, "only the Commands button sends the catalogue tools");
    for (scope, ids) in PACKS {
        assert!((3..=8).contains(&ids.len()), "{scope}: {}", ids.len());
        for id in *ids {
            let e = c.get(id).unwrap_or_else(|| panic!("pack {scope} names {id}, not an entry"));
            assert!(e.effect != Effect::Unreviewed && e.route() != Route::None, "{id} cannot be proposed");
        }
    }
    let p = c.build_pack(Some("commands"), None).expect("the commands pack");
    eprintln!("AC4 commands pack: {} ids, {} chars, ~{} tokens", p.ids.len(), p.text.chars().count(), p.approx_tokens);
    assert_eq!(p.ids, ["view.layer.territory", "view.style_preset", "timeline.set_year", "select.settlement", "window.factions", "view.zoom_to", "tool.way", "edit.undo"]);
    assert_eq!(p.from_search, 0);
    assert!(p.text.starts_with("<context kind=\"actions\" origin=\"pack\">\n") && p.text.ends_with("</context>\n"), "{}", p.text);
    assert!(p.text.chars().count() <= 2600 && p.approx_tokens <= 800, "{} chars, {} tokens", p.text.chars().count(), p.approx_tokens);
    assert_eq!((MAX_PACK_ACTIONS, PRE_RETRIEVE_HITS, MAX_PACK_CHARS), (8, 5, 2600), "the proposal's 'three to eight' and 'top five'");
    assert!(p.text.contains("- timeline.set_year | Go to year | Move the shared timeline cursor"), "{}", p.text);
    assert!(p.text.contains("takes: year (whole number -400..1200)"), "{}", p.text);
    assert!(p.text.contains("edit.undo | Undo the last change") && p.text.contains("would undo: Sculpt stamp"), "{}", p.text);
    // An unknown scope adds nothing; no scope and no request is no block at all.
    assert!(c.build_pack(Some("bogus"), None).is_none() && c.build_pack(None, None).is_none());
    // An unavailable entry is left out, never listed with a hope.
    let off = ActionCatalogue::build(Some(&params_catalogue()), Some(&real_view()), &full_facts(Some(UndoFact { available: false, label: String::new() })));
    assert!(!off.build_pack(Some("commands"), None).unwrap().ids.contains(&"edit.undo".to_owned()));
}

/// Pre-retrieval ranks the user's words with the SAME ranker as `search_actions`, takes the top five that
/// can run, puts them ahead of the scope's, never repeats an id, never exceeds eight, and falls back to the
/// scope when the request has no searchable words. MUTATION: take `PRE_RETRIEVE_HITS` + 1 hits and the
/// count assertion goes red; skip the dedupe and the unique assertion goes red.
#[test]
fn pre_retrieval_ranks_like_search_and_stays_inside_its_caps() {
    let c = full_catalogue();
    let p = c.build_pack(Some("commands"), Some("Make the map look like an old antique map")).unwrap();
    assert_eq!(p.ids[0], "view.style_preset", "{:?}", p.ids);
    assert!(p.from_search >= 1 && p.from_search <= 5 && p.ids.len() <= 8, "{:?}", p.ids);
    assert!(p.text.starts_with("<context kind=\"actions\" origin=\"search+pack\">"), "{}", p.text);
    let mut u = p.ids.clone();
    u.sort();
    u.dedup();
    assert_eq!(u.len(), p.ids.len(), "an id appears twice: {:?}", p.ids);
    let mut l = ActionLedger::default();
    let s = c.search("Make the map look like an old antique map", None, None, &mut l);
    assert_eq!(s["matches"][0]["id"], json!(p.ids[0]), "search and pre-retrieval rank alike");
    let only = c.build_pack(None, Some("Hide the rivers")).unwrap();
    assert!(only.text.contains("origin=\"search\"") && only.ids.len() <= 5 && only.ids[0] == "view.layer.rivers", "{:?}", only.ids);
    let stop = c.build_pack(Some("commands"), Some("the a an of to ...")).unwrap();
    assert_eq!((stop.from_search, stop.ids.len()), (0, 8), "no searchable word falls back to the scope");
}

/// A pack on a real conversation: the block leads the user's message, the system prompt names it, every id
/// in it is acceptable to `propose_actions` WITHOUT a list or search (it is "returned" by the pack), the
/// same proposal without the pack is refused (B6), `drop_action_pack` restores the request byte for byte
/// and takes the ids back out, and a conversation with no catalogue tools ignores the pack. This is the
/// guard's remedy ("drops the pack first, and says so"). MUTATION: skip `note_all` in `with_action_pack`
/// and the proposal is refused; skip `forget` and the dropped ids stay acceptable.
#[test]
fn a_pack_rides_the_message_fills_the_ledger_and_can_be_dropped() {
    let cat = params_catalogue();
    let actions = full_catalogue();
    let base = ParamsConversation::new(&cat, "go to the year 398").unwrap().with_queries(&cat, true).with_actions(&cat, Some(actions.clone()));
    let pack = actions.build_pack(Some("commands"), None).unwrap();
    let packed = base.clone().with_action_pack(&cat, pack.clone());
    assert!(packed.messages()[1].content.starts_with(&pack.text) && packed.messages()[1].content.ends_with("go to the year 398"), "{}", packed.messages()[1].content);
    let sys = &packed.messages()[0].content;
    assert!(sys.contains("The <context kind=\"actions\"> block in the user's message lists actions the app already found"), "{sys}");
    assert!(!base.messages()[0].content.contains("<context kind=\"actions\">"));
    for id in &pack.ids {
        assert!(packed.action_ledger().knows(id), "{id}");
    }
    let propose = |mut c: ParamsConversation| c.on_reply(&cat, "", vec![call("propose_actions", json!({"summary": "year", "actions": [{"id": "timeline.set_year", "args": {"year": 398}}]}))]);
    assert!(matches!(propose(packed.clone()), Step::ActionProposal(_)), "a pack id is acceptable without a lookup");
    assert!(matches!(propose(base.clone()), Step::ActionRetry(_)), "without the pack the same proposal is refused (B6)");
    let mut dropped = packed.clone();
    dropped.drop_action_pack(&cat);
    assert_eq!(dropped.messages(), base.messages(), "dropping restores the request byte for byte");
    assert!(dropped.action_ledger().is_empty() && dropped.action_pack().is_none());
    assert!(matches!(propose(dropped), Step::ActionRetry(_)), "dropped ids are no longer acceptable");
    let plain = ParamsConversation::new(&cat, "go to the year 398").unwrap().with_queries(&cat, true);
    assert_eq!(plain.clone().with_action_pack(&cat, pack).messages(), plain.messages(), "no catalogue tools, no block");
}

/// The first Commands request with the pack and with pre-retrieval still fits the 8 192 window AT THE
/// GUARD'S pessimistic ratio, and the pack costs what it says: between 150 and 800 tokens by the guard's
/// estimate. AC-2's per-call targets (list <= 400, search <= 400, describe <= 300, first request < 4 500)
/// are asserted unchanged by the AC-2 tests above, which run against this same code. MUTATION: double
/// `MAX_PACK_CHARS` and the 800-token ceiling goes red.
#[test]
fn the_first_request_with_a_pack_still_fits_the_window() {
    let cat = params_catalogue();
    let actions = full_catalogue();
    let base = ParamsConversation::new(&cat, "make the map look like an old antique map").unwrap().with_queries(&cat, true).with_actions(&cat, Some(actions.clone()));
    let est = |c: &ParamsConversation| estimate_prompt_tokens(&c.request(&cat, "qwen3-14b", 1024, MaxTokensField::MaxTokens, false, Thinking::Default));
    let plain = est(&base);
    let pack = actions.build_pack(Some("commands"), Some("make the map look like an old antique map")).unwrap();
    let packed = est(&base.clone().with_action_pack(&cat, pack));
    eprintln!("AC4 first request: {plain} tokens plain, {packed} with the pack and pre-retrieval (guard's estimate)");
    assert!((150..=800).contains(&(packed - plain)), "the pack costs {} tokens", packed - plain);
    assert!(packed < 4500);
    let req = base.clone().with_action_pack(&cat, actions.build_pack(Some("commands"), Some("make the map look like an old antique map")).unwrap()).request(&cat, "qwen3-14b", 1024, MaxTokensField::MaxTokens, false, Thinking::Default);
    assert!(check_budget(&req, 8192).is_ok());
}

/// AC-4's model-free half of "pre-retrieval of actions raises AC-0's local score": on AC-0's 30 tasks, how
/// many needed slots are IN the block the model is handed with no tool call at all, by the scope pack
/// alone, by pre-retrieval alone and by both. This measures REACH of the preloaded block, not a model's
/// choice, so it cannot by itself show a model scores higher; the AC-4 report pairs it with the real-model
/// run. Slots with no offered id are counted, not skipped (`select`, `selection.back` map to the AC-3
/// verbs, `menu:undo` to AC-5's `edit.undo`; the timeline's play and record have no shell code).
/// Literal floors measured 2026-11-08. MUTATION: rank by id only and the pre-retrieval floor goes red.
#[test]
fn ac4_preloaded_reach_on_ac0s_thirty_tasks() {
    let tasks: Value = serde_json::from_str(include_str!("../../../docs/evidence/assistant-actions-ac0-2026-10-31/tools/tasks.json")).unwrap();
    let c = full_catalogue();
    fn map<'a>(id: &'a str) -> &'a str {
        match id {
            "select" => "select.settlement",
            "selection.back" => "select.back",
            "menu:undo" => "edit.undo",
            other => other,
        }
    }
    let (mut needed, mut scope, mut pre, mut both, mut unoffered) = (0, 0, 0, 0, 0);
    let (mut declined_ok, mut declined) = (0, 0);
    for t in tasks["tasks"].as_array().unwrap() {
        let prompt = t["prompt"].as_str().unwrap();
        let sc = c.build_pack(Some("commands"), None).unwrap();
        let pr = c.build_pack(None, Some(prompt));
        let bo = c.build_pack(Some("commands"), Some(prompt)).unwrap();
        let slots = t["slots"].as_array().unwrap();
        if slots.is_empty() {
            // The two tasks the assistant must decline ("Save the project", "Delete the Pacific Ocean"):
            // the block must not hand it a destructive id.
            declined += 1;
            if bo.ids.iter().all(|i| refusal(i, "").is_none() || REFUSAL_EXEMPT.iter().any(|(x, ..)| x == i)) {
                declined_ok += 1;
            }
            continue;
        }
        for slot in slots {
            let ids: Vec<&str> = slot.as_array().unwrap().iter().map(|x| map(x.as_str().unwrap())).collect();
            if !ids.iter().any(|i| c.get(i).is_some()) {
                unoffered += 1;
                continue;
            }
            needed += 1;
            let has = |p: &ActionPack| ids.iter().any(|i| p.ids.iter().any(|x| x == i));
            scope += usize::from(has(&sc));
            pre += usize::from(pr.as_ref().is_some_and(has));
            both += usize::from(has(&bo));
        }
    }
    eprintln!("AC4 reach of the preloaded block: needed {needed} (unoffered {unoffered}); scope pack {scope}, pre-retrieval {pre}, both {both}; declined tasks kept clean {declined_ok}/{declined}");
    assert_eq!(declined, 2);
    assert_eq!(declined_ok, declined, "a decline task was handed a destructive id");
    assert!(needed >= 25, "{needed}");
    assert!(pre >= 22, "pre-retrieval floor: {pre}/{needed}");
    assert!(both >= pre && both >= scope, "{both} < {pre} or {scope}");
}

/// The hostile corpus for the CATALOGUE itself (`hostile_corpus_catalogue_2026-11-08.json`, only grows;
/// scope s.4 bar 3 extended by AC-5). Each row builds a real catalogue from hostile shell facts and a
/// hostile request, then reads back EVERYTHING a model is shown: every list page at every path, a search
/// per hostile title, a describe of the hostile entries, the capability map, the prompt paragraph and the
/// preloaded pack. Invariants for every row: (1) the id set differs from the baseline by exactly the row's
/// `new_ids` and `removed_ids`; (2) every id is `[a-z0-9._:]` and at most 64; (3) no entry or output
/// contains a control character; (4) titles <= 60, descriptions <= 120, states <= 72 ("would undo: " plus a label of at most 60); (5) no id is
/// destructive unless exact-id allow-listed, and an allow-listed entry has its fixed title; (6) the pack
/// has exactly one `<context` and one `</context>` (a hostile title cannot close or reopen the fence, in
/// any of the forms the LLM-9 corpus found: upper case, spaced, zero-width); (7) the request is never
/// copied into the pack; (8) `pack_has` and `pack_lacks` hold. MUTATION: drop `defuse_fence` from
/// `build_pack` and H02, H03 and H10 go red; drop `inert_text` from `MenuRow` parsing and H01 and H07 go red;
/// widen the allow-list match and H06 goes red.
#[test]
fn the_hostile_corpus_for_the_catalogue() {
    let doc: Value = serde_json::from_str(include_str!("../../cartalith-llm/tests/fixtures/hostile_corpus_catalogue_2026-11-08.json")).expect("valid JSON");
    let rows = doc["rows"].as_array().expect("rows");
    assert!(rows.len() >= 17, "the corpus only grows: {}", rows.len());
    let baseline: std::collections::BTreeSet<String> = full_catalogue().entries().iter().map(|e| e.id.clone()).collect();
    let mut seen = std::collections::BTreeSet::new();
    let lower = |s: &str| s.to_lowercase();
    let count = |hay: &str, needle: &str| hay.to_lowercase().matches(needle).count();
    for r in rows {
        let rid = r["id"].as_str().unwrap();
        assert!(seen.insert(rid.to_owned()), "duplicate corpus id {rid}");
        let mut json_facts: Value = serde_json::from_str(FACTS).unwrap();
        let rows_of = |k: &str| r.get(k).and_then(Value::as_array).cloned().unwrap_or_default();
        let mut menu: Vec<Value> = json_facts["menu_rows"].as_array().cloned().unwrap_or_default();
        for o in rows_of("override_rows") {
            menu.retain(|m| m["name"] != o["name"]);
            menu.push(o);
        }
        menu.extend(rows_of("extra_rows"));
        if let Some(d) = r.get("repeat_rows") {
            for _ in 0..d["times"].as_u64().unwrap() {
                menu.push(json!({"name": d["name"], "title": d["title"], "group": d["group"], "available": d["available"]}));
            }
        }
        json_facts["menu_rows"] = Value::Array(menu);
        json_facts["settlements"] = json!(212);
        json_facts["can_select_back"] = json!(true);
        json_facts["undo"] = r.get("undo").cloned().unwrap_or_else(|| json!({"available": true, "label": "Sculpt stamp"}));
        let facts = ShellFacts::from_json_text(&json_facts.to_string()).unwrap_or_else(|e| panic!("{rid}: the facts did not parse: {e}"));
        let view = match r.get("view") {
            Some(v) => ViewCatalogue::from_json(v).unwrap(),
            None => real_view(),
        };
        let c = ActionCatalogue::build(Some(&params_catalogue()), Some(&view), &facts);
        let ids: std::collections::BTreeSet<String> = c.entries().iter().map(|e| e.id.clone()).collect();
        let ex = &r["expect"];
        let strs = |k: &str| -> Vec<String> { ex.get(k).and_then(Value::as_array).map(|a| a.iter().map(|x| x.as_str().unwrap().to_owned()).collect()).unwrap_or_default() };
        if r.get("view").is_none() {
            let added: std::collections::BTreeSet<String> = ids.difference(&baseline).cloned().collect();
            let removed: std::collections::BTreeSet<String> = baseline.difference(&ids).cloned().collect();
            assert_eq!(added, strs("new_ids").into_iter().collect(), "{rid}: the id set grew or shrank by something the row does not expect");
            assert_eq!(removed, strs("removed_ids").into_iter().collect(), "{rid}: ids went missing");
        }
        // (2)-(5) per entry.
        for e in c.entries() {
            assert!(e.id.chars().count() <= MAX_ID_CHARS && e.id.chars().all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || "._:".contains(ch)), "{rid}: id {:?}", e.id);
            for (what, s, max) in [("title", e.title.as_str(), 60), ("desc", e.desc.as_str(), MAX_DESC_CHARS), ("why", e.why.as_str(), 130), ("state", e.state.as_deref().unwrap_or(""), 72)] {
                assert!(!s.chars().any(char::is_control), "{rid}: {} {what} has a control character: {s:?}", e.id);
                assert!(s.chars().count() <= max, "{rid}: {} {what} is {} characters", e.id, s.chars().count());
            }
            let exempt = REFUSAL_EXEMPT.iter().find(|(i, ..)| *i == e.id);
            match exempt {
                Some((_, t, _)) => assert_eq!(&e.title, t, "{rid}: {} must carry the fixed title", e.id),
                None => {
                    let title = if e.source == "params" { "" } else { e.title.as_str() };
                    assert_eq!(refusal(&e.id, title), None, "{rid}: {} / {}", e.id, e.title);
                }
            }
        }
        // Everything a model can read, as text.
        let mut shown: Vec<String> = Vec::new();
        let mut ledger = ActionLedger::default();
        let mut paths: Vec<String> = vec![String::new()];
        let mut i = 0;
        while i < paths.len() {
            let p = paths[i].clone();
            i += 1;
            let mut page = 0;
            loop {
                let out = c.list(&p, None, page, &mut ledger);
                if let Some(es) = out["entries"].as_array() {
                    for e in es {
                        if let Some(sub) = e["path"].as_str() {
                            if !paths.contains(&sub.to_owned()) {
                                paths.push(sub.to_owned());
                            }
                        }
                    }
                }
                shown.push(out.to_string());
                match out["next_page"].as_u64() {
                    Some(n) => page = n as usize,
                    None => break,
                }
            }
        }
        for e in c.entries().iter().filter(|e| e.source != "params").take(40) {
            shown.push(c.search(&e.title, None, None, &mut ledger).to_string());
        }
        let known: Vec<String> = c.entries().iter().filter(|e| ledger.knows(&e.id)).map(|e| e.id.clone()).take(60).collect();
        for ch in known.chunks(3) {
            shown.push(c.describe(ch, &ledger).to_string());
        }
        shown.push(c.capability_map());
        shown.push(c.prompt_paragraph());
        let request: String = match r.get("request_repeat") {
            Some(d) => d["unit"].as_str().unwrap().repeat(d["times"].as_u64().unwrap() as usize),
            None => r.get("request").and_then(Value::as_str).unwrap_or("").to_owned(),
        };
        let pack = c.build_pack(Some("commands"), (!request.is_empty()).then_some(request.as_str()));
        if let Some(p) = &pack {
            shown.push(p.text.clone());
            assert!(p.ids.len() <= 8 && p.text.chars().count() <= 2600, "{rid}: pack size");
            assert_eq!(count(&p.text, "<context"), 1, "{rid}: the pack reopens a context block:\n{}", p.text);
            assert_eq!(count(&p.text, "</context>"), 1, "{rid}: the pack closes its own fence early:\n{}", p.text);
            for id in &p.ids {
                assert!(ids.contains(id), "{rid}: the pack names {id}, not an entry");
            }
            for s in strs("pack_lacks") {
                assert!(!lower(&p.text).contains(&lower(&s)), "{rid}: the pack contains {s:?}:\n{}", p.text);
            }
            // A one-word request ("places") is legitimately a word of the catalogue's own text, so the
            // "copied verbatim" check is for requests of three words or more.
            if request.split_whitespace().count() >= 3 && request.chars().count() < 400 {
                assert!(!p.text.contains(request.trim()), "{rid}: the request was copied into the pack");
            }
        } else {
            assert!(strs("pack_has").is_empty(), "{rid}: expected a pack");
        }
        for s in strs("pack_has") {
            assert!(pack.as_ref().is_some_and(|p| p.text.contains(&s)), "{rid}: the pack lacks {s:?}");
        }
        // The system paragraph and the capability map are built from the catalogue alone.
        for s in &shown {
            assert!(!s.chars().any(|ch| ch.is_control() && ch != '\n'), "{rid}: a control character reached the model:\n{s:?}");
        }
    }
}
