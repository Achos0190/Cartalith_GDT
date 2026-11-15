//! Ruling EW item 1's acceptance bars for [`super::hug_wall`]: properties of
//! generated towns (there is no reference to diff against), each stated as a
//! literal bar measured on these exact towns on 2026-11-14, never as a constant
//! compared with itself. `wallgap` carries the measuring tape; its own unit
//! tests are at the end of this file.

use super::*;
use crate::generate::{GenOpts, Town, generate};
use crate::growth::ring_crossings;
use crate::wallgap::{WallGap, wall_gap};
use crate::wallside::WallBacking;

/// The measured set: the three site kinds the engine generates without a real
/// map, at populations from a small town to a city (`wallside`'s own five-town
/// set, plus two). `wall_generations` is on, as `run_layout` passes it.
const CASES: [(u32, f64, &str); 6] = [
    (42, 5000.0, "inland"),
    (1234, 12000.0, "inland"),
    (42, 5000.0, "river"),
    (99, 9000.0, "coast"),
    (7, 8000.0, "inland"),
    (2024, 14000.0, "river"),
];

fn town(seed: u32, pop: f64, site: &str, hug: bool) -> Town {
    generate(
        seed,
        &GenOpts {
            pop: Some(pop),
            site: Some(site.into()),
            wall_generations: true,
            walls: Some(true),
            wall_hug: hug,
            ..GenOpts::default()
        },
    )
}

fn gap_of(t: &Town) -> Option<WallGap> {
    let ring = t.wall.ring.as_deref()?;
    let polys: Vec<&[Vec2]> = t.buildings.iter().map(|b| b.poly.as_slice()).collect();
    wall_gap(ring, t.wall.land_arc.as_deref(), &polys)
}

fn refit(t: &Town) -> bool {
    matches!(t.wall_hug, Some(HugOutcome::Refit { .. }))
}

