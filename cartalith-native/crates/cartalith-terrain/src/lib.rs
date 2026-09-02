//! tectonics, height formula, normalize, volcanism, world-structure archetypes
//!
//! Ported in pipeline order starting Phase 1 (MVP_SCOPE.md).

use cartalith_noise::{fbm, pfbm, pridged, pvnoise, ridged, vnoise};
use cartalith_rng::Mulberry32;
use rayon::prelude::*;

pub mod amplify;
pub mod analysis;
pub mod center;
pub mod fjord;
pub mod infer;
pub mod landform;
pub mod sculpt;
pub mod tile_render;

// `Math.round`, `Math.sin`, `Math.cos` and `Math.atan2` with JS semantics.
//
// `js_round` was written here as `(x + 0.5).floor()` with a doc comment calling
// it "the standard exact equivalent"; it is not, and `JS_SEMANTICS_AUDIT.md`
// §3.1 measured the single input where it differs from V8
// (`0.49999999999999994`, where it gives 1 and `Math.round` gives 0). The
// comment was corrected in place at the time and the implementation left,
// because editing six crates under an active fork was the wrong trade. There
// is one implementation now, in `cartalith-jsmath`, and it is the
// fractional-part form that is right on that input too.
//
// `js_sin`/`js_cos`/`js_atan2` are `build_plates`' world-wrap circular mean --
// audit §4.4's `-terrain:372`, which it reported and did *not* change because
// `js_atan2` alone could not fix it. See that call site.
use cartalith_jsmath::{js_atan2, js_cos, js_round, js_sin};

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
    // Rows are independent (`warp_x[i]`/`warp_y[i]` depend only on `x, y`,
    // never another cell), so parallelizing across rows changes only which
    // core computes which row, not any value -- output is bit-for-bit
    // identical to the sequential version (CPU_MULTITHREADING_SCOPE.md).
    warp_x
        .par_chunks_mut(gw)
        .zip(warp_y.par_chunks_mut(gw))
        .enumerate()
        .for_each(|(y, (wx_row, wy_row))| {
            for x in 0..gw {
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
                wx_row[x] = (wx * 2.0 * amp) as f32;
                wy_row[x] = (wy * 2.0 * amp) as f32;
            }
        });
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

/// Bilinear sample on a coarse `ww`×`wh` grid, with optional x-wrap —
/// `bilC()` (reference HTML line 5537). Duplicated locally rather than
/// shared with `cartalith-climate`'s own private `bil_c` (same reasoning
/// as `js_round`'s per-crate duplication elsewhere in this port): one
/// line's worth of dependency isn't worth a shared crate for.
fn bil_c(a: &[f32], fx: f64, fy: f64, ww: usize, wh: usize, wrap_x: bool) -> f64 {
    let fx = if wrap_x {
        ((fx % ww as f64) + ww as f64) % ww as f64
    } else {
        fx.clamp(0.0, ww as f64 - 1.0)
    };
    let fy = fy.clamp(0.0, wh as f64 - 1.0);
    let x0 = fx as i64;
    let y0 = fy as i64;
    let x1 = if x0 + 1 >= ww as i64 {
        if wrap_x { 0 } else { ww as i64 - 1 }
    } else {
        x0 + 1
    };
    let y1 = (y0 + 1).min(wh as i64 - 1);
    let tx = fx - x0 as f64;
    let ty = fy - y0 as f64;
    let (x0, x1, y0, y1) = (x0 as usize, x1 as usize, y0 as usize, y1 as usize);
    let top = a[y0 * ww + x0] as f64 * (1.0 - tx) + a[y0 * ww + x1] as f64 * tx;
    let bot = a[y1 * ww + x0] as f64 * (1.0 - tx) + a[y1 * ww + x1] as f64 * tx;
    top * (1.0 - ty) + bot * ty
}

/// `generateContinentalityField()` (reference HTML lines 2556-2589) —
/// World-Structure's continentality/fragmentation archetype knobs turned
/// into the per-cell field `build_plates` reclassifies plate crust from.
/// Caller passes `world_structure.enabled` up front; this returns `None`
/// for that case rather than reproducing JS's `continentalField=null`
/// module-global convention (`build_plates`'s own `Option<WorldStructure>`
/// already models "disabled" the same way).
///
/// Runs on a coarse grid (`min(gw,240)` wide, matching every other
/// coarse-grid stage in this port) then bilinear-upsamples — a smooth
/// low-frequency field has no reason to pay full resolution. Percentile-
/// normalized via an `O(N)` histogram (not a full sort) so exactly
/// `continentality`'s fraction of coarse cells land above zero, then
/// rescaled to `[-1,1]` by the larger-magnitude extreme.
#[allow(clippy::too_many_arguments)]
pub fn generate_continentality_field(
    gw: usize,
    gh: usize,
    world: bool,
    seed: i32,
    continentality: f64,
    fragmentation: f64,
) -> Vec<f32> {
    let cfw = gw.min(240);
    let cfh = (js_round(cfw as f64 * gh as f64 / gw as f64) as usize).max(2);
    let n = cfw * cfh;
    let field_seed = seed ^ 0x00c0_ffee;
    let freq = 0.3 + fragmentation * 2.7;
    let p_x = (js_round(freq) as i32).max(2);

    let mut raw = vec![0f32; n];
    for y in 0..cfh {
        for x in 0..cfw {
            let nx = x as f64 / cfw as f64 * freq;
            let ny = y as f64 / cfh as f64 * freq;
            let mut vv = 0f64;
            let mut amp = 0.5f64;
            let mut fr = 1f64;
            let mut nrm = 0f64;
            for o in 0..3 {
                let sample = if world {
                    pvnoise(nx * fr, ny * fr, field_seed + o * 131, (p_x * (1 << o)).max(2))
                } else {
                    vnoise(nx * fr, ny * fr, field_seed + o * 131)
                };
                vv += amp * sample;
                nrm += amp;
                amp *= 0.5;
                fr *= 2.0;
            }
            raw[y * cfw + x] = (vv / nrm) as f32;
        }
    }

    const BINS: usize = 2000;
    let mut hist = vec![0i32; BINS];
    let mut hmin = f64::INFINITY;
    let mut hmax = f64::NEG_INFINITY;
    for &v in &raw {
        let v = v as f64;
        if v < hmin {
            hmin = v;
        }
        if v > hmax {
            hmax = v;
        }
    }
    let h_range = if hmax - hmin != 0.0 { hmax - hmin } else { 1.0 };
    for &v in &raw {
        let bin = (((v as f64 - hmin) / h_range * BINS as f64) as usize).min(BINS - 1);
        hist[bin] += 1;
    }
    let target = ((1.0 - continentality) * n as f64).floor() as i64;
    let mut cum_count = 0i64;
    let mut thresh = hmin;
    for (b, &count) in hist.iter().enumerate() {
        cum_count += count as i64;
        if cum_count >= target {
            thresh = hmin + b as f64 / BINS as f64 * h_range;
            break;
        }
    }

    let mut cmin = f64::INFINITY;
    let mut cmax = f64::NEG_INFINITY;
    for v in &mut raw {
        let shifted = (*v as f64 - thresh) as f32;
        *v = shifted;
        let v = shifted as f64;
        if v < cmin {
            cmin = v;
        }
        if v > cmax {
            cmax = v;
        }
    }
    let c_range = cmin.abs().max(cmax.abs()).max(1e-6);
    for v in &mut raw {
        *v = (*v as f64 / c_range) as f32;
    }

    let wrap_x = world;
    let mut continental_field = vec![0f32; gw * gh];
    for y in 0..gh {
        for x in 0..gw {
            let fx = x as f64 / (gw as f64 - 1.0) * (cfw as f64 - 1.0);
            let fy = y as f64 / (gh as f64 - 1.0) * (cfh as f64 - 1.0);
            continental_field[y * gw + x] = bil_c(&raw, fx, fy, cfw, cfh, wrap_x) as f32;
        }
    }
    continental_field
}

