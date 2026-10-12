//! Milestone 12's tests.
//!
//! **Golden**, on the same terms as milestones 2 and 7: `golden.rs` holds the
//! reference engine's own `buildBlocks`/`buildParcels` output for five
//! scenarios — three site kinds on a wide grid, a shallow-row grid, and a
//! diagonal-cut grid, the last two added because mutations survived without
//! them — and the fixtures below rebuild the identical input in this port and
//! compare. The comparison is exact: nothing here passes through a
//! transcendental whose last bit could legitimately differ between V8 and Rust
//! (the ray-casts are all rational arithmetic; the only library call in the
//! whole path is `logn`'s `exp`/`log`, which `rng.rs` already routes through
//! the reference's own semantics).
//!
//! Coverage is a hash over **everything** plus a handful of fully-written
//! anchors — see [`golden::Scenario::hash`]. The counts are asserted
//! separately and first, because a hash mismatch on a length difference is the
//! least informative failure this file could produce.
//!
//! ## What the mutation sweep found
//!
//! Golden-matching is necessary and not sufficient, so every constant this
//! milestone ports was mutated by one unit and the suite re-run. **Caught:**
//! the 2000 m probe-ray length, the 1.4 m verge, the 0.42 ray-cast depth
//! factor, the 6.4 m burgage-split threshold, the 0.4 split chance, the
//! 0.4-0.6 split ratio, the 11 m frontage median, the `age/3` subdivision
//! divisor, the 26 m² lot floor, and the frontage-parameter arithmetic. The
//! 140_000 m² face ceiling is caught by [`block_area_ceiling_boundary`], which
//! exists because the five scenarios did not pin it.
//!
//! **Not caught, and why each is a fixture limit rather than a hole:** the
//! 120 m² face floor is unreachable at all (see
//! [`block_area_ceiling_boundary`]'s note on `SNAP`); the 7 m minimum
//! frontage, the 4 m minimum depth, the `depthTarget*1.35` bisector cap, the
//! `riverW/2 + 1` wet margin and the 0.97 area-conservation trim are each
//! filters whose one-unit window no face or lot in any scenario lands inside
//! — the trim in particular never fires, because the centroid-overlap
//! rejection already brings every block under its cap. Each is pinned by the
//! full-state hash for every value the fixtures *do* produce; none is pinned
//! at its own boundary. Milestones 8 and 11 change the input graph, and are
//! the right moment to re-run this sweep rather than trusting it.
//!
//! ## One thing no golden can catch
//!
//! Every scenario's rules are finite, so none of them reaches the NaN
//! branches the reference's own `||` and `Math.min` semantics create — and
//! those branches are reachable in the app, because `applyPlotChaos` writes a
//! NaN slider straight into the parcel rules.
//! [`a_nan_rule_does_not_reach_lot_geometry`] is the only test that goes
//! there, and it found a real divergence: `(eLen)/(acc||eLen)` takes the
//! `eLen` branch for a NaN `acc`, which a Rust `acc == 0.0` test misses.

mod golden;

use crate::blocks::{Parcel, build_blocks, build_parcels};
use crate::geom::Vec2;
use crate::graph::Graph;
use crate::rng::fnv1a;
use crate::routes::place_anchors;
use crate::site::{SiteOpts, build_site};

/// The capture's own jitter table, verbatim.
const JIT: [f64; 8] = [6.5, -4.25, 9.0, -7.5, 3.25, -2.0, 11.5, -9.75];
/// The capture's `WIDE` grid.
const XS: [f64; 5] = [300.0, 520.0, 760.0, 1010.0, 1240.0];
// Rows.
const YS: [f64; 4] = [280.0, 470.0, 690.0, 900.0];
/// The capture's `NARROW` grid — shallow rows, so the bisector ray-cast cap
/// binds below the plot depth. See `golden.rs`'s header for why it exists.
const NXS: [f64; 5] = [300.0, 560.0, 830.0, 1090.0, 1340.0];
// Rows, shallow enough to bind the bisector ray-cast cap.
const NYS: [f64; 5] = [300.0, 332.0, 366.0, 398.0, 430.0];

/// The grid a scenario is built on, by name — the golden fixture carries the
/// outputs, not the inputs, so this is the one place the two must agree.
fn shape_for(name: &str) -> (&'static [f64], &'static [f64]) {
    if name == "narrow_rows" {
        (&NXS, &NYS)
    } else {
        (&XS, &YS)
    }
}

