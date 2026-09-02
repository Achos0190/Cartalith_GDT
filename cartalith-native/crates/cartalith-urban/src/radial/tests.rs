//! Milestone 8's `buildRadialStreets` / `buildWaterway` tests.
//!
//! **Golden**, on the same terms as milestones 6, 8a and 12: `golden.rs` holds
//! the reference engine's own output for **30** scenarios and the fixtures
//! below rebuild the identical input in this port and compare. Fifteen are
//! five site kinds at three seeds each at `generate()`'s own `maxRF` for a
//! 3 000-person town; six more sit at populations chosen so both `Math.max`
//! floors bite in three of them and neither bites in the other three; two more
//! at a population whose residential ring is the one place in the set where
//! `Math.round(radius/8)` disagrees with a truncation; and seven are probes.
//!
//! Everything is compared **bit for bit** through [`f64::to_bits`]. There are
//! no tolerances anywhere.
//!
//! ## What pins the twenty-eight-draw contract
//!
//! The capture instruments `rOrg` with a delegating counter and every scenario
//! reports 28, which is what `draws` carries. On the Rust side the enforcement
//! is `graph_hash`, and it is genuinely enforcement rather than a restatement:
//! the per-spoke jitter draw is taken *before* `landSeg` votes, so in the
//! majority of scenarios where some spoke is rejected and a later one is laid,
//! a port that skipped the rejected spoke's draw would shift every subsequent
//! spoke and cross-spoke angle and the hash would not match. `landlocked*` are
//! the controls that lay all twelve of both.
//!
//! ## Why the fixtures are whole sites rather than hand-drawn ones
//!
//! Milestone 5's rule again. `land()` is the only thing standing between a
//! ring and open water, and it reads `isWater`, `riverDist`, `riverW` and the
//! box margin together. A synthetic site fixes all four, and then the ring
//! splitting — the part of this function most likely to be got wrong — never
//! runs. Across the set the ring streets split into between 69 and 219 live
//! edges, the spokes between 10 and 61, and the cross-spokes between 10 and 27,
//! so every one of `landSeg`'s and `flush`'s branches is exercised.
//!
//! `landlocked` earns its place twice over: with no channel at all, `riverW`
//! is `0`, so the `site.riverW ? … : 0` ternary takes its **falsy** arm there
//! and its truthy arm everywhere else.
//!
//! ## The seven probes, and the one override the reference allows
//!
//! `edgeWest`/`edgeEast`/`edgeNorth`/`edgeSouth` push the market until the
//! residential ring's own extreme sits on the 25 m box margin, where the ring's
//! vertices crowd together in that coordinate and the margin is evaluated by
//! many points a metre either side of it rather than by none. `tieMargin` sets
//! `site.riverW` from a *measured* distance so that `riverDist(p) > riverW/2+8`
//! is an exact tie at a sample the function evaluates. `grid12v11river` and
//! `grid12v11through` pass a `maxRF` past `generate()`'s own 720 m cap —
//! legitimate input to *this* function, which takes it as a plain argument —
//! because a 13-sample grid and a 12-sample one only ever disagree on a spoke
//! long enough for the gap between the grids to exceed `land()`'s river band.
//!
//! **`site.river` is deliberately never overridden.** The reference's
//! `riverDistSynth` closes over `buildSite`'s local array and never reads the
//! property back off the returned object, so replacing it changes nothing
//! there — measured against the reference, after an earlier draft of these
//! probes did exactly that and produced graphs identical to the un-probed ones.
//! `site.riverW` is the one field `land()` reads live, and it is the only site
//! override any of these fixtures uses.
//!
//! ## What the mutation sweep found
//!
//! Fifty-one mutations of this module's constants, comparisons and libm calls,
//! each applied alone to a scratch copy with the whole suite re-run against a
//! private target directory: **48 killed, 3 survivors, and all three are
//! proved dead rather than uncovered.**
//!
//! | survivor | why no input can see it |
//! |---|---|
//! | `riverW` ternary condition → `false` | [`the_falsy_river_width_arm_is_unreachable_in_this_engine`] |
//! | `run.len() > 1` → `> 0` (mid-ring) | [`a_one_point_polyline_lays_nothing`] |
//! | `run.len() > 1` → `> 0` (final flush) | the same |
//!
//! Ten survived the first pass. Five were closed by the four `edge*` probes,
//! two by the `roundBoundary*` scenarios (`Math.round` vs a truncation, and
//! `radius/8` vs `radius/8.1`, which agree on every ring in a normally-sized
//! town), one by `tieMargin` and one by the two `grid12v11*` scenarios. The
//! runner takes a pristine snapshot first, restores from it, and re-runs the
//! suite as a post-sweep baseline — milestone 7's corrupted-source lesson
//! applied rather than re-learned.