/// `applyWorldStructureSeaLevel()` (reference HTML lines 2603-2617): a
/// World-Structure archetype promises a land/ocean ratio via
/// `continentality`, but the height field's own actual distribution
/// (reshaped independently by that archetype's `tectonicEnergy`/
/// `oceanDepth`) won't generally cross zero at the fixed default sea
/// level — so this re-measures the ACTUAL generated field's histogram
/// and picks the threshold that yields exactly the promised land
/// fraction, the same percentile technique
/// `generate_continentality_field` already uses. Returns the new sea
/// level; caller applies it (this crate has no `state.seaLevel` to
/// mutate).
pub fn apply_world_structure_sea_level(field: &[f32], continentality: f64) -> f64 {
    let n = field.len();
    const BINS: usize = 2000;
    let mut hist = vec![0i32; BINS];
    let mut hmin = f64::INFINITY;
    let mut hmax = f64::NEG_INFINITY;
    for &f in field {
        let f = f as f64;
        if f < hmin {
            hmin = f;
        }
        if f > hmax {
            hmax = f;
        }
    }
    let h_range = if hmax - hmin != 0.0 { hmax - hmin } else { 1.0 };
    for &f in field {
        let bin = (((f as f64 - hmin) / h_range * BINS as f64) as usize).min(BINS - 1);
        hist[bin] += 1;
    }
    let target = ((1.0 - continentality) * n as f64).floor() as i64;
    let mut cum = 0i64;
    let mut thresh = hmin;
    for (b, &count) in hist.iter().enumerate() {
        cum += count as i64;
        if cum >= target {
            thresh = hmin + b as f64 / BINS as f64 * h_range;
            break;
        }
    }
    thresh.clamp(0.05, 0.95)
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
                    // `js_sin`/`js_cos`, not `f64::sin`/`f64::cos`. See the
                    // circular mean below -- the divergence enters HERE, in the
                    // accumulation, before `atan2` is ever called.
                    let th = x as f64 / gw as f64 * std::f64::consts::PI * 2.0;
                    sxs[best] += js_sin(th);
                    sxc[best] += js_cos(th);
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
                    // `JS_SEMANTICS_AUDIT.md` §4.4's `-terrain:372`, fixed.
                    //
                    // The audit reported this site and deliberately did **not**
                    // change it, because at the time only `js_atan2` existed:
                    // Rust's own `sin`/`cos` already produce a different
                    // `(sum sin, sum cos)` pair from V8's on 92 of 2 000
                    // synthetic plates, *before* `atan2` is reached, so
                    // swapping in `js_atan2` alone moved the result from
                    // 98/2000 disagreeing to 7/2000 -- an improvement that
                    // leaves the site differently wrong, which is worse than
                    // leaving it alone because the next reader would believe
                    // it had been handled. "Fix this in the same pass that
                    // lands `js_sin`/`js_cos`, not before, and fix all three
                    // together."
                    //
                    // This is that pass. All three are `cartalith-jsmath`'s
                    // now, and the accumulation above uses the other two.
                    //
                    // It matters because of `-terrain:347`: the next Lloyd
                    // iteration's `dx = x as f64 - plate.x` feeds a
                    // nearest-plate argmin, which is structurally the same
                    // discrete-decision hazard as the river receiver in §2.3,
                    // one iteration removed. (`-terrain:385` quantises through
                    // `js_round` and was measured safe either way.)
                    (js_atan2(sxs[p], sxc[p]) / (std::f64::consts::PI * 2.0) + 1.0) * gw as f64
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
/// `u16`, not `usize`: `tect.plates` is capped at 40 by its own `ParamSpec`
/// (`cartalith-godot`'s `params.rs`, clamped on every set) and the
/// World-Structure override clamps to `4..=40` as well, so eight bytes a cell
/// held a number below 41. 2 B/cell instead of 8 is 15.36 MiB off a
/// 2 048 × 1 311 world's peak *and* its resident set
/// (`MEMORY_OPTIMIZATION_SCOPE.md` R4). Callers index `plates[]` with
/// `as usize` at the read.
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
) -> Vec<u16> {
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

    let mut plate_id = vec![0u16; n];
    for i in 0..n {
        if nearest[i] >= 0 {
            plate_id[i] = nearest[i] as u16;
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
        plate_id[i] = best as u16;
    }
    plate_id
}

/// `boxH()` (reference HTML line 2511): horizontal sliding-window box
/// blur. The running sum (`acc`) stays `f64` throughout — matching JS,
/// where `acc` is a plain number even though it sums `Float32Array`
/// reads — and is only rounded to `f32` at the point of writing `dst`.
fn box_h(src: &[f32], dst: &mut [f32], w: usize, h: usize, r: i64, wrap: bool) {
    debug_assert_eq!(src.len(), w * h);
    debug_assert_eq!(dst.len(), w * h);
    let norm = 1.0 / (2.0 * r as f64 + 1.0);
    let wi = w as i64;
    // Each row's running sum only reads/writes within that row -- rows are
    // independent, so chunking `dst`/`src` by row and processing chunks in
    // parallel is exact, not approximate (CPU_MULTITHREADING_SCOPE.md).
    dst.par_chunks_mut(w)
        .zip(src.par_chunks(w))
        .for_each(|(dst_row, src_row)| {
            let mut acc = 0.0f64;
            if wrap {
                for k in -r..=r {
                    let idx = (((k % wi) + wi) % wi) as usize;
                    acc += src_row[idx] as f64;
                }
            } else {
                for k in -r..=r {
                    let idx = k.clamp(0, wi - 1) as usize;
                    acc += src_row[idx] as f64;
                }
            }
            // `x` also drives the running-sum update below, not just the
            // `dst_row` index -- an iterator/enumerate rewrite wouldn't be
            // clearer here, same reasoning as this crate's other running-
            // sum/sliding-window loops.
            #[allow(clippy::needless_range_loop)]
            for x in 0..w {
                dst_row[x] = (acc * norm) as f32;
                let xi = x as i64;
                if wrap {
                    let o = (((xi - r) % wi) + wi) % wi;
                    let i = (((xi + r + 1) % wi) + wi) % wi;
                    acc += src_row[i as usize] as f64 - src_row[o as usize] as f64;
                } else {
                    let o = (xi - r).clamp(0, wi - 1) as usize;
                    let i = (xi + r + 1).clamp(0, wi - 1) as usize;
                    acc += src_row[i] as f64 - src_row[o] as f64;
                }
            }
        });
}

/// `boxV()` (reference HTML line 2512): vertical sliding-window box blur.
/// Always clamps at the top/bottom edge — maps don't wrap pole-to-pole,
/// only (optionally) east-west, so unlike `box_h` there's no wrap variant.
fn box_v(src: &[f32], dst: &mut [f32], w: usize, h: usize, r: i64) {
    let norm = 1.0 / (2.0 * r as f64 + 1.0);
    let hi = h as i64;
    // Columns are independent, but `dst`'s row-major layout makes a single
    // column a strided (non-contiguous) write target -- rather than reach
    // for unsafe to split disjoint strided slices, compute each column
    // (contiguous in this column-major scratch buffer) in parallel, then
    // scatter into `dst` in one cheap sequential pass. The scatter is
    // O(w*h) and memory-bound, negligible next to the O(w*h*(2r+1))-shaped
    // work it replaces (CPU_MULTITHREADING_SCOPE.md).
    let mut cols = vec![0f32; w * h];
    cols.par_chunks_mut(h).enumerate().for_each(|(x, col)| {
        let mut acc = 0.0f64;
        for k in -r..=r {
            let idx = k.clamp(0, hi - 1) as usize;
            acc += src[idx * w + x] as f64;
        }
        #[allow(clippy::needless_range_loop)] // y also drives the running-sum update below
        for y in 0..h {
            col[y] = (acc * norm) as f32;
            let yi = y as i64;
            let o = (yi - r).clamp(0, hi - 1) as usize;
            let i = (yi + r + 1).clamp(0, hi - 1) as usize;
            acc += src[i * w + x] as f64 - src[o * w + x] as f64;
        }
    });
    for x in 0..w {
        let col = &cols[x * h..x * h + h];
        for y in 0..h {
            dst[y * w + x] = col[y];
        }
    }
}

/// `gaussBlur()` (reference HTML line 2513), CPU path only — the GPU path
/// is unavailable headless, and JS itself falls back to exactly this code
/// when it is, so parity only needs this branch. Three box-blur passes
/// (H then V each time) approximate a Gaussian, alternating between two
/// buffers exactly as JS does (`boxH(a,b,...); boxV(b,a,...)`, three
/// times, `a` holds the result).
///
/// `r<1` returns an unmodified copy — a real, observable early-exit, not
/// an optimization to skip.
pub fn gauss_blur(src: &[f32], r: f64, w: usize, h: usize, wrap_x: bool) -> Vec<f32> {
    if r < 1.0 {
        return src.to_vec();
    }
    let pr = js_round(r / 1.6).max(1.0) as i64;
    let mut a = src.to_vec();
    let mut b = vec![0f32; src.len()];
    for _ in 0..3 {
        box_h(&a, &mut b, w, h, pr, wrap_x);
        box_v(&b, &mut a, w, h, pr);
    }
    a
}

/// Boundary type codes matching JS's `BTYPE` object exactly (reference
/// HTML line 2816) — used as plain `u8` rather than a Rust enum since
/// `boundaryType` is a `Uint8Array` in the original and later stages
/// (orogeny, rendering) index color/behavior tables by this number
/// directly.
pub mod btype {
    pub const NONE: u8 = 0;
    pub const COLLISION: u8 = 1;
    pub const SUBDUCTION_OC: u8 = 2;
    pub const ARC_OO: u8 = 3;
    pub const RIFT: u8 = 4;
    pub const TRANSFORM: u8 = 5;
}

/// `classifyBoundary()` (reference HTML line 2818): shear-dominant pairs
/// are transforms regardless of crust type; otherwise convergence splits
/// by ocean/continent combination, divergence is a rift.
pub(crate) fn classify_boundary(ocean_a: bool, ocean_b: bool, c: f64, s: f64) -> u8 {
    if s.abs() > 1.5 * c.abs() {
        return btype::TRANSFORM;
    }
    if c > 0.0 {
        if ocean_a && ocean_b {
            btype::ARC_OO
        } else if ocean_a != ocean_b {
            btype::SUBDUCTION_OC
        } else {
            btype::COLLISION
        }
    } else {
        btype::RIFT
    }
}

/// Output of `compute_stress` — `boundaryMask`/`boundaryType`/
/// `stressField`/`shearField` bundled together since JS computes all four
/// in the same pass.
pub struct StressResult {
    pub boundary_mask: Vec<u8>,
    pub boundary_type: Vec<u8>,
    pub stress_field: Vec<f32>,
    pub shear_field: Vec<f32>,
}

/// `computeStress()` (reference HTML lines 2819-2848): walks each cell's
/// right/down neighbors (plus, under world-wrap, the row-wrap neighbor at
/// the right edge — a cell never checks left/up, since each boundary is
/// still marked on *both* sides when found from one direction), and where
/// two different plates meet, accumulates convergence (`C`) and shear
/// (`S`) stress from the plates' relative velocity projected onto the
/// boundary normal/tangent.
///
/// `raw`/`rawS`/`domMag` are `Float32Array` in JS, and critically `+=`
/// writes directly into them — each accumulation step rounds to `f32`
/// immediately, not once at the end like `gauss_blur`'s internal `f64`
/// accumulator. A cell touched by multiple boundary edges genuinely
/// accumulates with per-step rounding, so these fields use `f32` math
/// throughout, not an `f64` running sum cast down afterward.
pub fn compute_stress(
    gw: usize,
    gh: usize,
    world: bool,
    plate_id: &[u16],
    plates: &[Plate],
    vel: f64,
    blur_r: f64,
) -> StressResult {
    let n = gw * gh;
    let mut raw = vec![0f32; n];
    let mut raw_s = vec![0f32; n];
    let mut dom_mag = vec![0f32; n];
    let mut boundary_mask = vec![0u8; n];
    let mut boundary_type = vec![0u8; n];

    for y in 0..gh {
        for x in 0..gw {
            let i = y * gw + x;
            let a = plate_id[i] as usize;
            let mut neighbors: Vec<usize> = Vec::with_capacity(3);
            if x + 1 < gw {
                neighbors.push(i + 1);
            }
            if y + 1 < gh {
                neighbors.push(i + gw);
            }
            if world && x == gw - 1 {
                neighbors.push(y * gw);
            }
            for j in neighbors {
                let b = plate_id[j] as usize;
                if b == a {
                    continue;
                }
                boundary_mask[i] = 1;
                boundary_mask[j] = 1;
                let pa = plates[a];
                let pb = plates[b];
                let mut nx = pb.x - pa.x;
                let mut ny = pb.y - pa.y;
                let nl = nx.hypot(ny);
                let nl = if nl == 0.0 || nl.is_nan() { 1.0 } else { nl };
                nx /= nl;
                ny /= nl;
                let tx = -ny;
                let ty = nx;
                let c = ((pa.vx - pb.vx) * nx + (pa.vy - pb.vy) * ny) * vel;
                let s = ((pa.vx - pb.vx) * tx + (pa.vy - pb.vy) * ty) * vel;
                // `raw[i] as f64 + c` (not `raw[i] + c as f32`): JS adds
                // the full f64 C to the f64-promoted array read and
                // rounds once — truncating c to f32 first would
                // double-round and can disagree with JS on the ULP.
                raw[i] = (raw[i] as f64 + c) as f32;
                raw[j] = (raw[j] as f64 + c) as f32;
                raw_s[i] = (raw_s[i] as f64 + s) as f32;
                raw_s[j] = (raw_s[j] as f64 + s) as f32;
                let mag = c.abs() + s.abs();
                let bt = classify_boundary(pa.base < 0.0, pb.base < 0.0, c, s);
                if mag >= dom_mag[i] as f64 {
                    dom_mag[i] = mag as f32;
                    boundary_type[i] = bt;
                }
                if mag >= dom_mag[j] as f64 {
                    dom_mag[j] = mag as f32;
                    boundary_type[j] = bt;
                }
            }
        }
    }

    // mx/ms are plain (f64) JS variables, not typed-array elements —
    // even though every value feeding them comes from an f32 read, the
    // division below happens in f64 before rounding back to f32 on
    // store, so mx/ms themselves must stay f64 throughout.
    let mut stress_field = gauss_blur(&raw, blur_r, gw, gh, world);
    let mut mx = 1e-6f64;
    for &v in &stress_field {
        let v = (v as f64).abs();
        if v > mx {
            mx = v;
        }
    }
    for v in &mut stress_field {
        *v = (*v as f64 / mx) as f32;
    }

    let mut shear_field = gauss_blur(&raw_s, blur_r, gw, gh, world);
    let mut ms = 1e-6f64;
    for &v in &shear_field {
        let v = (v as f64).abs();
        if v > ms {
            ms = v;
        }
    }
    for v in &mut shear_field {
        *v = (*v as f64 / ms) as f32;
    }

    StressResult {
        boundary_mask,
        boundary_type,
        stress_field,
        shear_field,
    }
}

/// `computeFlexure()` (reference HTML lines 3105-3111): seeds a field
/// from `stressField` only at boundary cells, blurs it wide (3x the
/// normal blur radius — flexural wavelength is much longer than the
/// stress wavelength that produced it), then normalizes by max
/// magnitude.
pub fn compute_flexure(
    gw: usize,
    gh: usize,
    boundary_mask: &[u8],
    stress_field: &[f32],
    blur_r: f64,
    world: bool,
) -> Vec<f32> {
    let n = gw * gh;
    let mut raw = vec![0f32; n];
    for i in 0..n {
        if boundary_mask[i] != 0 {
            raw[i] = stress_field[i];
        }
    }
    let broad = gauss_blur(&raw, blur_r * 3.0, gw, gh, world);
    let mut mx = 1e-6f64;
    for &v in &broad {
        let v = (v as f64).abs();
        if v > mx {
            mx = v;
        }
    }
    broad.iter().map(|&v| (v as f64 / mx) as f32).collect()
}

/// `distanceToBoundary()` + the `ageField` normalization step that always
/// immediately follows it in `buildTectonicSubstrate` (reference HTML
/// lines 2860-2879 for the transform, 2779-2783 for the normalize) —
/// bundled into one function since nothing else in the pipeline reads the
/// raw chamfer distances, only the normalized `[0,1]` age.
///
/// Two-pass chamfer distance transform (forward pass top-left→bottom-right,
/// backward pass bottom-right→top-left) from every boundary cell — `1.4142`
/// is a literal diagonal-step cost in the original, not `2f64.sqrt()`
/// (`1.4142135...`), and must match exactly, not just approximately.
/// World-wrap is **not** applied here, matching the original (a margin
/// distance that wraps around the seam is a documented gap in the JS
/// source itself, not something this port introduces).
pub fn build_age_field(gw: usize, gh: usize, boundary_mask: &[u8]) -> Vec<f32> {
    const INF: f64 = 1e9;
    // Deliberately the literal `1.4142`, not `SQRT_2` (1.41421356...) --
    // matches the JS source's own diagonal-step constant exactly, not an
    // "improved" more-precise approximation of root 2.
    #[allow(clippy::approx_constant)]
    const D2: f64 = 1.4142;
    let n = gw * gh;
    let mut d = vec![0f32; n];
    for i in 0..n {
        d[i] = if boundary_mask[i] != 0 { 0.0 } else { INF as f32 };
    }
    // `v` stays f64 for a whole cell's chain of `Math.min` comparisons,
    // exactly as JS's plain (non-typed-array) `v` does — every `d[...]`
    // read auto-promotes to f64, and only the final `d[idx]=v` rounds
    // back to f32 once. Rounding after each `.min()` step instead
    // (an all-f32 accumulator) diverges by up to 1 ULP on cells reached
    // via a longer diagonal chain, since the two aren't equivalent once
    // more than one step separates a cell from its nearest boundary.
    for y in 0..gh {
        let row = y * gw;
        for x in 0..gw {
            let idx = row + x;
            let mut v = d[idx] as f64;
            if x > 0 {
                v = v.min(d[idx - 1] as f64 + 1.0);
            }
            if y > 0 {
                v = v.min(d[idx - gw] as f64 + 1.0);
            }
            if x > 0 && y > 0 {
                v = v.min(d[idx - gw - 1] as f64 + D2);
            }
            if x < gw - 1 && y > 0 {
                v = v.min(d[idx - gw + 1] as f64 + D2);
            }
            d[idx] = v as f32;
        }
    }
    for y in (0..gh).rev() {
        for x in (0..gw).rev() {
            let idx = y * gw + x;
            let mut v = d[idx] as f64;
            if x < gw - 1 {
                v = v.min(d[idx + 1] as f64 + 1.0);
            }
            if y < gh - 1 {
                v = v.min(d[idx + gw] as f64 + 1.0);
            }
            if x < gw - 1 && y < gh - 1 {
                v = v.min(d[idx + gw + 1] as f64 + D2);
            }
            if x > 0 && y < gh - 1 {
                v = v.min(d[idx + gw - 1] as f64 + D2);
            }
            d[idx] = v as f32;
        }
    }
    let mut maxd = 1e-6f64;
    for &v in &d {
        let v = v as f64;
        if v < 1e8 && v > maxd {
            maxd = v;
        }
    }
    d.iter().map(|&v| ((v as f64 / maxd).min(1.0)) as f32).collect()
}

/// `REF_CELLKM`/`TERRAIN_DETAIL_MAX_K`/`terrainDetailK()` (reference HTML,
/// near line 2636): raises relief-noise frequency once the map's real
/// cell size drops below the app's own literal default (800km / 2048px).
/// A no-op (returns 1) at or above that reference — only genuinely finer
/// configurations ease, capped at `TERRAIN_DETAIL_MAX_K`.
pub fn terrain_detail_k(gw: usize, map_width_km: f64) -> f64 {
    const REF_CELLKM: f64 = 800.0 / 2048.0;
    const TERRAIN_DETAIL_MAX_K: f64 = 16.0;
    let mwk = if map_width_km > 0.0 { map_width_km } else { 800.0 };
    let cell_km = mwk / gw as f64;
    (REF_CELLKM / cell_km).clamp(1.0, TERRAIN_DETAIL_MAX_K)
}

/// `riverCoarseEase()` (reference HTML, near line 2672) —
/// `terrainDetailK`'s sibling on the size axis rather than the
/// resolution axis: eases the channel-initiation threshold for a
/// region/world *larger* than the app's own 800km default (finer
/// relief at small scale made drainage measurably sparser without this;
/// `docs/research/scale-invariant-terrain.md`). No-op at/below 800km,
/// same `TERRAIN_DETAIL_MAX_K` cap as `terrain_detail_k`, deliberately
/// never blended with grid width — a first cut that did regressed this
/// file's own low-resolution test battery, per the reference's own
/// comment.
pub fn river_coarse_ease(map_width_km: f64) -> f64 {
    const TERRAIN_DETAIL_MAX_K: f64 = 16.0;
    let mwk = if map_width_km > 0.0 { map_width_km } else { 800.0 };
    (mwk / 800.0).clamp(1.0, TERRAIN_DETAIL_MAX_K)
}

/// `computeHeterogeneity()` + `fillHeteroRows()` + `heteroParams()`
/// (reference HTML lines 3117-3125): low-frequency noise modulated by
/// tectonic age — old stable cratons show more internal diversity than
/// young near-boundary crust.
// JS groups seed/hf/world into a params object only because fillHeteroRows
// is shared between the sync and Web-Worker-pool paths; this port has no
// worker pool to share with, so a bespoke struct here would exist solely
// to satisfy this lint, not to serve a second caller.
#[allow(clippy::too_many_arguments)]
pub fn compute_heterogeneity(
    gw: usize,
    gh: usize,
    seed: i32,
    map_width_km: f64,
    world: bool,
    age: &[f32],
    warp_x: Option<&[f32]>,
    warp_y: Option<&[f32]>,
) -> Vec<f32> {
    let n = gw * gh;
    let hetero_seed = seed ^ 0x44bb;
    let hf = 1.5 * terrain_detail_k(gw, map_width_km);
    let oct = (js_round(hf).max(2.0)) as i32;
    let mut out = vec![0f32; n];
    // Per-row parallel: `out[i]` depends only on `x, y, age[i]` and the
    // (read-only) warp fields, never another cell -- exact, not
    // approximate (CPU_MULTITHREADING_SCOPE.md). The max-finding/rescale
    // passes below stay sequential: a single O(n) scan is not the
    // bottleneck here (the fbm calls above are), and max is the only
    // reduction, so parallelizing it would add risk for no measurable gain.
    out.par_chunks_mut(gw).enumerate().for_each(|(y, row)| {
        #[allow(clippy::needless_range_loop)] // x also drives i, wx, wy below
        for x in 0..gw {
            let i = y * gw + x;
            let wx = x as f64 + warp_x.map_or(0.0, |w| w[i] as f64);
            let wy = y as f64 + warp_y.map_or(0.0, |w| w[i] as f64);
            let low_n = if world {
                pfbm(wx * hf / gw as f64, wy * hf / gw as f64, hetero_seed, oct)
            } else {
                fbm(wx * hf / gw as f64, wy * hf / gw as f64, hetero_seed)
            } - 0.5;
            row[x] = (low_n * (0.3 + 0.7 * age[i] as f64)) as f32;
        }
    });
    let mut mx = 1e-6f64;
    for &v in &out {
        let v = (v as f64).abs();
        if v > mx {
            mx = v;
        }
    }
    for v in &mut out {
        *v = (*v as f64 / mx) as f32;
    }
    out
}

/// `computeResistance()` (reference HTML lines 3132-3139): erosion
/// resistance from crust type (continental base = harder) and tectonic
/// age (older = more resistant). Used later to spatially modulate
/// stream-power erodibility.
pub fn compute_resistance(
    gw: usize,
    gh: usize,
    plate_id: &[u16],
    plates: &[Plate],
    age_field: &[f32],
) -> Vec<f32> {
    let n = gw * gh;
    let mut resistance = vec![0f32; n];
    // `resistance[i] = f(plate_id[i], age_field[i])` -- no cross-cell
    // dependency, exact under parallel execution (CPU_MULTITHREADING_SCOPE.md).
    resistance.par_iter_mut().enumerate().for_each(|(i, r)| {
        let pl = plates[plate_id[i] as usize];
        let crustal = pl.base.max(0.0);
        *r = (crustal * 0.6 + age_field[i] as f64 * 0.4).min(1.0) as f32;
    });
    resistance
}

/// The tectonic "formula weights" `heightParams()` bundles in JS
/// (`state.tect.alpha`/`.beta`/`.age`/`.flexure`/`.hetero`/`.ridged`) —
/// grouped here because they're a real conceptual unit (the height
/// formula's own tuning knobs), unlike `compute_heterogeneity`'s params,
/// which JS only bundles to share code with a worker pool this port
/// doesn't have.
pub struct HeightParams {
    pub nf: f64,
    pub seed: i32,
    pub a: f64,
    pub b: f64,
    pub age_inf: f64,
    pub fwt: f64,
    pub hwt: f64,
    pub world: bool,
    pub ridged: bool,
}

/// `fillHeightRows()` / `heightParams()` (reference HTML lines 2335-2344)
/// — **the height formula**: tectonic base + boundary stress/orogeny +
/// flexure + heterogeneity + ridged/value-noise roughness, damped by
/// tectonic age. `MVP_SCOPE.md` point 2: reproduce exactly, do not
/// improve (`DECISIONS.md` §7) — this function is a literal transcription
/// of the JS formula's term order and weights, not a reformulation.
#[allow(clippy::too_many_arguments)]
pub fn compute_height(
    gw: usize,
    gh: usize,
    base_field: &[f32],
    stress: &[f32],
    flex: &[f32],
    hetero: &[f32],
    age: &[f32],
    warp_x: Option<&[f32]>,
    warp_y: Option<&[f32]>,
    oro: Option<&[f32]>,
    p: &HeightParams,
) -> Vec<f32> {
    let n = gw * gh;
    let mut field = vec![0f32; n];
    // `field[i]` reads only cell `i`'s own inputs across every field
    // parameter here -- no cross-cell dependency, exact under parallel
    // execution (CPU_MULTITHREADING_SCOPE.md). This is the most
    // fbm/pridged-heavy per-cell loop in the crate, so the real timing win
    // is expected to come mostly from here.
    field.par_chunks_mut(gw).enumerate().for_each(|(y, row)| {
        #[allow(clippy::needless_range_loop)] // x also drives i, wx, wy below
        for x in 0..gw {
            let i = y * gw + x;
            let sf = stress[i] as f64;
            let t = match oro {
                Some(o) => o[i] as f64 + sf.min(0.0),
                None => sf,
            };
            let bs = base_field[i] as f64;
            let rug = (-(age[i] as f64) * (1.0 + p.age_inf * 6.0)).exp();
            let wx = x as f64 + warp_x.map_or(0.0, |w| w[i] as f64);
            let wy = y as f64 + warp_y.map_or(0.0, |w| w[i] as f64);
            let nx = wx * p.nf / gw as f64;
            let ny = wy * p.nf / gw as f64;
            let n_val = (if p.world {
                if p.ridged {
                    pridged(nx, ny, p.seed, 5)
                } else {
                    pfbm(nx, ny, p.seed, 5)
                }
            } else if p.ridged {
                ridged(nx, ny, p.seed)
            } else {
                fbm(nx, ny, p.seed)
            }) - 0.5;
            row[x] = (0.5
                + p.a * (0.40 * bs + 0.50 * t)
                + p.fwt * flex[i] as f64
                + p.hwt * hetero[i] as f64
                + p.b * n_val * (0.25 + 0.75 * rug)) as f32;
        }
    });
    field
}

/// `normalize()` (reference HTML lines 4930-4935), CPU path only — the
/// GPU path is unavailable headless, and JS itself falls back to this
/// exact code when it is. Min-max stretch to `[0,1]` (`MVP_SCOPE.md`
/// point 3); `mx-mn||1` guards a flat field (JS's `||` treats `0` as
/// falsy, so a zero range falls back to dividing by `1`, not `NaN`).
pub fn normalize_field(field: &[f32]) -> Vec<f32> {
    let mut mn = f64::INFINITY;
    let mut mx = f64::NEG_INFINITY;
    for &v in field {
        let v = v as f64;
        if v < mn {
            mn = v;
        }
        if v > mx {
            mx = v;
        }
    }
    let range = mx - mn;
    let range = if range == 0.0 { 1.0 } else { range };
    field.iter().map(|&v| ((v as f64 - mn) / range) as f32).collect()
}

/// `FEATURE_RADIUS_MAX_FRAC`/`clampFeatureRadiusCells()` (reference HTML,
/// near line 3485): a single volcano or crater can't exceed ~12% of the
/// shorter grid axis, however large its real-km radius would compute to.
const FEATURE_RADIUS_MAX_FRAC: f64 = 0.12;

fn clamp_feature_radius_cells(rad_cells: f64, gw: usize, gh: usize) -> f64 {
    rad_cells.min(gw.min(gh) as f64 * FEATURE_RADIUS_MAX_FRAC)
}

/// Adds `delta` to `field[i]` and rounds to `f32` immediately — mirrors a
/// single JS `field[i]+=delta` on a `Float32Array`. `stampOneCrater` calls
/// this multiple times per cell (crater bowl, rim, basin ringing), and
/// each is its own read-modify-write-with-rounding step in JS, not one
/// `f64` accumulation rounded once at the end — see the doc comment on
/// `stamp_one_crater` for why that distinction is load-bearing here.
fn add_rounded(field: &mut [f32], i: usize, delta: f64) {
    field[i] = (field[i] as f64 + delta) as f32;
}

/// `stampOneVolcano()` (reference HTML lines 3466-3473): radial cone with
/// an optional caldera dip (a summit depression once the volcano is tall
/// enough), and an age-damped volcanic-field mark for later biome/texture
/// use.
///
/// The `field[i]+=add; if(field[i]>1)... else if(field[i]<0)...` sequence
/// needs its rounding-then-clamp order preserved exactly: JS rounds the
/// sum to `f32` *first* (the real `Float32Array` store), then clamps
/// based on that *rounded* value. Clamping the pre-rounding `f64` sum
/// instead (the natural-looking Rust translation) can disagree right at
/// the boundary — a sum just under `1.0` in `f64` can round *up* past
/// `1.0` in `f32`, or vice versa, and only checking post-rounding catches
/// the case JS actually clamps.
#[allow(clippy::too_many_arguments)]
fn stamp_one_volcano(
    gw: usize,
    gh: usize,
    field: &mut [f32],
    volcanic_field: &mut [f32],
    peak_m: f64,
    cx: f64,
    cy: f64,
    rad_cells: f64,
    height_m: f64,
    age: f64,
) {
    let h = (height_m / peak_m) * 0.9 * (1.0 - age * 0.5);
    let r = rad_cells.max(2.0);
    let caldera = height_m > 1000.0;
    let x0 = (((cx - r) as i64).max(0)) as usize;
    let x1 = (((cx + r) as i64).min(gw as i64 - 1)).max(0) as usize;
    let y0 = (((cy - r) as i64).max(0)) as usize;
    let y1 = (((cy + r) as i64).min(gh as i64 - 1)).max(0) as usize;
    for y in y0..=y1 {
        for x in x0..=x1 {
            let dx = x as f64 - cx;
            let dy = y as f64 - cy;
            let d = dx.hypot(dy);
            if d > r {
                continue;
            }
            let t = d / r;
            let mut add = h * (1.0 - t).powf(1.6 - age * 0.8);
            if caldera && t < 0.16 {
                add -= h * 0.5 * (1.0 - t / 0.16);
            }
            let i = y * gw + x;
            add_rounded(field, i, add);
            let stored = field[i] as f64;
            field[i] = if stored > 1.0 {
                1.0
            } else if stored < 0.0 {
                0.0
            } else {
                field[i]
            };
            let candidate = (1.0 - t) * (1.0 - age);
            let vv = candidate.max(volcanic_field[i] as f64);
            volcanic_field[i] = vv as f32;
        }
    }
}

/// `placeSizedVolcano()` (reference HTML lines 3487-3495): picks a
/// power-law size class (70% small cinder cones, 25% stratovolcanoes, 5%
/// large shields) and stamps one. Returns `None` (JS: `null`) when `x, y`
/// falls outside the grid — checked *before* any RNG draw, so an
/// out-of-bounds placement consumes zero random numbers, not a partial
/// draw; get this ordering wrong and every subsequent placement in the
/// caller's loop silently uses the wrong random stream.
#[allow(clippy::too_many_arguments)]
fn place_sized_volcano(
    gw: usize,
    gh: usize,
    field: &mut [f32],
    volcanic_field: &mut [f32],
    map_width_km: f64,
    peak_m: f64,
    x: f64,
    y: f64,
    rng: &mut Mulberry32,
    age: f64,
) -> Option<(f64, f64)> {
    if x < 0.0 || y < 0.0 || x >= gw as f64 - 1.0 || y >= gh as f64 - 1.0 {
        return None;
    }
    let cell_km = map_width_km / gw as f64;
    let r = rng.next_f64();
    let mut h_m;
    let rad_km;
    if r < 0.70 {
        h_m = 200.0 + rng.next_f64() * 800.0;
        rad_km = 2.0 + rng.next_f64() * 8.0;
    } else if r < 0.95 {
        h_m = 1000.0 + rng.next_f64() * 2000.0;
        rad_km = 10.0 + rng.next_f64() * 20.0;
    } else {
        h_m = 3000.0 + rng.next_f64() * 4000.0;
        rad_km = 30.0 + rng.next_f64() * 50.0;
    }
    h_m = h_m.min(peak_m * 0.95);
    stamp_one_volcano(
        gw,
        gh,
        field,
        volcanic_field,
        peak_m,
        x,
        y,
        clamp_feature_radius_cells(rad_km / cell_km, gw, gh),
        h_m,
        age,
    );
    Some((x, y))
}

/// `stampVolcanoesSimple()` (reference HTML lines 3497-3505): dusts
/// volcanoes along plate boundaries (80% of the time, when any exist) or
/// at random, jittered by up to 6 cells.
///
/// **Not the JS default** — `state.volc.provinces` defaults to `true`,
/// which routes through `stamp_volcanoes_provinces` (below; clustered
/// arc/rift/hotspot chains) instead. This is the shared foundation
/// (`stampOneVolcano`/`placeSizedVolcano`) both modes build on, and stays
/// reachable as the `provinces: false` path (`cartalith-engine`'s own
/// `VolcanismParams::provinces`).
#[allow(clippy::too_many_arguments)]
pub fn stamp_volcanoes_simple(
    gw: usize,
    gh: usize,
    seed: u32,
    map_width_km: f64,
    peak_m: f64,
    boundary_mask: &[u8],
    volc_count: i32,
    volc_age: f64,
    field: &mut [f32],
    volcanic_field: &mut [f32],
) {
    let mut rng = Mulberry32::new(seed ^ 0x5bf03635);
    let bc: Vec<usize> = (0..boundary_mask.len())
        .filter(|&i| boundary_mask[i] != 0)
        .collect();
    let mut placed = 0;
    let mut guard = 0;
    while placed < volc_count && guard < volc_count * 40 {
        guard += 1;
        let (mut cx, cy);
        if !bc.is_empty() && rng.next_f64() < 0.8 {
            let idx = bc[(rng.next_f64() * bc.len() as f64) as usize];
            cx = (idx % gw) as f64;
            cy = (idx / gw) as f64;
        } else {
            cx = (rng.next_f64() * gw as f64).floor();
            cy = (rng.next_f64() * gh as f64).floor();
        }
        cx += (rng.next_f64() * 2.0 - 1.0) * 6.0;
        let cy = cy + (rng.next_f64() * 2.0 - 1.0) * 6.0;
        // `rng()*v.age` is a call-site argument in JS, evaluated before
        // placeSizedVolcano's own body runs — the age draw happens
        // *before* the r/hM/radKm draws inside it, not after.
        let age = rng.next_f64() * volc_age;
        if place_sized_volcano(
            gw,
            gh,
            field,
            volcanic_field,
            map_width_km,
            peak_m,
            cx,
            cy,
            &mut rng,
            age,
        )
        .is_some()
        {
            placed += 1;
        }
    }
}

/// `classifyBoundaries()` (reference HTML lines 3508-3512): splits plate
/// boundary cells by stress sign — convergent (subduction, `s > 0.05`) vs.
/// divergent (rift, `s < -0.05`). `stress_field` is a per-cell `f32`
/// (`Float32Array` in JS); comparing the `f64`-promoted value against the
/// `f64` literal thresholds, rather than comparing two `f32`s directly,
/// matches how JS itself reads a typed-array element back as a full-width
/// number before the comparison.
fn classify_boundaries(boundary_mask: &[u8], stress_field: &[f32]) -> (Vec<usize>, Vec<usize>) {
    let mut conv = Vec::new();
    let mut div = Vec::new();
    for i in 0..boundary_mask.len() {
        if boundary_mask[i] == 0 {
            continue;
        }
        let s = stress_field[i] as f64;
        if s > 0.05 {
            conv.push(i);
        } else if s < -0.05 {
            div.push(i);
        }
    }
    (conv, div)
}

/// `placeProvinceVolcanoes()` (reference HTML lines 3514-3538): places one
/// province's volcanoes — an age-progressive hotspot chain along plate
/// drift, or (arc/rift) boundary-hugging placements spaced along the
/// matching convergent/divergent cell pool, falling back to a scatter
/// around the province centre when that pool has no candidates within
/// `rad_prov`. The reference's own `placed` return array is never read by
/// its one caller (`stamp_volcanoes_provinces` discards it) — the side
/// effects on `field`/`volcanic_field` are what matter, so this doesn't
/// build it.
///
/// Every RNG draw below is its own `let`, in the same left-to-right order
/// the JS source's (multi-draw) argument lists evaluate in —
/// `place_sized_volcano` also takes `rng: &mut Mulberry32`, so inlining
/// draws directly as call arguments the way the JS source reads would
/// need two live mutable borrows of `rng` at once (one held for the `rng`
/// argument itself, one for a later argument's own `.next_f64()` call) and
/// doesn't compile; splitting each draw into its own statement first, in
/// JS's exact argument-evaluation order, is both what compiles and what's
/// order-correct.
#[allow(clippy::too_many_arguments)]
fn place_province_volcanoes(
    gw: usize,
    gh: usize,
    field: &mut [f32],
    volcanic_field: &mut [f32],
    map_width_km: f64,
    peak_m: f64,
    plate_id: &[u16],
    plates: &[Plate],
    kind: &str,
    cx: f64,
    cy: f64,
    rad_prov: f64,
    count: i32,
    conv: &[usize],
    div: &[usize],
    rng: &mut Mulberry32,
    cell_km: f64,
    volc_age: f64,
) {
    if kind == "hotspot" {
        let pid = plate_id[(cy as usize) * gw + cx as usize] as usize;
        let pl = &plates[pid];
        let (mut ux, mut uy) = (pl.vx, pl.vy);
        let l = ux.hypot(uy);
        let l = if l == 0.0 { 1.0 } else { l };
        ux /= l;
        uy /= l;
        let step = (80.0 + rng.next_f64() * 70.0) / cell_km;
        for n in 0..count {
            let t = n as f64 - (count as f64 - 1.0) / 2.0;
            let jx = (rng.next_f64() * 2.0 - 1.0) * 6.0;
            let jy = (rng.next_f64() * 2.0 - 1.0) * 6.0;
            let x = cx + ux * step * t + jx;
            let y = cy + uy * step * t + jy;
            let age = ((n as f64 / (count as f64 - 1.0).max(1.0)) * 0.85 + rng.next_f64() * volc_age * 0.4).min(1.0);
            place_sized_volcano(gw, gh, field, volcanic_field, map_width_km, peak_m, x, y, rng, age);
        }
        return;
    }

    let pool: &[usize] = if kind == "arc" { conv } else { div };
    let mut cand: Vec<usize> = pool
        .iter()
        .copied()
        .filter(|&i| {
            let x = (i % gw) as f64;
            let y = (i / gw) as f64;
            (x - cx).hypot(y - cy) <= rad_prov
        })
        .collect();
    let sp = (50.0 + rng.next_f64() * 100.0) / cell_km;

    if cand.is_empty() {
        for _ in 0..count {
            // `6.283`, not `TAU` -- the reference HTML's own literal
            // (line 3530), not a full-precision 2*pi. Matching it exactly
            // is the point (`cartalith-rust-conventions`: match precision,
            // don't improve it).
            #[allow(clippy::approx_constant)]
            let a = rng.next_f64() * 6.283;
            let r = rng.next_f64() * rad_prov * 0.5;
            let age = rng.next_f64() * volc_age;
            place_sized_volcano(
                gw,
                gh,
                field,
                volcanic_field,
                map_width_km,
                peak_m,
                cx + a.cos() * r,
                cy + a.sin() * r,
                rng,
                age,
            );
        }
        return;
    }

    // Fisher-Yates shuffle -- JS's own loop shape exactly
    // (`for(k=cand.length-1;k>0;k--)`), not a library shuffle, since the
    // exact RNG draw sequence is load-bearing for parity.
    for k in (1..cand.len()).rev() {
        let j = (rng.next_f64() * (k as f64 + 1.0)) as usize;
        cand.swap(k, j);
    }

    let mut chosen: Vec<usize> = Vec::new();
    for &i in &cand {
        let x = (i % gw) as f64;
        let y = (i / gw) as f64;
        let ok = chosen.iter().all(|&c| {
            let cxp = (c % gw) as f64;
            let cyp = (c / gw) as f64;
            (x - cxp).hypot(y - cyp) >= sp
        });
        if ok {
            chosen.push(i);
            if chosen.len() as i32 >= count {
                break;
            }
        }
    }

    let mut pc = 0;
    for &i in &chosen {
        let x = (i % gw) as f64;
        let y = (i / gw) as f64;
        let jx = (rng.next_f64() * 2.0 - 1.0) * 6.0;
        let jy = (rng.next_f64() * 2.0 - 1.0) * 6.0;
        let age = rng.next_f64() * volc_age;
        place_sized_volcano(gw, gh, field, volcanic_field, map_width_km, peak_m, x + jx, y + jy, rng, age);
        pc += 1;
    }
    while pc < count {
        let base = if !chosen.is_empty() {
            chosen[(rng.next_f64() * chosen.len() as f64) as usize]
        } else {
            cand[(rng.next_f64() * cand.len() as f64) as usize]
        };
        let bx = (base % gw) as f64;
        let by = (base / gw) as f64;
        let jx = (rng.next_f64() * 2.0 - 1.0) * 8.0;
        let jy = (rng.next_f64() * 2.0 - 1.0) * 8.0;
        let age = rng.next_f64() * volc_age;
        place_sized_volcano(gw, gh, field, volcanic_field, map_width_km, peak_m, bx + jx, by + jy, rng, age);
        pc += 1;
    }
}

/// `stampVolcanoesProvinces()` (reference HTML lines 3540-3556): the JS
/// default (`state.volc.provinces` = `true`) — clusters volcanoes into a
/// handful of provinces (75% arc/subduction, 15% rift, 10% hotspot chain)
/// along plate boundaries rather than dusting them uniformly the way
/// `stamp_volcanoes_simple` does.
#[allow(clippy::too_many_arguments)]
pub fn stamp_volcanoes_provinces(
    gw: usize,
    gh: usize,
    seed: u32,
    map_width_km: f64,
    peak_m: f64,
    boundary_mask: &[u8],
    stress_field: &[f32],
    plate_id: &[u16],
    plates: &[Plate],
    volc_count: i32,
    volc_age: f64,
    field: &mut [f32],
    volcanic_field: &mut [f32],
) {
    let mut rng = Mulberry32::new(seed ^ 0x5bf03635);
    let (conv, div) = classify_boundaries(boundary_mask, stress_field);
    let cell_km = map_width_km / gw as f64;
    let n_prov = (js_round(volc_count as f64 / 7.0) as i32).clamp(1, 7);
    let mut remaining = volc_count;

    for pi in 0..n_prov {
        if remaining <= 0 {
            break;
        }
        let roll = rng.next_f64();
        let mut kind = if roll < 0.75 {
            "arc"
        } else if roll < 0.90 {
            "rift"
        } else {
            "hotspot"
        };
        if kind == "arc" && conv.is_empty() {
            kind = if !div.is_empty() { "rift" } else { "hotspot" };
        }
        if kind == "rift" && div.is_empty() {
            kind = if !conv.is_empty() { "arc" } else { "hotspot" };
        }

        let (cx, cy, rad_km_p);
        match kind {
            "arc" => {
                let i = conv[(rng.next_f64() * conv.len() as f64) as usize];
                cx = (i % gw) as f64;
                cy = (i / gw) as f64;
                rad_km_p = 100.0 + rng.next_f64() * 200.0;
            }
            "rift" => {
                let i = div[(rng.next_f64() * div.len() as f64) as usize];
                cx = (i % gw) as f64;
                cy = (i / gw) as f64;
                rad_km_p = 200.0 + rng.next_f64() * 400.0;
            }
            _ => {
                cx = (rng.next_f64() * gw as f64).floor();
                cy = (rng.next_f64() * gh as f64).floor();
                rad_km_p = 500.0 + rng.next_f64() * 500.0;
            }
        }

        let prov_left = (n_prov - pi) as f64;
        let mut sub = (js_round(remaining as f64 / prov_left * (0.6 + 0.8 * rng.next_f64())) as i32).max(1);
        if sub > remaining {
            sub = remaining;
        }
        place_province_volcanoes(
            gw,
            gh,
            field,
            volcanic_field,
            map_width_km,
            peak_m,
            plate_id,
            plates,
            kind,
            cx,
            cy,
            rad_km_p / cell_km,
            sub,
            &conv,
            &div,
            &mut rng,
            cell_km,
            volc_age,
        );
        remaining -= sub;
    }
}

/// `stampOneCrater()` (reference HTML lines 3567-3575): a bowl (with an
/// optional central peak for large craters), a raised rim ring, and
/// optional concentric basin ringing for the largest impacts.
///
/// **Three separate `field[i]+=` sites**, not one combined delta — each
/// is its own JS `Float32Array` read-modify-write-round step, and a later
/// conditional reads back the *already-rounded* result of an earlier one
/// (the rim and basin terms both see the bowl term's rounding). `add_rounded`
/// is called once per site to reproduce that, rather than summing all
/// three in `f64` and rounding once — which would occasionally disagree
/// right at the clamp boundary, the same trap `stamp_one_volcano` has.
#[allow(clippy::too_many_arguments)]
fn stamp_one_crater(
    gw: usize,
    gh: usize,
    field: &mut [f32],
    impact_field: &mut [f32],
    cx: f64,
    cy: f64,
    rad_cells: f64,
    large: bool,
    basin: bool,
    age: f64,
) {
    let r = rad_cells.max(1.5);
    let depth = (0.02 + rad_cells * 0.004).min(0.4) * (1.0 - age * 0.8);
    let rim = depth * 0.25;
    let x0 = (((cx - r * 1.3) as i64).max(0)) as usize;
    let x1 = (((cx + r * 1.3) as i64).min(gw as i64 - 1)).max(0) as usize;
    let y0 = (((cy - r * 1.3) as i64).max(0)) as usize;
    let y1 = (((cy + r * 1.3) as i64).min(gh as i64 - 1)).max(0) as usize;
    for y in y0..=y1 {
        for x in x0..=x1 {
            let dx = x as f64 - cx;
            let dy = y as f64 - cy;
            let d = dx.hypot(dy);
            let t = d / r;
            let i = y * gw + x;
            if t < 1.0 {
                add_rounded(field, i, -depth * (1.0 - t * t));
                if large && t < 0.18 {
                    add_rounded(field, i, depth * 0.5 * (1.0 - t / 0.18));
                }
            }
            if t > 0.85 && t < 1.25 {
                let rt = 1.0 - (t - 1.05).abs() / 0.2;
                if rt > 0.0 {
                    add_rounded(field, i, rim * rt);
                }
            }
            if basin && t < 1.0 {
                add_rounded(field, i, (t * std::f64::consts::PI * 3.0).cos() * depth * 0.08);
            }
            let stored = field[i] as f64;
            field[i] = if stored < 0.0 {
                0.0
            } else if stored > 1.0 {
                1.0
            } else {
                field[i]
            };
            if d < r {
                let vv = (1.0 - t).max(impact_field[i] as f64);
                impact_field[i] = vv as f32;
            }
        }
    }
}

/// `stampCraters()` (reference HTML lines 3568-3576): scattered impacts
/// with power-law size classes; large craters (top 1%) additionally
/// reject placements that would overlap an existing large crater, and
/// craters above 200km real radius also get concentric basin ringing.
/// Radius scales as `g^-0.22` — lower gravity, bigger craters for the
/// same impactor (`docs/research/gravity-influence.md`).
#[allow(clippy::too_many_arguments)]
pub fn stamp_craters(
    gw: usize,
    gh: usize,
    seed: u32,
    map_width_km: f64,
    g: f64,
    crater_count: i32,
    crater_age: f64,
    field: &mut [f32],
    impact_field: &mut [f32],
) {
    if crater_count <= 0 {
        return;
    }
    let mut rng = Mulberry32::new(seed ^ 0x27d4eb2f);
    let cell_km = map_width_km / gw as f64;
    let mut big: Vec<(f64, f64, f64)> = Vec::new();
    let mut placed = 0;
    let mut guard = 0;
    while placed < crater_count && guard < crater_count * 40 {
        guard += 1;
        let cx = (rng.next_f64() * gw as f64).floor();
        let cy = (rng.next_f64() * gh as f64).floor();
        let r = rng.next_f64();
        let rad_km;
        let mut large = false;
        let mut basin = false;
        if r < 0.90 {
            rad_km = 0.5 + rng.next_f64() * 4.5;
        } else if r < 0.99 {
            rad_km = 5.0 + rng.next_f64() * 20.0;
        } else {
            rad_km = 25.0 + rng.next_f64() * 175.0;
            large = true;
            if rad_km > 200.0 {
                basin = true;
            }
        }
        let rad_cells = clamp_feature_radius_cells(rad_km * g.powf(-0.22) / cell_km, gw, gh);
        if large {
            let overlaps = big
                .iter()
                .any(|&(bx, by, br)| (bx - cx).hypot(by - cy) < (br + rad_cells) * 0.8);
            if overlaps {
                continue;
            }
            big.push((cx, cy, rad_cells));
        }
        let age = rng.next_f64() * crater_age;
        stamp_one_crater(gw, gh, field, impact_field, cx, cy, rad_cells, large, basin, age);
        placed += 1;
    }
}

/* ===================== T1: boundary polyline graph (docs/research/tectonic-feature-graph.md) =====================
   Turns the per-cell boundary mask into vector polylines so T2+3 (`build_orogeny_field`) can grow
   features ALONG each margin (arc-length parameterised, per-segment type), instead of the blurred
   stress blob. Pure + testable, no RNG and no floating-point-precision concerns (the thinning/tracing
   stage is pure integer/topology; only `poly_meta`'s arc-length and turning-angle use floats, and
   both are ordinary `Math.hypot`/`Math.atan2` calls this port already handles the same way elsewhere).
   World-wrap is deliberately ignored here too (reference HTML's own comment: "a margin crossing the
   x-seam splits in two — a documented refinement for later"), matching upstream. */

/// `thinMask()` (reference HTML lines 2888-2909): Zhang-Suen thinning —
/// reduces a (possibly 2-cell-thick) boundary mask to a 1-pixel skeleton.
/// Two half-steps per pass (even step removes north/east-facing boundary
/// pixels, odd removes south/west-facing), repeated until a full pass
/// deletes nothing. Each half-step reads the mask as it stood BEFORE that
/// half-step's own deletions — `del` is collected during the scan and
/// applied only after it completes, matching JS exactly; deleting
/// in-place while scanning would remove extra pixels a real Zhang-Suen
/// pass wouldn't.
pub fn thin_mask(mask: &[u8], w: usize, h: usize) -> Vec<u8> {
    let mut a: Vec<u8> = mask.to_vec();
    let g = |a: &[u8], x: i64, y: i64| -> u8 {
        if x < 0 || y < 0 || x >= w as i64 || y >= h as i64 {
            0
        } else {
            a[y as usize * w + x as usize]
        }
    };
    let mut changed = true;
    while changed {
        changed = false;
        for step in 0..2 {
            let mut del = Vec::new();
            for y in 0..h {
                for x in 0..w {
                    let idx = y * w + x;
                    if a[idx] == 0 {
                        continue;
                    }
                    let (xi, yi) = (x as i64, y as i64);
                    let p2 = g(&a, xi, yi - 1);
                    let p3 = g(&a, xi + 1, yi - 1);
                    let p4 = g(&a, xi + 1, yi);
                    let p5 = g(&a, xi + 1, yi + 1);
                    let p6 = g(&a, xi, yi + 1);
                    let p7 = g(&a, xi - 1, yi + 1);
                    let p8 = g(&a, xi - 1, yi);
                    let p9 = g(&a, xi - 1, yi - 1);
                    let b = p2 + p3 + p4 + p5 + p6 + p7 + p8 + p9;
                    if !(2..=6).contains(&b) {
                        continue;
                    }
                    let seq = [p2, p3, p4, p5, p6, p7, p8, p9];
                    let mut a_count = 0;
                    for k in 0..8 {
                        if seq[k] == 0 && seq[(k + 1) % 8] == 1 {
                            a_count += 1;
                        }
                    }
                    if a_count != 1 {
                        continue;
                    }
                    if step == 0 {
                        if p2 * p4 * p6 != 0 || p4 * p6 * p8 != 0 {
                            continue;
                        }
                    } else if p2 * p4 * p8 != 0 || p2 * p6 * p8 != 0 {
                        continue;
                    }
                    del.push(idx);
                }
            }
            if !del.is_empty() {
                changed = true;
                for i in del {
                    a[i] = 0;
                }
            }
        }
    }
    a
}

/// One 8-connected neighbor offset ring, in the exact order the reference
/// HTML's own `N8` array lists them — iteration order is behaviorally
/// significant here (which neighbor `nbrs()` returns *first*, and thus
/// which direction a walk sets off in when a node has more than one
/// unvisited neighbor), not just a style choice.
const N8: [(i64, i64); 8] = [(-1, -1), (0, -1), (1, -1), (-1, 0), (1, 0), (-1, 1), (0, 1), (1, 1)];

fn nbrs(a: &[u8], w: usize, h: usize, x: usize, y: usize) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    for &(dx, dy) in &N8 {
        let nx = x as i64 + dx;
        let ny = y as i64 + dy;
        if nx >= 0 && ny >= 0 && (nx as usize) < w && (ny as usize) < h && a[ny as usize * w + nx as usize] != 0 {
            out.push((nx as usize, ny as usize));
        }
    }
    out
}

