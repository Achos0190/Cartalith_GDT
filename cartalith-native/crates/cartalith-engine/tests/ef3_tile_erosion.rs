//! **EF-3** — tile-bounded erosion (`cartalith_erosion::tile`) against a **real
//! generated world** and **real EF-0 output**, which is the only combination
//! that can check the claim EF-3 actually makes.
//!
//! These live in `cartalith-engine` for the same reason
//! `vector_features_real_world.rs` does: the function under test is another
//! crate's, and this is the crate that owns `generate_terrain`. It is also the
//! only crate that can see `cartalith-erosion` and `cartalith-terrain` at once
//! — `cartalith-erosion` does not depend on `cartalith-spatial`, so it cannot
//! so much as name the `FloatRegion` `amplify_region` takes. That is why
//! `tile.rs`'s own tests deliberately mock the refined tile and everything
//! below uses the real one, the same split EF-1 makes between
//! `cartalith-hydrology`'s unit tests and its `tile_hydrology.rs`.
//!
//! **Not golden-parity tests, and they do not pretend to be.** The reference
//! HTML never re-runs erosion at zoom — its `amplifyRegion`/`refineTile` are
//! elevation-only — so there is no JS oracle and no captured value to match.
//! `cartalith-porting-discipline`'s rule for a genuinely new capability applies
//! instead: check it against the properties it is supposed to have. Those are:
//!
//! 1. measured against a drainage network **held fixed from the un-eroded
//!    tile**, it deepens the high-drainage ground *with the deposition term
//!    off*, and the resolution correction is what does that rather than the
//!    pass merely running — while at the engine's own `deposit: 0.3` it moves
//!    the same measure the other way, exactly as the world's own pass does to
//!    the world's own field
//!    (`tile_erosion_incises_along_a_fixed_drainage_network_only_with_deposition_off`);
//! 2. tiling it does not change the answer
//!    (`two_tiles_agree_with_the_one_tile_that_covers_them`);
//! 3. the coarse network's inflow reaches the erosion, and reaches it where the
//!    water goes (`the_coarse_inflow_boundary_condition_deepens_the_trunk`);
//! 4. the whole EF-0 -> EF-3 composition is deterministic across a regenerated
//!    world (`the_whole_composition_is_deterministic`);
//! 5. the per-tile cost is what it is, stated rather than implied
//!    (`measure_tile_erosion_cost`, `#[ignore]`d — see its own note).
//!
//! Every assertion is a **property**, never a captured value, so none of this
//! needs re-baselining when generation changes.
//!
//! # Two approximations this file owns, rather than hides
//!
//! - **`up()` is a nearest-cell upsample of `stress`/`resist`/`rainfall`.**
//!   EF-0 refines elevation and nothing else, so there is no refined climate or
//!   lithology to hand the kernel. This is the same first-pass approximation
//!   EF-1's own header states for its uniform rainfall, and it is the caller's
//!   to own — `tile_erode` requires all three rather than inventing them.
//! - **`coarse_flow` is recomputed here, not read off `WorldState`.** With
//!   `carve_rivers = true` (the default) `WorldState::field` and
//!   `WorldState::flow_discharge` stop agreeing, because the carve mutates
//!   `field` after the discharge is taken — EF-1's header documents the same
//!   hazard. `compute_flow` over the final `field` gives the self-consistent
//!   pair the boundary condition needs.

use cartalith_engine::elevation::world_amplify_opts;
use cartalith_engine::{generate_terrain, WorldParams, WorldState};
use cartalith_erosion::tile::{ring_mask, tile_erode, uniform_area_seed};
use cartalith_erosion::{stream_power_kernel_bounded, StreamPowerParams};
use cartalith_hydrology::compute_flow;
use cartalith_hydrology::tile::TilePlacement;
use cartalith_spatial::FloatRegion;
use cartalith_terrain::amplify::amplify_region;

const GW: usize = 256;
const GH: usize = 192;
const SEED: i32 = 20260920;

/// Three all-land, high-relief rectangles of the fixture world, found by
/// sweeping it rather than picked by eye — `probe` in the history of this file;
/// the sweep's own answer for the single best 24x24 was `(48, 120)`, relief
/// `0.5594` against a sea level of `0.4200`.
const TILES: [(usize, usize); 3] = [(48, 120), (40, 40), (180, 120)];
const COLS: usize = 24;

fn world() -> (WorldState, WorldParams) {
    let mut p = WorldParams::defaults(GW, GH, SEED);
    p.map_width_km = 800.0;
    (generate_terrain(&p), p)
}

fn place_at(x0: usize, y0: usize, cols: usize, rows: usize, refine: usize) -> TilePlacement {
    TilePlacement { coarse_w: GW, coarse_h: GH, x0, y0, cols, rows, refine, world: false }
}

/// **EF-0's real refinement**, over exactly the fine-cell centres `place`
/// describes.
///
/// The two conventions have to be reconciled here rather than assumed
/// compatible, and getting it wrong is silent: `amplify_region` maps output
/// pixel `0..out_w-1` linearly onto `[rx, rx + rw - 1]` in coarse **cell-centre**
/// coordinates, while `TilePlacement` covers coarse cells `x0..x0+cols` in
/// **pixel** coordinates, where a cell centre is `i + 0.5`. So fine cell `0`
/// must land on `x0 + 0.5/refine - 0.5` and fine cell `fw-1` on
/// `x0 + cols - 0.5/refine - 0.5`, which is the region below.
/// `the_ef0_region_lands_on_the_placements_own_fine_cells` checks it against
/// the placement's own `fine_to_coarse` rather than against this comment.
fn ef0(ws: &WorldState, p: &WorldParams, place: &TilePlacement) -> Vec<f32> {
    let (fw, fh) = (place.fine_w(), place.fine_h());
    let r = place.refine as f64;
    let region = FloatRegion {
        x: place.x0 as f64 - 0.5 + 0.5 / r,
        y: place.y0 as f64 - 0.5 + 0.5 / r,
        w: place.cols as f64 + 1.0 - 1.0 / r,
        h: place.rows as f64 + 1.0 - 1.0 / r,
    };
    amplify_region(&ws.field, GW, GH, &region, fw, fh, &world_amplify_opts(ws, p, fw))
}