mod golden;

use crate::geom::Vec2;
use crate::graph::Graph;
use crate::radial::{
    PROV_CROSS, PROV_RING, PROV_SPOKE, PROV_WATERWAY, WATERWAY_KIND, build_radial_streets,
    build_waterway,
};
use crate::rng::fnv1a;
use crate::routes::{Anchors, place_anchors};
use crate::site::{Site, SiteOpts, build_site};

use golden::Case;

/// The site box every scenario is built in — `generate()`'s own
/// `const Wm=1700,Hm=1250`.
const WM: f64 = 1700.0;
const HM: f64 = 1250.0;

fn eq_bits(got: f64, want: f64, what: &str) {
    assert_eq!(got.to_bits(), want.to_bits(), "{what}: got {got:?}, want {want:?}");
}

/// The reference's own graph serialisation, field for field — the same dump
/// milestone 8a's plaza golden hashes. Each double is written as its exact 64
/// bits, so the hash cannot absorb a last-ulp difference.
fn graph_hash(g: &Graph) -> u32 {
    let mut parts: Vec<String> = Vec::new();
    for n in &g.nodes {
        parts.push(n.id.to_string());
        parts.push(format!("{:016x}", n.x.to_bits()));
        parts.push(format!("{:016x}", n.y.to_bits()));
        parts.push(n.adj.iter().map(usize::to_string).collect::<Vec<_>>().join(","));
    }
    for e in &g.edges {
        parts.push(e.id.to_string());
        parts.push(e.a.to_string());
        parts.push(e.b.to_string());
        parts.push(e.cls.to_string());
        parts.push(format!("{:016x}", e.w.to_bits()));
        parts.push(e.epoch.to_string());
        parts.push(u8::from(e.alive).to_string());
    }
    fnv1a(&parts.join("|"))
}

/// Likewise over the canal's own vertices.
fn poly_hash(pts: &[Vec2]) -> u32 {
    let mut parts: Vec<String> = Vec::new();
    for p in pts {
        parts.push(format!("{:016x}", p.x.to_bits()));
        parts.push(format!("{:016x}", p.y.to_bits()));
    }
    fnv1a(&parts.join("|"))
}

/// Rebuilds one scenario's input exactly as the capture did.
///
/// The two overrides are the capture's, applied here line for line. **They are
/// the only two the reference lets a fixture apply**: `site.river` cannot be
/// overridden, because `riverDistSynth` (reference line 28624) closes over
/// `buildSite`'s own local array and never reads the property back off the
/// returned object; `site.riverW` can be, because `land()` reads that one live
/// (line 28868). Measured against the reference, not assumed — an earlier draft
/// of these fixtures overrode `site.river` and the capture produced graphs
/// identical to the un-probed ones.
fn setup(c: &Case) -> (Site, Anchors) {
    let mut site = build_site(c.seed, WM, HM, c.kind, SiteOpts::default());
    let mut anchors = place_anchors(c.seed, &site);
    if let Some((x, y)) = c.market_override {
        anchors.market = Vec2::new(x, y);
    }
    if let Some(w) = c.river_w_override {
        site.river_w = w;
    }
    (site, anchors)
}

fn prov_count(g: &Graph, prov: &str) -> usize {
    g.edges.iter().filter(|e| e.alive && e.prov == prov).count()
}

