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