/// One traced boundary-mask polyline: its grid-cell points plus
/// `_polyMeta()`'s arc-length/curvature/closed-loop metadata. `kind`
/// (one of the `btype` constants) starts at `btype::NONE` — `trace_boundaries`
/// itself only knows geometry; `tag_boundary_types` (`currentBoundaryGraph`'s
/// own per-polyline dominant-boundary-type majority vote) fills it in,
/// matching the JS split between untyped `traceBoundaries` and typed
/// `currentBoundaryGraph`.
#[derive(Debug, Clone, PartialEq)]
pub struct BoundaryPolyline {
    pub pts: Vec<(usize, usize)>,
    pub length: f64,
    pub closed: bool,
    pub curvature: f64,
    pub kind: u8,
}

/// `traceBoundaries()`'s own return value: every traced polyline plus the
/// junction list (`deg >= 3` cells only — degree-1 endpoints chain-walk
/// terminates on too, but JS's own `nodes` array doesn't include them).
pub struct BoundaryGraph {
    pub polylines: Vec<BoundaryPolyline>,
    pub nodes: Vec<(usize, usize)>,
}

/// `_polyMeta()` (reference HTML lines 2910-2920): arc length (sum of
/// per-segment `Math.hypot`), total absolute turning angle normalized by
/// length (`curvature`), and whether the point list closes back on
/// itself.
fn poly_meta(pts: Vec<(usize, usize)>) -> BoundaryPolyline {
    let mut length = 0.0;
    for i in 1..pts.len() {
        let dx = pts[i].0 as f64 - pts[i - 1].0 as f64;
        let dy = pts[i].1 as f64 - pts[i - 1].1 as f64;
        length += dx.hypot(dy);
    }
    let mut turn = 0.0;
    for i in 1..pts.len().saturating_sub(1) {
        let a1 = (pts[i].1 as f64 - pts[i - 1].1 as f64).atan2(pts[i].0 as f64 - pts[i - 1].0 as f64);
        let a2 = (pts[i + 1].1 as f64 - pts[i].1 as f64).atan2(pts[i + 1].0 as f64 - pts[i].0 as f64);
        let mut d = a2 - a1;
        while d > std::f64::consts::PI {
            d -= 2.0 * std::f64::consts::PI;
        }
        while d < -std::f64::consts::PI {
            d += 2.0 * std::f64::consts::PI;
        }
        turn += d.abs();
    }
    let n = pts.len();
    let closed = n > 3 && pts[0] == pts[n - 1];
    let curvature = if length > 0.0 { turn / length } else { 0.0 };
    BoundaryPolyline { pts, length, closed, curvature, kind: btype::NONE }
}

