//! Deep burgage plots ([`crate::burgage`], owner Ruling FE item 3's
//! prototype, 2026-11-17). Each test says what it protects. The towns are the
//! generator's synthetic river site (no host water), so every figure comes
//! from this crate alone.

use crate::cleanup::LANE_PROV;
use crate::generate::{GenOpts, Town, generate, hash_model};

/// A town of `pop` on the synthetic river site, deep plots on or off.
fn town(seed: u32, pop: f64, burgage: bool) -> Town {
    generate(seed, &GenOpts { pop: Some(pop), burgage, ..GenOpts::default() })
}

/// The street-platted lots' depth / frontage ratios, sorted, and the share of
/// lots over 40 m deep. Asserts there are lots: an empty town must not pass
/// as a shallow one.
fn lots(t: &Town) -> (Vec<f64>, f64) {
    let mut r: Vec<f64> = t.parcels.iter().filter(|p| p.par.frontage > 0.0).map(|p| p.par.depth / p.par.frontage).collect();
    assert!(r.len() > 200, "premise: the town plats lots ({})", r.len());
    r.sort_by(|a, b| a.total_cmp(b));
    let deep = t.parcels.iter().filter(|p| p.par.depth > 40.0).count() as f64 / t.parcels.len() as f64;
    (r, deep)
}

/// Mean area of the town's non-plaza blocks, m2.
fn mean_block(t: &Town) -> f64 {
    let a: Vec<f64> = t.blocks.iter().filter(|b| !b.plaza).map(|b| crate::geom::poly_area(&b.poly).abs()).collect();
    assert!(!a.is_empty(), "premise: the town has blocks");
    a.iter().sum::<f64>() / a.len() as f64
}

/// Protects the prototype's whole effect on a town of 6 000: lots deeper for
/// their frontage (median depth / frontage and the share over 40 m both rise
/// past literals the reference stays under), on blocks at least half as large
/// again (streets further apart), with fewer interior lanes. MUTATION:
/// dropping the deeper platting, the wider street rules, the wide spacing
/// query or the lane threshold fails a bar.
#[test]
fn burgage_plats_deeper_lots_on_larger_blocks() {
    let (mut deep_off, mut deep_on) = (0.0, 0.0);
    let (mut lanes_off, mut lanes_on) = (0, 0);
    for seed in [4242u32, 77] {
        let (off, on) = (town(seed, 6000.0, false), town(seed, 6000.0, true));
        let ((r_off, d_off), (r_on, d_on)) = (lots(&off), lots(&on));
        let (m_off, m_on) = (r_off[r_off.len() / 2], r_on[r_on.len() / 2]);
        let lanes = |t: &Town| t.graph.edges.iter().filter(|e| e.alive && e.prov == LANE_PROV).count();
        // Measured 2026-11-17 with the back lanes and the whole budget
        // (Ruling FW item 4): median 1.34 -> 3.04 and 1.37 -> 2.88; blocks
        // 998 -> 2 857 and 1 180 -> 3 504 m2; interior lanes 6 -> 0 and
        // 5 -> 5 (the first prototype: 3.08 / 2.99, 2 939 / 2 527, 0 / 0).
        assert!(m_off < 1.6, "seed {seed}: the reference's lots are house-deep ({m_off:.2})");
        assert!(m_on > 2.5, "seed {seed}: deep plots ({m_on:.2})");
        assert!(mean_block(&on) > 1.8 * mean_block(&off), "seed {seed}: blocks {:.0} vs {:.0} m2", mean_block(&on), mean_block(&off));
        assert!(lanes(&off) > 0, "seed {seed}: premise, the reference cuts interior lanes");
        lanes_off += lanes(&off);
        lanes_on += lanes(&on);
        deep_off += d_off;
        deep_on += d_on;
    }
    // Measured: 0.005 + 0.003 -> 0.054 + 0.013 of lots over 40 m deep.
    assert!(deep_off < 0.02 && deep_on > 0.05, "lots over 40 m: {deep_off:.3} -> {deep_on:.3}");
    // The lane threshold (`LANE_MIN_AREA_M2`): 11 interior lanes -> 5.
    assert!(lanes_on * 2 < lanes_off, "interior lanes {lanes_off} -> {lanes_on}");
}

