//! Ruling CD batch 3 (2026-10-11): slope in planned land time, and Track's own
//! planner row, measured on the planner audit's world. **Ignored**; prints,
//! pins nothing.
//!
//! What this file is for. `PLANNER_AUDIT_2026-10-11.md` Q2 found that slope
//! never reached planned time. Ruling CD item 2 put it there (`JpSlope`,
//! `jp_slope_mod`) and addendum item 5 gave Track its own `None / Wild` row.
//! This harness plans the audit's own routes on the audit's own world (seed
//! 24601, 256 x 192, engine defaults; see `planner_audit_journey.rs`) with and
//! without the router's slope field, so the before/after of item 2 is one run:
//! "before" is `JpWorld::slope_field = None` (no slope data, the reference's
//! time) and "after" is the field `WorldGen::jp_world_parts` builds. It also
//! prints the slope distribution over land and, per stage, the router's
//! gradient-magnitude grade beside the route's own along-track grade
//! (`(gain + loss) / km`), because the two differ wherever a route contours.
//!
//! Why it lives in `cartalith-godot/tests`: the same reason as
//! `planner_audit_journey.rs` (the only crate whose dependency set has
//! `cartalith-civ` and `cartalith-engine` together without a `Cargo.toml`
//! edit). `WorldGen` is a cdylib `GodotClass`, so the `JpWorld` is assembled
//! from the same public functions `jp_world_parts` calls.
//!
//! Run: `cargo test -p cartalith-godot --test planner_audit_slope -- --ignored --nocapture`.
//! Must never be un-ignored: it generates a world and prints tables, and its
//! numbers are observations, not contracts. Every assertion only guards that
//! the fixture reached the code under test.

use std::collections::HashMap;

use cartalith_civ::tools::{self, ManualWayType, RouteContext, RouteMode, WayRef};
use cartalith_civ::{JpPlan, JpWorld};
use cartalith_engine::{generate_terrain, WorldParams, WorldState};

const SEED: i32 = 24_601;
const GW: usize = 256;
const GH: usize = 192;

/// The audit's world plus the rasters the router and planner read, and the
/// router's slope field.
struct Fx {
    ws: WorldState,
    p: WorldParams,
    wb: Vec<u8>,
    cart_biome: Vec<u8>,
    cart_terrain: Vec<u8>,
    flow_thresh: f64,
    slope: Vec<f32>,
}

/// `planner_audit_journey.rs`' fixture, plus `build_raw_slope_field` exactly as
/// `jp_world_parts` builds it (not wrapped: the audit world is flat-mapped).
fn fixture() -> Fx {
    let p = WorldParams::defaults(GW, GH, SEED);
    let ws = generate_terrain(&p);
    let wb = cartalith_civ::build_water_bodies(&ws.field, GW, GH, ws.sea_level, false, Some(&ws.rainfall)).classification;
    let cart_biome = cartalith_civ::build_cart_biome(&ws.field, &wb, &ws.temperature, &ws.rainfall, GW, GH, false, ws.sea_level);
    let cart_terrain = cartalith_civ::build_cart_terrain(&ws.field, &wb, &ws.temperature, &ws.rainfall, GW, GH, false, ws.sea_level);
    let flow_thresh = cartalith_hydrology::river_flow_thresh(GW, GH, GW, p.map_width_km);
    let slope = cartalith_civ::build_raw_slope_field(&ws.field, GW, GH, false);
    Fx { ws, p, wb, cart_biome, cart_terrain, flow_thresh, slope }
}

