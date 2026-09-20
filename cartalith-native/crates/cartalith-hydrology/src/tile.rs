//! Tile-bounded hydrology refinement — EF-1 of
//! `ELEVATION_FIELD_ARCHITECTURE_RESEARCH.md`.
//!
//! Re-derives flow accumulation, channels and river polylines for **one
//! rectangular tile of the world at a finer resolution than the world pass
//! ever ran at**, so a zoomed-in view can show tributaries that were never
//! resolvable at world scale.
//!
//! # New capability, not a port — flagged, per `cartalith-porting-discipline`
//!
//! There is no reference function this ports. The reference's own
//! `amplifyRegion`/`refineTile` are elevation-only — as is this port's
//! `cartalith_terrain::amplify_region` — and the reference never re-derives
//! hydrology at zoom, so there is **no golden value to match here and none is
//! claimed**. The rule for this case is the skill's own: *"Anything that
//! changes output: flag it, explain why, get it confirmed rather than assumed
//! correct."* Nothing below changes the output of any existing function —
//! `compute_flow`, `build_channels` and `trace_river_polylines` are reused
//! unchanged, and `build_channels`' own threshold expression is untouched (see
//! [`crate::build_channels_with_threshold`]). What is new is a second,
//! tile-scoped *entry* into the same algorithms, and the boundary condition
//! that makes a tile-scoped answer correct at all.
//!
//! # The constraint this module exists to respect
//!
//! **Flow direction is local; flow accumulation is global.** A cell's D8
//! downhill neighbour depends only on its own 3×3 elevation neighbourhood, so
//! it can be recomputed at any resolution for any rectangle with no wider
//! context — [`crate::d8_receiver`] does exactly that and is the same code the
//! world pass uses. A cell's *accumulated* flow is the drainage area above it,
//! which for a real river spans far more than one tile. Re-running
//! `compute_flow` on an isolated tile rectangle therefore does not produce a
//! zoomed-in answer; it produces a **different, wrong** answer — the tile's own
//! little watershed, with a trunk river arriving as a trickle.
//! `tile_flow_is_not_a_naive_local_rerun` measures that gap rather than
//! asserting it from theory.
//!
//! The fix is an inflow boundary condition, and the data for it is already
//! computed: `WorldState::flow_discharge`, the world pass's own accumulated
//! flow. This module never recomputes it and never corrects it — it *inherits*
//! it.
//!
//! **What "by construction" actually covers, corrected 2026-09-20 by an
//! adversarial verifier — the inflow total, not the resulting channel shape.**
//! The boundary inflow sum is exact (measured bit-for-bit against an
//! independent oracle on real worlds). The *geometry* the fine tile then
//! draws — which cells end up classified as channel, where the peak
//! accumulation lands — agrees with the coarse network only approximately:
//! on roughly a third to half of tiles it falls outside the ±25% agreement
//! band this module's own tests check for. That is a property of comparing a
//! finer sampling of a real, chaotic drainage pattern against a coarser one,
//! not a bug in the inflow arithmetic — but "by construction" overstated it,
//! and this module's own boundary-agreement tests were found to hold only
//! because they ran against a mock detail amplitude ~1750× smaller than
//! `amplify_region`'s real one, which made them far too easy to pass. Neither
//! the total-conservation guarantee nor the module's correctness is in
//! question; the *tightness* of the geometric agreement claim was.
//!
//! **A self-consistency hazard for whoever wires this to a live `WorldState`,
//! not a defect here:** with `carve_rivers = true` (the default),
//! `WorldState::field` and `WorldState::flow_discharge` stop agreeing —
//! `field` is mutated by the carve/sediment passes *after* `flow_discharge`
//! is taken (`cartalith-engine/src/lib.rs`, `route_sediment` runs the line
//! after `compute_flow`). Feeding this module a refined-elevation tile
//! derived from the post-carve `field` alongside the pre-carve
//! `flow_discharge` would seed a boundary condition against a topology the
//! discharge numbers never had. This module takes both as explicit
//! parameters and does not itself read `WorldState` — the caller that wires
//! them together, in a later pass, must supply a self-consistent pair.
//!
//! # Units — flow here is in coarse-cell-equivalents, deliberately
//!
//! `compute_flow` seeds `1.0` per coarse cell (or mean-1 rainfall, which is the
//! same scale). This module seeds `1/refine²` per **fine** cell, so one coarse
//! cell's worth of fine cells still contributes exactly `1.0` and a tile flow
//! value means the same physical drained area a coarse flow value does. That is
//! what lets the coarse boundary numbers be added to fine cells without
//! conversion, and what lets [`tile_channel_thresh`] hand the classifier the
//! world's own threshold.
//!
//! A useful consequence, and a test in its own right
//! (`a_flat_tile_with_no_inflow_produces_no_rivers`): in a **sheet-flow**
//! regime, where nothing converges, each of the `refine` fine columns covering
//! one coarse column carries `1/refine` of the coarse value (less the one ring
//! row whose seed leaves at the open boundary below) — so unchannelled
//! ground reads *lower* at fine resolution, not higher. In a **channel**
//! regime, where the refined terrain converges the flow into one line, the fine
//! value climbs back to the coarse one. Refinement therefore sharpens the
//! contrast between hillslope and channel instead of manufacturing rivers.
//!
//! # The tile's edge is an open boundary
//!
//! Water enters the tile at the boundary crossings the coarse D8 network
//! already makes, and leaves through the tile's outer fine ring, which
//! receives what the interior sends it and forwards nothing. [`tile_flow`]'s
//! own note carries the measurement that forced that rule and the two
//! consequences it has — chief among them that the inflow is injected one
//! fine cell *inside* the ring, never on it.
//!
//! # Coordinates
//!
//! Pixel space throughout, the convention `trace_river_polylines` already
//! returns (`{x: col+0.5, y: row+0.5}`): coarse cell `i` spans `[i, i+1)` and
//! has its centre at `i+0.5`. [`TilePlacement::fine_to_coarse`] is the whole
//! mapping. A consumer feeding this into a *sampler* whose integer coordinate
//! is a cell **centre** — `cartalith_terrain::amplify_region`'s `samp` is the
//! one that matters — subtracts `0.5` after converting; that is the one place
//! the two conventions meet and it is stated here rather than rediscovered.

