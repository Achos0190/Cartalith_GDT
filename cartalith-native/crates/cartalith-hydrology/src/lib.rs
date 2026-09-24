//! flow accumulation, river network, channel width
//!
//! Ported in pipeline order starting Phase 1 (MVP_SCOPE.md).

pub mod tile;

use rayon::prelude::*;

// `js_atan2` -- `Math.atan2` as V8 computes it, which is FDLIBM's
// `__ieee754_atan2` and not the platform libm `f64::atan2` reaches. It was
// written here because this is where the live bug was (`build_channels` below
// was picking the wrong receiver cell), and `JS_SEMANTICS_AUDIT.md` §5 recorded
// at the time that a private module here was an *eighth* copy site for the
// FDLIBM family and the wrong steady state. It now lives in
// `cartalith-jsmath`, where `cartalith-terrain` and `cartalith-urban` -- which
// both needed `atan2` and neither of which could see this module -- reach it
// too. The two `node`-derived goldens moved with it, unchanged, which is the
// check that the move was pure.
use cartalith_jsmath::{js_atan2, js_hypot, js_min};

/// Descending-height, ascending-index-on-tie comparison — the ordering
/// [`flow_sort_desc`] must produce. **The oracle, not the implementation**:
/// this is what `compute_flow` used to sort by directly, kept as the
/// reference definition the radix sort is tested against
/// (`flow_sort_desc_is_element_identical_to_the_comparison_sort`).
///
/// JS explicitly normalizes `-0.0`'s sort key to match `+0.0`
/// (`if(b===0x80000000) b=0`), which `f32::total_cmp` does not do on its
/// own (`total_cmp` treats `-0.0 < +0.0`) — normalized here before
/// comparing.
#[cfg(test)]
fn flow_cmp_desc(a: f32, b: f32) -> std::cmp::Ordering {
    let na = if a == 0.0 { 0.0f32 } else { a };
    let nb = if b == 0.0 { 0.0f32 } else { b };
    nb.total_cmp(&na)
}

/// `_flowRadixSortDesc()` (reference HTML lines 4846-4861): a stable LSD
/// radix sort over the raw `f32` bit patterns, producing cell indices in
/// descending height with ties in ascending index.
///
/// **Why this is a legitimate substitution at all.** Flow accumulation is
/// downstream of the heightmap pixels, so per `PROVENANCE.md` only the
/// *ordering guarantee* is part of the parity contract, not the sort
/// algorithm — the reference's own comment says the same thing from the
/// other side, calling its radix sort "BIT-IDENTICAL to the old
/// `order.sort((a,b)=>field[b]-field[a])` … by construction." This port
/// carried the comparison form until now; the reference replaced it in
/// v0.148 after measuring the comparator sort as *"the single hottest
/// `generate()` line"* (~1,005 ms per call at 2048², 1,005 → 120 ms).
///
/// **The key transform, verbatim from the reference.** Three steps on the
/// `u32` bit pattern:
/// 1. `if b == 0x8000_0000 { b = 0 }` — canonicalise `-0.0` to `+0.0`.
///    Without this one line the radix would order them deterministically by
///    sign and split a tie the comparator treats as equal.
/// 2. sign-flip so ascending `u32` means ascending `f32`: negatives get
///    `!b`, non-negatives get `b | 0x8000_0000`. (This is the same total
///    order `f32::total_cmp` implements, NaNs included — checked, not
///    assumed: `+NaN` maps above `+inf` and `-NaN` below `-inf` under both.)
/// 3. `!b` — invert, so ascending `u32` now means **descending** `f32`.
///
/// **Stability is load-bearing, and it is a property of the construction,
/// not an accident.** Counting sort per byte is stable, and the initial
/// permutation is ascending index, so equal keys come out in ascending
/// index order — matching JS's spec-stable `Array#sort`. Tie order is not
/// cosmetic: equal-height cells draining into one receiver add their
/// `f32` discharge in this order, and float addition rounding depends on
/// it (`cartalith-rust-conventions`: do not reorder float operations).
///
/// Four 8-bit passes end back in the buffer they started in, so the result
/// is `src` after the last swap, not a fixed one of the two.
fn flow_sort_desc(field: &[f32], n: usize) -> Vec<u32> {
    assert!(
        n <= u32::MAX as usize,
        "flow_sort_desc indexes cells with u32; {n} cells is beyond that (a >4-billion-cell grid is not a shape this engine ships)"
    );
    let mut keys = vec![0u32; n];
    for (key, &v) in keys.iter_mut().zip(&field[..n]) {
        let mut b = v.to_bits();
        if b == 0x8000_0000 {
            b = 0;
        }
        b = if b & 0x8000_0000 != 0 { !b } else { b | 0x8000_0000 };
        *key = !b;
    }

    let mut src: Vec<u32> = (0..n as u32).collect();
    let mut dst = vec![0u32; n];
    let mut cnt = [0u32; 256];
    for shift in [0u32, 8, 16, 24] {
        cnt.fill(0);
        for &k in &keys {
            cnt[((k >> shift) & 255) as usize] += 1;
        }
        let mut sum = 0u32;
        for c in cnt.iter_mut() {
            let here = *c;
            *c = sum;
            sum += here;
        }
        for &id in &src {
            let bucket = &mut cnt[((keys[id as usize] >> shift) & 255) as usize];
            dst[*bucket as usize] = id;
            *bucket += 1;
        }
        std::mem::swap(&mut src, &mut dst);
    }
    src
}

/// `computeFlow()` (reference HTML lines 4862-4890): D8 steepest-descent
/// flow accumulation. Processes cells in descending-height order so that
/// by the time a cell is visited, every upstream contribution has already
/// accumulated into it (classic drainage-order trick) — each cell then
/// pushes its full accumulated flow to its single steepest downhill
/// neighbor.
///
/// `useRain=true` seeds discharge from rainfall (mean-normalized, floor
/// `0.05`) rather than bare cell count — rivers accumulate runoff, not
/// area (Whipple & Tucker 1999). `useRain=false` (the `acc.fill(1)`
/// path) is the area-only seeding `computeFlow()`'s first call in
/// `generate()` uses, before climate exists yet.
///
/// `acc[best]+=acc[i]` is the same "multiple cells can write to one
/// target within a pass" trap `erode_thermal`'s `delta[j]+=` needed —
/// a downhill cell can receive accumulated flow from several different
/// upstream cells, each JS write rounding to `f32` individually. Kept as
/// per-write rounding here too, not an `f64` accumulator.
///
/// NOT parallelized (`CPU_MULTITHREADING_SCOPE.md`): the descending-order
/// accumulation loop below is exactly the flow-accumulation hazard this
/// project's own scope docs already named — each cell scatters into its
/// single downstream receiver in strict descending-height order, a genuine
/// wavefront dependency, confirmed here rather than assumed. `sm` is also
/// a running sum (not a max), so left sequential for the same
/// floating-point-reordering reason `stream_power_kernel::ss` is.
pub fn compute_flow(gw: usize, gh: usize, field: &[f32], rain: Option<&[f32]>, use_rain: bool, world: bool) -> Vec<f32> {
    let n = gw * gh;
    let order = flow_sort_desc(field, n);

    let mut acc = vec![0f32; n];
    if use_rain {
        let rain = rain.expect("rain field required when use_rain is true");
        let mut sm = 0.0f64;
        for i in 0..n {
            let r = (rain[i] as f64).max(0.05);
            acc[i] = r as f32;
            sm += r;
        }
        let k = n as f64 / sm.max(1e-6);
        // Per-cell rescale, independent -- safe.
        acc.par_iter_mut().for_each(|v| *v = (*v as f64 * k) as f32);
    } else {
        acc.fill(1.0);
    }

    let d8 = d8_table();

    for &i in &order {
        let i = i as usize;
        let best = d8_receiver(field, gw, gh, i, world, &d8);
        if best >= 0 {
            let best = best as usize;
            acc[best] = (acc[best] as f64 + acc[i] as f64) as f32;
        }
    }

    acc
}

/// The depression-filled **routing surface** that integrated drainage routes
/// over (`RC_ENGINE_CHANGES.md` §6g, the source's `buildRoutingSurface`,
/// v2.41; default-on in the source from v2.59, §6k).
///
/// **Not a line-by-line port.** The source's JavaScript postdates every
/// reference snapshot in this repository (v2.10/v2.11), so there is nothing to
/// diff against. What §6g specifies — and what this implements — is the
/// standard published algorithm it names: Barnes, Lehman & Mulla (2014),
/// *Priority-Flood: An Optimal Depression-Filling and Watershed-Labeling
/// Algorithm for Digital Elevation Models*, Algorithm 3, "Priority-Flood+ε",
/// in the shape of Barnes' own RichDEM implementation (`PriorityFloodEpsilon`,
/// including its equal-elevation check that pops the open queue before the pit
/// queue). The spec's disclosed before/after numbers are the validation
/// target, not the algorithm.
///
/// - **Seeds**: every sub-sea cell (`field < sea`) and every map edge — the
///   `y` edges always, the `x` edges only when not wrapping, since a
///   cylindrical world has no east/west boundary for water to leave by (the
///   same `x`-wraps/`y`-does-not rule [`d8_receiver`] applies). Seeds keep
///   their own height.
/// - **The ε tilt is the whole fix** (§6g: "a fill without the tilt is not a
///   fix"). A plain fill leaves a basin flat, and [`d8_receiver`] requires a
///   *strictly* positive drop, so a flat terminates accumulation exactly as
///   the pit did. Each raised cell takes `next_up` of the cell it was reached
///   from — the smallest representable `f32` step, which is Barnes' own
///   `nextafter` — so every non-seed cell ends with a neighbour strictly below
///   it and every land cell drains, by a strictly descending chain, to a seed.
///   At `f32` in `[0.25, 1)` one step is at most `5.96e-8` of the normalised
///   range: a flat 1 000 cells across rises by `6e-5`, i.e. 0.24 m at the
///   default 4 000 m peak.
/// - **It never modifies the heightmap.** The return value is a separate grid;
///   the terrain keeps its pits, so the water-body classifier still reads the
///   real surface and a lake stays a lake, while routing sees it filled and a
///   river flows *through* it to its outflow. Cells outside every depression
///   come back bit-identical to `field`.
///
/// Deterministic: the open queue orders by `(height, index)`, height taken
/// under the same total-order bit transform [`flow_sort_desc`] uses, so equal
/// heights pop in ascending index whatever the heap's internal layout.
///
/// Seeds whose eight neighbours are all seeds are never pushed — popping one
/// could only visit closed cells — so the heap holds the coastline and the map
/// edge, not the ocean. `O(n log n)` worst case.
pub fn build_routing_surface(field: &[f32], gw: usize, gh: usize, sea: f64, world: bool) -> Vec<f32> {
    use std::cmp::Reverse;
    use std::collections::{BinaryHeap, VecDeque};

    let n = gw * gh;
    let mut surf = field[..n].to_vec();
    if n == 0 {
        return surf;
    }
    let key = |v: f32| -> u32 {
        let mut b = v.to_bits();
        if b == 0x8000_0000 {
            b = 0;
        }
        if b & 0x8000_0000 != 0 { !b } else { b | 0x8000_0000 }
    };
    // Up to eight neighbours of `i`, `x` wrapping under `world`; returns the count.
    let neighbours = |i: usize, out: &mut [usize; 8]| -> usize {
        let (x, y) = ((i % gw) as i64, (i / gw) as i64);
        let mut k = 0;
        for dy in -1i64..=1 {
            let ny = y + dy;
            if ny < 0 || ny >= gh as i64 {
                continue;
            }
            for dx in -1i64..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let mut nx = x + dx;
                if world {
                    nx = nx.rem_euclid(gw as i64);
                } else if nx < 0 || nx >= gw as i64 {
                    continue;
                }
                let j = (ny * gw as i64 + nx) as usize;
                // A 1- or 2-wide wrapped grid reaches a cell through both
                // sides; visiting it twice is harmless, but never itself.
                if j != i {
                    out[k] = j;
                    k += 1;
                }
            }
        }
        k
    };

    let mut closed = vec![false; n];
    for (i, c) in closed.iter_mut().enumerate() {
        let (x, y) = (i % gw, i / gw);
        *c = (field[i] as f64) < sea || y == 0 || y + 1 == gh || (!world && (x == 0 || x + 1 == gw));
    }
    let mut nb = [0usize; 8];
    let mut open: BinaryHeap<Reverse<(u32, u32)>> = BinaryHeap::new();
    for i in 0..n {
        if closed[i] {
            let k = neighbours(i, &mut nb);
            if nb[..k].iter().any(|&j| !closed[j]) {
                open.push(Reverse((key(surf[i]), i as u32)));
            }
        }
    }

    let mut pit: VecDeque<u32> = VecDeque::new();
    loop {
        let c = match (pit.front().copied(), open.peek().copied()) {
            // RichDEM's tie rule: an open cell at exactly the pit front's
            // height goes first, so a flat that meets unraised ground of the
            // same height drains into it rather than being raised over it.
            (Some(p), Some(Reverse((k, o)))) if k == key(surf[p as usize]) => {
                open.pop();
                o
            }
            (Some(p), _) => {
                pit.pop_front();
                p
            }
            (None, Some(Reverse((_, o)))) => {
                open.pop();
                o
            }
            (None, None) => break,
        } as usize;
        let up = surf[c].next_up();
        let k = neighbours(c, &mut nb);
        for &j in &nb[..k] {
            if closed[j] {
                continue;
            }
            closed[j] = true;
            if surf[j] <= up {
                surf[j] = up;
                pit.push_back(j as u32);
            } else {
                open.push(Reverse((key(surf[j]), j as u32)));
            }
        }
    }
    surf
}

/// The routing surface to use for a world: [`build_routing_surface`] when
/// `integrate` is on, `field` itself when it is off — borrowed, so the
/// off-path costs nothing and routes on the raw field exactly as before.
pub fn routing_view<'a>(
    field: &'a [f32],
    gw: usize,
    gh: usize,
    sea: f64,
    world: bool,
    integrate: bool,
) -> std::borrow::Cow<'a, [f32]> {
    if integrate {
        std::borrow::Cow::Owned(build_routing_surface(field, gw, gh, sea, world))
    } else {
        std::borrow::Cow::Borrowed(field)
    }
}

/// [`compute_flow`] over [`routing_view`]: integrated drainage when
/// `integrate`, the raw-field accumulation — the same call, bit for bit — when
/// not. `compute_flow` reads its height argument only to order cells and pick
/// each one's receiver, so routing is purely a matter of which surface it is
/// handed; the discharge seeding (`rain`) is unaffected.
#[allow(clippy::too_many_arguments)]
pub fn compute_flow_routed(
    gw: usize,
    gh: usize,
    field: &[f32],
    rain: Option<&[f32]>,
    use_rain: bool,
    world: bool,
    sea: f64,
    integrate: bool,
) -> Vec<f32> {
    compute_flow(gw, gh, &routing_view(field, gw, gh, sea, world, integrate), rain, use_rain, world)
}

/// Every cell's single D8 receiver over `field` — the tree [`compute_flow`]
/// accumulates along, `-1` where the cell has no strictly lower neighbour.
/// Exposed for measurement: walking it to each chain's terminus is how §6g
/// says integrated drainage must be verified ("do not verify this with a flow
/// ratio").
pub fn flow_receivers(field: &[f32], gw: usize, gh: usize, world: bool) -> Vec<i32> {
    let d8 = d8_table();
    (0..gw * gh).map(|i| d8_receiver(field, gw, gh, i, world, &d8) as i32).collect()
}

/// `D8[(dy+1)*3+(dx+1)] = hypot(dx, dy)` for `dx,dy` in `{-1,0,1}`; center
/// (index 4) is unused ([`d8_receiver`] always skips `dx=dy=0`) but kept for a
/// direct match to the reference's own indexing scheme.
///
/// Built by the same loop the reference writes rather than pinned as nine
/// literals — the nine values are bit-identical under `Math.hypot` and
/// `f64::hypot` either way (`slope_hypot_divergence_is_measured_not_assumed`
/// asserts exactly that), and a literal table would be a second definition to
/// keep in step for no gain.
pub(crate) fn d8_table() -> [f64; 9] {
    let mut d8 = [0f64; 9];
    for dy in -1i32..=1 {
        for dx in -1i32..=1 {
            d8[((dy + 1) * 3 + (dx + 1)) as usize] = (dx as f64).hypot(dy as f64);
        }
    }
    d8
}

/// The single steepest-descent D8 receiver of cell `i`, or `-1` where the cell
/// is a pit (no neighbour strictly below it). `(h - h_n) / d8_dist`, first
/// strictly-greater wins, scanned `dy` then `dx` ascending — the reference's
/// own order, so a tie between two equal drops goes to the earlier offset.
///
/// **Extracted from [`compute_flow`]'s own inner loop, unchanged**, so that
/// [`tile::tile_flow`]'s tile-bounded accumulation resolves flow direction by
/// the *identical* rule the world pass used rather than a second
/// implementation that could drift from it. The arithmetic is the same
/// expression in the same order (`cartalith-rust-conventions`: float
/// operations are not reordered), and the three `compute_flow` golden cases
/// are what checks that.
///
/// `world` wraps `x` only, matching the reference — `y` never wraps, because a
/// cylindrical world has poles, not a torus.
#[inline]
pub(crate) fn d8_receiver(field: &[f32], gw: usize, gh: usize, i: usize, world: bool, d8: &[f64; 9]) -> i64 {
    let x = (i % gw) as i64;
    let y = (i / gw) as i64;
    let h = field[i] as f64;
    let mut best: i64 = -1;
    let mut best_drop = 0.0f64;
    for dy in -1i64..=1 {
        for dx in -1i64..=1 {
            if dx == 0 && dy == 0 {
                continue;
            }
            let mut nx = x + dx;
            let ny = y + dy;
            if world {
                nx = ((nx % gw as i64) + gw as i64) % gw as i64;
            } else if nx < 0 || nx >= gw as i64 {
                continue;
            }
            if ny < 0 || ny >= gh as i64 {
                continue;
            }
            let j = ny * gw as i64 + nx;
            let drop = (h - field[j as usize] as f64) / d8[((dy + 1) * 3 + (dx + 1)) as usize];
            if drop > best_drop {
                best_drop = drop;
                best = j;
            }
        }
    }
    best
}

/// Slope-dependent multiplier on the channel-initiation threshold
/// (reference HTML line 4549) — steep ground channelizes with less
/// accumulated area.
const RIVER_SLOPE_K: f64 = 8.0;

/// `riverFlowThresh()` (reference HTML line 4493): the canonical
/// channel-initiation threshold, replacing ~14 independently
/// re-implemented copies the reference's own history found drifting.
/// `world_gw`/`map_width_km` are the *world's own* grid width and real
/// km extent — deliberately separate from `gw`/`gh` (the grid actually
/// being classified), since an LOD tile's threshold must stay anchored
/// to the real world's detail level, not a tile-local guess. For the
/// MVP path (no tiled LOD), `world_gw` is always the same value as `gw`.
pub fn river_flow_thresh(gw: usize, gh: usize, world_gw: usize, map_width_km: f64) -> f64 {
    (gw * gh) as f64 * 0.0004
        / (cartalith_terrain::terrain_detail_k(world_gw, map_width_km) * cartalith_terrain::river_coarse_ease(map_width_km))
}

