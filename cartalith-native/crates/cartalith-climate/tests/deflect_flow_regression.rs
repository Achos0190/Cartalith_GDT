//! NOT a golden-parity test. `deflect_flow` (reference HTML `deflectFlow`,
//! lines 5315-5357) has no golden-parity coverage in this port: this
//! environment has no JS runtime (`PARITY_TESTING.md`'s own extraction
//! procedure needs one to run the reference HTML and read back real
//! numbers), so there's nothing to extract fixtures from for a
//! multi-iteration flow-blending algorithm. This file is a
//! same-input-same-output regression/sanity check only -- it catches a
//! future refactor breaking determinism or making the function a no-op,
//! NOT a wrong-vs-JS formula. `cartalith_engine`'s
//! `WeatherParams::terrain_wind_deflection` (which gates whether
//! `build_wind` calls this at all) stays `false` by default until someone
//! with a JS runtime extracts real fixtures -- see that field's own doc
//! comment.

use cartalith_climate::{deflect_flow, DeflectFlowParams};

fn deflect_default_params() -> DeflectFlowParams {
    DeflectFlowParams { strength: 1.0, k1: 0.6, k2: 0.65, gap_k: 0.32, iterations: 16, block_blur: 2 }
}

#[test]
fn deflect_flow_is_deterministic_and_bends_around_a_ridge() {
    let ww = 16usize;
    let wh = 12usize;
    let n = ww * wh;

    // Uniform eastward wind...
    let u0 = vec![1.0f32; n];
    let v0 = vec![0.0f32; n];
    // ...blocked by a north-south ridge down the middle column.
    let mut block = vec![0f32; n];
    for y in 0..wh {
        block[y * ww + ww / 2] = 1.0;
    }

    let p = deflect_default_params();
    let (u_a, v_a) = deflect_flow(&u0, &v0, &block, ww, wh, false, &p);
    let (u_b, v_b) = deflect_flow(&u0, &v0, &block, ww, wh, false, &p);
    assert_eq!(u_a, u_b, "same input must reproduce the same u field");
    assert_eq!(v_a, v_b, "same input must reproduce the same v field");

    // Immediately upstream of the ridge, flow should be measurably
    // deflected away from its original pure-eastward direction.
    let probe = 3 * ww + ww / 2 - 1;
    assert!((v_a[probe] as f64).abs() > 1e-3, "expected the ridge to deflect flow off its original heading");

    // A cell far from the ridge (wrapped edge column) should be
    // essentially untouched.
    let far = 3 * ww;
    assert!((u_a[far] as f64 - 1.0).abs() < 0.05, "expected flow far from the ridge to stay close to the original wind");
}

#[test]
fn deflect_flow_zero_strength_is_a_near_identity() {
    let ww = 10usize;
    let wh = 8usize;
    let n = ww * wh;
    let u0: Vec<f32> = (0..n).map(|i| (i % 3) as f32 * 0.3).collect();
    let v0: Vec<f32> = (0..n).map(|i| (i % 5) as f32 * 0.1).collect();
    let mut block = vec![0f32; n];
    block[n / 2] = 1.0;

    let p = DeflectFlowParams { strength: 0.0, k1: 0.6, k2: 0.65, gap_k: 0.0, iterations: 16, block_blur: 2 };
    let (u, v) = deflect_flow(&u0, &v0, &block, ww, wh, false, &p);
    assert_eq!(u, u0, "strength 0 and gap_k 0 should leave u untouched (no redirect, no gap-speed rescale)");
    assert_eq!(v, v0, "strength 0 and gap_k 0 should leave v untouched");
}
