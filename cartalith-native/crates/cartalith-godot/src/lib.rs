//! Boundary layer between Godot and the engine crates (ARCHITECTURE.md).
//!
//! `WalkingSkeleton` is the Phase 0 proof that a gdext-backed class loads in
//! the Godot editor and survives a Windows/Android export. `WorldGen`
//! (below) is Phase 1's real API surface (`ARCHITECTURE.md`: "a `WorldGen`
//! with `generate(seed, width_km, resolution)` and accessors returning
//! fields") — the only place in this crate (and the only crate in the
//! workspace) that touches `cartalith_engine::WorldState` and a Godot type
//! in the same function, exactly the boundary `ARCHITECTURE.md` describes:
//! "Rust never touches the scene tree... only `cartalith-godot` may depend
//! on `gdext`."

use cartalith_engine::staleness::{pipeline_stage_graph, recompute_stale, PipelineStage};
use cartalith_engine::{generate_terrain, WorldParams, WorldStructureParams};
use godot::classes::image::Format;
use godot::classes::{IRefCounted, INode, Image, ImageTexture, Node, RefCounted};
use godot::init::{ExtensionLibrary, gdextension};
use godot::prelude::*;

mod asset_bridge;
mod bake_bridge;
mod civ_military_bridge;
mod civ_roster_bridge;
mod civ_tools_bridge;
mod civ_trade_bridge;
mod export_raster;
mod geojson_bridge;
mod icon_bridge;
mod infra_tools_bridge;
mod journey_bridge;
mod label_bridge;
mod lod_bridge;
mod measure_bridge;
mod pack;
mod paint_bridge;
mod params;
mod render;
mod sample_bridge;
mod sculpt_bridge;
mod timeline_bridge;
mod travel_bridge;
mod undo;
mod urban_bridge;
mod vault_bridge;
use cartalith_terrain::sculpt::{Feature, FeatureParams, FreehandMode, SculptStamp, SCULPT_PRESETS};
use render::{QualityTier, RenderCtx, SplatTextures, TerrainAppearance};
use rayon::prelude::*;

struct CartalithExtension;

#[gdextension]
unsafe impl ExtensionLibrary for CartalithExtension {}

/// Placeholder GDExtension class for the Phase 0 walking skeleton.
#[derive(GodotClass)]
#[class(base=Node)]
struct WalkingSkeleton {
    base: Base<Node>,
}

#[godot_api]
impl INode for WalkingSkeleton {
    fn init(base: Base<Node>) -> Self {
        Self { base }
    }

    fn ready(&mut self) {
        godot_print!("cartalith-godot: WalkingSkeleton ready (Phase 0)");
    }
}

#[godot_api]
impl WalkingSkeleton {
    /// Round-trips a value through Rust so GDScript can confirm the
    /// extension is actually loaded, not just present on disk.
    #[func]
    fn ping(&self) -> GString {
        GString::from("cartalith-godot: pong")
    }
}

/// Either a fresh `generate_terrain()` run or a loaded save
/// (`cartalith_io::load_save`, `MVP_SCOPE.md` point 12/criterion 7). A
/// loaded save only carries the terrain fields `SAVEFILE_COMPAT.md`
/// documents (no plate/stress/flexure substrate — those aren't part of the
/// save format), so this is a separate variant rather than trying to
/// backfill a full `WorldState`; `build_color_texture` reads through
/// `WorldGen`'s own small accessor methods below so it doesn't need to
/// know which source is active.
enum WorldSource {
    Generated(Box<cartalith_engine::WorldState>),
    Loaded(Box<cartalith_io::SaveData>),
}

/// Phase 2's civilisation layer (`cartalith-civ`, `PHASE2_SCOPE.md`
/// milestones 1-11): settlements (placed, faction-assigned, named,
/// populated) and the road network connecting them. Computed once,
/// automatically, right after a fresh `generate()`/`generate_world_structure()`
/// call — `Loaded` saves carry none of the substrate fields (`crust_field`,
/// `boundary_type`, `shear_field`, `age_field`) this pipeline needs
/// (`SAVEFILE_COMPAT.md` doesn't store them), so civ data is only ever
/// real for a freshly generated world, never a loaded one. `None` before
/// the first successful `generate()`, or if generation produced zero
/// settlement candidates (a legitimate empty-map outcome, not an error).
struct CivData {
    settlements: Vec<cartalith_civ::NamedSettlement>,
    /// Consolidated, classified, Catmull-Rom-smoothed, named road
    /// polylines (`cartalith_civ::civ_consolidate_and_smooth_ways`, Phase
    /// 2 milestone 14) -- the real auto-populate road network
    /// (`civ_hierarchical_network_topology`, milestone 12) run through its
    /// own reference consolidation tail, not the earlier placeholder
    /// (`build_road_network`, the *manual*-tool algorithm this pipeline
    /// used as a stand-in before milestone 12/14 existed to replace it).
    ways: Vec<cartalith_civ::Way>,
    /// Sea-lane routes between port (coastal) settlements
    /// (`cartalith_civ::civ_sea_routes`, Phase 2 milestone 13,
    /// `_civMstRoutes(ports,true)`) -- a separate, leaner shape than `Way`
    /// (no classification/hidden-way flag/endpoint indices), pushed
    /// straight onto the reference's `civWays` rather than going through
    /// `civ_consolidate_and_smooth_ways`'s consolidation tail, so it's kept
    /// as its own field/getter rather than merged into `ways`. Empty
    /// whenever fewer than 2 coastal settlements exist (`civ_sea_routes`'s
    /// own `n < 2` early return) -- a legitimate empty-map outcome, not an
    /// error.
    sea_routes: Vec<cartalith_civ::SeaRoute>,
    /// `cartalith_civ::assign_territory`'s per-cell output (Phase 2
    /// milestone 10, `DECISIONS.md` §7b -- cost-distance Voronoi from
    /// capitals, population-weighted, no JS reference to match since the
    /// reference has no algorithmic territory generation at all). `0` =
    /// unowned (water, or unreachable from any capital). Always computed
    /// (cheap: one Dijkstra per capital, already-real `cost` field reused
    /// from the road network above) -- unlike `villages`, this isn't
    /// gated, since there's no reference default to match and no reason
    /// to withhold it.
    territory: Vec<i32>,
    /// `cartalith_civ::civ_generate_provinces`'s per-cell output (reference
    /// `_civGenerateProvinces`, ported once `territory` above gave it a
    /// real programmatic input to subdivide -- `PHASE2_SCOPE.md`'s earlier
    /// "territory/provinces is a dead end here" note was about the
    /// reference's own missing `civTerritory` producer, not about this
    /// port's `assign_territory`, which supplies the exact same per-cell
    /// shape). `0` = no province (unowned territory, or a faction that owns
    /// territory here but placed no settlement to seed one -- see
    /// `civ_generate_provinces`'s own doc comment for why that's a real,
    /// non-error outcome). Data only this pass -- no Godot-side rendering
    /// wired in yet, deliberately left for a dedicated UI/UX pass rather
    /// than improvised here (see this field's own CHANGELOG entry).
    provinces: Vec<i32>,
    /// Province metadata (id/faction/name/seed settlement index) parallel
    /// to `provinces` above's cell ids.
    province_list: Vec<cartalith_civ::Province>,
    /// Addressable landmasses (`cartalith_civ::civ_continents`,
    /// `MARKDOWN_VAULT_SCOPE.md` milestone 0) — the connected-component
    /// labelling `build_landmass_quality` has always produced and this
    /// pipeline has always discarded, kept this time because the vault
    /// integration needs a third linkable entity beside settlements and
    /// provinces. Metadata only: no per-cell raster, for the memory reason
    /// `civ_continents`' own doc comment gives.
    continents: Vec<cartalith_civ::Continent>,
    /// Per-settlement resource trade balance (`cartalith_civ::
    /// civ_resource_trade_balance`, reference `_civPlaceTrade`'s hinterland
    /// term -- `ECONOMY_SCOPE.md`), same order/index as `settlements`
    /// above. A settlement's own catchment-mean resource profile
    /// (`civ_place_resource_context`, `_civPlaceResourceContext`) against
    /// the world mean (`civ_world_mean_resources`, the one piece of
    /// `_civFactionAggregates` that's genuinely territory-independent).
    /// Requires all 15 `CIV_RESOURCE_KEYS` -- computed here, right after
    /// `settlements` is finalized and *before* `resources`' unused six
    /// fields are freed (see `compute_civilisation`'s own comment on why
    /// that free moved). Does not include `_civFactionAggregates`'s own
    /// territory-based per-faction aggregation (population, tax, the
    /// five-axis "power" heuristic, sector output) -- that remains real,
    /// unstarted future scope, not silently folded in here.
    trade_balances: Vec<cartalith_civ::TradeBalance>,
    /// Why each settlement is where it is (`VISION.md`'s causal chain),
    /// same order/index as `settlements`. Captured inside
    /// `compute_civilisation` while the suitability rasters are still
    /// alive -- see that function's own comment for why this is
    /// per-settlement rather than a general per-cell query.
    explanations: Vec<SettlementExplanation>,
    /// `build_water_bodies`'s classification (0 = land, 1 = ocean, 2 =
    /// lake), kept past `compute_civilisation`'s own return so `civ_tools_
    /// bridge`'s Settlement tool (`WorldGen::civ_drop_settlement`) can call
    /// `civ_drop_place`/the snap-to-water search without recomputing a full
    /// `build_water_bodies` flood fill on every click -- it was already
    /// real, already-computed data this function held anyway.
    water_bodies: Vec<u8>,
    /// `TIMELINE_SCOPE.md` milestone 1's stable-id counter -- the reference's
    /// `_civNextTid`, moved here because `cartalith-civ` is stateless
    /// (`ARCHITECTURE.md`) and this is the one place civ state is actually
    /// mutable. Every `settlements`/`ways` entry above is assigned a real
    /// `tid` (`cartalith_civ::timeline::civ_assign_tid`) right here in
    /// `compute_civilisation`, before this struct is built, so `next_tid`
    /// always starts past every id already in use. `civ_tools_bridge::
    /// drop_settlement` (the Settlement tool's manual-insertion path) draws
    /// from this same counter for anything appended afterward. See
    /// `cartalith_civ::timeline`'s own module doc for the full design
    /// decision (why eager assignment, not the reference's lazy
    /// first-touch).
    next_tid: u64,
    /// `TIMELINE_SCOPE.md` milestone 4's own recorded-year history -- the
    /// reference's `civTimeline` (reference line 14756: `let civTimeline=[]`).
    /// Empty until the first `civ_add_year`/`civ_run_collapse_simulation`
    /// call (the latter is milestone 5's job, not built yet). Every entry's
    /// `settlements`/`ways` are frozen COPIES made at save time
    /// (`cartalith_civ::timeline::civ_snapshot_save`) -- `settlements`/`ways`
    /// above stay the single always-current, always-editable arrays, exactly
    /// as the reference's own `state.places`/`civWays` do (reference lines
    /// 20559-20561). Capped at `TIMELINE_MAX_YEARS` entries -- `TIMELINE_
    /// SCOPE.md` §9's own "Snapshot cap" decision, a deliberate deviation
    /// from the reference's unbounded storage
    /// (`MEMORY_OPTIMIZATION_SCOPE.md`'s existing budget discipline).
    timeline: Vec<cartalith_civ::timeline::TimelineSnapshot>,
    /// `TIMELINE_SCOPE.md` milestone 4's active-year cursor -- the
    /// reference's `civYear` (reference line 14757: `let civYear=0`). `0`
    /// before any year has ever been added, matching the reference's own
    /// init value. No reader yet -- milestone 5's `#[func]` boundary
    /// (`timeline_bridge.rs`) is what will expose it to GDScript; this field
    /// is written by every method below regardless.
    #[allow(dead_code)]
    year: i64,
    /// `TIMELINE_SCOPE.md` milestone 5's own addition -- `currentAgrarianDensity()`'s
    /// per-cell output (`cartalith_civ::timeline::civ_current_agrarian_density`),
    /// computed once here from `carrying_cap`/`water_access`/`biome` (already-live
    /// locals of `compute_civilisation`, the same "it was already real, already-
    /// computed data this function held anyway" reasoning `water_bodies` above was
    /// kept for) and retained so `timeline_bridge::run_collapse_simulation` doesn't
    /// have to re-run the soil/water-access/biome sub-pipeline on every simulate
    /// call. Feeds `cartalith_civ::timeline::civ_settlement_population`'s `dens`
    /// parameter -- the collapse stepper's migration-headroom ceiling and the
    /// recovery stepper's logistic regrowth ceiling both key off it.
    dens: Vec<f32>,
    /// `GUI_GAP_REGISTER.md` CV-07/MS-13, `PARITY_AUDIT.md` §5 items 9/10:
    /// the reference's own mutable `CIV_FACTIONS` plus its five parallel
    /// per-faction arrays, collapsed into one row per index (see
    /// `civ_roster_bridge`'s module doc for why the roster is boundary
    /// state and not `cartalith-civ` state). Seeded to `CIV_FACTION_COUNT`
    /// real factions plus "Unclaimed" at index 0 on every
    /// `compute_civilisation`, then grown/shrunk by `civ_add_faction`/
    /// `civ_remove_faction`.
    ///
    /// **Generation still runs at `CIV_FACTION_COUNT`.** `assign_factions`
    /// is called inside `generate()` with that constant, so adding a
    /// faction yields a real, paintable, assignable, editable id that no
    /// *generated* settlement was seeded into -- which is exactly the
    /// reference's own behaviour: its `_civAddFaction` likewise touches
    /// nothing already placed.
    faction_roster: civ_roster_bridge::FactionRoster,
    /// `PARITY_AUDIT.md` §5 item 3 / `GUI_GAP_REGISTER.md` ED-03: the five
    /// place-editor fields `NamedSettlement` has no room for, keyed by
    /// `tid`. See `civ_roster_bridge`'s module doc for why they sit beside
    /// the settlement rather than on it, and what that costs.
    place_extras: civ_roster_bridge::PlaceExtrasTable,
    /// The `tid`s of the settlements `civ_seed_villages` added, when
    /// `CivOptions::villages` is on (empty otherwise, which is the default).
    ///
    /// Exists for exactly one reason: `WorldGen::recompute_civilisation`
    /// rebuilds the road network from the settlement list, and villages are
    /// **not** network nodes. The reference seeds them *after*
    /// `_civHierarchicalNetwork` has already run, so an auto-populated world
    /// has roads between its 35 placed settlements and none to its 198
    /// villages — and a recompute that fed the whole list back in would jump
    /// to 240 ways on one button press, restructuring the world rather than
    /// catching it up. Measured, on a real 384x288 village-enabled world,
    /// before this set existed.
    ///
    /// Keyed by `tid` rather than by index or by a trailing range because
    /// `civ_delete_settlement` splices and `civ_drop_settlement` appends, so
    /// neither an index nor a range survives a session of editing. Stale
    /// entries (a deleted village's `tid`) are harmless: `tid`s are unique
    /// and monotonic, so a removed one never matches anything again.
    village_tids: std::collections::HashSet<u64>,
}

/// What [`WorldGen::recompute_civilisation`] hands [`compute_civilisation`]
/// to hold fixed. See that function's `keep` parameter.
struct KeptCiv {
    settlements: Vec<cartalith_civ::NamedSettlement>,
    next_tid: u64,
    village_tids: std::collections::HashSet<u64>,
}

/// `TIMELINE_SCOPE.md` §9's own "Snapshot cap" decision: a generous ceiling
/// on recorded timeline years, bounding this struct's own memory use rather
/// than matching the reference's unbounded `civTimeline` growth. Logged here
/// (and in `cartalith-native/docs/CHANGELOG.md`) as a deliberate deviation,
/// not a silent one -- `civ_add_year`'s own doc comment is where it's
/// enforced.
#[allow(dead_code)] // read by `CivData::civ_add_year` below; the lint can't see through `impl` gating
const TIMELINE_MAX_YEARS: usize = 2000;

/// This milestone (`TIMELINE_SCOPE.md` milestone 4) deliberately stops at
/// plain Rust methods over `CivData` -- no `#[func]`/`Variant` surface, per
/// its own scope ("Do NOT start milestone 5"). Every method below is
/// consequently unreachable from anywhere in this crate yet (nothing calls
/// `civ_add_year`/`civ_remove_year`/`civ_year_diff` until milestone 5 wires a
/// `#[func]` to each) -- `#[allow(dead_code)]` on the block disclosing why,
/// rather than scattering it per-method.
#[allow(dead_code)]
impl CivData {
    /// **The** swatch for faction `f` (`1`-based) — the roster's own
    /// identity colour when the user has set one, and
    /// [`faction_rgb_default`]'s palette rule when they have not
    /// (`GUI_GAP_REGISTER.md` **CV-21**).
    ///
    /// One function so the three surfaces that draw a faction — the
    /// territory wash, the Political-control analysis field, and
    /// `get_factions`' swatch for the roster and the banner — cannot
    /// disagree. That "cannot disagree" is not hypothetical: before this,
    /// the Political-control field indexed `FACTION_RGB` with its own
    /// `% len()` wrap while the wash used `faction_rgb`'s no-wrap rule, so
    /// on a seven-faction world the field drew faction 7 in faction 1's
    /// colour and the map did not.
    fn faction_rgb(&self, f: i32) -> (u8, u8, u8) {
        self.faction_roster
            .0
            .get(f.max(0) as usize)
            .and_then(|e| e.color_override)
            .unwrap_or_else(|| faction_rgb_default(f))
    }

    /// `civGotoYear` (reference lines 20615-20617, minus `_civBuildTimelineUI()`
    /// -- UI wiring is milestone 6's job). Sets the active-year cursor and
    /// restores `territory` from that year's recorded snapshot
    /// (`cartalith_civ::timeline::civ_snapshot_load`) -- never touches
    /// `settlements`/`ways`, matching `TIMELINE_SCOPE.md` §7 success
    /// criterion 2.
    fn civ_goto_year(&mut self, year: i64) {
        self.year = year;
        cartalith_civ::timeline::civ_snapshot_load(&self.timeline, year, &mut self.territory);
    }

    /// `civAddYear` (reference lines 20618-20634): if the timeline is empty,
    /// seeds the requested `year` with the LIVE state and jumps to it
    /// (reference's own v0.62 fix, its comment lines 20619-20622 -- avoids
    /// conjuring a phantom "0 AD" entry at the init `civYear=0`). Otherwise,
    /// first snapshots the CURRENTLY ACTIVE year from live state (so it is
    /// never lost -- `TIMELINE_SCOPE.md` §7 success criterion 2), then, if
    /// `year` isn't already recorded, creates a new entry carrying forward
    /// territory/settlements/ways from the nearest EARLIER recorded year (or
    /// empty, if none exists), and jumps to it.
    ///
    /// A no-op past `TIMELINE_MAX_YEARS` recorded years (this port's own
    /// deliberate cap, `TIMELINE_SCOPE.md` §9 -- the reference has no
    /// equivalent limit) -- the currently-active year's live state was
    /// already safely snapshotted above in every case before this check, so
    /// refusing to grow the timeline further never loses data, it just stops
    /// recording new ones.
    fn civ_add_year(&mut self, year: i64) {
        if self.timeline.is_empty() {
            cartalith_civ::timeline::civ_snapshot_save(
                &mut self.timeline,
                year,
                self.territory.clone(),
                self.settlements.clone(),
                self.ways.clone(),
            );
            self.civ_goto_year(year);
            return;
        }
        cartalith_civ::timeline::civ_snapshot_save(
            &mut self.timeline,
            self.year,
            self.territory.clone(),
            self.settlements.clone(),
            self.ways.clone(),
        );
        if self.timeline.iter().any(|s| s.year == year) {
            return;
        }
        if self.timeline.len() >= TIMELINE_MAX_YEARS {
            return;
        }
        let prev = self
            .timeline
            .iter()
            .filter(|s| s.year <= year)
            .max_by_key(|s| s.year);
        let (territory, settlements, ways) = match prev {
            Some(p) => (p.territory.clone(), p.settlements.clone(), p.ways.clone()),
            None => (Vec::new(), Vec::new(), Vec::new()),
        };
        self.timeline
            .push(cartalith_civ::timeline::TimelineSnapshot {
                year,
                territory,
                settlements,
                ways,
            });
        self.timeline.sort_by_key(|s| s.year);
        self.civ_goto_year(year);
    }

    /// `civRemoveYear` (reference lines 20635-20641, minus `_civBuildTimelineUI()`
    /// -- milestone 6). A no-op if `year` was never recorded. Removing the
    /// active year falls back to the earliest remaining one (or year `0` if
    /// none remain, matching the reference's own `next?next.year:0`) --
    /// `TIMELINE_SCOPE.md` §7 success criterion 2. `self.timeline` stays
    /// sorted by construction (every write path above sorts), so `.first()`
    /// is always the earliest remaining year.
    fn civ_remove_year(&mut self, year: i64) {
        let Some(idx) = self.timeline.iter().position(|s| s.year == year) else {
            return;
        };
        self.timeline.remove(idx);
        if self.year == year {
            let next_year = self.timeline.first().map(|s| s.year).unwrap_or(0);
            self.civ_goto_year(next_year);
        }
    }

    /// `_civYearDiff` (reference lines 20580-20595) over this instance's own
    /// timeline -- thin passthrough to `cartalith_civ::timeline::civ_year_diff`,
    /// kept as a method for callers that only have a `CivData`, not the raw
    /// timeline vec.
    fn civ_year_diff(&self, year: i64) -> cartalith_civ::timeline::YearDiff {
        cartalith_civ::timeline::civ_year_diff(&self.timeline, year)
    }
}

/// `TIMELINE_SCOPE.md` §7 success criterion 2: `civ_add_year`/`civ_goto_year`/
/// `civ_remove_year` must reproduce the reference's own snapshot semantics.
/// Runs under plain `cargo test -p cartalith-godot` with no Godot runtime
/// involved -- `CivData` itself has no `godot` type anywhere in it, matching
/// `journey_bridge.rs`'s own precedent for testing Godot-adjacent state
/// without a live engine.
#[cfg(test)]
mod civ_timeline_tests {
    use super::{CIV_FACTION_COUNT, CivData, civ_roster_bridge};
    use cartalith_civ::{NamedSettlement, SettlementKind, SettlementPlacement};

    fn mk_settlement(tid: u64, x: usize, name: &str) -> NamedSettlement {
        NamedSettlement {
            tid,
            placement: SettlementPlacement {
                x,
                y: 0,
                suit: 0.5,
                faction: 1,
                capital: false,
                kind: SettlementKind::Village,
                coastal: false,
            },
            name: name.to_string(),
            pop: 100,
        }
    }

    fn empty_civ(settlements: Vec<NamedSettlement>, territory: Vec<i32>) -> CivData {
        CivData {
            settlements,
            ways: Vec::new(),
            sea_routes: Vec::new(),
            territory,
            provinces: Vec::new(),
            province_list: Vec::new(),
            continents: Vec::new(),
            trade_balances: Vec::new(),
            explanations: Vec::new(),
            water_bodies: Vec::new(),
            next_tid: 1,
            timeline: Vec::new(),
            year: 0,
            dens: Vec::new(),
            faction_roster: civ_roster_bridge::FactionRoster::seeded(CIV_FACTION_COUNT as usize),
            place_extras: civ_roster_bridge::PlaceExtrasTable::default(),
            village_tids: Default::default(),
        }
    }

    #[test]
    fn add_year_on_an_empty_timeline_seeds_it_with_the_live_state_and_jumps_to_it() {
        let mut civ = empty_civ(vec![mk_settlement(1, 5, "Alpha")], vec![0, 3, 0]);
        civ.civ_add_year(-1200);
        assert_eq!(civ.year, -1200);
        assert_eq!(civ.timeline.len(), 1);
        let snap = &civ.timeline[0];
        assert_eq!(snap.year, -1200);
        assert_eq!(snap.settlements.len(), 1);
        assert_eq!(snap.settlements[0].name, "Alpha");
        // civGotoYear's own snapshot-load roundtrip must not have altered the
        // live territory it was just captured from.
        assert_eq!(civ.territory, vec![0, 3, 0]);
    }

    #[test]
    fn add_year_never_loses_the_currently_active_years_live_edits() {
        let mut civ = empty_civ(vec![mk_settlement(1, 5, "Alpha")], vec![1, 0, 0]);
        civ.civ_add_year(0); // seeds year 0 with the live state above

        // Live-edit the always-current arrays (simulating manual editing while
        // year 0 is active) -- exactly what civAddYear's own carry-forward
        // step must snapshot before jumping away.
        civ.settlements.push(mk_settlement(2, 9, "Bravo"));
        civ.territory = vec![1, 0, 2];

        civ.civ_add_year(100); // jump to a brand new year
        assert_eq!(civ.year, 100);

        // Year 0's recorded snapshot must reflect the live edit made while it
        // was active, not the stale state from when it was first added --
        // `TIMELINE_SCOPE.md` §7 success criterion 2, verbatim ("adding a
        // year never loses the currently-active year's state").
        let y0 = civ.timeline.iter().find(|s| s.year == 0).unwrap();
        assert_eq!(y0.settlements.len(), 2, "year 0 must carry the Bravo edit");
        assert_eq!(y0.territory, vec![1, 0, 2]);

        // The live arrays themselves are untouched by any of this -- only
        // `territory` changes on a goto (civGotoYear never touches
        // settlements/ways).
        assert_eq!(civ.settlements.len(), 2);
    }

    #[test]
    fn add_year_carries_forward_from_the_nearest_earlier_recorded_year() {
        let mut civ = empty_civ(vec![mk_settlement(1, 5, "Alpha")], vec![7, 7, 7]);
        civ.civ_add_year(0);
        civ.civ_add_year(500); // no year <=500 other than 0 -> carries year 0 forward

        let y500 = civ.timeline.iter().find(|s| s.year == 500).unwrap();
        assert_eq!(y500.settlements.len(), 1);
        assert_eq!(y500.settlements[0].name, "Alpha");
        assert_eq!(y500.territory, vec![7, 7, 7]);
    }

    #[test]
    fn add_year_on_an_already_recorded_year_does_not_duplicate_or_move_the_cursor() {
        let mut civ = empty_civ(vec![mk_settlement(1, 5, "Alpha")], vec![0, 0, 0]);
        civ.civ_add_year(0);
        civ.civ_add_year(200);
        civ.civ_goto_year(0); // move the cursor away from 200
        civ.civ_add_year(200); // 200 already exists -- must be a no-op past the resnapshot
        assert_eq!(civ.timeline.len(), 2);
        assert_eq!(
            civ.year, 0,
            "civ_add_year on an existing year must not move the cursor"
        );
    }

    #[test]
    fn goto_year_never_mutates_settlements_or_ways_only_territory() {
        let mut civ = empty_civ(vec![mk_settlement(1, 5, "Alpha")], vec![0, 0, 0]);
        civ.civ_add_year(0); // year 0 snapshot: settlements=[Alpha], territory=[0,0,0]
        civ.civ_add_year(100); // year 100 carries [Alpha]/[0,0,0] forward unchanged; cursor=100

        // Diverge the LIVE arrays from BOTH recorded snapshots, so a goto that
        // incorrectly restored settlements from year 0's snapshot would be
        // caught (year 0's own snapshot still says "Alpha", never "live only").
        civ.settlements[0].name = "live only".to_string();
        civ.territory = vec![9, 9, 9];

        civ.civ_goto_year(0);
        assert_eq!(
            civ.territory,
            vec![0, 0, 0],
            "territory restored from year 0's snapshot"
        );
        assert_eq!(
            civ.settlements[0].name, "live only",
            "goto must never touch the live settlements array"
        );
    }

    #[test]
    fn remove_year_falls_back_to_the_earliest_remaining_year() {
        let mut civ = empty_civ(vec![mk_settlement(1, 5, "Alpha")], vec![0, 0, 0]);
        civ.civ_add_year(0);
        civ.civ_add_year(100);
        civ.civ_add_year(200);
        assert_eq!(civ.year, 200);
        civ.civ_remove_year(200); // removing the ACTIVE year
        assert_eq!(
            civ.year, 0,
            "must fall back to the earliest remaining year, not the next one"
        );
        assert_eq!(civ.timeline.len(), 2);
    }

    #[test]
    fn remove_year_falls_back_to_zero_when_none_remain() {
        let mut civ = empty_civ(vec![mk_settlement(1, 5, "Alpha")], vec![0, 0, 0]);
        civ.civ_add_year(50);
        civ.civ_remove_year(50);
        assert_eq!(civ.year, 0, "reference: `next?next.year:0`");
        assert!(civ.timeline.is_empty());
    }

    #[test]
    fn remove_year_on_an_unrecorded_year_is_a_no_op() {
        let mut civ = empty_civ(vec![mk_settlement(1, 5, "Alpha")], vec![0, 0, 0]);
        civ.civ_add_year(50);
        let cursor_before = civ.year;
        civ.civ_remove_year(9999);
        assert_eq!(civ.year, cursor_before);
        assert_eq!(civ.timeline.len(), 1);
    }
}

/// One settlement's "why here?" record: the real decomposition of its
/// suitability score, plus the handful of context readings the causal
/// chain in `VISION.md` actually names (river, coast, terrain, route
/// cost). Every value is read straight from a computed raster -- nothing
/// here is inferred or estimated.
struct SettlementExplanation {
    suit: cartalith_civ::SuitExplanation,
    elevation: f32,
    /// Distance to coast in cells (negative = offshore), matching
    /// `build_coast_sdf`'s own sign convention as the suitability coast
    /// term reads it.
    coast_dist_cells: f32,
    /// Strahler order at this cell, `0` where no river was extracted.
    river_order: i16,
    flow: f32,
    /// `build_travel_cost` at this cell -- the same movement-cost surface
    /// the road network and territory Dijkstras both run over.
    travel_cost: f32,
    /// `build_biome_raster`'s class id at this cell.
    biome: u8,
}

/// Runs the full Phase 2 pipeline (milestones 1-11) over a freshly
/// generated `WorldState`, in exactly the dependency order each
/// milestone's own golden test exercises
/// (`cartalith-civ/tests/golden_parity_settlement_placement.rs`'s
/// `compute_placements` helper is the canonical reference for this
/// chain — mirrored here, not reinvented). `CIV_FACTION_COUNT=6` matches
/// the reference's real `CIV_FACTIONS.length-1` (7 entries including
/// "Unclaimed" at index 0, reference line ~14568).
const CIV_FACTION_COUNT: i32 = 6;

/// Smallest landmass `get_continents()` will list
/// (`cartalith_civ::civ_continents`' `min_cells`).
///
/// Grid-relative would be the tempting choice and is the wrong one: the
/// question a user is asking is "is this a place, or is it a rock", and that
/// is a property of the landmass, not of the resolution it was sampled at.
/// 64 cells is a 8x8 block — the smallest island this port's own settlement
/// placement will ever put a settlement on with room around it. Everything
/// smaller is still in the partition and still scores settlement suitability;
/// it just does not get an entry in a list a person is expected to read.
const CONTINENT_MIN_CELLS: usize = 64;

/// The six-faction swatch, indexed `faction - 1` (faction `0` is always
/// "Unclaimed" and never drawn). Hoisted out of `build_territory_texture`
/// (its original, still-only-real, home) so `get_factions` -- CIVIL
/// milestone F's Territory tool right dock, `DCC_SHELL_SPEC.md` §4.5.3's
/// "faction swatch" -- can report the same colours a painted territory
/// overlay actually renders in, rather than inventing a second palette that
/// could silently drift from this one.
const FACTION_RGB: [(u8, u8, u8); 6] = [(230, 159, 0), (86, 180, 233), (0, 158, 115), (240, 228, 66), (0, 114, 178), (213, 94, 0)];

/// The territory wash's default alpha — `82/255`, ~0.32, low enough for
/// terrain and biome colour to read through. This port's own value and not
/// the reference's `130/255`: the renderer under the wash here is doing more
/// work (hillshade, splat, grade, NPR) than the reference's flat biome fill,
/// and a heavier wash buries it. `GUI_GAP_REGISTER.md` CA-17 made it a
/// slider; this is where that slider starts.
const TERRITORY_ALPHA_DEFAULT: f64 = 82.0 / 255.0;

/// The three opt-in civ-layer passes the reference gates behind its own
/// auto-populate checkboxes/dropdown, carried together rather than as three
/// more positional `bool`/enum arguments on an already-long signature.
/// Every default is the reference's own default, so
/// `CivOptions::default()` reproduces auto-populate's out-of-the-box run
/// exactly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct CivOptions {
    /// `_civVillages` (reference line ~6441), default OFF.
    villages: bool,
    /// `_civMetropolis` (reference line 6442), default OFF -- "OFF by
    /// default => auto-populate output bit-identical", the reference's own
    /// comment.
    metropolis: bool,
    /// `_civRecoveryPhase` (reference line 6443), default `Stable` (0),
    /// which makes `civ_apply_recovery` a strict no-op.
    recovery: RecoveryPhaseOpt,
    /// `_biomeK` (reference line 6441), default OFF -- the biome
    /// carrying-capacity residual (`civBiomeKChk`). The reference's own
    /// comment on that default is why it stays OFF here: "0 = biome
    /// carrying-capacity residual OFF (bit-identical)". `build_carrying_
    /// capacity` has always taken the parameter; until this flag existed
    /// nothing could turn it on (`PARITY_AUDIT.md` §5 item 12).
    biome_k: bool,
}

/// `RecoveryPhase` with a `Default` -- `cartalith_civ`'s own enum has no
/// meaningful default to declare (`Stable` is only "the default" from this
/// caller's point of view), so the defaulting lives here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct RecoveryPhaseOpt(u8);

impl RecoveryPhaseOpt {
    fn phase(self) -> cartalith_civ::timeline::RecoveryPhase {
        cartalith_civ::timeline::RecoveryPhase::from_index_clamped(i64::from(self.0))
    }
}

/// `keep` is what separates the two callers (`GUI_GAP_REGISTER.md` SG-02):
///
/// - **`None`** — `absorb`'s path, and the only one that existed before
///   SG-02. Everything below is derived from the terrain, settlement
///   placement included: seeds are found, places are dropped, named,
///   populated, optionally seeded with villages and put through the recovery
///   phase. Bit-identical to what this function has always done.
/// - **`Some(kept)`** — [`WorldGen::recompute_civilisation`]'s
///   path. The settlement list is an **input**, not something to re-derive:
///   it is taken verbatim (positions, names, tiers, populations, factions,
///   `tid`s, and any place the user dropped or edited by hand), and
///   everything downstream of it — water bodies, biome, soil, resources,
///   roads, sea lanes, territory, provinces, trade balances, explanations,
///   agrarian density — is recomputed against the *current* terrain.
///
///   Placement is deliberately not re-derived on that path. Re-rolling it
///   would move every settlement, re-run naming from a fresh RNG (so every
///   name changes), and drop every hand-placed and hand-edited one — the
///   silent-loss failure `civ_roster_bridge`'s `tid`-keyed side table exists
///   to prevent. "Re-place the world from scratch" already has a control:
///   `generate()`.
///
///   `KeptCiv::next_tid` comes in with it because ways are re-issued `tid`s
///   from the same counter the kept settlements were numbered out of;
///   restarting it at 1 would hand a new road the `tid` of a live
///   settlement. `KeptCiv::village_tids` comes in because villages are not
///   road-network nodes — see [`CivData::village_tids`].
#[allow(clippy::too_many_arguments)]
fn compute_civilisation(
    ws: &cartalith_engine::WorldState,
    gw: usize,
    gh: usize,
    world: bool,
    map_width_km: f64,
    river_density: f64,
    opts: CivOptions,
    keep: Option<KeptCiv>,
) -> CivData {
    let keeping = keep.is_some();
    let sea_level = ws.sea_level;
    let wb = cartalith_civ::build_water_bodies(&ws.field, gw, gh, sea_level, world, Some(&ws.rainfall));
    let biome = cartalith_civ::build_biome_raster(&wb.classification, &ws.temperature, &ws.rainfall);

    let soil_slope = cartalith_civ::build_slope_field(&ws.field, gw, gh, world);
    let lithology = cartalith_civ::build_lithology(
        &ws.field, &ws.age_field, &ws.volcanic_field, &ws.crust_field, &ws.resistance_field, &ws.rainfall, sea_level,
    );
    let soil = cartalith_civ::build_soil_fertility(&lithology, &ws.temperature, &ws.rainfall, &soil_slope, &ws.age_field);

    let flow_thresh = cartalith_hydrology::river_flow_thresh(gw, gh, gw, map_width_km);
    let water_access = cartalith_civ::build_water_access(&ws.flow_discharge, &ws.field, gw, gh, sea_level, flow_thresh);
    // `civBiomeKChk` (`set_biome_k_enabled`, `CivOptions::biome_k`). The
    // reference's own `currentCarryingCapacity` (line 6453) passes
    // `{biomeK:_biomeK, wetMask:_biomeK?currentWetlandMask():null}` -- the
    // wetland mask is built ONLY when the residual is on, and the `0.0`
    // default short-circuits the whole correction, so the OFF path below is
    // byte-identical to what this line always did.
    let wetland = if opts.biome_k {
        Some(cartalith_civ::build_wetland_mask(&wb.classification, &ws.field, &ws.rainfall, &soil_slope, sea_level))
    } else {
        None
    };
    let carrying_cap = cartalith_civ::build_carrying_capacity(
        &soil, &water_access, Some(&biome), &ws.temperature, &ws.field, sea_level,
        if opts.biome_k { 1.0 } else { 0.0 }, wetland.as_deref(),
    );

    let mut resources = cartalith_civ::build_resource_potentials(
        &lithology,
        Some(&ws.boundary_type),
        Some(&ws.shear_field),
        Some(&ws.flow_discharge),
        Some(&biome),
        &ws.field,
        &ws.rainfall,
        &ws.age_field,
        gw,
        gh,
        sea_level,
        Some(&ws.volcanic_field),
        true,
        false,
    );
    // `ResourcePotentials` carries all 15 fields (`build_resource_potentials`
    // computes them together in one shared per-cell loop -- not splittable
    // without real restructuring, `MEMORY_OPTIMIZATION_SCOPE.md`). Only 9
    // (`SUIT_RESOURCE_KEYS`) feed `build_settlement_suitability`'s mineral
    // term via `ctx.resources` below, but as of the economy wiring
    // (`ECONOMY_SCOPE.md`) the other 6 (clay/buildstone/flint/obsidian/
    // sulfur/alum) are no longer dead weight either -- the per-settlement
    // trade-balance computation further down (right before this struct's
    // free point) genuinely needs the full 15-key vocabulary. The free that
    // used to happen right here (immediately after this call) has moved to
    // just after that computation, once `settlements` exists -- a real,
    // bounded, measured tradeoff (these 6 fields, ~96 MB at 2048x2048, now
    // stay resident through settlement placement/roads/naming instead of
    // being dropped immediately), not a silent revert of the memory fix:
    // steady-state after `compute_civilisation()` returns is unaffected,
    // only this function's own transient peak grows by that bounded amount.

    let raw_slope = cartalith_civ::build_raw_slope_field(&ws.field, gw, gh, world);
    let corridors = cartalith_civ::build_route_corridors(&ws.field, &raw_slope, Some(&ws.flow_discharge), gw, gh, sea_level, world, flow_thresh);
    let landmass = cartalith_civ::build_landmass_quality(&ws.field, Some(&carrying_cap), gw, gh, sea_level, world);
    let coast_sdf = cartalith_civ::build_coast_sdf(&ws.field, gw, gh, sea_level);
    let flood = cartalith_civ::build_flood_field(&ws.field, &ws.flow_discharge, &raw_slope, gw, gh, sea_level);
    let river_order = cartalith_civ::fresh_river_order(&ws.field, &ws.flow_discharge, gw, gh, sea_level, world, river_density, map_width_km);

    let ctx = cartalith_civ::SuitabilityCtx {
        water_bodies: Some(&wb.classification),
        corridor: Some(&corridors),
        landmass: Some(&landmass.quality),
        flow: Some(&ws.flow_discharge),
        river_order: Some(&river_order),
        coast_sdf: Some(&coast_sdf),
        resources: Some(&resources),
        rain: Some(&ws.rainfall),
        flood: Some(&flood),
        slope_raw: Some(&raw_slope),
        flow_thresh,
    };

    // The reference names its own two slope reads separately (`currentSoil`'s
    // `slopeN` and the suitability pass's), and this port had followed it into
    // calling `build_slope_field` twice with the identical four arguments over
    // an immutable `ws.field` -- 2.65 ms of the same answer at 2048x2048.
    // `soil_slope` above IS that answer, bit for bit.
    let suit = cartalith_civ::build_settlement_suitability(&soil, &water_access, &carrying_cap, &ws.field, &soil_slope, gw, gh, sea_level, Some(&ctx));
    // SG-02: which kept settlements are road-network nodes, as indices back
    // into the kept list. Villages are excluded (see `CivData::village_tids`),
    // so this is not the identity map and `topology`'s edge endpoints have to
    // be remapped through it below. Empty on the auto-populate path, where
    // `placements` *is* the network and no remap is needed.
    let net_idx: Vec<usize> = match keep.as_ref() {
        Some(k) => (0..k.settlements.len())
            .filter(|&i| !k.village_tids.contains(&k.settlements[i].tid))
            .collect(),
        None => Vec::new(),
    };
    let mut placements: Vec<cartalith_civ::SettlementPlacement> = match keep.as_ref() {
        // SG-02: the kept list's own placements, straight through. Roads,
        // territory and provinces below are then rebuilt around exactly the
        // settlements that are actually on the map.
        Some(k) => net_idx.iter().map(|&i| k.settlements[i].placement).collect(),
        None => {
            // Reference `_civIterativeAutoWorld` (the real auto-populate entry
            // point this function mirrors -- not the standalone .f32/JSON export
            // path, which the earlier golden-parity fixtures were built against):
            // `findSettlementSeeds(suit,GW,GH,{thresh:wantCounts?0.35:SETTLE_SEED_THRESH,suppR})`
            // with `suppR=wantCounts?...:Math.max(6,(GW/22)|0)`. This port has no
            // `wantCounts` (no fixed-tier-count UI), so the real default branch is
            // `SETTLE_SEED_THRESH=0.42`/`max(6, floor(GW/22))` -- found and flagged
            // by Phase 2 milestone 15's own investigation (`PHASE2_SCOPE.md`),
            // corrected here rather than left as the placeholder 0.65/GW/20 an
            // earlier pass used before this real call site existed to check against.
            let seeds = cartalith_civ::find_settlement_seeds(&suit, gw, gh, 0.42, (gw as f64 / 22.0).floor().max(6.0));
            cartalith_civ::place_settlements_with_water_edge_snap(
                &seeds, &suit, &ws.field, &wb.classification, &wb.fill_level, gw, gh, sea_level, world, CIV_FACTION_COUNT,
                &flood, &ws.flow_discharge, flow_thresh, map_width_km,
            )
        }
    };

    // Real auto-populate road network, not `build_road_network` (that's
    // `buildRoadNetwork`, the reference's *manual*-placement-tool
    // algorithm -- an earlier pass here used it as a stand-in for the
    // auto-populate system before that system was ported at all). Now
    // that both exist: `civ_hierarchical_network_topology` (Phase 2
    // milestone 12, the real `_civHierarchicalNetwork` raw topology) then
    // `civ_consolidate_and_smooth_ways` (milestone 14) for the
    // Catmull-Rom-smoothed, classified, named polylines this map actually
    // draws.
    let mut topology = cartalith_civ::civ_hierarchical_network_topology(
        &placements, gw, gh, sea_level, &ws.field, &ws.flow_discharge, &river_order, &biome, &wb.classification, world, map_width_km,
    );
    // `RoadEdge::a`/`.b` index into `placements`, and
    // `civ_consolidate_and_smooth_ways` below reads them as indices into
    // `settlements` -- an identity the auto-populate path gets for free
    // (villages are appended after, never interleaved) and the SG-02 path
    // does not, since `net_idx` skipped the villages. `path` is cell
    // indices, and `usage_count` is per cell, so neither is affected;
    // `degree_of` is internal to the topology pass and read by nothing here.
    if keeping {
        for e in topology.edges.iter_mut() {
            e.a = net_idx[e.a];
            e.b = net_idx[e.b];
        }
    }
    // v0.75 imperial-seat promotion (`_civSelectMetropolises`, reference
    // lines 24961-24989), wired exactly where the reference wires it: inside
    // auto-populate, AFTER the road network exists and its betweenness has
    // been measured, BEFORE naming/population -- reference line 25711, whose
    // own guard is `_civMetropolis && !wantCounts`. This port has no
    // `wantCounts` (no fixed-tier-count UI, `place_settlements`' own note),
    // so only the opt-in flag remains, default OFF like the reference's.
    //
    // The reference reads betweenness out of `_civNetworkMetrics(places,
    // ways)` (line 21931), which this port has never needed and does not
    // have. What `civ_select_metropolises` actually consumes is only the
    // RATIO `betweenness[i]/max_btw`, so the missing piece is just Brandes
    // over the ways graph -- which `cartalith-civ` already ships as
    // `timeline::civ_betweenness_from_adjacency` (its own doc: "the same
    // algorithm `_civNetworkMetrics` uses"). `_civNetworkMetrics`' own
    // `(n-1)(n-2)` normalisation cancels in that ratio, so feeding it the
    // un-normalised values is bit-identical, and
    // `golden_parity_metropolis_recovery.rs`'s
    // `betweenness_normalisation_cancels_out` pins exactly that.
    //
    // Adjacency is built from `topology.edges`' own place indices (`a`/`b`),
    // which is what the reference's `w.aIdx`/`w.bIdx` preferred branch uses
    // too -- its geometric `nearestPlace` fallback exists only for ways that
    // carry no logical endpoints, a shape this port never produces. Sea
    // lanes are excluded here for the same reason the reference excludes
    // them (`if(w.sea) continue`): they are not part of `topology.edges` at
    // all.
    // `keep.is_none()`: promotion overwrites `kind`, and on the SG-02 path
    // `kind` may be a user's own choice from the place editor. Re-running it
    // there would quietly undo an edit, which is the one thing that path
    // exists not to do.
    if opts.metropolis && keep.is_none() {
        let mut adj: Vec<std::collections::BTreeSet<usize>> =
            vec![Default::default(); placements.len()];
        for e in &topology.edges {
            if e.a != e.b && e.a < adj.len() && e.b < adj.len() {
                adj[e.a].insert(e.b);
                adj[e.b].insert(e.a);
            }
        }
        let adj: Vec<Vec<usize>> = adj.into_iter().map(|s| s.into_iter().collect()).collect();
        let btw = cartalith_civ::timeline::civ_betweenness_from_adjacency(&adj);
        let max_btw = btw.iter().copied().fold(0.0f64, f64::max);
        // The reference's caller also pushes `trade_hub`/`administrative`
        // onto the promoted place's `traits` (line 25713-25714). This port
        // has no per-settlement trait vector at all (`NamedSettlement` is
        // tid/placement/name/pop) -- the same already-disclosed boundary
        // `timeline_bridge.rs` records for `fortified`/`ruins` -- so `kind`
        // is the whole of the promotion here. Nothing downstream in this
        // port reads either trait.
        for i in cartalith_civ::civ_select_metropolises(
            &placements,
            &btw,
            max_btw,
            cartalith_civ::MetropolisOpts::default(),
        ) {
            placements[i].kind = cartalith_civ::SettlementKind::Metropolis;
        }
    }
    let placements = placements;

    let roads = &topology.edges;
    // Still needed below by `assign_territory` (independent of which road
    // algorithm produced `roads` above -- territory's own Dijkstra runs
    // over this same real terrain-cost field directly, per capital).
    let cost = cartalith_civ::build_travel_cost(&ws.field, gw, gh, sea_level);

    // Reference `_civIterativeAutoWorld`: placement-naming and village
    // seeding draw from ONE continuous mulberry32 stream, not two
    // independent ones (`civ_seed_villages`'s own doc comment). Only
    // matters when villages are actually requested -- the plain
    // `name_and_populate_settlements()` (fresh RNG each call, milestones
    // 8/9's own golden-verified default path) stays exactly as before
    // when they're not, so disabling this toggle is still bit-identical
    // to every existing golden fixture. `_civVillages` (reference default
    // OFF, Phase 2 milestone 15's own "flagged, not resolved" note): opt-
    // in here too, matching JS's real default.
    //
    // The stream is hoisted out of that branch (it used to be created
    // inside it) because the v0.82 recovery pass below is the reference's
    // *third* consumer of the same continuous `rng` closure -- reference
    // line 25761 draws from the very stream naming and village seeding
    // already drew from. Hoisting is bit-identical for the villages-off
    // path: `name_and_populate_settlements()` is literally
    // `civ_name_rng()` followed by the `_with_rng` call now made here.
    let mut rng = cartalith_civ::civ_name_rng();
    // SG-02: naming, village seeding and the recovery phase all *author*
    // settlements. On the keep path they have already run once, and their
    // output is part of what is being kept -- re-running them would rename
    // every place, append a second copy of every village, and apply the
    // collapse a second time on top of itself.
    let (mut settlements, kept_next_tid, kept_village_tids) = match keep {
        Some(k) => (k.settlements, k.next_tid, k.village_tids),
        None => (
            cartalith_civ::name_and_populate_settlements_with_rng(&placements, &mut rng),
            1u64,
            Default::default(),
        ),
    };
    // Which entries are villages, tracked positionally until `tid`s are
    // assigned further down. Rebuilt through the recovery pass below, which
    // can drop entries and so invalidates any index range captured here.
    let mut is_village = vec![false; settlements.len()];
    if opts.villages && !keeping {
        // `civ_seed_villages` needs the downsampled routing grid's
        // (rw, sc) that `civ_hierarchical_network_topology` builds
        // internally (`civ_routing_grid`, private to `cartalith-civ`) --
        // replicated here rather than widening that crate's public API
        // mid-flight while another fork is concurrently editing it. Pure
        // function of `field`/`gw`/`gh` alone, independent of which road
        // algorithm actually produced `roads` above.
        let routing_rw = gw.min(384);
        let routing_sc = routing_rw as f64 / gw as f64;
        let villages = cartalith_civ::civ_seed_villages(
            &settlements,
            roads,
            routing_rw,
            routing_sc,
            &mut rng,
            &suit,
            &ws.field,
            &wb.classification,
            &wb.fill_level,
            gw,
            gh,
            sea_level,
            map_width_km,
        );
        // Reference `_civSeedVillages`'s own added object has no
        // suitability score, no capital/coastal flags, and an
        // unconditional `pop:0` -- `VillageSettlement`'s own doc comment
        // is explicit these are never populated for a village. Wrapped as
        // a `NamedSettlement`/`SettlementPlacement` here (not a distinct
        // rendered shape) purely so `get_settlements()`/`map_overlay.gd`
        // need zero changes: a village renders exactly like any other
        // hamlet, which is what the reference's own hamlet-tier tagging
        // for these already implies.
        settlements.extend(villages.into_iter().map(|v| cartalith_civ::NamedSettlement {
            tid: 0,
            placement: cartalith_civ::SettlementPlacement {
                x: v.x,
                y: v.y,
                suit: 0.0,
                faction: v.faction,
                capital: false,
                kind: cartalith_civ::SettlementKind::Hamlet,
                coastal: false,
            },
            name: v.name,
            pop: 0,
        }));
        is_village.resize(settlements.len(), true);
    }

    // v0.82 static post-collapse recovery (`_civApplyRecovery`, reference
    // lines 24619-24640), wired where the reference wires it: reference line
    // 25761, `if(_civRecoveryPhase>0) places=_civApplyRecovery(places,
    // _civRecoveryPhase, rng, {})` -- after the population pass, so tiers and
    // populations are final before collapse is applied, and drawing from the
    // same continuous stream. Phase `Stable` is a strict no-op that consumes
    // nothing, so the default path is byte-identical to not running it.
    //
    // `CollapsePlace` is the crate's own type for this pass (shared with the
    // v0.85 stepper); `timeline_bridge` already owns both conversions, and
    // the same disclosure applies here as there -- the `ruins`/`fortified`
    // flags this pass sets have no home on `NamedSettlement`, so they are
    // computed and then dropped at this boundary. `kind` and `pop`, which
    // everything downstream actually reads, survive intact.
    if !keeping && opts.recovery.phase() != cartalith_civ::timeline::RecoveryPhase::Stable {
        let before: Vec<cartalith_civ::timeline::CollapsePlace> = settlements
            .iter()
            .enumerate()
            .map(|(i, s)| cartalith_civ::timeline::CollapsePlace {
                // `tid` is still 0 at this point (assigned further down), so
                // the index stands in as the identity for the map-back.
                tid: i as u64,
                x: s.placement.x,
                y: s.placement.y,
                kind: s.placement.kind,
                pop: f64::from(s.pop),
                fortified: false,
                ruins: false,
                port: s.placement.coastal,
            })
            .collect();
        let after = cartalith_civ::timeline::civ_apply_recovery(
            &before,
            opts.recovery.phase(),
            &mut rng,
            cartalith_civ::timeline::RecoveryOpts::default(),
        );
        // `civ_apply_recovery` can drop entries, so the village flags are
        // carried through the same index map rather than left behind.
        is_village = after.iter().map(|p| is_village[p.tid as usize]).collect();
        settlements = after
            .into_iter()
            .map(|p| {
                let mut s = settlements[p.tid as usize].clone();
                s.placement.kind = p.kind;
                s.pop = p.pop.max(0.0).round() as u32;
                s
            })
            .collect();
    }


    // Economy (`ECONOMY_SCOPE.md`): per-settlement resource trade balance,
    // `_civPlaceTrade`'s hinterland term against `_civFactionAggregates`'s
    // world mean -- the one piece of that pair genuinely independent of
    // territory, so it runs here, right after `settlements` (not after
    // `territory` below, which it doesn't need). Real 15-key vocabulary,
    // hence run before `resources`' six extra fields are freed just below.
    let world_mean_resources = cartalith_civ::civ_world_mean_resources(&resources, &ws.field, sea_level);
    let trade_balances: Vec<cartalith_civ::TradeBalance> = settlements
        .iter()
        .map(|s| {
            let cat_km2 = cartalith_civ::civ_catchment_km2(s.placement.kind);
            let radius = cartalith_civ::civ_catchment_radius_cells(cat_km2, map_width_km, gw);
            let ctx_mean = cartalith_civ::civ_place_resource_context(
                &resources, &ws.field, gw, gh, sea_level, s.placement.x, s.placement.y, radius, world,
            );
            cartalith_civ::civ_resource_trade_balance(&ctx_mean, &world_mean_resources)
        })
        .collect();
    // Causal-chain explainer (`VISION.md`): decompose the suitability score
    // at each settlement's own cell into its real weighted terms, so the UI
    // can answer "why is this town here?" from the actual arithmetic that
    // placed it rather than a template.
    //
    // Computed HERE, and per-settlement rather than per-cell, for a real
    // reason: every raster this needs (soil/water/carrying-capacity/coast
    // SDF/river order/flow/corridor/landmass/flood/slope/resources) is a
    // local of this function and dies at its end. Answering "why here?" for
    // an arbitrary cell later would mean retaining all of them -- hundreds
    // of MB at 2048x2048, straight back into what
    // `MEMORY_OPTIMIZATION_SCOPE.md` spent real measurement getting out of.
    // A settlement list is ~40 entries, so explaining all of them costs
    // nothing and covers the question actually being asked.
    let explanations: Vec<SettlementExplanation> = settlements
        .iter()
        .map(|s| {
            let (x, y) = (s.placement.x, s.placement.y);
            let i = y * gw + x;
            SettlementExplanation {
                suit: cartalith_civ::explain_settlement_suitability(
                    &soil,
                    &water_access,
                    &carrying_cap,
                    &ws.field,
                    &soil_slope,
                    gw,
                    gh,
                    sea_level,
                    Some(&ctx),
                    x,
                    y,
                ),
                elevation: ws.field[i],
                // `build_coast_sdf` is negative offshore / positive inland;
                // the suitability coast term reads `-sdf`, so this is the
                // same sign convention, in cells.
                coast_dist_cells: -coast_sdf[i],
                river_order: river_order[i],
                flow: ws.flow_discharge[i],
                travel_cost: cost[i],
                biome: biome[i],
            }
        })
        .collect();

    // See the comment on `build_resource_potentials`'s own call site: this
    // free used to happen immediately after that call, before the economy
    // wiring above needed the full 15-key vocabulary through settlement
    // placement. Nothing after this point reads `resources` at all.
    resources.clay = Vec::new();
    resources.buildstone = Vec::new();
    resources.flint = Vec::new();
    resources.obsidian = Vec::new();
    resources.sulfur = Vec::new();
    resources.alum = Vec::new();

    // Territory (`DECISIONS.md` §7b): reuses the same real terrain
    // travel-cost field the road network above already computed --
    // capitals only (villages/non-capital settlements don't project
    // territory, `assign_territory`'s own `if !capital continue`), so
    // whether villages were added above doesn't change this at all.
    let territory = cartalith_civ::assign_territory(&settlements, &cost, gw, gh, world);
    // Provinces (`_civGenerateProvinces`, PHASE2_SCOPE.md): the reference
    // itself has no programmatic `civTerritory` producer to subdivide, but
    // this port's own `assign_territory` output above is the exact same
    // per-cell shape (`Vec<i32>` faction id, 0 = unowned) the reference
    // function expects -- see `civ_generate_provinces`'s own doc comment.
    let (provinces, province_list) = cartalith_civ::civ_generate_provinces(&settlements, &territory, gw, gh);
    // Addressable landmasses (`MARKDOWN_VAULT_SCOPE.md` milestone 0). Free:
    // `landmass` above is already the golden-verified flood fill, and this
    // only keeps its component bookkeeping instead of dropping it. After
    // `territory` because the naming culture comes from whichever faction
    // holds the most of each landmass.
    //
    // `CONTINENT_MIN_CELLS` is a floor on what gets *listed*, not a claim
    // about what a continent is: a two-cell rock is not something a user
    // wants a note attached to, and an archipelago world legitimately has no
    // large landmass at all and correctly reports none.
    let continents = cartalith_civ::civ_continents(&landmass, gw, gh, CONTINENT_MIN_CELLS, Some(&territory));

    // Milestone 14 consolidation/smoothing needs NAMED settlements
    // (`pa.name`/`pb.name`) -- must run after the naming/village block
    // above, not alongside `topology`. `topology.edges`' `a`/`b` indices
    // are into the pre-village `placements` order, which `settlements`
    // preserves as its own prefix (villages are appended after, never
    // interleaved), so indexing stays valid whether or not villages ran.
    let mut ways = cartalith_civ::civ_consolidate_and_smooth_ways(&topology, &settlements, &ws.field, &wb.classification, gw, gh, map_width_km);

    // Sea routes (milestone 13): reference calls `_civMstRoutes(ports,true)`
    // unconditionally whenever >=2 port-tagged settlements exist, over the
    // SAME `settlements` list `ways` was just built from (villages included,
    // if enabled -- `civ_sea_routes` itself gates on `.coastal`, and a
    // village's own `coastal: false` above means villages never qualify as
    // ports here, matching the reference's own hamlet-tier village shape).
    let ports: Vec<cartalith_civ::NamedSettlement> =
        settlements.iter().filter(|s| s.placement.coastal).cloned().collect();
    let sea_routes = cartalith_civ::civ_sea_routes(&ports, &ws.field, &wb.classification, gw, gh, world, map_width_km);

    // `TIMELINE_SCOPE.md` milestone 1: stamp every settlement/way with a
    // real `tid` right here -- the "placement/road-generation time" this
    // port chose over the reference's lazy first-touch (see `cartalith_civ::
    // timeline`'s module doc). Every entry above was freshly constructed
    // with `tid: 0` (the crate's own unassigned sentinel), so a simple
    // counter starting at 1 assigns every one of them in one pass; nothing
    // here can already carry a nonzero tid, but `civ_assign_tid` is
    // idempotent regardless.
    // `kept_next_tid` is 1 on the auto-populate path (nothing has an id yet)
    // and the live counter on the SG-02 keep path, where every settlement
    // already carries one and only the freshly rebuilt `ways` need new ones.
    let mut next_tid = kept_next_tid;
    for s in settlements.iter_mut() {
        s.tid = cartalith_civ::timeline::civ_assign_tid(s.tid, &mut next_tid);
    }
    for w in ways.iter_mut() {
        w.tid = cartalith_civ::timeline::civ_assign_tid(w.tid, &mut next_tid);
    }
    // Positional village flags become `tid`s here, the moment `tid`s exist
    // and the moment the positions stop being trustworthy. Empty on the keep
    // path, where `kept_village_tids` is already the answer.
    let village_tids: std::collections::HashSet<u64> = if keeping {
        kept_village_tids
    } else {
        settlements
            .iter()
            .zip(is_village.iter())
            .filter(|(_, v)| **v)
            .map(|(s, _)| s.tid)
            .collect()
    };

    // `TIMELINE_SCOPE.md` milestone 5: `currentAgrarianDensity()`'s per-cell output,
    // computed here (not in `timeline_bridge.rs`) because `carrying_cap`/`water_access`/
    // `biome` are already-live locals of THIS function -- exactly the reasoning
    // `water_bodies` above already documents for its own retention past this point.
    let dens = cartalith_civ::timeline::civ_current_agrarian_density(
        &carrying_cap, &water_access, Some(&biome), &ws.rainfall, &ws.field, sea_level,
    );

    // `TIMELINE_SCOPE.md` §1 Cluster D: the reference's own `generate()` wrapper clears
    // `civTerritory`/`civTimeline`/`civYear` back to empty on every fresh procedural
    // generation -- this is that same reset, for the one function that (re)builds `CivData`
    // from scratch. A loaded save never reaches this function at all (`self.civ` is only
    // ever set here, never restored from `SaveData` -- this struct's own top-of-file doc
    // comment), so there is no case where a real recorded timeline needs preserving across
    // this call.
    //
    // On the SG-02 keep path there *is* such a case, and it is handled one
    // level up rather than here: `recompute_civilisation` moves the previous
    // `timeline`/`year`/`faction_roster`/`place_extras` onto the struct this
    // builds. They are boundary state with no terrain input at all, so the
    // reset below would be a pure loss there — see that method's own doc.
    CivData {
        settlements,
        ways,
        sea_routes,
        territory,
        provinces,
        province_list,
        continents,
        trade_balances,
        explanations,
        water_bodies: wb.classification,
        next_tid,
        timeline: Vec::new(),
        year: 0,
        dens,
        // Same Cluster-D reset the timeline gets: a fresh generation is a
        // fresh roster. `CIV_FACTION_COUNT` is what `assign_factions` was
        // just run with, so the roster and the settlements agree on which
        // ids exist at the moment this struct is built.
        faction_roster: civ_roster_bridge::FactionRoster::seeded(CIV_FACTION_COUNT as usize),
        place_extras: civ_roster_bridge::PlaceExtrasTable::default(),
        village_tids,
    }
}

/// Grid cells between successive points of the road polyline `get_roads()`
/// hands the renderer — see that function's doc comment for why the way's
/// own 3-cell sampling is too coarse here.
///
/// 0.25 cells, not "as fine as possible": at `ViewportHost.ZOOM_MAX` on a
/// 384-cell grid one cell is ~29 screen px, so this is a ~7 px chord at the
/// deepest zoom the camera reaches and sub-pixel at every zoom below it —
/// the point at which a finer step buys nothing visible. It costs ~11x the
/// points (a measured 51-way, 384x288 world goes from ~1.0k to ~11k across
/// the whole network), which `map_overlay.gd` walks once per *redraw*, not
/// per frame: `ViewportHost.refresh()` pushes `get_roads()` into
/// `set_civ_data` and the overlay caches it.
const WAY_RENDER_STEP_CELLS: f64 = 0.25;

/// One way's drawable geometry: its Catmull-Rom curve re-sampled at
/// [`WAY_RENDER_STEP_CELLS`], with `brks` remapped onto the new indices.
///
/// Each run between breaks is re-sampled on its own, exactly as
/// `_civSmoothPath` built it — splining across a break would draw a phantom
/// curve through the seam the break exists to lift the pen at. A run of
/// fewer than two points is passed through untouched (there is no curve to
/// refine, and `civ_catmull_rom_sample` returns such input unchanged
/// anyway).
///
/// This is the pure core, so the index arithmetic is testable without a
/// Godot runtime (the same reason `civ_timeline_tests` keeps `CivData` free
/// of `godot` types); [`way_render_geometry`] is the thin `Packed*` wrapper
/// over it.
fn way_render_polyline(pts: &[(f64, f64)], brks: &[usize]) -> (Vec<(f64, f64)>, Vec<usize>) {
    let mut points: Vec<(f64, f64)> = Vec::new();
    let mut out_brks: Vec<usize> = Vec::new();
    let mut start = 0usize;
    for &cut in brks.iter().chain(std::iter::once(&pts.len())) {
        // Defensive on the engine's own indices: a `brks` entry outside the
        // point list, or out of order, must not panic across the gdext
        // boundary (`cartalith-rust-conventions`).
        let cut = cut.min(pts.len());
        if cut <= start {
            continue;
        }
        if !points.is_empty() {
            out_brks.push(points.len());
        }
        let run = &pts[start..cut];
        // `_civSmoothPath` rounds each spline sample to a whole cell, so
        // the list handed to us can repeat a point where the engine's own
        // input (`civ_rdp_simplify`'s output) never does. That used to come
        // back NaN; `civ_catmull_rom_sample` now collapses repeats itself,
        // which fixes it for roads, sea lanes and committed routes at once
        // -- see its doc comment for why that is parity-neutral.
        let curve = cartalith_civ::civ_catmull_rom_sample(run, WAY_RENDER_STEP_CELLS);
        // `civ_catmull_rom_sample` can still return fewer points than it was
        // given (an all-identical run collapses to nothing); keep the
        // original in that case rather than losing a stroke.
        let first = points.len();
        if curve.len() >= run.len() {
            points.extend_from_slice(&curve);
        } else {
            points.extend_from_slice(run);
        }
        // Re-assert the run's own endpoints. The spline passes through its
        // control points analytically, but evaluating it at `t == t1`/`t2`
        // lands ~1e-16 off, and `_civSmoothPath`'s v0.92 fix exists
        // precisely so a way's endpoint is the settlement's exact
        // coordinate. Below f32 resolution either way -- written so the
        // invariant is stated, not inferred.
        let last = points.len() - 1;
        points[first] = run[0];
        points[last] = run[run.len() - 1];
        start = cut;
    }
    (points, out_brks)
}

fn way_render_geometry(
    pts: &[(f64, f64)],
    brks: &[usize],
) -> (PackedVector2Array, PackedInt32Array) {
    let (points, brks) = way_render_polyline(pts, brks);
    (
        points.iter().map(|&(x, y)| Vector2::new(x as f32, y as f32)).collect(),
        brks.iter().map(|&b| b as i32).collect(),
    )
}

#[cfg(test)]
mod way_render_tests {
    use super::{WAY_RENDER_STEP_CELLS, way_render_polyline};

    /// The point of the whole change: a way's drawn polyline is denser than
    /// its control points, and its chords are `WAY_RENDER_STEP_CELLS`-scale
    /// rather than the 3 cells `_civSmoothPath` sampled at.
    #[test]
    fn resamples_a_way_to_render_density() {
        // 4 control points, 3 cells apart, with a real corner in them.
        let pts = vec![(0.0, 0.0), (3.0, 0.0), (6.0, 1.0), (9.0, 4.0)];
        let (out, brks) = way_render_polyline(&pts, &[]);
        assert!(brks.is_empty(), "no breaks in, none out");
        assert!(out.len() > 4 * pts.len(), "expected render density, got {} points", out.len());
        for w in out.windows(2) {
            let d = (w[1].0 - w[0].0).hypot(w[1].1 - w[0].1);
            assert!(d < WAY_RENDER_STEP_CELLS * 2.0, "chord {d} too long");
        }
        // Endpoints are the way's own, unmoved -- a road must still meet its
        // settlement pin exactly (`_civSmoothPath`'s own v0.92 fix).
        assert_eq!(out[0], pts[0]);
        let last = out[out.len() - 1];
        assert!((last.0 - 9.0).abs() < 1e-9 && (last.1 - 4.0).abs() < 1e-9, "{last:?}");
    }

    /// A break must still separate the two runs it separated before, and the
    /// runs must not be splined across it.
    #[test]
    fn remaps_breaks_onto_the_resampled_list() {
        let pts = vec![
            (0.0, 0.0), (3.0, 0.0), (6.0, 0.0), // run A
            (60.0, 40.0), (63.0, 40.0), (66.0, 41.0), // run B, far away
        ];
        let (out, brks) = way_render_polyline(&pts, &[3]);
        assert_eq!(brks.len(), 1, "one break in, one out");
        let cut = brks[0];
        assert_eq!(out[0], (0.0, 0.0), "run A still starts at A's first point");
        assert_eq!(out[cut], (60.0, 40.0), "run B still starts at B's first point");
        // Nothing was drawn across the seam: the last point of run A is A's
        // own end, not an interpolation towards B.
        let a_end = out[cut - 1];
        assert!((a_end.0 - 6.0).abs() < 1e-9 && a_end.1.abs() < 1e-9, "{a_end:?}");
    }

    /// Degenerate inputs the engine can legitimately produce must pass
    /// through rather than panic across the gdext boundary.
    #[test]
    fn degenerate_inputs_survive() {
        assert_eq!(way_render_polyline(&[], &[]), (vec![], vec![]));
        assert_eq!(way_render_polyline(&[(1.0, 2.0)], &[]), (vec![(1.0, 2.0)], vec![]));
        // Out-of-range and out-of-order break indices.
        let pts = vec![(0.0, 0.0), (3.0, 0.0), (6.0, 0.0)];
        let (out, _) = way_render_polyline(&pts, &[99]);
        assert!(out.len() > 3);
        let (out2, _) = way_render_polyline(&pts, &[2, 1]);
        assert!(!out2.is_empty());
    }

    /// `civ_dijkstra_path`'s unreachable-leg fallback is a two-point
    /// straight line, and committed routes (`route_get`'s `render_points`)
    /// can carry one. Densifying it must leave it dead straight -- a
    /// two-point run's Catmull-Rom phantom endpoints are its own
    /// reflections, so the curve through them is the chord.
    #[test]
    fn a_two_point_straight_leg_stays_straight() {
        let pts = vec![(4.0, 4.0), (40.0, 31.0)];
        let (out, _) = way_render_polyline(&pts, &[]);
        assert!(out.len() > 100, "expected render density, got {}", out.len());
        assert_eq!(out[0], pts[0]);
        assert_eq!(out[out.len() - 1], pts[1]);
        // Cross product of (end - start) with (p - start): zero for every
        // point on the chord.
        let (dx, dy) = (pts[1].0 - pts[0].0, pts[1].1 - pts[0].1);
        for &(x, y) in &out {
            let cross = dx * (y - pts[0].1) - dy * (x - pts[0].0);
            assert!(cross.abs() < 1e-9, "point ({x}, {y}) is off the chord by {cross}");
        }
    }

    /// The bug the real sea lanes reported as `chord mean -nan`.
    /// `_civSmoothPath` rounds every spline sample to a whole cell, so a
    /// slow-moving stretch of a way emits the same cell twice -- and a
    /// coincident control-point pair used to make `civ_catmull_rom_sample`
    /// divide by a zero knot interval. Guarded in that function now, so
    /// roads, sea lanes and committed routes are all covered; this pins the
    /// boundary, since it is this boundary that produces such input.
    ///
    /// Checked across a `brks` seam too: a NaN anywhere in the flat point
    /// list is enough to ruin the whole `PackedVector2Array`.
    #[test]
    fn a_repeated_cell_in_a_rounded_way_does_not_produce_nan() {
        // Two runs, each with a stalled sample: at the head of the first,
        // in the middle of the second.
        let pts = vec![
            (4.0, 4.0),
            (4.0, 4.0),
            (12.0, 9.0),
            (20.0, 7.0),
            (40.0, 30.0),
            (48.0, 35.0),
            (48.0, 35.0),
            (56.0, 33.0),
            (64.0, 41.0),
        ];
        let (out, brks) = way_render_polyline(&pts, &[4]);
        assert_eq!(brks.len(), 1, "the seam must survive the re-sample");
        assert!(out.len() > 150, "expected render density, got {}", out.len());
        assert!(
            out.iter().all(|&(x, y)| x.is_finite() && y.is_finite()),
            "non-finite coordinate in the drawn polyline"
        );
        // Endpoints of both runs are still the way's own.
        assert_eq!(out[0], pts[0]);
        assert_eq!(out[brks[0] - 1], pts[3]);
        assert_eq!(out[brks[0]], pts[4]);
        assert_eq!(out[out.len() - 1], pts[8]);
    }

    /// A run that is nothing but one repeated cell has no curve to draw.
    /// `civ_catmull_rom_sample` returns nothing for it (as the reference
    /// already did), and the fallback must keep the stroke rather than
    /// silently dropping it or indexing past the end.
    #[test]
    fn a_fully_degenerate_run_keeps_its_points() {
        let pts = vec![(7.0, 7.0), (7.0, 7.0), (7.0, 7.0)];
        let (out, _) = way_render_polyline(&pts, &[]);
        assert_eq!(out, pts);
    }
}

/// Named World-Structure archetype presets (reference HTML `ARCHETYPES`,
/// lines 2521-2526) as `(continentality, fragmentation, tectonic_energy,
/// ocean_depth, hotspot_density)`.
/// `cartalith_engine::WorldParams::world_structure` itself takes raw knobs
/// only, not named presets (its own doc comment: "a caller wanting
/// 'Archipelago' passes that preset's own numbers") — so the name -> knobs
/// lookup lives here, in the boundary layer, rather than in GDScript
/// (`ARCHITECTURE.md`: "Godot computes nothing beyond layout").
const ARCHETYPES: [(&str, [f64; 5]); 5] = [
    ("earth", [0.30, 0.50, 0.60, 0.60, 0.20]),
    ("supercontinent", [0.60, 0.10, 0.50, 0.70, 0.10]),
    ("archipelago", [0.15, 0.90, 0.80, 0.30, 0.50]),
    ("volcanic", [0.05, 1.00, 0.90, 0.80, 1.00]),
    ("rift", [0.40, 0.35, 0.75, 0.55, 0.30]),
];

/// `ARCHETYPES` lookup, case-insensitive, as a ready `WorldStructureParams`
/// with `enabled: true`. `None` for an unknown name.
fn archetype_knobs(name: &GString) -> Option<WorldStructureParams> {
    let name = name.to_string().to_lowercase();
    ARCHETYPES.iter().find(|(n, _)| *n == name).map(|(_, k)| WorldStructureParams {
        enabled: true,
        continentality: k[0],
        fragmentation: k[1],
        tectonic_energy: k[2],
        ocean_depth: k[3],
        hotspot_density: k[4],
    })
}

/// One `params::Value` as the Godot type its `Kind` promises — `bool` for a
/// checkbox parameter, `int` for a whole-number one, `float` otherwise. Keeps
/// GDScript from having to guess whether `params["tect.plates"]` is `14` or
/// `14.0`.
fn value_to_variant(kind: params::Kind, value: params::Value) -> Variant {
    match (kind, value) {
        (_, params::Value::Bool(b)) => b.to_variant(),
        (params::Kind::Int, params::Value::Num(n)) => (n as i64).to_variant(),
        (_, params::Value::Num(n)) => n.to_variant(),
    }
}

/// Every parameter in `p`, as the flat dotted-key `Dictionary` `get_params`/
/// `get_param_defaults` both return.
fn params_to_dict(p: &WorldParams) -> VarDictionary {
    let mut out = VarDictionary::new();
    for s in params::PARAMS {
        let v = params::get(p, s.key).expect("every table key resolves against its own table");
        out.set(s.key, &value_to_variant(s.kind, v));
    }
    out
}

/// The GDScript side of the invalid-value policy (`params::set`'s own doc
/// comment has the full rule): only `bool`, `int` and `float` Variants carry
/// a parameter value at all. Anything else (String, Array, `null`, an
/// Object) is `None` here and reported as rejected — never coerced, since
/// "0" and `null` both have a plausible-looking numeric coercion that would
/// silently write a wrong world.
fn variant_to_value(v: &Variant) -> Option<params::Value> {
    match v.get_type() {
        VariantType::BOOL => Some(params::Value::Bool(v.to::<bool>())),
        VariantType::INT => Some(params::Value::Num(v.to::<i64>() as f64)),
        VariantType::FLOAT => Some(params::Value::Num(v.to::<f64>())),
        _ => None,
    }
}

/// A bare number out of a `Variant`, for the Sculpt bridge's globals/
/// feature-param setters -- unlike `variant_to_value` these have no
/// `Kind::Bool` case at all (every control `sculpt_bridge` exposes is
/// numeric), so a bool `Variant` is correctly `None` here rather than
/// coerced through `params::Value`'s bool branch.
fn variant_to_num(v: &Variant) -> Option<f64> {
    match v.get_type() {
        VariantType::INT => Some(v.to::<i64>() as f64),
        VariantType::FLOAT => Some(v.to::<f64>()),
        _ => None,
    }
}

/// A `sculpt_bridge::Control` table (either `Feature::meta().controls` or
/// `sculpt_bridge::global_controls()`) as the `Array<Dictionary>`
/// `get_sculpt_features`/`get_sculpt_globals_info` both return -- one
/// `Dictionary` per control with `key`/`label`/`min`/`max`/`step`/
/// `default`, the same shape `get_param_info` uses per generation
/// parameter.
fn control_dict(c: &cartalith_terrain::sculpt::Control) -> VarDictionary {
    vdict! {
        "key" => c.key,
        "label" => c.label,
        "min" => c.min,
        "max" => c.max,
        "step" => c.step,
        "default" => c.default,
    }
}

/// `sculpt_bridge::feature_param_pairs`, as a flat `Dictionary` -- plus
/// `sub_mode` (Freehand only), which isn't a numeric control and so isn't
/// in the pairs list at all. Shared by `sculpt_get_feature_params` and
/// `sculpt_list_stamps`, which both need exactly this shape.
fn feature_params_dict(p: &FeatureParams) -> VarDictionary {
    let mut out = VarDictionary::new();
    for (k, v) in sculpt_bridge::feature_param_pairs(p) {
        out.set(k, v);
    }
    if let FeatureParams::Freehand { sub_mode, .. } = p {
        out.set("sub_mode", sub_mode.key());
    }
    out
}

/// `sculpt_bridge::global_controls()`'s keys read off `g`, as a flat
/// `Dictionary` -- shared by `sculpt_get_globals` and `sculpt_list_stamps`
/// (each stamp keeps its own captured globals, distinct from the live
/// tool state `sculpt_get_globals` reports).
fn globals_dict(g: &cartalith_terrain::sculpt::SculptGlobals) -> VarDictionary {
    let mut out = VarDictionary::new();
    for c in sculpt_bridge::global_controls() {
        out.set(c.key, sculpt_bridge::get_global(g, c.key).expect("global_controls key must resolve"));
    }
    out
}

/// Applies a `Dictionary` of key -> value through `set_one`, collecting
/// `{rejected, clamped}` the same way `set_params` does for generation
/// parameters. Shared by `sculpt_set_globals` and `sculpt_set_feature_params`
/// so the two `#[func]`s differ only in which `set_one` they pass.
fn apply_sculpt_values(
    values: &VarDictionary,
    mut set_one: impl FnMut(&str, f64) -> sculpt_bridge::Outcome,
) -> VarDictionary {
    let mut rejected = PackedStringArray::new();
    let mut clamped = PackedStringArray::new();
    for (k, v) in values.iter_shared() {
        let key = k.to_string();
        let outcome = match variant_to_num(&v) {
            Some(n) => set_one(&key, n),
            None => sculpt_bridge::Outcome::Rejected,
        };
        match outcome {
            sculpt_bridge::Outcome::Applied => {}
            sculpt_bridge::Outcome::Clamped => clamped.push(&GString::from(&key)),
            sculpt_bridge::Outcome::Rejected => rejected.push(&GString::from(&key)),
        }
    }
    dict! { "rejected" => &rejected, "clamped" => &clamped }
}

/// `MVP_SCOPE.md` points 10-11: basic 2D rendering + minimal UI. Owns the
/// last `generate_terrain()` result (or loaded save); GDScript drives it via
/// `generate()`/`generate_sized()`/`load_save()` then `build_color_texture()`.
///
/// **The grid need not be square.** `cartalith_engine::WorldParams` has
/// always carried independent `gw`/`gh` (and every golden-parity fixture in
/// this workspace is non-square — 14×11, 16×12, 24×18, 20×14, 48×40 — so the
/// whole engine and civ layer is JS-verified at non-square dimensions), but
/// until `generate_sized()` existed this layer's `generate(seed, width_km,
/// resolution)` forced `gh = gw` and threw that capability away. `generate()`
/// keeps doing exactly that (square, bit-identical, and the reason every
/// existing golden test is untouched); `generate_sized()` is the general
/// entry point. A loaded save has always kept whatever `GW`/`GH` it was
/// exported at.
#[derive(GodotClass)]
#[class(base=RefCounted)]
struct WorldGen {
    base: Base<RefCounted>,
    source: Option<WorldSource>,
    gw: i32,
    gh: i32,
    /// The real map width in km of the current world (`WorldParams::
    /// map_width_km` for a generated one, `SaveParams::map_width_km` for a
    /// loaded one). `0.0` before the first generation/load. Stored so
    /// `get_map_width_km`/`get_map_height_km` can report the world actually
    /// on screen rather than making the caller remember what it asked for.
    map_width_km: f64,
    sea_level: f64,
    /// **The persistent generation-parameter state** — every field of
    /// `cartalith_engine::WorldParams` except the three `generate()` takes
    /// as arguments (`gw`/`gh` from `resolution`, `tect.seed` from `seed`,
    /// `map_width_km` from `width_km`), which are overwritten on every call
    /// and therefore never read from here.
    ///
    /// Written by `set_params`/`reset_params` (the flat dotted-key API in
    /// `params.rs`) and by the three legacy convenience setters
    /// (`set_sea_level`, `set_experimental_flags`, both of which now write
    /// straight into this struct so there is exactly one source of truth).
    /// **Values persist between generations** — set once, and every
    /// subsequent `generate()`/`generate_world_structure()` uses them, the
    /// same way `set_sea_level` has always behaved. Initialized to
    /// `WorldParams::defaults(0, 0, 0)`, so a `WorldGen` nobody ever calls a
    /// setter on generates exactly what it generated before this API
    /// existed.
    ///
    /// Note `sea_level` here is the user-facing *input* (reference
    /// `state.seaLevel`, a raw `[0,1]` threshold against the height field's
    /// own `[0,1]` stretch, default `0.42`; the reference's `bind('sea')` UI
    /// is a 0-100% slider dividing by 100 before storage — same convention
    /// here, GDScript converts). It only takes effect when World Structure
    /// is disabled: `generate_world_structure()` (and any
    /// `world_structure.enabled = true`) makes
    /// `apply_world_structure_sea_level` re-anchor sea level from the
    /// archetype's own land-fraction target instead. The sibling `sea_level`
    /// field above tracks the *effective* post-generation value the renderer
    /// needs, not this input.
    params: WorldParams,
    /// `WorldState::gpu_stages_used` from the last generation — which of the
    /// GPU-eligible stages actually ran on GPU (`GPU_LAYER_INTEGRATION_SCOPE.md`
    /// milestone 6+). Empty when `use_gpu` was off *or* when every stage fell
    /// back to CPU, which is exactly the distinction the UI must be able to
    /// report honestly rather than assume from the checkbox.
    gpu_stages_used: Vec<String>,
    /// `latAt`'s inputs (`render.rs`) — `p.world`/`p.climate.lat_n`/`.lat_s`
    /// for a fresh `generate()`, or `save.params.world` + JS's own literal
    /// `climate` defaults (55/5) for a loaded save, whose format doesn't
    /// store latitude band at all (`SAVEFILE_COMPAT.md`).
    world: bool,
    lat_n: f64,
    lat_s: f64,
    /// The three opt-in civ passes, set via `set_villages_enabled` /
    /// `set_metropolis_enabled` / `set_recovery_phase`. Every default is
    /// the reference's own, so an untouched engine generates exactly what
    /// auto-populate generates out of the box.
    civ_options: CivOptions,
    /// Phase 2 civilisation-layer output for the current `Generated`
    /// source (`None` for a `Loaded` save, or before any `generate()`).
    civ: Option<CivData>,
    /// The seed the last `generate()`/`generate_world_structure()` call
    /// used (reference `state.tect.seed`) — threaded into
    /// `pack::composite_map_icons`'s placement hashing so a loaded pack's
    /// scattered icons are as deterministic-per-world as everything else
    /// this port generates. `0` before the first `generate()` call, same as
    /// every other field here.
    seed: i32,
    /// A real, loaded asset pack (`ASSET_LIBRARY_SCOPE.md` milestone 7) —
    /// `None` by default, since this port ships no default pack (verified:
    /// there is nothing in `godot-project/` that ships pack art, matching
    /// the reference's own `assetPack = null` default). Set via
    /// `load_asset_pack`; consumed by `build_color_texture` for both real
    /// sprite compositing and ground-texture splat.
    asset_pack: Option<pack::LoadedPack>,
    /// `TERRAIN_APPEARANCE_RESEARCH.md` §29's quality tier for the *appearance*
    /// pass only (`TERRAIN_APPEARANCE_SCOPE.md` milestone 6). Purely
    /// presentation: it feeds `TerrainAppearance::for_tier` inside
    /// `build_color_texture` and touches nothing the world model computes, so
    /// changing it and re-calling `build_color_texture()` re-renders the same
    /// world at a different cost -- research §23's "do not regenerate the
    /// entire world whenever a visual parameter changes", satisfied literally.
    ///
    /// Defaults to `Quality`, which is `TerrainAppearance::default()`
    /// bit-for-bit, so an untouched `WorldGen` renders exactly what it
    /// rendered before this field existed. **Deliberately not
    /// `recommended_quality_tier()`**: what a phone should default to is an
    /// owner policy decision, and `get_recommended_quality_tier()` exists so a
    /// caller can offer one rather than have it chosen for them.
    quality: QualityTier,
    /// The active **named look** (`render::LOOK_PRESETS`) — the colour,
    /// chroma, light-shaping and grade layer that sits on top of the quality
    /// tier. `render::LOOK_VIBRANT` on a fresh session (2026-08-24, owner
    /// instruction); `render::LOOK_TIER` is the identity and is what
    /// `reset_appearance` restores.
    ///
    /// A separate authority from `quality` for the reason `with_look`'s own
    /// doc gives: the tier decides what the renderer spends, the look decides
    /// what the picture is, and a phone answers only the first question
    /// differently. Purely presentation, on `set_quality_tier`'s exact terms.
    look: String,
    /// The reference's non-photorealistic block (`render::Npr`,
    /// `GUI_GAP_REGISTER.md` RN-01): the ten "Painter" hand-drawn styles, the
    /// coastal wave lines, the multi-sun rig and the animated-water flag.
    ///
    /// Every member is off by default, so an untouched `WorldGen` renders
    /// exactly what it rendered before this field existed. Purely
    /// presentation, on the same terms as `quality` above: it feeds
    /// `TerrainAppearance` inside `build_color_texture` and marks no
    /// generation stage stale, so `set_npr()` + `build_color_texture()`
    /// re-renders the same world in a different style with no regeneration.
    npr: render::Npr,
    /// User overrides over the quality tier's own appearance values
    /// (`GUI_GAP_REGISTER.md` CA-01/RN-01, the reference's Cartography ▸ Map
    /// view and Rendering-advanced blocks), keyed by
    /// `render::TerrainAppearance::TUNABLE`.
    ///
    /// **An override map, not a `TerrainAppearance`.** The tier ladder and the
    /// user's edits are two different authorities over the same struct, and
    /// storing the merged result would mean switching quality tier silently
    /// threw the user's sun azimuth away (or, worse, kept it and quietly
    /// undid the tier). Holding only the deltas lets `appearance()` layer them
    /// in one direction — tier first, user second — so both survive.
    ///
    /// Empty by default, so an untouched `WorldGen` renders exactly what it
    /// rendered before this field existed. Purely presentation, on
    /// `set_quality_tier`'s exact terms: nothing here touches the heightmap,
    /// climate, hydrology, biomes, settlements, routes or the seed.
    appearance_over: std::collections::HashMap<String, f64>,
    /// The caller's own elevation colour ramp (`GUI_GAP_REGISTER.md` CA-02),
    /// or `None` for whatever the layer beneath supplies.
    ///
    /// Its own field rather than a key in `appearance_over` for the obvious
    /// reason — that map is `f64`-valued — but also for the same reason it
    /// exists at all: the ramp is a *separate authority* from the tier and
    /// from the preset, so editing stops must not discard a loaded preset's
    /// sun azimuth and switching quality tier must not discard the stops.
    appearance_ramp: Option<render::ElevationRamp>,
    /// A loaded appearance preset (`GUI_GAP_REGISTER.md` CA-08), replacing the
    /// **quality tier** as the base layer that `appearance_over` and
    /// `appearance_ramp` sit on top of. `None` = the tier, which is what every
    /// session starts as and what `reset_appearance` restores.
    ///
    /// A whole `TerrainAppearance` rather than a second override map, and the
    /// difference matters: a preset is a *complete* description of a look
    /// (`ARCHITECTURE.md`'s presentation layer, serialized), so loading one
    /// must reproduce it exactly rather than merging it into whatever the
    /// session happened to be showing. `load_appearance_preset` therefore also
    /// clears the override map — see its own doc.
    ///
    /// Its cost is the tier's: a preset saved at `Ultra` and loaded on a phone
    /// really does render at `Ultra`, because that is what the file says. A
    /// caller who wants the tier's cost back calls `reset_appearance`.
    appearance_preset: Option<TerrainAppearance>,
    /// `UNIFIED_TOOL_PLAN.md` milestone F (`STRANDED_TOOLS.md` rows 4-8):
    /// the live, non-destructive Sculpt-editor draft. See
    /// `sculpt_bridge.rs`'s own module doc for why this lives here rather
    /// than a second `GodotClass`. `None` before the first successful
    /// `generate()`/`generate_sized()` call, and after `load_save()` — a
    /// draft only ever exists over a freshly generated `WorldState` (same
    /// restriction `civ` already has: a loaded save's format carries no
    /// `river_mask`/`river_floor` for the water hooks to adopt, the same
    /// reason `SAVEFILE_COMPAT.md` gives for civ data never existing on
    /// one either).
    sculpt: Option<sculpt_bridge::SculptEditor>,
    /// `UNIFIED_TOOL_PLAN.md` milestone F, the CARTO domain's Icon tool
    /// (`DCC_SHELL_SPEC.md` §4.5.5) — every hand-placed icon plus the
    /// currently-armed gallery selection. See `icon_bridge.rs`'s own module
    /// doc for why this lives here rather than a second `GodotClass`, the
    /// same reasoning `sculpt` above already follows. `None` before the
    /// first successful `generate()`/`generate_sized()` call, and after
    /// `load_save()` — a loaded save's format carries no manual-icon list
    /// at all (`SAVEFILE_COMPAT.md` has no `mapIcons` equivalent), the same
    /// restriction `civ` and `sculpt` already have for the same reason.
    icons: Option<icon_bridge::IconEditor>,
    /// `UNIFIED_TOOL_PLAN.md` milestone F, the CIVIL domain's Settlement and
    /// Territory tools (`DCC_SHELL_SPEC.md` §4.5.3) — the territory-paint
    /// draft/accumulator and the manual-placement name/population RNG
    /// stream. See `civ_tools_bridge.rs`'s own module doc for why this
    /// lives here rather than a second `GodotClass`, the same reasoning
    /// `sculpt`/`icons` above already follow. `None` before the first
    /// successful `generate()`/`generate_sized()` call, and after
    /// `load_save()` — same restriction `civ` itself already has (a loaded
    /// save carries none of the substrate the civ pipeline needs, so there
    /// is no `territory` to paint over in the first place).
    civ_tools: Option<civ_tools_bridge::CivTools>,
    /// `UNIFIED_TOOL_PLAN.md` milestone F, the WORLD domain's Biome/Terrain/
    /// Splat paint tool (`DCC_SHELL_SPEC.md` §4.5.2's `PAINT · BIOME` tool
    /// options row). See `paint_bridge.rs`'s own module doc for why this
    /// lives here rather than a second `GodotClass`, the same reasoning
    /// `sculpt`/`icons`/`civ_tools` above already follow. `None` before the
    /// first successful `generate()`/`generate_sized()` call, and after
    /// `load_save()` — the land-only gate needs this world's own
    /// water-body classification, which only a freshly generated
    /// `WorldState`'s `field`/`rainfall` can compute
    /// (`paint_bridge::PaintEditor::new`'s own doc comment).
    paint: Option<paint_bridge::PaintEditor>,
    /// `UNIFIED_TOOL_PLAN.md` milestone F, the CARTO domain's Label tool
    /// (`DCC_SHELL_SPEC.md` §4.5.5: "Click an empty spot creates a label;
    /// click an existing one edits it in place"). See `label_bridge.rs`'s
    /// own module doc for why this lives here rather than a second
    /// `GodotClass`, the same reasoning `sculpt`/`icons`/`civ_tools`/
    /// `paint` above already follow. `None` before the first successful
    /// `generate()`/`generate_sized()` call, and after `load_save()` — a
    /// label's own `x`/`y` are grid coordinates over one particular world,
    /// meaningless carried over to a differently-sized (or entirely
    /// different) one, the same restriction `icons` already has for the
    /// same reason.
    labels: Option<label_bridge::LabelBridge>,
    /// `UNIFIED_TOOL_PLAN.md` milestone F, the INFRA domain's Way/Route
    /// tools plus the Measure and Region select global tools
    /// (`DCC_SHELL_SPEC.md` §4.5.1/§4.5.4) — the in-progress way/route
    /// draft, every committed hand-drawn way/route, the Measure click
    /// chain, and the Region select marquee. See `infra_tools_bridge.rs`'s
    /// own module doc for why all four share one struct and live here
    /// rather than a second `GodotClass`, the same reasoning `sculpt`/
    /// `icons`/`civ_tools`/`paint`/`labels` above already follow. `None`
    /// before the first successful `generate()`/`generate_sized()` call,
    /// and after `load_save()` — a waypoint, measurement or marquee from
    /// the *previous* world's grid dimensions is meaningless (and, for the
    /// marquee, could sit outside the new grid's bounds entirely), the
    /// same restriction every sibling field above already has.
    infra: Option<infra_tools_bridge::InfraTools>,
    /// `TRAVEL_LIBRARY_SPEC.md`: user-editable project state, not
    /// civ-generation output -- unlike `civ`/`sculpt`/`icons`/`civ_tools`/
    /// `paint`/`labels`/`infra` above, this field is **not reset by
    /// `absorb()`**. It survives a re-generate the same way `asset_pack`
    /// and `quality` already do (loaded/set once, independent of any one
    /// `WorldState`), because a custom animal/vehicle/vessel/party-preset
    /// definition describes the *world's setting*, not something the
    /// terrain pipeline produced -- regenerating the map has no more
    /// reason to discard a hand-defined pack mule than it does to discard
    /// a loaded asset pack. Bootstrapped with stock content in `init()`
    /// below, so it is real and queryable even before the first
    /// `generate()` call, unlike every `Option<...>` field above that
    /// needs a generated world to mean anything.
    travel_library: travel_bridge::TravelLibrary,
    /// `ASSET_LIBRARY_SCOPE.md` / `GUI_GAP_REGISTER.md` AS-01..AS-08/AS-13,
    /// DM-05: the live, in-memory Asset Library authoring session (`AssetDB`
    /// plus its decoded pixels) that `asset_library_window.gd` edits. Not
    /// `Option` and not reset by `absorb()` — same reasoning
    /// `travel_library` above already carries: an authored library describes
    /// the *setting*, not one generation's output, so it survives a
    /// re-generate and is real even before the first `generate()` call.
    asset_library: asset_bridge::AssetLibrarySession,
    /// `state.viz.territoryOpacity` — the territory wash's own alpha, `0..1`
    /// (`GUI_GAP_REGISTER.md` **CA-17**, the reference's `#territoryOpacityR`).
    ///
    /// A **display** setting, not generation output: like `travel_library`
    /// and `asset_library` above it is not `Option` and survives `absorb()`,
    /// because how heavily territory is washed over the map is a choice about
    /// the sheet rather than something the terrain pipeline produced.
    /// [`TERRITORY_ALPHA_DEFAULT`] is this port's own long-standing 82/255,
    /// so at rest `build_territory_texture` is byte-identical to what it drew
    /// before this field existed.
    territory_opacity: f64,
    /// The reference's global heightmap undo stack (`pushUndo`/`undoLast`,
    /// `PARITY_AUDIT.md` §3.1, register `ED-01`/`PR-11`) — a bounded ring of
    /// pre-operation `field` snapshots. See `undo.rs` for what the reference
    /// snapshots, how deep it goes, and the three places this port
    /// deliberately diverges (a byte budget, a clear-on-generate, and no
    /// inline flow/climate recompute).
    ///
    /// Not `Option`: an empty stack is already the "nothing to undo" state,
    /// so there is no world-dependent construction to defer, unlike
    /// `sculpt`/`icons`/`paint` above. **Is** cleared by `absorb()` and by
    /// `load_save()` — a snapshot of the previous world's field is
    /// meaningless over a new one, and at a different resolution it is not
    /// even the right length.
    undo: undo::HeightUndo,
    /// `GUI_GAP_REGISTER.md` **ED-02** — the history ledger over every
    /// committed operation, not only the reversible ones. See
    /// `undo::HistoryLedger` for why it records more than `undo` can revert.
    ledger: undo::HistoryLedger,
    /// The live pipeline staleness graph (`cartalith_engine::staleness::
    /// pipeline_stage_graph`): height → hydrology → climate → civ, over the
    /// same tiling the Sculpt draft's `PassBuffer`/`DirtyTracker` pair uses,
    /// so a `CommitSummary::tiles_marked` drops straight into
    /// `mark_changed_tiles` without a re-tiling step.
    ///
    /// Rebuilt by `absorb()` (a graph over the previous world's tile count is
    /// the wrong size for this one, the same reason `sculpt` and `undo` are
    /// replaced there). Written by the commit paths that actually invalidate
    /// something — `sculpt_commit` and `carve_fjords` mark
    /// `PipelineStage::Height`, `paint_commit` marks `Civ` — and consumed by
    /// `cartalith_engine::staleness::recompute_stale`, which is what makes an
    /// edit reach rainfall and discharge instead of stopping at the height
    /// field.
    ///
    /// Not `Option`: a zero-world graph is a legal, harmless empty graph, so
    /// there is nothing to defer.
    stages: cartalith_spatial::StageGraph,
    /// `GUI_GAP_REGISTER.md` **SG-01**: has a settlement been added, edited or
    /// deleted since the civ layer was last derived?
    ///
    /// A flag rather than a `stages` mark because the staleness graph
    /// genuinely cannot express this state. `civ` is the leaf, and a manual
    /// place edit changes `civ`'s *own* data — `mark_changed(Civ)` therefore
    /// marks nothing stale at all (`staleness.rs`'s
    /// `a_downstream_only_edit_recomputes_nothing_upstream_of_it`), and
    /// marking any upstream node instead would be a lie that also drags a
    /// pointless `refresh_climate` along. What is out of date after a drop or
    /// a delete is everything `compute_civilisation` *derives from* the
    /// settlement list — roads, territory, provinces, trade balances — which
    /// is the same node, one pass later.
    ///
    /// Set by `civ_drop_settlement`/`civ_edit_settlement`/
    /// `civ_delete_settlement` (`ED-03d`'s own three), cleared by
    /// `recompute_civilisation` and by `absorb` (a freshly generated world's
    /// civ layer is derived from its own settlements by construction). Read
    /// only by `stale_stages`, so the indicator does not claim "up to date"
    /// straight after a hand-dropped town.
    civ_dirty: bool,
    /// The LOD tile-pyramid bake, its persistent atlas, and the finalize lock
    /// (`GUI_GAP_REGISTER.md` WW-01/PR-10/S4/S5, SH-07's `atlas` slot). See
    /// `bake_bridge.rs`'s own module doc.
    ///
    /// **Not reset by `absorb()`**, and that is the point rather than an
    /// oversight: the atlas root is a machine-level preference and the atlas
    /// itself is keyed by `atlas_world_key()`, so a regenerate simply moves to
    /// a different key namespace and the previous world's chunks stay on disk,
    /// intact, for a caller that regenerates back to the same parameters. What
    /// *is* cleared there is `finalized` — see `absorb`.
    bake: bake_bridge::BakeState,
    /// The Markdown Vault session (`vault_bridge.rs`,
    /// `MARKDOWN_VAULT_SCOPE.md` milestone 1) — knowledge links, and the
    /// device-local directory binding when this device has one.
    ///
    /// **Not reset by `absorb()`**, deliberately, and for the same reason
    /// `bake` above is not: a vault binding is a machine-level preference and
    /// the links are the *user's* filing, not a derived product of the world.
    /// Regenerating a world does not un-write the notes it was linked to.
    /// What regenerating *does* invalidate is the entity ids those links
    /// point at, which is why every link also stores the entity's name — see
    /// `cartalith_vault::links`' own module doc on identity stability.
    vault: cartalith_vault::VaultSession,
}

#[godot_api]
impl IRefCounted for WorldGen {
    fn init(base: Base<RefCounted>) -> Self {
        Self {
            base,
            source: None,
            gw: 0,
            gh: 0,
            map_width_km: 0.0,
            sea_level: 0.42,
            params: params::defaults(),
            gpu_stages_used: Vec::new(),
            world: false,
            lat_n: 55.0,
            lat_s: 5.0,
            civ_options: CivOptions::default(),
            civ: None,
            seed: 0,
            asset_pack: None,
            quality: QualityTier::Quality,
            look: render::LOOK_VIBRANT.to_string(),
            // Multi-sun on, as the shipped default (2026-08-24). It lives on
            // the `Npr` block rather than in the look because that is where
            // the reference keeps it and where `set_npr`/the Painter panel
            // read and write it; seeding it here rather than in
            // `Npr::default()` is what keeps `js_reference()` — which inherits
            // its `npr` from `Default` — the reference's single-sun shading.
            npr: render::Npr { multi_sun: true, ..render::Npr::default() },
            appearance_over: std::collections::HashMap::new(),
            appearance_ramp: None,
            appearance_preset: None,
            sculpt: None,
            icons: None,
            civ_tools: None,
            paint: None,
            labels: None,
            infra: None,
            travel_library: travel_bridge::TravelLibrary::new(),
            asset_library: asset_bridge::AssetLibrarySession::new(),
            territory_opacity: TERRITORY_ALPHA_DEFAULT,
            undo: undo::HeightUndo::new(),
            ledger: undo::HistoryLedger::new(),
            stages: pipeline_stage_graph(1),
            civ_dirty: false,
            bake: bake_bridge::BakeState::new(),
            vault: cartalith_vault::VaultSession::new(),
        }
    }
}

/// Plain (non-`#[func]`) helpers shared by `generate()` and
/// `generate_world_structure()` — kept out of the `#[godot_api]` block since
/// they are Rust-internal, not part of the GDScript surface.
impl WorldGen {
    /// This instance's persistent parameters with the four call-argument
    /// fields filled in. The single place `gw`/`gh`/`seed`/`map_width_km`
    /// enter a `WorldParams` — everything else comes from `self.params`
    /// unchanged, which is why an untouched `WorldGen` generates exactly what
    /// it did before the parameter API existed.
    ///
    /// `width_km` is the **width** of the map. The engine derives its cell
    /// size from `map_width_km / gw` alone (`cartalith_terrain::
    /// terrain_detail_k`, `river_flow_thresh`, `civ_catchment_radius_cells`,
    /// `suppression_radius_cells` — every km↔cell conversion in the
    /// workspace goes through that one quotient and is applied isotropically),
    /// so **cells are square in km and the map's real height is
    /// `width_km * gh / gw`** — derived, never independently settable. See
    /// `get_map_height_km`.
    fn call_params(&self, seed: i32, width_km: f64, gw: usize, gh: usize) -> WorldParams {
        let mut p = self.params.clone();
        p.gw = gw;
        p.gh = gh;
        p.tect.seed = seed;
        p.map_width_km = if width_km > 0.0 { width_km } else { 800.0 };
        p
    }

    /// The `WorldParams` describing the world **currently loaded**, for a
    /// post-generation recompute — `self.params` (the persistent dial state)
    /// with this world's own `gw`/`gh`/`seed`/`map_width_km` filled back in,
    /// which `call_params` already is the single place for.
    ///
    /// `world` is then pinned to the value `absorb()` snapshotted rather than
    /// read from `self.params`: it decides whether the grid wraps in
    /// longitude, so a dial moved after generation must not make a recompute
    /// describe a different world's *geometry*. The climate dials are
    /// deliberately **not** pinned — re-running climate with the current
    /// rainfall/temperature settings is the point of asking for a recompute.
    fn recompute_params(&self) -> WorldParams {
        let mut p = self.call_params(
            self.seed,
            self.map_width_km,
            self.gw.max(0) as usize,
            self.gh.max(0) as usize,
        );
        p.world = self.world;
        p
    }

    /// Marks `stage` changed at `tiles`, then re-runs whatever that
    /// invalidated (`cartalith_engine::staleness::recompute_stale`).
    ///
    /// This is the one function the commit paths share, so "an edit actually
    /// takes effect" is decided in a single place rather than restated per
    /// tool. Returns `(ran, still_stale)` as the two `PackedStringArray`s the
    /// callers put in their summary dictionaries.
    fn mark_and_recompute(
        &mut self,
        stage: PipelineStage,
        tiles: impl IntoIterator<Item = usize>,
        reason: &str,
    ) -> (PackedStringArray, PackedStringArray) {
        // `DirtyTracker` indexes by tile, so an out-of-range id would panic
        // -- and this runs under a `#[func]`, where a panic takes the Godot
        // process with it. Every producer of these ids tiles at 64 px over
        // the same grid, so the filter should never drop anything; it is
        // here so that the day one of them doesn't, the result is a missed
        // mark rather than a crash.
        let n = self.stages.tile_count();
        self.stages.mark_changed_tiles(stage.id(), tiles.into_iter().filter(|&t| t < n), reason);
        let p = self.recompute_params();
        let Some(WorldSource::Generated(ws)) = self.source.as_mut() else {
            return (PackedStringArray::new(), PackedStringArray::new());
        };
        let r = recompute_stale(&mut self.stages, &p, ws);
        let names = |v: Vec<&'static str>| -> PackedStringArray { v.into_iter().map(GString::from).collect() };
        (names(r.ran), names(r.still_stale))
    }

    /// `GUI_GAP_REGISTER.md` **SG-03**: records that a parameter moved, over
    /// the whole map — a dial is global, unlike a brush stroke's tile set.
    ///
    /// [`params::invalidates`] decides `stage`; this only does the marking,
    /// and deliberately does **not** recompute. `world_workspace.gd` writes a
    /// slider's value on every drag tick, so a recompute here would run
    /// `refresh_climate` sixty times a second; the recompute is somebody
    /// else's decision (`recompute_stale_stages`, `recompute_civilisation`,
    /// or the next commit path).
    ///
    /// Silent with no world: a graph over a world that does not exist yet
    /// describes nothing, and every dial in the New World dialog is set
    /// before the first `generate()`.
    fn mark_param_change(&mut self, stage: PipelineStage, reason: &str) {
        if self.source.is_none() {
            return;
        }
        let n = self.stages.tile_count();
        self.stages.mark_changed_tiles(stage.id(), 0..n, reason);
    }

    /// Stores a finished generation: the effective sea level, the render
    /// inputs `render.rs` needs, the civ layer, and which stages actually ran
    /// on GPU.
    fn absorb(&mut self, ws: cartalith_engine::WorldState, p: &WorldParams, seed: i32) {
        // Not `p.sea_level` -- World-Structure archetypes re-anchor it;
        // `WorldState` carries the value actually used.
        self.sea_level = ws.sea_level;
        self.gw = p.gw as i32;
        self.gh = p.gh as i32;
        self.map_width_km = p.map_width_km;
        self.world = p.world;
        self.lat_n = p.climate.lat_n;
        self.lat_s = p.climate.lat_s;
        self.gpu_stages_used = ws.gpu_stages_used.clone();
        self.civ = Some(compute_civilisation(&ws, p.gw, p.gh, p.world, p.map_width_km, p.river_density, self.civ_options, None));
        // Milestone F: a fresh Sculpt draft over this world's own
        // dimensions, seeding the water hooks from whatever
        // carve_river_valleys already locked during generation
        // (`ws.river_mask`/`river_floor`, `None` when `carve_rivers` was
        // off) so a hand-painted river's re-clamp step (`SCULPT_FUNCTION_
        // CHART.md` §7) protects generated channels too, not just ones
        // painted in this session.
        self.sculpt = Some(sculpt_bridge::SculptEditor::new(
            p.gw,
            p.gh,
            ws.river_mask.clone(),
            ws.river_floor.clone(),
            seed as u32,
        ));
        // Milestone F: a fresh, empty Icon editor over this world -- a
        // previous world's placed icons carry grid coordinates meaningless
        // over a differently-sized (or entirely different) generation, the
        // same reasoning `SculptEditor::new` above already follows for its
        // own draft.
        self.icons = Some(icon_bridge::IconEditor::new());
        // Milestone F, CIVIL group: a fresh territory-paint draft/accumulator
        // seeded from THIS generation's own `assign_territory` output (the
        // pristine base `civ_tools_bridge::CivTools::commit` always rebuilds
        // from) and a manual-placement name/population RNG stream folded
        // from this world's own seed -- see `civ_tools_bridge.rs`'s own
        // module doc for why both exist. `self.civ` was just set above, so
        // its `territory` is real here.
        self.civ_tools = Some(civ_tools_bridge::CivTools::new(
            p.gw,
            p.gh,
            self.civ.as_ref().expect("just set above").territory.clone(),
            seed as u32,
        ));
        // Milestone F, WORLD group: a fresh Paint editor over this world.
        // The land-only gate (`paint_bridge::PaintEditor`'s own module doc)
        // needs this world's water-body classification. This used to call
        // `build_water_bodies` a second time, on the reasoning that
        // `compute_civilisation` "never retains it past its own local scope" --
        // which stopped being true the moment `CivData::water_bodies` was added
        // to hold exactly that array. The second call was **not** cheap:
        // measured **417 ms at 2048x2048** (95 ms at 1024, 22 ms at 512), a
        // fully sequential priority-flood plus flood fill, ~7 % of a whole
        // generate -- `CPU_MULTITHREADING_SCOPE.md`'s 2026-08-19 investigation
        // found and recorded it and deliberately left it for a later pass.
        // `self.civ` is set unconditionally a few lines above, so this reads
        // the same world's own answer rather than recomputing it.
        let wb_class = &self.civ.as_ref().expect("just set above").water_bodies;
        self.paint = Some(paint_bridge::PaintEditor::new(p.gw, p.gh, std::sync::Arc::from(wb_class.as_slice())));
        // Milestone F, CARTO group: a fresh, empty Label bridge over this
        // world -- same reasoning `self.icons` above already follows (grid
        // coordinates from a previous generation are meaningless here).
        self.labels = Some(label_bridge::LabelBridge::new());
        // Milestone F, INFRA group + the two global tools: a fresh, empty
        // tool set over this world. Unlike `sculpt`/`civ_tools`/`paint`
        // above, nothing here adopts anything from `ws` -- Way, Route,
        // Measure and Region select are all user-driven from the first
        // click, so `InfraTools::new()` alone is the right starting state
        // (see `infra_tools_bridge.rs`'s own module doc, "nothing here is
        // computed at construction").
        self.infra = Some(infra_tools_bridge::InfraTools::new());
        // A fresh staleness graph over this world's own tiling. Sized from
        // the Sculpt draft's `PassBuffer` rather than recomputed here, so the
        // tile indices in a `CommitSummary::tiles_marked` mean the same thing
        // in both -- there is exactly one tiling, not two that agree by
        // coincidence. Everything starts current: a world that has just
        // finished generating is, by definition, not stale.
        self.stages = pipeline_stage_graph(
            self.sculpt.as_ref().expect("just set above").draft.tile_count(),
        );
        // SG-01: this world's civ layer was derived from this world's own
        // settlements, three lines up. Nothing is pending.
        self.civ_dirty = false;
        // Global heightmap undo: a snapshot of the *previous* world's field
        // is meaningless over this one, and at a different resolution it is
        // not even the right length. The reference does not clear here (its
        // grid cannot change size mid-session); this port's `generate_sized`
        // can, so it must. See `undo.rs`'s divergence list.
        self.undo.clear();
        // ED-02: a generate is the ledger's **floor**, and clears it for the
        // same reason `undo.clear()` above does -- nothing before it can be
        // reverted to, so drawing it would be an offer the engine cannot
        // keep.
        // `seed` is the argument, not `self.seed`: this runs a few lines
        // before the field is assigned, and reading the field here reported
        // `seed 0` on screen against a status bar saying 483920. Caught by
        // the windowed probe's own screenshot, which is the only place the
        // two are visible together.
        self.ledger.record(
            "world",
            "Generate world",
            format!("seed {} - {} x {}", seed, self.gw, self.gh),
            undo::EntryKind::Floor,
        );
        // The finalize lock is a statement about *this* world: "its atlas is
        // baked and its parameters are frozen". A new world has no atlas, so
        // it cannot be finalized, and carrying the flag across would lock a
        // freshly generated world out of being edited for a bake it never
        // had. The atlas *root* and *tile size* survive, being machine-level
        // settings rather than world state -- see the `bake` field's own note.
        //
        // Nothing is deleted here. The previous world's chunks stay on disk
        // under their own `atlas_world_key()`, so regenerating back to the
        // same parameters finds them again rather than re-baking.
        self.bake.finalized = false;
        self.source = Some(WorldSource::Generated(Box::new(ws)));
        self.seed = seed;
    }
}

/// `gridH(gw)` (reference HTML line 5049): the reference app's own grid
/// height for a chosen working resolution — `round(gw * 0.5)` in world mode
/// (2:1 equirectangular) and `round(gw * 0.64)` in region mode (a 1.5625:1
/// frame). **The reference's maps are never square**; this port's square
/// default came from `generate()`'s single `resolution` argument, not from
/// anything the reference does.
///
/// Exposed so a setup dialog gets it from the engine side rather than
/// hardcoding `0.64` a second time in GDScript (`ARCHITECTURE.md`: "Godot
/// computes nothing beyond layout").
///
/// One deliberate deviation from the reference: floored at 4, matching
/// `generate_sized`'s own minimum, so this can never hand back a grid the
/// generator would then clamp behind the caller's back.
fn reference_grid_h(gw: usize, world: bool) -> usize {
    let k = if world { 0.5 } else { 0.64 };
    ((gw as f64 * k).round() as usize).max(4)
}

#[godot_api]
impl WorldGen {
    /// Sets the four golden-verified subsystem flags this instance's
    /// `generate()`/`generate_world_structure()` calls apply from then on.
    ///
    /// Kept as its own `#[func]` because `main.gd` already drives it, but it
    /// is now pure sugar over `set_params` — exactly equivalent to
    /// `set_params({"tect.dynamic_lithology": …, "volc.provinces": …,
    /// "climate.terrain_wind_deflection": …, "climate.currents": …})`, and
    /// `get_params()` reads the same four values back.
    #[func]
    fn set_experimental_flags(
        &mut self,
        dynamic_lithology: bool,
        volc_provinces: bool,
        terrain_wind_deflection: bool,
        ocean_currents: bool,
    ) {
        self.params.tect.dynamic_lithology = dynamic_lithology;
        self.params.volc.provinces = volc_provinces;
        self.params.climate.terrain_wind_deflection = terrain_wind_deflection;
        self.params.climate.currents = ocean_currents;
    }

    /// Every generation parameter's **current** value, as a flat
    /// `Dictionary` of dotted keys (`"sea_level"`, `"tect.plates"`,
    /// `"climate.lat_n"`, …) — the exact keys `set_params` accepts and
    /// `get_param_info` describes. Booleans come back as `bool`, integer
    /// parameters as `int`, everything else as `float`.
    ///
    /// This is real current state, not assumed defaults: a dialog should
    /// populate itself from here, and re-read after `set_params` to see what
    /// was actually stored (a clamped value differs from what was sent).
    #[func]
    fn get_params(&self) -> VarDictionary {
        params_to_dict(&self.params)
    }

    /// The same shape as `get_params()`, but every value at its
    /// `cartalith_engine::WorldParams::defaults` setting — what a
    /// "reset to default" control should show. Never affected by anything
    /// this instance has been set to.
    #[func]
    fn get_param_defaults(&self) -> VarDictionary {
        params_to_dict(&params::defaults())
    }

    /// Metadata for building parameter dialogs without hardcoding any of it
    /// in GDScript: dotted key -> `Dictionary` with
    /// - `group` (String) — the dialog section, one of `get_param_groups()`,
    /// - `type` (String) — `"bool"` / `"int"` / `"float"`,
    /// - `default` (bool/int/float) — same value `get_param_defaults()` has,
    /// - `min`, `max`, `step` (float) — the valid range and the reference
    ///   control's own step, already in this parameter's units,
    /// - `label` (String) — the reference's own control label where it has
    ///   one,
    /// - `unit` (String) — display suffix (`"°"`, `"h"`, `"m"`, `"×"`), `""`
    ///   when unitless,
    /// - `reference_control` (String) — the reference HTML element id this
    ///   parameter's user control has, or `""` when the reference never
    ///   exposed it (see `GENERATION_PARAMETERS.md` — such a parameter is a
    ///   deliberate superset, not a parity claim).
    ///
    /// `min`/`max`/`step` for every parameter the reference *does* expose are
    /// that control's real reachable range, converted through its own mapping
    /// function — not plausible-looking invented numbers.
    #[func]
    fn get_param_info(&self) -> VarDictionary {
        let defaults = params::defaults();
        let mut out = VarDictionary::new();
        for s in params::PARAMS {
            let d: VarDictionary = vdict! {
                "group" => s.group,
                "type" => s.kind.as_str(),
                "default" => &value_to_variant(s.kind, params::get(&defaults, s.key).expect("every table key resolves against its own table")),
                "min" => s.min,
                "max" => s.max,
                "step" => s.step,
                "label" => s.label,
                "unit" => s.unit,
                "reference_control" => s.reference_control,
            };
            out.set(s.key, &d);
        }
        out
    }

    /// The distinct parameter groups, in the order a dialog should show its
    /// sections: `["world", "planet", "world_structure", "tectonics",
    /// "volcanism", "erosion", "climate", "weather"]`. Each matches a real
    /// panel heading in the reference HTML's own sidebar.
    #[func]
    fn get_param_groups(&self) -> PackedStringArray {
        params::groups().into_iter().map(GString::from).collect()
    }

    /// Applies a partial `Dictionary` of dotted key -> value. Keys not
    /// present are left alone, so a dialog can send only what its user
    /// touched. **Values persist on this `WorldGen` between generations** —
    /// every subsequent `generate()`/`generate_world_structure()` uses them
    /// until `reset_params()` or another `set_params()` changes them, the
    /// same way `set_sea_level`/`set_villages_enabled` have always behaved.
    ///
    /// Returns a report so the caller never has to guess what happened:
    /// - `rejected` (`PackedStringArray`) — keys **not** applied: unknown
    ///   key, wrong value type (a `bool` parameter given a number, or a
    ///   numeric parameter given a String/Array/…), or a non-finite number.
    /// - `clamped` (`PackedStringArray`) — keys applied but **adjusted**: an
    ///   out-of-range value pulled to the nearest bound, or a fractional
    ///   value rounded for an `int` parameter. Read `get_params()` back for
    ///   the stored value.
    ///
    /// Both empty means every key applied exactly as sent. Rejections are
    /// also printed to the Godot console — a typo'd key in a dialog is a bug
    /// worth seeing, not something to swallow.
    ///
    /// **Marks the staleness graph** (`GUI_GAP_REGISTER.md` SG-03) for the 25
    /// keys that have a live-apply path — see [`params::invalidates`] for the
    /// table and for why the other 56 mark nothing. Marking only: no stage is
    /// recomputed here, because a slider writes on every drag tick.
    #[func]
    fn set_params(&mut self, values: VarDictionary) -> VarDictionary {
        // Every row of this table is in the world key (`bake_bridge.rs`), so
        // a finalized world rejects the whole write rather than half of it --
        // a partial apply would leave the parameter state describing a world
        // the atlas was not baked from, which is the exact condition the lock
        // exists to prevent. `rejected` names every key, so the caller can
        // report it the same way it reports a bad key.
        if let Err(msg) = self.bake.check(cartalith_engine::bake::Mutation::Generation) {
            godot_print!("cartalith-godot: set_params refused -- {msg}");
            let mut out = VarDictionary::new();
            let mut all: PackedStringArray = PackedStringArray::new();
            for (k, _) in values.iter_shared() {
                all.push(&GString::from(k.to_string().as_str()));
            }
            out.set("rejected", &all);
            out.set("clamped", &PackedStringArray::new());
            return out;
        }
        let mut rejected: PackedStringArray = PackedStringArray::new();
        let mut clamped: PackedStringArray = PackedStringArray::new();
        for (k, v) in values.iter_shared() {
            let key = k.to_string();
            let outcome = match variant_to_value(&v) {
                Some(value) => params::set(&mut self.params, &key, value),
                None => params::Outcome::Rejected,
            };
            match outcome {
                params::Outcome::Applied => {}
                params::Outcome::Clamped => clamped.push(&GString::from(&key)),
                params::Outcome::Rejected => {
                    godot_print!("cartalith-godot: set_params rejected '{key}' (unknown key, wrong type, or non-finite)");
                    rejected.push(&GString::from(&key));
                }
            }
            // SG-03. A rejected key wrote nothing, so it invalidates nothing;
            // a clamped one wrote a *different* value than asked for, which
            // still moves the stage.
            if outcome != params::Outcome::Rejected
                && let Some(stage) = params::invalidates(&key)
            {
                self.mark_param_change(stage, &format!("param:{key}"));
            }
        }
        dict! { "rejected" => &rejected, "clamped" => &clamped }
    }

    /// Restores every generation parameter to its
    /// `cartalith_engine::WorldParams::defaults` value — the real
    /// "reset to defaults" action, not a GDScript re-send of remembered
    /// numbers. Does not touch `set_villages_enabled` (civ-layer, not a
    /// `WorldParams` field) or anything about an already-generated world.
    ///
    /// Marks the same two stages a whole-table rewrite can invalidate
    /// (SG-03): `Hydrology` for the climate half, `Climate` for
    /// `river_density`. Cheaper than asking [`params::invalidates`] key by key
    /// and exactly as accurate — a reset moves every dial at once.
    #[func]
    fn reset_params(&mut self) {
        self.params = params::defaults();
        self.mark_param_change(PipelineStage::Hydrology, "reset_params");
        self.mark_param_change(PipelineStage::Climate, "reset_params");
    }

    /// Which GPU-eligible stages actually ran on GPU during the last
    /// `generate()`/`generate_world_structure()` — a subset of
    /// `["warp", "heterogeneity", "plate_assignment", "base_field_blur",
    /// "weather", "flow"]`.
    ///
    /// Read-only, and deliberately not derivable from the `use_gpu`
    /// parameter: every stage falls back to CPU **individually** on any GPU
    /// init/dispatch failure (`HARDWARE_ACCELERATION.md` §27), so an empty
    /// array with `use_gpu` on means "asked for GPU, got none" — which the
    /// UI should report honestly rather than claiming acceleration it did not
    /// get. Empty before the first generation and after `load_save()`.
    #[func]
    fn get_gpu_stages_used(&self) -> PackedStringArray {
        self.gpu_stages_used.iter().map(GString::from).collect()
    }

    // -- Multi-GPU (`DCC_SHELL_SPEC.md` §2.5, `GUI_GAP_REGISTER.md`
    //    PR-01/PR-02/PR-04/PR-05) --------------------------------------------
    //
    // These read and write a *process-global* preference in `cartalith-gpu`,
    // not a `WorldParams` field, which is why they are `&self` and why they
    // are not in `params.rs`'s table: they describe the machine, not the
    // world. Nothing here is consulted at all while `use_gpu` is off, so the
    // CPU golden path is untouched.

    /// Every physical GPU this machine exposes, discrete first
    /// (`DCC_SHELL_SPEC.md` §2.5's "Devices"). One entry per **physical**
    /// device, not per adapter: a single GPU is typically reachable over
    /// several backends, and this folds those together.
    ///
    /// Each entry: `key` (String — the stable id to pass back to
    /// `gpu_set_selected_devices`; never store the array index), `name`,
    /// `kind` (`"discrete"`/`"integrated"`/`"virtual"`/`"software"`/
    /// `"other"`), `backend` (`"vulkan"`/`"dx12"`/…), `alt_backends`
    /// (PackedStringArray), `driver`, `driver_info`, `max_buffer_mb` (int),
    /// `compute` (bool), `software` (bool).
    ///
    /// **Empty is a normal answer**, not an error: a headless session or a
    /// machine with no usable GPU returns an empty array, and the UI should
    /// say so rather than treating it as a failure.
    ///
    /// Deliberately absent: any utilisation percentage. §2.5 sketches
    /// `GPU 0 · discrete 16 GB 71%`, and neither number is obtainable —
    /// `wgpu` 30 exposes no VRAM size and no system-wide utilisation on any
    /// backend. What is real is this application's own allocation total, in
    /// `gpu_last_device_usage`.
    #[func]
    fn gpu_enumerate_devices(&self) -> Array<VarDictionary> {
        cartalith_gpu::enumerate_devices()
            .iter()
            .map(|d| {
                let alts: PackedStringArray =
                    d.alternate_backend_strs().into_iter().map(GString::from).collect();
                vdict! {
                    "key" => d.key.clone(),
                    "name" => d.name.clone(),
                    "kind" => d.kind_str(),
                    "backend" => d.backend_str(),
                    "alt_backends" => &alts,
                    "driver" => d.driver.clone(),
                    "driver_info" => d.driver_info.clone(),
                    "max_buffer_mb" => (d.max_buffer_size / (1024 * 1024)) as i64,
                    "compute" => d.supports_compute,
                    "software" => d.is_software,
                }
            })
            .collect()
    }

    /// The device keys dispatch currently uses, in order. **Empty means
    /// automatic** — the same `PowerPreference::HighPerformance` request
    /// this port always made — which is the default and is not the same
    /// thing as "no devices selected".
    #[func]
    fn gpu_selected_devices(&self) -> PackedStringArray {
        cartalith_gpu::preferences().selected_keys.iter().map(GString::from).collect()
    }

    /// Choose which device(s) dispatch runs on. Order matters in
    /// `split_tiles` mode (the first device gets the first row band); in
    /// `single_device` mode only the first entry is used. Pass an empty
    /// array to return to automatic. Keys come from
    /// `gpu_enumerate_devices`; an unknown key degrades to automatic rather
    /// than to no GPU. Takes effect on the next generate.
    #[func]
    fn gpu_set_selected_devices(&self, keys: PackedStringArray) {
        let mut p = cartalith_gpu::preferences();
        p.selected_keys = keys.as_slice().iter().map(GString::to_string).collect();
        cartalith_gpu::set_preferences(p);
    }

    /// `"single_device"` · `"split_tiles"` · `"alternate_frames"`
    /// (§2.5's "Multi-GPU mode"). Default `"single_device"`.
    #[func]
    fn gpu_multi_mode(&self) -> GString {
        cartalith_gpu::preferences().mode.as_str().into()
    }

    /// Set the multi-GPU mode. Returns `false` — changing nothing — for an
    /// unknown name **and** for `"alternate_frames"`, which this port does
    /// not implement: §2.5's own note is that it "only helps the 3D
    /// viewport", and there is no 3D viewport. Refusing is the honest
    /// answer; accepting it and silently behaving as `single_device` is not.
    #[func]
    fn gpu_set_multi_mode(&self, mode: GString) -> bool {
        let Some(m) = cartalith_gpu::MultiGpuMode::parse(&mode.to_string()) else { return false };
        if !m.is_implemented() {
            return false;
        }
        let mut p = cartalith_gpu::preferences();
        p.mode = m;
        cartalith_gpu::set_preferences(p);
        true
    }

    /// The VRAM cap in GB, or `0.0` for no cap (the default).
    ///
    /// §2.5 specifies "default 75 % of the smallest active device"; that
    /// default is **not implementable** and is not faked here — `wgpu` 30
    /// reports no VRAM size for an adapter at all, so there is no quantity
    /// to take 75 % of.
    #[func]
    fn gpu_vram_budget_gb(&self) -> f64 {
        cartalith_gpu::preferences().vram_budget_bytes as f64 / (1024.0 * 1024.0 * 1024.0)
    }

    /// Cap the grid size the GPU path will accept, in GB. `0` removes the
    /// cap. Compared against a documented upper-bound *estimate* of what
    /// this pipeline allocates for a grid, not against real occupancy —
    /// see `gpu_vram_estimate`.
    #[func]
    fn gpu_set_vram_budget_gb(&self, gb: f64) {
        let mut p = cartalith_gpu::preferences();
        p.vram_budget_bytes = if gb <= 0.0 { 0 } else { (gb * 1024.0 * 1024.0 * 1024.0) as u64 };
        cartalith_gpu::set_preferences(p);
    }

    /// `"cpu_tile_pass"` · `"reduce_working_res"` · `"fail_with_error"`
    /// (§2.5's "Fallback when VRAM full"). Default `"cpu_tile_pass"`.
    #[func]
    fn gpu_vram_fallback(&self) -> GString {
        cartalith_gpu::preferences().fallback.as_str().into()
    }

    /// Set the over-budget fallback. Returns `false` — changing nothing —
    /// for an unknown name and for `"reduce_working_res"`, which has no
    /// implementation: nothing in this pipeline computes a stage at a
    /// reduced grid and resamples back up.
    #[func]
    fn gpu_set_vram_fallback(&self, fallback: GString) -> bool {
        let Some(f) = cartalith_gpu::VramFallback::parse(&fallback.to_string()) else { return false };
        if !f.is_implemented() {
            return false;
        }
        let mut p = cartalith_gpu::preferences();
        p.fallback = f;
        cartalith_gpu::set_preferences(p);
        true
    }

    /// What the budget says about a grid size:
    /// `estimate_mb` (int, this pipeline's upper-bound GPU working set),
    /// `budget_mb` (int, `0` = uncapped), `over_budget` (bool),
    /// `action` (`"gpu"` · `"cpu_fallback"` · `"fail"`), `gw`, `gh`.
    ///
    /// Pass the grid the **next** generate will use. `0` (or negative) for
    /// either falls back to the stored parameters — which is only useful
    /// after a generate has run: `WorldParams`' own `gw`/`gh` are `0` until
    /// the first `generate*()` call sets them, so asking before then and
    /// getting a `0x0` estimate back would be a silently useless answer.
    /// Found by driving this headlessly rather than by reading it.
    ///
    /// The shell calls this before generating: with the fallback set to
    /// `fail_with_error` the *caller* is where the error belongs, since
    /// `generate_terrain` returns a world rather than a `Result`.
    #[func]
    fn gpu_vram_estimate(&self, gw: i64, gh: i64) -> VarDictionary {
        let gw = if gw > 0 { gw as usize } else { self.params.gw };
        let gh = if gh > 0 { gh as usize } else { self.params.gh };
        let need = cartalith_gpu::gpu_working_set_bytes(gw, gh);
        let budget = cartalith_gpu::preferences().vram_budget_bytes;
        let verdict = cartalith_gpu::vram_verdict(gw, gh);
        vdict! {
            "gw" => gw as i64,
            "gh" => gh as i64,
            "estimate_mb" => (need / (1024 * 1024)) as i64,
            "budget_mb" => (budget / (1024 * 1024)) as i64,
            "over_budget" => verdict != cartalith_gpu::VramVerdict::Ok,
            "action" => match verdict {
                cartalith_gpu::VramVerdict::Ok => "gpu",
                cartalith_gpu::VramVerdict::FallBackToCpu => "cpu_fallback",
                cartalith_gpu::VramVerdict::Fail => "fail",
            },
        }
    }

    /// Real, measured GPU memory from the last GPU generation — one entry
    /// per active device, `name`, `allocated_mb`, `reserved_mb`.
    ///
    /// **This application's own allocations**, read from `wgpu`'s allocator,
    /// not system-wide VRAM occupancy and not a utilisation percentage.
    /// Empty before the first GPU generation of the session, which the UI
    /// must show as "not measured yet" rather than as zero.
    #[func]
    fn gpu_last_device_usage(&self) -> Array<VarDictionary> {
        cartalith_gpu::last_usage()
            .iter()
            .map(|(name, u)| {
                vdict! {
                    "name" => name.clone(),
                    "allocated_mb" => (u.allocated_bytes / (1024 * 1024)) as i64,
                    "reserved_mb" => (u.reserved_bytes / (1024 * 1024)) as i64,
                }
            })
            .collect()
    }

    /// The seed the last `generate()`/`generate_world_structure()` used
    /// (reference `state.tect.seed`). `0` before the first call. Seed is a
    /// `generate()` argument, not a settable parameter — this exists so a
    /// dialog can display the world it is actually looking at.
    #[func]
    fn get_seed(&self) -> i32 {
        self.seed
    }

    /// Whether the additive village-seeding pass is on (`set_villages_enabled`).
    /// Not part of `get_params()`: villages are a `cartalith-civ` concern,
    /// not a `WorldParams` field.
    #[func]
    fn get_villages_enabled(&self) -> bool {
        self.civ_options.villages
    }

    /// Whether the v0.75 imperial-seat promotion is on
    /// (`set_metropolis_enabled`). Same reasoning as
    /// `get_villages_enabled`: a `cartalith-civ` concern, not a
    /// `WorldParams` field.
    #[func]
    fn get_metropolis_enabled(&self) -> bool {
        self.civ_options.metropolis
    }

    /// The v0.82 recovery phase the next `generate()` will apply, as the
    /// reference's own numeric phase: `0` Stable / `1` Survival /
    /// `2` Subsistence / `3` Regional / `4` Mature.
    #[func]
    fn get_recovery_phase(&self) -> i32 {
        i32::from(self.civ_options.recovery.0)
    }

    /// The five `_CIV_RECOVERY_NAME` labels (reference line 24615), in
    /// phase order, so the shell's dropdown is filled from the engine's own
    /// table rather than a second transcription of it in GDScript.
    #[func]
    fn get_recovery_phase_names(&self) -> PackedStringArray {
        (0..=4)
            .map(|i| {
                GString::from(cartalith_civ::timeline::RecoveryPhase::from_index_clamped(i).name())
            })
            .collect()
    }

    /// The named World-Structure archetypes `apply_archetype` and
    /// `generate_world_structure` accept (reference HTML `ARCHETYPES`).
    #[func]
    fn get_archetypes(&self) -> PackedStringArray {
        ARCHETYPES.iter().map(|(name, _)| GString::from(*name)).collect()
    }

    /// Writes a named archetype's five raw World-Structure knobs into the
    /// persistent parameters and turns World Structure **on**, so a following
    /// plain `generate()` runs that archetype and `get_params()` shows its
    /// real numbers in the five sliders (the reference's own behaviour: its
    /// archetype segment sets the same five sliders, which stay editable as
    /// "Custom" fine-tuning afterwards).
    ///
    /// Returns `false` for an unknown name, changing nothing. Set
    /// `world_structure.enabled` back to `false` via `set_params` for the
    /// Classic (no continental steering) shape.
    ///
    /// Distinct from `generate_world_structure()`, which applies a preset for
    /// **one call only** and leaves these parameters untouched — that method
    /// keeps its original one-shot behaviour so existing GDScript that
    /// alternates between `generate()` and `generate_world_structure()` per
    /// menu selection keeps working unchanged.
    #[func]
    fn apply_archetype(&mut self, archetype: GString) -> bool {
        match archetype_knobs(&archetype) {
            Some(ws) => {
                self.params.world_structure = ws;
                true
            }
            None => false,
        }
    }

    /// Reference `_civVillages` (default OFF) -- the additive village-
    /// seeding pass (Phase 2 milestone 15), gated separately from the
    /// four flags above since it's civ-layer, not terrain-substrate.
    #[func]
    fn set_villages_enabled(&mut self, enabled: bool) {
        self.civ_options.villages = enabled;
    }

    /// Reference `_civMetropolis` (line 6442, default OFF) -- the v0.75
    /// imperial-seat promotion. The reference's own comment on that default
    /// is the reason it stays OFF here too: "OFF by default => auto-populate
    /// output bit-identical". When on, the next `generate()` promotes up to
    /// three high-betweenness capitals of large polities to
    /// `SettlementKind::Metropolis`, which `get_settlements()` then reports
    /// as `kind == "metropolis"`.
    #[func]
    fn set_metropolis_enabled(&mut self, enabled: bool) {
        self.civ_options.metropolis = enabled;
    }

    /// Reference `_civRecoveryPhase` (line 6443, default `0` Stable) -- the
    /// v0.82 static post-collapse recovery phase the next `generate()`
    /// applies. Clamped to `0..=4` exactly as the reference's own dropdown
    /// handler clamps it (`Math.max(0,Math.min(4,rp.value|0))`, line 26643).
    /// Phase `0` is a strict no-op.
    #[func]
    fn set_recovery_phase(&mut self, phase: i32) {
        self.civ_options.recovery = RecoveryPhaseOpt(i64::from(phase).clamp(0, 4) as u8);
    }

    /// `MVP_SCOPE.md` point 9 / `state.seaLevel` -- `sea_level` is the raw
    /// `[0,1]` normalized threshold `cartalith_engine::WorldParams` itself
    /// expects (see the `params` field's own doc comment for the
    /// World-Structure re-anchoring interaction this only partially
    /// controls). Clamped defensively -- an out-of-range value from a
    /// misconfigured GDScript control should not silently invert the
    /// land/ocean classification rather than just clamping to a sane edge.
    ///
    /// Pure sugar over `set_params({"sea_level": …})`, kept because
    /// `main.gd` already drives it; the two write the same field and clamp
    /// identically.
    #[func]
    fn set_sea_level(&mut self, sea_level: f64) {
        self.params.sea_level = sea_level.clamp(0.0, 1.0);
    }

    /// Runs the full ported pipeline (`cartalith_engine::generate_terrain`)
    /// at the given seed/real-km map width/grid resolution, using **this
    /// instance's persistent parameters** for everything else
    /// (`set_params`/`reset_params`/`set_sea_level`/`set_experimental_flags`
    /// — see the `params` field's own doc comment). `resolution` is clamped
    /// to a sane minimum (4) — a 0 or negative value from an unset GDScript
    /// `SpinBox` should not panic the extension.
    ///
    /// Seed, map width and resolution stay call arguments rather than
    /// parameters, matching the reference: the seed changes per "New seed"
    /// click, and map width is a creation-time decision the reference itself
    /// refuses to make editable mid-project ("changing it would silently
    /// rescale every derived distance, grade, route length and settlement
    /// spacing").
    ///
    /// **Square**: `gh = gw = resolution`. Exactly
    /// `generate_sized(seed, width_km, resolution, resolution)`, kept as its
    /// own `#[func]` because `main.gd` drives it — and kept square so every
    /// existing golden-parity fixture stays untouched. Use `generate_sized`
    /// for any other shape.
    #[func]
    fn generate(&mut self, seed: i32, width_km: f64, resolution: i32) {
        self.generate_sized(seed, width_km, resolution, resolution);
    }

    /// As `generate()`, but with **independent grid width and height** — the
    /// engine's real capability, which `generate()`'s single `resolution`
    /// argument hides.
    ///
    /// `grid_w`/`grid_h` are each clamped to a sane minimum (4), matching
    /// `generate()`'s own `resolution.max(4)`: a `0` from an unset GDScript
    /// `SpinBox` must not panic the extension.
    ///
    /// **`width_km` is the map's width, and cells stay square in km**, so the
    /// map's real height is `width_km * grid_h / grid_w` — read it back with
    /// `get_map_height_km()`. There is deliberately no independent
    /// `height_km`: every km↔cell conversion in this workspace derives from
    /// `map_width_km / gw` and applies it to both axes (see `call_params`),
    /// so a separately-set height would silently contradict every distance,
    /// grade, route length and settlement spacing the world is built from.
    /// A world that is 2:1 in cells is 2:1 in km.
    ///
    /// `reference_grid_height()` gives the shape the reference HTML app
    /// itself uses for a given working resolution.
    /// **Refused while the world is finalized** (`applyFinalizedUI`, reference
    /// line 10854): the baked atlas is keyed by the generation parameters, so
    /// changing them would strand every chunk the user just paid minutes of
    /// compute for. `finalize_check("generation")` is the same refusal as a
    /// query, so the shell can grey the control and say why.
    #[func]
    fn generate_sized(&mut self, seed: i32, width_km: f64, grid_w: i32, grid_h: i32) {
        if let Err(msg) = self.bake.check(cartalith_engine::bake::Mutation::Generation) {
            godot_print!("cartalith-godot: generate refused -- {msg}");
            return;
        }
        let (gw, gh) = (grid_w.max(4) as usize, grid_h.max(4) as usize);
        let p = self.call_params(seed, width_km, gw, gh);
        let ws = generate_terrain(&p);
        self.absorb(ws, &p, seed);
    }

    /// The reference's **third** way into a world (`Import ▸ Load
    /// heightmap…` + `Infer tectonics from heightmap`, reference HTML lines
    /// 534-535): read a PNG heightmap off disk, take it as the elevation
    /// field, and reconstruct a plausible tectonic substrate underneath it
    /// so every downstream layer has something coherent to read.
    ///
    /// `path` is a native OS filesystem path, exactly as `load_save` takes
    /// one. Returns `false` on any read or decode error, leaving the
    /// previous world untouched — the same fail-quietly-check-the-console
    /// shape `load_save` and `generate` already have. Nothing here can
    /// panic across the gdext boundary: every fallible step is a `Result`
    /// converted to a log line and a `false`
    /// (`cartalith-rust-conventions`).
    ///
    /// # Grid shape
    ///
    /// **`grid_h` is not a parameter, deliberately.** The reference derives
    /// the grid height from the *image's own* aspect ratio
    /// (`GH = max(80, round(GW / (imgW/imgH)))`), because an imported DEM
    /// has a shape of its own and resampling it into a caller-chosen frame
    /// would stretch it. `get_grid_size()` reports what was actually used.
    ///
    /// # Seed
    ///
    /// The inversion is deterministic from the heightmap alone — no RNG.
    /// `seed` is still taken because one downstream stage the reference
    /// reuses verbatim (`computeHeterogeneity`) is seeded, and because the
    /// tool editors `absorb` builds want a stream of their own.
    #[func]
    fn import_heightmap(&mut self, path: GString, seed: i32, width_km: f64, grid_w: i32) -> bool {
        let bytes = match std::fs::read(path.to_string()) {
            Ok(b) => b,
            Err(e) => {
                godot_print!("cartalith-godot: import_heightmap could not read the file: {e}");
                return false;
            }
        };
        let gw = grid_w.max(4) as usize;
        // `gh` here is a placeholder: `import_heightmap` overwrites it from
        // the image aspect and hands the corrected params back.
        let base = self.call_params(seed, width_km, gw, gw);
        match cartalith_engine::import::import_heightmap(&bytes, &base) {
            Ok(out) => {
                godot_print!(
                    "cartalith-godot: imported {}x{} heightmap onto a {}x{} grid",
                    out.source_size.0,
                    out.source_size.1,
                    out.params.gw,
                    out.params.gh
                );
                self.absorb(out.state, &out.params, seed);
                true
            }
            Err(e) => {
                godot_print!("cartalith-godot: import_heightmap failed: {e}");
                false
            }
        }
    }

    /// The reference's `#centerBtn` (`centerLandmasses`, reference HTML
    /// line 3179; `GUI_GAP_REGISTER.md` MS-01): rotate the wrapped world in
    /// X so the emptiest meridian sits at the map edge, and feather the
    /// join it moved into the interior.
    ///
    /// Returns a summary `Dictionary`: `ok` (bool), `offset` (int, columns
    /// rotated — `0` means the world was already centred and **nothing was
    /// touched**), `seam_column` (int), `reason` (String, only when `ok` is
    /// false).
    ///
    /// **World mode only.** In region mode the edges are hard borders and
    /// there is nothing to re-centre; this returns `ok: false` with a
    /// reason rather than silently rotating, matching the reference's own
    /// `alert()`-and-return.
    ///
    /// # What it invalidates
    ///
    /// The civilisation layer and the Sculpt draft are **dropped**, not
    /// shifted. Settlement, way and route coordinates are indices into the
    /// old grid; the reference does not shift them either
    /// (`centerLandmasses` clears its own derived caches and leaves
    /// `state.civ` alone), but this port would then render places over
    /// terrain that has moved out from under them. Re-run `generate()` for
    /// a civilisation layer over the centred world.
    ///
    /// Call `build_color_texture()` again afterwards — the same "no
    /// regeneration needed, the render path reads the field fresh"
    /// contract [`Self::sculpt_commit`] documents.
    #[func]
    fn center_landmasses(&mut self) -> VarDictionary {
        let (gw, gh, world) = (self.gw.max(0) as usize, self.gh.max(0) as usize, self.world);
        let Some(WorldSource::Generated(ws)) = self.source.as_mut() else {
            return dict! { "ok" => false, "offset" => 0i64, "seam_column" => 0i64,
                "reason" => "Center landmasses needs a generated world; a loaded save carries no tectonic substrate to rotate." };
        };
        let Some(r) = cartalith_engine::center::center_landmasses(ws, gw, gh, world) else {
            return dict! { "ok" => false, "offset" => 0i64, "seam_column" => 0i64,
                "reason" => "Center landmasses applies only in Whole-world mode -- the map wraps in longitude there. In Region mode the edges are hard borders, so there is nothing to re-centre." };
        };
        if r.offset != 0 {
            // Both hold coordinates into the grid that just rotated under
            // them. Dropping beats silently drifting.
            self.civ = None;
            self.sculpt = None;
        }
        dict! { "ok" => true, "offset" => r.offset as i64, "seam_column" => r.seam_column as i64 }
    }

    /// The reference's `#fjordBtn` (`carveFjordsOp`, reference HTML line
    /// 3245): build the fjord-probability mask and overdeepen the coastal
    /// valley floors inside it, drowning them into inlets while leaving the
    /// ridges between high.
    ///
    /// An **opt-in** pass, exactly as in the reference — it never runs
    /// during `generate()`, so a default world is bit-identical with or
    /// without this binding existing.
    ///
    /// Returns a summary `Dictionary`: `ok` (bool), `cells_masked` (int,
    /// cells with a non-zero fjord probability), `cells_carved` (int, cells
    /// the carve actually lowered), `recomputed`/`still_stale`
    /// (`PackedStringArray`, see below), `reason` (String, only when `ok` is
    /// false). `cells_carved == 0` on a warm or low-relief world is a real
    /// answer, not a failure: fjords are strictly bound to cold, steep,
    /// competent-rock coast.
    ///
    /// # What it re-runs, and what it does not
    ///
    /// Flow and climate **are** recomputed, via the same staleness-graph path
    /// [`Self::sculpt_commit`] uses: the carve marks `PipelineStage::Height`,
    /// and `recompute_stale` runs one `refresh_climate` over the carved
    /// surface. That is two thirds of the reference's own tail
    /// (`enforceRiverChannels()`, `computeFlow(true)`, `refreshClimate()`).
    /// The **river extraction** is the missing third: `channels`,
    /// `stream_order` and the carve-time `river_mask` are as they were
    /// before this call, since re-deriving the vector network is not part of
    /// what `refresh_climate` does.
    ///
    /// Preview the mask first with the `fjord` debug view
    /// (`build_debug_texture("fjord")`), which is the exact same
    /// computation.
    #[func]
    fn carve_fjords(&mut self) -> VarDictionary {
        let (gw, gh) = (self.gw.max(0) as usize, self.gh.max(0) as usize);
        let sea = self.sea_level;
        let Some(WorldSource::Generated(ws)) = self.source.as_mut() else {
            return dict! { "ok" => false, "cells_masked" => 0i64, "cells_carved" => 0i64,
                "reason" => "Carve fjords needs a generated world; a loaded save carries no lithology inputs (crust_field/age_field) to derive the mask from." };
        };
        let n = gw * gh;
        if n == 0 || ws.field.len() != n {
            return dict! { "ok" => false, "cells_masked" => 0i64, "cells_carved" => 0i64,
                "reason" => "No world." };
        }
        let sea_mask: Vec<u8> = ws.field.iter().map(|&h| u8::from((h as f64) < sea)).collect();
        let coast_d = cartalith_terrain::infer::chamfer_dist(&sea_mask, gw, gh);
        let lith = cartalith_civ::build_lithology(
            &ws.field,
            &ws.age_field,
            &ws.volcanic_field,
            &ws.crust_field,
            &ws.resistance_field,
            &ws.rainfall,
            sea,
        );
        let mask = cartalith_terrain::fjord::build_fjord_mask(
            &ws.field,
            &ws.temperature,
            &lith,
            &coast_d,
            gw,
            gh,
            sea,
            cartalith_terrain::fjord::FjordMaskOpts::for_width(gw),
        );
        let carved = cartalith_terrain::fjord::carve_fjords(
            &ws.field,
            &mask,
            gw,
            gh,
            sea,
            cartalith_terrain::fjord::CarveFjordsOpts::default(),
        );
        let cells_carved = carved.iter().zip(ws.field.iter()).filter(|(a, b)| a != b).count();
        // Global heightmap undo, the reference's own call site (`#fjordBtn`'s
        // handler opens with `pushUndo()`, reference HTML line 13017). Pushed
        // *here* rather than at the top of the function, unlike
        // `sculpt_commit` above: every early return above is a refusal that
        // never touches the field, and the reference's own button-level push
        // has no such refusals to avoid (its guards run before the click
        // handler's `pushUndo()`). No step is spent on a call that changed
        // nothing.
        self.undo.push("Carve fjords", &ws.field);
        self.ledger.record(
            "height",
            "Carve fjords",
            format!("{} x {}", self.gw, self.gh),
            undo::EntryKind::HeightSnapshot,
        );
        ws.field = carved;
        let cells_masked = mask.iter().filter(|&&m| m > 0.0).count() as i64;
        // A coastal carve is not tile-local -- it can touch any coast on the
        // map -- so the whole graph is marked, which is also all a
        // whole-field recompute could act on.
        let all_tiles = 0..self.stages.tile_count();
        let (recomputed, still_stale) = self.mark_and_recompute(PipelineStage::Height, all_tiles, "carve_fjords");
        dict! {
            "ok" => true,
            "cells_masked" => cells_masked,
            "cells_carved" => cells_carved as i64,
            "recomputed" => &recomputed,
            "still_stale" => &still_stale,
        }
    }

    /// Re-runs whatever the pipeline currently reports stale, and reports
    /// what it ran (`cartalith_engine::staleness::recompute_stale` — read
    /// that function's own doc comment for exactly which stages, and for the
    /// three things it deliberately leaves alone).
    ///
    /// The commit paths ([`Self::sculpt_commit`], [`Self::carve_fjords`],
    /// [`Self::paint_commit`]) already call this internally, so an edit takes
    /// effect without anyone calling this at all. It exists as its own
    /// `#[func]` for the cases the commits don't cover: a caller that
    /// deferred a recompute, a batch of edits settled in one pass at the end,
    /// or a "Recompute now" control when one is designed. Calling it with
    /// nothing stale is a no-op that runs nothing and costs a graph query.
    ///
    /// Returns `{"recomputed": PackedStringArray, "still_stale":
    /// PackedStringArray, "ms": float}` — the stage names that re-ran, the
    /// ones that did not, and how long it took. `"civ"` in `still_stale` is
    /// the normal steady state after a terrain edit, not an error: the civ
    /// layer is re-derived by a full `generate()`, and `UNIFIED_TOOL_PLAN.md`
    /// milestone C measured why it is not cascaded per stroke.
    #[func]
    fn recompute_stale_stages(&mut self) -> VarDictionary {
        let t0 = std::time::Instant::now();
        // No stage marked, so nothing is invalidated by the call itself --
        // this asks the graph what is *already* stale and settles it.
        let (recomputed, still_stale) = self.mark_and_recompute(PipelineStage::Height, [], "recompute");
        dict! {
            "recomputed" => &recomputed,
            "still_stale" => &still_stale,
            "ms" => t0.elapsed().as_secs_f64() * 1000.0,
        }
    }

    /// `GUI_GAP_REGISTER.md` **SG-01**: what is stale right now, and why —
    /// the read a staleness indicator is built on, and the answer
    /// [`Self::recompute_stale_stages`] would act on if it were called this
    /// instant.
    ///
    /// A pure query. Every [`cartalith_spatial::StageGraph`] accessor takes
    /// `&self` precisely so asking can never trigger work, and this preserves
    /// that: calling it once a second from a status bar recomputes nothing.
    ///
    /// Returns one entry per **stale** stage — an empty `Dictionary` is the
    /// healthy state, not an error — keyed by
    /// `cartalith_engine::staleness::PipelineStage::name` (`"height"`,
    /// `"hydrology"`, `"climate"`, `"civ"`), each holding:
    ///
    /// - `origin` (String) — the *most upstream* stage whose change has not
    ///   been consumed, **across every stale tile**, which is the one worth
    ///   naming: a sculpted ridge shows up at `civ` as `height`, not as a
    ///   chain of "my upstream moved" and not as the whole-map
    ///   `mark_recomputed` that ran in the same breath.
    /// - `reason` (String) — that change's own recorded reason: `"sculpt"`,
    ///   `"carve_fjords"`, `"paint"`, `"param:climate.rain_k"`,
    ///   `"reset_params"`. Empty only if a mark was recorded without one.
    /// - `tiles` (int) — how many of the graph's tiles are stale, so an
    ///   indicator can tell one brush stroke from a whole-map invalidation.
    ///   **`0` means not tile-scoped**, which today is only the `civ` entry
    ///   below.
    ///
    /// The `civ` entry has one extra source the graph cannot represent: a
    /// hand-dropped, hand-edited or deleted settlement (`self.civ_dirty` — see
    /// that field for why it is a flag and not a mark). It is reported as
    /// `origin: "settlements"`, `reason: "place_edited"`, `tiles: 0`, and only
    /// when the graph is not already reporting `civ` stale for a bigger
    /// reason. Without it the indicator would read "up to date" immediately
    /// after the edit that `ED-03d`'s Recompute button exists for.
    #[func]
    fn stale_stages(&self) -> VarDictionary {
        let entry = |origin: &str, reason: &str, tiles: i64| -> VarDictionary {
            dict! { "origin" => origin, "reason" => reason, "tiles" => tiles }
        };
        let mut out = VarDictionary::new();
        let mut civ_reported = false;
        for s in PipelineStage::ALL {
            let tiles = self.stages.stale_tiles(s.id());
            if tiles.is_empty() {
                continue;
            }
            // The most upstream origin over *every* stale tile, not the first
            // tile's. Found in the real shell, invisible at a grid small
            // enough for one stroke to cover the whole tiling: a sculpt marks
            // `Height` at the tiles it touched, but `recompute_stale`'s
            // `mark_recomputed` bumps hydrology over the *whole* map — so at
            // any tile the brush missed, civ's most-upstream unconsumed
            // change is hydrology's own "flow_recomputed" bookkeeping string,
            // and tile 0 is usually such a tile. Reporting that would name
            // the recompute instead of the edit that caused it. Ids are
            // topological, so the smallest is the most upstream — the same
            // rule `StageGraph::staleness` already applies within one tile.
            let mut best: Option<(usize, &str, &str)> = None;
            for &t in &tiles {
                let Some(why) = self.stages.staleness(s.id(), t) else { continue };
                if best.is_none_or(|(origin, _, _)| why.origin < origin) {
                    best = Some((why.origin, why.origin_name, why.reason.unwrap_or("")));
                }
            }
            let (_, origin, reason) = best.expect("stale_tiles only returns tiles staleness() reported");
            out.set(s.name(), &entry(origin, reason, tiles.len() as i64));
            civ_reported |= s == PipelineStage::Civ;
        }
        if self.civ_dirty && !civ_reported {
            out.set(PipelineStage::Civ.name(), &entry("settlements", "place_edited", 0));
        }
        out
    }

    /// `GUI_GAP_REGISTER.md` SG-02's **"Recompute now"** for `civ` — the one
    /// stage [`Self::recompute_stale_stages`] deliberately leaves stale, and
    /// the recompute `ED-03d` says a place edit or delete never triggered.
    ///
    /// Manual on purpose. `UNIFIED_TOOL_PLAN.md` milestone C measured
    /// cascading into civ after every stroke at ~7 s at 2048², which is not
    /// a per-stroke cost an interactive tool can pay; this is that same work,
    /// paid once, when the user asks for it.
    ///
    /// **Measured**, release build, CPU path, square grids at 1200 km:
    /// 0.94 s at 512², 1.60 s at 1024², 4.22 s at 2048² — about half the
    /// cost of a full `generate()` of the same world (1.28 s / 2.59 s /
    /// 8.16 s on the same machine and run), and cheaper than the ~7 s figure
    /// because skipping placement and naming also skips seed-finding and the
    /// water-edge snap. Repeatable to within a few ms on a second call: this
    /// does the same work every time, it has no "nothing changed" fast path.
    ///
    /// **What it does.** Hydrology and climate are settled first (a civ layer
    /// derived over pre-edit rainfall would be a different kind of stale),
    /// then `compute_civilisation` re-runs with the current settlement list
    /// held fixed — see that function's `keep` parameter for the full
    /// argument. Everything downstream of the settlements is rebuilt against
    /// the edited terrain: water bodies, biome, soil and lithology,
    /// resources, the road network and its consolidated ways, sea lanes,
    /// territory, provinces, per-settlement trade balances, the suitability
    /// explanations and agrarian density.
    ///
    /// **What it preserves.** The settlements themselves (so a hand-dropped
    /// or hand-edited place survives), and with them every `tid`-keyed side
    /// table: `place_extras` (traits, specialisation, history, age/walls
    /// overrides), the faction roster, the recorded timeline and year, and
    /// hand-painted territory — which is re-anchored onto the newly computed
    /// borders by [`civ_tools_bridge::CivTools::rebase`] rather than erased.
    ///
    /// **What it does not do.** It does not re-place settlements. Sculpt a
    /// mountain under a city and the city stays on the mountain; the control
    /// for re-deriving placement from terrain is `generate()`.
    ///
    /// Returns `{"ok": bool, "ms": float, "settlements": int, "ways": int,
    /// "provinces": int, "recomputed": PackedStringArray, "still_stale":
    /// PackedStringArray, "reason": String}` — `reason` empty when `ok`.
    #[func]
    fn recompute_civilisation(&mut self) -> VarDictionary {
        let t0 = std::time::Instant::now();
        let refuse = |reason: &str| -> VarDictionary {
            dict! {
                "ok" => false, "ms" => 0.0, "settlements" => 0i64, "ways" => 0i64,
                "provinces" => 0i64, "recomputed" => &PackedStringArray::new(),
                "still_stale" => &PackedStringArray::new(), "reason" => reason,
            }
        };
        // Civ sits downstream of both, so settle them before deriving over
        // them. Marks nothing itself (the empty tile list) — it only asks the
        // graph what is already stale, exactly as `recompute_stale_stages`
        // does.
        let (mut recomputed, _) = self.mark_and_recompute(PipelineStage::Height, [], "recompute_civ");
        let p = self.recompute_params();
        let n = p.gw * p.gh;
        let (Some(civ), Some(WorldSource::Generated(ws))) = (self.civ.as_ref(), self.source.as_ref()) else {
            return refuse(
                "Recompute civilisation needs a generated world with a civ layer; a loaded save carries none (SAVEFILE_COMPAT.md stores no lithology substrate to derive one from).",
            );
        };
        if n == 0 || ws.field.len() != n {
            return refuse("No world.");
        }
        let keep = KeptCiv {
            settlements: civ.settlements.clone(),
            next_tid: civ.next_tid,
            village_tids: civ.village_tids.clone(),
        };
        let mut fresh = compute_civilisation(
            ws, p.gw, p.gh, p.world, p.map_width_km, p.river_density, self.civ_options, Some(keep),
        );
        // Boundary state with no terrain input: moved across rather than
        // rebuilt, which is the whole difference between this and `generate`.
        let old = self.civ.take().expect("checked directly above");
        fresh.timeline = old.timeline;
        fresh.year = old.year;
        fresh.faction_roster = old.faction_roster;
        fresh.place_extras = old.place_extras;
        if let Some(tools) = self.civ_tools.as_mut() {
            tools.rebase(&mut fresh.territory);
        }
        let (n_places, n_ways, n_prov) = (fresh.settlements.len(), fresh.ways.len(), fresh.province_list.len());
        self.civ = Some(fresh);
        // SG-01: everything derived from the settlement list has just been
        // re-derived from the settlement list.
        self.civ_dirty = false;
        self.stages.mark_recomputed(PipelineStage::Civ.id(), "civ_recomputed");
        recomputed.push(&GString::from(PipelineStage::Civ.name()));
        let still_stale: PackedStringArray = PipelineStage::ALL
            .iter()
            .filter(|s| self.stages.any_stale(s.id()))
            .map(|s| GString::from(s.name()))
            .collect();
        dict! {
            "ok" => true,
            "ms" => t0.elapsed().as_secs_f64() * 1000.0,
            "settlements" => n_places as i64,
            "ways" => n_ways as i64,
            "provinces" => n_prov as i64,
            "recomputed" => &recomputed,
            "still_stale" => &still_stale,
            "reason" => "",
        }
    }

    /// What [`Self::import_heightmap`] would resample a given image onto,
    /// without doing the import — so an import dialog can show the real
    /// working grid before committing, rather than restating
    /// `max(80, round(gw / aspect))` in GDScript
    /// (`ARCHITECTURE.md`: "Godot computes nothing beyond layout").
    ///
    /// Returns `Vector2i(gw, gh)`.
    #[func]
    fn heightmap_grid_size(&self, grid_w: i32, image_w: i32, image_h: i32) -> Vector2i {
        let gw = grid_w.max(4) as usize;
        let gh = cartalith_terrain::infer::heightmap_grid_h(gw, image_w.max(0) as u32, image_h.max(0) as u32);
        Vector2i::new(gw as i32, gh as i32)
    }

    /// Generates with a named World-Structure archetype (`ARCHETYPES`, the
    /// reference's own five presets) applied for **this call only** — the
    /// stored parameters (`get_params()`) are left exactly as they were, so
    /// alternating between this and `generate()` per menu selection behaves
    /// as it always has. Use `apply_archetype()` instead to make an
    /// archetype stick and become editable as raw sliders.
    ///
    /// Returns `false` (generating nothing) for an unknown archetype name.
    ///
    /// **Square**, exactly as `generate()` is —
    /// `generate_world_structure_sized(seed, width_km, resolution,
    /// resolution, archetype)`. Use that method for any other shape.
    #[func]
    fn generate_world_structure(&mut self, seed: i32, width_km: f64, resolution: i32, archetype: GString) -> bool {
        self.generate_world_structure_sized(seed, width_km, resolution, resolution, archetype)
    }

    /// As `generate_world_structure()`, but with independent grid width and
    /// height — the same relationship `generate_sized()` has to `generate()`,
    /// including the square-cells-in-km rule (see `generate_sized`).
    #[func]
    fn generate_world_structure_sized(
        &mut self,
        seed: i32,
        width_km: f64,
        grid_w: i32,
        grid_h: i32,
        archetype: GString,
    ) -> bool {
        if let Err(msg) = self.bake.check(cartalith_engine::bake::Mutation::Generation) {
            godot_print!("cartalith-godot: generate refused -- {msg}");
            return false;
        }
        let Some(world_structure) = archetype_knobs(&archetype) else {
            godot_print!("cartalith-godot: unknown World-Structure archetype '{archetype}'");
            return false;
        };

        let (gw, gh) = (grid_w.max(4) as usize, grid_h.max(4) as usize);
        let mut p = self.call_params(seed, width_km, gw, gh);
        // `p.sea_level` carries the stored input for consistency with
        // `generate()`, but `apply_world_structure_sea_level` always
        // re-anchors it below since `enabled` is unconditionally true on this
        // path -- see the `params` field's own doc comment.
        p.world_structure = world_structure;

        let ws = generate_terrain(&p);
        self.absorb(ws, &p, seed);
        true
    }

    /// The grid height the **reference HTML app itself** uses for a given
    /// working resolution (`gridH`, reference line 5049): `round(grid_w *
    /// 0.5)` when `world` is on (2:1 equirectangular) and `round(grid_w *
    /// 0.64)` when it is off (a 1.5625:1 region frame), floored at 4.
    ///
    /// Exists so a setup dialog can offer "the shape the reference uses"
    /// without hardcoding those two constants in GDScript. Note the
    /// consequence: **this port's square default is a divergence from the
    /// reference, not a match** — the reference's maps are never square. It
    /// stays the default here only because every golden-parity fixture and
    /// every existing `main.gd` call is built on it.
    ///
    /// Pure function of its arguments; reads and changes no state.
    #[func]
    fn reference_grid_height(&self, grid_w: i32, world: bool) -> i32 {
        reference_grid_h(grid_w.max(4) as usize, world) as i32
    }

    /// The current world's real map **width** in km (`generate()`'s own
    /// `width_km` argument, or a loaded save's `mapWidthKm`). `0.0` before
    /// the first generation or load.
    #[func]
    fn get_map_width_km(&self) -> f64 {
        self.map_width_km
    }

    /// The current world's real map **height** in km — **derived**, as
    /// `map_width_km * gh / gw`, because the engine's one km↔cell quotient is
    /// `map_width_km / gw` applied isotropically (see `call_params`), i.e.
    /// cells are square in km. `0.0` before the first generation or load.
    ///
    /// This is a readout, not a setting: there is no `set_map_height_km`, and
    /// deliberately so — a height that disagreed with `width_km * gh / gw`
    /// would contradict every distance, grade, route length and settlement
    /// spacing the world was generated from.
    #[func]
    fn get_map_height_km(&self) -> f64 {
        if self.gw <= 0 {
            return 0.0;
        }
        self.map_width_km * self.gh as f64 / self.gw as f64
    }

    /// `MVP_SCOPE.md` point 12 / criterion 7: opens a real HTML-app `.zip`
    /// and renders that save's terrain. `path` is a native OS filesystem
    /// path (e.g. from a GDScript `FileDialog` in native/desktop mode) --
    /// `cartalith_io::load_save` only needs `Read + Seek`, so a plain
    /// `std::fs::File` satisfies it without any Godot `FileAccess`
    /// involvement. Returns `false` on any read/parse error and leaves the
    /// previous `source` untouched, matching `generate()`'s own
    /// fail-quietly-check-the-console shape (`main.gd`'s doc comment).
    #[func]
    fn load_save(&mut self, path: GString) -> bool {
        // Loading replaces the world outright, which is the most complete
        // form of the change `Mutation::Generation` names.
        if let Err(msg) = self.bake.check(cartalith_engine::bake::Mutation::Generation) {
            godot_print!("cartalith-godot: load_save refused -- {msg}");
            return false;
        }
        let file = match std::fs::File::open(path.to_string()) {
            Ok(f) => f,
            Err(e) => {
                godot_print!("cartalith-godot: load_save open failed: {e}");
                return false;
            }
        };
        let save = match cartalith_io::load_save(std::io::BufReader::new(file)) {
            Ok(s) => s,
            Err(e) => {
                godot_print!("cartalith-godot: load_save failed: {e}");
                return false;
            }
        };
        // A real reference export is very often non-square (the reference's
        // own `gridH` is `gw*0.64` in region mode, `gw*0.5` in world mode) --
        // this path has always carried both dimensions through correctly,
        // which is why loading was never affected by `generate()`'s square
        // restriction.
        self.gw = save.params.gw as i32;
        self.gh = save.params.gh as i32;
        self.map_width_km = save.params.map_width_km;
        self.sea_level = save.params.sea_level;
        self.world = save.params.world;
        // SaveParams carries no latitude band -- JS's own literal
        // `climate` defaults (reference HTML line 2287), same fallback
        // WorldParams::defaults uses.
        self.lat_n = 55.0;
        self.lat_s = 5.0;
        // A loaded save carries none of the tectonic substrate fields
        // (crust_field/boundary_type/shear_field/age_field) the civ
        // pipeline needs (SAVEFILE_COMPAT.md doesn't store them) --
        // civ data only ever exists for a freshly generated world.
        self.civ = None;
        // A loaded save was not generated by this process at all -- reporting
        // the previous world's GPU stages against it would be a lie.
        self.gpu_stages_used = Vec::new();
        // Same restriction as `civ` above, for the same reason: a draft's
        // water hooks need `river_mask`/`river_floor`, which the save
        // format doesn't carry (`SAVEFILE_COMPAT.md`). Any in-progress
        // draft over the *previous* world would also silently apply to
        // the wrong dimensions if kept, so this is a hard reset, not a
        // narrower "just don't offer commit" gate.
        self.sculpt = None;
        // Same restriction, for the same reason: a loaded save carries no
        // manual-icon list at all (`SAVEFILE_COMPAT.md`), and any
        // in-progress icons from the *previous* world would silently carry
        // grid coordinates over the wrong dimensions if kept.
        self.icons = None;
        // Same restriction as `civ` above, for the same reason: a loaded
        // save has no `territory` for `civ_tools_bridge::CivTools::
        // territory_base` to be a snapshot OF, and any in-progress paint
        // draft from the *previous* world would silently apply to the wrong
        // dimensions if kept.
        self.civ_tools = None;
        // Same restriction as `sculpt` above, for the same reason: the
        // land-only gate's cached water-body classification
        // (`paint_bridge::PaintEditor`'s own doc comment) is only ever
        // computed from a freshly generated `WorldState`'s `field`/
        // `rainfall`, and any in-progress paint draft from the *previous*
        // world would silently apply to the wrong dimensions if kept.
        self.paint = None;
        // Same restriction as `icons` above, for the same reason: a loaded
        // save carries no label list at all (`SAVEFILE_COMPAT.md`'s "civ
        // and UI payloads" note is about what the *reference's* save
        // format holds, not what this port's `load_save` reconstructs --
        // this loader has never rebuilt `civ`/`sculpt`/`icons`/`civ_tools`
        // from a save either), and any in-progress labels from the
        // *previous* world would silently carry grid coordinates over the
        // wrong dimensions if kept.
        self.labels = None;
        // Same restriction as `civ` above, for the same reason: Way/Route
        // commit both need `self.civ` (never real for a loaded save, see
        // above), and any in-progress waypoints, measurement or marquee
        // from the *previous* world would silently carry grid coordinates
        // over the wrong dimensions if kept.
        self.infra = None;
        // Same reasoning as `absorb()`'s own clear: an undo step holds the
        // *previous* world's height field, which is the wrong content and
        // possibly the wrong length over a loaded save.
        self.undo.clear();
        // ED-02: a load is the ledger's other floor, for the same reason.
        self.ledger.record(
            "world",
            "Open project",
            format!("{} x {}", self.gw, self.gh),
            undo::EntryKind::Floor,
        );
        // The generation parameters this port wrote into the save
        // (`params::apply_saved_state`, `SAVEFILE_COMPAT.md`). A genuine
        // HTML-app export carries no such block and leaves every parameter
        // at its default, exactly as this loader behaved before a writer
        // existed -- see `apply_saved_state`'s own doc comment for why it
        // deliberately does not reconstruct them from the reference's
        // `state` instead.
        params::apply_saved_state(&mut self.params, &save.state);
        self.seed = save.params.seed;
        self.source = Some(WorldSource::Loaded(Box::new(save)));
        true
    }

    /// `GUI_GAP_REGISTER.md` FI-01 (Save project) — writes the current world
    /// as a `.zip` in the format `SAVEFILE_COMPAT.md` documents, readable
    /// both by this port's own `load_save` and by the reference HTML app.
    ///
    /// `path` is a native OS filesystem path, the same convention
    /// `load_save`/`load_asset_pack` already use. Returns `false` and leaves
    /// any existing file **untouched** on any failure: the archive is built
    /// in memory and only then written, so a full disk or a serialization
    /// error cannot leave a half-written save where a good one used to be.
    /// That is the one behaviour a save button must have and the reason this
    /// does not stream straight to the file.
    ///
    /// What travels: the six field entries plus the whole generation
    /// parameter table. What does **not**: the civilisation layer, labels,
    /// icons, hand-drawn ways, paint and sculpt drafts. `load_save` clears
    /// all of those (see its own comments), so writing them would produce a
    /// file whose contents this port cannot read back — the format has no
    /// place for them and the loader has nothing to put them in.
    #[func]
    fn save_project(&mut self, path: GString) -> bool {
        let Some(source) = self.source.as_ref() else {
            godot_print!("cartalith-godot: save_project: no world to save");
            return false;
        };
        let n = (self.gw as usize) * (self.gh as usize);
        // `strahler_order` is `u8` in the save format (`0` = non-channel);
        // this port's own `stream_order` is `i16` for a wider internal
        // range, so it saturates on the way out exactly as the reference's
        // own exporter does (`so[i] = o > 255 ? 255 : o`, reference line
        // 12448). A world generated with `carve_rivers` off has no stream
        // order at all -- an all-zero raster is the honest encoding of
        // "no channels", and is what the loader reads back.
        let fields = match source {
            WorldSource::Generated(ws) => cartalith_io::SaveFields {
                heightmap: ws.field.clone(),
                temperature: ws.temperature.clone(),
                rainfall: ws.rainfall.clone(),
                volcanic_field: ws.volcanic_field.clone(),
                impact_field: ws.impact_field.clone(),
                strahler_order: match ws.stream_order.as_ref() {
                    Some(order) => order.iter().map(|&o| o.clamp(0, 255) as u8).collect(),
                    None => vec![0u8; n],
                },
            },
            WorldSource::Loaded(save) => save.fields.clone(),
        };
        let params = cartalith_io::SaveParams {
            gw: self.gw as usize,
            gh: self.gh as usize,
            seed: self.seed,
            map_width_km: self.map_width_km,
            // The *effective* sea level, not `self.params.sea_level` -- a
            // World-Structure archetype re-anchors it, and the renderer
            // reading this save back must classify land the same way this
            // one does. The user-facing input keeps its own place in the
            // parameter block.
            sea_level: self.sea_level,
            world: self.world,
        };
        let state = params::save_state(&self.params);

        let mut buf: Vec<u8> = Vec::new();
        if let Err(e) =
            cartalith_io::write_save(std::io::Cursor::new(&mut buf), &cartalith_io::SaveWrite {
                params: &params,
                state,
                fields: &fields,
            })
        {
            godot_print!("cartalith-godot: save_project failed: {e}");
            return false;
        }
        match std::fs::write(path.to_string(), &buf) {
            Ok(()) => true,
            Err(e) => {
                godot_print!("cartalith-godot: save_project write failed: {e}");
                false
            }
        }
    }

    /// `ASSET_LIBRARY_SCOPE.md` milestone 7: load a real `.zip` asset pack
    /// (the same format `cartalith-assets` milestones 1-2 golden-verified
    /// against the reference's own exporter, e.g. `tests/fixtures/
    /// reference_pack.zip`) and decode the two families this port composites
    /// — scattered feature icons and the six ground-texture splat channels
    /// (`pack.rs`'s own doc comment explains why the biome/terrain "painted
    /// layer" families are parsed but not decoded).
    ///
    /// `path` is a native OS filesystem path, matching `load_save`'s own
    /// convention (a plain `std::fs::File` read, no Godot `FileAccess`
    /// involved) rather than a `res://`/`user://` virtual path — this is a
    /// real API surface for a caller (a future importer UI, a debug script,
    /// this milestone's own verification pass) to drive, not a GUI control
    /// in itself. Returns `false` and leaves any previously loaded pack in
    /// place on any read/parse/decode error.
    #[func]
    fn load_asset_pack(&mut self, path: GString) -> bool {
        let bytes = match std::fs::read(path.to_string()) {
            Ok(b) => b,
            Err(e) => {
                godot_print!("cartalith-godot: load_asset_pack open failed: {e}");
                return false;
            }
        };
        match pack::load_pack_from_bytes(bytes) {
            Ok(loaded) => {
                self.asset_pack = Some(loaded);
                true
            }
            Err(e) => {
                godot_print!("cartalith-godot: load_asset_pack failed: {e}");
                false
            }
        }
    }

    /// Whether a real asset pack is currently loaded (`load_asset_pack`) —
    /// lets a caller (verification tooling, a future importer UI) confirm
    /// the load actually took without re-deriving it from `build_color_texture`'s
    /// pixel output.
    #[func]
    fn has_asset_pack(&self) -> bool {
        self.asset_pack.is_some()
    }

    /// The active §29 appearance quality tier (`TERRAIN_APPEARANCE_SCOPE.md`
    /// milestone 6), as its stable lowercase name. `"quality"` until a
    /// caller sets otherwise.
    #[func]
    fn get_quality_tier(&self) -> GString {
        GString::from(self.quality.name())
    }

    /// Set the appearance quality tier by name (see `list_quality_tiers`).
    /// Returns `false` and changes nothing for an unrecognised name --
    /// silently falling back to a default would hide a typo in a settings
    /// file and render a world at a quality nobody asked for.
    ///
    /// **Presentation only.** This never touches the heightmap, climate,
    /// hydrology, biome classification, settlements, routes or the seed: it
    /// selects which appearance stages `build_color_texture` runs. Call
    /// `build_color_texture()` again afterwards to see it -- no regeneration
    /// is needed, which is research §23's performance rule.
    #[func]
    fn set_quality_tier(&mut self, name: GString) -> bool {
        match QualityTier::from_name(&name.to_string()) {
            Some(t) => {
                self.quality = t;
                true
            }
            None => false,
        }
    }

    /// Every tier name, cheapest first -- so a caller can build a control
    /// without duplicating the vocabulary.
    #[func]
    fn list_quality_tiers(&self) -> PackedStringArray {
        QualityTier::ALL.iter().map(|t| GString::from(t.name())).collect()
    }

    // -- The named look (`render::LOOK_PRESETS`) ------------------------------

    /// Every named look, in the engine's own order -- so the picker is the
    /// engine's list rather than a second copy of it in GDScript, the rule
    /// `list_appearance_tunables` and `list_ramp_presets` already follow.
    #[func]
    fn list_looks(&self) -> PackedStringArray {
        render::LOOK_PRESETS.iter().map(|n| GString::from(*n)).collect()
    }

    /// The look currently in force. `"Natural Vibrant"` on a fresh session.
    #[func]
    fn get_look(&self) -> GString {
        GString::from(self.look.as_str())
    }

    /// Select a named look. Returns `false` and changes nothing for a name
    /// this build does not have -- a panel written against a newer engine
    /// loses a row rather than silently rendering a look nobody chose, which
    /// is `set_quality_tier`'s own contract.
    ///
    /// **Presentation only.** The look never touches the heightmap, climate,
    /// hydrology, biomes, settlements, routes or the seed; call
    /// `build_color_texture()` again to see it, with no regeneration.
    ///
    /// A look is the *base*, so the caller's own `set_appearance` overrides
    /// still sit on top of it and survive the change -- deliberately, and for
    /// the reason `appearance_over`'s own doc gives about the tier.
    #[func]
    fn set_look(&mut self, name: GString) -> bool {
        let name = name.to_string();
        match render::LOOK_PRESETS.iter().find(|n| n.eq_ignore_ascii_case(&name)) {
            Some(n) => {
                self.look = (*n).to_string();
                true
            }
            None => false,
        }
    }

    /// The appearance this `WorldGen` renders with: the active §29 quality
    /// tier, carrying the caller's NPR settings.
    ///
    /// **The single place the two combine.** Before this existed, five call
    /// sites each wrote `TerrainAppearance::for_tier(self.quality)` by hand,
    /// and every one of them would have had to remember the NPR block too --
    /// which is exactly how a raster and the overlays measured against it end
    /// up disagreeing about where the plate frame is.
    /// `Npr::peak_m` is filled in **here, from the world's own parameters**,
    /// and is deliberately not a `set_npr` key: the reference reads
    /// `state.peakM` -- the world's peak altitude -- rather than anything the
    /// Painter panel owns, and a caller who could set it separately could set
    /// it to something the world is not. It only ever turns a contour
    /// interval in metres into a fraction of relief.
    /// Layered in one direction, tier first and the user's own edits second,
    /// for the reason `appearance_over`'s own doc gives.
    fn appearance(&self) -> TerrainAppearance {
        let mut npr = self.npr.clone();
        npr.peak_m = self.params.peak_m;
        // Three layers, cheapest authority first: the quality tier, then a
        // loaded preset if there is one (CA-08), then the user's own edits
        // (CA-01) and their ramp (CA-02). Each layer only ever writes what it
        // actually carries, which is what lets a tier change survive an edit
        // and an edit survive a tier change.
        // A **saved look** (CA-08) is a complete description of an appearance,
        // so it replaces both the tier and the named look rather than being
        // graded by whichever one happened to be selected — the same reason it
        // already replaced the tier alone.
        let base = match self.appearance_preset.as_ref() {
            Some(p) => p.clone(),
            None => TerrainAppearance::for_tier(self.quality).with_look(&self.look),
        };
        let mut a = TerrainAppearance { npr, ..base };
        if let Some(ramp) = self.appearance_ramp.as_ref() {
            a.ramp = ramp.clone();
        }
        for (key, value) in &self.appearance_over {
            if key == render::TUNABLE_LIGHTS.0 {
                a.relief_lights = value.round().max(1.0) as usize;
            } else {
                a.set_tunable(key, *value);
            }
        }
        a
    }

    /// The appearance this `WorldGen` actually renders with, as a
    /// `Dictionary` keyed exactly as `set_appearance` reads it — the **merged**
    /// values (tier ladder plus the caller's own overrides), not the override
    /// map, so a panel opens showing what is on screen rather than what the
    /// caller last typed.
    ///
    /// `GUI_GAP_REGISTER.md` CA-01/CA-02/PR-09 and the reference's Cartography
    /// ▸ Map view + Rendering-advanced blocks all wanted this one binding.
    #[func]
    fn get_appearance(&self) -> VarDictionary {
        let a = self.appearance();
        let mut d = VarDictionary::new();
        for (key, _, _, _) in render::TerrainAppearance::TUNABLE {
            if let Some(v) = a.tunable(key) {
                d.set(*key, v);
            }
        }
        d.set(render::TUNABLE_LIGHTS.0, a.relief_lights as i64);
        d
    }

    /// The `(key, min, max, label)` table behind `get_appearance`, so a panel
    /// can build itself from the engine's own ranges rather than a second copy
    /// of them in GDScript. Returned as an `Array` of four-element `Array`s.
    ///
    /// This is the difference between a slider whose maximum is wrong and a
    /// slider that cannot be wrong: `set_appearance` clamps to these same
    /// numbers, so a UI built from this table can never send a value the
    /// engine will silently alter.
    #[func]
    fn list_appearance_tunables(&self) -> VarArray {
        let mut out = VarArray::new();
        let mut push = |key: &str, lo: f64, hi: f64, label: &str| {
            let mut row = VarArray::new();
            row.push(&GString::from(key).to_variant());
            row.push(&lo.to_variant());
            row.push(&hi.to_variant());
            row.push(&GString::from(label).to_variant());
            out.push(&row.to_variant());
        };
        for (key, lo, hi, label) in render::TerrainAppearance::TUNABLE {
            push(key, *lo, *hi, label);
        }
        let (key, lo, hi, label) = render::TUNABLE_LIGHTS;
        push(key, lo, hi, label);
        out
    }

    /// Set appearance values from a `Dictionary`, on `set_npr`'s exact
    /// contract: **every key is optional**, values are clamped to the range
    /// `list_appearance_tunables` publishes, and the return is the number of
    /// recognised keys applied — so a GDScript caller can tell a typo (`0`)
    /// from a real update without this method deciding what a typo means.
    ///
    /// **Presentation only.** This never touches the heightmap, climate,
    /// hydrology, biomes, settlements, routes or the seed; call
    /// `build_color_texture()` again to see it, with no regeneration.
    ///
    /// Overrides **survive a later `set_quality_tier`** — see
    /// `appearance_over`'s own doc for why that is the deliberate behaviour
    /// and not an oversight. `reset_appearance()` is how a caller gives the
    /// tier its values back.
    #[func]
    fn set_appearance(&mut self, values: VarDictionary) -> i32 {
        let mut applied = 0i32;
        for (key, lo, hi, _) in render::TerrainAppearance::TUNABLE {
            if let Some(v) = values.get(*key)
                && let Ok(f) = v.try_to::<f64>()
                && f.is_finite()
            {
                self.appearance_over.insert((*key).to_string(), f.clamp(*lo, *hi));
                applied += 1;
            }
        }
        let (lk, llo, lhi, _) = render::TUNABLE_LIGHTS;
        if let Some(v) = values.get(lk)
            && let Ok(f) = v.try_to::<f64>()
            && f.is_finite()
        {
            self.appearance_over.insert(lk.to_string(), f.round().clamp(llo, lhi));
            applied += 1;
        }
        applied
    }

    /// Drop the caller's overrides, ramp and loaded preset, and hand every
    /// appearance value back to the active quality tier. Returns how many
    /// things were dropped, so a "Reset" button can stay quiet when there was
    /// nothing to reset.
    ///
    /// All three layers, not just the override map: after a preset load the
    /// thing a user means by "reset" is the tier's own look, and leaving the
    /// preset in place would make the button appear to do nothing on exactly
    /// the occasion it is most needed.
    ///
    /// **The named look is deliberately not one of them.** It has its own
    /// picker, and a button in a different section silently moving that
    /// picker's selection is the exact desync `GUI_GAP_REGISTER.md` keeps
    /// finding one control at a time. `set_look("Quality tier")` is how a
    /// caller drops the look, and the picker shows that it did.
    #[func]
    fn reset_appearance(&mut self) -> i32 {
        let n = self.appearance_over.len() as i32 + self.appearance_ramp.is_some() as i32 + self.appearance_preset.is_some() as i32;
        self.appearance_over.clear();
        self.appearance_ramp = None;
        self.appearance_preset = None;
        n
    }

    // -- The elevation colour ramp (`GUI_GAP_REGISTER.md` CA-02) --------------

    /// The names of the built-in ramps, in `DCC_SHELL_SPEC.md` §7's own order,
    /// so the panel's picker is the engine's list rather than a second copy of
    /// it in GDScript -- the same rule `list_appearance_tunables` follows.
    #[func]
    fn list_ramp_presets(&self) -> PackedStringArray {
        render::RAMP_PRESETS.iter().map(|(n, _)| GString::from(*n)).collect()
    }

    /// The ramp currently in force, as an `Array` of `[position, Color]` rows,
    /// sorted by position. `position` is relative land elevation (`0` = the
    /// shoreline, `1` = the world's highest point); the panel turns that into
    /// metres with `peak_m` for display, and `RampStop`'s own doc explains why
    /// the engine will not store metres.
    ///
    /// The `Color`'s **alpha is the stop's own alpha**, not a placeholder `1`.
    /// Carrying it in the colour rather than as a third element keeps this row
    /// shape and `set_color_ramp`'s identical to what CA-02 published — a panel
    /// written against the older engine still round-trips, it simply always
    /// sends opaque stops.
    #[func]
    fn get_color_ramp(&self) -> VarArray {
        let a = self.appearance();
        let mut out = VarArray::new();
        for s in a.ramp.stops() {
            let mut row = VarArray::new();
            row.push(&s.at.to_variant());
            row.push(&Color::from_rgba((s.col.0 / 255.0) as f32, (s.col.1 / 255.0) as f32, (s.col.2 / 255.0) as f32, s.a as f32).to_variant());
            out.push(&row.to_variant());
        }
        out
    }

    /// Replace the ramp's stops with `stops`, each row `[position, Color]`
    /// exactly as `get_color_ramp` returns it — the `Color`'s alpha is the
    /// stop's own opacity. Returns the number of stops accepted. The
    /// interpolation mode is **not** part of this call and survives it; it is
    /// `set_ramp_mode`'s.
    ///
    /// **Adding, deleting and reordering are all this one call**: the panel
    /// sends the list it wants and the engine sorts it, so a stop dragged past
    /// its neighbour reorders by position rather than by list index, which is
    /// what a ramp means. Rows that are not `[number, Color]` are skipped, and
    /// a non-finite position is dropped rather than sorted against -- see
    /// `ElevationRamp::normalized` for the NaN policy and why a panic here
    /// would be worse than a dropped stop (`cartalith-rust-conventions`: a
    /// panic crossing the gdext boundary takes the process with it).
    ///
    /// An **empty** array is refused (returns `0`, changes nothing): a ramp
    /// with no stops renders nothing at all, and the honest way to turn the
    /// stage off is `ramp_strength` -- which is a published tunable, so the
    /// panel already has it.
    ///
    /// Presentation only, on `set_appearance`'s exact terms: call
    /// `build_color_texture()` again to see it, with no regeneration.
    #[func]
    fn set_color_ramp(&mut self, stops: VarArray) -> i32 {
        let mut parsed: Vec<render::RampStop> = Vec::new();
        for row in stops.iter_shared() {
            let Ok(row) = row.try_to::<VarArray>() else { continue };
            if row.len() < 2 {
                continue;
            }
            let Some(at) = row.at(0).try_to::<f64>().ok() else { continue };
            let Some(c) = row.at(1).try_to::<Color>().ok() else { continue };
            parsed.push(render::RampStop { at, col: (c.r as f64 * 255.0, c.g as f64 * 255.0, c.b as f64 * 255.0), a: c.a as f64 });
        }
        let mut ramp = render::ElevationRamp::normalized(parsed);
        // `normalized` builds a `Linear` ramp; the interpolation mode is a
        // property of the ramp, not of the stop list this call replaces, so
        // editing a stop must not silently reset it to Linear.
        ramp.set_mode(self.appearance().ramp.mode());
        if ramp.stops().is_empty() {
            return 0;
        }
        let n = ramp.stops().len() as i32;
        self.appearance_ramp = Some(ramp);
        n
    }

    /// Load one of `list_ramp_presets()`'s ramps by name. `false` for a name
    /// this build does not have, so a panel written against a newer engine
    /// loses a row rather than silently showing the wrong colours.
    #[func]
    fn load_ramp_preset(&mut self, name: GString) -> bool {
        match render::ElevationRamp::preset(&name.to_string()) {
            Some(mut r) => {
                // Same reasoning as `set_color_ramp`: the picker's own copy
                // says it "replaces every stop below", and the mode is not a
                // stop. A user who chose Step and then browsed the nine ramps
                // is browsing banded plates.
                r.set_mode(self.appearance().ramp.mode());
                self.appearance_ramp = Some(r);
                true
            }
            None => false,
        }
    }

    /// The interpolation modes this build has, in the engine's own order —
    /// `["Linear", "Ease", "Step"]`. The panel's picker is this list, not a
    /// second copy of it in GDScript.
    #[func]
    fn list_ramp_modes(&self) -> PackedStringArray {
        render::RAMP_MODES.iter().map(|n| GString::from(*n)).collect()
    }

    /// The mode currently in force, as one of `list_ramp_modes()`'s names.
    #[func]
    fn get_ramp_mode(&self) -> GString {
        GString::from(self.appearance().ramp.mode().name())
    }

    /// Set the interpolation mode by name. `false` for a name this build does
    /// not have, so a panel written against a newer engine loses a row rather
    /// than silently drawing the wrong curve.
    ///
    /// Presentation only, on `set_appearance`'s exact terms: call
    /// `build_color_texture()` again to see it, with no regeneration.
    #[func]
    fn set_ramp_mode(&mut self, name: GString) -> bool {
        let Some(mode) = render::RampMode::from_name(&name.to_string()) else {
            return false;
        };
        let mut ramp = self.appearance().ramp;
        ramp.set_mode(mode);
        self.appearance_ramp = Some(ramp);
        true
    }

    // -- Saving a look (`GUI_GAP_REGISTER.md` CA-08) --------------------------

    /// Write the appearance currently in force to `path` as a small JSON
    /// preset: the merged look (tier + preset + overrides + ramp), not the
    /// override map, so the file describes a picture rather than a diff
    /// against a tier the machine that reads it may not be on.
    ///
    /// **A preset file, deliberately not the project `.zip`.** A look is not
    /// world data -- it is reusable *across* worlds, which is the whole point
    /// of saving one, and `SAVEFILE_COMPAT.md`'s format is the reference HTML
    /// app's and shallow-merges `state`, so a block this port invented would
    /// be one more unshimmed key for that app to choke on. A named sidecar
    /// costs nothing and travels.
    ///
    /// `path` is a native OS filesystem path, the same convention
    /// `save_project` and `load_asset_pack` use; the shell resolves
    /// `user://…` through `ProjectSettings.globalize_path` before calling.
    #[func]
    fn save_appearance_preset(&self, path: GString, name: GString) -> bool {
        let doc = serde_json::json!({
            "format": "cartalith-appearance",
            "v": 1,
            "name": name.to_string(),
            "appearance": self.appearance(),
        });
        let text = match serde_json::to_string_pretty(&doc) {
            Ok(t) => t,
            Err(e) => {
                godot_print!("cartalith-godot: save_appearance_preset encode failed: {e}");
                return false;
            }
        };
        match std::fs::write(path.to_string(), text) {
            Ok(()) => true,
            Err(e) => {
                godot_print!("cartalith-godot: save_appearance_preset write failed: {e}");
                false
            }
        }
    }

    /// Read a preset written by [`Self::save_appearance_preset`] and make it
    /// the look. Returns `false` (and changes nothing) on any read, parse or
    /// format error.
    ///
    /// **Clears the override map and the ramp override.** Those are edits
    /// layered over a base, and the file *is* a base: keeping them would mean
    /// loading a saved look reproduced something other than the saved look,
    /// which is the one thing this call has to guarantee. `appearance()` after
    /// a successful load equals the `appearance()` that was saved, field for
    /// field -- with the single exception of `npr.peak_m`, which is a fact
    /// about the world on screen and is re-derived from `params` on every
    /// render.
    #[func]
    fn load_appearance_preset(&mut self, path: GString) -> bool {
        let text = match std::fs::read_to_string(path.to_string()) {
            Ok(t) => t,
            Err(e) => {
                godot_print!("cartalith-godot: load_appearance_preset open failed: {e}");
                return false;
            }
        };
        let doc: serde_json::Value = match serde_json::from_str(&text) {
            Ok(v) => v,
            Err(e) => {
                godot_print!("cartalith-godot: load_appearance_preset parse failed: {e}");
                return false;
            }
        };
        if doc.get("format").and_then(|v| v.as_str()) != Some("cartalith-appearance") {
            godot_print!("cartalith-godot: load_appearance_preset: not a Cartalith appearance preset");
            return false;
        }
        let Some(body) = doc.get("appearance") else {
            godot_print!("cartalith-godot: load_appearance_preset: no appearance block");
            return false;
        };
        match serde_json::from_value::<TerrainAppearance>(body.clone()) {
            Ok(a) => {
                self.appearance_over.clear();
                self.appearance_ramp = None;
                self.appearance_preset = Some(a);
                true
            }
            Err(e) => {
                godot_print!("cartalith-godot: load_appearance_preset decode failed: {e}");
                false
            }
        }
    }

    /// The `name` field of a preset file without loading it, or `""` -- so a
    /// picker can list what is in a folder by its own name rather than by
    /// filename. Cheap enough to call once per file in a directory listing.
    #[func]
    fn peek_appearance_preset(&self, path: GString) -> GString {
        let Ok(text) = std::fs::read_to_string(path.to_string()) else { return GString::new() };
        let Ok(doc) = serde_json::from_str::<serde_json::Value>(&text) else { return GString::new() };
        if doc.get("format").and_then(|v| v.as_str()) != Some("cartalith-appearance") {
            return GString::new();
        }
        GString::from(doc.get("name").and_then(|v| v.as_str()).unwrap_or(""))
    }

    /// The reference's NPR block (`GUI_GAP_REGISTER.md` RN-01) as a
    /// `Dictionary`, keyed exactly as `set_npr` reads it -- so a caller can
    /// round-trip the whole block, and a preset file has one shape to store.
    #[func]
    fn get_npr(&self) -> VarDictionary {
        let n = &self.npr;
        let mut d = VarDictionary::new();
        d.set("watercolor", n.watercolor);
        d.set("contours", n.contours);
        d.set("contour_m", n.contour_m);
        d.set("ink", n.ink);
        d.set("hachure", n.hachure);
        d.set("cel", n.cel);
        d.set("crosshatch", n.crosshatch);
        d.set("stipple", n.stipple);
        d.set("sepia", n.sepia);
        d.set("risograph", n.risograph);
        d.set("pointillism", n.pointillism);
        d.set("waves", n.waves);
        d.set("wave_dist", n.wave_dist);
        d.set("multi_sun", n.multi_sun);
        d.set("animate_water", n.animate_water);
        d
    }

    /// Set the NPR block from a `Dictionary`. **Every key is optional**: a
    /// missing one keeps its current value, so a caller may send one changed
    /// slider rather than the whole block. Returns the number of recognised
    /// keys applied, so a GDScript caller can tell a typo (`0`) from a real
    /// update without this method having to decide what a typo means.
    ///
    /// Intensities are clamped to `[0, 1]` -- the reference's own sliders are
    /// `0..100 / 100` and several styles multiply an intensity straight into
    /// a `1 - a` darkening term, where a value above 1 inverts the colour
    /// rather than intensifying it. Clamping is not "improving on JS": it is
    /// enforcing the range JS's own UI could only ever produce.
    ///
    /// **Presentation only**, on `set_quality_tier`'s exact terms: this never
    /// touches the heightmap, climate, hydrology, biomes, settlements, routes
    /// or the seed. Call `build_color_texture()` again to see it; no
    /// regeneration is needed.
    #[func]
    fn set_npr(&mut self, values: VarDictionary) -> i32 {
        let mut applied = 0i32;
        {
            let mut num = |key: &str, slot: &mut f64, clamp: bool| {
                if let Some(v) = values.get(key)
                    && let Ok(f) = v.try_to::<f64>()
                {
                    *slot = if clamp { f.clamp(0.0, 1.0) } else { f.max(0.0) };
                    applied += 1;
                }
            };
            num("watercolor", &mut self.npr.watercolor, true);
            num("contours", &mut self.npr.contours, true);
            num("ink", &mut self.npr.ink, true);
            num("hachure", &mut self.npr.hachure, true);
            num("cel", &mut self.npr.cel, true);
            num("crosshatch", &mut self.npr.crosshatch, true);
            num("stipple", &mut self.npr.stipple, true);
            num("sepia", &mut self.npr.sepia, true);
            num("risograph", &mut self.npr.risograph, true);
            num("pointillism", &mut self.npr.pointillism, true);
            // Not `[0,1]`: a contour interval is metres (the reference's own
            // slider is 5-50) and the wave reach is a multiplier (0.25-3.0).
            num("contour_m", &mut self.npr.contour_m, false);
            num("wave_dist", &mut self.npr.wave_dist, false);
        }
        let mut flag = |key: &str, slot: &mut bool| {
            if let Some(v) = values.get(key)
                && let Ok(b) = v.try_to::<bool>()
            {
                *slot = b;
                applied += 1;
            }
        };
        flag("waves", &mut self.npr.waves);
        flag("multi_sun", &mut self.npr.multi_sun);
        flag("animate_water", &mut self.npr.animate_water);
        applied
    }

    /// A tier this device can plausibly afford, as a **suggestion**. Nothing
    /// applies it: `WorldGen` starts at `"quality"` on every device, and what
    /// a phone should default to is an owner policy decision, not this
    /// crate's. A caller that wants a device-appropriate default calls this
    /// and then `set_quality_tier`.
    #[func]
    fn get_recommended_quality_tier(&self) -> GString {
        GString::from(render::recommended_quality_tier().name())
    }

    #[func]
    fn get_width(&self) -> i32 {
        self.gw
    }

    #[func]
    fn get_height(&self) -> i32 {
        self.gh
    }

    /// Width of the rendered plate's frame (paper margin + neatlines, Phase
    /// 3 milestone 4) as a **fraction of the texture's own width** — `0.0`
    /// when there is no frame. Returned as a fraction rather than in cells
    /// deliberately: `map_overlay.gd` works in screen pixels against a
    /// letterboxed texture, so a fraction survives the fit/scale maths with
    /// no resolution knowledge on the GDScript side.
    ///
    /// Exists so the marker overlay can keep its content inside the
    /// neatline instead of drawing settlements and roads onto the bare
    /// margin, without hardcoding `render.rs`'s `0.014` a second time.
    ///
    /// Correct for a **non-square** grid without a second value for the
    /// vertical inset: the frame is a uniform cell count on all four sides
    /// and `map_overlay.gd`'s `_displayed_rect()` fit scale is uniform, so
    /// `frac × displayed_width` is the inset in screen pixels on both axes.
    #[func]
    fn get_border_inset_frac(&self) -> f64 {
        let (gw, gh) = (self.gw as usize, self.gh as usize);
        if gw == 0 || gh == 0 {
            return 0.0;
        }
        render::border_width_cells(&self.appearance(), gw, gh) / gw as f64
    }

    /// Builds a colour + hillshade texture from the last `generate()`
    /// result. Ported from the reference HTML's own default-settings
    /// renderer (`render.rs`'s doc comment lists exactly what's ported vs.
    /// deliberately excluded) — no longer the MVP placeholder tint this
    /// method used before. A blue tint on channelized cells stands in for
    /// the reference's vector river overlay (`drawRiverWays`, not wired
    /// into this port), keeping "rivers visible" (`MVP_SCOPE.md`'s "done"
    /// checklist, point 2) satisfied. Returns `None` before the first
    /// `generate()` call.
    #[func]
    fn build_color_texture(&self) -> Option<Gd<ImageTexture>> {
        let (field, temperature, rainfall, flow, chan_mask) = match self.source.as_ref()? {
                WorldSource::Generated(ws) => (
                    &ws.field,
                    &ws.temperature,
                    &ws.rainfall,
                    Some(ws.flow_discharge.as_slice()),
                    ws.channels.as_ref().map(|c| c.chan.as_slice()),
                ),
                WorldSource::Loaded(save) => (
                    &save.fields.heightmap,
                    &save.fields.temperature,
                    &save.fields.rainfall,
                    None,
                    Some(save.fields.strahler_order.as_slice()),
                ),
            };
        let gw = self.gw as usize;
        let gh = self.gh as usize;
        let appearance = self.appearance();
        // Milestone 5 (`TERRAIN_APPEARANCE_SCOPE.md`, research §12): the
        // world's real rock types. Built here rather than threaded down from
        // `compute_civilisation` (which builds its own for the soil chain)
        // because the renderer must also work for a world generated with the
        // civilisation layer switched off, and because holding a second copy
        // on `WorldGen` would be a `gw*gh` byte field kept alive for the
        // lifetime of the world to save one single-pass, neighbour-free
        // `par_iter` -- the wrong trade at this port's 8192 ceiling.
        //
        // `None` for a loaded save: its format stores none of the tectonic
        // substrate this needs (`SAVEFILE_COMPAT.md`), the same reason
        // `flow` is `None` there and `CivData` is never computed for one.
        let lithology = match self.source.as_ref()? {
            WorldSource::Generated(ws) => Some(cartalith_civ::build_lithology(
                &ws.field, &ws.age_field, &ws.volcanic_field, &ws.crust_field, &ws.resistance_field, &ws.rainfall, self.sea_level,
            )),
            WorldSource::Loaded(_) => None,
        };
        let mut ctx = RenderCtx::with_appearance(
            field, temperature, rainfall, flow, gw, gh, self.sea_level, self.world, self.lat_n, self.lat_s, appearance.clone(),
        );
        if let Some(lith) = lithology.as_ref() {
            ctx = ctx.with_lithology(lith);
        }
        // Milestone 7 (`ASSET_LIBRARY_SCOPE.md`): attach real ground-texture
        // splat channels whenever a pack is loaded. `SplatTextures::default()`
        // (all `None`) is what a pack-less or textures-less pack produces --
        // `land_color`'s own splat branch never activates without at least
        // one `Some`, matching the reference's `texAny` gate exactly.
        if let Some(loaded) = self.asset_pack.as_ref() {
            let splat = SplatTextures {
                grass: loaded.splat.get("grass"),
                rock: loaded.splat.get("rock"),
                sand: loaded.splat.get("sand"),
                snow: loaded.splat.get("snow"),
                wetland: loaded.splat.get("wetland"),
                canopy: loaded.splat.get("canopy"),
            };
            ctx = ctx.with_splat(splat);
        }

        // The Cartography paint brush's three override grids
        // (`UNIFIED_TOOL_PLAN.md` milestone C/F). Without this the tool was
        // fully functional and completely invisible: `paint_commit` wrote
        // real cells, `build_paint_preview_texture` drew them as a separate
        // overlay, and the map itself never changed -- while the reference's
        // own `_paintAt` ends in `render()` and tints the map on the very
        // first dab. `render::land_color`'s own paint blend is the port of
        // that tint (`landColorCore` 7897-7901); this is its only producer.
        //
        // **Committed cells only, not the live draft.** The reference has no
        // draft stage for paint at all (`cartalith-spatial/src/paint.rs`'s
        // own "the only divergence"), so there is no reference answer for
        // what an uncommitted dab should do to the map; showing the
        // *committed* state here matches Sculpt's own draft/commit split
        // exactly, and the in-flight draft is what
        // `build_paint_preview_texture`'s overlay is for.
        if let Some(p) = self.paint.as_ref() {
            ctx = ctx.with_paint(p.layer_cells(paint_bridge::PaintTarget::Biome), p.layer_cells(paint_bridge::PaintTarget::Terrain), p.layer_cells(paint_bridge::PaintTarget::Splat));
        }

        // Milestone 6 (`TERRAIN_APPEARANCE_SCOPE.md`, research §21/§23): one
        // `rayon` row-parallel pass instead of one serial `for y`. Five
        // milestones of appearance work had grown this loop from a hillshade
        // and a palette lookup into ~1.1 s of `vnoise`, `exp` and `smoothstep`
        // at the app's own 2048x1311 -- on a single core, while the generation
        // pipeline feeding it has been Rayon-parallel since
        // `CPU_MULTITHREADING_SCOPE.md` milestones 2-3.
        //
        // **Bit-identical, not approximately so.** `cell_color` is a pure
        // function of `(&ctx, x, y)` -- it reads only shared immutable
        // slices and precomputed tables, accumulates nothing across pixels,
        // and no float is reassociated by the split. Each row writes its own
        // disjoint `gw * 3` bytes, so the output does not depend on how rayon
        // schedules the chunks (verified by
        // `render_parallel_matches_serial` in `tests/appearance_tiers.rs`).
        // §27's determinism therefore holds by construction, not by
        // convention -- the same standard `DECISIONS.md` §7a asks of the
        // parity path.
        let mut bytes = vec![0u8; gw * gh * 3];
        bytes.par_chunks_mut(gw * 3).enumerate().for_each(|(y, row)| {
            for x in 0..gw {
                let i = y * gw + x;
                let (mut r, mut g, mut b) = render::cell_color(&ctx, x, y);

                if let Some(mask) = chan_mask
                    && mask[i] != 0
                {
                    // The tint composites *over* a colour `cell_color` has
                    // already stamped the plate frame onto (milestone 4), so
                    // it has to fade back out exactly as the frame fades in
                    // — otherwise a river reaching the sheet edge paints
                    // blue across what is supposed to read as bare paper.
                    // `border_cover` is `0.0` throughout the plate interior
                    // and `0.0` everywhere when there is no frame, and
                    // `t + (v - t) * 0.0 == t` exactly, so the unframed
                    // tint is bit-identical to what it was before.
                    let cover = render::border_cover(&appearance, x, y, gw, gh);
                    if cover < 1.0 {
                        let (tr, tg, tb) = (r * 0.5, (g * 0.5 + 0.3).min(1.0), (b * 0.5 + 0.45).min(1.0));
                        r = tr + (r - tr) * cover;
                        g = tg + (g - tg) * cover;
                        b = tb + (b - tb) * cover;
                    }
                }

                let o = x * 3;
                row[o] = (r.clamp(0.0, 1.0) * 255.0) as u8;
                row[o + 1] = (g.clamp(0.0, 1.0) * 255.0) as u8;
                row[o + 2] = (b.clamp(0.0, 1.0) * 255.0) as u8;
            }
        });

        // Milestone 5 (`TERRAIN_APPEARANCE_SCOPE.md`, research §18): local
        // contrast. The one appearance stage that cannot live inside
        // `cell_color`, since it reads a *neighbourhood* of the finished
        // colour -- see `render::apply_local_contrast`'s own doc comment.
        //
        // Placed here deliberately: after the river channel tint (so a
        // river reads with the same separation from its valley that every
        // other material gets) and before the icon pass (drawn artwork is
        // not terrain and has no business being contrast-boosted). A no-op
        // whenever `local_contrast == 0.0`.
        render::apply_local_contrast(&appearance, &mut bytes, gw, gh, self.world);

        // The colour grade (2026-08-24) -- the last stage that is about the
        // *terrain image*. Placed after local contrast, and before the icon
        // pass below for the same reason local contrast is: drawn artwork is
        // not terrain, and rivers, labels, settlement markers, territory and
        // the scale bar are Godot overlays composited over this texture, so
        // everything downstream of here is furniture rather than ground.
        // A no-op whenever every grade parameter is at rest.
        //
        // The four field-influence weights are built here rather than inside
        // the pass because they are a *field* quantity and the pass only ever
        // sees a byte buffer -- `render::build_grade_influence` returns an
        // empty `Vec` (and the pass then grades flat) whenever all four are at
        // rest, which is the default.
        let grade_influence = render::build_grade_influence(&ctx, gw, gh);
        render::apply_color_grade(&appearance, &mut bytes, &grade_influence);

        // Milestone 7: `drawMapIcons`' own painter's pass, composited over
        // the finished raster exactly as it is in the reference (a separate
        // canvas draw call after the terrain fill, not blended into
        // `cell_color`'s own per-pixel loop above). A no-op whenever no
        // pack is loaded or the loaded pack's scatter-rule table ends up
        // empty (`pack::composite_map_icons`'s own doc comment) -- which is
        // this port's only state today, since it ships no default pack.
        if let Some(loaded) = self.asset_pack.as_ref() {
            pack::composite_map_icons(&mut bytes, field, temperature, rainfall, gw, gh, self.sea_level, self.seed, loaded);
        }

        let packed = PackedByteArray::from(bytes);
        let image = Image::create_from_data(gw as i32, gh as i32, false, Format::RGB8, &packed)?;
        ImageTexture::create_from_image(&image)
    }

    /// Per-cell territory ownership (`DECISIONS.md` §7b), as a semi-
    /// transparent RGBA overlay -- same Okabe-Ito palette
    /// (`map_overlay.gd`'s own `FACTION_COLORS`, duplicated here as plain
    /// `u8` triples since this crate has no reason to depend on Godot's
    /// `Color` type for a pure byte-buffer build) at low alpha so the
    /// terrain/biome colours underneath still read through -- this is
    /// presentation-only map content, independent of UI chrome, the same
    /// principle `build_color_texture` and every settlement/road marker
    /// already follow. Unowned cells (`territory[i] == 0` -- water, or
    /// unreachable from any capital) are fully transparent. `None` before
    /// any `generate()` call, after `load_save()`, or if territory hasn't
    /// been computed for this world (see `CivData::territory`'s own doc
    /// comment -- always computed when `civ` is `Some`, so in practice
    /// this is `None` under exactly the same conditions as
    /// `get_settlements()`/`get_roads()` returning empty).
    #[func]
    fn build_territory_texture(&self) -> Option<Gd<ImageTexture>> {
        let civ = self.civ.as_ref()?;
        let gw = self.gw as usize;
        let gh = self.gh as usize;
        // `state.viz.territoryOpacity` (`#territoryOpacityR`, reference line
        // 1490; applied at 15440 as `Math.round(opacity * 255)`) --
        // `GUI_GAP_REGISTER.md` **CA-17**. The reference's own default is
        // `130/255`; this port's is `TERRITORY_ALPHA_DEFAULT`'s 82/255,
        // which is where this constant has always been and is deliberately
        // not moved to the reference's -- a heavier wash than this one hides
        // the biome underneath it, and the terrain renderer here is doing
        // more work under the wash than the reference's is.
        let alpha = (self.territory_opacity.clamp(0.0, 1.0) * 255.0).round() as u8;

        // Same plate-frame rule the river tint follows: this wash is drawn
        // over the finished raster, and a faction whose territory reaches
        // the sheet edge would otherwise colour the bare-paper margin.
        // `border_cover` is `0.0` across the whole interior (and everywhere
        // when there is no frame), so `alpha` is untouched there.
        let appearance = self.appearance();
        let mut bytes = Vec::with_capacity(gw * gh * 4);
        for (i, &f) in civ.territory.iter().enumerate() {
            let cover = render::border_cover(&appearance, i % gw, i / gw, gw, gh);
            if f > 0 && cover < 1.0 {
                let (r, g, b) = civ.faction_rgb(f);
                bytes.extend_from_slice(&[r, g, b, (alpha as f64 * (1.0 - cover)) as u8]);
            } else {
                bytes.extend_from_slice(&[0, 0, 0, 0]);
            }
        }
        let packed = PackedByteArray::from(bytes);
        let image = Image::create_from_data(gw as i32, gh as i32, false, Format::RGBA8, &packed)?;
        ImageTexture::create_from_image(&image)
    }

    /// Phase 2 civilisation layer (`cartalith-civ`): one `Dictionary` per
    /// settlement with keys `x`/`y` (grid cell, int), `name` (String),
    /// `population` (int), `kind` (String: "metropolis"/"capital"/"city"/
    /// "town"/"village"/"hamlet" -- `journey_bridge::settlement_kind_key`
    /// is the single source of that vocabulary), `faction` (int, `1..=6`, matching
    /// `CIV_FACTION_COUNT`), `capital` (bool), `coastal` (bool), `tid` (int
    /// -- `NamedSettlement::tid`'s stable timeline id, `0` meaning
    /// "unassigned"; in practice always nonzero once `compute_civilisation`
    /// has run, since that's where real assignment happens -- see that
    /// field's own doc comment). Empty before any `generate()` call, after
    /// `load_save()` (no civ data for a loaded save, see `load_save`'s own
    /// doc comment), or if generation produced zero settlement candidates.
    #[func]
    fn get_settlements(&self) -> Array<VarDictionary> {
        let Some(civ) = self.civ.as_ref() else { return Array::new() };
        civ.settlements
            .iter()
            .map(|s| {
                let kind_str = journey_bridge::settlement_kind_key(s.placement.kind);
                vdict! {
                    "x" => s.placement.x as i32,
                    "y" => s.placement.y as i32,
                    "name" => s.name.as_str(),
                    "population" => s.pop as i32,
                    "kind" => kind_str,
                    "faction" => s.placement.faction,
                    "capital" => s.placement.capital,
                    "coastal" => s.placement.coastal,
                    "tid" => s.tid as i64,
                }
            })
            .collect()
    }

    /// Road network (`cartalith-civ::civ_consolidate_and_smooth_ways`,
    /// Phase 2 milestone 14): one `Dictionary` per visible way — `points`
    /// (a `PackedVector2Array`, already smoothed full-resolution map
    /// coordinates, not raw grid-cell indices) and `way_type` (`"highway"`/
    /// `"regional"`/`"road"`/`"track"`, by peak corridor usage) — draw as
    /// a polyline, weight/colour by type if desired. Hidden ways (an edge
    /// fully consolidated away into a busier neighbour, reference's own
    /// junction-continuity behaviour) are omitted entirely, not emitted as
    /// a 2-point stub — nothing to draw for those. Empty under the same
    /// conditions as `get_settlements`.
    ///
    /// **Hand-drawn ways are in here too** (`GUI_GAP_REGISTER.md` IN-02).
    /// Every `ManualWay` the Way tool has committed this session
    /// (`self.infra.ways`) is appended after the generated network, with
    /// `way_type` carrying `infra_tools_bridge::way_type_key`'s own
    /// vocabulary (`road`/`track`/`ancient`) and `manual` set `true`. This
    /// is the reference's arrangement, not a convenience: `_civCommitWay`
    /// (reference line 26077) pushes straight onto the same flat `civWays`
    /// array the generated network lives in, tagged `manual:true`, and the
    /// draw pass branches on `type` alone — so a hand-drawn `road` and a
    /// generated `road` are drawn identically, and `manual` exists to
    /// *survive a network rebuild* (`_civAutoRoutes` filters
    /// `civWays.filter(w => w.manual)`) and to be listable, never to be
    /// styled differently. Callers wanting only the user's own ways filter
    /// on `manual`; callers drawing the map should not.
    ///
    /// The one exception is a manual **sea lane**: `way_type == "sea_lane"`
    /// is a different routing domain, and the reference draws it with the
    /// navy/dashed sea style rather than any road style, so those are
    /// emitted from `get_sea_routes()` instead of here — same split this
    /// port's `map_overlay.gd` already draws along.
    ///
    /// `km` (float) and `manual` (bool) are on every entry, generated ones
    /// included; `km` is `Way::km`/`ManualWay::km`, the real routed length.
    ///
    /// # `points` is the way's curve at render density, not its control points
    ///
    /// `Way::pts` is `_civSmoothPath`'s own output: the Catmull-Rom curve
    /// sampled every 3 grid cells and rounded to whole cells. The reference
    /// draws that list with `lineTo`, and can, because it draws the map at
    /// roughly one grid cell per screen pixel -- a 3-cell chord is a 3 px
    /// chord and the polyline reads as the curve it came from.
    ///
    /// This port's viewport is a zoomable DCC surface. Fit to the panel a
    /// 384-cell grid is already ~3.6 px per cell, and `ViewportHost.ZOOM_MAX`
    /// multiplies that by 8: one grid cell can be ~29 screen px, so the same
    /// 3-cell chord is an ~87 px straight line and every corner in the curve
    /// is a visible angle. Owner report, 2026-08-24: *"settlement roads all
    /// render as straight lines -- no organic curvature."* Measured on a
    /// 384x288 world, the ways really do curve (mean sinuosity 1.07, ~11
    /// degrees of turn per vertex); what reached the screen was the chords
    /// between their vertices.
    ///
    /// So `points` is that same curve re-sampled through the same control
    /// points at [`WAY_RENDER_STEP_CELLS`] -- `cartalith_civ::
    /// civ_catmull_rom_sample`, the one definition, not a second smoothing
    /// pass. Nothing upstream moves: `Way::pts` (and therefore `km`, the
    /// network metrics, and `urban_adapter::um_primary_paths`, which reads
    /// the ways directly) is untouched and every road golden-parity test
    /// still asserts against it. The reference's own precedent for treating
    /// a grid-quantised road coordinate as too coarse *once LOD zoom exists*
    /// is `_civSmoothPath`'s v0.92 note, which un-rounds a way's endpoints
    /// for exactly this reason ("imperceptible at low zoom but, amplified by
    /// LOD zoom (one grid cell can span many screen pixels), visibly..."):
    /// this is that same observation applied to the interior.
    ///
    /// `brks` is remapped onto the re-sampled list, so a break still lands
    /// between the two runs it separates.
    #[func]
    fn get_roads(&self) -> Array<VarDictionary> {
        let Some(civ) = self.civ.as_ref() else { return Array::new() };
        // `brks` on every entry below: indices into `points` where that
        // way's own path has a real gap (two disjoint consolidated runs
        // sharing one `Way`, not a continuous curve) -- drawing straight
        // through these would render a phantom line across the gap, so
        // the renderer must split into separate strokes there instead of
        // treating `points` as one polyline.
        let mut out: Array<VarDictionary> = civ
            .ways
            .iter()
            .filter(|w| !w.hidden)
            .map(|w| {
                let (points, brks) = way_render_geometry(&w.pts, &w.brks);
                let way_type = match w.way_type {
                    cartalith_civ::WayType::Highway => "highway",
                    cartalith_civ::WayType::Regional => "regional",
                    cartalith_civ::WayType::Road => "road",
                    cartalith_civ::WayType::Track => "track",
                };
                dict! { "points" => &points, "brks" => &brks, "way_type" => way_type, "name" => w.name.as_str(), "km" => w.km, "manual" => false }
            })
            .collect();
        if let Some(infra) = self.infra.as_ref() {
            for w in infra.ways.iter().filter(|w| !w.hidden && !w.sea) {
                let (points, brks) = way_render_geometry(&w.pts, &w.brks);
                let way_type = infra_tools_bridge::way_type_key(w.way_type);
                out.push(&dict! { "points" => &points, "brks" => &brks, "way_type" => way_type, "name" => w.name.as_str(), "km" => w.km, "manual" => true });
            }
        }
        out
    }

    /// Sea-lane routes (`cartalith_civ::civ_sea_routes`, Phase 2 milestone
    /// 13) -- same `{points, brks, name}` shape as `get_roads()`, minus
    /// `way_type` (sea routes have no highway/regional/road/track tier,
    /// `SeaRoute` doesn't carry one). Draw distinctly from land roads --
    /// the reference's own convention (line ~15511) is a dark navy
    /// underlayer plus a lighter dashed overlay, not a road colour/width.
    /// Empty under the same conditions as `get_roads()`.
    ///
    /// **Hand-drawn sea lanes are in here too** (`GUI_GAP_REGISTER.md`
    /// IN-02): a committed `ManualWay` whose `way_type` is `sea_lane`
    /// (equivalently `ManualWay::sea`) is appended after the generated
    /// lanes with `manual` set `true`. It arrives here rather than in
    /// `get_roads()` because that is the *drawn* distinction the reference
    /// makes — its single `civWays` loop sends `type === 'sea-lane'` down
    /// the navy/dashed branch, never a road branch — and this port already
    /// splits those two styles across these two getters. `manual` and `km`
    /// are on every entry, generated ones included.
    ///
    /// # `points` is the lane's curve at render density
    ///
    /// Exactly as in `get_roads()`, and for the same reason: a sea lane's
    /// `pts` come from the same `civ_smooth_path` (`civ_sea_routes` for a
    /// generated lane, `civ_join_dijkstra_segs` for a committed one), so
    /// they are the same Catmull-Rom curve sampled every 3 grid cells and
    /// rounded — an ~87 px straight chord at `ViewportHost.ZOOM_MAX`. See
    /// `get_roads()`' own doc comment for the measurement and the
    /// precedent; this is the identical re-sample through
    /// [`WAY_RENDER_STEP_CELLS`], and `SeaRoute::pts`/`ManualWay::pts` (and
    /// therefore `km`) are equally untouched.
    #[func]
    fn get_sea_routes(&self) -> Array<VarDictionary> {
        let Some(civ) = self.civ.as_ref() else { return Array::new() };
        let mut out: Array<VarDictionary> = civ
            .sea_routes
            .iter()
            .map(|r| {
                let (points, brks) = way_render_geometry(&r.pts, &r.brks);
                dict! { "points" => &points, "brks" => &brks, "name" => r.name.as_str(), "km" => r.km, "manual" => false }
            })
            .collect();
        if let Some(infra) = self.infra.as_ref() {
            for w in infra.ways.iter().filter(|w| !w.hidden && w.sea) {
                let (points, brks) = way_render_geometry(&w.pts, &w.brks);
                out.push(&dict! { "points" => &points, "brks" => &brks, "name" => w.name.as_str(), "km" => w.km, "manual" => true });
            }
        }
        out
    }

    /// Province metadata (`cartalith_civ::civ_generate_provinces`) -- one
    /// `Dictionary` per province with keys `id` (int, matches
    /// `get_province_id_at`... see `build_province_boundary_texture`'s own
    /// doc comment for how a province id maps back to a cell), `faction`
    /// (int, `1..=6`), `name` (String), `capital_settlement_index` (int,
    /// index into `get_settlements()`'s own array -- the seed settlement
    /// this province was subdivided from). Empty under the same conditions
    /// as `get_settlements()`/`get_roads()`.
    #[func]
    fn get_provinces(&self) -> Array<VarDictionary> {
        let Some(civ) = self.civ.as_ref() else { return Array::new() };
        civ.province_list
            .iter()
            .map(|p| {
                dict! {
                    "id" => p.id,
                    "faction" => p.faction,
                    "name" => p.name.as_str(),
                    "capital_settlement_index" => p.capital_settlement_index as i64,
                }
            })
            .collect()
    }

    /// Addressable landmasses (`cartalith_civ::civ_continents`,
    /// `MARKDOWN_VAULT_SCOPE.md` milestone 0) — one `Dictionary` per
    /// continent, **largest first**, with keys `id` (int, 1-based rank by
    /// area — see below), `name` (String), `cells` (int, land cells),
    /// `faction` (int, the polity holding the most of it, `0` = unclaimed),
    /// `min_x`/`min_y`/`max_x`/`max_y` (int, inclusive cell bounds) and
    /// `cx`/`cy` (float, cell-space centroid, for focusing the camera).
    /// Empty under the same conditions as `get_settlements()`.
    ///
    /// # `id` is a rank, not a persistent identity
    ///
    /// Stated at the binding because it is what a caller has to know. This
    /// port had no continent entity at all until this milestone; what it had
    /// was `build_landmass_quality`'s connected-component labelling, computed
    /// and discarded on every generate. `id` is that partition ranked by
    /// area, so it is stable across anything that does not change the size
    /// ordering — and it is *not* stable across a terrain edit that merges or
    /// splits a landmass, or across a regenerate with different parameters.
    ///
    /// A settlement's `tid` is a real stable id; a continent's is not, and no
    /// amount of wanting one makes the underlying data carry one. The vault
    /// integration works with that rather than around it: every knowledge
    /// link also stores the entity's name at link time, so a link whose id
    /// has gone stale can be re-bound by hand instead of quietly resolving to
    /// a different landmass.
    ///
    /// Landmasses under `CONTINENT_MIN_CELLS` are omitted (see that constant).
    #[func]
    fn get_continents(&self) -> Array<VarDictionary> {
        let Some(civ) = self.civ.as_ref() else { return Array::new() };
        civ.continents
            .iter()
            .map(|c| {
                dict! {
                    "id" => c.id,
                    "name" => c.name.as_str(),
                    "cells" => c.cells as i64,
                    "faction" => c.faction,
                    "min_x" => c.min_x as i64,
                    "min_y" => c.min_y as i64,
                    "max_x" => c.max_x as i64,
                    "max_y" => c.max_y as i64,
                    "cx" => c.cx,
                    "cy" => c.cy,
                }
            })
            .collect()
    }

    /// Why the settlement at `index` is where it is (`VISION.md`'s causal
    /// chain) -- the real decomposition of `build_settlement_suitability`'s
    /// score at that settlement's own cell, not a template.
    ///
    /// Returns an empty `Dictionary` for an out-of-range index or before
    /// any generate. Otherwise:
    /// - `score` (float) -- the settlement's suitability, identical to
    ///   the value that placed it.
    /// - `terms` (Array of Dictionary) -- every weighted term, **sorted
    ///   most-decisive first**, each with `key` (stable String id like
    ///   `"river"`/`"farmland"`/`"flood_risk"`), `value` (0..1 raw
    ///   reading), `weight` (signed -- negative for the two penalties),
    ///   and `contribution` (`weight * value`, the signed amount this
    ///   term moved the score).
    /// - `excluded` (String) -- present only for a cell the suitability
    ///   pass skips outright (`"below_sea_level"` / `"water_body"`).
    /// - context readings named by the causal chain, each straight from a
    ///   real raster: `elevation`, `coast_dist_cells` (negative offshore),
    ///   `river_order` (Strahler, 0 = no river), `flow`, `travel_cost`,
    ///   `biome`.
    ///
    /// Deliberately keyed by settlement index rather than `(x, y)`: the
    /// rasters this is derived from live only inside `compute_civilisation`
    /// and retaining them for arbitrary-cell queries would cost hundreds of
    /// MB at production resolutions (`MEMORY_OPTIMIZATION_SCOPE.md`). All
    /// wording is left to the caller -- this returns facts, not prose.
    #[func]
    fn explain_settlement(&self, index: i64) -> VarDictionary {
        let Some(civ) = self.civ.as_ref() else { return VarDictionary::new() };
        let Some(e) = usize::try_from(index).ok().and_then(|i| civ.explanations.get(i)) else {
            return VarDictionary::new();
        };
        let terms: Array<VarDictionary> = e
            .suit
            .terms
            .iter()
            .map(|t| {
                dict! {
                    "key" => t.key,
                    "value" => t.value,
                    "weight" => t.weight,
                    "contribution" => t.contribution,
                }
            })
            .collect();
        let mut out = dict! {
            "score" => e.suit.score,
            "terms" => &terms,
            "elevation" => e.elevation,
            "coast_dist_cells" => e.coast_dist_cells,
            "river_order" => e.river_order as i64,
            "flow" => e.flow,
            "travel_cost" => e.travel_cost,
            "biome" => e.biome as i64,
        };
        if let Some(reason) = e.suit.excluded {
            out.set("excluded", reason);
        }
        out
    }

    /// Per-settlement resource trade balance (`cartalith_civ::
    /// civ_resource_trade_balance`, `ECONOMY_SCOPE.md`) -- one `Dictionary`
    /// per entry in `get_settlements()`, same order/index, with `exports`
    /// and `imports` as `PackedStringArray`s of `CIV_RESOURCE_KEYS` names
    /// (may be empty -- a settlement with no strong local surplus/deficit
    /// has no trade relationship, a real outcome, not missing data). This
    /// is the reference's `_civPlaceTrade` hinterland term only, not the
    /// full faction-level `_civFactionAggregates` aggregation (population,
    /// tax, the five-axis "power" heuristic) -- that remains unstarted,
    /// real future scope (`ECONOMY_SCOPE.md`'s own next-milestones list).
    /// Empty under the same conditions as `get_settlements()`.
    #[func]
    fn get_trade_balances(&self) -> Array<VarDictionary> {
        let Some(civ) = self.civ.as_ref() else { return Array::new() };
        civ.trade_balances
            .iter()
            .map(|t| {
                let exports: PackedStringArray = t.exports.iter().map(|s| GString::from(*s)).collect();
                let imports: PackedStringArray = t.imports.iter().map(|s| GString::from(*s)).collect();
                dict! { "exports" => &exports, "imports" => &imports }
            })
            .collect()
    }

    /// Province *boundaries* as a semi-transparent RGBA overlay -- a line
    /// wherever two orthogonally-adjacent cells belong to different
    /// (nonzero) provinces, transparent everywhere else. Deliberately not a
    /// per-province fill colour the way `build_territory_texture` fills by
    /// faction: a province count isn't bounded the way `CIV_FACTION_COUNT`
    /// is, so there's no fixed small palette to draw from without a real
    /// UI/UX design pass picking one -- boundary lines need no palette at
    /// all and read clearly layered on top of `build_territory_texture`'s
    /// own per-faction fill. `None` under the same conditions as
    /// `build_territory_texture`.
    ///
    /// The line is drawn 3px wide at full grid resolution, not 1px --
    /// found by real windowed-app screenshot verification (not assumed)
    /// that a literal single-cell-wide line at e.g. 2048px source
    /// resolution becomes sub-pixel once `TextureRect` downscales it to
    /// fit a typical viewport width, anti-aliasing it into near-invisible
    /// mush indistinguishable from roads/coastline. A real stroke width
    /// survives that downscale the way an actual cartographic line does.
    #[func]
    fn build_province_boundary_texture(&self) -> Option<Gd<ImageTexture>> {
        let civ = self.civ.as_ref()?;
        let gw = self.gw as usize;
        let gh = self.gh as usize;
        const LINE_RGBA: [u8; 4] = [35, 24, 9, 235]; // map_overlay.gd's ink tone, alpha nudged up (not to opaque)

        // Pass 1: symmetric boundary detection (checks all four neighbours,
        // not just +x/+y) so a boundary is a property of the edge, not of
        // which cell happened to be scanned first.
        let mut boundary = vec![false; gw * gh];
        for y in 0..gh {
            for x in 0..gw {
                let i = y * gw + x;
                let here = civ.provinces[i];
                if here == 0 {
                    continue;
                }
                boundary[i] = (x + 1 < gw && civ.provinces[i + 1] != here)
                    || (y + 1 < gh && civ.provinces[i + gw] != here)
                    || (x > 0 && civ.provinces[i - 1] != here)
                    || (y > 0 && civ.provinces[i - gw] != here);
            }
        }

        // Pass 2: dilate by one cell (3x3 neighbourhood) for a real ~3px
        // stroke instead of the single-pixel line that proved illegible.
        // Pass 3 is the plate frame (milestone 4), same rule as the river
        // tint and the territory wash: this line is drawn over the finished
        // raster, so it fades out as the bare-paper margin fades in rather
        // than ruling straight across it. No-op without a frame.
        let appearance = self.appearance();
        let mut bytes = vec![0u8; gw * gh * 4];
        for y in 0..gh {
            for x in 0..gw {
                let y0 = y.saturating_sub(1);
                let y1 = (y + 1).min(gh - 1);
                let x0 = x.saturating_sub(1);
                let x1 = (x + 1).min(gw - 1);
                let near = (y0..=y1).any(|ny| (x0..=x1).any(|nx| boundary[ny * gw + nx]));
                let cover = render::border_cover(&appearance, x, y, gw, gh);
                if near && cover < 1.0 {
                    let i = y * gw + x;
                    let mut px = LINE_RGBA;
                    px[3] = (px[3] as f64 * (1.0 - cover)) as u8;
                    bytes[i * 4..i * 4 + 4].copy_from_slice(&px);
                }
            }
        }
        let packed = PackedByteArray::from(bytes);
        let image = Image::create_from_data(gw as i32, gh as i32, false, Format::RGBA8, &packed)?;
        ImageTexture::create_from_image(&image)
    }

    // ---- Sculpt editor: registry (UNIFIED_TOOL_PLAN.md milestone F) ----
    //
    // Read-only enumeration -- feature list, presets, the shared brush/
    // noise globals, Freehand's sub-modes -- mirroring params.rs's own
    // flat-table approach (that file's module doc has the full argument)
    // so GDScript hardcodes none of the 13 features' ranges/defaults, the
    // 8 presets' seeded values, or the 8 globals' ranges. Every one of
    // these methods works before any `generate()` call -- the registry is
    // static data, not per-world state.

    /// The 13-entry Sculpt feature registry (`cartalith_terrain::sculpt::
    /// FEATURE_KEYS`, `DCC_SHELL_SPEC.md` §5.2) as an `Array<Dictionary>`,
    /// one entry per feature in **registry order** -- `index` is that
    /// order and is load-bearing (`SCULPT_FUNCTION_CHART.md` §2: it feeds
    /// the stamp's own noise seed, `(seed ^ ((index+1)*1013)) >>> 0`); a
    /// shell may re-*group* features visually but must send `key` back to
    /// `sculpt_set_feature`, never a re-sorted `index`. Each entry:
    /// - `index` (int), `key` (String), `label` (String), `icon` (String,
    ///   an emoji -- `sculpt_bridge`'s module doc and `SCULPT_FUNCTION_
    ///   CHART.md` §9 both note the real icon is `shell/dcc_icons.gd`'s
    ///   own drawn glyph, not this string; kept anyway since it is real
    ///   registry data, harmless to expose, and a fallback costs nothing),
    /// - `hint` (String), `radial` (bool -- Lake/Volcano only),
    /// - `modes` (`PackedStringArray`, empty except Freehand's 8),
    /// - `controls` (`Array<Dictionary>`, this feature's own parameter
    ///   table, same shape `get_param_info` uses per generation parameter).
    #[func]
    fn get_sculpt_features(&self) -> Array<VarDictionary> {
        cartalith_terrain::sculpt::FEATURE_KEYS
            .iter()
            .enumerate()
            .map(|(index, &f)| {
                let m = f.meta();
                let controls: Array<VarDictionary> = m.controls.iter().map(control_dict).collect();
                let modes: PackedStringArray = m.modes.iter().map(|s| GString::from(*s)).collect();
                vdict! {
                    "index" => index as i32,
                    "key" => m.key,
                    "label" => m.label,
                    "icon" => m.icon,
                    "hint" => m.hint,
                    "radial" => m.radial,
                    "modes" => &modes,
                    "controls" => &controls,
                }
            })
            .collect()
    }

    /// The 8 one-click presets (`SCULPT_PRESETS`, `DCC_SHELL_SPEC.md`
    /// §5.2), each a parameter seed -- **applying one never paints**, the
    /// caller still draws the stroke (`sculpt_apply_preset` sets the tool
    /// state; nothing here touches the draft). One `Dictionary` per
    /// preset, in the reference's own order: `name` (String), `feature`
    /// (String key, matches `get_sculpt_features`' own `key`),
    /// `noise_scale` (float -- the one global every preset overrides),
    /// `params` (`Dictionary`, the feature params this preset seeds, same
    /// control-key shape `sculpt_get_feature_params` returns).
    #[func]
    fn get_sculpt_presets(&self) -> Array<VarDictionary> {
        SCULPT_PRESETS
            .iter()
            .map(|preset| {
                let mut g = cartalith_terrain::sculpt::SculptGlobals::default();
                let params = preset.apply(&mut g);
                vdict! {
                    "name" => preset.name,
                    "feature" => preset.feature.meta().key,
                    "noise_scale" => preset.noise_scale,
                    "params" => &feature_params_dict(&params),
                }
            })
            .collect()
    }

    /// The 8 shared brush/noise globals (`SculptGlobals`, `DCC_SHELL_SPEC.md`
    /// §5.2's "Brush & noise · global" block) as an `Array<Dictionary>`,
    /// same shape [`WorldGen::get_sculpt_features`]' `controls` use plus a
    /// `type` field (`"int"` for `octaves`, `"float"` for the other 7).
    ///
    /// **`default` is the engine's own `SculptGlobals::default()` value,
    /// not the design spec's -- settled, not open.** `SCULPT_FUNCTION_
    /// CHART.md` §4 found five of the eight disagree between
    /// `DCC_SHELL_SPEC.md` and the golden-pinned `SCULPT_GLOBAL_DEF` this
    /// reads from; the owner has since resolved it in the engine's favour,
    /// on a concrete ground beyond precedent: `cartalith-engine/tests/
    /// golden_parity_sculpt_water.rs` spreads `..SculptGlobals::default()`
    /// into its own fixtures, so these numbers are golden-parity *inputs*.
    /// This binding reads them live rather than duplicating them, so this
    /// table can never silently drift from what the golden suite actually
    /// depends on. `DCC_SHELL_SPEC.md` §5.2's differing numbers will be
    /// corrected at the design end, not here.
    #[func]
    fn get_sculpt_globals_info(&self) -> Array<VarDictionary> {
        sculpt_bridge::global_controls()
            .iter()
            .map(|c| {
                vdict! {
                    "key" => c.key,
                    "label" => c.label,
                    "type" => if c.key == "octaves" { "int" } else { "float" },
                    "min" => c.min,
                    "max" => c.max,
                    "step" => c.step,
                    "default" => c.default,
                }
            })
            .collect()
    }

    /// Freehand's 8 sub-mode keys, in registry order (`FreehandMode`'s own
    /// declaration order -- Raise/Lower/Smooth/Cliff/Ridge/Canyon/Mesa/
    /// Volcano). Pass one to `sculpt_set_freehand_mode`.
    #[func]
    fn get_sculpt_freehand_modes(&self) -> PackedStringArray {
        Feature::Freehand.meta().modes.iter().map(|s| GString::from(*s)).collect()
    }

    // ---- Sculpt editor: current tool state ----

    /// The 8 shared brush/noise globals' **current** values (see
    /// `get_sculpt_globals_info` for their ranges/defaults). Empty before
    /// any `generate()` call.
    #[func]
    fn sculpt_get_globals(&self) -> VarDictionary {
        self.sculpt.as_ref().map_or_else(VarDictionary::new, |s| globals_dict(&s.globals))
    }

    /// Applies a partial `Dictionary` of global key -> value (keys from
    /// `get_sculpt_globals_info`). Same "send only what changed, get back
    /// `{rejected, clamped}`" contract `set_params` documents in full.
    /// A no-op (both arrays empty) before any `generate()` call.
    #[func]
    fn sculpt_set_globals(&mut self, values: VarDictionary) -> VarDictionary {
        let Some(s) = self.sculpt.as_mut() else {
            return dict! { "rejected" => &PackedStringArray::new(), "clamped" => &PackedStringArray::new() };
        };
        apply_sculpt_values(&values, |key, n| sculpt_bridge::set_global(&mut s.globals, key, n))
    }

    /// The feature the next stroke will paint (`get_sculpt_features`' own
    /// `key`). Empty string before any `generate()` call.
    #[func]
    fn sculpt_get_feature(&self) -> GString {
        self.sculpt.as_ref().map_or_else(GString::new, |s| GString::from(s.feature.meta().key))
    }

    /// Selects the feature the next stroke will paint and resets its
    /// parameters to that feature's own defaults (the reference's own
    /// behaviour on a feature switch -- tuning one feature's controls must
    /// not leak into the next). Returns `false`, changing nothing, for an
    /// unknown key or before any `generate()` call.
    #[func]
    fn sculpt_set_feature(&mut self, feature_key: GString) -> bool {
        let Some(s) = self.sculpt.as_mut() else { return false };
        let Some(f) = Feature::from_key(&feature_key.to_string()) else { return false };
        s.feature = f;
        s.params = f.default_params();
        true
    }

    /// The current feature's own live parameter values (control-key ->
    /// float, plus `sub_mode` for Freehand) -- empty `Dictionary` before
    /// any `generate()` call.
    #[func]
    fn sculpt_get_feature_params(&self) -> VarDictionary {
        self.sculpt.as_ref().map_or_else(VarDictionary::new, |s| feature_params_dict(&s.params))
    }

    /// Applies a partial `Dictionary` of the **current feature's own**
    /// control key -> value (keys from that feature's entry in
    /// `get_sculpt_features`' `controls`). Same `{rejected, clamped}`
    /// contract as `sculpt_set_globals` -- a key belonging to a different
    /// feature is reported `rejected`, not silently ignored (see
    /// `sculpt_bridge::set_feature_param`'s own doc comment). Does not
    /// touch Freehand's `sub_mode` -- see `sculpt_set_freehand_mode`.
    #[func]
    fn sculpt_set_feature_params(&mut self, values: VarDictionary) -> VarDictionary {
        let Some(s) = self.sculpt.as_mut() else {
            return dict! { "rejected" => &PackedStringArray::new(), "clamped" => &PackedStringArray::new() };
        };
        let feature = s.feature;
        apply_sculpt_values(&values, |key, n| sculpt_bridge::set_feature_param(&mut s.params, feature, key, n))
    }

    /// Seeds the current feature and its parameters from preset `index`
    /// (0-7, `get_sculpt_presets`' own order) and writes that preset's
    /// `noise_scale` into the live globals -- exactly what
    /// `get_sculpt_presets`' `params`/`noise_scale` describe, applied.
    /// **Never paints**: the caller still draws the stroke. `false`,
    /// changing nothing, for an out-of-range index or before any
    /// `generate()` call.
    #[func]
    fn sculpt_apply_preset(&mut self, index: i32) -> bool {
        let Some(s) = self.sculpt.as_mut() else { return false };
        let Some(preset) = usize::try_from(index).ok().and_then(|i| SCULPT_PRESETS.get(i)) else {
            return false;
        };
        s.params = preset.apply(&mut s.globals);
        s.feature = preset.feature;
        true
    }

    /// Freehand's current sub-mode key, or an empty string when the
    /// current feature isn't Freehand (or before any `generate()` call).
    #[func]
    fn sculpt_get_freehand_mode(&self) -> GString {
        let Some(s) = self.sculpt.as_ref() else { return GString::new() };
        match s.params {
            FeatureParams::Freehand { sub_mode, .. } => GString::from(sub_mode.key()),
            _ => GString::new(),
        }
    }

    /// Sets Freehand's sub-mode (`get_sculpt_freehand_modes`' own keys).
    /// `false`, changing nothing, when the current feature isn't Freehand,
    /// the key isn't one of the 8, or before any `generate()` call.
    #[func]
    fn sculpt_set_freehand_mode(&mut self, mode_key: GString) -> bool {
        let Some(s) = self.sculpt.as_mut() else { return false };
        let Some(mode) = FreehandMode::from_key(&mode_key.to_string()) else { return false };
        let FeatureParams::Freehand { sub_mode, .. } = &mut s.params else { return false };
        *sub_mode = mode;
        true
    }

    /// The seed the *next* stroke will capture into its `SculptStamp`
    /// (`0` before any `generate()` call, otherwise the last generation's
    /// own seed -- the reference's "project seed" default,
    /// `DCC_SHELL_SPEC.md` §5.2).
    #[func]
    fn sculpt_get_seed(&self) -> i64 {
        self.sculpt.as_ref().map_or(0, |s| i64::from(s.seed))
    }

    /// Sets the seed the *next* stroke will capture (a shell's dice button
    /// calls this with its own random value -- no `sculpt_randomize_seed`
    /// exists here since picking the random number is a UI concern this
    /// binding has no reason to own). Truncates like every other
    /// GDScript-int -> engine-`u32` field in this crate (`params.rs`'s own
    /// `as usize`/`as i32` casts); a caller sending a real 32-bit value
    /// round-trips exactly. No-op before any `generate()` call.
    #[func]
    fn sculpt_set_seed(&mut self, seed: i64) {
        if let Some(s) = self.sculpt.as_mut() {
            s.seed = seed as u32;
        }
    }

    // ---- Sculpt editor: stroke capture ----
    //
    // A GDScript pointer-drag loop is: begin_stroke() once, add_point()
    // per captured sample (dense -- `sculpt_commit.rs`'s own doc comment
    // on `enforce_channel_descent` warns a coarse stroke carves at
    // coarsely-spaced sites, since neither it nor this binding resamples),
    // then end_stroke() once on release. cancel_stroke() instead of
    // end_stroke() drops the points without creating a stamp (a drag that
    // leaves the canvas, say).

    /// Starts capturing a new stroke -- clears any previously in-progress
    /// (uncommitted-to-the-draft) points. `false`, nothing cleared, before
    /// any `generate()` call.
    #[func]
    fn sculpt_begin_stroke(&mut self) -> bool {
        let Some(s) = self.sculpt.as_mut() else { return false };
        s.points.clear();
        true
    }

    /// Appends one point (grid-cell coordinates -- the reference's own
    /// `evtToGridLOD` convention, so a stroke behaves identically at any
    /// zoom/LOD) to the in-progress stroke. Returns the new point count,
    /// or `-1` before any `generate()` call. A non-finite `x`/`y` is
    /// silently dropped (the point count is unchanged) rather than
    /// poisoning every stamp built from this stroke with a NaN.
    #[func]
    fn sculpt_add_point(&mut self, x: f64, y: f64) -> i32 {
        let Some(s) = self.sculpt.as_mut() else { return -1 };
        if x.is_finite() && y.is_finite() {
            s.points.push(cartalith_terrain::sculpt::Point::new(x, y));
        }
        s.points.len() as i32
    }

    /// The in-progress stroke's point count. `0` before any `generate()`
    /// call, same as a real empty stroke -- a caller cannot tell the two
    /// apart from this alone, which is fine since neither has anything to
    /// end.
    #[func]
    fn sculpt_stroke_point_count(&self) -> i32 {
        self.sculpt.as_ref().map_or(0, |s| s.points.len() as i32)
    }

    /// Drops the in-progress stroke's points without creating a stamp.
    #[func]
    fn sculpt_cancel_stroke(&mut self) {
        if let Some(s) = self.sculpt.as_mut() {
            s.points.clear();
        }
    }

    /// Freezes the current tool state (feature, its parameters, the live
    /// globals, the seed) plus the in-progress stroke's points into one
    /// `SculptStamp` and pushes it onto the draft -- **a draft push, not a
    /// commit**: nothing touches the real heightfield here
    /// (`cartalith_spatial::PassBuffer`'s own "nothing here touches field"
    /// contract). Clears the in-progress points and selects the new stamp.
    ///
    /// Returns the new stamp's index (pass to `sculpt_select_stamp` and
    /// friends), or `-1` if the stroke had zero points (a one-point stroke
    /// is legal -- it degenerates to a tap, `SCULPT_FUNCTION_CHART.md` §2 --
    /// only zero is rejected) or before any `generate()` call.
    #[func]
    fn sculpt_end_stroke(&mut self) -> i32 {
        let sea_level = self.sea_level;
        let Some(s) = self.sculpt.as_mut() else { return -1 };
        if s.points.is_empty() {
            return -1;
        }
        let stamp = SculptStamp {
            seed: s.seed,
            points: std::mem::take(&mut s.points),
            globals: s.globals,
            params: s.params,
            sea_level,
        };
        let index = s.draft.push(stamp);
        s.selected = Some(index);
        index as i32
    }

    // ---- Sculpt editor: stamp stack (DCC_SHELL_SPEC.md §6) ----

    /// Number of stamps currently on the draft. `0` before any
    /// `generate()` call.
    #[func]
    fn sculpt_stamp_count(&self) -> i32 {
        self.sculpt.as_ref().map_or(0, |s| s.draft.len() as i32)
    }

    /// The draft's stamps **newest-first** (`DCC_SHELL_SPEC.md` §6's own
    /// "Stamp stack" context), one `Dictionary` per stamp:
    /// - `index` (int) -- the real draft index; pass this straight to
    ///   `sculpt_select_stamp`/`sculpt_set_stamp_hidden`/
    ///   `sculpt_move_stamp_up`/`sculpt_move_stamp_down`/
    ///   `sculpt_delete_stamp`. **Not** the reversed position in this
    ///   list -- reordering the *display* must not renumber the stack.
    /// - `hidden` (bool), `feature` (String key), `label` (String),
    /// - `point_count` (int),
    /// - `params` (`Dictionary`, this stamp's own frozen parameter
    ///   values -- the "parameter summary" §6 asks for, same shape
    ///   `sculpt_get_feature_params` returns),
    /// - `globals` (`Dictionary`, this stamp's own frozen brush/noise
    ///   globals -- distinct from the *live* tool state
    ///   `sculpt_get_globals` reports, since each stamp captured its own
    ///   copy at the moment its stroke ended).
    ///
    /// Empty before any `generate()` call or while the draft is empty.
    #[func]
    fn sculpt_list_stamps(&self) -> Array<VarDictionary> {
        let Some(s) = self.sculpt.as_ref() else { return Array::new() };
        s.draft
            .entries()
            .iter()
            .enumerate()
            .rev()
            .map(|(i, e)| {
                vdict! {
                    "index" => i as i32,
                    "hidden" => e.hidden,
                    "feature" => e.stamp.feature().meta().key,
                    "label" => e.stamp.feature().meta().label,
                    "point_count" => e.stamp.points.len() as i32,
                    "params" => &feature_params_dict(&e.stamp.params),
                    "globals" => &globals_dict(&e.stamp.globals),
                }
            })
            .collect()
    }

    /// The currently selected stamp's draft index, or `-1` for none.
    #[func]
    fn sculpt_get_selected_stamp(&self) -> i32 {
        self.sculpt.as_ref().and_then(|s| s.selected).map_or(-1, |i| i as i32)
    }

    /// Selects a stamp by draft index (`sculpt_list_stamps`' own `index`),
    /// so a shell can re-populate the parameter block for re-tuning
    /// (`SCULPT_FUNCTION_CHART.md` §5's "Re-tune" row) -- note re-tuning
    /// itself is not implemented by this call: it only records the
    /// selection, matching `PassBuffer`'s own stack model, which has no
    /// "edit stamp N in place" operation (delete and re-paint is the real
    /// affordance today; see this crate's own report for what a live
    /// re-tune would need). Pass a negative index to deselect. `false` for
    /// an out-of-range non-negative index or before any `generate()` call.
    #[func]
    fn sculpt_select_stamp(&mut self, index: i32) -> bool {
        let Some(s) = self.sculpt.as_mut() else { return false };
        if index < 0 {
            s.selected = None;
            return true;
        }
        let Ok(i) = usize::try_from(index) else { return false };
        if i >= s.draft.len() {
            return false;
        }
        s.selected = Some(i);
        true
    }

    /// Hides or shows a stamp (`DCC_SHELL_SPEC.md` §6's "Hide/show"). A
    /// hidden stamp is skipped by both preview and commit, but still
    /// occupies its draft index and still counts in
    /// `build_sculpt_preview_texture`'s footprint bookkeeping
    /// (`PassBuffer::set_hidden`'s own doc comment). `false` for an
    /// out-of-range index or before any `generate()` call.
    #[func]
    fn sculpt_set_stamp_hidden(&mut self, index: i32, hidden: bool) -> bool {
        let Some(s) = self.sculpt.as_mut() else { return false };
        let Ok(i) = usize::try_from(index) else { return false };
        if i >= s.draft.len() {
            return false;
        }
        s.draft.set_hidden(i, hidden);
        true
    }

    /// Moves a stamp one place earlier in the stack -- stack order is bake
    /// order, so this changes the result for order-dependent stamps
    /// (`PassBuffer`'s own test, "reordering really changes the result").
    /// `false` (no-op) at index 0, an out-of-range index, or before any
    /// `generate()` call.
    #[func]
    fn sculpt_move_stamp_up(&mut self, index: i32) -> bool {
        let Some(s) = self.sculpt.as_mut() else { return false };
        let Ok(i) = usize::try_from(index) else { return false };
        s.draft.move_up(i)
    }

    /// As `sculpt_move_stamp_up`, one place later. `false` (no-op) at the
    /// top of the stack, an out-of-range index, or before any `generate()`
    /// call.
    #[func]
    fn sculpt_move_stamp_down(&mut self, index: i32) -> bool {
        let Some(s) = self.sculpt.as_mut() else { return false };
        let Ok(i) = usize::try_from(index) else { return false };
        s.draft.move_down(i)
    }

    /// Removes a stamp from the draft. Clears the selection if it pointed
    /// at the removed stamp (an index into a now-shorter stack would
    /// otherwise silently refer to a different stamp). `false` for an
    /// out-of-range index or before any `generate()` call.
    #[func]
    fn sculpt_delete_stamp(&mut self, index: i32) -> bool {
        let Some(s) = self.sculpt.as_mut() else { return false };
        let Ok(i) = usize::try_from(index) else { return false };
        if i >= s.draft.len() {
            return false;
        }
        s.draft.remove(i);
        if s.selected == Some(i) {
            s.selected = None;
        }
        true
    }

    /// Whether `sculpt_undo` would do anything.
    #[func]
    fn sculpt_can_undo(&self) -> bool {
        self.sculpt.as_ref().is_some_and(|s| s.draft.can_undo())
    }

    /// Whether `sculpt_redo` would do anything.
    #[func]
    fn sculpt_can_redo(&self) -> bool {
        self.sculpt.as_ref().is_some_and(|s| s.draft.can_redo())
    }

    /// Reverts the last structural edit (add/delete/hide/reorder) to the
    /// draft -- **draft-scoped only**, this never touches the real
    /// heightfield, because nothing in the draft ever did
    /// (`PassBuffer::undo`'s own doc comment). Tier one of the two-tier
    /// undo model `SCULPT_FUNCTION_CHART.md` §6 describes; tier two (one
    /// snapshot at Commit) is the shell's own global undo stack, outside
    /// this binding's scope. `false` at the bottom of draft history or
    /// before any `generate()` call.
    #[func]
    fn sculpt_undo(&mut self) -> bool {
        self.sculpt.as_mut().is_some_and(|s| s.draft.undo())
    }

    /// As `sculpt_undo`, forward. `false` with nothing to redo, or a new
    /// edit already cleared the redo branch (`PassBuffer::push`'s own
    /// contract), or before any `generate()` call.
    #[func]
    fn sculpt_redo(&mut self) -> bool {
        self.sculpt.as_mut().is_some_and(|s| s.draft.redo())
    }

    // ---- Sculpt editor: preview, commit, discard ----

    /// A **real, live** colour + hillshade texture for the current Sculpt
    /// draft, alongside the committed world's own `build_color_texture` --
    /// the owner's own resolution of the question `SCULPT_FUNCTION_CHART.md`
    /// §9 left open: *"the fix in this version would be to have all these
    /// manipulations live, as we have the computational power available
    /// directly."* The reference's `sculptRenderOverlay` (a translucent
    /// outline/hatch, its own comment calling it *"a deliberately simpler
    /// indicator than a full live-recolor"*) is a JavaScript-cost
    /// compromise this port is not reproducing: this method returns the
    /// actual drafted colour result, not an outline standing in for one.
    ///
    /// Composites the whole stamp stack over a **scratch copy** of the real
    /// height field (`PassBuffer::preview_into` -- the reference's own
    /// "nothing here touches field" contract, enforced by the borrow
    /// checker since the base field is passed as `&[_]`) and renders it
    /// through the same per-pixel colour pass (`render::cell_color`)
    /// `build_color_texture` uses, so a preview looks like what committing
    /// would actually produce.
    ///
    /// Deliberately lighter than `build_color_texture`: no channel tint,
    /// no local-contrast pass, no lithology, no icon compositing. All four
    /// either key off state a draft cannot have moved (`ws.channels`'
    /// Strahler mask and the lithology classification are both keyed to
    /// the *committed* field; a loaded asset pack's icon placement is
    /// independent of height entirely) or cost a whole-field pass
    /// (`build_lithology`) that this method would otherwise re-run on
    /// every brush stroke a caller previews, not just on commit -- the
    /// same ~7s/2048² measurement `UNIFIED_TOOL_PLAN.md` milestone C cites
    /// for why commit itself stays deferred applies here to *any* eager
    /// per-stroke whole-field work.
    ///
    /// **Renders the whole grid, not just the draft's touched region.**
    /// `PassBuffer::touched_bounds` would give the rectangle a bounded
    /// variant could restrict its own per-pixel colour loop to, but
    /// `RenderCtx::with_appearance` itself precomputes several derived
    /// rasters over the **entire** grid unconditionally on construction
    /// (`smooth_sea_h`, `build_ao`, `build_hydro_wetness`) -- restricting
    /// only the final per-pixel loop to the touched rectangle would shrink
    /// the returned image without touching the dominant cost, which would
    /// be a cosmetic optimisation reported as a real one. A genuinely
    /// bounded preview needs those three passes reworked to run over a
    /// caller-supplied window instead of the full field -- real surgery on
    /// `render.rs`, a file `golden_parity_render.rs` already pins bit-for-
    /// bit, not a small addition alongside this binding. Left for the
    /// separate live-preview scope document rather than half-done here.
    ///
    /// `None` before any `generate()` call, for a loaded save (no draft
    /// exists there at all -- see the `sculpt` field's own doc comment),
    /// or while the draft is empty (nothing would differ from
    /// `build_color_texture`).
    #[func]
    fn build_sculpt_preview_texture(&self) -> Option<Gd<ImageTexture>> {
        let s = self.sculpt.as_ref()?;
        if s.draft.is_empty() {
            return None;
        }
        let WorldSource::Generated(ws) = self.source.as_ref()? else { return None };
        let gw = self.gw as usize;
        let gh = self.gh as usize;
        let mut scratch = ws.field.clone();
        s.draft.preview_into(&ws.field, &mut scratch);

        let appearance = self.appearance();
        let ctx = RenderCtx::with_appearance(
            &scratch,
            &ws.temperature,
            &ws.rainfall,
            Some(ws.flow_discharge.as_slice()),
            gw,
            gh,
            self.sea_level,
            self.world,
            self.lat_n,
            self.lat_s,
            appearance,
        );
        let mut bytes = vec![0u8; gw * gh * 3];
        bytes.par_chunks_mut(gw * 3).enumerate().for_each(|(y, row)| {
            for x in 0..gw {
                let (r, g, b) = render::cell_color(&ctx, x, y);
                let o = x * 3;
                row[o] = (r.clamp(0.0, 1.0) * 255.0) as u8;
                row[o + 1] = (g.clamp(0.0, 1.0) * 255.0) as u8;
                row[o + 2] = (b.clamp(0.0, 1.0) * 255.0) as u8;
            }
        });
        let packed = PackedByteArray::from(bytes);
        let image = Image::create_from_data(gw as i32, gh as i32, false, Format::RGB8, &packed)?;
        ImageTexture::create_from_image(&image)
    }

    /// Bakes the whole draft into the real heightfield in one pass and
    /// empties the draft (`cartalith_engine::sculpt_commit::
    /// commit_sculpt_pass`, unchanged -- see that module's own doc for the
    /// exact five-step ordering: bake, re-clamp locked river channels,
    /// carve+lock this batch's own river stamps, deposit lakes against the
    /// final baked height, and nothing else).
    ///
    /// **Then re-runs hydrology and climate, and nothing else.** Every tile
    /// the commit touched is marked against `PipelineStage::Height` in the
    /// live staleness graph, and `cartalith_engine::staleness::
    /// recompute_stale` re-runs exactly the stages that invalidated: one
    /// `refresh_climate` producing new discharge, temperature and rainfall.
    /// That is the reference's own post-commit tail (`computeFlow(true);
    /// refreshClimate();`), not the eager cascade `DCC_SHELL_SPEC.md` §5.2's
    /// prose describes — erosion does **not** re-run (it is part of the
    /// height stage, which the user just edited by hand), and civ does not
    /// either, since the eager form was measured at ~7 s/stroke at 2048² and
    /// rejected in `UNIFIED_TOOL_PLAN.md` milestone C. `still_stale` below
    /// reports what was left, for a caller to act on when it chooses.
    ///
    /// `reason` is a short caller-chosen string for the dirty-tile record
    /// (`"sculpt"` is fine). Returns a summary `Dictionary`:
    /// `stamps_applied`/`stamps_skipped` (int), `tiles_marked`
    /// (`PackedInt32Array`), `rivers_carved`/`cells_locked` (int, the
    /// River hook), `lakes_deposited`/`lake_cells` (int, the Lake hook),
    /// and `recomputed`/`still_stale` (`PackedStringArray`, the stage names
    /// that re-ran and the ones still waiting) -- empty `Dictionary` before
    /// any `generate()` call.
    ///
    /// Call `build_color_texture()` again afterward to see the result --
    /// the same "no regeneration needed" contract `set_quality_tier`'s own
    /// doc comment establishes for a purely presentational change; this one
    /// really did change the height field, but the render path reads it
    /// fresh on every call regardless.
    #[func]
    fn sculpt_commit(&mut self, reason: GString) -> VarDictionary {
        // The reference forces the sculpt editor read-only while finalized
        // (`applyFinalizedUI`'s `_sculptNavSync` call, line 10869): the baked
        // atlas is the authoritative surface, and an edit under it would show
        // in the live view and vanish at the next zoom.
        if let Err(msg) = self.bake.check(cartalith_engine::bake::Mutation::HeightEdit) {
            godot_print!("cartalith-godot: sculpt_commit refused -- {msg}");
            return VarDictionary::new();
        }
        let sea_level = self.sea_level;
        let reason = reason.to_string();
        let (Some(sculpt), Some(WorldSource::Generated(ws))) = (self.sculpt.as_mut(), self.source.as_mut()) else {
            return VarDictionary::new();
        };
        // Global heightmap undo, the reference's own call site (`sculptCommit`
        // opens with `pushUndo()`, reference HTML line 9319). Pushed here
        // rather than at the button, so a commit that turns out to apply
        // nothing still costs a snapshot exactly as it does in the reference
        // — matching its step accounting is worth more than saving 16 MB on
        // an empty commit the UI already disables.
        self.undo.push("Sculpt commit", &ws.field);
        self.ledger.record(
            "height",
            "Sculpt commit",
            format!("{} x {}", self.gw, self.gh),
            undo::EntryKind::HeightSnapshot,
        );
        let summary = sculpt.commit(&mut ws.field, sea_level, &reason);
        // Keep WorldState's own optional river fields in sync with what the
        // Sculpt layer has now locked. `WaterState` (`sculpt.water`) is the
        // real source of truth for river locks from this point on -- both
        // arrays already exist and are already computed by the commit
        // above, so writing them back is a relocation, not a recompute.
        // Without this, a save taken after this call (`cartalith-io`,
        // `SAVEFILE_COMPAT.md`) would silently drop a hand-painted river's
        // lock, since it reads `ws.river_mask`/`river_floor` directly.
        ws.river_mask = Some(sculpt.water.river_mask.clone());
        ws.river_floor = Some(sculpt.water.river_floor.clone());

        let tiles_marked: PackedInt32Array = summary.pass.tiles_marked.iter().map(|&t| t as i32).collect();
        let tiles = summary.pass.tiles_marked.clone();
        let (applied, skipped) = (summary.pass.stamps_applied as i64, summary.pass.stamps_skipped as i64);
        let (rivers, locked) = (summary.rivers_carved as i64, summary.cells_locked as i64);
        let (lakes, lake_cells) = (summary.lakes_deposited as i64, summary.lake_cells as i64);
        // `ws`/`sculpt` are borrowed from `self` above; every value the
        // summary contributes is copied out first so those borrows end here
        // and the recompute can take `self` whole.
        let (recomputed, still_stale) = self.mark_and_recompute(PipelineStage::Height, tiles, &reason);
        dict! {
            "stamps_applied" => applied,
            "stamps_skipped" => skipped,
            "tiles_marked" => &tiles_marked,
            "rivers_carved" => rivers,
            "cells_locked" => locked,
            "lakes_deposited" => lakes,
            "lake_cells" => lake_cells,
            "recomputed" => &recomputed,
            "still_stale" => &still_stale,
        }
    }

    /// Drops the whole draft, touching nothing else (`PassBuffer::
    /// discard`'s own doc comment: nothing was ever written to the field,
    /// so there is nothing to undo there either). Returns how many stamps
    /// were dropped, `0` before any `generate()` call.
    #[func]
    fn sculpt_discard(&mut self) -> i32 {
        self.sculpt.as_mut().map_or(0, |s| s.draft.discard() as i32)
    }
}

/// `UNIFIED_TOOL_PLAN.md` milestone F, the CARTO domain's Icon tool
/// (`DCC_SHELL_SPEC.md` §4.5.5). Thin `Variant`<->Rust conversion over
/// `icon_bridge::IconEditor` -- see that module's own doc comment for the
/// state machine, the `family`/`variant` numeric mapping, and which of
/// `icon_arm`'s five parameters actually reach a placed icon.
///
/// `secondary`: gdext allows exactly one *primary* `#[godot_api] impl
/// WorldGen` block per class (it alone generates the class's
/// `__registration_storage`/`ImplementsGodotApi` machinery,
/// `godot-macros`' own `inherent_impl.rs` doc comment on
/// `InherentImplAttr::secondary`) — the existing block above (`generate`,
/// the Sculpt milestone F methods, etc.) already is that one. Every
/// further `#[func]`-bearing block, this one included, must be
/// `#[godot_api(secondary)]` instead of a second plain `#[godot_api]`, or
/// the two collide on that shared machinery at compile time.
#[godot_api(secondary)]
impl WorldGen {
    /// Arms `family` (one of `"settlement"`, `"feature"`, `"poi"` --
    /// `"custom"` is rejected, see `icon_bridge::resolve_variant`'s own doc
    /// comment) / `variant` (a zero-based index into that family's frozen
    /// slot vocabulary) for the next `icon_place` call. Requires a real
    /// asset pack to already be loaded (`has_asset_pack`) -- arming a
    /// family/slot this port cannot yet draw would let a caller stamp
    /// icons with nothing to render, silently. `false` and nothing armed
    /// without a loaded pack, for an unrecognised `family`, or for a
    /// `variant` outside that family's own vocabulary.
    #[func]
    fn icon_arm(&mut self, family: GString, variant: i64, scale: f64, rotation: f64, jitter: f64) -> bool {
        if !self.has_asset_pack() {
            return false;
        }
        let Some(icons) = self.icons.as_mut() else { return false };
        icons.arm(&family.to_string(), variant, scale, rotation, jitter)
    }

    /// The armed selection's chip contents (`DCC_SHELL_SPEC.md` §4.5.5:
    /// "the armed icon is shown as a chip") -- `family`, `slot`, `scale`,
    /// `rotation`, `jitter` (`set` omitted: arming can never reach
    /// `Custom`, see `icon_arm`). Empty `Dictionary` when nothing is
    /// armed or before any `generate()` call.
    #[func]
    fn icon_armed(&self) -> VarDictionary {
        let Some(a) = self.icons.as_ref().and_then(|i| i.armed.as_ref()) else { return VarDictionary::new() };
        vdict! {
            "family" => a.icon.family.key(),
            "slot" => a.icon.slot.as_str(),
            "scale" => a.scale,
            "rotation" => a.rotation,
            "jitter" => a.jitter,
        }
    }

    /// Disarms -- the next `icon_place` call does nothing until `icon_arm`
    /// is called again. A no-op before any `generate()` call.
    #[func]
    fn icon_disarm(&mut self) {
        if let Some(icons) = self.icons.as_mut() {
            icons.disarm();
        }
    }

    /// Stamps the armed icon at grid cell `(gx, gy)` and selects it
    /// (`place_manual_icon` plus the arm-time scale override --
    /// `icon_bridge`'s own doc comment). Returns the new icon's index, or
    /// `-1` when nothing is armed, the click is off-grid, or before any
    /// `generate()` call.
    #[func]
    fn icon_place(&mut self, gx: f64, gy: f64) -> i64 {
        let (gw, gh) = (self.gw as usize, self.gh as usize);
        let Some(icons) = self.icons.as_mut() else { return -1 };
        icons.place(gx, gy, gw, gh).map_or(-1, |i| i as i64)
    }

    /// The currently selected placed icon's index, or `-1` for none (or
    /// before any `generate()` call) -- `label_get_selected`'s own icon
    /// counterpart. `IconEditor`'s own `selected` field is otherwise
    /// unreadable from this side: `icon_resize` already requires `index`
    /// to match it, and `icon_handles`'s caller needs to know which index
    /// to ask for and when to draw a handle at all.
    #[func]
    fn icon_get_selected(&self) -> i64 {
        self.icons.as_ref().and_then(|i| i.selected).map_or(-1, |i| i as i64)
    }

    /// Grid-space hit test against every placed icon's box
    /// (`icon_bridge::IconEditor::hit_test` -- box hits only; a *handle*
    /// hit is the shell's own job, by comparing the pointer against
    /// `icon_handles`' own circle for whichever icon is selected, exactly
    /// `label_hit_test`'s own precedent one step further -- see that
    /// method's own doc comment). `(gx, gy)` are grid coordinates, matching
    /// `icon_place`'s own convention, not screen pixels. Selects and
    /// returns the hit icon's index on a hit; `-1` on a miss or before any
    /// `generate()` call.
    #[func]
    fn icon_hit_test(&mut self, gx: f64, gy: f64) -> i64 {
        let grid_w = self.gw as usize;
        let Some(icons) = self.icons.as_mut() else { return -1 };
        let env = cartalith_assets::manual::IconViewEnv { grid_w, zoom_scale: 1.0, icon_scale: 1.0 };
        icons.hit_test(gx, gy, &env).map_or(-1, |i| i as i64)
    }

    /// Icon `index`'s on-canvas resize-handle circle
    /// (`icon_bridge::IconEditor::handles`) -- `GUI_GAP_REGISTER.md` CA-05:
    /// the equivalent of `label_handles`, one handle instead of three since
    /// a manually-placed icon has no rotate/arc field to hand a handle to
    /// (`icon_bridge.rs`'s own doc comment). Returns `{"resize": {"x":..,
    /// "y":.., "r":..}}`, the same `"resize"` key and `{"x","y","r"}` shape
    /// `label_handles` already uses, so a caller (`tool_overlay.gd`'s
    /// `set_handles`) can consume either with no reshaping. `zoom` is the
    /// raw view scale, matching `label_handles`' own parameter (`civ_zoom_k`
    /// applies its own clamp internally). Empty top-level `Dictionary` for
    /// an out-of-range `index` or before any `generate()` call.
    #[func]
    fn icon_handles(&self, index: i64, zoom: f64) -> VarDictionary {
        let Ok(i) = usize::try_from(index) else { return VarDictionary::new() };
        let Some(icons) = self.icons.as_ref() else { return VarDictionary::new() };
        let env = cartalith_assets::manual::IconViewEnv { grid_w: self.gw as usize, zoom_scale: zoom, icon_scale: 1.0 };
        let Some(h) = icons.handles(i, &env) else { return VarDictionary::new() };
        vdict! { "resize" => &vdict! { "x" => h.x, "y" => h.y, "r" => h.r } }
    }

    /// Applies one resize-drag sample to the selected icon's scale
    /// (`icon_resize_scale`, via `icon_bridge::IconEditor::resize` --
    /// `index` must already be selected, normally by a prior
    /// `icon_hit_test` hit on its box; that method's own doc comment
    /// explains why `start_dist` alone, without a separate `start_scale`
    /// parameter, is enough for a whole drag). `false` if `index` is not
    /// currently selected, out of range, or before any `generate()` call.
    #[func]
    fn icon_resize(&mut self, index: i64, cx: f64, cy: f64, gx: f64, gy: f64, start_dist: f64) -> bool {
        let Ok(i) = usize::try_from(index) else { return false };
        let Some(icons) = self.icons.as_mut() else { return false };
        icons.resize(i, cx, cy, gx, gy, start_dist)
    }

    /// One placed icon's properties (`DCC_SHELL_SPEC.md` §4.5.5's right
    /// dock): `x`, `y`, `family`, `slot`, `set` (empty string outside
    /// `Custom`), `scale`. Empty `Dictionary` for an out-of-range `index`
    /// or before any `generate()` call.
    #[func]
    fn icon_get(&self, index: i64) -> VarDictionary {
        let Ok(i) = usize::try_from(index) else { return VarDictionary::new() };
        let Some(ic) = self.icons.as_ref().and_then(|s| s.icons.get(i)) else { return VarDictionary::new() };
        icon_dict(ic)
    }

    /// Removes a placed icon (`DCC_SHELL_SPEC.md` §4.5.6: "Delete removes
    /// the current selection... an icon"). Clears the selection if it
    /// pointed at the removed icon; shifts a selection past it down by one
    /// (`icon_bridge::IconEditor::delete`'s own doc comment). `false` for
    /// an out-of-range `index` or before any `generate()` call.
    #[func]
    fn icon_delete(&mut self, index: i64) -> bool {
        let Ok(i) = usize::try_from(index) else { return false };
        let Some(icons) = self.icons.as_mut() else { return false };
        icons.delete(i)
    }

    /// Every placed icon, in placement order (`DCC_SHELL_SPEC.md` §4.5.5's
    /// `#carIconList`), each the same shape `icon_get` returns plus its
    /// own `index`. Empty before any `generate()` call or while nothing is
    /// placed.
    #[func]
    fn icon_list(&self) -> Array<VarDictionary> {
        let Some(icons) = self.icons.as_ref() else { return Array::new() };
        icons
            .icons
            .iter()
            .enumerate()
            .map(|(i, ic)| {
                let mut d = icon_dict(ic);
                d.set("index", i as i64);
                d
            })
            .collect()
    }

    /// Drops every placed icon (armed selection untouched --
    /// `icon_bridge::IconEditor::clear_all`'s own doc comment). A no-op
    /// before any `generate()` call.
    #[func]
    fn icon_clear_all(&mut self) {
        if let Some(icons) = self.icons.as_mut() {
            icons.clear_all();
        }
    }
}

/// One `ManualIcon`'s fields as a flat `Dictionary` -- shared by `icon_get`
/// and `icon_list`, which both need exactly this shape (`icon_list` adds
/// its own `index` on top).
fn icon_dict(ic: &cartalith_assets::manual::ManualIcon) -> VarDictionary {
    vdict! {
        "x" => ic.x,
        "y" => ic.y,
        "family" => ic.family.key(),
        "slot" => ic.slot.as_str(),
        "set" => ic.set.as_deref().unwrap_or(""),
        "scale" => ic.scale,
    }
}

/// `UNIFIED_TOOL_PLAN.md` milestone F, the CIVIL domain's Settlement and
/// Territory tools (`DCC_SHELL_SPEC.md` §4.5.3). Thin `Variant`<->Rust
/// conversion over `civ_tools_bridge` -- see that module's own doc comment
/// for the territory-paint draft/base/accumulator split, the manual-
/// placement name/population RNG, and the two honest gaps this binds
/// around rather than papers over: **POI has no engine counterpart at all**
/// (`cartalith-civ` never ported `_civDropPOI`), and **"metropolis" is not
/// a real `SettlementKind`** (the port only has the five tiers
/// `place_settlements` actually produces). Neither is invented here.
///
/// `#[godot_api(secondary)]`, not a plain `#[godot_api]`: only the first
/// `#[func]`-bearing block for a class with a `Base<T>` field (`WorldGen`
/// has one) may be the plain form -- every further one collides with it on
/// the `WithSignals`/`WithUserSignals`/registration-storage machinery that
/// attribute generates unconditionally (`godot-macros`' own `signal.rs`:
/// "we always generate a collection struct ... if ... the struct has a
/// Base<T> field"). The existing block above (`generate`, the Sculpt
/// milestone F methods, etc.) already is that one plain block; the
/// `icon_bridge` block below it found this same requirement first.
#[godot_api(secondary)]
impl WorldGen {
    /// Hit-test an existing settlement near `(gx, gy)` -- `cartalith_civ::
    /// civ_pick_place_at`'s weighted-nearest pick (`tools.rs`: a bigger
    /// settlement outcompetes a closer small one, `v1.88`), at
    /// `civ_place_pick_radius`'s base radius. Returns its index, or `-1`
    /// for no hit, no generated world, or an empty settlement list.
    ///
    /// No zoom scaling: `civ_zoom_pick_r` exists in `cartalith-civ` for a
    /// caller that wants the pick radius to read as a constant *screen*
    /// size regardless of view zoom, but this method's own signature
    /// (matching `civ_drop_settlement`'s) has no zoom parameter to feed it
    /// -- a future overload can thread one through once the shell has a
    /// view-zoom value to pass. Until then this is the same radius at
    /// every zoom level, which is exactly what `civ_place_pick_radius`
    /// alone already gives.
    #[func]
    fn civ_pick_place_at(&self, gx: f64, gy: f64) -> i64 {
        let Some(civ) = self.civ.as_ref() else { return -1 };
        let pick_r = cartalith_civ::tools::civ_place_pick_radius(self.gw as usize);
        match cartalith_civ::tools::civ_pick_place_at(&civ.settlements, gx, gy, pick_r) {
            Some(i) => i as i64,
            None => -1,
        }
    }

    /// `_civDropPlace` (`cartalith_civ::civ_drop_place`) -- the Settlement
    /// tool's click handler. `kind` is one of the five real tiers
    /// (`"capital"/"city"/"town"/"village"/"hamlet"`, case-insensitive;
    /// `"metropolis"` and anything else unrecognised is rejected, see this
    /// impl block's own header comment). `name` blank (or all whitespace)
    /// gets an engine-generated one from `civ_settle_name`, matching
    /// `DCC_SHELL_SPEC.md` §4.5.3's "name (blank = generated)"; population
    /// always follows the same tier-populated curve auto-populated
    /// settlements use (`civ_tools_bridge::manual_settlement_pop`).
    ///
    /// Returns the new (or, if the click landed on an existing place, the
    /// already-there) settlement's index into `get_settlements()`'s own
    /// array -- so a shell can immediately re-read it for §4.5.3's "the new
    /// settlement's inspector, live, focused on the name field." `-1` for
    /// an out-of-bounds click, a water refusal, an unrecognised `kind`, a
    /// non-finite/negative coordinate, or before any `generate()` call.
    ///
    /// `snap_to_water`: **a new affordance, not a ported one** -- see
    /// `civ_tools_bridge::nearest_land_cell`'s own doc comment. When on and
    /// the raw click cell is water, this searches outward (within
    /// `civ_snap_radius`'s own base radius) for the nearest dry-land,
    /// non-water-body cell and places there instead; a click with nothing
    /// dry in range still refuses exactly as `civ_drop_place` always has.
    #[func]
    fn civ_drop_settlement(&mut self, gx: f64, gy: f64, kind: GString, faction: i64, name: GString, snap_to_water: bool) -> i64 {
        let Some(k) = civ_tools_bridge::kind_from_str(&kind.to_string()) else { return -1 };
        if !gx.is_finite() || !gy.is_finite() || gx < 0.0 || gy < 0.0 {
            return -1;
        }
        let gw = self.gw as usize;
        let gh = self.gh as usize;
        let sea = self.sea_level;
        let (Some(civ), Some(WorldSource::Generated(ws)), Some(tools)) = (self.civ.as_mut(), self.source.as_mut(), self.civ_tools.as_mut()) else {
            return -1;
        };
        let mut cx = gx.round() as usize;
        let mut cy = gy.round() as usize;
        if cx >= gw || cy >= gh {
            return -1;
        }
        if snap_to_water {
            let max_r = cartalith_civ::tools::civ_snap_radius(gw);
            if let Some((nx, ny)) = civ_tools_bridge::nearest_land_cell(cx, cy, gw, gh, &ws.field, &civ.water_bodies, sea, max_r) {
                cx = nx;
                cy = ny;
            }
        }
        let pick_r = cartalith_civ::tools::civ_place_pick_radius(gw);
        let name = name.to_string();
        match civ_tools_bridge::drop_settlement(
            &mut civ.settlements,
            &mut civ.next_tid,
            &mut tools.name_rng,
            cx,
            cy,
            pick_r,
            &ws.field,
            &civ.water_bodies,
            gw,
            gh,
            sea,
            faction as i32,
            k,
            &name,
        ) {
            Some(i) => {
                // SG-01: roads, territory, provinces and trade balances were
                // all derived before this settlement existed.
                self.civ_dirty = true;
                i as i64
            }
            None => -1,
        }
    }

    /// One dab of the Territory tool's brush (`merge_territory_paint`, via
    /// `civ_tools_bridge::CivTools::paint_at`) into the in-progress,
    /// uncommitted draft -- a no-op before any `generate()` call. `subtract`
    /// (⇧, `DCC_SHELL_SPEC.md` §4.5.3) erases rather than claims, so the
    /// affected cells fall through to `assign_territory`'s own computed
    /// answer on the next commit rather than to bare "unclaimed" -- see
    /// `civ_tools_bridge.rs`'s own module doc for why that needs
    /// `territory_base` at all.
    #[func]
    fn civ_territory_paint_at(&mut self, gx: f64, gy: f64, faction: i64, radius: f64, subtract: bool) {
        if let Some(tools) = self.civ_tools.as_mut() {
            tools.paint_at(gx, gy, faction as i32, radius, subtract);
        }
    }

    /// Bakes the in-progress territory draft into the accumulated paint
    /// layer and rebuilds `get_provinces`/`build_territory_texture`'s own
    /// `territory` from `territory_base` merged with the full accumulated
    /// layer -- a no-op with nothing pending, or before any `generate()`
    /// call.
    #[func]
    fn civ_territory_commit(&mut self) {
        let (Some(civ), Some(tools)) = (self.civ.as_mut(), self.civ_tools.as_mut()) else { return };
        let committed = tools.commit(&mut civ.territory);
        if !committed {
            return;
        }
        // ED-02, recorded and not reversible. `civ_tools`' own Discard
        // reverts an *uncommitted* draft; once a claim is baked into
        // `CivData::territory` the pre-commit grid is gone, and holding a
        // copy of it would be a second, unbudgeted undo stack beside the
        // height one.
        self.ledger.record(
            "civ",
            "Territory commit",
            format!("{} cells claimed", civ.territory.iter().filter(|&&t| t > 0).count()),
            undo::EntryKind::Recorded(
                "the pre-commit claim grid is not retained; the Territory tool's own Discard reverts an uncommitted draft only",
            ),
        );
    }

    /// Drops the in-progress territory draft, touching nothing already
    /// committed -- a no-op before any `generate()` call.
    #[func]
    fn civ_territory_discard(&mut self) {
        if let Some(tools) = self.civ_tools.as_mut() {
            tools.discard();
        }
    }

    /// `DCC_SHELL_SPEC.md` §4.5.3's Territory right dock: live `area_km2`
    /// (claimed-cell count times this world's own square cell area,
    /// `(map_width_km/gw)^2`), `claimed_cells` (count of `territory[i] ==
    /// faction`), and `contested_cells` -- **this bridge's own heuristic**,
    /// not a reference or engine concept (`civ_tools_bridge::
    /// contested_cell_count`'s own doc comment: a claimed cell bordering a
    /// *different* claimed faction, 4-connected). Empty `Dictionary` before
    /// any `generate()` call.
    #[func]
    fn civ_faction_territory_stats(&self, faction: i64) -> VarDictionary {
        let Some(civ) = self.civ.as_ref() else { return VarDictionary::new() };
        let gw = self.gw as usize;
        let gh = self.gh as usize;
        let f = faction as i32;
        let claimed = civ.territory.iter().filter(|&&t| t == f).count();
        let contested = civ_tools_bridge::contested_cell_count(&civ.territory, f, gw, gh);
        let cell_km = if gw > 0 { self.map_width_km / gw as f64 } else { 0.0 };
        let area_km2 = claimed as f64 * cell_km * cell_km;
        dict! {
            "faction" => faction,
            "claimed_cells" => claimed as i64,
            "contested_cells" => contested as i64,
            "area_km2" => area_km2,
        }
    }

    /// Every faction the placement/territory tools can target -- `id`
    /// (`1..=civ_faction_count()`, matching `get_settlements()`/
    /// `get_provinces()`'s own `faction` field), `name`, `culture`,
    /// `religion`, `government`, `ag_tech`, `color_r`/`color_g`/`color_b`
    /// (0-255, the exact swatch `build_territory_texture` paints this
    /// faction's cells in -- `§4.5.3`'s "faction swatch"),
    /// `settlement_count`, `population` and `claimed_cells`. Empty `Array`
    /// before any `generate()` call -- there is no `get_factions`-shaped
    /// method anywhere else in this crate to reuse (`get_provinces()`
    /// reports a `faction` id per province, not an enumerable faction
    /// list).
    ///
    /// **The five editable fields are real state now** (`GUI_GAP_REGISTER
    /// .md` CV-07/MS-13). `culture` used to be `civ_default_culture(f)`
    /// recomputed on every read, with this doc comment saying outright that
    /// "the reference has no faction *name* registry beyond this". It does
    /// -- `civFactionNames`/`civFactionCulture`/`civFactionReligion`/
    /// `civFactionGovernment`/`civFactionAgTech`, five parallel arrays --
    /// and this port now has its equivalent (`CivData::faction_roster`),
    /// seeded from exactly the same defaults so an unedited world reads
    /// identically to before. `civ_set_faction_field` is what moves them.
    ///
    /// `population` is the summed `pop` of this faction's own settlements
    /// -- a plain sum over data already here, not
    /// `_civFactionAggregates`' territory-integrated `foodProductionCapacity`
    /// (that needs the density and resource rasters `compute_civilisation`
    /// frees; still open, see `ECONOMY_SCOPE.md`).
    #[func]
    fn get_factions(&self) -> Array<VarDictionary> {
        let Some(civ) = self.civ.as_ref() else { return Array::new() };
        (1..civ.faction_roster.0.len() as i32)
            .map(|f| {
                let settlement_count = civ.settlements.iter().filter(|s| s.placement.faction == f).count();
                let population: u64 = civ
                    .settlements
                    .iter()
                    .filter(|s| s.placement.faction == f)
                    .map(|s| u64::from(s.pop))
                    .sum();
                let claimed_cells = civ.territory.iter().filter(|&&t| t == f).count();
                let e = &civ.faction_roster.0[f as usize];
                let (r, g, b) = civ.faction_rgb(f);
                dict! {
                    "id" => f,
                    "name" => e.name.as_str(),
                    "culture" => e.culture.as_str(),
                    "religion" => e.religion.as_str(),
                    "government" => e.government.as_str(),
                    "ag_tech" => e.ag_tech.as_str(),
                    "color_r" => r as i64,
                    "color_g" => g as i64,
                    "color_b" => b as i64,
                    // Whether `color_*` above is the user's own identity
                    // colour rather than the palette rule's
                    // (`GUI_GAP_REGISTER.md` CV-21) -- what a *Reset* row
                    // enables on.
                    "color_custom" => e.color_override.is_some(),
                    "settlement_count" => settlement_count as i64,
                    "population" => population as i64,
                    "claimed_cells" => claimed_cells as i64,
                }
            })
            .collect()
    }
}

/// The **default** swatch for faction `f` (`1`-based): [`FACTION_RGB`]'s
/// hand-picked six for the base roster,
/// `cartalith_civ::roster::civ_faction_color`'s golden-angle rotation for
/// anything `civ_add_faction` appended past them.
///
/// That split is the reference's own (line 14565: `_civFactionColor`
/// "deterministically colours any index beyond the hand-picked base palette
/// so appended factions stay visually distinct without needing a colour
/// picker"). It replaces a `% FACTION_RGB.len()` wrap that would have given
/// faction 7 faction 1's exact colour.
///
/// **Every renderer must go through [`CivData::faction_rgb`] rather than
/// this**, which is the same function with the roster's own user override
/// (`GUI_GAP_REGISTER.md` CV-21) consulted first. This one is the fallback
/// underneath it, and is called directly only where no roster is in hand.
fn faction_rgb_default(f: i32) -> (u8, u8, u8) {
    let i = (f - 1).max(0) as usize;
    match FACTION_RGB.get(i) {
        Some(&c) => c,
        None => cartalith_civ::roster::civ_faction_color(f as usize),
    }
}

/// `UNIFIED_TOOL_PLAN.md` milestone F, the WORLD domain's `PAINT · BIOME`
/// tool (`DCC_SHELL_SPEC.md` §4.5.2) — see `paint_bridge.rs`'s own module
/// doc for the state this thin layer wraps, and `WorldGen::paint`'s own
/// field doc for why it lives on `WorldGen` rather than a second
/// `GodotClass`. A separate `impl` block, not folded into the Sculpt one
/// above, since the two tools share nothing but the pattern.
///
/// `secondary`, same reason the Icon and Civil-tools blocks above are:
/// gdext allows exactly one *primary* `#[godot_api] impl WorldGen` block
/// per class (the existing block containing `generate`/the Sculpt methods
/// already is that one) — every further `#[func]`-bearing block must say
/// `#[godot_api(secondary)]` or collide with it on the shared
/// `__registration_storage` machinery at compile time.
#[godot_api(secondary)]
impl WorldGen {
    // ---- registry: no `generate()` call required ----

    /// The three paint layers (`paint_bridge::PaintTarget::ALL`), in
    /// `DCC_SHELL_SPEC.md` §4.5.2's own target-table order (Biome, Terrain,
    /// Splat) — pass one of these back to `paint_set_layer`.
    #[func]
    fn get_paint_layers(&self) -> PackedStringArray {
        paint_bridge::PaintTarget::ALL.iter().map(|t| GString::from(t.key())).collect()
    }

    /// `layer`'s own legal value range as `{index, label}` entries, 1-based
    /// (`CART_BIOMES`/`CART_TERRAINS`/`SPLAT_PAINT_SLOTS`' own order) —
    /// exactly what §4.5.2's value-swatch legend needs, without
    /// hardcoding any of those three arrays a second time. Biome is 13
    /// entries, not `CART_BIOMES`'s full 15 (`paint_bridge::PaintTarget::
    /// palette`'s own doc: water is excluded, "the brush never touches
    /// water"). Empty `Array` for an unrecognised `layer` key.
    #[func]
    fn get_paint_palette(&self, layer: GString) -> Array<VarDictionary> {
        let Some(target) = paint_bridge::PaintTarget::from_key(&layer.to_string()) else {
            return Array::new();
        };
        target
            .palette()
            .into_iter()
            .enumerate()
            .map(|(i, label)| vdict! { "index" => (i + 1) as i32, "label" => label })
            .collect()
    }

    // ---- current tool state ----

    /// Switches which layer the next `paint_stroke_at` writes to
    /// (`paint_bridge::PaintEditor::set_layer`, which also clamps the
    /// brush's stored value into the new layer's own palette range).
    /// `false` for an unrecognised key or before any `generate()` call —
    /// the active layer is left unchanged either way.
    #[func]
    fn paint_set_layer(&mut self, layer: GString) -> bool {
        let Some(target) = paint_bridge::PaintTarget::from_key(&layer.to_string()) else { return false };
        let Some(p) = self.paint.as_mut() else { return false };
        p.set_layer(target);
        true
    }

    /// The brush parameters `DCC_SHELL_SPEC.md` §4.5.2's `PAINT · BIOME`
    /// tool options row exposes: `value` (a 1-based index into the
    /// *active* layer's own `get_paint_palette`), `radius` (cells,
    /// reference default 6, clamped to `_paintRadius`'s own 1..=40),
    /// `hardness`/`softness` (0..1, stored and echoed back for the row but
    /// never consumed — `paint_bridge`'s own module doc: painting is a
    /// hard disc with no soft falloff, unlike Sculpt), `erase` (paints `0`
    /// regardless of `value`), `land_only` (gates the dab against this
    /// world's own water-body classification — **a toggle, not the
    /// reference's hard-always gate**, the flagged new affordance
    /// `PaintStamp::mask`'s own doc describes; defaults on, matching the
    /// reference's actual behaviour until a caller turns it off).
    ///
    /// Returns what was actually stored after clamping, one dictionary of
    /// the resulting values — every argument here is positional rather
    /// than a sparse key/value patch, so this reshapes `sculpt_set_globals`'
    /// "tell the caller what really happened" contract rather than reusing
    /// its `{rejected, clamped}` key-list shape verbatim. Empty
    /// `Dictionary` before any `generate()` call.
    #[func]
    fn paint_set_brush(&mut self, value: i64, radius: f64, hardness: f64, softness: f64, erase: bool, land_only: bool) -> VarDictionary {
        let Some(p) = self.paint.as_mut() else { return VarDictionary::new() };
        let b = p.set_brush(value, radius, hardness, softness, erase, land_only);
        vdict! {
            "value" => b.value as i32,
            "radius" => b.radius,
            "hardness" => b.hardness,
            "softness" => b.softness,
            "erase" => b.erase,
            "land_only" => b.land_only,
        }
    }

    // ---- drafting ----

    /// One brush dab at grid coordinates `(gx, gy)`, pushed straight onto
    /// the active layer's own draft (`paint_bridge::PaintEditor::
    /// stroke_at`). Paint is a continuous drag in the reference (`_paintAt`,
    /// called once per pointer-move sample) rather than Sculpt's
    /// captured-polyline-then-`sculpt_end_stroke` model, so there is no
    /// begin/end pair here: every call is already one complete,
    /// independently undo-able draft entry. A no-op before any
    /// `generate()` call.
    #[func]
    fn paint_stroke_at(&mut self, gx: f64, gy: f64) {
        if let Some(p) = self.paint.as_mut() {
            p.stroke_at(gx, gy);
        }
    }

    /// A real overlay raster of the *active* layer's current paint state —
    /// already-committed cells composited with this session's own pending
    /// draft (`PassBuffer::preview_into`, never mutating either), full
    /// grid, RGBA8: alpha `0` at an unpainted cell, alpha `255` at a
    /// painted one, coloured by `paint_bridge::swatch_color` — **this
    /// port's own convention, not the reference's**, see that function's
    /// own doc for why. Meant to be drawn *over* `build_color_texture()`'s
    /// own output, the same "translucent wash over the finished raster"
    /// shape `build_territory_texture` already uses for its own overlay.
    ///
    /// Full grid rather than a bounded region, unlike the note in
    /// `build_sculpt_preview_texture` inviting one: this pass is a flat
    /// per-cell colour lookup with no derived whole-grid rasters
    /// underneath it (no `RenderCtx`, no AO/wetness/hillshade), so the
    /// cost a bounded variant would save is negligible here, while a
    /// bounded texture would need this method to also report an offset
    /// for the caller to composite it at — a second return value this
    /// signature doesn't have.
    ///
    /// `None` before any `generate()` call, or when the active layer has
    /// nothing painted and nothing pending (matching nothing would differ
    /// from a texture that is fully transparent everywhere).
    #[func]
    fn build_paint_preview_texture(&self) -> Option<Gd<ImageTexture>> {
        let p = self.paint.as_ref()?;
        if p.active_layer().is_empty() && p.active_draft().is_empty() {
            return None;
        }
        let gw = self.gw as usize;
        let gh = self.gh as usize;
        let n = gw * gh;
        let base: Vec<u8> = p.active_layer().cells().map(<[u8]>::to_vec).unwrap_or_else(|| vec![0u8; n]);
        let mut scratch = vec![0u8; n];
        p.active_draft().preview_into(&base, &mut scratch);
        let palette_len = p.layer.palette().len();

        let mut bytes = Vec::with_capacity(n * 4);
        for &v in &scratch {
            if v == 0 {
                bytes.extend_from_slice(&[0, 0, 0, 0]);
            } else {
                let (r, g, b) = paint_bridge::swatch_color(p.layer, v, palette_len);
                bytes.extend_from_slice(&[r, g, b, 255]);
            }
        }
        let packed = PackedByteArray::from(bytes);
        let image = Image::create_from_data(gw as i32, gh as i32, false, Format::RGBA8, &packed)?;
        ImageTexture::create_from_image(&image)
    }

    /// Per-class painted-cell counts for the active layer, live — the same
    /// composite `build_paint_preview_texture` renders, summarised instead
    /// of rasterised (`paint_bridge::PaintEditor::painted_counts`).
    /// `{"layer": String, "total": int, "counts": {index:int -> count:int}}`
    /// — `counts` has one entry per legal index of the active layer's own
    /// palette (`get_paint_palette`'s own `index`), zero-count entries
    /// included, so a legend can render every class every time rather than
    /// only the ones currently painted. Empty `Dictionary` before any
    /// `generate()` call.
    #[func]
    fn paint_painted_counts(&self) -> VarDictionary {
        let Some(p) = self.paint.as_ref() else { return VarDictionary::new() };
        let n = (self.gw as usize) * (self.gh as usize);
        let (total, by_class) = p.painted_counts(n);
        let mut counts = VarDictionary::new();
        for (i, &c) in by_class.iter().enumerate() {
            counts.set((i + 1) as i32, c);
        }
        vdict! { "layer" => p.layer.key(), "total" => total, "counts" => &counts }
    }

    /// Pending, uncommitted dabs across all three paint drafts — what
    /// `paint_commit` would bake and `paint_discard` would throw away
    /// (`paint_bridge::PaintEditor::pending_stamps`), and therefore the
    /// number the Commit / Discard pair must gate on rather than
    /// `paint_painted_counts`'s composite total
    /// (`GUI_GAP_REGISTER.md` WW-13). `0` before any `generate()` call.
    #[func]
    fn paint_draft_count(&self) -> i64 {
        self.paint.as_ref().map_or(0, |p| p.pending_stamps() as i64)
    }

    // ---- commit / discard ----

    /// Bakes every layer's pending draft into its own override array
    /// (`paint_bridge::PaintEditor::commit_all`) and clears all three
    /// drafts. **Deliberately does not touch `field`/`temperature`/
    /// `rainfall`/`flow_discharge` at all** — `UNIFIED_TOOL_PLAN.md`'s own
    /// Biome-paint staleness note, *"painting biome does not mark height/
    /// hydrology/climate dirty (it's downstream, read-only of those)"*,
    /// holds here by construction: nothing in `PaintEditor::commit_all`
    /// borrows `WorldState` at all.
    ///
    /// `DCC_SHELL_SPEC.md` §4.5.2 names the downstream cost precisely:
    /// *"Commit ... marks stages 09 and 10 stale (a painted biome
    /// overrides classification for the cells it covers; soils and
    /// resources depend on it)."* `stale_stages` below reports exactly
    /// that pair as **data for a caller's own status bar**, not a live
    /// call into `cartalith_engine::staleness::StageGraph` — that graph is
    /// deliberately unwired into `WorldGen` altogether (its own module
    /// doc: *"milestone A ships the mechanism, and milestones B-F wire it
    /// to real tools"*), the exact same gap `sculpt_commit`'s own doc
    /// comment already discloses for the height/hydrology/climate chain.
    /// Wiring a live graph into `WorldGen` is real follow-up work for
    /// whichever milestone actually re-runs ecology/resources on demand,
    /// not something to improvise here.
    ///
    /// Returns a summary `Dictionary`: one `{stamps_applied,
    /// stamps_skipped}` sub-dictionary per layer key (`"biome"`/
    /// `"terrain"`/`"splat"`), `"tiles_marked"` (`PackedInt32Array`, the
    /// union across all three — `paint_bridge`'s three drafts share one
    /// `DirtyTracker`), and `"stale_stages"` (`PackedStringArray`, always
    /// `["ecology_biomes", "resources_soils"]` when anything was actually
    /// painted this commit, empty otherwise). Empty `Dictionary` before
    /// any `generate()` call.
    #[func]
    fn paint_commit(&mut self) -> VarDictionary {
        let Some(p) = self.paint.as_mut() else { return VarDictionary::new() };
        let n = (self.gw as usize) * (self.gh as usize);
        let [biome, terrain, splat] = p.commit_all(n);

        let mut tiles: std::collections::BTreeSet<i32> = std::collections::BTreeSet::new();
        for s in [&biome, &terrain, &splat] {
            tiles.extend(s.tiles_marked.iter().map(|&t| t as i32));
        }
        let tiles_marked: PackedInt32Array = tiles.into_iter().collect();

        let any_applied = biome.stamps_applied > 0 || terrain.stamps_applied > 0 || splat.stamps_applied > 0;
        // ED-02, recorded and not reversible: `paint_discard` drops a draft,
        // but a committed dab is merged into the accumulated layer and the
        // pre-commit layer is not kept.
        if any_applied {
            self.ledger.record(
                "paint",
                "Paint commit",
                format!(
                    "{} stamps - {} tiles",
                    biome.stamps_applied + terrain.stamps_applied + splat.stamps_applied,
                    tiles_marked.len()
                ),
                undo::EntryKind::Recorded(
                    "the pre-commit paint layer is not retained; Discard reverts an uncommitted draft only",
                ),
            );
        }
        // Record it in the live graph too, as `PipelineStage::Civ`'s own
        // change, and run the same consumer the terrain commits run. This is
        // what makes the graph's answer to "what must re-run" honest rather
        // than a special case: a mid-chain edit does not make its own
        // upstreams stale, so the shared consumer correctly re-runs neither
        // hydrology nor climate for a painted biome, and `recomputed` comes
        // back empty. The `stale_stages` list below stays as it was -- it
        // names the two civ *sub*-stages `DCC_SHELL_SPEC.md` §4.5.2 calls
        // out, which this four-stage graph does not resolve separately.
        let tiles: Vec<usize> = tiles_marked.as_slice().iter().map(|&t| t as usize).collect();
        let (recomputed, still_stale) = self.mark_and_recompute(PipelineStage::Civ, tiles, "paint");
        let stale_stages: PackedStringArray = if any_applied {
            ["ecology_biomes", "resources_soils"].iter().map(|s| GString::from(*s)).collect()
        } else {
            PackedStringArray::new()
        };

        fn summary_dict(s: &cartalith_spatial::CommitSummary) -> VarDictionary {
            vdict! { "stamps_applied" => s.stamps_applied as i64, "stamps_skipped" => s.stamps_skipped as i64 }
        }
        vdict! {
            "biome" => &summary_dict(&biome),
            "terrain" => &summary_dict(&terrain),
            "splat" => &summary_dict(&splat),
            "tiles_marked" => &tiles_marked,
            "stale_stages" => &stale_stages,
            "recomputed" => &recomputed,
            "still_stale" => &still_stale,
        }
    }

    /// Drops every layer's pending draft, touching nothing committed
    /// (`paint_bridge::PaintEditor::discard_all`). Returns how many dabs
    /// were dropped in total across all three layers, `0` before any
    /// `generate()` call.
    #[func]
    fn paint_discard(&mut self) -> i32 {
        self.paint.as_mut().map_or(0, |p| p.discard_all() as i32)
    }
}

/// `UNIFIED_TOOL_PLAN.md` milestone F, the INFRA domain's Way/Route tools
/// plus the two global tools that ride in the same left-dock TOOLS block
/// (`DCC_SHELL_SPEC.md` §4.5.1 Measure/Region select, §4.5.4 Way/Route) --
/// see `infra_tools_bridge.rs`'s own module doc for the state this thin
/// layer wraps, and `WorldGen::infra`'s own field doc for why it lives on
/// `WorldGen` rather than a second `GodotClass`, the same reasoning
/// `sculpt`/`icons`/`civ_tools`/`paint`/`labels` above already follow.
///
/// Plain (non-`#[func]`) helper shared by every method below -- kept out of
/// the `#[godot_api(secondary)]` block since it is Rust-internal, the same
/// split `call_params`/`absorb` use for `generate()`'s own helpers.
impl WorldGen {
    /// Snaps a click to the nearest settlement pin or existing way within
    /// `civ_snap_radius`'s own reach (`cartalith_civ::tools::
    /// civ_snap_radius`'s base radius, `max(5, GW/70)`, reference line
    /// 16003) -- the real engine primitive behind `DCC_SHELL_SPEC.md`
    /// §4.5.4's "snap to places" modifier, applied unconditionally by
    /// `way_append_point`/`route_append_stop` below. The reference's own
    /// `_civSnapEnabled` on/off preference (`state.viz.snapWays`, "on by
    /// default") is a shell toggle this binding does not model a switch
    /// for -- shipping it unconditionally is what "on by default" gives
    /// without one.
    ///
    /// Considers both the generated road network (`self.civ.ways`) and any
    /// hand-drawn ways committed so far this session (`self.infra.ways`) --
    /// a click can snap onto a way it is about to continue past. Returns
    /// the raw click unchanged before any civ data exists
    /// (`self.civ.is_none()`, the same gate `way_commit`/`route_commit`
    /// use) or when nothing is within reach.
    fn snap_point(&self, gx: f64, gy: f64) -> (f64, f64) {
        let Some(civ) = self.civ.as_ref() else { return (gx, gy) };
        let mut ways: Vec<cartalith_civ::tools::WayRef> = civ.ways.iter().map(cartalith_civ::tools::WayRef::from).collect();
        if let Some(infra) = self.infra.as_ref() {
            ways.extend(infra.ways.iter().map(cartalith_civ::tools::WayRef::from));
        }
        let radius = cartalith_civ::tools::civ_snap_radius(self.gw.max(0) as usize);
        cartalith_civ::tools::civ_snap_point(&civ.settlements, &ways, gx, gy, radius)
    }
}

#[godot_api(secondary)]
impl WorldGen {
    // ===================== Way (DCC_SHELL_SPEC.md §4.5.4) =====================

    /// Arms the Way tool with a type (`"road"`/`"track"`/`"sea_lane"`/
    /// `"ancient"` -- `infra_tools_bridge::parse_way_type`'s own doc
    /// comment on why this is the engine's real four-entry enum, not
    /// §4.5.4's differing "road/track/trail/bridge" UI list). `false` for
    /// an unrecognised string or before any `generate()` call -- no draft
    /// is started either way.
    #[func]
    fn way_begin(&mut self, way_type: GString) -> bool {
        let Some(t) = infra_tools_bridge::parse_way_type(&way_type.to_string()) else { return false };
        let Some(infra) = self.infra.as_mut() else { return false };
        infra.way_begin(t);
        true
    }

    /// Appends one waypoint to the in-progress way, snapped to a nearby
    /// place or way first (`snap_point`, "on by default" per §4.5.4).
    /// `false` if no Way draft is currently armed.
    #[func]
    fn way_append_point(&mut self, gx: f64, gy: f64) -> bool {
        let (sx, sy) = self.snap_point(gx, gy);
        self.infra.as_mut().is_some_and(|i| i.way_append_point(sx, sy))
    }

    /// Commits the in-progress way via real least-cost Dijkstra routing
    /// (`civ_commit_way`; `infra_tools_bridge`'s module doc explains why
    /// there is no "freehand" alternative to route through). Returns the
    /// new way's index (`get_settlements()`/`get_roads()`-style, a plain
    /// position into the committed list -- readable back through
    /// `get_roads()`/`get_sea_routes()`, which since `GUI_GAP_REGISTER.md`
    /// IN-02 append every committed manual way to the generated network
    /// they return, tagged `manual: true`; note this index counts *all*
    /// committed ways including sea lanes, so it is not an offset into
    /// either getter's array), or `-1` for no draft, fewer than two waypoints, or no
    /// `generate()` yet. Prints (does not block or discard the way) when
    /// some leg had to fall back to a straight line across terrain this
    /// way type is meant to avoid -- `CommitWay::unreachable_legs`'s own
    /// doc: "the reference alerts and keeps the way."
    #[func]
    fn way_commit(&mut self) -> i64 {
        let Some(mode) = self.infra.as_ref().and_then(|i| i.way_draft_mode()) else { return -1 };
        let (Some(WorldSource::Generated(ws)), Some(civ)) = (self.source.as_ref(), self.civ.as_ref()) else {
            if let Some(i) = self.infra.as_mut() {
                i.way_discard();
            }
            return -1;
        };
        let inputs = infra_tools_bridge::RouteInputs::build(
            ws, self.gw as usize, self.gh as usize, self.world, self.map_width_km, self.params.river_density, mode,
        );
        // An owned clone, not a borrow of `self.infra.ways` -- so that
        // borrow ends right here, before `way_commit` below needs
        // `&mut self.infra` to push the new way onto that same field.
        let manual_ways = self.infra.as_ref().map(|i| i.ways.clone()).unwrap_or_default();
        let mut way_refs: Vec<cartalith_civ::tools::WayRef> = civ.ways.iter().map(cartalith_civ::tools::WayRef::from).collect();
        way_refs.extend(manual_ways.iter().map(cartalith_civ::tools::WayRef::from));
        let ctx = cartalith_civ::tools::RouteContext {
            field: &ws.field,
            water_bodies: &inputs.water_bodies,
            biome: inputs.biome.as_deref(),
            river_order: inputs.river_order.as_deref(),
            places: &civ.settlements,
            ways: &way_refs,
            gw: self.gw as usize,
            gh: self.gh as usize,
            sea: self.sea_level,
            world: self.world,
            map_width_km: self.map_width_km,
        };
        let Some(infra) = self.infra.as_mut() else { return -1 };
        match infra.way_commit(&ctx) {
            Some((idx, unreachable)) => {
                if unreachable > 0 {
                    godot_print!(
                        "cartalith-godot: way commit has {unreachable} unreachable leg(s) -- straight-line fallback across terrain this way type avoids"
                    );
                }
                idx as i64
            }
            None => -1,
        }
    }

    /// Discards the in-progress way without committing it.
    #[func]
    fn way_discard(&mut self) {
        if let Some(i) = self.infra.as_mut() {
            i.way_discard();
        }
    }

    // ===================== Route (DCC_SHELL_SPEC.md §4.5.4) =====================

    /// Arms the Route tool with a `RouteMode` (`"land"`/`"water"`/
    /// `"mixed"`, `"least_cost"`/`"least-cost"` accepted as a `"mixed"`
    /// alias -- `infra_tools_bridge::parse_route_mode`'s own doc comment on
    /// why §4.5.4's other two labels, "freehand" and "snap", are not
    /// accepted: no distinct engine algorithm backs either). `false` for an
    /// unrecognised string or before any `generate()` call.
    #[func]
    fn route_begin(&mut self, mode: GString) -> bool {
        let Some(m) = infra_tools_bridge::parse_route_mode(&mode.to_string()) else { return false };
        let Some(infra) = self.infra.as_mut() else { return false };
        infra.route_begin(m);
        true
    }

    /// Appends one stop to the in-progress route, snapped to a nearby place
    /// or way first, same as `way_append_point`. `false` if no Route draft
    /// is currently armed.
    #[func]
    fn route_append_stop(&mut self, gx: f64, gy: f64) -> bool {
        let (sx, sy) = self.snap_point(gx, gy);
        self.infra.as_mut().is_some_and(|i| i.route_append_stop(sx, sy))
    }

    /// Commits the in-progress route via `civ_join_dijkstra_segs` under the
    /// mode `route_begin` armed. Same return convention as `way_commit`
    /// (new index or `-1`), and the same "prints rather than drops the
    /// route" handling for unreachable legs.
    #[func]
    fn route_commit(&mut self) -> i64 {
        let Some(mode) = self.infra.as_ref().and_then(|i| i.route_draft_mode()) else { return -1 };
        let (Some(WorldSource::Generated(ws)), Some(civ)) = (self.source.as_ref(), self.civ.as_ref()) else {
            if let Some(i) = self.infra.as_mut() {
                i.route_discard();
            }
            return -1;
        };
        let inputs = infra_tools_bridge::RouteInputs::build(
            ws, self.gw as usize, self.gh as usize, self.world, self.map_width_km, self.params.river_density, mode,
        );
        let manual_ways = self.infra.as_ref().map(|i| i.ways.clone()).unwrap_or_default();
        let mut way_refs: Vec<cartalith_civ::tools::WayRef> = civ.ways.iter().map(cartalith_civ::tools::WayRef::from).collect();
        way_refs.extend(manual_ways.iter().map(cartalith_civ::tools::WayRef::from));
        let ctx = cartalith_civ::tools::RouteContext {
            field: &ws.field,
            water_bodies: &inputs.water_bodies,
            biome: inputs.biome.as_deref(),
            river_order: inputs.river_order.as_deref(),
            places: &civ.settlements,
            ways: &way_refs,
            gw: self.gw as usize,
            gh: self.gh as usize,
            sea: self.sea_level,
            world: self.world,
            map_width_km: self.map_width_km,
        };
        let Some(infra) = self.infra.as_mut() else { return -1 };
        match infra.route_commit(&ctx) {
            Some((idx, unreachable)) => {
                if unreachable > 0 {
                    godot_print!("cartalith-godot: route commit has {unreachable} unreachable leg(s) -- straight-line fallback");
                }
                idx as i64
            }
            None => -1,
        }
    }

    /// Discards the in-progress route without committing it.
    #[func]
    fn route_discard(&mut self) {
        if let Some(i) = self.infra.as_mut() {
            i.route_discard();
        }
    }

    /// How many routes have been committed this session. `0` before any
    /// `generate()` call.
    ///
    /// `way_commit`/`route_commit` return an index into a list nothing
    /// could read back until now -- the INFRA milestone disclosed that gap
    /// rather than inventing a getter for it, and the Journey Planner
    /// binding below is its first real consumer (`jp_compute`'s `route`
    /// key names a committed route by exactly this index).
    #[func]
    fn route_count(&self) -> i64 {
        self.infra.as_ref().map_or(0, |i| i.routes.len() as i64)
    }

    /// Deletes one committed route, shifting every later index down by one
    /// (`InfraTools::route_delete`, i.e. the reference's own
    /// `civJourneys.splice(ji,1)` behind its per-row `×` button, reference
    /// line 17250). `false` for an out-of-range index or before any
    /// `generate()` call.
    ///
    /// **Indices renumber.** Anything holding a route index across this call
    /// -- `jp_compute`'s `route` key, `jp_reroute`'s `route_index`, a list
    /// row -- must re-read `route_count()`/`route_get()` afterwards, exactly
    /// as the reference's list does by re-rendering itself on delete. That
    /// is the deliberate choice `route_delete`'s own doc comment defends
    /// (a tombstone would break `route_count()`'s meaning instead).
    #[func]
    fn route_delete(&mut self, index: i64) -> bool {
        let Ok(i) = usize::try_from(index) else { return false };
        self.infra.as_mut().is_some_and(|t| t.route_delete(i))
    }

    /// Renames one committed route -- the reference journey list's own
    /// name field (`nameInput.oninput=()=>{ jn.name=nameInput.value; }`,
    /// line 17245). `false` for an out-of-range index or before any
    /// `generate()` call.
    ///
    /// An empty string is a legal name and restores the unnamed state; the
    /// `Journey N` fallback the reference shows for it is computed by
    /// whoever draws the list, never stored (see `CommittedRoute::name`).
    #[func]
    fn route_set_name(&mut self, index: i64, name: GString) -> bool {
        let Ok(i) = usize::try_from(index) else { return false };
        let name = name.to_string();
        self.infra.as_mut().is_some_and(|t| t.route_set_name(i, &name))
    }

    /// One committed route: `{"points": PackedVector2Array, "brks":
    /// PackedInt32Array, "km": float, "mode": String, "unreachable_legs":
    /// int, "name": String}` -- `civ_join_dijkstra_segs`' own
    /// `{pts, brks, km}` shape plus
    /// the `RouteMode` it was solved under and how many legs fell back to a
    /// straight line. Empty `Dictionary` for an out-of-range index or
    /// before any `generate()` call, the same convention `icon_get`/
    /// `label_get` use.
    ///
    /// `points` is `PackedVector2Array`, matching `get_roads()`/
    /// `get_sea_routes()`, so it is `f32` -- fine to draw with, and fine to
    /// feed back to `jp_compute` via its own `points` key, but `jp_compute`
    /// prefers the `route` index precisely so the planner samples the
    /// route's real `f64` grid coordinates rather than a rounded copy.
    ///
    /// # Draw `render_points`/`render_brks`, plan against `points`
    ///
    /// A committed route's `pts` are `civ_join_dijkstra_segs`' output, i.e.
    /// the same `civ_smooth_path` curve at the same 3-grid-cell sampling
    /// `get_roads()`' own doc comment measures as an ~87 px straight chord
    /// at `ViewportHost.ZOOM_MAX`. `render_points` is that curve re-sampled
    /// through the same control points at [`WAY_RENDER_STEP_CELLS`], with
    /// `render_brks` remapped onto it — the identical treatment
    /// `get_roads()`/`get_sea_routes()` apply to theirs.
    ///
    /// It is a *second* key rather than a denser `points` because unlike
    /// those two getters, this list is indexed into: `jp_compute` plans
    /// over `CommittedRoute::pts` and returns `plan.stages[i].{i0, i1}` as
    /// indices into exactly that list, which `journey_planner_view.gd`
    /// slices to colour the route map per stage. Densifying `points` would
    /// silently mis-slice every stage. So `points` stays the engine's own
    /// list, 1:1 with what `jp_compute` planned over, and only the drawn
    /// polyline is refined.
    #[func]
    fn route_get(&self, index: i64) -> VarDictionary {
        let Ok(i) = usize::try_from(index) else { return VarDictionary::new() };
        let Some(r) = self.infra.as_ref().and_then(|t| t.routes.get(i)) else { return VarDictionary::new() };
        let points: PackedVector2Array = r.pts.iter().map(|&(x, y)| Vector2::new(x as f32, y as f32)).collect();
        let brks: PackedInt32Array = r.brks.iter().map(|&b| b as i32).collect();
        let (render_points, render_brks) = way_render_geometry(&r.pts, &r.brks);
        let mode = match r.mode {
            cartalith_civ::tools::RouteMode::Land => "land",
            cartalith_civ::tools::RouteMode::Water => "water",
            cartalith_civ::tools::RouteMode::Mixed => "mixed",
        };
        vdict! {
            "points" => &points,
            "brks" => &brks,
            "render_points" => &render_points,
            "render_brks" => &render_brks,
            "km" => r.km,
            "mode" => mode,
            "unreachable_legs" => r.unreachable_legs as i64,
            "name" => r.name.as_str(),
        }
    }

    // ===================== Measure (DCC_SHELL_SPEC.md §4.5.1, global) =====================
    //
    // No golden-parity test exists for this tool and none can (see
    // `cartalith_spatial::measure`'s own module doc: "zero reference
    // precedent"); the km scale it reports is real (`cell_km`, the same
    // expression every route length in this port already uses).

    /// Starts a fresh Measure click chain, discarding any previous one.
    #[func]
    fn measure_begin(&mut self) {
        if let Some(i) = self.infra.as_mut() {
            i.measure_begin();
        }
    }

    /// Appends one point to the Measure chain. No snapping -- Measure is a
    /// raw ruler, not a routing tool, and §4.5.1's own row lists no snap
    /// modifier for it (unlike Way/Route's "snap to places").
    #[func]
    fn measure_add_point(&mut self, gx: f64, gy: f64) {
        if let Some(i) = self.infra.as_mut() {
            i.measure_add_point(gx, gy);
        }
    }

    /// The current chain's reading: `segments` (`Array<Dictionary>`, one
    /// `{cells, km, bearing_deg}` per leg in click order -- `bearing_deg`
    /// is this bridge's own convention, see `infra_tools_bridge::
    /// MeasuredLeg`'s doc comment, since no reference bearing readout
    /// exists to match), `total_cells`/`total_km` (the running total,
    /// summed along the path), `straight_line_cells`/`straight_line_km`
    /// (first point to last, direct -- §4.5.1's own right-dock "straight-
    /// line vs along-path difference"), and `point_count`. Empty
    /// `Dictionary` before any `generate()` call.
    #[func]
    fn measure_result(&self) -> VarDictionary {
        let Some(infra) = self.infra.as_ref() else { return VarDictionary::new() };
        let pts = infra.measure_points();
        let gw = self.gw.max(0) as usize;
        let legs = infra_tools_bridge::measure_legs(pts, gw, self.map_width_km, self.world);
        let mut segments: Array<VarDictionary> = Array::new();
        let mut total_cells = 0.0f64;
        let mut total_km = 0.0f64;
        for leg in &legs {
            total_cells += leg.m.cells;
            total_km += leg.m.km;
            segments.push(&dict! {
                "cells" => leg.m.cells,
                "km" => leg.m.km,
                "bearing_deg" => leg.bearing_deg,
            });
        }
        let (straight_cells, straight_km, overall_bearing) = match (pts.first(), pts.last()) {
            (Some(&a), Some(&b)) if pts.len() >= 2 => {
                let leg = infra_tools_bridge::measure_leg(a, b, gw, self.map_width_km, self.world);
                (leg.m.cells, leg.m.km, leg.bearing_deg)
            }
            _ => (0.0, 0.0, 0.0),
        };
        // `design/Cartalith Measurement Toolbar.dc.html` state 1's DERIVED
        // block. The three relief rows need the height field, which the
        // chain itself has no access to, so they come from `measure_bridge`
        // and read `—` (as zeros with `has_relief` false) whenever
        // `sample_refs()` does -- a loaded save, or before any `generate()`.
        let relief = self.sample_refs().map(|f| measure_bridge::chain_relief(&f, pts));
        dict! {
            "segments" => &segments,
            "total_cells" => total_cells,
            "total_km" => total_km,
            "straight_line_cells" => straight_cells,
            "straight_line_km" => straight_km,
            "overall_bearing_deg" => overall_bearing,
            "point_count" => pts.len() as i64,
            "has_relief" => relief.is_some(),
            "elevation_delta_m" => relief.map(|r| r.elevation_delta_m).unwrap_or(0.0),
            "total_km_3d" => relief.map(|r| r.total_km_3d).unwrap_or(0.0),
            "sinuosity" => relief.map(|r| r.sinuosity).unwrap_or(1.0),
        }
    }

    // ===================== Measurement toolbar (design canvas, 2026-08-23) =====================
    //
    // `design/Cartalith Measurement Toolbar.dc.html`'s three states beyond the
    // ruler above. Every one of these is stateless: the caller owns the
    // points (the tool overlay is already drawing them) and asks for a
    // reading. Nothing is retained on `WorldGen`, which is why none of them
    // has a `begin`/`clear` pair the way `measure_*` does -- there is no
    // chain to accumulate. See `measure_bridge.rs`'s module doc for what is
    // a port here (the polygon primitives, golden-verified) and what is new
    // (everything else, `DECISIONS.md` §7d).

    /// The cross-section profile between two grid points (canvas state 2).
    ///
    /// Returns `samples` (`Array<Dictionary>`, one per sample point in
    /// order: `km`, `x`, `y`, `elev_m`, `slope_deg`, `temp_c`, `rain`,
    /// `flow`, `river_order`, `lithology`, and `biome`/`water` where a
    /// civilisation layer exists), `stats` (the canvas's own PROFILE
    /// STATISTICS block), `crossings` (`km`/`kind`/`label`/`elev_m`), plus
    /// `length_km`, `length_3d_km`, `bearing_deg` and `spacing_m`.
    ///
    /// `samples` is clamped to `2 ..= 1024`. Empty `Dictionary` when there is
    /// no generated world to read.
    #[func]
    fn measure_section(&self, ax: f64, ay: f64, bx: f64, by: f64, samples: i64) -> VarDictionary {
        let Some(f) = self.sample_refs() else { return VarDictionary::new() };
        let p = measure_bridge::section_profile(&f, (ax, ay), (bx, by), samples.max(0) as usize);
        let mut out: Array<VarDictionary> = Array::new();
        for s in &p.samples {
            let mut d = dict! {
                "km" => s.km,
                "x" => s.x as i64,
                "y" => s.y as i64,
                "elev_m" => s.elev_m,
                "slope_deg" => s.slope_deg,
                "temp_c" => s.temp_c,
                "rain" => s.rain,
                "flow" => s.flow,
                "river_order" => s.river_order,
                "lithology" => s.lithology,
            };
            // Omitted rather than zeroed when absent -- `right_dock.gd`'s own
            // rule: an absent key becomes an em dash, a present zero is a
            // real reading.
            if let Some(b) = s.biome {
                d.set("biome", b);
            }
            if let Some(w) = s.water {
                d.set("water", w as i64);
            }
            out.push(&d);
        }
        let mut crossings: Array<VarDictionary> = Array::new();
        for c in &p.crossings {
            crossings.push(&dict! {
                "km" => c.km,
                "kind" => c.kind,
                "label" => c.label.clone(),
                "elev_m" => c.elev_m,
            });
        }
        let st = &p.stats;
        let stats: VarDictionary = dict! {
            "min_m" => st.min_m,
            "max_m" => st.max_m,
            "mean_m" => st.mean_m,
            "ascent_m" => st.ascent_m,
            "descent_m" => st.descent_m,
            "net_m" => st.net_m,
            "mean_slope_deg" => st.mean_slope_deg,
            "max_slope_deg" => st.max_slope_deg,
            "above_2000m_km" => st.above_2000m_km,
            "river_crossings" => st.river_crossings as i64,
            "ridge_crossings" => st.ridge_crossings as i64,
            "shore_crossings" => st.shore_crossings as i64,
        };
        dict! {
            "samples" => &out,
            "crossings" => &crossings,
            "length_km" => p.length_km,
            "length_3d_km" => p.length_3d_km,
            "bearing_deg" => p.bearing_deg,
            "spacing_m" => p.spacing_m,
            "stats" => &stats,
        }
    }

    /// The Area tool's reading over a closed ring given in grid cells (canvas
    /// state 3). Fewer than three vertices returns a zeroed reading, not an
    /// error -- a ring under construction is a normal state.
    ///
    /// `projected_km2` is the exact shoelace figure; `true_surface_km2`,
    /// `water_km2`, `land_km2` and `mean_elev_m` come from a strided walk of
    /// the bounding box and report the `stride` and `sampled_cells` they used
    /// so a caller can say so.
    #[func]
    fn measure_area(&self, points: PackedVector2Array) -> VarDictionary {
        let Some(f) = self.sample_refs() else { return VarDictionary::new() };
        let pts: Vec<(f64, f64)> = points.as_slice().iter().map(|p| (p.x as f64, p.y as f64)).collect();
        let a = measure_bridge::area_measure(&f, &pts);
        dict! {
            "vertices" => a.vertices as i64,
            "projected_km2" => a.projected_km2,
            "true_surface_km2" => a.true_surface_km2,
            "perimeter_km" => a.perimeter_km,
            "centroid_x" => a.centroid.0,
            "centroid_y" => a.centroid.1,
            "bbox_x" => a.bbox.0,
            "bbox_y" => a.bbox.1,
            "bbox_w" => a.bbox.2,
            "bbox_h" => a.bbox.3,
            "bbox_w_km" => a.bbox_w_km,
            "bbox_h_km" => a.bbox_h_km,
            "water_km2" => a.water_km2,
            "land_km2" => a.land_km2,
            "mean_elev_m" => a.mean_elev_m,
            "sampled_cells" => a.sampled_cells as i64,
            "stride" => a.stride as i64,
            "water_from_civ" => a.water_from_civ,
        }
    }

    /// The Radius tool: centre plus a rim point (canvas state 3).
    #[func]
    fn measure_radius(&self, cx: f64, cy: f64, px: f64, py: f64) -> VarDictionary {
        let Some(f) = self.sample_refs() else { return VarDictionary::new() };
        let r = measure_bridge::radius_measure(&f, (cx, cy), (px, py));
        dict! {
            "radius_km" => r.radius_km,
            "diameter_km" => r.diameter_km,
            "circumference_km" => r.circumference_km,
            "area_km2" => r.area_km2,
        }
    }

    /// The Δ-vertical / 3D-distance pair over two points (canvas state 3's
    /// VERTICAL · TWO POINTS block).
    #[func]
    fn measure_vertical(&self, ax: f64, ay: f64, bx: f64, by: f64) -> VarDictionary {
        let Some(f) = self.sample_refs() else { return VarDictionary::new() };
        let v = measure_bridge::vertical_measure(&f, (ax, ay), (bx, by));
        dict! {
            "p1_elev_m" => v.p1_elev_m,
            "p2_elev_m" => v.p2_elev_m,
            "delta_m" => v.delta_m,
            "horizontal_km" => v.horizontal_km,
            "distance_3d_km" => v.distance_3d_km,
            "grade_pct" => v.grade_pct,
            "angle_deg" => v.angle_deg,
        }
    }

    /// Clears the current Measure chain without ending the tool.
    #[func]
    fn measure_clear(&mut self) {
        if let Some(i) = self.infra.as_mut() {
            i.measure_clear();
        }
    }

    // ===================== Region select (DCC_SHELL_SPEC.md §4.5.1, global) =====================
    //
    // "Region select is the marquee §9's export route was missing:
    // dragging it fills the route's world-bounds fields, and the route's
    // fields write back to the marquee. Neither is authoritative -- they
    // are two views of one rect (`region_export.rs`)." `region_set`/
    // `region_get` below and `region_export_tiles` all read/write the
    // SAME `self.infra.region`, which is that one rect.

    /// Sets the marquee from a `(gx, gy)` origin and a `(gw, gh)` size (not
    /// two opposite corners), clamped and normalised to this world's own
    /// grid via `norm_region` -- the same primitive `export_region_tiles`'s
    /// own caller-facing selection type is built with, so a marquee this
    /// sets is guaranteed a legal `region_export_tiles` input. A no-op
    /// before any `generate()`/`load_save()` call.
    #[func]
    fn region_set(&mut self, gx: f64, gy: f64, gw: f64, gh: f64) {
        if self.gw <= 0 || self.gh <= 0 {
            return;
        }
        let r = cartalith_spatial::norm_region(gx, gy, gx + gw, gy + gh, self.gw as usize, self.gh as usize, None, None);
        if let Some(infra) = self.infra.as_mut() {
            infra.region_set(r);
        }
    }

    /// The current marquee, or an empty `Dictionary` if none is set (or
    /// before any `generate()` call): `x`/`y`/`w`/`h` in cells, `x_km`/
    /// `y_km`/`w_km`/`h_km` in kilometres (`cell_km`), `cell_count`
    /// (`w * h`), and `tile_estimates` (`Array<Dictionary>`, one
    /// `{lod, tiles, tile_w, tile_h}` per `infra_tools_bridge::
    /// REGION_LOD_GRIDS` tier -- see that constant's own doc comment for
    /// why this port picked a three-tier ladder with no reference
    /// precedent to match).
    #[func]
    fn region_get(&self) -> VarDictionary {
        let Some(r) = self.infra.as_ref().and_then(|i| i.region) else { return VarDictionary::new() };
        let km_per_cell = cartalith_spatial::cell_km(self.map_width_km, self.gw.max(1) as usize);
        let estimates: Array<VarDictionary> = infra_tools_bridge::region_tile_estimate(&r)
            .into_iter()
            .map(|(label, tiles, tw, th)| {
                dict! { "lod" => label, "tiles" => tiles as i64, "tile_w" => tw as i64, "tile_h" => th as i64 }
            })
            .collect();
        dict! {
            "x" => r.x as i64,
            "y" => r.y as i64,
            "w" => r.w as i64,
            "h" => r.h as i64,
            "x_km" => r.x as f64 * km_per_cell,
            "y_km" => r.y as f64 * km_per_cell,
            "w_km" => r.w as f64 * km_per_cell,
            "h_km" => r.h as f64 * km_per_cell,
            "cell_count" => (r.w * r.h) as i64,
            "tile_estimates" => &estimates,
        }
    }

    /// Clears the marquee.
    #[func]
    fn region_clear(&mut self) {
        if let Some(i) = self.infra.as_mut() {
            i.region_clear();
        }
    }

    /// The Data manager's real export route (`DCC_SHELL_SPEC.md` §9) over
    /// the current marquee -- `export_region_tiles` + `zip_region_export`
    /// (`cartalith_engine::region_export`), unchanged. Works over either a
    /// freshly generated world or a loaded save (`field` is read from
    /// whichever `WorldSource` is live, the same fallback
    /// `build_color_texture` already uses) -- unlike Way/Route, nothing
    /// here needs the civ layer.
    ///
    /// `opts` keys, all optional: `cols`/`rows`/`tile_size` (int, default
    /// `4`/`4`/`512`), `gzip`/`ridged`/`visual` (bool, default `false`),
    /// `version` (String, default `"cartalith-native"`), `detail_freq`/
    /// `detail_amp` (float, `AmplifyOpts` defaults `1.0`/`0.14`), and, only
    /// read when `visual` is `true`, `sun_az_deg`/`exag` (float, `315.0`/
    /// `3.4` -- `TileVisual::default()`'s own values). `seed` and `sea`
    /// come from this world's own state, not `opts` -- an export must match
    /// the world it was drawn over, not a caller-guessed one.
    ///
    /// Returns the zipped archive's bytes, or an empty `PackedByteArray`
    /// with no marquee set, no world loaded, or on a zip failure (printed).
    #[func]
    fn region_export_tiles(&self, opts: VarDictionary) -> PackedByteArray {
        let Some(region) = self.infra.as_ref().and_then(|i| i.region) else { return PackedByteArray::new() };
        let field: &[f32] = match self.source.as_ref() {
            Some(WorldSource::Generated(ws)) => &ws.field,
            Some(WorldSource::Loaded(save)) => &save.fields.heightmap,
            None => return PackedByteArray::new(),
        };
        let get_num = |key: &str| opts.get(key).and_then(|v| variant_to_num(&v));
        let cols = get_num("cols").map(|n| n as usize).filter(|&n| n > 0).unwrap_or(4);
        let rows = get_num("rows").map(|n| n as usize).filter(|&n| n > 0).unwrap_or(4);
        let tile_size = get_num("tile_size").map(|n| n as usize).filter(|&n| n > 0).unwrap_or(512);
        let gzip = opts.get("gzip").and_then(|v| v.try_to::<bool>().ok()).unwrap_or(false);
        let version = opts
            .get("version")
            .and_then(|v| v.try_to::<GString>().ok())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "cartalith-native".to_string());
        let visual = if opts.get("visual").and_then(|v| v.try_to::<bool>().ok()).unwrap_or(false) {
            Some(cartalith_engine::region_export::TileVisual {
                sea: self.sea_level,
                sun_az_deg: get_num("sun_az_deg").unwrap_or(315.0),
                exag: get_num("exag").unwrap_or(3.4),
            })
        } else {
            None
        };
        let amplify = cartalith_terrain::amplify::AmplifyOpts {
            seed: self.seed,
            detail_freq: get_num("detail_freq").unwrap_or(1.0),
            detail_amp: get_num("detail_amp").unwrap_or(0.14),
            sea: self.sea_level,
            ridged: opts.get("ridged").and_then(|v| v.try_to::<bool>().ok()).unwrap_or(false),
            // `z_base`/`zoom_detail_k` steer `add_zoom_detail`, which only the
            // pyramid bake runs; a region export has no pyramid level, so the
            // reference's own defaults are the only meaningful values here.
            ..cartalith_terrain::amplify::AmplifyOpts::default()
        };
        let export_opts = cartalith_engine::region_export::RegionExportOpts {
            cols, rows, tile_size, amplify: &amplify, world: self.world, version: &version, gzip, visual,
        };
        let export = cartalith_engine::region_export::export_region_tiles(field, self.gw as usize, self.gh as usize, &region, &export_opts);
        match cartalith_engine::region_export::zip_region_export(&export, None) {
            Ok(bytes) => PackedByteArray::from(bytes),
            Err(e) => {
                godot_print!("cartalith-godot: region_export_tiles zip failed: {e}");
                PackedByteArray::new()
            }
        }
    }

    // =======================================================================
    // Bake / tile pyramid / persistent atlas / finalize
    // (`GUI_GAP_REGISTER.md` WW-01, PR-10/S4, PR-12, S5, SH-07)
    // =======================================================================

    /// Where the atlas lives — a **real OS directory**, which the shell gets
    /// from `ProjectSettings.globalize_path("user://atlas")`.
    ///
    /// Called once at startup. Until it is, every atlas operation below
    /// reports "no cache directory set" rather than writing somewhere the user
    /// did not choose. Creates the directory if it does not exist; `false`
    /// (with a console line) if it cannot.
    #[func]
    fn atlas_set_root(&mut self, path: GString) -> bool {
        let p = std::path::PathBuf::from(path.to_string());
        if let Err(e) = std::fs::create_dir_all(&p) {
            godot_print!("cartalith-godot: atlas_set_root({p:?}) failed: {e}");
            return false;
        }
        self.bake.root = Some(p);
        true
    }

    /// `worldKey()` (reference line 10703) — the current world's atlas
    /// namespace, as lower-case hex.
    ///
    /// Empty before the first `generate()`/`load_save()`. **Changing any
    /// generation parameter changes this**, which is the whole
    /// cache-invalidation mechanism: see `bake_bridge.rs`'s own module doc for
    /// exactly which parameters are in the hash and which are deliberately
    /// out.
    #[func]
    fn atlas_world_key(&self) -> GString {
        GString::from(self.world_key().as_str())
    }

    /// The reference's `_lodTile` (default 1024) — the pixel size a baked tile
    /// is synthesised at. Part of the chunk key, so two tile sizes over one
    /// world are two coexisting bakes rather than one invalidating the other.
    ///
    /// Clamped to `[64, 4096]`: below 64 the pyramid is more overhead than
    /// data, and above 4096 a single level-6 bake would allocate more than any
    /// target device has.
    #[func]
    fn atlas_set_tile_size(&mut self, px: i64) {
        self.bake.tile_size = px.clamp(64, 4096) as usize;
    }

    #[func]
    fn atlas_tile_size(&self) -> i64 {
        self.bake.tile_size as i64
    }

    /// `updateAtlasStatus()` (reference line 10748) plus the two numbers it
    /// does not report — `GUI_GAP_REGISTER.md` SH-07's `atlas` status slot.
    ///
    /// Keys: `chunks`, `bytes`, `bytes_text`, `deepest_level` (`-1` for an
    /// empty atlas), `text`, `finalized`, `tile_size`, `world_key`, `root`.
    #[func]
    fn atlas_status(&self) -> VarDictionary {
        let wk = self.world_key();
        let st = bake_bridge::atlas_status(&self.bake, &wk);
        let mut d = VarDictionary::new();
        d.set("chunks", st.chunks as i64);
        d.set("bytes", st.bytes as i64);
        d.set("bytes_text", &GString::from(bake_bridge::human_bytes(st.bytes).as_str()));
        d.set("deepest_level", st.deepest_level as i64);
        d.set("text", &GString::from(st.text.as_str()));
        d.set("finalized", self.bake.finalized);
        d.set("tile_size", self.bake.tile_size as i64);
        d.set("world_key", &GString::from(wk.as_str()));
        d.set(
            "root",
            &GString::from(
                self.bake
                    .root
                    .as_ref()
                    .map(|p| p.display().to_string())
                    .unwrap_or_default()
                    .as_str(),
            ),
        );
        d
    }

    /// What a bake of this depth would cost, **before** committing to it —
    /// `GUI_GAP_REGISTER.md` WW-01's *"Bake depth"* row.
    ///
    /// Keys: `tiles`, `already_baked`, `remaining`, `seconds`, `bytes`,
    /// `bytes_text`, `tile_w`, `tile_h`.
    ///
    /// **`bytes` is the one to show.** A depth-3 bake of a 2048×1311 world at
    /// 1024 px tiles is 234 MiB, measured; depth 5 at the same settings is
    /// about 3.7 GiB. `seconds` is deliberately crude by comparison — see
    /// `bake_bridge::bake_estimate` for how each is arrived at.
    #[func]
    fn bake_estimate(&self, max_z: i64) -> VarDictionary {
        // 19 ms/tile, measured at 1024 px on a 2048x1311 world
        // (`bake_real_world.rs`). One number for one machine and one tile
        // size, which is why the doc above calls the seconds figure crude.
        let est = bake_bridge::bake_estimate(
            max_z as i32,
            self.gw.max(0) as usize,
            self.gh.max(0) as usize,
            self.bake.tile_size,
            19.0,
        );
        let (tiles, secs) = (est.tiles, est.seconds);
        let wk = self.world_key();
        let done = self
            .bake
            .store()
            .and_then(|s| s.keys_for_world(&wk).ok())
            .map(|k| {
                k.iter()
                    .filter(|x| x.ts == self.bake.tile_size && (x.id.z as i64) <= max_z)
                    .count()
            })
            .unwrap_or(0);
        let mut d = VarDictionary::new();
        d.set("tiles", tiles as i64);
        d.set("already_baked", done as i64);
        d.set("remaining", (tiles as i64 - done as i64).max(0));
        d.set("seconds", secs);
        d.set("bytes", est.total_bytes as i64);
        d.set("bytes_text", &GString::from(bake_bridge::human_bytes(est.total_bytes).as_str()));
        d.set("tile_w", est.tile_w as i64);
        d.set("tile_h", est.tile_h as i64);
        d
    }

    /// `bakeAllTiles(maxZ)` (reference line 10809) — bake every tile of every
    /// level `0..=max_z` into the persistent atlas.
    ///
    /// **This is the expensive one.** Depth 3 is 85 tiles, depth 4 is 341,
    /// depth 5 is 1365 — the reference's own numbers, and
    /// `bake_estimate(max_z)` reports them before the user commits. Already-
    /// baked chunks are skipped, so re-running after a partial bake only fills
    /// the gaps.
    ///
    /// Synchronous. The reference yields to the browser event loop between
    /// tiles; this runs the whole depth in one call (parallel across the tiles
    /// of each level, `rayon`), so the shell must show a busy state around it.
    /// A progress *signal* would need the call to be threaded, which is a
    /// separate piece of work — `GENERATION_PIPELINE_ARCHITECTURE_RESEARCH.md`
    /// covers the same question for `generate()` and it is not solved there
    /// either.
    ///
    /// Keys: `ok`, `baked`, `skipped`, `failed`, `total`, `seconds`, `error`.
    #[func]
    fn bake_all(&mut self, max_z: i64) -> VarDictionary {
        let ids: Vec<cartalith_spatial::pyramid::ChunkId> = Vec::new();
        self.run_bake(max_z.clamp(0, bake_bridge::MAX_BAKE_DEPTH as i64) as i32, &ids)
    }

    /// `bakeVisibleTiles()` (reference line 10765) — bake just the tiles a
    /// view rectangle touches, at one level.
    ///
    /// `x0`/`y0`/`x1`/`y1` are in **coarse grid cells**, which is what
    /// `viewport_host.gd` already computes for its own deep-zoom compositor.
    /// The camera stays in GDScript, where the camera is.
    #[func]
    fn bake_visible(&mut self, z: i64, x0: f64, y0: f64, x1: f64, y1: f64) -> VarDictionary {
        let (gw, gh) = (self.gw as usize, self.gh as usize);
        if gw == 0 || gh == 0 {
            return bake_error("no world generated yet");
        }
        let t = cartalith_spatial::pyramid::tiles_in_view(z as i32, x0, y0, x1, y1, gw, gh);
        let ids: Vec<cartalith_spatial::pyramid::ChunkId> = (t.r0..=t.r1)
            .flat_map(|r| (t.c0..=t.c1).map(move |c| cartalith_spatial::pyramid::ChunkId::new(z.max(0) as u32, c, r)))
            .collect();
        self.run_bake(-1, &ids)
    }

    /// `atlasClearWorld(wk)` (reference line 10738) — throw away this world's
    /// baked chunks. `GUI_GAP_REGISTER.md` PR-12's *Memory ▸ Clear caches…*.
    ///
    /// Returns how many chunks were removed. **Clears the finalize flag too**:
    /// a finalized world with no atlas is a lock protecting nothing, and
    /// leaving it set would strand the user in a read-only world they cannot
    /// explain.
    #[func]
    fn atlas_clear(&mut self) -> i64 {
        let wk = self.world_key();
        let Some(store) = self.bake.store() else { return 0 };
        match store.clear_world(&wk) {
            Ok(n) => {
                if n > 0 {
                    self.bake.finalized = false;
                }
                n as i64
            }
            Err(e) => {
                godot_print!("cartalith-godot: atlas_clear failed: {e}");
                0
            }
        }
    }

    /// `atlasExportEntries` + `zipStore` (reference lines 10890/12009) — this
    /// world's whole baked atlas as a portable `World/` archive.
    ///
    /// Empty `PackedByteArray` when nothing is baked, which is not an error.
    /// `gzip` compresses each chunk's `rg16`; the PNGs are stored either way
    /// (already DEFLATE'd internally, so re-compressing them is wasted CPU —
    /// the reference's own `zipStore` note).
    #[func]
    fn atlas_export_zip(&self, gzip: bool) -> PackedByteArray {
        let Some(store) = self.bake.store() else { return PackedByteArray::new() };
        let wk = self.world_key();
        let params = self.source.as_ref().map(|_| params::save_state(&self.params));
        let Some((entries, _man)) = cartalith_engine::bake::atlas_export_entries(
            &store,
            &wk,
            self.gw as usize,
            self.gh as usize,
            env!("CARGO_PKG_VERSION"),
            0,
            params,
            gzip,
        ) else {
            return PackedByteArray::new();
        };
        let refs: Vec<(&str, &[u8])> =
            entries.iter().map(|e| (e.name.as_str(), e.data.as_slice())).collect();
        match cartalith_assets::zip_store_bytes(&refs) {
            Ok(b) => PackedByteArray::from(b),
            Err(e) => {
                godot_print!("cartalith-godot: atlas_export_zip failed: {e}");
                PackedByteArray::new()
            }
        }
    }

    /// `atlasImportEntries(zip)` (reference line 10910) — read a portable
    /// `World/` archive into the store.
    ///
    /// The archive is filed under **its own** world key, not this session's:
    /// an atlas describes the world it was baked from, and serving it as this
    /// world's terrain would be silently wrong. `matches_current` says whether
    /// the two happen to agree, which is what the shell needs to decide
    /// between "loaded and live" and "loaded, but for a different world".
    ///
    /// Keys: `ok`, `chunks`, `world_key`, `matches_current`, `error`.
    #[func]
    fn atlas_import_zip(&mut self, bytes: PackedByteArray) -> VarDictionary {
        let Some(store) = self.bake.store() else { return bake_error("no atlas root set") };
        let raw = bytes.to_vec();
        let entries =
            match cartalith_assets::archive::read_pack_entries(std::io::Cursor::new(&raw)) {
                Ok(e) => e,
                Err(e) => return bake_error(&format!("unreadable archive: {e}")),
            };
        let lookup = |name: &str| entries.get(name).cloned();
        match cartalith_engine::bake::atlas_import_entries(&store, &lookup) {
            Ok((n, wk)) => {
                let mut d = VarDictionary::new();
                d.set("ok", true);
                d.set("chunks", n as i64);
                d.set("world_key", &GString::from(wk.as_str()));
                d.set("matches_current", wk == self.world_key());
                d.set("error", &GString::new());
                d
            }
            Err(e) => bake_error(&e.to_string()),
        }
    }

    /// One baked chunk's stored visual, as PNG bytes — the read side of the
    /// cache, and the thing that makes a bake worth doing.
    ///
    /// Empty when the chunk was never baked, when the bake stored height only
    /// (`visual: false`), or before any atlas root is set. A caller that gets
    /// nothing back falls through to live synthesis, exactly as the reference's
    /// `atlasLoadImg` (line 10752) does.
    #[func]
    fn atlas_tile_png(&self, z: i64, col: i64, row: i64) -> PackedByteArray {
        let Some(store) = self.bake.store() else { return PackedByteArray::new() };
        if z < 0 || col < 0 || row < 0 {
            return PackedByteArray::new();
        }
        let id = cartalith_spatial::pyramid::ChunkId::new(z as u32, col as u32, row as u32);
        match store.get(&self.world_key(), self.bake.tile_size, id) {
            Ok(Some(c)) => c.png.map(PackedByteArray::from).unwrap_or_default(),
            _ => PackedByteArray::new(),
        }
    }

    /// Is a chunk (or any ancestor of it) baked? — `bakedCover` (reference
    /// line 10715), the rule that stops the viewer refining beneath a baked
    /// tile and stops the editor composing an edit into one.
    #[func]
    fn atlas_is_covered(&self, z: i64, col: i64, row: i64) -> bool {
        let Some(store) = self.bake.store() else { return false };
        if z < 0 || col < 0 || row < 0 {
            return false;
        }
        let Ok(keys) = store.keys_for_world(&self.world_key()) else { return false };
        cartalith_engine::bake::chunk_is_covered(
            &keys,
            self.bake.tile_size,
            cartalith_spatial::pyramid::ChunkId::new(z as u32, col as u32, row as u32),
        )
    }

    /// `setFinalized(on)` (reference line 10872).
    ///
    /// Finalizing **requires a non-empty atlas**: the lock exists to protect
    /// baked work, and setting it with nothing baked would be a read-only
    /// world protecting nothing. Un-finalizing always succeeds — it is the
    /// escape hatch, and the reference's own v0.66 bug was letting the blanket
    /// disable reach the Un-finalize button itself.
    ///
    /// Returns whether the flag now holds the requested value.
    #[func]
    fn set_finalized(&mut self, on: bool) -> bool {
        if !on {
            self.bake.finalized = false;
            return true;
        }
        let wk = self.world_key();
        let n = self
            .bake
            .store()
            .and_then(|s| s.keys_for_world(&wk).ok())
            .map(|k| k.len())
            .unwrap_or(0);
        if n == 0 {
            godot_print!(
                "cartalith-godot: refusing to finalize -- nothing is baked for world {wk}"
            );
            return false;
        }
        self.bake.finalized = true;
        true
    }

    #[func]
    fn is_finalized(&self) -> bool {
        self.bake.finalized
    }

    /// May this kind of change proceed? — `applyFinalizedUI`'s rule
    /// (reference line 10854) as a query, so GDScript can grey a control and
    /// explain *why* with the same one sentence the engine would refuse with.
    ///
    /// `kind` is `"generation"`, `"height_edit"` or `"presentation"`. Returns
    /// the empty string when allowed, or the message to show when not. An
    /// unrecognised kind is treated as `"generation"` — the conservative
    /// reading, since a caller naming something this port does not know about
    /// is more likely to be mutating the world than styling it.
    #[func]
    fn finalize_check(&self, kind: GString) -> GString {
        let m = match kind.to_string().as_str() {
            "presentation" => cartalith_engine::bake::Mutation::Presentation,
            "height_edit" => cartalith_engine::bake::Mutation::HeightEdit,
            _ => cartalith_engine::bake::Mutation::Generation,
        };
        GString::from(self.bake.check(m).err().unwrap_or(""))
    }
}

/// A failed bake/atlas call, in the shape every one of them returns.
fn bake_error(msg: &str) -> VarDictionary {
    let mut d = VarDictionary::new();
    d.set("ok", false);
    d.set("baked", 0i64);
    d.set("skipped", 0i64);
    d.set("failed", 0i64);
    d.set("total", 0i64);
    d.set("chunks", 0i64);
    d.set("seconds", 0.0f64);
    d.set("error", &GString::from(msg));
    d
}

/// The non-`#[func]` half of the bake bridge.
impl WorldGen {
    /// `worldKey()`'s input and hash in one — see `bake_bridge.rs`'s module
    /// doc for what is in the signature and what is deliberately left out.
    ///
    /// Empty before the first `generate()`/`load_save()`: a world with no
    /// dimensions has no tiles to bake, and returning a hash of all-zeroes
    /// would let two different empty sessions share an atlas namespace.
    fn world_key(&self) -> String {
        if self.gw <= 0 || self.gh <= 0 {
            return String::new();
        }
        cartalith_io::world_key(&bake_bridge::world_key_signature(
            self.gw,
            self.gh,
            self.seed,
            self.map_width_km,
            self.sea_level,
            self.world,
            params::save_state(&self.params),
        ))
    }

    /// The body both `bake_all` and `bake_visible` share. `max_z >= 0` means
    /// "the whole pyramid to this depth"; otherwise `ids` names the tiles.
    ///
    /// One implementation rather than two, deliberately: two bake loops that
    /// could disagree about what a stored chunk contains would be the bug this
    /// system is least able to detect from the outside.
    fn run_bake(&mut self, max_z: i32, ids: &[cartalith_spatial::pyramid::ChunkId]) -> VarDictionary {
        let Some(store) = self.bake.store() else { return bake_error("no atlas root set") };
        let (gw, gh) = (self.gw as usize, self.gh as usize);
        let field: Vec<f32> = match self.source.as_ref() {
            Some(WorldSource::Generated(ws)) => ws.field.clone(),
            Some(WorldSource::Loaded(save)) => save.fields.heightmap.clone(),
            None => return bake_error("no world generated yet"),
        };
        if gw == 0 || gh == 0 || field.len() < gw * gh {
            return bake_error("no world generated yet");
        }
        let wk = self.world_key();
        // The tile visual uses this world's *current* appearance sun/exag, so
        // a baked tile and a live one shade under the same light. `sea` comes
        // from the world, never a caller guess -- `region_export_tiles`'s own
        // convention.
        let app = self.appearance();
        let visual = Some(cartalith_engine::region_export::TileVisual {
            sea: self.sea_level,
            sun_az_deg: app.sun_az_deg,
            exag: app.exag,
        });
        let amplify = cartalith_terrain::amplify::AmplifyOpts {
            seed: self.seed,
            sea: self.sea_level,
            ..cartalith_terrain::amplify::AmplifyOpts::default()
        };
        let o = cartalith_engine::bake::BakeOpts {
            world_key: &wk,
            tile_size: self.bake.tile_size,
            amplify: &amplify,
            visual,
            version: env!("CARGO_PKG_VERSION"),
        };
        let t0 = std::time::Instant::now();
        let report = if max_z >= 0 {
            cartalith_engine::bake::bake_all_tiles(&field, gw, gh, max_z, &store, &o, |_, _| {})
        } else {
            cartalith_engine::bake::bake_tiles(&field, gw, gh, ids, &store, &o, |_, _| {})
        };
        let secs = t0.elapsed().as_secs_f64();
        godot_print!(
            "cartalith-godot: bake {} baked, {} skipped, {} failed in {:.2}s (world {wk}, tile {}px)",
            report.baked, report.skipped, report.failed, secs, self.bake.tile_size
        );
        let mut d = VarDictionary::new();
        d.set("ok", report.failed == 0);
        d.set("baked", report.baked as i64);
        d.set("skipped", report.skipped as i64);
        d.set("failed", report.failed as i64);
        d.set("total", report.total() as i64);
        d.set("seconds", secs);
        let err = if report.failed > 0 {
            format!("{} chunk(s) could not be written", report.failed)
        } else {
            String::new()
        };
        d.set("error", &GString::from(err.as_str()));
        d
    }
}

/// A bare `String` out of a `Variant`, for the Label bridge's text/font/
/// color/size_mode setters -- explicit `get_type()` check rather than
/// `try_to::<GString>()`'s own fallibility, matching `variant_to_value`'s
/// own reasoning: Godot's `Variant` can stringify almost anything (a bool,
/// an int, ...), so a naive fallible conversion would silently accept a
/// wrong-typed value instead of reporting it `rejected`.
fn variant_to_string(v: &Variant) -> Option<String> {
    match v.get_type() {
        VariantType::STRING => Some(v.to::<GString>().to_string()),
        _ => None,
    }
}

/// One `MapLabel`'s fields as a flat `Dictionary` -- shared by `label_get`
/// and `label_list`, which both need exactly this shape (`label_list` adds
/// its own `index`/`selected` on top). `font`/`color` are always the
/// *effective* value (`font_or_default`/`color_or_default`), never an
/// empty string standing in for "unset".
fn label_dict(lb: &cartalith_civ::labels::MapLabel) -> VarDictionary {
    let size_mode = match lb.size_mode {
        cartalith_civ::labels::LabelSizeMode::Fixed => "fixed",
        cartalith_civ::labels::LabelSizeMode::Zoom => "zoom",
    };
    vdict! {
        "x" => lb.x,
        "y" => lb.y,
        "text" => lb.name.as_str(),
        "angle" => lb.angle,
        "arc" => lb.arc,
        "size" => lb.size,
        "size_mode" => size_mode,
        "font" => lb.font_or_default(),
        "color" => lb.color_or_default(),
    }
}

/// One `HandleCircle` as `{"x":.., "y":.., "r":..}`, or an empty
/// `Dictionary` for `None` -- `label_handles`'s own per-slot shape.
fn handle_circle_dict(c: Option<cartalith_civ::labels::HandleCircle>) -> VarDictionary {
    c.map_or_else(VarDictionary::new, |c| vdict! { "x" => c.x, "y" => c.y, "r" => c.r })
}

/// `UNIFIED_TOOL_PLAN.md` milestone F, the CARTO domain's Label tool
/// (`DCC_SHELL_SPEC.md` §4.5.5: "Click an empty spot creates a label;
/// click an existing one edits it in place"). Thin `Variant`<->Rust
/// conversion over `label_bridge::LabelBridge` -- see that module's own
/// doc comment for the edit-session semantics (position commits
/// immediately via `label_move`; the seven style fields are snapshot-and-
/// revertible via `label_select`/`label_confirm_edit`/`label_cancel_edit`),
/// the handle-geometry port (`label_bridge::handle_circles`, new to this
/// milestone -- `labels.rs` itself never had a home for it) and the text-
/// measurement gap this binding cannot close on its own (`label_hit_test`/
/// `label_handles` use a disclosed `meas_w = 0` placeholder;
/// `label_glyph_layout` is the one call that instead requires the caller's
/// own real measured widths, for the reasons that module's doc explains).
#[godot_api(secondary)]
impl WorldGen {
    /// New label at grid cell `(gx, gy)` with the given text, selected
    /// immediately (`_civSelectLabel`, reference line 9771's click-on-
    /// empty-ground branch). Returns the new label's index, or `-1` before
    /// any `generate()` call.
    #[func]
    fn label_create(&mut self, gx: f64, gy: f64, text: GString) -> i64 {
        let Some(labels) = self.labels.as_mut() else { return -1 };
        labels.create(gx, gy, text.to_string()) as i64
    }

    /// `_civLabelDrag`'s per-move assignment (reference line 9718) -- sets
    /// position directly, unclamped, with **no** selection side effect
    /// (the reference calls this on every drag sample and only selects
    /// once, on release -- call `label_select` separately once a drag
    /// ends, matching `pointerup`'s own `if(moved){ _civSelectLabel(lb);
    /// }`). `false` for an out-of-range `index`, a non-finite `gx`/`gy`,
    /// or before any `generate()` call.
    #[func]
    fn label_move(&mut self, index: i64, gx: f64, gy: f64) -> bool {
        let Ok(i) = usize::try_from(index) else { return false };
        let Some(labels) = self.labels.as_mut() else { return false };
        labels.move_to(i, gx, gy)
    }

    /// Selects label `index` for editing, snapshotting its seven style
    /// fields (`LabelEditSession::select`) -- re-selecting an already-
    /// selected label does **not** retake the snapshot, so a later
    /// `label_cancel_edit` always reverts to how it looked when the
    /// session *started*, not just the most recent tweak. Pass a negative
    /// index to deselect (matching `sculpt_select_stamp`'s own
    /// convention). `false` for an out-of-range non-negative index or
    /// before any `generate()` call.
    #[func]
    fn label_select(&mut self, index: i64) -> bool {
        let Some(labels) = self.labels.as_mut() else { return false };
        if index < 0 {
            labels.session.select(&labels.labels, None);
            return true;
        }
        let Ok(i) = usize::try_from(index) else { return false };
        if i >= labels.labels.len() {
            return false;
        }
        labels.session.select(&labels.labels, Some(i));
        true
    }

    /// The currently selected label's index, or `-1` for none (or before
    /// any `generate()` call).
    #[func]
    fn label_get_selected(&self) -> i64 {
        self.labels.as_ref().and_then(|l| l.session.selected()).map_or(-1, |i| i as i64)
    }

    /// `_civConfirmLabel()` -- ends the edit session, keeping whatever
    /// edits were made. A no-op with nothing selected or before any
    /// `generate()` call.
    #[func]
    fn label_confirm_edit(&mut self) {
        if let Some(labels) = self.labels.as_mut() {
            labels.session.confirm();
        }
    }

    /// `_civCancelLabel()` -- reverts the selected label's seven style
    /// fields to how they looked when the session started (**not** its
    /// position -- `label_bridge.rs`'s own doc comment on why `x`/`y` are
    /// excluded from the snapshot) and ends the session. Returns whether
    /// anything was actually reverted. `false` with nothing selected or
    /// before any `generate()` call.
    #[func]
    fn label_cancel_edit(&mut self) -> bool {
        let Some(labels) = self.labels.as_mut() else { return false };
        labels.session.cancel(&mut labels.labels)
    }

    /// One label's full state (`DCC_SHELL_SPEC.md` §4.5.5's right dock):
    /// `x`, `y`, `text`, `angle` (degrees), `arc` (`[-1,1]`), `size`,
    /// `size_mode` (`"fixed"`/`"zoom"`), `font`, `color` (the last two are
    /// always the *effective* value, never an empty string for "unset").
    /// Empty `Dictionary` for an out-of-range `index` or before any
    /// `generate()` call.
    #[func]
    fn label_get(&self, index: i64) -> VarDictionary {
        let Ok(i) = usize::try_from(index) else { return VarDictionary::new() };
        let Some(lb) = self.labels.as_ref().and_then(|l| l.labels.get(i)) else { return VarDictionary::new() };
        label_dict(lb)
    }

    /// Every label, in storage order (`DCC_SHELL_SPEC.md` §12's
    /// `#carLabelList`), each the same shape `label_get` returns plus its
    /// own `index` and `selected` (bool). Empty before any `generate()`
    /// call or while no labels are placed.
    #[func]
    fn label_list(&self) -> Array<VarDictionary> {
        let Some(labels) = self.labels.as_ref() else { return Array::new() };
        let selected = labels.session.selected();
        labels
            .labels
            .iter()
            .enumerate()
            .map(|(i, lb)| {
                let mut d = label_dict(lb);
                d.set("index", i as i64);
                d.set("selected", selected == Some(i));
                d
            })
            .collect()
    }

    /// Applies a partial `Dictionary` of style fields: `text` (String),
    /// `size` (float, clamped `[8,48]`), `size_mode` (String, `"fixed"`/
    /// `"zoom"`), `arc` (float, clamped `[-1,1]`), `angle` (float,
    /// degrees, unrestricted), `font`/`color` (String, empty resets to the
    /// engine default). Position (`x`/`y`) is not settable here -- see
    /// `label_move`.
    ///
    /// **Not modelled**: `DCC_SHELL_SPEC.md` §4.5.5's tool-options row also
    /// lists "letter-spacing" and "anchor", and calls `font` a "font role".
    /// `cartalith_civ::labels::MapLabel` has no letter-spacing or anchor
    /// field at all -- the reference itself has neither (arc placement's
    /// per-glyph spacing comes entirely from measured widths, not a
    /// separate spacing knob, and every label anchors at its own stored
    /// `(x, y)`, full stop) -- and `font` is the literal CSS font string
    /// the reference stores, not a named-role vocabulary a UI dropdown
    /// might present. Sending `"letter_spacing"`/`"anchor"` (or any other
    /// unrecognised key) is reported `rejected`, same as any other unknown
    /// key, per this codebase's "a typo'd key is a bug worth seeing"
    /// policy (`set_params`'s own doc comment) -- resolving a role name to
    /// a font string, if that's ever wanted, is a shell-side lookup this
    /// binding has no reason to own; sending the literal string via
    /// `"font"` already works today.
    ///
    /// Returns `{"ok": bool, "rejected": PackedStringArray, "clamped":
    /// PackedStringArray}`. `ok` is `false` (both arrays empty) for an
    /// out-of-range `index` or before any `generate()` call, matching
    /// `icon_delete`'s own bool-for-validity convention; when `ok` is
    /// `true`, `rejected`/`clamped` report per-key outcomes the same way
    /// `set_params`/`sculpt_set_globals` already do for their own multi-key
    /// dictionaries.
    #[func]
    fn label_set(&mut self, index: i64, values: VarDictionary) -> VarDictionary {
        let empty =
            || dict! { "ok" => false, "rejected" => &PackedStringArray::new(), "clamped" => &PackedStringArray::new() };
        let Ok(i) = usize::try_from(index) else { return empty() };
        let Some(labels) = self.labels.as_mut() else { return empty() };
        let Some(lb) = labels.labels.get_mut(i) else { return empty() };

        let mut rejected = PackedStringArray::new();
        let mut clamped = PackedStringArray::new();
        for (k, v) in values.iter_shared() {
            let key = k.to_string();
            let outcome = match key.as_str() {
                "text" => variant_to_string(&v).map_or(label_bridge::Outcome::Rejected, |s| label_bridge::set_text(lb, s)),
                "font" => variant_to_string(&v).map_or(label_bridge::Outcome::Rejected, |s| label_bridge::set_font(lb, s)),
                "color" => variant_to_string(&v).map_or(label_bridge::Outcome::Rejected, |s| label_bridge::set_color(lb, s)),
                "size_mode" => variant_to_string(&v)
                    .map_or(label_bridge::Outcome::Rejected, |s| label_bridge::set_size_mode(lb, &s)),
                "size" => variant_to_num(&v).map_or(label_bridge::Outcome::Rejected, |n| label_bridge::set_size(lb, n)),
                "arc" => variant_to_num(&v).map_or(label_bridge::Outcome::Rejected, |n| label_bridge::set_arc(lb, n)),
                "angle" => variant_to_num(&v).map_or(label_bridge::Outcome::Rejected, |n| label_bridge::set_angle(lb, n)),
                _ => label_bridge::Outcome::Rejected,
            };
            match outcome {
                label_bridge::Outcome::Applied => {}
                label_bridge::Outcome::Clamped => clamped.push(&GString::from(&key)),
                label_bridge::Outcome::Rejected => rejected.push(&GString::from(&key)),
            }
        }
        dict! { "ok" => true, "rejected" => &rejected, "clamped" => &clamped }
    }

    /// Removes a label. Clearing/keeping the session follows
    /// `label_bridge::LabelBridge::delete`'s own doc comment (any delete
    /// at or before the current selection clears the session, rather than
    /// risk mis-pointing a live revert snapshot at the wrong label).
    /// `false` for an out-of-range `index` or before any `generate()`
    /// call.
    #[func]
    fn label_delete(&mut self, index: i64) -> bool {
        let Ok(i) = usize::try_from(index) else { return false };
        let Some(labels) = self.labels.as_mut() else { return false };
        labels.delete(i)
    }

    /// Drops every label and ends any edit session. A no-op before any
    /// `generate()` call.
    #[func]
    fn label_clear_all(&mut self) {
        if let Some(labels) = self.labels.as_mut() {
            labels.clear_all();
        }
    }

    /// Box-only hit test against every placed label
    /// (`label_bridge::LabelBridge::hit_test`), selecting the hit label.
    /// `(gx, gy)` are grid coordinates, matching `label_create`'s/
    /// `label_move`'s own convention, not screen pixels. `-1` on a miss or
    /// before any `generate()` call.
    ///
    /// **Box hits only** -- matching `icon_hit_test`'s own "no on-canvas
    /// resize-handle geometry yet" scope, one step further: a *handle* hit
    /// is the shell's own job here, by comparing the pointer against the
    /// circles `label_handles` already returns for whichever label is
    /// selected (those need no separate hit-test call from this side -- a
    /// handle is only interactive while its owning label is selected and
    /// visible, which the shell already knows without asking Rust). Uses a
    /// placeholder text width of `0` for every label's box -- see
    /// `label_bridge.rs`'s own "text measurement" section; the box narrows
    /// to a font-height square rather than the label's true rendered width
    /// until a live `Font` is threaded through.
    #[func]
    fn label_hit_test(&mut self, gx: f64, gy: f64) -> i64 {
        let grid_w = self.gw as usize;
        let Some(labels) = self.labels.as_mut() else { return -1 };
        let env = cartalith_civ::labels::LabelViewEnv { grid_w, zoom_scale: 1.0, icon_scale: 1.0 };
        labels.hit_test(gx, gy, &env).map_or(-1, |i| i as i64)
    }

    /// The five on-canvas manipulation-box handle circles for label
    /// `index`'s current box (`label_bridge::handle_circles` --
    /// resize/rotate/arc/check/cross), each `{"x":.., "y":.., "r":..}` in
    /// the same coordinate space `label_hit_test`'s `(gx, gy)` and
    /// `label_resize_size`/`label_rotate_deg`/`label_arc_value`'s own
    /// `cx`/`cy`/`gx`/`gy` already live in -- a slot is an empty
    /// `Dictionary` only if `index` itself is invalid (all five are always
    /// present together otherwise). `zoom` is the raw view scale before
    /// `_civZoomK`'s own clamp (`civ_zoom_k` applies that internally),
    /// matching the reference's `viewT.scale`.
    ///
    /// Empty top-level `Dictionary` for an out-of-range `index` or before
    /// any `generate()` call. Uses the same `meas_w = 0` placeholder
    /// `label_hit_test` does (`label_bridge.rs`'s "text measurement"
    /// section) -- the handles are still correctly positioned relative to
    /// *that* box, just not the label's true rendered width.
    #[func]
    fn label_handles(&self, index: i64, zoom: f64) -> VarDictionary {
        let Ok(i) = usize::try_from(index) else { return VarDictionary::new() };
        let Some(labels) = self.labels.as_ref() else { return VarDictionary::new() };
        let env = cartalith_civ::labels::LabelViewEnv { grid_w: self.gw as usize, zoom_scale: zoom, icon_scale: 1.0 };
        let Some(h) = labels.handles(i, &env) else { return VarDictionary::new() };
        vdict! {
            "resize" => &handle_circle_dict(h.resize),
            "rotate" => &handle_circle_dict(h.rotate),
            "arc" => &handle_circle_dict(h.arc),
            "check" => &handle_circle_dict(h.check),
            "cross" => &handle_circle_dict(h.cross),
        }
    }

    /// Per-glyph arc placement for label `index`'s current text
    /// (`arc_label_layout`, via `label_bridge::LabelBridge::glyph_layout`)
    /// -- **the one Label call that needs real measured text, not a
    /// placeholder** (`label_bridge.rs`'s own "text measurement" section
    /// explains why: arc placement is fundamentally about per-glyph
    /// spacing, so a zero-width placeholder would collapse every glyph
    /// onto the label's own origin rather than merely under-sizing a box).
    /// `zoom` feeds this label's own font size exactly like
    /// `label_handles`' own parameter does.
    ///
    /// The caller measures `label_get(index).text` with a live Godot
    /// `Font` and supplies:
    /// - `char_widths` -- one entry per **Unicode scalar** of the text, in
    ///   order (the reference measures one `char` at a time in its own
    ///   layout loop);
    /// - `total_w` -- the width of measuring the **whole** string in one
    ///   call. **Not** the sum of `char_widths`: a kerned font measures a
    ///   string narrower than its glyphs' own advances added up, and
    ///   `arc_label_layout` reads each for a different purpose
    ///   (`labels.rs`'s own emphasis: a port that summed the per-char
    ///   widths instead drifts on any kerned string).
    ///
    /// Returns one `Dictionary` per glyph, each `{"dx": float, "dy":
    /// float, "rot": float (radians), "straight": bool}` in the label's
    /// own frame, *before* the whole-label `angle` rotation (exactly
    /// `ArcGlyph`'s own contract) -- draw each character translated by
    /// `(dx, dy)` then rotated by `rot`. When `|arc| < 0.01`, this is a
    /// **single**-entry array with `dx = dy = rot = 0.0` and `straight =
    /// true`: draw the whole string once at the label's own origin, not
    /// per-glyph (every other entry this method ever returns has
    /// `straight = false`). Empty `Array` for an out-of-range `index` or
    /// before any `generate()` call.
    #[func]
    fn label_glyph_layout(&self, index: i64, zoom: f64, char_widths: PackedFloat64Array, total_w: f64) -> Array<VarDictionary> {
        let Ok(i) = usize::try_from(index) else { return Array::new() };
        let Some(labels) = self.labels.as_ref() else { return Array::new() };
        let env = cartalith_civ::labels::LabelViewEnv { grid_w: self.gw as usize, zoom_scale: zoom, icon_scale: 1.0 };
        let Some(layout) = labels.glyph_layout(i, &env, char_widths.as_slice(), total_w) else { return Array::new() };
        match layout {
            cartalith_civ::labels::ArcLayout::Straight => {
                std::iter::once(vdict! { "dx" => 0.0, "dy" => 0.0, "rot" => 0.0, "straight" => true }).collect()
            }
            cartalith_civ::labels::ArcLayout::Arc(glyphs) => glyphs
                .into_iter()
                .map(|g| vdict! { "dx" => g.dx, "dy" => g.dy, "rot" => g.rot, "straight" => false })
                .collect(),
        }
    }

    /// The resize handle's live value during a drag
    /// (`cartalith_civ::labels::label_resize_size`) -- pure per-call math,
    /// no session kept on this side; the caller holds `start_size`/`cx`/
    /// `cy`/`start_dist` locally between pointer events (captured once at
    /// grab time from `label_get`/`label_handles`), the same way
    /// `icon_resize`'s own caller already must. Clamped to `[8, 48]`
    /// internally -- feed the result straight into
    /// `label_set({"size": ...})`.
    #[func]
    fn label_resize_size(&self, start_size: f64, cx: f64, cy: f64, gx: f64, gy: f64, start_dist: f64) -> f64 {
        label_bridge::resize_size(start_size, cx, cy, gx, gy, start_dist)
    }

    /// The rotate handle's live value during a drag
    /// (`cartalith_civ::labels::label_rotate_deg`) -- **absolute**, not
    /// relative to a grab angle: recomputed fresh every call, matching how
    /// the reference itself behaves (`labels.rs`'s own doc comment). Feed
    /// the result straight into `label_set({"angle": ...})`.
    #[func]
    fn label_rotate_deg(&self, cx: f64, cy: f64, gx: f64, gy: f64) -> f64 {
        label_bridge::rotate_deg(cx, cy, gx, gy)
    }

    /// The arc/curve handle's live value during a drag
    /// (`cartalith_civ::labels::label_arc_value`) -- `grab_angle_deg` and
    /// `side` are captured once at grab time (the label's own `angle` and
    /// its box's `side` at that moment; freezing the angle rather than
    /// reading it live is deliberate -- `label_rotate_deg`'s own "changing
    /// angle mid-drag would fight this drag" reasoning). Feed the result
    /// straight into `label_set({"arc": ...})`.
    #[func]
    fn label_arc_value(&self, cx: f64, cy: f64, grab_angle_deg: f64, side: f64, gx: f64, gy: f64) -> f64 {
        label_bridge::arc_value(cx, cy, grab_angle_deg, side, gx, gy)
    }
}

/// `LOD_TILING_INTEGRATION_SCOPE.md` milestone M1: an interactive,
/// camera-driven caller of `amplify_region`/`refine_tile`, independent of
/// the `region_export_tiles` export bundle those functions were previously
/// only reachable through. See `lod_bridge.rs`'s own module doc for the
/// full "why" — the real-numbers case against tiling the base raster (Z3),
/// and for tiling the deep-zoom *synthesis* instead (Z2), plus why this
/// binding computes tile bounds directly rather than routing through a real
/// `TiledField`/`QuadTree` instance.
///
/// `#[godot_api(secondary)]`, not a plain `#[godot_api]`: only the first
/// `#[godot_api] impl WorldGen` block in the crate may omit `secondary` —
/// every later one collides with it on the shared signal-registration
/// machinery `WorldGen`'s `Base<RefCounted>` field generates once, the same
/// rule every other block below `IRefCounted`'s already follows.
#[godot_api(secondary)]
impl WorldGen {
    /// The pyramid level to draw at, for a camera showing `px_per_cell`
    /// screen pixels per coarse grid cell — `pyramidLevelForZoom` (reference
    /// line 10600) against this world's own width.
    ///
    /// Read this rather than reimplementing the rule in GDScript, so the
    /// viewport's own "which tiles does this screen rect touch" arithmetic
    /// can never drift from the level [`Self::lod_synthesize_tile`] would
    /// actually resolve. `0` before any world, which is also the shallowest
    /// real level, so a caller that asks too early gets a harmless answer.
    #[func]
    fn lod_level_for_zoom(&self, px_per_cell: f64) -> i32 {
        lod_bridge::level_for_zoom(px_per_cell, self.gw.max(0) as usize)
    }

    /// Tiles per axis at pyramid level `z` — `2^z`, clamped at
    /// `lod_bridge::MAX_LEVEL`. Not tied to world state, so safe before any
    /// `generate()`.
    #[func]
    fn lod_tiles_per_axis(&self, z: i32) -> i32 {
        lod_bridge::tiles_per_axis(z) as i32
    }

    /// The deepest level [`Self::lod_level_for_zoom`] will ever return —
    /// `lod_bridge::MAX_LEVEL`, the reference's own `state.lodMaxLevel`
    /// rebased onto this port's smaller interactive tile.
    #[func]
    fn lod_max_level(&self) -> i32 {
        lod_bridge::MAX_LEVEL
    }

    /// One synthesized deep-zoom tile — what `viewport_host.gd`'s deep-zoom
    /// compositor calls per visible tile once the camera's zoom crosses the
    /// "more than roughly one screen pixel per grid cell" threshold
    /// (`LOD_TILING_INTEGRATION_SCOPE.md` milestone M1).
    ///
    /// `z`/`col`/`row` are the reference's own pyramid chunk address, the
    /// same one the bake stores under: level `z` divides the map into
    /// `2^z × 2^z` tiles of `lod_bridge::TILE_PX` pixels, so the *footprint*
    /// shrinks with depth while the pixel cost does not. Call
    /// [`Self::lod_level_for_zoom`] for `z` rather than deriving it.
    ///
    /// Reads height data from whichever `WorldSource` is live — a fresh
    /// `generate()`/`generate_sized()` or a loaded save both carry a
    /// heightmap, unlike `civ`/`sculpt`/the tool layers, which a loaded
    /// save never populates (`SAVEFILE_COMPAT.md`) — the same fallback
    /// `build_color_texture`/`region_export_tiles` already use. `seed`/
    /// `sea` come from this world's own state, never a caller-guessed one,
    /// matching `region_export_tiles`'s own documented convention.
    ///
    /// Returns `None` (not a texture Godot would have to special-case) for
    /// an out-of-range tile index, before any world exists, or on any
    /// malformed source state `lod_bridge::synthesize_tile_rgba` itself
    /// guards against — see that function's own doc comment.
    #[func]
    fn lod_synthesize_tile(&self, z: i32, col: i32, row: i32) -> Option<Gd<ImageTexture>> {
        let field: &[f32] = match self.source.as_ref()? {
            WorldSource::Generated(ws) => &ws.field,
            WorldSource::Loaded(save) => &save.fields.heightmap,
        };
        let (gw, gh) = (self.gw.max(0) as usize, self.gh.max(0) as usize);
        let (rgba, out_w, out_h) = lod_bridge::synthesize_tile_rgba(
            field, gw, gh, z, col, row, self.seed, self.sea_level,
        )?;
        let packed = PackedByteArray::from(rgba);
        let image = Image::create_from_data(out_w as i32, out_h as i32, false, Format::RGBA8, &packed)?;
        ImageTexture::create_from_image(&image)
    }
}

// ============================================================================
// Journey Planner (`JOURNEY_PLANNER_SCOPE.md`) -- the boundary its own
// "Closing status" section calls for, steps 2 and 4.
//
// The engine half has been complete since 2026-08-18: 65 of the reference's
// 74 `jp*`/`_jp*` functions are ported and golden-tested in `cartalith-civ`,
// and none of it had ever been reachable from Godot. What follows is
// plumbing, not new modelling -- `journey_bridge.rs` owns everything
// expressible without `godot`, and these three `#[func]`s own the `Variant`
// conversion plus the flattening of `JpJourneyPlan`, which is a deep
// structure (stages, per-leg results with their own effective plans,
// timeline, stops) that gdext needs as nested `VarDictionary`/`Array`.
// ============================================================================

/// A `(key, JpValue)` list as a flat `VarDictionary` -- the one place the
/// `godot`-free `journey_bridge::JpValue` becomes a real `Variant`.
fn jp_pairs_dict<S: AsRef<str>>(pairs: &[(S, journey_bridge::JpValue)]) -> VarDictionary {
    let mut d = VarDictionary::new();
    for (k, v) in pairs {
        match v {
            journey_bridge::JpValue::Int(n) => d.set(k.as_ref(), *n),
            journey_bridge::JpValue::Num(n) => d.set(k.as_ref(), *n),
            journey_bridge::JpValue::Str(s) => d.set(k.as_ref(), s.as_str()),
            journey_bridge::JpValue::Bool(b) => d.set(k.as_ref(), *b),
        }
    }
    d
}

/// One `Variant` as the three kinds a plan/party form uses, or `None` for
/// anything else (an array, a `Vector2`, a null) -- which the caller reports
/// `rejected`, the same as an unknown key.
fn variant_to_jp_value(v: &Variant) -> Option<journey_bridge::JpValue> {
    match v.get_type() {
        VariantType::INT => Some(journey_bridge::JpValue::Int(v.to::<i64>())),
        VariantType::FLOAT => Some(journey_bridge::JpValue::Num(v.to::<f64>())),
        VariantType::STRING => Some(journey_bridge::JpValue::Str(v.to::<GString>().to_string())),
        VariantType::BOOL => Some(journey_bridge::JpValue::Bool(v.to::<bool>())),
        _ => None,
    }
}

/// A `Dictionary` as the `(key, JpValue)` list `journey_bridge`'s parsers
/// take, plus every key whose value was not a number/string/bool at all.
fn jp_dict_to_pairs(d: &VarDictionary) -> (Vec<(String, journey_bridge::JpValue)>, Vec<String>) {
    let mut pairs = Vec::new();
    let mut rejected = Vec::new();
    for (k, v) in d.iter_shared() {
        let key = k.to_string();
        match variant_to_jp_value(&v) {
            Some(val) => pairs.push((key, val)),
            None => rejected.push(key),
        }
    }
    (pairs, rejected)
}

/// `JpCapacity` -- the whole mass model, flat (the JS nests its five mass
/// terms under `breakdown`; `cartalith-civ` already flattened them).
fn jp_capacity_dict(c: &cartalith_civ::JpCapacity) -> VarDictionary {
    vdict! {
        "total_mass" => c.total_mass,
        "capacity" => c.capacity,
        "draft_shortfall" => c.draft_shortfall,
        "cargo" => c.cargo,
        "human_food" => c.human_food,
        "human_water" => c.human_water,
        "fodder" => c.fodder,
        "animal_water" => c.animal_water,
        "animal_food_daily" => c.animal_food_daily,
        "animal_water_daily" => c.animal_water_daily,
        "draft_food_daily" => c.draft_food_daily,
        "draft_water_daily" => c.draft_water_daily,
        "human_water_rate" => c.human_water_rate,
        "mount_credit" => c.mount_credit,
    }
}

/// `jpAssessResupply`'s verdict. `stops_needed`/`limited_by`/`cause` and the
/// two intervals are all genuinely `None` on some paths; `-1` and `""` are
/// the absent markers, matching `region_get`/`icon_armed`'s own convention of
/// never emitting a null into a `Dictionary` a GDScript caller must
/// type-test.
fn jp_resupply_dict(r: &cartalith_civ::JpResupply) -> VarDictionary {
    vdict! {
        "feasible" => r.feasible,
        "stops_needed" => r.stops_needed.unwrap_or(-1),
        "limited_by" => r.limited_by.clone().unwrap_or_default(),
        "cause" => r.cause.unwrap_or_default(),
        "binding_interval" => r.binding_interval.unwrap_or(-1.0),
        "interval_km" => r.interval_km.unwrap_or(-1.0),
        "verdict" => r.verdict.as_str(),
    }
}

/// The speed chain as `Array[{key, detail, factor}]` -- JP-05's calculation
/// trace. Still not the reference's `formula` *string*: prose stays in
/// GDScript, which formats these rows into the running-value table
/// `GUI_GAP_REGISTER.md` §7.12 designed. What crosses is only the fact of
/// which factors were applied, in order, with what values -- engine
/// knowledge that cannot be re-derived on the far side without a second
/// copy of every table.
fn jp_trace_array(trace: &[cartalith_civ::JpTerm]) -> Array<VarDictionary> {
    trace
        .iter()
        .map(|t| vdict! { "key" => t.key, "detail" => t.detail.as_str(), "factor" => t.factor })
        .collect()
}

/// `jpCalcLand`'s return, minus the reference's `formula` trace string --
/// presentation, which `ARCHITECTURE.md` assigns to Godot, and every value it
/// prints is a key here.
fn jp_land_calc_dict(l: &cartalith_civ::JpLandCalc) -> VarDictionary {
    let (desert_tier, desert_tier_auto) = l.desert_tier.map_or(("", false), |(label, auto)| (label, auto));
    let capacity = jp_capacity_dict(&l.cap);
    let resupply = l.resupply.as_ref().map_or_else(VarDictionary::new, jp_resupply_dict);
    let trace = jp_trace_array(&l.trace);
    vdict! {
        "trace" => &trace,
        "daily_km" => l.daily_km,
        "days" => l.days,
        "load_ratio" => l.load_ratio,
        "capacity" => &capacity,
        "resupply" => &resupply,
        "transport_label" => l.transport_label.as_str(),
        "mount_key" => l.mount_key.unwrap_or_default(),
        "is_desert" => l.is_desert,
        "col_km" => l.col_km,
        "col_mod" => l.col_mod,
        "dry_km" => l.dry_km,
        "water_gap_days" => l.water_gap_days,
        "supply_days" => l.supply_days,
        "portage" => l.portage,
        "desert_tier" => desert_tier,
        "desert_tier_auto" => desert_tier_auto,
    }
}

/// `jpCalcWater`'s return, same `formula` omission.
fn jp_water_calc_dict(w: &cartalith_civ::JpWaterCalc) -> VarDictionary {
    let resupply = jp_resupply_dict(&w.resupply);
    let trace = jp_trace_array(&w.trace);
    vdict! {
        "trace" => &trace,
        // JP-09's third datum ("per water leg: vessel, hold used, sailing
        // window"). The other two are `transport_label` and `hold_kg`
        // below, both already here.
        "sailing_window_h" => w.sailing_window_h,
        "daily_km" => w.daily_km,
        "days" => w.days,
        "load_ratio" => w.load_ratio,
        "resupply" => &resupply,
        "transport_label" => w.transport_label.as_str(),
        "crew" => w.crew as i64,
        "hold_kg" => w.hold_kg,
        "food_needed" => w.food_needed,
        "water_needed" => w.water_needed,
    }
}

/// One `_jpDeriveStages` stage: what the route *is*, measured against the
/// world, before any party is applied to it.
fn jp_stage_dict(s: &cartalith_civ::JpDerivedStage) -> VarDictionary {
    vdict! {
        "cat" => s.cat.as_str(),
        "biome" => s.biome.as_str(),
        "terrain" => s.terrain.as_str(),
        "route_cond" => s.route_cond.as_str(),
        "derived_cond" => s.derived_cond.clone().unwrap_or_default(),
        "infra" => s.infra.as_str(),
        "km" => s.km,
        "i0" => s.i0 as i64,
        "i1" => s.i1 as i64,
        "river_crossings" => s.rx as i64,
        "gain" => s.gain,
        "loss" => s.loss,
        "settlements" => s.settlements as i64,
        "claimed_frac" => s.claimed_frac,
        "dry_km" => s.dry_km,
        "mx" => s.mx,
        "my" => s.my,
    }
}

/// One entry of `plan.results`: the stage's own calculation plus the
/// effective plan it was computed under. `blocked` is `true` exactly when the
/// stage could not be travelled as configured, and `land`/`water` is then
/// absent -- `JpLegResult::calc` is a `Result`, so a blocked stage cannot be
/// read as a computed one by accident, and this shape keeps that property
/// across the boundary.
fn jp_leg_result_dict(r: &cartalith_civ::JpLegResult) -> VarDictionary {
    let eff = jp_pairs_dict(&journey_bridge::plan_to_pairs(&r.eff));
    let mut d = vdict! {
        "cat" => r.cat.as_str(),
        "km" => r.km,
        "days" => r.days(),
        "daily_km" => r.daily_km(),
        "eff" => &eff,
    };
    match &r.calc {
        Ok(cartalith_civ::JpLegCalc::Land(l)) => {
            d.set("blocked", false);
            d.set("land", &jp_land_calc_dict(l));
        }
        Ok(cartalith_civ::JpLegCalc::Water(w)) => {
            d.set("blocked", false);
            d.set("water", &jp_water_calc_dict(w));
        }
        Err(b) => {
            d.set("blocked", true);
            d.set("blocked_reason", b.reason.as_str());
            d.set("blocked_seasonal", b.seasonal);
        }
    }
    d
}

fn jp_timeline_dict(t: &cartalith_civ::JpTimelineDay) -> VarDictionary {
    vdict! {
        "day" => t.day,
        "km" => t.km,
        "terrain" => t.terrain.as_str(),
        "biome" => t.biome.as_str(),
        "camp" => t.camp.clone().unwrap_or_default(),
    }
}

fn jp_stop_dict(s: &cartalith_civ::JpStop) -> VarDictionary {
    vdict! {
        "key" => s.key.as_str(),
        "name" => s.name.as_str(),
        "kind" => s.kind.as_str(),
        "x" => s.x,
        "y" => s.y,
        "layover_days" => s.layover_days,
    }
}

/// `_jpResupplyReach` -- v1.51's headline finding: `jpAssessResupply` states
/// a requirement computed purely from what the party can carry, and this is
/// what compares it with the settlements the route actually passes.
fn jp_resupply_reach_dict(r: &cartalith_civ::ResupplyReach) -> VarDictionary {
    vdict! {
        "required_km" => r.required_km,
        "max_gap_km" => r.max_gap_km,
        "gap_at_km" => r.gap_at_km,
        "total_km" => r.total_km,
        "stops" => r.stops as i64,
        "unmet" => r.unmet,
        "carry_food" => r.carry_food,
        "shortfall" => r.shortfall,
    }
}

/// `_jpPlan`'s whole return, flattened. Nested arrays keep their own depth:
/// `stages`/`results`/`timeline`/`stops` are `Array[Dictionary]`, and each
/// leg result nests its own `land`/`water` calculation and effective plan.
fn jp_journey_plan_dict(p: &cartalith_civ::JpJourneyPlan) -> VarDictionary {
    let stages: Array<VarDictionary> = p.stages.iter().map(jp_stage_dict).collect();
    let results: Array<VarDictionary> = p.results.iter().map(jp_leg_result_dict).collect();
    let timeline: Array<VarDictionary> = p.timeline.iter().map(jp_timeline_dict).collect();
    let stops: Array<VarDictionary> = p.stops.iter().map(jp_stop_dict).collect();
    let profile: PackedFloat64Array = p.profile.iter().copied().collect();
    let day_fracs: PackedFloat64Array = p.day_fracs.iter().copied().collect();
    let seasons: PackedStringArray = p.seasons_crossed.iter().map(GString::from).collect();
    let reach = p.resupply_reach.as_ref().map_or_else(VarDictionary::new, jp_resupply_reach_dict);
    vdict! {
        "stages" => &stages,
        "results" => &results,
        "timeline" => &timeline,
        "stops" => &stops,
        "km" => p.km,
        // `days` is TRAVEL days only -- rest days and layovers are calendar
        // time laid on top (v1.52). `total_days` is the sum, and is `-1` on a
        // blocked journey because there is no honest total for one.
        "days" => p.days,
        "avg_km_day" => p.avg_km_day,
        "blocked" => p.blocked_idx.is_some(),
        "blocked_idx" => p.blocked_idx.map_or(-1, |i| i as i64),
        "food_kg" => p.food_kg,
        "water_l" => p.water_l,
        "fodder_kg" => p.fodder_kg,
        "river_crossings" => p.riv_x as i64,
        "pass_km" => p.pass_km,
        "desert_km" => p.desert_km,
        "bad_wx_pct" => p.bad_wx_pct,
        "profile" => &profile,
        "day_fracs" => &day_fracs,
        "ascent" => p.ascent,
        "descent" => p.descent,
        "hi_m" => p.hi_m,
        "lo_m" => p.lo_m,
        "worst_land" => p.worst_land.map_or(-1, |i| i as i64),
        "transshipments" => p.transshipments,
        "transfer_overhead" => p.transfer_overhead,
        "handling_days" => p.handling_days,
        "layover_days" => p.layover_days,
        "travel_days" => p.travel_days,
        "rest_days" => p.rest_days,
        "rest_every" => p.rest.every,
        "rest_basis" => p.rest.basis.as_str(),
        "total_days" => p.total_days.unwrap_or(-1.0),
        "seasons_crossed" => &seasons,
        "season_drift" => p.season_drift,
        "resupply_reach" => &reach,
        "has_desert" => p.has_desert,
        "has_water" => p.has_water,
        "has_land" => p.has_land,
    }
}

/// `#[godot_api(secondary)]`, not a plain `#[godot_api]`: only the first
/// `#[godot_api] impl WorldGen` block in the crate may omit `secondary` --
/// `WorldGen` has a `Base<RefCounted>` field, and a second primary block
/// collides on the shared registration machinery (`E0119`/`E0592`/`E0034`),
/// exactly as every sibling bridge block above already documents.
#[godot_api(secondary)]
impl WorldGen {
    /// Every dropdown the party/plan form needs, as
    /// `{field_key: PackedStringArray}` -- plus `"route_cond"`, nested one
    /// level deeper (`{"land": [...], "river": [...], "sea": [...]}`)
    /// because a route condition is only legal for its own travel category,
    /// and `"reference"`, the vocabularies a *results* panel needs to label
    /// what came back (terrain, biome, category, animal).
    ///
    /// The field keys are exactly the ones `jp_compute`'s `plan` dictionary
    /// accepts, so a form can be built by walking this rather than by
    /// hard-coding a second copy of the vocabulary in GDScript -- which is
    /// the failure this exists to prevent: an option string the engine does
    /// not recognise does not error, it falls through to a `?? 1.0` default
    /// and reports a plausible number computed from the wrong row.
    ///
    /// Pure: no world state is read, so this is callable before
    /// `generate()`.
    #[func]
    fn jp_options(&self) -> VarDictionary {
        let mut out = VarDictionary::new();
        for (field, opts) in journey_bridge::option_tables() {
            let arr: PackedStringArray = opts.iter().map(|s| GString::from(*s)).collect();
            out.set(field, &arr);
        }
        let mut conds = VarDictionary::new();
        for cat in ["land", "river", "sea"] {
            let arr: PackedStringArray = journey_bridge::route_cond_keys(cat).iter().map(|s| GString::from(*s)).collect();
            conds.set(cat, &arr);
        }
        out.set("route_cond", &conds);
        let mut reference = VarDictionary::new();
        for (name, opts) in journey_bridge::reference_tables() {
            let arr: PackedStringArray = opts.iter().map(|s| GString::from(*s)).collect();
            reference.set(name, &arr);
        }
        out.set("reference", &reference);
        out
    }

    /// The reference's own default plan (`_jpEnsurePlan`'s default block),
    /// flat, in exactly the key vocabulary `jp_compute`'s `plan` dictionary
    /// takes back -- so a party form can seed itself from the engine rather
    /// than restating the defaults, and a partial `plan` sent to `jp_compute`
    /// fills its gaps from precisely these values.
    ///
    /// `"party_fields"` lists the ten numeric party keys in form order, so a
    /// spinner row can be generated rather than hand-listed. Pure: callable
    /// before `generate()`.
    #[func]
    fn jp_default_plan(&self) -> VarDictionary {
        let plan = cartalith_civ::JpPlan::default();
        let mut d = jp_pairs_dict(&journey_bridge::plan_to_pairs(&plan));
        let party: PackedStringArray =
            journey_bridge::party_count_pairs(&plan.party).iter().map(|(k, _)| GString::from(*k)).collect();
        d.set("party_fields", &party);
        d
    }

    /// Plans one journey: `jp_plan` -> `jp_verdict` -> `jp_confidence`, over
    /// a route and a party/plan form.
    ///
    /// `request` keys, all optional except that one of `route`/`points` must
    /// resolve to at least two points:
    ///
    /// * `route` (int) -- index into the committed routes (`route_commit()`'s
    ///   own return, readable via `route_get`). The preferred input: the
    ///   planner then samples the route's real `f64` grid coordinates.
    /// * `points` (`PackedVector2Array`) -- an explicit polyline in grid
    ///   coordinates, used instead of `route` when present. `f32`, so a route
    ///   round-tripped through here is a rounded copy of itself.
    /// * `plan` (Dictionary) -- the ~20 plan fields plus the ten party
    ///   counts, in `jp_default_plan()`'s vocabulary. Partial is legal;
    ///   anything omitted keeps the reference default.
    /// * `stage_overrides` (Dictionary) -- `{stage_index: {field: value}}`,
    ///   the sparse per-stage override map. Every field left out cascades
    ///   from the shared plan.
    /// * `layovers` (Dictionary) -- `{stop_key: days}`, keyed by the `key`
    ///   each entry of the returned `stops` array carries.
    /// * `animal_entries` (Dictionary) -- `{species_key: entry_id}`, the
    ///   Travel Library definition occupying each of the four built-in
    ///   party-form species slots (`donkey`/`mule`/`camel`/`horse`). Naming a
    ///   **stock** entry means the built-in table; naming a **custom** one
    ///   routes that species through
    ///   `travel_bridge::TravelLibrary::animal_overrides_selected` into
    ///   `jp_plan_ex`'s resolver, so its capacity/speed/fodder/water and its
    ///   ten-row terrain table drive the plan. An entry qualifies for a slot
    ///   through its own `species_key` or its `substitutes_for` chain
    ///   (`TravelLibrary::animal_species_slot`); anything else is rejected.
    ///   Omitting the key entirely leaves the pre-selection behaviour
    ///   (`animal_overrides()`'s own last-added-custom-per-species pick).
    ///
    /// Returns `{"ok": bool, "error": String, "rejected": PackedStringArray,
    /// "plan": {...}, "verdict": {...}, "confidence": {...}}`. `rejected`
    /// lists every key that was unrecognised or wrong-typed, per this
    /// codebase's "a typo'd key is a bug worth seeing" policy -- a rejected
    /// key changes nothing, so a plan can come back `ok` *with* rejections
    /// and still be a real plan computed from the defaults. `confidence` is
    /// an empty `Dictionary` on a blocked or non-finite journey, which is
    /// `jp_confidence`'s own `None`: there is nothing honest to band.
    ///
    /// `ok` is `false` (with `error` set) before any `generate()` call, on a
    /// loaded save (which carries none of the civ substrate), for a route
    /// index that does not exist, for a polyline under two points, and for
    /// the reference's own "no derivable stages" `return null`.
    #[func]
    fn jp_compute(&self, request: VarDictionary) -> VarDictionary {
        let fail = |msg: &str| vdict! { "ok" => false, "error" => msg, "rejected" => &PackedStringArray::new() };
        let (Some(WorldSource::Generated(ws)), Some(civ)) = (self.source.as_ref(), self.civ.as_ref()) else {
            return fail("no generated world -- call generate() first (a loaded save carries no civilisation layer)");
        };
        let (gw, gh) = (self.gw.max(0) as usize, self.gh.max(0) as usize);
        if gw == 0 || gh == 0 {
            return fail("no generated world -- call generate() first");
        }

        let mut rejected = PackedStringArray::new();

        // ---- the route ----
        let pts: Vec<(f64, f64)> = if let Some(v) = request.get("points") {
            let Ok(arr) = v.try_to::<PackedVector2Array>() else {
                return fail("`points` must be a PackedVector2Array of grid coordinates");
            };
            arr.as_slice().iter().map(|p| (p.x as f64, p.y as f64)).collect()
        } else {
            let Some(idx) = request.get("route").and_then(|v| v.try_to::<i64>().ok()) else {
                return fail("give either `route` (an int index from route_commit) or `points` (a PackedVector2Array)");
            };
            let Ok(i) = usize::try_from(idx) else {
                return fail("`route` must be a non-negative index");
            };
            let Some(r) = self.infra.as_ref().and_then(|t| t.routes.get(i)) else {
                return fail("no committed route at that index -- see route_count()");
            };
            r.pts.clone()
        };
        if pts.len() < 2 {
            return fail("a journey needs at least two route points");
        }

        // ---- the spine trim (JP-07) ----
        //
        // `JOURNEY_PLANNER_SPEC.md` §3's "⇧ drag trims", as two fractions of
        // the route's own arc length. It cuts the polyline BEFORE anything
        // else reads it, so every downstream stage index, stop key and
        // per-stage override belongs to the trimmed route -- which is what
        // makes a trim indistinguishable from having drawn the shorter route
        // in the first place.
        let pts = match request.get("trim") {
            None => pts,
            Some(v) => {
                let Ok(t) = v.try_to::<Vector2>() else {
                    return fail("`trim` must be a Vector2 of two 0-1 fractions of the route's length");
                };
                match cartalith_civ::jp_trim_points(&pts, t.x as f64, t.y as f64) {
                    Some(cut) if cut.len() >= 2 => cut,
                    _ => return fail("that trim leaves no route to plan"),
                }
            }
        };

        // ---- the plan, its per-stage overrides and the layover map ----
        let mut plan = cartalith_civ::JpPlan::default();
        if let Some(v) = request.get("plan") {
            let Ok(d) = v.try_to::<VarDictionary>() else { return fail("`plan` must be a Dictionary") };
            let (pairs, bad) = jp_dict_to_pairs(&d);
            let (parsed, more_bad) = journey_bridge::plan_from_pairs(&pairs);
            for k in bad.into_iter().chain(more_bad) {
                rejected.push(&GString::from(&k));
            }
            plan = parsed;
        }
        if let Some(v) = request.get("stage_overrides") {
            let Ok(d) = v.try_to::<VarDictionary>() else {
                return fail("`stage_overrides` must be a Dictionary keyed by stage index");
            };
            for (k, ov) in d.iter_shared() {
                let idx = k.try_to::<i64>().ok().and_then(|n| usize::try_from(n).ok());
                let inner = ov.try_to::<VarDictionary>().ok();
                let (Some(idx), Some(inner)) = (idx, inner) else {
                    rejected.push(&GString::from(&format!("stage_overrides[{k}]")));
                    continue;
                };
                let (pairs, bad) = jp_dict_to_pairs(&inner);
                let (parsed, more_bad) = journey_bridge::stage_override_from_pairs(&pairs);
                for b in bad.into_iter().chain(more_bad) {
                    rejected.push(&GString::from(&format!("stage_overrides[{idx}].{b}")));
                }
                plan.stage_overrides.insert(idx, parsed);
            }
        }
        let mut layovers = cartalith_civ::JpLayovers::new();
        if let Some(v) = request.get("layovers") {
            let Ok(d) = v.try_to::<VarDictionary>() else {
                return fail("`layovers` must be a Dictionary of {stop_key: days}");
            };
            for (k, days) in d.iter_shared() {
                match (k.get_type(), days.try_to::<i64>()) {
                    (VariantType::STRING, Ok(n)) => {
                        layovers.insert(k.to::<GString>().to_string(), n);
                    }
                    _ => rejected.push(&GString::from(&format!("layovers[{k}]"))),
                }
            }
        }
        // `animal_entries` -- which Travel Library definition occupies each of
        // the four built-in party-form species slots. Absent (or absent for a
        // given species) keeps `animal_overrides()`'s own implicit pick, so a
        // caller that never sends this key is byte-for-byte unaffected.
        let mut animal_entries: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        if let Some(v) = request.get("animal_entries") {
            let Ok(d) = v.try_to::<VarDictionary>() else {
                return fail("`animal_entries` must be a Dictionary of {species_key: entry_id}");
            };
            for (k, id) in d.iter_shared() {
                match (k.get_type(), id.try_to::<GString>()) {
                    (VariantType::STRING, Ok(s)) => {
                        animal_entries.insert(k.to::<GString>().to_string(), s.to_string());
                    }
                    _ => rejected.push(&GString::from(&format!("animal_entries[{k}]"))),
                }
            }
        }
        for (k, _) in request.iter_shared() {
            let key = k.to_string();
            if !matches!(
                key.as_str(),
                "route"
                    | "points"
                    | "plan"
                    | "stage_overrides"
                    | "layovers"
                    | "animal_entries"
                    | "auto_carriage"
                    | "trim"
            ) {
                rejected.push(&GString::from(&key));
            }
        }

        // ---- the world ----
        //
        // Every raster below already exists on this `WorldGen`; `JourneyWorld`
        // derives only the three tables no pipeline stage produces, and
        // `journey_bridge`'s module doc lists the whole mapping plus the three
        // inputs this port genuinely does not have.
        let jw = journey_bridge::JourneyWorld::build(
            &ws.field,
            &civ.water_bodies,
            &ws.temperature,
            &ws.rainfall,
            gw,
            gh,
            self.world,
            self.sea_level,
            &civ.ways,
            &civ.settlements,
        );
        let world = cartalith_civ::JpWorld {
            gw,
            gh,
            world: self.world,
            map_width_km: self.map_width_km,
            sea_level: self.sea_level,
            peak_m: self.params.peak_m,
            field: &ws.field,
            cart_biome: &jw.cart_biome,
            cart_terrain: &jw.cart_terrain,
            temp: &ws.temperature,
            rain: &ws.rainfall,
            flow_field: Some(&ws.flow_discharge),
            flow_thresh: cartalith_hydrology::river_flow_thresh(gw, gh, gw, self.map_width_km),
            water_bodies: Some(&civ.water_bodies),
            territory: Some(&civ.territory),
            places: &jw.places,
            road_cells: &jw.road_cells,
            // No retained coarse current/wind field exists in this port --
            // `journey_bridge`'s module doc has the full disclosure. `None` is
            // `jp_sea_condition`'s own supported input, not a stand-in value.
            ocean_field: None,
            wind_field: None,
        };

        // `|_, _| 1.0`: the reference's own answer on a world whose wildlife
        // layer was never built, and what an exactly-average region gives.
        //
        // `jp_plan_ex` with a live Travel Library resolver, not the plain
        // `jp_plan` this called before this dispatch -- `TRAVEL_LIBRARY_
        // SPEC.md` §6, `travel_bridge.rs`'s own module doc's "What a later
        // `#[func]` layer still needs to add". `animal_overrides()` is empty
        // whenever no custom entry duplicates one of the four built-in
        // species, and `resolve_animal_stats`/`resolve_animal_terrain_mod`
        // (`cartalith-civ`) fall back to the built-in table for exactly the
        // fields an empty (or partially-incomplete) override map does not
        // answer -- so a stock-only Travel Library changes nothing about
        // this call's output versus the plain `jp_plan` it replaces (see
        // `regression_stock_only_travel_library_matches_pre_dispatch_jp_plan`
        // in this file's own test module).
        let (overrides, bad_entries) =
            self.travel_library.animal_overrides_selected(&animal_entries);
        for b in bad_entries {
            rejected.push(&GString::from(&format!("animal_entries[{b}]")));
        }
        let (stats_fn, terrain_fn) = cartalith_civ::travel_library::animal_resolver_fns(&overrides);
        let resolver = cartalith_civ::JpAnimalResolver { stats: &*stats_fn, terrain_mod: &*terrain_fn };
        // The vessel half of the same dispatch (IN-06's stated remainder).
        // Unconditional and unparameterised, unlike `animal_entries`: a
        // vessel is chosen by NAME (`plan.vessel`), so the library needs no
        // slot selection -- naming the definition IS the selection.
        let vessel_overrides = self.travel_library.vessel_overrides();
        let vessel_fn = cartalith_civ::travel_library::vessel_resolver_fn(&vessel_overrides);
        let vessel_resolver = cartalith_civ::JpVesselResolver { stats: &*vessel_fn };

        // ---- auto carriage (JP-01) ----
        //
        // `_jpRunAuto` (reference 19614): the picker runs at exactly one
        // point per refresh, before the plan is computed, and MUTATES the
        // plan -- which is why the picked counts come back in `auto.plan`
        // for the party form to write into its own (disabled) spinners,
        // the reference's own `_jpSyncAssetInputs`.
        let mut auto = VarDictionary::new();
        if request.get("auto_carriage").and_then(|v| v.try_to::<bool>().ok()) == Some(true) {
            let picked = cartalith_civ::jp_auto_pick_transport(&world, &pts, &mut plan);
            auto = jp_auto_transport_dict(&picked);
            auto.set("plan", &jp_pairs_dict(&journey_bridge::plan_to_pairs(&plan)));
        }

        let Some(journey) = cartalith_civ::jp_plan_full(
            &world,
            &pts,
            &plan,
            &layovers,
            &|_, _| 1.0,
            Some(&resolver),
            Some(&vessel_resolver),
        ) else {
            return vdict! { "ok" => false, "error" => "no derivable stages for that route", "rejected" => &rejected };
        };
        let v = cartalith_civ::jp_verdict(&journey);
        let reasons: PackedStringArray = v.reasons.iter().map(GString::from).collect();
        let verdict = vdict! {
            "level" => v.level,
            "label" => v.label,
            "text" => v.text.as_str(),
            "reasons" => &reasons,
        };
        let confidence = cartalith_civ::jp_confidence(&journey).map_or_else(VarDictionary::new, |c| {
            vdict! { "lo_days" => c.lo_days, "hi_days" => c.hi_days, "lo" => c.lo, "hi" => c.hi, "note" => c.note }
        });
        let plan_dict = jp_journey_plan_dict(&journey);
        // JP-04. `jp_journey_cost` has been ported and golden-tested since
        // milestone 3 and was called by nothing; `jp_plan_cost` is the
        // reference's own call site (line 19854), and this is the line that
        // was missing. Empty `Dictionary` on a blocked journey -- the
        // reference's own `null`, and the same convention `confidence` uses.
        let cost = cartalith_civ::jp_plan_cost(&journey, &plan)
            .map_or_else(VarDictionary::new, |c| jp_cost_dict(&c));
        vdict! {
            "ok" => true,
            "error" => "",
            "rejected" => &rejected,
            "plan" => &plan_dict,
            "verdict" => &verdict,
            "confidence" => &confidence,
            "cost" => &cost,
            "auto" => &auto,
        }
    }

    /// `_jpRerouteForMode` (reference line 20391) over a **committed
    /// route**: re-paths its two endpoints under one travel domain and
    /// rewrites the route in place, so every consumer that already names it
    /// by index (`jp_compute`'s `route` key, `route_get`) sees the new line.
    ///
    /// `force_mode`: `""` derives the domain from `transport`
    /// (`jp_mode_for_route`); `"land"`, `"water"` or `"mixed"` overrides it,
    /// which is what a blocked WATER stage's "re-route land-only" needs.
    ///
    /// `{"ok": bool, "error": String, "km": float, "points":
    /// PackedVector2Array}`. `ok:false` carries the reference's own refusal
    /// text -- an unreachable answer is refused outright rather than drawn
    /// as the straight-line fallback `route_commit` tolerates.
    #[func]
    fn jp_reroute(&mut self, route_index: i64, transport: GString, force_mode: GString) -> VarDictionary {
        let fail = |msg: &str| vdict! { "ok" => false, "error" => msg, "km" => 0.0, "points" => &PackedVector2Array::new() };
        let (Some(WorldSource::Generated(ws)), Some(civ)) = (self.source.as_ref(), self.civ.as_ref()) else {
            return fail("no generated world -- call generate() first");
        };
        let Ok(i) = usize::try_from(route_index) else { return fail("`route_index` must be non-negative") };
        let Some(route) = self.infra.as_ref().and_then(|t| t.routes.get(i)) else {
            return fail("no committed route at that index -- see route_count()");
        };
        let (pts, mode) = (route.pts.clone(), route.mode);
        let inputs = infra_tools_bridge::RouteInputs::build(
            ws, self.gw as usize, self.gh as usize, self.world, self.map_width_km, self.params.river_density, mode,
        );
        let manual_ways = self.infra.as_ref().map(|t| t.ways.clone()).unwrap_or_default();
        let mut way_refs: Vec<cartalith_civ::tools::WayRef> =
            civ.ways.iter().map(cartalith_civ::tools::WayRef::from).collect();
        way_refs.extend(manual_ways.iter().map(cartalith_civ::tools::WayRef::from));
        let ctx = cartalith_civ::tools::RouteContext {
            field: &ws.field,
            water_bodies: &inputs.water_bodies,
            biome: inputs.biome.as_deref(),
            river_order: inputs.river_order.as_deref(),
            places: &civ.settlements,
            ways: &way_refs,
            gw: self.gw as usize,
            gh: self.gh as usize,
            sea: self.sea_level,
            world: self.world,
            map_width_km: self.map_width_km,
        };
        let forced = force_mode.to_string();
        let forced = (!forced.is_empty()).then_some(forced);
        match cartalith_civ::jp_reroute_for_mode(&ctx, &pts, &transport.to_string(), forced.as_deref()) {
            Err(e) => fail(&e),
            Ok(r) => {
                let points: PackedVector2Array =
                    r.pts.iter().map(|&(x, y)| Vector2::new(x as f32, y as f32)).collect();
                let km = r.km;
                if let Some(route) = self.infra.as_mut().and_then(|t| t.routes.get_mut(i)) {
                    route.pts = r.pts;
                    route.brks = r.brks;
                    route.km = km;
                    route.unreachable_legs = 0;
                }
                vdict! { "ok" => true, "error" => "", "km" => km, "points" => &points }
            }
        }
    }
}

/// [`cartalith_civ::JourneyCost`] flattened. Every figure is in **day-wages**
/// (`JP_COST_*`'s own unit), never a currency -- `jp_journey_cost`'s own doc
/// comment has the reasoning. `per_tonne_km`/`break_even_per_tonne` are `-1`
/// when there is no cargo to divide by, the same "-1 is absent" convention
/// `stops_needed`/`total_days` already use.
fn jp_cost_dict(c: &cartalith_civ::JourneyCost) -> VarDictionary {
    vdict! {
        "total" => c.total,
        "carriage" => c.carriage,
        "wages" => c.wages,
        "crew" => c.crew,
        "upkeep" => c.upkeep,
        "tolls" => c.tolls,
        "transship" => c.transship,
        "borders" => c.borders as i64,
        "days" => c.days,
        "cargo_t" => c.cargo_t,
        "per_tonne_km" => c.per_tonne_km.unwrap_or(-1.0),
        "break_even_per_tonne" => c.break_even_per_tonne.unwrap_or(-1.0),
        "unit" => "day-wages",
    }
}

/// [`cartalith_civ::JpAutoTransport`] flattened -- `jpAutoPickTransport`'s
/// own `{ok, hint, promoted, warn}` return, with the reference's *prose*
/// hint left to GDScript and its inputs carried instead.
fn jp_auto_transport_dict(a: &cartalith_civ::JpAutoTransport) -> VarDictionary {
    use cartalith_civ::JpAutoTransport as A;
    let mut d = match a {
        A::NoLandStages => vdict! { "ok" => false, "reason" => "no_land_stages" },
        A::NotALandMode => vdict! { "ok" => false, "reason" => "not_a_land_mode" },
        A::Walking { total_need, porter_cap } => vdict! {
            "ok" => true, "reason" => "walking", "total_need" => *total_need, "porter_cap" => *porter_cap,
        },
        A::WalkingOverloaded { total_need, porter_cap } => vdict! {
            "ok" => true, "reason" => "walking_overloaded", "warn" => true,
            "total_need" => *total_need, "porter_cap" => *porter_cap,
        },
        A::Mount { pick } => vdict! {
            "ok" => true, "reason" => "mount", "species" => pick.key, "why" => pick.reason.as_str(),
        },
        A::BaggageTrain { pick, count, carts, wagons, promoted, fodder_infeasible } => vdict! {
            "ok" => true, "reason" => "baggage_train", "species" => pick.key, "why" => pick.reason.as_str(),
            "count" => *count, "carts" => *carts, "wagons" => *wagons,
            "promoted" => *promoted, "fodder_infeasible" => *fodder_infeasible,
        },
    };
    if !d.contains_key("warn") {
        d.set("warn", false);
    }
    if !d.contains_key("promoted") {
        d.set("promoted", false);
    }
    d
}

/// `TRAVEL_LIBRARY_SPEC.md`'s `#[func]` boundary -- omission O1 / gap
/// register row DM-15. `travel_bridge.rs` owns every real behaviour (CRUD,
/// validation, usage tracking, the `Variant`-shaped field pairs); this block
/// is the thin `kind: GString` ("animal"/"vehicle"/"vessel"/"preset")
/// dispatch plus `Dictionary` flattening `ARCHITECTURE.md` assigns to this
/// crate, mirroring `jp_options`/`jp_compute` above's own division of labour
/// with `journey_bridge.rs`.
///
/// `#[godot_api(secondary)]`, not a plain `#[godot_api]`: only the first
/// `#[godot_api] impl WorldGen` block in the crate may omit `secondary` --
/// `WorldGen` has a `Base<RefCounted>` field, and a second primary block
/// collides on the shared registration machinery (`E0119`/`E0592`/`E0034`),
/// exactly as every sibling bridge block above already documents.
#[godot_api(secondary)]
impl WorldGen {
    /// `{kind: {"total": int, "custom": int, "stock": int}}` for all four
    /// definition types -- the tab counts §2's own mockup shows
    /// (`"ANIMALS & MOUNTS · 11"` etc). Callable before `generate()`: the
    /// library is real, stock-seeded state from `init()` onward.
    #[func]
    fn tl_counts(&self) -> VarDictionary {
        fn one<T: travel_bridge::TravelEntry>(set: &travel_bridge::EntrySet<T>) -> VarDictionary {
            let total = set.iter().count();
            let custom = set.iter().filter(|e| e.origin() == cartalith_civ::travel_library::EntryOrigin::Custom).count();
            vdict! { "total" => total as i64, "custom" => custom as i64, "stock" => (total - custom) as i64 }
        }
        vdict! {
            "animal" => &one(&self.travel_library.animals),
            "vehicle" => &one(&self.travel_library.vehicles),
            "vessel" => &one(&self.travel_library.vessels),
            "preset" => &one(&self.travel_library.presets),
        }
    }

    /// Every entry of one definition type (`kind`: `"animal"` / `"vehicle"`
    /// / `"vessel"` / `"preset"`), stock entries first in bootstrap order
    /// then custom entries in add order (`EntrySet`'s own iteration order --
    /// the list rail's own CUSTOM-then-STOCK section split is a GDScript
    /// grouping of this same flat array, not a second server-side order).
    /// Each row: `id`, `name`, `origin` (`"stock"`/`"custom"`), `editable`
    /// (bool), `subtitle`, `species_key` (animals only, else `""`),
    /// `validation_state` (`"ok"`/`"incomplete"`/`"conflicting"`),
    /// `validation_missing`/`validation_conflicts` (`PackedStringArray`),
    /// `usage_presets`/`usage_journeys` (int; always `0` for vehicles/
    /// vessels/presets themselves -- see `tl_get`'s own doc comment).
    /// Animal rows carry two more: `species_slot` and `usable_as_mount` --
    /// see `tl_animal_slot_keys`. Empty `Array` for an unrecognised `kind`.
    #[func]
    fn tl_list(&self, kind: GString) -> Array<VarDictionary> {
        let lib = &self.travel_library;
        match kind.to_string().as_str() {
            "animal" => lib
                .animals
                .iter()
                .map(|a| {
                    let mut d = tl_meta_dict(
                        &a.id,
                        &a.name,
                        a.origin,
                        &travel_bridge::animal_subtitle(a),
                        &cartalith_civ::travel_library::validate_animal(a),
                        lib.animal_usage_in_presets(&a.id),
                        lib.animal_usage_in_journeys(&a.id),
                        a.species_key.unwrap_or(""),
                    );
                    tl_animal_slot_keys(&mut d, lib, a);
                    d
                })
                .collect(),
            "vehicle" => lib
                .vehicles
                .iter()
                .map(|v| {
                    tl_meta_dict(
                        &v.id,
                        &v.name,
                        v.origin,
                        &travel_bridge::vehicle_subtitle(v),
                        &cartalith_civ::travel_library::validate_vehicle(v),
                        0,
                        0,
                        "",
                    )
                })
                .collect(),
            "vessel" => lib
                .vessels
                .iter()
                .map(|v| {
                    tl_meta_dict(
                        &v.id,
                        &v.name,
                        v.origin,
                        &travel_bridge::vessel_subtitle(v),
                        &cartalith_civ::travel_library::validate_vessel(v),
                        0,
                        0,
                        "",
                    )
                })
                .collect(),
            "preset" => lib
                .presets
                .iter()
                .map(|p| {
                    tl_meta_dict(
                        &p.id,
                        &p.name,
                        p.origin,
                        &travel_bridge::preset_subtitle(p),
                        &cartalith_civ::travel_library::validate_party_preset(p),
                        0,
                        0,
                        "",
                    )
                })
                .collect(),
            _ => Array::new(),
        }
    }

    /// One entry's full detail: `tl_list`'s own per-row keys plus every
    /// field `TRAVEL_LIBRARY_SPEC.md` §3 lists for that `kind`, flattened by
    /// `animal_to_pairs`/`vehicle_to_pairs`/`vessel_to_pairs`/
    /// `preset_to_pairs` (`travel_bridge.rs`) -- an unset optional field is
    /// simply **absent** from the returned `Dictionary`, matching §3's own
    /// "a field left unset is incomplete, not zero" (an inspector should
    /// test `has()`, not read a fabricated default). `{"ok": false}` for an
    /// unknown `kind` or `id`.
    ///
    /// `usage_presets`/`usage_journeys` are always `0` for vehicles/vessels/
    /// presets themselves: no `vehicle_key`/`vessel_key` equivalent to
    /// `AnimalDef::species_key` exists to attribute a `JpParty` vehicle
    /// count (`carts`/`wagons`/`sleds`/`travois`) back to one specific
    /// `VehicleDef` id, and a party preset does not reference itself --
    /// disclosed here rather than approximated, the same honesty
    /// `TravelLibrary::animal_usage_in_journeys` already applies to §4's
    /// "saved journeys" count.
    #[func]
    fn tl_get(&self, kind: GString, id: GString) -> VarDictionary {
        let lib = &self.travel_library;
        let id = id.to_string();
        match kind.to_string().as_str() {
            "animal" => {
                let Some(a) = lib.animals.get(&id) else { return vdict! { "ok" => false } };
                let mut d = tl_meta_dict(
                    &a.id,
                    &a.name,
                    a.origin,
                    &travel_bridge::animal_subtitle(a),
                    &cartalith_civ::travel_library::validate_animal(a),
                    lib.animal_usage_in_presets(&a.id),
                    lib.animal_usage_in_journeys(&a.id),
                    a.species_key.unwrap_or(""),
                );
                merge_pairs(&mut d, &travel_bridge::animal_to_pairs(a));
                tl_animal_slot_keys(&mut d, lib, a);
                d.set("ok", true);
                d
            }
            "vehicle" => {
                let Some(v) = lib.vehicles.get(&id) else { return vdict! { "ok" => false } };
                let mut d = tl_meta_dict(
                    &v.id,
                    &v.name,
                    v.origin,
                    &travel_bridge::vehicle_subtitle(v),
                    &cartalith_civ::travel_library::validate_vehicle(v),
                    0,
                    0,
                    "",
                );
                merge_pairs(&mut d, &travel_bridge::vehicle_to_pairs(v));
                d.set("ok", true);
                d
            }
            "vessel" => {
                let Some(v) = lib.vessels.get(&id) else { return vdict! { "ok" => false } };
                let mut d = tl_meta_dict(
                    &v.id,
                    &v.name,
                    v.origin,
                    &travel_bridge::vessel_subtitle(v),
                    &cartalith_civ::travel_library::validate_vessel(v),
                    0,
                    0,
                    "",
                );
                merge_pairs(&mut d, &travel_bridge::vessel_to_pairs(v));
                d.set("ok", true);
                d
            }
            "preset" => {
                let Some(p) = lib.presets.get(&id) else { return vdict! { "ok" => false } };
                let mut d = tl_meta_dict(
                    &p.id,
                    &p.name,
                    p.origin,
                    &travel_bridge::preset_subtitle(p),
                    &cartalith_civ::travel_library::validate_party_preset(p),
                    0,
                    0,
                    "",
                );
                merge_pairs(&mut d, &travel_bridge::preset_to_pairs(p));
                d.set("ok", true);
                d
            }
            _ => vdict! { "ok" => false },
        }
    }

    /// Clones `id` (stock or custom) into a new custom entry --
    /// `TRAVEL_LIBRARY_SPEC.md`'s "duplicate to edit", the only way to get
    /// an editable copy of a stock entry. `{"ok": true, "id": new_id}`, or
    /// `{"ok": false, "error": ...}` for an unknown `kind` or source `id`.
    #[func]
    fn tl_duplicate(&mut self, kind: GString, id: GString) -> VarDictionary {
        let id = id.to_string();
        let new_id = self.travel_library.fresh_id();
        let ok = match kind.to_string().as_str() {
            "animal" => self.travel_library.animals.duplicate(&id, new_id.clone()).is_some(),
            "vehicle" => self.travel_library.vehicles.duplicate(&id, new_id.clone()).is_some(),
            "vessel" => self.travel_library.vessels.duplicate(&id, new_id.clone()).is_some(),
            "preset" => self.travel_library.presets.duplicate(&id, new_id.clone()).is_some(),
            _ => false,
        };
        if ok {
            vdict! { "ok" => true, "id" => new_id.as_str(), "error" => "" }
        } else {
            vdict! { "ok" => false, "error" => "unknown kind or source id", "id" => "" }
        }
    }

    /// A brand-new custom entry with every field unset --
    /// `TRAVEL_LIBRARY_SPEC.md`'s "New blank definition…". `{"ok": true,
    /// "id": new_id}`.
    #[func]
    fn tl_add_blank(&mut self, kind: GString, name: GString) -> VarDictionary {
        let new_id = self.travel_library.fresh_id();
        let name_s = name.to_string();
        let ok = match kind.to_string().as_str() {
            "animal" => self
                .travel_library
                .animals
                .add(cartalith_civ::travel_library::AnimalDef::blank(new_id.clone(), name_s))
                .is_some(),
            "vehicle" => self
                .travel_library
                .vehicles
                .add(cartalith_civ::travel_library::VehicleDef::blank(new_id.clone(), name_s))
                .is_some(),
            "vessel" => self
                .travel_library
                .vessels
                .add(cartalith_civ::travel_library::VesselDef::blank(new_id.clone(), name_s))
                .is_some(),
            "preset" => self
                .travel_library
                .presets
                .add(cartalith_civ::travel_library::PartyPreset::blank(new_id.clone(), name_s))
                .is_some(),
            _ => false,
        };
        if ok {
            vdict! { "ok" => true, "id" => new_id.as_str(), "error" => "" }
        } else {
            vdict! { "ok" => false, "error" => "kind must be animal/vehicle/vessel/preset", "id" => "" }
        }
    }

    /// Deletes a custom entry. No-op (`"ok": false`) on an unknown id or a
    /// stock one -- `TRAVEL_LIBRARY_SPEC.md` §3's "stock entries are
    /// read-only", enforced by `EntrySet::delete` itself, not re-checked
    /// here.
    #[func]
    fn tl_delete(&mut self, kind: GString, id: GString) -> VarDictionary {
        let id = id.to_string();
        let ok = match kind.to_string().as_str() {
            "animal" => self.travel_library.animals.delete(&id),
            "vehicle" => self.travel_library.vehicles.delete(&id),
            "vessel" => self.travel_library.vessels.delete(&id),
            "preset" => self.travel_library.presets.delete(&id),
            _ => false,
        };
        vdict! { "ok" => ok }
    }

    /// Discards every custom entry of one `kind`, restoring the stock-only
    /// bootstrap -- `TRAVEL_LIBRARY_SPEC.md`'s "Reset to stock
    /// definitions…".
    #[func]
    fn tl_reset_to_stock(&mut self, kind: GString) -> VarDictionary {
        match kind.to_string().as_str() {
            "animal" => self.travel_library.animals.reset_to_stock(),
            "vehicle" => self.travel_library.vehicles.reset_to_stock(),
            "vessel" => self.travel_library.vessels.reset_to_stock(),
            "preset" => self.travel_library.presets.reset_to_stock(),
            _ => return vdict! { "ok" => false },
        }
        vdict! { "ok" => true }
    }

    /// Applies a partial `fields` `Dictionary` (exactly `tl_get`'s own key
    /// vocabulary for `kind`) onto an existing **custom** entry -- stock
    /// entries are read-only (`"ok": false`, per §3; duplicate first).
    /// Every field `fields` does not mention keeps its current value
    /// (`travel_bridge.rs`'s `_apply_pairs` functions' own "partial edit
    /// preserves the rest" contract). Returns `{"ok", "error", "rejected"
    /// (PackedStringArray of unrecognised/wrong-typed keys), "validation_state",
    /// "validation_missing", "validation_conflicts"}` -- the entry's *new*
    /// validation, computed immediately, so the caller can show a banner
    /// without a second round trip.
    #[func]
    fn tl_edit(&mut self, kind: GString, id: GString, fields: VarDictionary) -> VarDictionary {
        let id = id.to_string();
        let (pairs, bad_keys) = jp_dict_to_pairs(&fields);
        let mut rejected: PackedStringArray = bad_keys.iter().map(GString::from).collect();
        let fail = |msg: &str, rejected: &PackedStringArray| {
            vdict! { "ok" => false, "error" => msg, "rejected" => rejected }
        };
        match kind.to_string().as_str() {
            "animal" => {
                let Some(existing) = self.travel_library.animals.get(&id).cloned() else {
                    return fail("no such animal id", &rejected);
                };
                if existing.origin != cartalith_civ::travel_library::EntryOrigin::Custom {
                    return fail("stock entries are read-only -- duplicate to edit", &rejected);
                }
                let (updated, more) = travel_bridge::animal_apply_pairs(&existing, &pairs);
                rejected.extend_array(&more.iter().map(GString::from).collect());
                *self.travel_library.animals.get_mut(&id).expect("just confirmed custom") = updated;
                let validation = self.travel_library.animal_validation(&id).expect("just stored above");
                tl_edit_result(&rejected, &validation)
            }
            "vehicle" => {
                let Some(existing) = self.travel_library.vehicles.get(&id).cloned() else {
                    return fail("no such vehicle id", &rejected);
                };
                if existing.origin != cartalith_civ::travel_library::EntryOrigin::Custom {
                    return fail("stock entries are read-only -- duplicate to edit", &rejected);
                }
                let (updated, more) = travel_bridge::vehicle_apply_pairs(&existing, &pairs);
                rejected.extend_array(&more.iter().map(GString::from).collect());
                *self.travel_library.vehicles.get_mut(&id).expect("just confirmed custom") = updated;
                let validation = self.travel_library.vehicle_validation(&id).expect("just stored above");
                tl_edit_result(&rejected, &validation)
            }
            "vessel" => {
                let Some(existing) = self.travel_library.vessels.get(&id).cloned() else {
                    return fail("no such vessel id", &rejected);
                };
                if existing.origin != cartalith_civ::travel_library::EntryOrigin::Custom {
                    return fail("stock entries are read-only -- duplicate to edit", &rejected);
                }
                let (updated, more) = travel_bridge::vessel_apply_pairs(&existing, &pairs);
                rejected.extend_array(&more.iter().map(GString::from).collect());
                *self.travel_library.vessels.get_mut(&id).expect("just confirmed custom") = updated;
                let validation = self.travel_library.vessel_validation(&id).expect("just stored above");
                tl_edit_result(&rejected, &validation)
            }
            "preset" => {
                let Some(existing) = self.travel_library.presets.get(&id).cloned() else {
                    return fail("no such preset id", &rejected);
                };
                if existing.origin != cartalith_civ::travel_library::EntryOrigin::Custom {
                    return fail("stock entries are read-only -- duplicate to edit", &rejected);
                }
                let (updated, more) = travel_bridge::preset_apply_pairs(&existing, &pairs);
                rejected.extend_array(&more.iter().map(GString::from).collect());
                *self.travel_library.presets.get_mut(&id).expect("just confirmed custom") = updated;
                let validation = self.travel_library.preset_validation(&id).expect("just stored above");
                tl_edit_result(&rejected, &validation)
            }
            _ => fail("kind must be animal/vehicle/vessel/preset", &rejected),
        }
    }

    /// `TRAVEL_LIBRARY_SPEC.md`'s "Capture party from planner": a new
    /// custom party preset from the planner's current form, in
    /// `jp_default_plan()`/`jp_compute`'s own `plan` key vocabulary (a
    /// subset is fine -- unmentioned fields keep `JpPlan::default()`).
    /// `{"ok": true, "id": new_id, "rejected": [...]}`.
    #[func]
    fn tl_capture_preset_from_plan(&mut self, name: GString, plan: VarDictionary) -> VarDictionary {
        let (pairs, bad) = jp_dict_to_pairs(&plan);
        let (parsed_plan, more_bad) = journey_bridge::plan_from_pairs(&pairs);
        let rejected: PackedStringArray = bad.into_iter().chain(more_bad).map(|k| GString::from(&k)).collect();
        let new_id = self.travel_library.fresh_id();
        let preset =
            cartalith_civ::travel_library::PartyPreset::from_jp_plan(new_id.clone(), name.to_string(), &parsed_plan);
        self.travel_library.presets.add(preset);
        vdict! { "ok" => true, "id" => new_id.as_str(), "rejected" => &rejected }
    }
}

/// `tl_list`/`tl_get`'s shared per-entry metadata -- every key both
/// functions report about an entry regardless of `kind`, so `kind`-specific
/// field data (`tl_get` only) is simply merged on top rather than
/// duplicating this block four times.
/// The two animal-only keys the Journey Planner's party form needs on top of
/// [`tl_meta_dict`]'s shared metadata, so building the per-species dropdowns
/// costs one `tl_list("animal")` call rather than one `tl_get` per entry:
///
/// * `species_slot` -- which of the four built-in party-form species this
///   entry may occupy (`TravelLibrary::animal_species_slot`: its own
///   `species_key`, or the one its `substitutes_for` chain reaches). `""`
///   means it has no `JpParty` slot and the planner cannot offer it, which
///   the form states rather than hiding.
/// * `usable_as_mount` -- §3.1's own flag, the filter the Mount dropdown
///   applies.
fn tl_animal_slot_keys(
    d: &mut VarDictionary,
    lib: &travel_bridge::TravelLibrary,
    a: &cartalith_civ::travel_library::AnimalDef,
) {
    d.set("species_slot", lib.animal_species_slot(&a.id).unwrap_or(""));
    d.set("usable_as_mount", a.usable_as_mount);
}

fn tl_meta_dict(
    id: &str,
    name: &str,
    origin: cartalith_civ::travel_library::EntryOrigin,
    subtitle: &str,
    validation: &cartalith_civ::travel_library::ValidationState,
    usage_presets: usize,
    usage_journeys: usize,
    species_key: &str,
) -> VarDictionary {
    let (state, missing, conflicts) = travel_bridge::validation_state_parts(validation);
    let missing_arr: PackedStringArray = missing.iter().map(GString::from).collect();
    let conflicts_arr: PackedStringArray = conflicts.iter().map(GString::from).collect();
    vdict! {
        "id" => id,
        "name" => name,
        "origin" => if origin == cartalith_civ::travel_library::EntryOrigin::Stock { "stock" } else { "custom" },
        "editable" => origin == cartalith_civ::travel_library::EntryOrigin::Custom,
        "subtitle" => subtitle,
        "species_key" => species_key,
        "validation_state" => state,
        "validation_missing" => &missing_arr,
        "validation_conflicts" => &conflicts_arr,
        "usage_presets" => usage_presets as i64,
        "usage_journeys" => usage_journeys as i64,
    }
}

/// `tl_get`'s field-data merge step: every `(key, JpValue)` pair from a
/// `*_to_pairs` function, set onto an already-built metadata `Dictionary`.
/// Reuses `jp_pairs_dict` (already generic over `journey_bridge::JpValue`)
/// rather than a second flattening loop.
fn merge_pairs<S: AsRef<str>>(d: &mut VarDictionary, pairs: &[(S, journey_bridge::JpValue)]) {
    for (k, v) in jp_pairs_dict(pairs).iter_shared() {
        d.set(&k, &v);
    }
}

/// `tl_edit`'s shared success-path result `Dictionary` -- built once here
/// rather than four times inline across the `kind` match arms.
fn tl_edit_result(rejected: &PackedStringArray, validation: &cartalith_civ::travel_library::ValidationState) -> VarDictionary {
    let (state, missing, conflicts) = travel_bridge::validation_state_parts(validation);
    let missing_arr: PackedStringArray = missing.iter().map(GString::from).collect();
    let conflicts_arr: PackedStringArray = conflicts.iter().map(GString::from).collect();
    vdict! {
        "ok" => true,
        "error" => "",
        "rejected" => rejected,
        "validation_state" => state,
        "validation_missing" => &missing_arr,
        "validation_conflicts" => &conflicts_arr,
    }
}

/// `DCC_SHELL_SPEC.md` §6's Sample context and the canvas Layers popover.
/// See `sample_bridge.rs`'s own module doc for the memory rule this block
/// was written under -- in particular the table showing that **every one of
/// §6's sixteen Sample fields is answered from state generation already
/// retains**, with nothing added to `WorldGen`/`WorldState`/`CivData`.
///
/// `#[godot_api(secondary)]`, not a plain `#[godot_api]`: only the first
/// `#[godot_api] impl WorldGen` block in the crate may omit `secondary` --
/// `WorldGen` has a `Base<RefCounted>` field, and a second primary block
/// collides on the shared registration machinery (`E0119`/`E0592`/`E0034`),
/// exactly as every sibling bridge block above already documents.
#[godot_api(secondary)]
impl WorldGen {
    /// Every field the right dock's Sample panel shows, for one grid cell,
    /// in **one** call -- deliberately not sixteen per-field getters, since
    /// `right_dock.gd`'s `on_cursor_sampled` fires on every mouse-motion
    /// event over the viewport and sixteen round trips per motion event
    /// would be sixteen times the boundary crossings for one readout.
    ///
    /// Returns `{}` before any `generate()` and for an out-of-grid cell (the
    /// dock shows an em dash rather than this method clamping to an edge
    /// cell and reporting a neighbour's readings as this cell's). Otherwise
    /// every key below is present except the ones whose backing data
    /// genuinely is not there, which are **omitted rather than zero-filled**:
    ///
    /// * `x`, `y` (int), `elevation` (raw `[0,1]` field value),
    ///   `elevation_m` (metres, negative below sea level),
    ///   `slope_deg` (real ground angle), `slope_n` (`slopeAt*GW`, the
    ///   engine's own unit), `plate` (int), `plate_type`
    ///   (`"oceanic"`/`"continental"`), `boundary` (bool), `boundary_type`
    ///   (String), `stress`, `age`, `resistance`, `lithology` (String),
    ///   `temperature_c`, `precipitation`, `drainage` (flow discharge),
    ///   `soil` -- all from `WorldState`, always present.
    /// * `aspect_deg` + `aspect` (16-point compass) -- omitted on flat
    ///   ground, where an aspect is undefined rather than zero.
    /// * `boundary_dist_cells` -- omitted when no tagged boundary lies
    ///   within `sample_bridge::BOUNDARY_SEARCH_MAX` cells (the search is
    ///   capped so a world with no boundary at all cannot turn one
    ///   mouse-motion event into a full-grid scan).
    /// * `river_order` -- omitted when river extraction did not run
    ///   (`WorldState::stream_order` is `None`).
    /// * `water` (`"land"`/`"ocean"`/`"lake"`), `biome`, `control` --
    ///   omitted without a civilisation layer, i.e. on a loaded save.
    #[func]
    fn sample_cell(&self, gx: i32, gy: i32) -> VarDictionary {
        let Some(f) = self.sample_refs() else { return VarDictionary::new() };
        let Some(s) = sample_bridge::sample_cell(&f, gx as i64, gy as i64) else { return VarDictionary::new() };
        let mut d = vdict! {
            "x" => s.x as i64,
            "y" => s.y as i64,
            "elevation" => s.elevation,
            "elevation_m" => s.elevation_m,
            "slope_deg" => s.slope_deg,
            "slope_n" => s.slope_n,
            "plate" => s.plate,
            "plate_type" => if s.plate_oceanic { "oceanic" } else { "continental" },
            "boundary" => s.on_boundary,
            "boundary_type" => s.boundary_type,
            "stress" => s.stress,
            "age" => s.age,
            "resistance" => s.resistance,
            "lithology" => s.lithology,
            "temperature_c" => s.temperature_c,
            "precipitation" => s.precipitation,
            "drainage" => s.drainage,
        };
        if let Some(a) = s.aspect_deg {
            d.set("aspect_deg", a);
            d.set("aspect", sample_bridge::compass(a));
        }
        if let Some(bd) = s.boundary_dist_cells {
            d.set("boundary_dist_cells", bd);
        }
        if let Some(o) = s.river_order {
            d.set("river_order", o);
        }
        if let Some(so) = s.soil {
            d.set("soil", so);
        }
        if let Some(w) = s.water_body {
            d.set(
                "water",
                match w {
                    1 => "ocean",
                    2 => "lake",
                    _ => "land",
                },
            );
        }
        if let Some(b) = s.biome {
            d.set("biome", b);
        }
        if let Some(c) = s.control {
            d.set("control", c);
        }
        d
    }

    /// The Layers popover's own menu, in the reference's own
    /// `LAYER_GROUPS` order and headings (reference HTML line 13639):
    /// an `Array` of `{"group": String, "items": Array[{id, label, hint,
    /// available: bool, legend: Array[{r,g,b,label}]}]}`.
    ///
    /// `available` is real, not decorative: a view whose one input this
    /// particular world does not have (Strahler order without river
    /// extraction, biomes/terrain/control on a loaded save) reports
    /// `false`, so the popover can grey the row out instead of offering a
    /// pick that would return nothing. Callable before `generate()`, where
    /// only `"off"` is available.
    ///
    /// Answering `available` costs nothing: `layer_available` reads which
    /// *inputs* exist rather than building each raster to see whether it
    /// works. At 2048x2048 the try-it-and-see version would have derived
    /// seventeen full-grid rasters every time the popover opened.
    #[func]
    fn debug_layers(&self) -> Array<VarDictionary> {
        let refs = self.sample_refs();
        sample_bridge::LAYER_GROUPS
            .iter()
            .map(|(group, entries)| {
                let items: Array<VarDictionary> = entries
                    .iter()
                    .map(|(id, label, hint)| {
                        let available =
                            refs.as_ref().map(|f| sample_bridge::layer_available(f, id)).unwrap_or(*id == "off");
                        let legend: Array<VarDictionary> = sample_bridge::legend(id)
                            .into_iter()
                            .map(|(r, g, b, text)| {
                                vdict! { "r" => r as i64, "g" => g as i64, "b" => b as i64, "label" => text.as_str() }
                            })
                            .collect();
                        vdict! {
                            "id" => *id,
                            "label" => *label,
                            "hint" => *hint,
                            "available" => available,
                            "legend" => &legend,
                        }
                    })
                    .collect();
                vdict! { "group" => *group, "items" => &items }
            })
            .collect()
    }

    /// One debug view as an `ImageTexture` the size of the grid, ready to
    /// stack over `build_color_texture()`'s raster. `null` for `"off"`, an
    /// unknown id, before any `generate()`, or for a view this world has no
    /// input for (see `debug_layers`' `available`).
    ///
    /// **Nothing is cached.** The buffer is built, handed to Godot, and
    /// dropped on the Rust side; re-picking a view re-derives it. Caching
    /// all seventeen would be ~270 MB of RGBA at 2048x2048, which is
    /// exactly the kind of uncosted retention `MEMORY_OPTIMIZATION_SCOPE.md`
    /// exists to prevent.
    /// `showWildInfo`'s own data source (reference HTML lines 9785-9791 for
    /// the pick, 8259-8276 for what the popup shows): the wildlife
    /// ecoregion whose marker is nearest `(gx, gy)`, or an empty dictionary
    /// if none is within the reference's own hit radius.
    ///
    /// The reference's rules, reproduced: only regions at or above
    /// `markerMin` cells draw a marker and are therefore clickable, the
    /// nearest is chosen by squared distance to `(cx, cy)`, and the hit
    /// radius is `max(8, GW/40)` cells.
    ///
    /// Every population figure comes back pre-formatted by `wild_fmt_pop`
    /// alongside its raw number, so the dock renders the reference's own
    /// `~4.5M` wording without reimplementing the formatter in GDScript.
    #[func]
    fn wildlife_region_at(&self, gx: f32, gy: f32) -> VarDictionary {
        let Some(f) = self.sample_refs() else {
            return VarDictionary::new();
        };
        let Some(eco) = sample_bridge::wildlife_regions(&f) else {
            return VarDictionary::new();
        };
        let hit_r = (self.gw as f64 / 40.0).max(8.0);
        let mut best: Option<(&cartalith_civ::wildlife::Ecoregion, f64)> = None;
        for rec in eco.regions.iter() {
            if rec.cells < eco.marker_min {
                continue;
            }
            let (dx, dy) = (rec.cx as f64 - gx as f64, rec.cy as f64 - gy as f64);
            let d = dx * dx + dy * dy;
            if best.is_none_or(|(_, bd)| d < bd) {
                best = Some((rec, d));
            }
        }
        let Some((rec, d)) = best else {
            return VarDictionary::new();
        };
        if d > hit_r * hit_r {
            return VarDictionary::new();
        }
        let guilds: Array<VarDictionary> = rec
            .guilds
            .iter()
            .map(|g| {
                let species: Array<VarDictionary> = g
                    .species
                    .iter()
                    .map(|s| {
                        dict! {
                            "name" => s.name,
                            "mass_kg" => s.mass_kg,
                            "population_est" => s.population_est,
                            "population_text" => cartalith_civ::wildlife::wild_fmt_pop(s.population_est),
                        }
                    })
                    .collect();
                let slot = cartalith_civ::wildlife::WILD_GUILDS.iter().position(|x| *x == g.guild).unwrap_or(0);
                dict! {
                    "guild" => g.guild,
                    "label" => cartalith_civ::wildlife::WILD_GUILD_LABELS[slot],
                    "biomass_rel" => g.biomass_rel,
                    "species" => &species,
                }
            })
            .collect();
        dict! {
            "id" => rec.id as i64,
            "biome" => rec.biome as i64,
            "biome_name" => cartalith_civ::CART_BIOMES[(rec.biome as usize).saturating_sub(1).min(cartalith_civ::CART_BIOMES.len() - 1)],
            "richness" => rec.richness as i64,
            "area_km2" => rec.area_km2,
            "cells" => rec.cells as i64,
            "summary" => rec.summary.clone(),
            // The popup's own three readouts (reference line 8267): NPP is
            // shown de-normalised back to g/m2/yr, exactly as there.
            "npp" => rec.nppn * 3000.0,
            "tri" => rec.tri,
            "water" => rec.water,
            "lat_abs" => rec.lat_abs,
            "coastal" => rec.coastal,
            "rugged" => rec.ridge_frac >= 0.15,
            "cx" => rec.cx as i64,
            "cy" => rec.cy as i64,
            "guilds" => &guilds,
        }
    }

    /// The world's ecology in one record — `GUI_GAP_REGISTER.md` **WW-14**.
    ///
    /// v3 asks WORLD for an Ecology category, and §37 registered it as
    /// having nothing behind it ("no crate computes either, here or in the
    /// reference"). **That was wrong on both halves.** Ecological
    /// productivity is `cartalith_civ::build_npp`, the Miami model, ported
    /// and golden-verified; fauna distribution is
    /// `cartalith_civ::wildlife`'s ecoregion segmentation with its guild
    /// rosters and per-species population estimates, likewise. Both existed
    /// and neither was reachable from the WORLD rail: NPP was computed only
    /// *inside* `wildlife_regions` and thrown away, and the ecoregion
    /// records were reachable only by clicking the map while the Wildlife
    /// debug view happened to be open.
    ///
    /// This is a summary, not a second copy of that data: the per-region
    /// detail stays `wildlife_region_at`'s, so the dock and the map cannot
    /// disagree about the same world. Empty `Dictionary` when the
    /// civilisation layer's water bodies are missing (a loaded save), the
    /// same condition the Wildlife and Biomes views already report.
    ///
    /// `npp_mean` is over **land only** — averaging a sea of zeroes in
    /// would make the number a function of the sea level rather than of the
    /// ecology.
    #[func]
    fn ecology_summary(&self) -> VarDictionary {
        let Some(f) = self.sample_refs() else {
            return VarDictionary::new();
        };
        let npp = cartalith_civ::build_npp(f.temperature, f.rainfall, f.field, f.sea_level, 3000.0);
        let sea = f.sea_level;
        let mut land = 0usize;
        let mut sum = 0f64;
        let mut max = 0f64;
        for (i, &v) in npp.iter().enumerate() {
            if (f.field[i] as f64) < sea {
                continue;
            }
            land += 1;
            sum += v as f64;
            if (v as f64) > max {
                max = v as f64;
            }
        }
        let mut d = dict! {
            "npp_mean" => if land == 0 { 0.0 } else { sum / land as f64 },
            "npp_max" => max,
            "land_cells" => land as i64,
        };
        let Some(eco) = sample_bridge::wildlife_regions(&f) else {
            // NPP needs nothing but climate, so it is still real here; the
            // ecoregions are what a loaded save cannot have.
            d.set("regions", &Array::<VarDictionary>::new());
            d.set("region_count", 0i64);
            d.set("species_total", 0i64);
            return d;
        };
        let mut regions: Vec<&cartalith_civ::wildlife::Ecoregion> = eco.regions.iter().collect();
        regions.sort_by(|a, b| b.cells.cmp(&a.cells));
        let species_total: usize =
            eco.regions.iter().map(|r| r.guilds.iter().map(|g| g.species.len()).sum::<usize>()).sum();
        let rows: Array<VarDictionary> = regions
            .iter()
            .take(8)
            .map(|r| {
                dict! {
                    "id" => r.id as i64,
                    "biome_name" => cartalith_civ::CART_BIOMES[(r.biome as usize).saturating_sub(1).min(cartalith_civ::CART_BIOMES.len() - 1)],
                    "area_km2" => r.area_km2,
                    "richness" => r.richness as i64,
                    "npp" => r.nppn * 3000.0,
                    "summary" => r.summary.clone(),
                    "cx" => r.cx as i64,
                    "cy" => r.cy as i64,
                }
            })
            .collect();
        d.set("regions", &rows);
        d.set("region_count", eco.regions.len() as i64);
        d.set("species_total", species_total as i64);
        d
    }

    /// Borders, claims and influence as three separate quantities
    /// (`GUI_GAP_REGISTER.md` **CV-23**), aggregated per faction and per
    /// contested faction *pair*.
    ///
    /// **Built on demand, held nowhere** — the owner's own decision for this
    /// row, and the same shape `wildlife_regions()` above already uses.
    /// `transient_bytes` below reports the honest peak working set for *this*
    /// world (53 bytes a cell, itemised at the literal), `resident_bytes` is
    /// `0`, and nothing is added to `CivData`.
    ///
    /// The one input a recompute needs and `compute_civilisation` frees —
    /// `build_travel_cost`'s cost field — is rebuilt from the height field
    /// rather than retained; see `sample_bridge::territory_influence`.
    ///
    /// `{}` before any `generate()`, on a loaded save (no civilisation
    /// layer), and on a world with no capital.
    ///
    /// Returned shape:
    /// - `owned_cells`, `contested_cells` (at or above the frontier
    ///   threshold), `mean_contested`, `mean_influence` — world totals over
    ///   owned land only, since an unowned cell has no influence to average.
    /// - `factions`: one row per faction that owns anything, with its own
    ///   cell count, mean influence, mean contest and frontier cell count.
    /// - `borders`: one row per *pair* of factions that actually meet, with
    ///   how many frontier cells they contest and how evenly. This is the
    ///   "claims" quantity the register asked for: it names who disputes
    ///   whom, which the owner grid alone cannot.
    /// - `frontier_threshold`, `transient_bytes`, `resident_bytes`.
    #[func]
    fn civ_territory_influence(&self) -> VarDictionary {
        let Some(f) = self.sample_refs() else {
            return VarDictionary::new();
        };
        let Some(inf) = sample_bridge::territory_influence(&f) else {
            return VarDictionary::new();
        };
        let n = inf.owner.len();
        let thr = sample_bridge::CONTEST_HATCH_T as f32;
        let roster_len = self.civ.as_ref().map_or(0, |c| c.faction_roster.0.len());
        // Per-faction accumulators, indexed by faction id (0 = Unclaimed,
        // never counted). `+1` rather than `roster_len` alone so a
        // settlement carrying a faction id past the roster's end -- which
        // `civ_remove_faction` is careful to prevent, but which this
        // function must not panic on either way -- is clamped rather than
        // indexing out of bounds.
        let slots = roster_len.max(1);
        let mut cells = vec![0i64; slots];
        let mut frontier = vec![0i64; slots];
        let mut sum_inf = vec![0.0f64; slots];
        let mut sum_con = vec![0.0f64; slots];
        // Frontier cells per unordered faction pair, keyed `(min, max)` so
        // A-contests-B and B-contests-A are one border, not two.
        let mut pairs: std::collections::BTreeMap<(i32, i32), (i64, f64)> = Default::default();
        let (mut owned, mut contested_cells) = (0i64, 0i64);
        let (mut total_inf, mut total_con) = (0.0f64, 0.0f64);
        for i in 0..n {
            let o = inf.owner[i];
            if o <= 0 {
                continue;
            }
            let oi = (o as usize).min(slots - 1);
            let c = inf.contested[i];
            // `influence` is `INFINITY` only where nothing reached the cell,
            // which is exactly `owner == 0` -- guarded anyway rather than
            // trusting it, since an infinite term would poison the mean.
            let v = inf.influence[i];
            owned += 1;
            cells[oi] += 1;
            sum_con[oi] += c as f64;
            total_con += c as f64;
            if v.is_finite() {
                sum_inf[oi] += v as f64;
                total_inf += v as f64;
            }
            if c >= thr {
                contested_cells += 1;
                frontier[oi] += 1;
                let r = inf.rival[i];
                if r > 0 && r != o {
                    let key = (o.min(r), o.max(r));
                    let e = pairs.entry(key).or_insert((0, 0.0));
                    e.0 += 1;
                    e.1 += c as f64;
                }
            }
        }
        let name_of = |id: i32| -> String {
            self.civ
                .as_ref()
                .and_then(|c| c.faction_roster.0.get(id.max(0) as usize))
                .map_or_else(|| format!("Faction {id}"), |e| e.name.clone())
        };
        let factions: Array<VarDictionary> = (1..slots)
            .filter(|&i| cells[i] > 0)
            .map(|i| {
                let k = cells[i] as f64;
                dict! {
                    "id" => i as i64,
                    "name" => name_of(i as i32),
                    "cells" => cells[i],
                    "frontier_cells" => frontier[i],
                    "mean_influence" => sum_inf[i] / k,
                    "mean_contested" => sum_con[i] / k,
                }
            })
            .collect();
        let borders: Array<VarDictionary> = pairs
            .iter()
            .map(|(&(a, b), &(k, s))| {
                dict! {
                    "a" => a as i64,
                    "b" => b as i64,
                    "a_name" => name_of(a),
                    "b_name" => name_of(b),
                    "cells" => k,
                    "mean_contested" => s / k as f64,
                }
            })
            .collect();
        let mut d = dict! {
            "owned_cells" => owned,
            "contested_cells" => contested_cells,
            "mean_contested" => if owned == 0 { 0.0 } else { total_con / owned as f64 },
            "mean_influence" => if owned == 0 { 0.0 } else { total_inf / owned as f64 },
            "frontier_threshold" => sample_bridge::CONTEST_HATCH_T,
            // What the on-demand field actually costs, counted honestly and
            // at its *peak* rather than at its flattering subset. Per cell:
            //
            //   4   the rebuilt `build_travel_cost` f32
            //  24   the sweep — `owner` i32, `best_effective` f64,
            //       `rival_effective` f64, `rival` i32
            //  25   one capital's `road_dijkstra` at a time — `dist` f32,
            //       `prev` i32, `visited` bool, and the binary heap's own
            //       `with_capacity(n)` f64 + usize pair
            //  --
            //  53   bytes per cell, every one of them freed before this
            //       function returns.
            //
            // **41 of those 53 are what `assign_territory` already spends
            // inside `generate()` on this same world** (the same cost field,
            // the same `owner`/`best_effective`, the same Dijkstra) — so
            // opening this layer costs 12 bytes a cell more than the world's
            // own generation already paid, and holds none of it afterwards.
            // Reported rather than asserted so a probe can measure it.
            "transient_bytes" => (n * 53) as i64,
            "resident_bytes" => 0i64,
        };
        d.set("factions", &factions);
        d.set("borders", &borders);
        d
    }

    /// The frame this world's coordinates are in — `GUI_GAP_REGISTER.md`
    /// **WW-15**, and a correction to it.
    ///
    /// The register recorded that "the export writes a plain lon/lat-shaped
    /// frame with no CRS declared". It has always declared one, in the
    /// document's own `note` property (`cartalith_engine::geojson::CRS_NOTE`,
    /// quoted verbatim from the reference so a consumer learns the same thing
    /// from either implementation). RFC 7946 deprecated the `crs` member
    /// outright, so a `note` is the declaration a GeoJSON file gets to make.
    ///
    /// What *was* missing is any way to read the frame **in the app**, which
    /// is what this is. The frame is real and it is two different ones:
    ///
    /// - **World mode** (`world = true`): the grid wraps in X and the rows
    ///   run 90°N to 90°S. That is a plate carrée / equirectangular graticule
    ///   over a whole planet, and `climate.lat_n`/`lat_s` are ignored — the
    ///   climate pipeline's own `lat_of(y)` says so.
    /// - **Regional mode**: rows run `climate.lat_n` to `climate.lat_s` and X
    ///   does not wrap, so latitude is real (the climate model uses it) while
    ///   longitude is not modelled at all — the X axis is planar kilometres
    ///   at the map's own scale.
    ///
    /// What is still absent, and what CRS work would mean, is a *projection*:
    /// nothing reprojects, so the local planar km are not a map projection of
    /// the latitudes beside them, and a consumer that reads them as WGS84
    /// degrees is misreading the file.
    #[func]
    fn world_crs(&self) -> VarDictionary {
        let (gw, gh) = (self.gw.max(0), self.gh.max(0));
        if gw == 0 || gh == 0 {
            return VarDictionary::new();
        }
        let world = self.params.world;
        let cell_km = if self.map_width_km > 0.0 { self.map_width_km / gw as f64 } else { 0.0 };
        vdict! {
            "world" => world,
            "frame" => if world { "equirectangular graticule (plate carrée), X wraps" } else { "local planar, X does not wrap" },
            "lat_n" => if world { 90.0 } else { self.params.climate.lat_n },
            "lat_s" => if world { -90.0 } else { self.params.climate.lat_s },
            "grid_w" => gw as i64,
            "grid_h" => gh as i64,
            "map_width_km" => self.map_width_km,
            "map_height_km" => cell_km * gh as f64,
            "cell_km" => cell_km,
            "deg_per_row" => if gh > 1 {
                (if world { 180.0 } else { (self.params.climate.lat_n - self.params.climate.lat_s).abs() }) / (gh - 1) as f64
            } else { 0.0 },
            // The exact string the GeoJSON document carries, read from the
            // engine rather than transcribed, so the dock and the file cannot
            // drift apart.
            "export_note" => cartalith_engine::geojson::CRS_NOTE,
            "units" => "km",
        }
    }

    #[func]
    fn build_debug_texture(&self, view: GString) -> Option<Gd<ImageTexture>> {
        let f = self.sample_refs()?;
        let bytes = sample_bridge::debug_raster(&f, &view.to_string())?;
        let packed = PackedByteArray::from(bytes);
        let image = Image::create_from_data(self.gw, self.gh, false, Format::RGBA8, &packed)?;
        ImageTexture::create_from_image(&image)
    }
}

/// Plain (non-`#[func]`) glue for the block above -- Rust-internal, so it
/// stays out of the `#[godot_api(secondary)]` block, the same separation
/// every sibling bridge already uses.
impl WorldGen {
    /// Borrows every raster `sample_bridge` reads, straight off this
    /// instance's live state. `None` before any `generate()` and for a
    /// loaded save (whose format carries none of the substrate fields --
    /// `crust_field`, `boundary_type`, `resistance_field` and the rest --
    /// that both the Sample panel and the debug views read; see
    /// `SAVEFILE_COMPAT.md`). **Borrows only. Nothing is copied, nothing is
    /// retained.**
    fn sample_refs(&self) -> Option<sample_bridge::FieldRefs<'_>> {
        let Some(WorldSource::Generated(ws)) = self.source.as_ref() else { return None };
        let (gw, gh) = (self.gw.max(0) as usize, self.gh.max(0) as usize);
        if gw == 0 || gh == 0 {
            return None;
        }
        Some(sample_bridge::FieldRefs {
            gw,
            gh,
            world: self.world,
            sea_level: self.sea_level,
            peak_m: self.params.peak_m,
            map_width_km: self.map_width_km,
            field: &ws.field,
            temperature: &ws.temperature,
            rainfall: &ws.rainfall,
            flow_discharge: &ws.flow_discharge,
            stream_order: ws.stream_order.as_deref(),
            plate_id: &ws.plate_id,
            boundary_mask: &ws.boundary_mask,
            boundary_type: &ws.boundary_type,
            stress_field: &ws.stress_field,
            age_field: &ws.age_field,
            crust_field: &ws.crust_field,
            resistance_field: &ws.resistance_field,
            volcanic_field: &ws.volcanic_field,
            shear_field: &ws.shear_field,
            water_bodies: self.civ.as_ref().map(|c| c.water_bodies.as_slice()),
            territory: self.civ.as_ref().map(|c| c.territory.as_slice()),
            settlements: self.civ.as_ref().map(|c| c.settlements.as_slice()),
            faction_colors: self.civ.as_ref().map_or_else(Vec::new, |c| {
                (0..c.faction_roster.0.len() as i32).map(|f| c.faction_rgb(f)).collect()
            }),
            // The Wind/Ocean-currents debug views' own inputs (layer-
            // visualization audit, `sample_bridge.rs`'s module doc) --
            // `self.params` is only meaningful for a `Generated` source,
            // which this function already required above.
            lat_n: self.params.climate.lat_n,
            lat_s: self.params.climate.lat_s,
            equator_temp: self.params.climate.equator_temp,
            pole_temp: self.params.climate.pole_temp,
            tilt_deg: self.params.planet.axial_tilt_deg,
            rotation_hours: self.params.planet.rotation_hours,
            lapse_rate: self.params.climate.lapse_rate,
            wind_manual: self.params.climate.wind_manual,
            wind_dir_deg: self.params.climate.wind_dir_deg,
            press_k: self.params.climate.press_k,
            current_k: self.params.climate.current_k,
            // The Köppen view needs a whole `WeatherParams`; the Geoid and
            // Tides views need `state.planet.g` and `state.tect.seed`.
            climate: &self.params.climate,
            g: self.params.planet.g,
            seed: self.params.tect.seed,
        })
    }
}

/// `TIMELINE_SCOPE.md` §5 milestone 5 -- the Godot-facing surface for Timeline's manual
/// authoring (`civ_add_year`/`civ_goto_year`/`civ_remove_year`, thin wrappers over
/// `CivData`'s already-built milestone-4 methods above), the ghost/highlight/exist-only
/// overlay's own data source (`civ_year_diff`), and the mechanistic collapse/recovery
/// simulator (`civ_run_collapse_simulation`, `timeline_bridge::run_collapse_simulation`
/// -- the one real new wiring this milestone adds).
///
/// `#[godot_api(secondary)]`, not a plain `#[godot_api]`: only the first `#[godot_api]
/// impl WorldGen` block in the crate may omit `secondary` -- `WorldGen` has a
/// `Base<RefCounted>` field, and a second primary block collides on the shared
/// registration machinery (`E0119`/`E0592`/`E0034`), exactly as every sibling bridge
/// block above already documents.
#[godot_api(secondary)]
impl WorldGen {
    /// `civAddYear` (reference lines 20618-20634) -- see `CivData::civ_add_year`'s own
    /// doc comment for the full semantics. A no-op before any `generate()` call.
    #[func]
    fn civ_add_year(&mut self, year: i64) {
        if let Some(civ) = self.civ.as_mut() {
            civ.civ_add_year(year);
        }
    }

    /// `civGotoYear` (reference lines 20615-20617) -- never touches settlements/ways,
    /// only `territory`. A no-op before any `generate()` call.
    #[func]
    fn civ_goto_year(&mut self, year: i64) {
        if let Some(civ) = self.civ.as_mut() {
            civ.civ_goto_year(year);
        }
    }

    /// `civRemoveYear` (reference lines 20635-20641). A no-op before any `generate()`
    /// call or for a year that was never recorded.
    #[func]
    fn civ_remove_year(&mut self, year: i64) {
        if let Some(civ) = self.civ.as_mut() {
            civ.civ_remove_year(year);
        }
    }

    /// The active timeline cursor (`CivData::year`, reference `civYear`). `0` before
    /// any `generate()`/`civ_add_year` call, matching the reference's own init value.
    #[func]
    fn get_civ_year(&self) -> i64 {
        self.civ.as_ref().map_or(0, |c| c.year)
    }

    /// Every recorded timeline year, ascending -- lets a shell build the pill list
    /// (`_civBuildTimelineUI`, milestone 6's own job) without needing a per-year
    /// getter. Empty before any `generate()`/`civ_add_year` call.
    #[func]
    fn get_civ_timeline_years(&self) -> PackedInt64Array {
        self.civ.as_ref().map_or_else(PackedInt64Array::new, |c| c.timeline.iter().map(|s| s.year).collect())
    }

    /// `_civYearDiff` (reference lines 20580-20595) -- the ghost/highlight/exist-only
    /// overlay's own data source, milestone 6's future consumer. Returns
    /// `{"present": PackedInt64Array, "removed": PackedInt64Array, "added":
    /// PackedInt64Array}`, each ascending (tids, milestone 1's stable ids --
    /// disambiguates "same settlement, renamed" from "different settlement" the way
    /// name/position matching cannot). Empty sets (not an error) before any
    /// `generate()` call or for an unrecorded year -- both legitimate per
    /// `civ_year_diff`'s own doc comment.
    #[func]
    fn civ_year_diff(&self, year: i64) -> VarDictionary {
        let diff = self.civ.as_ref().map(|c| c.civ_year_diff(year)).unwrap_or_default();
        let present: PackedInt64Array = diff.present.iter().map(|&t| t as i64).collect();
        let removed: PackedInt64Array = diff.removed.iter().map(|&t| t as i64).collect();
        let added: PackedInt64Array = diff.added.iter().map(|&t| t as i64).collect();
        vdict! { "present" => &present, "removed" => &removed, "added" => &added }
    }

    /// `_civRunCollapseSimulation` (reference lines 24896-24950) -- the mechanistic
    /// collapse/recovery timeline simulator. `request` keys, all optional --
    /// unrecognised/wrong-typed ones fall back to the reference's own defaults and are
    /// reported in `rejected`, matching `jp_compute`'s own "typo'd key is a bug worth
    /// seeing" policy (see `timeline_bridge::CollapseSimRequest`'s own doc comment for
    /// the full vocabulary and unit conventions):
    ///
    /// * `mode` (String) -- `"collapse"` (default) or `"recovery"`.
    /// * `character` (String) -- `"mixed"` (default)/`"trade"`/`"disease"`/`"conflict"`,
    ///   collapse-mode only.
    /// * `severity` (float, `[0,1]`, default `0.5`) -- collapse-mode only.
    /// * `rate` (float, fraction/year, default `0.01`) -- recovery-mode only.
    /// * `start_year`/`duration`/`step_years` (int, default `0`/`100`/`10`).
    /// * `confirm_overwrite` (bool, default `false`) -- see the `needs_confirm`
    ///   response below.
    ///
    /// Returns one of:
    /// * `{"ok": false, "error": "..."}` -- no generated world, or no settlements to
    ///   simulate (reference's own `alert(...)`, surfaced as an error string here).
    /// * `{"ok": false, "needs_confirm": true, "clobber_years": PackedInt64Array,
    ///   "error": "..."}` -- the reference's own blocking `confirm()` dialog (line
    ///   24911), reported instead of blocked: re-send the SAME request with
    ///   `confirm_overwrite: true` to proceed.
    /// * `{"ok": true, "rejected": PackedStringArray, "steps": int, "end_year": int,
    ///   "died": int, "migrated": int, "unplaced": int, "failed": int, "grew": int,
    ///   "final_settlements": int}` on success -- the timeline cursor is left at
    ///   `end_year` (`CivData::civ_goto_year`, reusing milestone 4's own method, so
    ///   `territory` is already reloaded for that year by the time this returns).
    #[func]
    fn civ_run_collapse_simulation(&mut self, request: VarDictionary) -> VarDictionary {
        let fail = |msg: &str| vdict! { "ok" => false, "error" => msg };
        let (gw, gh) = (self.gw.max(0) as usize, self.gh.max(0) as usize);
        if gw == 0 || gh == 0 {
            return fail("no generated world -- call generate() first");
        }
        let sea_level = self.sea_level;
        let world_wrap = self.world;
        let map_width_km = self.map_width_km;

        let (pairs, mut rejected_vec) = sim_dict_to_pairs(&request);
        let (req, more_rejected) = timeline_bridge::collapse_sim_request_from_pairs(&pairs);
        rejected_vec.extend(more_rejected);
        let rejected: PackedStringArray = rejected_vec.iter().map(GString::from).collect();

        let (Some(WorldSource::Generated(ws)), Some(civ)) = (self.source.as_ref(), self.civ.as_mut()) else {
            return fail("no generated world -- call generate() first (a loaded save carries no civilisation layer)");
        };
        let active_year = civ.year;
        let world = cartalith_civ::timeline::SimulateWorldParams {
            dens: &civ.dens,
            field: &ws.field,
            gw,
            gh,
            sea: sea_level,
            world_wrap,
            map_width_km,
        };
        let outcome =
            timeline_bridge::run_collapse_simulation(&mut civ.timeline, active_year, &civ.settlements, &civ.ways, &civ.territory, &world, &req);
        match outcome {
            timeline_bridge::CollapseSimOutcome::NoSettlements => {
                fail("No settlements to simulate. Auto-populate the world first.")
            }
            timeline_bridge::CollapseSimOutcome::NeedsConfirmation { clobber_years } => {
                let years: PackedInt64Array = clobber_years.iter().copied().collect();
                vdict! {
                    "ok" => false,
                    "needs_confirm" => true,
                    "clobber_years" => &years,
                    "error" => format!(
                        "Simulation will overwrite {} existing timeline year{}.",
                        clobber_years.len(),
                        if clobber_years.len() == 1 { "" } else { "s" }
                    ),
                }
            }
            timeline_bridge::CollapseSimOutcome::Ran(report) => {
                // Reuses milestone 4's own `civ_goto_year` (never duplicated here) --
                // sets the cursor AND reloads `territory` for `end_year` in one call.
                civ.civ_goto_year(report.end_year);
                vdict! {
                    "ok" => true,
                    "rejected" => &rejected,
                    "steps" => i64::from(report.steps),
                    "end_year" => report.end_year,
                    "died" => report.died,
                    "migrated" => report.migrated,
                    "unplaced" => report.unplaced,
                    "failed" => i64::from(report.failed),
                    "grew" => i64::from(report.grew),
                    "final_settlements" => report.final_settlement_count as i64,
                }
            }
        }
    }
}

/// One `Variant` as the four kinds a Timeline sim-panel request uses, or `None` for
/// anything else (an array, a `Vector2`, a null) -- which the caller reports
/// `rejected`, the same as an unknown key. Mirrors `variant_to_jp_value`'s own shape
/// for `journey_bridge::JpValue`.
fn variant_to_sim_value(v: &Variant) -> Option<timeline_bridge::SimValue> {
    match v.get_type() {
        VariantType::INT => Some(timeline_bridge::SimValue::Int(v.to::<i64>())),
        VariantType::FLOAT => Some(timeline_bridge::SimValue::Num(v.to::<f64>())),
        VariantType::STRING => Some(timeline_bridge::SimValue::Str(v.to::<GString>().to_string())),
        VariantType::BOOL => Some(timeline_bridge::SimValue::Bool(v.to::<bool>())),
        _ => None,
    }
}

/// A `Dictionary` as the `(key, SimValue)` list `timeline_bridge`'s own parser takes,
/// plus every key whose value was not a number/string/bool at all. Mirrors
/// `jp_dict_to_pairs`'s own shape.
fn sim_dict_to_pairs(d: &VarDictionary) -> (Vec<(String, timeline_bridge::SimValue)>, Vec<String>) {
    let mut pairs = Vec::new();
    let mut rejected = Vec::new();
    for (k, v) in d.iter_shared() {
        let key = k.to_string();
        match variant_to_sim_value(&v) {
            Some(val) => pairs.push((key, val)),
            None => rejected.push(key),
        }
    }
    (pairs, rejected)
}

/// `ASSET_LIBRARY_SCOPE.md` / `GUI_GAP_REGISTER.md` rows AS-01..AS-08/AS-13,
/// DM-05: the Asset Library authoring session's `#[func]` surface, a thin
/// `Variant`<->Rust layer over `asset_bridge::AssetLibrarySession` --
/// `travel_bridge.rs`'s `tl_*` block just above is the precedent this
/// mirrors (pure logic in a godot-free module, conversion here).
///
/// `#[godot_api(secondary)]`, not a plain `#[godot_api]`: only the first
/// `#[godot_api] impl WorldGen` block in the crate may omit `secondary` --
/// `WorldGen` has a `Base<RefCounted>` field, and a second primary block
/// collides on the shared registration machinery (`E0119`/`E0592`/`E0034`),
/// exactly as every sibling bridge block above already documents.
#[godot_api(secondary)]
impl WorldGen {
    /// Decode `bytes` as a PNG and add it as a new item on `uid` --
    /// `AssetImporter.intake`'s per-file half (AS-01). `{"ok": true}` or
    /// `{"ok": false, "error": ...}` for an unknown uid or bytes that fail
    /// to decode as a PNG.
    #[func]
    fn as_import_item(&mut self, uid: GString, name: GString, bytes: PackedByteArray) -> VarDictionary {
        match self.asset_library.import_item(&uid.to_string(), name.to_string(), bytes.as_slice()) {
            Ok(()) => vdict! { "ok" => true, "error" => "" },
            Err(e) => vdict! { "ok" => false, "error" => e },
        }
    }

    /// Add (or return the existing) custom slot -- `AssetDB::addCustomSlot`
    /// (AS-01, AS-12's own "Unassigned imports" note: this is the real
    /// engine call a future such bucket would sit on top of). `set_name`
    /// empty means the reference's own `"Default"` fallback.
    #[func]
    fn as_add_custom_slot(&mut self, name: GString, set_name: GString) -> VarDictionary {
        let set = set_name.to_string();
        let set_opt = if set.trim().is_empty() { None } else { Some(set.as_str()) };
        let uid = self.asset_library.add_custom_slot(&name.to_string(), set_opt);
        vdict! { "ok" => true, "uid" => uid.as_str() }
    }

    /// Every slot in `family_key`'s registry, in the same order
    /// `AssetDB::slots_in_family` returns (frozen vocabulary order for the
    /// seven closed families, add order for `"custom"`, which starts empty
    /// every session until something is imported/duplicated into it) --
    /// AS-08's real per-slot fill state and AS-13's "filled slots" readout,
    /// both sourced from here. Each row: `uid`, `id`, `name`, `item_count`,
    /// `filled` (bool), `has_dupe` (bool, `AssetValidator.slotHasDupe`),
    /// `set` (empty string for the reference's own "Default").
    /// Empty `Array` for an unrecognised `family_key`.
    #[func]
    fn as_family_slots(&self, family_key: GString) -> Array<VarDictionary> {
        let Some(family) = cartalith_assets::Family::from_key(&family_key.to_string()) else {
            return Array::new();
        };
        let db = &self.asset_library.db;
        db.slots_in_family(family)
            .into_iter()
            .map(|slot| {
                let count = db.items(&slot.uid).len();
                vdict! {
                    "uid" => slot.uid.as_str(),
                    "id" => slot.id.as_str(),
                    "name" => slot.name.as_str(),
                    "item_count" => count as i64,
                    "filled" => count > 0,
                    "has_dupe" => cartalith_assets::slot_has_dupe(db, &slot.uid),
                    // AS-12's "Unassigned imports" bucket: a custom slot's own
                    // `set` (empty for the reference's "Default"), so a rail
                    // section can filter to one reserved set name without a
                    // second engine call per slot.
                    "set" => slot.set.clone().unwrap_or_default().as_str(),
                }
            })
            .collect()
    }

    /// One slot's full inspector detail (AS-07): id/name/family/set, tags,
    /// collection membership, and the free-form `SlotMeta` fields.
    /// `{"ok": false}` for an unknown uid.
    #[func]
    fn as_slot_summary(&self, uid: GString) -> VarDictionary {
        let uid_s = uid.to_string();
        let db = &self.asset_library.db;
        let Some(slot) = db.get(&uid_s) else { return vdict! { "ok" => false } };
        let tags: PackedStringArray = slot.meta.tags.iter().map(GString::from).collect();
        let collections: PackedStringArray = db.collections.membership(&uid_s).iter().map(|s| GString::from(*s)).collect();
        vdict! {
            "ok" => true,
            "id" => slot.id.as_str(),
            "name" => slot.name.as_str(),
            "family" => slot.family.key(),
            "set" => slot.set.clone().unwrap_or_default().as_str(),
            "item_count" => db.items(&uid_s).len() as i64,
            "has_dupe" => cartalith_assets::slot_has_dupe(db, &uid_s),
            "tags" => &tags,
            "collections" => &collections,
            "meta_author" => slot.meta.author.as_str(),
            "meta_copyright" => slot.meta.copyright.as_str(),
            "meta_license" => slot.meta.license.as_str(),
            "meta_source" => slot.meta.source.as_str(),
            "meta_notes" => slot.meta.notes.as_str(),
            "meta_version" => slot.meta.version.as_str(),
        }
    }

    /// One item's inspector detail (AS-07): name, transform (`scale`/
    /// `pan_x`/`pan_y`), decoded size, and its content hash. `{"ok": false}`
    /// for an unknown uid/index.
    #[func]
    fn as_item_summary(&self, uid: GString, index: i32) -> VarDictionary {
        if index < 0 {
            return vdict! { "ok" => false };
        }
        let uid_s = uid.to_string();
        let idx = index as usize;
        let Some(item) = self.asset_library.db.items(&uid_s).get(idx) else {
            return vdict! { "ok" => false };
        };
        let (w, h) = self.asset_library.image(&uid_s, idx).map(|img| (img.w, img.h)).unwrap_or((0, 0));
        vdict! {
            "ok" => true,
            "name" => item.name.as_str(),
            "scale" => item.transform.scale,
            "pan_x" => item.transform.pan_x,
            "pan_y" => item.transform.pan_y,
            "w" => w as i64,
            "h" => h as i64,
            "hash" => item.hash.as_str(),
        }
    }

    /// Directly write one item's scale/pan transform (AS-07): the reference's
    /// `alScale` slider and `ImageEditor`'s drag-to-pan (`E('alScale').oninput`
    /// / `ImageEditor.attach`'s `onpointermove`, both around line 27346),
    /// collapsed onto the single write the engine side needs -- the caller
    /// supplies whatever combination of scale/pan_x/pan_y it just changed.
    /// `false` for an unknown uid/index.
    #[func]
    fn as_set_item_transform(
        &mut self,
        uid: GString,
        index: i32,
        scale: f64,
        pan_x: f64,
        pan_y: f64,
    ) -> bool {
        if index < 0 {
            return false;
        }
        let Some(item) = self
            .asset_library
            .db
            .item_mut(&uid.to_string(), index as usize)
        else {
            return false;
        };
        item.transform = cartalith_assets::ItemTransform {
            scale,
            pan_x,
            pan_y,
        };
        true
    }

    /// Reset one item's transform to identity, optionally re-fitting it to the
    /// slot's family (AS-07's Fit/Reset buttons) -- `defaultTransform()` plus,
    /// for a bottom-anchored family when `fit` is true, `fitToBottom`
    /// (reference `alFit`/`alReset`, line 27347-27348). Returns the resulting
    /// transform so the UI never recomputes it. `{"ok": false}` for an
    /// unknown uid/index.
    #[func]
    fn as_reset_item_transform(&mut self, uid: GString, index: i32, fit: bool) -> VarDictionary {
        if index < 0 {
            return vdict! { "ok" => false };
        }
        let uid_s = uid.to_string();
        let idx = index as usize;
        let Some(slot) = self.asset_library.db.get(&uid_s).cloned() else {
            return vdict! { "ok" => false };
        };
        let dims = self
            .asset_library
            .image(&uid_s, idx)
            .map(|img| (img.w, img.h));
        let mut t = cartalith_assets::ItemTransform::default();
        if fit
            && slot.family.anchor() == cartalith_assets::Anchor::Bottom
            && let Some((w, h)) = dims
        {
            cartalith_assets::fit_to_bottom(&mut t, w, h, slot.family.size());
        }
        let Some(item) = self.asset_library.db.item_mut(&uid_s, idx) else {
            return vdict! { "ok" => false };
        };
        item.transform = cartalith_assets::ItemTransform {
            scale: t.scale,
            pan_x: t.pan_x,
            pan_y: t.pan_y,
        };
        vdict! { "ok" => true, "scale" => t.scale, "pan_x" => t.pan_x, "pan_y" => t.pan_y }
    }

    /// A `render_item`-baked, PNG-encoded thumbnail for one stored item
    /// (AS-08's real per-slot art, replacing the grid's checkerboard
    /// placeholder). Empty `PackedByteArray` when the uid/index doesn't
    /// resolve.
    #[func]
    fn as_thumbnail_png(&self, uid: GString, index: i32, size: i32) -> PackedByteArray {
        if index < 0 || size <= 0 {
            return PackedByteArray::new();
        }
        match self.asset_library.thumbnail_png(&uid.to_string(), index as usize, size as u32) {
            Some(bytes) => PackedByteArray::from(bytes),
            None => PackedByteArray::new(),
        }
    }

    /// Pack-level metadata and totals (AS-13's "Active pack" header):
    /// name/author/license plus `total_items`/`total_slots_filled`.
    #[func]
    fn as_pack_info(&self) -> VarDictionary {
        let db = &self.asset_library.db;
        vdict! {
            "name" => db.pack.name.as_str(),
            "author" => db.pack.author.as_str(),
            "license" => db.pack.license.as_str(),
            "total_items" => db.total_items() as i64,
        }
    }

    /// Set the pack's name/author/license fields directly (AS-13's "Pack
    /// metadata…"). Always succeeds -- these are free-form strings with no
    /// validation on the reference's own side either.
    #[func]
    fn as_set_pack_info(&mut self, name: GString, author: GString, license: GString) -> bool {
        self.asset_library.db.pack.name = name.to_string();
        self.asset_library.db.pack.author = author.to_string();
        self.asset_library.db.pack.license = license.to_string();
        true
    }

    /// Remove one item from a slot, keeping the pixel store in lockstep.
    #[func]
    fn as_remove_item(&mut self, uid: GString, index: i32) -> bool {
        index >= 0 && self.asset_library.remove_item(&uid.to_string(), index as usize)
    }

    /// Reset the whole session to a fresh, empty library (AS-03).
    #[func]
    fn as_clear_library(&mut self) -> bool {
        self.asset_library.clear();
        true
    }

    /// `AssetValidator.run()` (AS-05): the real, ordered warning strings.
    #[func]
    fn as_validate(&self) -> PackedStringArray {
        self.asset_library.validate().iter().map(GString::from).collect()
    }

    /// Bake every stored item, build a schema-2 manifest, and write the pack
    /// `.zip` bytes (AS-04, DM-05). `{"ok": false, "error": ...}` on an
    /// empty library or an encode/archive failure; otherwise `{"ok": true,
    /// "name": <pack name>, "bytes": PackedByteArray}`. The caller (GDScript)
    /// writes `bytes` to disk via `FileAccess` -- this crate's own convention
    /// for export surfaces (`region_export_tiles` does the same) is to hand
    /// back bytes rather than touch a user-chosen path from Rust.
    #[func]
    fn as_export_pack_bytes(&mut self) -> VarDictionary {
        match self.asset_library.export_pack_bytes() {
            Ok((name, bytes)) => {
                let packed = PackedByteArray::from(bytes);
                vdict! { "ok" => true, "error" => "", "name" => name.as_str(), "bytes" => &packed }
            }
            Err(e) => {
                let packed = PackedByteArray::new();
                vdict! { "ok" => false, "error" => e, "name" => "", "bytes" => &packed }
            }
        }
    }

    /// `AssetLibrary.applyToMap()` (AS-02): compile the current session into
    /// a pack (same bake `as_export_pack_bytes` does) and load it straight
    /// into `self.asset_pack` -- the reference's own `loadAssetPack(blob)`
    /// call, minus the round trip through a file. `{"ok": false, "error":
    /// ...}` on an empty library or a decode failure; `{"ok": true}` once
    /// the map is rendering these sprites/textures.
    #[func]
    fn as_apply_to_map(&mut self) -> VarDictionary {
        let (_, bytes) = match self.asset_library.export_pack_bytes() {
            Ok(pair) => pair,
            Err(e) => return vdict! { "ok" => false, "error" => e },
        };
        match pack::load_pack_from_bytes(bytes) {
            Ok(loaded) => {
                self.asset_pack = Some(loaded);
                vdict! { "ok" => true, "error" => "" }
            }
            Err(e) => vdict! { "ok" => false, "error" => e },
        }
    }

    /// `alBatchTag` (AS-06): comma-separated `tags_csv` onto every uid in
    /// `uids`.
    #[func]
    fn as_batch_tag(&mut self, uids: PackedStringArray, tags_csv: GString) -> VarDictionary {
        let uid_vec: Vec<String> = uids.as_slice().iter().map(GString::to_string).collect();
        let tags: Vec<String> = tags_csv.to_string().split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
        let tagged = self.asset_library.batch_tag(&uid_vec, &tags);
        vdict! { "ok" => true, "tagged" => tagged as i64 }
    }

    /// `alBatchColl` (AS-06): add every uid in `uids` to collection `name`.
    #[func]
    fn as_batch_collect(&mut self, uids: PackedStringArray, name: GString) -> VarDictionary {
        let uid_vec: Vec<String> = uids.as_slice().iter().map(GString::to_string).collect();
        self.asset_library.batch_collect(&name.to_string(), &uid_vec);
        vdict! { "ok" => true }
    }

    /// `alBatchColl`'s read side (AS-12): every collection currently defined
    /// on this session, in creation order (`AssetCollections::as_map`,
    /// itself an `OrderedMap` for exactly this reason -- reproducible order,
    /// not a `HashMap`'s arbitrary one), each with its member uid list.
    /// Powers the Collections rail row (`asset_library_window.gd`) -- until
    /// now `as_slot_summary`'s `collections` field could only answer "which
    /// collections is THIS uid in", with nothing to enumerate every
    /// collection that exists. Empty before any `as_batch_collect` call.
    #[func]
    fn as_collections(&self) -> Array<VarDictionary> {
        self.asset_library
            .db
            .collections
            .as_map()
            .iter()
            .map(|(name, uids)| {
                let uid_arr: PackedStringArray = uids.iter().map(GString::from).collect();
                vdict! { "name" => name, "uids" => &uid_arr }
            })
            .collect()
    }

    /// `alBatchRename` (AS-06): `{base}_01`, `{base}_02`, … over `uids` in
    /// order. `remap` carries `old_uid -> new_uid` for every custom slot
    /// whose uid changed (a caller's selection set needs to follow it) --
    /// frozen slots rename their item variants in place and keep their uid.
    #[func]
    fn as_batch_rename(&mut self, uids: PackedStringArray, base: GString) -> VarDictionary {
        let uid_vec: Vec<String> = uids.as_slice().iter().map(GString::to_string).collect();
        let (renamed, remap) = self.asset_library.batch_rename(&uid_vec, &base.to_string());
        let mut remap_dict = VarDictionary::new();
        for (old, new) in &remap {
            remap_dict.set(old.as_str(), new.as_str());
        }
        vdict! { "ok" => true, "renamed" => renamed as i64, "remap" => &remap_dict }
    }

    /// `alBatchDup` (AS-06): clone every slot in `uids` carrying at least
    /// one item into a new custom slot under `"Duplicates"`.
    #[func]
    fn as_batch_duplicate(&mut self, uids: PackedStringArray) -> VarDictionary {
        let uid_vec: Vec<String> = uids.as_slice().iter().map(GString::to_string).collect();
        let made = self.asset_library.batch_duplicate(&uid_vec);
        vdict! { "ok" => true, "made" => made as i64 }
    }

    /// `alBatchDel` (AS-06): custom slots in `uids` are removed entirely;
    /// frozen slots have their items cleared (a frozen slot can never be
    /// removed).
    #[func]
    fn as_batch_delete(&mut self, uids: PackedStringArray) -> VarDictionary {
        let uid_vec: Vec<String> = uids.as_slice().iter().map(GString::to_string).collect();
        let deleted = self.asset_library.batch_delete(&uid_vec);
        vdict! { "ok" => true, "deleted" => deleted as i64 }
    }

    // -- Sprite-sheet slicer (AS-09/AS-10/AS-11) --------------------------

    /// `SpriteSheetImporter.loadSheet` (AS-09): decode a sprite sheet and
    /// hold it on the session for slicing. `{"ok": true, "w", "h", "name"}`,
    /// or `{"ok": false, "error": ...}` for bytes that don't decode as a PNG.
    /// PNG only — `cartalith-assets` compiles `image` with the `png` codec
    /// alone, so this reports the limit rather than pretending otherwise.
    #[func]
    fn as_load_sheet(&mut self, name: GString, bytes: PackedByteArray) -> VarDictionary {
        match self.asset_library.load_sheet(name.to_string(), bytes.as_slice()) {
            Ok((w, h)) => vdict! { "ok" => true, "error" => "", "w" => w as i64, "h" => h as i64, "name" => &name },
            Err(e) => vdict! { "ok" => false, "error" => e, "w" => 0, "h" => 0, "name" => "" },
        }
    }

    /// Drop the loaded sheet (the slicer modal closing). Always succeeds.
    #[func]
    fn as_clear_sheet(&mut self) -> bool {
        self.asset_library.clear_sheet();
        true
    }

    /// The real `N cells detected · M non-empty` detection pass (AS-09) —
    /// the same crop, chroma key and `isBlank` threshold the slice itself
    /// would use, not a sample — plus the grid lines to draw it against.
    /// `{"ok": false, "error"}` with no sheet loaded or a margin that leaves
    /// no room; otherwise `{"ok": true, "total", "non_empty", "usable",
    /// "col_x0"/"col_x1"/"row_y0"/"row_y1"}`, where `usable` false is the
    /// reference's own "Grid too dense" state.
    ///
    /// The four span arrays are in **sheet pixels**, one entry per column or
    /// row, and exist so the overlay never has to reimplement
    /// `computeCells`'s half-gutter arithmetic in GDScript and drift from
    /// what the slice actually cuts.
    ///
    /// `col_lines_px`/`row_lines_px` (AS-17) are the *undisplaced* division
    /// lines behind those spans -- a drag handle's hit-test target, not the
    /// gutter-narrowed cell edge `col_x0`/`col_x1` are.
    #[func]
    fn as_slice_preview(&self, opts: VarDictionary) -> VarDictionary {
        let params = slice_params_from(&opts);
        match self.asset_library.slice_preview(&params) {
            Ok(p) => {
                let col_x0: PackedFloat64Array = p.col_spans.iter().map(|s| s.0).collect();
                let col_x1: PackedFloat64Array = p.col_spans.iter().map(|s| s.1).collect();
                let row_y0: PackedFloat64Array = p.row_spans.iter().map(|s| s.0).collect();
                let row_y1: PackedFloat64Array = p.row_spans.iter().map(|s| s.1).collect();
                let blank: PackedInt32Array = p.counts.blank.iter().map(|&i| i as i32).collect();
                let col_lines_px = PackedFloat64Array::from(p.col_lines_px);
                let row_lines_px = PackedFloat64Array::from(p.row_lines_px);
                vdict! {
                    "ok" => true, "error" => "",
                    "total" => p.counts.total as i64,
                    "non_empty" => p.counts.non_empty as i64,
                    "usable" => p.counts.usable,
                    "blank" => &blank,
                    "col_x0" => &col_x0, "col_x1" => &col_x1,
                    "row_y0" => &row_y0, "row_y1" => &row_y1,
                    "col_lines_px" => &col_lines_px, "row_lines_px" => &row_lines_px,
                }
            }
            Err(e) => {
                let empty = PackedFloat64Array::new();
                let blank = PackedInt32Array::new();
                vdict! {
                    "ok" => false, "error" => e,
                    "total" => 0, "non_empty" => 0, "usable" => false, "blank" => &blank,
                    "col_x0" => &empty, "col_x1" => &empty,
                    "row_y0" => &empty, "row_y1" => &empty,
                    "col_lines_px" => &empty, "row_lines_px" => &empty,
                }
            }
        }
    }

    /// AS-17: move interior line `index` of `lines` to `frac` (a fraction of
    /// the grid rect's own span, matching `cartalith_assets::SliceGrid`'s own
    /// `col_lines`/`row_lines` units) -- `cartalith_assets::move_line`,
    /// exposed directly since the clamp-so-lines-never-cross-their-neighbours
    /// rule is real engine logic, not something a drag handler should
    /// reimplement in GDScript. `lines` unchanged for `index <= 0` or
    /// `index >= lines.size() - 1` (the outer edges, which the grid rect's
    /// own margin owns, not a line).
    #[func]
    fn as_slicer_move_line(
        &self,
        lines: PackedFloat64Array,
        index: i32,
        frac: f64,
    ) -> PackedFloat64Array {
        let mut v: Vec<f64> = lines.as_slice().to_vec();
        if index >= 0 {
            cartalith_assets::move_line(&mut v, index as usize, frac);
        }
        PackedFloat64Array::from(v)
    }

    /// The uniform `n+1`-line array `cartalith_assets::SliceGrid` falls back
    /// to on its own (`resetLines`'s own construction) -- exposed so a drag
    /// handler always starts from the exact array the engine would have used
    /// implicitly, rather than recomputing `i/n` in GDScript. Empty for
    /// `n < 1`.
    #[func]
    fn as_uniform_lines(&self, n: i32) -> PackedFloat64Array {
        if n < 1 {
            return PackedFloat64Array::new();
        }
        PackedFloat64Array::from(cartalith_assets::uniform_lines(n as u32))
    }

    /// `addSlices()` (AS-09/AS-10/AS-11): slice the loaded sheet and land the
    /// cells in the library. Non-destructive — the sheet stays loaded and
    /// untouched, so the same sheet can be re-sliced with different settings.
    ///
    /// `opts` carries the grid (`cols`, `rows`, `margin`, `spacing`), the
    /// pixel toggles (`trim`, `skip_empty`, and `chroma`/`chroma_r`/
    /// `chroma_g`/`chroma_b`/`chroma_tol`), and the target: `target` is one of
    /// `"slot"` (+`uid`), `"new_custom"` (+`name`, `set`), `"per_cell"`
    /// (+`set`), or `"family"` (+`family`, `overwrite`). Anything
    /// unrecognised, a missing sheet, an impossible margin or a too-dense
    /// grid all come back as `{"ok": false, "error": ...}` — never a panic
    /// across the boundary.
    #[func]
    fn as_slice_apply(&mut self, opts: VarDictionary) -> VarDictionary {
        let params = slice_params_from(&opts);
        let target = match slice_target_from(&opts) {
            Ok(t) => t,
            Err(e) => return slice_err(e),
        };
        match self.asset_library.apply_slice(&params, &target) {
            Ok(o) => {
                let uids: PackedStringArray = o.uids.iter().map(GString::from).collect();
                vdict! {
                    "ok" => true, "error" => "",
                    "added" => o.added as i64,
                    "skipped_blank" => o.skipped_blank as i64,
                    "unplaced" => o.unplaced as i64,
                    "uids" => &uids,
                }
            }
            Err(e) => slice_err(e),
        }
    }
}

/// `{"ok": false, ...}` with every field `as_slice_apply` promises, so a
/// GDScript caller never has to guard a key lookup on the failure path.
fn slice_err(error: String) -> VarDictionary {
    let uids = PackedStringArray::new();
    vdict! {
        "ok" => false, "error" => error,
        "added" => 0, "skipped_blank" => 0, "unplaced" => 0, "uids" => &uids,
    }
}

/// Read the slicer modal's controls out of a `Dictionary`. Every field has a
/// defined default, and `cartalith_assets::SliceGrid::new`/`GridRect::inset`
/// apply the reference's own clamps on top, so no combination of missing or
/// nonsensical keys can reach the engine as a bad grid.
fn slice_params_from(opts: &VarDictionary) -> asset_bridge::SliceParams {
    let int_of = |k: &str, d: i64| opts.get(k).and_then(|v| v.try_to::<i64>().ok()).unwrap_or(d);
    let f_of = |k: &str, d: f64| opts.get(k).and_then(|v| v.try_to::<f64>().ok()).unwrap_or(d);
    let b_of = |k: &str, d: bool| opts.get(k).and_then(|v| v.try_to::<bool>().ok()).unwrap_or(d);
    let chroma = b_of("chroma", false).then(|| cartalith_assets::ChromaKey {
        color: [
            int_of("chroma_r", 255).clamp(0, 255) as u8,
            int_of("chroma_g", 255).clamp(0, 255) as u8,
            int_of("chroma_b", 255).clamp(0, 255) as u8,
        ],
        tol: f_of("chroma_tol", 40.0).max(0.0),
    });
    // AS-17: `col_lines`/`row_lines`, a `PackedFloat64Array` per dragged grid
    // (absent/empty means "no override" -- `cartalith_assets::SliceGrid`'s
    // own uniform default takes over); `only_cell`, the flat cell index a
    // click-to-select picked, `-1`/absent meaning "the whole grid" the
    // reference always sliced.
    let lines_of = |k: &str| -> Option<Vec<f64>> {
        opts.get(k)
            .and_then(|v| v.try_to::<PackedFloat64Array>().ok())
            .filter(|a| !a.is_empty())
            .map(|a| a.as_slice().to_vec())
    };
    let only_cell = opts
        .get("only_cell")
        .and_then(|v| v.try_to::<i64>().ok())
        .filter(|&i| i >= 0)
        .map(|i| i as usize);
    asset_bridge::SliceParams {
        cols: int_of("cols", 1),
        rows: int_of("rows", 1),
        margin: f_of("margin", 0.0),
        spacing: f_of("spacing", 0.0),
        chroma,
        trim: b_of("trim", false),
        // `#alSlSkip` ships checked in the reference, so that is the default
        // a caller who omits the key gets.
        skip_blank: b_of("skip_empty", true),
        col_lines: lines_of("col_lines"),
        row_lines: lines_of("row_lines"),
        only_cell,
    }
}

/// Read the slicer's target selection out of the same `Dictionary`. An
/// unrecognised `target`, an unknown family key, or a `"new_custom"` with a
/// blank name is a real error string rather than a silent fallback -- landing
/// a slice somewhere the caller did not ask for would be worse than refusing.
fn slice_target_from(opts: &VarDictionary) -> Result<asset_bridge::SliceTarget, String> {
    let s_of = |k: &str| opts.get(k).map(|v| v.to_string()).unwrap_or_default();
    let kind = s_of("target");
    match kind.as_str() {
        "slot" => {
            let uid = s_of("uid");
            if uid.trim().is_empty() {
                return Err("Pick a target slot.".to_string());
            }
            Ok(asset_bridge::SliceTarget::Slot { uid })
        }
        "new_custom" => {
            let name = s_of("name");
            if name.trim().is_empty() {
                return Err("Type a name for the new custom icon.".to_string());
            }
            Ok(asset_bridge::SliceTarget::NewCustom { name, set: s_of("set") })
        }
        "per_cell" => Ok(asset_bridge::SliceTarget::PerCell { set: s_of("set") }),
        "family" => {
            let key = s_of("family");
            let Some(family) = cartalith_assets::Family::from_key(&key) else {
                return Err(format!("no such family: {key}"));
            };
            let overwrite = opts.get("overwrite").and_then(|v| v.try_to::<bool>().ok()).unwrap_or(false);
            Ok(asset_bridge::SliceTarget::Family { family, overwrite })
        }
        other => Err(format!("unrecognised slice target: {other}")),
    }
}

/// The civ **roster and place-editor** surface — `PARITY_AUDIT.md` §5
/// items 2, 3, 4, 7, 9, 10 and 12, and `GUI_GAP_REGISTER.md`
/// CV-01/CV-07/MS-13/ED-03.
///
/// Before this block `civ_drop_settlement` *created* a settlement and
/// nothing edited, moved or deleted one — the audit's own words, "a live
/// usability hole, not just an inventory gap: a user can add a settlement
/// they can never fix or undo." The state these methods drive lives in
/// `civ_roster_bridge`; see that module's doc comment for the two design
/// calls (why the roster is boundary state, and why place edits are keyed
/// by `tid`).
///
/// `secondary` for the same compile-time reason every other extra block in
/// this file is: gdext allows exactly one primary `#[godot_api] impl
/// WorldGen`, and that one is already spoken for.
///
/// **POI is still not here, deliberately.** `civ_tools_bridge.rs`'s module
/// doc and `GUI_GAP_REGISTER.md` CV-01 both record that POI "is not a
/// ported concept" — `cartalith-civ` models Settlement and Territory only,
/// and the omission is explicit rather than an oversight. Nothing in this
/// block reverses it: the place editor's Category selector
/// (settlement ↔ POI) has no port, and the context menu's "Drop POI here"
/// op is absent for the same stated reason.
#[godot_api(secondary)]
impl WorldGen {
    // ---- vocabularies: no `generate()` call required ----

    /// `CIV_TRAITS` (reference line 14715) as `{key, label, glyph}` — the
    /// place editor's trait pills. Never reorder; the reference writes
    /// these keys into save files.
    #[func]
    fn civ_trait_vocabulary(&self) -> Array<VarDictionary> {
        cartalith_civ::roster::CIV_TRAITS
            .iter()
            .map(|&(key, label, glyph)| vdict! { "key" => key, "label" => label, "glyph" => glyph })
            .collect()
    }

    /// `CIV_SPECIALISATIONS` (reference 14729) as `{key, label}`.
    #[func]
    fn civ_specialisation_vocabulary(&self) -> Array<VarDictionary> {
        key_label_array(&cartalith_civ::roster::CIV_SPECIALISATIONS)
    }

    /// `CIV_RELIGIONS` as `{key, label}`.
    #[func]
    fn civ_religion_vocabulary(&self) -> Array<VarDictionary> {
        key_label_array(&cartalith_civ::roster::CIV_RELIGIONS)
    }

    /// `CIV_GOVERNMENTS` as `{key, label}`.
    #[func]
    fn civ_government_vocabulary(&self) -> Array<VarDictionary> {
        key_label_array(&cartalith_civ::roster::CIV_GOVERNMENTS)
    }

    /// `AG_TECH_LEVELS` as `{key, label, hint, farmers_per_urbanite}`.
    ///
    /// `farmers_per_urbanite` is reported honestly and consumed by nothing:
    /// its only readers in the reference are `_civFoodShed`/
    /// `foodSurplusRatio`, neither of which is ported, so in this port
    /// ag-tech is as inert as Government and Religion already are in the
    /// reference itself. See `cartalith_civ::roster`'s module doc.
    #[func]
    fn civ_ag_tech_vocabulary(&self) -> Array<VarDictionary> {
        cartalith_civ::roster::AG_TECH_LEVELS
            .iter()
            .map(|t| {
                vdict! {
                    "key" => t.key,
                    "label" => t.label,
                    "hint" => t.hint,
                    "farmers_per_urbanite" => t.farmers_per_urbanite,
                }
            })
            .collect()
    }

    /// `CIV_CULTURES`' seven keys (reference 14607) — the naming-culture
    /// picker's vocabulary. Label-free: the reference's own table carries
    /// no display label for a culture, only a key, and inventing one here
    /// would be a second source of truth for the shell to drift from.
    #[func]
    fn civ_culture_vocabulary(&self) -> PackedStringArray {
        cartalith_civ::CIV_CULTURES.iter().map(|c| GString::from(c.key)).collect()
    }

    // ---- roster (CV-07 / MS-13) ----

    /// How many real, assignable factions exist right now (`1..=n`),
    /// excluding "Unclaimed". `0` before any `generate()` call.
    ///
    /// This is what `CIV_FACTION_COUNT` used to be the only answer to. It
    /// still seeds the roster, but it is no longer the whole truth:
    /// `civ_add_faction`/`civ_remove_faction` move this number.
    #[func]
    fn civ_faction_count(&self) -> i64 {
        self.civ.as_ref().map_or(0, |c| c.faction_roster.count() as i64)
    }

    /// `_civAddFaction` (reference 14644): appends one faction with the
    /// reference's own defaults — name `"Faction N"`, `_civFactionColor(N)`,
    /// `_civDefaultCulture(N)`, religion `none`, government `monarchy`,
    /// ag-tech `traditionalAgrarian`. Returns its id, or `-1` before any
    /// `generate()` call.
    ///
    /// The new faction owns nothing until something is assigned to it — the
    /// reference behaves identically. Paint it territory with the Territory
    /// tool, or set a settlement's Polity to it in the place editor.
    #[func]
    fn civ_add_faction(&mut self) -> i64 {
        match self.civ.as_mut() {
            Some(civ) => civ.faction_roster.add() as i64,
            None => -1,
        }
    }

    /// `_civRemoveFaction` (reference 14657): drops the **last** faction —
    /// the reference removes by index-from-the-end only, and so does this —
    /// reverting every settlement and every territory cell that used it to
    /// Unclaimed rather than leaving a dangling index.
    ///
    /// Returns `false`, changing nothing, at the reference's own floor
    /// (Unclaimed plus one real faction) or before any `generate()` call.
    /// The caller owns the confirmation prompt, the same split
    /// `civ_run_collapse_simulation`'s `needs_confirm` uses — the
    /// reference's own button asks first, and so should the shell.
    #[func]
    fn civ_remove_faction(&mut self) -> bool {
        let Some(civ) = self.civ.as_mut() else { return false };
        let CivData { faction_roster, settlements, territory, .. } = civ;
        faction_roster.remove_last(settlements, territory)
    }

    /// Writes one editable faction field — `key` is `"name"`, `"culture"`,
    /// `"religion"`, `"government"` or `"ag_tech"`, exactly the five
    /// `_civPopulateFactionEditor` (reference 16247) makes editable.
    ///
    /// Returns `false` and changes nothing for an unknown faction, an
    /// unknown key, a blank name, or a value outside that field's own
    /// reference vocabulary — a typo from GDScript is rejected, not stored.
    #[func]
    fn civ_set_faction_field(&mut self, faction: i64, key: GString, value: GString) -> bool {
        let Some(civ) = self.civ.as_mut() else { return false };
        if faction < 0 {
            return false;
        }
        civ.faction_roster.set_field(faction as usize, &key.to_string(), &value.to_string())
    }

    /// `state.viz.territoryOpacity` — how heavily the territory wash is laid
    /// over the map, `0..1` (`GUI_GAP_REGISTER.md` **CA-17**, the reference's
    /// `#territoryOpacityR`). A **negative** value restores the default
    /// ([`TERRITORY_ALPHA_DEFAULT`]), which is how the dock's Reset row
    /// spells itself without a second binding.
    ///
    /// Takes effect on the next `build_territory_texture()`; nothing is
    /// invalidated here, the same contract every other display setting has.
    #[func]
    fn set_territory_opacity(&mut self, a: f64) {
        self.territory_opacity = if a < 0.0 { TERRITORY_ALPHA_DEFAULT } else { a.clamp(0.0, 1.0) };
    }

    #[func]
    fn territory_opacity(&self) -> f64 {
        self.territory_opacity
    }

    /// The default the dock's Reset row returns to, so it can label itself
    /// with the number rather than transcribing it.
    #[func]
    fn territory_opacity_default(&self) -> f64 {
        TERRITORY_ALPHA_DEFAULT
    }

    /// Sets faction `faction`'s **identity colour** —
    /// `GUI_GAP_REGISTER.md` **CV-21**, and v3's "CIVIL owns the colour,
    /// CARTO owns the paint" split at the CIVIL end.
    ///
    /// Every renderer that draws a faction reads this: the territory wash
    /// (`build_territory_texture`), the Political-control analysis field,
    /// and the roster/banner swatch `get_factions` reports. Returns `false`
    /// and changes nothing for an unknown faction and for faction `0`
    /// ("Unclaimed", which nothing renders).
    ///
    /// The caller is expected to re-ask for the territory texture
    /// afterwards; nothing is invalidated here, the same contract every
    /// other roster edit already has.
    #[func]
    fn civ_set_faction_color(&mut self, faction: i64, r: i64, g: i64, b: i64) -> bool {
        let Some(civ) = self.civ.as_mut() else { return false };
        if faction < 0 {
            return false;
        }
        let c = |v: i64| v.clamp(0, 255) as u8;
        civ.faction_roster.set_color(faction as usize, Some((c(r), c(g), c(b))))
    }

    /// Clears faction `faction`'s identity colour, returning it to the
    /// palette rule (`faction_rgb_default`). `false` for an unknown faction
    /// or for `0`, exactly as [`Self::civ_set_faction_color`].
    #[func]
    fn civ_clear_faction_color(&mut self, faction: i64) -> bool {
        let Some(civ) = self.civ.as_mut() else { return false };
        if faction < 0 {
            return false;
        }
        civ.faction_roster.set_color(faction as usize, None)
    }

    /// Whether any faction carries a user identity colour — what a
    /// *Reset all* control gates on, without the caller walking
    /// `get_factions`.
    #[func]
    fn civ_has_faction_colors(&self) -> bool {
        self.civ.as_ref().is_some_and(|c| c.faction_roster.any_color_override())
    }

    /// `_civCultureTerrainFit` (`cartalith_civ::civ_culture_terrain_fit`,
    /// reference 23748) for **every** faction in one call, plus the world
    /// terrain means it is judged against — the Faction Roster's "Territory
    /// fit" panel.
    ///
    /// One call for all of them on purpose: the underlying
    /// `civ_faction_aggregates` pass is O(cells), so a per-faction binding
    /// would re-walk the grid once per row of the roster list.
    ///
    /// Each entry: `faction`, `culture`, `mix` (a `Dictionary` of the five
    /// `CIV_TERRAIN_MIX_KEYS` fractions), and — only when that culture is
    /// terrain-themed — `key`, `value`, `world_mean`, `ratio` and `verdict`
    /// (`"match"`/`"typical"`/`"mismatch"`). `common` and `imperial` are
    /// identity-flavoured, not terrain-themed, and get **no verdict at all**
    /// rather than a fabricated one — the reference's own discipline,
    /// preserved: `has_verdict` is `false` for those and the shell must say
    /// "composition only".
    ///
    /// The aggregate runs with `resources`/`density` absent, which that
    /// function explicitly supports: `compute_civilisation` frees the
    /// resource rasters, and the terrain-mix half needs none of them. The
    /// biome raster and the ocean-distance field ARE rebuilt here, on
    /// demand — two full-grid passes, which is why this is a modal-open
    /// call and not a per-frame one.
    #[func]
    fn civ_faction_terrain_fits(&self) -> Array<VarDictionary> {
        let (Some(civ), Some(WorldSource::Generated(ws))) = (self.civ.as_ref(), self.source.as_ref()) else {
            return Array::new();
        };
        let gw = self.gw as usize;
        let gh = self.gh as usize;
        let sea = self.sea_level;
        let biome = cartalith_civ::build_biome_raster(&civ.water_bodies, &ws.temperature, &ws.rainfall);
        let ocean_dist = cartalith_civ::civ_ocean_dist_field(Some(&civ.water_bodies), &ws.field, gw, gh, sea);
        let flow_thresh = cartalith_hydrology::river_flow_thresh(gw, gh, gw, self.map_width_km);
        let has_religion = civ.faction_roster.has_religion_flags();
        let input = cartalith_civ::FactionAggregatesInput {
            faction_count: civ.faction_roster.0.len(),
            gw,
            gh,
            sea,
            map_width_km: self.map_width_km,
            field: &ws.field,
            territory: Some(&civ.territory),
            density: None,
            resources: None,
            biome: Some(&biome),
            flow: Some(&ws.flow_discharge),
            flow_thresh,
            ocean_dist: Some(&ocean_dist),
            faction_has_religion: Some(&has_religion),
        };
        let places: Vec<cartalith_civ::FactionPlace> =
            civ.settlements.iter().map(cartalith_civ::FactionPlace::from_settlement).collect();
        let agg = cartalith_civ::civ_faction_aggregates(&input, &places);

        (1..civ.faction_roster.0.len())
            .map(|f| {
                let culture = civ.faction_roster.0[f].culture.clone();
                let mix_map = &agg.by_faction[f].terrain_mix;
                let mut mix = VarDictionary::new();
                for k in cartalith_civ::CIV_TERRAIN_MIX_KEYS {
                    mix.set(k, *mix_map.get(k).unwrap_or(&0.0));
                }
                let mut out = vdict! {
                    "faction" => f as i64,
                    "culture" => culture.as_str(),
                    "mix" => &mix,
                    "has_verdict" => false,
                };
                let mix_ref: std::collections::HashMap<&str, f64> =
                    mix_map.iter().map(|(&k, &v)| (k, v)).collect();
                let world_ref: std::collections::HashMap<&str, f64> =
                    agg.world_mean_terrain.iter().map(|(&k, &v)| (k, v)).collect();
                if let Some(fit) = cartalith_civ::civ_culture_terrain_fit(&culture, &mix_ref, &world_ref) {
                    out.set("has_verdict", true);
                    out.set("key", fit.key);
                    out.set("value", fit.value);
                    out.set("world_mean", fit.world_mean);
                    out.set("ratio", fit.ratio);
                    out.set("verdict", fit.verdict);
                }
                out
            })
            .collect()
    }

    // ---- place editor (ED-03) ----

    /// Everything `_civPopulatePlaceEditor` (reference 16694) needs that
    /// `get_settlements()` does not already carry: the five side-table
    /// fields (`specialisation`, `traits`, `history`, `age`, `walls`) plus
    /// the settlement's `tid`, so a caller can tell one editing session
    /// from another across a delete.
    ///
    /// `age`/`walls` report `-1` for the reference's own "auto" state
    /// (`umAge == null` / `umWalls == null`), not the inferred value —
    /// which is what an editor field needs, since "auto" and "happens to
    /// currently infer stone" are different states to show. The inferred
    /// answers themselves are real now (`cartalith_civ::military`'s
    /// `um_infer_age`/`um_infer_walls`) and are what
    /// [`Self::civ_military_summary`] reports. Empty `Dictionary` for an
    /// out-of-range index.
    #[func]
    fn civ_settlement_details(&self, index: i64) -> VarDictionary {
        let Some(civ) = self.civ.as_ref() else { return VarDictionary::new() };
        let Some(s) = usize::try_from(index).ok().and_then(|i| civ.settlements.get(i)) else {
            return VarDictionary::new();
        };
        let e = civ.place_extras.get(s.tid);
        let traits: PackedStringArray = e.traits.iter().map(GString::from).collect();
        let spec = if e.specialisation.is_empty() { "none".to_string() } else { e.specialisation };
        dict! {
            "tid" => s.tid as i64,
            "specialisation" => spec,
            "traits" => &traits,
            "history" => e.history,
            "age" => e.age.map_or(-1i64, i64::from),
            "walls" => e.walls.map_or(-1i64, i64::from),
        }
    }

    /// `_civPopulatePlaceEditor`'s field handlers, batched: pass only the
    /// keys that changed. Recognised keys —
    ///
    /// - `name` (String, blank rejected), `kind` (String, one of the six
    ///   `SettlementKind` tiers), `faction` (int, must be a real assignable
    ///   id in the current roster), `population` (int, clamped `>= 0`);
    /// - `specialisation` (String, a `CIV_SPECIALISATIONS` key), `history`
    ///   (String), `age` (int, `< 0` = auto, else clamped `30..=1000`),
    ///   `walls` (int, `< 0` = auto, `0` = off, else on).
    ///
    /// Returns `false` and applies **nothing** if any supplied value is
    /// invalid — all-or-nothing, so a shell cannot half-apply a form.
    /// Traits toggle through `civ_settlement_toggle_trait` instead,
    /// matching the reference's own per-pill click handler.
    ///
    /// `kind` accepts `"metropolis"` here, unlike `civ_drop_settlement`
    /// which rejects it: promoting an existing settlement is exactly what
    /// `_civSelectMetropolises` does, and the editor's own Type dropdown
    /// lists all six classes.
    #[func]
    fn civ_edit_settlement(&mut self, index: i64, fields: VarDictionary) -> bool {
        let Some(civ) = self.civ.as_mut() else { return false };
        let Some(i) = usize::try_from(index).ok().filter(|&i| i < civ.settlements.len()) else {
            return false;
        };

        // -- validate everything first, mutate nothing yet --
        let get_s = |k: &str| fields.get(k).and_then(|v| v.try_to::<GString>().ok()).map(|g| g.to_string());
        let get_i = |k: &str| fields.get(k).and_then(|v| v.try_to::<i64>().ok());

        let name = match get_s("name") {
            Some(n) if n.trim().is_empty() => return false,
            other => other,
        };
        let kind = match get_s("kind") {
            Some(k) => match civ_tools_bridge::kind_from_str(&k) {
                Some(k) => Some(k),
                None => return false,
            },
            None => None,
        };
        let faction = match get_i("faction") {
            Some(f) => {
                let f = f as i32;
                if !civ.faction_roster.is_assignable(f) {
                    return false;
                }
                Some(f)
            }
            None => None,
        };
        let population = get_i("population").map(|p| p.max(0) as u32);
        let specialisation = get_s("specialisation");
        if let Some(sp) = specialisation.as_deref()
            && !cartalith_civ::roster::has_key(&cartalith_civ::roster::CIV_SPECIALISATIONS, sp)
        {
            return false;
        }

        // -- apply --
        let tid = civ.settlements[i].tid;
        let s = &mut civ.settlements[i];
        if let Some(n) = name {
            s.name = n;
        }
        if let Some(k) = kind {
            s.placement.kind = k;
            // `place_settlements`' own invariant, kept: `capital` and
            // `SettlementKind::Capital` are the same fact stored twice, and
            // `civ_drop_place` already sets them together.
            s.placement.capital = k == cartalith_civ::SettlementKind::Capital;
        }
        if let Some(f) = faction {
            s.placement.faction = f;
        }
        if let Some(p) = population {
            s.pop = p;
        }
        if let Some(sp) = specialisation {
            civ.place_extras.set_specialisation(tid, &sp);
        }
        if let Some(h) = get_s("history") {
            civ.place_extras.set_history(tid, &h);
        }
        if let Some(a) = get_i("age") {
            civ.place_extras.set_age(tid, a);
        }
        if let Some(w) = get_i("walls") {
            civ.place_extras.set_walls(tid, w);
        }
        // SG-01: a changed tier, faction or population moves territory,
        // roads and trade balances, none of which are re-derived here.
        self.civ_dirty = true;
        true
    }

    /// One trait pill's click (`_civPopulatePlaceEditor`'s `data-trait`
    /// handler): toggles `key` on or off, preserving insertion order the
    /// way the reference's own `push`/`splice` does. Returns `false` for an
    /// out-of-range index or a key outside `CIV_TRAITS`.
    #[func]
    fn civ_settlement_toggle_trait(&mut self, index: i64, key: GString) -> bool {
        let Some(civ) = self.civ.as_mut() else { return false };
        let Some(tid) = usize::try_from(index).ok().and_then(|i| civ.settlements.get(i)).map(|s| s.tid) else {
            return false;
        };
        civ.place_extras.toggle_trait(tid, &key.to_string())
    }

    /// `_civPeNameRoll` (the editor's dice button): re-rolls this
    /// settlement's name from its **own faction's** naming culture, which
    /// is the reference's v1.07 fix — a rename must match the polity it
    /// belongs to, not the global `common` pool.
    ///
    /// Draws from the same `CivTools::name_rng` stream every manual drop
    /// uses, so re-rolling never rewinds or forks the naming sequence.
    /// Returns the new name, or an empty string on failure.
    #[func]
    fn civ_reroll_settlement_name(&mut self, index: i64) -> GString {
        let (Some(civ), Some(tools)) = (self.civ.as_mut(), self.civ_tools.as_mut()) else {
            return GString::new();
        };
        let Some(s) = usize::try_from(index).ok().and_then(|i| civ.settlements.get_mut(i)) else {
            return GString::new();
        };
        s.name = cartalith_civ::civ_settle_name(&mut tools.name_rng, s.placement.faction);
        GString::from(s.name.as_str())
    }

    /// `_civPopulatePlaceEditor`'s Delete button and `_civCtxShow`'s
    /// "Delete <name>" op (reference 16776 / 25913), minus the `confirm()`
    /// — the shell owns the prompt, matching this file's own
    /// `civ_run_collapse_simulation` split.
    ///
    /// Every index into `get_settlements()` past `index` shifts down by
    /// one, exactly as the reference's `splice` does. Returns `false` for
    /// an out-of-range index or before any `generate()` call.
    ///
    /// Provinces, trade balances, explanations, roads and territory were
    /// all computed against the pre-delete roster and are **not**
    /// recomputed *by this call* — the same staleness `civ_drop_settlement`
    /// already discloses in its own status hint. `explanations` is not
    /// re-indexed either, so `explain_settlement` is stale past the deleted
    /// row until the layer is rebuilt; the shell says so rather than
    /// silently returning a neighbour's causal chain.
    ///
    /// [`Self::recompute_civilisation`] rebuilds all of them, including a
    /// correctly re-indexed `explanations`, without re-generating the world
    /// (`GUI_GAP_REGISTER.md` SG-02/ED-03d). It stays a separate, explicit
    /// call because it costs seconds, not milliseconds — a per-delete
    /// cascade would make the place editor unusable.
    #[func]
    fn civ_delete_settlement(&mut self, index: i64) -> bool {
        let Some(civ) = self.civ.as_mut() else { return false };
        let Some(i) = usize::try_from(index).ok() else { return false };
        match civ_roster_bridge::delete_settlement(&mut civ.settlements, i) {
            Some(tid) => {
                civ.place_extras.forget(tid);
                // SG-01: the deleted place is still a node in the road
                // network and still owns territory until a recompute.
                self.civ_dirty = true;
                true
            }
            None => false,
        }
    }

    // ---- readouts ----

    /// `_civAgrarianRegionalTotal` -> `civPopEstimateOut` (reference 23516,
    /// `PARITY_AUDIT.md` §5 item 7): the "Land sustains ≈ N" readout — the
    /// only world-level population sanity figure the reference shows, and
    /// one this port had no function for at all.
    ///
    /// `{sustains, land_km2, settled}` — the first two are the reference's
    /// own `{total, landKm2}`; `settled` is the summed population of the
    /// settlements that actually exist, so the two can be compared, which
    /// is the whole point of showing the figure. Empty `Dictionary` before
    /// any `generate()` call.
    #[func]
    fn civ_agrarian_regional_total(&self) -> VarDictionary {
        let (Some(civ), Some(WorldSource::Generated(ws))) = (self.civ.as_ref(), self.source.as_ref()) else {
            return VarDictionary::new();
        };
        if self.gw == 0 {
            return VarDictionary::new();
        }
        let cell_km = self.map_width_km / self.gw as f64;
        let out = cartalith_civ::timeline::civ_agrarian_regional_total(
            &civ.dens,
            &ws.field,
            self.sea_level,
            cell_km,
        );
        let settled: u64 = civ.settlements.iter().map(|s| u64::from(s.pop)).sum();
        dict! {
            "sustains" => out.total,
            "land_km2" => out.land_km2,
            "settled" => settled as i64,
        }
    }

    /// `civBiomeKChk` (reference line 1406 / `_biomeK`, line 6441,
    /// `PARITY_AUDIT.md` §5 item 12): the biome carrying-capacity residual
    /// — a disease/climate correction on `build_carrying_capacity`'s K.
    ///
    /// The engine function has always taken the parameter; nothing could
    /// turn it on. Default OFF, matching `_biomeK = 0` and its own comment
    /// ("bit-identical to v0.68"): a zero short-circuits the whole
    /// residual/wetland correction rather than merely zeroing its
    /// contribution.
    ///
    /// Applies on the **next** `generate()`, like every other `CivOptions`
    /// flag — K feeds settlement suitability, so flipping it mid-world
    /// would leave placed settlements sitting on a suitability raster they
    /// were never scored against.
    #[func]
    fn set_biome_k_enabled(&mut self, enabled: bool) {
        self.civ_options.biome_k = enabled;
    }

    /// Whether the biome carrying-capacity residual is on
    /// (`set_biome_k_enabled`).
    #[func]
    fn get_biome_k_enabled(&self) -> bool {
        self.civ_options.biome_k
    }
}

/// `{key, label}` rows from one of `cartalith_civ::roster`'s vocabulary
/// tables — the shape every picker in the Faction Roster and place editor
/// reads.
fn key_label_array(table: &[(&str, &str)]) -> Array<VarDictionary> {
    table
        .iter()
        .map(|&(key, label)| vdict! { "key" => key, "label" => label })
        .collect()
}

/// Global heightmap undo — the reference's `pushUndo`/`undoLast`/
/// `updateUndoUI` (`PARITY_AUDIT.md` §3.1's three missing functions,
/// register `ED-01`/`PR-11`). The mechanism, the bound and every deliberate
/// divergence from the reference live in `undo.rs`'s module doc; this block
/// is only the `Variant` surface over it.
///
/// **Distinct from `sculpt_undo`/`sculpt_redo`**, which pop a *stamp* off an
/// uncommitted Sculpt draft that was never written to the field. These
/// revert a whole committed height field. The reference draws the same line
/// in the same words (its own comment: the stamp history is *"draft-scoped —
/// separate from the field-level undoStack/pushUndo"*), and Ctrl+Z in the
/// reference routes to `sculptUndo` while the Sculpt editor is active and to
/// `undoLast` otherwise.
#[godot_api(secondary)]
impl WorldGen {
    /// Whether `undo_last()` would do anything — the reference's
    /// `undoBtn.disabled = undoStack.length === 0`.
    #[func]
    fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    /// The operation `undo_last()` would revert, for an
    /// "Undo <operation>" menu row. Empty string when the stack is empty.
    #[func]
    fn undo_label(&self) -> GString {
        GString::from(self.undo.next_label().unwrap_or_default())
    }

    /// Pop the newest snapshot back over the live height field — the
    /// reference's `undoLast`.
    ///
    /// Returns the reverted operation's label, or an empty string when there
    /// was nothing to revert (or when the snapshot no longer matches the
    /// live grid's length, which `undo.rs` refuses rather than
    /// half-applies).
    ///
    /// # What it does not re-run
    ///
    /// Flow, river extraction and climate, exactly as
    /// [`Self::sculpt_commit`] and [`Self::carve_fjords`] do not — the
    /// reference's `undoLast` recomputes them inline, this port defers them
    /// (`UNIFIED_TOOL_PLAN.md` milestone A). Undo is therefore as consistent
    /// as the commit it reverses, and no more.
    ///
    /// Nor does it revert the `river_mask`/`river_floor` locks a Sculpt
    /// commit's water hooks wrote. Neither does the reference: it snapshots
    /// `field` and nothing else. See `undo.rs` for the cost of diverging.
    ///
    /// Call `build_color_texture()` again afterwards to see the result —
    /// the same contract [`Self::sculpt_commit`] documents.
    #[func]
    fn undo_last(&mut self) -> GString {
        let Some(WorldSource::Generated(ws)) = self.source.as_mut() else {
            return GString::new();
        };
        match self.undo.restore(&mut ws.field) {
            Some(label) => {
                // ED-02: the ledger's row for that operation goes with it.
                self.ledger.pop_newest_height();
                GString::from(&label)
            }
            None => GString::new(),
        }
    }

    // -- the ledger (`GUI_GAP_REGISTER.md` ED-02) --------------------------

    /// Every committed operation this session, oldest first — the rows
    /// `Edit ▸ Undo history…` draws.
    ///
    /// One row per commit, **not** one row per reversible commit. Each
    /// carries `{seq, subsystem, label, detail, at_ms, kind, reversible,
    /// reason, steps}`:
    ///
    /// - `kind` is `"height"`, `"recorded"` or `"floor"`.
    /// - `reversible` is whether a snapshot is *still held* for it — which is
    ///   a property of the live stack, not of the row, because the stack
    ///   evicts on its own byte budget. A height row that has been evicted
    ///   reports `false` with a reason saying so.
    /// - `reason` is why it cannot be reverted, and is empty when it can.
    ///   Never "not implemented": every string names the specific thing that
    ///   is not retained.
    /// - `steps` is how many `undo_last()` calls reverting to it would take,
    ///   `0` when it is not an offer.
    ///
    /// Empty before anything has been committed. Cheap enough for an
    /// `about_to_popup`: it walks at most `undo::MAX_LEDGER` rows and reads
    /// no field.
    #[func]
    fn undo_ledger(&self) -> Array<VarDictionary> {
        let depth = self.undo.depth();
        self.ledger
            .rows(depth)
            .into_iter()
            .map(|(e, live)| {
                let (kind, reason) = match e.kind {
                    undo::EntryKind::HeightSnapshot if live => ("height", ""),
                    undo::EntryKind::HeightSnapshot => (
                        "height",
                        "its snapshot was dropped to stay inside the undo memory budget",
                    ),
                    undo::EntryKind::Recorded(r) => ("recorded", r),
                    undo::EntryKind::Floor => ("floor", "history starts here"),
                };
                dict! {
                    "seq" => e.seq as i64,
                    "subsystem" => e.subsystem,
                    "label" => e.label.clone(),
                    "detail" => e.detail.clone(),
                    "at_ms" => e.at_ms as i64,
                    "kind" => kind,
                    "reversible" => live,
                    "reason" => reason,
                    "steps" => self.ledger.steps_to_revert_to(e.seq, depth).unwrap_or(0) as i64,
                }
            })
            .collect()
    }

    /// Roll back to the state a ledger row recorded, popping every height
    /// snapshot above it as well — Photoshop's linear history, which
    /// `DCC_SHELL_SPEC.md` §7.1 chose deliberately over the non-linear kind.
    ///
    /// Returns the number of steps actually reverted; `0` when `seq` is
    /// unknown, is not a height row, or no longer has a snapshot. A caller
    /// that gets `0` should re-read [`Self::undo_ledger`] rather than assume
    /// the row is still there.
    ///
    /// Reverting **drops the row and everything after it**, including
    /// recorded-only rows: an operation whose height field has just been
    /// rolled back out from under it is not still in effect, and leaving it
    /// listed would be the worse lie.
    ///
    /// Re-render afterwards, exactly as after [`Self::undo_last`].
    #[func]
    fn undo_revert_to(&mut self, seq: i64) -> i64 {
        if seq <= 0 {
            return 0;
        }
        let seq = seq as u64;
        let Some(steps) = self.ledger.steps_to_revert_to(seq, self.undo.depth()) else {
            return 0;
        };
        let Some(WorldSource::Generated(ws)) = self.source.as_mut() else { return 0 };
        let mut done = 0i64;
        for _ in 0..steps {
            if self.undo.restore(&mut ws.field).is_none() {
                break;
            }
            done += 1;
        }
        if done > 0 {
            self.ledger.truncate_to(seq);
        }
        done
    }

    /// The reference's `#undoMem` readout (`updateUndoUI`), as data rather
    /// than as a formatted sentence: `depth` (int), `max_steps` (int, the
    /// reference's `MAX_UNDO`), `bytes` (int, live cost), `budget_bytes`
    /// (int), `step_bytes` (int, what the *next* push will cost on this
    /// grid) and `label` (String, the next step to revert).
    ///
    /// `step_bytes` is what makes the byte bound legible in a preference
    /// row: at 8192² it is 256 MB, which is why the depth there is 1 and not
    /// the reference's 5.
    #[func]
    fn undo_stats(&self) -> VarDictionary {
        let step_bytes = match self.source.as_ref() {
            Some(WorldSource::Generated(ws)) => ws.field.len() * 4,
            _ => 0,
        };
        dict! {
            "depth" => self.undo.depth() as i64,
            "max_steps" => undo::MAX_STEPS as i64,
            "bytes" => self.undo.bytes() as i64,
            "budget_bytes" => self.undo.budget_bytes() as i64,
            "step_bytes" => step_bytes as i64,
            "label" => self.undo.next_label().unwrap_or_default(),
        }
    }

    /// `Preferences ▸ Memory ▸ Undo history` (register `PR-11`): re-budget
    /// the stack, evicting immediately if the new budget is smaller.
    /// Floored at 4 MiB inside `undo.rs`, so a caller cannot set a budget
    /// that makes undo useless at every resolution.
    #[func]
    fn set_undo_budget_mb(&mut self, mb: i64) {
        self.undo.set_budget_bytes(mb.max(0) as usize * 1024 * 1024);
    }

    /// Drop every step. The reference has no such control — this exists
    /// because `Preferences ▸ Memory` is where a user goes to get memory
    /// back, and the undo buffer is the one line item there they can free on
    /// demand.
    #[func]
    fn clear_undo(&mut self) {
        self.undo.clear();
        // The ledger's rows go with the snapshots. Leaving them would show a
        // history of operations none of which could be reverted, which is
        // worse than an empty panel: the panel would look like it works.
        self.ledger.clear();
    }
}