/// The world's own light erosion pass, rebuilt from `WorldParams` exactly as
/// `generate_terrain`'s `carve_rivers` block builds it — including
/// `light_iters = max(4, round(iters * 0.6))`.
fn light_params(ws: &WorldState, p: &WorldParams) -> StreamPowerParams {
    StreamPowerParams {
        k: p.stream.k,
        uplift: p.stream.uplift,
        deposit: p.stream.deposit,
        climate_k: p.stream.climate_k,
        iters: ((p.stream.iters as f64 * 0.6).round() as i32).max(4),
        resist: p.tect.resist,
        g: p.planet.g,
        world: false,
        sea: ws.sea_level,
    }
}

/// Nearest-coarse-cell upsample onto the tile's fine grid — see the header.
fn up(coarse: &[f32], place: &TilePlacement) -> Vec<f32> {
    let (fw, fh) = (place.fine_w(), place.fine_h());
    let mut out = vec![0f32; fw * fh];
    for fy in 0..fh {
        for fx in 0..fw {
            let (cx, cy) = (place.x0 + fx / place.refine, place.y0 + fy / place.refine);
            out[fy * fw + fx] = coarse[cy * GW + cx];
        }
    }
    out
}

/// Everything a tile needs besides its elevation, assembled once.
struct TileIn {
    stress: Vec<f32>,
    resist: Vec<f32>,
    rain: Vec<f32>,
    seed: Vec<f32>,
}

fn tile_in(ws: &WorldState, place: &TilePlacement) -> TileIn {
    let (fw, fh) = (place.fine_w(), place.fine_h());
    TileIn {
        stress: up(&ws.stress_field, place),
        resist: up(&ws.resistance_field, place),
        rain: up(&ws.rainfall, place),
        seed: uniform_area_seed(fw, fh, place.refine),
    }
}

fn erode(base: &[f32], place: &TilePlacement, t: &TileIn, p: &StreamPowerParams) -> Vec<f32> {
    let (fw, fh) = (place.fine_w(), place.fine_h());
    let mut out = base.to_vec();
    tile_erode(&mut out, &t.stress, &t.resist, &t.rain, fw, fh, p, place.refine, &t.seed);
    out
}

// -- the oracles, written out here rather than borrowed from the crates under
//    test: an oracle built from the same function it is checking cannot catch
//    that function being wrong (EF-1's tests make the same choice).

fn on_ring(i: usize, w: usize, h: usize) -> bool {
    let (x, y) = (i % w, i / w);
    x == 0 || y == 0 || x + 1 == w || y + 1 == h
}

/// D8 steepest descent, `-1` for a pit.
fn recv(f: &[f32], w: usize, h: usize, i: usize) -> i64 {
    let (x, y) = ((i % w) as i64, (i / w) as i64);
    let hh = f[i] as f64;
    let (mut best, mut bd) = (-1i64, 0f64);
    for dy in -1i64..=1 {
        for dx in -1i64..=1 {
            if dx == 0 && dy == 0 {
                continue;
            }
            let (nx, ny) = (x + dx, y + dy);
            if nx < 0 || ny < 0 || nx >= w as i64 || ny >= h as i64 {
                continue;
            }
            let j = ny * w as i64 + nx;
            let d = (hh - f[j as usize] as f64) / (dx as f64).hypot(dy as f64);
            if d > bd {
                bd = d;
                best = j;
            }
        }
    }
    best
}

/// Drainage area by D8 accumulation, with the outer ring as an open boundary —
/// the same rule `tile_flow` uses, so a cell's number means "fine cells
/// upstream of here, inside this tile".
fn accum(f: &[f32], w: usize, h: usize) -> Vec<f32> {
    let n = w * h;
    let mut order: Vec<u32> = (0..n as u32).collect();
    order.sort_by(|&a, &b| f[b as usize].partial_cmp(&f[a as usize]).unwrap_or(std::cmp::Ordering::Equal));
    let mut acc = vec![1f32; n];
    for &i in &order {
        let i = i as usize;
        if on_ring(i, w, h) {
            continue;
        }
        let r = recv(f, w, h, i);
        if r >= 0 {
            acc[r as usize] += acc[i];
        }
    }
    acc
}

/// **The kernel's own drainage network**, taken once from one field and then
/// held fixed while *other* fields are measured against it.
struct Network {
    /// Multiple-flow-direction drainage area, `slope^1.1` weights, in
    /// `seed`'s own units.
    area: Vec<f32>,
    /// D8 steepest-descent receiver over the **filled** surface, `-1` for none.
    rcv: Vec<i32>,
}

