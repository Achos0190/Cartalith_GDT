//! Ruling CD batch 3 (owner, 2026-10-11, item 2 and addendum item 5): slope in
//! planned land speed, and Track's own planner row. Unit tests on hand-built
//! fixtures for `jp_derive_stages`' per-stage slope (`JpSlope`) and
//! `jp_calc_land_ex`'s `slope` term (`jp_slope_mod`).
//!
//! What the fixture is. A 30 x 12 all-land world (cart biome 5, cart terrain 0
//! = Open Plains) whose height rises linearly west to east by `STEP` per cell
//! (or is flat), with sea level 0.40, a 8000 m peak and 2 km cells. A route
//! along a row then climbs `STEP x 8000 / 0.60` m per 2 km cell, an
//! along-track grade of `STEP x 8000 / 0.60 / 2000` = 0.1 (to f32 rounding).
//! The route runs along row 6, one point per cell, x = 3..=26.
//!
//! Since 2026-10-12 the grade is the route's own along-track grade, not the
//! router's gradient magnitude (`cartalith_civ::JpSlope`'s doc says why); the
//! two contouring tests at the end are the cases where they differ.
//!
//! Every expectation is a literal or an independent quantity computed here from
//! Tobler's (1993) exponent 3.5 (`exp(-0.35)` off a way, `exp(-0.175)` on one),
//! never the constant under test read back; each `Protects:` line names the
//! behaviour guarded, and each core test was checked to fail under a mutant of
//! the code it guards (see the batch report's mutation list).

use std::collections::HashMap;

use cartalith_civ::{JpLegCalc, JpPlan, JpRoadCell, JpWorld};

const W: usize = 30;
const H: usize = 12;
const ROW: usize = 6;
const SEA: f64 = 0.40;
const PEAK_M: f64 = 8000.0;
const CELL_KM: f64 = 2.0;
/// Height step per cell for a grade of 0.1: `0.1 x 0.60 x 2000 / 8000`.
const STEP: f32 = 0.015;

/// `exp(-3.5 x 0.1)`: Tobler's uphill branch, normalised to flat ground, at a
/// 10 % grade. Written out, not read from the crate.
const OFF_WAY_10PCT: f64 = 0.704_688_089_718_713_4;
/// `exp(-3.5 x 0.1 / 2)`: the same grade on a way, which carries half the
/// exponent (the batch's labelled judgement).
const ON_WAY_10PCT: f64 = 0.839_457_020_769_207_4;

/// The fixture's owned rasters.
struct Sl {
    field: Vec<f32>,
    biome: Vec<u8>,
    terrain: Vec<u8>,
    temp: Vec<f32>,
    rain: Vec<f32>,
    wb: Vec<u8>,
}

/// Builds the fixture: height `0.45 + step x x` everywhere.
fn ground(step: f32) -> Sl {
    let n = W * H;
    let field: Vec<f32> = (0..n).map(|i| 0.45 + step * (i % W) as f32).collect();
    Sl {
        field,
        biome: vec![5; n],
        terrain: vec![0; n],
        temp: vec![15.0; n],
        rain: vec![0.5; n],
        wb: vec![0; n],
    }
}

impl Sl {
    /// The planner's world over this fixture. `slope` false turns slope
    /// measurement off: the "no slope data" marker, not a flat one.
    fn world<'a>(&'a self, road: &'a HashMap<(i64, i64), JpRoadCell>, slope: bool) -> JpWorld<'a> {
        JpWorld {
            gw: W,
            gh: H,
            world: false,
            map_width_km: W as f64 * CELL_KM,
            sea_level: SEA,
            peak_m: PEAK_M,
            field: &self.field,
            cart_biome: &self.biome,
            cart_terrain: &self.terrain,
            temp: &self.temp,
            rain: &self.rain,
            flow_field: None,
            flow_thresh: 10.0,
            water_bodies: Some(&self.wb),
            territory: None,
            places: &[],
            road_cells: road,
            ocean_field: None,
            wind_field: None,
            river_net: None,
            measure_slope: slope,
        }
    }
}

fn east() -> Vec<(f64, f64)> {
    (3..=26).map(|x| (x as f64, ROW as f64)).collect()
}

fn west() -> Vec<(f64, f64)> {
    east().into_iter().rev().collect()
}

/// A road along the route's row, stamped by the planner's own table.
fn road_along_row() -> HashMap<(i64, i64), JpRoadCell> {
    let way = cartalith_civ::Way {
        tid: 0,
        pts: (0..W).map(|x| (x as f64, ROW as f64)).collect(),
        brks: Vec::new(),
        km: 0.0,
        name: String::new(),
        way_type: cartalith_civ::WayType::Road,
        a_idx: 0,
        b_idx: 1,
        hidden: false,
    };
    let cells = cartalith_civ::jp_road_cells(&[way], &[], &[], W);
    assert!(cells.contains_key(&(10, ROW as i64)), "premise: the road stamps the route's row");
    cells
}

