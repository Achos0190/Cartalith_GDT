//! `wallside`'s own checks. The before/after measurement across real towns is
//! `generate`'s concern and lives beside its golden; these pin the helpers.

use super::*;

#[test]
fn face_offset_is_half_the_drawn_stroke_and_refuses_the_rest() {
    // Literals, not the function against itself: `urban_layout_draw.gd`'s
    // `WALL_W` is curtain 4.5 and palisade 2.2.
    assert_eq!(face_offset("curtain"), Some(2.25));
    assert_eq!(face_offset("palisade"), Some(1.1));
    assert_eq!(face_offset("ditch"), None);
    assert_eq!(face_offset("bastioned"), None);
}

#[test]
fn arc_walks_open_and_closed() {
    let sq = [Vec2::new(0.0, 0.0), Vec2::new(10.0, 0.0), Vec2::new(10.0, 10.0), Vec2::new(0.0, 10.0)];
    let open = Arc::new(&sq, false);
    let closed = Arc::new(&sq, true);
    assert_eq!(open.len(), 30.0);
    assert_eq!(closed.len(), 40.0);
    assert_eq!(open.at(15.0), Vec2::new(10.0, 5.0));
    assert_eq!(open.at(99.0), Vec2::new(0.0, 10.0), "an open arc clamps");
    assert_eq!(closed.at(45.0), Vec2::new(5.0, 0.0), "a closed arc wraps");
    assert_eq!(closed.at(-5.0), Vec2::new(0.0, 5.0), "and wraps backwards");
    assert_eq!(closed.project(Vec2::new(12.0, 4.0)), 14.0);
}

#[test]
fn nothing_is_platted_against_an_unwalled_or_unbuildable_circuit() {
    let site = crate::site::build_site(1, 1700.0, 1250.0, "inland", Default::default());
    let g = Graph::new();
    assert!(build_wall_lots(1, &g, &WallState::default(), &site, &[], &[], 5000.0, 8).is_empty());
    let ring = vec![Vec2::new(500.0, 500.0), Vec2::new(900.0, 500.0), Vec2::new(900.0, 800.0), Vec2::new(500.0, 800.0)];
    for style in ["ditch", "bastioned"] {
        let w = WallState {
            ring: Some(ring.clone()),
            land_arc: Some(ring.clone()),
            style: style.into(),
            ..Default::default()
        };
        assert!(build_wall_lots(1, &g, &w, &site, &[], &[], 5000.0, 8).is_empty(), "{style}");
    }
}

#[test]
fn a_row_ends_raggedly_about_its_nominal_length() {
    // Literals, not the constants against themselves: onset 0.85, span 0.7.
    assert_eq!(taper_end_chance(0.0), 0.0);
    assert_eq!(taper_end_chance(0.85), 0.0, "never ends before 85% of nominal");
    assert!((taper_end_chance(1.2) - 0.5).abs() < 1e-12, "even odds at 120% of nominal");
    assert_eq!(taper_end_chance(1.55), 1.0, "always ended by 155%");
    assert_eq!(taper_end_chance(9.0), 1.0);
    // Monotone: a row that has survived further is never likelier to survive.
    let xs: Vec<f64> = (0..=30).map(|i| i as f64 * 0.05).collect();
    assert!(xs.windows(2).all(|w| taper_end_chance(w[0]) <= taper_end_chance(w[1])));
    // And outward-only back-line jitter, half the widest 3.5 m lane.
    assert_eq!(BACK_JITTER, 1.75);
}

#[test]
fn gate_quality_falls_off_the_gate_with_bounded_noise() {
    // Beside the gate, halfway, at the run's end and past it (a ragged row).
    assert_eq!(gate_quality(0.0, 80.0, 0.0), 1.0);
    assert_eq!(gate_quality(40.0, 80.0, 0.0), 0.5);
    assert_eq!(gate_quality(80.0, 80.0, 0.0), 0.0);
    assert_eq!(gate_quality(120.0, 80.0, 0.0), 0.0, "clamped, never negative");
    assert_eq!(gate_quality(0.0, 80.0, 0.15), 1.0, "clamped, never above 1");
    assert!((gate_quality(40.0, 80.0, -0.15) - 0.35).abs() < 1e-12);
    // "Not 100% a rule": the noise can invert two lots 0.3 of a run apart and
    // no further. Literal 0.15 = QUALITY_NOISE.
    assert_eq!(QUALITY_NOISE, 0.15);
    assert!(gate_quality(20.0, 80.0, -0.15) < gate_quality(40.0, 80.0, 0.15));
    assert!(gate_quality(10.0, 80.0, -0.15) > gate_quality(40.0, 80.0, 0.15));
}