/// The capture's `DIAG` cuts — diagonals that turn rectangular faces into
/// wedges with acute vertices. See `golden.rs`'s header for the three
/// mutations that survived until this existed.
const DIAG: [(f64, f64, f64, f64); 6] = [
    (300.0, 280.0, 760.0, 690.0),
    (760.0, 280.0, 1240.0, 690.0),
    (300.0, 690.0, 760.0, 280.0),
    (520.0, 690.0, 1010.0, 900.0),
    (1010.0, 280.0, 1240.0, 470.0),
    (300.0, 470.0, 520.0, 280.0),
];

/// Builds the scenario's graph: the grid, plus the diagonal cuts when the
/// scenario is the wedge one.
fn build_graph(name: &str) -> Graph {
    let mut g = Graph::new();
    let (xs, ys) = shape_for(name);
    grid(&mut g, xs, ys);
    if name == "wedges" {
        for (ax, ay, bx, by) in DIAG {
            g.add_street(ax, ay, bx, by, "lane", 4.0, 3, "fixture-diagonal");
        }
    }
    g
}

/// The capture's `grid()`, reproduced exactly — including the single shared
/// `k` counter, which is what makes the jitter table's consumption order (and
/// therefore every node position) depend on the loop nesting.
#[allow(clippy::needless_range_loop, reason = "indexed exactly as the JS capture's loops are; the two must stay comparable line for line")]
fn grid(g: &mut Graph, xs: &[f64], ys: &[f64]) {
    let mut k = 0usize;
    let mut next = || {
        let v = JIT[k % JIT.len()];
        k += 1;
        v
    };
    let mut pt: Vec<Vec<Vec2>> = Vec::new();
    for (j, &y) in ys.iter().enumerate() {
        let mut row = Vec::new();
        for (i, &x) in xs.iter().enumerate() {
            let px = x + if i > 0 && i < xs.len() - 1 {
                next()
            } else {
                0.0
            };
            let py = y + if j > 0 && j < ys.len() - 1 {
                next()
            } else {
                0.0
            };
            row.push(Vec2::new(px, py));
        }
        pt.push(row);
    }
    for j in 0..ys.len() {
        for i in 0..xs.len() - 1 {
            let (a, b) = (pt[j][i], pt[j][i + 1]);
            let (cls, w) = if j == 1 {
                ("primary", 12.0)
            } else {
                ("street", 7.0)
            };
            g.add_street(a.x, a.y, b.x, b.y, cls, w, 1, "fixture");
        }
    }
    for i in 0..xs.len() {
        for j in 0..ys.len() - 1 {
            let (a, b) = (pt[j][i], pt[j + 1][i]);
            let (cls, w) = if i == 1 {
                ("primary", 12.0)
            } else {
                ("lane", 5.0)
            };
            g.add_street(a.x, a.y, b.x, b.y, cls, w, 2, "fixture");
        }
    }
}

/// The exact 64 bits of a double, hex, as the capture writes them.
fn bits(x: f64) -> String {
    format!("{:016x}", x.to_bits())
}

/// A polygon as space-separated `x_bits,y_bits` pairs, for the whole-state
/// hash — see [`bits`].
fn dump_poly(ps: &[Vec2]) -> String {
    ps.iter()
        .map(|p| format!("{},{}", bits(p.x), bits(p.y)))
        .collect::<Vec<_>>()
        .join(" ")
}

/// A parcel as a `|`-joined field dump, for the whole-state hash — see
/// [`bits`] and [`dump_poly`].
fn dump_parcel(p: &Parcel) -> String {
    format!(
        "{}|{}|{}|{}|{}|{}|{}|{}",
        p.id,
        p.block,
        bits(p.frontage),
        bits(p.depth),
        bits(p.area),
        bits(p.age),
        p.edge_cls,
        dump_poly(&p.poly)
    )
}

