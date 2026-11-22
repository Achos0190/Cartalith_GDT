//! PS-1's direction field, type rule and ground gate ([`crate::field`]). Each
//! test says what it protects. Expected values are literals (a bearing, a
//! weight worked by hand, a cap from Ruling GC), never read back from the
//! constant under test.

use super::*;
use crate::generate::{GenOpts, Town, generate, hash_model};
use crate::site::{SiteOpts, build_site};
use std::f64::consts::PI;

/// A field of one straight source along `a -> b`, weight 1, reach 80 m, with
/// no core (strength is the blend alone past distance 0).
fn one(a: Vec2, b: Vec2) -> DirectionField {
    DirectionField { sources: vec![Source { kind: SourceKind::Road, line: vec![a, b], weight: 1.0, reach_m: 80.0 }], core_m: 0.0, ramp_m: 0.0 }
}

/// The blend takes a source's own bearing, as an axis: a road running east
/// gives axis 0 on it at full strength, the same road drawn west gives the
/// same axis (opposite bearings are one grain), and a 45-degree road gives
/// pi/4.
#[test]
fn the_field_takes_a_sources_bearing_as_an_axis() {
    let g = one(Vec2::new(0.0, 0.0), Vec2::new(400.0, 0.0)).at(Vec2::new(200.0, 0.0), 500.0);
    assert!(g.axis.abs() < 1e-12 && (g.strength - 1.0).abs() < 1e-12, "{g:?}");
    let g = one(Vec2::new(400.0, 0.0), Vec2::new(0.0, 0.0)).at(Vec2::new(200.0, 0.0), 500.0);
    assert!((g.axis.abs() < 1e-9 || (g.axis.abs() - PI).abs() < 1e-9) && (g.strength - 1.0).abs() < 1e-12, "reversed road: {g:?}");
    let g = one(Vec2::new(0.0, 0.0), Vec2::new(300.0, 300.0)).at(Vec2::new(100.0, 100.0), 500.0);
    assert!((g.axis - PI / 4.0).abs() < 1e-12, "{g:?}");
}

/// Strength falls with distance as `1 / (1 + (d / reach)^2)`: 0.5 at the
/// reach (80 m), 0.2 at twice it.
#[test]
fn strength_falls_with_distance_from_the_source() {
    let f = one(Vec2::new(-1000.0, 0.0), Vec2::new(1000.0, 0.0));
    assert!((f.at(Vec2::new(0.0, 80.0), 500.0).strength - 0.5).abs() < 1e-12);
    assert!((f.at(Vec2::new(0.0, 160.0), 500.0).strength - 0.2).abs() < 1e-12);
}

/// Two equal sources crossing at right angles cancel (no grain where two
/// grains meet square); two parallel ones reinforce, capped at 1.
#[test]
fn crossing_sources_cancel_and_parallel_ones_agree() {
    let mk = |b2: Vec2| DirectionField {
        sources: vec![
            Source { kind: SourceKind::Road, line: vec![Vec2::new(-500.0, 0.0), Vec2::new(500.0, 0.0)], weight: 1.0, reach_m: 80.0 },
            Source { kind: SourceKind::Road, line: vec![Vec2::new(0.0, -500.0), b2], weight: 1.0, reach_m: 80.0 },
        ],
        core_m: 0.0,
        ramp_m: 0.0,
    };
    let g = mk(Vec2::new(0.0, 500.0)).at(Vec2::new(0.0, 0.0), 500.0);
    assert!(g.strength < 1e-12, "square crossing: {g:?}");
    let par = DirectionField {
        sources: vec![
            Source { kind: SourceKind::Road, line: vec![Vec2::new(-500.0, 0.0), Vec2::new(500.0, 0.0)], weight: 1.0, reach_m: 80.0 },
            Source { kind: SourceKind::Road, line: vec![Vec2::new(-500.0, 40.0), Vec2::new(500.0, 40.0)], weight: 1.0, reach_m: 80.0 },
        ],
        core_m: 0.0,
        ramp_m: 0.0,
    };
    let g = par.at(Vec2::new(0.0, 20.0), 500.0);
    assert!(g.axis.abs() < 1e-12 && (g.strength - 1.0).abs() < 1e-12, "two parallel roads, total weight capped at 1: {g:?}");
}

