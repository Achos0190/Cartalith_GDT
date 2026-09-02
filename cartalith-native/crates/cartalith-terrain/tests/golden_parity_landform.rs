//! Golden-parity test for R5 landform classification
//! (`PARITY_TESTING.md`) — reference HTML lines 8082-8104
//! (`LANDFORM_COLS`, `buildLandformField`).
//!
//! `landform_captured.json` was captured under the same Node
//! `vm.runInContext` harness `golden_parity_center.rs` documents, from a
//! real `generate()` at `gw=48 gh=32 seed=24601 world=true
//! mapWidthKm=4000`. `flow_hi` is the sandbox's own `riverFlowThresh(GW,
//! GH)` — this port takes it as a parameter rather than recomputing it
//! (`cartalith-hydrology` depends on this crate, not the other way round),
//! so pinning the value the reference actually passes matters.
//!
//! Assertions are exact: the output is a `Uint8Array` of class indices, so
//! there is no tolerance to have.
//!
//! # Fixture shapes, and what each is for
//!
//! Class histograms are asserted alongside the rasters, so a change that
//! merely *reshuffles* classes cannot pass by matching a raster length.
//!
//! - `real`: the real world. Reaches all six classes (46 cliff, 6 mesa, 40
//!   cirque, 16 dune, 115 badlands, 87 floodplain) — which is only true
//!   because the seed was chosen for it; a world with no cirque would make
//!   this suite quietly weaker.
//! - `no_climate`: `temp`/`rain`/`flow` all `null`, exercising the
//!   reference's own `:15` / `:0.4` fallbacks and the `flow&&` guard.
//!   Those two literals sit *outside* the dune (`T>18`) and badlands
//!   (`M<0.22`) windows, so this case must produce **only** cliffs and
//!   mesas — asserted, because it is the difference between "the defaults
//!   are these two numbers" and "the defaults are anything".
//! - `arid`: +22 °C, rainfall ×0.15. Dune jumps from 16 to 26 cells and
//!   badlands from 115 to 136, which is the mutation test for the
//!   `18 / 0.12 / 0.3 / 2.5 / 0.5` and `0.22 / 1.4 / 0.55 / 2.2` constants.
//! - `cold`: −25 °C. Cirque rises and dune falls to zero, pinning the
//!   `T<2` side independently.

use cartalith_terrain::landform::{LANDFORM_COLS, LANDFORM_NAMES, build_landform_field};

fn f32s(v: &serde_json::Value) -> Vec<f32> {
    v.as_array().unwrap().iter().map(|x| x.as_f64().unwrap() as f32).collect()
}

fn u8s(v: &serde_json::Value) -> Vec<u8> {
    v.as_array().unwrap().iter().map(|x| x.as_u64().unwrap() as u8).collect()
}

fn hist(out: &[u8]) -> Vec<u64> {
    let mut h = vec![0u64; 7];
    for &v in out {
        h[v as usize] += 1;
    }
    h
}

fn expect_hist(v: &serde_json::Value) -> Vec<u64> {
    v.as_array().unwrap().iter().map(|x| x.as_u64().unwrap()).collect()
}

struct Fx {
    v: serde_json::Value,
    gw: usize,
    gh: usize,
    sea: f64,
    flow_hi: f64,
    field: Vec<f32>,
    temp: Vec<f32>,
    rain: Vec<f32>,
    flow: Vec<f32>,
}

fn fixture() -> Fx {
    let s = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/landform_captured.json"))
        .expect("landform_captured.json fixture should read");
    let v: serde_json::Value = serde_json::from_str(&s).expect("fixture should parse");
    let gw = v["gw"].as_u64().unwrap() as usize;
    let gh = v["gh"].as_u64().unwrap() as usize;
    Fx {
        sea: v["sea"].as_f64().unwrap(),
        flow_hi: v["flow_hi"].as_f64().unwrap(),
        field: f32s(&v["field"]),
        temp: f32s(&v["temp"]),
        rain: f32s(&v["rain"]),
        flow: f32s(&v["flow"]),
        gw,
        gh,
        v,
    }
}

#[test]
fn the_palette_matches_the_reference() {
    let f = fixture();
    let cols: Vec<Vec<f64>> = f.v["cols"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c.as_array().unwrap().iter().map(|x| x.as_f64().unwrap()).collect())
        .collect();
    assert_eq!(cols.len(), LANDFORM_COLS.len());
    assert_eq!(cols.len(), LANDFORM_NAMES.len());
    for (i, c) in cols.iter().enumerate() {
        assert_eq!((c[0], c[1], c[2]), LANDFORM_COLS[i], "class {i} ({})", LANDFORM_NAMES[i]);
    }
}