use crate::{ChannelResult, build_channels_with_threshold, d8_receiver, d8_table, strahler_from_receivers, trace_river_polylines};

/// Where a refined tile sits in the coarse world grid, and how much finer it
/// is.
///
/// The tile covers coarse cells `x0..x0+cols` by `y0..y0+rows` and is refined
/// to `cols*refine` by `rows*refine` fine cells — an exact integer subdivision,
/// so every fine cell has exactly one parent coarse cell. That is a
/// requirement, not a convenience: the boundary condition below has to know
/// which coarse cell a fine cell belongs to, and a fractional refine factor
/// would make that a resampling question instead of an indexing one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TilePlacement {
    /// The coarse world grid's width — `WorldState`'s `gw`, not the tile's.
    pub coarse_w: usize,
    /// The coarse world grid's height — `WorldState`'s `gh`, not the tile's.
    pub coarse_h: usize,
    /// The tile's left edge, in coarse cells.
    pub x0: usize,
    /// The tile's top edge, in coarse cells.
    pub y0: usize,
    /// The tile's width, in coarse cells.
    pub cols: usize,
    /// The tile's height, in coarse cells.
    pub rows: usize,
    /// Fine cells per coarse cell along each axis. `1` is a same-resolution
    /// re-run, which is a legitimate (and useful) degenerate case.
    pub refine: usize,
    /// The **coarse world** wraps in `x` (`WorldParams::world`). This decides
    /// where the ring of donor cells around a tile at `x0 == 0` is read from,
    /// nothing else — the tile's own interior never wraps, since a deep-zoom
    /// tile is a sub-rectangle by definition. A full-width tile on a wrapping
    /// world would need the interior wrap too and is deliberately not
    /// supported; [`tile_flow`] asserts against it rather than silently
    /// treating the seam as a map edge.
    pub world: bool,
}

