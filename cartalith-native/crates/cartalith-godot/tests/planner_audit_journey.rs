//! Journey-planner route audit (2026-10-11): measurement harness, **ignored**.
//!
//! What this file is for. The owner asked four questions about the Journey
//! Planner: (1) does a journey follow the ways laid on the map, (2) does leg
//! speed read terrain and way type from the same data the router and map use,
//! (3) do river legs use current by flow direction, (4) do sea legs use ocean
//! currents and wind. This file answers each one by running the real code on a
//! real generated world (`generate_terrain`, default engine parameters) or on a
//! small hand-built `JpWorld` where a real world cannot isolate the effect, and
//! printing what it measured. It changes no behaviour and pins no value: every
//! assertion below only guards that the fixture really reached the code under
//! test (non-empty route, the river trace really is on river cells, and so on),
//! so a fixture that silently measures nothing fails instead of reporting.
//!
//! Why it lives in `cartalith-godot/tests`: it is the only crate whose
//! dependency set includes `cartalith-civ`, `cartalith-engine` **and**
//! `cartalith-climate` (needed to build the real wind/current fields the
//! planner reads), and an integration test can use those without touching any
//! `Cargo.toml`. `WorldGen` itself is a cdylib `GodotClass` and cannot be
//! built here, so the `JpWorld` is assembled the way
//! `story_bridge::WorldGen::jp_world_parts`/`jp_world` assemble it, from the
//! same public functions (`build_cart_biome`, `build_cart_terrain`,
//! `jp_road_cells`, `current_ocean_field`, `current_wind_field`).
//!
//! Run: `cargo test -p cartalith-godot --test planner_audit_journey -- --ignored --nocapture`.
//! Must never be un-ignored into the default suite: it generates worlds and
//! prints tables, and its numbers are observations, not contracts.

use std::collections::{HashMap, HashSet};

use cartalith_civ::tools::{self, ManualWay, ManualWayType, RouteContext, RouteMode, WayRef};
use cartalith_civ::{JpCoarseField, JpLegCalc, JpPlan, JpWorld};
use cartalith_engine::{generate_terrain, WorldParams, WorldState};
use cartalith_jsmath::js_round;

const SEED: i32 = 24_601;
const GW: usize = 256;
const GH: usize = 192;

/// A generated world plus the derived rasters the router and the planner read.
/// Built once per test; nothing here is cached across tests.
struct Fx {
    ws: WorldState,
    p: WorldParams,
    wb: Vec<u8>,
    cart_biome: Vec<u8>,
    cart_terrain: Vec<u8>,
    flow_thresh: f64,
}

/// Generates the terrain stage at `GW` x `GH` with the engine's default
/// parameters and derives water bodies, cart biome/terrain and the river
/// threshold exactly as `WorldGen::jp_world_parts` does.
fn fixture() -> Fx {
    let p = WorldParams::defaults(GW, GH, SEED);
    let ws = generate_terrain(&p);
    let wb = cartalith_civ::build_water_bodies(&ws.field, GW, GH, ws.sea_level, false, Some(&ws.rainfall)).classification;
    let cart_biome = cartalith_civ::build_cart_biome(&ws.field, &wb, &ws.temperature, &ws.rainfall, GW, GH, false, ws.sea_level);
    let cart_terrain = cartalith_civ::build_cart_terrain(&ws.field, &wb, &ws.temperature, &ws.rainfall, GW, GH, false, ws.sea_level);
    let flow_thresh = cartalith_hydrology::river_flow_thresh(GW, GH, GW, p.map_width_km);
    Fx { ws, p, wb, cart_biome, cart_terrain, flow_thresh }
}

impl Fx {
    /// The Route/Way tools' context over this world with `ways` as the
    /// existing infrastructure, the same fields `route_commit` passes except
    /// `corridors`/`biome`/`river_order` (left `None`; the existing-way
    /// discount under test does not read them).
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

