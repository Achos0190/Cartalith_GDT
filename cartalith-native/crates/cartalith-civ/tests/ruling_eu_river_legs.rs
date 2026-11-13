//! Ruling EU item 3 (owner, 2026-11-12): river stretches offered per stretch,
//! each with a recommended hull, and accepted stretches planned as river legs
//! whatever the journey's transport (`cartalith_civ::river_stretch`).
//!
//! What the fixture is. A 40 x 16 all-land world (12 km cells, the same shape
//! `ruling_cd_rivers.rs` uses) with one straight east-flowing channel along row 8
//! (cells x = 2..=37): every channel cell drains to its east neighbour, order 4,
//! discharge 30 against a threshold of 10 (a Mild current). A walking route runs
//! east along row 5, three cells north of the river, one point per cell,
//! x = 4..=35. Sizes, the 12 km and the offsets are test-local choices.
//!
//! Every expectation is a literal measured once on the fixture and pinned, or an
//! independent quantity; each `Protects:` line names the behaviour guarded.

use std::collections::HashMap;

use cartalith_civ::river_stretch::{
    ETA_MIN, JpRiverDir, JpRiverLeg, RECOMMEND_MARGIN, jp_apply_river_legs, jp_builtin_hulls, jp_rank_river_vessels,
    jp_recommend, jp_river_candidates,
};
use cartalith_civ::{JpPlan, JpRiverNet, JpRoadCell, JpWorld};

const W: usize = 40;
const H: usize = 16;
const ROW: usize = 8;

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

/// The fixture: the channel along row 8 at order `ord`, a gentle bed falling
/// east (0.60 at x = 0, 0.002 a cell: about 1 m/km, well under any current
/// threshold).
fn river(ord: i16) -> Rv {
    let n = W * H;
    let mut r = Rv {
        field: (0..n).map(|i| 0.60 - 0.002 * (i % W) as f32).collect(),
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
        r.flow[i] = 30.0;
        if x < 37 {
            r.recv[i] = (i + 1) as i32;
        }
    }
    r
}

impl Rv {
    fn world<'a>(&'a self, road: &'a HashMap<(i64, i64), JpRoadCell>) -> JpWorld<'a> {
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
            flow_thresh: 10.0,
            water_bodies: Some(&self.wb),
            territory: None,
            places: &[],
            road_cells: road,
            ocean_field: None,
            wind_field: None,
            river_net: Some(JpRiverNet { order: &self.order, recv: &self.recv }),
            measure_slope: false,
        }
    }
}

/// The walking route along row 5, west to east.
fn east() -> Vec<(f64, f64)> {
    (4..=35).map(|x| (x as f64, 5.0)).collect()
}

fn walking() -> JpPlan {
    JpPlan { transport: "Walking".into(), ..JpPlan::default() }
}

/// Protects the alignment threshold as a literal: `1 / 1.5` (Leopold and
/// Wolman's meander sinuosity), so a change to it is deliberate.
#[test]
fn eta_min_is_one_over_the_meander_sinuosity() {
    assert!((ETA_MIN - 0.666_666_666_666_666_6).abs() < 1e-15);
    assert!((RECOMMEND_MARGIN - 0.10).abs() < 1e-15);
}

/// Protects the stretch finder: a river running beside the route three cells off
/// is one downstream candidate over the channel cells level with the route
/// (x = 4..=35, 32 cells; the cells beyond either end tie on route km and are
/// longer), entering at the route's first point and leaving at its last, with
/// route km and channel km the independent step counts times 12 km (31 each).
#[test]
fn a_parallel_river_is_one_downstream_candidate() {
    let r = river(4);
    let road = HashMap::new();
    let w = r.world(&road);
    let c = jp_river_candidates(&w, &east(), 4.0, 24.0);
    assert_eq!(c.len(), 1, "{c:?}");
    let c = &c[0];
    assert_eq!(c.dir, JpRiverDir::Downstream);
    assert_eq!(c.cells.len(), 32);
    assert_eq!((c.cells[0], *c.cells.last().unwrap()), (ROW * W + 4, ROW * W + 35));
    assert_eq!((c.entry_k, c.exit_k), (0, 31));
    assert!((c.route_km - 31.0 * 12.0).abs() < 1e-9, "{}", c.route_km);
    assert!((c.channel_km - 31.0 * 12.0).abs() < 1e-9, "{}", c.channel_km);
}