/// The organic core (Q3): no grain inside `core_m`, half at the ramp's
/// middle, full past its end.
#[test]
fn the_core_has_no_grain_and_the_ramp_is_linear() {
    let mut f = one(Vec2::new(-1000.0, 0.0), Vec2::new(1000.0, 0.0));
    f.core_m = 200.0;
    f.ramp_m = 60.0;
    let p = Vec2::new(0.0, 0.0);
    assert_eq!(f.at(p, 150.0).strength, 0.0);
    assert!((f.at(p, 230.0).strength - 0.5).abs() < 1e-12);
    assert!((f.at(p, 400.0).strength - 1.0).abs() < 1e-12);
}

/// No source, no grain: strength 0, never a plausible default.
#[test]
fn a_field_with_no_source_has_no_grain() {
    let f = DirectionField { sources: Vec::new(), core_m: 0.0, ramp_m: 0.0 };
    assert_eq!(f.at(Vec2::new(5.0, 5.0), 500.0), Grain { axis: 0.0, strength: 0.0 });
}

/// The type rule's order (scope section 4): the radial plan never takes a
/// grain; river-front before market-street (the stricter cap wins a tie);
/// market-street before organic-core; no road at all is no grain.
#[test]
fn the_type_rule_in_the_scopes_order() {
    assert_eq!(town_type(false, true, true, 3), TownType::NoGrain);
    assert_eq!(town_type(true, true, true, 3), TownType::RiverFront);
    assert_eq!(town_type(true, false, true, 3), TownType::MarketStreet);
    assert_eq!(town_type(true, false, false, 1), TownType::OrganicCore);
    assert_eq!(town_type(true, false, false, 0), TownType::NoGrain);
}

/// Ruling GC's split of FS-2 (and Q12 (a)'s delta): market-street 0.35 per
/// town and 0.30 delta; river-front and organic-core 0.25 median, 0.45 p90,
/// 0.20 delta; no grain no cap.
#[test]
fn each_type_carries_ruling_gcs_caps() {
    assert_eq!(TownType::MarketStreet.fs2_caps(), Some(Fs2Caps { phi_per_town: Some(0.35), phi_median: None, phi_p90: None, phi_delta: 0.30 }));
    for t in [TownType::RiverFront, TownType::OrganicCore] {
        assert_eq!(t.fs2_caps(), Some(Fs2Caps { phi_per_town: None, phi_median: Some(0.25), phi_p90: Some(0.45), phi_delta: 0.20 }));
    }
    assert_eq!(TownType::NoGrain.fs2_caps(), None);
    assert_eq!([TownType::MarketStreet.key(), TownType::RiverFront.key(), TownType::OrganicCore.key(), TownType::NoGrain.key()], ["market_street", "river_front", "organic_core", "none"]);
}

/// The core grows with age (Q3): 0.38 of the reach at 300 years, 0.50 at
/// 1 000, 0.33 at the floor.
#[test]
fn the_core_grows_with_the_towns_age() {
    assert!((core_radius(1000.0, 300.0) - 380.0).abs() < 1e-9);
    assert!((core_radius(1000.0, 1000.0) - 500.0).abs() < 1e-9);
    assert!((core_radius(1000.0, 0.0) - 330.0).abs() < 1e-9);
}

/// Section 9.3's two requirements, worked by hand: 1.4 / 1.25 and 1.68 / 1.4.
#[test]
fn the_footprint_requirements_are_section_9_3s() {
    assert!((F_REQ_MEDIAN - 1.12).abs() < 1e-12);
    assert!((F_REQ_P90 - 1.2).abs() < 1e-12);
}

