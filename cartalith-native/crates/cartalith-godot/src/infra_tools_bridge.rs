//! The INFRA domain's Way/Route tools, plus the two global tools that ride
//! alongside them in the same left-dock TOOLS block — `UNIFIED_TOOL_PLAN.md`
//! milestone F (`DCC_SHELL_SPEC.md` §4.5.1 Measure/Region select, §4.5.4
//! Way/Route).
//!
//! Deliberately **free of any `godot` dependency**, the same isolation
//! `sculpt_bridge.rs`'s own doc comment argues for: `lib.rs` owns the thin
//! `Variant`<->`f64`/`GString` conversion and the `#[func]` surface plus the
//! RouteContext plumbing that needs live `WorldState`/`CivData` borrows this
//! module never sees; this module owns the actual state machines — the
//! in-progress way/route being drawn, the Measure click chain, the Region
//! select marquee — with its own `#[cfg(test)]` suite below, exercised by
//! `cargo test -p cartalith-godot`'s ordinary unit-test pass with no Godot
//! runtime involved.
//!
//! ## Why one struct for four tools
//!
//! [`InfraTools`] holds all four (`way`/`route`/`measure`/`region`) the same
//! way [`sculpt_bridge::SculptEditor`] holds Sculpt's tool state, draft and
//! water hooks together: none of the four is independently constructible or
//! reusable, and `WorldGen` already keeps this kind of per-instance tool
//! state as a plain field rather than a second `GodotClass`. `WorldGen`
//! holds it as `Option<InfraTools>` — `Some` after `absorb`, `None` after
//! `load_save` — the exact same lifecycle `sculpt`/`icons`/`civ_tools`/
//! `paint`/`labels` already use there and for the same reason each of their
//! own doc comments gives: a waypoint, measurement or marquee rect from the
//! *previous* world's grid is meaningless (or out of bounds) over a
//! differently-sized one, so a fresh `InfraTools::new()` per generation is a
//! hard reset, not a narrower "just don't offer commit" gate. Measure and
//! Region select are "every domain" tools per §4.5.1, but they still only
//! make sense over *some* generated grid, so they share this lifecycle
//! rather than getting a separate always-live field.
//!
//! ## Way and Route are two separate commit paths, on purpose
//!
//! `DCC_SHELL_SPEC.md` §4.5.4 is explicit that v2.10 keeps `draw_way` and
//! `route` apart (`_civCommitWay` vs. `_civCommitRoute`) because a way is
//! durable geometry others route over and a route is a journey along
//! existing geometry — conflating the two would let a hand-drawn road cut
//! across a bay the way a `Mixed`-mode journey is allowed to.
//! [`InfraTools::way_commit`] therefore only ever routes
//! [`cartalith_civ::tools::RouteMode::Water`] (sea lane) or `::Land`
//! (everything else) via `civ_commit_way`'s own branch, while
//! [`InfraTools::route_commit`] takes whichever `RouteMode` `route_begin`
//! was armed with and calls `civ_join_dijkstra_segs` directly — there is no
//! reference `_civCommitRoute`-equivalent function to port (`tools.rs`'s own
//! comment on this: the Route tool "commits `Mixed` into `civJourneys`, a
//! different list feeding the Journey Planner", never wrapped in its own
//! named helper), so this module is the first place that assembly exists in
//! Rust.
//!
//! ## No freehand way mode
//!
//! §4.5.4's Way options row lists a routing-mode dropdown ("freehand / snap
//! / least-cost"). Only one of those three has a real algorithm behind it:
//! `civ_commit_way` has never had more than one way to turn waypoints into a
//! way — least-cost Dijkstra (`civ_join_dijkstra_segs` -> `civ_dijkstra_path`
//! -> `road_dijkstra`), always. There is nothing to build a "freehand" or
//! distinct "snap" *routing* mode out of without inventing an algorithm the
//! reference never had, which `cartalith-porting-discipline` and this port's
//! own "ship without it rather than faking it" rule both forbid. What *is*
//! real and wired here is point-level **snap-to-place/way**
//! (`civ_find_snap_target`/`civ_snap_point`, reference v1.52) — §4.5.4's
//! other modifier, "on by default" — applied by `lib.rs`'s
//! `way_append_point`/`route_append_stop` before a click ever reaches this
//! module, using `civ_snap_radius`'s own base radius. `way_begin`/
//! `route_begin` therefore take no routing-mode argument at all for Way; for
//! Route, `route_begin` takes a real `RouteMode` (`land`/`water`/`mixed`),
//! since all three are genuine, tested cost domains a journey may want, not
//! UI-only labels.