/// One planned land journey: its single stage's slope, the `slope` trace
/// factor (`None` when the trace has no such term), and the travel days.
struct Planned {
    grade: Option<f64>,
    on_way: Option<bool>,
    factor: Option<f64>,
    days: f64,
}

fn plan_on(s: &Sl, road: &HashMap<(i64, i64), JpRoadCell>, slope: bool, pts: &[(f64, f64)]) -> Planned {
    let w = s.world(road, slope);
    let jp = cartalith_civ::jp_plan(&w, pts, &JpPlan::default(), &HashMap::new(), &|_, _| 1.0).expect("the fixture route plans");
    assert_eq!(jp.stages.len(), 1, "premise: one uniform land stage, got {:?}", jp.stages);
    assert_eq!(jp.stages[0].cat, "land");
    let st = &jp.stages[0];
    let Ok(JpLegCalc::Land(l)) = &jp.results[0].calc else { panic!("premise: the land leg computes: {:?}", jp.results[0].calc) };
    Planned {
        grade: st.slope.map(|s| s.grade),
        on_way: st.slope.map(|s| s.on_way),
        factor: l.trace.iter().find(|t| t.key == "slope").map(|t| t.factor),
        days: jp.travel_days,
    }
}

/// Protects: an uphill route is slower than the same route on flat ground by
/// Tobler's normalised factor, `exp(-3.5 x grade)`, measured along the
/// route; the grade itself is the fixture's 0.1.
#[test]
fn an_uphill_route_is_slower_than_a_flat_one_by_toblers_factor() {
    let none = HashMap::new();
    let hill = plan_on(&ground(STEP), &none, true, &east());
    let flat = plan_on(&ground(0.0), &none, true, &east());
    let g = hill.grade.expect("slope measurement gives the stage a grade");
    assert!((g - 0.1).abs() < 1e-6, "grade {g}");
    let f = hill.factor.expect("the trace names the slope term");
    assert!((f - OFF_WAY_10PCT).abs() < 1e-6, "factor {f}");
    // The day ratio is the factor up to the column term, which depends on the
    // day's distance (`jp_column_factor`) and is ~1e-4 for four walkers.
    let ratio = flat.days / hill.days;
    assert!((ratio - OFF_WAY_10PCT).abs() < 1e-3, "days ratio {ratio} ({} vs {})", flat.days, hill.days);
    assert!(hill.days > flat.days * 1.4, "uphill {} flat {}", hill.days, flat.days);
}

/// Protects: the slope term is symmetric -- downhill costs what uphill does --
/// because the along-track grade is unsigned (`|dh|`, the documented
/// simplification in `jp_slope_mod`; Tobler's own curve is asymmetric).
#[test]
fn downhill_and_uphill_cost_the_same() {
    let none = HashMap::new();
    let s = ground(STEP);
    let up = plan_on(&s, &none, true, &east());
    let down = plan_on(&s, &none, true, &west());
    // Equal up to rounding: the two directions sum the same height
    // differences in the opposite order.
    let (fu, fd) = (up.factor.expect("term present"), down.factor.expect("term present"));
    assert!((fu - fd).abs() < 1e-6, "up {fu} down {fd}");
    assert!((up.days - down.days).abs() < 1e-5 * up.days, "up {} down {}", up.days, down.days);
    assert!(down.factor.expect("term present") < 0.75, "downhill is penalised too");
}

/// Protects: a way halves the slope exponent -- it reduces, never removes, the
/// penalty -- and the stage records that it rode a way.
#[test]
fn slope_on_a_road_costs_less_than_off_road_but_still_costs() {
    let road = road_along_row();
    let on = plan_on(&ground(STEP), &road, true, &east());
    assert_eq!(on.on_way, Some(true));
    let f = on.factor.expect("the trace names the slope term");
    assert!((f - ON_WAY_10PCT).abs() < 1e-6, "on-way factor {f}");
    let none = HashMap::new();
    let off = plan_on(&ground(STEP), &none, true, &east());
    assert_eq!(off.on_way, Some(false));
    assert!(f > off.factor.expect("term present"), "a road must soften the slope");
    assert!(f < 1.0, "a road must not remove the slope");
    let flat_road = plan_on(&ground(0.0), &road, true, &east());
    assert!(on.days > flat_road.days, "a road up a hill is slower than a flat road");
}

