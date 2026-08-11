//! tectonics, height formula, normalize, volcanism, world-structure archetypes
//!
//! Ported in pipeline order starting Phase 1 (MVP_SCOPE.md).

use cartalith_noise::{fbm, pfbm};
use cartalith_rng::Mulberry32;

/// Mirrors JS `Math.round`: ties round toward `+Infinity`, unlike Rust's
/// `f64::round` (ties away from zero) — `Math.round(-0.5) == 0`, but
/// `(-0.5_f64).round() == -1.0`. `buildPlates`'s world-wrap math depends
/// on the JS behaviour specifically (`cartalith-rust-conventions`: match
/// precision, don't improve it). `(x + 0.5).floor()` is the standard
/// exact equivalent.
fn js_round(x: f64) -> f64 {
    (x + 0.5).floor()
}

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

/// One tectonic plate: centre position, drift velocity, and crust "base"
/// height contribution (positive = continental, negative = oceanic).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Plate {
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub base: f64,
}

/// The world-structure continentality field and its reclassification
/// parameter, passed together since JS only reads either when both
/// `state.world_structure.enabled` is true AND the field itself exists
/// (`generateContinentalityField` leaves it `null` when disabled) — one
/// `Option` captures that combined condition.
pub struct WorldStructure<'a> {
    pub ocean_depth: f64,
    pub continental_field: &'a [f32],
}

/// `buildPlates()` (reference HTML, lines 2740-2766): seeds `n` plates at
/// random positions/velocities/crust type, then relaxes their positions
/// toward their own Voronoi-cell centroids for `lloyd_iters` iterations
/// (brute-force nearest-plate per cell — `assignPlates`'s JFA replaces
/// this cost for the *final* per-pixel assignment, but this relaxation
/// step is its own, separate O(iters × cells × plates) loop in the
/// original and is ported as such).
///
/// `world` wrapping uses a circular mean (`atan2` of summed sin/cos) for
/// the x coordinate rather than a plain average, so a plate whose cell
/// straddles the map seam doesn't get pulled toward the middle.
pub fn build_plates(
    gw: usize,
    gh: usize,
    seed: u32,
    n: usize,
    lloyd_iters: usize,
    world: bool,
    world_structure: Option<WorldStructure>,
) -> Vec<Plate> {
    let mut rng = Mulberry32::new(seed);
    let mut plates: Vec<Plate> = (0..n)
        .map(|_| {
            let oceanic = rng.next_f64() < 0.45;
            let x = rng.next_f64() * gw as f64;
            let y = rng.next_f64() * gh as f64;
            let vx = rng.next_f64() * 2.0 - 1.0;
            let vy = rng.next_f64() * 2.0 - 1.0;
            let sign = if oceanic { -1.0 } else { 1.0 };
            let base = sign * (0.55 + 0.45 * rng.next_f64());
            Plate { x, y, vx, vy, base }
        })
        .collect();

    for _ in 0..lloyd_iters {
        let mut sx = vec![0f64; n];
        let mut sy = vec![0f64; n];
        let mut c = vec![0f64; n];
        let mut sxs = vec![0f64; n];
        let mut sxc = vec![0f64; n];
        for y in 0..gh {
            for x in 0..gw {
                let mut best = 0usize;
                let mut bd = f64::INFINITY;
                for (p, plate) in plates.iter().enumerate() {
                    let mut dx = x as f64 - plate.x;
                    if world {
                        dx -= js_round(dx / gw as f64) * gw as f64;
                    }
                    let dy = y as f64 - plate.y;
                    let d = dx * dx + dy * dy;
                    if d < bd {
                        bd = d;
                        best = p;
                    }
                }
                if world {
                    let th = x as f64 / gw as f64 * std::f64::consts::PI * 2.0;
                    sxs[best] += th.sin();
                    sxc[best] += th.cos();
                } else {
                    sx[best] += x as f64;
                }
                sy[best] += y as f64;
                c[best] += 1.0;
            }
        }
        for p in 0..n {
            if c[p] != 0.0 {
                plates[p].x = if world {
                    (sxs[p].atan2(sxc[p]) / (std::f64::consts::PI * 2.0) + 1.0) * gw as f64
                        % gw as f64
                } else {
                    sx[p] / c[p]
                };
                plates[p].y = sy[p] / c[p];
            }
        }
    }

    if let Some(ws) = world_structure {
        let d_scale = 0.5 + ws.ocean_depth * 0.5;
        for plate in &mut plates {
            let xi = js_round(plate.x).clamp(0.0, gw as f64 - 1.0) as usize;
            let yi = js_round(plate.y).clamp(0.0, gh as f64 - 1.0) as usize;
            let cval = ws.continental_field[yi * gw + xi] as f64;
            plate.base = if cval >= 0.0 {
                0.6 + cval * 0.4
            } else {
                -(0.5 + cval.abs() * 0.5) * d_scale
            };
        }
    }

    plates
}

