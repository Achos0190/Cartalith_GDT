//! The perimeter block's own acceptance bar. There is no reference to diff
//! against (`courtyard.rs`'s header): the geometry is pinned on hand-built
//! squares with literal answers, and the owner's plan as properties of real
//! generated towns.

use super::*;
use crate::generate::{GenOpts, Town, generate};
use crate::geom::{dist_pt_seg, point_in_poly};

fn square(s: f64) -> Vec<Vec2> {
    vec![Vec2::new(0.0, 0.0), Vec2::new(s, 0.0), Vec2::new(s, s), Vec2::new(0.0, s)]
}

#[test]
fn a_square_block_is_a_ring_of_lots_round_a_square_court() {
    let blk = square(60.0);
    let (court, lots) = ring_plat(&blk, 15.0, &mut || 10.0).expect("a 60 m block rings at 15 m");
    // The court is the block inset 15 m all round: a 30 m square, 900 m².
    assert!((poly_area(&court) - 900.0).abs() < 1e-6);
    // 60 m edges at a 10 m frontage: six lots an edge, 24 in all.
    assert_eq!(lots.len(), 24);
    // They tile the band exactly: ring + court = block, with no overlap.
    let ring: f64 = lots.iter().map(|(_, q)| poly_area(q)).sum();
    assert!((ring + 900.0 - 3600.0).abs() < 1e-6, "ring {ring}");
    for (k, (i, q)) in lots.iter().enumerate() {
        // Street frontage on the block's edge, back line on the court's.
        let on = |p: Vec2, s: Vec2, e: Vec2| dist_pt_seg(p, s, e) < 1e-9;
        let (a, b) = (blk[*i], blk[(i + 1) % 4]);
        assert!(on(q[0], a, b) && on(q[1], a, b), "lot {k} fronts edge {i}");
        let (ca, cb) = (court[*i], court[(i + 1) % 4]);
        assert!(on(q[2], ca, cb) && on(q[3], ca, cb), "lot {k} backs on the court");
        let c = poly_centroid(q);
        assert!(!point_in_poly(c, &court), "lot {k} is not in the court");
        for (j, (_, o)) in lots.iter().enumerate() {
            assert!(j == k || !point_in_poly(c, o), "lots {k} and {j} overlap");
        }
    }
}

#[test]
fn the_ring_depth_is_solved_for_the_court_share() {
    // (60 - 2d)² = 0.25 · 3600  ⇒  d = 15.
    let d = ring_depth(&square(60.0), 0.25).expect("dense");
    assert!((d - 15.0).abs() < 1e-3, "{d}");
    // Too big: even a 22 m ring leaves (200-44)² / 200² = 61% of court.
    assert_eq!(ring_depth(&square(200.0), 0.2), None);
    // Too small: an 8 m ring leaves a 2 m court, under the 80 m² floor.
    assert_eq!(ring_depth(&square(18.0), 0.2), None);
    // Clamped at the deep end but still dense: (100-44)² / 100² = 31.4%.
    assert_eq!(ring_depth(&square(100.0), 0.2), Some(22.0));
    // Clamped at the shallow end: (30-16)² = 196 m², 21.8% of 900.
    assert_eq!(ring_depth(&square(30.0), 0.25), Some(8.0));
    // Under the court floor: an 8 m ring leaves 8.5² = 72.25 m² < 80.
    assert_eq!(ring_depth(&square(24.5), 0.2), None);
    // Past its half-width an inset comes back as the block's mirror,
    // correctly wound: no court, not a 380 m² one.
    assert!(inset_poly(&square(24.5), &[22.0]).is_some_and(|c| (poly_area(&c) - 380.25).abs() < 1e-9));
    assert_eq!(court_of(&square(24.5), 22.0), None);
    assert_eq!(court_of(&square(24.5), 8.0).map(|c| poly_area(&c)), Some(72.25));
}

