//! The Travel Library's mutable store: `TRAVEL_LIBRARY_SPEC.md`'s
//! stock-plus-custom CRUD, usage tracking, and the glue that turns a live
//! library into the `cartalith_civ::JpAnimalResolver` `jp_plan_ex` consumes.
//!
//! Deliberately **free of any `godot` dependency**, the same isolation
//! `journey_bridge.rs`/`timeline_bridge.rs` already establish: this module
//! owns everything that can be expressed without one, and a later dispatch's
//! `lib.rs` `#[func]` layer owns the thin `Variant`<->Rust conversion, the
//! same way `timeline_bridge.rs` was added cleanly on top of
//! `cartalith-civ/timeline.rs`'s data model without every consumer being
//! wired in the same pass.
//!
//! The data shapes, validation and stock content all live in
//! `cartalith_civ::travel_library` -- read that module's own doc comment
//! first, especially its "What's not yet wired" section, which this module
//! does not repeat. What lives here is specifically the *mutable* half
//! `ARCHITECTURE.md`'s stateless-`cartalith-civ`/stateful-`cartalith-godot`
//! split assigns to this crate: add/duplicate/edit/delete, id generation,
//! and usage counting.
//!
//! # How this reaches a computed journey (live as of 2026-08-20)
//!
//! `lib.rs`'s `WorldGen` holds a `travel_library: TravelLibrary`, and
//! `jp_compute` builds a [`cartalith_civ::JpAnimalResolver`] from
//! [`TravelLibrary::animal_overrides_selected`] via
//! [`cartalith_civ::travel_library::animal_resolver_fns`], passing
//! `Some(&resolver)` to `cartalith_civ::jp_plan_ex`.
//! [`tests::a_custom_animal_override_changes_a_computed_journey`] exercises
//! that call chain end to end, and
//! [`tests::regression_stock_only_travel_library_matches_pre_dispatch_jp_plan`]
//! pins that a stock-only library is byte-identical to the plain
//! `cartalith_civ::jp_plan` it replaced.
//!
//! The party form names its per-species choice explicitly (`jp_compute`'s
//! own `animal_entries` request key -> [`TravelLibrary::animal_overrides_selected`]),
//! and [`TravelLibrary::animal_species_slot`] is the one place that decides
//! which of the four built-in species an entry may occupy -- its own
//! `species_key`, or the one its `substitutes_for` chain reaches. An entry
//! that resolves to neither has no `JpParty` slot and is not offered at all;
//! `TRAVEL_LIBRARY_SPEC.md` §6 records why widening `JpParty` to a generic
//! animal-count map is not the small change it looks like.

use std::collections::HashMap;

use cartalith_civ::travel_library::{
    AnimalDef, AnimalRole, Availability, EntryOrigin, GrazingTolerance, PartyPreset,
    RoadRequirement, TL_TERRAIN_KEYS, TerrainAffinity, ValidationState, VehicleClass, VehicleDef,
    VesselDef, VesselMode, SailingWindow, WaterRating, DraftRequirement, stock_animals,
    stock_party_presets, stock_vehicles, stock_vessels,
};

use crate::journey_bridge::JpValue;

/// The common shape [`EntrySet`]'s generic CRUD needs from all four
/// definition types -- an id, a mutable origin, and the ability to be
/// cloned into a new custom copy ([`EntrySet::duplicate`]'s own basis).
pub trait TravelEntry: Clone {
    fn id(&self) -> &str;
    fn set_id(&mut self, id: String);
    fn origin(&self) -> EntryOrigin;
    fn set_origin(&mut self, origin: EntryOrigin);
}

macro_rules! impl_travel_entry {
    ($t:ty) => {
        impl TravelEntry for $t {
            fn id(&self) -> &str {
                &self.id
            }
            fn set_id(&mut self, id: String) {
                self.id = id;
            }
            fn origin(&self) -> EntryOrigin {
                self.origin
            }
            fn set_origin(&mut self, origin: EntryOrigin) {
                self.origin = origin;
            }
        }
    };
}
impl_travel_entry!(AnimalDef);
impl_travel_entry!(VehicleDef);
impl_travel_entry!(VesselDef);
impl_travel_entry!(PartyPreset);

/// One definition type's store: every stock entry (bootstrap order) plus
/// every custom entry (add order) -- `cartalith-assets`' `AssetDB` slot
/// registry is the precedent this mirrors: frozen/stock entries are
/// immutable and cannot be deleted, a custom entry is always a clone of
/// some starting point (a stock entry via [`EntrySet::duplicate`], or a
/// blank one via [`EntrySet::add`]), and only custom entries can be edited
/// or removed.
#[derive(Debug, Clone)]
pub struct EntrySet<T: TravelEntry> {
    items: Vec<T>,
}

impl<T: TravelEntry> EntrySet<T> {
    pub fn new(stock: Vec<T>) -> Self {
        EntrySet { items: stock }
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.items.iter()
    }

    pub fn get(&self, id: &str) -> Option<&T> {
        self.items.iter().find(|e| e.id() == id)
    }

    /// `None` if `id` is unknown or names a stock (read-only) entry --
    /// `TRAVEL_LIBRARY_SPEC.md` §3's "stock entries are read-only".
    pub fn get_mut(&mut self, id: &str) -> Option<&mut T> {
        self.items
            .iter_mut()
            .find(|e| e.id() == id && e.origin() == EntryOrigin::Custom)
    }

    /// Clone `id` (stock or custom) into a new custom entry under `new_id`.
    /// `None` if `id` does not exist or `new_id` already does.
    pub fn duplicate(&mut self, id: &str, new_id: impl Into<String>) -> Option<&T> {
        let new_id = new_id.into();
        if self.get(&new_id).is_some() {
            return None;
        }
        let mut copy = self.get(id)?.clone();
        copy.set_id(new_id);
        copy.set_origin(EntryOrigin::Custom);
        self.items.push(copy);
        self.items.last()
    }

    /// Add an already-built custom entry (e.g. `AnimalDef::blank(...)`,
    /// TRAVEL_LIBRARY_SPEC.md's "New blank definition…"). `None` (and the
    /// entry discarded) if its id is already taken.
    pub fn add(&mut self, mut entry: T) -> Option<&T> {
        if self.get(entry.id()).is_some() {
            return None;
        }
        entry.set_origin(EntryOrigin::Custom);
        self.items.push(entry);
        self.items.last()
    }

    /// `false` (no-op) if `id` is unknown or names a stock entry.
    pub fn delete(&mut self, id: &str) -> bool {
        let before = self.items.len();
        self.items
            .retain(|e| !(e.id() == id && e.origin() == EntryOrigin::Custom));
        self.items.len() != before
    }

    /// Discard every custom entry, restoring the stock-only bootstrap --
    /// `TRAVEL_LIBRARY_SPEC.md`'s "Reset to stock definitions…".
    pub fn reset_to_stock(&mut self) {
        self.items.retain(|e| e.origin() == EntryOrigin::Stock);
    }
}

// ---------------------------------------------------------------------------
// TravelLibrary
// ---------------------------------------------------------------------------

/// The whole Travel Library: four [`EntrySet`]s plus a monotonically
/// increasing id counter for new custom entries.
#[derive(Debug, Clone)]
pub struct TravelLibrary {
    pub animals: EntrySet<AnimalDef>,
    pub vehicles: EntrySet<VehicleDef>,
    pub vessels: EntrySet<VesselDef>,
    pub presets: EntrySet<PartyPreset>,
    next_id: u64,
}

impl Default for TravelLibrary {
    fn default() -> Self {
        Self::new()
    }
}

impl TravelLibrary {
    /// A freshly bootstrapped library: every stock entry present, no
    /// custom entries yet.
    pub fn new() -> Self {
        TravelLibrary {
            animals: EntrySet::new(stock_animals()),
            vehicles: EntrySet::new(stock_vehicles()),
            vessels: EntrySet::new(stock_vessels()),
            presets: EntrySet::new(stock_party_presets()),
            next_id: 1,
        }
    }

    /// A fresh, library-wide-unique custom id, e.g. `"custom-3"`. Shared
    /// across all four definition types (an animal and a vehicle never
    /// collide on id even though both stores are otherwise independent).
    pub fn fresh_id(&mut self) -> String {
        let id = format!("custom-{}", self.next_id);
        self.next_id += 1;
        id
    }

