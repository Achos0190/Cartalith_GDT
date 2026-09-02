//! Golden-parity tests for `ECONOMY_SCOPE.md`'s two remaining unblocked
//! ports: `_civPlaceSmelting` (reference HTML line 24208, v1.31 —
//! charcoal-limited iron) and `_civSaltAccess` (24430, v1.37 — sea salt vs.
//! deposit vs. salt lake), together with the fidelity detail that is easiest
//! to get wrong in either of them: the smelting disc uses the **per-tier
//! catchment radius** (`_civCatchmentRadiusCells`), while the salt-deposit
//! window uses `_civPlaceResourceContext`'s **own default**,
//! `max(3, round(GW/128))` — two different radii, both asserted per place.
//!
//! # The harness
//!
//! Node `vm.runInContext`, transient per this project's established practice
//! (not checked in), same as `golden_parity_faction_aggregates.rs`. **Whole
//! `<script>` blocks, not line slices**, each asserted by its real
//! delimiters (the line before the slice *is* `<script>`, the line after
//! *is* `</script>`) and each compiled with `new vm.Script(...)` before
//! being run, because a real parser is a stronger slice-boundary guarantee
//! than any hand-rolled scanner. Four blocks are needed here, one more than
//! that file: #1 (2084-14556), #2 (14563-26720) and — because
//! `_civSaltAccess` calls `_civPlaceNavigability`, whose branch (c) calls
//! `_umSiteProfile`, which reads `UME` at reference line 28174 — the two
//! urban-morphology blocks #3 (26723-28161) and #4 (28167-31103).
//!
//! # The world is INJECTED, not regenerated
//!
//! This is the one real difference from `golden_parity_faction_aggregates.rs`,
//! and it is deliberate. That file regenerates the world inside the vm and
//! then discloses that `tempField`/`rainField` diverge from this port's by
//! 1-3 f32 ULP. Here the harness instead **writes this port's own arrays
//! into the context's `let` bindings** — `field`, `tempField`, `rainField`,
//! `flowField`, `state.seaLevel`/`mapWidthKm`/`world`, plus
//! `currentResourcePotentials()` and `currentWaterBodies()` overridden to
//! return this port's own outputs. Both sides then read **byte-identical**
//! inputs by construction, so the climate chain's known ULP divergence
//! cannot reach these comparisons at all and every number below is compared
//! at `1e-12` relative or exactly.
//!
//! That the injection actually took is not assumed: the reference's own
//! `buildBiomeRaster()` is re-run inside the context over the injected
//! `tempField`/`rainField`/water bodies and FNV-1a-64 hashed, and that hash
//! is re-asserted below against this port's `build_biome_raster`. It matches
//! exactly in all three cases. The remaining inputs (`field`, `rain`, and
//! the `iron`/`timber`/`salt` potentials) are hashed on this side and pinned
//! to the values the harness was handed.
//!
//! **Two reference globals are stubbed, and neither can reach an output.**
//! `_umSiteProfile` fills `carryK` from `currentCarryingCapacity()` (which
//! chains through `currentSoil` -> `currentLithology` -> `plateCrust` to
//! `plates`/`plateId`, tectonic state this harness does not inject) and
//! `floodplain` from `currentFloodField()`. Both are stubbed to zero rather
//! than fabricated, which is sound because the two consumers here read a
//! disjoint set of that profile's fields: `_civPlaceNavigability` reads only
//! `coastDistKm`/`riverDistKm`/`riverOrder`, and `_civSaltAccess` reads only
//! `biome`/`rain`. Everything on the path that *is* reachable —
//! `_civCoastDistField`, `_civRiverPolylines`, `_umSiteKindFromTerrain`,
//! `buildBiomeRaster` — runs for real over the injected fields.
//!
//! # `nav` is captured, not recomputed
//!
//! [`civ_salt_access`]'s first branch takes the navigability verdict as a
//! parameter, the same convention `FoodShedInput` uses. So the harness
//! captures the reference's own `_civPlaceNavigability(p).kind` per place
//! and this test feeds **that** value in — the salt rule is therefore
//! compared on identical inputs. It is deliberately *not* compared against
//! this port's `place_navigability`, which implements branches (a) and (b)
//! only (`trade`'s module doc records that divergence and why), and which is
//! a different function with its own tests.
//!
//! # Cases, and what each is for
//!
//! * **Case 0** (64x48, seed 24601, non-wrapping) and **case 1** (48x36,
//!   seed 314159, wrapping) are real generated worlds, at the default
//!   800 km map width. Case 1 reaches the salt-lake branch (place 5: a
//!   `lake` biome cell at rain 0.2566) and `ore_rich` (place 6) on real
//!   terrain, unprompted.
//! * **Case 2** is deliberately **synthetic and rule-generated**, the same
//!   device `golden_parity_faction_aggregates.rs`'s `synthetic_territory`
//!   uses, and for the same reason: what a real 800 km-wide world cannot
//!   give is a catchment radius bigger than 2 cells. At 24x18 over a 50 km
//!   map a hamlet's disc is 1 cell and a metropolis's is 14, so the
//!   `dx*dx+dy*dy>r2` circle rejection, the `xx<0||xx>=GW` **un-wrapped**
//!   clip and the `field[i]<sea` ocean exclusion are all genuinely
//!   exercised. It also separates ore from fuel geographically, which is
//!   what makes the Elba constraint visible: place 0 sits on ore with almost
//!   no woodland (`fuel_poor`), place 10 on woodland with a little ore
//!   (`ore_rich`), place 2 spans both. The rule is byte-identical on both
//!   sides and the arrays are hashed, so the two provably read the same
//!   world.
//!
//! # Emptiness / shape — because four subsystems in this port have been bitten
//!
//! A slice that parses and silently produces nothing has cost this project
//! four times. So the fixtures are asserted to be non-degenerate rather than
//! merely equal: all four `SaltAccess::source` literals occur across the
//! three cases, both `limited_by` values occur, `fuel_poor` and `ore_rich`
//! are each true somewhere and false somewhere, the catchment radii span
//! 1..=14, at least one place has a non-zero `iron_kg_yr`, and case 2's
//! discriminating pairs are named explicitly: places 3 and 4 are the same
//! `lake` biome at rain 0.20 vs 0.55 (only the first is a salt lake), and
//! the salt means 0.1448 / 0.2690 / 0.3310 straddle the 0.25 threshold from
//! both sides.