#[test]
fn a_notched_block_still_tiles_and_a_short_chamfer_is_refused() {
    // A reflex notch in a 40 m block: the ring still tiles it exactly.
    let blk = vec![
        Vec2::new(0.0, 0.0),
        Vec2::new(40.0, 0.0),
        Vec2::new(40.0, 40.0),
        Vec2::new(21.5, 40.0),
        Vec2::new(20.0, 38.0),
        Vec2::new(18.5, 40.0),
        Vec2::new(0.0, 40.0),
    ];
    let (court, lots) = ring_plat(&blk, 10.0, &mut || 9.0).expect("the notch rings");
    // Every trapezoid, the notch's two included, holds a lot at least the
    // 26 m² floor: 4 + 4 + 2 + 1 + 1 + 2 + 4 lots, and ring + court = block.
    assert_eq!(lots.len(), 18);
    let ring: f64 = lots.iter().map(|(_, q)| poly_area(q)).sum();
    assert!((ring + poly_area(&court) - 1597.0).abs() < 1e-6, "{ring}");
    for (_, q) in &lots {
        assert!(!poly_self_intersects(q) && poly_area(q) > 0.0);
    }
    // A 2.8 m chamfer folds under an 8 m inset: refused, never a bowtie.
    let chamfer = vec![
        Vec2::new(0.0, 0.0),
        Vec2::new(60.0, 0.0),
        Vec2::new(60.0, 38.0),
        Vec2::new(58.0, 40.0),
        Vec2::new(0.0, 40.0),
    ];
    assert!(ring_plat(&chamfer, 8.0, &mut || 9.0).is_none());
    // The same chamfer past 60 vertices, where `inset_poly` no longer runs
    // its own self-intersection test: the fold must still be refused. The
    // extra vertices are collinear points along the long bottom edge.
    let mut many = vec![Vec2::new(0.0, 0.0)];
    many.extend((1..60).map(|k| Vec2::new(k as f64, 0.0)));
    many.extend(chamfer[1..].iter().copied());
    assert!(many.len() > 60);
    assert!(inset_poly(&many, &[8.0]).is_some(), "the inset itself no longer refuses");
    assert!(ring_plat(&many, 8.0, &mut || 9.0).is_none());
}

fn town(seed: u32, site: &str, pop: f64, culture: Option<&str>) -> Town {
    let o = GenOpts {
        pop: Some(pop),
        site: Some(site.into()),
        culture: culture.map(Into::into),
        ..GenOpts::default()
    };
    generate(seed, &o)
}

#[test]
fn real_towns_ring_their_outermost_dense_blocks_and_nothing_else() {
    let mut rung = 0;
    for (seed, site, pop) in [
        (42u32, "river", 5000.0),
        (1234, "inland", 12000.0),
        (7, "inland", 1500.0),
        (99, "coast", 9000.0),
        (3, "riverthrough", 7000.0),
    ] {
        let t = town(seed, site, pop, None);
        let m = t.anchors.market;
        let block = |id: &str| t.blocks.iter().find(|b| b.id == id);
        let d_m = |b: &Block| poly_centroid(&b.poly).dist(m);
        // Wall lots are their own one-lot "blocks" and are not faces.
        let max_d = t
            .parcels
            .iter()
            .filter_map(|p| block(&p.par.block))
            .map(d_m)
            .fold(0.0, f64::max);
        let ring_blocks: std::collections::BTreeSet<&str> =
            t.parcels.iter().filter(|p| p.par.courtyard_ring).map(|p| p.par.block.as_str()).collect();
        rung += ring_blocks.len();
        for id in &ring_blocks {
            let b = block(id).expect("a ring lot's block is a face");
            let lots: Vec<_> = t.parcels.iter().filter(|p| p.par.block == *id).collect();
            // Ruling AD: only the outermost ring by market distance.
            assert!(d_m(b) >= max_d * OUTER_RING_FRAC, "{seed}/{site} {id} is not outer");
            // Converted whole: no strip lot is left in a ring block.
            assert!(lots.iter().all(|p| p.par.courtyard_ring), "{seed}/{site} {id} mixes plats");
            // A court is left open — at least the floor, since a dropped
            // corner sliver only adds to it.
            let open = b.area - lots.iter().map(|p| p.par.area).sum::<f64>();
            assert!(open >= COURT_MIN_AREA - 1e-6, "{seed}/{site} {id} court {open}");
        }
        // No ring lot overlaps any other lot of the town, wall lots included.
        let (ring, rest): (Vec<_>, Vec<_>) = t.parcels.iter().partition(|p| p.par.courtyard_ring);
        for p in &ring {
            let c = poly_centroid(&p.par.poly);
            for q in &rest {
                assert!(!point_in_poly(c, &q.par.poly), "{seed}/{site} {} in {}", p.par.id, q.par.id);
                let cq = poly_centroid(&q.par.poly);
                assert!(!point_in_poly(cq, &p.par.poly), "{seed}/{site} {} in {}", q.par.id, p.par.id);
            }
        }
        // Each ring lot carries exactly one building, the whole lot, unless
        // its district keeps its own grammar or a later sweep took it.
        for p in t.parcels.iter().filter(|p| p.par.courtyard_ring) {
            if matches!(p.district, "harbour" | "warehouse" | "oreyard" | "fishery" | "sawyard")
                || p.churchyard
                || p.cleared
            {
                continue;
            }
            assert!(p.built, "{seed}/{site} {} is built", p.par.id);
            let bs: Vec<_> = t.buildings.iter().filter(|b| b.parcel == p.par.id).collect();
            assert_eq!(bs.len(), 1, "{seed}/{site} {}", p.par.id);
            assert!((poly_area(&bs[0].poly).abs() - p.par.area).abs() < 1e-6);
            assert!(bs[0].courtyard);
        }
    }
    assert!(rung >= 10, "the fixtures must reach the stage: {rung} ring blocks");
}

