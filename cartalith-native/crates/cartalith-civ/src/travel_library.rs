//! The Travel Library data model: `TRAVEL_LIBRARY_SPEC.md`'s four
//! definition types (§3: animals & mounts, vehicles, vessels, party
//! set-ups), their validation states (§4) and stock data, plus the pure
//! functions that let a Travel Library entry's fields actually influence a
//! computed journey (§6).
//!
//! **Genuinely new design, not a reference port.** `TRAVEL_LIBRARY_SPEC.md`
//! is an owner-supplied addition to the DCC shell with no equivalent
//! anywhere in `Cartalith Gen1 v2.10.html` -- there is no golden-parity
//! target for any of this module. Where a stock figure could be grounded in
//! an existing golden-tested constant (the four animals this port's own
//! [`crate::JP_ANIMAL_KEYS`]/[`crate::jp_animal_stats`] already model, and
//! the vehicle/vessel constants `jp_capacity`/`jp_ship_stats` already carry),
//! it *is* grounded there rather than invented twice -- see
//! [`stock_animals`]' and [`stock_vehicles`]' own doc comments for exactly
//! which numbers are shared and which are new.
//!
//! # Where this lives, and why
//!
//! `ARCHITECTURE.md`'s stateless-`cartalith-civ` / stateful-`cartalith-godot`
//! split applies here exactly as it does everywhere else in this port: this
//! module owns the *data shapes*, their *validation*, the *stock content*,
//! and the *pure resolver functions* that read a set of overrides -- nothing
//! here is mutable, nothing here is a store a user edits at runtime. The
//! mutable Travel Library itself (add/duplicate/edit/delete, id generation,
//! usage tracking) is `cartalith-godot`'s `travel_bridge` module, matching
//! every other "user-editable persistent state belongs in cartalith-godot"
//! subsystem in this port (`CivData`, `AssetDB`-shaped stores, etc.).
//! `cartalith-godot` already depends on `cartalith-civ`, and `jp_capacity_ex`/
//! `jp_calc_land_ex`/`jp_plan_ex` (this milestone's other half, in
//! `cartalith-civ`'s own `lib.rs`) need to be *computable* from a Travel
//! Library's data without a Godot runtime -- exactly what putting the shapes
//! here, rather than in `cartalith-godot`, buys.
//!
//! # What "wired into computation" means today, precisely
//!
//! [`animal_resolver_fns`] turns a species-keyed override map into the two
//! closures [`crate::JpAnimalResolver`] wants, and `crate::jp_plan_ex`
//! actually consumes them (see that module's own doc comment for the call
//! chain). That is real: an integration test in
//! `cartalith-godot/src/travel_bridge.rs` proves a custom, edited animal
//! entry changes a computed journey's days and km/day, not merely that the
//! data model round-trips.
//!
//! What is **not** wired, disclosed rather than approximated:
//!
//! - **Only the four built-in party-form species can override anything.**
//!   [`AnimalDef::species_key`] is `Some("donkey"|"mule"|"camel"|"horse")` for
//!   an entry that represents one of [`crate::JP_ANIMAL_KEYS`], or `None`.
//!   `JpParty` is a fixed four-species struct (`donkey`/`mule`/`camel`/
//!   `horse` counts, no generic map) -- a wholly new species like the stock
//!   Ox/Yak/Reindeer entries below has no party-form slot to occupy yet, so
//!   its own capacity/speed/terrain fields are real, validated, and
//!   inspectable, but inert for computation until `JpParty`/`JpPlan` grow a
//!   generic animal-count shape. That is a real, larger change to
//!   golden-tested types and is out of this milestone's scope.
//! - **Vehicles and vessels are data-only.** `jp_capacity`'s cart/wagon/
//!   sled/travois masses and `jp_ship_stats`' vessel table are still the
//!   fixed built-in constants; no resolver equivalent to
//!   [`animal_resolver_fns`] exists for either yet. The data model,
//!   validation and stock content are complete and real; the computation
//!   hook is the named follow-up alongside the four-species limit above.
//! - **`cartalith-godot/src/lib.rs`'s `jp_compute` does not yet read a live
//!   Travel Library.** No `#[func]` boundary exists this milestone by
//!   design (`TRAVEL_LIBRARY_SPEC.md`'s own GUI is a separate, later
//!   dispatch) -- see `travel_bridge`'s own module doc for the exact shape a
//!   `#[func]` layer would need to add.
//! - **"Saved journeys" do not exist as a referenceable, persistent thing in
//!   this port.** `route_get`/`infra.routes` are drawn polylines with no
//!   attached party plan; `jp_compute` computes and returns a plan without
//!   storing it anywhere. §4's "how many saved journeys ... reference it"
//!   usage count is therefore always `0`, honestly, rather than invented --
//!   see `travel_bridge::TravelLibrary::animal_usage_in_journeys`.

use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Shared vocabulary
// ---------------------------------------------------------------------------

/// Stock entries are read-only; duplicating one for editing is the only way
/// to get a custom entry (`TRAVEL_LIBRARY_SPEC.md` §3, and the same rule
/// `cartalith-assets`' Asset Library already established for its own
/// frozen-vs-custom slots).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryOrigin {
    Stock,
    Custom,
}

/// A single terrain-constraint cell: an explicit speed multiplier, or
/// outright impassable (`TRAVEL_LIBRARY_SPEC.md` §3.1/§3.2's own "per-terrain
/// multiplier or `blocked`" vocabulary, shared by animals' full ten-row table
/// and a vehicle's narrower off-road/ford pair).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TerrainAffinity {
    Multiplier(f64),
    Blocked,
}

/// `TRAVEL_LIBRARY_SPEC.md` §3.1's fixed ten-row terrain-constraint table.
/// This is the Travel Library's own vocabulary, coarser than and not
/// identical to the engine's real per-stage terrain strings
/// (`crate::CART_TERRAINS`, e.g. "Rocky Terrain"/"Mountain Trails"/"Desert
/// Hardpack") -- [`tl_terrain_key_for_engine`] is the documented, one-way
/// mapping between the two, built for [`animal_resolver_fns`].
pub const TL_TERRAIN_KEYS: [&str; 10] = [
    "Plains",
    "Steppe",
    "Forest",
    "Hills",
    "Mountain",
    "Marsh",
    "Desert",
    "High Pass",
    "Snowfield",
    "River Ford",
];

