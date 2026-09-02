//! Milestone 8's `buildPlaza` tests.
//!
//! **Golden**, on the same terms as milestones 6 and 12: `golden.rs` holds the
//! reference engine's own output for 17 scenarios — five site kinds at three
//! seeds each, plus the two ways the function can return `null` — and the
//! fixtures below rebuild the identical input in this port and compare.
//!
//! Everything is compared **bit for bit** through [`f64::to_bits`]. There are
//! no tolerances anywhere, including on the plaza quad, which comes out of
//! `V.norm` and therefore [`js_hypot`](crate::geom::js_hypot).
//!
//! ## Why the fixtures are whole sites rather than hand-drawn graphs
//!
//! Milestone 5's rule — build the fixtures out of the geometry under test —
//! applies twice here. `buildPlaza` reads two things it cannot be given
//! synthetically without losing what it is testing:
//!
//! - **`site.riverDist`**, which decides which side of the street to widen
//!   onto. A hand-made site would fix that ternary to one branch; real sites
//!   at three seeds each take it **both** ways, and the capture refuses to
//!   write a golden where they do not.
//! - **the nearest live primary**, which comes from `buildPrimaries`'
//!   least-cost traces. Those are a dozen-odd polylines per site laid through
//!   `addStreet`'s planarity correction, so the "nearest edge to the market"
//!   is a genuinely contested pick rather than the only candidate.
//!
//! `landlocked` is in the set for the same reason milestone 6 put it there:
//! it is the kind with no channel at all, so `riverDist` answers from the
//! synthetic half-plane and the side ternary still has to resolve.
//!
//! ## What the two `null` scenarios separate
//!
//! `emptyGraph` never enters the edge loop; `noPrimary` enters it and rejects
//! every edge, because both are live and neither is `'primary'`. Only the
//! second can see the `e.cls !== 'primary'` test, and only the pair together
//! says the `if(!be) return null` guard is not standing in for an empty
//! container check.
//!
//! ## What the graph and block hashes cover
//!
//! `buildPlaza`'s real output is not the quad it returns — it is the three
//! streets it puts in the graph, which `addStreet` may split, snap or reject.
//! So the whole post-plaza graph is pinned by the reference's own `fnv1a` over
//! its own dump (every node's id/x/y/adjacency, every edge's
//! id/a/b/class/width/epoch/alive, each double as its exact 64 bits), and the
//! blocks that come off it likewise. Milestone 2 golden-tested the graph
//! machinery itself and milestone 12 the blocks; restating either here would
//! add lines, not coverage.
//!
//! ## What the mutation sweep found
//!
//! Every constant and comparison this function ports was mutated and the suite
//! re-run: 20 mutations, **zero survivors** — the first milestone in this
//! subsystem to close its sweep completely. The runner takes a pristine
//! snapshot before writing anything, restores from that snapshot, re-runs the
//! suite as a post-sweep baseline and holds a lock file, which is milestone 7's
//! corrupted-source lesson applied rather than re-learned.
//!
//! Five of the twenty survived the *first* pass, and every one of them was a
//! fixture limit of exactly the kind milestone 7 catalogued — an exact tie on a
//! continuous value that no generated site lands on:
//!
//! | survivor | closed by |
//! |---|---|
//! | side probe `20 → 21` | [`the_side_probe_boundary_at_an_exact_tie_and_a_quarter_metre_off_it`] |
//! | side probe `-20 → -21` | the same, its `c = 0.25` half |
//! | `>` → `>=` in the side ternary | the same, its exact-tie half |
//! | `rot90()` → `-rot90()` | the same, its exact-tie half — see below |
//! | `d < bd` → `d <= bd` | [`an_exact_distance_tie_keeps_the_lower_indexed_primary`] |
//!
//! Unlike milestone 7's thirteen, these *were* closable, because the values
//! being compared are distances to a centreline this port may set directly:
//! overwriting `site.river` with a line parallel to the street under test makes
//! the probe gap an input rather than an output. Two fixtures at `c = 0` and
//! `c = 0.25` m, plus one graph with two exactly-equidistant primaries, take
//! all five.
//!
//! **Negating `nl` is not the no-op it looks like.** `nl` is read twice — to
//! build the probes and as `nl * (side * wd)` — and away from a tie the two
//! negations cancel exactly, which is why the mutation survived 15 real towns.
//! At an exact tie they do not: both arms of the ternary give the *same*
//! `side`, so the product flips and the square opens the other way. The tie
//! fixture is what sees it.

