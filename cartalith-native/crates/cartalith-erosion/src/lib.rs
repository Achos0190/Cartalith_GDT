//! droplet, stream-power, thermal erosion
//!
//! Ported in pipeline order starting Phase 1 (MVP_SCOPE.md).
//!
//! [`passes`] holds the reference's **manual** passes — velocity, glacial,
//! coastal, hillslope, sediment routing, tidal flats — which `generate()`
//! never runs in the reference either. See that module's own header.

use cartalith_rng::Mulberry32;
use rayon::prelude::*;

pub mod passes;

pub use passes::{
    apply_tidal_sedimentation, centrifugal_shear, coastal_process, glacial_kernel,
    hillslope_diffuse, route_sediment, velocity_erode_kernel, CoastalParams, GlacialParams,
    VelocityField, VelocityParams,
};

struct HGrad {
    height: f64,
    grad_x: f64,
    grad_y: f64,
}

/// `hgrad()` (reference HTML, inside `dropletKernel`): bilinear height and
/// gradient at a fractional grid position.
fn hgrad(fld: &[f32], w: usize, px: f64, py: f64) -> HGrad {
    let nx = px as usize;
    let ny = py as usize;
    let fx = px - nx as f64;
    let fy = py - ny as f64;
    let i = ny * w + nx;
    let h00 = fld[i] as f64;
    let h10 = fld[i + 1] as f64;
    let h01 = fld[i + w] as f64;
    let h11 = fld[i + w + 1] as f64;
    let grad_x = (h10 - h00) * (1.0 - fy) + (h11 - h01) * fy;
    let grad_y = (h01 - h00) * (1.0 - fx) + (h11 - h10) * fx;
    let height = h00 * (1.0 - fx) * (1.0 - fy) + h10 * fx * (1.0 - fy) + h01 * (1.0 - fx) * fy + h11 * fx * fy;
    HGrad { height, grad_x, grad_y }
}

/// `deposit()`: bilinear-splat sediment into the (up to) four cells
/// surrounding a fractional position. No clamping — unlike `scrape`, JS's
/// `deposit` never bounds the result (erosion's own `field[i]<0/>1` clamp
/// happens once, later, in `erodeFinish`).
#[allow(clippy::too_many_arguments)]
fn deposit(fld: &mut [f32], w: usize, h: usize, nx: usize, ny: usize, fx: f64, fy: f64, amount: f64) {
    let i = ny * w + nx;
    fld[i] = (fld[i] as f64 + amount * (1.0 - fx) * (1.0 - fy)) as f32;
    if nx + 1 < w {
        fld[i + 1] = (fld[i + 1] as f64 + amount * fx * (1.0 - fy)) as f32;
    }
    if ny + 1 < h {
        fld[i + w] = (fld[i + w] as f64 + amount * (1.0 - fx) * fy) as f32;
    }
    if nx + 1 < w && ny + 1 < h {
        fld[i + w + 1] = (fld[i + w + 1] as f64 + amount * fx * fy) as f32;
    }
}

/// `scrape()`: subtracts a weighted amount across the circular brush
/// kernel, clamped at `0`. Same round-then-clamp order as every other
/// `Float32Array` clamp site in this port (`cartalith-terrain`'s
/// `stamp_one_volcano`/`stamp_one_crater`): round to `f32` first, then
/// clamp against *that* rounded value, not the pre-rounding `f64` delta.
#[allow(clippy::too_many_arguments)]
fn scrape(fld: &mut [f32], w: usize, h: usize, b_dx: &[i32], b_dy: &[i32], b_w: &[f64], cx: i64, cy: i64, amount: f64) {
    for k in 0..b_dx.len() {
        let x = cx + b_dx[k] as i64;
        let y = cy + b_dy[k] as i64;
        if x < 0 || y < 0 || x >= w as i64 || y >= h as i64 {
            continue;
        }
        let i = y as usize * w + x as usize;
        let stored = (fld[i] as f64 - amount * b_w[k]) as f32;
        fld[i] = if (stored as f64) < 0.0 { 0.0 } else { stored };
    }
}

