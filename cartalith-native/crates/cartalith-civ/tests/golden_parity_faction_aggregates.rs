//! Golden-parity tests for `ECONOMY_SCOPE.md`'s last unstarted piece:
//! `_civFactionAggregates` (reference HTML line 23575, v1.16, extended by
//! v1.55's "Territory Fit"), its helper `_civFactionCapital` (23566), the
//! `CIV_TAX_RATE`/`CIV_PRIMARY_SPECIALISATION` tables (23557/23553), and
//! `_civOceanDistField` (22450) which the terrain-mix coast axis needs.
//! `_civCultureTerrainFit` (23748) is re-verified here **through** the real
//! aggregate output rather than against hand-written maps -- that is the
//! concrete unblock this milestone exists for (the GUI parity audit,
//! `d84dfd0`, found it un-exposable because nothing computed its
//! `terrain_mix`/`world_mean_terrain` inputs).
//!
//! # The harness
//!
//! Node `vm.runInContext`, fresh per this project's established practice
//! (not checked in). **Whole `<script>` blocks, not line slices**: blocks #1
//! (2084-14556) and #2 (14563-26720), the same boundaries
//! `golden_parity_civ_tools.rs` and `golden_parity_hierarchical_network.rs`
//! document, asserted by the real delimiters (the line before each slice
//! *is* `<script>`, the line after *is* `</script>`). The block-comment
//! balance assertion ran on both blocks and earned its keep twice, both
//! times by being wrong -- which is how a check of this kind proves it is
//! actually looking:
//!
//! 1. A false "newline in regex" on `raw[i]/=cRange` (block #1, reference
//!    line 2585): a `/` after `]` is division, not a regex opener. The
//!    prevSig class was too generous.
//! 2. Then a false report inside `_jpPackRange`'s hint builder (block #2,
//!    reference line ~19663): the `/` in a `</div>` **inside a template
//!    literal**. Cause: a `${...}` substitution was being closed by the
//!    first `}` inside it, so an IIFE's own `try{...}` brace ended the
//!    substitution early and the rest of the template was scanned as code.
//!    Fixed with a real brace-depth counter per substitution, not by
//!    deleting the check.
//!
//! Both blocks are additionally compiled with `new vm.Script(...)` before
//! being run -- a real JS parser is the strongest slice-boundary guarantee
//! available, since a slice cut mid-comment, mid-string or mid-function
//! cannot compile at all.
//!
//! Everything is driven from **inside** the context (`civTerritory`,
//! `_civTerrGen`, `_civAggGen`, `GW`/`GH`, `CIV_FACTIONS` are all `let`
//! declarations, therefore lexical bindings a `vm` script cannot reach from
//! the host -- assigning them from outside would create a shadow the
//! reference never reads, and the failure mode is silently-empty output).
//!
//! # The world under it
//!
//! Before any aggregation ran, the harness FNV-1a-64'd its `field`, biome
//! raster, lithology, water-access field, ocean distance transform and the
//! river mask (`flowField[i] > riverFlowThresh(GW,GH)`) over their raw
//! bytes. All six hashes are re-asserted below against this port's own
//! `generate_terrain` + `build_biome_raster` + `build_lithology` +
//! `build_water_access` + `civ_ocean_dist_field`, and all six match
//! **exactly** in both cases -- so every categorical input the aggregation
//! branches on is provably the same on both sides. The synthetic territory
//! raster is hashed and compared the same way.
//!
//! Two inputs are **not** bit-identical, and this is disclosed rather than
//! papered over. `tempField`/`rainField` diverge from this port's own by
//! ~1-3 f32 ULP in a minority of cells (case 0: 1 temperature cell of 432,
//! 178 rainfall cells, max relative 2.7e-7), a **pre-existing** property of
//! the climate chain that predates this milestone and is entirely upstream
//! of it. That propagates into carrying capacity (44 cells, 3.1e-7), NPP
//! (17 cells, 2.0e-7), population density (41 cells, 3.1e-7) and the
//! resource potentials. It changes nothing categorical: it flips no river
//! cell across the flow threshold (the river-mask hash is exact), no biome
//! classification (that hash is exact), and no lithology class. So:
//!
//! * population density and flow are compared by their land-cell sums at
//!   `1e-6` relative -- the quantity the aggregation actually integrates;
//! * `world_mean_resource` and every per-faction `resource_potential` are
//!   compared at `1e-6` relative;
//! * `food_production_capacity`/`food_surplus` are `Math.round`ed sums of
//!   density, so they are compared to +/-1 -- the honest bound when the
//!   pre-rounding value can differ by ~0.04;
//! * **everything else** -- populations, trade volumes, tax income, means,
//!   counts, the capital pick, territory area, the full terrain mix, the
//!   whole five-axis power breakdown, the sector split, and the
//!   export/import/strategic lists -- is compared at `1e-9` relative or
//!   exactly, because it is pure arithmetic over the inputs that *are*
//!   bit-identical.
//!
//! # Emptiness / shape assertions -- because three subsystems have been bitten
//!
//! Journey Planner M5 got a silently-empty stage list from a slice that
//! parsed; tool-milestone C got silently-empty paint output from a shadowed
//! global. Both passed every structural check. So: land-cell count,
//! per-faction territory-cell counts and a `territory.iter().any(|t| t > 0)`
//! are asserted explicitly, and the fixture *shapes* were chosen to reach
//! the edges rather than the happy path -- a faction with neither territory
//! nor settlements, a faction with territory but no settlement, a faction
//! with exactly one settlement, a zero-population settlement, a settlement
//! whose `specialisation` maps to no primary sector (folds to `craft`), a
//! place with an out-of-range faction id, and (both cases) a faction whose
//! territory spans the x=0/x=gw-1 seam.
//!
//! # Territory
//!
//! Deliberately **synthetic and rule-generated**, not `assign_territory`'s
//! output: what is under test here is the aggregation, and a hand-specified
//! rule is the only way to place an empty faction and a seam-spanning
//! faction on purpose. The identical rule runs on both sides and the
//! resulting raster is FNV-hashed and compared, so the two sides provably
//! aggregate over the same ownership map.
//!
//! # Fields the port does not have
//!
//! `tradeVolume`/`economicImportance`/`specialisation` and
//! `_umInferWalls(p)` have no producer anywhere in this workspace. They are
//! caller-supplied on `FactionPlace`; the harness captured the reference's
//! own `_umInferWalls` verdict per place and the same booleans are passed
//! in here, so `fortified_fraction` and the military power axis are genuinely
//! tested rather than trivially zero on both sides.

use cartalith_civ::{FactionAggregatesInput, FactionPlace, ResourcePotentials, SettlementKind, civ_culture_terrain_fit};

/// `CIV_FACTIONS.length` (reference line 14568): seven, index 0 being
/// "Unclaimed". This port has no `CIV_FACTIONS` table of its own (faction
/// count is a caller parameter to `assign_landmass_factions`), so the
/// reference's own length is stated here.
const N_F: usize = 7;

struct World {
    ws: cartalith_engine::WorldState,
    biome: Vec<u8>,
    lith: Vec<u8>,
    water: Vec<f32>,
    dens: Vec<f32>,
    res: ResourcePotentials,
    ocean_dt: Vec<f32>,
    flow_thresh: f64,
}

/// Mirrors the reference's own `currentPopulationDensity()` /
/// `currentResourcePotentials()` / `currentCarryingCapacity()` /
/// `currentNPP()` argument chains exactly (reference lines 6452-6455,
/// 6613), including `_biomeK=0` (so no wetland mask reaches either
/// `buildCarryingCapacity` or `estimateRegionalDensityKm2`) and
/// `state.climate.maxRainMm=3000`.
fn build_world(gw: usize, gh: usize, seed: i32, world: bool) -> World {
    let mut p = cartalith_engine::WorldParams::defaults(gw, gh, seed);
    p.world = world;
    p.climate.w_iters = 12;
    let ws = cartalith_engine::generate_terrain(&p);
    assert!((ws.sea_level - 0.42f64).abs() < 1e-9, "sea_level mismatch, harness assumption broken");
    assert!((p.map_width_km - 800.0).abs() < 1e-9, "map_width_km mismatch, harness assumption broken");

    let wb = cartalith_civ::build_water_bodies(&ws.field, gw, gh, ws.sea_level, world, Some(&ws.rainfall));
    let biome = cartalith_civ::build_biome_raster(&wb.classification, &ws.temperature, &ws.rainfall);
    let lith = cartalith_civ::build_lithology(
        &ws.field, &ws.age_field, &ws.volcanic_field, &ws.crust_field, &ws.resistance_field, &ws.rainfall, ws.sea_level,
    );
    let slope_n = cartalith_civ::build_slope_field(&ws.field, gw, gh, world);
    let soil = cartalith_civ::build_soil_fertility(&lith, &ws.temperature, &ws.rainfall, &slope_n, &ws.age_field);
    let flow_thresh = cartalith_hydrology::river_flow_thresh(gw, gh, gw, p.map_width_km);
    let water = cartalith_civ::build_water_access(&ws.flow_discharge, &ws.field, gw, gh, ws.sea_level, flow_thresh);
    let k = cartalith_civ::build_carrying_capacity(&soil, &water, Some(&biome), &ws.temperature, &ws.field, ws.sea_level, 0.0, None);
    let npp = cartalith_civ::build_npp(&ws.temperature, &ws.rainfall, &ws.field, ws.sea_level, 3000.0);
    let dens = cartalith_civ::estimate_regional_density_km2(&k, &water, Some(&biome), Some(&npp), &ws.field, ws.sea_level, None);
    let res = cartalith_civ::build_resource_potentials(
        &lith,
        Some(&ws.boundary_type),
        Some(&ws.shear_field),
        Some(&ws.flow_discharge),
        Some(&biome),
        &ws.field,
        &ws.rainfall,
        &ws.age_field,
        gw,
        gh,
        ws.sea_level,
        Some(&ws.volcanic_field),
        true,
        false,
    );
    let ocean_dt = cartalith_civ::civ_ocean_dist_field(Some(&wb.classification), &ws.field, gw, gh, ws.sea_level);
    World { ws, biome, lith, water, dens, res, ocean_dt, flow_thresh }
}

/// The synthetic ownership rule, byte-for-byte identical to the harness's
/// (see this file's header). Territory only on land; both edge columns go
/// to `seam_faction`; every seventh column stays unclaimed; the rest cycle
/// through `1..=n_f-2`, so faction `n_f-1` never receives a single cell.
fn synthetic_territory(field: &[f32], gw: usize, gh: usize, sea: f64, n_f: usize, seam_faction: i32) -> Vec<i32> {
    let mut t = vec![0i32; gw * gh];
    for y in 0..gh {
        for x in 0..gw {
            let i = y * gw + x;
            if (field[i] as f64) < sea {
                continue;
            }
            t[i] = if x == 0 || x == gw - 1 {
                seam_faction
            } else if x % 7 == 0 {
                0
            } else {
                1 + ((x + y) % (n_f - 2)) as i32
            };
        }
    }
    t
}

fn place(
    faction: i32,
    pop: f64,
    kind: SettlementKind,
    trade_volume: f64,
    economic_importance: f64,
    specialisation: Option<&'static str>,
    fortified: bool,
) -> FactionPlace<'static> {
    FactionPlace { faction, pop, kind, trade_volume, economic_importance, specialisation, fortified }
}

/// The river axis reads `flowField[i] > riverFlowThresh(GW,GH)`, so what has
/// to agree between the two sides is that BOOLEAN, not the raw discharge.
fn river_mask(w: &World, gw: usize, gh: usize) -> Vec<u8> {
    (0..gw * gh)
        .map(|i| u8::from((w.ws.field[i] as f64) >= w.ws.sea_level && (w.ws.flow_discharge[i] as f64) > w.flow_thresh))
        .collect()
}

fn land_sum(v: &[f32], field: &[f32], sea: f64) -> f64 {
    v.iter().zip(field).filter(|&(_, &h)| (h as f64) >= sea).map(|(&x, _)| x as f64).sum()
}

fn fnv_bytes(bytes: &[u8]) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{h:x}")
}

fn fnv_f32(v: &[f32]) -> String {
    let mut bytes = Vec::with_capacity(v.len() * 4);
    for x in v {
        bytes.extend_from_slice(&x.to_le_bytes());
    }
    fnv_bytes(&bytes)
}

fn fnv_u8(v: &[u8]) -> String {
    fnv_bytes(v)
}

/// The reference's `civTerritory` is a `Uint8Array`; this port's is
/// `Vec<i32>`. Hashed as one byte per cell so the two are comparable --
/// legitimate because the faction index is `< 256` by construction.
fn fnv_terr(v: &[i32]) -> String {
    let bytes: Vec<u8> = v.iter().map(|&t| t as u8).collect();
    fnv_bytes(&bytes)
}

/// For values that are pure `f64` arithmetic over inputs this file has
/// already proved bit-identical (populations, trade volumes, tax, counts,
/// territory area, the terrain mix, the whole power breakdown).
#[track_caller]
fn near(got: f64, want: f64, what: &str) {
    near_rel(got, want, 1e-9, what);
}

#[track_caller]
fn near_rel(got: f64, want: f64, rel: f64, what: &str) {
    let tol = rel * want.abs().max(1.0);
    assert!((got - want).abs() <= tol, "{what}: got {got}, want {want} (tol {tol})");
}

#[track_caller]
fn near_abs(got: f64, want: f64, tol: f64, what: &str) {
    assert!((got - want).abs() <= tol, "{what}: got {got}, want {want} (tol {tol})");
}