#[test]
fn golden_blocks_and_parcels() {
    // Protects: the market anchor, the block/parcel counts, every listed
    // block's and parcel's fields and vertices, and the whole-state hash
    // over everything not individually listed — for all five scenarios.
    for sc in golden::SCENARIOS {
        let site = build_site(sc.seed, 1700.0, 1250.0, sc.kind, SiteOpts::default());
        let g = build_graph(sc.name);
        let anchors = place_anchors(sc.seed, &site);
        assert_eq!(
            (anchors.market.x, anchors.market.y),
            sc.market,
            "{}: the market anchor already diverges, so nothing below is meaningful",
            sc.name
        );

        let blocks = build_blocks(&g, None, &site);
        let parcels = build_parcels(sc.seed, &g, &blocks, anchors.market, 8, &site, None);

        // Shape first: a hash mismatch caused by a length difference is the
        // least informative failure this test could report.
        assert_eq!(blocks.len(), sc.block_count, "{}: block count", sc.name);
        assert_eq!(parcels.len(), sc.parcel_count, "{}: parcel count", sc.name);
        assert!(
            !blocks.is_empty() && !parcels.is_empty(),
            "{}: empty golden",
            sc.name
        );

        for (i, want) in sc.blocks.iter().enumerate() {
            let got = &blocks[i];
            assert_eq!(got.id, want.id, "{} block {i} id", sc.name);
            assert_eq!(got.plaza, want.plaza, "{} block {i} plaza", sc.name);
            assert_eq!(got.area, want.area, "{} block {i} area", sc.name);
            assert_eq!(
                got.poly.len(),
                want.poly.len(),
                "{} block {i} poly len",
                sc.name
            );
            for (j, p) in got.poly.iter().enumerate() {
                assert_eq!((p.x, p.y), want.poly[j], "{} block {i} vertex {j}", sc.name);
            }
        }
        for (i, want) in sc.parcels.iter().enumerate() {
            let got = &parcels[i];
            assert_eq!(got.id, want.id, "{} parcel {i} id", sc.name);
            assert_eq!(got.block, want.block, "{} parcel {i} block", sc.name);
            assert_eq!(
                got.frontage, want.frontage,
                "{} parcel {i} frontage",
                sc.name
            );
            assert_eq!(got.depth, want.depth, "{} parcel {i} depth", sc.name);
            assert_eq!(got.area, want.area, "{} parcel {i} area", sc.name);
            assert_eq!(got.age, want.age, "{} parcel {i} age", sc.name);
            assert_eq!(got.edge_cls, want.edge_cls, "{} parcel {i} class", sc.name);
            for (j, p) in got.poly.iter().enumerate() {
                assert_eq!(
                    (p.x, p.y),
                    want.poly[j],
                    "{} parcel {i} vertex {j}",
                    sc.name
                );
            }
        }

        let dump = blocks
            .iter()
            .map(|b| {
                format!(
                    "{}|{}|{}|{}|{}|{}|{}",
                    b.id,
                    bits(b.area),
                    u8::from(b.plaza),
                    dump_poly(&b.poly),
                    dump_poly(&b.face_poly),
                    b.face_ids
                        .iter()
                        .map(|i| i.to_string())
                        .collect::<Vec<_>>()
                        .join(","),
                    b.edge_dists
                        .iter()
                        .map(|d| bits(*d))
                        .collect::<Vec<_>>()
                        .join(",")
                )
            })
            .collect::<Vec<_>>()
            .join(";")
            + "#"
            + &parcels
                .iter()
                .map(dump_parcel)
                .collect::<Vec<_>>()
                .join(";");
        assert_eq!(fnv1a(&dump), sc.hash, "{}: full-state hash", sc.name);
    }
}