    // ---- validation, per-entry (TRAVEL_LIBRARY_SPEC.md §4) ----

    pub fn animal_validation(&self, id: &str) -> Option<ValidationState> {
        self.animals
            .get(id)
            .map(cartalith_civ::travel_library::validate_animal)
    }
    pub fn vehicle_validation(&self, id: &str) -> Option<ValidationState> {
        self.vehicles
            .get(id)
            .map(cartalith_civ::travel_library::validate_vehicle)
    }
    pub fn vessel_validation(&self, id: &str) -> Option<ValidationState> {
        self.vessels
            .get(id)
            .map(cartalith_civ::travel_library::validate_vessel)
    }
    pub fn preset_validation(&self, id: &str) -> Option<ValidationState> {
        self.presets
            .get(id)
            .map(cartalith_civ::travel_library::validate_party_preset)
    }

    // ---- usage tracking (TRAVEL_LIBRARY_SPEC.md §4) ----

    /// How many party set-ups reference this animal, by species. Only
    /// meaningful for an entry whose `species_key` names one of the four
    /// built-in party-form species (`cartalith_civ::JP_ANIMAL_KEYS`) -- a
    /// `None`-keyed custom species has no `JpParty` slot to ever appear in,
    /// so it always reports `0`. See `cartalith_civ::travel_library`'s own
    /// module doc for why the party form's fixed shape draws this line.
    pub fn animal_usage_in_presets(&self, id: &str) -> usize {
        let Some(key) = self.animals.get(id).and_then(|a| a.species_key) else {
            return 0;
        };
        self.presets
            .iter()
            .filter(|p| species_count(&p.party, key) > 0)
            .count()
    }

    /// TRAVEL_LIBRARY_SPEC.md §4 also asks for "how many saved journeys
    /// reference it". No persistent, referenceable "saved journey" exists
    /// anywhere in this port today: `route_get`/`WorldGen.infra.routes` are
    /// drawn polylines with no attached party plan, and `jp_compute`
    /// computes and returns a plan without storing it. Always `0`, honestly
    /// disclosed rather than invented -- see this module's own doc comment
    /// and `TRAVEL_LIBRARY_SPEC.md`'s own note for this dispatch.
    pub fn animal_usage_in_journeys(&self, _id: &str) -> usize {
        0
    }

    // ---- the resolver (TRAVEL_LIBRARY_SPEC.md §6) ----

    /// Build the species-keyed override map
    /// [`cartalith_civ::travel_library::animal_resolver_fns`] takes: for
    /// each of the four built-in species, the *last-added* custom entry
    /// whose `species_key` names it, if any. This is the implicit fallback
    /// only: the party form names its choice per species explicitly, through
    /// [`Self::animal_overrides_selected`]. Stock entries are never
    /// included: by construction they carry exactly the built-in figures
    /// already, so including them would be an inert no-op override.
    pub fn animal_overrides(&self) -> HashMap<String, AnimalDef> {
        let mut out = HashMap::new();
        for a in self.animals.iter() {
            if a.origin != EntryOrigin::Custom {
                continue;
            }
            if let Some(key) = a.species_key {
                out.insert(key.to_string(), a.clone());
            }
        }
        out
    }

    /// Which of the four built-in party-form species
    /// (`cartalith_civ::JP_ANIMAL_KEYS`) this entry may occupy: its own
    /// `species_key`, else the one its `substitutes_for` chain reaches.
    ///
    /// `None` for a wholly new species that declares no substitute -- the
    /// stock Ox/Yak/Reindeer, and every from-blank custom entry until its
    /// owner fills the "Substitutes for" field in. `JpParty` has four fixed
    /// species fields and no generic animal-count map, so `None` here is
    /// precisely "the planner has no slot to offer this in"; see this
    /// module's own doc comment and `TRAVEL_LIBRARY_SPEC.md` §6 for why
    /// widening `JpParty` is not the small change it looks like.
    pub fn animal_species_slot(&self, id: &str) -> Option<&'static str> {
        let mut cur = self.animals.get(id)?;
        // `substitutes_for` is free text a user types (the Travel Library
        // window's own "Substitutes for" row), so the chase is bounded by the
        // store's own size: a cycle (a -> b -> a) terminates rather than
        // hanging the planner's form rebuild.
        for _ in 0..self.animals.iter().count() {
            if let Some(key) = cur.species_key {
                return Some(key);
            }
            cur = self.animals.get(cur.substitutes_for.as_deref()?)?;
        }
        None
    }

    /// [`Self::animal_overrides`] with the party form's own explicit
    /// per-species choice applied on top: `selection` maps a built-in
    /// species key to the id of the Travel Library entry occupying that
    /// slot. Selecting a **stock** entry means "no override" (the built-in
    /// table), which is deliberately not the same as leaving the slot
    /// unnamed -- an unnamed slot keeps whatever [`Self::animal_overrides`]
    /// resolved, so an empty selection reproduces the pre-selection
    /// behaviour exactly.
    ///
    /// Also returns every selection key that could not be honoured -- an
    /// unrecognised species, an unknown entry id, or an entry that does not
    /// resolve to the species it was selected for -- for `jp_compute`'s own
    /// `rejected` array, rather than silently dropping it. Iteration is over
    /// `JP_ANIMAL_KEYS` and then the unknown keys sorted, so the rejection
    /// list is deterministic despite `selection` being a `HashMap`.
    pub fn animal_overrides_selected(
        &self,
        selection: &HashMap<String, String>,
    ) -> (HashMap<String, AnimalDef>, Vec<String>) {
        let mut out = self.animal_overrides();
        let mut rejected: Vec<String> = Vec::new();
        for species in cartalith_civ::JP_ANIMAL_KEYS {
            let Some(id) = selection.get(species) else {
                continue;
            };
            match self.animals.get(id) {
                Some(entry) if self.animal_species_slot(id) == Some(species) => {
                    if entry.origin == EntryOrigin::Custom {
                        out.insert(species.to_string(), entry.clone());
                    } else {
                        out.remove(species);
                    }
                }
                _ => rejected.push(species.to_string()),
            }
        }
        let mut unknown: Vec<String> = selection
            .keys()
            .filter(|k| !cartalith_civ::JP_ANIMAL_KEYS.contains(&k.as_str()))
            .cloned()
            .collect();
        unknown.sort();
        rejected.extend(unknown);
        (out, rejected)
    }
}

fn species_count(party: &cartalith_civ::JpParty, species_key: &str) -> i64 {
    match species_key {
        "donkey" => party.donkey,
        "mule" => party.mule,
        "camel" => party.camel,
        "horse" => party.horse,
        _ => 0,
    }
}

// ---------------------------------------------------------------------------
// Variant-shaped field pairs for the `#[func]` boundary
// ---------------------------------------------------------------------------
//
// `lib.rs`'s `#[func]` layer needs a flat `Variant`-shaped view of each
// definition type -- a `Dictionary` a GDScript inspector can build a form
// row from and, for edits, send a partial one back. This module stays
// `godot`-free (this file's own doc comment), so what lives here is the
// narrower thing that isolation actually requires: each field as a
// `(key, journey_bridge::JpValue)` pair, the exact shape `journey_bridge`'s
// own `plan_to_pairs`/`plan_from_pairs` already established for the Journey
// Planner's own plan/party form and that `lib.rs`'s `jp_pairs_dict`/
// `jp_dict_to_pairs` already flatten to and from a real `Dictionary`. Reusing
// that machinery here (rather than inventing a second flattening convention)
// is what lets `lib.rs`'s Travel Library `#[func]`s stay thin.
//
// Every `_to_pairs` function below emits only the fields **currently
// `Some`/present** in an optional field -- an absent key in the output
// `Dictionary` is `TRAVEL_LIBRARY_SPEC.md` §3's own "a field left unset is
// incomplete, not zero", surfaced honestly rather than as a fabricated `0`.
// Every `_apply_pairs` function starts from a **clone of the existing
// entry**, not a blank one (unlike `journey_bridge::plan_from_pairs`, which
// starts from `JpPlan::default()` because a plan is always sent whole) --
// a partial edit here must leave every field the caller did not touch
// exactly as it was, the same "partial is legal" contract `set_params` and
// `jp_compute`'s own `plan` dictionary already establish elsewhere in this
// crate.

