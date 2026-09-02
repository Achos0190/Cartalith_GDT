//! Unit tests for the generation-parameter table (`src/params.rs`) — the
//! Dictionary-to-`WorldParams` mapping layer `WorldGen::set_params` sits on.
//!
//! Pulled in by `#[path]` rather than through the crate, matching
//! `golden_parity_render.rs`'s own pattern: `cartalith-godot` is a
//! `cdylib`, and `params.rs` is deliberately `godot`-free precisely so this
//! can run under a plain `cargo test` with no engine process.
//!
//! `dead_code`: this target compiles `params.rs` alone, so the `ParamSpec`
//! fields only `lib.rs`'s Godot layer reads look unused from in here.
#![allow(dead_code)]

#[path = "../src/params.rs"]
mod params;

use params::{Kind, Outcome, Value};

/// The headline invariant this whole API is constrained by: a `WorldGen`
/// nobody sets anything on must generate exactly what it generated before
/// the parameter API existed. Concretely — the table's starting point *is*
/// `WorldParams::defaults`, field for field, and reading every key back
/// reproduces it.
#[test]
fn defaults_round_trip_through_every_key() {
    let d = params::defaults();
    let mut p = params::defaults();
    for s in params::PARAMS {
        let before = params::get(&d, s.key).unwrap_or_else(|| panic!("no getter for {}", s.key));
        assert_eq!(params::set(&mut p, s.key, before), Outcome::Applied, "writing its own default clamped: {}", s.key);
        let after = params::get(&p, s.key).unwrap_or_else(|| panic!("no getter for {}", s.key));
        assert_eq!(before, after, "get/set disagree for {}", s.key);
    }
    assert_eq!(p, d, "writing every default back must leave WorldParams untouched");
}

/// A default outside its own advertised range would make a "reset to
/// default" round trip through `set_params` silently clamp — the exact
/// failure this reports rather than hides.
#[test]
fn every_default_lies_inside_its_own_range() {
    let d = params::defaults();
    for s in params::PARAMS {
        assert!(s.min <= s.max, "{}: min > max", s.key);
        if let Some(Value::Num(n)) = params::get(&d, s.key) {
            assert!(n >= s.min && n <= s.max, "{} default {n} outside [{}, {}]", s.key, s.min, s.max);
            if s.kind == Kind::Int {
                assert_eq!(n.fract(), 0.0, "{} is an int parameter with a fractional default {n}", s.key);
            }
        }
    }
}

#[test]
fn keys_are_unique_and_grouped() {
    let mut keys: Vec<&str> = params::PARAMS.iter().map(|s| s.key).collect();
    let n = keys.len();
    keys.sort_unstable();
    keys.dedup();
    assert_eq!(keys.len(), n, "duplicate key in PARAMS");
    // Groups are contiguous: the table order is the dialog order, so a group
    // appearing twice would split a section in the generated UI.
    let groups = params::groups();
    let mut seen: Vec<&str> = Vec::new();
    let mut last = "";
    for s in params::PARAMS {
        if s.group != last {
            assert!(!seen.contains(&s.group), "group {} is not contiguous in PARAMS", s.group);
            seen.push(s.group);
            last = s.group;
        }
    }
    assert_eq!(seen, groups);
}

#[test]
fn unknown_key_is_rejected_and_writes_nothing() {
    let mut p = params::defaults();
    assert_eq!(params::set(&mut p, "tect.platez", Value::Num(30.0)), Outcome::Rejected);
    assert_eq!(params::set(&mut p, "", Value::Num(1.0)), Outcome::Rejected);
    assert_eq!(params::set(&mut p, "climate", Value::Num(1.0)), Outcome::Rejected);
    assert_eq!(p, params::defaults());
}

#[test]
fn wrong_type_is_rejected_both_directions() {
    let mut p = params::defaults();
    // A number for a checkbox parameter: no truthiness coercion.
    assert_eq!(params::set(&mut p, "tect.ridged", Value::Num(0.0)), Outcome::Rejected);
    assert!(p.tect.ridged, "a rejected write must not have landed");
    // A boolean for a numeric parameter.
    assert_eq!(params::set(&mut p, "tect.plates", Value::Bool(true)), Outcome::Rejected);
    assert_eq!(p.tect.plates, 14);
    assert_eq!(params::set(&mut p, "sea_level", Value::Bool(false)), Outcome::Rejected);
    assert_eq!(p.sea_level, 0.42);
    assert_eq!(p, params::defaults());
}

