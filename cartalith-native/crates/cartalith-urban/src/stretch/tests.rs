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
    // Ruling FQ's measures read the layout itself (`StretchMeasures::of`):
    // the housed share, and the wall's three answers only for a walled town.
    let m = StretchMeasures::of(&long);
    assert_eq!(m.housed, long.pop / long.pop_target);
    assert_eq!(m.wall.is_some(), long.wall.ring.is_some(), "{m:?}");
    if let Some((wh, out, _)) = m.wall {
        assert!((1.0..1.5).contains(&wh), "a wall at least as long as its hull: {wh}");
        assert!(!out, "the fixture town's square is inside its wall");
    }
    // Hand-set walls, so each measure is checked against a literal: an L of
    // 200 x 200 m less a 100 x 100 m corner is 800 m round against a hull of
    // 200 + 100 + 141.42 + 100 + 200 = 741.42 m, 1.0790; the market (no
    // square) inside it, 3 m outside it (within the census's street width)
    // and 10 m outside it.
    let mut walled = long.clone();
    let o = Vec2::new(2900.0, RIVER_Y + 20.0);
    let l = [(0.0, 0.0), (200.0, 0.0), (200.0, 100.0), (100.0, 100.0), (100.0, 200.0), (0.0, 200.0)];
    walled.wall.ring = Some(l.iter().map(|&(x, y)| o + Vec2::new(x, y)).collect());
    walled.plaza = None;
    let at = |t: &mut crate::generate::Town, x: f64, y: f64| {
        t.anchors.market = o + Vec2::new(x, y);
        StretchMeasures::of(t).wall.expect("walled")
    };
    let (wh, inside, _) = at(&mut walled, 50.0, 50.0);
    assert!((wh - 800.0 / (500.0 + 2.0f64.sqrt() * 100.0 + 100.0)).abs() < 1e-9, "wall / hull {wh}");
    assert!((wh - 1.0790).abs() < 1e-4, "wall / hull {wh}");
    assert!(!inside, "the market inside the wall");
    assert!(!at(&mut walled, -3.0, 50.0).1, "3 m outside: within the street width");
    assert!(at(&mut walled, -10.0, 50.0).1, "10 m outside: outside");
    assert!(at(&mut walled, 150.0, 150.0).1, "in the L's notch: outside");
}


/// Protects Ruling FQ's milder retry (`StretchPlan::milder`): each side's
/// reach beyond `max_rf` halved (720 + 360 / 2 = 900 m), the depth widened to
/// keep the area (720^2 / 900 = 576 m), a side already short kept but never
/// shorter than the new depth, and a plan whose milder form gains under
/// `STRETCH_MIN_GAIN` dropped.
#[test]
fn the_milder_plan_halves_the_extra_reach_and_keeps_the_area() {
    let p = StretchPlan { ahead_m: 1080.0, behind_m: 1080.0, across_m: 480.0 };
    assert_eq!(p.milder(720.0), Some(StretchPlan { ahead_m: 900.0, behind_m: 900.0, across_m: 576.0 }));
    // One side cut at 600 m by a neighbour: (900 + 600) / 2 = 750 m, under
    // 1.15 x 720 = 828 m, so there is no milder stretch.
    assert_eq!(StretchPlan { ahead_m: 1080.0, behind_m: 600.0, across_m: 548.6 }.milder(720.0), None);
    // A long one-sided stretch: (1 260 + 520) / 2 = 890 m; depth 720^2 / 890
    // = 582.5 m, and the short side is raised to it.
    let m = StretchPlan { ahead_m: 1800.0, behind_m: 520.0, across_m: 446.9 }.milder(720.0).expect("gains enough");
    assert_eq!((m.ahead_m, m.behind_m), (1260.0, 720.0 * 720.0 / 890.0));
    assert!((m.across_m - 582.47).abs() < 0.01, "{m:?}");
}

/// Measures of a layout with the given housed share and wall answer.
fn measures(housed: f64, wall: Option<(f64, bool, f64)>) -> StretchMeasures {
    StretchMeasures { housed, wall }
}

