//! Golden-parity tests for `TIMELINE_SCOPE.md` milestone 2
//! (`cartalith_civ::timeline`): `_civProximityAdjacency` (reference lines
//! 24672-24683) and `_civBetweennessFromAdjacency` (24687-24709).
//!
//! Generated from a Node `vm.runInContext` extraction run against
//! `reference/Cartalith Gen1 v2.10.html` (harness itself transient, not
//! checked in, per this project's convention -- see `PARITY_TESTING.md` and
//! `golden_parity_settlement_population.rs`'s own header), slicing lines
//! 24672-24709 verbatim into a `vm` context stubbed with `state={world:
//! false}` and `GW=<the fixture's own grid width>` (the two module globals
//! the reference functions themselves read for wrap/scale). No reference
//! source was transcribed or reimplemented by hand for the expected values
//! below -- every adjacency list and betweenness number is the real
//! reference's own output, read back out of the harness.
//!
//! Both functions are deterministic (no RNG) per the reference's own v0.85
//! block comment, so there is no seed-alignment risk -- a fixture either
//! matches exactly (up to float tolerance) or the port has a real bug.
//!
//! Four fixture groups, per this project's own "shape fixtures to reach
//! real branches" discipline:
//! - a 3-node hand-checkable path (also independently hand-derived in this
//!   file's own comments, not just asserted against the harness output);
//! - a 5-node hand-checkable chain/star;
//! - two world-wrap fixtures (a 2-relevant-node pair only adjacent through
//!   the wrap, and a 4-node ring that only closes into a cycle -- versus
//!   staying a path -- when wrap is on), proving `world_wrap` actually
//!   changes the graph, not just plumbing a bool through;
//! - an 8-settlement "real-scale" fixture on a 512x328/800km grid (the
//!   engine's own default extent), at two different `k` values, one of
//!   which produces a disconnected graph -- exercising Brandes over
//!   multiple components, which the reference's own BFS-per-source
//!   structure must handle without crossing them.

use cartalith_civ::timeline::{civ_betweenness_from_adjacency, civ_proximity_adjacency};

fn assert_adj_eq(actual: &[Vec<usize>], expected: &[Vec<usize>], label: &str) {
    assert_eq!(actual.len(), expected.len(), "{label}: node count");
    for (i, (a, e)) in actual.iter().zip(expected.iter()).enumerate() {
        let mut a_sorted = a.clone();
        a_sorted.sort_unstable();
        let mut e_sorted = e.clone();
        e_sorted.sort_unstable();
        assert_eq!(a_sorted, e_sorted, "{label}: node {i} neighbours");
    }
}

fn assert_btw_close(actual: &[f64], expected: &[f64], label: &str) {
    assert_eq!(actual.len(), expected.len(), "{label}: length");
    const ATOL: f64 = 1e-9;
    for (i, (a, e)) in actual.iter().zip(expected.iter()).enumerate() {
        assert!((a - e).abs() <= ATOL, "{label}: node {i} got {a}, want {e}");
    }
}

// ---------- hand-checkable: 3-node path ----------

/// Points at x=0,10,20 (y=0), cellKm=1, k=1, maxKm=15 (excludes the 0-2
/// pair, raw distance 20). Node 1 ends up adjacent to both 0 and 2 (each of
/// 0 and 2 picks node 1 as their single nearest neighbour; node 1 itself
/// picks node 0, tied at distance 10 with node 2 but 0 comes first in
/// iteration order, and stable sort keeps it first -- so the 1-2 edge only
/// exists because node 2's own pass adds it symmetrically), giving the path
/// 0-1-2. Hand-derived betweenness (`timeline.rs`'s own
/// `betweenness_on_a_3_node_path_matches_hand_derivation` unit test derives
/// this from Brandes by hand in comments): every shortest path between the
/// two endpoints, counted in both directions (the reference never divides
/// by 2), passes through node 1 -- raw betweenness `[0, 2, 0]`.
#[test]
fn path_graph_3_matches_the_reference_and_a_hand_derivation() {
    let positions = [(0.0, 0.0), (10.0, 0.0), (20.0, 0.0)];
    let adj = civ_proximity_adjacency(&positions, 1, 15.0, 1.0, 100.0, false);
    assert_adj_eq(
        &adj,
        &[vec![1], vec![0, 2], vec![1]],
        "path_graph_3 adjacency",
    );
    let btw = civ_betweenness_from_adjacency(&adj);
    assert_btw_close(&btw, &[0.0, 2.0, 0.0], "path_graph_3 betweenness");
}

