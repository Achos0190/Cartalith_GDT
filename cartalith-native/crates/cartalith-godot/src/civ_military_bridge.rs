//! CIVIL ▸ Military and CIVIL ▸ Relationships — `GUI_GAP_REGISTER.md`
//! **CV-25** and **CV-26**.
//!
//! Its own file with a `#[godot_api(secondary)]` block, following
//! `geojson_bridge.rs` and `export_raster.rs`, because both readouts are
//! self-contained modal-open queries with no state of their own.
//!
//! ## What each one is, in one line
//!
//! - [`WorldGen::civ_military_summary`] — three things under one call,
//!   because all three come off the same aggregate and paying for it once
//!   is the point:
//!   - `_civFactionAggregates`' `power.military` per faction, and the
//!     reference's own fortification ladder (`cartalith_civ::military`) per
//!     settlement. **Both ports.**
//!   - `cartalith_civ::manpower`'s four headcounts per faction, nested
//!     under each faction row's `manpower` key.
//!     **New — `MILITARY_MANPOWER_SCOPE.md`, no reference at any line.**
//! - [`WorldGen::civ_faction_relations`] — the derived, recomputed
//!   faction-to-faction edge (`cartalith_civ::relations`). **New, and
//!   deliberately small**; see that module's doc for the four terms.
//!
//! ## `power.military` is kept as it is, and the reason is worth stating
//!
//! `MILITARY_MANPOWER_SCOPE.md` raises whether the 0-100 axis should become
//! a presentation of the manpower model. It is deliberately **not**: it is a
//! golden-verified port of the reference's own formula
//! (`0.45·normPop + 0.35·fortifiedFraction + 0.20·capitalTierNorm`), and
//! rewriting it to derive from a model the reference does not have would
//! break that parity to gain nothing the headcounts do not already say
//! better. The two answer different questions — one is *this faction against
//! the others on this map*, the other is *how many people, in absolute
//! terms* — so they are reported side by side and the shell says which is
//! which. Recorded here rather than only in the scope document, because this
//! file is where somebody would go to change it.
//!
//! ## The one behaviour change worth stating
//!
//! `FactionPlace::from_settlement` hard-wires `fortified: false`, because
//! `cartalith-civ` is stateless and the `umWalls` override lives here in
//! `place_extras`. So every existing caller of `civ_faction_aggregates` in
//! this crate has been feeding the military axis a constant zero for its
//! `0.35*fortifiedFraction` term.
//!
//! This module composes the place rows itself — `um_infer_walls` per
//! settlement, then `FactionPlace { fortified, ..from_settlement(s) }` —
//! which is what the reference's own aggregate pass does
//! (`if(_umInferWalls(p)) b.fortifiedCount++`). That makes this file's
//! numbers *closer* to the reference than [`WorldGen::civ_faction_terrain_fits`]'s,
//! which does not need `fortified` and is left alone rather than perturbed:
//! its golden output is the terrain mix, and nothing in it reads `power`.

use godot::prelude::*;

use cartalith_civ::military::{
    WallPlace, civ_place_defensibility, civ_relative_elevation, um_infer_walls, um_wall_spec,
};
use cartalith_civ::manpower::{ManpowerInput, civ_military_manpower};
use cartalith_civ::relations::{FactionRelationsInput, civ_faction_relations};
use cartalith_civ::roster::civ_ag_tech_by_key;
use cartalith_civ::trade::{NavKind, RoadComponents, place_navigability};
use cartalith_civ::urban_adapter::UrbanWorld;
use cartalith_civ::{FactionAggregates, FactionAggregatesInput, FactionPlace, WayType};

use crate::{WorldGen, WorldSource};