/// The `|A| < 140_000` m² face **ceiling**, pinned at its own boundary.
///
/// The five golden scenarios do not pin it: a mutation to either end of the
/// band survives them, because no face in any of them lands in the one-unit
/// window a mutation moves. This is the "just-below-a-boundary" fixture the
/// project's own rules ask for — a 600 m frame cut into four cells, sized so
/// the corner cell sits either side of the ceiling and nothing else changes.
///
/// **The 120 m² floor is deliberately not tested here, because it cannot be
/// reached.** `attach_point`'s `SNAP` is 11 m, so any two nodes closer than
/// that merge — and an ~11 m cell (the only shape with an area near 120 m²
/// that a rectangular cut can produce) collapses to nothing before
/// `extract_faces` ever sees it. Measured, not assumed: cutting the frame at
/// 10.95 m and at 10.98 m both yield a 4-node graph with one 360_000 m² face.
/// The floor is a guard against the degenerate slivers `split_edge` and
/// crossing-resolution can produce, not against anything a clean street lay
/// can make, so a mutation of it is invisible to realistic input. Milestone
/// 11's `lanePass` is the first stage that could produce such a sliver, and
/// is where this is worth revisiting.
#[test]
fn block_area_ceiling_boundary() {
    // Protects: the 140_000 m² face-area ceiling, pinned one unit either
    // side of the boundary.
    // The frame's side length, metres.
    const F: f64 = 600.0;
    let site = build_site(1, 1700.0, 1250.0, "plain", SiteOpts::default());
    // A vertical and a horizontal cut at `s` split the frame into four cells:
    // s², s(F-s), (F-s)s and (F-s)². Only the last is anywhere near the
    // ceiling, so the block count is exactly "is (F-s)² in band".
    let kept = |s: f64| {
        let mut g = Graph::new();
        for (ax, ay, bx, by) in [
            (0.0, 0.0, F, 0.0),
            (F, 0.0, F, F),
            (F, F, 0.0, F),
            (0.0, F, 0.0, 0.0),
            (s, 0.0, s, F),
            (0.0, s, F, s),
        ] {
            g.add_street(ax, ay, bx, by, "lane", 4.0, 1, "boundary");
        }
        build_blocks(&g, None, &site).len()
    };
    // 374² = 139_876, in band. 375² = 140_625, over it. Both sit inside the
    // window a one-unit mutation of the constant moves.
    assert_eq!(kept(374.0), 4, "139_876 m² was dropped -- the ceiling is too low");
    assert_eq!(kept(375.0), 3, "140_625 m² was kept -- the ceiling is not applied");
}

/// The river scenario must actually lose a block to the channel, or the
/// wet-face guard in [`build_blocks`] is never exercised and the scenarios are
/// testing one code path between them. Both fixtures share the wide grid, so
/// the block counts are directly comparable.
#[test]
fn river_site_rejects_a_flooded_block() {
    // Protects: build_blocks' wet-face guard actually drops a block to the
    // river, rather than the fixture coincidentally never exercising it.
    let by = |n| {
        golden::SCENARIOS
            .iter()
            .find(|s| s.name == n)
            .expect("scenario")
    };
    let (plain, river) = (by("plain_grid"), by("river_grid"));
    assert!(
        river.block_count < plain.block_count,
        "the river fixture keeps every block, so nothing tests the water rejection"
    );
}

/// `tone` is this port's addition and must be (a) stable across runs and
/// (b) genuinely spread — a constant would render every roof the same shade,
/// which is the whole thing this field exists to prevent.
#[test]
fn tone_is_deterministic_and_spread() {
    // Protects: `tone` is reproducible across runs, spread rather than
    // constant, in range, and varies between settlements.
    let sc = &golden::SCENARIOS[0];
    let site = build_site(sc.seed, 1700.0, 1250.0, sc.kind, SiteOpts::default());
    let g = build_graph(sc.name);
    let anchors = place_anchors(sc.seed, &site);
    let blocks = build_blocks(&g, None, &site);
    let a = build_parcels(sc.seed, &g, &blocks, anchors.market, 8, &site, None);
    let b = build_parcels(sc.seed, &g, &blocks, anchors.market, 8, &site, None);
    let tones: Vec<f64> = a.iter().map(|p| p.tone).collect();
    assert_eq!(
        tones,
        b.iter().map(|p| p.tone).collect::<Vec<_>>(),
        "tone is not reproducible"
    );
    assert!(
        tones.iter().all(|t| (0.0..1.0).contains(t)),
        "tone out of range"
    );
    let lo = tones.iter().copied().fold(f64::INFINITY, f64::min);
    let hi = tones.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    assert!(
        hi - lo > 0.9,
        "tone spans only {lo}..{hi} -- roofs would look uniform"
    );
    // And a different settlement must not get the identical sequence.
    let other = build_parcels(
        sc.seed ^ 0x9e37,
        &g,
        &blocks,
        anchors.market,
        8,
        &site,
        None,
    );
    assert_ne!(
        tones[0], other[0].tone,
        "tone does not vary between settlements"
    );
}

