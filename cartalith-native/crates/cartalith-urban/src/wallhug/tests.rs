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
/// numbers [`crate::wallgap`] measures. Measured with the taut wall (Ruling FE
/// item 1, 2026-11-15): on every refit case the median wall-to-building gap
/// is 2.3-6.5 m (bar 9), the 90th percentile 16-66 m (bar 70), the free share
/// 0.001-0.257 (bar 0.30); off, the same towns read a median up to 46 m, a p90
/// up to 85 m and a free share up to 0.376. The p90 and free-share bars were
/// 40 m and 0.25 for the traced hug (Ruling EW item 1: 16-35 m and
/// 0.001-0.221); a taut run across a bay encloses the bay, the "about a
/// fifth more open ground" the owner accepted in Ruling FE item 1, while the
/// median gap (the wall street along the built frontage) is unchanged. Five of
/// the six towns refit; the sixth, a coast town, keeps its circuit
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
        assert!(g.p90_m <= 70.0, "{c:?}: p90 gap {:.1} m", g.p90_m);
        assert!(g.free_share <= 0.30, "{c:?}: free share {:.3}", g.free_share);
    }
    assert!(refits >= 5, "only {refits} of 6 towns refit");
}

/// Protects the taut circuit as the circuit builder draws it
/// (`BuiltMass::taut`: no Chaikin rounding, outward simplify, towers every
/// 90 m at most): on every refit case the ring is at most 15 % longer than
/// its convex hull (the judgement's p90 bar; measured up to 1.106, the 12 000
/// city whose bay is over the pocket cap and keeps its corner), has at most one vertex per 35 m
/// of wall, and no run longer than 90 m, and the land arc no run longer than
/// 90 m either.
#[test]
fn a_taut_wall_is_straight_runs_between_towers() {
    let len = |r: &[Vec2]| -> f64 { (0..r.len()).map(|i| r[i].dist(r[(i + 1) % r.len()])).sum() };
    for (c, _off, on) in pairs() {
        if !refit(&on) {
            continue;
        }
        let ring = on.wall.ring.as_deref().unwrap();
        let (l, h) = (len(ring), len(&convex_hull(ring)));
        assert!(l / h <= 1.15, "{c:?}: wall / hull {:.3}", l / h);
        assert!(ring.len() as f64 <= l / 35.0, "{c:?}: {} vertices for {l:.0} m", ring.len());
        for i in 0..ring.len() {
            assert!(ring[i].dist(ring[(i + 1) % ring.len()]) <= 90.0 + 1e-6, "{c:?}: a run of {:.0} m", ring[i].dist(ring[(i + 1) % ring.len()]));
        }
        if let Some(arc) = on.wall.land_arc.as_deref() {
            for w in arc.windows(2) {
                assert!(w[0].dist(w[1]) <= 90.0 + 1e-6, "{c:?}: an arc run of {:.0} m", w[0].dist(w[1]));
            }
        }
    }
}

