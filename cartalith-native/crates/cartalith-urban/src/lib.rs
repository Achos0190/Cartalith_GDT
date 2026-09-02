//! Urban morphology — the reference's script block 4 ("UME", lines 28166-31104
//! of `reference/Cartalith Gen1 v2.10.html`), ported.
//!
//! Block 4 is a self-contained IIFE with **no DOM references at all** (verified
//! by grep, not assumed: `document`/`window`/`canvas`/`ctx.`/`getElementById`/
//! `localStorage`/`requestAnimationFrame` produce zero hits inside its line
//! range) and **no asset-pack references** (confirming, independently, the
//! finding Phase 4's own milestone-1 investigation recorded). It also takes no
//! civ *types* — its whole input surface is scalars plus two plain rasters —
//! so this crate deliberately does **not** depend on `cartalith-civ`. See
//! `URBAN_MORPHOLOGY_SCOPE.md` for the full investigation and the milestone
//! plan this crate is being built along.
//!
//! Milestone 1 is the foundation every later milestone reads: the labelled RNG
//! substreams ([`rng`]) and the vector/polygon geometry kernel ([`geom`]).
//! Milestone 2 adds the [`graph`] the whole engine is built on — a planar
//! street graph with a uniform-grid spatial index, and the planar face
//! extraction that turns it into town blocks. Milestone 3 adds [`astar`], the
//! least-cost search over a site cost raster that milestone 6's primary routes
//! are traced with. Milestone 4 adds [`rules`] — the culture profiles and the
//! generation-rule table every later milestone reads its constants out of.
//! Milestone 5 adds [`site`], the physical setting every later stage queries.
//! Milestone 6 adds [`routes`] — the market anchor and the arterial backbone,
//! the first milestone that produces a real street graph end to end.
//! Milestone 7 adds [`growth`] — the epoch loop that grows the town onto that
//! backbone, and the successive-wall-generation machinery it drives.
//! Milestone 12 adds [`blocks`] — `buildBlocks` and `buildParcels`, the first
//! stage whose output is *building-sized*. It was written **out of order, while
//! 8-11 were unbuilt**: the City Viewer needed discrete, colourable shapes,
//! parcels are the smallest stage that produces them, and every primitive it
//! needs was already built and golden-tested at milestones 1-2. [`blocks`]'s
//! own header records what the missing upstream stages cost it at the time.
//!
//! Those stages have since landed, and this file is the evidence: milestone 8
//! is [`plaza`] and [`radial`] (`buildPlaza` carves the market square out of
//! the principal street on the organic branch as well as the radial one;
//! `buildRadialStreets` and the Venus waterway are [`radial`]'s), milestone 9
//! is [`water`], 10 is [`fortify`], 11 is [`cleanup`], and 13-15 are
//! [`districts`], [`hinterland`] and [`amenities`]. **What is built is
//! `docs/STATUS.md`'s to say, not this file's** — the list above is a map of
//! what each module *is*, and the previous parenthetical "(8-11 are still
//! unbuilt)" is exactly the kind of progress claim that goes stale in a
//! header. It was still there in the file that declares `pub mod radial`.
//!
//! **Wired as of 2026-08-23, and only through one door.**
//! `cartalith_civ::urban_adapter` is this crate's sole consumer: it supplies
//! the real map's water and relief as [`site::WaterCtx`]/[`site::TerrainCtx`],
//! runs the prefix of the reference's `generate()` that milestones 1-7 make
//! possible, and hands the result to `cartalith-godot`'s `urban_bridge` for
//! the map's deep-zoom town layer and the City Viewer
//! (`URBAN_MORPHOLOGY_SCOPE.md` milestone 17a). Nothing changed in this crate
//! to make that work, and nothing here depends on `cartalith-civ` — the
//! dependency runs one way, which is the whole reason the adapter lives
//! outside this crate. `compute_civilisation()` still does not call this
//! subsystem at all: a town is generated on demand, per settlement, never as
//! a generation stage.

