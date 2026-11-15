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
    // Since Rulings FB and EZ item 1 (2026-11-15) the town grows on both banks
    // and its riverside mixes quays with houses on the water, so which stretch
    // a given seed's town reaches is a property of its growth: the bar is over
    // twelve seeds (4240-4251, measured 8 of 12 within the widest street, the
    // rest 4.9-6.4 m), not this one seed (6.2 m).
    let within = (4240..4252u32)
        .filter(|&s| {
            let t = generate(s, &opts(true, true));
            built_to_bank(&t) <= widest(&t)
        })
        .count();
    assert!(within >= 8, "{within} of 12 towns have a house within a street's width of the bank");
    let _ = g_on;
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
    let e = super::build_embankment(&site, &mut g, None, None);
    assert!(!e.runs.is_empty() && e.ties >= 1, "premise: {e:?}");
    // A wall across the bank line.
    let ring_a = [Vec2::new(605.0, r + 12.0), Vec2::new(640.0, r + 12.0), Vec2::new(640.0, r + 60.0), Vec2::new(605.0, r + 60.0)];
    let (site, mut g) = bank_site_and_streets(&street);
    super::build_embankment(&site, &mut g, Some(&ring_a), None);
    assert!(!cross(&g, &ring_a, "quay"), "an embankment segment crosses the wall");
    // A wall between the street end and the bank.
    let ring_b = [Vec2::new(560.0, r + 17.0), Vec2::new(640.0, r + 17.0), Vec2::new(640.0, r + 60.0), Vec2::new(560.0, r + 60.0)];
    let (site, mut g) = bank_site_and_streets(&street);
    let e = super::build_embankment(&site, &mut g, Some(&ring_b), None);
    assert_eq!(e.ties, 0, "the tie would cross the wall");
}

// ------------------------------------------------ Rulings FB and EZ item 1 --

/// The bank of point `p` on the fixture: `true` north of the centreline.
fn north(p: Vec2) -> bool {
    p.y < RIVER_Y
}

/// The share of a town's houses on its smaller bank (centroid by vertex mean).
fn minority_share(t: &crate::generate::Town) -> f64 {
    let n = t.buildings.len().max(1);
    let k = t
        .buildings
        .iter()
        .filter(|b| north(b.poly.iter().fold(Vec2::new(0.0, 0.0), |a, v| a + *v) * (1.0 / b.poly.len() as f64)))
        .count();
    k.min(t.buildings.len() - k) as f64 / n as f64
}

/// Protects Ruling FB end to end on the fixture, and its size class: a town
/// of 6 000, which its first layout houses under target, is laid out again
/// with a far-bank pass, has a crossing at its bridge point and houses on
/// both banks (each bank at least 10 % of them); a town of
/// `BOTH_BANKS_MIN_POP` (1 500, a literal) is eligible (`both_banks`) and one
/// of 1 499 is not, and gets no bridge; with the switch off neither is a big
/// one.
#[test]
fn a_big_river_town_bridges_its_river_and_builds_on_both_banks() {
    assert!(generate(4242, &GenOpts { pop: Some(1500.0), ..opts(true, true) }).both_banks);
    let big = generate(4242, &opts(true, true));
    assert!(matches!(big.riverbank_pass, Some(super::RiverbankPass::FarBank { .. })), "{:?}", big.riverbank_pass);
    assert!(matches!(big.town_bridge, Some(super::TownBridge::Laid) | Some(super::TownBridge::Existing)), "{:?}", big.town_bridge);
    let crosses = big.graph.edges.iter().any(|e| {
        let (a, b) = (big.graph.nodes[e.a].pt(), big.graph.nodes[e.b].pt());
        e.cls == "primary" && north(a) != north(b)
    });
    assert!(crosses, "no road spans the river");
    assert!(minority_share(&big) >= 0.10, "smaller bank holds {:.3} of the houses", minority_share(&big));
    let village = generate(4242, &GenOpts { pop: Some(1499.0), ..opts(true, true) });
    assert!(!village.both_banks);
    assert_eq!(village.town_bridge, None);
    let off = generate(4242, &GenOpts { pop: Some(6000.0), ..opts(false, true) });
    assert_eq!(off.town_bridge, None);
}

/// Protects the 6 000-person fixture town's far bank against the reference's
/// bridgehead: with the switch on its smaller bank holds more of its houses
/// than the reference's town, which confines the far bank to 190 m of the
/// bridge, and the town houses at least as many (literal bar: 10 %).
#[test]
fn the_far_bank_grows_past_the_bridgehead() {
    let on = generate(4242, &opts(true, true));
    let off = generate(4242, &opts(false, true));
    assert!(minority_share(&on) >= 0.10, "on: {:.3}", minority_share(&on));
    assert!(minority_share(&on) > minority_share(&off), "on {:.3} off {:.3}", minority_share(&on), minority_share(&off));
}

/// Protects `far_bank_buildable` (literals worked from the fixture's flat
/// analytic relief, slope under 0.1): 15 m from the centreline the flood
/// factor is 15 / 42 and the ground is refused (a marsh bank); 100 m out it is
/// accepted.
#[test]
fn a_marsh_bank_is_not_buildable_and_dry_ground_is() {
    let mut site = build_site(7, 1700.0, 1250.0, "river", SiteOpts { water: Some(water(true)), ..Default::default() });
    assert!(site.use_riverbank());
    assert!(!super::far_bank_buildable(&site, Vec2::new(850.0, RIVER_Y + 15.0)));
    assert!(super::far_bank_buildable(&site, Vec2::new(850.0, RIVER_Y + 100.0)));
}

