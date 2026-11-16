//! Ruling FK item 1's gate (owner, 2026-11-16: *"harbour towns lay out to
//! their water"*): [`harbour_front_market`] and what `generate` does with it
//! (`GenOpts::harbour_front`). Each test says what it protects. The fixture is
//! a coastal box whose sea is a strip of whole raster columns at its west edge,
//! stamped the way `cartalith_civ`'s `finish_water_ctx` stamps one (sea cells in
//! both the mask and the open-water raster, no river), so every distance below
//! is checkable by hand: the box centre (850, 625) stands `850 - 22 * cols` m
//! from the sea.

use super::{GenOpts, HARBOUR_FRONT_M, HARBOUR_FRONT_RIVER_M, HARBOUR_REACH_M, Town, generate, harbour_front_market, hash_model, houses_reach_water, town_reaches_water};
use crate::geom::Vec2;
use crate::routes::{Anchors, build_primaries_from_paths, build_primaries_from_paths_at};
use crate::site::{SiteOpts, WaterCtx, build_site};

const CELL: f64 = 22.0;
const MW: usize = 77;
const MH: usize = 57;

/// A coast box whose westmost `cols` raster columns are sea. `open` decides
/// whether the host supplied the open-water raster.
fn coast(cols: usize, open: bool) -> WaterCtx {
    let mask: Vec<u8> = (0..MW * MH).map(|k| u8::from(k % MW < cols)).collect();
    // Whole columns: the distance transform is the column distance, in cells.
    let dt: Vec<f64> = (0..MW * MH).map(|k| ((k % MW) as f64 - (cols as f64 - 1.0)).max(0.0)).collect();
    WaterCtx {
        mask: mask.clone(),
        dt,
        mw: MW,
        mh: MH,
        cell_m: CELL,
        river_path: None,
        river_width_m: Some(12.0),
        river_order: 0.0,
        sea_lake_cells: (cols * MH) as f64,
        open_water: open.then_some(mask),
    }
}

fn opts(cols: usize, front: bool) -> GenOpts {
    GenOpts { site: Some("coast".into()), pop: Some(2500.0), water: Some(coast(cols, true)), harbour_front: front, ..GenOpts::default() }
}

/// The least distance from any building vertex to the sea strip's edge
/// (`22 * cols` m east of the box's west side), metres.
fn built_to_sea(t: &Town, cols: usize) -> f64 {
    t.buildings.iter().flat_map(|b| b.poly.iter()).map(|v| (v.x - CELL * cols as f64).max(0.0)).fold(f64::INFINITY, f64::min)
}

/// Protects the switch's off state: with `harbour_front` off the market is the
/// box centre and the pin is the market (the reference), and with it on a town
/// whose sea is already within [`HARBOUR_FRONT_M`] of the box centre is the
/// reference byte for byte (the same `hashModel`). Literals.
#[test]
fn the_harbour_front_changes_nothing_off_or_when_the_sea_is_already_near() {
    let off = generate(4242, &opts(9, false));
    assert_eq!(off.anchors.market, Vec2::new(850.0, 625.0), "the reference's market: the box centre");
    assert_eq!(off.pin, off.anchors.market, "off, the pin is the market");
    // 35 columns of sea end 770 m east of the west side: 80 m from the centre.
    let near_off = generate(4242, &opts(35, false));
    let near_on = generate(4242, &opts(35, true));
    assert_eq!(hash_model(&near_on), hash_model(&near_off), "a sea 80 m away: the market stays");
    assert_eq!(near_on.pin, near_on.anchors.market);
}

