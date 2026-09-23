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