/// How much of a way's own kilometres count toward
/// [`cartalith_civ::manpower::ManpowerInput::road_density`], by
/// [`WayType`]. A track carries a messenger; a highway carries a baggage
/// train, which is the thing an army actually needs.
///
/// The tiers are the reference's own classification
/// (`civ_consolidate_and_smooth_ways`' `maxU` thresholds), not a second
/// vocabulary invented here. Sea lanes are counted separately, through
/// `sea_share`, because a lane is not a road and its length says nothing
/// about how much of the interior it reaches.
fn way_logistics_weight(t: WayType) -> f64 {
    match t {
        WayType::Highway => 1.00,
        WayType::Regional => 0.80,
        WayType::Road => 0.55,
        WayType::Track => 0.30,
    }
}

/// The weighted way length per 1 000 km² of territory at which
/// `road_density` reads `1.0`.
///
/// **Measured, not guessed, and the first value was wrong.** Anchoring on
/// the Roman empire's ~80 000 km of built road over ~5 000 000 km²
/// (~16 km/1 000 km²) suggested `40`. On real generated worlds that made
/// roads a dead term: a 33-settlement world's six factions came out at
/// 1.1-9.1 weighted km/1 000 km², so `road_density` read 0.03-0.23 and
/// contributed at most 0.10 of a logistics capacity that ranged 0.37-0.53.
/// The mistake is a category one — this port's way network is *inter-
/// settlement trunk roads only*, with no local lanes, farm tracks or streets
/// in it, so it is not comparable to a real road inventory.
///
/// `10` is calibrated against what the way network actually produces, and it
/// makes roads carry real weight: the same six factions spread 0.11-0.91.
const ROAD_DENSITY_REF_KM_PER_1000_KM2: f64 = 10.0;

/// One settlement's fortification row, built once and reused by both
/// entry points.
struct Defence {
    index: usize,
    tid: u64,
    name: String,
    faction: i32,
    kind: &'static str,
    spec: &'static str,
    walled: bool,
    defensibility: f64,
    pop: u32,
}

impl WorldGen {
    /// Every settlement's wall spec and defensive strength, in
    /// `civ.settlements` order.
    ///
    /// Reads the place editor's own `umWalls`/`umAge`/`traits`/
    /// `specialisation` overrides out of `place_extras`, so an edit made in
    /// ED-03 is visible here — the one place in this port where those two
    /// overrides finally have a consumer (`civ_roster_bridge.rs`'s module
    /// doc says outright that they reached nothing).
    fn defences(&self) -> Vec<Defence> {
        let (Some(civ), Some(WorldSource::Generated(ws))) = (self.civ.as_ref(), self.source.as_ref())
        else {
            return Vec::new();
        };
        let (gw, gh, sea) = (self.gw.max(0) as usize, self.gh.max(0) as usize, self.sea_level);
        civ.settlements
            .iter()
            .enumerate()
            .map(|(index, s)| {
                let e = civ.place_extras.get(s.tid);
                let r = civ_relative_elevation(
                    &ws.field,
                    gw,
                    gh,
                    sea,
                    s.placement.x as f64,
                    s.placement.y as f64,
                );
                let p = WallPlace {
                    walls_override: e.walls,
                    kind: s.placement.kind,
                    pop: s.pop as f64,
                    fortified_trait: e.traits.iter().any(|t| t == "fortified"),
                    age_override: e.age.map(f64::from),
                    specialisation: if e.specialisation.is_empty() {
                        None
                    } else {
                        Some(e.specialisation.as_str())
                    },
                    relative_elevation: r,
                };
                let spec = um_wall_spec(&p);
                let walled = um_infer_walls(&p);
                Defence {
                    index,
                    tid: s.tid,
                    name: s.name.clone(),
                    faction: s.placement.faction,
                    kind: kind_key(s.placement.kind),
                    spec,
                    walled,
                    defensibility: civ_place_defensibility(r, walled),
                    pop: s.pop,
                }
            })
            .collect()
    }

