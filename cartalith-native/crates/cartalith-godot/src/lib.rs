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

mod render;
use render::RenderCtx;

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
    // (`SUIT_RESOURCE_KEYS`) are ever read below, via `ctx.resources` ->
    // `build_settlement_suitability`'s mineral term (confirmed: no other
    // reader anywhere in this production call chain -- grepped). `ctx`
    // keeps `resources` alive until `suit` is computed ~40 lines down, so
    // the other 6 (clay/buildstone/flint/obsidian/sulfur/alum, ~40% of this
    // struct's own ~240 MB at 2048x2048) would otherwise sit unused for
    // that whole span -- the single largest confirmed contributor to this
    // function's measured peak (real before/after numbers in
    // `MEMORY_OPTIMIZATION_SCOPE.md`/`CHANGELOG.md`). Freed immediately
    // instead of held: replacing with an empty `Vec` drops the old heap
    // allocation right here, and `resource_field()` never indexes these 6
    // keys (only `SUIT_RESOURCE_KEYS` ever reaches it), so nothing later
    // reads the now-empty buffers.
    resources.clay = Vec::new();
    resources.buildstone = Vec::new();
    resources.flint = Vec::new();
    resources.obsidian = Vec::new();
    resources.sulfur = Vec::new();
    resources.alum = Vec::new();

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
            &roads,
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

    CivData { settlements, ways, sea_routes, territory, provinces, province_list }
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
    /// Set via `set_experimental_flags`, applied by both `generate()` and
    /// `generate_world_structure()`. All four are now golden-verified
    /// (see each field's own doc comment in `cartalith-engine`/
    /// `cartalith-climate` -- `cartalith-native/docs/CHANGELOG.md` has the
    /// full extraction history). `dynamic_lithology` defaults `false`
    /// because that's JS's own real default; `volc_provinces`/
    /// `terrain_wind_deflection`/`ocean_currents` default `true` because
    /// JS's real defaults are `true` (unconditional, in wind deflection's
    /// case) -- this `WorldGen` wrapper's own defaults can match JS
    /// exactly regardless of what `cartalith_engine::WorldParams::defaults`
    /// itself defaults to, since every call site here overrides all four
    /// explicitly. Still exposed as toggles, not hardcoded: useful for
    /// comparing against the real HTML app with one turned off at a time.
    dynamic_lithology: bool,
    volc_provinces: bool,
    terrain_wind_deflection: bool,
    ocean_currents: bool,
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
            dynamic_lithology: false,
            volc_provinces: true,
            terrain_wind_deflection: true,
            ocean_currents: true,
            world: false,
            lat_n: 55.0,
            lat_s: 5.0,
            villages: false,
            civ: None,
        }
    }
}

#[godot_api]
impl WorldGen {
    /// Sets the four golden-verified subsystem flags this instance's
    /// `generate()`/`generate_world_structure()` calls apply from then on
    /// — see the `WorldGen` struct's own doc comment on the fields this
    /// writes.
    #[func]
    fn set_experimental_flags(
        &mut self,
        dynamic_lithology: bool,
        volc_provinces: bool,
        terrain_wind_deflection: bool,
        ocean_currents: bool,
    ) {
        self.dynamic_lithology = dynamic_lithology;
        self.volc_provinces = volc_provinces;
        self.terrain_wind_deflection = terrain_wind_deflection;
        self.ocean_currents = ocean_currents;
    }

    /// Reference `_civVillages` (default OFF) -- the additive village-
    /// seeding pass (Phase 2 milestone 15), gated separately from the
    /// four flags above since it's civ-layer, not terrain-substrate.
    #[func]
    fn set_villages_enabled(&mut self, enabled: bool) {
        self.villages = enabled;
    }

    /// Runs the full ported pipeline (`cartalith_engine::generate_terrain`)
    /// at the given seed/real-km map width/grid resolution. `resolution`
    /// is clamped to a sane minimum (4) — a 0 or negative value from an
    /// unset GDScript `SpinBox` should not panic the extension.
    #[func]
    fn generate(&mut self, seed: i32, width_km: f64, resolution: i32) {
        let gw = resolution.max(4) as usize;
        let gh = gw;
        let mut p = WorldParams::defaults(gw, gh, seed);
        p.map_width_km = if width_km > 0.0 { width_km } else { 800.0 };
        p.tect.dynamic_lithology = self.dynamic_lithology;
        p.volc.provinces = self.volc_provinces;
        p.climate.terrain_wind_deflection = self.terrain_wind_deflection;
        p.climate.currents = self.ocean_currents;
        let ws = generate_terrain(&p);
        // Not p.sea_level -- World-Structure archetypes re-anchor it;
        // WorldState carries the value actually used.
        self.sea_level = ws.sea_level;
        self.gw = gw as i32;
        self.gh = gh as i32;
        self.world = p.world;
        self.lat_n = p.climate.lat_n;
        self.lat_s = p.climate.lat_s;
        self.civ = Some(compute_civilisation(&ws, gw, gh, p.world, p.map_width_km, p.river_density, self.villages));
        self.source = Some(WorldSource::Generated(Box::new(ws)));
    }

