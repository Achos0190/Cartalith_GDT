//! LLM-6a and LLM-6b over the REAL registry rows (`LLM_ASSISTANT_SCOPE.md`
//! s.5b.1, s.5b.2): the catalogue the assistant is shown, built from
//! `params::PARAMS` and `param_catalogue.rs`'s side tables exactly as the bridge
//! builds it, and the validator run over it.
//!
//! Pulled in by `#[path]`, the `tests/params_mapping.rs` pattern: both files are
//! godot-free, so this runs under a plain `cargo test` with no engine process.
#![allow(dead_code)]

#[path = "../src/params.rs"]
mod params;
#[path = "../src/param_catalogue.rs"]
mod param_catalogue;

use cartalith_llm::catalogue::fuzz::{applies_nothing, hostile_corpus, valid_corpus};
use cartalith_llm::catalogue::{Catalogue, Kind, PValue, Regen, Rule};
use cartalith_llm::params_flow::{IncomingCall, ParamsConversation, Step};
use serde_json::{Value, json};

/// The catalogue as a fresh app world sees it: current = the shipped defaults.
fn fresh() -> Catalogue {
    let d = params::defaults();
    param_catalogue::build(&|k| {
        params::get(&d, k).map(|v| match v {
            params::Value::Bool(b) => PValue::Bool(b),
            params::Value::Num(n) => PValue::Num(n),
        })
    })
}

fn all_groups(c: &Catalogue) -> Vec<String> {
    c.group_names()
}

/// Protects the LLM-6a bar: `DESCS` has a non-empty entry for 100% of the
/// OFFERED keys, naming any key missing one; nothing in `DESCS` or `INERT_WHEN`
/// names a key the registry lacks (a stale row); the registry still has the 120
/// rows of 2026-10-26 and exactly six are excluded. MUTATION: delete one `DESCS`
/// row and this goes red naming it.
#[test]
fn every_offered_key_has_a_description_and_no_row_is_stale() {
    assert_eq!(params::PARAMS.len(), 120, "registry rows counted 2026-10-26; re-read the scope's counts if this moves");
    let offered: Vec<&str> =
        params::PARAMS.iter().map(|s| s.key).filter(|k| !param_catalogue::EXCLUDED.contains(k)).collect();
    assert_eq!(offered.len(), 114);
    let missing: Vec<&&str> = offered.iter().filter(|k| param_catalogue::desc(k).is_none_or(|d| d.trim().is_empty())).collect();
    assert!(missing.is_empty(), "offered keys with no description: {missing:?}");
    for (k, _) in param_catalogue::DESCS {
        assert!(params::spec(k).is_some(), "DESCS row for a key the registry does not have: {k}");
        assert!(!param_catalogue::EXCLUDED.contains(k), "DESCS describes an excluded key: {k}");
    }
    assert_eq!(param_catalogue::DESCS.len(), 114, "one row per offered key, no duplicates");
    for (k, gate, _, _) in param_catalogue::INERT_WHEN {
        assert!(params::spec(k).is_some(), "INERT_WHEN row for an unknown key {k}");
        assert_eq!(params::spec(gate).map(|s| s.kind), Some(params::Kind::Bool), "INERT_WHEN gate {gate} must be a switch");
    }
    for e in param_catalogue::EXCLUDED {
        assert!(params::spec(e).is_some(), "excluded key {e} is not in the registry");
    }
}

/// Protects the description FORMAT the direction test reads: every number row
/// says what raising and lowering do (`Raise: .. Lower: ..`), every switch what
/// on and off do (`On: .. Off: ..`), each at most 160 characters.
#[test]
fn every_description_states_both_directions() {
    for s in params::PARAMS.iter().filter(|s| !param_catalogue::EXCLUDED.contains(&s.key)) {
        let d = param_catalogue::desc(s.key).unwrap();
        assert!(d.chars().count() <= 160, "{}: {} characters", s.key, d.chars().count());
        let (a, b) = if s.kind == params::Kind::Bool { ("On: ", " Off: ") } else { ("Raise: ", " Lower: ") };
        assert!(d.starts_with(a) && d.contains(b), "{}: must read '{a}..{b}..': {d}", s.key);
    }
}