    /// A `JpWorld` over this world, as `WorldGen::jp_world` builds it, with the
    /// caller's road table and (optional) coarse current/wind fields. No
    /// settlements and no territory: both feed only the infrastructure tier,
    /// which is then equal on every compared run.
    fn jp<'a>(
        &'a self,
        road: &'a HashMap<(i64, i64), cartalith_civ::JpRoadCell>,
        ocean: Option<&'a JpCoarseField>,
        wind: Option<&'a JpCoarseField>,
    ) -> JpWorld<'a> {
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
            ocean_field: ocean,
            wind_field: wind,
            // The audit's own pre-Ruling-CD path: no river network, so its Q3
            // numbers stay the baseline (`planner_audit_rivers.rs` measures the
            // network-on path).
            river_net: None,
            measure_slope: false, // Ruling CD item 2: no slope data, so these plans keep the reference's (slope-free) land time
        }
    }

    /// The cell of `class` (0 land, 1 ocean) nearest a fraction of the map.
    fn nearest(&self, class: u8, fx: f64, fy: f64) -> (f64, f64) {
        let (tx, ty) = (fx * GW as f64, fy * GH as f64);
        let mut best = None;
        let mut bd = f64::INFINITY;
        for y in 0..GH {
            for x in 0..GW {
                if self.wb[y * GW + x] != class {
                    continue;
                }
                let d = (x as f64 - tx).powi(2) + (y as f64 - ty).powi(2);
                if d < bd {
                    bd = d;
                    best = Some((x as f64, y as f64));
                }
            }
        }
        best.expect("the fixture world has cells of this class")
    }
}

/// Every integer cell a polyline passes through, in travel order, by stepping
/// each segment at most one cell at a time (the same stepping rule as
/// `civ_walk_way_cells`, which is private). Asserts non-empty: an empty raster
/// would make every overlap ratio below vacuous.
fn raster(pts: &[(f64, f64)]) -> Vec<(i64, i64)> {
    let mut out: Vec<(i64, i64)> = Vec::new();
    let push = |c: (i64, i64), out: &mut Vec<(i64, i64)>| {
        if out.last() != Some(&c) {
            out.push(c);
        }
    };
    if let Some(&(x, y)) = pts.first() {
        push((js_round(x) as i64, js_round(y) as i64), &mut out);
    }
    for w in pts.windows(2) {
        let ((x0, y0), (x1, y1)) = (w[0], w[1]);
        let n = (x1 - x0).abs().max((y1 - y0).abs()).ceil().max(1.0);
        let mut s = 1.0;
        while s <= n {
            push((js_round(x0 + (x1 - x0) * s / n) as i64, js_round(y0 + (y1 - y0) * s / n) as i64), &mut out);
            s += 1.0;
        }
    }
    assert!(!out.is_empty(), "raster of a polyline must not be empty");
    out
}

/// `cells` dilated by one cell (3x3), the same neighbourhood both
/// `_civMarkWayNeighborhood` and `_jpRoadCells` use.
fn dilate(cells: &[(i64, i64)]) -> HashSet<(i64, i64)> {
    let mut s = HashSet::new();
    for &(x, y) in cells {
        for dy in -1..=1 {
            for dx in -1..=1 {
                s.insert((x + dx, y + dy));
            }
        }
    }
    s
}

/// How a route relates to a way: the share of the route's cells within one
/// cell of the way, the count of route cells off it, the share of the way's own
/// cells the route visits (coverage), and the number of backwards steps when
/// each on-way route cell is mapped to its nearest way-cell index (0 means the
/// route walks the way in the way's own direction, monotonically).
struct Overlap {
    on_frac: f64,
    off_cells: usize,
    coverage: f64,
    backsteps: usize,
}

fn overlap(route: &[(f64, f64)], way: &[(f64, f64)]) -> Overlap {
    let r = raster(route);
    let w = raster(way);
    let wd = dilate(&w);
    let rd = dilate(&r);
    let on: Vec<&(i64, i64)> = r.iter().filter(|c| wd.contains(c)).collect();
    let covered = w.iter().filter(|c| rd.contains(c)).count();
    let mut last = 0usize;
    let mut backsteps = 0usize;
    for c in &on {
        let k = w
            .iter()
            .enumerate()
            .min_by_key(|(_, q)| (q.0 - c.0).pow(2) + (q.1 - c.1).pow(2))
            .map(|(k, _)| k)
            .expect("way raster non-empty");
        if k + 2 < last {
            backsteps += 1;
        }
        last = last.max(k);
    }
    Overlap {
        on_frac: on.len() as f64 / r.len() as f64,
        off_cells: r.len() - on.len(),
        coverage: covered as f64 / w.len() as f64,
        backsteps,
    }
}