#[test]
fn faction_aggregates_case_0_region_no_wrap() {
    // Case 0 (gw=24 gh=18 seed=24601 world=false): a small non-wrapping region
    // whose fixture SHAPE reaches the edges on purpose -- faction 6 has neither
    // territory nor settlements (aggregation over an empty faction), faction 2
    // has exactly one settlement, faction 4 has territory but no settlement at
    // all, faction 1 carries a zero-population hamlet, one place is a
    // non-settlement category (excluded upstream, so it never appears in the
    // slice below), and one carries an out-of-range faction id 99 (must be
    // skipped by the `f<0||f>=nF` guard -- it IS in the slice). Faction 2 also
    // owns both x=0 and x=gw-1, so its territory spans the map seam.
    // riverFlowThresh is degenerate at this size (every land cell clears it, so
    // world_mean_terrain.river == 1.0) -- a real property of small grids, and
    // case 1 is the one that exercises a discriminating threshold.
    let w = build_world(24, 18, 24601, false);
    let (gw, gh) = (24, 18);
    assert_eq!(fnv_f32(&w.ws.field), "b2f9815ad751080", "field hash: the harness world and this one are not the same world");
    assert_eq!(fnv_u8(&w.biome), "b341d60750895ae0", "biome raster hash");
    assert_eq!(fnv_u8(&river_mask(&w, gw, gh)), "79a17f2b33015faf", "river-mask hash");
    assert_eq!(fnv_f32(&w.ocean_dt), "c442e9d0f86bdee2", "ocean distance-transform hash");
    assert_eq!(fnv_u8(&w.lith), "cf2f3c988dccebfe", "lithology hash");
    assert_eq!(fnv_f32(&w.water), "dfc4465c7e3792a5", "water-access hash");
    near_rel(land_sum(&w.dens, &w.ws.field, w.ws.sea_level), 1855.1646017581224, 1e-6, "population-density land sum");
    near_rel(land_sum(&w.ws.flow_discharge, &w.ws.field, w.ws.sea_level), 1146.414378106594, 1e-6, "flow land sum");
    assert!((w.flow_thresh - 0.1728).abs() < 1e-12, "flow threshold");

    let territory = synthetic_territory(&w.ws.field, gw, gh, w.ws.sea_level, N_F, 2);
    assert_eq!(fnv_terr(&territory), "944c3195cfcc3acf", "territory hash: the two sides did not build the same raster");
    let land = w.ws.field.iter().filter(|&&h| (h as f64) >= w.ws.sea_level).count();
    assert_eq!(land, 246, "land-cell count");
    assert!(land > 0 && territory.iter().any(|&t| t > 0), "fixture is not silently empty");
    let mut terr_count = [0usize; N_F];
    for &t in &territory { if t > 0 && (t as usize) < N_F { terr_count[t as usize] += 1; } }
    assert_eq!(terr_count, [0, 37, 69, 40, 37, 37, 0], "per-faction territory-cell counts");

    let places = [
        place(1, 15000.0, SettlementKind::Capital, 900.0, 0.8, Some("grain"), true),
        place(1, 1500.0, SettlementKind::Town, 210.0, 0.35, Some("mining"), true),
        place(1, 0.0, SettlementKind::Hamlet, 0.0, 0.0, Some("none"), false),
        place(2, 6000.0, SettlementKind::City, 500.0, 0.6, Some("fishing"), true),
        place(3, 400.0, SettlementKind::Village, 60.0, 0.2, Some("trade_hub"), false),
        place(5, 120.0, SettlementKind::Hamlet, 5.0, 0.05, Some("pastoral"), false),
        place(99, 4242.0, SettlementKind::Town, 0.0, 0.0, None, true),
    ];
    let input = FactionAggregatesInput {
        faction_count: N_F, gw, gh, sea: w.ws.sea_level, map_width_km: 800.0,
        field: &w.ws.field, territory: Some(&territory), density: Some(&w.dens),
        resources: Some(&w.res), biome: Some(&w.biome), flow: Some(&w.ws.flow_discharge),
        flow_thresh: w.flow_thresh, ocean_dist: Some(&w.ocean_dt), faction_has_religion: None,
    };
    let agg = cartalith_civ::civ_faction_aggregates(&input, &places);

    near(agg.max_pop, 16500.0, "max_pop");
    near(agg.max_trade_volume, 1110.0, "max_trade_volume");
    near(agg.max_territory_km2, 76667.0, "max_territory_km2");
    assert_eq!(agg.max_settlement_count, 3, "max_settlement_count");
    near(agg.world_mean_terrain["river"], 1.0, "world_mean_terrain.river");
    near(agg.world_mean_terrain["coast"], 0.24796747967479674, "world_mean_terrain.coast");
    near(agg.world_mean_terrain["arid"], 0.06504065040650407, "world_mean_terrain.arid");
    near(agg.world_mean_terrain["forest"], 0.6463414634146342, "world_mean_terrain.forest");
    near(agg.world_mean_terrain["hills"], 0.4065040650406504, "world_mean_terrain.hills");
    near_rel(agg.world_mean_resource["copper"], 0.17730051544835654, 1e-6, "world_mean_resource.copper");
    near_rel(agg.world_mean_resource["tin"], 0.15508129708166044, 1e-6, "world_mean_resource.tin");
    near_rel(agg.world_mean_resource["iron"], 0.08089430824043305, 1e-6, "world_mean_resource.iron");
    near_rel(agg.world_mean_resource["gold"], 0.16925468798575363, 1e-6, "world_mean_resource.gold");
    near_rel(agg.world_mean_resource["salt"], 0.0021797791729128457, 1e-6, "world_mean_resource.salt");
    near_rel(agg.world_mean_resource["timber"], 0.6778520701861963, 1e-6, "world_mean_resource.timber");
    near_rel(agg.world_mean_resource["lead"], 0.26321138332529764, 1e-6, "world_mean_resource.lead");
    near_rel(agg.world_mean_resource["silver"], 0.0648373997792965, 1e-6, "world_mean_resource.silver");
    near_rel(agg.world_mean_resource["clay"], 0.4838379827456746, 1e-6, "world_mean_resource.clay");
    near_rel(agg.world_mean_resource["buildstone"], 0.3103658566629984, 1e-6, "world_mean_resource.buildstone");
    near_rel(agg.world_mean_resource["flint"], 0.1317073223067493, 1e-6, "world_mean_resource.flint");
    near_rel(agg.world_mean_resource["obsidian"], 0.02356669185607414, 1e-6, "world_mean_resource.obsidian");
    near_rel(agg.world_mean_resource["gems"], 0.16280487833953486, 1e-6, "world_mean_resource.gems");
    near_rel(agg.world_mean_resource["sulfur"], 0.037031304303223524, 1e-6, "world_mean_resource.sulfur");
    near_rel(agg.world_mean_resource["alum"], 0.0359711014642948, 1e-6, "world_mean_resource.alum");

    // ---- faction 0 ----
    let b = &agg.by_faction[0];
    near(b.pop, 0.0, "f0.pop");
    near(b.territory_km2, 0.0, "f0.territory_km2");
    near(b.trade_volume, 0.0, "f0.trade_volume");
    near(b.tax_income, 0.0, "f0.tax_income");
    near(b.mean_importance, 0.0, "f0.mean_importance");
    near(b.fortified_fraction, 0.0, "f0.fortified_fraction");
    near(b.craft_share, 0.0, "f0.craft_share");
    near_abs(b.food_production_capacity, 0.0, 1.0, "f0.food_production_capacity");
    near_abs(b.food_surplus, 0.0, 1.0, "f0.food_surplus");
    assert_eq!(b.settlement_count, 0, "f0.settlement_count");
    assert_eq!(b.capital, None, "f0.capital");
    near(b.power.military, 0.0, "f0.power.military");
    near(b.power.economic, 0.0, "f0.power.economic");
    near(b.power.political, 0.0, "f0.power.political");
    near(b.power.cultural, 0.0, "f0.power.cultural");
    near(b.power.religious, 0.0, "f0.power.religious");
    near(b.power.overall, 0.0, "f0.power.overall");
    near(b.sector_output.fishing, 0.0, "f0.sector.fishing");
    near(b.sector_output.agriculture, 0.0, "f0.sector.agriculture");
    near(b.sector_output.livestock, 0.0, "f0.sector.livestock");
    near(b.sector_output.forestry, 0.0, "f0.sector.forestry");
    near(b.sector_output.mining, 0.0, "f0.sector.mining");
    near(b.sector_output.craft, 0.0, "f0.sector.craft");
    near(b.terrain_mix["river"], 0.0, "f0.terrain_mix.river");
    near(b.terrain_mix["coast"], 0.0, "f0.terrain_mix.coast");
    near(b.terrain_mix["arid"], 0.0, "f0.terrain_mix.arid");
    near(b.terrain_mix["forest"], 0.0, "f0.terrain_mix.forest");
    near(b.terrain_mix["hills"], 0.0, "f0.terrain_mix.hills");
    near_rel(b.resource_potential["copper"], 0.0, 1e-6, "f0.resource.copper");
    near_rel(b.resource_potential["tin"], 0.0, 1e-6, "f0.resource.tin");
    near_rel(b.resource_potential["iron"], 0.0, 1e-6, "f0.resource.iron");
    near_rel(b.resource_potential["gold"], 0.0, 1e-6, "f0.resource.gold");
    near_rel(b.resource_potential["salt"], 0.0, 1e-6, "f0.resource.salt");
    near_rel(b.resource_potential["timber"], 0.0, 1e-6, "f0.resource.timber");
    near_rel(b.resource_potential["lead"], 0.0, 1e-6, "f0.resource.lead");
    near_rel(b.resource_potential["silver"], 0.0, 1e-6, "f0.resource.silver");
    near_rel(b.resource_potential["clay"], 0.0, 1e-6, "f0.resource.clay");
    near_rel(b.resource_potential["buildstone"], 0.0, 1e-6, "f0.resource.buildstone");
    near_rel(b.resource_potential["flint"], 0.0, 1e-6, "f0.resource.flint");
    near_rel(b.resource_potential["obsidian"], 0.0, 1e-6, "f0.resource.obsidian");
    near_rel(b.resource_potential["gems"], 0.0, 1e-6, "f0.resource.gems");
    near_rel(b.resource_potential["sulfur"], 0.0, 1e-6, "f0.resource.sulfur");
    near_rel(b.resource_potential["alum"], 0.0, 1e-6, "f0.resource.alum");
    assert_eq!(b.exports, Vec::<&str>::new(), "f0.exports");
    assert_eq!(b.imports, vec!["copper", "iron", "salt", "timber", "lead", "clay", "buildstone", "alum"], "f0.imports");
    assert_eq!(b.strategic_resources, Vec::<&str>::new(), "f0.strategic");

    // ---- faction 1 ----
    let b = &agg.by_faction[1];
    near(b.pop, 16500.0, "f1.pop");
    near(b.territory_km2, 41111.0, "f1.territory_km2");
    near(b.trade_volume, 1110.0, "f1.trade_volume");
    near(b.tax_income, 1425.0, "f1.tax_income");
    near(b.mean_importance, 0.3833333333333333, "f1.mean_importance");
    near(b.fortified_fraction, 0.6666666666666666, "f1.fortified_fraction");
    near(b.craft_share, 0.0, "f1.craft_share");
    near_abs(b.food_production_capacity, 359769.0, 1.0, "f1.food_production_capacity");
    near_abs(b.food_surplus, 343269.0, 1.0, "f1.food_surplus");
    assert_eq!(b.settlement_count, 3, "f1.settlement_count");
    assert_eq!(b.capital, Some(0), "f1.capital");
    near(b.power.military, 84.33333333333334, "f1.power.military");
    near(b.power.economic, 81.5, "f1.power.economic");
    near(b.power.political, 68.51798361746255, "f1.power.political");
    near(b.power.cultural, 100.0, "f1.power.cultural");
    near(b.power.religious, 0.0, "f1.power.religious");
    near(b.power.overall, 66.87026339015918, "f1.power.overall");
    near(b.sector_output.fishing, 0.0, "f1.sector.fishing");
    near(b.sector_output.agriculture, 13200.0, "f1.sector.agriculture");
    near(b.sector_output.livestock, 0.0, "f1.sector.livestock");
    near(b.sector_output.forestry, 0.0, "f1.sector.forestry");
    near(b.sector_output.mining, 915.0, "f1.sector.mining");
    near(b.sector_output.craft, 0.0, "f1.sector.craft");
    near(b.terrain_mix["river"], 1.0, "f1.terrain_mix.river");
    near(b.terrain_mix["coast"], 0.2702702702702703, "f1.terrain_mix.coast");
    near(b.terrain_mix["arid"], 0.13513513513513514, "f1.terrain_mix.arid");
    near(b.terrain_mix["forest"], 0.6486486486486487, "f1.terrain_mix.forest");
    near(b.terrain_mix["hills"], 0.3783783783783784, "f1.terrain_mix.hills");
    near_rel(b.resource_potential["copper"], 0.17997947612122908, 1e-6, "f1.resource.copper");
    near_rel(b.resource_potential["tin"], 0.16486486067643036, 1e-6, "f1.resource.tin");
    near_rel(b.resource_potential["iron"], 0.07162162178271525, 1e-6, "f1.resource.iron");
    near_rel(b.resource_potential["gold"], 0.15851931636397903, 1e-6, "f1.resource.gold");
    near_rel(b.resource_potential["salt"], 0.0, 1e-6, "f1.resource.salt");
    near_rel(b.resource_potential["timber"], 0.6874778560689978, 1e-6, "f1.resource.timber");
    near_rel(b.resource_potential["lead"], 0.3297297310184788, 1e-6, "f1.resource.lead");
    near_rel(b.resource_potential["silver"], 0.0891891911223128, 1e-6, "f1.resource.silver");
    near_rel(b.resource_potential["clay"], 0.5232512967006581, 1e-6, "f1.resource.clay");
    near_rel(b.resource_potential["buildstone"], 0.343243247753865, 1e-6, "f1.resource.buildstone");
    near_rel(b.resource_potential["flint"], 0.16216216860590754, 1e-6, "f1.resource.flint");
    near_rel(b.resource_potential["obsidian"], 0.025029588390041043, 1e-6, "f1.resource.obsidian");
    near_rel(b.resource_potential["gems"], 0.18243243243243243, 1e-6, "f1.resource.gems");
    near_rel(b.resource_potential["sulfur"], 0.02472228939468796, 1e-6, "f1.resource.sulfur");
    near_rel(b.resource_potential["alum"], 0.033868868608732484, 1e-6, "f1.resource.alum");
    assert_eq!(b.exports, vec!["silver", "food"], "f1.exports");
    assert_eq!(b.imports, vec!["salt"], "f1.imports");
    assert_eq!(b.strategic_resources, vec!["timber", "clay"], "f1.strategic");

    // ---- faction 2 ----
    let b = &agg.by_faction[2];
    near(b.pop, 6000.0, "f2.pop");
    near(b.territory_km2, 76667.0, "f2.territory_km2");
    near(b.trade_volume, 500.0, "f2.trade_volume");
    near(b.tax_income, 420.0, "f2.tax_income");
    near(b.mean_importance, 0.6, "f2.mean_importance");
    near(b.fortified_fraction, 1.0, "f2.fortified_fraction");
    near(b.craft_share, 0.0, "f2.craft_share");
    near_abs(b.food_production_capacity, 519405.0, 1.0, "f2.food_production_capacity");
    near_abs(b.food_surplus, 513405.0, 1.0, "f2.food_surplus");
    assert_eq!(b.settlement_count, 1, "f2.settlement_count");
    assert_eq!(b.capital, Some(3), "f2.capital");
    near(b.power.military, 63.36363636363637, "f2.power.military");
    near(b.power.economic, 46.92710892710893, "f2.power.economic");
    near(b.power.political, 68.66666666666667, "f2.power.political");
    near(b.power.cultural, 35.45454545454545, "f2.power.cultural");
    near(b.power.religious, 0.0, "f2.power.religious");
    near(b.power.overall, 42.88239148239148, "f2.power.overall");
    near(b.sector_output.fishing, 4560.0, "f2.sector.fishing");
    near(b.sector_output.agriculture, 0.0, "f2.sector.agriculture");
    near(b.sector_output.livestock, 0.0, "f2.sector.livestock");
    near(b.sector_output.forestry, 0.0, "f2.sector.forestry");
    near(b.sector_output.mining, 0.0, "f2.sector.mining");
    near(b.sector_output.craft, 0.0, "f2.sector.craft");
    near(b.terrain_mix["river"], 1.0, "f2.terrain_mix.river");
    near(b.terrain_mix["coast"], 0.21739130434782608, "f2.terrain_mix.coast");
    near(b.terrain_mix["arid"], 0.043478260869565216, "f2.terrain_mix.arid");
    near(b.terrain_mix["forest"], 0.6666666666666666, "f2.terrain_mix.forest");
    near(b.terrain_mix["hills"], 0.4927536231884058, "f2.terrain_mix.hills");
    near_rel(b.resource_potential["copper"], 0.1724929709351905, 1e-6, "f2.resource.copper");
    near_rel(b.resource_potential["tin"], 0.18188405382460443, 1e-6, "f2.resource.tin");
    near_rel(b.resource_potential["iron"], 0.10652173731638037, 1e-6, "f2.resource.iron");
    near_rel(b.resource_potential["gold"], 0.21756906353909036, 1e-6, "f2.resource.gold");
    near_rel(b.resource_potential["salt"], 0.007771386616471885, 1e-6, "f2.resource.salt");
    near_rel(b.resource_potential["timber"], 0.707384252893752, 1e-6, "f2.resource.timber");
    near_rel(b.resource_potential["lead"], 0.25000000086383545, 1e-6, "f2.resource.lead");
    near_rel(b.resource_potential["silver"], 0.07173913198968639, 1e-6, "f2.resource.silver");
    near_rel(b.resource_potential["clay"], 0.4234047722125399, 1e-6, "f2.resource.clay");
    near_rel(b.resource_potential["buildstone"], 0.3340579729149307, 1e-6, "f2.resource.buildstone");
    near_rel(b.resource_potential["flint"], 0.12173913527226103, 1e-6, "f2.resource.flint");
    near_rel(b.resource_potential["obsidian"], 0.03438016729078431, 1e-6, "f2.resource.obsidian");
    near_rel(b.resource_potential["gems"], 0.1659420296765756, 1e-6, "f2.resource.gems");
    near_rel(b.resource_potential["sulfur"], 0.0434684157371521, 1e-6, "f2.resource.sulfur");
    near_rel(b.resource_potential["alum"], 0.03477473276248877, 1e-6, "f2.resource.alum");
    assert_eq!(b.exports, vec!["obsidian", "food"], "f2.exports");
    assert_eq!(b.imports, Vec::<&str>::new(), "f2.imports");
    assert_eq!(b.strategic_resources, vec!["timber", "clay"], "f2.strategic");

    // ---- faction 3 ----
    let b = &agg.by_faction[3];
    near(b.pop, 400.0, "f3.pop");
    near(b.territory_km2, 44444.0, "f3.territory_km2");
    near(b.trade_volume, 60.0, "f3.trade_volume");
    near(b.tax_income, 12.0, "f3.tax_income");
    near(b.mean_importance, 0.2, "f3.mean_importance");
    near(b.fortified_fraction, 0.0, "f3.fortified_fraction");
    near(b.craft_share, 1.0, "f3.craft_share");
    near_abs(b.food_production_capacity, 338853.0, 1.0, "f3.food_production_capacity");
    near_abs(b.food_surplus, 338453.0, 1.0, "f3.food_surplus");
    assert_eq!(b.settlement_count, 1, "f3.settlement_count");
    assert_eq!(b.capital, Some(4), "f3.capital");
    near(b.power.military, 5.090909090909092, "f3.power.military");
    near(b.power.economic, 8.889434889434888, "f3.power.economic");
    near(b.power.political, 35.95623062508424, "f3.power.political");
    near(b.power.cultural, 11.696969696969695, "f3.power.cultural");
    near(b.power.religious, 0.0, "f3.power.religious");
    near(b.power.overall, 12.326708860479583, "f3.power.overall");
    near(b.sector_output.fishing, 0.0, "f3.sector.fishing");
    near(b.sector_output.agriculture, 0.0, "f3.sector.agriculture");
    near(b.sector_output.livestock, 0.0, "f3.sector.livestock");
    near(b.sector_output.forestry, 0.0, "f3.sector.forestry");
    near(b.sector_output.mining, 0.0, "f3.sector.mining");
    near(b.sector_output.craft, 208.0, "f3.sector.craft");
    near(b.terrain_mix["river"], 1.0, "f3.terrain_mix.river");
    near(b.terrain_mix["coast"], 0.275, "f3.terrain_mix.coast");
    near(b.terrain_mix["arid"], 0.0, "f3.terrain_mix.arid");
    near(b.terrain_mix["forest"], 0.625, "f3.terrain_mix.forest");
    near(b.terrain_mix["hills"], 0.375, "f3.terrain_mix.hills");
    near_rel(b.resource_potential["copper"], 0.16916068901773543, 1e-6, "f3.resource.copper");
    near_rel(b.resource_potential["tin"], 0.1474999964237213, 1e-6, "f3.resource.tin");
    near_rel(b.resource_potential["iron"], 0.06749999932944775, 1e-6, "f3.resource.iron");
    near_rel(b.resource_potential["gold"], 0.20001388490200042, 1e-6, "f3.resource.gold");
    near_rel(b.resource_potential["salt"], 0.0, 1e-6, "f3.resource.salt");
    near_rel(b.resource_potential["timber"], 0.6516346678137779, 1e-6, "f3.resource.timber");
    near_rel(b.resource_potential["lead"], 0.22250000089406968, 1e-6, "f3.resource.lead");
    near_rel(b.resource_potential["silver"], 0.0550000011920929, 1e-6, "f3.resource.silver");
    near_rel(b.resource_potential["clay"], 0.48760131895542147, 1e-6, "f3.resource.clay");
    near_rel(b.resource_potential["buildstone"], 0.27625000178813935, 1e-6, "f3.resource.buildstone");
    near_rel(b.resource_potential["flint"], 0.10500000417232513, 1e-6, "f3.resource.flint");
    near_rel(b.resource_potential["obsidian"], 0.040552057325839996, 1e-6, "f3.resource.obsidian");
    near_rel(b.resource_potential["gems"], 0.1512500002980232, 1e-6, "f3.resource.gems");
    near_rel(b.resource_potential["sulfur"], 0.056063024699687956, 1e-6, "f3.resource.sulfur");
    near_rel(b.resource_potential["alum"], 0.05795342773199082, 1e-6, "f3.resource.alum");
    assert_eq!(b.exports, vec!["obsidian", "sulfur", "alum", "food"], "f3.exports");
    assert_eq!(b.imports, vec!["salt"], "f3.imports");
    assert_eq!(b.strategic_resources, vec!["timber", "clay"], "f3.strategic");

    // ---- faction 4 ----
    let b = &agg.by_faction[4];
    near(b.pop, 0.0, "f4.pop");
    near(b.territory_km2, 41111.0, "f4.territory_km2");
    near(b.trade_volume, 0.0, "f4.trade_volume");
    near(b.tax_income, 0.0, "f4.tax_income");
    near(b.mean_importance, 0.0, "f4.mean_importance");
    near(b.fortified_fraction, 0.0, "f4.fortified_fraction");
    near(b.craft_share, 0.0, "f4.craft_share");
    near_abs(b.food_production_capacity, 327293.0, 1.0, "f4.food_production_capacity");
    near_abs(b.food_surplus, 327293.0, 1.0, "f4.food_surplus");
    assert_eq!(b.settlement_count, 0, "f4.settlement_count");
    assert_eq!(b.capital, None, "f4.capital");
    near(b.power.military, 0.0, "f4.power.military");
    near(b.power.economic, 0.0, "f4.power.economic");
    near(b.power.political, 18.767983617462534, "f4.power.political");
    near(b.power.cultural, 0.0, "f4.power.cultural");
    near(b.power.religious, 0.0, "f4.power.religious");
    near(b.power.overall, 3.7535967234925067, "f4.power.overall");
    near(b.sector_output.fishing, 0.0, "f4.sector.fishing");
    near(b.sector_output.agriculture, 0.0, "f4.sector.agriculture");
    near(b.sector_output.livestock, 0.0, "f4.sector.livestock");
    near(b.sector_output.forestry, 0.0, "f4.sector.forestry");
    near(b.sector_output.mining, 0.0, "f4.sector.mining");
    near(b.sector_output.craft, 0.0, "f4.sector.craft");
    near(b.terrain_mix["river"], 1.0, "f4.terrain_mix.river");
    near(b.terrain_mix["coast"], 0.21621621621621623, "f4.terrain_mix.coast");
    near(b.terrain_mix["arid"], 0.05405405405405406, "f4.terrain_mix.arid");
    near(b.terrain_mix["forest"], 0.5675675675675675, "f4.terrain_mix.forest");
    near(b.terrain_mix["hills"], 0.4864864864864865, "f4.terrain_mix.hills");
    near_rel(b.resource_potential["copper"], 0.16151368843290853, 1e-6, "f4.resource.copper");
    near_rel(b.resource_potential["tin"], 0.17837837418994387, 1e-6, "f4.resource.tin");
    near_rel(b.resource_potential["iron"], 0.08513513368529242, 1e-6, "f4.resource.iron");
    near_rel(b.resource_potential["gold"], 0.1740894317626953, 1e-6, "f4.resource.gold");
    near_rel(b.resource_potential["salt"], 0.0, 1e-6, "f4.resource.salt");
    near_rel(b.resource_potential["timber"], 0.5845396099863825, 1e-6, "f4.resource.timber");
    near_rel(b.resource_potential["lead"], 0.21351351448007533, 1e-6, "f4.resource.lead");
    near_rel(b.resource_potential["silver"], 0.0445945955611564, 1e-6, "f4.resource.silver");
    near_rel(b.resource_potential["clay"], 0.4634788874033335, 1e-6, "f4.resource.clay");
    near_rel(b.resource_potential["buildstone"], 0.2445945965277182, 1e-6, "f4.resource.buildstone");
    near_rel(b.resource_potential["flint"], 0.09729730116354453, 1e-6, "f4.resource.flint");
    near_rel(b.resource_potential["obsidian"], 0.023702637569324392, 1e-6, "f4.resource.obsidian");
    near_rel(b.resource_potential["gems"], 0.20405405437624133, 1e-6, "f4.resource.gems");
    near_rel(b.resource_potential["sulfur"], 0.061170112442325904, 1e-6, "f4.resource.sulfur");
    near_rel(b.resource_potential["alum"], 0.06287234860497552, 1e-6, "f4.resource.alum");
    assert_eq!(b.exports, vec!["sulfur", "alum", "food"], "f4.exports");
    assert_eq!(b.imports, vec!["salt"], "f4.imports");
    assert_eq!(b.strategic_resources, vec!["timber", "clay"], "f4.strategic");

    // ---- faction 5 ----
    let b = &agg.by_faction[5];
    near(b.pop, 120.0, "f5.pop");
    near(b.territory_km2, 41111.0, "f5.territory_km2");
    near(b.trade_volume, 5.0, "f5.trade_volume");
    near(b.tax_income, 2.0, "f5.tax_income");
    near(b.mean_importance, 0.05, "f5.mean_importance");
    near(b.fortified_fraction, 0.0, "f5.fortified_fraction");
    near(b.craft_share, 0.0, "f5.craft_share");
    near_abs(b.food_production_capacity, 323522.0, 1.0, "f5.food_production_capacity");
    near_abs(b.food_surplus, 323402.0, 1.0, "f5.food_surplus");
    assert_eq!(b.settlement_count, 1, "f5.settlement_count");
    assert_eq!(b.capital, Some(5), "f5.capital");
    near(b.power.military, 0.32727272727272727, "f5.power.military");
    near(b.power.economic, 1.8983619983619984, "f5.power.economic");
    near(b.power.political, 26.1846502841292, "f5.power.political");
    near(b.power.cultural, 10.509090909090908, "f5.power.cultural");
    near(b.power.religious, 0.0, "f5.power.religious");
    near(b.power.overall, 7.783875183770968, "f5.power.overall");
    near(b.sector_output.fishing, 0.0, "f5.sector.fishing");
    near(b.sector_output.agriculture, 0.0, "f5.sector.agriculture");
    near(b.sector_output.livestock, 51.60000000000001, "f5.sector.livestock");
    near(b.sector_output.forestry, 0.0, "f5.sector.forestry");
    near(b.sector_output.mining, 0.0, "f5.sector.mining");
    near(b.sector_output.craft, 0.0, "f5.sector.craft");
    near(b.terrain_mix["river"], 1.0, "f5.terrain_mix.river");
    near(b.terrain_mix["coast"], 0.24324324324324326, "f5.terrain_mix.coast");
    near(b.terrain_mix["arid"], 0.08108108108108109, "f5.terrain_mix.arid");
    near(b.terrain_mix["forest"], 0.7027027027027027, "f5.terrain_mix.forest");
    near(b.terrain_mix["hills"], 0.43243243243243246, "f5.terrain_mix.hills");
    near_rel(b.resource_potential["copper"], 0.181421662202558, 1e-6, "f5.resource.copper");
    near_rel(b.resource_potential["tin"], 0.16486486067643036, 1e-6, "f5.resource.tin");
    near_rel(b.resource_potential["iron"], 0.06621621629676304, 1e-6, "f5.resource.iron");
    near_rel(b.resource_potential["gold"], 0.10215784569044371, 1e-6, "f5.resource.gold");
    near_rel(b.resource_potential["salt"], 0.0, 1e-6, "f5.resource.salt");
    near_rel(b.resource_potential["timber"], 0.7080320796451053, 1e-6, "f5.resource.timber");
    near_rel(b.resource_potential["lead"], 0.29729729890823364, 1e-6, "f5.resource.lead");
    near_rel(b.resource_potential["silver"], 0.059459460748208535, 1e-6, "f5.resource.silver");
    near_rel(b.resource_potential["clay"], 0.4989442728661202, 1e-6, "f5.resource.clay");
    near_rel(b.resource_potential["buildstone"], 0.30135135553978587, 1e-6, "f5.resource.buildstone");
    near_rel(b.resource_potential["flint"], 0.1459459517453168, 1e-6, "f5.resource.flint");
    near_rel(b.resource_potential["obsidian"], 0.0, 1e-6, "f5.resource.obsidian");
    near_rel(b.resource_potential["gems"], 0.18243243243243243, 1e-6, "f5.resource.gems");
    near_rel(b.resource_potential["sulfur"], 0.018644332885742188, 1e-6, "f5.resource.sulfur");
    near_rel(b.resource_potential["alum"], 0.01491546630859375, 1e-6, "f5.resource.alum");
    assert_eq!(b.exports, vec!["food"], "f5.exports");
    assert_eq!(b.imports, vec!["salt", "alum"], "f5.imports");
    assert_eq!(b.strategic_resources, vec!["timber", "clay"], "f5.strategic");

    // ---- faction 6 ----
    let b = &agg.by_faction[6];
    near(b.pop, 0.0, "f6.pop");
    near(b.territory_km2, 0.0, "f6.territory_km2");
    near(b.trade_volume, 0.0, "f6.trade_volume");
    near(b.tax_income, 0.0, "f6.tax_income");
    near(b.mean_importance, 0.0, "f6.mean_importance");
    near(b.fortified_fraction, 0.0, "f6.fortified_fraction");
    near(b.craft_share, 0.0, "f6.craft_share");
    near_abs(b.food_production_capacity, 0.0, 1.0, "f6.food_production_capacity");
    near_abs(b.food_surplus, 0.0, 1.0, "f6.food_surplus");
    assert_eq!(b.settlement_count, 0, "f6.settlement_count");
    assert_eq!(b.capital, None, "f6.capital");
    near(b.power.military, 0.0, "f6.power.military");
    near(b.power.economic, 0.0, "f6.power.economic");
    near(b.power.political, 0.0, "f6.power.political");
    near(b.power.cultural, 0.0, "f6.power.cultural");
    near(b.power.religious, 0.0, "f6.power.religious");
    near(b.power.overall, 0.0, "f6.power.overall");
    near(b.sector_output.fishing, 0.0, "f6.sector.fishing");
    near(b.sector_output.agriculture, 0.0, "f6.sector.agriculture");
    near(b.sector_output.livestock, 0.0, "f6.sector.livestock");
    near(b.sector_output.forestry, 0.0, "f6.sector.forestry");
    near(b.sector_output.mining, 0.0, "f6.sector.mining");
    near(b.sector_output.craft, 0.0, "f6.sector.craft");
    near(b.terrain_mix["river"], 0.0, "f6.terrain_mix.river");
    near(b.terrain_mix["coast"], 0.0, "f6.terrain_mix.coast");
    near(b.terrain_mix["arid"], 0.0, "f6.terrain_mix.arid");
    near(b.terrain_mix["forest"], 0.0, "f6.terrain_mix.forest");
    near(b.terrain_mix["hills"], 0.0, "f6.terrain_mix.hills");
    near_rel(b.resource_potential["copper"], 0.0, 1e-6, "f6.resource.copper");
    near_rel(b.resource_potential["tin"], 0.0, 1e-6, "f6.resource.tin");
    near_rel(b.resource_potential["iron"], 0.0, 1e-6, "f6.resource.iron");
    near_rel(b.resource_potential["gold"], 0.0, 1e-6, "f6.resource.gold");
    near_rel(b.resource_potential["salt"], 0.0, 1e-6, "f6.resource.salt");
    near_rel(b.resource_potential["timber"], 0.0, 1e-6, "f6.resource.timber");
    near_rel(b.resource_potential["lead"], 0.0, 1e-6, "f6.resource.lead");
    near_rel(b.resource_potential["silver"], 0.0, 1e-6, "f6.resource.silver");
    near_rel(b.resource_potential["clay"], 0.0, 1e-6, "f6.resource.clay");
    near_rel(b.resource_potential["buildstone"], 0.0, 1e-6, "f6.resource.buildstone");
    near_rel(b.resource_potential["flint"], 0.0, 1e-6, "f6.resource.flint");
    near_rel(b.resource_potential["obsidian"], 0.0, 1e-6, "f6.resource.obsidian");
    near_rel(b.resource_potential["gems"], 0.0, 1e-6, "f6.resource.gems");
    near_rel(b.resource_potential["sulfur"], 0.0, 1e-6, "f6.resource.sulfur");
    near_rel(b.resource_potential["alum"], 0.0, 1e-6, "f6.resource.alum");
    assert_eq!(b.exports, Vec::<&str>::new(), "f6.exports");
    assert_eq!(b.imports, vec!["copper", "iron", "salt", "timber", "lead", "clay", "buildstone", "alum"], "f6.imports");
    assert_eq!(b.strategic_resources, Vec::<&str>::new(), "f6.strategic");

    // ---- `civ_culture_terrain_fit` over the real Territory Fit output ----
    let fit = civ_culture_terrain_fit("highland", &agg.by_faction[0].terrain_mix, &agg.world_mean_terrain).expect("highland/f0");
    assert_eq!(fit.key, "hills"); near(fit.value, 0.0, "highland/f0.value"); near(fit.world_mean, 0.4065040650406504, "highland/f0.world_mean"); near(fit.ratio, 0.0, "highland/f0.ratio"); assert_eq!(fit.verdict, "mismatch", "highland/f0.verdict");
    let fit = civ_culture_terrain_fit("highland", &agg.by_faction[1].terrain_mix, &agg.world_mean_terrain).expect("highland/f1");
    assert_eq!(fit.key, "hills"); near(fit.value, 0.3783783783783784, "highland/f1.value"); near(fit.world_mean, 0.4065040650406504, "highland/f1.world_mean"); near(fit.ratio, 0.9308108108108109, "highland/f1.ratio"); assert_eq!(fit.verdict, "typical", "highland/f1.verdict");
    let fit = civ_culture_terrain_fit("highland", &agg.by_faction[2].terrain_mix, &agg.world_mean_terrain).expect("highland/f2");
    assert_eq!(fit.key, "hills"); near(fit.value, 0.4927536231884058, "highland/f2.value"); near(fit.world_mean, 0.4065040650406504, "highland/f2.world_mean"); near(fit.ratio, 1.2121739130434783, "highland/f2.ratio"); assert_eq!(fit.verdict, "match", "highland/f2.verdict");
    let fit = civ_culture_terrain_fit("highland", &agg.by_faction[3].terrain_mix, &agg.world_mean_terrain).expect("highland/f3");
    assert_eq!(fit.key, "hills"); near(fit.value, 0.375, "highland/f3.value"); near(fit.world_mean, 0.4065040650406504, "highland/f3.world_mean"); near(fit.ratio, 0.9225, "highland/f3.ratio"); assert_eq!(fit.verdict, "typical", "highland/f3.verdict");
    let fit = civ_culture_terrain_fit("highland", &agg.by_faction[4].terrain_mix, &agg.world_mean_terrain).expect("highland/f4");
    assert_eq!(fit.key, "hills"); near(fit.value, 0.4864864864864865, "highland/f4.value"); near(fit.world_mean, 0.4065040650406504, "highland/f4.world_mean"); near(fit.ratio, 1.1967567567567567, "highland/f4.ratio"); assert_eq!(fit.verdict, "match", "highland/f4.verdict");
    let fit = civ_culture_terrain_fit("highland", &agg.by_faction[5].terrain_mix, &agg.world_mean_terrain).expect("highland/f5");
    assert_eq!(fit.key, "hills"); near(fit.value, 0.43243243243243246, "highland/f5.value"); near(fit.world_mean, 0.4065040650406504, "highland/f5.world_mean"); near(fit.ratio, 1.0637837837837838, "highland/f5.ratio"); assert_eq!(fit.verdict, "typical", "highland/f5.verdict");
    let fit = civ_culture_terrain_fit("highland", &agg.by_faction[6].terrain_mix, &agg.world_mean_terrain).expect("highland/f6");
    assert_eq!(fit.key, "hills"); near(fit.value, 0.0, "highland/f6.value"); near(fit.world_mean, 0.4065040650406504, "highland/f6.world_mean"); near(fit.ratio, 0.0, "highland/f6.ratio"); assert_eq!(fit.verdict, "mismatch", "highland/f6.verdict");
    let fit = civ_culture_terrain_fit("desert", &agg.by_faction[0].terrain_mix, &agg.world_mean_terrain).expect("desert/f0");
    assert_eq!(fit.key, "arid"); near(fit.value, 0.0, "desert/f0.value"); near(fit.world_mean, 0.06504065040650407, "desert/f0.world_mean"); near(fit.ratio, 0.0, "desert/f0.ratio"); assert_eq!(fit.verdict, "mismatch", "desert/f0.verdict");
    let fit = civ_culture_terrain_fit("desert", &agg.by_faction[1].terrain_mix, &agg.world_mean_terrain).expect("desert/f1");
    assert_eq!(fit.key, "arid"); near(fit.value, 0.13513513513513514, "desert/f1.value"); near(fit.world_mean, 0.06504065040650407, "desert/f1.world_mean"); near(fit.ratio, 2.0777027027027026, "desert/f1.ratio"); assert_eq!(fit.verdict, "match", "desert/f1.verdict");
    let fit = civ_culture_terrain_fit("desert", &agg.by_faction[2].terrain_mix, &agg.world_mean_terrain).expect("desert/f2");
    assert_eq!(fit.key, "arid"); near(fit.value, 0.043478260869565216, "desert/f2.value"); near(fit.world_mean, 0.06504065040650407, "desert/f2.world_mean"); near(fit.ratio, 0.6684782608695651, "desert/f2.ratio"); assert_eq!(fit.verdict, "mismatch", "desert/f2.verdict");
    let fit = civ_culture_terrain_fit("desert", &agg.by_faction[3].terrain_mix, &agg.world_mean_terrain).expect("desert/f3");
    assert_eq!(fit.key, "arid"); near(fit.value, 0.0, "desert/f3.value"); near(fit.world_mean, 0.06504065040650407, "desert/f3.world_mean"); near(fit.ratio, 0.0, "desert/f3.ratio"); assert_eq!(fit.verdict, "mismatch", "desert/f3.verdict");
    let fit = civ_culture_terrain_fit("desert", &agg.by_faction[4].terrain_mix, &agg.world_mean_terrain).expect("desert/f4");
    assert_eq!(fit.key, "arid"); near(fit.value, 0.05405405405405406, "desert/f4.value"); near(fit.world_mean, 0.06504065040650407, "desert/f4.world_mean"); near(fit.ratio, 0.831081081081081, "desert/f4.ratio"); assert_eq!(fit.verdict, "mismatch", "desert/f4.verdict");
    let fit = civ_culture_terrain_fit("desert", &agg.by_faction[5].terrain_mix, &agg.world_mean_terrain).expect("desert/f5");
    assert_eq!(fit.key, "arid"); near(fit.value, 0.08108108108108109, "desert/f5.value"); near(fit.world_mean, 0.06504065040650407, "desert/f5.world_mean"); near(fit.ratio, 1.2466216216216215, "desert/f5.ratio"); assert_eq!(fit.verdict, "match", "desert/f5.verdict");
    let fit = civ_culture_terrain_fit("desert", &agg.by_faction[6].terrain_mix, &agg.world_mean_terrain).expect("desert/f6");
    assert_eq!(fit.key, "arid"); near(fit.value, 0.0, "desert/f6.value"); near(fit.world_mean, 0.06504065040650407, "desert/f6.world_mean"); near(fit.ratio, 0.0, "desert/f6.ratio"); assert_eq!(fit.verdict, "mismatch", "desert/f6.verdict");
    let fit = civ_culture_terrain_fit("riverlands", &agg.by_faction[0].terrain_mix, &agg.world_mean_terrain).expect("riverlands/f0");
    assert_eq!(fit.key, "river"); near(fit.value, 0.0, "riverlands/f0.value"); near(fit.world_mean, 1.0, "riverlands/f0.world_mean"); near(fit.ratio, 0.0, "riverlands/f0.ratio"); assert_eq!(fit.verdict, "mismatch", "riverlands/f0.verdict");
    let fit = civ_culture_terrain_fit("riverlands", &agg.by_faction[1].terrain_mix, &agg.world_mean_terrain).expect("riverlands/f1");
    assert_eq!(fit.key, "river"); near(fit.value, 1.0, "riverlands/f1.value"); near(fit.world_mean, 1.0, "riverlands/f1.world_mean"); near(fit.ratio, 1.0, "riverlands/f1.ratio"); assert_eq!(fit.verdict, "typical", "riverlands/f1.verdict");
    let fit = civ_culture_terrain_fit("riverlands", &agg.by_faction[2].terrain_mix, &agg.world_mean_terrain).expect("riverlands/f2");
    assert_eq!(fit.key, "river"); near(fit.value, 1.0, "riverlands/f2.value"); near(fit.world_mean, 1.0, "riverlands/f2.world_mean"); near(fit.ratio, 1.0, "riverlands/f2.ratio"); assert_eq!(fit.verdict, "typical", "riverlands/f2.verdict");
    let fit = civ_culture_terrain_fit("riverlands", &agg.by_faction[3].terrain_mix, &agg.world_mean_terrain).expect("riverlands/f3");
    assert_eq!(fit.key, "river"); near(fit.value, 1.0, "riverlands/f3.value"); near(fit.world_mean, 1.0, "riverlands/f3.world_mean"); near(fit.ratio, 1.0, "riverlands/f3.ratio"); assert_eq!(fit.verdict, "typical", "riverlands/f3.verdict");
    let fit = civ_culture_terrain_fit("riverlands", &agg.by_faction[4].terrain_mix, &agg.world_mean_terrain).expect("riverlands/f4");
    assert_eq!(fit.key, "river"); near(fit.value, 1.0, "riverlands/f4.value"); near(fit.world_mean, 1.0, "riverlands/f4.world_mean"); near(fit.ratio, 1.0, "riverlands/f4.ratio"); assert_eq!(fit.verdict, "typical", "riverlands/f4.verdict");
    let fit = civ_culture_terrain_fit("riverlands", &agg.by_faction[5].terrain_mix, &agg.world_mean_terrain).expect("riverlands/f5");
    assert_eq!(fit.key, "river"); near(fit.value, 1.0, "riverlands/f5.value"); near(fit.world_mean, 1.0, "riverlands/f5.world_mean"); near(fit.ratio, 1.0, "riverlands/f5.ratio"); assert_eq!(fit.verdict, "typical", "riverlands/f5.verdict");
    let fit = civ_culture_terrain_fit("riverlands", &agg.by_faction[6].terrain_mix, &agg.world_mean_terrain).expect("riverlands/f6");
    assert_eq!(fit.key, "river"); near(fit.value, 0.0, "riverlands/f6.value"); near(fit.world_mean, 1.0, "riverlands/f6.world_mean"); near(fit.ratio, 0.0, "riverlands/f6.ratio"); assert_eq!(fit.verdict, "mismatch", "riverlands/f6.verdict");
    let fit = civ_culture_terrain_fit("sylvan", &agg.by_faction[0].terrain_mix, &agg.world_mean_terrain).expect("sylvan/f0");
    assert_eq!(fit.key, "forest"); near(fit.value, 0.0, "sylvan/f0.value"); near(fit.world_mean, 0.6463414634146342, "sylvan/f0.world_mean"); near(fit.ratio, 0.0, "sylvan/f0.ratio"); assert_eq!(fit.verdict, "mismatch", "sylvan/f0.verdict");
    let fit = civ_culture_terrain_fit("sylvan", &agg.by_faction[1].terrain_mix, &agg.world_mean_terrain).expect("sylvan/f1");
    assert_eq!(fit.key, "forest"); near(fit.value, 0.6486486486486487, "sylvan/f1.value"); near(fit.world_mean, 0.6463414634146342, "sylvan/f1.world_mean"); near(fit.ratio, 1.0035696073431923, "sylvan/f1.ratio"); assert_eq!(fit.verdict, "typical", "sylvan/f1.verdict");
    let fit = civ_culture_terrain_fit("sylvan", &agg.by_faction[2].terrain_mix, &agg.world_mean_terrain).expect("sylvan/f2");
    assert_eq!(fit.key, "forest"); near(fit.value, 0.6666666666666666, "sylvan/f2.value"); near(fit.world_mean, 0.6463414634146342, "sylvan/f2.world_mean"); near(fit.ratio, 1.0314465408805031, "sylvan/f2.ratio"); assert_eq!(fit.verdict, "typical", "sylvan/f2.verdict");
    let fit = civ_culture_terrain_fit("sylvan", &agg.by_faction[3].terrain_mix, &agg.world_mean_terrain).expect("sylvan/f3");
    assert_eq!(fit.key, "forest"); near(fit.value, 0.625, "sylvan/f3.value"); near(fit.world_mean, 0.6463414634146342, "sylvan/f3.world_mean"); near(fit.ratio, 0.9669811320754716, "sylvan/f3.ratio"); assert_eq!(fit.verdict, "typical", "sylvan/f3.verdict");
    let fit = civ_culture_terrain_fit("sylvan", &agg.by_faction[4].terrain_mix, &agg.world_mean_terrain).expect("sylvan/f4");
    assert_eq!(fit.key, "forest"); near(fit.value, 0.5675675675675675, "sylvan/f4.value"); near(fit.world_mean, 0.6463414634146342, "sylvan/f4.world_mean"); near(fit.ratio, 0.8781234064252932, "sylvan/f4.ratio"); assert_eq!(fit.verdict, "typical", "sylvan/f4.verdict");
    let fit = civ_culture_terrain_fit("sylvan", &agg.by_faction[5].terrain_mix, &agg.world_mean_terrain).expect("sylvan/f5");
    assert_eq!(fit.key, "forest"); near(fit.value, 0.7027027027027027, "sylvan/f5.value"); near(fit.world_mean, 0.6463414634146342, "sylvan/f5.world_mean"); near(fit.ratio, 1.087200407955125, "sylvan/f5.ratio"); assert_eq!(fit.verdict, "typical", "sylvan/f5.verdict");
    let fit = civ_culture_terrain_fit("sylvan", &agg.by_faction[6].terrain_mix, &agg.world_mean_terrain).expect("sylvan/f6");
    assert_eq!(fit.key, "forest"); near(fit.value, 0.0, "sylvan/f6.value"); near(fit.world_mean, 0.6463414634146342, "sylvan/f6.world_mean"); near(fit.ratio, 0.0, "sylvan/f6.ratio"); assert_eq!(fit.verdict, "mismatch", "sylvan/f6.verdict");
    let fit = civ_culture_terrain_fit("maritime", &agg.by_faction[0].terrain_mix, &agg.world_mean_terrain).expect("maritime/f0");
    assert_eq!(fit.key, "coast"); near(fit.value, 0.0, "maritime/f0.value"); near(fit.world_mean, 0.24796747967479674, "maritime/f0.world_mean"); near(fit.ratio, 0.0, "maritime/f0.ratio"); assert_eq!(fit.verdict, "mismatch", "maritime/f0.verdict");
    let fit = civ_culture_terrain_fit("maritime", &agg.by_faction[1].terrain_mix, &agg.world_mean_terrain).expect("maritime/f1");
    assert_eq!(fit.key, "coast"); near(fit.value, 0.2702702702702703, "maritime/f1.value"); near(fit.world_mean, 0.24796747967479674, "maritime/f1.world_mean"); near(fit.ratio, 1.0899424014178114, "maritime/f1.ratio"); assert_eq!(fit.verdict, "typical", "maritime/f1.verdict");
    let fit = civ_culture_terrain_fit("maritime", &agg.by_faction[2].terrain_mix, &agg.world_mean_terrain).expect("maritime/f2");
    assert_eq!(fit.key, "coast"); near(fit.value, 0.21739130434782608, "maritime/f2.value"); near(fit.world_mean, 0.24796747967479674, "maritime/f2.world_mean"); near(fit.ratio, 0.8766928011404134, "maritime/f2.ratio"); assert_eq!(fit.verdict, "typical", "maritime/f2.verdict");
    let fit = civ_culture_terrain_fit("maritime", &agg.by_faction[3].terrain_mix, &agg.world_mean_terrain).expect("maritime/f3");
    assert_eq!(fit.key, "coast"); near(fit.value, 0.275, "maritime/f3.value"); near(fit.world_mean, 0.24796747967479674, "maritime/f3.world_mean"); near(fit.ratio, 1.1090163934426231, "maritime/f3.ratio"); assert_eq!(fit.verdict, "typical", "maritime/f3.verdict");
    let fit = civ_culture_terrain_fit("maritime", &agg.by_faction[4].terrain_mix, &agg.world_mean_terrain).expect("maritime/f4");
    assert_eq!(fit.key, "coast"); near(fit.value, 0.21621621621621623, "maritime/f4.value"); near(fit.world_mean, 0.24796747967479674, "maritime/f4.world_mean"); near(fit.ratio, 0.8719539211342491, "maritime/f4.ratio"); assert_eq!(fit.verdict, "typical", "maritime/f4.verdict");
    let fit = civ_culture_terrain_fit("maritime", &agg.by_faction[5].terrain_mix, &agg.world_mean_terrain).expect("maritime/f5");
    assert_eq!(fit.key, "coast"); near(fit.value, 0.24324324324324326, "maritime/f5.value"); near(fit.world_mean, 0.24796747967479674, "maritime/f5.world_mean"); near(fit.ratio, 0.9809481612760302, "maritime/f5.ratio"); assert_eq!(fit.verdict, "typical", "maritime/f5.verdict");
    let fit = civ_culture_terrain_fit("maritime", &agg.by_faction[6].terrain_mix, &agg.world_mean_terrain).expect("maritime/f6");
    assert_eq!(fit.key, "coast"); near(fit.value, 0.0, "maritime/f6.value"); near(fit.world_mean, 0.24796747967479674, "maritime/f6.world_mean"); near(fit.ratio, 0.0, "maritime/f6.ratio"); assert_eq!(fit.verdict, "mismatch", "maritime/f6.verdict");
    assert!(civ_culture_terrain_fit("common", &agg.by_faction[0].terrain_mix, &agg.world_mean_terrain).is_none(), "common/f0 must have no verdict");
    assert!(civ_culture_terrain_fit("common", &agg.by_faction[1].terrain_mix, &agg.world_mean_terrain).is_none(), "common/f1 must have no verdict");
    assert!(civ_culture_terrain_fit("common", &agg.by_faction[2].terrain_mix, &agg.world_mean_terrain).is_none(), "common/f2 must have no verdict");
    assert!(civ_culture_terrain_fit("common", &agg.by_faction[3].terrain_mix, &agg.world_mean_terrain).is_none(), "common/f3 must have no verdict");
    assert!(civ_culture_terrain_fit("common", &agg.by_faction[4].terrain_mix, &agg.world_mean_terrain).is_none(), "common/f4 must have no verdict");
    assert!(civ_culture_terrain_fit("common", &agg.by_faction[5].terrain_mix, &agg.world_mean_terrain).is_none(), "common/f5 must have no verdict");
    assert!(civ_culture_terrain_fit("common", &agg.by_faction[6].terrain_mix, &agg.world_mean_terrain).is_none(), "common/f6 must have no verdict");
    assert!(civ_culture_terrain_fit("imperial", &agg.by_faction[0].terrain_mix, &agg.world_mean_terrain).is_none(), "imperial/f0 must have no verdict");
    assert!(civ_culture_terrain_fit("imperial", &agg.by_faction[1].terrain_mix, &agg.world_mean_terrain).is_none(), "imperial/f1 must have no verdict");
    assert!(civ_culture_terrain_fit("imperial", &agg.by_faction[2].terrain_mix, &agg.world_mean_terrain).is_none(), "imperial/f2 must have no verdict");
    assert!(civ_culture_terrain_fit("imperial", &agg.by_faction[3].terrain_mix, &agg.world_mean_terrain).is_none(), "imperial/f3 must have no verdict");
    assert!(civ_culture_terrain_fit("imperial", &agg.by_faction[4].terrain_mix, &agg.world_mean_terrain).is_none(), "imperial/f4 must have no verdict");
    assert!(civ_culture_terrain_fit("imperial", &agg.by_faction[5].terrain_mix, &agg.world_mean_terrain).is_none(), "imperial/f5 must have no verdict");
    assert!(civ_culture_terrain_fit("imperial", &agg.by_faction[6].terrain_mix, &agg.world_mean_terrain).is_none(), "imperial/f6 must have no verdict");
}