/// Protects `lay_town_bridge`'s three outcomes on the fixture: with a street
/// on the north bank it lays a 6 m `primary` across the bridge point, joined
/// to that street, and a far approach on the south bank; with a road already
/// across the river it adds nothing (`Existing`); with the bridge point
/// beyond `max_r` of the market it lays nothing (`None`).
#[test]
fn a_town_bridge_is_laid_joined_or_refused() {
    let mut w = water(true);
    w.river_path = crate::riverbank::fine_river_path(w.river_path.as_deref().unwrap(), 1700.0, 1250.0);
    let mut site = build_site(7, 1700.0, 1250.0, "river", SiteOpts { water: Some(w), ..Default::default() });
    assert!(site.use_riverbank());
    let bp = site.bridge_pt.expect("a river site has a bridge point");
    let anchors = crate::routes::Anchors { market: Vec2::new(bp.x, RIVER_Y - 120.0), ..crate::routes::place_anchors(7, &site) };
    let mut g = crate::graph::Graph::new();
    g.add_street(bp.x - 100.0, RIVER_Y - 80.0, bp.x + 100.0, RIVER_Y - 80.0, "street", 4.0, 0, "test");
    assert_eq!(super::lay_town_bridge(&site, &anchors, &mut g, 400.0), super::TownBridge::Laid);
    let span = g.edges.iter().find(|e| e.cls == "primary").expect("a span");
    let (a, b) = (g.nodes[span.a].pt(), g.nodes[span.b].pt());
    assert!(north(a) != north(b) && span.w == 6.0);
    // The half-length is `riverW/2 + 16` = 28 m each side of the centreline.
    assert!(((a.y - RIVER_Y).abs() - 28.0).abs() < 1e-6 && ((b.y - RIVER_Y).abs() - 28.0).abs() < 1e-6);
    let streets: Vec<(Vec2, Vec2)> = g.edges.iter().filter(|e| e.cls == "street").map(|e| (g.nodes[e.a].pt(), g.nodes[e.b].pt())).collect();
    assert!(streets.iter().any(|(p, q)| !north(*p) && !north(*q)), "no far-bank approach");
    assert!(streets.iter().any(|(p, q)| (p.y - (RIVER_Y - 28.0)).abs() < 1e-6 || (q.y - (RIVER_Y - 28.0)).abs() < 1e-6), "the near end is not joined");
    // Again on the graph that now crosses: nothing more is laid.
    let edges = g.edges.len();
    assert_eq!(super::lay_town_bridge(&site, &anchors, &mut g, 400.0), super::TownBridge::Existing);
    assert_eq!(g.edges.len(), edges);
    // A bridge point 120 m from the market, with `max_r` 100 m: refused.
    let mut g = crate::graph::Graph::new();
    g.add_street(bp.x - 100.0, RIVER_Y - 80.0, bp.x + 100.0, RIVER_Y - 80.0, "street", 4.0, 0, "test");
    assert_eq!(super::lay_town_bridge(&site, &anchors, &mut g, 100.0), super::TownBridge::None);
    assert!(g.edges.iter().all(|e| e.cls != "primary"));
    // A primary already crossing 25 m from the bridge point counts (within
    // the span's half-length `riverW/2 + 16` = 28 m); one 35 m away does not,
    // and the bridge is laid.
    let mut g = crate::graph::Graph::new();
    g.add_street(bp.x - 100.0, RIVER_Y - 80.0, bp.x + 100.0, RIVER_Y - 80.0, "street", 4.0, 0, "test");
    g.add_street(bp.x + 25.0, RIVER_Y - 100.0, bp.x + 25.0, RIVER_Y + 100.0, "primary", 6.0, 0, "test");
    assert_eq!(super::lay_town_bridge(&site, &anchors, &mut g, 400.0), super::TownBridge::Existing);
    let mut g = crate::graph::Graph::new();
    g.add_street(bp.x - 100.0, RIVER_Y - 80.0, bp.x + 100.0, RIVER_Y - 80.0, "street", 4.0, 0, "test");
    g.add_street(bp.x + 35.0, RIVER_Y - 100.0, bp.x + 35.0, RIVER_Y + 100.0, "primary", 6.0, 0, "test");
    assert_eq!(super::lay_town_bridge(&site, &anchors, &mut g, 400.0), super::TownBridge::Laid);
    // No street within `BRIDGE_JOIN_M` (240 m) of the near end: refused.
    let mut g = crate::graph::Graph::new();
    g.add_street(bp.x, RIVER_Y - 300.0, bp.x + 100.0, RIVER_Y - 300.0, "street", 4.0, 0, "test");
    assert_eq!(super::lay_town_bridge(&site, &anchors, &mut g, 400.0), super::TownBridge::None);
    let mut g = crate::graph::Graph::new();
    g.add_street(bp.x, RIVER_Y - 250.0, bp.x + 100.0, RIVER_Y - 250.0, "street", 4.0, 0, "test");
    assert_eq!(super::lay_town_bridge(&site, &anchors, &mut g, 400.0), super::TownBridge::Laid, "222 m away is within 240 m");
}

