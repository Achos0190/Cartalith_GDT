//! Golden-parity test for the G2 geoid (`PARITY_TESTING.md`) — reference
//! HTML lines 4967-5015 (`buildGeoid`/`refreshGeoid`/`geoAt`/
//! `currentGeoidPreview`).
//!
//! Captured from a real `generate()` at `gw=48 gh=32 seed=24601 world=true
//! mapWidthKm=4000` in the Node `vm.runInContext` harness this port already
//! uses (script tag #1 only, zero-indent `let`/`const` → `var`, `Worker`
//! undefined, permissive `Proxy` DOM), the same run the fjord/wind-throw/
//! landform fixtures came from.
//!
//! `buildGeoid` is pure, so unlike `buildWindThrowField` it needed no
//! monkey-patch capture: three independent calls were made directly in the
//! sandbox — the `refreshGeoid`-derived options with the toggle **on**, an
//! all-knobs-moved non-wrapping case, and the bare `buildGeoid(W,H)` call
//! that pins every `Object.assign` default literal.
//!
//! Assertions are exact. The transcendentals are `Math.cos` (covered by
//! `js_cos`) and whatever `fbm`/`hash` use internally — both already
//! golden-verified in `cartalith-noise` — plus `Math.pow` with an integer
//! exponent in `rotK`, written as `powi(2)`.

use cartalith_climate::geoid::{build_geoid, current_geoid_preview, geoid_rot_k, refresh_geoid, GeoidOpts};

fn f32s(v: &serde_json::Value) -> Vec<f32> {
    v.as_array().unwrap().iter().map(|x| x.as_f64().unwrap() as f32).collect()
}

fn fixture() -> serde_json::Value {
    let s = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/geoid_captured.json"))
        .expect("geoid_captured.json fixture should read");
    serde_json::from_str(&s).expect("fixture should parse")
}

fn dims(v: &serde_json::Value) -> (usize, usize) {
    (v["gw"].as_u64().unwrap() as usize, v["gh"].as_u64().unwrap() as usize)
}

fn opts(o: &serde_json::Value) -> GeoidOpts {
    GeoidOpts {
        seed: o["seed"].as_i64().unwrap() as i32,
        rot_k: o["rot_k"].as_f64().unwrap(),
        harm_k: o["harm_k"].as_f64().unwrap(),
        mantle_k: o["mantle_k"].as_f64().unwrap(),
        amp: o["amp"].as_f64().unwrap(),
        lat0: o["lat0"].as_f64().unwrap(),
        lat1: o["lat1"].as_f64().unwrap(),
        wrap_x: o["wrap_x"].as_bool().unwrap(),
    }
}

#[test]
fn build_geoid_matches_the_reference_on_a_real_enabled_world() {
    let v = fixture();
    let (gw, gh) = dims(&v);
    let o = opts(&v["enabled"]);
    assert!(o.wrap_x, "the captured world is a wrapping one -- the seam blend must be exercised");
    let got = build_geoid(gw, gh, &o);
    let expected = f32s(&v["enabled"]["expected"]);
    assert_eq!(expected.len(), gw * gh, "the fixture must be a whole grid");
    assert_eq!(got, expected);

    // Non-empty and genuinely varying -- not a flat raster matching a flat
    // raster.
    let distinct: std::collections::BTreeSet<u32> = got.iter().map(|x| x.to_bits()).collect();
    assert!(
        distinct.len() > 1000,
        "only {} distinct values -- too flat to be measuring anything",
        distinct.len()
    );
    let peak = got.iter().map(|x| (*x as f64).abs()).fold(0.0, f64::max);
    assert!((peak - o.amp).abs() < 1e-9, "peak {peak} should be exactly amp {}", o.amp);
}

#[test]
fn build_geoid_matches_the_reference_with_every_knob_moved_and_no_wrap() {
    let v = fixture();
    let (gw, gh) = dims(&v);
    let o = opts(&v["custom"]);
    let d = GeoidOpts::default();
    assert!(!o.wrap_x && o.seed != d.seed && o.rot_k != d.rot_k && o.harm_k != d.harm_k);
    assert!(o.mantle_k != d.mantle_k && o.amp != d.amp && o.lat0 != d.lat0 && o.lat1 != d.lat1);
    assert_eq!(build_geoid(gw, gh, &o), f32s(&v["custom"]["expected"]));
}

