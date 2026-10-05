//! **RIM-4: river deltas on the painted path** (`RIVERS_IN_MAP_SCOPE.md` RIM-4
//! and owner question 3, Ruling BU).
//!
//! A real river that meets the sea or a lake with a heavy load splits into a
//! fan of distributaries; a drawn centreline that simply stops at the shore
//! reads as a pipe. [`delta_fans`] derives, from the drawn network and the
//! water the map draws, a small set of extra runs -- a few short
//! distributaries fanning out from a point a little upstream of each large
//! river's shore crossing, each marching on land until it first meets water --
//! that the painted path adds to its distance field
//! (`river_field::build_with`) and its colour field
//! (`river_stroke::rasterize_colour_field_with`).
//!
//! **RIM-7: the same fans on the deep-zoom tiles and in the export** (owner,
//! 2026-10-04: *"fix the deltas disappearing on zoom"*). Past the deep-zoom
//! switch (`viewport_host.gd`'s `LOD_AUTO_ZOOM`, camera zoom 2.2) the map is
//! the tiles, and every export strokes its own rivers; both stroke the network
//! with `river_stroke::rasterize`, and until RIM-7 neither had the fans, so a
//! delta drawn at zoom 2.1 vanished at 2.3 (measured: 0 px ON-vs-OFF at z3).
//! They now stroke these very runs before the network
//! (`river_stroke::rasterize_with`) -- derived once from the same network and
//! the same drawn water the painted path reads (the tiles do not re-derive the
//! network: `WorldGen::lod_snapshot_inputs` hands them the screen's own
//! `river_geometry`) -- by the same stroke law that draws every river there,
//! so the fan is the painted one at every zoom
//! (`tests::the_stroked_fan_covers_what_the_painted_fan_covers`). One gate,
//! [`fans_drawn`], decides for all three paths whether a look has fans at all.
//!
//! **Which mouths** (the data is `OUTSTANDING_WORK.md`'s Part A, measured with
//! `_riverzoom_probe.gd --rim4-mouths`; the numbers are in this module's
//! constants' docs). A river is a delta mouth when
//! the LAST piece of its run ends meeting water (`DrawnRun::reach`'s tail --
//! never an inlet part-way along a run that goes on through a lake), its
//! `own_order` is at least [`MIN_ORDER`] and its discharge over that piece is
//! at least [`MIN_DISCHARGE_FRAC`] of the grid's cells.
//!
//! **What it is not.** Not generation: a pure function of the drawn geometry
//! and the drawn-water classification, no RNG, nothing a simulation reads and
//! no golden moves. Not the base view's vector stroke (`map_overlay.gd`, the
//! fallback for a look that keeps the stroke or a grid over the field's texel
//! budget): [`fans_drawn`] is false for such a look, so no path draws it a
//! fan -- not the stroke, the tiles or the export. Not a change
//! to any existing river: the fans are SEPARATE runs, so with the switch off
//! (`WorldGen::set_river_deltas`) the field, the colour texture, every tile and
//! every export are byte-identical to before this module (the tiles and export
//! are handed no fan geometry at all, and the network as before).
//!
//! Must never put a fan point on land farther than [`MAX_REACH_OVER_SETBACK`]
//! times its setback from the apex, never keep a branch that meets no water
//! within that reach (it would run along the shore instead of into it), and
//! never read a missing discharge as zero.

use crate::river_stroke::{DrawnRun, RiverGeometry, EDGE_FRINGE_PX, FIELD_VIEW_PX, MIN_STROKE_PX};

/// The least own Strahler order of a river that gets a fan. **Labelled
/// judgement, from the data.** Part A (`_riverzoom_probe.gd --rim4-mouths`,
/// re-measured for this change) on three worlds -- the owner's 2048x1311 (seed
/// 246371, density 1.55, geology model), 1024x656 (483920) and 1536x983
/// (912345) -- found 2203, 171 and 69 river mouths (a drawn river whose last
/// point is water), of which 138 (6.3 %), 10 (5.8 %) and 9 (13.0 %) are own
/// order >= 3, while 72.6 %, 64.9 % and 65.2 % of mouths are order 1
/// (headwater trickles that happen to reach the coast) and 21.1 %, 29.2 % and
/// 21.7 % order 2. Order is the one measure that means the same on every
/// world: discharge as a share of the grid's cells has a median of 1.5e-5 on
/// the owner world and 4.4e-4 on the 1024x656 one (30x), because it counts
/// upstream cells. Strahler order 3 is "has absorbed two order-2 rivers": the
/// first rank that is a river system's trunk rather than one of its streams.
/// **Known cost of choosing order**: `own_order` is the run's own Strahler
/// order, and the biggest-discharge mouths are not always high order (the
/// owner world's largest, 40 214 cells = 1.5 % of its grid, is own order 1, a
/// single-run outlet), so some large rivers get no fan (an open issue of the RIM-4 lane, not
/// settled here).
pub const MIN_ORDER: i16 = 3;

/// The least discharge, as a fraction of the grid's cell count, a mouth needs
/// to get a fan. **Labelled judgement, from the data** (`_riverzoom_probe.gd
/// --rim4-mouths`, discharge = the run's largest, the reading `delta_fans`
/// takes): order alone admits topological stubs -- the order >= 3 mouths of the
/// three Part A worlds read down to 6.5, 147 and 2989 cells, and the owner
/// world's order-3+ median is 469 cells (1.7e-4 of its 2 684 928). 2e-4 is that
/// world's own 90th percentile of the discharge of ALL its 2203 mouths (2.00e-4,
/// 537 cells): a fan only where the river is in the top tenth of the mouths of
/// the world it is on. It keeps 65 of the owner world's 138 order >= 3 mouths
/// by the probe's reading and 62 by the engine's (2.8 % of 2203 mouths; the
/// engine reads the last drawn piece, the probe the whole run), all 10 of the
/// 1024x656 world's (5.8 % of 171) and all 9 of the 1536x983 world's (13.0 % of
/// 69), whose own 90th percentiles are 5.8e-3 and 6.4e-3 -- so the floor only
/// bites where a world has many order-3 trickles, and a small world's order-3
/// rivers are all large. Neighbours: 1e-4 would keep 91 and 1e-3 25 of the
/// owner world's. It is a share of the grid because discharge is an upstream
/// CELL count (weighted by rainfall); a different world's 90th percentile is
/// not 2e-4, so this is the owner world's, taken as the working value.
pub const MIN_DISCHARGE_FRAC: f32 = 2.0e-4;

