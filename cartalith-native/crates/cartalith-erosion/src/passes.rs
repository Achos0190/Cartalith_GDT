//! The reference's **manual** erosion passes — the ones `generate()` never
//! runs (`GENERATION_PARAMETERS.md`, "Parameters the reference exposed that
//! this port does not"; `GUI_GAP_REGISTER.md` WW-02 / MS-05).
//!
//! Ported here as pure kernels only. **None of them has a caller**, and that
//! is deliberate rather than unfinished: in the reference every one of these
//! is behind a button that mutates the finished field and then re-derives
//! flow and climate (`erodeFinish`/`eroFinish`/`veloFinish`), and the reference
//! says so in its own comments — `evolveCoupled`'s *"A new op (never
//! auto-runs) → generate() bit-identical at defaults"* and `glacialKernel`'s
//! *"Manual Glacial erosion button + its worker path only — not part of
//! default generate()"*. Whether this port grows those buttons, or folds the
//! passes into `generate_terrain` as parameters the reference never had, is an
//! owner decision recorded in `GUI_GAP_REGISTER.md` WW-02; the kernels are the
//! half of it that is the same either way.
//!
//! Every one is golden-parity tested bit-exact against the frozen reference
//! (`tests/golden_parity_passes.rs`), `assert_eq!` on `f32`, no tolerance.
//!
//! **Mutation sweep: 115 literal sites, 98 killed.** Golden-matching alone is
//! not sufficient in this project, so every numeric literal in this module was
//! perturbed in turn and the golden suite re-run. Four fixture passes were
//! shaped to reach what the first sweep missed — saturating clamps, quantised
//! heights so tie-breaks bite, a 34-wide 120-iteration velocity run, a 9×36
//! glacial ramp whose discharge climbs *through* the 100-cell cirque cut-off,
//! rain under its own floor, gravity under its own floor, negative discharge,
//! and a monotone chain whose whole result depends on the sort order.
//!
//! The 17 survivors are each explained rather than shrugged at, and they fall
//! into three groups:
//!
//! - **Dead in the reference** — no input can observe the change. `e = 0.0`
//!   and `d = 0.0` are the `< 0` arms of branches whose own guard already
//!   makes the value positive; `water[i] < 0.0` cannot fire because water is
//!   non-negative and `1 − evap·dt` is positive; the `±1e9` finite guard is
//!   unreachable while the reference's Invariant 2 holds; `dist[i] || 1`'s
//!   fallback needs `dist == 0`, which only border-seeded cells have, and
//!   those `continue` two lines earlier on `receiver < 0`;
//!   `carried[i] = 0.0` is a dead store (each cell is processed once and never
//!   read again). `applyTidalSedimentation`'s `tr <= 1e-5` floor is dead for a
//!   sharper reason: accretion needs `depth < tr`, so any cell the floor could
//!   gate also has `sea − 1e-4 − h < 0` and writes nothing — the 1e-4 headroom
//!   cap subsumes the floor entirely.
//! - **Redundant with a constant that *is* pinned** — `r > 1.0`'s partner
//!   assignment, and `min(1.0)`/`k < 1.0`, which are two halves of one clamp.
//!   `route_sediment`'s comparator `v > 0.0` arm is provably never consulted:
//!   a stable merge sort only ever asks whether the right element is strictly
//!   `Less` than the left, and that arm is killed.
//! - **Razor-edge thresholds** — a mutant 17 % away is only observable for an
//!   input inside that 17 % window: `sp > 1e-4`, `load <= 1e-12`,
//!   `min(1.0, water)`, and the priority-flood `EPS`, which would need a fill
//!   chain whose ordering flips under a larger increment.
//!
//! Precision discipline is `cartalith-rust-conventions`': every JS
//! `Float32Array` store rounds, so each one is written `(a as f64 op b) as
//! f32` and every subsequent read sees the *rounded* value. `Math.hypot` and
//! `Math.log` go through `cartalith-jsmath` because V8's are not Rust's.
//! `Math.max`/`Math.min` are plain `f64::max`/`f64::min`: the reference pins
//! its own all-finite invariant on these kernels (`velocityErodeKernel`'s
//! *"Every write is clamped — Invariant 2 (all-finite) holds"*), so the
//! JS-propagates-NaN / Rust-absorbs-NaN split is unreachable here.

use crate::{d8_table, MinHeap};
use cartalith_jsmath::{js_hypot, js_log, js_round};
use cartalith_noise::vnoise;
use std::cmp::Ordering;

/// The reference's `if(f<0)f=0; if(f>1)f=1;` applied to an **already-rounded**
/// `f32`, which is what JS does — the comparison reads the stored element, not
/// the `f64` that produced it.
#[inline]
// Not `f32::clamp`: this is the reference's own two-statement
// `if(f<0)f=0; if(f>1)f=1;`, transcribed. `clamp` would read the same today
// and would quietly stop reading like the source it is being checked against.
#[allow(clippy::manual_clamp)]
fn store_clamped01(slot: &mut f32, v: f64) {
    let r = v as f32;
    *slot = if r < 0.0 {
        0.0
    } else if r > 1.0 {
        1.0
    } else {
        r
    };
}

// ===================================================================
// hillslope diffusion
// ===================================================================

