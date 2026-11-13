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
/// Wolman's meander sinuosity), the recommendation margin and the least saving,
/// so a change to any of them is deliberate.
#[test]
fn eta_min_is_one_over_the_meander_sinuosity() {
    assert!((ETA_MIN - 0.666_666_666_666_666_6).abs() < 1e-15);
    assert!((RECOMMEND_MARGIN - 0.10).abs() < 1e-15);
    assert!((cartalith_civ::river_stretch::MIN_SAVING_DAYS - 0.05).abs() < 1e-15);
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
    assert!(offered.as_ref().is_ok_and(|o| !o.offers.is_empty() && o.unmeasured == 0), "{offered:?}");
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

/// The fixture with two terrains along the route (biome 5 west of x = 25, biome
/// 10 from it) and the channel navigable only over x = 2..=20 (order 0 beyond),
/// so the walked route has at least two stages and a leg can cut the first
/// without reaching the second. Test-local choices.
fn two_terrains() -> Rv {
    let mut r = river(4);
    for y in 0..H {
        for x in 25..W {
            r.biome[y * W + x] = 10;
        }
    }
    for x in 21..=37 {
        r.order[ROW * W + x] = 0;
    }
    r
}

/// Protects the hops (verifier finding, 2026-11-13): the overland steps from the
/// route onto the river and back are walked, never sailed. The route runs along
/// row 1, seven cells north of the channel, so each hop is 7 x 12 = 84 km; the
/// leg spans the whole route, so the plan's land km are exactly the two hops and
/// its river km exactly the channel (31 x 12 = 372 km). Before the fix the
/// entry hop was sailed (river km 456).
#[test]
fn the_hops_onto_and_off_the_river_are_walked() {
    let r = river(4);
    let road = HashMap::new();
    let w = r.world(&road);
    let north: Vec<(f64, f64)> = (4..=35).map(|x| (x as f64, 1.0)).collect();
    let c = jp_river_candidates(&w, &north, 8.0, 24.0).remove(0);
    assert_eq!((c.entry_k, c.exit_k), (0, 31), "premise");
    let mut p = walking();
    p.river_legs = vec![JpRiverLeg { cells: c.cells.clone(), entry: north[0], exit: north[31], vessel: Some("Keelboat".into()) }];
    let sail = cartalith_civ::jp_plan(&w, &north, &p, &HashMap::new(), &|_, _| 1.0).expect("plans");
    assert_eq!(sail.river_legs_applied.len(), 1);
    let river_km: f64 = sail.stages.iter().filter(|s| s.river_leg.is_some()).map(|s| s.km).sum();
    let land_km: f64 = sail.stages.iter().filter(|s| s.river_leg.is_none()).map(|s| s.km).sum();
    assert!((river_km - 372.0).abs() < 1e-9, "river {river_km}");
    assert!((land_km - 168.0).abs() < 1e-9, "land {land_km}");
    // The walked pieces reach the river: the first ends at the leg's first
    // channel point, the last starts at its last one.
    let rp = sail.route_pts.as_ref().expect("a leg was applied");
    let first = &sail.stages[0];
    let last = sail.stages.last().unwrap();
    assert_eq!((first.river_leg, rp[first.i1]), (None, (4.0, ROW as f64)));
    assert_eq!((last.river_leg, rp[last.i0]), (None, (35.0, ROW as f64)));
}

/// Protects the cut (verifier finding, 2026-11-13: a 20 km leg re-partitioned
/// unrelated land and was offered as saving days the river did not buy): a stage
/// of the route as walked that no leg touches is planned exactly as walked
/// (terrain, condition, infrastructure, km, climb), every stage is either a
/// leg's or a piece of a walked stage, and a cut piece keeps its walked stage's
/// terrain and loses only the km the leg replaces.
#[test]
fn a_leg_leaves_the_stages_it_does_not_touch_as_walked() {
    let r = two_terrains();
    let road = HashMap::new();
    let w = r.world(&road);
    let walked = cartalith_civ::jp_plan(&w, &east(), &walking(), &HashMap::new(), &|_, _| 1.0).expect("plans");
    assert!(walked.stages.len() >= 2, "premise: two terrains, {:?}", walked.stages.iter().map(|s| &s.terrain).collect::<Vec<_>>());
    let lastw = walked.stages.len() - 1;
    assert!(walked.stages[lastw].i0 >= 17, "premise: the last walked stage starts past the river's end");
    let c = jp_river_candidates(&w, &east(), 4.0, 24.0).remove(0);
    assert_eq!((c.entry_k, c.exit_k), (0, 16), "premise: the channel ends at x = 20");
    let mut p = walking();
    p.river_legs = vec![JpRiverLeg { cells: c.cells.clone(), entry: east()[0], exit: east()[16], vessel: Some("Keelboat".into()) }];
    let sail = cartalith_civ::jp_plan(&w, &east(), &p, &HashMap::new(), &|_, _| 1.0).expect("plans");
    assert_eq!(sail.walked_stages.as_deref(), Some(&walked.stages[..]));
    assert_eq!(sail.override_stages(), &walked.stages[..]);
    for s in &sail.stages {
        assert!(s.river_leg.is_some() != s.base.is_some(), "a stage is a leg's or a walked stage's: {s:?}");
    }
    let tail: Vec<_> = sail.stages.iter().filter(|s| s.base == Some(lastw)).collect();
    assert_eq!(tail.len(), 1);
    let (a, b) = (tail[0], &walked.stages[lastw]);
    assert_eq!((&a.terrain, &a.route_cond, &a.infra, a.km, a.gain, a.loss), (&b.terrain, &b.route_cond, &b.infra, b.km, b.gain, b.loss));
    // The leg's 16 channel steps replace 16 route steps of 12 km; the hops are
    // three cells (36 km) each way.
    let land_km: f64 = sail.stages.iter().filter(|s| s.river_leg.is_none()).map(|s| s.km).sum();
    let walked_km: f64 = walked.stages.iter().map(|s| s.km).sum();
    assert!((land_km - (walked_km - 16.0 * 12.0 + 2.0 * 36.0)).abs() < 1e-6, "land {land_km} walked {walked_km}");
    for s in sail.stages.iter().filter(|s| s.base.is_some()) {
        assert_eq!(s.terrain, walked.stages[s.base.unwrap()].terrain);
    }
}

/// Protects the override keys (verifier finding, 2026-11-13): a per-stage
/// override is keyed by the stage of the route as walked, so accepting a leg,
/// which inserts stages, applies it to the same ground and never to the leg's
/// stage that now holds the same number.
#[test]
fn an_override_stays_on_its_walked_stage_when_a_leg_is_accepted() {
    let r = two_terrains();
    let road = HashMap::new();
    let w = r.world(&road);
    let walked = cartalith_civ::jp_plan(&w, &east(), &walking(), &HashMap::new(), &|_, _| 1.0).expect("plans");
    let lastw = walked.stages.len() - 1;
    let c = jp_river_candidates(&w, &east(), 4.0, 24.0).remove(0);
    let mut p = walking();
    p.stage_overrides.insert(lastw, cartalith_civ::JpStageOverride { hours: Some(5.0), ..Default::default() });
    p.river_legs = vec![JpRiverLeg { cells: c.cells.clone(), entry: east()[c.entry_k], exit: east()[c.exit_k], vessel: Some("Keelboat".into()) }];
    let sail = cartalith_civ::jp_plan(&w, &east(), &p, &HashMap::new(), &|_, _| 1.0).expect("plans");
    for (s, res) in sail.stages.iter().zip(&sail.results) {
        let want = if s.base == Some(lastw) { 5.0 } else { 8.0 };
        assert_eq!(res.eff.hours, want, "{s:?}");
    }
    // Premise of the failure it guards: the spliced number `lastw` names a
    // different stage than the walked one.
    assert_ne!(sail.stages[lastw].base, Some(lastw));
}

/// Protects progress measured on the route's line (verifier finding,
/// 2026-11-13): a route drawn on sparse points (5 cells apart, as a route drawn
/// on a way's own points is) crossing a river at 53 degrees, past the 48 the
/// alignment rule allows, is not offered, though crediting each channel cell
/// with its nearest route point's arc (the old measure, computed here
/// independently) would have counted the crossing as running along the route.
#[test]
fn a_crossing_on_a_sparse_route_is_not_offered() {
    let n = W * H;
    let mut r = river(4);
    r.order = vec![0; n];
    r.recv = vec![-1; n];
    let row = 6usize;
    for x in 13..=19 {
        let i = row * W + x;
        r.order[i] = 4;
        r.flow[i] = 30.0;
        if x < 19 {
            r.recv[i] = (i + 1) as i32;
        }
    }
    let road = HashMap::new();
    let w = r.world(&road);
    let pts = vec![(11.0, 0.0), (14.0, 4.0), (17.0, 8.0), (20.0, 12.0), (22.0, 15.0)];
    // The old measure: nearest route point's arc for the first and last cells.
    let mut arc = vec![0.0f64];
    for k in 1..pts.len() {
        let (a, b): ((f64, f64), (f64, f64)) = (pts[k - 1], pts[k]);
        arc.push(arc[k - 1] + (b.0 - a.0).hypot(b.1 - a.1) * 12.0);
    }
    let nearest = |x: f64, y: f64| {
        (0..pts.len()).min_by(|&a, &b| (pts[a].0 - x).hypot(pts[a].1 - y).total_cmp(&(pts[b].0 - x).hypot(pts[b].1 - y))).unwrap()
    };
    let old_ds = arc[nearest(19.0, row as f64)] - arc[nearest(13.0, row as f64)];
    assert!(old_ds >= 24.0 && old_ds / 72.0 >= ETA_MIN, "premise: the old measure credits {old_ds} km over 72");
    assert!(jp_river_candidates(&w, &pts, 4.0, 24.0).is_empty());
}

/// Protects the party's path (verifier finding, 2026-11-13): a plan with a leg
/// is replayed on the route its stages index (`JourneyTimeline::points`), so a
/// party dated to a point on the leg is drawn on the river (row 8), not on the
/// committed route's point of the same number (row 1).
#[test]
fn a_journey_with_a_leg_is_replayed_on_the_spliced_route() {
    let r = river(4);
    let road = HashMap::new();
    let w = r.world(&road);
    let north: Vec<(f64, f64)> = (4..=35).map(|x| (x as f64, 1.0)).collect();
    let c = jp_river_candidates(&w, &north, 8.0, 24.0).remove(0);
    let mut p = walking();
    p.river_legs = vec![JpRiverLeg { cells: c.cells, entry: north[0], exit: north[31], vessel: Some("Keelboat".into()) }];
    let sail = cartalith_civ::jp_plan(&w, &north, &p, &HashMap::new(), &|_, _| 1.0).expect("plans");
    let t = cartalith_civ::journey_progress::JourneyTimeline::from_plan(&sail).expect("a timeline");
    assert_eq!(t.route, sail.route_pts);
    let pts = t.points(&north);
    assert_eq!(pts.len(), 34, "1 route point, 32 channel cells, 1 route point");
    let at = t.at(pts, t.elapsed_at_point(17));
    assert_eq!(at.point, (20.0, ROW as f64));
}

/// The fixture with only `cells` (in flow order) navigable, at order 4.
fn channel(cells: &[(usize, usize)]) -> Rv {
    let n = W * H;
    let mut r = river(4);
    r.order = vec![0; n];
    r.recv = vec![-1; n];
    for (k, &(x, y)) in cells.iter().enumerate() {
        let i = y * W + x;
        r.order[i] = 4;
        r.flow[i] = 30.0;
        if let Some(&(nx, ny)) = cells.get(k + 1) {
            r.recv[i] = (ny * W + nx) as i32;
        }
    }
    r
}

/// Protects the channel's own minimum (verifier finding, 2026-11-13: one-step
/// channels of 19.6 km were offered where the minimum was 39 km): a route that
/// doubles back (east along row 5, then west along row 8) has a one-cell
/// channel between its two arms whose progress along the route is hundreds of
/// km, yet the channel itself is one 12 km step, under the 24 km water gap, so it
/// would fold back into land and is not offered.
#[test]
fn a_channel_shorter_than_the_water_gap_is_not_offered_across_a_bend() {
    let r = channel(&[(10, 6), (10, 7)]);
    let road = HashMap::new();
    let w = r.world(&road);
    let mut pts: Vec<(f64, f64)> = (4..=20).map(|x| (x as f64, 5.0)).collect();
    pts.extend((6..=8).map(|y| (20.0, y as f64)));
    pts.extend((4..=19).rev().map(|x| (x as f64, 8.0)));
    assert!(jp_river_candidates(&w, &pts, 2.0, 24.0).is_empty());
    // Premise: a lower minimum would offer it, as a stretch cutting the bend.
    let c = jp_river_candidates(&w, &pts, 2.0, 12.0);
    assert_eq!(c.len(), 1, "{c:?}");
    assert!(c[0].route_km > 100.0 && (c[0].channel_km - 12.0).abs() < 1e-9, "{:?}", c[0]);
}

/// Protects the end points of a stretch inside long route segments: on a route
/// drawn on points six cells apart, a channel whose two ends are both nearest
/// one route point (x = 16) is still offered, leaving the route at the start of
/// its first end's segment (x = 10, point 1) and rejoining it at the end of its
/// last end's segment (x = 22, point 3), never dropped for rounding to one point.
#[test]
fn a_stretch_whose_ends_round_to_one_route_point_is_kept() {
    let r = channel(&[(14, 8), (15, 8), (16, 8), (17, 8), (18, 8)]);
    let road = HashMap::new();
    let w = r.world(&road);
    let pts: Vec<(f64, f64)> = (0..6).map(|k| (4.0 + 6.0 * k as f64, 5.0)).collect();
    let c = jp_river_candidates(&w, &pts, 4.0, 24.0);
    assert_eq!(c.len(), 1, "{c:?}");
    assert_eq!((c[0].entry_k, c[0].exit_k), (1, 3));
    assert!((c[0].route_km - 48.0).abs() < 1e-9 && (c[0].channel_km - 48.0).abs() < 1e-9, "{:?}", c[0]);
}