#[test]
fn the_radial_plan_keeps_its_warehouse_belt() {
    for seed in [42u32, 7, 1234] {
        let t = town(seed, "inland", 6000.0, Some("venus"));
        assert!(t.parcels.iter().all(|p| !p.par.courtyard_ring), "venus {seed}");
    }
}

// ------------------------------------------- mutation-sweep fixtures (2026-09-24) --
//
// Written for the survivors of the 2026-09-24 sweep; the table is in
// `URBAN_MORPHOLOGY_SCOPE.md` under "Beyond the reference: owner-ruled
// additions". Hand-built blocks with literal answers, ties made exact and
// asserted exact before use.

fn rect(w: f64, h: f64) -> Vec<Vec2> {
    vec![Vec2::new(0.0, 0.0), Vec2::new(w, 0.0), Vec2::new(w, h), Vec2::new(0.0, h)]
}

#[test]
fn the_dense_court_window_is_80_square_metres_to_35_percent_inclusive() {
    // Clamped deep (22 m) and 34.2% court: under 35%, so still dense.
    assert_eq!(ring_depth(&square(106.0), 0.2), Some(22.0));
    // Clamped shallow (8 m): an 8.97 m court square is 80.46 m², over the floor;
    // an 8.92 m one is 79.57 m², under it.
    assert_eq!(ring_depth(&square(24.97), 0.3), Some(8.0));
    assert_eq!(ring_depth(&square(24.92), 0.3), None);
    // Both ends exactly: a 26 x 24 block's 8 m court is 10 x 8 = 80 m², and a
    // 64 x 30 block's is 48 x 14 = 672 m², exactly 35% of 1920.
    assert_eq!(court_of(&rect(26.0, 24.0), 8.0).map(|c| poly_area(&c)), Some(80.0));
    assert_eq!(ring_depth(&rect(26.0, 24.0), 0.3), Some(8.0), "a court of exactly 80 m² is dense");
    assert_eq!(1920.0 * COURT_MAX_FRAC, 672.0);
    assert_eq!(court_of(&rect(64.0, 30.0), 8.0).map(|c| poly_area(&c)), Some(672.0));
    assert_eq!(ring_depth(&rect(64.0, 30.0), 0.4), Some(8.0), "a court of exactly 35% is dense");
}

#[test]
fn a_target_met_exactly_by_the_deepest_ring_takes_it_without_bisecting() {
    // `court(hi) >= area * target`, inclusive: a 100 m square's 22 m court is
    // 56² = 3136 m², so a target of 0.3136 is met exactly at 22.
    assert_eq!(10000.0 * 0.3136, 3136.0);
    assert_eq!(ring_depth(&square(100.0), 0.3136), Some(22.0));
}

#[test]
fn a_lot_of_exactly_26_square_metres_is_kept() {
    // A 21 m block at 8 m depth, four lots an edge: every lot is a trapezoid of
    // parallel sides 5.25 and 1.25 m, 8 m apart — exactly 26 m².
    let (_, lots) = ring_plat(&square(21.0), 8.0, &mut || 5.25).expect("rings");
    assert_eq!(lots.len(), 16);
    assert!(lots.iter().all(|(_, q)| poly_area(q) == 26.0));
}

fn blk(id: &str, poly: Vec<Vec2>, face_ids: Vec<usize>, plaza: bool) -> Block {
    Block {
        id: id.into(),
        area: poly_area(&poly),
        face_poly: poly.clone(),
        poly,
        face_ids,
        edge_dists: Vec::new(),
        plaza,
    }
}

fn strip(block: &str, poly: &[Vec2]) -> Parcel {
    Parcel {
        id: format!("strip-{block}"),
        poly: poly.to_vec(),
        block: block.into(),
        frontage: 0.0,
        depth: 0.0,
        area: 0.0,
        age: 0.0,
        edge_cls: "street",
        tone: 0.0,
        wall_backing: WallBacking::No,
        courtyard_ring: false,
        gate_quality: None,
    }
}

