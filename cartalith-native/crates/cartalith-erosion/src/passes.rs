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
/// Not `f32::clamp`: this is the reference's own two-statement
/// `if(f<0)f=0; if(f>1)f=1;`, transcribed. `clamp` would read the same today
/// and would quietly stop reading like the source it is being checked against.
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

/// Cell size the hillslope coefficient is calibrated at: the app's own untouched
/// default, 800 km across 2048 cells.
///
/// Same **anchor-at-the-default** discipline `terrain_detail_k`
/// (`REF_CELLKM = 800/2048`) and `_V3D_RATIO0` already use, for the same
/// reason: at the anchor the correction is exactly `1.0`, so the default
/// configuration — and every golden fixture — is bit-identical.
pub const HILLSLOPE_REF_CELL_KM: f64 = 800.0 / 2048.0;

/// Stability bound for the explicit FTCS Laplacian below.
///
/// `delta = d * (l + r + u + dn - 4h)` is forward-Euler on a 5-point stencil,
/// which is stable only for `d <= 1/4`. Past it the scheme diverges and the
/// height field explodes — and that does not present as a numerical bug, it
/// presents as bad terrain, which is far harder to diagnose.
///
/// **This is a ceiling on the correction below, not on `d` itself.** The
/// reference permits `d` up to 0.9 and `golden_parity_passes`'
/// `hillslope_diffuse_case_2` pins that behaviour at `d = 0.9`. Parity is the
/// contract, so an already-unstable `d` the caller chose is passed through
/// untouched; what must never happen is this port *amplifying* a stable `d`
/// into an unstable one through its own scale correction.
pub const HILLSLOPE_STABLE_D: f64 = 0.25;

/// How much to scale the hillslope coefficient for a map of this real extent.
///
/// # The defect this exists to fix
///
/// [`hillslope_diffuse`] computes its Laplacian at a grid spacing of exactly
/// one cell, so `delta = d * dx^2 * grad^2(z_real)`. Matching the physical
/// `dz = D * dt / dx^2 * grad^2(z)` therefore requires `d` to scale as
/// `1 / cell_km^2` — and it did not, at all. The same literal `diffuse_d`
/// applied at every extent.
///
/// At 2048 cells a 5 km region has 0.00244 km cells and a 40 000 km world has
/// 19.53 km cells; the ratio of cell **areas** is 8000^2 = **64 000 000** — the
/// identical figure `DECISIONS.md` §7l cites for the crater count, one layer
/// up in the pipeline. Verified: `cartalith-erosion` contained zero
/// occurrences of `map_width_km` before this.
///
/// # Why it is one-sided
///
/// Reducing `d` for a coarse map is unconditionally safe — a 19.5 km cell
/// should diffuse far less per pass than a 390 m one, and a smaller `d` only
/// moves further inside the stability bound. **Raising it for a fine map is
/// not**: the correction at 5 km / 2048 is ~25 600x, which would take a
/// `d` of 0.15 to 3840 and detonate the field. So the increase is capped at
/// [`HILLSLOPE_STABLE_D`].
///
/// That cap is a real ceiling, honestly: a very fine region cannot express its
/// full physical diffusion in one pass, and the correct upgrade is **more
/// passes**, not a larger coefficient — the explicit scheme's own constraint,
/// not a shortcut. Recorded rather than hidden.
///
/// ponytail: the scale is computed here and passed in, rather than threading
/// `map_width_km` into a kernel that has never known about real extent, so the
/// kernel stays a pure function of its own arguments and every existing
/// caller keeps working by passing `1.0`.
pub fn hillslope_extent_scale(map_width_km: f64, gw: usize, d: f64) -> f64 {
    if !(map_width_km > 0.0) || gw == 0 || !(d > 0.0) {
        return 1.0;
    }
    let cell_km = map_width_km / gw as f64;
    if !(cell_km > 0.0) {
        return 1.0;
    }
    let raw = (HILLSLOPE_REF_CELL_KM / cell_km).powi(2);
    if raw <= 1.0 {
        // Coarser than the anchor: always safe to apply in full.
        return raw;
    }
    // Finer than the anchor: allow the increase only up to the stability bound,
    // and never below 1.0 (which would silently *weaken* a `d` the caller
    // already chose above the bound -- see HILLSLOPE_STABLE_D).
    raw.min((HILLSLOPE_STABLE_D / d).max(1.0))
}

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
///
/// `d_scale` multiplies `d` and is **exactly `1.0` for the reference's own
/// behaviour** — every golden fixture passes it, so they are bit-identical.
/// `generate_terrain` passes [`hillslope_extent_scale`] instead, which is what
/// makes the pass aware of the map's real extent (`DECISIONS.md` §7m).
pub fn hillslope_diffuse(
    fld: &mut [f32],
    w: usize,
    h: usize,
    passes: i32,
    d: f64,
    d_scale: f64,
    world: bool,
) {
    let d = d * d_scale;
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
/// The two `if x > hi {..} else if x < lo {..}` clamps in step 5 are the
/// reference's own statement shape, kept literally; see `store_clamped01`.
#[allow(clippy::manual_clamp)]
pub fn velocity_erode_kernel(
    fld: &mut [f32],
    rain: Option<&[f32]>,
    w: usize,
    h: usize,
    p: &VelocityParams,
) -> VelocityField {
    // `pipeDamp` (reference HTML, `velocityErodeKernel`): virtual-pipe flux
    // damping so the flux solver doesn't oscillate. Carried from the
    // reference's own literal; provenance beyond that (a value from Mei et
    // al. 2007 versus a tuned constant) is not recorded here.
    const PIPE_DAMP: f64 = 0.92;
    // Velocity ceiling the reference clamps advection to, preventing a single
    // unstable cell's flux from throwing sediment arbitrarily far in one
    // semi-Lagrangian step. Reference's own literal.
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
    glacial_core(fld, temp, w, h, p, None);
}

/// GF-4's rock input to the glacial and coastal passes
/// (`GEOLOGY_FIRST_SCOPE.md` §4.4, §4.5; owner Rulings BH and BJ): the GF-1
/// column, read through §2.5's exposure rule, and the rock contrast `c`.
///
/// Read only. Neither pass writes the column: §4.9 has the **caller**
/// account the call's net change (gains become regolith, losses strip it
/// first, `crate::account_regolith`), as GF-2 does for stream power, so the
/// regolith a call reads is the one it started with.
pub struct PassRock<'a> {
    /// The GF-1 column. Read: `rock_top`, `rock_sub`, `contact`, `regolith`.
    pub column: &'a cartalith_terrain::geology::GeologyColumn,
    /// §4.1's rock contrast `c` (0 = uniform rock); the engine passes
    /// `tect.resist`, as for stream power (§9 Q5's default).
    pub contrast: f64,
    /// §2.5's `R_EXPOSE` in normalised height units
    /// (`geology::m_to_norm(R_EXPOSE_M, sea, peak_m)`).
    pub r_expose: f32,
}