/// One line per derived stage: category, terrain, condition, km.
fn stage_lines(st: &[cartalith_civ::JpDerivedStage]) -> String {
    st.iter()
        .map(|s| format!("[{} | {} | {} | {:.1} km | rx {}]", s.cat, s.terrain, s.route_cond, s.km, s.rx))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Plans `pts` and returns `(total_days, travel_days)`; panics if the planner
/// returns nothing (a fixture that cannot be planned measured nothing).
fn plan_days(world: &JpWorld, pts: &[(f64, f64)], plan: &JpPlan) -> (Option<f64>, f64, cartalith_civ::JpJourneyPlan) {
    let jp = cartalith_civ::jp_plan(world, pts, plan, &HashMap::new(), &|_, _| 1.0).expect("the planner must return a plan for a real route");
    (jp.total_days, jp.travel_days, jp)
}

/// Protects (measurement only): question 1, routes followed. Builds one hand-drawn
/// way of each land type (Road/Track/Ancient) through a deliberate detour waypoint,
/// then commits a land Route between the way's two ends with and without the way on
/// the map, and reports how much of the route lies on the way, in which direction,
/// how the planner classifies the route's points, and the planned days. Guards only
/// that the fixture routes (reachable, more than two points, non-empty rasters).
#[test]
#[ignore = "measurement harness for the 2026-10-11 planner audit; prints, pins nothing"]
fn q1_land_route_follows_a_hand_drawn_way() {
    let fx = fixture();
    let a = fx.nearest(0, 0.30, 0.50);
    let b = fx.nearest(0, 0.70, 0.50);
    let m = fx.nearest(0, 0.50, 0.25);
    let none: Vec<WayRef> = Vec::new();
    let ctx0 = fx.ctx(&none);
    let bare = tools::civ_dijkstra_path(&ctx0, a.0, a.1, b.0, b.1, RouteMode::Land);
    assert!(bare.reachable && bare.pts.len() > 2, "land fixture must route: A {a:?} B {b:?}");
    println!("Q1 fixture seed {SEED} {GW}x{GH}: A {a:?} B {b:?} detour waypoint {m:?}");
    println!("Q1 route with NO way: {} pts, {:.1} km", bare.pts.len(), bare.km);

    let mut prev_pts: Option<Vec<(f64, f64)>> = None;
    for ty in [ManualWayType::Road, ManualWayType::Track, ManualWayType::Ancient] {
        let cw = tools::civ_commit_way(&ctx0, &[a, m, b], ty).expect("three waypoints commit a way");
        let way: ManualWay = cw.way;
        assert!(way.pts.len() > 2 && cw.unreachable_legs == 0, "{ty:?} way must be a real path");
        let refs = vec![WayRef::from(&way)];
        let ctx = fx.ctx(&refs);
        let route = tools::civ_dijkstra_path(&ctx, a.0, a.1, b.0, b.1, RouteMode::Land);
        assert!(route.reachable && route.pts.len() > 2);
        let o = overlap(&route.pts, &way.pts);
        let o_bare = overlap(&bare.pts, &way.pts);
        println!(
            "Q1 {ty:?}: way {:.1} km; route-with-way {:.1} km: on-way {:.3} ({} cells off), covers {:.3} of the way, backsteps {}; \
             control route-without-way: on-way {:.3}, covers {:.3}",
            way.km, route.km, o.on_frac, o.off_cells, o.coverage, o.backsteps, o_bare.on_frac, o_bare.coverage
        );
        // Reverse direction: does the route walk the way B->A?
        let back = tools::civ_dijkstra_path(&ctx, b.0, b.1, a.0, a.1, RouteMode::Land);
        let rev_way: Vec<(f64, f64)> = way.pts.iter().rev().copied().collect();
        let ob = overlap(&back.pts, &rev_way);
        println!("Q1 {ty:?} reverse B->A: on-way {:.3}, covers {:.3}, backsteps vs reversed way {}", ob.on_frac, ob.coverage, ob.backsteps);
        if let Some(pp) = &prev_pts {
            println!("Q1 {ty:?}: route identical to previous type's route: {}", pp == &route.pts);
        }
        prev_pts = Some(route.pts.clone());

        // The planner over the committed route.
        let road = cartalith_civ::jp_road_cells(&[], std::slice::from_ref(&way), &[], GW);
        assert!(!road.is_empty(), "the way must stamp road cells");
        let empty: HashMap<(i64, i64), cartalith_civ::JpRoadCell> = HashMap::new();
        let w_road = fx.jp(&road, None, None);
        let w_none = fx.jp(&empty, None, None);
        let hits = route
            .pts
            .iter()
            .filter(|&&(x, y)| road.contains_key(&(js_round(x).clamp(0.0, GW as f64 - 1.0) as i64, js_round(y).clamp(0.0, GH as f64 - 1.0) as i64)))
            .count();
        let sample = road.values().next().expect("non-empty");
        println!(
            "Q1 {ty:?}: planner road table stamps '{}'/'{}' (pri {}); {}/{} route points read as road",
            sample.terrain, sample.cond, sample.pri, hits, route.pts.len()
        );
        let plan = JpPlan::default();
        let st = cartalith_civ::jp_derive_stages(&w_road, &route.pts, &plan);
        println!("Q1 {ty:?}: stages with road table: {}", stage_lines(&st));
        let (t1, tr1, _) = plan_days(&w_road, &route.pts, &plan);
        let (t2, tr2, _) = plan_days(&w_none, &route.pts, &plan);
        let (t3, tr3, _) = plan_days(&w_none, &bare.pts, &plan);
        println!(
            "Q1 {ty:?}: walking days -- route on way, road credited: total {t1:?} travel {tr1:.2}; same route, road NOT credited: total {t2:?} travel {tr2:.2}; \
             no-way route: total {t3:?} travel {tr3:.2}"
        );
    }
}

/// Protects (measurement only): question 1 for sea lanes. Commits a sea lane
/// between two ocean cells through a detour waypoint, then a Water-mode Route with
/// and without the lane, and reports overlap and whether the planner's stages see
/// the lane at all. Guards only that both routes are real.
#[test]
#[ignore = "measurement harness for the 2026-10-11 planner audit; prints, pins nothing"]
fn q1_sea_route_follows_a_sea_lane() {
    let fx = fixture();
    // Two ocean cells far apart in one ocean, and a detour waypoint between.
    let mut found = None;
    'outer: for (fa, fb, fm) in [
        ((0.05, 0.10), (0.95, 0.10), (0.50, 0.25)),
        ((0.05, 0.90), (0.95, 0.90), (0.50, 0.75)),
        ((0.05, 0.10), (0.05, 0.90), (0.20, 0.50)),
        ((0.95, 0.10), (0.95, 0.90), (0.80, 0.50)),
    ] {
        let a = fx.nearest(1, fa.0, fa.1);
        let b = fx.nearest(1, fb.0, fb.1);
        let m = fx.nearest(1, fm.0, fm.1);
        let none: Vec<WayRef> = Vec::new();
        let ctx0 = fx.ctx(&none);
        let bare = tools::civ_dijkstra_path(&ctx0, a.0, a.1, b.0, b.1, RouteMode::Water);
        if bare.reachable && bare.pts.len() > 2 {
            if let Some(cw) = tools::civ_commit_way(&ctx0, &[a, m, b], ManualWayType::SeaLane) {
                if cw.unreachable_legs == 0 && cw.way.pts.len() > 2 {
                    found = Some((a, b, m, bare, cw.way));
                    break 'outer;
                }
            }
        }
    }
    let (a, b, m, bare, lane) = found.expect("the fixture world has a connected ocean pair");
    assert!(lane.sea, "a committed SeaLane must carry sea = true");
    let refs = vec![WayRef::from(&lane)];
    let ctx = fx.ctx(&refs);
    let route = tools::civ_dijkstra_path(&ctx, a.0, a.1, b.0, b.1, RouteMode::Water);
    let o = overlap(&route.pts, &lane.pts);
    let o0 = overlap(&bare.pts, &lane.pts);
    println!("Q1 sea: A {a:?} B {b:?} via {m:?}; lane {:.1} km; route-with-lane {:.1} km on-lane {:.3} covers {:.3} backsteps {} ({} cells off); control without lane: on-lane {:.3} covers {:.3}",
        lane.km, route.km, o.on_frac, o.coverage, o.backsteps, o.off_cells, o0.on_frac, o0.coverage);
    // Mixed mode (the general Route tool's default domain).
    let mixed = tools::civ_dijkstra_path(&ctx, a.0, a.1, b.0, b.1, RouteMode::Mixed);
    let om = overlap(&mixed.pts, &lane.pts);
    println!("Q1 sea, Mixed-mode route with lane: on-lane {:.3} covers {:.3}", om.on_frac, om.coverage);
    // The planner's view: jp_road_cells skips sea lanes, so stage derivation is
    // identical with or without the lane.
    let road = cartalith_civ::jp_road_cells(&[], std::slice::from_ref(&lane), &[], GW);
    println!("Q1 sea: planner road table entries from the lane: {}", road.len());
    let empty: HashMap<(i64, i64), cartalith_civ::JpRoadCell> = HashMap::new();
    let w = fx.jp(&empty, None, None);
    let plan = JpPlan { transport: "Sea Faring".into(), vessel: "Cog".into(), ..JpPlan::default() };
    let st = cartalith_civ::jp_derive_stages(&w, &route.pts, &plan);
    println!("Q1 sea: stages on the lane route: {}", stage_lines(&st));
}