// ------------------------------------------- mutation-sweep fixtures (2026-09-24) --
//
// Written for the survivors of the 2026-09-24 sweep; the table is in
// `URBAN_MORPHOLOGY_SCOPE.md` under "Beyond the reference: owner-ruled
// additions". Hand-built walls and streets; every tie is made exact, and
// asserted exact, before it is used.

use crate::growth::Gate;
use crate::rng::stream;

fn landlocked() -> Site {
    crate::site::build_site(1, 1700.0, 1250.0, "landlocked", Default::default())
}

fn ctx<'a>(g: &'a Graph, site: &'a Site, ring: &'a [Vec2], wall: &'a WallState) -> Ctx<'a> {
    Ctx { g, site, ring, wall, plazas: Vec::new(), taken: Vec::new() }
}

#[test]
fn arc_at_a_vertex_is_the_vertex_itself() {
    // `partition_point(c <= s)`: arc length exactly at a vertex resolves to the
    // segment that starts there (t = 0, the vertex), not to the one that ends
    // there (t = 1, which `a + (b - a) * 1` misses by an ulp here).
    let arc = Arc::new(&[Vec2::new(1.1, 0.0), Vec2::new(0.1, 0.0), Vec2::new(0.1, 5.0)], false);
    assert_ne!(1.1 + (0.1 - 1.1) * 1.0, 0.1, "the fixture's lerp really misses");
    assert_eq!(arc.at(arc.cum[1]), Vec2::new(0.1, 0.0));
}

#[test]
fn arc_at_a_zero_length_last_segment_is_its_start() {
    let arc = Arc::new(&[Vec2::new(0.0, 0.0), Vec2::new(10.0, 0.0), Vec2::new(10.0, 0.0)], false);
    assert_eq!(arc.at(99.0), Vec2::new(10.0, 0.0), "not 0/0");
}

#[test]
fn bow_walks_a_closed_arc_one_lap_either_side() {
    let sq = [Vec2::new(0.0, 0.0), Vec2::new(10.0, 0.0), Vec2::new(10.0, 10.0), Vec2::new(0.0, 10.0)];
    let closed = Arc::new(&sq, true);
    let half_diag = 2.5 * 2f64.sqrt();
    // The (10, 0) corner at arc length 10, reached as 10 + 40 from a window past the seam.
    assert!((closed.bow(45.0, 55.0) - half_diag).abs() < 1e-9, "one lap forward");
    // The (0, 10) corner at arc length 30, reached as 30 - 40 from a negative window.
    assert!((closed.bow(-15.0, -5.0) - half_diag).abs() < 1e-9, "one lap back");
}

#[test]
fn project_keeps_the_first_of_two_equidistant_segments() {
    let arc = Arc::new(&[Vec2::new(0.0, 0.0), Vec2::new(10.0, 0.0), Vec2::new(10.0, 10.0)], false);
    assert_eq!(arc.project(Vec2::new(5.0, 5.0)), 5.0);
}

#[test]
fn street_face_keeps_the_first_of_two_streets_hit_at_the_same_point() {
    let mut g = Graph::new();
    g.add_street(30.0, 20.0, 50.0, 20.0, "street", 4.0, 1, "w");
    g.add_street(50.0, 20.0, 70.0, 20.0, "lane", 3.0, 1, "e");
    let site = landlocked();
    let wall = WallState::default();
    let c = ctx(&g, &site, &[], &wall);
    let (o, dir) = (Vec2::new(50.0, 0.0), Vec2::new(0.0, 1.0));
    let first = g.edges_near(o, o + dir * 52.0)[0];
    let (d, cls) = c.street_face(o, dir, 52.0).expect("a street");
    assert_eq!(cls, g.edges[first].cls, "the first edge found wins the tie");
    assert_eq!(d, 20.0 - g.edges[first].w / 2.0 - 1.4);
}

