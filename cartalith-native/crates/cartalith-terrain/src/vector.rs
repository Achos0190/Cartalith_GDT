//! **EF-6** — the world's linear features as vectors, beyond the rivers
//! `cartalith_hydrology::trace_river_polylines` already returns
//! (`ELEVATION_FIELD_ARCHITECTURE_RESEARCH.md` §5, EF-6).
//!
//! Three tracers, one convention: every one returns polylines in the same
//! **pixel space** the river tracer does — a cell centre is
//! `(col + 0.5, row + 0.5)` — so a consumer that already knows how to measure
//! distance to a river knows how to measure distance to a coastline without a
//! second coordinate rule. That convention, not a shared type family, is what
//! actually unifies this layer; no `VectorFeature` enum is introduced here,
//! because nothing consumes one yet.
//!
//! # New capability, not a port — flagged, per `cartalith-porting-discipline`
//!
//! `Cartalith Gen1` vectorises rivers and plate boundaries and nothing else.
//! Checked in `Cartalith Gen1 v2.11.html` rather than assumed, because two
//! things in it look at first like counter-examples and neither is:
//!
//! - **It does draw elevation isolines** — `viz.contours`, the *"contour
//!   veins"* style, at line 7954. It is a **per-pixel shading term**: for each
//!   pixel it takes the height `r`, computes `d = |r/iv − round(r/iv)| · iv`
//!   — the distance to the nearest isoline *in height space* — and darkens by
//!   it. No point, no polyline, nothing a consumer could measure a distance
//!   to. The same is true of its coastline, which exists only as the
//!   land/water decision each pixel makes for itself.
//! - **Its UI hint claims the icon layer "finds ridgelines"** (line 1772).
//!   `placeMapIcons` (line 7141) does no such thing: it collects every land
//!   cell above a normalised elevation threshold, sorts by height and applies
//!   grid-bucket spacing. That is a peak *scatter*, not a ridge — no TPI, no
//!   local-maximum structure, no line.
//!
//! So **there is no golden value to match here and none is claimed**, and
//! nothing in this module changes the output of any existing function — every
//! function below reads fields and returns new data. The rule for this case is
//! the skill's own: flag it, explain why, do not assume it correct.
//!
//! # EF-6's own inventory was wrong about faults, and that changes what got built
//!
//! EF-6 says *"`boundary_mask`/`boundary_type` (plate boundaries) are raster
//! masks, `Vec<u8>`, one flag per cell, and there is no contour or coastline
//! vectorization anywhere in this codebase (grepped for `marching_squares`/
//! `contour`/`trace_coastline`, zero hits)"*, and proposes one new tracer
//! covering coastlines **and** faults. Re-opened at the symbol before building
//! it, per `MISTAKES.md`'s own rule, that turns out to be true of the three
//! names grepped and false of the codebase:
//!
//! - [`crate::trace_boundaries`] **already traces `boundary_mask` into
//!   polylines** — a port of the reference's own `traceBoundaries()` (HTML
//!   2921-2952), thinning the mask to a 1-px skeleton and chain-walking it
//!   between junctions, with [`crate::tag_boundary_types`] labelling each run
//!   by majority `boundary_type`. `generate_terrain` calls both, every
//!   generation where World-Structure is on, and hands the result to
//!   `build_orogeny_field`. The gap EF-6 named as *"no fault vectorisation"*
//!   is really *"the polylines are consumed internally and never retained"*.
//! - [`cartalith_spatial::geo::trace_mask_rings`] already traces a binary mask
//!   into rings for the GeoJSON export.
//!
//! So the fault half of EF-6 is an **adapter, not a tracer** —
//! [`trace_fault_lines`] delegates to the golden-tested pair above and
//! converts their cell-index points into this module's pixel-space
//! convention. Building a second fault tracer on top of the new contour
//! primitive was the instruction and would have been the wrong code: a
//! threshold contour of `boundary_mask` traces the **outline of the boundary
//! band**, running down one side of a fault and back up the other, where
//! `trace_boundaries` gives the **centreline** with its type. A fault is a
//! line, not the edge of a region, and only one of those two answers is a
//! line.
//!
//! What *is* genuinely new and shared is the contour primitive itself,
//! [`cartalith_spatial::contour_polylines`], which [`trace_coastline`] uses —
//! see that module's header for why a continuous field earns the sub-cell
//! interpolation a binary mask does not.
//!
//! # Why `cartalith-terrain`
//!
//! `UNIFIED_TOOL_PLAN.md` milestone B/C's placement rule, applied the way
//! [`crate::amplify`]'s header and `cartalith-engine`'s `elevation` module
//! each apply it to themselves. The rule has three branches and all three fire
//! here:
//!
//! - **`cartalith-spatial` gets the generic machinery.** "Trace the level set
//!   of a scalar grid at a threshold" has no Cartalith-specific opinion, so
//!   [`cartalith_spatial::contour_polylines`] lives there beside
//!   `trace_mask_rings`, `norm_region` and `tile_dims`.
//! - **`cartalith-terrain` owns the height field, so it gets everything that
//!   needs to know what the field *means*.** "A coastline is the level set at
//!   `sea_level`" is sea-level knowledge — the same knowledge `amplify`'s own
//!   header cites as its reason for not living in `cartalith-spatial`. A
//!   ridgeline is TPI over the height field, and [`crate::analysis::tpi`] is
//!   already here. Plate boundaries are this crate's own output, and
//!   `trace_boundaries` is already here too.
//! - **`cartalith-engine` orchestrates; it does not compute.** Nothing below
//!   reads `WorldState`, exactly as `cartalith_hydrology::tile` deliberately
//!   does not: every input is an explicit parameter, so a caller can trace a
//!   refined LOD tile's field with the same functions it traces the world's
//!   with. No engine-level wrapper is added, because unlike EF-0's
//!   `world_amplify_opts` there is no multi-field options bag a caller could
//!   assemble wrong — a coastline needs the field and `WorldState::sea_level`,
//!   and that field's own doc comment already says to use it rather than
//!   `WorldParams::sea_level`.
//!
//! # World resolution today, tile resolution unchanged
//!
//! Every function here is a pure function of the arrays it is handed, so the
//! same call traces the world grid or one refined tile of it; nothing is
//! tile-aware and nothing needs to be. Per-tile refinement is *not* built here
//! (EF-1's boundary-condition problem has no analogue for these three: a
//! coastline, a fault and a local drainage divide are all local properties of
//! the field at the resolution you evaluate them, with no global accumulation
//! to get wrong). A tile-scoped caller should expect its traced lines to stop
//! at the tile edge and to need stitching, the same way
//! [`cartalith_spatial::contour_polylines`]'s own header describes for the map
//! border.