/// The three setup passes `stream_power_kernel_bounded` runs before its first
/// iteration — priority-flood depression fill (`+1e-5` per step), D8 receivers
/// over the filled surface, and multiple-flow-direction accumulation with
/// `slope^1.1` weights — written out here rather than called, for the reason
/// this section gives: an oracle taken from the function under test cannot
/// catch that function being wrong.
///
/// One substitution: `std::collections::BinaryHeap` stands in for the kernel's
/// own ported `MinHeap`. Both are binary heaps with an unspecified tie-break,
/// and the fill's `+1e-5` step is what makes exact ties vanishingly rare on a
/// real field, so the two orders agree except where the answer does not depend
/// on them.
fn network(f: &[f32], w: usize, h: usize, seed: &[f32]) -> Network {
    use std::cmp::Reverse;
    // Monotone f32 -> u32 so a `BinaryHeap` can order elevations directly.
    let key = |v: f32| {
        let b = v.to_bits();
        if b & 0x8000_0000 != 0 {
            !b
        } else {
            b | 0x8000_0000
        }
    };
    let n = w * h;
    let d8 = |dx: i64, dy: i64| (dx as f64).hypot(dy as f64);
    let mut filled = f.to_vec();
    let mut done = vec![false; n];
    let mut order: Vec<u32> = Vec::with_capacity(n);
    let mut heap = std::collections::BinaryHeap::with_capacity(n);
    for (x, y) in (0..w).map(|x| (x, 0)).chain((0..w).map(|x| (x, h - 1))).chain((0..h).flat_map(|y| [(0, y), (w - 1, y)])) {
        let i = y * w + x;
        if !done[i] {
            done[i] = true;
            heap.push(Reverse((key(filled[i]), i as u32)));
        }
    }
    while let Some(Reverse((_, i))) = heap.pop() {
        order.push(i);
        let (x, y) = ((i as usize % w) as i64, (i as usize / w) as i64);
        for dy in -1i64..=1 {
            for dx in -1i64..=1 {
                let (nx, ny) = (x + dx, y + dy);
                if (dx == 0 && dy == 0) || nx < 0 || ny < 0 || nx >= w as i64 || ny >= h as i64 {
                    continue;
                }
                let j = (ny * w as i64 + nx) as usize;
                if done[j] {
                    continue;
                }
                done[j] = true;
                if filled[j] <= filled[i as usize] {
                    filled[j] = (filled[i as usize] as f64 + 1e-5) as f32;
                }
                heap.push(Reverse((key(filled[j]), j as u32)));
            }
        }
    }
    assert_eq!(order.len(), n, "the priority flood must reach every cell");

    let mut rcv = vec![-1i32; n];
    for (i, r) in rcv.iter_mut().enumerate() {
        let (x, y) = ((i % w) as i64, (i / w) as i64);
        let hh = filled[i] as f64;
        let mut best_s = 0.0f64;
        for dy in -1i64..=1 {
            for dx in -1i64..=1 {
                let (nx, ny) = (x + dx, y + dy);
                if (dx == 0 && dy == 0) || nx < 0 || ny < 0 || nx >= w as i64 || ny >= h as i64 {
                    continue;
                }
                let j = (ny * w as i64 + nx) as usize;
                let sl = (hh - filled[j] as f64) / d8(dx, dy);
                if sl > best_s {
                    best_s = sl;
                    *r = j as i32;
                }
            }
        }
    }

    let mut area = seed.to_vec();
    for &i in order.iter().rev() {
        let i = i as usize;
        let (x, y) = ((i % w) as i64, (i / w) as i64);
        let (hh, a) = (filled[i] as f64, area[i] as f64);
        let mut sw = 0.0f64;
        let mut slopes = [0.0f64; 9];
        for dy in -1i64..=1 {
            for dx in -1i64..=1 {
                let (nx, ny) = (x + dx, y + dy);
                if (dx == 0 && dy == 0) || nx < 0 || ny < 0 || nx >= w as i64 || ny >= h as i64 {
                    continue;
                }
                let sl = (hh - filled[(ny * w as i64 + nx) as usize] as f64) / d8(dx, dy);
                if sl > 0.0 {
                    slopes[((dy + 1) * 3 + (dx + 1)) as usize] = sl.powf(1.1);
                    sw += sl.powf(1.1);
                }
            }
        }
        if sw <= 0.0 {
            continue;
        }
        for dy in -1i64..=1 {
            for dx in -1i64..=1 {
                let wgt = slopes[((dy + 1) * 3 + (dx + 1)) as usize];
                if wgt > 0.0 {
                    let j = ((y + dy) * w as i64 + x + dx) as usize;
                    area[j] = (area[j] as f64 + a * wgt / sw) as f32;
                }
            }
        }
    }
    Network { area, rcv }
}

/// **The measure everything below turns on: how much more deeply a cell is cut
/// when more water passes through it.**
///
/// For every cell, the height of the two neighbours **perpendicular to
/// `net`'s flow direction there**, minus its own — positive means the cell
/// sits in a channel cut into its surroundings — correlated against `log10` of
/// `net`'s drainage area through it. Returns `(regression slope, Pearson r, n)`.
///
/// # Why `net` is an argument and not recomputed from `f`
///
/// Because recomputing it from `f` is **circular**, and this file shipped that
/// version first. Drainage area is a function of the surface, so a groove that
/// erosion cut is a groove that collects water *in the recomputed network* —
/// the groove certifies itself by existing, and the correlation rises for any
/// deepening of any hollow whether or not the water was ever going there.
///
/// Passing the network in fixes that: take it **once** from the pre-erosion
/// (EF-0-only) field, then measure both the un-eroded control and the eroded
/// output against that same network, the same cells, the same `x` values. Only
/// `y` moves, so a rise means *erosion deepened the places the water already
/// went* — which is the claim EF-3 is actually making.
///
/// # Why this measure and not the obvious one
///
/// The obvious acceptance test, and the one `ELEVATION_FIELD_ARCHITECTURE_RESEARCH.md`
/// itself suggests — *"ridges in the refined tile align with drainage divides"*
/// — was tried first and is **tautological**. Measured on this engine's own
/// EF-0 output with no erosion at all: the top decile of topographic position
/// index are drainage divides **99.9%** of the time, and TPI correlates with
/// log drainage area at **r = -0.72**. That is not evidence of erosion; it is
/// arithmetic. Flow runs downhill by construction, so on *any* surface the
/// hollows collect the water and the highs shed it, noise included.
///
/// A surface that has actually been cut by running water differs in something
/// a random rough surface does not have: the depth of a hollow scales with how
/// much drains through it. A noise field's hollows are as deep as the noise is,
/// whatever their catchment. That is what this measures, and the control is
/// EF-0's own un-eroded tile.
///
/// The slope-area exponent (the other classic signature) was also tried and
/// rejected as a bar: this engine's *own world pass* does not produce one —
/// measured `-0.280` before its light erosion and `-0.259` after, i.e. slightly
/// *less* concave — because `StreamParams`' default `uplift` is `0.0`, so
/// there is no uplift/incision balance for a graded profile to form against. A
/// bar a faithful tile cannot meet because the world does not meet it either
/// would be a bar on the wrong thing.
fn incision_vs_area(f: &[f32], w: usize, h: usize, net: &Network) -> (f64, f64, usize) {
    let a = &net.area;
    let (mut xs, mut ys) = (Vec::new(), Vec::new());
    for y in 2..h - 2 {
        for x in 2..w - 2 {
            let i = y * w + x;
            let r = net.rcv[i] as i64;
            if r < 0 {
                continue;
            }
            let (rx, ry) = ((r as usize % w) as i64, (r as usize / w) as i64);
            let (dx, dy) = (rx - x as i64, ry - y as i64);
            // The two cells across the flow, not along it.
            let (px, py) = (-dy, dx);
            let n1 = ((y as i64 + py) as usize) * w + (x as i64 + px) as usize;
            let n2 = ((y as i64 - py) as usize) * w + (x as i64 - px) as usize;
            xs.push((a[i] as f64).log10());
            ys.push((f[n1] as f64 + f[n2] as f64) * 0.5 - f[i] as f64);
        }
    }
    assert!(xs.len() > 1000, "only {} cells measured -- a silently-tiny sample", xs.len());
    let n = xs.len() as f64;
    let (mx, my) = (xs.iter().sum::<f64>() / n, ys.iter().sum::<f64>() / n);
    let (mut sxy, mut sxx, mut syy) = (0.0, 0.0, 0.0);
    for k in 0..xs.len() {
        let (u, v) = (xs[k] - mx, ys[k] - my);
        sxy += u * v;
        sxx += u * u;
        syy += v * v;
    }
    (sxy / sxx, sxy / (sxx.sqrt() * syy.sqrt()).max(1e-300), xs.len())
}

