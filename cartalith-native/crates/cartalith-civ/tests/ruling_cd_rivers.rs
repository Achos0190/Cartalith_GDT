//! Ruling CD batch 2 (owner, 2026-10-11, item 1): real river travel. Unit tests
//! on hand-built fixtures for the Journey Planner's River Transport on traced
//! channels (`jp_derive_stages`' channel branch, `cartalith_civ::river_class`).
//!
//! What the fixture is. A 40 x 16 all-land world with one straight east-flowing
//! channel along row 8 (cells x = 2..=37): every channel cell's receiver is its
//! east neighbour (the last has none), its Strahler order and discharge are
//! set per test, the threshold is 10. Cells are 12 km. The bed falls gently
//! west to east unless a test says otherwise. The route runs along the channel,
//! one point per cell, x = 4..=35.
//!
//! Every expectation is a literal measured once on the fixture and pinned, or an
//! independent quantity (the reference's `JP_ROUTE.river` literals 1.40 / 0.55
//! for the up/down ratio); each `Protects:` line names the behaviour guarded,
//! and each core test was checked to fail under a mutant of the code it guards
//! (see the batch report's mutation list).

use std::collections::HashMap;

use cartalith_civ::{JpLegCalc, JpPlan, JpRiverNet, JpRoadCell, JpWorld};

const W: usize = 40;
const H: usize = 16;
const ROW: usize = 8;
const THRESH: f64 = 10.0;

/// The fixture's owned rasters.
struct Rv {
    field: Vec<f32>,
    biome: Vec<u8>,
    terrain: Vec<u8>,
    temp: Vec<f32>,
    rain: Vec<f32>,
    flow: Vec<f32>,
    wb: Vec<u8>,
    order: Vec<i16>,
    recv: Vec<i32>,
}

/// A gentle bed: 0.60 at x = 0 falling 0.002 per cell, about 1.15 m/km at a
/// 4000 m peak and 12 km cells -- well under the 8 m/km "felt current".
fn gentle(x: usize) -> f32 {
    0.60 - 0.002 * x as f32
}

/// Builds the fixture: channel order `ord` and discharge `q` along row 8,
/// receivers pointing east when `with_recv`, bed `bed(x)` everywhere.
fn river(ord: i16, q: f32, bed: impl Fn(usize) -> f32, with_recv: bool) -> Rv {
    let n = W * H;
    let mut r = Rv {
        field: (0..n).map(|i| bed(i % W)).collect(),
        biome: vec![5; n],
        terrain: vec![0; n],
        temp: vec![15.0; n],
        rain: vec![0.5; n],
        flow: vec![0.0; n],
        wb: vec![0; n],
        order: vec![0; n],
        recv: vec![-1; n],
    };
    for x in 2..=37 {
        let i = ROW * W + x;
        r.order[i] = ord;
        r.flow[i] = q;
        if with_recv && x < 37 {
            r.recv[i] = (i + 1) as i32;
        }
    }
    r
}

impl Rv {
    /// The planner's world over this fixture, with or without the network.
    fn world<'a>(&'a self, road: &'a HashMap<(i64, i64), JpRoadCell>, net: bool) -> JpWorld<'a> {
        JpWorld {
            gw: W,
            gh: H,
            world: false,
            map_width_km: W as f64 * 12.0,
            sea_level: 0.42,
            peak_m: 4000.0,
            field: &self.field,
            cart_biome: &self.biome,
            cart_terrain: &self.terrain,
            temp: &self.temp,
            rain: &self.rain,
            flow_field: Some(&self.flow),
            flow_thresh: THRESH,
            water_bodies: Some(&self.wb),
            territory: None,
            places: &[],
            road_cells: road,
            ocean_field: None,
            wind_field: None,
            river_net: net.then_some(JpRiverNet { order: &self.order, recv: &self.recv }),
            slope_field: None, // Ruling CD item 2: no slope data, so these plans keep the reference's (slope-free) land time
        }
    }
}