    /// `civ_faction_aggregates` with the fortification term actually fed —
    /// see this module's header. Returns `None` before any world exists.
    ///
    /// ## What is and is not supplied to it
    ///
    /// **`resources` is rebuilt on demand**, because CV-26's trade term
    /// reads the aggregate's `imports`/`exports` and those come from
    /// `resourceMean` against the world mean. `compute_civilisation` frees
    /// the resource rasters, so this rebuilds the three passes they need
    /// (biome, lithology, potentials) — the same on-demand-rebuild shape
    /// [`WorldGen::civ_faction_terrain_fits`] already uses for the biome and
    /// ocean-distance fields, and the reason both are modal-open calls
    /// rather than per-frame ones.
    ///
    /// **`density` is deliberately absent.** The reference passes
    /// `currentPopulationDensity()`; the only per-cell density this port
    /// retains is `CivData::dens`, which is `civ_current_agrarian_density`
    /// — a *different field*, and substituting it would silently change
    /// `foodProductionCapacity` away from the reference's number. So the
    /// food half of the trade balance is absent, which puts `food` in every
    /// faction's imports and nobody's exports. That costs nothing:
    /// `civ_faction_relations` discounts any good no faction supplies, for
    /// exactly this reason (see its `trade_complement`).
    fn aggregates_with_walls(&self, defences: &[Defence]) -> Option<FactionAggregates> {
        let (Some(civ), Some(WorldSource::Generated(ws))) = (self.civ.as_ref(), self.source.as_ref())
        else {
            return None;
        };
        let (gw, gh, sea) = (self.gw.max(0) as usize, self.gh.max(0) as usize, self.sea_level);
        let biome = cartalith_civ::build_biome_raster(
            &civ.water_bodies,
            &ws.temperature,
            &ws.rainfall,
        );
        let lithology = cartalith_civ::build_lithology(
            &ws.field,
            &ws.age_field,
            &ws.volcanic_field,
            &ws.crust_field,
            &ws.resistance_field,
            &ws.rainfall,
            sea,
        );
        // `scarcity=true, scarcity_legacy=false` — the production defaults
        // `currentResourcePotentials()` runs with, not a choice made here.
        let resources = cartalith_civ::build_resource_potentials(
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
            sea,
            Some(&ws.volcanic_field),
            true,
            false,
        );
        let has_religion = civ.faction_roster.has_religion_flags();
        let input = FactionAggregatesInput {
            faction_count: civ.faction_roster.0.len(),
            gw,
            gh,
            sea,
            map_width_km: self.map_width_km,
            field: &ws.field,
            territory: Some(&civ.territory),
            density: None,
            resources: Some(&resources),
            // The terrain-mix half of the aggregate; neither readout here
            // reads `terrain_mix`, and `civ_faction_terrain_fits` is the
            // call that pays for those two extra passes.
            biome: None,
            flow: None,
            flow_thresh: f64::INFINITY,
            ocean_dist: None,
            faction_has_religion: Some(&has_religion),
        };
        let places: Vec<FactionPlace> = civ
            .settlements
            .iter()
            .zip(defences)
            .map(|(s, d)| FactionPlace { fortified: d.walled, ..FactionPlace::from_settlement(s) })
            .collect();
        Some(cartalith_civ::civ_faction_aggregates(&input, &places))
    }