/// Protects Ruling EZ item 1's mixed frontage (literals from the fixture):
/// with a frontage seed, the riverside is cut into embankment `quay` stretches
/// whose every vertex is 15 m from the centreline (12 + 1 + 2) and `bank`
/// stretches whose interior vertices are 13.5 m from it (12 + 1 + 0.5), each
/// stretch 90-220 m long but for a run's last; both kinds are in the graph,
/// and a house fronts a `bank` edge within one street width (5.4 m) of the
/// channel's edge. Without a seed every stretch is a quay.
#[test]
fn the_riverside_mixes_quays_and_houses_on_the_water() {
    let streets: Vec<(Vec2, Vec2)> = (0..20).map(|k| {
        let x = 200.0 + k as f64 * 60.0;
        (Vec2::new(x, RIVER_Y - 120.0), Vec2::new(x, RIVER_Y - 30.0))
    }).collect();
    let (site, mut g) = bank_site_and_streets(&streets);
    let e = super::build_embankment(&site, &mut g, None, Some(11));
    assert!(!e.runs.is_empty() && !e.banks.is_empty(), "both forms: {} quay, {} bank", e.runs.len(), e.banks.len());
    for run in &e.runs {
        assert!(run.iter().all(|v| ((v.y - RIVER_Y).abs() - 15.0).abs() < 1e-9), "quay vertex off 15 m: {run:?}");
    }
    let len = |r: &Vec<Vec2>| r.windows(2).map(|w| w[0].dist(w[1])).sum::<f64>();
    let mut stretches: Vec<&Vec<Vec2>> = e.runs.iter().chain(e.banks.iter()).collect();
    stretches.sort_by(|a, b| a[0].x.total_cmp(&b[0].x));
    for s in &stretches[..stretches.len() - 1] {
        // A stretch ends once 90-220 m of centreline is behind it; the bank
        // stretch's junction step adds at most 1.5 m.
        assert!(len(s) >= 90.0 - 8.0 && len(s) <= 220.0 + 8.0 + 1.5, "stretch of {} m", len(s));
    }
    for b in &e.banks {
        assert!(b[1..b.len() - 1].iter().all(|v| ((v.y - RIVER_Y).abs() - 13.5).abs() < 1e-9), "bank vertex off 13.5 m: {b:?}");
    }
    assert!(g.edges.iter().any(|x| x.alive && x.cls == super::BANK_CLASS));
    assert!(g.edges.iter().any(|x| x.alive && x.cls == "quay"));
    let (site, mut g) = bank_site_and_streets(&streets);
    let e = super::build_embankment(&site, &mut g, None, None);
    assert!(e.banks.is_empty() && !e.runs.is_empty());
    // End to end (seed 4243, four bank stretches, measured 1.9 m): a big
    // town's houses on a bank edge stand within a street width of the
    // channel's edge.
    let t = generate(4243, &opts(true, true));
    assert!(t.embankment.as_ref().is_some_and(|e| !e.banks.is_empty()), "the fixture town has a bank stretch");
    assert!(built_to_bank(&t) <= 5.4, "nearest house {} m", built_to_bank(&t));
}

/// Protects `lot_suitability`: on a riverbank site a lot fronting a `bank` or
/// `quay` edge 3.4 m off the channel is judged on its slope alone (the flood
/// factor would score it 0.47 at its centroid, 19.9 m out: vacant), a lot
/// in the same place fronting a `street` keeps the reference's score, and on a
/// site the riverbank geometry did not take, a `bank` lot keeps it too.
#[test]
fn a_riverside_lot_is_judged_on_its_slope() {
    let lot = |cls: &'static str| crate::blocks::Parcel {
        id: "t".into(),
        poly: vec![
            Vec2::new(800.0, RIVER_Y + 15.4),
            Vec2::new(810.0, RIVER_Y + 15.4),
            Vec2::new(810.0, RIVER_Y + 24.4),
            Vec2::new(800.0, RIVER_Y + 24.4),
        ],
        block: "b".into(),
        frontage: 10.0,
        depth: 9.0,
        area: 90.0,
        age: 0.0,
        edge_cls: cls,
        tone: 0.5,
        wall_backing: crate::wallside::WallBacking::No,
        courtyard_ring: false,
        gate_quality: None,
    };
    let mut site = build_site(7, 1700.0, 1250.0, "river", SiteOpts { water: Some(water(true)), ..Default::default() });
    let c = Vec2::new(805.0, RIVER_Y + 19.9);
    let reference = crate::site::terrain_suitability(&site, c);
    assert_eq!(crate::districts::lot_suitability(&site, &lot(super::BANK_CLASS)), reference, "not a riverbank site yet");
    assert!(site.use_riverbank());
    let flood = crate::site::terrain_suitability(&site, c);
    let slope = crate::site::slope_suitability(&site, c);
    assert!(flood < crate::districts::TERRAIN_BUILD_MIN && slope >= crate::districts::TERRAIN_BUILD_MIN, "premise: {flood} {slope}");
    assert_eq!(crate::districts::lot_suitability(&site, &lot(super::BANK_CLASS)), slope);
    assert_eq!(crate::districts::lot_suitability(&site, &lot("quay")), slope);
    assert_eq!(crate::districts::lot_suitability(&site, &lot("street")), flood);
}


/// Protects the run-end ties (literals): a U-shaped street whose corners
/// stand 40 m from the river and whose ends are 200 m back has no dead end
/// near the bank, so only the ends of the riverside runs laid by its corners
/// can be tied to it; they are (at least one tie), which closes the faces
/// between the street and the river.
#[test]
fn a_riverside_run_is_tied_at_its_ends() {
    let r = RIVER_Y;
    let u = [
        (Vec2::new(400.0, r - 200.0), Vec2::new(400.0, r - 40.0)),
        (Vec2::new(400.0, r - 40.0), Vec2::new(800.0, r - 40.0)),
        (Vec2::new(800.0, r - 40.0), Vec2::new(800.0, r - 200.0)),
    ];
    let (site, mut g) = bank_site_and_streets(&u);
    let e = super::build_embankment(&site, &mut g, None, None);
    assert!(!e.runs.is_empty(), "premise: a run is laid by the corners");
    assert!(e.ties >= 1, "{e:?}");
}