/// The erosion tuning knobs `dropletParams()` bundles (reference HTML
/// line 3889) — `state.erosion`'s own fields, plus the derived `ck`
/// (climate-coupling strength) and `seed`.
pub struct DropletParams {
    pub droplets: i32,
    pub inertia: f64,
    pub capacity: f64,
    pub min_slope: f64,
    pub deposit: f64,
    pub erode: f64,
    pub evaporate: f64,
    pub gravity: f64,
    pub g: f64,
    pub max_lifetime: i32,
    pub init_speed: f64,
    pub init_water: f64,
    pub radius: i32,
    pub ck: f64,
    pub seed: u32,
}

/// `dropletKernel()` (reference HTML lines 3584-3616) — particle-based
/// hydraulic erosion. Self-contained by design in the original (no module
/// globals; ships to a Web Worker via `toString()`), which made it a
/// natural first erosion stage to port: the whole simulation is captured
/// by this one function's arguments.
///
/// Each droplet: spawns (rain-weighted rejection sampling when `ck>0`),
/// then for up to `max_lifetime` steps follows the inertia-blended
/// downhill gradient, erodes or deposits based on carrying capacity
/// versus current sediment load, and gains/loses speed via a simplified
/// energy-conservation term (`speed² += -dH·gravity·g`) scaled by planet
/// gravity. Mutates `fld` in place.
///
/// `rain` must be `Some` whenever `ck>0` — mirrors the JS caller's own
/// invariant (`erode()`: `ck>0?rainField:null`), not independently
/// re-validated here.
///
/// NOT parallelized (`CPU_MULTITHREADING_SCOPE.md`): confirmed, not just
/// flagged, genuine per-droplet sequential state — each droplet's own
/// `deposit`/`scrape` calls mutate `fld` in place, and the *next* droplet's
/// path (`hgrad` samples) depends on exactly what every previous droplet
/// already carved. This is the outer `for _ in 0..p.droplets` loop; there
/// is no independent per-cell shape to parallelize here at all.
pub fn droplet_kernel(fld: &mut [f32], rain: Option<&[f32]>, w: usize, h: usize, p: &DropletParams) {
    let mut rng = Mulberry32::new(p.seed ^ 0x9e3779b9);

    let r = p.radius;
    let mut b_dx: Vec<i32> = Vec::new();
    let mut b_dy: Vec<i32> = Vec::new();
    let mut b_w: Vec<f64> = Vec::new();
    {
        let mut sum = 0.0f64;
        let mut w_raw: Vec<f64> = Vec::new();
        for dy in -r..=r {
            for dx in -r..=r {
                let d2 = (dx * dx + dy * dy) as f64;
                if d2 <= (r * r) as f64 {
                    let wt = 1.0 - d2.sqrt() / r as f64;
                    b_dx.push(dx);
                    b_dy.push(dy);
                    w_raw.push(wt);
                    sum += wt;
                }
            }
        }
        for v in w_raw {
            b_w.push(v / sum);
        }
    }

    let ck = p.ck;

    for _ in 0..p.droplets {
        let (mut px, mut py);
        let mut tries = 0i32;
        loop {
            px = rng.next_f64() * (w as f64 - 1.0);
            py = rng.next_f64() * (h as f64 - 1.0);
            if ck > 0.0 {
                let rf = rain.expect("rain field required when ck > 0")[py as usize * w + px as usize] as f64;
                if rng.next_f64() > 0.15 + 0.85 * rf {
                    tries += 1;
                    if tries < 16 {
                        continue;
                    }
                }
            }
            break;
        }

        let mut dx = 0.0f64;
        let mut dy = 0.0f64;
        let mut speed = p.init_speed;
        let mut water = p.init_water;
        let mut sed = 0.0f64;

        for _ in 0..p.max_lifetime {
            let nx = px as usize;
            let ny = py as usize;
            let fx = px - nx as f64;
            let fy = py - ny as f64;
            let hg = hgrad(fld, w, px, py);

            dx = dx * p.inertia - hg.grad_x * (1.0 - p.inertia);
            dy = dy * p.inertia - hg.grad_y * (1.0 - p.inertia);
            let len = dx.hypot(dy);
            if len < 1e-6 {
                break;
            }
            dx /= len;
            dy /= len;
            px += dx;
            py += dy;
            if px < 0.0 || py < 0.0 || px >= w as f64 - 1.0 || py >= h as f64 - 1.0 {
                break;
            }

            let d_h = hgrad(fld, w, px, py).height - hg.height;
            let cap = (-d_h).max(p.min_slope) * speed * water * p.capacity;

            if sed > cap || d_h > 0.0 {
                let dep = if d_h > 0.0 { d_h.min(sed) } else { (sed - cap) * p.deposit };
                sed -= dep;
                deposit(fld, w, h, nx, ny, fx, fy, dep);
            } else {
                let rf = if ck > 0.0 {
                    rain.expect("rain field required when ck > 0")[ny * w + nx] as f64
                } else {
                    1.0
                };
                let ero = ((cap - sed) * p.erode * (1.0 + ck * rf)).min(-d_h);
                sed += ero;
                scrape(fld, w, h, &b_dx, &b_dy, &b_w, nx as i64, ny as i64, ero);
            }

            speed = (speed * speed + (-d_h) * p.gravity * p.g).max(0.0).sqrt();
            water *= 1.0 - p.evaporate;
            if water < 1e-3 {
                break;
            }
        }
    }
}