/// Protects (measurement only): question 3 on a real river. Traces a river channel
/// downstream on the generated world (each step to the 8-neighbour with the most
/// discharge, which on a flow-accumulation field is the downstream cell) and plans
/// it as River Transport in both directions. Reports how the planner classifies the
/// river's points and the days each way. Guards that the trace really stays on
/// cells above the river threshold.
#[test]
#[ignore = "measurement harness for the 2026-10-11 planner audit; prints, pins nothing"]
fn q3_real_river_channel_is_planned_as_land() {
    let fx = fixture();
    let f = &fx.ws.flow_discharge;
    let trace_from = |s: usize| -> Vec<usize> {
        let mut path = vec![s];
        let mut c = s;
        loop {
            let (x, y) = ((c % GW) as i64, (c / GW) as i64);
            let mut best = c;
            for dy in -1..=1i64 {
                for dx in -1..=1i64 {
                    let (nx, ny) = (x + dx, y + dy);
                    if nx < 0 || ny < 0 || nx >= GW as i64 || ny >= GH as i64 {
                        continue;
                    }
                    let n = ny as usize * GW + nx as usize;
                    if f[n] > f[best] {
                        best = n;
                    }
                }
            }
            if best == c || fx.wb[best] != 0 || path.len() > 4 * GW {
                break;
            }
            path.push(best);
            c = best;
        }
        path
    };
    // The longest downstream trace from a river head (just above threshold).
    let mut best: Vec<usize> = Vec::new();
    for i in 0..GW * GH {
        if fx.wb[i] == 0 && (f[i] as f64) > fx.flow_thresh && (f[i] as f64) < 1.5 * fx.flow_thresh {
            let t = trace_from(i);
            if t.len() > best.len() {
                best = t;
            }
        }
    }
    assert!(best.len() >= 20, "the fixture must have a river at least 20 cells long, got {}", best.len());
    let on_river = best.iter().filter(|&&i| (f[i] as f64) > fx.flow_thresh).count();
    assert_eq!(on_river, best.len(), "every traced cell must be a river cell");
    let down: Vec<(f64, f64)> = best.iter().map(|&i| ((i % GW) as f64, (i / GW) as f64)).collect();
    let up: Vec<(f64, f64)> = down.iter().rev().copied().collect();
    let h = |p: (f64, f64)| fx.ws.field[p.1 as usize * GW + p.0 as usize];
    println!(
        "Q3 river: {} cells, head {:?} (h {:.4}) -> mouth {:?} (h {:.4}); flow {:.1} -> {:.1} (thresh {:.1})",
        down.len(), down[0], h(down[0]), down[down.len() - 1], h(down[down.len() - 1]),
        f[best[0]], f[best[best.len() - 1]], fx.flow_thresh
    );
    let empty: HashMap<(i64, i64), cartalith_civ::JpRoadCell> = HashMap::new();
    let w = fx.jp(&empty, None, None);
    let plan = JpPlan { transport: "River Transport".into(), vessel: "Keelboat".into(), ..JpPlan::default() };
    for (label, pts) in [("downstream", &down), ("upstream", &up)] {
        let st = cartalith_civ::jp_derive_stages(&w, pts, &plan);
        let (tot, trav, jp) = plan_days(&w, pts, &plan);
        let land_meta: Vec<String> = jp
            .results
            .iter()
            .map(|r| match &r.calc {
                Ok(JpLegCalc::Land(l)) => format!("land: {} portage={} {:.1} km/day", l.transport_label, l.portage, l.daily_km),
                Ok(JpLegCalc::Water(wc)) => format!("water: {} {:.1} km/day", wc.transport_label, wc.daily_km),
                Err(e) => format!("blocked: {}", e.reason),
            })
            .collect();
        println!("Q3 river {label}: stages {}", stage_lines(&st));
        println!("Q3 river {label}: total {tot:?} travel {trav:.2} days; legs {land_meta:?}");
    }
}

