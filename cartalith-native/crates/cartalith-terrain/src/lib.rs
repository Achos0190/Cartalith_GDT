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

/// Which edifice profile the volcanism stamps build.
///
/// # What the reference actually does, and what it does not
///
/// `stampOneVolcano` (reference HTML lines 3466-3473) is **not** a Gaussian —
/// it is a power-law cone, `H*(1-t)^(1.6-age*0.8)`, mildly concave-up. But it
/// is a *single* profile: the same exponent shapes a 200 m scoria cone and a
/// 7 000 m shield, and the 70/25/5 size roll in `placeSizedVolcano` varies only
/// height and radius (and all three classes land at the same ~1:10 aspect). No
/// edifice-type distinction exists anywhere in it, and `FUNCTION_INDEX.md`'s
/// one-line summary of it — "cone with crater and noise" — is wrong on the last
/// word: there is no roughness term at all.
///
/// Its summit depression is exactly the thing this lane was opened for.
/// `add -= H*0.5*(1-t/0.16)` subtracts a linear inverted cone from the profile,
/// so the deepest point is a *point* at dead centre: no flat floor, no
/// ring-fault wall. It is a crater painted on a cone, not a collapse.
///
/// [`EdificeModel::Reference`] is that code, unchanged and still what every
/// golden pins. [`EdificeModel::Morphological`] replaces the profile with three
/// shapes chosen by the volcanic setting `stamp_volcanoes_provinces` already
/// classifies and then discards, a summit depression that is a genuine
/// collapse, and two-scale flank relief.
///
/// **Gated, and default-off at both boundaries.** It moves the height field, so
/// lithology, biomes, carrying capacity, settlements, roads and sea routes move
/// with it — the same blast radius `DECISIONS.md` §7l had to authorise for
/// craters, and that authorisation was *for craters*. Turning this on needs its
/// own owner ruling.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EdificeModel {
    /// `stampOneVolcano` exactly.
    #[default]
    Reference,
    /// Setting-driven shield/stratocone/scoria-cone profiles, a collapsed
    /// summit, and multi-scale flank relief.
    Morphological,
}

/// The volcanic setting a placement was made in.
///
/// This is not new information: `stampVolcanoesProvinces` already rolls
/// arc/rift/hotspot per province and hands it to `placeProvinceVolcanoes`,
/// which uses it to pick *where* the volcano goes and then throws it away.
/// Nothing downstream of placement has ever seen it. Driving the edifice
/// profile from it is plumbing, not classification.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VolcanicSetting {
    /// Convergent margin. Andesitic stratocones.
    Arc,
    /// Divergent margin. Basaltic, fissure-fed: low shields and scoria cones.
    Rift,
    /// Intraplate chain. Basaltic shields.
    Hotspot,
    /// `stamp_volcanoes_simple`, which classifies nothing.
    Unclassified,
}

/// The three edifice morphologies this model distinguishes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EdificeKind {
    /// Scoria/cinder cone: small, near-straight-sided at the angle of repose,
    /// with a summit crater that is a large fraction of its own base.
    Cinder,
    /// Stratovolcano: steep near the summit, decaying to a long gentle apron —
    /// strongly concave-up in cross-section.
    Strato,
    /// Shield: broad flat summit region, low mid-flank slope, tapering toe.
    Shield,
}

/// One edifice morphology's shape constants, for `h(t) = H*(1 - t^a)^b` over
/// the normalized radius `t ∈ [0,1]`.
///
/// Choosing that two-exponent family (rather than the reference's one-exponent
/// `(1-t)^p`) is what buys the shield: with `a = 1` the summit slope is
/// `-b·H/R`, non-zero for every `b`, so a flat summit is unreachable — and
/// `b < 1`, the only way to get a convex flank out of one exponent, sends the
/// slope at the toe to infinity. `a = 2` puts a zero-slope summit and a
/// zero-slope toe on the same curve with the maximum slope in between, which is
/// the shield profile.
#[derive(Clone, Copy, Debug)]
struct EdificeShape {
    /// Radial exponent. `1.0` = summit is a cone tip; `2.0` = flat summit.
    a: f64,
    /// Summit exponent. Larger = steeper summit and longer gentle apron.
    b: f64,
    /// Summit-depression rim, as a fraction of the edifice radius.
    rim: f64,
    /// Flat-floor radius, as a fraction of the edifice radius.
    floor: f64,
    /// Floor depth below the rim, as a fraction of the edifice height.
    depth: f64,
    /// Flank relief amplitude, as a fraction of the edifice height.
    rough: f64,
}

/// Shape constants per morphology.
///
/// The slopes these produce, at the reference's own height/radius draws:
/// a stratocone (2 000 m over 20 km) reaches ~15° near the summit against the
/// reference cone's ~9°, and a shield (5 000 m over 55 km) peaks at ~8° on the
/// mid-flank with a level summit. Both are still gentler than the real thing
/// (a scoria cone stands at 30-33°, a stratocone summit at ~30°) because the
/// *footprints* are the reference's and they are far too wide for the heights
/// drawn with them — 1:10 for every class. Widening the aspect ratio is a
/// separate change with a separate blast radius; this lane changes shape only.
///
/// Summit-depression geometry, in edifice-radius fractions:
/// - Scoria cones carry proportionally huge craters — a real one's crater is a
///   third of its base diameter — hence `rim: 0.38`.
/// - `Strato`'s 0.35-of-height collapse is Crater-Lake-shaped: the summit is
///   gone, not dimpled.
/// - Shield calderas are shallow relative to a vast edifice (Mokuʻāweoweo is
///   ~180 m on a ~4 km-high shield, so ~0.045); `depth: 0.10` overstates that
///   deliberately, because at grid resolution a truer value rounds to nothing.
///   Same reasoning for `rim: 0.12` against a real ~0.05.
fn edifice_shape(kind: EdificeKind) -> EdificeShape {
    match kind {
        EdificeKind::Cinder => {
            EdificeShape { a: 1.0, b: 1.15, rim: 0.38, floor: 0.16, depth: 0.30, rough: 0.05 }
        }
        EdificeKind::Strato => {
            EdificeShape { a: 1.0, b: 2.60, rim: 0.24, floor: 0.11, depth: 0.35, rough: 0.10 }
        }
        EdificeKind::Shield => {
            EdificeShape { a: 2.0, b: 1.40, rim: 0.12, floor: 0.05, depth: 0.10, rough: 0.05 }
        }
    }
}

/// Which edifice a setting builds at a given size class.
///
/// **Takes no RNG.** `size_roll` is the `r` draw `place_sized_volcano` has
/// already made for the 70/25/5 height/radius classes, reused rather than
/// re-drawn, so switching the model on cannot shift the random stream by a
/// single call — which is what makes `volcanic_field` bit-identical between the
/// two models and gives `volcano_edifice.rs` its stream test.
fn edifice_kind(setting: VolcanicSetting, size_roll: f64) -> EdificeKind {
    let big = size_roll >= 0.70;
    match setting {
        // Subduction arcs build stratocones, with scoria cones as the small
        // parasitic and monogenetic vents around them.
        VolcanicSetting::Arc => {
            if big {
                EdificeKind::Strato
            } else {
                EdificeKind::Cinder
            }
        }
        // Divergent margins are basaltic: low shields and scoria cones. A rift
        // stratocone is the wrong magma.
        VolcanicSetting::Rift => {
            if big {
                EdificeKind::Shield
            } else {
                EdificeKind::Cinder
            }
        }
        // Hotspots are shields at every size — a small one is a young shield,
        // not a different edifice.
        VolcanicSetting::Hotspot => EdificeKind::Shield,
        // `stamp_volcanoes_simple` classifies nothing, so fall back to the
        // reference's own three size-class labels ("small cinder cones,
        // stratovolcanoes, large shields" — `place_sized_volcano`'s own doc).
        VolcanicSetting::Unclassified => {
            if size_roll < 0.70 {
                EdificeKind::Cinder
            } else if size_roll < 0.95 {
                EdificeKind::Strato
            } else {
                EdificeKind::Shield
            }
        }
    }
}

/// Target spacing between radial gullies at mid-flank, in cells. Below about
/// three the gullies alias into per-cell noise; above about six a small cone
/// gets two of them and reads as lopsided rather than dissected.
const EDIFICE_GULLY_SPACING_CELLS: f64 = 4.0;

/// Base frequency of the flank's fbm term, per cell. `fbm` runs six octaves,
/// so this sets the *coarsest* lobe (~17 cells) and the finest lands at about
/// half a cell — that spread is the "multi-scale" part; one octave here would
/// be per-cell hash noise, which is what a heightfield least needs.
const EDIFICE_LUMP_FREQ_PER_CELL: f64 = 0.06;

/// How far age lerps the summit exponent toward `1.0` (a straight cone) by
/// `age = 1`. Same direction the reference's own `1.6 - age*0.8` moves in —
/// older edifices are rounder — but expressed as a fraction of the distance to
/// a straight cone, so it can never push `b` below `1.0`, where the toe slope
/// would diverge.
const EDIFICE_AGE_ROUNDING: f64 = 0.5;

/// The uncollapsed edifice profile, `H*(1 - t^a)^b`.
///
/// Held apart from [`Edifice::add_at`] because both the collapse and the flank
/// relief are defined *relative to it* — the caldera wall has to meet it at the
/// rim, and the roughness is a residual on top of it — so a test can measure
/// either one by subtracting this.
fn edifice_cone(h: f64, a: f64, b: f64, t: f64) -> f64 {
    h * (1.0 - t.powf(a)).powf(b)
}

/// One edifice's precomputed profile: everything in `stamp_one_volcano`'s
/// per-cell loop that does not vary per cell.
#[derive(Clone, Copy, Debug)]
struct Edifice {
    s: EdificeShape,
    /// Normalized peak height, the reference's own `H`.
    h: f64,
    /// Age-adjusted summit exponent.
    b: f64,
    /// Profile height at the depression rim, and on its floor.
    rim_h: f64,
    floor_h: f64,
    /// Radial gully count, and this edifice's own noise salt.
    gullies: i32,
    salt: i32,
}

impl Edifice {
    /// Precompute for a peak height `h` (already age-damped), a radius of
    /// `r` cells, at centre `(cx, cy)`.
    fn new(s: EdificeShape, h: f64, r: f64, cx: f64, cy: f64, age: f64) -> Self {
        let b = s.b - age * EDIFICE_AGE_ROUNDING * (s.b - 1.0);
        let rim_h = edifice_cone(h, s.a, b, s.rim);
        // A caldera floor sits above the surrounding plain (Crater Lake's is
        // ~1 880 m against a ~1 600 m apron), and this term is *added* to the
        // field, so a negative floor would excavate pre-existing terrain the
        // collapse never touched. Deliberately **not** clamped at zero here:
        // the sign of `rim_h - h*depth` is independent of `h`, and `age` only
        // raises `rim_h`, so whether it can go negative is decided entirely by
        // the shape table — which `caldera_floor_is_flat_and_has_extent` checks
        // for all three morphologies. A `.max(0.0)` would silently reshape a
        // mis-tuned constant into a flat-topped edifice instead of failing.
        let floor_h = rim_h - h * s.depth;
        // Circumference at mid-flank is 2*pi*(r/2) = pi*r cells.
        let gullies = ((std::f64::consts::PI * r / EDIFICE_GULLY_SPACING_CELLS).round() as i32)
            .clamp(6, 48);
        // Deterministic per-edifice salt from the centre. Deliberately not an
        // RNG draw: see `edifice_kind`.
        let salt = ((cx * 7919.0) as i32) ^ ((cy * 104_729.0) as i32).wrapping_mul(31);
        Edifice { s, h, b, rim_h, floor_h, gullies, salt }
    }

    /// This edifice's uncollapsed profile at `t`.
    fn cone_at(&self, t: f64) -> f64 {
        edifice_cone(self.h, self.s.a, self.b, t)
    }