    /// One [`cartalith_civ::manpower::Manpower`] per real faction, indexed
    /// `1..faction_count`, in that order.
    ///
    /// ## Where each of the five variables is actually read from
    ///
    /// Nothing here is a new pass over the grid except the one sweep over
    /// `civ.territory`, which is `O(cells)` and shares its loop with the
    /// land-capacity sum. Everything else is per-settlement or per-way.
    ///
    /// - **Agricultural labour ratio** — the roster's own `ag_tech` key
    ///   through [`civ_ag_tech_by_key`]. This is
    ///   [`cartalith_civ::roster::AG_TECH_LEVELS`]' first consumer anywhere
    ///   in this port; its module doc says outright that it had none.
    /// - **Food surplus** — `CivData::dens`
    ///   ([`cartalith_civ::timeline::civ_current_agrarian_density`]) summed
    ///   over the faction's own territory cells, which is exactly what
    ///   `civ_agrarian_regional_total`'s *"Land sustains ≈ N"* readout does
    ///   for the whole map, restricted to one owner.
    /// - **Fiscal extraction** — the roster's `government` key (likewise its
    ///   first consumer) and the share of the faction's settlements that
    ///   share a [`RoadComponents`] component with its capital.
    /// - **Logistics** — the way network's four tiers by length per unit
    ///   territory, plus [`place_navigability`]'s river/sea verdict per
    ///   settlement.
    /// - **Professionalization** — derived inside the model from the two
    ///   above; nothing is read for it here.
    fn manpower_rows(
        &self,
        agg: &FactionAggregates,
    ) -> Vec<cartalith_civ::manpower::Manpower> {
        let (Some(civ), Some(WorldSource::Generated(ws))) = (self.civ.as_ref(), self.source.as_ref())
        else {
            return Vec::new();
        };
        let n_f = civ.faction_roster.0.len();
        let (gw, gh) = (self.gw.max(0) as usize, self.gh.max(0) as usize);
        let cell_km = self.map_width_km / (gw.max(1) as f64);
        let cell_km2 = cell_km * cell_km;

        // People each faction's own land sustains -- the same integral
        // `civ_agrarian_regional_total` takes over the whole map.
        let mut land_capacity = vec![0.0f64; n_f];
        if civ.dens.len() == gw * gh && civ.territory.len() == gw * gh {
            for (i, &f) in civ.territory.iter().enumerate() {
                if f > 0 && (f as usize) < n_f {
                    land_capacity[f as usize] += f64::from(civ.dens[i]) * cell_km2;
                }
            }
        }

        // Water access, per settlement. The same `UrbanWorld` shape
        // `civ_trade_bridge.rs` builds, with the same deliberately-empty
        // `river_polys`: `um_site_kind_from_terrain` reads `field`, `flow`
        // and `flow_thresh` and never consults the traced polylines, so the
        // expensive trace is not paid for a share.
        let polys: Vec<Vec<(f64, f64)>> = Vec::new();
        let world = UrbanWorld {
            field: &ws.field,
            flow: &ws.flow_discharge,
            water_bodies: &civ.water_bodies,
            order: ws.stream_order.as_deref(),
            river_polys: &polys,
            gw,
            gh,
            sea_level: self.sea_level,
            map_width_km: self.map_width_km,
            flow_thresh: cartalith_hydrology::river_flow_thresh(gw, gh, gw, self.map_width_km),
            world_seed: self.seed,
        };
        let nav: Vec<NavKind> =
            civ.settlements.iter().map(|s| place_navigability(&world, s).kind).collect();

        // Way length per faction, weighted by tier. A way whose two ends sit
        // in different factions counts for neither: it is a road *between*
        // polities, and crediting both with it would let a shared frontier
        // road make two states look better supplied than either is.
        let mut road_km = vec![0.0f64; n_f];
        for w in &civ.ways {
            let (Some(a), Some(b)) = (civ.settlements.get(w.a_idx), civ.settlements.get(w.b_idx))
            else {
                continue;
            };
            let f = a.placement.faction;
            if f > 0 && (f as usize) < n_f && f == b.placement.faction {
                road_km[f as usize] += w.km * way_logistics_weight(w.way_type);
            }
        }

        let mut rc = RoadComponents::build(civ.settlements.len(), &civ.ways);

        (1..n_f)
            .map(|f| {
                let entry = &civ.faction_roster.0[f];
                let mine: Vec<usize> = civ
                    .settlements
                    .iter()
                    .enumerate()
                    .filter(|(_, s)| s.placement.faction == f as i32)
                    .map(|(i, _)| i)
                    .collect();
                let count = mine.len() as f64;

                // A faction with no capital reaches nothing by definition:
                // there is no seat for a road to run back to.
                let capital = agg.by_faction.get(f).and_then(|a| a.capital);
                let capital_road_reach = match (capital, count > 0.0) {
                    (Some(c), true) => {
                        mine.iter().filter(|&&i| rc.connected(i, c)).count() as f64 / count
                    }
                    _ => 0.0,
                };
                let share = |pred: fn(NavKind) -> bool| {
                    if count > 0.0 {
                        mine.iter().filter(|&&i| nav.get(i).copied().is_some_and(pred)).count()
                            as f64
                            / count
                    } else {
                        0.0
                    }
                };

                let territory_km2 =
                    agg.by_faction.get(f).map_or(0.0, |a| a.territory_km2).max(0.0);
                let road_density = if territory_km2 > 0.0 {
                    (road_km[f] / (territory_km2 / 1000.0)) / ROAD_DENSITY_REF_KM_PER_1000_KM2
                } else {
                    0.0
                };

                civ_military_manpower(&ManpowerInput {
                    nucleated_pop: agg.by_faction.get(f).map_or(0.0, |a| a.pop),
                    farmers_per_urbanite: civ_ag_tech_by_key(&entry.ag_tech).farmers_per_urbanite,
                    land_capacity: land_capacity[f],
                    government: &entry.government,
                    capital_road_reach,
                    road_density,
                    navigable_share: share(NavKind::navigable),
                    sea_share: share(|k| k == NavKind::Sea),
                })
            })
            .collect()
    }
}