/// A hand-built `JpWorld` holding one lake strip (`cart_biome` 14) three cells
/// wide, so every point on its centre row is within two cells of land and the
/// planner classifies it `river`/`Calm River`. `bed(x)` sets the field along the
/// strip; `flow` is the discharge stamped on the strip (the planner may or may
/// not read it -- that is what is measured).
struct Lake {
    field: Vec<f32>,
    biome: Vec<u8>,
    terrain: Vec<u8>,
    temp: Vec<f32>,
    rain: Vec<f32>,
    flow: Vec<f32>,
    wb: Vec<u8>,
}

const LW: usize = 64;
const LH: usize = 16;

fn lake(bed: impl Fn(usize) -> f32, flow_on_strip: f32) -> Lake {
    let n = LW * LH;
    let mut l = Lake {
        field: vec![0.95; n],
        biome: vec![5; n],
        terrain: vec![0; n],
        temp: vec![15.0; n],
        rain: vec![0.5; n],
        flow: vec![0.0; n],
        wb: vec![0; n],
    };
    for y in 7..=9 {
        for x in 4..60 {
            let i = y * LW + x;
            l.biome[i] = 14;
            l.wb[i] = 2;
            l.field[i] = bed(x);
            l.flow[i] = flow_on_strip;
        }
    }
    l
}

