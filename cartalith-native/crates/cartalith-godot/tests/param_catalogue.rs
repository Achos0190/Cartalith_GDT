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

/// Replays Claude's "more volcanoes" run of 2026-10-26 (REPORT.md, Defects 1)
/// over the REAL rows: schema call 1 for `world`, schema call 2 for `volcanism`,
/// then `volc.count`. Before the union fix call 2 replaced call 1's groups and
/// the model still had `volc.count` (so this one passes under replace too); the
/// DISTINGUISHING half is the reverse order: `volcanism` first, then `world`,
/// then `volc.count` -- which the old replace refused V6. Also: a `climate` key
/// never opened is still V6. MUTATION: `shown_groups = groups` in params_flow.rs
/// turns the second proposal red.
#[test]
fn claude_more_volcanoes_replay_accepts_the_earlier_groups_key() {
    let c = fresh();
    let call = |g: &str| vec![IncomingCall { id: None, name: "get_param_schema".into(), arguments: json!({"groups": [g]}) }];
    let propose = |key: &str, v: f64| vec![IncomingCall { id: None, name: "set_generation_params".into(), arguments: json!({"changes": [{"key": key, "value": v}]}) }];
    for order in [["world", "volcanism"], ["volcanism", "world"]] {
        let mut conv = ParamsConversation::new(&c, "more volcanoes").unwrap();
        assert_eq!(conv.on_reply(&c, "", call(order[0])), Step::Continue);
        assert_eq!(conv.on_reply(&c, "", call(order[1])), Step::Continue);
        assert_eq!(conv.shown_groups().len(), 2, "{order:?}");
        let st = conv.on_reply(&c, "", propose("volc.count", 30.0));
        let Step::Proposal(v) = st else { panic!("{order:?}: {st:?}") };
        assert_eq!(v.valid.len(), 1, "{order:?}: {v:?}");
    }
    let mut conv = ParamsConversation::new(&c, "more volcanoes").unwrap();
    conv.on_reply(&c, "", call("world"));
    conv.on_reply(&c, "", call("volcanism"));
    let Step::Retry(v) = conv.on_reply(&c, "", propose("climate.ocean", 0.7)) else { panic!() };
    assert_eq!(v.rejected[0].rule, Rule::V6);
}

// ---------------------------------------------------------------------------
// LLM-8: the read-only query tools over the REAL registry rows
// (`LLM_ASSISTANT_SCOPE.md` s.5b.5; bar: `get_parameters` returns, for 100% of the
// offered keys, a value equal to the engine's own getter).
// ---------------------------------------------------------------------------

use cartalith_llm::query::{
    GET_CHANGED_TOOL, GET_PARAMETERS_TOOL, MAX_CHANGED_LISTED, MAX_RESULT_BYTES, NoWorld, QueryBudget, QueryCall, answer, parse,
};

/// The catalogue as a world whose settings are `p` sees it.
fn catalogue_of(p: &cartalith_engine::WorldParams) -> Catalogue {
    param_catalogue::build(&|k| {
        params::get(p, k).map(|v| match v {
            params::Value::Bool(b) => PValue::Bool(b),
            params::Value::Num(n) => PValue::Num(n),
        })
    })
}

/// A world's settings with a few keys moved off their defaults through the registry's own setter
/// (the one `WorldGen::set_params` uses), so "changed" has something to find. Six keys: under the
/// 40-key list cap, in four groups, covering floats, an int and a bool.
fn moved() -> (cartalith_engine::WorldParams, Vec<&'static str>) {
    let mut p = params::defaults();
    let keys = ["sea_level", "peak_m", "climate.evap", "tect.plates", "volc.count", "climate.wind_manual"];
    for k in keys {
        let spec = params::spec(k).unwrap();
        let v = match params::get(&p, k).unwrap() {
            params::Value::Bool(b) => params::Value::Bool(!b),
            params::Value::Num(n) => {
                let mid = (spec.min + spec.max) / 2.0;
                let to = if (n - mid).abs() > 1e-9 { mid } else { spec.min };
                params::Value::Num(if spec.kind == params::Kind::Int { to.round() } else { to })
            }
        };
        assert!(matches!(params::set(&mut p, k, v), params::Outcome::Applied), "{k} must store exactly");
        assert_ne!(params::get(&p, k), params::get(&params::defaults(), k), "{k} must differ from its default");
    }
    (p, keys.to_vec())
}

fn json_of(v: params::Value, kind: params::Kind) -> Value {
    match (v, kind) {
        (params::Value::Bool(b), _) => json!(b),
        (params::Value::Num(n), params::Kind::Int) => json!(n as i64),
        (params::Value::Num(n), _) => json!(n),
    }
}