fn down() -> Vec<(f64, f64)> {
    (4..=35).map(|x| (x as f64, ROW as f64)).collect()
}

fn up() -> Vec<(f64, f64)> {
    down().into_iter().rev().collect()
}

fn plan(transport: &str, vessel: &str) -> JpPlan {
    JpPlan { transport: transport.into(), vessel: vessel.into(), ..JpPlan::default() }
}

/// One planned journey's stages as `(cat, terrain, route_cond)`, the vessel each
/// leg used, and its water days (land legs count as `NaN`, which no test pins).
struct Planned {
    stages: Vec<(String, String, String)>,
    vessels: Vec<String>,
    days: f64,
}

fn plan_on(r: &Rv, net: bool, pts: &[(f64, f64)], p: &JpPlan) -> Planned {
    let road = HashMap::new();
    let w = r.world(&road, net);
    let jp = cartalith_civ::jp_plan(&w, pts, p, &HashMap::new(), &|_, _| 1.0).expect("the fixture route plans");
    assert!(!jp.stages.is_empty(), "premise: the planner derived stages");
    let days = jp
        .results
        .iter()
        .map(|x| match &x.calc {
            Ok(JpLegCalc::Water(wc)) => wc.days,
            Ok(JpLegCalc::Land(l)) => l.days,
            Err(e) => panic!("premise: every leg travels, got blocked: {}", e.reason),
        })
        .sum();
    Planned {
        stages: jp.stages.iter().map(|s| (s.cat.clone(), s.terrain.clone(), s.route_cond.clone())).collect(),
        vessels: jp.results.iter().map(|x| x.eff.vessel.clone()).collect(),
        days,
    }
}

fn one(cat: &str, terrain: &str, cond: &str) -> Vec<(String, String, String)> {
    vec![(cat.to_string(), terrain.to_string(), cond.to_string())]
}

/// Protects: Ruling CD item 1's core. A big (order 5), strongly flowing channel
/// (discharge 200x threshold, strength saturated) planned as River Transport is
/// ONE `river` stage on `Calm River`, reading Strong Downstream with the flow and
/// Strong Upstream against it, and the upstream trip takes exactly 1.40 / 0.55
/// times as long (the reference's `JP_ROUTE.river` literals, independent of the
/// code under test). Before the ruling this route was land, walked as a portage.
#[test]
fn downstream_is_faster_than_upstream_by_the_reference_ratio() {
    let r = river(5, 2000.0, gentle, true);
    let p = plan("River Transport", "Keelboat");
    let d = plan_on(&r, true, &down(), &p);
    let u = plan_on(&r, true, &up(), &p);
    assert_eq!(d.stages, one("river", "Calm River", "Strong Downstream"));
    assert_eq!(u.stages, one("river", "Calm River", "Strong Upstream"));
    assert_eq!(d.vessels, vec!["Keelboat".to_string()]);
    assert_eq!(d.days, 8.480_604_037_861_795);
    assert_eq!(u.days, 21.586_992_096_375_47);
    assert!((u.days / d.days - 1.40 / 0.55).abs() < 1e-12, "ratio {}", u.days / d.days);
}

/// Protects: the current scales with discharge. A modest discharge (3x the
/// threshold, strength ln(4)/5 = 0.277) reads only Mild, both ways.
#[test]
fn a_modest_discharge_reads_mild() {
    let r = river(5, 30.0, gentle, true);
    let p = plan("River Transport", "Keelboat");
    assert_eq!(plan_on(&r, true, &down(), &p).stages, one("river", "Calm River", "Mild Downstream"));
    assert_eq!(plan_on(&r, true, &up(), &p).stages, one("river", "Calm River", "Mild Upstream"));
}