/// The approach roads are read on a circle round the market: three primaries
/// from a market at (500, 500), east, west (slightly bent) and north, give
/// three bearings, and the east-west pair is a road through the market (about
/// 177 degrees apart); east and north alone are not (90 degrees).
#[test]
fn approach_roads_and_a_through_road() {
    let mut g = Graph::new();
    g.add_street(500.0, 500.0, 900.0, 500.0, "primary", 6.0, 0, "t");
    g.add_street(500.0, 500.0, 100.0, 520.0, "primary", 6.0, 0, "t");
    g.add_street(500.0, 500.0, 500.0, 100.0, "primary", 6.0, 0, "t");
    g.add_street(500.0, 500.0, 520.0, 900.0, "street", 4.0, 0, "t");
    let m = Vec2::new(500.0, 500.0);
    let b = approach_bearings(&g, m, 100.0);
    assert_eq!(b.len(), 3, "{b:?}");
    assert!((b[0] + PI / 2.0).abs() < 1e-9 && b[1].abs() < 1e-9 && (b[2] - (PI - (20.0f64 / 400.0).atan())).abs() < 1e-9, "{b:?}");
    let (a, c) = through_pair(&b).expect("east-west runs through");
    assert!(a.abs() < 1e-9 && (c - b[2]).abs() < 1e-12);
    assert_eq!(through_pair(&[0.0, PI / 2.0]), None);
    assert_eq!(through_pair(&[0.0, 2.0]), None, "115 degrees is a bend, not a through road");
    assert!(through_pair(&[0.0, 2.7]).is_some(), "155 degrees runs through");
}

/// Two crossings of the probe circle within 20 degrees are one road,
/// including across the wrap at pi.
#[test]
fn near_crossings_merge_into_one_road() {
    let mut g = Graph::new();
    g.add_street(500.0, 500.0, 100.0, 510.0, "primary", 6.0, 0, "t");
    g.add_street(500.0, 500.0, 100.0, 490.0, "primary", 6.0, 0, "t");
    let b = approach_bearings(&g, Vec2::new(500.0, 500.0), 100.0);
    assert_eq!(b.len(), 1, "two roads leaving west 3 degrees apart are one approach: {b:?}");
}

/// The town-type and ground pass on generated towns: with the switch on, a
/// town with a grain grows a different layout (PS-2, `crate::growth`) and has
/// a plan with a type and a ground figure; a town with no grain keeps the
/// switch-off layout (the reference's `hash_model` and the building count);
/// off there is no plan. A village (800) and a radial town carry the same
/// layout both ways, the village no plan and the radial town no grain.
#[test]
fn only_a_town_with_a_grain_changes_layout() {
    let t = |pop: f64, culture: Option<&str>, on: bool| -> Town {
        generate(4242, &GenOpts { pop: Some(pop), culture: culture.map(str::to_string), parallel_streets: on, ..GenOpts::default() })
    };
    for (pop, culture) in [(6000.0, None), (2500.0, None), (800.0, None), (6000.0, Some("venus"))] {
        let (off, on) = (t(pop, culture, false), t(pop, culture, true));
        let grain = on.parallel.as_ref().is_some_and(|p| p.field.is_some());
        assert_eq!(hash_model(&off) == hash_model(&on), !grain, "pop {pop} {culture:?}");
        assert!(grain || pop != 6000.0 || culture.is_some(), "the 6 000 organic town has a grain, so the moved branch is reached");
        if !grain {
            assert_eq!((off.buildings.len(), off.pop), (on.buildings.len(), on.pop));
        }
        assert!(off.parallel.is_none());
        match (pop, culture) {
            (800.0, _) => assert!(on.parallel.is_none(), "a village has no plan"),
            (_, Some(_)) => assert_eq!(on.parallel.as_ref().map(|p| p.town_type), Some(TownType::NoGrain), "the radial plan never takes a grain"),
            _ => {
                let p = on.parallel.as_ref().expect("a town of 1 500 and up has a plan");
                assert!(p.ground.bank_m2 > 0.0 && p.settled_m2 > 0.0, "{:?} {}", p.ground, p.settled_m2);
                assert!(p.ground_ratio().is_some());
            }
        }
    }
}