impl Lake {
    fn world<'a>(&'a self, road: &'a HashMap<(i64, i64), cartalith_civ::JpRoadCell>) -> JpWorld<'a> {
        JpWorld {
            gw: LW,
            gh: LH,
            world: false,
            // 1 km cells, so a 0.4 field drop over ~53 km is ~52 m/km at peak 4000 m.
            map_width_km: LW as f64,
            sea_level: 0.42,
            peak_m: 4000.0,
            field: &self.field,
            cart_biome: &self.biome,
            cart_terrain: &self.terrain,
            temp: &self.temp,
            rain: &self.rain,
            flow_field: Some(&self.flow),
            flow_thresh: 10.0,
            water_bodies: Some(&self.wb),
            territory: None,
            places: &[],
            road_cells: road,
            ocean_field: None,
            wind_field: None,
            river_net: None,
            measure_slope: false, // Ruling CD item 2: no slope data, so these plans keep the reference's (slope-free) land time
        }
    }
}

/// Protects (measurement only): question 3's mechanism. On a hand-built "river"
/// stage (a lake strip, the only thing the planner calls a river), reports the
/// derived condition and water days in both directions for a sloping bed and a
/// flat bed, each with and without a large discharge on the strip -- i.e. whether
/// direction comes from flow or from the elevation of the points. Guards that
/// every point really derives as `river`.
#[test]
#[ignore = "measurement harness for the 2026-10-11 planner audit; prints, pins nothing"]
fn q3_river_condition_reads_elevation_not_flow() {
    let road: HashMap<(i64, i64), cartalith_civ::JpRoadCell> = HashMap::new();
    let fwd: Vec<(f64, f64)> = (5..59).map(|x| (x as f64, 8.0)).collect();
    let rev: Vec<(f64, f64)> = fwd.iter().rev().copied().collect();
    let plan = JpPlan { transport: "River Transport".into(), vessel: "Keelboat".into(), ..JpPlan::default() };
    for (bed_label, slope) in [("sloping bed (falls west->east)", true), ("flat bed", false)] {
        for flow in [0.0f32, 1.0e6] {
            let l = lake(|x| if slope { 0.9 - 0.4 * (x as f32 - 4.0) / 55.0 } else { 0.6 }, flow);
            let w = l.world(&road);
            for (dir, pts) in [("west->east", &fwd), ("east->west", &rev)] {
                let st = cartalith_civ::jp_derive_stages(&w, pts, &plan);
                assert!(st.iter().all(|s| s.cat == "river"), "lake-strip points must derive as river: {}", stage_lines(&st));
                let (tot, trav, _) = plan_days(&w, pts, &plan);
                println!("Q3 lake-strip {bed_label}, strip flow {flow:e}, {dir}: {} -> total {tot:?} travel {trav:.3} days", stage_lines(&st));
            }
        }
    }
}