/// Protects the gate: a street village (its tofts are its own plan) and the
/// radial plan (which never calls `grow`) are byte-identical with the switch
/// on, and a town with it off is the reference (`hash_model`).
#[test]
fn burgage_leaves_street_villages_and_the_radial_plan_alone() {
    let sv = |b: bool| generate(4242, &GenOpts { pop: Some(800.0), street_village: true, burgage: b, ..GenOpts::default() });
    assert_eq!(hash_model(&sv(false)), hash_model(&sv(true)), "a street village is untouched");
    let radial = |b: bool| generate(4242, &GenOpts { pop: Some(6000.0), culture: Some("venus".into()), burgage: b, ..GenOpts::default() });
    assert_eq!(hash_model(&radial(false)), hash_model(&radial(true)), "the radial plan is untouched");
    assert_eq!(hash_model(&town(4242, 6000.0, false)), hash_model(&generate(4242, &GenOpts { pop: Some(6000.0), ..GenOpts::default() })), "off is the default");
    assert_ne!(hash_model(&town(4242, 6000.0, false)), hash_model(&town(4242, 6000.0, true)), "premise: on moves an organic town");
}

/// A street grid with no jitter: every `xs` x `ys` crossing joined to its
/// neighbours by a 6 m street.
fn grid(xs: &[f64], ys: &[f64]) -> crate::graph::Graph {
    let mut g = crate::graph::Graph::new();
    for &y in ys {
        for w in xs.windows(2) {
            g.add_street(w[0], y, w[1], y, "street", 6.0, 1, "fixture");
        }
    }
    for &x in xs {
        for w in ys.windows(2) {
            g.add_street(x, w[0], x, w[1], "street", 6.0, 1, "fixture");
        }
    }
    g
}

/// The deepest lot `build_parcels_with` plats on the blocks of `xs` x `ys`,
/// metres, with the market far away (so the far-from-market depth target
/// applies) and `rules` (`None`: the defaults).
fn deepest(xs: &[f64], ys: &[f64], burgage: bool, rules: Option<&crate::rules::Rules>) -> f64 {
    let g = grid(xs, ys);
    let site = crate::site::build_site(1, 1700.0, 1250.0, "plain", crate::site::SiteOpts::default());
    let blocks = crate::blocks::build_blocks(&g, None, &site);
    assert!(!blocks.is_empty(), "premise: the grid closes blocks");
    let lots = crate::blocks::build_parcels_with(1, &g, &blocks, crate::geom::Vec2::new(-5000.0, -5000.0), 8, &site, rules, burgage);
    assert!(!lots.is_empty(), "premise: the blocks are platted");
    lots.iter().map(|p| p.depth).fold(0.0, f64::max)
}

/// Protects the platting's two limits on blocks drawn for them: the waist
/// (lots reach [`crate::burgage::WAIST`] of a 70 m block's width, not the
/// reference's 0.42) and the depth cap (on 400 m blocks a widely varying
/// target reaches past the reference's 46 m). MUTATION: the reference's 0.42
/// waist or 46 m cap under the switch fails a bar.
#[test]
fn burgage_lots_reach_the_waist_and_past_the_reference_cap() {
    // Blocks 70 m between street centrelines, 300 m long: the waist binds
    // (both under the 42 m target), so the deepest lot (a corner lot, whose
    // bisector runs further than the edge normal) scales by 0.48 / 0.42.
    // Measured 2026-11-17: 35.2 -> 40.3 m.
    let (xs, ys) = ([300.0, 600.0, 900.0], [300.0, 370.0, 440.0]);
    let (off, on) = (deepest(&xs, &ys, false, None), deepest(&xs, &ys, true, None));
    assert!(off < 37.0 && on > 38.5, "waist: deepest lot {off:.1} -> {on:.1} m");
    // Fifteen blocks 250 m square and a target spread wide
    // (`plot_depth_variance` 0.6, the plot-chaos slider's top): some block
    // draws past 46 m.
    let mut wide = crate::rules::DEFAULT_RULES;
    wide.parcels.plot_depth_variance = 0.6;
    let (xs, ys) = ([100.0, 350.0, 600.0, 850.0, 1100.0, 1350.0], [100.0, 350.0, 600.0, 850.0]);
    let (off, on) = (deepest(&xs, &ys, false, Some(&wide)), deepest(&xs, &ys, true, Some(&wide)));
    println!("cap: deepest lot {off:.1} -> {on:.1} m");
    assert!(off <= 46.0 + 1e-9 && on > 50.0, "cap: deepest lot {off:.1} -> {on:.1} m");
}