/// Walks one chain starting at node/endpoint `(sx,sy)`, having just
/// stepped to `(fx,fy)` — advances along degree-2 cells until hitting
/// another node/endpoint, a dead end, or closing back on `(sx,sy)`.
/// Marks every cell it steps onto (never the start `(sx,sy)` itself) as
/// `visited`.
#[allow(clippy::too_many_arguments)]
fn walk(
    a: &[u8],
    deg: &[u8],
    visited: &mut [u8],
    w: usize,
    h: usize,
    sx: usize,
    sy: usize,
    fx: usize,
    fy: usize,
) -> Vec<(usize, usize)> {
    let mut pts = vec![(sx, sy)];
    let (mut px, mut py) = (sx, sy);
    let (mut cx, mut cy) = (fx, fy);
    loop {
        pts.push((cx, cy));
        visited[cy * w + cx] = 1;
        if deg[cy * w + cx] != 2 {
            break;
        }
        let ns: Vec<(usize, usize)> = nbrs(a, w, h, cx, cy).into_iter().filter(|&q| q != (px, py)).collect();
        let Some(&(nx, ny)) = ns.first() else {
            break;
        };
        px = cx;
        py = cy;
        cx = nx;
        cy = ny;
        if (cx, cy) == (sx, sy) {
            pts.push((cx, cy));
            break;
        }
    }
    pts
}