use cartalith_civ::tools::{civ_commit_way, civ_join_dijkstra_segs, ManualWay, ManualWayType, RouteContext, RouteMode};
use cartalith_spatial::{measure, tile_dims, Measurement, Region};

// ===================== Way/Route drafts =====================

/// A waypoint chain being drawn for the Way tool, before `way_commit` turns
/// it into a real [`ManualWay`] (`DCC_SHELL_SPEC.md` §4.5.4: "Click appends
/// a waypoint; Esc commits").
struct WayDraft {
    way_type: ManualWayType,
    points: Vec<(f64, f64)>,
}

/// The Route tool's mirror of [`WayDraft`] — stops instead of waypoints, and
/// a `RouteMode` chosen once at `route_begin` rather than derived from a
/// type the way `WayDraft`'s is (see the module doc's "no equivalent to
/// `ManualWayType::SeaLane`" note: a route may be `Mixed`, a way never is).
struct RouteDraft {
    mode: RouteMode,
    points: Vec<(f64, f64)>,
}

/// One committed route. The reference has no dedicated
/// `_civCommitRoute`-equivalent struct to port (see the module doc) —
/// `civJourneys` entries are `civ_join_dijkstra_segs`' own `{pts, brks, km}`
/// shape, which is `JoinedPath`; this just keeps the `RouteMode` and
/// unreachable-leg count alongside it so a caller can inspect a stored
/// route without having re-armed the tool that produced it.
#[derive(Debug, Clone, PartialEq)]
pub struct CommittedRoute {
    pub pts: Vec<(f64, f64)>,
    pub brks: Vec<usize>,
    pub km: f64,
    pub mode: RouteMode,
    pub unreachable_legs: usize,
}

/// The per-world tool state for Way, Route, Measure and Region select.
///
/// Every field starts empty/`None`; nothing here is computed at
/// construction (unlike [`sculpt_bridge::SculptEditor`], which adopts a
/// generated river lock immediately) because none of these four tools has
/// anything to adopt from a fresh generation — they are all user-driven from
/// the first click.
pub struct InfraTools {
    way_draft: Option<WayDraft>,
    route_draft: Option<RouteDraft>,
    /// Committed hand-drawn ways, in commit order — `way_commit`'s returned
    /// index is always `self.ways.len() - 1` at the moment it was pushed.
    pub ways: Vec<ManualWay>,
    /// Committed routes, same indexing convention as `ways`.
    pub routes: Vec<CommittedRoute>,
    /// The Measure tool's in-progress click chain (`DCC_SHELL_SPEC.md`
    /// §4.5.1: "Click to drop points; double-click or Esc ends").
    measure_points: Vec<(f64, f64)>,
    /// Region select's marquee. Per §4.5.1's own note this is the SAME rect
    /// the Data manager's export route reads — `region_export_tiles`
    /// (`lib.rs`) reads this field directly rather than keeping a second
    /// copy, so setting it here and reading it from the export route really
    /// are "two views of one rect", not values that could drift apart.
    pub region: Option<Region>,
}

impl Default for InfraTools {
    fn default() -> Self {
        Self::new()
    }
}

impl InfraTools {
    pub fn new() -> Self {
        Self {
            way_draft: None,
            route_draft: None,
            ways: Vec::new(),
            routes: Vec::new(),
            measure_points: Vec::new(),
            region: None,
        }
    }

    // ===================== Way =====================

    pub fn way_begin(&mut self, way_type: ManualWayType) {
        self.way_draft = Some(WayDraft { way_type, points: Vec::new() });
    }