use cartalith_civ::trade::{civ_place_smelting, civ_salt_access, NavKind, PlaceWorld};
use cartalith_civ::{ResourcePotentials, SettlementKind};

/// One captured place: its inputs, both radii, and every field of both
/// return values.
struct Expect {
    x: usize,
    y: usize,
    kind: SettlementKind,
    /// `_civCatchmentRadiusCells(_CIV_CATCHMENT_KM2[kind])`, captured from
    /// the reference so the disc bound itself is compared, not just its sum.
    radius: usize,
    /// `_civPlaceNavigability(p).kind`, captured from the reference.
    nav: NavKind,
    iron_kg_yr: f64,
    charcoal_kg_yr: f64,
    ore_kg_yr: f64,
    woodland_ha: f64,
    limited_by: &'static str,
    fuel_poor: bool,
    ore_rich: bool,
    coppice_ha_needed: f64,
    /// `_civPlaceResourceContext(p).mean.salt` at that function's OWN
    /// default radius — the number `_civSaltAccess` thresholds at 0.25.
    salt_mean: f64,
    has: bool,
    source: &'static str,
}

struct Case {
    field: Vec<f32>,
    temp: Vec<f32>,
    rain: Vec<f32>,
    res: ResourcePotentials,
    biome: Vec<u8>,
    gw: usize,
    gh: usize,
    sea: f64,
    map_width_km: f64,
}

// ------------------------------------------------------------- expectations