    /// Height added at normalized radius `t ∈ [0,1]`, bearing `theta`, in cell
    /// `(x, y)`.
    fn add_at(&self, t: f64, theta: f64, x: f64, y: f64) -> f64 {
        let mut add = self.cone_at(t);
        if t < self.s.rim {
            // A *collapsed* summit, not a crater subtracted from a cone: the
            // profile inside the ring fault is replaced outright by a flat
            // floor and a smoothstep wall. Smoothstep meets the cone at the rim
            // with matching value *and* matching (zero) slope, so the wall
            // reads as a scarp with a rounded lip rather than a crease.
            let w = ((t - self.s.floor) / (self.s.rim - self.s.floor)).clamp(0.0, 1.0);
            add = self.floor_h + (self.rim_h - self.floor_h) * (w * w * (3.0 - 2.0 * w));
        }
        // Flank relief: exactly zero inside the depression and at the toe,
        // peaking on the mid-flank.
        let u = (t - self.s.rim) / (1.0 - self.s.rim);
        if u > 0.0 {
            let flank = 4.0 * u * (1.0 - u);
            // Bearing mapped onto [0, gullies) and sampled with `pridged`'s
            // matching period, so the noise closes across the +/-pi seam
            // instead of leaving a radial scar there. The radial axis advances
            // ~2.2 units over the whole flank against the angular axis'
            // `gullies`, which is what elongates the structures radially —
            // barrancas, not blobs. Ridged noise is *inverted* here: its sharp
            // high ridges become the incised gullies.
            let ang = (theta * std::f64::consts::FRAC_1_PI * 0.5 + 0.5) * self.gullies as f64;
            let gully = 1.0 - 2.0 * pridged(ang, t * 2.2, self.salt, self.gullies);
            let lumps = 2.0
                * fbm(
                    x * EDIFICE_LUMP_FREQ_PER_CELL,
                    y * EDIFICE_LUMP_FREQ_PER_CELL,
                    self.salt ^ 0x2f1b,
                )
                - 1.0;
            add += self.h * self.s.rough * flank * (gully * 0.65 + lumps * 0.35);
        }
        add
    }
}

/// Geometry of the morphological edifice model. Private items, so these are
/// unit tests rather than an integration suite; the end-to-end behaviour
/// (random stream, whole-field divergence) is in
/// `tests/volcano_edifice.rs`.
#[cfg(test)]
mod edifice_tests {
    use super::*;

    /// A stratocone at the reference's own mid-class draw: 2 000 m over a 20 km
    /// radius on a 4 000 m peak, freshly built.
    fn strato() -> Edifice {
        let h = (2000.0 / 4000.0) * 0.9;
        Edifice::new(edifice_shape(EdificeKind::Strato), h, 40.0, 100.0, 100.0, 0.0)
    }

    fn shield() -> Edifice {
        let h = (5000.0 / 4000.0_f64).min(0.95) * 0.9;
        Edifice::new(edifice_shape(EdificeKind::Shield), h, 110.0, 100.0, 100.0, 0.0)
    }

    fn cinder() -> Edifice {
        let h = (600.0 / 4000.0) * 0.9;
        Edifice::new(edifice_shape(EdificeKind::Cinder), h, 40.0, 100.0, 100.0, 0.0)
    }

    /// The gap this lane exists to close, stated as a test on the *reference's*
    /// own expression: its summit dip has no floor at all. `H*(1-t)^1.6 -
    /// H*0.5*(1-t/0.16)` is strictly increasing outward from `t = 0`, so the
    /// single deepest point is dead centre — a V-notch, i.e. a crater painted
    /// on a cone.
    #[test]
    fn the_reference_summit_dip_is_a_point_not_a_floor() {
        let h = 0.45;
        let ref_add = |t: f64| {
            let mut a = h * (1.0 - t).powf(1.6);
            if t < 0.16 {
                a -= h * 0.5 * (1.0 - t / 0.16);
            }
            a
        };
        let centre = ref_add(0.0);
        for k in 1..16 {
            let t = k as f64 * 0.16 / 16.0;
            assert!(
                ref_add(t) > centre + 1e-9,
                "reference dip should rise immediately from the centre at t={t}"
            );
        }
    }

    /// The replacement: a genuinely flat floor of real extent out to `floor`,
    /// then a rise. Both halves matter — "flat out to `floor`" alone is
    /// vacuously true if `floor` is zero, which is exactly the degenerate
    /// point-bottomed shape this lane exists to remove.
    #[test]
    fn caldera_floor_is_flat_and_has_extent() {
        let r_cells = 40.0;
        for e in [cinder(), strato(), shield()] {
            let centre = e.add_at(0.0, 0.0, 0.0, 0.0);
            assert!(centre > 0.0, "floor must sit above the edifice base, got {centre}");
            for k in 0..=8 {
                let t = e.s.floor * k as f64 / 8.0;
                let v = e.add_at(t, 0.7, 3.0, 5.0);
                assert!((v - centre).abs() < 1e-12, "floor is not flat at t={t}: {v} vs {centre}");
            }
            // A floor narrower than two cells is not a floor, it is the
            // reference's point with extra arithmetic.
            assert!(
                2.0 * e.s.floor * r_cells >= 2.0,
                "floor spans only {} cells on a {r_cells}-cell radius",
                2.0 * e.s.floor * r_cells
            );
            assert!(
                e.add_at(e.s.floor * 1.4, 0.7, 3.0, 5.0) > centre + 1e-9,
                "the wall does not begin outside the floor"
            );
        }
    }

    /// The three morphologies are actually three shapes, and each one's
    /// summit steepness is ordered the way its geology says: a scoria cone is
    /// nearly straight-sided, a stratocone is strongly concave-up, a shield is
    /// level at the top.
    #[test]
    fn the_three_morphologies_are_distinct() {
        let d = 1e-4;
        let slope = |e: &Edifice, t: f64| ((e.cone_at(t + d) - e.cone_at(t)) / d).abs();
        let (c, st, sh) = (cinder(), strato(), shield());
        assert!(
            slope(&c, 0.0) < slope(&st, 0.0),
            "a scoria cone should be less peaked at the summit than a stratocone"
        );
        assert!(slope(&sh, 0.0) < slope(&c, 0.0), "a shield should be the flattest at the summit");
        // Scoria cones carry proportionally the largest summit crater, shields
        // the smallest relative to their own footprint.
        assert!(c.s.rim > st.s.rim, "a scoria cone's crater should be wider in proportion");
        assert!(st.s.rim > sh.s.rim, "a shield caldera should be the narrowest in proportion");
        assert!(st.s.depth > sh.s.depth, "a stratocone collapse should be deeper than a shield's");
    }

    /// A shield's flanks are gentler than a stratocone's at the same height,
    /// and it tapers into the plain instead of ending in a scarp. The taper is
    /// what needs `b > 1`: at `b = 1` the profile `h(1-t^2)` reaches the toe at
    /// full slope.
    #[test]
    fn shield_flanks_are_gentler_and_taper() {
        let h = 0.45;
        let st = Edifice::new(edifice_shape(EdificeKind::Strato), h, 40.0, 0.0, 0.0, 0.0);
        let sh = Edifice::new(edifice_shape(EdificeKind::Shield), h, 40.0, 0.0, 0.0, 0.0);
        let d = 1e-4;
        let max_slope = |e: &Edifice| {
            (0..1000)
                .map(|k| {
                    let t = k as f64 / 1000.0;
                    ((e.cone_at(t + d) - e.cone_at(t)) / d).abs()
                })
                .fold(0.0f64, f64::max)
        };
        assert!(
            max_slope(&sh) < max_slope(&st),
            "a shield should be gentler everywhere than a stratocone of the same height: {} vs {}",
            max_slope(&sh),
            max_slope(&st)
        );
        // Measured against the shield's own steepest flank, so the assertion is
        // about shape rather than scale: at `b = 1` the toe *is* the steepest
        // point (ratio 1.0); at `b = 1.4` it is a few percent of it.
        let toe = ((sh.cone_at(1.0) - sh.cone_at(1.0 - d)) / d).abs();
        assert!(
            toe < 0.15 * max_slope(&sh),
            "a shield should taper into the plain, toe slope {toe} against max {}",
            max_slope(&sh)
        );
    }

    /// Gullies are spaced by arc length, not by a fixed count, so a small cone
    /// and a large shield are both dissected at a legible scale.
    #[test]
    fn gullies_are_spaced_by_arc_length() {
        for r in [6.0, 12.0, 40.0, 90.0] {
            let e = Edifice::new(edifice_shape(EdificeKind::Strato), 0.45, r, 0.0, 0.0, 0.0);
            // Arc length between gullies at mid-flank, in cells.
            let spacing = std::f64::consts::PI * r / e.gullies as f64;
            assert!(
                (3.0..=6.0).contains(&spacing),
                "gully spacing at r={r} is {spacing} cells, outside the legible band"
            );
        }
        // Outside that band the clamp takes over, and its two bounds are the
        // whole behaviour there: a 2-cell cone still gets a full ring rather
        // than three stripes, and a 200-cell shield stops adding gullies it
        // cannot resolve.
        let at = |r: f64| Edifice::new(edifice_shape(EdificeKind::Strato), 0.45, r, 0.0, 0.0, 0.0).gullies;
        assert_eq!(at(2.0), 6, "tiny edifices should floor at a full ring of gullies");
        assert_eq!(at(200.0), 48, "huge edifices should cap rather than sub-divide per cell");
    }

    /// The floor sits `depth` of the edifice height below the rim, and the wall
    /// climbs monotonically from one to the other.
    #[test]
    fn caldera_wall_climbs_from_floor_to_rim() {
        let e = strato();
        let floor = e.add_at(0.0, 0.0, 0.0, 0.0);
        let rim = e.cone_at(e.s.rim);
        assert!(
            (rim - floor - e.h * e.s.depth).abs() < 1e-12,
            "collapse depth wrong: rim {rim}, floor {floor}, h {}",
            e.h
        );
        let mut prev = floor;
        for k in 1..=40 {
            let t = e.s.rim * k as f64 / 40.0;
            let v = e.add_at(t, 0.0, 0.0, 0.0);
            assert!(v >= prev - 1e-12, "wall dips at t={t}: {v} < {prev}");
            prev = v;
        }
        assert!((prev - rim).abs() < 1e-9, "wall does not reach the rim: {prev} vs {rim}");
    }

    /// The wall meets the outer flank with matching value *and* slope — the
    /// smoothstep's whole job. A crease here would read as a painted-on ring.
    #[test]
    fn caldera_rim_is_smooth() {
        let e = strato();
        let d = 1e-5;
        let inner = (e.add_at(e.s.rim - d, 0.0, 0.0, 0.0) - e.add_at(e.s.rim - 2.0 * d, 0.0, 0.0, 0.0)) / d;
        let outer = (e.cone_at(e.s.rim + 2.0 * d) - e.cone_at(e.s.rim + d)) / d;
        // Inner slope is ~0 (smoothstep flattens at 1); the cone is descending.
        assert!(inner.abs() < 1e-3, "wall arrives at the rim with slope {inner}");
        assert!(outer < 0.0, "flank should descend outward, got {outer}");
    }

    /// A shield has a level summit; a stratocone does not. This is the whole
    /// point of the two-exponent family — with `a = 1` a flat summit is
    /// unreachable at any `b`.
    #[test]
    fn shield_summit_is_level_and_stratocone_summit_is_not() {
        let d = 1e-4;
        let sh = shield();
        let st = strato();
        let sh_slope = ((sh.cone_at(d) - sh.cone_at(0.0)) / d).abs();
        let st_slope = ((st.cone_at(d) - st.cone_at(0.0)) / d).abs();
        assert!(sh_slope < 1e-3, "shield summit should be level, slope {sh_slope}");
        assert!(st_slope > 0.5, "stratocone summit should be steep, slope {st_slope}");
    }

    /// The stratocone is steeper at the summit than the reference's one cone,
    /// and gentler at the base: that is what "concave-up" buys, and it is what
    /// distinguishes a stratovolcano from the reference's single shape.
    #[test]
    fn stratocone_is_more_concave_than_the_reference_cone() {
        let e = strato();
        let refc = |t: f64| e.h * (1.0 - t).powf(1.6);
        let d = 1e-4;
        let slope = |f: &dyn Fn(f64) -> f64, t: f64| ((f(t + d) - f(t)) / d).abs();
        let near = 0.30;
        let far = 0.80;
        assert!(
            slope(&|t| e.cone_at(t), near) > slope(&refc, near),
            "stratocone should be steeper than the reference cone near the summit"
        );
        assert!(
            slope(&|t| e.cone_at(t), far) < slope(&refc, far),
            "stratocone should be gentler than the reference cone on the apron"
        );
    }