/// Sum of squared discrete Laplacians: how much high-frequency content a patch
/// carries. EF-0's own tests use it for the same purpose — here it is the guard
/// that EF-3 *organises* detail rather than smoothing it away or manufacturing
/// numerical roughness.
fn curvature(v: &[f32], w: usize, h: usize) -> f64 {
    let mut e = 0.0;
    for y in 1..h - 1 {
        for x in 1..w - 1 {
            let c = v[y * w + x] as f64;
            let lx = v[y * w + x - 1] as f64 - 2.0 * c + v[y * w + x + 1] as f64;
            let ly = v[(y - 1) * w + x] as f64 - 2.0 * c + v[(y + 1) * w + x] as f64;
            e += lx * lx + ly * ly;
        }
    }
    e
}

fn relief(v: &[f32]) -> f64 {
    let (lo, hi) = v.iter().fold((f32::MAX, f32::MIN), |(a, b), x| (a.min(*x), b.max(*x)));
    (hi - lo) as f64
}

fn rms(a: &[f32], b: &[f32]) -> f64 {
    (a.iter().zip(b).map(|(x, y)| (*x as f64 - *y as f64).powi(2)).sum::<f64>() / a.len() as f64).sqrt()
}

// -- 1. the headline claim ---------------------------------------------------