/// Captured from the reference for case0.
const CASE0_EXPECT: &[Expect] = &[
    Expect { x: 0, y: 0, kind: SettlementKind::Capital, radius: 2, nav: NavKind::River, iron_kg_yr: 0.0, charcoal_kg_yr: 0.0, ore_kg_yr: 54843747.98834324, woodland_ha: 0.0, limited_by: "fuel", fuel_poor: true, ore_rich: false, coppice_ha_needed: 0.0, salt_mean: 0.0, has: false, source: "none" },
    Expect { x: 63, y: 24, kind: SettlementKind::Metropolis, radius: 2, nav: NavKind::Sea, iron_kg_yr: 0.0, charcoal_kg_yr: 93750000.0, ore_kg_yr: 0.0, woodland_ha: 93750.0, limited_by: "ore", fuel_poor: false, ore_rich: false, coppice_ha_needed: 0.0, salt_mean: 0.0, has: true, source: "sea salt" },
    Expect { x: 32, y: 47, kind: SettlementKind::City, radius: 1, nav: NavKind::River, iron_kg_yr: 0.0, charcoal_kg_yr: 0.0, ore_kg_yr: 0.0, woodland_ha: 0.0, limited_by: "ore", fuel_poor: false, ore_rich: false, coppice_ha_needed: 0.0, salt_mean: 0.0, has: false, source: "none" },
    Expect { x: 21, y: 16, kind: SettlementKind::Town, radius: 1, nav: NavKind::Sea, iron_kg_yr: 0.0, charcoal_kg_yr: 0.0, ore_kg_yr: 0.0, woodland_ha: 0.0, limited_by: "ore", fuel_poor: false, ore_rich: false, coppice_ha_needed: 0.0, salt_mean: 0.4336773478067838, has: true, source: "sea salt" },
    Expect { x: 32, y: 24, kind: SettlementKind::Village, radius: 1, nav: NavKind::Sea, iron_kg_yr: 0.0, charcoal_kg_yr: 0.0, ore_kg_yr: 0.0, woodland_ha: 0.0, limited_by: "ore", fuel_poor: false, ore_rich: false, coppice_ha_needed: 0.0, salt_mean: 0.0, has: true, source: "sea salt" },
    Expect { x: 42, y: 32, kind: SettlementKind::Hamlet, radius: 1, nav: NavKind::Sea, iron_kg_yr: 0.0, charcoal_kg_yr: 0.0, ore_kg_yr: 0.0, woodland_ha: 0.0, limited_by: "ore", fuel_poor: false, ore_rich: false, coppice_ha_needed: 0.0, salt_mean: 0.0, has: true, source: "sea salt" },
    Expect { x: 16, y: 36, kind: SettlementKind::Capital, radius: 2, nav: NavKind::River, iron_kg_yr: 0.0, charcoal_kg_yr: 0.0, ore_kg_yr: 0.0, woodland_ha: 0.0, limited_by: "ore", fuel_poor: false, ore_rich: false, coppice_ha_needed: 0.0, salt_mean: 0.0, has: false, source: "none" },
    Expect { x: 48, y: 12, kind: SettlementKind::City, radius: 1, nav: NavKind::Sea, iron_kg_yr: 0.0, charcoal_kg_yr: 0.0, ore_kg_yr: 0.0, woodland_ha: 0.0, limited_by: "ore", fuel_poor: false, ore_rich: false, coppice_ha_needed: 0.0, salt_mean: 0.0, has: true, source: "sea salt" },
    Expect { x: 10, y: 24, kind: SettlementKind::Town, radius: 1, nav: NavKind::River, iron_kg_yr: 0.0, charcoal_kg_yr: 0.0, ore_kg_yr: 0.0, woodland_ha: 0.0, limited_by: "ore", fuel_poor: false, ore_rich: false, coppice_ha_needed: 0.0, salt_mean: 0.0, has: false, source: "none" },
    Expect { x: 53, y: 40, kind: SettlementKind::Metropolis, radius: 2, nav: NavKind::River, iron_kg_yr: 0.0, charcoal_kg_yr: 164951988.49588633, ore_kg_yr: 0.0, woodland_ha: 164951.98849588633, limited_by: "ore", fuel_poor: false, ore_rich: false, coppice_ha_needed: 0.0, salt_mean: 0.0, has: false, source: "none" },
];

