//! Golden-parity tests for `UNIFIED_TOOL_PLAN.md` milestone E's region-export
//! compute half: `amplifyRegion` (reference line 10265) and `refineTile`
//! (10305).
//!
//! # The harness
//!
//! Node `vm.runInContext`, fresh per this project's established practice (not
//! checked in). **Whole `<script>` blocks, not line slices** — milestone D's
//! technique: blocks #1 (2084-14556) and #2 (14563-26720), with the harness
//! asserting that the line before each slice *is* `<script>` and the line
//! after *is* `</script>`, so the boundary is the real delimiter rather than
//! an inferred top-level one.
//!
//! The block-comment balance assertion ran too, and **fired — wrongly, twice**,
//! which is how a check of this kind proves it is looking (milestone D's
//! experience exactly):
//!
//! 1. An orphan `*/` reported in block #2, inside a real comment. Cause:
//!    milestone D's template-literal stack still mishandled a `}` that closes
//!    an *object or arrow body* inside a `${ }` substitution, which ends the
//!    substitution early and desynchronises the rest of the scan. Fixed with a
//!    brace-depth-anchored substitution stack, not by deleting the check.
//! 2. Then six orphan `*/` in block #1, at lines like
//!    `c0.waveStr/Math.max(...)`. Cause: the regex-literal skipper's
//!    "is the previous token a value?" test matched a **single** identifier
//!    character, so any multi-character identifier before a `/` read as
//!    "no value precedes this" and the divide was consumed as a regex. Fixed.
//!
//! The apostrophe-in-prose blind spot the project has documented since
//! milestone C showed up as the *symptom* of (1) rather than as a cause: the
//! desynchronised scan re-entered code inside a block comment and read
//! `stage's water` as a string opener.
//!
//! # The fixture is a synthetic field, and both sides hash it first
//!
//! Unlike milestone D (which reproduced a real `generate_terrain` world), the
//! fixture here is a small synthetic height field built from **pure
//! arithmetic** — no `sin`/`cos`/`exp` anywhere, so V8's libm and Rust's
//! cannot disagree about the input before the function under test even runs.
//! It carries a deliberately **quantised** `% 11` term, urban M3's lesson that
//! a continuous fixture can structurally fail to reach the paths under test.
//! Both sides FNV-1a-64 the raw `f32` bytes and the test asserts the hashes
//! match before trusting any other value — the "the world under the tools is
//! bit-identical" check milestone D established.
//!
//! # Emptiness and shape assertions
//!
//! Because three subsystems have now been bitten by silently-empty output that
//! passed every structural check, the extraction asserted, before any golden
//! was written down: every non-degenerate amplification is non-constant and
//! inside `[0,1]`; the collapsed-region run is constant *and finite*; the
//! `outW == 1` run is **entirely NaN** (a real reference division by zero, see
//! below); and the four `refineTile` tiles agree on their shared edge with
//! delta exactly 0. All are re-asserted here against the port.

use cartalith_spatial::{FloatRegion, Region};
use cartalith_terrain::amplify::{amplify_region, refine_tile, AmplifyOpts};

fn fnv_f32(a: &[f32]) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for v in a {
        for &b in &v.to_bits().to_le_bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(0x100_0000_01b3);
        }
    }
    format!("{h:016x}")
}

/// The harness's own `mkField`, reproduced arithmetic-for-arithmetic.
fn synthetic_field(gw: usize, gh: usize, k: i64) -> Vec<f32> {
    let mut f = vec![0.0f32; gw * gh];
    let cx = gw as f64 * 0.42;
    let cy = gh as f64 * 0.55;
    let r2 = (gw as f64 * 0.3) * (gh as f64 * 0.3);
    for y in 0..gh {
        for x in 0..gw {
            let dx = x as f64 - cx;
            let dy = y as f64 - cy;
            let mut v = 0.30 + 0.62 * f64::max(0.0, 1.0 - (dx * dx + dy * dy) / r2);
            let q = (x as i64 * 7 + y as i64 * 13 + k).rem_euclid(11);
            v += 0.05 * ((q as f64 / 10.0) - 0.5);
            v += 0.10 * f64::max(0.0, 1.0 - (y as f64 - gh as f64 * 0.25).abs() / (gh as f64 * 0.12));
            f[y * gw + x] = v.clamp(0.0, 1.0) as f32;
        }
    }
    f
}

