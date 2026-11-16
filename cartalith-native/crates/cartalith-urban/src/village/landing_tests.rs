//! The street village's river leftovers of 2026-11-16 ([`super::road_bank_market`],
//! [`super::landing_market`], [`super::bank_behind`] through [`super::plat_tofts`],
//! and the `generate` path that calls them). Each test says what it protects.
//! The fixture is a straight east-west river across a real water raster with
//! no sea or lake (the riverbank tests' own shape, `crate::riverbank`'s
//! `tests.rs`), so every number below is checkable by hand. Not a port: no
//! reference to diff.

use super::*;
use crate::generate::{GenOpts, generate};
use crate::site::{SiteOpts, WaterCtx, build_site};

const CELL: f64 = 22.0;
const MW: usize = 77;
const MH: usize = 57;
/// The river's width, metres (`um_water_ctx`'s `10 + 7 * order` at order 2).
const RW: f64 = 24.0;

/// The fixture's water: a channel along `y = river_y` (on a cell boundary, so
/// the stamped band is symmetric), every column, and an all-dry open-water
/// raster (no sea or lake).
fn water(river_y: f64) -> WaterCtx {
    let band = RW / 2.0 + CELL / 2.0;
    let mut mask = vec![0u8; MW * MH];
    for j in 0..MH {
        if ((j as f64 + 0.5) * CELL - river_y).abs() <= band {
            for i in 0..MW {
                mask[j * MW + i] = 1;
            }
        }
    }
    let wet: Vec<usize> = (0..MH).filter(|&j| mask[j * MW] == 1).collect();
    let mut dt = vec![0.0; MW * MH];
    for j in 0..MH {
        let d = wet.iter().map(|&w| (w as f64 - j as f64).abs()).fold(f64::INFINITY, f64::min);
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
        river_path: Some((0..=17).map(|k| Vec2::new(k as f64 * 100.0, river_y)).collect()),
        river_width_m: Some(RW),
        river_order: 2.0,
        sea_lake_cells: 0.0,
        open_water: Some(vec![0u8; MW * MH]),
    }
}

/// The riverbank site `generate` builds on [`water`] (the centreline
/// resampled every 8 m first).
fn bank_site(river_y: f64) -> Site {
    let mut w = water(river_y);
    w.river_path = crate::riverbank::fine_river_path(w.river_path.as_deref().unwrap(), 1700.0, 1250.0);
    let mut site = build_site(7, 1700.0, 1250.0, "river", SiteOpts { water: Some(w), ..Default::default() });
    assert!(site.use_riverbank(), "premise: the fixture takes the riverbank geometry");
    site
}

/// The river 33 cells down (726 m), 101 m south of the box centre.
const RIVER_Y: f64 = 33.0 * CELL;

/// Protects `landing_market`'s distance and its trigger (literals): a market
/// 101 m from the centreline (89 m from the 12 m half-width edge) moves
/// straight toward the river until it stands `LANDING_M` (32 m) from the
/// edge, at y = 726 - 12 - 32 = 682; one already within that stays; a site
/// with no river line stays.
#[test]
fn the_landing_takes_a_village_down_to_its_river_and_no_closer() {
    let site = bank_site(RIVER_Y);
    let q = landing_market(&site, Vec2::new(850.0, 625.0)).expect("101 m inland: it moves");
    assert!((q.x - 850.0).abs() < 1e-9 && (q.y - 682.0).abs() < 1e-9, "landing at {q:?}");
    assert!(landing_market(&site, Vec2::new(850.0, 682.0)).is_none(), "exactly 32 m from the edge: it stays");
    assert!(landing_market(&site, Vec2::new(850.0, 690.0)).is_none(), "24 m from the edge: it stays");
    assert!(landing_market(&site, Vec2::new(850.0, 681.0)).is_some(), "33 m from the edge: it moves");
    let mut dry = build_site(7, 1700.0, 1250.0, "landlocked", SiteOpts::default());
    dry.river.clear();
    assert!(landing_market(&dry, Vec2::new(850.0, 625.0)).is_none(), "no river line: no landing");
}