#[test]
fn crosses_street_ignores_a_dead_street() {
    let mut g = Graph::new();
    g.add_street(0.0, 50.0, 100.0, 50.0, "street", 4.0, 1, "s");
    let site = landlocked();
    let wall = WallState::default();
    assert!(ctx(&g, &site, &[], &wall).crosses_street(Vec2::new(50.0, 0.0), Vec2::new(50.0, 100.0)));
    g.edges[0].alive = false;
    assert!(!ctx(&g, &site, &[], &wall).crosses_street(Vec2::new(50.0, 0.0), Vec2::new(50.0, 100.0)));
}

fn q(x0: f64, y0: f64, x1: f64, y1: f64) -> Vec<Vec2> {
    vec![Vec2::new(x0, y0), Vec2::new(x1, y0), Vec2::new(x1, y1), Vec2::new(x0, y1)]
}

const BIG: [Vec2; 4] = [Vec2::new(-100.0, -100.0), Vec2::new(1800.0, -100.0), Vec2::new(1800.0, 1400.0), Vec2::new(-100.0, 1400.0)];

#[test]
fn accepts_bounds_every_test_it_makes() {
    let g = Graph::new();
    let site = landlocked();
    let wall = WallState::default();
    let c = ctx(&g, &site, &BIG, &wall);
    assert!(c.accepts(&q(100.0, 100.0, 110.0, 110.0), true), "the baseline lot");
    // Area 26..=2600, inclusive.
    assert!(c.accepts(&q(100.0, 100.0, 113.0, 102.0), true), "exactly 26 m²");
    assert!(!c.accepts(&q(100.0, 100.0, 123.0, 101.0), true), "23 m²");
    // A bowtie of real area is refused.
    let bow = vec![Vec2::new(100.0, 100.0), Vec2::new(160.0, 100.0), Vec2::new(120.0, 140.0), Vec2::new(140.0, 140.0)];
    assert!(poly_self_intersects(&bow) && poly_area(&bow).abs() >= 26.0);
    assert!(!c.accepts(&bow, true));
    // The box, on the two edges the goldens never reach.
    assert!(!c.accepts(&q(100.0, -1.0, 110.0, 10.0), true), "y < 0");
    assert!(!c.accepts(&q(100.0, 1245.0, 110.0, 1251.0), true), "y > hm");
    // The back line clears every gate by 14 m, exactly 14 included.
    let gated = WallState {
        gates: vec![Gate { pt: Vec2::new(105.0, 124.0), water: false, prov: String::new() }],
        ..WallState::default()
    };
    assert_eq!(crate::geom::dist_pt_seg(Vec2::new(105.0, 124.0), Vec2::new(100.0, 110.0), Vec2::new(110.0, 110.0)), 14.0);
    assert!(ctx(&g, &site, &BIG, &gated).accepts(&q(100.0, 100.0, 110.0, 110.0), true), "14 m from a gate");
    // Not on the plaza.
    let plaza = q(90.0, 90.0, 120.0, 120.0);
    let mut cp = ctx(&g, &site, &BIG, &wall);
    cp.plazas.push(&plaza);
    assert!(!cp.accepts(&q(100.0, 100.0, 110.0, 110.0), true));
    // Every edge, the closing one (3 -> 0) included, is tested against the streets.
    let mut sg = Graph::new();
    sg.add_street(95.0, 105.0, 107.0, 105.0, "street", 4.0, 1, "stub");
    assert!(!ctx(&sg, &site, &BIG, &wall).accepts(&q(100.0, 100.0, 110.0, 110.0), true));
}