/// Captured from the reference for case1.
const CASE1_EXPECT: &[Expect] = &[
    Expect { x: 0, y: 0, kind: SettlementKind::Capital, radius: 1, nav: NavKind::Sea, iron_kg_yr: 0.0, charcoal_kg_yr: 0.0, ore_kg_yr: 0.0, woodland_ha: 0.0, limited_by: "ore", fuel_poor: false, ore_rich: false, coppice_ha_needed: 0.0, salt_mean: 0.0, has: true, source: "sea salt" },
    Expect { x: 47, y: 18, kind: SettlementKind::Metropolis, radius: 2, nav: NavKind::Sea, iron_kg_yr: 0.0, charcoal_kg_yr: 0.0, ore_kg_yr: 0.0, woodland_ha: 0.0, limited_by: "ore", fuel_poor: false, ore_rich: false, coppice_ha_needed: 0.0, salt_mean: 0.0, has: true, source: "sea salt" },
    Expect { x: 24, y: 35, kind: SettlementKind::City, radius: 1, nav: NavKind::River, iron_kg_yr: 0.0, charcoal_kg_yr: 0.0, ore_kg_yr: 64999997.61581421, woodland_ha: 0.0, limited_by: "fuel", fuel_poor: true, ore_rich: false, coppice_ha_needed: 0.0, salt_mean: 0.0, has: false, source: "none" },
    Expect { x: 16, y: 12, kind: SettlementKind::Town, radius: 1, nav: NavKind::River, iron_kg_yr: 0.0, charcoal_kg_yr: 119196227.85515256, ore_kg_yr: 0.0, woodland_ha: 119196.22785515257, limited_by: "ore", fuel_poor: false, ore_rich: false, coppice_ha_needed: 0.0, salt_mean: 0.0, has: false, source: "none" },
    Expect { x: 24, y: 18, kind: SettlementKind::Village, radius: 1, nav: NavKind::Sea, iron_kg_yr: 0.0, charcoal_kg_yr: 0.0, ore_kg_yr: 0.0, woodland_ha: 0.0, limited_by: "ore", fuel_poor: false, ore_rich: false, coppice_ha_needed: 0.0, salt_mean: 0.0, has: true, source: "sea salt" },
    Expect { x: 32, y: 24, kind: SettlementKind::Hamlet, radius: 1, nav: NavKind::River, iron_kg_yr: 0.0, charcoal_kg_yr: 0.0, ore_kg_yr: 0.0, woodland_ha: 0.0, limited_by: "ore", fuel_poor: false, ore_rich: false, coppice_ha_needed: 0.0, salt_mean: 0.0, has: true, source: "salt lake" },
    Expect { x: 12, y: 27, kind: SettlementKind::Capital, radius: 1, nav: NavKind::River, iron_kg_yr: 4950000.073760749, charcoal_kg_yr: 138888888.8888889, ore_kg_yr: 15000000.22351742, woodland_ha: 138888.8888888889, limited_by: "ore", fuel_poor: false, ore_rich: true, coppice_ha_needed: 33165.00049419702, salt_mean: 0.0, has: false, source: "none" },
    Expect { x: 36, y: 9, kind: SettlementKind::City, radius: 1, nav: NavKind::River, iron_kg_yr: 0.0, charcoal_kg_yr: 0.0, ore_kg_yr: 0.0, woodland_ha: 0.0, limited_by: "ore", fuel_poor: false, ore_rich: false, coppice_ha_needed: 0.0, salt_mean: 0.0, has: false, source: "none" },
    Expect { x: 8, y: 18, kind: SettlementKind::Town, radius: 1, nav: NavKind::Sea, iron_kg_yr: 0.0, charcoal_kg_yr: 0.0, ore_kg_yr: 0.0, woodland_ha: 0.0, limited_by: "ore", fuel_poor: false, ore_rich: false, coppice_ha_needed: 0.0, salt_mean: 0.0, has: true, source: "sea salt" },
    Expect { x: 40, y: 30, kind: SettlementKind::Metropolis, radius: 2, nav: NavKind::River, iron_kg_yr: 0.0, charcoal_kg_yr: 0.0, ore_kg_yr: 16249999.403953552, woodland_ha: 0.0, limited_by: "fuel", fuel_poor: true, ore_rich: false, coppice_ha_needed: 0.0, salt_mean: 0.0, has: false, source: "none" },
];