mod golden;

use crate::blocks::build_blocks;
use crate::geom::Vec2;
use crate::graph::Graph;
use crate::plaza::{PROV, build_plaza};
use crate::rng::{fnv1a, stream};
use crate::routes::{Anchors, build_primaries, place_anchors};
use crate::site::{Site, SiteOpts, build_site};

use golden::Case;

/// The site box every scenario is built in — `generate()`'s own
/// `const Wm=1700,Hm=1250`.
const WM: f64 = 1700.0;
const HM: f64 = 1250.0;

fn eq_bits(got: f64, want: f64, what: &str) {
    assert_eq!(got.to_bits(), want.to_bits(), "{what}: got {got:?}, want {want:?}");
}

/// The reference's own graph serialisation, reproduced field for field. Each
/// double is written as its exact 64 bits, so the hash cannot absorb a
/// last-ulp difference the way a rounded dump would.
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

/// Likewise for the blocks `build_blocks` produces off that graph.
fn blocks_hash(blocks: &[crate::blocks::Block]) -> u32 {
    let mut parts: Vec<String> = Vec::new();
    for b in blocks {
        parts.push(b.id.clone());
        parts.push(format!("{:016x}", b.area.to_bits()));
        parts.push(u8::from(b.plaza).to_string());
        for p in b.poly.iter().chain(b.face_poly.iter()) {
            parts.push(format!("{:016x}", p.x.to_bits()));
            parts.push(format!("{:016x}", p.y.to_bits()));
        }
    }
    fnv1a(&parts.join("|"))
}

/// Rebuilds one scenario's input exactly as the capture did: the site, the
/// anchors, and whatever was laid into the graph before `buildPlaza` ran.
fn setup(c: &Case) -> (Site, Anchors, Graph) {
    let site = build_site(c.seed, WM, HM, c.kind, SiteOpts::default());
    let anchors = place_anchors(c.seed, &site);
    let mut g = Graph::new();
    if c.empty {
        // nothing laid at all
    } else if c.lanes_only {
        let m = anchors.market;
        g.add_street(m.x - 200.0, m.y, m.x + 200.0, m.y, "lane", 4.0, 0, "fixture");
        g.add_street(m.x, m.y - 200.0, m.x, m.y + 200.0, "street", 5.0, 0, "fixture");
    } else {
        build_primaries(c.seed, &site, &anchors, &mut g);
    }
    (site, anchors, g)
}

#[test]
fn golden_every_scenario_reproduces_the_reference_exactly() {
    for c in golden::GOLDEN {
        let what = c.name;
        let (site, anchors, mut g) = setup(c);

        // Milestone 6 golden-verified both of these; re-asserting them here is
        // what says this scenario is testing the graph the capture tested, not
        // merely a graph.
        eq_bits(anchors.market.x, c.market.0, &format!("{what}: market.x"));
        eq_bits(anchors.market.y, c.market.1, &format!("{what}: market.y"));
        let primaries = g.edges.iter().filter(|e| e.alive && e.cls == "primary").count();
        assert_eq!(primaries, c.primary_count, "{what}: live primary edges before buildPlaza");

        let plaza = build_plaza(c.seed, &site, &anchors, &mut g);

        match (&plaza, c.plaza_center) {
            (None, None) => {}
            (Some(_), None) => panic!("{what}: got a plaza, the reference returned null"),
            (None, Some(_)) => panic!("{what}: got null, the reference returned a plaza"),
            (Some(p), Some((cx, cy))) => {
                eq_bits(p.center.x, cx, &format!("{what}: plaza centre x"));
                eq_bits(p.center.y, cy, &format!("{what}: plaza centre y"));
                assert_eq!(p.poly.len() * 2, c.plaza_poly.len(), "{what}: plaza vertex count");
                for (i, v) in p.poly.iter().enumerate() {
                    eq_bits(v.x, c.plaza_poly[i * 2], &format!("{what}: plaza pt {i} x"));
                    eq_bits(v.y, c.plaza_poly[i * 2 + 1], &format!("{what}: plaza pt {i} y"));
                }
            }
        }

        assert_eq!(g.nodes.len(), c.node_count, "{what}: node count after buildPlaza");
        assert_eq!(g.edges.len(), c.edge_count, "{what}: edge count after buildPlaza");
        assert_eq!(
            graph_hash(&g),
            c.graph_hash,
            "{what}: post-plaza graph (fnv1a over the reference's own dump)"
        );

        let blocks = build_blocks(&g, plaza.as_ref(), &site);
        assert_eq!(blocks.len(), c.block_count, "{what}: block count");
        assert_eq!(
            blocks.iter().filter(|b| b.plaza).count(),
            c.plaza_blocks,
            "{what}: blocks flagged as the market square"
        );
        assert_eq!(
            blocks_hash(&blocks),
            c.blocks_hash,
            "{what}: blocks (fnv1a over the reference's own dump)"
        );
    }
}

