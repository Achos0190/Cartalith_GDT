//! Ruling CD batch 1 (owner, 2026-10-11): unit tests on hand-built fixtures for
//! the four journey-planner conformance fixes `PLANNER_AUDIT_2026-10-11.md`
//! found and Ruling CD items 3 and 4 ordered:
//!
//! - (A) a sea stage's condition is scored for the vessel that stage sails;
//! - (B) a hidden generated way is not credited through its raw edge;
//! - (C) the Route/Way tools' router sees the same edges the planner credits;
//! - (D) way types differ in the router, from the one `way_class` table.
//!
//! Every expectation is a literal measured once on the fixture and pinned; each
//! test's `Protects:` line names the behaviour it guards, and each was checked
//! to fail with its fix reverted (see the batch report's mutation list).

use std::collections::HashMap;

use cartalith_civ::tools::{self, RouteContext, RouteMode, WayRef};
use cartalith_civ::way_class::WayClass;
use cartalith_civ::{JpCoarseField, JpLegCalc, JpPlan, JpRoadCell, JpStageOverride, JpWorld, RoadEdge, Way, WayType};

// ---------------------------------------------------------------- (A) sea ----

const SW: usize = 40;
const SH: usize = 20;

/// An all-ocean world, so every route point is more than 8 cells from land and
/// derives as `Open Sea` -- water an oared Keelboat cannot enter, so the plan
/// auto-substitutes a sailing hull for it. No current; a uniform wind blowing
/// due west (`u = -3`), i.e. dead against an eastbound route.
struct Sea {
    field: Vec<f32>,
    biome: Vec<u8>,
    terrain: Vec<u8>,
    temp: Vec<f32>,
    rain: Vec<f32>,
    wb: Vec<u8>,
    ocean: JpCoarseField,
    wind: JpCoarseField,
}

fn sea() -> Sea {
    let n = SW * SH;
    Sea {
        field: vec![0.2; n],
        biome: vec![15; n],
        terrain: vec![0; n],
        temp: vec![15.0; n],
        rain: vec![0.5; n],
        wb: vec![1; n],
        ocean: JpCoarseField { ww: 4, wh: 4, u: vec![0.0; 16], v: vec![0.0; 16], max_speed: 1.0 },
        wind: JpCoarseField { ww: 4, wh: 4, u: vec![-3.0; 16], v: vec![0.0; 16], max_speed: 3.0 },
    }
}

impl Sea {
    fn world<'a>(&'a self, road: &'a HashMap<(i64, i64), JpRoadCell>) -> JpWorld<'a> {
        JpWorld {
            gw: SW,
            gh: SH,
            world: false,
            map_width_km: SW as f64 * 12.0,
            sea_level: 0.42,
            peak_m: 4000.0,
            field: &self.field,
            cart_biome: &self.biome,
            cart_terrain: &self.terrain,
            temp: &self.temp,
            rain: &self.rain,
            flow_field: None,
            flow_thresh: 0.0,
            water_bodies: Some(&self.wb),
            territory: None,
            places: &[],
            road_cells: road,
            ocean_field: Some(&self.ocean),
            wind_field: Some(&self.wind),
            river_net: None,
            slope_field: None, // Ruling CD item 2: no slope data, so these plans keep the reference's (slope-free) land time
        }
    }
}

/// An eastbound route along the middle row, one point per cell.
fn eastbound() -> Vec<(f64, f64)> {
    (5..=35).map(|x| (x as f64, 10.0)).collect()
}

fn sea_plan(vessel: &str) -> JpPlan {
    JpPlan { transport: "Sea Faring".into(), vessel: vessel.into(), ..JpPlan::default() }
}

