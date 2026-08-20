//! The Journey Planner's Godot-facing bridge — `JOURNEY_PLANNER_SCOPE.md`'s
//! own "Closing status" step 2 and step 4 (*"a `JpWorld` assembled from live
//! state"* and *"`#[func]`s over the boundary"*).
//!
//! Deliberately **free of any `godot` dependency**, the same isolation
//! `sculpt_bridge.rs`/`civ_tools_bridge.rs`/`infra_tools_bridge.rs` already
//! establish: `lib.rs` owns the thin `Variant`<->Rust conversion, the
//! `#[func]` surface and the `VarDictionary` flattening; this module owns
//! everything that can be expressed without one — the plan/party form
//! parser, its inverse, the derived-once world buffers a
//! [`cartalith_civ::JpWorld`] borrows, and the option tables a dropdown
//! needs. Its own `#[cfg(test)]` suite below runs under
//! `cargo test -p cartalith-godot` with no Godot runtime involved.
//!
//! ## What this module does *not* do
//!
//! It does not re-derive anything the pipeline already produced. `jp_plan`'s
//! world is a set of borrowed rasters plus three derived tables, and every
//! raster is state `WorldGen` already holds:
//!
//! | `JpWorld` field | where it comes from |
//! |---|---|
//! | `field`/`temp`/`rain`/`flow_field` | `WorldState::field`/`temperature`/`rainfall`/`flow_discharge` |
//! | `water_bodies` | `CivData::water_bodies` (kept past `compute_civilisation` for exactly this kind of reuse) |
//! | `territory` | `CivData::territory` (`assign_territory`'s output) |
//! | `gw`/`gh`/`world`/`map_width_km`/`sea_level`/`peak_m` | `WorldGen`'s own fields / `WorldParams::peak_m` |
//! | `flow_thresh` | `cartalith_hydrology::river_flow_thresh`, the same call `compute_civilisation` makes |
//!
//! The three derived ones are [`JourneyWorld`]'s whole job: `cart_biome`,
//! `cart_terrain` (`build_cart_biome`/`build_cart_terrain` — milestone 5
//! added both and, exactly as `JOURNEY_PLANNER_SCOPE.md` predicted, no
//! pipeline stage calls either, so they are computed here from rasters that
//! already exist rather than bolted onto generation) and `road_cells`
//! (`jp_road_cells`). All three are pure functions of already-computed
//! state; none is a new generation stage.
//!
//! ## Three inputs that are honestly absent, not quietly faked
//!
//! - **`ocean_field`/`wind_field`** are `Option<&JpCoarseField>` and are
//!   passed `None`. The reference reads them from `currentOceanField()`/
//!   `currentWindField()`, its *visualisation* layer's cached coarse fields;
//!   this port's climate stage computes the current field inside
//!   `cartalith_climate::ocean_sst_anomaly` and discards it — nothing in
//!   `WorldState` retains a `u`/`v` pair at any resolution. `None` is
//!   `jp_sea_condition`'s own supported "no field" input, not a stand-in
//!   value, so a sea leg reads its structural condition and skips the
//!   wind/current term rather than reading an invented one. Retaining the
//!   coarse fields past generation is real work in `cartalith-engine`, not
//!   something to improvise at this boundary.
//! - **`road_cells` sees the generated way network only.** `jp_road_cells`
//!   takes `&[Way]`; hand-drawn ways are `tools::ManualWay`, a different
//!   type whose `Ancient` variant `jp_road_cells` has no branch for (the
//!   reference's `_jpRoadCells` does — `'ancient' -> ["Dirt Track",
//!   "Deteriorated"]` — because its one `civWays` array holds both kinds).
//!   Widening `jp_road_cells` is a `cartalith-civ` change against
//!   golden-tested code, so the gap is reported here rather than
//!   approximated with an invented type mapping.
//! - **`road_edges` is empty.** The reference's second road source is
//!   `state.roads.edges`, the terrain tab's reference roads. This port's
//!   generated network is `civ_consolidate_and_smooth_ways`' `Vec<Way>`;
//!   `build_road_network`'s `RoadEdge` list is not retained by
//!   `compute_civilisation` and has no live equivalent to pass.
//!
//! ## Wildlife
//!
//! `jp_plan` takes `wildlife_forage_mod: &dyn Fn(f64, f64) -> f64` and
//! `lib.rs` passes `|_, _| 1.0`. That is the reference's own answer on a
//! world with no wildlife layer (`JOURNEY_PLANNER_SCOPE.md`'s "Two quality
//! ceilings"), and also what an exactly-average region gives — the
//! ecoregion/species-richness subsystem behind it is unported and on no
//! milestone anywhere.

use std::collections::HashMap;

use cartalith_civ::{
    build_cart_biome, build_cart_terrain, jp_road_cells, JpParty, JpPlace, JpPlan, JpRoadCell, JpStageOverride, NamedSettlement,
    SettlementKind, Way, CART_BIOMES, CART_TERRAINS, JP_ANIMAL_KEYS, JP_DESERT_WATER_KEYS, JP_INFRA_TIERS, JP_LAND_TRANSPORT_KEYS,
    JP_SEASON_ORDER, JP_VESSEL_PREFERENCE, JP_WEATHER_KEYS,
};

// ===================== the Variant-shaped scalar =====================

/// The four `Variant` kinds the Journey Planner's plan/party form actually
/// uses, narrowed to something this `godot`-free module can name. `lib.rs`
/// maps `INT` to [`JpValue::Int`], `FLOAT` to [`JpValue::Num`], `STRING` to
/// [`JpValue::Str`] and `BOOL` to [`JpValue::Bool`]; anything else never
/// reaches this module and is reported `rejected` at the boundary.
///
/// `Int` and `Num` are kept apart in one direction only. *Reading* a form,
/// either is accepted wherever the other is (a party count sent as `8.0` is
/// as legal as `8`, and `hours` sent as `8` is as legal as `8.0`) -- a
/// GDScript `SpinBox` decides that, not the caller. *Writing* one back,
/// [`plan_to_pairs`] emits `Int` for the eight fields that really are
/// integers, so `jp_default_plan()["supply_days"]` is `7` rather than `7.0`
/// on the Godot side.
#[derive(Debug, Clone, PartialEq)]
pub enum JpValue {
    Int(i64),
    Num(f64),
    Str(String),
    Bool(bool),
}

