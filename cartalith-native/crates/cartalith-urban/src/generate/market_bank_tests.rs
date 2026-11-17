//! The gate of `GenOpts::market_road_bank` (the 483920 #4 under-build,
//! 2026-11-17): a river town whose box centre stands in the channel takes its
//! market on the bank holding more of the host's road. Each test says what it
//! protects. The fixture is a straight river along the box at y = 627 m (a
//! raster row centre), stamped the way `cartalith_civ`'s `finish_water_ctx`
//! stamps one, with the riverbank geometry on, so the channel is the
//! centreline plus half the 24 m width, y 615 to 639 m, and the box centre
//! (850, 625) is in it. The reference's nudge then lands the market at its
//! first dry ring point, (875.98, 640) on the south bank (ring 30 m, bearing
//! 30 degrees), and the host's one road comes from the north, ending at that
//! market: about 575 m of it on the north bank against 1 m on the south.

use super::{GenOpts, generate, hash_model, keep_road_bank_move};
use crate::geom::Vec2;
use crate::site::WaterCtx;

const CELL: f64 = 22.0;
const MW: usize = 77;
const MH: usize = 57;
/// The river's width, metres (`um_water_ctx`'s `10 + 7 * order` at order 2).
const RW: f64 = 24.0;

/// A river along the box at `river_y` (metres), real open water supplied (all
/// dry: no sea or lake), so the site takes the riverbank geometry.
fn river(river_y: f64) -> WaterCtx {
    let band = RW / 2.0 + CELL / 2.0;
    let mut mask = vec![0u8; MW * MH];
    for j in 0..MH {
        if ((j as f64 + 0.5) * CELL - river_y).abs() <= band {
            for i in 0..MW {
                mask[j * MW + i] = 1;
            }
        }
    }
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
        river_path: Some((0..=17).map(|k| Vec2::new(k as f64 * 100.0, river_y)).collect()),
        river_width_m: Some(RW),
        river_order: 2.0,
        sea_lake_cells: 0.0,
        open_water: Some(vec![0u8; MW * MH]),
    }
}

/// The host's road, offsets from the reference's market (875.98, 640): due north
/// for 600 m, so it crosses the river and ends at the market.
fn north_road() -> Vec<Vec<Vec2>> {
    vec![(0..=6).map(|k| Vec2::new(0.0, -600.0 + 100.0 * k as f64)).collect()]
}

fn opts(river_y: f64, on: bool, roads: bool) -> GenOpts {
    GenOpts {
        site: Some("river".into()),
        pop: Some(6000.0),
        water: Some(river(river_y)),
        riverbank: true,
        primary_paths: if roads { north_road() } else { Vec::new() },
        market_road_bank: on,
        ..GenOpts::default()
    }
}

/// Protects the fix itself: with the switch off the market is the
/// reference's first dry ring point on the south bank (a literal, by hand
/// above); on, it goes across to the north bank its road holds, the town's
/// map pin stays the reference's market, and the town houses more people.
#[test]
fn a_wet_centre_market_goes_to_the_bank_its_road_holds() {
    let off = generate(5, &opts(627.0, false, true));
    let on = generate(5, &opts(627.0, true, true));
    // (850 + 30 cos 30, 625 + 30 sin 30): the nudge's ring 30 m, bearing 2 of 24.
    assert!((off.anchors.market.x - 875.98).abs() < 0.01 && (off.anchors.market.y - 640.0).abs() < 1e-6, "the reference's nudge: {:?}", off.anchors.market);
    assert!(on.anchors.market.y < 615.0, "the market went to the road's (north) bank: {:?}", on.anchors.market);
    assert_eq!(on.pin, off.anchors.market, "the map anchor stays the reference's market");
    assert!(on.road_bank_moved && !off.road_bank_moved, "the move is recorded for `generate`'s keep test");
    assert_eq!(off.pin, off.anchors.market, "off, the reference's anchor is its market");
    assert!(on.pop > off.pop, "on the road's bank the town houses more: {} against {}", on.pop, off.pop);
}

/// Protects the scope: a dry box centre is the reference's own market choice
/// and is left exactly alone, even with its road on the other bank (the river
/// at y = 605 m runs 8 m north of the centre's bank, and the road comes from
/// the north across it); and a town with no host roads has nothing to join.
/// Identity by `hash_model`.
#[test]
fn a_dry_centre_or_a_roadless_town_is_untouched() {
    let dry = generate(5, &opts(605.0, false, true));
    assert_eq!((dry.anchors.market.x, dry.anchors.market.y), (850.0, 625.0), "premise: the centre is dry and is the market");
    let dry_on = generate(5, &opts(605.0, true, true));
    assert!(!dry_on.road_bank_moved);
    assert_eq!(hash_model(&dry), hash_model(&dry_on));
    assert_eq!(hash_model(&generate(5, &opts(627.0, false, false))), hash_model(&generate(5, &opts(627.0, true, false))));
}

/// Protects the street village's own rule from a second move: a village of
/// 400 under `street_village` already takes its roads' bank
/// (`crate::village::road_bank_market`), and this switch adds nothing to it.
#[test]
fn a_street_village_is_left_to_its_own_rule() {
    let v = |on: bool| GenOpts { pop: Some(400.0), street_village: true, ..opts(627.0, on, true) };
    assert_eq!(hash_model(&generate(5, &v(false))), hash_model(&generate(5, &v(true))));
}

/// Protects the riverbank gate: without the riverbank geometry (the
/// reference's stamped-mask site) the switch moves nothing.
#[test]
fn without_the_riverbank_geometry_nothing_moves() {
    let o = |on: bool| GenOpts { riverbank: false, ..opts(627.0, on, true) };
    assert_eq!(hash_model(&generate(5, &o(false))), hash_model(&generate(5, &o(true))));
}

/// Protects `generate`'s keep-or-revert test for a moved market: a move that
/// houses fewer is dropped, a tie keeps it, and housing past the target counts
/// only up to it (literals, by hand).
#[test]
fn a_move_is_kept_only_where_it_houses_as_many() {
    assert!(keep_road_bank_move((900.0, 1000.0), (900.0, 1000.0)), "a tie keeps the move");
    assert!(!keep_road_bank_move((899.0, 1000.0), (900.0, 1000.0)), "fewer housed drops it");
    assert!(keep_road_bank_move((1000.0, 1000.0), (1500.0, 1000.0)), "past the target counts only to it");
    assert!(keep_road_bank_move((901.0, 1000.0), (900.0, 1000.0)));
}