/// `(stage condition, derived condition, vessel used, water days)` of the one
/// stage the fixture derives.
fn one_sea_stage(plan: &JpPlan) -> (String, Option<String>, String, f64) {
    let s = sea();
    let road = HashMap::new();
    let w = s.world(&road);
    let jp = cartalith_civ::jp_plan(&w, &eastbound(), plan, &HashMap::new(), &|_, _| 1.0).expect("the sea route plans");
    assert_eq!(jp.stages.len(), 1, "premise: one stage");
    assert_eq!((jp.stages[0].cat.as_str(), jp.stages[0].terrain.as_str()), ("sea", "Open Sea"), "premise: open sea");
    let days = match &jp.results[0].calc {
        Ok(JpLegCalc::Water(wc)) => wc.days,
        other => panic!("premise: a water leg that sails, got {other:?}"),
    };
    (jp.stages[0].route_cond.clone(), jp.stages[0].derived_cond.clone(), jp.results[0].eff.vessel.clone(), days)
}

/// Protects: (A) the audit's Keelboat case. The shared vessel is oared, the open-sea
/// stage is auto-substituted to a Dhow, and the stage is now scored for the Dhow's
/// sail rig (a dead headwind), not as an oared hull ("Neutral"), with the slower
/// days that follow; the reported stage carries the condition it was costed on.
#[test]
fn an_auto_substituted_hull_is_scored_for_its_own_rig() {
    let (cond, derived, vessel, days) = one_sea_stage(&sea_plan("Keelboat"));
    assert_eq!(vessel, "Dhow", "premise: the Keelboat cannot enter open sea and a Dhow is substituted");
    assert_eq!(cond, "Strong Headwind");
    assert_eq!(derived.as_deref(), Some("Strong Headwind"));
    assert_eq!(days, 5.331_911_490_269_261_5);
    // The independent check: the same stage planned with a Dhow as the shared
    // vessel (the path the reference already got right) scores the same.
    let (cond_d, _, vessel_d, days_d) = one_sea_stage(&sea_plan("Dhow"));
    assert_eq!((cond_d.as_str(), vessel_d.as_str(), days_d), ("Strong Headwind", "Dhow", days));
}

/// Protects: (A) the per-stage vessel override. A Keelboat plan whose one stage is
/// explicitly sailed by a Cog is scored for the Cog, equal to a plan whose shared
/// vessel is the Cog.
#[test]
fn a_per_stage_vessel_override_is_scored_for_that_vessel() {
    let mut plan = sea_plan("Keelboat");
    plan.stage_overrides.insert(0, JpStageOverride { vessel: Some("Cog".into()), ..JpStageOverride::default() });
    let (cond, derived, vessel, days) = one_sea_stage(&plan);
    assert_eq!(vessel, "Cog");
    assert_eq!((cond.as_str(), derived.as_deref()), ("Strong Headwind", Some("Strong Headwind")));
    let shared = one_sea_stage(&sea_plan("Cog"));
    assert_eq!((cond, vessel, days), (shared.0, shared.2, shared.3));
}

/// Protects: (A) never second-guesses a condition the user chose: a plan-wide
/// manual "Neutral" stays Neutral on the substituted Dhow, and a per-stage manual
/// condition stays too.
#[test]
fn a_manual_sea_condition_is_never_re_derived() {
    let mut plan = sea_plan("Keelboat");
    plan.route_cond = Some("Neutral".into());
    let (cond, _, vessel, _) = one_sea_stage(&plan);
    assert_eq!((cond.as_str(), vessel.as_str()), ("Neutral", "Dhow"));

    let mut plan = sea_plan("Keelboat");
    plan.stage_overrides.insert(
        0,
        JpStageOverride { vessel: Some("Cog".into()), route_cond: Some("Favorable Wind".into()), ..JpStageOverride::default() },
    );
    let (cond, _, vessel, _) = one_sea_stage(&plan);
    assert_eq!((cond.as_str(), vessel.as_str()), ("Favorable Wind", "Cog"));
}

// ------------------------------------------------- (B)+(C)+(D) ways ----------

const LW: usize = 40;
const LH: usize = 20;

