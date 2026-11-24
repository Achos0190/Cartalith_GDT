//! CS-1's tests (Ruling HS): the two point choosers of [`super`] on hand-built rasters. Every expected point and
//! distance is a literal worked out from the raster, never read back from a constant, so a changed
//! [`super::COAST_FRONT_M`], [`super::COAST_CAP_M`], [`super::RIVER_CAP_M`], [`super::H_FLOOD_M`] or a flipped
//! comparison turns one of them red.

use super::*;

const C: f64 = 22.0;

/// A `mw` by `mh` raster of 22 m cells with sea in its first two rows (y < 44 m), flat ground 1 m above sea
/// elsewhere, each land cell's sea distance its centre's height above the sea edge, and the switches on.
fn coast(mw: usize, mh: usize) -> SetbackCtx {
    let mut sc = SetbackCtx { coast: true, river: true, fine: true, mw, mh, cell_m: C, ..Default::default() };
    for j in 0..mh {
        for _ in 0..mw {
            let sea = j < 2;
            sc.bits.push(if sea { SEA } else { 0 });
            sc.ground_m.push(if sea { -1.0 } else { 1.0 });
            sc.sea_m.push(if sea { 0.0 } else { ((j as f64 + 0.5) * C - 44.0) as f32 });
        }
    }
    sc
}

fn dry(_: Vec2) -> bool {
    false
}

#[test]
fn a_coastal_market_with_no_reason_comes_to_seventy_metres_from_the_sea() {
    let sc = coast(20, 40);
    let (m, water, rec) = coast_market(&sc, Vec2::new(220.0, 440.0), &dry).expect("the rule acts");
    assert_eq!((m.x, m.y), (220.0, 114.0), "44 m sea edge + 70 m");
    assert_eq!((water.x, water.y), (220.0, 44.0));
    assert!(rec.is_none(), "no reason, no record");
    // A market already nearer than that stays where it is.
    let (m2, _, _) = coast_market(&sc, Vec2::new(220.0, 90.0), &dry).unwrap();
    assert_eq!((m2.x, m2.y), (220.0, 90.0));
}

#[test]
fn a_surge_band_sets_the_market_back_to_the_nearest_free_cell_preferring_higher_ground() {
    let mut sc = coast(20, 40);
    // Surge over rows 2..=6 (y 44..154): the 70 m point (y 114, row 5) is in it.
    for j in 2..7 {
        for i in 0..20 {
            sc.bits[j * 20 + i] |= SURGE;
        }
    }
    // Row 7's centres stand at y 165; columns 9 and 10 (x 209, 231) are equally near x 220. Column 10 is higher.
    sc.ground_m[7 * 20 + 10] = 5.0;
    let (m, _, rec) = coast_market(&sc, Vec2::new(220.0, 440.0), &dry).unwrap();
    assert_eq!((m.x, m.y), (231.0, 165.0));
    let rec = rec.expect("a reason is recorded");
    assert_eq!(rec.water, SetbackWater::Coast);
    assert_eq!(rec.reasons, SURGE);
    assert_eq!(rec.to, Some((Vec2::new(231.0, 165.0), 121.0)), "165 - 44 m from the sea");
    assert_eq!(rec.describe(), "set back 121 m (storm surge)");
    // A wet cell is never chosen: with row 7 water to the site, row 8 (y 187) is.
    let wet = |p: Vec2| p.y > 154.0 && p.y < 176.0;
    let (m, _, _) = coast_market(&sc, Vec2::new(220.0, 440.0), &wet).unwrap();
    assert_eq!(m.y, 187.0);
}