/// Protects `trim_overbuilt` (literals): ten built lots in a row 10-100 m
/// from the market, the two farthest fronting a `quay` and a `bank` edge; a
/// town of 20.8 people may house `floor(20.8 * 1.25 / 5.2)` = 5 lots, so the
/// five farthest inland lots (40-80 m) are emptied with their buildings and
/// the riverside lots stay built; a town within its cap is left alone; and
/// with a wall, the lots outside it go before any inside it.
#[test]
fn an_overbuilt_river_town_empties_its_farthest_inland_lots() {
    let parcel = |k: usize| crate::blocks::Parcel {
        id: format!("p{k}"),
        poly: vec![
            Vec2::new(10.0 * k as f64 + 8.0, 0.0),
            Vec2::new(10.0 * k as f64 + 12.0, 0.0),
            Vec2::new(10.0 * k as f64 + 12.0, 4.0),
            Vec2::new(10.0 * k as f64 + 8.0, 4.0),
        ],
        block: "b".into(),
        frontage: 4.0,
        depth: 4.0,
        area: 16.0,
        age: 0.0,
        edge_cls: match k {
            9 => "quay",
            10 => super::BANK_CLASS,
            _ => "street",
        },
        tone: 0.5,
        wall_backing: crate::wallside::WallBacking::No,
        courtyard_ring: false,
        gate_quality: None,
    };
    let parcels: Vec<crate::blocks::Parcel> = (1..=10).map(parcel).collect();
    let building = |k: usize| crate::districts::Building {
        id: format!("b{k}"),
        poly: parcels[k - 1].poly.clone(),
        ridge: [Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0)],
        parcel: format!("p{k}"),
        kind: "house",
        district: "",
        age: 0.0,
        courtyard: false,
        prov: "test",
    };
    let fresh = || {
        let mut lots: Vec<crate::districts::Lot<'_>> = parcels.iter().map(crate::districts::Lot::new).collect();
        for l in &mut lots {
            l.built = true;
        }
        lots
    };
    let mut lots = fresh();
    let mut blds: Vec<crate::districts::Building> = (1..=10).map(building).collect();
    super::trim_overbuilt(&mut lots, &mut blds, Vec2::new(0.0, 2.0), 20.8, None);
    let built: Vec<&str> = lots.iter().filter(|l| l.built).map(|l| l.par.id.as_str()).collect();
    assert_eq!(built, ["p1", "p2", "p3", "p9", "p10"]);
    assert_eq!(blds.iter().map(|b| b.parcel.as_str()).collect::<Vec<_>>(), ["p1", "p2", "p3", "p9", "p10"]);
    assert!(lots.iter().filter(|l| !l.built).all(|l| l.empty));
    let mut lots = fresh();
    let mut blds: Vec<crate::districts::Building> = (1..=10).map(building).collect();
    super::trim_overbuilt(&mut lots, &mut blds, Vec2::new(0.0, 2.0), 41.6, None);
    assert_eq!(lots.iter().filter(|l| l.built).count(), 10, "41.6 people may house 10 lots");
    assert_eq!(blds.len(), 10);
    // A wall round the lots 50-90 m out (p5-p8, x 58-92): the inland lots
    // outside it go first (p4, p3, p2, p1), then the farthest inside (p8).
    let ring = [Vec2::new(55.0, -10.0), Vec2::new(95.0, -10.0), Vec2::new(95.0, 10.0), Vec2::new(55.0, 10.0)];
    let mut lots = fresh();
    let mut blds: Vec<crate::districts::Building> = (1..=10).map(building).collect();
    super::trim_overbuilt(&mut lots, &mut blds, Vec2::new(0.0, 2.0), 20.8, Some(&ring));
    let built: Vec<&str> = lots.iter().filter(|l| l.built).map(|l| l.par.id.as_str()).collect();
    assert_eq!(built, ["p5", "p6", "p7", "p9", "p10"]);
}

/// Protects `generate`'s call of the housing cap and its village gate: the
/// 400-person fixture village houses at most 500 (125 %) with the switch on, where the same
/// village with the riverbank geometry but no cap would house more (its
/// streets plat more lots than 400 people need: the reference's 1 600 m
/// street floor). The premise is read off the switch-off town.
#[test]
fn a_river_village_is_not_overbuilt() {
    let off = generate(4242, &GenOpts { pop: Some(400.0), ..opts(false, true) });
    let on = generate(4242, &GenOpts { pop: Some(400.0), ..opts(true, true) });
    assert!(off.pop > 500.0, "premise: the reference's village houses {}", off.pop);
    assert!(on.pop <= 500.0, "the riverbank village houses {}", on.pop);
    // A town of 1 500 (not a village) is never trimmed: seed 4201's houses
    // 2 616 (found by scanning seeds 4200-4259), over 125 %.
    let town = generate(4201, &GenOpts { pop: Some(1500.0), ..opts(true, true) });
    assert!(town.pop > 1875.0, "a chartered town keeps its houses: {}", town.pop);
}

/// Protects `bank_street_lengths`, which `grow` reads against
/// `FAR_BANK_MAX_RATIO`: a 100 m street north of the river and a 30 m one
/// south of it split by bank (literals). The cap's call site in `grow` is not
/// pinned by a test (its mutant survives: the far bank's final street length
/// also counts the lane passes and primaries after `grow`); the both-banks
/// audit measures its effect.
#[test]
fn bank_street_lengths_split_by_bank() {
    let (site, g) = bank_site_and_streets(&[
        (Vec2::new(100.0, RIVER_Y - 100.0), Vec2::new(200.0, RIVER_Y - 100.0)),
        (Vec2::new(100.0, RIVER_Y + 100.0), Vec2::new(130.0, RIVER_Y + 100.0)),
    ]);
    let north_side = site.bank_side(Vec2::new(150.0, RIVER_Y - 100.0));
    let (n, f) = super::bank_street_lengths(&site, &g, north_side);
    assert!((n - 100.0).abs() < 1e-9 && (f - 30.0).abs() < 1e-9, "{n} {f}");
}