/// Mutation guards. Each constant below is one the reference states explicitly
/// and which a plausible typo would leave passing every other assertion in
/// this file, so each is checked by the effect it has rather than by reading
/// it back.
#[test]
fn block_area_band_is_enforced() {
    // Protects: every kept block's face area sits in the reference's
    // 120..140_000 m² band, and the outer (whole-graph) face is never kept.
    let sc = &golden::SCENARIOS[0];
    let site = build_site(sc.seed, 1700.0, 1250.0, sc.kind, SiteOpts::default());
    let g = build_graph(sc.name);
    let (xs, _ys) = shape_for(sc.name);
    let blocks = build_blocks(&g, None, &site);
    // Every kept block's *face* must sit inside the reference's 120 .. 140_000
    // band. The inset interior may be smaller; the band is applied to the face.
    for b in &blocks {
        let a = crate::geom::poly_area(&b.face_poly).abs();
        assert!(
            (120.0..=140_000.0).contains(&a),
            "block {} face area {a} outside the band",
            b.id
        );
    }
    // And the outer face is skipped: the union of kept faces cannot include the
    // whole-graph boundary, so no block may reach the grid's full extent.
    let span = xs[xs.len() - 1] - xs[0];
    for b in &blocks {
        let w = b
            .face_poly
            .iter()
            .map(|p| p.x)
            .fold(f64::NEG_INFINITY, f64::max)
            - b.face_poly
                .iter()
                .map(|p| p.x)
                .fold(f64::INFINITY, f64::min);
        assert!(
            w < span,
            "block {} spans the whole grid -- the outer face was kept",
            b.id
        );
    }
}

#[test]
fn parcels_conserve_block_area() {
    // Protects: the 0.97 area-conservation trim per block, and the 26..2600
    // m² lot-area band, over every scenario.
    // The reference's own invariant: sum(parcels) <= 0.97 * block area. It is
    // enforced by a trim loop, so a broken trim shows up here and nowhere else.
    for sc in golden::SCENARIOS {
        let site = build_site(sc.seed, 1700.0, 1250.0, sc.kind, SiteOpts::default());
        let g = build_graph(sc.name);
        let anchors = place_anchors(sc.seed, &site);
        let blocks = build_blocks(&g, None, &site);
        let parcels = build_parcels(sc.seed, &g, &blocks, anchors.market, 8, &site, None);
        for b in &blocks {
            let sum: f64 = parcels
                .iter()
                .filter(|p| p.block == b.id)
                .map(|p| p.area)
                .sum();
            let cap = crate::geom::poly_area(&b.poly).abs() * 0.97;
            assert!(
                sum <= cap,
                "block {}: parcels sum {sum} exceeds {cap}",
                b.id
            );
        }
        // Every parcel is within the reference's own 26 .. 2600 m² lot band.
        for p in &parcels {
            assert!(
                (26.0..=2600.0).contains(&p.area),
                "parcel {} area {} outside the band",
                p.id,
                p.area
            );
        }
    }
}

/// A NaN generation rule must not put a NaN into a lot's geometry.
///
/// `applyPlotChaos` writes a NaN slider straight into
/// `frontageWidthVariance`, `logn` returns NaN from it, and `js_min`/`js_max`
/// propagate NaN by design — so `acc` goes NaN and the reference's
/// `(eLen)/(acc||eLen)` takes the `eLen` branch, because **JS `||` is falsy
/// for NaN as well as zero**. A Rust `acc == 0.0` test alone misses that and
/// pushes the NaN into every corner of every lot on the frontage, which
/// renders as nothing at all rather than as an error.
///
/// The five golden scenarios cannot catch this: their rules are finite, so
/// the NaN branch is never taken. This is the only test that reaches it.
#[test]
fn a_nan_rule_does_not_reach_lot_geometry() {
    // Protects: a NaN generation-rule slider does not put a NaN into any
    // surviving lot's geometry or area, and a NaN subdivision cap runs the
    // burgage cycle zero times rather than panicking or looping.
    use crate::rules::DEFAULT_RULES;

    let sc = &golden::SCENARIOS[0];
    let site = build_site(sc.seed, 1700.0, 1250.0, sc.kind, SiteOpts::default());
    let g = build_graph(sc.name);
    let anchors = place_anchors(sc.seed, &site);
    let blocks = build_blocks(&g, None, &site);

    let mut rules = DEFAULT_RULES;
    rules.parcels.frontage_width_variance = f64::NAN;
    let parcels = build_parcels(sc.seed, &g, &blocks, anchors.market, 8, &site, Some(&rules));

    // Whatever survives the area and self-intersection filters must be finite
    // geometry. (A NaN area fails `26.0..=2600.0`, so most lots are rejected
    // outright -- the point is that none of the survivors carries a NaN.)
    for p in &parcels {
        for (i, v) in p.poly.iter().enumerate() {
            assert!(
                v.x.is_finite() && v.y.is_finite(),
                "parcel {} vertex {i} is {:?} under a NaN frontage rule",
                p.id,
                (v.x, v.y)
            );
        }
        assert!(p.area.is_finite(), "parcel {} has a non-finite area", p.id);
    }

    // And the NaN subdivision cap the rules doc comment describes must simply
    // run the burgage cycle zero times rather than panicking or looping.
    let mut cap_nan = DEFAULT_RULES;
    cap_nan.parcels.subdivision_cap = f64::NAN;
    let capped = build_parcels(sc.seed, &g, &blocks, anchors.market, 8, &site, Some(&cap_nan));
    assert!(!capped.is_empty(), "a NaN subdivision cap produced no lots at all");
}