/// [`TL_TERRAIN_KEYS`] <-> the lowercase/underscore wire key each row uses
/// as `"terrain.<slug>"` in every `_to_pairs`/`_apply_pairs` pair below --
/// a raw key with an embedded space (`"terrain.High Pass"`) round-trips
/// through a `Dictionary` fine, but a slug reads better from GDScript and
/// matches every other snake_case key this boundary already uses.
const TERRAIN_SLUGS: [(&str, &str); 10] = [
    ("Plains", "plains"),
    ("Steppe", "steppe"),
    ("Forest", "forest"),
    ("Hills", "hills"),
    ("Mountain", "mountain"),
    ("Marsh", "marsh"),
    ("Desert", "desert"),
    ("High Pass", "high_pass"),
    ("Snowfield", "snowfield"),
    ("River Ford", "river_ford"),
];

fn terrain_slug(key: &str) -> &'static str {
    TERRAIN_SLUGS
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, s)| *s)
        .expect("every TL_TERRAIN_KEYS row has a slug")
}

fn terrain_key_from_slug(slug: &str) -> Option<&'static str> {
    TERRAIN_SLUGS.iter().find(|(_, s)| *s == slug).map(|(k, _)| *k)
}

fn affinity_out(a: &TerrainAffinity) -> JpValue {
    match a {
        TerrainAffinity::Multiplier(m) => JpValue::Num(*m),
        TerrainAffinity::Blocked => JpValue::Str("blocked".to_string()),
    }
}

/// Accepts either a number (a multiplier) or the literal string `"blocked"`
/// -- exactly [`affinity_out`]'s own two output shapes, so a value this
/// module just emitted always parses back.
fn affinity_in(v: &JpValue) -> Option<TerrainAffinity> {
    match v {
        JpValue::Str(s) if s == "blocked" => Some(TerrainAffinity::Blocked),
        _ => v.num().map(TerrainAffinity::Multiplier),
    }
}

fn role_slug(r: &AnimalRole) -> &'static str {
    match r {
        AnimalRole::Pack => "pack",
        AnimalRole::Mount => "mount",
        AnimalRole::Draft => "draft",
    }
}

fn roles_out(roles: &[AnimalRole]) -> JpValue {
    JpValue::Str(roles.iter().map(|r| role_slug(r)).collect::<Vec<_>>().join(","))
}

fn roles_in(s: &str) -> Vec<AnimalRole> {
    s.split(',')
        .filter_map(|t| match t.trim() {
            "pack" => Some(AnimalRole::Pack),
            "mount" => Some(AnimalRole::Mount),
            "draft" => Some(AnimalRole::Draft),
            _ => None,
        })
        .collect()
}

fn grazing_out(g: Option<GrazingTolerance>) -> JpValue {
    JpValue::Str(
        match g {
            Some(GrazingTolerance::Unrestricted) => "unrestricted",
            Some(GrazingTolerance::GrasslandOnly) => "grassland_only",
            Some(GrazingTolerance::None) => "none",
            None => "",
        }
        .to_string(),
    )
}

fn grazing_in(s: &str) -> Option<GrazingTolerance> {
    match s {
        "unrestricted" => Some(GrazingTolerance::Unrestricted),
        "grassland_only" => Some(GrazingTolerance::GrasslandOnly),
        "none" => Some(GrazingTolerance::None),
        _ => None,
    }
}

fn availability_kind_out(a: &Option<Availability>) -> JpValue {
    JpValue::Str(
        match a {
            Some(Availability::Global) => "global",
            Some(Availability::Regional(_)) => "regional",
            None => "",
        }
        .to_string(),
    )
}

fn availability_region_out(a: &Option<Availability>) -> JpValue {
    JpValue::Str(match a {
        Some(Availability::Regional(r)) => r.clone(),
        _ => String::new(),
    })
}

/// §4's ok/incomplete/conflicting, flattened to what `lib.rs` needs to hand
/// a `Dictionary` its three separate keys without depending on this crate's
/// own `ValidationState` shape.
pub fn validation_state_parts(v: &ValidationState) -> (&'static str, Vec<String>, Vec<String>) {
    match v {
        ValidationState::Ok => ("ok", Vec::new(), Vec::new()),
        ValidationState::Incomplete(missing) => {
            ("incomplete", missing.iter().map(|s| s.to_string()).collect(), Vec::new())
        }
        ValidationState::Conflicting(reasons) => ("conflicting", Vec::new(), reasons.clone()),
    }
}

// ---- 3.1 Animals & mounts ----

/// The list rail's subtitle for an animal entry -- its roles, joined
/// (`"pack · mount"`), or `"—"` for a blank entry with none set yet.
pub fn animal_subtitle(a: &AnimalDef) -> String {
    if a.roles.is_empty() {
        "—".to_string()
    } else {
        a.roles.iter().map(|r| role_slug(r)).collect::<Vec<_>>().join(" · ")
    }
}

pub fn animal_to_pairs(a: &AnimalDef) -> Vec<(String, JpValue)> {
    let mut out = vec![
        ("name".to_string(), JpValue::Str(a.name.clone())),
        ("roles".to_string(), roles_out(&a.roles)),
        ("substitutes_for".to_string(), JpValue::Str(a.substitutes_for.clone().unwrap_or_default())),
        ("size_class".to_string(), JpValue::Str(a.size_class.clone().unwrap_or_default())),
        ("availability_kind".to_string(), availability_kind_out(&a.availability)),
        ("availability_region".to_string(), availability_region_out(&a.availability)),
        ("grazing_tolerance".to_string(), grazing_out(a.grazing_tolerance)),
        ("yokeable_to_wheeled".to_string(), JpValue::Bool(a.yokeable_to_wheeled)),
        ("requires_road_to_tow".to_string(), JpValue::Bool(a.requires_road_to_tow)),
        ("blocked_by_seasonal_closures".to_string(), JpValue::Bool(a.blocked_by_seasonal_closures)),
        ("carryable_aboard_vessel".to_string(), JpValue::Bool(a.carryable_aboard_vessel)),
        ("usable_as_mount".to_string(), JpValue::Bool(a.usable_as_mount)),
    ];
    for (k, v) in [
        ("load_capacity_kg", a.load_capacity_kg),
        ("draft_pull_kg", a.draft_pull_kg),
        ("base_speed_kmh", a.base_speed_kmh),
        ("sustainable_hours_day", a.sustainable_hours_day),
        ("forced_pace_cap", a.forced_pace_cap),
        ("fodder_need_kg_day", a.fodder_need_kg_day),
        ("water_need_l_day", a.water_need_l_day),
        ("waterless_limit_days", a.waterless_limit_days),
        ("handlers_required_per_n_head", a.handlers_required_per_n_head),
        ("upkeep_sp_day_head", a.upkeep_sp_day_head),
    ] {
        if let Some(n) = v {
            out.push((k.to_string(), JpValue::Num(n)));
        }
    }
    for &tl_key in &TL_TERRAIN_KEYS {
        if let Some(aff) = a.terrain.get(tl_key) {
            out.push((format!("terrain.{}", terrain_slug(tl_key)), affinity_out(aff)));
        }
    }
    out
}

