//! Golden-parity test for the G3 tidal-range field (`PARITY_TESTING.md`) —
//! reference HTML lines 5016-5048 (`tidalForcing`/`computeTideField`/
//! `buildTideField`/`refreshTides`/`currentTideField`).
//!
//! Captured from the same real `generate()` the geoid fixture came from
//! (`gw=48 gh=32 seed=24601 world=true mapWidthKm=4000`, Node
//! `vm.runInContext` harness). `computeTideField` reads `field`,
//! `state.seaLevel`, `state.planet.g` and `geoidField` as globals but takes
//! its moon roster as an argument, so it was driven directly rather than
//! monkey-patched: the captured `field` below is fed back in, and the three
//! global-dependent cases (default, `g`/`k2`/moons all moved, and geoid
//! **on**) were each produced by setting exactly that global in the sandbox.
//!
//! Assertions are exact. `Math.exp` is `js_exp` (V8's libm is not Rust's —
//! this project has already been bitten by exactly this function);
//! `Math.min` is `js_min`; `Math.pow(x, -0.25)` is `powf`, whose result is
//! pinned by the golden itself.

use cartalith_climate::tides::{build_tide_field, compute_tide_field, current_tide_field, tidal_forcing, Moon, TideParams};

fn f32s(v: &serde_json::Value) -> Vec<f32> {
    v.as_array().unwrap().iter().map(|x| x.as_f64().unwrap() as f32).collect()
}

fn fixture() -> serde_json::Value {
    let s = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/tides_captured.json"))
        .expect("tides_captured.json fixture should read");
    serde_json::from_str(&s).expect("fixture should parse")
}

fn moons(v: &serde_json::Value) -> Vec<Moon> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|m| Moon {
            // The reference's own `m.massRel||0` / `m.distRel||1` coercions,
            // applied where a JS object literal would have left the key out.
            mass_rel: m["massRel"].as_f64().unwrap_or(0.0),
            dist_rel: m["distRel"].as_f64().unwrap_or(1.0),
        })
        .collect()
}

#[test]
fn tidal_forcing_matches_the_reference_including_its_own_coercions() {
    let v = fixture();
    let cases = v["forcing"].as_array().unwrap();
    assert_eq!(cases.len(), 5, "the fixture must cover empty / one / two / floored / defaulted");
    for c in cases {
        assert_eq!(tidal_forcing(&moons(&c["moons"])), c["f"].as_f64().unwrap());
    }
    // The last case is the bare `{}` moon: massRel -> 0, distRel -> 1, so
    // it contributes nothing. If `||1` were read as `||0` this would be
    // infinite instead.
    assert_eq!(cases[4]["f"].as_f64().unwrap(), 0.0);
}

#[test]
fn compute_tide_field_matches_the_reference_on_a_real_world() {
    let v = fixture();
    let (gw, gh) = (v["gw"].as_u64().unwrap() as usize, v["gh"].as_u64().unwrap() as usize);
    let field = f32s(&v["field"]);
    assert_eq!(field.len(), gw * gh, "the fixture must be a whole grid");
    let p = TideParams {
        g: v["plain"]["g"].as_f64().unwrap(),
        k2: v["plain"]["k2"].as_f64().unwrap(),
        moons: moons(&v["plain"]["moons"]),
    };
    let got = compute_tide_field(gw, gh, &field, None, v["sea"].as_f64().unwrap(), &p);
    assert_eq!(got, f32s(&v["plain"]["expected"]));

    // Real, non-empty, genuinely varying water -- and real land.
    let wet = got.iter().filter(|x| **x > 0.0).count();
    assert!(
        wet > 100 && wet < gw * gh,
        "{wet} wet cells of {} -- the fixture must have both",
        gw * gh
    );
    let distinct: std::collections::BTreeSet<u32> = got.iter().map(|x| x.to_bits()).collect();
    assert!(distinct.len() > 100, "only {} distinct values -- too flat", distinct.len());
}