#[test]
fn golden_every_scenario_reproduces_the_reference_exactly() {
    assert!(golden::GOLDEN.len() >= 21, "the golden set shrank");
    for c in golden::GOLDEN {
        let what = c.name;
        let (site, anchors) = setup(c);

        // Milestone 6 golden-verified `place_anchors`; restating it here is
        // what says this scenario is testing the site the capture tested.
        eq_bits(anchors.market.x, c.market.0, &format!("{what}: market.x"));
        eq_bits(anchors.market.y, c.market.1, &format!("{what}: market.y"));

        let mut g = Graph::new();
        let rs = build_radial_streets(c.seed, &site, &anchors, &mut g, c.max_rf);

        eq_bits(rs.center.x, c.center.0, &format!("{what}: center.x"));
        eq_bits(rs.center.y, c.center.1, &format!("{what}: center.y"));
        eq_bits(rs.outer_r, c.outer_r, &format!("{what}: outerR"));
        assert_eq!(rs.spokes, c.spokes, "{what}: spokes");
        assert_eq!(rs.rings.len(), c.rings.len(), "{what}: ring count");
        assert_eq!(rs.rings.len(), 6, "{what}: the hub ring plus nRings = 5");
        for (i, r) in rs.rings.iter().enumerate() {
            eq_bits(*r, c.rings[i], &format!("{what}: ring {i}"));
        }
        eq_bits(
            *rs.rings.last().unwrap(),
            rs.outer_r,
            &format!("{what}: the last ring is the residential ring"),
        );

        // The graph is the real output. Counts first, so a failure says what
        // moved before the hash says only *that* something did.
        assert_eq!(
            g.edges.iter().filter(|e| e.alive && e.cls == "primary").count(),
            c.live_primary,
            "{what}: live 'primary' edges"
        );
        assert_eq!(
            g.edges.iter().filter(|e| e.alive && e.cls == "street").count(),
            c.live_street,
            "{what}: live 'street' edges"
        );
        assert_eq!(prov_count(&g, PROV_RING), c.prov_ring, "{what}: ring-street edges");
        assert_eq!(prov_count(&g, PROV_SPOKE), c.prov_spoke, "{what}: spoke edges");
        assert_eq!(prov_count(&g, PROV_CROSS), c.prov_cross, "{what}: cross-spoke edges");
        assert_eq!(g.nodes.len(), c.node_count, "{what}: node count");
        assert_eq!(g.edges.len(), c.edge_count, "{what}: edge count");
        assert_eq!(
            graph_hash(&g),
            c.graph_hash,
            "{what}: post-radial graph (fnv1a over the reference's own dump)"
        );

        // --- the canal, at generate()'s own radius and at two others -------
        let w = build_waterway(c.seed, &site, &anchors, c.max_rf * 0.95);
        assert_eq!(w.len(), c.waterway_len, "{what}: waterway record count");
        if let Some(ww) = w.first() {
            assert_eq!(ww.kind, c.waterway_kind, "{what}: waterway kind");
            assert_eq!(ww.prov, PROV_WATERWAY, "{what}: waterway provenance");
            assert_eq!(ww.poly.len(), c.waterway_pts, "{what}: waterway vertex count");
            eq_bits(ww.poly[0].x, c.waterway_first.0, &format!("{what}: canal vertex 0 x"));
            eq_bits(ww.poly[0].y, c.waterway_first.1, &format!("{what}: canal vertex 0 y"));
            eq_bits(ww.poly[16].x, c.waterway_quarter.0, &format!("{what}: canal vertex 16 x"));
            eq_bits(ww.poly[16].y, c.waterway_quarter.1, &format!("{what}: canal vertex 16 y"));
            assert_eq!(poly_hash(&ww.poly), c.waterway_hash, "{what}: canal vertices");
        }

        let w60 = build_waterway(c.seed, &site, &anchors, 60.0);
        assert_eq!(w60.len(), c.w60_len, "{what}: waterway at r = 60");
        if let Some(ww) = w60.first() {
            assert_eq!(poly_hash(&ww.poly), c.w60_hash, "{what}: canal vertices at r = 60");
        }
        assert_eq!(
            build_waterway(c.seed, &site, &anchors, 39.9999).len(),
            c.w40_len,
            "{what}: waterway just under the 40 m floor"
        );
    }
}

/// The four provenance strings, character for character against the
/// reference's own literals as the capture read them out of its own output.
///
/// Retyped by hand these would be exactly the kind of thing that drifts: two
/// of them carry an em-dash and the canal's carries an escaped apostrophe.
#[test]
fn the_provenance_strings_are_the_references_own() {
    assert_eq!(PROV_RING, golden::PROV_RING);
    assert_eq!(PROV_SPOKE, golden::PROV_SPOKE);
    assert_eq!(PROV_CROSS, golden::PROV_CROSS);
    assert_eq!(PROV_WATERWAY, golden::PROV_WATERWAY);
    assert_eq!(WATERWAY_KIND, "waterway");
}

