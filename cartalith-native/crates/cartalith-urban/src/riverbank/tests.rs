//! The riverbank geometry's gate ([`crate::riverbank`], 2026-11-15). Each
//! test says what it protects. The fixture is a straight river across a real
//! water raster stamped the way `cartalith_civ`'s `finish_water_ctx` stamps
//! one (a band half the width plus half a cell each side of the centreline),
//! with no sea or lake, so every number below is checkable by hand.

use crate::generate::{GenOpts, generate, hash_model};
use crate::geom::Vec2;
use crate::site::{SiteOpts, WaterCtx, build_site};

const CELL: f64 = 22.0;
const MW: usize = 77;
const MH: usize = 57;
/// The river's width in the fixture, metres (`um_water_ctx`'s `10 + 7 * order`
/// at order 2).
const RW: f64 = 24.0;
/// The centreline's y, metres: on a cell boundary, so the stamped band is
/// symmetric.
const RIVER_Y: f64 = 33.0 * CELL;

/// The fixture's water. `open` decides whether the host supplied the
/// open-water raster (all dry: no sea or lake in the box).
fn water(open: bool) -> WaterCtx {
    let band = RW / 2.0 + CELL / 2.0;
    let mut mask = vec![0u8; MW * MH];
    for j in 0..MH {
        let cy = (j as f64 + 0.5) * CELL;
        if (cy - RIVER_Y).abs() <= band {
            for i in 0..MW {
                mask[j * MW + i] = 1;
            }
        }
    }
    // The band is whole rows, so the distance transform is the row distance.
    let wet_rows: Vec<usize> = (0..MH).filter(|&j| mask[j * MW] == 1).collect();
    let mut dt = vec![0.0; MW * MH];
    for j in 0..MH {
        let d = wet_rows.iter().map(|&w| (w as f64 - j as f64).abs()).fold(f64::INFINITY, f64::min);
        for i in 0..MW {
            dt[j * MW + i] = d;
        }
    }
    WaterCtx {
        mask,
        dt,
        mw: MW,
        mh: MH,
        cell_m: CELL,
        river_path: Some((0..=17).map(|k| Vec2::new(k as f64 * 100.0, RIVER_Y)).collect()),
        river_width_m: Some(RW),
        river_order: 2.0,
        sea_lake_cells: 0.0,
        open_water: open.then(|| vec![0u8; MW * MH]),
    }
}

fn opts(riverbank: bool, open: bool) -> GenOpts {
    GenOpts {
        site: Some("river".into()),
        pop: Some(6000.0),
        water: Some(water(open)),
        riverbank,
        ..GenOpts::default()
    }
}

/// The least distance from any building vertex to the channel's edge.
fn built_to_bank(t: &crate::generate::Town) -> f64 {
    t.buildings
        .iter()
        .flat_map(|b| b.poly.iter())
        .map(|v| ((v.y - RIVER_Y).abs() - RW / 2.0).max(0.0))
        .fold(f64::INFINITY, f64::min)
}

/// Protects the switch's off state and its two refusals: with the option off,
/// with no open-water raster, and on a synthetic site, the town is the
/// reference's byte for byte (the same `hashModel`) and reports no riverbank.
#[test]
fn riverbank_changes_nothing_off_without_open_water_or_on_a_synthetic_site() {
    let reference = generate(4242, &opts(false, true));
    assert!(!reference.riverbank && reference.embankment.is_none());
    let no_open = generate(4242, &opts(true, false));
    assert!(!no_open.riverbank, "no open-water raster: nothing is guessed from the stamped mask");
    assert_eq!(hash_model(&no_open), hash_model(&reference));
    let synth_off = generate(4242, &GenOpts { site: Some("river".into()), ..GenOpts::default() });
    let synth_on = generate(4242, &GenOpts { site: Some("river".into()), riverbank: true, ..GenOpts::default() });
    assert!(!synth_on.riverbank);
    assert_eq!(hash_model(&synth_on), hash_model(&synth_off));
}