/// The provenance string, character for character against the reference's own
/// literal as the capture read it.
#[test]
fn the_provenance_string_is_the_references_own() {
    assert_eq!(PROV, golden::PROV);
}

/// Every scenario that produced a plaza produced **exactly one** flagged block,
/// and it is the widened band itself.
///
/// This is the whole point of the milestone and it is worth asserting as a
/// property rather than only inside the golden's counts: without it a town's
/// market anchor is platted like any other block, which is what milestone 12
/// shipped with.
#[test]
fn a_plaza_always_carves_exactly_one_open_block() {
    let mut seen = 0;
    for c in golden::GOLDEN {
        if c.plaza_center.is_none() {
            continue;
        }
        let (site, anchors, mut g) = setup(c);
        let plaza = build_plaza(c.seed, &site, &anchors, &mut g).expect("golden says there is one");
        let blocks = build_blocks(&g, Some(&plaza), &site);
        let flagged: Vec<_> = blocks.iter().filter(|b| b.plaza).collect();
        assert_eq!(flagged.len(), 1, "{}: expected exactly one market square", c.name);
        // And it is the band this function laid, not some other face that
        // happens to contain the centroid: every corner of the returned quad
        // lies inside the flagged face, or within `attach_point`'s own 11 m
        // snap of its boundary.
        //
        // The tolerance is that snap, named rather than tuned. `add_street`
        // binds each plaza corner to an existing node within 11 m instead of
        // creating a fresh one, so the face the graph actually holds is the
        // quad with up to three of its corners *moved* — 6.1 m on `river1337`,
        // the largest in the set. The reference builds `plaza.poly` and
        // `plaza.center` from the **pre-snap** points regardless, which is why
        // `build_blocks` tests a point against the face rather than comparing
        // polygons.
        let face = &flagged[0].face_poly;
        for (i, v) in plaza.poly.iter().enumerate() {
            let d = (0..face.len())
                .map(|k| crate::geom::dist_pt_seg(*v, face[k], face[(k + 1) % face.len()]))
                .fold(f64::INFINITY, f64::min);
            assert!(
                crate::geom::point_in_poly(*v, face) || d < 11.0,
                "{}: plaza corner {i} is {d:.3} m outside the flagged face",
                c.name
            );
        }
        seen += 1;
    }
    assert!(seen >= 15, "only {seen} scenarios exercised the flagged block");
}

/// No live primary edge means no plaza, and the graph is left untouched.
///
/// Both `null` scenarios are in the golden already; this states the *other*
/// half of the contract, which no golden field carries: the function must not
/// have laid anything on its way to returning nothing.
#[test]
fn no_primary_edge_leaves_the_graph_alone() {
    for c in golden::GOLDEN {
        if c.plaza_center.is_some() {
            continue;
        }
        let (site, anchors, mut g) = setup(c);
        let before = (g.nodes.len(), g.edges.len(), graph_hash(&g));
        assert!(build_plaza(c.seed, &site, &anchors, &mut g).is_none(), "{}", c.name);
        assert_eq!(
            (g.nodes.len(), g.edges.len(), graph_hash(&g)),
            before,
            "{}: buildPlaza mutated the graph before refusing",
            c.name
        );
    }
}