/// Protects: flat ground with slope measured plans to the bit what it planned
/// before slope existed (no slope data), so no flat-terrain plan moves; the
/// trace shows the term at exactly 1.
#[test]
fn flat_ground_is_unchanged_to_the_bit() {
    let none = HashMap::new();
    let s = ground(0.0);
    let with = plan_on(&s, &none, true, &east());
    let without = plan_on(&s, &none, false, &east());
    assert_eq!(with.grade, Some(0.0));
    assert_eq!(with.factor, Some(1.0));
    assert_eq!(with.days.to_bits(), without.days.to_bits(), "{} vs {}", with.days, without.days);
}

/// Protects: no slope data is not zero slope. Without slope measurement the stage
/// carries no slope at all (`None`, not a grade of 0), the trace has no slope
/// term, and the plan is today's (reference) plan -- on a hill, where a
/// zero-grade reading would be wrong.
#[test]
fn no_slope_data_is_not_a_zero_slope_and_falls_back_to_the_reference() {
    let none = HashMap::new();
    let s = ground(STEP);
    let nodata = plan_on(&s, &none, false, &east());
    assert_eq!(nodata.grade, None);
    assert_eq!(nodata.factor, None);
    // Today's behaviour on this hill is the flat plan (the reference never read
    // slope into time).
    let flat = plan_on(&ground(0.0), &none, false, &east());
    assert_eq!(nodata.days.to_bits(), flat.days.to_bits());
    // A sea level at 1 leaves no land range to scale a height difference by:
    // also no data, never a zero (or infinite) grade.
    let w = JpWorld { sea_level: 1.0, ..s.world(&none, true) };
    let st = cartalith_civ::jp_derive_stages(&w, &east(), &JpPlan::default());
    assert!(st.iter().all(|x| x.slope.is_none()), "no land range must read as no slope data: {st:?}");
    // Above 1 the scale is finite but negative: still no land range.
    let w = JpWorld { sea_level: 1.5, ..s.world(&none, true) };
    let st = cartalith_civ::jp_derive_stages(&w, &east(), &JpPlan::default());
    assert!(st.iter().all(|x| x.slope.is_none()), "a negative scale must read as no slope data: {st:?}");
}

/// Protects: only land stages carry a slope; `jp_slope_mod` is exactly 1 at
/// zero grade and falls monotonically with grade.
#[test]
fn the_slope_modifier_is_one_on_the_flat_and_falls_with_grade() {
    use cartalith_civ::{jp_slope_mod, JpSlope};
    assert_eq!(jp_slope_mod(&JpSlope { grade: 0.0, on_way: false }).to_bits(), 1.0f64.to_bits());
    assert_eq!(jp_slope_mod(&JpSlope { grade: 0.0, on_way: true }).to_bits(), 1.0f64.to_bits());
    let a = jp_slope_mod(&JpSlope { grade: 0.05, on_way: false });
    let b = jp_slope_mod(&JpSlope { grade: 0.20, on_way: false });
    assert!((a - 0.839_457_020_769_207_4).abs() < 1e-12, "5 % grade {a}");
    assert!((b - 0.496_585_303_791_409_7).abs() < 1e-12, "20 % grade {b}");
}

/// Protects: Track's own planner row (Ruling CD addendum item 5): a track reads
/// `Dirt Track / None / Wild`, slower than a road (`Standard`) and faster than
/// an ancient way (`Deteriorated`), in the planner's own days.
#[test]
fn a_track_is_planned_slower_than_a_road_and_faster_than_an_ancient_way() {
    let s = ground(0.0);
    let days = |ty: cartalith_civ::WayType| {
        let way = cartalith_civ::Way {
            tid: 0,
            pts: (0..W).map(|x| (x as f64, ROW as f64)).collect(),
            brks: Vec::new(),
            km: 0.0,
            name: String::new(),
            way_type: ty,
            a_idx: 0,
            b_idx: 1,
            hidden: false,
        };
        let road = cartalith_civ::jp_road_cells(&[way], &[], &[], W);
        let w = s.world(&road, true);
        let jp = cartalith_civ::jp_plan(&w, &east(), &JpPlan::default(), &HashMap::new(), &|_, _| 1.0).expect("plans");
        assert_eq!(jp.stages.len(), 1);
        (jp.stages[0].route_cond.clone(), jp.travel_days)
    };
    let (rc, rd) = days(cartalith_civ::WayType::Road);
    let (tc, td) = days(cartalith_civ::WayType::Track);
    let (ac, ad) = days(cartalith_civ::WayType::Ancient);
    assert_eq!((rc.as_str(), tc.as_str(), ac.as_str()), ("Standard", "None / Wild", "Deteriorated"));
    assert!(rd < td && td < ad, "road {rd} track {td} ancient {ad}");
    // Flat ground, same everything else: days scale as 1 / JP_ROUTE.land, so
    // track/road = 1.00/0.85 up to the column term.
    assert!((td / rd - 1.0 / 0.85).abs() < 1e-3, "track/road {}", td / rd);
}

