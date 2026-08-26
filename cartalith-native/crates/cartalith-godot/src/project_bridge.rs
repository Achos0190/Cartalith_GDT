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
//! | vault links | `vault_state_json()`/`vault_restore_state()`, the pair `vault_bridge.rs` already publishes |
//! | anything GDScript owns | [`WorldGen::project_save_with_documents`]'s dictionary |
//!
//! That last row is the channel `SAVEFILE_COMPAT.md` §6.5 calls for and the
//! reason there is no `WorldGen` field holding shell state: a payload the
//! shell owns travels *through* a save call rather than being mirrored into
//! the engine first, so adding one needs no engine change at all.
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
    /// real for a loaded project), labels, icons, the selected region, and
    /// the vault links.
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
            if let Ok(store) = cartalith_vault::LinkStore::from_json(&text) {
                self.vault.store = store;
                restored.push("vault");
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