/// `hillslopeDiffuseCPU()` (reference HTML lines 3872-3882) — the *Hillslope
/// diffuse* button's kernel: `∂z/∂t = D∇²z` by explicit forward Euler, one
/// fresh `Float32Array` of deltas per pass.
///
/// `world` is the reference's `ww = !!state.world && w === GW`: X wraps only
/// when the grid being diffused *is* the world grid. The reference's only
/// caller passes no `w`/`h` at all, so that guard is the caller's business and
/// is taken here as one boolean.
///
/// Y never wraps in either mode — the poles are hard edges, and a row-0 or
/// row-(h−1) cell uses its own height as the missing neighbour, which makes
/// the Laplacian vanish there rather than pulling the edge downhill.
pub fn hillslope_diffuse(fld: &mut [f32], w: usize, h: usize, passes: i32, d: f64, world: bool) {
    let n = w * h;
    for _ in 0..passes {
        let mut delta = vec![0f32; n];
        for y in 0..h {
            for x in 0..w {
                let i = y * w + x;
                let xl = if world {
                    (x + w - 1) % w
                } else if x > 0 {
                    x - 1
                } else {
                    x
                };
                let xr = if world {
                    (x + 1) % w
                } else if x < w - 1 {
                    x + 1
                } else {
                    x
                };
                let l = fld[y * w + xl] as f64;
                let r = fld[y * w + xr] as f64;
                let u = if y > 0 { fld[(y - 1) * w + x] } else { fld[i] } as f64;
                let dn = if y < h - 1 { fld[(y + 1) * w + x] } else { fld[i] } as f64;
                delta[i] = (d * (l + r + u + dn - 4.0 * fld[i] as f64)) as f32;
            }
        }
        for i in 0..n {
            let v = fld[i] as f64 + delta[i] as f64;
            store_clamped01(&mut fld[i], v);
        }
    }
}

// ===================================================================
// velocity (momentum) erosion — Mei et al. 2007 virtual pipes
// ===================================================================

/// `_bilin()` (reference HTML lines 3919-3922): clamped bilinear sample of an
/// `f32` grid at a fractional position.
///
/// `fx|0` is JS `ToInt32` — truncation toward zero — but `fx` is clamped
/// non-negative one line above, so truncation *is* `floor` here and a plain
/// `as usize` reproduces it.
#[inline]
fn bilin(a: &[f32], fx: f64, fy: f64, w: usize, h: usize) -> f64 {
    let fx = fx.clamp(0.0, (w - 1) as f64);
    let fy = fy.clamp(0.0, (h - 1) as f64);
    let x0 = fx as usize;
    let y0 = fy as usize;
    let x1 = if x0 < w - 1 { x0 + 1 } else { x0 };
    let y1 = if y0 < h - 1 { y0 + 1 } else { y0 };
    let tx = fx - x0 as f64;
    let ty = fy - y0 as f64;
    a[y0 * w + x0] as f64 * (1.0 - tx) * (1.0 - ty)
        + a[y0 * w + x1] as f64 * tx * (1.0 - ty)
        + a[y1 * w + x0] as f64 * (1.0 - tx) * ty
        + a[y1 * w + x1] as f64 * tx * ty
}

/// `centrifugalShear()` (reference HTML lines 3926-3930): the outer-bank
/// direction and turn strength when flow `(vx, vy)` bends toward
/// `(nvx, nvy)` a little downstream — the meander/oxbow mechanism, since
/// inertia throws the stream against the outer bank and leaves slack water
/// on the inner one.
///
/// Returns `(ox, oy, mag)`: a unit vector toward the outer bank, times a
/// normalised turn magnitude. `(0, 0, 0)` for flow slower than `1e-6`.
pub fn centrifugal_shear(vx: f64, vy: f64, nvx: f64, nvy: f64) -> (f64, f64, f64) {
    let sp = js_hypot(vx, vy);
    if sp < 1e-6 {
        return (0.0, 0.0, 0.0);
    }
    let ux = vx / sp;
    let uy = vy / sp;
    let cross = ux * nvy - uy * nvx;
    let s = if cross >= 0.0 { 1.0 } else { -1.0 };
    (s * uy, -s * ux, cross.abs().min(1.0))
}

/// The knobs `veloParams()` bundles (reference HTML line 3995).
///
/// `veloParams` also sets a `world` key that `velocityErodeKernel` never
/// reads — the map border is reflective in both modes — so it has no field
/// here. The reference's own mapping from `state.velo`, kept at the call
/// site rather than baked in: `iters = clamp(v.iters, 10, 160)`,
/// `dt = 0.02`, `gravity = 9.8 × planet.g`, `rain_rate = 0.012`,
/// `evap = 0.05`, `capacity = 0.5 + 1.5·strength`,
/// `erode_k = 0.05 + 0.5·strength`, `deposit_k = 0.25`,
/// `min_slope = 0.001`, `centrifugal_k = 1.4·meander`, `sea = seaLevel`.
#[derive(Clone, Copy, Debug)]
pub struct VelocityParams {
    pub iters: i32,
    pub dt: f64,
    pub gravity: f64,
    pub rain_rate: f64,
    pub evap: f64,
    pub capacity: f64,
    pub erode_k: f64,
    pub deposit_k: f64,
    pub min_slope: f64,
    pub centrifugal_k: f64,
    pub sea: f64,
}

/// What `velocityErodeKernel` returns beside the mutated height: the last
/// iteration's water depth and velocity field. The reference keeps these for
/// its *Velocity* debug view and its flow-map, which is why they are returned
/// rather than dropped.
pub struct VelocityField {
    pub water: Vec<f32>,
    pub vx: Vec<f32>,
    pub vy: Vec<f32>,
}