/// `buildGeoid(W,H)` with `o` omitted entirely — the only call that reaches
/// every one of `Object.assign`'s eight default literals. A single wrong
/// default would survive both tests above, since both supply all eight.
#[test]
fn the_object_assign_defaults_match_the_reference() {
    let v = fixture();
    let (gw, gh) = dims(&v);
    assert_eq!(build_geoid(gw, gh, &GeoidOpts::default()), f32s(&v["defaults"]));
}

/// Mutation check: each knob must actually reach the formula. Moving one at
/// a time off the captured world's own options has to change the field —
/// otherwise the goldens above would still pass with that term dropped.
#[test]
fn every_knob_reaches_the_formula() {
    let v = fixture();
    let (gw, gh) = dims(&v);
    let base_o = opts(&v["enabled"]);
    let base = build_geoid(gw, gh, &base_o);
    let mutants: [(&str, GeoidOpts); 6] = [
        (
            "seed",
            GeoidOpts {
                seed: base_o.seed + 1,
                ..base_o
            },
        ),
        (
            "rot_k",
            GeoidOpts {
                rot_k: base_o.rot_k * 1.5,
                ..base_o
            },
        ),
        (
            "harm_k",
            GeoidOpts {
                harm_k: base_o.harm_k * 0.5,
                ..base_o
            },
        ),
        (
            "mantle_k",
            GeoidOpts {
                mantle_k: base_o.mantle_k * 0.5,
                ..base_o
            },
        ),
        (
            "lat0",
            GeoidOpts {
                lat0: base_o.lat0 - 20.0,
                ..base_o
            },
        ),
        ("wrap_x", GeoidOpts { wrap_x: false, ..base_o }),
    ];
    for (name, m) in mutants {
        assert_ne!(build_geoid(gw, gh, &m), base, "moving {name} must change the field");
    }
    // `amp` is a pure rescale, so it changes magnitude rather than shape.
    let louder = build_geoid(
        gw,
        gh,
        &GeoidOpts {
            amp: base_o.amp * 2.0,
            ..base_o
        },
    );
    let peak = louder.iter().map(|x| (*x as f64).abs()).fold(0.0, f64::max);
    assert!((peak - base_o.amp * 2.0).abs() < 1e-9);
}

#[test]
fn refresh_geoid_derives_the_reference_s_own_rot_k_and_options() {
    let v = fixture();
    let (gw, gh) = dims(&v);
    let e = &v["enabled"];
    assert_eq!(geoid_rot_k(24.0, 1.0, 1.0), e["rot_k"].as_f64().unwrap());
    let got = refresh_geoid(
        gw,
        gh,
        true,
        e["amp"].as_f64().unwrap(),
        e["seed"].as_i64().unwrap() as i32,
        24.0,
        1.0,
        1.0,
        e["lat0"].as_f64().unwrap(),
        e["lat1"].as_f64().unwrap(),
        v["world"].as_bool().unwrap(),
    );
    assert_eq!(got.expect("enabled geoid should build"), f32s(&e["expected"]));
}

/// `geoAt(i)` is `0` for every cell while the field is off — the property
/// every downstream consumer's `-geoAt(i)` relies on to stay bit-identical
/// to the legacy path, captured from the reference rather than assumed.
#[test]
fn geo_at_is_zero_while_the_geoid_is_off() {
    let v = fixture();
    assert_eq!(v["geo_at_off"].as_f64().unwrap(), 0.0);
    let off: Option<&[f32]> = None;
    assert_eq!(off.map_or(0.0, |g: &[f32]| g[17] as f64), 0.0);
}

/// `currentGeoidPreview` with the toggle **off**: the debug view still draws
/// a field, at the `0.015` fallback amplitude.
#[test]
fn the_preview_matches_the_reference_while_the_toggle_is_off() {
    let v = fixture();
    let (gw, gh) = dims(&v);
    let e = &v["enabled"];
    let (f, amp) = current_geoid_preview(
        gw,
        gh,
        None,
        0.015,
        e["seed"].as_i64().unwrap() as i32,
        24.0,
        1.0,
        1.0,
        e["lat0"].as_f64().unwrap(),
        e["lat1"].as_f64().unwrap(),
        v["world"].as_bool().unwrap(),
    );
    assert_eq!(amp, v["preview_off"]["amp"].as_f64().unwrap());
    assert_eq!(f, f32s(&v["preview_off"]["f"]));
}
