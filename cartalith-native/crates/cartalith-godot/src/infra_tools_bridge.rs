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
//! ## No freehand way mode -- for the click path (ET-D4 adds one for the drag)
//!
//! **ET-D4 (Ruling CB item 7) supersedes this section for the drag-to-draw
//! gesture only:** [`WayDragMode::Freehand`] commits the resampled drag as a
//! cell-by-cell line with no routing, behind an explicit toggle whose default is
//! the routed mode. The click path (`way_begin` / `way_append_point` /
//! `way_commit`) is untouched and still has no freehand mode; the prose below
//! describes why, and stays true of it. See the ET-D4 section further down for
//! the drag lifecycle and what freehand does at water.
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

use cartalith_civ::tools::{
    civ_commit_way, civ_commit_way_guided, civ_join_dijkstra_segs, ManualWay, ManualWayType, RouteContext, RouteGuide, RouteMode,
};
use cartalith_spatial::stroke::{Sample, StrokeConfig, StrokeEngine, StrokeError};
use cartalith_spatial::{js_round, measure, tile_dims, Measurement, Region};
use cartalith_terrain::sculpt::Point;

use crate::sculpt_bridge::{mirrored_polylines, StrokeSymmetry};

// ===================== Way/Route drafts =====================

/// A waypoint chain being drawn for the Way tool, before `way_commit` turns
/// it into a real [`ManualWay`] (`DCC_SHELL_SPEC.md` §4.5.4: "Click appends
/// a waypoint; Esc commits").
struct WayDraft {
    /// Fixed at `way_begin`; decides the commit's `RouteMode` and the stored way's type.
    way_type: ManualWayType,
    /// Clicked waypoints in click order, grid coordinates; not yet routed.
    points: Vec<(f64, f64)>,
}

/// The Route tool's mirror of [`WayDraft`] — stops instead of waypoints, and
/// a `RouteMode` chosen once at `route_begin` rather than derived from a
/// type the way `WayDraft`'s is (see the module doc's "no equivalent to
/// `ManualWayType::SeaLane`" note: a route may be `Mixed`, a way never is).
struct RouteDraft {
    /// The cost domain given to `route_begin`; used unchanged at commit.
    mode: RouteMode,
    /// Clicked stops in click order, grid coordinates; not yet routed.
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
    /// The joined least-cost polyline, as `civ_join_dijkstra_segs` returned it.
    pub pts: Vec<(f64, f64)>,
    /// The join's break indices (`brks`) as returned; their meaning belongs to `JoinedPath`.
    pub brks: Vec<usize>,
    /// Total route length in kilometres, from the join.
    pub km: f64,
    /// The cost domain the route was committed under.
    pub mode: RouteMode,
    /// Legs the router could not path and drew straight; `0` means none.
    pub unreachable_legs: usize,
    /// The user's own name for this route, `""` until [`InfraTools::
    /// route_set_name`] is called. Empty is the *reference's* own resting
    /// state, not a placeholder this port invented: a `civJourneys` entry
    /// carries no `name` at all until the list's name field is typed into,
    /// and `_civRenderJourneyList` (reference line 17237) falls back to
    /// `'Journey '+(ji+1)` — an index-derived label computed at draw time,
    /// which is why that fallback belongs to the caller drawing the list
    /// and is deliberately NOT stored here (storing it would survive a
    /// delete and leave "Journey 3" sitting at index 1).
    pub name: String,
}

/// The per-world tool state for Way, Route, Measure and Region select.
///
/// Every field starts empty/`None`; nothing here is computed at
/// construction (unlike [`sculpt_bridge::SculptEditor`], which adopts a
/// generated river lock immediately) because none of these four tools has
/// anything to adopt from a fresh generation — they are all user-driven from
/// the first click.
pub struct InfraTools {
    /// `Some` between `way_begin` and the commit/discard that consumes it.
    way_draft: Option<WayDraft>,
    /// `Some` between `route_begin` and the commit/discard that consumes it.
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
    /// `STORY_PLANNING_SCOPE.md` SP-1: saved journeys, in save order. Lives
    /// beside `routes` rather than as its own `WorldGen` field because a
    /// journey's route is a **snapshot** of a committed route's geometry
    /// (`cartalith_civ::travel_library::JourneyRoute`) taken at save time --
    /// the two share `load_save`'s reset. **A regenerate is the one
    /// exception** (SP-2, Ruling AO): every other field here is dropped with
    /// the old world, but saved journeys are carried and re-snapped onto the
    /// new one (`story_bridge.rs`, `adopt_resnapped` below).
    pub journeys: Vec<cartalith_civ::travel_library::Journey>,
    /// The id `journey_save` hands out next; starts at 1 and only ever rises.
    next_journey_id: u64,
    /// SP-2 (Ruling AO): what the last regenerate's re-snap did to each
    /// journey the previous world held -- `(journey id, journey name,
    /// outcome)`, one row per carried journey, **including the dropped ones**
    /// (`ResnapOutcome::MissingStop`), which are no longer in `journeys` and
    /// would otherwise vanish without a word. Empty on a world no journey was
    /// carried into; not persisted (a report on one regenerate, not state).
    pub resnap_report: Vec<(u64, String, cartalith_civ::journey_progress::ResnapOutcome)>,
    /// ET-D4: the drag-to-draw gesture in progress, `Some` between
    /// `way_drag_begin` and the `way_drag_finish`/`way_drag_cancel` that
    /// consumes it. Independent of `way_draft`: a click draft and a drag never
    /// share state, so clicks keep working exactly as before.
    way_drag: Option<WayDrag>,
    /// ET-D4: the ways the most recent drag commit pushed, held so one
    /// [`InfraTools::way_drag_undo`] can take the whole drag back. Not persisted.
    last_drag: Option<DragRecord>,
}

impl Default for InfraTools {
    /// Same as [`InfraTools::new`]: nothing armed, nothing stored.
    fn default() -> Self {
        Self::new()
    }
}

impl InfraTools {
    /// Empty tool state: no drafts, no committed ways/routes/journeys, no region, journey ids from 1. Never carries anything over from a previous world.
    pub fn new() -> Self {
        Self {
            way_draft: None,
            route_draft: None,
            ways: Vec::new(),
            routes: Vec::new(),
            measure_points: Vec::new(),
            region: None,
            journeys: Vec::new(),
            next_journey_id: 1,
            resnap_report: Vec::new(),
            way_drag: None,
            last_drag: None,
        }
    }

    // ===================== Way =====================

    /// Arms a fresh way draft of `way_type`, silently replacing any draft already in progress (its points are lost).
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

    /// Drops the way draft, if any, without committing; committed ways are untouched.
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

    /// Arms a fresh route draft under `mode`, silently replacing any draft in progress.
    pub fn route_begin(&mut self, mode: RouteMode) {
        self.route_draft = Some(RouteDraft { mode, points: Vec::new() });
    }

    /// Appends a stop to the armed route draft. `false` (and nothing stored) when no draft is armed, as for `way_append_point`.
    pub fn route_append_stop(&mut self, x: f64, y: f64) -> bool {
        match self.route_draft.as_mut() {
            Some(d) => {
                d.points.push((x, y));
                true
            }
            None => false,
        }
    }

    /// Drops the route draft, if any, without committing; committed routes are untouched.
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
        self.routes.push(CommittedRoute {
            pts: j.pts,
            brks: j.brks,
            km: j.km,
            mode: draft.mode,
            unreachable_legs: j.unreachable_legs,
            name: String::new(),
        });
        Some((idx, unreachable))
    }

    /// Removes one committed route, shifting every later index down by one.
    /// `false` for an out-of-range index.
    ///
    /// `Vec::remove`, i.e. the reference's own `civJourneys.splice(ji,1)`
    /// (line 17250) — **not** a tombstone. The renumbering is real and every
    /// caller holding an index (`jp_compute`'s `route` key, `jp_reroute`'s
    /// `route_index`, `route_get`) must re-read after a delete, exactly as
    /// the reference's own list does by re-rendering itself. A tombstoned
    /// slot would keep those indices stable at the cost of `route_count()`
    /// no longer meaning "how many routes there are", which every existing
    /// consumer already assumes it does.
    pub fn route_delete(&mut self, index: usize) -> bool {
        if index >= self.routes.len() {
            return false;
        }
        self.routes.remove(index);
        true
    }

    /// Renames one committed route. `false` for an out-of-range index.
    /// An empty string is a legal name and means "unnamed" — see
    /// [`CommittedRoute::name`] on why the `Journey N` fallback is the
    /// caller's, so clearing the field really does restore it.
    pub fn route_set_name(&mut self, index: usize, name: &str) -> bool {
        match self.routes.get_mut(index) {
            Some(r) => {
                r.name = name.to_string();
                true
            }
            None => false,
        }
    }

    // ===================== Journey (SP-1) =====================

    /// Saves a new [`Journey`](cartalith_civ::travel_library::Journey):
    /// `route_index` names a committed route in `self.routes`, and its
    /// geometry is copied into the journey rather than referenced (see
    /// [`cartalith_civ::travel_library::JourneyRoute`]'s own doc comment on
    /// why). Returns the new journey's stable id, or `None` for an
    /// out-of-range route index -- the same "no route, no journey" guard
    /// `route_get` already applies.
    ///
    /// Id assignment is `civ_assign_tid`'s own idempotent-counter shape,
    /// scoped to this store rather than sharing `CivData::next_tid`: a
    /// journey is not a settlement or a way, and nothing cross-references a
    /// journey by id from outside this store today.
    pub fn journey_save(
        &mut self,
        name: &str,
        party_preset: &str,
        route_index: usize,
        start_year: i64,
    ) -> Option<u64> {
        let r = self.routes.get(route_index)?;
        let id = self.next_journey_id;
        self.next_journey_id += 1;
        self.journeys.push(cartalith_civ::travel_library::Journey {
            id,
            name: name.to_string(),
            party_preset: party_preset.to_string(),
            route: cartalith_civ::travel_library::JourneyRoute {
                points: r.pts.clone(),
                breaks: r.brks.clone(),
                length_km: r.km,
                mode: r.mode,
            },
            start_year,
            // Ruling BS: set afterwards through `journey_set_resupply`, so
            // this signature (and every caller of it) is unchanged.
            resupply_stops: Vec::new(),
            // Ruling AR: likewise set afterwards (`journey_set_plan`); a
            // journey is `None` -- no plan stored -- until then.
            plan: None,
        });
        Some(id)
    }

    /// Removes one saved journey by its stable `id`. `false` if no journey
    /// carries that id -- unlike [`Self::route_delete`], nothing else holds
    /// this id across the call (a journey's own `id` is never used as an
    /// array index), so no renumbering-across-delete hazard exists here.
    pub fn journey_delete(&mut self, id: u64) -> bool {
        let before = self.journeys.len();
        self.journeys.retain(|j| j.id != id);
        self.journeys.len() != before
    }

    /// The saved journey with this stable `id`, or `None`. Looked up by id, never by position.
    pub fn journey_get(&self, id: u64) -> Option<&cartalith_civ::travel_library::Journey> {
        self.journeys.iter().find(|j| j.id == id)
    }

    /// `TRAVEL_LIBRARY_SPEC.md` §4's "how many saved journeys reference it",
    /// made real for a [`PartyPreset`](cartalith_civ::travel_library::
    /// PartyPreset) id -- `TravelLibrary::animal_usage_in_journeys` stays
    /// honestly `0` (SP-1's `Journey` carries no per-animal override to
    /// count), but a preset's own usage no longer has to.
    pub fn preset_usage_in_journeys(&self, preset_id: &str) -> usize {
        self.journeys.iter().filter(|j| j.party_preset == preset_id).count()
    }

    /// The next id [`Self::journey_save`] will hand out -- restored from an
    /// archive's `entities/journeys.json` `next_id` member so a re-save
    /// never collides with a journey the file already carries. Raised, never
    /// lowered, by [`project_bridge`](crate::project_bridge)'s own restore
    /// path, the same `next_id`-cannot-trail-its-data rule §9.1 states for
    /// settlements.
    pub fn set_next_journey_id(&mut self, next: u64) {
        self.next_journey_id = self.next_journey_id.max(next);
    }

    /// The id the next [`Self::journey_save`] will assign (starts at 1).
    pub fn next_journey_id(&self) -> u64 {
        self.next_journey_id
    }

    /// SP-2's regenerate re-snap, applied to this (fresh) store: every
    /// carried journey goes through `resnap_journey` against `ctx_for(mode)`,
    /// the survivors keep their ids, and the id counter is carried so a
    /// dropped journey's id is never reissued. `ctx_for` builds a
    /// `RouteContext` over the NEW world for one mode -- the caller's,
    /// because building one needs `WorldGen`'s rasters.
    pub fn adopt_resnapped(
        &mut self,
        carried: &[cartalith_civ::travel_library::Journey],
        next_id: u64,
        old_places: &[cartalith_civ::NamedSettlement],
        old_dims: (usize, usize),
        ctx_for: &dyn Fn(RouteMode, &mut dyn FnMut(&RouteContext)),
    ) {
        use cartalith_civ::journey_progress::{resnap_journey, ResnapOutcome};
        self.set_next_journey_id(next_id);
        for j in carried {
            let mut result: Option<Result<_, ResnapOutcome>> = None;
            ctx_for(j.route.mode, &mut |ctx| result = Some(resnap_journey(j, old_places, old_dims, ctx)));
            match result.expect("ctx_for calls back exactly once") {
                Ok((nj, outcome)) => {
                    self.journeys.push(nj);
                    self.resnap_report.push((j.id, j.name.clone(), outcome));
                }
                Err(outcome) => self.resnap_report.push((j.id, j.name.clone(), outcome)),
            }
        }
    }

    // ===================== Measure =====================

    /// Starts a new Measure chain by clearing the points; there is no separate armed flag.
    pub fn measure_begin(&mut self) {
        self.measure_points.clear();
    }

    /// Appends one click to the Measure chain. No validation: bounds are the caller's.
    pub fn measure_add_point(&mut self, x: f64, y: f64) {
        self.measure_points.push((x, y));
    }

    /// Empties the Measure chain.
    pub fn measure_clear(&mut self) {
        self.measure_points.clear();
    }

    /// The Measure chain so far, in click order; empty is a normal state.
    pub fn measure_points(&self) -> &[(f64, f64)] {
        &self.measure_points
    }

    // ===================== Region select =====================

    /// Stores `r` as the Region-select marquee, replacing any previous one. No clamping to the grid here.
    pub fn region_set(&mut self, r: Region) {
        self.region = Some(r);
    }

    /// Clears the marquee (`region` becomes `None`).
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