/// `velocityErodeKernel()` (reference HTML lines 3936-3994) — grid
/// (virtual-pipes) shallow-water hydraulic erosion with a continuous 2D
/// velocity field and semi-Lagrangian momentum advection, after Mei et al.
/// (2007) and Beyer (2015), with centrifugal bank shear for meanders.
///
/// Six sub-steps per iteration, in this order and not another: rain input
/// (sea cells are open sinks, their water is reset to zero every step) →
/// virtual-pipe flux, scaled so outflow never exceeds available water →
/// water update and flux-to-velocity → semi-Lagrangian advection of momentum
/// *and* suspended sediment → capacity erode/deposit with the outer-bank
/// bias → evaporation. Suspended load settles onto the surface at the end.
///
/// `rain` weights the per-cell rain input by `max(0.05, rain[i])`; `None` is
/// the reference's own `rain?…:1` fallback, a flat unit rate.
///
/// Three ordering details that a tidier rewrite would get wrong:
/// - step 3's `wAvg` averages the water depth *before* the update with the
///   value **just stored** (so `f32`-rounded), not with the `f64` that
///   produced it;
/// - step 5's outer-bank erosion writes `fld[j]` and `sed[i]` *before* the
///   cell's own `fld[i] -= e` / `sed[i] += e`, and `j` can be `i`;
/// - the final finite guard runs over every cell, sea included, not just the
///   land cells that got their sediment back.
// The two `if x > hi {..} else if x < lo {..}` clamps in step 5 are the
// reference's own statement shape, kept literally; see `store_clamped01`.
#[allow(clippy::manual_clamp)]
pub fn velocity_erode_kernel(
    fld: &mut [f32],
    rain: Option<&[f32]>,
    w: usize,
    h: usize,
    p: &VelocityParams,
) -> VelocityField {
    const PIPE_DAMP: f64 = 0.92;
    const VMAX: f64 = 4.0;

    let n = w * h;
    let (sea, dt, g, evap) = (p.sea, p.dt, p.gravity, p.evap);
    let (kc, ke, kd) = (p.capacity, p.erode_k, p.deposit_k);
    let (min_s, cf_k, rain_rate) = (p.min_slope, p.centrifugal_k, p.rain_rate);

    let mut water = vec![0f32; n];
    let mut sed = vec![0f32; n];
    let mut vx = vec![0f32; n];
    let mut vy = vec![0f32; n];
    let (mut f_l, mut f_r) = (vec![0f32; n], vec![0f32; n]);
    let (mut f_t, mut f_b) = (vec![0f32; n], vec![0f32; n]);
    let mut nvx = vec![0f32; n];
    let mut nvy = vec![0f32; n];
    let mut nsed = vec![0f32; n];

    for _ in 0..p.iters {
        // 1) rain input (sea cells are sinks -> water reset)
        for i in 0..n {
            if (fld[i] as f64) < sea {
                water[i] = 0.0;
            } else {
                let r = match rain {
                    Some(rr) => (rr[i] as f64).max(0.05),
                    None => 1.0,
                };
                water[i] = (water[i] as f64 + rain_rate * r) as f32;
            }
        }

        // 2) virtual-pipe flux (Mei et al. 2007)
        for y in 0..h {
            for x in 0..w {
                let i = y * w + x;
                if (fld[i] as f64) < sea {
                    f_l[i] = 0.0;
                    f_r[i] = 0.0;
                    f_t[i] = 0.0;
                    f_b[i] = 0.0;
                    continue;
                }
                let hi = fld[i] as f64 + water[i] as f64;
                let hl = if x > 0 { fld[i - 1] as f64 + water[i - 1] as f64 } else { hi };
                let hr = if x < w - 1 { fld[i + 1] as f64 + water[i + 1] as f64 } else { hi };
                let ht = if y > 0 { fld[i - w] as f64 + water[i - w] as f64 } else { hi };
                let hb = if y < h - 1 { fld[i + w] as f64 + water[i + w] as f64 } else { hi };
                let mut l = (f_l[i] as f64 * PIPE_DAMP + dt * g * (hi - hl)).max(0.0);
                let mut r = (f_r[i] as f64 * PIPE_DAMP + dt * g * (hi - hr)).max(0.0);
                let mut t = (f_t[i] as f64 * PIPE_DAMP + dt * g * (hi - ht)).max(0.0);
                let mut b = (f_b[i] as f64 * PIPE_DAMP + dt * g * (hi - hb)).max(0.0);
                let out = l + r + t + b;
                if out > 0.0 {
                    let k = (water[i] as f64 / (out * dt)).min(1.0);
                    if k < 1.0 {
                        l *= k;
                        r *= k;
                        t *= k;
                        b *= k;
                    }
                }
                f_l[i] = l as f32;
                f_r[i] = r as f32;
                f_t[i] = t as f32;
                f_b[i] = b as f32;
            }
        }

        // 3) water update + flux -> velocity (momentum proxy)
        for y in 0..h {
            for x in 0..w {
                let i = y * w + x;
                if (fld[i] as f64) < sea {
                    vx[i] = 0.0;
                    vy[i] = 0.0;
                    continue;
                }
                let in_l = if x > 0 { f_r[i - 1] as f64 } else { 0.0 };
                let in_r = if x < w - 1 { f_l[i + 1] as f64 } else { 0.0 };
                let in_t = if y > 0 { f_b[i - w] as f64 } else { 0.0 };
                let in_b = if y < h - 1 { f_t[i + w] as f64 } else { 0.0 };
                let w_before = water[i] as f64;
                let outflow = f_l[i] as f64 + f_r[i] as f64 + f_t[i] as f64 + f_b[i] as f64;
                water[i] =
                    (water[i] as f64 + dt * ((in_l + in_r + in_t + in_b) - outflow)).max(0.0) as f32;
                let w_avg = (0.5 * (w_before + water[i] as f64)).max(1e-3);
                let mut u = 0.5 * ((in_l - f_l[i] as f64) + (f_r[i] as f64 - in_r)) / w_avg;
                let mut v = 0.5 * ((in_t - f_t[i] as f64) + (f_b[i] as f64 - in_b)) / w_avg;
                let sp = js_hypot(u, v);
                if sp > VMAX {
                    u *= VMAX / sp;
                    v *= VMAX / sp;
                }
                vx[i] = u as f32;
                vy[i] = v as f32;
            }
        }

        // 4) semi-Lagrangian momentum advection + gravity accel
        for y in 0..h {
            for x in 0..w {
                let i = y * w + x;
                let bx = x as f64 - vx[i] as f64 * dt;
                let by = y as f64 - vy[i] as f64 * dt;
                let mut au = bilin(&vx, bx, by, w, h);
                let mut av = bilin(&vy, bx, by, w, h);
                let hl = if x > 0 { fld[i - 1] } else { fld[i] } as f64;
                let hr = if x < w - 1 { fld[i + 1] } else { fld[i] } as f64;
                let ht = if y > 0 { fld[i - w] } else { fld[i] } as f64;
                let hb = if y < h - 1 { fld[i + w] } else { fld[i] } as f64;
                au += g * dt * ((hl - hr) * 0.5);
                av += g * dt * ((ht - hb) * 0.5);
                let sp = js_hypot(au, av);
                if sp > VMAX {
                    au *= VMAX / sp;
                    av *= VMAX / sp;
                }
                nvx[i] = au as f32;
                nvy[i] = av as f32;
                nsed[i] = bilin(&sed, bx, by, w, h) as f32;
            }
        }
        vx.copy_from_slice(&nvx);
        vy.copy_from_slice(&nvy);
        sed.copy_from_slice(&nsed);

        // 5) capacity erode/deposit + centrifugal outer-bank bias (meanders)
        for y in 0..h {
            for x in 0..w {
                let i = y * w + x;
                if (fld[i] as f64) < sea {
                    continue;
                }
                let hl = if x > 0 { fld[i - 1] } else { fld[i] } as f64;
                let hr = if x < w - 1 { fld[i + 1] } else { fld[i] } as f64;
                let ht = if y > 0 { fld[i - w] } else { fld[i] } as f64;
                let hb = if y < h - 1 { fld[i + w] } else { fld[i] } as f64;
                let slope =
                    (js_hypot((hr - hl) * 0.5, (hb - ht) * 0.5) * w as f64 * 0.05).min(1.0);
                let sp = js_hypot(vx[i] as f64, vy[i] as f64);
                let c = kc * min_s.max(slope) * sp * (water[i] as f64).min(1.0);
                if c > sed[i] as f64 {
                    let mut e = ke * (c - sed[i] as f64);
                    if e > 0.05 {
                        e = 0.05;
                    } else if e < 0.0 {
                        e = 0.0;
                    }
                    if cf_k > 0.0 && sp > 1e-4 {
                        let ux = vx[i] as f64 / sp;
                        let uy = vy[i] as f64 / sp;
                        let (px, py) = (x as f64 + ux * 2.0, y as f64 + uy * 2.0);
                        let (ox_v, oy_v, mag) = centrifugal_shear(
                            vx[i] as f64,
                            vy[i] as f64,
                            bilin(&vx, px, py, w, h),
                            bilin(&vy, px, py, w, h),
                        );
                        if mag > 0.0 {
                            let ox = js_round(x as f64 + ox_v);
                            let oy = js_round(y as f64 + oy_v);
                            if ox >= 0.0 && oy >= 0.0 && ox < w as f64 && oy < h as f64 {
                                let j = oy as usize * w + ox as usize;
                                if fld[j] as f64 >= sea {
                                    let mut eb = cf_k * mag * e;
                                    if eb > 0.03 {
                                        eb = 0.03;
                                    }
                                    fld[j] = (fld[j] as f64 - eb) as f32;
                                    sed[i] = (sed[i] as f64 + eb) as f32;
                                }
                            }
                        }
                    }
                    fld[i] = (fld[i] as f64 - e) as f32;
                    sed[i] = (sed[i] as f64 + e) as f32;
                } else {
                    let mut d = kd * (sed[i] as f64 - c);
                    if d > sed[i] as f64 {
                        d = sed[i] as f64;
                    } else if d < 0.0 {
                        d = 0.0;
                    }
                    fld[i] = (fld[i] as f64 + d) as f32;
                    sed[i] = (sed[i] as f64 - d) as f32;
                }
            }
        }

        // 6) evaporation
        for i in 0..n {
            if fld[i] as f64 >= sea {
                let v = (water[i] as f64 * (1.0 - evap * dt)) as f32;
                water[i] = if v < 0.0 { 0.0 } else { v };
            }
        }
    }

    // settle suspended load + finite guard (the reference's Invariant 2)
    for i in 0..n {
        if fld[i] as f64 >= sea {
            fld[i] = (fld[i] as f64 + sed[i] as f64) as f32;
        }
        if !(fld[i] as f64 <= 1e9 && fld[i] as f64 >= -1e9) {
            fld[i] = sea as f32;
        }
    }

    VelocityField { water, vx, vy }
}