/// The clause of `key`'s description for one direction: `+1` raise/on, `-1` lower/off.
fn clause(key: &str, dir: i32) -> String {
    let d = param_catalogue::desc(key).unwrap();
    let cut = if d.starts_with("On: ") { " Off: " } else { " Lower: " };
    let (up, down) = d.split_at(d.find(cut).unwrap());
    (if dir > 0 { up } else { down }).to_lowercase()
}

/// Protects the DIRECTION hints, for every key the LLM-6c stage-1 report scored
/// and the intent it scored it for (`run_eval.py` `EXPECT`), reviewed against
/// `GENERATION_PARAMETERS.md`: the clause for the expected direction names the
/// intent's word. `climate.rain_dep` ("Dryness") is NOT scored as "drier": the
/// document says it sharpens wet/dry boundaries past the first ridge, so its
/// raise clause must say "first ridge" and must not call itself a world-wide
/// effect, while the three global moisture keys must say "drier world" when
/// lowered. MUTATION: swap the clauses of `climate.evap`, or write "drier world"
/// into `climate.rain_dep`'s raise clause, and this goes red.
#[test]
fn direction_hints_point_the_right_way() {
    let cases: &[(&str, i32, &str)] = &[
        // "drier" (and its mirror "wetter")
        ("climate.ocean", -1, "drier world"),
        ("climate.ocean", 1, "wetter world"),
        ("climate.evap", -1, "drier world"),
        ("climate.ocean_hum", -1, "drier world"),
        ("climate.w_iters", -1, "drier interiors"),
        // "lower mountains" / "big mountain ranges"
        ("peak_m", -1, "lower mountains"),
        ("peak_m", 1, "taller mountains"),
        ("tect.alpha", -1, "lower mountains"),
        ("tect.alpha", 1, "stronger mountain"),
        ("world_structure.tectonic_energy", -1, "lower relief"),
        ("world_structure.tectonic_energy", 1, "bigger mountains"),
        // "no sea"
        ("sea_level", -1, "more land"),
        // "colder"
        ("climate.equator_temp", -1, "colder world"),
        ("climate.pole_temp", -1, "colder world"),
        ("climate.lapse_rate", 1, "colder"),
        // "more volcanoes"
        ("volc.count", 1, "more volcanoes"),
        ("world_structure.hotspot_density", 1, "more volcanoes"),
        // "a chain of islands"
        ("world_structure.fragmentation", 1, "island"),
        ("world_structure.continentality", -1, "more ocean"),
        ("sea_level", 1, "more of the map is sea"),
    ];
    for (k, dir, word) in cases {
        let c = clause(k, *dir);
        assert!(c.contains(word), "{k} ({}): '{c}' should say '{word}'", if *dir > 0 { "raise" } else { "lower" });
        assert!(!clause(k, -dir).contains(word), "{k}: the opposite clause also says '{word}'");
    }
    let up = clause("climate.rain_dep", 1);
    assert!(up.contains("first ridge") && up.contains("drier"), "{up}");
    assert!(!param_catalogue::desc("climate.rain_dep").unwrap().contains("world"), "rain_dep is not a global dial");
}

/// Protects the LLM-6a schema bar over the real rows: 100% of offered keys have a
/// valid entry (type, finite min <= default <= max, label, desc) and
/// `Catalogue::problems()` is empty; each group's schema is at most 12 000 bytes
/// (scope labelled judgement) and its measured size is printed; every excluded
/// key is absent from every group; the nine groups each have a blurb.
/// MUTATION: delete a `BLURBS` row, or widen a registry `max` so a default... (the
/// widening is caught by `problems()` only when it strands a value; see the
/// sibling test that narrows one instead).
#[test]
fn the_generated_schema_is_valid_and_small() {
    let c = fresh();
    assert!(c.problems().is_empty(), "{:?}", c.problems());
    assert_eq!(c.groups.len(), 9);
    assert_eq!(c.rows.len(), 114);
    let mut total = 0;
    for g in all_groups(&c) {
        let (_, v, keys) = c.schema_for(Some(&json!([g]))).unwrap();
        let bytes = v.to_string().len();
        total += bytes;
        println!("schema group {g:<16} {:>3} keys {bytes:>6} bytes", keys.len());
        assert!(bytes <= 12_000, "{g}: {bytes} bytes");
        for e in v["settings"].as_array().unwrap() {
            let ty = e["type"].as_str().unwrap();
            assert!(["bool", "int", "float"].contains(&ty));
            assert!(!e["label"].as_str().unwrap().is_empty() && !e["desc"].as_str().unwrap().is_empty());
            assert!(e.get("current").is_some() && e.get("default").is_some(), "{e}");
            if ty != "bool" {
                let (lo, hi, d) = (e["min"].as_f64().unwrap(), e["max"].as_f64().unwrap(), e["default"].as_f64().unwrap());
                assert!(lo.is_finite() && hi.is_finite() && lo <= d && d <= hi, "{e}");
            }
        }
        for x in param_catalogue::EXCLUDED {
            assert!(!keys.iter().any(|k| k == x), "{x} offered in {g}");
        }
    }
    println!("schema total {total} bytes");
    let missing_blurb: Vec<&str> = params::groups().into_iter().filter(|g| param_catalogue::blurb(g).is_none()).collect();
    assert!(missing_blurb.is_empty(), "groups with no blurb: {missing_blurb:?}");
}