#[test]
fn the_coastal_cap_is_six_hundred_metres_inclusive() {
    let mut sc = coast(20, 40);
    // Every land cell has a reason, but one, whose sea distance is set by hand.
    for j in 2..40 {
        for i in 0..20 {
            sc.bits[j * 20 + i] |= MARSH;
        }
    }
    let k = 30 * 20 + 10;
    sc.bits[k] = 0;
    sc.sea_m[k] = 600.0;
    let (_, _, rec) = coast_market(&sc, Vec2::new(220.0, 440.0), &dry).unwrap();
    assert_eq!(rec.unwrap().to.map(|t| t.1), Some(600.0), "600 m is within the cap");
    sc.sea_m[k] = 600.5;
    let (m, _, rec) = coast_market(&sc, Vec2::new(220.0, 440.0), &dry).unwrap();
    let rec = rec.unwrap();
    assert_eq!(rec.to, None, "600.5 m is past it: no safe ground in reach");
    assert_eq!((m.x, m.y), (220.0, 114.0), "the market stays at the water");
    assert_eq!(rec.describe(), "no safe ground in reach (marsh or flood)");
}

#[test]
fn the_coastal_rule_does_not_act_switched_off_on_a_coarse_grid_or_with_no_sea() {
    let mc = Vec2::new(220.0, 440.0);
    assert!(coast_market(&SetbackCtx { coast: false, ..coast(20, 40) }, mc, &dry).is_none());
    assert!(coast_market(&SetbackCtx { fine: false, ..coast(20, 40) }, mc, &dry).is_none());
    let mut no_sea = coast(20, 40);
    no_sea.bits.iter_mut().for_each(|b| *b = 0);
    assert!(coast_market(&no_sea, mc, &dry).is_none());
}

/// A raster for the river walk: a river along y = 100 m, floodplain everywhere, the ground 0 m above the channel
/// up to row 13 and `terrace` m from row 14 (y from 308 m) on.
fn river(terrace: f32) -> (SetbackCtx, Vec<Vec2>) {
    let (mw, mh) = (20, 50);
    let mut sc = SetbackCtx { coast: true, river: true, fine: true, mw, mh, cell_m: C, ..Default::default() };
    for j in 0..mh {
        for _ in 0..mw {
            sc.bits.push(FLOODPLAIN);
            sc.ground_m.push(if j >= 14 { terrace } else { 0.0 });
            sc.sea_m.push(f32::INFINITY);
        }
    }
    (sc, vec![Vec2::new(0.0, 100.0), Vec2::new(440.0, 100.0)])
}

#[test]
fn a_floodplain_market_walks_away_from_its_river_to_the_first_ground_two_metres_up() {
    let (sc, rv) = river(2.0);
    let (m, rec) = river_market(&sc, &rv, Vec2::new(220.0, 150.0), &dry).expect("on floodplain");
    // 150 + 11 m steps: the first in row 14 (y >= 308) is 315.
    assert_eq!((m.x, m.y), (220.0, 315.0));
    assert_eq!(rec.to, Some((Vec2::new(220.0, 315.0), 215.0)));
    assert_eq!(rec.reasons, FLOODPLAIN);
    // Under two metres is still flood ground: no terrace in reach.
    let (sc, rv) = river(1.9);
    let (m, rec) = river_market(&sc, &rv, Vec2::new(220.0, 150.0), &dry).unwrap();
    assert_eq!(((m.x, m.y), rec.to), ((220.0, 150.0), None));
    assert_eq!(rec.describe(), "no terrace in reach (floodplain)");
}

#[test]
fn the_walk_skips_wetland_stops_at_water_and_at_five_hundred_metres() {
    let (mut sc, rv) = river(2.0);
    for i in 0..20 {
        sc.bits[14 * 20 + i] |= WETLAND;
    }
    let (m, _) = river_market(&sc, &rv, Vec2::new(220.0, 150.0), &dry).unwrap();
    assert_eq!(m.y, 337.0, "row 14 is wetland; row 15 starts at 330");
    let wet = |p: Vec2| p.y > 200.0 && p.y < 210.0;
    assert_eq!(river_market(&sc, &rv, Vec2::new(220.0, 150.0), &wet).unwrap().1.to, None, "water first");
    // The cap: from y 150, 500 m reaches y 650 (row 29). A terrace from row 30 (660) is out of reach, from row 29
    // (y 638..660, first step 645) in it.
    let mut far = river(0.0);
    for j in 29..50 {
        for i in 0..20 {
            far.0.ground_m[j * 20 + i] = 3.0;
        }
    }
    assert_eq!(river_market(&far.0, &far.1, Vec2::new(220.0, 150.0), &dry).unwrap().0.y, 645.0);
    for i in 0..20 {
        far.0.ground_m[29 * 20 + i] = 0.0;
    }
    assert_eq!(river_market(&far.0, &far.1, Vec2::new(220.0, 150.0), &dry).unwrap().1.to, None, "row 30 is past 500 m");
}