/// The street length on each bank of the fixture's graph: `(north, south)`,
/// by each live edge's midpoint (the market is north in
/// `the_far_bank_pass_is_capped_and_grows_only_the_far_bank`).
fn bank_lengths(g: &crate::graph::Graph) -> (f64, f64) {
    let (mut n, mut s) = (0.0, 0.0);
    for e in g.edges.iter().filter(|e| e.alive) {
        let (a, b) = (g.nodes[e.a].pt(), g.nodes[e.b].pt());
        if north(a.lerp(b, 0.5)) { n += a.dist(b) } else { s += a.dist(b) }
    }
    (n, s)
}

/// Protects the far-bank pass's cap at its call site (2026-11-15; the
/// coordinator's item 5, the survivor of `88fb0ef3`'s mutants): asked for
/// ten times the market bank's street length (`share` 10.0, past anything
/// `second_pass` asks), `crate::growth::grow_far_bank` grows the south bank
/// and stops once its streets reach the north bank's (`FAR_BANK_MAX_RATIO`
/// 1.0) plus at most one street (the 125 m segment cap of `grow`, a
/// literal); and it lays nothing on the market's bank. Removing the cap, or
/// the far-bank test on an origin, turns it red.
#[test]
fn the_far_bank_pass_is_capped_and_grows_only_the_far_bank() {
    let r = RIVER_Y;
    let mut streets: Vec<(Vec2, Vec2)> = (0..4).map(|k| (Vec2::new(500.0, r - 60.0 - k as f64 * 70.0), Vec2::new(1100.0, r - 60.0 - k as f64 * 70.0))).collect();
    streets.push((Vec2::new(800.0, r - 40.0), Vec2::new(800.0, r - 300.0)));
    streets.push((Vec2::new(800.0, r + 30.0), Vec2::new(800.0, r + 120.0)));
    let (site, mut g) = bank_site_and_streets(&streets);
    let anchors = crate::routes::Anchors { market: Vec2::new(800.0, r - 120.0), prov: "test" };
    let (n0, s0) = bank_lengths(&g);
    assert!((n0 - 2660.0).abs() < 1.0 && (s0 - 90.0).abs() < 1.0, "premise: {n0} {s0}");
    let opts = crate::growth::GrowOpts { max_rf: 720.0, ..Default::default() };
    let ws = crate::growth::WallState::default();
    let laid = crate::growth::grow_far_bank(7, &site, &anchors, &mut g, &ws, &opts, 8, 10.0, Vec2::new(800.0, r + 30.0));
    let (n1, s1) = bank_lengths(&g);
    assert!(laid > 500.0, "the far bank grew only {laid:.0} m");
    assert!(s1 <= n0 + 125.0, "far bank {s1:.0} m against the market bank's {n0:.0} m");
    assert!((n1 - n0).abs() < 1e-6, "the market's bank changed: {n0} -> {n1}");
}

// ------------------------------------- 2026-11-15, the both-banks rework --

/// Protects `second_pass`'s three answers (literals; the 6 000-person
/// fixture town's first layout, its head count set by hand): a both-banks
/// town housing 2 000 of 6 000 (a shortfall of 2.0 times what it housed) is
/// given `FarBank` at the cap (`share` 1.0), 4 000 of 6 000 a share of 0.5,
/// and one at 7 000 nothing; a
/// village of 400 housing 600 is refit at scale 400 / 600, one housing 499
/// (under 500, the 125 % cap) is not; with the switch off, or for a layout
/// that is already a second pass, nothing.
#[test]
fn second_pass_asks_for_the_shortfall_and_the_village_refit() {
    use super::{RiverbankPass, second_pass};
    let first = crate::generate::generate_pass(4242, &opts(true, true), RiverbankPass::First);
    assert!(first.both_banks && first.riverbank && first.riverbank_pass == Some(RiverbankPass::First));
    let with = |pop: f64, target: f64| {
        let mut t = first.clone();
        t.pop = pop;
        t.pop_target = target;
        t
    };
    assert_eq!(second_pass(&with(2000.0, 6000.0)), Some(RiverbankPass::FarBank { share: 1.0 }));
    assert_eq!(second_pass(&with(4000.0, 6000.0)), Some(RiverbankPass::FarBank { share: 0.5 }));
    assert_eq!(second_pass(&with(7000.0, 6000.0)), None);
    let mut village = with(600.0, 400.0);
    village.both_banks = false;
    assert_eq!(second_pass(&village), Some(RiverbankPass::Village { street_scale: 400.0 / 600.0, trim: false }));
    village.pop = 499.0;
    assert_eq!(second_pass(&village), None);
    let mut off = with(2000.0, 6000.0);
    off.riverbank = false;
    assert_eq!(second_pass(&off), None);
    let mut again = with(2000.0, 6000.0);
    again.riverbank_pass = Some(RiverbankPass::FarBank { share: 1.0 });
    assert_eq!(second_pass(&again), None);
}

/// Protects the coordinator's defect (c) (2026-11-15): the far bank is grown
/// after the market's bank and from a budget of its own, so the second
/// layout of the 6 000-person fixture town houses at least 95 % (a literal)
/// as many on the market's bank as its first layout did, and more in all.
#[test]
fn growing_the_far_bank_does_not_shrink_the_market_bank() {
    use super::RiverbankPass;
    let first = crate::generate::generate_pass(4242, &opts(true, true), RiverbankPass::First);
    let both = generate(4242, &opts(true, true));
    assert!(matches!(both.riverbank_pass, Some(RiverbankPass::FarBank { .. })), "{:?}", both.riverbank_pass);
    let market_north = north(first.anchors.market);
    let on_market_bank = |t: &crate::generate::Town| {
        t.buildings.iter().filter(|b| north(b.poly.iter().fold(Vec2::new(0.0, 0.0), |a, v| a + *v) * (1.0 / b.poly.len() as f64)) == market_north).count()
    };
    let (n1, n2) = (on_market_bank(&first), on_market_bank(&both));
    assert!(n2 as f64 >= 0.95 * n1 as f64, "market bank {n1} -> {n2} houses");
    assert!(both.buildings.len() > first.buildings.len(), "{} -> {}", first.buildings.len(), both.buildings.len());
}

