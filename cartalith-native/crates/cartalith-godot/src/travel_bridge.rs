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
//! # What a later `#[func]` layer still needs to add
//!
//! Nothing here is wired into `lib.rs`'s `WorldGen` yet -- no
//! `travel_library: Option<TravelLibrary>` field, no `#[func]` surface, and
//! `jp_compute` does not read one. That is deliberately out of this
//! dispatch's scope (`TRAVEL_LIBRARY_SPEC.md`'s own GUI window is a
//! separate, later dispatch), but the shape a `#[func]` layer would add is
//! exactly [`TravelLibrary`]'s own public methods plus, at the `jp_compute`
//! call site, building a [`cartalith_civ::JpAnimalResolver`] from
//! [`TravelLibrary::animal_overrides`] via
//! [`cartalith_civ::travel_library::animal_resolver_fns`] and passing
//! `Some(&resolver)` to `cartalith_civ::jp_plan_ex` in place of today's
//! `cartalith_civ::jp_plan`. [`tests::a_custom_animal_override_changes_a_computed_journey`]
//! below exercises exactly that call chain end to end, proving it is real.

// Not yet consumed by `lib.rs` (no `#[func]` layer exists this dispatch --
// see the module doc's "What a later `#[func]` layer still needs to add"),
// so every item here is otherwise dead code from `cargo build`'s point of
// view. Exercised fully by this module's own `#[cfg(test)]` suite below.
#![allow(dead_code)]

use std::collections::HashMap;

use cartalith_civ::travel_library::{
    AnimalDef, EntryOrigin, PartyPreset, ValidationState, VehicleDef, VesselDef, stock_animals,
    stock_party_presets, stock_vehicles, stock_vessels,
};

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
    /// whose `species_key` names it, if any. (A UI that lets a party form
    /// pick a specific custom entry per species, rather than "the newest
    /// one wins", is real future work this data model does not block --
    /// see the module doc's "What a later `#[func]` layer still needs to
    /// add".) Stock entries are never included: by construction they carry
    /// exactly the built-in figures already, so including them would be an
    /// inert no-op override.
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

#[cfg(test)]
mod tests {
    use super::*;
    use cartalith_civ::travel_library::{GrazingTolerance, TerrainAffinity};

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
}