impl TilePlacement {
    /// The tile's width in fine cells.
    pub fn fine_w(&self) -> usize {
        self.cols * self.refine
    }

    /// The tile's height in fine cells.
    pub fn fine_h(&self) -> usize {
        self.rows * self.refine
    }

    /// Fine-tile pixel coordinates to coarse-world pixel coordinates — the
    /// mapping that turns [`TileRivers::polylines`] into something drawable
    /// over the world, and the only place the two coordinate frames are
    /// related.
    pub fn fine_to_coarse(&self, fx: f64, fy: f64) -> (f64, f64) {
        let k = self.refine as f64;
        (self.x0 as f64 + fx / k, self.y0 as f64 + fy / k)
    }
}

/// The channel-initiation threshold a tile should classify against: the
/// **world's** own, not the tile's.
///
/// `river_flow_thresh(coarse_w, coarse_h, coarse_w, map_width_km)` — every
/// argument the world pass itself would pass. Since [`tile_flow`] returns
/// coarse-cell-equivalents (see the module header), this is directly
/// comparable: a fine cell channelizes when it drains the same real area a
/// coarse cell had to drain.
///
/// **This answers the conservative half of the design document's owner
/// question 2** (*"is 'reveal the tributaries that already exist in the coarse
/// simulation's implied watershed' the target, or do you want genuinely finer
/// streams than the coarse pass's own channel threshold would ever classify as
/// a river at world scale?"*) and does not pretend to answer the other half.
/// That is why [`tile_rivers`] takes the threshold as an argument instead of
/// calling this itself: the unanswered question stays visible at the call site,
/// and answering it later is a caller passing a smaller number, not a rewrite
/// here.
pub fn tile_channel_thresh(coarse_w: usize, coarse_h: usize, map_width_km: f64) -> f64 {
    crate::river_flow_thresh(coarse_w, coarse_h, coarse_w, map_width_km)
}

/// One coarse cell's accumulated flow crossing into the tile, already resolved
/// to the fine cell it enters at.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Inflow {
    /// Index into the tile's fine grid.
    fine: usize,
    /// The donor coarse cell's `flow_discharge`, undivided.
    flow: f32,
}