#[test]
fn accepts_measures_the_water_margin_on_every_corner_and_the_centroid() {
    let g = Graph::new();
    let wall = WallState::default();
    let one = |d: f64, wet: bool| crate::site::WaterCtx {
        mask: vec![u8::from(wet)],
        dt: vec![d],
        mw: 1,
        mh: 1,
        cell_m: 1.0,
        river_path: None,
        river_width_m: None,
        river_order: 0.0,
        sea_lake_cells: 0.0,
    };
    let site = |kind: &str, w: crate::site::WaterCtx| {
        let mut s = crate::site::build_site(1, 1700.0, 1250.0, kind, crate::site::SiteOpts { water: Some(w), ..Default::default() });
        s.river_w = 10.0;
        s
    };
    let lot = q(100.0, 100.0, 110.0, 110.0);
    // A channel: riverW/2 + 1 = 6, inclusive.
    for (d, ok) in [(6.0, true), (6.5, true), (5.5, false)] {
        let s = site("river", one(d, false));
        assert_eq!(ctx(&g, &s, &BIG, &wall).accepts(&lot, true), ok, "channel at {d} m");
    }
    // Anything else: 3 m.
    let s = site("coast", one(3.5, false));
    assert!(ctx(&g, &s, &BIG, &wall).accepts(&lot, true), "3.5 m from a coast");
    // The centroid alone in the water: three 60 m cells, the middle one wet.
    let mid = crate::site::WaterCtx { mask: vec![0, 1, 0], dt: vec![100.0; 3], mw: 3, cell_m: 60.0, ..one(100.0, false) };
    let s = site("coast", mid);
    let wide = q(40.0, 10.0, 160.0, 20.0);
    assert!(wide.iter().all(|p| !s.is_water(*p)) && s.is_water(crate::geom::poly_centroid(&wide)));
    assert!(!ctx(&g, &s, &BIG, &wall).accepts(&wide, true), "a wet centroid refuses");
}

#[test]
fn accepts_refuses_a_lot_touching_a_taken_one_where_the_half_open_test_says_inside() {
    // `point_in_poly` is half-open: a point on a polygon's min-x or min-y edge
    // is inside. A taken lot whose bounding box only touches the candidate's
    // is therefore still tested, not skipped.
    let g = Graph::new();
    let site = landlocked();
    let wall = WallState::default();
    let lot = q(100.0, 100.0, 110.0, 110.0);
    for taken in [q(110.0, 100.0, 120.0, 110.0), q(100.0, 110.0, 110.0, 120.0)] {
        let mut c = ctx(&g, &site, &BIG, &wall);
        c.taken.push((taken.clone(), bbox(&taken)));
        assert!(!c.accepts(&lot, true), "touching {taken:?}");
    }
}

#[test]
fn outward_probes_three_metres_off_the_chord() {
    // A chord 3.5 m outside a square: 3 m toward the square is still outside,
    // so the normal is kept as it is.
    let ring = q(0.0, 0.0, 100.0, 100.0);
    assert_eq!(outward(&ring, Vec2::new(20.0, -3.5), Vec2::new(60.0, -3.5)), Vec2::new(0.0, 1.0));
}

/// A straight 20 m land arc along the top of a 20 x 50 ring, and a landlocked
/// site. A faubourg lot off it is 20 x 8, back line at y = 197.75.
fn faub_fixture(bent: bool) -> (Vec<Vec2>, Vec<Vec2>) {
    let mid = if bent { vec![Vec2::new(110.0, 201.0)] } else { Vec::new() };
    let mut arc = vec![Vec2::new(100.0, 200.0)];
    arc.extend(mid.iter().copied());
    arc.push(Vec2::new(120.0, 200.0));
    let mut ring = arc.clone();
    ring.extend([Vec2::new(120.0, 250.0), Vec2::new(100.0, 250.0)]);
    (arc, ring)
}

fn faub(g: &Graph, bent: bool) -> Option<(Vec<Vec2>, &'static str)> {
    let site = landlocked();
    let wall = WallState::default();
    let (arc, ring) = faub_fixture(bent);
    let a = Arc::new(&arc, false);
    let c = ctx(g, &site, &ring, &wall);
    faubourg_lot(&c, &a, &ring, 2.25, 0.0, a.len(), 0.0, 8.0)
}

#[test]
fn a_faubourg_lot_allows_a_bow_of_exactly_one_metre() {
    let (arc, _) = faub_fixture(true);
    let a = Arc::new(&arc, false);
    assert_eq!(a.bow(0.0, a.len()), 1.0);
    assert!(faub(&Graph::new(), true).is_some());
}

#[test]
fn a_faubourg_lot_looks_eight_metres_past_its_depth_for_a_street() {
    let plain = faub(&Graph::new(), false).expect("a lot");
    assert_eq!(plain.0[0], Vec2::new(100.0, 189.75));
    assert_eq!(plain.1, "");
    // A 16 m-wide street whose centreline is 16.5 m out from fa: past depth + 8,
    // so it is not seen, although its building line (7.1 m) would cut the lot.
    let mut g = Graph::new();
    g.add_street(90.0, 181.25, 105.0, 181.25, "primary", 16.0, 1, "wide");
    assert_eq!(faub(&g, false).expect("a lot").0[0], Vec2::new(100.0, 189.75));
}