    /// Age rounds the profile toward a straight cone, the same direction the
    /// reference's `1.6 - age*0.8` moves in, and never past it — below `b = 1`
    /// the toe slope diverges.
    #[test]
    fn age_rounds_the_profile_but_never_past_a_straight_cone() {
        let s = edifice_shape(EdificeKind::Strato);
        let fresh = Edifice::new(s, 0.45, 40.0, 0.0, 0.0, 0.0);
        let old = Edifice::new(s, 0.45, 40.0, 0.0, 0.0, 1.0);
        assert!(old.b < fresh.b, "age should lower the summit exponent");
        assert!(old.b > 1.0, "age must never push b to or below 1.0, got {}", old.b);
        assert!((fresh.b - s.b).abs() < 1e-12);
    }

    /// The setting picks the morphology; the size roll only picks the class
    /// within it. Same roll, three settings, three different answers.
    #[test]
    fn edifice_kind_follows_the_setting() {
        let mid = 0.80;
        assert_eq!(edifice_kind(VolcanicSetting::Arc, mid), EdificeKind::Strato);
        assert_eq!(edifice_kind(VolcanicSetting::Rift, mid), EdificeKind::Shield);
        assert_eq!(edifice_kind(VolcanicSetting::Hotspot, mid), EdificeKind::Shield);
        assert_eq!(edifice_kind(VolcanicSetting::Unclassified, mid), EdificeKind::Strato);
        // Hotspots are shields at every size; arcs and rifts build scoria cones
        // at the small end.
        for roll in [0.0, 0.35, 0.69, 0.70, 0.94, 0.95, 0.999] {
            assert_eq!(edifice_kind(VolcanicSetting::Hotspot, roll), EdificeKind::Shield);
        }
        assert_eq!(edifice_kind(VolcanicSetting::Arc, 0.69), EdificeKind::Cinder);
        assert_eq!(edifice_kind(VolcanicSetting::Rift, 0.69), EdificeKind::Cinder);
        assert_eq!(edifice_kind(VolcanicSetting::Unclassified, 0.96), EdificeKind::Shield);
    }

    /// Flank relief is exactly zero inside the depression and at the toe, so it
    /// can neither fill the caldera nor leave a rough edge where the edifice
    /// meets the surrounding terrain.
    #[test]
    fn flank_relief_vanishes_at_the_rim_and_the_toe() {
        let e = strato();
        assert!(
            (e.add_at(1.0, 0.4, 7.0, 11.0) - e.cone_at(1.0)).abs() < 1e-12,
            "relief should vanish at the toe"
        );
        for k in 0..=6 {
            let t = e.s.rim * k as f64 / 6.0;
            let residual = e.add_at(t, 0.4, 7.0, 11.0) - e.add_at(t, 2.9, 41.0, 3.0);
            assert!(residual.abs() < 1e-12, "relief leaked inside the caldera at t={t}");
        }
    }

    /// The relief is real, signed, and structured around the edifice rather
    /// than a constant offset — this is the assertion that would catch the
    /// silently-empty case the project has been bitten by four times.
    #[test]
    fn flank_relief_is_present_and_structured() {
        let e = strato();
        let t = 0.6;
        let n = 256;
        let mut vals = Vec::with_capacity(n);
        for k in 0..n {
            let theta = -std::f64::consts::PI + 2.0 * std::f64::consts::PI * k as f64 / n as f64;
            let x = 100.0 + 40.0 * t * theta.cos();
            let y = 100.0 + 40.0 * t * theta.sin();
            vals.push(e.add_at(t, theta, x, y) - e.cone_at(t));
        }
        let max = vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let min = vals.iter().cloned().fold(f64::INFINITY, f64::min);
        assert!(max > 1e-4, "relief has no positive lobe: max {max}");
        assert!(min < -1e-4, "relief has no incision: min {min}");
        let crossings = vals.windows(2).filter(|w| (w[0] < 0.0) != (w[1] < 0.0)).count();
        assert!(
            crossings >= 6,
            "relief around the flank is not structured (only {crossings} sign changes)"
        );
    }

    /// Both relief terms are live, isolated from each other.
    ///
    /// `add_at`'s gully term is a function of `(t, theta)` alone and its fbm
    /// term of `(x, y)` alone, so holding one pair fixed while varying the other
    /// measures exactly one of them. Physically the two pairs move together on a
    /// real edifice; here they are separable, which is what lets a zeroed
    /// frequency or a zeroed blend weight fail loudly instead of hiding behind
    /// the other term.
    #[test]
    fn both_relief_scales_are_live() {
        for e in [cinder(), strato(), shield()] {
            let t = 0.6;
            // Same cell, two bearings a half-gully apart: only the gully term moves.
            let half = std::f64::consts::PI / e.gullies as f64;
            let angular =
                (e.add_at(t, 0.3 + half, 100.0, 100.0) - e.add_at(t, 0.3, 100.0, 100.0)).abs();
            assert!(angular > 1e-6, "the gully term is dead: angular variation {angular}");
            // Same (t, theta), two cells: only the fbm term moves.
            let spatial = (e.add_at(t, 0.3, 100.0, 100.0) - e.add_at(t, 0.3, 137.0, 91.0)).abs();
            assert!(spatial > 1e-6, "the fbm term is dead: spatial variation {spatial}");
            // Same bearing and cell, two radii, with the mid-flank envelope
            // divided out so only the gully term's *radial* axis is left. Drop
            // that axis and gullies become perfectly straight radial stripes.
            let relief_over_envelope = |t: f64| {
                let u = (t - e.s.rim) / (1.0 - e.s.rim);
                (e.add_at(t, 0.3, 100.0, 100.0) - e.cone_at(t)) / (4.0 * u * (1.0 - u))
            };
            let radial = (relief_over_envelope(0.75) - relief_over_envelope(0.45)).abs();
            assert!(radial > 1e-6, "gullies do not vary down the flank: {radial}");
        }
    }

    /// Two edifices at different centres get different relief. The salt is
    /// derived from the centre — deliberately, rather than from an RNG draw —
    /// so a whole volcanic province does not come out stamped from one mould.
    #[test]
    fn relief_is_decorrelated_by_edifice_centre() {
        let s = edifice_shape(EdificeKind::Strato);
        let base = Edifice::new(s, 0.45, 40.0, 100.0, 100.0, 0.0);
        let moved_x = Edifice::new(s, 0.45, 40.0, 143.0, 100.0, 0.0);
        let moved_y = Edifice::new(s, 0.45, 40.0, 100.0, 167.0, 0.0);
        let sample = |e: &Edifice| e.add_at(0.6, 0.3, 100.0, 100.0);
        assert_ne!(sample(&base), sample(&moved_x), "the salt ignores the centre's x");
        assert_ne!(sample(&base), sample(&moved_y), "the salt ignores the centre's y");
    }

    /// The setting reaches the stamp through `place_sized_volcano`, on the same
    /// RNG stream: identical seed and placement, two settings, two fields.
    #[test]
    fn setting_changes_what_place_sized_volcano_stamps() {
        let (gw, gh) = (64usize, 64usize);
        let stamp = |setting| {
            let mut field = vec![0f32; gw * gh];
            let mut volc = vec![0f32; gw * gh];
            let mut rng = Mulberry32::new(7);
            place_sized_volcano(
                gw,
                gh,
                &mut field,
                &mut volc,
                800.0,
                4000.0,
                32.0,
                32.0,
                &mut rng,
                0.1,
                setting,
                EdificeModel::Morphological,
            );
            (field, volc)
        };
        let (arc, arc_v) = stamp(VolcanicSetting::Arc);
        let (hot, hot_v) = stamp(VolcanicSetting::Hotspot);
        assert_eq!(arc_v, hot_v, "the volcanic-field mark depends on placement only");
        assert!(arc.iter().any(|&v| v > 0.0), "arc stamp wrote nothing");
        assert_ne!(arc, hot, "arc and hotspot should build different edifices");
    }
}

/// `stampOneVolcano()` (reference HTML lines 3466-3473): radial cone with
/// an optional caldera dip (a summit depression once the volcano is tall
/// enough), and an age-damped volcanic-field mark for later biome/texture
/// use.
///
/// `shape` is `None` for [`EdificeModel::Reference`] — the JS code below runs
/// untouched — and `Some` for the morphological model, which replaces the
/// per-cell `add` and nothing else. The bounding box, the `d > R` cut, the
/// rounding-then-clamp order and the `volcanic_field` mark are shared: that
/// mark is `(1-t)*(1-age)`, a function of placement alone, so it is
/// bit-identical under both models by construction.
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
    shape: Option<EdificeShape>,
) {
    let h = (height_m / peak_m) * 0.9 * (1.0 - age * 0.5);
    let r = rad_cells.max(2.0);
    let caldera = height_m > 1000.0;
    // The morphological model gives *every* edifice a summit depression, so it
    // does not consult `caldera`. That is not a loosened threshold, it is the
    // right morphology: a scoria cone always has a summit crater (the 70% class
    // the reference's 1 000 m gate leaves as smooth points), a stratocone
    // usually does, and a shield has a summit pit. The gate stays only on the
    // reference path, where it is `stampOneVolcano`'s own.
    let edifice = shape.map(|s| Edifice::new(s, h, r, cx, cy, age));
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
            let add = match &edifice {
                None => {
                    let mut add = h * (1.0 - t).powf(1.6 - age * 0.8);
                    if caldera && t < 0.16 {
                        add -= h * 0.5 * (1.0 - t / 0.16);
                    }
                    add
                }
                // `js_atan2`, not `f64::atan2`, for the same reason the rest of
                // this crate uses it: it is a vendored implementation, so a
                // seeded world renders identically on every platform rather
                // than tracking whatever libm the host shipped.
                Some(e) => e.add_at(t, js_atan2(dy, dx), x as f64, y as f64),
            };
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
///
/// The size roll `r` does double duty under [`EdificeModel::Morphological`]:
/// it still picks the height/radius class exactly as the reference does, and
/// [`edifice_kind`] reads the *same already-drawn value* to pick the
/// morphology. No extra draw, so the stream is untouched either way.
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
    setting: VolcanicSetting,
    model: EdificeModel,
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
    let shape = match model {
        EdificeModel::Reference => None,
        EdificeModel::Morphological => Some(edifice_shape(edifice_kind(setting, r))),
    };
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
        shape,
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
    stamp_volcanoes_simple_shaped(
        gw,
        gh,
        seed,
        map_width_km,
        peak_m,
        boundary_mask,
        volc_count,
        volc_age,
        field,
        volcanic_field,
        EdificeModel::Reference,
    )
}