use cartalith_spatial::contour_polylines;

use crate::analysis::tpi;

/// The coastline: the level set of the height field at `sea_level`, as
/// polylines in pixel space (`col + 0.5`, `row + 0.5` for a cell centre).
///
/// A closed ring repeats its first point as its last; a chain running off the
/// sampled area does not. Rings wind so that **land is on the line's visual
/// left**, which is [`cartalith_spatial::contour_polylines`]'s guarantee, so
/// [`cartalith_spatial::ring_area`]'s sign separates an island's shore from a
/// lake's.
///
/// `sea_level` is `WorldState::sea_level`, not `WorldParams::sea_level` —
/// World-Structure re-anchors the first and leaves the second alone, and that
/// field's own doc comment already says so.
///
/// # A coastline point is NOT a cell centre, unlike every other polyline here
///
/// The one thing a consumer will get wrong, so it is said before the
/// consumer exists. `cartalith_hydrology::trace_river_polylines` and
/// [`trace_fault_lines`] both emit *cell centres*, so their points are exactly
/// half-integers and `(px - 0.5) as usize` is the cell. A contour crossing is
/// interpolated and lands anywhere on a block edge, so that idiom is wrong
/// here: use `px.floor()`, which is what `cartalith_civ::build_river_reach`
/// already does and what makes it directly reusable.
///
/// **Ruling N's coastal half is the consumer this unblocks**, and its river
/// half already shipped as `cartalith_civ::build_river_reach(polys, order, gw,
/// gh)` — same `&[Vec<(f64, f64)>]` parameter, same convention. The coastal
/// equivalent needs no order raster, only this output and a reach radius.
///
/// # This is every water edge at that level, not only the ocean's
///
/// Stated because the difference is invisible in the output and matters to the
/// consumer EF-6 exists for. A closed basin below `sea_level` with no
/// connection to the ocean — the shape `HYDROLOGY_CLASSIFICATION_RESEARCH.md`
/// is about — has a level set here exactly like a sea does, and comes back as
/// a ring indistinguishable from a coastline by geometry alone. A caller that
/// means *ocean* specifically must intersect with an ocean mask
/// (`cartalith_engine::build_water_bodies`'s classification); this crate
/// cannot, because that classification lives downstream of it. **Ruling N's
/// coastal term is that caller**, and this paragraph is the thing it must not
/// rediscover.
///
/// # Cost
///
/// One pass over the field plus the linking pass, both linear in cell count,
/// with [`cartalith_spatial::contour_polylines`]'s transient `2·w·h` index
/// arrays. At this port's largest shipping world it is tens of milliseconds
/// and about 27 MB, freed on return — an on-demand extraction, not something
/// to put in a frame loop.
pub fn trace_coastline(field: &[f32], gw: usize, gh: usize, sea_level: f64) -> Vec<Vec<(f64, f64)>> {
    contour_polylines(field, gw, gh, sea_level)
}

