//! Golden-parity test for the wind-throw hazard field
//! (`PARITY_TESTING.md`) — reference HTML lines 5602-5618 (`_CANOPY`,
//! `buildWindThrowField`).
//!
//! `buildWindThrowField()` takes no arguments: it reads `field`,
//! `state.seaLevel`, `state.world`, `currentWindField()` and
//! `buildBiomeRaster()` as globals. It was therefore captured by the
//! **monkey-patch-and-capture** technique this port's `CHANGELOG.md`
//! records for `stampVolcanoesProvinces` — the sandbox's own
//! `buildWindThrowField` was replaced with a wrapper that snapshots every
//! global input immediately before delegating to the unmodified original,
//! so each input below is what a real `generate()` at
//! `gw=48 gh=32 seed=24601 world=true mapWidthKm=4000` genuinely produced
//! rather than a hand-built approximation of it.
//!
//! Feeding the *captured* wind field in, rather than re-deriving one with
//! `current_wind_field`, is deliberate: that function has its own golden
//! coverage, and re-deriving here would make this suite measure the wind
//! simulation a second time instead of the hazard formula.
//!
//! Assertions are exact. The only transcendental is `Math.hypot`, covered
//! by `js_hypot`; `Math.min` is `js_min`; everything else is fixed-order
//! `f64` arithmetic stored through `f32`.

use cartalith_climate::WindFieldResult;
use cartalith_climate::windthrow::{CANOPY_BIOMES, build_wind_throw_field};

fn f32s(v: &serde_json::Value) -> Vec<f32> {
    v.as_array().unwrap().iter().map(|x| x.as_f64().unwrap() as f32).collect()
}

fn fixture() -> serde_json::Value {
    let s = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/windthrow_captured.json"))
        .expect("windthrow_captured.json fixture should read");
    serde_json::from_str(&s).expect("fixture should parse")
}

fn parts(v: &serde_json::Value) -> (Vec<f32>, Vec<u8>, WindFieldResult, usize, usize, f64, bool) {
    let inp = &v["input"];
    let wind = WindFieldResult {
        u: f32s(&inp["u"]),
        v: f32s(&inp["v"]),
        ww: inp["WW"].as_u64().unwrap() as usize,
        wh: inp["WH"].as_u64().unwrap() as usize,
        max_speed: inp["maxSpeed"].as_f64().unwrap(),
    };
    let biome = inp["biome"].as_array().unwrap().iter().map(|x| x.as_u64().unwrap() as u8).collect();
    (
        f32s(&inp["field"]),
        biome,
        wind,
        v["gw"].as_u64().unwrap() as usize,
        v["gh"].as_u64().unwrap() as usize,
        inp["sea"].as_f64().unwrap(),
        inp["world"].as_bool().unwrap(),
    )
}

#[test]
fn the_canopy_class_set_matches_the_reference() {
    let v = fixture();
    let expected: Vec<u8> = v["canopy"].as_array().unwrap().iter().map(|x| x.as_u64().unwrap() as u8).collect();
    let mut got = CANOPY_BIOMES.to_vec();
    got.sort_unstable();
    assert_eq!(got, expected);
}

#[test]
fn build_wind_throw_field_matches_the_reference_on_a_real_world() {
    let v = fixture();
    let (field, biome, wind, gw, gh, sea, world) = parts(&v);
    assert_eq!(field.len(), gw * gh, "the fixture must be a whole grid");
    assert_eq!(wind.u.len(), wind.ww * wind.wh);

    let got = build_wind_throw_field(&field, &biome, &wind, gw, gh, sea, world);
    assert_eq!(got, f32s(&v["expected"]));

    // Non-empty and genuinely varying -- not a raster of one value that
    // happens to match a raster of the same one value.
    assert_eq!(
        got.iter().filter(|&&x| x > 0.0).count(),
        v["nonzero"].as_u64().unwrap() as usize,
        "the fixture must produce real, non-empty hazard"
    );
    let distinct: std::collections::BTreeSet<u32> = got.iter().map(|x| x.to_bits()).collect();
    assert!(distinct.len() > 100, "only {} distinct values -- too flat to be measuring anything", distinct.len());
}

/// The captured world contains both closed-canopy and open-canopy land, so
/// rewriting the biome raster to a single class must change the answer. If
/// it did not, `_CANOPY` would not be being read at all and the golden
/// above would still pass.
#[test]
fn the_captured_biome_raster_is_actually_consulted() {
    let v = fixture();
    let (field, biome, wind, gw, gh, sea, world) = parts(&v);
    let base = build_wind_throw_field(&field, &biome, &wind, gw, gh, sea, world);

    let closed: std::collections::BTreeSet<u8> = biome.iter().copied().filter(|b| CANOPY_BIOMES.contains(b)).collect();
    assert!(!closed.is_empty(), "the fixture must contain closed-canopy land");
    assert!(
        biome.iter().any(|b| !CANOPY_BIOMES.contains(b) && field[0] >= 0.0),
        "the fixture must contain non-canopy cells too"
    );

    let all_forest = build_wind_throw_field(&field, &vec![5u8; gw * gh], &wind, gw, gh, sea, world);
    assert_ne!(all_forest, base, "flattening every cell to forest must change the field");
}

/// `world` selects whether `bilC` wraps in X. The captured world does
/// wrap, so running it as a region must move the answer at the seam — the
/// only place the two differ.
#[test]
fn the_x_wrap_flag_reaches_the_bilinear_sample() {
    let v = fixture();
    let (field, biome, wind, gw, gh, sea, world) = parts(&v);
    assert!(world, "the captured world must be a wrapping one for this test to say anything");
    let wrapped = build_wind_throw_field(&field, &biome, &wind, gw, gh, sea, true);
    let clamped = build_wind_throw_field(&field, &biome, &wind, gw, gh, sea, false);
    assert_ne!(wrapped, clamped);
}