/// Protects the corridor: the same river two cells beyond the radius is not
/// offered, and a route with no river network has no candidates.
#[test]
fn a_river_beyond_the_corridor_or_no_network_offers_nothing() {
    let r = river(4);
    let road = HashMap::new();
    let w = r.world(&road);
    assert!(jp_river_candidates(&w, &east(), 2.0, 24.0).is_empty());
    let mut no_net = r.world(&road);
    no_net.river_net = None;
    assert!(jp_river_candidates(&no_net, &east(), 4.0, 24.0).is_empty());
}

/// Protects the navigability gate: the same channel at order 2 (below
/// `NAVIGABLE_ORDER` 3) is never a candidate.
#[test]
fn a_channel_below_the_navigable_order_is_not_offered() {
    let r = river(2);
    let road = HashMap::new();
    assert!(jp_river_candidates(&r.world(&road), &east(), 4.0, 24.0).is_empty());
}

/// Protects the direction: the route walked west is offered the same channel
/// upstream, in travel order (east end first).
#[test]
fn a_route_against_the_flow_is_offered_upstream() {
    let r = river(4);
    let road = HashMap::new();
    let west: Vec<(f64, f64)> = east().into_iter().rev().collect();
    let c = jp_river_candidates(&r.world(&road), &west, 4.0, 24.0);
    assert_eq!(c.len(), 1);
    assert_eq!(c[0].dir, JpRiverDir::Upstream);
    assert_eq!(c[0].cells[0], ROW * W + 35);
}

/// Protects the alignment rule: a channel crossing the route at right angles
/// (north to south along x = 20) makes no progress along it and is not offered.
#[test]
fn a_river_across_the_route_is_not_offered() {
    let n = W * H;
    let mut r = river(4);
    r.order = vec![0; n];
    r.recv = vec![-1; n];
    for y in 0..H {
        let i = y * W + 20;
        r.order[i] = 4;
        r.flow[i] = 30.0;
        if y + 1 < H {
            r.recv[i] = (i + W) as i32;
        }
    }
    let road = HashMap::new();
    assert!(jp_river_candidates(&r.world(&road), &east(), 4.0, 24.0).is_empty());
}

/// Protects the alignment threshold itself: a channel that winds down and up
/// between rows 7 and 11 while it advances one column (a sinuosity near 5, far
/// past the meander threshold) beside the route is never offered, though every
/// cell is inside the corridor and the whole stretch is long enough.
#[test]
fn a_river_winding_far_more_than_a_meander_is_not_offered() {
    let n = W * H;
    let mut r = river(4);
    r.order = vec![0; n];
    r.recv = vec![-1; n];
    let mut chain: Vec<usize> = Vec::new();
    for x in 4..=35 {
        let rows: Vec<usize> = if x % 2 == 0 { (7..=11).collect() } else { (7..=11).rev().collect() };
        for y in rows {
            chain.push(y * W + x);
        }
    }
    for (k, &c) in chain.iter().enumerate() {
        r.order[c] = 4;
        r.flow[c] = 30.0;
        if k + 1 < chain.len() {
            r.recv[c] = chain[k + 1] as i32;
        }
    }
    let road = HashMap::new();
    let w = r.world(&road);
    // Premise: the corridor reaches every cell (row 11 is 6 cells off row 5).
    assert!(jp_river_candidates(&w, &east(), 8.0, 24.0).is_empty());
}