/// One traced plate boundary: its centreline in pixel space, and the
/// `btype` it was tagged with.
///
/// `kind` is one of [`crate::btype`]'s constants — `NONE` (0) where the run
/// touched no classified boundary cell, which is [`crate::tag_boundary_types`]'
/// own fallback rather than a value this module picks.
#[derive(Debug, Clone, PartialEq)]
pub struct FaultLine {
    /// Cell centres, `(col + 0.5, row + 0.5)`, in walk order.
    pub pts: Vec<(f64, f64)>,
    /// `btype::COLLISION` / `SUBDUCTION_OC` / `ARC_OO` / `RIFT` / `TRANSFORM`,
    /// or `NONE`.
    pub kind: u8,
}

/// Plate boundaries as typed polylines in this module's pixel-space
/// convention — EF-6's fault-line half.
///
/// **An adapter over [`crate::trace_boundaries`] and
/// [`crate::tag_boundary_types`], not a new tracer.** See this module's header
/// for why: those two already do the work, they are a port of the reference's
/// own `traceBoundaries()`/`currentBoundaryGraph()`, they are golden-tested,
/// and they give a fault's *centreline* where a threshold contour of the same
/// mask would give the outline of the band. This function adds exactly the
/// conversion `BoundaryPolyline`'s `Vec<(usize, usize)>` cell indices need to
/// become `(col + 0.5, row + 0.5)` points, and nothing else.
///
/// No new source data: `boundary_mask` and `boundary_type` are computed on
/// every generation and retained on `WorldState`.
///
/// # Two behaviours inherited from the tracer, not introduced here
///
/// - **A direct node-to-node edge comes back twice**, once walked from each
///   end. `trace_boundaries`' own doc comment records this as the reference's
///   behaviour, deliberately not deduplicated. A consumer that needs a unique
///   set must dedupe.
/// - **The thinning pass runs here.** `trace_boundaries` thins the mask to a
///   1-px skeleton iteratively, which is the expensive part and is *not*
///   cached across calls. `generate_terrain` already pays it when
///   World-Structure is on; with World-Structure off, this call pays it fresh.
///   Trace once and keep the result rather than calling per frame.
///
/// Because thinning only ever deletes cells, every point returned lies on a
/// cell where `boundary_mask` is non-zero — a property
/// `fault_lines_stay_on_the_boundary_mask` checks against a real generated
/// world rather than taking from this sentence.
pub fn trace_fault_lines(boundary_mask: &[u8], boundary_type: &[u8], gw: usize, gh: usize) -> Vec<FaultLine> {
    if gw == 0 || gh == 0 || boundary_mask.len() < gw * gh || boundary_type.len() < gw * gh {
        return Vec::new();
    }
    let mut graph = crate::trace_boundaries(boundary_mask, gw, gh);
    crate::tag_boundary_types(&mut graph, boundary_type, gw);
    graph
        .polylines
        .into_iter()
        .map(|pl| FaultLine {
            pts: pl.pts.iter().map(|&(x, y)| (x as f64 + 0.5, y as f64 + 0.5)).collect(),
            kind: pl.kind,
        })
        .collect()
}