/// How far upstream of the shore crossing the fans start, in cells, per
/// Strahler order above 1: `(own_order - 1) * SETBACK_PER_ORDER_CELLS`, i.e.
/// 16 cells for order 3, 24 for order 4, 32 for order 5. **Labelled
/// judgement**: a delta's length scales with the river that built it, and a
/// large delta is some 200-400 km across (the Nile's ~240 km coast, the
/// Mississippi's bird's foot ~300 km); at the owner world's 19.6 km a cell
/// that is 12-20 cells, at the 1200 km regional map's 1.2 km a cell it is a
/// modest 19-38 km one. Tuned against the probe's frames, not measured.
pub const SETBACK_PER_ORDER_CELLS: f32 = 8.0;

/// The shortest setback worth a fan, in cells. A river whose last piece is
/// shorter than this above its mouth (a stub beside a coast) gets no fan: four
/// cells is what leaves the branches room to part. **Labelled judgement.**
pub const MIN_SETBACK_CELLS: f32 = 4.0;

/// The most a branch may march before it must have met water, as a multiple of
/// its fan's setback. A straight shore met at the outer branch's 45 degrees
/// takes `1 / cos 45 = 1.41` setbacks; 1.6 leaves the 0.19 for a shore that
/// bends and the branch's own curvature. A branch that has not met water by
/// then is DROPPED -- never drawn to the limit -- so a fan cannot run along a
/// coast. This is also the bound on how far onto land any fan point lies from
/// its apex. **Labelled judgement.**
pub const MAX_REACH_OVER_SETBACK: f32 = 1.6;

/// The inner distributary pair's divergence from the river's mouth heading, in
/// degrees, and the outer pair's. Order >= 4 gets both pairs (four
/// distributaries beside the trunk), order 3 the inner pair only (two).
/// **Labelled judgement**: a fan of about 90 degrees overall is what the
/// birdsfoot and arcuate deltas span (Galloway 1975's delta classification);
/// the branches start at half this angle and reach it by one setback, so they
/// leave the trunk smoothly instead of kinking.
pub const BRANCH_ANGLES_DEG: [f32; 2] = [22.0, 45.0];

/// How much each distributary narrows from its start to its tip, as a fraction
/// of its width at the start. **Labelled judgement**: distributaries thin
/// toward the water as they lose discharge to overbank flow and each other.
pub const TAPER: f32 = 0.4;

/// The march's step along a branch, in cells. **Labelled judgement**: half a
/// cell finds where a branch first stands on a water cell to better than the
/// cell the classification itself is made at.
pub const STEP_CELLS: f32 = 0.5;

/// One vertex every this many steps (one cell). Fewer vertices make the
/// field's segment list longer for no gain: a branch bends over a setback.
const VERTEX_EVERY_STEPS: usize = 2;

/// What one [`delta_fans`] call did, for `WorldGen::river_delta_stats` and the
/// probe. Every count is of MOUTHS (runs) unless it says branches.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DeltaStats {
    /// Runs whose last piece ends meeting water.
    pub mouths: usize,
    /// Of those, own order >= [`MIN_ORDER`] with a discharge reading at or
    /// above [`MIN_DISCHARGE_FRAC`] of the grid.
    pub eligible: usize,
    /// Eligible mouths that carried no finite discharge reading at all (left
    /// out, never read as zero).
    pub no_discharge: usize,
    /// Eligible mouths given a fan (at least one branch kept).
    pub fans: usize,
    /// Branches kept, over all fans.
    pub branches: usize,
    /// Branches planned and dropped for meeting no water within
    /// [`MAX_REACH_OVER_SETBACK`] setbacks (or leaving the map).
    pub dropped_dry: usize,
    /// Eligible mouths skipped because the river is under
    /// [`MIN_SETBACK_CELLS`] long above its shore crossing, has no dry point,
    /// or its apex stands on water.
    pub skipped: usize,
}

/// Whether grid cell `(i, j)` is drawn water -- the closure `river_draws`'
/// `cell_wet` is (out of the grid is not water).
pub type Wet<'a> = &'a dyn Fn(i64, i64) -> bool;

/// Whether the grid cell holding the point `p` is drawn water. Must never be
/// asked about a point outside the grid for anything but "not water": the
/// [`Wet`] closure owns that rule.
fn wet_at(wet: Wet, p: (f32, f32)) -> bool {
    wet(p.0.floor() as i64, p.1.floor() as i64)
}