/// Protects Ruling FK item 1 on a coast: the owner's case of a sea strip 198 m
/// wide at the box's west edge (483920 359:258 in `cfa1cf0d`'s frames, 650 m
/// from the walls). On, the market moves due west to [`HARBOUR_FRONT_M`] from
/// the sea, x = 198 + 100 = 298 (a literal), the pin stays at the box centre,
/// and the town's nearest house stands within one street of the water (23 m,
/// the census's `HARBOUR_STREET_GAP_M`), where off it stood hundreds of metres
/// inland, and the harbour street runs down to the water. MUTATION: dropping
/// the move, moving to the sea point itself, moving the pin with the market,
/// or not laying the harbour street fails a bar.
#[test]
fn a_harbour_town_lays_out_to_its_sea() {
    let off = generate(4242, &opts(9, false));
    let on = generate(4242, &opts(9, true));
    assert!((on.anchors.market.x - 298.0).abs() < 1e-9 && (on.anchors.market.y - 625.0).abs() < 1e-9, "market {:?}", on.anchors.market);
    assert_eq!(on.pin, Vec2::new(850.0, 625.0), "the pin stays on the map position");
    let (b_off, b_on) = (built_to_sea(&off, 9), built_to_sea(&on, 9));
    assert!(b_off > 200.0, "premise: off, the town stands inland of its sea ({b_off:.0} m)");
    assert!(b_on <= 23.0, "on, a house stands within one street of the sea ({b_on:.0} m)");
    assert!(on.harbour.is_some(), "and the harbour is built");
    // `harbour_street`: a live street runs west from the market along y = 625
    // to within 40 m of the strip (its last stretch is the wall's rampart
    // strip, which the fort-zone sweep clears).
    let at_shore = on.graph.edges.iter().any(|e| {
        let (a, b) = (on.graph.nodes[e.a].pt(), on.graph.nodes[e.b].pt());
        (a.y - 625.0).abs() < 5.0 && (b.y - 625.0).abs() < 5.0 && a.x.min(b.x) < 238.0
    });
    assert!(at_shore, "the harbour street runs down toward the water");
}

/// Protects the two-pass rule (`generate`): a town whose reference layout
/// already has a house of its body (`town_reaches_water`) within
/// [`HARBOUR_REACH_M`] of its water keeps that
/// layout byte for byte even with its box centre far from the sea (a city of
/// 20 000 fills its box to a 440 m strip, 410 m from the centre, measured 2 m
/// off it), while a hamlet that does not reach
/// it is laid out again from the water. MUTATION: dropping the reach test (always
/// re-laying) fails the first bar; never re-laying fails the second.
#[test]
fn a_town_already_built_to_its_water_keeps_the_references_layout() {
    let big = |front| generate(4242, &GenOpts { pop: Some(20000.0), ..opts(20, front) });
    let off = big(false);
    assert!(town_reaches_water(&off, &coast(20, true), HARBOUR_REACH_M), "premise: the city reaches its sea ({:.0} m)", built_to_sea(&off, 20));
    assert!(harbour_front_market(&build_site(7, 1700.0, 1250.0, "coast", SiteOpts { water: Some(coast(20, true)), ..Default::default() }), Vec2::new(850.0, 625.0)).is_some(), "premise: its centre is far enough to move");
    assert_eq!(hash_model(&big(true)), hash_model(&off), "built to its water: the reference layout");
    let hamlet = |front| generate(4242, &GenOpts { pop: Some(400.0), ..opts(9, front) });
    let h_off = hamlet(false);
    assert!(!town_reaches_water(&h_off, &coast(9, true), HARBOUR_REACH_M), "premise: the hamlet stands inland");
    let h_on = hamlet(true);
    assert_ne!(h_on.pin, h_on.anchors.market, "the hamlet is laid out again from the water");
}

/// Protects [`houses_reach_water`] with literals: one 10 m square house whose
/// west side stands 23 m east of the 198 m strip reaches it (the line is
/// inclusive), 24 m does not; a house whose only near point is an edge
/// midpoint (over a single lake cell) reaches it too; no raster, or no house, is `false`.
/// MUTATION: `<` for `<=`, or dropping the midpoints, fails a bar.
#[test]
fn houses_reach_water_measures_to_the_cells_edge() {
    let w = coast(9, true);
    let house = |x0: f64| crate::districts::Building {
        poly: vec![Vec2::new(x0, 600.0), Vec2::new(x0 + 10.0, 600.0), Vec2::new(x0 + 10.0, 610.0), Vec2::new(x0, 610.0)],
        ..test_building()
    };
    assert!(houses_reach_water(&[house(198.0 + 23.0)], &w, 23.0), "23 m: within one street");
    assert!(!houses_reach_water(&[house(198.0 + 24.0)], &w, 23.0), "24 m: not");
    // One lake cell (column 33, row 5: x 726..748, y 110..132) under a house
    // whose south-west and south-east corners are 35 m off it but whose south
    // edge's midpoint (737, 150) is 18 m off.
    let mut lake = coast(0, true);
    lake.open_water.as_mut().unwrap()[5 * MW + 33] = 1;
    let over = crate::districts::Building {
        poly: vec![Vec2::new(696.0, 150.0), Vec2::new(778.0, 150.0), Vec2::new(778.0, 190.0), Vec2::new(696.0, 190.0)],
        ..test_building()
    };
    assert!(houses_reach_water(&[over], &lake, 23.0), "an edge midpoint 18 m off reaches it");
    assert!(!houses_reach_water(&[house(200.0)], &coast(9, false), 23.0), "no raster");
    assert!(!houses_reach_water(&[], &w, 23.0), "no house");
}

