//! tectonics, height formula, normalize, volcanism, world-structure archetypes
//!
//! Ported in pipeline order starting Phase 1 (MVP_SCOPE.md).

use cartalith_noise::{fbm, pfbm};

/// `computeWarp()` / `computeWarpPrep()` / `warpParams()` (reference HTML,
/// lines 2621-2735) — domain-warped fbm producing the per-cell (x, y)
/// displacement later stages sample through (e.g. `assignPlates`'s
/// `ax=warpX?x+warpX[i]:x`).
///
/// Returns `None` when `warp * 0.18 * gw < 0.5` — the JS engine's own
/// threshold below which it treats the warp field as absent (`warpX=null`)
/// rather than computing a negligible one. The JS version also caches
/// across calls (`_warpCacheSeed`/`_warpCacheAmt`); that's a perf
/// optimisation with no effect on output values, so it isn't reproduced
/// here — memoizing belongs to whichever orchestration layer calls this,
/// if it ever needs to.
///
/// Stores through `f32` (matching `Float32Array` in JS): every cell is
/// computed in `f64` and rounded to `f32` at the point of storage, exactly
/// where JS's own `Float32Array` assignment would round it — later stages
/// reading `warpX[i]` read that already-rounded value, not the full-`f64`
/// intermediate, so this rounding point matters for parity.
pub fn compute_warp(
    gw: usize,
    gh: usize,
    seed: i32,
    warp: f64,
    world: bool,
) -> Option<(Vec<f32>, Vec<f32>)> {
    let amp = warp * 0.18 * (gw as f64);
    if amp < 0.5 {
        return None;
    }
    let wf = if world { 3.0 / gw as f64 } else { 2.5 / gw as f64 };
    let p_x: i32 = 3;
    let n = gw * gh;
    let mut warp_x = vec![0f32; n];
    let mut warp_y = vec![0f32; n];
    for y in 0..gh {
        for x in 0..gw {
            let i = y * gw + x;
            let xf = x as f64 * wf;
            let yf = y as f64 * wf;
            let qx = if world {
                pfbm(xf, yf, seed + 17, p_x)
            } else {
                fbm(xf, yf, seed + 17)
            };
            let qy = if world {
                pfbm(xf, yf, seed + 101, p_x)
            } else {
                fbm(xf, yf, seed + 101)
            };
            let wx = if world {
                pfbm(xf + 4.0 * qx, yf + 4.0 * qy, seed + 213, p_x) - 0.5
            } else {
                fbm(xf + 4.0 * qx, yf + 4.0 * qy, seed + 213) - 0.5
            };
            let wy = if world {
                pfbm(xf + 4.0 * qx, yf + 4.0 * qy, seed + 331, p_x) - 0.5
            } else {
                fbm(xf + 4.0 * qx, yf + 4.0 * qy, seed + 331) - 0.5
            };
            warp_x[i] = (wx * 2.0 * amp) as f32;
            warp_y[i] = (wy * 2.0 * amp) as f32;
        }
    }
    Some((warp_x, warp_y))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crate_compiles_and_tests_run() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn none_below_threshold() {
        assert!(compute_warp(100, 100, 1, 0.0, false).is_none());
    }

    #[test]
    fn deterministic_for_same_input() {
        let a = compute_warp(6, 5, 42, 0.6, false);
        let b = compute_warp(6, 5, 42, 0.6, false);
        assert_eq!(a, b);
    }
}
