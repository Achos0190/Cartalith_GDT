//! AC-1 / AC-2 unit tests over a small SYNTHETIC catalogue: the rules, the caps and the guard.
//! The same code over the REAL registry rows and the REAL `CommandIndex` rows is
//! `cartalith-godot/tests/ac2_actions.rs`.

use serde_json::{Value, json};

use crate::action_tools::*;
use crate::actions::{ActionCatalogue, ActionKind, Effect, ShellFacts};
use crate::catalogue::{Catalogue, Group, Kind, PValue, Regen, Row};
use crate::openai::ChatRequest;
use crate::view_actions::ViewCatalogue;

fn row(key: &str, group: &str, kind: Kind, min: f64, max: f64, label: &str, desc: &str) -> Row {
    Row {
        key: key.into(),
        group: group.into(),
        kind,
        min,
        max,
        step: 1.0,
        label: label.into(),
        unit: String::new(),
        desc: desc.into(),
        default: Some(PValue::Num(min)),
        current: Some(PValue::Num(min)),
        advanced: false,
        inert: None,
        regen: Regen::FullGenerate,
        from_reference: true,
    }
}

fn params() -> Catalogue {
    Catalogue {
        rows: vec![
            row("hydrology.sea_level", "hydrology", Kind::Float, 0.0, 1.0, "Sea level", "Raise: more ocean. Lower: more land."),
            row("world.peak_height", "world", Kind::Float, 100.0, 9000.0, "Peak height", "Raise: taller mountains. Lower: lower ones."),
            row("civ.factions", "civ", Kind::Int, 1.0, 40.0, "Factions", "Raise: more factions. Lower: fewer."),
        ],
        excluded: vec![],
        groups: vec![
            Group { name: "hydrology".into(), blurb: "water".into() },
            Group { name: "world".into(), blurb: "shape".into() },
            Group { name: "civ".into(), blurb: "peoples".into() },
        ],
    }
}

fn view() -> ViewCatalogue {
    ViewCatalogue::from_json(&json!({
        "layers": [{"id": "roads", "label": "Ways & routes", "on": true}, {"id": "rivers", "label": "Rivers", "on": false}],
        "presets": [{"id": "natural_vibrant", "label": "Natural Vibrant", "on": true}, {"id": "satellite_relief", "label": "Satellite relief", "on": false}],
        "windows": [{"id": "vault", "label": "Markdown vault"}, {"id": "factions", "label": "Factions"}],
        "camera": {"w": 256, "h": 164, "zoom_min": 0.4, "zoom_max": 64.0, "zoom_now": 1.0},
        "withheld": {"set_landmark_filter": "This world carries no landmarks to filter."}
    }))
    .unwrap()
}

fn shell() -> ShellFacts {
    ShellFacts::from_json_text(
        &json!({"has_world": true, "timeline": {"min": -400, "max": 1200}, "menu_rows": [
            {"name": "slicer", "title": "Sprite sheet slicer", "group": "ASSETS", "available": true, "why": ""},
            {"name": "factions", "title": "Factions", "group": "RECORDS", "window": "factions", "available": true, "why": ""},
            {"name": "save", "title": "Save", "group": "FILE", "available": true, "why": ""},
            {"name": "export_png", "title": "Export PNG", "group": "FILE", "available": true, "why": ""},
            {"name": "factions_table", "title": "Faction table", "group": "RECORDS", "available": true, "why": ""},
            {"name": "help_about", "title": "About", "group": "HELP", "available": false, "why": "No window yet."}
        ]})
        .to_string(),
    )
    .unwrap()
}

fn cat() -> ActionCatalogue {
    ActionCatalogue::build(Some(&params()), Some(&view()), &shell())
}

/// Protects AC-1's "ids identical across two builds": the same facts in a different row order give
/// byte-identical entries. MUTATION: drop the `sort_by` in `build` and the reversed build differs.
#[test]
fn the_catalogue_is_deterministic_under_input_order() {
    let a = cat();
    let mut s = shell();
    s.menu_rows.reverse();
    let b = ActionCatalogue::build(Some(&params()), Some(&view()), &s);
    assert_eq!(a, b);
    let ids: Vec<&str> = a.entries().iter().map(|e| e.id.as_str()).collect();
    let mut sorted = ids.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(sorted.len(), ids.len(), "ids are unique");
}

/// Protects the AC-1 bar on every entry: a description of 1..=120 characters, a path of at most
/// three levels, an id of at most 64 characters.
#[test]
fn every_entry_meets_the_bar() {
    for e in cat().entries() {
        assert!(!e.desc.is_empty() && e.desc.chars().count() <= 120, "{}: {:?}", e.id, e.desc);
        assert!((1..=3).contains(&e.path.split('/').count()), "{}: {}", e.id, e.path);
        assert!(!e.id.is_empty() && e.id.chars().count() <= 64);
    }
}