impl PassRock<'_> {
    /// A per-rock lookup table of `prop(rock)^c`, indexed by the rock's code,
    /// so the per-cell cost is the exposure rule and one load (the same
    /// shape as stream power's `kc` table).
    fn table(&self, prop: impl Fn(&cartalith_terrain::geology::RockProps) -> f32) -> [f64; cartalith_terrain::geology::ROCK_COUNT] {
        cartalith_terrain::geology::Rock::ALL.map(|r| (prop(r.props()) as f64).powf(self.contrast))
    }

    /// The exposed rock's code at cell `i` for surface height `z`. Panics,
    /// as `stream_power_kernel_rock` does, on a column cell holding no rock:
    /// a mismatched column describes a different world, and a fallback
    /// factor would be a plausible number standing in for none.
    fn exposed_code(&self, i: usize, z: f32) -> usize {
        self.column
            .exposed(i, z, self.r_expose)
            .unwrap_or_else(|| panic!("GF-4: cell {i} of the rock column holds no valid rock (rock_top {})", self.column.rock_top[i]))
            as usize
    }
}

/// The glacial-erodibility multiplier of one rock at contrast `c`: `γ^c`
/// (`GEOLOGY_FIRST_SCOPE.md` §4.4). `γ` is `ROCK_PROPS`' judgement column
/// (§2.3: "Judgement. GF-4 reads it"). Exactly `1.0` for every rock at
/// `c = 0` (`powf(0) = 1`).
pub fn gamma_multiplier(rock: cartalith_terrain::geology::Rock, contrast: f64) -> f64 {
    (rock.props().gamma as f64).powf(contrast)
}

/// The cliff-retreat multiplier of one rock at contrast `c`: `ρ^c`
/// (`GEOLOGY_FIRST_SCOPE.md` §4.5). `ρ` is `ROCK_PROPS`' judgement column
/// (§2.3: "Judgement. GF-4 reads it"). Exactly `1.0` at `c = 0`.
pub fn rho_multiplier(rock: cartalith_terrain::geology::Rock, contrast: f64) -> f64 {
    (rock.props().rho as f64).powf(contrast)
}

/// [`glacial_kernel`] reading the GF-1 rock column (`GEOLOGY_FIRST_SCOPE.md`
/// §4.4, milestone GF-4): `e_i = kg·γ(exposed_i)^c·g·Q_i^mg·S_i·0.001`, and
/// the trunk's flank (`u_factor`) and cirque (`0.6`) terms scale with the
/// same `e_i`, as §4.4 says.
///
/// - **Exposed rock** is read per pass at the cell's height at the start of
///   that pass (the kernel accumulates a pass into `delta` and applies it at
///   the end, so every read inside a pass sees the same surface): a cap the
///   ice cuts through in one pass is eroded as substrate from the next.
/// - **Why a separate entry point**: the legacy body is a golden-verified
///   port (`tests/golden_parity_passes.rs`); [`glacial_kernel`] passes no rock
///   to the shared core, which then takes the legacy arithmetic by control
///   flow, not by an identity of arithmetic (`MISTAKES.md`, "Change generated
///   output").
///
/// Must never write the column (the caller accounts regolith, §4.9), and
/// must never be called with a column shorter than the grid (it asserts).
pub fn glacial_kernel_rock(fld: &mut [f32], temp: &[f32], w: usize, h: usize, p: &GlacialParams, rock: &PassRock) {
    assert!(rock.column.len() == w * h, "rock column is {} cells, needs exactly {} ({w}x{h})", rock.column.len(), w * h);
    glacial_core(fld, temp, w, h, p, Some(rock));
}