/// Protects owner question Q6's interim default: a party with a pack animal is
/// offered no river stretch and is told why (its animals would otherwise vanish
/// from the costing aboard), while the same party walking is offered one.
#[test]
fn a_party_with_animals_is_told_why_no_stretch_is_offered() {
    let r = river(4);
    let road = HashMap::new();
    let w = r.world(&road);
    // One cell off the river: inside a walking day's corridor (about two cells).
    let pts: Vec<(f64, f64)> = (4..=35).map(|x| (x as f64, 7.0)).collect();
    let none = HashMap::new();
    let plan = walking();
    let base = cartalith_civ::jp_plan(&w, &pts, &plan, &none, &|_, _| 1.0).unwrap();
    let hulls = jp_builtin_hulls();
    let offered = cartalith_civ::river_stretch::jp_river_offers(&w, &pts, &plan, &none, &|_, _| 1.0, None, None, &base, &hulls);
    assert!(offered.as_ref().is_ok_and(|o| !o.is_empty()), "{offered:?}");
    let mut mule = walking();
    mule.party.mule = 1;
    let base = cartalith_civ::jp_plan(&w, &pts, &mule, &none, &|_, _| 1.0).unwrap();
    let refused = cartalith_civ::river_stretch::jp_river_offers(&w, &pts, &mule, &none, &|_, _| 1.0, None, None, &base, &hulls);
    assert_eq!(refused, Err(cartalith_civ::river_stretch::NOTE_ANIMALS_ABOARD));
}

/// Protects the minimum length: a stretch shorter than `min_km` along the route is
/// not offered (the whole stretch is 31 x 12 = 372 route km here).
#[test]
fn a_stretch_shorter_than_the_minimum_is_not_offered() {
    let r = river(4);
    let road = HashMap::new();
    assert!(jp_river_candidates(&r.world(&road), &east(), 4.0, 400.0).is_empty());
    assert_eq!(jp_river_candidates(&r.world(&road), &east(), 4.0, 372.0).len(), 1);
}

/// The accepted leg for the fixture's one candidate, with `vessel`.
fn leg(r: &Rv, vessel: Option<&str>) -> JpRiverLeg {
    let road = HashMap::new();
    let pts = east();
    let c = jp_river_candidates(&r.world(&road), &pts, 4.0, 24.0).remove(0);
    JpRiverLeg { cells: c.cells, entry: pts[c.entry_k], exit: pts[c.exit_k], vessel: vessel.map(str::to_string) }
}

/// Protects the leg itself: a walking journey with the leg accepted plans the
/// channel as a river stage on its leg (whatever the transport), sails the leg's
/// own hull, and travels fewer days than walking the whole way; with no leg the plan
/// has no river stage at all.
#[test]
fn an_accepted_leg_is_sailed_whatever_the_transport() {
    let r = river(4);
    let road = HashMap::new();
    let w = r.world(&road);
    let walk = cartalith_civ::jp_plan(&w, &east(), &walking(), &HashMap::new(), &|_, _| 1.0).expect("plans");
    assert!(walk.stages.iter().all(|s| s.cat == "land" && s.river_leg.is_none()));
    assert!(walk.river_legs_applied.is_empty() && walk.route_pts.is_none());
    let mut p = walking();
    // A hull other than the plan's own default (Keelboat), so the leg's hull is
    // seen to reach the stage.
    p.river_legs = vec![leg(&r, Some("River Galley"))];
    let sail = cartalith_civ::jp_plan(&w, &east(), &p, &HashMap::new(), &|_, _| 1.0).expect("plans");
    let river: Vec<_> = sail.stages.iter().enumerate().filter(|(_, s)| s.river_leg == Some(0)).collect();
    assert_eq!(river.len(), 1, "{:?}", sail.stages.iter().map(|s| (&s.cat, s.river_leg)).collect::<Vec<_>>());
    let (i, st) = river[0];
    assert_eq!(st.cat, "river");
    assert_eq!(sail.results[i].eff.vessel, "River Galley");
    assert!(sail.river_legs_dropped.is_empty());
    // Travel days: the walk has no total (it outruns the default party's food
    // with no settlement on the way), the sail does.
    assert!(walk.total_days.is_none() && sail.total_days.is_some());
    let (a, b) = (walk.days, sail.days);
    assert!(b < a, "sailing {b} d vs walking {a} d");
}