/// `erodeThermalCPU()` (reference HTML lines 3856-3865), CPU path only —
/// GPU is unavailable headless and JS falls back to this exact code when
/// it is. Talus-angle-driven diffusion: any cell steeper than `talus`
/// relative to a 4-connected neighbor sheds the excess, split
/// proportionally among however many neighbors are over-steep.
///
/// `delta` stays `f32` throughout, matching JS's fresh `Float32Array`
/// each pass — a downhill neighbor cell can receive `+=` contributions
/// from *several* different uphill cells within one pass, and JS rounds
/// each one individually, not once at the end. An `f64` accumulator for
/// `delta` would occasionally disagree with a genuinely multi-contributor
/// cell — the same trap `stamp_one_crater`'s three-site `field[i]+=` and
/// `compute_stress`'s `raw[i]+=` both needed `add_rounded`-style handling
/// for, just with the accumulation spread across *different cells* within
/// one pass here instead of multiple terms at *one* cell.
pub fn erode_thermal(fld: &mut [f32], w: usize, h: usize, passes: i32, talus: f64) {
    for _ in 0..passes {
        let mut delta = vec![0f32; w * h];
        // NOT parallelized: this loop writes `delta[i]` (its own cell) AND
        // scatters into `delta[j]` for up to 4 neighbours in the same
        // pass -- the same cross-cell scatter-write hazard already
        // identified for `compute_stress` (GPU_LAYER_INTEGRATION_SCOPE.md)
        // and `cartalith-civ`'s scatter-based functions: two cells' writes
        // could land on the same neighbour's `delta` simultaneously. A
        // gather reformulation (each cell reads its neighbours' excess
        // instead of writing to them) would fix this, but that's a real
        // algorithmic redesign, not a `par_iter` swap -- left sequential.
        for y in 0..h {
            for x in 0..w {
                let i = y * w + x;
                let hh = fld[i] as f64;
                let mut excess = 0.0f64;
                let mut nb: Vec<(usize, f64)> = Vec::new();
                let ne: [(i64, i64); 4] = [
                    (x as i64 - 1, y as i64),
                    (x as i64 + 1, y as i64),
                    (x as i64, y as i64 - 1),
                    (x as i64, y as i64 + 1),
                ];
                for &(nx, ny) in &ne {
                    if nx < 0 || ny < 0 || nx >= w as i64 || ny >= h as i64 {
                        continue;
                    }
                    let (nx, ny) = (nx as usize, ny as usize);
                    let dh = hh - fld[ny * w + nx] as f64;
                    if dh > talus {
                        nb.push((ny * w + nx, dh - talus));
                        excess += dh - talus;
                    }
                }
                if excess > 0.0 {
                    let move_amt = hh.min(excess * 0.5 * 0.25);
                    delta[i] = (delta[i] as f64 - move_amt) as f32;
                    for &(j, e) in &nb {
                        delta[j] = (delta[j] as f64 + move_amt * (e / excess)) as f32;
                    }
                }
            }
        }
        // Independent per cell (reads only `fld[i]`/`delta[i]`), safe.
        fld.par_iter_mut().zip(delta.par_iter()).for_each(|(f, d)| {
            let stored = (*f as f64 + *d as f64) as f32;
            *f = if (stored as f64) < 0.0 {
                0.0
            } else if (stored as f64) > 1.0 {
                1.0
            } else {
                stored
            };
        });
    }
}

