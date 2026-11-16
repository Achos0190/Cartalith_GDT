//! Ruling FN's stretch (`crate::stretch`): the plan's target and each of its
//! four cuts, the frame's distance, the layout box, and a whole layout grown
//! with and without it.
//!
//! The fixture is a straight river along `y = 3000` through a 6 km square
//! site (flat relief, so every bank is buildable unless a test makes it not),
//! vertices every 20 m, 24 m wide, with an open-water raster of the mask's
//! size so the riverbank geometry takes the site, as the app's sites do.

use super::*;
use crate::generate::{GenOpts, generate};
use crate::site::{SiteOpts, TerrainCtx, WaterCtx, build_site};

const BOX: f64 = 6000.0;
const CELL: f64 = 22.0;
const RIVER_Y: f64 = 3000.0;

fn cells() -> usize {
    (BOX / CELL).round() as usize
}

/// The straight river's water context; `open` marks open-water cells by
/// `(i, j)`.
fn water(open: &dyn Fn(usize, usize) -> bool) -> WaterCtx {
    let n = cells();
    let path: Vec<Vec2> = (0..=300).map(|k| Vec2::new(20.0 * k as f64, RIVER_Y)).collect();
    let mut ow = vec![0u8; n * n];
    for j in 0..n {
        for i in 0..n {
            ow[j * n + i] = u8::from(open(i, j));
        }
    }
    WaterCtx {
        mask: ow.clone(),
        dt: vec![10.0; n * n],
        mw: n,
        mh: n,
        cell_m: CELL,
        river_path: Some(path),
        river_width_m: Some(24.0),
        river_order: 2.0,
        sea_lake_cells: 0.0,
        open_water: Some(ow),
    }
}

/// Flat relief (slope 0 everywhere).
fn flat() -> TerrainCtx {
    let n = cells();
    TerrainCtx { grid: vec![0.5; n * n], mw: n, mh: n, cell_m: CELL, h_min: 0.5, h_max: 0.5 }
}

fn site(open: &dyn Fn(usize, usize) -> bool) -> Site {
    let mut s = build_site(7, BOX, BOX, "river", SiteOpts { water: Some(water(open)), terrain: Some(flat()), economy: None });
    assert!(s.use_riverbank(), "premise: the fixture takes the riverbank geometry");
    s
}

const MARKET: Vec2 = Vec2 { x: 3000.0, y: RIVER_Y };

/// Protects the plan's target: a 20 000 town (`max_rf` 720) on an open river
/// reaches `min(720^2 / 450, 1.5 x 720)` = 1 080 m each way, and its depth is
/// `720^2 / 1 080` = 480 m (the area kept).
#[test]
fn an_open_river_stretches_a_big_town_to_its_cap_both_ways() {
    let p = plan_river_stretch(&site(&|_, _| false), MARKET, 720.0, 20000.0, &[]).expect("a big town on an open river stretches");
    assert_eq!(p, StretchPlan { ahead_m: 1080.0, behind_m: 1080.0, across_m: 480.0 });
}

/// Protects the walk's open-water cut: a lake from x = 3 608 m on (cells
/// `i >= 164`) ends the reach downstream at the last step before it (600 m),
/// the depth widens to keep the area (`720^2 / ((600 + 1 080) / 2)`), and the
/// cut side is held at that depth so the town's end stays round.
#[test]
fn open_water_downstream_cuts_the_reach_on_that_side() {
    let p = plan_river_stretch(&site(&|i, _| i >= 164), MARKET, 720.0, 20000.0, &[]).expect("still stretched upstream");
    assert!((p.across_m - 720.0 * 720.0 / 840.0).abs() < 1e-9, "{p:?}");
    assert_eq!(p.ahead_m, p.across_m, "{p:?}");
    assert_eq!(p.behind_m, 1080.0);
    // A pond two cells (44 m) long across the river at x = 3 608: short
    // enough for the bank-gap rule to bridge, so only the open-water cut
    // stops the walk there.
    let pond = |i: usize, j: usize| (164..166).contains(&i) && ((j as f64 + 0.5) * CELL - RIVER_Y).abs() < 120.0;
    let p = plan_river_stretch(&site(&pond), MARKET, 720.0, 20000.0, &[]).expect("still stretched upstream");
    assert_eq!(p.ahead_m, p.across_m, "the river entering open water ends the reach: {p:?}");
}