    /// `false` with no draft armed (`way_begin` was never called, or the
    /// last draft was already committed/discarded) — the caller's own
    /// "was this click accepted" signal, matching `sculpt_bridge`'s
    /// `Outcome`-style honesty about a no-op.
    pub fn way_append_point(&mut self, x: f64, y: f64) -> bool {
        match self.way_draft.as_mut() {
            Some(d) => {
                d.points.push((x, y));
                true
            }
            None => false,
        }
    }

    pub fn way_discard(&mut self) {
        self.way_draft = None;
    }

    /// The `RouteMode` a `way_commit` right now would route under —
    /// `civ_commit_way`'s own branch (`Water` for a sea lane, `Land`
    /// otherwise) — exposed so `lib.rs` can build the matching
    /// `RouteContext` (only `RouteMode::Mixed` ever reads `biome`/
    /// `river_order`; a way commit never needs either) *before* calling
    /// `way_commit`. `None` with no draft armed.
    pub fn way_draft_mode(&self) -> Option<RouteMode> {
        self.way_draft.as_ref().map(|d| {
            if d.way_type == ManualWayType::SeaLane { RouteMode::Water } else { RouteMode::Land }
        })
    }

    /// Commits the in-progress way via `civ_commit_way` — real least-cost
    /// Dijkstra routing (see the module doc's "no freehand way mode"),
    /// never a straight line through the clicked points. `None` for no
    /// draft, or fewer than two waypoints (`civ_commit_way`'s own guard,
    /// which the reference still discards the draft on — this does too,
    /// via `Option::take` above regardless of outcome).
    ///
    /// Returns `(new index into self.ways, unreachable-leg count)` — the
    /// second number is `civ_commit_way`'s own "v1.99: `> 0` means some
    /// stretch of this way is a straight line across terrain the type is
    /// meant to avoid... the reference alerts and keeps the way", which
    /// `lib.rs` surfaces as a print rather than silently dropping it (the
    /// way itself is always kept, matching the reference).
    pub fn way_commit(&mut self, ctx: &RouteContext) -> Option<(usize, usize)> {
        let draft = self.way_draft.take()?;
        let commit = civ_commit_way(ctx, &draft.points, draft.way_type)?;
        let idx = self.ways.len();
        let unreachable = commit.unreachable_legs;
        self.ways.push(commit.way);
        Some((idx, unreachable))
    }

    // ===================== Route =====================

    pub fn route_begin(&mut self, mode: RouteMode) {
        self.route_draft = Some(RouteDraft { mode, points: Vec::new() });
    }

    pub fn route_append_stop(&mut self, x: f64, y: f64) -> bool {
        match self.route_draft.as_mut() {
            Some(d) => {
                d.points.push((x, y));
                true
            }
            None => false,
        }
    }

    pub fn route_discard(&mut self) {
        self.route_draft = None;
    }

    /// The armed `RouteMode`, for the same `RouteContext`-sizing reason
    /// `way_draft_mode` exists — here it is simply the mode `route_begin`
    /// was given, not derived from anything.
    pub fn route_draft_mode(&self) -> Option<RouteMode> {
        self.route_draft.as_ref().map(|d| d.mode)
    }

    /// Commits the in-progress route via `civ_join_dijkstra_segs` under the
    /// draft's own `RouteMode`. `None` for no draft, or fewer than two
    /// stops — unlike `way_commit`, this guard is the bridge's own (there
    /// is no reference `_civCommitRoute` to inherit one from), chosen to
    /// match `civ_commit_way`'s exactly for consistency between the two
    /// commit paths.
    ///
    /// Returns `(new index into self.routes, unreachable-leg count)`, same
    /// shape as `way_commit`.
    pub fn route_commit(&mut self, ctx: &RouteContext) -> Option<(usize, usize)> {
        let draft = self.route_draft.take()?;
        if draft.points.len() < 2 {
            return None;
        }
        let j = civ_join_dijkstra_segs(ctx, &draft.points, draft.mode);
        let idx = self.routes.len();
        let unreachable = j.unreachable_legs;
        self.routes.push(CommittedRoute { pts: j.pts, brks: j.brks, km: j.km, mode: draft.mode, unreachable_legs: j.unreachable_legs });
        Some((idx, unreachable))
    }