/// Protects what the site measures with the riverbank geometry: the water is
/// the channel (the centreline within half the width, 12 m here), not the
/// stamped band (23 m), and the river distance is the distance to the
/// centreline (literals, worked from the fixture by hand).
#[test]
fn the_channel_is_the_water_and_distance_is_from_the_centreline() {
    let mut site = build_site(7, 1700.0, 1250.0, "river", SiteOpts { water: Some(water(true)), ..Default::default() });
    let x = 850.0;
    // The reference first: the stamped band is water 20 m out, and a point
    // 40 m out reads the row distance to the band (one cell, 22 m).
    assert!(site.is_water(Vec2::new(x, RIVER_Y + 20.0)));
    assert_eq!(site.river_dist(Vec2::new(x, RIVER_Y + 40.0)), 22.0);
    assert!(site.use_riverbank());
    assert!(site.is_water(Vec2::new(x, RIVER_Y + 11.9)));
    assert!(!site.is_water(Vec2::new(x, RIVER_Y + 12.1)));
    assert!(!site.is_water(Vec2::new(x, RIVER_Y - 12.1)));
    assert_eq!(site.river_dist(Vec2::new(x, RIVER_Y + 40.0)), 40.0);
    assert_eq!(site.river_dist(Vec2::new(x, RIVER_Y - 25.0)), 25.0);
}

/// Protects the open-water half of the riverbank site: an open-water cell is
/// water, and the river distance near it is half the river's width plus the
/// distance to that cell (so every `riverW/2 + x` margin means x from it).
#[test]
fn open_water_is_still_read_off_the_raster() {
    let mut w = water(true);
    let open = w.open_water.as_mut().unwrap();
    // One lake cell well away from the river: column 10, row 5.
    open[5 * MW + 10] = 1;
    let mut site = build_site(7, 1700.0, 1250.0, "river", SiteOpts { water: Some(w), ..Default::default() });
    assert!(site.use_riverbank());
    assert!(site.is_water(Vec2::new(10.5 * CELL, 5.5 * CELL)));
    // Three cells east of it, on the same row: 12 + 3 * 22 = 78 m.
    assert_eq!(site.river_dist(Vec2::new(13.5 * CELL, 5.5 * CELL)), 78.0);
    // Two cells east and two south, two diagonal chamfer steps (the
    // reference's 1.41421): 12 + 2 * 1.41421 * 22 = 74.22524 m.
    assert!((site.river_dist(Vec2::new(12.5 * CELL, 7.5 * CELL)) - 74.22524).abs() < 1e-9);
}

/// Protects `use_riverbank`'s two other refusals: a site whose water has no
/// river path (a shoreline site: `real_river` is false), and an open-water
/// raster of the wrong size, both keep the reference's water.
#[test]
fn riverbank_refuses_a_site_without_a_river_or_with_a_misfit_raster() {
    let no_river = WaterCtx { river_path: None, ..water(true) };
    let mut site = build_site(7, 1700.0, 1250.0, "river", SiteOpts { water: Some(no_river), ..Default::default() });
    assert!(!site.use_riverbank());
    let misfit = WaterCtx { open_water: Some(vec![0u8; 4]), ..water(true) };
    let mut site = build_site(7, 1700.0, 1250.0, "river", SiteOpts { water: Some(misfit), ..Default::default() });
    assert!(!site.use_riverbank());
    assert!(!site.riverbank());
}