/// Protects `fit_village`'s search and its trim fallback, on stand-in
/// layouts (the fixture town with its head count set by `lay`; target 400,
/// so the band is 320..=500): a head count of `1 000 * scale` lands in the
/// band on the first try (scale 0.4, untrimmed); a head count that no scale
/// moves (900) is never in the band, so the over-cap layout is laid once
/// more with the trim, and only that once; and a village that every rescale
/// under-builds (200) is never taken under the band: the first layout
/// (scale 1) is trimmed instead.
#[test]
fn a_village_is_refit_by_its_street_scale_and_trimmed_only_as_a_last_resort() {
    use std::cell::RefCell;
    let base = crate::generate::generate_pass(4242, &GenOpts { pop: Some(400.0), ..opts(true, true) }, super::RiverbankPass::First);
    let mk = |pop: f64| {
        let mut t = base.clone();
        t.pop = pop;
        t.pop_target = 400.0;
        t
    };
    let calls = RefCell::new(Vec::new());
    let t = super::fit_village(mk(1000.0), |s, trim| {
        calls.borrow_mut().push((s, trim));
        mk(1000.0 * s)
    });
    assert_eq!(calls.borrow().as_slice(), &[(0.4, false)]);
    assert_eq!(t.pop, 400.0);
    calls.borrow_mut().clear();
    let t = super::fit_village(mk(1000.0), |s, trim| {
        calls.borrow_mut().push((s, trim));
        mk(if trim { 500.0 } else { 900.0 })
    });
    let c = calls.borrow();
    assert_eq!(c.iter().filter(|(_, trim)| *trim).count(), 1, "{c:?}");
    assert!(c.last().is_some_and(|(_, trim)| *trim), "{c:?}");
    assert!(c.len() <= 1 + super::VILLAGE_FITS && c.len() >= 2, "{c:?}");
    assert_eq!(t.pop, 500.0);
    drop(c);
    calls.borrow_mut().clear();
    let t = super::fit_village(mk(1000.0), |s, trim| {
        calls.borrow_mut().push((s, trim));
        mk(if trim { 500.0 } else if s >= 1.0 { 1000.0 } else { 200.0 })
    });
    assert_eq!(calls.borrow().last(), Some(&(1.0, true)), "{:?}", calls.borrow());
    assert_eq!(t.pop, 500.0);
}

/// Protects defect (a) end to end on the fixture (2026-11-15): with the wall
/// hug on, the 6 000-person river town's market square stands inside its
/// circuit, and the refit circuit does not cross itself.
#[test]
fn the_market_square_stays_inside_the_wall() {
    let t = generate(4242, &GenOpts { wall_hug: true, ..opts(true, true) });
    let ring = t.wall.ring.as_deref().expect("the fixture town is walled");
    let plaza = t.plaza.as_ref().expect("the fixture town has a square");
    assert!(crate::geom::point_in_poly(plaza.center, ring), "square at {:?} outside the wall ({:?})", plaza.center, t.wall_hug);
    assert!(!crate::wallhug::ring_self_crosses(ring));
}

/// Protects `plaza::riverbank_square` (2026-11-15, defect (a); literals from
/// the fixture's straight river at `RIVER_Y`, 24 m wide): a market on the
/// north bank at the foot of a 600 m primary running north gets its 60 by 30
/// square on that primary, wholly north of the channel by at least 15 m
/// (`SQUARE_RIVER_CLEAR_M`) past its 12 m half-width, and its centre within
/// 80 m of the market (the reference's mid-edge square would stand 300 m up
/// the road); a primary that crosses the river to the south bank is never
/// chosen for it.
#[test]
fn a_riverbank_square_stands_by_its_market_on_its_bank() {
    let r = RIVER_Y;
    let (site, mut g) = bank_site_and_streets(&[]);
    g.add_street(850.0, r - 20.0, 850.0, r - 620.0, "primary", 6.0, 0, "test");
    g.add_street(900.0, r + 300.0, 900.0, r - 20.0, "primary", 6.0, 0, "test");
    let anchors = crate::routes::Anchors { market: Vec2::new(850.0, r - 20.0), prov: "test" };
    let (dir, mid) = crate::plaza::riverbank_square(&site, &anchors, &g, 60.0, 30.0).expect("room for a square");
    assert!(dir.x.abs() < 1e-9, "on the north road: {dir:?}");
    assert!((mid.x - 850.0).abs() < 1e-6, "{mid:?}");
    assert!(mid.y <= r - 12.0 - 15.0 - 30.0, "too near the water: {mid:?}");
    assert!(mid.dist(anchors.market) < 80.0, "{mid:?}");
}