/// The coarse current and wind fields exactly as `coarse_ocean_wind_fields`
/// (`cartalith-godot/src/lib.rs`) builds them for the planner, from `p`.
fn coarse_fields(fx: &Fx, p: &WorldParams) -> (JpCoarseField, JpCoarseField) {
    let ww = GW.min(240);
    let wh = (js_round(ww as f64 * GH as f64 / GW as f64) as usize).max(2);
    let sea = fx.ws.sea_level;
    let cur = cartalith_climate::current_ocean_field(
        GW, GH, &fx.ws.field, ww, wh, false, 3.0, sea, false,
        p.climate.lat_n, p.climate.lat_s, p.climate.equator_temp, p.climate.pole_temp,
        p.planet.axial_tilt_deg, p.planet.rotation_hours,
        p.climate.wind_manual, p.climate.wind_dir_deg, p.climate.press_k,
    );
    let max_speed = cur
        .u
        .iter()
        .zip(cur.v.iter())
        .zip(cur.ocean.iter())
        .filter(|&(_, &o)| o != 0)
        .map(|((&u, &v), _)| (u as f64).hypot(v as f64))
        .fold(1e-6_f64, f64::max);
    let ocean = JpCoarseField { ww, wh, u: cur.u, v: cur.v, max_speed };
    let wf = cartalith_climate::current_wind_field(
        GW, GH, &fx.ws.field, sea, p.peak_m, false,
        p.climate.lat_n, p.climate.lat_s, p.climate.equator_temp, p.climate.pole_temp,
        p.planet.axial_tilt_deg, p.planet.rotation_hours, p.climate.lapse_rate,
        p.climate.wind_manual, p.climate.wind_dir_deg, p.climate.press_k,
    );
    let wind = JpCoarseField { ww: wf.ww, wh: wf.wh, u: wf.u, v: wf.v, max_speed: wf.max_speed };
    (ocean, wind)
}

