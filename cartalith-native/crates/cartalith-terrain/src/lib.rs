//! tectonics, height formula, normalize, volcanism, world-structure archetypes
//!
//! Ported in pipeline order starting Phase 1 (MVP_SCOPE.md).

use cartalith_noise::{fbm, pfbm, pridged, ridged};
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

/// `boxH()` (reference HTML line 2511): horizontal sliding-window box
/// blur. The running sum (`acc`) stays `f64` throughout — matching JS,
/// where `acc` is a plain number even though it sums `Float32Array`
/// reads — and is only rounded to `f32` at the point of writing `dst`.
fn box_h(src: &[f32], dst: &mut [f32], w: usize, h: usize, r: i64, wrap: bool) {
    let norm = 1.0 / (2.0 * r as f64 + 1.0);
    let wi = w as i64;
    for y in 0..h {
        let row = y * w;
        let mut acc = 0.0f64;
        if wrap {
            for k in -r..=r {
                let idx = (((k % wi) + wi) % wi) as usize;
                acc += src[row + idx] as f64;
            }
        } else {
            for k in -r..=r {
                let idx = k.clamp(0, wi - 1) as usize;
                acc += src[row + idx] as f64;
            }
        }
        for x in 0..w {
            dst[row + x] = (acc * norm) as f32;
            let xi = x as i64;
            if wrap {
                let o = (((xi - r) % wi) + wi) % wi;
                let i = (((xi + r + 1) % wi) + wi) % wi;
                acc += src[row + i as usize] as f64 - src[row + o as usize] as f64;
            } else {
                let o = (xi - r).clamp(0, wi - 1) as usize;
                let i = (xi + r + 1).clamp(0, wi - 1) as usize;
                acc += src[row + i] as f64 - src[row + o] as f64;
            }
        }
    }
}

/// `boxV()` (reference HTML line 2512): vertical sliding-window box blur.
/// Always clamps at the top/bottom edge — maps don't wrap pole-to-pole,
/// only (optionally) east-west, so unlike `box_h` there's no wrap variant.
fn box_v(src: &[f32], dst: &mut [f32], w: usize, h: usize, r: i64) {
    let norm = 1.0 / (2.0 * r as f64 + 1.0);
    let hi = h as i64;
    for x in 0..w {
        let mut acc = 0.0f64;
        for k in -r..=r {
            let idx = k.clamp(0, hi - 1) as usize;
            acc += src[idx * w + x] as f64;
        }
        for y in 0..h {
            dst[y * w + x] = (acc * norm) as f32;
            let yi = y as i64;
            let o = (yi - r).clamp(0, hi - 1) as usize;
            let i = (yi + r + 1).clamp(0, hi - 1) as usize;
            acc += src[i * w + x] as f64 - src[o * w + x] as f64;
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
fn classify_boundary(ocean_a: bool, ocean_b: bool, c: f64, s: f64) -> u8 {
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
    plate_id: &[usize],
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
            let a = plate_id[i];
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
                let b = plate_id[j];
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
    for y in 0..gh {
        for x in 0..gw {
            let i = y * gw + x;
            let wx = x as f64 + warp_x.map_or(0.0, |w| w[i] as f64);
            let wy = y as f64 + warp_y.map_or(0.0, |w| w[i] as f64);
            let low_n = if world {
                pfbm(wx * hf / gw as f64, wy * hf / gw as f64, hetero_seed, oct)
            } else {
                fbm(wx * hf / gw as f64, wy * hf / gw as f64, hetero_seed)
            } - 0.5;
            out[i] = (low_n * (0.3 + 0.7 * age[i] as f64)) as f32;
        }
    }
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
    plate_id: &[usize],
    plates: &[Plate],
    age_field: &[f32],
) -> Vec<f32> {
    let n = gw * gh;
    let mut resistance = vec![0f32; n];
    for i in 0..n {
        let pl = plates[plate_id[i]];
        let crustal = pl.base.max(0.0);
        resistance[i] = (crustal * 0.6 + age_field[i] as f64 * 0.4).min(1.0) as f32;
    }
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
    for y in 0..gh {
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
            field[i] = (0.5
                + p.a * (0.40 * bs + 0.50 * t)
                + p.fwt * flex[i] as f64
                + p.hwt * hetero[i] as f64
                + p.b * n_val * (0.25 + 0.75 * rug)) as f32;
        }
    }
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
/// which routes through `stampVolcanoesProvinces` (clustered arc/rift/
/// hotspot chains) instead. That function isn't ported yet; this one is
/// the shared foundation (`stampOneVolcano`/`placeSizedVolcano`) both
/// modes build on, ported first because it's independently useful and
/// far simpler to golden-verify in isolation. Tracked, not silently
/// dropped — see `cartalith-native/docs/CHANGELOG.md`.
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