#[test]
fn the_river_rule_needs_floodplain_low_ground_and_the_switch_and_never_walks_on_a_coarse_grid() {
    let mc = Vec2::new(220.0, 150.0);
    let (sc, rv) = river(2.0);
    assert!(river_market(&SetbackCtx { river: false, ..sc.clone() }, &rv, mc, &dry).is_none());
    let mut unflagged = sc.clone();
    unflagged.bits.iter_mut().for_each(|b| *b = 0);
    assert!(river_market(&unflagged, &rv, mc, &dry).is_none(), "not floodplain");
    let mut high = sc.clone();
    high.ground_m.iter_mut().for_each(|h| *h = 2.0);
    // The channel at y 100 is row 4, also 2 m: the market is 0 m above it. Raise the market's own cell only.
    high.ground_m.iter_mut().for_each(|h| *h = 0.0);
    high.ground_m[6 * 20 + 10] = 2.0;
    assert!(river_market(&high, &rv, mc, &dry).is_none(), "2 m above the channel is above flood level");
    let (m, rec) = river_market(&SetbackCtx { fine: false, ..sc }, &rv, mc, &dry).unwrap();
    assert_eq!(((m.x, m.y), rec.to), ((220.0, 150.0), None), "a coarse grid reports no terrace in reach");
}

/// The Ruling HZ fixture site: a 150 m channel (y 550 to 700, centreline y = 625) through a 1 700 x 1 250 m box on a
/// real mask, as `cleanup`'s own real-map fixture lays it.
fn river_site() -> crate::site::Site {
    use crate::site::{SiteOpts, WaterCtx, build_site};
    let (mw, mh, cell_m) = (34usize, 25usize, 50.0f64);
    let mut mask = vec![0u8; mw * mh];
    let mut dt = vec![0.0f64; mw * mh];
    for j in 0..mh {
        for i in 0..mw {
            let yj = j as f64 * cell_m;
            mask[j * mw + i] = u8::from((550.0..700.0).contains(&yj));
            dt[j * mw + i] = (yj - 625.0).abs() / cell_m;
        }
    }
    let w = WaterCtx { mask, dt, mw, mh, cell_m, river_path: Some(vec![Vec2::new(0.0, 625.0), Vec2::new(1700.0, 625.0)]), river_width_m: Some(150.0), river_order: 4.0, sea_lake_cells: 0.0, open_water: None };
    let site = build_site(88, 1700.0, 1250.0, "river", SiteOpts { water: Some(w), ..Default::default() });
    assert!(site.real_river && site.uses_real_water);
    site
}