/// Captured from the reference for case2.
const CASE2_EXPECT: &[Expect] = &[
    Expect { x: 6, y: 8, kind: SettlementKind::City, radius: 8, nav: NavKind::None, iron_kg_yr: 58302.23726148827, charcoal_kg_yr: 390624.98965197144, ore_kg_yr: 28437500.423751745, woodland_ha: 390.6249896519714, limited_by: "fuel", fuel_poor: true, ore_rich: false, coppice_ha_needed: 390.6249896519714, salt_mean: 0.0, has: false, source: "none" },
    Expect { x: 18, y: 8, kind: SettlementKind::City, radius: 8, nav: NavKind::None, iron_kg_yr: 0.0, charcoal_kg_yr: 46484373.76858471, ore_kg_yr: 0.0, woodland_ha: 46484.373768584715, limited_by: "ore", fuel_poor: false, ore_rich: false, coppice_ha_needed: 0.0, salt_mean: 0.2689655279291087, has: true, source: "salt deposit" },
    Expect { x: 12, y: 8, kind: SettlementKind::City, radius: 8, nav: NavKind::None, iron_kg_yr: 4372667.794611627, charcoal_kg_yr: 29296874.223897904, ore_kg_yr: 18750000.279396757, woodland_ha: 29296.874223897903, limited_by: "fuel", fuel_poor: false, ore_rich: false, coppice_ha_needed: 29296.874223897903, salt_mean: 0.0, has: false, source: "none" },
    Expect { x: 11, y: 5, kind: SettlementKind::Village, radius: 1, nav: NavKind::None, iron_kg_yr: 0.0, charcoal_kg_yr: 0.0, ore_kg_yr: 0.0, woodland_ha: 0.0, limited_by: "ore", fuel_poor: false, ore_rich: false, coppice_ha_needed: 0.0, salt_mean: 0.0, has: true, source: "salt lake" },
    Expect { x: 12, y: 5, kind: SettlementKind::Village, radius: 1, nav: NavKind::None, iron_kg_yr: 0.0, charcoal_kg_yr: 0.0, ore_kg_yr: 0.0, woodland_ha: 0.0, limited_by: "ore", fuel_poor: false, ore_rich: false, coppice_ha_needed: 0.0, salt_mean: 0.0, has: false, source: "none" },
    Expect { x: 18, y: 10, kind: SettlementKind::Town, radius: 3, nav: NavKind::None, iron_kg_yr: 0.0, charcoal_kg_yr: 11328124.699907167, ore_kg_yr: 0.0, woodland_ha: 11328.124699907166, limited_by: "ore", fuel_poor: false, ore_rich: false, coppice_ha_needed: 0.0, salt_mean: 0.3310344959127492, has: true, source: "salt deposit" },
    Expect { x: 0, y: 0, kind: SettlementKind::Capital, radius: 10, nav: NavKind::Sea, iron_kg_yr: 0.0, charcoal_kg_yr: 0.0, ore_kg_yr: 15000000.223517407, woodland_ha: 0.0, limited_by: "fuel", fuel_poor: true, ore_rich: false, coppice_ha_needed: 0.0, salt_mean: 0.0, has: true, source: "sea salt" },
    Expect { x: 23, y: 17, kind: SettlementKind::Metropolis, radius: 14, nav: NavKind::Sea, iron_kg_yr: 0.0, charcoal_kg_yr: 34765624.07902552, ore_kg_yr: 0.0, woodland_ha: 34765.624079025525, limited_by: "ore", fuel_poor: false, ore_rich: false, coppice_ha_needed: 0.0, salt_mean: 0.0, has: true, source: "sea salt" },
    Expect { x: 1, y: 9, kind: SettlementKind::Town, radius: 3, nav: NavKind::Sea, iron_kg_yr: 0.0, charcoal_kg_yr: 0.0, ore_kg_yr: 312500.00465661293, woodland_ha: 0.0, limited_by: "fuel", fuel_poor: true, ore_rich: false, coppice_ha_needed: 0.0, salt_mean: 0.0, has: true, source: "sea salt" },
    Expect { x: 14, y: 3, kind: SettlementKind::Hamlet, radius: 1, nav: NavKind::None, iron_kg_yr: 0.0, charcoal_kg_yr: 1562499.9586078858, ore_kg_yr: 0.0, woodland_ha: 1562.4999586078857, limited_by: "ore", fuel_poor: false, ore_rich: false, coppice_ha_needed: 0.0, salt_mean: 0.0, has: false, source: "none" },
    Expect { x: 15, y: 8, kind: SettlementKind::City, radius: 8, nav: NavKind::None, iron_kg_yr: 1959375.0291969622, charcoal_kg_yr: 44140623.830672875, ore_kg_yr: 5937500.088475643, woodland_ha: 44140.62383067288, limited_by: "ore", fuel_poor: false, ore_rich: true, coppice_ha_needed: 13127.812695619648, salt_mean: 0.14482759196182776, has: false, source: "none" },
];

// -------------------------------------------------------------- the worlds

fn build_real(gw: usize, gh: usize, seed: i32, world: bool) -> Case {
    let mut p = cartalith_engine::WorldParams::defaults(gw, gh, seed);
    p.world = world;
    p.climate.w_iters = 12;
    let ws = cartalith_engine::generate_terrain(&p);
    assert!((ws.sea_level - 0.42).abs() < 1e-9, "sea_level: harness assumption broken");
    assert!((p.map_width_km - 800.0).abs() < 1e-9, "map_width_km: harness assumption broken");
    let wb =
        cartalith_civ::build_water_bodies(&ws.field, gw, gh, ws.sea_level, world, Some(&ws.rainfall));
    let lith = cartalith_civ::build_lithology(
        &ws.field,
        &ws.age_field,
        &ws.volcanic_field,
        &ws.crust_field,
        &ws.resistance_field,
        &ws.rainfall,
        ws.sea_level,
    );
    let biome = cartalith_civ::build_biome_raster(&wb.classification, &ws.temperature, &ws.rainfall);
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
    Case {
        field: ws.field,
        temp: ws.temperature,
        rain: ws.rainfall,
        res,
        biome,
        gw,
        gh,
        sea: ws.sea_level,
        map_width_km: p.map_width_km,
    }
}