/// Protects the LLM-8 bar "`get_parameters` returns, for 100% of the registry's offered keys, a value
/// equal to `get_params()[key]`" over the real registry: for each of the 114 offered keys, across the
/// nine group queries, `current` equals the setter-side value (read back through `params::get`, the
/// function `get_params()` is built from) and `default` equals `params::defaults()`'s, with
/// `changed` agreeing with the difference. Counts 114 so a skipped group fails. MUTATION: make
/// `setting_line` write `default` into `current`, or drop a row from a group's answer: red.
#[test]
fn get_parameters_equals_the_registry_for_every_offered_key() {
    let (p, moved_keys) = moved();
    let cat = catalogue_of(&p);
    let defaults = params::defaults();
    let mut seen = 0;
    for g in cat.group_names() {
        let call = parse(GET_PARAMETERS_TOOL, &json!({"group": g}), &cat).unwrap();
        let v = answer(&call, &cat, &NoWorld("n/a".into()), &mut QueryBudget::default());
        let rows = v["settings"].as_array().unwrap_or_else(|| panic!("group {g}: {v}"));
        for r in rows {
            let key = r["key"].as_str().unwrap();
            let spec = params::spec(key).unwrap();
            assert_eq!(r["current"], json_of(params::get(&p, key).unwrap(), spec.kind), "{key} current");
            assert_eq!(r["default"], json_of(params::get(&defaults, key).unwrap(), spec.kind), "{key} default");
            assert_eq!(r["changed"], json!(moved_keys.contains(&key)), "{key} changed");
            seen += 1;
        }
    }
    assert_eq!(seen, 114, "every offered key is answered exactly once across the nine groups");
}

/// Protects `get_changed_parameters` against an independent diff of the two registry snapshots, and
/// that it reports no excluded key even if one was moved (a machine setting is not the assistant's
/// to see). MUTATION: compare with `<` instead of `!=`, or list `use_gpu`: red.
#[test]
fn changed_parameters_equal_an_independent_diff() {
    let (mut p, moved_keys) = moved();
    // An excluded key moved too: the machine setting must not appear in the answer.
    let Some(params::Value::Bool(now)) = params::get(&p, "use_gpu") else { panic!("use_gpu is a switch") };
    let _ = params::set(&mut p, "use_gpu", params::Value::Bool(!now));
    assert_ne!(params::get(&p, "use_gpu"), params::get(&params::defaults(), "use_gpu"), "the excluded key really moved");
    let cat = catalogue_of(&p);
    let defaults = params::defaults();
    let mut want: Vec<&str> = params::PARAMS
        .iter()
        .map(|s| s.key)
        .filter(|k| !param_catalogue::EXCLUDED.contains(k))
        .filter(|k| params::get(&p, k) != params::get(&defaults, k))
        .collect();
    want.sort();
    let v = answer(&parse(GET_CHANGED_TOOL, &json!({}), &cat).unwrap(), &cat, &NoWorld("n/a".into()), &mut QueryBudget::default());
    let mut got: Vec<&str> = v["settings"].as_array().unwrap().iter().map(|r| r["key"].as_str().unwrap()).collect();
    got.sort();
    assert_eq!(got, want);
    assert_eq!(got.len(), moved_keys.len(), "the six moved keys and nothing else: {got:?}");
    assert_eq!(v["changed_count"], json!(want.len()));
    assert!(!got.contains(&"use_gpu"));
    assert!(want.len() < MAX_CHANGED_LISTED);
}

/// Protects `describe_parameter` over all 114 offered keys: the group, the description, the "had a
/// control in the original" flag and the "what a change needs" word each equal an independent read
/// of the registry (`ParamSpec::group`, `param_catalogue::desc`, `reference_control`,
/// `params::invalidates`), and the six excluded keys are refused in words. MUTATION: swap the
/// climate and settlements words, or invert `from_reference`: red.
#[test]
fn describe_parameter_equals_the_registry_for_every_offered_key() {
    use cartalith_engine::staleness::PipelineStage;
    let cat = fresh();
    let mut n = 0;
    for s in params::PARAMS.iter().filter(|s| !param_catalogue::EXCLUDED.contains(&s.key)) {
        let v = answer(&QueryCall::Describe { key: s.key.to_owned() }, &cat, &NoWorld("n/a".into()), &mut QueryBudget::default());
        assert_eq!(v["group"], s.group, "{}", s.key);
        assert_eq!(v["desc"], param_catalogue::desc(s.key).unwrap(), "{}", s.key);
        assert_eq!(v["in_original_cartalith"], json!(!s.reference_control.is_empty()), "{}", s.key);
        let needs = match params::invalidates(s.key) {
            Some(PipelineStage::Hydrology) => "climate_refresh",
            Some(PipelineStage::Climate) => "settlements_refresh",
            Some(_) => "full_generate",
            None if s.key == "civ.garrison_exposure_scale" => "live",
            None => "full_generate",
        };
        assert_eq!(v["needs"], needs, "{}", s.key);
        n += 1;
    }
    assert_eq!(n, 114);
    for e in param_catalogue::EXCLUDED {
        let v = answer(&QueryCall::Describe { key: (*e).to_owned() }, &cat, &NoWorld("n/a".into()), &mut QueryBudget::default());
        assert!(v["error"].as_str().unwrap().contains("not offered"), "{e}: {v}");
        assert!(v.get("desc").is_none());
    }
}