/// **Property 1, restated to what it actually measures.** Against a drainage
/// network taken **once from the un-eroded tile and held fixed**, `tile_erode`
/// deepens the high-drainage ground — *only* with the deposition term off. At
/// the engine's own `deposit: 0.3` it does the opposite, and **so does the
/// world's own pass on the world's own field**, which is the control that says
/// this is the kernel's signature rather than anything tiling did.
///
/// # What the earlier version of this test claimed, and why it was wrong
///
/// It asserted *"`tile_erode` raises the incision/drainage correlation"* at the
/// world's own parameters, and it was measuring drainage area **recomputed on
/// its own output**. That is circular — a groove certifies itself by existing —
/// and the two answers are not close. Same six cases, same runs, at the world's
/// own `deposit: 0.3`: recomputed, `-0.027 .. +0.049`, five of the six
/// positive; against the fixed network, `-0.050 .. -0.148`, six of six
/// negative. So the circular metric did not merely overstate the effect — it
/// reported the opposite sign five times out of six, and on the sixth it
/// reported a fall `4.5x` smaller than the real one. (The version this replaced
/// routed its circular measure differently again — unfilled D8 accumulation
/// rather than the kernel's priority-flood + MFD — and read `+0.041 .. +0.086`;
/// the numbers above are the same routing as the fixed network, so only the
/// circularity differs.) The disagreement is asserted below rather than left as
/// prose, so re-introducing the circular measure goes red.
///
/// # The bars, and what each one is for
///
/// **The world control, first, because nothing below means anything without
/// it.** The same kernel on `ws.field` at the world's own resolution — world
/// path, no pin, no seed, `k` untouched, no tile anywhere — measured by this
/// same fixed-network metric: `+0.0356` with deposition off, `-0.0551` with the
/// world's own `0.3`. A `refine = 1` tile reproduces both to three decimals
/// (`+0.0360` against `+0.0356`, `-0.0552` against `-0.0551`), the only
/// difference being the pinned ring. So the sign flip is **the engine's
/// erosion, at any resolution**, and EF-3's job is to reproduce it, not to
/// improve on it.
///
/// **With `deposit: 0.0`** — incision only — `tile_erode` raises the
/// fixed-network correlation in all six cases, `+0.058 .. +0.087`, and the
/// resolution correction is what does it: the same kernel with `k` left at the
/// world's manages `+0.017 .. +0.031`. The bar is `+0.02` on the gain and
/// `+0.02` on the margin over that control; the weakest case clears them by
/// `2.9x` and `1.8x`.
///
/// **With the world's own `deposit: 0.3`** the correlation **falls** in all six,
/// `-0.050 .. -0.148`, and the unscaled control is inert (`-0.0042 .. +0.0031`).
/// Both are asserted, because both are the finding: the fall is real, and the
/// resolution correction is what makes the pass do anything at all.
///
/// # Why it falls, measured rather than supposed
///
/// Not "deposition smooths the detail away" — deposition **inverts** the
/// drainage signal, and the kernel says so line by line. `sed[i]` is this
/// iteration's own incision, it accumulates down the receiver chain
/// (`sed[r] += sed[i]`), and the transport capacity it is tested against is
/// `0.005·A^0.5·slope` — which a low-gradient valley floor cannot meet however
/// large `A` is. With `uplift: 0.0` (the world's default) the deposition
/// ceiling `old_h[i] + dt·u[i]` is **exactly the cell's height at the start of
/// the iteration**, so the trunk is refilled to where it began while the
/// low-drainage hillslopes keep their incision. Measured on tile `(180,120)` at
/// `refine 8`, mean `Δh` by decile of the fixed drainage area: at `deposit 0.0`
/// the top decile falls `-4.52e-3` against the bottom decile's `-6.07e-4`
/// (7.4x deeper where the water is); one step to `deposit 0.05` and the top
/// decile is `-1.20e-3` against `-5.43e-4` (2.2x), while the bottom decile has
/// barely moved. The cancellation is aimed at exactly the cells the metric is
/// asking about.
///
/// It is not an iteration-count artefact either, though iterations make it
/// worse: at `deposit 0.3` that same `(180,120)` tile measures `+0.003 /
/// -0.004 / -0.039 / -0.148 / -0.254 / -0.327 / -0.375` at `iters`
/// 1/2/4/9/18/36/72, crossing zero after the very first iteration. `(48,120)`
/// at `refine 8` crosses later, between 2 and 4 — still below the world's own
/// 9, which is the point.
///
/// # The consequence for EF-3, stated rather than buried
///
/// A caller who wants detail organised **by the tile's own drainage** runs the
/// tile with `deposit: 0.0`. A caller who wants the tile to look like the world
/// it is a zoom of runs the world's own parameters and gets the world's own
/// behaviour, sign included. `tile_erode` does not choose: `p.deposit` reaches
/// the kernel untouched (`k` is the one field it corrects), and this test pins
/// both outcomes so neither can change unnoticed.
///
/// The regression *slope* ratio is printed and not asserted, deliberately: it
/// is the same quantity in units that move with the tile's relief, so a bar on
/// it would be a bar on the fixture. The correlation is scale-free, which is
/// why it is the one with a number on it. (EF-0's own zoom sweep prints
/// curvature-per-texel and asserts only the ratio, for the same reason.)
#[test]
fn tile_erosion_incises_along_a_fixed_drainage_network_only_with_deposition_off() {
    let (ws, p) = world();
    let sp = light_params(&ws, &p);
    // The fixture must be the world's own pass, or this measures a parameter
    // set nothing generates. Literals, not `assert_eq!(sp.iters, computed)`.
    assert_eq!(sp.iters, 9, "the default light pass is max(4, round(15*0.6))");
    assert_eq!(sp.uplift, 0.0, "the default world has no uplift term");
    assert_eq!(sp.deposit, 0.3, "the default world deposits -- which is the whole subject below");

    // -- the world control: this kernel, this metric, no tile anywhere.
    let coarse_net = network(&ws.field, GW, GH, &vec![1f32; GW * GH]);
    let (_, rw0, _) = incision_vs_area(&ws.field, GW, GH, &coarse_net);
    let world_moved = |dep: f64| {
        let mut f = ws.field.as_ref().clone();
        let wp = StreamPowerParams { deposit: dep, ..sp };
        stream_power_kernel_bounded(&mut f, &ws.stress_field, &ws.resistance_field, &ws.rainfall, GW, GH, &wp, None, None);
        incision_vs_area(&f, GW, GH, &coarse_net).1 - rw0
    };
    let (w_off, w_on) = (world_moved(0.0), world_moved(sp.deposit));
    println!("world control {GW}x{GH}, no tile, fixed-net r {rw0:+.4}: deposit 0.0 moved {w_off:+.4}, deposit {} moved {w_on:+.4}", sp.deposit);
    assert!(w_off >= 0.01, "the world's own kernel must RAISE this measure with deposition off; it moved {w_off:+.4}");
    assert!(
        w_on <= -0.01,
        "the world's own kernel must LOWER this measure at its own deposit; it moved {w_on:+.4}. If it does not, the tile numbers below are a tile finding after all and this test's whole framing is wrong"
    );

    let mut cases = 0usize;
    for dep in [0.0f64, sp.deposit] {
        let sp = StreamPowerParams { deposit: dep, ..sp };
        for (x0, y0) in TILES {
            for refine in [4usize, 8] {
                let place = place_at(x0, y0, COLS, COLS, refine);
                let (fw, fh) = (place.fine_w(), place.fine_h());
                let tin = tile_in(&ws, &place);
                let base = ef0(&ws, &p, &place);
                let tag = format!("deposit {dep:.1} ({x0},{y0}) refine {refine} -> {fw}x{fh}");
                assert!(relief(&base) > 0.2, "{tag}: the fixture tile must have real relief");

                // Taken from the UN-ERODED tile and never rebuilt -- see
                // `incision_vs_area`'s own note on why that is the point.
                let net = network(&base, fw, fh, &tin.seed);
                let (s0, r0, n0) = incision_vs_area(&base, fw, fh, &net);
                let c0 = curvature(&base, fw, fh);

                let done = erode(&base, &place, &tin, &sp);
                let (s1, r1, _) = incision_vs_area(&done, fw, fh, &net);
                let c1 = curvature(&done, fw, fh);

                // The control: the identical call with the resolution correction
                // removed, reached through the kernel directly because
                // `tile_erode` will not run without it.
                let mut unscaled = base.clone();
                stream_power_kernel_bounded(
                    &mut unscaled,
                    &tin.stress,
                    &tin.resist,
                    &tin.rain,
                    fw,
                    fh,
                    &sp,
                    Some(&ring_mask(fw, fh)),
                    Some(&tin.seed),
                );
                let (_, rc, _) = incision_vs_area(&unscaled, fw, fh, &net);
                let cc = curvature(&unscaled, fw, fh);

                println!(
                    "{tag}: EF-0 r {r0:+.3} (slope {s0:+.5}, n {n0}) -> EF-3 r {r1:+.3} ({:+.3}, slope x{:.2}, curv x{:.2}, rms {:.3}% relief) | unscaled control r {rc:+.3} ({:+.3}, curv x{:.2})",
                    r1 - r0,
                    s1 / s0,
                    c1 / c0,
                    rms(&base, &done) / relief(&base) * 100.0,
                    rc - r0,
                    cc / c0
                );

                if dep == 0.0 {
                    assert!(
                        r1 - r0 >= 0.02,
                        "{tag}: with deposition off tile_erode must deepen the fixed network's own high-drainage ground; it moved only {:+.4} ({r0:+.4} -> {r1:+.4})",
                        r1 - r0
                    );
                    assert!(
                        r1 - rc >= 0.02,
                        "{tag}: the resolution correction must be what does it -- k x refine {r1:+.4} against the world's own k {rc:+.4}, a margin of only {:+.4}",
                        r1 - rc
                    );
                } else {
                    assert!(
                        r1 - r0 <= -0.02,
                        "{tag}: at the world's own deposit this measure FALLS (see the doc comment, and the world control above); it moved {:+.4} instead",
                        r1 - r0
                    );
                    assert!(
                        (rc - r0).abs() < 0.02,
                        "{tag}: without the resolution correction the pass is inert here, and it moved {:+.4}",
                        rc - r0
                    );
                    // The circular measure, on the same run, for the one
                    // purpose it still has: showing that it disagrees. Drainage
                    // recomputed on the eroded output, everything else equal.
                    let (_, r_circular, _) = incision_vs_area(&done, fw, fh, &network(&done, fw, fh, &tin.seed));
                    println!("{tag}: recomputed-on-the-output r {r_circular:+.3} ({:+.3}) -- the circular answer, for contrast", r_circular - r0);
                    assert!(
                        r_circular - r1 >= 0.05,
                        "{tag}: recomputing drainage on the eroded output reported {r_circular:+.4} against the fixed network's {r1:+.4}, a gap of only {:+.4} -- if these ever agree, re-read this test's doc comment before relaxing the bar",
                        r_circular - r1
                    );
                }
                assert!(c1 >= 0.85 * c0, "{tag}: erosion destroyed EF-0's detail -- curvature fell to {:.2}x", c1 / c0);
                assert!(c1 <= 4.0 * c0, "{tag}: curvature rose to {:.2}x, which is numerical roughness rather than terrain", c1 / c0);
                cases += 1;
            }
        }
    }
    // A loop that never ran passes every assertion inside it.
    assert_eq!(cases, TILES.len() * 2 * 2, "3 tiles x 2 refinements x 2 deposit settings");
}