/// `traceBoundaries()` (reference HTML lines 2921-2952): thins
/// `boundary_mask` to a 1-px skeleton, then traces it into polylines —
/// chains run between nodes (degree != 2: endpoints degree-1, junctions
/// degree-3+), pure loops (all degree-2, no node) traced separately.
///
/// **A direct node-to-node edge is walked from both ends** (reference
/// HTML's own behavior, not a bug this port introduces): `walk()` never
/// marks its *starting* cell visited, only cells it steps onto — so a
/// one-cell edge between two nodes gets recorded twice, once per
/// endpoint, in whichever order the outer node scan reaches each end.
/// Ported as-is; deduplicating would be a behavioral improvement over
/// JS, which `cartalith-porting-discipline` reserves for a deliberately
/// logged decision, not a silent "fix" made while porting.
pub fn trace_boundaries(mask: &[u8], w: usize, h: usize) -> BoundaryGraph {
    let a = thin_mask(mask, w, h);

    let mut deg = vec![0u8; w * h];
    for y in 0..h {
        for x in 0..w {
            if a[y * w + x] != 0 {
                deg[y * w + x] = nbrs(&a, w, h, x, y).len() as u8;
            }
        }
    }

    let mut nodes = Vec::new();
    for y in 0..h {
        for x in 0..w {
            if a[y * w + x] != 0 && deg[y * w + x] >= 3 {
                nodes.push((x, y));
            }
        }
    }

    let mut visited = vec![0u8; w * h];
    let mut polylines = Vec::new();

    for y in 0..h {
        for x in 0..w {
            if a[y * w + x] == 0 || deg[y * w + x] == 2 {
                continue;
            }
            for (nx, ny) in nbrs(&a, w, h, x, y) {
                if visited[ny * w + nx] != 0 {
                    continue;
                }
                let pts = walk(&a, &deg, &mut visited, w, h, x, y, nx, ny);
                if pts.len() >= 2 {
                    polylines.push(poly_meta(pts));
                }
            }
        }
    }

    for y in 0..h {
        for x in 0..w {
            if a[y * w + x] == 0 || deg[y * w + x] != 2 || visited[y * w + x] != 0 {
                continue;
            }
            visited[y * w + x] = 1;
            let ns = nbrs(&a, w, h, x, y);
            let Some(&(fx, fy)) = ns.first() else {
                continue;
            };
            let pts = walk(&a, &deg, &mut visited, w, h, x, y, fx, fy);
            if pts.len() >= 2 {
                polylines.push(poly_meta(pts));
            }
        }
    }

    BoundaryGraph { polylines, nodes }
}