/// A landlocked synthetic site with no water, the box `generate` uses.
fn dry_site() -> crate::site::Site {
    crate::site::build_site(7, 1700.0, 1250.0, "landlocked", crate::site::SiteOpts::default())
}

/// A closed rectangle of streets, `w` by `h` metres, its south-west corner at
/// (x, y), as four `street` edges.
fn block(g: &mut crate::graph::Graph, x: f64, y: f64, w: f64, h: f64) {
    g.add_street(x, y, x + w, y, "street", 4.0, 1, "test");
    g.add_street(x + w, y, x + w, y + h, "street", 4.0, 1, "test");
    g.add_street(x + w, y + h, x, y + h, "street", 4.0, 1, "test");
    g.add_street(x, y + h, x, y, "street", 4.0, 1, "test");
}

/// The back lanes a graph holds, as (a, b) end points.
fn back_lanes(g: &crate::graph::Graph) -> Vec<(crate::geom::Vec2, crate::geom::Vec2)> {
    g.edges.iter().filter(|e| e.alive && e.prov == crate::burgage::BACK_LANE_PROV).map(|e| (g.nodes[e.a].pt(), g.nodes[e.b].pt())).collect()
}

/// Protects `ray_to_boundary` by hand: from the centre of a 20 m square the
/// boundary is 10 m away along an axis and 10 * sqrt 2 along a diagonal;
/// pointing out of an open polyline's reach, nothing.
#[test]
fn a_ray_meets_the_boundary_where_hand_arithmetic_says() {
    use crate::geom::Vec2;
    let sq = [Vec2::new(-10.0, -10.0), Vec2::new(10.0, -10.0), Vec2::new(10.0, 10.0), Vec2::new(-10.0, 10.0)];
    assert!((super::ray_to_boundary(Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0), &sq).unwrap() - 10.0).abs() < 1e-9);
    let d = Vec2::new(1.0, 1.0).norm();
    assert!((super::ray_to_boundary(Vec2::new(0.0, 0.0), d, &sq).unwrap() - 200.0f64.sqrt()).abs() < 1e-9);
    assert!(super::ray_to_boundary(Vec2::new(30.0, 0.0), Vec2::new(1.0, 0.0), &sq).is_none());
}