/// A flat land world: every cell costs the same, so only way discounts bend a
/// route.
struct Land {
    field: Vec<f32>,
    wb: Vec<u8>,
}

fn land() -> Land {
    Land { field: vec![0.6; LW * LH], wb: vec![0; LW * LH] }
}

fn ctx<'a>(l: &'a Land, ways: &'a [WayRef<'a>]) -> RouteContext<'a> {
    RouteContext {
        field: &l.field,
        water_bodies: &l.wb,
        biome: None,
        river_order: None,
        places: &[],
        ways,
        gw: LW,
        gh: LH,
        sea: 0.42,
        world: false,
        map_width_km: LW as f64 * 12.0,
        corridors: None,
        flow: None,
        flow_thresh: 0.0,
    }
}

/// The cell indices of row `y` from `x0` to `x1` inclusive, as a raw
/// routing-grid path (the grid is 1:1 at this size).
fn row(y: usize, x0: usize, x1: usize) -> Vec<usize> {
    (x0..=x1).map(|x| y * LW + x).collect()
}

/// A generated way joining settlements `a`/`b`; only `hidden` and the pair are
/// read by the edge filter.
fn gen_way(a: usize, b: usize, hidden: bool, pts: Vec<(f64, f64)>) -> Way {
    Way { tid: 0, pts, brks: Vec::new(), km: 0.0, name: String::new(), way_type: WayType::Road, a_idx: a, b_idx: b, hidden }
}

/// Mean row of a route's points: 8.0 is the straight line, lower means it was
/// pulled up toward a way on row 3.
fn mean_y(pts: &[(f64, f64)]) -> f64 {
    assert!(!pts.is_empty(), "a route must have points");
    pts.iter().map(|p| p.1).sum::<f64>() / pts.len() as f64
}

/// Protects: (B) a hidden generated way is not credited through its raw edge, in
/// either endpoint order, while a visible way's edge still is (the audit's
/// `hidden_generated_way_is_still_credited_through_its_edge` read 90 cells before).
#[test]
fn a_hidden_way_is_not_credited_through_its_edge() {
    let edge = RoadEdge { a: 0, b: 1, path: row(10, 2, 29) };
    let shown = gen_way(0, 1, false, Vec::new());
    let hidden = gen_way(1, 0, true, Vec::new());
    let credited = cartalith_civ::jp_road_cells(std::slice::from_ref(&shown), &[], std::slice::from_ref(&edge), LW);
    assert_eq!(credited.len(), 90, "a visible way's edge: 28 cells dilated to 30 x 3");
    assert_eq!(credited[&(15, 10)], JpRoadCell { terrain: "Dirt Track", cond: "Standard", pri: 1 });
    let none = cartalith_civ::jp_road_cells(std::slice::from_ref(&hidden), &[], std::slice::from_ref(&edge), LW);
    assert_eq!(none.len(), 0, "the hidden way's edge, endpoints given in the other order");
    assert_eq!(tools::civ_credited_road_edges(std::slice::from_ref(&hidden), std::slice::from_ref(&edge)).len(), 0);
}

/// Protects: (C) and (B) together, in the router. A raw edge along row 3 with no
/// drawn way over it pulls a row-8 route onto it (the router now sees the edges the
/// planner credits); hiding the edge's way leaves the straight row-8 route, so the
/// router and the planner drop it together.
#[test]
fn the_router_rides_a_credited_edge_and_ignores_a_hidden_one() {
    let l = land();
    let edge = RoadEdge { a: 0, b: 1, path: row(3, 1, 38) };
    let route = |ways: &[Way]| {
        let credited = tools::civ_credited_road_edges(ways, std::slice::from_ref(&edge));
        let polys = tools::civ_road_edge_polylines(&credited, LW);
        let refs = tools::civ_network_way_refs(ways, &[], &polys);
        let c = ctx(&l, &refs);
        let r = tools::civ_dijkstra_path(&c, 2.0, 8.0, 37.0, 8.0, RouteMode::Land);
        assert!(r.reachable, "premise: flat land routes");
        r.pts
    };
    // A way record for the edge's pair with no drawn geometry of its own, so
    // any pull toward row 3 can only come from the edge.
    let shown = [gen_way(0, 1, false, Vec::new())];
    let hidden = [gen_way(0, 1, true, Vec::new())];
    assert_eq!(mean_y(&route(&shown)), 4.117647058823529, "pulled onto the credited edge");
    assert_eq!(mean_y(&route(&hidden)), 8.0, "a hidden way's edge is not ridden");
}