impl Fx {
    /// The router's context, as `planner_audit_journey.rs` builds it.
    fn ctx<'a>(&'a self, ways: &'a [WayRef<'a>]) -> RouteContext<'a> {
        RouteContext {
            field: &self.ws.field,
            water_bodies: &self.wb,
            biome: None,
            river_order: None,
            places: &[],
            ways,
            gw: GW,
            gh: GH,
            sea: self.ws.sea_level,
            world: false,
            map_width_km: self.p.map_width_km,
            corridors: None,
            flow: Some(&self.ws.flow_discharge),
            flow_thresh: self.flow_thresh,
        }
    }

    /// The planner's world; `slope` false is the "before" (no slope data).
    fn jp<'a>(&'a self, road: &'a HashMap<(i64, i64), cartalith_civ::JpRoadCell>, slope: bool) -> JpWorld<'a> {
        JpWorld {
            gw: GW,
            gh: GH,
            world: false,
            map_width_km: self.p.map_width_km,
            sea_level: self.ws.sea_level,
            peak_m: self.p.peak_m,
            field: &self.ws.field,
            cart_biome: &self.cart_biome,
            cart_terrain: &self.cart_terrain,
            temp: &self.ws.temperature,
            rain: &self.ws.rainfall,
            flow_field: Some(&self.ws.flow_discharge),
            flow_thresh: self.flow_thresh,
            water_bodies: Some(&self.wb),
            territory: None,
            places: &[],
            road_cells: road,
            ocean_field: None,
            wind_field: None,
            river_net: None,
            slope_field: slope.then_some(&self.slope[..]),
        }
    }

    /// The land cell nearest a fraction of the map.
    fn nearest_land(&self, fx: f64, fy: f64) -> (f64, f64) {
        let (tx, ty) = (fx * GW as f64, fy * GH as f64);
        let mut best = None;
        let mut bd = f64::INFINITY;
        for y in 0..GH {
            for x in 0..GW {
                if self.wb[y * GW + x] != 0 {
                    continue;
                }
                let d = (x as f64 - tx).powi(2) + (y as f64 - ty).powi(2);
                if d < bd {
                    bd = d;
                    best = Some((x as f64, y as f64));
                }
            }
        }
        best.expect("the fixture world has land")
    }

    /// Normalised-height-per-cell to rise-over-run, as `JpWorld` converts it.
    fn to_grade(&self) -> f64 {
        self.p.peak_m / (1.0 - self.ws.sea_level) / (self.p.map_width_km / GW as f64 * 1000.0)
    }
}

/// The planned travel days (blocked land legs plan no total on this
/// settlement-less world, so `travel_days` is the comparable figure).
fn travel_days(w: &JpWorld, pts: &[(f64, f64)]) -> (f64, cartalith_civ::JpJourneyPlan) {
    let jp = cartalith_civ::jp_plan(w, pts, &JpPlan::default(), &HashMap::new(), &|_, _| 1.0).expect("a real route plans");
    assert!(!jp.stages.is_empty(), "premise: stages derived");
    (jp.travel_days, jp)
}