/// `currentBoundaryGraph()`'s own per-polyline tagging step (reference
/// HTML lines 2954-2964, minus the `_boundaryGraph` cache — a perf-only
/// detail with no effect on output values, the same reasoning
/// `compute_warp`'s own doc comment gives for not reproducing JS's
/// analogous memoization). For each polyline, counts each `btype` value
/// among its points and sets `.kind` to the most frequent one —
/// `btype::NONE` (0) is excluded from the vote (a polyline touching no
/// classified boundary cell falls back to `NONE` via `kind`'s own
/// `poly_meta` default, not through this function picking it).
pub fn tag_boundary_types(graph: &mut BoundaryGraph, boundary_type: &[u8], gw: usize) {
    for pl in &mut graph.polylines {
        let mut counts = [0i32; 6];
        for &(x, y) in &pl.pts {
            counts[boundary_type[y * gw + x] as usize] += 1;
        }
        let mut best = -1;
        let mut bk = 0u8;
        for (k, &count) in counts.iter().enumerate().skip(1) {
            if count > best {
                best = count;
                bk = k as u8;
            }
        }
        pl.kind = bk;
    }
}

/// A Gaussian bump: `G(u,c,s) = exp(-(u-c)^2/s^2)` (reference HTML line
/// 3032's own inline arrow function).
fn orogeny_g(u: f64, c: f64, s: f64) -> f64 {
    (-((u - c) * (u - c)) / (s * s)).exp()
}