#[test]
fn non_finite_is_rejected_never_clamped() {
    let mut p = params::defaults();
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(params::set(&mut p, "sea_level", Value::Num(bad)), Outcome::Rejected, "{bad} must be rejected");
        assert_eq!(params::set(&mut p, "tect.plates", Value::Num(bad)), Outcome::Rejected, "{bad} must be rejected");
    }
    assert_eq!(p, params::defaults(), "a non-finite write must leave the field alone");
    assert!(p.sea_level.is_finite());
}

#[test]
fn out_of_range_clamps_and_reports() {
    let mut p = params::defaults();
    assert_eq!(params::set(&mut p, "sea_level", Value::Num(4.0)), Outcome::Clamped);
    assert_eq!(p.sea_level, 1.0);
    assert_eq!(params::set(&mut p, "sea_level", Value::Num(-2.0)), Outcome::Clamped);
    assert_eq!(p.sea_level, 0.0);
    assert_eq!(params::set(&mut p, "tect.plates", Value::Num(-5.0)), Outcome::Clamped);
    assert_eq!(p.tect.plates, 4, "a negative plate count must never reach build_plates");
    assert_eq!(params::set(&mut p, "tect.plates", Value::Num(1000.0)), Outcome::Clamped);
    assert_eq!(p.tect.plates, 40);
    // In range: applied verbatim, no clamp report.
    assert_eq!(params::set(&mut p, "tect.plates", Value::Num(22.0)), Outcome::Applied);
    assert_eq!(p.tect.plates, 22);
}

#[test]
fn int_parameters_round_rather_than_reject() {
    let mut p = params::defaults();
    // A GDScript slider with a float step happily produces 13.999999 for 14.
    assert_eq!(params::set(&mut p, "tect.plates", Value::Num(13.999_999)), Outcome::Clamped);
    assert_eq!(p.tect.plates, 14);
    assert_eq!(params::set(&mut p, "climate.w_iters", Value::Num(70.4)), Outcome::Clamped);
    assert_eq!(p.climate.w_iters, 70);
    assert_eq!(params::set(&mut p, "crater.count", Value::Num(150.0)), Outcome::Applied);
    assert_eq!(p.crater.count, 150);
}

/// A partial update is the normal case: a dialog sends only what its user
/// touched, and every other parameter must survive untouched.
#[test]
fn partial_update_touches_only_the_named_keys() {
    let mut p = params::defaults();
    assert_eq!(params::set(&mut p, "planet.g", Value::Num(1.62)), Outcome::Applied);
    assert_eq!(params::set(&mut p, "climate.currents", Value::Bool(false)), Outcome::Applied);

    let mut expected = params::defaults();
    expected.planet.g = 1.62;
    expected.climate.currents = false;
    assert_eq!(p, expected);
}

/// Each of the eight `cartalith-engine` parameter structs is genuinely
/// reachable — the gap this API exists to close, asserted rather than
/// assumed. Before it, only `sea_level`, four flags and the archetype
/// presets were.
#[test]
fn every_engine_param_struct_is_reachable() {
    let mut p = params::defaults();
    let writes: [(&str, Value); 8] = [
        ("sea_level", Value::Num(0.5)),                          // WorldParams top level
        ("tect.warp", Value::Num(0.7)),                          // TectonicParams
        ("volc.count", Value::Num(33.0)),                        // VolcanismParams
        ("crater.age", Value::Num(0.9)),                         // CraterParams
        ("planet.axial_tilt_deg", Value::Num(41.0)),             // PlanetParams
        ("climate.equator_temp", Value::Num(38.0)),              // ClimateInputParams
        ("stream.iters", Value::Num(30.0)),                      // StreamParams
        ("world_structure.tectonic_energy", Value::Num(0.85)),   // WorldStructureParams
    ];
    for (k, v) in writes {
        assert_eq!(params::set(&mut p, k, v), Outcome::Applied, "{k}");
    }
    assert_eq!(p.sea_level, 0.5);
    assert_eq!(p.tect.warp, 0.7);
    assert_eq!(p.volc.count, 33);
    assert_eq!(p.crater.age, 0.9);
    assert_eq!(p.planet.axial_tilt_deg, 41.0);
    assert_eq!(p.climate.equator_temp, 38.0);
    assert_eq!(p.stream.iters, 30);
    assert_eq!(p.world_structure.tectonic_energy, 0.85);
}