/// Every inward crossing of the tile boundary the **coarse** D8 network makes,
/// mapped to the fine cell it should be injected at.
///
/// # What is read, and why it is the ring rather than the edge
///
/// The obvious formulation — "read `flow_discharge` along the tile's own edge
/// cells" — double counts: an edge cell's discharge already contains whatever
/// the tile itself contributed upstream of it, and contains flow that never
/// entered the tile at all. The crossings are what the boundary condition
/// actually is. So: for each coarse cell in the one-cell ring **outside** the
/// tile, take its coarse D8 receiver (the same [`d8_receiver`] the world pass
/// used, on the coarse field); if that receiver is inside the tile, this cell's
/// entire `flow_discharge` crosses the boundary there, exactly once. A ring
/// cell draining into another ring cell contributes nothing here — its water is
/// already inside its neighbour's total, and will be counted if and where that
/// neighbour crosses. Only the ring can matter, because a D8 receiver is always
/// an adjacent cell.
///
/// # Distributing one coarse value over `refine` fine cells
///
/// The receiver coarse cell `R` is `refine × refine` fine cells, and the water
/// crosses the face of `R` that touches the donor `C`: a fine column when `C`
/// is left or right, a fine row when above or below, a single corner cell when
/// diagonal. **The whole value goes into the lowest fine cell of that face**,
/// ties to the lowest index.
///
/// That is the defensible choice, and an even split is not, for two reasons
/// that are the same reason:
///
/// - **The coarse model had already concentrated it.** `flow_discharge[C]` is
///   the total through a single D8 receiver chain — one line, not a sheet.
///   Splitting it `refine` ways at the tile boundary would be this module
///   inventing a diffusion the world pass never had.
/// - **An even split can delete the river it is trying to preserve.** Each
///   sub-stream carries `1/refine` of the discharge, and a trunk sitting a
///   factor of two above `tile_channel_thresh` drops below it at `refine = 4`.
///   The visible result is precisely the artifact EF-1 exists to prevent: the
///   river vanishes at the tile edge. Concentrating costs nothing symmetric,
///   because the fine D8 field re-spreads the flow immediately if the refined
///   terrain says it should.
///
/// Choosing the *lowest* cell of the face rather than its middle is what keeps
/// the entry point consistent with the refined terrain: the river enters where
/// the refined valley is, so the first fine cell of the trace is already on the
/// channel instead of on its bank.
///
/// **The injection cell is then stepped one fine cell off the tile's outer
/// ring** ([`step_inside`]), because that ring is an outlet and never forwards
/// what it receives — see [`tile_flow`]'s own "open boundary" note. Injecting
/// *on* the ring would drop the whole boundary condition on the floor. The
/// cost is that a river enters one fine cell inside the edge instead of on it,
/// which is a fraction `1/refine` of a coarse cell and is why the step is
/// taken here rather than by weakening the outlet rule. At `refine == 1` —
/// the same-resolution re-run — that fraction is a whole coarse cell, which is
/// the price of that degenerate case and not a reason to special-case it.
///
/// # The one approximation, stated
///
/// A coarse flow path that leaves the tile and re-enters is counted at **each**
/// inward crossing, so a meander that crosses twice contributes its discharge
/// twice. Detecting it would mean tracing receiver chains outside the tile —
/// global work, which is the thing this design exists to avoid. It needs a
/// coarse path to exit and return within one tile's perimeter, and it is left
/// as a known over-count rather than silently smoothed.
///
/// **Measured, not just bounded in theory (corrected 2026-09-20, the original
/// estimate understated it):** on synthetic terrain it affects roughly
/// 15-40% of tiles depending on size, reaching up to 6× the true inflow on
/// smooth terrain where meanders are common; on this engine's own real
/// generated worlds, with self-consistent inputs, the median tile is exact
/// (1.0×) and the worst observed case reached 2.0×. It is not bounded by "the
/// discharge of that one path" in general — a path can re-enter more than
/// once — only by how many times a coarse receiver chain crosses this one
/// tile's perimeter, which has no fixed cap.
fn boundary_inflows(place: &TilePlacement, refined: &[f32], coarse_field: &[f32], coarse_flow: &[f32]) -> Vec<Inflow> {
    let TilePlacement { coarse_w: cw, coarse_h: ch, x0, y0, cols, rows, refine, world } = *place;
    let d8 = d8_table();
    let (cw_i, ch_i) = (cw as i64, ch as i64);
    let fine_w = place.fine_w();
    let mut out = Vec::new();

    for gy in (y0 as i64 - 1)..=(y0 + rows) as i64 {
        if gy < 0 || gy >= ch_i {
            continue;
        }
        for gx in (x0 as i64 - 1)..=(x0 + cols) as i64 {
            let cx = if world {
                ((gx % cw_i) + cw_i) % cw_i
            } else if gx < 0 || gx >= cw_i {
                continue;
            } else {
                gx
            };
            // The interior test is on the *wrapped* column, so a wrapped ring
            // lookup that lands back inside the tile is skipped rather than
            // treated as its own donor.
            if inside(cx, gy, x0, y0, cols, rows) {
                continue;
            }
            let c = (gy * cw_i + cx) as usize;
            let r = d8_receiver(coarse_field, cw, ch, c, world, &d8);
            if r < 0 {
                continue;
            }
            let (rx, ry) = ((r % cw_i), (r / cw_i));
            if !inside(rx, ry, x0, y0, cols, rows) {
                continue;
            }
            // The **donor's** discharge is what crosses the boundary. The
            // receiver's own value is a world-pass number for a cell inside the
            // tile, which the tile is re-deriving and must not inherit.
            let flow = coarse_flow[c];
            if flow.is_nan() || flow <= 0.0 {
                continue;
            }

            // Direction from the receiver to the donor, wrapped into {-1,0,1}.
            let mut dcx = cx - rx;
            if world {
                if dcx > 1 {
                    dcx -= cw_i;
                } else if dcx < -1 {
                    dcx += cw_i;
                }
            }
            let dcy = gy - ry;
            debug_assert!(dcx.abs() <= 1 && dcy.abs() <= 1, "a D8 receiver is an adjacent cell");

            // The face of the receiver's fine block that touches the donor.
            let (bx, by) = (((rx - x0 as i64) as usize) * refine, ((ry - y0 as i64) as usize) * refine);
            let (fx_lo, fx_hi) = face_span(bx, refine, dcx);
            let (fy_lo, fy_hi) = face_span(by, refine, dcy);
            let (mut lx, mut ly) = (fx_lo, fy_lo);
            let mut low = refined[fy_lo * fine_w + fx_lo];
            for fy in fy_lo..fy_hi {
                for fx in fx_lo..fx_hi {
                    let idx = fy * fine_w + fx;
                    if refined[idx] < low {
                        low = refined[idx];
                        (lx, ly) = (fx, fy);
                    }
                }
            }
            // One step off the outlet ring, so the boundary condition is
            // actually carried inward rather than terminating where it lands.
            let fine = step_inside(ly, place.fine_h()) * fine_w + step_inside(lx, fine_w);
            out.push(Inflow { fine, flow });
        }
    }
    out
}