/// `PARCEL_GRANT_MAX_SPIN` (§6r.5). Variance 0.10 is the "Planned Grid" chip's
/// value, and this exact town (seed 2, river, pop 4000) did not return in 100 s
/// before the bound. A regression hangs here rather than failing, which is
/// still a failure: the suite does not finish.
#[test]
fn the_planned_grid_variance_terminates() {
    // Protects: PARCEL_GRANT_MAX_SPIN's bound actually terminates a town
    // that previously hung, within the test suite's own time budget.
    use crate::generate::{GenOpts, generate};
    use crate::rules::{ParcelPatch, RulesPatch};
    let opts = GenOpts {
        pop: Some(4000.0),
        site: Some("river".to_string()),
        rules: Some(RulesPatch {
            parcels: Some(ParcelPatch {
                frontage_width_variance: Some(0.10),
                ..ParcelPatch::default()
            }),
            ..RulesPatch::default()
        }),
        ..GenOpts::default()
    };
    let t = generate(2, &opts);
    // 899 until 2026-09-23, when `crate::courtyard` (Ruling AD) re-platted
    // this town's outermost dense blocks as perimeter blocks: +10 lots; 909
    // until the same day's faubourg grain pass (`crate::wallside`: ragged row
    // ends, per-lot back lines) re-drew its rows behind the first: +13.
    assert_eq!(t.parcels.len(), 922, "the bounded town's parcel count");
}