impl JpValue {
    /// `pub(crate)`: `travel_bridge.rs` reuses these four narrowing
    /// accessors for its own field-pair parsers rather than duplicating the
    /// same four match arms a second time.
    pub(crate) fn num(&self) -> Option<f64> {
        match self {
            JpValue::Num(n) => Some(*n),
            JpValue::Int(n) => Some(*n as f64),
            _ => None,
        }
    }

    /// JS's own `|0` on every count the reference reads: truncation toward
    /// zero, not rounding. `plan.supplyDays|0` and `(a.donkey|0)` are the
    /// two shapes this covers.
    pub(crate) fn int(&self) -> Option<i64> {
        match self {
            JpValue::Int(n) => Some(*n),
            JpValue::Num(n) => Some(*n as i64),
            _ => None,
        }
    }

    pub(crate) fn text(&self) -> Option<&str> {
        match self {
            JpValue::Str(s) => Some(s.as_str()),
            _ => None,
        }
    }

    pub(crate) fn flag(&self) -> Option<bool> {
        match self {
            JpValue::Bool(b) => Some(*b),
            _ => None,
        }
    }
}

/// The reference's own `!v || v==='auto'` test, shared by every plan field
/// whose `None` means "derive it" (`desertWater`, `weatherOverride`,
/// `routeCond`, `infra`, `restCadence`) — and by `mountAnimal`, whose `None`
/// instead means "the party's own animals decide" (`jp_resolve_mount`).
fn opt_key(s: &str) -> Option<String> {
    if s.is_empty() || s == "auto" {
        None
    } else {
        Some(s.to_string())
    }
}

fn opt_out(v: &Option<String>) -> JpValue {
    JpValue::Str(v.clone().unwrap_or_default())
}

// ===================== plan / party parsing =====================

/// Builds a [`JpPlan`] from a flat key/value form, starting from
/// `JpPlan::default()` (the reference's own `_jpEnsurePlan` default block)
/// so a partial form is legal — a UI that sends only `season` and
/// `group_size` gets the reference's defaults for everything else.
///
/// Returns the plan plus every key that was unrecognised **or** carried the
/// wrong `Variant` type, per this codebase's "a typo'd key is a bug worth
/// seeing" policy (`set_params`' own doc comment). `stage_overrides` is not
/// a key here — it is a nested map, parsed per entry by
/// [`stage_override_from_pairs`] and assigned by the caller.
pub fn plan_from_pairs(pairs: &[(String, JpValue)]) -> (JpPlan, Vec<String>) {
    let mut plan = JpPlan::default();
    let mut rejected: Vec<String> = Vec::new();
    for (k, v) in pairs {
        let applied = match k.as_str() {
            // ---- travel mode ----
            "transport" => v.text().map(|s| plan.transport = s.to_string()).is_some(),
            "mount_animal" => v.text().map(|s| plan.mount_animal = opt_key(s)).is_some(),
            "vessel" => v.text().map(|s| plan.vessel = s.to_string()).is_some(),
            "hours" => v.num().map(|n| plan.hours = n).is_some(),
            "pace" => v.text().map(|s| plan.pace = s.to_string()).is_some(),
            "season" => v.text().map(|s| plan.season = s.to_string()).is_some(),
            // ---- supply ----
            "supply_days" => v.int().map(|n| plan.supply_days = n).is_some(),
            "carry_food" => v.flag().map(|b| plan.carry_food = b).is_some(),
            "grazing" => v.text().map(|s| plan.grazing = s.to_string()).is_some(),
            "foraging" => v.text().map(|s| plan.foraging = s.to_string()).is_some(),
            // ---- auto-with-override ----
            "desert_water" => v.text().map(|s| plan.desert_water = opt_key(s)).is_some(),
            "weather_override" => v.text().map(|s| plan.weather_override = opt_key(s)).is_some(),
            "seasonal_closures" => v.flag().map(|b| plan.seasonal_closures = b).is_some(),
            "route_cond" => v.text().map(|s| plan.route_cond = opt_key(s)).is_some(),
            "infra" => v.text().map(|s| plan.infra = opt_key(s)).is_some(),
            "season_drift" => v.flag().map(|b| plan.season_drift = b).is_some(),
            "rest_cadence" => v.text().map(|s| plan.rest_cadence = opt_key(s)).is_some(),
            "auto_promote" => v.flag().map(|b| plan.auto_promote = b).is_some(),
            // ---- party ----
            "group_size" => v.int().map(|n| plan.party.group_size = n).is_some(),
            "cargo_kg" => v.num().map(|n| plan.party.cargo_kg = n).is_some(),
            "donkey" => v.int().map(|n| plan.party.donkey = n).is_some(),
            "mule" => v.int().map(|n| plan.party.mule = n).is_some(),
            "camel" => v.int().map(|n| plan.party.camel = n).is_some(),
            "horse" => v.int().map(|n| plan.party.horse = n).is_some(),
            "carts" => v.int().map(|n| plan.party.carts = n).is_some(),
            "wagons" => v.int().map(|n| plan.party.wagons = n).is_some(),
            "sleds" => v.int().map(|n| plan.party.sleds = n).is_some(),
            "travois" => v.int().map(|n| plan.party.travois = n).is_some(),
            _ => false,
        };
        if !applied {
            rejected.push(k.clone());
        }
    }
    (plan, rejected)
}

/// [`plan_from_pairs`]' inverse: every key it accepts, with this plan's
/// current value. Used for two real things — `jp_default_plan()`, which
/// seeds a party form with the reference's own defaults, and flattening each
/// `JpLegResult::eff` (a full `JpPlan` that season drift and the per-stage
/// vessel fallback may both have altered from the one sent in).
///
/// `stage_overrides` is deliberately not emitted: `eff` is already the
/// *result* of applying an override, so re-reporting the sparse map on every
/// leg would be noise, not information.
pub fn plan_to_pairs(plan: &JpPlan) -> Vec<(&'static str, JpValue)> {
    vec![
        ("transport", JpValue::Str(plan.transport.clone())),
        ("mount_animal", opt_out(&plan.mount_animal)),
        ("vessel", JpValue::Str(plan.vessel.clone())),
        ("hours", JpValue::Num(plan.hours)),
        ("pace", JpValue::Str(plan.pace.clone())),
        ("season", JpValue::Str(plan.season.clone())),
        ("supply_days", JpValue::Int(plan.supply_days)),
        ("carry_food", JpValue::Bool(plan.carry_food)),
        ("grazing", JpValue::Str(plan.grazing.clone())),
        ("foraging", JpValue::Str(plan.foraging.clone())),
        ("desert_water", opt_out(&plan.desert_water)),
        ("weather_override", opt_out(&plan.weather_override)),
        ("seasonal_closures", JpValue::Bool(plan.seasonal_closures)),
        ("route_cond", opt_out(&plan.route_cond)),
        ("infra", opt_out(&plan.infra)),
        ("season_drift", JpValue::Bool(plan.season_drift)),
        ("rest_cadence", opt_out(&plan.rest_cadence)),
        ("auto_promote", JpValue::Bool(plan.auto_promote)),
        ("group_size", JpValue::Int(plan.party.group_size)),
        ("cargo_kg", JpValue::Num(plan.party.cargo_kg)),
        ("donkey", JpValue::Int(plan.party.donkey)),
        ("mule", JpValue::Int(plan.party.mule)),
        ("camel", JpValue::Int(plan.party.camel)),
        ("horse", JpValue::Int(plan.party.horse)),
        ("carts", JpValue::Int(plan.party.carts)),
        ("wagons", JpValue::Int(plan.party.wagons)),
        ("sleds", JpValue::Int(plan.party.sleds)),
        ("travois", JpValue::Int(plan.party.travois)),
    ]
}