// ===================================================================
// glacial erosion
// ===================================================================

/// The knobs `glacialParams()` bundles (reference HTML line 4262) —
/// `state.glacial` plus the derived `g`/`sea`/`world` context.
#[derive(Clone, Copy, Debug)]
pub struct GlacialParams {
    /// Glacial erodibility. Reference default `0.15`.
    pub kg: f64,
    /// Discharge exponent in `E ∝ Q^mg`. Reference default `0.4`.
    pub mg: f64,
    /// Snowline as a fraction of the above-sea range: the ice only forms
    /// above `sea + (1 − sea)·snowline`. Reference default `0.65`.
    pub snowline: f64,
    /// How much of the trunk's incision is dealt to the two cells flanking
    /// it — the U-shaped-valley term. Reference default `0.6`.
    pub u_factor: f64,
    pub passes: i32,
    /// Planet gravity: abrasion scales with it (the reference's G1
    /// gravity workstream).
    pub g: f64,
    pub sea: f64,
    pub world: bool,
}

/// `glacialKernel()` (reference HTML lines 4198-4257) — ice-sheet abrasion
/// carving U-shaped valleys, on its own priority-flood-filled drainage tree.
///
/// A cell erodes only where it is **both** above the snowline and below
/// freezing (`temp < 0`), so a warm or low world honestly carves nothing.
/// Each eroding cell takes `E` out of itself, `E·u_factor` out of each of the
/// two cells perpendicular to its flow direction (the trough walls — this is
/// what makes the valley U-shaped rather than V-shaped), and a further
/// `E·0.6` out of itself where discharge is under 100 (cirque overdeepening
/// at the head of the network).
///
/// The flood is `stream_power_kernel`'s, duplicated rather than shared —
/// the reference duplicates it too (its Invariant 11: a worker-shippable
/// kernel carries everything it needs), and the two have diverged: this one
/// records a receiver and a D8 distance per cell, which stream-power's does
/// not keep past the fill.
///
/// The perpendicular offsets are computed from raw index arithmetic
/// (`rx − x`, `ry − y`) with no seam correction, exactly as in the reference:
/// on a wrapped world a receiver across the seam gives `±(w−1)` instead of
/// `∓1`, which puts both flank cells outside the grid and the bounds test
/// drops them. That is the reference's behaviour, not an oversight of this
/// port — one seam column of a wrapped world gets trunk incision without
/// flank widening.
pub fn glacial_kernel(fld: &mut [f32], temp: &[f32], w: usize, h: usize, p: &GlacialParams) {
    let n = w * h;
    let wrap = p.world;
    let snow_el = p.sea + (1.0 - p.sea) * p.snowline;
    let d8 = d8_table();

    let mut receiver = vec![-1i32; n];
    let mut order = vec![0i32; n];
    let mut dist = vec![0f32; n];
    let mut done = vec![false; n];
    let mut filled: Vec<f32> = fld.to_vec();
    let mut heap = MinHeap::new(n);

    let seed_cell = |x: usize, y: usize, done: &mut Vec<bool>, filled: &[f32], heap: &mut MinHeap| {
        let i = y * w + x;
        if !done[i] {
            done[i] = true;
            heap.push(filled[i], i as i32);
        }
    };
    for x in 0..w {
        seed_cell(x, 0, &mut done, &filled, &mut heap);
        seed_cell(x, h - 1, &mut done, &filled, &mut heap);
    }
    if !wrap {
        for y in 0..h {
            seed_cell(0, y, &mut done, &filled, &mut heap);
            seed_cell(w - 1, y, &mut done, &filled, &mut heap);
        }
    }

    const EPS: f64 = 1e-5;
    let mut cnt = 0usize;
    while heap.size() > 0 {
        let i = heap.pop() as usize;
        order[cnt] = i as i32;
        cnt += 1;
        let x = (i % w) as i64;
        let y = (i / w) as i64;
        for dy in -1i64..=1 {
            for dx in -1i64..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let mut nx = x + dx;
                let ny = y + dy;
                if wrap {
                    nx = ((nx % w as i64) + w as i64) % w as i64;
                } else if nx < 0 || nx >= w as i64 {
                    continue;
                }
                if ny < 0 || ny >= h as i64 {
                    continue;
                }
                let j = (ny * w as i64 + nx) as usize;
                if done[j] {
                    continue;
                }
                done[j] = true;
                if filled[j] <= filled[i] {
                    filled[j] = (filled[i] as f64 + EPS) as f32;
                }
                receiver[j] = i as i32;
                dist[j] = d8[((dy + 1) * 3 + (dx + 1)) as usize] as f32;
                heap.push(filled[j], j as i32);
            }
        }
    }

    let mut flow = vec![0f32; n];
    let mut delta = vec![0f32; n];
    for _ in 0..p.passes {
        flow.fill(1.0);
        // Downstream accumulation, reverse fill order. `flow[r] += flow[i]`
        // rounds each contribution to `f32` individually: one receiver takes
        // several donors within the same pass.
        for k in (0..n).rev() {
            let i = order[k] as usize;
            let r = receiver[i];
            if r >= 0 {
                let r = r as usize;
                flow[r] = (flow[r] as f64 + flow[i] as f64) as f32;
            }
        }
        delta.fill(0.0);
        // Walked in fill order, ascending: `order` is a permutation and the
        // rank is the loop's whole point, so it is indexed, not iterated.
        #[allow(clippy::needless_range_loop)]
        for k in 0..n {
            let i = order[k] as usize;
            if (fld[i] as f64) < snow_el || temp[i] as f64 >= 0.0 {
                continue;
            }
            let r = receiver[i];
            if r < 0 {
                continue;
            }
            let r = r as usize;
            // JS `dist[i]||1`: a border-seeded cell never got a distance.
            let dd = dist[i] as f64;
            let denom = if dd != 0.0 { dd } else { 1.0 };
            let slope = (fld[i] as f64 - fld[r] as f64).max(0.0) / denom;
            let e = p.kg * p.g * (flow[i] as f64).powf(p.mg) * slope * 0.001;
            delta[i] = (delta[i] as f64 - e) as f32;
            let x = (i % w) as i64;
            let y = (i / w) as i64;
            let ddx = (r % w) as i64 - x;
            let ddy = (r / w) as i64 - y;
            let (lx1, ly1) = (x - ddy, y + ddx);
            let (lx2, ly2) = (x + ddy, y - ddx);
            for (lx, ly) in [(lx1, ly1), (lx2, ly2)] {
                if lx >= 0 && lx < w as i64 && ly >= 0 && ly < h as i64 {
                    let j = (ly * w as i64 + lx) as usize;
                    delta[j] = (delta[j] as f64 - e * p.u_factor) as f32;
                }
            }
            if (flow[i] as f64) < 100.0 {
                delta[i] = (delta[i] as f64 - e * 0.6) as f32;
            }
        }
        for i in 0..n {
            let v = fld[i] as f64 + delta[i] as f64;
            store_clamped01(&mut fld[i], v);
        }
    }
}