/// `use_gpu` and the five raw World-Structure knobs are the two items
/// `GUI_FEATURE_PARITY_SCOPE.md` Category 1 names as real-but-unreachable.
#[test]
fn parity_audit_category_one_items_are_reachable() {
    let mut p = params::defaults();
    assert!(!p.use_gpu);
    assert_eq!(params::set(&mut p, "use_gpu", Value::Bool(true)), Outcome::Applied);
    assert!(p.use_gpu);

    for k in [
        "world_structure.enabled",
        "world_structure.continentality",
        "world_structure.fragmentation",
        "world_structure.tectonic_energy",
        "world_structure.ocean_depth",
        "world_structure.hotspot_density",
        "planet.g",
        "planet.rotation_hours",
        "planet.axial_tilt_deg",
    ] {
        assert!(params::spec(k).is_some(), "{k} must be an exposed parameter");
    }
}

// ---------------------------------------------------------------------------
// The save round trip (`SAVEFILE_COMPAT.md`, FI-01)
// ---------------------------------------------------------------------------

/// The drift guard `JS_PATHS` needs, since it is a second table rather than
/// a column on `ParamSpec`: a parameter added to `PARAMS` without a decision
/// recorded here would silently stop being written to the reference half of
/// a save.
#[test]
fn every_param_has_a_reference_path_decision() {
    let state = params::save_state(&params::defaults());
    let native = state[params::NATIVE_PARAMS_KEY].as_object().expect("the native block must be an object");
    for s in params::PARAMS {
        assert!(native.contains_key(s.key), "{} is missing from the save's native block", s.key);
        assert!(
            params::reference_path(s.key).is_some(),
            "{} has no JS_PATHS row -- add its reference `state` path, or \"\" if the reference has none",
            s.key
        );
    }
    assert_eq!(native.len(), params::PARAMS.len(), "the native block must be exactly the table");
}

/// The actual bar for a save file: a non-default world's parameters must
/// come back byte-identical, not approximately. Every row is moved off its
/// default first — a table where half the rows never changed would pass a
/// round-trip test that a broken writer also passes.
#[test]
fn a_fully_non_default_table_round_trips_exactly() {
    let mut original = params::defaults();
    for s in params::PARAMS {
        let moved = match params::get(&original, s.key).unwrap() {
            Value::Bool(b) => Value::Bool(!b),
            // The far end of the range from wherever the default sits, so
            // every row genuinely moves.
            Value::Num(n) => Value::Num(if (n - s.min).abs() > (s.max - n).abs() { s.min } else { s.max }),
        };
        assert_ne!(params::set(&mut original, s.key, moved), Outcome::Rejected, "{} rejected its own bound", s.key);
    }
    assert_ne!(original, params::defaults(), "the fixture must actually differ from the defaults");

    let state = params::save_state(&original);
    let mut restored = params::defaults();
    let applied = params::apply_saved_state(&mut restored, &state);

    assert_eq!(applied, params::PARAMS.len(), "every parameter should have been applied");
    assert_eq!(restored, original, "a saved world's parameters must restore exactly");
}

/// The reference half is what makes the file reopenable in the HTML app.
/// Spot-checks the three shapes that differ from a straight name copy: a
/// renamed key, the int/float distinction, and the one type-changing
/// mapping (`wind_manual` -> `climate.windMode`).
#[test]
fn the_reference_half_uses_the_reference_vocabulary() {
    let mut p = params::defaults();
    params::set(&mut p, "tect.blur_r", Value::Num(21.0));
    params::set(&mut p, "tect.plates", Value::Num(9.0));
    params::set(&mut p, "climate.lat_n", Value::Num(61.0));
    params::set(&mut p, "climate.wind_manual", Value::Bool(true));
    let state = params::save_state(&p);

    assert_eq!(state["tect"]["blurR"], 21.0);
    assert!(state["tect"].get("blur_r").is_none(), "the port's own key must not leak into the reference half");
    // An int parameter is a JSON integer, not `9.0` -- the reference's own
    // sliders and labels read these directly.
    assert_eq!(state["tect"]["plates"], 9);
    assert!(state["tect"]["plates"].is_i64());
    assert_eq!(state["climate"]["latN"], 61.0);
    assert_eq!(state["climate"]["windMode"], "manual");

    params::set(&mut p, "climate.wind_manual", Value::Bool(false));
    assert_eq!(params::save_state(&p)["climate"]["windMode"], "auto");
}

/// `loadZip()` merges `state` shallowly, so a nested block written here
/// replaces the reference's own default block outright. `tect` is the one
/// block that is both mandatory (the seed lives in it) and unshimmed, so it
/// must be written whole.
#[test]
fn the_tect_block_is_complete_enough_for_a_shallow_merge() {
    let state = params::save_state(&params::defaults());
    let tect = state["tect"].as_object().expect("tect must be written");
    // Every key of the reference's own `tect` literal (reference HTML line
    // 2264) that `loadZip()` does NOT backfill. The four it does backfill
    // -- tectonicGraph, foldIntensity, trenchDepth, faultBlock -- are this
    // port's known gap and are deliberately absent.
    for key in [
        "plates", "vel", "warp", "blurR", "alpha", "beta", "age", "ridged", "lloyd", "flexure", "hetero", "resist",
        "dynamicLithology",
    ] {
        assert!(tect.contains_key(key), "tect.{key} must be written or the reference app loses its default");
    }
}

