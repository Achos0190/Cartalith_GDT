//! CIVIL ▸ Military and CIVIL ▸ Relationships — `GUI_GAP_REGISTER.md`
//! **CV-25** and **CV-26**.
//!
//! Its own file with a `#[godot_api(secondary)]` block, following
//! `geojson_bridge.rs` and `export_raster.rs`, because both readouts are
//! self-contained modal-open queries with no state of their own.
//!
//! ## What each one is, in one line
//!
//! - [`WorldGen::civ_military_summary`] — `_civFactionAggregates`'
//!   `power.military` per faction, plus the reference's own fortification
//!   ladder (`cartalith_civ::military`) per settlement. **A port.**
//! - [`WorldGen::civ_faction_relations`] — the derived, recomputed
//!   faction-to-faction edge (`cartalith_civ::relations`). **New, and
//!   deliberately small**; see that module's doc for the four terms.
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
use cartalith_civ::relations::{FactionRelationsInput, civ_faction_relations};
use cartalith_civ::{FactionAggregates, FactionAggregatesInput, FactionPlace};

use crate::{WorldGen, WorldSource};

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
    ///   `walled_stone`/`walled_palisade`/`walled_ditch` counts, and
    ///   `capital` (name, or `""` when the faction seats nobody).
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
                Some(vdict! {
                    "faction" => f as i64,
                    "name" => civ.faction_roster.0[f].name.as_str(),
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