// ===================================================================
// coastal processes
// ===================================================================

/// `state.coastal` (reference HTML line 2275) — the *Coastal* button's four
/// knobs. Reference defaults `0.5 / 0.08 / 0.03 / 4`.
#[derive(Clone, Copy, Debug)]
pub struct CoastalParams {
    /// Wave energy. Divided by planet gravity inside [`coastal_process`]
    /// (wave energy ∝ 1/g), which is where the reference does it too.
    pub wave_str: f64,
    /// How far below the coast a river mouth still widens into an estuary.
    pub estuary_depth: f64,
    /// Height band above sea level where tidal marsh accretes.
    pub marsh_band: f64,
    pub passes: i32,
}

/// `slopeAt()` (reference HTML line 7584) — the central-difference slope
/// magnitude the marsh pass reads. Local to this module because it samples
/// the field **as it is being mutated**, which is the reference's behaviour
/// and not something a shared, snapshot-taking helper would reproduce.
fn slope_at(fld: &[f32], w: usize, h: usize, world: bool, x: usize, y: usize) -> f64 {
    let xl = if world {
        (x + w - 1) % w
    } else if x > 0 {
        x - 1
    } else {
        x
    };
    let xr = if world {
        (x + 1) % w
    } else if x < w - 1 {
        x + 1
    } else {
        x
    };
    let l = fld[y * w + xl] as f64;
    let r = fld[y * w + xr] as f64;
    let u = if y > 0 { fld[(y - 1) * w + x] } else { fld[y * w + x] } as f64;
    let d = if y < h - 1 { fld[(y + 1) * w + x] } else { fld[y * w + x] } as f64;
    js_hypot((r - l) * 0.5, (d - u) * 0.5)
}