/// Protects Ruling FQ's gate (`stretch_holds`), each bar on its own: inside
/// the bound alone it holds; past a bound it fails without the unstretched
/// layout, and holds against one no better on that bar (and only that bar).
#[test]
fn the_gate_holds_each_bar_by_its_bound_or_by_the_plain_layout() {
    let good = (1.05, false, 1_000.0);
    assert!(stretch_holds(&measures(0.95, Some(good)), None), "inside every bound");
    assert!(stretch_holds(&measures(0.90, None), None), "unwalled at the housed bound");
    // Housed.
    let short = measures(0.80, Some(good));
    assert!(!stretch_holds(&short, None), "housed under 90 %");
    assert!(stretch_holds(&short, Some(&measures(0.78, Some(good)))), "no less than plain");
    assert!(!stretch_holds(&short, Some(&measures(0.85, Some(good)))), "less than plain");
    // Wall over hull.
    let bent = measures(1.0, Some((1.13, false, 0.0)));
    assert!(!stretch_holds(&bent, None), "wall / hull over 1.12");
    assert!(stretch_holds(&bent, Some(&measures(1.0, Some((1.14, false, 0.0))))), "no longer than plain's");
    assert!(!stretch_holds(&bent, Some(&measures(1.0, Some((1.10, false, 0.0))))), "longer than plain's");
    assert!(!stretch_holds(&bent, Some(&measures(1.0, None))), "plain unwalled: the bound alone");
    // The square.
    let out = measures(1.0, Some((1.0, true, 0.0)));
    assert!(!stretch_holds(&out, None), "square outside");
    assert!(stretch_holds(&out, Some(&measures(1.0, Some((1.0, true, 0.0))))), "plain's outside too");
    assert!(!stretch_holds(&out, Some(&measures(1.0, Some((1.3, false, 9e9))))), "plain's inside");
    // The unbuilt patch.
    let open = measures(1.0, Some((1.0, false, 5_001.0)));
    assert!(!stretch_holds(&open, None), "patch over half a hectare");
    assert!(stretch_holds(&measures(1.0, Some((1.0, false, 5_000.0))), None), "at the bound");
    assert!(stretch_holds(&open, Some(&measures(1.0, Some((1.0, false, 6_000.0))))), "no larger than plain's");
    assert!(!stretch_holds(&open, Some(&measures(1.0, Some((1.0, false, 4_000.0))))), "larger than plain's");
}

/// A square building `side` metres across centred on `c`.
fn house(c: Vec2, side: f64) -> crate::districts::Building {
    let h = side / 2.0;
    crate::districts::Building {
        id: "bld0".into(),
        poly: vec![c + Vec2::new(-h, -h), c + Vec2::new(h, -h), c + Vec2::new(h, h), c + Vec2::new(-h, h)],
        ridge: [c, c],
        parcel: "p0".into(),
        kind: "house",
        district: "core",
        age: 0.0,
        courtyard: false,
        prov: "",
    }
}

/// Protects the unbuilt-patch measure (`unbuilt_patch_m2`): an empty 200 m
/// square wall encloses its whole 4 ha; a house at its centre leaves a ring
/// of open ground (smaller, and still one patch); a grid of houses every 50 m
/// leaves no cell 40 m from them all; a river through the middle splits the
/// open ground and is not counted; neither a plaza nor a water body is; and
/// a house's whole footprint counts as built, not only its corners.
#[test]
fn the_unbuilt_patch_is_ground_far_from_every_house_inside_the_wall() {
    let ring = vec![Vec2::new(0.0, 0.0), Vec2::new(200.0, 0.0), Vec2::new(200.0, 200.0), Vec2::new(0.0, 200.0)];
    let empty = unbuilt_patch_m2(&ring, &[], &[], 0.0, &[], &[]);
    assert_eq!(empty, 40_000.0, "the whole square, 50 x 50 cells of 16 m2");
    let one = unbuilt_patch_m2(&ring, &[house(Vec2::new(100.0, 100.0), 10.0)], &[], 0.0, &[], &[]);
    assert!(one < empty - 5_000.0 && one > 30_000.0, "a ring round the house: {one}");
    let mut row = Vec::new();
    for j in 0..6 {
        for i in 0..6 {
            row.push(house(Vec2::new(-25.0 + 50.0 * i as f64, -25.0 + 50.0 * j as f64), 8.0));
        }
    }
    assert_eq!(unbuilt_patch_m2(&ring, &row, &[], 0.0, &[], &[]), 0.0, "houses every 50 m");
    let river = vec![Vec2::new(-50.0, 100.0), Vec2::new(250.0, 100.0)];
    let split = unbuilt_patch_m2(&ring, &[], &river, 20.0, &[], &[]);
    assert!(split < 20_000.0 && split > 15_000.0, "one bank's half: {split}");
    let plaza = vec![Vec2::new(0.0, 0.0), Vec2::new(200.0, 0.0), Vec2::new(200.0, 100.0), Vec2::new(0.0, 100.0)];
    assert_eq!(unbuilt_patch_m2(&ring, &[], &[], 0.0, &[], &plaza), 20_000.0, "the square's half is not counted");
    assert_eq!(unbuilt_patch_m2(&ring, &[], &[], 0.0, &plaza, &[]), 20_000.0, "nor a water body's");
    // A house far wider than twice the reach: its own footprint is built, not
    // only its corners (its centre is 113 m from every corner).
    assert_eq!(unbuilt_patch_m2(&ring, &[house(Vec2::new(100.0, 100.0), 160.0)], &[], 0.0, &[], &[]), 0.0, "a 160 m house leaves no open ground");
}

/// The fixture's water context with the river along `path` instead of the
/// straight one, no open water.
fn water_along(path: Vec<Vec2>) -> WaterCtx {
    WaterCtx { river_path: Some(path), ..water(&|_, _| false) }
}