/// **The delta fans of a drawn network**, as a [`RiverGeometry`] of their own
/// (one run per distributary, in network order) and what was done
/// ([`DeltaStats`]).
///
/// For each run whose last piece ends meeting water (`reach` tail), of own
/// order >= [`MIN_ORDER`] and discharge >= [`MIN_DISCHARGE_FRAC`] of
/// `gw * gh`:
///
/// 1. the shore crossing `D` is the last point of that piece that is dry;
/// 2. the apex `A` is the point `U = (own_order - 1) *
///    [`SETBACK_PER_ORDER_CELLS`]` cells of arc length upstream of `D` (less if
///    the piece is shorter; the mouth is skipped under [`MIN_SETBACK_CELLS`]);
/// 3. the fan's heading is the chord `A -> D`; one pair of distributaries at
///    [`BRANCH_ANGLES_DEG`]`[0]` either side (two pairs, at both angles, from
///    order 4) marches from `A`, bending out from half its angle to its full
///    angle over one setback, every [`STEP_CELLS`], until it first stands on
///    water, then on while it stays on water for the cap a raster draws at
///    the coarsest view (`river_field::segments`' own cap), so its end never
///    lies short of the shore's contour;
/// 4. a branch that meets no water within [`MAX_REACH_OVER_SETBACK`] setbacks
///    of its apex, or leaves the grid, is dropped;
/// 5. its width starts at the river's at `A` over `sqrt(channels)`
///    (hydraulic geometry: width goes as the square root of discharge, which
///    the fan splits `channels = branches + 1` ways) and narrows by [`TAPER`]
///    to the tip; its colour, order and own order are the river's at `A`, its
///    discharge NaN (no reading -- the fan has no cell to read one from).
///
/// `river_width` is the preset's width multiplier (`river_width_factor`), used
/// only for the water cap. Deterministic: no RNG, a function of `geom`, the
/// grid and `wet`. The network itself is only read.
pub fn delta_fans(geom: &RiverGeometry, gw: usize, gh: usize, river_width: f32, wet: Wet) -> (RiverGeometry, DeltaStats) {
    let mut st = DeltaStats::default();
    let mut out: Vec<DrawnRun> = Vec::new();
    let cells = (gw * gh) as f32;
    let s_min = gw as f32 / FIELD_VIEW_PX;
    for run in &geom.runs {
        let n = run.pts.len();
        let Some(&(s0, e0)) = run.pieces.last() else { continue };
        let tail_meets_water = run.reach.get(run.pieces.len() - 1).is_some_and(|r| r.1.is_some());
        let e0 = e0.min(n);
        if !tail_meets_water || e0 < s0 + 2 || run.widths.len() != n || run.colors.len() != n || run.orders.len() != n || run.discharge.len() != n {
            continue;
        }
        st.mouths += 1;
        if run.own_order < MIN_ORDER {
            continue;
        }
        let q = run.discharge[s0..e0].iter().copied().filter(|v| v.is_finite()).fold(None, |m: Option<f32>, v| Some(m.map_or(v, |m| m.max(v))));
        let Some(q) = q else {
            st.no_discharge += 1;
            continue;
        };
        if q < MIN_DISCHARGE_FRAC * cells {
            continue;
        }
        st.eligible += 1;
        // 1. The shore crossing: the last dry point of the piece.
        let Some(kd) = (s0..e0).rev().find(|&k| !wet_at(wet, run.pts[k])) else {
            st.skipped += 1;
            continue;
        };
        let d = run.pts[kd];
        // 2. The apex, `u_want` cells of arc length upstream of it.
        let u_want = (run.own_order as f32 - 1.0) * SETBACK_PER_ORDER_CELLS;
        let (mut acc, mut j) = (0.0f32, kd);
        let mut apex = None;
        while j > s0 {
            let (p, pp) = (run.pts[j], run.pts[j - 1]);
            let l = ((p.0 - pp.0).powi(2) + (p.1 - pp.1).powi(2)).sqrt();
            if l > 0.0 && acc + l >= u_want {
                let t = (u_want - acc) / l;
                apex = Some(((p.0 + (pp.0 - p.0) * t, p.1 + (pp.1 - p.1) * t), if t < 0.5 { j } else { j - 1 }, p, pp, t, j));
                acc = u_want;
                break;
            }
            acc += l;
            j -= 1;
        }
        // The piece was shorter than the setback: the fan starts at its head.
        let (a, ai, w_a) = match apex {
            Some((a, ai, _, _, t, jj)) => (a, ai, run.widths[jj] + (run.widths[jj - 1] - run.widths[jj]) * t),
            None => (run.pts[s0], s0, run.widths[s0]),
        };
        let u_eff = acc.min(u_want);
        if u_eff < MIN_SETBACK_CELLS || wet_at(wet, a) {
            st.skipped += 1;
            continue;
        }
        let chord = (d.0 - a.0, d.1 - a.1);
        let cl = (chord.0 * chord.0 + chord.1 * chord.1).sqrt();
        // 1e-3 cells: labelled judgement -- a mouth chord shorter than this has no
        // direction to fan about (guards the atan2 of a zero vector), far below any
        // real chord (the shortest kept setback is MIN_SETBACK_CELLS).
        if cl < 1e-3 {
            st.skipped += 1;
            continue;
        }
        let theta_c = chord.1.atan2(chord.0);
        let pairs = if run.own_order >= 4 { 2 } else { 1 };
        let channels = (2 * pairs + 1) as f32;
        let share = 1.0 / channels.sqrt();
        let lmax = MAX_REACH_OVER_SETBACK * u_eff;
        // The branch's stop width in cells: its own half-width, floored at half the
        // stroke's minimum drawn width (0.5 = half of a full width), plus the edge
        // fringe, so a tapered branch is still a drawn line when it reaches water.
        let cap = |hw: f32| (hw * river_width).max(0.5 * MIN_STROKE_PX * s_min) + EDGE_FRINGE_PX * s_min;
        let mut kept = 0usize;
        for pair in BRANCH_ANGLES_DEG.iter().take(pairs) {
            for sign in [-1.0f32, 1.0] {
                let phi = sign * pair.to_radians();
                let w0 = w_a * share;
                match march(a, theta_c, phi, u_eff, lmax, cap(0.5 * w0), gw, gh, wet) {
                    None => st.dropped_dry += 1,
                    Some(pts) => {
                        let total = pts.windows(2).map(|s| ((s[1].0 - s[0].0).powi(2) + (s[1].1 - s[0].1).powi(2)).sqrt()).sum::<f32>().max(1e-6);
                        let mut at = 0.0f32;
                        let mut widths = Vec::with_capacity(pts.len());
                        for (k, p) in pts.iter().enumerate() {
                            if k > 0 {
                                at += ((p.0 - pts[k - 1].0).powi(2) + (p.1 - pts[k - 1].1).powi(2)).sqrt();
                            }
                            widths.push(w0 * (1.0 - TAPER * (at / total)));
                        }
                        let m = pts.len();
                        out.push(DrawnRun {
                            pts,
                            widths,
                            colors: vec![run.colors[ai]; m],
                            orders: vec![run.orders[ai]; m],
                            discharge: vec![f32::NAN; m],
                            pieces: vec![(0, m)],
                            reach: vec![(None, None)],
                            own_order: run.own_order,
                        });
                        kept += 1;
                    }
                }
            }
        }
        if kept > 0 {
            st.fans += 1;
            st.branches += kept;
        }
    }
    (RiverGeometry { runs: out }, st)
}

/// One distributary's centreline from `apex`, or `None` if it meets no water.
/// Heading starts at `theta_c + phi / 2` and reaches `theta_c + phi` over
/// `u_eff` cells of march; each [`STEP_CELLS`] it moves on, and at the first
/// step that stands on water it carries on while it stays on water, up to
/// `cap` cells more (the cap a raster draws, so the end is wholly in the
/// water). `None` past `lmax` cells of march (a branch that would run along
/// the shore) or when it leaves the grid. The returned points start at the
/// apex, one every [`VERTEX_EVERY_STEPS`] steps, and end on water.
#[allow(clippy::too_many_arguments)]
fn march(apex: (f32, f32), theta_c: f32, phi: f32, u_eff: f32, lmax: f32, cap: f32, gw: usize, gh: usize, wet: Wet) -> Option<Vec<(f32, f32)>> {
    let mut pts = vec![apex];
    let mut p = apex;
    let mut s = 0.0f32;
    let mut k = 0usize;
    let mut on_water_since: Option<f32> = None;
    loop {
        s += STEP_CELLS;
        k += 1;
        if s > lmax + cap {
            return None;
        }
        let heading = theta_c + phi * (0.5 + 0.5 * (s / u_eff).min(1.0));
        let next = (p.0 + heading.cos() * STEP_CELLS, p.1 + heading.sin() * STEP_CELLS);
        if next.0 < 0.0 || next.1 < 0.0 || next.0 >= gw as f32 || next.1 >= gh as f32 {
            return None;
        }
        let wet_now = wet_at(wet, next);
        match on_water_since {
            None => {
                if s > lmax {
                    return None;
                }
                if wet_now {
                    on_water_since = Some(s);
                    p = next;
                    pts.push(p);
                    if cap <= 0.0 {
                        return Some(pts);
                    }
                    continue;
                }
                p = next;
                if k % VERTEX_EVERY_STEPS == 0 {
                    pts.push(p);
                }
            }
            Some(s_w) => {
                // Stay on water for the cap, then stop; leave it and stop at
                // the last wet point (a strait: the branch has crossed it).
                if !wet_now || s - s_w >= cap {
                    return Some(pts);
                }
                p = next;
                pts.push(p);
            }
        }
    }
}