/// Protects `fit_village`'s secant step, its aim and its layout count
/// (`VILLAGE_AIM_SHARE` 1.1, `VILLAGE_FITS` 4; literals worked by hand): a
/// village of target 400 whose head count is `1 000 * scale^2` misses on the
/// first rescale (scale 0.4: 160, under the band) and the secant through
/// (1, 1 000) and (0.4, 160) aimed at 440 lands on scale 0.6 (360, inside
/// the 320..=500 band), which is taken untrimmed after two layouts.
#[test]
fn a_village_refit_follows_the_secant_to_its_aim() {
    use std::cell::RefCell;
    let base = crate::generate::generate_pass(4242, &GenOpts { pop: Some(400.0), ..opts(true, true) }, super::RiverbankPass::First);
    let mk = |pop: f64| {
        let mut t = base.clone();
        t.pop = pop;
        t.pop_target = 400.0;
        t
    };
    let calls = RefCell::new(Vec::new());
    let t = super::fit_village(mk(1000.0), |s, trim| {
        calls.borrow_mut().push((s, trim));
        mk(1000.0 * s * s)
    });
    let c = calls.borrow();
    assert_eq!(c.len(), 2, "{c:?}");
    assert!((c[0].0 - 0.4).abs() < 1e-9 && (c[1].0 - 0.6).abs() < 1e-9 && !c[0].1 && !c[1].1, "{c:?}");
    assert!((t.pop - 360.0).abs() < 1e-6, "{}", t.pop);
}

/// Protects `fit_village`'s choice among over-cap layouts (literals): when no
/// layout lands in the band, the one trimmed is the over-cap layout that
/// housed the fewest (the first rescale's 520 here, against 900 after it),
/// laid out again at its own scale (0.4).
#[test]
fn an_unfittable_village_trims_the_layout_nearest_the_cap() {
    use std::cell::RefCell;
    let base = crate::generate::generate_pass(4242, &GenOpts { pop: Some(400.0), ..opts(true, true) }, super::RiverbankPass::First);
    let mk = |pop: f64| {
        let mut t = base.clone();
        t.pop = pop;
        t.pop_target = 400.0;
        t
    };
    let calls = RefCell::new(Vec::new());
    super::fit_village(mk(1000.0), |s, trim| {
        let n = calls.borrow().len();
        calls.borrow_mut().push((s, trim));
        mk(if trim { 500.0 } else if n == 0 { 520.0 } else { 900.0 })
    });
    let c = calls.borrow();
    let last = c.last().copied().expect("laid out");
    assert!(last.1 && (last.0 - 0.4).abs() < 1e-9, "{c:?}");
}

/// Protects `far_bridge_end` (literals from the fixture: the river along
/// `RIVER_Y`, 24 m wide, so the span's half-length is 12 + 16 = 28 m): with
/// the market on the north bank the far end stands 28 m south of the bridge
/// point, and with the market on the south bank 28 m north.
#[test]
fn the_far_bridge_end_is_across_the_river_from_the_market() {
    let (site, _) = bank_site_and_streets(&[]);
    let bp = site.bridge_pt.expect("the fixture has a bridge point");
    let south = super::far_bridge_end(&site, Vec2::new(850.0, RIVER_Y - 100.0)).expect("a far end");
    assert!((south.y - (bp.y + 28.0)).abs() < 1.0 && (south.x - bp.x).abs() < 1.0, "{south:?} from {bp:?}");
    let north = super::far_bridge_end(&site, Vec2::new(850.0, RIVER_Y + 100.0)).expect("a far end");
    assert!((north.y - (bp.y - 28.0)).abs() < 1.0, "{north:?} from {bp:?}");
}

/// Protects the riverbank square at its call site in `build_plaza`
/// (2026-11-15; literals from the fixture, river 24 m wide along `RIVER_Y`):
/// the 6 000-person river town's market square stands wholly on the
/// market's bank with every corner at least 12 + 15 m from the centreline
/// (`SQUARE_RIVER_CLEAR_M`); a village of 1 499 keeps the reference's green.
/// And `riverbank_square` never uses a primary on the other bank: with only
/// a primary that crosses the river from the south, it finds no room.
#[test]
fn the_market_square_stands_back_from_the_water_on_its_bank() {
    let t = generate(4242, &opts(true, true));
    let z = t.plaza.as_ref().expect("a square");
    let north_market = north(t.anchors.market);
    for c in &z.poly {
        assert_eq!(north(*c), north_market, "corner {c:?} on the other bank");
        assert!((c.y - RIVER_Y).abs() >= 27.0 - 1e-6, "corner {c:?} within 27 m of the centreline");
    }
    let r = RIVER_Y;
    let (site, mut g) = bank_site_and_streets(&[]);
    g.add_street(900.0, r + 300.0, 900.0, r - 20.0, "primary", 6.0, 0, "test");
    let anchors = crate::routes::Anchors { market: Vec2::new(900.0, r - 20.0), prov: "test" };
    assert_eq!(crate::plaza::riverbank_square(&site, &anchors, &g, 60.0, 30.0), None);
}

// ------------------------------- Ruling FE item 2, the street village (2026-11-15) --