// -- 2. tiling ---------------------------------------------------------------

/// **Property 2, and the seam test, in the shape EF-1's own boundary test uses:
/// measure the disagreement rather than assume it away.**
///
/// Two claims, and only the first is exact:
///
/// - **The ring is bit-identical**, so the step *across* a shared edge — the
///   last fine column of the left tile against the first of the right — is
///   exactly what EF-0 left there. No new seam. Asserted at `to_bits`.
/// - **The interior nearly is.** Erosion is not a local operation, so a cell
///   near the middle of a 16-coarse-cell tile genuinely does depend on ground
///   the neighbouring tile cannot see, and two tiles cannot reproduce one
///   larger tile exactly. What they can do is disagree by far less than the
///   detail they are carrying, and that is what is measured: mean and max
///   `|two tiles - one tile|` against the tile's own relief.
///   Measured on this fixture at `refine 8` over a `32x16` coarse footprint
///   split in two: mean `0.000027` (0.005% of relief `0.5654`), max
///   `0.004119` (0.73%), against bars of 0.05% and 2%. The lip the pinned
///   ring costs is measured in the same test: the worst ring-to-interior step
///   goes from `0.002086` to `0.002751`, i.e. it opens by 0.12% of relief.
///
/// The un-eroded control is asserted to be **exactly zero**, which is what says
/// the numbers above are erosion's doing and not the EF-0 region arithmetic's.
#[test]
fn two_tiles_agree_with_the_one_tile_that_covers_them() {
    let (ws, p) = world();
    let sp = light_params(&ws, &p);
    let refine = 8usize;
    let (x0, y0) = TILES[0];

    let big = place_at(x0, y0, 32, 16, refine);
    let bw = big.fine_w();
    let pre_big = ef0(&ws, &p, &big);
    let post_big = erode(&pre_big, &big, &tile_in(&ws, &big), &sp);

    let halves = [place_at(x0, y0, 16, 16, refine), place_at(x0 + 16, y0, 16, 16, refine)];
    let (hw, hh) = (halves[0].fine_w(), halves[0].fine_h());
    let (mut pre, mut post) = (Vec::new(), Vec::new());
    for pl in &halves {
        let b = ef0(&ws, &p, pl);
        post.push(erode(&b, pl, &tile_in(&ws, pl), &sp));
        pre.push(b);
    }

    let rel = relief(&pre_big);
    let (mut s1, mut m1, mut s0, mut m0, mut cnt) = (0.0f64, 0.0f64, 0.0f64, 0.0f64, 0usize);
    for y in 0..hh {
        for x in 0..hw {
            for (k, ox) in [(0usize, 0usize), (1, hw)] {
                s1 += (post[k][y * hw + x] as f64 - post_big[y * bw + x + ox] as f64).abs();
                m1 = m1.max((post[k][y * hw + x] as f64 - post_big[y * bw + x + ox] as f64).abs());
                s0 += (pre[k][y * hw + x] as f64 - pre_big[y * bw + x + ox] as f64).abs();
                m0 = m0.max((pre[k][y * hw + x] as f64 - pre_big[y * bw + x + ox] as f64).abs());
                cnt += 1;
            }
        }
    }
    assert_eq!(cnt, 2 * hw * hh, "the comparison must cover both halves");
    println!(
        "relief {rel:.4}: two tiles vs one, after erosion mean {:.6} ({:.4}% of relief) max {m1:.6} ({:.2}%); before erosion mean {s0:.6} max {m0:.6}",
        s1 / cnt as f64,
        s1 / cnt as f64 / rel * 100.0,
        m1 / rel * 100.0
    );
    assert_eq!(m0, 0.0, "the two halves must reproduce the big tile EXACTLY before erosion, or the region arithmetic is wrong and everything after it is noise");
    assert!(s1 / cnt as f64 / rel < 0.0005, "mean tiling disagreement {:.5}% of relief", s1 / cnt as f64 / rel * 100.0);
    assert!(m1 / rel < 0.02, "worst tiling disagreement {:.3}% of relief", m1 / rel * 100.0);

    // The seam itself, at exact equality: the ring came back untouched, so the
    // step across the shared edge is EF-0's own.
    let mut checked = 0usize;
    for y in 0..hh {
        assert_eq!(post[0][y * hw + hw - 1].to_bits(), pre[0][y * hw + hw - 1].to_bits(), "left tile's shared column moved at row {y}");
        assert_eq!(post[1][y * hw].to_bits(), pre[1][y * hw].to_bits(), "right tile's shared column moved at row {y}");
        checked += 1;
    }
    assert_eq!(checked, hh, "every row of the shared edge");

    // And the lip the pin costs: the first interior cell moved while the ring
    // did not, so the step between them changed. Bounded against the step EF-0
    // already had there, which is the only scale that means anything.
    let (mut lip, mut pre_step) = (0.0f64, 0.0f64);
    for x in 1..hw - 1 {
        pre_step = pre_step.max((pre[0][hw + x] as f64 - pre[0][x] as f64).abs());
        lip = lip.max((post[0][hw + x] as f64 - post[0][x] as f64).abs());
    }
    println!("ring lip: worst step ring-to-interior {lip:.6}, was {pre_step:.6} before erosion, relief {rel:.4}");
    assert!(lip - pre_step < 0.02 * rel, "the pinned ring opened a {:.3}% lip", (lip - pre_step) / rel * 100.0);
}