/// Protects the defect's fix end to end on the fixture: off, the town stands
/// back from its river further than its widest street (the reference's gap);
/// on, a house stands within a street's width of the bank, an embankment runs
/// along it at exactly half the width plus 1 m plus half of 4 m from the
/// centreline (12 + 1 + 2 = 15 m, a literal), and no building reaches into the
/// channel.
#[test]
fn a_river_town_builds_to_its_bank_and_never_into_the_channel() {
    let off = generate(4242, &opts(false, true));
    let on = generate(4242, &opts(true, true));
    let widest = |t: &crate::generate::Town| {
        t.graph.edges.iter().filter(|e| e.cls == "street").map(|e| e.w).fold(f64::NAN, f64::max)
    };
    let (g_off, g_on) = (built_to_bank(&off), built_to_bank(&on));
    assert!(g_off > widest(&off), "premise: the reference stands back ({g_off} m, street {})", widest(&off));
    assert!(on.riverbank);
    assert!(g_on <= widest(&on), "built to bank {g_on} m against a widest street of {} m", widest(&on));
    let emb = on.embankment.as_ref().expect("an embankment report");
    assert!(!emb.runs.is_empty());
    for run in &emb.runs {
        for v in run {
            assert!(((v.y - RIVER_Y).abs() - 15.0).abs() < 1e-9, "embankment vertex {v:?} not 15 m out");
        }
    }
    assert!(on.graph.edges.iter().any(|e| e.cls == "quay"), "the embankment is in the street graph");
    // Street ends near the bank are tied onto it (three on this fixture).
    assert!(emb.ties >= 1, "no street end was tied to the embankment");
    // No field or pasture strip spans the river (`generate`'s farmland sweep).
    let across = |t: &crate::generate::Town| {
        t.details
            .iter()
            .filter(|d| matches!(d.kind, "field" | "pasture"))
            .filter(|d| match &d.geom {
                crate::hinterland::DetailGeom::Poly(p) => p.iter().any(|v| v.y > RIVER_Y) && p.iter().any(|v| v.y < RIVER_Y),
                _ => false,
            })
            .count()
    };
    assert_eq!(across(&on), 0, "a field strip spans the river");
    for b in &on.buildings {
        let n = b.poly.len();
        for k in 0..n {
            let (a, c) = (b.poly[k], b.poly[(k + 1) % n]);
            assert!((a.y - RIVER_Y).abs() >= RW / 2.0, "building {} vertex in the channel", b.id);
            assert!((a.y > RIVER_Y) == (c.y > RIVER_Y), "building {} spans the river", b.id);
        }
    }
}

/// Protects `WaterCtx::open_water_runs`: it runs over the open-water raster
/// only (one lake cell is one run, the river band none), the mask's own runs
/// are unchanged, and a context with no open-water raster has none.
#[test]
fn open_water_runs_are_the_sea_and_lake_only() {
    let mut w = water(true);
    assert!(w.open_water_runs().is_empty());
    assert_eq!(w.water_runs().len(), 2, "two band rows, each one run");
    w.open_water.as_mut().unwrap()[5 * MW + 10] = 1;
    assert_eq!(w.open_water_runs(), vec![(Vec2::new(220.0, 110.0), Vec2::new(242.0, 132.0))]);
    assert!(water(false).open_water_runs().is_empty());
}

/// Protects `fine_river_path`: a path kilometres past the box keeps its own
/// vertices, and only its stretch inside the box grown by 300 m is resampled
/// every 8 m or less along the same line; a river that leaves the grown box
/// and comes back keeps its real course (no chord across the gap); a path
/// that never comes near the box is refused (literals).
#[test]
fn a_coarse_real_river_is_resampled_near_the_box_and_keeps_its_vertices() {
    let coarse = [Vec2::new(-1233.0, 625.0), Vec2::new(850.0, 625.0), Vec2::new(2933.0, 625.0)];
    let fine = crate::riverbank::fine_river_path(&coarse, 1700.0, 1250.0).expect("the river crosses the box");
    assert_eq!(fine.first().copied(), Some(Vec2::new(-1233.0, 625.0)));
    assert_eq!(fine[1], Vec2::new(-300.0, 625.0));
    assert_eq!(fine[fine.len() - 2], Vec2::new(2000.0, 625.0));
    assert_eq!(fine.last().copied(), Some(Vec2::new(2933.0, 625.0)));
    assert!(fine.contains(&Vec2::new(850.0, 625.0)), "the traced vertex is kept");
    let inner = &fine[1..fine.len() - 1];
    assert!(inner.windows(2).all(|w| w[0].dist(w[1]) <= 8.0 + 1e-9 && w[0].dist(w[1]) > 7.0));
    assert!(fine.iter().all(|p| p.y == 625.0));
    // Out of the grown box to the north and back: the far vertex stays.
    let wander = [Vec2::new(100.0, 600.0), Vec2::new(100.0, -3000.0), Vec2::new(1600.0, -3000.0), Vec2::new(1600.0, 600.0)];
    let w = crate::riverbank::fine_river_path(&wander, 1700.0, 1250.0).expect("near the box");
    assert!(w.contains(&Vec2::new(100.0, -3000.0)) && w.contains(&Vec2::new(1600.0, -3000.0)));
    assert!(w.windows(2).all(|s| !(s[0].y == -300.0 && s[1].y == -300.0 && (s[0].x - s[1].x).abs() > 8.0 + 1e-9)), "no chord along the grown box edge");
    let far = [Vec2::new(-3000.0, 5000.0), Vec2::new(4000.0, 5000.0)];
    assert!(crate::riverbank::fine_river_path(&far, 1700.0, 1250.0).is_none());
}