/// `assignPlates()` (reference HTML lines 2771-2810): Jump Flood Algorithm
/// Voronoi rasterization — O(N log N) instead of the O(N × plates)
/// brute-force nested loop `build_plates`'s own Lloyd step still uses.
/// Returns one plate index per cell (row-major, `y*gw+x`).
///
/// Two precision details matter for parity, not just the algorithm shape:
/// - `bestD2` is a `Float32Array` in JS, so the running best distance is
///   rounded to `f32` on every write, and later comparisons read that
///   rounded value back — an `f64` accumulator here would occasionally
///   pick a different winner on a near-tie. `best_d2: Vec<f32>` replicates
///   the rounding point exactly.
/// - The world-wrap distance correction (`ddx-Math.round(ddx/GW)*GW`)
///   needs `js_round`, same trap as `build_plates`.
pub fn assign_plates(
    gw: usize,
    gh: usize,
    world: bool,
    plates: &[Plate],
    warp_x: Option<&[f32]>,
    warp_y: Option<&[f32]>,
) -> Vec<usize> {
    let n = gw * gh;
    let np = plates.len();
    let px: Vec<f64> = plates.iter().map(|p| p.x).collect();
    let py: Vec<f64> = plates.iter().map(|p| p.y).collect();

    let mut nearest = vec![-1i32; n];
    let mut best_d2 = vec![1e30f32; n];

    for p in 0..np {
        // JS `PX[p]|0`: ToInt32 truncates toward zero, matching `as i32`.
        let cx = px[p] as i32;
        let cy = (py[p] as i32).clamp(0, gh as i32 - 1);
        // JS only bounds the upper side of wx_ (`cx<GW?cx:0`) — a
        // negative cx (never produced by this crate's own seeding, but
        // preserved for faithfulness) falls through to a negative index,
        // which a JS TypedArray write silently no-ops on. Mirror that: skip.
        if cx < 0 || cx >= gw as i32 {
            continue;
        }
        let wy = cy.clamp(0, gh as i32 - 1) as usize;
        let i = wy * gw + cx as usize;
        nearest[i] = p as i32;
        best_d2[i] = 0.0;
    }

    let max_dim = gw.max(gh) as f64;
    let max_step = 1u32 << (max_dim.log2().ceil() as u32);
    let mut step_u = max_step >> 1;
    while step_u >= 1 {
        let step = step_u as i64;
        for y in 0..gh {
            for x in 0..gw {
                let i = y * gw + x;
                let ax = x as f64 + warp_x.map_or(0.0, |w| w[i] as f64);
                let ay = y as f64 + warp_y.map_or(0.0, |w| w[i] as f64);
                // JS `for(dy=-step; dy<=step; dy+=step)` visits exactly
                // three offsets — -step, 0, +step — never a full range.
                for &dy in &[-step, 0, step] {
                    for &dx in &[-step, 0, step] {
                        if dx == 0 && dy == 0 {
                            continue;
                        }
                        let nx = x as i64 + dx;
                        let ny = y as i64 + dy;
                        let nx = if world {
                            ((nx % gw as i64) + gw as i64) % gw as i64
                        } else if nx < 0 || nx >= gw as i64 {
                            continue;
                        } else {
                            nx
                        };
                        if ny < 0 || ny >= gh as i64 {
                            continue;
                        }
                        let j = ny as usize * gw + nx as usize;
                        let p = nearest[j];
                        if p < 0 {
                            continue;
                        }
                        let p = p as usize;
                        let mut ddx = ax - px[p];
                        if world {
                            ddx -= js_round(ddx / gw as f64) * gw as f64;
                        }
                        let ddy = ay - py[p];
                        let d2 = ddx * ddx + ddy * ddy;
                        if d2 < best_d2[i] as f64 {
                            best_d2[i] = d2 as f32;
                            nearest[i] = p as i32;
                        }
                    }
                }
            }
        }
        if step_u == 1 {
            break;
        }
        step_u >>= 1;
    }

    let mut plate_id = vec![0usize; n];
    for i in 0..n {
        if nearest[i] >= 0 {
            plate_id[i] = nearest[i] as usize;
            continue;
        }
        let x = i % gw;
        let y = i / gw;
        let ax = x as f64 + warp_x.map_or(0.0, |w| w[i] as f64);
        let ay = y as f64 + warp_y.map_or(0.0, |w| w[i] as f64);
        let mut best = 0usize;
        let mut bd = f64::INFINITY;
        for p in 0..np {
            let mut dx = ax - px[p];
            if world {
                dx -= js_round(dx / gw as f64) * gw as f64;
            }
            let dy = ay - py[p];
            let d = dx * dx + dy * dy;
            if d < bd {
                bd = d;
                best = p;
            }
        }
        plate_id[i] = best;
    }
    plate_id
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