/// A market 14 m north of the channel's edge (y = 700), for the road-bank tests.
const NEAR: Vec2 = Vec2::new(850.0, 700.0);

/// Protects `road_bank_market`'s choice of bank (literals): a road running
/// south from the market across the river (200 m of it on the south bank
/// within `max_rf` 200, none on the north) takes the market to the south
/// bank, onto the first ring point (60 m out, 15-degree steps from east: 45
/// degrees, 16.4 m from the centreline) that is dry, south of the channel and
/// joined to the road by a dry line; a road
/// running north keeps it; equal road on each bank keeps it.
#[test]
fn a_village_market_goes_to_the_bank_its_road_is_on() {
    let site = bank_site(RIVER_Y);
    let south: Vec<Vec2> = (0..=5).map(|k| Vec2::new(0.0, k as f64 * 100.0)).collect();
    let north: Vec<Vec2> = (0..=5).map(|k| Vec2::new(0.0, -(k as f64) * 100.0)).collect();
    let q = road_bank_market(&site, NEAR, NEAR, std::slice::from_ref(&south), 200.0).expect("the road is across the river");
    assert_ne!(site.bank_side(q), site.bank_side(NEAR), "on the road's bank");
    assert!((q.dist(NEAR) - 60.0).abs() < 1e-9, "the 60 m ring, {:.3} m", q.dist(NEAR));
    let ang = (q.y - NEAR.y).atan2(q.x - NEAR.x).to_degrees();
    assert!((ang - 45.0).abs() < 1e-6, "the first dry bearing south of the river, {ang:.3} degrees");
    assert!(site.river_dist(q) >= RW / 2.0 + 1.0 && !site.is_water(q), "dry and clear of the channel");
    assert!(road_bank_market(&site, NEAR, NEAR, std::slice::from_ref(&north), 200.0).is_none(), "the road is on its own bank");
    // 200 m within reach on each bank (the south road's first leg straddles
    // the river with its midpoint on the south bank; the north road's first
    // two legs are 100 m each).
    let short_south: Vec<Vec2> = (0..=2).map(|k| Vec2::new(0.0, k as f64 * 100.0)).collect();
    assert!(road_bank_market(&site, NEAR, NEAR, &[short_south, north.clone()], 200.0).is_none(), "a tie keeps the market");
    let mut dry = build_site(7, 1700.0, 1250.0, "landlocked", SiteOpts::default());
    dry.river.clear();
    assert!(road_bank_market(&dry, NEAR, NEAR, &[south], 200.0).is_none(), "no river line: nothing to cross");
}

/// Protects `road_bank_market`'s reach and its offsets (literals): road
/// beyond `ROAD_BANK_REACH_RF * max_rf` of the market does not count (the
/// road below keeps the market at `max_rf` 40 and moves it at 200), and the
/// paths are offsets from `origin`, not from the market: the same road laid
/// from an origin 300 m east is over 300 m away.
#[test]
fn only_the_road_near_the_market_counts() {
    let site = bank_site(RIVER_Y);
    // Legs' midpoints 50, 175 and 325 m south of the market, all on the
    // south bank: none within 40 m, 250 m of road within 200 m.
    let far_south: Vec<Vec2> = vec![Vec2::new(0.0, 0.0), Vec2::new(0.0, 100.0), Vec2::new(0.0, 250.0), Vec2::new(0.0, 400.0)];
    assert!(road_bank_market(&site, NEAR, NEAR, std::slice::from_ref(&far_south), 40.0).is_none(), "no road within the reach");
    assert!(road_bank_market(&site, NEAR, NEAR, std::slice::from_ref(&far_south), 200.0).is_some(), "premise: within 200 m it counts");
    let east = Vec2::new(NEAR.x + 300.0, NEAR.y);
    assert!(road_bank_market(&site, NEAR, east, &[far_south], 200.0).is_none(), "laid from the origin 300 m east: out of reach");
}