/// [`parse_way_type`]'s inverse, for `get_roads()`'s `way_type` key —
/// the canonical spelling of each variant, chosen so `parse_way_type`
/// round-trips it (the test below asserts exactly that).
///
/// These four strings join `get_roads()`'s existing four generated tiers
/// (`highway`/`regional`/`road`/`track`) in ONE key, which is the
/// reference's own arrangement rather than a convenience: `civWays` holds
/// generated and hand-drawn ways in one flat array and the draw pass
/// (reference line ~15494) branches on `rt.type` alone — a manual `road`
/// and a generated `road` are drawn identically, and `manual` is never
/// consulted while drawing. `road` and `track` therefore deliberately
/// collide across the two vocabularies: the reference collides them too,
/// and it means the same thing in both.
pub fn way_type_key(t: ManualWayType) -> &'static str {
    match t {
        ManualWayType::Road => "road",
        ManualWayType::Track => "track",
        ManualWayType::SeaLane => "sea_lane",
        ManualWayType::Ancient => "ancient",
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

// ===================== ET-D4: drag-to-draw ways =====================
//
// `EDITING_TOOLS_SCOPE.md` ET-D4, Ruling CB item 7. Lifecycle, all on
// `InfraTools`: `way_drag_begin` (options validated, nothing else touched) ->
// `way_drag_sample` per pointer event -> `way_drag_finish` (flushes the engine's
// smoothing tail, returns a `FinishedDrag`) -> the caller optionally snaps its
// ends (`FinishedDrag::snap_endpoints`, which needs `WorldGen`'s places) ->
// `way_drag_commit` (mirrors, validates, routes or rasterises, pushes ALL the
// drag's ways, or refuses and pushes none) -> at most one `way_drag_undo`.

/// Largest distance, in cells, at which the drag's final pointer position counts
/// as already being the last waypoint. **Labelled judgement**: the engine walks
/// a segment in `f64`, so a waypoint meant to land on the pointer can miss it by
/// float noise (~1e-12 cells); 1e-9 is the same tolerance ET-D1's spacing test
/// holds (`EDITING_TOOLS_SCOPE.md`), far below any real pointer movement.
const TAIL_EPS: f64 = 1e-9;

/// What a drag's waypoints become. `Routed` is the default everywhere.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WayDragMode {
    /// Every consecutive waypoint pair is routed by least-cost Dijkstra
    /// (`civ_commit_way`), exactly as a click-drawn way is.
    Routed,
    /// No routing: the way is the waypoints stepped cell by cell with a
    /// deterministic line rasteriser ([`rasterise_cells`]), terrain cost ignored.
    Freehand,
}

impl WayDragMode {
    /// `"routed"` / `"freehand"` (exactly, lower case); anything else is `None`,
    /// never a default, so a typo cannot silently pick a mode.
    pub fn parse(s: &str) -> Option<WayDragMode> {
        match s {
            "routed" => Some(WayDragMode::Routed),
            "freehand" => Some(WayDragMode::Freehand),
            _ => None,
        }
    }

    /// The wire key; the exact inverse of [`WayDragMode::parse`].
    pub fn key(self) -> &'static str {
        match self {
            WayDragMode::Routed => "routed",
            WayDragMode::Freehand => "freehand",
        }
    }
}

/// Everything a drag is configured with, snapshotted at `way_drag_begin` so a
/// mid-drag control change cannot alter a drag already under the pointer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WayDragOptions {
    /// The type every way of this drag is committed as.
    pub way_type: ManualWayType,
    /// Routed (default) or freehand.
    pub mode: WayDragMode,
    /// Waypoint spacing in **cells** (a road has no tip diameter), `0.01..=10`
    /// (the ET-D1 engine's `MIN_SPACING..=MAX_SPACING` applied to a 1-cell base
    /// diameter). `None` is spacing **off**: every distinct pointer sample is a
    /// waypoint, the click path's one-point-per-sample picture.
    pub spacing: Option<f64>,
    /// Pulled-string smoothing length in cells; `0.0` is off.
    pub smoothing: f64,
    /// The engine's RNG seed. **Inert today**: roads use no scatter, jitter or
    /// dynamics, so nothing draws from it; carried so the option set matches
    /// paint's and a later randomised mode needs no signature change.
    pub seed: u64,
    /// Mirroring, applied to the finished (snapped) source waypoints, i.e. to the
    /// **control points, before** smoothing and routing: a mirrored way is the
    /// smoothed/routed reflection of the source drag, not a reflection of its output.
    pub symmetry: StrokeSymmetry,
    /// Smooth the control points with a centripetal Catmull-Rom spline before
    /// they are used (owner requirement 2026-10-11). `false` keeps the raw
    /// polyline through the control points: the exact drag, in freehand mode.
    pub smooth: bool,
    /// The spline's sampling step in cells, [`MIN_SMOOTH_SPACING`]`..=`[`MAX_SMOOTH_SPACING`].
    pub smooth_spacing: f64,
    /// Routed mode only: `Some` routes each leg inside a corridor round the
    /// (smoothed) drag with a pull towards it; `None` is the **explicit off**:
    /// plain least-cost routing between the waypoints, exactly today's click
    /// result. An absent option in the shell is NOT `None`; it is
    /// `Some(GuideOptions::default())` (see [`WayDragOptions::new`]).
    pub guide: Option<GuideOptions>,
}

/// The routed-mode guide: how far the road may leave the drag and how hard it is pulled back.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GuideOptions {
    /// Corridor half-width in cells, `> 0` and at most [`MAX_CORRIDOR`].
    pub corridor: f64,
    /// Pull weight, `0..=`[`MAX_PULL`]: the cost multiplier is `1 + pull * (d / corridor)^2`.
    pub pull: f64,
}

/// Default corridor half-width, cells. **Labelled judgement**: wide enough
/// (3 cells) to step round a 3-cell lake or a steep cell on a 24-cell map, small
/// enough that the road is still visibly the drag; measured only on the unit
/// fixtures below, not on a generated world.
pub const DEFAULT_CORRIDOR: f64 = 3.0;
/// Default pull weight. **Labelled judgement**: 2 makes a corridor-edge cell
/// cost 3x, so terrain must be a clear gain to justify leaving the line.
pub const DEFAULT_PULL: f64 = 2.0;
/// Upper bound of [`GuideOptions::corridor`]. **Labelled judgement**: a 64-cell
/// corridor already stops being a guide.
pub const MAX_CORRIDOR: f64 = 64.0;
/// Upper bound of [`GuideOptions::pull`]. **Labelled judgement**: above 100 is
/// indistinguishable from "on the line" while risking `f32` cost overflow on a
/// steep cell.
pub const MAX_PULL: f64 = 100.0;
/// Default spline sampling step, cells. **Labelled judgement**: one cell, so a
/// rasterised spline step never skips a cell the curve crosses and the guide is
/// finer than the routing grid.
pub const DEFAULT_SMOOTH_SPACING: f64 = 1.0;
/// Smallest accepted spline step, cells. **Labelled judgement**: below 0.25 only
/// multiplies guide segments.
pub const MIN_SMOOTH_SPACING: f64 = 0.25;
/// Largest accepted spline step, cells. **Labelled judgement**: above 8 a curve
/// is a polygon.
pub const MAX_SMOOTH_SPACING: f64 = 8.0;

impl Default for GuideOptions {
    /// [`DEFAULT_CORRIDOR`] and [`DEFAULT_PULL`].
    fn default() -> Self {
        GuideOptions { corridor: DEFAULT_CORRIDOR, pull: DEFAULT_PULL }
    }
}

impl WayDragOptions {
    /// The defaults for `way_type`: routed, spacing off, no pointer smoothing, no
    /// symmetry, spline smoothing on at [`DEFAULT_SMOOTH_SPACING`], guided routing
    /// on at [`GuideOptions::default`].
    pub fn new(way_type: ManualWayType) -> Self {
        WayDragOptions {
            way_type,
            mode: WayDragMode::Routed,
            spacing: None,
            smoothing: 0.0,
            seed: 0,
            symmetry: StrokeSymmetry::default(),
            smooth: true,
            smooth_spacing: DEFAULT_SMOOTH_SPACING,
            guide: Some(GuideOptions::default()),
        }
    }

    /// Range-checks the spline and guide options (the engine checks the rest).
    fn validate(&self) -> Result<(), StrokeError> {
        let ok = |v: f64, lo: f64, hi: f64| v.is_finite() && v >= lo && v <= hi;
        if !ok(self.smooth_spacing, MIN_SMOOTH_SPACING, MAX_SMOOTH_SPACING) {
            return Err(StrokeError::Invalid("smooth_spacing"));
        }
        if let Some(g) = self.guide {
            if !(g.corridor.is_finite() && g.corridor > 0.0 && g.corridor <= MAX_CORRIDOR) {
                return Err(StrokeError::Invalid("corridor"));
            }
            if !ok(g.pull, 0.0, MAX_PULL) {
                return Err(StrokeError::Invalid("pull"));
            }
        }
        Ok(())
    }

    /// The routing domain this drag's ways are routed (or, freehand, validated)
    /// under: `Water` for a sea lane, `Land` otherwise, as `way_draft_mode` says
    /// for a click draft.
    pub fn route_mode(&self) -> RouteMode {
        if self.way_type == ManualWayType::SeaLane { RouteMode::Water } else { RouteMode::Land }
    }
}

/// A drag in progress: the engine resampling the pointer path into waypoints.
struct WayDrag {
    /// Snapshot of the options this drag began with.
    opts: WayDragOptions,
    /// The ET-D1 engine, run with scatter, dynamics and symmetry all off (its
    /// symmetry would mirror BEFORE the endpoint snap; see [`FinishedDrag::copies`]).
    engine: StrokeEngine,
    /// Source waypoints so far, consecutive exact duplicates dropped.
    waypoints: Vec<(f64, f64)>,
    /// The latest finite pointer position, so the drag's end can be a waypoint.
    last_raw: Option<(f64, f64)>,
}

impl WayDrag {
    /// Appends the engine's dabs as waypoints, skipping a dab that sits exactly
    /// on the previous waypoint (a pointer held still repeats a sample; routing a
    /// zero-length leg would only cost a Dijkstra).
    fn push_dabs(&mut self, dabs: &[cartalith_spatial::stroke::Dab]) {
        for d in dabs {
            let p = (d.x, d.y);
            if self.waypoints.last() != Some(&p) {
                self.waypoints.push(p);
            }
        }
    }
}

/// The ways the last drag commit pushed, for [`InfraTools::way_drag_undo`].
struct DragRecord {
    /// Index in `InfraTools::ways` of the drag's first way.
    first: usize,
    /// The pushed ways, whole, in push order.
    ways: Vec<ManualWay>,
}

/// A drag whose pointer has lifted: the resampled source waypoints, ready to be
/// snapped, mirrored and committed.
#[derive(Debug, Clone, PartialEq)]
pub struct FinishedDrag {
    /// The options the drag began with.
    pub opts: WayDragOptions,
    /// Source waypoints in drag order, grid coordinates.
    pub waypoints: Vec<(f64, f64)>,
}