/// Protects: merging chunks keeps the slope mean. A ramp whose cart terrain
/// alternates column by column (Open Plains / Hills) cuts 24 one-step runs,
/// which the 14-stage cap merges; every merged land stage must still read the
/// ramp's own 0.1 grade, because `jp_merge_chunk` carries both the absorbed
/// chunk's `km x grade` sum and its km.
#[test]
fn merged_stages_keep_the_length_weighted_grade() {
    let mut s = ground(STEP);
    for i in 0..W * H {
        // CART_TERRAINS index 6 is "Hills"; 0 reads as "Open Plains".
        s.terrain[i] = if (i % W) % 2 == 1 { 6 } else { 0 };
    }
    let none = HashMap::new();
    let st = cartalith_civ::jp_derive_stages(&s.world(&none, true), &east(), &JpPlan::default());
    assert_eq!(st.len(), 14, "premise: the 14-stage cap merged the alternating runs");
    for (i, x) in st.iter().enumerate() {
        let g = x.slope.expect("land stage with slope measured").grade;
        assert!((g - 0.1).abs() < 1e-6, "stage {i} grade {g}");
    }
}

/// Protects: only land stages carry a slope. A one-step land sliver inside a
/// sea crossing is absorbed into the sea stage, bringing its measured land
/// step with it; the sea stage must still report no slope (the water
/// calculator never reads one, and a sea "grade" would be a fabricated value).
#[test]
fn a_water_stage_never_carries_a_slope() {
    let mut s = ground(STEP);
    for i in 0..W * H {
        let land = i % W == 15;
        s.biome[i] = if land { 5 } else { 15 };
        s.wb[i] = if land { 0 } else { 1 };
    }
    // 0.6-cell steps: exactly one point (x = 15.0) rounds onto the land
    // column, so the land run is a 1.2 km sliver under the 2 km floor.
    let pts: Vec<(f64, f64)> = (0..=38).map(|k| (3.0 + 0.6 * k as f64, ROW as f64)).collect();
    let none = HashMap::new();
    let plan = JpPlan { transport: "Sea Faring".into(), vessel: "Cog".into(), ..JpPlan::default() };
    let st = cartalith_civ::jp_derive_stages(&s.world(&none, true), &pts, &plan);
    assert!(!st.is_empty() && st.iter().all(|x| x.cat == "sea"), "premise: the sliver was absorbed: {st:?}");
    assert!(st.iter().all(|x| x.slope.is_none()), "a sea stage carries no slope: {st:?}");
}

/// Protects: a route that contours -- runs across the fall line, level -- is
/// charged no slope, though the ground under it is the fixture's 10 % ramp.
/// The router's gradient magnitude read 0.1 at every one of these cells (the
/// overstatement this grade replaced, `OUTSTANDING_WORK.md` "Slope in planned
/// speed"); the planned days equal the slope-free plan's to the bit.
#[test]
fn a_contouring_route_on_a_hillside_is_charged_no_slope() {
    let none = HashMap::new();
    let s = ground(STEP);
    // Column 15, rows 1..=10: north to south, at one height.
    let across: Vec<(f64, f64)> = (1..=10).map(|y| (15.0, y as f64)).collect();
    let on = plan_on(&s, &none, true, &across);
    let off = plan_on(&s, &none, false, &across);
    assert_eq!(on.grade, Some(0.0), "a level route on a hillside has no along-track grade");
    assert_eq!(on.factor, Some(1.0));
    assert_eq!(on.days.to_bits(), off.days.to_bits(), "{} vs {}", on.days, off.days);
    // Control: the same hillside walked up its fall line is charged.
    assert!(plan_on(&s, &none, true, &east()).factor.expect("term present") < 0.75);
}

/// Protects: a level coastal route is not charged for the sea floor beside
/// it. Land at one height with a deep sea row next to the route: a gradient
/// would difference the route's cells against the sea floor; the along-track
/// grade reads only the route's own points and is 0.
#[test]
fn a_level_coastal_route_is_not_charged_for_the_sea_floor() {
    let none = HashMap::new();
    let mut s = ground(0.0);
    for x in 0..W {
        let i = (ROW + 1) * W + x;
        s.field[i] = 0.05; // far below sea level 0.40
        s.biome[i] = 15;
        s.wb[i] = 1;
    }
    let p = plan_on(&s, &none, true, &east());
    assert_eq!(p.grade, Some(0.0), "the route's own points are level");
    assert_eq!(p.factor, Some(1.0));
}