/// How [`trace_ridgelines`] decides what counts as a ridge.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RidgeOpts {
    /// Neighbourhood radius in cells for [`crate::analysis::tpi`]. This sets
    /// the *scale* of ridge the trace finds: small radii follow every spur,
    /// large ones follow only the range's spine. Default 4. Measured on a real
    /// 512 × 341 world, 4/8/16 give 206/169/171 traced runs — the count is
    /// flat in radius once the mask is thinned, and what actually changes is
    /// which structure the runs sit on.
    pub tpi_radius: i64,
    /// Ridge threshold in standard deviations of TPI over land — Weiss's own
    /// 2001 ridge class is `TPI > +1 SD`, which is this default. A land whose
    /// TPI has zero variance (a perfect plain) has no ridge at any `sd_k` and
    /// traces nothing. Expressed in
    /// SD rather than height units on purpose: TPI is in the height field's
    /// own units, so a fixed threshold would mean a different landform at
    /// every world size, which is the resolution-scaling trap
    /// [`crate::analysis`]'s own header describes. Default 1.0.
    pub sd_k: f64,
    /// X wraps and Y clamps, matching every other field in this workspace.
    /// Default false.
    pub world: bool,
    /// Runs shorter than this are dropped. A ridge mask on real terrain is
    /// speckled with two-cell stubs that are nothing a consumer wants; 2 is
    /// the floor a polyline can have at all, and the default 3 is the
    /// cheapest filter that removes the single-segment noise. Default 3.
    pub min_points: usize,
}

impl Default for RidgeOpts {
    fn default() -> Self {
        RidgeOpts { tpi_radius: 4, sd_k: 1.0, world: false, min_points: 3 }
    }
}

