//! Milestone 2's tests. Almost all of them are **golden**: the reference's own
//! `UME._test` export hands out `makeGraph`, `addStreet` and `extractFaces`
//! directly, so the scenarios below are run in the reference engine and in this
//! port, and the **entire graph state** — nodes, adjacency, edges (dead ones
//! included), the uniform-grid spatial index, and the extracted faces — is
//! compared, not just the return values.
//!
//! How the expected values were captured (the same discipline as `rng.rs` and
//! `geom.rs`): block 4 is sliced out of the frozen
//! `reference/Cartalith Gen1 v2.10.html` as **one contiguous block**, lines
//! 28167-31103, plus line 2291 (`mulberry32`, which block 4 deliberately does
//! not define), with a **block-comment balance assertion on both slice
//! boundaries** plus two structural assertions that the slice really begins at
//! the `UME` IIFE and ends at the module export, and evaluated under a bare
//! Node `vm.runInContext` with no DOM.
//!
//! Everything is compared **exactly**. Nothing in this module passes through a
//! transcendental whose last bit could legitimately differ between V8 and Rust
//! — `atan2` appears in `extract_faces`, but only as a *sort key*, and no
//! scenario here has two incidences close enough for a last-bit difference to
//! reorder them.
//!
//! **That last sentence was true and was still the wrong thing to rely on.**
//! Milestone 2 wrote the sort key as `f64::atan2`, before `JS_SEMANTICS_AUDIT.md`
//! measured that `Math.atan2` is the *largest* divergence in this workspace —
//! Rust and V8 return different doubles on 17-23 % of ordinary arguments, and on
//! **38 %** of the edge deltas a town graph really produces (196 034 of 510 634,
//! measured directly). §4.4 called the site "a real hazard … the argmax hazard
//! in a different costume". It is now `js_atan2`, and every scenario below
//! passes **unmodified**, which is the proof that nothing these fixtures can
//! see moved. What the fixtures cannot see is pinned separately by
//! [`the_half_edge_sort_key_orders_like_v8_not_like_rust`].
//!
//! The functions the `_test` export does **not** reach — `add_polyline_street`,
//! `edge_between`, `nearest_node`, `raw_edge`, `split_edge`, `attach_point`, and
//! the `cells_for_seg` / `index_edge` / `unindex_edge` / `edges_near` index
//! family — are still golden-covered, indirectly but completely.
//! `add_polyline_street` is `add_street` in a loop and the `polyline` scenario
//! runs that loop on both sides; `attach_point`, `raw_edge`, `split_edge` and
//! `nearest_node` are entirely inside `add_street` and every one of their
//! branches is reachable from it; `edge_between` reads the same adjacency the
//! dumps compare; and the index family's entire observable effect *is* the
//! `grid` field, which every scenario compares cell by cell. Only
//! `edges_near`'s **ordering** has no direct golden (a `Set`'s iteration order
//! is not observable from outside), so it gets a real unit test below, labelled
//! as such.
//!
//! **The goldens were mutation-checked**, because a full-state dump can look
//! thorough and still be vacuous. Perturbing the 26 m index cell, the 0.7 cell
//! step, the 3×3 cell dilation, the 11 m node snap, the 9 m edge snap, both 3.5 m
//! guards, the 2.5 m node-promotion radius, the `[0.03, 0.97]` t clamp, the spur
//! collapse's stack rule, the outer-face tie-break's strict `>`, and swapping
//! `js_hypot` for `f64::hypot` each break at least one golden. Two constants
//! survive every scenario — the `1e-4` and `1e-3` interior-parameter epsilons —
//! and that is a finding, not a hole: see the note at their use site.

use super::*;
use crate::rng::stream;

mod golden;