/// One faction's manpower row as the shell reads it, or an empty dictionary
/// when there is no row (a faction index the aggregate never produced).
///
/// Every driver is carried beside every output, deliberately: the model has
/// no reference to check against, so the only defensible presentation is one
/// that shows its working. A reader who disagrees with a number can see
/// which of the five variables produced it.
fn manpower_dict(
    m: Option<&cartalith_civ::manpower::Manpower>,
    ag_tech: &str,
    government: &str,
) -> VarDictionary {
    let Some(m) = m else {
        return VarDictionary::new();
    };
    let d = &m.drivers;
    let ladder: Array<VarDictionary> = m
        .force_ladder
        .iter()
        .map(|r| {
            vdict! {
                "days" => r.days,
                "force" => r.force,
                "share" => r.share,
                "capped_by_pool" => r.capped_by_pool,
            }
        })
        .collect();
    vdict! {
        // -- the four outputs
        "standing_army" => m.standing_army,
        "professional_core" => m.professional_core,
        "field_army" => m.field_army,
        "emergency_mobilization" => m.emergency_mobilization,
        "field_duration_days" => m.field_duration_days,
        "emergency_duration_days" => m.emergency_duration_days,
        "force_ladder" => &ladder,
        // -- the populations behind them
        "total_population" => m.total_population,
        "farming_population" => m.farming_population,
        "mobilization_pool" => m.mobilization_pool,
        "standing_share" => m.standing_share,
        "emergency_share" => m.emergency_share,
        // -- the era band's denominator (owner ruling, 2026-08-25). The
        //    bands are shares of the citizen/free body, not of the total, so
        //    that body is surfaced beside the verdict rather than being an
        //    invisible divisor: a reader must be able to see what the
        //    percentage is a percentage OF.
        "citizen_population" => m.citizen_population,
        "citizen_fraction" => d.citizen_fraction,
        "standing_citizen_share" => m.standing_citizen_share,
        "emergency_citizen_share" => m.emergency_citizen_share,
        "concentration_ratio" => m.concentration_ratio,
        // -- the five variables, plus the two they are built from
        "food_surplus_per_farmer" => d.food_surplus_per_farmer,
        "agricultural_labour_ratio" => d.agricultural_labour_ratio,
        "fiscal_extraction_efficiency" => d.fiscal_extraction_efficiency,
        "professionalization" => d.professionalization,
        "logistics_capacity" => d.logistics_capacity,
        "road_density" => d.road_density,
        "navigable_share" => d.navigable_share,
        "sea_share" => d.sea_share,
        "state_capacity" => d.state_capacity,
        "ecological_factor" => d.ecological_factor,
        // -- the era, derived, with its band reported and never enforced
        "era" => m.era_band.name,
        "era_constraint" => m.era_band.constraint,
        "era_standing_lo" => m.era_band.standing.0,
        "era_standing_hi" => m.era_band.standing.1,
        "era_mobilization_lo" => m.era_band.mobilization.0,
        "era_mobilization_hi" => m.era_band.mobilization.1,
        "era_standing_verdict" => m.era_standing_verdict,
        "era_mobilization_verdict" => m.era_mobilization_verdict,
        // -- the two roster fields this is the first consumer of
        "ag_tech" => ag_tech,
        "government" => government,
    }
}