// ---------- hand-checkable: 5-node chain ----------

/// Points at x=0,5,10,15,20 (y=0), cellKm=1, k=2, generous maxKm=100 (every
/// pair is within range). Each node's two nearest neighbours are its
/// immediate left/right along the line, so the graph is the path
/// 0-1-2-3-4 with no shortcuts. Node 2 (the centre) is the sole
/// intermediate for every shortest path that crosses it.
#[test]
fn chain_5_matches_the_reference() {
    let positions = [
        (0.0, 0.0),
        (5.0, 0.0),
        (10.0, 0.0),
        (15.0, 0.0),
        (20.0, 0.0),
    ];
    let adj = civ_proximity_adjacency(&positions, 2, 100.0, 1.0, 100.0, false);
    assert_adj_eq(
        &adj,
        &[
            vec![1, 2],
            vec![0, 2],
            vec![0, 1, 3, 4],
            vec![2, 4],
            vec![2, 3],
        ],
        "chain_5 adjacency",
    );
    let btw = civ_betweenness_from_adjacency(&adj);
    assert_btw_close(&btw, &[0.0, 0.0, 8.0, 0.0, 0.0], "chain_5 betweenness");
}

// ---------- world-wrap: a pair only adjacent through the seam ----------

/// GW=100. Points A=(2,0), B=(98,0), C=(50,0), D=(50,30); maxKm=20, k=1.
/// Without wrap every pairwise distance exceeds 20 (A-B=96, the rest at
/// least 30) -- an empty graph. With wrap, A-B's wrap distance is
/// `min(96, 100-96)=4`, well under 20, so A and B become each other's sole
/// neighbour while C and D stay isolated. Proves `world_wrap` actually
/// changes which edges exist, not just that the flag is threaded through.
#[test]
fn wrap_pair_is_adjacent_only_with_world_wrap_on() {
    let positions = [(2.0, 0.0), (98.0, 0.0), (50.0, 0.0), (50.0, 30.0)];

    let no_wrap = civ_proximity_adjacency(&positions, 1, 20.0, 1.0, 100.0, false);
    assert_adj_eq(
        &no_wrap,
        &[vec![], vec![], vec![], vec![]],
        "wrap_pair no-wrap adjacency",
    );
    assert_btw_close(
        &civ_betweenness_from_adjacency(&no_wrap),
        &[0.0, 0.0, 0.0, 0.0],
        "wrap_pair no-wrap betweenness",
    );

    let with_wrap = civ_proximity_adjacency(&positions, 1, 20.0, 1.0, 100.0, true);
    assert_adj_eq(
        &with_wrap,
        &[vec![1], vec![0], vec![], vec![]],
        "wrap_pair with-wrap adjacency",
    );
    assert_btw_close(
        &civ_betweenness_from_adjacency(&with_wrap),
        &[0.0, 0.0, 0.0, 0.0],
        "wrap_pair with-wrap betweenness",
    );
}

// ---------- world-wrap: a ring that only closes with wrap on ----------

/// GW=100. Points evenly spaced around the wrapped X axis: 0, 25, 50, 75
/// (y=0), k=2, maxKm=30. Each node's two nearest neighbours are its
/// immediate ring neighbours (distance 25); the far side (distance 50)
/// never qualifies. With wrap on, the seam edge (75 -> 0, wrap distance 25)
/// exists, closing a clean 4-cycle: raw betweenness ties at `[1,1,1,1]`
/// (each node sits on exactly one of the two tied shortest paths between
/// the two "opposite" pairs, in both directions). With wrap off, the seam
/// edge never forms and the same positions are just the path 0-1-2-3:
/// raw betweenness `[0,4,4,0]`.
#[test]
fn wrap_ring_4_closes_into_a_cycle_only_with_world_wrap_on() {
    let positions = [(0.0, 0.0), (25.0, 0.0), (50.0, 0.0), (75.0, 0.0)];

    let ring = civ_proximity_adjacency(&positions, 2, 30.0, 1.0, 100.0, true);
    assert_adj_eq(
        &ring,
        &[vec![1, 3], vec![0, 2], vec![1, 3], vec![0, 2]],
        "wrap_ring_4 with-wrap adjacency",
    );
    assert_btw_close(
        &civ_betweenness_from_adjacency(&ring),
        &[1.0, 1.0, 1.0, 1.0],
        "wrap_ring_4 with-wrap betweenness",
    );

    let path = civ_proximity_adjacency(&positions, 2, 30.0, 1.0, 100.0, false);
    assert_adj_eq(
        &path,
        &[vec![1], vec![0, 2], vec![1, 3], vec![2]],
        "wrap_ring_4 no-wrap adjacency",
    );
    assert_btw_close(
        &civ_betweenness_from_adjacency(&path),
        &[0.0, 4.0, 4.0, 0.0],
        "wrap_ring_4 no-wrap betweenness",
    );
}