// -- 3. the inflow boundary condition ----------------------------------------

/// **Property 3.** The coarse world's own discharge reaches the tile's erosion,
/// and reaches it *where the water goes* rather than uniformly.
///
/// The seed is built here from an independent oracle — this file's own
/// [`recv`] over the coarse field, not `cartalith-hydrology`'s `d8_receiver` —
/// in EF-1's own shape: for each coarse cell in the one-cell ring **outside**
/// the tile, if its coarse D8 receiver is inside, its whole `flow_discharge`
/// crosses there, injected one fine cell in from the ring so the open boundary
/// does not swallow it. EF-1's module header is the authority on why it is the
/// ring's crossings and not the edge cells' own discharge.
///
/// Two bars:
///
/// - the seeded tile differs from the uniformly-seeded one at all (the boundary
///   condition is wired), and
/// - the difference is **concentrated at high drainage area**: mean `|Δ|` over
///   the top decile of drainage area must be at least `5x` the bottom decile's.
///   Inflow only raises `A` downstream of a crossing, so a difference spread
///   evenly over the tile would mean the seed was reaching the kernel as a
///   constant rather than as a boundary condition.
#[test]
fn the_coarse_inflow_boundary_condition_deepens_the_trunk() {
    let (ws, p) = world();
    let sp = light_params(&ws, &p);
    // Self-consistent with `ws.field` -- see the header on why this is not
    // `ws.flow_discharge`.
    let coarse_flow = compute_flow(GW, GH, &ws.field, None, false, false);

    let refine = 8usize;
    let (x0, y0) = TILES[0];
    let place = place_at(x0, y0, 16, 16, refine);
    let (fw, fh) = (place.fine_w(), place.fine_h());
    let base = ef0(&ws, &p, &place);
    let mut tin = tile_in(&ws, &place);
    let uniform = erode(&base, &place, &tin, &sp);

    // EF-1's boundary crossings, from this file's own oracle.
    let mut crossings = 0usize;
    let mut injected = 0.0f64;
    for gy in (y0 as i64 - 1)..=(y0 + place.rows) as i64 {
        for gx in (x0 as i64 - 1)..=(x0 + place.cols) as i64 {
            if gx < 0 || gy < 0 || gx >= GW as i64 || gy >= GH as i64 {
                continue;
            }
            let inside = |x: i64, y: i64| {
                x >= x0 as i64 && x < (x0 + place.cols) as i64 && y >= y0 as i64 && y < (y0 + place.rows) as i64
            };
            if inside(gx, gy) {
                continue;
            }
            let c = gy as usize * GW + gx as usize;
            let r = recv(&ws.field, GW, GH, c);
            if r < 0 {
                continue;
            }
            let (rx, ry) = (r % GW as i64, r / GW as i64);
            if !inside(rx, ry) || coarse_flow[c] <= 0.0 {
                continue;
            }
            // The receiver coarse cell's own fine block, centred, then stepped
            // off the open-boundary ring.
            let bx = ((rx - x0 as i64) as usize) * refine + refine / 2;
            let by = ((ry - y0 as i64) as usize) * refine + refine / 2;
            let f = by.clamp(1, fh - 2) * fw + bx.clamp(1, fw - 2);
            tin.seed[f] = (tin.seed[f] as f64 + coarse_flow[c] as f64) as f32;
            injected += coarse_flow[c] as f64;
            crossings += 1;
        }
    }
    assert!(crossings > 0, "no coarse crossing reaches this tile, so this test checks nothing");
    assert!(injected > 10.0, "only {injected} coarse cells of drainage enter -- too small to move anything");

    let seeded = erode(&base, &place, &tin, &sp);
    assert_ne!(uniform, seeded, "the coarse inflow must reach the output");

    // Where did it land? Split by the tile's own drainage area.
    let a = accum(&seeded, fw, fh);
    let mut by_area: Vec<(f32, f64)> =
        (0..fw * fh).filter(|&i| !on_ring(i, fw, fh)).map(|i| (a[i], (seeded[i] as f64 - uniform[i] as f64).abs())).collect();
    by_area.sort_by(|p, q| p.0.partial_cmp(&q.0).unwrap());
    let d = by_area.len() / 10;
    let lo: f64 = by_area[..d].iter().map(|v| v.1).sum::<f64>() / d as f64;
    let hi: f64 = by_area[by_area.len() - d..].iter().map(|v| v.1).sum::<f64>() / d as f64;
    println!(
        "{crossings} crossings carrying {injected:.0} coarse cells of drainage; mean |seeded-uniform| {hi:.3e} at the top decile of area vs {lo:.3e} at the bottom ({:.1}x)",
        hi / lo.max(1e-30)
    );
    assert!(hi > 5.0 * lo, "the inflow's effect is spread evenly ({hi:.3e} vs {lo:.3e}), so it is not acting as a boundary condition");
}

