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
//! | landmark settings | `LandmarksDoc` below, from `WorldGen::landmark_store` |
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
//! written, and until 2026-08-31 **nothing anywhere could build one**. The
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
//! The return leg is not symmetric, and the asymmetry is structural rather
//! than unfinished:
//!
//! * `library/travel.json` and `library/assets.json` **restore**. Both live
//!   on `WorldGen` as plain non-`Option` fields that survive `absorb()`, so
//!   there is always something to restore into.
//! * `drafts/sculpt.json` restores **only into a live Sculpt editor**, i.e. a
//!   generated world of the same grid size. `WorldGen::sculpt` is `None`
//!   after `load_save()`/`project_open` (a loaded save carries no
//!   `river_mask`/`river_floor` for the water hooks -- that field's own doc),
//!   so a project opened from disk has no editor to hold a draft.
//! * `drafts/paint.json` **does not restore at all**, and there is no
//!   `paint_restore_document` for the same reason there is no way to write
//!   one honestly: `WorldGen::paint` is likewise `None` on a loaded project,
//!   and `PaintEditor`'s three committed layers are private to
//!   `paint_bridge.rs` with no setter. Writing the document ends the data
//!   loss the row was about; reading it back into a session needs a
//!   `PaintEditor::restore_layers` in that file plus a decision about where a
//!   loaded project's paint editor comes from at all.
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
//! It does **not** make a loaded project regenerable. Every function that
//! needs the tectonic substrate pattern-matches `WorldSource::Generated`
//! and bails on a loaded world; that was already true and is unchanged.
//! The distinction is between *recalling* the civilisation layer, which the
//! archive now carries, and *recomputing* it, which needs rasters the
//! archive deliberately does not store (`SAVEFILE_COMPAT.md` §16.2).

use crate::{
    civ_roster_bridge, icon_bridge, infra_tools_bridge, journey_bridge, label_bridge, params,
    CivData, WorldGen, WorldSource,
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

const SLOT_SETTLEMENTS: &str = "entities/settlements.json";
const SLOT_FACTIONS: &str = "entities/factions.json";
const SLOT_WAYS: &str = "entities/ways.json";
const SLOT_PROVINCES: &str = "entities/provinces.json";
const SLOT_CONTINENTS: &str = "entities/continents.json";
const SLOT_TIMELINE: &str = "history/timeline.json";
const SLOT_LABELS: &str = "annotations/labels.json";
const SLOT_ICONS: &str = "annotations/icons.json";
const SLOT_REGIONS: &str = "annotations/regions.json";
const SLOT_APPEARANCE: &str = "appearance.json";
const SLOT_VAULT: &str = "vault.json";
const SLOT_LANDMARKS: &str = "entities/landmarks.json";

// The four caller-owned slots this file builds the *contents* of without
// owning the slot itself. They stay out of [`ENGINE_OWNED_SLOTS`] below --
// see "The four documents nothing could produce" in this module's own doc.
const SLOT_PAINT: &str = "drafts/paint.json";
const SLOT_SCULPT: &str = "drafts/sculpt.json";
const SLOT_ASSETS: &str = "library/assets.json";
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
];