/// Protects the embankment's wall guard (`crosses_wall`): a segment through a
/// ring crosses it, one inside it does not, and no ring is no wall.
#[test]
fn the_embankment_never_crosses_the_wall() {
    let ring = [Vec2::new(0.0, 0.0), Vec2::new(100.0, 0.0), Vec2::new(100.0, 100.0), Vec2::new(0.0, 100.0)];
    assert!(super::crosses_wall(Vec2::new(50.0, 50.0), Vec2::new(150.0, 50.0), Some(&ring)));
    assert!(!super::crosses_wall(Vec2::new(20.0, 50.0), Vec2::new(80.0, 50.0), Some(&ring)));
    assert!(!super::crosses_wall(Vec2::new(50.0, 50.0), Vec2::new(150.0, 50.0), None));
}

/// Protects `Site::poly_in_water`, the exact guard `generate` sweeps houses
/// on: a 10 m square 0.5 m from the channel's edge is in the water, one 0.5 m
/// clear of it is not; a square whose corner reaches 1 m into an open-water
/// cell is, one 1 m short of the cell is not; and a site the riverbank
/// geometry did not take answers `false` (literals, from the fixture).
#[test]
fn a_house_in_the_water_is_found_exactly() {
    let sq = |x: f64, y: f64| vec![Vec2::new(x, y), Vec2::new(x + 10.0, y), Vec2::new(x + 10.0, y + 10.0), Vec2::new(x, y + 10.0)];
    let mut w = water(true);
    w.open_water.as_mut().unwrap()[5 * MW + 10] = 1; // the cell x 220..242, y 110..132
    let mut site = build_site(7, 1700.0, 1250.0, "river", SiteOpts { water: Some(w.clone()), ..Default::default() });
    let edge = RIVER_Y + RW / 2.0;
    assert!(!site.poly_in_water(&sq(850.0, edge - 0.5)), "not a riverbank site yet");
    assert!(site.use_riverbank());
    assert!(site.poly_in_water(&sq(850.0, edge - 0.5)));
    assert!(!site.poly_in_water(&sq(850.0, edge + 0.5)));
    assert!(site.poly_in_water(&sq(211.0, 101.0)), "its corner is 1 m inside the cell");
    assert!(!site.poly_in_water(&sq(209.0, 99.0)), "1 m short of the cell");
    // A 5 m house wholly inside the cell: no edge crosses it, no cell corner
    // is inside the house; only its own vertices say so.
    let small = vec![Vec2::new(228.0, 118.0), Vec2::new(233.0, 118.0), Vec2::new(233.0, 123.0), Vec2::new(228.0, 123.0)];
    assert!(site.poly_in_water(&small));
}

/// Protects `generate`'s farmland sweep on a riverbank site: on the fixture at
/// seed 4 and 3 000 people (found by scanning seeds 1-399, the first that lays
/// a strip across the river), field strips that span the channel are swept,
/// and none is left spanning it.
#[test]
fn a_field_strip_across_the_river_is_swept() {
    let t = generate(4, &GenOpts { pop: Some(3000.0), ..opts(true, true) });
    let e = t.embankment.as_ref().expect("an embankment report");
    assert!(e.fields_swept >= 1, "premise: this town laid a strip across the river");
    let across = t
        .details
        .iter()
        .filter(|d| matches!(d.kind, "field" | "pasture"))
        .filter(|d| match &d.geom {
            crate::hinterland::DetailGeom::Poly(p) => p.iter().any(|v| v.y > RIVER_Y) && p.iter().any(|v| v.y < RIVER_Y),
            _ => false,
        })
        .count();
    assert_eq!(across, 0);
}