pub(crate) fn d8_table() -> [f64; 9] {
    let mut d8 = [0f64; 9];
    for dy in -1i32..=1 {
        for dx in -1i32..=1 {
            d8[((dy + 1) * 3 + (dx + 1)) as usize] = (dx as f64).hypot(dy as f64);
        }
    }
    d8
}

/// Array-based binary min-heap, ported field-for-field from the JS
/// `MinHeap` inside `streamPowerKernel` (reference HTML lines
/// 4098-4107) — **not** substituted for `std::collections::BinaryHeap`.
/// `PROVENANCE.md` names priority-flood fill specifically: "equal-
/// priority pop order decides the fill tie-break and therefore lake
/// shape" — this exact sift-up/sift-down comparison and swap order is
/// the thing being ported, not just "a min-heap."
pub(crate) struct MinHeap {
    p: Vec<f32>,
    v: Vec<i32>,
    len: usize,
}

impl MinHeap {
    pub(crate) fn new(cap: usize) -> Self {
        Self { p: vec![0.0; cap], v: vec![0; cap], len: 0 }
    }

    pub(crate) fn size(&self) -> usize {
        self.len
    }

    pub(crate) fn push(&mut self, prio: f32, val: i32) {
        let mut i = self.len;
        self.len += 1;
        self.p[i] = prio;
        self.v[i] = val;
        while i > 0 {
            let par = (i - 1) / 2;
            if self.p[par] <= self.p[i] {
                break;
            }
            self.p.swap(par, i);
            self.v.swap(par, i);
            i = par;
        }
    }

    pub(crate) fn pop(&mut self) -> i32 {
        let rv = self.v[0];
        self.len -= 1;
        let last = self.len;
        if last > 0 {
            self.p[0] = self.p[last];
            self.v[0] = self.v[last];
            let mut i = 0usize;
            loop {
                let l = 2 * i + 1;
                let r = 2 * i + 2;
                let mut s = i;
                if l < last && self.p[l] < self.p[s] {
                    s = l;
                }
                if r < last && self.p[r] < self.p[s] {
                    s = r;
                }
                if s == i {
                    break;
                }
                self.p.swap(s, i);
                self.v.swap(s, i);
                i = s;
            }
        }
        rv
    }
}

/// The stream-power tuning knobs `streamParams()` bundles (reference
/// HTML line 4261) — `state.stream`'s own fields plus the derived
/// `resist`/`g`/`world`/`sea` context values.
pub struct StreamPowerParams {
    pub k: f64,
    pub uplift: f64,
    /// JS: `sp.deposit||0` — pass `0.0` for that fallback explicitly;
    /// not defaulted here.
    pub deposit: f64,
    /// JS: `sp.climateK||0`.
    pub climate_k: f64,
    pub iters: i32,
    pub resist: f64,
    pub g: f64,
    pub world: bool,
    pub sea: f64,
}