/// Per-boundary-type search radius (reference HTML line 2991's `RADS`
/// map) — `None` for `btype::NONE` or any other value, meaning "this
/// polyline contributes no orogeny stamp" (`build_orogeny_field`'s own
/// skip condition).
fn orogeny_radius(kind: u8, blur_r: f64) -> Option<f64> {
    match kind {
        btype::COLLISION => Some(blur_r * 3.3),
        btype::SUBDUCTION_OC => Some(blur_r * 2.3),
        btype::ARC_OO => Some(blur_r * 3.05),
        btype::RIFT => Some(blur_r * 1.85),
        btype::TRANSFORM => Some(blur_r * 2.0),
        _ => None,
    }
}

/// Tuning knobs `buildOrogenyField()`'s own `opts` bag takes (reference
/// HTML lines 2982-2990). `block_w`/`jitter` (JS: `opts.blockW`/
/// `opts.jitter`) aren't exposed here — this port's one call site
/// (`cartalith-engine`) never overrides either, matching JS's own only
/// caller, so their JS defaults (`0.55`, `0.5`) are hardcoded constants
/// inside `build_orogeny_field` instead of unused knobs threaded through
/// a struct nothing varies (same reasoning `WeatherParams` gives for
/// omitting `radius_rel`).
pub struct OrogenyParams<'a> {
    pub blur_r: f64,
    pub seed: i32,
    pub shear: Option<&'a [f32]>,
    pub fold_k: f64,
    pub trench_k: f64,
    pub fault_block_k: f64,
}

/// `buildOrogenyField()` (reference HTML lines 2965-3071, "T2+T3: tectonic
/// feature kernels") — per-boundary-type uplift/depression profiles
/// stamped along margin polylines, replacing the blurred convergent-stress
/// blob with linear, asymmetric, type-specific landforms: collision
/// (multi-ridge fold + orogenic plateau + foreland-basin depression),
/// subduction/arc (trench + volcanic arc, arc's narrower + has a backarc
/// basin), rift (axial graben + uplifted shoulders, optionally repeated
/// Basin-and-Range fault blocks when `fault_block_k > 0`), and transform
/// (shear-driven linear fault valley + en-echelon pressure ridge/
/// releasing-bend pair, offset by signed shear).
///
/// For each polyline: mean `|stress|`/`|shear|`/signed-shear along its
/// points (skipped entirely if both are negligible), a majority vote for
/// which geometric side is oceanic (sampled ±3 cells along the local
/// normal at up to 16 points), then a signed-distance-field scan
/// (per-segment bounding-box windows, nearest-segment `t`-projection) that
/// finds every cell within that type's search radius. Each found cell's
/// signed distance is wiggled by an `fbm` crest jitter and its overall
/// vigor modulated by a second `fbm` along-strike factor before the
/// per-type profile is evaluated and combined into `U` by `|max|` across
/// overlapping margins (so a junction of two features doesn't double-stack).
///
/// Pure. `polylines` must already be typed (`tag_boundary_types`).
pub fn build_orogeny_field(polylines: &[BoundaryPolyline], stress: &[f32], crust: &[f32], w: usize, h: usize, p: &OrogenyParams) -> Vec<f32> {
    let blur_r = (if p.blur_r == 0.0 { 18.0 } else { p.blur_r }).max(4.0);
    let seed = p.seed;
    let s1 = blur_r * 0.42;
    let d1 = blur_r * 1.0;
    let s2 = blur_r * 0.30;
    let block_w = 0.55 * blur_r;
    let jit = 0.5;

    let n = w * h;
    let mut u = vec![0f32; n];
    // JS: `new Float32Array(W*H)` -- every store narrows to f32, so a
    // later read is the f32-rounded value widened back to f64, not the
    // full-precision f64 that produced it. Keeping these as f64 (a first
    // pass here did) drops that per-cell rounding step JS always takes.
    let mut dist = vec![0f32; n];
    let mut side = vec![0f32; n];
    let mut mark = vec![-1i32; n];
    let mut pid = -1i32;

    for pl in polylines {
        let Some(rad) = orogeny_radius(pl.kind, blur_r) else {
            continue;
        };
        if pl.pts.len() < 2 {
            continue;
        }
        pid += 1;

        let mut amp = 0f64;
        let mut sh_amp = 0f64;
        let mut sh_sig = 0f64;
        for &(x, y) in &pl.pts {
            let i0 = y * w + x;
            amp += (stress[i0] as f64).abs();
            if let Some(shear) = p.shear {
                sh_amp += (shear[i0] as f64).abs();
                sh_sig += shear[i0] as f64;
            }
        }
        let len = pl.pts.len() as f64;
        amp /= len;
        sh_amp /= len;
        sh_sig /= len;
        if amp < 1e-4 && sh_amp < 1e-4 {
            continue;
        }

        // Which geometric side is oceanic? Majority vote over `crust`
        // (<0 = oceanic) sampled +-3 cells along the local normal, at up
        // to 16 points spaced along the polyline. Ties (O-O, C-C) -> -1.
        let q = &pl.pts;
        let step = (q.len() / 16).max(1);
        let mut vote = 0i32;
        let mut k = 1usize;
        while k + 1 < q.len() {
            let (kx1, ky1) = q[k + 1];
            let (kxm1, kym1) = q[k - 1];
            let mut tx = kx1 as f64 - kxm1 as f64;
            let mut ty = ky1 as f64 - kym1 as f64;
            let tl = tx.hypot(ty);
            let tl = if tl == 0.0 { 1.0 } else { tl };
            tx /= tl;
            ty /= tl;
            let nx = -ty;
            let ny = tx;
            let (kx, ky) = q[k];
            let xp = (js_round(kx as f64 + nx * 3.0)).clamp(0.0, w as f64 - 1.0) as usize;
            let yp = (js_round(ky as f64 + ny * 3.0)).clamp(0.0, h as f64 - 1.0) as usize;
            let xm = (js_round(kx as f64 - nx * 3.0)).clamp(0.0, w as f64 - 1.0) as usize;
            let ym = (js_round(ky as f64 - ny * 3.0)).clamp(0.0, h as f64 - 1.0) as usize;
            vote += i32::from(crust[yp * w + xp] < 0.0) - i32::from(crust[ym * w + xm] < 0.0);
            k += step;
        }
        let ocean_sign = if vote > 0 { 1.0 } else { -1.0 };

        // Signed distance field around this polyline: nearest-segment
        // scan in per-segment windows.
        let mut touched: Vec<usize> = Vec::new();
        let pts = &pl.pts;
        for s in 0..pts.len() - 1 {
            let (ax, ay) = (pts[s].0 as f64, pts[s].1 as f64);
            let (bx, by) = (pts[s + 1].0 as f64, pts[s + 1].1 as f64);
            let ex = bx - ax;
            let ey = by - ay;
            let e_l2 = { let v = ex * ex + ey * ey; if v == 0.0 { 1.0 } else { v } };
            let x0 = (ax.min(bx) - rad).floor().max(0.0) as usize;
            let x1 = (((ax.max(bx) + rad).ceil()).min(w as f64 - 1.0)) as usize;
            let y0 = (ay.min(by) - rad).floor().max(0.0) as usize;
            let y1 = (((ay.max(by) + rad).ceil()).min(h as f64 - 1.0)) as usize;
            for y in y0..=y1 {
                for x in x0..=x1 {
                    let (xf, yf) = (x as f64, y as f64);
                    let t = (((xf - ax) * ex + (yf - ay) * ey) / e_l2).clamp(0.0, 1.0);
                    let dx = xf - (ax + ex * t);
                    let dy = yf - (ay + ey * t);
                    let d = dx.hypot(dy);
                    if d > rad {
                        continue;
                    }
                    let i = y * w + x;
                    let sgn = if ex * dy - ey * dx >= 0.0 { 1.0 } else { -1.0 };
                    if mark[i] != pid {
                        mark[i] = pid;
                        dist[i] = d as f32;
                        side[i] = sgn as f32;
                        touched.push(i);
                    } else if d < dist[i] as f64 {
                        dist[i] = d as f32;
                        side[i] = sgn as f32;
                    }
                }
            }
        }

        for &i in &touched {
            let x = i % w;
            let y = i / w;
            let sd = side[i] as f64 * dist[i] as f64;
            let de = sd
                + (fbm(x as f64 / (blur_r * 2.2), y as f64 / (blur_r * 2.2), seed ^ 0x7e11) - 0.5) * jit * blur_r * 2.0;
            let aj = 0.75 + 0.5 * fbm(x as f64 / (blur_r * 3.1), y as f64 / (blur_r * 3.1), seed ^ 0x33aa);
            let dor = -ocean_sign * de;
            let mut amp_here = amp;
            let prof = if pl.kind == btype::COLLISION {
                (orogeny_g(de, 0.0, s1) + 0.5 * orogeny_g(de, d1, s2) + 0.3 * orogeny_g(de, -d1, s2))
                    * (1.0 + p.fold_k * (2.0 * std::f64::consts::PI * de / d1).cos())
                    + 0.15 * orogeny_g(de, 0.0, blur_r * 0.95)
                    - 0.35 * orogeny_g(de, -blur_r * 1.7, blur_r * 0.5)
            } else if pl.kind == btype::SUBDUCTION_OC {
                -0.9 * p.trench_k * orogeny_g(dor, -blur_r * 0.5, blur_r * 0.30) + 0.75 * orogeny_g(dor, blur_r * 0.8, blur_r * 0.45)
            } else if pl.kind == btype::ARC_OO {
                -0.85 * p.trench_k * orogeny_g(dor, -blur_r * 0.45, blur_r * 0.28) + 0.6 * orogeny_g(dor, blur_r * 0.55, blur_r * 0.30)
                    - 0.22 * orogeny_g(dor, blur_r * 1.5, blur_r * 0.5)
            } else if pl.kind == btype::RIFT {
                let mut prof = -0.45 * orogeny_g(de, 0.0, blur_r * 0.30)
                    + 0.28 * (orogeny_g(de, blur_r * 0.75, blur_r * 0.35) + orogeny_g(de, -blur_r * 0.75, blur_r * 0.35));
                if p.fault_block_k > 0.0 {
                    let uu = de / block_w;
                    let frac = uu - uu.floor();
                    let tilt = (-2.4 * frac).exp() - 0.36;
                    let env = (-(de * de) / ((blur_r * 1.7) * (blur_r * 1.7))).exp();
                    prof += p.fault_block_k * 0.55 * tilt * env;
                }
                prof
            } else {
                // T4 transform (San Andreas / Dead Sea / Alpine Fault):
                // driven by shear, not normal stress.
                amp_here = sh_amp;
                let ro = sh_sig * blur_r * 1.2;
                -0.55 * orogeny_g(de, 0.0, blur_r * 0.45) + 0.32 * orogeny_g(de, ro, blur_r * 0.42)
                    - 0.12 * orogeny_g(de, -ro, blur_r * 0.5)
            };
            // JS compares the full f64 `v` against `U[i]` widened back to
            // f64 (reading a Float32Array never truncates further) and
            // only narrows to f32 at the point of storage. Narrowing `v`
            // to f32 before the comparison, as a first pass here did, can
            // flip which side wins when the two magnitudes are within a
            // few ULPs of each other -- matching JS's own order, not just
            // its final stored value, per cartalith-rust-conventions.
            let v = amp_here * aj * prof;
            if v.abs() > (u[i] as f64).abs() {
                u[i] = v as f32;
            }
        }
    }

    u
}