// The synthetic rule, byte-for-byte identical to the harness's input (which
// was written by this same code). Ocean in the two leftmost columns and the
// two bottom rows; a 4x4 lake block at x in 10..14, y in 4..8; rain 0.20 west
// of x=12 and 0.55 east of it, so the lake block STRADDLES the 0.30 aridity
// threshold and places 3 and 4 differ on it alone. Iron in x in 4..10,
// timber in x in 14..22, salt in the x in 16..20 / y in 8..12 box -- ore and
// fuel deliberately disjoint, which is what makes the Elba constraint
// visible at all.
const SGW: usize = 24;
const SGH: usize = 18;
const SSEA: f64 = 0.42;
const SMAPKM: f64 = 50.0;

fn synth_field() -> Vec<f32> {
    let mut v = vec![0.0f32; SGW * SGH];
    for y in 0..SGH {
        for x in 0..SGW {
            v[y * SGW + x] =
                if x < 2 || y >= SGH - 2 { 0.10 } else { 0.55 + 0.001 * ((x * 7 + y * 13) % 100) as f32 };
        }
    }
    v
}

fn synth_rain() -> Vec<f32> {
    let mut v = vec![0.0f32; SGW * SGH];
    for y in 0..SGH {
        for x in 0..SGW {
            v[y * SGW + x] = if x < 12 { 0.20 } else { 0.55 };
        }
    }
    v
}

fn synth_wb(field: &[f32]) -> Vec<u8> {
    let mut v = vec![0u8; SGW * SGH];
    for y in 0..SGH {
        for x in 0..SGW {
            let i = y * SGW + x;
            v[i] = if (field[i] as f64) < SSEA {
                1
            } else if (10..14).contains(&x) && (4..8).contains(&y) {
                2
            } else {
                0
            };
        }
    }
    v
}

fn synth_pots() -> ResourcePotentials {
    let z = || vec![0.0f32; SGW * SGH];
    let (mut iron, mut timber, mut salt) = (z(), z(), z());
    for y in 0..SGH {
        for x in 0..SGW {
            let i = y * SGW + x;
            if (4..10).contains(&x) {
                iron[i] = 0.8;
            }
            if (14..22).contains(&x) {
                timber[i] = 0.9;
            }
            if (16..20).contains(&x) && (8..12).contains(&y) {
                salt[i] = 0.6;
            }
        }
    }
    ResourcePotentials {
        copper: z(),
        tin: z(),
        iron,
        gold: z(),
        salt,
        timber,
        lead: z(),
        silver: z(),
        clay: z(),
        buildstone: z(),
        flint: z(),
        obsidian: z(),
        gems: z(),
        sulfur: z(),
        alum: z(),
    }
}

fn build_synth() -> Case {
    let field = synth_field();
    let temp = vec![15.0f32; SGW * SGH];
    let rain = synth_rain();
    let wb = synth_wb(&field);
    let biome = cartalith_civ::build_biome_raster(&wb, &temp, &rain);
    Case {
        field,
        temp,
        rain,
        res: synth_pots(),
        biome,
        gw: SGW,
        gh: SGH,
        sea: SSEA,
        map_width_km: SMAPKM,
    }
}

// ------------------------------------------------------------------ helpers

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

/// Everything compared here is `f64` arithmetic over inputs proved
/// bit-identical by the hashes above, in the reference's own accumulation
/// order, so the bar is `1e-12` relative rather than a fixture tolerance.
#[track_caller]
fn near(got: f64, want: f64, what: &str) {
    let tol = 1e-12 * want.abs().max(1.0);
    assert!((got - want).abs() <= tol, "{what}: got {got}, want {want} (tol {tol})");
}

