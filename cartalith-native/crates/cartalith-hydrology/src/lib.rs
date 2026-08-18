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
use cartalith_jsmath::js_atan2;

/// Descending-height, ascending-index-on-tie comparison — the ordering
/// `_flowRadixSortDesc()` (reference HTML lines 4846-4861) guarantees.
/// The JS implementation is a radix sort operating on IEEE-754 bit
/// patterns (an order-preserving float→uint key, inverted for descending
/// order); the *algorithm* is a correctness-equivalent substitution
/// target per `PROVENANCE.md` (flow accumulation is downstream of the
/// heightmap pixels — only the ordering guarantee matters for parity,
/// not the sort implementation), but its **quirk carries over**: JS
/// explicitly normalizes `-0.0`'s sort key to match `+0.0`
/// (`if(b===0x80000000) b=0`), which `f32::total_cmp` does not do on its
/// own (`total_cmp` treats `-0.0 < +0.0`) — normalized here before
/// comparing.
fn flow_cmp_desc(a: f32, b: f32) -> std::cmp::Ordering {
    let na = if a == 0.0 { 0.0f32 } else { a };
    let nb = if b == 0.0 { 0.0f32 } else { b };
    nb.total_cmp(&na)
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
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by(|&a, &b| flow_cmp_desc(field[a], field[b]).then(a.cmp(&b)));

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
                let slope_n = gx.hypot(gy) * w as f64;
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

    ChannelResult { recv, chan, slope }
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

/// `enforceChannelDescent()` (reference HTML lines 8725-8737): walks an
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
                let d = (x as f64 - px as f64).hypot(y as f64 - py as f64);
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

#[cfg(test)]
mod tests {
    use super::{build_channels, enforce_river_channels};

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
}