/// A building with every field but its footprint at a neutral value, for
/// [`houses_reach_water`]'s literals (it reads only `poly`).
fn test_building() -> crate::districts::Building {
    crate::districts::Building {
        id: String::new(),
        poly: Vec::new(),
        ridge: [Vec2::new(0.0, 0.0); 2],
        parcel: String::new(),
        kind: "",
        district: "",
        age: 0.0,
        courtyard: false,
        prov: "",
    }
}

/// Protects [`harbour_front_market`]'s own rules, on `build_site` directly:
/// a `river` site keeps [`HARBOUR_FRONT_RIVER_M`] from its water, a coast
/// [`HARBOUR_FRONT_M`] (literals 318 and 298 on the 198 m strip); no
/// open-water raster, or one of the wrong size, or no open water at all, moves
/// nothing; a centre already within the distance stays. MUTATION: one distance
/// for both kinds, reading the stamped mask instead of the open-water raster,
/// or `<` for `<=` at the boundary fails a bar.
#[test]
fn the_market_keeps_the_references_distance_from_its_water() {
    let mc = Vec2::new(850.0, 625.0);
    let at = |kind: &str, w: WaterCtx| harbour_front_market(&build_site(7, 1700.0, 1250.0, kind, SiteOpts { water: Some(w), ..Default::default() }), mc);
    assert_eq!((HARBOUR_FRONT_M, HARBOUR_FRONT_RIVER_M), (100.0, 120.0), "placeAnchors' |d - 100| and |d - 120|");
    let coast_q = at("coast", coast(9, true)).expect("a coast 652 m away moves");
    assert!((coast_q.x - 298.0).abs() < 1e-9 && (coast_q.y - 625.0).abs() < 1e-9, "{coast_q:?}");
    let river_q = at("river", coast(9, true)).expect("and so does a river site's");
    assert!((river_q.x - 318.0).abs() < 1e-9, "{river_q:?}");
    assert_eq!(at("coast", coast(9, false)), None, "no open-water raster: nothing is guessed from the mask");
    assert_eq!(at("coast", WaterCtx { open_water: Some(vec![1; 4]), ..coast(9, true) }), None, "a raster of the wrong size");
    assert_eq!(at("coast", WaterCtx { open_water: Some(vec![0; MW * MH]), ..coast(9, true) }), None, "no open water");
    // 34 columns end at 748 m: 102 m from the centre, just over; 35 at 770 m: 80 m.
    assert!(at("coast", coast(34, true)).is_some_and(|q| (q.x - 848.0).abs() < 1e-9), "102 m away: moved 2 m");
    assert_eq!(at("coast", coast(35, true)), None, "80 m away: stays");
    // Exactly at the distance: a market 100 m east of the 198 m strip stays.
    let site = build_site(7, 1700.0, 1250.0, "coast", SiteOpts { water: Some(coast(9, true)), ..Default::default() });
    assert_eq!(harbour_front_market(&site, Vec2::new(298.0, 625.0)), None, "at the distance: stays");
    // One lake cell (column 33, row 5: x 726..748, y 110..132) nearer than the
    // strip: the market aims at the cell's nearest point, (748, 132), and stops
    // 100 m short of it on the line to the centre.
    let mut lake = coast(0, true);
    lake.open_water.as_mut().unwrap()[5 * MW + 33] = 1;
    let q = at("coast", lake).expect("a lake cell 503 m away");
    let (dx, dy) = (850.0 - 748.0, 625.0 - 132.0);
    let n = (dx * dx + dy * dy as f64).sqrt();
    assert!((q.x - (748.0 + dx / n * 100.0)).abs() < 1e-9 && (q.y - (132.0 + dy / n * 100.0)).abs() < 1e-9, "toward the cell's nearest point: {q:?}");
}