/// Protects: (D) the router reads each way's class from the shared table. A plain
/// road five rows off a straight line beats an ancient way four rows off on the other
/// side (0.25 vs 0.333 per cell), where the type-blind x0.25 picked the nearer
/// ancient way.
#[test]
fn the_router_prefers_the_faster_way_class() {
    let l = land();
    let road_pts: Vec<(f64, f64)> = (1..=38).map(|x| (x as f64, 3.0)).collect();
    let ancient_pts: Vec<(f64, f64)> = (1..=38).map(|x| (x as f64, 12.0)).collect();
    let go = |ancient_class: WayClass| {
        let ways = [
            WayRef { pts: &road_pts, brks: &[], sea: false, hidden: false, class: WayClass::Road },
            WayRef { pts: &ancient_pts, brks: &[], sea: false, hidden: false, class: ancient_class },
        ];
        let c = ctx(&l, &ways);
        mean_y(&tools::civ_dijkstra_path(&c, 2.0, 8.0, 37.0, 8.0, RouteMode::Land).pts)
    };
    // Control: both plain roads (the old type-blind router) -- the nearer way wins.
    assert_eq!(go(WayClass::Road), 12.0, "type-blind: the nearer way (row 12)");
    // The table: the ancient way costs 0.333, so the farther plain road wins.
    assert_eq!(go(WayClass::Ancient), 4.117647058823529, "per class: the plain road (row 3)");
}

/// Protects: (D) the router's overlap rule matches `jp_road_cells`': where two ways
/// of equal priority share cells, the first-listed one owns them. A plain road on
/// row 3 listed before an ancient way drawn on the very same cells keeps the road's
/// 0.25, so the route still rides row 3 against an ancient way on row 12; if the
/// later ancient way took the shared cells (0.333 on both rows), the nearer row 12
/// would win instead. The planner's own table for the same overlap is checked too.
#[test]
fn equal_priority_overlap_keeps_the_first_writer_in_both_consumers() {
    let l = land();
    let row3: Vec<(f64, f64)> = (1..=38).map(|x| (x as f64, 3.0)).collect();
    let row12: Vec<(f64, f64)> = (1..=38).map(|x| (x as f64, 12.0)).collect();
    let r = |pts, class| WayRef { pts, brks: &[], sea: false, hidden: false, class };
    let ways = [r(&row3, WayClass::Road), r(&row3, WayClass::Ancient), r(&row12, WayClass::Ancient)];
    let c = ctx(&l, &ways);
    assert_eq!(mean_y(&tools::civ_dijkstra_path(&c, 2.0, 8.0, 37.0, 8.0, RouteMode::Land).pts), 4.117647058823529);

    let manual = |pts: &Vec<(f64, f64)>, t| tools::ManualWay {
        pts: pts.clone(),
        brks: Vec::new(),
        km: 0.0,
        sea: false,
        way_type: t,
        name: String::new(),
        hidden: false,
    };
    let cells = cartalith_civ::jp_road_cells(
        &[],
        &[manual(&row3, tools::ManualWayType::Road), manual(&row3, tools::ManualWayType::Ancient)],
        &[],
        LW,
    );
    assert_eq!(cells[&(10, 3)].cond, "Standard", "the planner keeps the first writer too");
}