/// The ground gate on the synthetic river site: the far bank counts only for
/// a both-banks town, and then no more than the market's bank
/// (`FAR_BANK_MAX_RATIO`); a building at the market is settled in the region
/// (one building, nine 20 m cells, 3 600 m2) and one 2 km away is not.
#[test]
fn the_ground_gate_counts_the_far_bank_only_for_both_banks() {
    let site = build_site(4242, 1700.0, 1250.0, "river", SiteOpts::default());
    let m = Vec2::new(850.0, 625.0);
    let one_bank = ground_gate(&site, m, 500.0, false);
    assert!(one_bank.mkt_side.is_some() && one_bank.far_end.is_none());
    assert_eq!((one_bank.far_raw_m2, one_bank.far_m2), (0.0, 0.0));
    assert!(one_bank.bank_m2 > 0.0 && one_bank.bank_m2 < PI * 500.0 * 500.0);
    let both = ground_gate(&site, m, 500.0, true);
    assert_eq!(both.bank_m2, one_bank.bank_m2, "the market's bank does not depend on the far bank");
    assert!(both.far_m2 <= both.bank_m2 && both.far_m2 <= both.far_raw_m2);
    let sq = |c: Vec2| vec![c + Vec2::new(-3.0, -3.0), c + Vec2::new(3.0, -3.0), c + Vec2::new(3.0, 3.0), c + Vec2::new(-3.0, 3.0)];
    let near = sq(m + Vec2::new(5.0, 5.0));
    let far = sq(m + Vec2::new(2000.0, 0.0));
    assert_eq!(settled_in_region(&one_bank, &site, &[near.as_slice()]), 3600.0);
    assert_eq!(settled_in_region(&one_bank, &site, &[far.as_slice()]), 0.0);
}

/// A plan's ratio and fit (section 9.3): `g = growable / settled`, a fit at
/// `g >= f_req`; nothing settled is no ratio and no fit.
#[test]
fn the_ratio_and_the_fit() {
    let ground = Ground { market: Vec2::new(0.0, 0.0), max_rf: 500.0, mkt_side: None, far_end: None, bank_m2: 90_000.0, far_raw_m2: 30_000.0, far_m2: 22_000.0 };
    let mut p = ParallelPlan { town_type: TownType::OrganicCore, approaches: vec![0.0], field: None, ground, settled_m2: 100_000.0, settled_bank_m2: 90_000.0, samples: Vec::new() };
    assert_eq!(p.ground_ratio(), Some(1.12));
    assert!(p.fits(1.12) && !p.fits(1.2));
    p.settled_m2 = 0.0;
    assert_eq!(p.ground_ratio(), None);
    assert!(!p.fits(0.0));
}

/// Total weight is capped at 1 before it scales the coherence, not after:
/// two equal roads 60 degrees apart through the same point have coherence
/// cos 60 = 0.5 and total weight 2, so strength 0.5 (uncapped it would clamp
/// to 1).
#[test]
fn the_weight_cap_applies_before_the_coherence() {
    let f = DirectionField {
        sources: vec![
            Source { kind: SourceKind::Road, line: vec![Vec2::new(-500.0, 0.0), Vec2::new(500.0, 0.0)], weight: 1.0, reach_m: 80.0 },
            Source { kind: SourceKind::Road, line: vec![Vec2::new(-250.0, -433.0127018922193), Vec2::new(250.0, 433.0127018922193)], weight: 1.0, reach_m: 80.0 },
        ],
        core_m: 0.0,
        ramp_m: 0.0,
    };
    let g = f.at(Vec2::new(0.0, 0.0), 500.0);
    assert!((g.strength - 0.5).abs() < 1e-9 && (g.axis - PI / 6.0).abs() < 1e-9, "{g:?}");
}