/// The body of [`glacial_kernel`] and [`glacial_kernel_rock`]. With `rock`
/// `None` every statement is the reference port's, unchanged.
fn glacial_core(fld: &mut [f32], temp: &[f32], w: usize, h: usize, p: &GlacialParams, rock: Option<&PassRock>) {
    // GF-4: `γ^c` per rock, built once per call; `None` without rock.
    let gamma_c = rock.map(|r| r.table(|props| props.gamma));
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

    // Priority-flood fill step, same value and role as `stream_power_routing`'s
    // own `EPS` in `lib.rs` — the reference's literal, nudging each newly
    // filled neighbour strictly above its discoverer so ties break by fill
    // order rather than leaving equal heights.
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
            // GF-4 (§4.4): scale by the exposed rock's `γ^c`. `fld` is not
            // written until this pass's `delta` is applied below, so this is
            // the pass-start surface. Without rock, `e` is the reference's.
            let e = match (rock, gamma_c.as_ref()) {
                (Some(r), Some(t)) => e * t[r.exposed_code(i, fld[i])],
                _ => e,
            };
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

/// The lowest of a cell's eight neighbours' heights (x wrapped when
/// `world`; rows never wrap; off-grid neighbours skipped): the cell's
/// steepest-descent ("downstream") neighbour's height, which the estuary
/// cut may not go below (Ruling CH addendum 2). `+inf` only for a 1 x 1
/// grid, which has no neighbour, so the cap is 0 and it gets no estuary cut.
fn lowest_neighbour(fld: &[f32], w: usize, h: usize, world: bool, x: usize, y: usize) -> f64 {
    let mut lo = f64::INFINITY;
    for dy in -1i64..=1 {
        for dx in -1i64..=1 {
            if dx == 0 && dy == 0 {
                continue;
            }
            let ny = y as i64 + dy;
            if ny < 0 || ny >= h as i64 {
                continue;
            }
            let mut nx = x as i64 + dx;
            if world {
                nx = nx.rem_euclid(w as i64);
            } else if nx < 0 || nx >= w as i64 {
                continue;
            }
            lo = lo.min(fld[ny as usize * w + nx as usize] as f64);
        }
    }
    lo
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
    coastal_core(fld, flow, w, h, sea, world, g, p, None);
}

/// [`coastal_process`] reading the GF-1 rock column (`GEOLOGY_FIRST_SCOPE.md`
/// §4.5, milestone GF-4): the wave cut, the debris each cut cell sends to its
/// land neighbours, and the estuary cut are each multiplied by the cutting
/// cell's `ρ(exposed)^c`, so soft rock retreats into bays and hard rock
/// stands as headlands. The tidal marsh does not change.
///
/// - **Exposed rock** is read at the cell's height at the start of each pass
///   (the pass accumulates into `delta` before applying it).
/// - **Debris is regolith** (§4.5, §4.9) by the caller's accounting of the
///   call's net change (`crate::account_regolith` with gains as deposit);
///   this kernel never writes the column.
/// - **Why a separate entry point**: as for [`glacial_kernel_rock`], the
///   legacy body is golden-verified (`tests/golden_parity_passes.rs`) and
///   takes the reference's arithmetic by control flow when no rock is given.
///
/// Must never be called with a column shorter than the grid (it asserts).
#[allow(clippy::too_many_arguments)]
pub fn coastal_process_rock(
    fld: &mut [f32],
    flow: &[f32],
    w: usize,
    h: usize,
    sea: f64,
    world: bool,
    g: f64,
    p: &CoastalParams,
    rock: &PassRock,
) {
    assert!(rock.column.len() == w * h, "rock column is {} cells, needs exactly {} ({w}x{h})", rock.column.len(), w * h);
    coastal_core(fld, flow, w, h, sea, world, g, p, Some(rock));
}

/// The body of [`coastal_process`] and [`coastal_process_rock`]. With `rock`
/// `None` every statement is the reference port's, unchanged.
#[allow(clippy::too_many_arguments)]
fn coastal_core(
    fld: &mut [f32],
    flow: &[f32],
    w: usize,
    h: usize,
    sea: f64,
    world: bool,
    g: f64,
    p: &CoastalParams,
    rock: Option<&PassRock>,
) {
    // GF-4: `ρ^c` per rock, built once per call; `None` without rock.
    let rho_c = rock.map(|r| r.table(|props| props.rho));
    // The 4-connected (not 8-connected) neighbourhood the reference's own
    // coastal pass uses for counting sea-adjacency and spreading wave
    // erosion — orthogonal only, no diagonals.
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
                // GF-4 (§4.5): this cell's `ρ(exposed)^c` at the pass-start
                // surface (`fld` is written only after the pass), or `None`
                // without rock, where every statement below is the
                // reference's own.
                let rf = match (rock, rho_c.as_ref()) {
                    (Some(r), Some(t)) => Some(t[r.exposed_code(i, fld[i])]),
                    _ => None,
                };
                if sea_nb > 0 {
                    let exposure = sea_nb as f64 / 4.0;
                    let cut = wave_str * exposure * 0.002;
                    let cut = rf.map_or(cut, |f| cut * f);
                    delta[i] = (delta[i] as f64 - cut) as f32;
                    for (dx, dy) in OFFS {
                        let nx = x as i64 + dx;
                        let ny = y as i64 + dy;
                        if nx < 0 || ny < 0 || nx >= w as i64 || ny >= h as i64 {
                            continue;
                        }
                        let j = (ny * w as i64 + nx) as usize;
                        if fld[j] as f64 >= sea {
                            // The debris is the cutting cell's, so it scales
                            // with the cutting cell's rock (§4.5).
                            let debris = wave_str * exposure * 0.0005;
                            let debris = rf.map_or(debris, |f| debris * f);
                            delta[j] = (delta[j] as f64 + debris) as f32;
                        }
                    }
                }
                let fl = flow[i] as f64;
                if fl > river_thresh && hh < sea + p.estuary_depth {
                    let ff = (js_log(fl / river_thresh) / 5.0).min(1.0);
                    let cut = 0.003 * ff * (1.0 - (hh - sea) / p.estuary_depth) * wave_str;
                    let cut = rf.map_or(cut, |f| cut * f);
                    // Capped at the drop to the cell's downstream neighbour
                    // (Ruling CH addendum 2, 2026-10-13; a DEVIATION from
                    // the reference, which cuts by `cut` whatever the
                    // neighbours). The cut reaches `estuary_depth` above
                    // sea (about 552 m at the default, far up every large
                    // river) and scales with `ln(flow)`, so neighbouring
                    // channel cells were lowered by different amounts and
                    // some ended below every neighbour: closed hollows the
                    // lake classifier counts as lakes (seed 24601,
                    // processes on: 169 one- to three-cell lakes with the
                    // cut against 53 without it, control 49;
                    // `GEOLOGY_FIRST_SCOPE.md` §7 "GF-4: accretion-only
                    // marsh and the second lake cause"). The downstream
                    // neighbour is the lowest of the eight, read on the
                    // pass-start surface (`fld` is written only after the
                    // pass), x wrapped on a world map; a cell already lower
                    // than all eight is not cut at all.
                    let cut = cut.min((hh - lowest_neighbour(fld, w, h, world, x, y)).max(0.0));
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
                    // Accretion only (Ruling CH addendum, 2026-10-13; a
                    // DEVIATION from the reference, which adds the centred
                    // bump both ways). A marsh builds up and does not dig:
                    // the negative half of the noise dug one- to three-cell
                    // hollows on flat lowland, which the lake classifier
                    // counted as lakes (measured: with the marsh off, the
                    // app world's 1-3-cell lake count with the coastal pass
                    // on fell from 205 to 69 against a control of 68 on
                    // seed 24601; `GEOLOGY_FIRST_SCOPE.md` §7 "GF-4 under
                    // Ruling CH"). That a marsh only accretes is a physical
                    // reading, not a value from a source at this line.
                    // `golden_parity_passes.rs`' coastal cases were
                    // re-recorded for it (DECISIONS §7p).
                    if bump > 0.0 {
                        let v = fld[i] as f64 + bump;
                        store_clamped01(&mut fld[i], v);
                    }
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

/// Unit tests for this module's small internal helpers and one manual pass
/// each, distinct from the crate-level `tests/golden_parity_passes.rs`
/// bit-exact fixtures.
#[cfg(test)]
mod tests {
    use super::*;

    /// Exercises all three regions of the clamp: below zero, above one, and
    /// the pass-through middle.
    #[test]
    fn store_clamped01_clamps_at_both_ends_and_after_rounding() {
        // Protects: store_clamped01's two-sided clamp (low, high, and the
        // pass-through middle) against the reference's own two-statement form.
        let mut v = 0.5f32;
        store_clamped01(&mut v, -3.0);
        assert_eq!(v, 0.0);
        store_clamped01(&mut v, 7.5);
        assert_eq!(v, 1.0);
        store_clamped01(&mut v, 0.25);
        assert_eq!(v, 0.25);
    }

    /// Two out-of-bounds corners (clamped to the nearest edge value) and two
    /// in-bounds fractional positions (linear and bilinear).
    #[test]
    fn bilin_clamps_outside_the_grid_and_interpolates_inside() {
        // Protects: bilin's out-of-bounds clamp-to-edge behaviour and its
        // in-bounds bilinear interpolation, on a minimal 2x2 grid.
        let a = [0.0f32, 1.0, 2.0, 3.0]; // 2x2
        assert_eq!(bilin(&a, -5.0, -5.0, 2, 2), 0.0);
        assert_eq!(bilin(&a, 99.0, 99.0, 2, 2), 3.0);
        assert_eq!(bilin(&a, 0.5, 0.0, 2, 2), 0.5);
        assert_eq!(bilin(&a, 0.5, 0.5, 2, 2), 1.5);
    }

    /// A near-zero speed input, below whatever floor the reference guards
    /// meander shear with.
    #[test]
    fn centrifugal_shear_is_zero_below_the_speed_floor() {
        // Protects: centrifugal_shear's speed-floor gate -- below it the
        // function must return exactly zero shear, not a tiny nonzero value.
        assert_eq!(centrifugal_shear(1e-7, 0.0, 1.0, 1.0), (0.0, 0.0, 0.0));
    }

    /// Same spike run twice, once non-wrapping and once world-wrapping, and
    /// compared at the far edge.
    #[test]
    fn hillslope_diffuse_wraps_in_x_only_when_world() {
        // Protects: hillslope_diffuse's `wrap` flag actually gates x-wrapping
        // -- a spike on the left edge must not reach the right edge unless
        // `world` is true.
        // One spike on the left edge. In world mode the right edge feels it.
        let mut flat = vec![0.5f32; 8];
        flat[0] = 1.0;
        let mut wrapped = flat.clone();
        hillslope_diffuse(&mut flat, 8, 1, 1, 0.2, 1.0, false);
        hillslope_diffuse(&mut wrapped, 8, 1, 1, 0.2, 1.0, true);
        assert_eq!(flat[7], 0.5, "no wrap: the far edge must not see the spike");
        assert!(wrapped[7] > 0.5, "world wrap: the far edge must see it");
    }

    /// A 3x3 bowl where every cell's supply drains into the single central
    /// pit, checked for exact mass conservation.
    #[test]
    fn route_sediment_conserves_mass_into_a_single_pit() {
        // Protects: route_sediment's mass conservation -- everything supplied
        // into a single pit must be deposited, and the field's total mass
        // must rise by exactly the reported deposited amount.
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

    /// Three cells spanning land, the intertidal band and deep water, so all
    /// three branches of the depth gate are exercised at once.
    #[test]
    fn apply_tidal_sedimentation_ignores_land_and_deep_water() {
        // Protects: apply_tidal_sedimentation's band gate -- only the
        // intertidal cell accretes; land above sea level and water deeper
        // than the tidal range are both left untouched.
        let mut fld = vec![0.60f32, 0.41, 0.10];
        let tide = vec![0.05f32, 0.05, 0.05];
        let deposited = apply_tidal_sedimentation(&mut fld, &tide, 0.42, 3, 1, 0.45);
        assert_eq!(fld[0], 0.60, "above sea level: untouched");
        assert!(fld[1] > 0.41, "inside the tidal band: accretes");
        assert_eq!(fld[2], 0.10, "deeper than the tidal range: untouched");
        assert!(deposited > 0.0);
    }
}

/// Unit tests for [`hillslope_extent_scale`] -- the real-extent/stability
/// correction to `hillslope_diffuse`'s coefficient, none of which the
/// bit-exact `golden_parity_passes.rs` fixtures exercise directly since they
/// pin the anchor configuration where the correction is exactly `1.0`.
#[cfg(test)]
mod hillslope_extent_tests {
    use super::*;

    /// The anchor is the whole safety story: at the app's own default the
    /// correction is exactly 1.0, so `WorldParams::defaults()` output — and
    /// every golden fixture — is unchanged, bit for bit.
    #[test]
    fn the_correction_is_exactly_one_at_the_anchor() {
        // Protects: hillslope_extent_scale returns exactly 1.0 at the app's
        // own default cell size, so every golden fixture stays bit-identical.
        assert_eq!(hillslope_extent_scale(800.0, 2048, 0.15), 1.0);
        // Same cell size reached a different way must agree exactly.
        assert_eq!(hillslope_extent_scale(400.0, 1024, 0.15), 1.0);
    }

    /// The defect, stated as a test: a coarse world must diffuse far less per
    /// pass than the anchor, because one 19.5 km cell is not one 390 m cell.
    ///
    /// Two ratios live here and they are easy to confuse — I did, first time.
    /// Against the **anchor**, 40 000 km / 2048 is 50x the cell size, so 2 500x
    /// less diffusion. The **64 000 000x** figure `DECISIONS.md` §7l cites is
    /// the span between the two extremes of the app's own range, 5 km to
    /// 40 000 km: 8 000x in cell size, squared. That full span is the raw
    /// physical correction and is *not* observable through this function,
    /// because the fine end is clamped at the stability bound — which is the
    /// next test.
    #[test]
    fn a_coarse_world_diffuses_far_less_than_the_anchor() {
        // Protects: hillslope_extent_scale's real-extent correction actually
        // shrinks diffusion at a coarse cell size, by the ratio the physical
        // model predicts (both the clamped anchor-relative ratio and the
        // raw, pre-clamp span across the app's whole extent range).
        let world = hillslope_extent_scale(40_000.0, 2048, 0.15);
        assert!(world < 1.0e-3, "40 000 km / 2048 should be tiny, got {world}");
        let vs_anchor = 1.0 / world;
        assert!(
            (2_400.0..2_600.0).contains(&vs_anchor),
            "expected ~2 500x less than the anchor, got {vs_anchor}"
        );
        // And the raw, pre-clamp span across the app's whole extent range is
        // the 64 000 000x the crater ruling names. Computed from cell sizes
        // directly, since the clamp hides it in the returned value.
        let raw = |w: f64| (HILLSLOPE_REF_CELL_KM / (w / 2048.0)).powi(2);
        let span = raw(5.0) / raw(40_000.0);
        assert!(
            (6.0e7..7.0e7).contains(&span),
            "expected ~64 000 000x across 5 km..40 000 km, got {span}"
        );
    }

    /// **The one that stops an exploded height field.** A fine region wants a
    /// ~25 600x increase, which would take d = 0.15 to 3840 and detonate an
    /// explicit FTCS scheme whose stability bound is 0.25.
    #[test]
    fn a_fine_region_is_capped_at_the_stability_bound() {
        // Protects: hillslope_extent_scale never lets the effective
        // diffusion coefficient (d * scale) exceed the explicit FTCS
        // stability wall, across several fine-cell configurations.
        for (w, gw, d) in [(5.0, 2048, 0.15), (1.0, 2048, 0.2), (50.0, 4096, 0.05)] {
            let s = hillslope_extent_scale(w, gw, d);
            let effective = d * s;
            assert!(
                effective <= HILLSLOPE_STABLE_D + 1e-12,
                "{w} km / {gw} at d={d} gives d_eff={effective}, past the {HILLSLOPE_STABLE_D} wall"
            );
            assert!(s >= 1.0, "a finer-than-anchor map should never diffuse less: {s}");
        }
    }

    /// A `d` the caller already chose above the stability bound is passed
    /// through untouched rather than quietly weakened — `golden_parity_passes`'
    /// `hillslope_diffuse_case_2` pins the reference at d = 0.9, and parity is
    /// the contract.
    #[test]
    fn an_already_unstable_d_is_never_reduced_by_the_correction() {
        // Protects: hillslope_extent_scale's stability guard only ever
        // scales down toward the wall -- it must never weaken a `d` the
        // caller (or a golden fixture) already chose above it.
        assert_eq!(hillslope_extent_scale(800.0, 2048, 0.9), 1.0);
        assert_eq!(hillslope_extent_scale(5.0, 2048, 0.9), 1.0, "must not scale it down");
        // ...but a coarse map still reduces it, which is correct: that is the
        // real-extent term, not the stability guard.
        assert!(hillslope_extent_scale(40_000.0, 2048, 0.9) < 1.0);
    }

    /// Degenerate inputs return the identity rather than a NaN or an infinity
    /// that would silently poison the field.
    #[test]
    fn degenerate_inputs_return_the_identity() {
        // Protects: hillslope_extent_scale never returns NaN or infinity on
        // degenerate inputs (zero/negative extent, zero grid width, zero d,
        // NaN extent) -- it falls back to the identity scale instead.
        assert_eq!(hillslope_extent_scale(0.0, 2048, 0.15), 1.0);
        assert_eq!(hillslope_extent_scale(-5.0, 2048, 0.15), 1.0);
        assert_eq!(hillslope_extent_scale(800.0, 0, 0.15), 1.0);
        assert_eq!(hillslope_extent_scale(800.0, 2048, 0.0), 1.0);
        assert_eq!(hillslope_extent_scale(f64::NAN, 2048, 0.15), 1.0);
        assert!(hillslope_extent_scale(800.0, 2048, 0.15).is_finite());
    }

    /// Monotonic in extent: a larger map never diffuses more per pass.
    #[test]
    fn the_correction_is_monotonic_in_extent() {
        // Protects: hillslope_extent_scale is monotonically non-increasing
        // in map extent -- a larger map must never diffuse more per pass
        // than a smaller one.
        let mut prev = f64::INFINITY;
        for w in [1.0, 10.0, 100.0, 800.0, 5_000.0, 40_000.0] {
            let s = hillslope_extent_scale(w, 2048, 0.15);
            assert!(s <= prev + 1e-12, "not monotonic at {w} km: {s} after {prev}");
            prev = s;
        }
    }

    /// And it reaches the kernel: scaling must actually change the field, or
    /// the whole correction is dead code.
    #[test]
    fn the_scale_actually_reaches_the_kernel() {
        // Protects: the extent-scale correction is actually wired into
        // hillslope_diffuse's call site, not computed and discarded -- a
        // near-zero scale must visibly starve the diffusion versus the
        // unscaled run.
        let base: Vec<f32> = (0..64).map(|i| if i % 8 < 4 { 0.8 } else { 0.2 }).collect();
        let mut full = base.clone();
        let mut damped = base.clone();
        hillslope_diffuse(&mut full, 8, 8, 2, 0.15, 1.0, false);
        hillslope_diffuse(&mut damped, 8, 8, 2, 0.15, 1.0e-4, false);
        assert_ne!(full, damped, "d_scale changed nothing -- the parameter is dead");
        // The damped run should barely move off the input.
        let moved_full: f32 = full.iter().zip(&base).map(|(a, b)| (a - b).abs()).sum();
        let moved_damped: f32 = damped.iter().zip(&base).map(|(a, b)| (a - b).abs()).sum();
        assert!(moved_full > moved_damped * 100.0, "{moved_full} vs {moved_damped}");
    }
}

/// GF-4's rock-reading glacial and coastal passes (`GEOLOGY_FIRST_SCOPE.md`
/// §4.4, §4.5). Every expected ratio is a literal worked out by hand from
/// §2.3's table at `c = 0.5`, never the table read back.
#[cfg(test)]
mod gf4_rock_tests {
    use super::*;
    use cartalith_terrain::geology::{Construction, GeologyColumn, Rock, NO_LAYER};

    /// A column of one rock everywhere, single-layer, bare.
    fn uniform(rock: Rock, n: usize) -> GeologyColumn {
        GeologyColumn {
            rock_top: vec![rock as u8; n],
            rock_sub: vec![NO_LAYER; n],
            contact: vec![f32::NAN; n],
            regolith: vec![0.0; n],
            volcanic_setting: vec![0; n],
            construction: Construction::Off,
        }
    }

    /// A cold 8 x 12 ramp, every land cell above a snowline of 0 and below
    /// freezing, so every cell with a receiver erodes.
    fn glacial_fixture() -> (Vec<f32>, Vec<f32>, usize, usize, GlacialParams) {
        let (w, h) = (8usize, 12usize);
        let fld: Vec<f32> = (0..w * h).map(|i| 0.3 + 0.04 * (i / w) as f32 + 0.003 * (i % w) as f32).collect();
        let temp = vec![-5.0f32; w * h];
        let p = GlacialParams { kg: 5.0, mg: 0.4, snowline: 0.0, u_factor: 0.6, passes: 1, g: 1.0, sea: 0.1, world: false };
        (fld, temp, w, h, p)
    }

    /// Per-cell lowering of one glacial run with `rock`, or the legacy run.
    fn glacial_lowering(rock: Option<&PassRock>) -> Vec<f64> {
        let (mut fld, temp, w, h, p) = glacial_fixture();
        let before = fld.clone();
        match rock {
            Some(r) => glacial_kernel_rock(&mut fld, &temp, w, h, &p, r),
            None => glacial_kernel(&mut fld, &temp, w, h, &p),
        }
        before.iter().zip(&fld).map(|(a, b)| *a as f64 - *b as f64).collect()
    }

    /// Ratios of `num` to `den` at every cell `den` lowers by more than 1e-4
    /// (asserted non-empty, so the comparison cannot pass vacuously).
    fn ratios(num: &[f64], den: &[f64]) -> Vec<f64> {
        let r: Vec<f64> = num.iter().zip(den).filter(|(_, d)| **d > 1e-4).map(|(n, d)| n / d).collect();
        assert!(r.len() >= 20, "the fixture must lower at least 20 cells, lowered {}", r.len());
        r
    }

    /// Protects: GF-4's control arm. At `c = 0` the rock kernel is the legacy
    /// glacial kernel bit for bit (`γ^0 = 1`), whatever the column holds, so
    /// a `c = 0` control is the reference pass.
    #[test]
    fn glacial_rock_at_zero_contrast_is_the_legacy_kernel_bit_for_bit() {
        let (_, _, w, h, _) = glacial_fixture();
        let col = uniform(Rock::Shale, w * h);
        let rock = PassRock { column: &col, contrast: 0.0, r_expose: 0.001 };
        let a = glacial_lowering(Some(&rock));
        let b = glacial_lowering(None);
        assert!(a.iter().zip(&b).all(|(x, y)| x.to_bits() == y.to_bits()));
        assert!(b.iter().any(|&v| v > 1e-4), "the fixture must erode");
    }

    /// Protects: §4.4's `γ^c` reaches every term of the pass. Shale (γ 1.8)
    /// against granite (γ 0.5) at `c = 0.5` lowers every cell by
    /// `sqrt(1.8 / 0.5)` = sqrt(3.6) = 1.8974 times as much (arithmetic), and
    /// granite against the legacy pass by sqrt(0.5) = 0.7071. Dropping the
    /// factor reads 1, ignoring `c` reads 3.6, and reading `ρ` instead reads
    /// sqrt(1.5 / 0.1) = 3.873.
    #[test]
    fn glacial_lowering_scales_by_gamma_to_the_contrast() {
        let (_, _, w, h, _) = glacial_fixture();
        let (shale, granite) = (uniform(Rock::Shale, w * h), uniform(Rock::Granite, w * h));
        let ls = glacial_lowering(Some(&PassRock { column: &shale, contrast: 0.5, r_expose: 0.001 }));
        let lg = glacial_lowering(Some(&PassRock { column: &granite, contrast: 0.5, r_expose: 0.001 }));
        let legacy = glacial_lowering(None);
        for r in ratios(&ls, &lg) {
            assert!((r - 1.897_366_6).abs() < 2e-3, "shale/granite lowering ratio {r}, expected 1.8974");
        }
        for r in ratios(&lg, &legacy) {
            assert!((r - 0.707_106_8).abs() < 2e-3, "granite/legacy lowering ratio {r}, expected 0.7071");
        }
    }

    /// Protects: the pass reads the **exposed** rock (§2.5), not `rock_top`.
    /// A granite cap over shale whose contact is above the surface exposes
    /// shale, so it erodes exactly as all-shale; regolith thicker than
    /// `R_EXPOSE` exposes unconsolidated material.
    #[test]
    fn glacial_reads_the_exposed_rock_through_the_column() {
        let (fld, _, w, h, _) = glacial_fixture();
        let n = w * h;
        let mut capped = uniform(Rock::Granite, n);
        capped.rock_sub = vec![Rock::Shale as u8; n];
        capped.contact = fld.iter().map(|z| z + 0.5).collect();
        let a = glacial_lowering(Some(&PassRock { column: &capped, contrast: 0.5, r_expose: 0.001 }));
        let s = glacial_lowering(Some(&PassRock { column: &uniform(Rock::Shale, n), contrast: 0.5, r_expose: 0.001 }));
        assert!(a.iter().zip(&s).all(|(x, y)| x.to_bits() == y.to_bits()), "cap breached: substrate (shale) must erode");
        let mut mantled = uniform(Rock::Granite, n);
        mantled.regolith = vec![0.01; n];
        let m = glacial_lowering(Some(&PassRock { column: &mantled, contrast: 0.5, r_expose: 0.001 }));
        let u = glacial_lowering(Some(&PassRock { column: &uniform(Rock::Unconsolidated, n), contrast: 0.5, r_expose: 0.001 }));
        assert!(m.iter().zip(&u).all(|(x, y)| x.to_bits() == y.to_bits()), "regolith over R_EXPOSE must read as unconsolidated");
    }

    /// A 10 x 6 coast: the left four columns are sea at 0.05, the rest land
    /// at 0.15 + 0.02 per column (sea 0.1), with discharge 10 on the first
    /// land column's fourth row so the estuary cut fires there. `marsh_band`
    /// is 0 unless a test sets it.
    fn coastal_fixture() -> (Vec<f32>, Vec<f32>, usize, usize, CoastalParams) {
        let (w, h) = (10usize, 6usize);
        let fld: Vec<f32> = (0..w * h).map(|i| if i % w < 4 { 0.05 } else { 0.15 + 0.02 * (i % w - 4) as f32 }).collect();
        let mut flow = vec![0.0f32; w * h];
        flow[3 * w + 4] = 10.0;
        let p = CoastalParams { wave_str: 10.0, estuary_depth: 0.08, marsh_band: 0.0, passes: 1 };
        (fld, flow, w, h, p)
    }

    /// Per-cell field change (after minus before, signed) of one coastal run.
    fn coastal_change(rock: Option<&PassRock>, marsh: f64) -> Vec<f64> {
        let (mut fld, flow, w, h, mut p) = coastal_fixture();
        p.marsh_band = marsh;
        let before = fld.clone();
        match rock {
            Some(r) => coastal_process_rock(&mut fld, &flow, w, h, 0.1, false, 1.0, &p, r),
            None => coastal_process(&mut fld, &flow, w, h, 0.1, false, 1.0, &p),
        }
        fld.iter().zip(&before).map(|(a, b)| *a as f64 - *b as f64).collect()
    }

    /// Protects: the coastal control arm. At `c = 0` the rock pass is the
    /// legacy pass bit for bit, the marsh included.
    #[test]
    fn coastal_rock_at_zero_contrast_is_the_legacy_pass_bit_for_bit() {
        let (_, _, w, h, _) = coastal_fixture();
        let col = uniform(Rock::Tuff, w * h);
        let a = coastal_change(Some(&PassRock { column: &col, contrast: 0.0, r_expose: 0.001 }), 0.1);
        let b = coastal_change(None, 0.1);
        assert!(a.iter().zip(&b).all(|(x, y)| x.to_bits() == y.to_bits()));
        assert!(b.iter().any(|&v| v.abs() > 1e-4), "the fixture must move");
    }

    /// Protects: §4.5's `ρ^c` on the wave cut, the debris and the estuary
    /// cut. Tuff (ρ 3.0) against granite (ρ 0.10) at `c = 0.5` moves every
    /// cell by sqrt(30) = 5.4772 times as much (arithmetic): the cut cells
    /// lose it and their land neighbours gain the debris, both scaled by the
    /// cutting cell's rock. Dropping the factor on any one term breaks the
    /// ratio on the cells that term reaches; reading `γ` instead reads
    /// sqrt(1.5 / 0.5) = 1.732.
    #[test]
    fn coastal_change_scales_by_rho_to_the_contrast() {
        let (_, _, w, h, _) = coastal_fixture();
        let (tuff, granite) = (uniform(Rock::Tuff, w * h), uniform(Rock::Granite, w * h));
        let t = coastal_change(Some(&PassRock { column: &tuff, contrast: 0.5, r_expose: 0.001 }), 0.0);
        let g = coastal_change(Some(&PassRock { column: &granite, contrast: 0.5, r_expose: 0.001 }), 0.0);
        let legacy = coastal_change(None, 0.0);
        let (mut cut, mut gained) = (0usize, 0usize);
        for i in 0..w * h {
            if legacy[i].abs() <= 1e-5 {
                assert_eq!(t[i], 0.0, "cell {i}: rock must not move a cell the legacy pass leaves");
                continue;
            }
            if legacy[i] < 0.0 {
                cut += 1;
            } else {
                gained += 1;
            }
            let r = t[i] / g[i];
            assert!((r - 5.477_225_6).abs() < 1e-2, "cell {i}: tuff/granite change ratio {r}, expected 5.4772");
            let r = g[i] / legacy[i];
            assert!((r - 0.316_227_8).abs() < 2e-3, "cell {i}: granite/legacy change ratio {r}, expected 0.3162");
        }
        assert!(cut >= 6 && gained >= 6, "the fixture must reach both the cut and the debris: {cut} cut, {gained} gained");
        // The river mouth carries the wave cut and the estuary cut together,
        // so it loses more than the plain coast cell above it.
        assert!(legacy[3 * w + 4] < legacy[2 * w + 4], "the estuary cut must fire on the fixture's river mouth");
    }

    /// Protects: the coastal pass reads the exposed rock, as glacial does: a
    /// granite cap breached to its tuff substrate retreats as tuff.
    #[test]
    fn coastal_reads_the_exposed_rock_through_the_column() {
        let (fld, _, w, h, _) = coastal_fixture();
        let n = w * h;
        let mut capped = uniform(Rock::Granite, n);
        capped.rock_sub = vec![Rock::Tuff as u8; n];
        capped.contact = fld.iter().map(|z| z + 0.5).collect();
        let a = coastal_change(Some(&PassRock { column: &capped, contrast: 0.5, r_expose: 0.001 }), 0.0);
        let t = coastal_change(Some(&PassRock { column: &uniform(Rock::Tuff, n), contrast: 0.5, r_expose: 0.001 }), 0.0);
        assert!(a.iter().zip(&t).all(|(x, y)| x.to_bits() == y.to_bits()));
    }

    /// Protects: the marsh accretes only (Ruling CH addendum). With the wave
    /// and estuary cut off (`wave_str` 0), a pass over a flat lowland in the
    /// marsh band raises some cells and lowers none, in both kernels. With
    /// the reference's centred bump about half of them would sink.
    #[test]
    fn the_marsh_never_lowers_a_cell() {
        let (w, h) = (24usize, 16usize);
        let before: Vec<f32> = (0..w * h).map(|i| if i % w < 3 { 0.05 } else { 0.11 }).collect();
        let flow = vec![0.0f32; w * h];
        let p = CoastalParams { wave_str: 0.0, estuary_depth: 0.08, marsh_band: 0.03, passes: 1 };
        let col = uniform(Rock::Shale, w * h);
        for rock in [None, Some(PassRock { column: &col, contrast: 0.5, r_expose: 0.001 })] {
            let mut f = before.clone();
            match &rock {
                Some(r) => coastal_process_rock(&mut f, &flow, w, h, 0.1, false, 1.0, &p, r),
                None => coastal_process(&mut f, &flow, w, h, 0.1, false, 1.0, &p),
            }
            assert!(f.iter().zip(&before).all(|(a, b)| a >= b), "a marsh cell was lowered");
            let raised = f.iter().zip(&before).filter(|(a, b)| a > b).count();
            assert!(raised >= 20, "the marsh must accrete somewhere: {raised} cells");
        }
    }

    /// Protects: the estuary cut's cap (Ruling CH addendum 2). A river cell
    /// 0.0001 above its flat land neighbours, with a cut the reference would
    /// make 0.015 deep (`0.003·1·(1 − 0.1/0.2)·10`, arithmetic), may only
    /// come down to them, never below, in both kernels; and a cell lower
    /// than all eight is not cut at all.
    #[test]
    fn the_estuary_cut_never_digs_below_the_lowest_neighbour() {
        let (w, h) = (5usize, 5usize);
        let p = CoastalParams { wave_str: 10.0, estuary_depth: 0.2, marsh_band: 0.0, passes: 1 };
        let col = uniform(Rock::Shale, w * h);
        for (centre, expect) in [(0.2001f32, 0.2f32), (0.199f32, 0.199f32)] {
            let mut before = vec![0.2f32; w * h];
            before[12] = centre;
            let mut flow = vec![0.0f32; w * h];
            flow[12] = 1.0e6;
            for rock in [None, Some(PassRock { column: &col, contrast: 0.0, r_expose: 0.001 })] {
                let mut f = before.clone();
                match &rock {
                    Some(r) => coastal_process_rock(&mut f, &flow, w, h, 0.1, false, 1.0, &p, r),
                    None => coastal_process(&mut f, &flow, w, h, 0.1, false, 1.0, &p),
                }
                assert!((f[12] - expect).abs() < 1e-6, "centre {centre}: got {}, expected {expect}", f[12]);
            }
        }
    }

    /// Protects: the multiplier helpers against §2.3's table as literals
    /// (shale γ 1.8, granite ρ 0.10) and their `c = 0` identity.
    #[test]
    fn multipliers_are_the_table_value_to_the_contrast() {
        assert!((gamma_multiplier(Rock::Shale, 1.0) - 1.8).abs() < 1e-6);
        assert!((rho_multiplier(Rock::Granite, 1.0) - 0.1).abs() < 1e-6);
        assert!((gamma_multiplier(Rock::Shale, 0.5) - 1.341_640_8).abs() < 1e-6);
        for r in Rock::ALL {
            assert_eq!(gamma_multiplier(r, 0.0), 1.0);
            assert_eq!(rho_multiplier(r, 0.0), 1.0);
        }
    }
}