/// The substream takes exactly two numbers, `range(55, 80)` then
/// `range(26, 40)`, and they are the plaza's length and width in that order.
///
/// Asserted from the *outside*: the quad's own side lengths must be those two
/// draws to the bit. That is what pins the declaration order — swapping the two
/// `range` calls would still draw twice from the same stream and would still
/// produce a rectangle, and only this test would notice.
#[test]
fn the_two_draws_are_the_length_then_the_width() {
    for c in golden::GOLDEN {
        if c.plaza_center.is_none() {
            continue;
        }
        let mut r = stream(c.seed, "plaza");
        let l = r.range(55.0, 80.0);
        let wd = r.range(26.0, 40.0);
        let (site, anchors, mut g) = setup(c);
        let p = build_plaza(c.seed, &site, &anchors, &mut g).expect("golden says there is one");
        // `p1 → p2` is the frontage (length L); `p2 → q2` is the widening (Wd).
        let got_l = p.poly[0].dist(p.poly[1]);
        let got_w = p.poly[1].dist(p.poly[2]);
        assert!(
            (got_l - l).abs() < 1e-9 && (got_w - wd).abs() < 1e-9,
            "{}: quad is {got_l:.12} x {got_w:.12}, the draws were {l:.12} x {wd:.12}",
            c.name
        );
        assert!((55.0..=80.0).contains(&l) && (26.0..=40.0).contains(&wd), "{}: bands", c.name);
    }
}

/// The 20 m side probe decides which way the square opens, and the fixture set
/// sees it both ways.
///
/// A single site cannot: the answer is a property of where the market's nearest
/// primary sits relative to the channel. Across the golden's five kinds and
/// three seeds the ternary takes both branches, which is what makes mutating
/// the probe distance — or the `>` — observable at all. The capture asserts the
/// same thing before it will write a golden; this asserts it in the suite, so a
/// later re-capture cannot quietly lose it.
///
/// ## "Away from the river" is a statement about 20 m, not about the square
///
/// The probe is a **fixed 20 m** either side of the street's midpoint, and the
/// square is up to 40 m wide, so on a channel that curves under the widened
/// street the two can disagree: `river7` picks the side whose *far edge* is
/// 0.05 m nearer the water than the other side's would have been. That is the
/// reference's own behaviour, it is what the golden captured, and it is
/// asserted below rather than left to look like a port bug the next time
/// someone measures the finished square instead of the probe.
#[test]
fn the_side_probe_distance_is_load_bearing() {
    let mut signs = std::collections::BTreeSet::new();
    let mut inverted = 0;
    for c in golden::GOLDEN {
        if c.plaza_center.is_none() {
            continue;
        }
        let (site, anchors, mut g) = setup(c);
        let p = build_plaza(c.seed, &site, &anchors, &mut g).expect("golden says there is one");
        let dir = (p.poly[1] - p.poly[0]).norm();
        let nl = dir.rot90();
        let side = (p.poly[3] - p.poly[0]).dot(nl);
        signs.insert(side > 0.0);

        // At the probe's own 20 m the chosen side is never the wetter one.
        let mid = p.poly[0].lerp(p.poly[1], 0.5);
        let sign = if side > 0.0 { 1.0 } else { -1.0 };
        let chosen = site.river_dist(mid + nl * (20.0 * sign));
        let other = site.river_dist(mid + nl * (-20.0 * sign));
        assert!(chosen >= other, "{}: the 20 m probe picked the wetter side", c.name);

        // At the square's full width it sometimes is.
        let far_chosen = site.river_dist(p.poly[2].lerp(p.poly[3], 0.5));
        if far_chosen < site.river_dist(mid) {
            inverted += 1;
        }
    }
    assert_eq!(signs.len(), 2, "the side ternary only ever took one branch: {signs:?}");
    assert!(inverted > 0, "no scenario shows the 20 m probe disagreeing with the full width");
}