/// `streamPowerKernel()` (reference HTML lines 4082-4194): implicit
/// stream-power incision (Braun & Willett 2013) on a priority-flood-
/// filled surface, with multiple-flow-direction drainage area (Freeman
/// 1991) and an optional sediment-deposition pass.
///
/// Three real precision/ordering subtleties preserved deliberately:
/// - `Cc` (the per-cell implicit-incision coefficient) is a
///   `Float64Array` in JS, computed once and reused across all
///   `P.iters` passes — genuinely full `f64` precision throughout, not
///   rounded through `f32` at any point. Kept as `Vec<f64>` here.
/// - `area[j]+=...` (drainage-area spreading) and `sed[r]+=sed[i]`
///   (deposition's downstream sediment carry) are both the same
///   multi-writer-per-pass trap `erode_thermal`'s `delta[j]+=` and
///   `compute_flow`'s `acc[best]+=` already established: a single
///   target cell can receive contributions from several different
///   source cells within one pass, each JS write rounding to `f32`
///   individually.
/// - The deposition block's two conditional adjustments to `fld[i]`/
///   `sed[i]` are sequential, not simultaneous — the second condition
///   reads back the *already-updated* (rounded) values the first one
///   just wrote, exactly mirroring JS's statement order.
pub fn stream_power_kernel(
    fld: &mut [f32],
    stress: &[f32],
    resist: &[f32],
    rain: &[f32],
    w: usize,
    h: usize,
    p: &StreamPowerParams,
) {
    let n = w * h;
    let wrap = p.world;
    let sea = p.sea;
    let d8 = d8_table();

    let mut order = vec![0i32; n];
    let mut rdist = vec![0f32; n];
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

    let mut cnt = 0usize;
    const EPS: f64 = 1e-5;
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
                rdist[j] = d8[((dy + 1) * 3 + (dx + 1)) as usize] as f32;
                heap.push(filled[j], j as i32);
            }
        }
    }
    let dist = rdist; // renamed for clarity below (matches JS's `dist`, reused as `rdist` after receiver computation overwrites it)

    // NOT parallelized: `ss` is a running SUM (not a max), and unlike
    // max/min, floating-point summation is order-dependent -- a parallel
    // reduction could round differently and, in a rare edge case, flip
    // the `ss < 1e-3` branch below. Cheap (O(n), trivial per-cell work),
    // not worth risking bit-exactness for.
    let mut u = vec![0f32; n];
    let mut ss = 0.0f64;
    for i in 0..n {
        let s = stress[i];
        if s > 0.0 {
            u[i] = s;
        }
        ss += (s as f64).abs();
    }
    if ss < 1e-3 {
        // Per-cell, independent -- safe.
        u.par_iter_mut().enumerate().for_each(|(i, uv)| {
            *uv = ((fld[i] as f64 - 0.3).max(0.0)) as f32;
        });
    }
    // Max is associative/commutative for real values -- a parallel
    // reduction gives the exact same result as the sequential running-max
    // (same reasoning `cartalith-climate::build_wind`'s own `mx` uses).
    let u_max = u.par_iter().map(|&uv| uv as f64).reduce(|| 1e-6f64, f64::max);
    u.par_iter_mut().for_each(|uv| {
        *uv = ((*uv as f64 / u_max) * p.uplift) as f32;
    });

    let m = 0.5f64;
    let k_coef = p.k * p.g;
    let dt = 1.0f64;
    let dep = p.deposit;
    let ck = p.climate_k;

    let mut rcv = vec![-1i32; n];
    let mut rdist = dist; // overwrite with receiver distances, matching JS reusing `rdist`
    // Per-cell, fixed 3x3 read of the frozen `filled` -- independent
    // across cells, safe to parallelize.
    rcv.par_iter_mut().zip(rdist.par_iter_mut()).enumerate().for_each(|(i, (rcv_i, rdist_i))| {
        let x = (i % w) as i64;
        let y = (i / w) as i64;
        let hh = filled[i] as f64;
        let mut best = -1i32;
        let mut best_s = 0.0f64;
        let mut brd = 1.0f32;
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
                let d = d8[((dy + 1) * 3 + (dx + 1)) as usize];
                let sl = (hh - filled[j] as f64) / d;
                if sl > best_s {
                    best_s = sl;
                    best = j as i32;
                    brd = d as f32;
                }
            }
        }
        *rcv_i = best;
        *rdist_i = brd;
    });

    // NOT parallelized: descending-order flow accumulation -- each cell
    // scatters its own accumulated `area` into its downstream donors
    // (`area[j] += ...`), the same genuine cross-cell dependency already
    // confirmed unsafe for `cartalith-hydrology::compute_flow` and flagged
    // in `CPU_MULTITHREADING_SCOPE.md`/`GPU_LAYER_INTEGRATION_SCOPE.md`.
    let mut area = vec![1f32; n];
    for k in (0..n).rev() {
        let i = order[k] as usize;
        let x = (i % w) as i64;
        let y = (i / w) as i64;
        let hh = filled[i] as f64;
        let a = area[i] as f64;
        let mut sw = 0.0f64;
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
                let sl = (hh - filled[j] as f64) / d8[((dy + 1) * 3 + (dx + 1)) as usize];
                if sl > 0.0 {
                    sw += sl.powf(1.1);
                }
            }
        }
        if sw > 0.0 {
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
                    let sl = (hh - filled[j] as f64) / d8[((dy + 1) * 3 + (dx + 1)) as usize];
                    if sl > 0.0 {
                        area[j] = (area[j] as f64 + a * sl.powf(1.1) / sw) as f32;
                    }
                }
            }
        }
    }

    // `order[k]` only ever supplies an index `i` here -- the computation
    // itself doesn't depend on `k`/visitation order, and `order` visits
    // every `0..n` index exactly once, so iterating `i` directly (instead
    // of through `order[k]`) is the identical computation, just without
    // the indirection -- and lets this run as one independent per-cell
    // pass (reads only `rcv[i]`/`rdist[i]`/`resist[i]`/`rain[i]`/`area[i]`,
    // all already frozen).
    let mut cc = vec![0f64; n];
    cc.par_iter_mut().enumerate().for_each(|(i, cc_i)| {
        let r = rcv[i];
        if r < 0 {
            return;
        }
        let l = rdist[i] as f64;
        let res = p.resist * 0.7 * resist[i] as f64;
        let ki = k_coef * (1.0 - res).max(0.05) * (1.0 + ck * 2.0 * rain[i] as f64);
        *cc_i = ki * dt * (area[i] as f64).powf(m) / l;
    });

    // NOT parallelized, this whole loop: a genuine donor-receiver
    // wavefront dependency, not just across `p.iters` iterations but
    // WITHIN a single iteration too -- `fld[i]`'s update reads `fld[r]`,
    // which the receivers-before-donors comment below confirms was
    // *already updated earlier in this same pass*. The deposition
    // sub-loop below has the identical shape in reverse (scatters into
    // `sed[r]`). Same category as `area`'s flow accumulation above.
    for _ in 0..p.iters {
        let old_h: Option<Vec<f32>> = if dep > 0.0 { Some(fld.to_vec()) } else { None };
        // receivers-before-donors: `order` runs low-to-high fill order,
        // so a cell's receiver (always lower) is updated before it is.
        #[allow(clippy::needless_range_loop)]
        for k in 0..n {
            let i = order[k] as usize;
            let r = rcv[i];
            if r < 0 {
                continue;
            }
            let r = r as usize;
            let c = cc[i];
            let val = (fld[i] as f64 + dt * u[i] as f64 + c * fld[r] as f64) / (1.0 + c);
            fld[i] = val as f32;
        }
        if dep > 0.0 {
            let old_h = old_h.expect("old_h is Some whenever dep > 0.0");
            let mut sed = vec![0f32; n];
            for i in 0..n {
                sed[i] = ((old_h[i] as f64 + dt * u[i] as f64 - fld[i] as f64).max(0.0)) as f32;
            }
            for k in (0..n).rev() {
                let i = order[k] as usize;
                let r = rcv[i];
                if r < 0 {
                    continue;
                }
                let r = r as usize;
                let rd = if rdist[i] != 0.0 { rdist[i] as f64 } else { 1.0 };
                let slope = ((fld[i] as f64 - fld[r] as f64) / rd).max(1e-6);
                let cap = 0.005 * (area[i] as f64).powf(0.5) * slope;
                let ceil = old_h[i] as f64 + dt * u[i] as f64;

                if sed[i] as f64 > cap {
                    let mut d = (sed[i] as f64 - cap) * dep;
                    if fld[i] as f64 + d > ceil {
                        d = (ceil - fld[i] as f64).max(0.0);
                    }
                    fld[i] = (fld[i] as f64 + d) as f32;
                    sed[i] = (sed[i] as f64 - d) as f32;
                }
                if fld[i] as f64 <= sea && sed[i] as f64 > 0.0 {
                    let mut d = sed[i] as f64 * dep * 0.8;
                    if fld[i] as f64 + d > ceil {
                        d = (ceil - fld[i] as f64).max(0.0);
                    }
                    fld[i] = (fld[i] as f64 + d) as f32;
                    sed[i] = (sed[i] as f64 - d) as f32;
                }
                sed[r] = (sed[r] as f64 + sed[i] as f64) as f32;
            }
        }
    }

    fld.par_iter_mut().for_each(|v| *v = v.clamp(0.0, 1.0));
}