/// Protects Ruling HZ's wall rule: a street that crosses the river and meets the wall with no gate within
/// [`GATE_NEAR_M`] gets one; one with a gate 5 m away gets none, one 15 m away gets one (the bar is the 12 m).
#[test]
fn a_bridge_road_at_the_wall_gets_a_gate_unless_one_is_near() {
    let site = river_site();
    let mut g = crate::graph::Graph::new();
    g.add_street(300.0, 500.0, 300.0, 730.0, "primary", 6.0, 0, "t");
    let ring = vec![Vec2::new(100.0, 100.0), Vec2::new(1500.0, 100.0), Vec2::new(1500.0, 545.0), Vec2::new(100.0, 545.0)];
    let mut wall = crate::growth::WallState { ring: Some(ring), land_arc: Some(vec![Vec2::new(100.0, 545.0), Vec2::new(1500.0, 545.0)]), ..Default::default() };
    assert_eq!(gate_bridges(&site, &g, &mut wall), 1);
    assert!((wall.gates[0].pt.y - 545.0).abs() < 1e-6 && (wall.gates[0].pt.x - 300.0).abs() < 1e-6);
    wall.gates = vec![crate::growth::Gate { pt: Vec2::new(305.0, 545.0), water: false, prov: String::new() }];
    assert_eq!(gate_bridges(&site, &g, &mut wall), 0);
    wall.gates = vec![crate::growth::Gate { pt: Vec2::new(315.0, 545.0), water: false, prov: String::new() }];
    assert_eq!(gate_bridges(&site, &g, &mut wall), 1);
}

/// Protects Ruling HZ's street rule: with none of the market's streets across the river, one is laid from the street
/// end nearest it to the first dry ground past the centreline; with one already across, none; with the street end
/// off the market's own streets (a different component), the market's end is the one used.
#[test]
fn a_town_without_a_crossing_lays_one_from_its_street_end() {
    let site = river_site();
    let mut g = crate::graph::Graph::new();
    g.add_street(300.0, 400.0, 300.0, 520.0, "street", 4.0, 0, "t");
    let before = g.edges.len();
    assert!(reseek_crossing(&site, &mut g, Vec2::new(300.0, 400.0), 2.0));
    assert_eq!(g.edges.len(), before + 1);
    let e = g.edges.last().unwrap();
    let (a, b) = (g.nodes[e.a].pt(), g.nodes[e.b].pt());
    assert!((a.y - 520.0).abs() < 1e-6, "from the street end {a:?}");
    assert!((b.x - 300.0).abs() < 1e-6 && (700.0..=704.0).contains(&b.y), "to the far bank {b:?}");
    assert!(!reseek_crossing(&site, &mut g, Vec2::new(300.0, 400.0), 2.0), "already across");
    // A street end on the far bank's component only (not the market's): not used.
    let mut h = crate::graph::Graph::new();
    h.add_street(300.0, 400.0, 300.0, 440.0, "street", 4.0, 0, "t");
    h.add_street(900.0, 720.0, 900.0, 760.0, "street", 4.0, 0, "t");
    assert!(reseek_crossing(&site, &mut h, Vec2::new(300.0, 400.0), 2.0));
    let e = h.edges.last().unwrap();
    assert!((h.nodes[e.a].pt().y - 440.0).abs() < 1e-6);
}

/// Protects Ruling HZ's wall rule for a road that stands outside the ring within the clear distance (a graze, no true
/// crossing): the gate goes at its midpoint when the land arc is 10 m away, not when it is 40 m away.
#[test]
fn a_bridge_road_grazing_the_wall_gets_a_gate_at_its_midpoint() {
    let site = river_site();
    let mut g = crate::graph::Graph::new();
    g.add_street(300.0, 530.0, 300.0, 730.0, "primary", 6.0, 0, "t");
    let ring = vec![Vec2::new(100.0, 100.0), Vec2::new(1500.0, 100.0), Vec2::new(1500.0, 500.0), Vec2::new(100.0, 500.0)];
    let mut wall = crate::growth::WallState { ring: Some(ring.clone()), land_arc: Some(vec![Vec2::new(310.0, 600.0), Vec2::new(310.0, 660.0)]), ..Default::default() };
    assert_eq!(gate_bridges(&site, &g, &mut wall), 1);
    assert!((wall.gates[0].pt.y - 630.0).abs() < 1e-6);
    wall.gates.clear();
    wall.land_arc = Some(vec![Vec2::new(340.0, 600.0), Vec2::new(340.0, 660.0)]);
    assert_eq!(gate_bridges(&site, &g, &mut wall), 0);
}