/// Every scenario took the same twenty-eight draws, and nothing here is empty.
///
/// The non-emptiness half is the lesson four subsystems in this port learned
/// the hard way: a golden that only compares hashes passes happily on a set of
/// empty graphs. Asserted as a shape property so a capture that silently
/// produced nothing could not be written and then agreed with.
#[test]
fn the_golden_set_is_neither_empty_nor_degenerate() {
    let mut all_twelve = 0;
    let mut some_rejected = 0;
    for c in golden::GOLDEN {
        assert_eq!(c.draws, 28, "{}: the 'radial-organic' draw budget", c.name);
        assert!(c.edge_count > 0, "{}: laid no streets at all", c.name);
        assert!(c.node_count > 0, "{}: made no nodes at all", c.name);
        assert!(c.prov_ring > 0, "{}: laid no ring streets", c.name);
        assert!(c.prov_spoke > 0, "{}: laid no spokes", c.name);
        assert_eq!(c.rings.len(), 6, "{}: ring count", c.name);
        assert_eq!(c.spokes, 12, "{}: spoke count", c.name);
        assert_eq!(c.waterway_pts, 65, "{}: 64 sides means 65 points", c.name);
        assert_eq!(c.w40_len, 0, "{}: 39.9999 m must be rejected", c.name);
        // A spoke is split by the four interior rings into five edges, so all
        // twelve laid is 60; a cross-spoke crosses one ring, so all twelve is 24.
        if c.prov_spoke == 60 && c.prov_cross == 24 {
            all_twelve += 1;
        } else {
            some_rejected += 1;
        }
    }
    assert!(all_twelve >= 3, "no scenario laid the whole skeleton");
    assert!(some_rejected >= 10, "no scenario had a radial rejected by landSeg");
}

/// Both `Math.max` floors are reached, and both are also *not* reached.
///
/// `outerR = max(90, maxRF*0.38)` and `hubR = max(24, outerR*0.13)`. A fixture
/// set that only ever took one arm of either would let the floor be deleted.
#[test]
fn both_radius_floors_bite_somewhere_and_are_slack_somewhere() {
    let outer_floored = golden::GOLDEN.iter().filter(|c| c.outer_r == 90.0).count();
    let outer_free = golden::GOLDEN.iter().filter(|c| c.outer_r > 90.0).count();
    let hub_floored = golden::GOLDEN.iter().filter(|c| c.rings[0] == 24.0).count();
    let hub_free = golden::GOLDEN.iter().filter(|c| c.rings[0] > 24.0).count();
    assert!(outer_floored > 0 && outer_free > 0, "outerR: {outer_floored} floored, {outer_free} free");
    assert!(hub_floored > 0 && hub_free > 0, "hubR: {hub_floored} floored, {hub_free} free");
}

/// The rings are evenly spaced and never cross, which is what the 5.5% wobble
/// amplitude is chosen to guarantee (reference line 28856).
///
/// A property, not a golden restatement: the goldens pin the six radii, but
/// nothing there says *why* 0.055 rather than 0.5, and 0.5 would let ring `i`'s
/// outer excursion pass ring `i+1`'s inner one. Checked against the ring gap
/// the port actually computes, at the tightest ring pair in the set.
#[test]
fn consecutive_rings_cannot_cross_at_the_chosen_wobble_amplitude() {
    for c in golden::GOLDEN {
        let (site, anchors) = setup(c);
        let mut g = Graph::new();
        let rs = build_radial_streets(c.seed, &site, &anchors, &mut g, c.max_rf);
        for w in rs.rings.windows(2) {
            let (a, b) = (w[0], w[1]);
            assert!(b > a, "{}: rings not ascending: {a} then {b}", c.name);
            // 0.055 amplitude, so ring r spans [r*0.945, r*1.055].
            assert!(
                b * 0.945 > a * 1.055,
                "{}: rings {a} and {b} can overlap at the wobble amplitude",
                c.name
            );
        }
    }
}

