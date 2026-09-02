//! flow accumulation, river network, channel width
//!
//! Ported in pipeline order starting Phase 1 (MVP_SCOPE.md).

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

    // D8[(dy+1)*3+(dx+1)] = hypot(dx, dy) for dx,dy in {-1,0,1}; center
    // (index 4) is unused (the main loop always skips dx=dy=0) but kept
    // for a direct match to the reference's own indexing scheme.
    let mut d8 = [0f64; 9];
    for dy in -1i32..=1 {
        for dx in -1i32..=1 {
            d8[((dy + 1) * 3 + (dx + 1)) as usize] = (dx as f64).hypot(dy as f64);
        }
    }

    for &i in &order {
        let i = i as usize;
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
        if best >= 0 {
            let best = best as usize;
            acc[best] = (acc[best] as f64 + acc[i] as f64) as f32;
        }
    }

    acc
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
    /// stamp on this world" — a loaded save has one, since
    /// `SAVEFILE_COMPAT.md` stores no channel topology.
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
    let wrap = world;
    let n = w * h;
    let thresh = river_flow_thresh(w, h, w, map_width_km);
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
                let slope_n = js_hypot(gx, gy) * w as f64;
                slope_row[x] = slope_n as f32;
                if flow[i] as f64 <= channel_threshold(thresh, slope_n, density) {
                    continue;
                }
                chan_row[x] = 1;

                let hh = fld[i] as f64;
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
                        let drop = (hh - fld[j as usize] as f64) / d8[((dy + 1) * 3 + (dx + 1)) as usize];
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
                recv_row[x] = if best >= 0 { best as i32 } else { s_best as i32 };
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
    /// Channel half-width in cells at the mouth ([`channel_disc`]), or `None`
    /// when the mouth carries no positive flow.
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
        .map(|pl| {
            let head = cell_of(pl[0]);
            let mouth = cell_of(pl[pl.len() - 1]);
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
                half_width_cells: channel_disc(fld, flow, order, w, h, wrap, thresh, width_k, lmax, mouth)
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
#[allow(clippy::too_many_arguments)]
pub fn stamp_river_intensity(
    fld: &[f32],
    flow: &[f32],
    chan: &[u8],
    order: &[i16],
    w: usize,
    h: usize,
    wrap: bool,
    thresh: f64,
    width_k: f64,
) -> Vec<f32> {
    let n = w * h;
    let mut intensity = vec![0f32; n];
    if n == 0 || fld.len() < n || flow.len() < n || chan.len() < n || order.len() < n {
        return intensity;
    }
    let lmax = channel_lmax(n);
    if !(lmax > 0.0) || !(thresh > 0.0) {
        return intensity;
    }

    for i in 0..n {
        if chan[i] == 0 {
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
    intensity
}

#[cfg(test)]
mod tests {
    use super::{build_channels, enforce_river_channels, flow_cmp_desc, flow_sort_desc};

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
        let mut order = vec![0i16; n];
        let thresh = 4.0f64;
        flow[mid] = 4000.0; // well above thresh, so `mag` is near its ceiling
        chan[mid] = 1;
        order[mid] = 5;

        let inked = |k: f64| -> usize {
            super::stamp_river_intensity(&fld, &flow, &chan, &order, w, h, false, thresh, k)
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
        let v = super::stamp_river_intensity(&fld, &flow, &chan, &order, w, h, false, thresh, 16.0);
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
            &vec![0.5f32; n], &vec![1.0f32; n], &vec![0u8; n], &vec![0i16; n], w, h, false, 1.0, 4.0,
        );
        assert_eq!(out.len(), n);
        assert!(out.iter().all(|&v| v == 0.0), "no channel cells must ink nothing");

        // Short slices: return a correctly-sized zero grid rather than panic.
        let short = super::stamp_river_intensity(&[0.5f32; 4], &[1.0f32; 4], &[1u8; 4], &[1i16; 4], w, h, false, 1.0, 4.0);
        assert_eq!(short.len(), n);
        assert!(short.iter().all(|&v| v == 0.0));
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
}