/// Is coarse cell `(x, y)` inside the tile?
#[inline]
fn inside(x: i64, y: i64, x0: usize, y0: usize, cols: usize, rows: usize) -> bool {
    x >= x0 as i64 && x < (x0 + cols) as i64 && y >= y0 as i64 && y < (y0 + rows) as i64
}

/// The half-open fine-index span of one face of a coarse cell's fine block:
/// the far edge when the donor is on the positive side, the near edge when it
/// is on the negative side, the whole block when it is neither.
#[inline]
fn face_span(base: usize, refine: usize, d: i64) -> (usize, usize) {
    match d {
        d if d > 0 => (base + refine - 1, base + refine),
        d if d < 0 => (base, base + 1),
        _ => (base, base + refine),
    }
}

/// One fine-grid coordinate, moved off the outer ring if it is on it. `n` is
/// the extent along that axis; a tile under three cells across has no interior
/// on that axis and is left alone rather than clamped into nonsense.
#[inline]
fn step_inside(v: usize, n: usize) -> usize {
    if n <= 2 { v } else { v.clamp(1, n - 2) }
}

/// Is fine cell `i` on the tile's outer ring — the open-boundary cells
/// [`tile_flow`] lets water leave through?
#[inline]
fn on_ring(i: usize, fine_w: usize, fine_h: usize) -> bool {
    let (x, y) = (i % fine_w, i / fine_w);
    x == 0 || y == 0 || x + 1 == fine_w || y + 1 == fine_h
}