/// Protects the drop rule: a leg whose channel is no longer navigable (the
/// world's rivers changed) is dropped with its reason, never moved onto another
/// channel, and the journey plans exactly as with no leg.
#[test]
fn a_leg_on_a_changed_river_is_dropped_not_moved() {
    let r = river(4);
    let l = leg(&r, Some("Keelboat"));
    let changed = river(2);
    let road = HashMap::new();
    let w = changed.world(&road);
    let s = jp_apply_river_legs(&w, &east(), &[l.clone()]);
    assert!(s.applied.is_empty());
    assert_eq!(s.dropped.len(), 1);
    assert_eq!(s.dropped[0].key, l.key());
    assert!(s.dropped[0].reason.contains("no longer navigable"), "{}", s.dropped[0].reason);
    assert_eq!(s.pts, east());
    let mut p = walking();
    p.river_legs = vec![l];
    let with = cartalith_civ::jp_plan(&w, &east(), &p, &HashMap::new(), &|_, _| 1.0).unwrap();
    let without = cartalith_civ::jp_plan(&w, &east(), &walking(), &HashMap::new(), &|_, _| 1.0).unwrap();
    assert_eq!(with.total_days, without.total_days);
    assert_eq!(with.stages.len(), without.stages.len());
    assert_eq!(with.river_legs_dropped.len(), 1);
}

/// Protects the route check: a leg whose entry the route no longer passes is
/// dropped with that reason.
#[test]
fn a_leg_off_the_route_is_dropped() {
    let r = river(4);
    let mut l = leg(&r, None);
    l.entry = (100.0, 100.0);
    let road = HashMap::new();
    let s = jp_apply_river_legs(&r.world(&road), &east(), &[l]);
    assert!(s.applied.is_empty());
    assert!(s.dropped[0].reason.contains("no longer passes"), "{}", s.dropped[0].reason);
}

/// Protects the splice: the spliced route is the route to the entry point, every
/// channel cell's centre, then the route from the exit point, with each spliced
/// point marked by its leg.
#[test]
fn the_splice_puts_the_channel_between_entry_and_exit() {
    let r = river(4);
    let l = leg(&r, None);
    let road = HashMap::new();
    let s = jp_apply_river_legs(&r.world(&road), &east(), &[l]);
    // 1 route point up to the entry, 32 channel cells, 1 route point from the exit.
    assert_eq!(s.pts.len(), 34);
    assert_eq!(s.pts[0], (4.0, 5.0));
    assert_eq!(s.pts[1], (4.0, ROW as f64));
    assert_eq!(s.pts[32], (35.0, ROW as f64));
    assert_eq!(s.pts[33], (35.0, 5.0));
    assert_eq!(s.leg_of.iter().filter(|l| **l == Some(0)).count(), 32);
    assert_eq!((s.leg_of[0], s.leg_of[33]), (None, None));
}

/// Protects the vessel ranking on a real leg: hulls that cannot work a river are
/// reported with the planner's own refusal, the rest are costed, and the
/// recommendation is the cheapest hull within the margin of the fastest (an
/// independent check over the ranks, not the function's own loop).
#[test]
fn hulls_are_ranked_on_the_legs_own_stages() {
    let r = river(4);
    let road = HashMap::new();
    let w = r.world(&road);
    let mut p = walking();
    p.river_legs = vec![leg(&r, Some("Keelboat"))];
    let sail = cartalith_civ::jp_plan(&w, &east(), &p, &HashMap::new(), &|_, _| 1.0).unwrap();
    let stages: Vec<_> = sail
        .stages
        .iter()
        .enumerate()
        .filter(|(_, s)| s.river_leg == Some(0))
        .map(|(i, s)| (s.to_stage(1.0), sail.results[i].eff.clone()))
        .collect();
    let names = jp_builtin_hulls();
    let ranks = jp_rank_river_vessels(&stages, &names, None);
    assert_eq!(ranks.len(), names.len());
    let fits: Vec<_> = ranks.iter().filter_map(|r| r.fit.as_ref().ok().map(|f| (r.name.as_str(), *f))).collect();
    assert!(fits.iter().any(|f| f.0 == "Keelboat"), "{ranks:?}");
    assert!(ranks.iter().any(|r| r.fit.as_ref().is_err_and(|e| e.contains("cannot operate on rivers"))), "{ranks:?}");
    let fastest = fits.iter().map(|f| f.1.0).fold(f64::INFINITY, f64::min);
    let want = fits
        .iter()
        .filter(|f| f.1.0 <= fastest * 1.10)
        .min_by(|a, b| a.1.1.total_cmp(&b.1.1))
        .map(|f| f.0)
        .unwrap();
    assert_eq!(jp_recommend(&ranks).map(|r| r.name.as_str()), Some(want));
}