/// Protects that `problems()` sees the real failure modes on real rows: a row
/// stripped of its description, a group stripped of its blurb, and a range
/// narrowed so the current value falls outside each turn it non-empty.
#[test]
fn a_broken_catalogue_is_reported() {
    let mut c = fresh();
    c.rows[0].desc.clear();
    assert_eq!(c.problems().len(), 1);
    let mut c = fresh();
    c.groups[4].blurb.clear();
    assert_eq!(c.problems().len(), 1);
    let mut c = fresh();
    let i = c.rows.iter().position(|r| r.key == "peak_m").unwrap();
    c.rows[i].max = 3000.0;
    assert_eq!(c.problems().len(), 2, "default and current 4000 m both outside: {:?}", c.problems());
}

/// Protects the derived tags on real rows: the ruled-divergence switches are
/// tagged advanced (thirteen-plus by `params.rs`'s own count; asserted against an
/// independent recount of bool rows whose two defaults differ), no number row is;
/// the cost classes follow `params::invalidates`, with literals for one key of
/// each; INERT_WHEN fires on a fresh world for the keys continental steering
/// gates (it is off by default) and not for those it would need on.
#[test]
fn derived_tags_follow_the_registry() {
    let c = fresh();
    let app = params::defaults();
    let parity = cartalith_engine::WorldParams::defaults(0, 0, 0);
    let recount = params::PARAMS
        .iter()
        .filter(|s| !param_catalogue::EXCLUDED.contains(&s.key))
        .filter(|s| s.kind == params::Kind::Bool && params::get(&app, s.key) != params::get(&parity, s.key))
        .count();
    let tagged = c.rows.iter().filter(|r| r.advanced).count();
    assert_eq!(tagged, recount);
    assert!(tagged >= 13, "{tagged}");
    assert!(c.rows.iter().filter(|r| r.advanced).all(|r| r.kind == Kind::Bool));
    assert!(c.row("tect.orogen_relief").unwrap().advanced && !c.row("tect.ridged").unwrap().advanced);
    assert_eq!(c.row("climate.evap").unwrap().regen, Regen::RefreshClimate);
    assert_eq!(c.row("peak_m").unwrap().regen, Regen::RefreshClimate);
    assert_eq!(c.row("civ.factions").unwrap().regen, Regen::RefreshSettlements);
    assert_eq!(c.row("tect.alpha").unwrap().regen, Regen::FullGenerate);
    assert_eq!(c.row("civ.garrison_exposure_scale").unwrap().regen, Regen::Live);
    assert!(c.row("world_structure.fragmentation").unwrap().inert.is_some(), "steering off by default");
    assert!(c.row("tect.plates").unwrap().inert.is_none());
    assert!(c.row("crater.count").unwrap().inert.is_some(), "the physical crater model ships on");
    assert!(c.row("geo.age").unwrap().inert.is_some());
    // An absent current is omitted and does not evaluate the gate.
    let none = param_catalogue::build(&|_| None);
    assert!(none.rows.iter().all(|r| r.current.is_none() && r.inert.is_none()));
}