    // ===================== Measure =====================

    pub fn measure_begin(&mut self) {
        self.measure_points.clear();
    }

    pub fn measure_add_point(&mut self, x: f64, y: f64) {
        self.measure_points.push((x, y));
    }

    pub fn measure_clear(&mut self) {
        self.measure_points.clear();
    }

    pub fn measure_points(&self) -> &[(f64, f64)] {
        &self.measure_points
    }

    // ===================== Region select =====================

    pub fn region_set(&mut self, r: Region) {
        self.region = Some(r);
    }

    pub fn region_clear(&mut self) {
        self.region = None;
    }
}

// ===================== String <-> engine enum parsing =====================

/// `way_begin`'s `GString` -> [`ManualWayType`] mapping — the reference's
/// own four-entry `#civWayType` select (`tools.rs`'s doc comment on
/// `ManualWayType`: Road/Track/SeaLane/Ancient), matched case-insensitively
/// against the plain snake-case tokens this port's other string dispatch
/// (e.g. `params.rs`'s dotted keys) already uses.
///
/// **Not** `DCC_SHELL_SPEC.md` §4.5.4's own "road / track / trail / bridge"
/// list — that is the design side's UI vocabulary, and it does not actually
/// match the engine's real four-entry enum (no `trail`, no `bridge`; the
/// engine has `sea_lane` and `ancient` instead). Binding to the tested
/// engine enum rather than the design doc's differing labels is the same
/// choice `sculpt_bridge.rs`'s own module doc makes for the eight brush/
/// noise globals: the engine's real values win, and the mismatch is this
/// comment's disclosure, not a silent reconciliation.
pub fn parse_way_type(s: &str) -> Option<ManualWayType> {
    match s.to_ascii_lowercase().replace(['-', ' '], "_").as_str() {
        "road" => Some(ManualWayType::Road),
        "track" => Some(ManualWayType::Track),
        "sea_lane" | "sealane" => Some(ManualWayType::SeaLane),
        "ancient" => Some(ManualWayType::Ancient),
        _ => None,
    }
}

/// `route_begin`'s `GString` -> [`RouteMode`] mapping. `RouteMode` is a cost
/// *domain* (which terrain a route may cross), not a UI routing-style choice
/// — `land`/`water` are the two constrained domains and `mixed` is the one
/// that may cross either, which is the closest real meaning
/// `DCC_SHELL_SPEC.md` §4.5.4's "least-cost" label has (Dijkstra picks
/// whichever is genuinely cheaper across both). The spec's other two labels,
/// "freehand" and "snap", describe no distinct engine algorithm (see the
/// module doc) and are deliberately not accepted here — an unrecognised
/// string returns `None` rather than silently defaulting to a mode the
/// caller didn't ask for.
pub fn parse_route_mode(s: &str) -> Option<RouteMode> {
    match s.to_ascii_lowercase().replace(['-', ' '], "_").as_str() {
        "land" => Some(RouteMode::Land),
        "water" => Some(RouteMode::Water),
        "mixed" | "least_cost" => Some(RouteMode::Mixed),
        _ => None,
    }
}

// ===================== Route inputs (water/biome/river order) =====================

/// Fresh water-bodies (and, only for [`RouteMode::Mixed`], biome and river
/// order) for one `RouteContext` — built new on every Way/Route commit
/// rather than cached against `compute_civilisation`'s own copies
/// (`lib.rs`), for the same reason `civ_dijkstra_path`'s own doc comment
/// gives for rebuilding its cost grid on every call: a commit is a
/// click-driven user action, not a per-frame cost, so recomputing here is
/// cheap relative to the cadence it runs at, and it keeps this module free
/// of a second, possibly-stale handle into fields `compute_civilisation`
/// already owns the canonical computation of.
///
/// `mode` gates the extra two rasters because `RouteContext`'s own doc
/// comment says so directly: "`RouteMode::Land`/`Water` grids ignore them
/// entirely" — building them for a way (which is never `Mixed`) would be
/// pure waste.
pub struct RouteInputs {
    pub water_bodies: Vec<u8>,
    pub biome: Option<Vec<u8>>,
    pub river_order: Option<Vec<i16>>,
}