/// `coastalProcess()` + `coastalProcessCPU()` (reference HTML lines
/// 4388-4424) — sea-cliff retreat, estuary widening and tidal-marsh
/// deposition, as one pass over a finished surface.
///
/// Three effects, in the reference's order:
/// 1. **Wave erosion**, `passes` times: every land cell touching the sea
///    loses height in proportion to how much of its 4-neighbourhood is sea,
///    and a quarter of that goes to each of its *land* neighbours — cliffs
///    retreat and the debris builds the shore platform behind them.
/// 2. **Estuary widening**, in the same passes: a cell carrying more than
///    `w·h·0.001` of discharge and sitting within `estuary_depth` of sea
///    level is cut down further, log-scaled by how much discharge it carries.
/// 3. **Tidal marsh**, once, after both: cells in the `marsh_band` above sea
///    level with a slope under 0.08 get a noise-textured accretion, faded out
///    as the slope approaches that limit.
///
/// The marsh pass stays separate from the two banded ones for the reason the
/// reference gives — it is per-cell value noise, which its GPU path could not
/// take — and it reads `slope_at` against the *already wave-eroded* field,
/// in place, cell by cell.
///
/// `g` is planet gravity; `wave_str` is divided by `max(0.05, g)` before
/// anything reads it, so a low-gravity world has stronger surf. `flow` is the
/// discharge field (`flowField`); the wave and marsh halves ignore it.
#[allow(clippy::too_many_arguments)]
pub fn coastal_process(
    fld: &mut [f32],
    flow: &[f32],
    w: usize,
    h: usize,
    sea: f64,
    world: bool,
    g: f64,
    p: &CoastalParams,
) {
    const OFFS: [(i64, i64); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
    let n = w * h;
    let wave_str = p.wave_str / g.max(0.05);
    let river_thresh = w as f64 * h as f64 * 0.001;

    for _ in 0..p.passes {
        let mut delta = vec![0f32; n];
        for y in 0..h {
            for x in 0..w {
                let i = y * w + x;
                let hh = fld[i] as f64;
                if hh < sea {
                    continue;
                }
                let mut sea_nb = 0i32;
                for (dx, dy) in OFFS {
                    let nx = x as i64 + dx;
                    let ny = y as i64 + dy;
                    if nx < 0 || ny < 0 || nx >= w as i64 || ny >= h as i64 {
                        continue;
                    }
                    if (fld[(ny * w as i64 + nx) as usize] as f64) < sea {
                        sea_nb += 1;
                    }
                }
                if sea_nb > 0 {
                    let exposure = sea_nb as f64 / 4.0;
                    delta[i] = (delta[i] as f64 - wave_str * exposure * 0.002) as f32;
                    for (dx, dy) in OFFS {
                        let nx = x as i64 + dx;
                        let ny = y as i64 + dy;
                        if nx < 0 || ny < 0 || nx >= w as i64 || ny >= h as i64 {
                            continue;
                        }
                        let j = (ny * w as i64 + nx) as usize;
                        if fld[j] as f64 >= sea {
                            delta[j] = (delta[j] as f64 + wave_str * exposure * 0.0005) as f32;
                        }
                    }
                }
                let fl = flow[i] as f64;
                if fl > river_thresh && hh < sea + p.estuary_depth {
                    let ff = (js_log(fl / river_thresh) / 5.0).min(1.0);
                    let cut = 0.003 * ff * (1.0 - (hh - sea) / p.estuary_depth) * wave_str;
                    delta[i] = (delta[i] as f64 - cut) as f32;
                }
            }
        }
        for i in 0..n {
            let v = fld[i] as f64 + delta[i] as f64;
            store_clamped01(&mut fld[i], v);
        }
    }

    // tidal marsh: per-cell value noise, read against the live field
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            let hh = fld[i] as f64;
            if hh >= sea && hh < sea + p.marsh_band {
                let sl = slope_at(fld, w, h, world, x, y);
                if sl < 0.08 {
                    let fade = 1.0 - sl / 0.08;
                    let bump = (vnoise(x as f64 * 0.18, y as f64 * 0.18, 77) - 0.5)
                        * p.marsh_band
                        * 0.55
                        * fade;
                    let v = fld[i] as f64 + bump;
                    store_clamped01(&mut fld[i], v);
                }
            }
        }
    }
}