/// Protects the back lane's placement on a block whose answer is hand
/// arithmetic: a 300 x 130 m block 400 m from the market (beyond 160 m, so
/// the plot depth is `DEPTH_M`, 42 m) gets one lane parallel to its long
/// streets, 2 x 42 = 84 m behind one of them (46 m from the other, at least
/// `BACK_LANE_REMAIN_M` 15), running the block's whole 300 m; a block 90 m
/// deep (under 84 + 15) gets none, nor does a block whose longest street is
/// under `BACK_LANE_MIN_FRONT_M` (45 m).
#[test]
fn a_back_lane_runs_two_plot_depths_behind_the_longest_street() {
    use crate::geom::Vec2;
    let site = dry_site();
    let market = Vec2::new(850.0, 625.0);
    let mut g = crate::graph::Graph::new();
    block(&mut g, 1000.0, 900.0, 300.0, 130.0);
    assert_eq!(super::back_lane_pass(&site, market, &mut g, 8, 720.0, None, None), 1);
    let lanes = back_lanes(&g);
    assert_eq!(lanes.len(), 1, "one lane edge: {lanes:?}");
    let (a, b) = lanes[0];
    assert!((a.y - b.y).abs() < 1e-6, "parallel to the long streets: {a:?} {b:?}");
    let y = a.y;
    assert!((y - 984.0).abs() < 1e-6 || (y - 946.0).abs() < 1e-6, "84 m behind one long street: y {y}");
    assert!((a.x - b.x).abs() > 299.0, "the block's whole length: {a:?} {b:?}");
    let e = g.edges.iter().find(|e| e.prov == crate::burgage::BACK_LANE_PROV).unwrap();
    assert_eq!((e.cls, e.w), ("lane", 3.0));
    // Too shallow (90 m: room for the 84 m strip but not 15 m behind it), and
    // too short a street.
    let mut g2 = crate::graph::Graph::new();
    block(&mut g2, 1000.0, 900.0, 300.0, 90.0);
    assert_eq!(super::back_lane_pass(&site, market, &mut g2, 8, 720.0, None, None), 0);
    let mut g3 = crate::graph::Graph::new();
    block(&mut g3, 1000.0, 900.0, 44.0, 40.0);
    assert_eq!(super::back_lane_pass(&site, market, &mut g3, 8, 720.0, None, None), 0);
}

/// Protects the gates: no back lane beyond `max_rf` of the market, through
/// the market square, or across the wall; and near the market (within 160
/// m) the core depth `DEPTH_CORE_M`, 30 m, so the lane is 60 m behind.
#[test]
fn a_back_lane_respects_reach_square_wall_and_the_core_depth() {
    use crate::geom::Vec2;
    let site = dry_site();
    let market = Vec2::new(850.0, 625.0);
    let lay = |x: f64, y: f64, max_rf: f64, ring: Option<&[Vec2]>, plaza: Option<&[Vec2]>| {
        let mut g = crate::graph::Graph::new();
        block(&mut g, x, y, 300.0, 130.0);
        let n = super::back_lane_pass(&site, market, &mut g, 8, max_rf, ring, plaza);
        (n, back_lanes(&g))
    };
    assert_eq!(lay(1000.0, 900.0, 300.0, None, None).0, 0, "beyond max_rf");
    let sq = [Vec2::new(1100.0, 950.0), Vec2::new(1120.0, 950.0), Vec2::new(1120.0, 970.0), Vec2::new(1100.0, 970.0)];
    assert_eq!(lay(1000.0, 900.0, 720.0, None, Some(&sq)).0, 0, "the block holds the square");
    let ring = [Vec2::new(1150.0, 800.0), Vec2::new(1400.0, 800.0), Vec2::new(1400.0, 1100.0), Vec2::new(1150.0, 1100.0)];
    assert_eq!(lay(1000.0, 900.0, 720.0, Some(&ring), None).0, 0, "the lane would cross the wall");
    // A block centred on the market: the core depth.
    let (n, lanes) = lay(700.0, 560.0, 720.0, None, None);
    assert_eq!(n, 1);
    let y = lanes[0].0.y;
    assert!((y - 620.0).abs() < 1e-6 || (y - 630.0).abs() < 1e-6, "60 m behind one long street: y {y}");
}

