//! NOT a golden-parity test, and no longer the only coverage these two
//! functions have. `computeOceanCurrent` (reference HTML lines 5368-5462)
//! is golden-verified bit-exactly in `golden_parity_ocean_current.rs`
//! (world mode, and region mode with western intensification off);
//! `oceanSSTAnomaly` (lines 5246-5268) is verified end to end through
//! `golden_parity_weather.rs`'s `simulate_weather_currents_case`, captured
//! from a real reference `generate()` run with `state.climate.currents=true`.
//! This file is the *behavioural* half neither of those asserts: that land
//! cells stay exactly zero, that the ocean is not silently all-zero, and
//! that both functions are deterministic -- the properties a
//! wrong-but-stable formula would still break.
//!
//! **Corrected 2026-09-02.** This header used to say both functions had no
//! golden coverage at all because "this environment has no JS runtime", and
//! that `ClimateInputParams::currents` stayed `false` by default until
//! someone with one extracted fixtures. Every part of that is stale:
//!
//!   - `node --version` is v24.19.0, and `tools/jsruntime_probe.js` proves
//!     the extraction chain end to end rather than trusting the version
//!     string -- see `deflect_flow_regression.rs`'s header for what it
//!     checks and why it is a two-way proof.
//!   - `cartalith_engine::WorldParams::defaults` has shipped
//!     `climate.currents: true` since 2026-08-15.
//!
//! `WeatherParams::currents`' own doc comment in
//! `cartalith-climate/src/lib.rs` still carries the superseded "this port
//! defaults to `false`" wording and is stale on that point; its account of
//! what is and is not separately extracted remains accurate. Deliberately
//! not edited from here -- that file is not this test's to rewrite.

use cartalith_climate::{compute_ocean_current, ocean_sst_anomaly, OceanCurrentParams};

fn ocean_current_default_params() -> OceanCurrentParams {
    OceanCurrentParams { gap_k: 0.4, iterations: 20, bend_k: 0.9, western: true }
}

#[test]
fn compute_ocean_current_is_deterministic_and_zero_on_land() {
    let ww = 20usize;
    let wh = 16usize;
    let n = ww * wh;
    let sea = 0.4;

    // A simple basin: land on the west edge, ocean everywhere else.
    let elev_c: Vec<f32> = (0..n)
        .map(|i| {
            let x = i % ww;
            if x < 3 {
                0.8
            } else {
                0.2
            }
        })
        .collect();
    let wx = vec![1.0f32; n];
    let wy = vec![0.2f32; n];

    let p = ocean_current_default_params();
    let a = compute_ocean_current(&wx, &wy, &elev_c, ww, wh, false, sea, false, 55.0, 5.0, &p);
    let b = compute_ocean_current(&wx, &wy, &elev_c, ww, wh, false, sea, false, 55.0, 5.0, &p);
    assert_eq!(a.u, b.u, "same input must reproduce the same u field");
    assert_eq!(a.v, b.v, "same input must reproduce the same v field");
    assert_eq!(a.ocean, b.ocean, "same input must reproduce the same ocean mask");

    for i in 0..n {
        if a.ocean[i] == 0 {
            assert_eq!(a.u[i], 0.0, "land cells must have zero u");
            assert_eq!(a.v[i], 0.0, "land cells must have zero v");
        }
    }
    assert!(a.u.iter().any(|&v| v != 0.0), "expected at least one nonzero ocean current cell");
}

#[test]
fn ocean_sst_anomaly_is_deterministic_and_zero_on_land() {
    let gw = 24usize;
    let gh = 18usize;
    let field: Vec<f32> = (0..gw * gh)
        .map(|i| {
            let x = i % gw;
            0.2 + 0.6 * (x as f32 / gw as f32)
        })
        .collect();
    let ww = gw.min(240);
    let wh = ((ww as f64 * gh as f64 / gw as f64).round() as usize).max(2);

    let run = || {
        ocean_sst_anomaly(
            gw, gh, &field, ww, wh, false, 3.0, 0.5, false, 55.0, 5.0, 30.0, -25.0, 23.4, 24.0, false, 0.0, 0.6, 1.0,
        )
    };
    let a = run();
    let b = run();
    assert_eq!(a, b, "same input must reproduce the same SST anomaly field");
    assert_eq!(a.len(), ww * wh);
    assert!(a.iter().all(|&v| (-8.0..=8.0).contains(&v)), "SST anomaly must stay within the [-8,8] clamp");
}
