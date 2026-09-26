//! `OUTSTANDING_WORK.md` §2.11, "The Village carto preset still looks harsh
//! (yellow/purple/teal)". Ruling AZ (`LARGE_ITEM_RULINGS.md`, 2026-09-28):
//! "Village map style. Softer: more colour bands, matching the other
//! presets." `render.rs::quantize_flat_palette`'s `BANDS` moved from `3.0`
//! (four flat levels per channel, this port's original own choice, never a
//! verified reference constant) to `5.0` (six levels), by measurement:
//! `godot-project/_villagebands_probe.gd`, fixed seed 20260927, 512x328,
//! sRGB, windowed. The harsh-transition metric (fraction of adjacent pixel
//! pairs whose Euclidean RGB distance exceeds 90 of a possible 441.7) put
//! Village at 0.044551 with `BANDS = 3` against the other six
//! `STYLE_PRESETS` entries' own range [0.029251, 0.032794] on the same
//! world; `BANDS = 5` is the smallest count that lands inside that range
//! (0.031397; `BANDS = 7` gives 0.029340, also inside but not smaller).
//!
//! Two tests here, per this project's own preflight rule ("a test that pins
//! a constant asserts a literal, never the constant against itself"):
//! [`village_quantises_to_six_levels_of_51`] hard-codes `apply_npr`'s actual
//! output for six input triples as literal expected floats (every one a
//! multiple of the literal `51.0` -- `255.0 / 5.0`, spelled out rather than
//! computed from `BANDS`) and [`non_village_presets_are_unmoved_by_the_band_count`]
//! pins two "other preset" shapes (sepia+multi-sun, matching "Antique") that
//! never reach `quantize_flat_palette` at all, so a change that leaked the
//! band count into the non-village path would fail here.

#[path = "../src/render.rs"]
mod render;
use render::{Npr, TerrainAppearance};

/// `l0, l1, l2, r, slope, curv, gx, gy, px, py` -- shared with
/// `golden_parity_npr.rs`'s `NPR_CASES` (a subset), plus two hand-picked
/// triples reaching the clamp ends (all-near-black, and a saturated
/// primary) that the golden's own 16 cases do not isolate on their own.
const CASES: [[f64; 10]; 6] = [
    [180f64, 170f64, 150f64, 0.25f64, 0.01f64, 0.003f64, 0.01f64, 0.004f64, 3f64, 5f64],
    [60f64, 72f64, 45f64, 0.4f64, 0.04f64, 0.0005f64, -0.02f64, 0.011f64, 11f64, 2f64],
    [220f64, 225f64, 230f64, 0.15f64, 0.002f64, -0.004f64, 0.0004f64, -0.0009f64, 17f64, 23f64],
    [12f64, 15f64, 10f64, 0.85f64, 0.15f64, 0.025f64, -0.08f64, -0.045f64, 63f64, 63f64],
    [25.5f64, 76.4f64, 127.6f64, 0.5f64, 0.03f64, 0.002f64, 0.015f64, -0.009f64, 7f64, 58f64],
    [255f64, 0f64, 128.0f64, 0.5f64, 0.03f64, 0.002f64, 0.015f64, -0.009f64, 7f64, 58f64],
];

/// `apply_npr` with `Npr::village` on, for each of [`CASES`] -- the literal
/// floats `apply_npr` actually returns at `BANDS = 5.0`. Every one is a
/// multiple of the literal `51.0` (`255.0 / 5.0`, six levels: 0, 51, 102,
/// 153, 204, 255), spelled out below rather than derived from that division,
/// so this assertion cannot pass by re-deriving the same constant it is
/// meant to pin (`MISTAKES.md`'s "write a test that pins a constant" row).
const VILLAGE_EXPECTED: [(f64, f64, f64); 6] = [
    (204.0, 153.0, 153.0),
    (51.0, 51.0, 51.0),
    (204.0, 204.0, 255.0),
    (0.0, 0.0, 0.0),
    (51.0, 51.0, 153.0),
    (255.0, 0.0, 153.0),
];

#[test]
fn village_quantises_to_six_levels_of_51() {
    let a = TerrainAppearance { npr: Npr { village: true, ..Npr::default() }, ..TerrainAppearance::default() };
    let mut touched = 0usize;
    for (i, c) in CASES.iter().enumerate() {
        let got = render::apply_npr(&a, (c[0], c[1], c[2]), c[3], c[4], c[5], (c[6], c[7]), c[8], c[9], 64);
        let want = VILLAGE_EXPECTED[i];
        assert_eq!(got, want, "case {i}: got {got:?}, want {want:?}");
        // Every emitted channel is an exact multiple of the literal step
        // 51.0 -- the six-level claim, checked per channel rather than
        // trusted from the hard-coded triples above.
        const STEP: f64 = 51.0;
        for (label, ch) in [("r", got.0), ("g", got.1), ("b", got.2)] {
            let steps = ch / STEP;
            assert!(
                (steps - steps.round()).abs() < 1e-9,
                "case {i} channel {label}: {ch} is not a multiple of {STEP}"
            );
        }
        if (got.0 - c[0]).abs() > 1e-6 || (got.1 - c[1]).abs() > 1e-6 || (got.2 - c[2]).abs() > 1e-6 {
            touched += 1;
        }
    }
    // Non-emptiness: a quantiser that silently passed its input through
    // would satisfy "every channel is a multiple of 51" for none of these
    // inputs by accident, but would also change nothing -- catch that too.
    assert!(touched >= 4, "the village quantiser changed almost nothing -- {touched} of {} cases moved", CASES.len());
}

/// Mutation guard, in prose: this constant is asserted by
/// `village_quantises_to_six_levels_of_51` above with a **literal** step
/// (`51.0`), not `255.0 / BANDS` -- so mutating `BANDS` in `render.rs`
/// (5.0 -> anything else) moves the quantised output and fails that test,
/// rather than silently re-deriving the same wrong answer.
#[test]
fn non_village_presets_are_unmoved_by_the_band_count() {
    // Two "other preset" recipes from `render_workspace.gd`'s `STYLE_PRESETS`
    // -- Antique (`sepia: 0.35, multi_sun: true`) and Print
    // (`risograph: 0.5, contours: 0.25`) -- neither of which ever sets
    // `village`, so neither reaches `quantize_flat_palette` regardless of
    // `BANDS`. Pinned as exact literals: if a future change let `BANDS` leak
    // into the shared chain (e.g. by moving the village step earlier, or by
    // some other style reading it), these would move and this test would
    // catch it even though it names no village behaviour itself.
    let antique = TerrainAppearance { npr: Npr { sepia: 0.35, multi_sun: true, ..Npr::default() }, ..TerrainAppearance::default() };
    const ANTIQUE_EXPECTED: [(f64, f64, f64); 6] = [
        (197.437, 182.124, 153.2865),
        (69.60855, 74.0622, 50.48205),
        (232.25, 235.5, 223.042),
        (14.149350000000002, 15.4053, 10.904399999999999),
        (49.086325, 78.621345, 105.49722),
        (209.29245, 38.67464999999999, 113.3448),
    ];
    for (i, c) in CASES.iter().enumerate() {
        let got = render::apply_npr(&antique, (c[0], c[1], c[2]), c[3], c[4], c[5], (c[6], c[7]), c[8], c[9], 64);
        let want = ANTIQUE_EXPECTED[i];
        let tol = 1e-9;
        assert!(
            (got.0 - want.0).abs() < tol && (got.1 - want.1).abs() < tol && (got.2 - want.2).abs() < tol,
            "case {i}: got {got:?}, want {want:?}"
        );
    }
}