/// [`stamp_volcanoes_simple`] with the edifice model selectable.
///
/// Additive rather than a parameter on the existing entry point on purpose:
/// the reference-behaviour symbol the golden suites call stays exactly the
/// symbol they call, so no golden's call site has to be edited to add a
/// divergent mode — and nobody can re-baseline one by accident.
///
/// This mode has no province classification (that is what makes it "simple"),
/// so every placement is [`VolcanicSetting::Unclassified`] and the morphology
/// falls back to the reference's own size classes.
#[allow(clippy::too_many_arguments)]
pub fn stamp_volcanoes_simple_shaped(
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
    edifice: EdificeModel,
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
            VolcanicSetting::Unclassified,
            edifice,
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
///
/// # The transform leak this sign test cannot see
///
/// `stress_field` carries only the **normal** (convergence) component `C`.
/// The tangential component `S` is accumulated separately into `shearField`
/// and never reaches here — so a transform boundary, whose whole signature
/// is *small `|C|`, large `|S|`*, is invisible to a test on the sign of `C`.
/// Worse, `stress_field` is Gaussian-blurred and max-normalized before this
/// runs, so a transform cell inherits its neighbours' convergent stress and
/// reads as a healthy `s > 0.05`. It then lands in `conv` and becomes a
/// volcanic-arc site.
///
/// Measured on a 256x160 grid over 12 seeds (`volcano_transform_boundaries.rs`,
/// `arc_and_rift_pools_are_polluted_by_transform_cells`): of 11 248 cells in
/// the `conv` pool, 3 863 (34%) are cells this crate's own `classify_boundary`
/// types as `btype::TRANSFORM`; the `div` pool is 32% transform. Transform
/// margins are not a major volcanic environment — they produce earthquakes,
/// fault scarps and offset drainage, not stratocones — so a third of every
/// arc and rift is geologically in the wrong place.
///
/// `boundary_type` is the fix, and it is **opt-in**: pass `None` and this is
/// bit-for-bit the reference's own two-pool split (what
/// `golden_parity_volc_provinces.rs` pins). Pass `Some(&stress.boundary_type)`
/// — the field `compute_stress` already returns beside the mask, computed from
/// the raw *unblurred* per-edge `C`/`S` where the shear test is still
/// meaningful — and shear-dominant cells are dropped from both pools.
fn classify_boundaries(
    boundary_mask: &[u8],
    stress_field: &[f32],
    boundary_type: Option<&[u8]>,
) -> (Vec<usize>, Vec<usize>) {
    let mut conv = Vec::new();
    let mut div = Vec::new();
    for i in 0..boundary_mask.len() {
        if boundary_mask[i] == 0 {
            continue;
        }
        // Opt-in only: `None` keeps the reference's exact pools.
        if boundary_type.is_some_and(|bt| bt[i] == btype::TRANSFORM) {
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
    edifice: EdificeModel,
) {
    // The province `kind` the reference draws and then uses only for placement.
    // This is the whole of "the volcanic setting the placement lane classifies"
    // — it already existed, one frame up the stack, and was discarded here.
    let setting = match kind {
        "arc" => VolcanicSetting::Arc,
        "rift" => VolcanicSetting::Rift,
        _ => VolcanicSetting::Hotspot,
    };
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
            place_sized_volcano(
                gw,
                gh,
                field,
                volcanic_field,
                map_width_km,
                peak_m,
                x,
                y,
                rng,
                age,
                setting,
                edifice,
            );
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
                setting,
                edifice,
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
        place_sized_volcano(
            gw,
            gh,
            field,
            volcanic_field,
            map_width_km,
            peak_m,
            x + jx,
            y + jy,
            rng,
            age,
            setting,
            edifice,
        );
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
        place_sized_volcano(
            gw,
            gh,
            field,
            volcanic_field,
            map_width_km,
            peak_m,
            bx + jx,
            by + jy,
            rng,
            age,
            setting,
            edifice,
        );
        pc += 1;
    }
}

/// `stampVolcanoesProvinces()` (reference HTML lines 3540-3556): the JS
/// default (`state.volc.provinces` = `true`) — clusters volcanoes into a
/// handful of provinces (75% arc/subduction, 15% rift, 10% hotspot chain)
/// along plate boundaries rather than dusting them uniformly the way
/// `stamp_volcanoes_simple` does.
///
/// The three settings are already geologically distinct here, and the split
/// is the reference's own: **arc** hugs the convergent pool, **rift** the
/// divergent pool, and **hotspot** takes no boundary at all — it seeds at a
/// uniformly random cell and lays an age-progressive chain along the local
/// plate's drift vector, so a volcano genuinely does not imply a plate
/// boundary here. What was missing is the fourth setting: **transform
/// margins, which are not a volcanic environment and were silently feeding
/// both the arc and rift pools.** `boundary_type` excludes them; see
/// `classify_boundaries` for the measurement and for why `stress_field`
/// alone cannot detect them. `None` reproduces the reference exactly.
#[allow(clippy::too_many_arguments)]
pub fn stamp_volcanoes_provinces(
    gw: usize,
    gh: usize,
    seed: u32,
    map_width_km: f64,
    peak_m: f64,
    boundary_mask: &[u8],
    stress_field: &[f32],
    boundary_type: Option<&[u8]>,
    plate_id: &[u16],
    plates: &[Plate],
    volc_count: i32,
    volc_age: f64,
    field: &mut [f32],
    volcanic_field: &mut [f32],
) {
    stamp_volcanoes_provinces_shaped(
        gw,
        gh,
        seed,
        map_width_km,
        peak_m,
        boundary_mask,
        stress_field,
        boundary_type,
        plate_id,
        plates,
        volc_count,
        volc_age,
        field,
        volcanic_field,
        EdificeModel::Reference,
    )
}

/// [`stamp_volcanoes_provinces`] with the edifice model selectable.
///
/// Additive rather than a parameter on the existing entry point, for the reason
/// [`stamp_volcanoes_simple_shaped`] gives.
///
/// This is where the setting actually reaches the edifice: the province `kind`
/// this function already rolls is handed to `place_province_volcanoes`, which
/// (under [`EdificeModel::Morphological`]) now passes it on to
/// [`edifice_kind`] instead of dropping it. Arc provinces build stratocones,
/// rift and hotspot provinces build shields, and the small end of every
/// province builds scoria cones — none of it from a fresh random roll.
#[allow(clippy::too_many_arguments)]
pub fn stamp_volcanoes_provinces_shaped(
    gw: usize,
    gh: usize,
    seed: u32,
    map_width_km: f64,
    peak_m: f64,
    boundary_mask: &[u8],
    stress_field: &[f32],
    boundary_type: Option<&[u8]>,
    plate_id: &[u16],
    plates: &[Plate],
    volc_count: i32,
    volc_age: f64,
    field: &mut [f32],
    volcanic_field: &mut [f32],
    edifice: EdificeModel,
) {
    let mut rng = Mulberry32::new(seed ^ 0x5bf03635);
    let (conv, div) = classify_boundaries(boundary_mask, stress_field, boundary_type);
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
            edifice,
        );
        remaining -= sub;
    }
}

/// Characteristic width of each of a crater's four topographic features, as a
/// fraction of its **diameter**. These are *read off* the profile
/// `stamp_one_crater` already draws, not chosen — which is why they carry no
/// calibration of their own:
///
/// | feature | where it lives in the profile below | width |
/// |---|---|---|
/// | bowl | `1 - t²` over `t < 1` | `2r` = `1.00 D` |
/// | rim | the annulus `0.85 < t < 1.25` | `0.40 r` = `0.20 D` |
/// | central peak | `t < 0.18` | `0.36 r` = `0.18 D` |
/// | basin rings | `cos(3πt)`, 1.5 cycles over `r` | `0.67 r` = `0.33 D` |
///
/// Diffusion attenuates a feature at a rate set by *its own* length, not the
/// crater's, so `tau` scales as `1/(frac·D)²` — the rim, a fifth of the
/// diameter wide, ages **25x** faster than the bowl it encloses. That is the
/// familiar sequence: a degraded crater is a rimless shallow depression long
/// before it is gone.
const CRATER_FEATURE_BOWL: f64 = 1.0;
const CRATER_FEATURE_RIM: f64 = 0.2;
const CRATER_FEATURE_PEAK: f64 = 0.18;
const CRATER_FEATURE_RINGS: f64 = 1.0 / 3.0;