/// `channelThreshold()` (reference HTML lines 4550-4554): scales the
/// base threshold by slope (steeper ⇒ lower threshold) and by
/// `riverDensity` (a density≠1 also re-shapes the slope response via
/// `dexp`, not just a flat rescale).
fn channel_threshold(base_thresh: f64, slope_n: f64, density: f64) -> f64 {
    let density = if density > 0.0 { density } else { 1.0 };
    let dexp = density.ln().abs();
    (base_thresh / density) * (1.0 + RIVER_SLOPE_K * slope_n).powf(-dexp)
}

/// Output of `build_channels` — the channel mask, single-receiver tree,
/// and slope field `buildRiverNetwork`'s channelization loop produces
/// (reference HTML lines 4503-4522), bundled since they're computed
/// together in one pass.
pub struct ChannelResult {
    pub recv: Vec<i32>,
    pub chan: Vec<u8>,
    pub slope: Vec<f32>,
    /// The stamped disc raster from [`stamp_river_intensity`], or empty.
    ///
    /// **Empty as `build_channels` returns it**, and filled by the caller
    /// afterwards: the stamp needs Strahler order, which is
    /// `strahler_from_receivers`' output and therefore not available until
    /// after this function has returned its `recv`. `generate_terrain` fills
    /// it two statements later. A consumer must treat an empty vector as "no
    /// stamp on this world". A project archive stores the stamp beside the
    /// channel topology since 2026-09-24 (`SAVEFILE_COMPAT.md` §8.3), and a
    /// reopened project gets it back, empty if it was saved empty.
    pub intensity: Vec<f32>,
}

/// `buildRiverNetwork()`'s channelization loop (reference HTML lines
/// 4503-4522) — **not the whole function**. This covers the network's
/// *topology*: which cells channelize (slope-area threshold,
/// `channel_threshold`) and each channel cell's single downstream
/// receiver, picked via a D∞-style continuous-aspect projection
/// (Tarboton 1997) rather than raw steepest-D8-of-8, to avoid the
/// 45°/90° staircase bias a pure D8 receiver tree would carry into the
/// traced polylines. Falls back to steepest-descent when no neighbor is
/// well-aligned with the true gradient aspect.
///
/// Width/depth/intensity stamping and polyline tracing (the rest of
/// `buildRiverNetwork`) are deferred — this piece was ported first
/// because it's what `MVP_SCOPE.md`'s own "Strahler ordering" bullet
/// names, and because `strahler_from_receivers` needs exactly this
/// output (`recv`/`chan`) and nothing more.
///
/// **The aspect chain uses [`js_atan2`], not `f64::atan2`, and that is
/// load-bearing.** `best` here is a discrete argmax — the cell a river
/// flows into — so a one-ulp difference in the steering weight is not
/// absorbed by a later `f32` store the way most of this workspace's
/// libm divergences are (`JS_SEMANTICS_AUDIT.md` §4.2). It changes which
/// cell the river takes, and everything downstream of that cell moves.
///
/// The reachable case is narrow but structural, not accidental: when a
/// cell's 3x3 is left-right symmetric, `gx` is exactly `0.0`, `aspect`
/// comes out at exactly `-pi/2` off the signed-zero branch, and the two
/// symmetric downhill diagonals get **exactly equal** `drop` and
/// mathematically equal `da`. The argmax is then settled by which of two
/// last bits is larger, and `f64::atan2` settles it differently from V8.
/// Measured over 1 200 000 randomly generated 3x3 blocks on a quantised
/// height lattice, `f64::atan2` picks a different receiver from V8 on 84;
/// `js_atan2` picks V8's on all 1 200 000. See
/// `build_channels_receiver_follows_v8_not_rust_atan2`.
///
/// `sin`/`cos` diverge from V8 too (2.34 % each) and are **not** ported
/// here, because measurement says they cannot reach this argmax: the wrap
/// `js_atan2(sin(da), cos(da))` only decides the outcome when the two
/// competing `da` are exact negatives of each other, and `sin`/`cos`
/// preserve that antisymmetry exactly whatever their accuracy. Over
/// 600 000 blocks spanning four terrain regimes, `js_atan2` with Rust's
/// own `sin`/`cos` agreed with V8 on every single receiver.
#[allow(clippy::too_many_arguments)]
pub fn build_channels(
    fld: &[f32],
    flow: &[f32],
    w: usize,
    h: usize,
    sea: f64,
    world: bool,
    river_density: f64,
    map_width_km: f64,
) -> ChannelResult {
    build_channels_with_threshold(fld, flow, w, h, sea, world, river_density, w, river_flow_thresh(w, h, w, map_width_km))
}

/// [`build_channels`] with the channel-initiation threshold and the slope
/// normalisation width supplied rather than derived from the grid being
/// classified.
///
/// **Why this exists, and why it is not a convenience.** `build_channels`
/// computes `river_flow_thresh(w, h, w, map_width_km)` — `0.04 %` of *its
/// own* grid's cell count. That is right for the world pass, where the grid
/// being classified *is* the world, and wrong for a tile: a 64×64 tile cut
/// out of a 256×256 world would set its threshold from 4 096 cells instead of
/// 65 536 and call a hillslope a river 16× too eagerly.
/// [`tile::tile_channel_thresh`] is the world-anchored number a tile wants,
/// and this entry point is how it reaches the classifier. `river_flow_thresh`'s
/// existing `world_gw` argument cannot express it — that one anchors
/// `terrain_detail_k` only, leaving the `(gw*gh)` area term tile-sized.
///
/// **`slope_w` is the second half of the same problem.** `slope_n` below is
/// `hypot(gx, gy) * w` — a *rise across the whole map*, and therefore
/// resolution-independent, only because the reference's `w` is the world's own
/// grid width. Hand the function a tile and both halves shrink: the gradient
/// is per *fine* cell and the multiplier is the *tile's* width, so a tile of
/// `cols` coarse columns reports `cols/gw` of the world's `slope_n` for the
/// same physical slope. `slope_w` is what that multiplier should be —
/// `gw * refine` for a tile, the world's own width expressed at the tile's
/// resolution, which is the same anchoring [`tile::tile_channel_thresh`]
/// applies to the threshold. It reaches `channel_threshold`'s
/// `(1 + 8*slope_n)^(-|ln density|)` factor, so at the default
/// `river_density == 1` it cannot change an outcome (the exponent is `0`) and
/// away from `1` it decides whether a steep tile channelizes as readily as the
/// same ground does at world scale.
///
/// `build_channels` itself is unchanged: it computes the same threshold from
/// the same expression, passes its own `w` as `slope_w`, and hands both here.
/// All three `golden_parity_river` cases assert `slope` cell for cell against
/// the JS reference, so any change to what the wrapper passes fails there
/// immediately; `build_channels_case_2`'s `river_density = 2.0` is
/// additionally the one fixture where `slope_w` reaches `chan` as well.
#[allow(clippy::too_many_arguments)]
pub fn build_channels_with_threshold(
    fld: &[f32],
    flow: &[f32],
    w: usize,
    h: usize,
    sea: f64,
    world: bool,
    river_density: f64,
    slope_w: usize,
    thresh: f64,
) -> ChannelResult {
    build_channels_core(fld, fld, flow, w, h, sea, world, river_density, slope_w, thresh)
}

/// [`build_channels`] with the receiver tree built over a separate routing
/// surface — integrated drainage's half of the channel network
/// (`RC_ENGINE_CHANGES.md` §6g: "both trees need the same surface").
///
/// `fld` is the **real** terrain and still decides everything that is a
/// statement about the ground: the sub-sea skip, and the gradient — so the
/// slope field and the channel-initiation threshold it feeds are unchanged,
/// which is §6g's one named exception. `route` (normally
/// [`build_routing_surface`] of `fld`) decides only which neighbours are
/// downhill: every `drop` in the receiver search is measured on it. So the
/// aspect that steers the pick is the real ground's, choosing among neighbours
/// that are strictly lower *on the routing surface* — which can never make a
/// cycle or a pit, because every candidate strictly descends there.
///
/// **Except inside a filled depression** (`route[i] > fld[i]`), where a cell
/// takes the plain steepest-descent receiver on `route` — the very tree
/// [`compute_flow`] accumulated along. There the real gradient points at the
/// basin floor rather than the outlet, and every drop is the ε tilt (a few
/// `f32` steps), so aspect steering picks a neighbour off the accumulation path
/// whose discharge is below the channel threshold, and the channel breaks
/// mid-lake. Following the accumulation tree cannot break: the receiver's
/// discharge is at least the donor's. Measured on four generated worlds
/// (`cartalith-civ/tests/integrated_drainage.rs`): steering by the real
/// aspect everywhere left 579 / 1 398 / 4 175 / 2 875 channel mouths ending
/// on dry, non-channel land against 254 / 221 / 361 / 2 197 without the fill,
/// and the longest main stem only ×1.22 / ×1.10 / ×1.19 / ×1.20; steepest
/// inside filled cells gives 285 / 382 / 856 / 2 097 and ×1.22 / ×1.86 /
/// ×2.83 / ×2.00 (§6k's own figure is ×1.80). Steering by the ROUTED aspect
/// instead was measured too (406 / 755 / 2 379 / 2 412 alone; within 1 % of
/// this rule when combined with it) and is not used. The source's own choice
/// here is not recoverable from §6g; this one is chosen by that measurement.
///
/// With `route == fld` no cell is filled and this is `build_channels`
/// exactly.
#[allow(clippy::too_many_arguments)]
pub fn build_channels_routed(
    fld: &[f32],
    route: &[f32],
    flow: &[f32],
    w: usize,
    h: usize,
    sea: f64,
    world: bool,
    river_density: f64,
    map_width_km: f64,
) -> ChannelResult {
    build_channels_core(fld, route, flow, w, h, sea, world, river_density, w, river_flow_thresh(w, h, w, map_width_km))
}

#[allow(clippy::too_many_arguments)]
fn build_channels_core(
    fld: &[f32],
    route: &[f32],
    flow: &[f32],
    w: usize,
    h: usize,
    sea: f64,
    world: bool,
    river_density: f64,
    slope_w: usize,
    thresh: f64,
) -> ChannelResult {
    let wrap = world;
    let n = w * h;
    let density = if river_density > 0.0 { river_density } else { 1.0 };

    let mut recv = vec![-1i32; n];
    let mut chan = vec![0u8; n];
    let mut slope = vec![0f32; n];

    let mut d8 = [0f64; 9];
    for dy in -1i32..=1 {
        for dx in -1i32..=1 {
            d8[((dy + 1) * 3 + (dx + 1)) as usize] = (dx as f64).hypot(dy as f64);
        }
    }

    // Per-cell: writes only `slope[i]`/`chan[i]`/`recv[i]`, reads only a
    // fixed 3x3 neighbourhood of the frozen `fld`/`flow` inputs -- no
    // cross-cell write, no dependency on any other output cell. Unlike
    // `compute_flow` above (a real downstream-accumulation scatter), this
    // is genuinely independent -- the one real win in this crate.
    recv.par_chunks_mut(w)
        .zip(chan.par_chunks_mut(w))
        .zip(slope.par_chunks_mut(w))
        .enumerate()
        .for_each(|(y, ((recv_row, chan_row), slope_row))| {
            for x in 0..w {
                let i = y * w + x;
                if (fld[i] as f64) < sea {
                    continue;
                }
                let xl = if wrap {
                    (x + w - 1) % w
                } else if x > 0 {
                    x - 1
                } else {
                    x
                };
                let xr = if wrap {
                    (x + 1) % w
                } else if x < w - 1 {
                    x + 1
                } else {
                    x
                };
                let gx = (fld[y * w + xr] as f64 - fld[y * w + xl] as f64) * 0.5;
                let above = if y < h - 1 { fld[(y + 1) * w + x] as f64 } else { fld[i] as f64 };
                let below = if y > 0 { fld[(y - 1) * w + x] as f64 } else { fld[i] as f64 };
                let gy = (above - below) * 0.5;
                // `Math.hypot(gx, gy) * W`, not `f64::hypot` -- the reference
                // writes `Math.hypot` here (line 4507; 4506 is the gx/gy line
                // above it, which is what this comment used to cite) and this
                // port did not.
                // `slope_n` goes straight into `channel_threshold`, whose
                // result is compared `flow[i] <=`, so a one-ulp difference
                // could flip whether the cell channelizes at all.
                //
                // **Measured, and the measurement is smaller than the worry**
                // (`slope_hypot_divergence_is_measured_not_assumed`): over
                // 400 000 sampled gradients the two hypots differ on 125 490
                // -- 31 % -- but on none of those does the difference move
                // `channel_threshold` far enough to flip the `<=`. All three
                // node-derived `golden_parity_river` cases stayed green across
                // the swap, which is the same answer from the oracle side. It
                // is corrected anyway: being one ulp more accurate than the
                // reference is the wrong answer here, and the next fixture
                // is not obliged to be as forgiving as these.
                // `* slope_w`, not `* w`: the grid the slope is normalised
                // against is the world's, which is the same grid for every
                // caller but a tile (this function's own doc comment).
                let slope_n = js_hypot(gx, gy) * slope_w as f64;
                slope_row[x] = slope_n as f32;
                if flow[i] as f64 <= channel_threshold(thresh, slope_n, density) {
                    continue;
                }
                chan_row[x] = 1;

                // Drops are measured on the routing surface (`route == fld`
                // unless integrated drainage is on) -- see
                // `build_channels_routed`.
                let hh = route[i] as f64;
                let aspect = js_atan2(-gy, -gx);
                let mut best: i64 = -1;
                let mut best_score = 0.0f64;
                let mut s_best: i64 = -1;
                let mut s_drop = 0.0f64;
                for dy in -1i64..=1 {
                    for dx in -1i64..=1 {
                        if dx == 0 && dy == 0 {
                            continue;
                        }
                        let mut nx = x as i64 + dx;
                        let ny = y as i64 + dy;
                        if wrap {
                            nx = ((nx % w as i64) + w as i64) % w as i64;
                        } else if nx < 0 || nx >= w as i64 {
                            continue;
                        }
                        if ny < 0 || ny >= h as i64 {
                            continue;
                        }
                        let j = ny * w as i64 + nx;
                        let drop = (hh - route[j as usize] as f64) / d8[((dy + 1) * 3 + (dx + 1)) as usize];
                        if drop <= 0.0 {
                            continue;
                        }
                        if drop > s_drop {
                            s_drop = drop;
                            s_best = j;
                        }
                        let mut da = js_atan2(dy as f64, dx as f64) - aspect;
                        da = js_atan2(da.sin(), da.cos()).abs();
                        let score = drop * (0.5 + 0.5 * da.cos());
                        if score > best_score {
                            best_score = score;
                            best = j;
                        }
                    }
                }
                // A cell the fill raised has no real gradient to steer by --
                // its drops are the ε tilt, a few ULPs -- so it takes the
                // plain steepest receiver: exactly `d8_receiver`'s pick (same
                // drops, same scan order, same strict `>`), i.e. the tree
                // `compute_flow` accumulated along. See
                // `build_channels_routed` for why and what it measured.
                let filled = route[i] > fld[i];
                recv_row[x] = if best >= 0 && !filled { best as i32 } else { s_best as i32 };
            }
        });

    ChannelResult { recv, chan, slope, intensity: Vec::new() }
}

/// `strahlerFromReceivers()` (reference HTML lines 4454-4464): standard
/// Strahler stream ordering over the single-receiver tree
/// `build_channels` produces — a channel cell's order bumps by 1 only
/// when *at least two* same-order tributaries converge into it, matching
/// the classical definition.
///
/// Processes channel cells sorted ascending by flow (JS: `Array.sort`,
/// stable since ES2019, so ties keep their original — ascending index —
/// relative order; Rust's `sort_by` is also stable, and building `cells`
/// by iterating `0..n` ascending reproduces the same starting order, so
/// no explicit index tiebreak is needed in the comparator itself).
///
/// NOT parallelized (`CPU_MULTITHREADING_SCOPE.md`): a genuine sequential
/// graph accumulation — each channel cell's own order depends on
/// `max_in`/`max_cnt` at its receiver having already been updated by
/// every one of its own upstream tributaries. Also channel-cell-count
/// sized, not grid-sized, so the real payoff would be small even if it
/// were parallelizable.
pub fn strahler_from_receivers(recv: &[i32], flow: &[f32], chan: &[u8]) -> Vec<i16> {
    let n = chan.len();
    let mut order = vec![0i16; n];
    let mut max_in = vec![0i16; n];
    let mut max_cnt = vec![0i16; n];

    let mut cells: Vec<usize> = (0..n).filter(|&i| chan[i] != 0).collect();
    cells.sort_by(|&a, &b| flow[a].total_cmp(&flow[b]));

    for &i in &cells {
        let o = if max_in[i] == 0 {
            1
        } else if max_cnt[i] >= 2 {
            max_in[i] + 1
        } else {
            max_in[i]
        };
        order[i] = o;
        let r = recv[i];
        if r >= 0 && chan[r as usize] != 0 {
            let r = r as usize;
            if o > max_in[r] {
                max_in[r] = o;
                max_cnt[r] = 1;
            } else if o == max_in[r] {
                max_cnt[r] += 1;
            }
        }
    }

    order
}

/// `riverWidthScaleK()` (reference HTML lines 2731-2734): channel-width
/// scale factor, real-km-aware like `river_coarse_ease`/`terrain_detail_k`
/// — but on the inverse side: a wider real map does not widen the literal
/// channel, so this divides `800/mapWidthKm` rather than multiplying, and
/// floors at `1/TERRAIN_DETAIL_MAX_K` rather than `1`. Deliberately
/// `map_width_km` alone, never blended with grid width, per the reference
/// comment's own explanation (matches `riverCoarseEase`'s established
/// reasoning). No-op (returns 1) at the literal default `mapWidthKm=800`,
/// at any resolution.
pub fn river_width_scale_k(map_width_km: f64) -> f64 {
    const TERRAIN_DETAIL_MAX_K: f64 = 16.0;
    let mwk = if map_width_km > 0.0 { map_width_km } else { 800.0 };
    (800.0 / mwk).clamp(1.0 / TERRAIN_DETAIL_MAX_K, TERRAIN_DETAIL_MAX_K)
}

/// `RIVER_RENDER_AREA_K` (reference HTML, v2.72's own comment above the
/// constant): the raster river renderer's own bar, and the reason it exists
/// at all is the reference's own line — *"A DETECTION EASE IS NOT A DISPLAY
/// THRESHOLD"*. [`river_flow_thresh`] eases `river_coarse_ease`'s factor into
/// whether a cell channelizes at all (so a genuine minor stream on a coarse
/// map is not missed); this constant, scaled the same way, is a second,
/// independent gate on whether a channelized cell is worth **painting**.
/// Without it, easing the detection threshold for a 40 000 km world also
/// eased what got drawn, and the whole 16x came out as ink: measured on the
/// reference at that extent, 13.02% of the map painted as channel against an
/// 800 km region's 4.27% — a dense, blocky network at a uniform floored
/// width in an opaque colour, which is the "solid block" the owner reported.
///
/// `2.0` is the reference's own literal, carried verbatim.
pub const RIVER_RENDER_AREA_K: f64 = 2.0;