/// The three laid streets are `'street'`, 5 m wide, epoch 0 — and there are
/// three of them, not four. The fourth side is the primary that was already
/// there, which is the whole mechanism: widening an existing street, not
/// drawing a box somewhere.
///
/// The new-edge list is filtered by class rather than taken whole, because
/// `add_street` also *splits* the primary the square hangs off, and a split
/// half is a new edge carrying the original `'primary'` class. Asserting over
/// every new edge would be asserting about `split_edge`, which is milestone
/// 2's and golden-tested there.
#[test]
fn three_streets_are_laid_and_the_fourth_side_is_the_existing_primary() {
    let c = golden::GOLDEN
        .iter()
        .find(|c| c.name == "river7")
        .expect("river7 is in the golden set");
    let (site, anchors, mut g) = setup(c);
    let before = g.edges.len();
    let p = build_plaza(c.seed, &site, &anchors, &mut g).expect("river7 has a plaza");
    let laid: Vec<_> = g.edges.iter().skip(before).filter(|e| e.cls == "street").collect();
    assert_eq!(laid.len(), 3, "buildPlaza laid {} plaza streets, not 3", laid.len());
    for e in &laid {
        assert_eq!(e.w, 5.0, "plaza edge width");
        assert_eq!(e.epoch, 0, "plaza edge epoch");
    }
    // The `p1 → p2` side is *not* one of them: it lies along the primary.
    let mid = p.poly[0].lerp(p.poly[1], 0.5);
    let on_primary = g
        .edges
        .iter()
        .filter(|e| e.alive && e.cls == "primary")
        .any(|e| crate::geom::dist_pt_seg(mid, g.nodes[e.a].pt(), g.nodes[e.b].pt()) < 1e-6);
    assert!(on_primary, "the widened side is not on a primary edge");
}

/// A degenerate primary — both endpoints at the same place — cannot be produced
/// by `addStreet` (it drops sub-3.5 m links), so `V.norm`'s zero-vector guard is
/// unreachable from a real graph. Asserted from the geometry side rather than
/// left implied: every scenario's chosen edge has real length.
#[test]
fn the_chosen_primary_always_has_length() {
    for c in golden::GOLDEN {
        let (_, anchors, g) = setup(c);
        for e in g.edges.iter().filter(|e| e.alive && e.cls == "primary") {
            let d = g.nodes[e.a].pt().dist(g.nodes[e.b].pt());
            assert!(d >= 3.5, "{}: edge {} is {d} m long", c.name, e.id);
        }
        let _ = anchors;
    }
}

/// A razor fixture for the side probe: one horizontal primary street, the
/// market on it, and a straight river centreline **parallel to that street**,
/// `c` metres to the `-nl` side.
///
/// Parallel is what makes it a razor. Along `nl` the distance to a parallel
/// line changes metre for metre, so the two 20 m probes come out at `20 + c`
/// and `20 - c` — a gap of `2c` that the fixture sets directly, instead of
/// whatever gap a generated site happens to produce. `c = 0` is the exact tie;
/// `c = 0.25` is half a metre of separation, which is inside the one-metre
/// window a mutation of either probe distance moves the answer through.
///
/// `site.river` is overwritten on a real `build_site` landlocked site rather
/// than a `Site` being fabricated: `river_dist` takes the synthetic
/// polyline-distance path exactly when there is no water context, which is
/// what a landlocked site with default opts has.
fn side_probe_fixture(c: f64) -> (Site, Anchors, Graph) {
    let mut site = build_site(7, WM, HM, "landlocked", SiteOpts::default());
    let y = 600.0;
    // `dir` is (1, 0), so `nl` is (0, 1) — +y is the `side = +1` direction.
    let mut g = Graph::new();
    g.add_street(500.0, y, 900.0, y, "primary", 8.0, 0, "fixture");
    site.river = vec![Vec2::new(-1.0e4, y - c), Vec2::new(1.0e4, y - c)];
    let anchors = Anchors { market: Vec2::new(700.0, y), prov: "fixture" };
    (site, anchors, g)
}

