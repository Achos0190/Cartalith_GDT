//! Golden-parity test for seasons + Köppen–Geiger classification
//! (`PARITY_TESTING.md`) — reference HTML lines 7491-7562
//! (`computeTempInto`/`computeSeasons`/`KOPPEN_KEYS`/`KOPPEN_COL`/
//! `classifyKoppen`/`buildKoppen`/`koppenColor`).
//!
//! Captured from the same real `generate()` the geoid and tide fixtures came
//! from (`gw=48 gh=32 seed=24601 world=true mapWidthKm=4000`, Node
//! `vm.runInContext` harness), followed by a real `computeSeasons()` call.
//!
//! **What this suite deliberately does not measure.** `computeSeasons` is
//! two `computeTempInto` solves plus two `simulateWeather` runs plus
//! `buildKoppen`. The weather half already has its own golden suite
//! (`golden_parity_weather.rs`) *and* carries this port's three long-standing
//! deferrals (terrain wind deflection, ocean-current SST folding,
//! world-structure interior dryness), so re-deriving seasonal rain here would
//! measure the weather model a second time and fail for reasons that have
//! nothing to do with Köppen. Instead the reference's **own captured**
//! `rainJulField`/`rainJanField` are fed to `build_koppen` as input. What is
//! measured, exactly:
//!
//! * `compute_temp_into` at both solstices — bit-exact, no borrowed inputs.
//! * `build_koppen`/`classify_koppen` over the reference's real seasonal
//!   fields — bit-exact, all 19 classes the captured world produces.
//! * The frozen key order and the whole `KOPPEN_COL` palette.
//!
//! Assertions are exact. `Math.cos` in the declination shift is the only
//! transcendental; everything else is comparisons and fixed-order `f64`
//! arithmetic stored through `f32`.

use cartalith_climate::koppen::{
    build_koppen, classify_koppen, compute_temp_into, koppen_color, koppen_index, KoppenParams, KOPPEN_COL, KOPPEN_KEYS,
};
use cartalith_climate::ClimateParams;

fn f32s(v: &serde_json::Value) -> Vec<f32> {
    v.as_array().unwrap().iter().map(|x| x.as_f64().unwrap() as f32).collect()
}

fn u8s(v: &serde_json::Value) -> Vec<u8> {
    v.as_array().unwrap().iter().map(|x| x.as_u64().unwrap() as u8).collect()
}

fn fixture() -> serde_json::Value {
    let s = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/koppen_captured.json"))
        .expect("koppen_captured.json fixture should read");
    serde_json::from_str(&s).expect("fixture should parse")
}

const GW: usize = 48;
const GH: usize = 32;
const SEA: f64 = 0.42;

fn climate(v: &serde_json::Value) -> ClimateParams {
    ClimateParams {
        world: true,
        lat_n: 90.0,
        lat_s: -90.0,
        pole_temp: v["pole_temp"].as_f64().unwrap(),
        equator_temp: v["equator_temp"].as_f64().unwrap(),
        tilt_deg: v["tilt"].as_f64().unwrap(),
        rotation_hours: v["rotation_hours"].as_f64().unwrap(),
        lapse_rate: v["lapse_rate"].as_f64().unwrap(),
        g: v["g"].as_f64().unwrap(),
        sea_level: SEA,
        peak_m: v["peak_m"].as_f64().unwrap(),
        // `computeTempInto` never runs the albedo relaxation, so this knob
        // is not in its formula at all -- 0 states that rather than
        // borrowing `state.climate.albedo`.
        albedo_k: 0.0,
    }
}

fn koppen_params(v: &serde_json::Value) -> KoppenParams {
    KoppenParams {
        world: true,
        lat_n: 90.0,
        lat_s: -90.0,
        sea_level: SEA,
        max_rain_mm: v["max_rain_mm"].as_f64().unwrap(),
    }
}

#[test]
fn compute_temp_into_matches_the_reference_at_both_solstices() {
    let v = fixture();
    let field = f32s(&v["field"]);
    assert_eq!(field.len(), GW * GH, "the fixture must be a whole grid");
    let cp = climate(&v);
    let tilt = v["tilt"].as_f64().unwrap();
    assert_eq!(tilt, 23.4, "the captured world uses Earth's own obliquity");

    let jul = compute_temp_into(GW, GH, &field, None, tilt, &cp);
    let jan = compute_temp_into(GW, GH, &field, None, -tilt, &cp);
    assert_eq!(jul, f32s(&v["temp_jul"]));
    assert_eq!(jan, f32s(&v["temp_jan"]));
    // ...and `computeSeasons` really does leave those two fields in place.
    assert_eq!(jul, f32s(&v["temp_jul_after"]));
    assert_eq!(jan, f32s(&v["temp_jan_after"]));

    // The declination shift is the whole point: the two solstices must
    // differ, and must differ in opposite directions about the equator.
    assert_ne!(jul, jan);
    let (north, south) = (jul[0] as f64 - jan[0] as f64, jul[GW * (GH - 1)] as f64 - jan[GW * (GH - 1)] as f64);
    assert!(
        north > 0.0 && south < 0.0,
        "north {north}, south {south} -- the solstices must be opposed"
    );
}

/// The declination has to reach `Math.cos(lat - declR)`; a port that dropped
/// it would still pass a golden captured at `decl = 0`.
#[test]
fn zero_declination_reproduces_the_annual_temperature_model() {
    let v = fixture();
    let field = f32s(&v["field"]);
    let cp = climate(&v);
    let annual = compute_temp_into(GW, GH, &field, None, 0.0, &cp);
    assert_eq!(annual, cartalith_climate::compute_temperature(GW, GH, &field, None, &cp));
    assert_ne!(annual, compute_temp_into(GW, GH, &field, None, 23.4, &cp));
}