/// Ruling CF item 7's measurement harness: how many final parcels of real
/// generated towns touch water, by four footprint tests.
///
/// **What it does.** Generates a 400-town matrix (both cultures × the five
/// site kinds × fortified or not × seeds 1-20, everything else at the
/// reference default) and scores every final parcel by: the reference's four
/// corners; the source's v2.67 seven points (corners, centroid, two
/// depth-side midpoints, `RC_ENGINE_CHANGES.md` §6s.4); this port's nine
/// ([`crate::blocks::lot_is_wet`]); and a dense 9 × 9 bilinear sample of the
/// quad, used as the oracle for "the footprint overlaps water". Prints totals
/// per platting path (`par` = `build_parcels`, `cyd` = courtyard ring,
/// `wallin`/`faub` = the wall-side passes) and, with `CF7_OUT` set, one line
/// per town to that file so a before/after pair can be diffed.
///
/// **Why it exists.** The owner's ruling asked for the choice of test to be
/// justified by a measured comparison on real generated settlements. It is
/// `#[ignore]`d because it is a measurement, not a regression test; run it
/// with `cargo test -p cartalith-urban measure_parcel_water -- --ignored
/// --nocapture`.
///
/// **What it must never do.** Assert: its numbers are evidence for a
/// decision record, and the decision is pinned by the tests that follow it.
#[test]
#[ignore = "measurement harness for Ruling CF item 7, not a regression test"]
fn measure_parcel_water_footprint_tests() {
    use crate::blocks::{lot_is_wet, wet_margin};
    use crate::geom::poly_centroid;
    use crate::generate::{GenOpts, SITE_HM, SITE_WM, generate};
    use std::collections::BTreeMap;
    use std::fmt::Write as _;

    let mut per_path: BTreeMap<&str, [usize; 6]> = BTreeMap::new();
    let mut lines = String::new();
    for culture in ["medieval", "venus"] {
        for kind in ["river", "riverthrough", "coast", "estuary", "inland"] {
            for fortified in [false, true] {
                for seed in 1u32..=20 {
                    let opts = GenOpts {
                        culture: Some(culture.to_string()),
                        site: Some(kind.to_string()),
                        fortified,
                        ..GenOpts::default()
                    };
                    let t = generate(seed, &opts);
                    let site = build_site(seed, SITE_WM, SITE_HM, kind, SiteOpts::default());
                    let margin = wet_margin(&site);
                    let wet = |q: Vec2| site.is_water(q) || site.river_dist(q) < margin;
                    let (mut dense_n, mut built, mut area) = (0usize, 0usize, 0.0f64);
                    for tp in &t.parcels {
                        let p = &tp.par.poly;
                        let path = ["par", "cyd", "wallin", "faub"]
                            .into_iter()
                            .find(|k| tp.par.id.starts_with(k))
                            .unwrap_or("other");
                        let e = per_path.entry(path).or_default();
                        e[0] += 1;
                        if tp.built {
                            built += 1;
                        }
                        area += tp.par.area;
                        if p.len() != 4 {
                            e[5] += 1;
                            continue;
                        }
                        let corners = p.iter().any(|q| wet(*q));
                        let c = poly_centroid(p);
                        let seven = corners
                            || wet(c)
                            || wet(p[0].lerp(p[3], 0.5))
                            || wet(p[1].lerp(p[2], 0.5));
                        let nine = lot_is_wet(&site, p);
                        let mut dense = false;
                        for i in 0..=8 {
                            for j in 0..=8 {
                                let (u, v) = (i as f64 / 8.0, j as f64 / 8.0);
                                let q = p[0].lerp(p[1], u).lerp(p[3].lerp(p[2], u), v);
                                dense |= wet(q);
                            }
                        }
                        e[1] += usize::from(corners);
                        e[2] += usize::from(seven);
                        e[3] += usize::from(nine);
                        e[4] += usize::from(dense);
                        dense_n += usize::from(dense);
                    }
                    let _ = writeln!(
                        lines,
                        "{culture} {kind} fort={fortified} seed={seed} parcels={} built={built} area={area:.1} dense_wet={dense_n} hash={}",
                        t.parcels.len(),
                        crate::generate::hash_model(&t)
                    );
                }
            }
        }
    }
    println!("path: [parcels, corner-wet, seven-wet, nine-wet, dense-wet, non-quad]");
    for (k, v) in &per_path {
        println!("{k}: {v:?}");
    }
    if let Ok(path) = std::env::var("CF7_OUT") {
        std::fs::write(path, lines).expect("CF7_OUT is writable");
    }
}

/// A lot laid straight across a synthetic river channel: every corner on dry
/// bank, its middle in the water. Returns the site and the quad, wound like
/// `build_parcels`' own (`[front0, front1, back1, back0]`).
fn lot_across_the_channel() -> (crate::site::Site, Vec<Vec2>) {
    let site = build_site(4242, 1700.0, 1250.0, "river", SiteOpts::default());
    // The middle segment of the centreline, so the lot is far from its ends.
    let i = site.river.len() / 2;
    let (a, b) = (site.river[i], site.river[i + 1]);
    let m = a.lerp(b, 0.5);
    let d = (b - a).norm();
    let nrm = Vec2::new(-d.y, d.x);
    // Corners 6 m beyond the wider of the two wet bands (`isWater`'s
    // `riverW/2 + 2`, the margin's `riverW/2 + 1`); 3 m either side along it.
    let off = site.river_w / 2.0 + 2.0 + 6.0;
    let quad = vec![
        m - nrm * off - d * 3.0,
        m - nrm * off + d * 3.0,
        m + nrm * off + d * 3.0,
        m + nrm * off - d * 3.0,
    ];
    (site, quad)
}