#[test]
fn a_faubourg_lot_takes_a_streets_class_only_when_the_street_cuts_it() {
    // A street whose building line lies exactly at the lot's depth: 12 m out,
    // 5.2 m wide, so 12 - 2.6 - 1.4 = 8.0. Not a cut, so no class.
    assert_eq!(12.0 - 5.2 / 2.0 - 1.4, 8.0);
    for (x0, x1) in [(90.0, 105.0), (115.0, 130.0)] {
        let mut g = Graph::new();
        g.add_street(x0, 185.75, x1, 185.75, "lane", 5.2, 1, "tie");
        assert_eq!(faub(&g, false).expect("a lot").1, "", "a tie on the {x0} side");
    }
    // A street only in front of fb, 10 m out: it cuts fb's side to 6.6 m, and
    // the lot takes its class from it.
    let mut g = Graph::new();
    g.add_street(115.0, 187.75, 130.0, 187.75, "lane", 4.0, 1, "fb only");
    let (quad, cls) = faub(&g, false).expect("a lot");
    assert_eq!(cls, "lane");
    assert!((quad[1].y - 191.15).abs() < 1e-9 && quad[1].x == 120.0, "{:?}", quad[1]);
}

/// A 300 m-deep rectangle whose top edge from x = 500 to `500 + len` is the
/// whole land arc (open), and a curtain on it.
fn top_wall(len: f64) -> WallState {
    let ring = vec![
        Vec2::new(400.0, 500.0),
        Vec2::new(600.0 + len, 500.0),
        Vec2::new(600.0 + len, 800.0),
        Vec2::new(400.0, 800.0),
    ];
    WallState {
        ring: Some(ring),
        land_arc: Some(vec![Vec2::new(500.0, 500.0), Vec2::new(500.0 + len, 500.0)]),
        style: "curtain".into(),
        ..WallState::default()
    }
}

fn inside(out: &[Parcel]) -> Vec<&Parcel> {
    out.iter().filter(|p| p.wall_backing == WallBacking::Inside).collect()
}

#[test]
fn a_two_point_land_arc_is_platted_and_every_wall_lot_is_as_old_as_the_town() {
    let wall = top_wall(200.0);
    let mut g = Graph::new();
    g.add_street(450.0, 528.25, 750.0, 528.25, "street", 4.0, 1, "inner");
    let out = build_wall_lots(3, &g, &wall, &landlocked(), &[], &[], 2000.0, 8);
    assert!(!inside(&out).is_empty(), "a two-point arc is a wall");
    assert!(out.iter().all(|p| p.age == 8.0));
}

#[test]
fn the_last_intramural_lot_needs_four_metres_of_arc_and_gets_it() {
    // Replay the intramural widths, and make the arc end 4.5 m past the third
    // lot: the loop runs a fourth time (4.5 m remain, more than 4) and that
    // lot is 4.5 m wide, over the 4 m chord minimum.
    let seed = 3;
    let mut r = stream(seed, "wallside/in");
    let s3: f64 = (0..3).map(|_| r.logn(9.5, 0.22).clamp(5.0, 15.0)).sum();
    let wall = top_wall(s3 + 4.5);
    let mut g = Graph::new();
    g.add_street(450.0, 528.25, 590.0 + s3, 528.25, "street", 4.0, 1, "inner");
    let out = build_wall_lots(seed, &g, &wall, &landlocked(), &[], &[], 2000.0, 8);
    let lots = inside(&out);
    assert_eq!(lots.len(), 4);
    assert!((lots[3].frontage - 4.5).abs() < 1e-9, "the last lot is the 4.5 m remainder");
}

#[test]
fn an_intramural_lot_exactly_six_metres_deep_is_kept() {
    // A street 13 m in from the wall's inner face (t = 13/52 = 0.25 exactly),
    // 11.2 m wide: 13 - 5.6 - 1.4 = 6.0, the minimum depth, exactly.
    assert_eq!(13.0 - 11.2 / 2.0 - 1.4, 6.0);
    let wall = top_wall(120.0);
    let mut g = Graph::new();
    g.add_street(450.0, 515.25, 700.0, 515.25, "street", 11.2, 1, "inner");
    let out = build_wall_lots(3, &g, &wall, &landlocked(), &[], &[], 2000.0, 8);
    let lots = inside(&out);
    assert!(!lots.is_empty(), "six metres deep is deep enough");
    assert!(lots.iter().all(|p| (p.depth - 6.0).abs() < 1e-9));
}