/// `isostaticRebound()` (reference HTML lines 4426-4432): erosional
/// unloading returns as broad flexural uplift (England & Molnar 1990) —
/// one-sided, only net *removal* (`pre[i] - field[i] > 0`) rebounds;
/// tectonic uplift inside the same erosion pass never does. `pre` is the
/// field snapshot from *before* the erosion pass that just ran (JS:
/// `field.slice()`, taken by the caller); `field` is mutated in place.
/// No-op (JS: early `return`) when nothing eroded net-downward anywhere.
pub fn isostatic_rebound(field: &mut [f32], pre: &[f32], gw: usize, gh: usize, blur_r: f64, world: bool) {
    let n = gw * gh;
    // Per-cell, independent -- parallel fill. `any` is a boolean OR, safe
    // to reduce in parallel (order-independent, unlike a sum).
    let mut d = vec![0f32; n];
    d.par_iter_mut().enumerate().for_each(|(i, di)| {
        let e = pre[i] - field[i];
        if e > 0.0 {
            *di = e;
        }
    });
    let any = d.par_iter().any(|&v| v > 0.0);
    if !any {
        return;
    }
    // only long-wavelength unloading rebounds
    let b = cartalith_terrain::gauss_blur(&d, blur_r.max(8.0), gw, gh, world);
    field.par_iter_mut().zip(b.par_iter()).for_each(|(f, bi)| {
        let v = *f as f64 + 0.8 * *bi as f64;
        *f = v.clamp(0.0, 1.0) as f32;
    });
}