// ---------- real-scale: 8 settlements on the engine's default extent ----------

/// `gw=512`, `mapWidthKm=800` -> `cellKm=1.5625` (the engine's own default
/// extent, `MVP_SCOPE.md`'s reference configuration). Eight positions
/// loosely forming two coastal-scale clusters plus a bridging settlement
/// (node 7) and an outlier (node 6), on a 512x328 grid. `maxLinkKm =
/// cellKm*GW*0.5`, the reference's own default from `_civCollapseStep`
/// (line 24794) -- reused here even though milestone 3 (which calls
/// `_civCollapseStep` itself) isn't ported yet, because it's the
/// reference's own realistic default, not an invented one.
///
/// Two `k` values: `k=4` (the reference's own default,
/// `opts.kNearest||4`) produces one connected graph where the bridging
/// node (7) and the largest cluster's hub (3) both carry the graph's
/// entire betweenness load; `k=2` splits the same positions into two
/// disconnected components (`{0,1,2,7}` and `{3,4,5,6}`), exercising
/// Brandes across multiple components in one call -- BFS from any node in
/// one component never reaches the other, so no node ever accrues
/// cross-component betweenness.
#[test]
fn real_settlements_k4_matches_the_reference() {
    let gw = 512.0f64;
    let cell_km = 800.0 / gw;
    let positions = [
        (100.0, 80.0),
        (120.0, 90.0),
        (140.0, 70.0),
        (300.0, 200.0),
        (310.0, 210.0),
        (330.0, 190.0),
        (420.0, 60.0),
        (200.0, 150.0),
    ];
    let max_km = cell_km * gw * 0.5;
    let adj = civ_proximity_adjacency(&positions, 4, max_km, cell_km, gw, false);
    assert_adj_eq(
        &adj,
        &[
            vec![1, 2, 3, 7],
            vec![0, 2, 3, 7],
            vec![0, 1, 3, 7],
            vec![0, 1, 2, 4, 5, 6, 7],
            vec![3, 5, 6, 7],
            vec![3, 4, 6, 7],
            vec![3, 4, 5, 7],
            vec![0, 1, 2, 3, 4, 5, 6],
        ],
        "real_settlements k=4 adjacency",
    );
    let btw = civ_betweenness_from_adjacency(&adj);
    assert_btw_close(
        &btw,
        &[0.0, 0.0, 0.0, 9.0, 0.0, 0.0, 0.0, 9.0],
        "real_settlements k=4 betweenness",
    );
}

#[test]
fn real_settlements_k2_splits_into_two_components_and_betweenness_never_crosses_them() {
    let gw = 512.0f64;
    let cell_km = 800.0 / gw;
    let positions = [
        (100.0, 80.0),
        (120.0, 90.0),
        (140.0, 70.0),
        (300.0, 200.0),
        (310.0, 210.0),
        (330.0, 190.0),
        (420.0, 60.0),
        (200.0, 150.0),
    ];
    let max_km = cell_km * gw * 0.5;
    let adj = civ_proximity_adjacency(&positions, 2, max_km, cell_km, gw, false);
    assert_adj_eq(
        &adj,
        &[
            vec![1, 2],
            vec![0, 2, 7],
            vec![0, 1, 7],
            vec![4, 5, 6],
            vec![3, 5],
            vec![3, 4, 6],
            vec![3, 5],
            vec![1, 2],
        ],
        "real_settlements k=2 adjacency",
    );
    let btw = civ_betweenness_from_adjacency(&adj);
    assert_btw_close(
        &btw,
        &[0.0, 1.0, 1.0, 1.0, 0.0, 1.0, 0.0, 0.0],
        "real_settlements k=2 betweenness",
    );
}