pub mod amenities;
pub mod astar;
pub mod blocks;
pub mod cleanup;
pub mod districts;
pub mod fortify;
pub mod generate;
pub mod geom;
pub mod graph;
pub mod growth;
pub mod hinterland;
pub mod plaza;
pub mod radial;
pub mod rng;
pub mod routes;
pub mod rules;
pub mod site;
pub mod water;

// Every module's types and free functions are re-exported here, so a consumer
// spells `cartalith_urban::build_wall` rather than reaching through the module
// path. Six of the seventeen were missing — `amenities`, `cleanup`, `districts`,
// `fortify`, `radial` and `water` — which is why `cartalith-civ` would have had
// to write `cartalith_urban::fortify::build_wall`.
//
// **Constants are deliberately not re-exported**, following what the eleven
// existing lines already did: `hinterland`'s seven `FARM_*`/`BAND_*` and
// `plaza`'s `PROV` are reached through their modules. The provenance strings
// are the bulk of what that leaves out, and they are compared against in tests
// and read by a renderer, both of which name the module anyway.
pub use astar::astar;
pub use generate::{GenOpts, Town, TownGraph, TownParcel, TownSite, generate, hash_model};
pub use blocks::{Block, Parcel, build_blocks, build_parcels};
pub use plaza::{Plaza, build_plaza};
pub use geom::{
    Vec2, js_acos, js_cos, js_exp, js_hypot, js_log, js_log10, js_max, js_min, js_num_cmp, js_or,
    js_round, js_sin, js_truthy_num,
};
pub use amenities::{
    Civic, GamesBuilding, GamesSpec, Market, Markets, build_civic, build_games, build_markets,
    games_shape_at, games_spec, oriented_rect,
};
pub use cleanup::{
    FortZoneSweep, clear_fort_zone, kill_edge, lane_pass, privatize_alleys, prune_largest,
    remove_water_crossings,
};
pub use districts::{
    Building, FaithSite, Lot, Tower, assign_districts, bmap, build_buildings, build_faith_sites,
    peristyle, rect_poly, rect_pts,
};
pub use fortify::{
    Annulus, Bastion, BuiltMass, Curtain, Fort, FortOpts, FortificationBuilder, Spur,
    apply_star_fort, build_wall, built_mass_hull, corner_cut, densify_loop, nearest_idx, town_bank,
};
pub use radial::{RadialStreets, Waterway, build_radial_streets, build_waterway};
pub use water::{
    Bridge, Crossings, Defence, Ford, HarbourOpts, HarbourOutcome, HarbourWorks, Pier,
    add_river_bridges, build_harbour, detect_river_crossings,
};
pub use graph::{Edge, Face, Graph, Node};
pub use hinterland::{
    Decay, Detail, DetailGeom, FarmSpec, Metrics, apply_decay, build_details, build_farmland,
    compute_metrics, crosses_street, farm_spec, ring_fields, strip_fields,
};
pub use growth::{
    Gate, GrowOpts, HarbourFront, Occupancy, RecordingWallBuilder, WallBuilder, WallGeneration,
    WallState, dist_to_line, estimate_carrying_capacity, grow, logistic_ramp, ring_crossings,
    supersede_wall, wall_occupancy,
};
pub use rng::{Substream, fnv1a, stream};
pub use routes::{Anchors, Route, build_primaries, build_primaries_from_paths, place_anchors};
pub use site::{
    Economy, Harbour, Hill, Site, SiteOpts, TerrainCtx, WaterCtx, build_site, shore_from_mask,
    terrain_suitability,
};
pub use rules::{
    CULTURE_PROFILES, CultureProfile, DEFAULT_RULES, MEDIEVAL, MetaRules, ParcelRules, Rules,
    RulesPatch, SettlementRules, StreetRules, VENUS, apply_plot_chaos, apply_wildness, clamp,
    resolve_profile, resolve_rules,
};