/// `RIVER_RENDER_AREA_K * riverCoarseEase(mapWidthKm)` — the drainage bar
/// [`stamp_river_intensity`] gates its disc stamp on, centralised the same
/// way [`river_flow_thresh`] centralises the detection threshold: one
/// canonical call rather than every caller re-deriving the product.
///
/// **Why this stays cheap at the default extent.** `river_coarse_ease` is a
/// no-op (`1.0`) at or below 800 km, so here the bar is just `2.0` — enough
/// to drop a channel cell with no upstream channel neighbour of its own (a
/// literal one-cell dead-end source) but nothing more. A fixed bar sized for
/// the 40 000 km case (`~32`) applied uniformly would instead have gutted an
/// ordinary 800 km world's network — measured on the reference as a stem
/// count dropping 761 -> 93 (88%) there, the "fixes one world by gutting
/// every other" the source comment names. Scaling by the same ease that
/// eased detection is what keeps the two calibrated together.
pub fn river_render_area_bar(map_width_km: f64) -> f64 {
    RIVER_RENDER_AREA_K * cartalith_terrain::river_coarse_ease(map_width_km)
}

/// `traceRiverPolylines()` (reference HTML lines 4559-4575): walks each
/// channel cell's single receiver downstream from every *source* (a
/// channelized cell with no channelized upstream donor) until it either
/// runs off the channel network or rejoins an already-traced trunk —
/// sources ordered main-stems-first (descending Strahler order, stable on
/// ties) so trunks trace as long contiguous polylines rather than being
/// fragmented by a tributary trace claiming shared cells first.
///
/// Returns cell-center points (`{x: col+0.5, y: row+0.5}` in JS); this
/// port returns the same as `(f64, f64)` tuples rather than re-threading
/// a Godot/UI point type through a pure-Rust crate (`ARCHITECTURE.md`:
/// only `cartalith-godot` may depend on a rendering type).
///
/// NOT parallelized (`CPU_MULTITHREADING_SCOPE.md`): a sequential
/// downstream graph walk per source (`visited` also gates cross-source
/// sharing, so sources aren't even independent of each other) —
/// source-count sized, not grid-sized.
pub fn trace_river_polylines(order: &[i16], recv: &[i32], w: usize, h: usize, min_order: i32) -> Vec<Vec<(f64, f64)>> {
    let min_order = if min_order > 1 { min_order } else { 1 };
    let n = w * h;
    let mut has_up = vec![0u8; n];
    for i in 0..n {
        if (order[i] as i32) < min_order {
            continue;
        }
        let r = recv[i];
        if r >= 0 && (order[r as usize] as i32) >= min_order {
            has_up[r as usize] = 1;
        }
    }
    let mut sources: Vec<usize> = (0..n).filter(|&i| (order[i] as i32) >= min_order && has_up[i] == 0).collect();
    // JS `Array#sort` is stable (ES2019); `sources` was built by iterating
    // 0..n ascending, so this reproduces JS's ascending-index tiebreak
    // without an explicit secondary key, same reasoning
    // `strahler_from_receivers`'s own doc comment already applies.
    sources.sort_by(|&a, &b| order[b].cmp(&order[a]));
    let mut visited = vec![0u8; n];
    let mut polys = Vec::new();
    for &s in &sources {
        if visited[s] != 0 {
            continue;
        }
        let mut pts = Vec::new();
        let mut cur: i64 = s as i64;
        while cur >= 0 && (order[cur as usize] as i32) >= min_order {
            let ci = cur as usize;
            pts.push(((ci % w) as f64 + 0.5, (ci / w) as f64 + 0.5));
            if visited[ci] != 0 {
                break;
            }
            visited[ci] = 1;
            cur = recv[ci] as i64;
        }
        if pts.len() >= 2 {
            polys.push(pts);
        }
    }
    polys
}

/// `splitRiverPolylines()` (reference HTML lines 4596-4608): cuts a traced
/// chain wherever the next point is not reachable by a straight stroke from
/// the previous one, so a wrapped receiver chain is not drawn (or exported)
/// as one `LineString` running back across the whole map.
///
/// Two cuts, matching the reference's two reasons: `skip` (an optional
/// predicate — the renderer passes "is this point inside an open-water body",
/// so a river disappears under a lake surface; `exportGeoJSON` passes `null`,
/// because a lake reach is real hydrology and belongs in the geometry) and
/// the antimeridian seam, an x-jump of more than half the grid width.
///
/// Runs at the render/export sites only — `trace_river_polylines` itself is
/// untouched, so the `generate()`/carve pipeline stays bit-identical. Runs of
/// fewer than two points are dropped, exactly as the tracer drops them.
pub fn split_river_polylines(
    polys: &[Vec<(f64, f64)>],
    w: usize,
    skip: Option<&dyn Fn((f64, f64)) -> bool>,
) -> Vec<Vec<(f64, f64)>> {
    /// End the current run: keep it only if it is drawable, then start fresh.
    fn cut(run: &mut Vec<(f64, f64)>, out: &mut Vec<Vec<(f64, f64)>>) {
        if run.len() >= 2 {
            out.push(std::mem::take(run));
        } else {
            run.clear();
        }
    }

    let half = w as f64 * 0.5;
    let mut out: Vec<Vec<(f64, f64)>> = Vec::new();
    for pl in polys {
        let mut run: Vec<(f64, f64)> = Vec::new();
        for &p in pl {
            if skip.is_some_and(|f| f(p)) {
                cut(&mut run, &mut out);
                continue;
            }
            if run.last().is_some_and(|&last| (p.0 - last.0).abs() > half) {
                cut(&mut run, &mut out);
            }
            run.push(p);
        }
        cut(&mut run, &mut out);
    }
    out
}

/// One addressable river — a traced, drawable run of the receiver tree with
/// the readings that belong to the *run* rather than to a cell.
///
/// # There was no river entity, and this is the shape the reference implies
///
/// `buildRiverNetwork` returns rasters (`order`, `intensity`, `depth`, `recv`,
/// `slope`, `omax`); nothing in the reference aggregates a channel run into a
/// named object. What it does have is `drawRiverWays` (reference line 9473),
/// which per polyline computes `maxO` — *"the reference rescans the polyline
/// rather than trusting the source cell's order"*, as `geojson_bridge.rs`
/// already puts it — and colours the stroke by it. So a river here is exactly
/// what the reference draws as one: a `split_river_polylines` run, carrying
/// `maxO`. Everything else on this struct is read off rasters the engine
/// already retains, at cells this run already owns.
///
/// `pts` are cell centres (`col+0.5`, `row+0.5`), head first and mouth last —
/// `trace_river_polylines`' own downstream order.
pub struct River {
    pub pts: Vec<(f64, f64)>,
    /// Highest Strahler order anywhere on the run (`drawRiverWays`' `maxO`).
    pub order: i16,
    /// Summed segment length in grid cells.
    pub length_cells: f64,
    /// The largest `flow_discharge` anywhere on the run — the same
    /// rescan-the-polyline rule `order` follows (`drawRiverWays`' `maxO`).
    ///
    /// **Not the value at the mouth, and that is a correction rather than a
    /// preference.** The polyline follows `build_channels`' receiver tree,
    /// which is a D∞ *aspect* projection (Tarboton 1997); `flow_discharge`
    /// accumulated along `compute_flow`'s plain D8 steepest-descent tree, and
    /// the carve pass then moved the field under both. Those are different
    /// trees, so discharge is **not monotone** down a traced run. Measured on
    /// a real 192x144 world (`cartalith-godot/tests/river_entities.rs`): a run
    /// whose head carries 11.29 and whose mouth carries 3.32. Reporting the
    /// mouth would have shown a trunk as a trickle.
    pub discharge: f32,
    /// `flow_discharge` at the outlet cell specifically — kept alongside
    /// [`River::discharge`] because the two genuinely differ (see above) and a
    /// caller asking "what leaves this river" means this one.
    pub mouth_discharge: f32,
    /// Channel half-width in cells ([`channel_disc`]) at the run's last OWN
    /// cell -- the mouth for an outlet, the cell above the confluence for a
    /// tributary (whose mouth belongs to its trunk) -- or `None` when that
    /// cell carries no positive flow.
    pub half_width_cells: Option<f64>,
    /// How many other runs end on a cell of this one.
    pub tributaries: u32,
    /// Cell index of the first point (the headwater).
    pub head: u32,
    /// Cell index of the last point (the outlet).
    pub mouth: u32,
}

/// Every river on the world, as [`River`] entities.
///
/// Reuses the pair the GeoJSON exporter and the urban pass already run —
/// `trace_river_polylines` then `split_river_polylines(.., None)` — so an
/// entity is one *drawable* run: a receiver chain that wraps the antimeridian
/// becomes two rivers rather than one that streaks back across the map, which
/// is the same cut `export_geojson` makes and for the same reason. No lake
/// predicate, again matching the exporter: a lake reach is real hydrology.
///
/// `min_order` is clamped to `>= 1` by `trace_river_polylines` itself. Order 1
/// is thousands of headwater trickles on a large world; 2 is what
/// `EXPORT_MIN_RIVER_ORDER` uses.
///
/// # Tributaries are counted, not estimated
///
/// `trace_river_polylines` traces main stems first and stops a run at the
/// first already-visited cell, pushing that shared cell as the run's last
/// point — so a tributary's mouth *is* a cell of its trunk. Counting is
/// therefore exact: map every cell to the run that claimed it first (trunks
/// claim first, by that same ordering), then charge each run's mouth to the
/// run that owns it. A run whose mouth cell is its own is an outlet to sea,
/// lake or off-network, and charges nobody.
#[allow(clippy::too_many_arguments)]
pub fn river_entities(
    order: &[i16],
    recv: &[i32],
    flow: &[f32],
    fld: &[f32],
    w: usize,
    h: usize,
    min_order: i32,
    thresh: f64,
    width_k: f64,
    wrap: bool,
) -> Vec<River> {
    let n = w * h;
    if n == 0 || order.len() < n || recv.len() < n || flow.len() < n || fld.len() < n {
        return Vec::new();
    }
    let polys = split_river_polylines(&trace_river_polylines(order, recv, w, h, min_order), w, None);
    let lmax = channel_lmax(n);

    // Cell -> owning run, first writer wins (see the doc comment: trunks are
    // traced first, so a shared junction cell belongs to the trunk).
    let mut owner = vec![u32::MAX; n];
    let cell_of = |p: (f64, f64)| -> usize { (p.1 as usize).min(h - 1) * w + (p.0 as usize).min(w - 1) };
    for (ri, pl) in polys.iter().enumerate() {
        for &p in pl {
            let c = cell_of(p);
            if owner[c] == u32::MAX {
                owner[c] = ri as u32;
            }
        }
    }

    let mut out: Vec<River> = polys
        .iter()
        .enumerate()
        .map(|(ri, pl)| {
            let head = cell_of(pl[0]);
            let mouth = cell_of(pl[pl.len() - 1]);
            // A tributary's mouth is a cell of its TRUNK (see the doc comment),
            // so `channel_disc` there is the width *below* the confluence, and
            // every tributary drew as wide as the river it joins -- two equal
            // bands side by side into every junction. Its own last cell is
            // the width it actually has.
            let width_cell = if owner[mouth] as usize != ri { cell_of(pl[pl.len() - 2]) } else { mouth };
            let mut max_o = 0i16;
            let mut max_q = 0.0f32;
            let mut length_cells = 0.0f64;
            for (k, &p) in pl.iter().enumerate() {
                let c = cell_of(p);
                let o = order[c];
                if o > max_o {
                    max_o = o;
                }
                if flow[c] > max_q {
                    max_q = flow[c];
                }
                if k > 0 {
                    let q = pl[k - 1];
                    length_cells += js_hypot(p.0 - q.0, p.1 - q.1);
                }
            }
            River {
                pts: pl.clone(),
                order: max_o,
                length_cells,
                discharge: max_q,
                mouth_discharge: flow[mouth],
                half_width_cells: channel_disc(fld, flow, order, w, h, wrap, thresh, width_k, lmax, width_cell)
                    .map(|d| d.half_w),
                tributaries: 0,
                head: head as u32,
                mouth: mouth as u32,
            }
        })
        .collect();

    for ri in 0..out.len() {
        let trunk = owner[out[ri].mouth as usize];
        if trunk != u32::MAX && trunk as usize != ri {
            out[trunk as usize].tributaries += 1;
        }
    }
    out
}

/// How [`river_entities`]' runs are DRAWN, without changing what they are.
///
/// Owner, 2026-09-22, on the vector river strokes: *"make sure that rivers
/// don't become interrupted lines visually and connect them. Neither the tons
/// of parallel rivers."* Both are real properties of the traced runs, measured
/// on the shell's own 2048x1312 world (`_riverconnect_probe.gd`) before this
/// existed: 344 runs ended on dry land one D8 step from another run, and 56
/// ran alongside a heavier one.
///
/// * **Interrupted.** A run does not only end at a confluence or the sea: the
///   channel receiver tree has land pits (`recv == -1`), and the river resumes
///   as a new "source" a cell away. Following `recv` past such a mouth never
///   reached another run in 9 steps on either measured world, so the tree
///   itself offers no link -- the bridge is drawn, to the nearest cell of
///   another drawn run within ONE D8 step, the same reach the trace itself
///   links cells across. Never from a coastal mouth (it or a neighbour at or
///   below `sea_level`), where the adjacent run is a different river's mouth.
///   Never to one of the run's own tributaries, which would draw a loop.
/// * **Parallel.** The channel mask is often more than one cell wide, and
///   each column traces as its own run beside the others. Runs are visited
///   heaviest first -- weight is the largest `flow` over the run's OWN cells,
///   excluding a mouth that belongs to its trunk (that cell is shared, so
///   counting it gives a tributary its trunk's discharge; measured, pairs of
///   runs tied at exactly that value) -- and a run is hidden when at least
///   [`PARALLEL_MIN_CELLS`] of its own cells, and at least half of them, lie
///   within the two strokes' half-widths plus [`PARALLEL_GAP_CELLS`] of a
///   single already-kept run -- close enough to draw as one band. A real
///   tributary meets its trunk at an angle and approaches it for only its
///   last few cells. A run that ended on a now-hidden run reconnects to what
///   is drawn by that same reach.
///
/// Nothing here moves a run's `pts`, so the entity list, its indices and
/// `pick_river` are unchanged; the caller decides what to draw.
pub struct RiverDrawPlan {
    /// `Some(j)`: this run hugs the heavier run `j` and is not drawn.
    pub parallel_of: Vec<Option<usize>>,
    /// A cell centre this run's stroke continues to past its traced end.
    pub bridge: Vec<Option<(f64, f64)>>,
}

/// Fewest own cells alongside another run before a run counts as a parallel
/// duplicate rather than a tributary meeting it (see [`RiverDrawPlan`]).
pub const PARALLEL_MIN_CELLS: usize = 3;

/// Ground, in cells, between two strokes' edges below which they read as one
/// band (see [`RiverDrawPlan`]). Sized on the owner's own example: the
/// shell's 2048x1312 world draws an order-3 trunk and an order-1 stream side
/// by side for ~20 cells at centre distances of mostly 2.83 cells (two
/// diagonal steps) -- 1.83 cells of ground between 1-cell strokes. `1.0`
/// leaves that pair drawn; `2.0` is the smallest whole value that hides it.
/// Measured share of river cells hidden at `2.0`: 10% there, but 40% on an
/// 800 km / 384x288 world, where the channel mask is dense per cell.
pub const PARALLEL_GAP_CELLS: f64 = 2.0;