/// Protects (measurement only): question 4. Plans one sea route (Water mode, real
/// generated ocean) in both directions under the real current/wind fields, with the
/// wind left automatic and then forced from four directions (`wind_manual`,
/// `wind_dir_deg`), for a square-rigged Cog and an oared Keelboat-class hull, and
/// reports each stage's derived condition and the days. Guards that the route is a
/// real multi-point water path and that the planner derives at least one sea stage.
#[test]
#[ignore = "measurement harness for the 2026-10-11 planner audit; prints, pins nothing"]
fn q4_sea_leg_wind_and_current_both_directions() {
    let fx = fixture();
    let none: Vec<WayRef> = Vec::new();
    let ctx = fx.ctx(&none);
    let mut route = None;
    for (fa, fb) in [((0.05, 0.10), (0.95, 0.10)), ((0.05, 0.90), (0.95, 0.90)), ((0.05, 0.10), (0.05, 0.90)), ((0.95, 0.10), (0.95, 0.90))] {
        let a = fx.nearest(1, fa.0, fa.1);
        let b = fx.nearest(1, fb.0, fb.1);
        let r = tools::civ_dijkstra_path(&ctx, a.0, a.1, b.0, b.1, RouteMode::Water);
        if r.reachable && r.pts.len() > 2 && r.km > 100.0 {
            route = Some((a, b, r));
            break;
        }
    }
    let (a, b, r) = route.expect("the fixture world has a long water route");
    let fwd = r.pts.clone();
    let rev: Vec<(f64, f64)> = fwd.iter().rev().copied().collect();
    println!("Q4 sea route {a:?} -> {b:?}: {} pts, {:.1} km", fwd.len(), r.km);
    let empty: HashMap<(i64, i64), cartalith_civ::JpRoadCell> = HashMap::new();

    // Control: no fields at all.
    let w0 = fx.jp(&empty, None, None);
    let cog = JpPlan { transport: "Sea Faring".into(), vessel: "Cog".into(), ..JpPlan::default() };
    let st0 = cartalith_civ::jp_derive_stages(&w0, &fwd, &cog);
    assert!(st0.iter().any(|s| s.cat == "sea"), "the route must derive a sea stage: {}", stage_lines(&st0));
    println!("Q4 control, no current/wind fields: {}", stage_lines(&st0));

    let mut cases: Vec<(String, WorldParams)> = vec![("auto wind".into(), fx.p.clone())];
    for dir in [0.0, 90.0, 180.0, 270.0] {
        let mut p = fx.p.clone();
        p.climate.wind_manual = true;
        p.climate.wind_dir_deg = dir;
        cases.push((format!("manual wind {dir} deg"), p));
    }
    for (label, p) in &cases {
        let (oc, wi) = coarse_fields(&fx, p);
        // Mean wind vector over ocean, to show the field really changed.
        let (mut su, mut sv) = (0.0f64, 0.0f64);
        for i in 0..wi.u.len() {
            su += wi.u[i] as f64;
            sv += wi.v[i] as f64;
        }
        let n = wi.u.len() as f64;
        let w = fx.jp(&empty, Some(&oc), Some(&wi));
        for vessel in ["Cog", "Keelboat"] {
            let plan = JpPlan { transport: "Sea Faring".into(), vessel: vessel.into(), ..JpPlan::default() };
            let sf = cartalith_civ::jp_derive_stages(&w, &fwd, &plan);
            let sr = cartalith_civ::jp_derive_stages(&w, &rev, &plan);
            let (tf, _, jf) = plan_days(&w, &fwd, &plan);
            let (tr, _, jr) = plan_days(&w, &rev, &plan);
            let used = |j: &cartalith_civ::JpJourneyPlan| -> Vec<String> {
                j.results.iter().map(|x| match &x.calc {
                    Ok(JpLegCalc::Water(wc)) => format!("{} {:.0} km/d", wc.transport_label, wc.daily_km),
                    Ok(JpLegCalc::Land(l)) => format!("land {:.0} km/d", l.daily_km),
                    Err(e) => format!("blocked: {}", e.reason),
                }).collect()
            };
            println!("Q4 {label} (mean wind u {:.3} v {:.3}), {vessel}: fwd {} -> total {tf:?} {:?}", su / n, sv / n, stage_lines(&sf), used(&jf));
            println!("Q4 {label}, {vessel}: rev {} -> total {tr:?} {:?}", stage_lines(&sr), used(&jr));
        }
    }
}

/// Protects (measurement only): a road-source inconsistency. A generated way that
/// is hidden is skipped by `jp_road_cells`, but the raw network edge it was built
/// from is not, so the planner still credits the hidden road. Reports the road
/// cells produced by (way hidden, edge present) and (way hidden, no edge).
#[test]
#[ignore = "measurement harness for the 2026-10-11 planner audit; prints, pins nothing"]
fn hidden_generated_way_is_still_credited_through_its_edge() {
    let gw = 32usize;
    let pts: Vec<(f64, f64)> = (2..30).map(|x| (x as f64, 10.0)).collect();
    let way = cartalith_civ::Way {
        tid: 1,
        pts: pts.clone(),
        brks: Vec::new(),
        km: 28.0,
        name: String::new(),
        way_type: cartalith_civ::WayType::Highway,
        a_idx: 0,
        b_idx: 1,
        hidden: true,
    };
    let edge = cartalith_civ::RoadEdge { a: 0, b: 1, path: (2..30).map(|x| 10 * gw + x).collect() };
    let with_edge = cartalith_civ::jp_road_cells(std::slice::from_ref(&way), &[], std::slice::from_ref(&edge), gw);
    let no_edge = cartalith_civ::jp_road_cells(std::slice::from_ref(&way), &[], &[], gw);
    let c = with_edge.get(&(15, 10)).map(|r| (r.terrain, r.cond));
    println!("hidden highway: road cells with its edge {} (cell (15,10) = {c:?}); without the edge {}", with_edge.len(), no_edge.len());
}