impl FinishedDrag {
    /// Snaps the **first and last** waypoint only, through `snap` (the shell's
    /// snap-to-place/way). Judgement from the scope's risk note: snapping every
    /// dense waypoint zig-zags a road between nearby places. Does nothing for a
    /// freehand drag (freehand is the literal drag; its ends are not moved) or
    /// for fewer than two waypoints. Call before [`InfraTools::way_drag_commit`]
    /// so each mirror is the reflection of the snapped source.
    pub fn snap_endpoints(&mut self, snap: &dyn Fn(f64, f64) -> (f64, f64)) {
        if self.opts.mode != WayDragMode::Routed || self.waypoints.len() < 2 {
            return;
        }
        let last = self.waypoints.len() - 1;
        self.waypoints[0] = snap(self.waypoints[0].0, self.waypoints[0].1);
        self.waypoints[last] = snap(self.waypoints[last].0, self.waypoints[last].1);
    }

    /// The source waypoints and each symmetry copy of them, source first. With
    /// no symmetry that is exactly the source (no engine round trip), so a
    /// symmetry-off drag is byte-identical to the unmirrored waypoints.
    pub fn copies(&self, gw: usize, gh: usize) -> Result<Vec<Vec<(f64, f64)>>, WayDragRefusal> {
        if self.opts.symmetry.mode == cartalith_spatial::stroke::Symmetry::None {
            return Ok(vec![self.waypoints.clone()]);
        }
        let pts: Vec<Point> = self.waypoints.iter().map(|p| Point::new(p.0, p.1)).collect();
        let copies = mirrored_polylines(&pts, &self.opts.symmetry, gw, gh).map_err(|_| WayDragRefusal::Symmetry)?;
        Ok(copies.into_iter().map(|c| c.into_iter().map(|p| (p.x, p.y)).collect()).collect())
    }
}

/// Why a drag commit pushed nothing. Every variant is all-or-nothing: no way of
/// the drag was kept, and the drag itself is consumed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WayDragRefusal {
    /// Fewer than two waypoints (a click, not a drag); the count is carried.
    TooFewWaypoints(usize),
    /// A waypoint of `copy` (0 = source) at index `index` lies outside the grid
    /// `[0, gw) x [0, gh)`. A mirrored drag can leave the map; clamping it onto
    /// the edge would be a road nobody drew.
    OutOfGrid { copy: usize, index: usize },
    /// The symmetry configuration failed the engine's own validation.
    Symmetry,
    /// Freehand only: `copy` rasterised to fewer than two distinct cells.
    Degenerate { copy: usize },
    /// Freehand only: `blocked_cells` cells of `copy` are terrain this way type
    /// may not cross (water for a land way, land for a sea lane); `first` is the
    /// first such cell in drag order. See [`freehand_cell_blocked`].
    Water { copy: usize, blocked_cells: usize, first: (usize, usize) },
}

impl WayDragRefusal {
    /// The wire key the `#[func]` reply carries under `"refusal"`.
    pub fn key(&self) -> &'static str {
        match self {
            WayDragRefusal::TooFewWaypoints(_) => "too_few_waypoints",
            WayDragRefusal::OutOfGrid { .. } => "out_of_grid",
            WayDragRefusal::Symmetry => "symmetry",
            WayDragRefusal::Degenerate { .. } => "degenerate",
            WayDragRefusal::Water { .. } => "water",
        }
    }
}

/// What a successful drag commit pushed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DragCommit {
    /// Index in `InfraTools::ways` of the source way; mirrors follow at `first + 1 ..`.
    pub first: usize,
    /// Ways pushed: 1 with no symmetry, otherwise the mode's copy count.
    pub count: usize,
    /// Routed legs, summed over every way, that fell back to a straight line
    /// (`civ_commit_way`'s v1.99 count); always 0 for a freehand drag.
    pub unreachable_legs: usize,
    /// Guided legs that could not be routed inside their corridor and were
    /// re-routed unguided (a detour wider than the corridor); the road left its
    /// drag there by more than the corridor. 0 for freehand or unguided drags.
    pub relaxed_legs: usize,
}

/// Whether a freehand way of this kind may not occupy a cell of water class
/// `wb` (`RouteContext::water_bodies`: 0 land, 1 ocean, 2 lake).
///
/// **The water decision for freehand (ET-D4).** A routed land way avoids ocean
/// and lake cells (and, when it must cross, is flagged `unreachable_legs`); a
/// routed sea lane avoids land. Freehand has no router to avoid anything, so it
/// holds the same line by **refusing the whole drag** if any cell of any copy is
/// blocked: never a silently-allowed crossing, never a half-drawn way. Rivers are
/// not in `water_bodies` and routed ways cross them (ford or bridge cost), so a
/// freehand way crosses them too, unflagged. Deliberately **no ferry exemption**:
/// the router lets a land way follow an existing sea lane over water, a freehand
/// way never does. The test is per cell, so a diagonal step can still cut the
/// corner of a water cell between two land cells, the same corner-only defect
/// `RC_ENGINE_CHANGES.md` section 6s.4 records for the routed validity test.
pub fn freehand_cell_blocked(wb: u8, sea_lane: bool) -> bool {
    if sea_lane { wb == 0 } else { wb != 0 }
}

/// Splits the spline `line` into one guide polyline per leg of `controls`.
/// `line` passes through every control point, so leg `k` is the stretch of `line`
/// from the first point equal (to 1e-9) to `controls[k]` to the one equal to
/// `controls[k + 1]`, searching forward only so a curve that loops back near an
/// earlier control cannot be matched to it. If no point matches, the nearest
/// point at or after the previous match is used; a leg whose slice is shorter
/// than two points falls back to the straight chord. Deterministic.
fn leg_sections(line: &[(f64, f64)], controls: &[(f64, f64)]) -> Vec<Vec<(f64, f64)>> {
    let mut idx: Vec<usize> = Vec::with_capacity(controls.len());
    let mut from = 0usize;
    for c in controls {
        let mut best = (f64::INFINITY, from);
        for (i, p) in line.iter().enumerate().skip(from) {
            let d = (p.0 - c.0).hypot(p.1 - c.1);
            if d < best.0 {
                best = (d, i);
            }
            if d < 1e-9 {
                break;
            }
        }
        idx.push(best.1);
        from = best.1;
    }
    (0..controls.len().saturating_sub(1))
        .map(|k| {
            let sec: Vec<(f64, f64)> = line[idx[k]..=idx[k + 1]].to_vec();
            if sec.len() >= 2 { sec } else { vec![controls[k], controls[k + 1]] }
        })
        .collect()
}


/// The grid cell a point lies in for the way system: `js_round` clamped to the
/// grid, exactly `TerrainValid::check`'s rule (`cartalith-civ`), so a freehand
/// way's cells and the router's validity test agree on what "this cell" means.
fn way_cell(p: (f64, f64), gw: usize, gh: usize) -> (i64, i64) {
    ((js_round(p.0) as i64).clamp(0, gw as i64 - 1), (js_round(p.1) as i64).clamp(0, gh as i64 - 1))
}