/// `state.erosion` is the documented omission (see `params.rs`'s own note):
/// unshimmed by `loadZip()` and only 2/16 keys modelled here, so writing it
/// partially would be worse than not writing it.
#[test]
fn the_unshimmed_erosion_block_is_not_written_partially() {
    let state = params::save_state(&params::defaults());
    assert!(state.get("erosion").is_none(), "a partial erosion block would replace the reference's whole one");
    // ...but the two values this port does own still round-trip, via the
    // native half.
    let native = &state[params::NATIVE_PARAMS_KEY];
    assert!(native.get("passes.diffuse_d").is_some());
    assert!(native.get("passes.diffuse_passes").is_some());
}

/// A save from a future version, a hand-edited one, or a genuine HTML-app
/// export must never panic or half-apply — the three shapes that can arrive.
#[test]
fn a_state_this_port_does_not_recognise_is_survivable() {
    let mut p = params::defaults();
    // A real HTML export: no native block at all.
    assert_eq!(params::apply_saved_state(&mut p, &serde_json::json!({"tect": {"seed": 1, "plates": 30}})), 0);
    assert_eq!(p, params::defaults(), "a reference export must leave the table at its defaults");

    // Unknown keys, wrong types, and out-of-range values side by side.
    let applied = params::apply_saved_state(
        &mut p,
        &serde_json::json!({params::NATIVE_PARAMS_KEY: {
            "tect.plates": 999.0,          // clamped, and counted
            "tect.ridged": "yes",          // wrong type -- skipped
            "not.a.parameter": 3,          // unknown -- skipped
            "climate.lat_n": 12.0,
        }}),
    );
    assert_eq!(applied, 2);
    assert_eq!(p.tect.plates, 40, "an out-of-range value is clamped, as it is from the GUI");
    assert_eq!(p.climate.lat_n, 12.0);
    assert!(p.tect.ridged, "a wrong-typed entry must leave the default alone");
}

// ===========================================================================
// SG-03: the per-parameter -> stage invalidation table
// ===========================================================================

use cartalith_engine::staleness::PipelineStage;
use cartalith_engine::{WorldParams, WorldState};

/// A world small enough to run `refresh_climate` seventy-odd times, with
/// every *gating* toggle in a state that lets the knob behind it register.
///
/// Deliberately not `WorldParams::defaults`: `wind_manual` is off by default,
/// which would make `wind_dir_deg` provably inert and the table's honest
/// answer for it "not live" — true of the default world, false of the
/// parameter. The latitude band is widened for the same reason, so there is
/// ice for `albedo_k` to act on.
fn tunable_baseline() -> (WorldParams, WorldState) {
    let mut p = WorldParams::defaults(48, 32, 4242);
    p.climate.wind_manual = true;
    p.climate.wind_dir_deg = 90.0;
    p.climate.lat_n = 75.0;
    let ws = cartalith_engine::generate_terrain(&p);
    (p, ws)
}

/// `refresh_climate` over a fixed height field — exactly the call
/// `recompute_stale` makes, including the two arguments it does *not* take
/// from `p` (`WorldState::sea_level`, and the height field itself).
fn refreshed(p: &WorldParams, ws: &WorldState) -> (Vec<f32>, Vec<f32>, Vec<f32>) {
    let (mut t, mut r, mut q) = (
        ws.temperature.clone(),
        ws.rainfall.clone(),
        ws.flow_discharge.clone(),
    );
    cartalith_engine::refresh_climate(
        p,
        ws.sea_level,
        &ws.field,
        &cartalith_engine::climate_params_for(p, ws.sea_level),
        &cartalith_engine::weather_params_for(p, ws.sea_level),
        &mut t,
        &mut r,
        &mut q,
    );
    (t, r, q)
}