/// See [`RiverDrawPlan`].
pub fn river_draw_plan(rivers: &[River], flow: &[f32], fld: &[f32], sea_level: f64, w: usize, h: usize) -> RiverDrawPlan {
    let n = w * h;
    let mut plan = RiverDrawPlan { parallel_of: vec![None; rivers.len()], bridge: vec![None; rivers.len()] };
    if n == 0 || flow.len() < n || fld.len() < n {
        return plan;
    }
    let cell_of = |p: (f64, f64)| -> usize { (p.1 as usize).min(h - 1) * w + (p.0 as usize).min(w - 1) };
    let neighbours = |c: usize| {
        let (x, y) = ((c % w) as i64, (c / w) as i64);
        // Orthogonal first, so a nearest-first scan prefers distance 1 over sqrt 2.
        [(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (1, -1), (-1, 1), (-1, -1)]
            .into_iter()
            .map(move |(dx, dy)| (x + dx, y + dy))
            .filter(|&(x, y)| x >= 0 && y >= 0 && (x as usize) < w && (y as usize) < h)
            .map(|(x, y)| y as usize * w + x as usize)
    };
    // First writer owns a cell -- `river_entities`' own rule, so a tributary's
    // mouth resolves to its trunk.
    let mut owner = vec![usize::MAX; n];
    for (i, r) in rivers.iter().enumerate() {
        for &p in &r.pts {
            let c = cell_of(p);
            if owner[c] == usize::MAX {
                owner[c] = i;
            }
        }
    }
    let own_cells = |i: usize| -> Vec<usize> {
        let r = &rivers[i];
        let joins = owner[r.mouth as usize] != i;
        r.pts[..r.pts.len() - usize::from(joins)].iter().map(|&p| cell_of(p)).collect()
    };
    let weight: Vec<f32> =
        (0..rivers.len()).map(|i| own_cells(i).iter().map(|&c| flow[c]).fold(0.0f32, f32::max)).collect();
    // Drawn stroke width in cells -- what `get_rivers()` hands the renderer,
    // with the `channel_disc` floor (half-width 0.5) where it has none.
    let wid = |i: usize| rivers[i].half_width_cells.map_or(1.0, |hw| 2.0 * hw);
    let max_w = (0..rivers.len()).map(wid).fold(1.0f64, f64::max);
    // Two strokes read as one band when their centrelines are closer than
    // their half-widths plus `PARALLEL_GAP_CELLS` of ground.
    let reach = |i: usize, o: usize| (wid(i) + wid(o)) * 0.5 + PARALLEL_GAP_CELLS;
    // Every cell within `r` of `c`, nearest first (row-major on ties).
    let window = |c: usize, r: f64| -> Vec<(f64, usize)> {
        let (x, y) = ((c % w) as i64, (c / w) as i64);
        let k = r.floor() as i64;
        let mut v: Vec<(f64, usize)> = (-k..=k)
            .flat_map(|dy| (-k..=k).map(move |dx| (dx, dy)))
            .map(|(dx, dy)| (((dx * dx + dy * dy) as f64).sqrt(), x + dx, y + dy))
            .filter(|&(d, nx, ny)| d <= r && nx >= 0 && ny >= 0 && (nx as usize) < w && (ny as usize) < h)
            .map(|(d, nx, ny)| (d, ny as usize * w + nx as usize))
            .collect();
        v.sort_by(|a, b| a.0.total_cmp(&b.0));
        v
    };

    let mut order: Vec<usize> = (0..rivers.len()).collect();
    order.sort_by(|&a, &b| weight[b].total_cmp(&weight[a]).then(a.cmp(&b)));
    // Kept cells in WEIGHT order, for the hug test only.
    let mut kept = vec![usize::MAX; n];
    for &i in &order {
        let own = own_cells(i);
        let mut alongside: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
        let r_max = wid(i) * 0.5 + max_w * 0.5 + PARALLEL_GAP_CELLS;
        for &c in &own {
            let mut seen: Vec<usize> = Vec::new();
            for (d, q) in window(c, r_max) {
                let o = kept[q];
                if o != usize::MAX && o != i && !seen.contains(&o) && d <= reach(i, o) {
                    seen.push(o);
                    *alongside.entry(o).or_insert(0) += 1;
                }
            }
        }
        let hug = alongside
            .iter()
            .filter(|&(_, &k)| k >= PARALLEL_MIN_CELLS && 2 * k >= own.len())
            .max_by_key(|&(&j, &k)| (k, std::cmp::Reverse(j)));
        if let Some((&j, _)) = hug {
            plan.parallel_of[i] = Some(j);
            continue;
        }
        for &p in &rivers[i].pts {
            let c = cell_of(p);
            if kept[c] == usize::MAX {
                kept[c] = i;
            }
        }
    }
    // Drawn cells in TRACE order -- the `owner` rule restricted to drawn runs,
    // so a tributary's mouth resolves to its trunk whichever was heavier.
    let mut drawn = vec![usize::MAX; n];
    for (i, r) in rivers.iter().enumerate() {
        if plan.parallel_of[i].is_some() {
            continue;
        }
        for &p in &r.pts {
            let c = cell_of(p);
            if drawn[c] == usize::MAX {
                drawn[c] = i;
            }
        }
    }

    for i in 0..rivers.len() {
        if plan.parallel_of[i].is_some() {
            continue;
        }
        let m = rivers[i].mouth as usize;
        if drawn[m] != i {
            continue; // ends on another drawn run: already connected
        }
        let coastal = std::iter::once(m).chain(neighbours(m)).any(|c| fld[c] as f64 <= sea_level);
        if coastal {
            continue;
        }
        // A run whose mouth sat on a now-hidden run lost its trunk by the
        // hug's reach, so it reconnects by that reach; a land pit by one D8
        // step (the doc comment's two cases).
        let orphan = owner[m] != i;
        let r_max = if orphan { wid(i) * 0.5 + max_w * 0.5 + PARALLEL_GAP_CELLS } else { std::f64::consts::SQRT_2 };
        let target = window(m, r_max).into_iter().find(|&(d, q)| {
            let j = drawn[q];
            j != usize::MAX
                && j != i
                && drawn[rivers[j].mouth as usize] != i
                && (!orphan || d <= reach(i, j))
        });
        if let Some((_, q)) = target {
            plan.bridge[i] = Some(((q % w) as f64 + 0.5, (q / w) as f64 + 0.5));
        }
    }
    plan
}

/// Nearest river to a grid-space point, within `radius_cells` of one of its
/// segments — the engine half of viewport river hit-testing.
///
/// Distance is to the polyline, not to its vertices: a click between two cell
/// centres of the same reach must select that reach, and at
/// `ViewportHost.ZOOM_MAX` a one-cell gap is ~29 screen px (`get_roads()`'s
/// own measurement of the same problem).
///
/// Ties go to the nearer run, and where two runs are equidistant, to the
/// higher `order` — a trunk and the tributary that ends on it share exactly
/// one cell (see [`river_entities`]), and selecting the trunk there is what a
/// pointer means.
///
/// Comparisons are on squared distances and no `hypot` appears: this is a
/// pointer pick with no counterpart in the reference, so there is no JS result
/// to match and nothing to spend a compensated sum on.
pub fn pick_river(rivers: &[River], gx: f64, gy: f64, radius_cells: f64) -> Option<usize> {
    /// Squared distance from `p` to segment `a..b`.
    fn seg_d2(p: (f64, f64), a: (f64, f64), b: (f64, f64)) -> f64 {
        let (vx, vy) = (b.0 - a.0, b.1 - a.1);
        let len2 = vx * vx + vy * vy;
        let t = if len2 > 0.0 { (((p.0 - a.0) * vx + (p.1 - a.1) * vy) / len2).clamp(0.0, 1.0) } else { 0.0 };
        let (dx, dy) = (p.0 - (a.0 + t * vx), p.1 - (a.1 + t * vy));
        dx * dx + dy * dy
    }

    let p = (gx, gy);
    let r2 = radius_cells * radius_cells;
    let mut best: Option<(usize, f64, i16)> = None;
    for (i, r) in rivers.iter().enumerate() {
        let mut d2 = f64::INFINITY;
        for seg in r.pts.windows(2) {
            let d = seg_d2(p, seg[0], seg[1]);
            if d < d2 {
                d2 = d;
            }
        }
        if d2 > r2 {
            continue;
        }
        match best {
            Some((_, bd, bo)) if !(d2 < bd || (d2 == bd && r.order > bo)) => {}
            _ => best = Some((i, d2, r.order)),
        }
    }
    best.map(|(i, _, _)| i)
}

/// `enforceChannelDescent()` (reference HTML lines 8725-8739): walks an
/// ordered (downstream) polyline and carves a channel whose centreline
/// descends monotonically — cutting through any rises so the carved
/// valley actually drains to its outlet — stamping a parabolic cross-
/// section (floor at centre, blending to existing terrain at `half_w`)
/// per point. Returns the carved cell indices so the caller can lock them
/// against later deposition refill (JS: `riverMask`/`riverFloor`).
///
/// `drop` is the JS default `opts.drop` (`0.0006`) inlined as an explicit
/// parameter rather than an `Option` — this port has no caller yet that
/// needs a different value, and an unused-override knob would exist
/// solely to mirror JS's options-object shape.
///
/// NOT parallelized (`CPU_MULTITHREADING_SCOPE.md`): each point's `floor`
/// is bounded by `prev`, the previous (upstream) point's own floor — a
/// genuine sequential dependency along the polyline. Adjacent points'
/// `half_w`-radius stamps can also overlap the same cells, a real
/// scatter-write hazard between iterations. One river's polyline at a
/// time anyway, not grid-sized.
pub fn enforce_channel_descent(
    fld: &mut [f32],
    w: usize,
    h: usize,
    pts: &[(f64, f64)],
    sea: f64,
    half_w: f64,
    drop: f64,
) -> Vec<usize> {
    let floor_lim = sea - 0.06;
    let mut out = Vec::new();
    let mut prev = f64::INFINITY;
    for (k, &(px_f, py_f)) in pts.iter().enumerate() {
        let px = (px_f as i64).clamp(0, w as i64 - 1) as usize;
        let py = (py_f as i64).clamp(0, h as i64 - 1) as usize;
        // never higher than the previous (upstream) point
        let mut floor = (fld[py * w + px] as f64).min(prev - if k > 0 { drop } else { 0.0 });
        if floor < floor_lim {
            floor = floor_lim;
        }
        prev = floor;
        let r = half_w.ceil() as i64;
        let x0 = (px as i64 - r).max(0) as usize;
        let x1 = (px as i64 + r).min(w as i64 - 1) as usize;
        let y0 = (py as i64 - r).max(0) as usize;
        let y1 = (py as i64 + r).min(h as i64 - 1) as usize;
        for y in y0..=y1 {
            for x in x0..=x1 {
                // `Math.hypot`, not `f64::hypot` (reference line 8733). The
                // offsets are small integers, which is *not* enough to make
                // the two agree: over the 400 integer pairs `0..19` they
                // differ on **108**, e.g. `(1,5)` gives 5.099019513592785
                // under V8 and 5.0990195135927845 correctly rounded. Reached
                // whenever `half_w` exceeds ~4 cells, which
                // `river_width_scale_k` makes ordinary below ~200 km.
                //
                // Worse here than at the two slope call sites: `d` does not
                // merely cross a branch, it becomes `t = d / half_w` and is
                // written into the terrain field as
                // `floor + (fld[i] - floor) * t * t`. See
                // `enforce_channel_descent_carves_the_v8_hypot_disc`.
                let d = js_hypot(x as f64 - px as f64, y as f64 - py as f64);
                if d > half_w {
                    continue;
                }
                let t = d / half_w;
                let i = y * w + x;
                // parabolic: floor at centre -> terrain at edge
                let target = floor + (fld[i] as f64 - floor) * t * t;
                if target < fld[i] as f64 {
                    fld[i] = target as f32;
                    out.push(i);
                }
            }
        }
    }
    out
}

/// `enforceRiverChannels()` (reference HTML lines 8742-8745): clamp every
/// locked river cell back down to its carved floor.
///
/// The reference's own framing is "England-style entrenchment" — protect a
/// carved channel from being refilled by later deposition, isostatic rebound
/// or (`UNIFIED_TOOL_PLAN.md` milestone C, the new caller) a Sculpt stamp
/// that raises terrain straight over an already-locked channel. It is a
/// no-op until something has actually locked cells, which is what
/// `_riverAny` guards; here the caller's `river_any` flag carries that, and
/// an all-zero mask makes the loop a no-op anyway.
///
/// Deliberately one-directional: it only ever *lowers*. A cell that erosion
/// cut *below* its recorded floor keeps the deeper value.
pub fn enforce_river_channels(field: &mut [f32], river_mask: &[u8], river_floor: &[f32]) {
    for i in 0..field.len().min(river_mask.len()).min(river_floor.len()) {
        if river_mask[i] != 0 && field[i] > river_floor[i] {
            field[i] = river_floor[i];
        }
    }
}

/// `lmax = Math.log(W*H*0.05)` (reference HTML line 4495) — the discharge
/// log-scale every channel cell's `mag` is normalised against. One definition,
/// because [`stamp_river_intensity`] and [`channel_disc`] must agree on it and
/// a caller of the second has no other way to obtain it.
pub fn channel_lmax(n: usize) -> f64 {
    ((n as f64) * 0.05).ln()
}

/// One channel cell's stamped disc: how wide the channel is drawn there, how
/// bright, and the normalised discharge both come from.
pub struct ChannelDisc {
    /// Channel half-width in **grid cells**, `[0.5, 9·width_k]`. Double it for
    /// a full width; multiply by `map_width_km / gw` for kilometres.
    pub half_w: f64,
    /// Peak ink at the centreline, `[0, 1]`.
    pub amp: f64,
    /// Normalised discharge, `log(flow/thresh)/lmax` capped at 1. **Not floored
    /// at 0** — see below.
    pub mag: f64,
}

/// `buildRiverNetwork`'s per-cell disc geometry (reference HTML lines
/// **4532-4537**), lifted verbatim out of [`stamp_river_intensity`]'s loop.
/// (`half_w_cap` is the same expression the reference hoists to 4530.)
///
/// The range used to read "4534-4540" — which **excludes 4532**, the `mag`
/// line the `.max(0.0)` correction below is entirely about, and runs three
/// lines past `amp` into the stamp loop this function does not contain.
///
/// # Why it is its own function
///
/// The channel-width law had exactly one consumer — the intensity raster — so
/// "how wide is this river?" could not be asked about a *river*, only inked
/// per cell. `right_dock.gd`'s River context said so in as many words
/// (*"strahler_from_receivers, compute_flow's flow_discharge,
/// river_width_scale_k … only ever as per-CELL rasters"*). [`river_entities`]
/// is the second caller, and it reads the same arithmetic in the same order
/// rather than restating it — the reference's own history is the argument
/// here, having found ~14 drifting re-implementations of `riverFlowThresh`.
///
/// Returns `None` for a cell with no positive flow, which is the loop's own
/// `continue`. The caller checks `chan[i]` itself: [`stamp_river_intensity`]
/// needs that test to skip the cell entirely, and a river entity's cells are
/// channel cells by construction.
///
/// # `mag` is not floored at zero, and that is a fix
///
/// This port had `mag = ln(f/thresh).max(0).min(lmax) / lmax`; the reference
/// is `Math.min(1, Math.log(f/thresh)/lmax)` — **no lower clamp**. The
/// `.min(lmax)/lmax` half is exactly equivalent to `.min(1)` after the divide;
/// the `.max(0)` was not in the reference at all.
///
/// It is reachable, not theoretical. A cell channelizes when `flow >
/// channel_threshold(thresh, slope_n, density)`, and at `river_density != 1`
/// that threshold sits *below* `thresh` — so a channel cell can carry
/// `flow < thresh`, giving a negative `mag`. `half_w` is unaffected (`mag` only
/// enters it squared), but `amp = min(1, 0.45 + 0.7·mag)` is: the reference
/// dims a barely-channelized trickle below `0.45` ink and this port did not.
/// At the default `river_density = 1` the two forms are identical, because
/// `channel_threshold` is then exactly `thresh` — which is why no golden moved.
#[allow(clippy::too_many_arguments)]
pub fn channel_disc(
    fld: &[f32],
    flow: &[f32],
    order: &[i16],
    w: usize,
    h: usize,
    wrap: bool,
    thresh: f64,
    width_k: f64,
    lmax: f64,
    i: usize,
) -> Option<ChannelDisc> {
    // Total rather than panicking: this is `pub`, and a panic here would cross
    // the gdext boundary and take the Godot process down with it
    // (`cartalith-rust-conventions`).
    let n = w * h;
    if i >= n || fld.len() < n || flow.len() < n || order.len() < n || !(lmax > 0.0) || !(thresh > 0.0) {
        return None;
    }
    let f = flow[i] as f64;
    if !(f > 0.0) {
        return None;
    }
    let (x, y) = (i % w, i / w);
    let o = order[i].max(1) as f64;
    let mag = js_min(1.0, (f / thresh).ln() / lmax);

    let xl = if wrap { (x + w - 1) % w } else { x.saturating_sub(1) };
    let xr = if wrap { (x + 1) % w } else { (x + 1).min(w - 1) };
    let gx = (fld[y * w + xr] as f64 - fld[y * w + xl] as f64) * 0.5;
    let up = if y > 0 { fld[(y - 1) * w + x] as f64 } else { fld[i] as f64 };
    let dn = if y < h - 1 { fld[(y + 1) * w + x] as f64 } else { fld[i] as f64 };
    let gy = (dn - up) * 0.5;
    // `Math.hypot`, as in `build_channels`' own slope above -- `slope_fac`
    // reaches the `d > half_w` test that decides which cells the disc inks.
    let slope_fac = 1.0 / (1.0 + 5.0 * js_hypot(gx, gy) * w as f64);

    let mut half_w = (0.6 + 3.0 * mag * mag + 0.45 * (o - 1.0)) * slope_fac * width_k;
    let half_w_cap = 9.0 * width_k;
    if half_w < 0.5 {
        half_w = 0.5;
    } else if half_w > half_w_cap {
        half_w = half_w_cap;
    }
    let amp = js_min(1.0, 0.45 + mag * 0.7);
    Some(ChannelDisc { half_w, amp, mag })
}

/// Each channel cell's own upstream drainage, counted in **channel cells**
/// off the `recv`/`chan` receiver tree — the reference's `buildMainStems`
/// Kahn accumulation (v2.72's `st.area`), never `flow`
/// ([`stamp_river_intensity`]'s own doc comment explains why that
/// substitution would be wrong).
///
/// Processed in the same ascending-`flow` order
/// [`strahler_from_receivers`] already establishes as a valid topological
/// order for this tree (see that function's own doc comment: a channel
/// cell's own accumulated value is complete only after every upstream
/// tributary that drains into it has already been folded in, and sorting by
/// ascending discharge visits upstream cells first because discharge only
/// grows downstream).
///
/// A non-channel cell reads `0`; a channel cell with no channelized upstream
/// neighbour of its own reads `1` (itself, only).
fn channel_cell_drainage(recv: &[i32], flow: &[f32], chan: &[u8]) -> Vec<i32> {
    let n = chan.len();
    let mut area = vec![0i32; n];
    let mut cells: Vec<usize> = (0..n).filter(|&i| chan[i] != 0).collect();
    cells.sort_by(|&a, &b| flow[a].total_cmp(&flow[b]));
    for &i in &cells {
        area[i] += 1;
        let r = recv[i];
        if r >= 0 && (r as usize) < n && chan[r as usize] != 0 {
            area[r as usize] += area[i];
        }
    }
    area
}

/// The stamped channel *intensity* raster — `buildRiverNetwork`'s disc stamp
/// (reference HTML lines 4528-4543), which this port had never carried.
///
/// # Why this exists
///
/// Owner, 2026-08-30: *"As soon as the map width/size becomes lower the size
/// width and length of a river should become bigger and more visible."*
///
/// Until now the port drew a river as `chan[i] != 0` — a binary flag, so
/// **every river was exactly one grid cell wide** regardless of its Strahler
/// order or the world's real extent. At a 2048 grid in a 1400 px viewport that
/// is 0.68 screen pixels: the "barely visible" in the report. The km-aware
/// width law already existed ([`river_width_scale_k`]) and was used **only to
/// carve terrain**; the mask it produced was never the mask that got drawn.
///
/// # What it computes
///
/// Per channel cell, a parabolic disc of half-width
/// `(0.6 + 3·mag² + 0.45·(o−1)) · slope_fac · width_k`, clamped to
/// `[0.5, 9·width_k]`, where `mag` is normalised discharge, `o` is Strahler
/// order and `slope_fac = 1/(1 + 5·|∇field|·w)` narrows a river on steep
/// ground. Discs composite by `max`, exactly as the reference does.
///
/// `width_k` is [`river_width_scale_k`]'s inverse-extent factor, so a 200 km
/// world stamps 4× the half-width of an 800 km one. Measured against this
/// formula at `w = 2048`: an order-7 river is ~1.1 cells wide at 800 km, ~4.5
/// at 200 km and ~18 at 50 km, while an order-1 stream stays at the 0.5 floor
/// until about 100 km. That floor is the reference's own literal and is
/// **not** scaled by `width_k` — so at world scale it binds and rivers stay
/// one cell, which is the intended "a world-scale map stops exaggerating a
/// river" behaviour rather than an oversight.
///
/// # Deliberately not ported
///
/// The reference computes `depth` and `omax` in the same loop. Both are
/// omitted: `depth` feeds a terrain-shading path this port does not have, and
/// `omax` exists to let a biome overlay filter by minimum stream order — a
/// filter this port does not implement anywhere (`min_river_order` has no
/// consumer outside the GeoJSON exporter's own constant). Adding either now
/// would be a second grid with no reader.
///
/// The per-cell disc geometry itself moved out to [`channel_disc`], which is
/// the loop body's first half unchanged — see that function for why it has a
/// second caller.
///
/// `Math.sqrt(dx*dx+dy*dy)`, not `Math.hypot`, is what the reference's own
/// inner stamp loop uses (reference line 4539), and this port matches it. On
/// the small exact integers `dx`/`dy` take here the two agree to the bit, but
/// the reference is the reference.
///
/// # `area_bar` — v2.72's display-side gate (`RIVER_RENDER_AREA_K`)
///
/// A second reason a channel cell can go un-inked, independent of `thresh`:
/// [`river_render_area_bar`], applied to [`channel_cell_drainage`] — each
/// channel cell's own upstream drainage **in channel-cell count**, off this
/// exact receiver tree (`recv`/`chan`), never `flow`. The reference is
/// explicit that `flow`/`flowField` is a *different* tree
/// (`compute_flow`'s plain D8 accumulation) from the one `chan`'s
/// channel-cell topology walks (`build_channels`' D-infinity aspect
/// projection) — "v2.58/v2.41: they are two different trees" is the
/// reference's own words for the same distinction this port already draws
/// between `compute_flow` and `build_channels`.
///
/// **A per-cell gate, not a per-stem one, and that is a real translation, not
/// an oversight.** The reference groups traced polylines into stems
/// (`buildMainStems`) and keeps or drops a *whole* stem on its own area. This
/// port's disc stamp has no stem grouping — it inks per channel cell, as it
/// always has — so the natural gate is per cell on the same metric. Because
/// `channel_cell_drainage` only grows going downstream (a confluence sums its
/// tributaries), the practical effect is the same shape: a headwater cell
/// with no upstream channel neighbour of its own (`area == 1`) drops out
/// first, and a trunk downstream of enough confluences keeps drawing even
/// where an individual tributary above it did not clear the bar. `area_bar
/// <= 0.0` disables the gate entirely (every channel cell inks, the pre-v2.72
/// behaviour), which is what a caller that has not computed
/// [`river_render_area_bar`] gets by passing `0.0`.
///
/// # Connectivity across a diagonal receiver step
///
/// Each channel cell stamps its own disc independently, with no reference to
/// its neighbours in the receiver chain — so a cell with `half_w < 1.0`
/// (common: the floor in [`channel_disc`] is 0.5, and `slope_fac` narrows a
/// disc further on steep ground) inks *only* its own centre, since `d==1`
/// for an orthogonal neighbour and `d~=1.414` for a diagonal one both exceed
/// `half_w`. A D8 chain steps diagonally about 42% of the time (measured in
/// the reference, `RC_ENGINE_CHANGES.md` §6l), so two consecutive narrow
/// channel cells one diagonal step apart can paint two discs that never
/// touch — a visible break. This is v2.60's fix in the reference, ported
/// forward here after this port shipped ahead of it: after the main stamp
/// loop, a second pass finds every channel cell whose receiver is a genuine
/// diagonal step away (wrap-aware in `x`, matching [`d8_receiver`]'s own "`x`
/// only" rule) and raises the two cells common to both endpoints to the
/// dimmer of the two centres, guaranteeing 4-connectivity along the actual
/// flow chain without touching any river's rendered width elsewhere. An
/// orthogonal or same-cell receiver is already 4-connected on its own and
/// untouched by this pass; a receiver that is not a true grid neighbour at
/// all cannot occur (`d8_receiver` only ever returns one of the 8 immediate
/// neighbours or `-1`), so this is not a heuristic distance cutoff — it is
/// the complete set of steps a receiver chain can take that are not already
/// connected.
/// The shortest signed offset from grid coordinate `a` to `b` along one axis,
/// wrapping through the seam when `wrap` is set (mirrors `d8_receiver`'s own
/// "`world` wraps `x` only" rule — call this only for the `x` axis, never
/// `y`). Used by [`stamp_river_intensity`]'s connectivity bridge to tell a
/// genuine one-cell diagonal step (raw index difference near `w`, because the
/// step crosses the antimeridian) from any other receiver relationship.
#[inline]
fn wrapped_axis_delta(a: usize, b: usize, w: usize, wrap: bool) -> i64 {
    let raw = b as i64 - a as i64;
    if !wrap || w == 0 {
        return raw;
    }
    let w = w as i64;
    let half = w / 2;
    if raw > half {
        raw - w
    } else if raw < -half {
        raw + w
    } else {
        raw
    }
}

#[allow(clippy::too_many_arguments)]
pub fn stamp_river_intensity(
    fld: &[f32],
    flow: &[f32],
    chan: &[u8],
    recv: &[i32],
    order: &[i16],
    w: usize,
    h: usize,
    wrap: bool,
    thresh: f64,
    width_k: f64,
    area_bar: f64,
) -> Vec<f32> {
    let n = w * h;
    let mut intensity = vec![0f32; n];
    if n == 0 || fld.len() < n || flow.len() < n || chan.len() < n || recv.len() < n || order.len() < n {
        return intensity;
    }
    let lmax = channel_lmax(n);
    if !(lmax > 0.0) || !(thresh > 0.0) {
        return intensity;
    }
    let area = if area_bar > 0.0 { Some(channel_cell_drainage(recv, flow, chan)) } else { None };

    for i in 0..n {
        if chan[i] == 0 {
            continue;
        }
        if let Some(a) = &area
            && (a[i] as f64) < area_bar
        {
            continue;
        }
        let Some(disc) = channel_disc(fld, flow, order, w, h, wrap, thresh, width_k, lmax, i) else {
            continue;
        };
        let ChannelDisc { half_w, amp, .. } = disc;
        let (x, y) = (i % w, i / w);

        let r = half_w.ceil() as isize;
        let (xi, yi) = (x as isize, y as isize);
        for yy in (yi - r).max(0)..=(yi + r).min(h as isize - 1) {
            for xx in (xi - r).max(0)..=(xi + r).min(w as isize - 1) {
                let (dx, dy) = ((xx - xi) as f64, (yy - yi) as f64);
                let d = (dx * dx + dy * dy).sqrt();
                if d > half_w {
                    continue;
                }
                let v = (amp * (1.0 - d / half_w)) as f32;
                let j = yy as usize * w + xx as usize;
                if v > intensity[j] {
                    intensity[j] = v;
                }
            }
        }
    }

    // Connectivity bridge (v2.60's reference fix this port never carried,
    // `RC_ENGINE_CHANGES.md` §6l): each channel cell stamps its own disc
    // independently, so a `half_w < 1.0` cell (an order-1 headwater at the
    // 0.5 floor, or any cell narrowed by `slope_fac`) inks *only* its own
    // centre — `d==1` for an orthogonal neighbour and `d~=1.414` for a
    // diagonal one are both `> half_w`. A D8 receiver chain steps diagonally
    // about 42% of the time (the reference's own measurement), and when two
    // consecutive channel cells with sub-1 half-widths are a diagonal step
    // apart, their discs do not touch even at a shared corner: the rendered
    // river breaks.
    //
    // Fix, scoped to exactly that gap: for each channel cell `i` whose
    // receiver `recv[i]` is a diagonal step away (wrap-aware in `x`, per
    // `wrapped_axis_delta`; never in `y`, which never wraps) and whose own
    // centre and its receiver's centre both actually inked, raise the two
    // cells common to both -- `(recv.x, i.y)` and `(i.x, recv.y)`, the only
    // cells 4-adjacent to *both* endpoints of a diagonal step -- to at most
    // the dimmer of the two centres. That guarantees a 4-connected path
    // `i -> bridge -> recv[i]` without touching `half_w`, `amp`, or any other
    // river's rendered width: an orthogonal or non-adjacent receiver (the
    // v2.72 "cut the stem where it wraps" case this port already tests) is
    // untouched, because `d8_receiver` only ever points at one of the 8
    // immediate neighbours, so every *other* receiver relationship already
    // has `|dx|<=1 && |dy|<=1` with at least one of them 0 -- already
    // 4-connected on its own.
    for i in 0..n {
        if chan[i] == 0 || intensity[i] <= 0.0 {
            continue;
        }
        let r = recv[i];
        if r < 0 {
            continue;
        }
        let r = r as usize;
        if r >= n || chan[r] == 0 || intensity[r] <= 0.0 {
            continue;
        }
        let (x, y) = (i % w, i / w);
        let (rx, ry) = (r % w, r / w);
        let dx = wrapped_axis_delta(x, rx, w, wrap);
        let dy = ry as i64 - y as i64;
        if dx.abs() != 1 || dy.abs() != 1 {
            continue;
        }
        let bridge_val = intensity[i].min(intensity[r]);
        let b1 = y * w + rx;
        if bridge_val > intensity[b1] {
            intensity[b1] = bridge_val;
        }
        let b2 = ry * w + x;
        if bridge_val > intensity[b2] {
            intensity[b2] = bridge_val;
        }
    }

    intensity
}

#[cfg(test)]
mod tests {
    use super::{build_channels, enforce_river_channels, flow_cmp_desc, flow_sort_desc};

    /// A run over cell centres `(x, y)` with every cell at `q` in `flow`.
    fn run(cells: &[(usize, usize)], w: usize, flow: &mut [f32], q: f32) -> super::River {
        for &(x, y) in cells {
            flow[y * w + x] = flow[y * w + x].max(q);
        }
        let c = |i: usize| (cells[i].1 * w + cells[i].0) as u32;
        super::River {
            pts: cells.iter().map(|&(x, y)| (x as f64 + 0.5, y as f64 + 0.5)).collect(),
            order: 1,
            length_cells: 0.0,
            discharge: q,
            mouth_discharge: q,
            half_width_cells: Some(0.5),
            tributaries: 0,
            head: c(0),
            mouth: c(cells.len() - 1),
        }
    }

    /// `river_draw_plan` on a hand-built network, each rule exercised once.
    /// Strokes are 1 cell wide (so the hug reach is `1 + PARALLEL_GAP_CELLS`
    /// = 3 cells) except `coast_b`, whose 4-cell width widens every search
    /// window to 4.5 cells without changing any 1-cell pair's reach. Runs
    /// meant not to interact sit further apart than their reach.
    #[test]
    fn river_draw_plan_hides_parallels_and_bridges_land_pits() {
        let (w, h) = (30usize, 16usize);
        let n = w * h;
        let mut flow = vec![0f32; n];
        let mut fld = vec![0.8f32; n];
        // Sea along the bottom row.
        for x in 0..w {
            fld[(h - 1) * w + x] = 0.1;
        }
        let trunk: Vec<(usize, usize)> = (0..10).map(|x| (x, 5)).collect();
        // Exactly 3 rows below the trunk (the reach, to the cell), then onto
        // it: one band with it. Its head (0,9) is 4 cells off.
        let beside: Vec<(usize, usize)> =
            [(0, 9)].into_iter().chain((0..6).map(|x| (x, 8))).chain([(6, 7), (7, 6), (8, 5)]).collect();
        // Onto the trunk's end with 2 of 4 own cells in reach -- half its
        // length, but under `PARALLEL_MIN_CELLS`, so it stays drawn.
        let trib: Vec<(usize, usize)> = vec![(11, 1), (11, 2), (10, 3), (10, 4), (9, 5)];
        // 3 own cells, all in reach of the trunk: exactly the count floor.
        let stub: Vec<(usize, usize)> = vec![(8, 7), (9, 7), (10, 7)];
        // Onto the trunk with its last 3 of 8 own cells in reach: at the count
        // floor but under half, so it is a tributary, not a duplicate.
        let long_trib: Vec<(usize, usize)> =
            vec![(1, 0), (2, 0), (3, 0), (4, 0), (5, 1), (5, 2), (5, 3), (5, 4), (5, 5)];
        // Both end on `beside`, which is hidden, so both are outlets now.
        // `orphan` ends 3 cells from the trunk and reconnects to it; `orphan2`
        // ends 4 cells off -- inside the 4.5-cell window, outside the reach.
        let orphan: Vec<(usize, usize)> = vec![(4, 12), (4, 11), (4, 10), (4, 9), (4, 8)];
        let orphan2: Vec<(usize, usize)> = vec![(0, 13), (0, 12), (0, 11), (0, 10), (0, 9)];
        // A land pit at (17,3); the river resumes diagonally at (18,4).
        let upper: Vec<(usize, usize)> = (14..18).map(|x| (x, 3)).collect();
        let lower: Vec<(usize, usize)> = (18..23).map(|x| (x, 4)).collect();
        // Ends on `upper`'s cell (16,3), so it is `upper`'s tributary, and its
        // (17,2) is an ORTHOGONAL step from `upper`'s pit -- scanned before
        // `lower`'s diagonal (18,4). It must be skipped, or the bridge draws a
        // loop. (The trunk's end (9,5) exercises the same guard: `trib` ends
        // there, and its (10,4) is diagonal to it.)
        let upper_trib: Vec<(usize, usize)> = vec![(17, 1), (17, 2), (16, 3)];
        // Ends at (26,14), beside the sea row and one step from `coast_b`.
        let coast_a: Vec<(usize, usize)> = (2..15).map(|y| (26, y)).collect();
        let coast_b: Vec<(usize, usize)> = vec![(27, 14), (28, 14), (29, 14)];
        let mut rivers = vec![
            run(&trunk, w, &mut flow, 100.0),
            run(&beside, w, &mut flow, 40.0),
            run(&trib, w, &mut flow, 30.0),
            run(&upper, w, &mut flow, 5.0),
            run(&lower, w, &mut flow, 20.0),
            run(&coast_a, w, &mut flow, 10.0),
            run(&coast_b, w, &mut flow, 12.0),
            run(&upper_trib, w, &mut flow, 1.0),
            run(&long_trib, w, &mut flow, 25.0),
            run(&orphan, w, &mut flow, 15.0),
            run(&orphan2, w, &mut flow, 14.0),
            run(&stub, w, &mut flow, 8.0),
        ];
        rivers[6].half_width_cells = Some(2.0);
        // `beside`'s shared mouth (8,5) carries the trunk's 100: its weight
        // comes from its own cells.
        let plan = super::river_draw_plan(&rivers, &flow, &fld, 0.42, w, h);
        let hidden: Vec<usize> = (0..rivers.len()).filter(|&i| plan.parallel_of[i].is_some()).collect();
        assert_eq!(hidden, vec![1, 11], "`beside` and `stub` hide, nothing else");
        assert_eq!((plan.parallel_of[1], plan.parallel_of[11]), (Some(0), Some(0)), "both behind the trunk");
        assert_eq!(plan.bridge[9], Some((4.5, 5.5)), "a hidden run's tributary reconnects to what is drawn");
        assert_eq!(plan.bridge[10], None, "but not from beyond the reach, however wide the window");
        assert_eq!(plan.bridge[3], Some((18.5, 4.5)), "the land pit continues onto the next run, not its own tributary");
        assert_eq!(plan.bridge[5], None, "a coastal mouth is a river mouth, not a pit");
        assert_eq!(plan.bridge[4], None, "`lower` has nothing within one D8 step of its end");
        assert_eq!(plan.bridge[0], None, "the trunk's only neighbour is its own tributary");
        assert_eq!(plan.bridge[2], None, "a tributary already ends on its trunk");
        assert_eq!(plan.bridge[7], None, "`upper_trib` already ends on `upper`");
    }

    /// The two *slope* `Math.hypot` call sites (`build_channels`' `slope_n`
    /// and `channel_disc`'s `slope_fac`) take arbitrary `f64` gradients, so V8's
    /// scaled Kahan sum and Rust's `f64::hypot` are free to disagree by an ulp
    /// — and `slope_n` lands in `flow[i] <= channel_threshold(..)`, a discrete
    /// branch. **Measured, not asserted from theory:** this counts how often
    /// the two differ at all, and how often the difference flips the
    /// channelization decision. Last run: **125 490 of 400 000 gradients
    /// differ (31 %), 0 flip the threshold.** The divergence is real and
    /// common; its reach into this particular branch is not.
    ///
    /// The `d8` tables in `compute_flow`/`build_channels` are the same
    /// `Math.hypot` in the reference but take only `{-1,0,1}`; the second half
    /// of this test pins that those nine values are bit-identical either way,
    /// which is why those two lines are deliberately left as `f64::hypot`.
    ///
    /// **This test measures; it does not pin either call site**, and saying so
    /// is the point — reverting `channel_disc`'s `js_hypot` to `f64::hypot`
    /// scored green against it. `channel_disc_width_law_is_bit_exact_against_the_reference`
    /// is the pin for that one, and
    /// `enforce_channel_descent_carves_the_v8_hypot_disc` for the carve radius.
    ///
    /// `build_channels`' own `slope_n` has **no** such pin, and not for want of
    /// looking: at `density == 1` `channel_threshold` collapses to exactly
    /// `thresh` (reference line 4508's own comment), so `slope_n` cannot reach
    /// `chan` at all there, and the only other output it feeds is
    /// `slope[i] = slope_n as f32` — a cast that swallows a one-ulp `f64`
    /// difference unless the two straddle an `f32` rounding midpoint.
    /// Searched: **600 000 000 random `f32` height quads, 0 produced a
    /// differing `slope` entry.** The correction there stands on the two
    /// call sites that *are* pinned plus this measurement, and a future
    /// reverter of that one line will not be caught by a test.
    #[test]
    fn slope_hypot_divergence_is_measured_not_assumed() {
        use cartalith_jsmath::js_hypot;

        // The nine D8 offsets: identical under both, so the tables need no
        // change.
        for dy in -1i32..=1 {
            for dx in -1i32..=1 {
                let (a, b) = (dx as f64, dy as f64);
                assert_eq!(
                    js_hypot(a, b).to_bits(),
                    a.hypot(b).to_bits(),
                    "D8 offset ({dx},{dy}) must be bit-identical under both hypots"
                );
            }
        }

        // Gradients of the size `build_channels` actually sees: a central
        // difference of two `f32` heights, halved, so O(1e-4)..O(1e-2).
        let mut state = 0x2545_f491_4f6c_dd1du64;
        let mut rng = move || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            (state >> 11) as f64 / (1u64 << 53) as f64
        };
        let (w, density) = (384usize, 1.0f64);
        let thresh = super::river_flow_thresh(w, 288, w, 800.0);
        let (mut differ, mut flips) = (0usize, 0usize);
        const N: usize = 400_000;
        for _ in 0..N {
            let gx = (rng() - 0.5) * 0.02;
            let gy = (rng() - 0.5) * 0.02;
            let (js, rs) = (js_hypot(gx, gy) * w as f64, gx.hypot(gy) * w as f64);
            if js != rs {
                differ += 1;
                // A flow sitting exactly on the JS threshold is the worst
                // case, and the one a real world reaches whenever a cell's
                // accumulation lands between the two thresholds.
                let tj = super::channel_threshold(thresh, js, density);
                let tr = super::channel_threshold(thresh, rs, density);
                if tj != tr {
                    let f = tj.min(tr);
                    if (f <= tj) != (f <= tr) {
                        flips += 1;
                    }
                }
            }
        }
        // The measurement itself is the point; assert only that it ran on a
        // real sample and that the divergence is real rather than zero.
        assert!(differ > 0, "js_hypot and f64::hypot must actually differ on {N} sampled gradients");
        println!("hypot: {differ}/{N} gradients differ, {flips} flip the channelization threshold");
    }

    /// `channel_disc`'s `mag` must be the reference's `Math.min(1, log(f/t)/lmax)`
    /// with **no lower clamp** — see that function's own doc comment for why
    /// the `.max(0.0)` this port carried was wrong and where it is reachable.
    ///
    /// A channel cell with `flow < thresh` is reachable only at
    /// `river_density != 1`, so the fixture drives `channel_disc` directly at
    /// the sub-threshold flow such a cell has.
    #[test]
    fn a_sub_threshold_channel_cell_dims_its_ink_as_the_reference_does() {
        // 64x64, so `lmax = ln(4096*0.05)` is the ~5.3 a real grid gives
        // rather than the degenerate sub-1 a toy grid would.
        let (w, h) = (64usize, 64usize);
        let n = w * h;
        let fld = vec![0.5f32; n];
        let mut flow = vec![0f32; n];
        let order = vec![1i16; n];
        let mid = 32 * w + 32;
        let thresh = 100.0f64;
        flow[mid] = 10.0; // an order-of-magnitude below `thresh`
        let lmax = super::channel_lmax(n);
        let d = super::channel_disc(&fld, &flow, &order, w, h, false, thresh, 1.0, lmax, mid)
            .expect("a positive-flow cell must produce a disc");

        let expect_mag = (10.0f64 / thresh).ln() / lmax;
        assert!(expect_mag < 0.0, "the fixture must actually reach a negative mag, got {expect_mag}");
        assert_eq!(d.mag.to_bits(), expect_mag.to_bits(), "mag must not be floored at 0");
        assert_eq!(
            d.amp.to_bits(),
            (0.45 + expect_mag * 0.7).to_bits(),
            "amp must carry the negative mag through, as Math.min(1, 0.45+0.7*mag) does"
        );
        assert!(d.amp < 0.45, "the reference dims a barely-channelized trickle; the old floor did not");

        // And at the default density a channel cell is above `thresh` by
        // construction, where both forms agree exactly.
        flow[mid] = 1000.0;
        let above = super::channel_disc(&fld, &flow, &order, w, h, false, thresh, 1.0, lmax, mid).unwrap();
        assert!(above.mag > 0.0 && above.amp > 0.45);
    }

    /// The entity itself, on a synthetic tree: three headwater arms and one
    /// trunk running to the south edge. Asserts **shape and non-emptiness**,
    /// not just "no panic" — four subsystems in this port shipped tests that
    /// passed on empty golden output.
    ///
    /// **The trunk is not its own entity, and that is `traceRiverPolylines`'
    /// definition rather than a shortcoming.** A trunk cell has an upstream
    /// donor, so it is never a *source*; the first arm traced walks straight
    /// through the junction and down the trunk to the outlet, and the other
    /// two arms stop on its cells. So there are three runs — one main stem
    /// (arm A + trunk, `order` 2 by the rescan) and two tributaries — which is
    /// exactly what `drawRiverWays` strokes.
    #[test]
    fn river_entities_aggregate_a_confluence_into_a_main_stem_and_two_tributaries() {
        let (w, h) = (9usize, 9usize);
        let n = w * h;
        let mut fld = vec![0f32; n];
        for y in 0..h {
            for x in 0..w {
                fld[y * w + x] = 0.9 - 0.05 * y as f32 + 0.001 * (x as f32 - 4.0).abs();
            }
        }
        let mut recv = vec![-1i32; n];
        let mut order = vec![0i16; n];
        let mut chan = vec![0u8; n];
        let mut mark = |c: usize, r: i32, o: i16| {
            recv[c] = r;
            order[c] = o;
            chan[c] = 1;
        };
        // Arm A (x=2) and arm B (x=6) run down to y=3 and both feed (4,4).
        for y in 0..3 {
            mark(y * w + 2, ((y + 1) * w + 2) as i32, 1);
            mark(y * w + 6, ((y + 1) * w + 6) as i32, 1);
        }
        mark(3 * w + 2, (4 * w + 4) as i32, 1);
        mark(3 * w + 6, (4 * w + 4) as i32, 1);
        // Arm C (x=7) joins the trunk lower down, at (4,6).
        mark(4 * w + 7, (5 * w + 7) as i32, 1);
        mark(5 * w + 7, (6 * w + 4) as i32, 1);
        // The trunk, (4,4) down to the outlet at (4,8).
        for y in 4..h - 1 {
            mark(y * w + 4, ((y + 1) * w + 4) as i32, 2);
        }
        mark((h - 1) * w + 4, -1, 2);

        let flow: Vec<f32> = (0..n).map(|i| if chan[i] != 0 { 10.0 * order[i] as f32 } else { 0.0 }).collect();
        let rivers = super::river_entities(&order, &recv, &flow, &fld, w, h, 1, 1.0, 1.0, false);

        assert_eq!(rivers.len(), 3, "one main stem plus two tributaries");
        // Picked by identity, not by `order`: **all three runs report order 2**,
        // because a tributary's last point IS its trunk's junction cell and
        // `maxO` rescans every point of the run. That is the reference's own
        // `drawRiverWays` loop, verbatim, not a defect here -- pinned so a
        // later "tidy-up" that excludes the shared point knows it is a
        // divergence.
        let ti = rivers.iter().position(|r| r.head == 2).expect("arm A's headwater seeds the main stem");
        let trunk = &rivers[ti];
        assert!(rivers.iter().all(|r| r.order == 2), "every run touches an order-2 cell");
        assert_eq!(trunk.tributaries, 2, "arms B and C both end on it");
        assert!(trunk.length_cells > 0.0, "a >=2-point run has a real length");
        assert_eq!(trunk.mouth as usize, (h - 1) * w + 4, "the main stem's mouth is the outlet cell");
        assert!(trunk.pts.len() >= 2, "every entity is a drawable run");
        assert_eq!(trunk.discharge, flow[trunk.mouth as usize]);
        assert!(trunk.half_width_cells.is_some(), "a positive-flow mouth has a channel width");

        // Every tributary charges exactly one trunk, and no run charges itself.
        let charged: u32 = rivers.iter().map(|r| r.tributaries).sum();
        assert_eq!(charged, 2, "exactly the two arms are tributaries, got {charged}");

        // A tributary's width is its OWN last cell's, not the junction's (the
        // junction is a trunk cell carrying the combined flow). The fixture
        // must be able to tell them apart for this to mean anything.
        let lmax = super::channel_lmax(n);
        let disc = |c: usize| super::channel_disc(&fld, &flow, &order, w, h, false, 1.0, 1.0, lmax, c).map(|d| d.half_w);
        for r in rivers.iter().filter(|r| r.head != 2) {
            let last = r.pts[r.pts.len() - 2];
            let own = (last.1 as usize) * w + last.0 as usize;
            assert_ne!(disc(own), disc(r.mouth as usize), "fixture cannot distinguish the two widths");
            assert_eq!(r.half_width_cells, disc(own), "tributary width is sampled at its own last cell");
        }

        // The pick: a point on the main stem's own line selects it, and a
        // point far off the network selects nothing.
        let p = rivers[ti].pts[1];
        assert_eq!(super::pick_river(&rivers, p.0, p.1, 1.5), Some(ti));
        // Between two cell centres of the same reach -- the segment case that
        // a vertex-only pick would miss.
        let mid = ((rivers[ti].pts[0].0 + p.0) * 0.5, (rivers[ti].pts[0].1 + p.1) * 0.5);
        assert_eq!(super::pick_river(&rivers, mid.0, mid.1, 0.4), Some(ti));
        assert_eq!(super::pick_river(&rivers, 0.5, 0.5, 0.5), None, "bare ground selects nothing");
    }

    /// A world with no channels at all must produce no rivers and refuse a
    /// pick, rather than panicking or returning a phantom entity — the loaded
    /// -save case (`SAVEFILE_COMPAT.md` stores no channel topology).
    #[test]
    fn no_channels_means_no_river_entities() {
        let (w, h) = (8usize, 8usize);
        let n = w * h;
        let rivers = super::river_entities(
            &vec![0i16; n], &vec![-1i32; n], &vec![0f32; n], &vec![0.5f32; n], w, h, 1, 1.0, 1.0, false,
        );
        assert!(rivers.is_empty());
        assert_eq!(super::pick_river(&rivers, 4.0, 4.0, 8.0), None);
        // Short slices are refused, not indexed.
        assert!(super::river_entities(&[0i16; 4], &[-1i32; 4], &[0f32; 4], &[0.5f32; 4], w, h, 1, 1.0, 1.0, false).is_empty());
    }

    /// The whole contract of the radix substitution: **element-identical**,
    /// not merely value-identical. `assert_eq!` on the index vector itself,
    /// because tie order decides the order equal-height cells add their
    /// `f32` discharge into a shared receiver, and float addition is not
    /// associative (`cartalith-rust-conventions`).
    ///
    /// The oracle is the comparison sort this replaced, verbatim:
    /// `sort_by(flow_cmp_desc(field[a], field[b]).then(a.cmp(&b)))`.
    ///
    /// The fixtures are shaped to *reach* the two quirks rather than to look
    /// varied. `field` is `f32` throughout, which is what `WorldState.field`
    /// is — the reference's `Float32Array`.
    #[test]
    fn flow_sort_desc_is_element_identical_to_the_comparison_sort() {
        let oracle = |field: &[f32]| -> Vec<u32> {
            let mut order: Vec<u32> = (0..field.len() as u32).collect();
            order.sort_by(|&a, &b| flow_cmp_desc(field[a as usize], field[b as usize]).then(a.cmp(&b)));
            order
        };

        let cases: Vec<(&str, Vec<f32>)> = vec![
            ("empty", vec![]),
            ("one", vec![0.5]),
            // Negative zero next to positive zero, in both orders, with a
            // duplicate of each: the one quirk the reference calls out by
            // name. `total_cmp` alone would order these by sign.
            ("signed zeros", vec![-0.0, 0.0, -0.0, 0.5, 0.0, -0.0, -0.25, 0.0]),
            ("all zeros, mixed sign", vec![-0.0, 0.0, -0.0, -0.0, 0.0, 0.0, -0.0, 0.0]),
            // Every element tied: the sort degenerates to "is it stable?".
            ("all equal", vec![0.375f32; 64]),
            // Many ties in runs, so a non-stable radix would scramble
            // within a run without changing any *value*.
            (
                "long tied runs",
                (0..300).map(|i| ((i / 7) as f32) * 0.125 - 8.0).collect(),
            ),
            // Signs, subnormals, infinities and NaN (both signs). NaN cannot
            // occur in a generated `field` -- the reference asserts as much
            // ("all fields are finite, Invariant 2") -- but the two orderings
            // must still agree on it, or the equivalence claim is narrower
            // than it reads.
            (
                "the awkward IEEE values",
                vec![
                    f32::NAN,
                    -f32::NAN,
                    f32::INFINITY,
                    f32::NEG_INFINITY,
                    f32::MIN_POSITIVE,
                    -f32::MIN_POSITIVE,
                    f32::from_bits(1),
                    f32::from_bits(0x8000_0001),
                    0.0,
                    -0.0,
                    1.0,
                    -1.0,
                    f32::MAX,
                    f32::MIN,
                ],
            ),
            // Quantised heights: a real heightmap's ties come from a coarse
            // lattice, not from bit-identical randomness.
            (
                "quantised lattice",
                (0..1024).map(|i| ((i * 37) % 19) as f32 / 19.0).collect(),
            ),
            // A monotone ramp and its reverse: no ties at all, so any
            // failure here is the key transform, not the stability.
            ("ramp up", (0..500).map(|i| i as f32 * 1e-3 - 0.25).collect()),
            ("ramp down", (0..500).map(|i| 0.25 - i as f32 * 1e-3).collect()),
        ];

        for (label, field) in &cases {
            let got = flow_sort_desc(field, field.len());
            assert_eq!(got, oracle(field), "{label}");
        }

        // A pseudo-random f32 field spanning the whole exponent range, so
        // every byte of every key actually varies -- the ten curated cases
        // above are narrow by design and would not catch a wrong shift.
        let mut s = 0x12345678u32;
        let mut wide: Vec<f32> = Vec::with_capacity(5000);
        for _ in 0..5000 {
            s ^= s << 13;
            s ^= s >> 17;
            s ^= s << 5;
            let v = f32::from_bits(s);
            wide.push(if v.is_finite() { v } else { (s as f32) * 1e-9 });
        }
        assert_eq!(flow_sort_desc(&wide, wide.len()), oracle(&wide), "wide random");

        // And the same field quantised hard, to force thousands of ties on
        // top of that spread.
        let tied: Vec<f32> = wide.iter().map(|v| (v.signum() * (v.abs().log2().floor())).max(-40.0)).collect();
        assert_eq!(flow_sort_desc(&tied, tied.len()), oracle(&tied), "wide random, quantised");
    }


    // ---- Math.atan2 fidelity (JS_SEMANTICS_AUDIT.md §4.4) ----------------
    //
    // Every expectation below was read off `node` v24.19.0 as raw IEEE-754
    // bits, never from a paraphrase of ECMA-262 — the audit's §5
    // recommendation, written after a `toFixed` unit test spent two
    // milestones asserting a bug it had reasoned its way into.

    /// The whole reason `js_atan2` exists: `build_channels`'s receiver
    /// argmax picks a **different cell** on a real input, and the cell
    /// `f64::atan2` picks is the wrong one.
    ///
    /// The mechanism is structural, not a freak coincidence. A cell whose
    /// 3x3 is left-right symmetric has `gx == 0.0` exactly, so
    /// `aspect = atan2(-gy, -0.0)` lands on the signed-zero branch and
    /// comes out at exactly `-pi/2`. Its two downhill diagonals then have
    /// **exactly equal** `drop`, and `|wrap(atan2(dy,dx) - aspect)|` is
    /// mathematically `3*pi/4` for both — so the argmax is decided purely
    /// by which of two last bits comes out larger, and `f64::atan2` and V8
    /// break that tie differently. `score > best_score` is strict, so the
    /// tie goes to whichever neighbour the loop reached first.
    ///
    /// Both fixtures below are 3x3 grids, which makes the centre cell's
    /// 3x3 neighbourhood the whole grid and the block index equal to the
    /// grid index. `flow` is zero everywhere but the centre, so only the
    /// centre channelizes and `recv` is `-1` elsewhere — the assertion is
    /// on one number, the receiver.
    ///
    /// Expected receivers were computed by running the reference's own
    /// `buildRiverNetwork` channelization loop (HTML lines 4504-4525),
    /// transcribed verbatim, under `node` v24.19.0 on these exact `f32`
    /// bit patterns. Before the `js_atan2` change both cases returned `8`;
    /// V8 returns `6`.
    #[test]
    fn build_channels_receiver_follows_v8_not_rust_atan2() {
        // (a) A near-flat plateau cell — ordinary generated-terrain f32
        // values, symmetric to the bit in the left/right pairs.
        // Shortest round-tripping `f32` literals; each is bit-identical to
        // the value the search produced.
        let field: Vec<f32> = vec![
            0.5790264, 0.57902455, 0.5790278, 0.5790286, 0.5790234, 0.5790286, 0.5790227, 0.5790266,
            0.5790227,
        ];
        let flow = vec![0.0f32, 0.0, 0.0, 0.0, 1.0e9, 0.0, 0.0, 0.0, 0.0];
        let r = build_channels(&field, &flow, 3, 3, 0.0, false, 1.0, 800.0);
        assert_eq!(r.chan, vec![0u8, 0, 0, 0, 1, 0, 0, 0, 0], "only the centre channelizes");
        assert_eq!(
            r.recv,
            vec![-1i32, -1, -1, -1, 6, -1, -1, -1, -1],
            "V8 steers the centre cell into cell 6; f64::atan2 steers it into cell 8"
        );

        // (b) The same mechanism on exactly-representable heights, so the
        // symmetry is obvious by eye: columns 0 and 2 are equal in the top
        // and bottom rows, and the middle row is flat.
        let field: Vec<f32> = vec![0.8125, 0.5, 0.5625, 0.25, 0.25, 0.25, 0.125, 0.625, 0.125];
        let r = build_channels(&field, &flow, 3, 3, 0.0, false, 1.0, 800.0);
        assert_eq!(r.recv[4], 6, "V8 picks cell 6; f64::atan2 picks cell 8");
    }

    #[test]
    fn crate_compiles_and_tests_run() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn enforce_river_channels_clamps_only_raised_masked_cells() {
        let mut field = vec![0.9f32, 0.9, 0.9, 0.1];
        let mask = vec![1u8, 0, 1, 1];
        let floor = vec![0.3f32, 0.3, 0.3, 0.3];
        enforce_river_channels(&mut field, &mask, &floor);
        assert_eq!(field[0], 0.3, "masked and raised -> clamped");
        assert_eq!(field[1], 0.9, "unmasked -> untouched");
        assert_eq!(field[2], 0.3);
        assert_eq!(field[3], 0.1, "already below the floor -> kept deeper");
    }

    #[test]
    fn enforce_river_channels_is_a_no_op_on_an_empty_mask() {
        let mut field = vec![0.9f32; 4];
        let before = field.clone();
        enforce_river_channels(&mut field, &[0u8; 4], &[0f32; 4]);
        assert_eq!(field, before);
    }

    /// The owner's requirement, pinned: *"As soon as the map width/size becomes
    /// lower the size width and length of a river should become bigger and
    /// more visible."*
    ///
    /// `width_k` is `river_width_scale_k(map_width_km)` -- 1.0 at 800 km, 4.0
    /// at 200 km, 16.0 at 50 km -- so a strictly larger `width_k` must ink a
    /// strictly larger area. Before this stamp existed the renderer tested
    /// `chan[i] != 0` and the answer was the same single cell at every extent,
    /// which is exactly what this test would now catch.
    #[test]
    fn a_smaller_map_stamps_a_wider_river() {
        // A flat 41x41 world with one channel cell dead centre. Flat on
        // purpose: `slope_fac` is 1/(1+5*|grad|*w), so a gradient would damp
        // the very term under test and could hide a regression.
        let (w, h) = (41usize, 41usize);
        let n = w * h;
        let mid = (h / 2) * w + w / 2;
        let fld = vec![0.5f32; n];
        let mut flow = vec![0f32; n];
        let mut chan = vec![0u8; n];
        let recv = vec![-1i32; n]; // the sole channel cell has no receiver -- irrelevant to width
        let mut order = vec![0i16; n];
        let thresh = 4.0f64;
        flow[mid] = 4000.0; // well above thresh, so `mag` is near its ceiling
        chan[mid] = 1;
        order[mid] = 5;

        // `area_bar = 0.0`: this test is about `width_k`, not the v2.72 area
        // gate, so the gate stays off (its own coverage is
        // `a_channel_cell_below_the_area_bar_does_not_ink` below).
        let inked = |k: f64| -> usize {
            super::stamp_river_intensity(&fld, &flow, &chan, &recv, &order, w, h, false, thresh, k, 0.0)
                .iter()
                .filter(|&&v| v > 0.0)
                .count()
        };

        let world_scale = inked(1.0); // 800 km
        let regional = inked(4.0); //  200 km
        let local = inked(16.0); //   50 km

        assert!(world_scale > 0, "the stamp must ink at least the channel cell itself");
        assert!(
            regional > world_scale,
            "200 km ({regional} cells) must ink more than 800 km ({world_scale})"
        );
        assert!(
            local > regional,
            "50 km ({local} cells) must ink more than 200 km ({regional})"
        );

        // And the ink is a falloff, not a flat disc -- the centre is the
        // brightest cell, which is what gives a wide river a soft bank
        // instead of a hard edge.
        let v = super::stamp_river_intensity(&fld, &flow, &chan, &recv, &order, w, h, false, thresh, 16.0, 0.0);
        let peak = v.iter().cloned().fold(0.0f32, f32::max);
        assert_eq!(v[mid], peak, "the channel cell itself must carry the peak ink");
        assert!(peak > 0.0 && peak <= 1.0, "ink stays in [0,1], got {peak}");
    }

    /// A cell with no channel must ink nothing at all, and an empty/mismatched
    /// input must not panic -- the renderer calls this on whatever a world
    /// happens to carry.
    #[test]
    fn no_channels_means_no_ink_and_bad_input_is_refused_quietly() {
        let (w, h) = (8usize, 8usize);
        let n = w * h;
        let out = super::stamp_river_intensity(
            &vec![0.5f32; n], &vec![1.0f32; n], &vec![0u8; n], &vec![-1i32; n], &vec![0i16; n], w, h, false, 1.0, 4.0,
            0.0,
        );
        assert_eq!(out.len(), n);
        assert!(out.iter().all(|&v| v == 0.0), "no channel cells must ink nothing");

        // Short slices: return a correctly-sized zero grid rather than panic.
        let short = super::stamp_river_intensity(
            &[0.5f32; 4], &[1.0f32; 4], &[1u8; 4], &[-1i32; 4], &[1i16; 4], w, h, false, 1.0, 4.0, 0.0,
        );
        assert_eq!(short.len(), n);
        assert!(short.iter().all(|&v| v == 0.0));
    }

    /// `RIVER_RENDER_AREA_K` pinned as a **literal**, not against itself
    /// (`MISTAKES.md`'s "write a test that pins a constant" row) -- this is
    /// the reference's own v2.72 literal, carried verbatim.
    #[test]
    fn river_render_area_k_is_the_references_literal_two() {
        assert_eq!(super::RIVER_RENDER_AREA_K, 2.0);
    }

    /// [`super::river_render_area_bar`] at the default extent and at the
    /// reference's own measured large-extent case, pinned as literals.
    /// `river_coarse_ease` is a no-op (`1.0`) at/below 800 km and caps at
    /// `16.0`, so the bar is `2.0` at 800 km and `32.0` at 40 000 km (already
    /// past the 12 800 km point the ease saturates).
    #[test]
    fn river_render_area_bar_scales_with_the_same_ease_river_flow_thresh_uses() {
        assert_eq!(super::river_render_area_bar(800.0), 2.0);
        assert_eq!(super::river_render_area_bar(40_000.0), 32.0);
        // Below the reference default: still the no-op floor, same as
        // `river_coarse_ease` itself.
        assert_eq!(super::river_render_area_bar(200.0), 2.0);
    }

    /// The core claim under test for the v2.72 "a detection ease is not a
    /// display threshold" fix: a channelized cell with no channelized
    /// upstream neighbour of its own (`area == 1`) must not ink once
    /// `area_bar` is active, while the cell it drains into -- whose own
    /// accumulated drainage clears the bar -- still does.
    ///
    /// `head` and `mouth` are placed far apart in the grid (not spatial
    /// neighbours) specifically so neither cell's disc can physically
    /// overlap the other's coordinate -- what is under test is
    /// [`super::channel_cell_drainage`]'s topology-only gate, not disc
    /// geometry, and a false pass from disc overlap would be exactly the
    /// silently-wrong-oracle `MISTAKES.md` warns against.
    #[test]
    fn a_channel_cell_below_the_area_bar_does_not_ink_but_its_receiver_does() {
        let (w, h) = (21usize, 21usize);
        let n = w * h;
        let fld = vec![0.5f32; n];
        let mut flow = vec![0f32; n];
        let mut chan = vec![0u8; n];
        let mut recv = vec![-1i32; n];
        let order = vec![1i16; n];
        let thresh = 4.0f64;

        let head = 2 * w + 2; // upstream: area == 1 (no channelized upstream donor)
        let mouth = 18 * w + 18; // downstream: area == 2 (head drains into it)
        flow[head] = 50.0; // smaller flow -> visited first in ascending order (upstream)
        flow[mouth] = 200.0;
        chan[head] = 1;
        chan[mouth] = 1;
        recv[head] = mouth as i32;

        let area = super::channel_cell_drainage(&recv, &flow, &chan);
        assert_eq!(area[head], 1, "a headwater cell with no channelized donor must read area == 1");
        assert_eq!(area[mouth], 2, "the receiver must accumulate its own donor's count");

        let area_bar = 2.0; // river_render_area_bar(800.0)
        let out = super::stamp_river_intensity(&fld, &flow, &chan, &recv, &order, w, h, false, thresh, 1.0, area_bar);
        assert_eq!(out[head], 0.0, "area 1 < area_bar 2.0 must not ink");
        assert!(out[mouth] > 0.0, "area 2 >= area_bar 2.0 must still ink");

        // And with the gate off (`area_bar == 0.0`, the pre-v2.72 behaviour),
        // both cells ink -- proving the difference above is the gate, not
        // some other input.
        let ungated = super::stamp_river_intensity(&fld, &flow, &chan, &recv, &order, w, h, false, thresh, 1.0, 0.0);
        assert!(ungated[head] > 0.0, "with the gate off, the headwater cell must ink too");
    }

    /// v2.72's *second* reference fix — "cut the stem where it wraps"
    /// (`splitRiverPolylines` applied to the raster path) — measured against
    /// the disc-stamp technique, on a **real generated wrap regime**, not by
    /// reasoning about the code shape alone (`MISTAKES.md`: "write an oracle
    /// for a ported function" and the general discipline of measuring before
    /// concluding).
    ///
    /// A world-wrapped field with a single valley straddling the antimeridian
    /// (`1 - cos(2*pi*x/w)`, periodic, so the terrain is smooth and
    /// physically continuous across the `x=0`/`x=w-1` seam): run the real
    /// [`super::compute_flow`] and [`super::build_channels`] pipeline with
    /// `world: true`, exactly as `generate_terrain` does, and confirm two
    /// things at once —
    ///
    /// 1. the receiver tree genuinely crosses the seam (a channel cell near
    ///    `x=w-1` whose receiver sits near `x=0`), so the wrap regime is
    ///    actually reached and this is not a vacuous pass; and
    /// 2. [`super::stamp_river_intensity`] paints **nothing** in the middle
    ///    of the map, far from either side of the valley — which is what the
    ///    reference's bug would have failed: a stroked polyline renderer
    ///    draws a straight line between two consecutive points regardless of
    ///    their distance in flat pixel-space, so a wrap-crossing step there
    ///    paints one long band clean across the map. The disc stamp has no
    ///    such step: each channel cell inks a small, locally-bounded disc
    ///    around its own coordinate (`stamp_river_intensity`'s inner loop
    ///    clamps to `[0, w-1]`/`[0, h-1]`, never wraps, and never draws
    ///    between two cells at all) — this test is the empirical
    ///    confirmation that conclusion actually holds on generated output,
    ///    not just on a reading of the loop bounds.
    #[test]
    fn wrap_crossing_receivers_do_not_paint_a_line_across_the_map() {
        let (w, h) = (101usize, 101usize);
        let n = w * h;
        let sea = 0.3f64;
        let band = 10.0f64; // slope band width in cells, each side of the seam

        // A narrow valley straddling the seam (minimum at x=0, which is also
        // x=w since it wraps) and a flat plateau everywhere else: every row
        // is `0.5 + 0.02*min(d, band)` where `d` is the wrapped distance to
        // the seam. Only the `band`-wide strip on each side of x=0 has any
        // slope at all -- the plateau is exactly flat, so `d8_receiver`
        // (strictly-greater-drop-required) finds no receiver there and it
        // stays unchannelized, giving a real, honest "far from the river"
        // middle to check rather than one hand-picked to be empty.
        let mut fld = vec![0f32; n];
        for y in 0..h {
            for x in 0..w {
                let d = x.min(w - x) as f64; // wrapped distance to the seam
                fld[y * w + x] = (0.5 + 0.02 * d.min(band)) as f32;
            }
        }
        assert!(fld.iter().all(|&v| (v as f64) > sea), "fixture must be all-land, or sea masking hides the seam cells");

        let flow = super::compute_flow(w, h, &fld, None, false, true);
        let ch = super::build_channels(&fld, &flow, w, h, sea, true, 1.0, 800.0);
        let chan_count = ch.chan.iter().filter(|&&c| c != 0).count();
        assert!(chan_count > 0, "the fixture must actually channelize somewhere, or this test is vacuous");

        // Condition 1: the wrap regime is genuinely reached -- a channelized
        // cell whose receiver's x is on the *other* side of the seam from its
        // own x (a jump of more than half the grid width, the same test
        // `split_river_polylines` itself uses to detect a wrap).
        let half = w as f64 * 0.5;
        let wrap_edge = (0..n).find(|&i| {
            if ch.chan[i] == 0 {
                return false;
            }
            let r = ch.recv[i];
            if r < 0 {
                return false;
            }
            let (xi, xr) = ((i % w) as f64, (r as usize % w) as f64);
            (xi - xr).abs() > half
        });
        assert!(
            wrap_edge.is_some(),
            "the fixture must produce at least one receiver that crosses the seam, or the wrap regime was never reached \
             (chan_count={chan_count})"
        );

        let order = super::strahler_from_receivers(&ch.recv, &flow, &ch.chan);
        let thresh = super::river_flow_thresh(w, h, w, 800.0);
        let width_k = super::river_width_scale_k(800.0);
        let area_bar = super::river_render_area_bar(800.0);
        let intensity = super::stamp_river_intensity(&fld, &flow, &ch.chan, &ch.recv, &order, w, h, true, thresh, width_k, area_bar);

        // Condition 2: nothing paints in the middle of the map, deep in the
        // flat plateau and far (>30 cells, well past the disc's own 9-cell
        // cap) from either edge of the slope band.
        let mid_inked: Vec<usize> = (0..h)
            .flat_map(|y| (40..=60).map(move |x| y * w + x))
            .filter(|&i| intensity[i] > 0.0)
            .collect();
        assert!(
            mid_inked.is_empty(),
            "a wrap-crossing receiver painted ink in the map's middle (x in 40..=60): {} cells, e.g. index {:?} -- \
             this is exactly the defect Fix 2 exists to prevent",
            mid_inked.len(),
            mid_inked.first()
        );

        // And the valley itself, on both sides of the seam, does ink -- so
        // the empty middle above is a real finding and not an all-zero stamp.
        let seam_inked = (0..h).any(|y| intensity[y * w] > 0.0 || intensity[y * w + w - 1] > 0.0);
        assert!(seam_inked, "the valley straddling the seam must ink on at least one side, or nothing channelized there");
    }

    /// The port's own defect (`RC_ENGINE_CHANGES.md` §6l, ported forward
    /// unfixed until now): each channel cell stamps its disc independently of
    /// its neighbours in the receiver chain, so a diagonal receiver step
    /// between two `half_w < 1.0` cells left two discs that never touch, not
    /// even at a shared corner.
    ///
    /// # Proving the gap exists before the fix
    ///
    /// A disc only inks a cell at grid distance `d <= half_w`. The nearest
    /// possible *other* grid cell is `d == 1` (orthogonal); a diagonal
    /// neighbour is `d == sqrt(2)`. So **`half_w < 1.0` is not a heuristic
    /// trigger, it is a mathematical guarantee that a disc centred on cell
    /// `i` paints nothing but `i` itself** — no separate "run the old code"
    /// step is possible (the old, unbridged loop no longer exists to run),
    /// so this test establishes the gap the same way the fix's own doc
    /// comment does: by computing each fixture cell's real `half_w` through
    /// [`super::channel_disc`] (the exact function the stamp loop calls) and
    /// asserting it is under 1.0, which makes "the two centres' own discs
    /// cannot reach each other or the cell between them" a certainty rather
    /// than an assumption.
    ///
    /// # The fixture
    ///
    /// A flat field (`slope_fac == 1`, so `half_w` is driven by `mag` alone)
    /// with four channel cells on a straight diagonal, `(5,5)->(6,6)->(7,7)
    /// ->(8,8)`, `recv` chained exactly along that diagonal (the same D8 step
    /// shape the reference measured at ~42% of a real chain) and `flow` just
    /// above `thresh` at each, which keeps `mag` small and `half_w` in
    /// `[0.6, 0.66]` -- comfortably under the 1.0 bound the proof above needs,
    /// and nowhere near `channel_disc`'s own 0.5 floor, so this is an ordinary
    /// reachable case, not an edge one.
    #[test]
    fn diagonal_channel_steps_are_bridged_to_stay_4_connected() {
        let (w, h) = (20usize, 20usize);
        let n = w * h;
        let fld = vec![0.5f32; n]; // flat: gx=gy=0, slope_fac=1 exactly
        let mut flow = vec![0f32; n];
        let mut chan = vec![0u8; n];
        let mut recv = vec![-1i32; n];
        let order = vec![1i16; n];

        let chain = [(5usize, 5usize), (6, 6), (7, 7), (8, 8)];
        let flows = [120.0f32, 130.0, 140.0, 150.0]; // all > thresh, mag small and positive
        let idx = |x: usize, y: usize| y * w + x;
        for (k, &(x, y)) in chain.iter().enumerate() {
            let i = idx(x, y);
            chan[i] = 1;
            flow[i] = flows[k];
            recv[i] = if k + 1 < chain.len() {
                let (nx, ny) = chain[k + 1];
                idx(nx, ny) as i32
            } else {
                -1 // the chain's own pit
            };
        }

        let thresh = 100.0f64;
        let width_k = 1.0f64;
        let lmax = super::channel_lmax(n);

        // The proof: every fixture cell's own half-width is under 1.0, so its
        // disc cannot reach any other grid cell on its own.
        for &(x, y) in &chain {
            let i = idx(x, y);
            let d = super::channel_disc(&fld, &flow, &order, w, h, false, thresh, width_k, lmax, i)
                .expect("every fixture cell has positive flow and must produce a disc");
            assert!(
                d.half_w < 1.0,
                "fixture cell ({x},{y}) must have half_w < 1.0 for the proof to hold, got {}",
                d.half_w
            );
        }

        let intensity = super::stamp_river_intensity(&fld, &flow, &chan, &recv, &order, w, h, false, thresh, width_k, 0.0);

        // Each centre still inks itself.
        for &(x, y) in &chain {
            assert!(intensity[idx(x, y)] > 0.0, "channel cell ({x},{y}) must ink its own centre");
        }

        // The bridge cells the fix adds: for the (5,5)->(6,6) step, the two
        // cells 4-adjacent to both endpoints are (6,5) and (5,6). Grid
        // distance from either endpoint to either bridge cell is exactly 1,
        // which is `> half_w` for every fixture cell proven above -- so
        // under disc-only painting these would be 0. After the fix, at least
        // one must be > 0 (the doc comment raises both, so assert both here
        // to pin that, not just "at least one").
        assert!(intensity[idx(6, 5)] > 0.0, "bridge cell (6,5) between (5,5) and (6,6) must be inked");
        assert!(intensity[idx(5, 6)] > 0.0, "bridge cell (5,6) between (5,5) and (6,6) must be inked");
        assert!(intensity[idx(7, 6)] > 0.0, "bridge cell (7,6) between (6,6) and (7,7) must be inked");
        assert!(intensity[idx(6, 7)] > 0.0, "bridge cell (6,7) between (6,6) and (7,7) must be inked");
        assert!(intensity[idx(8, 7)] > 0.0, "bridge cell (8,7) between (7,7) and (8,8) must be inked");
        assert!(intensity[idx(7, 8)] > 0.0, "bridge cell (7,8) between (7,7) and (8,8) must be inked");

        // The bridge value must never exceed the dimmer of the two centres it
        // connects -- the fix must not brighten a river, only connect it.
        for w2 in chain.windows(2) {
            let (ax, ay) = w2[0];
            let (bx, by) = w2[1];
            let dimmer = intensity[idx(ax, ay)].min(intensity[idx(bx, by)]);
            assert!(
                intensity[idx(bx, ay)] <= dimmer + f32::EPSILON,
                "bridge ({bx},{ay}) must not exceed the dimmer of its two endpoints"
            );
            assert!(
                intensity[idx(ax, by)] <= dimmer + f32::EPSILON,
                "bridge ({ax},{by}) must not exceed the dimmer of its two endpoints"
            );
        }

        // Whole-chain 4-connectivity: flood-fill from the first channel cell
        // over every nonzero-intensity cell and confirm all four chain cells
        // land in the same connected component.
        let mut seen = vec![false; n];
        let mut stack = vec![idx(chain[0].0, chain[0].1)];
        seen[stack[0]] = true;
        while let Some(i) = stack.pop() {
            let (x, y) = (i % w, i / w);
            let mut push4 = |nx: i64, ny: i64| {
                if nx < 0 || ny < 0 || nx as usize >= w || ny as usize >= h {
                    return;
                }
                let j = ny as usize * w + nx as usize;
                if !seen[j] && intensity[j] > 0.0 {
                    seen[j] = true;
                    stack.push(j);
                }
            };
            push4(x as i64 - 1, y as i64);
            push4(x as i64 + 1, y as i64);
            push4(x as i64, y as i64 - 1);
            push4(x as i64, y as i64 + 1);
        }
        for &(x, y) in &chain {
            assert!(
                seen[idx(x, y)],
                "channel cell ({x},{y}) must be 4-connected to the rest of the chain through nonzero intensity"
            );
        }
    }

    /// [`super::wrapped_axis_delta`] is what tells the connectivity bridge a
    /// genuine one-cell diagonal step (short via the wrap) from any other
    /// receiver relationship, and it must give the *same* short-distance
    /// answer the existing wrap-crossing test's own seam-detection code uses
    /// (`(xi - xr).abs() > half`) — this test exercises the bridge itself,
    /// not just the helper in isolation, on a receiver step that actually
    /// crosses the seam.
    #[test]
    fn diagonal_step_across_the_wrap_seam_is_bridged_without_reaching_the_map_middle() {
        let (w, h) = (10usize, 10usize);
        let n = w * h;
        let fld = vec![0.5f32; n];
        let mut flow = vec![0f32; n];
        let mut chan = vec![0u8; n];
        let mut recv = vec![-1i32; n];
        let order = vec![1i16; n];
        let idx = |x: usize, y: usize| y * w + x;

        // (9,5) -> (0,6): a diagonal step only because x wraps (raw dx is -9,
        // wrapped it is +1).
        let a = (9usize, 5usize);
        let b = (0usize, 6usize);
        chan[idx(a.0, a.1)] = 1;
        chan[idx(b.0, b.1)] = 1;
        flow[idx(a.0, a.1)] = 120.0;
        flow[idx(b.0, b.1)] = 130.0;
        recv[idx(a.0, a.1)] = idx(b.0, b.1) as i32;
        recv[idx(b.0, b.1)] = -1;

        let thresh = 100.0f64;
        let width_k = 1.0f64;
        let lmax = super::channel_lmax(n);
        for &(x, y) in &[a, b] {
            let d = super::channel_disc(&fld, &flow, &order, w, h, true, thresh, width_k, lmax, idx(x, y)).unwrap();
            assert!(d.half_w < 1.0, "fixture cell ({x},{y}) must have half_w < 1.0, got {}", d.half_w);
        }

        let intensity = super::stamp_river_intensity(&fld, &flow, &chan, &recv, &order, w, h, true, thresh, width_k, 0.0);

        // The bridge cells for a wrapped (9,5)->(0,6) step are (0,5) and
        // (9,6) -- both 4-adjacent to (9,5) through the wrap and to (0,6)
        // directly or through the wrap.
        assert!(intensity[idx(0, 5)] > 0.0, "wrap bridge cell (0,5) must be inked");
        assert!(intensity[idx(9, 6)] > 0.0, "wrap bridge cell (9,6) must be inked");

        // And the bridge must not have painted anywhere near the map's own
        // middle column -- confirming `wrapped_axis_delta` picked the short
        // wrap distance (1), not the long raw one (9), which would place a
        // bridge cell far from the seam.
        for y in 0..h {
            assert_eq!(intensity[idx(4, y)], 0.0, "the wrap bridge must stay local to the seam, not reach column 4");
            assert_eq!(intensity[idx(5, y)], 0.0, "the wrap bridge must stay local to the seam, not reach column 5");
        }
    }

    /// `enforce_channel_descent`'s carve radius is `Math.hypot(x-px, y-py)`
    /// (reference line 8733), and this port wrote `f64::hypot` until today.
    ///
    /// **Small integers are not safe here.** Over the 400 offset pairs in
    /// `0..19` the two hypots differ on 108; `(2,3)` is one of them —
    /// 3.6055512754639896 under V8, 3.605551275463989 correctly rounded, V8's
    /// being the *larger*.
    ///
    /// So the fixture sets `half_w` to V8's own value for that offset, which
    /// puts the eight `(±2,±3)/(±3,±2)` cells exactly **on** the rim: under
    /// `js_hypot` `d == half_w`, so `t == 1.0`, `target == fld[i]`, and
    /// `target < fld[i]` is false — the rim is not carved and not returned.
    /// Under `f64::hypot` `d` is one ulp *below* `half_w`, `t < 1`, and all
    /// eight get carved. 36 cells versus 44: a discrete difference in the
    /// terrain this writes, not a float epsilon a later `f32` store absorbs.
    #[test]
    fn enforce_channel_descent_carves_the_v8_hypot_disc() {
        use cartalith_jsmath::js_hypot;

        let (dx, dy) = (2.0f64, 3.0f64);
        let half_w = js_hypot(dx, dy);
        assert_ne!(
            half_w.to_bits(),
            dx.hypot(dy).to_bits(),
            "the fixture must sit on an offset the two hypots disagree about"
        );
        assert!(half_w > dx.hypot(dy), "V8 is the larger here, which is what puts the rim outside");

        let (w, h) = (16usize, 16usize);
        let n = w * h;
        let (px, py) = (8usize, 8usize);
        let mut fld = vec![0.5f32; n];
        fld[py * w + px] = 0.2; // the centreline sits below its banks, so the disc carves
        let sea = 0.0f64; // floor_lim = -0.06, well below the 0.2 floor

        let carved = super::enforce_channel_descent(&mut fld, w, h, &[(px as f64, py as f64)], sea, half_w, 0.0006);

        // The rim: eight cells that `f64::hypot` would carve and V8 does not.
        for &(ox, oy) in &[(2i64, 3i64), (-2, 3), (2, -3), (-2, -3), (3, 2), (-3, 2), (3, -2), (-3, -2)] {
            let i = (py as i64 + oy) as usize * w + (px as i64 + ox) as usize;
            assert_eq!(
                fld[i], 0.5,
                "({ox},{oy}) is exactly on the V8 rim and must keep its terrain height"
            );
            assert!(!carved.contains(&i), "({ox},{oy}) must not be reported as carved");
        }

        // Non-emptiness and the exact count, so a revert cannot pass by
        // carving *more*: 36 under `js_hypot`, 44 under `f64::hypot`.
        assert_eq!(carved.len(), 36, "the V8 disc carves 36 cells; the f64::hypot disc carves 44");
        let inside = (py + 1) * w + px + 1;
        assert!(fld[inside] < 0.5 && carved.contains(&inside), "cells well inside the rim are still carved");
        assert!(fld[py * w + px] <= 0.2, "the centreline is never raised");
    }

    /// The channel-width law, bit-for-bit against reference lines 4532-4537 —
    /// the pin `slope_hypot_divergence_is_measured_not_assumed` never was.
    ///
    /// The fixture's gradient is one of the 33 475-in-160 000 `f32` height
    /// pairs whose central differences make V8's `Math.hypot` and
    /// `f64::hypot` disagree, and `width_k = 4` (200 km) keeps the result off
    /// both clamps, so the disagreement survives into `half_w` instead of
    /// being clamped or rounded away. Three `assert_ne!`s state what the
    /// fixture discriminates rather than leaving it to be assumed:
    ///
    /// - `f64::hypot` in `slope_fac` — the revert that scored green before,
    /// - `5.0` → `6.0` in `slope_fac` — a survived mutant,
    /// - `mag`'s `0.05` inside `channel_lmax` — reached through `mag²`.
    #[test]
    fn channel_disc_width_law_is_bit_exact_against_the_reference() {
        use cartalith_jsmath::{js_hypot, js_min};

        let (w, h) = (64usize, 64usize);
        let n = w * h;
        let (cx, cy) = (32usize, 32usize);
        let i = cy * w + cx;

        let mut fld = vec![0.5f32; n];
        fld[cy * w + cx + 1] = 0.50003; // gx = 1.4990568161010742e-5
        fld[(cy + 1) * w + cx] = 0.50019; // gy = 9.500980377197266e-5
        let mut flow = vec![0f32; n];
        flow[i] = 1500.0;
        let mut order = vec![1i16; n];
        order[i] = 3;
        let (thresh, width_k) = (100.0f64, 4.0f64);
        let lmax = super::channel_lmax(n);

        let d = super::channel_disc(&fld, &flow, &order, w, h, false, thresh, width_k, lmax, i)
            .expect("a positive-flow cell must produce a disc");

        // Reference 4534/4532, transcribed: central differences, then `mag`.
        let gx = (fld[cy * w + cx + 1] as f64 - fld[cy * w + cx - 1] as f64) * 0.5;
        let gy = (fld[(cy + 1) * w + cx] as f64 - fld[(cy - 1) * w + cx] as f64) * 0.5;
        let mag = js_min(1.0, (1500.0f64 / thresh).ln() / lmax);
        assert!(mag > 0.0 && mag < 1.0, "the fixture must reach the interior of `mag`, got {mag}");

        // Reference 4535/4536, in the reference's own operation order.
        let law = |k: f64, hyp: f64| {
            (0.6 + 3.0 * mag * mag + 0.45 * (3.0 - 1.0)) * (1.0 / (1.0 + k * hyp * w as f64)) * width_k
        };
        let (js, rs) = (js_hypot(gx, gy), gx.hypot(gy));
        assert_ne!(js.to_bits(), rs.to_bits(), "the fixture gradient must actually split the two hypots");

        let expect = law(5.0, js);
        assert!(expect > 0.5 && expect < 9.0 * width_k, "the fixture must sit off both clamps, got {expect}");
        assert_eq!(
            d.half_w.to_bits(),
            expect.to_bits(),
            "half_w must be (0.6+3*mag^2+0.45*(o-1))*slope_fac*width_k with V8's hypot"
        );

        assert_ne!(d.half_w.to_bits(), law(5.0, rs).to_bits(), "f64::hypot in slope_fac would change half_w");
        assert_ne!(d.half_w.to_bits(), law(6.0, js).to_bits(), "slope_fac's 5.0 is load-bearing");
        let wrong_lmax = js_min(1.0, (1500.0f64 / thresh).ln() / (0.06 * n as f64).ln());
        assert_ne!(
            d.half_w.to_bits(),
            ((0.6 + 3.0 * wrong_lmax * wrong_lmax + 0.9) * (1.0 / (1.0 + 5.0 * js * w as f64)) * width_k).to_bits(),
            "channel_lmax's 0.05 reaches half_w through mag^2"
        );
    }

    /// `if(halfW<0.5)halfW=0.5` (reference line 4536) — the floor, and the
    /// fact that it is **not** scaled by `width_k`, which is what keeps a
    /// world-scale map's rivers one cell wide.
    ///
    /// Reaching it needs the unclamped value to land *between* the real floor
    /// and the mutant's: `mag == 0` (flow exactly at `thresh`) and order 1
    /// give the bare `0.6`, and a 0.001 gradient at `w = 64` damps that to
    /// ~0.4545 — inside `(0.4, 0.5)`. A 0.4 floor would leave it unclamped.
    ///
    /// This fixture is deliberately separate from the width-law test above:
    /// here `slope_fac`'s own constant is *not* observable, because 6.0 would
    /// give ~0.4335 and clamp to the same 0.5.
    #[test]
    fn the_channel_half_width_floor_is_the_references_own_half_cell() {
        let (w, h) = (64usize, 64usize);
        let n = w * h;
        let (cx, cy) = (32usize, 32usize);
        let i = cy * w + cx;

        // A pure x-ramp of 0.001 per cell: gx = 0.001, gy = 0.
        let mut fld = vec![0f32; n];
        for y in 0..h {
            for x in 0..w {
                fld[y * w + x] = 0.5 + 0.001 * x as f32;
            }
        }
        let mut flow = vec![0f32; n];
        let thresh = 100.0f64;
        flow[i] = 100.0; // f/thresh == 1 exactly, so mag == 0 and 3*mag^2 vanishes
        let order = vec![1i16; n];
        let lmax = super::channel_lmax(n);

        let d = super::channel_disc(&fld, &flow, &order, w, h, false, thresh, 1.0, lmax, i)
            .expect("a positive-flow cell must produce a disc");
        assert_eq!(d.mag, 0.0, "the fixture must sit exactly on `thresh`");

        let gx = (fld[cy * w + cx + 1] as f64 - fld[cy * w + cx - 1] as f64) * 0.5;
        let unclamped = 0.6 * (1.0 / (1.0 + 5.0 * cartalith_jsmath::js_hypot(gx, 0.0) * w as f64)) * 1.0;
        assert!(
            unclamped > 0.4 && unclamped < 0.5,
            "the fixture must land between the mutant floor and the real one, got {unclamped}"
        );
        assert_eq!(d.half_w.to_bits(), 0.5f64.to_bits(), "the floor is half a cell, unscaled by width_k");

        // And the floor really is unscaled: the same cell at 800 km
        // (`width_k = 1`) and at a hypothetical narrower width_k both bottom
        // out at 0.5 rather than at `0.5*width_k`.
        let narrow = super::channel_disc(&fld, &flow, &order, w, h, false, thresh, 0.5, lmax, i).unwrap();
        assert_eq!(narrow.half_w.to_bits(), 0.5f64.to_bits());
    }

    /// `lmax = Math.log(W*H*0.05)` (reference line 4495). Every channel cell's
    /// `mag` is divided by this, so the coefficient sets both `amp` and — through
    /// `mag²` — `half_w`.
    ///
    /// The expected side writes `0.05 * n` rather than `n * 0.05` so that a
    /// literal-replace mutation of the function's own text stays unique in the
    /// file; `f64` multiplication is commutative to the bit, so nothing moved.
    #[test]
    fn channel_lmax_is_the_log_of_five_percent_of_the_grid() {
        for &(w, h) in &[(64usize, 64usize), (384, 288), (2048, 2048)] {
            let n = w * h;
            assert_eq!(
                super::channel_lmax(n).to_bits(),
                (0.05f64 * n as f64).ln().to_bits(),
                "{w}x{h}"
            );
            assert_ne!(
                super::channel_lmax(n).to_bits(),
                (0.06f64 * n as f64).ln().to_bits(),
                "{w}x{h}: the coefficient is 0.05, not 0.06"
            );
        }
        // A stated value, so the assertions above cannot both drift together:
        // ln(4096*0.05) = ln(204.8).
        assert!(
            (super::channel_lmax(4096) - 5.322_033_893_165_353).abs() < 1e-12,
            "64x64 gives lmax = ln(204.8), got {}",
            super::channel_lmax(4096)
        );
    }

    // ---- integrated drainage: `build_routing_surface` (§6g) ------------------

    /// Deterministic noisy terrain, heights in `[0.3, 0.9)`.
    fn lcg_field(w: usize, h: usize, seed: u64) -> Vec<f32> {
        let mut s = seed;
        (0..w * h)
            .map(|_| {
                s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                0.3 + 0.6 * ((s >> 40) as f32 / (1u64 << 24) as f32)
            })
            .collect()
    }

    fn is_seed(field: &[f32], w: usize, h: usize, i: usize, sea: f64, world: bool) -> bool {
        let (x, y) = (i % w, i / w);
        (field[i] as f64) < sea || y == 0 || y + 1 == h || (!world && (x == 0 || x + 1 == w))
    }

    /// A 5x5 bowl with one low outlet on the top edge: the raw field holds the
    /// whole interior in a pit at the centre, the routed surface sends it out.
    fn bowl() -> Vec<f32> {
        let mut f = vec![0.9f32; 25];
        f[2] = 0.3; // (2,0): the outlet, an edge seed
        for y in 1..4 {
            for x in 1..4 {
                f[y * 5 + x] = 0.5;
            }
        }
        f[12] = 0.2; // (2,2): the pit
        f
    }

    #[test]
    fn routing_surface_fills_a_pit_with_a_strict_tilt() {
        let f = bowl();
        let s = super::build_routing_surface(&f, 5, 5, 0.0, false);
        // The pit is raised to exactly one f32 step above the 0.5 ring cell it
        // is first reached from -- not to 0.5 (a flat, which would still stop
        // the water), and not by a real-valued epsilon.
        assert_eq!(s[12], 0.5f32.next_up(), "pit must sit one ULP above the ring");
        assert!(s[12] > 0.5, "a fill without the tilt is not a fix");
        // Every seed, and every cell not in the depression, is bit-identical.
        for i in 0..25 {
            if is_seed(&f, 5, 5, i, 0.0, false) {
                assert_eq!(s[i].to_bits(), f[i].to_bits(), "seed {i} moved");
            }
            assert!(s[i] >= f[i], "cell {i} was lowered");
        }
        // Raw: the centre is a pit. Routed: nothing interior is.
        assert_eq!(super::flow_receivers(&f, 5, 5, false)[12], -1);
        let r = super::flow_receivers(&s, 5, 5, false);
        for y in 1..4 {
            for x in 1..4 {
                assert!(r[y * 5 + x] >= 0, "interior ({x},{y}) is still a pit on the routed surface");
            }
        }
    }

    #[test]
    fn routed_flow_carries_the_pits_catchment_to_the_outlet() {
        let f = bowl();
        let raw = super::compute_flow_routed(5, 5, &f, None, false, false, 0.0, false);
        let routed = super::compute_flow_routed(5, 5, &f, None, false, false, 0.0, true);
        // `integrate = false` is `compute_flow` itself, bit for bit.
        assert_eq!(raw, super::compute_flow(5, 5, &f, None, false, false));
        // Raw: the pit keeps its own cell, the eight ring cells and the
        // thirteen rim cells that drain inward over them.
        assert_eq!(raw[12], 22.0);
        // Routed: all of it arrives at the edge outlet instead.
        assert_eq!(routed[2] - raw[2], raw[12]);
    }

    #[test]
    fn routing_surface_is_identity_where_nothing_is_enclosed() {
        // A monotone ramp down to the y = 0 edge has no depression anywhere.
        let (w, h) = (9, 7);
        let f: Vec<f32> = (0..w * h).map(|i| 0.4 + 0.05 * (i / w) as f32 + 0.001 * (i % w) as f32).collect();
        let s = super::build_routing_surface(&f, w, h, 0.0, false);
        assert!(s.iter().zip(&f).all(|(a, b)| a.to_bits() == b.to_bits()));
    }

    #[test]
    fn sub_sea_cells_are_outlets_and_keep_their_height() {
        // The bowl again, but its pit is below sea level: it IS the sea, so
        // nothing is filled and the surface is the field.
        let f = bowl();
        let s = super::build_routing_surface(&f, 5, 5, 0.25, false);
        assert!(s.iter().zip(&f).all(|(a, b)| a.to_bits() == b.to_bits()));
    }

    #[test]
    fn a_wrapped_world_has_no_east_west_outlet() {
        // An 8x5 plateau with its only real outlet on the top edge, and a low
        // cell on the x = 0 column. Unwrapped, that column is a map edge -- an
        // outlet that keeps its height. Wrapped, it is interior ground and must
        // be filled over the plateau.
        let (w, h) = (8, 5);
        let mut f = vec![0.8f32; w * h];
        f[5] = 0.1; // (5,0)
        f[2 * w] = 0.2; // (0,2)
        let flat = super::build_routing_surface(&f, w, h, 0.0, false);
        assert_eq!(flat[2 * w], 0.2, "unwrapped: x = 0 is an edge seed");
        let wrapped = super::build_routing_surface(&f, w, h, 0.0, true);
        assert!(wrapped[2 * w] > 0.8, "wrapped: x = 0 is interior and must be filled, got {}", wrapped[2 * w]);
        assert_eq!(wrapped[5], 0.1, "the y edge is still an outlet under wrap");
    }

    #[test]
    fn every_cell_drains_to_a_seed_on_the_routed_surface() {
        for &(w, h, world, sea) in &[(40usize, 30usize, false, 0.0f64), (40, 30, true, 0.0), (33, 21, false, 0.45), (33, 21, true, 0.45)] {
            let f = lcg_field(w, h, 0xC0FFEE ^ w as u64 ^ (world as u64) << 8);
            let raw_pits = super::flow_receivers(&f, w, h, world)
                .iter()
                .enumerate()
                .filter(|&(i, &r)| r < 0 && !is_seed(&f, w, h, i, sea, world))
                .count();
            assert!(raw_pits > 0, "fixture must contain interior pits to be a test ({w}x{h} world={world})");
            let s = super::build_routing_surface(&f, w, h, sea, world);
            let r = super::flow_receivers(&s, w, h, world);
            for start in 0..w * h {
                let mut i = start;
                let mut steps = 0;
                while r[i] >= 0 {
                    assert!(s[r[i] as usize] < s[i], "receiver must be strictly lower");
                    i = r[i] as usize;
                    steps += 1;
                    assert!(steps <= w * h, "cycle");
                }
                assert!(
                    is_seed(&f, w, h, i, sea, world),
                    "cell {start} ends in a non-seed pit at {i} ({w}x{h} world={world} sea={sea})"
                );
            }
        }
    }

    #[test]
    fn inside_a_filled_basin_the_channel_follows_the_accumulation_tree() {
        let (w, h) = (48, 36);
        let f = lcg_field(w, h, 11);
        let s = super::build_routing_surface(&f, w, h, 0.0, false);
        let flow = super::compute_flow(w, h, &s, None, false, false);
        let d8 = super::flow_receivers(&s, w, h, false);
        let c = super::build_channels_routed(&f, &s, &flow, w, h, 0.0, false, 1.0, 800.0);
        let filled: Vec<usize> = (0..w * h).filter(|&i| c.chan[i] != 0 && s[i] > f[i]).collect();
        assert!(filled.len() > 20, "fixture must route channels through filled cells ({})", filled.len());
        for &i in &filled {
            assert_eq!(c.recv[i], d8[i], "filled channel cell {i} left the accumulation tree");
            // ...and so never steps off the channel mask (density 1: the
            // threshold is flat, and the receiver's discharge is >= its own).
            assert_ne!(c.chan[c.recv[i] as usize], 0, "filled channel cell {i} steps onto a non-channel cell");
        }
    }

    #[test]
    fn build_channels_routed_over_the_field_is_build_channels() {
        let (w, h) = (36, 28);
        let f = lcg_field(w, h, 7);
        let flow = super::compute_flow(w, h, &f, None, false, false);
        let a = super::build_channels(&f, &flow, w, h, 0.35, false, 1.0, 800.0);
        let b = super::build_channels_routed(&f, &f, &flow, w, h, 0.35, false, 1.0, 800.0);
        assert_eq!(a.recv, b.recv);
        assert_eq!(a.chan, b.chan);
        assert!(a.slope.iter().zip(&b.slope).all(|(x, y)| x.to_bits() == y.to_bits()));
    }

    #[test]
    fn routed_channels_never_end_in_an_interior_pit_and_keep_the_real_slope() {
        let (w, h) = (48, 36);
        let f = lcg_field(w, h, 11);
        let s = super::build_routing_surface(&f, w, h, 0.0, false);
        let flow = super::compute_flow(w, h, &s, None, false, false);
        let raw = super::build_channels(&f, &flow, w, h, 0.0, false, 1.0, 800.0);
        let routed = super::build_channels_routed(&f, &s, &flow, w, h, 0.0, false, 1.0, 800.0);
        // Slope and channel initiation are statements about the ground.
        assert_eq!(raw.chan, routed.chan);
        assert!(raw.slope.iter().zip(&routed.slope).all(|(x, y)| x.to_bits() == y.to_bits()));
        let dead = |c: &super::ChannelResult| {
            (0..w * h).filter(|&i| c.chan[i] != 0 && c.recv[i] < 0 && !is_seed(&f, w, h, i, 0.0, false)).count()
        };
        assert!(dead(&raw) > 0, "fixture must have channel cells ending in a raw pit to be a test");
        assert_eq!(dead(&routed), 0);
    }
}