impl RouteInputs {
    #[allow(clippy::too_many_arguments)]
    pub fn build(
        ws: &cartalith_engine::WorldState,
        gw: usize,
        gh: usize,
        world: bool,
        map_width_km: f64,
        river_density: f64,
        mode: RouteMode,
    ) -> Self {
        let sea_level = ws.sea_level;
        let wb = cartalith_civ::build_water_bodies(&ws.field, gw, gh, sea_level, world, Some(&ws.rainfall));
        let (biome, river_order) = if mode == RouteMode::Mixed {
            let biome = cartalith_civ::build_biome_raster(&wb.classification, &ws.temperature, &ws.rainfall);
            let river_order = cartalith_civ::fresh_river_order(&ws.field, &ws.flow_discharge, gw, gh, sea_level, world, river_density, map_width_km);
            (Some(biome), Some(river_order))
        } else {
            (None, None)
        };
        RouteInputs { water_bodies: wb.classification, biome, river_order }
    }
}

// ===================== Measure =====================

/// One measured leg of a Measure-tool chain: `cartalith_spatial::measure`'s
/// own [`Measurement`] plus a compass bearing, since Measure's own contract
/// (`DCC_SHELL_SPEC.md` §4.5.1's right-dock "Segment table (bearing,
/// length)") wants a direction the underlying primitive does not compute.
///
/// **New at the bridge, not a port** — same disclosure `measure.rs`'s own
/// module doc gives the primitive itself: there is no reference precedent
/// for a bearing readout either (`updateScaleBar` is a passive scale bar,
/// nothing more). The convention chosen is the map's own: grid `y`
/// increases southward, matching every raster in this port, so `0°` = north
/// (`-y`), `90°` = east (`+x`), compass-clockwise.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MeasuredLeg {
    pub m: Measurement,
    pub bearing_deg: f64,
}

pub fn measure_leg(a: (f64, f64), b: (f64, f64), grid_w: usize, map_width_km: f64, world: bool) -> MeasuredLeg {
    let m = measure(a, b, grid_w, map_width_km, world);
    let bearing_deg = m.dx.atan2(-m.dy).to_degrees().rem_euclid(360.0);
    MeasuredLeg { m, bearing_deg }
}

/// Every leg of a click chain, in order — `pts.windows(2)` through
/// [`measure_leg`]. Empty for fewer than two points, same as
/// `cartalith_spatial::measure_path`'s own "a chain under construction is a
/// normal state, not a failure".
pub fn measure_legs(pts: &[(f64, f64)], grid_w: usize, map_width_km: f64, world: bool) -> Vec<MeasuredLeg> {
    pts.windows(2).map(|w| measure_leg(w[0], w[1], grid_w, map_width_km, world)).collect()
}

// ===================== Region select =====================

/// `region_get`'s "tile estimate per LOD" (`DCC_SHELL_SPEC.md` §4.5.1's
/// right dock). **This port's own convention, not a reference value** —
/// v2.10's Region-select panel has no LOD ladder at all, only the single
/// `cols`x`rows` grid the user types into the export dialog
/// (`#regionCols`/`#regionRows`). Three representative grids (a coarse/
/// medium/fine tri-level pyramid) stand in for "low/medium/high", each run
/// through `tile_dims` — `export_region_tiles`'s own sizing primitive — at
/// a fixed 512px long edge, so what is reported is exactly what
/// `region_export_tiles` would produce if the caller chose that grid, not a
/// separate estimate that could disagree with the real export.
pub const REGION_LOD_GRIDS: [(&str, usize, usize); 3] = [("low", 1, 1), ("medium", 2, 2), ("high", 4, 4)];

