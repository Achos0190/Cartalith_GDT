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
    for seed in [4242u32, 77] {
        let (off, on) = (town(seed, 6000.0, false), town(seed, 6000.0, true));
        let ((r_off, d_off), (r_on, d_on)) = (lots(&off), lots(&on));
        let (m_off, m_on) = (r_off[r_off.len() / 2], r_on[r_on.len() / 2]);
        let lanes = |t: &Town| t.graph.edges.iter().filter(|e| e.alive && e.prov == LANE_PROV).count();
        // Measured 2026-11-17 (release): median 1.34 -> 3.08 and 1.37 -> 2.99;
        // blocks 998 -> 2 939 and 1 180 -> 2 527 m2; lanes 6 -> 0 and 5 -> 0.
        assert!(m_off < 1.6, "seed {seed}: the reference's lots are house-deep ({m_off:.2})");
        assert!(m_on > 2.5, "seed {seed}: deep plots ({m_on:.2})");
        assert!(mean_block(&on) > 1.8 * mean_block(&off), "seed {seed}: blocks {:.0} vs {:.0} m2", mean_block(&on), mean_block(&off));
        assert!(lanes(&off) > 0 && lanes(&on) == 0, "seed {seed}: lanes {} -> {}", lanes(&off), lanes(&on));
        deep_off += d_off;
        deep_on += d_on;
    }
    // Measured: 0.005 + 0.003 -> 0.088 + 0.013 of lots over 40 m deep.
    assert!(deep_off < 0.02 && deep_on > 0.05, "lots over 40 m: {deep_off:.3} -> {deep_on:.3}");
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