    /// Named World-Structure archetype presets (reference HTML
    /// `ARCHETYPES`, lines 2521-2526) as
    /// `(continentality, fragmentation, tectonic_energy, ocean_depth,
    /// hotspot_density)`. `cartalith_engine::WorldParams::world_structure`
    /// itself takes raw knobs only, not named presets (its own doc
    /// comment: "a caller wanting 'Archipelago' passes that preset's own
    /// numbers") -- so the name -> knobs lookup lives here, in the
    /// boundary layer, rather than in GDScript
    /// (`ARCHITECTURE.md`: "Godot computes nothing beyond layout").
    #[func]
    fn generate_world_structure(&mut self, seed: i32, width_km: f64, resolution: i32, archetype: GString) -> bool {
        let (continentality, fragmentation, tectonic_energy, ocean_depth, hotspot_density) =
            match archetype.to_string().to_lowercase().as_str() {
                "earth" => (0.30, 0.50, 0.60, 0.60, 0.20),
                "supercontinent" => (0.60, 0.10, 0.50, 0.70, 0.10),
                "archipelago" => (0.15, 0.90, 0.80, 0.30, 0.50),
                "volcanic" => (0.05, 1.00, 0.90, 0.80, 1.00),
                "rift" => (0.40, 0.35, 0.75, 0.55, 0.30),
                other => {
                    godot_print!("cartalith-godot: unknown World-Structure archetype '{other}'");
                    return false;
                }
            };

        let gw = resolution.max(4) as usize;
        let gh = gw;
        let mut p = WorldParams::defaults(gw, gh, seed);
        p.map_width_km = if width_km > 0.0 { width_km } else { 800.0 };
        p.world_structure =
            WorldStructureParams { enabled: true, continentality, fragmentation, tectonic_energy, ocean_depth, hotspot_density };
        p.tect.dynamic_lithology = self.dynamic_lithology;
        p.volc.provinces = self.volc_provinces;
        p.climate.terrain_wind_deflection = self.terrain_wind_deflection;
        p.climate.currents = self.ocean_currents;

        let ws = generate_terrain(&p);
        self.sea_level = ws.sea_level;
        self.gw = gw as i32;
        self.gh = gh as i32;
        self.world = p.world;
        self.lat_n = p.climate.lat_n;
        self.lat_s = p.climate.lat_s;
        self.civ = Some(compute_civilisation(&ws, gw, gh, p.world, p.map_width_km, p.river_density, self.villages));
        self.source = Some(WorldSource::Generated(Box::new(ws)));
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
        self.source = Some(WorldSource::Loaded(Box::new(save)));
        true
    }

    #[func]
    fn get_width(&self) -> i32 {
        self.gw
    }

    #[func]
    fn get_height(&self) -> i32 {
        self.gh
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
        let ctx = RenderCtx::new(field, temperature, rainfall, flow, gw, gh, self.sea_level, self.world, self.lat_n, self.lat_s);

        let mut bytes = Vec::with_capacity(gw * gh * 3);
        for y in 0..gh {
            for x in 0..gw {
                let i = y * gw + x;
                let (mut r, mut g, mut b) = render::cell_color(&ctx, x, y);

                if let Some(mask) = chan_mask
                    && mask[i] != 0
                {
                    r *= 0.5;
                    g = (g * 0.5 + 0.3).min(1.0);
                    b = (b * 0.5 + 0.45).min(1.0);
                }

                bytes.push((r.clamp(0.0, 1.0) * 255.0) as u8);
                bytes.push((g.clamp(0.0, 1.0) * 255.0) as u8);
                bytes.push((b.clamp(0.0, 1.0) * 255.0) as u8);
            }
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

        let mut bytes = Vec::with_capacity(gw * gh * 4);
        for &f in &civ.territory {
            if f > 0 {
                let (r, g, b) = FACTION_RGB[((f - 1) as usize) % FACTION_RGB.len()];
                bytes.extend_from_slice(&[r, g, b, ALPHA]);
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

    /// Province *boundaries* as a semi-transparent RGBA overlay -- a thin
    /// line wherever two orthogonally-adjacent cells belong to different
    /// (nonzero) provinces, transparent everywhere else. Deliberately not a
    /// per-province fill colour the way `build_territory_texture` fills by
    /// faction: a province count isn't bounded the way `CIV_FACTION_COUNT`
    /// is, so there's no fixed small palette to draw from without a real
    /// UI/UX design pass picking one -- boundary lines need no palette at
    /// all and read clearly layered on top of `build_territory_texture`'s
    /// own per-faction fill. Not yet wired into `main.gd`/`map_overlay.gd`
    /// (`CHANGELOG.md`) -- this method exists so that wiring is a small,
    /// later addition, not a green-field task. `None` under the same
    /// conditions as `build_territory_texture`.
    #[func]
    fn build_province_boundary_texture(&self) -> Option<Gd<ImageTexture>> {
        let civ = self.civ.as_ref()?;
        let gw = self.gw as usize;
        let gh = self.gh as usize;
        const LINE_RGBA: [u8; 4] = [43, 30, 10, 200]; // matches map_overlay.gd's MARKER_OUTLINE ink tone

        let mut bytes = vec![0u8; gw * gh * 4];
        for y in 0..gh {
            for x in 0..gw {
                let i = y * gw + x;
                let here = civ.provinces[i];
                if here == 0 {
                    continue;
                }
                let differs_from_neighbor = (x + 1 < gw && civ.provinces[i + 1] != here)
                    || (y + 1 < gh && civ.provinces[i + gw] != here);
                if differs_from_neighbor {
                    bytes[i * 4..i * 4 + 4].copy_from_slice(&LINE_RGBA);
                }
            }
        }
        let packed = PackedByteArray::from(bytes);
        let image = Image::create_from_data(gw as i32, gh as i32, false, Format::RGBA8, &packed)?;
        ImageTexture::create_from_image(&image)
    }
}