/// `recomputeResistanceAfterErosion()` (reference HTML line 3144, the
/// comment right before it at lines 3140-3143): "exhumation exposes
/// harder basement — where erosion has carved deeply (pre−post large),
/// resistance climbs toward a basement maximum so the NEXT pass bites
/// less there (differential erosion → benches / inselbergs / hard
/// sills)". Mutates `resist` in place. JS gates the call on
/// `state.tect.dynamicLithology` (default `false`); `cartalith-engine`
/// mirrors that gate on `p.tect.dynamic_lithology`, so this only runs
/// when a caller opts in.
pub fn recompute_resistance_after_erosion(resist: &mut [f32], pre: &[f32], post: &[f32], k: f64) {
    resist.par_iter_mut().enumerate().for_each(|(i, r)| {
        let ex = pre[i] as f64 - post[i] as f64;
        if ex > 0.0 {
            *r = ((*r as f64 + k * ex).min(1.0)) as f32;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::recompute_resistance_after_erosion;

    #[test]
    fn crate_compiles_and_tests_run() {
        assert_eq!(2 + 2, 4);
    }

    // Hand-derived against the JS formula (reference HTML line 3146):
    // `const ex=pre[i]-post[i]; if(ex>0) resist[i]=Math.min(1, resist[i]+k*ex);`
    #[test]
    fn recompute_resistance_matches_js_formula() {
        let mut resist = vec![0.5f32, 0.5, 0.5, 0.9];
        let pre = vec![1.0f32, 1.0, 1.0, 1.0];
        let post = vec![0.8f32, 1.0, 1.1, 0.0];
        recompute_resistance_after_erosion(&mut resist, &pre, &post, 2.0);

        // ex = 0.2 > 0 -> 0.5 + 2.0*0.2 = 0.9
        assert!((resist[0] - 0.9).abs() < 1e-6);
        // ex = 0.0, not > 0 -> untouched
        assert!((resist[1] - 0.5).abs() < 1e-6);
        // ex = -0.1 (deposition, not exhumation) -> untouched
        assert!((resist[2] - 0.5).abs() < 1e-6);
        // ex = 1.0, 0.9 + 2.0*1.0 = 2.9 -> clamped to the basement max of 1.0
        assert!((resist[3] - 1.0).abs() < 1e-6);
    }
}