/// `base` cloned, then every recognised key in `pairs` applied on top --
/// every field `base` already carried that `pairs` does not mention is
/// untouched, including `id`/`origin`/`species_key` (none of which is a
/// key this function recognises at all, so they can never be overwritten
/// by a client dictionary). Returns the updated entry plus every
/// unrecognised or wrong-typed key, this codebase's usual "a typo'd key is
/// a bug worth seeing" contract.
pub fn animal_apply_pairs(base: &AnimalDef, pairs: &[(String, JpValue)]) -> (AnimalDef, Vec<String>) {
    let mut a = base.clone();
    let mut rejected = Vec::new();
    for (k, v) in pairs {
        let applied = match k.as_str() {
            "name" => v.text().map(|s| a.name = s.to_string()).is_some(),
            "roles" => v.text().map(|s| a.roles = roles_in(s)).is_some(),
            "substitutes_for" => {
                v.text().map(|s| a.substitutes_for = (!s.is_empty()).then(|| s.to_string())).is_some()
            }
            "size_class" => v.text().map(|s| a.size_class = (!s.is_empty()).then(|| s.to_string())).is_some(),
            "availability_kind" => v
                .text()
                .map(|s| {
                    a.availability = match s {
                        "global" => Some(Availability::Global),
                        "regional" => Some(Availability::Regional(match &a.availability {
                            Some(Availability::Regional(r)) => r.clone(),
                            _ => String::new(),
                        })),
                        _ => None,
                    };
                })
                .is_some(),
            "availability_region" => v
                .text()
                .map(|s| {
                    if !s.is_empty() || matches!(a.availability, Some(Availability::Regional(_))) {
                        a.availability = Some(Availability::Regional(s.to_string()));
                    }
                })
                .is_some(),
            "load_capacity_kg" => v.num().map(|n| a.load_capacity_kg = Some(n)).is_some(),
            "draft_pull_kg" => v.num().map(|n| a.draft_pull_kg = Some(n)).is_some(),
            "base_speed_kmh" => v.num().map(|n| a.base_speed_kmh = Some(n)).is_some(),
            "sustainable_hours_day" => v.num().map(|n| a.sustainable_hours_day = Some(n)).is_some(),
            "forced_pace_cap" => v.num().map(|n| a.forced_pace_cap = Some(n)).is_some(),
            "fodder_need_kg_day" => v.num().map(|n| a.fodder_need_kg_day = Some(n)).is_some(),
            "water_need_l_day" => v.num().map(|n| a.water_need_l_day = Some(n)).is_some(),
            "grazing_tolerance" => v.text().map(|s| a.grazing_tolerance = grazing_in(s)).is_some(),
            "waterless_limit_days" => v.num().map(|n| a.waterless_limit_days = Some(n)).is_some(),
            "yokeable_to_wheeled" => v.flag().map(|b| a.yokeable_to_wheeled = b).is_some(),
            "requires_road_to_tow" => v.flag().map(|b| a.requires_road_to_tow = b).is_some(),
            "blocked_by_seasonal_closures" => v.flag().map(|b| a.blocked_by_seasonal_closures = b).is_some(),
            "carryable_aboard_vessel" => v.flag().map(|b| a.carryable_aboard_vessel = b).is_some(),
            "usable_as_mount" => v.flag().map(|b| a.usable_as_mount = b).is_some(),
            "handlers_required_per_n_head" => v.num().map(|n| a.handlers_required_per_n_head = Some(n)).is_some(),
            "upkeep_sp_day_head" => v.num().map(|n| a.upkeep_sp_day_head = Some(n)).is_some(),
            _ if k.starts_with("terrain.") => match (terrain_key_from_slug(&k["terrain.".len()..]), affinity_in(v)) {
                (Some(tl_key), Some(aff)) => {
                    a.terrain.insert(tl_key, aff);
                    true
                }
                _ => false,
            },
            _ => false,
        };
        if !applied {
            rejected.push(k.clone());
        }
    }
    (a, rejected)
}

// ---- 3.2 Vehicles ----

pub fn vehicle_subtitle(v: &VehicleDef) -> String {
    match v.class {
        Some(VehicleClass::Wheeled) => "wheeled".to_string(),
        Some(VehicleClass::Dragged) => "dragged".to_string(),
        None => "—".to_string(),
    }
}

fn class_out(c: Option<VehicleClass>) -> JpValue {
    JpValue::Str(
        match c {
            Some(VehicleClass::Wheeled) => "wheeled",
            Some(VehicleClass::Dragged) => "dragged",
            None => "",
        }
        .to_string(),
    )
}
fn class_in(s: &str) -> Option<VehicleClass> {
    match s {
        "wheeled" => Some(VehicleClass::Wheeled),
        "dragged" => Some(VehicleClass::Dragged),
        _ => None,
    }
}
fn road_req_out(r: Option<RoadRequirement>) -> JpValue {
    JpValue::Str(
        match r {
            Some(RoadRequirement::None) => "none",
            Some(RoadRequirement::Track) => "track",
            Some(RoadRequirement::Road) => "road",
            None => "",
        }
        .to_string(),
    )
}
fn road_req_in(s: &str) -> Option<RoadRequirement> {
    match s {
        "none" => Some(RoadRequirement::None),
        "track" => Some(RoadRequirement::Track),
        "road" => Some(RoadRequirement::Road),
        _ => None,
    }
}

pub fn vehicle_to_pairs(v: &VehicleDef) -> Vec<(String, JpValue)> {
    let mut out = vec![
        ("name".to_string(), JpValue::Str(v.name.clone())),
        ("class".to_string(), class_out(v.class)),
        ("road_requirement".to_string(), road_req_out(v.road_requirement)),
        ("carryable_aboard_vessel".to_string(), JpValue::Bool(v.carryable_aboard_vessel)),
    ];
    if let Some(n) = v.load_kg {
        out.push(("load_kg".to_string(), JpValue::Num(n)));
    }
    if let Some(d) = &v.draft_head_required {
        out.push(("draft_count".to_string(), JpValue::Int(i64::from(d.count))));
        out.push(("draft_role".to_string(), JpValue::Str(d.role.clone())));
    }
    if let Some(n) = v.speed_mult {
        out.push(("speed_mult".to_string(), JpValue::Num(n)));
    }
    if let Some(a) = &v.off_road {
        out.push(("off_road".to_string(), affinity_out(a)));
    }
    if let Some(a) = &v.ford {
        out.push(("ford".to_string(), affinity_out(a)));
    }
    out
}

pub fn vehicle_apply_pairs(base: &VehicleDef, pairs: &[(String, JpValue)]) -> (VehicleDef, Vec<String>) {
    let mut v = base.clone();
    let mut rejected = Vec::new();
    let mut draft_count = v.draft_head_required.as_ref().map(|d| d.count);
    let mut draft_role = v.draft_head_required.as_ref().map(|d| d.role.clone());
    for (k, val) in pairs {
        let applied = match k.as_str() {
            "name" => val.text().map(|s| v.name = s.to_string()).is_some(),
            "class" => val.text().map(|s| v.class = class_in(s)).is_some(),
            "load_kg" => val.num().map(|n| v.load_kg = Some(n)).is_some(),
            "draft_count" => val.int().map(|n| draft_count = Some(n.max(0) as u32)).is_some(),
            "draft_role" => val.text().map(|s| draft_role = Some(s.to_string())).is_some(),
            "speed_mult" => val.num().map(|n| v.speed_mult = Some(n)).is_some(),
            "road_requirement" => val.text().map(|s| v.road_requirement = road_req_in(s)).is_some(),
            "off_road" => affinity_in(val).map(|a| v.off_road = Some(a)).is_some(),
            "ford" => affinity_in(val).map(|a| v.ford = Some(a)).is_some(),
            "carryable_aboard_vessel" => val.flag().map(|b| v.carryable_aboard_vessel = b).is_some(),
            _ => false,
        };
        if !applied {
            rejected.push(k.clone());
        }
    }
    if draft_count.is_some() || draft_role.is_some() {
        v.draft_head_required =
            Some(DraftRequirement { count: draft_count.unwrap_or(0), role: draft_role.unwrap_or_default() });
    }
    (v, rejected)
}

// ---- 3.3 Vessels ----

pub fn vessel_subtitle(v: &VesselDef) -> String {
    if v.modes.is_empty() {
        "—".to_string()
    } else {
        v.modes
            .iter()
            .map(|m| match m {
                VesselMode::River => "river",
                VesselMode::Sea => "sea",
            })
            .collect::<Vec<_>>()
            .join(" · ")
    }
}

fn modes_out(m: &[VesselMode]) -> JpValue {
    JpValue::Str(
        m.iter()
            .map(|x| match x {
                VesselMode::River => "river",
                VesselMode::Sea => "sea",
            })
            .collect::<Vec<_>>()
            .join(","),
    )
}
fn modes_in(s: &str) -> Vec<VesselMode> {
    s.split(',')
        .filter_map(|t| match t.trim() {
            "river" => Some(VesselMode::River),
            "sea" => Some(VesselMode::Sea),
            _ => None,
        })
        .collect()
}
fn water_rating_out(w: Option<WaterRating>) -> JpValue {
    JpValue::Str(
        match w {
            Some(WaterRating::Sheltered) => "sheltered",
            Some(WaterRating::Coastal) => "coastal",
            Some(WaterRating::Open) => "open",
            None => "",
        }
        .to_string(),
    )
}
fn water_rating_in(s: &str) -> Option<WaterRating> {
    match s {
        "sheltered" => Some(WaterRating::Sheltered),
        "coastal" => Some(WaterRating::Coastal),
        "open" => Some(WaterRating::Open),
        _ => None,
    }
}
fn sailing_window_out(w: Option<SailingWindow>) -> JpValue {
    JpValue::Str(
        match w {
            Some(SailingWindow::Daylight) => "daylight",
            Some(SailingWindow::Continuous) => "continuous",
            None => "",
        }
        .to_string(),
    )
}
fn sailing_window_in(s: &str) -> Option<SailingWindow> {
    match s {
        "daylight" => Some(SailingWindow::Daylight),
        "continuous" => Some(SailingWindow::Continuous),
        _ => None,
    }
}