const GW: usize = 48;
const GH: usize = 32;

#[test]
fn the_fixture_field_is_bit_identical_to_the_harnesss_own() {
    // Checked before anything else: every golden below is only evidence about
    // the function under test if the input agrees exactly.
    assert_eq!(fnv_f32(&synthetic_field(GW, GH, 5)), "e6a8f7dd46187082");
}

#[test]
fn case0_default_fbm_detail() {
    let src = synthetic_field(GW, GH, 5);
    let o = amplify_region(&src, GW, GH, &Region { x: 4, y: 6, w: 20, h: 14 }.to_float(), 32, 24,
                           &AmplifyOpts { seed: 1234, sea: 0.42, ridged: false, ..Default::default() });
    assert_eq!(o.len(), 32 * 24);
    assert_eq!(fnv_f32(&o), "2fbdd6aaa9a36b0d");
    // shape: non-constant and inside the unit range, re-asserted from the harness
    assert!(o.iter().any(|v| *v != o[0]));
    assert!(o.iter().all(|v| (0.0..=1.0).contains(v)));
}

#[test]
fn case1_the_same_region_with_ridged_detail_differs() {
    let src = synthetic_field(GW, GH, 5);
    let reg = Region { x: 4, y: 6, w: 20, h: 14 }.to_float();
    let o = amplify_region(&src, GW, GH, &reg, 32, 24,
                           &AmplifyOpts { seed: 1234, sea: 0.42, ridged: true, ..Default::default() });
    assert_eq!(fnv_f32(&o), "a650e09b0454984b");
    // A cross-check the harness could not have faked with one copy-pasted
    // value: the two noise families must not produce the same field.
    let fbm = amplify_region(&src, GW, GH, &reg, 32, 24,
                             &AmplifyOpts { seed: 1234, sea: 0.42, ridged: false, ..Default::default() });
    assert_ne!(fnv_f32(&o), fnv_f32(&fbm));
}

#[test]
fn case2_the_whole_field_downsampled_with_a_raised_frequency_and_amplitude() {
    let src = synthetic_field(GW, GH, 5);
    let o = amplify_region(&src, GW, GH, &Region { x: 0, y: 0, w: 48, h: 32 }.to_float(), 24, 16,
                           // `z_base`/`zoom_detail_k` steer `add_zoom_detail`,
                           // which `amplify_region` never calls -- the
                           // reference's single shared `opts` bag, reproduced.
                           &AmplifyOpts { seed: 99, sea: 0.5, ridged: false, detail_freq: 2.5, detail_amp: 0.3,
                                          ..AmplifyOpts::default() });
    assert_eq!(fnv_f32(&o), "67d797b79ee67574");
}

#[test]
fn case3_a_region_hard_against_the_far_edge_exercises_the_clamped_sampler() {
    let src = synthetic_field(GW, GH, 5);
    // x+w == 48 == GW, so `cx + e` runs past the last column on every row and
    // the relief gradient reads the sampler's clamp, not an interior cell.
    let o = amplify_region(&src, GW, GH, &Region { x: 30, y: 18, w: 18, h: 14 }.to_float(), 40, 30,
                           &AmplifyOpts { seed: 7, sea: 0.42, ridged: false, ..Default::default() });
    assert_eq!(fnv_f32(&o), "e4907cf4893eca3c");
}

#[test]
fn case4_a_collapsed_one_cell_region_is_constant_and_finite() {
    let src = synthetic_field(GW, GH, 5);
    let o = amplify_region(&src, GW, GH, &FloatRegion { x: 5.0, y: 5.0, w: 1.0, h: 1.0 }, 6, 6,
                           &AmplifyOpts { seed: 3, sea: 0.42, ridged: false, ..Default::default() });
    assert_eq!(fnv_f32(&o), "c4011b0e054ff565");
    assert!(o.iter().all(|v| v.to_bits() == o[0].to_bits() && v.is_finite()));
}