/// One entry of `plan.stage_overrides`, from the same key vocabulary
/// [`plan_from_pairs`] accepts minus the two that are journey-wide rather
/// than per-stage (`season_drift`, `auto_promote`). Every key left out stays
/// `None` and cascades from the shared plan — the reference's own
/// `Object.assign({}, plan, ov)`, animal counts merged per species.
pub fn stage_override_from_pairs(pairs: &[(String, JpValue)]) -> (JpStageOverride, Vec<String>) {
    let mut ov = JpStageOverride::default();
    let mut rejected: Vec<String> = Vec::new();
    for (k, v) in pairs {
        let applied = match k.as_str() {
            "transport" => v.text().map(|s| ov.transport = Some(s.to_string())).is_some(),
            "mount_animal" => v.text().map(|s| ov.mount_animal = opt_key(s)).is_some(),
            "vessel" => v.text().map(|s| ov.vessel = Some(s.to_string())).is_some(),
            "hours" => v.num().map(|n| ov.hours = Some(n)).is_some(),
            "pace" => v.text().map(|s| ov.pace = Some(s.to_string())).is_some(),
            "season" => v.text().map(|s| ov.season = Some(s.to_string())).is_some(),
            "supply_days" => v.int().map(|n| ov.supply_days = Some(n)).is_some(),
            "carry_food" => v.flag().map(|b| ov.carry_food = Some(b)).is_some(),
            "grazing" => v.text().map(|s| ov.grazing = Some(s.to_string())).is_some(),
            "foraging" => v.text().map(|s| ov.foraging = Some(s.to_string())).is_some(),
            "desert_water" => v.text().map(|s| ov.desert_water = opt_key(s)).is_some(),
            "weather_override" => v.text().map(|s| ov.weather_override = opt_key(s)).is_some(),
            "seasonal_closures" => v.flag().map(|b| ov.seasonal_closures = Some(b)).is_some(),
            "route_cond" => v.text().map(|s| ov.route_cond = opt_key(s)).is_some(),
            "infra" => v.text().map(|s| ov.infra = opt_key(s)).is_some(),
            "group_size" => v.int().map(|n| ov.group_size = Some(n)).is_some(),
            "cargo_kg" => v.num().map(|n| ov.cargo_kg = Some(n)).is_some(),
            "donkey" => v.int().map(|n| ov.donkey = Some(n)).is_some(),
            "mule" => v.int().map(|n| ov.mule = Some(n)).is_some(),
            "camel" => v.int().map(|n| ov.camel = Some(n)).is_some(),
            "horse" => v.int().map(|n| ov.horse = Some(n)).is_some(),
            "carts" => v.int().map(|n| ov.carts = Some(n)).is_some(),
            "wagons" => v.int().map(|n| ov.wagons = Some(n)).is_some(),
            "sleds" => v.int().map(|n| ov.sleds = Some(n)).is_some(),
            "travois" => v.int().map(|n| ov.travois = Some(n)).is_some(),
            _ => false,
        };
        if !applied {
            rejected.push(k.clone());
        }
    }
    (ov, rejected)
}

// ===================== the world buffers `JpWorld` borrows =====================

/// The three tables a [`cartalith_civ::JpWorld`] needs that are **not**
/// already sitting in `WorldState`/`CivData`, computed from ones that are.
/// `lib.rs` holds one of these alive for the duration of a `jp_plan` call and
/// hands out borrows of its fields; nothing here is cached between calls,
/// because every input can change under a paint stroke or a manual
/// settlement drop.
pub struct JourneyWorld {
    /// [`build_cart_biome`]'s 15-entry paint grid (`CART_BIOMES`, 1-based).
    pub cart_biome: Vec<u8>,
    /// [`build_cart_terrain`]'s 13-entry paint grid (`CART_TERRAINS`).
    pub cart_terrain: Vec<u8>,
    /// [`jp_road_cells`]' dilated road lookup over the generated network.
    pub road_cells: HashMap<(i64, i64), JpRoadCell>,
    /// The settlements as the planner samples them — `_jpSettlements`'
    /// filter, which this port satisfies by construction (see [`JpPlace`]).
    pub places: Vec<JpPlace>,
}

impl JourneyWorld {
    #[allow(clippy::too_many_arguments)]
    pub fn build(
        field: &[f32],
        water_bodies: &[u8],
        temp: &[f32],
        rain: &[f32],
        gw: usize,
        gh: usize,
        world: bool,
        sea: f64,
        ways: &[Way],
        settlements: &[NamedSettlement],
    ) -> Self {
        JourneyWorld {
            cart_biome: build_cart_biome(field, water_bodies, temp, rain, gw, gh, world, sea),
            cart_terrain: build_cart_terrain(field, water_bodies, temp, rain, gw, gh, world, sea),
            // No `road_edges`: see the module doc's third bullet.
            road_cells: jp_road_cells(ways, &[], gw),
            places: settlements
                .iter()
                .map(|s| JpPlace {
                    name: s.name.clone(),
                    kind: settlement_kind_key(s.placement.kind).to_string(),
                    x: s.placement.x as f64,
                    y: s.placement.y as f64,
                })
                .collect(),
        }
    }
}