/// Protects the street village's green placement (`plaza::square_near` and
/// `GREEN_RIVER_CLEAR_M`; literals from the fixture's straight river at
/// `RIVER_Y`, 24 m wide). A road along the north bank 22 m from the
/// centreline (10 m from the channel's edge): a town's square, which keeps
/// 15 m (`SQUARE_RIVER_CLEAR_M`) past the half-width, finds no room on it;
/// a village green, keeping 4 m, does, beside the market. A market at the
/// end of a road across the river, its own bank roadless: no green on the
/// market's bank, one at the bridge's other foot with `same_bank` false.
/// And a village road cut into 40 m edges by its lanes, 18 m from the
/// centreline (6 m from the channel's edge): no square fits one edge, but a
/// village green spans them (`village`, `ROAD_ON_M`) beside the market.
#[test]
fn a_village_green_stands_at_the_landing() {
    let r = RIVER_Y;
    let (site, mut g) = bank_site_and_streets(&[]);
    g.add_street(400.0, r - 22.0, 1300.0, r - 22.0, "primary", 6.0, 0, "test");
    let anchors = crate::routes::Anchors { market: Vec2::new(850.0, r - 22.0), prov: "test" };
    assert!(crate::plaza::square_near_market(&site, &anchors, &g, 60.0, 30.0, crate::plaza::SQUARE_RIVER_CLEAR_M).is_none());
    let (_, mid) = crate::plaza::square_near_market(&site, &anchors, &g, 60.0, 30.0, crate::plaza::GREEN_RIVER_CLEAR_M).expect("room by the bank");
    assert!(mid.dist(anchors.market) < 1.0, "{mid:?}");
    // The road comes from the north and ends across the bridge at the market.
    let (site, mut g) = bank_site_and_streets(&[]);
    g.add_street(850.0, r - 600.0, 850.0, r + 20.0, "primary", 6.0, 0, "test");
    let anchors = crate::routes::Anchors { market: Vec2::new(850.0, r + 20.0), prov: "test" };
    assert!(crate::plaza::square_near(&site, &anchors, &g, 60.0, 30.0, crate::plaza::GREEN_RIVER_CLEAR_M, true, false).is_none());
    let (_, mid) = crate::plaza::square_near(&site, &anchors, &g, 60.0, 30.0, crate::plaza::GREEN_RIVER_CLEAR_M, false, false).expect("room across the bridge");
    assert!(mid.y < r && mid.dist(anchors.market) < 90.0, "{mid:?}");
    let (site, mut g) = bank_site_and_streets(&[]);
    for k in 0..20 {
        let x = 450.0 + 40.0 * k as f64;
        g.add_street(x, r - 18.0, x + 40.0, r - 18.0, "primary", 6.0, 0, "test");
        g.add_street(x, r - 18.0, x, r - 60.0, "lane", 3.0, 1, "test");
    }
    let anchors = crate::routes::Anchors { market: Vec2::new(850.0, r - 18.0), prov: "test" };
    assert!(crate::plaza::square_near(&site, &anchors, &g, 60.0, 30.0, crate::plaza::GREEN_RIVER_CLEAR_M, true, false).is_none());
    let (_, mid) = crate::plaza::square_near(&site, &anchors, &g, 60.0, 30.0, crate::plaza::GREEN_RIVER_CLEAR_M, true, true).expect("a green across the short edges");
    assert!(mid.dist(anchors.market) < 25.0, "{mid:?}");
}

/// Protects `cleanup::prune_keeping` (Ruling FE item 2): of two pieces of
/// road, the larger far from the market, `prune_largest` keeps the larger
/// and `prune_keeping` with the market keeps the market's.
#[test]
fn a_street_village_keeps_its_market_road() {
    let mk = || {
        let mut g = crate::graph::Graph::new();
        g.add_street(100.0, 100.0, 200.0, 100.0, "primary", 6.0, 0, "near");
        for k in 0..4 {
            let x = 1000.0 + 100.0 * k as f64;
            g.add_street(x, 900.0, x + 100.0, 900.0, "primary", 6.0, 0, "far");
        }
        g
    };
    let alive_near = |g: &crate::graph::Graph| g.edges.iter().any(|e| e.alive && g.nodes[e.a].x < 300.0);
    let mut g = mk();
    crate::cleanup::prune_largest(&mut g);
    assert!(!alive_near(&g));
    let mut g = mk();
    crate::cleanup::prune_keeping(&mut g, Some(Vec2::new(150.0, 110.0)));
    assert!(alive_near(&g) && g.edges.iter().filter(|e| e.alive).count() == 1);
}

/// Protects `second_pass`'s street-village answer and `fit_short_village`
/// (literals worked by hand). A street village of target 400 housing 200
/// (under the 80 % `VILLAGE_LOW_SHARE`) is laid out again at scale 2.0
/// (400 / 200); housing nothing, at `VILLAGE_MAX_SCALE` (8); housing 330, not
/// at all. With a head count of `150 + 50 * scale` the secant from (1, 200)
/// and (2, 250) aimed at 440 lands on 5.8 (440, in the band) and is taken;
/// with `150 + 20 * scale` it is clamped to 8 (310) and, no scale reaching the
/// band, the layout housing the most (310) is returned.
#[test]
fn a_short_street_village_is_given_more_lanes() {
    use std::cell::RefCell;
    let base = crate::generate::generate_pass(4242, &GenOpts { pop: Some(400.0), street_village: true, ..opts(false, true) }, super::RiverbankPass::First);
    assert!(base.street_village && base.riverbank_pass == Some(super::RiverbankPass::First));
    let mk = |pop: f64| {
        let mut t = base.clone();
        t.pop = pop;
        t.pop_target = 400.0;
        t
    };
    assert_eq!(super::second_pass(&mk(200.0)), Some(super::RiverbankPass::Village { street_scale: 2.0, trim: false }));
    assert_eq!(super::second_pass(&mk(0.0)), Some(super::RiverbankPass::Village { street_scale: 8.0, trim: false }));
    assert_eq!(super::second_pass(&mk(330.0)), None);
    let calls = RefCell::new(Vec::new());
    let t = super::fit_village(mk(200.0), |s, _| {
        calls.borrow_mut().push(s);
        mk(150.0 + 50.0 * s)
    });
    assert_eq!(calls.borrow().len(), 2, "{:?}", calls.borrow());
    assert!((calls.borrow()[1] - 5.8).abs() < 1e-9 && (t.pop - 440.0).abs() < 1e-9, "{:?} {}", calls.borrow(), t.pop);
    calls.borrow_mut().clear();
    let t = super::fit_village(mk(170.0), |s, _| {
        calls.borrow_mut().push(s);
        mk(150.0 + 20.0 * s)
    });
    assert_eq!(calls.borrow().last().copied(), Some(8.0), "{:?}", calls.borrow());
    assert!((t.pop - 310.0).abs() < 1e-9, "{}", t.pop);
}