/// Protects: the change being an improvement on the set, not on a cherry-picked
/// town: the 90th-percentile gap summed over the six towns falls by at least
/// 20 % (measured 312 m off, 233 m on with the taut wall, a 25 % fall; the
/// traced hug before Ruling FE item 1 fell 47 %, to 164 m, and a taut run
/// across a bay gives some of that back by design).
#[test]
fn the_wall_gap_falls_across_the_whole_set() {
    let (mut off_sum, mut on_sum) = (0.0, 0.0);
    for (_c, off, on) in pairs() {
        off_sum += gap_of(&off).unwrap().p90_m;
        on_sum += gap_of(&on).unwrap().p90_m;
    }
    assert!(on_sum <= off_sum * 0.80, "p90 sum {off_sum:.0} -> {on_sum:.0}");
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
/// do not, and the circuit keeps the market's bank; 5 (or none) on the
/// market's bank and 100 across span the river, so the market stays inside
/// (2026-11-15: never the other bank alone); 10 and 7 do not span (under
/// eight on the far bank), and four and four keep the market's.
#[test]
fn a_wall_spans_its_river_only_when_both_banks_are_built() {
    let sides = |near: usize, far: usize| -> Vec<f64> { std::iter::repeat_n(1.0, near).chain(std::iter::repeat_n(-1.0, far)).collect() };
    assert_eq!(super::one_bank_by_lots(&sides(100, 32), 1.0), None);
    assert_eq!(super::one_bank_by_lots(&sides(100, 31), 1.0), Some(1.0));
    assert_eq!(super::one_bank_by_lots(&sides(5, 100), 1.0), None);
    assert_eq!(super::one_bank_by_lots(&sides(0, 100), 1.0), None);
    assert_eq!(super::one_bank_by_lots(&sides(10, 7), 1.0), Some(1.0));
    assert_eq!(super::one_bank_by_lots(&sides(10, 8), 1.0), None);
    assert_eq!(super::one_bank_by_lots(&sides(4, 4), -1.0), Some(-1.0));
}

/// Lots on a 20 m grid filling `rect` (x0, y0, x1, y1), each 16 m square.
fn lot_grid(rect: (f64, f64, f64, f64)) -> Vec<Vec<Vec2>> {
    let mut out = Vec::new();
    let mut y = rect.1;
    while y + 16.0 <= rect.3 {
        let mut x = rect.0;
        while x + 16.0 <= rect.2 {
            out.push(vec![Vec2::new(x, y), Vec2::new(x + 16.0, y), Vec2::new(x + 16.0, y + 16.0), Vec2::new(x, y + 16.0)]);
            x += 20.0;
        }
        y += 20.0;
    }
    out
}

/// Protects `outline` (2026-11-15, defect (b); literals): a U of lots, two
/// 100 m arms 200 m apart joined by a base, is outlined round the arms and
/// the base and not across the 200 m bay between them (wider than twice
/// `CLOSE_R_M`); every lot corner stands inside it; the outline is the pad
/// (8 m) out from the lots less the trace's half cell and the 2 m simplify
/// at a corner (at least 2 m; `hug_wall` widens for its 5 m bar), and not
/// more than 16 m; it is
/// `None` for two quarters 300 m apart and for a pad of 40 m (`CLOSE_R_M`);
/// a single stray lot 300 m off a town is left out (`OUTLINE_MINOR_SHARE`).
#[test]
fn the_outline_follows_a_bend_and_does_not_wall_the_bay() {
    let mut lots = lot_grid((0.0, 0.0, 100.0, 300.0));
    lots.extend(lot_grid((300.0, 0.0, 400.0, 300.0)));
    lots.extend(lot_grid((100.0, 200.0, 300.0, 300.0)));
    let polys: Vec<&[Vec2]> = lots.iter().map(Vec::as_slice).collect();
    let (ring, dropped) = outline(&polys, &[], 8.0, None).expect("one piece");
    assert!(dropped.is_empty());
    assert!(!point_in_poly(Vec2::new(200.0, 60.0), &ring), "the bay is walled");
    assert!(point_in_poly(Vec2::new(200.0, 250.0), &ring), "the base is not");
    for l in &lots {
        for c in l {
            assert!(point_in_poly(*c, &ring), "lot corner {c:?} outside");
        }
    }
    let m = least_margin(&lots.concat(), &ring, None);
    assert!((2.0..=16.0).contains(&m), "margin {m}");
    let mut apart = lot_grid((0.0, 0.0, 100.0, 100.0));
    apart.extend(lot_grid((400.0, 0.0, 500.0, 100.0)));
    let polys: Vec<&[Vec2]> = apart.iter().map(Vec::as_slice).collect();
    assert!(outline(&polys, &[], 8.0, None).is_none());
    let one = lot_grid((0.0, 0.0, 100.0, 100.0));
    let polys: Vec<&[Vec2]> = one.iter().map(Vec::as_slice).collect();
    assert!(outline(&polys, &[], 40.0, None).is_none());
    assert!(outline(&polys, &[], 8.0, None).is_some());
    // A stray lot 300 m off a 300 m town is left out, and named.
    let mut stray = lot_grid((0.0, 0.0, 300.0, 300.0));
    stray.push(vec![Vec2::new(600.0, 0.0), Vec2::new(616.0, 0.0), Vec2::new(616.0, 16.0), Vec2::new(600.0, 16.0)]);
    let polys: Vec<&[Vec2]> = stray.iter().map(Vec::as_slice).collect();
    let (ring, dropped) = outline(&polys, &[], 8.0, None).expect("a stray lot is a suburb");
    assert!(!dropped.is_empty() && dropped.iter().all(|d| d.x > 590.0), "{dropped:?}");
    assert!(!point_in_poly(Vec2::new(608.0, 8.0), &ring));
    // ...but never the market square: a square in the stray piece refuses the loop.
    assert!(outline(&polys, &[], 8.0, Some(Vec2::new(608.0, 8.0))).is_none());
    assert!(outline(&polys, &[], 8.0, Some(Vec2::new(150.0, 150.0))).is_some());
}

/// Protects `untangle` (2026-11-15; literals): a 100 m square whose right
/// side has a 2 m bow-tie sliver (a loop of 2 m2, under 2 % of 10 000 m2) is
/// returned without the crossing and with its area within 10 m2; a figure
/// of eight with two equal halves (a loop of half the area) is refused.
#[test]
fn a_sliver_loop_is_cut_and_a_fold_is_refused() {
    let sliver = [
        Vec2::new(0.0, 0.0),
        Vec2::new(100.0, 0.0),
        Vec2::new(100.0, 49.0),
        Vec2::new(102.0, 51.0),
        Vec2::new(102.0, 49.0),
        Vec2::new(100.0, 51.0),
        Vec2::new(100.0, 100.0),
        Vec2::new(0.0, 100.0),
    ];
    assert!(ring_self_crosses(&sliver));
    let u = untangle(&sliver).expect("a sliver is cut");
    assert!(!ring_self_crosses(&u));
    assert!((poly_area(&u).abs() - 10_000.0).abs() < 10.0, "area {}", poly_area(&u));
    let eight = [Vec2::new(0.0, 0.0), Vec2::new(100.0, 100.0), Vec2::new(100.0, 0.0), Vec2::new(0.0, 100.0)];
    assert!(ring_self_crosses(&eight));
    assert!(untangle(&eight).is_none());
    let square = [Vec2::new(0.0, 0.0), Vec2::new(10.0, 0.0), Vec2::new(10.0, 10.0), Vec2::new(0.0, 10.0)];
    assert!(!ring_self_crosses(&square));
}

/// Protects `outline`'s closing radius (`CLOSE_R_M` 40 m; literals): two
/// 100 m blocks of lots with a 64 m gap between them (under twice 40 m) are
/// one quarter, the gap closed over and inside the loop; at a 10 m radius
/// they would be two quarters and no loop at all.
#[test]
fn a_gap_under_twice_the_closing_radius_is_walled_over() {
    let mut lots = lot_grid((0.0, 0.0, 100.0, 100.0));
    lots.extend(lot_grid((160.0, 0.0, 260.0, 100.0)));
    let polys: Vec<&[Vec2]> = lots.iter().map(Vec::as_slice).collect();
    let (ring, dropped) = outline(&polys, &[], 8.0, None).expect("one quarter");
    assert!(dropped.is_empty());
    assert!(point_in_poly(Vec2::new(128.0, 50.0), &ring), "the 64 m gap is not walled over");
}

/// Protects `untangle`'s refusal (`UNTANGLE_MAX_SHARE` 2 %; literals): a
/// 100 m square whose right side is folded into a bow-tie 40 m deep crosses
/// itself in a loop of 800 m2, 8 % of its 10 000 m2, which is a fold, not a
/// sliver, so it is refused rather than cut.
#[test]
fn a_fold_over_two_percent_is_refused() {
    let fold = [
        Vec2::new(0.0, 0.0),
        Vec2::new(100.0, 0.0),
        Vec2::new(100.0, 10.0),
        Vec2::new(140.0, 90.0),
        Vec2::new(140.0, 10.0),
        Vec2::new(100.0, 90.0),
        Vec2::new(100.0, 100.0),
        Vec2::new(0.0, 100.0),
    ];
    assert!(ring_self_crosses(&fold));
    assert!(untangle(&fold).is_none());
}

/// Protects `BuiltMass::exact` in `build_wall_from_mass` (2026-11-15;
/// literals): a circuit across the river (`spans_water`) built on a U-shaped
/// hull open to the north keeps the bay at (850, 500) outside it when
/// `exact`, and walls it, the reference's convex hull, when not.
#[test]
fn an_exact_circuit_keeps_the_town_shape() {
    let site = crate::site::build_site(7, 1700.0, 1250.0, "landlocked", crate::site::SiteOpts::default());
    let anchors = crate::routes::Anchors { market: Vec2::new(650.0, 750.0), prov: "test" };
    let g = crate::graph::Graph::new();
    let hull: Vec<Vec2> = [(600.0, 400.0), (700.0, 400.0), (700.0, 700.0), (1000.0, 700.0), (1000.0, 400.0), (1100.0, 400.0), (1100.0, 800.0), (600.0, 800.0)]
        .iter()
        .map(|&(x, y)| Vec2::new(x, y))
        .collect();
    for exact in [true, false] {
        let mut ws = WallState::default();
        let built = BuiltMass { hull: hull.clone(), spans_water: true, exact, taut: false };
        build_wall_from_mass(7, &site, &anchors, &g, &mut ws, 3, None, &FortOpts::default(), built, false);
        let ring = ws.ring.expect("a circuit");
        assert_eq!(point_in_poly(Vec2::new(850.0, 500.0), &ring), !exact, "exact {exact}");
    }
}


// ---------------------------------------------------------------- taut wall --
// Ruling FE item 1 (2026-11-15): `tauten`, `simplify_out`, `subdivide_runs`,
// `largest_open_m2` and `taut_hull`, on hand-built shapes whose answers are
// known without the code under test.

fn loop_len(r: &[Vec2]) -> f64 {
    (0..r.len()).map(|i| r[i].dist(r[(i + 1) % r.len()])).sum()
}

/// No point of `before` stands outside `after` by more than a millimetre.
fn encloses(after: &[Vec2], before: &[Vec2]) -> bool {
    before.iter().all(|p| point_in_poly(*p, after) || least_margin(&[*p], after, None) > -1e-3)
}

/// A 400 m square (counter-clockwise in the plane's axes) with a V-notch cut
/// into its south side, `w` wide at the mouth and `d` deep: the notch's
/// area is `w * d / 2`, and its apex is the one reflex vertex.
fn notched_square(w: f64, d: f64) -> Vec<Vec2> {
    vec![
        Vec2::new(0.0, 0.0),
        Vec2::new(200.0 - w / 2.0, 0.0),
        Vec2::new(200.0, d),
        Vec2::new(200.0 + w / 2.0, 0.0),
        Vec2::new(400.0, 0.0),
        Vec2::new(400.0, 400.0),
        Vec2::new(0.0, 400.0),
    ]
}

/// Protects: the pocket cap. A notch of 7 000 m2 (under the 8 000 m2 cap) is
/// walled over by a straight run; one of 10 000 m2 (over it) keeps its
/// re-entrant corner, so its apex stays outside. Either way nothing moves
/// inward. A cap outside 7 000..10 000 turns one of the two red.
#[test]
fn a_small_notch_is_walled_over_and_a_large_one_keeps_its_corner() {
    let small = notched_square(100.0, 140.0);
    let t = tauten(&small, TAUT_POCKET_MAX_M2);
    assert!(point_in_poly(Vec2::new(200.0, 120.0), &t), "a 0.70 ha notch is filled: {t:?}");
    assert!(encloses(&t, &small));
    let large = notched_square(100.0, 200.0);
    let t = tauten(&large, TAUT_POCKET_MAX_M2);
    assert!(!point_in_poly(Vec2::new(200.0, 180.0), &t), "a 1.0 ha notch keeps its corner: {t:?}");
    assert!(encloses(&t, &large));
}

/// Protects: the scalloped trace becoming straight runs. A 200 m circle
/// carrying 12 scallops 6 m deep, sampled every 2 degrees (180 vertices,
/// the traced outline's kind of line): pulled taut it is at most 1 % longer
/// than its convex hull (measured 1.002; the scalloped input is 1.008, and
/// both shrink to 1.0 only as a polygon), has at most one vertex per
/// `TAUT_MIN_RUN_M` of length, every run at least that long, no turn
/// sharper than `TAUT_MAX_TURN`, never crosses itself and never moves a
/// point inward.
#[test]
fn a_scalloped_outline_is_pulled_into_straight_runs() {
    let ring: Vec<Vec2> = (0..180)
        .map(|k| {
            let a = k as f64 * 2.0_f64.to_radians();
            let r = 200.0 - 6.0 * (6.0 * a).sin().abs();
            Vec2::new(r * a.cos(), r * a.sin())
        })
        .collect();
    let t = tauten(&ring, TAUT_POCKET_MAX_M2);
    let hull = convex_hull(&t);
    assert!(loop_len(&t) / loop_len(&hull) <= 1.01, "taut/hull {:.3}", loop_len(&t) / loop_len(&hull));
    assert!(t.len() as f64 <= loop_len(&t) / 40.0 + 1.0, "{} vertices for {:.0} m", t.len(), loop_len(&t));
    for i in 0..t.len() {
        let (a, b, c) = (t[(i + t.len() - 1) % t.len()], t[i], t[(i + 1) % t.len()]);
        assert!(b.dist(c) >= 40.0 - 1e-6, "run {i} is {:.1} m", b.dist(c));
        let turn = ((b - a).norm().dot((c - b).norm())).clamp(-1.0, 1.0).acos();
        assert!(turn <= 75.0_f64.to_radians() + 1e-9, "turn {:.1} deg at {i}", turn.to_degrees());
    }
    assert!(!ring_self_crosses(&t));
    assert!(encloses(&t, &ring));
}

/// Protects: the turn cap. A blunted arrow tip (two long sides meeting
/// through a 10 m end, a 60 degree point) is left blunt: extending the sides
/// to meet would turn 120 degrees, past `TAUT_MAX_TURN`'s 75.
#[test]
fn a_sharp_tip_is_not_extended_into_a_spike() {
    // Five vertices: `tauten` leaves a loop of four alone, so the tail point
    // (-100, 0) is what lets the merge be tried at all.
    let ring = vec![Vec2::new(0.0, -150.0), Vec2::new(300.0, -5.0), Vec2::new(300.0, 5.0), Vec2::new(0.0, 150.0), Vec2::new(-100.0, 0.0)];
    let t = tauten(&ring, TAUT_POCKET_MAX_M2);
    assert_eq!(t.len(), 5, "the blunt end stays: {t:?}");
}

/// Protects: `simplify_out`'s one-sidedness and its tolerance. On a line
/// whose loop lies to its left (`orient` +1, the plane's axes), a point half
/// a metre to the left is dropped (the chord passes outside it), one half a
/// metre to the right is kept (the chord would leave it outside the wall),
/// and one 1.5 m to the left is kept (past `TAUT_SIMPLIFY_M`).
#[test]
fn simplify_out_moves_the_line_outward_only() {
    let line = |y: f64| vec![Vec2::new(0.0, 0.0), Vec2::new(50.0, y), Vec2::new(100.0, 0.0)];
    assert_eq!(simplify_out(&line(0.5), TAUT_SIMPLIFY_M, 1.0, false).len(), 2);
    assert_eq!(simplify_out(&line(-0.5), TAUT_SIMPLIFY_M, 1.0, false).len(), 3);
    assert_eq!(simplify_out(&line(1.5), TAUT_SIMPLIFY_M, 1.0, false).len(), 3);
    // Reversed orientation reverses the sides.
    assert_eq!(simplify_out(&line(-0.5), TAUT_SIMPLIFY_M, -1.0, false).len(), 2);
}

/// Protects: towers at least every `TAUT_MAX_RUN_M` (90 m). A 200 m run
/// becomes three equal runs (66.7 m); an 80 m run is left alone; a closed
/// square of 100 m sides gets one tower mid-side.
#[test]
fn long_runs_get_towers_at_equal_steps() {
    let r = subdivide_runs(&[Vec2::new(0.0, 0.0), Vec2::new(200.0, 0.0)], false);
    assert_eq!(r.len(), 4);
    assert!((r[1].x - 200.0 / 3.0).abs() < 1e-9);
    assert_eq!(subdivide_runs(&[Vec2::new(0.0, 0.0), Vec2::new(80.0, 0.0)], false).len(), 2);
    let sq = [Vec2::new(0.0, 0.0), Vec2::new(100.0, 0.0), Vec2::new(100.0, 100.0), Vec2::new(0.0, 100.0)];
    assert_eq!(subdivide_runs(&sq, true).len(), 8);
}

/// A landlocked synthetic site with no water: `largest_open_m2` reads only
/// its water tests.
fn dry_site() -> Site {
    crate::site::build_site(7, 1700.0, 1250.0, "landlocked", crate::site::SiteOpts::default())
}

/// Protects: the open-ground measure and its 40 m reach. A 200 x 200 m
/// ring whose west half is one lot: open is what lies over 40 m from it,
/// the strip from x 140 to 200, 1.2 ha (the 4 m grid and the chamfer's
/// diagonal error give 1.1-1.3).
#[test]
fn open_ground_is_what_lies_beyond_reach_of_every_lot() {
    let ring = [Vec2::new(500.0, 500.0), Vec2::new(700.0, 500.0), Vec2::new(700.0, 700.0), Vec2::new(500.0, 700.0)];
    let lot = [Vec2::new(500.0, 500.0), Vec2::new(600.0, 500.0), Vec2::new(600.0, 700.0), Vec2::new(500.0, 700.0)];
    let a = largest_open_m2(&ring, &[&lot[..]], &dry_site());
    assert!((11_000.0..=13_000.0).contains(&a), "open {a:.0} m2");
    assert_eq!(largest_open_m2(&ring, &[&ring[..]], &dry_site()), 0.0);
}

/// Protects: `taut_hull` pulling less taut where a straight run would wall
/// an empty field. An 800 m square with a V-notch 500 m wide and 500 m deep
/// (12.5 ha), every 20 m cell of the square a lot: filled at a cap that
/// takes it whole, the notch would enclose about 6.9 ha over 40 m from any
/// lot, so the guard falls back to the quarter cap, which does not fill it,
/// and the apex stays outside.
#[test]
fn a_taut_run_never_walls_an_empty_field() {
    let site = dry_site();
    let off = Vec2::new(450.0, 225.0);
    let ring: Vec<Vec2> = vec![
        Vec2::new(0.0, 0.0),
        Vec2::new(150.0, 0.0),
        Vec2::new(400.0, 500.0),
        Vec2::new(650.0, 0.0),
        Vec2::new(800.0, 0.0),
        Vec2::new(800.0, 800.0),
        Vec2::new(0.0, 800.0),
    ]
    .into_iter()
    .map(|p| p + off)
    .collect();
    let mut lots: Vec<Vec<Vec2>> = Vec::new();
    for i in 0..40 {
        for j in 0..40 {
            let (x, y) = (i as f64 * 20.0, j as f64 * 20.0);
            let c = Vec2::new(x + 10.0, y + 10.0) + off;
            if point_in_poly(c, &ring) {
                lots.push(vec![Vec2::new(x, y) + off, Vec2::new(x + 20.0, y) + off, Vec2::new(x + 20.0, y + 20.0) + off, Vec2::new(x, y + 20.0) + off]);
            }
        }
    }
    let refs: Vec<&[Vec2]> = lots.iter().map(Vec::as_slice).collect();
    let apex = Vec2::new(400.0, 300.0) + off;
    assert!(point_in_poly(apex, &tauten(&ring, 200_000.0)), "premise: the full cap fills the notch");
    let (t, ok) = taut_hull(&ring, &refs, &site, 200_000.0);
    assert!(ok, "a smaller cap meets the bar");
    assert!(largest_open_m2(&t, &refs, &site) <= OPEN_PATCH_MAX_M2);
    assert!(!point_in_poly(apex, &t), "the empty notch stays outside");
    // The open ground the outline holds itself: a 200 x 200 m ring with one
    // lot in a corner fails every cap, and the tautest circuit comes back.
    let sq = [Vec2::new(500.0, 500.0), Vec2::new(700.0, 500.0), Vec2::new(700.0, 700.0), Vec2::new(500.0, 700.0)];
    let lot = [Vec2::new(500.0, 500.0), Vec2::new(520.0, 500.0), Vec2::new(520.0, 520.0), Vec2::new(500.0, 520.0)];
    let (t, ok) = taut_hull(&sq, &[&lot[..]], &site, TAUT_POCKET_MAX_M2);
    assert!(!ok && t.len() == 4, "{t:?}");
}


/// Protects `TAUT_MIN_RUN_M` (40 m) on its own: a regular 36-gon of radius
/// 200 m has 34.9 m runs, which are merged (a 30 m minimum would keep all 36).
#[test]
fn runs_under_forty_metres_are_merged() {
    let ring: Vec<Vec2> = (0..36).map(|k| {
        let a = k as f64 * 10.0_f64.to_radians();
        Vec2::new(200.0 * a.cos(), 200.0 * a.sin())
    }).collect();
    let t = tauten(&ring, TAUT_POCKET_MAX_M2);
    assert!(t.len() < 36, "{} vertices", t.len());
    for i in 0..t.len() {
        assert!(t[i].dist(t[(i + 1) % t.len()]) >= 40.0 - 1e-6);
    }
}

/// Protects `tauten`'s two refusals, the crossing and swallowed-vertex test
/// and the forward test on a merge, by property: 300 random star-shaped
/// outlines (a fixed LCG; 40-120 vertices, radius 60-260 m with deep
/// notches) pulled taut never cross themselves and never move a point of the
/// outline outside the result.
#[test]
fn random_outlines_pulled_taut_stay_simple_and_enclosing() {
    let mut seed: u64 = 0x5eed_1115;
    let mut rnd = || {
        seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1_442_695_040_888_963_407);
        ((seed >> 33) as f64) / ((1u64 << 31) as f64)
    };
    for case in 0..300 {
        let n = 40 + (rnd() * 80.0) as usize;
        let ring: Vec<Vec2> = (0..n)
            .map(|k| {
                let a = k as f64 / n as f64 * 2.0 * std::f64::consts::PI;
                let r = if rnd() < 0.15 { 60.0 + 60.0 * rnd() } else { 200.0 + 60.0 * rnd() };
                Vec2::new(r * a.cos(), r * a.sin())
            })
            .collect();
        for cap in [TAUT_POCKET_MAX_M2, 1.0e9] {
            let t = tauten(&ring, cap);
            assert!(!ring_self_crosses(&t), "case {case} cap {cap}: crosses");
            assert!(encloses(&t, &ring), "case {case} cap {cap}: moved a point inward");
        }
    }
}