/// Protects the LLM-6b fuzz bar over the REAL rows: at least 500 hostile
/// proposals (one family per rejecting row of the s.5b.2 table), opened on three
/// groups, 100% apply nothing; a generated valid set of 300 is 100% accepted with
/// no rejection, so a validator that refuses everything fails here.
#[test]
fn fuzz_over_the_real_rows() {
    let c = fresh();
    for (shown, seed) in [
        (vec!["weather".to_owned(), "world".to_owned(), "tectonics".to_owned()], 11u64),
        (vec!["erosion".to_owned(), "civ".to_owned(), "volcanism".to_owned()], 12),
        (vec!["climate".to_owned(), "planet".to_owned(), "world_structure".to_owned()], 13),
    ] {
        let bad = hostile_corpus(&c, &shown, 40, seed);
        assert!(bad.len() >= 500, "{}", bad.len());
        for (family, case) in &bad {
            let v = c.validate_text(case, &shown);
            assert!(applies_nothing(&v), "{family}: {case} -> {v:?}");
        }
        for case in valid_corpus(&c, &shown, 300, seed + 100) {
            let v = c.validate_text(&case, &shown);
            assert!(v.structural.is_none() && v.rejected.is_empty() && !v.valid.is_empty(), "{case} -> {v:?}");
        }
    }
}

/// Protects the worked example on the REAL rows (scope s.5b.3): the five changes
/// with the registry's current values 1.0, 0.12, 0.35, 4000, 0.85; and its
/// failing variant's five verdicts, including `use_gpu` as excluded (V6) and
/// `climate.rain_k` 2.7 over its real max 2.0 (V10).
#[test]
fn worked_example_on_real_rows() {
    let c = fresh();
    let shown = vec!["weather".to_owned(), "world".to_owned(), "tectonics".to_owned()];
    let worked = r#"{"summary":"Less ocean moisture and a lower relief ceiling.","changes":[
      {"key":"climate.ocean","value":0.7,"reason":"Cut the moisture budget by 30 percent."},
      {"key":"climate.evap","value":0.09,"reason":"Less pickup over the sea."},
      {"key":"climate.rain_dep","value":0.5,"reason":"Rain out sooner so interiors dry."},
      {"key":"peak_m","value":2800,"reason":"Lower the vertical scale by 30 percent."},
      {"key":"tect.alpha","value":0.65,"reason":"Weight the tectonic signal less."}]}"#;
    let v = c.validate_text(worked, &shown);
    let got: Vec<(&str, Option<PValue>)> = v.valid.iter().map(|x| (x.key.as_str(), x.old)).collect();
    let n = |x| Some(PValue::Num(x));
    assert_eq!(
        got,
        vec![("climate.ocean", n(1.0)), ("climate.evap", n(0.12)), ("climate.rain_dep", n(0.35)), ("peak_m", n(4000.0)), ("tect.alpha", n(0.85))]
    );
    assert!(v.rejected.is_empty());
    let bad = r#"{"changes":[{"key":"climate.aridity","value":0.8},{"key":"climate.rain_k","value":2.7},
        {"key":"tect.plates","value":"many"},{"key":"use_gpu","value":false},{"key":"peak_m","value":3000},{"key":"peak_m","value":2000}]}"#;
    let v = c.validate_text(bad, &shown);
    let rules: Vec<Rule> = v.rejected.iter().map(|r| r.rule).collect();
    assert_eq!(rules, vec![Rule::V5, Rule::V10, Rule::V8, Rule::V6, Rule::V7, Rule::V7]);
    assert!(v.valid.is_empty());
    println!("V5 hint for climate.aridity: {}", v.rejected[0].why);
}

/// Protects the two-step conversation over the real catalogue: three groups
/// open 34 keys (weather 12 + world 6 + tectonics 16, scope s.5b.3 step 3), the
/// second request's key enum is exactly those, and a schema call for a group that
/// does not exist opens nothing.
#[test]
fn two_step_flow_over_real_rows() {
    let c = fresh();
    let mut conv = ParamsConversation::new(&c, "drier climate, lower mountains").unwrap();
    let s = conv.on_reply(&c, "", vec![IncomingCall { id: None, name: "get_param_schema".into(), arguments: json!({"groups": ["weather", "world", "tectonics"]}) }]);
    assert_eq!(s, Step::Continue);
    let tools = conv.tools(&c);
    let en: &Value = &tools[1].parameters["properties"]["changes"]["items"]["properties"]["key"]["enum"];
    assert_eq!(en.as_array().unwrap().len(), 34);
    let mut bad = ParamsConversation::new(&c, "x").unwrap();
    bad.on_reply(&c, "", vec![IncomingCall { id: None, name: "get_param_schema".into(), arguments: json!({"groups": ["rainfall"]}) }]);
    assert!(bad.shown_groups().is_empty() && bad.tools(&c).len() == 1);
}