/// Run every place of one case and compare all sixteen captured values.
fn check(c: &Case, expect: &[Expect], label: &str) {
    let w = PlaceWorld {
        res: &c.res,
        field: &c.field,
        biome: &c.biome,
        rain: &c.rain,
        gw: c.gw,
        gh: c.gh,
        sea: c.sea,
        map_width_km: c.map_width_km,
    };
    // Shape: the harness must not have been handed an empty world.
    let land = c.field.iter().filter(|&&h| (h as f64) >= c.sea).count();
    assert!(land > 0, "{label}: no land cells -- fixture is silently empty");
    assert_eq!(c.temp.len(), c.gw * c.gh, "{label}: temp length");
    assert!(!expect.is_empty(), "{label}: no places");

    for (i, e) in expect.iter().enumerate() {
        let what = |f: &str| format!("{label} place {i} ({},{}) {f}", e.x, e.y);

        // The disc bound itself, not just its sum.
        let rad = cartalith_civ::civ_catchment_radius_cells(
            cartalith_civ::civ_catchment_km2(e.kind),
            c.map_width_km,
            c.gw,
        );
        assert_eq!(rad, e.radius, "{}", what("catchment radius"));

        let sm = civ_place_smelting(&w, e.x, e.y, e.kind);
        near(sm.iron_kg_yr, e.iron_kg_yr, &what("iron_kg_yr"));
        near(sm.charcoal_kg_yr, e.charcoal_kg_yr, &what("charcoal_kg_yr"));
        near(sm.ore_kg_yr, e.ore_kg_yr, &what("ore_kg_yr"));
        near(sm.woodland_ha, e.woodland_ha, &what("woodland_ha"));
        near(sm.coppice_ha_needed, e.coppice_ha_needed, &what("coppice_ha_needed"));
        assert_eq!(sm.limited_by, e.limited_by, "{}", what("limited_by"));
        assert_eq!(sm.fuel_poor, e.fuel_poor, "{}", what("fuel_poor"));
        assert_eq!(sm.ore_rich, e.ore_rich, "{}", what("ore_rich"));

        // `_civSaltAccess`'s deposit window is `_civPlaceResourceContext`'s
        // OWN default radius, not the catchment radius above -- asserted
        // separately so a regression that "tidied" the two into one is
        // caught here rather than only through a flipped verdict.
        let radius = (3.0f64).max((c.gw as f64 / 128.0).round()) as usize;
        let mean = cartalith_civ::civ_place_resource_context(
            &c.res, &c.field, c.gw, c.gh, c.sea, e.x as i64, e.y as i64, radius, false,
        );
        near(mean["salt"], e.salt_mean, &what("resource-context salt mean"));

        let sa = civ_salt_access(&w, e.x, e.y, e.nav);
        assert_eq!(sa.has, e.has, "{}", what("salt has"));
        assert_eq!(sa.source, e.source, "{}", what("salt source"));
    }
}

// -------------------------------------------------------------------- tests

#[test]
fn smelting_and_salt_case_0_region_no_wrap() {
    let c = build_real(64, 48, 24601, false);
    assert_eq!(fnv_f32(&c.field), "4d5ea30082db2da3", "field hash: not the world the harness saw");
    assert_eq!(fnv_bytes(&c.biome), "d980d83eb13b114c", "biome hash: the reference's own buildBiomeRaster over the injected fields disagrees");
    assert_eq!(fnv_f32(&c.rain), "bbf800d467046331", "rain hash");
    assert_eq!(fnv_f32(&c.res.iron), "970b5b6cd9a9add9", "iron-potential hash");
    assert_eq!(fnv_f32(&c.res.timber), "60b3746558e5690", "timber-potential hash");
    assert_eq!(fnv_f32(&c.res.salt), "87d9fbce40473eb8", "salt-potential hash");
    assert_eq!(c.field.iter().filter(|&&h| (h as f64) >= c.sea).count(), 2273, "land-cell count");
    check(&c, CASE0_EXPECT, "case0");
}