/// Protects the embankment's bank test (`dry_on_side`): a 200 m segment from
/// 100 m north of the centreline to 100 m south has every one of its six
/// probes clear of the 24 m channel (the nearest are 20 m out), so only the
/// side check refuses it; a segment along one bank is accepted.
#[test]
fn the_embankment_never_crosses_to_the_other_bank() {
    let mut site = build_site(7, 1700.0, 1250.0, "river", SiteOpts { water: Some(water(true)), ..Default::default() });
    assert!(site.use_riverbank());
    assert!(!super::dry_on_side(&site, Vec2::new(850.0, RIVER_Y - 100.0), Vec2::new(850.0, RIVER_Y + 100.0), -1.0));
    assert!(!super::dry_on_side(&site, Vec2::new(850.0, RIVER_Y - 100.0), Vec2::new(850.0, RIVER_Y + 100.0), 1.0));
    let side = super::side_of(&site, Vec2::new(850.0, RIVER_Y + 15.0));
    assert!(super::dry_on_side(&site, Vec2::new(800.0, RIVER_Y + 15.0), Vec2::new(900.0, RIVER_Y + 15.0), side));
    // 14 m out: dry ground, but a 4 m street there would put its paved edge
    // within 1 m of the 12 m channel edge (12 + 1 + 2 = 15 m needed).
    assert!(!super::dry_on_side(&site, Vec2::new(800.0, RIVER_Y + 14.0), Vec2::new(900.0, RIVER_Y + 14.0), side));
}

/// A riverbank site on the fixture with its centreline resampled, as
/// `generate` builds one, and a street graph of the given dead-end streets.
fn bank_site_and_streets(streets: &[(Vec2, Vec2)]) -> (crate::site::Site, crate::graph::Graph) {
    let mut w = water(true);
    w.river_path = crate::riverbank::fine_river_path(w.river_path.as_deref().unwrap(), 1700.0, 1250.0);
    let mut site = build_site(7, 1700.0, 1250.0, "river", SiteOpts { water: Some(w), ..Default::default() });
    assert!(site.use_riverbank());
    let mut g = crate::graph::Graph::new();
    for (a, b) in streets {
        g.add_street(a.x, a.y, b.x, b.y, "street", 4.0, 0, "test");
    }
    (site, g)
}

/// Protects both wall guards in `build_embankment` (literals): with a wall
/// whose side crosses the bank line, no embankment segment crosses it; with a
/// wall whose bottom lies between a street end and the bank, the tie is not
/// laid, though without a wall it is.
#[test]
fn neither_the_embankment_nor_a_tie_crosses_the_wall() {
    let r = RIVER_Y;
    let street = [(Vec2::new(600.0, r + 100.0), Vec2::new(600.0, r + 20.0))];
    let cross = |g: &crate::graph::Graph, ring: &[Vec2], cls: &str| {
        g.edges.iter().filter(|e| e.alive && e.cls == cls).any(|e| {
            super::crosses_wall(g.nodes[e.a].pt(), g.nodes[e.b].pt(), Some(ring))
        })
    };
    // No wall: an embankment and one tie.
    let (site, mut g) = bank_site_and_streets(&street);
    let e = super::build_embankment(&site, &mut g, None);
    assert!(!e.runs.is_empty() && e.ties == 1, "premise: {e:?}");
    // A wall across the bank line.
    let ring_a = [Vec2::new(605.0, r + 12.0), Vec2::new(640.0, r + 12.0), Vec2::new(640.0, r + 60.0), Vec2::new(605.0, r + 60.0)];
    let (site, mut g) = bank_site_and_streets(&street);
    super::build_embankment(&site, &mut g, Some(&ring_a));
    assert!(!cross(&g, &ring_a, "quay"), "an embankment segment crosses the wall");
    // A wall between the street end and the bank.
    let ring_b = [Vec2::new(560.0, r + 17.0), Vec2::new(640.0, r + 17.0), Vec2::new(640.0, r + 60.0), Vec2::new(560.0, r + 60.0)];
    let (site, mut g) = bank_site_and_streets(&street);
    let e = super::build_embankment(&site, &mut g, Some(&ring_b));
    assert_eq!(e.ties, 0, "the tie would cross the wall");
}