/// A 60 m square block centred at (100, 0), its four street edges laid in `g`
/// except the west one: south `primary` epoch 2, east `lane` epoch 5, north
/// `ringroad` epoch 9. Returns the block.
fn square_block(g: &mut Graph) -> Block {
    let c = [Vec2::new(70.0, -30.0), Vec2::new(130.0, -30.0), Vec2::new(130.0, 30.0), Vec2::new(70.0, 30.0)];
    g.add_street(c[0].x, c[0].y, c[1].x, c[1].y, "primary", 6.0, 2, "s");
    g.add_street(c[1].x, c[1].y, c[2].x, c[2].y, "lane", 3.0, 5, "e");
    g.add_street(c[2].x, c[2].y, c[3].x, c[3].y, "ringroad", 7.5, 9, "n");
    let ids: Vec<usize> = c.iter().map(|p| g.nearest_node(p.x, p.y, 0.5).expect("a corner node")).collect();
    blk("blk0", c.to_vec(), ids, false)
}

/// The one-cell real-water context: `river_dist` is `d` and `is_water` is
/// `wet` everywhere.
fn uniform_water(d: f64, wet: bool, width: f64) -> crate::site::WaterCtx {
    crate::site::WaterCtx {
        mask: vec![u8::from(wet)],
        dt: vec![d],
        mw: 1,
        mh: 1,
        cell_m: 1.0,
        river_path: None,
        river_width_m: Some(width),
        river_order: 0.0,
        sea_lake_cells: 0.0,
    }
}

/// A real-water site is built with a nominal 12 m band; the fixtures set a
/// 10 m channel so the river margin is 6 m.
fn site_with(kind: &str, w: Option<crate::site::WaterCtx>) -> Site {
    let real = w.is_some();
    let mut s = crate::site::build_site(3, 1700.0, 1250.0, kind, crate::site::SiteOpts { water: w, ..Default::default() });
    if real {
        s.river_w = 10.0;
    }
    s
}

#[test]
fn ring_lots_carry_their_own_streets_class_and_age() {
    let mut g = Graph::new();
    let b = square_block(&mut g);
    let mut parcels = vec![strip("blk0", &b.poly)];
    let site = site_with("landlocked", None);
    let got = build_courtyard_rings(9, &g, std::slice::from_ref(&b), &mut parcels, Vec2::new(0.0, 0.0), 8, &site);
    assert_eq!(got, vec!["blk0".to_string()]);
    assert!(parcels.iter().all(|p| p.courtyard_ring), "the strip lot is gone");
    let side = |i: usize| parcels.iter().filter(|p| {
        let m = p.poly[0].lerp(p.poly[1], 0.5);
        match i {
            0 => m.y == -30.0,
            1 => m.x == 130.0,
            2 => m.y == 30.0,
            _ => m.x == 70.0,
        }
    }).collect::<Vec<_>>();
    for (i, cls, age) in [(0, "primary", 6.0), (1, "lane", 3.0), (2, "ringroad", 0.0), (3, "street", 8.0)] {
        let lots = side(i);
        assert!(!lots.is_empty(), "side {i} is platted");
        for p in lots {
            assert_eq!(p.edge_cls, cls, "side {i}");
            assert_eq!(p.age, age, "side {i}: epochs 8 less the street's epoch, floored at 0; 8 with no street");
        }
    }
    // A lot's depth is the mean of its two sides, street to court.
    for p in side(0) {
        let d = p.poly[3].y - p.poly[0].y;
        assert!(d >= 8.0, "a real ring depth: {d}");
        let sides = p.poly[0].dist(p.poly[3]) + p.poly[1].dist(p.poly[2]);
        assert_eq!(p.depth, sides / 2.0);
        assert!(p.depth >= d);
    }
}

#[test]
fn the_outer_ring_skips_plazas_and_is_inclusive_at_seventy_percent() {
    let mut g = Graph::new();
    let b = square_block(&mut g); // centroid (100, 0): 100 m from the market
    // A plaza 300 m out carrying a lot of its own, and a block at exactly 70 m.
    let far = vec![Vec2::new(270.0, -30.0), Vec2::new(330.0, -30.0), Vec2::new(330.0, 30.0), Vec2::new(270.0, 30.0)];
    let at70 = vec![Vec2::new(-30.0, 40.0), Vec2::new(30.0, 40.0), Vec2::new(30.0, 100.0), Vec2::new(-30.0, 100.0)];
    assert_eq!(crate::geom::poly_centroid(&at70).dist(Vec2::new(0.0, 0.0)), 100.0 * OUTER_RING_FRAC);
    let n0 = b.face_ids[0];
    let blocks = vec![b.clone(), blk("plaza", far.clone(), vec![n0; 4], true), blk("blk70", at70.clone(), vec![n0; 4], false)];
    let mut parcels = vec![strip("blk0", &b.poly), strip("plaza", &far), strip("blk70", &at70)];
    let site = site_with("landlocked", None);
    let got = build_courtyard_rings(9, &g, &blocks, &mut parcels, Vec2::new(0.0, 0.0), 8, &site);
    assert_eq!(got, vec!["blk0".to_string(), "blk70".to_string()], "the plaza sets no extent and is never ringed");
}