// ===================== the document schemas =====================
//
// `SAVEFILE_COMPAT.md` §9-§11 is the specification; these are its Rust
// expression. Every field carries `#[serde(default)]` so that a document
// written by a different implementation, or by an older version of this
// one, loses only what it actually omitted -- §14.3's unknown-member rule
// in its constructive direction.

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct SettlementsDoc {
    #[serde(default)]
    next_id: u64,
    #[serde(default)]
    settlements: Vec<SettlementDto>,
}

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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct TradeDto {
    #[serde(default)]
    exports: Vec<String>,
    #[serde(default)]
    imports: Vec<String>,
}

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
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct FactionsDoc {
    #[serde(default)]
    factions: Vec<FactionDto>,
}

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
}

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
}

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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct ProvincesDoc {
    #[serde(default)]
    provinces: Vec<ProvinceDto>,
}

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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct ContinentsDoc {
    #[serde(default)]
    continents: Vec<ContinentDto>,
}

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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct TimelineDoc {
    #[serde(default)]
    year: i64,
    #[serde(default)]
    years: Vec<TimelineYearDto>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct TimelineYearDto {
    #[serde(default)]
    year: i64,
    #[serde(default)]
    settlements: Vec<SettlementDto>,
    #[serde(default)]
    ways: Vec<RoadDto>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct LabelsDoc {
    #[serde(default)]
    labels: Vec<LabelDto>,
}

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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct IconsDoc {
    #[serde(default)]
    icons: Vec<IconDto>,
}

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
}

/// `SAVEFILE_COMPAT.md` §13.2. Every member optional, and a reader may
/// ignore the whole file and still render the world with its own defaults --
/// which is what makes this presentation and not state.
///
/// `appearance_preset` (`GUI_GAP_REGISTER.md` CA-08's loaded preset) is
/// deliberately **not** here: a preset is a complete, separately-saved
/// description of a look with its own file format, and embedding a copy in
/// every project would give the same look two homes that could then disagree.
// No `Debug`: `render::Npr` has none, and deriving one there for a struct
// nothing ever prints would be a change to the renderer for this file's
// convenience.
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
struct AppearanceDoc {
    quality: String,
    look: String,
    territory_opacity: f64,
    overrides: std::collections::HashMap<String, f64>,
    ramp: Option<crate::render::ElevationRamp>,
    npr: crate::render::Npr,
}

/// `entities/landmarks.json` -- the Landmark Generation dock's **settings**,
/// and deliberately not its results.
///
/// The split is not laziness. `LandmarkStore::last` is the output of
/// `cartalith_civ::landmark::generate`, a pure function of the world, the
/// settings and the seed (`LANDMARK_GENERATION_RESEARCH.md` §27), and the
/// loader clears it on every open for a reason that would still hold if it
/// were written: placements taken over the previous field must not be shown
/// against a new one. Re-running the pass is one click and reproduces them
/// exactly.
///
/// The settings are the opposite: hand-entered configuration -- forty-nine
/// caps, forty-nine armed flags, a crowding factor and four class radii --
/// that no amount of recomputation brings back. Before this document existed
/// they were **not saved at all** and were **never cleared**, so they leaked
/// out of whichever project was open last into the next one, belonging to
/// neither. Writing them here and resetting to `LandmarkSettings::default()`
/// in `load_save`'s own reset closes both halves at once.
///
/// Every field is `#[serde(default)]`, and `LandmarkSettings`' own
/// `cap`/`is_armed` accessors already fall back to each kind's spec default
/// for a key the map has no row for -- so a document written before a new
/// landmark kind existed loads without inventing a value for it.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
struct LandmarksDoc {
    settings: LandmarkSettingsDto,
}

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
    fn default() -> Self {
        Self::from(&cartalith_civ::landmark::LandmarkSettings::default())
    }
}

impl From<&cartalith_civ::landmark::LandmarkSettings> for LandmarkSettingsDto {
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct RegionsDoc {
    #[serde(default)]
    region: Option<RegionDto>,
}

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

fn way_class_key(t: cartalith_civ::WayType) -> &'static str {
    match t {
        cartalith_civ::WayType::Highway => "highway",
        cartalith_civ::WayType::Regional => "regional",
        cartalith_civ::WayType::Road => "road",
        cartalith_civ::WayType::Track => "track",
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
        _ => cartalith_civ::WayType::Track,
    }
}

fn manual_way_kind_key(t: cartalith_civ::tools::ManualWayType) -> &'static str {
    use cartalith_civ::tools::ManualWayType as M;
    match t {
        M::Road => "road",
        M::Track => "track",
        M::SeaLane => "sea_lane",
        M::Ancient => "ancient",
    }
}

fn manual_way_kind_from(key: &str) -> cartalith_civ::tools::ManualWayType {
    use cartalith_civ::tools::ManualWayType as M;
    match key {
        "road" => M::Road,
        "sea_lane" => M::SeaLane,
        "ancient" => M::Ancient,
        _ => M::Track,
    }
}

fn route_mode_key(m: cartalith_civ::tools::RouteMode) -> &'static str {
    use cartalith_civ::tools::RouteMode as R;
    match m {
        R::Land => "land",
        R::Water => "water",
        R::Mixed => "mixed",
    }
}

