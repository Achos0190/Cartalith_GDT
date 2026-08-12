//! flow accumulation, river network, channel width
//!
//! Ported in pipeline order starting Phase 1 (MVP_SCOPE.md).

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
        for v in &mut acc {
            *v = (*v as f64 * k) as f32;
        }
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

    for y in 0..h {
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
            slope[i] = slope_n as f32;
            if flow[i] as f64 <= channel_threshold(thresh, slope_n, density) {
                continue;
            }
            chan[i] = 1;

            let hh = fld[i] as f64;
            let aspect = (-gy).atan2(-gx);
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
                    let mut da = (dy as f64).atan2(dx as f64) - aspect;
                    da = da.sin().atan2(da.cos()).abs();
                    let score = drop * (0.5 + 0.5 * da.cos());
                    if score > best_score {
                        best_score = score;
                        best = j;
                    }
                }
            }
            recv[i] = if best >= 0 { best as i32 } else { s_best as i32 };
        }
    }

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

#[cfg(test)]
mod tests {
    #[test]
    fn crate_compiles_and_tests_run() {
        assert_eq!(2 + 2, 4);
    }
}