// ===================================================================
// sediment routing and deposition
// ===================================================================

/// `routeSediment()` (reference HTML lines 4286-4307) — mass-conserving
/// fluvial sediment routing: takes a per-cell `supply` (typically the column
/// an erosion pass just removed) down the steepest-descent network on the
/// *current* surface and drops it where transport capacity, `capacity ×
/// discharge × slope`, falls below the load.
///
/// Returns the total deposited. **Mass is conserved**: every unit either
/// deposits on the grid or pools in a sink, and in non-world mode nothing
/// leaves the grid at all.
///
/// Below sea level the rule changes — a cell builds toward sea level by half
/// the remaining depth and the rest progrades seaward, which is what makes
/// deltas and shelves rather than a single spike at the river mouth.
///
/// **The sort is the parity-critical part.** The reference sorts cell
/// indices by descending height with `sort((a,b) => fld[b]-fld[a])`, and
/// `Array.prototype.sort` has been required to be *stable* since ES2019 — so
/// cells of exactly equal height keep ascending index order, and that order
/// decides which of two tied cells routes first. `slice::sort_by` is stable
/// too, and the comparator below reproduces `SortCompare`'s own reading of a
/// numeric result (negative / positive / everything else, NaN included,
/// equal) rather than Rust's `partial_cmp`, which would panic on a NaN the
/// reference silently treats as a tie.
#[allow(clippy::too_many_arguments)]
pub fn route_sediment(
    fld: &mut [f32],
    disch: &[f32],
    supply: &[f32],
    w: usize,
    h: usize,
    sea: f64,
    capacity: f64,
    world: bool,
) -> f64 {
    let n = w * h;

    let mut recv = vec![-1i32; n];
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            let mut best = -1i32;
            let mut best_h = fld[i];
            for dy in -1i64..=1 {
                for dx in -1i64..=1 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let mut nx = x as i64 + dx;
                    let ny = y as i64 + dy;
                    if world {
                        nx = ((nx % w as i64) + w as i64) % w as i64;
                    } else if nx < 0 || nx >= w as i64 {
                        continue;
                    }
                    if ny < 0 || ny >= h as i64 {
                        continue;
                    }
                    let j = (ny * w as i64 + nx) as usize;
                    if fld[j] < best_h {
                        best_h = fld[j];
                        best = j as i32;
                    }
                }
            }
            recv[i] = best;
        }
    }

    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by(|&a, &b| {
        let v = fld[b] as f64 - fld[a] as f64;
        if v < 0.0 {
            Ordering::Less
        } else if v > 0.0 {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    });

    let mut carried: Vec<f32> = supply.to_vec();
    let mut deposited = 0.0f64;
    for &i in &order {
        let load = carried[i] as f64;
        if load <= 1e-12 {
            continue;
        }
        let r = recv[i];
        if r < 0 {
            // sink / pit / deepest basin: pool everything
            fld[i] = (fld[i] as f64 + load) as f32;
            deposited += load;
            carried[i] = 0.0;
            continue;
        }
        let r = r as usize;
        let dep = if (fld[i] as f64) < sea {
            load.min((sea - fld[i] as f64) * 0.5)
        } else {
            let slope = (fld[i] as f64 - fld[r] as f64).max(0.0);
            let cap = capacity * (disch[i] as f64).max(0.0) * slope;
            if load > cap {
                load - cap
            } else {
                0.0
            }
        };
        if dep > 0.0 {
            fld[i] = (fld[i] as f64 + dep) as f32;
            deposited += dep;
        }
        carried[r] = (carried[r] as f64 + (load - dep)) as f32;
    }
    deposited
}