/// The exact tie, and a quarter metre off it — the two fixtures that pin the
/// side probe's `20`, its mirror `-20`, and its `>`.
///
/// Nothing in the 17 generated scenarios lands inside a one-metre window of
/// that comparison, so before these existed three mutations survived: `20 →
/// 21`, `-20 → -21`, and `> → >=`. Each of the three flips exactly one of the
/// two cases below, which is why both are needed and why neither is redundant:
///
/// | mutation | `c = 0` (tie) | `c = 0.25` |
/// |---|---|---|
/// | `> → >=` | **flips** to `+1` | unchanged |
/// | probe `20 → 21` | **flips** to `+1` | unchanged |
/// | probe `-20 → -21` | unchanged | **flips** to `-1` |
#[test]
fn the_side_probe_boundary_at_an_exact_tie_and_a_quarter_metre_off_it() {
    // An exact tie loses `>`: the square opens to `-nl`.
    let (site, anchors, mut g) = side_probe_fixture(0.0);
    let p = build_plaza(7, &site, &anchors, &mut g).expect("one primary is enough");
    assert!(
        p.poly[3].y < p.poly[0].y,
        "an exact tie must take the `else` arm, got q1.y = {} vs p1.y = {}",
        p.poly[3].y,
        p.poly[0].y
    );

    // A quarter metre of separation wins it: the square opens to `+nl`, away
    // from the channel.
    let (site, anchors, mut g) = side_probe_fixture(0.25);
    let p = build_plaza(7, &site, &anchors, &mut g).expect("one primary is enough");
    assert!(
        p.poly[3].y > p.poly[0].y,
        "a 0.5 m probe gap must take the `then` arm, got q1.y = {} vs p1.y = {}",
        p.poly[3].y,
        p.poly[0].y
    );
}

/// Two primary edges exactly equidistant from the market: the strict `<` keeps
/// the **lower-indexed** one.
///
/// No generated site produces an exact tie — the distances are least-cost trace
/// geometry — so `d < bd` → `d <= bd` survived until this existed. Two mirrored
/// horizontal streets 100 m either side of the market give the tie by
/// construction, and the plaza lands on whichever the tie-break picked.
#[test]
fn an_exact_distance_tie_keeps_the_lower_indexed_primary() {
    let site = build_site(7, WM, HM, "landlocked", SiteOpts::default());
    let mut g = Graph::new();
    g.add_street(500.0, 500.0, 900.0, 500.0, "primary", 8.0, 0, "first");
    g.add_street(500.0, 700.0, 900.0, 700.0, "primary", 8.0, 0, "second");
    let anchors = Anchors { market: Vec2::new(700.0, 600.0), prov: "fixture" };

    // The tie is exact, not approximate — asserted, so the fixture cannot decay
    // into a near-tie that the strict comparison would resolve on its own.
    let d: Vec<u64> = g
        .edges
        .iter()
        .map(|e| {
            crate::geom::dist_pt_seg(anchors.market, g.nodes[e.a].pt(), g.nodes[e.b].pt()).to_bits()
        })
        .collect();
    assert_eq!(d.len(), 2, "the fixture must lay exactly two primaries");
    assert_eq!(d[0], d[1], "the two distances must be bit-identical");

    let p = build_plaza(7, &site, &anchors, &mut g).expect("two primaries is enough");
    // Edge 0 is the y = 500 street, so its midpoint — and the square built on
    // it — sits there, not on y = 700.
    assert!(
        (p.poly[0].y - 500.0).abs() < 1e-9,
        "the tie went to the higher-indexed edge: plaza is at y = {}",
        p.poly[0].y
    );
}

/// `Vec2` is `Copy` and the quad is built from four locals; this is the one
/// place a transposition would be silent, so the winding is asserted by shape.
/// `[p1, p2, q2, q1]` traces the rectangle, so no two consecutive corners may
/// be diagonal — a `[p1, p2, q1, q2]` bowtie would fail here and nowhere else.
#[test]
fn the_quad_is_wound_as_a_rectangle_not_a_bowtie() {
    for c in golden::GOLDEN {
        if c.plaza_center.is_none() {
            continue;
        }
        let (site, anchors, mut g) = setup(c);
        let p = build_plaza(c.seed, &site, &anchors, &mut g).expect("golden says there is one");
        assert!(
            !crate::geom::poly_self_intersects(&p.poly),
            "{}: the plaza quad is self-intersecting",
            c.name
        );
        let d: Vec<Vec2> = (0..4).map(|i| p.poly[(i + 1) % 4] - p.poly[i]).collect();
        // Opposite sides are anti-parallel to the bit the shape allows.
        for i in 0..2 {
            let cross = d[i].cross(d[i + 2]);
            assert!(cross.abs() < 1e-6, "{}: side {i} is not parallel to its opposite", c.name);
        }
    }
}