/// The tier key the shell labels with — the reference's own
/// `CIV_SETTLEMENT_CLASSES` keys, not a second vocabulary.
fn kind_key(k: cartalith_civ::SettlementKind) -> &'static str {
    use cartalith_civ::SettlementKind::*;
    match k {
        Metropolis => "metropolis",
        Capital => "capital",
        City => "city",
        Town => "town",
        Village => "village",
        Hamlet => "hamlet",
    }
}

#[godot_api(secondary)]
impl WorldGen {
    /// CIVIL ▸ Military (`GUI_GAP_REGISTER.md` **CV-25**).
    ///
    /// Two arrays under one call, because both come off the same
    /// fortification pass and the category shows them together:
    ///
    /// - `"factions"` — one row per real faction (index 0, Unclaimed, is
    ///   omitted): `faction`, `name`, `military` and `overall` (0-100,
    ///   `_civFactionAggregates`' own labelled heuristic), `pop`,
    ///   `settlement_count`, `fortified_count`, `fortified_fraction`,
    ///   `walled_stone`/`walled_palisade`/`walled_ditch` counts,
    ///   `capital` (name, or `""` when the faction seats nobody), and
    ///   `manpower` — the whole of [`cartalith_civ::manpower`]'s answer for
    ///   this faction, nested (see [`manpower_dict`] for the keys and for
    ///   why it is nested rather than flattened alongside `military`).
    /// - `"settlements"` — one row per settlement: `index`, `tid`, `name`,
    ///   `faction`, `kind`, `pop`, `wall_spec` (one of
    ///   `cartalith_civ::military::WALL_SPECS`), `walled`, and
    ///   `defensibility` (0-1).
    ///
    /// Empty arrays before the first generate, or on a loaded save (which
    /// carries no `CivData` — `SAVEFILE_COMPAT.md`), never an error: an
    /// absent civilisation layer is a real state, not a fault.
    ///
    /// **What this deliberately does not report:** garrison headcounts,
    /// campaigns, or anything that moves. See
    /// `cartalith_civ::military`'s module doc.
    #[func]
    fn civ_military_summary(&self) -> VarDictionary {
        let mut out = VarDictionary::new();
        let defences = self.defences();
        // Both `None` arms are the same real state (no civilisation layer),
        // and neither is an error. No `unwrap`/`expect` anywhere in this
        // file: a panic here unwinds through a GDExtension callback.
        let (Some(civ), Some(agg)) = (self.civ.as_ref(), self.aggregates_with_walls(&defences))
        else {
            out.set("factions", &Array::<VarDictionary>::new());
            out.set("settlements", &Array::<VarDictionary>::new());
            return out;
        };
        let manpower = self.manpower_rows(&agg);

        let factions: Array<VarDictionary> = (1..civ.faction_roster.0.len())
            .filter_map(|f| {
                let a = agg.by_faction.get(f)?;
                let count = |spec: &str| {
                    defences.iter().filter(|d| d.faction == f as i32 && d.spec == spec).count()
                        as i64
                };
                let fortified_count =
                    defences.iter().filter(|d| d.faction == f as i32 && d.walled).count() as i64;
                let capital = a
                    .capital
                    .and_then(|i| defences.get(i))
                    .map_or_else(String::new, |d| d.name.clone());
                let entry = &civ.faction_roster.0[f];
                Some(vdict! {
                    "faction" => f as i64,
                    "name" => entry.name.as_str(),
                    "military" => a.power.military,
                    "overall" => a.power.overall,
                    "pop" => a.pop,
                    "settlement_count" => a.settlement_count as i64,
                    "fortified_count" => fortified_count,
                    "fortified_fraction" => a.fortified_fraction,
                    "walled_stone" => count("stone"),
                    "walled_palisade" => count("palisade"),
                    "walled_ditch" => count("ditch"),
                    "capital" => capital.as_str(),
                    // Nested rather than flattened, so the two models stay
                    // visibly separate: `military`/`overall` above are the
                    // reference's own relative 0-100 heuristic, and this is
                    // an absolute headcount model with no reference at all.
                    // Reading one where the other was meant is the mistake
                    // this nesting exists to make hard.
                    "manpower" => &manpower_dict(
                        manpower.get(f - 1),
                        entry.ag_tech.as_str(),
                        entry.government.as_str(),
                    ),
                })
            })
            .collect();

        let settlements: Array<VarDictionary> = defences
            .iter()
            .map(|d| {
                vdict! {
                    "index" => d.index as i64,
                    "tid" => d.tid as i64,
                    "name" => d.name.as_str(),
                    "faction" => d.faction as i64,
                    "kind" => d.kind,
                    "pop" => d.pop as i64,
                    "wall_spec" => d.spec,
                    "walled" => d.walled,
                    "defensibility" => d.defensibility,
                }
            })
            .collect();

        out.set("factions", &factions);
        out.set("settlements", &settlements);
        out
    }

