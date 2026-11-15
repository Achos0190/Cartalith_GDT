//! Ruling FE item 2's street village ([`super::plat_tofts`] and the
//! `generate` path it rides): hand-built roads whose answers are known
//! without the code under test, then generated villages against literal bars
//! measured on these exact fixtures on 2026-11-15 (no reference to diff:
//! this is not a port).

use super::*;
use crate::generate::{GenOpts, Town, generate};
use crate::site::{SiteOpts, build_site};

/// A landlocked synthetic site with no water, the box `generate` uses.
fn dry_site() -> Site {
    build_site(7, 1700.0, 1250.0, "landlocked", SiteOpts::default())
}

/// One straight east-west primary across the box through (850, 625), 6 m
/// wide, as `build_primaries` lays one.
fn one_road() -> Graph {
    let mut g = Graph::new();
    g.add_street(250.0, 625.0, 1450.0, 625.0, "primary", 6.0, 0, "test road");
    g
}

const MARKET: Vec2 = Vec2::new(850.0, 625.0);

/// Protects: the tofts stopping at the target and lining the road from the
/// market outward. 104 people are 20 households (5.2 each), so 20 tofts;
/// every one fronts the road at its building line (half the road's 6 m, plus
/// the 1.4 m verge), is `TOFT_DEPTH_M` (30 m) deep on open ground, and stands
/// within 120 m of the market (20 plots of about 11 m, ten a side).
#[test]
fn tofts_line_the_road_nearest_the_market_until_the_target_is_housed() {
    let (g, site) = (one_road(), dry_site());
    let t = plat_tofts(1, &g, &site, &[], &[], MARKET, 200.0, 104.0, 8, false);
    assert_eq!(t.len(), 20);
    for p in &t {
        let front = (p.poly[0].y - 625.0).abs();
        assert!((front - 4.4).abs() < 1e-9, "front line {front} m off the centreline");
        assert!((p.depth - 30.0).abs() < 1e-9, "depth {}", p.depth);
        assert!(poly_centroid(&p.poly).dist(MARKET) <= 120.0, "toft at {:?}", poly_centroid(&p.poly));
        assert_eq!(p.edge_cls, "primary");
    }
    let north = t.iter().filter(|p| p.poly[0].y < 625.0).count();
    assert_eq!(north, 10, "both sides of the road, evenly");
    // The frontages are the platting's own: a median about 11 m.
    let mut fr: Vec<f64> = t.iter().map(|p| p.frontage).collect();
    fr.sort_by(f64::total_cmp);
    assert!((9.0..=13.5).contains(&fr[fr.len() / 2]), "median frontage {}", fr[fr.len() / 2]);
}

/// Protects: no toft for a village the block lots already house, and none
/// past `TOFT_REACH_RF` (3) times `max_rf` from the market however many are
/// wanted: 150 m of front either way at `max_rf` 50 m.
#[test]
fn tofts_stop_at_the_target_and_at_their_reach() {
    let (g, site) = (one_road(), dry_site());
    assert!(plat_tofts(1, &g, &site, &[], &[], MARKET, 200.0, 0.0, 8, false).is_empty());
    let t = plat_tofts(1, &g, &site, &[], &[], MARKET, 50.0, 5000.0, 8, false);
    let far = t.iter().map(|p| poly_centroid(&p.poly).dist(MARKET)).fold(0.0, f64::max);
    assert!((130.0..=150.0).contains(&far), "farthest toft {far:.0} m");
}

/// Protects: a toft cut back to a street behind it, never across it. A lane
/// 20 m north of the road, 4 m wide: the northern tofts are 20 - 3 - 1.4
/// (the road's building line) - 2 - 1.4 (the lane's) = 12.2 m deep.
#[test]
fn a_toft_stops_at_the_street_behind_it() {
    let (mut g, site) = (one_road(), dry_site());
    g.add_street(600.0, 605.0, 1100.0, 605.0, "lane", 4.0, 1, "test lane");
    let t = plat_tofts(1, &g, &site, &[], &[], MARKET, 200.0, 52.0, 8, false);
    let north: Vec<&Parcel> = t.iter().filter(|p| p.poly[0].y < 625.0 && p.poly[0].y > 605.0).collect();
    assert!(!north.is_empty());
    for p in north {
        assert!((p.depth - 12.2).abs() < 1e-6, "depth {}", p.depth);
    }
}