/// Protects the neighbour cuts. A neighbour 3 km upstream with a 2 km reach
/// stops the walk where it comes within 2 000 + 120 m (x = 2 120, 880 m from
/// the market); one 1.6 km upstream with no reach stops it at the half-way
/// line (800 m).
#[test]
fn a_neighbour_stops_the_stretch_short_of_its_reach_and_the_half_way_line() {
    let s = site(&|_, _| false);
    let far = [StretchNeighbour { at: Vec2::new(0.0, RIVER_Y), reach_m: 2000.0 }];
    let p = plan_river_stretch(&s, MARKET, 720.0, 20000.0, &far).expect("stretched");
    assert_eq!(p.behind_m, 880.0, "{p:?}");
    assert_eq!(p.ahead_m, 1080.0);
    let near = [StretchNeighbour { at: Vec2::new(1400.0, RIVER_Y), reach_m: 0.0 }];
    let p = plan_river_stretch(&s, MARKET, 720.0, 20000.0, &near).expect("stretched");
    assert_eq!(p.behind_m, 800.0, "{p:?}");
}

/// Protects the bank-gap cut: both banks under water (the probe points 57 m
/// off the centreline) for x in [2 100, 2 400) upstream is a gap over
/// `STRETCH_GAP_M`, so the reach stops at the last buildable step before it
/// (600 m: the depth keeps the area over the mean reach (1 080 + 600) / 2,
/// and the cut side is held at it); a gap of 100 m (x in [2 300, 2 400)) is
/// bridged, and so is one bank under water for 600 m (the other is
/// buildable).
#[test]
fn a_long_unbuildable_bank_ends_the_stretch_and_a_short_one_does_not() {
    let banks = |x0: f64, x1: f64| {
        move |i: usize, j: usize| {
            let (x, y) = ((i as f64 + 0.5) * CELL, (j as f64 + 0.5) * CELL);
            (x0..x1).contains(&x) && (y - RIVER_Y).abs() > 40.0 && (y - RIVER_Y).abs() < 75.0
        }
    };
    let long = plan_river_stretch(&site(&banks(2100.0, 2400.0)), MARKET, 720.0, 20000.0, &[]).expect("stretched downstream");
    assert!((long.across_m - 720.0 * 720.0 / 840.0).abs() < 1e-9, "a 300 m wet bank ends the reach at 600 m ({long:?})");
    assert_eq!(long.behind_m, long.across_m, "{long:?}");
    let short = plan_river_stretch(&site(&banks(2300.0, 2400.0)), MARKET, 720.0, 20000.0, &[]).expect("stretched");
    assert_eq!(short.behind_m, 1080.0, "a 100 m wet bank is bridged ({short:?})");
    let one = move |i: usize, j: usize| {
        let (x, y) = ((i as f64 + 0.5) * CELL, (j as f64 + 0.5) * CELL);
        (1800.0..2400.0).contains(&x) && y - RIVER_Y > 40.0 && y - RIVER_Y < 75.0
    };
    let one = plan_river_stretch(&site(&one), MARKET, 720.0, 20000.0, &[]).expect("stretched");
    assert_eq!(one.behind_m, 1080.0, "one bank buildable carries the town on ({one:?})");
}

/// Protects the three refusals: a village, a town no deeper than the target
/// depth, and a site with no centreline.
#[test]
fn small_towns_and_riverless_sites_are_not_stretched() {
    let s = site(&|_, _| false);
    assert!(plan_river_stretch(&s, MARKET, 720.0, 1499.0, &[]).is_none(), "a village");
    assert!(plan_river_stretch(&s, MARKET, 450.0, 20000.0, &[]).is_none(), "max_rf at the target depth");
    // 530 m: a reach of 530^2 / 450 = 624 m clears the 1.15 gain floor (610 m).
    assert!(plan_river_stretch(&s, MARKET, 530.0, 1500.0, &[]).is_some(), "a town of the smallest class");
    // 510 m: 578 m does not (586 m).
    assert!(plan_river_stretch(&s, MARKET, 510.0, 20000.0, &[]).is_none(), "under the gain floor");
    let mut dry = build_site(7, BOX, BOX, "landlocked", SiteOpts::default());
    dry.river.clear();
    assert!(plan_river_stretch(&dry, MARKET, 720.0, 20000.0, &[]).is_none(), "no centreline");
}

/// Protects the gain floor: a town whose walk reaches only about half
/// `max_rf` each way (neighbours 800 m up and down with no reach: the
/// half-way lines at 400 m) is not stretched.
#[test]
fn a_stretch_that_gains_too_little_is_dropped() {
    let s = site(&|_, _| false);
    let n = [StretchNeighbour { at: Vec2::new(2200.0, RIVER_Y), reach_m: 0.0 }, StretchNeighbour { at: Vec2::new(3800.0, RIVER_Y), reach_m: 0.0 }];
    assert!(plan_river_stretch(&s, MARKET, 720.0, 20000.0, &n).is_none());
}