/// Protects the size bar: every group's `get_parameters` answer is under the result cap, the sizes
/// are printed (`--nocapture`; the numbers go into STATUS.md), and "every setting at once" either
/// fits or is REFUSED with advice (never cut).
#[test]
fn query_answers_fit_the_result_cap() {
    let cat = fresh();
    let mut biggest = (String::new(), 0usize);
    let mut total = 0usize;
    for g in all_groups(&cat) {
        let v = answer(&QueryCall::Parameters { group: Some(g.clone()), keys: None }, &cat, &NoWorld("n/a".into()), &mut QueryBudget::default());
        let n = v.to_string().len();
        println!("get_parameters group {g}: {n} bytes");
        assert!(v.get("error").is_none() && n <= MAX_RESULT_BYTES, "{g}: {n}");
        total += n;
        if n > biggest.1 {
            biggest = (g, n);
        }
    }
    let all = answer(&QueryCall::Parameters { group: None, keys: None }, &cat, &NoWorld("n/a".into()), &mut QueryBudget::default());
    println!("get_parameters (no arguments, the overview): {} bytes; biggest group {} {} bytes; all nine groups together would be {} bytes", all.to_string().len(), biggest.0, biggest.1, total);
    assert!(total > MAX_RESULT_BYTES, "all 114 at once would not fit, which is why no-argument is an overview: {total}");
    let groups = all["groups"].as_array().unwrap();
    assert_eq!(groups.len(), 9);
    assert_eq!(groups.iter().map(|g| g["settings"].as_u64().unwrap()).sum::<u64>(), 114);
    assert!(all.get("settings").is_none(), "the overview carries no values");
    let ch = answer(&QueryCall::Changed, &cat, &NoWorld("n/a".into()), &mut QueryBudget::default());
    assert_eq!(ch["changed_count"], 0, "a fresh world has nothing changed");
}

/// Protects the tool-result vocabulary against a world's free text: no catalogue field a query can
/// return is a place for a user's own words (labels and descriptions are app constants), so every
/// string in a full dump is from the registry or one of the fixed phrases. MUTATION: add a
/// world-name field to an answer and this reds.
#[test]
fn query_results_contain_only_registry_vocabulary() {
    let cat = fresh();
    let mut allowed: std::collections::HashSet<String> = std::collections::HashSet::new();
    for r in &cat.rows {
        for s in [&r.key, &r.group, &r.label, &r.unit, &r.desc] {
            allowed.insert(s.clone());
        }
    }
    for w in ["float", "int", "bool", "live", "climate_refresh", "settlements_refresh", "full_generate"] {
        allowed.insert(w.into());
    }
    fn strings(v: &Value, out: &mut Vec<String>) {
        match v {
            Value::String(s) => out.push(s.clone()),
            Value::Array(a) => a.iter().for_each(|x| strings(x, out)),
            Value::Object(m) => m.values().for_each(|x| strings(x, out)),
            _ => {}
        }
    }
    let mut got = Vec::new();
    for g in all_groups(&cat) {
        strings(&answer(&QueryCall::Parameters { group: Some(g), keys: None }, &cat, &NoWorld("n/a".into()), &mut QueryBudget::default()), &mut got);
    }
    for r in &cat.rows {
        strings(&answer(&QueryCall::Describe { key: r.key.clone() }, &cat, &NoWorld("n/a".into()), &mut QueryBudget::default()), &mut got);
    }
    let fixed = ["read fresh", "the climate refresh", "the settlements refresh", "only a full Generate", "ruled behaviour", "Will have no effect"];
    let foreign: Vec<&String> = got.iter().filter(|s| !allowed.contains(*s) && !fixed.iter().any(|f| s.starts_with(f))).collect();
    assert!(foreign.is_empty(), "strings outside the registry vocabulary: {foreign:?}");
}