#[test]
fn case5_a_one_pixel_output_is_all_nan_exactly_as_the_reference_computes_it() {
    let src = synthetic_field(GW, GH, 5);
    // `(oy/(outH-1))` with outH == 1 and rh > 1 is 0/0. The reference really
    // does return NaN here; `tile_dims`' max(2, ..) floor is why no shipped
    // caller reaches it. Pinned so nobody "fixes" the port into disagreeing.
    let o = amplify_region(&src, GW, GH, &Region { x: 2, y: 2, w: 10, h: 10 }.to_float(), 1, 1,
                           &AmplifyOpts { seed: 3, sea: 0.42, ridged: false, ..Default::default() });
    assert_eq!(fnv_f32(&o), "4a99077f9ba3d218");
    assert!(o[0].is_nan());
}

#[test]
fn case7_a_fully_degenerate_region_and_output_stays_finite() {
    // Both the region AND the output collapse to one cell. Unlike case 5 this
    // is *not* NaN, because `rh > 1.0` is false and the mapping takes its
    // `: ry` branch instead of dividing. The pair of cases is what pins the
    // guard as `> 1.0` rather than `>= 1.0` -- with `>=`, this one would
    // evaluate `(0/0) * 0` and come back NaN. Added after mutation testing
    // found the guard survived on the original fixture set.
    let src = synthetic_field(GW, GH, 5);
    let o = amplify_region(&src, GW, GH, &FloatRegion { x: 5.0, y: 5.0, w: 1.0, h: 1.0 }, 1, 1,
                           &AmplifyOpts { seed: 3, sea: 0.42, ridged: false, ..Default::default() });
    assert_eq!(fnv_f32(&o), "31e755ae5bd7e759");
    assert!(o[0].is_finite());
    assert_eq!(o[0], 0.326_875);
}

#[test]
fn case6_every_option_left_at_its_default() {
    let src = synthetic_field(GW, GH, 5);
    let o = amplify_region(&src, GW, GH, &Region { x: 2, y: 2, w: 10, h: 10 }.to_float(), 8, 8,
                           &AmplifyOpts::default());
    assert_eq!(fnv_f32(&o), "2c1d56fb45ed3b2d");
}

#[test]
fn refine_tile_matches_the_reference_tile_for_tile() {
    let src = synthetic_field(GW, GH, 5);
    let reg = Region { x: 4, y: 4, w: 24, h: 16 }.to_float();
    let o = AmplifyOpts { seed: 4242, sea: 0.42, ridged: false, ..Default::default() };
    let (tw, th) = (16usize, 12usize);
    let want = [
        (0usize, 0usize, "8c568b9c80eb8e88"),
        (0, 1, "7c2fde84ded39f33"),
        (1, 0, "377c45b3a30f53b7"),
        (1, 1, "64b1c71fbf26a8e4"),
    ];
    for (row, col, hash) in want {
        let t = refine_tile(&src, GW, GH, &reg, 2, 2, col, row, tw, th, &o);
        assert_eq!(t.len(), tw * th);
        assert_eq!(fnv_f32(&t), hash, "tile r{row} c{col}");
    }
}

#[test]
fn the_shared_tile_edge_delta_is_exactly_zero_as_the_harness_measured() {
    let src = synthetic_field(GW, GH, 5);
    let reg = Region { x: 4, y: 4, w: 24, h: 16 }.to_float();
    let o = AmplifyOpts { seed: 4242, sea: 0.42, ridged: false, ..Default::default() };
    let (tw, th) = (16usize, 12usize);
    let left = refine_tile(&src, GW, GH, &reg, 2, 2, 0, 0, tw, th, &o);
    let right = refine_tile(&src, GW, GH, &reg, 2, 2, 1, 0, tw, th, &o);
    let mut worst = 0.0f32;
    for y in 0..th {
        worst = worst.max((left[y * tw + (tw - 1)] - right[y * tw]).abs());
    }
    assert_eq!(worst, 0.0);
}