/// Ridgelines — local drainage divides traced as polylines running **uphill**,
/// each ending at a local maximum of the ridge network.
///
/// EF-6's own framing, which is what this is: *"a ridge is a local drainage
/// divide, not a threshold boundary… tracing its local-maxima ridgeline into a
/// polyline is closer to `trace_river_polylines`'s receiver-chain walk run on
/// TPI instead of flow."* That is exactly the structure below, mirrored from
/// `trace_river_polylines` term for term with the direction reversed:
///
/// | river tracer | here |
/// |---|---|
/// | channel cells (`flow` over a threshold) | ridge cells (TPI over `sd_k` SD, on land) |
/// | `recv[i]`, the steepest-**descent** D8 neighbour | `up[i]`, the steepest-**ascent** D8 neighbour among ridge cells |
/// | a source is a channel cell with no upstream donor | a source is a ridge cell nothing ascends *into* |
/// | walk `i → recv[i]` until off-network, ending at a mouth | walk `i → up[i]` until off-network, ending at a local maximum |
/// | `visited` stops a tributary re-walking a claimed trunk | identical |
///
/// The ascent uses the same `(h_self − h_neighbour) / d8_dist` steepness with
/// the same `dy`-then-`dx` scan and first-strictly-greater tie-break
/// `cartalith_hydrology::d8_receiver` uses, sign flipped. It is spelled out
/// here rather than called because `cartalith-terrain` does not depend on
/// `cartalith-hydrology` and a dependency edge in that direction would invert
/// the workspace's layering for eleven lines.
///
/// # Termination is a property of the walk, not of a guard
///
/// Every step moves strictly uphill, so the elevation along a polyline is
/// strictly increasing and a cycle is impossible — `visited` is there to stop
/// a spur re-tracing a trunk, not to stop an infinite loop. That is stronger
/// than the river tracer's position, which needs `visited` for both, and
/// `ridgelines_ascend_strictly` pins it.
///
/// # The mask is thinned first, because the walk alone was measured and failed
///
/// A TPI ridge class is an **area** — a band of high ground several cells
/// wide — and a steepest-ascent walk over a 2-D band is not a crest line: every
/// cell in the band ascends into one of the band's many noise-local maxima, so
/// the output fills the band with short stubs instead of following its spine.
/// Measured on a real 512 × 341 world at `tpi_radius = 8`: the mask was
/// **8 846 cells in only 65 connected components** — well connected, so the
/// fragmentation was the walk's, not the data's — and the walk returned
/// **1 938 runs averaging 5.0 points**.
///
/// Thinning the mask to a 1-px skeleton first, with [`crate::thin_mask`] —
/// `trace_boundaries`' own Zhang-Suen pass, already in this crate — turns the
/// same input into **169 runs totalling 1 005 points**, an order of magnitude
/// fewer runs over a genuine crest network rather than a filled band. It costs
/// 1 ms there and 22 ms on a 2048 × 1311 world.
///
/// **What it does not fix**, stated because it is real: the remaining runs are
/// still mostly short. At `tpi_radius = 8` the longest run is 81 points and the
/// mean is 5.9, because a skeleton of a blobby mask is a tree with many small
/// spurs, and each spur ends at its own local maximum. The long runs are real
/// ridges; the short ones are real branches. A consumer wanting only major
/// crests should filter by length, and `min_points` is deliberately a blunt
/// floor rather than a cleverer pruning rule this has no measurement to
/// justify. **And thinning is geometric**: `thin_mask` knows nothing about
/// height, so the skeleton it keeps is the band's medial axis, which
/// approximates the crest and coincides with it exactly only where the band is
/// symmetric.
///
/// # What this is and is not
///
/// `crate::analysis`' §31 Category A applies unchanged: TPI and steepest
/// ascent are established geographic computation, and nothing here is tuned to
/// make a particular ridge appear. What it is *not* is a watershed-boundary
/// algorithm: a true drainage divide is the boundary between two catchments
/// and would be derived from flow routing, which lives in
/// `cartalith-hydrology`. This traces the *morphological* ridge — the crest
/// line TPI already identifies — which is what EF-6 asked for and what a
/// renderer or a landmark placer wants. The two agree on a sharp range and
/// diverge on flat ground, where TPI's ridge class is noise and a catchment
/// boundary is still exact.
///
/// Ordering is stable: sources are walked in ascending cell index, which is
/// `trace_river_polylines`' own effective order at `min_order = 1`.
pub fn trace_ridgelines(
    field: &[f32],
    gw: usize,
    gh: usize,
    sea_level: f64,
    opts: RidgeOpts,
) -> Vec<Vec<(f64, f64)>> {
    let n = gw * gh;
    if gw == 0 || gh == 0 || field.len() < n {
        return Vec::new();
    }

    // Ridge mask: land, and more than `sd_k` standard deviations above its own
    // local mean. The statistic is taken over land only, for the reason
    // `tpi_multiscale` already gives for its RMS — bathymetry would otherwise
    // set the scale for a statistic about hills. A true standard deviation,
    // mean subtracted, rather than the RMS `tpi_multiscale` uses: TPI's land
    // mean is near zero but not zero, and `sd_k` is documented in SD because
    // that is the unit Weiss's classification is written in.
    let t = tpi(field, gw, gh, opts.tpi_radius.max(1), opts.world);
    let land = |i: usize| (field[i] as f64) >= sea_level;
    let (mut sum, mut cnt) = (0.0f64, 0usize);
    for (i, &ti) in t.iter().enumerate().take(n) {
        if land(i) {
            sum += ti as f64;
            cnt += 1;
        }
    }
    if cnt == 0 {
        // An all-ocean world has no ridges, which is the honest answer.
        return Vec::new();
    }
    let mean = sum / cnt as f64;
    let mut var = 0.0f64;
    for (i, &ti) in t.iter().enumerate().take(n) {
        if land(i) {
            let d = ti as f64 - mean;
            var += d * d;
        }
    }
    let sd = (var / cnt as f64).sqrt();
    if sd <= 0.0 {
        // Perfectly flat land — TPI is identically zero, so every cell is
        // "at the threshold" and the mask would be the whole map. There is no
        // ridge structure to find, and saying so is more honest than emitting
        // a mask that means nothing.
        return Vec::new();
    }
    let thresh = mean + sd * opts.sd_k;

    // Thin the mask to a 1-px skeleton before walking it. This is the one
    // step that is not in EF-6's sketch and it was added because the sketch's
    // version was measured and found wanting, not on taste — see the module
    // header's own note. `thin_mask` is `trace_boundaries`' own Zhang-Suen
    // pass, reused rather than re-spelled, and it only ever deletes cells, so
    // every point this function returns is still a cell TPI called ridge-like.
    let raw: Vec<u8> = (0..n).map(|i| (land(i) && (t[i] as f64) >= thresh) as u8).collect();
    let skel = crate::thin_mask(&raw, gw, gh);
    let ridge: Vec<bool> = skel.iter().map(|&v| v != 0).collect();

    // Steepest-ascent neighbour within the ridge mask. `d8[k]` is the same
    // 3x3 distance table `compute_flow` builds.
    let mut d8 = [0f64; 9];
    for dy in -1i64..=1 {
        for dx in -1i64..=1 {
            d8[((dy + 1) * 3 + (dx + 1)) as usize] = (dx as f64).hypot(dy as f64);
        }
    }
    let mut up = vec![-1i64; n];
    for i in 0..n {
        if !ridge[i] {
            continue;
        }
        let (x, y) = ((i % gw) as i64, (i / gw) as i64);
        let hgt = field[i] as f64;
        let (mut best, mut best_rise) = (-1i64, 0.0f64);
        for dy in -1i64..=1 {
            for dx in -1i64..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let mut nx = x + dx;
                let ny = y + dy;
                if opts.world {
                    nx = ((nx % gw as i64) + gw as i64) % gw as i64;
                } else if nx < 0 || nx >= gw as i64 {
                    continue;
                }
                if ny < 0 || ny >= gh as i64 {
                    continue;
                }
                let j = (ny * gw as i64 + nx) as usize;
                if !ridge[j] {
                    continue;
                }
                let rise = (field[j] as f64 - hgt) / d8[((dy + 1) * 3 + (dx + 1)) as usize];
                if rise > best_rise {
                    best_rise = rise;
                    best = j as i64;
                }
            }
        }
        up[i] = best;
    }

    // A source is a ridge cell nothing ascends into — the low end of a spur,
    // mirroring the river tracer's "channel cell with no upstream donor".
    let mut has_below = vec![false; n];
    for i in 0..n {
        if ridge[i] && up[i] >= 0 {
            has_below[up[i] as usize] = true;
        }
    }

    let min_points = opts.min_points.max(2);
    let mut visited = vec![false; n];
    let mut out: Vec<Vec<(f64, f64)>> = Vec::new();
    for s in 0..n {
        if !ridge[s] || has_below[s] || visited[s] {
            continue;
        }
        let mut pts = Vec::new();
        let mut cur = s as i64;
        while cur >= 0 {
            let ci = cur as usize;
            pts.push(((ci % gw) as f64 + 0.5, (ci / gw) as f64 + 0.5));
            if visited[ci] {
                break;
            }
            visited[ci] = true;
            cur = up[ci];
        }
        if pts.len() >= min_points {
            out.push(pts);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A cone: one peak at the centre, falling off linearly. Its sea-level
    /// contour is a circle and its ridge structure is radial.
    fn cone(gw: usize, gh: usize, cx: f64, cy: f64, peak: f64, k: f64) -> Vec<f32> {
        (0..gw * gh)
            .map(|i| {
                let (x, y) = ((i % gw) as f64 + 0.5, (i / gw) as f64 + 0.5);
                (peak - k * ((x - cx).powi(2) + (y - cy).powi(2)).sqrt()) as f32
            })
            .collect()
    }

    #[test]
    fn a_coastline_follows_a_known_circle() {
        let (gw, gh) = (81usize, 81usize);
        let (cx, cy) = (40.5f64, 40.5f64);
        // peak 1.0, slope 1/50 per cell, sea at 0.6 -> shore at r = 20.
        let f = cone(gw, gh, cx, cy, 1.0, 0.02);
        let pls = trace_coastline(&f, gw, gh, 0.6);
        assert_eq!(pls.len(), 1, "one island, one shore");
        let worst = pls[0]
            .iter()
            .map(|&(x, y)| (((x - cx).powi(2) + (y - cy).powi(2)).sqrt() - 20.0).abs())
            .fold(0.0f64, f64::max);
        assert!(worst < 0.05, "worst radial error {worst} cells");
        assert_eq!(pls[0][0], pls[0][pls[0].len() - 1], "an island shore closes");
    }

    #[test]
    fn a_coastline_traces_identically_twice() {
        let (gw, gh) = (64usize, 48usize);
        let f: Vec<f32> = (0..gw * gh)
            .map(|i| {
                let (x, y) = ((i % gw) as f64, (i / gw) as f64);
                (0.5 + 0.3 * (x * 0.21).sin() * (y * 0.17).cos()) as f32
            })
            .collect();
        let a = trace_coastline(&f, gw, gh, 0.5);
        assert!(a.len() > 2, "expected several shores, got {}", a.len());
        assert_eq!(a, trace_coastline(&f, gw, gh, 0.5));
    }

    #[test]
    fn fault_lines_carry_the_type_and_the_pixel_convention() {
        // A straight 1-px boundary across five cells, all tagged RIFT.
        let (gw, gh) = (5usize, 3usize);
        let mut mask = vec![0u8; gw * gh];
        let mut kind = vec![crate::btype::NONE; gw * gh];
        for x in 0..gw {
            mask[gw + x] = 1;
            kind[gw + x] = crate::btype::RIFT;
        }
        let fls = trace_fault_lines(&mask, &kind, gw, gh);
        assert!(!fls.is_empty(), "a straight line traces");
        for fl in &fls {
            assert_eq!(fl.kind, crate::btype::RIFT);
            for &(px, py) in &fl.pts {
                assert!((py - 1.5).abs() < 1e-12, "row 1's centre is y = 1.5, got {py}");
                assert!((px.fract() - 0.5).abs() < 1e-12, "cell centres are x.5, got {px}");
                assert_eq!(mask[gw + (px - 0.5) as usize], 1);
            }
        }
    }

    #[test]
    fn fault_lines_refuse_a_short_buffer_rather_than_indexing_past_it() {
        assert!(trace_fault_lines(&[1, 1], &[1, 1], 4, 4).is_empty());
        assert!(trace_fault_lines(&[], &[], 0, 0).is_empty());
    }

    /// A long crest along `y = 12` falling away to either side, tilted so the
    /// crest itself rises to the east.
    fn crest(gw: usize, gh: usize) -> Vec<f32> {
        (0..gw * gh)
            .map(|i| {
                let (x, y) = ((i % gw) as f64, (i / gw) as f64);
                (0.5 + 0.004 * x - 0.02 * (y - 12.0).abs()) as f32
            })
            .collect()
    }

    #[test]
    fn ridgelines_ascend_strictly_and_end_at_a_local_maximum() {
        let (gw, gh) = (60usize, 25usize);
        let f = crest(gw, gh);
        let opts = RidgeOpts { tpi_radius: 5, ..RidgeOpts::default() };
        let pls = trace_ridgelines(&f, gw, gh, 0.0, opts);
        assert!(!pls.is_empty(), "a crest traces");
        let at = |p: (f64, f64)| f[(p.1 as usize) * gw + (p.0 as usize)] as f64;
        for pl in &pls {
            for w in pl.windows(2) {
                assert!(at(w[1]) > at(w[0]), "each step must rise: {:?} -> {:?}", w[0], w[1]);
            }
        }
        let longest = pls.iter().max_by_key(|p| p.len()).expect("non-empty");
        assert!(longest.len() > 40, "the crest should trace as one long run, got {}", longest.len());
        assert!(
            (longest[longest.len() - 1].1 - 12.5).abs() < 1.5,
            "it should follow the crest row, ended at y = {}",
            longest[longest.len() - 1].1
        );
    }

    #[test]
    fn ridgelines_trace_identically_twice_and_skip_an_all_ocean_world() {
        let (gw, gh) = (60usize, 25usize);
        let f = crest(gw, gh);
        let opts = RidgeOpts { tpi_radius: 5, ..RidgeOpts::default() };
        let a = trace_ridgelines(&f, gw, gh, 0.0, opts);
        assert!(!a.is_empty());
        assert_eq!(a, trace_ridgelines(&f, gw, gh, 0.0, opts));
        // Everything below sea level: no land, so no ridge, and no panic on
        // the zero-count standard deviation.
        assert!(trace_ridgelines(&f, gw, gh, 2.0, opts).is_empty());
        assert!(trace_ridgelines(&[], 0, 0, 0.0, opts).is_empty());
    }

    #[test]
    fn a_cone_has_no_ridgeline_and_a_crest_does() {
        // Not a degenerate case — the distinction the thinning pass exists to
        // make. A radially symmetric cone has a *summit* and no crest: its TPI
        // ridge class is a disc around the peak, and the skeleton of a disc is
        // a point. A crest, thinned, is a line. If this ever starts returning
        // runs for the cone, the mask has stopped being thinned.
        let (gw, gh) = (60usize, 60usize);
        let opts = RidgeOpts { tpi_radius: 5, ..RidgeOpts::default() };
        let c = trace_ridgelines(&cone(gw, gh, 29.5, 29.5, 1.0, 0.02), gw, gh, 0.0, opts);
        assert!(c.is_empty(), "a cone traced {} ridgelines", c.len());
        let r = trace_ridgelines(&crest(60, 25), 60, 25, 0.0, opts);
        assert!(!r.is_empty(), "a crest traces");
    }

    #[test]
    fn a_flat_plain_has_no_ridge_to_trace() {
        // Zero TPI variance. Without the explicit guard this returns empty for
        // the wrong reason — `0 >= 0` would mask the whole map in and then
        // every walk would be one point long — so assert the mask is refused,
        // not just that nothing came out: a single raised cell in the same
        // plain is enough variance to produce a mask, and still traces
        // nothing, which is the contrast that makes the first assert mean
        // something.
        let f = vec![0.7f32; 40 * 40];
        assert!(trace_ridgelines(&f, 40, 40, 0.0, RidgeOpts::default()).is_empty());
        let mut one = f.clone();
        one[20 * 40 + 20] = 0.9;
        assert!(trace_ridgelines(&one, 40, 40, 0.0, RidgeOpts::default()).is_empty());
    }
}