/// The river-front test (scope section 4): a real river and a market square
/// within two street widths (10.8 m) of the bank, measured from the square's
/// outline where there is one; a market 300 m off is not; a synthetic river
/// (no host water, `real_river` false) never is.
#[test]
fn river_front_is_a_real_river_with_the_market_at_the_bank() {
    let mut site = build_site(4242, 1700.0, 1250.0, "river", SiteOpts::default());
    assert!(site.river.len() >= 2, "premise: the synthetic river site has a centreline");
    let (a, b) = (site.river[site.river.len() / 2], site.river[site.river.len() / 2 + 1]);
    let n = (b - a).rot90().norm();
    let mid = a.lerp(b, 0.5);
    let at_bank = mid + n * (site.river_w / 2.0 + 8.0);
    let off = mid + n * (site.river_w / 2.0 + 300.0);
    assert!(!is_river_front(&site, at_bank, None), "a synthetic river is not a real one");
    site.real_river = true;
    assert!(is_river_front(&site, at_bank, None));
    assert!(!is_river_front(&site, mid + n * (site.river_w / 2.0 + 12.0), None), "12 m from the bank is past two street widths");
    assert!(!is_river_front(&site, off, None));
    let square = [at_bank, at_bank + n * 60.0, at_bank + n * 60.0 + (b - a).norm() * 40.0];
    assert!(is_river_front(&site, off, Some(&square)), "the square's outline reaches the bank");
    // A town stretched along its river (Ruling FN) is river-front wherever its market stands.
    let f = crate::stretch::StretchFrame::new(&site.river.clone(), off, crate::stretch::StretchPlan { ahead_m: 900.0, behind_m: 900.0, across_m: 400.0 }, 600.0);
    assert!(f.is_some(), "premise: a frame is built");
    site.set_stretch(f);
    assert!(is_river_front(&site, off, None), "stretched along its river");
}

/// One grain under 5 000, the roads too from 5 000 (scope section 4, era and
/// size): a small organic-core town has one straight source through the
/// market; at 6 000 it has one per primary edge besides; a market-street
/// town's spine is a bent line through the market (three points).
#[test]
fn one_grain_under_5000_and_the_roads_above() {
    let site = build_site(4242, 1700.0, 1250.0, "inland", SiteOpts::default());
    let mut g = Graph::new();
    g.add_street(500.0, 500.0, 900.0, 500.0, "primary", 6.0, 0, "t");
    g.add_street(500.0, 500.0, 500.0, 100.0, "primary", 6.0, 0, "t");
    let m = Vec2::new(500.0, 500.0);
    let b = approach_bearings(&g, m, 100.0);
    let small = sources_for(TownType::OrganicCore, 2500.0, &site, &g, m, None, &b, 600.0).unwrap();
    assert_eq!(small.len(), 1);
    assert_eq!(small[0].line.len(), 2);
    let big = sources_for(TownType::OrganicCore, 6000.0, &site, &g, m, None, &b, 600.0).unwrap();
    assert_eq!(big.len(), 2, "two primary edges, no single grain");
    assert!(big.iter().all(|s| s.kind == SourceKind::Road && s.reach_m == 80.0));
    let spine = sources_for(TownType::MarketStreet, 2500.0, &site, &g, m, Some((0.0, PI)), &b, 600.0).unwrap();
    assert_eq!((spine.len(), spine[0].line.len(), spine[0].kind, spine[0].weight), (1, 3, SourceKind::Spine, 0.7));
    assert_eq!(sources_for(TownType::MarketStreet, 6000.0, &site, &g, m, Some((0.0, PI)), &b, 600.0).unwrap().len(), 3);
    assert!(sources_for(TownType::NoGrain, 6000.0, &site, &g, m, None, &b, 600.0).is_none());
}

/// Two crossings either side of the bearing wrap (179 and -179 degrees) are
/// one road, not two, so a road leaving due west is not read twice.
#[test]
fn crossings_across_the_bearing_wrap_merge() {
    let mut g = Graph::new();
    g.add_street(500.0, 500.0, 100.0, 507.0, "primary", 6.0, 0, "t");
    g.add_street(500.0, 500.0, 100.0, 493.0, "primary", 6.0, 0, "t");
    g.add_street(500.0, 500.0, 900.0, 500.0, "primary", 6.0, 0, "t");
    let b = approach_bearings(&g, Vec2::new(500.0, 500.0), 100.0);
    assert_eq!(b.len(), 2, "west (wrapping) and east: {b:?}");
}