    /// CIVIL ▸ Relationships (`GUI_GAP_REGISTER.md` **CV-26**).
    ///
    /// One row per unordered pair of real factions, ascending by
    /// `(a, b)`: `a`, `b`, `a_name`, `b_name`, `value` (-1..1), `stance`
    /// (one of `cartalith_civ::relations::RELATION_STANCES`),
    /// `border_cells`, `border_fraction`, and the four terms
    /// `culture_term`, `religion_term`, `trade_term`, `rivalry_term` so the
    /// shell can show its working rather than assert a number.
    ///
    /// **Derived and recomputed, never stored.** There is no relation on
    /// `CivData`, nothing to save, and no `#[func]` that writes one — the
    /// same shape as `civ_faction_terrain_fits` above and
    /// `wildlife_regions`. Diplomacy actions, treaties and change over time
    /// are out of scope by design, not by omission.
    #[func]
    fn civ_faction_relations(&self) -> Array<VarDictionary> {
        let defences = self.defences();
        let (Some(civ), Some(agg)) = (self.civ.as_ref(), self.aggregates_with_walls(&defences))
        else {
            return Array::new();
        };
        let n = civ.faction_roster.0.len();
        let cultures: Vec<&str> =
            civ.faction_roster.0.iter().map(|e| e.culture.as_str()).collect();
        let religions: Vec<&str> =
            civ.faction_roster.0.iter().map(|e| e.religion.as_str()).collect();
        let input = FactionRelationsInput {
            faction_count: n,
            gw: self.gw.max(0) as usize,
            gh: self.gh.max(0) as usize,
            territory: Some(&civ.territory),
            cultures: &cultures,
            religions: &religions,
            wrap_x: self.world,
        };
        civ_faction_relations(&input, &agg)
            .pairs
            .iter()
            .map(|r| {
                vdict! {
                    "a" => r.a as i64,
                    "b" => r.b as i64,
                    "a_name" => civ.faction_roster.0[r.a].name.as_str(),
                    "b_name" => civ.faction_roster.0[r.b].name.as_str(),
                    "value" => r.value,
                    "stance" => r.stance,
                    "border_cells" => r.border_cells as i64,
                    "border_fraction" => r.border_fraction,
                    "culture_term" => r.culture_term,
                    "religion_term" => r.religion_term,
                    "trade_term" => r.trade_term,
                    "rivalry_term" => r.rivalry_term,
                }
            })
            .collect()
    }
}