#[test]
fn build_landform_field_matches_the_reference_on_a_real_world() {
    let f = fixture();
    let got = build_landform_field(
        &f.field,
        Some(&f.temp),
        Some(&f.rain),
        Some(&f.flow),
        f.gw,
        f.gh,
        f.sea,
        f.flow_hi,
    );
    assert_eq!(got, u8s(&f.v["real"]["out"]));
    assert_eq!(hist(&got), expect_hist(&f.v["real"]["hist"]));
    // Every non-zero class must actually occur, or this world is testing
    // fewer branches than the doc comment above claims it does.
    for c in 1u8..=6 {
        assert!(got.contains(&c), "class {c} ({}) is absent -- the fixture is weaker than claimed", LANDFORM_NAMES[c as usize]);
    }
}

#[test]
fn the_no_climate_fallbacks_match_the_reference_and_reach_only_two_classes() {
    let f = fixture();
    let got = build_landform_field(&f.field, None, None, None, f.gw, f.gh, f.sea, f.flow_hi);
    assert_eq!(got, u8s(&f.v["no_climate"]["out"]));
    assert_eq!(hist(&got), expect_hist(&f.v["no_climate"]["hist"]));
    for c in [3u8, 4, 5, 6] {
        assert!(!got.contains(&c), "{} must be unreachable at T=15, M=0.4, flow=null", LANDFORM_NAMES[c as usize]);
    }
    assert!(got.contains(&1) && got.contains(&2), "cliff and mesa must still be reachable");
}

#[test]
fn a_hot_arid_world_matches_the_reference_and_moves_dune_and_badlands() {
    let f = fixture();
    let temp = f32s(&f.v["arid"]["temp"]);
    let rain = f32s(&f.v["arid"]["rain"]);
    let got = build_landform_field(&f.field, Some(&temp), Some(&rain), Some(&f.flow), f.gw, f.gh, f.sea, f.flow_hi);
    assert_eq!(got, u8s(&f.v["arid"]["out"]));
    let h = hist(&got);
    assert_eq!(h, expect_hist(&f.v["arid"]["hist"]));
    let base = expect_hist(&f.v["real"]["hist"]);
    assert_ne!(h[4], base[4], "aridity must move the dune count");
    assert_ne!(h[5], base[5], "aridity must move the badlands count");
}

#[test]
fn a_cold_world_matches_the_reference_and_moves_cirque_and_dune() {
    let f = fixture();
    let temp = f32s(&f.v["cold"]["temp"]);
    let got = build_landform_field(&f.field, Some(&temp), Some(&f.rain), Some(&f.flow), f.gw, f.gh, f.sea, f.flow_hi);
    assert_eq!(got, u8s(&f.v["cold"]["out"]));
    let h = hist(&got);
    assert_eq!(h, expect_hist(&f.v["cold"]["hist"]));
    let base = expect_hist(&f.v["real"]["hist"]);
    assert_ne!(h[3], base[3], "cold must move the cirque count");
    assert_eq!(h[4], 0, "nothing is a dune at -25 C");
    assert_ne!(base[4], 0, "...and the warm world's dunes were real");
}

/// `flow_hi` is a parameter here and a `riverFlowThresh(W,H)` call in the
/// reference. Feeding it a different threshold must move the floodplain
/// count, or the parameter is not actually being read.
#[test]
fn the_flow_threshold_is_read_rather_than_assumed() {
    let f = fixture();
    let build = |t: f64| {
        build_landform_field(&f.field, Some(&f.temp), Some(&f.rain), Some(&f.flow), f.gw, f.gh, f.sea, t)
    };
    let base = hist(&build(f.flow_hi));
    assert!(base[6] > 0, "the real threshold must produce floodplain at all");
    assert_eq!(hist(&build(f64::INFINITY))[6], 0, "an unreachable threshold means no floodplain");
    // Floodplain is the *last* branch, so lowering the threshold can only
    // ever claim cells no earlier class already took -- monotone, not
    // strictly increasing.
    assert!(hist(&build(f.flow_hi * 0.01))[6] >= base[6], "a lower threshold cannot lose floodplain");
}