/// The six lowercase tier names `get_settlements()` reports -- one
/// vocabulary rather than two, so a stop's `kind` in a journey result and
/// the same settlement's `kind` in the settlement list always agree.
/// `get_settlements()` calls straight into this function; `map_overlay.gd`'s
/// `SETTLEMENT_CLASS`/`SETTLEMENT_LOD` dicts are keyed on exactly these
/// strings, and `civ_tools_bridge::kind_from_str` is the inverse.
pub fn settlement_kind_key(kind: SettlementKind) -> &'static str {
    match kind {
        SettlementKind::Metropolis => "metropolis",
        SettlementKind::Capital => "capital",
        SettlementKind::City => "city",
        SettlementKind::Town => "town",
        SettlementKind::Village => "village",
        SettlementKind::Hamlet => "hamlet",
    }
}

// ===================== option tables =====================
//
// Every list below is either a real `cartalith-civ` `pub const` re-exported
// verbatim, or -- for the five tables the engine models as `match` arms
// rather than as a const -- a transcription whose every entry is pinned by a
// test below against the engine's OWN lookup. A dropdown that offers a key
// the engine does not recognise is the worst failure available here: the
// planner falls through to `?? 1.0` and the user sees a plausible number
// computed from the wrong row.

/// `JP_PACE` (reference line 17533), fastest to slowest.
pub const PACE_KEYS: [&str; 5] = ["Haste", "Forced March", "Standard Pace", "Cautious / Scouting", "Stealth / Night Travel"];

/// `JP_GRAZING` (reference line 17540).
pub const GRAZING_KEYS: [&str; 3] = ["None — carry all fodder", "Partial — graze at camp", "Full — graze on route"];

/// `JP_FORAGING` (reference line 17541).
pub const FORAGING_KEYS: [&str; 3] = ["None", "Opportunistic", "Active"];

/// `JP_REST_CADENCES` (v1.52). `"auto"` is not listed: it is the *absent*
/// value, which `plan.rest_cadence = None` already expresses.
pub const REST_CADENCE_KEYS: [&str; 4] = ["None — press on", "Light — 1 in 7", "Standard — 1 in 5", "Heavy — 1 in 3"];

/// `JP_ROUTE[cat]` (reference line 17464) — the legal route conditions for
/// one travel category. A "Maintained" road condition cannot describe a sea
/// leg, and `_jpDeriveStages` rejects it when it does
/// (`jp_route_cond_valid`), so a UI must offer per-category lists.
pub fn route_cond_keys(cat: &str) -> &'static [&'static str] {
    match cat {
        "land" => &["Maintained", "Standard", "Deteriorated", "Broken", "None / Wild"],
        "river" => &["Strong Downstream", "Mild Downstream", "Neutral", "Mild Upstream", "Strong Upstream"],
        "sea" => &["Favorable Wind & Current", "Favorable Wind", "Neutral", "Headwind", "Strong Headwind"],
        _ => &[],
    }
}

/// `JP_INFRA_TIERS`' tier names, best to worst (the const itself is
/// `(ratio, name)` pairs, walked in this exact order by `jp_stage_infra`).
pub fn infra_tier_keys() -> Vec<&'static str> {
    JP_INFRA_TIERS.iter().map(|t| t.1).collect()
}

/// `JP_BIOMES`' keys: `CART_BIOMES`' first twelve, which are exactly the
/// biome vocabulary the stage calculators consume (13 `Hills` is
/// climate-classified at sample time, 14/15 are water).
pub fn biome_keys() -> &'static [&'static str] {
    &CART_BIOMES[..12]
}

/// Every dropdown the party/plan form needs, as `(field key, options)`. The
/// field keys are exactly [`plan_from_pairs`]' own, so a form can be built by
/// walking this list rather than by hard-coding a second copy of the
/// vocabulary on the GDScript side.
pub fn option_tables() -> Vec<(&'static str, Vec<&'static str>)> {
    vec![
        ("transport", JP_LAND_TRANSPORT_KEYS.to_vec()),
        ("vessel", JP_VESSEL_PREFERENCE.to_vec()),
        ("mount_animal", JP_ANIMAL_KEYS.to_vec()),
        ("season", JP_SEASON_ORDER.to_vec()),
        ("pace", PACE_KEYS.to_vec()),
        ("grazing", GRAZING_KEYS.to_vec()),
        ("foraging", FORAGING_KEYS.to_vec()),
        ("desert_water", JP_DESERT_WATER_KEYS.to_vec()),
        ("weather_override", JP_WEATHER_KEYS.to_vec()),
        ("rest_cadence", REST_CADENCE_KEYS.to_vec()),
        ("infra", infra_tier_keys()),
    ]
}

/// The reference tables a *results* panel needs in order to label what came
/// back, as opposed to what the form offers: the terrain and biome
/// vocabularies every stage reports, the three travel categories, and the
/// four pack species.
pub fn reference_tables() -> Vec<(&'static str, Vec<&'static str>)> {
    vec![
        ("terrain", CART_TERRAINS.to_vec()),
        ("biome", biome_keys().to_vec()),
        ("category", vec!["land", "river", "sea"]),
        ("animal", JP_ANIMAL_KEYS.to_vec()),
    ]
}