/// The drift guard `params::invalidates`' own doc comment promises: the
/// Hydrology half of the table is **derived**, not asserted.
///
/// For every row, move the value to the far end of its own range and re-run
/// `refresh_climate`. If the output moves, the parameter has a live-apply
/// path and must be marked; if it does not, marking it would promise a
/// recompute that applies nothing. The two must agree for all 81 rows, so a
/// new parameter cannot be added without deciding this — and a wrong decision
/// fails here rather than in the shell.
///
/// `p.world` is restored after the write for the same reason
/// `WorldGen::recompute_params` pins it: a recompute reads the world's own
/// geometry, not the dial.
#[test]
fn every_key_that_moves_refresh_climate_is_marked_and_no_other() {
    let (base, ws) = tunable_baseline();
    let reference = refreshed(&base, &ws);
    assert!(
        reference.1.iter().any(|&r| r > 0.0),
        "a baseline with no rainfall at all would make every key look inert"
    );
    for s in params::PARAMS {
        let mut p = base.clone();
        let far = match params::get(&base, s.key).expect("every row has a getter") {
            Value::Bool(b) => Value::Bool(!b),
            Value::Num(n) => Value::Num(if (n - s.min).abs() > (s.max - n).abs() { s.min } else { s.max }),
        };
        assert_ne!(params::set(&mut p, s.key, far), Outcome::Rejected, "{}", s.key);
        p.world = base.world;
        let moved = refreshed(&p, &ws) != reference;
        let marked = params::invalidates(s.key) == Some(PipelineStage::Hydrology);
        assert_eq!(
            moved, marked,
            "{}: refresh_climate {} it, so invalidates() must {} return Hydrology",
            s.key,
            if moved { "reads" } else { "does not read" },
            if moved { "" } else { "not" }
        );
    }
}

/// The one row that is not about climate. `river_density` reaches the civ
/// layer through `fresh_river_order`, and nothing else — so it must mark
/// `Climate`, whose *only* consumer is `civ`, and a recompute must therefore
/// run nothing at all while still reporting civ as outstanding.
#[test]
fn river_density_makes_civ_stale_and_costs_no_climate_pass() {
    assert_eq!(params::invalidates("river_density"), Some(PipelineStage::Climate));
    let (p, mut ws) = tunable_baseline();
    let mut g = cartalith_engine::staleness::pipeline_stage_graph(4);
    g.mark_changed_tiles(PipelineStage::Climate.id(), 0..4, "param:river_density");
    assert!(g.any_stale(PipelineStage::Civ.id()));
    assert!(!g.any_stale(PipelineStage::Hydrology.id()));
    assert!(!g.any_stale(PipelineStage::Climate.id()), "a stage's own mark does not make it stale");

    let before = (ws.temperature.clone(), ws.rainfall.clone(), ws.flow_discharge.clone());
    let r = cartalith_engine::staleness::recompute_stale(&mut g, &p, &mut ws);
    assert!(r.ran.is_empty(), "a civ-only dial must not pay for a climate pass");
    assert_eq!(r.still_stale, vec!["civ"]);
    assert_eq!((ws.temperature, ws.rainfall, ws.flow_discharge), before);
}

/// And the climate half's own behaviour, end to end: marking `Hydrology` is
/// what makes `recompute_stale`'s gate fire, which is the whole reason the
/// table names the node *above* the stage that goes stale.
#[test]
fn a_climate_dial_marks_the_node_that_actually_triggers_a_recompute() {
    assert_eq!(params::invalidates("climate.rain_k"), Some(PipelineStage::Hydrology));
    let (mut p, mut ws) = tunable_baseline();
    let mut g = cartalith_engine::staleness::pipeline_stage_graph(4);
    p.climate.rain_k = 2.0;
    g.mark_changed_tiles(PipelineStage::Hydrology.id(), 0..4, "param:climate.rain_k");
    let rain_before = ws.rainfall.clone();

    let r = cartalith_engine::staleness::recompute_stale(&mut g, &p, &mut ws);
    assert_eq!(r.ran, vec!["hydrology", "climate"]);
    assert_eq!(r.still_stale, vec!["civ"], "civ waits for its own button");
    assert_ne!(ws.rainfall, rain_before, "the moved dial has to actually apply");
}

/// A parameter with no live-apply path marks nothing — the half of the table
/// that is about *not* promising anything. Stated as its own test because it
/// is the answer for 56 of the 81 rows, including every terrain, tectonic
/// and erosion knob.
#[test]
fn generation_time_only_parameters_mark_nothing() {
    for key in [
        "tect.plates", "tect.warp", "stream.k", "passes.evolve_cycles",
        "volc.count", "world_structure.continentality", "carve_rivers", "use_gpu",
        // The two documented exclusions: `recompute_stale` reads neither
        // from the dial table.
        "sea_level", "world",
    ] {
        assert!(params::spec(key).is_some(), "{key} is not a real parameter");
        assert_eq!(params::invalidates(key), None, "{key}");
    }
    assert_eq!(params::invalidates("not.a.parameter"), None);
}