/// Protects "destructive refused by construction": Save and Export (both pinnable in the real index)
/// are dropped and recorded; an unreviewed row is listed but cannot be proposed; a window row is
/// merged. MUTATION: remove "save" from the stems and `menu:save` becomes an entry.
#[test]
fn destructive_rows_never_become_entries_and_unreviewed_ones_cannot_be_proposed() {
    let c = cat();
    assert!(c.get("menu:save").is_none() && c.get("menu:export_png").is_none());
    let why = |id: &str| c.dropped.iter().find(|d| d.id == id).map(|d| d.why.clone());
    assert_eq!(why("menu:save").as_deref(), Some("contains the forbidden word 'save'"));
    assert_eq!(why("menu:export_png").as_deref(), Some("contains the forbidden word 'export'"));
    assert_eq!(why("menu:factions").as_deref(), Some("merged into window.factions"));
    let slicer = c.get("menu:slicer").unwrap();
    assert_eq!((slicer.effect, slicer.available), (Effect::OpensWindow, true));
    let unrev = c.get("menu:factions_table").unwrap();
    assert_eq!((unrev.effect, unrev.available), (Effect::Unreviewed, false));
    let about = c.get("menu:help_about").unwrap();
    assert!(!about.available && about.why == "No window yet.", "the index's own reason is kept");
    let mut l = ActionLedger::default();
    let _ = c.search("faction table", None, None, &mut l);
    let v = c.validate(&json!({"actions": [{"id": "menu:factions_table"}]}), &l);
    assert_eq!(v.rejected[0].rule, ActionRule::B7, "{v:?}");
}

/// Protects the tree bar: categories with counts at the top, an unknown or too-deep path answers
/// with an error, the leaf lists actions one line each and joins the ledger.
#[test]
fn list_browses_the_tree_and_notes_what_it_returned() {
    let c = cat();
    let mut l = ActionLedger::default();
    let top = c.list("", None, 0, &mut l);
    let cats: Vec<&str> = top["entries"].as_array().unwrap().iter().map(|e| e["path"].as_str().unwrap()).collect();
    assert_eq!(cats, vec!["carto", "civil", "map", "more", "story", "window", "world"]);
    assert!(l.is_empty(), "categories return no action ids");
    let g = c.list("world/generation", None, 0, &mut l);
    assert_eq!(g["entries"].as_array().unwrap().len(), 2, "{g}");
    assert!(c.list("a/b/c/d", None, 0, &mut l)["error"].as_str().unwrap().contains("no such category"));
    assert!(c.list("nowhere", None, 0, &mut l)["error"].as_str().unwrap().contains("the top categories are: carto"));
    let leaf = c.list("carto/layers", None, 0, &mut l);
    assert_eq!(leaf["entries"].as_array().unwrap().len(), 3, "two layers and the landmark filter (withheld, listed unavailable): {leaf}");
    assert!(l.knows("view.layer.rivers") && l.knows("view.layer.roads"));
    let only_view = c.list("map", Some(ActionKind::Window), 0, &mut l);
    assert!(only_view["entries"].as_array().unwrap().is_empty(), "no window-kind action under map: {only_view}");
}

/// Protects the page caps with a leaf larger than one page (30 parameters under one group): at most 25
/// rows AND at most 1 400 characters of rows a page, so these 30 split 24 + 6 (the character cap binds
/// first), nothing is lost across pages, every page's id joins the ledger, and a page past the end is an
/// error naming the page count. MUTATION: remove the character cap and the first page has 25 rows.
#[test]
fn a_leaf_of_30_pages_by_rows_and_by_characters() {
    let mut p = params();
    for i in 0..30 {
        p.rows.push(row(&format!("big.k{i:02}"), "big", Kind::Float, 0.0, 1.0, &format!("Knob {i}"), "Raise: more. Lower: less."));
    }
    let c = ActionCatalogue::build(Some(&p), None, &ShellFacts::default());
    let mut l = ActionLedger::default();
    let p0 = c.list("world/generation/big", None, 0, &mut l);
    assert_eq!(p0["entries"].as_array().unwrap().len(), 24, "{p0}");
    assert_eq!(p0["next_page"], json!(1));
    assert!(p0.to_string().len() <= MAX_LIST_CHARS + 120);
    let p1 = c.list("world/generation/big", None, 1, &mut l);
    assert_eq!(p1["entries"].as_array().unwrap().len(), 6);
    assert!(p1.get("next_page").is_none());
    assert_eq!(l.len(), 30);
    assert!(c.list("world/generation/big", None, 2, &mut l)["error"].as_str().unwrap().contains("this list has 2 pages"));
    // With short rows the 25-row cap binds instead.
    let mut q = params();
    for i in 0..30 {
        q.rows.push(row(&format!("b.{i}"), "b", Kind::Float, 0.0, 1.0, "K", "Raise: more. Lower: less."));
    }
    let d = ActionCatalogue::build(Some(&q), None, &ShellFacts::default());
    let r = d.list("world/generation/b", None, 0, &mut ActionLedger::default());
    assert_eq!(r["entries"].as_array().unwrap().len(), 25, "{r}");
}