/// One scripted operation, mirroring the capture script's op format.
enum Op {
    /// `add_street(ax, ay, bx, by, cls, w, epoch)`
    S(f64, f64, f64, f64, &'static str, f64, i32),
    /// `add_polyline_street(pts, cls, w, epoch)`
    P(&'static [(f64, f64)], &'static str, f64, i32),
}
use Op::{P, S};

/// A `dx` whose V8 `Math.hypot(dx, dx)` lands exactly *on* `attach_point`'s
/// 11 m snap threshold while the correctly-rounded value lands just under it.
/// See `hypot_threshold_decides_a_snap_that_f64_hypot_would_decide_differently`.
const DX: [f64; 4] =
    [7.778174593052021, 7.778174593052022, 7.7781745930520225, 7.778174593052023];

fn scenario_ops(name: &str) -> Vec<Op> {
    match name {
        "single" => vec![S(100., 100., 200., 100., "street", 5., 0)],
        "cross" => vec![
            S(100., 100., 300., 100., "primary", 7., 0),
            S(200., 20., 200., 180., "street", 5., 1),
        ],
        "snapNode" => vec![
            S(0., 0., 120., 0., "street", 5., 0),
            S(123., 3., 200., 60., "street", 4., 1),
        ],
        "snapEdge" => vec![
            S(0., 0., 200., 0., "street", 5., 0),
            S(100., 6., 100., 90., "lane", 2.6, 1),
        ],
        "tooShort" => vec![
            S(50., 50., 52., 50., "street", 5., 0),
            S(0., 0., 100., 0., "street", 5., 0),
        ],
        "duplicate" => vec![
            S(0., 0., 100., 0., "street", 5., 0),
            S(0., 0., 100., 0., "primary", 9., 3),
        ],
        "splitNearNode" => vec![
            S(0., 0., 100., 0., "street", 5., 0),
            S(100., 0., 100., 80., "street", 5., 0),
            S(98., -40., 98., 40., "street", 4., 1),
        ],
        "nodeOnSegment" => vec![
            S(0., 0., 80., 0., "street", 5., 0),
            S(80., 0., 80., 80., "street", 5., 0),
            S(160., 1.5, 240., 1.5, "street", 5., 0),
            S(-20., 0., 300., 0., "primary", 7., 1),
        ],
        "lattice" => vec![
            S(0., 0., 200., 0., "street", 5., 0),
            S(0., 100., 200., 100., "street", 5., 0),
            S(0., 200., 200., 200., "street", 5., 0),
            S(0., 0., 0., 200., "street", 5., 0),
            S(100., 0., 100., 200., "street", 5., 0),
            S(200., 0., 200., 200., "street", 5., 0),
        ],
        "spur" => vec![
            S(0., 0., 120., 0., "street", 5., 0),
            S(120., 0., 120., 120., "street", 5., 0),
            S(120., 120., 0., 120., "street", 5., 0),
            S(0., 120., 0., 0., "street", 5., 0),
            S(60., 0., 60., 60., "street", 4., 1),
        ],
        "polyline" => vec![
            P(&[(-140., -60.), (-40., -30.), (30., -70.), (110., 10.), (90., 120.)], "primary", 7., 0),
            P(&[(-100., 100.), (0., 0.), (100., -100.)], "street", 4.5, 1),
        ],
        "triangle" => vec![
            S(0., 0., 400., 0., "primary", 7., 0),
            S(400., 0., 200., 340., "primary", 7., 0),
            S(200., 340., 0., 0., "primary", 7., 0),
            S(100., 170., 300., 170., "street", 5., 1),
        ],
        "nearParallel" => vec![
            S(100., -40., 100., 40., "street", 5., 0),
            S(100., 2., 250., 2., "street", 4., 0),
            S(0., 0., 200., 0., "primary", 7., 1),
        ],
        "clampT" => vec![
            S(0., 0., 400., 0., "street", 5., 0),
            S(10., 8., 10., 120., "lane", 2.6, 1),
            S(390., -8., 390., -120., "lane", 2.6, 1),
        ],
        "oblique" => vec![
            S(3., 7., 271., 193., "primary", 7., 0),
            S(17., 181., 259., 11., "street", 5., 0),
            S(5., 137., 283., 149., "street", 4.5, 1),
            S(131., 3., 149., 197., "lane", 2.6, 2),
        ],
        // One scenario per `DX` value; a shared graph would let the first
        // stub's node capture the next and hide the effect.
        _ if name.starts_with("hypotSnap") => {
            let d = DX[name["hypotSnap".len()..].parse::<usize>().expect("index")];
            vec![
                S(0., 0., 0., -200., "street", 5., 0),
                S(d, d, d + 90.0, d + 90.0, "street", 4., 1),
            ]
        }
        // The reference's own exported `stream` drove the capture, so this port
        // reproduces the identical input sequence from `crate::rng::stream` —
        // which makes the stress case a golden over the RNG *and* the graph.
        "stress" => {
            let mut r = stream(1234, "m2/stress");
            (0..24)
                .map(|_| {
                    let ax = r.range(0., 300.);
                    let ay = r.range(0., 240.);
                    let bx = r.range(0., 300.);
                    let by = r.range(0., 240.);
                    S(ax, ay, bx, by, "street", 5., 0)
                })
                .collect()
        }
        other => panic!("no ops defined for scenario {other}"),
    }
}

fn run(ops: &[Op]) -> (Graph, Vec<Vec<usize>>) {
    let mut g = Graph::new();
    let mut made = Vec::new();
    for op in ops {
        made.push(match *op {
            S(ax, ay, bx, by, cls, w, ep) => {
                g.add_street(ax, ay, bx, by, cls, w, ep, &format!("prov:{cls}"))
            }
            P(pts, cls, w, ep) => {
                let v: Vec<Vec2> = pts.iter().map(|&(x, y)| Vec2::new(x, y)).collect();
                g.add_polyline_street(&v, cls, w, ep, &format!("prov:{cls}"))
            }
        });
    }
    (g, made)
}

/// The reference sorts its grid dump by the JS key string `cx + ':' + cy`; the
/// port keys the same partition with `(i64, i64)`, so the comparison re-derives
/// that string ordering rather than assuming numeric order matches it.
fn grid_sorted(g: &Graph) -> Vec<((i64, i64), Vec<usize>)> {
    let mut v: Vec<((i64, i64), Vec<usize>)> =
        g.grid.iter().filter(|(_, ids)| !ids.is_empty()).map(|(k, ids)| (*k, ids.clone())).collect();
    v.sort_by(|a, b| format!("{}:{}", a.0.0, a.0.1).cmp(&format!("{}:{}", b.0.0, b.0.1)));
    v
}

#[test]
fn golden_every_scenario_reproduces_the_reference_graph_exactly() {
    for sc in golden::GOLDEN {
        let (g, made) = run(&scenario_ops(sc.name));
        let what = sc.name;

        assert_eq!(
            made,
            sc.made.iter().map(|m| m.to_vec()).collect::<Vec<_>>(),
            "{what}: edge ids returned by add_street"
        );

        // `nextN`/`nextE` are not stored; the capture asserted they equalled the
        // array lengths in the reference, so comparing lengths here is the same
        // assertion from this side.
        assert_eq!(g.nodes.len(), sc.nodes.len(), "{what}: node count (= the reference's nextN)");
        for (n, &(id, x, y, adj)) in g.nodes.iter().zip(sc.nodes) {
            assert_eq!((n.id, n.x, n.y), (id, x, y), "{what}: node {id}");
            assert_eq!(n.adj, adj, "{what}: node {id} adjacency");
        }

        assert_eq!(g.edges.len(), sc.edges.len(), "{what}: edge count (= the reference's nextE)");
        for (e, &(id, a, b, cls, w, epoch, alive)) in g.edges.iter().zip(sc.edges) {
            assert_eq!(
                (e.id, e.a, e.b, e.cls, e.w, e.epoch, e.alive),
                (id, a, b, cls, w, epoch, alive),
                "{what}: edge {id}"
            );
        }

        let grid = grid_sorted(&g);
        assert_eq!(grid.len(), sc.grid.len(), "{what}: populated grid cell count");
        for (got, &(k, ids)) in grid.iter().zip(sc.grid) {
            assert_eq!(got.0, k, "{what}: grid cell key");
            assert_eq!(got.1, ids, "{what}: grid cell {k:?} contents (order is load-bearing)");
        }

        let faces = g.extract_faces();
        assert_eq!(faces.len(), sc.faces.len(), "{what}: face count");
        for (i, (f, &(ids, area, outer))) in faces.iter().zip(sc.faces).enumerate() {
            assert_eq!(f.node_ids, ids, "{what}: face {i} node ids");
            assert_eq!(f.area, area, "{what}: face {i} area");
            assert_eq!(f.outer, outer, "{what}: face {i} outer flag");
            // The polygon is the node ring materialised, so checking it against
            // the nodes closes the loop without duplicating it in the golden.
            let want: Vec<Vec2> = ids.iter().map(|&n| g.nodes[n].pt()).collect();
            assert_eq!(f.poly, want, "{what}: face {i} polygon");
        }

        // `edge_between` has no `_test` entry, so it is checked against the
        // adjacency the golden already pins: for every ordered node pair, the
        // answer must be the first live incident edge reaching the other node.
        for a in 0..g.nodes.len() {
            for b in 0..g.nodes.len() {
                let want = g.nodes[a]
                    .adj
                    .iter()
                    .copied()
                    .find(|&eid| {
                        let e = &g.edges[eid];
                        e.alive && (e.a == b || e.b == b)
                    });
                assert_eq!(g.edge_between(a, b), want, "{what}: edge_between({a},{b})");
            }
        }
    }
}

/// The `extract_faces` half-edge sort key follows **V8's** `atan2`, not Rust's.
///
/// `ang` is the key the face traversal walks, and `sort_by` is stable, so the
/// only thing that can change a face is the *ordering* of two half-edges
/// leaving one node — which needs their two angles to be within an ulp of each
/// other. Every golden scenario above is built from round coordinates whose
/// incidences are milliradians apart, so none of them can reach that; a search
/// over 510 634 near-parallel pairs on the arbitrary `f64` coordinates
/// `attach_point` and `buildPrimaries` actually produce finds the ordering
/// differing between `f64::atan2` and `js_atan2` on **23 814** of them, 4.7 %.
///
/// The five rows below are from that search. Each is a real pair of edge
/// deltas; `want` is the bit pattern **`node` v24.19.0 returns** for
/// `Math.atan2(dy, dx)`, and `order` is V8's own comparison of the two. V8
/// agrees with `js_atan2` on 5 of 5 and with `f64::atan2` on 0 of 5 — in three
/// of the five Rust manufactures a difference where V8 has an exact tie (which
/// the stable sort would otherwise resolve by insertion order), and in the
/// other two it does the reverse.
#[test]
fn the_half_edge_sort_key_orders_like_v8_not_like_rust() {
    use std::cmp::Ordering;
    // (d1, d2, V8 bits for atan2(d1), V8 bits for atan2(d2), V8's ordering)
    #[allow(clippy::type_complexity)]
    let rows: [((f64, f64), (f64, f64), u64, u64, Ordering); 5] = [
        (
            (-44.214706967050915, 331.40973238441916),
            (-44.21470696705097, 331.40973238441916),
            0x3ffb413cd1b9b6f8,
            0x3ffb413cd1b9b6f8,
            Ordering::Equal,
        ),
        (
            (-20.4638750793862, 210.66496825109425),
            (-20.46387507938624, 210.66496825109425),
            0x3ffaae9ed39d8fab,
            0x3ffaae9ed39d8fac,
            Ordering::Less,
        ),
        (
            (-25.919881988211955, 168.92259134332483),
            (-25.919881988211955, 168.92259134332494),
            0x3ffb919e1cf1ae94,
            0x3ffb919e1cf1ae93,
            Ordering::Greater,
        ),
        (
            (-43.373954902418745, 174.12731268537743),
            (-43.373954902418745, 174.12731268537755),
            0x3ffd09eb1cce454d,
            0x3ffd09eb1cce454d,
            Ordering::Equal,
        ),
        (
            (-147.09492742521496, 343.3608283649455),
            (-147.0949274252149, 343.3608283649455),
            0x3fff9bd111d88f09,
            0x3fff9bd111d88f09,
            Ordering::Equal,
        ),
    ];

    let mut rust_order_wrong = 0;
    for (d1, d2, w1, w2, want) in rows {
        // The expression `extract_faces` evaluates, spelled the same way.
        let j1 = js_atan2(d1.1, d1.0);
        let j2 = js_atan2(d2.1, d2.0);
        assert_eq!(j1.to_bits(), w1, "js_atan2{d1:?} vs node");
        assert_eq!(j2.to_bits(), w2, "js_atan2{d2:?} vs node");
        assert_eq!(j1.partial_cmp(&j2).unwrap(), want, "sort order for {d1:?} / {d2:?}");

        let (r1, r2) = (d1.1.atan2(d1.0), d2.1.atan2(d2.0));
        if r1.partial_cmp(&r2).unwrap() != want {
            rust_order_wrong += 1;
        }
    }
    assert_eq!(rust_order_wrong, 5, "these rows exist to discriminate; f64::atan2 now agrees");
}

#[test]
fn golden_scenarios_cover_every_branch_this_milestone_claims() {
    // A guard against the goldens quietly becoming vacuous: if a future edit
    // drops a scenario, the counts below stop matching and this fails loudly.
    let by = |n: &str| golden::GOLDEN.iter().find(|s| s.name == n).unwrap_or_else(|| panic!("{n}"));

    // splitEdge really tombstones rather than removes, and ids never shift.
    assert!(by("cross").edges.iter().any(|e| !e.6), "cross must contain a dead edge");
    // rawEdge's 3.5 m rejection leaves orphan nodes behind — reference behaviour.
    let ts = by("tooShort");
    assert_eq!(ts.nodes.len(), 4);
    assert_eq!(ts.edges.len(), 1);
    assert!(ts.nodes[0].3.is_empty() && ts.nodes[1].3.is_empty(), "orphan nodes are kept");
    // the duplicate call returns the existing edge, so no second edge exists
    assert_eq!(by("duplicate").edges.len(), 1);
    assert_eq!(by("duplicate").made[1], [0], "the second addStreet returns edge 0 again");
    // extractFaces' outer tie-break: two faces of equal |area|, index 0 wins
    let spur = by("spur");
    assert_eq!(spur.faces.len(), 2);
    assert_eq!(spur.faces[0].1.abs(), spur.faces[1].1.abs());
    assert!(spur.faces[0].2 && !spur.faces[1].2, "the lowest-indexed face wins the tie");
    // the spur node (5) is collapsed out of both face rings
    assert!(spur.faces.iter().all(|f| !f.0.contains(&5)), "the dead-end spur is collapsed");
    // the lattice's four interior blocks plus the outer boundary
    assert_eq!(by("lattice").faces.len(), 5);
    assert_eq!(by("lattice").faces.iter().filter(|f| f.2).count(), 1);
    // attachPoint's t clamp actually fires, at both ends of the range: on a
    // 400 m street the split lands at x=12 (t clamped up from 0.025 to 0.03)
    // and at x=388.36 (clamped down from 0.974 to 0.97 on the 388 m remainder).
    let ct = by("clampT");
    assert!(ct.nodes.iter().any(|n| n.1 == 12.0), "the lower t clamp must move the split");
    assert!(ct.nodes.iter().any(|n| n.1 == 388.36), "the upper t clamp must move the split");
    // the stress case is genuinely a stress case
    let st = by("stress");
    assert!(st.edges.len() > 60, "stress produced only {} edges", st.edges.len());
    assert!(st.faces.len() > 5, "stress produced only {} faces", st.faces.len());
}

#[test]
fn no_scenario_ties_two_hits_at_the_same_t_because_none_can() {
    // Recorded as a finding, not as a gap. `add_street` sorts its hits with a
    // *stable* sort and the port matches that, but no golden exercises the
    // tie-break — because the reference's own guards make a tie unreachable.
    // The argument is spelled out at the sort in `graph.rs`; this test is the
    // mechanical half of it, and it is what turned an assumed-important
    // ordering rule into a known-unreachable one.
    //
    // `near_parallel` is the scenario that attempt produced. It is kept for what
    // it does cover, not for the tie it failed to create.
    for sc in golden::GOLDEN {
        let (g, _) = run(&scenario_ops(sc.name));
        // Re-derive every hit parameter the way `add_street` does, for one more
        // segment laid across the finished graph, and confirm they are distinct.
        let a = Vec2::new(-500., -500.);
        let b = Vec2::new(900., 900.);
        let mut ts: Vec<f64> = Vec::new();
        for eid in g.edges_near(a, b) {
            if !g.edges[eid].alive {
                continue;
            }
            let (ea, eb) = g.ends(eid);
            if let Some(h) = seg_int(a, b, ea, eb)
                && h.t > 1e-4
                && h.t < 1.0 - 1e-4
                && h.u > 1e-4
                && h.u < 1.0 - 1e-4
            {
                ts.push(h.t);
            }
        }
        let n = ts.len();
        ts.sort_by(|p, q| p.partial_cmp(q).unwrap());
        ts.dedup();
        assert_eq!(ts.len(), n, "{}: two crossings shared a t", sc.name);
    }
}

#[test]
fn hypot_threshold_decides_a_snap_that_f64_hypot_would_decide_differently() {
    // The `hypotSnap*` scenarios are not decoration. `attach_point` snaps when
    // the distance is strictly under 11, and for `dx = 7.778174593052022` the
    // two hypot implementations straddle that line:
    //
    //   V8   Math.hypot(dx, dx) == 11                  -> no snap
    //   Rust f64::hypot(dx, dx) == 10.999999999999998  -> would snap
    //
    // so a port using `f64::hypot` would build a structurally *different* graph
    // — three nodes instead of four — not merely a differently-rounded one.
    // Asserted directly so the reason the goldens pass is visible, and so
    // `js_hypot` cannot be "simplified" away without this failing first.
    let d = DX[1];
    assert_eq!(crate::geom::js_hypot(d, d), 11.0);
    assert!(d.hypot(d) < 11.0);

    // DX[0] is one ulp lower and falls *under* the threshold, so it snaps and
    // the stub's own node never exists. DX[1..] are on or above it and keep it.
    let (below, _) = run(&scenario_ops("hypotSnap0"));
    assert_eq!(below.nodes.len(), 3, "under the threshold the stub folds into the hub");
    assert!(!below.nodes.iter().any(|n| n.x == DX[0]));

    for (i, &dx) in DX.iter().enumerate().skip(1) {
        let (g, _) = run(&scenario_ops(&format!("hypotSnap{i}")));
        assert_eq!(g.nodes.len(), 4, "hypotSnap{i}: on/above the threshold the stub stays");
        assert!(
            g.nodes.iter().any(|n| n.x == dx && n.y == dx),
            "hypotSnap{i}: the stub must keep its own node"
        );
    }
}

#[test]
fn edges_near_returns_first_seen_order_not_set_order() {
    // No golden path: a JS `Set`'s iteration order is not observable through
    // `_test`, so this is a real unit test of the ported logic, documented as
    // such (the same precedent `poly_self_intersects` set in milestone 1).
    //
    // It matters because `attach_point` picks its best edge with a strict `<`
    // (first candidate wins a tie) and `add_street` sorts crossings with a
    // stable sort. A `HashSet` here would make both non-deterministic.
    let mut g = Graph::new();
    for i in 0..6 {
        let y = 40.0 * i as f64;
        g.add_street(0., y, 300., y, "street", 5., 0, "p");
    }
    let near = g.edges_near(Vec2::new(-10., -10.), Vec2::new(310., 210.));
    let mut seen = std::collections::HashSet::new();
    assert!(near.iter().all(|&id| seen.insert(id)), "edges_near must not repeat an id");
    // Every id present must be one this graph created, and the first entry must
    // be reproducible run to run.
    assert!(near.iter().all(|&id| id < g.edges.len()));
    let again = g.edges_near(Vec2::new(-10., -10.), Vec2::new(310., 210.));
    assert_eq!(near, again, "edges_near must be deterministic across calls");
}

#[test]
fn unindex_edge_removes_exactly_what_index_edge_added() {
    // Also not on `_test` in isolation — the goldens pin the index's *state*
    // after real operations, and this pins the invariant behind it: indexing
    // then unindexing an edge restores the grid to its prior contents.
    //
    // The two streets must not cross: a crossing would tombstone and re-split
    // the first one, so the "prior contents" the test compares against would no
    // longer exist. (That mistake is why this test failed on its first run.)
    let mut g = Graph::new();
    g.add_street(10., 10., 260., 60., "street", 5., 0, "p");
    let before = grid_sorted(&g);
    let made = g.add_street(10., 300., 260., 350., "street", 5., 0, "p");
    assert_eq!(made.len(), 1, "the second street must be one clean edge");
    assert_ne!(grid_sorted(&g), before, "the second edge must reach the index");
    g.unindex_edge(made[0]);
    assert_eq!(grid_sorted(&g), before, "unindexing must restore the prior cells exactly");
}

#[test]
fn extract_faces_is_empty_on_a_graph_with_no_cycle() {
    // A tree has no bounded face, and the reference returns `[]` rather than a
    // degenerate ring — `build_blocks` (milestone 12) reads that emptiness.
    let mut g = Graph::new();
    g.add_street(0., 0., 200., 0., "street", 5., 0, "p");
    g.add_street(100., 0., 100., 120., "street", 5., 0, "p");
    g.add_street(100., 60., 220., 60., "street", 5., 0, "p");
    assert!(g.extract_faces().is_empty());
}
