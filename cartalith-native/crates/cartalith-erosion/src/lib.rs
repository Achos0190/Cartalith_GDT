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
pub mod tile;

pub use passes::{
    apply_tidal_sedimentation, centrifugal_shear, coastal_process, glacial_kernel,
    hillslope_diffuse, hillslope_extent_scale, route_sediment, velocity_erode_kernel,
    CoastalParams, GlacialParams, VelocityField, VelocityParams, HILLSLOPE_REF_CELL_KM,
    HILLSLOPE_STABLE_D,
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

/// `erodeThermalCPU()` (reference HTML lines 3856-3865), the CPU/golden-
/// parity path — JS's own reference falls back to exactly this code when
/// its GPU path is unavailable headless. This port also has a GPU kernel
/// (`cartalith_gpu::thermal_grid_gpu_with`, dispatched from
/// `cartalith-engine`'s `erode_op.rs` for the Erode tool); this function is
/// that path's own CPU fallback and golden-verified reference. Talus-angle-
/// driven diffusion: any cell steeper than `talus`
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
    thermal_core(fld, w, h, passes, talus, None, false);
}

/// The body shared by [`erode_thermal`] (the golden-verified reference port:
/// scalar `talus`, `per_cell = None`, `wrap = false`) and
/// [`threshold_hillslope`] (GF-3: one pass at a time with a per-cell
/// threshold, wrapping in x on world maps).
///
/// Why one body: `GEOLOGY_FIRST_SCOPE.md` §4.3 defines the threshold
/// hillslope as "`erode_thermal`'s rule with a per-cell threshold", and says
/// the thermal golden stays untouched "because the per-cell form is an
/// `Option`". With `per_cell = None` and `wrap = false` every statement below
/// is the legacy port's, so `golden_parity_thermal.rs` holds by control flow,
/// not by an arithmetic coincidence.
///
/// Must never change the move rule (`0.5·0.25` of the summed excess, split by
/// each neighbour's share, clamped to 0..1) for either caller: that rule is the
/// reference's, and §4.3 changes only the threshold.
fn thermal_core(fld: &mut [f32], w: usize, h: usize, passes: i32, talus: f64, per_cell: Option<&[f64]>, wrap: bool) {
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
        // That gather exists on the GPU side: `cartalith-gpu`'s
        // `gpu_thermal.wgsl`, which `erode_op` runs instead under `use_gpu`.
        for y in 0..h {
            for x in 0..w {
                let i = y * w + x;
                let hh = fld[i] as f64;
                // The threshold belongs to the cell that sheds (§4.3's
                // `talus_i`): it is the upper cell's rock that fails.
                let talus = match per_cell {
                    Some(t) => t[i],
                    None => talus,
                };
                let mut excess = 0.0f64;
                let mut nb: Vec<(usize, f64)> = Vec::new();
                let ne: [(i64, i64); 4] = [
                    (x as i64 - 1, y as i64),
                    (x as i64 + 1, y as i64),
                    (x as i64, y as i64 - 1),
                    (x as i64, y as i64 + 1),
                ];
                for &(nx, ny) in &ne {
                    // A world map wraps in longitude (x) only; the legacy
                    // reference kernel (`wrap = false`) never wraps.
                    let nx = if wrap && nx < 0 {
                        nx + w as i64
                    } else if wrap && nx >= w as i64 {
                        nx - w as i64
                    } else {
                        nx
                    };
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

/// GF-3's pass count `N_h` for the threshold hillslope stage
/// (`GEOLOGY_FIRST_SCOPE.md` §4.3: "the number of passes, `N_h`, is a
/// **judgement**. GF-3 measures it against the cap-edge bar (B4) and the cost
/// bar (B9)").
///
/// Provenance of **8**, from that measurement (§5.7, 800 km, five seeds, the
/// GF-0 harness's `gf2_arms` and `gf2_b9_cost`): at 0, 8, 16 and 32 passes B4
/// fails on every seed and does not discriminate between them, and B1 and B2
/// read the same to four decimals; B8's 1-3-cell lakes pass at 8, 16 and 32
/// (and fail at 0, which is GF-2 alone). So B9 decides: the stage is one
/// sequential full-grid scan per pass, and at 16 passes B9 read 1.202 and
/// 1.132 against its 1.20 bar. 8 is the smallest count measured that keeps B8
/// passing. What 8 means physically: the move rule sheds `0.5·0.25` of a
/// cell's summed excess per pass, so on an isolated over-steep pair the
/// excess falls by a quarter each pass, and 8 passes leave `0.75^8 ≈ 0.100`
/// of it (arithmetic): the stage relaxes toward `θc`, it does not reach it.
/// The clock (§4.12, GF-7) scales it linearly.
pub const THRESHOLD_HILLSLOPE_PASSES: i32 = 8;

/// The rock-aware threshold hillslope's per-world inputs (GF-3,
/// `GEOLOGY_FIRST_SCOPE.md` §4.3).
pub struct ThresholdHillslope<'a> {
    /// The GF-1 column. **Read only**: the exposed rock at each pass
    /// (§2.5's rule, so a cap eroded through its contact mid-stage sheds at
    /// the substrate's angle from the next pass on). The caller accounts the
    /// stage's net change to regolith afterwards (§4.3: "the moved mass
    /// becomes regolith where it lands"; §4.9's caller-side rule), exactly as
    /// GF-2 does for stream power.
    pub column: &'a cartalith_terrain::geology::GeologyColumn,
    /// §2.5's `R_EXPOSE`, in normalised height units.
    pub r_expose: f32,
    /// The real cell width in metres, `map_width_km·1000/gw` (§4.3).
    pub cell_m: f64,
    /// Sea level, normalised: `1 − sea` of height spans `peak_m`.
    pub sea: f64,
    /// Peak altitude in metres (`metersPerUnit`'s anchor).
    pub peak_m: f64,
}

/// §4.3's critical height step for one cell: the largest 4-neighbour height
/// difference, in **normalised** units, that a slope at `theta_c_deg` spans
/// over one cell of `cell_m` metres:
///
/// ```text
/// talus = tan(θc) · cell_m · (1 − sea) / peak_m
/// ```
///
/// Why: `erode_thermal`'s scalar `talus = 0.012` is a raw normalised height
/// difference, so the angle it stands for changes with extent (7° at 800 km,
/// 87° at 5 km: `EROSION_GEOLOGICAL_TIME_SCOPE.md` §2). Scaling by the real
/// cell size and `peak_m` makes the threshold an angle at every extent.
///
/// Must never be called with `peak_m <= 0` or a non-positive `cell_m`
/// (asserted): either would turn the threshold into 0 (every slope fails) or
/// a negative number, both plausible-looking and wrong.
pub fn critical_talus(theta_c_deg: f64, cell_m: f64, sea: f64, peak_m: f64) -> f64 {
    assert!(peak_m > 0.0 && cell_m > 0.0, "critical_talus needs a positive peak_m and cell_m (got {peak_m}, {cell_m})");
    theta_c_deg.to_radians().tan() * cell_m * (1.0 - sea) / peak_m
}

/// GF-3, `GEOLOGY_FIRST_SCOPE.md` §4.3 (owner Rulings BH, BJ): **the
/// threshold hillslope stage.** `passes` passes of `erode_thermal`'s move rule
/// with a per-cell threshold, [`critical_talus`] of the critical angle `θc`
/// (`ROCK_PROPS`) of the rock each cell exposes *at the start of that pass*.
///
/// So strong rock (granite 60°) keeps steep faces that weak rock (shale 30°,
/// unconsolidated 33°) cannot hold: the excess over each rock's own angle
/// moves downslope, and a cap edge keeps a steeper face than the substrate
/// below it (§2.5, "the cap edge keeps a steep face (θc)").
///
/// Choices, each from the scope:
/// - the threshold is the **shedding** cell's (§4.3 indexes `talus_i` by the
///   cell whose excess moves);
/// - the exposed rock is re-read every pass (the contact switch of §4.1,
///   applied per pass), but the regolith thickness is the one the stage
///   started with, and the caller accounts the net change once, afterwards
///   (§4.9). GF-2 measured that writing regolith inside the iterations feeds
///   back (§5.6), so this stage does not;
/// - world maps wrap in x, as every generation stage does (`wrap`); the
///   manual Erode op's `erode_thermal` keeps the reference's no-wrap.
///
/// Must never write the column (it only reads it), and must never be used in
/// place of `erode_thermal` for the manual Erode op, whose golden is the
/// reference's scalar rule (§4.3). Panics if the column does not match the
/// grid or holds a rock byte that is not a [`cartalith_terrain::geology::Rock`]:
/// a threshold for "no rock" would have to be invented, and a mismatched
/// column describes a different world.
pub fn threshold_hillslope(fld: &mut [f32], w: usize, h: usize, passes: i32, wrap: bool, rock: &ThresholdHillslope) {
    use cartalith_terrain::geology::{Rock, ROCK_COUNT};
    assert!(rock.column.len() == w * h, "rock column is {} cells, needs exactly {} ({w}x{h})", rock.column.len(), w * h);
    // One threshold per rock type: `tan` once per rock, not once per cell.
    let mut by_rock = [0f64; ROCK_COUNT];
    for r in Rock::ALL {
        by_rock[r as usize] = critical_talus(r.props().theta_c_deg as f64, rock.cell_m, rock.sea, rock.peak_m);
    }
    let mut talus = vec![0f64; w * h];
    for _ in 0..passes {
        talus.par_iter_mut().enumerate().for_each(|(i, t)| {
            let r = rock
                .column
                .exposed(i, fld[i], rock.r_expose)
                .unwrap_or_else(|| panic!("cell {i}: the column holds no valid rock (rock_top {})", rock.column.rock_top[i]));
            *t = by_rock[r as usize];
        });
        thermal_core(fld, w, h, 1, 0.0, Some(&talus), wrap);
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
///
/// `Copy` because [`crate::tile::tile_erode`] derives a tile-scaled variant of
/// the caller's own params with `..*p` — a bag of eight scalars, so a copy is
/// what a reference would have cost anyway.
#[derive(Clone, Copy)]
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
    stream_power_kernel_bounded(fld, stress, resist, rain, w, h, p, None, None);
}

/// [`stream_power_kernel`] with the two boundary conditions a **tile-bounded**
/// re-run needs, and nothing else changed.
///
/// Both extras are `None` on the whole-world path, where this is
/// [`stream_power_kernel`] verbatim — the world entry point above is a
/// delegation, so the two cannot drift, and the golden-parity fixtures in
/// `tests/golden_parity_streampower.rs` exercise this body through it. The same
/// shape `cartalith_hydrology::build_channels_with_threshold` already uses for
/// the same reason: a tile needs one world-anchored number supplied rather than
/// derived, and the ported expression stays untouched.
///
/// - **`pinned`** — cells whose height is held fixed for the whole run
///   (Dirichlet base level). A pinned cell still routes: it is filled, it gets a
///   receiver, it carries drainage area and it passes sediment downstream. Only
///   its own `fld[i]` never moves — in the incision loop, in the deposition
///   loop, and in the final `clamp`, so it comes out bit-identical to what went
///   in. That last part is [`crate::tile::tile_erode`]'s whole seam guarantee
///   and is why the clamp is skipped too rather than assumed a no-op.
/// - **`area_seed`** — the per-cell starting drainage area, in place of the
///   world path's uniform `1.0` (one cell drains itself). This is the tile's
///   inflow boundary condition: EF-1's own units (`1/refine²` per fine cell,
///   plus the coarse network's inward crossings) make a tile's `A` mean the
///   same physical drained area the world pass's `A` means, so `Cc` — which is
///   `K·dt·A^m/L` — comes out on the world's own scale rather than the tile's.
///
/// # Panics
///
/// If either slice is supplied and is not exactly `w * h` long. A shorter one
/// would index out of bounds mid-loop after the expensive fill has already run;
/// a longer one means the caller is describing a different grid.
#[allow(clippy::too_many_arguments)]
pub fn stream_power_kernel_bounded(
    fld: &mut [f32],
    stress: &[f32],
    resist: &[f32],
    rain: &[f32],
    w: usize,
    h: usize,
    p: &StreamPowerParams,
    pinned: Option<&[bool]>,
    area_seed: Option<&[f32]>,
) {
    stream_power_core(fld, stress, resist, rain, w, h, p, pinned, area_seed, None);
}

/// GF-2's rock input to stream power (`GEOLOGY_FIRST_SCOPE.md` §4.1, owner
/// Rulings BH and BJ): the column the kernel reads.
///
/// **What it does.** It replaces the legacy strength factor
/// `max(0.05, 1 − 0.7·resist·R_i)` with `κ(exposed_i)^c`, where the exposed
/// rock follows §2.5's rule at the cell's *current* height, so it is
/// re-read after every iteration. That is the scope's in-loop contact switch:
/// once a cap is incised below its contact, the next iteration erodes the
/// substrate at the substrate's rate.
///
/// **What it must never do.** It never changes the kernel's routing (fill,
/// receivers, drainage area are frozen before the iterations, exactly as on
/// the legacy path), and it never reads `resistance_field`: with a rock input
/// the legacy factor is gone, not multiplied in (§4.1's formula has no
/// `resist` term). **It never writes `regolith`**: §4.9 says "the kernels do
/// not change; the caller adds `max(0, field_after − field_before)` to
/// regolith", so the caller accounts the call's net change
/// ([`account_regolith`]). A first build wrote regolith inside the
/// iterations; measured, it fed back (a pit filled in one iteration read as
/// unconsolidated, κ = 4, in the next) and roughly doubled the 1-3-cell lake
/// count against the control on all five GF-0 seeds at 800 km
/// (`GEOLOGY_FIRST_SCOPE.md` §5.6), so it was replaced by the scope's own
/// design.
pub struct StreamPowerRock<'a> {
    /// The GF-1 column. Read: `rock_top`, `rock_sub`, `contact`, `regolith`
    /// (§2.5's exposure rule). Written: only `contact`, and only by tectonic
    /// uplift (`dt·u`, the whole column rising), which is zero at the default
    /// `stream.uplift = 0`.
    pub column: &'a mut cartalith_terrain::geology::GeologyColumn,
    /// The rock contrast `c` of §4.1 (0 = uniform rock). The engine passes
    /// `tect.resist`, which §4.1 repurposes as `c` (§9 Q5).
    pub contrast: f64,
    /// §2.5's `R_EXPOSE`, already in normalised height units
    /// (`geology::m_to_norm(R_EXPOSE_M, sea, peak_m)`).
    pub r_expose: f32,
}

/// The stream-power `K` multiplier of one rock at contrast `c`: `κ^c`
/// (`GEOLOGY_FIRST_SCOPE.md` §4.1). `κ` is `ROCK_PROPS`' judgement column
/// (§2.3). At `c = 0` it is exactly `1.0` for every rock (`powf(0) = 1`), so
/// uniform rock is uniform by arithmetic identity.
///
/// Must never be clamped or floored: §4.1 dropped the legacy `0.05` floor
/// with the legacy factor, and `κ` is positive for every rock (asserted in
/// `cartalith_terrain::geology`'s table tests).
pub fn kappa_multiplier(rock: cartalith_terrain::geology::Rock, contrast: f64) -> f64 {
    (rock.props().kappa as f64).powf(contrast)
}

/// [`stream_power_kernel`] reading the GF-2 rock column instead of the legacy
/// resistance factor (`GEOLOGY_FIRST_SCOPE.md` §4.1, §4.9).
///
/// Why a separate entry point rather than a flag on the old one: the legacy
/// body is a golden-verified port (`tests/golden_parity_streampower.rs`), and
/// with no rock input the shared core takes the legacy arithmetic **by
/// control flow**, never by an identity of arithmetic (`MISTAKES.md`,
/// "Change generated output"). The whole-world path only: `tile_erode` gains
/// a rock input when EF-3 gains a production caller (§4.1).
///
/// Inside the iterations, per cell:
/// - the coefficient is `K·g·κ(exposed)^c·(1 + 2·ck·rain)·dt·A^m/L`, the
///   exposed rock read at the cell's height *before* this update -- i.e.
///   after the previous iteration, which is §4.1's "after it updates
///   `fld[i]`, if `fld[i] < contact[i]`, set `Cc[i] ← Cc_sub[i]`";
/// - the regolith thickness is the one the call started with (§4.9: the
///   caller accounts deposits and stripping once, on the call's net change).
///
/// Must never be called with a column shorter than the grid (it panics on
/// the index, which is the right failure: a mismatched column describes a
/// different world).
pub fn stream_power_kernel_rock(
    fld: &mut [f32],
    stress: &[f32],
    rain: &[f32],
    w: usize,
    h: usize,
    p: &StreamPowerParams,
    rock: &mut StreamPowerRock,
) {
    assert!(rock.column.len() == w * h, "rock column is {} cells, needs exactly {} ({w}x{h})", rock.column.len(), w * h);
    stream_power_core(fld, stress, &[], rain, w, h, p, None, None, Some(rock));
}

/// The body shared by [`stream_power_kernel_bounded`] (legacy, `rock =
/// None`) and [`stream_power_kernel_rock`]. With `rock = None` every
/// statement below is the legacy port's, unchanged: the rock branches are
/// separate arms, so the kernel goldens stay bit-identical by construction.
#[allow(clippy::too_many_arguments)]
fn stream_power_core(
    fld: &mut [f32],
    stress: &[f32],
    resist: &[f32],
    rain: &[f32],
    w: usize,
    h: usize,
    p: &StreamPowerParams,
    pinned: Option<&[bool]>,
    area_seed: Option<&[f32]>,
    mut rock: Option<&mut StreamPowerRock>,
) {
    let n = w * h;
    if let Some(m) = pinned {
        assert!(m.len() == n, "pinned mask is {} cells, needs exactly {n} ({w}x{h})", m.len());
    }
    if let Some(a) = area_seed {
        assert!(a.len() == n, "area seed is {} cells, needs exactly {n} ({w}x{h})", a.len());
    }
    // One closure rather than an `if let` at four sites. It is a runtime check,
    // **not** a compiled-away one: `pinned` is a value, and this function is
    // far too large for LLVM to inline into `stream_power_kernel` and
    // specialise the `None`. Measured rather than assumed, at 1024x1024,
    // median of seven, harness run alone -- `iters: 0` is 195.2/195.4 ms across
    // two runs against a single pre-change sample of 196.6, and `iters: 9` is
    // 289.2/289.1 against 284.9. The setup is unaffected (the check is only in
    // the iteration loops) and the iteration loops are ~1.5% dearer, which is
    // ~0.1% of a generation -- but the pre-change figure is one sample with no
    // spread, so **no difference is established** and none is claimed. The
    // zero-cost version is a private generic inner function monomorphised over
    // a pin type; it is not worth the duplication in a golden-parity kernel for
    // a number this size.
    let is_pinned = |i: usize| pinned.is_some_and(|m| m[i]);
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
    let mut area = match area_seed {
        Some(a) => a.to_vec(),
        None => vec![1f32; n],
    };
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
    // GF-2 (§4.1): with a rock input, `cc` holds everything in the
    // coefficient except the rock multiplier -- `K·g·(1 + 2·ck·rain)·dt·A^m/L`
    // -- and the loop multiplies in `κ(exposed)^c` per iteration, because the
    // exposed rock changes as the cell is lowered through its contact. The
    // legacy arm below is the ported expression, untouched.
    // `kc[k]` = κ_k^c for each rock, looked up by the exposed rock's index.
    let kc: Option<[f64; cartalith_terrain::geology::ROCK_COUNT]> = rock
        .as_ref()
        .map(|r| cartalith_terrain::geology::Rock::ALL.map(|k| kappa_multiplier(k, r.contrast)));
    if kc.is_some() {
        cc.par_iter_mut().enumerate().for_each(|(i, cc_i)| {
            let r = rcv[i];
            if r < 0 {
                return;
            }
            let l = rdist[i] as f64;
            *cc_i = k_coef * (1.0 + ck * 2.0 * rain[i] as f64) * dt * (area[i] as f64).powf(m) / l;
        });
    } else {
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
    }
    // The rock multiplier at cell `i`, read at its current height through
    // §2.5's exposure rule (`GeologyColumn::exposed`). A column always has a
    // top rock (GF-1 writes one per cell). An out-of-range code is refused
    // loudly rather than read as some plausible rock (`MISTAKES.md`: never
    // encode "no value" as a plausible value); `build_geology` cannot write
    // one.
    let rock_mult = |col: &cartalith_terrain::geology::GeologyColumn, kc: &[f64; cartalith_terrain::geology::ROCK_COUNT], r_expose: f32, i: usize, z: f32| -> f64 {
        let k = col.exposed(i, z, r_expose).expect("GF-2: the rock column holds a code that is not a rock");
        kc[k as usize]
    };

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
            if r < 0 || is_pinned(i) {
                continue;
            }
            let r = r as usize;
            match (rock.as_deref_mut(), kc.as_ref()) {
                (Some(rk), Some(kc)) => {
                    // GF-2: the contact switch (§4.1) is this read -- the
                    // exposed rock at the height the previous iteration left.
                    let c = cc[i] * rock_mult(&*rk.column, kc, rk.r_expose, i, fld[i]);
                    let lifted = fld[i] as f64 + dt * u[i] as f64;
                    let val = (lifted + c * fld[r] as f64) / (1.0 + c);
                    let new = val as f32;
                    // Tectonic uplift raises the whole column, contact with
                    // it (the rebound rule of §4.2 applied to uplift; zero at
                    // the default `stream.uplift = 0`). NaN stays NaN on a
                    // single-layer cell.
                    if u[i] != 0.0 {
                        rk.column.contact[i] = (rk.column.contact[i] as f64 + dt * u[i] as f64) as f32;
                    }
                    fld[i] = new;
                }
                _ => {
                    let c = cc[i];
                    let val = (fld[i] as f64 + dt * u[i] as f64 + c * fld[r] as f64) / (1.0 + c);
                    fld[i] = val as f32;
                }
            }
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

                // A pinned cell deposits nothing (its height is fixed) and
                // therefore keeps all of its sediment, which the carry below
                // then passes downstream unchanged -- mass conserving, and the
                // two `sed[i]` decrements are inside the same guard as the
                // `fld[i]` writes they pay for.
                if !is_pinned(i) {
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
                }
                sed[r] = (sed[r] as f64 + sed[i] as f64) as f32;
            }
        }
    }

    fld.par_iter_mut().enumerate().for_each(|(i, v)| {
        if !is_pinned(i) {
            *v = v.clamp(0.0, 1.0)
        }
    });
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

/// GF-2 (`GEOLOGY_FIRST_SCOPE.md` §4.2, §3.2 item 4): **rebound lifts the
/// column.** Adds to every contact the increment the rebound actually applied
/// to the field (`after − before`, so `isostatic_rebound`'s own 0..1 clamp is
/// honoured rather than re-derived from the blur).
///
/// Why: rebound raises the bedrock and whatever lies in it. Without this a
/// rebounded cap would read as having risen *through* its own contact,
/// re-burying a substrate erosion had exposed, or exposing one erosion never
/// reached. Regolith is untouched: it rides on the bedrock, so its thickness
/// does not change.
///
/// Must never be applied to any change except a rebound's (or another
/// whole-column uplift): an erosional lowering does not lower the contact.
/// A single-layer cell's contact is NaN and stays NaN.
pub fn lift_column(column: &mut cartalith_terrain::geology::GeologyColumn, before: &[f32], after: &[f32]) {
    column.contact.par_iter_mut().enumerate().for_each(|(i, c)| {
        let inc = after[i] as f64 - before[i] as f64;
        if inc != 0.0 {
            *c = (*c as f64 + inc) as f32;
        }
    });
}

/// GF-2 (`GEOLOGY_FIRST_SCOPE.md` §4.9, §2.5): **regolith is consumed before
/// bedrock, and deposition becomes regolith.** For a process that changed the
/// field from `before` to `after`:
/// - a lowering first strips regolith (never below `0.0`, bare rock);
/// - a rise is added to regolith when `gains_are_deposit` -- a stream-power
///   call's net gain (its internal deposition and the pits it fills) and
///   sediment routing (`route_sediment`) are; a rebound is not, and must go
///   through [`lift_column`] instead.
///
/// This is §4.9's caller-side rule, applied once per call on the call's net
/// change: "the kernels do not change; the caller adds `max(0, field_after −
/// field_before)` to regolith for each deposition step".
///
/// Must never be called for a whole-column uplift (it would bury the rock
/// under a regolith that was never deposited).
pub fn account_regolith(column: &mut cartalith_terrain::geology::GeologyColumn, before: &[f32], after: &[f32], gains_are_deposit: bool) {
    column.regolith.par_iter_mut().enumerate().for_each(|(i, r)| {
        let dz = after[i] as f64 - before[i] as f64;
        if dz < 0.0 || (gains_are_deposit && dz > 0.0) {
            *r = ((*r as f64 + dz).max(0.0)) as f32;
        }
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