/// Protects the frame's distance: `max_rf` at each reach and at the depth,
/// 0 at the market's foot, and the two sides scaled by their own reach.
#[test]
fn the_frame_reads_along_and_across_the_river_at_their_own_scales() {
    let river: Vec<Vec2> = (0..=300).map(|k| Vec2::new(20.0 * k as f64, RIVER_Y)).collect();
    let f = StretchFrame::new(&river, MARKET, StretchPlan { ahead_m: 1440.0, behind_m: 720.0, across_m: 360.0 }, 720.0).expect("frame");
    let d = |x: f64, y: f64| f.dist(Vec2::new(x, y));
    assert!(d(3000.0, RIVER_Y).abs() < 1e-9);
    assert!((d(4440.0, RIVER_Y) - 720.0).abs() < 1e-9, "ahead reach {}", d(4440.0, RIVER_Y));
    assert!((d(2280.0, RIVER_Y) - 720.0).abs() < 1e-9, "behind reach {}", d(2280.0, RIVER_Y));
    assert!((d(3000.0, RIVER_Y + 360.0) - 720.0).abs() < 1e-9, "depth {}", d(3000.0, RIVER_Y + 360.0));
    assert!((d(3720.0, RIVER_Y - 180.0) - 720.0 * 0.5f64.hypot(0.5)).abs() < 1e-9);
}

/// Protects the layout box: the stretched region's extent from the market
/// plus the 150 m margin, never under the reference's half box.
#[test]
fn the_box_holds_the_stretched_region_and_never_shrinks() {
    let river: Vec<Vec2> = (0..=300).map(|k| Vec2::new(20.0 * k as f64, RIVER_Y)).collect();
    let (hx, hy) = stretch_half_box(&river, MARKET, StretchPlan { ahead_m: 1440.0, behind_m: 600.0, across_m: 360.0 });
    assert_eq!((hx, hy), (1590.0, 625.0));
    let (hx, hy) = stretch_half_box(&river, MARKET, StretchPlan { ahead_m: 300.0, behind_m: 300.0, across_m: 500.0 });
    assert_eq!((hx, hy), (850.0, 650.0));
}

/// The x and y spread (p10 to p90) of a layout's building centroids.
fn spread(t: &crate::generate::Town) -> (f64, f64) {
    let mut xs: Vec<f64> = t.buildings.iter().map(|b| crate::geom::poly_centroid(&b.poly).x).collect();
    let mut ys: Vec<f64> = t.buildings.iter().map(|b| (crate::geom::poly_centroid(&b.poly).y - RIVER_Y).abs()).collect();
    xs.sort_by(f64::total_cmp);
    ys.sort_by(f64::total_cmp);
    let q = |v: &[f64], f: f64| v[((v.len() - 1) as f64 * f) as usize];
    (q(&xs, 0.9) - q(&xs, 0.1), q(&ys, 0.9))
}

/// Protects the growth's read of the frame (`GrowOpts::stretch`, set in
/// `generate_pass` from `GenOpts::river_stretch`): the same town laid out in
/// the same box with the plan reaches farther along its river and less far
/// inland than without it, and houses about as many people.
#[test]
fn a_stretched_town_grows_along_its_river_not_inland() {
    let w = water(&|_, _| false);
    let base = GenOpts {
        site: Some("river".into()),
        pop: Some(12000.0),
        water: Some(w),
        terrain: Some(flat()),
        riverbank: true,
        site_box: Some((BOX, BOX)),
        ..GenOpts::default()
    };
    let plain = generate(11, &base);
    // The plan the ruling's own sizing gives this town (`max_rf` 720 m).
    let plan = plan_river_stretch(&site(&|_, _| false), MARKET, 720.0, 12000.0, &[]).expect("premise: the fixture town stretches");
    assert_eq!(plan, StretchPlan { ahead_m: 1080.0, behind_m: 1080.0, across_m: 480.0 }, "premise");
    let long = generate(11, &GenOpts { river_stretch: Some(plan), ..base.clone() });
    let ((px, py), (lx, ly)) = (spread(&plain), spread(&long));
    eprintln!("plain: bld {} pop {} spread {px:.0} {py:.0}; long: bld {} pop {} spread {lx:.0} {ly:.0}", plain.buildings.len(), plain.pop, long.buildings.len(), long.pop);
    assert!(lx > px * 1.4, "along: plain {px:.0} m, stretched {lx:.0} m");
    assert!(ly < py * 0.8, "inland: plain {py:.0} m, stretched {ly:.0} m");
    assert!(long.pop > plain.pop * 0.85, "housed: plain {}, stretched {}", plain.pop, long.pop);
    // The districts read the same frame (`Site::market_dist`): no lot the
    // frame puts inside the 430 m core ring is zoned agrarian.
    let f = StretchFrame::new(&long.site.river, long.anchors.market, plan, 720.0).expect("frame");
    let near_agrarian = long.parcels.iter().filter(|p| p.district == "agrarian" && f.dist(crate::geom::poly_centroid(&p.par.poly)) <= 430.0).count();
    assert_eq!(near_agrarian, 0, "agrarian lots inside the stretched core");
    assert!(long.parcels.iter().any(|p| p.district == "agrarian"), "premise: the town has agrarian lots");
}