/// Which [`TL_TERRAIN_KEYS`] row governs a given *engine* terrain string
/// (`crate::JpStage::terrain`, one of `crate::CART_TERRAINS`'s surface
/// keys) -- the one real seam between the Travel Library's coarser
/// ten-category vocabulary and the golden-tested terrain strings
/// `jp_calc_land` actually reads.
///
/// `None` for three real, disclosed gaps, not an oversight:
/// - `"Paved Road"`/`"Dirt Track"` are deliberately excluded, matching
///   [`crate::jp_animal_terrain_mod`]'s own built-in convention: no species
///   entry in the reference's `JP_ANIMAL_TERRAIN_OVERRIDE` overrides either
///   road surface for any animal, so a Travel Library entry does not either
///   -- a maintained route normalises travel for every species alike.
/// - `"Ruins / Debris"` has no row in the Travel Library's ten-category
///   table at all.
/// - `"Steppe"` and `"River Ford"` are two of the ten *rows* with no engine
///   terrain that ever maps back to them: the engine has no distinct
///   grassland surface (`"Open Plains"` is what `JP_TERRAIN.land` uses
///   regardless of the underlying grass/non-grass biome, and biome-level
///   aridity is a separate axis `jp_capacity`'s desert multiplier already
///   covers), and a river ford is a crossing-count hazard
///   (`JpDerivedStage::rx`) rather than a `terrain` string a stage takes.
///   Both fields are still stored and validated (`TRAVEL_LIBRARY_SPEC.md`
///   §3.1 names them explicitly) -- only their *consumption* by
///   [`animal_resolver_fns`] is inert until a live engine hook exists for
///   either.
pub fn tl_terrain_key_for_engine(engine_terrain: &str) -> Option<&'static str> {
    match engine_terrain {
        "Open Plains" => Some("Plains"),
        "Forest Path" => Some("Forest"),
        "Hills" => Some("Hills"),
        "Rocky Terrain" | "Mountain Trails" => Some("Mountain"),
        "Mountain Pass" => Some("High Pass"),
        "Swamp / Marsh" => Some("Marsh"),
        "Desert Hardpack" | "Deep Sand" => Some("Desert"),
        "Snow / Ice" => Some("Snowfield"),
        _ => None,
    }
}

/// The two grassland-like rows [`ValidationState`]'s grazing-vs-terrain
/// conflict check treats as "grassland" -- `"Plains"` and `"Steppe"`, the
/// same two rows [`tl_terrain_key_for_engine`] both fold onto the engine's
/// single `"Open Plains"` surface.
const GRASSLAND_TERRAIN_KEYS: [&str; 2] = ["Plains", "Steppe"];