#[test]
fn smelting_and_salt_case_1_world_wrap() {
    let c = build_real(48, 36, 314159, true);
    assert_eq!(fnv_f32(&c.field), "ff792a79c88c72a3", "field hash: not the world the harness saw");
    assert_eq!(fnv_bytes(&c.biome), "e8a30c1895843612", "biome hash");
    assert_eq!(fnv_f32(&c.rain), "d299fd202af296d7", "rain hash");
    assert_eq!(fnv_f32(&c.res.iron), "4a61d7256e5e3589", "iron-potential hash");
    assert_eq!(fnv_f32(&c.res.timber), "c11596212bf5f6e5", "timber-potential hash");
    assert_eq!(fnv_f32(&c.res.salt), "297759ae6e9ff46b", "salt-potential hash");
    assert_eq!(c.field.iter().filter(|&&h| (h as f64) >= c.sea).count(), 1118, "land-cell count");
    check(&c, CASE1_EXPECT, "case1");

    // The two branches a real world reached on its own, named so a fixture
    // change that quietly loses them fails here rather than silently.
    assert_eq!(CASE1_EXPECT[5].source, "salt lake", "case1 place 5 is the real-terrain salt lake");
    assert_eq!(c.biome[CASE1_EXPECT[5].y * 48 + CASE1_EXPECT[5].x], cartalith_civ::BIOME_LAKE);
    assert!(f64::from(c.rain[CASE1_EXPECT[5].y * 48 + CASE1_EXPECT[5].x]) < 0.30);
    assert!(CASE1_EXPECT[6].ore_rich, "case1 place 6 is the real-terrain ore_rich case");
}

#[test]
fn smelting_and_salt_case_2_synthetic_wide_catchments() {
    let c = build_synth();
    assert_eq!(fnv_f32(&c.field), "e40faf723eec8a15", "synthetic field hash: the two sides built different worlds");
    assert_eq!(fnv_bytes(&c.biome), "7a92b1a21b5a3bd5", "synthetic biome hash");
    assert_eq!(fnv_f32(&c.rain), "da1083a4b0044b15", "synthetic rain hash");
    assert_eq!(fnv_f32(&c.res.iron), "858eb46e8eb0445", "synthetic iron hash");
    assert_eq!(fnv_f32(&c.res.timber), "e52270fef0d608e5", "synthetic timber hash");
    assert_eq!(fnv_f32(&c.res.salt), "31d87f70ed765405", "synthetic salt hash");
    assert_eq!(c.field.iter().filter(|&&h| (h as f64) >= c.sea).count(), 352, "land-cell count");
    check(&c, CASE2_EXPECT, "case2");

    // The discriminating pair: same `lake` biome, rain 0.20 vs 0.55.
    let (a, b) = (&CASE2_EXPECT[3], &CASE2_EXPECT[4]);
    assert_eq!(c.biome[a.y * SGW + a.x], cartalith_civ::BIOME_LAKE);
    assert_eq!(c.biome[b.y * SGW + b.x], cartalith_civ::BIOME_LAKE);
    assert_eq!((a.source, b.source), ("salt lake", "none"), "only the arid lake is a salt lake");
}

/// Every branch this fixture set is meant to reach, asserted as a property
/// of the captured data rather than left to inspection -- the "silently
/// empty golden output" trap this port has hit four times.
#[test]
fn fixtures_reach_every_branch() {
    let all: Vec<&Expect> =
        CASE0_EXPECT.iter().chain(CASE1_EXPECT).chain(CASE2_EXPECT).collect();
    for s in ["none", "sea salt", "salt deposit", "salt lake"] {
        assert!(all.iter().any(|e| e.source == s), "no place reaches salt source {s:?}");
    }
    for l in ["ore", "fuel"] {
        assert!(all.iter().any(|e| e.limited_by == l), "no place is limited by {l}");
    }
    assert!(all.iter().any(|e| e.fuel_poor), "no fuel_poor place (the Elba case)");
    assert!(all.iter().any(|e| !e.fuel_poor), "every place is fuel_poor");
    assert!(all.iter().any(|e| e.ore_rich), "no ore_rich place (the charcoal-exporter case)");
    assert!(all.iter().any(|e| !e.ore_rich), "every place is ore_rich");
    assert!(all.iter().any(|e| e.iron_kg_yr > 0.0), "no place actually smelts anything");
    assert!(all.iter().any(|e| e.coppice_ha_needed > 0.0), "coppice_ha_needed is zero everywhere");
    assert_eq!(all.iter().map(|e| e.radius).min(), Some(1), "no 1-cell catchment");
    assert_eq!(all.iter().map(|e| e.radius).max(), Some(14), "no wide catchment: the disc is untested");

    // The 0.25 deposit threshold is straddled from both sides by real
    // captured means, not only by the hand-written unit tests.
    assert!(all.iter().any(|e| e.salt_mean > 0.0 && e.salt_mean < 0.25 && e.source != "salt deposit"));
    assert!(all.iter().any(|e| e.salt_mean > 0.25 && e.source == "salt deposit"));
    for k in [NavKind::None, NavKind::River, NavKind::Sea] {
        assert!(all.iter().any(|e| e.nav == k), "no place with nav {k:?}");
    }
}