#[test]
fn a_block_whose_every_lot_is_a_sliver_keeps_its_strip_plat() {
    // A 60-gon of radius 20: every edge is 2.1 m, every ring lot under 26 m²
    // and dropped. No lots is no conversion, not an empty ring.
    let poly: Vec<Vec2> = (0..60)
        .map(|i| {
            let a = 2.0 * std::f64::consts::PI * i as f64 / 60.0;
            Vec2::new(100.0 + 20.0 * crate::geom::js_cos(a), 20.0 * crate::geom::js_sin(a))
        })
        .collect();
    let mut g = Graph::new();
    g.nodes.push(crate::graph::Node { id: 0, x: 0.0, y: 0.0, adj: Vec::new() });
    let b = blk("blk0", poly.clone(), vec![0; 60], false);
    assert!(ring_plat(&poly, 10.0, &mut || 9.0).is_some_and(|(_, l)| l.is_empty()), "rings, but no lot survives");
    let mut parcels = vec![strip("blk0", &poly)];
    let site = site_with("landlocked", None);
    let got = build_courtyard_rings(9, &g, std::slice::from_ref(&b), &mut parcels, Vec2::new(0.0, 0.0), 8, &site);
    assert!(got.is_empty());
    assert_eq!(parcels.len(), 1, "the strip lot stays");
}

#[test]
fn every_corner_of_every_lot_must_be_dry_by_both_tests() {
    let convert = |site: &Site| {
        let mut g = Graph::new();
        let b = square_block(&mut g);
        let mut parcels = vec![strip("blk0", &b.poly)];
        !build_courtyard_rings(9, &g, std::slice::from_ref(&b), &mut parcels, Vec2::new(0.0, 0.0), 8, site).is_empty()
    };
    // A channel 10 m wide: the margin is riverW/2 + 1 = 6, inclusive.
    assert!(convert(&site_with("river", Some(uniform_water(6.0, false, 10.0)))), "6 m is dry");
    assert!(convert(&site_with("river", Some(uniform_water(6.5, false, 10.0)))), "6.5 m is dry");
    assert!(!convert(&site_with("river", Some(uniform_water(5.5, false, 10.0)))), "5.5 m is not");
    // Anything else: 3 m.
    assert!(convert(&site_with("coast", Some(uniform_water(3.5, false, 10.0)))), "3.5 m from a coast is dry");
    // The mask counts even far from any centreline.
    assert!(!convert(&site_with("coast", Some(uniform_water(100.0, true, 10.0)))), "a masked cell is wet");
    // A block half in the water: its east lots' street corners are wet, their
    // court corners dry. Every lot has a dry corner and some lots are wholly
    // dry, and the block is still refused.
    let half = crate::site::WaterCtx {
        mask: vec![0, 1],
        dt: vec![100.0, 100.0],
        mw: 2,
        mh: 1,
        cell_m: 130.0,
        ..uniform_water(100.0, false, 10.0)
    };
    let site = site_with("coast", Some(half));
    assert!(site.is_water(Vec2::new(130.0, 0.0)) && !site.is_water(Vec2::new(129.0, 0.0)));
    assert!(!convert(&site), "one wet corner refuses the whole block");
}

#[test]
fn the_bisection_moves_down_on_a_midpoint_that_meets_the_target_exactly() {
    // `court(mid) > target` sends `lo` up; a tie sends `hi` down. The first
    // midpoint of (8, 22) is 15, and a 60 m square's 15 m court is exactly a
    // quarter of it — so `hi` becomes 15 and `lo` climbs toward it from below,
    // never reaching it.
    assert_eq!(court_of(&square(60.0), 15.0).map(|c| poly_area(&c)), Some(900.0));
    let d = ring_depth(&square(60.0), 0.25).expect("dense");
    assert!(d < 15.0 && d > 15.0 - 1e-5, "{d}");
}