/// Ruling CF item 7: a lot spanning a narrow channel with every corner dry is
/// wet. This is the case `RC_ENGINE_CHANGES.md` §6s.4 names, and the one the
/// reference's corner-only test accepted.
#[test]
fn a_lot_spanning_the_channel_with_dry_corners_is_wet() {
    // Protects: `lot_is_wet` samples the footprint's interior, not only its
    // corners. Reverting it to the four corners turns this red.
    use crate::blocks::{lot_is_wet, wet_margin};
    let (site, quad) = lot_across_the_channel();
    let margin = wet_margin(&site);
    // The fixture's premise, checked rather than assumed: every corner is dry
    // by the reference's own test, and the lot's middle is in the river.
    for q in &quad {
        assert!(!site.is_water(*q) && site.river_dist(*q) >= margin, "corner {q:?} must be dry");
    }
    let mid = quad[0].lerp(quad[2], 0.5);
    assert!(site.is_water(mid), "the lot's middle must be in the channel");
    assert!(lot_is_wet(&site, &quad), "a lot across the channel is wet");
}

/// The nine samples are a superset of the corners: a lot the corner test
/// rejected is still rejected, and a lot well clear of the water is dry.
#[test]
fn lot_is_wet_rejects_a_wet_corner_and_accepts_dry_ground() {
    // Protects: the fix only ever rejects MORE lots (it must never readmit one
    // with a wet corner), and it does not reject everything.
    use crate::blocks::lot_is_wet;
    let (site, quad) = lot_across_the_channel();
    // Shift the whole lot so its back edge sits on the centreline: two corners wet.
    let back = (quad[2] - quad[1]) * 0.5;
    let wet_corner: Vec<Vec2> = quad.iter().map(|q| *q - back).collect();
    assert!(site.is_water(wet_corner[2]), "premise: a back corner is in the channel");
    assert!(lot_is_wet(&site, &wet_corner));
    // The same lot moved 200 m clear of the channel, along the normal.
    let away = (quad[2] - quad[1]).norm() * 200.0;
    let dry: Vec<Vec2> = quad.iter().map(|q| *q + away).collect();
    assert!(!lot_is_wet(&site, &dry), "a lot 200 m from the channel is dry");
}

/// Each of the two new kinds of sample decides on its own: a pond under only
/// the lot's middle, and one under only an edge's midpoint.
#[test]
fn lot_is_wet_reads_the_middle_and_every_edge_midpoint() {
    // Protects: both the vertex-mean sample and the edge-midpoint samples of
    // `lot_is_wet`; dropping either turns one of the cases red (the channel
    // test above cannot tell them apart -- a channel across the lot wets both).
    use crate::blocks::lot_is_wet;
    // A 3 x 3 real-water mask of 10 m cells with at most one wet cell, every
    // cell far from any channel centreline (`dt` 100 cells), on a coast so the
    // margin is the flat 3 m. The lot is the 30 m square over the whole mask.
    let site_wet_at = |cell: Option<usize>| {
        let mut mask = vec![0u8; 9];
        if let Some(c) = cell {
            mask[c] = 1;
        }
        let w = crate::site::WaterCtx {
            mask,
            dt: vec![100.0; 9],
            mw: 3,
            mh: 3,
            cell_m: 10.0,
            river_path: None,
            river_width_m: Some(10.0),
            river_order: 0.0,
            sea_lake_cells: 0.0,
        };
        build_site(3, 1700.0, 1250.0, "coast", SiteOpts { water: Some(w), ..Default::default() })
    };
    let lot = [Vec2::new(0.0, 0.0), Vec2::new(30.0, 0.0), Vec2::new(30.0, 30.0), Vec2::new(0.0, 30.0)];
    // Control first: no wet cell, a dry lot.
    assert!(!lot_is_wet(&site_wet_at(None), &lot), "an all-dry mask leaves the lot dry");
    // Corners land in cells 0, 2, 8, 6 (the far edge clamps into the mask).
    for c in [0usize, 2, 6, 8] {
        assert!(lot_is_wet(&site_wet_at(Some(c)), &lot), "a wet corner (cell {c})");
    }
    // Only the middle sample lands in cell 4.
    assert!(lot_is_wet(&site_wet_at(Some(4)), &lot), "a pond under the middle only");
    // Only one edge midpoint lands in each of cells 1, 3, 5, 7.
    for c in [1usize, 3, 5, 7] {
        assert!(lot_is_wet(&site_wet_at(Some(c)), &lot), "a pond under one edge midpoint (cell {c})");
    }
}