/// Protects the host's roads under the harbour front
/// (`build_primaries_from_paths_at`): a road's offsets are laid from the given
/// origin, so with the pin as origin the road stays where the map draws it
/// while its inner end is still anchored on the moved market; with the market
/// as origin it is `build_primaries_from_paths` exactly (the reference).
/// MUTATION: laying the offsets from the market fails the first bar.
#[test]
fn the_hosts_roads_stay_on_the_map_when_the_market_moves() {
    let site = build_site(7, 1700.0, 1250.0, "coast", SiteOpts { water: Some(coast(9, true)), ..Default::default() });
    let anchors = Anchors { market: Vec2::new(298.0, 625.0), prov: "test" };
    let path = vec![vec![Vec2::new(0.0, 0.0), Vec2::new(0.0, -200.0), Vec2::new(0.0, -400.0), Vec2::new(0.0, -600.0)]];
    let pin = Vec2::new(850.0, 625.0);
    let mut g = crate::graph::Graph::new();
    let r = build_primaries_from_paths_at(1, &site, &anchors, &mut g, &path, pin);
    let pts = &r[0].pts;
    assert_eq!(pts[0], anchors.market, "the inner end is anchored on the market");
    assert!(pts.iter().any(|p| (p.x - 850.0).abs() < 1.0 && p.y < 300.0), "the road runs north from the pin, as on the map: {pts:?}");
    let mut g1 = crate::graph::Graph::new();
    let mut g2 = crate::graph::Graph::new();
    let a = build_primaries_from_paths(1, &site, &anchors, &mut g1, &path);
    let b = build_primaries_from_paths_at(1, &site, &anchors, &mut g2, &path, anchors.market);
    assert_eq!(a.iter().map(|r| r.pts.clone()).collect::<Vec<_>>(), b.iter().map(|r| r.pts.clone()).collect::<Vec<_>>(), "with the market as origin, the reference");
}

/// Protects `crate::routes::dry_join`, the moved market's join to a road
/// (Ruling FK item 1): a lake (cells 23..32 by 23..34, x 506..726 and y
/// 506..770) stands between the market (298, 625) and the road's first two
/// points, so the join skips them and takes the third, (850, 225), the nearest
/// point a dry line reaches; a wet point is never taken; and with no dry join
/// at all the answer is `None`; and the road laid from the pin keeps out of
/// the lake. MUTATION: dropping the line test returns 0; dropping the point
/// test lets a point inside the lake win; not using the join in
/// `build_primaries_from_paths_at` lays the road through the lake.
#[test]
fn the_moved_market_joins_a_road_by_a_dry_line() {
    let mut w = coast(9, true);
    for j in 23..35 {
        for i in 23..33 {
            w.mask[j * MW + i] = 1;
            w.open_water.as_mut().unwrap()[j * MW + i] = 1;
        }
    }
    let site = build_site(7, 1700.0, 1250.0, "coast", SiteOpts { water: Some(w), ..Default::default() });
    let m = Vec2::new(298.0, 625.0);
    let road = [Vec2::new(850.0, 625.0), Vec2::new(850.0, 425.0), Vec2::new(850.0, 225.0), Vec2::new(850.0, 25.0)];
    assert!(site.is_water(Vec2::new(600.0, 600.0)), "premise: the lake");
    assert_eq!(crate::routes::dry_join(&site, m, &road), Some(2), "the nearest point a dry line reaches");
    // A point in the lake itself, nearer than any other, is never the join.
    let wet_first = [Vec2::new(520.0, 625.0), Vec2::new(850.0, 225.0)];
    assert_eq!(crate::routes::dry_join(&site, m, &wet_first), Some(1));
    // Every point behind the lake: no join.
    let behind = [Vec2::new(850.0, 625.0), Vec2::new(900.0, 625.0)];
    assert_eq!(crate::routes::dry_join(&site, m, &behind), None);
    // Through `build_primaries_from_paths_at` with the pin as origin: the road
    // joins the market at (850, 225), so none of its laid points is in the lake.
    let anchors = Anchors { market: m, prov: "test" };
    let offsets = vec![road.iter().map(|p| Vec2::new(p.x - 850.0, p.y - 625.0)).collect::<Vec<_>>()];
    let mut g = crate::graph::Graph::new();
    let r = build_primaries_from_paths_at(1, &site, &anchors, &mut g, &offsets, Vec2::new(850.0, 625.0));
    assert!(r[0].pts.iter().all(|p| !site.is_water(*p)), "the laid road stays out of the lake: {:?}", r[0].pts);
}

