//! The project archive's Godot surface — `SAVEFILE_COMPAT.md`, owner
//! decision 2026-08-25 (`DECISIONS.md` §7h).
//!
//! `cartalith-io` owns the container, the slot registry and the raster
//! encoding, and deliberately owns **no schema** for what goes in the
//! slots. This file owns the schemas: it is the one place that knows both
//! what a settlement *is* in this port and what a settlement *looks like*
//! in the format.
//!
//! ## Why the archive's shapes are not the Rust types
//!
//! Every document below goes through a DTO rather than a `#[derive(
//! Serialize)]` on the live type, and that is the point rather than
//! boilerplate. `SAVEFILE_COMPAT.md` is a specification a second
//! implementation will be written from in JavaScript; its member names are
//! chosen for that reader (`points`, `length_km`, `class`, `wrap_x`), not
//! for this one (`pts`, `km`, `way_type`, `world`). Deriving on the live
//! types would publish this port's private vocabulary as the format and
//! make every future rename a format break. It would also mean adding
//! `serde` to `cartalith-civ`, a golden-tested crate that has no other
//! reason to carry it.
//!
//! The DTOs additionally *flatten* — a settlement's `placement` is an
//! artefact of how this port computes one, and the archive has no reason to
//! carry the nesting.
//!
//! ## What reaches the archive, and by which route
//!
//! | Payload | Route |
//! |---|---|
//! | terrain rasters, grid, parameters | `cartalith-io` directly, from `WorldGen`'s own state |
//! | settlements, factions, ways, provinces, continents, timeline, civ rasters | the DTOs below, from `CivData` |
//! | labels, icons, region | the DTOs below, from the tool bridges |
//! | landmark settings **and the last run's placements** | `LandmarksDoc` below, from `WorldGen::landmark_store` |
//! | journeys (`STORY_PLANNING_SCOPE.md` SP-1) | the DTOs below, from `InfraTools::journeys` -- see "The Journey entity" section further down |
//! | vault links | `vault_state_json()`/`vault_restore_state()`, the pair `vault_bridge.rs` already publishes |
//! | anything GDScript owns | [`WorldGen::project_save_with_documents`]'s dictionary |
//!
//! That last row is the channel `SAVEFILE_COMPAT.md` §6.5 calls for and the
//! reason there is no `WorldGen` field holding shell state: a payload the
//! shell owns travels *through* a save call rather than being mirrored into
//! the engine first, so adding one needs no engine change at all.
//!
//! ## The four documents nothing could produce
//!
//! `cartalith-io` has registered `drafts/paint.json`, `drafts/sculpt.json`,
//! `library/assets.json` and `library/travel.json` since the format was
//! written, and until 2026-08-31 **nothing anywhere could build one** (the
//! *reading* half took until 2026-09-03; see below). The
//! channel was never the missing piece: the shell may write all four, and
//! could always have done so. What was missing is that the payloads are
//! *engine* state -- a sparse paint layer, a `PassBuffer` of sculpt recipes,
//! an `AssetDB`, a `TravelLibrary` -- and GDScript has no view of any of
//! them to serialize. Painted biome/terrain/splat layers and the whole
//! Sculpt draft stack were therefore lost on every save.
//!
//! The four `*_document_json()` functions below close that: each builds its
//! slot's JSON text out of live engine state, through the codec the
//! subsystem already owns (`PaintLayer::encode_sparse`,
//! `sculpt_bridge`'s own control-pair pair, `AssetDB::to_library_json`,
//! `travel_bridge`'s `*_to_pairs`). The shell hands the text straight to
//! `project_save_with_documents` and gets it back byte for byte from
//! `project_open`'s `documents`.
//!
//! **They stay caller-owned rather than moving to
//! [`ENGINE_OWNED_SLOTS`]**, and that is the deliberate half. An engine-owned
//! slot is one the engine writes on *every* save whether the caller wants it
//! or not; a draft and a library are documents a caller decides about (Save a
//! copy without my drafts; import a library from another project). The
//! partition assertion in this file's own tests names all five callers, and
//! this pass did not change it.
//!
//! **The return leg was missing for both drafts until 2026-09-03, and this
//! paragraph used to explain why that was structural.** It was not. The two
//! libraries always restored -- `travel_library`/`asset_library` are plain
//! non-`Option` fields on `WorldGen` that survive `absorb()`, so there was
//! always something to restore into -- while `drafts/paint.json` and
//! `drafts/sculpt.json` were written on every save and applied to nothing,
//! because `load_save` drops `WorldGen::paint` and `WorldGen::sculpt`. A
//! painted world reopened unpainted and a stamp stack reopened empty, with
//! the documents sitting in the archive the whole time.
//!
//! Both now restore, in [`WorldGen::project_open`], which builds the editor
//! that holds them:
//!
//! * The Paint editor's one missing input was the land-only gate's
//!   classification, and the archive carries what computes it --
//!   `build_water_bodies` over `rasters/heightmap.f32` and
//!   `rasters/rainfall.f32`, both of which §5.1 requires. Nothing is
//!   invented; see `loaded_water_mask`.
//! * The Sculpt editor's were `river_mask`/`river_floor`, and `None`/`None`
//!   is the *legal* empty lock state (`SculptEditor::new` falls back to
//!   `WaterState::new`, exactly what a world generated with `carve_rivers`
//!   off gets) rather than a stand-in for a missing value. A project saved
//!   with its substrate (`SAVEFILE_COMPAT.md` §8.3) seeds them from the
//!   world's own saved carve lock instead, exactly as `absorb` does, and can
//!   **commit** the draft; one saved without it still cannot, since
//!   `sculpt_commit` needs `WorldSource::Generated`.
//!
//! [`WorldGen::paint_restore_document`] and
//! [`WorldGen::sculpt_restore_document`] stay on the surface as the *import*
//! route -- a draft read out of a different file with
//! `project_read_document` and applied to the world already on screen.
//!
//! It has two return legs, and they answer different questions.
//! [`WorldGen::project_open`]'s `documents` hands back every caller-owned
//! document the archive held, which is what a caller loading the project
//! wants. [`WorldGen::project_read_document`] answers the same question
//! about a file the caller does **not** want to open — reloading saved
//! journeys should not replace the world on screen. Both return JSON
//! **text**, byte for byte as it was written, for the reason
//! `project_save_with_documents`' own doc comment gives: a `Dictionary`
//! would go through Godot's JSON, which floats every integer it touches.
//! Text in, text out; the engine never types a number it does not model.
//!
//! ## What a restored project is, and is not
//!
//! [`WorldGen::project_open`] rebuilds `CivData` from the archive, so
//! `get_settlements()`, `get_ways()`, the faction roster, territory and the
//! timeline are all real for a loaded project. That is the unblock
//! `MARKDOWN_VAULT_SCOPE.md` milestone 3 and `STORY_PLANNING_SCOPE.md` SP-1
//! were both waiting on.
//!
//! **Since owner Ruling AR (2026-09-24) it also restores the world.** The
//! archive carries every `WorldState` grid the core rasters do not
//! (`SAVEFILE_COMPAT.md` §8.3, `substrate.rs`), and `project_open` installs
//! the rebuilt `WorldState` as `WorldSource::Generated`, so journey planning,
//! trade flows, military, town layouts, the faction economy, the Sample panel,
//! Erode and the rest answer on a reopened project exactly as they did before
//! the save. An archive without the substrate -- written before that date, or
//! a flat legacy one -- still opens as `WorldSource::Loaded`, restores its civ
//! layer all the same, and every readout that needs the substrate refuses with
//! `substrate::needs_substrate` (one wording for each case): recalling the civilisation layer and
//! recomputing over it are still different questions for such a world.
//!
//! ## The Journey entity (`STORY_PLANNING_SCOPE.md` SP-1)
//!
//! `entities/journeys.json` moved from the shell's own caller-built slot
//! (§9.6 was specified before anything could write it) to
//! [`ENGINE_OWNED_SLOTS`], the day a real `Journey` type
//! (`cartalith_civ::travel_library::Journey`) landed. The mutable store is
//! `InfraTools::journeys`, beside `routes` rather than a new `WorldGen`
//! field -- a journey's `route` is a **snapshot** of a committed route's
//! geometry taken at `journey_save` time (`JourneyRoute`'s own doc comment:
//! committed routes have no stable id to reference instead), so the two
//! share `InfraTools`' lifecycle (reset on `load_save`, restored together
//! below whenever the archive carries either; on a regenerate journeys are
//! re-snapped rather than reset -- SP-2, `story_bridge.rs`).
//!
//! `journey_to_dto`/`dto_to_journey` below are this document's DTOs, same
//! shape as every other entity in this file: the archive's member names
//! (§9.6: `party_preset`, nested `route`) are not this port's own
//! (`Journey::party_preset`, flat `JourneyRoute`), so the DTO is what keeps
//! a future rename of the Rust type from becoming a format break.
//!
//! **What the shell no longer owns.** Before this, `journey_planner_view.gd`
//! built and parsed this document itself, in its own ad hoc shape -- a route
//! **array index** (renumbers on delete, meaningless across a regenerate)
//! plus a raw party-form snapshot, neither `id`-bearing nor matching §9.6's
//! own long-published shape. That local, richer, in-session list
//! (`stage_overrides`/`layovers`/`animal_entries`/`trim`, the Journey
//! Planner's own working state) is **out of SP-1's scope** -- the scope
//! document is explicit that SP-1 is "the entity and its persistence only" --
//! and does not round-trip through this slot any more; see
//! `journey_planner_view.gd`'s own updated doc comment on `_save_journey`
//! for the bridge this pass wired instead (capture a real `PartyPreset`,
//! register a real `Journey`, still track the richer fields locally for the
//! duration of the session).

use crate::{
    civ_military_bridge, civ_roster_bridge, icon_bridge, infra_tools_bridge, journey_bridge,
    label_bridge, lod_bridge, params, CivData, WorldGen, WorldSource,
};
use cartalith_io::project::{self, ProjectWrite, Raster};
use godot::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// ===================== slot names =====================
//
// Named constants rather than literals at the call sites: a slot is
// registered in `cartalith-io` and used here, and a typo at one end
// produces a silently unwritten payload rather than a compile error.

/// The settlement table, with its `next_id` counter and optional `name_stream` position.
const SLOT_SETTLEMENTS: &str = "entities/settlements.json";
/// The faction roster, including tariffs, currency and religion choices.
const SLOT_FACTIONS: &str = "entities/factions.json";
/// Roads, sea lanes and the router's `road_edges`.
const SLOT_WAYS: &str = "entities/ways.json";
/// The province list.
const SLOT_PROVINCES: &str = "entities/provinces.json";
/// The continent list.
const SLOT_CONTINENTS: &str = "entities/continents.json";
/// The per-year timeline snapshots and the held territory year.
const SLOT_TIMELINE: &str = "history/timeline.json";
/// The Label tool's own label list.
const SLOT_LABELS: &str = "annotations/labels.json";
/// Manually placed and generated icons.
const SLOT_ICONS: &str = "annotations/icons.json";
/// The regions annotation document.
const SLOT_REGIONS: &str = "annotations/regions.json";
/// The look, quality, ramp, NPR, `biome_cols` and (ET-C2, optional)
/// `terrain_adjust` / `water_adjust` settings.
const SLOT_APPEARANCE: &str = "appearance.json";
/// The Markdown Vault store.
const SLOT_VAULT: &str = "vault.json";
/// Landmark settings, the last run and the refined viewshed.
const SLOT_LANDMARKS: &str = "entities/landmarks.json";
/// The saved Journey entities.
const SLOT_JOURNEYS: &str = "entities/journeys.json";
/// The Conflict entities and their `anchor_convention` marker.
const SLOT_CONFLICTS: &str = "entities/conflicts.json";
/// `FACTION_HUB_DESIGN.md` §6 R1, `SAVEFILE_COMPAT.md` §9.9: the custom
/// religion library (`CivData::religions`).
const SLOT_RELIGIONS: &str = "entities/religions.json";
/// `COLOUR_PAINT_SCOPE.md` CP-3 (Ruling CN): the colour-paint layer
/// (`WorldGen::colour_paint`), written only when it has marks.
const SLOT_COLOUR_PAINT: &str = "appearance_paint.json";

// The four caller-owned slots this file builds the *contents* of without
// owning the slot itself. They stay out of [`ENGINE_OWNED_SLOTS`] below --
// see "The four documents nothing could produce" in this module's own doc.
/// The painted-layer draft; caller-owned.
const SLOT_PAINT: &str = "drafts/paint.json";
/// The sculpt-stamp draft; caller-owned.
const SLOT_SCULPT: &str = "drafts/sculpt.json";
/// The asset-library document; caller-owned.
const SLOT_ASSETS: &str = "library/assets.json";
/// The travel-library document of custom entries; caller-owned.
const SLOT_TRAVEL: &str = "library/travel.json";

/// The slots this file writes and reads itself. A document in one of these
/// handed to [`WorldGen::project_save_with_documents`] is refused, because
/// the engine would overwrite it half a step later and the caller would
/// never know which copy won.
const ENGINE_OWNED_SLOTS: &[&str] = &[
    SLOT_SETTLEMENTS,
    SLOT_FACTIONS,
    SLOT_WAYS,
    SLOT_PROVINCES,
    SLOT_CONTINENTS,
    SLOT_TIMELINE,
    SLOT_LABELS,
    SLOT_ICONS,
    SLOT_REGIONS,
    SLOT_APPEARANCE,
    SLOT_VAULT,
    // Engine-owned for the reason every other entity table is: the payload
    // is `WorldGen::landmark_store`, which GDScript has no view of. Added
    // the day `cartalith-io` registered the slot -- an unclassified slot
    // falls through to *caller*-owned by default, and
    // `the_document_channel_partitions_every_registered_slot` exists to make
    // that arrive as a failing test rather than as a shell that can
    // overwrite the engine's own document.
    SLOT_LANDMARKS,
    // `STORY_PLANNING_SCOPE.md` SP-1: a `Journey` is real engine state
    // (`InfraTools::journeys`) since this slot's payload became a typed
    // entity rather than shell-only state. Until SP-1 this was one of the
    // shell's own caller-built slots; the module doc's "The Journey entity"
    // section describes the move. `annotations/measurements.json` stays
    // caller-owned.
    SLOT_JOURNEYS,
    // `STORY_PLANNING_SCOPE.md` SP-4: the payload is `WorldGen::conflicts`,
    // which GDScript has no view of -- the landmarks slot's reason.
    SLOT_CONFLICTS,
    // `FACTION_HUB_DESIGN.md` §6 R1: the payload is `CivData::religions`,
    // which GDScript only reaches through the `civ_*_religion` funcs -- the
    // landmarks slot's reason again.
    SLOT_RELIGIONS,
    // CP-3: the payload is `WorldGen::colour_paint`, engine state the shell
    // reaches only through the `colour_paint_*` funcs -- the landmarks
    // slot's reason.
    SLOT_COLOUR_PAINT,
];

// ===================== the document schemas =====================
//
// `SAVEFILE_COMPAT.md` §9-§11 is the specification; these are its Rust
// expression. Every field carries `#[serde(default)]` so that a document
// written by a different implementation, or by an older version of this
// one, loses only what it actually omitted -- §14.3's unknown-member rule
// in its constructive direction.

/// `entities/settlements.json` (`SAVEFILE_COMPAT.md` §9.1): the settlement table, the id counter and the manual-name stream position.
/// Every member is `#[serde(default)]`, so a foreign or older document loses only what it omitted.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct SettlementsDoc {
    #[serde(default)]
    next_id: u64,
    /// §9.1's `name_stream`: the manual-placement name/population stream's
    /// position (`CivTools::name_rng`, `Mulberry32::state`). Without it a
    /// reopened project restarts that stream from the seed and hands the next
    /// blank-named drop a name an earlier drop already drew. Omitted when
    /// there is no stream to record (no Territory/Settlement tool state), and
    /// absent in every archive written before 2026-09-24, which reopen as they
    /// always did: from the seed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    name_stream: Option<u32>,
    #[serde(default)]
    settlements: Vec<SettlementDto>,
}

/// Just §9.1's `name_stream`, read without materialising the settlements --
/// the one member of that document `civ_from_project` does not hand back,
/// because the stream belongs to `CivTools`, not `CivData`.
#[derive(Deserialize)]
struct NameStreamOnly {
    #[serde(default)]
    name_stream: Option<u32>,
}

/// One settlement as §9.1 spells it, flattened from `NamedSettlement`, its placement, trade balance and place extras.
/// `kind` is the `journey_bridge::settlement_kind_key` spelling; an unrecognised one reads back as `town` (`dto_to_settlement`).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct SettlementDto {
    #[serde(default)]
    id: u64,
    #[serde(default)]
    x: usize,
    #[serde(default)]
    y: usize,
    #[serde(default)]
    name: String,
    #[serde(default)]
    population: u32,
    #[serde(default)]
    faction: i32,
    #[serde(default)]
    kind: String,
    #[serde(default)]
    capital: bool,
    #[serde(default)]
    coastal: bool,
    #[serde(default)]
    suitability: f64,
    #[serde(default)]
    village_seeded: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    trade: Option<TradeDto>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    extras: Option<ExtrasDto>,
}

/// A settlement's trade balance as resource-key strings; read back through `resource_key_static`, so an unknown key is dropped.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct TradeDto {
    #[serde(default)]
    exports: Vec<String>,
    #[serde(default)]
    imports: Vec<String>,
}

/// A settlement's place-editor extras (`civ_roster_bridge::PlaceExtras`). `age`/`walls` are `None` for "auto"; `culture`/`rules_preset`/`variant` are Ruling J's town plan and are omitted when unset.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct ExtrasDto {
    #[serde(default)]
    specialisation: String,
    #[serde(default)]
    traits: Vec<String>,
    #[serde(default)]
    history: String,
    #[serde(default)]
    age: Option<u32>,
    #[serde(default)]
    walls: Option<bool>,
    // Ruling J's town plan. Skipped when unset so a project with no town-plan
    // edit writes exactly the bytes it wrote before these fields existed.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    culture: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    rules_preset: String,
    #[serde(default, skip_serializing_if = "is_zero")]
    variant: u32,
}

/// `skip_serializing_if` predicate: true for `0`, so a default `variant` writes no member.
fn is_zero(v: &u32) -> bool {
    *v == 0
}

/// `entities/factions.json` (§9.2): the faction roster. Its one other member is a read-only legacy value, documented on the field.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct FactionsDoc {
    #[serde(default)]
    factions: Vec<FactionDto>,
    /// **Read-only compatibility, never written again.** Ruling AZ
    /// (2026-09-28, `LARGE_ITEM_RULINGS.md`) first shipped the garrison
    /// rule's border-exposure scale (`cartalith_civ::garrison::
    /// EXPOSURE_SCALE`, `MILITARY_MANPOWER_SCOPE.md` §5.6) as this optional
    /// `factions.json` member, at commit 382945a. The fold-in that followed
    /// moved the live value onto `cartalith_engine::CivParams` --
    /// `civ.garrison_exposure_scale`, an ordinary `params::PARAMS` row now,
    /// carried in `params.json`'s `cartalith` block like every other civ
    /// dial -- so nothing writes this member any more (`#[serde(skip_
    /// serializing)]` below, unconditionally, not the old `skip_serializing_
    /// if`). It stays on this struct, and stays read, only so a project saved
    /// between those two commits still opens with its value --
    /// `resolve_garrison_exposure_scale` reads it, and only when the new
    /// `civ.garrison_exposure_scale` key never reached the archive at all.
    ///
    /// `None` -- the key is absent, or the whole file is -- means "no legacy
    /// value to fall back to", never a plausible `0.0` (`MISTAKES.md`: no
    /// value is never a plausible value; `0.0` is a real, different setting
    /// that removes the border term entirely).
    #[serde(default, skip_serializing)]
    garrison_exposure_scale: Option<f64>,
}

/// One faction roster row. `color` is the roster colour, `user_color` the user's override (`None` = none set); `tariffs` and `currency` are omitted when empty/unset.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct FactionDto {
    #[serde(default)]
    id: usize,
    #[serde(default)]
    name: String,
    #[serde(default)]
    culture: String,
    #[serde(default)]
    religion: String,
    #[serde(default)]
    government: String,
    #[serde(default)]
    ag_tech: String,
    #[serde(default)]
    color: [u8; 3],
    #[serde(default)]
    user_color: Option<[u8; 3]>,
    /// IN-13 tariffs this faction levies as importer, keyed by exporting
    /// faction id (Ruling AE). Omitted when empty, so a project with no
    /// tariff set writes the same `factions.json` it did before the field
    /// existed, and an older file reads back as "no tariffs".
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    tariffs: std::collections::BTreeMap<usize, f64>,
    /// This faction's currency (Ruling R/AR/AU; `SAVEFILE_COMPAT.md` §9.2).
    /// Omitted while the user has set none of its members, so an unedited
    /// roster writes the `factions.json` it wrote before the member existed,
    /// and an older file reads back as "no currency set" -- every member
    /// unset, shown as its default, never as a user choice.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    currency: Option<CurrencyDto>,
    /// FH-R2 R2-6 (2026-10-24, `SAVEFILE_COMPAT.md` §9.2): the faction's
    /// religious policy key. **Omitted for `"plural"`**, the default, so an
    /// archive where no policy was changed is byte-identical to the pre-R2-6
    /// `factions.json`; absent reads as `"plural"`. A raw JSON value so a
    /// wrongly typed member costs only itself (read as `"plural"`, warned),
    /// never the roster's parse.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    religious_policy: Option<serde_json::Value>,
}

/// One faction's currency as saved: each member present only when the user
/// set it. See `civ_roster_bridge::FactionCurrency`.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
struct CurrencyDto {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    symbol: Option<String>,
    /// Units of this currency per one unit of the world price index.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    rate: Option<f64>,
}

/// `entities/ways.json` (§9.3): the single home of every linear route. `civ_documents` writes `roads`, `sea_lanes` and `road_edges`; `manual` and `routes` are merged in by the caller from `InfraTools`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct WaysDoc {
    #[serde(default)]
    roads: Vec<RoadDto>,
    #[serde(default)]
    sea_lanes: Vec<SeaLaneDto>,
    #[serde(default)]
    manual: Vec<ManualWayDto>,
    #[serde(default)]
    routes: Vec<RouteDto>,
    /// `CivData::road_edges`, the auto-populate router's raw cell paths
    /// (`SAVEFILE_COMPAT.md` §9.3, since 2026-09-24). Omitted when there are
    /// none, so absent means one thing -- no router edges known -- whether the
    /// router laid none or the archive predates the member; both restore as
    /// no edges, which is how every earlier project has always reopened, and
    /// re-saving such a project leaves this document byte-identical.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    road_edges: Vec<RoadEdgeDto>,
    /// The routing-grid cap (`cartalith_civ::RW_CAP`) the router ran under,
    /// which says what grid `road_edges`' `cells` index: `min(gw, cap)` wide
    /// (`SAVEFILE_COMPAT.md` §9.3, added 2026-10-12). Written with
    /// `road_edges` and only with it. Absent in every archive written before,
    /// where the cap was either 384 (until 2026-09-28) or 1024, and
    /// `civ_from_project` decides which from the edges themselves
    /// (`cartalith_civ::road_edges_infer_grid_w`). Must never be written
    /// without edges: absent then means "no router edges known", as it always
    /// has, and an old project without edges re-saves byte-identically.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    road_edges_rw_cap: Option<usize>,
}

/// One `CivData::road_edges` entry: `from`/`to` are `RoadEdge::a`/`b`, and
/// `cells` is `RoadEdge::path` -- flat indices `y * rw + x` on the **routing
/// grid**, `rw = min(gw, road_edges_rw_cap)` wide (equal to `y * gw + x` of the
/// map only while `gw <= rw`).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
struct RoadEdgeDto {
    #[serde(default)]
    from: usize,
    #[serde(default)]
    to: usize,
    #[serde(default)]
    cells: Vec<usize>,
}

/// One generated road (`cartalith_civ::Way`): polyline `points` in grid cells, `breaks` as indices into it (range-checked on read by `sane_breaks`), `class` from `way_class_key`, `from`/`to` the endpoint settlement indices.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct RoadDto {
    #[serde(default)]
    id: u64,
    #[serde(default)]
    name: String,
    #[serde(default)]
    points: Vec<[f64; 2]>,
    #[serde(default)]
    breaks: Vec<usize>,
    #[serde(default)]
    length_km: f64,
    #[serde(default)]
    class: String,
    #[serde(default)]
    from: usize,
    #[serde(default)]
    to: usize,
    #[serde(default)]
    hidden: bool,
}

/// One generated sea route (`cartalith_civ::SeaRoute`): the polyline fields of [`RoadDto`] with no class and no endpoints.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct SeaLaneDto {
    #[serde(default)]
    name: String,
    #[serde(default)]
    points: Vec<[f64; 2]>,
    #[serde(default)]
    breaks: Vec<usize>,
    #[serde(default)]
    length_km: f64,
}

/// One hand-drawn way from `InfraTools` (`cartalith_civ::tools::ManualWay`); `kind` is `manual_way_kind_key`'s spelling.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct ManualWayDto {
    #[serde(default)]
    name: String,
    #[serde(default)]
    points: Vec<[f64; 2]>,
    #[serde(default)]
    breaks: Vec<usize>,
    #[serde(default)]
    length_km: f64,
    #[serde(default)]
    kind: String,
    #[serde(default)]
    sea: bool,
    #[serde(default)]
    hidden: bool,
}

/// One committed route from `InfraTools`: polyline, `mode` from `route_mode_key`, and the count of legs the planner could not reach.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct RouteDto {
    #[serde(default)]
    name: String,
    #[serde(default)]
    points: Vec<[f64; 2]>,
    #[serde(default)]
    breaks: Vec<usize>,
    #[serde(default)]
    length_km: f64,
    #[serde(default)]
    mode: String,
    #[serde(default)]
    unreachable_legs: usize,
}

/// `SAVEFILE_COMPAT.md` §9.7 (SP-4). `end_year` and `anchor` are omitted
/// when absent -- an ongoing conflict has no end year, an unattached one no
/// anchor -- rather than written as a sentinel.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct ConflictsDoc {
    #[serde(default)]
    next_id: u64,
    #[serde(default)]
    conflicts: Vec<ConflictDto>,
    /// Ruling AY (`LARGE_ITEM_RULINGS.md`, 2026-09-28). Additive marker, set
    /// by every writer from `36312e2` on: `"centre"` when `anchor.at` was
    /// written in the cell-*centre* convention (`conflict_bridge::anchor_pos`,
    /// `+0.5` on both axes). A document from before that fix has no member at
    /// all -- `serde`'s default gives `""`, never confused with a real value
    /// -- and its `anchor.at` is in the old *corner* convention, half a cell
    /// off where `anchor_pos` now resolves. The loader (`project_bridge.rs`'s
    /// restore, not this DTO conversion) reads this to decide whether to
    /// shift `anchor.at` by `+0.5, +0.5` before use; see `SAVEFILE_COMPAT.md`
    /// §9.7.
    #[serde(default)]
    anchor_convention: String,
}

/// One `cartalith_civ::conflict::Conflict` row. `kind` is `ConflictKind::key`; a key this build does not know costs the row on read (`dto_to_conflict`). `end_year` `None` = ongoing; `sides` are faction ids.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct ConflictDto {
    #[serde(default)]
    id: u64,
    #[serde(default)]
    name: String,
    #[serde(default)]
    kind: String,
    #[serde(default)]
    start_year: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    end_year: Option<i64>,
    #[serde(default)]
    sides: Vec<i32>,
    #[serde(default)]
    outcome: String,
    /// As drawn, grid cells -- **not** resolved against the anchor, so a
    /// reopened project translates them exactly as the live one did.
    #[serde(default)]
    points: Vec<[f64; 2]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    anchor: Option<ConflictAnchorDto>,
}

/// A conflict's attachment to a settlement or province: which kind, its `tid`, and where the anchor stood when attached.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct ConflictAnchorDto {
    /// `"settlement"` or `"province"`.
    #[serde(default)]
    kind: String,
    /// The settlement's `tid`; for a province, its seed settlement's `tid`.
    #[serde(default)]
    tid: u64,
    /// Where the anchor stood when attached (`Conflict::anchor_at`).
    #[serde(default)]
    at: [f64; 2],
}

/// Live `Conflict` to its row; the inverse of [`dto_to_conflict`]. `anchor` is written only when the conflict is attached, with `anchor_at` as stored (cell-centre convention, see [`ConflictsDoc`]).
fn conflict_to_dto(c: &cartalith_civ::conflict::Conflict) -> ConflictDto {
    ConflictDto {
        id: c.id,
        name: c.name.clone(),
        kind: c.kind.key().to_string(),
        start_year: c.start_year,
        end_year: c.end_year,
        sides: c.sides.clone(),
        outcome: c.outcome.clone(),
        points: c.points.iter().map(|&(x, y)| [x, y]).collect(),
        anchor: c.anchor.map(|a| {
            let (kind, tid) = crate::conflict_bridge::anchor_key(a);
            ConflictAnchorDto { kind: kind.to_string(), tid, at: [c.anchor_at.0, c.anchor_at.1] }
        }),
    }
}

/// `None` for a row this build cannot read (unknown kind): that row is
/// skipped, the rest of the document opens (§6.4a rung 3).
fn dto_to_conflict(d: &ConflictDto) -> Option<cartalith_civ::conflict::Conflict> {
    let anchor = d.anchor.as_ref().and_then(|a| crate::conflict_bridge::anchor_from_key(&a.kind, a.tid));
    Some(cartalith_civ::conflict::Conflict {
        id: d.id,
        name: d.name.clone(),
        kind: cartalith_civ::conflict::ConflictKind::from_key(&d.kind)?,
        start_year: d.start_year,
        end_year: d.end_year,
        sides: d.sides.clone(),
        outcome: d.outcome.clone(),
        points: d.points.iter().map(|p| (p[0], p[1])).collect(),
        anchor,
        anchor_at: d.anchor.as_ref().filter(|_| anchor.is_some()).map_or((0.0, 0.0), |a| (a.at[0], a.at[1])),
    })
}

/// `entities/conflicts.json`'s full read: every parseable row (an unknown
/// `kind` is dropped, `dto_to_conflict`'s own rule), migrated for Ruling AY
/// (`LARGE_ITEM_RULINGS.md`, 2026-09-28) if the document predates the
/// cell-centre fix (`36312e2`) -- no `anchor_convention: "centre"` marker.
///
/// **Only an anchored conflict's `anchor_at` moves, by `+0.5` on both axes.**
/// A document written before the fix stored `anchor_at` in the settlement's
/// raw cell (the corner); `conflict_bridge::anchor_pos` now always resolves
/// to the centre, so reading an unmigrated `anchor_at` against it would shift
/// the drawn shape by half a cell the moment its anchor is next read, even
/// though nothing about the anchored place moved. Migrating it once here
/// keeps `resolved_points`'s `anchor_now - anchor_at` delta exactly what the
/// pre-fix build computed. A free (unanchored) conflict has no `anchor_at`
/// in either convention and is never touched. See `SAVEFILE_COMPAT.md` §9.7.
fn conflicts_from_doc_migrated(doc: &ConflictsDoc) -> Vec<cartalith_civ::conflict::Conflict> {
    let migrate_pre_fix_anchor = doc.anchor_convention != "centre";
    let mut conflicts: Vec<_> = doc.conflicts.iter().filter_map(dto_to_conflict).collect();
    if migrate_pre_fix_anchor {
        for c in &mut conflicts {
            if c.anchor.is_some() {
                c.anchor_at.0 += 0.5;
                c.anchor_at.1 += 0.5;
            }
        }
    }
    conflicts
}

/// `entities/religions.json`, `SAVEFILE_COMPAT.md` §9.9 (`FACTION_HUB_DESIGN.md`
/// §6 R1): the project's custom religions. **Additive**: a reader that
/// predates it ignores the member (§14.3), and a project that never created a
/// religion never writes it, so its archive is unchanged byte for byte.
///
/// `next_id` is saved, not re-derived, because ids are never reused: after a
/// delete the highest surviving id may be lower than an id a reader once saw.
/// It is raised past every row present on load (§9.1's rule) and never read
/// below `1`.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
struct ReligionsDoc {
    #[serde(default)]
    next_id: u32,
    #[serde(default)]
    religions: Vec<ReligionDto>,
}

/// One custom religion as saved. Every member `#[serde(default)]`, so a row
/// missing one is judged by `ReligionLibrary::from_saved`'s validation (an
/// empty name or base fails it and costs that row alone), never by a parse
/// failure that would cost the whole document.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
struct ReligionDto {
    #[serde(default)]
    id: u32,
    #[serde(default)]
    name: String,
    /// One of the `CIV_RELIGIONS` keys -- the engine behaviour.
    #[serde(default)]
    base_key: String,
    #[serde(default)]
    colour: [u8; 3],
    #[serde(default, skip_serializing_if = "String::is_empty")]
    notes: String,
    /// FH-R2 R2-3 (2026-10-24, `SAVEFILE_COMPAT.md` §9.9): the religion's axis
    /// overrides as `{axis: value_key}`, only the axes the user set (absent =
    /// "as base"), `locus` as a one-element list. **Omitted when there is no
    /// override**, so an archive with no edited religion is byte-identical to
    /// the pre-R2-3 format. Held as a raw JSON value, not a typed struct, so a
    /// wrongly typed or unknown member costs that one axis (`axes_from_json`)
    /// and never the whole document's parse.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    axes: Option<serde_json::Value>,
}

/// `AxisOverrides` as saved: only the overridden axes, by their stable keys.
/// `None` for no override, so the member is not written at all.
fn axes_to_json(o: &cartalith_civ::religion_profile::AxisOverrides) -> Option<serde_json::Value> {
    use cartalith_civ::religion_profile::AxisValue;
    if o.is_empty() {
        return None;
    }
    let mut m = serde_json::Map::new();
    if let Some(v) = o.object {
        m.insert("object".into(), v.key().into());
    }
    if let Some(v) = o.exclusivity {
        m.insert("exclusivity".into(), v.key().into());
    }
    if let Some(v) = o.membership {
        m.insert("membership".into(), v.key().into());
    }
    if let Some(v) = o.locus {
        m.insert("locus".into(), serde_json::Value::Array(vec![v.key().into()]));
    }
    if let Some(v) = o.institution {
        m.insert("institution".into(), v.key().into());
    }
    Some(serde_json::Value::Object(m))
}

/// Reads a saved `axes` member back, axis by axis. A member this build does
/// not know (a newer build's axis), a value key it does not know, or a wrong
/// type reads as "as base" for that axis alone and is named in `bad`, so the
/// caller can warn; it never guesses a neighbouring value
/// (`MISTAKES.md`, "Return, marshal or default a value that can be absent").
fn axes_from_json(v: Option<&serde_json::Value>, bad: &mut Vec<String>) -> cartalith_civ::religion_profile::AxisOverrides {
    use cartalith_civ::religion_profile::{AxisOverrides, AxisValue, Exclusivity, Institution, Locus, Membership, WorshipObject};
    let mut o = AxisOverrides::default();
    let Some(v) = v else { return o };
    let Some(m) = v.as_object() else {
        bad.push("axes".to_string());
        return o;
    };
    fn one<T: AxisValue>(m: &serde_json::Map<String, serde_json::Value>, k: &str, bad: &mut Vec<String>) -> Option<T> {
        let v = m.get(k)?;
        let key = match v {
            serde_json::Value::String(s) => Some(s.as_str()),
            // `locus` is saved as a list; one element is this build's form.
            serde_json::Value::Array(a) if a.len() == 1 => a[0].as_str(),
            _ => None,
        };
        let got = key.and_then(T::from_key);
        if got.is_none() {
            bad.push(k.to_string());
        }
        got
    }
    o.object = one::<WorshipObject>(m, "object", bad);
    o.exclusivity = one::<Exclusivity>(m, "exclusivity", bad);
    o.membership = one::<Membership>(m, "membership", bad);
    o.locus = one::<Locus>(m, "locus", bad);
    o.institution = one::<Institution>(m, "institution", bad);
    for k in m.keys() {
        if !["object", "exclusivity", "membership", "locus", "institution"].contains(&k.as_str()) {
            bad.push(k.clone());
        }
    }
    o
}

/// The document for a library, or `None` when nothing was ever created (the
/// slot is then absent, not empty -- the journeys/conflicts rule).
fn religions_doc(lib: &cartalith_civ::religion_library::ReligionLibrary) -> Option<ReligionsDoc> {
    (!lib.is_pristine()).then(|| ReligionsDoc {
        next_id: lib.next_id(),
        religions: lib
            .defs()
            .iter()
            .map(|d| ReligionDto {
                id: d.id,
                name: d.name.clone(),
                base_key: d.base_key.clone(),
                colour: [d.colour.0, d.colour.1, d.colour.2],
                notes: d.notes.clone(),
                axes: axes_to_json(&d.axes),
            })
            .collect(),
    })
}

/// Reads `entities/religions.json` back into a library. **Absent is an empty
/// library with no warning** -- every project saved before R1, and every
/// project that never made a religion. A document that does not parse, or rows
/// `ReligionLibrary::from_saved` drops, are reported in `warnings` (§6.4a rung
/// 3: they cost themselves, never the archive); a faction that named a lost
/// row then reads as "Missing religion" in the shell, not as a guessed faith.
fn religions_from_data(
    data: &project::ProjectData,
    warnings: &mut Vec<String>,
) -> cartalith_civ::religion_library::ReligionLibrary {
    use cartalith_civ::religion_library::{ReligionDef, ReligionLibrary};
    match data.parse::<ReligionsDoc>(SLOT_RELIGIONS) {
        None => ReligionLibrary::new(),
        Some(Err(_)) => {
            warnings.push(format!(
                "{SLOT_RELIGIONS}: unreadable -- custom religions were not restored; factions that held one show it as a missing religion and behave as having none"
            ));
            ReligionLibrary::new()
        }
        Some(Ok(doc)) => {
            let mut unread_axes: Vec<String> = Vec::new();
            let rows = doc
                .religions
                .into_iter()
                .map(|r| {
                    let mut bad = Vec::new();
                    let axes = axes_from_json(r.axes.as_ref(), &mut bad);
                    if !bad.is_empty() {
                        unread_axes.push(format!("{} ({})", r.name, bad.join(", ")));
                    }
                    ReligionDef {
                        id: r.id,
                        name: r.name,
                        base_key: r.base_key,
                        colour: (r.colour[0], r.colour[1], r.colour[2]),
                        notes: r.notes,
                        axes,
                    }
                })
                .collect();
            if !unread_axes.is_empty() {
                warnings.push(format!(
                    "{SLOT_RELIGIONS}: some custom religion beliefs could not be read and behave as their base religion's: {}",
                    unread_axes.join("; ")
                ));
            }
            let (lib, dropped) = ReligionLibrary::from_saved(doc.next_id, rows);
            if dropped > 0 {
                warnings.push(format!(
                    "{SLOT_RELIGIONS}: {dropped} custom religion row{} could not be read (bad id, duplicate, unknown base religion or invalid name) and {} dropped",
                    if dropped == 1 { "" } else { "s" },
                    if dropped == 1 { "was" } else { "were" }
                ));
            }
            lib
        }
    }
}

/// `entities/journeys.json`, `SAVEFILE_COMPAT.md` §9.6. `next_id` mirrors
/// `SettlementsDoc::next_id`'s own convention (§9.1: "raised to `max(id)+1`
/// if the stored value is lower").
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct JourneysDoc {
    #[serde(default)]
    next_id: u64,
    #[serde(default)]
    journeys: Vec<JourneyDto>,
}

/// One `Journey` as §9.6 spells it: a nested route snapshot ([`JourneyRouteDto`]), the party preset key, start year and the optional resupply stops and saved plan. Converted by [`journey_to_dto`]/[`dto_to_journey`].
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct JourneyDto {
    #[serde(default)]
    id: u64,
    #[serde(default)]
    name: String,
    #[serde(default)]
    party_preset: String,
    #[serde(default)]
    route: JourneyRouteDto,
    #[serde(default)]
    start_year: i64,
    /// Ruling BS (2026-09-28): the accepted resupply stops. `default` so a
    /// document written before the ruling loads with none -- which is how
    /// those journeys were planned -- and skipped when empty so re-saving
    /// such a journey writes the same bytes it was read from.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    resupply_stops: Vec<AcceptedStopDto>,
    /// Ruling AR (2026-09-24): the whole saved plan -- see
    /// [`JourneyPlanDto`]. **Absent means "no plan stored"** (every journey
    /// saved before the ruling) and is never filled in with a default plan:
    /// such a journey is planned from its `party_preset`, as it always was.
    /// Skipped when `None`, so re-saving a pre-ruling journey writes the
    /// bytes it was read from.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    plan: Option<JourneyPlanDto>,
}

/// Ruling AR: `cartalith_civ::travel_library::JourneyPlan` on the wire,
/// `SAVEFILE_COMPAT.md` 9.6's `plan` member.
///
/// `fields` is `journey_bridge::plan_to_pairs`' flat vocabulary -- the very
/// keys `jp_compute`'s `plan` Dictionary and `jp_default_plan()` speak -- so
/// a plan field the planner gains is one new key here and needs no format
/// change. `stage_overrides` nests `stage_override_to_pairs` per stage index
/// (the index as a decimal string, JSON object keys being strings).
/// `BTreeMap`s throughout, so the same plan always writes the same bytes.
///
/// **Tolerance, both ways** (`SAVEFILE_COMPAT.md` 14.3): an unknown `fields`
/// key or an unparsable stage index is dropped, a missing member takes the
/// planner's own default for "not set" (`trim` absent = the whole route,
/// `auto_*` absent = off, `fields` keys absent = the reference defaults).
/// A reader never refuses the journey over its plan.
///
/// No `Default` derive on purpose: `[f64; 2]`'s `Default` is `[0, 0]`, an
/// empty trim that plans nothing -- exactly the plausible-looking
/// "no value" `MISTAKES.md` forbids; `trim` has its own `serde` default.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct JourneyPlanDto {
    #[serde(default)]
    fields: BTreeMap<String, serde_json::Value>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    stage_overrides: BTreeMap<String, BTreeMap<String, serde_json::Value>>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    layovers: BTreeMap<String, i64>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    animal_entries: BTreeMap<String, String>,
    #[serde(default = "whole_route_trim")]
    trim: [f64; 2],
    #[serde(default)]
    auto_carriage: bool,
    #[serde(default)]
    auto_stage: bool,
}

/// [`JourneyPlanDto::trim`]'s serde default: the untrimmed route, the
/// planner's own `Vector2(0, 1)` ("not sent at all").
fn whole_route_trim() -> [f64; 2] {
    [0.0, 1.0]
}

/// One accepted resupply stop (Ruling BS), `cartalith_civ::JpAcceptedStop`
/// field for field; `at` is the route point the stop is taken at, in grid
/// coordinates, in the same `[x, y]` pair form the route's own `points` use.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct AcceptedStopDto {
    #[serde(default)]
    key: String,
    #[serde(default)]
    kind: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    at: [f64; 2],
}

/// §9.6's embedded route snapshot -- deliberately a leaner sibling of
/// [`RouteDto`] (no `name`, no `unreachable_legs`): those two describe a
/// *committed route in the tool's own list*, and a journey's route is a copy
/// taken out of one, not that entry itself.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct JourneyRouteDto {
    #[serde(default)]
    points: Vec<[f64; 2]>,
    #[serde(default)]
    breaks: Vec<usize>,
    #[serde(default)]
    length_km: f64,
    #[serde(default)]
    mode: String,
}

/// `entities/provinces.json`; written only when the province list is non-empty.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct ProvincesDoc {
    #[serde(default)]
    provinces: Vec<ProvinceDto>,
}

/// One province row; `capital_settlement_index` indexes the settlements list, and an index that points nowhere costs the province on read (§9.4).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct ProvinceDto {
    #[serde(default)]
    id: i32,
    #[serde(default)]
    faction: i32,
    #[serde(default)]
    name: String,
    #[serde(default)]
    capital_settlement_index: usize,
}

/// `entities/continents.json`; written only when `CivData::continents` is non-empty.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct ContinentsDoc {
    #[serde(default)]
    continents: Vec<ContinentDto>,
}

/// One `cartalith_civ::Continent`: id, name, cell count, inclusive bounding box, cell-space centroid (`cx`, `cy`) and the faction holding most of it.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct ContinentDto {
    #[serde(default)]
    id: i32,
    #[serde(default)]
    name: String,
    #[serde(default)]
    cells: usize,
    #[serde(default)]
    min_x: usize,
    #[serde(default)]
    min_y: usize,
    #[serde(default)]
    max_x: usize,
    #[serde(default)]
    max_y: usize,
    #[serde(default)]
    cx: f64,
    #[serde(default)]
    cy: f64,
    #[serde(default)]
    faction: i32,
}

/// `history/timeline.json` (§10.1): the year cursor and one snapshot per recorded year. Each year's territory raster is a separate `history/territory/<year>.i32` (§10.2), not part of this document.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct TimelineDoc {
    #[serde(default)]
    year: i64,
    #[serde(default)]
    years: Vec<TimelineYearDto>,
    /// `CivData::territory_year` (Ruling AT): the recorded year whose snapshot
    /// the live claim grid last came from. Absent when it came from none, and
    /// in every archive written before 2026-09-24 -- both restore as `None`.
    /// A reader keeps it only when it names a year in `years`
    /// (`SAVEFILE_COMPAT.md` §10.1).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    territory_year: Option<i64>,
}

/// One recorded year: its settlements and ways (flattened like the live ones) plus the optional institutions and place-override records. No territory raster -- that is stored separately.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct TimelineYearDto {
    #[serde(default)]
    year: i64,
    #[serde(default)]
    settlements: Vec<SettlementDto>,
    #[serde(default)]
    ways: Vec<RoadDto>,
    /// The roster institutions in force when this year was recorded from live state, by
    /// faction id (`SAVEFILE_COMPAT.md` §10.1, added 2026-10-06). **Absent = not recorded**
    /// (every earlier archive, a collapse-run year, a year carried forward from an
    /// unrecorded one): the military reading then takes today's. Never written as `[]`
    /// for "unknown" -- an empty array would read as "recorded: no factions".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    institutions: Option<Vec<YearInstitutionsDto>>,
    /// The place-editor overrides in force when this year was recorded, one row per
    /// settlement that had any. **Absent = not recorded**; present-and-empty = recorded,
    /// no overrides. Same skip rule as `institutions`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    place_overrides: Option<Vec<YearPlaceOverrideDto>>,
}

/// One faction's recorded ag-tech and government keys (the array index is the faction id).
/// Both keys are the roster's own, stored verbatim; see
/// [`cartalith_civ::timeline::YearFactionInstitutions`].
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct YearInstitutionsDto {
    #[serde(default)]
    ag_tech: String,
    #[serde(default)]
    government: String,
}

/// One settlement's recorded military-relevant place overrides, keyed by its stable `tid`.
/// `walls` and `age` are absent for "auto" (never a plausible `false` / `0`); `traits` and
/// `specialisation` follow the place table's own empty-means-none convention.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct YearPlaceOverrideDto {
    tid: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    walls: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    age: Option<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    traits: Vec<String>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    specialisation: String,
}

/// Snapshot record -> its two optional DTO members. `None` stays `None` (omitted on write).
fn year_record_to_dtos(
    r: &cartalith_civ::timeline::YearRecord,
) -> (Option<Vec<YearInstitutionsDto>>, Option<Vec<YearPlaceOverrideDto>>) {
    let inst = r.institutions.as_ref().map(|v| {
        v.iter()
            .map(|i| YearInstitutionsDto { ag_tech: i.ag_tech.clone(), government: i.government.clone() })
            .collect()
    });
    let over = r.place_overrides.as_ref().map(|m| {
        m.iter()
            .map(|(&tid, o)| YearPlaceOverrideDto {
                tid,
                walls: o.walls,
                age: o.age,
                traits: o.traits.clone(),
                specialisation: o.specialisation.clone(),
            })
            .collect()
    });
    (inst, over)
}

/// The inverse of [`year_record_to_dtos`]. A duplicate `tid` in a hand-edited file keeps the
/// last row (`BTreeMap::insert`), which is the only deterministic choice available.
fn year_record_from_dtos(y: &TimelineYearDto) -> cartalith_civ::timeline::YearRecord {
    cartalith_civ::timeline::YearRecord {
        institutions: y.institutions.as_ref().map(|v| {
            v.iter()
                .map(|i| cartalith_civ::timeline::YearFactionInstitutions {
                    ag_tech: i.ag_tech.clone(),
                    government: i.government.clone(),
                })
                .collect()
        }),
        place_overrides: y.place_overrides.as_ref().map(|v| {
            v.iter()
                .map(|o| {
                    (
                        o.tid,
                        cartalith_civ::timeline::YearPlaceOverride {
                            walls: o.walls,
                            age: o.age,
                            traits: o.traits.clone(),
                            specialisation: o.specialisation.clone(),
                        },
                    )
                })
                .collect()
        }),
    }
}

/// `annotations/labels.json`: every label in the label editor's list.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct LabelsDoc {
    #[serde(default)]
    labels: Vec<LabelDto>,
}

/// One label row as `label_bridge` stores it: grid position, text, angle, arc, size, optional font/colour, `size_mode` (`label_size_mode_key`) and `class` (`LabelClass::key`).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct LabelDto {
    #[serde(default)]
    x: f64,
    #[serde(default)]
    y: f64,
    #[serde(default)]
    name: String,
    #[serde(default)]
    angle: f64,
    #[serde(default)]
    arc: f64,
    #[serde(default)]
    size: f64,
    #[serde(default)]
    font: Option<String>,
    #[serde(default)]
    color: Option<String>,
    #[serde(default)]
    size_mode: String,
    /// A `LabelClass::key`. `#[serde(default)]` so an archive written before
    /// the class field existed deserialises to the empty string, which
    /// `LabelClass::from_key` rejects and the reader below resolves to the
    /// default class — the same "an older archive opens, it does not fail"
    /// contract every other field here ships on.
    #[serde(default)]
    class: String,
}

/// `annotations/icons.json` — every icon in `IconEditor::icons`.
///
/// # Owner ruling 14, built 2026-09-06
///
/// *"How does a generated landmark relate to the existing manual icon
/// tool?"* — `LANDMARK_GENERATION_SCOPE.md` §4's open question 6, which asks
/// it as *"one representation, two origins, or a wholly separate data/render
/// path"* — was answered on 2026-09-06: `LARGE_ITEM_RULINGS.md` 14,
/// *"ONE LAYER, TWO ORIGINS. One collection with an `origin` field. The
/// renderer draws one layer; M6 spacing sees everything, so generation
/// cannot place a landmark on top of a hand-placed icon; a regenerate
/// replaces only the generated ones."*
///
/// **Three producers write into one list, and the generated one now marks
/// its output**: click-placement ([`icon_bridge::IconEditor::place`]) and
/// the density brush ([`icon_bridge::IconEditor::brush_stamp`], 2026-09-03)
/// write `cartalith_assets::manual::IconOrigin::Manual`; the generated
/// placement pass ([`icon_bridge::IconEditor::generate`]) — whose `POI`
/// family reads `landmark_store.last.landmarks` directly — writes
/// `Generated`. This comment used to record the field as the missing half
/// and is rewritten rather than deleted, because the migration rule it
/// fixed is still the rule.
///
/// **The wire spelling of `Manual` is an absent member, not `"manual"`.**
/// [`IconDto::origin`] is `#[serde(skip_serializing_if = "String::is_empty")]`
/// and the writer ([`icon_to_dto`]) emits nothing for a hand-placed row, so a
/// project written before this field existed opens and re-saves
/// **byte-identically** (`SAVEFILE_COMPAT.md` §6.2's "without data loss";
/// asserted by `a_document_written_before_origin_reopens_and_re_serialises_byte_identically`).
/// That is the same fact from the other side: absent means `Manual`, so
/// writing `"manual"` would add bytes that carry no information and rewrite
/// every existing project on first open.
///
/// [`icon_from_dto`] is the one seam the load half lands on, and the
/// migration rule is fixed by the file's own history: **a document written
/// before `origin` existed carries only hand-placed icons**, because it
/// predates the generated pass having anywhere to write. Absent therefore
/// reads as `Manual`, not as unknown. A value that is neither known string
/// costs the row, the answer `family` already gives.
///
/// **Not to be confused with `world.origin`** (`SAVEFILE_COMPAT.md` §16's
/// `"gen"`/`"import"`/`"region"`, `bake_bridge::ORIGIN_GENERATED`), which is
/// a different member in a different document with a different vocabulary —
/// how the *height field* was produced, not who placed an icon.
///
/// **Generated icons are persisted here exactly as they always were**, as
/// rows of this document, and this change adds only the `origin` marker to
/// them. Whether they *should* live here at all — rather than being
/// re-derived from `entities/landmarks.json` on open — is a separate
/// decision that is not taken by this field.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct IconsDoc {
    #[serde(default)]
    icons: Vec<IconDto>,
}

/// One row of `annotations/icons.json`; see [`IconsDoc`] for the `origin` rule and [`icon_to_dto`]/[`icon_from_dto`] for the conversion.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct IconDto {
    #[serde(default)]
    x: f64,
    #[serde(default)]
    y: f64,
    #[serde(default)]
    family: String,
    #[serde(default)]
    slot: String,
    #[serde(default)]
    set: Option<String>,
    #[serde(default)]
    scale: f64,
    /// An `IconOrigin::key()`, or the empty string for "the member was not
    /// there" — which is what a pre-2026-09-06 document and every
    /// hand-placed row both look like. `skip_serializing_if` is what keeps
    /// those two the same bytes; see [`IconsDoc`]'s own comment for why that
    /// is the honest spelling of `Manual` rather than an omission.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    origin: String,
}

/// One `ManualIcon` as a row of `annotations/icons.json` — the inverse of
/// [`icon_from_dto`], and extracted from `project_save_with_documents` for
/// the same reason that one was extracted from `project_open`: owner ruling
/// 14's `origin` gets exactly one seam on each side, and the writer's own
/// choice to spell `Manual` as an absent member becomes something a test can
/// assert instead of something a reader has to take on trust.
fn icon_to_dto(i: &cartalith_assets::manual::ManualIcon) -> IconDto {
    use cartalith_assets::manual::IconOrigin;
    IconDto {
        x: i.x,
        y: i.y,
        family: i.family.key().to_string(),
        slot: i.slot.clone(),
        set: i.set.clone(),
        scale: i.scale,
        // `Manual` is written as absent, so opening and re-saving a project
        // that predates this field does not rewrite it.
        origin: match i.origin {
            IconOrigin::Manual => String::new(),
            o => o.key().to_string(),
        },
    }
}

/// One row of `annotations/icons.json`, resolved — or `None` when the row
/// names a **family** that cannot be placed at all, or an **`origin`** that
/// is present and unrecognised. Those are the only two drops; everything
/// else here is a substitution.
///
/// **Extracted from `project_open`'s own closure so that owner ruling 14's
/// `origin` has exactly one seam to land on**, and so the two substitutions
/// below can be asserted rather than only described. Nothing about the
/// resolution changed in the extraction; the reporting of the drop did (see
/// the call site). `origin` landed on that seam 2026-09-06.
///
/// Two of §6.4a's rungs meet here and take opposite answers, which is the
/// only thing in this function worth reading twice:
///
/// * **An unresolvable `slot` is kept.** §11.2 states the substitution — the
///   reader renders a placeholder — because dropping it would lose the
///   author's work over an art-pack mismatch. `slot` is carried as the
///   archive's own string and resolved at draw time.
/// * **An unresolvable `family` costs the row.** There is no family to place
///   it in — `ManualIconFamily::from_key` answers the four §11.2 names plus
///   this port's own `seamarks`, and every consumer, the gallery, the hit
///   test and the pack resolver, indexes by it. Guessing `feature` would put
///   the icon under art the author never chose, which is §6.4a's "inventing
///   the world" rather than reporting a gap.
///
/// `scale` is §6.4a rung 1: a non-positive or absent multiplier is
/// substituted with `1.0`, the value a plain click placement writes. **That
/// substitution is not reported**, and §6.4a's closing rule says every one
/// should be. It is left as it is rather than half-fixed, because it is not
/// alone: `dto_to_settlement` reads an unrecognised `kind` as
/// `SettlementKind::Town` (§9.1's own stated substitution) and `sane_breaks`
/// silently discards every out-of-range break index, both unreported. Fixing
/// one member of that set would read as though the set were covered. The two
/// **row** drops in the restore pass — this function's, and
/// `LandmarkRunDto::into_result`'s — do report, and they are the two
/// `filter_map`s in it.
fn icon_from_dto(d: &IconDto) -> Option<cartalith_assets::manual::ManualIcon> {
    let family = cartalith_assets::manual::ManualIconFamily::from_key(&d.family)?;
    // Owner ruling 14's third answer, and the only one of the three that is a
    // *migration* rather than a substitution: an absent `origin` is not a
    // missing value to be guessed at, it is a document that predates the
    // generated placement pass existing at all, so every row in it was placed
    // by hand. An `origin` that is present but neither known string takes
    // `family`'s answer above -- the row costs itself -- rather than being
    // folded into `Manual`, which would silently relabel a newer writer's
    // third origin as hand-placed.
    let origin = if d.origin.is_empty() {
        cartalith_assets::manual::IconOrigin::Manual
    } else {
        cartalith_assets::manual::IconOrigin::from_key(&d.origin)?
    };
    Some(cartalith_assets::manual::ManualIcon {
        x: d.x,
        y: d.y,
        family,
        slot: d.slot.clone(),
        set: d.set.clone(),
        scale: if d.scale > 0.0 { d.scale } else { 1.0 },
        origin,
    })
}

// No `Debug`: `render::Npr` has none, and deriving one there for a struct
// nothing ever prints would be a change to the renderer for this file's
// convenience.
/// `SAVEFILE_COMPAT.md` §13.2. Every member optional, and a reader may
/// ignore the whole file and still render the world with its own defaults --
/// which is what makes this presentation and not state.
///
/// `appearance_preset` (`GUI_GAP_REGISTER.md` CA-08's loaded preset) is
/// deliberately **not** here: a preset is a complete, separately-saved
/// description of a look with its own file format, and embedding a copy in
/// every project would give the same look two homes that could then disagree.
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
struct AppearanceDoc {
    quality: String,
    look: String,
    territory_opacity: f64,
    /// A `BTreeMap`, not the `HashMap` `WorldGen::appearance_over` is: a hash
    /// map serialises in a per-process random order, so the same project
    /// saved twice produced two different `appearance.json`s and no load/save
    /// cycle could be byte-identical (found 2026-09-24 writing CA-19's
    /// backward-compatibility test, which asserts exactly that). The keys are
    /// the same; only their order on disk is now fixed (sorted).
    overrides: BTreeMap<String, f64>,
    ramp: Option<crate::render::ElevationRamp>,
    npr: crate::render::Npr,
    /// CA-19 (`LARGE_ITEM_RULINGS.md` Ruling P): the biome colour table, as
    /// all fifteen `[r, g, b]` entries in `CART_BIOME_COLS` order (class 1
    /// first) -- the shape `TerrainAppearance::biome_cols` already has in a
    /// saved-look preset file, so the two formats spell the table the same
    /// way. `SAVEFILE_COMPAT.md` §13.2 has the format note.
    ///
    /// **Absent means the reference's own table**, and the writer omits the
    /// member when the table equals it ([`biome_cols_member`]), so a project
    /// nobody recoloured is byte-identical to one written before the member
    /// existed. Kept as a raw `Value` rather than a typed array on purpose:
    /// a typed member that failed to parse would fail the **whole**
    /// `AppearanceDoc`, and one bad table would silently cost the look, the
    /// overrides, the ramp and the NPR block with it. [`biome_overrides_from_member`]
    /// validates it on its own and `project_open` reports a refusal.
    #[serde(skip_serializing_if = "Option::is_none")]
    biome_cols: Option<serde_json::Value>,
    /// ET-C2 (`EDITING_TOOLS_SCOPE.md` milestone C): the session's own terrain
    /// adjustment stack (`WorldGen::appearance_terrain_adjust`), in
    /// `AdjustStack`'s saved form (`{"entries": [...]}`).
    ///
    /// **Absent means no stack**, and the writer omits the member when the
    /// session set none or set an empty one ([`adjust_member`]), so a project
    /// that never used a stack is byte-identical to one written before the
    /// member existed. A raw `Value` for `biome_cols`'s reason: a stack this
    /// build refuses ([`adjust_from_member`]: unparsable, or out of range by
    /// `AdjustStack::validate`) costs itself only, with a warning, never the
    /// rest of the look.
    #[serde(skip_serializing_if = "Option::is_none")]
    terrain_adjust: Option<serde_json::Value>,
    /// ET-C2: the water stack; see [`Self::terrain_adjust`].
    #[serde(skip_serializing_if = "Option::is_none")]
    water_adjust: Option<serde_json::Value>,
}

/// `AppearanceDoc::terrain_adjust` / `water_adjust` as the writer spells them:
/// the stack's saved form, or `None` -- **no member** -- when the session has
/// no stack for the layer or an empty one. Empty is written as nothing for
/// `biome_cols_member`'s reason: absent already means "no adjustment", and a
/// second spelling of that state would make an untouched project differ from
/// the one the previous writer produced. A stack of disabled entries is not
/// empty and IS written (it is the user's data).
fn adjust_member(stack: Option<&crate::render::adjust::AdjustStack>) -> Option<serde_json::Value> {
    stack.filter(|s| !s.is_empty()).and_then(|s| serde_json::to_value(s).ok())
}

/// The inverse of [`adjust_member`]: `Ok(None)` for an absent member, the
/// stack for a member that parses **and validates**, and the reason for any
/// other. All or nothing, `biome_overrides_from_member`'s rule: a stack with
/// one bad entry is refused whole rather than loaded without it.
fn adjust_from_member(member: Option<&serde_json::Value>) -> Result<Option<crate::render::adjust::AdjustStack>, String> {
    let Some(v) = member else { return Ok(None) };
    let stack: crate::render::adjust::AdjustStack = serde_json::from_value(v.clone()).map_err(|e| format!("is not an adjustment stack ({e})"))?;
    stack.validate().map_err(|e| format!("is refused: {e}"))?;
    Ok(Some(stack))
}

/// `AppearanceDoc::biome_cols` as the writer spells it: the reference table
/// with `WorldGen::biome_col_overrides` laid over it, or `None` -- **no
/// member** -- when that equals `CART_BIOME_COLS`.
///
/// Why omit rather than write the default: absent already means "the
/// reference's own table" to every reader (§13.2), so writing the default
/// would be a second spelling of the same state, and it would make every
/// untouched project differ from the one the previous writer produced. An
/// override that happens to equal its class's default colour is dropped by
/// the same rule; it draws identically, and a project does not save the
/// loaded preset that is the only base it could ever differ from.
///
/// Over `CART_BIOME_COLS`, not over `appearance()`: the table here is the
/// user's edits, the member `appearance_over` is to the scalar tunables. A
/// loaded preset's own table is deliberately not written into the project
/// (see [`AppearanceDoc`]'s note on `appearance_preset`).
fn biome_cols_member(overrides: &[Option<(u8, u8, u8)>; 15]) -> Option<serde_json::Value> {
    let base = crate::render::CART_BIOME_COLS;
    let table: [(u8, u8, u8); 15] = std::array::from_fn(|i| overrides[i].unwrap_or(base[i]));
    if table == base {
        return None;
    }
    serde_json::to_value(table).ok()
}

/// The inverse of [`biome_cols_member`]: the overrides a stored table
/// describes. An absent member (`None`) is the reference table, i.e. no
/// override at all -- never a table of zeros.
///
/// **All or nothing.** Anything but exactly fifteen `[r, g, b]` triples of
/// integers in `0..=255` is refused as a whole with the reason, and the
/// caller falls back to the default table: applying the entries that did
/// parse would draw a palette nobody chose. Entries equal to the reference
/// colour come back as `None`, the writer's own rule.
fn biome_overrides_from_member(
    member: Option<&serde_json::Value>,
) -> Result<[Option<(u8, u8, u8)>; 15], String> {
    let Some(v) = member else { return Ok([None; 15]) };
    let table: [(u8, u8, u8); 15] = serde_json::from_value(v.clone())
        .map_err(|e| format!("is not 15 [r, g, b] triples of 0-255 ({e})"))?;
    let base = crate::render::CART_BIOME_COLS;
    Ok(std::array::from_fn(|i| (table[i] != base[i]).then_some(table[i])))
}

/// `entities/landmarks.json` -- the Landmark Generation dock's **settings and
/// its last run's placements**.
///
/// ## The results half (owner ruling 10, 2026-09-06)
///
/// `LARGE_ITEM_RULINGS.md`: *"Landmark persistence? → PERSIST in
/// `entities/landmarks.json`. Consistent with the recorded finding that
/// research §25's state transitions cannot be recomputed."*
///
/// **This doc comment used to argue the opposite**, and the argument was
/// sound on its own terms and is superseded rather than wrong: `generate` is
/// a pure function of the world, the settings and the seed, so re-running the
/// pass reproduces a placement exactly. What it does not reproduce is
/// anything a person or a later pass then *attaches* to that placement -- §25
/// state, a name, a knowledge link -- and that is what a save has to carry.
/// The ruling is the decision; this is the wiring.
///
/// A landmark is addressed by `cartalith_civ::landmark::Landmark::key()` --
/// `"<kind>@<x>,<y>"`, composed from three members [`LandmarkDto`] carries.
/// Not by `Landmark::id`, which is a position in the result vector and moves
/// whenever a cap does.
///
/// ## The settings half, unchanged
///
/// Hand-entered configuration -- forty-nine caps, forty-nine armed flags, a
/// crowding factor and four class radii -- that no amount of recomputation
/// brings back. Before this document existed they were **not saved at all**
/// and were **never cleared**, so they leaked out of whichever project was
/// open last into the next one, belonging to neither. Writing them here and
/// resetting to `LandmarkSettings::default()` in `load_save`'s own reset
/// closes both halves at once.
///
/// ## Absent is absent
///
/// Every field is `#[serde(default)]`, and `LandmarkSettings`' own
/// `cap`/`is_armed` accessors already fall back to each kind's spec default
/// for a key the map has no row for -- so a document written before a new
/// landmark kind existed loads without inventing a value for it.
///
/// [`results`](Self::results) is `Option` and `skip_serializing_if`: a
/// project whose landmark pass has never run writes **no `results` member at
/// all**, rather than an empty run that would read as "it ran and placed
/// nothing". A document written by the previous writer carries no `results`
/// member either -- that writer had none to carry -- and loads as `None`,
/// which is what `LandmarkStore::last` already means and what the panel
/// already draws as `—`.
/// `a_document_written_before_results_existed_keeps_its_settings` builds
/// that document and asserts it.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
struct LandmarksDoc {
    settings: LandmarkSettingsDto,
    #[serde(skip_serializing_if = "Option::is_none")]
    results: Option<LandmarkRunDto>,
    /// Ruling AT's refined viewshed, as a loose JSON value so a malformed
    /// member costs only itself (see [`RefinedViewshedDto`]). **Absent means
    /// never refined**: `skip_serializing_if` writes nothing for `None`, so a
    /// project that was never refined, and every document written before this
    /// member existed, re-serialises byte-identically.
    #[serde(skip_serializing_if = "Option::is_none")]
    viewshed_refined: Option<serde_json::Value>,
}

/// The stored refined viewshed (Ruling AT) as `entities/landmarks.json`'s
/// optional `viewshed_refined` member carries it. `SAVEFILE_COMPAT.md` §9.8.
///
/// * **Keys are 16-digit lowercase hex strings**, not numbers: a 64-bit hash
///   written as a JSON number would pass `Number.MAX_SAFE_INTEGER` (2^53) and
///   be silently rounded by any JavaScript reader, which is the bug
///   `the_landmark_document_writes_no_integer_past_2_53` exists to forbid.
/// * **`cells` is run-length encoded** as `[run, value]` pairs, row-major over
///   the rectangle. The field is mostly zero (most cells see no observer) and
///   a 2048x1024 view would otherwise write two million numbers; a run is
///   consecutive cells with bit-identical `f32` values. The sum of the runs
///   must equal `w * h`.
/// * **Values are the raw accumulated observer weights**, lossless `f32`
///   (serde_json writes the shortest text that parses back to the same
///   `f32`), so a saved result re-reads bit-identically.
/// * **No member of this DTO is defaulted**: it is decoded from a loose
///   `serde_json::Value` ([`decode_viewshed_refined`]) so that a malformed
///   member costs only itself, never the settings and placements beside it.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RefinedViewshedDto {
    x: usize,
    y: usize,
    w: usize,
    h: usize,
    radius_cells: i64,
    observers: usize,
    terrain_key: String,
    observer_key: String,
    cells: Vec<(u32, f32)>,
}

/// Encode a stored refined viewshed as the document member's JSON value.
///
/// Never fails: every field is a plain number or string. The RLE compares
/// `f32::to_bits` so `0.0` and `-0.0` stay distinct and a restored field is
/// bit-identical.
fn encode_viewshed_refined(rv: &cartalith_civ::landmark::RefinedViewshed) -> serde_json::Value {
    let mut cells: Vec<(u32, f32)> = Vec::new();
    for &v in &rv.vis {
        match cells.last_mut() {
            Some((run, last)) if last.to_bits() == v.to_bits() && *run < u32::MAX => *run += 1,
            _ => cells.push((1, v)),
        }
    }
    let (terrain_key, observer_key) = rv.keys_hex();
    serde_json::to_value(RefinedViewshedDto {
        x: rv.rect.x,
        y: rv.rect.y,
        w: rv.rect.w,
        h: rv.rect.h,
        radius_cells: rv.radius_cells,
        observers: rv.observers,
        terrain_key,
        observer_key,
        cells,
    })
    .expect("a struct of numbers and strings serialises")
}

/// Decode the document member back into a [`RefinedViewshed`], or say why
/// it was refused (`Err`, for the loader's warning). Trusts nothing: wrong
/// types, a rectangle that does not lie inside this archive's `gw x gh` grid,
/// a malformed key, a zero run, runs that do not sum to `w * h`, or a
/// non-finite or negative weight all drop the member and the project opens as
/// "never refined". A *stale* result is not an error here — staleness is
/// judged later against the live world by content key.
fn decode_viewshed_refined(
    v: &serde_json::Value,
    gw: usize,
    gh: usize,
) -> Result<cartalith_civ::landmark::RefinedViewshed, String> {
    use cartalith_civ::landmark::{RefinedViewshed, ViewRect, REFINE_MAX_CELLS};
    let dto: RefinedViewshedDto = serde_json::from_value(v.clone()).map_err(|e| e.to_string())?;
    let rect = ViewRect { x: dto.x, y: dto.y, w: dto.w, h: dto.h };
    if !rect.fits(gw, gh) {
        return Err(format!("rectangle {}x{} at ({},{}) is not inside the {gw}x{gh} grid", dto.w, dto.h, dto.x, dto.y));
    }
    if rect.area() > REFINE_MAX_CELLS {
        return Err("rectangle larger than the refine cap".to_string());
    }
    let terrain_key = RefinedViewshed::parse_key(&dto.terrain_key).ok_or("malformed terrain_key")?;
    let observer_key = RefinedViewshed::parse_key(&dto.observer_key).ok_or("malformed observer_key")?;
    let mut vis: Vec<f32> = Vec::with_capacity(rect.area());
    for (run, value) in &dto.cells {
        if *run == 0 {
            return Err("a zero-length run".to_string());
        }
        // Bound before extending so a hostile run cannot allocate gigabytes.
        if vis.len() + *run as usize > rect.area() {
            return Err("runs sum to more than w * h".to_string());
        }
        vis.resize(vis.len() + *run as usize, *value);
    }
    RefinedViewshed::from_parts(rect, dto.radius_cells, dto.observers, terrain_key, observer_key, vis)
        .ok_or_else(|| "cells disagree with the rectangle, or hold a non-finite or negative weight".to_string())
}

/// One `landmark_run()`'s output, as the archive carries it.
///
/// **`rejects` is not here**, and the omission is the same one
/// `cartalith_civ::landmark::all_skipped` already makes for the same reason:
/// an empty reject list is *"this result carries no reject rows"*, not
/// *"nothing was rejected"*, and the funnels below keep the true totals in
/// `rejected_constraint`/`rejected_score`/`rejected_spacing`/`rejected_cap`
/// either way. What decided it is size against purpose: the list is a tuning
/// overlay bounded at `REJECT_LIST_MAX_PER_KIND` (256) **per kind**, and one
/// real world -- 256 x 192, terrain seed 24601, default settings, measured
/// 2026-09-06 -- carries **2 902 reject rows against 250 placements**, an
/// order of magnitude more JSON than the entities the slot is named for, for
/// a diagnostic of a run the reader is not in. One world is one sample; the
/// ratio is what the decision rests on, not the figure.
/// `landmark_rejects()` therefore returns nothing for a loaded project until
/// the pass is run again, which is the honest state and not a claim.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
struct LandmarkRunDto {
    landmarks: Vec<LandmarkDto>,
    /// One per row of `cartalith_civ::landmark::kinds()`, which is what
    /// `LandmarkResult::funnels`' own contract requires. A row this build has
    /// no entry for is rebuilt as `LandmarkLimit::Unrecorded` -- see
    /// [`LandmarkRunDto::into_result`].
    funnels: Vec<LandmarkFunnelDto>,
    seconds: f64,
}

/// One placed landmark.
///
/// **Three members are deliberately absent, and each for its own reason.**
///
/// * `id` -- a position in the result vector, re-derived on load exactly as
///   `generate` derives it (`k + 1` over the restored order). Writing it
///   would put an unstable number in the format and give the list two orders
///   that could disagree.
/// * `class` -- a property of the *kind*, not of the placement
///   (`LandmarkKindSpec::class`). Writing it per row would be writing one
///   compile-time table two hundred and fifty times; `kind_spec()` resolves
///   it, and a `kind` that does not resolve costs the row rather than being
///   guessed into a class, which is the rule `IconDto` above already applies
///   to an unknown icon family.
/// * `seed` -- a 64-bit hash, which `SAVEFILE_COMPAT.md` §14.1 permits in
///   neither form: not as a number (above 2^53) and not as a string ("there
///   are no string-encoded integers anywhere in this format and none should
///   be added"). It is recomputed by
///   `cartalith_civ::landmark::seed_for(world_seed, gw, kind, x, y)`, whose
///   own doc records the one thing that does not survive -- a
///   `GENERATOR_VERSION` bump.
///
/// `elevation_m` names its unit, the way `length_km` does in `WaysDoc`: the
/// engine's field is `Landmark::elevation` and is documented as metres above
/// sea level, and the format's reader should not have to find that out.
///
/// **The three floats come back bit for bit.** Until 2026-09-24 they could
/// move by one ULP on the first save-open cycle -- `serde_json` was built
/// without `float_roundtrip`, and 34 of 128 placements moved on a real world
/// -- and `cartalith-io` now takes the feature; `an_open_save_cycle_reaches_
/// a_fixed_point` asserts that nothing moves. Nothing here rounds or
/// truncates on purpose.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct LandmarkDto {
    #[serde(default)]
    kind: String,
    #[serde(default)]
    x: usize,
    #[serde(default)]
    y: usize,
    #[serde(default)]
    elevation_m: f64,
    #[serde(default)]
    score: f64,
    #[serde(default)]
    importance: f64,
    #[serde(default)]
    causal: Vec<String>,
}

/// One kind's funnel -- §5's "why fewer than I asked for" counters.
///
/// Carried in full rather than recomputed, because they cannot be: the
/// candidate counts and the four reject buckets are products of a walk over a
/// world, and `LandmarkFunnel`'s own documented identity
/// (`candidates == rejected_constraint + rejected_score + rejected_spacing +
/// rejected_cap + placed`) has to keep closing after a round trip. It is
/// asserted after one in `a_landmark_run_survives_a_document_round_trip`.
///
/// `limit` is `LandmarkLimit::as_str()`'s machine token, read back through
/// `LandmarkLimit::from_key`. A token this build does not know becomes
/// `Unrecorded`, never a guess -- that variant exists for exactly this.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct LandmarkFunnelDto {
    #[serde(default)]
    kind: String,
    #[serde(default)]
    candidates: usize,
    #[serde(default)]
    rejected_constraint: usize,
    #[serde(default)]
    rejected_score: usize,
    #[serde(default)]
    rejected_spacing: usize,
    #[serde(default)]
    rejected_cap: usize,
    #[serde(default)]
    cap: usize,
    #[serde(default)]
    placed: usize,
    #[serde(default)]
    limit: String,
}

/// The settings half of `entities/landmarks.json`: per-kind caps and armed flags, the crowding factor, class radii in km (in `LandmarkClass::all()` order) and the cross-type switch. [`Self::into_settings`] re-validates all of it on read.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
struct LandmarkSettingsDto {
    caps: BTreeMap<String, u32>,
    armed: BTreeMap<String, bool>,
    crowding: f64,
    class_radius_km: Vec<f64>,
    cross_type_competition: bool,
}

impl Default for LandmarkSettingsDto {
    /// Absent `settings` member opens on the engine's own defaults, not on zeros or empty maps.
    fn default() -> Self {
        Self::from(&cartalith_civ::landmark::LandmarkSettings::default())
    }
}

impl From<&cartalith_civ::landmark::LandmarkSettings> for LandmarkSettingsDto {
    /// Copies the live settings into their document form; infallible.
    fn from(s: &cartalith_civ::landmark::LandmarkSettings) -> Self {
        Self {
            caps: s.caps.clone(),
            armed: s.armed.clone(),
            crowding: s.crowding,
            class_radius_km: s.class_radius_km.to_vec(),
            cross_type_competition: s.cross_type_competition,
        }
    }
}

impl LandmarkSettingsDto {
    /// Back to the engine type, **through the bridge's own setters** rather
    /// than by assigning the fields: `landmark_bridge::set_cap`/`set_armed`
    /// reject a key `cartalith_civ::landmark::kinds()` does not carry, which
    /// is the same defence they give the `#[func]` surface and is needed
    /// here for the same reason -- an archive is untrusted input, and a
    /// hand-edited `caps` map must not be able to introduce a fiftieth
    /// landmark type. `crowding` and the class radii go through
    /// `set_crowding`/`set_class_radius`, so a document carrying a
    /// nonsensical number is clamped to the documented range rather than
    /// stored.
    ///
    /// A `class_radius_km` array of the wrong length is not an error: each
    /// entry is applied by index against the class list, and a missing one
    /// keeps the default. That is §14.3's unknown-member rule pointed at an
    /// array.
    fn into_settings(self) -> cartalith_civ::landmark::LandmarkSettings {
        let mut out = cartalith_civ::landmark::LandmarkSettings::default();
        for (k, v) in &self.caps {
            crate::landmark_bridge::set_cap(&mut out, k, i64::from(*v));
        }
        for (k, v) in &self.armed {
            crate::landmark_bridge::set_armed(&mut out, k, *v);
        }
        crate::landmark_bridge::set_crowding(&mut out, self.crowding);
        for (i, class) in cartalith_civ::landmark::LandmarkClass::all().iter().enumerate() {
            if let Some(km) = self.class_radius_km.get(i) {
                crate::landmark_bridge::set_class_radius(&mut out, class.as_str(), *km);
            }
        }
        crate::landmark_bridge::set_cross_competition(&mut out, self.cross_type_competition);
        out
    }
}

impl From<&cartalith_civ::landmark::LandmarkResult> for LandmarkRunDto {
    /// A run's document form. Rows with a non-finite elevation, score or importance are dropped (inline note), `rejects` is not carried (see [`LandmarkRunDto`]), and a non-finite `seconds` is written as `0.0`.
    fn from(r: &cartalith_civ::landmark::LandmarkResult) -> Self {
        Self {
            landmarks: r
                .landmarks
                .iter()
                // `SAVEFILE_COMPAT.md` §14.1: `NaN` and the infinities are
                // not valid numbers in this format. Dropping the row is
                // §6.4a's own escalation -- "the array element it sits in"
                // -- and it is also self-defence: `serde_json` *refuses* to
                // serialize a non-finite float, and `insert_doc` swallows
                // that into "the slot is absent", so one poisoned elevation
                // would silently cost the whole document, settings and all.
                .filter(|l| l.elevation.is_finite() && l.score.is_finite() && l.importance.is_finite())
                .map(|l| LandmarkDto {
                    kind: l.kind.clone(),
                    x: l.x,
                    y: l.y,
                    elevation_m: l.elevation,
                    score: l.score,
                    importance: l.importance,
                    causal: l.causal.clone(),
                })
                .collect(),
            funnels: r
                .funnels
                .iter()
                .map(|f| LandmarkFunnelDto {
                    kind: f.kind.clone(),
                    candidates: f.candidates,
                    rejected_constraint: f.rejected_constraint,
                    rejected_score: f.rejected_score,
                    rejected_spacing: f.rejected_spacing,
                    rejected_cap: f.rejected_cap,
                    cap: f.cap,
                    placed: f.placed,
                    limit: f.limit.as_str().to_string(),
                })
                .collect(),
            seconds: if r.seconds.is_finite() { r.seconds } else { 0.0 },
        }
    }
}

impl LandmarkRunDto {
    /// Back to the engine type. `world_seed`, `gw` and `gh` come from the
    /// archive that carried this document, so every derived value belongs to
    /// the world it is being restored onto.
    ///
    /// Three things a row can fail, and all three cost the row rather than
    /// the document (`SAVEFILE_COMPAT.md` §6.4a):
    ///
    /// * a `kind` that is not in `cartalith_civ::landmark::kinds()` -- there
    ///   is no class, no spec and no seed for it;
    /// * a cell outside `gw x gh` -- an archive is untrusted input, and a
    ///   landmark at `(9000, 3)` on a 512-wide grid is not a landmark;
    /// * a non-finite number, which §14.1 does not allow in the format.
    ///
    /// **Each drop is reported** — `project_open` subtracts the surviving
    /// count from the document's own and raises one `warnings` line for the
    /// document, which is §6.4a rung 2's *"skip that element, keep the rest
    /// of the array, and report the skip"*. This comment used to say the
    /// drops were "counted by the caller rather than announced", and no
    /// caller counted them: `landmarks()` returns what survived and nothing
    /// anywhere held the number it was fewer than.
    ///
    /// `id` is re-derived here, `k + 1` over the surviving order -- the same
    /// two lines `generate` runs at the end of its own pass. It is a
    /// position, and this is the position.
    fn into_result(
        self,
        world_seed: u64,
        gw: usize,
        gh: usize,
    ) -> cartalith_civ::landmark::LandmarkResult {
        use cartalith_civ::landmark::{
            kind_spec, kinds, seed_for, Landmark, LandmarkFunnel, LandmarkLimit, LandmarkResult,
        };

        let mut landmarks: Vec<Landmark> = self
            .landmarks
            .into_iter()
            .filter_map(|d| {
                let spec = kind_spec(&d.kind)?;
                if d.x >= gw || d.y >= gh {
                    return None;
                }
                if !(d.elevation_m.is_finite() && d.score.is_finite() && d.importance.is_finite()) {
                    return None;
                }
                Some(Landmark {
                    id: 0,
                    seed: seed_for(world_seed, gw, &d.kind, d.x, d.y)?,
                    kind: d.kind,
                    class: spec.class,
                    x: d.x,
                    y: d.y,
                    elevation: d.elevation_m,
                    score: d.score,
                    importance: d.importance,
                    causal: d.causal,
                })
            })
            .collect();
        for (k, lm) in landmarks.iter_mut().enumerate() {
            lm.id = k as u64 + 1;
        }

        // `LandmarkResult::funnels` is "one entry per row of `kinds()`, **in
        // table order**", which is a contract the panel walks its own rows
        // against -- so the list is rebuilt from `kinds()` and filled from
        // the document, never taken from the document's order.
        let funnels = kinds()
            .iter()
            .map(|k| {
                match self.funnels.iter().find(|f| f.kind == k.key) {
                    Some(f) => LandmarkFunnel {
                        kind: k.key.to_string(),
                        candidates: f.candidates,
                        rejected_constraint: f.rejected_constraint,
                        rejected_score: f.rejected_score,
                        rejected_spacing: f.rejected_spacing,
                        rejected_cap: f.rejected_cap,
                        cap: f.cap,
                        placed: f.placed,
                        // An unknown token is not a reason to invent one.
                        limit: LandmarkLimit::from_key(&f.limit)
                            .unwrap_or(LandmarkLimit::Unrecorded),
                    },
                    // A kind this build has and the writing build did not.
                    // Every counter is genuinely unknown, so every counter is
                    // zero -- which keeps the funnel's own identity closed --
                    // and `Unrecorded` says why instead of picking one of the
                    // six measured reasons.
                    None => LandmarkFunnel {
                        kind: k.key.to_string(),
                        candidates: 0,
                        rejected_constraint: 0,
                        rejected_score: 0,
                        rejected_spacing: 0,
                        rejected_cap: 0,
                        cap: 0,
                        placed: 0,
                        limit: LandmarkLimit::Unrecorded,
                    },
                }
            })
            .collect();

        LandmarkResult {
            landmarks,
            funnels,
            // Not carried; `LandmarkRunDto`'s own doc comment says why.
            rejects: Vec::new(),
            seconds: self.seconds,
        }
    }
}

/// `annotations/regions.json`: the selected region, `None` when nothing is selected.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct RegionsDoc {
    #[serde(default)]
    region: Option<RegionDto>,
}

/// A selected region as a grid rectangle: top-left `x`, `y` and size `w`, `h`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct RegionDto {
    #[serde(default)]
    x: usize,
    #[serde(default)]
    y: usize,
    #[serde(default)]
    w: usize,
    #[serde(default)]
    h: usize,
}

// ===================== vocabulary =====================

/// `WayType` to §9.3's `class` spelling; the inverse is [`way_class_from`].
fn way_class_key(t: cartalith_civ::WayType) -> &'static str {
    match t {
        cartalith_civ::WayType::Highway => "highway",
        cartalith_civ::WayType::Regional => "regional",
        cartalith_civ::WayType::Road => "road",
        cartalith_civ::WayType::Track => "track",
        cartalith_civ::WayType::Ancient => "ancient",
    }
}

/// `SAVEFILE_COMPAT.md` §9.3: an unrecognised class reads as `track`
/// rather than dropping the road. A road drawn by a future version with a
/// class this build has never heard of is still a road.
fn way_class_from(key: &str) -> cartalith_civ::WayType {
    match key {
        "highway" => cartalith_civ::WayType::Highway,
        "regional" => cartalith_civ::WayType::Regional,
        "road" => cartalith_civ::WayType::Road,
        // A village connector. Without this arm a save would reload it as a
        // `track` -- drawn from track's zoom threshold while its village stays
        // hidden until `VILLAGE_ADDON_LOD`: the reference's own v1.72 BUG-A.
        "ancient" => cartalith_civ::WayType::Ancient,
        _ => cartalith_civ::WayType::Track,
    }
}

/// `ManualWayType` to its archive spelling; the inverse is [`manual_way_kind_from`].
fn manual_way_kind_key(t: cartalith_civ::tools::ManualWayType) -> &'static str {
    use cartalith_civ::tools::ManualWayType as M;
    match t {
        M::Road => "road",
        M::Track => "track",
        M::SeaLane => "sea_lane",
        M::Ancient => "ancient",
    }
}

/// Archive spelling to `ManualWayType`; an unrecognised key reads as `Track` rather than dropping the way.
fn manual_way_kind_from(key: &str) -> cartalith_civ::tools::ManualWayType {
    use cartalith_civ::tools::ManualWayType as M;
    match key {
        "road" => M::Road,
        "sea_lane" => M::SeaLane,
        "ancient" => M::Ancient,
        _ => M::Track,
    }
}

/// `RouteMode` to its archive spelling (`land`/`water`/`mixed`); the inverse is [`route_mode_from`]. `pub(crate)` because `lib.rs` also spells a journey's mode with it.
pub(crate) fn route_mode_key(m: cartalith_civ::tools::RouteMode) -> &'static str {
    use cartalith_civ::tools::RouteMode as R;
    match m {
        R::Land => "land",
        R::Water => "water",
        R::Mixed => "mixed",
    }
}

/// Archive spelling to `RouteMode`; an unrecognised key reads as `Land`.
fn route_mode_from(key: &str) -> cartalith_civ::tools::RouteMode {
    use cartalith_civ::tools::RouteMode as R;
    match key {
        "water" => R::Water,
        "mixed" => R::Mixed,
        _ => R::Land,
    }
}

/// `LabelSizeMode` to its archive spelling; the inverse is [`label_size_mode_from`].
fn label_size_mode_key(m: cartalith_civ::labels::LabelSizeMode) -> &'static str {
    match m {
        cartalith_civ::labels::LabelSizeMode::Zoom => "zoom",
        cartalith_civ::labels::LabelSizeMode::Fixed => "fixed",
    }
}

/// Archive spelling to `LabelSizeMode`; an unrecognised key reads as `Zoom`.
fn label_size_mode_from(key: &str) -> cartalith_civ::labels::LabelSizeMode {
    match key {
        "fixed" => cartalith_civ::labels::LabelSizeMode::Fixed,
        _ => cartalith_civ::labels::LabelSizeMode::Zoom,
    }
}

/// A resource key back to its `'static` spelling.
///
/// `TradeBalance` holds `&'static str`, so a string read from a file cannot
/// go straight in: it has to be matched against the one table of real keys.
/// An unrecognised key is dropped rather than leaked -- §9.1's own rule,
/// and the only alternative would be leaking a `String` for the process
/// lifetime to satisfy a lifetime this port did not choose.
fn resource_key_static(key: &str) -> Option<&'static str> {
    cartalith_civ::CIV_RESOURCE_KEYS
        .iter()
        .copied()
        .find(|&k| k == key)
}

/// Polyline tuples to the archive's `[x, y]` pairs; the inverse is [`dto_to_pts`].
fn pts_to_dto(pts: &[(f64, f64)]) -> Vec<[f64; 2]> {
    pts.iter().map(|&(x, y)| [x, y]).collect()
}

/// The archive's `[x, y]` pairs back to polyline tuples; the inverse of [`pts_to_dto`].
fn dto_to_pts(pts: &[[f64; 2]]) -> Vec<(f64, f64)> {
    pts.iter().map(|p| (p[0], p[1])).collect()
}

/// `SAVEFILE_COMPAT.md` §9.3: every break must index into `points`.
/// An out-of-range one is dropped rather than kept, because a renderer that
/// trusts it walks off the end of the polyline.
fn sane_breaks(brks: &[usize], n: usize) -> Vec<usize> {
    brks.iter().copied().filter(|&b| b < n).collect()
}

// ===================== CivData <-> documents =====================

/// A settlement plus its optional trade balance, place extras and village flag as a [`SettlementDto`]. Extras equal to their default are omitted, so an unedited place writes no `extras` member.
fn settlement_to_dto(
    s: &cartalith_civ::NamedSettlement,
    trade: Option<&cartalith_civ::TradeBalance>,
    extras: Option<&civ_roster_bridge::PlaceExtras>,
    village: bool,
) -> SettlementDto {
    SettlementDto {
        id: s.tid,
        x: s.placement.x,
        y: s.placement.y,
        name: s.name.clone(),
        population: s.pop,
        faction: s.placement.faction,
        kind: journey_bridge::settlement_kind_key(s.placement.kind).to_string(),
        capital: s.placement.capital,
        coastal: s.placement.coastal,
        suitability: s.placement.suit,
        village_seeded: village,
        trade: trade.map(|t| TradeDto {
            exports: t.exports.iter().map(|s| (*s).to_string()).collect(),
            imports: t.imports.iter().map(|s| (*s).to_string()).collect(),
        }),
        extras: extras
            .filter(|e| **e != civ_roster_bridge::PlaceExtras::default())
            .map(|e| ExtrasDto {
                specialisation: e.specialisation.clone(),
                traits: e.traits.clone(),
                history: e.history.clone(),
                age: e.age,
                walls: e.walls,
                culture: e.culture.clone(),
                rules_preset: e.rules_preset.clone(),
                variant: e.variant,
            }),
    }
}

/// A [`SettlementDto`]'s settlement half; trade, extras and the village flag are restored by `civ_from_project`. An unrecognised `kind` reads as `Town` (§9.1).
fn dto_to_settlement(d: &SettlementDto) -> cartalith_civ::NamedSettlement {
    cartalith_civ::NamedSettlement {
        tid: d.id,
        placement: cartalith_civ::SettlementPlacement {
            x: d.x,
            y: d.y,
            suit: d.suitability,
            faction: d.faction,
            capital: d.capital,
            // §9.1: an unrecognised tier reads as `town` rather than
            // costing the settlement.
            kind: crate::civ_tools_bridge::kind_from_str(&d.kind)
                .unwrap_or(cartalith_civ::SettlementKind::Town),
            coastal: d.coastal,
        },
        name: d.name.clone(),
        pop: d.population,
    }
}

/// A live `Way` as a [`RoadDto`]; the inverse is [`dto_to_road`].
fn road_to_dto(w: &cartalith_civ::Way) -> RoadDto {
    RoadDto {
        id: w.tid,
        name: w.name.clone(),
        points: pts_to_dto(&w.pts),
        breaks: w.brks.clone(),
        length_km: w.km,
        class: way_class_key(w.way_type).to_string(),
        from: w.a_idx,
        to: w.b_idx,
        hidden: w.hidden,
    }
}

/// A [`RoadDto`] back to a `Way`: out-of-range breaks are dropped by `sane_breaks` and an unknown class reads as `track`.
fn dto_to_road(d: &RoadDto) -> cartalith_civ::Way {
    let pts = dto_to_pts(&d.points);
    let brks = sane_breaks(&d.breaks, pts.len());
    cartalith_civ::Way {
        tid: d.id,
        pts,
        brks,
        km: d.length_km,
        name: d.name.clone(),
        way_type: way_class_from(&d.class),
        a_idx: d.from,
        b_idx: d.to,
        hidden: d.hidden,
    }
}

/// A `Journey` as a [`JourneyDto`]; the inverse is [`dto_to_journey`]. The stored plan, if any, goes through [`journey_plan_to_dto`].
fn journey_to_dto(j: &cartalith_civ::travel_library::Journey) -> JourneyDto {
    JourneyDto {
        id: j.id,
        name: j.name.clone(),
        party_preset: j.party_preset.clone(),
        route: JourneyRouteDto {
            points: pts_to_dto(&j.route.points),
            breaks: j.route.breaks.clone(),
            length_km: j.route.length_km,
            mode: route_mode_key(j.route.mode).to_string(),
        },
        start_year: j.start_year,
        resupply_stops: j
            .resupply_stops
            .iter()
            .map(|a| AcceptedStopDto { key: a.key.clone(), kind: a.kind.clone(), name: a.name.clone(), at: [a.x, a.y] })
            .collect(),
        plan: j.plan.as_ref().map(journey_plan_to_dto),
    }
}

/// [`JourneyPlanDto`] from a saved plan. The inverse of
/// [`dto_to_journey_plan`]; must write every member of
/// `cartalith_civ::travel_library::JourneyPlan` (a test pins that field by
/// field) and never writes `accepted_resupply`, which lives on the journey.
fn journey_plan_to_dto(p: &cartalith_civ::travel_library::JourneyPlan) -> JourneyPlanDto {
    JourneyPlanDto {
        fields: travel_pairs_to_fields(journey_bridge::plan_to_pairs(&p.plan).into_iter().map(|(k, v)| (k.to_string(), v)).collect()),
        stage_overrides: p
            .plan
            .stage_overrides
            .iter()
            .map(|(idx, ov)| {
                let pairs = journey_bridge::stage_override_to_pairs(ov).into_iter().map(|(k, v)| (k.to_string(), v)).collect();
                (idx.to_string(), travel_pairs_to_fields(pairs))
            })
            .collect(),
        layovers: p.layovers.clone(),
        animal_entries: p.animal_entries.clone(),
        trim: [p.trim.0, p.trim.1],
        auto_carriage: p.auto_carriage,
        auto_stage: p.auto_stage,
    }
}

/// A saved plan from [`JourneyPlanDto`], tolerant per its doc: unknown or
/// wrong-typed keys are dropped (the planner's `rejected` policy), never
/// fatal. `accepted_resupply` is left empty by construction.
fn dto_to_journey_plan(d: &JourneyPlanDto) -> cartalith_civ::travel_library::JourneyPlan {
    let (mut plan, _rejected) = journey_bridge::plan_from_pairs(&travel_fields_to_pairs(&d.fields));
    for (idx, fields) in &d.stage_overrides {
        let Ok(idx) = idx.parse::<usize>() else { continue };
        let (ov, _rejected) = journey_bridge::stage_override_from_pairs(&travel_fields_to_pairs(fields));
        plan.stage_overrides.insert(idx, ov);
    }
    cartalith_civ::travel_library::JourneyPlan {
        plan,
        layovers: d.layovers.clone(),
        animal_entries: d.animal_entries.clone(),
        trim: (d.trim[0], d.trim[1]),
        auto_carriage: d.auto_carriage,
        auto_stage: d.auto_stage,
    }
}

/// A [`JourneyDto`] back to a `Journey`. Route breaks are range-checked by `sane_breaks`; an unknown mode reads as `Land`; a missing `plan` stays `None` (planned from its preset).
fn dto_to_journey(d: &JourneyDto) -> cartalith_civ::travel_library::Journey {
    let pts = dto_to_pts(&d.route.points);
    let brks = sane_breaks(&d.route.breaks, pts.len());
    cartalith_civ::travel_library::Journey {
        id: d.id,
        name: d.name.clone(),
        party_preset: d.party_preset.clone(),
        route: cartalith_civ::travel_library::JourneyRoute {
            points: pts,
            breaks: brks,
            length_km: d.route.length_km,
            mode: route_mode_from(&d.route.mode),
        },
        start_year: d.start_year,
        resupply_stops: d
            .resupply_stops
            .iter()
            .map(|a| cartalith_civ::JpAcceptedStop {
                key: a.key.clone(),
                kind: a.kind.clone(),
                name: a.name.clone(),
                x: a.at[0],
                y: a.at[1],
            })
            .collect(),
        plan: d.plan.as_ref().map(dto_to_journey_plan),
    }
}

/// Everything the archive holds about the civilisation layer, as JSON text
/// keyed by slot, plus the civ rasters. Pure — no Godot, no filesystem —
/// so the round-trip tests below run under `cargo test -p cartalith-godot`
/// with no Godot runtime involved.
///
/// `name_stream` is `CivTools::name_rng`'s position, `None` when there is no
/// tool state to record it from (the member is then omitted, §9.1).
///
/// Takes no `garrison_exposure_scale` any more -- the fold-in moved that
/// setting onto `cartalith_engine::CivParams`, which `params::save_state`
/// writes separately (`project_open`'s own call site), so this function's
/// `FactionsDoc` never carries a live value for it, only ever reads one back
/// (`FactionsDoc::garrison_exposure_scale`'s own doc comment).
fn civ_documents(civ: &CivData, name_stream: Option<u32>, out: &mut BTreeMap<String, String>) {
    let extras = &civ.place_extras.0;
    let settlements = SettlementsDoc {
        next_id: civ.next_tid,
        name_stream,
        settlements: civ
            .settlements
            .iter()
            .enumerate()
            .map(|(i, s)| {
                settlement_to_dto(
                    s,
                    civ.trade_balances.get(i),
                    extras.get(&s.tid),
                    civ.village_tids.contains(&s.tid),
                )
            })
            .collect(),
    };
    insert_doc(out, SLOT_SETTLEMENTS, &settlements);

    let factions = FactionsDoc {
        factions: civ
            .faction_roster
            .0
            .iter()
            .enumerate()
            .map(|(i, f)| FactionDto {
                id: i,
                name: f.name.clone(),
                culture: f.culture.clone(),
                religion: f.religion.clone(),
                government: f.government.clone(),
                ag_tech: f.ag_tech.clone(),
                color: [f.color.0, f.color.1, f.color.2],
                user_color: f.color_override.map(|c| [c.0, c.1, c.2]),
                tariffs: f.tariffs.clone(),
                currency: (f.currency != civ_roster_bridge::FactionCurrency::default()).then(|| {
                    CurrencyDto {
                        name: f.currency.name.clone(),
                        symbol: f.currency.symbol.clone(),
                        rate: f.currency.rate,
                    }
                }),
                religious_policy: (f.religious_policy != "plural")
                    .then(|| serde_json::Value::String(f.religious_policy.clone())),
            })
            .collect(),
        // Never written (`#[serde(skip_serializing)]` above) -- kept `None`
        // so a document built here and then round-tripped through
        // `FactionsDoc`'s own `Deserialize` (as some tests do) does not
        // manufacture a legacy value nothing wrote.
        garrison_exposure_scale: None,
    };
    insert_doc(out, SLOT_FACTIONS, &factions);
    // R1: beside the roster whose `custom:<id>` keys index it. Absent until a
    // religion has ever been created, so an untouched project's archive does
    // not change.
    if let Some(doc) = religions_doc(&civ.religions) {
        insert_doc(out, SLOT_RELIGIONS, &doc);
    }

    let ways = WaysDoc {
        roads: civ.ways.iter().map(road_to_dto).collect(),
        sea_lanes: civ
            .sea_routes
            .iter()
            .map(|r| SeaLaneDto {
                name: r.name.clone(),
                points: pts_to_dto(&r.pts),
                breaks: r.brks.clone(),
                length_km: r.km,
            })
            .collect(),
        // Hand-drawn ways and routes live on `InfraTools`, not `CivData`;
        // they are merged in by the caller so that this one document stays
        // the single home for every linear route (§9.3).
        manual: Vec::new(),
        routes: Vec::new(),
        road_edges: civ
            .road_edges
            .iter()
            .map(|e| RoadEdgeDto { from: e.a, to: e.b, cells: e.path.clone() })
            .collect(),
        // The grid the paths above are on: every `RoadEdge` in memory was
        // routed (or re-gridded on open) at today's cap.
        road_edges_rw_cap: (!civ.road_edges.is_empty()).then_some(cartalith_civ::RW_CAP),
    };
    insert_doc(out, SLOT_WAYS, &ways);

    if !civ.province_list.is_empty() {
        let provinces = ProvincesDoc {
            provinces: civ
                .province_list
                .iter()
                .map(|p| ProvinceDto {
                    id: p.id,
                    faction: p.faction,
                    name: p.name.clone(),
                    capital_settlement_index: p.capital_settlement_index,
                })
                .collect(),
        };
        insert_doc(out, SLOT_PROVINCES, &provinces);
    }

    if !civ.continents.is_empty() {
        let continents = ContinentsDoc {
            continents: civ
                .continents
                .iter()
                .map(|c| ContinentDto {
                    id: c.id,
                    name: c.name.clone(),
                    cells: c.cells,
                    min_x: c.min_x,
                    min_y: c.min_y,
                    max_x: c.max_x,
                    max_y: c.max_y,
                    cx: c.cx,
                    cy: c.cy,
                    faction: c.faction,
                })
                .collect(),
        };
        insert_doc(out, SLOT_CONTINENTS, &continents);
    }

    if !civ.timeline.is_empty() || civ.year != 0 {
        let timeline = TimelineDoc {
            year: civ.year,
            years: civ
                .timeline
                .iter()
                .map(|snap| {
                    let (institutions, place_overrides) = year_record_to_dtos(&snap.record);
                    TimelineYearDto {
                        year: snap.year,
                        settlements: snap
                            .settlements
                            .iter()
                            .map(|s| settlement_to_dto(s, None, None, false))
                            .collect(),
                        ways: snap.ways.iter().map(road_to_dto).collect(),
                        institutions,
                        place_overrides,
                    }
                })
                .collect(),
            territory_year: civ.territory_year,
        };
        insert_doc(out, SLOT_TIMELINE, &timeline);
    }
}

/// Resolves the garrison rule's border-exposure scale for a project being
/// opened (Ruling AZ, 2026-09-28, `LARGE_ITEM_RULINGS.md`, plus the fold-in
/// that followed it onto `cartalith_engine::CivParams`). Pure -- no Godot, no
/// `WorldGen` -- so it is unit-tested directly the way `civ_documents` is,
/// rather than needing a probe (`WorldGen` is a `cdylib` `GodotClass` and
/// cannot be constructed in a unit test, `MISTAKES.md`).
///
/// `applied` is what `load_save`'s call to `params::apply_saved_state` already
/// left on `self.params.civ.garrison_exposure_scale` from *this same
/// archive*: the value at `civ.garrison_exposure_scale` in `params.json`'s
/// `cartalith` block when the archive carries that key at all, or
/// `cartalith_civ::garrison::EXPOSURE_SCALE` (`apply_saved_state`'s own
/// reset) when it does not.
///
/// **The params copy wins whenever both exist.** A save written from
/// 382945a (which first shipped this setting, only as `factions.json`'s
/// `garrison_exposure_scale` member) up to the fold-in that gave it a
/// `params.json` row could only ever carry the legacy member -- the two are
/// not the same file era in practice -- but a hand-edited or otherwise
/// unusual archive could carry both, and there is no reason for this one
/// setting to answer to a different authority than every other row in
/// `params::PARAMS` does. So the legacy member is read, and applied, only
/// when the new key is absent from the archive's `cartalith` block entirely
/// -- never merely because it happens to equal the default.
fn resolve_garrison_exposure_scale(data: &project::ProjectData, applied: f64) -> f64 {
    let has_native_key = data
        .save
        .state
        .get(params::NATIVE_PARAMS_KEY)
        .and_then(|c| c.get("civ.garrison_exposure_scale"))
        .is_some();
    if has_native_key {
        return applied;
    }
    data.parse::<FactionsDoc>(SLOT_FACTIONS)
        .and_then(|r| r.ok())
        .and_then(|d| d.garrison_exposure_scale)
        .filter(|v| v.is_finite())
        .map(|v| {
            v.clamp(
                civ_military_bridge::GARRISON_EXPOSURE_SCALE_MIN,
                civ_military_bridge::GARRISON_EXPOSURE_SCALE_MAX,
            )
        })
        .unwrap_or(applied)
}

/// `rasters/forced_lakes.u8` (`SAVEFILE_COMPAT.md` §8.1): Ruling BO's
/// forced-lake mask, `1` on a cell the user counted as lake, `0` elsewhere.
const RASTER_FORCED_LAKES: &str = "rasters/forced_lakes.u8";

/// Writes `WorldGen::forced_lakes` as [`RASTER_FORCED_LAKES`] -- only when a
/// lake was forced (`Some`, a whole grid). **Absent, not all-zero**, for a
/// world with none: an untouched project gains no member, which is what
/// keeps every existing archive and its round-trip tests byte-for-byte as
/// they were.
///
/// Why it is saved at all: until Ruling BO a forced lake survived a save only
/// inside `rasters/water_bodies.u8` (the civ copy), where it cannot be told
/// apart from a cell that copy had merely gone stale on after a sculpt -- so
/// a reopened project could not draw it. This is the one source of the mask
/// on reopen.
fn forced_lakes_raster(mask: Option<&[u8]>, n: usize, write: &mut ProjectWrite<'_>) {
    if let Some(m) = mask.filter(|m| m.len() == n && m.iter().any(|&c| c != 0)) {
        write.raster(RASTER_FORCED_LAKES, Raster::U8(m.to_vec()));
    }
}

/// [`RASTER_FORCED_LAKES`] read back: `Some` only for a `u8` grid of this
/// world's `n` cells with at least one forced cell. Absent -- every archive
/// written before 2026-09-28, and every world nothing was forced on -- is
/// `None` and says nothing. A present member of the wrong type or size is
/// dropped **with a warning** (§6.4a: every substitution is reported), since
/// forcing it onto this grid would flood the wrong cells.
fn forced_lakes_from_project(data: &project::ProjectData, n: usize, warnings: &mut Vec<String>) -> Option<Vec<u8>> {
    match data.raster(RASTER_FORCED_LAKES) {
        None => None,
        Some(Raster::U8(v)) if v.len() == n => v.iter().any(|&c| c != 0).then(|| v.clone()),
        Some(_) => {
            warnings.push(format!(
                "{RASTER_FORCED_LAKES}: not a u8 grid of this world's size -- the lakes painted and counted as                  water were not restored; press Count painted lakes as water again after a Lake stamp"
            ));
            None
        }
    }
}

/// The civ layer's rasters and recorded-year territories, into `write` --
/// `project_save_with_documents`' own block, a function so the substrate
/// round-trip test saves a civ layer exactly the way a save does.
fn civ_rasters(civ: &CivData, n: usize, write: &mut ProjectWrite<'_>) {
    if civ.territory.len() == n {
        write.raster("rasters/territory.i32", Raster::I32(civ.territory.clone()));
    }
    if civ.provinces.len() == n {
        write.raster("rasters/provinces.i32", Raster::I32(civ.provinces.clone()));
    }
    if civ.water_bodies.len() == n {
        write.raster(
            "rasters/water_bodies.u8",
            Raster::U8(civ.water_bodies.clone()),
        );
    }
    if civ.dens.len() == n {
        write.raster(
            "rasters/agrarian_density.f32",
            Raster::F32(civ.dens.clone()),
        );
    }
    // Reconstructed, not copied out: owner ruling 27 made
    // `TimelineSnapshot::territory` a delta chain in memory. **The
    // archive shape did not move** -- §10.2's
    // `history/territory/<year>.i32` still gets one whole raster per
    // recorded year, so no `format_version` bump and no fail-loud
    // marker is owed here. The on-disk half of ruling 27 is a
    // separate change; see this batch's report.
    for snap in &civ.timeline {
        if let Some(raster) =
            cartalith_civ::timeline::civ_territory_at(&civ.timeline, snap.year)
        {
            if raster.len() == n {
                write.history_territory.insert(snap.year, raster);
            }
        }
    }
}

/// Serialises `value` as pretty JSON into `out[slot]`. A serialisation failure leaves the slot absent rather than panicking (see the inline note); callers must not read absence as "empty document".
/// CP-3's saved form of the colour-paint layer: `None` with no layer or no
/// marks (the slot is then absent, so an untouched project's archive does
/// not change), else the document with every coordinate, radius and edge
/// rounded to the metre (`COLOUR_PAINT_SCOPE.md` §9: 3 decimals of km). The
/// rounding is idempotent, so re-saving an opened project writes the same
/// bytes. A hidden layer with marks is saved (with `visible: false`).
///
/// Ruling CS's paint layers: the payload is the layered document
/// (`PaintLayers`), every layer -- hidden and locked ones too, a hidden
/// layer is saved, not dropped. It serialises as version 1 while it is one
/// default layer, so a pre-layers project re-saves its own bytes, and as
/// version 2 otherwise. "No marks" is counted over every layer: layers
/// holding no marks write nothing (their names are not kept).
pub(crate) fn colour_paint_save_doc(layer: Option<&cartalith_spatial::colour_paint::ColourPaint>) -> Option<cartalith_spatial::colour_paint::PaintLayers> {
    use cartalith_spatial::colour_paint::MarkShape;
    let r = |v: f64| (v * 1000.0).round() / 1000.0;
    let mut doc = layer.filter(|l| !l.is_empty())?.layers().clone();
    for m in doc.layers.iter_mut().flat_map(|l| l.marks.iter_mut()) {
        match &mut m.shape {
            MarkShape::Line { points_km, radius_km } => {
                for p in points_km.iter_mut() {
                    *p = [r(p[0]), r(p[1])];
                }
                // Never rounded below the layer's own minimum, so a saved
                // stroke always reopens.
                *radius_km = r(*radius_km).max(cartalith_spatial::colour_paint::MIN_RADIUS_KM);
            }
            MarkShape::Area { ring_km, edge_km } => {
                for p in ring_km.iter_mut() {
                    *p = [r(p[0]), r(p[1])];
                }
                *edge_km = r(*edge_km);
            }
        }
        if let Some(e) = m.emissive.as_mut() {
            e.glow_radius_km = r(e.glow_radius_km);
        }
    }
    Some(doc)
}

/// Reads CP-3's slot back. Absent is `None` with no warning (every archive
/// written before the slot, and every untouched project); a member that is
/// not a colour-paint document, or one the layer refuses (an unknown version
/// or blend, a cap, a non-finite value), is a warning and `None` -- the rest
/// of the project opens.
pub(crate) fn colour_paint_from_data(data: &project::ProjectData, warnings: &mut Vec<String>) -> Option<cartalith_spatial::colour_paint::PaintLayers> {
    use cartalith_spatial::colour_paint::{ColourPaint, PaintLayers};
    // Either version: a version-1 member (every archive written before
    // Ruling CS) reads as one "Paint 1" layer.
    match data.parse::<PaintLayers>(SLOT_COLOUR_PAINT)? {
        Err(_) => {
            warnings.push(format!("{SLOT_COLOUR_PAINT}: unreadable -- the colour paint was not restored"));
            None
        }
        Ok(doc) => match ColourPaint::from_layers(doc.clone(), 0.0) {
            Ok(_) => Some(doc),
            Err(e) => {
                warnings.push(format!("{SLOT_COLOUR_PAINT}: {e} -- the colour paint was not restored"));
                None
            }
        },
    }
}

fn insert_doc<T: Serialize>(out: &mut BTreeMap<String, String>, slot: &str, value: &T) {
    // `to_string_pretty` cannot fail for these shapes (no non-string map
    // keys, no non-finite floats reachable from them), and a failure here
    // would be a bug rather than a condition -- but a panic crossing the
    // gdext boundary takes the Godot process down, so it is swallowed into
    // "the slot is absent" rather than unwrapped.
    if let Ok(text) = serde_json::to_string_pretty(value) {
        out.insert(slot.to_string(), text);
    }
}

/// A saved currency, checked by `FactionRoster::set_currency`'s rules on
/// read: nothing on faction `0` (Unclaimed issues no currency), a blank name
/// or symbol is unset, and a rate that fails `cartalith_civ::currency::
/// valid_rate` is **dropped with a warning** -- never kept, and never
/// replaced by `1`, which would read as a rate someone chose. An absent
/// member (every project saved before 2026-09-27) is every member unset.
fn currency_from_dto(f: &FactionDto, warnings: &mut Vec<String>) -> civ_roster_bridge::FactionCurrency {
    let Some(c) = f.currency.as_ref() else {
        return civ_roster_bridge::FactionCurrency::default();
    };
    if f.id == 0 {
        warnings.push("entities/factions.json: a currency on faction 0 (Unclaimed) was ignored".to_string());
        return civ_roster_bridge::FactionCurrency::default();
    }
    let text = |v: &Option<String>| v.as_ref().map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
    let symbol = text(&c.symbol).filter(|s| s.chars().count() <= civ_roster_bridge::CURRENCY_SYMBOL_MAX);
    if c.symbol.is_some() && symbol.is_none() && text(&c.symbol).is_some() {
        warnings.push(format!(
            "entities/factions.json: faction {}'s currency symbol is longer than {} characters and was ignored",
            f.id,
            civ_roster_bridge::CURRENCY_SYMBOL_MAX
        ));
    }
    let rate = match c.rate {
        Some(r) if cartalith_civ::currency::valid_rate(r) => Some(r),
        Some(r) => {
            warnings.push(format!(
                "entities/factions.json: faction {}'s exchange rate {r} is not a positive number and was ignored; it reads as unset (at par, default)",
                f.id
            ));
            None
        }
        None => None,
    };
    civ_roster_bridge::FactionCurrency { name: text(&c.name), symbol, rate }
}

/// Rebuilds a [`CivData`] from an opened archive. `None` when the archive carries no settlement document at all -- a project saved before any civilisation layer existed, not a failure -- and also when that document does not parse (no warning is raised for that case here).
///
/// Every element it drops is reported into `warnings` (§6.4a's closing rule: *"every substitution above is reported, never silent"*). `n` is the world's `gw * gh`; a raster of any other length is treated as absent.
fn civ_from_project(data: &cartalith_io::ProjectData, n: usize, warnings: &mut Vec<String>) -> Option<CivData> {
    let settlements_doc: SettlementsDoc = data.parse(SLOT_SETTLEMENTS)?.ok()?;

    let mut settlements = Vec::with_capacity(settlements_doc.settlements.len());
    let mut trade_balances = Vec::with_capacity(settlements_doc.settlements.len());
    let mut place_extras = std::collections::HashMap::new();
    let mut village_tids = std::collections::HashSet::new();
    let mut next_tid = settlements_doc.next_id;

    for d in &settlements_doc.settlements {
        settlements.push(dto_to_settlement(d));
        trade_balances.push(cartalith_civ::TradeBalance {
            exports: d
                .trade
                .as_ref()
                .map(|t| {
                    t.exports
                        .iter()
                        .filter_map(|k| resource_key_static(k))
                        .collect()
                })
                .unwrap_or_default(),
            imports: d
                .trade
                .as_ref()
                .map(|t| {
                    t.imports
                        .iter()
                        .filter_map(|k| resource_key_static(k))
                        .collect()
                })
                .unwrap_or_default(),
        });
        if let Some(e) = &d.extras {
            place_extras.insert(
                d.id,
                civ_roster_bridge::PlaceExtras {
                    specialisation: e.specialisation.clone(),
                    traits: e.traits.clone(),
                    history: e.history.clone(),
                    age: e.age,
                    walls: e.walls,
                    culture: e.culture.clone(),
                    rules_preset: e.rules_preset.clone(),
                    variant: e.variant,
                },
            );
        }
        if d.village_seeded {
            village_tids.insert(d.id);
        }
        // §9.1: `next_id` is raised to past every id actually present. A
        // stored counter lower than the ids beside it would hand the next
        // hand-placed settlement an id that already exists.
        next_tid = next_tid.max(d.id.saturating_add(1));
    }

    let ways_doc: WaysDoc = data
        .parse(SLOT_WAYS)
        .and_then(|r| r.ok())
        .unwrap_or_default();
    let ways: Vec<cartalith_civ::Way> = ways_doc.roads.iter().map(dto_to_road).collect();
    let sea_routes: Vec<cartalith_civ::SeaRoute> = ways_doc
        .sea_lanes
        .iter()
        .map(|d| {
            let pts = dto_to_pts(&d.points);
            let brks = sane_breaks(&d.breaks, pts.len());
            cartalith_civ::SeaRoute {
                pts,
                brks,
                km: d.length_km,
                name: d.name.clone(),
            }
        })
        .collect();
    for w in &ways {
        next_tid = next_tid.max(w.tid.saturating_add(1));
    }
    // §9.3's `road_edges`. Absent restores as none -- every archive written
    // before the member existed, which reopens exactly as it always did. An
    // edge with a cell off the grid is dropped and reported (§6.4a rung 2):
    // `cells` is all the Journey Planner reads (`jp_road_cells`). `from`/`to`
    // are **not** range-checked: nothing reads them after the router, and a
    // settlement deleted since leaves them stale in memory too, so dropping
    // on them would make the reopened network differ from the saved one.
    let stored_edges = &ways_doc.road_edges;
    let raw: Vec<cartalith_civ::RoadEdge> = stored_edges
        .iter()
        .filter(|e| e.cells.iter().all(|&c| c < n))
        .map(|e| cartalith_civ::RoadEdge { a: e.from, b: e.to, path: e.cells.clone() })
        .collect();
    // §9.3's `road_edges_rw_cap`: the routing grid the cells index. A save
    // written while the cap was 384 (2026-09-24 to 09-28) of a map wider than
    // 384 holds indices on a 384-wide grid; today's readers assume today's
    // grid (`road_edge_map_path`), so such a list is re-expressed on it here
    // (OUTSTANDING_WORK.md "Saves written while `RW_CAP` was 384", 2026-10-12).
    // A list on today's grid is left exactly as stored (`road_edge_regrid`'s
    // identity), and so is every save whose grid size does not match `n`
    // (nothing to scale against).
    let (sgw, sgh) = (data.save.params.gw, data.save.params.gh);
    let today_w = sgw.min(cartalith_civ::RW_CAP);
    let from_w = if raw.is_empty() || sgw * sgh != n {
        today_w
    } else if let Some(cap) = ways_doc.road_edges_rw_cap {
        sgw.min(cap.max(1))
    } else {
        let places: Vec<(usize, usize)> = settlements.iter().map(|s| (s.placement.x, s.placement.y)).collect();
        let (w, decided) = cartalith_civ::road_edges_infer_grid_w(&raw, &places, sgw, sgh);
        // Said only when it was a guess between two real candidates (a map
        // wider than the old cap); a decided move is reported below.
        if !decided && sgw.min(cartalith_civ::RW_CAP_BEFORE_RULING_BT) != today_w {
            warnings.push(format!(
                "{SLOT_WAYS}: road edges carry no grid size (saved before 2026-10-12) and nothing in the save told the old 384-wide grid from today's; read as {w} wide"
            ));
        }
        w
    };
    let road_edges: Vec<cartalith_civ::RoadEdge> = if from_w == today_w {
        raw
    } else {
        raw.into_iter()
            .filter_map(|e| cartalith_civ::road_edge_regrid(&e.path, from_w, sgw, sgh).map(|path| cartalith_civ::RoadEdge { path, ..e }))
            .collect()
    };
    if road_edges.len() < stored_edges.len() {
        warnings.push(format!(
            "{SLOT_WAYS}: {} of {} road edges skipped (a cell outside the grid)",
            stored_edges.len() - road_edges.len(),
            stored_edges.len()
        ));
    }
    if from_w != today_w {
        warnings.push(format!(
            "{SLOT_WAYS}: {} road edges moved from a {from_w}-cell-wide routing grid to today's {today_w}",
            road_edges.len()
        ));
    }

    let provinces_doc: ProvincesDoc = data
        .parse(SLOT_PROVINCES)
        .and_then(|r| r.ok())
        .unwrap_or_default();
    let province_list: Vec<cartalith_civ::Province> = provinces_doc
        .provinces
        .iter()
        // §9.4: an index that points nowhere costs the province, not the
        // project.
        .filter(|p| p.capital_settlement_index < settlements.len())
        .map(|p| cartalith_civ::Province {
            id: p.id,
            faction: p.faction,
            name: p.name.clone(),
            capital_settlement_index: p.capital_settlement_index,
        })
        .collect();

    let continents_doc: ContinentsDoc = data
        .parse(SLOT_CONTINENTS)
        .and_then(|r| r.ok())
        .unwrap_or_default();
    let continents: Vec<cartalith_civ::Continent> = continents_doc
        .continents
        .iter()
        .map(|c| cartalith_civ::Continent {
            id: c.id,
            name: c.name.clone(),
            cells: c.cells,
            min_x: c.min_x,
            min_y: c.min_y,
            max_x: c.max_x,
            max_y: c.max_y,
            cx: c.cx,
            cy: c.cy,
            faction: c.faction,
        })
        .collect();

    let factions_doc: FactionsDoc = data
        .parse(SLOT_FACTIONS)
        .and_then(|r| r.ok())
        .unwrap_or_default();
    let faction_roster = if factions_doc.factions.is_empty() {
        civ_roster_bridge::FactionRoster::seeded(crate::CIV_FACTION_COUNT as usize)
    } else {
        civ_roster_bridge::FactionRoster(
            factions_doc
                .factions
                .iter()
                .map(|f| civ_roster_bridge::FactionEntry {
                    name: f.name.clone(),
                    culture: f.culture.clone(),
                    religion: f.religion.clone(),
                    government: f.government.clone(),
                    ag_tech: f.ag_tech.clone(),
                    color: (f.color[0], f.color[1], f.color[2]),
                    color_override: f.user_color.map(|c| (c[0], c[1], c[2])),
                    // Rows outside `0..=1`, at 0, or naming a faction id this
                    // roster does not have are dropped rather than trusted:
                    // `FactionRoster::set_tariff`'s rules, applied on read.
                    tariffs: f
                        .tariffs
                        .iter()
                        .filter(|&(&x, &r)| {
                            f.id != 0
                                && x < factions_doc.factions.len()
                                && x != f.id
                                && r > 0.0
                                && r <= 1.0
                        })
                        .map(|(&x, &r)| (x, r))
                        .collect(),
                    currency: currency_from_dto(f, warnings),
                    religious_policy: match &f.religious_policy {
                        None => "plural".to_string(),
                        // Kept verbatim, known or not (an unknown key is a
                        // newer build's and is preserved on write-back).
                        Some(serde_json::Value::String(k)) => k.clone(),
                        Some(_) => {
                            warnings.push(format!(
                                "{SLOT_FACTIONS}: faction {} \"{}\": its religious policy is not text and reads as Plural",
                                f.id, f.name
                            ));
                            "plural".to_string()
                        }
                    },
                })
                .collect(),
        )
    };

    // R1: absent (every pre-R1 archive) is an empty library and no warning.
    let religions = religions_from_data(data, warnings);
    // A faction naming a custom religion the library does not hold (the
    // document was dropped, damaged, or hand-edited) keeps its stored key --
    // never re-pointed at a guess -- and the engine reads it as no religion.
    // Said once here so the loss is visible at open, not only in the hub.
    for (i, e) in faction_roster.0.iter().enumerate() {
        if let cartalith_civ::religion_library::ReligionChoice::Missing(raw) = religions.choice(&e.religion) {
            warnings.push(format!(
                "{SLOT_FACTIONS}: faction {i}'s religion {raw} names no custom religion in this project -- it behaves as having none and shows as a missing religion until another is picked"
            ));
        }
    }

    let timeline_doc: TimelineDoc = data
        .parse(SLOT_TIMELINE)
        .and_then(|r| r.ok())
        .unwrap_or_default();
    // §10.1: sorted ascending, unique. Enforced here rather than trusted,
    // because the year cursor walks this list by index -- and, since owner
    // ruling 27, because `civ_snapshot_save` encodes each year against the
    // one before it and would otherwise be handed them out of order.
    let mut years: Vec<&TimelineYearDto> = timeline_doc.years.iter().collect();
    years.sort_by_key(|y| y.year);
    years.dedup_by_key(|y| y.year);
    let mut timeline: Vec<cartalith_civ::timeline::TimelineSnapshot> = Vec::new();
    for y in years {
        cartalith_civ::timeline::civ_snapshot_save(
            &mut timeline,
            y.year,
            // A snapshot with no stored territory raster is legal (§10.2)
            // and is an empty vec, which is what a reader of `territory`
            // must already tolerate for a year recorded before territory
            // existed.
            data.history_territory
                .get(&y.year)
                .cloned()
                .unwrap_or_default(),
            y.settlements.iter().map(dto_to_settlement).collect(),
            y.ways.iter().map(dto_to_road).collect(),
        );
        // §10.1: the institutions record is optional per year; absent members stay None
        // ("not recorded: read today's"), which is what civ_snapshot_save just left.
        cartalith_civ::timeline::civ_snapshot_set_record(&mut timeline, y.year, year_record_from_dtos(y));
    }

    // Milestone 4's reseed, at the one place in this port where a recorded
    // history is attached to a counter that was derived without it.
    //
    // `next_tid` above starts from `settlements_doc.next_id`, which carries
    // `#[serde(default)]` like every other member here -- §14.3's rule in its
    // constructive direction. An archive written by a second implementation
    // that omits the member (or by a build that predates it) therefore lands
    // on `0`, and the loop below rebuilds the counter from the *live*
    // settlements and ways only. A settlement recorded in `history/
    // timeline.json` and since deleted is in neither, so the next hand-placed
    // settlement would be issued an id a snapshot already uses -- and
    // `civ_year_diff` compares snapshots by `tid`.
    //
    // `civ_resync_next_tid_with_timeline`, not `civ_resync_next_tid`: the
    // milestone-1 twin scans only the live arrays, which is exactly the scan
    // that has already happened above and would change nothing.
    next_tid = next_tid.max(cartalith_civ::timeline::civ_resync_next_tid_with_timeline(
        &settlements,
        &ways,
        &timeline,
    ));

    // Ruling AT's held year (§10.1), kept only while it still names a
    // recorded year -- `civ_remove_year`'s own rule for a year gone from the
    // track. Absent (every earlier archive) is `None`, as it always was.
    let territory_year = timeline_doc.territory_year.filter(|y| timeline.iter().any(|s| s.year == *y));

    // §8.1: an absent claim grid is "No territory" -- not a grid of unowned
    // cells, which is a real answer (everything unclaimed) and which the next
    // save would then write back as if it had been read. So it stays empty,
    // the shape every reader of `CivData::territory` already tests for
    // (`len() == gw * gh`), a re-save leaves it absent, and the loss is
    // reported. A raster that is present but the wrong size or element type
    // is the same unknown, and says so.
    let territory = match data.raster("rasters/territory.i32") {
        Some(Raster::I32(v)) if v.len() == n => v.clone(),
        other => {
            warnings.push(if other.is_none() {
                "rasters/territory.i32: absent -- the settlements came back but their claimed territory did not; \
                 paint it, or run Recompute civilisation (which needs the full world), to rebuild it"
                    .to_string()
            } else {
                "rasters/territory.i32: not an i32 grid of this world's size -- territory was not restored; \
                 paint it, or run Recompute civilisation (which needs the full world), to rebuild it"
                    .to_string()
            });
            Vec::new()
        }
    };
    // The province raster, under the same §8.1 rule as the claim grid: absent
    // (or unreadable) is "provinces not known", held empty -- never a grid of
    // zeros, which says "no cell is in any province" and which the next save
    // would write back as if it had been read. Every reader of
    // `CivData::provinces` tests `len() == gw * gh` (or reads it with `get`,
    // which finds nothing); the first committed Territory stroke rebuilds it
    // over a whole claim grid (`civ_reprovince`). The province list itself is
    // `entities/provinces.json`'s and is kept.
    let provinces = match data.raster("rasters/provinces.i32") {
        Some(Raster::I32(v)) if v.len() == n => v.clone(),
        other => {
            warnings.push(if other.is_none() {
                "rasters/provinces.i32: absent -- the province list came back but which cells each province \
                 covers did not; province boundaries are not drawn until the next territory edit or \
                 Recompute civilisation rebuilds them"
                    .to_string()
            } else {
                "rasters/provinces.i32: not an i32 grid of this world's size -- province cells were not \
                 restored; province boundaries are not drawn until the next territory edit or Recompute \
                 civilisation rebuilds them"
                    .to_string()
            });
            Vec::new()
        }
    };
    let water_bodies = match data.raster("rasters/water_bodies.u8") {
        Some(Raster::U8(v)) if v.len() == n => v.clone(),
        _ => Vec::new(),
    };
    let dens = match data.raster("rasters/agrarian_density.f32") {
        Some(Raster::F32(v)) if v.len() == n => v.clone(),
        _ => Vec::new(),
    };

    Some(CivData {
        settlements,
        ways,
        // Stored since 2026-09-24 (§9.3): the raw cell paths cannot be
        // recovered from the smoothed `ways` they were consolidated into, and
        // the Journey Planner reads both. Empty for an archive written before.
        road_edges,
        sea_routes,
        territory,
        provinces,
        province_list,
        continents,
        trade_balances,
        // Deliberately empty: an explanation is a diagnostic over
        // suitability rasters the archive does not store
        // (`SAVEFILE_COMPAT.md` §16.2), and synthesising one from what is
        // stored would be inventing a reason rather than recalling it.
        explanations: Vec::new(),
        water_bodies,
        next_tid,
        timeline,
        year: timeline_doc.year,
        dens,
        faction_roster,
        religions,
        place_extras: civ_roster_bridge::PlaceExtrasTable(place_extras),
        village_tids,
        // Not persisted: `CivData::belief`'s own doc states the decision and
        // its one cost. A restored project starts with no belief layer, the
        // same as a freshly generated world.
        belief: Vec::new(),
        belief_seed_key: Vec::new(),
        territory_year,
    })
}

/// The Territory/Settlement tools' model for a reopened project, as `absorb`
/// builds it for a generate, from what the archive actually holds:
///
/// - **The paint base** is the restored claim grid (every stroke committed
///   before the save is already merged into it; §16.6). When the archive
///   carried no claim grid (`civ.territory` empty, reported by
///   [`civ_from_project`]) nothing is known to be claimed, so the base is
///   "nothing claimed" at this world's size: a stroke then commits onto a
///   whole grid and a subtract falls through to unclaimed -- the only owner
///   the archive can vouch for. `civ.territory` itself stays empty until that
///   first commit, so nothing reads the unknown as "all unclaimed" before then.
/// - **The manual-placement name stream** resumes at §9.1's `name_stream`, so
///   a blank-named drop after reopening cannot draw a name one before the save
///   already drew. Absent (every archive written before 2026-09-24) keeps the
///   seed-folded start `CivTools::new` gives.
fn civ_tools_for_reopen(
    gw: usize,
    gh: usize,
    civ: &CivData,
    seed: u32,
    data: &cartalith_io::ProjectData,
) -> crate::civ_tools_bridge::CivTools {
    let base = if civ.territory.len() == gw * gh { civ.territory.clone() } else { vec![0; gw * gh] };
    let mut tools = crate::civ_tools_bridge::CivTools::new(gw, gh, base, seed);
    if let Some(state) = civ_name_stream(data) {
        tools.name_rng = cartalith_rng::Mulberry32::new(state);
    }
    tools
}

/// §9.1's `name_stream`, or `None` when the settlements document is absent,
/// unreadable, or predates the member.
fn civ_name_stream(data: &cartalith_io::ProjectData) -> Option<u32> {
    data.parse::<NameStreamOnly>(SLOT_SETTLEMENTS).and_then(|r| r.ok()).and_then(|d| d.name_stream)
}

// ===================== the Godot surface =====================

/// The failure shape every `#[func]` below returns: `{ok: false, error: <text>}`.
/// GDScript callers must test `ok` before reading any other key, since this
/// dictionary carries nothing else (no partial result).
fn err(message: impl std::fmt::Display) -> VarDictionary {
    vdict! { "ok" => false, "error" => message.to_string() }
}

/// The document channel's two rules, stated once: a slot must be one the
/// format defines, and it must not be one the engine writes and reads
/// itself. Returns the refusal, or `None` when the slot is the caller's to
/// have.
///
/// The engine-owned half is the rule that matters. Handing
/// `entities/settlements.json` back as text would let the shell parse it,
/// edit it and write it again — and then there would be two answers to
/// "what are this world's settlements", one in `CivData` and one in
/// GDScript, with no rule about which wins. The engine's own accessors are
/// the single answer; this refusal is what keeps them so.
fn caller_slot_refusal(slot: &str) -> Option<String> {
    if !cartalith_io::DOCUMENT_SLOTS.contains(&slot) {
        return Some(format!("{slot} is not a slot the project format defines"));
    }
    if ENGINE_OWNED_SLOTS.contains(&slot) {
        return Some(format!(
            "{slot} is written and read by the engine itself; ask the engine for its \
             contents (get_settlements(), get_ways(), the faction roster, the timeline, \
             journey_list()/journey_get()) rather than round-tripping the document \
             through the shell"
        ));
    }
    None
}

#[godot_api(secondary)]
impl WorldGen {
    /// Saves the whole project as a `.zip` in the tree layout
    /// `SAVEFILE_COMPAT.md` specifies — terrain, parameters, the
    /// civilisation layer, labels, icons, the selected region, recorded
    /// history and the vault links.
    ///
    /// `path` is a native OS filesystem path, the same convention
    /// `load_save`/`save_project`/`load_asset_pack` already use.
    ///
    /// Returns `{ok, error, entries, bytes}`. On any failure `ok` is
    /// `false`, `error` says why, and **any existing file at `path` is
    /// untouched**: the archive is built in memory and only written once it
    /// is complete, so a full disk cannot leave a half-written save where a
    /// good one used to be (`SAVEFILE_COMPAT.md` §3.2).
    ///
    /// This is the call the shell's Save command should make.
    /// `save_project` remains as the flat-layout **interoperability
    /// export** (§1.1) and is lossy by construction — it can carry no part
    /// of the project layer.
    #[func]
    fn project_save(&mut self, path: GString) -> VarDictionary {
        self.project_save_with_documents(path, VarDictionary::new(), PackedByteArray::new(), false)
    }

    /// [`WorldGen::project_save`] plus the caller's own documents.
    ///
    /// `extra_documents` maps a registered slot name
    /// (`project_document_slots()`) to that document's JSON **text**. This
    /// is the channel for project state GDScript owns and the engine does
    /// not model — the shell's own panel state, or a payload a future
    /// subsystem lands in the shell before it lands in Rust.
    ///
    /// Text rather than a `Dictionary`, deliberately: a `Dictionary` would
    /// have to be converted through Godot's JSON, which types every number
    /// as a float and is exactly how `GUI_GAP_REGISTER.md` KV-04 discarded
    /// every knowledge link a user ever made. Handing over a string the
    /// caller built with `JSON.stringify` keeps this crate out of that
    /// path entirely; the reader's own coercion pass
    /// (`SAVEFILE_COMPAT.md` §14.2) covers what does slip through.
    ///
    /// A slot the engine writes itself is refused rather than merged — see
    /// [`ENGINE_OWNED_SLOTS`].
    ///
    /// `preview_png` is optional, caller-encoded PNG bytes for
    /// `preview.png` (`SAVEFILE_COMPAT.md`'s thumbnail slot) — Rust has no
    /// rendered world texture to encode at save time, so the shell captures
    /// and encodes it (`Image.save_png_to_buffer()`) and hands the bytes
    /// down. An empty array (the default for every existing call site)
    /// writes no `preview.png` entry at all, exactly as before this
    /// parameter existed.
    ///
    /// `include_lod_tiles` is owner rulings 28/29's optional stored LOD
    /// pyramid — **default `false`**, matching ruling 28's *"off by
    /// default"* and every call site before this parameter existed. `true`
    /// synthesizes [`lod_bridge::SAVE_PYRAMID_MAX_LEVEL`] levels of the deep-
    /// zoom tile pyramid through [`crate::lod_worker::LodSnapshot::render_pyramid_masks`]
    /// and writes them into `cartography/tiles/**`
    /// (`cartalith_io::project::LodTiles`); the shell should show
    /// [`Self::lod_save_pyramid_estimate_bytes`]'s number next to whatever
    /// control sets this flag, per ruling 28's *"the size shown at save
    /// time"*. Silently omitted (as if `false`) rather than failing the save
    /// if no world context can be built — the archive is still complete
    /// without a cached pyramid (`SAVEFILE_COMPAT.md` §16.1: it is a
    /// derived cache, never authored data).
    #[func]
    fn project_save_with_documents(
        &mut self,
        path: GString,
        extra_documents: VarDictionary,
        #[opt(default = &PackedByteArray::new())] preview_png: PackedByteArray,
        #[opt(default = false)] include_lod_tiles: bool,
    ) -> VarDictionary {
        let Some(source) = self.source.as_ref() else {
            return err("no world to save");
        };
        let n = (self.gw.max(0) as usize) * (self.gh.max(0) as usize);

        // The same saturation the reference exporter applies (`o > 255 ?
        // 255 : o`): `stream_order` is wider in memory than in the archive.
        let fields = match source {
            WorldSource::Generated(ws) => cartalith_io::SaveFields {
                // See `lib.rs`'s own `save_project`: `heightmap`/`temperature`/
                // `rainfall` are `Arc<Vec<f32>>` on `SaveFields` now, so this is
                // a refcount bump rather than a copy.
                heightmap: ws.field.clone(),
                temperature: ws.temperature.clone(),
                rainfall: ws.rainfall.clone(),
                volcanic_field: ws.volcanic_field.as_ref().clone(),
                impact_field: ws.impact_field.clone(),
                strahler_order: match ws.stream_order.as_ref() {
                    Some(order) => order.iter().map(|&o| o.clamp(0, 255) as u8).collect(),
                    None => vec![0u8; n],
                },
            },
            WorldSource::Loaded(save) => save.fields.clone(),
        };
        let params = cartalith_io::SaveParams {
            gw: self.gw.max(0) as usize,
            gh: self.gh.max(0) as usize,
            seed: self.seed,
            map_width_km: self.map_width_km,
            // The *effective* sea level, not the user's input: a
            // World-Structure archetype re-anchors it, and a reader must
            // classify land the same way this session does
            // (`SAVEFILE_COMPAT.md` §7).
            sea_level: self.sea_level,
            world: self.world,
            // Provenance (`world_origin`), carried into the archive so a
            // world that was imported or resampled reopens as one -- and
            // `None` written as an absent key, so re-saving an archive that
            // never said does not make it start claiming.
            origin: self.world_origin.clone(),
            // The world's own display name, same shape as `origin`
            // immediately above -- absent when this session's `world_name`
            // is `None`, never a fabricated one. `SAVEFILE_COMPAT.md` §7's
            // `world.name`, added for `OUTSTANDING_WORK.md`'s Recent-worlds
            // row.
            name: self.world_name.clone(),
        };

        let mut write = ProjectWrite::new(&params, &fields);
        write.readme = Some(project::DEFAULT_README.to_string());
        // Owner Ruling AR (2026-09-24): the rest of the `WorldState`, so the
        // project reopens as this world and not as a terrain-only stand-in
        // (`SAVEFILE_COMPAT.md` §8.3, `substrate.rs`). A `Loaded` world has
        // none to write -- it was opened from an archive that did not carry
        // one -- and saving it keeps it that way, honestly, rather than
        // inventing grids. `Err` (a grid of the wrong length, a stream order
        // the core `u8` raster cannot hold) writes none of it: a manifest
        // claiming a set it does not have is worse than no set.
        let substrate_written = match source {
            WorldSource::Generated(ws) => match crate::substrate::write_substrate(ws, n, &mut write) {
                Ok(()) => true,
                Err(e) => {
                    godot_warn!("cartalith-godot: saved without the world substrate ({e})");
                    false
                }
            },
            WorldSource::Loaded(_) => false,
        };
        // GF-8's save slice (`SAVEFILE_COMPAT.md` §8.4, `geology_save.rs`): the
        // geology column, painted rock included, beside the substrate it is
        // read with. Only with the substrate: a reopened column needs the
        // `WorldState` the substrate rebuilds, so a column written without one
        // could never be restored. A world with no column (`Geology::Absent`:
        // the model off, an import, or a project reopened from a save without
        // the set) writes nothing -- which is also what makes a re-save of such
        // a project carry no geology members at all. `age` is the clock of
        // this session's parameters, the dial `params.json` also records.
        let geology_written = match source {
            WorldSource::Generated(ws) if substrate_written => {
                let (gw, gh) = (self.gw.max(0) as usize, self.gh.max(0) as usize);
                match crate::geology_save::write_geology(ws, gw, gh, self.recompute_params().geo_age, &mut write) {
                    Ok(w) => w,
                    Err(e) => {
                        godot_warn!("cartalith-godot: saved without the geology column ({e})");
                        false
                    }
                }
            }
            _ => false,
        };
        // `preview.png` — see the doc comment above for why this is
        // caller-supplied. Empty/absent writes no entry (the damage ladder
        // for a missing thumbnail is "no thumbnail", never a zero-byte
        // file), matching every call site before this parameter existed.
        if !preview_png.is_empty() {
            write.preview_png = Some(preview_png.to_vec());
        }

        // Owner rulings 28/29's optional stored LOD pyramid — off unless the
        // caller asked. `source_key` is left empty deliberately:
        // `cartalith_io::project::write_project` computes and stamps the real
        // key itself from the heightmap this very call is writing, and an
        // **empty** key is that writer's documented contract for "a fresh
        // producer" (its own doc comment on `LodTiles::source_key`) — filling
        // it in here would be asserting a claim this function cannot verify.
        if include_lod_tiles {
            if let Some(snapshot) = self.lod_snapshot() {
                // Through the worker, not `snapshot.render_pyramid_masks`: a
                // project reopened with a pyramid whose producer is still this
                // snapshot's writes those held tiles straight back instead of
                // drawing every one again (`LodWorker::pyramid_masks`). Any
                // tile the seed does not validly cover is synthesised as before.
                if let Some((tile_w, tile_h, tiles)) =
                    self.lod_worker.pyramid_masks(&snapshot, lod_bridge::SAVE_PYRAMID_MAX_LEVEL)
                {
                    write.lod_tiles = Some(project::LodTiles {
                        source_key: String::new(),
                        // The snapshot's own id: `tile_producer_id` plus a
                        // digest of every other tile input, the colour space
                        // among them -- what `project_open` compares before
                        // it will seed these tiles back in
                        // (`LodSnapshot::producer_id`).
                        producer: snapshot.producer_id().to_string(),
                        tile_w,
                        tile_h,
                        tiles,
                    });
                }
            }
        }

        // params.json's two views. `params::save_state` builds one object
        // carrying both -- the reference-named block with this port's own
        // dotted keys nested under `cartalith` -- and the tree splits them
        // so neither is buried inside the other (§13.1).
        let mut state = params::save_state(&self.params);
        if let Some(obj) = state.as_object_mut() {
            write.cartalith_params = obj.remove("cartalith").unwrap_or(serde_json::Value::Null);
        }
        write.reference_params = state;

        let mut documents: BTreeMap<String, String> = BTreeMap::new();
        if let Some(civ) = self.civ.as_ref() {
            civ_documents(civ, self.civ_tools.as_ref().map(|t| t.name_rng.state()), &mut documents);
            civ_rasters(civ, n, &mut write);
        }
        // Ruling BO: the forced-lake mask, beside the civ rasters but not
        // inside them -- it is the drawn map's, and a world without a civ
        // layer can hold one.
        forced_lakes_raster(self.forced_lake_mask(), n, &mut write);

        // Hand-drawn ways and routes merge into the one ways document, so
        // that §9.3's "one home for every linear route" holds in the file
        // even though the two sources are different types in memory.
        if let Some(infra) = self.infra.as_ref() {
            let mut ways: WaysDoc = documents
                .get(SLOT_WAYS)
                .and_then(|t| serde_json::from_str(t).ok())
                .unwrap_or_default();
            ways.manual = infra
                .ways
                .iter()
                .map(|w| ManualWayDto {
                    name: w.name.clone(),
                    points: pts_to_dto(&w.pts),
                    breaks: w.brks.clone(),
                    length_km: w.km,
                    kind: manual_way_kind_key(w.way_type).to_string(),
                    sea: w.sea,
                    hidden: w.hidden,
                })
                .collect();
            ways.routes = infra
                .routes
                .iter()
                .map(|r| RouteDto {
                    name: r.name.clone(),
                    points: pts_to_dto(&r.pts),
                    breaks: r.brks.clone(),
                    length_km: r.km,
                    mode: route_mode_key(r.mode).to_string(),
                    unreachable_legs: r.unreachable_legs,
                })
                .collect();
            if !ways.manual.is_empty()
                || !ways.routes.is_empty()
                || documents.contains_key(SLOT_WAYS)
            {
                insert_doc(&mut documents, SLOT_WAYS, &ways);
            }
            if let Some(region) = infra.region {
                insert_doc(
                    &mut documents,
                    SLOT_REGIONS,
                    &RegionsDoc {
                        region: Some(RegionDto {
                            x: region.x,
                            y: region.y,
                            w: region.w,
                            h: region.h,
                        }),
                    },
                );
            }
            // `STORY_PLANNING_SCOPE.md` SP-1. Written only when there is at
            // least one, same "absent, not empty" discipline as `ways`/
            // `region` just above -- an untouched project should not gain a
            // zero-length `entities/journeys.json`.
            if !infra.journeys.is_empty() {
                insert_doc(
                    &mut documents,
                    SLOT_JOURNEYS,
                    &JourneysDoc {
                        next_id: infra.next_journey_id(),
                        journeys: infra.journeys.iter().map(journey_to_dto).collect(),
                    },
                );
            }
        }

        if let Some(labels) = self.labels.as_ref().filter(|l| !l.labels.is_empty()) {
            insert_doc(
                &mut documents,
                SLOT_LABELS,
                &LabelsDoc {
                    labels: labels
                        .labels
                        .iter()
                        .map(|l| LabelDto {
                            x: l.x,
                            y: l.y,
                            name: l.name.clone(),
                            angle: l.angle,
                            arc: l.arc,
                            size: l.size,
                            font: l.font.clone(),
                            color: l.color.clone(),
                            size_mode: label_size_mode_key(l.size_mode).to_string(),
                            class: l.class.key().to_string(),
                        })
                        .collect(),
                },
            );
        }

        if let Some(icons) = self.icons.as_ref().filter(|i| !i.icons.is_empty()) {
            insert_doc(
                &mut documents,
                SLOT_ICONS,
                &IconsDoc { icons: icons.icons.iter().map(icon_to_dto).collect() },
            );
        }

        insert_doc(
            &mut documents,
            SLOT_APPEARANCE,
            &AppearanceDoc {
                quality: self.quality.name().to_string(),
                look: self.look.clone(),
                territory_opacity: self.territory_opacity,
                overrides: self.appearance_over.iter().map(|(k, v)| (k.clone(), *v)).collect(),
                ramp: self.appearance_ramp.clone(),
                npr: self.npr.clone(),
                biome_cols: biome_cols_member(&self.biome_col_overrides),
                terrain_adjust: adjust_member(self.appearance_terrain_adjust.as_ref()),
                water_adjust: adjust_member(self.appearance_water_adjust.as_ref()),
            },
        );
        // CP-3: the colour paint, only when the layer has marks.
        if let Some(doc) = colour_paint_save_doc(self.colour_paint_layer()) {
            insert_doc(&mut documents, SLOT_COLOUR_PAINT, &doc);
        }

        // The Landmark dock's settings. Unconditional, unlike the editors
        // above: `LandmarkSettings::default()` is not "nothing" -- it is
        // forty-nine caps and forty-nine armed flags derived from each
        // kind's own spec -- so "absent" and "at defaults" are the same
        // state and writing the block costs a few hundred bytes for a
        // document the user can then diff.
        //
        // The retained run is written **only when there is one** (owner
        // ruling 10, 2026-09-06). `LandmarkStore::last` is `None` before any
        // pass and after every `invalidate()`, and that state has its own
        // meaning the panel draws as `—`; `skip_serializing_if` keeps it out
        // of the file rather than writing an empty run that would read as
        // "it ran and placed nothing".
        insert_doc(
            &mut documents,
            SLOT_LANDMARKS,
            &LandmarksDoc {
                settings: LandmarkSettingsDto::from(&self.landmark_store.settings),
                results: self.landmark_store.last.as_ref().map(LandmarkRunDto::from),
                viewshed_refined: self.landmark_store.refined_view.as_ref().map(encode_viewshed_refined),
            },
        );

        // SP-4. Written only when there is one, the journeys slot's "absent,
        // not empty" rule: an untouched project gains no conflicts document.
        if !self.conflicts.conflicts.is_empty() {
            insert_doc(
                &mut documents,
                SLOT_CONFLICTS,
                &ConflictsDoc {
                    next_id: self.conflicts.next_id,
                    conflicts: self.conflicts.conflicts.iter().map(conflict_to_dto).collect(),
                    // Ruling AY: every write from the cell-centre fix on
                    // marks its convention, so a future reader never has to
                    // guess from a missing member.
                    anchor_convention: "centre".to_string(),
                },
            );
        }

        // The vault's own serialized store, verbatim (§13.3) -- through
        // `LinkStore`'s own `to_json`, so the vault keeps owning its shape
        // and this file never has to know what a knowledge link is. Skipped
        // when there is nothing filed, so an untouched project carries no
        // empty `vault.json`.
        //
        // **Gated on the whole store, not on `links` alone (2026-09-04).** It
        // read `!self.vault.store.links.is_empty()`, which asks one member of a
        // three-member store: a project carrying a connected vault and a map
        // snapshot but no knowledge links wrote **no document at all**, and the
        // snapshot was lost on save. `vault_snapshot` requires no link anywhere
        // in its path, so that state is reachable. `LinkStore::is_empty()` is
        // the predicate this gate always wanted -- it answers for `vaults`,
        // `links` and `snapshots` together, and its own doc records what it
        // costs to be right (a store holding only a `VaultRef` is not empty, by
        // §26's save-file model). Found by a verifier; the predicate existed
        // and was mutation-tested in the same batch, and simply was not wired.
        if !self.vault.store.is_empty() {
            documents.insert(SLOT_VAULT.to_string(), self.vault.store.to_json());
        }

        // The caller's own documents, last -- but never over a slot the
        // engine already wrote, because the loser of that race would be
        // invisible.
        for (key, value) in extra_documents.iter_shared() {
            let slot = key.to_string();
            if ENGINE_OWNED_SLOTS.contains(&slot.as_str()) {
                return err(format!(
                    "{slot} is written by the engine; it cannot be supplied by the caller"
                ));
            }
            if !cartalith_io::DOCUMENT_SLOTS.contains(&slot.as_str()) {
                return err(format!("{slot} is not a slot the project format defines"));
            }
            documents.insert(slot, value.to_string());
        }

        write.documents = documents;
        // `SAVEFILE_COMPAT.md` §6.2, the half that used to be missing: every
        // entry the open project carried that this build does not model goes
        // back out unchanged. Cloned rather than moved because a save is not
        // a close -- the same session may save again, and the second save
        // owes the file the same entries as the first.
        //
        // Empty for a generated world, and for one opened in the flat layout
        // (`read_flat`'s own comment says why it censuses nothing).
        write.foreign = self.carried_foreign.clone();

        let mut buf: Vec<u8> = Vec::new();
        if let Err(e) = project::write_project(std::io::Cursor::new(&mut buf), &write) {
            return err(e);
        }
        let entries = write.documents.len()
            + write.rasters.len()
            + write.history_territory.len()
            + write.foreign.len()
            + 8;
        match std::fs::write(path.to_string(), &buf) {
            Ok(()) => {
                // ED-02's `COMMITTED` rule: everything in the ledger is now
                // in this file, so the boundary moves to the newest row.
                // **Only on `Ok`** -- a refused write leaves the previous
                // file in place (this function's own contract) and must not
                // move a rule that says what is on disk.
                //
                // The shell derived this from `EngineBridge.project_saved`
                // until 2026-09-06 and was right within a session; it is
                // recorded here so the save side and the load side are one
                // fact rather than two that can disagree.
                self.ledger.mark_saved_now();
                let mut d = vdict! { "ok" => true, "error" => "" };
                d.set("bytes", buf.len() as i64);
                d.set("entries", entries as i64);
                // Whether the file carries the world substrate (§8.3): `false`
                // is a world opened from an archive without one, and that
                // file will reopen without journey planning, trade flows,
                // military, town layouts or the faction economy.
                d.set("substrate", substrate_written);
                // Whether the file carries the geology column (§8.4): `false`
                // is a world with no column to write, and that file reopens
                // with no rock readouts and no Rock paint.
                d.set("geology", geology_written);
                d
            }
            Err(e) => err(format!("could not write {path}: {e}")),
        }
    }

    /// Opens a project archive — **either layout** (`SAVEFILE_COMPAT.md`
    /// §1): the tree, or a flat legacy `Cartalith Gen1` export.
    ///
    /// Replaces the world outright and then restores everything the archive
    /// carries: the civilisation layer (so `get_settlements()`,
    /// `get_ways()`, the faction roster, territory and the timeline are all
    /// real for a loaded project), labels, icons, hand-drawn ways and
    /// routes, the selected region, the appearance block, the Landmark
    /// dock's settings, and the vault links.
    ///
    /// Returns
    /// `{ok, error, layout, format_version, warnings, documents, foreign_entries, restored}`:
    ///
    /// - `layout` is `"tree"` or `"flat"`.
    /// - `warnings` is a `PackedStringArray` of everything that was skipped
    ///   and why. Non-empty is **not** failure — a damaged optional payload
    ///   costs itself and nothing else (§6.4) — but it must be shown, or
    ///   the loss is silent.
    ///
    ///   **Two sources, one list.** `cartalith_io::read_project` raises the
    ///   entry- and document-level ones while reading the archive; the
    ///   restore pass below raises **two** row-level ones that §6.4a rung 2
    ///   requires (*"skip that element, keep the rest of the array, and
    ///   report the skip"*) and that were silent until 2026-09-06 — a
    ///   landmark row whose `kind` this build does not know, and an icon row
    ///   whose `family` it cannot resolve. A caller has no use for the
    ///   distinction between the two sources, so there is one key rather
    ///   than two to remember to show.
    ///
    ///   Beside those row-level skips the restore pass also reports a
    ///   substrate it could not honour, and (since 2026-09-24) a civ layer
    ///   whose `rasters/territory.i32` was absent or unreadable -- restored
    ///   with no claim grid rather than an all-unowned one ([`civ_from_project`]).
    ///
    ///   **Two is what this pass drops, not what §6.4a asks for.** They are
    ///   the pass's only two `filter_map`s (`grep -n 'filter_map'` over the
    ///   restore block, 2026-09-06); its rung-1 *substitutions* — a
    ///   settlement `kind` read as `town`, an icon `scale` read as `1.0`, an
    ///   out-of-range way break discarded — are still silent, and
    ///   [`icon_from_dto`] says why they were not half-fixed here.
    /// - `documents` carries the slots the engine did **not** consume, as
    ///   JSON **text**, for the shell to restore its own state from. The
    ///   return half of `project_save_with_documents`' channel, and its
    ///   mirror image: text out, exactly as text went in.
    ///
    ///   **Its keys are also the answer to "which of my documents does this
    ///   archive have?"** — only slots the archive actually carried appear,
    ///   so a caller iterates them rather than guessing slot names and
    ///   testing for empty. That is why no separate list key was added: one
    ///   would be a second copy of these keys, free to drift from them.
    ///   `project_read_document` is the same question asked of a file the
    ///   caller does not want to open.
    /// - `foreign_entries` names entries this build did not understand.
    ///   **They are carried, not dropped**: their bytes are held on
    ///   `WorldGen::carried_foreign` and re-emitted unchanged by the next
    ///   `project_save_with_documents`, which is §6.2's first option. The
    ///   list is therefore informational -- "this archive holds N payloads a
    ///   newer build wrote" -- and no longer a warning a Save command has to
    ///   put in front of the user.
    /// - `lod_tiles_held` is how many stored LOD tiles the archive carried
    ///   and this call held for seeding (`0` when it carried none, or
    ///   `read_project` dropped them as another world's). Held is not used:
    ///   a held tile is served only while the live tile context's producer
    ///   string matches the one it was saved under; `lod_worker_stats`'
    ///   `seeded_served` counts the ones that were.
    /// - `restored` names the engine-owned payloads that were applied, in
    ///   push order: `civ`, `landmark settings`, `landmarks`, `labels`,
    ///   `icons`, `ways`, `region`, `journeys`, `conflicts`, `appearance`,
    ///   `vault`, `paint layers`, `sculpt draft`.
    ///
    ///   **Thirteen, and this list once said seven, then eleven.** It was
    ///   written before the landmark, paint and sculpt restores existed and
    ///   was not moved with them; a caller reading it would have concluded a
    ///   payload that does come back does not. Counted 2026-09-06 at eleven
    ///   with `grep -n 'restored.push' project_bridge.rs`; `journeys`
    ///   (`STORY_PLANNING_SCOPE.md` SP-1) and `conflicts` (SP-4) landed since
    ///   and were not added here either -- the same grep now returns
    ///   thirteen lines.
    ///
    /// - `substrate` is `"complete"` when the archive carried the world
    ///   substrate and this world is its rebuilt `WorldState`, `"absent"` for
    ///   an archive written without one, and `"incomplete"` for one whose
    ///   substrate could not be honoured (with a line in `warnings`). Only a
    ///   `"complete"` world runs the readouts that need flow, channels or the
    ///   tectonic grids. See this module's own doc comment.
    /// - `geology` is the same three words for the geology column
    ///   (`SAVEFILE_COMPAT.md` §8.4, `geology_save.rs`): `"complete"` restored
    ///   it (rock readouts and Rock paint work, painted rock included);
    ///   `"absent"` for an archive written without one; `"incomplete"` for a
    ///   set that could not be honoured (with a line in `warnings`). Only a
    ///   `"complete"` substrate can carry it.
    #[func]
    fn project_open(&mut self, path: GString) -> VarDictionary {
        let file = match std::fs::File::open(path.to_string()) {
            Ok(f) => f,
            Err(e) => return err(format!("could not open {path}: {e}")),
        };
        let mut data = match project::read_project(std::io::BufReader::new(file)) {
            Ok(d) => d,
            Err(e) => return err(e),
        };

        // The terrain half goes in through the existing loader, which
        // already clears every per-world editor for the reason its own
        // comments give; the project layer is restored on top afterwards.
        //
        // **This reads the archive a second time**, and that is a deliberate
        // trade rather than an oversight: `load_save` performs about twenty
        // field resets, and a copy of them here would be two lists that
        // drift, with the failure showing up as an editor from the previous
        // world silently surviving into a loaded one. The cost is one extra
        // decompression pass over the rasters. If it ever measures, the fix
        // is to split `load_save`'s body into a function taking an
        // already-read `SaveData` -- a change in `lib.rs`, which two other
        // agents are editing today.
        if !self.load_save(path.clone()) {
            return err("the world in this archive could not be loaded");
        }

        let n = (self.gw.max(0) as usize) * (self.gh.max(0) as usize);
        let mut restored: Vec<&str> = Vec::new();
        // Warnings raised by the *restore* pass, as distinct from
        // `data.warnings`, which `cartalith_io::read_project` raised while
        // reading the archive. Both go out through the one `warnings` key --
        // a caller has no use for the distinction, and two keys would be two
        // things to remember to show. §6.4a's closing rule is why the list
        // exists at all: *"every substitution above is reported, never
        // silent"*.
        let mut restore_warnings: Vec<String> = Vec::new();

        // Immediately after `load_save`, which cleared the previous project's
        // copy: these bytes are what the *next* save owes this file (§6.2),
        // and `carried_foreign`'s own doc explains why they may not outlive
        // the project they came from. Moved, not cloned -- `data` is dropped
        // at the end of this function and nothing else reads them.
        self.carried_foreign = std::mem::take(&mut data.foreign);

        // Ruling AZ (2026-09-28) plus the fold-in that followed it: `load_save`
        // above already restored `self.params.civ.garrison_exposure_scale`
        // from *this same archive's* `params.json` `cartalith` block via
        // `params::apply_saved_state` -- authoritative whenever that block
        // carries the key at all, defaulting to `cartalith_civ::garrison::
        // EXPOSURE_SCALE` when it does not (`apply_saved_state`'s own reset).
        // The one case that default is wrong for is a project written
        // between 382945a (which put this setting only in `factions.json`)
        // and the fold-in (which gave it a `params.json` row) -- so
        // `resolve_garrison_exposure_scale` reads `data` a second time, the
        // same shape `carried_foreign` above uses, and applies that legacy
        // value only when the new key never reached the archive at all. See
        // that function's own doc comment for the precedence this
        // implements and why.
        self.params.civ.garrison_exposure_scale =
            resolve_garrison_exposure_scale(&data, self.params.civ.garrison_exposure_scale);

        // Owner Ruling AR (2026-09-24): an archive that carries the world
        // substrate (`SAVEFILE_COMPAT.md` §8.3) reopens as the complete
        // `WorldState` it was saved from, so every readout that needs flow,
        // channels or the tectonic grids works on it exactly as it did before
        // the save. One that does not -- written before the substrate existed,
        // a flat legacy archive, or one whose substrate is damaged -- stays the
        // `Loaded` world `load_save` just installed, and those readouts refuse
        // with `substrate::needs_substrate`. Before the civ restore, which
        // does not depend on it, and before the sculpt restore, which does.
        // GF-8's save slice: the geology column (`SAVEFILE_COMPAT.md` §8.4,
        // `geology_save.rs`) goes onto the rebuilt world before it is
        // installed, so every reader sees the column from the first frame.
        // `"absent"` unless the substrate rebuilt a world to carry it.
        let mut geology = "absent";
        let substrate = match self.source.take() {
            Some(WorldSource::Loaded(save)) => match crate::substrate::world_from_project(&mut data, &save) {
                Ok(Some(mut ws)) => {
                    let (gw, gh) = (save.params.gw, save.params.gh);
                    let (g, warning) = crate::geology_save::restore_into(&mut ws, &mut data, gw, gh);
                    geology = g;
                    restore_warnings.extend(warning);
                    self.install_reopened_world(ws);
                    "complete"
                }
                Ok(None) => {
                    self.source = Some(WorldSource::Loaded(save));
                    "absent"
                }
                Err(e) => {
                    restore_warnings.push(format!(
                        "{e} -- the terrain opened, but journey planning, trade flows, military, town layouts \
                         and the faction economy need the full world: regenerate it to use them"
                    ));
                    self.source = Some(WorldSource::Loaded(save));
                    "incomplete"
                }
            },
            // `load_save` returned `true`, so it installed a `Loaded` world;
            // anything else here is put back untouched.
            other => {
                self.source = other;
                "absent"
            }
        };
        // A geology set with no substrate world to carry it (the substrate
        // was damaged, or the archive was hand-assembled) is reported, not
        // silently dropped: the column cannot exist on a `Loaded` world.
        if substrate != "complete" && !data.geology.is_null() {
            geology = "incomplete";
            restore_warnings.push(
                "the archive carries a geology column, but not a usable world substrate to restore it onto -- \
                 the world opened without its rock column, so rock readouts and Rock paint are off on it"
                    .to_string(),
            );
        }

        if let Some(civ) = civ_from_project(&data, n, &mut restore_warnings) {
            // The Territory tool's model, as `absorb` builds it for a generate:
            // the claim grid as the base, no paint, and the manual-placement
            // name stream where the save left it, or folded from this world's
            // seed for an older archive (`load_save` has just set `self.seed`
            // from it) -- [`civ_tools_for_reopen`]. The base is the *restored*
            // grid, which already carries every stroke committed before the
            // save -- the paint layer is not stored -- so a subtract dab
            // restores the saved owner, not the one `assign_territory` first
            // computed; that is the same base `civ_year_loaded` gives the
            // grid after a timeline jump. With it, territory paint and
            // GeoJSON border import work on a reopened project. Built for
            // any restored civ layer, substrate or not: neither reads it.
            self.civ_tools = Some(civ_tools_for_reopen(
                self.gw.max(0) as usize,
                self.gh.max(0) as usize,
                &civ,
                self.seed as u32,
                &data,
            ));
            self.civ = Some(civ);
            self.civ_dirty = false;
            restored.push("civ");
        }

        // Before the editors, because it is the one restore here that is
        // *replacing a reset value rather than filling an empty slot*:
        // `load_save` a few dozen lines above put `landmark_store.settings`
        // back to `LandmarkSettings::default()` precisely so that an archive
        // with no landmark document opens with defaults instead of the
        // previous project's tuning. This puts the archive's own back on top
        // when it has one.
        if let Some(Ok(doc)) = data.parse::<LandmarksDoc>(SLOT_LANDMARKS) {
            self.landmark_store.settings = doc.settings.into_settings();
            restored.push("landmark settings");
            // Owner ruling 10. `load_save` a few dozen lines above called
            // `LandmarkStore::invalidate()`, and its comment gives the reason:
            // *"a landmark sits at grid coordinates over terrain readings
            // taken from the previous field"*. That reason is satisfied here
            // and not evaded -- these placements came out of **this** archive,
            // over the field `load_save` has just installed, at the grid
            // dimensions it has just set. `load_save` reached on its own (a
            // flat reference archive, an HTML-app export) restores nothing
            // here and keeps the cleared store, which is still right for it.
            // Ruling AT: the refined viewshed rides in the same document but is
            // its own restore, so a malformed member drops itself only. Whether
            // it is still *true* of the world is not decided here: the content
            // keys are compared against the live inputs when it is next read.
            if let Some(v) = doc.viewshed_refined.as_ref() {
                match decode_viewshed_refined(v, self.gw.max(0) as usize, self.gh.max(0) as usize) {
                    Ok(rv) => {
                        self.landmark_store.refined_view = Some(rv);
                        restored.push("refined viewshed");
                    }
                    Err(why) => restore_warnings.push(format!(
                        "{SLOT_LANDMARKS}: refined viewshed dropped ({why}); it will read as never refined"
                    )),
                }
            }
            if let Some(run) = doc.results {
                let rows = run.landmarks.len();
                let result = run.into_result(
                    // The same expression `landmark_run()` uses to widen the
                    // seed, so a restored seed and a re-run one agree bit for
                    // bit. `load_save` sets `self.seed` from `save.params`,
                    // so it is this archive's seed and not the last world's.
                    self.seed as u64,
                    self.gw.max(0) as usize,
                    self.gh.max(0) as usize,
                );
                // The same §6.4a rung-2 obligation the icon restore below
                // carries, and it was silent here for the same reason: the
                // count only exists as the difference between the document's
                // rows and the surviving ones, and nothing was subtracting
                // them. `LandmarkRunDto::into_result`'s own doc lists the
                // three ways a row is untrusted.
                let dropped = rows - result.landmarks.len();
                if dropped > 0 {
                    restore_warnings.push(format!(
                        "{SLOT_LANDMARKS}: {dropped} of {rows} landmarks skipped \
                         (unknown kind, off-grid cell, or a non-finite number)"
                    ));
                }
                self.landmark_store.last = Some(result);
                restored.push("landmarks");
            }
        }

        if let Some(Ok(doc)) = data.parse::<LabelsDoc>(SLOT_LABELS) {
            let mut bridge = label_bridge::LabelBridge::new();
            bridge.labels = doc
                .labels
                .iter()
                .map(|d| cartalith_civ::labels::MapLabel {
                    x: d.x,
                    y: d.y,
                    name: d.name.clone(),
                    angle: d.angle,
                    arc: d.arc,
                    size: d.size,
                    font: d.font.clone(),
                    color: d.color.clone(),
                    size_mode: label_size_mode_from(&d.size_mode),
                    class: cartalith_civ::labels::LabelClass::from_key(&d.class).unwrap_or_default(),
                    // A restored label is always hand-placed (`LabelsDoc` is
                    // `SLOT_LABELS`, the Label tool's own list -- the generated
                    // pass is re-run, not saved), so it carries no candidate
                    // weight, same as `MapLabel::new`'s default.
                    weight: 0.0,
                })
                .collect();
            self.labels = Some(bridge);
            restored.push("labels");
        }

        if let Some(Ok(doc)) = data.parse::<IconsDoc>(SLOT_ICONS) {
            let mut editor = icon_bridge::IconEditor::new();
            editor.icons = doc.icons.iter().filter_map(icon_from_dto).collect();
            // §6.4a rung 2: *"the reader MUST skip that element, keep the rest
            // of the array, and report the skip"*, and its closing rule --
            // *"every substitution above is reported, never silent"*. This
            // drop was silent, and [`icon_from_dto`]'s own comment used to
            // claim it was "reported by the caller's own count": no count of
            // it existed anywhere, in this function or in the shell. One
            // warning for the whole document rather than one per row, because
            // a corrupted `family` column would otherwise fill the dialog
            // with one line per icon and say the same thing each time.
            //
            // `family` was the only reason a row could be dropped until
            // 2026-09-06; owner ruling 14's `origin` added a second, so the
            // wording names both rather than blaming the wrong column. See
            // [`icon_from_dto`] for why `origin` is exact-match and absent
            // is not a drop.
            let dropped = doc.icons.len() - editor.icons.len();
            if dropped > 0 {
                restore_warnings.push(format!(
                    "{SLOT_ICONS}: {dropped} of {} icons skipped (unresolvable family or origin)",
                    doc.icons.len()
                ));
            }
            self.icons = Some(editor);
            restored.push("icons");
        }

        let hand_drawn = data
            .parse::<WaysDoc>(SLOT_WAYS)
            .and_then(|d| d.ok())
            .filter(|d| !d.manual.is_empty() || !d.routes.is_empty());
        if let Some(doc) = hand_drawn {
            let mut infra = infra_tools_bridge::InfraTools::new();
            infra.ways = doc
                .manual
                .iter()
                .map(|d| {
                    let pts = dto_to_pts(&d.points);
                    let brks = sane_breaks(&d.breaks, pts.len());
                    cartalith_civ::tools::ManualWay {
                        pts,
                        brks,
                        km: d.length_km,
                        sea: d.sea,
                        way_type: manual_way_kind_from(&d.kind),
                        name: d.name.clone(),
                        hidden: d.hidden,
                    }
                })
                .collect();
            infra.routes = doc
                .routes
                .iter()
                .map(|d| {
                    let pts = dto_to_pts(&d.points);
                    let brks = sane_breaks(&d.breaks, pts.len());
                    infra_tools_bridge::CommittedRoute {
                        pts,
                        brks,
                        km: d.length_km,
                        mode: route_mode_from(&d.mode),
                        unreachable_legs: d.unreachable_legs,
                        name: d.name.clone(),
                    }
                })
                .collect();
            self.infra = Some(infra);
            restored.push("ways");
        }

        if let Some(r) = data
            .parse::<RegionsDoc>(SLOT_REGIONS)
            .and_then(|d| d.ok())
            .and_then(|d| d.region)
        {
            // §11.3: clamped to the grid rather than rejected -- a
            // marquee from a differently-sized world is recoverable,
            // and an out-of-bounds one would fault the export path.
            let gw = self.gw.max(0) as usize;
            let gh = self.gh.max(0) as usize;
            let x = r.x.min(gw.saturating_sub(1));
            let y = r.y.min(gh.saturating_sub(1));
            let w = r.w.min(gw - x.min(gw));
            let h = r.h.min(gh - y.min(gh));
            if w > 0 && h > 0 {
                let infra = self
                    .infra
                    .get_or_insert_with(infra_tools_bridge::InfraTools::new);
                infra.region = Some(cartalith_spatial::Region { x, y, w, h });
                restored.push("region");
            }
        }

        // `STORY_PLANNING_SCOPE.md` SP-1. A parse failure (an archive
        // written by the pre-SP-1 shell, whose `route` member was a bare
        // integer index rather than this shape's nested object) is not
        // fatal to the archive -- `SAVEFILE_COMPAT.md` §6.4a rung 3: a
        // damaged document costs that document, silently skipped here the
        // same way `WaysDoc`/`RegionsDoc` above are on their own parse
        // failure, never `restored.push`ed. Every other document in the
        // archive still opens.
        if let Some(Ok(doc)) = data.parse::<JourneysDoc>(SLOT_JOURNEYS) {
            let journeys: Vec<cartalith_civ::travel_library::Journey> =
                doc.journeys.iter().map(dto_to_journey).collect();
            if !journeys.is_empty() || doc.next_id > 0 {
                let infra = self
                    .infra
                    .get_or_insert_with(infra_tools_bridge::InfraTools::new);
                // §9.1's own "next_id must not trail its data" rule, applied
                // to journeys: raised, never lowered, past the highest id
                // actually present.
                let max_present = journeys.iter().map(|j| j.id + 1).max().unwrap_or(0);
                infra.set_next_journey_id(doc.next_id.max(max_present));
                infra.journeys = journeys;
                restored.push("journeys");
            }
        }

        // SP-4. `load_save` (run first) emptied the store; a damaged document
        // costs itself, and an unreadable row costs that row (§6.4a rung 3).
        if let Some(Ok(doc)) = data.parse::<ConflictsDoc>(SLOT_CONFLICTS) {
            let conflicts = conflicts_from_doc_migrated(&doc);
            if !conflicts.is_empty() {
                // §9.1's rule: `next_id` never trails its data.
                let max_present = conflicts.iter().map(|c| c.id + 1).max().unwrap_or(1);
                self.conflicts = cartalith_civ::conflict::ConflictStore {
                    next_id: doc.next_id.max(max_present),
                    conflicts,
                };
                restored.push("conflicts");
            }
        }

        // CP-3: the colour paint. `load_save` (run first) removed the
        // previous project's layer; an archive without the slot opens with
        // none, a damaged one warns and opens with none.
        if let Some(doc) = colour_paint_from_data(&data, &mut restore_warnings) {
            if self.colour_paint_install_layers(Some(doc)).is_ok() {
                restored.push("colour_paint");
            }
        }

        if let Some(Ok(doc)) = data.parse::<AppearanceDoc>(SLOT_APPEARANCE) {
            // Every member is applied only if this build recognises it: an
            // unknown tier or look name leaves the current one rather than
            // rendering at a quality nobody chose, which is
            // `set_quality_tier`/`set_look`'s own long-standing contract.
            if let Some(t) = crate::render::QualityTier::from_name(&doc.quality) {
                self.quality = t;
            }
            if let Some(n) = crate::render::LOOK_PRESETS
                .iter()
                .find(|n| n.eq_ignore_ascii_case(&doc.look))
            {
                self.look = (*n).to_string();
            }
            if (0.0..=1.0).contains(&doc.territory_opacity) {
                self.territory_opacity = doc.territory_opacity;
            }
            self.appearance_over = doc.overrides.into_iter().collect();
            self.appearance_ramp = doc.ramp;
            self.npr = doc.npr;
            // CA-19. Replaced wholesale, `appearance_over`'s rule: an absent
            // member is this project's table being the reference one, so the
            // outgoing project's edits must not survive into it. A member
            // this build refuses costs itself only -- the default table and a
            // warning, never a partial table and never the rest of the look.
            self.biome_col_overrides = match biome_overrides_from_member(doc.biome_cols.as_ref()) {
                Ok(ov) => ov,
                Err(e) => {
                    restore_warnings.push(format!(
                        "{SLOT_APPEARANCE}: biome_cols {e}; the default biome colour table is used"
                    ));
                    [None; 15]
                }
            };
            // ET-C2's stacks, replaced wholesale on the same rule: absent is no
            // stack, and a member this build refuses costs itself only.
            for (key, member, slot) in [
                ("terrain_adjust", doc.terrain_adjust.as_ref(), &mut self.appearance_terrain_adjust),
                ("water_adjust", doc.water_adjust.as_ref(), &mut self.appearance_water_adjust),
            ] {
                *slot = match adjust_from_member(member) {
                    Ok(s) => s,
                    Err(e) => {
                        restore_warnings.push(format!("{SLOT_APPEARANCE}: {key} {e}; that layer opens with no adjustment"));
                        None
                    }
                };
            }
            restored.push("appearance");
        }

        // The vault is a document this engine *models*, so it goes through
        // the parsed, §14.2-coerced value rather than the verbatim text
        // below: `LinkStore::from_json` is the strict parser KV-04 was
        // about, and the coercion is what stops it refusing a store some
        // other layer re-emitted with `1.0` in it.
        if let Some(Ok(text)) = data.document(SLOT_VAULT).map(serde_json::to_string) {
            // A store this build cannot parse is skipped, never merged and
            // never allowed to clear the links already in memory -- the
            // rule `vault_restore_state`'s own doc comment states, and the
            // half of KV-04 that was correct.
            // 2026-08-26: this used to be a bare `if let Ok(..)`, so a store
            // this build could not parse was skipped **in total silence** --
            // the user opened a project, every knowledge link was gone, and
            // nothing said so. Skipping is still right (never merge, never
            // clear what is in memory), but it is now reported. The narrower
            // half of the same defect is fixed at the source: `Selection`'s
            // hand-written `Deserialize` no longer fails the whole store over
            // one unrecognised selection type (`SAVEFILE_COMPAT.md` §13.3.6).
            match cartalith_vault::LinkStore::from_json(&text) {
                Ok(store) => {
                    // The links are this project's, wholesale --
                    // `WorldGen::load_save` (run a few dozen lines above)
                    // has already emptied the outgoing project's.
                    self.vault.store.links = store.links;
                    // Snapshots are this project's too, and for the same
                    // reason. Assigning `links` alone left the outgoing
                    // project's snapshots in place, so an archive that carries
                    // none inherited whatever was already loaded.
                    self.vault.store.snapshots = store.snapshots;
                    // The vault *registry* is merged, not replaced. A
                    // `VaultRef` is `{id, display_name}` and the first entry
                    // is what `vault_info()` reports as the device's bound
                    // vault; assigning the archive's list over it used to
                    // rename the user's own connection to whoever authored
                    // the project. Appending only ids this device has never
                    // seen keeps the binding intact and still lets a
                    // restored link name the vault it wants.
                    for v in store.vaults {
                        if !self.vault.store.vaults.iter().any(|e| e.id == v.id) {
                            self.vault.store.vaults.push(v);
                        }
                    }
                    restored.push("vault");
                }
                Err(e) => godot_warn!(
                    "cartalith-godot: this project's vault.json could not be read ({e}); \
                     its links were left out of the opened project rather than merged. \
                     The links already in memory are untouched."
                ),
            }
        }

        // Ruling BO: the forced lakes, after `load_save` and the substrate
        // install (each of which reset the mask with the world) so nothing
        // clears them again. The civ copy needs nothing: its forcing is
        // already in the restored `rasters/water_bodies.u8`.
        if let Some(mask) = forced_lakes_from_project(&data, n, &mut restore_warnings) {
            self.merge_forced_lakes(&mask);
            restored.push("forced lakes");
        }

        // --- the two drafts -------------------------------------------
        //
        // Restored here, not by the shell, for the reason this module's own
        // doc gives: a draft is *engine* state. Both were written from
        // 2026-08-31 and neither came back -- `load_save` a few dozen lines
        // above drops `paint`/`sculpt` outright, so the documents were
        // carried in the archive and applied to nothing. Every painted biome,
        // terrain and splat cell, and every stamp in the Sculpt stack, was
        // invisible from the moment a project was reopened.
        //
        // The grid check is not defensive padding: a paint index and a sculpt
        // stroke point are both grid-cell coordinates, so a draft captured on
        // another world's grid is not a smaller picture, it is a scrambled
        // one. `decode_sparse` would silently drop the out-of-range half and
        // leave a plausible-looking wrong result.
        let grid = (self.gw.max(0) as usize, self.gh.max(0) as usize);
        if let Some(Ok(doc)) = data.parse::<PaintDoc>(SLOT_PAINT)
            && (doc.gw, doc.gh) == grid
            && n > 0
        {
            // The land-only gate's classification, recomputed from the
            // archive's own heightmap and rainfall. This is the answer to
            // "where does a loaded project's paint editor come from at all?"
            // that `paint_bridge.rs` had no setter for: the inputs
            // `build_water_bodies` needs are exactly the two rasters
            // `SAVEFILE_COMPAT.md` §5.1 requires every archive to carry, so
            // there is nothing to invent. It costs one flood fill on open
            // (measured 417 ms at 2048x2048, `absorb`'s own note), paid only
            // when the archive actually carries a paint draft.
            let mask = self.loaded_water_mask(n);
            let mut editor = crate::paint_bridge::PaintEditor::new(doc.gw, doc.gh, mask);
            editor.restore_layers(n, &doc.biome, &doc.terrain, &doc.splat);
            self.paint = Some(editor);
            restored.push("paint layers");
        }

        if let Some(Ok(doc)) = data.parse::<SculptDoc>(SLOT_SCULPT)
            && (doc.gw, doc.gh) == grid
            && n > 0
        {
            // The water hooks are the world's own carve lock -- exactly what
            // `absorb` seeds a fresh editor with -- when the archive carried
            // the substrate (§8.3). That matters because such a world can now
            // **commit** the draft (`sculpt_commit` needs
            // `WorldSource::Generated`, which a rebuilt world is), and a
            // commit writes the editor's lock masks back over `ws.river_mask`/
            // `river_floor`: seeded with `None` it would erase the saved lock.
            //
            // Without a substrate it is `None`/`None`, the *legal* empty lock
            // state rather than a stand-in (`SculptEditor::new` falls back to
            // `WaterState::new(n)`, what a world generated with `carve_rivers`
            // off gets), and the draft comes back visible and editable but
            // cannot be committed -- that still needs the world it was drawn
            // over.
            let (mask, floor) = match self.source.as_ref() {
                Some(WorldSource::Generated(ws)) => (ws.river_mask.clone(), ws.river_floor.clone()),
                _ => (None, None),
            };
            self.sculpt = Some(crate::sculpt_bridge::SculptEditor::new(
                doc.gw,
                doc.gh,
                mask,
                floor,
                doc.seed,
            ));
            if self.sculpt_apply_doc(&doc).is_ok() {
                restored.push("sculpt draft");
            }
        }

        // A complete world gets the two draft editors `absorb` gives a
        // generated one, when the archive carried no draft for them: an empty
        // Paint editor over this world's water mask, and a Sculpt editor
        // seeded with the saved carve lock. Without them a reopened project
        // was a different session state from the world it saved -- the Paint
        // and Sculpt tools said "no editor for this world", and the LOD
        // snapshot's `paint_present` input differed, so a pyramid stored by
        // the generated session was never seeded into its reopened self
        // (`_lodseed_probe.gd` case D). A `Loaded` world keeps its long-standing
        // "no draft session" state.
        if let Some(WorldSource::Generated(ws)) = self.source.as_ref() {
            let (gw, gh) = grid;
            let seed_lock = (ws.river_mask.clone(), ws.river_floor.clone());
            if self.sculpt.is_none() && n > 0 {
                self.sculpt = Some(crate::sculpt_bridge::SculptEditor::new(
                    gw,
                    gh,
                    seed_lock.0,
                    seed_lock.1,
                    self.seed as u32,
                ));
            }
            if self.paint.is_none() && n > 0 {
                let mask = self.loaded_water_mask(n);
                self.paint = Some(crate::paint_bridge::PaintEditor::new(gw, gh, mask));
            }
        }

        // The shell's own slots are handed back rather than applied: this
        // crate has no idea what they mean, which is the whole point of the
        // channel (`SAVEFILE_COMPAT.md` §6.5).
        //
        // Verbatim text, not `serde_json::to_string` of the parsed value.
        // That re-serialization sorts object members, drops the caller's
        // whitespace and re-emits the coercion the paragraph above wants,
        // so a document handed back through it would not be the document
        // that was saved -- and this crate, not modelling it, could not tell
        // which of those edits mattered. The slot the caller wrote is the
        // slot they get back, byte for byte.
        let mut documents = VarDictionary::new();
        for slot in cartalith_io::DOCUMENT_SLOTS {
            if caller_slot_refusal(slot).is_some() {
                continue;
            }
            if let Some(text) = data.text_of(slot) {
                documents.set(*slot, text);
            }
        }

        // Owner rulings 28/29's read half: the stored LOD pyramid, if the
        // archive carried one that `read_project` did not drop on a
        // `source_key` mismatch. **Held, not trusted**: `LodWorker::tile`
        // and `request` serve a held tile only when its producer string is
        // exactly the live snapshot's -- the same world, look, colour space
        // and attachments -- and synthesise as before otherwise. Last, after
        // every restore above, though nothing here depends on the order: the
        // comparison happens when a tile is asked for, against whatever the
        // world then is. `load_save` above already cleared the previous
        // project's (`release_world`).
        let lod_tiles_held = match data.lod_tiles.take() {
            Some(lod) => {
                let n = lod.tiles.len();
                self.lod_worker.seed(crate::lod_worker::StoredPyramid::from(lod));
                n
            }
            None => 0,
        };

        let mut out = vdict! { "ok" => true, "error" => "" };
        out.set("lod_tiles_held", lod_tiles_held as i64);
        // `"complete"`, `"absent"` or `"incomplete"` -- whether this world
        // reopened with its substrate (§8.3). `"incomplete"` also put a line in
        // `warnings`; `"absent"` is every archive written before 2026-09-24
        // and is not a warning, since nothing in the file is damaged.
        out.set("substrate", substrate);
        // The same three words for the geology column (§8.4, GF-8):
        // `"complete"` restored it, `"absent"` is an archive written without
        // one (every one before 2026-10-18; not a warning), `"incomplete"` was
        // refused with a line in `warnings`.
        out.set("geology", geology);
        out.set(
            "layout",
            if data.layout == cartalith_io::Layout::Tree {
                "tree"
            } else {
                "flat"
            },
        );
        out.set("format_version", data.format_version);
        let warnings: PackedStringArray =
            data.warnings.iter().chain(restore_warnings.iter()).map(GString::from).collect();
        let foreign: PackedStringArray = self.carried_foreign.keys().map(GString::from).collect();
        let restored: PackedStringArray = restored.iter().map(|s| GString::from(*s)).collect();
        out.set("warnings", &warnings);
        out.set("foreign_entries", &foreign);
        out.set("documents", &documents);
        out.set("restored", &restored);
        out
    }

    /// One caller-owned document's JSON **text**, read out of an archive on
    /// disk **without loading the world it describes**.
    ///
    /// The other half of `project_save_with_documents`, for the case
    /// `project_open`'s `documents` does not cover: reading a project's
    /// saved measurements — or any other shell-owned payload — while the
    /// session keeps the world it already has. Opening the archive to get at
    /// one small JSON document would replace the current world as a side
    /// effect, which is not a price a "peek at another project" command
    /// should pay. (Journeys used to be this function's own example; since
    /// `STORY_PLANNING_SCOPE.md` SP-1, `entities/journeys.json` is
    /// engine-owned and this function refuses it -- see the bullet below.)
    ///
    /// Returns `{ok, error, slot, present, text}`. `present` is `false` with
    /// `ok` still `true` when the archive simply does not carry that
    /// document; `text` is then empty. **Check `present`, not `text`** — an
    /// empty document and an absent one are different answers, and only one
    /// of them means "the user has never saved any".
    ///
    /// Text, never a `Dictionary`, for the reason
    /// `project_save_with_documents`' doc comment gives at length: Godot's
    /// `JSON` has one number type and floats every integer it touches, and
    /// KV-04 is what that cost once already. The caller parses this string
    /// with `JSON.parse_string` at the moment it wants a value, and the
    /// engine never sees a `Dictionary` at all.
    ///
    /// An **engine-owned** slot is refused rather than returned,
    /// symmetrically with the writer — see [`caller_slot_refusal`]. The
    /// settlements document is not the shell's to read here;
    /// `get_settlements()` is.
    ///
    /// # No shell caller, deliberately — `PARITY_AUDIT.md` §23, 2026-08-26
    ///
    /// The wiring audit flags this as a `#[func]` no `.gd` file names, and it
    /// is correct that nothing calls it. That was examined and **declined**,
    /// so a later pass does not re-flag it as an oversight:
    ///
    /// * The obvious consumer would be showing something about a project
    ///   *before* opening it. The one surface that does that is
    ///   `open_project_dialog.gd`'s gallery tile, and what it shows — the
    ///   seed — comes out of `params.json`, an **engine-owned** slot this
    ///   function refuses by design. Widening the refusal to feed a caption
    ///   would break the writer's symmetry for a readout the dialog already
    ///   has by its own `ZIPReader`.
    /// * The other candidate used to be the Journey Planner loading saved
    ///   journeys without replacing the world. Since SP-1,
    ///   `entities/journeys.json` is **engine-owned**
    ///   (`ENGINE_OWNED_SLOTS`), so this function refuses it outright the
    ///   same as `params.json` above -- `journey_list()`/`journey_get()` are
    ///   the read path now, and both need a world open already (they read
    ///   `WorldGen::infra`), which is the same "no shell caller for the
    ///   read-without-opening case" shape this whole doc comment describes.
    ///
    /// So this is a public API with no shell caller **yet**, not a gap: the
    /// capability `project_open` structurally cannot offer (read one document,
    /// keep the current world), kept ready for the first command that wants
    /// it. Inventing a control to justify it would be the opposite of what
    /// the audit is for.
    #[func]
    fn project_read_document(&self, path: GString, slot: GString) -> VarDictionary {
        let slot = slot.to_string();
        let mut out = vdict! { "ok" => false, "error" => "" };
        out.set("slot", slot.as_str());
        out.set("present", false);
        out.set("text", "");

        if let Some(refusal) = caller_slot_refusal(&slot) {
            out.set("error", refusal);
            return out;
        }
        let file = match std::fs::File::open(path.to_string()) {
            Ok(f) => f,
            Err(e) => {
                out.set("error", format!("could not open {path}: {e}"));
                return out;
            }
        };
        match project::read_document(std::io::BufReader::new(file), &slot) {
            Ok(Some(text)) => {
                out.set("ok", true);
                out.set("present", true);
                out.set("text", text);
            }
            // Absent is a successful answer, not a failure: a project saved
            // before the shell ever wrote this slot is a normal project.
            Ok(None) => out.set("ok", true),
            Err(e) => out.set("error", e.to_string()),
        }
        out
    }

    /// Every document slot the project format defines, in the order
    /// `SAVEFILE_COMPAT.md` §5 lists them. Lets a caller validate a slot
    /// name before handing it to `project_save_with_documents` rather than
    /// discovering the typo as a failed save.
    #[func]
    fn project_document_slots(&self) -> PackedStringArray {
        cartalith_io::DOCUMENT_SLOTS
            .iter()
            .map(|s| GString::from(*s))
            .collect()
    }

    /// The slots this engine writes and reads itself — the subset of
    /// `project_document_slots()` a caller must **not** supply.
    #[func]
    fn project_engine_owned_slots(&self) -> PackedStringArray {
        ENGINE_OWNED_SLOTS
            .iter()
            .map(|s| GString::from(*s))
            .collect()
    }

    /// The `format_version` this build writes (`SAVEFILE_COMPAT.md` §4).
    #[func]
    fn project_format_version(&self) -> i64 {
        cartalith_io::PROJECT_FORMAT_VERSION
    }
}


// ===================== the four caller-owned document schemas =====================
//
// Same rules as the engine-owned schemas above: a DTO rather than a derive
// on the live type, and `#[serde(default)]` on every member so a document
// from another implementation loses only what it actually omitted.

/// `drafts/paint.json` -- `state.cartoPaint`'s own persistence shape.
///
/// Each layer is [`cartalith_spatial::PaintLayer::encode_sparse`]'s flat
/// `[index, value, index, value, ...]` pair list, which is the reference's
/// `_paintSyncToState` encoding verbatim rather than a shape invented here.
/// `gw`/`gh` are carried because an index means nothing without them: a
/// layer decoded against a different grid is not a smaller picture, it is a
/// scrambled one.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct PaintDoc {
    #[serde(default)]
    gw: usize,
    #[serde(default)]
    gh: usize,
    #[serde(default)]
    biome: Vec<u32>,
    #[serde(default)]
    terrain: Vec<u32>,
    #[serde(default)]
    splat: Vec<u32>,
}

/// `drafts/sculpt.json` -- the uncommitted Sculpt draft plus the editor
/// state that decides what the *next* stroke will be.
///
/// A stamp is stored as its recipe (feature key, seed, stroke points, the
/// shared globals and that feature's own controls), not as a height delta:
/// that is what `PassBuffer` holds, and it is the whole reason a draft is
/// non-destructive. `hidden` travels with it because hiding a stamp is an
/// edit to the draft, not a view setting.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct SculptDoc {
    #[serde(default)]
    gw: usize,
    #[serde(default)]
    gh: usize,
    /// The seed the *next* stroke will capture, not any stamp's own.
    #[serde(default)]
    seed: u32,
    #[serde(default)]
    feature: String,
    #[serde(default)]
    globals: BTreeMap<String, f64>,
    #[serde(default)]
    params: BTreeMap<String, f64>,
    /// Freehand's one non-numeric control, which is why it is not in
    /// `params` (`sculpt_bridge::feature_param_pairs`' own note). Empty for
    /// every other feature.
    #[serde(default)]
    sub_mode: String,
    #[serde(default)]
    stamps: Vec<SculptStampDto>,
}

/// One stamp inside `SculptDoc::stamps`: the recipe `PassBuffer` holds for a
/// single stroke (feature key, its seed and sea level, the stroke points, the
/// globals and per-feature controls it was captured with, and its hidden flag).
/// Never a height delta; the stamp is replayed on restore.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct SculptStampDto {
    #[serde(default)]
    feature: String,
    #[serde(default)]
    seed: u32,
    #[serde(default)]
    sea_level: f64,
    #[serde(default)]
    hidden: bool,
    #[serde(default)]
    points: Vec<[f64; 2]>,
    #[serde(default)]
    globals: BTreeMap<String, f64>,
    #[serde(default)]
    params: BTreeMap<String, f64>,
    #[serde(default)]
    sub_mode: String,
    /// ET-D3: the stroke's per-point pointer pressure, one value per `points`
    /// entry, or **absent** for a stamp whose device reported none.
    ///
    /// Omitted from the file when `None` (`skip_serializing_if`), so a draft with
    /// no pressure -- every draft written before ET-D3 -- re-serialises
    /// byte-identically (the `biome_cols_member` precedent), and an old file
    /// without the member deserialises to `None` (`default`). Never `[]` or a
    /// run of `1.0` for "no pressure".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pressure: Option<Vec<f32>>,
}

/// One draft stamp as its `drafts/sculpt.json` record: the writer half of
/// [`sculpt_stamp_from_dto`], shared by `sculpt_document_json` and the tests so
/// the round trip is exercised through the code the save path really runs.
fn sculpt_stamp_to_dto(stamp: &cartalith_terrain::sculpt::SculptStamp, hidden: bool) -> SculptStampDto {
    SculptStampDto {
        feature: stamp.feature().meta().key.to_string(),
        seed: stamp.seed,
        sea_level: stamp.sea_level,
        hidden,
        points: stamp.points.iter().map(|p| [p.x, p.y]).collect(),
        globals: sculpt_globals_to_map(&stamp.globals),
        params: sculpt_params_to_map(&stamp.params),
        sub_mode: sculpt_sub_mode_of(&stamp.params),
        // ET-D3: `None` stays `None` (and is then omitted from the file).
        pressure: stamp.pressure.clone(),
    }
}

/// The reader half: a record back into a stamp, or `None` for a feature key this
/// build does not have (dropped, not defaulted -- see `sculpt_apply_doc`).
///
/// ET-D3: the pressure goes through `SculptStamp::with_pressure`, so a
/// hand-edited or truncated vector (wrong length, a non-finite value) restores
/// as **absent**, never repaired or guessed.
fn sculpt_stamp_from_dto(d: &SculptStampDto) -> Option<cartalith_terrain::sculpt::SculptStamp> {
    let f = cartalith_terrain::sculpt::Feature::from_key(&d.feature)?;
    let stamp = cartalith_terrain::sculpt::SculptStamp {
        seed: d.seed,
        points: d
            .points
            .iter()
            .map(|p| cartalith_terrain::sculpt::Point::new(p[0], p[1]))
            .collect(),
        globals: sculpt_globals_from_map(&d.globals),
        params: sculpt_params_from_map(f, &d.params, &d.sub_mode),
        sea_level: d.sea_level,
        pressure: None,
    };
    Some(match &d.pressure {
        Some(p) => stamp.with_pressure(p.clone()),
        None => stamp,
    })
}

/// `library/travel.json` -- every **custom** Travel Library entry.
///
/// Stock entries are deliberately absent: `EntrySet::get_mut` refuses them,
/// so they are read-only by construction and `TravelLibrary::new()` rebuilds
/// exactly the same four sets on every launch. Storing them would be storing
/// this build's own constants and then having to decide what to do when the
/// next build changes one.
///
/// `fields` is `travel_bridge`'s own `_to_pairs`/`_apply_pairs` flattening --
/// the same field map `tl_get`/`tl_edit` already speak, rather than a second
/// wire vocabulary for the same four types.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct TravelDoc {
    #[serde(default)]
    animals: Vec<TravelEntryDto>,
    #[serde(default)]
    vehicles: Vec<TravelEntryDto>,
    #[serde(default)]
    vessels: Vec<TravelEntryDto>,
    #[serde(default)]
    presets: Vec<TravelEntryDto>,
}

/// One custom Travel Library entry as stored in `library/travel.json`: its
/// stable `id`, the animal-only `species` key, and the flattened `fields` map
/// (see `travel_pairs_to_fields`). Every member defaults so a partial row loads.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct TravelEntryDto {
    #[serde(default)]
    id: String,
    /// `AnimalDef::species_key` -- which of `JP_ANIMAL_KEYS` this entry
    /// stands in for, `""` for none. Carried explicitly because it is the
    /// one field `animal_apply_pairs` deliberately cannot set (its own doc:
    /// `id`/`origin`/`species_key` "can never be overwritten by a client
    /// dictionary"), and losing it would silently stop a custom pack animal
    /// from affecting a computed journey at all. Empty on the other three
    /// types, which have no such field.
    #[serde(default)]
    species: String,
    #[serde(default)]
    fields: BTreeMap<String, serde_json::Value>,
}

// ---- the flattening both travel halves share ----

/// Encodes one `JpValue` as JSON for the travel document, keeping the Int/Num
/// split. A non-finite `Num` becomes `null` (reads back as an absent field),
/// never `0`.
fn jp_value_to_json(v: &journey_bridge::JpValue) -> serde_json::Value {
    match v {
        journey_bridge::JpValue::Int(i) => serde_json::Value::from(*i),
        // A non-finite never reaches here (every travel validator and every
        // `set_*` above rejects one), but `Number::from_f64` has to be
        // answered anyway; `null` is the honest answer and reads back as
        // "field absent", not as `0`.
        journey_bridge::JpValue::Num(n) => serde_json::Number::from_f64(*n)
            .map_or(serde_json::Value::Null, serde_json::Value::Number),
        journey_bridge::JpValue::Str(s) => serde_json::Value::String(s.clone()),
        journey_bridge::JpValue::Bool(b) => serde_json::Value::Bool(*b),
    }
}

/// The exact inverse of [`jp_value_to_json`] for the four shapes it emits.
/// An integral JSON number comes back as `Int` and a fractional one as
/// `Num`, which is the same split `serde_json` itself made on the way in --
/// and the reason this boundary carries text rather than a `Dictionary`
/// (`project_save_with_documents`' own doc comment: Godot's `JSON` has one
/// number type and floats every integer it touches).
fn json_to_jp_value(v: &serde_json::Value) -> Option<journey_bridge::JpValue> {
    match v {
        serde_json::Value::Bool(b) => Some(journey_bridge::JpValue::Bool(*b)),
        serde_json::Value::String(s) => Some(journey_bridge::JpValue::Str(s.clone())),
        serde_json::Value::Number(n) => match n.as_i64() {
            Some(i) => Some(journey_bridge::JpValue::Int(i)),
            None => n.as_f64().map(journey_bridge::JpValue::Num),
        },
        _ => None,
    }
}

/// Flattens `travel_bridge`'s `(key, JpValue)` pairs into the document's
/// `fields` map via [`jp_value_to_json`]. Key order becomes `BTreeMap` order.
fn travel_pairs_to_fields(
    pairs: Vec<(String, journey_bridge::JpValue)>,
) -> BTreeMap<String, serde_json::Value> {
    pairs.into_iter().map(|(k, v)| (k, jp_value_to_json(&v))).collect()
}

/// Inverse of [`travel_pairs_to_fields`]: a field whose JSON is not a bool,
/// string or number (null, array, object) is dropped rather than guessed at.
fn travel_fields_to_pairs(
    fields: &BTreeMap<String, serde_json::Value>,
) -> Vec<(String, journey_bridge::JpValue)> {
    fields
        .iter()
        .filter_map(|(k, v)| json_to_jp_value(v).map(|jv| (k.clone(), jv)))
        .collect()
}

// ---- the flattening both sculpt halves share ----

/// Reads every global sculpt control into a `key -> value` map for the draft
/// document, one entry per control `global_controls()` lists.
fn sculpt_globals_to_map(g: &cartalith_terrain::sculpt::SculptGlobals) -> BTreeMap<String, f64> {
    crate::sculpt_bridge::global_controls()
        .into_iter()
        .filter_map(|c| crate::sculpt_bridge::get_global(g, c.key).map(|v| (c.key.to_string(), v)))
        .collect()
}

/// Every recognised key applied over `SculptGlobals::default()`, through
/// `set_global` so the range clamp and the `octaves` rounding are the same
/// ones a live edit goes through. An unknown key is dropped rather than
/// failing the document -- SAVEFILE_COMPAT.md 14.3's unknown-member rule.
fn sculpt_globals_from_map(m: &BTreeMap<String, f64>) -> cartalith_terrain::sculpt::SculptGlobals {
    let mut g = cartalith_terrain::sculpt::SculptGlobals::default();
    for (k, v) in m {
        let _ = crate::sculpt_bridge::set_global(&mut g, k, *v);
    }
    g
}

/// The numeric per-feature controls of `p` as a `key -> value` map; Freehand's
/// non-numeric sub-mode is carried separately by [`sculpt_sub_mode_of`].
fn sculpt_params_to_map(p: &cartalith_terrain::sculpt::FeatureParams) -> BTreeMap<String, f64> {
    crate::sculpt_bridge::feature_param_pairs(p)
        .into_iter()
        .map(|(k, v)| (k.to_string(), v))
        .collect()
}

/// Freehand's sub-mode key, or `""` for every other feature (the document
/// reads `""` as "no sub-mode", and [`sculpt_params_from_map`] ignores it then).
fn sculpt_sub_mode_of(p: &cartalith_terrain::sculpt::FeatureParams) -> String {
    match p {
        cartalith_terrain::sculpt::FeatureParams::Freehand { sub_mode, .. } => {
            sub_mode.key().to_string()
        }
        _ => String::new(),
    }
}

/// `feature`'s own defaults with every recognised key applied over them,
/// through `set_feature_param` for the same reason
/// [`sculpt_globals_from_map`] uses `set_global`. A key belonging to a
/// *different* feature is rejected there and dropped here.
fn sculpt_params_from_map(
    feature: cartalith_terrain::sculpt::Feature,
    m: &BTreeMap<String, f64>,
    sub_mode: &str,
) -> cartalith_terrain::sculpt::FeatureParams {
    let mut p = feature.default_params();
    for (k, v) in m {
        let _ = crate::sculpt_bridge::set_feature_param(&mut p, feature, k, *v);
    }
    if let cartalith_terrain::sculpt::FeatureParams::Freehand { sub_mode: sm, .. } = &mut p
        && let Some(mode) = cartalith_terrain::sculpt::FreehandMode::from_key(sub_mode)
    {
        *sm = mode;
    }
    p
}

#[godot_api(secondary)]
impl WorldGen {
    /// Every document this engine can build for a caller-owned slot, keyed
    /// by slot name and ready to hand straight to
    /// `project_save_with_documents` -- the four `*_document_json()`
    /// functions below, called in one pass, with a slot that returned `""`
    /// left out entirely.
    ///
    /// **This is the call a Save command should make.** The four getters
    /// below hand back text with no slot attached, which means the shell
    /// types the slot name itself -- and `project_document_slots()` exists
    /// precisely because a typo there is discovered "as a failed save".
    /// Naming the slots here, next to the constants that are also what
    /// `cartalith-io` registered, removes that step rather than documenting
    /// around it.
    ///
    /// Merge it into whatever the shell owns (`annotations/measurements.json`
    /// as of 2026-09-03; `entities/journeys.json` was the other one until
    /// `STORY_PLANNING_SCOPE.md` SP-1 moved it to `ENGINE_OWNED_SLOTS`) and
    /// pass the union; the sets never collide, since none of these four is a
    /// slot GDScript writes.
    #[func]
    fn project_engine_built_documents(&mut self) -> VarDictionary {
        let mut out = VarDictionary::new();
        for (slot, text) in [
            (SLOT_PAINT, self.paint_document_json()),
            (SLOT_SCULPT, self.sculpt_document_json()),
            (SLOT_ASSETS, self.asset_library_document_json()),
            (SLOT_TRAVEL, self.travel_library_document_json()),
        ] {
            if !text.is_empty() {
                out.set(slot, &text);
            }
        }
        out
    }

    /// `drafts/paint.json`'s text for the current session, or `""` when
    /// there is nothing to write (no paint editor, or all three layers still
    /// unallocated).
    ///
    /// **Check for the empty string, not for an empty document.** `""` means
    /// "do not put this slot in the archive at all", which is what a project
    /// with no painting should carry -- an empty `drafts/paint.json` and an
    /// absent one look the same to a reader but differ to anyone diffing two
    /// saves.
    ///
    /// Hand the result to `project_save_with_documents` under
    /// `"drafts/paint.json"`. [`Self::paint_restore_document`] is the other
    /// half; `project_open` applies the archive's own copy without being
    /// asked.
    #[func]
    fn paint_document_json(&self) -> GString {
        let Some(paint) = self.paint.as_ref() else {
            return GString::new();
        };
        // `PaintLayer::encode_sparse`'s own `[index, value, ...]` shape, run
        // off `layer_cells` because that is the only public view of a
        // committed layer this boundary has.
        let layer = |t: crate::paint_bridge::PaintTarget| -> Vec<u32> {
            paint
                .layer_cells(t)
                .map(|cells| {
                    let mut out = Vec::new();
                    for (i, &v) in cells.iter().enumerate() {
                        if v != 0 {
                            out.push(i as u32);
                            out.push(u32::from(v));
                        }
                    }
                    out
                })
                .unwrap_or_default()
        };
        let doc = PaintDoc {
            gw: self.gw.max(0) as usize,
            gh: self.gh.max(0) as usize,
            biome: layer(crate::paint_bridge::PaintTarget::Biome),
            terrain: layer(crate::paint_bridge::PaintTarget::Terrain),
            splat: layer(crate::paint_bridge::PaintTarget::Splat),
        };
        if doc.biome.is_empty() && doc.terrain.is_empty() && doc.splat.is_empty() {
            return GString::new();
        }
        serde_json::to_string(&doc).map_or_else(|_| GString::new(), |t| GString::from(&t))
    }

    /// Restores a `drafts/paint.json` into the **live** Paint editor.
    /// Returns `{ok, error, biome, terrain, splat}` -- the painted-cell count
    /// each layer came back with.
    ///
    /// The exact mirror of [`Self::sculpt_restore_document`], including which
    /// caller it is for: `project_open` applies the archive's own copy
    /// itself, and this is the *import* route for a document read out of
    /// another file. Refused on a grid mismatch for the reason
    /// `PaintEditor::restore_layers` gives -- a paint index is a cell number.
    #[func]
    fn paint_restore_document(&mut self, text: GString) -> VarDictionary {
        let doc: PaintDoc = match serde_json::from_str(&text.to_string()) {
            Ok(d) => d,
            Err(e) => return err(format!("drafts/paint.json is not valid: {e}")),
        };
        let (gw, gh) = (self.gw.max(0) as usize, self.gh.max(0) as usize);
        if doc.gw != gw || doc.gh != gh {
            return err(format!(
                "these layers were painted on a {}x{} grid and this world is {gw}x{gh}; \
                 a paint index is a cell number and does not carry over",
                doc.gw, doc.gh
            ));
        }
        let Some(p) = self.paint.as_mut() else {
            return err(
                "there is no Paint editor to restore into: opening a project builds one only \
                 when the archive carries a drafts/paint.json for this world's own grid, and \
                 generating a world builds one always",
            );
        };
        let (biome, terrain, splat) = p.restore_layers(gw * gh, &doc.biome, &doc.terrain, &doc.splat);
        let mut out = vdict! { "ok" => true, "error" => "" };
        out.set("biome", biome as i64);
        out.set("terrain", terrain as i64);
        out.set("splat", splat as i64);
        out
    }

    /// `drafts/sculpt.json`'s text for the current session, or `""` when
    /// there is no Sculpt editor and nothing to write. Same empty-string
    /// contract as [`Self::paint_document_json`].
    ///
    /// A draft with **no** stamps still writes a document when the editor
    /// exists: the armed feature, its controls and the next stroke's seed
    /// are real authoring state, and losing them across a save is the same
    /// kind of loss as losing the stack.
    #[func]
    fn sculpt_document_json(&self) -> GString {
        let Some(s) = self.sculpt.as_ref() else {
            return GString::new();
        };
        let doc = SculptDoc {
            gw: self.gw.max(0) as usize,
            gh: self.gh.max(0) as usize,
            seed: s.seed,
            feature: s.feature.meta().key.to_string(),
            globals: sculpt_globals_to_map(&s.globals),
            params: sculpt_params_to_map(&s.params),
            sub_mode: sculpt_sub_mode_of(&s.params),
            stamps: s
                .draft
                .entries()
                .iter()
                .map(|e| sculpt_stamp_to_dto(&e.stamp, e.hidden))
                .collect(),
        };
        serde_json::to_string(&doc).map_or_else(|_| GString::new(), |t| GString::from(&t))
    }

    /// Restores a `drafts/sculpt.json` into the **live** Sculpt editor.
    ///
    /// Returns `{ok, error, stamps}`. Refused, rather than silently partly
    /// applied, when there is no Sculpt editor or when the document's grid is
    /// not this world's: a stroke point is a grid-cell coordinate, so a draft
    /// from a 512x384 world laid over a 256x192 one is not a smaller draft,
    /// it is the wrong one.
    ///
    /// **Not the call the load path makes.** `project_open` restores the
    /// archive's own draft itself, building the editor to hold it; this is
    /// the *import* route -- a draft read out of some other file with
    /// `project_read_document` and applied to the world already on screen.
    ///
    /// **Replaces the draft**, it does not merge into it, and it clears the
    /// undo/redo history with it -- `PassBuffer::discard` is what a restore
    /// starts from, so "undo the restore" is not offered rather than offered
    /// and wrong.
    #[func]
    fn sculpt_restore_document(&mut self, text: GString) -> VarDictionary {
        let doc: SculptDoc = match serde_json::from_str(&text.to_string()) {
            Ok(d) => d,
            Err(e) => return err(format!("drafts/sculpt.json is not valid: {e}")),
        };
        match self.sculpt_apply_doc(&doc) {
            Ok(restored) => {
                let mut out = vdict! { "ok" => true, "error" => "" };
                out.set("stamps", restored as i64);
                out
            }
            Err(e) => err(e),
        }
    }

    /// The land-only gate's classification for the world currently in
    /// `self.source` -- `PaintEditor::new`'s third argument, computed from
    /// whichever pair of rasters this world has.
    ///
    /// `0` is land, and every non-`0` is water, so a length that is not `n`
    /// would degrade the gate to "never excludes anything"
    /// (`PaintEditor::new`'s own contract). That is why the fallback is an
    /// all-land mask of the right length rather than an empty one: a caller
    /// with no world has nothing to gate against, and the honest gate is the
    /// one that lets every dab through rather than one that silently
    /// mis-sizes.
    fn loaded_water_mask(&self, n: usize) -> std::sync::Arc<[u8]> {
        // `absorb()`'s own source first: the civ layer's classification, which
        // a reopened project restores from `rasters/water_bodies.u8`.
        if let Some(civ) = self.civ.as_ref().filter(|c| c.water_bodies.len() == n) {
            return std::sync::Arc::from(civ.water_bodies.as_slice());
        }
        let (gw, gh) = (self.gw.max(0) as usize, self.gh.max(0) as usize);
        // Otherwise classified from the terrain -- from either source: a
        // project reopened with its substrate is `Generated` (§8.3), and this
        // used to answer all-land for it because only `Loaded` was matched.
        let (field, rain): (&[f32], &[f32]) = match self.source.as_ref() {
            Some(WorldSource::Loaded(save)) => (&save.fields.heightmap, &save.fields.rainfall),
            Some(WorldSource::Generated(ws)) => (&ws.field, &ws.rainfall),
            // No world: nothing to classify.
            None => return std::sync::Arc::from(vec![0u8; n].as_slice()),
        };
        // Ruling CC's floor, as the drawn map applies it; no mask, as before.
        let wb = crate::render::water_bodies_floored(field, rain, gw, gh, self.sea_level, self.world, None);
        std::sync::Arc::from(wb.classification.as_slice())
    }

    /// [`Self::sculpt_restore_document`]'s body, shared with
    /// [`Self::project_open`], which parses the same document out of the
    /// archive and must not re-serialize it to text to get at this.
    /// `Ok(n)` is the number of stamps that rebuilt.
    fn sculpt_apply_doc(&mut self, doc: &SculptDoc) -> Result<usize, String> {
        let (gw, gh) = (self.gw.max(0) as usize, self.gh.max(0) as usize);
        if doc.gw != gw || doc.gh != gh {
            return Err(format!(
                "this draft was captured on a {}x{} grid and this world is {gw}x{gh}; \
                 a stroke point is a grid-cell coordinate and does not carry over",
                doc.gw, doc.gh
            ));
        }
        let Some(s) = self.sculpt.as_mut() else {
            return Err(
                "there is no Sculpt editor to restore into: opening a project builds one \
                 only when the archive carries a drafts/sculpt.json for this world's own \
                 grid, and generating a world builds one always"
                    .to_string(),
            );
        };
        if let Some(f) = cartalith_terrain::sculpt::Feature::from_key(&doc.feature) {
            s.feature = f;
            s.params = sculpt_params_from_map(f, &doc.params, &doc.sub_mode);
        }
        s.globals = sculpt_globals_from_map(&doc.globals);
        s.seed = doc.seed;
        s.points.clear();
        s.selection.clear();
        s.draft.discard();
        let mut restored = 0usize;
        for d in &doc.stamps {
            // A stamp naming a feature this build does not have is dropped,
            // not defaulted: `Mountains` where the file said something else
            // would be a stroke the user never drew.
            let Some(stamp) = sculpt_stamp_from_dto(d) else {
                continue;
            };
            let index = s.draft.push(stamp);
            if d.hidden {
                s.draft.set_hidden(index, true);
            }
            restored += 1;
        }
        Ok(restored)
    }

    /// `library/assets.json`'s text -- `cartalith_assets::AssetDB::to_library_json`,
    /// the reference's own `window._alExportEntries` record. `""` for an
    /// empty library, matching that builder's own `None` (the reference's
    /// `if(AssetDB.totalItems()===0) return null`).
    ///
    /// **Item pixels are not in it, and cannot be.** The record carries each
    /// item's `img` index, name and transform; the bytes those indices point
    /// at live at `assetlib/img/<idx>.png`, and `cartalith-io`'s project
    /// writer has no channel for a binary entry that is not a registered
    /// raster. Restoring therefore rebuilds pack info, collections, custom
    /// slots and every slot's metadata and scatter rules, and restores no
    /// items -- which is exactly what
    /// `AssetDB::apply_library_file_with_items` does when handed no bytes,
    /// rather than something this boundary invented.
    ///
    /// `&mut self`: `to_library_json` lazily attaches a scatterable slot's
    /// preset the first time its rule is read, and the reference does the
    /// same for every scatterable slot on every export. That real (if
    /// surprising) side effect is reproduced rather than hidden behind a
    /// `&self` -- see that function's own doc comment.
    #[func]
    fn asset_library_document_json(&mut self) -> GString {
        match self.asset_library.db.to_library_json() {
            Some(file) => {
                serde_json::to_string(&file).map_or_else(|_| GString::new(), |t| GString::from(&t))
            }
            None => GString::new(),
        }
    }

    /// Restores a `library/assets.json`. Returns `{ok, error, slots, items}`.
    ///
    /// `items` is the count `apply_library_file_with_items` actually restored
    /// and is `0` today for the reason [`Self::asset_library_document_json`]
    /// gives at length -- reported rather than omitted, so a caller can see
    /// that a library came back without its pixels instead of inferring it.
    ///
    /// Goes through `parse_library_json`, so an unresolvable record (an
    /// unknown family, or an id outside a frozen family's vocabulary) is
    /// dropped exactly as the reference's own `if(!uid) continue` drops it,
    /// and a scatter rule is normalised on load the same way.
    #[func]
    fn asset_library_restore_document(&mut self, text: GString) -> VarDictionary {
        let raw = text.to_string();
        let file = match cartalith_assets::parse_library_json(raw.as_bytes()) {
            Ok(f) => f,
            Err(e) => return err(e),
        };
        // Drops the decoded pixels along with the items `apply_library_file`
        // is about to clear. `AssetDB::clear` alone would leave the session's
        // parallel image store holding variants for items that no longer
        // exist.
        self.asset_library.clear();
        let items = self
            .asset_library
            .db
            .apply_library_file_with_items(&file, &std::collections::HashMap::new());
        let mut out = vdict! { "ok" => true, "error" => "" };
        out.set("slots", file.slots.len() as i64);
        out.set("items", items as i64);
        out
    }

    /// `library/travel.json`'s text -- every custom animal, vehicle, vessel
    /// and party preset. `""` when the library is still stock-only, which is
    /// what a project nobody has authored travel content in should carry.
    #[func]
    fn travel_library_document_json(&self) -> GString {
        use cartalith_civ::travel_library::EntryOrigin;
        let lib = &self.travel_library;
        let doc = TravelDoc {
            animals: lib
                .animals
                .iter()
                .filter(|a| a.origin == EntryOrigin::Custom)
                .map(|a| TravelEntryDto {
                    id: a.id.clone(),
                    species: a.species_key.unwrap_or_default().to_string(),
                    fields: travel_pairs_to_fields(crate::travel_bridge::animal_to_pairs(a)),
                })
                .collect(),
            vehicles: lib
                .vehicles
                .iter()
                .filter(|v| v.origin == EntryOrigin::Custom)
                .map(|v| TravelEntryDto {
                    id: v.id.clone(),
                    species: String::new(),
                    fields: travel_pairs_to_fields(crate::travel_bridge::vehicle_to_pairs(v)),
                })
                .collect(),
            vessels: lib
                .vessels
                .iter()
                .filter(|v| v.origin == EntryOrigin::Custom)
                .map(|v| TravelEntryDto {
                    id: v.id.clone(),
                    species: String::new(),
                    fields: travel_pairs_to_fields(crate::travel_bridge::vessel_to_pairs(v)),
                })
                .collect(),
            presets: lib
                .presets
                .iter()
                .filter(|p| p.origin == EntryOrigin::Custom)
                .map(|p| TravelEntryDto {
                    id: p.id.clone(),
                    species: String::new(),
                    fields: travel_pairs_to_fields(crate::travel_bridge::preset_to_pairs(p)),
                })
                .collect(),
        };
        if doc.animals.is_empty()
            && doc.vehicles.is_empty()
            && doc.vessels.is_empty()
            && doc.presets.is_empty()
        {
            return GString::new();
        }
        serde_json::to_string(&doc).map_or_else(|_| GString::new(), |t| GString::from(&t))
    }

    /// Restores a `library/travel.json`. Returns `{ok, error, restored,
    /// rejected}` -- `rejected` being every field key the document carried
    /// that this build's own `*_apply_pairs` did not recognise, this
    /// codebase's usual "a typo'd key is a bug worth seeing" contract.
    ///
    /// **Replaces the custom half and only the custom half.** Every set is
    /// reset to stock first (`TRAVEL_LIBRARY_SPEC.md`'s own "Reset to stock
    /// definitions"), then the document's entries are added on top, so
    /// opening a project cannot leave another project's pack mule behind.
    /// Stock entries are untouched throughout -- they are read-only by
    /// construction and the document never carried them.
    #[func]
    fn travel_library_restore_document(&mut self, text: GString) -> VarDictionary {
        use cartalith_civ::travel_library::{AnimalDef, PartyPreset, VehicleDef, VesselDef};
        let doc: TravelDoc = match serde_json::from_str(&text.to_string()) {
            Ok(d) => d,
            Err(e) => return err(format!("library/travel.json is not valid: {e}")),
        };
        let lib = &mut self.travel_library;
        lib.animals.reset_to_stock();
        lib.vehicles.reset_to_stock();
        lib.vessels.reset_to_stock();
        lib.presets.reset_to_stock();

        let mut restored = 0i64;
        let mut rejected = PackedStringArray::new();
        // `blank(id, "")` then the field map on top: `name` is one of the
        // pairs, so the placeholder is always overwritten by the document's
        // own value and never survives into the library.
        for d in &doc.animals {
            if lib.animals.add(AnimalDef::blank(d.id.clone(), String::new())).is_none() {
                continue;
            }
            let Some(base) = lib.animals.get(&d.id).cloned() else {
                continue;
            };
            let (updated, bad) = crate::travel_bridge::animal_apply_pairs(
                &base,
                &travel_fields_to_pairs(&d.fields),
            );
            if let Some(slot) = lib.animals.get_mut(&d.id) {
                *slot = updated;
                // The one field `animal_apply_pairs` deliberately cannot
                // touch -- see `TravelEntryDto::species`.
                slot.species_key = cartalith_civ::JP_ANIMAL_KEYS
                    .iter()
                    .find(|k| **k == d.species)
                    .copied();
                restored += 1;
            }
            for k in bad {
                rejected.push(&GString::from(&k));
            }
        }
        for d in &doc.vehicles {
            if lib.vehicles.add(VehicleDef::blank(d.id.clone(), String::new())).is_none() {
                continue;
            }
            let Some(base) = lib.vehicles.get(&d.id).cloned() else {
                continue;
            };
            let (updated, bad) = crate::travel_bridge::vehicle_apply_pairs(
                &base,
                &travel_fields_to_pairs(&d.fields),
            );
            if let Some(slot) = lib.vehicles.get_mut(&d.id) {
                *slot = updated;
                restored += 1;
            }
            for k in bad {
                rejected.push(&GString::from(&k));
            }
        }
        for d in &doc.vessels {
            if lib.vessels.add(VesselDef::blank(d.id.clone(), String::new())).is_none() {
                continue;
            }
            let Some(base) = lib.vessels.get(&d.id).cloned() else {
                continue;
            };
            let (updated, bad) = crate::travel_bridge::vessel_apply_pairs(
                &base,
                &travel_fields_to_pairs(&d.fields),
            );
            if let Some(slot) = lib.vessels.get_mut(&d.id) {
                *slot = updated;
                restored += 1;
            }
            for k in bad {
                rejected.push(&GString::from(&k));
            }
        }
        for d in &doc.presets {
            if lib.presets.add(PartyPreset::blank(d.id.clone(), String::new())).is_none() {
                continue;
            }
            let Some(base) = lib.presets.get(&d.id).cloned() else {
                continue;
            };
            let (updated, bad) = crate::travel_bridge::preset_apply_pairs(
                &base,
                &travel_fields_to_pairs(&d.fields),
            );
            if let Some(slot) = lib.presets.get_mut(&d.id) {
                *slot = updated;
                restored += 1;
            }
            for k in bad {
                rejected.push(&GString::from(&k));
            }
        }
        // Advance the shared custom-id counter clear of everything just
        // reinstated, so the next "New blank definition" cannot be handed an
        // id a restored entry already holds. `TravelLibrary::next_id` is
        // private and `fresh_id()` is the only thing that moves it, so this
        // draws ids until one lands free and discards it -- ids only ever
        // have to be unique, and gaps in them are already normal (deleting a
        // custom entry leaves one).
        loop {
            let id = lib.fresh_id();
            if lib.animals.get(&id).is_none()
                && lib.vehicles.get(&id).is_none()
                && lib.vessels.get(&id).is_none()
                && lib.presets.get(&id).is_none()
            {
                break;
            }
        }
        let mut out = vdict! { "ok" => true, "error" => "" };
        out.set("restored", restored);
        out.set("rejected", &rejected);
        out
    }
}

/// Round-trip tests for the civ-layer, journey and conflict documents, run
/// through real in-memory `.ctl` archives (no Godot runtime needed).
#[cfg(test)]
mod tests {
    use super::*;

    /// A hand-built `CivData` fixture touching every persisted civ member:
    /// two settlements (one with `PlaceExtras`, one a village), a way, a sea
    /// route, territory/provinces, a continent, trade balances and a
    /// one-snapshot timeline, on a 4x3 grid (12 cells).
    fn sample_civ() -> CivData {
        let mut place_extras = std::collections::HashMap::new();
        place_extras.insert(
            7,
            civ_roster_bridge::PlaceExtras {
                specialisation: "port".into(),
                traits: vec!["walled".into(), "university".into()],
                history: "Founded after the second flood.".into(),
                age: Some(320),
                walls: Some(true),
                culture: "venus".into(),
                rules_preset: "market_town".into(),
                variant: 3,
            },
        );
        let mut village_tids = std::collections::HashSet::new();
        village_tids.insert(8u64);

        CivData {
            road_edges: Vec::new(),
            belief: Vec::new(),
            belief_seed_key: Vec::new(),
            territory_year: None,
            settlements: vec![
                cartalith_civ::NamedSettlement {
                    tid: 7,
                    placement: cartalith_civ::SettlementPlacement {
                        x: 4,
                        y: 3,
                        suit: 0.75,
                        faction: 2,
                        capital: true,
                        kind: cartalith_civ::SettlementKind::City,
                        coastal: true,
                    },
                    name: "Sevjuniana".into(),
                    pop: 41230,
                },
                cartalith_civ::NamedSettlement {
                    tid: 8,
                    placement: cartalith_civ::SettlementPlacement {
                        x: 1,
                        y: 1,
                        suit: 0.25,
                        faction: 0,
                        capital: false,
                        kind: cartalith_civ::SettlementKind::Hamlet,
                        coastal: false,
                    },
                    name: String::new(),
                    pop: 120,
                },
            ],
            ways: vec![cartalith_civ::Way {
                tid: 21,
                pts: vec![(1.5, 2.5), (3.0, 4.0)],
                brks: vec![1],
                km: 12.5,
                name: "Verath Road".into(),
                way_type: cartalith_civ::WayType::Highway,
                a_idx: 0,
                b_idx: 1,
                hidden: false,
            }],
            sea_routes: vec![cartalith_civ::SeaRoute {
                pts: vec![(0.0, 0.0), (2.0, 2.0)],
                brks: vec![],
                km: 402.0,
                name: "Northern Passage".into(),
            }],
            territory: vec![0, 1, 2, 2, 0, 1, 1, 0, 2, 0, 1, 2],
            provinces: vec![0; 12],
            province_list: vec![cartalith_civ::Province {
                id: 1,
                faction: 2,
                name: "Upper Verath".into(),
                capital_settlement_index: 0,
            }],
            continents: vec![cartalith_civ::Continent {
                id: 1,
                name: "Aurelia".into(),
                cells: 9,
                min_x: 0,
                min_y: 0,
                max_x: 3,
                max_y: 2,
                cx: 1.5,
                cy: 1.0,
                faction: 2,
            }],
            trade_balances: vec![
                cartalith_civ::TradeBalance {
                    exports: vec![cartalith_civ::CIV_RESOURCE_KEYS[0]],
                    imports: vec![cartalith_civ::CIV_RESOURCE_KEYS[1]],
                },
                cartalith_civ::TradeBalance::default(),
            ],
            explanations: Vec::new(),
            water_bodies: vec![0u8; 12],
            next_tid: 22,
            timeline: vec![cartalith_civ::timeline::TimelineSnapshot {
                year: 120,
                territory: cartalith_civ::timeline::TerritoryFrame::Key(vec![3; 12]),
                settlements: Vec::new(),
                ways: Vec::new(),
                collapse_flags: Default::default(),
                record: Default::default(),
            }],
            year: 120,
            dens: vec![0.5f32; 12],
            faction_roster: civ_roster_bridge::FactionRoster::seeded(6),
            religions: Default::default(),
            place_extras: civ_roster_bridge::PlaceExtrasTable(place_extras),
            village_tids,
        }
    }

    /// Builds a real archive from a `CivData`, reads it back, and returns
    /// the reconstructed one -- a write -> read -> compare through the real
    /// container rather than the DTOs alone.
    fn round_trip(civ: &CivData, gw: usize, gh: usize) -> CivData {
        let n = gw * gh;
        let params = cartalith_io::SaveParams {
            gw,
            gh,
            seed: 4242,
            map_width_km: 800.0,
            sea_level: 0.42,
            world: false,
            // Pre-provenance fixture: the archive shape a user's existing
            // save has, so the assertions below cover the absent case.
            origin: None,
            name: None,
        };
        let fields = cartalith_io::SaveFields {
            heightmap: std::sync::Arc::new(vec![0.5; n]),
            temperature: std::sync::Arc::new(vec![10.0; n]),
            rainfall: std::sync::Arc::new(vec![1.0; n]),
            volcanic_field: vec![0.0; n],
            impact_field: vec![0.0; n],
            strahler_order: vec![0; n],
        };
        let mut write = ProjectWrite::new(&params, &fields);
        let mut documents = BTreeMap::new();
        civ_documents(civ, None, &mut documents);
        write.documents = documents;
        if civ.territory.len() == n {
            write.raster("rasters/territory.i32", Raster::I32(civ.territory.clone()));
        }
        if civ.provinces.len() == n {
            write.raster("rasters/provinces.i32", Raster::I32(civ.provinces.clone()));
        }
        if civ.water_bodies.len() == n {
            write.raster(
                "rasters/water_bodies.u8",
                Raster::U8(civ.water_bodies.clone()),
            );
        }
        if civ.dens.len() == n {
            write.raster(
                "rasters/agrarian_density.f32",
                Raster::F32(civ.dens.clone()),
            );
        }
        for snap in &civ.timeline {
            if let Some(raster) =
                cartalith_civ::timeline::civ_territory_at(&civ.timeline, snap.year)
            {
                if raster.len() == n {
                    write.history_territory.insert(snap.year, raster);
                }
            }
        }

        let mut buf = Vec::new();
        project::write_project(std::io::Cursor::new(&mut buf), &write)
            .expect("write_project should succeed");
        let data = cartalith_io::read_project(std::io::Cursor::new(&buf))
            .expect("read_project should succeed");
        assert!(data.warnings.is_empty(), "{:?}", data.warnings);
        civ_from_project(&data, n, &mut Vec::new()).expect("a civ layer that was written must come back")
    }

    /// [`round_trip`] with a chosen `civ.garrison_exposure_scale`, returning
    /// the raw archive `ProjectData` instead of the restored `CivData`. Since
    /// the fold-in, the setting rides `params.json`'s `cartalith` block like
    /// every other `civ.*` dial -- built here through `params::save_state`
    /// exactly as `project_open`'s own call site builds `write.cartalith_
    /// params` -- rather than through `civ_documents`/`FactionsDoc`, which no
    /// longer carries a live value for it at all.
    fn round_trip_data(civ: &CivData, gw: usize, gh: usize, scale: f64) -> cartalith_io::ProjectData {
        let n = gw * gh;
        let params = cartalith_io::SaveParams {
            gw, gh, seed: 4242, map_width_km: 800.0, sea_level: 0.42, world: false,
            origin: None, name: None,
        };
        let fields = cartalith_io::SaveFields {
            heightmap: std::sync::Arc::new(vec![0.5; n]),
            temperature: std::sync::Arc::new(vec![10.0; n]),
            rainfall: std::sync::Arc::new(vec![1.0; n]),
            volcanic_field: vec![0.0; n],
            impact_field: vec![0.0; n],
            strahler_order: vec![0; n],
        };
        let mut write = ProjectWrite::new(&params, &fields);
        let mut documents = BTreeMap::new();
        civ_documents(civ, None, &mut documents);
        write.documents = documents;
        let mut p = cartalith_engine::WorldParams::defaults(gw, gh, 4242);
        p.civ.garrison_exposure_scale = scale;
        let mut state = params::save_state(&p);
        write.cartalith_params = state
            .as_object_mut()
            .and_then(|o| o.remove(params::NATIVE_PARAMS_KEY))
            .unwrap_or(serde_json::Value::Null);
        let mut buf = Vec::new();
        project::write_project(std::io::Cursor::new(&mut buf), &write)
            .expect("write_project should succeed");
        let data = cartalith_io::read_project(std::io::Cursor::new(&buf))
            .expect("read_project should succeed");
        assert!(data.warnings.is_empty(), "{:?}", data.warnings);
        data
    }

    /// Writes a real archive whose `params.json` native block is built from
    /// `params::save_state` of a world with `integrate_drainage = flag`, then
    /// lets `edit` alter the native object (to delete the key for an
    /// old-save fixture), reads it back through `cartalith_io::read_project`
    /// and returns the `WorldParams` that `params::apply_saved_state` restores
    /// into a session that starts with `session_value` (so the result shows
    /// what the load decided, not what the session held). Test-only; Ruling BW
    /// item 4.
    fn load_drainage_flag(
        flag: bool,
        session_value: bool,
        edit: impl FnOnce(&mut serde_json::Map<String, serde_json::Value>),
    ) -> bool {
        let (gw, gh) = (4usize, 3usize);
        let n = gw * gh;
        let params = cartalith_io::SaveParams {
            gw, gh, seed: 4242, map_width_km: 800.0, sea_level: 0.42, world: false,
            origin: None, name: None,
        };
        let fields = cartalith_io::SaveFields {
            heightmap: std::sync::Arc::new(vec![0.5; n]),
            temperature: std::sync::Arc::new(vec![10.0; n]),
            rainfall: std::sync::Arc::new(vec![1.0; n]),
            volcanic_field: vec![0.0; n],
            impact_field: vec![0.0; n],
            strahler_order: vec![0; n],
        };
        let mut write = ProjectWrite::new(&params, &fields);
        let mut p = cartalith_engine::WorldParams::defaults(gw, gh, 4242);
        p.integrate_drainage = flag;
        let mut state = params::save_state(&p);
        let mut native = state
            .as_object_mut()
            .and_then(|o| o.remove(params::NATIVE_PARAMS_KEY))
            .expect("save_state writes a native block");
        edit(native.as_object_mut().expect("native block is an object"));
        write.cartalith_params = native;
        let mut buf = Vec::new();
        project::write_project(std::io::Cursor::new(&mut buf), &write).expect("write_project");
        let data = cartalith_io::read_project(std::io::Cursor::new(&buf)).expect("read_project");
        let mut restored = cartalith_engine::WorldParams::defaults(gw, gh, 4242);
        restored.integrate_drainage = session_value;
        params::apply_saved_state(&mut restored, &data.save.state);
        restored.integrate_drainage
    }

    /// Protects: Ruling BW item 4 (owner, 2026-10-10) -- an archive written
    /// before `integrate_drainage` existed (the key ABSENT from the native
    /// block of `params.json`) loads with it ON. Detection is absence of the
    /// key, never a stored value; the fixture is a real archive with the key
    /// deleted, loaded into a session that held `false` so the migration is
    /// what turns it on. Mutate `apply_saved_state`'s default back to false
    /// and this goes red.
    #[test]
    fn old_archive_without_integrate_drainage_loads_with_it_on() {
        assert!(load_drainage_flag(false, false, |n| {
            assert!(n.remove("integrate_drainage").is_some(), "the writer must emit the key");
        }));
    }

    /// Protects: a new archive saved with `integrate_drainage = true` loads
    /// true (the migration must not touch a key that is present).
    #[test]
    fn new_archive_saved_with_integrate_drainage_true_stays_true() {
        assert!(load_drainage_flag(true, false, |_| {}));
    }

    /// Protects: Ruling BW item 4's "never encode no-value as a plausible
    /// value" rule -- an archive that EXPLICITLY saved `integrate_drainage =
    /// false` stays false; only an absent key migrates to true.
    #[test]
    fn new_archive_saved_with_explicit_false_stays_false() {
        assert!(!load_drainage_flag(false, true, |_| {}));
    }

    /// Protects: save-then-load stability of `integrate_drainage` for both
    /// values and from either session starting value (a second save of the
    /// loaded params writes what was loaded).
    #[test]
    fn integrate_drainage_save_round_trip_is_stable() {
        for flag in [true, false] {
            for session in [true, false] {
                assert_eq!(load_drainage_flag(flag, session, |_| {}), flag);
            }
        }
    }

    /// Protects: `civ.garrison_exposure_scale` persisting through `params.json`.
    /// The fold-in's own round trip: `civ.garrison_exposure_scale` is an
    /// ordinary `params::PARAMS` row now, so it is written unconditionally
    /// (like every other row -- `params::save_state`'s own doc comment) and
    /// comes back exact through a real archive at several values, including
    /// the default.
    #[test]
    fn garrison_exposure_scale_round_trips_through_params_json() {
        let civ = sample_civ();
        for scale in [0.0, 0.6, 1.0, 2.5, 5.0] {
            let data = round_trip_data(&civ, 4, 3, scale);
            let native = data
                .save
                .state
                .get(params::NATIVE_PARAMS_KEY)
                .expect("cartalith block must be present");
            assert_eq!(
                native.get("civ.garrison_exposure_scale").and_then(|v| v.as_f64()),
                Some(scale),
                "scale {scale}"
            );
            let mut p = cartalith_engine::WorldParams::defaults(4, 3, 4242);
            params::apply_saved_state(&mut p, &data.save.state);
            assert_eq!(p.civ.garrison_exposure_scale, scale, "scale {scale}");
        }
    }

    /// Protects: the legacy factions.json garrison member never being written
    /// again, and an absent member reading back as `None` (not `0.0`).
    /// [`FactionsDoc::garrison_exposure_scale`] is read-only compatibility
    /// now: `civ_documents` never writes it (checked against a document a
    /// freshly generated project actually produces, not a hand-built one),
    /// and a document built from `FactionsDoc::default()` -- what a project
    /// that predates the member entirely looks like -- reads back `None`, the
    /// "no legacy value" case, never a manufactured `0.0`.
    #[test]
    fn legacy_garrison_exposure_scale_member_is_never_written() {
        let civ = sample_civ();
        let mut docs = BTreeMap::new();
        civ_documents(&civ, None, &mut docs);
        assert!(
            !docs[SLOT_FACTIONS].contains("garrison_exposure_scale"),
            "civ_documents must never write the legacy member any more: {}",
            docs[SLOT_FACTIONS]
        );

        let empty = serde_json::to_string(&FactionsDoc::default()).unwrap();
        assert!(!empty.contains("garrison_exposure_scale"));
        let doc: FactionsDoc = serde_json::from_str(&empty).unwrap();
        assert_eq!(doc.garrison_exposure_scale, None);
    }

    /// Protects: archives written by commit 382945a (legacy factions.json
    /// garrison value, no params.json key) still opening with that value.
    /// **The real backward-compatibility fixture**: bytes actually written by
    /// commit 382945a's own `civ_documents`/`FactionsDoc` -- the format that
    /// first shipped this setting, only as `factions.json`'s
    /// `garrison_exposure_scale` member, before the fold-in gave it a
    /// `params.json` row at all. Generated once from a worktree at 382945a
    /// (`sample_civ()`, `scale = 2.5`, a 4x3 grid) and embedded verbatim --
    /// not hand-built JSON, per `MISTAKES.md`'s own rule on backward-
    /// compatibility fixtures.
    ///
    /// `resolve_garrison_exposure_scale` must read the legacy `2.5` back: the
    /// archive's `params.json` (if it has one at all -- 382945a's writer
    /// built no `cartalith_params` for this test helper) carries no
    /// `civ.garrison_exposure_scale` key, so the params copy is genuinely
    /// absent and the legacy member is the only value there is.
    #[test]
    fn a_project_saved_by_382945a_opens_with_its_legacy_garrison_value() {
        const FIXTURE: &[u8] = include_bytes!(
            "../tests/fixtures/project_382945a_garrison_2_5.ctl"
        );
        let data = cartalith_io::read_project(std::io::Cursor::new(FIXTURE))
            .expect("the 382945a fixture must still read as a project");
        assert!(data.warnings.is_empty(), "{:?}", data.warnings);
        let has_native_key = data
            .save
            .state
            .get(params::NATIVE_PARAMS_KEY)
            .and_then(|c| c.get("civ.garrison_exposure_scale"))
            .is_some();
        assert!(
            !has_native_key,
            "a 382945a-era archive must genuinely predate this params.json key"
        );
        let resolved = resolve_garrison_exposure_scale(&data, cartalith_civ::garrison::EXPOSURE_SCALE);
        assert_eq!(resolved, 2.5, "the legacy factions.json member must win when the new key is absent");
    }

    /// Protects: Ruling BO's persistence: the forced-lake mask a save writes
    /// (`forced_lakes_raster`) comes back byte for byte on open
    /// (`forced_lakes_from_project`) through a real in-memory archive; a world
    /// with nothing forced writes **no** member (absent, not all-zero), and an
    /// archive without one -- every archive before 2026-09-28 -- restores
    /// nothing and warns about nothing. A mask for another grid is never
    /// written.
    #[test]
    fn forced_lakes_survive_a_save_and_an_unforced_world_writes_none() {
        let (gw, gh) = (4usize, 3usize);
        let n = gw * gh;
        let save_params = cartalith_io::SaveParams {
            gw, gh, seed: 4242, map_width_km: 800.0, sea_level: 0.42, world: false,
            origin: None, name: None,
        };
        let fields = cartalith_io::SaveFields {
            heightmap: std::sync::Arc::new(vec![0.5; n]),
            temperature: std::sync::Arc::new(vec![10.0; n]),
            rainfall: std::sync::Arc::new(vec![1.0; n]),
            volcanic_field: vec![0.0; n],
            impact_field: vec![0.0; n],
            strahler_order: vec![0; n],
        };
        let archive = |mask: Option<&[u8]>| {
            let mut write = ProjectWrite::new(&save_params, &fields);
            forced_lakes_raster(mask, n, &mut write);
            let mut buf = Vec::new();
            project::write_project(std::io::Cursor::new(&mut buf), &write).unwrap();
            cartalith_io::read_project(std::io::Cursor::new(&buf)).unwrap()
        };
        let mask: Vec<u8> = (0..n).map(|i| u8::from(i == 5 || i == 6)).collect();
        let data = archive(Some(&mask));
        assert!(data.warnings.is_empty(), "{:?}", data.warnings);
        let mut warnings = Vec::new();
        assert_eq!(forced_lakes_from_project(&data, n, &mut warnings), Some(mask.clone()));
        assert!(warnings.is_empty());

        for none in [None, Some(&vec![0u8; n][..])] {
            let data = archive(none);
            assert!(data.raster(RASTER_FORCED_LAKES).is_none(), "nothing forced, no member written");
            assert_eq!(forced_lakes_from_project(&data, n, &mut warnings), None);
            assert!(warnings.is_empty());
        }

        // A mask for another grid never reaches the archive: `write_project`
        // refuses a raster whose length is not the grid's (`RasterLength`),
        // and `forced_lakes_raster` never offers one.
        let short = vec![1u8; n - 1];
        let data = archive(Some(&short));
        assert!(data.raster(RASTER_FORCED_LAKES).is_none(), "a short mask is not written");
    }

    /// Protects: the params.json garrison value winning over the legacy
    /// factions.json member when an archive carries both.
    /// The other half of the same precedence: when *both* the new
    /// `params.json` key and the legacy `factions.json` member exist in the
    /// same archive -- which no writer in this codebase produces any more,
    /// but a hand-edited file could -- the params copy wins. Built by hand
    /// rather than through `civ_documents` (which no longer writes the
    /// legacy member at all, by design) precisely to construct the
    /// combination no live writer can.
    #[test]
    fn resolve_garrison_exposure_scale_prefers_the_params_copy_when_both_exist() {
        let civ = sample_civ();
        let gw = 4usize;
        let gh = 3usize;
        let n = gw * gh;
        let save_params = cartalith_io::SaveParams {
            gw, gh, seed: 4242, map_width_km: 800.0, sea_level: 0.42, world: false,
            origin: None, name: None,
        };
        let fields = cartalith_io::SaveFields {
            heightmap: std::sync::Arc::new(vec![0.5; n]),
            temperature: std::sync::Arc::new(vec![10.0; n]),
            rainfall: std::sync::Arc::new(vec![1.0; n]),
            volcanic_field: vec![0.0; n],
            impact_field: vec![0.0; n],
            strahler_order: vec![0; n],
        };
        let mut write = ProjectWrite::new(&save_params, &fields);
        let mut documents = BTreeMap::new();
        civ_documents(&civ, None, &mut documents);
        // Hand-insert the legacy member into the faction document nothing
        // here writes any more -- the "unusual archive" case.
        let mut factions: FactionsDoc = serde_json::from_str(&documents[SLOT_FACTIONS]).unwrap();
        factions.garrison_exposure_scale = Some(0.2);
        documents.insert(SLOT_FACTIONS.to_string(), serde_json::to_string(&factions).unwrap());
        write.documents = documents;
        // And the new-format key, present at a different value.
        let mut p = cartalith_engine::WorldParams::defaults(gw, gh, 4242);
        p.civ.garrison_exposure_scale = 3.0;
        let mut state = params::save_state(&p);
        write.cartalith_params = state
            .as_object_mut()
            .and_then(|o| o.remove(params::NATIVE_PARAMS_KEY))
            .unwrap_or(serde_json::Value::Null);

        let mut buf = Vec::new();
        project::write_project(std::io::Cursor::new(&mut buf), &write).unwrap();
        let data = cartalith_io::read_project(std::io::Cursor::new(&buf)).unwrap();
        assert!(data.warnings.is_empty(), "{:?}", data.warnings);

        // Both are present in this archive; the params copy (3.0) must win
        // over the legacy member (0.2).
        assert_eq!(resolve_garrison_exposure_scale(&data, 3.0), 3.0);
    }

    /// `STORY_PLANNING_SCOPE.md` SP-1's own round trip, at the same level
    /// `round_trip` above exercises the civ layer at -- through a real
    /// in-memory `.ctl` archive (`project::write_project`/
    /// `cartalith_io::read_project`), never through a live `WorldGen`
    /// (`InfraTools`/`Journey` need no `godot` dependency at all, so this
    /// runs under plain `cargo test`, no Godot runtime).
    fn journeys_round_trip(
        next_id: u64,
        journeys: &[cartalith_civ::travel_library::Journey],
    ) -> (u64, Vec<cartalith_civ::travel_library::Journey>) {
        let params = cartalith_io::SaveParams {
            gw: 4,
            gh: 3,
            seed: 4242,
            map_width_km: 800.0,
            sea_level: 0.42,
            world: false,
            origin: None,
            name: None,
        };
        let fields = cartalith_io::SaveFields {
            heightmap: std::sync::Arc::new(vec![0.5; 12]),
            temperature: std::sync::Arc::new(vec![10.0; 12]),
            rainfall: std::sync::Arc::new(vec![1.0; 12]),
            volcanic_field: vec![0.0; 12],
            impact_field: vec![0.0; 12],
            strahler_order: vec![0; 12],
        };
        let mut write = ProjectWrite::new(&params, &fields);
        let mut documents = BTreeMap::new();
        insert_doc(
            &mut documents,
            SLOT_JOURNEYS,
            &JourneysDoc {
                next_id,
                journeys: journeys.iter().map(journey_to_dto).collect(),
            },
        );
        write.documents = documents;

        let mut buf = Vec::new();
        project::write_project(std::io::Cursor::new(&mut buf), &write)
            .expect("write_project should succeed");
        let data = cartalith_io::read_project(std::io::Cursor::new(&buf))
            .expect("read_project should succeed");
        assert!(data.warnings.is_empty(), "{:?}", data.warnings);

        let doc: JourneysDoc = data
            .parse(SLOT_JOURNEYS)
            .expect("entities/journeys.json must be present")
            .expect("entities/journeys.json must parse");
        (doc.next_id, doc.journeys.iter().map(dto_to_journey).collect())
    }

    /// Protects: `STORY_PLANNING_SCOPE.md` SP-1's bar that a plan-less journey
    /// (including its Ruling BS resupply stop) survives save -> load -> reopen
    /// field for field through a real archive.
    #[test]
    fn a_journey_survives_save_load_reopen() {
        // The scope document's own acceptance bar, verbatim: "a journey
        // survives save -> load -> reopen". Every SP-1 field, checked
        // individually rather than via `assert_eq!` on the whole struct, so
        // a future field added to `Journey` without a matching DTO member
        // fails loudly here instead of silently not round-tripping.
        let journey = cartalith_civ::travel_library::Journey {
            id: 7,
            name: "The salt road".to_string(),
            party_preset: "merchant_caravan".to_string(),
            route: cartalith_civ::travel_library::JourneyRoute {
                points: vec![(10.0, 4.0), (11.5, 5.25), (13.0, 6.0)],
                breaks: vec![0],
                length_km: 120.5,
                mode: cartalith_civ::tools::RouteMode::Mixed,
            },
            start_year: 412,
            // Ruling BS: one accepted stop, so the new field round-trips too.
            resupply_stops: vec![cartalith_civ::JpAcceptedStop {
                key: "resupply:Kessra|town|11|5".to_string(),
                kind: "settlement".to_string(),
                name: "Kessra".to_string(),
                x: 11.5,
                y: 5.25,
            }],
            // Ruling AR: this fixture is the plan-less shape; a journey with
            // a full plan is `a_journey_with_a_full_non_default_plan_...`.
            plan: None,
        };
        let (next_id, back) = journeys_round_trip(8, std::slice::from_ref(&journey));
        assert_eq!(next_id, 8);
        assert_eq!(back.len(), 1);
        let b = &back[0];
        assert_eq!(b.id, journey.id);
        assert_eq!(b.name, journey.name);
        assert_eq!(b.party_preset, journey.party_preset);
        assert_eq!(b.start_year, journey.start_year);
        assert_eq!(b.route.points, journey.route.points);
        assert_eq!(b.route.breaks, journey.route.breaks);
        assert_eq!(b.route.length_km, journey.route.length_km);
        assert_eq!(b.route.mode, journey.route.mode);
        // The whole point of `PartialEq` on `Journey`/`JourneyRoute`: one
        // assertion that nothing above was a partial check in disguise.
        assert_eq!(*b, journey);
    }

    /// A journey carrying a plan in which **every** member is non-default:
    /// all 28 plan-form fields, two stage overrides (one sets all 25 fields,
    /// one a single field), a layover, an animal entry, a trim and both auto
    /// flags. Built field by field (no `..Default::default()` on the plan) so
    /// a field added to `JpPlan` fails to compile here until it is chosen
    /// non-default too.
    fn full_plan_journey() -> cartalith_civ::travel_library::Journey {
        use cartalith_civ::{JpParty, JpPlan, JpStageOverride};
        let plan = JpPlan {
            party: JpParty { group_size: 12, cargo_kg: 900.5, donkey: 3, mule: 4, camel: 1, horse: 2, carts: 5, wagons: 6, sleds: 7, travois: 8 },
            transport: "Mounted Rider".into(),
            mount_animal: Some("mule".into()),
            vessel: "Dhow".into(),
            hours: 7.5,
            pace: "Cautious / Scouting".into(),
            season: "Autumn".into(),
            supply_days: 14,
            carry_food: false,
            grazing: "Forced".into(),
            foraging: "Opportunistic".into(),
            desert_water: Some("Deep Desert Crossing".into()),
            weather_override: Some("Snow".into()),
            seasonal_closures: false,
            route_cond: Some("Broken".into()),
            infra: Some("Stable Settlements".into()),
            stage_overrides: std::collections::HashMap::from([
                (
                    0,
                    JpStageOverride {
                        transport: Some("Baggage Train".into()),
                        mount_animal: Some("horse".into()),
                        vessel: Some("Cog".into()),
                        hours: Some(11.5),
                        pace: Some("Haste".into()),
                        season: Some("Winter".into()),
                        supply_days: Some(21),
                        carry_food: Some(true),
                        grazing: Some("Light".into()),
                        foraging: Some("Active".into()),
                        desert_water: Some("Oasis".into()),
                        weather_override: Some("Rain".into()),
                        seasonal_closures: Some(true),
                        route_cond: Some("Deteriorated".into()),
                        infra: Some("Road".into()),
                        group_size: Some(9),
                        cargo_kg: Some(650.25),
                        donkey: Some(1),
                        mule: Some(2),
                        camel: Some(3),
                        horse: Some(4),
                        carts: Some(5),
                        wagons: Some(6),
                        sleds: Some(7),
                        travois: Some(8),
                    },
                ),
                (3, JpStageOverride { camel: Some(6), ..Default::default() }),
            ]),
            season_drift: true,
            // The single non-default the Ruling AR tests lean on: the plan
            // form's rest cadence ("Heavy -- 1 in 3"), which moves dates.
            rest_cadence: Some("Heavy \u{2014} 1 in 3".into()),
            auto_promote: true,
            // Accepted stops live on `Journey::resupply_stops`, never here.
            accepted_resupply: Vec::new(),
        };
        cartalith_civ::travel_library::Journey {
            id: 9,
            name: "The planned road".to_string(),
            party_preset: "merchant_caravan".to_string(),
            route: cartalith_civ::travel_library::JourneyRoute {
                points: vec![(10.0, 4.0), (11.5, 5.25), (13.0, 6.0)],
                breaks: vec![0],
                length_km: 120.5,
                mode: cartalith_civ::tools::RouteMode::Mixed,
            },
            start_year: 412,
            resupply_stops: Vec::new(),
            plan: Some(cartalith_civ::travel_library::JourneyPlan {
                plan,
                layovers: BTreeMap::from([("resupply:Kessra|town|11|5".to_string(), 3), ("stop:2".to_string(), 1)]),
                animal_entries: BTreeMap::from([("horse".to_string(), "custom_horse".to_string())]),
                trim: (0.25, 0.75),
                auto_carriage: true,
                auto_stage: true,
            }),
        }
    }

    /// Protects: (Ruling AR) a journey's whole planner plan -- every plan
    /// field, both stage overrides (all 25 fields of one), the layover and
    /// animal-entry maps, the trim and both auto flags -- survives
    /// `journey_to_dto` -> a real in-memory `.ctl` archive -> `dto_to_journey`
    /// unchanged, alongside a plan-less journey in the same document (so
    /// neither shape bleeds into the other), and a second save writes the
    /// same bytes as the first.
    #[test]
    fn a_journey_with_a_full_non_default_plan_survives_save_load_reopen() {
        let planned = full_plan_journey();
        let mut plain = full_plan_journey();
        plain.id = 10;
        plain.plan = None;
        let (next_id, back) = journeys_round_trip(11, &[planned.clone(), plain.clone()]);
        assert_eq!(next_id, 11);
        assert_eq!(back.len(), 2);
        assert_eq!(back[0], planned, "the full plan came back field for field");
        assert_eq!(back[1], plain, "a plan-less neighbour stays plan-less");
        assert!(back[1].plan.is_none(), "an absent plan is None, not a default plan");
        // The two scalars the GDScript probe moves the dates with.
        let p = back[0].plan.as_ref().expect("plan");
        assert_eq!(p.plan.rest_cadence.as_deref(), Some("Heavy \u{2014} 1 in 3"));
        assert_eq!(p.trim, (0.25, 0.75));
        // Byte stability: serialise, parse, serialise.
        let doc = JourneysDoc { next_id: 11, journeys: back.iter().map(journey_to_dto).collect() };
        let once = serde_json::to_string(&doc).unwrap();
        let reparsed: JourneysDoc = serde_json::from_str(&once).unwrap();
        let again = JourneysDoc { next_id: 11, journeys: reparsed.journeys.iter().map(dto_to_journey).map(|j| journey_to_dto(&j)).collect() };
        assert_eq!(serde_json::to_string(&again).unwrap(), once);
    }

    /// Protects: (Ruling AR, `SAVEFILE_COMPAT.md` 9.6) a journeys document
    /// written by the build just before the ruling -- Ruling BS's shape, with
    /// `resupply_stops` and no `plan` -- opens with **no plan** (`None`, not
    /// a default plan: the planner restores a preset for it) and re-saves to
    /// the very bytes it was read from.
    #[test]
    fn a_pre_ruling_ar_journey_loads_with_no_plan_and_resaves_byte_identically() {
        let old = r#"{"next_id":8,"journeys":[{"id":7,"name":"The salt road","party_preset":"merchant_caravan","route":{"points":[[10.0,4.0],[11.5,5.25],[13.0,6.0]],"breaks":[0],"length_km":120.5,"mode":"mixed"},"start_year":412,"resupply_stops":[{"key":"resupply:Kessra|town|11|5","kind":"settlement","name":"Kessra","at":[11.5,5.25]}]}]}"#;
        let doc: JourneysDoc = serde_json::from_str(old).expect("the pre-AR shape parses");
        let journeys: Vec<_> = doc.journeys.iter().map(dto_to_journey).collect();
        assert_eq!(journeys.len(), 1);
        assert!(journeys[0].plan.is_none(), "no plan member means no plan");
        assert_eq!(journeys[0].resupply_stops.len(), 1, "the BS member came through");
        let again = JourneysDoc { next_id: doc.next_id, journeys: journeys.iter().map(journey_to_dto).collect() };
        assert_eq!(serde_json::to_string(&again).unwrap(), old);
    }

    /// Protects: (Ruling AR, `SAVEFILE_COMPAT.md` 9.6 / 14.3) a reader
    /// tolerates a `plan` member from a newer or hand-edited file. Unknown
    /// plan-form keys, wrong-typed values and an unparsable stage index are
    /// dropped without refusing the journey; every missing member takes the
    /// "not set" value -- the whole route for `trim`, off for the auto flags
    /// -- never a zero trim that would plan nothing.
    #[test]
    fn an_unknown_or_partial_plan_member_is_tolerated() {
        let text = r#"{"id":7,"name":"n","party_preset":"p","route":{"points":[[0.0,0.0],[1.0,1.0]],"breaks":[],"length_km":1.0,"mode":"mixed"},"start_year":1,"plan":{"fields":{"hours":6.5,"rest_cadence":"Light \u2014 1 in 7","not_a_field":1,"season":42},"stage_overrides":{"2":{"camel":3,"bogus":true},"x":{"camel":1}},"future_member":[1,2]}}"#;
        let dto: JourneyDto = serde_json::from_str(text).expect("a partial plan still parses");
        let j = dto_to_journey(&dto);
        let p = j.plan.expect("the plan member was present");
        assert_eq!(p.plan.hours, 6.5);
        assert_eq!(p.plan.rest_cadence.as_deref(), Some("Light \u{2014} 1 in 7"));
        assert_eq!(p.plan.season, cartalith_civ::JpPlan::default().season, "a wrong-typed value keeps the default");
        assert_eq!(p.plan.stage_overrides.len(), 1, "the unparsable stage index was dropped");
        assert_eq!(p.plan.stage_overrides[&2].camel, Some(3));
        assert_eq!(p.trim, (0.0, 1.0), "an absent trim is the whole route");
        assert!(!p.auto_carriage && !p.auto_stage);
        assert!(p.layovers.is_empty() && p.animal_entries.is_empty());
        // A `plan` of the wrong JSON type is a parse error on the document,
        // which `SAVEFILE_COMPAT.md` 6.4a rung 3 already routes to "that one
        // slot is skipped, every other document opens" rather than a panic.
        let head = &text[..text.find(r#""plan":"#).unwrap()];
        let bad = format!(r#"{head}"plan":[1]}}"#);
        assert!(serde_json::from_str::<JourneyDto>(&bad).is_err());
    }

    /// Protects: (Ruling AR) the wire form names each plan member under the
    /// key `SAVEFILE_COMPAT.md` 9.6 documents -- an independent assertion on
    /// the JSON text (not a round trip, which would pass with both ends
    /// renamed together).
    #[test]
    fn the_plan_member_is_written_under_the_documented_keys() {
        let v = serde_json::to_value(journey_to_dto(&full_plan_journey())).unwrap();
        let plan = &v["plan"];
        assert_eq!(plan["fields"]["rest_cadence"], "Heavy \u{2014} 1 in 3");
        assert_eq!(plan["fields"]["hours"], 7.5);
        assert_eq!(plan["fields"]["group_size"], 12);
        assert_eq!(plan["stage_overrides"]["3"]["camel"], 6);
        assert_eq!(plan["stage_overrides"]["0"]["hours"], 11.5);
        assert_eq!(plan["layovers"]["stop:2"], 1);
        assert_eq!(plan["animal_entries"]["horse"], "custom_horse");
        assert_eq!(plan["trim"], serde_json::json!([0.25, 0.75]));
        assert_eq!(plan["auto_carriage"], true);
        assert_eq!(plan["auto_stage"], true);
        assert!(plan.get("accepted_resupply").is_none(), "accepted stops live on the journey, not the plan");
        let plain = serde_json::to_value(journey_to_dto(&{
            let mut j = full_plan_journey();
            j.plan = None;
            j
        }))
        .unwrap();
        assert!(plain.get("plan").is_none(), "no plan means no member, not a null or an empty object");
    }

    /// Protects: `SAVEFILE_COMPAT.md` for Ruling BS's new `resupply_stops`
    /// member -- a journeys document written before the ruling opens with no
    /// stops and re-serialises to the very bytes it was read from, so opening
    /// an old project never silently rewrites it. The fixture is a document
    /// in the pre-ruling `JourneyDto` shape (its five members, in serde's
    /// declaration order), not an empty collection.
    #[test]
    fn a_pre_ruling_bs_journey_loads_with_no_stops_and_resaves_byte_identically() {
        let old = r#"{"next_id":8,"journeys":[{"id":7,"name":"The salt road","party_preset":"merchant_caravan","route":{"points":[[10.0,4.0],[11.5,5.25],[13.0,6.0]],"breaks":[0],"length_km":120.5,"mode":"mixed"},"start_year":412}]}"#;
        let doc: JourneysDoc = serde_json::from_str(old).expect("the pre-ruling shape parses");
        let journeys: Vec<_> = doc.journeys.iter().map(dto_to_journey).collect();
        assert_eq!(journeys.len(), 1);
        assert!(journeys[0].resupply_stops.is_empty());
        assert_eq!(journeys[0].route.points.len(), 3, "the rest of the journey came through");
        let again = JourneysDoc { next_id: doc.next_id, journeys: journeys.iter().map(journey_to_dto).collect() };
        assert_eq!(serde_json::to_string(&again).unwrap(), old);
    }

    /// Protects: SP-4's conflict persistence (anchored and unanchored, ended
    /// and ongoing) and that absent optional members are omitted, not sentinels.
    /// SP-4's "the drawing persists through save/load", through a real
    /// in-memory archive like `journeys_round_trip`. Two conflicts cover
    /// every optional member both ways: an anchored, ended siege and an
    /// unanchored, ongoing front.
    #[test]
    fn a_conflict_survives_save_load_reopen() {
        use cartalith_civ::conflict::{Conflict, ConflictAnchor, ConflictKind};
        let siege = Conflict {
            id: 3,
            name: "Siege of Kessra".into(),
            kind: ConflictKind::Siege,
            start_year: 212,
            end_year: Some(214),
            sides: vec![2, 1],
            outcome: "Relieved in the third spring".into(),
            points: vec![(40.5, 17.25)],
            anchor: Some(ConflictAnchor::Province(9)),
            anchor_at: (41.0, 18.0),
        };
        let front = Conflict {
            id: 5,
            name: "Northern front".into(),
            kind: ConflictKind::Front,
            start_year: 300,
            end_year: None,
            sides: vec![3, 4],
            outcome: String::new(),
            points: vec![(1.0, 2.0), (3.5, 2.5), (6.0, 1.0)],
            anchor: None,
            anchor_at: (0.0, 0.0),
        };
        let params = cartalith_io::SaveParams {
            gw: 4,
            gh: 3,
            seed: 4242,
            map_width_km: 800.0,
            sea_level: 0.42,
            world: false,
            origin: None,
            name: None,
        };
        let fields = cartalith_io::SaveFields {
            heightmap: std::sync::Arc::new(vec![0.5; 12]),
            temperature: std::sync::Arc::new(vec![10.0; 12]),
            rainfall: std::sync::Arc::new(vec![1.0; 12]),
            volcanic_field: vec![0.0; 12],
            impact_field: vec![0.0; 12],
            strahler_order: vec![0; 12],
        };
        let mut write = ProjectWrite::new(&params, &fields);
        let mut documents = BTreeMap::new();
        insert_doc(
            &mut documents,
            SLOT_CONFLICTS,
            &ConflictsDoc {
                next_id: 6,
                conflicts: vec![conflict_to_dto(&siege), conflict_to_dto(&front)],
                anchor_convention: "centre".to_string(),
            },
        );
        write.documents = documents;
        let mut buf = Vec::new();
        project::write_project(std::io::Cursor::new(&mut buf), &write).expect("write");
        let data = cartalith_io::read_project(std::io::Cursor::new(&buf)).expect("read");
        assert!(data.warnings.is_empty(), "{:?}", data.warnings);
        let text = data.document(SLOT_CONFLICTS).expect("present").to_string();
        // Absent, not a sentinel: the ongoing front writes no end year and the
        // unattached one no anchor.
        assert_eq!(text.matches("end_year").count(), 1, "{text}");
        assert_eq!(text.matches("\"anchor\"").count(), 1, "{text}");
        let doc: ConflictsDoc = data.parse(SLOT_CONFLICTS).unwrap().unwrap();
        assert_eq!(doc.next_id, 6);
        let back: Vec<Conflict> = doc.conflicts.iter().filter_map(dto_to_conflict).collect();
        assert_eq!(back, vec![siege, front]);
    }

    /// Ruling AY (`LARGE_ITEM_RULINGS.md`, 2026-09-28), OUTSTANDING_WORK.md
    /// section 2.11 "Migrate pre-fix conflict anchors on load". This literal
    /// JSON is not hand-built: it is `entities/conflicts.json` exactly as a
    /// real pre-fix build wrote it, captured from a worktree at `36312e2^`
    /// (the commit immediately before the cell-centre fix) with a settlement
    /// anchored siege at cell `(7, 4)` -- the pre-fix build's own
    /// `conflict_bridge::anchor_pos` resolved that to the raw corner `(7, 4)`,
    /// which is what `anchor.at` holds here, and no `anchor_convention`
    /// member exists at all (the marker did not yet exist). Provenance is
    /// recorded in the message of commit `d4b2bd9a`, which added this literal.
    const PRE_FIX_CONFLICTS_JSON: &str = r#"{"conflicts":[{"anchor":{"at":[7,4],"kind":"settlement","tid":9},"end_year":214,"id":3,"kind":"siege","name":"Siege of Kessra","outcome":"Relieved in the third spring","points":[[7,4]],"sides":[2,1],"start_year":212}],"next_id":4}"#;

    /// Protects: Ruling AY's migration -- a pre-fix archive with no
    /// `anchor_convention` has its corner-convention `anchor_at` shifted to
    /// the cell centre on load, leaving `points` untouched, so the drawn
    /// pixels equal what the pre-fix build drew.
    #[test]
    fn a_pre_fix_archive_migrates_its_stored_anchor_to_the_centre_convention() {
        let doc: ConflictsDoc = serde_json::from_str(PRE_FIX_CONFLICTS_JSON).expect("parses");
        // No marker at all: serde's default for a missing string member,
        // never confused with a real "centre" value.
        assert_eq!(doc.anchor_convention, "");
        let conflicts = conflicts_from_doc_migrated(&doc);
        assert_eq!(conflicts.len(), 1);
        let c = &conflicts[0];
        // The stored corner (7.0, 4.0) shifted to the centre (7.5, 4.5).
        assert_eq!(c.anchor_at, (7.5, 4.5));
        // Unmigrated points are never touched -- only anchor_at moves.
        assert_eq!(c.points, vec![(7.0, 4.0)]);

        // The settlement (tid 9) has since moved to cell (10, 6); today's
        // (post-fix) anchor_pos resolves that to the centre (10.5, 6.5).
        // The pre-fix build, asked to draw this same archive with the
        // settlement at (10, 6), would have resolved its own (unmigrated,
        // corner) anchor_pos to (10.0, 6.0) and drawn
        // points + (anchor_now - anchor_at) = (7,4) + ((10,6)-(7,4)) =
        // (10.0, 6.0) -- captured directly from that pre-fix build in the
        // same worktree run as the fixture above (resolved_points, literal
        // [(10.0, 6.0)]). The fixed loader must draw the identical pixel
        // from the migrated anchor and today's centre-convention anchor_now:
        assert_eq!(c.resolved_points(Some((10.5, 6.5))), vec![(10.0, 6.0)]);
    }

    /// Protects: Ruling AY's migration runs once -- a conflicts.json carrying `anchor_convention:
    /// "centre"` reopens with its centre-convention `anchor_at` unshifted, so re-saving a
    /// migrated project never shifts it a second time.
    #[test]
    fn a_conflict_saved_by_the_fixed_build_reopens_with_no_double_shift() {
        use cartalith_civ::conflict::{Conflict, ConflictAnchor, ConflictKind};
        let siege = Conflict {
            id: 3,
            name: "Siege of Kessra".into(),
            kind: ConflictKind::Siege,
            start_year: 212,
            end_year: Some(214),
            sides: vec![2, 1],
            outcome: "Relieved in the third spring".into(),
            points: vec![(7.0, 4.0)],
            anchor: Some(ConflictAnchor::Settlement(9)),
            // Already the fixed, centre-convention value -- what
            // conflict_bridge::anchor_pos writes into a freshly attached
            // conflict on this build.
            anchor_at: (7.5, 4.5),
        };
        let params = cartalith_io::SaveParams {
            gw: 4,
            gh: 3,
            seed: 4242,
            map_width_km: 800.0,
            sea_level: 0.42,
            world: false,
            origin: None,
            name: None,
        };
        let fields = cartalith_io::SaveFields {
            heightmap: std::sync::Arc::new(vec![0.5; 12]),
            temperature: std::sync::Arc::new(vec![10.0; 12]),
            rainfall: std::sync::Arc::new(vec![1.0; 12]),
            volcanic_field: vec![0.0; 12],
            impact_field: vec![0.0; 12],
            strahler_order: vec![0; 12],
        };
        let mut write = ProjectWrite::new(&params, &fields);
        let mut documents = BTreeMap::new();
        insert_doc(
            &mut documents,
            SLOT_CONFLICTS,
            &ConflictsDoc {
                next_id: 4,
                conflicts: vec![conflict_to_dto(&siege)],
                anchor_convention: "centre".to_string(),
            },
        );
        write.documents = documents;
        let mut buf = Vec::new();
        project::write_project(std::io::Cursor::new(&mut buf), &write).expect("write");
        let data = cartalith_io::read_project(std::io::Cursor::new(&buf)).expect("read");
        let text = data.document(SLOT_CONFLICTS).expect("present").to_string();
        assert!(text.contains("\"anchor_convention\":\"centre\""), "{text}");
        let doc: ConflictsDoc = data.parse(SLOT_CONFLICTS).unwrap().unwrap();
        assert_eq!(doc.anchor_convention, "centre");
        let conflicts = conflicts_from_doc_migrated(&doc);
        assert_eq!(conflicts.len(), 1);
        // Marker present: no shift. Reopening a fixed-build save must not
        // apply the pre-fix migration a second time.
        assert_eq!(conflicts[0].anchor_at, (7.5, 4.5));
        assert_eq!(conflicts[0], siege);
    }

    /// Protects: Ruling AY's migration touches only anchored conflicts -- in a pre-fix document
    /// (no marker) the free front keeps its points and `anchor_at` exactly, while the
    /// settlement-anchored siege beside it shifts to (7.5, 4.5).
    #[test]
    fn a_free_unanchored_conflict_never_moves_on_migration() {
        use cartalith_civ::conflict::{Conflict, ConflictAnchor, ConflictKind};
        let front = Conflict {
            id: 5,
            name: "Northern front".into(),
            kind: ConflictKind::Front,
            start_year: 300,
            end_year: None,
            sides: vec![3, 4],
            outcome: String::new(),
            points: vec![(1.0, 2.0), (3.5, 2.5), (6.0, 1.0)],
            anchor: None,
            anchor_at: (0.0, 0.0),
        };
        // A pre-fix document (no anchor_convention marker) with one anchored
        // (migrates) and one free conflict (must not move).
        let anchored = Conflict {
            id: 3,
            name: "Siege of Kessra".into(),
            kind: ConflictKind::Siege,
            start_year: 212,
            end_year: Some(214),
            sides: vec![2, 1],
            outcome: String::new(),
            points: vec![(7.0, 4.0)],
            anchor: Some(ConflictAnchor::Settlement(9)),
            anchor_at: (7.0, 4.0),
        };
        let doc = ConflictsDoc {
            next_id: 6,
            conflicts: vec![conflict_to_dto(&anchored), conflict_to_dto(&front)],
            anchor_convention: String::new(),
        };
        let conflicts = conflicts_from_doc_migrated(&doc);
        assert_eq!(conflicts.len(), 2);
        let migrated_front = conflicts.iter().find(|c| c.id == 5).expect("front present");
        assert_eq!(migrated_front.points, front.points);
        assert_eq!(migrated_front.anchor_at, (0.0, 0.0));
        assert_eq!(*migrated_front, front);
        let migrated_anchored = conflicts.iter().find(|c| c.id == 3).expect("siege present");
        assert_eq!(migrated_anchored.anchor_at, (7.5, 4.5));
    }

    /// Protects: the reader's honest-absence half for `entities/journeys.json` -- an archive
    /// written without that document reads back with the slot absent, not as an empty array. Weak
    /// by its own comment: it never writes a journey list or calls `project_save_with_documents`,
    /// so it would pass whatever the writer's `is_empty()` gate did; the gate itself is not
    /// covered here.
    #[test]
    fn an_empty_journeys_list_writes_no_document() {
        // `project_save_with_documents`'s own gate (`if !infra.journeys.is_empty()`)
        // is exercised at the `WorldGen` level, not here -- this asserts the
        // reader's own honest-absence half: a document this test never wrote
        // is absent, not an empty array standing in for "never saved any".
        let data_has_no_slot = {
            let params = cartalith_io::SaveParams {
                gw: 4,
                gh: 3,
                seed: 1,
                map_width_km: 800.0,
                sea_level: 0.42,
                world: false,
                origin: None,
                name: None,
            };
            let fields = cartalith_io::SaveFields {
                heightmap: std::sync::Arc::new(vec![0.5; 12]),
                temperature: std::sync::Arc::new(vec![10.0; 12]),
                rainfall: std::sync::Arc::new(vec![1.0; 12]),
                volcanic_field: vec![0.0; 12],
                impact_field: vec![0.0; 12],
                strahler_order: vec![0; 12],
            };
            let write = ProjectWrite::new(&params, &fields);
            let mut buf = Vec::new();
            project::write_project(std::io::Cursor::new(&mut buf), &write).unwrap();
            let data = cartalith_io::read_project(std::io::Cursor::new(&buf)).unwrap();
            data.document(SLOT_JOURNEYS).is_none()
        };
        assert!(data_has_no_slot);
    }

    /// Protects: backward compatibility with pre-SP-1 archives -- a `journeys.json` whose `route`
    /// is a bare integer does not refuse the archive and fails to parse as `JourneysDoc`, so
    /// `project_open`'s `Some(Ok(..))` gate skips just that document (SAVEFILE_COMPAT.md 6.4a
    /// rung 3).
    #[test]
    fn a_pre_sp1_journeys_document_is_skipped_not_fatal() {
        // Backward compatibility: an archive written by the pre-SP-1 shell
        // stored `route` as a bare array INDEX (`journey_planner_view.gd`'s
        // old `_save_journey`), not this shape's nested `{points, breaks,
        // length_km, mode}` object. `data.parse::<JourneysDoc>` must fail to
        // deserialize that -- proving the type mismatch is real, not an
        // assumption -- and `project_open`'s own `if let Some(Ok(doc))` gate
        // (`SAVEFILE_COMPAT.md` §6.4a rung 3) then skips just this document.
        // Nothing else in this test's archive is touched by that.
        let old_shape = serde_json::json!({
            "journeys": [
                {"name": "Journey 1", "route": 0, "plan": {"transport": "Walking"},
                 "stage_overrides": {}, "layovers": {}, "animal_entries": {},
                 "trim": [0.0, 1.0]}
            ]
        });
        let params = cartalith_io::SaveParams {
            gw: 4,
            gh: 3,
            seed: 1,
            map_width_km: 800.0,
            sea_level: 0.42,
            world: false,
            origin: None,
            name: None,
        };
        let fields = cartalith_io::SaveFields {
            heightmap: std::sync::Arc::new(vec![0.5; 12]),
            temperature: std::sync::Arc::new(vec![10.0; 12]),
            rainfall: std::sync::Arc::new(vec![1.0; 12]),
            volcanic_field: vec![0.0; 12],
            impact_field: vec![0.0; 12],
            strahler_order: vec![0; 12],
        };
        let mut write = ProjectWrite::new(&params, &fields);
        write.document(SLOT_JOURNEYS, old_shape.to_string());
        let mut buf = Vec::new();
        project::write_project(std::io::Cursor::new(&mut buf), &write).unwrap();
        // `read_project` succeeding at all is §6.4's own proof: only a
        // missing/unparseable `project.json` or a wrong-length heightmap may
        // refuse the whole archive, and this one has neither -- a damaged
        // `entities/journeys.json` is not among the four bullets that can.
        let data = cartalith_io::read_project(std::io::Cursor::new(&buf))
            .expect("a damaged entities/journeys.json must not refuse the archive");

        // The document is present (this archive really does carry the old
        // shape)...
        assert!(data.document(SLOT_JOURNEYS).is_some());
        // ...and does not fit the new schema: `route` is an integer where
        // `JourneyRouteDto` needs an object -- proving `project_open`'s own
        // `if let Some(Ok(doc)) = data.parse::<JourneysDoc>(...)` gate really
        // does skip this document (§6.4a rung 3) rather than silently
        // coercing it or panicking.
        let parsed: Option<Result<JourneysDoc, _>> = data.parse(SLOT_JOURNEYS);
        assert!(
            matches!(parsed, Some(Err(_))),
            "an old-shape document must fail to parse as JourneysDoc, not silently coerce"
        );
    }

    /// Protects: `next_tid` after reopen exceeds a `tid` held only by a timeline snapshot (a
    /// deleted settlement, 500) when `settlements.json` carries no `next_id`, so a deleted id is
    /// never reissued and `civ_year_diff` keeps matching by `tid`.
    #[test]
    fn a_restored_counter_clears_every_id_the_timeline_remembers() {
        // The collision `civ_resync_next_tid_with_timeline` exists for: a
        // settlement recorded in a snapshot and since deleted is in neither
        // live array, so a counter rebuilt from the live arrays alone would
        // reissue its `tid` -- and `civ_year_diff` matches snapshots by
        // `tid`.
        //
        // Reachable only when `entities/settlements.json` carries no
        // `next_id` (`#[serde(default)]`, SAVEFILE_COMPAT.md 14.3 -- an
        // archive from a second implementation, or from a build that
        // predates the member), which is why the fixture strips it rather
        // than setting it low.
        let mut civ = sample_civ();
        let mut ghost = civ.settlements[0].clone();
        ghost.tid = 500;
        ghost.name = "Drowned".into();
        // A year `sample_civ` does not already record: the reader dedups by
        // year, so a second snapshot at the same one would be dropped and
        // the fixture would test nothing.
        civ.timeline.push(cartalith_civ::timeline::TimelineSnapshot {
            year: 240,
            territory: cartalith_civ::timeline::TerritoryFrame::empty(),
            settlements: vec![ghost],
            ways: Vec::new(),
            collapse_flags: Default::default(),
            record: Default::default(),
        });
        civ.next_tid = 501;

        let mut documents = BTreeMap::new();
        civ_documents(&civ, None, &mut documents);
        let settlements = documents
            .get_mut(SLOT_SETTLEMENTS)
            .expect("the civ layer always writes its settlements");
        let mut value: serde_json::Value =
            serde_json::from_str(settlements).expect("this file just wrote it");
        value
            .as_object_mut()
            .expect("a document is an object")
            .remove("next_id")
            .expect("the member being stripped must have been there");
        *settlements = serde_json::to_string(&value).expect("re-serializes");
        assert!(
            !settlements.contains("next_id"),
            "the fixture only tests anything with the member gone"
        );

        let params = cartalith_io::SaveParams {
            gw: 4,
            gh: 3,
            seed: 4242,
            map_width_km: 800.0,
            sea_level: 0.42,
            world: false,
            // Pre-provenance fixture: the archive shape a user's existing
            // save has, so the assertions below cover the absent case.
            origin: None,
            name: None,
        };
        let fields = cartalith_io::SaveFields {
            heightmap: std::sync::Arc::new(vec![0.5; 12]),
            temperature: std::sync::Arc::new(vec![10.0; 12]),
            rainfall: std::sync::Arc::new(vec![1.0; 12]),
            volcanic_field: vec![0.0; 12],
            impact_field: vec![0.0; 12],
            strahler_order: vec![0; 12],
        };
        let mut write = ProjectWrite::new(&params, &fields);
        write.documents = documents;
        let mut buf = Vec::new();
        project::write_project(std::io::Cursor::new(&mut buf), &write)
            .expect("write_project should succeed");
        let data = cartalith_io::read_project(std::io::Cursor::new(&buf))
            .expect("read_project should succeed");
        let back = civ_from_project(&data, 12, &mut Vec::new()).expect("a civ layer that was written comes back");

        assert!(
            back.timeline.iter().any(|s| s.settlements.iter().any(|p| p.tid == 500)),
            "the snapshot has to survive for the counter to have anything to clear"
        );
        assert!(
            back.next_tid > 500,
            "the counter must clear the deleted settlement the snapshot still remembers, got {}",
            back.next_tid
        );
        // ...and it is the *timeline* that lifted it, not the live arrays:
        // the milestone-1 twin over the same inputs stops well below.
        assert!(
            cartalith_civ::timeline::civ_resync_next_tid(&back.settlements, &back.ways) <= 500,
            "the fixture must not be one the live-only scan already covers"
        );
    }

    // ===================== the four caller-owned documents =====================

    /// One caller-owned document through a **real** archive: written into a
    /// project `.zip`, read back out, and handed back the way
    /// `project_open`'s `documents` hands it back -- `text_of`, verbatim,
    /// not `serde_json::to_string` of the parsed value.
    ///
    /// A helper rather than four copies, and deliberately not reusing
    /// [`round_trip`] above: that one asserts the *engine*-owned civ layer
    /// survives, and this one asserts the caller-owned channel does not
    /// touch what it carries.
    fn document_round_trip(slot: &str, text: &str) -> String {
        assert!(
            caller_slot_refusal(slot).is_none(),
            "{slot} must be a slot the caller may write"
        );
        let params = cartalith_io::SaveParams {
            gw: 4,
            gh: 3,
            seed: 4242,
            map_width_km: 800.0,
            sea_level: 0.42,
            world: false,
            // Pre-provenance fixture: the archive shape a user's existing
            // save has, so the assertions below cover the absent case.
            origin: None,
            name: None,
        };
        let fields = cartalith_io::SaveFields {
            heightmap: std::sync::Arc::new(vec![0.5; 12]),
            temperature: std::sync::Arc::new(vec![10.0; 12]),
            rainfall: std::sync::Arc::new(vec![1.0; 12]),
            volcanic_field: vec![0.0; 12],
            impact_field: vec![0.0; 12],
            strahler_order: vec![0; 12],
        };
        let mut write = ProjectWrite::new(&params, &fields);
        write.document(slot, text);
        let mut buf = Vec::new();
        project::write_project(std::io::Cursor::new(&mut buf), &write)
            .expect("write_project should succeed");
        let data = cartalith_io::read_project(std::io::Cursor::new(&buf))
            .expect("read_project should succeed");
        assert!(data.warnings.is_empty(), "{:?}", data.warnings);
        data.text_of(slot)
            .unwrap_or_else(|| panic!("{slot} was written and must come back"))
            .to_string()
    }

    /// Protects: `drafts/paint.json` crosses a real archive byte for byte (an unpainted layer
    /// stays an empty list) and its sparse `[index, value]` pairs still decode through
    /// `PaintLayer::decode_sparse`.
    #[test]
    fn a_paint_draft_survives_a_real_archive_round_trip() {
        // Sparse `[index, value, ...]` on two of the three layers and an
        // unallocated third -- the shape `PaintLayer::encode_sparse` emits,
        // including its "an unpainted layer is an empty list, not a run of
        // zeroes" rule.
        let doc = PaintDoc {
            gw: 4,
            gh: 3,
            biome: vec![0, 5, 7, 2, 11, 13],
            terrain: Vec::new(),
            splat: vec![3, 1],
        };
        let text = serde_json::to_string(&doc).expect("PaintDoc serializes");
        let back_text = document_round_trip(SLOT_PAINT, &text);
        assert_eq!(back_text, text, "the archive must not rewrite the document");
        let back: PaintDoc = serde_json::from_str(&back_text).expect("PaintDoc parses back");
        assert_eq!(back.gw, 4);
        assert_eq!(back.gh, 3);
        assert_eq!(back.biome, doc.biome);
        assert!(back.terrain.is_empty());
        assert_eq!(back.splat, doc.splat);
        // The pair list really is a decodable layer and not just a list of
        // numbers that happens to survive: `decode_sparse` is the reader
        // half `_paintSyncFromState` uses.
        let layer = cartalith_spatial::PaintLayer::decode_sparse(&back.biome, 12);
        assert_eq!(layer.cells().expect("a painted layer allocates")[0], 5);
        assert_eq!(layer.cells().expect("allocated")[7], 2);
        assert_eq!(layer.cells().expect("allocated")[11], 13);
        assert_eq!(layer.cells().expect("allocated")[1], 0);
    }

    /// Protects: painted cells from a live `PaintEditor`, written through the paint document and
    /// a real archive, come back cell for cell via `restore_layers`, including a gated water hole
    /// and an unallocated layer.
    /// The round trip that was missing: **painted cells out of a live editor,
    /// through the archive, and back into a live editor.**
    ///
    /// The archive-level test above proves the document's bytes survive; this
    /// one proves the two halves are inverses, which is the part that was
    /// never true. `paint_document_json` shipped on 2026-08-31 with nothing
    /// to read it, so every painted cell was written and dropped on the next
    /// open. Reverting `PaintEditor::restore_layers` fails this test at the
    /// `layer_cells` comparison, not at the JSON.
    ///
    /// The encode side is `paint_document_json`'s own loop rather than a
    /// literal, for the reason the asset-library test gives: a literal would
    /// pin this file's idea of the shape instead of the writer's.
    #[test]
    fn painted_layers_come_back_out_of_a_real_archive() {
        use crate::paint_bridge::{PaintEditor, PaintTarget};
        /// Grid width of the painted-layers fixture: a labelled judgement, small enough to read
        /// by eye yet wide enough for a 2-cell brush beside a water cell.
        const GW: usize = 8;
        /// Grid height of the painted-layers fixture: a labelled judgement, paired with `GW`.
        const GH: usize = 6;
        let n = GW * GH;

        // One cell of the 48 is water, and the brush is land-only, so the
        // fixture also carries a *hole* -- a layer that is uniformly painted
        // would round-trip even if `decode_sparse` mis-indexed.
        let mut mask = vec![0u8; n];
        mask[2 * GW + 2] = 2;
        let mut src = PaintEditor::new(GW, GH, mask.clone().into());
        src.set_brush(3, 2.0, 1.0, 0.0, false, true);
        src.stroke_at(2.0, 2.0);
        src.set_layer(PaintTarget::Splat);
        src.set_brush(2, 1.0, 1.0, 0.0, false, true);
        src.stroke_at(6.0, 4.0);
        src.commit_all(n);

        let encode = |e: &PaintEditor, t: PaintTarget| -> Vec<u32> {
            e.layer_cells(t)
                .map(|cells| {
                    let mut out = Vec::new();
                    for (i, &v) in cells.iter().enumerate() {
                        if v != 0 {
                            out.push(i as u32);
                            out.push(u32::from(v));
                        }
                    }
                    out
                })
                .unwrap_or_default()
        };
        let doc = PaintDoc {
            gw: GW,
            gh: GH,
            biome: encode(&src, PaintTarget::Biome),
            terrain: encode(&src, PaintTarget::Terrain),
            splat: encode(&src, PaintTarget::Splat),
        };
        assert!(
            !doc.biome.is_empty() && !doc.splat.is_empty(),
            "the fixture must actually paint something on two layers"
        );
        assert!(
            doc.terrain.is_empty(),
            "and leave the third unallocated, so the empty case travels too"
        );

        let text = serde_json::to_string(&doc).expect("PaintDoc serializes");
        let back_text = document_round_trip(SLOT_PAINT, &text);
        let back: PaintDoc = serde_json::from_str(&back_text).expect("PaintDoc parses back");

        let mut dst = PaintEditor::new(GW, GH, mask.into());
        let counts = dst.restore_layers(n, &back.biome, &back.terrain, &back.splat);

        for t in [PaintTarget::Biome, PaintTarget::Terrain, PaintTarget::Splat] {
            assert_eq!(
                dst.layer_cells(t),
                src.layer_cells(t),
                "{t:?} must come back cell for cell"
            );
        }
        // ...and the gated cell really is a hole in what came back, so the
        // comparison above is not two identical blanks.
        assert_eq!(
            dst.layer_cells(PaintTarget::Biome).expect("biome allocated")[2 * GW + 2],
            0,
            "the water cell the brush was gated out of must still be unpainted"
        );
        assert_eq!(counts.1, 0, "an unallocated layer restores as zero painted cells");
        assert!(counts.0 > 0 && counts.2 > 0);
    }

    /// Protects: ET-D3's backward-compatibility promise -- a `drafts/sculpt.json`
    /// written BEFORE `pressure` existed (this literal is the exact member set
    /// and order the pre-ET-D3 writer produced) opens, parses to a stamp whose
    /// pressure is `None`, and re-serialises to the **same bytes**: no `pressure`
    /// member appears, not even `null`.
    #[test]
    fn an_old_sculpt_draft_without_pressure_reserialises_byte_identically() {
        let old = concat!(
            r#"{"gw":4,"gh":3,"seed":909,"feature":"mountains","globals":{"brush_size":24.0},"params":{"mtnHeight":0.31},"sub_mode":"","#,
            r#""stamps":[{"feature":"mountains","seed":11,"sea_level":0.42,"hidden":false,"points":[[1.0,1.0],[2.5,1.5]],"#,
            r#""globals":{"brush_size":24.0},"params":{"mtnHeight":0.31},"sub_mode":""}]}"#
        );
        let doc: SculptDoc = serde_json::from_str(old).expect("a pre-ET-D3 draft parses");
        assert_eq!(doc.stamps.len(), 1, "fixture: the stamp parsed (not a silently empty doc)");
        let stamp = sculpt_stamp_from_dto(&doc.stamps[0]).expect("mountains is a feature");
        assert_eq!(stamp.pressure, None, "an old draft has no pressure");
        assert_eq!(stamp.points.len(), 2);
        let again = serde_json::to_string(&doc).expect("SculptDoc serializes");
        assert_eq!(again, old, "no pressure member may be written for a stamp without pressure");
        assert!(!again.contains("pressure"));
    }

    /// Protects: a stamp's pressure surviving the real save/restore code paths
    /// (`sculpt_stamp_to_dto` -> JSON text -> `sculpt_stamp_from_dto`, the two
    /// helpers `sculpt_document_json` and `sculpt_apply_doc` call) as an EQUAL
    /// vector, with literal values that are exact in f32 (`0.25`, `0.75`); and a
    /// malformed vector (wrong length, or a non-finite value) restoring as ABSENT,
    /// never repaired.
    #[test]
    fn a_stamps_pressure_round_trips_and_a_malformed_vector_restores_as_absent() {
        use cartalith_terrain::sculpt::{Feature, Point, SculptStamp};
        let stamp = SculptStamp::new(Feature::River, 5, vec![Point::new(1.0, 1.0), Point::new(2.5, 1.5)], 0.4)
            .with_pressure(vec![0.25, 0.75]);
        assert!(stamp.pressure.is_some(), "fixture: valid pressure accepted");
        let doc = SculptDoc { gw: 4, gh: 3, stamps: vec![sculpt_stamp_to_dto(&stamp, false)], ..SculptDoc::default() };
        let text = serde_json::to_string(&doc).unwrap();
        assert!(text.contains(r#""pressure":[0.25,0.75]"#), "the member is written: {text}");
        let back: SculptDoc = serde_json::from_str(&text).unwrap();
        let restored = sculpt_stamp_from_dto(&back.stamps[0]).unwrap();
        assert_eq!(restored.pressure, Some(vec![0.25, 0.75]));
        assert_eq!(restored, stamp, "the whole stamp, pressure included, is equal after a round trip");

        // A short vector (one value, two points) and a run holding a NaN-as-null.
        let mut bad = back.stamps[0].clone();
        bad.pressure = Some(vec![0.25]);
        assert_eq!(sculpt_stamp_from_dto(&bad).unwrap().pressure, None, "wrong length restores as absent");
        bad.pressure = Some(vec![0.25, f32::NAN]);
        assert_eq!(sculpt_stamp_from_dto(&bad).unwrap().pressure, None, "a non-finite value restores as absent");
    }

    /// Protects: `drafts/sculpt.json` crosses a real archive verbatim and each stamp's feature,
    /// params, globals and `sub_mode` rebuild into the same sculpt parameters (a renamed control
    /// or a lost `sub_mode` would show).
    #[test]
    fn a_sculpt_draft_survives_a_real_archive_round_trip() {
        use cartalith_terrain::sculpt::{Feature, FreehandMode};
        // Two stamps of *different* features, one hidden, plus a non-default
        // control on each -- a single-feature fixture would pass even if
        // `sculpt_params_from_map` ignored the feature key entirely.
        let mut params = BTreeMap::new();
        params.insert("mtnHeight".to_string(), 0.31);
        let mut globals = BTreeMap::new();
        globals.insert("brush_size".to_string(), 24.0);
        let mut free_params = BTreeMap::new();
        free_params.insert("amount".to_string(), 0.2);
        let doc = SculptDoc {
            gw: 4,
            gh: 3,
            seed: 909,
            feature: "mountains".to_string(),
            globals: globals.clone(),
            params: params.clone(),
            sub_mode: String::new(),
            stamps: vec![
                SculptStampDto {
                    feature: "mountains".to_string(),
                    seed: 11,
                    sea_level: 0.42,
                    hidden: false,
                    points: vec![[1.0, 1.0], [2.5, 1.5]],
                    globals: globals.clone(),
                    params: params.clone(),
                    sub_mode: String::new(),
                    pressure: None,
                },
                SculptStampDto {
                    feature: "freehand".to_string(),
                    seed: 12,
                    sea_level: 0.42,
                    hidden: true,
                    points: vec![[0.0, 0.0]],
                    globals: BTreeMap::new(),
                    params: free_params,
                    sub_mode: "lower".to_string(),
                    pressure: None,
                },
            ],
        };
        let text = serde_json::to_string(&doc).expect("SculptDoc serializes");
        let back_text = document_round_trip(SLOT_SCULPT, &text);
        assert_eq!(back_text, text, "the archive must not rewrite the document");
        let back: SculptDoc = serde_json::from_str(&back_text).expect("SculptDoc parses back");
        assert_eq!(back.seed, 909);
        assert_eq!(back.stamps.len(), 2);
        assert!(back.stamps[1].hidden);
        assert_eq!(back.stamps[0].points, vec![[1.0, 1.0], [2.5, 1.5]]);

        // ...and the recipe really rebuilds. This is the half that rots:
        // the map is keyed by control name, and a renamed control would
        // round-trip the *document* perfectly while silently rebuilding a
        // default stamp.
        let rebuilt = sculpt_params_from_map(
            Feature::from_key(&back.stamps[0].feature).expect("mountains is a feature"),
            &back.stamps[0].params,
            &back.stamps[0].sub_mode,
        );
        assert_eq!(sculpt_params_to_map(&rebuilt).get("mtnHeight"), Some(&0.31));
        assert_eq!(
            sculpt_globals_to_map(&sculpt_globals_from_map(&back.stamps[0].globals))
                .get("brush_size"),
            Some(&24.0)
        );
        let free = sculpt_params_from_map(
            Feature::from_key(&back.stamps[1].feature).expect("freehand is a feature"),
            &back.stamps[1].params,
            &back.stamps[1].sub_mode,
        );
        match free {
            cartalith_terrain::sculpt::FeatureParams::Freehand { sub_mode, amount } => {
                assert_eq!(sub_mode, FreehandMode::Lower, "sub_mode is not a numeric control and would be lost with the params map alone");
                assert!((amount - 0.2).abs() < 1e-12);
            }
            other => panic!("freehand must rebuild as Freehand, got {other:?}"),
        }
    }

    /// Protects: `library/assets.json`, built from a real `to_library_json`, crosses a real
    /// archive verbatim and re-applying it recreates the custom slot by uid (zero items: the
    /// project format has no channel for item pixels).
    #[test]
    fn an_asset_library_survives_a_real_archive_round_trip() {
        // A real `to_library_json` record, not a hand-written literal: the
        // point of the test is that what the *builder* emits is what the
        // *parser* takes back, and a literal would only pin this file's own
        // idea of the shape.
        let mut db = cartalith_assets::AssetDB::new();
        let uid = db.add_custom_slot("Watchtower", Some("Landmarks")).uid.clone();
        assert!(db.add_item(&uid, cartalith_assets::LibraryItem::new("tower-a", "deadbeef")));
        let file = db.to_library_json().expect("a library with an item exports");
        let text = serde_json::to_string(&file).expect("LibraryFile serializes");

        let back_text = document_round_trip(SLOT_ASSETS, &text);
        assert_eq!(back_text, text, "the archive must not rewrite the document");

        let parsed = cartalith_assets::parse_library_json(back_text.as_bytes())
            .expect("what to_library_json wrote must parse");
        let mut restored = cartalith_assets::AssetDB::new();
        let items = restored
            .apply_library_file_with_items(&parsed, &std::collections::HashMap::new());
        // Zero, and deliberately: the item's pixels live at
        // `assetlib/img/0.png`, which the project format has no channel for
        // -- `asset_library_document_json`'s own doc comment. The *slot*
        // comes back, which is what makes the custom slot and its metadata
        // survive a save.
        assert_eq!(items, 0, "no image bytes were supplied, so no item can be restored");
        assert!(
            restored.get(&uid).is_some(),
            "the custom slot itself must be recreated by uid, not merely by name"
        );
    }

    /// Protects: `library/travel.json` carries a custom animal, including the species key
    /// `animal_apply_pairs` cannot set, verbatim through a real archive and rebuilds its name and
    /// load capacity.
    #[test]
    fn a_travel_library_survives_a_real_archive_round_trip() {
        use cartalith_civ::travel_library::AnimalDef;
        // A custom animal duplicated from a stock species: the case where
        // `species_key` is `Some` and therefore the case that would silently
        // stop affecting a computed journey if the document dropped it.
        let mut lib = crate::travel_bridge::TravelLibrary::new();
        let id = lib.fresh_id();
        lib.animals
            .duplicate("donkey", id.clone())
            .expect("donkey is a stock animal");
        {
            let a = lib.animals.get_mut(&id).expect("just duplicated, so custom");
            a.name = "Pack Mule".to_string();
            a.load_capacity_kg = Some(97.5);
        }
        let species = lib.animals.get(&id).expect("present").species_key;
        assert_eq!(species, Some("donkey"), "a duplicate of a stock species inherits its key");

        let doc = TravelDoc {
            animals: lib
                .animals
                .iter()
                .filter(|a| a.origin == cartalith_civ::travel_library::EntryOrigin::Custom)
                .map(|a| TravelEntryDto {
                    id: a.id.clone(),
                    species: a.species_key.unwrap_or_default().to_string(),
                    fields: travel_pairs_to_fields(crate::travel_bridge::animal_to_pairs(a)),
                })
                .collect(),
            vehicles: Vec::new(),
            vessels: Vec::new(),
            presets: Vec::new(),
        };
        let text = serde_json::to_string(&doc).expect("TravelDoc serializes");
        let back_text = document_round_trip(SLOT_TRAVEL, &text);
        assert_eq!(back_text, text, "the archive must not rewrite the document");

        let back: TravelDoc = serde_json::from_str(&back_text).expect("TravelDoc parses back");
        assert_eq!(back.animals.len(), 1);
        assert_eq!(back.animals[0].species, "donkey");

        // Rebuild the entry the way `travel_library_restore_document` does,
        // into a library that has never seen it.
        let mut fresh = crate::travel_bridge::TravelLibrary::new();
        let d = &back.animals[0];
        fresh
            .animals
            .add(AnimalDef::blank(d.id.clone(), String::new()))
            .expect("a fresh library has no custom entries");
        let base = fresh.animals.get(&d.id).cloned().expect("just added");
        let (updated, rejected) =
            crate::travel_bridge::animal_apply_pairs(&base, &travel_fields_to_pairs(&d.fields));
        assert!(rejected.is_empty(), "every key this file writes must be one apply_pairs knows: {rejected:?}");
        *fresh.animals.get_mut(&d.id).expect("custom") = updated;
        let rebuilt = fresh.animals.get(&d.id).expect("present");
        assert_eq!(rebuilt.name, "Pack Mule");
        assert_eq!(rebuilt.load_capacity_kg, Some(97.5));
        // `species_key` is the one field `animal_apply_pairs` cannot set,
        // which is why the DTO carries it separately.
        assert_eq!(rebuilt.species_key, None, "apply_pairs must not have set it");
    }

    /// Protects: `jp_value_to_json` and `json_to_jp_value` are exact inverses for every `JpValue`
    /// shape, keeping `Int` and `Num` apart so an integral float never comes back as an integer
    /// (`GUI_GAP_REGISTER.md` KV-04).
    #[test]
    fn every_jp_value_shape_is_an_exact_json_inverse() {
        // The travel field map's whole fidelity rests on this pair, and its
        // one real hazard is the integer/float split: an `Int(3)` that came
        // back as `Num(3.0)` would re-serialize as `3.0`, which is exactly
        // the coercion `GUI_GAP_REGISTER.md` KV-04 was about.
        for v in [
            journey_bridge::JpValue::Int(3),
            journey_bridge::JpValue::Int(-1),
            journey_bridge::JpValue::Num(0.5),
            journey_bridge::JpValue::Str("blocked".to_string()),
            journey_bridge::JpValue::Str(String::new()),
            journey_bridge::JpValue::Bool(true),
            journey_bridge::JpValue::Bool(false),
        ] {
            let json = jp_value_to_json(&v);
            let back = json_to_jp_value(&json).expect("a value this file emitted must parse back");
            assert_eq!(back, v, "{json}");
        }
        // ...and through real JSON text, which is where a float that happens
        // to be integral would collapse.
        let text = serde_json::to_string(&jp_value_to_json(&journey_bridge::JpValue::Num(2.0)))
            .expect("serializes");
        let parsed: serde_json::Value = serde_json::from_str(&text).expect("parses");
        assert_eq!(
            json_to_jp_value(&parsed),
            Some(journey_bridge::JpValue::Num(2.0)),
            "an integral float must not come back as an Int: {text}"
        );
    }

    /// Protects: the paint, sculpt, assets and travel slots stay registered in `cartalith-io` and
    /// caller-owned, so moving one into `ENGINE_OWNED_SLOTS` is a deliberate, failing change.
    #[test]
    fn the_four_new_slots_are_the_caller_s_and_not_the_engine_s() {
        // The partition assertion above names all six callers; this one
        // names the four constants this file added, so that moving one into
        // `ENGINE_OWNED_SLOTS` has to be deliberate rather than a
        // side effect of adding a builder for it.
        for slot in [SLOT_PAINT, SLOT_SCULPT, SLOT_ASSETS, SLOT_TRAVEL] {
            assert!(
                cartalith_io::DOCUMENT_SLOTS.contains(&slot),
                "{slot} must be a slot the format defines"
            );
            assert!(
                caller_slot_refusal(slot).is_none(),
                "{slot} must stay caller-owned"
            );
        }
    }

    /// Protects: every persisted `CivData` member of `sample_civ` survives a real archive round
    /// trip equal to what was written; the one deliberate exception, `explanations`
    /// (SAVEFILE_COMPAT.md 16.2), comes back empty.
    #[test]
    fn the_civ_layer_survives_a_real_archive_round_trip() {
        let civ = sample_civ();
        let back = round_trip(&civ, 4, 3);

        assert_eq!(back.settlements, civ.settlements);
        assert_eq!(back.ways, civ.ways);
        assert_eq!(back.sea_routes, civ.sea_routes);
        assert_eq!(back.territory, civ.territory);
        assert_eq!(back.provinces, civ.provinces);
        // `Province` derives no `PartialEq`, so this compares what the
        // format actually carries rather than adding a derive to a
        // golden-tested crate for a test's convenience.
        let province_rows = |v: &[cartalith_civ::Province]| -> Vec<(i32, i32, String, usize)> {
            v.iter()
                .map(|p| (p.id, p.faction, p.name.clone(), p.capital_settlement_index))
                .collect()
        };
        assert_eq!(
            province_rows(&back.province_list),
            province_rows(&civ.province_list)
        );
        assert_eq!(back.continents, civ.continents);
        assert_eq!(back.trade_balances, civ.trade_balances);
        assert_eq!(back.water_bodies, civ.water_bodies);
        assert_eq!(back.dens, civ.dens);
        assert_eq!(back.next_tid, civ.next_tid);
        assert_eq!(back.year, civ.year);
        assert_eq!(back.timeline, civ.timeline);
        assert_eq!(back.faction_roster, civ.faction_roster);
        assert_eq!(back.place_extras.0, civ.place_extras.0);
        assert_eq!(back.village_tids, civ.village_tids);
        // The one field that deliberately does not survive
        // (`SAVEFILE_COMPAT.md` §16.2).
        assert!(back.explanations.is_empty());
    }

    /// Protects: Ruling J's town-plan fields write no key while unset, so a project
    /// edited only in the pre-existing five fields saves the same bytes.
    #[test]
    fn an_unset_town_plan_writes_no_key() {
        let e = ExtrasDto { walls: Some(true), ..Default::default() };
        let text = serde_json::to_string(&e).unwrap();
        for k in ["culture", "rules_preset", "variant"] {
            assert!(!text.contains(k), "{k} leaked into {text}");
        }
        let set = ExtrasDto { culture: "venus".into(), variant: 1, ..Default::default() };
        let text = serde_json::to_string(&set).unwrap();
        assert!(text.contains("\"culture\":\"venus\"") && text.contains("\"variant\":1"));
    }

    /// Protects: IN-13 tariffs (Ruling AE) ride `factions.json`: set rows survive a
    /// real archive round trip, an untariffed roster writes no `tariffs`
    /// key at all (so an unedited project's document is unchanged by the
    /// field's existence), and rows `set_tariff` would refuse are dropped
    /// on read rather than trusted.
    #[test]
    fn faction_tariffs_survive_a_real_archive_round_trip() {
        let plain = sample_civ();
        let mut docs = BTreeMap::new();
        civ_documents(&plain, None, &mut docs);
        assert!(
            !docs[SLOT_FACTIONS].contains("tariffs"),
            "no tariff set, no key written"
        );

        let mut civ = sample_civ();
        assert!(civ.faction_roster.set_tariff(2, 5, 0.25));
        assert!(civ.faction_roster.set_tariff(5, 2, 0.75));
        assert!(civ.faction_roster.set_tariff(3, 0, 1.0));
        let back = round_trip(&civ, 4, 3);
        assert_eq!(back.faction_roster, civ.faction_roster);
        assert_eq!(back.faction_roster.tariff_rows().len(), 3);

        // Rows only a hand-edited file could hold.
        let mut bad = sample_civ();
        bad.faction_roster.0[2].tariffs.insert(2, 0.5); // self
        bad.faction_roster.0[2].tariffs.insert(99, 0.5); // no such faction
        bad.faction_roster.0[2].tariffs.insert(3, 1.5); // out of range
        bad.faction_roster.0[0].tariffs.insert(1, 0.5); // Unclaimed levying
        bad.faction_roster.0[4].tariffs.insert(1, 0.2); // the one valid row
        let back = round_trip(&bad, 4, 3);
        assert_eq!(
            back.faction_roster.tariff_rows(),
            vec![cartalith_civ::trade::Tariff { importer: 4, exporter: 1, rate: 0.2 }]
        );
    }

    /// Protects: R1's save format (`entities/religions.json`). Custom
    /// religions, the faction choice that points at one, and `next_id` past a
    /// deleted id all survive a real archive round trip, so a reopened
    /// project never reissues a deleted id.
    #[test]
    fn custom_religions_survive_a_real_archive_round_trip() {
        let mut civ = sample_civ();
        let gone = civ.religions.create("Short-lived", "sun_cult", (9, 9, 9), "").unwrap();
        let id = civ.religions.create("Church of the Tide", "sea_lords", (10, 120, 200), "Harbour rites.").unwrap();
        civ.religion_delete(gone).unwrap();
        let key = cartalith_civ::religion_library::custom_religion_key(id);
        let CivData { faction_roster, religions, .. } = &mut civ;
        assert!(faction_roster.set_religion(2, &key, religions));

        let mut docs = BTreeMap::new();
        civ_documents(&civ, None, &mut docs);
        let doc: ReligionsDoc = serde_json::from_str(&docs[SLOT_RELIGIONS]).unwrap();
        assert_eq!(doc.next_id, 3, "the deleted id 1 is remembered as issued");
        assert_eq!(doc.religions.len(), 1);

        let mut back = round_trip(&civ, 4, 3);
        assert_eq!(back.religions, civ.religions);
        assert_eq!(back.faction_roster, civ.faction_roster);
        assert_eq!(back.faction_roster.0[2].religion, key);
        assert_eq!(back.religions.engine_key(&key), "sea_lords");
        let next = back.religions.create("Third", "old_gods", (0, 0, 0), "").unwrap();
        assert_eq!(next, 3, "never 1: that id was issued before the save");
    }

    /// Protects: FH-R2 R2-3's `axes` member (`SAVEFILE_COMPAT.md` §9.9). Set
    /// overrides survive a real archive round trip; a religion with none
    /// writes no `axes` key, so an R1-format document (the literal below is
    /// the exact shape R1 wrote: `id, name, base_key, colour, notes`) reads
    /// and re-serialises byte-identically; a bad value costs its axis only,
    /// with a warning that names it.
    #[test]
    fn religion_axes_round_trip_and_an_r1_document_is_unchanged() {
        use cartalith_civ::religion_profile::{AxisOverrides, Exclusivity, Locus, Membership};
        let mut civ = sample_civ();
        let id = civ.religions.create("Church of the Tide", "sea_lords", (10, 120, 200), "").unwrap();
        let plain = civ.religions.create("Plain", "sun_cult", (1, 2, 3), "").unwrap();
        let o = AxisOverrides {
            exclusivity: Some(Exclusivity::Exclusive),
            membership: Some(Membership::Birth),
            locus: Some(Locus::Portable),
            ..Default::default()
        };
        civ.religions.set_axes(id, o.clone()).unwrap();
        let mut docs = BTreeMap::new();
        civ_documents(&civ, None, &mut docs);
        let text = &docs[SLOT_RELIGIONS];
        let v: serde_json::Value = serde_json::from_str(text).unwrap();
        assert_eq!(
            v["religions"][0]["axes"],
            serde_json::json!({"exclusivity": "exclusive", "membership": "birth", "locus": ["portable"]}),
            "{text}"
        );
        assert!(v["religions"][1].get("axes").is_none(), "only the edited religion writes axes: {text}");
        let back = round_trip(&civ, 4, 3);
        assert_eq!(back.religions.get(id).unwrap().axes, o);
        assert!(back.religions.get(plain).unwrap().axes.is_empty());

        // An R1-format document, as R1's `ReligionDto` serialised it.
        let r1 = r#"{"next_id":2,"religions":[{"id":1,"name":"Church of the Tide","base_key":"sea_lords","colour":[10,120,200],"notes":"Harbour rites."}]}"#;
        let doc: ReligionsDoc = serde_json::from_str(r1).unwrap();
        assert_eq!(serde_json::to_string(&doc).unwrap(), r1, "an R1 document re-serialises byte for byte");

        // A bad value costs its axis, not the religion or the document.
        let mut data = round_trip_data(&civ, 4, 3, 1.0);
        data.documents.insert(
            SLOT_RELIGIONS.to_string(),
            serde_json::json!({"next_id": 3, "religions": [
                {"id": 1, "name": "Tide", "base_key": "sea_lords", "colour": [1, 1, 1],
                 "axes": {"membership": "birth", "exclusivity": "zealous", "locus": 7}}
            ]}),
        );
        let mut warnings = Vec::new();
        let back = civ_from_project(&data, 12, &mut warnings).unwrap();
        let axes = &back.religions.get(1).unwrap().axes;
        assert_eq!(axes.membership, Some(Membership::Birth));
        assert_eq!((axes.exclusivity, axes.locus), (None, None));
        let w: Vec<&String> = warnings.iter().filter(|m| m.contains("beliefs could not be read")).collect();
        assert_eq!(w.len(), 1, "{warnings:?}");
        assert!(w[0].contains("Tide (exclusivity, locus)"), "{warnings:?}");
    }

    /// Protects: FH-R2 R2-6's `religious_policy` member of `factions.json`
    /// (`SAVEFILE_COMPAT.md` §9.2). A default roster writes no such key (an
    /// unedited roster is byte-identical to the pre-R2-6 document); a set
    /// policy survives a real archive round trip; an unknown key is kept
    /// verbatim; a non-text value reads as plural with a warning.
    #[test]
    fn religious_policy_rides_factions_json_and_is_absent_by_default() {
        let plain = sample_civ();
        let mut docs = BTreeMap::new();
        civ_documents(&plain, None, &mut docs);
        assert!(!docs[SLOT_FACTIONS].contains("religious_policy"), "default writes no key");
        let mut civ = sample_civ();
        assert!(civ.faction_roster.set_field(2, "religious_policy", "tolerant_establishment"));
        civ.faction_roster.0[1].religious_policy = "future_policy".to_string();
        let back = round_trip(&civ, 4, 3);
        assert_eq!(back.faction_roster.0[2].religious_policy, "tolerant_establishment");
        assert_eq!(back.faction_roster.0[1].religious_policy, "future_policy", "unknown keys are preserved");
        assert_eq!(back.faction_roster.0[3].religious_policy, "plural");

        let mut data = round_trip_data(&plain, 4, 3, 1.0);
        let mut doc: serde_json::Value = serde_json::from_str(data.text_of(SLOT_FACTIONS).unwrap()).unwrap();
        doc["factions"][1]["religious_policy"] = serde_json::json!(7);
        data.documents.insert(SLOT_FACTIONS.to_string(), doc);
        let mut warnings = Vec::new();
        let back = civ_from_project(&data, 12, &mut warnings).unwrap();
        assert_eq!(back.faction_roster.0[1].religious_policy, "plural");
        assert_eq!(back.faction_roster.0.len(), plain.faction_roster.0.len(), "the roster still opens");
        assert_eq!(warnings.iter().filter(|w| w.contains("religious policy")).count(), 1, "{warnings:?}");
    }

    /// Protects: R1 is additive. A project saved before custom religions
    /// existed (a real archive, not a hand-built one) opens with an empty
    /// library and **no warning**, and an untouched project writes no
    /// `entities/religions.json` at all -- so its archive is unchanged.
    #[test]
    fn a_project_saved_before_custom_religions_opens_with_none_and_writes_none() {
        let bytes: &[u8] = include_bytes!("../tests/fixtures/project_pre_substrate_2026-09-24.zip");
        let data = project::read_project(std::io::Cursor::new(bytes)).expect("the fixture reads");
        assert!(data.text_of(SLOT_RELIGIONS).is_none(), "premise: written before the slot existed");
        let n = data.save.params.gw * data.save.params.gh;
        let mut warnings = Vec::new();
        let civ = civ_from_project(&data, n, &mut warnings).expect("its civ layer restores");
        assert!(warnings.is_empty(), "{warnings:?}");
        assert!(civ.religions.is_pristine());
        let mut docs = BTreeMap::new();
        civ_documents(&civ, None, &mut docs);
        assert!(!docs.contains_key(SLOT_RELIGIONS), "absent, not empty");
        let mut fresh = BTreeMap::new();
        civ_documents(&sample_civ(), None, &mut fresh);
        assert!(!fresh.contains_key(SLOT_RELIGIONS), "an untouched civ writes none either");
    }

    /// A small colour-paint layer: a flat stroke with sub-metre coordinates
    /// and a lava area, so the save's rounding and the material both show.
    fn sample_colour_paint() -> cartalith_spatial::colour_paint::ColourPaint {
        use cartalith_spatial::colour_paint::*;
        let flat = ColourMark {
            shape: MarkShape::Line { points_km: vec![[100.123456, 200.98765], [140.5, 210.25]], radius_km: 2.000_4 },
            rgb: [200, 30, 18],
            opacity: 0.8,
            hardness: 0.5,
            blend: ColourBlend::Replace,
            land_only: true,
            emissive: None,
        };
        let lava = ColourMark {
            shape: MarkShape::Area { ring_km: vec![[300.0, 300.0], [320.000_7, 300.0], [310.0, 318.3]], edge_km: 0.25 },
            rgb: [34, 33, 38],
            opacity: 1.0,
            hardness: 1.0,
            blend: ColourBlend::Replace,
            land_only: false,
            emissive: Some(EmissiveMaterial { ramp: GlowRamp::Fire, heat: 0.75, crack_density: 0.5, glow_radius_km: 1.5, heat_along: 0.0 }),
        };
        ColourPaint::new(ColourPaintDoc { version: DOC_VERSION, visible: false, marks: vec![flat, lava] }, 1.0).unwrap()
    }

    /// Writes a real archive holding `docs` beside the civ documents and reads it back.
    fn round_trip_with(docs: BTreeMap<String, String>) -> (cartalith_io::ProjectData, usize) {
        let (gw, gh) = (4usize, 3usize);
        let n = gw * gh;
        let params = cartalith_io::SaveParams { gw, gh, seed: 4242, map_width_km: 800.0, sea_level: 0.42, world: false, origin: None, name: None };
        let fields = cartalith_io::SaveFields {
            heightmap: std::sync::Arc::new(vec![0.5; n]),
            temperature: std::sync::Arc::new(vec![10.0; n]),
            rainfall: std::sync::Arc::new(vec![1.0; n]),
            volcanic_field: vec![0.0; n],
            impact_field: vec![0.0; n],
            strahler_order: vec![0; n],
        };
        let mut write = ProjectWrite::new(&params, &fields);
        write.documents = docs;
        let mut buf = Vec::new();
        project::write_project(std::io::Cursor::new(&mut buf), &write).expect("write_project should succeed");
        let data = cartalith_io::read_project(std::io::Cursor::new(&buf)).expect("read_project should succeed");
        assert!(data.warnings.is_empty(), "{:?}", data.warnings);
        (data, buf.len())
    }

    /// Protects CP-3 (`COLOUR_PAINT_SCOPE.md` §9): the colour paint survives a
    /// real archive round trip -- marks, material, the hidden flag -- equal
    /// to the metre-rounded document written; re-saving the opened document
    /// writes the same bytes (the rounding is idempotent); and the slot is
    /// engine-owned (a caller cannot write it). Prints the archive size with
    /// and without the paint (the size delta).
    #[test]
    fn colour_paint_survives_a_real_archive_round_trip() {
        let layer = sample_colour_paint();
        let saved = colour_paint_save_doc(Some(&layer)).expect("a layer with marks is saved");
        if let cartalith_spatial::colour_paint::MarkShape::Line { points_km, radius_km } = &saved.layers[0].marks[0].shape {
            assert_eq!(points_km[0], [100.123, 200.988], "metre-rounded");
            assert_eq!(*radius_km, 2.0);
        } else {
            panic!("the first mark is a stroke");
        }
        let mut docs = BTreeMap::new();
        insert_doc(&mut docs, SLOT_COLOUR_PAINT, &saved);
        let text = docs[SLOT_COLOUR_PAINT].clone();
        let (data, with_len) = round_trip_with(docs);
        let (_, without_len) = round_trip_with(BTreeMap::new());
        let mut warnings = Vec::new();
        let back = colour_paint_from_data(&data, &mut warnings).expect("restored");
        assert!(warnings.is_empty(), "{warnings:?}");
        assert_eq!(back, saved);
        assert!(!back.visible && back.layers[0].marks[1].emissive.is_some(), "the hidden flag and the material come back");
        let reopened = cartalith_spatial::colour_paint::ColourPaint::from_layers(back, 1.0).unwrap();
        let mut again = BTreeMap::new();
        insert_doc(&mut again, SLOT_COLOUR_PAINT, &colour_paint_save_doc(Some(&reopened)).unwrap());
        assert_eq!(again[SLOT_COLOUR_PAINT], text, "re-saving an opened project writes the same bytes");
        assert!(caller_slot_refusal(SLOT_COLOUR_PAINT).is_some(), "engine-owned: a caller may not write it");
        eprintln!("archive with the 2-mark colour paint {with_len} bytes, without {without_len} bytes (+{})", with_len as i64 - without_len as i64);
    }

    /// Protects CP-3's additivity: a real archive written before the slot
    /// existed opens with no paint and no warning; a project with no layer, or
    /// an empty one, writes no slot at all (absent, not empty).
    #[test]
    fn a_project_without_colour_paint_opens_and_saves_without_the_slot() {
        let bytes: &[u8] = include_bytes!("../tests/fixtures/project_pre_substrate_2026-09-24.zip");
        let data = project::read_project(std::io::Cursor::new(bytes)).expect("the fixture reads");
        assert!(data.text_of(SLOT_COLOUR_PAINT).is_none(), "premise: written before the slot existed");
        let mut warnings = Vec::new();
        assert!(colour_paint_from_data(&data, &mut warnings).is_none());
        assert!(warnings.is_empty(), "{warnings:?}");
        assert!(colour_paint_save_doc(None).is_none(), "no layer, no slot");
        let empty = cartalith_spatial::colour_paint::ColourPaint::new(cartalith_spatial::colour_paint::ColourPaintDoc { version: 1, visible: true, marks: vec![] }, 1.0).unwrap();
        assert!(colour_paint_save_doc(Some(&empty)).is_none(), "an empty layer, no slot");
        // Ruling CS: named paint layers that hold no marks still write nothing.
        let mut named = cartalith_spatial::colour_paint::PaintLayers::empty();
        named.add_layer(Some("Lava")).unwrap();
        named.add_layer(None).unwrap();
        let named = cartalith_spatial::colour_paint::ColourPaint::from_layers(named, 1.0).unwrap();
        assert!(colour_paint_save_doc(Some(&named)).is_none(), "layers without marks, no slot");
    }

    /// Protects Ruling CS's backward compatibility at the archive: a
    /// version-1 member -- written here through `ColourPaintDoc`, the type
    /// CP-3's save serialised and which the layers change left untouched --
    /// opens as one "Paint 1" layer, and saving it unchanged writes the
    /// member's **same bytes** (version 1, not 2). A layered document (a
    /// hidden layer, a locked one, a rename) writes version 2 and survives
    /// a real archive round trip, hidden and locked layers included, and
    /// re-saves byte-identically.
    #[test]
    fn colour_paint_layers_round_trip_and_version_one_resaves_its_own_bytes() {
        use cartalith_spatial::colour_paint::*;
        // What CP-3's save wrote: the metre-rounded document, as a
        // `ColourPaintDoc` (the flattened view of a one-layer document is it).
        let rounded = colour_paint_save_doc(Some(&sample_colour_paint())).unwrap();
        let v1 = ColourPaint::from_layers(rounded, 1.0).unwrap().doc().clone();
        let mut docs = BTreeMap::new();
        insert_doc(&mut docs, SLOT_COLOUR_PAINT, &v1);
        let v1_text = docs[SLOT_COLOUR_PAINT].clone();
        assert!(v1_text.contains("\"version\": 1") && v1_text.contains("\"marks\""), "premise: a version-1 member");
        let (data, _) = round_trip_with(docs);
        let mut warnings = Vec::new();
        let opened = colour_paint_from_data(&data, &mut warnings).expect("restored");
        assert!(warnings.is_empty(), "{warnings:?}");
        assert_eq!(opened.layers.len(), 1);
        assert_eq!(opened.layers[0].name, DEFAULT_LAYER_NAME);
        let layer = ColourPaint::from_layers(opened, 1.0).unwrap();
        let mut again = BTreeMap::new();
        insert_doc(&mut again, SLOT_COLOUR_PAINT, &colour_paint_save_doc(Some(&layer)).unwrap());
        assert_eq!(again[SLOT_COLOUR_PAINT], v1_text, "an unchanged version-1 project re-saves its own bytes");

        let mut layered = layer.layers().clone();
        layered.add_layer(Some("Hidden lava")).unwrap();
        // The sample's unrounded lava mark, so the save must round the
        // second layer's marks too (not only the first's).
        let lava = sample_colour_paint().doc().marks[1].clone();
        layered.add_mark(1, lava).unwrap();
        layered.set_layer_visible(1, false).unwrap();
        layered.rename(0, "Base").unwrap();
        layered.set_layer_locked(0, true).unwrap();
        let layer = ColourPaint::from_layers(layered, 1.0).unwrap();
        assert_eq!((layer.len(), layer.drawn_len()), (3, 2), "premise: the hidden layer's mark is not drawn");
        let saved = colour_paint_save_doc(Some(&layer)).expect("saved");
        let mut docs = BTreeMap::new();
        insert_doc(&mut docs, SLOT_COLOUR_PAINT, &saved);
        let text = docs[SLOT_COLOUR_PAINT].clone();
        assert!(text.contains("\"version\": 2") && text.contains("\"Hidden lava\"") && text.contains("\"locked\": true"), "{text}");
        if let MarkShape::Area { ring_km, .. } = &saved.layers[1].marks[0].shape {
            assert_eq!(ring_km[1], [320.001, 300.0], "every layer's marks are metre-rounded");
        } else {
            panic!("the hidden layer holds the lava area");
        }
        let (data, _) = round_trip_with(docs);
        let back = colour_paint_from_data(&data, &mut warnings).expect("restored");
        assert!(warnings.is_empty(), "{warnings:?}");
        assert_eq!(back, saved, "every layer, its name, visibility, lock and marks come back");
        let reopened = ColourPaint::from_layers(back, 1.0).unwrap();
        let mut again = BTreeMap::new();
        insert_doc(&mut again, SLOT_COLOUR_PAINT, &colour_paint_save_doc(Some(&reopened)).unwrap());
        assert_eq!(again[SLOT_COLOUR_PAINT], text, "a layered project re-saves its own bytes");
    }

    /// Protects: a damaged colour-paint member costs only itself -- not a
    /// document, an unknown version, an unknown blend -- each a warning and no
    /// paint, never a failed open or a guessed layer.
    #[test]
    fn a_damaged_colour_paint_member_warns_and_opens_without_paint() {
        let (mut data, _) = round_trip_with(BTreeMap::new());
        for bad in [
            serde_json::json!([1, 2, 3]),
            serde_json::json!({"version": 2, "visible": true, "marks": []}),
            serde_json::json!({"version": 1, "visible": true, "marks": [{"shape": {"kind": "line", "points_km": [[0, 0]], "radius_km": 1}, "rgb": [1, 2, 3], "opacity": 1, "hardness": 1, "blend": "glow"}]}),
        ] {
            data.documents.insert(SLOT_COLOUR_PAINT.to_string(), bad.clone());
            let mut warnings = Vec::new();
            assert!(colour_paint_from_data(&data, &mut warnings).is_none(), "{bad}");
            assert_eq!(warnings.len(), 1, "{bad}: {warnings:?}");
        }
    }

    /// Protects: damage costs only itself. A faction pointing at a custom
    /// religion the archive no longer holds opens (warned, key kept, read as
    /// no religion); an unreadable religions document is a warning, not a
    /// failed open; and a bad row is dropped while the good one survives.
    #[test]
    fn damaged_custom_religion_data_degrades_honestly_on_open() {
        let scale = cartalith_engine::WorldParams::defaults(4, 3, 4242).civ.garrison_exposure_scale;
        // 1. A dangling reference: the faction names custom:5, the library is empty.
        let mut civ = sample_civ();
        civ.faction_roster.0[2].religion = "custom:5".to_string();
        let data = round_trip_data(&civ, 4, 3, scale);
        let mut warnings = Vec::new();
        let back = civ_from_project(&data, 12, &mut warnings).unwrap();
        assert_eq!(back.faction_roster.0[2].religion, "custom:5", "kept, never re-pointed");
        assert_eq!(back.religions.engine_key("custom:5"), "none");
        // Only religion warnings are this test's business: this fixture writes
        // no claim rasters, which `civ_from_project` reports separately.
        let ours = |w: &Vec<String>| -> Vec<String> {
            w.iter().filter(|m| m.contains("religion")).cloned().collect()
        };
        assert_eq!(ours(&warnings).len(), 1, "{warnings:?}");
        assert!(ours(&warnings)[0].contains("custom:5"));

        // 2. The document present but not a religions document.
        let mut civ = sample_civ();
        civ.religions.create("Good", "sun_cult", (1, 1, 1), "").unwrap();
        let mut data = round_trip_data(&civ, 4, 3, scale);
        data.documents.insert(SLOT_RELIGIONS.to_string(), serde_json::json!([1, 2, 3]));
        let mut warnings = Vec::new();
        let back = civ_from_project(&data, 12, &mut warnings).expect("the rest of the archive opens");
        assert!(back.religions.is_pristine());
        assert_eq!(ours(&warnings).len(), 1, "{warnings:?}");

        // 3. One bad row (unknown base) beside a good one.
        data.documents.insert(
            SLOT_RELIGIONS.to_string(),
            serde_json::json!({"next_id": 4, "religions": [
                {"id": 1, "name": "Good", "base_key": "sun_cult", "colour": [1, 1, 1]},
                {"id": 2, "name": "Bad", "base_key": "cargo_cult", "colour": [1, 1, 1]},
                {"id": 3, "name": "Future", "base_key": "sun_cult", "colour": [1, 1, 1], "axes": {"x": 1}}
            ]}),
        );
        let mut warnings = Vec::new();
        let back = civ_from_project(&data, 12, &mut warnings).unwrap();
        let ids: Vec<u32> = back.religions.defs().iter().map(|d| d.id).collect();
        assert_eq!(ids, vec![1, 3], "an unknown axis costs nothing but itself, an unknown base costs its row");
        assert!(back.religions.get(3).unwrap().axes.is_empty(), "an unknown axis reads as its base");
        assert_eq!(back.religions.next_id(), 4);
        // CHANGED 2026-10-24 (FH-R2 R2-3): 1 -> 2. `axes` became a real member,
        // so an axis this build does not know ("x") is now named in its own
        // warning (it would be lost on the next save) instead of passing
        // silently; the unknown-base row's warning is the other.
        assert_eq!(ours(&warnings).len(), 2, "{warnings:?}");
        assert!(ours(&warnings).iter().any(|w| w.contains("Future (x)")), "{warnings:?}");
    }

    /// Protects: Ruling AU's currencies ride `factions.json` (`SAVEFILE_COMPAT.md`
    /// §9.2): set members survive a real archive round trip, an unset
    /// currency writes no key at all, a partly-set one writes only what was
    /// set, and a member `set_currency` would refuse is dropped on read with
    /// a warning rather than trusted or replaced by a par rate.
    #[test]
    fn faction_currencies_survive_a_real_archive_round_trip() {
        let plain = sample_civ();
        let mut docs = BTreeMap::new();
        civ_documents(&plain, None, &mut docs);
        assert!(!docs[SLOT_FACTIONS].contains("currency"), "nothing set, no key written");

        let mut civ = sample_civ();
        assert!(civ.faction_roster.set_currency(2, "name", "Aurelian denarius"));
        assert!(civ.faction_roster.set_currency(2, "symbol", "dn"));
        assert!(civ.faction_roster.set_currency(2, "rate", "12.5"));
        assert!(civ.faction_roster.set_currency(3, "rate", "0.25"));
        let mut docs = BTreeMap::new();
        civ_documents(&civ, None, &mut docs);
        let doc: serde_json::Value = serde_json::from_str(&docs[SLOT_FACTIONS]).unwrap();
        assert_eq!(
            doc["factions"][2]["currency"],
            serde_json::json!({"name": "Aurelian denarius", "symbol": "dn", "rate": 12.5})
        );
        assert_eq!(
            doc["factions"][3]["currency"],
            serde_json::json!({"rate": 0.25}),
            "only the member set is written"
        );
        assert!(doc["factions"][1].get("currency").is_none(), "an unset faction writes no key");
        let back = round_trip(&civ, 4, 3);
        assert_eq!(back.faction_roster, civ.faction_roster);
        assert_eq!(back.faction_roster.0[2].currency.rate, Some(12.5));
        assert_eq!(back.faction_roster.0[2].currency.symbol.as_deref(), Some("dn"));
        assert_eq!(back.faction_roster.0[3].currency.name, None, "unset stays unset");
        assert_eq!(back.faction_roster.currency_rate(3), Some((0.25, false)));
        assert_eq!(back.faction_roster.currency_rate(4), Some((1.0, true)), "unset reads as the default");

        // Members only a hand-edited file could hold.
        let mut warnings = Vec::new();
        for (json, want_rate) in [
            (r#"{"id":2,"currency":{"rate":0}}"#, None),
            (r#"{"id":2,"currency":{"rate":-3}}"#, None),
            (r#"{"id":2,"currency":{"rate":4}}"#, Some(4.0)),
        ] {
            let dto: FactionDto = serde_json::from_str(json).unwrap();
            assert_eq!(currency_from_dto(&dto, &mut warnings).rate, want_rate, "{json}");
        }
        assert_eq!(warnings.len(), 2, "each dropped rate says so: {warnings:?}");
        let dto: FactionDto = serde_json::from_str(r#"{"id":0,"currency":{"rate":2}}"#).unwrap();
        assert_eq!(currency_from_dto(&dto, &mut warnings), civ_roster_bridge::FactionCurrency::default());
        let dto: FactionDto =
            serde_json::from_str(r#"{"id":1,"currency":{"name":"  ","symbol":"TOOLONGX"}}"#).unwrap();
        assert_eq!(currency_from_dto(&dto, &mut warnings), civ_roster_bridge::FactionCurrency::default());
    }

    /// Protects: The real pre-currency archive (the same fixture the substrate tests
    /// document the provenance of) opens with every faction's currency
    /// unset, says nothing about it, and re-saves its `factions.json` byte
    /// for byte: an absent member is written back absent.
    #[test]
    fn a_project_saved_before_currencies_opens_with_defaults() {
        let bytes: &[u8] = include_bytes!("../tests/fixtures/project_pre_substrate_2026-09-24.zip");
        let data = project::read_project(std::io::Cursor::new(bytes)).expect("the fixture reads");
        let old = data.text_of(SLOT_FACTIONS).expect("it has a roster").to_string();
        assert!(!old.contains("currency"), "premise: written before the member existed");
        let n = data.save.params.gw * data.save.params.gh;
        let mut warnings = Vec::new();
        let civ = civ_from_project(&data, n, &mut warnings).expect("its civ layer restores");
        assert!(warnings.is_empty(), "{warnings:?}");
        assert!(civ.faction_roster.count() >= 2, "premise: real factions to check");
        for (i, e) in civ.faction_roster.0.iter().enumerate() {
            assert_eq!(e.currency, civ_roster_bridge::FactionCurrency::default(), "faction {i}");
        }
        assert_eq!(civ.faction_roster.currency_rate(1), Some((1.0, true)));
        assert_eq!(civ.faction_roster.currency_rate(0), None, "Unclaimed has no currency");
        let mut docs = BTreeMap::new();
        civ_documents(&civ, None, &mut docs);
        assert_eq!(docs[SLOT_FACTIONS], old, "factions.json re-serialises byte for byte");
    }

    /// Protects: settlement and way `tid`s (7, 8, 21) survive a round trip unrenumbered and the
    /// counter clears them, so vault links and journeys naming a `tid` keep pointing at the same
    /// entity.
    #[test]
    fn a_stable_id_survives_and_that_is_the_whole_point() {
        // `MARKDOWN_VAULT_SCOPE.md` milestone 3 and `STORY_PLANNING_SCOPE.md`
        // SP-1 both turn on exactly this: a link or a journey names an
        // entity by `tid`, and a save that renumbered would silently
        // repoint every one of them.
        let civ = sample_civ();
        let back = round_trip(&civ, 4, 3);
        assert_eq!(back.settlements[0].tid, 7);
        assert_eq!(back.settlements[1].tid, 8);
        assert_eq!(back.ways[0].tid, 21);
        assert!(
            back.next_tid > 21,
            "the id counter must clear every id in the file"
        );
    }

    /// Protects: SAVEFILE_COMPAT.md 9.1 -- a stored `next_id` below the ids beside it is raised
    /// past them (22 here), so the next hand-placed settlement never reuses an existing id.
    #[test]
    fn a_stored_next_id_lower_than_the_ids_beside_it_is_raised() {
        // §9.1. A counter that trailed its own data would hand the next
        // hand-placed settlement an id that already exists -- and every
        // vault link and journey pointing at the original would follow the
        // impostor.
        let mut civ = sample_civ();
        civ.next_tid = 1;
        let back = round_trip(&civ, 4, 3);
        assert_eq!(
            back.next_tid, 22,
            "next_id must clear both settlement 8 and way 21"
        );
    }

    /// Protects: an archive written with no civ documents reopens with no civ layer
    /// (`civ_from_project` is `None`), not an invented empty one.
    #[test]
    fn a_world_with_no_civ_layer_round_trips_to_no_civ_layer() {
        // Not "an empty civ layer": the distinction between "this project
        // has no settlements" and "this project was never populated" is
        // real, and inventing an empty roster would make every loaded
        // terrain look like a depopulated world.
        let n = 12;
        let params = cartalith_io::SaveParams {
            gw: 4,
            gh: 3,
            seed: 1,
            map_width_km: 800.0,
            sea_level: 0.42,
            world: false,
            // Pre-provenance fixture: the archive shape a user's existing
            // save has, so the assertions below cover the absent case.
            origin: None,
            name: None,
        };
        let fields = cartalith_io::SaveFields {
            heightmap: std::sync::Arc::new(vec![0.5; n]),
            temperature: std::sync::Arc::new(vec![10.0; n]),
            rainfall: std::sync::Arc::new(vec![1.0; n]),
            volcanic_field: vec![0.0; n],
            impact_field: vec![0.0; n],
            strahler_order: vec![0; n],
        };
        let mut buf = Vec::new();
        project::write_project(
            std::io::Cursor::new(&mut buf),
            &ProjectWrite::new(&params, &fields),
        )
        .unwrap();
        let data = cartalith_io::read_project(std::io::Cursor::new(&buf)).unwrap();
        assert!(civ_from_project(&data, n, &mut Vec::new()).is_none());
    }

    /// Protects: a civ layer whose entity lists are all emptied still reopens as a civ layer
    /// (roster and territory intact), distinguishable from a project that never had one.
    #[test]
    fn an_empty_civ_layer_is_distinguishable_from_an_absent_one() {
        let mut civ = sample_civ();
        civ.settlements.clear();
        civ.trade_balances.clear();
        civ.ways.clear();
        civ.sea_routes.clear();
        civ.village_tids.clear();
        civ.place_extras.0.clear();
        civ.province_list.clear();
        civ.continents.clear();
        civ.timeline.clear();
        civ.year = 0;
        let back = round_trip(&civ, 4, 3);
        assert!(back.settlements.is_empty());
        assert!(back.ways.is_empty());
        // ...but the roster and the rasters are still there, which is what
        // makes it an empty civilisation rather than no project.
        assert_eq!(back.faction_roster, civ.faction_roster);
        assert_eq!(back.territory, civ.territory);
    }

    /// Protects: SAVEFILE_COMPAT.md 9.1/9.3 -- an unknown settlement tier or road class from a
    /// future version keeps its row and id, falling back to `Town` and `Track` rather than
    /// dropping.
    #[test]
    fn an_unrecognised_vocabulary_value_costs_nothing() {
        // §9.1/§9.3: a tier or a road class from a future version must not
        // drop the settlement or the road.
        let mut doc = SettlementsDoc::default();
        doc.settlements.push(SettlementDto {
            id: 3,
            kind: "arcology".into(),
            ..Default::default()
        });
        let s = dto_to_settlement(&doc.settlements[0]);
        assert_eq!(s.placement.kind, cartalith_civ::SettlementKind::Town);
        assert_eq!(s.tid, 3);

        let road = dto_to_road(&RoadDto {
            id: 4,
            class: "maglev".into(),
            ..Default::default()
        });
        assert_eq!(road.way_type, cartalith_civ::WayType::Track);
        assert_eq!(road.tid, 4);
    }

    /// Protects: a road break index at or past the end of its polyline (2 and 99 on two points)
    /// is dropped on read, so a renderer cannot walk off the polyline.
    #[test]
    fn an_out_of_range_break_is_dropped_not_trusted() {
        // A renderer that walked an out-of-range break would run off the
        // end of the polyline.
        let road = dto_to_road(&RoadDto {
            points: vec![[0.0, 0.0], [1.0, 1.0]],
            breaks: vec![0, 1, 2, 99],
            ..Default::default()
        });
        assert_eq!(road.brks, vec![0, 1]);
    }

    /// Protects: a province whose `capital_settlement_index` names no settlement (99) does not
    /// survive the round trip; the valid province does.
    #[test]
    fn a_province_pointing_at_no_settlement_is_dropped() {
        let mut civ = sample_civ();
        civ.province_list.push(cartalith_civ::Province {
            id: 2,
            faction: 1,
            name: "Nowhere".into(),
            capital_settlement_index: 99,
        });
        let back = round_trip(&civ, 4, 3);
        assert_eq!(
            back.province_list.len(),
            1,
            "the dangling province must not survive"
        );
        assert_eq!(back.province_list[0].id, 1);
    }

    /// Protects: tuned `LandmarkSettings` (cap, armed flag, crowding, class radius,
    /// cross-competition) survive the document round trip, and a store that has never run writes
    /// no `results` member.
    #[test]
    fn landmark_settings_survive_a_document_round_trip() {
        use cartalith_civ::landmark::LandmarkSettings;
        let mut tuned = LandmarkSettings::default();
        // A key that really exists, so the setter accepts it -- taken from
        // the vocabulary rather than typed, so a rename fails this test
        // instead of silently making it vacuous.
        let key = cartalith_civ::landmark::kinds()[0].key.to_string();
        crate::landmark_bridge::set_cap(&mut tuned, &key, 17);
        let flipped = !tuned.is_armed(&key);
        crate::landmark_bridge::set_armed(&mut tuned, &key, flipped);
        crate::landmark_bridge::set_crowding(&mut tuned, 2.25);
        crate::landmark_bridge::set_class_radius(&mut tuned, "local", 3.5);
        crate::landmark_bridge::set_cross_competition(&mut tuned, false);
        assert_ne!(tuned, LandmarkSettings::default(), "the fixture must differ from the default, or this proves nothing");

        let doc = LandmarksDoc {
            settings: LandmarkSettingsDto::from(&tuned),
            results: None,
            viewshed_refined: None,
        };
        let text = serde_json::to_string(&doc).expect("serializes");
        // A store that has never run writes no `results` member at all --
        // "it has not run" and "it ran and placed nothing" are different
        // claims and the format keeps them different.
        assert!(!text.contains("results"), "an empty run reached the document: {text}");
        let back: LandmarksDoc = serde_json::from_str(&text).expect("parses");
        assert_eq!(back.settings.into_settings(), tuned);
        assert!(back.results.is_none());
    }

    /// Protects: a real landmark run survives the document -- ids, kinds, positions, causal
    /// chains and seeds exact, floats within one ulp, funnel arithmetic intact and in `kinds()`
    /// order; `rejects` are deliberately not carried.
    /// A real world's landmark pass, through the document and back.
    ///
    /// Generated rather than hand-built: the members under test are a
    /// forty-nine-row funnel table and a couple of hundred placements with
    /// causal chains on them, and a fixture typed by hand would agree with
    /// whatever this file believes rather than with what `generate` emits.
    #[test]
    fn a_landmark_run_survives_a_document_round_trip() {
        use cartalith_civ::landmark::{kinds, LandmarkLimit};

        let (r, seed) = sample_landmark_run();
        assert!(r.landmarks.len() > 50, "only {} placed; a thin fixture proves little", r.landmarks.len());

        let doc = LandmarksDoc {
            settings: LandmarkSettingsDto::from(&cartalith_civ::landmark::LandmarkSettings::default()),
            results: Some(LandmarkRunDto::from(&r)),
            viewshed_refined: None,
        };
        let text = serde_json::to_string_pretty(&doc).expect("serializes");
        let back: LandmarksDoc = serde_json::from_str(&text).expect("parses");
        let out = back.results.expect("the run is in the document").into_result(
            seed,
            SAMPLE_GW,
            SAMPLE_GH,
        );

        assert_eq!(out.landmarks.len(), r.landmarks.len(), "a placement was dropped");
        for (a, b) in out.landmarks.iter().zip(&r.landmarks) {
            // Everything the format stores as an integer or a string comes
            // back exact, **including the seed**, which the archive does not
            // store at all and which `seed_for` rebuilds. The three floats
            // are compared to a tolerance for the reason
            // `an_open_save_cycle_reaches_a_fixed_point` measures.
            assert_eq!(
                (a.id, &a.kind, a.class, a.x, a.y, &a.causal, a.seed),
                (b.id, &b.kind, b.class, b.x, b.y, &b.causal, b.seed),
                "a placement changed across the round trip"
            );
            assert!(within_one_ulp(a.elevation, b.elevation), "{} elevation", a.key());
            assert!(within_one_ulp(a.score, b.score), "{} score", a.key());
            assert!(within_one_ulp(a.importance, b.importance), "{} importance", a.key());
        }
        // The funnels are integers and one token each, so they are exact.
        assert_eq!(out.funnels, r.funnels, "a funnel changed across the round trip");
        assert!(within_one_ulp(out.seconds, r.seconds));
        // The `rejects` list is deliberately not carried; everything else is.
        assert!(out.rejects.is_empty());
        assert!(!r.rejects.is_empty(), "the fixture has no rejects, so the omission is untested");

        // `LandmarkFunnel`'s own documented identity, after the trip.
        for f in &out.funnels {
            assert_eq!(
                f.candidates,
                f.rejected_constraint + f.rejected_score + f.rejected_spacing + f.rejected_cap + f.placed,
                "the funnel arithmetic stopped closing for {}",
                f.kind
            );
        }
        assert_eq!(out.funnels.len(), kinds().len());
        for (f, k) in out.funnels.iter().zip(kinds()) {
            assert_eq!(f.kind, k.key, "the funnel table left `kinds()` order");
            assert_ne!(f.limit, LandmarkLimit::Unrecorded, "{}: a measured run came back unmeasured", f.kind);
        }
        // The id is a position and is re-derived as one.
        for (k, lm) in out.landmarks.iter().enumerate() {
            assert_eq!(lm.id, k as u64 + 1);
        }
        // And the identity the whole exercise is for.
        let keys: std::collections::BTreeSet<String> = out.landmarks.iter().map(|l| l.key()).collect();
        assert_eq!(keys.len(), out.landmarks.len());
        assert_eq!(keys, r.landmarks.iter().map(|l| l.key()).collect());
    }

    /// Protects: SAVEFILE_COMPAT.md 14.1 for `entities/landmarks.json` -- no integer in it
    /// exceeds 2^53 - 1 and the 64-bit `seed` is never written.
    /// §14.1, over a document with a real world's numbers in it. The existing
    /// `every_id_the_format_writes_is_a_safe_integer` walks `civ_documents`
    /// only, so `entities/landmarks.json` was outside it -- and this document
    /// is the one that had a 64-bit hash in reach.
    #[test]
    fn the_landmark_document_writes_no_integer_past_2_53() {
        /// 2^53 - 1, JavaScript's `Number.MAX_SAFE_INTEGER`: the largest integer a JS reader
        /// holds exactly (SAVEFILE_COMPAT.md 14.1).
        const MAX_SAFE: u64 = 9_007_199_254_740_991;
        let (r, _) = sample_landmark_run();
        let doc = LandmarksDoc {
            settings: LandmarkSettingsDto::from(&cartalith_civ::landmark::LandmarkSettings::default()),
            results: Some(LandmarkRunDto::from(&r)),
            viewshed_refined: None,
        };
        let v: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&doc).expect("serializes")).unwrap();
        /// Records the largest unsigned integer found anywhere in `v` into `worst`, descending
        /// arrays and objects.
        fn walk(v: &serde_json::Value, worst: &mut u64) {
            match v {
                serde_json::Value::Number(n) => {
                    if let Some(i) = n.as_u64() {
                        *worst = (*worst).max(i);
                    }
                }
                serde_json::Value::Array(a) => a.iter().for_each(|x| walk(x, worst)),
                serde_json::Value::Object(o) => o.values().for_each(|x| walk(x, worst)),
                _ => {}
            }
        }
        let mut worst = 0u64;
        walk(&v, &mut worst);
        assert!(worst <= MAX_SAFE, "an integer past Number.MAX_SAFE_INTEGER reached the archive: {worst}");
        // And the one that would have: no `seed` member anywhere.
        assert!(
            !serde_json::to_string(&doc).unwrap().contains("\"seed\""),
            "the 64-bit seed reached the format"
        );
    }

    /// Protects: owner ruling 10's non-destruction -- a `landmarks.json` holding only `settings`
    /// still parses, keeps its tuned settings, and re-serialises without inventing a `results`
    /// member.
    /// The non-destruction half of owner ruling 10, and the one this batch
    /// could most easily have broken: a document written **before** the
    /// `results` member existed must still bring its settings back.
    ///
    /// The fixture is the exact text the previous writer produced -- one
    /// `settings` object and nothing else -- rather than an empty document,
    /// because `MISTAKES.md`'s own backward-compatibility rule is that a
    /// fixture of empty collections proves nothing about the installed base.
    #[test]
    fn a_document_written_before_results_existed_keeps_its_settings() {
        use cartalith_civ::landmark::LandmarkSettings;
        let mut tuned = LandmarkSettings::default();
        let key = cartalith_civ::landmark::kinds()[0].key.to_string();
        crate::landmark_bridge::set_cap(&mut tuned, &key, 17);
        crate::landmark_bridge::set_crowding(&mut tuned, 2.25);
        crate::landmark_bridge::set_class_radius(&mut tuned, "local", 3.5);
        crate::landmark_bridge::set_cross_competition(&mut tuned, false);
        assert_ne!(tuned, LandmarkSettings::default());

        // Serialized through the *old* shape: `LandmarkSettingsDto` alone
        // under a `settings` key, which is byte for byte what the writer
        // before this change emitted.
        let legacy = format!(
            "{{\n  \"settings\": {}\n}}",
            serde_json::to_string(&LandmarkSettingsDto::from(&tuned)).unwrap()
        );
        assert!(!legacy.contains("results"));

        let back: LandmarksDoc = serde_json::from_str(&legacy).expect("an older document still parses");
        // Absent is absent: no run is invented for a document that has none.
        assert!(back.results.is_none());
        // Re-serialising it does not grow a `results` member, so opening an
        // old project and saving it does not write a run that never ran.
        let again = serde_json::to_string(&back).unwrap();
        assert!(!again.contains("results"), "{again}");
        assert_eq!(back.settings.into_settings(), tuned, "the settings did not survive");
    }


    /// A refined viewshed fixture with the shape that matters to the RLE: long
    /// zero runs, a run of one, negative-zero's bit pattern kept apart from
    /// `0.0`, and a fractional value that only a lossless `f32` text keeps.
    fn sample_refined() -> cartalith_civ::landmark::RefinedViewshed {
        use cartalith_civ::landmark::{RefinedViewshed, ViewRect};
        let (w, h) = (7usize, 5usize);
        let mut vis = vec![0.0f32; w * h];
        vis[3] = 1.0 / 3.0;
        vis[4] = 1.0 / 3.0;
        vis[5] = 2.5;
        vis[20] = -0.0; // bit-distinct from the 0.0 on both sides of it
        vis[34] = 0.1;
        RefinedViewshed::from_parts(
            ViewRect { x: 11, y: 4, w, h },
            40,
            23,
            0xfedc_ba98_7654_3210,
            0x0123_4567_89ab_cdef,
            vis,
        )
        .expect("a consistent fixture")
    }

    /// Protects: Ruling AT's "saved with the project". A stored refined
    /// viewshed survives encode -> JSON text -> decode **bit-identically** --
    /// rectangle, horizon, observer count, both content keys (as hex, above
    /// 2^53) and every cell including the sign of zero -- and the RLE really
    /// is run-length (a mostly-zero view must not write one entry per cell).
    #[test]
    fn a_refined_viewshed_survives_the_document_bit_for_bit() {
        let rv = sample_refined();
        let doc = LandmarksDoc {
            settings: LandmarkSettingsDto::from(&cartalith_civ::landmark::LandmarkSettings::default()),
            results: None,
            viewshed_refined: Some(encode_viewshed_refined(&rv)),
        };
        let text = serde_json::to_string_pretty(&doc).expect("serializes");
        assert!(text.contains("\"fedcba9876543210\"") && text.contains("\"0123456789abcdef\""), "{text}");
        let back: LandmarksDoc = serde_json::from_str(&text).expect("parses");
        let value = back.viewshed_refined.expect("the member is in the document");
        let cells = value["cells"].as_array().expect("cells is an array");
        assert!(cells.len() < rv.vis.len() / 2, "{} entries for {} cells: not run-length", cells.len(), rv.vis.len());
        let out = decode_viewshed_refined(&value, 64, 64).expect("decodes");
        assert_eq!(out.rect, rv.rect);
        assert_eq!((out.radius_cells, out.observers), (rv.radius_cells, rv.observers));
        assert_eq!((out.terrain_key, out.observer_key), (rv.terrain_key, rv.observer_key));
        let bits = |v: &[f32]| v.iter().map(|x| x.to_bits()).collect::<Vec<_>>();
        assert_eq!(bits(&out.vis), bits(&rv.vis), "a cell changed through the document");
    }

    /// Protects: "absent means never refined; old files re-serialise
    /// byte-identically". A document with no `viewshed_refined` member --
    /// every file written before Ruling AT, and every project never refined --
    /// parses to `None` and writes back **the same bytes**, and a store
    /// without a result writes no such member.
    #[test]
    fn a_document_without_a_refined_viewshed_round_trips_byte_identically() {
        let doc = LandmarksDoc {
            settings: LandmarkSettingsDto::from(&cartalith_civ::landmark::LandmarkSettings::default()),
            results: None,
            viewshed_refined: None,
        };
        let text = serde_json::to_string_pretty(&doc).expect("serializes");
        assert!(!text.contains("viewshed_refined"), "an absent member was written: {text}");
        let back: LandmarksDoc = serde_json::from_str(&text).expect("parses");
        assert!(back.viewshed_refined.is_none());
        assert_eq!(serde_json::to_string_pretty(&back).unwrap(), text, "an old document did not re-serialise identically");
    }

    /// Protects: §6.4a "untrusted input costs the member, not the document".
    /// Each way a stored member can be wrong -- off the grid, a malformed or
    /// short key, a run that is zero, runs that do not sum to `w * h`, a
    /// negative or non-finite weight, a wrong type -- is refused with a
    /// reason, and the same document's `settings` still parse. The positive
    /// control (the unmodified fixture decodes) keeps "everything fails"
    /// from passing as success.
    #[test]
    fn a_malformed_refined_viewshed_costs_only_itself() {
        let good = encode_viewshed_refined(&sample_refined());
        assert!(decode_viewshed_refined(&good, 64, 64).is_ok(), "the control must decode");
        let mutate = |f: &dyn Fn(&mut serde_json::Value)| {
            let mut v = good.clone();
            f(&mut v);
            v
        };
        let bad: Vec<(&str, serde_json::Value, usize, usize)> = vec![
            ("off the grid", good.clone(), 12, 64),
            ("short key", mutate(&|v| v["terrain_key"] = "abc".into()), 64, 64),
            ("non-hex key", mutate(&|v| v["observer_key"] = "zzzzzzzzzzzzzzzz".into()), 64, 64),
            ("zero run", mutate(&|v| v["cells"][0][0] = 0.into()), 64, 64),
            ("runs too long", mutate(&|v| v["cells"][0][0] = 99.into()), 64, 64),
            ("runs too short", mutate(&|v| v["cells"] = serde_json::json!([[3, 0.0]])), 64, 64),
            ("negative weight", mutate(&|v| v["cells"][1][1] = (-1.0).into()), 64, 64),
            ("wrong type", mutate(&|v| v["w"] = "seven".into()), 64, 64),
            ("not an object", serde_json::json!([1, 2, 3]), 64, 64),
        ];
        for (name, v, gw, gh) in &bad {
            assert!(decode_viewshed_refined(v, *gw, *gh).is_err(), "{name} was accepted");
        }
        // The same document with a hostile member still gives its settings back.
        let doc: LandmarksDoc = serde_json::from_str(
            r#"{"settings":{},"viewshed_refined":{"x":"nonsense"}}"#,
        )
        .expect("a bad refined member must not fail the whole document");
        assert!(doc.viewshed_refined.is_some());
        assert!(decode_viewshed_refined(doc.viewshed_refined.as_ref().unwrap(), 64, 64).is_err());
    }

    /// Protects: SAVEFILE_COMPAT.md 6.4a -- an unknown kind or an off-grid coordinate costs its
    /// own row only; the survivor is renumbered from 1, its seed recomputed, and an empty funnel
    /// list is rebuilt as `Unrecorded`.
    /// The three ways a row can be untrusted, each costing the row and not
    /// the document (`SAVEFILE_COMPAT.md` §6.4a).
    #[test]
    fn an_untrusted_landmark_row_costs_the_row() {
        use cartalith_civ::landmark::kinds;
        let good = kinds().iter().find(|k| k.buildable).expect("a buildable kind").key;
        let text = format!(
            r#"{{"settings":{{}},"results":{{"landmarks":[
                {{"kind":"{good}","x":4,"y":5,"elevation_m":100.0,"score":0.5,"importance":0.5}},
                {{"kind":"not_a_landmark_kind","x":6,"y":7}},
                {{"kind":"{good}","x":9000,"y":5}},
                {{"kind":"{good}","x":4,"y":9000}}
            ],"funnels":[],"seconds":0.25}}}}"#
        );
        let doc: LandmarksDoc = serde_json::from_str(&text).expect("parses");
        let out = doc.results.expect("results present").into_result(41, 64, 64);
        assert_eq!(out.landmarks.len(), 1, "{:?}", out.landmarks);
        assert_eq!(out.landmarks[0].key(), format!("{good}@4,5"));
        assert_eq!(out.landmarks[0].id, 1, "the surviving row is renumbered from 1");
        assert_eq!(
            out.landmarks[0].seed,
            cartalith_civ::landmark::seed_for(41, 64, good, 4, 5).expect("a known kind"),
            "the recomputed seed is not the generator's"
        );
        // An empty funnel list is still rebuilt to the full table, and every
        // row says it does not know rather than picking a measured reason.
        assert_eq!(out.funnels.len(), kinds().len());
        for f in &out.funnels {
            assert_eq!(f.limit, cartalith_civ::landmark::LandmarkLimit::Unrecorded, "{}", f.kind);
            assert_eq!(f.candidates, 0);
        }
    }

    /// Protects: `serde_json`'s `float_roundtrip` stays enabled -- landmark floats read back bit
    /// for bit and the document is unchanged by its first read-write cycle.
    /// **An open/save cycle is exact from the first write.** Until 2026-09-24
    /// `serde_json` was built without its `float_roundtrip` feature, so its
    /// parser was the fast one and an `f64` read back could land **one ULP**
    /// from the one written: measured 2026-09-06 on `sample_landmark_run`
    /// (192 x 144, seed 24601), 34 of its 128 placements carried a float that
    /// moved on the first cycle, and the document only became a fixed point
    /// from the second write on. This test asserted that walk.
    ///
    /// It mattered beyond landmarks: `length_km` goes through the same parser,
    /// and owner Ruling AR's reopen round trip found 32 of 191 way lengths one
    /// ULP off, moving `civ_military_summary`'s road density. `cartalith-io`
    /// now takes `float_roundtrip` (its `Cargo.toml` says why), which feature
    /// unification applies to this crate too, so the first read is exact and
    /// **nothing** moves -- which is what is asserted now, so that losing the
    /// feature again turns this red.
    #[test]
    fn an_open_save_cycle_reaches_a_fixed_point() {
        let (r, _) = sample_landmark_run();
        assert!(r.landmarks.len() > 50);
        let t0 = serde_json::to_string(&LandmarkRunDto::from(&r)).unwrap();
        let back: LandmarkRunDto = serde_json::from_str(&t0).unwrap();
        assert_eq!(serde_json::to_string(&back).unwrap(), t0, "the document moved on its first read-write cycle");
        let mut moved = 0usize;
        for (a, b) in back.landmarks.iter().zip(&r.landmarks) {
            assert_eq!((&a.kind, a.x, a.y, &a.causal), (&b.kind, b.x, b.y, &b.causal));
            for (x, y) in [
                (a.elevation_m, b.elevation),
                (a.score, b.score),
                (a.importance, b.importance),
            ] {
                if x.to_bits() != y.to_bits() {
                    moved += 1;
                }
            }
        }
        assert_eq!(moved, 0, "a float came back other than it was written -- is `float_roundtrip` still on?");
    }

    /// `a == b`, or the two are adjacent doubles. Written as a relative
    /// tolerance at `f64::EPSILON` rather than by bit-twiddling because the
    /// values under test are `0..1` scores and metre elevations, where the
    /// two agree.
    fn within_one_ulp(a: f64, b: f64) -> bool {
        a == b || (a - b).abs() <= a.abs().max(b.abs()) * f64::EPSILON
    }

    /// Landmark fixture width: a labelled judgement, large enough that the default run places
    /// more than 50 landmarks (asserted by the tests using it).
    const SAMPLE_GW: usize = 192;
    /// Landmark fixture height, paired with `SAMPLE_GW` (a labelled judgement).
    const SAMPLE_GH: usize = 144;

    /// A real terrain pass with a real landmark run over it, and the world
    /// seed the run was given, so a caller can rebuild the seeds the same way
    /// `project_open` does.
    fn sample_landmark_run() -> (cartalith_civ::landmark::LandmarkResult, u64) {
        use cartalith_civ::landmark::{LandmarkInputs, LandmarkSettings};
        /// Arbitrary fixed world seed for the landmark fixture: a labelled judgement, changed
        /// only if the fixture's asserted premises (more than 50 placements, some rejects) stop
        /// holding.
        const WORLD_SEED: u64 = 24601;
        let p = cartalith_engine::WorldParams::defaults(SAMPLE_GW, SAMPLE_GH, WORLD_SEED as i32);
        let ws = cartalith_engine::generate_terrain(&p);
        let mut inp = LandmarkInputs::new(
            &ws.field,
            SAMPLE_GW,
            SAMPLE_GH,
            ws.sea_level,
            p.world,
            p.map_width_km,
        );
        inp.flow = Some(&ws.flow_discharge);
        inp.channel = ws.channels.as_ref().map(|c| c.chan.as_slice());
        inp.recv = ws.channels.as_ref().map(|c| c.recv.as_slice());
        inp.order = ws.stream_order.as_deref();
        inp.volcanism = Some(&ws.volcanic_field);
        // GF-6: rock formation reads the column's `s` (`RockView`); these
        // parity defaults build no column, so the view refuses and rock
        // formation reports `NoTerrain` in this fixture.
        let rock = cartalith_civ::landmark::RockView::for_world(&ws, &p).ok();
        inp.rock = rock.as_ref();
        let r = cartalith_civ::landmark::generate(&inp, &LandmarkSettings::default(), WORLD_SEED);
        (r, WORLD_SEED)
    }

    /// Protects: an untrusted landmarks `settings` member cannot add an unknown kind's cap,
    /// exceed `CROWDING_MAX`, or hold a negative class radius (floored at zero).
    #[test]
    fn a_landmark_document_cannot_smuggle_in_an_unknown_kind() {
        // An archive is untrusted input. `into_settings` rebuilds from
        // `LandmarkSettings::default()` through the same rejecting setters
        // the `#[func]` surface uses, so a hand-edited `caps` map cannot
        // introduce a fiftieth landmark type -- and an out-of-range crowding
        // is clamped rather than stored.
        let dto: LandmarkSettingsDto = serde_json::from_str(
            r#"{"caps":{"not_a_landmark_kind":9},"crowding":99.0,"class_radius_km":[-4.0]}"#,
        )
        .expect("parses");
        let settings = dto.into_settings();
        assert!(!settings.caps.contains_key("not_a_landmark_kind"));
        assert!(
            settings.crowding <= crate::landmark_bridge::CROWDING_MAX,
            "crowding {} escaped the clamp",
            settings.crowding
        );
        assert_eq!(settings.class_radius_km[0], 0.0, "a negative radius floors at zero");
    }

    /// Protects: every slot registered in `cartalith-io` is exactly one of engine-owned (refused
    /// to callers) or caller-owned, and the caller-owned set is exactly the six named, so a
    /// mis-classified new slot fails here.
    #[test]
    fn the_document_channel_partitions_every_registered_slot() {
        // The partition is the contract, so it is asserted over the whole
        // registry rather than over a sample: every slot the format defines
        // is either the engine's or the caller's, and none is both or
        // neither. A slot added to `cartalith-io` and forgotten here would
        // otherwise arrive as caller-owned by default -- which is the safe
        // direction for `drafts/` and the wrong one for a new entity table.
        for slot in cartalith_io::DOCUMENT_SLOTS {
            assert_eq!(
                caller_slot_refusal(slot).is_some(),
                ENGINE_OWNED_SLOTS.contains(slot),
                "{slot}"
            );
        }
        // The ones the shell may write today, named so that a change to the
        // split has to be deliberate. `annotations/measurements.json` joined
        // on 2026-09-03 and is the one GDScript still writes itself: a
        // saved measurement is a mode, the clicked points and the reading,
        // and `measure_bridge.rs`'s own rule is that the caller owns the
        // points. It is deliberately not in `ENGINE_OWNED_SLOTS`, and this
        // list is what makes moving it there a failing test rather than a
        // silently unwritable store.
        //
        // `entities/journeys.json` was the *second* shell-owned slot until
        // `STORY_PLANNING_SCOPE.md` SP-1 gave it a real engine type
        // (`cartalith_civ::travel_library::Journey`) and moved it into
        // `ENGINE_OWNED_SLOTS` -- see this file's own module doc, "The
        // Journey entity" section.
        //
        // `library/settlement_types.json` joined 2026-09-21
        // (`lazy-riding-piglet.md` Batch D): a settlement type is authored
        // GDScript-only data (`cartalith-io`'s own doc comment on the slot),
        // same shape as the measurements slot, so it stays caller-owned too.
        let callers: Vec<&str> = cartalith_io::DOCUMENT_SLOTS
            .iter()
            .copied()
            .filter(|s| caller_slot_refusal(s).is_none())
            .collect();
        assert_eq!(
            callers,
            vec![
                "annotations/measurements.json",
                "library/assets.json",
                "library/travel.json",
                "library/settlement_types.json",
                "drafts/paint.json",
                "drafts/sculpt.json",
                "appearance_overlay_adjust.json",
                // ET-F3 (2026-10-12): the style-preset variant, GDScript state.
                "appearance_style.json",
            ]
        );
    }

    /// Protects: `caller_slot_refusal` gives distinct messages for an unknown slot ("not a slot")
    /// and an engine-owned one (which points at `get_settlements()`), so a typo is not read as a
    /// permissions rule.
    #[test]
    fn a_slot_that_is_not_a_slot_is_refused_for_a_different_reason() {
        // Two refusals, not one: "you may not have this" and "this does not
        // exist" send the user to different places, and a typo that reported
        // itself as an ownership rule would be read as a permissions bug.
        let unknown =
            caller_slot_refusal("entities/journey.json").expect("a typo must be refused");
        assert!(unknown.contains("not a slot"), "{unknown}");
        let owned = caller_slot_refusal(SLOT_SETTLEMENTS).expect("an engine slot must be refused");
        assert!(owned.contains("get_settlements()"), "{owned}");
        assert!(!owned.contains("not a slot"), "{owned}");
    }

    /// Protects: `resource_key_static` returns a known `CIV_RESOURCE_KEYS` entry unchanged and
    /// `None` for an unknown key.
    #[test]
    fn an_unknown_resource_key_is_dropped_rather_than_leaked() {
        assert_eq!(
            resource_key_static(cartalith_civ::CIV_RESOURCE_KEYS[3]),
            Some(cartalith_civ::CIV_RESOURCE_KEYS[3])
        );
        assert_eq!(resource_key_static("unobtanium"), None);
    }

    /// Protects: the key/from pairs for way class, manual way kind, route mode and label size
    /// mode are exact inverses over every variant, so a road never demotes itself on save.
    #[test]
    fn every_vocabulary_mapping_is_an_exact_inverse() {
        // The two halves live twenty lines apart and are the kind of pair
        // that rots silently -- a wrong direction shows up as a road that
        // quietly demotes itself on every save.
        for t in [
            cartalith_civ::WayType::Highway,
            cartalith_civ::WayType::Regional,
            cartalith_civ::WayType::Road,
            cartalith_civ::WayType::Track,
        ] {
            assert_eq!(way_class_from(way_class_key(t)), t);
        }
        use cartalith_civ::tools::{ManualWayType, RouteMode};
        for t in [
            ManualWayType::Road,
            ManualWayType::Track,
            ManualWayType::SeaLane,
            ManualWayType::Ancient,
        ] {
            assert_eq!(manual_way_kind_from(manual_way_kind_key(t)), t);
        }
        for m in [RouteMode::Land, RouteMode::Water, RouteMode::Mixed] {
            assert_eq!(route_mode_from(route_mode_key(m)), m);
        }
        use cartalith_civ::labels::LabelSizeMode;
        for m in [LabelSizeMode::Zoom, LabelSizeMode::Fixed] {
            assert_eq!(label_size_mode_from(label_size_mode_key(m)), m);
        }
    }

    /// Protects: SAVEFILE_COMPAT.md 13.2 -- an appearance document with no members, or with
    /// unknown ones, parses; the default writes its fixed members and no `biome_cols` (CA-19).
    #[test]
    fn the_appearance_document_is_optional_in_every_member() {
        // §13.2: a reader must be able to ignore this file entirely, and a
        // file written by an older build must not fail because it predates
        // a member. `#[serde(default)]` is what buys both, so it is
        // asserted rather than assumed.
        let doc: AppearanceDoc = serde_json::from_str("{}").expect("an empty appearance document is valid");
        assert!(doc.quality.is_empty());
        assert!(doc.look.is_empty());
        assert!(doc.overrides.is_empty());
        assert!(doc.ramp.is_none());

        let doc: AppearanceDoc = serde_json::from_str(
            r#"{"quality":"Quality","look":"vibrant","territory_opacity":0.32,
                "overrides":{"sun_azimuth":315.0},"unknown_member":[1,2,3]}"#,
        )
        .expect("an unknown member must not fail the document");
        assert_eq!(doc.quality, "Quality");
        assert_eq!(doc.look, "vibrant");
        assert_eq!(doc.territory_opacity, 0.32);
        assert_eq!(doc.overrides["sun_azimuth"], 315.0);

        // The written form really is the slot the format registers.
        assert!(cartalith_io::DOCUMENT_SLOTS.contains(&SLOT_APPEARANCE));
        let text = serde_json::to_string(&AppearanceDoc::default()).unwrap();
        let v: serde_json::Value = serde_json::from_str(&text).unwrap();
        for key in ["quality", "look", "territory_opacity", "overrides", "ramp", "npr"] {
            assert!(v.get(key).is_some(), "appearance.json must carry {key}");
        }
        // CA-19: the default table is spelled by omission, not by a member.
        assert!(v.get("biome_cols").is_none(), "a default table must write no biome_cols: {text}");
    }

    /// Protects: every `ENGINE_OWNED_SLOTS` entry is registered in
    /// `cartalith_io::DOCUMENT_SLOTS`, since an unregistered slot would write and read nothing.
    #[test]
    fn every_slot_this_file_names_is_registered_in_the_format() {
        // A slot named here and not registered in `cartalith-io` writes
        // nothing and reads nothing, silently. Asserted rather than
        // trusted, because the two lists are in different crates.
        for slot in ENGINE_OWNED_SLOTS {
            assert!(
                cartalith_io::DOCUMENT_SLOTS.contains(slot),
                "{slot} is not a registered slot"
            );
        }
    }

    /// Protects: SAVEFILE_COMPAT.md 10.1 backward compatibility -- a timeline written before
    /// per-year `institutions`/`place_overrides` parses with both absent, re-serialises to the
    /// same bytes, and a timeline with no records writes neither key.
    #[test]
    fn a_timeline_written_before_year_records_loads_unrecorded_and_re_serialises_byte_identically() {
        // Protects `SAVEFILE_COMPAT.md` §10.1's backward-compatibility promise for the
        // per-year `institutions` / `place_overrides` members: a `history/timeline.json` in
        // the prior format (the literal below is exactly what the writer emitted before the
        // members existed -- two years, a `territory_year`) parses with both members absent,
        // re-serialises to the SAME BYTES, and a civ written from a timeline with no records
        // emits neither key. If an absent member were ever written as `[]` / `null` this
        // fails, and that is the "no value encoded as a plausible value" mistake.
        let prior = "{\n  \"year\": 412,\n  \"years\": [\n    {\n      \"year\": 0,\n      \"settlements\": [],\n      \"ways\": []\n    },\n    {\n      \"year\": 120,\n      \"settlements\": [],\n      \"ways\": []\n    }\n  ],\n  \"territory_year\": 120\n}";
        let doc: TimelineDoc = serde_json::from_str(prior).unwrap();
        assert!(doc.years.iter().all(|y| y.institutions.is_none() && y.place_overrides.is_none()));
        for y in &doc.years {
            assert_eq!(year_record_from_dtos(y), cartalith_civ::timeline::YearRecord::default());
        }
        assert_eq!(serde_json::to_string_pretty(&doc).unwrap(), prior);

        // And the production writer, over a timeline that has no record, writes neither key.
        let civ = sample_civ();
        assert!(civ.timeline.iter().all(|s| s.record == Default::default()));
        let mut docs = BTreeMap::new();
        civ_documents(&civ, None, &mut docs);
        let text = &docs[SLOT_TIMELINE];
        assert!(!text.contains("institutions") && !text.contains("place_overrides"), "{text}");
    }

    /// Protects: a year's recorded institutions and place overrides survive an archive, with
    /// recorded-with-content, recorded-as-none (an empty map) and unrecorded (`None`) staying
    /// three distinct states.
    #[test]
    fn a_recorded_years_institutions_and_overrides_survive_an_archive_and_stay_distinct_from_none() {
        // Protects the row this change closes: a year's recorded ag-tech/government and
        // place overrides come back from a saved archive as written. Three years separate
        // the three states that must never collapse into one: recorded with content, recorded
        // as "no overrides" (an EMPTY map -- not None), and not recorded at all (None). Built
        // through `civ_add_year`, the production recorder, so the record is the real one.
        let mut civ = sample_civ();
        civ.timeline.clear();
        civ.faction_roster.0[1].government = "monarchy".to_string();
        civ.place_extras.0.clear();
        civ.place_extras.0.insert(
            7,
            civ_roster_bridge::PlaceExtras {
                walls: Some(true),
                age: Some(320),
                traits: vec!["fortified".into()],
                specialisation: "port".into(),
                history: "not part of the record".into(),
                ..Default::default()
            },
        );
        assert!(civ.settlements.iter().any(|s| s.tid == 7), "the fixture must hold tid 7");
        civ.civ_add_year(100);
        let rec100 = civ.timeline[0].record.clone();
        assert!(rec100.place_overrides.as_ref().unwrap().contains_key(&7), "tid 7's edit is recorded");
        civ.civ_add_year(200); // cursor 200; carries 100's record forward
        // Today's overrides vanish; recording the active year 200 (by adding 300) captures
        // "no overrides" for it -- a recorded fact, an EMPTY map, not None.
        civ.place_extras.0.clear();
        civ.civ_add_year(300);
        // A year the collapse run (or any older writer) produced: no record at all.
        civ_snapshot_save_for_test(&mut civ, 400);

        let back = round_trip(&civ, 4, 3);
        let at = |y: i64| back.timeline.iter().find(|s| s.year == y).unwrap().record.clone();
        assert_eq!(at(100), rec100, "year 100 comes back as recorded");
        assert_eq!(
            at(100).institutions.as_ref().unwrap()[1].government,
            "monarchy",
            "the recorded government, not whatever the roster says now"
        );
        assert_eq!(at(400), cartalith_civ::timeline::YearRecord::default(), "unrecorded stays None");
        let r200 = at(200);
        assert!(r200.institutions.is_some());
        assert_eq!(r200.place_overrides, Some(Default::default()), "recorded-none is not unrecorded");
    }

    /// Protects G1-4's bar 1 (`GOVERNMENTS_SCOPE.md` §7.2, §8; Ruling CZ): the
    /// two **real** pre-G1 archives committed as fixtures (written 2026-09-24
    /// and 2026-09-28, before any G1 form existed; `MISTAKES.md`: never a
    /// fixture of empty collections) open with every faction on a reference
    /// government, say nothing about governments, and re-serialise
    /// `entities/factions.json` byte for byte. G1 adds vocabulary, not schema:
    /// no new member, no `format_version` bump.
    #[test]
    fn a_project_saved_before_g1_reopens_and_re_saves_its_roster_byte_identically() {
        for (name, bytes) in [
            ("pre_substrate", &include_bytes!("../tests/fixtures/project_pre_substrate_2026-09-24.zip")[..]),
            ("384cap", &include_bytes!("../tests/fixtures/project_384cap_512x328_2026-09-28.zip")[..]),
        ] {
            let data = project::read_project(std::io::Cursor::new(bytes)).expect("the fixture reads");
            let old = data.text_of(SLOT_FACTIONS).expect("it has a roster").to_string();
            let n = data.save.params.gw * data.save.params.gh;
            let mut warnings = Vec::new();
            let civ = civ_from_project(&data, n, &mut warnings).expect("its civ layer restores");
            assert!(!warnings.iter().any(|w| w.contains("government")), "{name}: {warnings:?}");
            assert!(civ.faction_roster.count() >= 2, "{name}: premise, real factions");
            for (i, e) in civ.faction_roster.0.iter().enumerate() {
                let pos = cartalith_civ::roster::CIV_GOVERNMENTS.iter().position(|&(k, _)| k == e.government);
                assert!(
                    pos.is_some_and(|p| p < cartalith_civ::roster::CIV_GOVERNMENTS_REFERENCE_LEN),
                    "{name}: faction {i} holds {:?}, not a reference form",
                    e.government
                );
            }
            let mut docs = BTreeMap::new();
            civ_documents(&civ, None, &mut docs);
            assert_eq!(docs[SLOT_FACTIONS], old, "{name}: factions.json re-serialises byte for byte");
        }
    }

    /// Protects G1-4's bar 2: every government key, the reference nine and the
    /// eight G1 appended in turn, survives a save and reopen on a faction
    /// (17 of 17), and a key this build does not know is kept verbatim on
    /// write-back (`SAVEFILE_COMPAT.md` §9.2: preserved, never defaulted).
    #[test]
    fn every_government_key_round_trips_and_an_unknown_one_is_kept() {
        let mut kept = 0;
        for &(key, _) in cartalith_civ::roster::CIV_GOVERNMENTS.iter() {
            let mut civ = sample_civ();
            assert!(civ.faction_roster.set_field(1, "government", key), "{key} is settable");
            let back = round_trip(&civ, 4, 3);
            assert_eq!(back.faction_roster.0[1].government, key);
            kept += 1;
        }
        assert_eq!(kept, 17);
        let mut civ = sample_civ();
        assert!(!civ.faction_roster.set_field(1, "government", "future_form"), "the setter still refuses unknown keys");
        civ.faction_roster.0[1].government = "future_form".to_string();
        let back = round_trip(&civ, 4, 3);
        assert_eq!(back.faction_roster.0[1].government, "future_form", "unknown keys are preserved");
    }

    /// Protects G1-4's bar 3: a timeline year recorded while a faction was a
    /// `merchant_republic` keeps that form through a save and reopen, and the
    /// military reading of that year resolves to it (`resolve_institutions`)
    /// even after today's roster moves the faction back to `monarchy`.
    #[test]
    fn a_recorded_year_keeps_a_g1_government_through_an_archive() {
        let mut civ = sample_civ();
        civ.timeline.clear();
        civ.faction_roster.0[1].government = "merchant_republic".to_string();
        civ.civ_add_year(100);
        civ.faction_roster.0[1].government = "monarchy".to_string();
        let back = round_trip(&civ, 4, 3);
        let rec = back.timeline.iter().find(|s| s.year == 100).expect("year 100").record.clone();
        let inst = rec.institutions.expect("institutions recorded");
        assert_eq!(inst[1].government, "merchant_republic");
        let live = (back.faction_roster.0[1].ag_tech.as_str(), back.faction_roster.0[1].government.as_str());
        assert_eq!(live.1, "monarchy", "premise: today's roster differs");
        let (_, gov) = crate::civ_military_bridge::resolve_institutions(live, Some(&inst), 1);
        assert_eq!(gov, "merchant_republic");
    }

    /// Records `year` the way an older writer (or the collapse run) did: a plain snapshot of
    /// the live lists with the default, unrecorded [`cartalith_civ::timeline::YearRecord`].
    fn civ_snapshot_save_for_test(civ: &mut CivData, year: i64) {
        cartalith_civ::timeline::civ_snapshot_save(
            &mut civ.timeline,
            year,
            civ.territory.clone(),
            civ.settlements.clone(),
            civ.ways.clone(),
        );
    }

    /// Protects: SAVEFILE_COMPAT.md 10.1 -- an out-of-order timeline comes back sorted by year
    /// with each year keeping its own raster. The fixture has no duplicate years, so the "unique"
    /// half of the name is not exercised here.
    #[test]
    fn the_timeline_comes_back_sorted_and_unique() {
        // §10.1. The year cursor walks this list by index, so a file whose
        // years arrived out of order would scrub backwards.
        let mut civ = sample_civ();
        // Built out of order on purpose, and with `Key` frames rather than through
        // `civ_snapshot_save`, because a writer that hands the reader an unsorted timeline is
        // exactly what §10.1 is about. Every frame here stands on its own, so no delta chain is
        // being relied on to survive the shuffle -- the reconstruction assertions below are
        // reading three independent rasters.
        civ.timeline = vec![
            cartalith_civ::timeline::TimelineSnapshot {
                year: 300,
                territory: cartalith_civ::timeline::TerritoryFrame::Key(vec![1; 12]),
                settlements: vec![],
                ways: vec![],
                collapse_flags: Default::default(),
                record: Default::default(),
            },
            cartalith_civ::timeline::TimelineSnapshot {
                year: 100,
                territory: cartalith_civ::timeline::TerritoryFrame::Key(vec![2; 12]),
                settlements: vec![],
                ways: vec![],
                collapse_flags: Default::default(),
                record: Default::default(),
            },
            cartalith_civ::timeline::TimelineSnapshot {
                year: 200,
                territory: cartalith_civ::timeline::TerritoryFrame::Key(vec![3; 12]),
                settlements: vec![],
                ways: vec![],
                collapse_flags: Default::default(),
                record: Default::default(),
            },
        ];
        let back = round_trip(&civ, 4, 3);
        assert_eq!(
            back.timeline.iter().map(|s| s.year).collect::<Vec<_>>(),
            vec![100, 200, 300]
        );
        // ...and each year kept its own raster, not its neighbour's. Read through the
        // reconstruction, since owner ruling 27 made the stored frame a delta chain -- the
        // reader re-encodes on the way in, so `back`'s 200 and 300 are deltas against 100.
        assert_eq!(
            cartalith_civ::timeline::civ_territory_at(&back.timeline, 100).unwrap(),
            vec![2; 12]
        );
        assert_eq!(
            cartalith_civ::timeline::civ_territory_at(&back.timeline, 300).unwrap(),
            vec![1; 12]
        );
    }

    /// Protects: `GUI_GAP_REGISTER.md` CV-23 -- a recorded year's territory raster survives an
    /// archive as it was then, differs from the live grid, and keeps its compact delta encoding
    /// (owner ruling 27).
    #[test]
    fn a_recorded_year_comes_back_holding_what_was_true_then_not_what_is_live_now() {
        // The claim `GUI_GAP_REGISTER.md`'s CV-23 row is about, end to end: which faction
        // held which cells *in a given past year*, surviving an archive.
        //
        // `the_timeline_comes_back_sorted_and_unique` above is the ordering test -- it
        // builds `TerritoryFrame::Key` entries by hand, so nothing in it goes through the
        // production recording path and no delta chain crosses the archive boundary. This
        // one records through `civ_add_year`, one of the three production routes into
        // `civ_snapshot_save` (the others are `timeline_bridge`'s simulation recorder and
        // `civ_from_project`'s own re-encode below), and the two years it records hold
        // *genuinely different* rasters: a fixture where the territory never changed
        // cannot tell a historical read from a live one, so it would pass whether or not
        // any history was kept at all.
        let mut civ = sample_civ();
        let a = vec![1i32; 12];
        let mut b = a.clone();
        // Faction 9 takes two cells somewhere between year 100 and year 300.
        b[5] = 9;
        b[6] = 9;
        assert_ne!(a, b, "the fixture's whole point is that these differ");

        // Recorded from an empty timeline rather than on top of `sample_civ`'s own year
        // 120 entry, so the years below are a plain recording sequence. Worth stating why
        // rather than just doing it: `civ_add_year` carries forward from the nearest
        // *earlier* recorded year and has no earlier year to carry when there is none, so
        // a year added below every recorded one records an empty raster (its documented
        // `None` branch -- the reference's behaviour, not a defect). A fixture that left
        // year 120 in place and recorded years beneath it would therefore assert zeroes.
        civ.timeline.clear();
        civ.territory = a.clone();
        civ.civ_add_year(100);
        civ.civ_add_year(300); // carries year 100's raster forward, year cursor lands on 300
        civ.territory = b.clone(); // the live edit made while year 300 is the active year
        civ.civ_add_year(500); // snapshots the active year 300 with `b` before jumping

        // The live grid then moves on again, to a state *no* recorded year holds. Without
        // this, "the read answered the past" and "the read answered the present" have the
        // same value at year 300 and the assertions below would not separate them.
        let live = vec![4i32; 12];
        civ.territory = live.clone();

        let back = round_trip(&civ, 4, 3);

        assert_eq!(
            cartalith_civ::timeline::civ_territory_at(&back.timeline, 100).unwrap(),
            a,
            "year 100 must come back holding the raster that was live when it was recorded"
        );
        assert_eq!(
            cartalith_civ::timeline::civ_territory_at(&back.timeline, 300).unwrap(),
            b,
            "year 300 must come back holding its own later raster, not year 100's"
        );
        assert_eq!(
            back.territory, live,
            "the live grid is `rasters/territory.i32`, and is none of the recorded years"
        );
        assert_ne!(
            cartalith_civ::timeline::civ_territory_at(&back.timeline, 100).unwrap(),
            back.territory,
            "a reader that answered the live state would satisfy every other shape here"
        );

        // ...and the compact encoding survived the trip rather than being flattened by it.
        // The archive carries one whole raster per recorded year (§10.2) and `civ_from_
        // project` re-encodes in year order on the way in, so year 300 comes back as a
        // difference naming year 100 and carrying exactly the two cells that changed --
        // which is also what makes the historical record affordable (owner ruling 27, and
        // `TERRITORY_KEYFRAME_INTERVAL`'s own doc comment for the measured bound).
        let y300 = back.timeline.iter().find(|s| s.year == 300).unwrap();
        assert_eq!(
            y300.territory,
            cartalith_civ::timeline::TerritoryFrame::Delta {
                base_year: 100,
                cells: vec![(5, 9), (6, 9)]
            }
        );
        // Year 500 changed nothing, and costs nothing: the ruling's own case.
        let y500 = back.timeline.iter().find(|s| s.year == 500).unwrap();
        assert_eq!(
            y500.territory,
            cartalith_civ::timeline::TerritoryFrame::Delta {
                base_year: 300,
                cells: Vec::new()
            }
        );
    }

    /// Protects: the member names and values of the settlements, ways and factions documents
    /// match what `SAVEFILE_COMPAT.md` publishes, since a second implementation is written from
    /// those names.
    #[test]
    fn the_documents_are_the_shapes_the_specification_publishes() {
        // The specification is what a second implementation is written
        // from, so the member *names* are the contract. A rename here that
        // did not reach `SAVEFILE_COMPAT.md` would break that
        // implementation and nothing in this workspace.
        let civ = sample_civ();
        let mut docs = BTreeMap::new();
        civ_documents(&civ, None, &mut docs);

        let s: serde_json::Value = serde_json::from_str(&docs[SLOT_SETTLEMENTS]).unwrap();
        assert_eq!(s["next_id"], 22);
        assert_eq!(s["settlements"][0]["id"], 7);
        assert_eq!(s["settlements"][0]["population"], 41230);
        assert_eq!(s["settlements"][0]["kind"], "city");
        assert_eq!(s["settlements"][0]["suitability"], 0.75);
        assert_eq!(s["settlements"][0]["extras"]["age"], 320);
        assert_eq!(s["settlements"][1]["village_seeded"], true);
        // A settlement with nothing authored on it carries no `extras` at
        // all rather than an object of defaults.
        assert!(s["settlements"][1].get("extras").is_none());

        let w: serde_json::Value = serde_json::from_str(&docs[SLOT_WAYS]).unwrap();
        assert_eq!(w["roads"][0]["length_km"], 12.5);
        assert_eq!(w["roads"][0]["class"], "highway");
        assert_eq!(w["roads"][0]["points"][0][0], 1.5);
        assert_eq!(w["sea_lanes"][0]["name"], "Northern Passage");

        let f: serde_json::Value = serde_json::from_str(&docs[SLOT_FACTIONS]).unwrap();
        assert_eq!(f["factions"][0]["id"], 0);
        assert_eq!(f["factions"][0]["name"], "Unclaimed");
        assert!(f["factions"][0]["user_color"].is_null());
        assert_eq!(f["factions"][1]["color"].as_array().unwrap().len(), 3);
    }

    /// Protects: SAVEFILE_COMPAT.md 14.1 -- no integer in any civ document exceeds 2^53 - 1,
    /// which a JavaScript reader could not hold.
    #[test]
    fn every_id_the_format_writes_is_a_safe_integer() {
        // §14.1 is a constraint on the *format*, not a warning: a second
        // implementation in JavaScript cannot represent an id past 2^53.
        /// 2^53 - 1, JavaScript's `Number.MAX_SAFE_INTEGER`: the largest integer a JS reader
        /// holds exactly (SAVEFILE_COMPAT.md 14.1).
        const MAX_SAFE: u64 = 9_007_199_254_740_991;
        let civ = sample_civ();
        let mut docs = BTreeMap::new();
        civ_documents(&civ, None, &mut docs);
        for text in docs.values() {
            let v: serde_json::Value = serde_json::from_str(text).unwrap();
            let mut worst: u64 = 0;
            /// Records the largest unsigned integer found anywhere in `v` into `worst`,
            /// descending arrays and objects.
            fn walk(v: &serde_json::Value, worst: &mut u64) {
                match v {
                    serde_json::Value::Number(n) => {
                        if let Some(i) = n.as_u64() {
                            *worst = (*worst).max(i);
                        }
                    }
                    serde_json::Value::Array(a) => a.iter().for_each(|x| walk(x, worst)),
                    serde_json::Value::Object(o) => o.values().for_each(|x| walk(x, worst)),
                    _ => {}
                }
            }
            walk(&v, &mut worst);
            assert!(
                worst <= MAX_SAFE,
                "an id past Number.MAX_SAFE_INTEGER reached the archive: {worst}"
            );
        }
    }
}

/// `annotations/icons.json`'s own tests. Separate from the module above
/// because they exercise the *document*, not `WorldGen` -- `WorldGen` is a
/// cdylib `GodotClass` and cannot be constructed in a unit test, which is why
/// [`icon_from_dto`] was lifted out of `project_open`'s body in the first
/// place.
#[cfg(test)]
mod icon_document_tests {
    use super::*;

    /// **The exact text this build's writer produces**, and the fixture owner
    /// ruling 14's `origin` migration is owed.
    ///
    /// `MISTAKES.md`: *"A backward-compatibility test whose fixture is an
    /// empty collection proves nothing about the installed base. Build the
    /// fixture from a real prior-format document at the commit that wrote
    /// it."* This is that document, frozen **before** the format changes
    /// rather than reconstructed after: every project saved by this build and
    /// every one saved before it carries this shape, with no `origin` member
    /// anywhere in it. `the_frozen_fixture_is_the_writers_own_text` below is
    /// what stops it drifting into a hand-built approximation --
    /// `project_save_with_documents` writes icons through `insert_doc`, which
    /// is `serde_json::to_string_pretty`, and that call is made there too.
    ///
    /// Four rows, chosen to reach the branches rather than to look plausible:
    /// a plain click placement (`scale` 1.0, `set` null), a brush stamp (a
    /// scale that is not 1.0), a `custom` row (the only family that carries a
    /// `set`), and a row whose `slot` this build cannot resolve -- which
    /// §11.2 requires be **kept**.
    const ICONS_BEFORE_ORIGIN: &str = r#"{
  "icons": [
    {
      "x": 40.0,
      "y": 12.0,
      "family": "feature",
      "slot": "mountain",
      "set": null,
      "scale": 1.0
    },
    {
      "x": 7.5,
      "y": 91.25,
      "family": "settlement",
      "slot": "city",
      "set": null,
      "scale": 0.625
    },
    {
      "x": 3.0,
      "y": 4.0,
      "family": "custom",
      "slot": "obelisk",
      "set": "my pack",
      "scale": 2.0
    },
    {
      "x": 100.0,
      "y": 200.0,
      "family": "poi",
      "slot": "a_slot_no_pack_here_defines",
      "set": null,
      "scale": 1.0
    }
  ]
}"#;

    /// Protects: `ICONS_BEFORE_ORIGIN` is exactly what the current writer emits for those four
    /// rows, so the migration tests below run against a real writer document and not a hand-built
    /// lookalike.
    /// The fixture is the writer's output and not a hand-typed lookalike.
    ///
    /// Without this the migration test would be checking a reader against a
    /// document no writer ever produced, which is the failure mode
    /// `MISTAKES.md`'s backward-compatibility row exists to name.
    #[test]
    fn the_frozen_fixture_is_the_writers_own_text() {
        let doc = IconsDoc {
            icons: vec![
                IconDto {
                    x: 40.0,
                    y: 12.0,
                    family: "feature".to_string(),
                    slot: "mountain".to_string(),
                    set: None,
                    scale: 1.0,
                    origin: String::new(),
                },
                IconDto {
                    x: 7.5,
                    y: 91.25,
                    family: "settlement".to_string(),
                    slot: "city".to_string(),
                    set: None,
                    scale: 0.625,
                    origin: String::new(),
                },
                IconDto {
                    x: 3.0,
                    y: 4.0,
                    family: "custom".to_string(),
                    slot: "obelisk".to_string(),
                    set: Some("my pack".to_string()),
                    scale: 2.0,
                    origin: String::new(),
                },
                IconDto {
                    x: 100.0,
                    y: 200.0,
                    family: "poi".to_string(),
                    slot: "a_slot_no_pack_here_defines".to_string(),
                    set: None,
                    scale: 1.0,
                    origin: String::new(),
                },
            ],
        };
        // `insert_doc`'s own call, which is what puts this document in the
        // archive.
        assert_eq!(serde_json::to_string_pretty(&doc).expect("serialises"), ICONS_BEFORE_ORIGIN);
    }

    /// Protects: the pre-`origin` icons document opens, resolves and re-serialises
    /// byte-identically through `icon_from_dto` and `icon_to_dto`.
    /// The whole chain MISTAKES.md's backward-compatibility rule asks for:
    /// the frozen document **opens, resolves, and re-serialises
    /// byte-identically** -- through [`icon_from_dto`] and [`icon_to_dto`],
    /// the two seams a real save/open uses, not through serde alone.
    ///
    /// This is what makes "`Manual` is spelled as an absent member" a fact
    /// about the *writer* rather than about a hand-built DTO: the icons
    /// below are `IconOrigin::Manual` after resolution, and the bytes that
    /// come back out carry no `origin` member at all.
    #[test]
    fn the_frozen_document_survives_a_full_resolve_and_write_back_unchanged() {
        let doc: IconsDoc = serde_json::from_str(ICONS_BEFORE_ORIGIN).expect("parses");
        let icons: Vec<_> = doc.icons.iter().filter_map(icon_from_dto).collect();
        assert_eq!(icons.len(), 4, "a row was dropped on the way in");
        let back = IconsDoc { icons: icons.iter().map(icon_to_dto).collect() };
        assert_eq!(
            serde_json::to_string_pretty(&back).expect("serialises"),
            ICONS_BEFORE_ORIGIN,
            "a load-and-save cycle rewrote a project written before `origin` existed"
        );
        assert!(
            !ICONS_BEFORE_ORIGIN.contains("origin"),
            "the fixture is no longer a pre-`origin` document"
        );
    }

    /// Protects: owner ruling 14's migration rule -- a document with no `origin` member loads
    /// every icon as `Manual`, while explicit `manual` and `generated` rows still resolve to
    /// their own variants.
    /// **Owner ruling 14's migration rule, and the assertion that could not
    /// exist before the field did**: a document written before `origin`
    /// loads with every icon `Manual`.
    ///
    /// Not "defaults to `Manual` because that is convenient" -- it is the
    /// true answer for old data. The generated placement pass
    /// (`icon_bridge::IconEditor::generate`) is newer than this document
    /// format, so nothing that could have written a generated row existed
    /// when the fixture's writer ran.
    #[test]
    fn a_document_written_before_origin_loads_with_every_icon_manual() {
        use cartalith_assets::manual::IconOrigin;
        let doc: IconsDoc = serde_json::from_str(ICONS_BEFORE_ORIGIN).expect("parses");
        let icons: Vec<_> = doc.icons.iter().filter_map(icon_from_dto).collect();
        assert_eq!(icons.len(), 4, "a row was dropped: {icons:?}");
        for (i, ic) in icons.iter().enumerate() {
            assert_eq!(
                ic.origin,
                IconOrigin::Manual,
                "row {i} of a pre-`origin` document did not read as hand-placed"
            );
        }
        // The negative control: the same reader does *not* answer `Manual`
        // for everything. A row that says so explicitly still resolves, and
        // a `generated` row resolves to the other variant -- so the loop
        // above is reading the member, not returning a constant.
        let explicit: IconsDoc = serde_json::from_str(
            r#"{"icons":[
                {"x":1.0,"y":2.0,"family":"feature","slot":"mountain","scale":1.0,"origin":"manual"},
                {"x":3.0,"y":4.0,"family":"poi","slot":"ruin","scale":1.0,"origin":"generated"}
            ]}"#,
        )
        .expect("parses");
        let got: Vec<IconOrigin> =
            explicit.icons.iter().filter_map(icon_from_dto).map(|i| i.origin).collect();
        assert_eq!(got, vec![IconOrigin::Manual, IconOrigin::Generated]);
    }

    /// Protects: A generated icon survives the round trip **as generated**, and is the
    /// only kind that puts an `origin` member on the wire.
    #[test]
    fn a_generated_icon_writes_its_origin_and_reads_back_as_generated() {
        use cartalith_assets::manual::{IconOrigin, ManualIcon, ManualIconFamily};
        let ic = ManualIcon {
            x: 12.0,
            y: 34.0,
            family: ManualIconFamily::Poi,
            slot: "ruin".to_string(),
            set: None,
            scale: 1.0,
            origin: IconOrigin::Generated,
        };
        let text = serde_json::to_string_pretty(&IconsDoc { icons: vec![icon_to_dto(&ic)] })
            .expect("serialises");
        assert!(text.contains(r#""origin": "generated""#), "{text}");

        let doc: IconsDoc = serde_json::from_str(&text).expect("parses");
        let back: Vec<_> = doc.icons.iter().filter_map(icon_from_dto).collect();
        assert_eq!(back, vec![ic], "a generated icon did not survive the round trip");

        // And the hand path's own row carries no `origin` member at all --
        // the same icon, one field different, is the pre-`origin` shape.
        let manual = ManualIcon { origin: IconOrigin::Manual, ..back[0].clone() };
        let text = serde_json::to_string_pretty(&IconsDoc { icons: vec![icon_to_dto(&manual)] })
            .expect("serialises");
        assert!(!text.contains("origin"), "a hand-placed row wrote an `origin` member: {text}");
    }

    /// Protects: SAVEFILE_COMPAT.md 6.4a rung 2 -- an `origin` that is not exactly `manual` or
    /// `generated` (`imported`, `Manual`) costs its row rather than being folded into `Manual`.
    /// §6.4a rung 2 again, for the new member: an `origin` that is neither
    /// known string costs its row and nothing more -- `family`'s answer, not
    /// a silent fold into `Manual`.
    ///
    /// The difference matters because the two are not the same claim. Absent
    /// means "written before the field existed", which this build knows the
    /// answer to. `"imported"` means "written by something this build does
    /// not understand", which it does not -- and relabelling it hand-placed
    /// would be inventing the world.
    #[test]
    fn an_unrecognised_origin_costs_the_row_and_is_not_folded_into_manual() {
        let text = r#"{"icons":[
            {"x":1.0,"y":2.0,"family":"feature","slot":"mountain","scale":1.0},
            {"x":3.0,"y":4.0,"family":"feature","slot":"mountain","scale":1.0,"origin":"manual"},
            {"x":5.0,"y":6.0,"family":"feature","slot":"mountain","scale":1.0,"origin":"generated"},
            {"x":7.0,"y":8.0,"family":"feature","slot":"mountain","scale":1.0,"origin":"imported"},
            {"x":9.0,"y":10.0,"family":"feature","slot":"mountain","scale":1.0,"origin":"Manual"}
        ]}"#;
        let doc: IconsDoc = serde_json::from_str(text).expect("parses");
        assert_eq!(doc.icons.len(), 5, "the fixture itself is wrong");
        let icons: Vec<_> = doc.icons.iter().filter_map(icon_from_dto).collect();
        // Three kept: absent, `manual`, `generated`. Two dropped: `imported`
        // and the wrong-case `Manual` -- the keys are exact, as every other
        // `from_key` in this port is.
        assert_eq!(icons.len(), 3, "{icons:?}");
        assert!(
            !icons.iter().any(|i| i.x == 7.0 || i.x == 9.0),
            "an unrecognised origin was guessed into a known one: {icons:?}"
        );
        // The same shape `project_open` builds its warning line from.
        assert_eq!(doc.icons.len() - icons.len(), 2, "the warning would understate the loss");
    }

    /// Protects: Opening a document written before `origin` existed must not silently
    /// rewrite it (`SAVEFILE_COMPAT.md` §6.2, §14.3).
    #[test]
    fn a_document_written_before_origin_reopens_and_re_serialises_byte_identically() {
        let doc: IconsDoc =
            serde_json::from_str(ICONS_BEFORE_ORIGIN).expect("an older document parses");
        assert_eq!(doc.icons.len(), 4, "a row was lost on the way in");
        let again = serde_json::to_string_pretty(&doc).expect("serialises");
        assert_eq!(again, ICONS_BEFORE_ORIGIN, "opening an old project rewrote it");
    }

    /// Protects: every field of every fixture row (family, position, scale, set, and an
    /// unresolvable slot kept as written) survives resolution, including SAVEFILE_COMPAT.md
    /// 11.2's keep-the-row rule.
    /// Every field of every row survives resolution -- the property the
    /// `origin` merge has to keep, asserted before the merge rather than
    /// after.
    #[test]
    fn every_row_of_the_frozen_fixture_resolves_with_its_own_values() {
        use cartalith_assets::manual::ManualIconFamily;
        let doc: IconsDoc = serde_json::from_str(ICONS_BEFORE_ORIGIN).expect("parses");
        let icons: Vec<_> = doc.icons.iter().filter_map(icon_from_dto).collect();
        assert_eq!(icons.len(), 4, "a row was dropped: {icons:?}");

        assert_eq!(icons[0].family, ManualIconFamily::Feature);
        assert_eq!((icons[0].x, icons[0].y), (40.0, 12.0));
        assert_eq!(icons[0].scale, 1.0);
        assert_eq!(icons[0].set, None);

        assert_eq!(icons[1].family, ManualIconFamily::Settlement);
        assert_eq!((icons[1].x, icons[1].y), (7.5, 91.25));
        // A brush stamp's own multiplier, carried rather than normalised.
        assert_eq!(icons[1].scale, 0.625);

        assert_eq!(icons[2].family, ManualIconFamily::Custom);
        assert_eq!(icons[2].set.as_deref(), Some("my pack"));
        assert_eq!(icons[2].scale, 2.0);

        // §11.2: *"A reader MUST keep an icon whose `slot` it cannot resolve,
        // and render a placeholder -- dropping it silently loses the author's
        // work over an art-pack mismatch."* The slot is carried as written.
        assert_eq!(icons[3].family, ManualIconFamily::Poi);
        assert_eq!(icons[3].slot, "a_slot_no_pack_here_defines");
    }

    /// Protects: SAVEFILE_COMPAT.md 6.4a -- an unknown or empty `family` costs the row, an
    /// unresolvable `slot` does not, and a non-positive `scale` is replaced by 1.0.
    /// §6.4a rung 2: a damaged row costs its element and nothing more.
    ///
    /// The three cases are the three answers `icon_from_dto` can give, and
    /// they are deliberately not the same answer: an unresolvable **family**
    /// costs the row, an unresolvable **slot** does not, and a non-positive
    /// **scale** is substituted rather than costing anything.
    #[test]
    fn an_icon_row_whose_family_does_not_resolve_costs_the_row() {
        let text = r#"{"icons":[
            {"x":1.0,"y":2.0,"family":"feature","slot":"mountain","scale":1.0},
            {"x":3.0,"y":4.0,"family":"not_a_family","slot":"mountain","scale":1.0},
            {"x":5.0,"y":6.0,"family":"","slot":"mountain","scale":1.0},
            {"x":7.0,"y":8.0,"family":"poi","slot":"nothing_defines_this","scale":1.0},
            {"x":9.0,"y":10.0,"family":"feature","slot":"mountain","scale":0.0},
            {"x":11.0,"y":12.0,"family":"feature","slot":"mountain","scale":-4.0}
        ]}"#;
        let doc: IconsDoc = serde_json::from_str(text).expect("parses");
        assert_eq!(doc.icons.len(), 6, "the fixture itself is wrong");
        let icons: Vec<_> = doc.icons.iter().filter_map(icon_from_dto).collect();

        // Two rows dropped, and only those two: `not_a_family` and the empty
        // string. The empty string is the shape a *missing* `family` member
        // takes through `#[serde(default)]`, and it must not resolve to a
        // family either.
        assert_eq!(icons.len(), 4, "{icons:?}");
        assert!(
            !icons.iter().any(|i| i.x == 3.0 || i.x == 5.0),
            "an unresolvable family was guessed into a family: {icons:?}"
        );

        // The unresolvable slot survived, with its string intact.
        assert_eq!(icons[1].slot, "nothing_defines_this");

        // §6.4a rung 1: a stated substitution, applied. `1.0` is what a plain
        // click placement writes, so a row with no usable multiplier draws at
        // the size an unscaled one does.
        assert_eq!(icons[2].scale, 1.0, "a zero scale was carried through");
        assert_eq!(icons[3].scale, 1.0, "a negative scale was carried through");
    }

    /// Protects: The count `project_open` reports is the document's rows minus the
    /// survivors, which is the number the `warnings` line is built from.
    #[test]
    fn the_reported_drop_count_is_the_rows_that_did_not_survive() {
        let text = r#"{"icons":[
            {"family":"feature","slot":"mountain"},
            {"family":"nope","slot":"mountain"},
            {"family":"also_nope","slot":"mountain"}
        ]}"#;
        let doc: IconsDoc = serde_json::from_str(text).expect("parses");
        let kept = doc.icons.iter().filter_map(icon_from_dto).count();
        assert_eq!(doc.icons.len() - kept, 2, "the warning would understate the loss");
    }
}

/// CA-19: `appearance.json`'s `biome_cols` member (`SAVEFILE_COMPAT.md`
/// §13.2).
#[cfg(test)]
mod biome_cols_tests {
    use super::*;

    /// **A real project's `appearance.json`, saved by the writer before this
    /// member existed** -- not a hand-typed document. Provenance: written
    /// 2026-09-24 by `_biomesave_probe.gd -- --capture`, running
    /// `WorldGen::project_save` in the `cartalith_godot.dll` built at 09:56
    /// from a tree whose `project_bridge.rs` was byte-identical to `bab9edd`
    /// (mtime 09:43, `git diff --stat HEAD` empty). That session had set the
    /// look to vibrant, two tunable overrides **and biome class 3 to
    /// (12, 200, 77)** -- and the file carries no trace of the last, which
    /// is the defect this member closes. Copied out of the archive verbatim,
    /// no trailing newline, exactly as `insert_doc` wrote it.
    const APPEARANCE_BEFORE_BIOME_COLS: &str = r#"{
  "quality": "quality",
  "look": "Natural Vibrant",
  "territory_opacity": 0.3215686274509804,
  "overrides": {
    "bio_blend": 0.625,
    "sun_az_deg": 300.0
  },
  "ramp": null,
  "npr": {
    "contours": 0.0,
    "contour_m": 0.0,
    "peak_m": 0.0,
    "ink": 0.0,
    "hachure": 0.0,
    "watercolor": 0.0,
    "cel": 0.0,
    "crosshatch": 0.0,
    "stipple": 0.0,
    "sepia": 0.0,
    "risograph": 0.0,
    "pointillism": 0.0,
    "waves": false,
    "wave_dist": 0.0,
    "multi_sun": true,
    "animate_water": false,
    "village": false
  }
}"#;

    /// Protects: a real `appearance.json` written before `biome_cols` opens at the reference
    /// table (no overrides) and re-serialises byte-identically, without costing its sibling
    /// members.
    /// MISTAKES.md's backward-compatibility rule, whole: the old document
    /// **opens, resolves, and re-serialises byte-identically**, through the
    /// two seams a real save/open uses -- so opening an old project and
    /// saving it does not grow a `biome_cols` member it never had.
    #[test]
    fn a_project_saved_before_biome_cols_opens_at_the_default_table_and_writes_back_unchanged() {
        let doc: AppearanceDoc =
            serde_json::from_str(APPEARANCE_BEFORE_BIOME_COLS).expect("the HEAD-written document parses");
        assert!(doc.biome_cols.is_none());
        let ov = biome_overrides_from_member(doc.biome_cols.as_ref()).expect("absent is valid");
        assert_eq!(ov, [None; 15], "absent must be the reference table, not a table of anything");
        // The rest of the look came through the parse, so the member did not
        // cost its siblings.
        assert_eq!(doc.look, "Natural Vibrant");
        assert_eq!(doc.overrides["bio_blend"], 0.625);
        assert_eq!(doc.overrides["sun_az_deg"], 300.0);
        let back = AppearanceDoc { biome_cols: biome_cols_member(&ov), ..doc };
        assert_eq!(
            serde_json::to_string_pretty(&back).expect("serialises"),
            APPEARANCE_BEFORE_BIOME_COLS,
            "a load-and-save cycle rewrote a project written before biome_cols existed"
        );
    }

    /// Protects: An edited class survives the writer and the reader, and the table on
    /// disk is the literal colours, in class order.
    #[test]
    fn an_edited_biome_colour_survives_the_round_trip() {
        let mut ov = [None; 15];
        ov[2] = Some((12, 200, 77));
        let doc = AppearanceDoc { biome_cols: biome_cols_member(&ov), ..AppearanceDoc::default() };
        let text = serde_json::to_string_pretty(&doc).unwrap();
        let v: serde_json::Value = serde_json::from_str(&text).unwrap();
        let table = v["biome_cols"].as_array().expect("an edited table is written");
        assert_eq!(table.len(), 15);
        assert_eq!(table[2], serde_json::json!([12, 200, 77]));
        // Unedited classes are written as the reference's own colours.
        assert_eq!(table[0], serde_json::json!([90, 147, 184]));
        assert_eq!(table[14], serde_json::json!([30, 70, 110]));

        let back: AppearanceDoc = serde_json::from_str(&text).unwrap();
        let got = biome_overrides_from_member(back.biome_cols.as_ref()).unwrap();
        assert_eq!(got[2], Some((12, 200, 77)));
        assert_eq!(got.iter().filter(|c| c.is_some()).count(), 1, "only the edited class is an override");
    }

    /// Protects: Equal to the default is written as nothing, even when every class
    /// carries an explicit override that happens to equal its colour.
    #[test]
    fn a_table_equal_to_the_reference_writes_no_member() {
        assert!(biome_cols_member(&[None; 15]).is_none());
        let explicit: [Option<(u8, u8, u8)>; 15] = std::array::from_fn(|i| Some(crate::render::CART_BIOME_COLS[i]));
        assert!(biome_cols_member(&explicit).is_none());
    }

    /// Protects: Every malformed shape is refused whole -- never a partial table.
    #[test]
    fn a_malformed_table_is_refused_whole() {
        let good: Vec<serde_json::Value> = (0..15).map(|_| serde_json::json!([1, 2, 3])).collect();
        let short = serde_json::Value::Array(good[..14].to_vec());
        let long = serde_json::Value::Array([good.clone(), vec![serde_json::json!([1, 2, 3])]].concat());
        let mut two = good.clone();
        two[4] = serde_json::json!([1, 2]);
        let mut big = good.clone();
        big[4] = serde_json::json!([1, 256, 3]);
        let mut neg = good.clone();
        neg[4] = serde_json::json!([1, -1, 3]);
        let mut frac = good.clone();
        frac[4] = serde_json::json!([1, 2.5, 3]);
        let mut word = good.clone();
        word[4] = serde_json::json!("red");
        for (name, v) in [
            ("14 entries", short),
            ("16 entries", long),
            ("a pair", serde_json::Value::Array(two)),
            ("256", serde_json::Value::Array(big)),
            ("-1", serde_json::Value::Array(neg)),
            ("2.5", serde_json::Value::Array(frac)),
            ("a string entry", serde_json::Value::Array(word)),
            ("an object", serde_json::json!({"3": [1, 2, 3]})),
            ("a number", serde_json::json!(7)),
        ] {
            assert!(biome_overrides_from_member(Some(&v)).is_err(), "{name} was accepted");
        }
        // The positive control: the same fifteen, well-formed, are accepted.
        let ok = biome_overrides_from_member(Some(&serde_json::Value::Array(good))).expect("15 triples parse");
        assert_eq!(ok[4], Some((1, 2, 3)));
        // And a bad member does not fail its document: the siblings parse.
        let doc: AppearanceDoc =
            serde_json::from_str(r#"{"look":"vibrant","biome_cols":[[1,2]]}"#).expect("the document still parses");
        assert_eq!(doc.look, "vibrant");
        assert!(biome_overrides_from_member(doc.biome_cols.as_ref()).is_err());
    }
}

/// ET-C2: `appearance.json`'s optional `terrain_adjust` / `water_adjust`
/// members (`SAVEFILE_COMPAT.md` §13.2's appearance document).
#[cfg(test)]
mod adjust_member_tests {
    use super::*;
    use crate::render::adjust::{AdjustKind, AdjustStack, Adjustment};

    /// The real HEAD-written `appearance.json` `biome_cols_tests` pins (its
    /// provenance is documented there: `_biomesave_probe.gd -- --capture`,
    /// 2026-09-24), repeated verbatim because that module's copy is private.
    /// Written long before ET-C2, so it carries neither stack member.
    const APPEARANCE_BEFORE_ADJUST: &str = r#"{
  "quality": "quality",
  "look": "Natural Vibrant",
  "territory_opacity": 0.3215686274509804,
  "overrides": {
    "bio_blend": 0.625,
    "sun_az_deg": 300.0
  },
  "ramp": null,
  "npr": {
    "contours": 0.0,
    "contour_m": 0.0,
    "peak_m": 0.0,
    "ink": 0.0,
    "hachure": 0.0,
    "watercolor": 0.0,
    "cel": 0.0,
    "crosshatch": 0.0,
    "stipple": 0.0,
    "sepia": 0.0,
    "risograph": 0.0,
    "pointillism": 0.0,
    "waves": false,
    "wave_dist": 0.0,
    "multi_sun": true,
    "animate_water": false,
    "village": false
  }
}"#;

    /// A valid two-entry stack.
    fn stack() -> AdjustStack {
        AdjustStack {
            entries: vec![
                Adjustment::new(AdjustKind::Hsl { hue_deg: 30.0, saturation: -0.25, lightness: 0.0 }),
                Adjustment { kind: AdjustKind::Levels { in_black: 5.0, in_white: 250.0, gamma: 1.2, out_black: 0.0, out_white: 255.0 }, enabled: false, opacity: 0.5 },
            ],
        }
    }

    /// Protects: a project written before the members existed (the real
    /// HEAD-written `appearance.json` `biome_cols_tests` already pins) opens
    /// with no stack on either layer and re-serialises byte-identically, so
    /// opening and saving an old project does not grow a member it never had.
    #[test]
    fn a_project_saved_before_the_stacks_opens_with_none_and_writes_back_unchanged() {
        let doc: AppearanceDoc = serde_json::from_str(APPEARANCE_BEFORE_ADJUST).expect("the HEAD-written document parses");
        assert!(doc.terrain_adjust.is_none() && doc.water_adjust.is_none());
        assert_eq!(adjust_from_member(doc.terrain_adjust.as_ref()), Ok(None));
        let back = AppearanceDoc { terrain_adjust: adjust_member(None), water_adjust: adjust_member(Some(&AdjustStack::default())), ..doc };
        assert_eq!(serde_json::to_string_pretty(&back).unwrap(), APPEARANCE_BEFORE_ADJUST, "an empty or absent stack must write no member");
    }

    /// Protects: a stack survives the writer and the reader exactly (a
    /// disabled, half-opacity entry included: user data, saved), on its own
    /// layer only.
    #[test]
    fn a_stack_survives_the_round_trip_on_its_own_layer() {
        let doc = AppearanceDoc { water_adjust: adjust_member(Some(&stack())), ..AppearanceDoc::default() };
        let text = serde_json::to_string_pretty(&doc).unwrap();
        let v: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert!(v.get("terrain_adjust").is_none(), "the untouched layer writes nothing: {text}");
        assert_eq!(v["water_adjust"]["entries"].as_array().map(Vec::len), Some(2));
        let back: AppearanceDoc = serde_json::from_str(&text).unwrap();
        assert_eq!(adjust_from_member(back.water_adjust.as_ref()), Ok(Some(stack())));
        assert_eq!(adjust_from_member(back.terrain_adjust.as_ref()), Ok(None));
    }

    /// Protects: a member that does not parse, names an unknown kind, or
    /// carries an out-of-range parameter is refused whole with a reason --
    /// never loaded clamped or without its bad entry -- and does not fail
    /// the document it sits in.
    #[test]
    fn a_bad_stack_is_refused_whole_and_costs_only_itself() {
        for (name, v) in [
            ("out of range", serde_json::json!({"entries": [{"kind": {"hsl": {"hue_deg": 400.0, "saturation": 0.0, "lightness": 0.0}}}]})),
            ("unknown kind", serde_json::json!({"entries": [{"kind": {"sepia": {}}}]})),
            ("not a stack", serde_json::json!([1, 2, 3])),
            ("bad opacity", serde_json::json!({"entries": [{"kind": {"hsl": {"hue_deg": 10.0, "saturation": 0.0, "lightness": 0.0}}, "opacity": 2.0}]})),
        ] {
            assert!(adjust_from_member(Some(&v)).is_err(), "{name} was accepted");
        }
        let doc: AppearanceDoc = serde_json::from_str(r#"{"look":"vibrant","water_adjust":{"entries":[{"kind":{"hsl":{"hue_deg":400.0,"saturation":0.0,"lightness":0.0}}}]}}"#).expect("the document still parses");
        assert_eq!(doc.look, "vibrant");
        assert!(adjust_from_member(doc.water_adjust.as_ref()).unwrap_err().contains("hue_deg"));
    }
}

/// Owner Ruling AR's round trip (`SAVEFILE_COMPAT.md` §8.3, `substrate.rs`):
/// generate a real world with a real civ layer, save it through the same
/// writers `project_save_with_documents` calls, reopen it through the same
/// readers `project_open` calls, and compare what the formerly refusing
/// readouts compute -- not "it opened".
#[cfg(test)]
mod substrate_tests {
    use super::*;
    use crate::substrate::{self, SubstrateManifest};
    use std::io::Cursor;

    /// Substrate fixture width: a labelled judgement, big enough to carve rivers and route road
    /// edges, whose premises the tests assert.
    const GW: usize = 160;
    /// Substrate fixture height, paired with `GW` (a labelled judgement).
    const GH: usize = 112;

    /// One real world, the shipped parameters, CPU only, rivers carved.
    fn world() -> &'static (cartalith_engine::WorldParams, cartalith_engine::WorldState, CivData) {
        static W: std::sync::OnceLock<(cartalith_engine::WorldParams, cartalith_engine::WorldState, CivData)> =
            std::sync::OnceLock::new();
        W.get_or_init(|| {
            let mut p = crate::params::defaults();
            p.gw = GW;
            p.gh = GH;
            p.tect.seed = 24601;
            p.use_gpu = false;
            p.civ.villages = true;
            // The orogen relief term (Ruling CU item 1) is pinned OFF for this
            // fixture: these tests are about saving and reopening a project, not
            // about terrain, and `a_reopened_project_plans_over_the_saved_road_edges`
            // needs a straight road-edge-only stretch of at least 4 cells, which
            // this seed's smooth-belt world has and its ranged world did not
            // (measured 2026-10-19 with the first build's ridged term: "no
            // stretch of 4 road-edge-only cells"; not re-measured under the
            // follow-up's gully term, since the pin stands either way).
            p.tect.orogen_relief = false;
            let ws = cartalith_engine::generate_terrain(&p);
            let (o, w) = crate::coarse_ocean_wind_fields(&ws.field, p.gw, p.gh, p.world, ws.sea_level, &p);
            let civ = crate::compute_civilisation(
                &ws, p.gw, p.gh, p.world, p.map_width_km, p.river_density, &p.civ, &[], None, (&o, &w), false,
                &mut Vec::new(),
                None,
            );
            (p, ws, civ)
        })
    }

    /// `project_save_with_documents`' own field extraction.
    fn core_fields(ws: &cartalith_engine::WorldState, n: usize) -> cartalith_io::SaveFields {
        cartalith_io::SaveFields {
            heightmap: ws.field.clone(),
            temperature: ws.temperature.clone(),
            rainfall: ws.rainfall.clone(),
            volcanic_field: ws.volcanic_field.as_ref().clone(),
            impact_field: ws.impact_field.clone(),
            strahler_order: match ws.stream_order.as_ref() {
                Some(order) => order.iter().map(|&o| o.clamp(0, 255) as u8).collect(),
                None => vec![0u8; n],
            },
        }
    }

    /// The `SaveParams` for the fixture world: its own size, seed, width, sea level and `world`
    /// flag, origin `gen`, no name.
    fn save_params(p: &cartalith_engine::WorldParams, ws: &cartalith_engine::WorldState) -> cartalith_io::SaveParams {
        cartalith_io::SaveParams {
            gw: p.gw,
            gh: p.gh,
            seed: p.tect.seed,
            map_width_km: p.map_width_km,
            sea_level: ws.sea_level,
            world: p.world,
            origin: Some("gen".into()),
            name: None,
        }
    }

    /// The archive bytes, with or without the substrate.
    fn archive(ws: &cartalith_engine::WorldState, civ: &CivData, with_substrate: bool) -> Vec<u8> {
        let p = &world().0;
        let n = GW * GH;
        let params = save_params(p, ws);
        let fields = core_fields(ws, n);
        let mut write = ProjectWrite::new(&params, &fields);
        if with_substrate {
            substrate::write_substrate(ws, n, &mut write).expect("a generated world's substrate is writable");
        }
        let mut documents = BTreeMap::new();
        civ_documents(civ, None, &mut documents);
        civ_rasters(civ, n, &mut write);
        write.documents = documents;
        let mut buf = Vec::new();
        project::write_project(Cursor::new(&mut buf), &write).expect("write");
        buf
    }

    /// `project_open`'s order: read, rebuild the world from the save half,
    /// restore the civ layer.
    fn reopen(bytes: &[u8]) -> (Result<Option<cartalith_engine::WorldState>, String>, Option<CivData>, cartalith_io::ProjectData) {
        let mut data = project::read_project(Cursor::new(bytes)).expect("read");
        let save = data.save.clone();
        let ws = substrate::world_from_project(&mut data, &save);
        let civ = civ_from_project(&data, GW * GH, &mut Vec::new());
        (ws, civ, data)
    }

    /// Asserts the fields the archive carries are identical in `a` and `b`, floats compared by
    /// `to_bits` so a `-0.0` or one-ulp difference fails.
    fn assert_same_world(a: &cartalith_engine::WorldState, b: &cartalith_engine::WorldState) {
        assert_eq!(a.sea_level.to_bits(), b.sea_level.to_bits(), "sea_level");
        let bits = |v: &[f32]| v.iter().map(|x| x.to_bits()).collect::<Vec<_>>();
        for (name, x, y) in [
            ("field", &a.field[..], &b.field[..]),
            ("temperature", &a.temperature[..], &b.temperature[..]),
            ("rainfall", &a.rainfall[..], &b.rainfall[..]),
            ("flow_discharge", &a.flow_discharge[..], &b.flow_discharge[..]),
            ("stress_field", &a.stress_field[..], &b.stress_field[..]),
            ("shear_field", &a.shear_field[..], &b.shear_field[..]),
            ("age_field", &a.age_field[..], &b.age_field[..]),
            ("resistance_field", &a.resistance_field[..], &b.resistance_field[..]),
            ("crust_field", &a.crust_field[..], &b.crust_field[..]),
            ("volcanic_field", &a.volcanic_field[..], &b.volcanic_field[..]),
            ("impact_field", &a.impact_field[..], &b.impact_field[..]),
        ] {
            assert!(bits(x) == bits(y), "{name} differs after the round trip");
        }
        assert_eq!(a.plate_id, b.plate_id, "plate_id");
        assert_eq!(a.boundary_mask, b.boundary_mask, "boundary_mask");
        assert_eq!(a.boundary_type, b.boundary_type, "boundary_type");
        assert_eq!(a.integrated_drainage, b.integrated_drainage, "integrated_drainage");
        assert_eq!(a.stream_order, b.stream_order, "stream_order");
        assert_eq!(a.river_mask, b.river_mask, "river_mask");
        assert_eq!(a.river_floor.as_deref().map(bits), b.river_floor.as_deref().map(bits), "river_floor");
        match (&a.channels, &b.channels) {
            (Some(x), Some(y)) => {
                assert_eq!(x.recv, y.recv, "channels.recv");
                assert_eq!(x.chan, y.chan, "channels.chan");
                assert_eq!(bits(&x.intensity), bits(&y.intensity), "channels.intensity");
                assert_eq!(x.slope, y.slope, "channels.slope");
            }
            (None, None) => {}
            _ => panic!("channels present on one side only"),
        }
    }

    /// A real journey plan, through `jp_world_parts`/`jp_world`'s own
    /// construction and `jp_plan_full` -- what `jp_compute` runs -- between
    /// the first two settlements. Wildlife forage at the neutral 1.0 on both
    /// sides, so the comparison is the world's and nothing else's.
    fn plan(ws: &cartalith_engine::WorldState, civ: &CivData) -> Option<cartalith_civ::JpJourneyPlan> {
        plan_between(ws, civ, 0, 1)
    }

    /// [`plan`] between settlements `i` and `j`.
    fn plan_between(ws: &cartalith_engine::WorldState, civ: &CivData, i: usize, j: usize) -> Option<cartalith_civ::JpJourneyPlan> {
        let (a, b) = (&civ.settlements[i].placement, &civ.settlements[j].placement);
        plan_via(ws, civ, &[(a.x as f64, a.y as f64), (b.x as f64, b.y as f64)])
    }

    /// [`plan`] through the waypoints `pts`, grid coordinates.
    fn plan_via(ws: &cartalith_engine::WorldState, civ: &CivData, pts: &[(f64, f64)]) -> Option<cartalith_civ::JpJourneyPlan> {
        let p = &world().0;
        let mut jw = journey_bridge::JourneyWorld::build(
            &ws.field, &civ.water_bodies, &ws.temperature, &ws.rainfall, GW, GH, p.world, ws.sea_level, &civ.ways,
            &civ.settlements,
        );
        jw.road_cells = cartalith_civ::jp_road_cells(&civ.ways, &[], &civ.road_edges, GW);
        let (ocean, wind) = crate::coarse_ocean_wind_fields(&ws.field, GW, GH, p.world, ws.sea_level, p);
        // Ruling CD item 1: the river network `jp_world_parts` builds, built
        // the same way, so this mirror stays `jp_compute`'s construction.
        let (river_order, river_recv) = cartalith_civ::fresh_river_channels(
            &ws.field, &ws.flow_discharge, GW, GH, ws.sea_level, p.world, p.river_density, p.map_width_km,
            ws.integrated_drainage,
        );
        let jpw = cartalith_civ::JpWorld {
            gw: GW,
            gh: GH,
            world: p.world,
            map_width_km: p.map_width_km,
            sea_level: ws.sea_level,
            peak_m: p.peak_m,
            field: &ws.field,
            cart_biome: &jw.cart_biome,
            cart_terrain: &jw.cart_terrain,
            temp: &ws.temperature,
            rain: &ws.rainfall,
            flow_field: Some(&ws.flow_discharge),
            flow_thresh: cartalith_hydrology::river_flow_thresh(GW, GH, GW, p.map_width_km),
            water_bodies: Some(&civ.water_bodies),
            territory: Some(&civ.territory),
            places: &jw.places,
            road_cells: &jw.road_cells,
            ocean_field: Some(&ocean),
            wind_field: Some(&wind),
            river_net: Some(cartalith_civ::JpRiverNet { order: &river_order, recv: &river_recv }),
            // Ruling CD item 2: slope measured, as `jp_world` sets it.
            measure_slope: true,
        };
        cartalith_civ::jp_plan_full(
            &jpw,
            pts,
            &cartalith_civ::JpPlan::default(),
            &cartalith_civ::JpLayovers::new(),
            &|_, _| 1.0,
            None,
            None,
        )
    }

    /// Protects: the journey planner reads the live claim grid's convention (0 = unowned, never
    /// -1) -- an unowned cell is unclaimed, an owned one claimed, and an open-ocean route has
    /// `claimed_frac` 0 in every stage.
    /// The live claim grid, not a fixture: a real generated world's
    /// `CivData::territory` uses `0` for unowned (never `-1`), and the Journey
    /// Planner, through the same `JpWorld` construction `jp_compute` uses,
    /// reads an unowned cell as unclaimed and an owned one as claimed.
    /// Before 2026-09-26 `jp_claimed_at` tested `>= 0`, which the
    /// milestone-5 golden (captured on a `-1` grid) could not see: here every
    /// stage came back `claimed_frac == 1.0`.
    #[test]
    fn the_live_claim_grid_reads_unowned_cells_as_unclaimed() {
        let (_, ws, civ) = world();
        let n = GW * GH;
        assert_eq!(civ.territory.len(), n, "premise: a generated world has a whole claim grid");
        assert!(civ.territory.iter().all(|&t| t >= 0), "the live convention is 0 = unowned, never -1");
        // This world's capitals reach all of its land, so the unowned cells
        // are the sea's -- which the planner samples too (the reference's
        // `_jpDeriveStages` measures `claimedFrac` on sea stages as well). A
        // run of eight open-ocean cells in one row is a sea stage on unowned
        // cells.
        let ocean = |i: usize| civ.water_bodies.get(i) == Some(&1);
        let u = (0..n)
            .find(|&i| i % GW + 8 < GW && (0..=8).all(|k| ocean(i + k) && civ.territory[i + k] == 0))
            .expect("premise: the fixture world has open ocean");
        let c = (0..n).find(|&i| civ.territory[i] > 0).expect("premise: some land is claimed");
        let at = |i: usize| ((i % GW) as f64, (i / GW) as f64);
        let (ux, uy) = at(u);
        let (cx, cy) = at(c);
        assert!(!cartalith_civ::jp_claimed_at(Some(&civ.territory), GW, GH, ux, uy), "an unowned cell is not claimed");
        assert!(cartalith_civ::jp_claimed_at(Some(&civ.territory), GW, GH, cx, cy), "an owned cell is claimed");

        let journey = plan_via(ws, civ, &[(ux, uy), (ux + 8.0, uy)]).expect("an open-ocean route plans");
        let claimed: Vec<f64> = journey.stages.iter().map(|s| s.claimed_frac).collect();
        assert!(!claimed.is_empty(), "premise: the route derives a stage");
        assert!(claimed.iter().all(|&f| f == 0.0), "a route on unowned cells claims nothing: {claimed:?}");
    }

    /// `faction_economy_aggregates` over the fixture grid at the world's own sea level and map
    /// width, for generated-versus-reopened comparison.
    fn economy(ws: &cartalith_engine::WorldState, civ: &CivData) -> cartalith_civ::FactionAggregates {
        let p = &world().0;
        crate::faction_economy_aggregates(ws, civ, GW, GH, ws.sea_level, p.map_width_km)
    }

    /// Protects: Ruling AR -- a generated world saved with its substrate reopens bit for bit, and
    /// the journey plan, faction economy and way lengths computed over it equal the generated
    /// world's.
    #[test]
    fn a_saved_world_reopens_as_the_world_that_was_saved() {
        let (_, ws, civ) = world();
        assert!(ws.channels.is_some() && ws.stream_order.is_some(), "the fixture must carve rivers to test them");
        assert!(civ.settlements.len() >= 2, "the fixture needs two settlements to plan between");
        let (back, civ2, data) = reopen(&archive(ws, civ, true));
        assert!(data.warnings.is_empty(), "{:?}", data.warnings);
        let ws2 = back.expect("the substrate is honoured").expect("the archive carries a substrate");
        assert_same_world(ws, &ws2);
        assert!(ws2.gpu_stages_used.is_empty(), "nothing ran on the GPU for a reopened world");
        let civ2 = civ2.expect("the civ layer is restored");

        // The substrate half on its own: the reopened world under the
        // original civ layer plans and prices exactly as the generated one.
        let generated = plan(ws, civ);
        let g = generated.as_ref().expect("the generated world plans this journey");
        assert!(!g.stages.is_empty(), "a plan with no stages proves nothing");
        assert_eq!(plan(&ws2, civ), generated, "journey plan: generated vs reopened world");
        let e = economy(ws, civ);
        assert!(e.by_faction.iter().any(|f| f.pop > 0.0), "an economy with no population proves nothing");
        assert_eq!(economy(&ws2, civ), e, "faction economy: generated vs reopened world");

        // The whole reopened project -- its own restored civ layer too.
        assert_eq!(economy(&ws2, &civ2), e, "faction economy: generated vs reopened project");
        // Every way's length comes back bit for bit. It did not until
        // `cartalith-io` took serde_json's `float_roundtrip`: 32 of this
        // fixture's 191 came back one ulp off, and `civ_military_summary`'s
        // road density with them.
        let way_bits = |c: &CivData| {
            c.ways.iter().map(|w| (w.km.to_bits(), w.a_idx, w.b_idx)).collect::<Vec<_>>()
        };
        assert_eq!(way_bits(&civ2), way_bits(civ), "way lengths and endpoints");
        // The restored civ layer carries the router's own edges too (§9.3),
        // so this is not a coincidence of the fixture any more -- see
        // `a_reopened_project_plans_over_the_saved_road_edges`.
        assert_eq!(plan(&ws2, &civ2), generated, "journey plan: generated vs reopened project");
    }

    /// The road edges as comparable `(a, b, path)` tuples, in stored order.
    fn edge_tuples(c: &CivData) -> Vec<(usize, usize, Vec<usize>)> {
        c.road_edges.iter().map(|e| (e.a, e.b, e.path.clone())).collect()
    }

    /// `jp_road_cells`' keys, the grid cells the planner treats as road.
    fn road_keys(civ: &CivData, edges: &[cartalith_civ::RoadEdge]) -> std::collections::HashSet<(i64, i64)> {
        cartalith_civ::jp_road_cells(&civ.ways, &[], edges, GW).into_keys().collect()
    }

    /// Protects: the router's `road_edges` (endpoints and every cell) survive a reopen exactly
    /// and are stored under SAVEFILE_COMPAT.md 9.3's `from`/`to`/`cells` names.
    #[test]
    fn a_reopened_project_restores_road_edges_exactly() {
        let (_, ws, civ) = world();
        assert!(!civ.road_edges.is_empty(), "the fixture must route edges to test them");
        let (_, civ2, data) = reopen(&archive(ws, civ, true));
        assert!(data.warnings.is_empty(), "{:?}", data.warnings);
        let civ2 = civ2.expect("the civ layer is restored");
        assert_eq!(edge_tuples(&civ2), edge_tuples(civ), "road_edges: generated vs reopened, endpoints and every cell");
        // Independent of the round trip: the stored member is the list, in
        // order, under §9.3's names.
        let w: serde_json::Value = serde_json::from_str(data.text_of(SLOT_WAYS).expect("ways.json")).unwrap();
        let stored = w["road_edges"].as_array().expect("road_edges is written");
        assert_eq!(stored.len(), civ.road_edges.len());
        assert_eq!(stored[0]["from"], civ.road_edges[0].a);
        assert_eq!(stored[0]["to"], civ.road_edges[0].b);
        assert_eq!(stored[0]["cells"].as_array().unwrap().len(), civ.road_edges[0].path.len());
    }

    /// Protects: a journey along a stretch that is road only because of a stored `road_edge`
    /// plans identically to the generated world after reopening, and differently when the edges
    /// are cleared.
    /// Whether a road cell can exist that no stored way covers, and what it
    /// does to a plan. It can: the ways are consolidated and smoothed, so the
    /// router's raw cell path leaves their 3x3 footprint. A journey whose two
    /// waypoints lie on such a stretch -- a real hand-placed journey -- plans
    /// differently over the restored civ layer without the edges (every
    /// project reopened before 2026-09-24) and exactly as generated with them.
    #[test]
    fn a_reopened_project_plans_over_the_saved_road_edges() {
        let (_, ws, civ) = world();
        let (back, civ2, _) = reopen(&archive(ws, civ, true));
        let ws2 = back.unwrap().unwrap();
        let civ2 = civ2.unwrap();

        let with = road_keys(civ, &civ.road_edges);
        let without = road_keys(civ, &[]);
        let edge_only: std::collections::HashSet<(i64, i64)> = with.difference(&without).copied().collect();
        assert!(!edge_only.is_empty(), "no road cell lies off the stored ways on this fixture");

        // Before the fix: the restored civ layer with no edges.
        let mut before = civ_from_project(&data_of(ws, civ), GW * GH, &mut Vec::new()).unwrap();
        before.road_edges.clear();
        // Straight legs along a row or a column of the grid, at least `GAP`
        // cells long, every cell of which is road only because of an edge.
        const GAP: i64 = 4;
        let mut legs: Vec<[(f64, f64); 2]> = Vec::new();
        let mut sorted: Vec<(i64, i64)> = edge_only.iter().copied().collect();
        sorted.sort_unstable();
        for &(x, y) in &sorted {
            for (dx, dy) in [(1, 0), (0, 1)] {
                let run = (1..).take_while(|k| edge_only.contains(&(x + dx * k, y + dy * k))).count() as i64;
                if run >= GAP {
                    legs.push([(x as f64, y as f64), ((x + dx * run) as f64, (y + dy * run) as f64)]);
                }
            }
        }
        assert!(!legs.is_empty(), "no stretch of {GAP} road-edge-only cells on this fixture");
        let leg = legs
            .iter()
            .find(|pts| {
                let g = plan_via(ws, civ, &pts[..]);
                g.as_ref().is_some_and(|g| !g.stages.is_empty()) && plan_via(&ws2, &before, &pts[..]) != g
            })
            .expect("a journey along a road-edge-only stretch plans differently without the edges");
        let generated = plan_via(ws, civ, &leg[..]);
        assert_ne!(plan_via(&ws2, &before, &leg[..]), generated, "without road_edges the reopened plan differs {leg:?}");
        assert_eq!(plan_via(&ws2, &civ2, &leg[..]), generated, "with road_edges it matches {leg:?}");
        eprintln!(
            "road-edge-only cells: {} of {} road cells; {} candidate legs; differing leg {leg:?}",
            edge_only.len(),
            with.len(),
            legs.len()
        );
    }

    /// Protects: Ruling CD item 4's hidden-way edge filter (`civ_credited_road_edges`)
    /// changes nothing on a real generated world where the user has hidden no way.
    /// Consolidation does emit hidden *placeholder* ways (an edge whose every cell a
    /// busier edge already claimed), and the filter drops their edges; this measures
    /// that every such cell is still credited, as the same row, through the busier
    /// edge. "Before" is reconstructed by handing `jp_road_cells` only the visible
    /// ways, so its edge filter has no hidden way to match and passes every edge,
    /// exactly as the unfiltered code did.
    #[test]
    fn a_hidden_way_is_not_credited_through_its_edge_on_a_real_world() {
        let (_, _, civ) = world();
        assert!(!civ.road_edges.is_empty(), "premise: the fixture routes edges");
        let visible: Vec<cartalith_civ::Way> = civ.ways.iter().filter(|w| !w.hidden).cloned().collect();
        let placeholders = civ.ways.len() - visible.len();
        let before = cartalith_civ::jp_road_cells(&visible, &[], &civ.road_edges, GW);
        let after = cartalith_civ::jp_road_cells(&civ.ways, &[], &civ.road_edges, GW);
        let dropped = civ.road_edges.len() - cartalith_civ::tools::civ_credited_road_edges(&civ.ways, &civ.road_edges).len();
        eprintln!("ways {} (hidden placeholders {placeholders}), edges {} (dropped {dropped}), road cells {}", civ.ways.len(), civ.road_edges.len(), after.len());
        assert!(!before.is_empty(), "premise: the planner credits road cells");
        assert_eq!(after, before, "no road cell is lost or re-classed when only placeholders are hidden");
    }

    /// The archive `reopen` reads, read again, for a second restore.
    fn data_of(ws: &cartalith_engine::WorldState, civ: &CivData) -> cartalith_io::ProjectData {
        project::read_project(Cursor::new(&archive(ws, civ, true))).expect("read")
    }

    /// Protects: a stored road edge with a cell off the grid is dropped alone on reopen and
    /// reported in exactly one warning.
    #[test]
    fn a_road_edge_with_a_cell_off_the_grid_is_dropped_and_reported() {
        let (_, ws, civ) = world();
        let mut bad = civ_from_project(&data_of(ws, civ), GW * GH, &mut Vec::new()).unwrap();
        let kept = bad.road_edges.len();
        bad.road_edges.push(cartalith_civ::RoadEdge { a: 0, b: 1, path: vec![5, GW * GH] });
        let mut warnings = Vec::new();
        let back = civ_from_project(&data_of(ws, &bad), GW * GH, &mut warnings).unwrap();
        assert_eq!(back.road_edges.len(), kept, "the off-grid edge alone is dropped");
        assert_eq!(warnings.len(), 1, "{warnings:?}");
        assert!(warnings[0].contains(&format!("1 of {} road edges", kept + 1)), "{}", warnings[0]);
    }

    /// Protects: Ruling AT's held year survives the round trip, and a stored year that
    /// names no recorded snapshot is not believed.
    #[test]
    fn the_held_territory_year_round_trips_and_a_dangling_one_is_dropped() {
        let (_, ws, civ) = world();
        let mut held = civ_from_project(&data_of(ws, civ), GW * GH, &mut Vec::new()).unwrap();
        assert!(held.civ_add_year(300), "a first recorded year loads");
        assert_eq!(held.territory_year, Some(300));
        let back = civ_from_project(&data_of(ws, &held), GW * GH, &mut Vec::new()).unwrap();
        assert_eq!(back.territory_year, Some(300));
        held.territory_year = Some(301);
        let back = civ_from_project(&data_of(ws, &held), GW * GH, &mut Vec::new()).unwrap();
        assert_eq!(back.territory_year, None, "301 is not a recorded year");
    }

    /// Protects: backward compatibility with a real pre-substrate archive -- it reads,
    /// `world_from_project` refuses it as no substrate without taking rasters, its civ layer
    /// restores with no `road_edges` and no warning, and re-saved ways/timeline documents are
    /// byte-identical.
    #[test]
    fn a_project_saved_before_the_substrate_opens_and_says_so() {
        // `MISTAKES.md`: a backward-compatibility fixture built from a real
        // prior-format archive, not a reconstruction. Provenance: written by
        // `_biomesave_probe.gd -- --capture` through `WorldGen::project_save`
        // on 2026-09-24 (archive mtime 10:04), by a build before this
        // change -- `project.json` has no `substrate` member and `rasters/`
        // holds only the six core grids plus the civ layer's four.
        let bytes = include_bytes!("../tests/fixtures/project_pre_substrate_2026-09-24.zip");
        let mut data = project::read_project(Cursor::new(&bytes[..])).expect("a pre-substrate project still reads");
        assert!(data.substrate.is_null());
        assert_eq!(data.core_substituted.as_deref(), Some(&[][..]));
        let save = data.save.clone();
        let before = data.rasters.len();
        assert!(matches!(substrate::world_from_project(&mut data, &save), Ok(None)), "no member is no substrate, not an error");
        assert_eq!(data.rasters.len(), before, "a refusal takes nothing");
        let n = save.params.gw * save.params.gh;
        let mut warnings = Vec::new();
        let civ = civ_from_project(&data, n, &mut warnings).expect("its civ layer still restores");
        assert!(!civ.settlements.is_empty());
        // §9.3's `road_edges` is absent from it, and that is no edges -- how
        // it always reopened -- not a warning, and not an invented network.
        assert!(!data.text_of(SLOT_WAYS).expect("it has ways").contains("road_edges"));
        assert!(civ.road_edges.is_empty() && warnings.is_empty(), "{warnings:?}");
        assert_eq!(civ.territory_year, None);
        // Re-saved, its ways and timeline documents are byte-identical: an
        // absent member is written back absent, never as an empty claim.
        let mut docs = BTreeMap::new();
        civ_documents(&civ, None, &mut docs);
        for slot in [SLOT_WAYS, SLOT_TIMELINE] {
            let Some(old) = data.text_of(slot) else { continue };
            assert_eq!(docs[slot], old, "{slot} re-serialises byte for byte");
        }
        // What every refusing readout now says of it: the substrate, not the
        // civ layer, which this very archive restores.
        assert!(!substrate::NEEDS_SUBSTRATE.contains("civilisation layer"));
        assert!(substrate::NEEDS_SUBSTRATE.contains("regenerate"));
    }

    /// The end cells of every road edge, decoded on today's grid by the
    /// readers' own map (`cartalith_civ::road_edge_map_path`), and the largest
    /// Chebyshev distance, in map cells, from an edge's two ends to its two
    /// settlements (either orientation); edges whose settlements are out of
    /// range are skipped. The measure the 384-cap fixture test reports.
    fn worst_end_gap(civ: &CivData, gw: usize) -> (usize, u64) {
        let mut judged = 0;
        let mut worst = 0u64;
        for e in &civ.road_edges {
            let (Some(a), Some(b)) = (civ.settlements.get(e.a), civ.settlements.get(e.b)) else { continue };
            let (pts, _) = cartalith_civ::road_edge_map_path(&e.path, gw);
            let (Some(&f), Some(&l)) = (pts.first(), pts.last()) else { continue };
            let d = |p: (f64, f64), s: &cartalith_civ::NamedSettlement| {
                (p.0 - s.placement.x as f64).abs().max((p.1 - s.placement.y as f64).abs()) as u64
            };
            worst = worst.max((d(f, b) + d(l, a)).min(d(f, a) + d(l, b)));
            judged += 1;
        }
        (judged, worst)
    }

    /// Protects: a project saved while the routing cap was 384
    /// (OUTSTANDING_WORK.md "Saves written while `RW_CAP` was 384") reopens
    /// with its road edges where the router laid them. The fixture is real:
    /// `project.json`, `entities/settlements.json`, `ways.json` and
    /// `factions.json` copied byte for byte from an archive written by the
    /// pre-Ruling-BT tree (`1127dfef~1`, cap 384) through `substrate_tests`'s
    /// own `archive()` on a 512 x 328 world, seed 24601 (2026-10-12). Its six
    /// core rasters are zero-filled at their real sizes and the rest left out,
    /// to keep the fixture at 35 KB: nothing here reads them.
    /// Its 55 edges index a 384-wide grid and carry no grid member.
    ///
    /// Measured (printed): read on today's grid as before this fix, the edge
    /// ends land far from their settlements; re-gridded, every judged edge ends
    /// within 2 cells of both (the old grid's own rounding, `1 / 0.75`). A
    /// re-save writes `road_edges_rw_cap` and re-reads with no move and no
    /// warning, the paths unchanged.
    #[test]
    fn a_project_saved_under_the_384_cap_reopens_with_its_road_edges_in_place() {
        let bytes = include_bytes!("../tests/fixtures/project_384cap_512x328_2026-09-28.zip");
        let data = project::read_project(Cursor::new(&bytes[..])).expect("the fixture reads");
        let (gw, gh) = (data.save.params.gw, data.save.params.gh);
        assert_eq!((gw, gh), (512, 328), "premise: wider than 384, under 1024");
        let ways: serde_json::Value = serde_json::from_str(data.text_of(SLOT_WAYS).expect("ways")).unwrap();
        assert!(ways.get("road_edges_rw_cap").is_none(), "premise: written before the member");
        assert_eq!(ways["road_edges"].as_array().map(|a| a.len()), Some(55));

        // As read before this fix: the stored cells taken as today's grid.
        let mut raw_civ = civ_from_project(&data, gw * gh, &mut Vec::new()).expect("restores");
        let stored: Vec<Vec<usize>> = ways["road_edges"].as_array().unwrap().iter()
            .map(|e| e["cells"].as_array().unwrap().iter().map(|c| c.as_u64().unwrap() as usize).collect()).collect();
        for (e, s) in raw_civ.road_edges.iter_mut().zip(&stored) {
            e.path = s.clone();
        }
        let (judged_raw, worst_raw) = worst_end_gap(&raw_civ, gw);

        let mut warnings = Vec::new();
        let civ = civ_from_project(&data, gw * gh, &mut warnings).expect("restores");
        assert_eq!(civ.road_edges.len(), 55, "no edge lost: {warnings:?}");
        let (judged, worst) = worst_end_gap(&civ, gw);
        eprintln!("384-cap fixture: {judged_raw} edges judged; worst end gap read as today's grid {worst_raw} cells, re-gridded {worst} cells");
        assert!(judged >= 50, "premise: the edges' settlements are in range ({judged})");
        assert!(worst_raw > 50, "premise: the old reading really is misplaced ({worst_raw})");
        assert!(worst <= 2, "every re-gridded edge ends at its settlements ({worst})");
        assert!(warnings.iter().any(|w| w.contains("55 road edges moved from a 384-cell-wide routing grid to today's 512")), "{warnings:?}");
        // Continuity: an old path with no break (every step 8-adjacent on
        // the 384 grid) is still unbroken on today's grid.
        for (e, s) in civ.road_edges.iter().zip(&stored) {
            let old_unbroken = s.windows(2).all(|p| (p[0] % 384).abs_diff(p[1] % 384) <= 1 && (p[0] / 384).abs_diff(p[1] / 384) <= 1);
            if old_unbroken {
                assert!(cartalith_civ::road_edge_map_path(&e.path, gw).1.is_empty(), "re-gridding adds no break");
            }
        }

        // Re-saved: the member is written, and the re-read is the identity.
        let mut docs = BTreeMap::new();
        civ_documents(&civ, None, &mut docs);
        let w2: serde_json::Value = serde_json::from_str(&docs[SLOT_WAYS]).unwrap();
        assert_eq!(w2["road_edges_rw_cap"], 1024);
        let mut data2 = data.clone();
        data2.documents.insert(SLOT_WAYS.to_string(), w2.clone());
        data2.document_text.insert(SLOT_WAYS.to_string(), docs[SLOT_WAYS].clone());
        let mut warnings2 = Vec::new();
        let back = civ_from_project(&data2, gw * gh, &mut warnings2).expect("restores");
        // (The fixture omits the civ rasters, so territory/provinces warn;
        // nothing may be said about road edges.)
        assert!(!warnings2.iter().any(|w| w.contains("road edge")), "{warnings2:?}");
        assert_eq!(back.road_edges, civ.road_edges);
    }

    /// The real pre-substrate archive `a_project_saved_before_the_substrate_
    /// opens_and_says_so` documents the provenance of.
    const PRE_SUBSTRATE: &[u8] = include_bytes!("../tests/fixtures/project_pre_substrate_2026-09-24.zip");

    /// Protects: a missing `rasters/territory.i32` reopens as no territory (not zeros) with one
    /// warning, is not written back as zeros, and a later stroke commits onto a whole grid
    /// claiming only its own cells.
    #[test]
    fn an_archive_without_its_claim_grid_reopens_with_no_territory_and_says_so() {
        // Control: the real archive, whole.
        let data = project::read_project(Cursor::new(PRE_SUBSTRATE)).expect("the fixture reads");
        let (gw, gh) = (data.save.params.gw, data.save.params.gh);
        let n = gw * gh;
        let mut warnings = Vec::new();
        let whole = civ_from_project(&data, n, &mut warnings).expect("its civ layer restores");
        assert_eq!(whole.territory.len(), n);
        assert!(whole.territory.iter().any(|&t| t > 0), "premise: the fixture holds claims");
        assert!(warnings.is_empty(), "{warnings:?}");

        // The same archive with that one raster taken out.
        let stripped = without(PRE_SUBSTRATE, "rasters/territory.shuffled.i32");
        let data = project::read_project(Cursor::new(&stripped[..])).expect("still reads");
        assert!(data.raster("rasters/territory.i32").is_none(), "premise: the raster is gone");
        let mut warnings = Vec::new();
        let mut civ = civ_from_project(&data, n, &mut warnings).expect("its civ layer still restores");
        assert!(civ.territory.is_empty(), "absent is no territory, not {} unowned cells", civ.territory.len());
        assert_eq!(warnings.len(), 1, "{warnings:?}");
        assert!(warnings[0].starts_with("rasters/territory.i32: absent"), "{warnings:?}");
        assert_eq!(civ.settlements.len(), whole.settlements.len(), "everything else still restores");

        // A re-save leaves it absent rather than writing the unknown as zeros.
        let mut write = ProjectWrite::new(&data.save.params, &data.save.fields);
        civ_rasters(&civ, n, &mut write);
        assert!(!write.rasters.contains_key("rasters/territory.i32"));
        assert!(write.rasters.contains_key("rasters/provinces.i32"), "premise: the writer ran");

        // The tool's base is "nothing claimed" at this world's size, so a
        // stroke commits onto a whole grid and claims exactly its own cells.
        let mut tools = civ_tools_for_reopen(gw, gh, &civ, data.save.params.seed as u32, &data);
        assert_eq!(tools.territory_base, vec![0; n]);
        assert!(tools.paint_at(10.0, 10.0, 2, 3.0, false));
        assert!(tools.commit(&mut civ.territory));
        assert_eq!(civ.territory.len(), n);
        assert_eq!(civ.territory[10 * gw + 10], 2);
        assert!(civ.territory.iter().all(|&t| t == 0 || t == 2), "only the stroke is claimed");
    }

    /// Protects: `OUTSTANDING_WORK.md` §2.11: the province raster had the defect
    /// `an_archive_without_its_claim_grid_...` above fixed for territory -- a
    /// missing `rasters/provinces.i32` came back as a grid of zeros ("no cell
    /// is in any province"), which the next save wrote back as if read.
    #[test]
    fn an_archive_without_its_province_raster_reopens_with_none_and_says_so() {
        let data = project::read_project(Cursor::new(PRE_SUBSTRATE)).expect("the fixture reads");
        let (gw, gh) = (data.save.params.gw, data.save.params.gh);
        let n = gw * gh;
        let whole = civ_from_project(&data, n, &mut Vec::new()).expect("its civ layer restores");
        assert_eq!(whole.provinces.len(), n);
        assert!(whole.provinces.iter().any(|&p| p > 0), "premise: the fixture holds provinces");

        let stripped = without(PRE_SUBSTRATE, "rasters/provinces.shuffled.i32");
        let data = project::read_project(Cursor::new(&stripped[..])).expect("still reads");
        assert!(data.raster("rasters/provinces.i32").is_none(), "premise: the raster is gone");
        let mut warnings = Vec::new();
        let mut civ = civ_from_project(&data, n, &mut warnings).expect("its civ layer still restores");
        assert!(civ.provinces.is_empty(), "absent is unknown, not {} cells of no province", civ.provinces.len());
        assert_eq!(warnings.len(), 1, "{warnings:?}");
        assert!(warnings[0].starts_with("rasters/provinces.i32: absent"), "{warnings:?}");
        assert_eq!(civ.province_list.len(), whole.province_list.len(), "the list is its own document and still restores");
        assert!(!civ.province_list.is_empty());
        assert_eq!(civ.territory, whole.territory, "the claim grid is untouched");

        // A re-save leaves it absent rather than writing the unknown as zeros.
        let mut write = ProjectWrite::new(&data.save.params, &data.save.fields);
        civ_rasters(&civ, n, &mut write);
        assert!(!write.rasters.contains_key("rasters/provinces.i32"));
        assert!(write.rasters.contains_key("rasters/territory.i32"), "premise: the writer ran");

        // A rebuild over the whole claim grid makes it whole again.
        crate::civ_reprovince(&mut civ, gw, gh);
        assert_eq!(civ.provinces.len(), n);
        assert!(civ.provinces.iter().any(|&p| p > 0));
    }

    /// Protects: the manual settlement name stream's saved position -- a reopened session
    /// continues the sequence the saved session would have drawn, an archive without the member
    /// restarts at the seed, and no stream writes no member.
    #[test]
    fn a_reopened_project_resumes_the_manual_name_stream() {
        use crate::civ_tools_bridge::{manual_settlement_name, manual_settlement_pop, CivTools};
        let mut data = project::read_project(Cursor::new(PRE_SUBSTRATE)).expect("the fixture reads");
        let (gw, gh) = (data.save.params.gw, data.save.params.gh);
        let seed = data.save.params.seed as u32;
        let civ = civ_from_project(&data, gw * gh, &mut Vec::new()).expect("its civ layer restores");
        assert_eq!(civ_name_stream(&data), None, "premise: written before the member existed");
        // One blank-named drop's draws, in `drop_settlement`'s order.
        let culture = cartalith_civ::civ_default_culture(1);
        let drop = |t: &mut CivTools| {
            let name = manual_settlement_name("", culture, &mut t.name_rng);
            manual_settlement_pop(cartalith_civ::SettlementKind::Town, 0.0, &mut t.name_rng);
            name
        };

        let mut session = civ_tools_for_reopen(gw, gh, &civ, seed, &data);
        let before: Vec<String> = (0..5).map(|_| drop(&mut session)).collect();
        // The defect: an archive without the member restarts at the seed.
        let mut restarted = civ_tools_for_reopen(gw, gh, &civ, seed, &data);
        assert_eq!(drop(&mut restarted), before[0], "premise: without it, the first name repeats");

        // Save with the stream's position, reopen, place five more.
        let mut docs = BTreeMap::new();
        civ_documents(&civ, Some(session.name_rng.state()), &mut docs);
        assert!(docs[SLOT_SETTLEMENTS].contains("\"name_stream\":"), "{}", &docs[SLOT_SETTLEMENTS][..80]);
        data.documents.insert(SLOT_SETTLEMENTS.to_string(), serde_json::from_str(&docs[SLOT_SETTLEMENTS]).unwrap());
        let mut reopened = civ_tools_for_reopen(gw, gh, &civ, seed, &data);
        let after: Vec<String> = (0..5).map(|_| drop(&mut reopened)).collect();
        let unbroken: Vec<String> = (0..5).map(|_| drop(&mut session)).collect();
        assert_eq!(after, unbroken, "the reopened stream is the session's own continuation");
        for a in &after {
            assert!(!before.contains(a), "{a} was drawn before the save as well: {before:?}");
        }
        // Absent stays absent: no stream, no member.
        let mut none = BTreeMap::new();
        civ_documents(&civ, None, &mut none);
        assert!(!none[SLOT_SETTLEMENTS].contains("name_stream"));
    }

    /// The archive without one entry, copied raw.
    fn without(bytes: &[u8], entry: &str) -> Vec<u8> {
        let mut src = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
        let mut out = Vec::new();
        {
            let mut w = zip::ZipWriter::new(Cursor::new(&mut out));
            for i in 0..src.len() {
                let e = src.by_index_raw(i).unwrap();
                if e.name() == entry {
                    continue;
                }
                w.raw_copy_file(e).unwrap();
            }
            w.finish().unwrap();
        }
        out
    }

    /// Protects: an archive missing a promised substrate raster or the core stream-order raster
    /// is refused whole, with an error naming the entry, and takes no rasters.
    #[test]
    fn a_damaged_substrate_is_refused_whole_and_names_what_is_missing() {
        let (_, ws, civ) = world();
        let bytes = archive(ws, civ, true);
        // A promised substrate raster, and the core raster stream order is
        // rebuilt from -- whose absence would otherwise read as "no rivers".
        for (entry, named) in [
            ("rasters/river_mask.u8", "rasters/river_mask.u8"),
            ("rasters/strahler_order.u8", "rasters/strahler_order.u8"),
        ] {
            let mut data = project::read_project(Cursor::new(&without(&bytes, entry))).unwrap();
            let save = data.save.clone();
            let before = data.rasters.len();
            let err = substrate::world_from_project(&mut data, &save).err().expect("refused");
            assert!(err.contains(named), "{err}");
            assert_eq!(data.rasters.len(), before, "a refusal takes nothing");
        }
    }

    /// Protects: a substrate manifest of another version (2) is refused with an error naming that
    /// version.
    #[test]
    fn a_substrate_member_of_another_version_is_refused() {
        let (_, ws, civ) = world();
        let (_, _, mut data) = reopen(&archive(ws, civ, true));
        // `reopen` consumed the rasters; the manifest check comes first.
        data.substrate["version"] = serde_json::json!(2);
        let save = data.save.clone();
        let err = substrate::world_from_project(&mut data, &save).err().expect("refused");
        assert!(err.contains("version 2"), "{err}");
    }

    /// Protects: the substrate manifest records each optional grid as present or absent as it
    /// was; a world with no carve writes only the nine always-present grids and reopens with
    /// those grids `None`.
    #[test]
    fn the_manifest_records_each_optional_grid_as_it_was() {
        let (_, ws, _) = world();
        let (rasters, m) = substrate::substrate_rasters(ws, GW * GH).unwrap();
        assert_eq!(
            m,
            SubstrateManifest {
                version: 1,
                integrated_drainage: ws.integrated_drainage,
                channels: true,
                river_intensity: !ws.channels.as_ref().unwrap().intensity.is_empty(),
                stream_order: true,
                river_mask: true,
                river_floor: true,
            }
        );
        let names: Vec<&str> = rasters.iter().map(|(p, _)| *p).collect();
        for p in cartalith_io::SUBSTRATE_RASTERS {
            let optional = p == "rasters/river_intensity.f32" && !m.river_intensity;
            assert_eq!(names.contains(&p), !optional, "{p}");
        }

        // A world with no carve: every optional grid absent, none written as
        // zeros, and it comes back `None` rather than as an empty lock.
        let mut bare = clone_ws(ws);
        bare.channels = None;
        bare.stream_order = None;
        bare.river_mask = None;
        bare.river_floor = None;
        let (rasters, m) = substrate::substrate_rasters(&bare, GW * GH).unwrap();
        assert!(!m.channels && !m.stream_order && !m.river_mask && !m.river_floor && !m.river_intensity);
        assert_eq!(rasters.len(), 9, "the nine always-present grids and nothing else");
        let (back, _, _) = reopen(&archive(&bare, &world().2, true));
        let back = back.unwrap().unwrap();
        assert!(back.channels.is_none() && back.stream_order.is_none());
        assert!(back.river_mask.is_none() && back.river_floor.is_none());
    }

    /// Protects: a stream order of 256, which the core u8 raster cannot hold, makes
    /// `substrate_rasters` refuse (naming 256) rather than truncate.
    #[test]
    fn a_stream_order_the_core_raster_cannot_hold_writes_no_substrate() {
        let (_, ws, _) = world();
        let mut w = clone_ws(ws);
        w.stream_order.as_mut().unwrap()[0] = 256;
        let err = substrate::substrate_rasters(&w, GW * GH).err().expect("refused");
        assert!(err.contains("256"), "{err}");
    }

    /// Copies each `WorldState` field by hand so a test can null out optional grids;
    /// `gpu_stages_used` starts empty.
    fn clone_ws(ws: &cartalith_engine::WorldState) -> cartalith_engine::WorldState {
        cartalith_engine::WorldState {
            sea_level: ws.sea_level,
            field: ws.field.clone(),
            plate_id: ws.plate_id.clone(),
            boundary_mask: ws.boundary_mask.clone(),
            stress_field: ws.stress_field.clone(),
            age_field: ws.age_field.clone(),
            resistance_field: ws.resistance_field.clone(),
            crust_field: ws.crust_field.clone(),
            boundary_type: ws.boundary_type.clone(),
            shear_field: ws.shear_field.clone(),
            volcanic_field: ws.volcanic_field.clone(),
            impact_field: ws.impact_field.clone(),
            temperature: ws.temperature.clone(),
            rainfall: ws.rainfall.clone(),
            flow_discharge: ws.flow_discharge.clone(),
            integrated_drainage: ws.integrated_drainage,
            channels: ws.channels.as_ref().map(|c| cartalith_hydrology::ChannelResult {
                recv: c.recv.clone(),
                chan: c.chan.clone(),
                slope: c.slope.clone(),
                intensity: c.intensity.clone(),
            }),
            stream_order: ws.stream_order.clone(),
            river_mask: ws.river_mask.clone(),
            river_floor: ws.river_floor.clone(),
            gpu_stages_used: Vec::new(),
            geology: ws.geology.clone(),
            volcanoes: ws.volcanoes.clone(),
        }
    }
}

/// GF-8's save slice (`SAVEFILE_COMPAT.md` §8.4, `GEOLOGY_FIRST_SCOPE.md`
/// §2.6, `geology_save.rs`): the geology column round-trips a real archive bit
/// for bit, painted mask and construction record included; a project saved
/// before the set opens with no column and re-saves with no geology entries;
/// and every refusal the reader states actually fires.
#[cfg(test)]
mod geology_save_tests {
    use super::*;
    use crate::geology_save::{self, GeologyManifest};
    use crate::substrate;
    use cartalith_terrain::geology::{Construction, ConstructionSkip, GeologyColumn, NO_LAYER};
    use std::io::Cursor;

    /// Fixture width: a labelled judgement, the substrate tests' own size (big
    /// enough to carve rivers, small enough to generate several times).
    const GW: usize = 160;
    /// Fixture height, paired with `GW`.
    const GH: usize = 112;

    /// One world at `seed` on the shell's own parameters (geology model on,
    /// as the app ships), CPU only; `processes` turns GF-2's rock-reading
    /// erosion and GF-11's relief model on, so deposition leaves regolith and
    /// construction is asked for.
    fn world(seed: i32, processes: bool) -> (cartalith_engine::WorldParams, cartalith_engine::WorldState) {
        let mut p = crate::params::defaults();
        p.gw = GW;
        p.gh = GH;
        p.tect.seed = seed;
        p.use_gpu = false;
        p.geology_processes = processes;
        p.relief_model = processes;
        let ws = cartalith_engine::generate_terrain(&p);
        (p, ws)
    }

    /// Paints a block of shale with no second layer into `ws`'s column through
    /// the commit's own bake, so the mask, the NaN contacts and the changed
    /// rock are the ones a real Rock commit writes. Returns the cells baked.
    fn paint_block(p: &cartalith_engine::WorldParams, ws: &mut cartalith_engine::WorldState) -> usize {
        let n = GW * GH;
        let (mut top, mut sub, cap) = (vec![0u8; n], vec![0u8; n], vec![0u8; n]);
        for y in 40..56 {
            for x in 60..84 {
                top[y * GW + x] = cartalith_terrain::geology::Rock::Shale as u8 + 1;
                sub[y * GW + x] = cartalith_engine::rock_paint::SUB_NONE;
            }
        }
        let field = std::sync::Arc::clone(&ws.field);
        let sea = ws.sea_level;
        let cartalith_engine::Geology::Column(col) = &mut ws.geology else { panic!("premise: the world has a column") };
        let paint = cartalith_engine::rock_paint::RockPaint { top: &top, sub: &sub, cap: &cap };
        cartalith_engine::rock_paint::bake(col, &field, &paint, GW, sea, p.peak_m).expect("bake").cells_baked
    }

    /// The `SaveParams` for a fixture world.
    fn save_params(p: &cartalith_engine::WorldParams, ws: &cartalith_engine::WorldState) -> cartalith_io::SaveParams {
        cartalith_io::SaveParams {
            gw: p.gw,
            gh: p.gh,
            seed: p.tect.seed,
            map_width_km: p.map_width_km,
            sea_level: ws.sea_level,
            world: p.world,
            origin: Some("gen".into()),
            name: None,
        }
    }

    /// The archive `project_save_with_documents` writes for this world's
    /// terrain, substrate and geology (no civ layer: nothing here reads it),
    /// and whether the geology set was written.
    fn archive(p: &cartalith_engine::WorldParams, ws: &cartalith_engine::WorldState) -> (Vec<u8>, bool) {
        let n = GW * GH;
        let params = save_params(p, ws);
        let fields = cartalith_io::SaveFields {
            heightmap: ws.field.clone(),
            temperature: ws.temperature.clone(),
            rainfall: ws.rainfall.clone(),
            volcanic_field: ws.volcanic_field.as_ref().clone(),
            impact_field: ws.impact_field.clone(),
            strahler_order: match ws.stream_order.as_ref() {
                Some(o) => o.iter().map(|&v| v.clamp(0, 255) as u8).collect(),
                None => vec![0u8; n],
            },
        };
        let mut write = ProjectWrite::new(&params, &fields);
        substrate::write_substrate(ws, n, &mut write).expect("substrate");
        let wrote = geology_save::write_geology(ws, GW, GH, p.geo_age, &mut write).expect("geology");
        let mut buf = Vec::new();
        project::write_project(Cursor::new(&mut buf), &write).expect("write");
        (buf, wrote)
    }

    /// `project_open`'s own order: the substrate rebuilds the world, then the
    /// column goes onto it.
    fn reopen(bytes: &[u8]) -> (cartalith_engine::WorldState, &'static str, Option<String>, cartalith_io::ProjectData) {
        let mut data = project::read_project(Cursor::new(bytes)).expect("read");
        let save = data.save.clone();
        let mut ws = substrate::world_from_project(&mut data, &save).expect("substrate honoured").expect("substrate present");
        let (g, w) = geology_save::restore_into(&mut ws, &mut data, save.params.gw, save.params.gh);
        (ws, g, w, data)
    }

    /// Every member of two columns, floats by bits (a NaN payload or a `-0.0`
    /// that moved fails).
    fn assert_same_column(a: &GeologyColumn, b: &GeologyColumn) {
        let bits = |v: &[f32]| v.iter().map(|x| x.to_bits()).collect::<Vec<_>>();
        assert_eq!(a.rock_top, b.rock_top, "rock_top");
        assert_eq!(a.rock_sub, b.rock_sub, "rock_sub");
        assert_eq!(bits(&a.contact), bits(&b.contact), "contact");
        assert_eq!(bits(&a.regolith), bits(&b.regolith), "regolith");
        assert_eq!(a.volcanic_setting, b.volcanic_setting, "volcanic_setting");
        assert_eq!(a.painted, b.painted, "painted");
        match (&a.construction, &b.construction) {
            (Construction::Built { structural_surface: x }, Construction::Built { structural_surface: y }) => {
                assert_eq!(bits(x), bits(y), "structural_surface")
            }
            (x, y) => assert_eq!(x, y, "construction"),
        }
    }

    /// Protects: GF-8's "the save round trip is byte-identical for the column
    /// and the mask" -- at two seeds, one painted through the real bake (mask
    /// `Some`, NaN contacts under the painted single-layer cells) with
    /// processes off as the app ships, one with processes and the relief
    /// model on (regolith from deposition, a construction record) and nobody
    /// painting, whose absent mask must come back absent, never as zeros.
    #[test]
    fn a_saved_column_reopens_bit_for_bit_with_its_painted_mask() {
        for (seed, processes) in [(24601, false), (483920, true)] {
            let (p, mut ws) = world(seed, processes);
            let baked = if processes { 0 } else { paint_block(&p, &mut ws) };
            let col = ws.geology.column().expect("premise: the shell's parameters build a column").clone();
            if processes {
                assert!(col.painted.is_none(), "premise: nobody painted");
                assert!(col.regolith.iter().any(|&r| r > 0.0), "premise: deposition left regolith");
                assert!(!matches!(col.construction, Construction::Off), "premise: the relief model was asked for");
            } else {
                assert!(baked > 0, "premise: the bake painted cells");
                assert_eq!(col.painted.as_ref().map(|m| m.iter().filter(|&&v| v == 1).count()), Some(baked));
                assert!(col.contact.iter().any(|c| c.is_nan()) && col.contact.iter().any(|c| c.is_finite()), "premise: both layer kinds");
            }
            let (bytes, wrote) = archive(&p, &ws);
            assert!(wrote, "a world with a column writes the set");
            let (ws2, status, warning, data) = reopen(&bytes);
            assert_eq!((status, warning), ("complete", None), "seed {seed}");
            assert!(data.warnings.is_empty(), "{:?}", data.warnings);
            assert_same_column(&col, ws2.geology.column().expect("restored"));
            // The member says what it holds, checked against literals.
            let m: GeologyManifest = serde_json::from_value(data.geology.clone()).unwrap();
            assert_eq!((m.version, m.grid_width, m.grid_height), (1, 160, 112));
            assert_eq!(m.painted, !processes);
            assert_eq!(m.age.to_bits(), p.geo_age.to_bits());
            // The reader took every geology raster it used.
            for path in cartalith_io::GEOLOGY_RASTERS {
                assert!(data.raster(path).is_none(), "{path} left behind after a restore");
            }
        }
    }

    /// Protects VA-1's save (Ruling CW, `SAVEFILE_COMPAT.md` §8.4
    /// `volcanoes`): a generated world's volcano table reopens equal, every
    /// float by bits; a world without one writes no key and reopens without
    /// one (absent, never empty); a damaged table is refused whole with the
    /// column still restored and the refusal in the warning; and the two
    /// checked-in prior-format archives carry no member, so they open with no
    /// table (the fixture test below holds that).
    #[test]
    fn the_volcano_table_round_trips_and_absent_stays_absent() {
        let (p, mut ws) = world(24601, false);
        let table = ws.volcanoes.clone().expect("premise: the shell's parameters keep a volcano table");
        assert!(!table.is_empty());
        let (ws2, status, warning, data) = reopen(&archive(&p, &ws).0);
        assert_eq!((status, warning), ("complete", None));
        let back = ws2.volcanoes.expect("the table reopens");
        assert_eq!(back.len(), table.len());
        for (a, b) in table.iter().zip(back.iter()) {
            assert_eq!((a.setting, a.kind), (b.setting, b.kind));
            for (x, y) in [(a.cx, b.cx), (a.cy, b.cy), (a.r, b.r), (a.h, b.h), (a.age, b.age)] {
                assert_eq!(x.to_bits(), y.to_bits());
            }
            // Ruling CX item 2: a new world's rows carry their years, and
            // they come back bit for bit.
            assert!(a.years.is_some(), "premise: a new world records each edifice's years");
            assert_eq!(a.years.map(f64::to_bits), b.years.map(f64::to_bits));
            // Ruling DB item 1: a rift row's trend comes back bit for bit,
            // and only a rift row carries one.
            assert_eq!(a.rift_bearing.map(f64::to_bits), b.rift_bearing.map(f64::to_bits));
            assert!(a.rift_bearing.is_none() || a.setting == cartalith_terrain::VolcanicSetting::Rift);
        }
        // A table saved before Ruling CX carries no `years` key (VA-1's row
        // shape): the real table's own rows, serialised, with the key taken
        // out. It loads with no years, so every edifice is read under Ruling
        // CW's map exactly as it was saved, and re-serialises to the same
        // JSON (no `years` key gained).
        let old_json: Vec<serde_json::Value> = volcano_rows_of(&table)
            .iter()
            .map(|r| {
                let mut v = serde_json::to_value(r).unwrap();
                assert!(v.as_object_mut().unwrap().remove("years").is_some(), "the writer writes the key");
                // A pre-CX table predates Ruling DB's trend too.
                v.as_object_mut().unwrap().remove("rift_bearing");
                v
            })
            .collect();
        let old_rows: Vec<geology_save::VolcanoRow> = old_json.iter().map(|v| serde_json::from_value(v.clone()).unwrap()).collect();
        let old = geology_save::volcanoes_from_rows(&old_rows).expect("a pre-CX table still loads");
        assert!(old.iter().all(|e| e.years.is_none()));
        for (e, t) in old.iter().zip(table.iter()) {
            assert_eq!(cartalith_terrain::volcano_attr::years(e), cartalith_terrain::volcano_attr::legacy_age_years(t.age));
            assert_eq!(cartalith_terrain::volcano_attr::activity(e), cartalith_terrain::volcano_attr::legacy_activity_of(t.age));
        }
        let again: Vec<serde_json::Value> = volcano_rows_of(&old).iter().map(|r| serde_json::to_value(r).unwrap()).collect();
        assert_eq!(again, old_json, "a pre-CX table re-serialises without gaining a key");
        let m: GeologyManifest = serde_json::from_value(data.geology.clone()).unwrap();
        assert_eq!(m.volcanoes.as_ref().map(Vec::len), Some(table.len()));
        // Absent stays absent.
        ws.volcanoes = None;
        let (bytes, _) = archive(&p, &ws);
        let (ws3, status, warning, data) = reopen(&bytes);
        assert_eq!((status, warning), ("complete", None));
        assert!(ws3.volcanoes.is_none());
        assert!(data.geology.get("volcanoes").is_none(), "{}", data.geology);
        // A damaged row refuses the table, never the column.
        let mut rows = volcano_rows_of(&table);
        rows[0].kind = Some("dome".to_string());
        assert!(geology_save::volcanoes_from_rows(&rows).unwrap_err().contains("unknown kind"));
        let mut rows = volcano_rows_of(&table);
        rows[1].age = 1.5;
        assert!(geology_save::volcanoes_from_rows(&rows).is_err());
        let mut rows = volcano_rows_of(&table);
        rows[0].r = 0.0;
        assert!(geology_save::volcanoes_from_rows(&rows).is_err(), "a zero radius is refused");
        assert!(geology_save::volcanoes_from_rows(&[]).is_err(), "an empty table is refused");
        // VA-5's recorded crater: kept when usable, refused when not.
        let mut rows = volcano_rows_of(&table);
        rows[0].crater = Some((0.02, 0.05));
        assert_eq!(geology_save::volcanoes_from_rows(&rows).unwrap()[0].crater, Some((0.02, 0.05)));
        rows[0].crater = Some((0.06, 0.05));
        assert!(geology_save::volcanoes_from_rows(&rows).is_err(), "a crater floor wider than its rim is refused");
        // Ruling CX item 2's years: refused when not a positive finite age.
        for bad in [0.0, -5.0, f64::NAN, f64::INFINITY] {
            let mut rows = volcano_rows_of(&table);
            rows[0].years = Some(bad);
            assert!(geology_save::volcanoes_from_rows(&rows).is_err(), "years {bad} is refused");
        }
        // Ruling DB item 1's trend: kept when an axis in 0..PI, refused
        // otherwise; a row without the key reloads with none.
        let mut rows = volcano_rows_of(&table);
        rows[0].rift_bearing = Some(1.25);
        assert_eq!(geology_save::volcanoes_from_rows(&rows).unwrap()[0].rift_bearing, Some(1.25));
        rows[0].rift_bearing = None;
        assert_eq!(geology_save::volcanoes_from_rows(&rows).unwrap()[0].rift_bearing, None);
        for bad in [-0.1, std::f64::consts::PI, 4.0, f64::NAN] {
            let mut rows = volcano_rows_of(&table);
            rows[0].rift_bearing = Some(bad);
            assert!(geology_save::volcanoes_from_rows(&rows).is_err(), "bearing {bad} is refused");
        }
    }

    /// The rows a save writes for `t` (the manifest's own serialisation, read
    /// back), for the refusal cases above.
    fn volcano_rows_of(t: &[cartalith_terrain::PlacedEdifice]) -> Vec<geology_save::VolcanoRow> {
        let col = cartalith_terrain::geology::GeologyColumn {
            rock_top: vec![0; GW * GH],
            rock_sub: vec![NO_LAYER; GW * GH],
            contact: vec![f32::NAN; GW * GH],
            regolith: vec![0.0; GW * GH],
            volcanic_setting: vec![0; GW * GH],
            construction: Construction::Off,
            painted: None,
        };
        geology_save::geology_rasters(&col, GW, GH, 1.0, Some(t)).unwrap().1.volcanoes.unwrap()
    }

    /// Protects: each `Construction` variant's encoding round-trips -- `Off`,
    /// all three `Skipped` reasons and `Built` with its surface -- so the
    /// re-erosion of a reopened world takes the branch it took before the save.
    #[test]
    fn every_construction_record_round_trips() {
        let (p, mut ws) = world(24601, false);
        let surface: Vec<f32> = ws.field.iter().map(|v| v * 0.5 + 0.125).collect();
        for c in [
            Construction::Off,
            Construction::Skipped(ConstructionSkip::NoLand),
            Construction::Skipped(ConstructionSkip::NoTwoLayerLand),
            Construction::Skipped(ConstructionSkip::WorldStructureOn),
            Construction::Built { structural_surface: surface.clone() },
        ] {
            ws.geology.column_mut().unwrap().construction = c.clone();
            let (ws2, status, _, _) = reopen(&archive(&p, &ws).0);
            assert_eq!(status, "complete", "{c:?}");
            assert_same_column(ws.geology.column().unwrap(), ws2.geology.column().unwrap());
        }
    }

    /// Protects: a world with no column writes no geology member and no
    /// geology raster (§8.4's "does not invent a column"), so re-saving a
    /// project reopened without one keeps it without one, and reopening that
    /// gives `"absent"` with no warning.
    #[test]
    fn a_world_without_a_column_writes_no_geology_entries() {
        let (p, mut ws) = world(24601, false);
        ws.geology = cartalith_engine::Geology::Absent(cartalith_engine::GeologyAbsent::Restored);
        let (bytes, wrote) = archive(&p, &ws);
        assert!(!wrote);
        let names: Vec<String> = zip::ZipArchive::new(Cursor::new(&bytes)).unwrap().file_names().map(str::to_string).collect();
        for path in cartalith_io::GEOLOGY_RASTERS {
            assert!(!names.iter().any(|n| n.as_str() == path), "{path} written for a world with no column");
        }
        let manifest: serde_json::Value = {
            let mut z = zip::ZipArchive::new(Cursor::new(&bytes)).unwrap();
            let f = z.by_name("project.json").unwrap();
            serde_json::from_reader(f).unwrap()
        };
        assert!(manifest.get("geology").is_none(), "{manifest}");
        let (ws2, status, warning, _) = reopen(&bytes);
        assert_eq!((status, warning), ("absent", None));
        assert!(matches!(ws2.geology, cartalith_engine::Geology::Absent(cartalith_engine::GeologyAbsent::Restored)));
    }

    /// Protects: backward compatibility with the two real prior-format
    /// archives checked in (`MISTAKES.md`, "Write a backward-compatibility
    /// test"): each still reads, carries no geology member and no geology
    /// raster, is read as having no column (not an error), and a re-save the
    /// way `project_save_with_documents` saves a `Loaded` world writes no
    /// geology entry and the same raster bytes it read.
    #[test]
    fn projects_saved_before_the_set_open_without_a_column_and_resave_without_one() {
        let fixtures: [(&str, &[u8]); 2] = [
            ("project_pre_substrate_2026-09-24", include_bytes!("../tests/fixtures/project_pre_substrate_2026-09-24.zip")),
            ("project_384cap_512x328_2026-09-28", include_bytes!("../tests/fixtures/project_384cap_512x328_2026-09-28.zip")),
        ];
        for (name, bytes) in fixtures {
            let mut data = project::read_project(Cursor::new(bytes)).unwrap_or_else(|e| panic!("{name}: {e}"));
            assert!(data.geology.is_null(), "{name}: premise, written before the member");
            let (gw, gh) = (data.save.params.gw, data.save.params.gh);
            let before = data.rasters.clone();
            assert!(matches!(geology_save::column_from_project(&mut data, gw, gh), Ok(None)), "{name}: no member is no column, not an error");
            assert_eq!(data.rasters, before, "{name}: the reader took nothing");
            // Re-save as a `Loaded` world: its own core fields and the rasters
            // it carried, nothing geology-shaped to add.
            let mut write = ProjectWrite::new(&data.save.params, &data.save.fields);
            for (k, v) in &data.rasters {
                write.raster(k.clone(), v.clone());
            }
            let mut buf = Vec::new();
            project::write_project(Cursor::new(&mut buf), &write).unwrap();
            let back = project::read_project(Cursor::new(&buf)).unwrap();
            assert!(back.geology.is_null(), "{name}: a re-save invents no geology member");
            for path in cartalith_io::GEOLOGY_RASTERS {
                assert!(back.raster(path).is_none(), "{name}: {path}");
            }
            assert_eq!(back.rasters, data.rasters, "{name}: every raster re-saved as read");
            assert_eq!(back.save.fields, data.save.fields, "{name}: core rasters re-saved as read");
        }
    }

    /// A painted fixture archive, read, with `edit` applied, handed to the
    /// reader; returns its refusal and asserts it took nothing.
    fn refused(edit: impl Fn(&mut cartalith_io::ProjectData)) -> String {
        let (p, mut ws) = world(24601, false);
        paint_block(&p, &mut ws);
        let mut data = project::read_project(Cursor::new(&archive(&p, &ws).0)).unwrap();
        edit(&mut data);
        let before = raster_bits(&data.rasters);
        let e = geology_save::column_from_project(&mut data, GW, GH).expect_err("must be refused");
        assert_eq!(raster_bits(&data.rasters), before, "a refusal takes nothing");
        e
    }

    /// Every raster as `(path, bit patterns)`, so a NaN compares equal to the
    /// same NaN -- `Raster`'s `PartialEq` holds `NaN != NaN`, and the contact
    /// raster is NaN under every single-layer cell.
    fn raster_bits(r: &BTreeMap<String, Raster>) -> Vec<(String, Vec<u32>)> {
        r.iter()
            .map(|(k, v)| {
                let bits = match v {
                    Raster::F32(x) => x.iter().map(|f| f.to_bits()).collect(),
                    Raster::I32(x) => x.iter().map(|&i| i as u32).collect(),
                    Raster::U8(x) => x.iter().map(|&b| u32::from(b)).collect(),
                };
                (k.clone(), bits)
            })
            .collect()
    }

    /// Sets one cell of a `u8` geology raster.
    fn set_u8(data: &mut cartalith_io::ProjectData, path: &str, i: usize, v: u8) {
        match data.rasters.get_mut(path) {
            Some(Raster::U8(r)) => r[i] = v,
            _ => panic!("premise: {path} is a u8 raster in the archive"),
        }
    }

    /// Protects: every refusal §8.4 states fires, each by its own message,
    /// and the edge of each legal range is accepted (rock index 10,
    /// `NO_LAYER`, setting code 4, a foreign number under a single-layer
    /// cell read as NaN).
    #[test]
    fn a_damaged_or_foreign_set_is_refused_and_names_why() {
        assert!(refused(|d| d.geology["version"] = serde_json::json!(2)).contains("version 2"));
        assert!(refused(|d| d.geology["grid_width"] = serde_json::json!(161)).contains("written for a 161x112 grid"));
        assert!(refused(|d| d.geology["construction"] = serde_json::json!("sideways")).contains("construction"));
        assert!(refused(|d| {
            d.rasters.remove("rasters/rock_painted.u8");
        })
        .contains("rasters/rock_painted.u8 missing"));
        assert!(refused(|d| {
            d.rasters.remove("rasters/regolith.f32");
        })
        .contains("rasters/regolith.f32 missing"));
        assert!(refused(|d| set_u8(d, "rasters/rock_top.u8", 7, 11)).contains("rock_top.u8 holds 11"));
        assert!(refused(|d| set_u8(d, "rasters/rock_sub.u8", 7, 11)).contains("rock_sub.u8 holds 11"));
        assert!(refused(|d| set_u8(d, "rasters/volcanic_setting.u8", 7, 5)).contains("volcanic_setting.u8 holds 5"));
        assert!(refused(|d| set_u8(d, "rasters/rock_painted.u8", 7, 2)).contains("rock_painted.u8 holds 2"));
        assert!(refused(|d| {
            if let Some(Raster::F32(r)) = d.rasters.get_mut("rasters/regolith.f32") {
                r[3] = -1.0e-6;
            }
        })
        .contains("regolith is a thickness"));
        // A NaN contact on a two-layer cell.
        assert!(refused(|d| {
            let Some(Raster::U8(sub)) = d.rasters.get("rasters/rock_sub.u8") else { panic!("premise") };
            let i = sub.iter().position(|&s| s != NO_LAYER).expect("premise: a two-layer cell");
            if let Some(Raster::F32(c)) = d.rasters.get_mut("rasters/rock_contact.f32") {
                c[i] = f32::NAN;
            }
        })
        .contains("not a number at cell"));
        // An older build's re-save: rasters carried, member dropped.
        assert!(refused(|d| d.geology = serde_json::Value::Null).contains("no geology member"));

        // The legal edges are read, not refused.
        let (p, mut ws) = world(24601, false);
        paint_block(&p, &mut ws);
        let mut data = project::read_project(Cursor::new(&archive(&p, &ws).0)).unwrap();
        set_u8(&mut data, "rasters/rock_top.u8", 7, 10);
        set_u8(&mut data, "rasters/rock_sub.u8", 8, NO_LAYER);
        set_u8(&mut data, "rasters/volcanic_setting.u8", 7, 4);
        if let Some(Raster::F32(c)) = data.rasters.get_mut("rasters/rock_contact.f32") {
            c[8] = 0.5;
        }
        let col = geology_save::column_from_project(&mut data, GW, GH).expect("legal edges").expect("a column");
        assert_eq!((col.rock_top[7], col.rock_sub[8], col.volcanic_setting[7]), (10, NO_LAYER, 4));
        assert!(col.contact[8].is_nan(), "a contact under a single-layer cell is read as NaN");
    }
}
