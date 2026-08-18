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

use cartalith_engine::{generate_terrain, WorldParams, WorldStructureParams};
use godot::classes::image::Format;
use godot::classes::{IRefCounted, INode, Image, ImageTexture, Node, RefCounted};
use godot::init::{ExtensionLibrary, gdextension};
use godot::prelude::*;

mod pack;
mod params;
mod render;
mod sculpt_bridge;
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

#[allow(clippy::too_many_arguments)]
fn compute_civilisation(
    ws: &cartalith_engine::WorldState,
    gw: usize,
    gh: usize,
    world: bool,
    map_width_km: f64,
    river_density: f64,
    villages_enabled: bool,
) -> CivData {
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
    let carrying_cap = cartalith_civ::build_carrying_capacity(&soil, &water_access, Some(&biome), &ws.temperature, &ws.field, sea_level, 0.0, None);

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

    let slope_n = cartalith_civ::build_slope_field(&ws.field, gw, gh, world);
    let suit = cartalith_civ::build_settlement_suitability(&soil, &water_access, &carrying_cap, &ws.field, &slope_n, gw, gh, sea_level, Some(&ctx));
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

    let placements = cartalith_civ::place_settlements(&seeds, &suit, &ws.field, &wb.classification, &wb.fill_level, gw, gh, sea_level, world, CIV_FACTION_COUNT);

    // Real auto-populate road network, not `build_road_network` (that's
    // `buildRoadNetwork`, the reference's *manual*-placement-tool
    // algorithm -- an earlier pass here used it as a stand-in for the
    // auto-populate system before that system was ported at all). Now
    // that both exist: `civ_hierarchical_network_topology` (Phase 2
    // milestone 12, the real `_civHierarchicalNetwork` raw topology) then
    // `civ_consolidate_and_smooth_ways` (milestone 14) for the
    // Catmull-Rom-smoothed, classified, named polylines this map actually
    // draws.
    let topology = cartalith_civ::civ_hierarchical_network_topology(
        &placements, gw, gh, sea_level, &ws.field, &ws.flow_discharge, &river_order, &biome, &wb.classification, world, map_width_km,
    );
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
    let mut settlements;
    if villages_enabled {
        let mut rng = cartalith_civ::civ_name_rng();
        settlements = cartalith_civ::name_and_populate_settlements_with_rng(&placements, &mut rng);
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
    } else {
        settlements = cartalith_civ::name_and_populate_settlements(&placements);
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
                    &slope_n,
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

    // Milestone 14 consolidation/smoothing needs NAMED settlements
    // (`pa.name`/`pb.name`) -- must run after the naming/village block
    // above, not alongside `topology`. `topology.edges`' `a`/`b` indices
    // are into the pre-village `placements` order, which `settlements`
    // preserves as its own prefix (villages are appended after, never
    // interleaved), so indexing stays valid whether or not villages ran.
    let ways = cartalith_civ::civ_consolidate_and_smooth_ways(&topology, &settlements, &ws.field, &wb.classification, gw, gh, map_width_km);

    // Sea routes (milestone 13): reference calls `_civMstRoutes(ports,true)`
    // unconditionally whenever >=2 port-tagged settlements exist, over the
    // SAME `settlements` list `ways` was just built from (villages included,
    // if enabled -- `civ_sea_routes` itself gates on `.coastal`, and a
    // village's own `coastal: false` above means villages never qualify as
    // ports here, matching the reference's own hamlet-tier village shape).
    let ports: Vec<cartalith_civ::NamedSettlement> =
        settlements.iter().filter(|s| s.placement.coastal).cloned().collect();
    let sea_routes = cartalith_civ::civ_sea_routes(&ports, &ws.field, &wb.classification, gw, gh, world, map_width_km);

    CivData { settlements, ways, sea_routes, territory, provinces, province_list, trade_balances, explanations }
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
    /// Set via `set_villages_enabled`. Defaults `false`, matching the
    /// reference's own real `_civVillages` default (Phase 2 milestone 15's
    /// own "flagged, not resolved" note on this exact toggle) -- disabled
    /// generation stays bit-identical to every existing golden fixture.
    villages: bool,
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
            villages: false,
            civ: None,
            seed: 0,
            asset_pack: None,
            quality: QualityTier::Quality,
            sculpt: None,
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
        self.civ = Some(compute_civilisation(&ws, p.gw, p.gh, p.world, p.map_width_km, p.river_density, self.villages));
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
    #[func]
    fn set_params(&mut self, values: VarDictionary) -> VarDictionary {
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
        }
        dict! { "rejected" => &rejected, "clamped" => &clamped }
    }

    /// Restores every generation parameter to its
    /// `cartalith_engine::WorldParams::defaults` value — the real
    /// "reset to defaults" action, not a GDScript re-send of remembered
    /// numbers. Does not touch `set_villages_enabled` (civ-layer, not a
    /// `WorldParams` field) or anything about an already-generated world.
    #[func]
    fn reset_params(&mut self) {
        self.params = params::defaults();
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
        self.villages
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
        self.villages = enabled;
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
    #[func]
    fn generate_sized(&mut self, seed: i32, width_km: f64, grid_w: i32, grid_h: i32) {
        let (gw, gh) = (grid_w.max(4) as usize, grid_h.max(4) as usize);
        let p = self.call_params(seed, width_km, gw, gh);
        let ws = generate_terrain(&p);
        self.absorb(ws, &p, seed);
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
        self.source = Some(WorldSource::Loaded(Box::new(save)));
        true
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
        render::border_width_cells(&TerrainAppearance::for_tier(self.quality), gw, gh) / gw as f64
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
        let appearance = TerrainAppearance::for_tier(self.quality);
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
        const FACTION_RGB: [(u8, u8, u8); 6] =
            [(230, 159, 0), (86, 180, 233), (0, 158, 115), (240, 228, 66), (0, 114, 178), (213, 94, 0)];
        const ALPHA: u8 = 82; // ~0.32, low enough for terrain/biome colour to read through

        // Same plate-frame rule the river tint follows: this wash is drawn
        // over the finished raster, and a faction whose territory reaches
        // the sheet edge would otherwise colour the bare-paper margin.
        // `border_cover` is `0.0` across the whole interior (and everywhere
        // when there is no frame), so `alpha` is untouched there.
        let appearance = TerrainAppearance::for_tier(self.quality);
        let mut bytes = Vec::with_capacity(gw * gh * 4);
        for (i, &f) in civ.territory.iter().enumerate() {
            let cover = render::border_cover(&appearance, i % gw, i / gw, gw, gh);
            if f > 0 && cover < 1.0 {
                let (r, g, b) = FACTION_RGB[((f - 1) as usize) % FACTION_RGB.len()];
                bytes.extend_from_slice(&[r, g, b, (ALPHA as f64 * (1.0 - cover)) as u8]);
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
    /// `population` (int), `kind` (String: "capital"/"city"/"town"/
    /// "village"/"hamlet"), `faction` (int, `1..=6`, matching
    /// `CIV_FACTION_COUNT`), `capital` (bool), `coastal` (bool). Empty
    /// before any `generate()` call, after `load_save()` (no civ data for
    /// a loaded save, see `load_save`'s own doc comment), or if generation
    /// produced zero settlement candidates.
    #[func]
    fn get_settlements(&self) -> Array<VarDictionary> {
        let Some(civ) = self.civ.as_ref() else { return Array::new() };
        civ.settlements
            .iter()
            .map(|s| {
                let kind_str = match s.placement.kind {
                    cartalith_civ::SettlementKind::Capital => "capital",
                    cartalith_civ::SettlementKind::City => "city",
                    cartalith_civ::SettlementKind::Town => "town",
                    cartalith_civ::SettlementKind::Village => "village",
                    cartalith_civ::SettlementKind::Hamlet => "hamlet",
                };
                vdict! {
                    "x" => s.placement.x as i32,
                    "y" => s.placement.y as i32,
                    "name" => s.name.as_str(),
                    "population" => s.pop as i32,
                    "kind" => kind_str,
                    "faction" => s.placement.faction,
                    "capital" => s.placement.capital,
                    "coastal" => s.placement.coastal,
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
    #[func]
    fn get_roads(&self) -> Array<VarDictionary> {
        let Some(civ) = self.civ.as_ref() else { return Array::new() };
        civ.ways
            .iter()
            .filter(|w| !w.hidden)
            .map(|w| {
                let points: PackedVector2Array =
                    w.pts.iter().map(|&(x, y)| Vector2::new(x as f32, y as f32)).collect();
                let way_type = match w.way_type {
                    cartalith_civ::WayType::Highway => "highway",
                    cartalith_civ::WayType::Regional => "regional",
                    cartalith_civ::WayType::Road => "road",
                    cartalith_civ::WayType::Track => "track",
                };
                // `brks`: indices into `points` where this way's own path
                // has a real gap (two disjoint consolidated runs sharing
                // one `Way`, not a continuous curve) -- drawing straight
                // through these would render a phantom line across the
                // gap, so the renderer must split into separate strokes
                // there instead of treating `points` as one polyline.
                let brks: PackedInt32Array = w.brks.iter().map(|&b| b as i32).collect();
                dict! { "points" => &points, "brks" => &brks, "way_type" => way_type, "name" => w.name.as_str() }
            })
            .collect()
    }

    /// Sea-lane routes (`cartalith_civ::civ_sea_routes`, Phase 2 milestone
    /// 13) -- same `{points, brks, name}` shape as `get_roads()`, minus
    /// `way_type` (sea routes have no highway/regional/road/track tier,
    /// `SeaRoute` doesn't carry one). Draw distinctly from land roads --
    /// the reference's own convention (line ~15511) is a dark navy
    /// underlayer plus a lighter dashed overlay, not a road colour/width.
    /// Empty under the same conditions as `get_roads()`.
    #[func]
    fn get_sea_routes(&self) -> Array<VarDictionary> {
        let Some(civ) = self.civ.as_ref() else { return Array::new() };
        civ.sea_routes
            .iter()
            .map(|r| {
                let points: PackedVector2Array =
                    r.pts.iter().map(|&(x, y)| Vector2::new(x as f32, y as f32)).collect();
                let brks: PackedInt32Array = r.brks.iter().map(|&b| b as i32).collect();
                dict! { "points" => &points, "brks" => &brks, "name" => r.name.as_str() }
            })
            .collect()
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
        let appearance = TerrainAppearance::for_tier(self.quality);
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

        let appearance = TerrainAppearance::for_tier(self.quality);
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
    /// **Deliberately does not re-run erosion, hydrology or climate.**
    /// `DCC_SHELL_SPEC.md` §5.2's prose still says Commit "re-runs erosion,
    /// hydrology and climate once" -- that line is stale
    /// (`SCULPT_FUNCTION_CHART.md` §7 flags it at the design end); the
    /// eager form was measured at ~7s/stroke at 2048² and rejected in
    /// `UNIFIED_TOOL_PLAN.md` milestone C. Instead every tile the commit
    /// touched is marked dirty (`tiles_marked` below) for a caller to
    /// re-run those stages on its own schedule -- this binding does not
    /// currently expose an entry point that consumes that dirty set (no
    /// `#[func]` here re-runs flow/climate/hydrology for a subset of
    /// tiles); see this crate's own report for that gap.
    ///
    /// `reason` is a short caller-chosen string for the dirty-tile record
    /// (`"sculpt"` is fine). Returns a summary `Dictionary`:
    /// `stamps_applied`/`stamps_skipped` (int), `tiles_marked`
    /// (`PackedInt32Array`), `rivers_carved`/`cells_locked` (int, the
    /// River hook), `lakes_deposited`/`lake_cells` (int, the Lake hook) --
    /// empty `Dictionary` before any `generate()` call.
    ///
    /// Call `build_color_texture()` again afterward to see the result --
    /// the same "no regeneration needed" contract `set_quality_tier`'s own
    /// doc comment establishes for a purely presentational change; this one
    /// really did change the height field, but the render path reads it
    /// fresh on every call regardless.
    #[func]
    fn sculpt_commit(&mut self, reason: GString) -> VarDictionary {
        let sea_level = self.sea_level;
        let reason = reason.to_string();
        let (Some(sculpt), Some(WorldSource::Generated(ws))) = (self.sculpt.as_mut(), self.source.as_mut()) else {
            return VarDictionary::new();
        };
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
        dict! {
            "stamps_applied" => summary.pass.stamps_applied as i64,
            "stamps_skipped" => summary.pass.stamps_skipped as i64,
            "tiles_marked" => &tiles_marked,
            "rivers_carved" => summary.rivers_carved as i64,
            "cells_locked" => summary.cells_locked as i64,
            "lakes_deposited" => summary.lakes_deposited as i64,
            "lake_cells" => summary.lake_cells as i64,
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