#[test]
fn an_intramural_wedge_of_exactly_twelve_metres_is_kept() {
    // The first lot's two rays meet different streets: 26 m in and 1 m wide
    // (24.1) under fa, 39 m in and 3 m wide (36.1) under fb — exactly 12 apart.
    assert_eq!((26.0 - 1.0 / 2.0 - 1.4) - (39.0 - 3.0 / 2.0 - 1.4), -12.0);
    let seed = 3;
    let w1 = stream(seed, "wallside/in").logn(9.5, 0.22).clamp(5.0, 15.0);
    let wall = top_wall(w1 + 1.0);
    let mut g = Graph::new();
    g.add_street(489.0, 528.25, 500.5, 528.25, "street", 1.0, 1, "under fa");
    g.add_street(499.0 + w1, 541.25, 511.0 + w1, 541.25, "street", 3.0, 1, "under fb");
    let out = build_wall_lots(seed, &g, &wall, &landlocked(), &[], &[], 2000.0, 8);
    let lots = inside(&out);
    assert_eq!(lots.len(), 1, "the wedge is platted");
    assert!((lots[0].poly[0].y - (502.25 + 24.1)).abs() < 1e-9 && (lots[0].poly[1].y - (502.25 + 36.1)).abs() < 1e-9);
}

/// A 400 m square curtain, closed (the land arc is the whole ring), with land
/// gates at the given points.
fn square_wall(gates: &[Vec2]) -> WallState {
    let ring = q(600.0, 300.0, 1000.0, 700.0);
    WallState {
        ring: Some(ring.clone()),
        land_arc: Some(ring),
        gates: gates.iter().map(|&pt| Gate { pt, water: false, prov: String::new() }).collect(),
        style: "curtain".into(),
        ..WallState::default()
    }
}

/// Every faubourg lot on the square's west side, checked against the owner's
/// gradient computed from its own geometry: arc distance from the lot's
/// midpoint to the nearest land gate, round the closed circuit, less the
/// 14 m gate passage, over the run length, plus that lot's own noise draw.
fn check_gradient(gates: &[Vec2], gate_s: &[f64]) {
    let seed = (1..500u32)
        .find(|&s| {
            let mut r = stream(s, "faubourg");
            !r.chance(0.5) && r.pick_index(gates.len()) == Some(0)
        })
        .expect("a seed whose one run heads backward from gate 0");
    let mut r = stream(seed, "faubourg");
    r.chance(0.5);
    r.pick_index(gates.len());
    let run_len = 1600.0 * r.range(0.10, 0.20);
    let wall = square_wall(gates);
    let out = build_wall_lots(seed, &Graph::new(), &wall, &landlocked(), &[], &[], 2000.0, 8);
    let mut noise = stream(seed, "wallside/quality");
    let mut checked = 0;
    let mut low = 0;
    for p in out.iter().filter(|p| p.wall_backing.is_faubourg()) {
        let n = noise.range(-0.15, 0.15);
        let (fa, fb) = (p.poly[3], p.poly[2]);
        if !(fa.x < 600.0 && fb.x < 600.0 && fa.y > 300.0 && fb.y > 300.0) {
            continue;
        }
        let s = 1200.0 + (700.0 - (fa.y + fb.y) / 2.0);
        let d = gate_s
            .iter()
            .map(|&g| {
                let d = (s - g).abs();
                d.min(1600.0 - d)
            })
            .fold(f64::INFINITY, f64::min);
        let want = (1.0 - (d - 14.0).max(0.0) / run_len + n).clamp(0.0, 1.0);
        let got = p.gate_quality.expect("a faubourg lot has a quality");
        assert!((got - want).abs() < 1e-9, "{}: {got} vs {want} at s = {s}", p.id);
        checked += 1;
        low += usize::from(got < 0.5);
    }
    assert!(checked >= 10, "only {checked} lots on the west side");
    assert!(low >= 1, "the run reaches the poor end of the gradient");
}