/// **RIM-7: whether a look draws delta fans at all** -- the one gate the
/// painted screen path (`WorldGen::build_color_texture`), the deep-zoom tiles
/// (`WorldGen::lod_snapshot_inputs`) and the export
/// (`WorldGen::export_river_fans`) share, so no zoom and no export can
/// show a fan the screen does not, or drop one it does.
///
/// True when the off switch is clear (`off` is `WorldGen::set_river_deltas`
/// negated) and the look paints its rivers into the map -- `smooth_shores`
/// and `rivers_as_water`, the painted path's own condition -- on a grid whose
/// river field fits its texel budget (`river_field::field_scale`). Each other
/// case is a base view that draws the vector stroke (`map_overlay.gd`), which
/// draws no fan, so the tiles and the export of that look must not either:
/// a fan appearing only past the deep-zoom switch would be the same pop this
/// gate exists to remove, the other way round.
///
/// Must never read anything but its arguments: the three callers pass the
/// appearance they draw with (an export its own style override), and the
/// answer must be the same function of it everywhere.
pub fn fans_drawn(a: &crate::render::TerrainAppearance, off: bool, gw: usize, gh: usize) -> bool {
    !off && a.smooth_shores && a.rivers_as_water && crate::river_field::field_scale(gw, gh).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::river_stroke::ShoreReach;

    const GW: usize = 100;
    const GH: usize = 100;

    /// A run heading east along `y = 50.5` from `x = 10.5` to `x_end`, one
    /// point a cell, whose last piece ends meeting water (`reach` tail set),
    /// of order `order` and discharge `q` everywhere. Full width 1 cell.
    fn run(order: i16, q: f32, x_end: f32, tail: bool) -> DrawnRun {
        let n = (x_end - 10.5) as usize + 1;
        DrawnRun {
            pts: (0..n).map(|i| (10.5 + i as f32, 50.5)).collect(),
            widths: vec![1.0; n],
            colors: vec![[0.2, 0.4, 0.8, 1.0]; n],
            orders: vec![order; n],
            discharge: vec![q; n],
            pieces: vec![(0, n)],
            reach: vec![(None, tail.then(ShoreReach::default))],
            own_order: order,
        }
    }

    /// [`run`] starting at `x0` instead of 10.5.
    fn run_from(x0: f32, order: i16, q: f32, x_end: f32) -> DrawnRun {
        let mut r = run(order, q, x_end, true);
        let skip = (x0 - 10.5) as usize;
        r.pts.drain(..skip);
        r.widths.drain(..skip);
        r.colors.drain(..skip);
        r.orders.drain(..skip);
        r.discharge.drain(..skip);
        r.pieces = vec![(0, r.pts.len())];
        r
    }

    fn east_coast(x: i64, _y: i64) -> bool {
        x >= 60
    }

    fn fans_of(r: DrawnRun, wet: Wet) -> (RiverGeometry, DeltaStats) {
        delta_fans(&RiverGeometry { runs: vec![r] }, GW, GH, 1.0, wet)
    }

    /// Protects: the fan exists and has the designed shape -- an order-3 mouth
    /// gets exactly two distributaries, an order-4 mouth four; each starts at
    /// the apex `(own_order - 1) * SETBACK_PER_ORDER_CELLS` cells upstream of
    /// the last dry point, ends ON water, and never lies on land farther than
    /// `MAX_REACH_OVER_SETBACK` setbacks from the apex. A dropped fan, a wrong
    /// setback or a wrong branch count fails here.
    #[test]
    fn a_big_mouth_gets_a_fan_that_starts_upstream_and_ends_on_water() {
        for (order, want_branches) in [(3i16, 2usize), (4, 4), (5, 4)] {
            let (fan, st) = fans_of(run(order, 5000.0, 62.5, true), &east_coast);
            assert_eq!((st.mouths, st.eligible, st.fans, st.branches), (1, 1, 1, want_branches), "order {order}: {st:?}");
            assert_eq!(fan.runs.len(), want_branches);
            // Last dry point is x = 59.5; the apex is the setback upstream.
            let setback = (order as f32 - 1.0) * SETBACK_PER_ORDER_CELLS;
            let apex_x = 59.5 - setback;
            for r in &fan.runs {
                let (a, e) = (r.pts[0], *r.pts.last().unwrap());
                assert!((a.0 - apex_x).abs() < 1e-3 && (a.1 - 50.5).abs() < 1e-3, "order {order}: apex {a:?}, want x {apex_x}");
                assert!(east_coast(e.0.floor() as i64, e.1.floor() as i64), "a branch ends on water: {e:?}");
                for p in &r.pts {
                    let d = ((p.0 - a.0).powi(2) + (p.1 - a.1).powi(2)).sqrt();
                    let on_land = !east_coast(p.0.floor() as i64, p.1.floor() as i64);
                    assert!(!on_land || d <= MAX_REACH_OVER_SETBACK * setback + 1e-3, "land point {p:?} is {d} from the apex");
                }
                assert_eq!(r.own_order, order);
                assert!(r.discharge.iter().all(|v| v.is_nan()), "a fan has no discharge reading, and must never fake one");
                assert_eq!(r.colors[0], [0.2, 0.4, 0.8, 1.0]);
            }
        }
    }

    /// Protects: the branches go either side of the river in equal measure
    /// (a symmetric fan on a symmetric coast), start at half their angle, and
    /// the outer pair is wider than the inner -- the angle constants'
    /// RELATIONS are checked here (symmetry, half-angle start, outer wider);
    /// their values are pinned by literals in
    /// `the_tuned_constants_are_pinned_by_literals_not_by_themselves`.
    #[test]
    fn the_fan_is_symmetric_and_the_outer_pair_is_wider() {
        let (fan, _) = fans_of(run(4, 5000.0, 62.5, true), &east_coast);
        let first_dir = |r: &DrawnRun| (r.pts[1].1 - r.pts[0].1).atan2(r.pts[1].0 - r.pts[0].0).to_degrees();
        let mut angles: Vec<f32> = fan.runs.iter().map(first_dir).collect();
        angles.sort_by(|a, b| a.partial_cmp(b).unwrap());
        // Over the first cell the heading is 0.5 phi growing toward phi: a
        // little over half the angle. The tolerance is the growth over it.
        let (inner, outer) = (BRANCH_ANGLES_DEG[0], BRANCH_ANGLES_DEG[1]);
        for (got, want) in angles.iter().zip([-outer, -inner, inner, outer]) {
            assert!((got - want * 0.5).abs() < 0.12 * want.abs(), "first heading {got} vs half of {want}: {angles:?}");
        }
        // Symmetry in y of the ends.
        let ends: Vec<f32> = fan.runs.iter().map(|r| r.pts.last().unwrap().1 - 50.5).collect();
        let (lo, hi) = (ends.iter().cloned().fold(f32::MAX, f32::min), ends.iter().cloned().fold(f32::MIN, f32::max));
        assert!((lo + hi).abs() < 1.0, "the fan leans: {ends:?}");
        assert!(hi > 6.0, "the outer branch spreads wider than the river's own width: {ends:?}");
    }

    /// Protects: a river that is not a big mouth gets no fan -- order 2
    /// (`MIN_ORDER`), a Strahler-3 stub with a trickle of discharge
    /// (`MIN_DISCHARGE_FRAC`), a run whose tail meets no water (a confluence
    /// or an inland end) and a run whose only shore end is an INLET with the
    /// run going on (a through-lake river) -- and a missing reading is counted,
    /// not read as zero.
    #[test]
    fn only_a_large_river_that_ends_in_water_gets_a_fan() {
        let none = |r: DrawnRun| fans_of(r, &east_coast);
        let (f, st) = none(run(2, 5000.0, 62.5, true));
        assert!(f.runs.is_empty() && st.mouths == 1 && st.eligible == 0, "{st:?}");
        let (f, st) = none(run(3, MIN_DISCHARGE_FRAC * (GW * GH) as f32 * 0.9, 62.5, true));
        assert!(f.runs.is_empty() && st.eligible == 0, "{st:?}");
        let (f, st) = none(run(3, MIN_DISCHARGE_FRAC * (GW * GH) as f32 * 1.1, 62.5, true));
        assert!(!f.runs.is_empty() && st.eligible == 1, "just above the floor gets one: {st:?}");
        let (f, st) = none(run(5, 9e9, 55.5, false));
        assert!(f.runs.is_empty() && st.mouths == 0, "a run that ends on land is no mouth: {st:?}");
        // A through-lake river: first piece ends at an inlet (tail set), the
        // second piece ends inland.
        let mut thru = run(5, 9e9, 62.5, false);
        let n = thru.pts.len();
        thru.pieces = vec![(0, n / 2), (n / 2, n)];
        thru.reach = vec![(None, Some(ShoreReach::default())), (None, None)];
        let (f, st) = none(thru);
        assert!(f.runs.is_empty() && st.mouths == 0, "an inlet is not a mouth: {st:?}");
        let mut nodis = run(5, 0.0, 62.5, true);
        nodis.discharge = vec![f32::NAN; nodis.pts.len()];
        let (f, st) = none(nodis);
        assert!(f.runs.is_empty() && st.no_discharge == 1 && st.eligible == 0, "{st:?}");
    }

    /// Protects: `MIN_SETBACK_CELLS` and the short-piece rule -- an order-4
    /// river whose last piece is under the minimum above its mouth gets no
    /// fan (it is counted skipped, not drawn as a stub), one just over it gets
    /// a fan that starts at the piece's head rather than the full setback up.
    #[test]
    fn a_river_too_short_above_its_mouth_gets_no_fan() {
        // The last dry point is x = 59.5: a run from 56.5 is 3 cells above it,
        // one from 54.5 is 5.
        let short = run_from(56.5, 4, 5000.0, 62.5);
        let (f, st) = fans_of(short, &east_coast);
        assert!(f.runs.is_empty() && st.skipped == 1, "3 cells above the mouth is under the minimum: {st:?}");
        let (f, st) = fans_of(run_from(54.5, 4, 5000.0, 62.5), &east_coast);
        assert_eq!(st.fans, 1, "{st:?}");
        // The piece (5 cells) is shorter than the 24-cell setback: the apex is its head.
        assert!(f.runs.iter().all(|r| (r.pts[0].0 - 54.5).abs() < 1e-3), "{:?}", f.runs.iter().map(|r| r.pts[0]).collect::<Vec<_>>());
    }

    /// Protects: no leak along a shore. With the water only on one side of
    /// the river's line (a coast running along it, the sea to the south), the
    /// branches that would have to run along the land to meet it are DROPPED,
    /// not drawn to the limit; and no kept branch has any point on water
    /// before its end.
    #[test]
    fn a_branch_that_meets_no_water_is_dropped_not_drawn_along_the_shore() {
        // The river runs east on y = 50.5 and the sea lies only south of
        // y = 52 for x >= 54: a coast along the river's south side, so the
        // branches heading north have no water to find at all.
        let bay = |x: i64, y: i64| x >= 54 && y >= 52;
        let (fan, st) = fans_of(run(4, 5000.0, 62.5, true), &bay);
        assert_eq!(st.dropped_dry, 2, "the two branches heading north find no water: {st:?}");
        assert_eq!(st.branches + st.dropped_dry, 4, "every planned branch is kept or dropped: {st:?}");
        assert_eq!(fan.runs.len(), st.branches);
        for r in &fan.runs {
            let last = r.pts.len() - 1;
            let first_wet = r.pts.iter().position(|p| bay(p.0.floor() as i64, p.1.floor() as i64)).expect("a kept branch reaches water");
            assert!(first_wet >= 1, "the apex is dry");
            let setback = 3.0 * SETBACK_PER_ORDER_CELLS;
            for p in &r.pts[..first_wet] {
                let d = ((p.0 - r.pts[0].0).powi(2) + (p.1 - r.pts[0].1).powi(2)).sqrt();
                assert!(d <= MAX_REACH_OVER_SETBACK * setback + 1e-3, "{p:?}");
            }
            assert!(last >= first_wet);
        }
        // A world with no water in reach at all: nothing drawn, nothing leaks.
        let (fan, st) = fans_of(run(4, 5000.0, 62.5, true), &|_, _| false);
        assert!(fan.runs.is_empty(), "no water, no fan: {st:?}");
    }

    /// Protects: distributary widths -- each starts at the river's width over
    /// the square root of the channels the fan splits it into (a pair: 1 / sqrt
    /// 3; two pairs: 1 / sqrt 5) and narrows by `TAPER` to the tip, strictly,
    /// never to zero; so a mutated share or taper fails.
    #[test]
    fn distributaries_share_the_width_and_taper() {
        for (order, channels) in [(3i16, 3.0f32), (4, 5.0)] {
            let (fan, _) = fans_of(run(order, 5000.0, 62.5, true), &east_coast);
            for r in &fan.runs {
                let w0 = r.widths[0];
                assert!((w0 - 1.0 / channels.sqrt()).abs() < 1e-5, "order {order}: start {w0}");
                let tip = *r.widths.last().unwrap();
                assert!((tip - w0 * (1.0 - TAPER)).abs() < 1e-4 && tip > 0.0, "tip {tip} of {w0}");
                assert!(r.widths.windows(2).all(|v| v[1] <= v[0] + 1e-6));
            }
        }
    }

    /// Protects: the tuned constants against a silent retune. Every other test
    /// here derives its expectation from the constant it checks (`0.9 *
    /// MIN_DISCHARGE_FRAC`, `1 - TAPER`, the setback from `SETBACK_PER_ORDER_CELLS`),
    /// so a changed constant moves test and code together and nothing fails
    /// (the RIM-4 mutation run left 15 mutants of 8 constants alive for this reason). This test uses LITERAL
    /// numbers measured once on the shipped values (2026-10-04, the shapes of
    /// `east_coast` and a coast at `x >= 69` / `75`), so retuning a constant fails
    /// here and the change must be made on purpose and re-judged on the screen,
    /// not by accident: the discharge floor (2e-4 of 10 000 cells = 2.0 cells), the
    /// setback per order (8 cells), the reach limit (1.6 setbacks, 38.4 cells here: a coast 30.5 cells
    /// from the apex keeps the inner pair and drops the outer, one at 36.5 cells
    /// drops all four), the branch angles (first heading 11.3 and 23.2 degrees,
    /// ends at +-7.6 and +-18.3 cells across), the taper (tip at 60 % of the
    /// start) and the march (one vertex a cell). Neighbours the run kills:
    /// floor x0.5 / x2, setback 6 / 10, reach 1.3 / 1.7 / 2.0, inner angle 15 / 30,
    /// outer 35 / 60, taper 0.2 / 0.6, step 0.25 / 1.0, vertex 1 / 4.
    #[test]
    fn the_tuned_constants_are_pinned_by_literals_not_by_themselves() {
        let near = |a: f32, b: f32, tol: f32, what: &str| assert!((a - b).abs() <= tol, "{what}: {a} vs {b}");
        // The discharge floor: 2.0 cells on this 10 000-cell grid.
        let at = |q: f32| fans_of(run(3, q, 62.5, true), &east_coast).1.eligible;
        assert_eq!((at(1.9), at(2.1)), (0, 1), "the floor is 2e-4 of the grid's cells");
        // The setback: apex = last dry point (59.5) - (order - 1) * 8.
        for (order, apex_x) in [(3i16, 43.5f32), (4, 35.5), (5, 27.5)] {
            let (fan, _) = fans_of(run(order, 5000.0, 62.5, true), &east_coast);
            for r in &fan.runs {
                near(r.pts[0].0, apex_x, 1e-3, "apex x");
            }
        }
        // The reach limit. The run ends dry at x = 62.5 (apex x = 38.5 for
        // order 4, setback 24, limit 38.4 cells); the sea starts at `cx`.
        let wall = |cx: i64| {
            let w = move |x: i64, _y: i64| x >= cx;
            let (f, st) = fans_of(run(4, 5000.0, 62.5, true), &w);
            (f.runs.len(), st.dropped_dry)
        };
        assert_eq!(wall(66), (4, 0), "a sea 27.5 cells away is reached by all four");
        assert_eq!(wall(69), (2, 2), "at 30.5 cells only the inner pair reaches it within the limit");
        // Measured edges of the 38.4-cell limit: the outer pair still reaches a
        // sea at x = 68 and the inner pair one at x = 74 (a 1.5-setback limit
        // loses both).
        assert_eq!(wall(68), (4, 0), "the outer pair reaches a sea 29.5 cells away");
        assert_eq!(wall(74), (2, 2), "the inner pair reaches a sea 35.5 cells away");
        assert_eq!(wall(75), (0, 4), "at 36.5 cells none does: the branches would run along dry land");
        // The angles, by the first heading and by where the branches end.
        let (fan, _) = fans_of(run(4, 5000.0, 62.5, true), &east_coast);
        let mut firsts: Vec<f32> = fan.runs.iter().map(|r| (r.pts[1].1 - r.pts[0].1).atan2(r.pts[1].0 - r.pts[0].0).to_degrees()).collect();
        firsts.sort_by(|a, b| a.partial_cmp(b).unwrap());
        for (got, want) in firsts.iter().zip([-23.2f32, -11.34, 11.34, 23.2]) {
            near(*got, want, 0.4, "first heading");
        }
        let mut ends: Vec<f32> = fan.runs.iter().map(|r| r.pts.last().unwrap().1 - 50.5).collect();
        ends.sort_by(|a, b| a.partial_cmp(b).unwrap());
        for (got, want) in ends.iter().zip([-18.28f32, -7.6, 7.6, 18.28]) {
            near(*got, want, 0.3, "end offset across the river");
        }
        // The taper and the march.
        for r in &fan.runs {
            near(*r.widths.last().unwrap() / r.widths[0], 0.6, 1e-3, "tip width over start width");
            let sp: Vec<f32> = r.pts.windows(2).map(|w| ((w[1].0 - w[0].0).powi(2) + (w[1].1 - w[0].1).powi(2)).sqrt()).collect();
            for d in &sp[..sp.len() - 1] {
                near(*d, 1.0, 0.01, "vertex spacing on land (STEP_CELLS x VERTEX_EVERY_STEPS)");
            }
        }
    }

    /// Protects: determinism and purity -- two calls agree bit for bit, and
    /// the network handed in is not touched.
    #[test]
    fn the_fans_are_a_pure_deterministic_function_of_the_network() {
        let g = RiverGeometry { runs: vec![run(4, 5000.0, 62.5, true)] };
        let before: Vec<(f32, f32)> = g.runs[0].pts.clone();
        let (a, sa) = delta_fans(&g, GW, GH, 1.0, &east_coast);
        let (b, sb) = delta_fans(&g, GW, GH, 1.0, &east_coast);
        assert_eq!(sa, sb);
        assert_eq!(a.runs.len(), b.runs.len());
        for (x, y) in a.runs.iter().zip(&b.runs) {
            assert_eq!(x.pts.iter().map(|p| (p.0.to_bits(), p.1.to_bits())).collect::<Vec<_>>(), y.pts.iter().map(|p| (p.0.to_bits(), p.1.to_bits())).collect::<Vec<_>>());
            assert_eq!(x.widths.iter().map(|v| v.to_bits()).collect::<Vec<_>>(), y.widths.iter().map(|v| v.to_bits()).collect::<Vec<_>>());
        }
        assert_eq!(g.runs[0].pts, before);
    }

    fn field_of(g: &RiverGeometry, fans: Option<&RiverGeometry>) -> crate::river_field::RiverField {
        crate::river_field::build_with(g, fans, GW, GH, 1.0, &|_, _| 0.0).expect("a field")
    }

    /// Protects: the off switch. With no fans the field is bit-identical to the
    /// plain build (`WorldGen::set_river_deltas(false)` takes this path: the
    /// fans are never derived), and so is a fan geometry with no runs; with
    /// fans it differs. And the colour field, likewise.
    #[test]
    fn the_off_switch_is_the_plain_field() {
        let g = RiverGeometry { runs: vec![run(4, 5000.0, 62.5, true)] };
        let plain = crate::river_field::build(&g, GW, GH, 1.0, &|_, _| 0.0).expect("a field");
        let off = field_of(&g, None);
        assert_eq!(plain.bits, off.bits);
        assert_eq!(plain.segments, off.segments);
        let empty = RiverGeometry { runs: vec![] };
        assert_eq!(plain.bits, field_of(&g, Some(&empty)).bits);
        let (fans, _) = delta_fans(&g, GW, GH, 1.0, &east_coast);
        let on = field_of(&g, Some(&fans));
        assert!(on.segments > plain.segments && on.bits != plain.bits, "positive control: the fan changes the field");
    }

    /// Protects: `a_fan_only_adds_coverage` -- at every texel the shader's
    /// coverage with the fans is at least the network's alone (at fit and a
    /// deep density), so no existing river pixel is thinned or removed; the
    /// field changes only within the fans' own reach of the apex and the
    /// shore (no leak elsewhere); and the changed area is large enough to be
    /// a visible fan (a dropped fan changes nothing and fails the count).
    #[test]
    fn a_fan_only_adds_coverage_and_only_near_its_mouth() {
        let g = RiverGeometry { runs: vec![run(4, 5000.0, 62.5, true)] };
        let (fans, st) = delta_fans(&g, GW, GH, 1.0, &east_coast);
        assert_eq!(st.branches, 4);
        let base = field_of(&g, None);
        let with = field_of(&g, Some(&fans));
        let setback = 3.0 * SETBACK_PER_ORDER_CELLS;
        let apex = (59.5 - setback, 50.5);
        let reach = MAX_REACH_OVER_SETBACK * setback + 6.0;
        for ppc in [0.5f32, 2.0] {
            let mut gained = 0usize;
            for j in 0..base.h {
                for i in 0..base.w {
                    let (x, y) = ((i as f32 + 0.5) / base.scale as f32, (j as f32 + 0.5) / base.scale as f32);
                    let (cb, _) = crate::river_field::coverage(base.texel(i as i64, j as i64), ppc, 1.0);
                    let (cw, _) = crate::river_field::coverage(with.texel(i as i64, j as i64), ppc, 1.0);
                    assert!(cw + 1e-4 >= cb, "coverage fell at ({x},{y}): {cb} -> {cw}");
                    if cw > cb + 0.05 {
                        gained += 1;
                        let d = ((x - apex.0).powi(2) + (y - apex.1).powi(2)).sqrt();
                        assert!(d <= reach, "coverage gained {d} cells from the apex at ({x},{y}), past the fan's reach {reach}");
                        assert!(x >= apex.0 - 1.0, "a fan texel upstream of its own apex: ({x},{y})");
                    }
                }
            }
            assert!(gained > 30, "ppc {ppc}: the fan painted only {gained} texels more");
        }
    }

    /// Protects: the colour field -- with the fans the colour field covers the
    /// fan's own cells with a river colour (it would sample terrain without
    /// them) and the network's own cells are exactly the plain field's; with
    /// `None` the whole layer is the plain one.
    #[test]
    fn the_colour_field_gets_the_fans_and_keeps_the_network_as_it_was() {
        let a = crate::render::TerrainAppearance::default();
        let g = RiverGeometry { runs: vec![run(4, 5000.0, 62.5, true)] };
        let (fans, _) = delta_fans(&g, GW, GH, 1.0, &east_coast);
        let plain = crate::river_stroke::rasterize_colour_field(&g, &a, GW, GH);
        let none = crate::river_stroke::rasterize_colour_field_with(&g, None, &a, GW, GH);
        let with = crate::river_stroke::rasterize_colour_field_with(&g, Some(&fans), &a, GW, GH);
        let mut fan_cells = 0usize;
        for y in 0..GH {
            for x in 0..GW {
                assert_eq!(plain.at(x, y), none.at(x, y), "None is the plain colour field");
                if (y as i64 - 50).abs() == 0 && x < 60 {
                    assert_eq!(plain.at(x, y), with.at(x, y), "the network's own cells are untouched at ({x},{y})");
                }
                if plain.at(x, y).is_none() && with.at(x, y).is_some() {
                    fan_cells += 1;
                }
            }
        }
        assert!(fan_cells > 100, "the fan reached only {fan_cells} colour cells");
        // A cell on the outer branch's path, well off the river's own band.
        let r = fans.runs.iter().max_by(|p, q| p.pts.last().unwrap().1.partial_cmp(&q.pts.last().unwrap().1).unwrap()).unwrap();
        let mid = r.pts[r.pts.len() / 2];
        assert!(with.at(mid.0 as usize, mid.1 as usize).is_some(), "a branch's own cell has a river colour: {mid:?}");
    }

    /// Protects: RIM-7's one gate. A fan is drawn (screen, tiles, export) only
    /// with the switch on AND a look that paints its rivers (`smooth_shores`
    /// and `rivers_as_water`) AND a grid whose river field fits its texel
    /// budget; each term alone turns it off. The grids are literals, not
    /// `MAX_TEXELS` arithmetic: the owner's 2048x1311 and a 4096x2622 map fit,
    /// an 8192x5240 one does not (`river_field::field_scale`'s `None`) and
    /// keeps the stroke, which draws no fan. Mutating any term goes red.
    #[test]
    fn fans_are_drawn_only_where_the_painted_river_is() {
        let a = crate::render::TerrainAppearance::default();
        assert!(a.smooth_shores && a.rivers_as_water, "precondition: the default look paints its rivers");
        assert!(fans_drawn(&a, false, 2048, 1311), "the owner's world, switch on");
        assert!(fans_drawn(&a, false, 4096, 2622), "a 4096 map fits the field at one texel a cell");
        assert!(!fans_drawn(&a, true, 2048, 1311), "the off switch");
        assert!(!fans_drawn(&a, false, 8192, 5240), "a grid over the field's budget keeps the stroke, which has no fan");
        let stroke_look = crate::render::TerrainAppearance { rivers_as_water: false, ..a.clone() };
        assert!(!fans_drawn(&stroke_look, false, 2048, 1311), "a look that keeps the stroke");
        let cell_coast = crate::render::TerrainAppearance { smooth_shores: false, ..a.clone() };
        assert!(!fans_drawn(&cell_coast, false, 2048, 1311), "a look with no smooth shore paints no river");
        assert!(!fans_drawn(&crate::render::TerrainAppearance::js_reference(), false, 2048, 1311), "the reference look has no river layer");
    }

    /// Protects: the order the tiles and the export stroke in
    /// (`river_stroke::rasterize_with`) -- the fans FIRST, the network over
    /// them, so wherever the network is opaque its pixel is exactly the
    /// network's alone (`rasterize`), whatever colour a fan is; the fan shows
    /// beside the network, and `None` is `rasterize` bit for bit. The fans are
    /// recoloured red here so the order is visible: drawn last, a fan's own
    /// core over the trunk at its apex would turn trunk pixels red, which
    /// fails the first loop.
    #[test]
    fn the_stroke_draws_the_fans_first_and_the_network_over_them() {
        let a = crate::render::TerrainAppearance::default();
        let g = RiverGeometry { runs: vec![run(4, 5000.0, 62.5, true)] };
        let (mut fans, st) = delta_fans(&g, GW, GH, 1.0, &east_coast);
        assert_eq!(st.branches, 4, "positive control: the order-4 mouth fans, {st:?}");
        for r in &mut fans.runs {
            for c in &mut r.colors {
                *c = [1.0, 0.0, 0.0, 1.0];
            }
        }
        let map = crate::river_stroke::RasterMap { scale: (4.0, 4.0), offset: (0.0, 0.0) };
        let (pw, ph) = (GW * 4, GH * 4);
        let net = crate::river_stroke::rasterize(&g, &a, pw, ph, map);
        let none = crate::river_stroke::rasterize_with(&g, None, &a, pw, ph, map);
        let with = crate::river_stroke::rasterize_with(&g, Some(&fans), &a, pw, ph, map);
        let (mut opaque, mut fan_only) = (0usize, 0usize);
        for y in 0..ph {
            for x in 0..pw {
                assert_eq!(net.at(x, y).map(|p| p.map(f32::to_bits)), none.at(x, y).map(|p| p.map(f32::to_bits)), "None is rasterize at ({x},{y})");
                match net.at(x, y) {
                    Some(p) if p[3] >= 1.0 => {
                        opaque += 1;
                        assert_eq!(with.at(x, y), Some(p), "the network's opaque pixel ({x},{y}) is the network's");
                    }
                    None if with.at(x, y).is_some() => fan_only += 1,
                    _ => {}
                }
            }
        }
        assert!(opaque > 100 && fan_only > 100, "the trunk ({opaque} px) and the fans beside it ({fan_only} px) both drew");
    }

    /// Protects: **no pop at the deep-zoom switch** (RIM-7). The painted path
    /// draws the fan from the river field (`river_field::build_with` and the
    /// shader's [`crate::river_field::coverage`]); the deep-zoom tiles and the
    /// export stroke it (`river_stroke::rasterize_with`). Over the fan's
    /// ground on land (from just
    /// past its apex to the shore -- the tips lie on water, which both paths
    /// hide under the water's own colour), the two cover the same pixels
    /// within the antialiasing band at the densities the painted path is shown
    /// at around the switch (0.5 to 2 px a cell; the owner's world crosses it
    /// near 1.3). A tile path that dropped the fans, or stroked them at another
    /// width, fails the total; one that placed them elsewhere fails the
    /// per-pixel term. The bounds (2 % of the stroke's coverage in total, 3 %
    /// per pixel) are a labelled judgement over the run of 2026-10-04, which
    /// measured 0.16-0.97 % in total and 0.65-2.27 % per pixel at 0.5, 1, 1.5
    /// and 2 (the straight-line case,
    /// `river_field::tests::the_field_covers_what_the_stroke_covers`, holds
    /// 2 %). **Not asserted at 3 or 4 px a cell**, where the same run measured
    /// the field 3.2 % / 4.3 % short of the stroke (5.9 % per pixel): a
    /// branch under half a cell wide is narrowed there by the field's own
    /// quarter-cell bilinear error (`river_field`'s module doc, "Resolution
    /// and memory"), which is the painted path's limit for every thin river,
    /// not the fans'. On the owner's world that density is reached only past
    /// the switch, where the tiles stroke the fan instead.
    #[test]
    fn the_stroked_fan_covers_what_the_painted_fan_covers() {
        let a = crate::render::TerrainAppearance::default();
        let rw = a.river_width as f32;
        let g = RiverGeometry { runs: vec![run(4, 5000.0, 62.5, true)] };
        let (fans, st) = delta_fans(&g, GW, GH, rw, &east_coast);
        assert_eq!(st.branches, 4, "{st:?}");
        let field = crate::river_field::build_with(&g, Some(&fans), GW, GH, rw, &|_, _| 0.0).expect("a field");
        let plain = crate::river_field::build(&g, GW, GH, rw, &|_, _| 0.0).expect("a field");
        // The apex is at x = 35.5 (`the_tuned_constants_are_pinned_by_literals_not_by_themselves`);
        // the shore at x = 60.
        let (x_lo, x_hi) = (37.0f32, 59.5f32);
        for ppc in [0.5f32, 1.0, 1.5, 2.0] {
            let (pw, ph) = ((GW as f32 * ppc) as usize, (GH as f32 * ppc) as usize);
            let layer = crate::river_stroke::rasterize_with(&g, Some(&fans), &a, pw, ph, crate::river_stroke::RasterMap { scale: (ppc, ppc), offset: (0.0, 0.0) });
            let (mut stroke_px, mut field_px, mut diff, mut fan_px) = (0.0f64, 0.0f64, 0.0f64, 0.0f64);
            for py in 0..ph {
                for px in 0..pw {
                    let (x, y) = (px as f32 / ppc, py as f32 / ppc);
                    if x < x_lo || x > x_hi {
                        continue;
                    }
                    let s = layer.at(px, py).map_or(0.0, |p| p[3]) as f64;
                    let (c, am) = crate::river_field::coverage(field.sample(x, y), ppc, rw);
                    let fv = (c * am) as f64;
                    let (c0, am0) = crate::river_field::coverage(plain.sample(x, y), ppc, rw);
                    stroke_px += s;
                    field_px += fv;
                    diff += (s - fv).abs();
                    fan_px += (fv - (c0 * am0) as f64).max(0.0);
                }
            }
            // Positive control: the window holds a real fan, not just the trunk.
            assert!(fan_px > 0.3 * field_px, "ppc {ppc}: the fan is {fan_px:.1} of {field_px:.1} covered px");
            let rel = (field_px - stroke_px).abs() / stroke_px;
            assert!(rel < 0.02 && diff / stroke_px < 0.03, "ppc {ppc}: stroke {stroke_px:.1} painted {field_px:.1} ({:.2}%), per-pixel {:.2}%", rel * 100.0, diff / stroke_px * 100.0);
        }
    }
}