/// §4's three validation states, own per entry. `Incomplete` always takes
/// priority over `Conflicting`: a conflict check over data that is not even
/// fully present would itself be a guess, so completeness is checked first,
/// exactly as `TRAVEL_LIBRARY_SPEC.md` §4 presents the two as ordered
/// severities ("ok, incomplete ... conflicting").
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationState {
    Ok,
    /// Every unset constraint field, by name.
    Incomplete(Vec<&'static str>),
    /// Every detected conflict, as a human-readable sentence -- shown
    /// verbatim in the list/inspector per §4.
    Conflicting(Vec<String>),
}

impl ValidationState {
    pub fn is_ok(&self) -> bool {
        matches!(self, ValidationState::Ok)
    }
}

// ---------------------------------------------------------------------------
// 3.1 Animals & mounts
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnimalRole {
    Pack,
    Mount,
    Draft,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Availability {
    Global,
    /// A named region -- this port has no region-name registry to validate
    /// against yet (`cartalith-civ::labels` is unwired per its own doc
    /// comment), so the name is free text.
    Regional(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrazingTolerance {
    Unrestricted,
    /// TRAVEL_LIBRARY_SPEC.md §4's own worked conflict example.
    GrasslandOnly,
    None,
}

/// TRAVEL_LIBRARY_SPEC.md §3.1's full field list. Every field the spec marks
/// as a "constraint field" (§5: the ones blocked-stage/advisory logic reads)
/// is `Option`-wrapped -- unset is *incomplete*, not a zero, matching §3's
/// own rule ("A field left unset is incomplete, not zero").
#[derive(Debug, Clone, PartialEq)]
pub struct AnimalDef {
    pub id: String,
    pub origin: EntryOrigin,

    // ---- Classification ----
    pub name: String,
    pub roles: Vec<AnimalRole>,
    /// Another [`AnimalDef`]'s id this one is a substitute for -- the
    /// planner's own "better animal available" advisory reads this
    /// direction (§5).
    pub substitutes_for: Option<String>,
    pub size_class: Option<String>,
    pub availability: Option<Availability>,

    // ---- Capacity & speed ----
    pub load_capacity_kg: Option<f64>,
    pub draft_pull_kg: Option<f64>,
    pub base_speed_kmh: Option<f64>,
    pub sustainable_hours_day: Option<f64>,
    /// Multiple of `base_speed_kmh` a forced pace can reach.
    pub forced_pace_cap: Option<f64>,

    // ---- Sustenance ----
    pub fodder_need_kg_day: Option<f64>,
    pub water_need_l_day: Option<f64>,
    pub grazing_tolerance: Option<GrazingTolerance>,
    pub waterless_limit_days: Option<f64>,

    // ---- Terrain constraints ----
    /// Keyed by [`TL_TERRAIN_KEYS`]. A complete entry carries all ten;
    /// [`validate_animal`] reports any missing row as incomplete.
    pub terrain: HashMap<&'static str, TerrainAffinity>,

    // ---- Requirements & prohibitions ----
    pub yokeable_to_wheeled: bool,
    pub requires_road_to_tow: bool,
    pub blocked_by_seasonal_closures: bool,
    pub carryable_aboard_vessel: bool,
    pub usable_as_mount: bool,
    pub handlers_required_per_n_head: Option<f64>,

    // ---- Cost ----
    pub upkeep_sp_day_head: Option<f64>,

    /// Which of [`crate::JP_ANIMAL_KEYS`] this entry represents for
    /// computation, if any -- see the module docs' "What's not yet wired".
    /// `Some` on the four stock entries that mirror the engine's own
    /// species and on any custom entry duplicated from one of them; `None`
    /// on a wholly new species (the stock Ox/Yak/Reindeer) or a blank
    /// custom entry, both of which are real, validated data with no live
    /// engine hook yet.
    pub species_key: Option<&'static str>,
}

impl AnimalDef {
    /// A blank custom entry -- `TRAVEL_LIBRARY_SPEC.md`'s "New blank
    /// definition…" menu item. Every constraint field starts unset (reports
    /// `Incomplete`), matching §3's "unset, not zero" rule.
    pub fn blank(id: impl Into<String>, name: impl Into<String>) -> Self {
        AnimalDef {
            id: id.into(),
            origin: EntryOrigin::Custom,
            name: name.into(),
            roles: Vec::new(),
            substitutes_for: None,
            size_class: None,
            availability: None,
            load_capacity_kg: None,
            draft_pull_kg: None,
            base_speed_kmh: None,
            sustainable_hours_day: None,
            forced_pace_cap: None,
            fodder_need_kg_day: None,
            water_need_l_day: None,
            grazing_tolerance: None,
            waterless_limit_days: None,
            terrain: HashMap::new(),
            yokeable_to_wheeled: false,
            requires_road_to_tow: false,
            blocked_by_seasonal_closures: false,
            carryable_aboard_vessel: false,
            usable_as_mount: false,
            handlers_required_per_n_head: None,
            upkeep_sp_day_head: None,
            species_key: None,
        }
    }
}

/// §4: ok / incomplete / conflicting, for one [`AnimalDef`].
///
/// Two conflict rules, both mechanically derivable from the field list
/// alone (no route or plan is needed to check either, matching §1's "an
/// information layer, nothing more"):
/// 1. §4's own worked example: `grazing_tolerance` is
///    [`GrazingTolerance::GrasslandOnly`] while a non-grassland row still
///    carries a real, non-blocked, non-zero multiplier -- the entry claims
///    it cannot graze there yet is rated to travel it efficiently anyway.
/// 2. The `role`/`usable_as_mount` fields directly contradict each other:
///    `roles` names [`AnimalRole::Mount`] but `usable_as_mount` is `false`,
///    or the reverse. Two independently-editable fields that must agree
///    given the same shape as the terrain check.
pub fn validate_animal(a: &AnimalDef) -> ValidationState {
    let mut missing: Vec<&'static str> = Vec::new();
    if a.load_capacity_kg.is_none() {
        missing.push("load capacity kg");
    }
    if a.base_speed_kmh.is_none() {
        missing.push("base speed km/h");
    }
    if a.fodder_need_kg_day.is_none() {
        missing.push("fodder need kg/day");
    }
    if a.water_need_l_day.is_none() {
        missing.push("water need L/day");
    }
    if a.grazing_tolerance.is_none() {
        missing.push("grazing tolerance");
    }
    if a.waterless_limit_days.is_none() {
        missing.push("waterless limit days");
    }
    if a.availability.is_none() {
        missing.push("availability");
    }
    for k in TL_TERRAIN_KEYS {
        if !a.terrain.contains_key(k) {
            missing.push("terrain constraints");
            break;
        }
    }
    if !missing.is_empty() {
        return ValidationState::Incomplete(missing);
    }

    let mut conflicts = Vec::new();
    if a.grazing_tolerance == Some(GrazingTolerance::GrasslandOnly) {
        for k in TL_TERRAIN_KEYS {
            if GRASSLAND_TERRAIN_KEYS.contains(&k) {
                continue;
            }
            if let Some(TerrainAffinity::Multiplier(m)) = a.terrain.get(k)
                && *m > 0.0
            {
                conflicts.push(format!(
                    "grazing tolerance is grassland-only, but {k} still carries a {m:.2}x multiplier -- block {k} or loosen the grazing tolerance."
                ));
            }
        }
    }
    let claims_mount = a.roles.contains(&AnimalRole::Mount);
    if claims_mount != a.usable_as_mount {
        conflicts.push(if claims_mount {
            "role includes Mount, but \"usable as a mount\" is unset.".to_string()
        } else {
            "\"usable as a mount\" is set, but role does not include Mount.".to_string()
        });
    }
    if conflicts.is_empty() {
        ValidationState::Ok
    } else {
        ValidationState::Conflicting(conflicts)
    }
}

// ---------------------------------------------------------------------------
// 3.2 Vehicles
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VehicleClass {
    Wheeled,
    Dragged,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoadRequirement {
    None,
    Track,
    Road,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DraftRequirement {
    pub count: u32,
    /// Free-text role description (e.g. "oxen", "heavy horse") -- this port
    /// has no separate closed vocabulary for draft roles.
    pub role: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VehicleDef {
    pub id: String,
    pub origin: EntryOrigin,
    pub name: String,
    pub class: Option<VehicleClass>,
    pub load_kg: Option<f64>,
    pub draft_head_required: Option<DraftRequirement>,
    /// Multiplier applied to the draft/pace-setting speed.
    pub speed_mult: Option<f64>,
    pub road_requirement: Option<RoadRequirement>,
    pub off_road: Option<TerrainAffinity>,
    pub ford: Option<TerrainAffinity>,
    pub carryable_aboard_vessel: bool,
}

impl VehicleDef {
    pub fn blank(id: impl Into<String>, name: impl Into<String>) -> Self {
        VehicleDef {
            id: id.into(),
            origin: EntryOrigin::Custom,
            name: name.into(),
            class: None,
            load_kg: None,
            draft_head_required: None,
            speed_mult: None,
            road_requirement: None,
            off_road: None,
            ford: None,
            carryable_aboard_vessel: false,
        }
    }
}

/// §4 for a [`VehicleDef`]. The one conflict rule this field list makes
/// mechanically checkable: declaring no road is needed at all
/// (`road_requirement == RoadRequirement::None`) while off-road travel is
/// itself `blocked` -- a vehicle that needs no road yet cannot move without
/// one could never depart anywhere.
pub fn validate_vehicle(v: &VehicleDef) -> ValidationState {
    let mut missing: Vec<&'static str> = Vec::new();
    if v.class.is_none() {
        missing.push("class");
    }
    if v.load_kg.is_none() {
        missing.push("load kg");
    }
    if v.draft_head_required.is_none() {
        missing.push("draft head required");
    }
    if v.speed_mult.is_none() {
        missing.push("speed multiplier");
    }
    if v.road_requirement.is_none() {
        missing.push("road requirement");
    }
    if v.off_road.is_none() {
        missing.push("off-road multiplier");
    }
    if v.ford.is_none() {
        missing.push("ford multiplier");
    }
    if !missing.is_empty() {
        return ValidationState::Incomplete(missing);
    }
    let mut conflicts = Vec::new();
    if v.road_requirement == Some(RoadRequirement::None)
        && v.off_road == Some(TerrainAffinity::Blocked)
    {
        conflicts.push("declares no road is required, but off-road travel is blocked -- it could never depart.".to_string());
    }
    if conflicts.is_empty() {
        ValidationState::Ok
    } else {
        ValidationState::Conflicting(conflicts)
    }
}

// ---------------------------------------------------------------------------
// 3.3 Vessels
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VesselMode {
    River,
    Sea,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum WaterRating {
    Sheltered,
    Coastal,
    Open,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SailingWindow {
    Daylight,
    Continuous,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VesselDef {
    pub id: String,
    pub origin: EntryOrigin,
    pub name: String,
    pub modes: Vec<VesselMode>,
    pub hold_kg: Option<f64>,
    pub crew_required: Option<u32>,
    pub base_speed_kmh: Option<f64>,
    pub water_rating: Option<WaterRating>,
    pub sailing_window: Option<SailingWindow>,
    pub portage_capable: bool,
}

impl VesselDef {
    pub fn blank(id: impl Into<String>, name: impl Into<String>) -> Self {
        VesselDef {
            id: id.into(),
            origin: EntryOrigin::Custom,
            name: name.into(),
            modes: Vec::new(),
            hold_kg: None,
            crew_required: None,
            base_speed_kmh: None,
            water_rating: None,
            sailing_window: None,
            portage_capable: false,
        }
    }
}

/// §4 for a [`VesselDef`]. The one conflict rule this field list makes
/// mechanically checkable: a vessel rated only [`WaterRating::Sheltered`]
/// but whose `modes` claims [`VesselMode::Sea`] -- a sheltered-water rating
/// cannot cover open sea travel by definition.
pub fn validate_vessel(v: &VesselDef) -> ValidationState {
    let mut missing: Vec<&'static str> = Vec::new();
    if v.modes.is_empty() {
        missing.push("mode");
    }
    if v.hold_kg.is_none() {
        missing.push("hold kg");
    }
    if v.crew_required.is_none() {
        missing.push("crew required");
    }
    if v.base_speed_kmh.is_none() {
        missing.push("base speed");
    }
    if v.water_rating.is_none() {
        missing.push("water rating");
    }
    if v.sailing_window.is_none() {
        missing.push("sailing window");
    }
    if !missing.is_empty() {
        return ValidationState::Incomplete(missing);
    }
    let mut conflicts = Vec::new();
    if v.water_rating == Some(WaterRating::Sheltered) && v.modes.contains(&VesselMode::Sea) {
        conflicts.push("mode includes Sea, but the water rating is Sheltered -- sheltered vessels cannot be rated for open sea.".to_string());
    }
    if conflicts.is_empty() {
        ValidationState::Ok
    } else {
        ValidationState::Conflicting(conflicts)
    }
}

// ---------------------------------------------------------------------------
// 3.4 Party set-ups
// ---------------------------------------------------------------------------

/// One row = one preset of party-form values only (`TRAVEL_LIBRARY_SPEC.md`
/// §3.4) -- no route, and the fields are exactly the party-form subset of
/// [`crate::JpPlan`]/[`crate::JpParty`] (everything §3.4 lists: transport,
/// group size, cargo kg, pace, hours/day, supplies carried, animal counts by
/// species, vehicle counts by type, grazing, foraging, season). Reusing
/// `JpPlan`'s own field shape here, rather than inventing a parallel one, is
/// what makes [`PartyPreset::to_jp_plan`]/[`PartyPreset::from_jp_plan`]
/// (the spec's own "Capture party from planner" action) a straight
/// field-for-field copy instead of a translation layer that could drift.
#[derive(Debug, Clone, PartialEq)]
pub struct PartyPreset {
    pub id: String,
    pub origin: EntryOrigin,
    pub name: String,
    pub transport: String,
    pub mount_animal: Option<String>,
    pub vessel: String,
    pub hours: f64,
    pub pace: String,
    pub season: String,
    pub supply_days: i64,
    pub carry_food: bool,
    pub grazing: String,
    pub foraging: String,
    pub party: crate::JpParty,
}

impl PartyPreset {
    pub fn blank(id: impl Into<String>, name: impl Into<String>) -> Self {
        let d = crate::JpPlan::default();
        PartyPreset {
            id: id.into(),
            origin: EntryOrigin::Custom,
            name: name.into(),
            transport: d.transport,
            mount_animal: d.mount_animal,
            vessel: d.vessel,
            hours: d.hours,
            pace: d.pace,
            season: d.season,
            supply_days: d.supply_days,
            carry_food: d.carry_food,
            grazing: d.grazing,
            foraging: d.foraging,
            party: d.party,
        }
    }

    /// "Capture party from planner" (§2's submenu item): a new preset from
    /// the planner's current form.
    pub fn from_jp_plan(
        id: impl Into<String>,
        name: impl Into<String>,
        plan: &crate::JpPlan,
    ) -> Self {
        PartyPreset {
            id: id.into(),
            origin: EntryOrigin::Custom,
            name: name.into(),
            transport: plan.transport.clone(),
            mount_animal: plan.mount_animal.clone(),
            vessel: plan.vessel.clone(),
            hours: plan.hours,
            pace: plan.pace.clone(),
            season: plan.season.clone(),
            supply_days: plan.supply_days,
            carry_food: plan.carry_food,
            grazing: plan.grazing.clone(),
            foraging: plan.foraging.clone(),
            party: plan.party,
        }
    }

    /// Apply this preset onto a [`crate::JpPlan`], leaving every route-only
    /// field (`desert_water`, `route_cond`, `stage_overrides`, ...) at
    /// whatever `base` already carries -- §3.4's own "applying a set-up
    /// leaves per-stage overrides untouched".
    pub fn apply_to(&self, base: &crate::JpPlan) -> crate::JpPlan {
        crate::JpPlan {
            transport: self.transport.clone(),
            mount_animal: self.mount_animal.clone(),
            vessel: self.vessel.clone(),
            hours: self.hours,
            pace: self.pace.clone(),
            season: self.season.clone(),
            supply_days: self.supply_days,
            carry_food: self.carry_food,
            grazing: self.grazing.clone(),
            foraging: self.foraging.clone(),
            party: self.party,
            ..base.clone()
        }
    }
}

/// §4-shaped, but party set-ups carry no terrain/grazing constraint fields
/// to conflict-check against each other -- only completeness is meaningful,
/// so [`ValidationState::Conflicting`] never appears here. A blank
/// `transport`/`vessel` or a non-positive `group_size` is the only thing
/// worth flagging: everything else is a plain number a spinner can default.
pub fn validate_party_preset(p: &PartyPreset) -> ValidationState {
    let mut missing: Vec<&'static str> = Vec::new();
    if p.transport.trim().is_empty() {
        missing.push("transport");
    }
    if p.party.group_size <= 0 {
        missing.push("group size");
    }
    if missing.is_empty() {
        ValidationState::Ok
    } else {
        ValidationState::Incomplete(missing)
    }
}

// ---------------------------------------------------------------------------
// Stock data
// ---------------------------------------------------------------------------

fn full_terrain(
    rows: &[(&'static str, TerrainAffinity)],
) -> HashMap<&'static str, TerrainAffinity> {
    let mut m: HashMap<&'static str, TerrainAffinity> = TL_TERRAIN_KEYS
        .iter()
        .map(|&k| (k, TerrainAffinity::Multiplier(1.0)))
        .collect();
    for &(k, v) in rows {
        m.insert(k, v);
    }
    m
}

/// TRAVEL_LIBRARY_SPEC.md §3.1 names Donkey/Mule/Camel/Horse/Ox/Yak/Reindeer
/// as its own mockup examples. The first four mirror
/// [`crate::JP_ANIMAL_KEYS`]' already golden-tested `cap_kg`/`food_kg_day`/
/// `water_l_day`/`mounted_speed_kmh` figures exactly ([`AnimalDef::species_key`]
/// is `Some` for all four, so a stock, un-duplicated entry changes nothing
/// about a computed plan versus today's built-in table -- see
/// [`animal_resolver_fns`]). Their terrain rows mirror
/// [`crate::jp_animal_terrain_mod`]'s own per-species overrides, mapped
/// through [`tl_terrain_key_for_engine`] in reverse.
///
/// Ox/Yak/Reindeer are genuinely new content (`species_key: None` -- no
/// engine hook exists for them yet): plausible, internally-consistent
/// draft-animal figures grounded in common domain knowledge (ox: slow, huge
/// pull, cheap keep; yak: mid pack capacity with strong high-altitude
/// affinity; reindeer: light but fast on snow, minimal fodder need from
/// lichen grazing), not academic citations -- `TRAVEL_LIBRARY_SPEC.md`'s own
/// framing for this milestone ("new content... ground it in something
/// real").
pub fn stock_animals() -> Vec<AnimalDef> {
    vec![
        AnimalDef {
            id: "donkey".into(),
            origin: EntryOrigin::Stock,
            name: "Donkey".into(),
            roles: vec![AnimalRole::Pack, AnimalRole::Mount],
            substitutes_for: None,
            size_class: Some("Small".into()),
            availability: Some(Availability::Global),
            load_capacity_kg: Some(80.0),
            draft_pull_kg: Some(120.0),
            base_speed_kmh: Some(4.0),
            sustainable_hours_day: Some(8.0),
            forced_pace_cap: Some(1.2),
            fodder_need_kg_day: Some(4.0),
            water_need_l_day: Some(15.0),
            grazing_tolerance: Some(GrazingTolerance::Unrestricted),
            waterless_limit_days: Some(2.0),
            terrain: full_terrain(&[
                ("Hills", TerrainAffinity::Multiplier(0.80)),
                ("Mountain", TerrainAffinity::Multiplier(0.70)),
                ("High Pass", TerrainAffinity::Multiplier(0.75)),
                ("Forest", TerrainAffinity::Multiplier(0.85)),
                ("Marsh", TerrainAffinity::Multiplier(0.40)),
                ("Desert", TerrainAffinity::Multiplier(0.60)),
                ("Snowfield", TerrainAffinity::Multiplier(0.55)),
            ]),
            yokeable_to_wheeled: true,
            requires_road_to_tow: false,
            blocked_by_seasonal_closures: true,
            carryable_aboard_vessel: true,
            usable_as_mount: true,
            handlers_required_per_n_head: Some(6.0),
            upkeep_sp_day_head: Some(2.0),
            species_key: Some("donkey"),
        },
        AnimalDef {
            id: "mule".into(),
            origin: EntryOrigin::Stock,
            name: "Mule".into(),
            roles: vec![AnimalRole::Pack, AnimalRole::Mount, AnimalRole::Draft],
            substitutes_for: None,
            size_class: Some("Medium".into()),
            availability: Some(Availability::Global),
            load_capacity_kg: Some(110.0),
            draft_pull_kg: Some(250.0),
            base_speed_kmh: Some(5.0),
            sustainable_hours_day: Some(8.0),
            forced_pace_cap: Some(1.25),
            fodder_need_kg_day: Some(5.0),
            water_need_l_day: Some(20.0),
            grazing_tolerance: Some(GrazingTolerance::Unrestricted),
            waterless_limit_days: Some(2.0),
            terrain: full_terrain(&[
                ("Hills", TerrainAffinity::Multiplier(0.85)),
                ("Mountain", TerrainAffinity::Multiplier(0.75)),
                ("High Pass", TerrainAffinity::Multiplier(0.85)),
                ("Forest", TerrainAffinity::Multiplier(1.0)),
                ("Marsh", TerrainAffinity::Multiplier(0.45)),
                ("Desert", TerrainAffinity::Multiplier(0.45)),
                ("Snowfield", TerrainAffinity::Multiplier(0.60)),
            ]),
            yokeable_to_wheeled: true,
            requires_road_to_tow: false,
            blocked_by_seasonal_closures: false,
            carryable_aboard_vessel: true,
            usable_as_mount: true,
            handlers_required_per_n_head: Some(8.0),
            upkeep_sp_day_head: Some(3.0),
            species_key: Some("mule"),
        },
        AnimalDef {
            id: "camel".into(),
            origin: EntryOrigin::Stock,
            name: "Camel".into(),
            roles: vec![AnimalRole::Pack, AnimalRole::Mount, AnimalRole::Draft],
            substitutes_for: None,
            size_class: Some("Large".into()),
            availability: Some(Availability::Global),
            load_capacity_kg: Some(300.0),
            draft_pull_kg: Some(400.0),
            base_speed_kmh: Some(4.5),
            sustainable_hours_day: Some(9.0),
            forced_pace_cap: Some(1.3),
            fodder_need_kg_day: Some(6.0),
            water_need_l_day: Some(30.0),
            grazing_tolerance: Some(GrazingTolerance::Unrestricted),
            waterless_limit_days: Some(10.0),
            terrain: full_terrain(&[
                ("Desert", TerrainAffinity::Multiplier(0.85)),
                ("Marsh", TerrainAffinity::Multiplier(0.20)),
                ("Mountain", TerrainAffinity::Multiplier(0.30)),
                ("High Pass", TerrainAffinity::Multiplier(0.50)),
                ("Snowfield", TerrainAffinity::Multiplier(0.30)),
            ]),
            yokeable_to_wheeled: true,
            requires_road_to_tow: false,
            blocked_by_seasonal_closures: false,
            carryable_aboard_vessel: false,
            usable_as_mount: true,
            handlers_required_per_n_head: Some(10.0),
            upkeep_sp_day_head: Some(4.0),
            species_key: Some("camel"),
        },
        AnimalDef {
            id: "horse".into(),
            origin: EntryOrigin::Stock,
            name: "Horse".into(),
            roles: vec![AnimalRole::Mount, AnimalRole::Draft],
            substitutes_for: None,
            size_class: Some("Medium".into()),
            availability: Some(Availability::Global),
            load_capacity_kg: Some(120.0),
            draft_pull_kg: Some(350.0),
            base_speed_kmh: Some(6.0),
            sustainable_hours_day: Some(8.0),
            forced_pace_cap: Some(1.4),
            fodder_need_kg_day: Some(7.0),
            water_need_l_day: Some(25.0),
            // Not `GrasslandOnly`: real horses forage on a wide range of
            // vegetation, not strictly grassland -- `GrasslandOnly` is
            // reserved as the deliberately-narrow choice a custom entry
            // (or a test fixture, see `travel_bridge`'s conflict test)
            // would pick, which is exactly what makes §4's conflict check
            // meaningful rather than something every stock herbivore trips.
            grazing_tolerance: Some(GrazingTolerance::Unrestricted),
            waterless_limit_days: Some(1.5),
            terrain: full_terrain(&[
                ("Marsh", TerrainAffinity::Multiplier(0.35)),
                ("Desert", TerrainAffinity::Multiplier(0.55)),
            ]),
            yokeable_to_wheeled: true,
            requires_road_to_tow: false,
            blocked_by_seasonal_closures: true,
            carryable_aboard_vessel: true,
            usable_as_mount: true,
            handlers_required_per_n_head: Some(4.0),
            upkeep_sp_day_head: Some(5.0),
            species_key: Some("horse"),
        },
        AnimalDef {
            id: "ox".into(),
            origin: EntryOrigin::Stock,
            name: "Ox".into(),
            roles: vec![AnimalRole::Draft],
            substitutes_for: None,
            size_class: Some("Large".into()),
            availability: Some(Availability::Global),
            load_capacity_kg: Some(150.0),
            draft_pull_kg: Some(700.0),
            base_speed_kmh: Some(3.0),
            sustainable_hours_day: Some(6.0),
            forced_pace_cap: Some(1.1),
            fodder_need_kg_day: Some(8.0),
            water_need_l_day: Some(35.0),
            grazing_tolerance: Some(GrazingTolerance::Unrestricted),
            waterless_limit_days: Some(2.0),
            terrain: full_terrain(&[
                ("Hills", TerrainAffinity::Multiplier(0.75)),
                ("Mountain", TerrainAffinity::Multiplier(0.55)),
                ("High Pass", TerrainAffinity::Multiplier(0.45)),
                ("Marsh", TerrainAffinity::Multiplier(0.30)),
                ("Desert", TerrainAffinity::Multiplier(0.35)),
                ("Snowfield", TerrainAffinity::Multiplier(0.40)),
            ]),
            yokeable_to_wheeled: true,
            requires_road_to_tow: false,
            blocked_by_seasonal_closures: false,
            carryable_aboard_vessel: false,
            usable_as_mount: false,
            handlers_required_per_n_head: Some(2.0),
            upkeep_sp_day_head: Some(3.0),
            species_key: None,
        },
        AnimalDef {
            id: "yak".into(),
            origin: EntryOrigin::Stock,
            name: "Yak".into(),
            roles: vec![AnimalRole::Pack, AnimalRole::Draft, AnimalRole::Mount],
            substitutes_for: None,
            size_class: Some("Large".into()),
            availability: Some(Availability::Regional("Highland".into())),
            load_capacity_kg: Some(100.0),
            draft_pull_kg: Some(300.0),
            base_speed_kmh: Some(3.5),
            sustainable_hours_day: Some(7.0),
            forced_pace_cap: Some(1.15),
            fodder_need_kg_day: Some(5.0),
            water_need_l_day: Some(20.0),
            grazing_tolerance: Some(GrazingTolerance::Unrestricted),
            waterless_limit_days: Some(3.0),
            terrain: full_terrain(&[
                ("Mountain", TerrainAffinity::Multiplier(1.10)),
                ("High Pass", TerrainAffinity::Multiplier(1.20)),
                ("Snowfield", TerrainAffinity::Multiplier(1.15)),
                ("Hills", TerrainAffinity::Multiplier(1.05)),
                ("Desert", TerrainAffinity::Blocked),
                ("Marsh", TerrainAffinity::Multiplier(0.40)),
            ]),
            yokeable_to_wheeled: true,
            requires_road_to_tow: false,
            blocked_by_seasonal_closures: false,
            carryable_aboard_vessel: false,
            usable_as_mount: true,
            handlers_required_per_n_head: Some(5.0),
            upkeep_sp_day_head: Some(4.0),
            species_key: None,
        },
        AnimalDef {
            id: "reindeer".into(),
            origin: EntryOrigin::Stock,
            name: "Reindeer".into(),
            roles: vec![AnimalRole::Draft, AnimalRole::Pack],
            substitutes_for: None,
            size_class: Some("Medium".into()),
            availability: Some(Availability::Regional("Tundra".into())),
            load_capacity_kg: Some(60.0),
            draft_pull_kg: Some(150.0),
            base_speed_kmh: Some(7.0),
            sustainable_hours_day: Some(8.0),
            forced_pace_cap: Some(1.3),
            fodder_need_kg_day: Some(4.0),
            water_need_l_day: Some(10.0),
            grazing_tolerance: Some(GrazingTolerance::Unrestricted),
            waterless_limit_days: Some(4.0),
            terrain: full_terrain(&[
                ("Snowfield", TerrainAffinity::Multiplier(1.50)),
                ("Mountain", TerrainAffinity::Multiplier(0.90)),
                ("High Pass", TerrainAffinity::Multiplier(0.85)),
                ("Marsh", TerrainAffinity::Multiplier(0.50)),
                ("Desert", TerrainAffinity::Blocked),
            ]),
            yokeable_to_wheeled: false,
            requires_road_to_tow: false,
            blocked_by_seasonal_closures: false,
            carryable_aboard_vessel: false,
            usable_as_mount: false,
            handlers_required_per_n_head: Some(10.0),
            upkeep_sp_day_head: Some(2.0),
            species_key: None,
        },
    ]
}

/// TRAVEL_LIBRARY_SPEC.md §3.2's stock roster, one per existing party-form
/// vehicle count (`crate::JpParty::carts`/`wagons`/`sleds`/`travois`) plus a
/// human-drawn Handcart the party form has no dedicated slot for yet. Load
/// and draft-head figures mirror `jp_capacity`'s own built-in
/// `JP_CART_CAP`/`JP_WAGON_CAP`/`JP_SLED_CAP`/`JP_TRAVOIS_CAP`/
/// `JP_CART_DRAFT`/`JP_WAGON_DRAFT`/`JP_SLED_DRAFT` constants exactly, for
/// the same reason the four stock animals mirror `jp_animal_stats`: this is
/// existing golden-tested data, not invented twice.
pub fn stock_vehicles() -> Vec<VehicleDef> {
    vec![
        VehicleDef {
            id: "handcart".into(),
            origin: EntryOrigin::Stock,
            name: "Handcart".into(),
            class: Some(VehicleClass::Wheeled),
            load_kg: Some(150.0),
            draft_head_required: Some(DraftRequirement {
                count: 0,
                role: "human-drawn".into(),
            }),
            speed_mult: Some(0.9),
            road_requirement: Some(RoadRequirement::Track),
            off_road: Some(TerrainAffinity::Multiplier(0.5)),
            ford: Some(TerrainAffinity::Blocked),
            carryable_aboard_vessel: true,
        },
        VehicleDef {
            id: "cart".into(),
            origin: EntryOrigin::Stock,
            name: "Cart".into(),
            class: Some(VehicleClass::Wheeled),
            load_kg: Some(750.0),
            draft_head_required: Some(DraftRequirement {
                count: 2,
                role: "draft animal".into(),
            }),
            speed_mult: Some(1.0),
            road_requirement: Some(RoadRequirement::Track),
            off_road: Some(TerrainAffinity::Multiplier(0.4)),
            ford: Some(TerrainAffinity::Blocked),
            carryable_aboard_vessel: true,
        },
        VehicleDef {
            id: "wagon".into(),
            origin: EntryOrigin::Stock,
            name: "Wagon".into(),
            class: Some(VehicleClass::Wheeled),
            load_kg: Some(1000.0),
            draft_head_required: Some(DraftRequirement {
                count: 3,
                role: "draft animal".into(),
            }),
            speed_mult: Some(0.85),
            road_requirement: Some(RoadRequirement::Road),
            off_road: Some(TerrainAffinity::Blocked),
            ford: Some(TerrainAffinity::Blocked),
            carryable_aboard_vessel: false,
        },
        VehicleDef {
            id: "sledge".into(),
            origin: EntryOrigin::Stock,
            name: "Sledge".into(),
            class: Some(VehicleClass::Dragged),
            load_kg: Some(500.0),
            draft_head_required: Some(DraftRequirement {
                count: 2,
                role: "draft animal".into(),
            }),
            speed_mult: Some(1.0),
            road_requirement: Some(RoadRequirement::None),
            off_road: Some(TerrainAffinity::Multiplier(1.0)),
            ford: Some(TerrainAffinity::Blocked),
            carryable_aboard_vessel: false,
        },
        VehicleDef {
            id: "travois".into(),
            origin: EntryOrigin::Stock,
            name: "Travois".into(),
            class: Some(VehicleClass::Dragged),
            load_kg: Some(100.0),
            draft_head_required: Some(DraftRequirement {
                count: 1,
                role: "pack animal".into(),
            }),
            speed_mult: Some(0.95),
            road_requirement: Some(RoadRequirement::None),
            off_road: Some(TerrainAffinity::Multiplier(0.8)),
            ford: Some(TerrainAffinity::Multiplier(0.5)),
            carryable_aboard_vessel: true,
        },
    ]
}

/// TRAVEL_LIBRARY_SPEC.md §3.3's stock roster -- every entry mirrors
/// `crate::jp_ship_stats`' own built-in `speed_kmh`/`cargo_kg`/`crew`
/// figures exactly, with `water_rating` derived from that same function's
/// `river`/`sea`/`open_sea` flags (open sea capable -> `Open`; sea but not
/// open-sea -> `Coastal`; river-only -> `Sheltered`). `sailing_window` and
/// `portage_capable` are genuinely new fields with no engine equivalent:
/// oared/river craft get `Daylight` and portage capability; true sailing
/// ships get `Continuous` and none.
pub fn stock_vessels() -> Vec<VesselDef> {
    let row = |id: &str,
               name: &str,
               modes: Vec<VesselMode>,
               hold: f64,
               crew: u32,
               speed: f64,
               rating: WaterRating,
               window: SailingWindow,
               portage: bool| VesselDef {
        id: id.into(),
        origin: EntryOrigin::Stock,
        name: name.into(),
        modes,
        hold_kg: Some(hold),
        crew_required: Some(crew),
        base_speed_kmh: Some(speed),
        water_rating: Some(rating),
        sailing_window: Some(window),
        portage_capable: portage,
    };
    vec![
        row(
            "river_barge",
            "River Barge",
            vec![VesselMode::River],
            30000.0,
            12,
            2.0,
            WaterRating::Sheltered,
            SailingWindow::Daylight,
            false,
        ),
        row(
            "keelboat",
            "Keelboat",
            vec![VesselMode::River, VesselMode::Sea],
            8000.0,
            8,
            4.0,
            WaterRating::Coastal,
            SailingWindow::Daylight,
            true,
        ),
        row(
            "river_galley",
            "River Galley",
            vec![VesselMode::River],
            3000.0,
            30,
            5.0,
            WaterRating::Sheltered,
            SailingWindow::Daylight,
            true,
        ),
        row(
            "fishing_vessel",
            "Fishing Vessel",
            vec![VesselMode::Sea],
            1500.0,
            4,
            8.0,
            WaterRating::Coastal,
            SailingWindow::Daylight,
            true,
        ),
        row(
            "longship",
            "Longship",
            vec![VesselMode::River, VesselMode::Sea],
            5000.0,
            40,
            11.0,
            WaterRating::Open,
            SailingWindow::Continuous,
            true,
        ),
        row(
            "cog",
            "Cog",
            vec![VesselMode::Sea],
            80000.0,
            20,
            10.0,
            WaterRating::Open,
            SailingWindow::Continuous,
            false,
        ),
        row(
            "dhow",
            "Dhow",
            vec![VesselMode::Sea],
            20000.0,
            15,
            12.0,
            WaterRating::Open,
            SailingWindow::Continuous,
            false,
        ),
        row(
            "caravel",
            "Caravel",
            vec![VesselMode::Sea],
            30000.0,
            20,
            13.0,
            WaterRating::Open,
            SailingWindow::Continuous,
            false,
        ),
        row(
            "carrack",
            "Carrack",
            vec![VesselMode::Sea],
            200000.0,
            80,
            11.0,
            WaterRating::Open,
            SailingWindow::Continuous,
            false,
        ),
        row(
            "galleon",
            "Galleon",
            vec![VesselMode::Sea],
            300000.0,
            150,
            13.0,
            WaterRating::Open,
            SailingWindow::Continuous,
            false,
        ),
        row(
            "fluyt",
            "Fluyt",
            vec![VesselMode::Sea],
            250000.0,
            50,
            11.0,
            WaterRating::Open,
            SailingWindow::Continuous,
            false,
        ),
    ]
}

/// TRAVEL_LIBRARY_SPEC.md §3.4's stock roster: two representative party
/// set-ups spanning the party form's own extremes (a light pack column and
/// a heavier wagon caravan), grounded in [`crate::JpPlan::default`]'s own
/// reference-derived defaults for every field a preset does not need to
/// vary.
pub fn stock_party_presets() -> Vec<PartyPreset> {
    let base = crate::JpPlan::default();
    vec![
        PartyPreset {
            id: "light_pack_column".into(),
            origin: EntryOrigin::Stock,
            name: "Light Pack Column".into(),
            transport: "Baggage Train".into(),
            mount_animal: Some("mule".into()),
            vessel: base.vessel.clone(),
            hours: 8.0,
            pace: "Standard Pace".into(),
            season: "Summer".into(),
            supply_days: 7,
            carry_food: true,
            grazing: "Partial — graze at camp".into(),
            foraging: "None".into(),
            party: crate::JpParty {
                group_size: 6,
                cargo_kg: 200.0,
                mule: 4,
                ..crate::JpParty::default()
            },
        },
        PartyPreset {
            id: "heavy_wagon_caravan".into(),
            origin: EntryOrigin::Stock,
            name: "Heavy Wagon Caravan".into(),
            transport: "Baggage Train".into(),
            mount_animal: Some("horse".into()),
            vessel: base.vessel,
            hours: 8.0,
            pace: "Standard Pace".into(),
            season: "Summer".into(),
            supply_days: 14,
            carry_food: true,
            grazing: "None — carry all fodder".into(),
            foraging: "None".into(),
            party: crate::JpParty {
                group_size: 20,
                cargo_kg: 800.0,
                horse: 6,
                wagons: 2,
                ..crate::JpParty::default()
            },
        },
    ]
}

// ---------------------------------------------------------------------------
// §6 -- wiring the resolver
// ---------------------------------------------------------------------------

/// [`animal_resolver_fns`]' first return value -- named purely to keep that
/// signature readable (`clippy::type_complexity`).
pub type AnimalStatsFn<'a> = Box<dyn Fn(&str) -> Option<crate::AnimalStats> + 'a>;
/// [`animal_resolver_fns`]' second return value.
pub type AnimalTerrainFn<'a> = Box<dyn Fn(&str, &str) -> Option<Option<f64>> + 'a>;

/// Build the two closures [`crate::JpAnimalResolver`] wants from a
/// species-keyed override map (`"donkey"|"mule"|"camel"|"horse"` ->
/// the [`AnimalDef`] currently standing in for that species). Every
/// override query still falls back to the built-in table centrally inside
/// `jp_capacity_ex`/`jp_calc_land_ex` (see [`crate::JpAnimalResolver`]'s own
/// doc comment) -- a partially-incomplete override (one `Option` field
/// still unset) degrades gracefully to the built-in figure for exactly that
/// field, per-lookup, not an all-or-nothing swap.
///
/// `cartalith-godot`'s `travel_bridge::TravelLibrary::animal_overrides`
/// builds the map this takes; see that method's own doc comment for how a
/// custom entry is chosen for a species when more than one exists.
pub fn animal_resolver_fns(overrides: &HashMap<String, AnimalDef>) -> (AnimalStatsFn<'_>, AnimalTerrainFn<'_>) {
    let stats = move |key: &str| -> Option<crate::AnimalStats> {
        let def = overrides.get(key)?;
        Some(crate::AnimalStats {
            cap_kg: def.load_capacity_kg?,
            food_kg_day: def.fodder_need_kg_day?,
            water_l_day: def.water_need_l_day?,
            mounted_speed_kmh: def.base_speed_kmh?,
            // The built-in label, not the custom entry's own display name:
            // `AnimalStats::label` is `&'static str`, and this override path
            // only ever represents one of the four built-in species -- see
            // the module docs' "What's not yet wired".
            label: crate::jp_animal_stats(key)
                .map(|a| a.label)
                .unwrap_or("Animal"),
        })
    };
    let terrain_mod = move |key: &str, engine_terrain: &str| -> Option<Option<f64>> {
        let def = overrides.get(key)?;
        let tl_key = tl_terrain_key_for_engine(engine_terrain)?;
        match def.terrain.get(tl_key)? {
            TerrainAffinity::Multiplier(m) => Some(Some(*m)),
            TerrainAffinity::Blocked => Some(None),
        }
    };
    (Box::new(stats), Box::new(terrain_mod))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------- stock data is real, valid data ----------

    #[test]
    fn every_stock_animal_validates_ok() {
        for a in stock_animals() {
            assert_eq!(validate_animal(&a), ValidationState::Ok, "{}", a.name);
        }
    }

    #[test]
    fn every_stock_vehicle_validates_ok() {
        for v in stock_vehicles() {
            assert_eq!(validate_vehicle(&v), ValidationState::Ok, "{}", v.name);
        }
    }

    #[test]
    fn every_stock_vessel_validates_ok() {
        for v in stock_vessels() {
            assert_eq!(validate_vessel(&v), ValidationState::Ok, "{}", v.name);
        }
    }

    #[test]
    fn every_stock_party_preset_validates_ok() {
        for p in stock_party_presets() {
            assert_eq!(validate_party_preset(&p), ValidationState::Ok, "{}", p.name);
        }
    }

    #[test]
    fn four_stock_animals_carry_the_engine_species_key_the_other_three_do_not() {
        let keyed: Vec<&str> = stock_animals()
            .iter()
            .filter_map(|a| a.species_key)
            .collect();
        assert_eq!(keyed, vec!["donkey", "mule", "camel", "horse"]);
    }

    // ---------- validation states: all three reachable ----------

    #[test]
    fn a_blank_animal_is_incomplete_and_names_every_missing_field() {
        let a = AnimalDef::blank("test", "Test Animal");
        let ValidationState::Incomplete(missing) = validate_animal(&a) else {
            panic!("expected Incomplete")
        };
        assert!(missing.contains(&"load capacity kg"));
        assert!(missing.contains(&"terrain constraints"));
        assert!(missing.contains(&"availability"));
    }

    #[test]
    fn grazing_vs_terrain_is_the_specs_own_worked_conflict() {
        // TRAVEL_LIBRARY_SPEC.md §4's own example: grassland-only grazing,
        // but a non-grassland terrain still carries a real multiplier.
        let mut a = stock_animals()
            .into_iter()
            .find(|a| a.id == "mule")
            .unwrap();
        a.grazing_tolerance = Some(GrazingTolerance::GrasslandOnly);
        let ValidationState::Conflicting(reasons) = validate_animal(&a) else {
            panic!("expected Conflicting")
        };
        assert!(reasons.iter().any(|r| r.contains("grassland-only")));
    }

    #[test]
    fn mount_role_and_usable_as_mount_must_agree() {
        let mut a = stock_animals()
            .into_iter()
            .find(|a| a.id == "donkey")
            .unwrap();
        a.usable_as_mount = false; // roles still claim Mount
        let ValidationState::Conflicting(reasons) = validate_animal(&a) else {
            panic!("expected Conflicting")
        };
        assert!(reasons.iter().any(|r| r.contains("usable as a mount")));
    }

    #[test]
    fn incomplete_takes_priority_over_a_conflict_that_would_otherwise_fire() {
        // Same contradiction as above, but with a constraint field also
        // unset -- Incomplete must win, per this module's own ordering.
        let mut a = stock_animals()
            .into_iter()
            .find(|a| a.id == "donkey")
            .unwrap();
        a.usable_as_mount = false;
        a.waterless_limit_days = None;
        assert!(matches!(
            validate_animal(&a),
            ValidationState::Incomplete(_)
        ));
    }

    #[test]
    fn vehicle_no_road_needed_but_off_road_blocked_is_conflicting() {
        let mut v = stock_vehicles()
            .into_iter()
            .find(|v| v.id == "sledge")
            .unwrap();
        v.off_road = Some(TerrainAffinity::Blocked);
        assert!(matches!(
            validate_vehicle(&v),
            ValidationState::Conflicting(_)
        ));
    }

    #[test]
    fn vessel_sheltered_but_rated_for_sea_is_conflicting() {
        let mut v = stock_vessels()
            .into_iter()
            .find(|v| v.id == "river_barge")
            .unwrap();
        v.modes.push(VesselMode::Sea);
        assert!(matches!(
            validate_vessel(&v),
            ValidationState::Conflicting(_)
        ));
    }

    #[test]
    fn a_blank_party_preset_is_incomplete_never_conflicting() {
        let mut p = PartyPreset::blank("t", "Test");
        p.transport.clear();
        p.party.group_size = 0;
        assert!(matches!(
            validate_party_preset(&p),
            ValidationState::Incomplete(_)
        ));
    }

    // ---------- terrain vocabulary mapping ----------

    #[test]
    fn engine_roads_are_deliberately_unmapped() {
        assert_eq!(tl_terrain_key_for_engine("Paved Road"), None);
        assert_eq!(tl_terrain_key_for_engine("Dirt Track"), None);
        assert_eq!(tl_terrain_key_for_engine("Ruins / Debris"), None);
    }

    #[test]
    fn every_engine_land_terrain_that_does_map_names_a_real_tl_key() {
        for t in [
            "Open Plains",
            "Forest Path",
            "Hills",
            "Rocky Terrain",
            "Mountain Trails",
            "Mountain Pass",
            "Swamp / Marsh",
            "Desert Hardpack",
            "Deep Sand",
            "Snow / Ice",
        ] {
            let k = tl_terrain_key_for_engine(t).unwrap_or_else(|| panic!("{t} should map"));
            assert!(TL_TERRAIN_KEYS.contains(&k), "{k} not in TL_TERRAIN_KEYS");
        }
    }

    // ---------- party preset round trip ----------

    #[test]
    fn a_captured_preset_applies_back_onto_a_plan_unchanged() {
        let plan = crate::JpPlan {
            transport: "Mounted Rider".into(),
            mount_animal: Some("camel".into()),
            party: crate::JpParty {
                group_size: 9,
                camel: 2,
                ..crate::JpParty::default()
            },
            ..crate::JpPlan::default()
        };
        let preset = PartyPreset::from_jp_plan("p1", "Captured", &plan);
        let applied = preset.apply_to(&crate::JpPlan::default());
        assert_eq!(applied.transport, plan.transport);
        assert_eq!(applied.mount_animal, plan.mount_animal);
        assert_eq!(applied.party, plan.party);
        // Route-only fields are untouched -- they came from `base`, not the preset.
        assert_eq!(applied.route_cond, crate::JpPlan::default().route_cond);
    }

    // ---------- the resolver ----------

    #[test]
    fn animal_resolver_falls_back_to_the_built_in_table_for_an_unlisted_species() {
        let overrides: HashMap<String, AnimalDef> = HashMap::new();
        let (stats, terrain) = animal_resolver_fns(&overrides);
        assert_eq!(
            stats("donkey"),
            None,
            "no override present -- resolve_animal_stats falls back centrally, not here"
        );
        assert_eq!(terrain("donkey", "Hills"), None);
    }

    #[test]
    fn animal_resolver_reports_a_real_override_and_a_blocked_terrain() {
        let mut donkey = stock_animals()
            .into_iter()
            .find(|a| a.id == "donkey")
            .unwrap();
        donkey.load_capacity_kg = Some(999.0);
        donkey.terrain.insert("Marsh", TerrainAffinity::Blocked);
        let mut overrides = HashMap::new();
        overrides.insert("donkey".to_string(), donkey);
        let (stats, terrain) = animal_resolver_fns(&overrides);
        assert_eq!(stats("donkey").unwrap().cap_kg, 999.0);
        assert_eq!(terrain("donkey", "Swamp / Marsh"), Some(None), "blocked");
        assert_eq!(
            terrain("donkey", "Hills"),
            Some(Some(0.80)),
            "unmodified row from the stock entry"
        );
    }

    #[test]
    fn a_partially_incomplete_override_yields_no_stats_for_that_species() {
        // `animal_resolver_fns`' own contract: an incomplete override does
        // not partially apply here -- `resolve_animal_stats` (cartalith-civ
        // lib.rs) is what falls back to the built-in table field-by-field,
        // by re-trying `jp_animal_stats` whenever this closure returns
        // `None` for the whole struct.
        let mut donkey = stock_animals()
            .into_iter()
            .find(|a| a.id == "donkey")
            .unwrap();
        donkey.load_capacity_kg = None;
        let mut overrides = HashMap::new();
        overrides.insert("donkey".to_string(), donkey);
        let (stats, _) = animal_resolver_fns(&overrides);
        assert_eq!(stats("donkey"), None);
    }
}