#[test]
fn gate_quality_is_measured_round_the_closed_circuit_to_the_nearest_gate() {
    // One gate 5 m along the first side: the run heads backward across the
    // seam, so every lot is `total - s` from it the short way round.
    check_gradient(&[Vec2::new(605.0, 300.0)], &[5.0]);
    // A second gate 3 m before the seam: the lots are nearer it, and only a
    // wrapped arc length knows that.
    check_gradient(&[Vec2::new(605.0, 300.0), Vec2::new(600.0, 303.0)], &[5.0, 1597.0]);
}

#[test]
fn a_faubourg_run_stops_at_the_end_of_an_open_arc() {
    // The end-of-arc break is backstopped by `Arc::bow`, which walks even an
    // open arc one lap either side and so finds vertex 0 at arc length
    // `total` — refusing any lot that crosses the end, unless vertex 0 lies
    // within 1 m of that lot's chord. So: an open arc round three sides and
    // back to 1 m short of its start, and a gate 34 m before its end.
    let arc = vec![
        Vec2::new(600.0, 300.0),
        Vec2::new(1000.0, 300.0),
        Vec2::new(1000.0, 700.0),
        Vec2::new(600.0, 700.0),
        Vec2::new(600.0, 301.0),
    ];
    let total = Arc::new(&arc, false).len();
    assert_eq!(total, 1599.0);
    let seed = (1..5000u32)
        .find(|&s| {
            let mut r = stream(s, "faubourg");
            r.chance(0.5)
        })
        .expect("a seed whose run heads toward the end");
    let wall = WallState {
        ring: Some(q(600.0, 300.0, 1000.0, 700.0)),
        land_arc: Some(arc),
        gates: vec![Gate { pt: Vec2::new(600.0, 335.0), water: false, prov: String::new() }],
        style: "curtain".into(),
        ..WallState::default()
    };
    let out = build_wall_lots(seed, &Graph::new(), &wall, &landlocked(), &[], &[], 2000.0, 8);
    let faub: Vec<&Parcel> = out.iter().filter(|p| p.wall_backing.is_faubourg()).collect();
    assert!(!faub.is_empty(), "the run plats lots before it reaches the end");
    for p in &faub {
        assert!(p.poly.iter().all(|c| c.y > 301.0 + 1e-9), "{} reaches past the end of the arc: {:?}", p.id, p.poly);
    }
}

#[test]
fn a_city_has_three_faubourg_runs() {
    // No gates: each run starts at a drawn point. A city (pop >= 10 000) has
    // three runs; replaying the draws finds each run's start, and each has a
    // first-row lot within a lot's width of it.
    let wall = square_wall(&[]);
    let out = build_wall_lots(11, &Graph::new(), &wall, &landlocked(), &[], &[], 12000.0, 8);
    let first: Vec<&Parcel> = out.iter().filter(|p| p.wall_backing == WallBacking::Outside).collect();
    assert!(!first.is_empty());
    let arc = Arc::new(&q(600.0, 300.0, 1000.0, 700.0), true);
    let mut r = stream(11, "faubourg");
    let mut starts = Vec::new();
    for _ in 0..3 {
        let dir = if r.chance(0.5) { 1.0 } else { -1.0 };
        assert_eq!(r.pick_index(0), None, "no gate: the pick draws and finds nothing");
        let start: f64 = r.range(0.0, 1600.0);
        let run_len = 1600.0 * r.range(0.10, 0.20);
        let mut t = 0.0;
        while t < run_len {
            let w = r.logn(6.5, 0.18).clamp(4.5, 9.0);
            r.logn(9.0, 0.2);
            t += w;
        }
        starts.push((start, dir, run_len));
    }
    // Every first-row lot lies in one of the three replayed runs, and each run
    // holds at least one — a fourth run would put lots outside all three.
    let mut hit = [0usize; 3];
    for p in &first {
        let s = arc.project(p.poly[3].lerp(p.poly[2], 0.5));
        let k = starts.iter().position(|&(st, dir, len)| {
            let rel = ((s - st) * dir).rem_euclid(1600.0);
            rel <= len + 9.0 || rel >= 1600.0 - 9.0
        });
        hit[k.unwrap_or_else(|| panic!("{} is outside every run", p.id))] += 1;
    }
    assert!(hit.iter().all(|&n| n > 0), "each run has a first row: {hit:?}");
}