/// Protects: discharge 0 is a still channel (Neutral both ways, equal days),
/// never a strong current.
#[test]
fn zero_discharge_is_neutral_not_strong() {
    let r = river(5, 0.0, gentle, true);
    let p = plan("River Transport", "Keelboat");
    let d = plan_on(&r, true, &down(), &p);
    let u = plan_on(&r, true, &up(), &p);
    assert_eq!(d.stages, one("river", "Calm River", "Neutral"));
    assert_eq!(u.stages, one("river", "Calm River", "Neutral"));
    assert_eq!(d.days, u.days);
    assert_eq!(d.days, 11.872_845_653_006_513);
}

/// Protects: a channel with no receiver has no known flow direction, so the
/// stage keeps the reference's elevation condition (here a bed falling 0.014 per
/// cell, about 8.05 m/km: Mild Downstream west to east) instead of a fabricated
/// Neutral; the same bed WITH receivers and no discharge reads Neutral, so the
/// two paths are told apart.
#[test]
fn no_flow_direction_falls_back_to_the_reference_elevation_condition() {
    let steep = |x: usize| 0.99 - 0.014 * x as f32;
    let p = plan("River Transport", "Keelboat");
    let no_recv = river(5, 2000.0, steep, false);
    assert_eq!(plan_on(&no_recv, true, &down(), &p).stages, one("river", "Calm River", "Mild Downstream"));
    let still = river(5, 0.0, steep, true);
    assert_eq!(plan_on(&still, true, &down(), &p).stages, one("river", "Moderate River", "Neutral"));
}

/// Protects: what is NOT a river. The channel is a river only under River
/// Transport, only with a network supplied (no network is land, not a
/// zero-flow river), only at or above the navigable order 3, and only within
/// one cell of the channel: row 9 still reads river, row 10 reads land.
#[test]
fn only_river_transport_on_a_navigable_channel_reads_as_river() {
    let r = river(5, 2000.0, gentle, true);
    let land = |pl: &Planned| pl.stages.iter().all(|s| s.0 == "land");
    assert!(land(&plan_on(&r, true, &down(), &plan("Walking", "Keelboat"))), "Walking on the channel is land");
    assert!(land(&plan_on(&r, false, &down(), &plan("River Transport", "Keelboat"))), "no network is land");
    let order2 = river(2, 2000.0, gentle, true);
    assert!(land(&plan_on(&order2, true, &down(), &plan("River Transport", "Keelboat"))), "order 2 is not navigable");
    let row = |y: f64| -> Vec<(f64, f64)> { (4..=35).map(|x| (x as f64, y)).collect() };
    let rt = plan("River Transport", "Keelboat");
    assert_eq!(plan_on(&r, true, &row(9.0), &rt).stages, one("river", "Calm River", "Strong Downstream"));
    assert!(land(&plan_on(&r, true, &row(10.0), &rt)), "two cells off the channel is land");
    assert!(land(&plan_on(&r, true, &row(2.0), &rt)), "far off the channel is land");
}

/// Protects: the terrain rows from order and gradient (order 3 Shallows, order
/// 4 Moderate) and the vessel rules: a Cog (sea-only) cannot take a river stage
/// and is substituted by the first river hull in preference order (Keelboat);
/// River Barge stays on a calm channel but cannot run rapids (a bed falling
/// 0.13 per cell, about 75 m/km) and is substituted there.
#[test]
fn terrain_rows_and_vessel_rules_follow_the_channel() {
    let rt = |v: &str| plan("River Transport", v);
    let s3 = plan_on(&river(3, 2000.0, gentle, true), true, &down(), &rt("Keelboat"));
    assert_eq!(s3.stages, one("river", "River with Shallows", "Strong Downstream"));
    let s4 = plan_on(&river(4, 2000.0, gentle, true), true, &down(), &rt("Keelboat"));
    assert_eq!(s4.stages, one("river", "Moderate River", "Strong Downstream"));
    let calm = river(6, 2000.0, gentle, true);
    let cog = plan_on(&calm, true, &down(), &rt("Cog"));
    assert_eq!(cog.vessels, vec!["Keelboat".to_string()], "a Cog cannot sail a river");
    let barge = plan_on(&calm, true, &down(), &rt("River Barge"));
    assert_eq!(barge.vessels, vec!["River Barge".to_string()]);
    let rapids = river(6, 2000.0, |x| 5.5 - 0.13 * x as f32, true);
    let br = plan_on(&rapids, true, &down(), &rt("River Barge"));
    assert_eq!(br.stages, one("river", "River with Rapids", "Strong Downstream"));
    assert_eq!(br.vessels, vec!["Keelboat".to_string()], "a River Barge cannot run rapids");
}