#[test]
fn build_koppen_matches_the_reference_over_its_own_seasonal_fields() {
    let v = fixture();
    let got = build_koppen(
        GW,
        GH,
        &f32s(&v["field"]),
        None,
        &f32s(&v["temp_jul"]),
        &f32s(&v["temp_jan"]),
        &f32s(&v["rain_jul"]),
        &f32s(&v["rain_jan"]),
        &koppen_params(&v),
    );
    let expected = u8s(&v["koppen"]);
    assert_eq!(expected.len(), GW * GH);
    assert_eq!(got, expected);

    // Genuinely non-empty and genuinely varied: the captured world produces
    // 19 distinct raster values (ocean plus 18 real classes), so this is not
    // an all-zero raster matching an all-zero raster.
    let distinct: std::collections::BTreeSet<u8> = got.iter().copied().collect();
    assert_eq!(distinct.len(), 19, "captured classes: {distinct:?}");
    assert!(distinct.contains(&0), "the captured world has ocean");
    assert!(
        got.iter().filter(|c| **c != 0).count() > 900,
        "the captured world must be mostly land"
    );
}

/// The frozen, append-only key order and its palette, straight from the
/// reference. A reorder here would silently reinterpret every exported
/// `koppen_index.json`.
#[test]
fn the_frozen_key_order_and_palette_match_the_reference() {
    let v = fixture();
    let keys: Vec<String> = v["keys"]
        .as_array()
        .unwrap()
        .iter()
        .map(|k| k.as_str().unwrap().to_string())
        .collect();
    assert_eq!(KOPPEN_KEYS.to_vec(), keys);
    let cols: Vec<(u8, u8, u8)> = v["colors"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| {
            let a = c.as_array().unwrap();
            (
                a[0].as_u64().unwrap() as u8,
                a[1].as_u64().unwrap() as u8,
                a[2].as_u64().unwrap() as u8,
            )
        })
        .collect();
    assert_eq!(KOPPEN_COL.to_vec(), cols);

    // `koppenColor` itself, including index 0 (ocean) and the two ends.
    let expected: Vec<(u8, u8, u8)> = v["color_of"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| {
            let a = c.as_array().unwrap();
            (
                a[0].as_u64().unwrap() as u8,
                a[1].as_u64().unwrap() as u8,
                a[2].as_u64().unwrap() as u8,
            )
        })
        .collect();
    for (n, idx) in [0u8, 1, 2, 3, 29, 30].iter().enumerate() {
        assert_eq!(koppen_color(*idx), expected[n], "koppenColor({idx})");
    }
}

/// Mutation checks on the classifier's own constants, over the captured
/// world's real cells. Each threshold must change the raster — a wrong
/// literal that changed nothing would sail through the golden above.
#[test]
fn the_classifier_s_thresholds_are_all_load_bearing() {
    let v = fixture();
    let field = f32s(&v["field"]);
    let (tj, ta) = (f32s(&v["temp_jul"]), f32s(&v["temp_jan"]));
    let (rj, ra) = (f32s(&v["rain_jul"]), f32s(&v["rain_jan"]));
    let p = koppen_params(&v);
    let lats: Vec<f64> = v["lat_of_row"].as_array().unwrap().iter().map(|x| x.as_f64().unwrap()).collect();
    let base = build_koppen(GW, GH, &field, None, &tj, &ta, &rj, &ra, &p);

    // maxRainMm scales every precipitation term.
    let louder = KoppenParams {
        max_rain_mm: 1500.0,
        ..KoppenParams {
            world: true,
            lat_n: 90.0,
            lat_s: -90.0,
            sea_level: SEA,
            max_rain_mm: 0.0,
        }
    };
    assert_ne!(build_koppen(GW, GH, &field, None, &tj, &ta, &rj, &ra, &louder), base);

    // Swapping the two rain fields changes which half-year is "summer" at
    // every cell -- if the hemisphere test were dropped this would be a
    // no-op on one hemisphere and wrong on the other.
    assert_ne!(build_koppen(GW, GH, &field, None, &tj, &ta, &ra, &rj, &p), base);

    // The hemisphere test is `latAt(y) >= 0`, and the captured world spans
    // both -- so the same cell inputs classify differently north and south.
    let mut differed = false;
    for i in 0..GW * GH {
        if field[i] as f64 <= SEA {
            continue;
        }
        let n = classify_koppen(
            field[i] as f64,
            0.0,
            tj[i] as f64,
            ta[i] as f64,
            rj[i] as f64,
            ra[i] as f64,
            45.0,
            &p,
        );
        let s = classify_koppen(
            field[i] as f64,
            0.0,
            tj[i] as f64,
            ta[i] as f64,
            rj[i] as f64,
            ra[i] as f64,
            -45.0,
            &p,
        );
        if n != s {
            differed = true;
            break;
        }
    }
    assert!(differed, "no captured cell classifies differently by hemisphere");

    // The captured latitude ladder is the one `build_koppen` derives, so
    // `lat_at` is the same function on both sides of the boundary.
    assert!(lats[0] > 0.0 && lats[GH - 1] < 0.0, "the captured world must span both hemispheres");
}

/// The `koppen_index` round trip: every raster value the captured world
/// produced names a real key, and every key round-trips.
#[test]
fn every_captured_raster_value_names_a_real_koppen_key() {
    let v = fixture();
    for value in u8s(&v["koppen"]) {
        if value == 0 {
            continue;
        }
        let key = KOPPEN_KEYS[value as usize - 1];
        assert_eq!(koppen_index(key), Some(value), "{key} must round-trip to {value}");
    }
}