/// A site on which `land()` is false everywhere still returns a complete,
/// correct layout — and lays nothing.
///
/// The reference has no guard for this: `drawRing`'s `flush` simply never sees
/// a run of two, and `landSeg` rejects all twenty-four radials. The 28 draws
/// are still taken. Built by overwriting `river_w` on a real landlocked site,
/// which is milestone 8a's trick — a field this port may set is what turns an
/// unreachable branch into a fixture.
#[test]
fn a_site_that_is_land_nowhere_lays_nothing_and_still_returns_its_rings() {
    let mut site = build_site(7, WM, HM, "landlocked", SiteOpts::default());
    let anchors = place_anchors(7, &site);
    // `riverDist(p) > riverW/2 + 8` can then never hold: the synthetic
    // centreline is a few hundred metres away at most inside a 1700 x 1250 box.
    site.river_w = 1.0e6;
    let max_rf = golden::GOLDEN[0].max_rf; // generate()'s own, for a 3 000-person town
    let mut g = Graph::new();
    let rs = build_radial_streets(7, &site, &anchors, &mut g, max_rf);
    assert_eq!(g.nodes.len(), 0, "nothing should have been laid");
    assert_eq!(g.edges.len(), 0, "nothing should have been laid");
    assert_eq!(rs.rings.len(), 6);
    assert_eq!(rs.spokes, 12);
    assert_eq!(rs.center, anchors.market);
    eq_bits(rs.outer_r, max_rf * 0.38, "outerR is computed regardless");
}

/// The `site.riverW ? … : 0` ternary's **falsy** arm is unreachable inside this
/// engine, and this is the argument, checked rather than asserted.
///
/// `buildSite` sets `riverW = 0` on exactly one kind — landlocked — and a
/// landlocked site's centreline is the far synthetic dummy at
/// `(-10 000, -10 000)`, so every point inside the 1700 × 1250 box is about
/// 15 km from it. The falsy arm gives a margin of 0 and the truthy arm would
/// give 8; both are cleared by four orders of magnitude, so no reachable input
/// can tell the two apart.
///
/// That is why the mutation replacing the ternary's condition with `false`
/// survives the sweep. It is recorded as proved dead rather than as a coverage
/// gap, and this test is what stops that claim going stale: if a later
/// milestone gives a landlocked site a real centreline, it fails here first.
#[test]
fn the_falsy_river_width_arm_is_unreachable_in_this_engine() {
    for kind in ["river", "riverthrough", "coast", "bay", "landlocked"] {
        for seed in [7u32, 41, 1337] {
            let site = build_site(seed, WM, HM, kind, SiteOpts::default());
            if kind != "landlocked" {
                assert!(site.river_w > 0.0, "{kind}{seed}: riverW is {}", site.river_w);
                continue;
            }
            assert_eq!(site.river_w, 0.0, "{kind}{seed}: only landlocked takes the falsy arm");
            let mut min_d = f64::INFINITY;
            for i in 0..=60 {
                for j in 0..=60 {
                    let p = Vec2::new(WM * f64::from(i) / 60.0, HM * f64::from(j) / 60.0);
                    min_d = min_d.min(site.river_dist(p));
                }
            }
            assert!(min_d > 8.0, "{kind}{seed}: a point sits {min_d} m from the dummy centreline");
        }
    }
}

/// A one-point run lays nothing whichever way the guard is written, which is
/// why `run.len() > 1` cannot be mutation-killed.
///
/// The reference's `flush` is `if(run.length>1)addPolylineStreet(...)`, and
/// `addPolylineStreet` itself iterates consecutive pairs — so handing it a
/// single point is already a no-op. Relaxing the guard to `> 0` changes the
/// call count and not the graph. Recorded as proved dead, with the proof here
/// rather than in prose.
#[test]
fn a_one_point_polyline_lays_nothing() {
    let mut g = Graph::new();
    let made = g.add_polyline_street(&[Vec2::new(100.0, 100.0)], "street", 4.5, 0, PROV_RING);
    assert!(made.is_empty(), "a one-point run produced {} edges", made.len());
    assert_eq!(g.nodes.len(), 0, "and it must not create a node either");
    assert_eq!(g.edges.len(), 0);
}