// ------------------------------------------------------------- lake margin ----

/// The audit's lake strip (`planner_audit_journey.rs` `lake`): a 3-cell-wide
/// lake on a 64 x 16 world, 1 km cells. `bed` sets the strip's field.
fn lake_world(bed: impl Fn(usize) -> f32) -> (Vec<f32>, Vec<u8>, Vec<u8>, Vec<f32>, Vec<f32>, Vec<f32>, Vec<u8>) {
    let (lw, lh) = (64usize, 16usize);
    let n = lw * lh;
    let (mut field, mut biome, mut wb) = (vec![0.95f32; n], vec![5u8; n], vec![0u8; n]);
    let mut flow = vec![0.0f32; n];
    for y in 7..=9 {
        for x in 4..60 {
            let i = y * lw + x;
            biome[i] = 14;
            wb[i] = 2;
            field[i] = bed(x);
            flow[i] = 1.0e6;
        }
    }
    (field, biome, vec![0u8; n], vec![15.0; n], vec![0.5; n], flow, wb)
}

/// Protects: a lake margin is unchanged by Ruling CD item 1. Under River
/// Transport, with a river network supplied whose orders mark the lake itself
/// as a big channel, the lake strip still derives the reference's `Calm River`
/// with its elevation condition, and the days equal the audit's HEAD
/// measurement before this batch (`PLANNER_AUDIT_2026-10-11.md` Q3: 1.212 /
/// 3.084 sloping, 1.696 flat; pinned here at full precision as measured on the
/// pre-batch tree).
#[test]
fn a_lake_margin_stage_is_unchanged() {
    let (lw, lh) = (64usize, 16usize);
    let order = vec![7i16; lw * lh];
    let recv: Vec<i32> = (0..lw * lh).map(|i| if i % lw + 1 < lw { (i + 1) as i32 } else { -1 }).collect();
    let fwd: Vec<(f64, f64)> = (5..59).map(|x| (x as f64, 8.0)).collect();
    let rev: Vec<(f64, f64)> = fwd.iter().rev().copied().collect();
    let p = plan("River Transport", "Keelboat");
    let road: HashMap<(i64, i64), JpRoadCell> = HashMap::new();
    for (sloping, want_fwd, want_rev, cf, cr) in [
        (true, 1.211_506_107_819_472, 3.083_833_728_995_019_3, "Strong Downstream", "Strong Upstream"),
        (false, 1.696_108_550_947_260_9, 1.696_108_550_947_260_9, "Neutral", "Neutral"),
    ] {
        let (field, biome, terrain, temp, rain, flow, wb) =
            lake_world(|x| if sloping { 0.9 - 0.4 * (x as f32 - 4.0) / 55.0 } else { 0.6 });
        for net in [true, false] {
            let w = JpWorld {
                gw: lw,
                gh: lh,
                world: false,
                map_width_km: lw as f64,
                sea_level: 0.42,
                peak_m: 4000.0,
                field: &field,
                cart_biome: &biome,
                cart_terrain: &terrain,
                temp: &temp,
                rain: &rain,
                flow_field: Some(&flow),
                flow_thresh: 10.0,
                water_bodies: Some(&wb),
                territory: None,
                places: &[],
                road_cells: &road,
                ocean_field: None,
                wind_field: None,
                river_net: net.then_some(JpRiverNet { order: &order, recv: &recv }),
                slope_field: None, // Ruling CD item 2: no slope data, so these plans keep the reference's (slope-free) land time
            };
            for (pts, want, cond) in [(&fwd, want_fwd, cf), (&rev, want_rev, cr)] {
                let jp = cartalith_civ::jp_plan(&w, pts, &p, &HashMap::new(), &|_, _| 1.0).expect("plans");
                let st: Vec<(&str, &str, &str)> =
                    jp.stages.iter().map(|s| (s.cat.as_str(), s.terrain.as_str(), s.route_cond.as_str())).collect();
                assert_eq!(st, vec![("river", "Calm River", cond)], "net {net} sloping {sloping}");
                assert_eq!(jp.total_days, Some(want), "net {net} sloping {sloping}");
            }
        }
    }
}