/// Protects `GrowOpts::whole_budget` and the back lanes as `generate` wires
/// them: a deep-plot town of 6 000 lays back lanes (none with the prototype
/// off) and lays more street than it does without the carried budget.
#[test]
fn a_deep_plot_town_lays_back_lanes_and_its_carried_budget() {
    let n_lanes = |t: &Town| t.graph.edges.iter().filter(|e| e.prov == crate::burgage::BACK_LANE_PROV).count();
    let (mut lanes_on, mut lanes_off) = (0, 0);
    for seed in [1u32, 2, 3] {
        lanes_on += n_lanes(&town(seed, 6000.0, true));
        lanes_off += n_lanes(&town(seed, 6000.0, false));
    }
    assert_eq!(lanes_off, 0);
    assert!(lanes_on >= 3, "back lanes laid over three towns: {lanes_on}");
    // The carried budget (`GrowOpts::whole_budget`): street `grow` laid,
    // without the lane passes, of a 12 600 m budget. Measured 2026-11-17:
    // seed 3 10 950 m (10 090 on per-epoch shares), seed 77 11 016 m (10 034).
    for seed in [3u32, 77] {
        let t = town(seed, 6000.0, true);
        let grown: f64 = t.graph.edges.iter().filter(|e| e.prov != crate::burgage::BACK_LANE_PROV && e.prov != LANE_PROV).map(|e| t.graph.nodes[e.a].pt().dist(t.graph.nodes[e.b].pt())).sum();
        assert!(grown > 10_500.0, "seed {seed}: grown street {grown:.0} of {:.0}", t.target_len);
    }
}

/// Protects the later passes: a block 250 m deep is cut once 84 m behind a
/// long street, and the 166 m left behind that lane (still over 84 + 15) is
/// cut again on the next pass, so two lanes.
#[test]
fn a_deep_block_is_cut_again_on_the_next_pass() {
    let site = dry_site();
    let mut g = crate::graph::Graph::new();
    block(&mut g, 1000.0, 800.0, 300.0, 250.0);
    let n = super::back_lane_pass(&site, crate::geom::Vec2::new(850.0, 625.0), &mut g, 8, 720.0, None, None);
    assert_eq!(n, 2, "two passes, one lane each: {:?}", back_lanes(&g));
}

/// Protects the water gates: on the synthetic river site, a block laid across
/// the river gets no back lane (its centroid, or the lane, would be wet).
#[test]
fn no_back_lane_is_laid_in_the_river() {
    use crate::geom::Vec2;
    let site = crate::site::build_site(7, 1700.0, 1250.0, "river", crate::site::SiteOpts::default());
    // A wet point near the box centre, found by scanning (the synthetic river's course is the site's own).
    let wet = (0..60).flat_map(|i| (0..40).map(move |j| Vec2::new(400.0 + 15.0 * i as f64, 300.0 + 15.0 * j as f64))).find(|&p| site.is_water(p)).expect("premise: the river site has water");
    let mut g = crate::graph::Graph::new();
    block(&mut g, wet.x - 150.0, wet.y - 65.0, 300.0, 130.0);
    assert_eq!(super::back_lane_pass(&site, Vec2::new(wet.x, wet.y - 400.0), &mut g, 8, 720.0, None, None), 0, "{:?}", back_lanes(&g));
}

/// Protects the deep-plot reach in the stretch gate
/// (`stretch::StretchMeasures::of`): for walled deep-plot towns the unbuilt
/// patch is read at `UNBUILT_REACH_M` (40 x 42 / 30 = 56 m), never larger than
/// at the reference's 40 m, and smaller for at least one of three towns (a
/// deep yard read as open ground at 40 m).
#[test]
fn a_deep_plot_town_reads_unbuilt_ground_at_its_own_reach() {
    assert!((super::UNBUILT_REACH_M - 56.0).abs() < 1e-9, "40 x 42 / 30");
    let mut smaller = 0;
    for seed in [1u32, 2, 3, 4242] {
        let t = town(seed, 6000.0, true);
        assert!(t.burgage, "premise: the town was laid out with deep plots");
        let Some((_, _, own)) = crate::stretch::StretchMeasures::of(&t).wall else { continue };
        let mut as_ref = t.clone();
        as_ref.burgage = false;
        let (_, _, at40) = crate::stretch::StretchMeasures::of(&as_ref).wall.expect("the same wall");
        assert!(own <= at40, "seed {seed}: {own} > {at40}");
        smaller += usize::from(own < at40);
    }
    assert!(smaller >= 1, "some deep yard reads open at 40 m");
}