#[test]
fn compute_tide_field_matches_the_reference_with_gravity_love_number_and_two_moons_moved() {
    let v = fixture();
    let (gw, gh) = (v["gw"].as_u64().unwrap() as usize, v["gh"].as_u64().unwrap() as usize);
    let c = &v["custom"];
    let p = TideParams {
        g: c["g"].as_f64().unwrap(),
        k2: c["k2"].as_f64().unwrap(),
        moons: moons(&c["moons"]),
    };
    assert_eq!(p.moons.len(), 2);
    let got = compute_tide_field(gw, gh, &f32s(&v["field"]), None, v["sea"].as_f64().unwrap(), &p);
    assert_eq!(got, f32s(&c["expected"]));
}

/// The `eff = field − geoid` branch. The reference materialises it as its
/// own `Float32Array` first, so the subtraction rounds through `f32` once
/// and *both* the coast-distance transform and the depth term read the same
/// rounded values.
#[test]
fn compute_tide_field_matches_the_reference_with_the_geoid_on() {
    let v = fixture();
    let (gw, gh) = (v["gw"].as_u64().unwrap() as usize, v["gh"].as_u64().unwrap() as usize);
    let field = f32s(&v["field"]);
    let geoid = f32s(&v["with_geoid"]["geoid"]);
    assert_eq!(geoid.len(), gw * gh);
    assert!(geoid.iter().any(|g| *g != 0.0), "the captured geoid must be a real field");
    let p = TideParams::default();
    let sea = v["sea"].as_f64().unwrap();
    let got = compute_tide_field(gw, gh, &field, Some(&geoid), sea, &p);
    assert_eq!(got, f32s(&v["with_geoid"]["expected"]));
    // ...and it genuinely differs from the geoid-off answer, so this test
    // is not silently re-running the one above.
    assert_ne!(got, compute_tide_field(gw, gh, &field, None, sea, &p));
}

/// Mutation checks against the captured world: each of the three
/// amplification terms has to reach the answer.
#[test]
fn every_amplification_term_reaches_the_field() {
    let v = fixture();
    let (gw, gh) = (v["gw"].as_u64().unwrap() as usize, v["gh"].as_u64().unwrap() as usize);
    let field = f32s(&v["field"]);
    let sea = v["sea"].as_f64().unwrap();
    let base = compute_tide_field(gw, gh, &field, None, sea, &TideParams::default());

    // A0: gravity, Love number and moon mass each scale the whole field.
    for p in [
        TideParams {
            g: 2.0,
            ..TideParams::default()
        },
        TideParams {
            k2: 0.5,
            ..TideParams::default()
        },
        TideParams {
            moons: vec![Moon {
                mass_rel: 2.0,
                dist_rel: 1.0,
            }],
            ..TideParams::default()
        },
        TideParams {
            moons: vec![Moon {
                mass_rel: 1.0,
                dist_rel: 2.0,
            }],
            ..TideParams::default()
        },
    ] {
        assert_ne!(compute_tide_field(gw, gh, &field, None, sea, &p), base);
    }
    // Green's law: the shallow-water cap is real -- some cell must sit at
    // the capped 3.0 multiplier, otherwise `min(3.0, ...)` is dead code and
    // a wrong cap would go unnoticed.
    let a0 = 0.04 * tidal_forcing(&[Moon::DEFAULT]);
    let capped = base.iter().filter(|x| (**x as f64) > a0 * 2.99).count();
    assert!(capped > 0, "no cell reaches the Green's-law cap -- the fixture never exercises it");
}

#[test]
fn the_enable_gate_and_the_off_toggle_preview_match_the_reference() {
    let v = fixture();
    let (gw, gh) = (v["gw"].as_u64().unwrap() as usize, v["gh"].as_u64().unwrap() as usize);
    let field = f32s(&v["field"]);
    let sea = v["sea"].as_f64().unwrap();
    assert!(
        v["disabled_is_null"].as_bool().unwrap(),
        "the reference nulls the field when tides are off"
    );
    assert!(build_tide_field(gw, gh, &field, None, sea, false, &TideParams::default()).is_none());

    // `currentTideField` previews with the toggle off, and reports the same
    // maximum the view divides by.
    let (f, mx) = current_tide_field(gw, gh, &field, None, sea, None, &TideParams::default());
    assert_eq!(f, f32s(&v["plain"]["expected"]));
    let expect_max = f.iter().map(|x| *x as f64).fold(1e-6, f64::max);
    assert_eq!(mx, expect_max);
    assert!(mx > 0.1, "the captured world must have real tidal range, got {mx}");
}