fn route_mode_from(key: &str) -> cartalith_civ::tools::RouteMode {
    use cartalith_civ::tools::RouteMode as R;
    match key {
        "water" => R::Water,
        "mixed" => R::Mixed,
        _ => R::Land,
    }
}

fn label_size_mode_key(m: cartalith_civ::labels::LabelSizeMode) -> &'static str {
    match m {
        cartalith_civ::labels::LabelSizeMode::Zoom => "zoom",
        cartalith_civ::labels::LabelSizeMode::Fixed => "fixed",
    }
}

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

fn pts_to_dto(pts: &[(f64, f64)]) -> Vec<[f64; 2]> {
    pts.iter().map(|&(x, y)| [x, y]).collect()
}

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
            }),
    }
}

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

/// Everything the archive holds about the civilisation layer, as JSON text
/// keyed by slot, plus the civ rasters. Pure — no Godot, no filesystem —
/// so the round-trip tests below run under `cargo test -p cartalith-godot`
/// with no Godot runtime involved.
fn civ_documents(civ: &CivData, out: &mut BTreeMap<String, String>) {
    let extras = &civ.place_extras.0;
    let settlements = SettlementsDoc {
        next_id: civ.next_tid,
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
            })
            .collect(),
    };
    insert_doc(out, SLOT_FACTIONS, &factions);

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
                .map(|snap| TimelineYearDto {
                    year: snap.year,
                    settlements: snap
                        .settlements
                        .iter()
                        .map(|s| settlement_to_dto(s, None, None, false))
                        .collect(),
                    ways: snap.ways.iter().map(road_to_dto).collect(),
                })
                .collect(),
        };
        insert_doc(out, SLOT_TIMELINE, &timeline);
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

/// Rebuilds a [`CivData`] from an opened archive. `None` when the archive
/// carries no settlement document at all, which is a project saved before
/// any civilisation layer existed rather than a failure.
fn civ_from_project(data: &cartalith_io::ProjectData, n: usize) -> Option<CivData> {
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
                })
                .collect(),
        )
    };

    let timeline_doc: TimelineDoc = data
        .parse(SLOT_TIMELINE)
        .and_then(|r| r.ok())
        .unwrap_or_default();
    let mut timeline: Vec<cartalith_civ::timeline::TimelineSnapshot> = timeline_doc
        .years
        .iter()
        .map(|y| cartalith_civ::timeline::TimelineSnapshot {
            year: y.year,
            // A snapshot with no stored territory raster is legal (§10.2)
            // and is an empty vec, which is what a reader of `territory`
            // must already tolerate for a year recorded before territory
            // existed.
            territory: data
                .history_territory
                .get(&y.year)
                .cloned()
                .unwrap_or_default(),
            settlements: y.settlements.iter().map(dto_to_settlement).collect(),
            ways: y.ways.iter().map(dto_to_road).collect(),
        })
        .collect();
    // §10.1: sorted ascending, unique. Enforced here rather than trusted,
    // because the year cursor walks this list by index.
    timeline.sort_by_key(|s| s.year);
    timeline.dedup_by_key(|s| s.year);

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

    let take_i32 = |path: &str| -> Vec<i32> {
        match data.raster(path) {
            Some(Raster::I32(v)) if v.len() == n => v.clone(),
            _ => vec![0; n],
        }
    };
    let territory = take_i32("rasters/territory.i32");
    let provinces = take_i32("rasters/provinces.i32");
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
        // Deliberately empty, for `explanations`' reason one field below: the
        // archive stores no channel topology (`SAVEFILE_COMPAT.md` §16.2), and
        // the raw `RoadEdge` cell paths cannot be recovered from the smoothed
        // `ways` they were consolidated into. A loaded project's Journey
        // Planner therefore reads one road source instead of two -- the same
        // road network, without the un-smoothed router paths.
        road_edges: Vec::new(),
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
        place_extras: civ_roster_bridge::PlaceExtrasTable(place_extras),
        village_tids,
    })
}