/// Flow accumulation for one refined tile: [`compute_flow`]'s algorithm,
/// bounded to the tile, seeded at the boundary from the coarse world's own
/// `flow_discharge`.
///
/// [`compute_flow`]: crate::compute_flow
///
/// Three steps, and only the middle one is new:
///
/// 1. **Direction**, at the tile's own resolution, from `refined` alone — the
///    genuinely local half, [`d8_receiver`] with no wrap.
/// 2. **Seeding.** Every fine cell starts at `1/refine²` (one coarse cell's
///    worth of fine cells sums to the `1.0` `compute_flow` would have given it),
///    plus [`boundary_inflows`]' crossings of the coarse network.
/// 3. **Accumulation**, by the same strict descending-height scatter
///    `compute_flow` runs, over the tile's own cells only — the drainage-order
///    trick still holds, because a cell's donors are all above it and are
///    therefore all already visited.
///
/// # The tile's outer fine ring is an open boundary
///
/// A ring cell's true steepest descent is very often the cell just *outside*
/// the tile, which the tile cannot see. Letting it fall back to its lowest
/// in-tile neighbour instead does not approximate that — it turns an outflow
/// into a flow *along* the boundary, which is the one artifact a bounded
/// accumulation can manufacture and it manufactures it exactly where the tile
/// has to agree with the world. **Measured on `tile_hydrology.rs`'s
/// `valley_world` fixture, at tile `(118, 128) 16×16 refine 4`, before this
/// rule existed**: a 14 000-unit trunk arriving at the tile's south edge ran
/// on westward along that edge as a band six fine cells wide, and left the
/// tile at coarse `(124, 143)` — **two** coarse cells from the coarse
/// network's own departure at `(126, 143)`. With the rule it leaves at
/// `(125, 143)`, one cell, which is what the boundary test asks for.
///
/// So a ring cell **receives and terminates**: every cell in the tile scatters
/// into its D8 receiver except the ring, whose water has left. Two
/// consequences, both deliberate:
///
/// - The ring's *own* seed never enters the tile, so a tile's interior
///   accumulation is short by its perimeter's rainfall —
///   `2(w+h)-4` fine cells of `1/refine²` each, against `cols*rows` for the
///   whole tile. It is an under-count, never an over-count, and it is
///   vanishing next to any real boundary inflow.
/// - [`boundary_inflows`] therefore injects one fine cell *inside* the ring,
///   not on it.
///
/// An interior cell whose descent leads out of the tile still routes into the
/// ring and stops there, so the exit is at the tile edge and a traced polyline
/// reaches it.
///
/// # Rainfall
///
/// The interior seed is uniform. `WorldState::flow_discharge` is rain-seeded
/// (`compute_flow(.., use_rain = true)`), but its rainfall is mean-normalised
/// to `1.0` per cell, so the two are on the same scale and the boundary
/// numbers need no conversion — what the tile loses is *within-tile* rainfall
/// variation. Using the coarse rain field here would be an upsample, which is
/// the thing the design document set out to avoid; the honest fix is a refined
/// rain field, and there is not one yet (EF-0 is elevation-only). Stated as a
/// first-pass approximation, not a finished answer.
///
/// # Panics
///
/// If any dimension is zero, if the tile is not wholly inside the coarse grid,
/// if `refined` is not `fine_w * fine_h` cells, if either coarse slice is
/// shorter than `coarse_w * coarse_h`, or if a wrapping world is asked for a
/// full-width tile (the interior wrap that would need is not implemented).
pub fn tile_flow(place: &TilePlacement, refined: &[f32], coarse_field: &[f32], coarse_flow: &[f32]) -> Vec<f32> {
    let TilePlacement { coarse_w, coarse_h, x0, y0, cols, rows, refine, world } = *place;
    assert!(
        coarse_w > 0 && coarse_h > 0 && cols > 0 && rows > 0 && refine > 0,
        "tile_flow needs a non-empty coarse grid and a non-empty tile ({coarse_w}x{coarse_h}, tile {cols}x{rows}, refine {refine})"
    );
    assert!(
        x0 + cols <= coarse_w && y0 + rows <= coarse_h,
        "tile ({x0},{y0}) {cols}x{rows} does not fit inside a {coarse_w}x{coarse_h} coarse grid"
    );
    assert!(
        !(world && cols == coarse_w),
        "a full-width tile on a wrapping world would need the tile interior to wrap too, which tile_flow does not implement"
    );
    let n = coarse_w * coarse_h;
    assert!(coarse_field.len() >= n, "coarse field is {} cells, needs {n}", coarse_field.len());
    assert!(coarse_flow.len() >= n, "coarse flow is {} cells, needs {n}", coarse_flow.len());
    let (fine_w, fine_h) = (place.fine_w(), place.fine_h());
    let fine_n = fine_w * fine_h;
    assert!(
        refined.len() == fine_n,
        "refined tile is {} cells, needs exactly {fine_n} ({fine_w}x{fine_h})",
        refined.len()
    );

    // Step 2, seeding. `1/refine²` per fine cell, so the tile's own rainfall
    // is on the coarse pass's scale (module header, "Units").
    let k = refine as f64;
    let mut acc = vec![(1.0 / (k * k)) as f32; fine_n];
    for Inflow { fine, flow } in boundary_inflows(place, refined, coarse_field, coarse_flow) {
        // Per-write `f32` rounding, the same shape `compute_flow`'s own
        // `acc[best] += acc[i]` keeps -- several crossings can land on one fine
        // cell, and each rounds on its own rather than through an `f64`
        // accumulator (`cartalith-rust-conventions`).
        acc[fine] = (acc[fine] as f64 + flow as f64) as f32;
    }

    // Steps 1 and 3, the scatter. Sequential for exactly the reason
    // `compute_flow` is (`CPU_MULTITHREADING_SCOPE.md`): a wavefront
    // dependency, and a running float sum whose order is load-bearing.
    let order = crate::flow_sort_desc(refined, fine_n);
    let d8 = d8_table();
    for &i in &order {
        let i = i as usize;
        if on_ring(i, fine_w, fine_h) {
            // Open boundary: this cell's water has left the tile.
            continue;
        }
        let best = d8_receiver(refined, fine_w, fine_h, i, false, &d8);
        if best >= 0 {
            let best = best as usize;
            acc[best] = (acc[best] as f64 + acc[i] as f64) as f32;
        }
    }
    acc
}