/// Makes cells `x0..=x1` of rows 7-9 a lake (`cart_biome` 14, water class 2):
/// three rows wide, so every lake point on row 8 is within two cells of shore
/// and derives the reference's `Calm River`.
fn add_lake(r: &mut Rv, x0: usize, x1: usize) {
    for y in 7..=9 {
        for x in x0..=x1 {
            r.biome[y * W + x] = 14;
            r.wb[y * W + x] = 2;
        }
    }
}

/// Protects: a channel reach and a lake margin never chunk into one stage (the
/// `|ch` chunk-key marker). The channel runs x = 2..=20 and a lake fills x =
/// 21..=37 on the same row; the river reach reads its flow (Strong Downstream)
/// and the lake keeps the reference's elevation condition (Neutral on this
/// gentle bed), as two stages.
#[test]
fn a_channel_reach_and_a_lake_margin_stay_separate_stages() {
    let mut r = river(5, 2000.0, gentle, true);
    for x in 21..=37 {
        r.order[ROW * W + x] = 0;
        r.flow[ROW * W + x] = 0.0;
    }
    add_lake(&mut r, 21, 37);
    let got = plan_on(&r, true, &down(), &plan("River Transport", "Keelboat")).stages;
    let s = |t: &str, c: &str| ("river".to_string(), t.to_string(), c.to_string());
    assert_eq!(got, vec![s("Calm River", "Strong Downstream"), s("Calm River", "Neutral")]);
}

/// Protects: a channel inside a lake is the lake's, not a navigable channel a
/// land point beside the lake can borrow. Row 8 is a lake whose cells carry a
/// big order; a route along row 6 (land, one cell from the lake's edge row 7,
/// two from the order-carrying row) and along row 10 reads land.
#[test]
fn a_land_point_beside_a_lake_never_borrows_its_order() {
    let mut r = river(5, 2000.0, gentle, true);
    add_lake(&mut r, 2, 37);
    for y in [7usize, 9] {
        for x in 2..=37 {
            r.order[y * W + x] = 5;
            r.flow[y * W + x] = 2000.0;
        }
    }
    let row = |y: f64| -> Vec<(f64, f64)> { (4..=35).map(|x| (x as f64, y)).collect() };
    let rt = plan("River Transport", "Keelboat");
    for y in [6.0, 10.0] {
        let got = plan_on(&r, true, &row(y), &rt);
        assert!(got.stages.iter().all(|s| s.0 == "land"), "row {y}: {:?}", got.stages);
    }
}

/// Protects: a river network not sized to the grid is no network (all land),
/// never indexed as if it were one.
#[test]
fn a_mis_sized_network_is_no_network() {
    let r = river(5, 2000.0, gentle, true);
    let road = HashMap::new();
    let mut w = r.world(&road, true);
    w.river_net = Some(JpRiverNet { order: &r.order[..W * H - 1], recv: &r.recv });
    let jp = cartalith_civ::jp_plan(&w, &down(), &plan("River Transport", "Keelboat"), &HashMap::new(), &|_, _| 1.0)
        .expect("plans");
    assert!(jp.stages.iter().all(|s| s.cat == "land"), "{:?}", jp.stages.iter().map(|s| &s.cat).collect::<Vec<_>>());
}