/// Protects search: the AC-0 misses are fixed by aliases ("road" finds the Way tool, "mountains"
/// finds Peak height), at most 8 matches, deterministic, a group filter narrows it.
#[test]
fn search_finds_by_alias_and_caps_at_eight() {
    let c = cat();
    let mut l = ActionLedger::default();
    let r = c.search("make the mountains higher", None, None, &mut l);
    assert_eq!(r["matches"][0]["id"], json!("param:world.peak_height"), "{r}");
    assert!(l.knows("param:world.peak_height"));
    let r = c.search("draw a new road", None, None, &mut l);
    assert_eq!(r["matches"][0]["id"], json!("tool.way"), "{r}");
    assert!(c.search("the of a", None, None, &mut l)["error"].is_string(), "all stop words is an error, not 'everything'");
    let many = c.search("view layer", None, None, &mut l);
    assert!(many["matches"].as_array().unwrap().len() <= 8);
    assert_eq!(c.search("rivers", None, None, &mut ActionLedger::default()), c.search("rivers", None, None, &mut ActionLedger::default()));
    assert!(c.search("rivers", Some("civil"), None, &mut ActionLedger::default())["matches"].as_array().unwrap().is_empty(), "the group filter narrows");
}

/// Protects the known-id rule on `describe_action`: an id never returned gets the rule, not a schema.
#[test]
fn describe_refuses_an_id_that_was_not_returned_and_caps_at_three() {
    let c = cat();
    let mut l = ActionLedger::default();
    let r = c.describe(&["menu:save".into()], &l);
    assert!(r["actions"][0]["error"].as_str().unwrap().contains("was not returned"), "{r}");
    assert!(r["actions"][0].get("args").is_none());
    let _ = c.list("carto/layers", None, 0, &mut l);
    let ok = c.describe(&["view.layer.rivers".into()], &l);
    assert_eq!(ok["actions"][0]["effect"], json!("view"));
    assert_eq!(ok["actions"][0]["args"]["required"], json!(["visible"]));
    assert_eq!(ok["actions"][0]["state"], json!("hidden"));
    let four: Vec<String> = ["view.layer.rivers", "view.layer.roads", "a", "b"].iter().map(|s| s.to_string()).collect();
    let r = c.describe(&four, &l);
    assert_eq!(r["actions"].as_array().unwrap().len(), 3);
    assert!(r["note"].as_str().unwrap().contains("first 3"));
}