/// Protects `harbour_street` (Ruling FK item 1, "its streets reaching the
/// water"): from the market (298, 625) to the 198 m strip it lays one street
/// ending 5 m short of the water, at (203, 625) (a literal); when a wet band
/// that is not open water (a stamped river band, mask only: cells 11..12 by
/// 25..32) blocks every line to the sea within twice that distance, it takes
/// the nearest open water a dry line reaches, a pond whose nearest point is
/// (298, 462), and ends at (298, 467); with nothing reachable it lays nothing.
/// MUTATION: dropping the inset, the dry test or the fallback fails a bar.
#[test]
fn the_harbour_street_runs_from_the_market_to_the_water() {
    let m = Vec2::new(298.0, 625.0);
    let q = Vec2::new(198.0, 625.0);
    let lay = |w: WaterCtx| {
        let site = build_site(7, 1700.0, 1250.0, "coast", SiteOpts { water: Some(w), ..Default::default() });
        let mut g = crate::graph::Graph::new();
        let end = super::harbour_street(&site, &mut g, m, q);
        (end, g.edges.iter().filter(|e| e.alive).count())
    };
    let (end, n) = lay(coast(9, true));
    assert_eq!((end, n), (Some(Vec2::new(203.0, 625.0)), 1), "straight to the strip, 5 m short");
    let mut banded = coast(9, true);
    for j in 25..33 {
        for i in 11..13 {
            banded.mask[j * MW + i] = 1;
        }
    }
    assert_eq!(lay(banded.clone()), (None, 0), "every line within reach crosses the band: nothing laid");
    let mut pond = banded;
    pond.mask[20 * MW + 13] = 1;
    pond.open_water.as_mut().unwrap()[20 * MW + 13] = 1;
    let (end, n) = lay(pond);
    assert_eq!(n, 1);
    let e = end.expect("the pond");
    assert!((e.x - 298.0).abs() < 1e-9 && (e.y - 467.0).abs() < 1e-9, "{e:?}");
}

/// Protects the second layout's river (`GenOpts::water_to_sea`, Ruling FK
/// item 1): a city already built to its 440 m strip keeps its market, but
/// when the host carried a river on to that sea the town is laid out again
/// from that river and keeps it when it houses enough (the river runs east
/// to west along y = 625 into the strip); with `harbour_front` off the
/// carried river is never read. MUTATION: dropping the `to_sea` term of
/// `changed`, or laying the second layout from `water`, fails a bar.
#[test]
fn a_river_carried_to_the_sea_is_the_second_layouts_river() {
    let river: Vec<Vec2> = (0..=12).map(|k| Vec2::new(1700.0 - 110.0 * k as f64, 625.0)).collect();
    let mut ws = coast(20, true);
    for j in 0..MH {
        let cy = (j as f64 + 0.5) * CELL;
        if (cy - 625.0).abs() <= 10.0 + CELL / 2.0 {
            for i in 0..MW {
                ws.mask[j * MW + i] = 1;
            }
        }
    }
    ws.river_path = Some(river.clone());
    ws.river_width_m = Some(20.0);
    ws.river_order = 2.0;
    let o = |front: bool| GenOpts { pop: Some(20000.0), water_to_sea: Some(ws.clone()), ..opts(20, front) };
    let on = generate(4242, &o(true));
    assert_eq!(on.site.river, river, "the second layout is laid out from the carried river");
    assert!(on.pop >= 0.9 * 20000.0_f64.min(generate(4242, &o(false)).pop), "and houses enough to be kept");
    let off = generate(4242, &o(false));
    assert_ne!(off.site.river, river, "off: the carried river is never read");
}