/// The canal's radius cap and its rejection floor, at the exact boundary.
///
/// Nothing `generate()` produces lands near `radius < 40` — `maxRF * 0.95` is
/// at least 195 m — so the mutation that moved that constant survived every
/// real town. `edgeR` is `min(C.x, C.y, Wm-C.x, Hm-C.y) - 12`, and the market
/// is a value this test may set, so the boundary becomes an input: a market at
/// `(52, 700)` gives `edgeR = 40` exactly, which is kept, and one at
/// `(51.9999, 700)` gives `39.9999`, which is dropped.
#[test]
fn the_canal_is_kept_at_forty_metres_and_dropped_just_below_it() {
    let site = build_site(7, WM, HM, "landlocked", SiteOpts::default());
    let at = |x: f64| Anchors { market: Vec2::new(x, 700.0), prov: "fixture" };

    // edgeR = min(52, 700, 1648, 550) - 12 = 40. Requested 900 m, capped to 40.
    let kept = build_waterway(7, &site, &at(52.0), 900.0);
    assert_eq!(kept.len(), 1, "40 m exactly is not below 40");
    eq_bits(kept[0].poly[0].x, 52.0 + 40.0, "the cap, not the request, set the radius");
    assert_eq!(kept[0].poly.len(), 65);

    // A tenth of a millimetre in, and it is gone.
    assert_eq!(build_waterway(7, &site, &at(51.9999), 900.0).len(), 0, "39.9999 m is below 40");

    // And the cap is a `min`, not an assignment: a request under `edgeR` wins.
    let small = build_waterway(7, &site, &at(600.0), 100.0);
    assert_eq!(small.len(), 1);
    eq_bits(small[0].poly[0].x, 700.0, "the request, not the cap, set the radius");
}

/// The canal closes — to within a picometre, and **not** bit for bit.
///
/// The reference's own comment (line 28930) says a closed ring is the whole
/// point of the `edgeR` cap: an earlier version ran off the map edge and was
/// cut flat there, and "a fully-closed circle never terminates in a straight
/// edge".
///
/// The last vertex is nevertheless not the first one's bits. `k = sides` gives
/// `a = 2π`, and `Math.sin(2*Math.PI)` is `-2.4492935982947064e-16`, not `0` —
/// `2*Math.PI` is the nearest double to 2π, not 2π. At the largest radius in
/// the set that is about 1.5e-13 m of gap in `y`. This test asserted bitwise
/// equality on the first run and failed exactly there; the reference has the
/// same gap (the golden's `waterway_hash` covers all 65 vertices and matches),
/// so it is the reference's behaviour and not this port's, and a consumer that
/// dedupes the closing vertex must do it by tolerance rather than by identity.
#[test]
fn the_canal_is_a_closed_ring() {
    let mut max_gap = 0.0f64;
    for c in golden::GOLDEN {
        let (site, anchors) = setup(c);
        let w = build_waterway(c.seed, &site, &anchors, c.max_rf * 0.95);
        let poly = &w.first().expect("the golden says there is one").poly;
        assert_eq!(poly.len(), 65, "{}: 64 sides", c.name);
        let gap = poly[64].dist(poly[0]);
        assert!(gap < 1.0e-9, "{}: the canal does not close: {gap} m", c.name);
        max_gap = max_gap.max(gap);
    }
    // Non-zero, or the paragraph above is describing something that stopped
    // happening and this test has quietly become an identity check.
    assert!(max_gap > 0.0, "sin(2π) is exactly zero now; re-derive the note above");
}

/// The class split the wall's gate logic depends on: every spoke is
/// `'primary'`, every ring and every cross-spoke is `'street'`.
///
/// Reference line 28884 records what happens when this is got wrong — an
/// earlier version left the spokes untagged and produced a fortified Venus town
/// with **zero** land gates, because `buildWall` only cuts one where a
/// `'primary'` crosses the trace. Asserted as a property because milestone 10
/// will read it and the golden's class counts alone would not say which edge
/// got which class.
#[test]
fn only_the_spokes_are_primary() {
    let mut checked = 0;
    for c in golden::GOLDEN {
        let (site, anchors) = setup(c);
        let mut g = Graph::new();
        build_radial_streets(c.seed, &site, &anchors, &mut g, c.max_rf);
        for e in g.edges.iter().filter(|e| e.alive) {
            let want = if e.prov == PROV_SPOKE { "primary" } else { "street" };
            assert_eq!(e.cls, want, "{}: edge {} has provenance {:?}", c.name, e.id, e.prov);
            checked += 1;
        }
        // ...and the widths: 6.5 on the residential ring, 4.5 on the inner
        // rings and the cross-spokes, 5 on the spokes.
        for e in g.edges.iter().filter(|e| e.alive) {
            if e.prov == PROV_SPOKE {
                eq_bits(e.w, 5.0, "spoke width");
            } else if e.prov == PROV_CROSS {
                eq_bits(e.w, 4.5, "cross-spoke width");
            } else {
                assert!(e.w == 6.5 || e.w == 4.5, "{}: ring width {}", c.name, e.w);
            }
            assert_eq!(e.epoch, 0, "{}: everything here is epoch 0", c.name);
        }
    }
    assert!(checked > 2000, "only {checked} edges checked across the set");
}