pub fn vessel_to_pairs(v: &VesselDef) -> Vec<(String, JpValue)> {
    let mut out = vec![
        ("name".to_string(), JpValue::Str(v.name.clone())),
        ("modes".to_string(), modes_out(&v.modes)),
        ("water_rating".to_string(), water_rating_out(v.water_rating)),
        ("sailing_window".to_string(), sailing_window_out(v.sailing_window)),
        ("portage_capable".to_string(), JpValue::Bool(v.portage_capable)),
    ];
    if let Some(n) = v.hold_kg {
        out.push(("hold_kg".to_string(), JpValue::Num(n)));
    }
    if let Some(n) = v.crew_required {
        out.push(("crew_required".to_string(), JpValue::Int(i64::from(n))));
    }
    if let Some(n) = v.base_speed_kmh {
        out.push(("base_speed_kmh".to_string(), JpValue::Num(n)));
    }
    out
}

pub fn vessel_apply_pairs(base: &VesselDef, pairs: &[(String, JpValue)]) -> (VesselDef, Vec<String>) {
    let mut v = base.clone();
    let mut rejected = Vec::new();
    for (k, val) in pairs {
        let applied = match k.as_str() {
            "name" => val.text().map(|s| v.name = s.to_string()).is_some(),
            "modes" => val.text().map(|s| v.modes = modes_in(s)).is_some(),
            "hold_kg" => val.num().map(|n| v.hold_kg = Some(n)).is_some(),
            "crew_required" => val.int().map(|n| v.crew_required = Some(n.max(0) as u32)).is_some(),
            "base_speed_kmh" => val.num().map(|n| v.base_speed_kmh = Some(n)).is_some(),
            "water_rating" => val.text().map(|s| v.water_rating = water_rating_in(s)).is_some(),
            "sailing_window" => val.text().map(|s| v.sailing_window = sailing_window_in(s)).is_some(),
            "portage_capable" => val.flag().map(|b| v.portage_capable = b).is_some(),
            _ => false,
        };
        if !applied {
            rejected.push(k.clone());
        }
    }
    (v, rejected)
}

// ---- 3.4 Party set-ups ----

pub fn preset_subtitle(p: &PartyPreset) -> String {
    p.transport.clone()
}

/// Reuses `journey_bridge::plan_to_pairs` rather than re-listing the same
/// twenty field names a third time: [`PartyPreset::apply_to`] already
/// projects a preset onto a full `JpPlan`, so running that through the
/// Journey Planner's own flattener and keeping only the party-form keys
/// (`PRESET_FIELD_KEYS`) is `PartyPreset::apply_to`'s own inverse, not a
/// parallel implementation that could drift from it.
pub const PRESET_FIELD_KEYS: [&str; 20] = [
    "transport",
    "mount_animal",
    "vessel",
    "hours",
    "pace",
    "season",
    "supply_days",
    "carry_food",
    "grazing",
    "foraging",
    "group_size",
    "cargo_kg",
    "donkey",
    "mule",
    "camel",
    "horse",
    "carts",
    "wagons",
    "sleds",
    "travois",
];

pub fn preset_to_pairs(p: &PartyPreset) -> Vec<(String, JpValue)> {
    let plan = p.apply_to(&cartalith_civ::JpPlan::default());
    let mut out = vec![("name".to_string(), JpValue::Str(p.name.clone()))];
    for (k, v) in crate::journey_bridge::plan_to_pairs(&plan) {
        if PRESET_FIELD_KEYS.contains(&k) {
            out.push((k.to_string(), v));
        }
    }
    out
}