// -- 4. determinism ----------------------------------------------------------

/// **Property 4.** The whole EF-0 -> EF-3 composition is bit-identical across a
/// **regenerated** world, not just across two calls over one `Vec` — which is
/// what makes a cached tile safe to drop and re-derive (EF-5), and the property
/// `tile.rs`'s own determinism test cannot reach because it has no world.
#[test]
fn the_whole_composition_is_deterministic() {
    let refine = 8usize;
    let place = place_at(TILES[0].0, TILES[0].1, 16, 16, refine);
    let mut runs = Vec::new();
    for _ in 0..2 {
        let (ws, p) = world();
        let sp = light_params(&ws, &p);
        let base = ef0(&ws, &p, &place);
        runs.push((base.clone(), erode(&base, &place, &tile_in(&ws, &place), &sp)));
    }
    assert_eq!(
        runs[0].0.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
        runs[1].0.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
        "EF-0's own output must be reproducible, or the EF-3 comparison below means nothing"
    );
    assert_eq!(
        runs[0].1.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
        runs[1].1.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
        "the eroded tile must be bit-identical across a regenerated world"
    );
    assert_ne!(runs[0].0, runs[0].1, "a determinism test over a pass that did nothing proves nothing");
}

// -- 5. the region arithmetic ------------------------------------------------

/// [`ef0`]'s region is the one place two coordinate conventions meet, and
/// getting it wrong shifts every tile by a fraction of a coarse cell — silently,
/// since the result still looks like terrain. Checked against
/// `TilePlacement::fine_to_coarse` rather than against [`ef0`]'s own comment.
#[test]
fn the_ef0_region_lands_on_the_placements_own_fine_cells() {
    for refine in [1usize, 2, 4, 8] {
        let place = place_at(TILES[0].0, TILES[0].1, COLS, COLS, refine);
        let (fw, _) = (place.fine_w(), place.fine_h());
        let r = refine as f64;
        let (rx, rw) = (place.x0 as f64 - 0.5 + 0.5 / r, place.cols as f64 + 1.0 - 1.0 / r);
        for ox in [0usize, 1, fw / 2, fw - 1] {
            // `amplify_region`'s own mapping ...
            let got = rx + (ox as f64 / (fw as f64 - 1.0)) * (rw - 1.0);
            // ... against the placement's, converted from pixel to cell-centre
            // coordinates the way `cartalith_hydrology::tile`'s header says.
            let (want, _) = place.fine_to_coarse(ox as f64 + 0.5, 0.0);
            assert!((got - (want - 0.5)).abs() < 1e-9, "refine {refine}, fine column {ox}: {got} vs {}", want - 0.5);
        }
    }
}

// -- 6. cost -----------------------------------------------------------------

/// **The number that decides whether EF-3 is usable at interactive zoom.**
///
/// `#[ignore]`d on purpose: `MISTAKES.md`'s rule for quoting a timing is to run
/// the harness **alone**, never under a parallel suite, and a timing assertion
/// inside `cargo test --workspace` is a flake generator. Run it with
///
/// ```text
/// cargo test --release -p cartalith-engine --test ef3_tile_erosion \
///     -- --ignored --nocapture --test-threads=1 measure_tile_erosion_cost
/// ```
///
/// Median of nine with min..max, on this machine (16 logical cores, release):
/// see the printout. What the numbers looked like when this was written is in
/// `cartalith_erosion::tile`'s module documentation, next to the decision they
/// informed; they are not repeated here, because a figure in two places is a
/// figure that will disagree with itself.
#[test]
#[ignore = "timing harness -- run alone, see the doc comment"]
fn measure_tile_erosion_cost() {
    let (ws, p) = world();
    let sp = light_params(&ws, &p);
    println!("{} logical cores; world light pass iters {}", std::thread::available_parallelism().map(|v| v.get()).unwrap_or(0), sp.iters);
    for (cols, refine) in [(16usize, 8usize), (32, 8), (16, 16), (32, 16)] {
        let place = place_at(TILES[0].0.min(GW - cols), TILES[0].1.min(GH - cols), cols, cols, refine);
        let (fw, fh) = (place.fine_w(), place.fine_h());
        let tin = tile_in(&ws, &place);
        let (mut a, mut b) = (Vec::new(), Vec::new());
        for _ in 0..9 {
            let s = std::time::Instant::now();
            let base = ef0(&ws, &p, &place);
            a.push(s.elapsed().as_secs_f64() * 1000.0);
            let s = std::time::Instant::now();
            let _ = erode(&base, &place, &tin, &sp);
            b.push(s.elapsed().as_secs_f64() * 1000.0);
        }
        a.sort_by(|x, y| x.partial_cmp(y).unwrap());
        b.sort_by(|x, y| x.partial_cmp(y).unwrap());
        // Where the time goes: the priority-flood fill and the
        // multiple-flow-direction drainage-area pass run once, before the
        // first iteration. `iters: 0` isolates them, and the difference is
        // what one more iteration would actually cost -- the number that
        // decides whether `iters x refine` was affordable.
        let base = ef0(&ws, &p, &place);
        let mut setup = Vec::new();
        for _ in 0..9 {
            let s = std::time::Instant::now();
            let _ = erode(&base, &place, &tin, &StreamPowerParams { iters: 0, ..sp });
            setup.push(s.elapsed().as_secs_f64() * 1000.0);
        }
        setup.sort_by(|x, y| x.partial_cmp(y).unwrap());
        println!(
            "{fw}x{fh} fine ({cols}x{cols} coarse @ refine {refine}): EF-0 {:.2} ms ({:.2}..{:.2}), EF-3 {:.2} ms ({:.2}..{:.2}), total {:.2} ms  [of EF-3: setup {:.2} ms, {} iterations {:.2} ms = {:.3} ms each]",
            a[4], a[0], a[8], b[4], b[0], b[8], a[4] + b[4],
            setup[4], sp.iters, b[4] - setup[4], (b[4] - setup[4]) / sp.iters as f64
        );
    }
}
