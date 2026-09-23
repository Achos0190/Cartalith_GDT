//! The citadel's own acceptance bar. There is no reference to diff against
//! (`citadel.rs`'s header), so these assert the owner's plan as properties of
//! real generated towns, not as values the code produced.

use super::*;
use crate::generate::{GenOpts, Town, generate};
use crate::geom::point_in_poly;
use crate::growth::dist_to_line;

const SEED: u32 = 1234;

fn town(pop: f64, f: impl FnOnce(&mut GenOpts)) -> Town {
    let mut o = GenOpts { pop: Some(pop), site: Some("inland".into()), ..GenOpts::default() };
    f(&mut o);
    generate(SEED, &o)
}

#[test]
fn only_the_largest_tier_of_walled_organic_curtain_towns_gets_one() {
    // The boundary, as literals: 10 000 qualifies and 9 999 does not.
    assert!(town(10000.0, |_| {}).citadel.is_some(), "10 000 is the tier");
    assert!(town(9999.0, |_| {}).citadel.is_none(), "9 999 is below it");
    assert!(town(12000.0, |_| {}).citadel.is_some());
    // No circuit, no citadel.
    assert!(town(12000.0, |o| o.walls = Some(false)).citadel.is_none());
    // A bastioned trace is already the state's fortification.
    let fort = town(12000.0, |o| o.fortified = true);
    assert!(fort.fortified && fort.citadel.is_none());
    // The designed radial plan is not a castle town.
    assert!(town(12000.0, |o| o.culture = Some("venus".into())).citadel.is_none());
}

#[test]
fn the_citadel_sits_astride_the_curtain_with_an_inner_gate_a_keep_and_a_court() {
    let mut built = 0;
    let sites = ["inland", "river", "riverthrough", "coast", "estuary"];
    let fixtures = [SEED, 42, 7, 99, 3, 11, 2024, 555]
        .iter()
        .flat_map(|&s| sites.iter().map(move |&k| (s, k)));
    for (seed, site) in fixtures {
        let o = GenOpts { pop: Some(14000.0), site: Some(site.into()), ..GenOpts::default() };
        let t = generate(seed, &o);
        let Some(c) = t.citadel.as_ref() else {
            // Refusing is legal (no dry, gate-free, primary-free stretch); the
            // first fixture must not refuse, or this test proves nothing.
            assert_ne!((seed, site), (SEED, "inland"), "the anchor fixture must get a citadel");
            continue;
        };
        built += 1;
        let name = format!("seed {seed} {site}");
        let ring = t.wall.ring.as_ref().expect("a citadel needs a circuit");
        let arc = t.wall.land_arc.as_ref().expect("and a drawn curtain");

        // Astride: the two inner corners inside the town, the two outer ones out.
        assert_eq!(c.wall.len(), 4, "{name}");
        assert!(point_in_poly(c.wall[0], ring) && point_in_poly(c.wall[1], ring), "{name}: inner face inside");
        assert!(!point_in_poly(c.wall[2], ring) && !point_in_poly(c.wall[3], ring), "{name}: outer face outside");
        assert!(arc.contains(&c.on_curtain), "{name}: centred across a vertex of the drawn curtain");
        let w = c.wall[0].dist(c.wall[1]);
        let d = c.wall[1].dist(c.wall[2]);
        assert!((90.0..=120.0).contains(&w) && (80.0..=100.0).contains(&d), "{name}: {w} x {d}");

        // Towers at the four corners and where its walls meet the curtain.
        assert_eq!(&c.towers[..4], &c.wall[..], "{name}");
        let junctions = &c.towers[4..];
        assert!(junctions.len() >= 2, "{name}: {} wall junctions", junctions.len());
        let mut closed = ring.clone();
        closed.push(ring[0]);
        for q in junctions {
            assert!(dist_to_line(*q, &closed) < 1e-6, "{name}: junction tower off the curtain");
        }

        // The inner gate: mid inner face, inside the town, with a live street
        // from it to a junction inside the circuit.
        assert!(c.gate.dist(c.wall[0].lerp(c.wall[1], 0.5)) < 1e-9, "{name}");
        assert!(point_in_poly(c.gate, ring), "{name}: gate is inside the town");
        assert_eq!(c.approach[0], c.gate, "{name}");
        assert!(point_in_poly(c.approach[1], ring), "{name}: approach starts inside");
        let reaches_gate = t.graph.edges.iter().any(|e| {
            t.graph.nodes[e.a].pt().dist(c.gate) < 11.0 || t.graph.nodes[e.b].pt().dist(c.gate) < 11.0
        });
        assert!(reaches_gate, "{name}: a live street reaches the gate");

        // The keep against the outer face, the open court toward the town.
        let centre = t.wall.centroid.expect("set with the ring");
        for q in c.keep.iter().chain(&c.court) {
            assert!(point_in_poly(*q, &c.wall), "{name}: keep and court inside the enclosure");
        }
        let kc = poly_centroid(&c.keep);
        let cc = poly_centroid(&c.court);
        assert!(kc.dist(centre) > cc.dist(centre), "{name}: keep outward of the court");
        // They share an edge at most: along the outward axis the keep starts
        // where the court ends.
        let n = (c.wall[3] - c.wall[0]).norm();
        let keep_near = c.keep.iter().map(|q| q.dot(n)).fold(f64::INFINITY, f64::min);
        let court_far = c.court.iter().map(|q| q.dot(n)).fold(f64::NEG_INFINITY, f64::max);
        assert!(keep_near >= court_far - 1e-6, "{name}: court is open ground, {keep_near} < {court_far}");
        assert!(crate::geom::poly_area(&c.court).abs() > 2000.0, "{name}: an open court");
        let keep_area = crate::geom::poly_area(&c.keep).abs();
        assert!(keep_area > 1000.0, "{name}: one LARGE building, {keep_area} m2");

        // Clear of every town gate, every primary, and nothing built inside.
        for g in &t.wall.gates {
            assert!(g.pt.dist(c.on_curtain) >= w / 2.0 + 40.0, "{name}: on a gate road");
        }
        for e in &t.graph.edges {
            let (a, b) = (t.graph.nodes[e.a].pt(), t.graph.nodes[e.b].pt());
            assert!(!point_in_poly(a.lerp(b, 0.5), &c.wall), "{name}: a street runs inside");
            if e.cls == "primary" {
                assert!(!crosses_rect(a, b, &c.wall), "{name}: a primary crosses it");
            }
        }
        for b in &t.buildings {
            assert!(b.poly.iter().all(|q| !point_in_poly(*q, &c.wall)), "{name}: {} inside", b.id);
        }
        for p in &t.parcels {
            if point_in_poly(poly_centroid(&p.par.poly), &c.wall) {
                assert!(p.cleared, "{name}: lot {} under the citadel not cleared", p.par.id);
            }
        }
    }
    // Every one of the 40 got one (measured 2026-09-23): each site kind
    // reaches every assertion above, eight seeds apiece.
    assert_eq!(built, 40);
}