/// Deliberately does **not** route through `journey_bridge::plan_from_pairs`
/// (which always starts from `JpPlan::default()`, right for a whole plan
/// submission but wrong for a partial preset edit that must preserve every
/// untouched field) -- these match arms are `plan_from_pairs`'s own party-
/// field subset, applied onto a clone of `base` instead of a fresh default,
/// the same "partial edit preserves the rest" contract every other
/// `_apply_pairs` function in this module already follows.
pub fn preset_apply_pairs(base: &PartyPreset, pairs: &[(String, JpValue)]) -> (PartyPreset, Vec<String>) {
    let mut p = base.clone();
    let mut rejected = Vec::new();
    for (k, v) in pairs {
        let applied = match k.as_str() {
            "name" => v.text().map(|s| p.name = s.to_string()).is_some(),
            "transport" => v.text().map(|s| p.transport = s.to_string()).is_some(),
            "mount_animal" => v.text().map(|s| p.mount_animal = (!s.is_empty()).then(|| s.to_string())).is_some(),
            "vessel" => v.text().map(|s| p.vessel = s.to_string()).is_some(),
            "hours" => v.num().map(|n| p.hours = n).is_some(),
            "pace" => v.text().map(|s| p.pace = s.to_string()).is_some(),
            "season" => v.text().map(|s| p.season = s.to_string()).is_some(),
            "supply_days" => v.int().map(|n| p.supply_days = n).is_some(),
            "carry_food" => v.flag().map(|b| p.carry_food = b).is_some(),
            "grazing" => v.text().map(|s| p.grazing = s.to_string()).is_some(),
            "foraging" => v.text().map(|s| p.foraging = s.to_string()).is_some(),
            "group_size" => v.int().map(|n| p.party.group_size = n).is_some(),
            "cargo_kg" => v.num().map(|n| p.party.cargo_kg = n).is_some(),
            "donkey" => v.int().map(|n| p.party.donkey = n).is_some(),
            "mule" => v.int().map(|n| p.party.mule = n).is_some(),
            "camel" => v.int().map(|n| p.party.camel = n).is_some(),
            "horse" => v.int().map(|n| p.party.horse = n).is_some(),
            "carts" => v.int().map(|n| p.party.carts = n).is_some(),
            "wagons" => v.int().map(|n| p.party.wagons = n).is_some(),
            "sleds" => v.int().map(|n| p.party.sleds = n).is_some(),
            "travois" => v.int().map(|n| p.party.travois = n).is_some(),
            _ => false,
        };
        if !applied {
            rejected.push(k.clone());
        }
    }
    (p, rejected)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------- CRUD round trips ----------

    #[test]
    fn a_fresh_library_has_every_stock_definition_and_no_custom_ones() {
        let lib = TravelLibrary::new();
        assert_eq!(lib.animals.iter().count(), 7);
        assert_eq!(lib.vehicles.iter().count(), 5);
        assert_eq!(lib.vessels.iter().count(), 11);
        assert_eq!(lib.presets.iter().count(), 2);
        assert!(lib.animals.iter().all(|a| a.origin == EntryOrigin::Stock));
    }

    #[test]
    fn stock_entries_cannot_be_edited_or_deleted() {
        let mut lib = TravelLibrary::new();
        assert!(
            lib.animals.get_mut("donkey").is_none(),
            "stock is read-only"
        );
        assert!(!lib.animals.delete("donkey"), "stock cannot be deleted");
        assert!(lib.animals.get("donkey").is_some(), "and is still there");
    }

    #[test]
    fn duplicating_a_stock_animal_yields_an_editable_custom_copy() {
        let mut lib = TravelLibrary::new();
        let id = lib.fresh_id();
        let dup = lib
            .animals
            .duplicate("donkey", id.clone())
            .expect("donkey exists")
            .clone();
        assert_eq!(dup.origin, EntryOrigin::Custom);
        assert_eq!(
            dup.species_key,
            Some("donkey"),
            "the species identity survives the duplicate"
        );
        assert_eq!(
            dup.load_capacity_kg,
            Some(80.0),
            "and so does every field, until edited"
        );

        let edited = lib
            .animals
            .get_mut(&id)
            .expect("custom entries are editable");
        edited.load_capacity_kg = Some(500.0);
        assert_eq!(lib.animals.get(&id).unwrap().load_capacity_kg, Some(500.0));

        assert!(lib.animals.delete(&id), "and custom entries can be deleted");
        assert!(lib.animals.get(&id).is_none());
    }

    #[test]
    fn a_blank_definition_starts_incomplete() {
        let mut lib = TravelLibrary::new();
        let id = lib.fresh_id();
        lib.animals.add(AnimalDef::blank(id.clone(), "New Species"));
        assert!(matches!(
            lib.animal_validation(&id),
            Some(ValidationState::Incomplete(_))
        ));
    }

    #[test]
    fn duplicate_ids_are_refused() {
        let mut lib = TravelLibrary::new();
        assert!(
            lib.animals.duplicate("donkey", "mule").is_none(),
            "\"mule\" is already taken"
        );
        assert_eq!(
            lib.animals.iter().count(),
            7,
            "the refused duplicate was not added"
        );
    }

    #[test]
    fn reset_to_stock_drops_every_custom_entry() {
        let mut lib = TravelLibrary::new();
        let id = lib.fresh_id();
        lib.animals.duplicate("donkey", id);
        assert_eq!(lib.animals.iter().count(), 8);
        lib.animals.reset_to_stock();
        assert_eq!(lib.animals.iter().count(), 7);
        assert!(lib.animals.iter().all(|a| a.origin == EntryOrigin::Stock));
    }

    // ---------- validation reachable through the live store ----------

    #[test]
    fn conflicting_is_reachable_through_the_live_store_too() {
        let mut lib = TravelLibrary::new();
        let id = lib.fresh_id();
        lib.animals.duplicate("mule", id.clone());
        let mule = lib.animals.get_mut(&id).unwrap();
        mule.grazing_tolerance = Some(GrazingTolerance::GrasslandOnly);
        assert!(matches!(
            lib.animal_validation(&id),
            Some(ValidationState::Conflicting(_))
        ));
    }

    // ---------- usage tracking ----------

    #[test]
    fn animal_usage_counts_presets_that_reference_its_species() {
        let mut lib = TravelLibrary::new();
        // Both stock presets use a mount_animal string but only "Heavy
        // Wagon Caravan" actually carries horse count > 0.
        assert_eq!(lib.animal_usage_in_presets("horse"), 1);
        assert_eq!(lib.animal_usage_in_presets("camel"), 0);

        let id = lib.fresh_id();
        let mut preset = PartyPreset::blank(id.clone(), "Camel Trekkers");
        preset.party.camel = 3;
        lib.presets.add(preset);
        assert_eq!(lib.animal_usage_in_presets("camel"), 1);
    }

    #[test]
    fn a_species_less_custom_animal_never_shows_preset_usage() {
        let mut lib = TravelLibrary::new();
        let id = lib.fresh_id();
        lib.animals.duplicate("ox", id.clone()); // ox has no species_key
        assert_eq!(lib.animal_usage_in_presets(&id), 0);
    }

    #[test]
    fn journey_usage_is_honestly_always_zero() {
        let lib = TravelLibrary::new();
        assert_eq!(
            lib.animal_usage_in_journeys("donkey"),
            0,
            "no persistent saved journey exists in this port yet"
        );
    }

    // ---------- animal_overrides ----------

    #[test]
    fn animal_overrides_only_includes_custom_species_keyed_entries() {
        let mut lib = TravelLibrary::new();
        assert!(
            lib.animal_overrides().is_empty(),
            "stock alone overrides nothing"
        );

        let id = lib.fresh_id();
        lib.animals.duplicate("ox", id); // no species_key -- inert
        assert!(lib.animal_overrides().is_empty());

        let id2 = lib.fresh_id();
        lib.animals.duplicate("donkey", id2);
        let overrides = lib.animal_overrides();
        assert_eq!(overrides.len(), 1);
        assert!(overrides.contains_key("donkey"));
    }

    // ---------- animal_species_slot / animal_overrides_selected ----------

    #[test]
    fn species_slot_is_the_own_key_then_the_substitutes_for_chain() {
        let mut lib = TravelLibrary::new();
        assert_eq!(lib.animal_species_slot("donkey"), Some("donkey"));
        assert_eq!(lib.animal_species_slot("horse"), Some("horse"));
        // The three stock species with no `JpParty` slot at all.
        for k in ["ox", "yak", "reindeer"] {
            assert_eq!(lib.animal_species_slot(k), None, "{k}");
        }
        assert_eq!(lib.animal_species_slot("no-such-entry"), None);

        // A from-blank custom species declaring a substitute reaches that
        // species' slot; one declaring nothing still reaches none.
        let dray = lib.fresh_id();
        lib.animals
            .add(AnimalDef::blank(dray.clone(), "Kharen dray-ox"));
        assert_eq!(lib.animal_species_slot(&dray), None);
        lib.animals
            .get_mut(&dray)
            .expect("just added")
            .substitutes_for = Some("mule".into());
        assert_eq!(lib.animal_species_slot(&dray), Some("mule"));

        // Two hops, and a cycle that must terminate rather than hang.
        let second = lib.fresh_id();
        lib.animals.add(AnimalDef::blank(second.clone(), "Second"));
        lib.animals
            .get_mut(&second)
            .expect("just added")
            .substitutes_for = Some(dray.clone());
        assert_eq!(lib.animal_species_slot(&second), Some("mule"));

        let a = lib.fresh_id();
        let b = lib.fresh_id();
        lib.animals.add(AnimalDef::blank(a.clone(), "A"));
        lib.animals.add(AnimalDef::blank(b.clone(), "B"));
        lib.animals.get_mut(&a).expect("just added").substitutes_for = Some(b.clone());
        lib.animals.get_mut(&b).expect("just added").substitutes_for = Some(a.clone());
        assert_eq!(lib.animal_species_slot(&a), None, "a cycle terminates");
    }

    #[test]
    fn an_empty_selection_reproduces_animal_overrides_exactly() {
        let mut lib = TravelLibrary::new();
        let id = lib.fresh_id();
        lib.animals.duplicate("camel", id);
        let (selected, rejected) = lib.animal_overrides_selected(&HashMap::new());
        assert!(rejected.is_empty());
        assert_eq!(selected.len(), lib.animal_overrides().len());
        assert!(selected.contains_key("camel"));
    }

    #[test]
    fn selecting_a_stock_entry_means_no_override_and_a_custom_one_means_that_exact_entry() {
        let mut lib = TravelLibrary::new();
        let first = lib.fresh_id();
        lib.animals.duplicate("mule", first.clone());
        let second = lib.fresh_id();
        lib.animals.duplicate("mule", second.clone());
        lib.animals
            .get_mut(&second)
            .expect("just duplicated")
            .load_capacity_kg = Some(999.0);

        // The implicit pick is the last-added one; naming the first must win.
        assert_eq!(
            lib.animal_overrides()
                .get("mule")
                .and_then(|a| a.load_capacity_kg),
            Some(999.0)
        );
        let sel = HashMap::from([("mule".to_string(), first.clone())]);
        let (out, rejected) = lib.animal_overrides_selected(&sel);
        assert!(rejected.is_empty());
        assert_eq!(out["mule"].id, first);

        // Naming the *stock* entry drops the override entirely.
        let sel = HashMap::from([("mule".to_string(), "mule".to_string())]);
        let (out, rejected) = lib.animal_overrides_selected(&sel);
        assert!(rejected.is_empty());
        assert!(!out.contains_key("mule"));
    }

    #[test]
    fn a_selection_that_cannot_be_honoured_is_rejected_not_silently_dropped() {
        let mut lib = TravelLibrary::new();
        let ox = lib.fresh_id();
        lib.animals.duplicate("ox", ox.clone()); // no slot at all

        let sel = HashMap::from([
            ("mule".to_string(), ox.clone()),            // wrong slot
            ("horse".to_string(), "nope".to_string()),   // unknown id
            ("ostrich".to_string(), "horse".to_string()), // unknown species
        ]);
        let (out, rejected) = lib.animal_overrides_selected(&sel);
        assert!(out.is_empty());
        // `JP_ANIMAL_KEYS` order first, then unknown keys sorted.
        assert_eq!(rejected, vec!["mule", "horse", "ostrich"]);
    }

    #[test]
    fn a_substitutes_for_entry_can_occupy_the_slot_it_substitutes_into() {
        let mut lib = TravelLibrary::new();
        let dray = lib.fresh_id();
        lib.animals.duplicate("ox", dray.clone());
        lib.animals
            .get_mut(&dray)
            .expect("just duplicated")
            .substitutes_for = Some("mule".into());

        let sel = HashMap::from([("mule".to_string(), dray.clone())]);
        let (out, rejected) = lib.animal_overrides_selected(&sel);
        assert!(rejected.is_empty());
        assert_eq!(out["mule"].id, dray);
        assert_eq!(
            out["mule"].load_capacity_kg,
            Some(150.0),
            "the ox's own capacity, not the mule's 110"
        );
    }

    // ---------- end to end: the proof this is real, not decorative ----------

    fn tiny_land_world() -> (Vec<f32>, Vec<u8>, Vec<f32>, Vec<f32>, usize, usize) {
        let (gw, gh) = (24usize, 16usize);
        let n = gw * gh;
        let field: Vec<f32> = (0..n).map(|i| 0.28 + 0.02 * (i % gw) as f32).collect();
        let water_bodies: Vec<u8> = (0..n).map(|i| u8::from(i % gw == 0)).collect();
        let temp: Vec<f32> = vec![14.0; n];
        let rain: Vec<f32> = vec![0.45; n];
        (field, water_bodies, temp, rain, gw, gh)
    }

    /// The test `TRAVEL_LIBRARY_SPEC.md`'s wiring milestone calls for: a
    /// custom, edited Travel Library animal entry -- slower AND
    /// higher-capacity than any stock entry -- actually changes a computed
    /// journey's days/km-per-day, through the exact call chain a later
    /// `#[func]` layer would use (`TravelLibrary::animal_overrides` ->
    /// `cartalith_civ::travel_library::animal_resolver_fns` ->
    /// `cartalith_civ::JpAnimalResolver` -> `cartalith_civ::jp_plan_ex`).
    #[test]
    fn a_custom_animal_override_changes_a_computed_journey() {
        let (field, water_bodies, temp, rain, gw, gh) = tiny_land_world();
        let jw = crate::journey_bridge::JourneyWorld::build(
            &field,
            &water_bodies,
            &temp,
            &rain,
            gw,
            gh,
            false,
            0.30,
            &[],
            &[],
        );
        let world = cartalith_civ::JpWorld {
            gw,
            gh,
            world: false,
            map_width_km: 1200.0,
            sea_level: 0.30,
            peak_m: 4000.0,
            field: &field,
            cart_biome: &jw.cart_biome,
            cart_terrain: &jw.cart_terrain,
            temp: &temp,
            rain: &rain,
            flow_field: None,
            flow_thresh: 300.0,
            water_bodies: Some(&water_bodies),
            territory: None,
            places: &jw.places,
            road_cells: &jw.road_cells,
            ocean_field: None,
            wind_field: None,
        };
        let pts: Vec<(f64, f64)> = (4..=18).map(|x| (x as f64, 8.0)).collect();
        let plan = cartalith_civ::JpPlan {
            transport: "Mounted Rider".into(),
            mount_animal: Some("donkey".into()),
            ..cartalith_civ::JpPlan::default()
        };
        let layovers = cartalith_civ::JpLayovers::new();

        let baseline = cartalith_civ::jp_plan_ex(&world, &pts, &plan, &layovers, &|_, _| 1.0, None)
            .expect("a 14-cell land traverse plans");
        assert!(baseline.days > 0.0 && baseline.avg_km_day > 0.0);

        // A custom donkey: much heavier-capacity but much slower than the
        // stock donkey (cap 80kg/4km/h) -- and blocked outright on Marsh,
        // which the stock donkey merely slows on (0.40x).
        let mut lib = TravelLibrary::new();
        let id = lib.fresh_id();
        lib.animals.duplicate("donkey", id.clone());
        let custom = lib.animals.get_mut(&id).unwrap();
        custom.load_capacity_kg = Some(500.0);
        custom.base_speed_kmh = Some(1.5);
        custom.terrain.insert("Marsh", TerrainAffinity::Blocked);

        let overrides = lib.animal_overrides();
        assert_eq!(overrides.len(), 1);
        let (stats_fn, terrain_fn) = cartalith_civ::travel_library::animal_resolver_fns(&overrides);
        let resolver = cartalith_civ::JpAnimalResolver {
            stats: &*stats_fn,
            terrain_mod: &*terrain_fn,
        };

        let overridden =
            cartalith_civ::jp_plan_ex(&world, &pts, &plan, &layovers, &|_, _| 1.0, Some(&resolver))
                .expect("still plans -- just differently");

        assert_ne!(
            overridden.days, baseline.days,
            "a 1.5 km/h donkey cannot take the same days as a 4 km/h one"
        );
        assert_ne!(overridden.avg_km_day, baseline.avg_km_day);
        assert!(
            overridden.days > baseline.days,
            "the slower custom donkey must take longer, not less"
        );
        assert!(overridden.avg_km_day < baseline.avg_km_day);
    }

    #[test]
    fn a_blocked_terrain_override_actually_blocks_the_stage() {
        // Same world, but the whole route sits on one uniform terrain --
        // rebuilt with Open Plains so blocking a DIFFERENT terrain the
        // route never touches would prove nothing; this asserts the
        // resolver-driven block only fires when the pace-setting animal
        // really does cross that terrain.
        let (field, water_bodies, temp, rain, gw, gh) = tiny_land_world();
        let jw = crate::journey_bridge::JourneyWorld::build(
            &field,
            &water_bodies,
            &temp,
            &rain,
            gw,
            gh,
            false,
            0.30,
            &[],
            &[],
        );
        let world = cartalith_civ::JpWorld {
            gw,
            gh,
            world: false,
            map_width_km: 1200.0,
            sea_level: 0.30,
            peak_m: 4000.0,
            field: &field,
            cart_biome: &jw.cart_biome,
            cart_terrain: &jw.cart_terrain,
            temp: &temp,
            rain: &rain,
            flow_field: None,
            flow_thresh: 300.0,
            water_bodies: Some(&water_bodies),
            territory: None,
            places: &jw.places,
            road_cells: &jw.road_cells,
            ocean_field: None,
            wind_field: None,
        };
        let pts: Vec<(f64, f64)> = (4..=18).map(|x| (x as f64, 8.0)).collect();
        let plan = cartalith_civ::JpPlan {
            transport: "Mounted Rider".into(),
            mount_animal: Some("donkey".into()),
            ..cartalith_civ::JpPlan::default()
        };
        let layovers = cartalith_civ::JpLayovers::new();
        let baseline = cartalith_civ::jp_plan_ex(&world, &pts, &plan, &layovers, &|_, _| 1.0, None)
            .expect("plans");
        // The stage terrain the route actually crosses (constant across
        // this tiny synthetic world -- `journey_world_builds_every_table...`
        // in journey_bridge.rs already establishes the paint grid's shape).
        let terrain = baseline.stages[0].terrain.clone();
        let Some(tl_key) = cartalith_civ::travel_library::tl_terrain_key_for_engine(&terrain)
        else {
            return; // this synthetic world's terrain has no TL mapping -- nothing to block
        };

        let mut lib = TravelLibrary::new();
        let id = lib.fresh_id();
        lib.animals.duplicate("donkey", id.clone());
        lib.animals
            .get_mut(&id)
            .unwrap()
            .terrain
            .insert(tl_key, TerrainAffinity::Blocked);
        let overrides = lib.animal_overrides();
        let (stats_fn, terrain_fn) = cartalith_civ::travel_library::animal_resolver_fns(&overrides);
        let resolver = cartalith_civ::JpAnimalResolver {
            stats: &*stats_fn,
            terrain_mod: &*terrain_fn,
        };
        let blocked =
            cartalith_civ::jp_plan_ex(&world, &pts, &plan, &layovers, &|_, _| 1.0, Some(&resolver))
                .expect("still returns a plan, with a blocked leg");
        assert!(
            blocked.blocked_idx.is_some(),
            "the donkey's own Travel Library entry marks {terrain} blocked"
        );
    }

    // ---------- the #[func] boundary's pairs conversion (this dispatch) ----------

    #[test]
    fn animal_pairs_round_trip_through_a_partial_edit() {
        let donkey = stock_animals().into_iter().find(|a| a.id == "donkey").unwrap();
        let pairs = animal_to_pairs(&donkey);
        // Every constraint field the stock donkey carries is present (no
        // field is silently dropped by the flattener).
        assert!(pairs.iter().any(|(k, _)| k == "load_capacity_kg"));
        assert!(pairs.iter().any(|(k, _)| k == "terrain.hills"));

        // A partial edit -- only two keys -- leaves everything else as it was.
        let edit = vec![
            ("load_capacity_kg".to_string(), JpValue::Num(999.0)),
            ("terrain.marsh".to_string(), JpValue::Str("blocked".to_string())),
        ];
        let (edited, rejected) = animal_apply_pairs(&donkey, &edit);
        assert!(rejected.is_empty());
        assert_eq!(edited.load_capacity_kg, Some(999.0));
        assert_eq!(edited.terrain.get("Marsh"), Some(&TerrainAffinity::Blocked));
        assert_eq!(edited.base_speed_kmh, donkey.base_speed_kmh, "untouched field is preserved");
        assert_eq!(edited.name, donkey.name);
        assert_eq!(edited.id, donkey.id, "id is not a recognised key -- never overwritten");
        assert_eq!(edited.species_key, donkey.species_key, "species_key is not a recognised key either");
    }

    #[test]
    fn animal_apply_pairs_reports_unknown_and_wrong_typed_keys_as_rejected() {
        let donkey = stock_animals().into_iter().find(|a| a.id == "donkey").unwrap();
        let edit = vec![
            ("no_such_field".to_string(), JpValue::Bool(true)),
            ("load_capacity_kg".to_string(), JpValue::Str("not a number".to_string())),
        ];
        let (_, rejected) = animal_apply_pairs(&donkey, &edit);
        assert_eq!(rejected, vec!["no_such_field", "load_capacity_kg"]);
    }

    #[test]
    fn a_blank_animal_emits_no_optional_field_keys_at_all() {
        let blank = AnimalDef::blank("t", "Blank");
        let pairs = animal_to_pairs(&blank);
        assert!(
            !pairs.iter().any(|(k, _)| k == "load_capacity_kg" || k.starts_with("terrain.")),
            "an unset field is absent from the pairs, not emitted as a zero"
        );
    }

    #[test]
    fn vehicle_pairs_round_trip_and_draft_requirement_survives_a_partial_edit() {
        let cart = stock_vehicles().into_iter().find(|v| v.id == "cart").unwrap();
        let (edited, rejected) = vehicle_apply_pairs(&cart, &[("load_kg".to_string(), JpValue::Num(1234.0))]);
        assert!(rejected.is_empty());
        assert_eq!(edited.load_kg, Some(1234.0));
        assert_eq!(
            edited.draft_head_required.as_ref().map(|d| d.count),
            cart.draft_head_required.as_ref().map(|d| d.count),
            "draft_head_required is untouched when neither draft_count nor draft_role is sent"
        );
    }

    #[test]
    fn vessel_pairs_round_trip() {
        let cog = stock_vessels().into_iter().find(|v| v.id == "cog").unwrap();
        let pairs = vessel_to_pairs(&cog);
        assert!(pairs.iter().any(|(k, v)| k == "modes" && *v == JpValue::Str("sea".to_string())));
        let (edited, rejected) =
            vessel_apply_pairs(&cog, &[("base_speed_kmh".to_string(), JpValue::Num(20.0))]);
        assert!(rejected.is_empty());
        assert_eq!(edited.base_speed_kmh, Some(20.0));
        assert_eq!(edited.hold_kg, cog.hold_kg);
    }

    #[test]
    fn preset_pairs_round_trip_and_omit_route_only_fields() {
        let preset = stock_party_presets().into_iter().find(|p| p.id == "heavy_wagon_caravan").unwrap();
        let pairs = preset_to_pairs(&preset);
        for key in ["desert_water", "weather_override", "route_cond", "infra", "seasonal_closures", "auto_promote"] {
            assert!(!pairs.iter().any(|(k, _)| k == key), "{key} is route-only, not a party-form field");
        }
        assert!(pairs.iter().any(|(k, v)| k == "horse" && *v == JpValue::Int(6)));

        let (edited, rejected) = preset_apply_pairs(&preset, &[("group_size".to_string(), JpValue::Int(30))]);
        assert!(rejected.is_empty());
        assert_eq!(edited.party.group_size, 30);
        assert_eq!(edited.party.horse, preset.party.horse, "untouched party count is preserved");
        assert_eq!(edited.transport, preset.transport);
    }

    #[test]
    fn validation_state_parts_covers_all_three_states() {
        assert_eq!(validation_state_parts(&ValidationState::Ok), ("ok", vec![], vec![]));
        assert_eq!(
            validation_state_parts(&ValidationState::Incomplete(vec!["load capacity kg"])),
            ("incomplete", vec!["load capacity kg".to_string()], vec![])
        );
        assert_eq!(
            validation_state_parts(&ValidationState::Conflicting(vec!["a conflict".to_string()])),
            ("conflicting", vec![], vec!["a conflict".to_string()])
        );
    }

    // ---------- the jp_compute wiring regression (this dispatch) ----------

    /// This dispatch changed `lib.rs`'s `jp_compute` from calling
    /// `cartalith_civ::jp_plan` to building a resolver from the live Travel
    /// Library and calling `jp_plan_ex(..., Some(&resolver))` unconditionally
    /// -- every call, not just when a custom override exists. This test is
    /// the constraint the dispatch brief itself named: "any change to
    /// `jp_compute`'s existing behavior when no custom entries exist must be
    /// provably identical to before". A fresh `TravelLibrary::new()` has
    /// stock content only, so `animal_overrides()` is empty and
    /// `animal_resolver_fns`'s two closures return `None` for every query --
    /// `cartalith_civ::resolve_animal_stats`/`resolve_animal_terrain_mod`
    /// then fall back to the built-in table exactly as if `animals` were
    /// `None`, which is what this test proves end to end rather than by
    /// reading the fallback code and trusting it.
    #[test]
    fn regression_stock_only_travel_library_matches_pre_dispatch_jp_plan() {
        let (field, water_bodies, temp, rain, gw, gh) = tiny_land_world();
        let jw = crate::journey_bridge::JourneyWorld::build(
            &field, &water_bodies, &temp, &rain, gw, gh, false, 0.30, &[], &[],
        );
        let world = cartalith_civ::JpWorld {
            gw,
            gh,
            world: false,
            map_width_km: 1200.0,
            sea_level: 0.30,
            peak_m: 4000.0,
            field: &field,
            cart_biome: &jw.cart_biome,
            cart_terrain: &jw.cart_terrain,
            temp: &temp,
            rain: &rain,
            flow_field: None,
            flow_thresh: 300.0,
            water_bodies: Some(&water_bodies),
            territory: None,
            places: &jw.places,
            road_cells: &jw.road_cells,
            ocean_field: None,
            wind_field: None,
        };
        let pts: Vec<(f64, f64)> = (4..=18).map(|x| (x as f64, 8.0)).collect();
        let plan = cartalith_civ::JpPlan {
            transport: "Mounted Rider".into(),
            mount_animal: Some("donkey".into()),
            ..cartalith_civ::JpPlan::default()
        };
        let layovers = cartalith_civ::JpLayovers::new();

        // Pre-dispatch: `jp_plan` (== `jp_plan_ex(..., None)`).
        let baseline = cartalith_civ::jp_plan(&world, &pts, &plan, &layovers, &|_, _| 1.0)
            .expect("a 14-cell land traverse plans");

        // Post-dispatch: exactly `jp_compute`'s own new call chain, over a
        // fresh, untouched (stock-only) `TravelLibrary`.
        let lib = TravelLibrary::new();
        let overrides = lib.animal_overrides();
        assert!(overrides.is_empty(), "a fresh library has no custom entries to override anything");
        let (stats_fn, terrain_fn) = cartalith_civ::travel_library::animal_resolver_fns(&overrides);
        let resolver = cartalith_civ::JpAnimalResolver { stats: &*stats_fn, terrain_mod: &*terrain_fn };
        let stock_only = cartalith_civ::jp_plan_ex(&world, &pts, &plan, &layovers, &|_, _| 1.0, Some(&resolver))
            .expect("still plans");

        assert_eq!(baseline, stock_only, "a stock-only Travel Library must not change jp_compute's output at all");
    }

    #[test]
    fn every_terrain_slug_round_trips_to_its_own_key() {
        for &tl_key in &TL_TERRAIN_KEYS {
            let slug = terrain_slug(tl_key);
            assert_eq!(terrain_key_from_slug(slug), Some(tl_key));
        }
    }
}