/// Protects: a toft never laid over a lot already platted. A lot covering the
/// road's south frontage either side of the market takes those frontages, so
/// every toft there is north of the road.
#[test]
fn a_toft_never_overlaps_a_lot() {
    let (g, site) = (one_road(), dry_site());
    let lot = Parcel {
        id: "b0".into(),
        poly: vec![Vec2::new(700.0, 630.0), Vec2::new(1000.0, 630.0), Vec2::new(1000.0, 660.0), Vec2::new(700.0, 660.0)],
        block: "b".into(),
        frontage: 300.0,
        depth: 30.0,
        area: 9000.0,
        age: 1.0,
        edge_cls: "primary",
        tone: 0.5,
        wall_backing: WallBacking::No,
        courtyard_ring: false,
        gate_quality: None,
    };
    let t = plat_tofts(1, &g, &site, &[], std::slice::from_ref(&lot), MARKET, 200.0, 5.2 * 11.0, 8, false);
    assert_eq!(t.len(), 10, "the big lot counts as one household");
    for p in &t {
        let c = poly_centroid(&p.poly);
        assert!(!(c.y > 625.0 && c.x > 690.0 && c.x < 1010.0), "a toft over the lot at {c:?}");
    }
}

fn village(seed: u32, site: &str, pop: f64, sv: bool) -> Town {
    // Unwalled, as the app's wall ladder leaves almost every village
    // (`cartalith_civ::military::um_wall_spec`; 2 of 650 at 384 on the census).
    generate(seed, &GenOpts { pop: Some(pop), site: Some(site.into()), walls: Some(false), street_village: sv, ..GenOpts::default() })
}


/// The fixtures: three synthetic villages, one per site kind the engine
/// generates without a map.
const VILLAGES: [(u32, &str, f64); 3] = [(11, "landlocked", 400.0), (12, "river", 600.0), (13, "coast", 900.0)];

/// Protects: the street village's three measures on the synthetic sites,
/// against the reference's village from the same seed. Measured 2026-11-15
/// (see the assertion messages for the figures): housed within 80-125 % of
/// target; the green's centre within 60 m of the market point; less lane per
/// building than the reference's village; and tofts on two of the three (the
/// river fixture's block lots house it on their own).
#[test]
fn a_street_village_houses_its_target_with_its_green_at_the_market() {
    let mut tofted = 0;
    for (seed, site, pop) in VILLAGES {
        let (off, on) = (village(seed, site, pop, false), village(seed, site, pop, true));
        let share = on.pop / on.pop_target;
        assert!((0.8..=1.25).contains(&share), "{site}: housed {:.0} %", 100.0 * share);
        let green = on.plaza.as_ref().expect("a green").center;
        assert!(green.dist(on.anchors.market) <= 60.0, "{site}: green {:.0} m from the market", green.dist(on.anchors.market));
        // Lanes and streets per building, the primaries left out: the
        // synthetic box's primaries are the same 1.4-3.6 km either way and
        // the reference's village is over-built (169 % on the river fixture),
        // so the whole-network figure does not compare like with like.
        let per = |t: &Town| {
            let n = &t.graph.nodes;
            let m: f64 = t.graph.edges.iter().filter(|e| e.alive && e.cls != "primary").map(|e| n[e.a].pt().dist(n[e.b].pt())).sum();
            m / t.buildings.len().max(1) as f64
        };
        assert!(per(&on) < per(&off), "{site}: lane per building {:.1} -> {:.1}", per(&off), per(&on));
        tofted += usize::from(on.parcels.iter().any(|p| p.par.id.starts_with("toft")));
    }
    assert!(tofted >= 2, "tofts in {tofted} of 3");
}