/// Protects `bank_behind` through `plat_tofts` (literal): a road along the
/// bank, 20 m north of the channel's edge (centre y = 694, 6 m wide), gets
/// tofts on its river side too, cut back `TOFT_BANK_CLEAR_M` (1 m) short of
/// the edge: front line at 694 + 3 + 1.4 = 698.4, and the last whole metre
/// still 13 m or more from the centreline (726) is t = 14 (712.4). Without the
/// cut a 30 m toft there stands in the river and none is laid.
#[test]
fn a_road_along_the_bank_has_tofts_on_its_river_side() {
    let site = bank_site(RIVER_Y);
    let mut g = Graph::new();
    g.add_street(250.0, 694.0, 1450.0, 694.0, "primary", 6.0, 0, "test road");
    let t = plat_tofts(1, &g, &site, &[], &[], Vec2::new(850.0, 694.0), 200.0, 208.0, 8, false);
    let river_side: Vec<&Parcel> = t.iter().filter(|p| p.poly[0].y > 694.0).collect();
    assert!(!river_side.is_empty(), "tofts between the road and the river");
    for p in &river_side {
        assert!((p.depth - 14.0).abs() < 1e-9, "river-side toft {} m deep", p.depth);
        assert!(p.poly.iter().all(|v| site.river_dist(*v) >= RW / 2.0 + 0.5), "a toft corner in the channel");
    }
    assert!(t.iter().any(|p| p.poly[0].y < 694.0 && (p.depth - 30.0).abs() < 1e-9), "the land side keeps full depth");
}

/// Protects the `generate` path (rides `urban_street_village`): a street
/// village whose box centre stands 375 m north of its river (y = 1 000) takes
/// its market down to the landing, `LANDING_M` (32 m) from the channel's edge
/// at the most (the ring search may set it a step further on dry ground
/// joined to a road), keeps the box centre as its map pin, and houses its
/// target there with a house within two street widths (10.8 m) of the bank;
/// with the switch off the market is the box centre and every house is over
/// 300 m from the water.
#[test]
fn a_street_village_inland_of_its_river_is_laid_out_at_the_landing() {
    let opts = |sv: bool| GenOpts { site: Some("river".into()), pop: Some(400.0), water: Some(water(1000.0)), riverbank: true, street_village: sv, walls: Some(false), ..GenOpts::default() };
    let gap = |t: &crate::generate::Town| {
        t.buildings.iter().flat_map(|b| b.poly.iter()).map(|v| ((v.y - 1000.0).abs() - RW / 2.0).max(0.0)).fold(f64::INFINITY, f64::min)
    };
    let off = generate(5, &opts(false));
    assert!((off.anchors.market.y - 625.0).abs() < 1e-9 && off.pin == off.anchors.market, "off: the box centre, {:?}", off.anchors.market);
    assert!(gap(&off) > 300.0, "premise: off, the nearest house is {:.0} m from the water", gap(&off));
    let on = generate(5, &opts(true));
    let edge = 1000.0 - RW / 2.0 - on.anchors.market.y;
    assert!((LANDING_M - 1e-9..=LANDING_M + 30.0).contains(&edge), "on: the market {edge:.1} m from the edge");
    assert_eq!(on.pin, Vec2::new(850.0, 625.0), "the map pin stays at the box centre");
    assert!(gap(&on) <= 10.8, "on: the nearest house {:.1} m from the water", gap(&on));
    assert!(on.pop >= 0.8 * on.pop_target, "on: housed {} of {}", on.pop, on.pop_target);
}