/// The **shock aureole**'s characteristic width, in the same units as the four
/// topographic fractions above: a multiple of the crater's diameter.
///
/// `impact_field` is not topography. It marks shocked rock and impact melt for
/// the lithology stage, and that signature is emplaced *in the rock* rather
/// than *in the surface* — a crater whose relief has relaxed away still leaves
/// shatter cones and a melt sheet. **The shock record therefore outlives the
/// landform**, and the owner's third ruling of 2026-09-02 is that it should
/// fade rather than stay pristine, not that it should fade at the same rate.
///
/// The rate is expressed the only way this file already knows how — a feature
/// length fed through the same `1/(frac·D)²` law — so it introduces no second
/// mechanism and no second calibration. `2.0` is the shocked zone's own extent:
/// the continuous ejecta blanket and its shock aureole reach roughly one crater
/// *radius* beyond the rim, so the affected patch is about **twice** the
/// crater's diameter across, against the bowl's `1.00 D`. That makes the shock
/// signature's timescale exactly **4x** the bowl's, which is the "gentler than
/// the topographic one" the ruling asks for.
///
/// **This is a ratio chosen inside the existing family, not a fitted number.**
/// The 2:1 extent is the standard order-of-magnitude figure for a continuous
/// ejecta blanket; nothing here claims a shock-annealing rate was measured, and
/// no source in this repository has been checked for one. What is defensible is
/// the *sign and the shape*: monotone decay, slower than the relief, on the
/// same diffusive law. Read `DECISIONS.md` §7l-ii before retuning it.
const CRATER_FEATURE_SHOCK: f64 = 2.0;

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
///
/// # `tau`: geological degradation, and how it composes with `age`
///
/// `tau` is [`crater_degradation_tau`] for this crater — `None` on the
/// reference path, where every preservation factor is exactly `1.0` and the
/// arithmetic is therefore bit-identical to what it was (multiplying an `f64`
/// by `1.0` is exact, and left-associativity keeps the operand order).
///
/// The two terms **multiply**, and they are not the same quantity:
///
/// - `age` (`CraterParams::age`, 0-1) is an authoring control. It shallows the
///   whole population uniformly, regardless of size, and it is what the user
///   reaches for when they want a worn-looking map. Keeping it as an
///   independent multiplier means it does exactly what it did before, at every
///   setting, under both models.
/// - `tau` is physics, and it is *size-dependent by construction*.
///
/// Folding `age` into `tau` instead — scaling the elapsed time by it, say —
/// was rejected: `age = 0` would then mean "no degradation at all", so a
/// cosmetic-looking slider would silently switch off the physical model, and
/// the parameter's meaning would depend on `crater.physical_model`. A
/// multiplier keeps each term answering exactly one question.
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
    tau: Option<f64>,
) {
    // Preservation of a feature `frac` of the crater's diameter wide, after
    // `tau`. Exactly 1.0 -- and so exactly a no-op -- on the reference path.
    let keep = |frac: f64| tau.map_or(1.0, |t| (-t / (frac * frac)).exp());
    let bowl_keep = keep(CRATER_FEATURE_BOWL);
    let rim_keep = keep(CRATER_FEATURE_RIM);
    let peak_keep = keep(CRATER_FEATURE_PEAK);
    let rings_keep = keep(CRATER_FEATURE_RINGS);
    // Owner ruling 3, 2026-09-02. Gated by construction: `tau` is `None` on the
    // reference path, so this is exactly `1.0` there and `(1 - t) * 1.0` is the
    // bit-identical value it always was. See `CRATER_FEATURE_SHOCK`.
    let shock_keep = keep(CRATER_FEATURE_SHOCK);

    let r = rad_cells.max(1.5);
    let depth = (0.02 + rad_cells * 0.004).min(0.4) * (1.0 - age * 0.8);
    let rim = depth * 0.25 * rim_keep;
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
                add_rounded(field, i, -depth * bowl_keep * (1.0 - t * t));
                if large && t < 0.18 {
                    add_rounded(field, i, depth * 0.5 * peak_keep * (1.0 - t / 0.18));
                }
            }
            if t > 0.85 && t < 1.25 {
                let rt = 1.0 - (t - 1.05).abs() / 0.2;
                if rt > 0.0 {
                    add_rounded(field, i, rim * rt);
                }
            }
            if basin && t < 1.0 {
                add_rounded(
                    field,
                    i,
                    (t * std::f64::consts::PI * 3.0).cos() * depth * 0.08 * rings_keep,
                );
            }
            let stored = field[i] as f64;
            field[i] = if stored < 0.0 {
                0.0
            } else if stored > 1.0 {
                1.0
            } else {
                field[i]
            };
            // `impact_field` degrades too, but four times slower than the bowl
            // -- the shock record outlives the landform (`CRATER_FEATURE_SHOCK`,
            // `DECISIONS.md` §7l-ii). Owner ruling 3, 2026-09-02, which reverses
            // the "deliberately NOT degraded" note that stood here.
            //
            // **The gate is `tau`, and its real blast radius is smaller than
            // this comment first claimed.** It said damping this field "moves
            // lithology, and through it biomes, carrying capacity, settlement
            // placement, roads and sea routes -- sixteen `cartalith-civ` golden
            // suites", and called that measured. Corrected 2026-09-02:
            // `impact_field` reaches the civ layer nowhere. `grep -rn
            // impact_field crates/cartalith-civ/src/` returns nothing,
            // `build_lithology` takes no impact field, and with the gate
            // deliberately removed `cargo test -p cartalith-civ` passes 27 of 27
            // binaries. The sixteen-suite figure belongs to the crater
            // HEIGHT-field change, where it was real; it was carried across to
            // the wrong field. See `DECISIONS.md` §7l-ii.
            //
            // What the gate genuinely protects: this field is saved
            // (`cartalith-io`) and pinned by `golden_parity_volc_craters`' two
            // reference-extracted `expected_impact` arrays. `tau` is `None`
            // whenever `crater.physical_model` is false, which is what
            // `WorldParams::defaults` is, so `shock_keep` is exactly `1.0` there
            // and those fixtures are bit-identical.
            //
            // The `max` still resolves overlaps in favour of the stronger
            // signature, which is now "fresher or bigger" rather than just
            // "closer to a centre" -- a young crater overprints an ancient one.
            if d < r {
                let vv = ((1.0 - t) * shock_keep).max(impact_field[i] as f64);
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
/// Terrestrial crater production rate for `D >= 20 km`, craters km⁻² yr⁻¹.
///
/// Grieve & Shoemaker (1994), *The record of past impacts on Earth*, in
/// *Hazards Due to Comets and Asteroids*, pp. 417-462; reproduced in French
/// (1998), *Traces of Catastrophe*. Stated there as **(5.6 ± 2.8) x 10⁻¹⁵**,
/// and the uncertainty is not decorative — it is ±50%.
///
/// **This is the high end of the published estimates**, recorded here so nobody
/// mistakes it for a consensus: Hughes (1981) gives (2.6 ± 0.9) x 10⁻¹⁵ and
/// Hughes (2000) (3.46 ± 0.30) x 10⁻¹⁵, roughly half. Halving this constant is
/// a defensible reading of the literature, not a bug.
pub const CRATER_RATE_D20_PER_KM2_YR: f64 = 5.6e-15;

/// The diameter that rate is quoted for, km.
pub const CRATER_RATE_REF_D_KM: f64 = 20.0;

/// Default **geological** surface exposure age, in millions of years.
///
/// # This is not the civilisation Timeline, and must never be wired to it
///
/// Cartalith carries two clocks that differ by six orders of magnitude and are
/// not convertible:
///
/// - the **civilisation** Timeline (`TIMELINE_SCOPE.md`, the year cursor,
///   `civ_apply_recovery`) runs in years to millennia;
/// - **crater accumulation** runs in 10⁴-10⁹ years.
///
/// A settlement founded 300 years ago and a cratered surface 100 Myr old are
/// not points on one axis. Reading the year cursor here would make a
/// civilisation's rise change the crater count, which is nonsense.
///
/// Distinct again from [`CraterParams::age`](../cartalith_engine), which is a
/// **morphological** 0-1 degradation term controlling how worn each crater
/// looks — not how long the surface has been collecting them.
///
/// 100 Myr is chosen because at the app's own default map (800 km wide,
/// 2048x1311) it yields ~92 craters against the reference's default `count` of
/// 100 — so the physical model lands almost exactly where the hand-tuned
/// default already sat. That is a calibration convenience, not a claim about
/// any particular world's history.
pub const CRATER_SURFACE_AGE_MYR: f64 = 100.0;

/// Smallest crater worth generating, in **cells** of diameter.
///
/// Below roughly two cells a crater cannot be resolved by the heightfield at
/// all, so generating it costs time and changes nothing. This is what keeps a
/// 40 000 km world tractable: at 2048 cells one cell is 19.5 km, so the
/// smallest *resolvable* crater there is ~39 km across, not 1 km — and the
/// `D^-2` law means raising the floor that far removes the overwhelming
/// majority of the population. Physics and performance agree here.
pub const CRATER_MIN_DIAM_CELLS: f64 = 2.0;

/// Upper bound on the generated crater count. A guard, not a model: with the
/// resolution-aware floor below, the physical count stays well under this for
/// every configuration Cartalith exposes. Same motivation as
/// `TERRAIN_DETAIL_MAX_K`'s cap — sane worst-case cost.
pub const CRATER_AUTO_MAX: i32 = 4000;

/// The smallest crater diameter this map can actually resolve, km.
pub fn crater_min_diameter_km(map_width_km: f64, gw: usize) -> f64 {
    let cell_km = if map_width_km > 0.0 && gw > 0 { map_width_km / gw as f64 } else { 800.0 / 2048.0 };
    CRATER_D_MIN_KM.max(CRATER_MIN_DIAM_CELLS * cell_km)
}

/// Expected number of craters on this map: `lambda = R20 * T * A * (20/Dmin)^b * I`.
///
/// Each factor is a separate physical quantity, which is the whole point of the
/// rewrite — the reference conflated all of them into one absolute count:
///
/// - `R20` — production rate for `D >= 20 km` ([`CRATER_RATE_D20_PER_KM2_YR`]);
/// - `T` — geological surface age in years ([`CRATER_SURFACE_AGE_MYR`]);
/// - `A` — real map area in km², so a 5 km region and a 40 000 km world differ
///   by the 64 000 000x they actually differ by;
/// - `(20/Dmin)^b` — the cumulative size-frequency law extrapolated from the
///   20 km reference diameter down to the smallest *resolvable* crater;
/// - `I` — the user slider as an intensity multiplier, `count/100`, so the
///   reference's own default of 100 means "physically calibrated" and 200 means
///   twice that.
pub fn crater_lambda(
    crater_count: i32,
    area_km2: f64,
    d_min_km: f64,
    surface_age_myr: f64,
) -> f64 {
    if crater_count <= 0 || !(area_km2 > 0.0) || !(d_min_km > 0.0) || !(surface_age_myr > 0.0) {
        return 0.0;
    }
    let intensity = crater_count as f64 / 100.0;
    let years = surface_age_myr * 1.0e6;
    let size_factor = (CRATER_RATE_REF_D_KM / d_min_km).powf(CRATER_SFD_EXPONENT);
    let lambda =
        CRATER_RATE_D20_PER_KM2_YR * years * area_km2 * size_factor * intensity;
    lambda.min(CRATER_AUTO_MAX as f64)
}

/// A Poisson draw with mean `lambda`, from this engine's own `Mulberry32`.
///
/// Impacts are independent events in space and time, so the count on any given
/// map is a random variable rather than a constant — two worlds of the same
/// size and intensity should not contain the identical number of craters.
///
/// Knuth's product method below `KNUTH_MAX`; above it `exp(-lambda)` underflows
/// `f64`, so a normal approximation (`N(lambda, sqrt(lambda))`, Box-Muller off
/// the same stream) takes over. The crossover is far above any lambda
/// `crater_lambda`'s cap can produce, so the approximation is a guard rather
/// than a path the shipped defaults take.
pub fn poisson_sample(rng: &mut Mulberry32, lambda: f64) -> i32 {
    const KNUTH_MAX: f64 = 500.0;
    if lambda <= 0.0 {
        return 0;
    }
    if lambda < KNUTH_MAX {
        let l = (-lambda).exp();
        let mut k = 0i32;
        let mut p = 1.0f64;
        loop {
            p *= rng.next_f64();
            if p <= l {
                return k;
            }
            k += 1;
            if k > 100_000 {
                return k; // unreachable in practice; refuses to spin forever
            }
        }
    }
    let u1 = rng.next_f64().max(f64::MIN_POSITIVE);
    let u2 = rng.next_f64();
    let z = (-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos();
    (lambda + z * lambda.sqrt()).round().max(0.0) as i32
}

/// Cumulative size-frequency exponent, `N(>=D) ∝ D^-b`.
///
/// `2.0` is the terrestrial large-crater value (Grieve & Dence 1979).
///
/// **Named rather than inlined because it is being used outside the range its
/// evidence covers, deliberately and on the owner's ruling.** The −2 slope is
/// established for `D >= 20 km`; this engine's population runs down to
/// `D = 1 km`, and Oetting et al. (2025) measure a slope near **2.85** for
/// craters ≤10 m, so the real exponent is not constant with size. A piecewise
/// law would be more faithful, but no source in the supplied research gives a
/// terrestrial exponent for the 1-20 km band, and inventing one would be worse
/// than extrapolating a measured one. Tune here when a source exists.
pub const CRATER_SFD_EXPONENT: f64 = 2.0;

/// Diameter bounds of the generated population, km. Chosen to span the
/// reference's own three radius bands (0.5-200 km radius) so the *visual* size
/// range is preserved while the *distribution* across it changes.
pub const CRATER_D_MIN_KM: f64 = 1.0;
pub const CRATER_D_MAX_KM: f64 = 400.0;

/// The hillslope diffusivity [`crater_degradation_tau`]'s half-life anchor was
/// calibrated at — the reference's own `state.erosion.diffuseD`, which is
/// `ErosionPassParams::off().diffuse_d` and `params::defaults()`'
/// `passes.diffuse_d` alike.
///
/// **It is a *reference point*, not a default.** `crater_degradation_tau` takes
/// the live `diffuse_d` and divides by this, so the ratio is exactly `1.0` at
/// the shipped configuration and the anchor is preserved bit for bit; move the
/// slider and craters move with it, linearly. Duplicated here rather than
/// imported because `cartalith-terrain` does not depend on `cartalith-engine`
/// (the dependency runs the other way) — `cartalith_engine`'s
/// `the_crater_anchor_matches_the_shipped_diffusivity` asserts the two agree, so
/// a change to either side fails loudly instead of silently re-calibrating
/// craters.
pub const CRATER_DEGRADATION_DIFFUSE_D_REF: f64 = 0.15;

/// Draw one crater diameter (km) from the truncated power-law size-frequency
/// distribution.
///
/// For a cumulative `N(>=D) ∝ D^-b` truncated to `[d_min, d_max]`, the inverse
/// CDF of a uniform `u` is
///
/// ```text
/// D = d_min * [ 1 - u * (1 - (d_min/d_max)^b) ] ^ (-1/b)
/// ```
///
/// which returns `d_min` at `u = 0` and exactly `d_max` at `u = 1`. This
/// replaces the reference's three flat bands (90% at 0.5-5 km radius, 9% at
/// 5-25, 1% at 25-200), which sampled *uniformly inside* each band and so
/// produced far too many large craters relative to small ones.
pub fn crater_diameter_km(u: f64, d_min: f64, d_max: f64, b: f64) -> f64 {
    if !(d_min > 0.0) || !(d_max > d_min) || !(b > 0.0) {
        return d_min.max(0.0);
    }
    let u = u.clamp(0.0, 1.0);
    let ratio_b = (d_min / d_max).powf(b);
    let denom = 1.0 - u * (1.0 - ratio_b);
    if denom <= 0.0 {
        return d_max;
    }
    (d_min * denom.powf(-1.0 / b)).min(d_max)
}

/// Dimensionless diffusive degradation state of a crater, `tau = kappa·T/D²`.
///
/// # Why two craters of the same age do not look the same age
///
/// Crater topography relaxes **diffusively**: material moves downslope at a
/// rate set by the local curvature, so relief decays as `exp(-t/t_diff)` with a
/// characteristic time
///
/// ```text
/// t_diff  ∝  L² / kappa
/// ```
///
/// The `L²` is the entire finding. At one diffusivity a 100 km crater's
/// timescale is **10 000x** a 1 km crater's, so a single surface of a single
/// age has its small craters erased and its large ones barely touched. That is
/// why Earth's crater record is a *preserved subset* rather than a census —
/// the point `DECISIONS.md` §7l left open when it built crater frequency and
/// size but not degradation.
///
/// It is also why the term this replaces was wrong in kind rather than in
/// calibration: `stamp_one_crater`'s `depth * (1 - age*0.8)` is linear in a
/// unitless age and carries no length at all, so a 1 km and a 100 km crater of
/// the same age came out equally fresh.
///
/// # `kappa` is the erosion diffusivity — **moving an erosion slider changes
/// how craters look**
///
/// Owner ruling 2, 2026-09-02: read the real diffusivity rather than keep a
/// private anchor. `hillslope_diffuse` (`cartalith-erosion`, `DECISIONS.md`
/// §7m) is the *same physics at a different scale*, so a world cannot coherently
/// hold two unrelated `kappa`s. `diffuse_d` is therefore an input here, and the
/// coupling runs one way: **turn `Diffusivity D` up and craters relax further at
/// the same surface age.** That is the intended consequence, not a leak — it is
/// called out here, in [`stamp_craters`], and in the parameter's own GUI label,
/// because a coupling nobody can see is worse than no coupling.
///
/// It reads the **raw** `diffuse_d`, not `hillslope_extent_scale`'s corrected
/// value. That correction exists to make a one-cell Laplacian mean the same
/// physical diffusion at any cell size (§7m); it is a discretisation fix for the
/// kernel, not a different `kappa`. This function already works in kilometres
/// and megayears, so it wants the physical quantity, which is the raw one.
/// It also reads it whether or not the hillslope *pass* is enabled: a
/// diffusivity is a property of the landscape, not of which buttons the user
/// pressed, and gating it would freeze craters whenever the pass was off.
///
/// ## The calibration is preserved exactly, and asserted
///
/// **No diffusivity is claimed as measured, and none was taken from a source.**
/// The scale is still pinned the way §7l pinned crater density — one free
/// constant chosen so the default lands where the hand-tuned default already
/// sat:
///
/// > at the default [`CRATER_DEGRADATION_DIFFUSE_D_REF`], a crater of
/// > [`CRATER_D_MIN_KM`], the smallest in the generated population, loses
/// > **half** its relief in one [`CRATER_SURFACE_AGE_MYR`].
///
/// which gives `tau = ln2 · (d/d_ref) · (T/T_ref) · (D_ref/D)²`. At the default
/// `d` the middle factor is exactly `1.0`, so this is **bit-identical** to the
/// private anchor it replaces — `the_default_diffusivity_reproduces_the_old_anchor`
/// asserts that, and `the_anchor_is_a_half_life_for_the_smallest_crater` still
/// asserts the half-life it encodes. Everything else — how a 3 km crater
/// compares to a 30 km one, how 400 Myr compares to 100 — follows from the `L²`
/// law alone.
///
/// The implied `kappa = ln2·D_ref²/T_ref` at that `d` is about **7e-3 m² yr⁻¹**,
/// which does land inside the order of magnitude usually quoted for terrestrial
/// hillslope diffusion (10⁻³-10⁻² m² yr⁻¹). Recorded as an order-of-magnitude
/// sanity check and nothing more: it was chosen for the look, not fitted, and
/// unlike §7l's citations that range has *not* been verified against a paper in
/// this repository. Do not quote this function as a measurement.
///
/// # Three clocks, still three
///
/// `surface_age_myr` here is the **geological** one — the same
/// [`CRATER_SURFACE_AGE_MYR`] that sets how many craters accumulated, now also
/// setting how long they have had to relax. That is the physically right
/// coupling: one exposure age governs both. It is *not* the civilisation
/// Timeline, and it is not the morphological `CraterParams::age`, which stays a
/// separate multiplier — see `stamp_one_crater`.
pub fn crater_degradation_tau(diam_km: f64, surface_age_myr: f64, diffuse_d: f64) -> f64 {
    if !(diam_km > 0.0) || !(surface_age_myr > 0.0) || !(diffuse_d > 0.0) {
        // A pristine surface (T = 0) degrades nothing, a crater with no
        // diameter has nothing to degrade, and a zero diffusivity transports no
        // material. All three are tau = 0, i.e. fully preserved -- NOT
        // "degradation off", which is `None` at the call site.
        return 0.0;
    }
    let ratio = CRATER_D_MIN_KM / diam_km;
    // `diffuse_d / d_ref` is written FIRST and is exactly 1.0 at the default, so
    // multiplying LN_2 by it is exact and the remaining operand order is
    // untouched -- that is what makes the default bit-identical to the private
    // anchor this replaces. Do not reorder these factors.
    std::f64::consts::LN_2
        * (diffuse_d / CRATER_DEGRADATION_DIFFUSE_D_REF)
        * (surface_age_myr / CRATER_SURFACE_AGE_MYR)
        * ratio
        * ratio
}

/// The realised crater count for this map: a Poisson draw about
/// [`crater_lambda`].
///
/// Draws on its own RNG stream, seeded distinctly from the stamper's, so the
/// count and the placement sequence are independent.
pub fn auto_crater_count(
    seed: u32,
    crater_count: i32,
    area_km2: f64,
    d_min_km: f64,
    surface_age_myr: f64,
) -> i32 {
    let lambda = crater_lambda(crater_count, area_km2, d_min_km, surface_age_myr);
    if lambda <= 0.0 {
        return 0;
    }
    let mut rng = Mulberry32::new(seed ^ 0x9E37_79B1);
    poisson_sample(&mut rng, lambda).min(CRATER_AUTO_MAX)
}

/// Stamp `crater_count` craters into `field`.
///
/// `physical` selects the size-frequency law:
///
/// - `false` — the reference's own three flat bands (90% at 0.5-5 km radius,
///   9% at 5-25, 1% at 25-200), sampled uniformly *within* each band. Kept
///   byte-identical so the import/inversion path and any A/B comparison still
///   have the original available.
/// - `true` — a truncated `D^-b` cumulative size-frequency law over
///   `[d_min_km, CRATER_D_MAX_KM]`. **This changes generated terrain and is the
///   shipped default**, on the owner's 2026-09-02 ruling; see `DECISIONS.md`
///   §7l. The band scheme produced far too many large craters relative to
///   small ones, because uniform-within-band ignores that crater frequency
///   rises as roughly the inverse square of diameter.
///
/// The `large`/`basin` morphology thresholds are keyed off the drawn radius in
/// both modes, so a crater of a given size looks the same either way — only how
/// often each size is drawn changes.
///
/// `physical` also selects **degradation** ([`crater_degradation_tau`]): under
/// the physical model each crater relaxes for `surface_age_myr` at a rate set
/// by its own diameter and by `diffuse_d`, so the small ones are worn away while
/// the large ones stay sharp — and the shock record in `impact_field` fades with
/// them, four times more slowly ([`CRATER_FEATURE_SHOCK`]). Both
/// `surface_age_myr` and `diffuse_d` are inert when `physical` is `false` — the
/// reference has no such term, and the reference path stays byte-identical.
///
/// `diffuse_d` is the **erosion** diffusivity (`ErosionPassParams::diffuse_d`,
/// `DECISIONS.md` §7m), read raw. One world, one `kappa`: raising it relaxes
/// craters further at the same surface age. Owner ruling 2, 2026-09-02.
#[allow(clippy::too_many_arguments)]
pub fn stamp_craters(
    gw: usize,
    gh: usize,
    seed: u32,
    map_width_km: f64,
    g: f64,
    crater_count: i32,
    crater_age: f64,
    physical: bool,
    d_min_km: f64,
    surface_age_myr: f64,
    diffuse_d: f64,
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
        if physical {
            // One draw, one diameter: the power law spans the whole range, so
            // there are no bands to pick between. `r` is that draw -- the RNG
            // is consumed once per crater here versus twice in the band path,
            // which is itself a reason the two sequences diverge.
            let d = crater_diameter_km(r, d_min_km, CRATER_D_MAX_KM, CRATER_SFD_EXPONENT);
            rad_km = d * 0.5;
            // Same morphology thresholds the bands implied, now keyed off the
            // drawn size rather than off which band was chosen.
            if rad_km >= 25.0 {
                large = true;
            }
            if rad_km > 200.0 {
                basin = true;
            }
        } else if r < 0.90 {
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
        // Degrade the crater that is actually *drawn*: `rad_cells` is the
        // gravity-scaled radius after `clamp_feature_radius_cells`, so
        // `rad_cells * cell_km * 2` is that feature's real diameter. Consuming
        // no RNG here is deliberate -- degradation is a deterministic function
        // of size and surface age, so turning it on does not shift the
        // placement sequence of any later crater.
        let tau =
            physical.then(|| crater_degradation_tau(rad_cells * cell_km * 2.0, surface_age_myr, diffuse_d));
        stamp_one_crater(gw, gh, field, impact_field, cx, cy, rad_cells, large, basin, age, tau);
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


#[cfg(test)]
mod crater_density_tests {
    use super::*;

    /// Area of the app's own untouched default map (800 km wide, 2048x1311).
    fn default_area() -> f64 {
        let h = 800.0 * 1311.0 / 2048.0;
        800.0 * h
    }

    /// **The calibration claim, asserted rather than asserted-in-prose.**
    /// `CRATER_SURFACE_AGE_MYR` is documented as landing the physical model
    /// near the reference's hand-tuned default of 100 craters on the default
    /// map. If that stops being true the doc comment is a lie, and this is
    /// where it gets caught.
    #[test]
    fn the_physical_model_lands_near_the_references_own_default() {
        let d_min = crater_min_diameter_km(800.0, 2048);
        let lambda = crater_lambda(100, default_area(), d_min, CRATER_SURFACE_AGE_MYR);
        assert!(
            (80.0..=110.0).contains(&lambda),
            "lambda {lambda} is no longer near the reference default of 100 -- \
             re-check CRATER_SURFACE_AGE_MYR's doc comment before changing this bound"
        );
    }

    /// The defect this whole change exists to fix: a 5 km region and a
    /// 40 000 km world get wildly different crater populations, because they
    /// represent areas differing by 64 000 000x.
    #[test]
    fn map_extent_now_drives_the_population() {
        let region = crater_lambda(
            200,
            5.0 * (5.0 * 1311.0 / 2048.0),
            crater_min_diameter_km(5.0, 2048),
            CRATER_SURFACE_AGE_MYR,
        );
        let world = crater_lambda(
            200,
            40_000.0 * (40_000.0 * 1311.0 / 2048.0),
            crater_min_diameter_km(40_000.0, 2048),
            CRATER_SURFACE_AGE_MYR,
        );
        assert!(region < 1.0, "a 5 km region should rarely hold one crater, got {region}");
        assert!(world > 50.0, "a 40 000 km world should be visibly cratered, got {world}");
        assert!(world <= CRATER_AUTO_MAX as f64, "the guard must still bound it");
    }

    /// The resolution floor is what keeps a world tractable: at 2048 cells a
    /// 40 000 km map cannot resolve anything under ~39 km, so the `D^-2` law
    /// must not be asked to generate the millions of 1 km craters that would
    /// physically be there.
    #[test]
    fn the_diameter_floor_follows_resolution() {
        let world = crater_min_diameter_km(40_000.0, 2048);
        let default = crater_min_diameter_km(800.0, 2048);
        let region = crater_min_diameter_km(5.0, 2048);
        assert!(world > 35.0, "40 000 km / 2048 cells should floor near 39 km, got {world}");
        assert_eq!(default, CRATER_D_MIN_KM, "the default map resolves below the 1 km floor");
        assert_eq!(region, CRATER_D_MIN_KM, "so does a small region");
        // Without the floor the world's count would be absurd.
        let unfloored = crater_lambda(
            100,
            40_000.0 * 25_605.0,
            CRATER_D_MIN_KM,
            CRATER_SURFACE_AGE_MYR,
        );
        assert_eq!(unfloored, CRATER_AUTO_MAX as f64, "unfloored, it pins the guard");
    }

    /// The size-frequency law: small craters must vastly outnumber large ones,
    /// which is exactly what the reference's flat bands got wrong.
    #[test]
    fn diameters_follow_the_inverse_square_law() {
        // Inverse-CDF endpoints are exact.
        assert_eq!(crater_diameter_km(0.0, 1.0, 400.0, 2.0), 1.0);
        assert!((crater_diameter_km(1.0, 1.0, 400.0, 2.0) - 400.0).abs() < 1e-9);

        // Halving the minimum diameter should roughly quadruple the count of
        // craters above it -- the D^-2 relation, checked on the sampler itself.
        let n = 20_000;
        let draws: Vec<f64> =
            (0..n).map(|i| crater_diameter_km(i as f64 / n as f64, 1.0, 400.0, 2.0)).collect();
        let ge = |d: f64| draws.iter().filter(|&&x| x >= d).count() as f64;
        let ratio = ge(2.0) / ge(4.0);
        assert!(
            (3.0..=5.0).contains(&ratio),
            "N(>=2km)/N(>=4km) = {ratio}, expected ~4 for an inverse-square law"
        );
        // And the population is dominated by the small end.
        assert!(ge(20.0) / ge(1.0) < 0.01, "large craters should be rare");
    }

    /// Poisson: right mean, and genuinely stochastic. Both halves matter -- a
    /// sampler that always returned `lambda` would pass a mean check alone.
    #[test]
    fn the_poisson_draw_has_the_right_mean_and_real_variance() {
        let mut rng = Mulberry32::new(12345);
        let (lambda, n) = (25.0, 4000);
        let mut sum = 0i64;
        let mut distinct = std::collections::BTreeSet::new();
        for _ in 0..n {
            let k = poisson_sample(&mut rng, lambda);
            assert!(k >= 0);
            sum += k as i64;
            distinct.insert(k);
        }
        let mean = sum as f64 / n as f64;
        assert!((mean - lambda).abs() < 1.0, "mean {mean} is not near {lambda}");
        assert!(distinct.len() > 10, "only {} distinct counts", distinct.len());
    }

    /// Deterministic per seed (worlds must be reproducible) and varying across
    /// seeds (or the stochastic half is decorative).
    #[test]
    fn auto_count_is_seed_deterministic_and_seed_varying() {
        let (a_km2, d) = (default_area(), crater_min_diameter_km(800.0, 2048));
        let first = auto_crater_count(777, 100, a_km2, d, CRATER_SURFACE_AGE_MYR);
        assert_eq!(first, auto_crater_count(777, 100, a_km2, d, CRATER_SURFACE_AGE_MYR));
        let varied: Vec<i32> =
            (0..24).map(|s| auto_crater_count(s, 100, a_km2, d, CRATER_SURFACE_AGE_MYR)).collect();
        assert!(varied.iter().any(|&v| v != first), "every seed gave {first}");
        assert!(varied.iter().all(|&v| v >= 0));
    }

    /// Zero-guards on every argument, and monotonic in the ones that should be.
    #[test]
    fn lambda_is_guarded_and_monotonic() {
        let d = CRATER_D_MIN_KM;
        assert_eq!(crater_lambda(0, 1.0e6, d, 100.0), 0.0);
        assert_eq!(crater_lambda(-5, 1.0e6, d, 100.0), 0.0);
        assert_eq!(crater_lambda(100, 0.0, d, 100.0), 0.0);
        assert_eq!(crater_lambda(100, 1.0e6, 0.0, 100.0), 0.0);
        assert_eq!(crater_lambda(100, 1.0e6, d, 0.0), 0.0, "a zero-age surface has no craters");
        assert!(crater_lambda(100, 2.0e5, d, 100.0) > crater_lambda(100, 1.0e5, d, 100.0));
        assert!(crater_lambda(100, 1.0e5, d, 200.0) > crater_lambda(100, 1.0e5, d, 100.0));
        assert!(crater_lambda(200, 1.0e5, d, 100.0) > crater_lambda(100, 1.0e5, d, 100.0));
    }
}

/// Degradation over geological time — the half of `DECISIONS.md` §7l that
/// built crater *frequency* and *size* but left *morphology with age* alone.
#[cfg(test)]
mod crater_degradation_tests {
    use super::*;

    /// Preservation: the fraction of a feature's relief still standing.
    fn keep(frac: f64, tau: f64) -> f64 {
        (-tau / (frac * frac)).exp()
    }

    /// The shipped diffusivity, so every test below reads the anchor's own
    /// configuration unless it is deliberately varying it.
    const D: f64 = CRATER_DEGRADATION_DIFFUSE_D_REF;

    /// **The anchor, asserted rather than left in prose.** `kappa` is not
    /// measured; it is pinned so a crater of `CRATER_D_MIN_KM` keeps exactly
    /// half its relief after one `CRATER_SURFACE_AGE_MYR` **at the default
    /// erosion diffusivity**. If this stops being true,
    /// `crater_degradation_tau`'s doc comment has become a lie.
    #[test]
    fn the_anchor_is_a_half_life_for_the_smallest_crater() {
        let p = (-crater_degradation_tau(CRATER_D_MIN_KM, CRATER_SURFACE_AGE_MYR, D)).exp();
        assert!(
            (p - 0.5).abs() < 1e-12,
            "a {CRATER_D_MIN_KM} km crater keeps {p} of its relief after \
             {CRATER_SURFACE_AGE_MYR} Myr, not the documented half"
        );
    }

    /// **Owner ruling 2, 2026-09-02: the calibration survived the rewiring.**
    ///
    /// `crater_degradation_tau` used to carry a private `kappa`; it now reads
    /// `ErosionPassParams::diffuse_d`. The requirement was that the default
    /// configuration land in exactly the same place, and "exactly" is meant
    /// literally: `d/d_ref` is `1.0` bit for bit at the default, so the whole
    /// expression is the old one unchanged.
    ///
    /// The old closed form is written out as a **literal expression** rather
    /// than by calling the function with the reference `d` — otherwise both
    /// sides would move together under any mutation of the constant, which is
    /// the self-snapshotting trap this repository has been bitten by four times.
    #[test]
    fn the_default_diffusivity_reproduces_the_old_anchor() {
        assert_eq!(
            CRATER_DEGRADATION_DIFFUSE_D_REF, 0.15,
            "the anchor was calibrated at the reference's own state.erosion.diffuseD"
        );
        for &(d_km, t_myr) in &[(1.0, 100.0), (3.0, 250.0), (37.5, 4000.0), (400.0, 1.0)] {
            let ratio = CRATER_D_MIN_KM / d_km;
            let was = std::f64::consts::LN_2 * (t_myr / CRATER_SURFACE_AGE_MYR) * ratio * ratio;
            let now = crater_degradation_tau(d_km, t_myr, 0.15);
            assert_eq!(now, was, "D={d_km} km, T={t_myr} Myr: the anchor moved");
        }
    }

    /// **The coupling the ruling asked for, and the reason it is documented in
    /// the GUI label**: `tau` is linear in the erosion diffusivity, so doubling
    /// `Diffusivity D` doubles the degradation state and a crater's surviving
    /// relief is squared. A user who touches an erosion slider changes how
    /// craters look, on purpose.
    #[test]
    fn the_erosion_diffusivity_drives_crater_degradation() {
        let base = crater_degradation_tau(4.0, 100.0, D);
        let twice = crater_degradation_tau(4.0, 100.0, D * 2.0);
        assert!(base > 0.0, "the probe must actually degrade something");
        assert!(
            (twice / base - 2.0).abs() < 1e-12,
            "doubling the diffusivity must double tau, got {}",
            twice / base
        );
        // And end to end, on the shipped path: a more diffusive world is flatter
        // at the same surface age, with the craters in the same places.
        let (slow, _) = stamp_probe_d(64.0, true, 200.0, D);
        let (fast, _) = stamp_probe_d(64.0, true, 200.0, D * 4.0);
        assert!(relief(&slow) > 1.0, "the probe stamped nothing to erode");
        assert!(
            relief(&fast) < relief(&slow),
            "a 4x diffusivity left {} of relief against {}",
            relief(&fast),
            relief(&slow)
        );
        // A zero diffusivity transports nothing, so nothing relaxes -- distinct
        // from "degradation off", and it must not divide by zero.
        assert_eq!(crater_degradation_tau(4.0, 100.0, 0.0), 0.0);
        assert_eq!(crater_degradation_tau(4.0, 100.0, -1.0), 0.0);
        assert_eq!(crater_degradation_tau(4.0, 100.0, f64::NAN), 0.0);
    }

    /// **The finding.** `t_diff ∝ L²/kappa`, so degradation state goes as
    /// `1/D²` — halving the diameter quadruples it, and a 100 km crater's
    /// timescale is 10 000x a 1 km crater's. This is why a single surface of a
    /// single age holds fresh large craters and erased small ones, and why the
    /// linear `1 - age*0.8` this replaces was wrong in kind.
    #[test]
    fn degradation_goes_as_the_inverse_square_of_diameter() {
        let two = crater_degradation_tau(2.0, 100.0, D);
        let four = crater_degradation_tau(4.0, 100.0, D);
        assert!(
            (two / four - 4.0).abs() < 1e-12,
            "halving the diameter must quadruple tau, got {}",
            two / four
        );
        let one = crater_degradation_tau(1.0, 100.0, D);
        let hundred = crater_degradation_tau(100.0, 100.0, D);
        assert!(
            (one / hundred - 10_000.0).abs() < 1e-6,
            "1 km vs 100 km should differ by 10 000x, got {}",
            one / hundred
        );

        // And the consequence, in the terms the old code got wrong: at the
        // default surface age these two craters are NOT equally fresh.
        let small = (-one).exp();
        let large = (-hundred).exp();
        assert!(small < 0.51, "a 1 km crater should be half gone, got {small}");
        assert!(large > 0.9999, "a 100 km crater should be untouched, got {large}");
    }

    /// Diffusive decay is exponential in time, so doubling the surface age
    /// squares what survives. Pins the `exp` form, not just monotonicity.
    #[test]
    fn doubling_the_surface_age_squares_what_survives() {
        let p1 = (-crater_degradation_tau(3.0, 100.0, D)).exp();
        let p2 = (-crater_degradation_tau(3.0, 200.0, D)).exp();
        assert!((p2 - p1 * p1).abs() < 1e-12, "exp({p1}) doubled gave {p2}, not {}", p1 * p1);
        assert!(p2 < p1, "an older surface must be more degraded");
    }

    /// `tau = 0` is "pristine, fully preserved" — **not** "degradation off",
    /// which is `None` at the call site. Conflating the two would silently make
    /// a brand-new surface behave like the reference path.
    #[test]
    fn a_pristine_surface_and_a_degenerate_crater_degrade_nothing() {
        assert_eq!(crater_degradation_tau(1.0, 0.0, D), 0.0, "T = 0 is a pristine surface");
        assert_eq!(crater_degradation_tau(1.0, -5.0, D), 0.0);
        assert_eq!(crater_degradation_tau(0.0, 100.0, D), 0.0);
        assert_eq!(crater_degradation_tau(f64::NAN, 100.0, D), 0.0);
        assert_eq!(crater_degradation_tau(1.0, f64::NAN, D), 0.0);
        // Preserved, i.e. keep == 1, for every feature.
        assert_eq!(keep(CRATER_FEATURE_RIM, 0.0), 1.0);
    }

    /// **The other half of the anchor**: the default map must still look like
    /// itself. Averaged over the population the physical model actually draws,
    /// craters keep about three quarters of their relief at the shipped
    /// default — worn, not erased. (The closed form for `b = 2` is
    /// `(1 - e^-τ)/τ` at `τ = ln2`, i.e. 0.721.)
    #[test]
    fn the_default_map_keeps_most_of_its_crater_relief() {
        let d_min = crater_min_diameter_km(800.0, 2048);
        let n = 20_000;
        let mut sum = 0.0;
        for i in 0..n {
            let u = (i as f64 + 0.5) / n as f64;
            let d = crater_diameter_km(u, d_min, CRATER_D_MAX_KM, CRATER_SFD_EXPONENT);
            sum += (-crater_degradation_tau(d, CRATER_SURFACE_AGE_MYR, D)).exp();
        }
        let mean = sum / n as f64;
        assert!(
            (0.65..=0.80).contains(&mean),
            "the population keeps {mean} of its relief at the default -- the anchor was \
             chosen so this stays near 0.72, i.e. close to the map's previous appearance"
        );
    }

    /// Each feature relaxes on **its own** length, not the crater's, so the rim
    /// — a fifth of the diameter wide — ages 25x faster than the bowl. Checked
    /// against the closed form at the crater's centre cell, where the bowl,
    /// the central peak and the first basin ring all land and the rim annulus
    /// does not. Pins all four `CRATER_FEATURE_*` fractions at once.
    #[test]
    fn every_feature_relaxes_on_its_own_length() {
        let (gw, gh) = (81usize, 81usize);
        let (tau, rad_cells) = (0.05, 20.0);
        let depth = 0.02 + rad_cells * 0.004; // 0.1, under the 0.4 cap
        let mut field = vec![0.5f32; gw * gh];
        let mut impact = vec![0.0f32; gw * gh];
        stamp_one_crater(
            gw, gh, &mut field, &mut impact, 40.0, 40.0, rad_cells, true, true, 0.0, Some(tau),
        );

        // The fractions are written out as literals ON PURPOSE. Reusing the
        // `CRATER_FEATURE_*` constants here would move both sides of the
        // comparison together and the test would survive any mutation of them
        // -- a snapshot of the code by the code, which is the trap this
        // repository has already been bitten by four times.
        let centre =
            -depth * keep(1.0, tau) + depth * 0.5 * keep(0.18, tau) + depth * 0.08 * keep(1.0 / 3.0, tau);
        assert!(
            (field[40 * gw + 40] as f64 - (0.5 + centre)).abs() < 1e-6,
            "centre cell {} != {}",
            field[40 * gw + 40],
            0.5 + centre
        );

        // (61, 40) is exactly t = 1.05, the rim crest, and nothing else reaches
        // it: the bowl and the rings stop at t = 1.
        let crest = depth * 0.25 * keep(0.2, tau);
        assert!(
            (field[40 * gw + 61] as f64 - (0.5 + crest)).abs() < 1e-6,
            "rim crest {} != {}",
            field[40 * gw + 61],
            0.5 + crest
        );
        assert!(crest > 0.0, "the probe must actually raise a rim, or it proves nothing");

        // The point of the exercise: at the same tau the rim is far further
        // gone than the bowl it encloses. A degraded crater is a rimless
        // shallow depression before it is nothing.
        assert!(
            keep(CRATER_FEATURE_RIM, tau) < keep(CRATER_FEATURE_BOWL, tau) * 0.5,
            "the rim must degrade much faster than the bowl"
        );
    }

    /// A real 128x128 stamping run, so the end-to-end tests measure the
    /// shipped path and not a re-implementation of it.
    fn stamp_probe(
        map_width_km: f64,
        physical: bool,
        surface_age_myr: f64,
    ) -> (Vec<f32>, Vec<f32>) {
        stamp_probe_d(map_width_km, physical, surface_age_myr, CRATER_DEGRADATION_DIFFUSE_D_REF)
    }

    /// The same probe with the erosion diffusivity exposed, for the tests that
    /// exercise owner ruling 2's coupling.
    fn stamp_probe_d(
        map_width_km: f64,
        physical: bool,
        surface_age_myr: f64,
        diffuse_d: f64,
    ) -> (Vec<f32>, Vec<f32>) {
        let (gw, gh) = (128usize, 128usize);
        let mut field = vec![0.5f32; gw * gh];
        let mut impact = vec![0.0f32; gw * gh];
        let d_min = crater_min_diameter_km(map_width_km, gw);
        stamp_craters(
            gw, gh, 4242, map_width_km, 1.0, 40, 0.5, physical, d_min, surface_age_myr, diffuse_d,
            &mut field, &mut impact,
        );
        (field, impact)
    }

    fn relief(f: &[f32]) -> f64 {
        f.iter().map(|&v| (v as f64 - 0.5).abs()).sum::<f64>()
    }

    /// End to end over the parameter's whole exposed range (0-4000 Myr): an old
    /// surface is measurably flatter than a young one, the output is not
    /// silently empty, and degradation consumes **no RNG** — the craters land
    /// in the same places at either age, so turning the surface age up wears a
    /// world down instead of reshuffling it. "Same places" is checked on the
    /// *support* of `impact_field` rather than its values, because owner ruling
    /// 3 made those values age-dependent — see
    /// `the_shock_record_fades_but_outlives_the_landform`.
    ///
    /// The **size** of the drop is the second half of the anchor, and it is
    /// bounded on both sides deliberately. At map scale the total relief is
    /// carried by the few largest craters, which are exactly the ones that
    /// barely relax, so 4000 Myr removes only ~13% of it: the map still looks
    /// like itself. A mutation that switched degradation off would push this
    /// ratio to 1.0, and one that over-fired would push it to 0.
    #[test]
    fn an_older_surface_is_flatter_but_identically_placed() {
        let (young, young_impact) = stamp_probe(128.0, true, 1.0);
        let (old, old_impact) = stamp_probe(128.0, true, 4000.0);
        let touched = |f: &[f32]| f.iter().filter(|&&v| v != 0.5).count();

        assert!(touched(&young) > 200, "only {} cells stamped -- probe is empty", touched(&young));
        assert!(touched(&old) > 200, "only {} cells stamped -- probe is empty", touched(&old));
        let ratio = relief(&old) / relief(&young);
        assert!(
            (0.75..=0.95).contains(&ratio),
            "4000 Myr left {ratio} of the young surface's relief; measured 0.870 when this \
             was written -- 1.0 means degradation stopped firing, near 0 means it took the \
             big craters it should not"
        );
        // Degradation must not consume RNG or move a crater. `impact_field` is
        // written at exactly `d < r` for every stamped crater, so its support is
        // the placement sequence made visible: identical support at two ages
        // means identical placement, while the *values* are free to fade.
        let support = |f: &[f32]| f.iter().map(|&v| v > 0.0).collect::<Vec<_>>();
        assert_eq!(
            support(&young_impact),
            support(&old_impact),
            "degradation moved a crater or consumed RNG: the shock record covers \
             different cells at the two ages"
        );
        assert!(
            young_impact.iter().any(|&v| v > 0.0),
            "the shock record is empty -- the probe proves nothing"
        );
    }

    /// **Owner ruling 3, 2026-09-02**, and the physics it has to respect.
    ///
    /// `impact_field` marks shocked rock and impact melt for the lithology
    /// stage. It used to be written pristine no matter how relaxed the crater
    /// was, which is wrong — but a relaxed crater still has shatter cones, so
    /// **the shock signature outlives the topography** and the damping must be
    /// gentler than the topographic one, not equal to it.
    ///
    /// Both halves are asserted: it fades (a mutation that dropped the damping
    /// fails the first pair) and it fades *more slowly than the relief* (a
    /// mutation that set `CRATER_FEATURE_SHOCK` to the bowl's `1.0`, or below
    /// it, fails the second).
    #[test]
    fn the_shock_record_fades_but_outlives_the_landform() {
        let (young, young_impact) = stamp_probe(64.0, true, 1.0);
        let (old, old_impact) = stamp_probe(64.0, true, 4000.0);
        let total = |f: &[f32]| f.iter().map(|&v| v as f64).sum::<f64>();

        assert!(total(&young_impact) > 1.0, "the young probe wrote no shock record");
        assert!(
            total(&old_impact) < total(&young_impact),
            "the shock record did not fade at all: {} at 4000 Myr vs {} at 1 Myr",
            total(&old_impact),
            total(&young_impact)
        );

        // The ordering that is the whole point. Relief is measured against the
        // 0.5 background `stamp_probe` starts from; the shock record against 0.
        let relief_kept = relief(&old) / relief(&young);
        let shock_kept = total(&old_impact) / total(&young_impact);
        assert!(
            shock_kept > relief_kept,
            "the shock record ({shock_kept}) must outlive the landform ({relief_kept})"
        );
        assert!(shock_kept > 0.0, "the shock record was erased entirely");
    }

    /// The rate, pinned against the closed form rather than against itself: the
    /// shock aureole is `CRATER_FEATURE_SHOCK` crater diameters across, so its
    /// timescale is that squared — **4x** the bowl's at `1.0 D`.
    ///
    /// The `2.0` is written as a literal here ON PURPOSE, for the same reason
    /// `every_feature_relaxes_on_its_own_length` writes its four out: reusing
    /// the constant would move both sides together and the test would survive
    /// any mutation of it.
    #[test]
    fn the_shock_aureole_relaxes_four_times_slower_than_the_bowl() {
        let (gw, gh) = (81usize, 81usize);
        let (tau, rad_cells) = (0.35, 20.0);
        let mut field = vec![0.5f32; gw * gh];
        let mut impact = vec![0.0f32; gw * gh];
        stamp_one_crater(gw, gh, &mut field, &mut impact, 40.0, 40.0, rad_cells, false, false, 0.0, Some(tau));

        // (50, 40) is t = 0.5 exactly, so the undamped write would be 1 - t = 0.5.
        let probe = impact[40 * gw + 50] as f64;
        let expected = 0.5 * keep(2.0, tau);
        assert!((probe - expected).abs() < 1e-6, "shock at t=0.5 was {probe}, not {expected}");
        assert!(probe < 0.5, "the shock record was not damped at all");
        assert!(
            probe > 0.5 * keep(CRATER_FEATURE_BOWL, tau),
            "the shock record must outlive the bowl it sits in"
        );
    }

    /// **Golden safety for ruling 3, stated as its own test.** The whole
    /// authorisation rested on the damping being gated by `crater.physical_model`
    /// — `WorldParams::defaults` has it `false`, so sixteen `cartalith-civ`
    /// golden suites read an undamped `impact_field` and stay parity tests.
    /// This asserts the gate directly: on the reference path the shock record is
    /// exactly `1 - t`, at any surface age and any diffusivity.
    #[test]
    fn the_reference_path_writes_an_undamped_shock_record() {
        let (gw, gh) = (81usize, 81usize);
        let mut field = vec![0.5f32; gw * gh];
        let mut impact = vec![0.0f32; gw * gh];
        stamp_one_crater(gw, gh, &mut field, &mut impact, 40.0, 40.0, 20.0, false, false, 0.0, None);
        assert_eq!(impact[40 * gw + 50] as f64, 0.5, "t = 0.5 must write exactly 1 - t");
        assert_eq!(impact[40 * gw + 40] as f64, 1.0, "the centre must write exactly 1.0");

        // And through the real stamper, across the parameter's whole range.
        let (_, a) = stamp_probe_d(128.0, false, 1.0, D);
        let (_, b) = stamp_probe_d(128.0, false, 4000.0, D * 8.0);
        assert_eq!(a, b, "physical_model: false must ignore both age and diffusivity");
        assert!(a.iter().any(|&v| v > 0.0), "the reference probe wrote no shock record");
    }

    /// The same run on an 8 km map, where every crater the grid can resolve is
    /// about 2 km across — and there the same 4000 Myr erases the population
    /// almost completely.
    ///
    /// **This is the point of the whole change**, and the test above is its
    /// other half: one surface age, one diffusivity, and whether a crater
    /// survives depends entirely on how big it is. Earth's crater record is a
    /// preserved subset rather than a census, which is the question
    /// `DECISIONS.md` §7l left open.
    #[test]
    fn the_same_age_erases_small_craters_and_spares_large_ones() {
        let (young, _) = stamp_probe(8.0, true, 1.0);
        let (old, _) = stamp_probe(8.0, true, 4000.0);
        assert!(relief(&young) > 1.0, "the young probe stamped nothing to erase");
        let ratio = relief(&old) / relief(&young);
        assert!(
            ratio < 0.05,
            "kilometre-scale craters should be all but gone after 4000 Myr, {ratio} survived"
        );
    }

    /// **Golden safety.** The reference has no degradation term, so the
    /// reference path must ignore `surface_age_myr` entirely — bit for bit,
    /// which is what keeps `golden_parity_volc_craters`, `golden_parity_carve`,
    /// `golden_parity_pipeline` and the 16 `cartalith-civ` suites untouched.
    #[test]
    fn the_reference_path_ignores_the_surface_age() {
        let (a, a_impact) = stamp_probe(128.0, false, 1.0);
        let (b, b_impact) = stamp_probe(128.0, false, 4000.0);
        assert_eq!(a, b, "physical_model: false must be byte-identical at any surface age");
        assert_eq!(a_impact, b_impact);
        assert!(a.iter().any(|&v| v != 0.5), "the reference probe stamped nothing");
    }
}