/// Appends the cells of the Bresenham line `a -> b` to `out`, skipping a first
/// cell equal to `out`'s last so consecutive segments share their junction once.
/// The all-octant integer form (error term `dx + dy`, step x when `2 err >= dy`
/// and y when `2 err <= dx`): the same two inputs always give the same cells, and
/// the tie rule is that `>=` / `<=` pair, pinned by the tests' literals.
fn bresenham(a: (i64, i64), b: (i64, i64), out: &mut Vec<(i64, i64)>) {
    let (mut x, mut y) = a;
    let dx = (b.0 - a.0).abs();
    let dy = -(b.1 - a.1).abs();
    let sx = if a.0 < b.0 { 1 } else { -1 };
    let sy = if a.1 < b.1 { 1 } else { -1 };
    let mut err = dx + dy;
    loop {
        if out.last() != Some(&(x, y)) {
            out.push((x, y));
        }
        if (x, y) == b {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
}

/// The cells a polyline passes through, one per step, no consecutive repeats:
/// each point is placed in its cell ([`way_cell`]) and consecutive cells are
/// joined by [`bresenham`]. Deterministic and independent of terrain.
pub fn rasterise_cells(pts: &[(f64, f64)], gw: usize, gh: usize) -> Vec<(usize, usize)> {
    let mut cells: Vec<(i64, i64)> = Vec::new();
    let mut prev: Option<(i64, i64)> = None;
    for &p in pts {
        let c = way_cell(p, gw, gh);
        match prev {
            None => cells.push(c),
            Some(q) => bresenham(q, c, &mut cells),
        }
        prev = Some(c);
    }
    cells.into_iter().map(|(x, y)| (x as usize, y as usize)).collect()
}

impl InfraTools {
    /// Opens a drag. Validates every option **first** (the ET-D1 engine's own
    /// range checks for spacing, smoothing and seed-independent config, and the
    /// symmetry probe `begin_stroke` uses) and, on `Err`, changes nothing --
    /// a drag already open is left alone. A successful begin replaces an open drag.
    pub fn way_drag_begin(&mut self, opts: WayDragOptions, gw: usize, gh: usize) -> Result<(), StrokeError> {
        // Base diameter 1.0 so spacing, a fraction of the diameter, is in cells.
        opts.validate()?;
        let mut cfg = StrokeConfig::new(opts.seed, 1.0);
        match opts.spacing {
            Some(s) => cfg.spacing = s,
            None => cfg.per_sample = true,
        }
        cfg.smoothing = opts.smoothing;
        let engine = StrokeEngine::new(cfg)?;
        mirrored_polylines(&[Point::new(0.0, 0.0)], &opts.symmetry, gw, gh)?;
        self.way_drag = Some(WayDrag { opts, engine, waypoints: Vec::new(), last_raw: None });
        Ok(())
    }

    /// Feeds one pointer sample (grid cells, `t_ms` on any monotonic clock).
    /// Returns the source waypoint count so far, or `None` with no open drag. A
    /// non-finite position is ignored whole (the count is unchanged).
    pub fn way_drag_sample(&mut self, x: f64, y: f64, t_ms: u64) -> Option<usize> {
        let d = self.way_drag.as_mut()?;
        if !(x.is_finite() && y.is_finite()) {
            return Some(d.waypoints.len());
        }
        d.last_raw = Some((x, y));
        let mut dabs = Vec::new();
        d.engine.feed_one(Sample { x, y, pressure: None, tilt: None, t_ms }, &mut dabs);
        d.push_dabs(&dabs);
        Some(d.waypoints.len())
    }

    /// Drops the open drag without committing anything.
    pub fn way_drag_cancel(&mut self) {
        self.way_drag = None;
    }

    /// Closes the drag: flushes the engine's smoothing tail and makes the last
    /// pointer position a waypoint if the engine's spacing walk stopped short of
    /// it (a road must end where the drag ended; the added leg is at most one
    /// spacing long). `None` with no open drag. The drag is consumed.
    pub fn way_drag_finish(&mut self) -> Option<FinishedDrag> {
        let mut d = self.way_drag.take()?;
        let mut dabs = Vec::new();
        d.engine.finish(&mut dabs);
        d.push_dabs(&dabs);
        if let Some(raw) = d.last_raw {
            let at_end = d.waypoints.last().is_some_and(|l| (l.0 - raw.0).hypot(l.1 - raw.1) < TAIL_EPS);
            if !at_end {
                d.waypoints.push(raw);
            }
        }
        Some(FinishedDrag { opts: d.opts, waypoints: d.waypoints })
    }

    /// Commits a finished drag as **one unit**: every symmetry copy becomes its
    /// own [`ManualWay`], all routed (or rasterised) against the same `ctx`, so a
    /// mirror is not routed over its own source's discount; and either every way
    /// is pushed or none is. Returns what was pushed, or why nothing was.
    ///
    /// Each copy's control points are first smoothed (`opts.smooth`) into a
    /// centripetal Catmull-Rom guide line, `cartalith_civ::civ_catmull_rom_sample`
    /// (Barry-Goldman evaluation with knot spacing `chord^0.5`, i.e. alpha 0.5; the
    /// function the way renderer already uses, reused rather than duplicated).
    ///
    /// Routed with `opts.guide == None` is the **explicit off**: each copy goes
    /// through `civ_commit_way`, the click path's own call, on the raw control
    /// points, so with spacing, spline and symmetry off the result is byte-identical
    /// to the same waypoints clicked one by one (pinned by
    /// `a_plain_routed_drag_is_byte_identical_to_the_click_path`). Routed with
    /// `Some(guide)` routes each leg through `civ_commit_way_guided`, restricted to
    /// a corridor round that leg's slice of the guide line with a pull towards it.
    /// Freehand: see [`freehand_cell_blocked`]
    /// and [`rasterise_cells`], run over the guide line; the way's points are the cells themselves
    /// (integer coordinates, the convention `TerrainValid::check` reads back), it
    /// has no breaks, and its `km` is the sum of the step lengths at
    /// `ctx.map_width_km / ctx.gw` (no world wrap: a freehand step is one cell).
    pub fn way_drag_commit(&mut self, ctx: &RouteContext, drag: FinishedDrag) -> Result<DragCommit, WayDragRefusal> {
        let n = drag.waypoints.len();
        if n < 2 {
            return Err(WayDragRefusal::TooFewWaypoints(n));
        }
        let copies = drag.copies(ctx.gw, ctx.gh)?;
        for (ci, c) in copies.iter().enumerate() {
            for (pi, p) in c.iter().enumerate() {
                if !(p.0.is_finite() && p.1.is_finite() && p.0 >= 0.0 && p.1 >= 0.0 && p.0 < ctx.gw as f64 && p.1 < ctx.gh as f64) {
                    return Err(WayDragRefusal::OutOfGrid { copy: ci, index: pi });
                }
            }
        }
        let sea = drag.opts.way_type == ManualWayType::SeaLane;
        let mut built: Vec<ManualWay> = Vec::with_capacity(copies.len());
        let mut unreachable = 0usize;
        let mut relaxed_total = 0usize;
        for (ci, c) in copies.iter().enumerate() {
            // The guide line: the centripetal spline through this copy's control
            // points, or the raw polyline when smoothing is off. Built from the
            // already-mirrored control points: symmetry is before smoothing.
            let line: Vec<(f64, f64)> =
                if drag.opts.smooth { cartalith_civ::civ_catmull_rom_sample(c, drag.opts.smooth_spacing) } else { c.clone() };
            if line.len() < 2 {
                return Err(WayDragRefusal::Degenerate { copy: ci });
            }
            match drag.opts.mode {
                WayDragMode::Routed => {
                    let (commit, relaxed) = match drag.opts.guide {
                        None => (civ_commit_way(ctx, c, drag.opts.way_type).ok_or(WayDragRefusal::TooFewWaypoints(c.len()))?, 0),
                        Some(g) => {
                            let sections = leg_sections(&line, c);
                            let guides: Vec<RouteGuide> =
                                sections.iter().map(|s| RouteGuide { pts: s, half_width: g.corridor, pull: g.pull }).collect();
                            civ_commit_way_guided(ctx, c, drag.opts.way_type, Some(&guides))
                                .ok_or(WayDragRefusal::TooFewWaypoints(c.len()))?
                        }
                    };
                    unreachable += commit.unreachable_legs;
                    relaxed_total += relaxed;
                    built.push(commit.way);
                }
                WayDragMode::Freehand => {
                    let cells = rasterise_cells(&line, ctx.gw, ctx.gh);
                    if cells.len() < 2 {
                        return Err(WayDragRefusal::Degenerate { copy: ci });
                    }
                    let blocked: Vec<&(usize, usize)> =
                        cells.iter().filter(|&&(x, y)| freehand_cell_blocked(ctx.water_bodies[y * ctx.gw + x], sea)).collect();
                    if let Some(&&first) = blocked.first() {
                        return Err(WayDragRefusal::Water { copy: ci, blocked_cells: blocked.len(), first });
                    }
                    let pts: Vec<(f64, f64)> = cells.iter().map(|&(x, y)| (x as f64, y as f64)).collect();
                    let cell_km = ctx.map_width_km / ctx.gw as f64;
                    let km: f64 = pts.windows(2).map(|w| cartalith_jsmath::js_hypot(w[1].0 - w[0].0, w[1].1 - w[0].1) * cell_km).sum();
                    built.push(ManualWay { pts, brks: Vec::new(), km, sea, way_type: drag.opts.way_type, name: String::new(), hidden: false });
                }
            }
        }
        let first = self.ways.len();
        let count = built.len();
        self.last_drag = Some(DragRecord { first, ways: built.clone() });
        self.ways.extend(built);
        Ok(DragCommit { first, count, unreachable_legs: unreachable, relaxed_legs: relaxed_total })
    }

    /// Takes the most recent drag commit back as **one step**: removes exactly the
    /// ways it pushed and returns how many. `0` (and nothing changed) when there
    /// is no drag to undo or the way list has moved since (a way added, deleted or
    /// edited after the drag): the held record is then dropped, because putting
    /// back or removing by stale indices would delete a way the drag never made.
    /// Single-level: undoing twice, or a second drag, replaces the record.
    ///
    /// This is a drag-level primitive; wiring it into the global Edit > Undo ledger
    /// needs an `undo::EntryKind` (a file outside ET-D4's Rust lane), so the shell
    /// calls it directly for now.
    pub fn way_drag_undo(&mut self) -> usize {
        let Some(rec) = self.last_drag.take() else { return 0 };
        let end = rec.first + rec.ways.len();
        if self.ways.len() != end || self.ways[rec.first..] != rec.ways[..] {
            return 0;
        }
        self.ways.truncate(rec.first);
        rec.ways.len()
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
///
/// **Forced lakes** (`WorldGen::forced_lakes`, Ruling BO) are folded in via
/// [`Self::build`]'s `forced_lakes` parameter, applied with the same
/// `render::apply_forced_lakes` post-pass `WorldGen::drawn_water_bodies`
/// uses — "fresh, not cached" above is about staying independent of
/// `compute_civilisation`'s copy, not about ignoring a forced lake, which is
/// no less part of "the water the map draws" than the height field is.
/// Found 2026-09-28 (`OUTSTANDING_WORK.md` "Map-data residuals"): before this,
/// a road or route drawn straight over a stamped-and-forced lake, or a `Mixed`
/// biome raster built for one, both used the lake's plain `build_water_bodies`
/// reading (dry, since a forced lake need not clear the rain gate) — the
/// water pathfinding avoids and the water the map paints disagreed. `apply_forced_lakes`
/// is a no-op when `forced_lakes` is `None` or all-zero, so a world with no
/// forced lake computes exactly what it did before.
pub struct RouteInputs {
    /// Per-cell water classification (`0` land, `1` ocean, `2` lake in this module's fixtures and forced-lake test), forced lakes applied.
    pub water_bodies: Vec<u8>,
    /// Biome raster; `Some` only for `RouteMode::Mixed`.
    pub biome: Option<Vec<u8>>,
    /// Fresh river-order raster; `Some` only for `RouteMode::Mixed`.
    pub river_order: Option<Vec<i16>>,
    /// `build_route_corridors`' field — `DECISIONS.md` §7i. Built for every
    /// land-capable mode (`Land` and `Mixed`; `Water` has no slope term for
    /// it to relieve), and only when the world is big enough for the
    /// detector's own `gw/64` search radius to mean anything.
    pub corridors: Option<Vec<f32>>,
}

impl RouteInputs {
    /// `forced_lakes`: `WorldGen::forced_lakes.as_deref()` — the same mask
    /// `WorldGen::drawn_water_bodies` applies. `None` (no world has ever had
    /// one forced) or all-zero leaves `wb.classification` exactly as
    /// `build_water_bodies` and Ruling CC's lake basin-area floor returned it
    /// (`render::water_bodies_floored`) — see [`RouteInputs`]'s own doc
    /// comment.
    #[allow(clippy::too_many_arguments)]
    pub fn build(
        ws: &cartalith_engine::WorldState,
        gw: usize,
        gh: usize,
        world: bool,
        map_width_km: f64,
        river_density: f64,
        mode: RouteMode,
        forced_lakes: Option<&[u8]>,
    ) -> Self {
        let sea_level = ws.sea_level;
        // Ruling CC's floor, then the forced lakes: the drawn map's water
        // (`render::water_bodies_floored`).
        let wb = crate::render::water_bodies_floored(&ws.field, &ws.rainfall, gw, gh, sea_level, world, forced_lakes);
        let (biome, river_order) = if mode == RouteMode::Mixed {
            let biome = cartalith_civ::build_biome_raster(&wb.classification, &ws.temperature, &ws.rainfall);
            let river_order = cartalith_civ::fresh_river_order(&ws.field, &ws.flow_discharge, gw, gh, sea_level, world, river_density, map_width_km, ws.integrated_drainage);
            (Some(biome), Some(river_order))
        } else {
            (None, None)
        };
        // `DECISIONS.md` §7i. Land and mixed both have a slope term; water
        // does not. `gw >= 128` keeps the detector off worlds where its own
        // `max(2, gw/64)` reach collapses to two cells and it would be
        // measuring noise rather than a range — the small fixtures this
        // crate's own unit tests use are all below it.
        let corridors = (mode != RouteMode::Water && gw >= 128).then(|| {
            let slope = cartalith_civ::build_raw_slope_field(&ws.field, gw, gh, world);
            cartalith_civ::build_route_corridors(
                &ws.field,
                &slope,
                Some(&ws.flow_discharge),
                gw,
                gh,
                sea_level,
                world,
                cartalith_hydrology::river_flow_thresh(gw, gh, gw, map_width_km),
            )
        });
        RouteInputs { water_bodies: wb.classification, biome, river_order, corridors }
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
    /// The unmodified `cartalith_spatial::measure` result for this leg.
    pub m: Measurement,
    /// Compass bearing in degrees within `[0, 360)`; `0` = north, clockwise.
    pub bearing_deg: f64,
}

/// One leg `a` -> `b`: `cartalith_spatial::measure` plus a compass bearing (north = 0, clockwise, grid `y` down), normalised to `[0, 360)`. A zero-length leg has no meaningful bearing; do not rely on the value.
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

/// Godot-free unit tests for the draft state machines, parsers and measure/region helpers above.
#[cfg(test)]
mod tests {
    use super::*;
    use cartalith_civ::tools::WayRef;
    use cartalith_civ::{NamedSettlement, SettlementKind, SettlementPlacement};

    // ---------- string parsing ----------

    /// Protects: each of the four engine way types parses (case/hyphen-insensitive) and a non-engine label like `bridge` is rejected, not defaulted.
    #[test]
    fn parse_way_type_covers_all_four_reference_entries() {
        assert_eq!(parse_way_type("road"), Some(ManualWayType::Road));
        assert_eq!(parse_way_type("Track"), Some(ManualWayType::Track));
        assert_eq!(parse_way_type("SEA-LANE"), Some(ManualWayType::SeaLane));
        assert_eq!(parse_way_type("sealane"), Some(ManualWayType::SeaLane));
        assert_eq!(parse_way_type("ancient"), Some(ManualWayType::Ancient));
        assert_eq!(parse_way_type("bridge"), None, "not a real engine way type -- see the module doc");
    }

    /// Protects: `way_type_key` output parses back to the same variant, and `sea_lane` keeps the exact spelling `get_roads()` branches on.
    #[test]
    fn way_type_key_round_trips_through_parse_way_type() {
        for t in [ManualWayType::Road, ManualWayType::Track, ManualWayType::SeaLane, ManualWayType::Ancient] {
            assert_eq!(parse_way_type(way_type_key(t)), Some(t), "{} must parse back to itself", way_type_key(t));
        }
        // The `get_roads()` contract this key feeds: only `sea_lane`
        // diverts a manual way to `get_sea_routes()` -- assert the exact
        // string that branch tests against, so renaming it here breaks a
        // test rather than silently drawing sea lanes as roads.
        assert_eq!(way_type_key(ManualWayType::SeaLane), "sea_lane");
    }

    /// Protects: `land`/`water`/`mixed`/`least-cost` map to modes and `freehand`/`snap`/unknown return `None` rather than a silent default.
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

    /// Test fixture (not a test): a tiny two-band world: land on the right, ocean on the left, so both
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

    /// Test fixture: a `RouteContext` over [`route_fixture`]'s 24x16 grid, sea level 0.42, 240 km wide; no places, biome or river data.
    fn route_ctx<'a>(field: &'a [f32], wb: &'a [u8], ways: &'a [WayRef<'a>]) -> RouteContext<'a> {
        RouteContext { field, water_bodies: wb, biome: None, river_order: None, places: &[], ways, gw: 24, gh: 16, sea: 0.42, corridors: None, world: false, map_width_km: 240.0, flow: None, flow_thresh: 0.0 }
    }

    /// Protects: a click with no armed way draft is refused (`false`), not buffered.
    #[test]
    fn way_append_point_is_a_no_op_before_way_begin() {
        let mut t = InfraTools::new();
        assert!(!t.way_append_point(1.0, 1.0));
    }

    /// Protects: discarding a way draft disarms it and commits nothing.
    #[test]
    fn way_discard_clears_the_draft_without_committing() {
        let mut t = InfraTools::new();
        t.way_begin(ManualWayType::Road);
        t.way_append_point(12.0, 2.0);
        t.way_discard();
        assert!(t.way_draft_mode().is_none());
        assert!(t.ways.is_empty());
    }

    /// Protects: a one-waypoint commit returns `None`, stores no way, and still consumes the draft.
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

    /// Protects: a two-waypoint land commit stores one non-sea way of the drafted type, returns index 0 and zero unreachable legs.
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

    /// Protects: a sea-lane draft reports `RouteMode::Water` (only sea lanes route by water).
    #[test]
    fn a_sea_lane_way_draft_routes_water_mode() {
        let mut t = InfraTools::new();
        t.way_begin(ManualWayType::SeaLane);
        assert_eq!(t.way_draft_mode(), Some(RouteMode::Water));
    }

    /// Protects: way commit indices follow push order (second way is index 1).
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

    /// Protects: a stop with no armed route draft is refused (`false`).
    #[test]
    fn route_append_stop_is_a_no_op_before_route_begin() {
        let mut t = InfraTools::new();
        assert!(!t.route_append_stop(1.0, 1.0));
    }

    /// Protects: a one-stop route commit returns `None` and stores no route.
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

    /// Protects: a `Mixed` route crossing the strait commits at index 0, keeps its `Mixed` mode, has positive length and no unreachable legs.
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

    /// Two routes so the delete/rename tests below can tell "removed the
    /// right one" from "removed one".
    fn two_routes(t: &mut InfraTools, ctx: &RouteContext) {
        for (a, b) in [((2.0, 2.0), (22.0, 13.0)), ((2.0, 6.0), (22.0, 6.0))] {
            t.route_begin(RouteMode::Mixed);
            t.route_append_stop(a.0, a.1);
            t.route_append_stop(b.0, b.1);
            t.route_commit(ctx).expect("mixed mode can cross the strait");
        }
    }

    /// Protects: deleting route 0 shifts later routes down (a tombstone would fail), matching `civJourneys.splice`.
    #[test]
    fn route_delete_removes_that_route_and_renumbers_the_rest() {
        let (field, wb) = route_fixture();
        let ctx = route_ctx(&field, &wb, &[]);
        let mut t = InfraTools::new();
        two_routes(&mut t, &ctx);
        t.route_set_name(1, "Salt road");
        assert!(t.route_delete(0));
        assert_eq!(t.routes.len(), 1);
        // The renumbering is the point: what was index 1 is now index 0,
        // matching `civJourneys.splice(ji,1)`. A tombstone would fail here.
        assert_eq!(t.routes[0].name, "Salt road");
    }

    /// Protects: out-of-range deletes return `false` and leave the list intact.
    #[test]
    fn route_delete_refuses_an_out_of_range_index_without_touching_the_list() {
        let (field, wb) = route_fixture();
        let ctx = route_ctx(&field, &wb, &[]);
        let mut t = InfraTools::new();
        two_routes(&mut t, &ctx);
        assert!(!t.route_delete(2));
        assert!(!t.route_delete(99));
        assert_eq!(t.routes.len(), 2);
        assert!(!InfraTools::new().route_delete(0), "nothing to delete before any commit");
    }

    /// Protects: renaming affects only the target route, an empty name restores the unnamed state, and a bad index returns `false`.
    #[test]
    fn route_set_name_stores_the_name_and_an_empty_string_clears_it() {
        let (field, wb) = route_fixture();
        let ctx = route_ctx(&field, &wb, &[]);
        let mut t = InfraTools::new();
        two_routes(&mut t, &ctx);
        assert_eq!(t.routes[0].name, "", "a fresh commit is unnamed -- the reference's own resting state");
        assert!(t.route_set_name(0, "Amber way"));
        assert_eq!(t.routes[0].name, "Amber way");
        assert_eq!(t.routes[1].name, "", "renaming one route must not touch its neighbour");
        assert!(t.route_set_name(0, ""));
        assert_eq!(t.routes[0].name, "", "clearing restores the caller's `Journey N` fallback");
        assert!(!t.route_set_name(5, "nowhere"));
    }

    // ---------- Journey (SP-1) ----------

    /// Protects: SP-2's regenerate re-snap as the store applies it: the survivor keeps
    /// its id, the journey whose stop vanished is dropped from `journeys` but
    /// still reported by name, and the id counter is carried so the dropped
    /// id is never reissued.
    #[test]
    fn adopt_resnapped_keeps_survivors_reports_the_dropped_and_carries_the_counter() {
        use cartalith_civ::journey_progress::ResnapOutcome;
        use cartalith_civ::{NamedSettlement, SettlementKind, SettlementPlacement};
        let (field, wb) = route_fixture();
        let town = |tid: u64, name: &str, x: usize, y: usize| NamedSettlement {
            tid,
            placement: SettlementPlacement { x, y, suit: 1.0, faction: 0, capital: false, kind: SettlementKind::Town, coastal: false },
            name: name.into(),
            pop: 100,
        };
        let old = vec![town(1, "Ard", 12, 2), town(2, "Bel", 20, 13)];
        let new = vec![town(1, "Ard", 13, 3)]; // Bel is gone
        let journey = |id: u64, name: &str, pts: Vec<(f64, f64)>| cartalith_civ::travel_library::Journey {
            id,
            name: name.into(),
            party_preset: "p".into(),
            route: cartalith_civ::travel_library::JourneyRoute { points: pts, breaks: vec![], length_km: 1.0, mode: RouteMode::Land },
            start_year: 5,
            resupply_stops: vec![],
            plan: None,
        };
        let carried = vec![
            journey(3, "To the coast", vec![(12.0, 2.0), (22.0, 8.0)]),
            journey(4, "To Bel", vec![(12.0, 2.0), (20.0, 13.0)]),
        ];
        let mut t = InfraTools::new();
        let ctx_for = |_: RouteMode, f: &mut dyn FnMut(&RouteContext)| f(&RouteContext { places: &new, ..route_ctx(&field, &wb, &[]) });
        t.adopt_resnapped(&carried, 9, &old, (24, 16), &ctx_for);

        assert_eq!(t.journeys.iter().map(|j| j.id).collect::<Vec<_>>(), vec![3]);
        assert_eq!(t.journeys[0].route.points.first(), Some(&(13.0, 3.0)), "follows Ard to its new cell");
        assert_eq!(
            t.resnap_report,
            vec![
                (3, "To the coast".to_string(), ResnapOutcome::Resnapped { unreachable_legs: 0 }),
                (4, "To Bel".to_string(), ResnapOutcome::MissingStop { name: "Bel".into() }),
            ]
        );
        assert_eq!(t.next_journey_id(), 9, "id 4 is never handed out again");
        let (tfield, twb) = route_fixture();
        let tctx = route_ctx(&tfield, &twb, &[]);
        two_routes(&mut t, &tctx);
        assert_eq!(t.journey_save("next", "p", 0, 1), Some(9));
    }

    /// Protects: a saved journey copies the named route's geometry and mode and gets distinct, increasing ids.
    #[test]
    fn journey_save_snapshots_the_named_route_and_assigns_increasing_ids() {
        let (field, wb) = route_fixture();
        let ctx = route_ctx(&field, &wb, &[]);
        let mut t = InfraTools::new();
        two_routes(&mut t, &ctx);
        let route1_pts = t.routes[1].pts.clone();

        let id_a = t.journey_save("Salt road", "merchant_caravan", 1, 412).expect("route 1 exists");
        let id_b = t.journey_save("Second trip", "merchant_caravan", 1, 500).expect("route 1 exists");
        assert_ne!(id_a, id_b, "each save gets its own stable id");
        assert!(id_b > id_a, "ids increase, matching civ_assign_tid's own counter shape");

        let j = t.journey_get(id_a).expect("just saved");
        assert_eq!(j.name, "Salt road");
        assert_eq!(j.party_preset, "merchant_caravan");
        assert_eq!(j.start_year, 412);
        assert_eq!(j.route.points, route1_pts, "the route is copied, not re-derived");
        assert_eq!(j.route.mode, RouteMode::Mixed);
    }

    /// Protects: a save naming a missing route returns `None` and pushes no partial journey.
    #[test]
    fn journey_save_refuses_an_out_of_range_route_index() {
        let mut t = InfraTools::new();
        assert_eq!(t.journey_save("x", "p", 0, 1), None, "no route committed yet");
        let (field, wb) = route_fixture();
        let ctx = route_ctx(&field, &wb, &[]);
        two_routes(&mut t, &ctx);
        assert_eq!(t.journey_save("x", "p", 5, 1), None);
        assert!(t.journeys.is_empty(), "a refused save must not push a partial entry");
    }

    /// Protects: deletion is keyed on id, repeat deletion returns `false`, and the survivor is untouched.
    #[test]
    fn journey_delete_removes_by_id_not_by_position() {
        let (field, wb) = route_fixture();
        let ctx = route_ctx(&field, &wb, &[]);
        let mut t = InfraTools::new();
        two_routes(&mut t, &ctx);
        let id_a = t.journey_save("A", "p", 0, 1).unwrap();
        let id_b = t.journey_save("B", "p", 1, 1).unwrap();
        assert!(t.journey_delete(id_a));
        assert!(!t.journey_delete(id_a), "already gone");
        assert_eq!(t.journeys.len(), 1);
        assert_eq!(t.journeys[0].id, id_b, "deleting by id must not touch the survivor");
        assert!(t.journey_get(id_a).is_none());
        assert!(t.journey_get(id_b).is_some());
    }

    /// Protects: usage counts only journeys with the named preset; unknown presets and the empty store give `0`.
    #[test]
    fn preset_usage_in_journeys_counts_only_the_named_preset() {
        let (field, wb) = route_fixture();
        let ctx = route_ctx(&field, &wb, &[]);
        let mut t = InfraTools::new();
        two_routes(&mut t, &ctx);
        assert_eq!(t.preset_usage_in_journeys("merchant_caravan"), 0, "the honest 0 before any journey exists");
        t.journey_save("A", "merchant_caravan", 0, 1).unwrap();
        t.journey_save("B", "merchant_caravan", 1, 1).unwrap();
        t.journey_save("C", "hunting_party", 0, 1).unwrap();
        assert_eq!(t.preset_usage_in_journeys("merchant_caravan"), 2);
        assert_eq!(t.preset_usage_in_journeys("hunting_party"), 1);
        assert_eq!(t.preset_usage_in_journeys("no_such_preset"), 0);
    }

    /// Protects: a fresh instance starts empty, which `WorldGen::absorb`/`load_save`
    /// rely on. They reset this whole tool set by swapping
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
        assert!(t.journeys.is_empty());
        assert_eq!(t.next_journey_id(), 1);
        assert!(t.measure_points().is_empty());
        assert!(t.region.is_none());
        assert!(!t.way_append_point(1.0, 1.0), "no way draft armed yet");
        assert!(!t.route_append_stop(1.0, 1.0), "no route draft armed yet");
    }

    // ---------- snap sanity (reused engine primitive, not re-tested here) ----------

    /// Protects: only that the `NamedSettlement` imports still compile and construct; it has no assertions, so it guards nothing else.
    #[test]
    fn a_named_settlement_constructs_for_context_tests() {
        // Just exercises the import path this module's own doc references
        // (`civ_find_snap_target`/`civ_snap_point` live in `lib.rs`'s
        // `snap_point` helper, not here -- this module never builds
        // `NamedSettlement` itself, so this only guards against the type
        // import silently rotting).
        let _s = NamedSettlement {
            tid: 0,
            placement: SettlementPlacement { x: 0, y: 0, suit: 0.0, faction: 0, capital: false, kind: SettlementKind::Hamlet, coastal: false },
            name: String::new(),
            pop: 0,
        };
    }

    // ---------- measure ----------

    /// Protects: a chain of three points yields two legs with the expected cell lengths.
    #[test]
    fn measure_legs_reports_one_leg_per_pair() {
        let pts = [(0.0, 0.0), (10.0, 0.0), (10.0, 10.0)];
        let legs = measure_legs(&pts, 512, 512.0, false);
        assert_eq!(legs.len(), 2);
        assert_eq!(legs[0].m.cells, 10.0);
        assert_eq!(legs[1].m.cells, 10.0);
    }

    /// Protects: fewer than two points yield no legs (a chain under construction is not an error).
    #[test]
    fn measure_legs_is_empty_under_two_points() {
        assert!(measure_legs(&[], 512, 512.0, false).is_empty());
        assert!(measure_legs(&[(1.0, 1.0)], 512, 512.0, false).is_empty());
    }

    /// Protects: bearings follow the grid convention: east 90, south 180, north 0/360, west 270.
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

    /// Protects: the estimate lists low/medium/high with strictly increasing tile counts and non-zero tile sizes.
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

    /// Protects: `RouteInputs::build`'s forced-lake plumbing
    /// (`OUTSTANDING_WORK.md` "Map-data residuals", item 3): a cell the
    /// caller forces reads `2` (lake) in the returned `water_bodies`, the
    /// same class [`crate::apply_forced_lakes`] gives it in
    /// `WorldGen::drawn_water_bodies` -- so a way/route no longer routes
    /// straight through a forced lake the map draws as water. `None`
    /// reproduces the plain build exactly, protecting "no change to worlds
    /// with no forced lake".
    #[test]
    fn route_inputs_build_applies_a_forced_lake_mask() {
        let mut p = crate::params::defaults();
        p.gw = 64;
        p.gh = 64;
        p.tect.seed = 777;
        p.use_gpu = false;
        let ws = cartalith_engine::generate_terrain(&p);
        let plain = RouteInputs::build(&ws, p.gw, p.gh, p.world, p.map_width_km, p.river_density, RouteMode::Land, None);
        let land_idx = plain.water_bodies.iter().position(|&c| c == 0).expect("premise: some land exists in the fixture world");
        let mut mask = vec![0u8; p.gw * p.gh];
        mask[land_idx] = 1;
        let forced = RouteInputs::build(&ws, p.gw, p.gh, p.world, p.map_width_km, p.river_density, RouteMode::Land, Some(&mask));
        assert_eq!(forced.water_bodies[land_idx], 2, "the forced cell routes as lake");
        assert_eq!(
            RouteInputs::build(&ws, p.gw, p.gh, p.world, p.map_width_km, p.river_density, RouteMode::Land, None).water_bodies,
            plain.water_bodies,
            "no mask, no change"
        );
    }

    /// Protects: NOTHING MEANINGFUL: it builds `RouteInputs` literals and asserts the fields it just set, so it exercises no `RouteInputs::build` logic (the forced-lake test is the only real `build` test).
    #[test]
    fn route_inputs_mixed_mode_flag_gates_biome_and_river_order() {
        // `cartalith_engine::generate_terrain` (used just above, in
        // `route_inputs_build_applies_a_forced_lake_mask`) is a real, if not
        // free, `WorldState` fixture -- this test predates that discovery and
        // still pins the cheap, pure part of the contract directly: the
        // `Option` shape callers rely on, with no generation cost at all.
        let inputs_land = RouteInputs { water_bodies: vec![0; 4], biome: None, river_order: None, corridors: None };
        assert!(inputs_land.biome.is_none() && inputs_land.river_order.is_none());
        let inputs_mixed = RouteInputs { water_bodies: vec![0; 4], biome: Some(vec![1; 4]), river_order: Some(vec![0; 4]), corridors: Some(vec![0.0; 4]) };
        assert!(inputs_mixed.biome.is_some() && inputs_mixed.river_order.is_some());
        // `corridors` is the one field that is NOT mixed-gated -- land mode
        // needs it too (DECISIONS.md 7i), and only `Water` and a world under
        // 128 cells wide suppress it.
        assert!(inputs_mixed.corridors.is_some());
    }

    // ---------- ET-D4: drag-to-draw ways ----------

    /// Test fixture: [`route_fixture`] plus a 3x3 lake (`water_bodies == 2`, field below sea level) at x 15..=17, y 7..=9: an obstacle a land road must step round.
    fn lake_fixture() -> (Vec<f32>, Vec<u8>) {
        let (mut field, mut wb) = route_fixture();
        for y in 7..=9 {
            for x in 15..=17 {
                field[y * 24 + x] = 0.30;
                wb[y * 24 + x] = 2;
            }
        }
        (field, wb)
    }

    /// Test fixture: the options of a drag configured as **today's click path**: routed, no spacing, no spline, no guide. Everything the ET-D4 owner requirement added is explicitly off.
    fn plain_opts(t: ManualWayType) -> WayDragOptions {
        WayDragOptions { smooth: false, guide: None, ..WayDragOptions::new(t) }
    }

    /// Test helper: runs a whole drag (begin, one sample per point, finish, commit) on `t` against a 24x16 grid and returns the commit result.
    fn run_drag(t: &mut InfraTools, opts: WayDragOptions, pts: &[(f64, f64)], ctx: &RouteContext) -> Result<DragCommit, WayDragRefusal> {
        t.way_drag_begin(opts, 24, 16).expect("valid options");
        for (i, p) in pts.iter().enumerate() {
            assert!(t.way_drag_sample(p.0, p.1, i as u64 * 16).is_some());
        }
        let fin = t.way_drag_finish().expect("a drag was open");
        t.way_drag_commit(ctx, fin)
    }

    /// Test helper: the largest distance from any of `pts` to the polyline `line`.
    fn max_dev(pts: &[(f64, f64)], line: &[(f64, f64)]) -> f64 {
        pts.iter()
            .map(|&p| line.windows(2).map(|w| ((p.0 - w[0].0).hypot(p.1 - w[0].1)).min(seg_dist(p, w[0], w[1]))).fold(f64::INFINITY, f64::min))
            .fold(0.0, f64::max)
    }

    /// Test helper: distance from `p` to segment `a`-`b`.
    fn seg_dist(p: (f64, f64), a: (f64, f64), b: (f64, f64)) -> f64 {
        let (dx, dy) = (b.0 - a.0, b.1 - a.1);
        let l2 = dx * dx + dy * dy;
        let t = if l2 < 1e-18 { 0.0 } else { (((p.0 - a.0) * dx + (p.1 - a.1) * dy) / l2).clamp(0.0, 1.0) };
        (p.0 - (a.0 + dx * t)).hypot(p.1 - (a.1 + dy * t))
    }

    /// Protects: the rasteriser's literal cells, including its tie rule: a shallow line, the same line reversed, a steep line, and a two-segment chain whose shared junction cell appears once. A changed tie comparison moves the diagonal step and fails here.
    #[test]
    fn rasterise_cells_is_pinned_to_literals() {
        assert_eq!(rasterise_cells(&[(0.0, 0.0), (3.0, 1.0)], 24, 16), vec![(0, 0), (1, 0), (2, 1), (3, 1)]);
        assert_eq!(rasterise_cells(&[(3.0, 1.0), (0.0, 0.0)], 24, 16), vec![(3, 1), (2, 1), (1, 0), (0, 0)]);
        assert_eq!(rasterise_cells(&[(0.0, 0.0), (1.0, 3.0)], 24, 16), vec![(0, 0), (0, 1), (1, 2), (1, 3)]);
        assert_eq!(
            rasterise_cells(&[(0.0, 0.0), (3.0, 1.0), (3.0, 3.0)], 24, 16),
            vec![(0, 0), (1, 0), (2, 1), (3, 1), (3, 2), (3, 3)]
        );
        // The two exact ties (2 err == dx, 2 err == dy): a changed comparison moves the diagonal step.
        assert_eq!(rasterise_cells(&[(0.0, 0.0), (2.0, 1.0)], 24, 16), vec![(0, 0), (1, 1), (2, 1)]);
        assert_eq!(rasterise_cells(&[(0.0, 0.0), (1.0, 2.0)], 24, 16), vec![(0, 0), (1, 1), (1, 2)]);
        // A point past the right edge is the edge cell (`way_cell`'s clamp), never an index off the grid.
        assert_eq!(rasterise_cells(&[(22.0, 2.0), (23.9, 2.0)], 24, 16), vec![(22, 2), (23, 2)]);
    }

    /// Protects: the waypoint spacing contract in cells: a straight drag fed as three far-apart samples yields waypoints exactly 4 cells apart, first sample included, and the last pointer position is the final waypoint even when the spacing walk stops short of it.
    #[test]
    fn spacing_is_in_cells_and_the_drag_ends_on_the_pointer() {
        let mut t = InfraTools::new();
        t.way_drag_begin(WayDragOptions { spacing: Some(4.0), ..plain_opts(ManualWayType::Road) }, 24, 16).unwrap();
        for (i, x) in [1.0, 11.0, 21.0].iter().enumerate() {
            t.way_drag_sample(*x, 2.0, i as u64).unwrap();
        }
        let fin = t.way_drag_finish().unwrap();
        let xs: Vec<f64> = fin.waypoints.iter().map(|p| p.0).collect();
        assert_eq!(xs.len(), 6, "{xs:?}");
        for (got, want) in xs.iter().zip([1.0, 5.0, 9.0, 13.0, 17.0, 21.0]) {
            assert!((got - want).abs() < 1e-9, "{xs:?}");
        }
        assert!(fin.waypoints.iter().all(|p| p.1 == 2.0));
        // Ten cells is not a multiple of 4: the walk stops at 9, the drag ended at 11.
        t.way_drag_begin(WayDragOptions { spacing: Some(4.0), ..plain_opts(ManualWayType::Road) }, 24, 16).unwrap();
        t.way_drag_sample(1.0, 2.0, 0).unwrap();
        t.way_drag_sample(11.0, 2.0, 1).unwrap();
        let fin = t.way_drag_finish().unwrap();
        let xs: Vec<f64> = fin.waypoints.iter().map(|p| p.0).collect();
        assert_eq!(xs.len(), 4, "{xs:?}");
        assert!((xs[2] - 9.0).abs() < 1e-9 && xs[3] == 11.0, "{xs:?}");
        // Pre-change contrast: spacing off is one waypoint per sample, gaps of 10 cells.
        t.way_drag_begin(plain_opts(ManualWayType::Road), 24, 16).unwrap();
        for (i, x) in [1.0, 11.0, 21.0].iter().enumerate() {
            t.way_drag_sample(*x, 2.0, i as u64).unwrap();
        }
        assert_eq!(t.way_drag_finish().unwrap().waypoints.len(), 3);
    }

    /// Protects: a pointer held still (a repeated identical sample) adds no waypoint, a non-finite sample is ignored whole, and sampling with no drag open reports that (`None`) rather than inventing a count.
    #[test]
    fn repeated_and_non_finite_samples_add_no_waypoint() {
        let mut t = InfraTools::new();
        assert_eq!(t.way_drag_sample(1.0, 1.0, 0), None, "no drag open");
        t.way_drag_begin(plain_opts(ManualWayType::Road), 24, 16).unwrap();
        assert_eq!(t.way_drag_sample(5.0, 5.0, 0), Some(1));
        assert_eq!(t.way_drag_sample(5.0, 5.0, 1), Some(1));
        assert_eq!(t.way_drag_sample(f64::NAN, 5.0, 2), Some(1));
        assert_eq!(t.way_drag_sample(6.0, 5.0, 3), Some(2));
    }

    /// Protects: the byte-identity acceptance: with spacing, spline, guide and symmetry all off, a routed drag commits exactly the `ManualWay` (points, breaks, km) and unreachable count a click-by-click way over the same waypoints does.
    #[test]
    fn a_plain_routed_drag_is_byte_identical_to_the_click_path() {
        let (field, wb) = route_fixture();
        let ctx = route_ctx(&field, &wb, &[]);
        let wps = [(12.0, 2.0), (18.0, 8.0), (22.0, 13.0)];
        let mut clicked = InfraTools::new();
        clicked.way_begin(ManualWayType::Road);
        for p in wps {
            clicked.way_append_point(p.0, p.1);
        }
        let (_, click_unreachable) = clicked.way_commit(&ctx).unwrap();
        let mut dragged = InfraTools::new();
        let c = run_drag(&mut dragged, plain_opts(ManualWayType::Road), &wps, &ctx).unwrap();
        assert_eq!((c.first, c.count, c.unreachable_legs, c.relaxed_legs), (0, 1, click_unreachable, 0));
        assert_eq!(dragged.ways, clicked.ways, "same pts, brks, km, type, name");
        assert!(dragged.ways[0].pts.len() > 3, "premise: the way was routed, not the three points");
    }

    /// Protects: freehand commits exactly the rasterised path, never a routed one: the literal cells and km, no breaks, and a route over the same waypoints is a different polyline.
    #[test]
    fn freehand_commits_exactly_the_rasterised_drag() {
        let (field, wb) = route_fixture();
        let ctx = route_ctx(&field, &wb, &[]);
        let mut t = InfraTools::new();
        let opts = WayDragOptions { mode: WayDragMode::Freehand, ..plain_opts(ManualWayType::Road) };
        let c = run_drag(&mut t, opts, &[(12.0, 2.0), (15.0, 3.0)], &ctx).unwrap();
        assert_eq!((c.first, c.count, c.unreachable_legs), (0, 1, 0));
        let w = &t.ways[0];
        assert_eq!(w.pts, vec![(12.0, 2.0), (13.0, 2.0), (14.0, 3.0), (15.0, 3.0)]);
        assert!(w.brks.is_empty() && !w.sea && w.way_type == ManualWayType::Road);
        // 1 + sqrt(2) + 1 cells at 240 km / 24 cells = 10 km per cell.
        assert!((w.km - 34.142_135_623_730_95).abs() < 1e-9, "{}", w.km);
        let mut routed = InfraTools::new();
        run_drag(&mut routed, plain_opts(ManualWayType::Road), &[(12.0, 2.0), (15.0, 3.0)], &ctx).unwrap();
        assert_ne!(routed.ways[0].pts, w.pts, "premise: routing smooths this leg, freehand does not");
    }

    /// Protects: freehand's water rule: a land way over ocean cells is refused whole with the blocked count and first cell, a lake counts as water too, a sea lane may not cross land, a sea lane over water is accepted and flagged `sea`, and a refusal pushes nothing.
    #[test]
    fn freehand_refuses_water_for_land_ways_and_land_for_sea_lanes() {
        let (field, wb) = lake_fixture();
        let ctx = route_ctx(&field, &wb, &[]);
        let free = |wt| WayDragOptions { mode: WayDragMode::Freehand, ..plain_opts(wt) };
        let mut t = InfraTools::new();
        // Ocean is x < 10: cells 9 and 8 of (12,2)..(8,2) are water.
        let r = run_drag(&mut t, free(ManualWayType::Road), &[(12.0, 2.0), (8.0, 2.0)], &ctx);
        assert_eq!(r, Err(WayDragRefusal::Water { copy: 0, blocked_cells: 2, first: (9, 2) }));
        // The lake (wb == 2) is water: a straight run along y = 8 crosses x 15..=17.
        let r = run_drag(&mut t, free(ManualWayType::Track), &[(12.0, 8.0), (20.0, 8.0)], &ctx);
        assert_eq!(r, Err(WayDragRefusal::Water { copy: 0, blocked_cells: 3, first: (15, 8) }));
        assert!(t.ways.is_empty(), "a refusal pushes nothing");
        // A sea lane must stay on water: land cells x >= 10 of (2,2)..(12,2) are blocked (10, 11, 12).
        let r = run_drag(&mut t, free(ManualWayType::SeaLane), &[(2.0, 2.0), (12.0, 2.0)], &ctx);
        assert_eq!(r, Err(WayDragRefusal::Water { copy: 0, blocked_cells: 3, first: (10, 2) }));
        let c = run_drag(&mut t, free(ManualWayType::SeaLane), &[(2.0, 2.0), (8.0, 2.0)], &ctx).unwrap();
        assert_eq!(c.count, 1);
        assert!(t.ways[0].sea && t.ways[0].way_type == ManualWayType::SeaLane);
    }

    /// Protects: `freehand_cell_blocked`'s truth table, so each water class and each way kind keeps its rule: land (0) blocks only sea lanes; ocean (1) and lake (2) block only land ways.
    #[test]
    fn freehand_cell_blocked_truth_table() {
        assert!(!freehand_cell_blocked(0, false) && freehand_cell_blocked(0, true));
        assert!(freehand_cell_blocked(1, false) && !freehand_cell_blocked(1, true));
        assert!(freehand_cell_blocked(2, false) && !freehand_cell_blocked(2, true));
    }

    /// Protects: the degenerate and range refusals: one control point (a click), two points in one cell, and a waypoint off the map each refuse with their own reason and push nothing; a refused drag is consumed.
    #[test]
    fn too_few_degenerate_and_out_of_grid_drags_are_refused() {
        let (field, wb) = route_fixture();
        let ctx = route_ctx(&field, &wb, &[]);
        let mut t = InfraTools::new();
        assert_eq!(run_drag(&mut t, plain_opts(ManualWayType::Road), &[(12.0, 2.0)], &ctx), Err(WayDragRefusal::TooFewWaypoints(1)));
        let free = WayDragOptions { mode: WayDragMode::Freehand, ..plain_opts(ManualWayType::Road) };
        assert_eq!(run_drag(&mut t, free, &[(12.1, 2.1), (12.2, 2.2)], &ctx), Err(WayDragRefusal::Degenerate { copy: 0 }));
        assert_eq!(
            run_drag(&mut t, plain_opts(ManualWayType::Road), &[(12.0, 2.0), (24.0, 2.0)], &ctx),
            Err(WayDragRefusal::OutOfGrid { copy: 0, index: 1 })
        );
        assert!(t.ways.is_empty());
        assert!(t.way_drag_finish().is_none(), "every refused drag was consumed");
    }

    /// Protects: symmetry mirrors the control points exactly: literal reflections for a vertical mirror about x = 16 and for dual (the vertical mirror, the horizontal mirror about y = 8, and their composition), source first.
    #[test]
    fn symmetry_mirrors_waypoints_to_literal_reflections() {
        let sym = |mode| StrokeSymmetry { mode, centre: Some((16.0, 8.0)), angle: 0.0 };
        let fin = |mode| FinishedDrag {
            opts: WayDragOptions { symmetry: sym(mode), ..plain_opts(ManualWayType::Road) },
            waypoints: vec![(12.0, 2.0), (15.0, 3.0)],
        };
        let v = fin(cartalith_spatial::stroke::Symmetry::Vertical).copies(24, 16).unwrap();
        assert_eq!(v, vec![vec![(12.0, 2.0), (15.0, 3.0)], vec![(20.0, 2.0), (17.0, 3.0)]]);
        let d = fin(cartalith_spatial::stroke::Symmetry::Dual).copies(24, 16).unwrap();
        assert_eq!(d.len(), 4);
        assert_eq!(d[0], vec![(12.0, 2.0), (15.0, 3.0)]);
        for want in [vec![(20.0, 2.0), (17.0, 3.0)], vec![(12.0, 14.0), (15.0, 13.0)], vec![(20.0, 14.0), (17.0, 13.0)]] {
            assert!(d.contains(&want), "missing {want:?} in {d:?}");
        }
        let none = fin(cartalith_spatial::stroke::Symmetry::None).copies(24, 16).unwrap();
        assert_eq!(none, vec![vec![(12.0, 2.0), (15.0, 3.0)]]);
    }

    /// Protects: a mirrored drag is one commit and one undo: two freehand ways at literal cells (the second the x = 16 reflection), one `way_drag_undo` removes both, a second undo removes nothing, and a drag whose mirror leaves the map is refused whole (no half-drawn symmetric pair).
    #[test]
    fn a_mirrored_drag_commits_two_ways_and_one_undo_removes_both() {
        let (field, wb) = route_fixture();
        let ctx = route_ctx(&field, &wb, &[]);
        let mut t = InfraTools::new();
        let opts = WayDragOptions {
            mode: WayDragMode::Freehand,
            symmetry: StrokeSymmetry { mode: cartalith_spatial::stroke::Symmetry::Vertical, centre: Some((16.0, 8.0)), angle: 0.0 },
            ..plain_opts(ManualWayType::Road)
        };
        let c = run_drag(&mut t, opts, &[(12.0, 2.0), (15.0, 3.0)], &ctx).unwrap();
        assert_eq!((c.first, c.count), (0, 2));
        assert_eq!(t.ways[0].pts, vec![(12.0, 2.0), (13.0, 2.0), (14.0, 3.0), (15.0, 3.0)]);
        assert_eq!(t.ways[1].pts, vec![(20.0, 2.0), (19.0, 2.0), (18.0, 3.0), (17.0, 3.0)]);
        assert_eq!(t.way_drag_undo(), 2);
        assert!(t.ways.is_empty());
        assert_eq!(t.way_drag_undo(), 0, "single level");
        // Centre x = 2 reflects x = 12 to x = -8: off the map, so nothing is committed.
        let off = WayDragOptions {
            symmetry: StrokeSymmetry { mode: cartalith_spatial::stroke::Symmetry::Vertical, centre: Some((2.0, 8.0)), angle: 0.0 },
            ..opts
        };
        assert_eq!(run_drag(&mut t, off, &[(12.0, 2.0), (15.0, 3.0)], &ctx), Err(WayDragRefusal::OutOfGrid { copy: 1, index: 0 }));
        assert!(t.ways.is_empty());
    }

    /// Protects: undo's staleness guard: after the drag, any change to the way list (here one more committed way) makes `way_drag_undo` do nothing, so it cannot delete a way the drag never made.
    #[test]
    fn drag_undo_refuses_when_the_way_list_has_moved() {
        let (field, wb) = route_fixture();
        let ctx = route_ctx(&field, &wb, &[]);
        let mut t = InfraTools::new();
        let free = WayDragOptions { mode: WayDragMode::Freehand, ..plain_opts(ManualWayType::Road) };
        run_drag(&mut t, free, &[(12.0, 2.0), (15.0, 3.0)], &ctx).unwrap();
        t.way_begin(ManualWayType::Track);
        t.way_append_point(12.0, 6.0);
        t.way_append_point(20.0, 6.0);
        t.way_commit(&ctx).unwrap();
        assert_eq!(t.way_drag_undo(), 0);
        assert_eq!(t.ways.len(), 2, "nothing was removed");
        // Same length, different content (a way renamed since): still refused. The length check alone would let this through.
        let mut r = InfraTools::new();
        run_drag(&mut r, WayDragOptions { mode: WayDragMode::Freehand, ..plain_opts(ManualWayType::Road) }, &[(12.0, 2.0), (15.0, 3.0)], &ctx).unwrap();
        r.ways[0].name = "renamed".to_string();
        assert_eq!(r.way_drag_undo(), 0);
        assert_eq!(r.ways.len(), 1);
    }

    /// Protects: determinism: the same options and samples give equal ways on two independent editors, for a smoothed, guided, mirrored routed drag (every ET-D4 stage on).
    #[test]
    fn a_drag_is_deterministic() {
        let (field, wb) = lake_fixture();
        let ctx = route_ctx(&field, &wb, &[]);
        let opts = WayDragOptions {
            seed: 7,
            spacing: Some(3.0),
            symmetry: StrokeSymmetry { mode: cartalith_spatial::stroke::Symmetry::Vertical, centre: Some((16.0, 8.0)), angle: 0.0 },
            ..WayDragOptions::new(ManualWayType::Road)
        };
        let pts = [(11.0, 4.0), (14.0, 6.0), (18.0, 5.0), (21.0, 3.0)];
        let (mut a, mut b) = (InfraTools::new(), InfraTools::new());
        let ca = run_drag(&mut a, opts, &pts, &ctx).unwrap();
        let cb = run_drag(&mut b, opts, &pts, &ctx).unwrap();
        assert_eq!(ca, cb);
        assert_eq!(a.ways, b.ways);
        assert_eq!(a.ways.len(), 2);
    }

    /// Protects: only the first and last waypoint are snapped (and only in routed mode): a snap that shifts every point by one cell moves exactly those two, and freehand moves none.
    #[test]
    fn snap_endpoints_moves_only_the_ends_and_only_when_routed() {
        let mk = |mode| FinishedDrag {
            opts: WayDragOptions { mode, ..plain_opts(ManualWayType::Road) },
            waypoints: vec![(1.0, 1.0), (2.0, 1.0), (3.0, 1.0), (4.0, 1.0)],
        };
        let mut d = mk(WayDragMode::Routed);
        d.snap_endpoints(&|x, y| (x + 10.0, y));
        assert_eq!(d.waypoints, vec![(11.0, 1.0), (2.0, 1.0), (3.0, 1.0), (14.0, 1.0)]);
        let mut f = mk(WayDragMode::Freehand);
        f.snap_endpoints(&|x, y| (x + 10.0, y));
        assert_eq!(f.waypoints, mk(WayDragMode::Freehand).waypoints);
    }

    /// Protects: option validation and its all-or-nothing rule: every out-of-range option refuses, and a refused begin leaves a drag already open alone.
    #[test]
    fn begin_refuses_bad_options_and_leaves_an_open_drag_alone() {
        let mut t = InfraTools::new();
        t.way_drag_begin(plain_opts(ManualWayType::Road), 24, 16).unwrap();
        t.way_drag_sample(3.0, 3.0, 0).unwrap();
        let base = plain_opts(ManualWayType::Road);
        let bad = [
            WayDragOptions { spacing: Some(0.0), ..base },
            WayDragOptions { spacing: Some(11.0), ..base },
            WayDragOptions { spacing: Some(f64::NAN), ..base },
            WayDragOptions { smoothing: -1.0, ..base },
            WayDragOptions { smooth_spacing: 0.1, ..base },
            WayDragOptions { smooth_spacing: 9.0, ..base },
            WayDragOptions { guide: Some(GuideOptions { corridor: 0.0, pull: 1.0 }), ..base },
            WayDragOptions { guide: Some(GuideOptions { corridor: 65.0, pull: 1.0 }), ..base },
            WayDragOptions { guide: Some(GuideOptions { corridor: 3.0, pull: -1.0 }), ..base },
            WayDragOptions { guide: Some(GuideOptions { corridor: 3.0, pull: f64::INFINITY }), ..base },
            WayDragOptions { symmetry: StrokeSymmetry { mode: cartalith_spatial::stroke::Symmetry::Radial(0), ..base.symmetry }, ..base },
            WayDragOptions { symmetry: StrokeSymmetry { mode: cartalith_spatial::stroke::Symmetry::Radial(65), ..base.symmetry }, ..base },
            WayDragOptions { symmetry: StrokeSymmetry { centre: Some((f64::NAN, 1.0)), ..base.symmetry }, ..base },
        ];
        for b in bad {
            assert!(t.way_drag_begin(b, 24, 16).is_err(), "{b:?}");
        }
        assert_eq!(t.way_drag_sample(4.0, 3.0, 1), Some(2), "the open drag survived every refused begin");
        assert_eq!(WayDragMode::parse("routed"), Some(WayDragMode::Routed));
        assert_eq!(WayDragMode::parse("freehand"), Some(WayDragMode::Freehand));
        assert_eq!(WayDragMode::parse("Freehand"), None);
        assert_eq!(WayDragMode::parse(WayDragMode::Freehand.key()), Some(WayDragMode::Freehand));
    }

    /// Protects: the spline passes through every control point to 1e-9 (the owner's "passing through every control point"), and a spline on two points is the straight chord.
    #[test]
    fn the_spline_passes_through_every_control_point() {
        let ctrl = [(1.0, 1.0), (6.0, 4.0), (7.0, 9.0), (15.0, 8.0), (20.0, 2.0)];
        let line = cartalith_civ::civ_catmull_rom_sample(&ctrl, 0.5);
        for c in ctrl {
            assert!(line.iter().any(|p| (p.0 - c.0).hypot(p.1 - c.1) < 1e-9), "{c:?} missing");
        }
        assert_eq!(line[0], ctrl[0]);
        let two = cartalith_civ::civ_catmull_rom_sample(&[(1.0, 1.0), (9.0, 5.0)], 1.0);
        assert!(max_dev(&two, &[(1.0, 1.0), (9.0, 5.0)]) < 1e-9);
    }

    /// Test helper: the textbook UNIFORM Catmull-Rom (Hermite form, tangent `(p[i+1] - p[i-1]) / 2`, phantom ends by reflection) sampled `n` times per segment; the contrast `the_centripetal_spline_does_not_overshoot...` measures against.
    fn uniform_catmull_rom(p: &[(f64, f64)], n: usize) -> Vec<(f64, f64)> {
        let mut q = vec![(2.0 * p[0].0 - p[1].0, 2.0 * p[0].1 - p[1].1)];
        q.extend_from_slice(p);
        let l = p.len();
        q.push((2.0 * p[l - 1].0 - p[l - 2].0, 2.0 * p[l - 1].1 - p[l - 2].1));
        let mut out = Vec::new();
        for s in 0..l - 1 {
            let (p0, p1, p2, p3) = (q[s], q[s + 1], q[s + 2], q[s + 3]);
            for i in 0..=n {
                let t = i as f64 / n as f64;
                let (t2, t3) = (t * t, t * t * t);
                let f = |a: f64, b: f64, c: f64, d: f64| 0.5 * ((2.0 * b) + (-a + c) * t + (2.0 * a - 5.0 * b + 4.0 * c - d) * t2 + (-a + 3.0 * b - 3.0 * c + d) * t3);
                out.push((f(p0.0, p1.0, p2.0, p3.0), f(p0.1, p1.1, p2.1, p3.1)));
            }
        }
        out
    }

    /// Protects: why the spline is centripetal (alpha 0.5): on an unevenly spaced drag (a 1-cell hop between two 10-cell strides) the uniform form swings more than a quarter cell past the short segment's far end, and the centripetal curve stays within a tenth of a cell of it. Asserting BOTH is what makes it fail if the parameterisation changes to uniform (alpha 0).
    #[test]
    fn the_centripetal_spline_does_not_overshoot_where_the_uniform_one_does() {
        let ctrl = [(0.0, 0.0), (10.0, 0.0), (10.2, 1.0), (20.0, 1.0)];
        // The short segment is control 1 -> control 2; its x never legitimately exceeds 10.2.
        let uniform_max_x = uniform_catmull_rom(&ctrl, 200)[201..402].iter().map(|p| p.0).fold(f64::MIN, f64::max);
        assert!(uniform_max_x > 10.2 + 0.25, "premise: the uniform form overshoots; got {uniform_max_x}");
        let cr = cartalith_civ::civ_catmull_rom_sample(&ctrl, 0.05);
        let i1 = cr.iter().position(|p| *p == ctrl[1]).unwrap();
        let i2 = cr.iter().position(|p| *p == ctrl[2]).unwrap();
        let centripetal_max_x = cr[i1..=i2].iter().map(|p| p.0).fold(f64::MIN, f64::max);
        assert!(centripetal_max_x < 10.2 + 0.1, "centripetal max x {centripetal_max_x}");
    }

    /// Protects: smoothed freehand: the spline's cells are a connected 8-neighbour chain that visits every control point's cell, differ from the raw polygon's cells, and the raw path is still available with `smooth: false` (literal pin of the raw cells).
    #[test]
    fn smoothed_freehand_is_connected_through_the_controls_and_raw_is_still_available() {
        let (field, wb) = route_fixture();
        let ctx = route_ctx(&field, &wb, &[]);
        let ctrl = [(12.0, 2.0), (16.0, 7.0), (20.0, 3.0), (22.0, 12.0)];
        let mut smooth = InfraTools::new();
        let so = WayDragOptions { mode: WayDragMode::Freehand, smooth: true, ..plain_opts(ManualWayType::Road) };
        run_drag(&mut smooth, so, &ctrl, &ctx).unwrap();
        let sp = &smooth.ways[0].pts;
        for w in sp.windows(2) {
            assert!((w[1].0 - w[0].0).abs() <= 1.0 && (w[1].1 - w[0].1).abs() <= 1.0 && w[0] != w[1], "{w:?}");
        }
        for c in ctrl {
            assert!(sp.contains(&c), "control cell {c:?} not visited");
        }
        let mut raw = InfraTools::new();
        let ro = WayDragOptions { mode: WayDragMode::Freehand, smooth: false, ..plain_opts(ManualWayType::Road) };
        run_drag(&mut raw, ro, &ctrl, &ctx).unwrap();
        assert_eq!(raw.ways[0].pts, rasterise_cells(&ctrl, 24, 16).iter().map(|&(x, y)| (x as f64, y as f64)).collect::<Vec<_>>());
        assert_ne!(raw.ways[0].pts, *sp, "premise: smoothing changes the path");
    }

    /// Protects: symmetry is applied to the control points BEFORE smoothing: with a smoothed freehand vertical mirror about x = 16 the mirrored way is the cell-for-cell reflection of the source (every source cell (x, y) has (32 - x, y) in the copy).
    #[test]
    fn a_smoothed_mirror_is_the_reflection_of_the_smoothed_source() {
        let (field, wb) = route_fixture();
        let ctx = route_ctx(&field, &wb, &[]);
        let mut t = InfraTools::new();
        let opts = WayDragOptions {
            mode: WayDragMode::Freehand,
            smooth: true,
            symmetry: StrokeSymmetry { mode: cartalith_spatial::stroke::Symmetry::Vertical, centre: Some((16.0, 8.0)), angle: 0.0 },
            ..plain_opts(ManualWayType::Road)
        };
        run_drag(&mut t, opts, &[(11.0, 2.0), (14.0, 6.0), (13.0, 11.0)], &ctx).unwrap();
        let (a, b) = (&t.ways[0].pts, &t.ways[1].pts);
        assert_eq!(a.len(), b.len());
        for p in a {
            assert!(b.contains(&(32.0 - p.0, p.1)), "mirror of {p:?} missing");
        }
    }

    /// Protects: guided routing on open flat ground follows the smoothed drag: the committed road stays within one cell of the spline everywhere (the owner's "follows the drag closely"), and ends on the exact waypoints.
    #[test]
    fn a_guided_route_on_flat_ground_stays_within_a_cell_of_the_spline() {
        let (field, wb) = route_fixture();
        let ctx = route_ctx(&field, &wb, &[]);
        let ctrl = [(11.0, 3.0), (15.0, 6.0), (19.0, 6.0), (22.0, 11.0)];
        let mut t = InfraTools::new();
        let c = run_drag(&mut t, WayDragOptions::new(ManualWayType::Road), &ctrl, &ctx).unwrap();
        assert_eq!((c.unreachable_legs, c.relaxed_legs), (0, 0));
        let pts = &t.ways[0].pts;
        let spline = cartalith_civ::civ_catmull_rom_sample(&ctrl, DEFAULT_SMOOTH_SPACING);
        let dev = max_dev(pts, &spline);
        assert!(dev <= 1.0 + 1e-9, "max deviation {dev}");
        assert_eq!((pts[0], pts[pts.len() - 1]), (ctrl[0], ctrl[3]));
    }

    /// Protects: the small-deviation acceptance: a 3x3 lake sits on the drag; the guided road leaves the spline by at most the corridor half-width plus the one-cell drawn-position rounding, really does go round (more than 1.5 cells off at its widest), never enters the lake, and rejoins the spline near the far end.
    #[test]
    fn a_guided_route_goes_round_an_obstacle_and_rejoins() {
        let (field, wb) = lake_fixture();
        let ctx = route_ctx(&field, &wb, &[]);
        let ctrl = [(11.0, 8.0), (22.0, 8.0)];
        let g = GuideOptions::default();
        let mut t = InfraTools::new();
        let c = run_drag(&mut t, WayDragOptions::new(ManualWayType::Road), &ctrl, &ctx).unwrap();
        assert_eq!((c.unreachable_legs, c.relaxed_legs), (0, 0), "the corridor was wide enough");
        let pts = &t.ways[0].pts;
        let spline = cartalith_civ::civ_catmull_rom_sample(&ctrl, DEFAULT_SMOOTH_SPACING);
        let dev = max_dev(pts, &spline);
        assert!(dev <= g.corridor + 1.0 + 1e-9, "left the spline by {dev}");
        assert!(dev > 1.5, "premise: it went round the lake; deviation only {dev}");
        for p in pts {
            let (cx, cy) = (p.0.round() as usize, p.1.round() as usize);
            assert_eq!(wb[cy * 24 + cx], 0, "{p:?} is on water");
        }
        let tail: Vec<_> = pts.iter().filter(|p| p.0 >= 20.0).collect();
        assert!(!tail.is_empty() && tail.iter().all(|p| (p.1 - 8.0).abs() <= 1.0), "did not rejoin: {tail:?}");
    }

    /// Protects: the corridor really restricts the route: a lake taller than the corridor allows cannot be rounded inside it, so the leg is re-routed unguided and REPORTED (`relaxed_legs`), and the way is still valid land (not a straight line over water).
    #[test]
    fn an_obstacle_wider_than_the_corridor_relaxes_the_leg_and_says_so() {
        let (field, mut wb) = route_fixture();
        // A wall of lake across the whole drag column x 15..=17, y 1..=14: the only way round is far outside a 3-cell corridor.
        let mut field = field;
        for y in 1..=14 {
            for x in 15..=17 {
                field[y * 24 + x] = 0.30;
                wb[y * 24 + x] = 2;
            }
        }
        let ctx = route_ctx(&field, &wb, &[]);
        let mut t = InfraTools::new();
        let c = run_drag(&mut t, WayDragOptions::new(ManualWayType::Road), &[(11.0, 8.0), (22.0, 8.0)], &ctx).unwrap();
        assert!(c.relaxed_legs >= 1, "{c:?}");
        let dev = max_dev(&t.ways[0].pts, &cartalith_civ::civ_catmull_rom_sample(&[(11.0, 8.0), (22.0, 8.0)], 1.0));
        assert!(dev > DEFAULT_CORRIDOR + 1.0, "premise: the relaxed road left the corridor ({dev})");
        for p in &t.ways[0].pts {
            // `civ_smooth_path` can round a point onto the row/column just off the grid; clamp as its own validity test does.
            let (cx, cy) = ((p.0.round() as usize).min(23), (p.1.round() as usize).min(15));
            assert_eq!(wb[cy * 24 + cx], 0, "{p:?} is on water");
        }
    }

    /// Protects: the pull weight: an existing road (x0.25 cost) runs 3 cells below the drag. With pull 0 the guided route abandons the drag to ride it (more than 2 cells off); with pull 100 it stays within a cell of the drag. Without any guide it rides the road too, which is what the pull is for.
    #[test]
    fn the_pull_weight_keeps_the_road_on_the_drag() {
        let (field, wb) = route_fixture();
        let road_pts: Vec<(f64, f64)> = (11..=22).map(|x| (x as f64, 11.0)).collect();
        let road = [WayRef { pts: &road_pts, brks: &[], sea: false, hidden: false }];
        let ctx = route_ctx(&field, &wb, &road);
        let ctrl = [(11.0, 8.0), (22.0, 8.0)];
        let spline = cartalith_civ::civ_catmull_rom_sample(&ctrl, 1.0);
        let dev_for = |guide: Option<GuideOptions>| {
            let mut t = InfraTools::new();
            run_drag(&mut t, WayDragOptions { guide, ..WayDragOptions::new(ManualWayType::Road) }, &ctrl, &ctx).unwrap();
            max_dev(&t.ways[0].pts, &spline)
        };
        let off = dev_for(None);
        let loose = dev_for(Some(GuideOptions { corridor: 5.0, pull: 0.0 }));
        let tight = dev_for(Some(GuideOptions { corridor: 5.0, pull: 100.0 }));
        assert!(off > 2.0, "premise: an unguided route rides the road; {off}");
        assert!(loose > 2.0, "pull 0 should still ride the road; {loose}");
        assert!(tight <= 1.0 + 1e-9, "pull 100 should hold the drag; {tight}");
    }
}