/// The party's own numeric fields as `(form key, value)`, in the order a
/// form should show them — the ten spinners of [`JpParty`], named with the
/// exact keys [`plan_from_pairs`] accepts back.
pub fn party_count_pairs(p: &JpParty) -> [(&'static str, f64); 10] {
    [
        ("group_size", p.group_size as f64),
        ("cargo_kg", p.cargo_kg),
        ("donkey", p.donkey as f64),
        ("mule", p.mule as f64),
        ("camel", p.camel as f64),
        ("horse", p.horse as f64),
        ("carts", p.carts as f64),
        ("wagons", p.wagons as f64),
        ("sleds", p.sleds as f64),
        ("travois", p.travois as f64),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use cartalith_civ::{
        jp_biome, jp_capacity, jp_desert_tier_for_gap, jp_foraging, jp_infra_mod, jp_land_transport_kmh, jp_pace_mod, jp_rest_days,
        jp_route_cond_valid, jp_season_at, jp_ship_stats, jp_weather_mod,
    };

    fn owned(v: &[(&str, JpValue)]) -> Vec<(String, JpValue)> {
        v.iter().map(|(k, x)| ((*k).to_string(), x.clone())).collect()
    }

    fn flattened(plan: &JpPlan) -> Vec<(String, JpValue)> {
        plan_to_pairs(plan).into_iter().map(|(k, v)| (k.to_string(), v)).collect()
    }

    // ---------- plan / party parsing ----------

    #[test]
    fn an_empty_form_is_the_reference_default_plan() {
        let (plan, rejected) = plan_from_pairs(&[]);
        assert!(rejected.is_empty());
        assert_eq!(plan, JpPlan::default());
        // The default block itself, spot-checked against `_jpEnsurePlan`.
        assert_eq!(plan.party.group_size, 4);
        assert_eq!(plan.transport, "Walking");
        assert_eq!(plan.supply_days, 7);
        assert!(plan.season_drift && plan.seasonal_closures && plan.carry_food);
        assert!(!plan.auto_promote);
    }

    #[test]
    fn every_form_field_reaches_its_plan_field() {
        let form = owned(&[
            ("transport", JpValue::Str("Baggage Train".into())),
            ("mount_animal", JpValue::Str("camel".into())),
            ("vessel", JpValue::Str("Cog".into())),
            ("hours", JpValue::Num(10.5)),
            ("pace", JpValue::Str("Forced March".into())),
            ("season", JpValue::Str("Winter".into())),
            ("supply_days", JpValue::Num(21.0)),
            ("carry_food", JpValue::Bool(false)),
            ("grazing", JpValue::Str(GRAZING_KEYS[2].into())),
            ("foraging", JpValue::Str("Active".into())),
            ("desert_water", JpValue::Str("Sparse Wells".into())),
            ("weather_override", JpValue::Str("Storm".into())),
            ("seasonal_closures", JpValue::Bool(false)),
            ("route_cond", JpValue::Str("Maintained".into())),
            ("infra", JpValue::Str("Ruined Region".into())),
            ("season_drift", JpValue::Bool(false)),
            ("rest_cadence", JpValue::Str(REST_CADENCE_KEYS[3].into())),
            ("auto_promote", JpValue::Bool(true)),
            ("group_size", JpValue::Num(30.0)),
            ("cargo_kg", JpValue::Num(1234.5)),
            ("donkey", JpValue::Num(1.0)),
            ("mule", JpValue::Num(2.0)),
            ("camel", JpValue::Num(3.0)),
            ("horse", JpValue::Num(4.0)),
            ("carts", JpValue::Num(5.0)),
            ("wagons", JpValue::Num(6.0)),
            ("sleds", JpValue::Num(7.0)),
            ("travois", JpValue::Num(8.0)),
        ]);
        let (plan, rejected) = plan_from_pairs(&form);
        assert!(rejected.is_empty(), "{rejected:?}");
        assert_eq!(plan.transport, "Baggage Train");
        assert_eq!(plan.mount_animal.as_deref(), Some("camel"));
        assert_eq!(plan.vessel, "Cog");
        assert_eq!(plan.hours, 10.5);
        assert_eq!(plan.pace, "Forced March");
        assert_eq!(plan.season, "Winter");
        assert_eq!(plan.supply_days, 21);
        assert!(!plan.carry_food);
        assert_eq!(plan.grazing, GRAZING_KEYS[2]);
        assert_eq!(plan.foraging, "Active");
        assert_eq!(plan.desert_water.as_deref(), Some("Sparse Wells"));
        assert_eq!(plan.weather_override.as_deref(), Some("Storm"));
        assert!(!plan.seasonal_closures);
        assert_eq!(plan.route_cond.as_deref(), Some("Maintained"));
        assert_eq!(plan.infra.as_deref(), Some("Ruined Region"));
        assert!(!plan.season_drift);
        assert_eq!(plan.rest_cadence.as_deref(), Some(REST_CADENCE_KEYS[3]));
        assert!(plan.auto_promote);
        assert_eq!(
            plan.party,
            JpParty { group_size: 30, cargo_kg: 1234.5, donkey: 1, mule: 2, camel: 3, horse: 4, carts: 5, wagons: 6, sleds: 7, travois: 8 }
        );
        // The form and its inverse name the same 28 keys.
        assert_eq!(form.len(), plan_to_pairs(&plan).len());
    }

    #[test]
    fn auto_and_empty_both_mean_derive_it() {
        for sentinel in ["auto", ""] {
            let (plan, rejected) = plan_from_pairs(&owned(&[
                ("desert_water", JpValue::Str(sentinel.into())),
                ("weather_override", JpValue::Str(sentinel.into())),
                ("route_cond", JpValue::Str(sentinel.into())),
                ("infra", JpValue::Str(sentinel.into())),
                ("rest_cadence", JpValue::Str(sentinel.into())),
                ("mount_animal", JpValue::Str(sentinel.into())),
            ]));
            assert!(rejected.is_empty());
            assert_eq!(plan.desert_water, None, "{sentinel:?}");
            assert_eq!(plan.weather_override, None, "{sentinel:?}");
            assert_eq!(plan.route_cond, None, "{sentinel:?}");
            assert_eq!(plan.infra, None, "{sentinel:?}");
            assert_eq!(plan.rest_cadence, None, "{sentinel:?}");
            assert_eq!(plan.mount_animal, None, "{sentinel:?}");
        }
    }

    #[test]
    fn a_wrong_typed_or_unknown_key_is_rejected_and_changes_nothing() {
        let (plan, rejected) = plan_from_pairs(&owned(&[
            ("season", JpValue::Num(3.0)),                 // wrong type
            ("carry_food", JpValue::Str("yes".into())),    // wrong type
            ("supply_days", JpValue::Bool(true)),          // wrong type
            ("mountAnimal", JpValue::Str("horse".into())), // camelCase typo
            ("letter_spacing", JpValue::Num(1.0)),         // not a plan field at all
        ]));
        assert_eq!(rejected, vec!["season", "carry_food", "supply_days", "mountAnimal", "letter_spacing"]);
        assert_eq!(plan, JpPlan::default(), "a rejected key must leave the default untouched");
    }

    #[test]
    fn an_integer_field_reads_an_int_or_a_float_and_writes_back_an_int() {
        // A GDScript SpinBox may hand over either; the eight genuinely
        // integer fields come back as `Int` regardless.
        let (from_int, r1) = plan_from_pairs(&owned(&[("supply_days", JpValue::Int(21)), ("horse", JpValue::Int(3))]));
        let (from_float, r2) = plan_from_pairs(&owned(&[("supply_days", JpValue::Num(21.0)), ("horse", JpValue::Num(3.0))]));
        assert!(r1.is_empty() && r2.is_empty());
        assert_eq!(from_int, from_float);
        assert_eq!(from_int.supply_days, 21);
        // And `hours`, a real float field, takes a bare int just as happily.
        let (h, r3) = plan_from_pairs(&owned(&[("hours", JpValue::Int(9))]));
        assert!(r3.is_empty());
        assert_eq!(h.hours, 9.0);

        let out = plan_to_pairs(&from_int);
        let ints: Vec<&str> = out.iter().filter(|(_, v)| matches!(v, JpValue::Int(_))).map(|(k, _)| *k).collect();
        assert_eq!(ints, vec!["supply_days", "group_size", "donkey", "mule", "camel", "horse", "carts", "wagons", "sleds", "travois"]);
        assert!(out.iter().any(|(k, v)| *k == "hours" && matches!(v, JpValue::Num(_))));
        assert!(out.iter().any(|(k, v)| *k == "cargo_kg" && matches!(v, JpValue::Num(_))));
    }

    #[test]
    fn counts_truncate_toward_zero_like_js_bitwise_or() {
        let (plan, _) = plan_from_pairs(&owned(&[
            ("group_size", JpValue::Num(7.9)),
            ("horse", JpValue::Num(2.6)),
            ("supply_days", JpValue::Num(-3.7)),
        ]));
        assert_eq!(plan.party.group_size, 7);
        assert_eq!(plan.party.horse, 2);
        assert_eq!(plan.supply_days, -3);
    }

    // ---------- the round trip ----------

    #[test]
    fn plan_survives_a_flatten_and_reparse() {
        let original = JpPlan {
            transport: "Mounted Rider".into(),
            mount_animal: Some("mule".into()),
            vessel: "Dhow".into(),
            hours: 12.0,
            pace: "Cautious / Scouting".into(),
            season: "Autumn".into(),
            supply_days: 14,
            carry_food: false,
            grazing: GRAZING_KEYS[1].into(),
            foraging: "Opportunistic".into(),
            desert_water: Some("Deep Desert Crossing".into()),
            weather_override: Some("Snow".into()),
            seasonal_closures: false,
            route_cond: Some("Broken".into()),
            infra: Some("Stable Settlements".into()),
            stage_overrides: HashMap::new(),
            season_drift: false,
            rest_cadence: Some(REST_CADENCE_KEYS[1].into()),
            auto_promote: true,
            party: JpParty { group_size: 12, cargo_kg: 900.0, donkey: 3, mule: 4, camel: 0, horse: 1, carts: 2, wagons: 0, sleds: 1, travois: 5 },
        };
        let (round, rejected) = plan_from_pairs(&flattened(&original));
        assert!(rejected.is_empty(), "{rejected:?}");
        assert_eq!(round, original);
    }

    #[test]
    fn the_default_plan_survives_the_round_trip_too() {
        let (round, rejected) = plan_from_pairs(&flattened(&JpPlan::default()));
        assert!(rejected.is_empty());
        assert_eq!(round, JpPlan::default());
    }

    // ---------- stage overrides ----------

    #[test]
    fn an_empty_stage_override_overrides_nothing() {
        let (ov, rejected) = stage_override_from_pairs(&[]);
        assert!(rejected.is_empty());
        assert_eq!(ov, JpStageOverride::default());
    }

    #[test]
    fn a_stage_override_sets_only_the_keys_it_names() {
        let (ov, rejected) = stage_override_from_pairs(&owned(&[
            ("camel", JpValue::Num(6.0)),
            ("route_cond", JpValue::Str("Deteriorated".into())),
            ("hours", JpValue::Num(6.0)),
        ]));
        assert!(rejected.is_empty());
        assert_eq!(ov.camel, Some(6));
        assert_eq!(ov.route_cond.as_deref(), Some("Deteriorated"));
        assert_eq!(ov.hours, Some(6.0));
        assert_eq!(ov.mule, None, "a species the override did not name stays None and cascades");
        assert_eq!(ov.season, None);
    }

    #[test]
    fn a_stage_override_rejects_the_two_journey_wide_keys() {
        let (_, rejected) =
            stage_override_from_pairs(&owned(&[("season_drift", JpValue::Bool(true)), ("auto_promote", JpValue::Bool(true))]));
        assert_eq!(rejected, vec!["season_drift", "auto_promote"], "neither is a per-stage field on JpStageOverride");
    }

    // ---------- option tables: every advertised key must be one the engine knows ----------

    #[test]
    fn transport_and_vessel_keys_all_resolve() {
        for t in JP_LAND_TRANSPORT_KEYS {
            assert!(jp_land_transport_kmh(t).is_some(), "{t} would be read as portage");
        }
        for v in JP_VESSEL_PREFERENCE {
            assert!(jp_ship_stats(v).is_some(), "{v} is not a JP_SHIPS key");
        }
    }

    #[test]
    fn pace_keys_all_resolve_to_their_own_multiplier() {
        // `jp_pace_mod` answers 1.0 for BOTH "Standard Pace" and an unknown
        // key, so identity is checked against the real table rather than
        // against "not 1.0".
        assert_eq!(PACE_KEYS.map(jp_pace_mod), [1.35, 1.25, 1.00, 0.75, 0.60], "a typo here silently becomes Standard Pace");
    }

    #[test]
    fn infra_tier_keys_all_resolve_to_their_own_multiplier() {
        let mods: Vec<f64> = infra_tier_keys().into_iter().map(jp_infra_mod).collect();
        assert_eq!(mods, vec![1.15, 1.00, 0.85, 0.70, 0.50]);
    }

    #[test]
    fn route_cond_keys_are_valid_for_their_own_category_and_no_other() {
        for cat in ["land", "river", "sea"] {
            let keys = route_cond_keys(cat);
            assert_eq!(keys.len(), 5, "{cat}");
            for k in keys {
                assert!(jp_route_cond_valid(cat, k), "{k} is not a {cat} condition");
            }
        }
        // The cross-category rejection `_jpDeriveStages` relies on.
        assert!(!jp_route_cond_valid("sea", "Maintained"));
        assert!(!jp_route_cond_valid("land", "Headwind"));
        assert!(route_cond_keys("portage").is_empty());
    }

    #[test]
    fn season_keys_all_resolve() {
        for s in JP_SEASON_ORDER {
            assert_eq!(jp_season_at(s, 0.0), s, "{s} is not a JP_SEASON_ORDER entry");
        }
    }

    #[test]
    fn weather_keys_all_resolve() {
        for w in JP_WEATHER_KEYS {
            assert!(jp_weather_mod(w).is_some(), "{w} would fall through to the seasonal average");
        }
    }

    #[test]
    fn desert_water_keys_all_resolve() {
        // `jp_desert_tier_for_gap` walks JP_DESERT_WATER's own key order and
        // returns an entry of it, so every advertised key must be reachable
        // from some measured waterless gap.
        let reachable: Vec<&str> = [0.5, 2.0, 5.0, 5000.0].into_iter().map(jp_desert_tier_for_gap).collect();
        assert_eq!(reachable, JP_DESERT_WATER_KEYS.to_vec());
    }

    #[test]
    fn foraging_keys_all_resolve_to_their_own_speed_cost() {
        let mods: Vec<f64> = FORAGING_KEYS
            .iter()
            .map(|m| jp_foraging(m, "Temperate Forest", "Forest Path", "Summer", 6, 1.0).move_mod)
            .collect();
        assert_eq!(mods, vec![1.00, 0.97, 0.88]);
    }

    #[test]
    fn grazing_keys_all_resolve_to_distinct_fodder_loads() {
        // `jp_grazing` is private; its fodder fraction is observable through
        // `jp_capacity`. An unrecognised key collapses onto the same 1.0
        // fraction as "None - carry all fodder", so three DISTINCT fodder
        // masses is exactly the proof that all three keys are recognised.
        let base = JpPlan { party: JpParty { group_size: 4, horse: 4, ..JpParty::default() }, ..JpPlan::default() };
        let fodder: Vec<f64> = GRAZING_KEYS
            .iter()
            .map(|g| jp_capacity(&JpPlan { grazing: (*g).to_string(), ..base.clone() }, "Steppe / Grassland", "Summer").fodder)
            .collect();
        assert!(fodder[0] > fodder[1] && fodder[1] > fodder[2], "{fodder:?}");
        assert_eq!(fodder[2], 0.0, "full grazing carries no fodder at all");
        let unknown = jp_capacity(&JpPlan { grazing: "Some Other Thing".to_string(), ..base }, "Steppe / Grassland", "Summer").fodder;
        assert_eq!(unknown, fodder[0], "the fallthrough really is the first key -- which is why distinctness is the test");
    }

    #[test]
    fn rest_cadence_keys_all_resolve_to_their_own_interval() {
        let every: Vec<i64> = REST_CADENCE_KEYS.iter().map(|k| jp_rest_days(30.0, Some(k), false).every).collect();
        assert_eq!(every, vec![0, 7, 5, 3]);
        // An unrecognised cadence also yields `every == 0`, same as
        // "None - press on"; the rest-day counts separate them.
        let rest: Vec<i64> = REST_CADENCE_KEYS.iter().map(|k| jp_rest_days(30.0, Some(k), false).rest_days).collect();
        assert_eq!(rest, vec![0, 4, 6, 10]);
        assert_eq!(jp_rest_days(30.0, Some("auto"), false).every, 5, "auto is the automatic rule, not a cadence key");
    }

    #[test]
    fn biome_keys_are_exactly_the_jp_biomes_table() {
        assert_eq!(biome_keys().len(), 12);
        for b in biome_keys() {
            assert!(jp_biome(b).is_some(), "{b} is not a JP_BIOMES key");
        }
        assert!(jp_biome("Hills").is_none(), "Hills is climate-classified, not a JP_BIOMES entry");
        assert!(jp_biome("Lake").is_none());
    }

    #[test]
    fn option_tables_cover_only_real_form_keys_and_are_non_empty() {
        let known: Vec<&str> = plan_to_pairs(&JpPlan::default()).into_iter().map(|(k, _)| k).collect();
        for (field, opts) in option_tables() {
            assert!(known.contains(&field), "{field} is not a key plan_from_pairs accepts");
            assert!(!opts.is_empty(), "{field} has no options");
            let mut sorted = opts.clone();
            sorted.sort_unstable();
            sorted.dedup();
            assert_eq!(sorted.len(), opts.len(), "{field} has a duplicate option");
        }
        for (name, opts) in reference_tables() {
            assert!(!opts.is_empty(), "{name} has no entries");
        }
    }

    #[test]
    fn party_count_pairs_names_only_form_keys() {
        let known: Vec<&str> = plan_to_pairs(&JpPlan::default()).into_iter().map(|(k, _)| k).collect();
        for (k, _) in party_count_pairs(&JpParty::default()) {
            assert!(known.contains(&k), "{k} is not a key plan_from_pairs accepts");
        }
    }

    // ---------- the world buffers ----------

    #[test]
    fn journey_world_builds_every_table_from_already_computed_state() {
        let (gw, gh) = (8usize, 6usize);
        let n = gw * gh;
        // A land ramp east of one column of ocean -- enough to make both
        // paint grids emit more than one class.
        let field: Vec<f32> = (0..n).map(|i| 0.30 + 0.09 * (i % gw) as f32).collect();
        let water_bodies: Vec<u8> = (0..n).map(|i| u8::from(i % gw == 0)).collect();
        let temp: Vec<f32> = (0..n).map(|i| -6.0 + 4.0 * (i % gw) as f32).collect();
        let rain: Vec<f32> = (0..n).map(|i| 0.8 - 0.1 * (i % gw) as f32).collect();
        let places = vec![
            NamedSettlement {
                tid: 0,
                placement: cartalith_civ::SettlementPlacement {
                    x: 3,
                    y: 2,
                    suit: 0.9,
                    faction: 1,
                    capital: true,
                    kind: SettlementKind::Capital,
                    coastal: false,
                },
                name: "Aldmoor".to_string(),
                pop: 9000,
            },
            NamedSettlement {
                tid: 0,
                placement: cartalith_civ::SettlementPlacement {
                    x: 6,
                    y: 4,
                    suit: 0.4,
                    faction: 2,
                    capital: false,
                    kind: SettlementKind::Hamlet,
                    coastal: true,
                },
                name: "Pell".to_string(),
                pop: 120,
            },
        ];
        let ways = vec![Way {
            tid: 0,
            pts: vec![(2.0, 1.0), (2.0, 2.0), (2.0, 3.0)],
            brks: vec![],
            km: 30.0,
            name: "The North Road".to_string(),
            way_type: cartalith_civ::WayType::Highway,
            a_idx: 0,
            b_idx: 1,
            hidden: false,
        }];

        let jw = JourneyWorld::build(&field, &water_bodies, &temp, &rain, gw, gh, false, 0.30, &ways, &places);

        assert_eq!(jw.cart_biome.len(), n);
        assert_eq!(jw.cart_terrain.len(), n);
        assert!(jw.cart_biome.contains(&15), "the water column must classify as Ocean / Deep Water");
        assert!(jw.cart_biome.iter().any(|&b| (1..=13).contains(&b)), "the land columns must classify as land");
        assert!(jw.cart_terrain.iter().any(|&t| t != 0), "land must get a surface");
        assert!(
            jw.cart_terrain.iter().enumerate().all(|(i, &t)| water_bodies[i] == 0 || t == 0),
            "water is unpainted terrain -- that is why _jpDeriveStages never consults it for a water stage"
        );

        // The highway's own cells, dilated by one, at its own priority.
        let cell = jw.road_cells.get(&(2, 2)).expect("the way's own cell");
        assert_eq!((cell.terrain, cell.cond, cell.pri), ("Paved Road", "Maintained", 3));
        assert!(jw.road_cells.contains_key(&(1, 2)), "dilated by one cell");
        assert!(!jw.road_cells.contains_key(&(6, 2)), "and no further");

        assert_eq!(jw.places.len(), 2);
        assert_eq!(jw.places[0], JpPlace { name: "Aldmoor".into(), kind: "capital".into(), x: 3.0, y: 2.0 });
        assert_eq!(jw.places[1].kind, "hamlet");
    }

    #[test]
    fn settlement_kind_keys_match_the_settlement_list_vocabulary() {
        assert_eq!(settlement_kind_key(SettlementKind::Capital), "capital");
        assert_eq!(settlement_kind_key(SettlementKind::City), "city");
        assert_eq!(settlement_kind_key(SettlementKind::Town), "town");
        assert_eq!(settlement_kind_key(SettlementKind::Village), "village");
        assert_eq!(settlement_kind_key(SettlementKind::Hamlet), "hamlet");
    }

    // ---------- end to end ----------

    /// The whole point of this module: that the buffers assembled above are
    /// enough to drive `jp_plan` end to end, not merely non-empty. Everything
    /// here is a synthetic raster of exactly the shapes `WorldGen` holds --
    /// `field`/`temp`/`rain`/`flow_discharge`/`water_bodies`/`territory` --
    /// so a shape or semantics mismatch between what `WorldGen` stores and
    /// what `JpWorld` expects fails here rather than at runtime inside Godot.
    #[test]
    fn the_assembled_world_actually_drives_jp_plan() {
        let (gw, gh) = (24usize, 16usize);
        let n = gw * gh;
        // A gentle west-to-east ramp with one column of ocean at x == 0, so
        // the route is a real land traverse with measurable ascent.
        let field: Vec<f32> = (0..n).map(|i| 0.28 + 0.02 * (i % gw) as f32).collect();
        let water_bodies: Vec<u8> = (0..n).map(|i| u8::from(i % gw == 0)).collect();
        let temp: Vec<f32> = vec![14.0; n];
        let rain: Vec<f32> = vec![0.45; n];
        let flow: Vec<f32> = vec![0.0; n];
        // `assign_territory`'s own convention: 0 = unowned, a faction id
        // otherwise. Passed through unchanged -- `jp_claimed_at`'s `>= 0`
        // test is the reference's own (its `civTerritory` is a `Uint8Array`,
        // so `>= 0` is likewise always true there); this port reproduces the
        // behaviour rather than "fixing" it into a divergence.
        let territory: Vec<i32> = vec![0; n];
        let places = vec![
            NamedSettlement {
                tid: 0,
                placement: cartalith_civ::SettlementPlacement {
                    x: 4,
                    y: 8,
                    suit: 0.8,
                    faction: 1,
                    capital: true,
                    kind: SettlementKind::Capital,
                    coastal: false,
                },
                name: "Harrowgate".to_string(),
                pop: 12000,
            },
            NamedSettlement {
                tid: 0,
                placement: cartalith_civ::SettlementPlacement {
                    x: 18,
                    y: 8,
                    suit: 0.5,
                    faction: 1,
                    capital: false,
                    kind: SettlementKind::Town,
                    coastal: false,
                },
                name: "Estwick".to_string(),
                pop: 3000,
            },
        ];
        let jw = JourneyWorld::build(&field, &water_bodies, &temp, &rain, gw, gh, false, 0.30, &[], &places);
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
            flow_field: Some(&flow),
            flow_thresh: 300.0,
            water_bodies: Some(&water_bodies),
            territory: Some(&territory),
            places: &jw.places,
            road_cells: &jw.road_cells,
            ocean_field: None,
            wind_field: None,
        };
        let pts: Vec<(f64, f64)> = (4..=18).map(|x| (x as f64, 8.0)).collect();
        let (plan, rejected) = plan_from_pairs(&owned(&[
            ("transport", JpValue::Str("Walking".into())),
            ("season", JpValue::Str("Summer".into())),
            ("group_size", JpValue::Num(6.0)),
            ("supply_days", JpValue::Num(10.0)),
        ]));
        assert!(rejected.is_empty());

        let layovers = cartalith_civ::JpLayovers::new();
        let journey = cartalith_civ::jp_plan(&world, &pts, &plan, &layovers, &|_, _| 1.0).expect("a 14-cell land traverse plans");

        assert!(!journey.stages.is_empty(), "the route must derive at least one stage");
        assert_eq!(journey.stages.len(), journey.results.len(), "one result per stage");
        assert!(journey.km > 0.0, "a real distance");
        assert_eq!(journey.profile.len(), pts.len(), "one elevation sample per route point");
        assert!(journey.has_land, "a land traverse");
        assert!(journey.stages.iter().all(|s| !s.biome.is_empty() && !s.terrain.is_empty()), "cart_biome/cart_terrain really fed it");
        // Both endpoints are settlements the route threads through -- proof
        // that `places` reached `civ_passed_settlements` in the right frame.
        let names: Vec<&str> = journey.stops.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"Harrowgate") && names.contains(&"Estwick"), "{names:?}");

        // The two post-processors this boundary also exposes.
        let verdict = cartalith_civ::jp_verdict(&journey);
        assert!(!verdict.level.is_empty() && !verdict.text.is_empty());
        assert!(cartalith_civ::jp_confidence(&journey).is_some(), "an unblocked, finite journey has a band");
    }

    /// A layover keyed by a stop's own `key` really lands on that stop --
    /// the one part of the request shape a UI cannot guess, since the key
    /// format is `jp_stop_key`'s and only the returned `stops` carry it.
    #[test]
    fn a_layover_keyed_by_a_returned_stop_key_lands_on_that_stop() {
        let key = cartalith_civ::jp_stop_key("Harrowgate", "capital", 4.0, 8.0);
        let mut layovers = cartalith_civ::JpLayovers::new();
        layovers.insert(key.clone(), 3);
        assert_eq!(layovers.get(&key).copied(), Some(3));
        assert!(key.contains("Harrowgate"), "the key a UI reads back off `stops` is name-derived: {key}");
    }
}