/// Protects the spine's stop at a tight bend (`lay_river_spine`): a market
/// inside a 30 degree V of the river. Walking toward the apex, the offset
/// point 52 m inside the near arm closes on the far arm, so the spine must
/// stop before it: it is laid (not dropped), every point stands on the
/// market's bank and dry, and it ends at least 100 m short of the apex.
/// Written (Ruling FQ, 2026-11-16) for the prototype's surviving mutant
/// "spine-bank" (the `site.bank_side(p) == side` guard, `496c7a2d`), and it
/// does NOT kill it: the walk stops one step earlier on `buildable`, whose
/// flood band (`river_w / 2 + 30` m round every segment) is wider than the
/// 20 m a 40 m step closes on the far arm, so the guard never acts here.
/// The guard stays as a belt-and-braces check; the survivor and its reason
/// are in `cartalith-native/docs/evidence/river-stretch-2026-11-16/`.
#[test]
fn the_spine_stops_where_a_tight_bend_would_carry_it_across_the_river() {
    let apex = Vec2::new(4000.0, 3000.0);
    let (a0, b1) = (Vec2::new(1000.0, 2200.0), Vec2::new(1000.0, 3800.0));
    let mut path: Vec<Vec2> = (0..=150).map(|k| a0.lerp(apex, k as f64 / 150.0)).collect();
    path.extend((1..=150).map(|k| apex.lerp(b1, k as f64 / 150.0)));
    let mut s = build_site(7, BOX, BOX, "river", SiteOpts { water: Some(water_along(path)), terrain: Some(flat()), economy: None });
    assert!(s.use_riverbank(), "premise: the riverbank geometry");
    // On the near arm's inner side, 60 m off it, 1.5 km from the apex.
    let market = Vec2::new(2500.0, 2200.0 + 800.0 * 0.5 + 60.0);
    let side = s.bank_side(market);
    let plan = StretchPlan { ahead_m: 2400.0, behind_m: 500.0, across_m: 450.0 };
    let f = StretchFrame::new(&s.river, market, plan, 720.0);
    s.set_stretch(f);
    let mut g = crate::graph::Graph::new();
    let ids = lay_river_spine(&s, &mut g, market);
    assert!(!ids.is_empty(), "the spine is laid up to the bend, not dropped");
    let pts: Vec<Vec2> = g.nodes.iter().map(|n| n.pt()).collect();
    assert!(pts.iter().all(|&p| s.bank_side(p) == side && !s.is_water(p)), "every spine point on the market's bank");
    let reach = pts.iter().map(|p| p.x).fold(f64::NEG_INFINITY, f64::max);
    assert!(reach > 3000.0 && reach < apex.x - 100.0, "it runs toward the apex and stops short of it: {reach:.0}");
}

/// Protects the interior-lane cut-off's read of the frame (`cleanup::lane_pass`
/// through `Site::market_dist`; the prototype's surviving mutant "lane-dist",
/// `496c7a2d`): a 2 ha face 700 m along the river from the market and 100 m
/// off it is 707 m from the market (past the 520 m cut-off) but 490 m in the
/// stretched frame (1 080 m along, 480 m across), so a stretched town lays a
/// lane through it and the same town unstretched does not.
#[test]
fn a_stretched_town_lays_interior_lanes_along_its_river_where_a_plain_one_does_not() {
    let face = |g: &mut crate::graph::Graph| {
        let (x0, x1, y0, y1) = (3600.0, 3800.0, RIVER_Y + 50.0, RIVER_Y + 150.0);
        g.add_street(x0, y0, x1, y0, "street", 5.0, 0, "test");
        g.add_street(x1, y0, x1, y1, "street", 5.0, 0, "test");
        g.add_street(x1, y1, x0, y1, "street", 5.0, 0, "test");
        g.add_street(x0, y1, x0, y0, "street", 5.0, 0, "test");
    };
    let anchors = crate::routes::Anchors { market: MARKET, prov: "test" };
    let mut s = site(&|_, _| false);
    let mut plain = crate::graph::Graph::new();
    face(&mut plain);
    assert_eq!(crate::cleanup::lane_pass(7, &s, &anchors, &mut plain, 1, None), 0, "unstretched: 707 m is past the cut-off");
    let plan = StretchPlan { ahead_m: 1080.0, behind_m: 1080.0, across_m: 480.0 };
    s.set_stretch(StretchFrame::new(&s.river.clone(), MARKET, plan, 720.0));
    assert!((s.market_dist(Vec2::new(3700.0, RIVER_Y + 100.0), MARKET) - 490.2).abs() < 0.5, "premise: 490 m in the frame");
    let mut long = crate::graph::Graph::new();
    face(&mut long);
    assert_eq!(crate::cleanup::lane_pass(7, &s, &anchors, &mut long, 1, None), 1, "stretched: one lane through the face");
}