/// Protects every proposal rule B1..B9 with literals; nothing is clamped.
#[test]
fn propose_rules_b1_to_b9() {
    let c = cat();
    let mut l = ActionLedger::default();
    for q in ["rivers layer", "sea level", "peak height", "factions", "go to year", "zoom", "faction table", "style preset", "vault"] {
        let _ = c.search(q, None, None, &mut l);
    }
    let rule = |v: Value| -> (Option<ActionRule>, Vec<ActionRule>) {
        let x = c.validate(&v, &l);
        (x.structural.map(|s| s.0), x.rejected.iter().map(|r| r.rule).collect())
    };
    assert_eq!(rule(json!([])).0, Some(ActionRule::B1));
    assert_eq!(rule(json!({"actions": [], "x": 1})).0, Some(ActionRule::B2));
    assert_eq!(rule(json!({"actions": []})).0, Some(ActionRule::B2));
    let six: Vec<Value> = (0..6).map(|_| json!({"id": "tool.way"})).collect();
    assert_eq!(rule(json!({"actions": six})).0, Some(ActionRule::B3));
    assert_eq!(rule(json!({"actions": [{"id": "x", "args": {"a": {"b": {"c": 1}}}}]})).0, Some(ActionRule::B1), "too deep");
    assert_eq!(rule(json!({"actions": [5]})).1, vec![ActionRule::B4]);
    assert_eq!(rule(json!({"actions": [{"id": "view.layer.rivers", "extra": 1}]})).1, vec![ActionRule::B4]);
    assert_eq!(rule(json!({"actions": [{"id": "view.layer.rivers", "args": 3}]})).1, vec![ActionRule::B4]);
    assert_eq!(rule(json!({"actions": [{"id": "view.layer.nope"}]})).1, vec![ActionRule::B6], "an unknown id reads as 'not returned', so ids cannot be probed");
    assert_eq!(rule(json!({"actions": [{"id": "menu:save"}]})).1, vec![ActionRule::B6]);
    assert_eq!(rule(json!({"actions": [{"id": "view.layer.rivers", "args": {"visible": 1}}]})).1, vec![ActionRule::B8], "1 is not a boolean");
    assert_eq!(rule(json!({"actions": [{"id": "view.layer.rivers", "args": {"visible": "true"}}]})).1, vec![ActionRule::B8]);
    assert_eq!(rule(json!({"actions": [{"id": "view.layer.rivers"}]})).1, vec![ActionRule::B8], "missing argument");
    assert_eq!(rule(json!({"actions": [{"id": "view.layer.rivers", "args": {"visible": true, "x": 1}}]})).1, vec![ActionRule::B8]);
    assert_eq!(rule(json!({"actions": [{"id": "param:hydrology.sea_level", "args": {"value": 1.5}}]})).1, vec![ActionRule::B8], "out of range is refused, not clamped to 1.0");
    assert_eq!(rule(json!({"actions": [{"id": "param:civ.factions", "args": {"value": 3.5}}]})).1, vec![ActionRule::B8], "an int setting takes a whole number");
    assert_eq!(rule(json!({"actions": [{"id": "timeline.set_year", "args": {"year": 1201}}]})).1, vec![ActionRule::B8]);
    assert_eq!(rule(json!({"actions": [{"id": "view.style_preset", "args": {"preset": "natural_vibrnt"}}]})).1, vec![ActionRule::B8]);
    assert_eq!(
        rule(json!({"actions": [{"id": "view.layer.rivers", "args": {"visible": true}}, {"id": "view.layer.rivers", "args": {"visible": false}}]})).1,
        vec![ActionRule::B9]
    );
    assert_eq!(rule(json!({"actions": [{"id": "menu:factions_table"}]})).1, vec![ActionRule::B7]);
    let ok = c.validate(
        &json!({"summary": "x\u{0007}y", "actions": [
            {"id": "param:hydrology.sea_level", "args": {"value": 0.25}, "reason": "lower the sea"},
            {"id": "timeline.set_year", "args": {"year": 398}},
            {"id": "view.zoom_to", "args": {"x": 10, "y": 20.5}},
            {"id": "window.factions"}]}),
        &l,
    );
    assert!(ok.structural.is_none() && ok.rejected.is_empty(), "{ok:?}");
    assert_eq!(ok.valid.len(), 4);
    assert_eq!(ok.valid[0].args["value"], json!(0.25), "the value is exactly as written");
    assert_eq!(ok.valid[0].from.as_deref(), Some("now 0.0"), "the old value is the app's, in words");
    assert_eq!(ok.summary.as_deref(), Some("x y"), "control characters are made inert");
    assert!(!ok.wants_retry());
    assert!(c.validate(&json!({"actions": [{"id": "menu:save"}]}), &l).wants_retry());
}

fn request(sys: String, cat: &ActionCatalogue) -> ChatRequest {
    use crate::openai::{MaxTokensField, Message, Role, Thinking};
    ChatRequest {
        model: "m".into(),
        messages: vec![Message::text(Role::System, sys), Message::text(Role::User, "hi")],
        tools: cat.tool_specs(),
        max_tokens: 1024,
        max_tokens_field: MaxTokensField::MaxTokens,
        include_usage: false,
        thinking: Thinking::Default,
    }
}

/// Protects the guard with literals: a request over the window is refused with the numbers; one that
/// fits passes; a window below the 8 192 design passes with a warning; window 0 is refused; the
/// answer reserve counts. MUTATION: drop the reserve from the sum and the 1100-token case passes.
#[test]
fn the_guard_refuses_with_numbers_and_never_truncates() {
    let c = cat();
    let small = request("You help.".into(), &c);
    let ok = check_budget(&small, 8192).unwrap();
    assert_eq!((ok.window, ok.reserve, ok.warning.clone()), (8192, 1024, None));
    assert!(ok.prompt_tokens > 300 && ok.prompt_tokens < 2000, "the four tools alone are well under the fixed target: {}", ok.prompt_tokens);
    let big = request("x".repeat(30_000), &c);
    let err = check_budget(&big, 8192).unwrap_err();
    assert!(err.contains("not sent") && err.contains("8192-token window") && err.contains("1024 kept"), "{err}");
    assert!(check_budget(&big, 32_768).is_ok(), "the same request fits a 32k window");
    assert!(check_budget(&small, 4096).unwrap().warning.unwrap().contains("designed for 8192"));
    assert!(check_budget(&small, 0).unwrap_err().contains("context size is not known"));
    assert!(check_budget(&small, 1100).is_err(), "prompt plus the answer reserve must fit");
}
