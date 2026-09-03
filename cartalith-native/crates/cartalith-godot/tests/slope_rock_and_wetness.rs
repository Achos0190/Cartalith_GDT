//! **The constants of the two reference viz stages ported 2026-09-03**
//! (`OUTSTANDING_WORK.md` §2.5: `rockSlope` refinement and wetness darkening).
//!
//! `appearance_tiers.rs`'s `every_tunable_is_load_bearing` already proves both
//! stages reach the image — it kills a mutant that makes either gate
//! unreachable. What a whole-image mutation battery does **not** kill, measured
//! rather than assumed:
//!
//! | mutant | whole-render verdict |
//! |---|---|
//! | `rock_slope` exponent `1.5 -> 1.0` | SURVIVED |
//! | wetness tilt `0.95`/`1.05` -> `1.0`/`1.0` | SURVIVED |
//!
//! That is this project's own recorded failure mode in a second shape: a test
//! that only asks "did anything move" pins the *presence* of a stage and none
//! of its numbers. So every assertion below is against a literal computed
//! **independently of the constant it pins** — `0.25^1.5` is exactly `0.125`,
//! and `0.5^1.5` is `0.3535…`, which is neither the linear `0.5` nor the
//! quadratic `0.25` a wrong exponent would give.
//!
//! Both stages are `0.0` in `default()` and in `js_reference()`; the shipped
//! and reference images are guarded by `color_space.rs`'s own
//! `FINISHED_RENDER_FNV1A` digest and by `golden_parity_render.rs`, and both
//! were confirmed unmoved by this port.

#[path = "../src/render.rs"]
mod render;

use render::{apply_wetness, rock_slope_mix};

/// Absolute tolerance for a value that is one `powf` away from an exact
/// decimal — tight enough that a different exponent or knee cannot hide.
const EPS: f64 = 1e-12;

/// Reference HTML 7789: `Math.pow(Math.min(1, slope/0.08), 1.5) * rsK`.
#[test]
fn the_slope_rock_ramp_is_the_reference_curve() {
    // A quarter of the way up the knee. `0.25^1.5 = 0.25 · sqrt(0.25) = 0.125`,
    // written out rather than as `0.25f64.powf(1.5)` so the assertion does not
    // restate the expression it is checking.
    assert!((rock_slope_mix(0.02, 1.0) - 0.125).abs() < EPS, "slope/0.08 = 0.25 must give 1/8, got {}", rock_slope_mix(0.02, 1.0));

    // Halfway. `0.5^1.5 = 1 / (2·sqrt(2))`. The point of this row is the two
    // values it is NOT: `0.5` (exponent 1) and `0.25` (exponent 2).
    let half = rock_slope_mix(0.04, 1.0);
    assert!((half - 1.0 / (2.0 * std::f64::consts::SQRT_2)).abs() < EPS, "slope/0.08 = 0.5 must give 1/(2√2), got {half}");
    assert!((half - 0.5).abs() > 0.14, "a linear ramp would pass every other row here");

    // The knee itself, and past it: `min(1, ·)` must saturate rather than run on.
    assert!((rock_slope_mix(0.08, 1.0) - 1.0).abs() < EPS, "slope = 0.08 is the top of the ramp");
    assert!((rock_slope_mix(4.0, 1.0) - 1.0).abs() < EPS, "the ramp must clamp, not exceed 1");

    // The slider scales the whole ramp linearly, and nothing else does.
    assert!((rock_slope_mix(0.02, 0.5) - 0.0625).abs() < EPS, "half strength must halve the mix");
    assert_eq!(rock_slope_mix(0.02, 0.0), 0.0, "zero strength must mix nothing");

    // **`min(1, ·)` is not redundant with the outer `clamp01`, and only a
    // partial strength shows it.** Ten times the knee at a tenth strength must
    // still be a tenth: without the inner saturation the ramp reaches `10^1.5`
    // and the outer clamp turns a subtle tint into full rock. This exact mutant
    // survived a battery that had only full-strength rows.
    assert!((rock_slope_mix(0.8, 0.1) - 0.1).abs() < EPS, "a cliff at 10% strength must take 10% rock, got {}", rock_slope_mix(0.8, 0.1));

    // Flat ground gets nothing at any strength -- the stage is *slope* rock.
    assert_eq!(rock_slope_mix(0.0, 1.0), 0.0, "flat ground must take no rock");
}

/// Reference HTML 7797-7798: `wv = clamp01((twi+1)/4)`, `dk = 1 - 0.30·wtK·wv`,
/// then `c0·dk·0.95`, `c1·dk`, `c2·dk·1.05`.
#[test]
fn wetness_darkens_and_cools_by_the_reference_constants() {
    // `twi = 1` puts `wv` at exactly `0.5`, so `dk = 1 - 0.30·0.5 = 0.85` at
    // full strength. Against a neutral 100 the three channels are 80.75 / 85 /
    // 89.25 — three independent decimals that between them pin `0.30`, the
    // `(twi+1)/4` calibration and both tilt factors.
    let (r, g, b) = apply_wetness((100.0, 100.0, 100.0), 1.0, 1.0);
    assert!((r - 80.75).abs() < 1e-10, "red: {r}");
    assert!((g - 85.00).abs() < 1e-10, "green: {g}");
    assert!((b - 89.25).abs() < 1e-10, "blue: {b}");

    // The tilt is what makes this *wet* rather than *dark*, and it is
    // independent of how dark: the blue:red ratio is `1.05/0.95` at every
    // depth. A flattened tilt (the mutant that survived the image battery)
    // makes this exactly 1.
    for twi in [0.0, 1.0, 2.5, 7.0] {
        let (r, _, b) = apply_wetness((100.0, 100.0, 100.0), twi, 1.0);
        assert!((b / r - 21.0 / 19.0).abs() < 1e-12, "twi {twi}: blue/red must be 1.05/0.95, got {}", b / r);
        assert!(b > r, "twi {twi}: wet ground must read cooler, not just darker");
    }

    // The top of the calibration: `twi = 3` saturates `wv` at 1, and beyond it
    // nothing more happens.
    let deep = apply_wetness((100.0, 100.0, 100.0), 3.0, 1.0);
    assert!((deep.1 - 70.0).abs() < 1e-10, "wv must saturate at 1, giving dk = 0.70; got {}", deep.1);
    assert_eq!(deep, apply_wetness((100.0, 100.0, 100.0), 40.0, 1.0), "past saturation the stage must be flat");

    // **The `wv > 0` guard.** At or below `twi = -1` the darken is nothing, and
    // the reference skips the block rather than applying an unopposed tilt. Without
    // the guard this returns (95, 100, 105) and the whole map cools by 5%.
    let dry = (123.0, 45.0, 200.0);
    assert_eq!(apply_wetness(dry, -1.0, 1.0), dry, "dry ground must be returned untouched, tilt included");
    assert_eq!(apply_wetness(dry, -9.0, 1.0), dry, "below the calibration floor likewise");

    // Zero strength is identity wherever the guard does not already make it so.
    assert_eq!(apply_wetness(dry, 3.0, 0.0), dry, "strength 0 must change nothing");
}