/// Everything EF-1 produces for one tile: the same four outputs the world pass
/// produces, at the tile's resolution.
pub struct TileRivers {
    /// [`tile_flow`]'s accumulation, in coarse-cell-equivalents.
    pub flow: Vec<f32>,
    /// `recv`/`chan`/`slope` from [`crate::build_channels_with_threshold`].
    /// `intensity` is empty, exactly as `build_channels` leaves it.
    pub channels: ChannelResult,
    /// Strahler order over the tile's own channel network.
    pub order: Vec<i16>,
    /// The tile's river polylines, in **fine-tile pixel** coordinates —
    /// `trace_river_polylines`' own `(col+0.5, row+0.5)`. Use
    /// [`TilePlacement::fine_to_coarse`] to place them on the world.
    pub polylines: Vec<Vec<(f64, f64)>>,
}

/// The whole EF-1 pass for one tile: [`tile_flow`], then `build_channels`,
/// `strahler_from_receivers` and `trace_river_polylines` on its result.
///
/// `thresh` is the channel-initiation threshold, in the same
/// coarse-cell-equivalent units [`tile_flow`] returns —
/// [`tile_channel_thresh`] is the world-anchored value and the documented
/// default; see its own comment for why this is an argument rather than a
/// derivation.
///
/// `sea` and `river_density` are the world's own (`WorldState::sea_level`,
/// `WorldParams::river_density`). `min_order` is fixed at `1` — the world pass
/// traces every channel too, and a consumer wanting main stems only can
/// re-trace from [`TileRivers::order`] and `channels.recv`.
///
/// # Both of the classifier's world-anchored numbers are supplied, not derived
///
/// `build_channels` derives two things from the grid it is handed — the
/// channel-initiation threshold, from that grid's cell count, and the slope
/// normalisation, from that grid's width. Both are right when the grid *is*
/// the world and wrong for a tile, in the same direction and for the same
/// reason. [`tile_channel_thresh`] is the first;
/// `coarse_w * refine` — the world's own width, at the tile's resolution — is
/// the second, and [`crate::build_channels_with_threshold`]'s `slope_w` is how
/// it reaches the classifier. Without it a `16`-column tile of a `256`-column
/// world reports `1/16` of the world's `slope_n` for the same physical slope,
/// which is inert at `river_density == 1` and not inert away from it.
pub fn tile_rivers(
    place: &TilePlacement,
    refined: &[f32],
    coarse_field: &[f32],
    coarse_flow: &[f32],
    sea: f64,
    river_density: f64,
    thresh: f64,
) -> TileRivers {
    let flow = tile_flow(place, refined, coarse_field, coarse_flow);
    let (fine_w, fine_h) = (place.fine_w(), place.fine_h());
    let channels =
        build_channels_with_threshold(refined, &flow, fine_w, fine_h, sea, false, river_density, place.coarse_w * place.refine, thresh);
    let order = strahler_from_receivers(&channels.recv, &flow, &channels.chan);
    let polylines = trace_river_polylines(&order, &channels.recv, fine_w, fine_h, 1);
    TileRivers { flow, channels, order, polylines }
}