/// Protects `road_bank_market`'s river-end rule (`abeam`, literals): a river
/// that ends 100 m short of the market (the fixture's line from x = 0 to
/// 750, market at x = 850) has no other bank there, so a road running off
/// toward the line's far side moves nothing; the same road abeam of the line
/// (market at x = 600) still moves the market.
#[test]
fn there_is_no_other_bank_beyond_the_river_end() {
    let mut site = bank_site(RIVER_Y);
    site.river.retain(|p| p.x <= 750.0);
    let south: Vec<Vec2> = (0..=5).map(|k| Vec2::new(0.0, k as f64 * 100.0)).collect();
    assert!(road_bank_market(&site, NEAR, NEAR, std::slice::from_ref(&south), 200.0).is_none(), "beyond the river's end");
    let abeam_m = Vec2::new(600.0, 700.0);
    assert!(road_bank_market(&site, abeam_m, abeam_m, &[south], 200.0).is_some(), "abeam of the river");
}

/// Protects each of `road_bank_market`'s three `abeam` guards on its own
/// (literals; the river line cut at x = 750, its resampled end at about
/// (744-750, 726)):
/// - the market's: a market beyond the end (850, 700) whose road runs off
///   south-west abeam of the river is left where it is (the mutation harness
///   finds this case also held by the other two guards: `mutants.txt`, 19);
/// - the ring point's: a market abeam (730, 700) whose road runs south is
///   moved to a point abeam of the river (x at or short of the line's end),
///   never to the first dry "south" point past the end (782, 730);
/// - the road's: a road that runs off east past the river's end, south of its
///   extension, counts for no bank, so a market abeam keeps its place.
#[test]
fn each_abeam_guard_holds_on_its_own() {
    let mut site = bank_site(RIVER_Y);
    site.river.retain(|p| p.x <= 750.0);
    let end = site.river.last().unwrap().x;
    assert!((740.0..=750.0).contains(&end), "premise: the 8 m resampled line ends just short of x = 750 ({end})");
    let beyond = Vec2::new(850.0, 700.0);
    let sw = vec![Vec2::new(0.0, 0.0), Vec2::new(-150.0, 100.0), Vec2::new(-250.0, 150.0)];
    assert!(road_bank_market(&site, beyond, beyond, &[sw], 200.0).is_none(), "a market beyond the river's end stays");
    let near_end = Vec2::new(730.0, 700.0);
    let south: Vec<Vec2> = (0..=2).map(|k| Vec2::new(0.0, k as f64 * 100.0)).collect();
    let q = road_bank_market(&site, near_end, near_end, &[south], 200.0).expect("abeam: it moves");
    assert!(q.x <= end + 1e-9, "the moved market stands abeam of the river, at {q:?}");
    let east = vec![Vec2::new(0.0, 0.0), Vec2::new(100.0, 60.0), Vec2::new(250.0, 60.0)];
    assert!(road_bank_market(&site, near_end, near_end, &[east], 200.0).is_none(), "road past the river's end counts for no bank");
}

/// Protects `generate_pass`'s `pin.get_or_insert` (the map pin through both
/// moves): a street village whose box centre stands 375 m north of its river
/// (y = 1 000) and whose only road lies across it (the host's road from 350 m
/// south of the pin onward) is taken down to the landing and then across to
/// the road's bank; its pin stays the box centre, where the map draws it,
/// not the landing it passed through.
#[test]
fn the_map_pin_survives_both_moves() {
    let road: Vec<Vec2> = (0..4).map(|k| Vec2::new(0.0, 350.0 + 100.0 * k as f64)).collect();
    let opts = GenOpts {
        site: Some("river".into()),
        pop: Some(400.0),
        water: Some(water(1000.0)),
        riverbank: true,
        street_village: true,
        walls: Some(false),
        primary_paths: vec![road],
        ..GenOpts::default()
    };
    let t = generate(5, &opts);
    assert!(t.anchors.market.y > 1000.0, "premise: the market crossed to the road's bank, at {:?}", t.anchors.market);
    assert_eq!(t.pin, Vec2::new(850.0, 625.0), "the pin is the box centre");
}