/// Every case, off and on, generated once per test run is cheap (under 2 s), so
/// each test regenerates rather than sharing a static.
fn pairs() -> Vec<((u32, f64, &'static str), Town, Town)> {
    CASES.iter().map(|&c| (c, town(c.0, c.1, c.2, false), town(c.0, c.1, c.2, true))).collect()
}

/// Protects: the switch's off position being the reference circuit and its
/// outcome being reported only when on. The byte-identity of the off position
/// is protected by the JS-captured goldens (`generate/tests/golden.rs`), whose
/// options set `wall_hug: false` and which pass unchanged.
#[test]
fn the_outcome_is_reported_only_when_the_switch_is_on() {
    for (c, off, on) in pairs() {
        assert!(off.wall_hug.is_none(), "{c:?}: off must report nothing");
        assert!(on.wall_hug.is_some(), "{c:?}: on must report what it did");
    }
}

/// Protects: the wall actually hugging the town, as a literal bar on the three
/// numbers [`crate::wallgap`] measures. Measured: on every refit case the
/// median wall-to-building gap is 4-7 m (bar 9), the 90th percentile 16-35 m
/// (bar 40), the free share 0.001-0.221 (bar 0.25); off, the same towns read a
/// median up to 46 m, a p90 up to 85 m and a free share up to 0.376. Five of
/// the six towns refit; the sixth, a 983-person coast town, keeps its circuit
/// (`NotSmaller`).
#[test]
fn a_refitted_wall_stands_a_few_metres_from_the_first_buildings() {
    let mut refits = 0;
    for (c, _off, on) in pairs() {
        if !refit(&on) {
            continue;
        }
        refits += 1;
        let g = gap_of(&on).expect("a walled town with buildings measures");
        assert!(g.median_m <= 9.0, "{c:?}: median gap {:.1} m", g.median_m);
        assert!(g.p90_m <= 40.0, "{c:?}: p90 gap {:.1} m", g.p90_m);
        assert!(g.free_share <= 0.25, "{c:?}: free share {:.3}", g.free_share);
    }
    assert!(refits >= 5, "only {refits} of 6 towns refit");
}

/// Protects: the change being an improvement on the set, not on a cherry-picked
/// town: the 90th-percentile gap summed over the six towns falls by at least
/// 40 % (measured 312 m off, 164 m on, a 47 % fall).
#[test]
fn the_wall_gap_falls_across_the_whole_set() {
    let (mut off_sum, mut on_sum) = (0.0, 0.0);
    for (_c, off, on) in pairs() {
        off_sum += gap_of(&off).unwrap().p90_m;
        on_sum += gap_of(&on).unwrap().p90_m;
    }
    assert!(on_sum <= off_sum * 0.60, "p90 sum {off_sum:.0} -> {on_sum:.0}");
}

/// Protects: no lot being left under or outside the curtain, and no building cut
/// by it. Every street-platted lot that is big enough to build (the building
/// stage's own floors) and whose centre was inside the circuit the refit
/// replaced has all its corners at least 4.9 m inside the new one (the bar is
/// [`MIN_MARGIN_M`], 5 m, less a tenth of float slack). A lot that was outside
/// the old circuit (an extramural ribbon lot) is not owed a margin, so is not
/// tested; but no building of any town may be straddled by the ring, in either
/// position of the switch (measured 0 of up to 2 160 buildings a town).
#[test]
fn no_lot_is_under_the_wall() {
    for (c, off, on) in pairs() {
        let (ring, arc) = (on.wall.ring.as_deref().unwrap(), on.wall.land_arc.as_deref());
        let old = off.wall.ring.as_deref().unwrap();
        if refit(&on) {
            for p in on.parcels.iter().filter(|p| p.par.wall_backing == WallBacking::No) {
                let cen = poly_centroid(&p.par.poly);
                if p.par.frontage < 3.2 || p.par.depth < 6.0 || !point_in_poly(cen, old) {
                    continue;
                }
                let m = least_margin(&p.par.poly, ring, arc);
                assert!(m >= 4.9, "{c:?}: lot {} corner {m:.1} m inside the wall", p.par.id);
            }
        }
        for b in &on.buildings {
            let inside = b.poly.iter().filter(|v| point_in_poly(**v, ring)).count();
            assert!(inside == 0 || inside == b.poly.len(), "{c:?}: the wall cuts building {}", b.id);
        }
    }
}

/// Protects: streets and the wall staying consistent. No live non-primary street
/// crosses the circuit (a street may pass the wall only at a gate, which is a
/// primary), and every land gate stands on a live primary edge, within a metre.
/// Off, the reference holds the first half by construction (`grow` refuses a
/// crossing); this holds it after the wall has moved.
#[test]
fn streets_cross_the_wall_only_at_gates_on_primaries() {
    for (c, _off, on) in pairs() {
        let ring = on.wall.ring.as_deref().expect("walled");
        let nodes = &on.graph.nodes;
        for e in &on.graph.edges {
            let (a, b) = (nodes[e.a].pt(), nodes[e.b].pt());
            // A quay runs to the water's edge, where the circuit follows the bank, so
            // it legitimately meets the water wall (the harbour mouth is left open).
            if e.cls != "primary" && e.cls != "quay" {
                assert!(
                    ring_crossings(ring, a, b).is_empty(),
                    "{c:?}: a {} street crosses the wall at {:?}",
                    e.cls,
                    ring_crossings(ring, a, b)
                );
            }
        }
        for g in on.wall.gates.iter().filter(|g| !g.water) {
            let on_primary = on.graph.edges.iter().any(|e| {
                e.cls == "primary" && dist_pt_seg(g.pt, nodes[e.a].pt(), nodes[e.b].pt()) <= 1.0
            });
            assert!(on_primary, "{c:?}: a gate at {:?} stands on no primary", g.pt);
        }
    }
}

/// Protects: the citadel staying tied to the wall: it is built after the refit,
/// from the refitted circuit, so its centre point is on the drawn curtain (the
/// land arc) within a metre, and it exists in the same towns it did with the
/// switch off. `1234/12000/inland` is the one city over `CITADEL_MIN_POP`.
#[test]
fn the_citadel_still_stands_astride_the_curtain() {
    let mut seen = 0;
    for (c, off, on) in pairs() {
        assert_eq!(off.citadel.is_some(), on.citadel.is_some(), "{c:?}: citadel presence moved");
        if let Some(cit) = &on.citadel {
            seen += 1;
            let arc = on.wall.land_arc.as_deref().unwrap();
            let d = (0..arc.len() - 1)
                .map(|i| dist_pt_seg(cit.on_curtain, arc[i], arc[i + 1]))
                .fold(f64::INFINITY, f64::min);
            assert!(d <= 1.0, "{c:?}: the citadel centre is {d:.1} m off the curtain");
        }
    }
    assert!(seen >= 1, "the fixture must include a citadel town");
}

/// Protects: the town not being rebuilt by the wall. The street-platted lots
/// (everything but the wall lots and the faubourg) are the same count, and the
/// head count `pop` stays within 10 % (measured: -8.4 % to +9.3 %, from the wall
/// lots, which are platted from the circuit and so move with it; no street lot,
/// block or street-platted building is added or removed by the refit).
#[test]
fn the_town_and_its_population_are_kept() {
    for (c, off, on) in pairs() {
        let street_lots = |t: &Town| t.parcels.iter().filter(|p| p.par.wall_backing == WallBacking::No).count();
        assert_eq!(street_lots(&off), street_lots(&on), "{c:?}: street-platted lots moved");
        assert_eq!(off.blocks.len(), on.blocks.len(), "{c:?}: blocks moved");
        let rel = (on.pop - off.pop).abs() / off.pop;
        assert!(rel <= 0.10, "{c:?}: pop {} -> {}", off.pop, on.pop);
    }
}

/// Protects: a circuit that cannot be improved staying as it was: where the
/// outcome is `Kept`, the ring, the gates and the graph are exactly the
/// switch-off ones.
#[test]
fn a_kept_circuit_is_untouched() {
    for (c, off, on) in pairs() {
        if let Some(HugOutcome::Kept(_)) = on.wall_hug {
            assert_eq!(off.wall.ring, on.wall.ring, "{c:?}");
            assert_eq!(off.wall.gates, on.wall.gates, "{c:?}");
            assert_eq!(off.graph.edges.len(), on.graph.edges.len(), "{c:?}");
        }
    }
}

/// Protects: the envelope following a bay. A ring of points with a notch cut in
/// one side takes a smaller radius in the notch's direction than at the arms
/// beside it, and encloses every point with the pad to spare; a convex hull
/// could do neither.
#[test]
fn the_envelope_follows_a_notch_and_encloses_every_point() {
    let c0 = Vec2::new(0.0, 0.0);
    let mut pts = Vec::new();
    for k in 0..720 {
        let a = f64::from(k) * std::f64::consts::PI / 360.0;
        // Radius 100 m, except a notch 60 degrees wide at the east with radius 40 m.
        let r = if a.cos() > 0.866 { 40.0 } else { 100.0 };
        pts.push(Vec2::new(r * a.cos(), r * a.sin()));
    }
    let env = envelope(c0, &pts, 8.0);
    let radius_at = |deg: f64| -> f64 {
        let k = ((deg.rem_euclid(360.0) / 360.0) * env.len() as f64) as usize % env.len();
        env[k].dist(c0)
    };
    assert!(radius_at(0.0) < 60.0, "the notch radius is {:.0}", radius_at(0.0));
    assert!(radius_at(90.0) > 100.0, "the arm radius is {:.0}", radius_at(90.0));
    assert!(least_margin(&pts, &env, None) >= 7.0, "a point is not enclosed with the pad");
    assert!(envelope(c0, &[], 8.0).is_empty(), "no points is no outline, not a point at the centre");
}

/// Protects: the margin test's two rules: a point outside is negative by its
/// distance, and a point at the water side within the shore tolerance is not
/// measured (a shore street sits on the bank wall).
#[test]
fn the_margin_is_signed_and_a_shore_street_is_exempt() {
    let ring = [Vec2::new(0.0, 0.0), Vec2::new(100.0, 0.0), Vec2::new(100.0, 100.0), Vec2::new(0.0, 100.0)];
    assert!((least_margin(&[Vec2::new(50.0, 50.0)], &ring, None) - 50.0).abs() < 1e-9);
    assert!((least_margin(&[Vec2::new(150.0, 50.0)], &ring, None) + 50.0).abs() < 1e-9);
    assert!(least_margin(&[], &ring, None).is_infinite());
    // The land arc is the top, left and bottom sides; the right side is water.
    let arc = [Vec2::new(100.0, 0.0), Vec2::new(0.0, 0.0), Vec2::new(0.0, 100.0), Vec2::new(100.0, 100.0)];
    let shore_street = Vec2::new(101.5, 50.0);
    assert!(least_margin(&[shore_street], &ring, Some(&arc)).is_infinite(), "exempt at the shore");
    // The same point beside a LAND side is a real failure.
    let land_side = Vec2::new(-1.5, 50.0);
    assert!((least_margin(&[land_side], &ring, Some(&arc)) + 1.5).abs() < 1e-9);
}

/// Protects: `wall_gap`'s arithmetic on a case solvable by hand: a 100 m square
/// wall round one 40 m square building at its centre. The wall is 30 m from the
/// building at each side's middle and 42.4 m at the corners, so the median is
/// between 33 and 38, the 90th percentile at least 40 and at most 42.5, and the
/// share of the ground farther than 30 m from the building (the corners, outside
/// a rounded 30 m margin) about 7.7 %.
#[test]
fn the_wall_gap_matches_a_hand_computed_square() {
    let ring = [Vec2::new(0.0, 0.0), Vec2::new(100.0, 0.0), Vec2::new(100.0, 100.0), Vec2::new(0.0, 100.0)];
    let bld = [Vec2::new(30.0, 30.0), Vec2::new(70.0, 30.0), Vec2::new(70.0, 70.0), Vec2::new(30.0, 70.0)];
    let g = wall_gap(&ring, None, &[&bld[..]]).expect("measurable");
    assert!(g.median_m >= 30.0 && g.median_m < 32.0, "median {}", g.median_m);
    assert!(g.p90_m >= 36.0 && g.p90_m <= 42.5, "p90 {}", g.p90_m);
    assert!(g.free_share > 0.06 && g.free_share < 0.10, "free share {}", g.free_share);
    assert!((g.area_m2 - 10000.0).abs() < 1e-6);
    // No value is not a perfect fit: nothing inside, or a degenerate ring, is None.
    assert!(wall_gap(&ring, None, &[]).is_none());
    let outside = [Vec2::new(200.0, 200.0), Vec2::new(210.0, 200.0), Vec2::new(210.0, 210.0)];
    assert!(wall_gap(&ring, None, &[&outside[..]]).is_none(), "a building outside the wall is not inside");
    assert!(wall_gap(&ring[..2], None, &[&bld[..]]).is_none());
}

/// Protects Ruling FB's span test, `one_bank_by_lots` (literals): 100 lots
/// on the market's bank and 32 on the other span the river (32 is 32 %); 31
/// do not, and the circuit keeps the market's bank; 5 on the market's bank
/// and 100 across keep the other bank; 10 and 7 do not span (under eight on
/// the smaller bank), and an even split keeps the market's.
#[test]
fn a_wall_spans_its_river_only_when_both_banks_are_built() {
    let sides = |near: usize, far: usize| -> Vec<f64> { std::iter::repeat_n(1.0, near).chain(std::iter::repeat_n(-1.0, far)).collect() };
    assert_eq!(super::one_bank_by_lots(&sides(100, 32), 1.0), None);
    assert_eq!(super::one_bank_by_lots(&sides(100, 31), 1.0), Some(1.0));
    assert_eq!(super::one_bank_by_lots(&sides(5, 100), 1.0), Some(-1.0));
    assert_eq!(super::one_bank_by_lots(&sides(10, 7), 1.0), Some(1.0));
    assert_eq!(super::one_bank_by_lots(&sides(10, 8), 1.0), None);
    assert_eq!(super::one_bank_by_lots(&sides(4, 4), -1.0), Some(-1.0));
}