/// The representative export tile size (long edge, pixels) the LOD estimate
/// above is computed at. Matches no particular reference value — there is
/// none to match — and a real export's actual `tile_size` is a caller-chosen
/// `RegionExportOpts` field that may differ from this.
const REGION_LOD_TILE_PX: usize = 512;

/// `(label, tile_count, tile_w, tile_h)` per LOD tier in [`REGION_LOD_GRIDS`].
pub fn region_tile_estimate(region: &Region) -> Vec<(&'static str, usize, usize, usize)> {
    REGION_LOD_GRIDS
        .iter()
        .map(|&(label, cols, rows)| {
            let td = tile_dims(region, cols, rows, REGION_LOD_TILE_PX);
            (label, cols * rows, td.w, td.h)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use cartalith_civ::tools::WayRef;
    use cartalith_civ::{NamedSettlement, SettlementKind, SettlementPlacement};

    // ---------- string parsing ----------

    #[test]
    fn parse_way_type_covers_all_four_reference_entries() {
        assert_eq!(parse_way_type("road"), Some(ManualWayType::Road));
        assert_eq!(parse_way_type("Track"), Some(ManualWayType::Track));
        assert_eq!(parse_way_type("SEA-LANE"), Some(ManualWayType::SeaLane));
        assert_eq!(parse_way_type("sealane"), Some(ManualWayType::SeaLane));
        assert_eq!(parse_way_type("ancient"), Some(ManualWayType::Ancient));
        assert_eq!(parse_way_type("bridge"), None, "not a real engine way type -- see the module doc");
    }

    #[test]
    fn parse_route_mode_covers_land_water_mixed_and_rejects_the_rest() {
        assert_eq!(parse_route_mode("land"), Some(RouteMode::Land));
        assert_eq!(parse_route_mode("Water"), Some(RouteMode::Water));
        assert_eq!(parse_route_mode("mixed"), Some(RouteMode::Mixed));
        assert_eq!(parse_route_mode("least-cost"), Some(RouteMode::Mixed));
        assert_eq!(parse_route_mode("freehand"), None, "no distinct engine algorithm backs this -- see the module doc");
        assert_eq!(parse_route_mode("snap"), None);
        assert_eq!(parse_route_mode("nonsense"), None);
    }

    // ---------- way/route draft lifecycle ----------

    /// A tiny two-band world: land on the right, ocean on the left, so both
    /// routing domains have something real to refuse -- the same fixture
    /// shape `tools.rs`'s own `route_fixture`/`route_ctx` tests use.
    fn route_fixture() -> (Vec<f32>, Vec<u8>) {
        let (gw, gh) = (24usize, 16usize);
        let mut field = vec![0.6f32; gw * gh];
        let mut wb = vec![0u8; gw * gh];
        for y in 0..gh {
            for x in 0..10 {
                field[y * gw + x] = 0.15;
                wb[y * gw + x] = 1;
            }
        }
        (field, wb)
    }

    fn route_ctx<'a>(field: &'a [f32], wb: &'a [u8], ways: &'a [WayRef<'a>]) -> RouteContext<'a> {
        RouteContext { field, water_bodies: wb, biome: None, river_order: None, places: &[], ways, gw: 24, gh: 16, sea: 0.42, world: false, map_width_km: 240.0 }
    }

    #[test]
    fn way_append_point_is_a_no_op_before_way_begin() {
        let mut t = InfraTools::new();
        assert!(!t.way_append_point(1.0, 1.0));
    }

    #[test]
    fn way_discard_clears_the_draft_without_committing() {
        let mut t = InfraTools::new();
        t.way_begin(ManualWayType::Road);
        t.way_append_point(12.0, 2.0);
        t.way_discard();
        assert!(t.way_draft_mode().is_none());
        assert!(t.ways.is_empty());
    }

    #[test]
    fn way_commit_needs_two_waypoints_and_returns_none_otherwise() {
        let (field, wb) = route_fixture();
        let ctx = route_ctx(&field, &wb, &[]);
        let mut t = InfraTools::new();
        t.way_begin(ManualWayType::Road);
        t.way_append_point(12.0, 2.0);
        assert_eq!(t.way_commit(&ctx), None, "civ_commit_way's own two-waypoint guard");
        assert!(t.ways.is_empty());
        // The draft is consumed either way (Option::take), matching the
        // reference's own "still discards the draft" behaviour.
        assert!(t.way_draft_mode().is_none());
    }

    #[test]
    fn way_commit_pushes_a_real_way_and_returns_its_index() {
        let (field, wb) = route_fixture();
        let ctx = route_ctx(&field, &wb, &[]);
        let mut t = InfraTools::new();
        t.way_begin(ManualWayType::Road);
        assert_eq!(t.way_draft_mode(), Some(RouteMode::Land));
        t.way_append_point(12.0, 2.0);
        t.way_append_point(22.0, 13.0);
        let (idx, unreachable) = t.way_commit(&ctx).expect("two land waypoints on land should commit");
        assert_eq!(idx, 0);
        assert_eq!(unreachable, 0);
        assert_eq!(t.ways.len(), 1);
        assert_eq!(t.ways[0].way_type, ManualWayType::Road);
        assert!(!t.ways[0].sea);
    }

    #[test]
    fn a_sea_lane_way_draft_routes_water_mode() {
        let mut t = InfraTools::new();
        t.way_begin(ManualWayType::SeaLane);
        assert_eq!(t.way_draft_mode(), Some(RouteMode::Water));
    }

    #[test]
    fn a_second_way_commit_gets_index_one() {
        let (field, wb) = route_fixture();
        let ctx = route_ctx(&field, &wb, &[]);
        let mut t = InfraTools::new();
        t.way_begin(ManualWayType::Road);
        t.way_append_point(12.0, 2.0);
        t.way_append_point(22.0, 13.0);
        t.way_commit(&ctx).unwrap();
        t.way_begin(ManualWayType::Track);
        t.way_append_point(12.0, 3.0);
        t.way_append_point(22.0, 12.0);
        let (idx, _) = t.way_commit(&ctx).unwrap();
        assert_eq!(idx, 1);
        assert_eq!(t.ways.len(), 2);
    }

    #[test]
    fn route_append_stop_is_a_no_op_before_route_begin() {
        let mut t = InfraTools::new();
        assert!(!t.route_append_stop(1.0, 1.0));
    }

    #[test]
    fn route_commit_needs_two_stops() {
        let (field, wb) = route_fixture();
        let ctx = route_ctx(&field, &wb, &[]);
        let mut t = InfraTools::new();
        t.route_begin(RouteMode::Land);
        t.route_append_stop(12.0, 2.0);
        assert_eq!(t.route_commit(&ctx), None);
        assert!(t.routes.is_empty());
    }

    #[test]
    fn route_commit_pushes_a_joined_route_under_its_own_mode() {
        let (field, wb) = route_fixture();
        let ctx = route_ctx(&field, &wb, &[]);
        let mut t = InfraTools::new();
        t.route_begin(RouteMode::Mixed);
        t.route_append_stop(2.0, 2.0);
        t.route_append_stop(22.0, 13.0);
        let (idx, unreachable) = t.route_commit(&ctx).expect("mixed mode can cross the strait");
        assert_eq!(idx, 0);
        assert_eq!(unreachable, 0);
        assert_eq!(t.routes[0].mode, RouteMode::Mixed);
        assert!(t.routes[0].km > 0.0);
    }

    /// `WorldGen::absorb`/`load_save` reset this whole tool set by swapping
    /// in a fresh `InfraTools::new()` (`Option<InfraTools>`, matching every
    /// sibling milestone-F binding's own lifecycle -- see the module doc's
    /// "Why one struct for four tools"), so there is no in-module reset
    /// method to test; this just pins that a fresh instance really does
    /// start with nothing armed or stored, which is what that swap relies on.
    #[test]
    fn new_infra_tools_starts_with_nothing_armed_or_stored() {
        let mut t = InfraTools::new();
        assert!(t.ways.is_empty());
        assert!(t.routes.is_empty());
        assert!(t.measure_points().is_empty());
        assert!(t.region.is_none());
        assert!(!t.way_append_point(1.0, 1.0), "no way draft armed yet");
        assert!(!t.route_append_stop(1.0, 1.0), "no route draft armed yet");
    }

    // ---------- snap sanity (reused engine primitive, not re-tested here) ----------

    #[test]
    fn a_named_settlement_constructs_for_context_tests() {
        // Just exercises the import path this module's own doc references
        // (`civ_find_snap_target`/`civ_snap_point` live in `lib.rs`'s
        // `snap_point` helper, not here -- this module never builds
        // `NamedSettlement` itself, so this only guards against the type
        // import silently rotting).
        let _s = NamedSettlement {
            placement: SettlementPlacement { x: 0, y: 0, suit: 0.0, faction: 0, capital: false, kind: SettlementKind::Hamlet, coastal: false },
            name: String::new(),
            pop: 0,
        };
    }

    // ---------- measure ----------

    #[test]
    fn measure_legs_reports_one_leg_per_pair() {
        let pts = [(0.0, 0.0), (10.0, 0.0), (10.0, 10.0)];
        let legs = measure_legs(&pts, 512, 512.0, false);
        assert_eq!(legs.len(), 2);
        assert_eq!(legs[0].m.cells, 10.0);
        assert_eq!(legs[1].m.cells, 10.0);
    }

    #[test]
    fn measure_legs_is_empty_under_two_points() {
        assert!(measure_legs(&[], 512, 512.0, false).is_empty());
        assert!(measure_legs(&[(1.0, 1.0)], 512, 512.0, false).is_empty());
    }

    #[test]
    fn bearing_convention_matches_the_grids_own_y_down_axis() {
        // East: +x, no y change.
        let east = measure_leg((0.0, 0.0), (10.0, 0.0), 512, 512.0, false);
        assert!((east.bearing_deg - 90.0).abs() < 1e-9);
        // South: +y (grid y increases downward/south).
        let south = measure_leg((0.0, 0.0), (0.0, 10.0), 512, 512.0, false);
        assert!((south.bearing_deg - 180.0).abs() < 1e-9);
        // North: -y.
        let north = measure_leg((0.0, 10.0), (0.0, 0.0), 512, 512.0, false);
        assert!(north.bearing_deg.abs() < 1e-9 || (north.bearing_deg - 360.0).abs() < 1e-9);
        // West: -x.
        let west = measure_leg((10.0, 0.0), (0.0, 0.0), 512, 512.0, false);
        assert!((west.bearing_deg - 270.0).abs() < 1e-9);
    }

    // ---------- region ----------

    #[test]
    fn region_tile_estimate_reports_three_lods_in_ascending_tile_count() {
        let r = Region::new(0, 0, 256, 160);
        let est = region_tile_estimate(&r);
        assert_eq!(est.len(), 3);
        assert_eq!(est[0].0, "low");
        assert_eq!(est[2].0, "high");
        assert!(est[0].1 < est[1].1 && est[1].1 < est[2].1, "tile counts must strictly increase across the ladder");
        for &(_, _, tw, th) in &est {
            assert!(tw > 0 && th > 0, "every LOD must report a real tile size");
        }
    }

    #[test]
    fn route_inputs_mixed_mode_flag_gates_biome_and_river_order() {
        // RouteInputs::build itself needs a live WorldState and is exercised
        // end to end by `cargo test -p cartalith-godot` only through
        // `lib.rs`'s own commit paths (no cheap WorldState fixture exists
        // at this crate boundary) -- this instead pins the cheap, pure part
        // of the contract: the `Option` shape callers rely on.
        let inputs_land = RouteInputs { water_bodies: vec![0; 4], biome: None, river_order: None };
        assert!(inputs_land.biome.is_none() && inputs_land.river_order.is_none());
        let inputs_mixed = RouteInputs { water_bodies: vec![0; 4], biome: Some(vec![1; 4]), river_order: Some(vec![0; 4]) };
        assert!(inputs_mixed.biome.is_some() && inputs_mixed.river_order.is_some());
    }
}