#[test]
fn the_sweep_reports_descending_and_by_footprint() {
    let sq = |x: f64, y: f64| {
        vec![Vec2::new(x, y), Vec2::new(x + 4.0, y), Vec2::new(x + 4.0, y + 4.0), Vec2::new(x, y + 4.0)]
    };
    let c = Citadel {
        wall: sq(0.0, 0.0).iter().map(|p| *p * 25.0).collect(), // 0..100 square
        towers: vec![],
        tower_r: TOWER_R,
        keep: vec![],
        court: vec![],
        gate: Vec2::new(50.0, 0.0),
        approach: vec![Vec2::new(50.0, 0.0), Vec2::new(50.0, -60.0)],
        on_curtain: Vec2::new(50.0, 20.0),
        prov: PROV,
    };
    let buildings = vec![
        sq(10.0, 10.0),   // inside
        sq(200.0, 200.0), // far
        sq(98.0, 50.0),   // one vertex inside
        sq(49.0, -30.0),  // straddles the approach
        sq(60.0, -30.0),  // 10 m off the approach
    ];
    let parcels = vec![sq(10.0, 10.0), sq(98.0, 50.0), sq(150.0, 0.0)];
    let details = vec![Some(Vec2::new(5.0, 5.0)), None, Some(Vec2::new(-5.0, 5.0))];
    let s = citadel_sweep(&c, &buildings, &parcels, &details);
    assert_eq!(s.buildings_removed, vec![3, 2, 0]);
    // Lot 1's centroid is at x=100, on the edge — not inside.
    assert_eq!(s.parcels_cleared, vec![0]);
    assert_eq!(s.details_removed, vec![0]);
}