/// The outermost ring is the wide one, and it is the only wide one.
///
/// `idx === ringR.length-1 ? 6.5 : 4.5`. A port that widened the *hub* instead
/// would still produce six rings and the same total edge count.
#[test]
fn the_residential_ring_is_the_only_six_and_a_half_metre_one() {
    let c = &golden::GOLDEN[0];
    let (site, anchors) = setup(c);
    let mut g = Graph::new();
    let rs = build_radial_streets(c.seed, &site, &anchors, &mut g, c.max_rf);
    let wide: Vec<&crate::graph::Edge> =
        g.edges.iter().filter(|e| e.alive && e.prov == PROV_RING && e.w == 6.5).collect();
    assert!(!wide.is_empty(), "no 6.5 m ring street was laid at all");
    // Every wide edge sits on the outermost ring: within the 5.5% wobble band
    // about `outer_r`, and further out than the second ring's own outer edge.
    for e in &wide {
        for nid in [e.a, e.b] {
            let d = g.nodes[nid].pt().dist(rs.center);
            assert!(
                d > rs.rings[4] * 1.055 && d < rs.outer_r * 1.056,
                "{}: a 6.5 m edge sits at {d}, not on the residential ring at {}",
                c.name,
                rs.outer_r
            );
        }
    }
}

/// The cross-spokes span `rings[2] → rings[4]` and stop one ring short of the
/// wall boundary, which is the property that lets them stay ungated.
///
/// `midR = ringR[Math.max(1, Math.floor(nRings/2))]` and
/// `crossOuter = ringR[ringR.length-2]`. Both indices are constants a mutation
/// moves silently: `ringR[1]` and `ringR[3]` produce a perfectly plausible
/// town. Checked on `landlocked7`, where all twelve are laid, so no cross-spoke
/// is missing for reasons unrelated to the index.
#[test]
fn cross_spokes_run_from_the_third_ring_to_the_fifth() {
    let c = golden::GOLDEN
        .iter()
        .find(|c| c.name == "landlocked7")
        .expect("landlocked7 is the all-twelve control");
    let (site, anchors) = setup(c);
    let mut g = Graph::new();
    let rs = build_radial_streets(c.seed, &site, &anchors, &mut g, c.max_rf);

    let mut near = f64::INFINITY;
    let mut far = 0.0f64;
    let mut n = 0;
    for e in g.edges.iter().filter(|e| e.alive && e.prov == PROV_CROSS) {
        for nid in [e.a, e.b] {
            let d = g.nodes[nid].pt().dist(rs.center);
            near = near.min(d);
            far = far.max(d);
        }
        n += 1;
    }
    assert_eq!(n, 24, "twelve cross-spokes, each split by rings[3]");
    // `add_street` snaps an endpoint to a ring node within 11 m, so the
    // envelope is the ring radius plus the wobble band plus that snap.
    assert!(near > rs.rings[1], "a cross-spoke reached inside rings[2] ({near})");
    assert!(far < rs.rings[5] * 0.945, "a cross-spoke reached the residential ring ({far})");
    assert!(
        (near - rs.rings[2]).abs() < rs.rings[2] * 0.055 + 11.0,
        "the inner end is not on rings[2]: {near} vs {}",
        rs.rings[2]
    );
    assert!(
        (far - rs.rings[4]).abs() < rs.rings[4] * 0.055 + 11.0,
        "the outer end is not on rings[4]: {far} vs {}",
        rs.rings[4]
    );
}