/// Protects `axis_off`, `block_corners` and `rectangularity` (PS-2's closed-block and grain tests, moved from the census):
/// literals. A bearing 0.1 rad off an axis, or off its cross-direction, is 0.1 from the grain, and 45 degrees is the most
/// (pi / 4); a 100 m square has four corners and rectangularity 1.0, a right triangle three corners and 0.5, a pentagon
/// (a square with its corner cut) five.
#[test]
fn the_grain_and_block_shape_measures_read_literals() {
    use std::f64::consts::{FRAC_PI_2, FRAC_PI_4};
    for (a, want) in [(0.0, 0.0), (0.1, 0.1), (-0.2, 0.2), (FRAC_PI_2 + 0.1, 0.1), (FRAC_PI_4, FRAC_PI_4), (3.0 * FRAC_PI_2 - 0.3, 0.3)] {
        assert!((axis_off(a, 0.0) - want).abs() < 1e-12, "{a}: {}", axis_off(a, 0.0));
    }
    assert!((axis_off(1.0 + 0.25, 1.0) - 0.25).abs() < 1e-12, "the axis offsets the bearing");
    let poly = |v: &[(f64, f64)]| -> Vec<Vec2> { v.iter().map(|&(x, y)| Vec2::new(x, y)).collect() };
    let square = poly(&[(0.0, 0.0), (100.0, 0.0), (100.0, 100.0), (0.0, 100.0)]);
    let tri = poly(&[(0.0, 0.0), (100.0, 0.0), (0.0, 100.0)]);
    let pent = poly(&[(0.0, 0.0), (100.0, 0.0), (100.0, 60.0), (60.0, 100.0), (0.0, 100.0)]);
    assert_eq!((block_corners(&square), block_corners(&tri), block_corners(&pent)), (4, 3, 5));
    assert!((rectangularity(&square) - 1.0).abs() < 1e-9);
    assert!((rectangularity(&tri) - 0.5).abs() < 1e-9);
    assert!(rectangularity(&[]) == 0.0, "a degenerate outline reads 0");
}

/// Protects PS-3's measure of scope 9.2's access rule (`rear_served`, `rear_access_m`): a 10 m wide lot whose front
/// is on y = 0 and whose rear edge is at y = 40 (40 m deep, over the 30 m line). A street along y = 70 is 30 m behind
/// it: served. One along y = 71, 31 m behind: not served. A lot 30 m deep (rear at y = 30) is not over the line, so it
/// has no answer whatever streets there are, and neither has a lot that is not four-cornered; one 30.5 m deep is over it. The distance itself is
/// read from the rear edge's middle: 25 m to a street on y = 65.
#[test]
fn a_deep_lot_is_served_from_behind_only_by_a_street_within_30_m_of_its_rear_edge() {
    let v = |x: f64, y: f64| Vec2::new(x, y);
    let street = |y: f64| vec![(v(-50.0, 0.0), v(60.0, 0.0)), (v(-50.0, y), v(60.0, y))];
    let lot = [v(0.0, 0.0), v(10.0, 0.0), v(10.0, 40.0), v(0.0, 40.0)];
    assert_eq!(rear_served(&lot, &street(70.0)), Some(true), "a street 30 m behind serves it");
    assert_eq!(rear_served(&lot, &street(71.0)), Some(false), "one 31 m behind does not");
    assert_eq!(rear_served(&lot, &street(0.5)[..1]), Some(false), "its own front street, 40 m off, does not");
    let shallow = [v(0.0, 0.0), v(10.0, 0.0), v(10.0, 30.0), v(0.0, 30.0)];
    assert_eq!(rear_served(&shallow, &street(31.0)), None, "a lot 30 m deep is not over the line");
    assert_eq!(rear_served(&lot[..3], &street(41.0)), None, "a three-cornered lot has no answer");
    let just = [v(0.0, 0.0), v(10.0, 0.0), v(10.0, 30.5), v(0.0, 30.5)];
    assert_eq!(rear_served(&just, &street(50.0)), Some(true), "a lot 30.5 m deep is over the line");
    assert!((rear_access_m(&lot, &street(65.0)).unwrap() - 25.0).abs() < 1e-12);
    assert_eq!(rear_access_m(&lot, &[]), None, "no street, no distance");
}