/// Protects: the lanes a street village grows: about `STREET_PER_HEAD_M`
/// (0.25 m) a head with no floor, and each started within `ROAD_REACH_M`
/// (40 m) of a primary. The grown edges (`epoch` 1 and up, neither primary
/// nor the green's) total under 0.5 m a head on every fixture (measured
/// 2026-11-15; the reference's floor alone is 1 600 m), and the reference's
/// village grows several times more.
#[test]
fn a_street_village_grows_few_lanes_and_only_by_its_road() {
    for (seed, site, pop) in VILLAGES {
        let grown = |t: &Town| -> f64 {
            let n = &t.graph.nodes;
            t.graph.edges.iter().filter(|e| e.alive && e.epoch >= 1 && e.cls != "primary").map(|e| n[e.a].pt().dist(n[e.b].pt())).sum()
        };
        let (off, on) = (village(seed, site, pop, false), village(seed, site, pop, true));
        assert!(grown(&on) <= 0.5 * pop, "{site}: grown {:.0} m for {pop}", grown(&on));
        assert!(grown(&off) >= 3.0 * grown(&on).max(1.0), "{site}: off {:.0} m, on {:.0} m", grown(&off), grown(&on));
    }
}

/// Protects: the switch reaching villages only. A town of 5 000 is laid out
/// identically with it on and off (the chartered-town line is
/// `PLAZA_MARKET_POP`, 1 500).
#[test]
fn a_town_is_not_a_street_village() {
    for site in ["landlocked", "river"] {
        let (off, on) = (village(21, site, 5000.0, false), village(21, site, 5000.0, true));
        assert_eq!(off.parcels.len(), on.parcels.len(), "{site}");
        assert_eq!(off.graph.edges.len(), on.graph.edges.len(), "{site}");
        assert_eq!(off.pop, on.pop, "{site}");
    }
    // Just under the line it is one.
    let v = village(21, "landlocked", 1499.0, true);
    assert!(v.parcels.iter().any(|p| p.par.id.starts_with("toft")));
}

/// Protects the lanes staying by the road (`ROAD_REACH_M`, 40 m) and the
/// village keeping strip lots, not the towns' courtyard ranges (Ruling FE
/// item 2). Every grown edge's nearer end stands within 40 m of a primary
/// (measured 0-34 m; the reference's villages from the same seeds, 129-184 m),
/// and no lot is a courtyard range (the reference's: 10 and 42 on the
/// landlocked and river fixtures).
#[test]
fn a_street_village_keeps_its_lanes_by_the_road_and_no_courtyards() {
    let worst = |t: &Town| -> f64 {
        let n = &t.graph.nodes;
        let prim: Vec<_> = t.graph.edges.iter().filter(|e| e.alive && e.cls == "primary").collect();
        t.graph
            .edges
            .iter()
            .filter(|e| e.alive && e.epoch >= 1 && e.cls != "primary")
            .map(|e| {
                [n[e.a].pt(), n[e.b].pt()]
                    .iter()
                    .map(|p| prim.iter().map(|q| crate::geom::dist_pt_seg(*p, n[q.a].pt(), n[q.b].pt())).fold(f64::INFINITY, f64::min))
                    .fold(f64::INFINITY, f64::min)
            })
            .fold(0.0, f64::max)
    };
    let mut off_courts = 0;
    for (seed, site, pop) in VILLAGES {
        let (off, on) = (village(seed, site, pop, false), village(seed, site, pop, true));
        assert!(worst(&on) <= 40.0 + 1e-9, "{site}: a lane starts {:.0} m from the road", worst(&on));
        assert!(worst(&off) > 100.0, "{site}: premise, the reference's lanes wander ({:.0} m)", worst(&off));
        assert_eq!(on.parcels.iter().filter(|p| p.par.courtyard_ring).count(), 0, "{site}");
        off_courts += off.parcels.iter().filter(|p| p.par.courtyard_ring).count();
    }
    assert!(off_courts > 0, "premise: the reference's villages have courtyard ranges");
}