/// `applyTidalSedimentation()` (reference HTML lines 4324-4334) — the
/// *Tidal flats* button's kernel: shallow submerged cells inside the spring
/// tidal range accrete toward sea level, hardest where the water is
/// shallowest, building mudflats and estuary fill.
///
/// `tide` is the reference's `tideField` — **which this port does not
/// generate** (`GUI_GAP_REGISTER.md` WW-07: geoid and tides are unported
/// default-off sub-systems). The reference's own op is equally conditional
/// (`if(!tideField) return;`), so this takes the field as an argument and
/// leaves producing one to whoever ports `computeTideField`.
///
/// `k` is the accretion rate; the reference's own default, and its only
/// caller's, is `0.45`. Returns the total deposited.
pub fn apply_tidal_sedimentation(
    fld: &mut [f32],
    tide: &[f32],
    sea: f64,
    w: usize,
    h: usize,
    k: f64,
) -> f64 {
    let n = w * h;
    let mut deposited = 0.0f64;
    for i in 0..n {
        let tr = tide[i] as f64;
        if tr <= 1e-5 {
            continue;
        }
        let hh = fld[i] as f64;
        if hh >= sea {
            continue; // submerged cells only
        }
        let depth = sea - hh;
        if depth >= tr {
            continue; // intertidal / shallow-subtidal band only
        }
        let accr = (sea - 1e-4 - hh).min(k * tr * (1.0 - depth / tr));
        if accr > 0.0 {
            fld[i] = (fld[i] as f64 + accr) as f32;
            deposited += accr;
        }
    }
    deposited
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store_clamped01_clamps_at_both_ends_and_after_rounding() {
        let mut v = 0.5f32;
        store_clamped01(&mut v, -3.0);
        assert_eq!(v, 0.0);
        store_clamped01(&mut v, 7.5);
        assert_eq!(v, 1.0);
        store_clamped01(&mut v, 0.25);
        assert_eq!(v, 0.25);
    }

    #[test]
    fn bilin_clamps_outside_the_grid_and_interpolates_inside() {
        let a = [0.0f32, 1.0, 2.0, 3.0]; // 2x2
        assert_eq!(bilin(&a, -5.0, -5.0, 2, 2), 0.0);
        assert_eq!(bilin(&a, 99.0, 99.0, 2, 2), 3.0);
        assert_eq!(bilin(&a, 0.5, 0.0, 2, 2), 0.5);
        assert_eq!(bilin(&a, 0.5, 0.5, 2, 2), 1.5);
    }

    #[test]
    fn centrifugal_shear_is_zero_below_the_speed_floor() {
        assert_eq!(centrifugal_shear(1e-7, 0.0, 1.0, 1.0), (0.0, 0.0, 0.0));
    }

    #[test]
    fn hillslope_diffuse_wraps_in_x_only_when_world() {
        // One spike on the left edge. In world mode the right edge feels it.
        let mut flat = vec![0.5f32; 8];
        flat[0] = 1.0;
        let mut wrapped = flat.clone();
        hillslope_diffuse(&mut flat, 8, 1, 1, 0.2, false);
        hillslope_diffuse(&mut wrapped, 8, 1, 1, 0.2, true);
        assert_eq!(flat[7], 0.5, "no wrap: the far edge must not see the spike");
        assert!(wrapped[7] > 0.5, "world wrap: the far edge must see it");
    }

    #[test]
    fn route_sediment_conserves_mass_into_a_single_pit() {
        // A 3x3 bowl: everything drains to the middle, which is a pit.
        let mut fld = vec![0.9f32; 9];
        fld[4] = 0.1;
        let disch = vec![0.0f32; 9];
        let supply = vec![0.01f32; 9];
        let before: f64 = fld.iter().map(|&v| v as f64).sum();
        let deposited = route_sediment(&mut fld, &disch, &supply, 3, 3, 0.0, 6.0, false);
        let after: f64 = fld.iter().map(|&v| v as f64).sum();
        assert!((deposited - 0.09).abs() < 1e-6, "all 9 x 0.01 must land");
        assert!((after - before - deposited).abs() < 1e-6);
    }

    #[test]
    fn apply_tidal_sedimentation_ignores_land_and_deep_water() {
        let mut fld = vec![0.60f32, 0.41, 0.10];
        let tide = vec![0.05f32, 0.05, 0.05];
        let deposited = apply_tidal_sedimentation(&mut fld, &tide, 0.42, 3, 1, 0.45);
        assert_eq!(fld[0], 0.60, "above sea level: untouched");
        assert!(fld[1] > 0.41, "inside the tidal band: accretes");
        assert_eq!(fld[2], 0.10, "deeper than the tidal range: untouched");
        assert!(deposited > 0.0);
    }
}