// ===================== the Godot surface =====================

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
             contents (get_settlements(), get_ways(), the faction roster, the timeline) \
             rather than round-tripping the document through the shell"
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
        self.project_save_with_documents(path, VarDictionary::new())
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
    #[func]
    fn project_save_with_documents(
        &mut self,
        path: GString,
        extra_documents: VarDictionary,
    ) -> VarDictionary {
        let Some(source) = self.source.as_ref() else {
            return err("no world to save");
        };
        let n = (self.gw.max(0) as usize) * (self.gh.max(0) as usize);

        // The same saturation the reference exporter applies (`o > 255 ?
        // 255 : o`): `stream_order` is wider in memory than in the archive.
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
        };

        let mut write = ProjectWrite::new(&params, &fields);
        write.readme = Some(project::DEFAULT_README.to_string());

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
            civ_documents(civ, &mut documents);
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
                if snap.territory.len() == n {
                    write
                        .history_territory
                        .insert(snap.year, snap.territory.clone());
                }
            }
        }

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
                &IconsDoc {
                    icons: icons
                        .icons
                        .iter()
                        .map(|i| IconDto {
                            x: i.x,
                            y: i.y,
                            family: i.family.key().to_string(),
                            slot: i.slot.clone(),
                            set: i.set.clone(),
                            scale: i.scale,
                        })
                        .collect(),
                },
            );
        }

        insert_doc(
            &mut documents,
            SLOT_APPEARANCE,
            &AppearanceDoc {
                quality: self.quality.name().to_string(),
                look: self.look.clone(),
                territory_opacity: self.territory_opacity,
                overrides: self.appearance_over.clone(),
                ramp: self.appearance_ramp.clone(),
                npr: self.npr.clone(),
            },
        );

        // The Landmark dock's settings. Unconditional, unlike the editors
        // above: `LandmarkSettings::default()` is not "nothing" -- it is
        // forty-nine caps and forty-nine armed flags derived from each
        // kind's own spec -- so "absent" and "at defaults" are the same
        // state and writing the block costs a few hundred bytes for a
        // document the user can then diff. The retained *run* is not
        // written; `LandmarksDoc`'s own doc comment says why.
        insert_doc(
            &mut documents,
            SLOT_LANDMARKS,
            &LandmarksDoc { settings: LandmarkSettingsDto::from(&self.landmark_store.settings) },
        );

        // The vault's own serialized store, verbatim (§13.3) -- through
        // `LinkStore`'s own `to_json`, so the vault keeps owning its shape
        // and this file never has to know what a knowledge link is. Skipped
        // when there is nothing filed, so an untouched project carries no
        // empty `vault.json`.
        if !self.vault.store.links.is_empty() {
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

        let mut buf: Vec<u8> = Vec::new();
        if let Err(e) = project::write_project(std::io::Cursor::new(&mut buf), &write) {
            return err(e);
        }
        let entries =
            write.documents.len() + write.rasters.len() + write.history_territory.len() + 8;
        match std::fs::write(path.to_string(), &buf) {
            Ok(()) => {
                let mut d = vdict! { "ok" => true, "error" => "" };
                d.set("bytes", buf.len() as i64);
                d.set("entries", entries as i64);
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
    ///   Saving over this file would drop them (§6.2), so a caller that
    ///   offers Save should say so first.
    /// - `restored` names the engine-owned payloads that were applied —
    ///   `civ`, `labels`, `icons`, `ways`, `region`, `appearance`, `vault`.
    ///
    /// A loaded project is **not** regenerable: every path that needs the
    /// tectonic substrate still requires a freshly generated world. See
    /// this module's own doc comment.
    #[func]
    fn project_open(&mut self, path: GString) -> VarDictionary {
        let file = match std::fs::File::open(path.to_string()) {
            Ok(f) => f,
            Err(e) => return err(format!("could not open {path}: {e}")),
        };
        let data = match project::read_project(std::io::BufReader::new(file)) {
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

        if let Some(civ) = civ_from_project(&data, n) {
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
                })
                .collect();
            self.labels = Some(bridge);
            restored.push("labels");
        }

        if let Some(Ok(doc)) = data.parse::<IconsDoc>(SLOT_ICONS) {
            let mut editor = icon_bridge::IconEditor::new();
            editor.icons = doc
                .icons
                .iter()
                .filter_map(|d| {
                    // §11.2 says keep an icon whose *slot* cannot be
                    // resolved. A `family` that is not one of the four is a
                    // different thing: there is no family to place it in,
                    // so it is dropped and reported by the caller's own
                    // count rather than guessed into `feature`.
                    let family = cartalith_assets::manual::ManualIconFamily::from_key(&d.family)?;
                    Some(cartalith_assets::manual::ManualIcon {
                        x: d.x,
                        y: d.y,
                        family,
                        slot: d.slot.clone(),
                        set: d.set.clone(),
                        scale: if d.scale > 0.0 { d.scale } else { 1.0 },
                    })
                })
                .collect();
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
            self.appearance_over = doc.overrides;
            self.appearance_ramp = doc.ramp;
            self.npr = doc.npr;
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

        let mut out = vdict! { "ok" => true, "error" => "" };
        out.set(
            "layout",
            if data.layout == cartalith_io::Layout::Tree {
                "tree"
            } else {
                "flat"
            },
        );
        out.set("format_version", data.format_version);
        let warnings: PackedStringArray = data.warnings.iter().map(GString::from).collect();
        let foreign: PackedStringArray = data.foreign_entries.iter().map(GString::from).collect();
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
    /// journeys — or any other shell-owned payload — while the session keeps
    /// the world it already has. Opening the archive to get at one small
    /// JSON document would replace the current world as a side effect, which
    /// is not a price a "load my saved journeys" command should pay.
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
    /// * The other candidate is the Journey Planner loading saved journeys
    ///   without replacing the world. It has no such command: journeys ride
    ///   `project_open`'s own `documents` dictionary, restored in
    ///   `app.gd`'s `world_loaded` handler, which is the whole round trip
    ///   `_savereopen_probe.gd` asserts.
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

fn travel_pairs_to_fields(
    pairs: Vec<(String, journey_bridge::JpValue)>,
) -> BTreeMap<String, serde_json::Value> {
    pairs.into_iter().map(|(k, v)| (k, jp_value_to_json(&v))).collect()
}

fn travel_fields_to_pairs(
    fields: &BTreeMap<String, serde_json::Value>,
) -> Vec<(String, journey_bridge::JpValue)> {
    fields
        .iter()
        .filter_map(|(k, v)| json_to_jp_value(v).map(|jv| (k.clone(), jv)))
        .collect()
}

// ---- the flattening both sculpt halves share ----

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

fn sculpt_params_to_map(p: &cartalith_terrain::sculpt::FeatureParams) -> BTreeMap<String, f64> {
    crate::sculpt_bridge::feature_param_pairs(p)
        .into_iter()
        .map(|(k, v)| (k.to_string(), v))
        .collect()
}

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
    /// Merge it into whatever the shell owns (`entities/journeys.json`) and
    /// pass the union; the two never collide, since none of these four is a
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
    /// `"drafts/paint.json"`. There is no matching restore, and this
    /// module's own doc says why: a project opened from disk has no
    /// `PaintEditor` to restore into. `project_open`'s `documents` still
    /// hands the text back, so nothing is *lost* -- it is carried, not yet
    /// re-applied.
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
                .map(|e| SculptStampDto {
                    feature: e.stamp.feature().meta().key.to_string(),
                    seed: e.stamp.seed,
                    sea_level: e.stamp.sea_level,
                    hidden: e.hidden,
                    points: e.stamp.points.iter().map(|p| [p.x, p.y]).collect(),
                    globals: sculpt_globals_to_map(&e.stamp.globals),
                    params: sculpt_params_to_map(&e.stamp.params),
                    sub_mode: sculpt_sub_mode_of(&e.stamp.params),
                })
                .collect(),
        };
        serde_json::to_string(&doc).map_or_else(|_| GString::new(), |t| GString::from(&t))
    }

    /// Restores a `drafts/sculpt.json` into the **live** Sculpt editor.
    ///
    /// Returns `{ok, error, stamps}`. Refused, rather than silently partly
    /// applied, when there is no Sculpt editor (no generated world) or when
    /// the document's grid is not this world's: a stroke point is a grid-cell
    /// coordinate, so a draft from a 512x384 world laid over a 256x192 one is
    /// not a smaller draft, it is the wrong one.
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
        let (gw, gh) = (self.gw.max(0) as usize, self.gh.max(0) as usize);
        if doc.gw != gw || doc.gh != gh {
            return err(format!(
                "this draft was captured on a {}x{} grid and this world is {gw}x{gh}; \
                 a stroke point is a grid-cell coordinate and does not carry over",
                doc.gw, doc.gh
            ));
        }
        let Some(s) = self.sculpt.as_mut() else {
            return err(
                "there is no Sculpt editor to restore into: one exists only over a freshly \
                 generated world, never over a loaded save (a save carries no river_mask/\
                 river_floor for the water hooks to adopt)",
            );
        };
        if let Some(f) = cartalith_terrain::sculpt::Feature::from_key(&doc.feature) {
            s.feature = f;
            s.params = sculpt_params_from_map(f, &doc.params, &doc.sub_mode);
        }
        s.globals = sculpt_globals_from_map(&doc.globals);
        s.seed = doc.seed;
        s.points.clear();
        s.selected = None;
        s.draft.discard();
        let mut restored = 0usize;
        for d in &doc.stamps {
            // A stamp naming a feature this build does not have is dropped,
            // not defaulted: `Mountains` where the file said something else
            // would be a stroke the user never drew.
            let Some(f) = cartalith_terrain::sculpt::Feature::from_key(&d.feature) else {
                continue;
            };
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
            };
            let index = s.draft.push(stamp);
            if d.hidden {
                s.draft.set_hidden(index, true);
            }
            restored += 1;
        }
        let mut out = vdict! { "ok" => true, "error" => "" };
        out.set("stamps", restored as i64);
        out
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

#[cfg(test)]
mod tests {
    use super::*;

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
            },
        );
        let mut village_tids = std::collections::HashSet::new();
        village_tids.insert(8u64);

        CivData {
            road_edges: Vec::new(),
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
                territory: vec![3; 12],
                settlements: Vec::new(),
                ways: Vec::new(),
            }],
            year: 120,
            dens: vec![0.5f32; 12],
            faction_roster: civ_roster_bridge::FactionRoster::seeded(6),
            place_extras: civ_roster_bridge::PlaceExtrasTable(place_extras),
            village_tids,
        }
    }

    /// Builds a real archive from a `CivData`, reads it back, and returns
    /// the reconstructed one -- the write -> read -> compare the brief
    /// requires, through the real container rather than the DTOs alone.
    fn round_trip(civ: &CivData, gw: usize, gh: usize) -> CivData {
        let n = gw * gh;
        let params = cartalith_io::SaveParams {
            gw,
            gh,
            seed: 4242,
            map_width_km: 800.0,
            sea_level: 0.42,
            world: false,
        };
        let fields = cartalith_io::SaveFields {
            heightmap: vec![0.5; n],
            temperature: vec![10.0; n],
            rainfall: vec![1.0; n],
            volcanic_field: vec![0.0; n],
            impact_field: vec![0.0; n],
            strahler_order: vec![0; n],
        };
        let mut write = ProjectWrite::new(&params, &fields);
        let mut documents = BTreeMap::new();
        civ_documents(civ, &mut documents);
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
            if snap.territory.len() == n {
                write
                    .history_territory
                    .insert(snap.year, snap.territory.clone());
            }
        }

        let mut buf = Vec::new();
        project::write_project(std::io::Cursor::new(&mut buf), &write)
            .expect("write_project should succeed");
        let data = cartalith_io::read_project(std::io::Cursor::new(&buf))
            .expect("read_project should succeed");
        assert!(data.warnings.is_empty(), "{:?}", data.warnings);
        civ_from_project(&data, n).expect("a civ layer that was written must come back")
    }



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
            territory: Vec::new(),
            settlements: vec![ghost],
            ways: Vec::new(),
        });
        civ.next_tid = 501;

        let mut documents = BTreeMap::new();
        civ_documents(&civ, &mut documents);
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
        };
        let fields = cartalith_io::SaveFields {
            heightmap: vec![0.5; 12],
            temperature: vec![10.0; 12],
            rainfall: vec![1.0; 12],
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
        let back = civ_from_project(&data, 12).expect("a civ layer that was written comes back");

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
        };
        let fields = cartalith_io::SaveFields {
            heightmap: vec![0.5; 12],
            temperature: vec![10.0; 12],
            rainfall: vec![1.0; 12],
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

    #[test]
    fn the_four_new_slots_are_the_caller_s_and_not_the_engine_s() {
        // The partition assertion above names all five callers; this one
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
        };
        let fields = cartalith_io::SaveFields {
            heightmap: vec![0.5; n],
            temperature: vec![10.0; n],
            rainfall: vec![1.0; n],
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
        assert!(civ_from_project(&data, n).is_none());
    }

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

        let doc = LandmarksDoc { settings: LandmarkSettingsDto::from(&tuned) };
        let text = serde_json::to_string(&doc).expect("serializes");
        let back: LandmarksDoc = serde_json::from_str(&text).expect("parses");
        assert_eq!(back.settings.into_settings(), tuned);
    }

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
        // The five the shell may have today, named so that a change to the
        // split has to be deliberate.
        let callers: Vec<&str> = cartalith_io::DOCUMENT_SLOTS
            .iter()
            .copied()
            .filter(|s| caller_slot_refusal(s).is_none())
            .collect();
        assert_eq!(
            callers,
            vec![
                "entities/journeys.json",
                "library/assets.json",
                "library/travel.json",
                "drafts/paint.json",
                "drafts/sculpt.json",
            ]
        );
    }

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

    #[test]
    fn an_unknown_resource_key_is_dropped_rather_than_leaked() {
        assert_eq!(
            resource_key_static(cartalith_civ::CIV_RESOURCE_KEYS[3]),
            Some(cartalith_civ::CIV_RESOURCE_KEYS[3])
        );
        assert_eq!(resource_key_static("unobtanium"), None);
    }

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
    }

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

    #[test]
    fn the_timeline_comes_back_sorted_and_unique() {
        // §10.1. The year cursor walks this list by index, so a file whose
        // years arrived out of order would scrub backwards.
        let mut civ = sample_civ();
        civ.timeline = vec![
            cartalith_civ::timeline::TimelineSnapshot {
                year: 300,
                territory: vec![1; 12],
                settlements: vec![],
                ways: vec![],
            },
            cartalith_civ::timeline::TimelineSnapshot {
                year: 100,
                territory: vec![2; 12],
                settlements: vec![],
                ways: vec![],
            },
            cartalith_civ::timeline::TimelineSnapshot {
                year: 200,
                territory: vec![3; 12],
                settlements: vec![],
                ways: vec![],
            },
        ];
        let back = round_trip(&civ, 4, 3);
        assert_eq!(
            back.timeline.iter().map(|s| s.year).collect::<Vec<_>>(),
            vec![100, 200, 300]
        );
        // ...and each year kept its own raster, not its neighbour's.
        assert_eq!(back.timeline[0].territory, vec![2; 12]);
        assert_eq!(back.timeline[2].territory, vec![1; 12]);
    }

    #[test]
    fn the_documents_are_the_shapes_the_specification_publishes() {
        // The specification is what a second implementation is written
        // from, so the member *names* are the contract. A rename here that
        // did not reach `SAVEFILE_COMPAT.md` would break that
        // implementation and nothing in this workspace.
        let civ = sample_civ();
        let mut docs = BTreeMap::new();
        civ_documents(&civ, &mut docs);

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

    #[test]
    fn every_id_the_format_writes_is_a_safe_integer() {
        // §14.1 is a constraint on the *format*, not a warning: a second
        // implementation in JavaScript cannot represent an id past 2^53.
        const MAX_SAFE: u64 = 9_007_199_254_740_991;
        let civ = sample_civ();
        let mut docs = BTreeMap::new();
        civ_documents(&civ, &mut docs);
        for text in docs.values() {
            let v: serde_json::Value = serde_json::from_str(text).unwrap();
            let mut worst: u64 = 0;
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
