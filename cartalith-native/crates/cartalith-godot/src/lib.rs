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
use render::{RenderCtx, SplatTextures, TerrainAppearance};

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

/// `MVP_SCOPE.md` points 10-11: basic 2D rendering + minimal UI. Owns the
/// last `generate_terrain()` result (or loaded save); GDScript drives it via
/// `generate()`/`load_save()` then `build_color_texture()`. Square grid
/// (`gw == gh`) for MVP **generation** — a loaded save keeps whatever
/// `GW`/`GH` it was exported at, which need not be square (the reference
/// HTML's own `resW`/aspect-from-image handling is UI-layer scope this port
/// hasn't built yet, but a save's own stored resolution isn't that).
#[derive(GodotClass)]
#[class(base=RefCounted)]
struct WorldGen {
    base: Base<RefCounted>,
    source: Option<WorldSource>,
    gw: i32,
    gh: i32,
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
}

#[godot_api]
impl IRefCounted for WorldGen {
    fn init(base: Base<RefCounted>) -> Self {
        Self {
            base,
            source: None,
            gw: 0,
            gh: 0,
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
        }
    }
}

/// Plain (non-`#[func]`) helpers shared by `generate()` and
/// `generate_world_structure()` — kept out of the `#[godot_api]` block since
/// they are Rust-internal, not part of the GDScript surface.
impl WorldGen {
    /// This instance's persistent parameters with the three call-argument
    /// fields filled in. The single place `gw`/`gh`/`seed`/`map_width_km`
    /// enter a `WorldParams` — everything else comes from `self.params`
    /// unchanged, which is why an untouched `WorldGen` generates exactly what
    /// it did before the parameter API existed.
    fn call_params(&self, seed: i32, width_km: f64, gw: usize) -> WorldParams {
        let mut p = self.params.clone();
        p.gw = gw;
        p.gh = gw;
        p.tect.seed = seed;
        p.map_width_km = if width_km > 0.0 { width_km } else { 800.0 };
        p
    }

    /// Stores a finished generation: the effective sea level, the render
    /// inputs `render.rs` needs, the civ layer, and which stages actually ran
    /// on GPU.
    fn absorb(&mut self, ws: cartalith_engine::WorldState, p: &WorldParams, gw: usize, seed: i32) {
        // Not `p.sea_level` -- World-Structure archetypes re-anchor it;
        // `WorldState` carries the value actually used.
        self.sea_level = ws.sea_level;
        self.gw = gw as i32;
        self.gh = gw as i32;
        self.world = p.world;
        self.lat_n = p.climate.lat_n;
        self.lat_s = p.climate.lat_s;
        self.gpu_stages_used = ws.gpu_stages_used.clone();
        self.civ = Some(compute_civilisation(&ws, gw, gw, p.world, p.map_width_km, p.river_density, self.villages));
        self.source = Some(WorldSource::Generated(Box::new(ws)));
        self.seed = seed;
    }
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
    #[func]
    fn generate(&mut self, seed: i32, width_km: f64, resolution: i32) {
        let gw = resolution.max(4) as usize;
        let p = self.call_params(seed, width_km, gw);
        let ws = generate_terrain(&p);
        self.absorb(ws, &p, gw, seed);
    }

    /// Generates with a named World-Structure archetype (`ARCHETYPES`, the
    /// reference's own five presets) applied for **this call only** — the
    /// stored parameters (`get_params()`) are left exactly as they were, so
    /// alternating between this and `generate()` per menu selection behaves
    /// as it always has. Use `apply_archetype()` instead to make an
    /// archetype stick and become editable as raw sliders.
    ///
    /// Returns `false` (generating nothing) for an unknown archetype name.
    #[func]
    fn generate_world_structure(&mut self, seed: i32, width_km: f64, resolution: i32, archetype: GString) -> bool {
        let Some(world_structure) = archetype_knobs(&archetype) else {
            godot_print!("cartalith-godot: unknown World-Structure archetype '{archetype}'");
            return false;
        };

        let gw = resolution.max(4) as usize;
        let mut p = self.call_params(seed, width_km, gw);
        // `p.sea_level` carries the stored input for consistency with
        // `generate()`, but `apply_world_structure_sea_level` always
        // re-anchors it below since `enabled` is unconditionally true on this
        // path -- see the `params` field's own doc comment.
        p.world_structure = world_structure;

        let ws = generate_terrain(&p);
        self.absorb(ws, &p, gw, seed);
        true
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
        self.gw = save.params.gw as i32;
        self.gh = save.params.gh as i32;
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
    #[func]
    fn get_border_inset_frac(&self) -> f64 {
        let gw = self.gw as usize;
        if gw == 0 {
            return 0.0;
        }
        render::border_width_cells(&TerrainAppearance::default(), gw) / gw as f64
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
        let appearance = TerrainAppearance::default();
        let mut ctx = RenderCtx::with_appearance(
            field, temperature, rainfall, flow, gw, gh, self.sea_level, self.world, self.lat_n, self.lat_s, appearance.clone(),
        );
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

        let mut bytes = Vec::with_capacity(gw * gh * 3);
        for y in 0..gh {
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

                bytes.push((r.clamp(0.0, 1.0) * 255.0) as u8);
                bytes.push((g.clamp(0.0, 1.0) * 255.0) as u8);
                bytes.push((b.clamp(0.0, 1.0) * 255.0) as u8);
            }
        }

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
        let appearance = TerrainAppearance::default();
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
        let appearance = TerrainAppearance::default();
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
}