/// `smoothOrogeny()` (reference HTML lines 3077-3080): a light separable
/// box blur on the finished orogeny field — the per-type Gaussian
/// profiles + `|max|` margin-combine + crest-wiggle leave sharp creases;
/// a small blur (radius proportional to `blur_r`) rounds them off.
pub fn smooth_orogeny(u: &[f32], w: usize, h: usize, blur_r: f64, wrap: bool) -> Vec<f32> {
    let r = (js_round((if blur_r == 0.0 { 18.0 } else { blur_r }) * 0.16) as i64).max(1);
    let mut tmp = vec![0f32; w * h];
    let mut out = u.to_vec();
    box_h(&out, &mut tmp, w, h, r, wrap);
    box_v(&tmp, &mut out, w, h, r);
    out
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

    // Hand-derived against the Zhang-Suen conditions transcribed from the
    // JS source (see thin_mask's own doc comment).
    #[test]
    fn thin_mask_leaves_a_straight_1px_line_untouched() {
        // Endpoints are always kept (B<2 skip); interior cells of a
        // straight line have exactly 2 opposite-side 1-neighbors, which
        // is 2 separate 0->1 transitions around the ring (A=2, not 1),
        // so they're skipped too -- a straight line is a fixed point.
        let mask = vec![1u8, 1, 1, 1, 1];
        let thinned = thin_mask(&mask, 5, 1);
        assert_eq!(thinned, mask);
    }

    #[test]
    fn thin_mask_is_idempotent_and_never_grows() {
        // A solid 5x4 block: not a fixed point (thinning must remove
        // interior/edge pixels down toward a skeleton), but whatever it
        // converges to must itself be stable under a second pass, and
        // must never have MORE set pixels than the input.
        let (w, h) = (5, 4);
        let mask = vec![1u8; w * h];
        let once = thin_mask(&mask, w, h);
        let twice = thin_mask(&once, w, h);
        assert_eq!(once, twice, "a converged thinning must be a fixed point of another pass");
        let count = |m: &[u8]| m.iter().filter(|&&v| v != 0).count();
        assert!(count(&once) < count(&mask), "thinning a solid block must remove pixels");
        assert!(count(&once) > 0, "thinning must not erase the shape entirely");
    }

    #[test]
    fn thin_mask_all_zero_stays_all_zero() {
        let mask = vec![0u8; 12];
        assert_eq!(thin_mask(&mask, 4, 3), mask);
    }

    #[test]
    fn trace_boundaries_traces_a_straight_line_end_to_end() {
        // W=5,H=1, all-ones: two degree-1 endpoints and three degree-2
        // interior cells. `nodes` (the returned list) only reports
        // degree>=3 junctions -- degree-1 endpoints don't count as
        // "nodes" there even though `isNode()`/the walk's own stopping
        // condition (deg != 2) treats them as chain terminators, matching
        // the JS source's own split between the two. Hand-traced:
        // walk(0,0 -> 1,0) advances cell-by-cell to (4,0) (the far
        // endpoint), producing one 5-point chain; (4,0)'s own single
        // neighbor (3,0) is already visited by then, so no second walk
        // starts there.
        let mask = vec![1u8, 1, 1, 1, 1];
        let g = trace_boundaries(&mask, 5, 1);
        assert!(g.nodes.is_empty(), "no degree>=3 junctions on a straight line");
        assert_eq!(g.polylines.len(), 1);
        let p = &g.polylines[0];
        assert_eq!(p.pts, vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)]);
        assert!((p.length - 4.0).abs() < 1e-9);
        assert!(!p.closed);
        assert_eq!(p.curvature, 0.0, "a straight line never turns");
    }

    #[test]
    fn trace_boundaries_walks_a_direct_node_edge_from_both_ends() {
        // A 2-cell mask: both cells are degree-1 endpoints (no degree>=3
        // junction, so `nodes` is empty), each other's only neighbor.
        // Reference HTML's own `traceBoundaries` never marks a walk's
        // STARTING cell visited, so the single edge between them is
        // recorded twice -- once from each end -- documented on
        // `trace_boundaries`'s own doc comment, not a bug this port
        // introduces.
        let mask = vec![1u8, 1];
        let g = trace_boundaries(&mask, 2, 1);
        assert!(g.nodes.is_empty());
        assert_eq!(g.polylines.len(), 2);
        assert_eq!(g.polylines[0].pts, vec![(0, 0), (1, 0)]);
        assert_eq!(g.polylines[1].pts, vec![(1, 0), (0, 0)]);
    }

    #[test]
    fn tag_boundary_types_majority_vote_and_tie_break() {
        // Hand-traceable (pure counting + argmax over a fixed 6-entry
        // array) rather than Node-extracted: the JS this mirrors is
        // inlined in currentBoundaryGraph(), which needs boundaryMask/
        // boundaryType/GW/GH globals a Node vm sandbox can't reach (`let`-
        // scoped, never attached to the context object) -- same class of
        // verification T1's thin_mask/trace_boundaries tests already use.
        //
        // Grid (gw=3): row y=0 is [collision, collision, subductionOC];
        // row y=1 is [rift, rift, _]; row y=2 is [subductionOC, arcOO, _].
        #[rustfmt::skip]
        let boundary_type: [u8; 9] = [
            btype::COLLISION, btype::COLLISION, btype::SUBDUCTION_OC,
            btype::RIFT,       btype::RIFT,       btype::NONE,
            btype::SUBDUCTION_OC, btype::ARC_OO,   btype::NONE,
        ];
        let mut graph = BoundaryGraph {
            polylines: vec![
                // 2 collision cells + 1 subduction -> collision wins outright.
                BoundaryPolyline { pts: vec![(0, 0), (1, 0), (2, 0)], length: 0.0, closed: false, curvature: 0.0, kind: btype::NONE },
                // both rift -> rift.
                BoundaryPolyline { pts: vec![(0, 1), (1, 1)], length: 0.0, closed: false, curvature: 0.0, kind: btype::NONE },
                // tie: 1 subductionOC (k=2) vs 1 arcOO (k=3) -- JS's
                // `if(cnt[k]>best)` is strict, so the lower k found first
                // during the k=1..len scan keeps the win, not the last.
                BoundaryPolyline { pts: vec![(0, 2), (1, 2)], length: 0.0, closed: false, curvature: 0.0, kind: btype::NONE },
            ],
            nodes: vec![],
        };
        tag_boundary_types(&mut graph, &boundary_type, 3);
        assert_eq!(graph.polylines[0].kind, btype::COLLISION);
        assert_eq!(graph.polylines[1].kind, btype::RIFT);
        assert_eq!(graph.polylines[2].kind, btype::SUBDUCTION_OC, "tie keeps the lower boundary-type id, matching JS's strict >");
    }
}