#[test]
fn faction_aggregates_case_1_world_wrap() {
    // Case 1 (gw=48 gh=36 seed=314159 world=true): a wrapping world, big enough
    // that riverFlowThresh actually discriminates (0.920 of land is river here,
    // not case 0's degenerate 1.0) and that coast/arid/forest/hills all take
    // intermediate values. Faction 3 owns both edge columns AND holds
    // settlements at x=0 and x=gw-1 -- a faction whose settlements straddle the
    // seam. Factions 2 and 5 have territory but no settlement; faction 6 has
    // neither.
    let w = build_world(48, 36, 314159, true);
    let (gw, gh) = (48, 36);
    assert_eq!(fnv_f32(&w.ws.field), "ff792a79c88c72a3", "field hash: the harness world and this one are not the same world");
    assert_eq!(fnv_u8(&w.biome), "e8a30c1895843612", "biome raster hash");
    assert_eq!(fnv_u8(&river_mask(&w, gw, gh)), "e2d81a3cb1484f50", "river-mask hash");
    assert_eq!(fnv_f32(&w.ocean_dt), "70587b9c8a3777a", "ocean distance-transform hash");
    assert_eq!(fnv_u8(&w.lith), "79d25fa78b008708", "lithology hash");
    assert_eq!(fnv_f32(&w.water), "bcc02fe7ce704a2c", "water-access hash");
    near_rel(land_sum(&w.dens, &w.ws.field, w.ws.sea_level), 4033.9746332336217, 1e-6, "population-density land sum");
    near_rel(land_sum(&w.ws.flow_discharge, &w.ws.field, w.ws.sea_level), 6427.570258393884, 1e-6, "flow land sum");
    assert!((w.flow_thresh - 0.6912).abs() < 1e-12, "flow threshold");

    let territory = synthetic_territory(&w.ws.field, gw, gh, w.ws.sea_level, N_F, 3);
    assert_eq!(fnv_terr(&territory), "45822eb5310bbf2a", "territory hash: the two sides did not build the same raster");
    let land = w.ws.field.iter().filter(|&&h| (h as f64) >= w.ws.sea_level).count();
    assert_eq!(land, 1118, "land-cell count");
    assert!(land > 0 && territory.iter().any(|&t| t > 0), "fixture is not silently empty");
    let mut terr_count = [0usize; N_F];
    for &t in &territory { if t > 0 && (t as usize) < N_F { terr_count[t as usize] += 1; } }
    assert_eq!(terr_count, [0, 191, 193, 195, 191, 193, 0], "per-faction territory-cell counts");

    let places = [
        place(1, 15000.0, SettlementKind::Capital, 1200.0, 0.9, Some("timber"), true),
        place(3, 6000.0, SettlementKind::City, 400.0, 0.5, Some("grain"), true),
        place(3, 1500.0, SettlementKind::Town, 100.0, 0.4, Some("pastoral"), true),
        place(4, 120.0, SettlementKind::Hamlet, 0.0, 0.0, Some("fishing"), false),
    ];
    let input = FactionAggregatesInput {
        faction_count: N_F, gw, gh, sea: w.ws.sea_level, map_width_km: 800.0,
        field: &w.ws.field, territory: Some(&territory), density: Some(&w.dens),
        resources: Some(&w.res), biome: Some(&w.biome), flow: Some(&w.ws.flow_discharge),
        flow_thresh: w.flow_thresh, ocean_dist: Some(&w.ocean_dt), faction_has_religion: None,
    };
    let agg = cartalith_civ::civ_faction_aggregates(&input, &places);

    near(agg.max_pop, 15000.0, "max_pop");
    near(agg.max_trade_volume, 1200.0, "max_trade_volume");
    near(agg.max_territory_km2, 54167.0, "max_territory_km2");
    assert_eq!(agg.max_settlement_count, 2, "max_settlement_count");
    near(agg.world_mean_terrain["river"], 0.9203935599284436, "world_mean_terrain.river");
    near(agg.world_mean_terrain["coast"], 0.09660107334525939, "world_mean_terrain.coast");
    near(agg.world_mean_terrain["arid"], 0.03398926654740608, "world_mean_terrain.arid");
    near(agg.world_mean_terrain["forest"], 0.313953488372093, "world_mean_terrain.forest");
    near(agg.world_mean_terrain["hills"], 0.2513416815742397, "world_mean_terrain.hills");
    near_rel(agg.world_mean_resource["copper"], 0.15320699228567766, 1e-6, "world_mean_resource.copper");
    near_rel(agg.world_mean_resource["tin"], 0.13953488020223026, 1e-6, "world_mean_resource.tin");
    near_rel(agg.world_mean_resource["iron"], 0.062343470512327866, 1e-6, "world_mean_resource.iron");
    near_rel(agg.world_mean_resource["gold"], 0.08185113187419493, 1e-6, "world_mean_resource.gold");
    near_rel(agg.world_mean_resource["salt"], 0.004677522161139145, 1e-6, "world_mean_resource.salt");
    near_rel(agg.world_mean_resource["timber"], 0.34079689022040327, 1e-6, "world_mean_resource.timber");
    near_rel(agg.world_mean_resource["lead"], 0.17787080749949316, 1e-6, "world_mean_resource.lead");
    near_rel(agg.world_mean_resource["silver"], 0.04312261016722869, 1e-6, "world_mean_resource.silver");
    near_rel(agg.world_mean_resource["clay"], 0.4655005922257794, 1e-6, "world_mean_resource.clay");
    near_rel(agg.world_mean_resource["buildstone"], 0.33922182751256363, 1e-6, "world_mean_resource.buildstone");
    near_rel(agg.world_mean_resource["flint"], 0.1347048354063563, 1e-6, "world_mean_resource.flint");
    near_rel(agg.world_mean_resource["obsidian"], 0.022661428438742814, 1e-6, "world_mean_resource.obsidian");
    near_rel(agg.world_mean_resource["gems"], 0.10424865838240212, 1e-6, "world_mean_resource.gems");
    near_rel(agg.world_mean_resource["sulfur"], 0.03350535466973811, 1e-6, "world_mean_resource.sulfur");
    near_rel(agg.world_mean_resource["alum"], 0.04463388097734059, 1e-6, "world_mean_resource.alum");

    // ---- faction 0 ----
    let b = &agg.by_faction[0];
    near(b.pop, 0.0, "f0.pop");
    near(b.territory_km2, 0.0, "f0.territory_km2");
    near(b.trade_volume, 0.0, "f0.trade_volume");
    near(b.tax_income, 0.0, "f0.tax_income");
    near(b.mean_importance, 0.0, "f0.mean_importance");
    near(b.fortified_fraction, 0.0, "f0.fortified_fraction");
    near(b.craft_share, 0.0, "f0.craft_share");
    near_abs(b.food_production_capacity, 0.0, 1.0, "f0.food_production_capacity");
    near_abs(b.food_surplus, 0.0, 1.0, "f0.food_surplus");
    assert_eq!(b.settlement_count, 0, "f0.settlement_count");
    assert_eq!(b.capital, None, "f0.capital");
    near(b.power.military, 0.0, "f0.power.military");
    near(b.power.economic, 0.0, "f0.power.economic");
    near(b.power.political, 0.0, "f0.power.political");
    near(b.power.cultural, 0.0, "f0.power.cultural");
    near(b.power.religious, 0.0, "f0.power.religious");
    near(b.power.overall, 0.0, "f0.power.overall");
    near(b.sector_output.fishing, 0.0, "f0.sector.fishing");
    near(b.sector_output.agriculture, 0.0, "f0.sector.agriculture");
    near(b.sector_output.livestock, 0.0, "f0.sector.livestock");
    near(b.sector_output.forestry, 0.0, "f0.sector.forestry");
    near(b.sector_output.mining, 0.0, "f0.sector.mining");
    near(b.sector_output.craft, 0.0, "f0.sector.craft");
    near(b.terrain_mix["river"], 0.0, "f0.terrain_mix.river");
    near(b.terrain_mix["coast"], 0.0, "f0.terrain_mix.coast");
    near(b.terrain_mix["arid"], 0.0, "f0.terrain_mix.arid");
    near(b.terrain_mix["forest"], 0.0, "f0.terrain_mix.forest");
    near(b.terrain_mix["hills"], 0.0, "f0.terrain_mix.hills");
    near_rel(b.resource_potential["copper"], 0.0, 1e-6, "f0.resource.copper");
    near_rel(b.resource_potential["tin"], 0.0, 1e-6, "f0.resource.tin");
    near_rel(b.resource_potential["iron"], 0.0, 1e-6, "f0.resource.iron");
    near_rel(b.resource_potential["gold"], 0.0, 1e-6, "f0.resource.gold");
    near_rel(b.resource_potential["salt"], 0.0, 1e-6, "f0.resource.salt");
    near_rel(b.resource_potential["timber"], 0.0, 1e-6, "f0.resource.timber");
    near_rel(b.resource_potential["lead"], 0.0, 1e-6, "f0.resource.lead");
    near_rel(b.resource_potential["silver"], 0.0, 1e-6, "f0.resource.silver");
    near_rel(b.resource_potential["clay"], 0.0, 1e-6, "f0.resource.clay");
    near_rel(b.resource_potential["buildstone"], 0.0, 1e-6, "f0.resource.buildstone");
    near_rel(b.resource_potential["flint"], 0.0, 1e-6, "f0.resource.flint");
    near_rel(b.resource_potential["obsidian"], 0.0, 1e-6, "f0.resource.obsidian");
    near_rel(b.resource_potential["gems"], 0.0, 1e-6, "f0.resource.gems");
    near_rel(b.resource_potential["sulfur"], 0.0, 1e-6, "f0.resource.sulfur");
    near_rel(b.resource_potential["alum"], 0.0, 1e-6, "f0.resource.alum");
    assert_eq!(b.exports, Vec::<&str>::new(), "f0.exports");
    assert_eq!(b.imports, vec!["copper", "iron", "salt", "timber", "lead", "clay", "buildstone", "alum"], "f0.imports");
    assert_eq!(b.strategic_resources, Vec::<&str>::new(), "f0.strategic");

    // ---- faction 1 ----
    let b = &agg.by_faction[1];
    near(b.pop, 15000.0, "f1.pop");
    near(b.territory_km2, 53056.0, "f1.territory_km2");
    near(b.trade_volume, 1200.0, "f1.trade_volume");
    near(b.tax_income, 1350.0, "f1.tax_income");
    near(b.mean_importance, 0.9, "f1.mean_importance");
    near(b.fortified_fraction, 1.0, "f1.fortified_fraction");
    near(b.craft_share, 0.0, "f1.craft_share");
    near_abs(b.food_production_capacity, 182995.0, 1.0, "f1.food_production_capacity");
    near_abs(b.food_surplus, 167995.0, 1.0, "f1.food_surplus");
    assert_eq!(b.settlement_count, 1, "f1.settlement_count");
    assert_eq!(b.capital, Some(0), "f1.capital");
    near(b.power.military, 96.00000000000001, "f1.power.military");
    near(b.power.economic, 97.0, "f1.power.economic");
    near(b.power.political, 81.78212749460003, "f1.power.political");
    near(b.power.cultural, 85.0, "f1.power.cultural");
    near(b.power.religious, 0.0, "f1.power.religious");
    near(b.power.overall, 71.95642549892001, "f1.power.overall");
    near(b.sector_output.fishing, 0.0, "f1.sector.fishing");
    near(b.sector_output.agriculture, 0.0, "f1.sector.agriculture");
    near(b.sector_output.livestock, 0.0, "f1.sector.livestock");
    near(b.sector_output.forestry, 14100.0, "f1.sector.forestry");
    near(b.sector_output.mining, 0.0, "f1.sector.mining");
    near(b.sector_output.craft, 0.0, "f1.sector.craft");
    near(b.terrain_mix["river"], 0.9319371727748691, "f1.terrain_mix.river");
    near(b.terrain_mix["coast"], 0.08900523560209424, "f1.terrain_mix.coast");
    near(b.terrain_mix["arid"], 0.03664921465968586, "f1.terrain_mix.arid");
    near(b.terrain_mix["forest"], 0.3089005235602094, "f1.terrain_mix.forest");
    near(b.terrain_mix["hills"], 0.2617801047120419, "f1.terrain_mix.hills");
    near_rel(b.resource_potential["copper"], 0.14844285579963895, 1e-6, "f1.resource.copper");
    near_rel(b.resource_potential["tin"], 0.14214659330108403, 1e-6, "f1.resource.tin");
    near_rel(b.resource_potential["iron"], 0.07068062827225131, 1e-6, "f1.resource.iron");
    near_rel(b.resource_potential["gold"], 0.09032899812253982, 1e-6, "f1.resource.gold");
    near_rel(b.resource_potential["salt"], 0.008368251523422321, 1e-6, "f1.resource.salt");
    near_rel(b.resource_potential["timber"], 0.3287587521587991, 1e-6, "f1.resource.timber");
    near_rel(b.resource_potential["lead"], 0.19543643038310304, 1e-6, "f1.resource.lead");
    near_rel(b.resource_potential["silver"], 0.05430001441720893, 1e-6, "f1.resource.silver");
    near_rel(b.resource_potential["clay"], 0.46338905339465714, 1e-6, "f1.resource.clay");
    near_rel(b.resource_potential["buildstone"], 0.37120419178957714, 1e-6, "f1.resource.buildstone");
    near_rel(b.resource_potential["flint"], 0.15078534630580723, 1e-6, "f1.resource.flint");
    near_rel(b.resource_potential["obsidian"], 0.011316346248407015, 1e-6, "f1.resource.obsidian");
    near_rel(b.resource_potential["gems"], 0.09842931943414099, 1e-6, "f1.resource.gems");
    near_rel(b.resource_potential["sulfur"], 0.023244274224286304, 1e-6, "f1.resource.sulfur");
    near_rel(b.resource_potential["alum"], 0.03273154490905282, 1e-6, "f1.resource.alum");
    assert_eq!(b.exports, vec!["food"], "f1.exports");
    assert_eq!(b.imports, Vec::<&str>::new(), "f1.imports");
    assert_eq!(b.strategic_resources, vec!["clay"], "f1.strategic");

    // ---- faction 2 ----
    let b = &agg.by_faction[2];
    near(b.pop, 0.0, "f2.pop");
    near(b.territory_km2, 53611.0, "f2.territory_km2");
    near(b.trade_volume, 0.0, "f2.trade_volume");
    near(b.tax_income, 0.0, "f2.tax_income");
    near(b.mean_importance, 0.0, "f2.mean_importance");
    near(b.fortified_fraction, 0.0, "f2.fortified_fraction");
    near(b.craft_share, 0.0, "f2.craft_share");
    near_abs(b.food_production_capacity, 199216.0, 1.0, "f2.food_production_capacity");
    near_abs(b.food_surplus, 199216.0, 1.0, "f2.food_surplus");
    assert_eq!(b.settlement_count, 0, "f2.settlement_count");
    assert_eq!(b.capital, None, "f2.capital");
    near(b.power.military, 0.0, "f2.power.military");
    near(b.power.economic, 0.0, "f2.power.economic");
    near(b.power.political, 34.6407406723651, "f2.power.political");
    near(b.power.cultural, 0.0, "f2.power.cultural");
    near(b.power.religious, 0.0, "f2.power.religious");
    near(b.power.overall, 6.92814813447302, "f2.power.overall");
    near(b.sector_output.fishing, 0.0, "f2.sector.fishing");
    near(b.sector_output.agriculture, 0.0, "f2.sector.agriculture");
    near(b.sector_output.livestock, 0.0, "f2.sector.livestock");
    near(b.sector_output.forestry, 0.0, "f2.sector.forestry");
    near(b.sector_output.mining, 0.0, "f2.sector.mining");
    near(b.sector_output.craft, 0.0, "f2.sector.craft");
    near(b.terrain_mix["river"], 0.927461139896373, "f2.terrain_mix.river");
    near(b.terrain_mix["coast"], 0.10880829015544041, "f2.terrain_mix.coast");
    near(b.terrain_mix["arid"], 0.05181347150259067, "f2.terrain_mix.arid");
    near(b.terrain_mix["forest"], 0.3005181347150259, "f2.terrain_mix.forest");
    near(b.terrain_mix["hills"], 0.25906735751295334, "f2.terrain_mix.hills");
    near_rel(b.resource_potential["copper"], 0.15565578571467664, 1e-6, "f2.resource.copper");
    near_rel(b.resource_potential["tin"], 0.1277202040420295, 1e-6, "f2.resource.tin");
    near_rel(b.resource_potential["iron"], 0.06243523322238823, 1e-6, "f2.resource.iron");
    near_rel(b.resource_potential["gold"], 0.07905572482958977, 1e-6, "f2.resource.gold");
    near_rel(b.resource_potential["salt"], 0.0, 1e-6, "f2.resource.salt");
    near_rel(b.resource_potential["timber"], 0.347395273687926, 1e-6, "f2.resource.timber");
    near_rel(b.resource_potential["lead"], 0.17704223521015186, 1e-6, "f2.resource.lead");
    near_rel(b.resource_potential["silver"], 0.045835141536485345, 1e-6, "f2.resource.silver");
    near_rel(b.resource_potential["clay"], 0.4452734864437518, 1e-6, "f2.resource.clay");
    near_rel(b.resource_potential["buildstone"], 0.3461139924785634, 1e-6, "f2.resource.buildstone");
    near_rel(b.resource_potential["flint"], 0.13678757020228885, 1e-6, "f2.resource.flint");
    near_rel(b.resource_potential["obsidian"], 0.018467933093945597, 1e-6, "f2.resource.obsidian");
    near_rel(b.resource_potential["gems"], 0.08937823834196891, 1e-6, "f2.resource.gems");
    near_rel(b.resource_potential["sulfur"], 0.026785161519915328, 1e-6, "f2.resource.sulfur");
    near_rel(b.resource_potential["alum"], 0.03325132909834076, 1e-6, "f2.resource.alum");
    assert_eq!(b.exports, vec!["food"], "f2.exports");
    assert_eq!(b.imports, vec!["salt"], "f2.imports");
    assert_eq!(b.strategic_resources, vec!["clay"], "f2.strategic");

    // ---- faction 3 ----
    let b = &agg.by_faction[3];
    near(b.pop, 7500.0, "f3.pop");
    near(b.territory_km2, 54167.0, "f3.territory_km2");
    near(b.trade_volume, 500.0, "f3.trade_volume");
    near(b.tax_income, 495.0, "f3.tax_income");
    near(b.mean_importance, 0.45, "f3.mean_importance");
    near(b.fortified_fraction, 1.0, "f3.fortified_fraction");
    near(b.craft_share, 0.0, "f3.craft_share");
    near_abs(b.food_production_capacity, 191297.0, 1.0, "f3.food_production_capacity");
    near_abs(b.food_surplus, 183797.0, 1.0, "f3.food_surplus");
    assert_eq!(b.settlement_count, 2, "f3.settlement_count");
    assert_eq!(b.capital, Some(1), "f3.capital");
    near(b.power.military, 69.5, "f3.power.military");
    near(b.power.economic, 45.166666666666664, "f3.power.economic");
    near(b.power.political, 79.75, "f3.power.political");
    near(b.power.cultural, 64.99999999999999, "f3.power.cultural");
    near(b.power.religious, 0.0, "f3.power.religious");
    near(b.power.overall, 51.883333333333326, "f3.power.overall");
    near(b.sector_output.fishing, 0.0, "f3.sector.fishing");
    near(b.sector_output.agriculture, 4200.0, "f3.sector.agriculture");
    near(b.sector_output.livestock, 960.0, "f3.sector.livestock");
    near(b.sector_output.forestry, 0.0, "f3.sector.forestry");
    near(b.sector_output.mining, 0.0, "f3.sector.mining");
    near(b.sector_output.craft, 0.0, "f3.sector.craft");
    near(b.terrain_mix["river"], 0.9179487179487179, "f3.terrain_mix.river");
    near(b.terrain_mix["coast"], 0.11282051282051282, "f3.terrain_mix.coast");
    near(b.terrain_mix["arid"], 0.041025641025641026, "f3.terrain_mix.arid");
    near(b.terrain_mix["forest"], 0.3282051282051282, "f3.terrain_mix.forest");
    near(b.terrain_mix["hills"], 0.2564102564102564, "f3.terrain_mix.hills");
    near_rel(b.resource_potential["copper"], 0.1546378334542402, 1e-6, "f3.resource.copper");
    near_rel(b.resource_potential["tin"], 0.14384615023930866, 1e-6, "f3.resource.tin");
    near_rel(b.resource_potential["iron"], 0.05589743569875375, 1e-6, "f3.resource.iron");
    near_rel(b.resource_potential["gold"], 0.05985449598385738, 1e-6, "f3.resource.gold");
    near_rel(b.resource_potential["salt"], 0.0, 1e-6, "f3.resource.salt");
    near_rel(b.resource_potential["timber"], 0.3502148331739964, 1e-6, "f3.resource.timber");
    near_rel(b.resource_potential["lead"], 0.15355399892880367, 1e-6, "f3.resource.lead");
    near_rel(b.resource_potential["silver"], 0.03434858994606214, 1e-6, "f3.resource.silver");
    near_rel(b.resource_potential["clay"], 0.4818221431512099, 1e-6, "f3.resource.clay");
    near_rel(b.resource_potential["buildstone"], 0.30333333504505644, 1e-6, "f3.resource.buildstone");
    near_rel(b.resource_potential["flint"], 0.10769231197161552, 1e-6, "f3.resource.flint");
    near_rel(b.resource_potential["obsidian"], 0.03461622397104899, 1e-6, "f3.resource.obsidian");
    near_rel(b.resource_potential["gems"], 0.10410256416369708, 1e-6, "f3.resource.gems");
    near_rel(b.resource_potential["sulfur"], 0.045521856576968465, 1e-6, "f3.resource.sulfur");
    near_rel(b.resource_potential["alum"], 0.05747804091526912, 1e-6, "f3.resource.alum");
    assert_eq!(b.exports, vec!["obsidian", "sulfur", "food"], "f3.exports");
    assert_eq!(b.imports, vec!["salt"], "f3.imports");
    assert_eq!(b.strategic_resources, vec!["clay"], "f3.strategic");

    // ---- faction 4 ----
    let b = &agg.by_faction[4];
    near(b.pop, 120.0, "f4.pop");
    near(b.territory_km2, 53056.0, "f4.territory_km2");
    near(b.trade_volume, 0.0, "f4.trade_volume");
    near(b.tax_income, 2.0, "f4.tax_income");
    near(b.mean_importance, 0.0, "f4.mean_importance");
    near(b.fortified_fraction, 0.0, "f4.fortified_fraction");
    near(b.craft_share, 0.0, "f4.craft_share");
    near_abs(b.food_production_capacity, 187541.0, 1.0, "f4.food_production_capacity");
    near_abs(b.food_surplus, 187421.0, 1.0, "f4.food_surplus");
    assert_eq!(b.settlement_count, 1, "f4.settlement_count");
    assert_eq!(b.capital, Some(3), "f4.capital");
    near(b.power.military, 0.36000000000000004, "f4.power.military");
    near(b.power.economic, 0.24, "f4.power.economic");
    near(b.power.political, 44.28212749460003, "f4.power.political");
    near(b.power.cultural, 15.559999999999999, "f4.power.cultural");
    near(b.power.religious, 0.0, "f4.power.religious");
    near(b.power.overall, 12.088425498920007, "f4.power.overall");
    near(b.sector_output.fishing, 48.0, "f4.sector.fishing");
    near(b.sector_output.agriculture, 0.0, "f4.sector.agriculture");
    near(b.sector_output.livestock, 0.0, "f4.sector.livestock");
    near(b.sector_output.forestry, 0.0, "f4.sector.forestry");
    near(b.sector_output.mining, 0.0, "f4.sector.mining");
    near(b.sector_output.craft, 0.0, "f4.sector.craft");
    near(b.terrain_mix["river"], 0.9267015706806283, "f4.terrain_mix.river");
    near(b.terrain_mix["coast"], 0.08900523560209424, "f4.terrain_mix.coast");
    near(b.terrain_mix["arid"], 0.015706806282722512, "f4.terrain_mix.arid");
    near(b.terrain_mix["forest"], 0.3403141361256545, "f4.terrain_mix.forest");
    near(b.terrain_mix["hills"], 0.25654450261780104, "f4.terrain_mix.hills");
    near_rel(b.resource_potential["copper"], 0.1518470255148485, 1e-6, "f4.resource.copper");
    near_rel(b.resource_potential["tin"], 0.1397905724210889, 1e-6, "f4.resource.tin");
    near_rel(b.resource_potential["iron"], 0.06753926696889688, 1e-6, "f4.resource.iron");
    near_rel(b.resource_potential["gold"], 0.05709764369183186, 1e-6, "f4.resource.gold");
    near_rel(b.resource_potential["salt"], 0.005406251752563796, 1e-6, "f4.resource.salt");
    near_rel(b.resource_potential["timber"], 0.3443452032448734, 1e-6, "f4.resource.timber");
    near_rel(b.resource_potential["lead"], 0.1852640557975669, 1e-6, "f4.resource.lead");
    near_rel(b.resource_potential["silver"], 0.03991386153935138, 1e-6, "f4.resource.silver");
    near_rel(b.resource_potential["clay"], 0.48588777899117996, 1e-6, "f4.resource.clay");
    near_rel(b.resource_potential["buildstone"], 0.3431937204605622, 1e-6, "f4.resource.buildstone");
    near_rel(b.resource_potential["flint"], 0.14136126216169428, 1e-6, "f4.resource.flint");
    near_rel(b.resource_potential["obsidian"], 0.037539749245369, 1e-6, "f4.resource.obsidian");
    near_rel(b.resource_potential["gems"], 0.11047120431330816, 1e-6, "f4.resource.gems");
    near_rel(b.resource_potential["sulfur"], 0.0544227970208173, 1e-6, "f4.resource.sulfur");
    near_rel(b.resource_potential["alum"], 0.060094528173276894, 1e-6, "f4.resource.alum");
    assert_eq!(b.exports, vec!["obsidian", "sulfur", "food"], "f4.exports");
    assert_eq!(b.imports, Vec::<&str>::new(), "f4.imports");
    assert_eq!(b.strategic_resources, vec!["clay"], "f4.strategic");

    // ---- faction 5 ----
    let b = &agg.by_faction[5];
    near(b.pop, 0.0, "f5.pop");
    near(b.territory_km2, 53611.0, "f5.territory_km2");
    near(b.trade_volume, 0.0, "f5.trade_volume");
    near(b.tax_income, 0.0, "f5.tax_income");
    near(b.mean_importance, 0.0, "f5.mean_importance");
    near(b.fortified_fraction, 0.0, "f5.fortified_fraction");
    near(b.craft_share, 0.0, "f5.craft_share");
    near_abs(b.food_production_capacity, 193365.0, 1.0, "f5.food_production_capacity");
    near_abs(b.food_surplus, 193365.0, 1.0, "f5.food_surplus");
    assert_eq!(b.settlement_count, 0, "f5.settlement_count");
    assert_eq!(b.capital, None, "f5.capital");
    near(b.power.military, 0.0, "f5.power.military");
    near(b.power.economic, 0.0, "f5.power.economic");
    near(b.power.political, 34.6407406723651, "f5.power.political");
    near(b.power.cultural, 0.0, "f5.power.cultural");
    near(b.power.religious, 0.0, "f5.power.religious");
    near(b.power.overall, 6.92814813447302, "f5.power.overall");
    near(b.sector_output.fishing, 0.0, "f5.sector.fishing");
    near(b.sector_output.agriculture, 0.0, "f5.sector.agriculture");
    near(b.sector_output.livestock, 0.0, "f5.sector.livestock");
    near(b.sector_output.forestry, 0.0, "f5.sector.forestry");
    near(b.sector_output.mining, 0.0, "f5.sector.mining");
    near(b.sector_output.craft, 0.0, "f5.sector.craft");
    near(b.terrain_mix["river"], 0.9119170984455959, "f5.terrain_mix.river");
    near(b.terrain_mix["coast"], 0.09844559585492228, "f5.terrain_mix.coast");
    near(b.terrain_mix["arid"], 0.02072538860103627, "f5.terrain_mix.arid");
    near(b.terrain_mix["forest"], 0.30569948186528495, "f5.terrain_mix.forest");
    near(b.terrain_mix["hills"], 0.23834196891191708, "f5.terrain_mix.hills");
    near_rel(b.resource_potential["copper"], 0.147339132938949, 1e-6, "f5.resource.copper");
    near_rel(b.resource_potential["tin"], 0.15829015148736034, 1e-6, "f5.resource.tin");
    near_rel(b.resource_potential["iron"], 0.07331606205263286, 1e-6, "f5.resource.iron");
    near_rel(b.resource_potential["gold"], 0.08148095833977269, 1e-6, "f5.resource.gold");
    near_rel(b.resource_potential["salt"], 0.005908148276373512, 1e-6, "f5.resource.salt");
    near_rel(b.resource_potential["timber"], 0.3505791376291779, 1e-6, "f5.resource.timber");
    near_rel(b.resource_potential["lead"], 0.1992374763587596, 1e-6, "f5.resource.lead");
    near_rel(b.resource_potential["silver"], 0.05042420134643199, 1e-6, "f5.resource.silver");
    near_rel(b.resource_potential["clay"], 0.4423946373820922, 1e-6, "f5.resource.clay");
    near_rel(b.resource_potential["buildstone"], 0.36735751622699087, 1e-6, "f5.resource.buildstone");
    near_rel(b.resource_potential["flint"], 0.14922280385704237, 1e-6, "f5.resource.flint");
    near_rel(b.resource_potential["obsidian"], 0.0131531759247261, 1e-6, "f5.resource.obsidian");
    near_rel(b.resource_potential["gems"], 0.11321243535669356, 1e-6, "f5.resource.gems");
    near_rel(b.resource_potential["sulfur"], 0.019110983208671134, 1e-6, "f5.resource.sulfur");
    near_rel(b.resource_potential["alum"], 0.029344234750678502, 1e-6, "f5.resource.alum");
    assert_eq!(b.exports, vec!["food"], "f5.exports");
    assert_eq!(b.imports, Vec::<&str>::new(), "f5.imports");
    assert_eq!(b.strategic_resources, vec!["clay"], "f5.strategic");

    // ---- faction 6 ----
    let b = &agg.by_faction[6];
    near(b.pop, 0.0, "f6.pop");
    near(b.territory_km2, 0.0, "f6.territory_km2");
    near(b.trade_volume, 0.0, "f6.trade_volume");
    near(b.tax_income, 0.0, "f6.tax_income");
    near(b.mean_importance, 0.0, "f6.mean_importance");
    near(b.fortified_fraction, 0.0, "f6.fortified_fraction");
    near(b.craft_share, 0.0, "f6.craft_share");
    near_abs(b.food_production_capacity, 0.0, 1.0, "f6.food_production_capacity");
    near_abs(b.food_surplus, 0.0, 1.0, "f6.food_surplus");
    assert_eq!(b.settlement_count, 0, "f6.settlement_count");
    assert_eq!(b.capital, None, "f6.capital");
    near(b.power.military, 0.0, "f6.power.military");
    near(b.power.economic, 0.0, "f6.power.economic");
    near(b.power.political, 0.0, "f6.power.political");
    near(b.power.cultural, 0.0, "f6.power.cultural");
    near(b.power.religious, 0.0, "f6.power.religious");
    near(b.power.overall, 0.0, "f6.power.overall");
    near(b.sector_output.fishing, 0.0, "f6.sector.fishing");
    near(b.sector_output.agriculture, 0.0, "f6.sector.agriculture");
    near(b.sector_output.livestock, 0.0, "f6.sector.livestock");
    near(b.sector_output.forestry, 0.0, "f6.sector.forestry");
    near(b.sector_output.mining, 0.0, "f6.sector.mining");
    near(b.sector_output.craft, 0.0, "f6.sector.craft");
    near(b.terrain_mix["river"], 0.0, "f6.terrain_mix.river");
    near(b.terrain_mix["coast"], 0.0, "f6.terrain_mix.coast");
    near(b.terrain_mix["arid"], 0.0, "f6.terrain_mix.arid");
    near(b.terrain_mix["forest"], 0.0, "f6.terrain_mix.forest");
    near(b.terrain_mix["hills"], 0.0, "f6.terrain_mix.hills");
    near_rel(b.resource_potential["copper"], 0.0, 1e-6, "f6.resource.copper");
    near_rel(b.resource_potential["tin"], 0.0, 1e-6, "f6.resource.tin");
    near_rel(b.resource_potential["iron"], 0.0, 1e-6, "f6.resource.iron");
    near_rel(b.resource_potential["gold"], 0.0, 1e-6, "f6.resource.gold");
    near_rel(b.resource_potential["salt"], 0.0, 1e-6, "f6.resource.salt");
    near_rel(b.resource_potential["timber"], 0.0, 1e-6, "f6.resource.timber");
    near_rel(b.resource_potential["lead"], 0.0, 1e-6, "f6.resource.lead");
    near_rel(b.resource_potential["silver"], 0.0, 1e-6, "f6.resource.silver");
    near_rel(b.resource_potential["clay"], 0.0, 1e-6, "f6.resource.clay");
    near_rel(b.resource_potential["buildstone"], 0.0, 1e-6, "f6.resource.buildstone");
    near_rel(b.resource_potential["flint"], 0.0, 1e-6, "f6.resource.flint");
    near_rel(b.resource_potential["obsidian"], 0.0, 1e-6, "f6.resource.obsidian");
    near_rel(b.resource_potential["gems"], 0.0, 1e-6, "f6.resource.gems");
    near_rel(b.resource_potential["sulfur"], 0.0, 1e-6, "f6.resource.sulfur");
    near_rel(b.resource_potential["alum"], 0.0, 1e-6, "f6.resource.alum");
    assert_eq!(b.exports, Vec::<&str>::new(), "f6.exports");
    assert_eq!(b.imports, vec!["copper", "iron", "salt", "timber", "lead", "clay", "buildstone", "alum"], "f6.imports");
    assert_eq!(b.strategic_resources, Vec::<&str>::new(), "f6.strategic");

    // ---- `civ_culture_terrain_fit` over the real Territory Fit output ----
    let fit = civ_culture_terrain_fit("highland", &agg.by_faction[0].terrain_mix, &agg.world_mean_terrain).expect("highland/f0");
    assert_eq!(fit.key, "hills"); near(fit.value, 0.0, "highland/f0.value"); near(fit.world_mean, 0.2513416815742397, "highland/f0.world_mean"); near(fit.ratio, 0.0, "highland/f0.ratio"); assert_eq!(fit.verdict, "mismatch", "highland/f0.verdict");
    let fit = civ_culture_terrain_fit("highland", &agg.by_faction[1].terrain_mix, &agg.world_mean_terrain).expect("highland/f1");
    assert_eq!(fit.key, "hills"); near(fit.value, 0.2617801047120419, "highland/f1.value"); near(fit.world_mean, 0.2513416815742397, "highland/f1.world_mean"); near(fit.ratio, 1.041530808071398, "highland/f1.ratio"); assert_eq!(fit.verdict, "typical", "highland/f1.verdict");
    let fit = civ_culture_terrain_fit("highland", &agg.by_faction[2].terrain_mix, &agg.world_mean_terrain).expect("highland/f2");
    assert_eq!(fit.key, "hills"); near(fit.value, 0.25906735751295334, "highland/f2.value"); near(fit.world_mean, 0.2513416815742397, "highland/f2.world_mean"); near(fit.ratio, 1.0307377427027824, "highland/f2.ratio"); assert_eq!(fit.verdict, "typical", "highland/f2.verdict");
    let fit = civ_culture_terrain_fit("highland", &agg.by_faction[3].terrain_mix, &agg.world_mean_terrain).expect("highland/f3");
    assert_eq!(fit.key, "hills"); near(fit.value, 0.2564102564102564, "highland/f3.value"); near(fit.world_mean, 0.2513416815742397, "highland/f3.world_mean"); near(fit.ratio, 1.0201660735468565, "highland/f3.ratio"); assert_eq!(fit.verdict, "typical", "highland/f3.verdict");
    let fit = civ_culture_terrain_fit("highland", &agg.by_faction[4].terrain_mix, &agg.world_mean_terrain).expect("highland/f4");
    assert_eq!(fit.key, "hills"); near(fit.value, 0.25654450261780104, "highland/f4.value"); near(fit.world_mean, 0.2513416815742397, "highland/f4.world_mean"); near(fit.ratio, 1.02070019190997, "highland/f4.ratio"); assert_eq!(fit.verdict, "typical", "highland/f4.verdict");
    let fit = civ_culture_terrain_fit("highland", &agg.by_faction[5].terrain_mix, &agg.world_mean_terrain).expect("highland/f5");
    assert_eq!(fit.key, "hills"); near(fit.value, 0.23834196891191708, "highland/f5.value"); near(fit.world_mean, 0.2513416815742397, "highland/f5.world_mean"); near(fit.ratio, 0.9482787232865598, "highland/f5.ratio"); assert_eq!(fit.verdict, "typical", "highland/f5.verdict");
    let fit = civ_culture_terrain_fit("highland", &agg.by_faction[6].terrain_mix, &agg.world_mean_terrain).expect("highland/f6");
    assert_eq!(fit.key, "hills"); near(fit.value, 0.0, "highland/f6.value"); near(fit.world_mean, 0.2513416815742397, "highland/f6.world_mean"); near(fit.ratio, 0.0, "highland/f6.ratio"); assert_eq!(fit.verdict, "mismatch", "highland/f6.verdict");
    let fit = civ_culture_terrain_fit("desert", &agg.by_faction[0].terrain_mix, &agg.world_mean_terrain).expect("desert/f0");
    assert_eq!(fit.key, "arid"); near(fit.value, 0.0, "desert/f0.value"); near(fit.world_mean, 0.03398926654740608, "desert/f0.world_mean"); near(fit.ratio, 0.0, "desert/f0.ratio"); assert_eq!(fit.verdict, "mismatch", "desert/f0.verdict");
    let fit = civ_culture_terrain_fit("desert", &agg.by_faction[1].terrain_mix, &agg.world_mean_terrain).expect("desert/f1");
    assert_eq!(fit.key, "arid"); near(fit.value, 0.03664921465968586, "desert/f1.value"); near(fit.world_mean, 0.03398926654740608, "desert/f1.world_mean"); near(fit.ratio, 1.0782584734086524, "desert/f1.ratio"); assert_eq!(fit.verdict, "typical", "desert/f1.verdict");
    let fit = civ_culture_terrain_fit("desert", &agg.by_faction[2].terrain_mix, &agg.world_mean_terrain).expect("desert/f2");
    assert_eq!(fit.key, "arid"); near(fit.value, 0.05181347150259067, "desert/f2.value"); near(fit.world_mean, 0.03398926654740608, "desert/f2.world_mean"); near(fit.ratio, 1.5244068721025361, "desert/f2.ratio"); assert_eq!(fit.verdict, "match", "desert/f2.verdict");
    let fit = civ_culture_terrain_fit("desert", &agg.by_faction[3].terrain_mix, &agg.world_mean_terrain).expect("desert/f3");
    assert_eq!(fit.key, "arid"); near(fit.value, 0.041025641025641026, "desert/f3.value"); near(fit.world_mean, 0.03398926654740608, "desert/f3.world_mean"); near(fit.ratio, 1.207017543859649, "desert/f3.ratio"); assert_eq!(fit.verdict, "match", "desert/f3.verdict");
    let fit = civ_culture_terrain_fit("desert", &agg.by_faction[4].terrain_mix, &agg.world_mean_terrain).expect("desert/f4");
    assert_eq!(fit.key, "arid"); near(fit.value, 0.015706806282722512, "desert/f4.value"); near(fit.world_mean, 0.03398926654740608, "desert/f4.world_mean"); near(fit.ratio, 0.4621107743179939, "desert/f4.ratio"); assert_eq!(fit.verdict, "mismatch", "desert/f4.verdict");
    let fit = civ_culture_terrain_fit("desert", &agg.by_faction[5].terrain_mix, &agg.world_mean_terrain).expect("desert/f5");
    assert_eq!(fit.key, "arid"); near(fit.value, 0.02072538860103627, "desert/f5.value"); near(fit.world_mean, 0.03398926654740608, "desert/f5.world_mean"); near(fit.ratio, 0.6097627488410144, "desert/f5.ratio"); assert_eq!(fit.verdict, "mismatch", "desert/f5.verdict");
    let fit = civ_culture_terrain_fit("desert", &agg.by_faction[6].terrain_mix, &agg.world_mean_terrain).expect("desert/f6");
    assert_eq!(fit.key, "arid"); near(fit.value, 0.0, "desert/f6.value"); near(fit.world_mean, 0.03398926654740608, "desert/f6.world_mean"); near(fit.ratio, 0.0, "desert/f6.ratio"); assert_eq!(fit.verdict, "mismatch", "desert/f6.verdict");
    let fit = civ_culture_terrain_fit("riverlands", &agg.by_faction[0].terrain_mix, &agg.world_mean_terrain).expect("riverlands/f0");
    assert_eq!(fit.key, "river"); near(fit.value, 0.0, "riverlands/f0.value"); near(fit.world_mean, 0.9203935599284436, "riverlands/f0.world_mean"); near(fit.ratio, 0.0, "riverlands/f0.ratio"); assert_eq!(fit.verdict, "mismatch", "riverlands/f0.verdict");
    let fit = civ_culture_terrain_fit("riverlands", &agg.by_faction[1].terrain_mix, &agg.world_mean_terrain).expect("riverlands/f1");
    assert_eq!(fit.key, "river"); near(fit.value, 0.9319371727748691, "riverlands/f1.value"); near(fit.world_mean, 0.9203935599284436, "riverlands/f1.world_mean"); near(fit.ratio, 1.0125420400022387, "riverlands/f1.ratio"); assert_eq!(fit.verdict, "typical", "riverlands/f1.verdict");
    let fit = civ_culture_terrain_fit("riverlands", &agg.by_faction[2].terrain_mix, &agg.world_mean_terrain).expect("riverlands/f2");
    assert_eq!(fit.key, "river"); near(fit.value, 0.927461139896373, "riverlands/f2.value"); near(fit.world_mean, 0.9203935599284436, "riverlands/f2.world_mean"); near(fit.ratio, 1.0076788672537853, "riverlands/f2.ratio"); assert_eq!(fit.verdict, "typical", "riverlands/f2.verdict");
    let fit = civ_culture_terrain_fit("riverlands", &agg.by_faction[3].terrain_mix, &agg.world_mean_terrain).expect("riverlands/f3");
    assert_eq!(fit.key, "river"); near(fit.value, 0.9179487179487179, "riverlands/f3.value"); near(fit.world_mean, 0.9203935599284436, "riverlands/f3.world_mean"); near(fit.ratio, 0.9973436993845157, "riverlands/f3.ratio"); assert_eq!(fit.verdict, "typical", "riverlands/f3.verdict");
    let fit = civ_culture_terrain_fit("riverlands", &agg.by_faction[4].terrain_mix, &agg.world_mean_terrain).expect("riverlands/f4");
    assert_eq!(fit.key, "river"); near(fit.value, 0.9267015706806283, "riverlands/f4.value"); near(fit.world_mean, 0.9203935599284436, "riverlands/f4.world_mean"); near(fit.ratio, 1.0068536015752598, "riverlands/f4.ratio"); assert_eq!(fit.verdict, "typical", "riverlands/f4.verdict");
    let fit = civ_culture_terrain_fit("riverlands", &agg.by_faction[5].terrain_mix, &agg.world_mean_terrain).expect("riverlands/f5");
    assert_eq!(fit.key, "river"); near(fit.value, 0.9119170984455959, "riverlands/f5.value"); near(fit.world_mean, 0.9203935599284436, "riverlands/f5.world_mean"); near(fit.ratio, 0.990790394618247, "riverlands/f5.ratio"); assert_eq!(fit.verdict, "typical", "riverlands/f5.verdict");
    let fit = civ_culture_terrain_fit("riverlands", &agg.by_faction[6].terrain_mix, &agg.world_mean_terrain).expect("riverlands/f6");
    assert_eq!(fit.key, "river"); near(fit.value, 0.0, "riverlands/f6.value"); near(fit.world_mean, 0.9203935599284436, "riverlands/f6.world_mean"); near(fit.ratio, 0.0, "riverlands/f6.ratio"); assert_eq!(fit.verdict, "mismatch", "riverlands/f6.verdict");
    let fit = civ_culture_terrain_fit("sylvan", &agg.by_faction[0].terrain_mix, &agg.world_mean_terrain).expect("sylvan/f0");
    assert_eq!(fit.key, "forest"); near(fit.value, 0.0, "sylvan/f0.value"); near(fit.world_mean, 0.313953488372093, "sylvan/f0.world_mean"); near(fit.ratio, 0.0, "sylvan/f0.ratio"); assert_eq!(fit.verdict, "mismatch", "sylvan/f0.verdict");
    let fit = civ_culture_terrain_fit("sylvan", &agg.by_faction[1].terrain_mix, &agg.world_mean_terrain).expect("sylvan/f1");
    assert_eq!(fit.key, "forest"); near(fit.value, 0.3089005235602094, "sylvan/f1.value"); near(fit.world_mean, 0.313953488372093, "sylvan/f1.world_mean"); near(fit.ratio, 0.9839053713399263, "sylvan/f1.ratio"); assert_eq!(fit.verdict, "typical", "sylvan/f1.verdict");
    let fit = civ_culture_terrain_fit("sylvan", &agg.by_faction[2].terrain_mix, &agg.world_mean_terrain).expect("sylvan/f2");
    assert_eq!(fit.key, "forest"); near(fit.value, 0.3005181347150259, "sylvan/f2.value"); near(fit.world_mean, 0.313953488372093, "sylvan/f2.world_mean"); near(fit.ratio, 0.9572059105737862, "sylvan/f2.ratio"); assert_eq!(fit.verdict, "typical", "sylvan/f2.verdict");
    let fit = civ_culture_terrain_fit("sylvan", &agg.by_faction[3].terrain_mix, &agg.world_mean_terrain).expect("sylvan/f3");
    assert_eq!(fit.key, "forest"); near(fit.value, 0.3282051282051282, "sylvan/f3.value"); near(fit.world_mean, 0.313953488372093, "sylvan/f3.world_mean"); near(fit.ratio, 1.0453941120607788, "sylvan/f3.ratio"); assert_eq!(fit.verdict, "typical", "sylvan/f3.verdict");
    let fit = civ_culture_terrain_fit("sylvan", &agg.by_faction[4].terrain_mix, &agg.world_mean_terrain).expect("sylvan/f4");
    assert_eq!(fit.key, "forest"); near(fit.value, 0.3403141361256545, "sylvan/f4.value"); near(fit.world_mean, 0.313953488372093, "sylvan/f4.world_mean"); near(fit.ratio, 1.083963544696529, "sylvan/f4.ratio"); assert_eq!(fit.verdict, "typical", "sylvan/f4.verdict");
    let fit = civ_culture_terrain_fit("sylvan", &agg.by_faction[5].terrain_mix, &agg.world_mean_terrain).expect("sylvan/f5");
    assert_eq!(fit.key, "forest"); near(fit.value, 0.30569948186528495, "sylvan/f5.value"); near(fit.world_mean, 0.313953488372093, "sylvan/f5.world_mean"); near(fit.ratio, 0.9737094607560928, "sylvan/f5.ratio"); assert_eq!(fit.verdict, "typical", "sylvan/f5.verdict");
    let fit = civ_culture_terrain_fit("sylvan", &agg.by_faction[6].terrain_mix, &agg.world_mean_terrain).expect("sylvan/f6");
    assert_eq!(fit.key, "forest"); near(fit.value, 0.0, "sylvan/f6.value"); near(fit.world_mean, 0.313953488372093, "sylvan/f6.world_mean"); near(fit.ratio, 0.0, "sylvan/f6.ratio"); assert_eq!(fit.verdict, "mismatch", "sylvan/f6.verdict");
    let fit = civ_culture_terrain_fit("maritime", &agg.by_faction[0].terrain_mix, &agg.world_mean_terrain).expect("maritime/f0");
    assert_eq!(fit.key, "coast"); near(fit.value, 0.0, "maritime/f0.value"); near(fit.world_mean, 0.09660107334525939, "maritime/f0.world_mean"); near(fit.ratio, 0.0, "maritime/f0.ratio"); assert_eq!(fit.verdict, "mismatch", "maritime/f0.verdict");
    let fit = civ_culture_terrain_fit("maritime", &agg.by_faction[1].terrain_mix, &agg.world_mean_terrain).expect("maritime/f1");
    assert_eq!(fit.key, "coast"); near(fit.value, 0.08900523560209424, "maritime/f1.value"); near(fit.world_mean, 0.09660107334525939, "maritime/f1.world_mean"); near(fit.ratio, 0.9213690129920497, "maritime/f1.ratio"); assert_eq!(fit.verdict, "typical", "maritime/f1.verdict");
    let fit = civ_culture_terrain_fit("maritime", &agg.by_faction[2].terrain_mix, &agg.world_mean_terrain).expect("maritime/f2");
    assert_eq!(fit.key, "coast"); near(fit.value, 0.10880829015544041, "maritime/f2.value"); near(fit.world_mean, 0.09660107334525939, "maritime/f2.world_mean"); near(fit.ratio, 1.1263672999424295, "maritime/f2.ratio"); assert_eq!(fit.verdict, "typical", "maritime/f2.verdict");
    let fit = civ_culture_terrain_fit("maritime", &agg.by_faction[3].terrain_mix, &agg.world_mean_terrain).expect("maritime/f3");
    assert_eq!(fit.key, "coast"); near(fit.value, 0.11282051282051282, "maritime/f3.value"); near(fit.world_mean, 0.09660107334525939, "maritime/f3.world_mean"); near(fit.ratio, 1.1679012345679014, "maritime/f3.ratio"); assert_eq!(fit.verdict, "match", "maritime/f3.verdict");
    let fit = civ_culture_terrain_fit("maritime", &agg.by_faction[4].terrain_mix, &agg.world_mean_terrain).expect("maritime/f4");
    assert_eq!(fit.key, "coast"); near(fit.value, 0.08900523560209424, "maritime/f4.value"); near(fit.world_mean, 0.09660107334525939, "maritime/f4.world_mean"); near(fit.ratio, 0.9213690129920497, "maritime/f4.ratio"); assert_eq!(fit.verdict, "typical", "maritime/f4.verdict");
    let fit = civ_culture_terrain_fit("maritime", &agg.by_faction[5].terrain_mix, &agg.world_mean_terrain).expect("maritime/f5");
    assert_eq!(fit.key, "coast"); near(fit.value, 0.09844559585492228, "maritime/f5.value"); near(fit.world_mean, 0.09660107334525939, "maritime/f5.world_mean"); near(fit.ratio, 1.0190942237574363, "maritime/f5.ratio"); assert_eq!(fit.verdict, "typical", "maritime/f5.verdict");
    let fit = civ_culture_terrain_fit("maritime", &agg.by_faction[6].terrain_mix, &agg.world_mean_terrain).expect("maritime/f6");
    assert_eq!(fit.key, "coast"); near(fit.value, 0.0, "maritime/f6.value"); near(fit.world_mean, 0.09660107334525939, "maritime/f6.world_mean"); near(fit.ratio, 0.0, "maritime/f6.ratio"); assert_eq!(fit.verdict, "mismatch", "maritime/f6.verdict");
    assert!(civ_culture_terrain_fit("common", &agg.by_faction[0].terrain_mix, &agg.world_mean_terrain).is_none(), "common/f0 must have no verdict");
    assert!(civ_culture_terrain_fit("common", &agg.by_faction[1].terrain_mix, &agg.world_mean_terrain).is_none(), "common/f1 must have no verdict");
    assert!(civ_culture_terrain_fit("common", &agg.by_faction[2].terrain_mix, &agg.world_mean_terrain).is_none(), "common/f2 must have no verdict");
    assert!(civ_culture_terrain_fit("common", &agg.by_faction[3].terrain_mix, &agg.world_mean_terrain).is_none(), "common/f3 must have no verdict");
    assert!(civ_culture_terrain_fit("common", &agg.by_faction[4].terrain_mix, &agg.world_mean_terrain).is_none(), "common/f4 must have no verdict");
    assert!(civ_culture_terrain_fit("common", &agg.by_faction[5].terrain_mix, &agg.world_mean_terrain).is_none(), "common/f5 must have no verdict");
    assert!(civ_culture_terrain_fit("common", &agg.by_faction[6].terrain_mix, &agg.world_mean_terrain).is_none(), "common/f6 must have no verdict");
    assert!(civ_culture_terrain_fit("imperial", &agg.by_faction[0].terrain_mix, &agg.world_mean_terrain).is_none(), "imperial/f0 must have no verdict");
    assert!(civ_culture_terrain_fit("imperial", &agg.by_faction[1].terrain_mix, &agg.world_mean_terrain).is_none(), "imperial/f1 must have no verdict");
    assert!(civ_culture_terrain_fit("imperial", &agg.by_faction[2].terrain_mix, &agg.world_mean_terrain).is_none(), "imperial/f2 must have no verdict");
    assert!(civ_culture_terrain_fit("imperial", &agg.by_faction[3].terrain_mix, &agg.world_mean_terrain).is_none(), "imperial/f3 must have no verdict");
    assert!(civ_culture_terrain_fit("imperial", &agg.by_faction[4].terrain_mix, &agg.world_mean_terrain).is_none(), "imperial/f4 must have no verdict");
    assert!(civ_culture_terrain_fit("imperial", &agg.by_faction[5].terrain_mix, &agg.world_mean_terrain).is_none(), "imperial/f5 must have no verdict");
    assert!(civ_culture_terrain_fit("imperial", &agg.by_faction[6].terrain_mix, &agg.world_mean_terrain).is_none(), "imperial/f6 must have no verdict");
}