/// Per stage: terrain, condition, km, the router-slope grade and its factor,
/// and the along-track grade `(gain + loss) / km`.
fn stage_lines(jp: &cartalith_civ::JpJourneyPlan) -> String {
    jp.stages
        .iter()
        .map(|s| {
            let along = if s.km > 0.0 { (s.gain + s.loss) / (s.km * 1000.0) } else { f64::NAN };
            match s.slope {
                Some(sl) => format!(
                    "[{} | {} | {:.1} km | grade {:.4}{} x{:.3} | along-track {:.4}]",
                    s.terrain,
                    s.route_cond,
                    s.km,
                    sl.grade,
                    if sl.on_way { " way" } else { "" },
                    cartalith_civ::jp_slope_mod(&sl),
                    along
                ),
                None => format!("[{} | {} | {:.1} km | no slope]", s.terrain, s.route_cond, s.km),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Protects (measurement only): the size of the slope term on a real world --
/// the distribution of the router's grade over land cells and the factor each
/// percentile gives off and on a way. Guards that land exists and the field is
/// not all zero.
#[test]
#[ignore = "measurement harness for Ruling CD batch 3; prints, pins nothing"]
fn slope_distribution_over_land() {
    let fx = fixture();
    let k = fx.to_grade();
    let mut g: Vec<f64> = (0..GW * GH).filter(|&i| fx.wb[i] == 0).map(|i| fx.slope[i] as f64 * k).collect();
    assert!(!g.is_empty(), "premise: land cells");
    g.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
    assert!(g[g.len() - 1] > 0.0, "premise: the slope field is not flat");
    println!(
        "world seed {SEED} {GW}x{GH}: peak {} m, sea {:.4}, cell {:.2} km, grade per unit slope {:.3}",
        fx.p.peak_m,
        fx.ws.sea_level,
        fx.p.map_width_km / GW as f64,
        k
    );
    for q in [0.10, 0.25, 0.50, 0.75, 0.90, 0.99, 1.0] {
        let v = g[((g.len() - 1) as f64 * q) as usize];
        let off = cartalith_civ::jp_slope_mod(&cartalith_civ::JpSlope { grade: v, on_way: false });
        let on = cartalith_civ::jp_slope_mod(&cartalith_civ::JpSlope { grade: v, on_way: true });
        println!("land grade p{:>3}: {:.4} -> factor off-way {:.3}, on-way {:.3}", (q * 100.0) as i32, v, off, on);
    }
    // The coastal cells: a land cell beside the sea differences against the
    // sea floor (a limitation the router shares).
    let coast: Vec<f64> = (0..GW * GH)
        .filter(|&i| {
            let (x, y) = (i % GW, i / GW);
            fx.wb[i] == 0 && x > 0 && x + 1 < GW && y > 0 && y + 1 < GH && [i - 1, i + 1, i - GW, i + GW].iter().any(|&j| fx.wb[j] == 1)
        })
        .map(|i| fx.slope[i] as f64 * k)
        .collect();
    let inland_mean = g.iter().sum::<f64>() / g.len() as f64;
    let coast_mean = coast.iter().sum::<f64>() / coast.len().max(1) as f64;
    println!("mean grade: all land {inland_mean:.4}, coastal land ({} cells) {coast_mean:.4}", coast.len());
}

/// Protects (measurement only): before/after of Ruling CD items 2 and 5 on the
/// audit's Q1 routes -- a land route with no way, and the same route with a
/// hand-drawn Road, Track or Ancient way through the audit's detour waypoint --
/// planned without the slope field (before) and with it (after), per stage.
/// Guards that each route is real and each way stamps road cells.
#[test]
#[ignore = "measurement harness for Ruling CD batch 3; prints, pins nothing"]
fn q1_routes_before_and_after_slope() {
    let fx = fixture();
    let a = fx.nearest_land(0.30, 0.50);
    let b = fx.nearest_land(0.70, 0.50);
    let m = fx.nearest_land(0.50, 0.25);
    let none: Vec<WayRef> = Vec::new();
    let ctx0 = fx.ctx(&none);
    let bare = tools::civ_dijkstra_path(&ctx0, a.0, a.1, b.0, b.1, RouteMode::Land);
    assert!(bare.reachable && bare.pts.len() > 2, "land fixture must route");
    let empty: HashMap<(i64, i64), cartalith_civ::JpRoadCell> = HashMap::new();
    let (d0, _) = travel_days(&fx.jp(&empty, false), &bare.pts);
    let (d1, j1) = travel_days(&fx.jp(&empty, true), &bare.pts);
    println!("Q1 no-way route ({:.1} km): before {d0:.3} d, after {d1:.3} d ({:+.1}%)", bare.km, (d1 / d0 - 1.0) * 100.0);
    println!("   stages after: {}", stage_lines(&j1));

    let mut routes = Vec::new();
    for ty in [ManualWayType::Road, ManualWayType::Track, ManualWayType::Ancient] {
        let cw = tools::civ_commit_way(&ctx0, &[a, m, b], ty).expect("three waypoints commit a way");
        let way = cw.way;
        let refs = vec![WayRef::from(&way)];
        let ctx = fx.ctx(&refs);
        let route = tools::civ_dijkstra_path(&ctx, a.0, a.1, b.0, b.1, RouteMode::Land);
        assert!(route.reachable && route.pts.len() > 2);
        let road = cartalith_civ::jp_road_cells(&[], std::slice::from_ref(&way), &[], GW);
        assert!(!road.is_empty(), "premise: the way stamps road cells");
        let (before, jb) = travel_days(&fx.jp(&road, false), &route.pts);
        let (after, ja) = travel_days(&fx.jp(&road, true), &route.pts);
        println!(
            "Q1 {ty:?} way: route {:.1} km; before {before:.3} d, after {after:.3} d ({:+.1}%)",
            route.km,
            (after / before - 1.0) * 100.0
        );
        println!("   stages before: {}", stage_lines(&jb));
        println!("   stages after:  {}", stage_lines(&ja));
        routes.push((ty, route.pts));
    }
    // Track's router discount moved (0.25 -> 0.2941): does its route still
    // equal the Road's?
    println!("Q1 Track route identical to Road route: {}", routes[0].1 == routes[1].1);
}
