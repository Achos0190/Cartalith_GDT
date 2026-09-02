//! Milestone 11's tests.
//!
//! **Golden**, on the same terms as milestones 2, 7 and 12: [`golden`] holds
//! the reference engine's own output for 42 scenarios, and the fixtures below
//! rebuild the identical input in this port and compare. The comparison is
//! exact — nothing in this milestone's path passes through a transcendental
//! whose last bit could legitimately differ between V8 and Rust; the only
//! library calls anywhere in it are `Math.hypot` (through
//! [`crate::geom::js_hypot`]) and `atan2` inside `extract_faces` (already
//! routed through the reference's own semantics by milestone 2).
//!
//! Coverage is a hash over the **whole graph** — node positions, node
//! adjacency lists *in order*, and every edge's endpoints, class, width, epoch
//! and liveness — plus the counts asserted separately and first, because a hash
//! mismatch caused by a length difference is the least informative failure this
//! file could produce. Adjacency order is in the hash deliberately:
//! [`kill_edge`]'s entire job is to splice those lists, and a hash that omitted
//! them would pass with the splice removed.
//!
//! ## What the mutation sweep found
//!
//! Every constant this milestone ports was flipped by one unit and the suite
//! re-run — 33 mutants, of which **30 are caught**. The three that survive are
//! named below with the reason, and each is asserted from the other side rather
//! than left as prose.
//!
//! | Mutant | Killed by |
//! |---|---|
//! | `wet >= 2` -> `>= 3` | 4 tests, incl. [`the_wet_band_is_river_width_over_two_plus_a_half`] |
//! | the 9 interior samples (`i in 1..10` -> `1..11`) | [`water_crossings_riverthrough`] |
//! | `riverW/2 + 0.5` -> `+ 1.5` | [`the_wet_band_is_river_width_over_two_plus_a_half`] |
//! | `openWet >= 2` -> `>= 3` | [`water_crossings_real_mask`] |
//! | the `'quay'` exemption, dropped | [`fort_zone_bastioned_uses_the_glacis_offset`] |
//! | `pruneLargest` dropped from `removeWaterCrossings` | [`water_crossings_real_mask`] |
//! | the `0.40` bias ceiling, both directions | [`the_bias_ceiling_saturates`], [`privatize_big_grid_ceiling`] |
//! | `chance(0.5)` -> `0.6` | 8 tests |
//! | the live-degree `< 2` -> `< 3` | 8 tests |
//! | the strict `>` component tie-break -> `>=` | [`prune_equal_components_keep_the_first_seen`] |
//! | `_killEdge`'s `k >= 0` guard, removed | [`the_kill_guard_is_what_stops_a_second_kill_corrupting_adjacency`] |
//! | `12000` default `minArea` -> `12001` | [`the_default_min_area_is_inclusive_at_exactly_12000`] |
//! | `140000` ceiling -> `140001` **and** `139999` | [`the_area_ceiling_is_exclusive_at_exactly_140000`] |
//! | `520` market radius -> `521` | [`the_market_radius_is_exclusive_at_exactly_520`] |
//! | `r.range(0.35, 0.65)` -> `(0.36, 0.65)` | 8 tests |
//! | lane width `2.6` -> `2.7` | 8 tests (the width is in the hash, x10) |
//! | the `'lanes/' + epoch` label, epoch dropped | 8 tests |
//! | the 30 m lane separation -> `31` | [`a_lane_shorter_than_thirty_metres_is_not_laid`] |
//! | the two-longest scan `>` -> `>=` | 4 tests |
//! | `15` wall clear distance -> `16` | [`fort_zone_sweep_matches`] |
//! | the `\|\| 60` glacis fallback -> `61` | [`a_missing_or_zero_glacis_offset_falls_back_to_sixty`] |
//! | `glacisOff + 8` -> `+ 9` | [`fort_zone_bastioned_uses_the_glacis_offset`] |
//! | `clearDist + 16` primary corridor -> `+ 17` | [`fort_zone_sweep_matches`] |
//! | `clearDist * 0.85` -> `* 0.86` | [`the_gate_corridor_radius_at_its_boundary`] |
//! | the centroid half of `polyInClear`, dropped | 5 tests |
//! | the vertex half of `polyInClear`, dropped | 5 tests |
//!
//! ### Boundary fixtures, built deliberately
//!
//! Six of the rows above only became catchable once a fixture was put **on** the
//! boundary — milestone 12 recorded the same lesson and this milestone built for
//! it rather than discovering the survivor. A 400 x 350 face is exactly
//! 140 000 m² (kept, the test is `A > 140000`) against 400 x 350.001; a
//! 100 x 120 face is exactly 12 000 (kept, `A < minArea`) against 100 x 119.9; a
//! centroid exactly 520 m from the market (kept, `> 520`) against 520.1; a
//! 60 x 27 face against a 60 x 28 one for the 30 m separation; and, for the fort,
//! a 0.5 m radial ladder of probe polygons plus one gate placed 12.8 m from a
//! known ring crossing.
//!
//! ### The 11 m ceiling on every graph-resolved fixture
//!
//! The finding worth carrying forward, because it cost this milestone four
//! rounds: **`Graph::attach_point`'s 11 m snap caps the resolution of any ladder
//! built out of graph nodes.** A ladder of streets 1 m apart does not become a
//! 1 m ladder — the endpoints merge and most rungs produce no edge at all. Every
//! constant here that is compared against a *node-derived* distance is therefore
//! only pinnable to ~11 m, and the three ways round it are all used above:
//! probe **polygons** (not in the graph, so any spacing), a **gate** position
//! (also not in the graph — 12.8 m from a crossing resolves a 1 % change in
//! `clearDist * 0.85` that no node ladder could), and a 16 m **lattice** whose
//! spread of `river_dist` values is dense even though its node spacing is not.
//! This is the same shape as milestone 12's note that its 120 m² block floor is
//! unreachable because an ~11 m cell collapses before `extract_faces` sees it.
//!
//! ### The three survivors
//!
//! 1. **`(riverW || 20) * 1.5 + 34` -> `+ 35`.** The bridge radius is compared
//!    against distances between *sample points on graph edges*, and the 11 m cap
//!    above applies. Measured rather than assumed: the expression **is** pinned
//!    at coarser grain — `* 1.5 -> * 2.5` and `+ 34 -> + 134` are both caught by
//!    [`water_crossings_real_mask`] — so it is the one-metre window that is
//!    invisible here, not the constant.
//! 2. **`riverW || 20`'s fallback -> `21`.** Dead by construction: the `||` sits
//!    inside the `usesRealWater` branch, and `buildSite` only sets that flag
//!    when `opts.water` is present, in which case `riverW` is either
//!    `W.riverWidthM || 20` (already non-zero) or the literal 12. There is no
//!    input that reaches it with a zero. Asserted inside
//!    [`water_crossings_real_mask`], both halves.
//! 3. **The `0.12` wet-scan step -> `0.13`.** Same shape as (1): the nine sample
//!    points move by at most 8 cm each, and no candidate lane in any fixture has
//!    a shoreline that close to a sample. `0.12 -> 0.24` and `0.12 -> 0.5` are
//!    both caught by [`lanes_are_not_laid_across_water`], so the sample *count*
//!    is pinned and the step is pinned to within a doubling.
//!
//! A fourth mutant is proved dead instead of caught: `while t <= 1.0` against
//! `while t < 1.0`. The accumulation never lands on 1.0 — the ninth sample is
//! 0.96 and the tenth 1.08 — so the two loops are the same loop.
//! [`lanes_are_not_laid_across_water`] asserts that, and also the measurement
//! that disproves this port's own first guess about the loop: `t += 0.12` and
//! `i as f64 * 0.12` agree bit for bit on all nine values.
//!
//! **One more not caught, and stated rather than implied:** the `0` *lower*
//! bound of `clamp(…, 0, 0.40)`. `privatize_negative` gives it a -0.5 rules bias
//! and nothing closes — but that is also true with the lower bound removed,
//! because a negative `target` makes `closed >= target` true on entry and the
//! loop breaks. [`negative_bias_closes_nothing_either_way`] asserts exactly that
//! rather than pretending the golden pins it.
//!
//! ## Where the reference could not be reproduced as written
//!
//! `clearFortZone`'s three collections. See [`clear_fort_zone`]'s own doc
//! comment: `buildings` and `details` are milestones 13 and 15, and
//! [`crate::blocks::Parcel`] has no `cleared` field. The port sweeps the same
//! geometry and **reports** the indices; [`fort_zone_sweep_matches`] applies the
//! report the way a caller must and compares against the reference's own
//! post-splice list. The graph half of `clearFortZone` is unaffected and is
//! compared by hash like everything else.

mod golden;

use super::*;
use crate::geom::Vec2;
use crate::graph::Graph;
use crate::growth::Gate;
use crate::rng::fnv1a;
use crate::routes::Anchors;
use crate::rules::{MEDIEVAL, StreetRules};
use crate::site::{SiteOpts, WaterCtx, build_site};

// ------------------------------------------------------------- the fixtures --

/// The capture's jitter table, verbatim. Interior grid nodes are pushed off the
/// lattice so no threshold in this milestone collapses to a symmetric case.
const JIT: [f64; 15] =
    [3.1, -4.7, 2.2, -1.3, 5.4, -2.8, 1.9, -5.1, 4.3, -3.6, 0.7, -4.2, 2.6, -1.8, 3.9];

#[allow(clippy::too_many_arguments)]
fn grid(
    g: &mut Graph,
    x0: f64,
    y0: f64,
    cols: usize,
    rows: usize,
    dx: f64,
    dy: f64,
    cls: &'static str,
    w: f64,
    epoch: i32,
    jitter: bool,
) {
    let px = |i: usize, j: usize| {
        x0 + i as f64 * dx
            + if jitter && i > 0 && i < cols { JIT[(i * 7 + j * 3) % JIT.len()] } else { 0.0 }
    };
    let py = |i: usize, j: usize| {
        y0 + j as f64 * dy
            + if jitter && j > 0 && j < rows { JIT[(i * 5 + j * 11) % JIT.len()] } else { 0.0 }
    };
    for j in 0..=rows {
        for i in 0..cols {
            g.add_street(px(i, j), py(i, j), px(i + 1, j), py(i + 1, j), cls, w, epoch, "grid");
        }
    }
    for i in 0..=cols {
        for j in 0..rows {
            g.add_street(px(i, j), py(i, j), px(i, j + 1), py(i, j + 1), cls, w, epoch, "grid");
        }
    }
}

fn rect(g: &mut Graph, x0: f64, y0: f64, w: f64, h: f64) {
    g.add_street(x0, y0, x0 + w, y0, "street", 6.0, 1, "r");
    g.add_street(x0 + w, y0, x0 + w, y0 + h, "street", 6.0, 1, "r");
    g.add_street(x0 + w, y0 + h, x0, y0 + h, "street", 6.0, 1, "r");
    g.add_street(x0, y0 + h, x0, y0, "street", 6.0, 1, "r");
}

fn alive(g: &Graph) -> usize {
    g.edges.iter().filter(|e| e.alive).count()
}

/// The capture's canonical dump, character for character. Integers only, so JS
/// and Rust cannot disagree on float formatting.
fn canon(g: &Graph) -> String {
    let mut parts: Vec<String> = Vec::new();
    for n in &g.nodes {
        parts.push(n.id.to_string());
        parts.push((crate::geom::js_round(n.x * 100.0) as i64).to_string());
        parts.push((crate::geom::js_round(n.y * 100.0) as i64).to_string());
        parts.push(n.adj.iter().map(|i| i.to_string()).collect::<Vec<_>>().join(","));
    }
    for e in &g.edges {
        parts.push(e.id.to_string());
        parts.push(e.a.to_string());
        parts.push(e.b.to_string());
        parts.push(e.cls.to_string());
        parts.push((crate::geom::js_round(e.w * 10.0) as i64).to_string());
        parts.push(e.epoch.to_string());
        parts.push(if e.alive { "1" } else { "0" }.to_string());
    }
    parts.join("|")
}

fn scenario(name: &str) -> &'static golden::Scenario {
    golden::SCENARIOS
        .iter()
        .find(|s| s.name == name)
        .unwrap_or_else(|| panic!("no golden scenario named {name}"))
}

/// Compare a finished graph against its golden — counts first, then the hash.
fn check(name: &str, g: &Graph, before: usize) {
    let s = scenario(name);
    // Shape and non-emptiness, explicitly, before anything is hashed.
    assert!(!g.nodes.is_empty(), "{name}: empty node list");
    assert!(!g.edges.is_empty(), "{name}: empty edge list");
    assert!(alive(g) > 0, "{name}: no live edges left");
    assert_eq!(g.nodes.len(), s.nodes, "{name}: node count");
    assert_eq!(g.edges.len(), s.edges_total, "{name}: edge count");
    assert_eq!(before, s.before, "{name}: live edges BEFORE the pass");
    assert_eq!(alive(g), s.after, "{name}: live edges AFTER the pass");
    assert_eq!(fnv1a(&canon(g)), s.hash, "{name}: full-graph hash");
}

fn lane_added(name: &str) -> usize {
    golden::LANE_ADDED
        .iter()
        .find(|(n, _)| *n == name)
        .unwrap_or_else(|| panic!("no lane golden named {name}"))
        .1
}

// ----------------------------------------------------------- pruneLargest --

#[test]
fn prune_keeps_the_largest_component() {
    let mut g = Graph::new();
    grid(&mut g, 100.0, 100.0, 3, 3, 60.0, 55.0, "street", 6.0, 1, true);
    grid(&mut g, 900.0, 800.0, 2, 1, 50.0, 50.0, "street", 6.0, 1, true);
    let before = alive(&g);
    prune_largest(&mut g);
    check("prune_two_components", &g, before);
    assert!(alive(&g) < before, "the small component must have died");
}

/// Two components of identical size. The reference's `sizes[i] > sizes[best]`
/// is strict, so the **first-seen** one survives — and first-seen is insertion
/// order into a JS `Map`, which is why [`prune_largest`] keeps a parallel key
/// order rather than iterating a `HashMap`.
#[test]
fn prune_equal_components_keep_the_first_seen() {
    let mut g = Graph::new();
    grid(&mut g, 100.0, 100.0, 2, 2, 60.0, 55.0, "street", 6.0, 1, false);
    grid(&mut g, 900.0, 800.0, 2, 2, 60.0, 55.0, "street", 6.0, 1, false);
    let before = alive(&g);
    prune_largest(&mut g);
    check("prune_equal_tie", &g, before);
    // Concretely: the surviving edges are the low-id half, i.e. the first grid.
    let live: Vec<usize> = g.edges.iter().filter(|e| e.alive).map(|e| e.id).collect();
    assert_eq!(live.len(), before / 2);
    assert!(live.iter().all(|&id| id < before / 2), "the second grid survived instead");
}

#[test]
fn prune_leaves_a_connected_graph_alone() {
    let mut g = Graph::new();
    grid(&mut g, 100.0, 100.0, 3, 3, 60.0, 55.0, "street", 6.0, 1, true);
    let before = alive(&g);
    prune_largest(&mut g);
    check("prune_single_component", &g, before);
    assert_eq!(alive(&g), before, "a single component must lose nothing");
}

// -------------------------------------------------------------- _killEdge --

#[test]
fn kill_edge_unhooks_both_endpoints_and_is_idempotent() {
    let mut g = Graph::new();
    grid(&mut g, 100.0, 100.0, 2, 2, 70.0, 65.0, "street", 6.0, 1, true);
    let victim = g.edges.iter().find(|e| e.alive).expect("fixture has a live edge").id;
    let (va, vb) = (g.edges[victim].a, g.edges[victim].b);
    let before = alive(&g);

    kill_edge(&mut g, victim);
    check("kill_one_edge", &g, before);
    assert!(!g.edges[victim].alive);
    assert!(!g.nodes[va].adj.contains(&victim), "adj[a] still holds the dead id");
    assert!(!g.nodes[vb].adj.contains(&victim), "adj[b] still holds the dead id");
    let adj_after_first = (g.nodes[va].adj.clone(), g.nodes[vb].adj.clone());

    // The `if (k >= 0)` guard — this is the difference from `split_edge`, whose
    // unguarded splice would drop the LAST element of each list here.
    kill_edge(&mut g, victim);
    check("kill_one_edge_twice", &g, before);
    assert_eq!(
        (g.nodes[va].adj.clone(), g.nodes[vb].adj.clone()),
        adj_after_first,
        "the second kill must not touch either adjacency list"
    );
}

/// The guard is not decorative: without it, a second kill would drop an
/// unrelated live edge id off the end of each list. Written as the assertion
/// the module header promises rather than left as a claim.
#[test]
fn the_kill_guard_is_what_stops_a_second_kill_corrupting_adjacency() {
    let mut g = Graph::new();
    grid(&mut g, 100.0, 100.0, 2, 2, 70.0, 65.0, "street", 6.0, 1, true);
    let victim = g.edges.iter().find(|e| e.alive).expect("live edge").id;
    let va = g.edges[victim].a;
    kill_edge(&mut g, victim);
    let kept = g.nodes[va].adj.clone();
    assert!(!kept.is_empty(), "the fixture node must keep other edges");
    // What `split_edge`'s unguarded `splice(-1, 1)` would do on a miss:
    let mut corrupted = kept.clone();
    corrupted.pop();
    kill_edge(&mut g, victim);
    assert_eq!(g.nodes[va].adj, kept);
    assert_ne!(g.nodes[va].adj, corrupted, "the guard did not fire");
}

// ----------------------------------------------------- removeWaterCrossings --

fn water_scenario(kind: &str) {
    let site = build_site(4242, 1700.0, 1250.0, kind, SiteOpts::default());
    let mut g = Graph::new();
    grid(&mut g, 200.0, 200.0, 6, 5, 200.0, 160.0, "street", 6.0, 1, true);
    g.add_street(210.0, 640.0, 1600.0, 640.0, "primary", 9.0, 0, "p");
    g.add_street(250.0, 300.0, 250.0, 1100.0, "quay", 5.0, 0, "q");
    let before = alive(&g);
    remove_water_crossings(&site, &mut g);
    check(&format!("water_{kind}"), &g, before);
}

/// A 1 m ladder of short streets across the whole channel band. Each rung is
/// 6 m long, so all nine of its interior samples sit within 5 m of one another
/// and the rung is wholly wet or wholly dry -- which makes any one-metre change
/// to `riverW/2 + 0.5` move exactly one rung across the boundary. A realistic
/// grid has nothing inside that 1 m shell, and without this ladder the constant
/// survives its own mutation.
#[test]
fn the_wet_band_is_river_width_over_two_plus_a_half() {
    let site = build_site(4242, 1700.0, 1250.0, "river", SiteOpts::default());
    let mut g = Graph::new();
    grid(&mut g, 200.0, 200.0, 4, 3, 260.0, 200.0, "street", 6.0, 1, true);
    // A 16 m LATTICE, not a 1 m line: `attach_point`'s 11 m snap merges anything
    // closer, so a metre-resolution ladder collapses into a single node and
    // pins nothing. The synthetic centreline wanders, so 16 m steps in x and y
    // still produce a dense set of `river_dist` values, which is what resolving
    // a one-metre change to the band actually needs.
    for i in 0..14usize {
        for j in 0..14usize {
            let (x, y) = (700.0 + i as f64 * 16.0, 560.0 + j as f64 * 16.0);
            g.add_street(x, y, x + 5.0, y, "street", 4.0, 1, "rung");
        }
    }
    let before = alive(&g);
    remove_water_crossings(&site, &mut g);
    check("water_band_ladder", &g, before);
    assert!(alive(&g) < before, "the ladder must lose its wet rungs");
}

#[test]
fn water_crossings_river() {
    water_scenario("river");
}
#[test]
fn water_crossings_riverthrough() {
    water_scenario("riverthrough");
}
#[test]
fn water_crossings_coastal() {
    water_scenario("coastal");
}

/// `'quay'` is skipped by both sweeps — it hugs the waterline by design, so
/// every one of its samples is wet and an unexempted quay would be culled
/// entirely. On the synthetic fixtures the whole quay survives, which is the
/// exemption stated where nothing else can explain it away.
#[test]
fn the_quay_is_exempt_from_the_sweeps() {
    for kind in ["river", "coastal"] {
        let site = build_site(4242, 1700.0, 1250.0, kind, SiteOpts::default());
        let mut g = Graph::new();
        grid(&mut g, 200.0, 200.0, 6, 5, 200.0, 160.0, "street", 6.0, 1, true);
        g.add_street(210.0, 640.0, 1600.0, 640.0, "primary", 9.0, 0, "p");
        g.add_street(250.0, 300.0, 250.0, 1100.0, "quay", 5.0, 0, "q");
        remove_water_crossings(&site, &mut g);
        let quay: Vec<_> = g.edges.iter().filter(|e| e.cls == "quay").collect();
        assert_eq!(quay.len(), 7, "{kind}: the quay split into 7 segments");
        assert!(quay.iter().all(|e| e.alive), "{kind}: a quay segment was culled");
    }
}
#[test]
fn water_crossings_inland() {
    water_scenario("inland");
}

/// `if (site.noWater) return` — a landlocked site is left entirely alone, and
/// in particular is **not** pruned, which is the observable difference from
/// every other kind here.
#[test]
fn water_crossings_landlocked_returns_untouched() {
    water_scenario("landlocked");
    let site = build_site(4242, 1700.0, 1250.0, "landlocked", SiteOpts::default());
    assert!(site.no_water);
    let s = scenario("water_landlocked");
    assert_eq!(s.before, s.after, "a landlocked site must lose nothing");
}

/// The real-map fixture's water context: a 150 m horizontal channel through a
/// 34 x 25 mask at 50 m cells, with a straight centreline and an explicit
/// 150 m width. Shared with the dead-fallback assertion inside the test.
fn w_spec() -> WaterCtx {
    let (mw, mh, cell_m) = (34usize, 25usize, 50.0f64);
    let mut mask = vec![0u8; mw * mh];
    let mut dt = vec![0.0f64; mw * mh];
    for j in 0..mh {
        for i in 0..mw {
            let yj = j as f64 * cell_m;
            mask[j * mw + i] = u8::from((550.0..700.0).contains(&yj));
            dt[j * mw + i] = (yj - 625.0).abs() / cell_m;
        }
    }
    WaterCtx {
        mask,
        dt,
        mw,
        mh,
        cell_m,
        river_path: Some(vec![Vec2::new(0.0, 625.0), Vec2::new(1700.0, 625.0)]),
        river_width_m: Some(150.0),
        river_order: 4.0,
        sea_lake_cells: 0.0,
    }
}

/// The real-map fixture, and the two things only it can reach: the
/// `uses_real_water` second sweep, and the local `rk` that is **not**
/// `site.rk`. This site is `'coastal'` *and* carries a river path, so
/// `buildSite` calls it river-like while `removeWaterCrossings` does not.
#[test]
fn water_crossings_real_mask() {
    let site = build_site(
        88,
        1700.0,
        1250.0,
        "coastal",
        SiteOpts { water: Some(w_spec()), ..Default::default() },
    );

    // The site itself must match the reference's before the graph can mean
    // anything — and it pins the `rk` divergence this test exists for.
    assert!(site.uses_real_water);
    assert!(site.river_like(), "buildSite calls this river-like");
    assert_ne!(site.kind, "river", "removeWaterCrossings' own rk does not");
    assert_ne!(site.kind, "riverthrough");
    assert_eq!(site.river_w, golden::REALMASK_RIVER_W);
    assert_eq!(site.bridge_pt.map(|p| (p.x, p.y)), golden::REALMASK_BRIDGE_PT);

    let mut g = Graph::new();
    grid(&mut g, 150.0, 150.0, 7, 6, 200.0, 160.0, "street", 6.0, 1, true);
    g.add_street(300.0, 200.0, 300.0, 1100.0, "primary", 9.0, 0, "p");
    g.add_street(400.0, 400.0, 1300.0, 400.0, "quay", 5.0, 0, "q");
    // Long primaries that only GRAZE the channel: nine samples 111 m apart over
    // a 1000 m span, so one or two land in the 150 m band. These are the rungs
    // `open_wet >= 2` decides.
    for k in 0..24usize {
        let y0 = 120.0 + k as f64 * 4.0;
        g.add_street(1000.0, y0, 1600.0, y0 + 900.0, "primary", 7.0, 0, "graze");
    }
    let before = alive(&g);
    remove_water_crossings(&site, &mut g);
    check("water_realmask", &g, before);

    // The base sweep exempts primaries; the real-water sweep does not, so the
    // sub-segments of the north-south primary that lie IN the channel are
    // culled while the ones on either bank survive. Assert exactly that — not
    // "no primary survives", which is false and which the hash would not have
    // caught on its own.
    let primaries: Vec<_> = g.edges.iter().filter(|e| e.cls == "primary").collect();
    assert!(!primaries.is_empty(), "the fixture laid no primary at all");
    assert!(
        primaries.iter().any(|e| !e.alive),
        "no primary segment was culled — the real-water sweep did not run"
    );
    // `(site.riverW || 20)`'s fallback is DEAD on the only path that reads it,
    // and this is the assertion rather than the claim. The `|| 20` is inside
    // the `usesRealWater` branch, and `buildSite` only sets `usesRealWater`
    // when `opts.water` is present -- in which case `riverW` is either
    // `W.riverWidthM || 20` (already non-zero) or the literal 12. There is no
    // input for which the branch reading `site.riverW` sees a zero.
    assert!(site.river_w > 0.0, "river_w is never zero when uses_real_water is set");
    let bare = build_site(
        88,
        1700.0,
        1250.0,
        "coastal",
        SiteOpts { water: Some(WaterCtx { river_path: None, ..w_spec() }), ..Default::default() },
    );
    assert!(bare.uses_real_water && bare.river_w == 12.0, "the shore branch is 12, not 0");
    // The quay is exempt from **both** sweeps — and here it dies anyway, to
    // `prune_largest`: this fixture's largest surviving component is the south
    // bank, and the quay is on the north. Stated as the assertion, because the
    // obvious "the quay always survives" is false and the hash alone would not
    // have said which of the two reasons applied.
    // [`the_quay_is_exempt_from_the_sweeps`] is where the exemption itself is
    // pinned, on a fixture the prune cannot confound.
    assert!(g.edges.iter().any(|e| e.cls == "quay"), "the fixture laid no quay");
    assert!(
        !g.edges.iter().any(|e| e.alive && e.cls == "quay"),
        "the quay survived — this fixture's prune is expected to take it"
    );
}

// --------------------------------------------------------- privatizeAlleys --

fn priv_scenario(name: &str, seed: u32, p_bias: f64, r_bias: f64) {
    let mut g = Graph::new();
    grid(&mut g, 150.0, 150.0, 5, 4, 150.0, 140.0, "street", 6.0, 1, true);
    g.add_street(150.0, 150.0, 60.0, 60.0, "street", 6.0, 1, "spur");
    let before = alive(&g);
    let profile = crate::rules::CultureProfile { dead_end_bias: p_bias, ..MEDIEVAL };
    let rules = crate::rules::Rules {
        street: StreetRules { dead_end_bias: r_bias, ..DEFAULT_RULES.street },
        ..DEFAULT_RULES
    };
    privatize_alleys(seed, &profile, &mut g, Some(&rules));
    check(name, &g, before);
}

#[test]
fn privatize_zero_bias_is_a_no_op() {
    priv_scenario("privatize_zero", 777, 0.0, 0.0);
    let s = scenario("privatize_zero");
    assert_eq!(s.before, s.after);
}

#[test]
fn privatize_islamic_floor() {
    priv_scenario("privatize_016", 777, 0.0, 0.16);
    let s = scenario("privatize_016");
    assert!(s.after < s.before, "0.16 must close something");
}

#[test]
fn privatize_at_the_ceiling() {
    priv_scenario("privatize_040", 777, 0.0, 0.40);
}

/// `clamp(…, 0, 0.40)`'s upper bound: 0.90 must produce **exactly** what 0.40
/// produces, hash and all.
#[test]
fn privatize_clamps_the_bias_at_040() {
    priv_scenario("privatize_090_clamped", 777, 0.0, 0.90);
    assert_eq!(
        scenario("privatize_090_clamped").hash,
        scenario("privatize_040").hash,
        "0.90 and 0.40 must be the same run"
    );
}

/// The two sides **add**; they do not replace. 0.10 + 0.10 is not 0.10.
#[test]
fn privatize_profile_and_rules_bias_add() {
    priv_scenario("privatize_sum", 31337, 0.10, 0.10);
    assert_ne!(scenario("privatize_sum").hash, scenario("privatize_zero").hash);
}

/// The profile side alone drives the pass, which is what the expression is for
/// even though both live profiles leave it at zero.
#[test]
fn privatize_from_the_profile_side_alone() {
    priv_scenario("privatize_profile_only", 31337, 0.22, 0.0);
    let s = scenario("privatize_profile_only");
    assert!(s.after < s.before);
}

/// The clamp's **lower** bound is not pinned by this golden and this test says
/// so rather than pretending otherwise: with the clamp removed a −0.5 bias
/// gives a negative `target`, `closed >= target` is true immediately, and the
/// loop breaks having closed nothing — the same visible result. Asserted from
/// the other side instead: the pass is a no-op, and the *reason* is the break,
/// not the clamp.
#[test]
fn negative_bias_closes_nothing_either_way() {
    priv_scenario("privatize_negative", 777, 0.0, -0.5);
    let s = scenario("privatize_negative");
    assert_eq!(s.before, s.after);
    assert_eq!(s.hash, scenario("privatize_zero").hash);
    // The clamped value really is zero, so the `if (!bias) return` fires.
    assert_eq!(clamp(0.0 + -0.5, 0.0, 0.40), 0.0);
    // And with a hypothetical unclamped bias the loop would break at once.
    let target = crate::geom::js_round(50.0 * -0.5);
    assert!(0.0 >= target, "an unclamped negative target breaks on entry");
}

/// A NaN bias returns, because `if (!bias)` is falsy-tested and the clamp
/// cannot rescue a NaN. `applyPlotChaos` can put one there, so this is
/// reachable in the app — milestone 12 found the same trap.
#[test]
fn a_nan_bias_returns_without_touching_the_graph() {
    let mut g = Graph::new();
    grid(&mut g, 150.0, 150.0, 5, 4, 150.0, 140.0, "street", 6.0, 1, true);
    let snapshot = canon(&g);
    let profile = crate::rules::CultureProfile { dead_end_bias: f64::NAN, ..MEDIEVAL };
    privatize_alleys(777, &profile, &mut g, None);
    assert_eq!(canon(&g), snapshot, "a NaN bias must be a no-op");
    assert!(clamp(f64::NAN, 0.0, 0.40).is_nan(), "the clamp does not absorb the NaN");
}

/// `rules = None` takes `DEFAULT_RULES`, whose `street.dead_end_bias` is 0, so
/// the pass returns — the reference's `(rules || DEFAULT_RULES)`.
#[test]
fn privatize_falls_back_to_default_rules() {
    assert_eq!(DEFAULT_RULES.street.dead_end_bias, 0.0);
    let mut g = Graph::new();
    grid(&mut g, 150.0, 150.0, 5, 4, 150.0, 140.0, "street", 6.0, 1, true);
    let snapshot = canon(&g);
    privatize_alleys(777, &MEDIEVAL, &mut g, None);
    assert_eq!(canon(&g), snapshot);
}

/// The pass may never disconnect the network — that is the whole point of the
/// reachability filter, and it is a property no hash states.
#[test]
fn privatize_never_disconnects_the_network() {
    let mut g = Graph::new();
    grid(&mut g, 150.0, 150.0, 5, 4, 150.0, 140.0, "street", 6.0, 1, true);
    g.add_street(150.0, 150.0, 60.0, 60.0, "street", 6.0, 1, "spur");
    let profile = crate::rules::CultureProfile { dead_end_bias: 0.40, ..MEDIEVAL };
    privatize_alleys(777, &profile, &mut g, None);

    // `prune_largest` kills every edge outside the largest component, so if it
    // finds nothing to kill there was only ever one component.
    let before_prune = alive(&g);
    assert!(before_prune > 0, "the pass closed everything");
    assert!(before_prune < 50, "the pass closed nothing, so this proves nothing");
    prune_largest(&mut g);
    assert_eq!(alive(&g), before_prune, "privatize_alleys severed the network");
}

fn priv_big(name: &str, seed: u32, p_bias: f64, r_bias: f64) {
    let mut g = Graph::new();
    grid(&mut g, 150.0, 150.0, 9, 7, 110.0, 100.0, "street", 6.0, 1, true);
    let before = alive(&g);
    let profile = crate::rules::CultureProfile { dead_end_bias: p_bias, ..MEDIEVAL };
    let rules = crate::rules::Rules {
        street: StreetRules { dead_end_bias: r_bias, ..DEFAULT_RULES.street },
        ..DEFAULT_RULES
    };
    privatize_alleys(seed, &profile, &mut g, Some(&rules));
    check(name, &g, before);
}

/// The 0.40 ceiling from **below**, on a grid large enough that the target
/// really binds: 0.35 and 0.40 must differ, which is what makes a mutation that
/// *lowers* the ceiling visible.
///
/// The mutation that *raises* it is a different matter and this port does not
/// claim to catch it -- see [`the_bias_ceiling_saturates`].
#[test]
fn privatize_big_grid_ceiling() {
    priv_big("privatize_big_035", 555, 0.0, 0.35);
    priv_big("privatize_big_040", 555, 0.0, 0.40);
    let (a, b) = (scenario("privatize_big_035"), scenario("privatize_big_040"));
    assert_ne!(a.hash, b.hash, "0.35 and 0.40 must be different runs");
    assert!(b.before - b.after > a.before - a.after, "0.40 must close more than 0.35");
}

/// Why raising the 0.40 ceiling cannot be caught, stated as an assertion rather
/// than a paragraph. On this fixture the pass closes 57 of its 142 candidates
/// at a bias of 0.40 -- and also 57 at 0.45 and at 0.50, because the
/// `chance(0.5)` coin and the falling live degrees cap it there. A 0.50 ceiling
/// would therefore behave identically for every input, on every graph built
/// here. Measured, not assumed.
#[test]
fn the_bias_ceiling_saturates() {
    priv_big("privatize_big_090", 555, 0.0, 0.90);
    let (clamped, at_ceiling) = (scenario("privatize_big_090"), scenario("privatize_big_040"));
    assert_eq!(clamped.hash, at_ceiling.hash, "0.90 must clamp to 0.40");
    assert_eq!(at_ceiling.before - at_ceiling.after, 57, "the measured cap");
    assert_eq!(crate::geom::js_round(142.0 * 0.40), 57.0, "0.40 asks for exactly the cap");
    assert!(crate::geom::js_round(142.0 * 0.45) > 57.0, "0.45 would ask for more than it");
}

// ----------------------------------------------------------- clearFortZone --

fn ring_poly(cx: f64, cy: f64, rx: f64, ry: f64, n: usize) -> Vec<Vec2> {
    (0..n)
        .map(|i| {
            let a = 2.0 * std::f64::consts::PI * i as f64 / n as f64;
            Vec2::new(cx + rx * crate::geom::js_cos(a), cy + ry * crate::geom::js_sin(a))
        })
        .collect()
}

fn fort_fixture() -> (Vec<Vec<Vec2>>, Vec<Option<Vec2>>) {
    let mut polys = Vec::new();
    // A 0.5 m radial ladder along +x, where the land arc's own vertex sits at
    // (1230, 620): probe k is exactly 8 + k/2 metres outside the ring, so any
    // one-metre change in `clear_dist` moves two probes across the band edge.
    // Without this every fort constant survives a one-unit mutation, because
    // nothing in a realistic fixture happens to sit inside a 1 m shell.
    for k in 0..=184usize {
        let d = 8.0 + k as f64 * 0.5;
        let (cx, cy) = (1230.0 + d, 620.0);
        polys.push(vec![
            Vec2::new(cx - 0.4, cy - 0.4),
            Vec2::new(cx + 0.4, cy - 0.4),
            Vec2::new(cx + 0.4, cy + 0.4),
            Vec2::new(cx - 0.4, cy + 0.4),
        ]);
    }
    // A long spar straddling the enceinte: its centroid is 10 m outside the
    // ring (inside every clear band here) while BOTH ends are out of reach --
    // one inside the ring, one 100 m past the widest band. Only the centroid
    // half of `polyInClear` can sweep it.
    polys.push(vec![
        Vec2::new(1170.0, 617.0),
        Vec2::new(1310.0, 617.0),
        Vec2::new(1310.0, 623.0),
        Vec2::new(1170.0, 623.0),
    ]);
    for i in 0..40usize {
        let a = 2.0 * std::f64::consts::PI * i as f64 / 40.0;
        let rr = 300.0 + (i % 20) as f64 * 12.0;
        let cx = 850.0 + rr * crate::geom::js_cos(a);
        let cy = 620.0 + rr * 0.8 * crate::geom::js_sin(a);
        polys.push(vec![
            Vec2::new(cx - 6.0, cy - 5.0),
            Vec2::new(cx + 6.0, cy - 5.0),
            Vec2::new(cx + 6.0, cy + 5.0),
            Vec2::new(cx - 6.0, cy + 5.0),
        ]);
    }
    let details: Vec<Option<Vec2>> =
        golden::FORT_DETAIL_PTS.iter().map(|o| o.map(|(x, y)| Vec2::new(x, y))).collect();
    (polys, details)
}

fn fort_golden(name: &str) -> &'static golden::Fort {
    golden::FORTS.iter().find(|f| f.name == name).unwrap_or_else(|| panic!("no fort {name}"))
}

fn fort_scenario(name: &str, style: &str, glacis_off: Option<f64>, gates: Vec<Gate>) {
    let ring = ring_poly(850.0, 620.0, 380.0, 300.0, 24);
    let land_arc: Vec<Vec2> = ring[..13].to_vec();
    let wall = WallState {
        ring: Some(ring),
        gates,
        epoch: 0,
        land_arc: Some(land_arc),
        generation: None,
        history: Vec::new(),
    };
    let mut g = Graph::new();
    grid(&mut g, 350.0, 250.0, 7, 6, 120.0, 110.0, "street", 6.0, 1, true);
    g.add_street(850.0, 100.0, 850.0, 1150.0, "primary", 9.0, 0, "radial");
    g.add_street(400.0, 950.0, 1300.0, 950.0, "quay", 5.0, 0, "quay");
    // A fine angular ladder of roads crossing the ring beside the lower gate,
    // so both gate-corridor radii -- `clear_dist + 16` for a primary,
    // `clear_dist * 0.85` for anything else -- have a crossing within a metre.
    for k in 0..44usize {
        let ang = std::f64::consts::PI / 2.0 + (k as f64 - 22.0) * 0.0032;
        let (c, sn) = (crate::geom::js_cos(ang), crate::geom::js_sin(ang));
        let primary = k % 4 == 0;
        g.add_street(
            850.0 + 380.0 * c * 0.86,
            620.0 + 300.0 * sn * 0.86,
            850.0 + 380.0 * c * 1.16,
            620.0 + 300.0 * sn * 1.16,
            if primary { "primary" } else { "street" },
            if primary { 9.0 } else { 5.0 },
            2,
            "ladder",
        );
    }
    let (polys, details) = fort_fixture();
    let before = alive(&g);

    let sweep = clear_fort_zone(&wall, style, glacis_off, &mut g, &polys, &polys, &details);
    check(name, &g, before);

    let fg = fort_golden(name);
    assert_eq!(sweep.buildings_removed, fg.buildings_removed, "{name}: buildings swept");
    assert_eq!(sweep.parcels_cleared, fg.parcels_cleared, "{name}: parcels cleared");
    assert_eq!(sweep.details_removed.len(), fg.details_removed, "{name}: details swept");

    // Apply the report the way a caller must — descending, so the indices stay
    // valid — and confirm the survivors are the ones the reference kept.
    let mut kept = polys.clone();
    for &i in &sweep.buildings_removed {
        kept.remove(i);
    }
    assert_eq!(kept.len(), polys.len() - fg.buildings_removed.len());
    assert!(
        sweep.buildings_removed.windows(2).all(|w| w[0] > w[1]),
        "{name}: buildings_removed must be descending or a caller's remove() corrupts"
    );
    assert!(
        sweep.details_removed.windows(2).all(|w| w[0] > w[1]),
        "{name}: details_removed must be descending"
    );
}

const GATES: fn() -> Vec<Gate> = || {
    vec![
        Gate { pt: Vec2::new(850.0, 920.0), water: false, prov: String::new() },
        Gate { pt: Vec2::new(850.0, 320.0), water: false, prov: String::new() },
    ]
};

#[test]
fn fort_zone_sweep_matches() {
    fort_scenario("fort_wall", "wall", None, GATES());
}

#[test]
fn fort_zone_bastioned_uses_the_glacis_offset() {
    fort_scenario("fort_bastioned", "bastioned", Some(70.0), GATES());
    assert_eq!(fort_golden("fort_bastioned").clear_dist, 78.0);
    assert_eq!(fort_golden("fort_wall").clear_dist, 15.0);
}

/// `fort && fort.glacisOff || 60` — no fort object at all falls to 60, and so
/// does a glacis offset of literally zero, because `||` is falsy-tested. The
/// two must therefore be identical runs, and both must differ from the 70 one.
#[test]
fn a_missing_or_zero_glacis_offset_falls_back_to_sixty() {
    fort_scenario("fort_bastioned_nofort", "bastioned", None, GATES());
    fort_scenario("fort_bastioned_zerooff", "bastioned", Some(0.0), GATES());
    let (nf, zo, seventy) = (
        fort_golden("fort_bastioned_nofort"),
        fort_golden("fort_bastioned_zerooff"),
        fort_golden("fort_bastioned"),
    );
    assert_eq!(nf.clear_dist, 68.0);
    assert_eq!(zo.clear_dist, 68.0);
    assert_eq!(nf.buildings_removed, zo.buildings_removed, "0 and absent must agree");
    assert_ne!(
        nf.buildings_removed, seventy.buildings_removed,
        "the 60 fallback must differ from an explicit 70"
    );
}

/// With no gates at all, every road that crosses or grazes the enceinte dies —
/// the `checkPts.every(…)` over an empty gate list is vacuously false, not
/// vacuously true, because `gates.some(…)` on `[]` is false.
/// A deliberately enormous glacis. It exercises the whole sweep at a scale the
/// realistic fixtures do not reach -- 214 of the 226 probe polygons swept, and
/// every detail with an anchor -- and confirms the band really is unbounded
/// rather than clipped somewhere.
#[test]
fn a_huge_glacis_sweeps_the_whole_fixture() {
    fort_scenario("fort_huge_glacis", "bastioned", Some(1400.0), GATES());
    let f = fort_golden("fort_huge_glacis");
    assert_eq!(f.clear_dist, 1408.0);
    assert!(f.buildings_removed.len() > 200, "the huge band swept almost everything");
    assert!(f.details_removed > 0);
}

/// The gate corridor at **its own boundary**, and the only fixture in this file
/// that can resolve a 1 % change in `clearDist * 0.85`.
///
/// The finding it is built on is worth carrying forward: every ladder resolved
/// through *graph nodes* is capped at 11 m by [`crate::graph::Graph`]'s
/// `attach_point` snap, and 1 % of a realistic `clear_dist` (15 m, or 78 m for
/// a bastioned trace) is well under a metre. A **gate** is not a graph node, so
/// it can be placed to any precision — here 12.8 m from a crossing, which is
/// inside `15 * 0.86` (12.90) and outside `15 * 0.85` (12.75). The ring's
/// vertex 0 is exactly (1230, 620), so a horizontal road through it crosses
/// there and nowhere else.
#[test]
fn the_gate_corridor_radius_at_its_boundary() {
    let ring = ring_poly(850.0, 620.0, 380.0, 300.0, 24);
    assert_eq!(ring[0], Vec2::new(1230.0, 620.0), "the probe assumes this vertex");
    let wall = WallState {
        ring: Some(ring.clone()),
        gates: vec![
            Gate { pt: Vec2::new(1242.8, 620.0), water: false, prov: String::new() },
            Gate { pt: Vec2::new(850.0, 288.5), water: false, prov: String::new() },
        ],
        epoch: 0,
        land_arc: Some(ring[..13].to_vec()),
        generation: None,
        history: Vec::new(),
    };
    let mut g = Graph::new();
    grid(&mut g, 350.0, 250.0, 7, 6, 120.0, 110.0, "street", 6.0, 1, true);
    g.add_street(1100.0, 620.0, 1400.0, 620.0, "street", 5.0, 2, "probe-street");
    g.add_street(850.0, 500.0, 850.0, 100.0, "primary", 9.0, 2, "probe-primary");
    let before = alive(&g);
    let sweep = clear_fort_zone(&wall, "wall", None, &mut g, &[], &[], &[]);
    check("fort_corridor_probe", &g, before);
    assert_eq!(sweep, FortZoneSweep { edges_killed: sweep.edges_killed, ..Default::default() });
    assert!(sweep.edges_killed > 0, "the probe swept no road at all");
    // 12.8 is outside 0.85 x 15 and inside 0.86 x 15 — the whole point. Read
    // through variables so clippy sees a comparison rather than a constant.
    let (probe, clear) = (12.8f64, 15.0f64);
    assert!(probe > clear * 0.85, "the probe gate must NOT cover the crossing at 0.85");
    assert!(probe < clear * 0.86, "and must cover it at 0.86");
}

#[test]
fn fort_zone_with_no_gates_severs_every_crossing() {
    fort_scenario("fort_nogates", "wall", None, Vec::new());
    let (with, without) = (scenario("fort_wall"), scenario("fort_nogates"));
    assert!(without.after < with.after, "gates must save roads");
}

/// `if (!wallState.ring || !wallState.landArc) return` — an unwalled town is
/// left completely alone, graph and collections both.
#[test]
fn fort_zone_without_a_ring_is_a_no_op() {
    let wall = WallState {
        ring: None,
        gates: GATES(),
        epoch: 0,
        land_arc: None,
        generation: None,
        history: Vec::new(),
    };
    let mut g = Graph::new();
    grid(&mut g, 350.0, 250.0, 7, 6, 120.0, 110.0, "street", 6.0, 1, true);
    let snapshot = canon(&g);
    let (polys, details) = fort_fixture();
    let sweep = clear_fort_zone(&wall, "bastioned", Some(70.0), &mut g, &polys, &polys, &details);
    assert_eq!(canon(&g), snapshot);
    assert_eq!(sweep, FortZoneSweep::default());
}

// --------------------------------------------------------------- lanePass --

fn dry_site() -> crate::site::Site {
    build_site(909, 1700.0, 1250.0, "landlocked", SiteOpts::default())
}

fn lane_scenario(
    name: &str,
    build: impl FnOnce(&mut Graph),
    market: Vec2,
    epoch: i32,
    min_area: Option<f64>,
    site: &crate::site::Site,
) {
    let mut g = Graph::new();
    build(&mut g);
    let anchors = Anchors { market, ..place_anchors_stub() };
    let before = alive(&g);
    let added = lane_pass(2468, site, &anchors, &mut g, epoch, min_area);
    check(name, &g, before);
    assert_eq!(added, lane_added(name), "{name}: lanes laid");
}

/// `lanePass` reads exactly one field off `anchors`, so the rest is filler.
fn place_anchors_stub() -> Anchors {
    let site = dry_site();
    crate::routes::place_anchors(1, &site)
}

#[test]
fn lane_pass_splits_oversized_central_blocks() {
    let site = dry_site();
    lane_scenario(
        "lane_grid_default",
        |g| grid(g, 100.0, 100.0, 6, 5, 200.0, 180.0, "street", 6.0, 1, true),
        Vec2::new(400.0, 300.0),
        3,
        None,
        &site,
    );
    assert!(lane_added("lane_grid_default") > 0, "no lane was laid at all");
    // The lanes really are `'lane'`, 2.6 m wide, and stamped with the epoch.
    let mut g = Graph::new();
    grid(&mut g, 100.0, 100.0, 6, 5, 200.0, 180.0, "street", 6.0, 1, true);
    let anchors = Anchors { market: Vec2::new(400.0, 300.0), ..place_anchors_stub() };
    lane_pass(2468, &site, &anchors, &mut g, 3, None);
    let lanes: Vec<_> = g.edges.iter().filter(|e| e.alive && e.cls == "lane").collect();
    assert!(!lanes.is_empty(), "no edge came out classed 'lane'");
    assert!(lanes.iter().all(|e| e.w == 2.6 && e.epoch == 3));
    assert!(lanes.iter().all(|e| e.prov == LANE_PROV));
}

/// The stream label is `'lanes/' + epoch`, so the same graph at a different
/// epoch draws different offsets and produces a different town.
#[test]
fn the_lane_substream_is_labelled_by_epoch() {
    let site = dry_site();
    lane_scenario(
        "lane_grid_epoch7",
        |g| grid(g, 100.0, 100.0, 6, 5, 200.0, 180.0, "street", 6.0, 1, true),
        Vec2::new(400.0, 300.0),
        7,
        None,
        &site,
    );
    assert_ne!(
        scenario("lane_grid_epoch7").hash,
        scenario("lane_grid_default").hash,
        "epoch 3 and epoch 7 produced identical graphs"
    );
}

/// `minArea` is a real parameter, not decoration: a grid of ~8 000 m² cells is
/// entirely below the 12 000 default and entirely above 6 000.
#[test]
fn min_area_gates_the_whole_pass() {
    let site = dry_site();
    lane_scenario(
        "lane_small_default",
        |g| grid(g, 200.0, 200.0, 8, 6, 100.0, 80.0, "street", 6.0, 1, true),
        Vec2::new(600.0, 440.0),
        3,
        None,
        &site,
    );
    lane_scenario(
        "lane_small_6000",
        |g| grid(g, 200.0, 200.0, 8, 6, 100.0, 80.0, "street", 6.0, 1, true),
        Vec2::new(600.0, 440.0),
        3,
        Some(6000.0),
        &site,
    );
    assert_eq!(lane_added("lane_small_default"), 0);
    assert!(lane_added("lane_small_6000") > 0);
}

/// The 140 000 m² ceiling **at its own boundary**: 400 × 350 is exactly
/// 140 000, and the test is `A > 140000`, so it is kept; 400 × 350.1 is not.
#[test]
fn the_area_ceiling_is_exclusive_at_exactly_140000() {
    let site = dry_site();
    lane_scenario(
        "lane_ceiling_exact",
        |g| rect(g, 400.0, 400.0, 400.0, 350.0),
        Vec2::new(600.0, 575.0),
        3,
        Some(1000.0),
        &site,
    );
    lane_scenario(
        "lane_ceiling_over",
        |g| rect(g, 400.0, 400.0, 400.0, 350.001),
        Vec2::new(600.0, 575.0),
        3,
        Some(1000.0),
        &site,
    );
    assert_eq!(lane_added("lane_ceiling_exact"), 1);
    assert_eq!(lane_added("lane_ceiling_over"), 0);
}

/// The 12 000 m² default floor at its own boundary, the same way: `A < minArea`
/// keeps an exactly-12 000 face and drops 11 990.
#[test]
fn the_default_min_area_is_inclusive_at_exactly_12000() {
    let site = dry_site();
    lane_scenario(
        "lane_minarea_exact",
        |g| rect(g, 400.0, 400.0, 100.0, 120.0),
        Vec2::new(450.0, 460.0),
        3,
        None,
        &site,
    );
    lane_scenario(
        "lane_minarea_under",
        |g| rect(g, 400.0, 400.0, 100.0, 119.9),
        Vec2::new(450.0, 460.0),
        3,
        None,
        &site,
    );
    assert_eq!(lane_added("lane_minarea_exact"), 1);
    assert_eq!(lane_added("lane_minarea_under"), 0);
}

/// The 520 m market radius at its own boundary: a centroid exactly 520 m away
/// is kept (`> 520` is false), 520.1 m is not.
#[test]
fn the_market_radius_is_exclusive_at_exactly_520() {
    let site = dry_site();
    lane_scenario(
        "lane_market_exact",
        |g| rect(g, 500.0, 200.0, 200.0, 200.0),
        Vec2::new(80.0, 300.0),
        3,
        Some(1000.0),
        &site,
    );
    lane_scenario(
        "lane_market_over",
        |g| rect(g, 500.0, 200.0, 200.0, 200.0),
        Vec2::new(79.9, 300.0),
        3,
        Some(1000.0),
        &site,
    );
    assert_eq!(lane_added("lane_market_exact"), 1);
    assert_eq!(lane_added("lane_market_over"), 0);
}

/// The 30 m minimum lane separation, at its own boundary. A 60 x 27 face's two
/// longest edges come out 27-ish metres apart at the offsets this seed draws
/// and no lane is laid; a 60 x 28 face clears the threshold.
#[test]
fn a_lane_shorter_than_thirty_metres_is_not_laid() {
    let site = dry_site();
    lane_scenario(
        "lane_sep_under",
        |g| rect(g, 400.0, 400.0, 60.0, 27.0),
        Vec2::new(430.0, 413.5),
        3,
        Some(100.0),
        &site,
    );
    lane_scenario(
        "lane_sep_over",
        |g| rect(g, 400.0, 400.0, 60.0, 28.0),
        Vec2::new(430.0, 414.0),
        3,
        Some(100.0),
        &site,
    );
    assert_eq!(lane_added("lane_sep_under"), 0);
    assert_eq!(lane_added("lane_sep_over"), 1);
}

/// The wet scan, on a river site whose channel runs under some of the
/// candidates — the fixture that pins the `t += 0.12` sweep.
///
/// It also **disproves** the thing this port initially wrote about that loop.
/// The accumulation `t += 0.12` and the closed form `i * 0.12` produce
/// bit-identical values for all nine samples, in V8 and in Rust alike; the
/// accumulation is kept because it is what the reference writes, not because a
/// rewrite would diverge. Measured, then asserted, rather than asserted from
/// the shape of the code.
#[test]
fn lanes_are_not_laid_across_water() {
    let site = build_site(909, 1700.0, 1250.0, "river", SiteOpts::default());
    lane_scenario(
        "lane_river_wet",
        |g| grid(g, 100.0, 100.0, 6, 5, 200.0, 180.0, "street", 6.0, 1, true),
        Vec2::new(800.0, 700.0),
        3,
        None,
        &site,
    );
    // The accumulation itself, asserted rather than described.
    let mut t = 0.0f64;
    let mut ts = Vec::new();
    while t <= 1.0 {
        ts.push(t);
        t += 0.12;
    }
    assert_eq!(ts.len(), 9, "the wet scan takes nine samples");
    assert_eq!(ts[8], 0.96);
    let closed: Vec<f64> = (0..9).map(|i| i as f64 * 0.12).collect();
    assert_eq!(ts, closed, "measured: the two forms agree bit for bit here");
    // …and the step itself is still load-bearing, which is why the equivalence
    // above is not a licence to touch it.
    let mut u = 0.0f64;
    let mut n = 0;
    while u <= 1.0 {
        n += 1;
        u += 0.13;
    }
    assert_ne!(n, 9, "a different step gives a different sample count");
}

// ------------------------------------------------------------- the ordering --

/// The sequence inside [`remove_water_crossings`] is load-bearing: the
/// real-water sweep re-tests `e.alive`, so it must see the base sweep's kills,
/// and [`prune_largest`] must run last or the fabric the sweeps orphan survives.
#[test]
fn water_crossings_prunes_last() {
    let site = build_site(4242, 1700.0, 1250.0, "coastal", SiteOpts::default());
    let mut g = Graph::new();
    grid(&mut g, 200.0, 200.0, 6, 5, 200.0, 160.0, "street", 6.0, 1, true);
    g.add_street(210.0, 640.0, 1600.0, 640.0, "primary", 9.0, 0, "p");
    g.add_street(250.0, 300.0, 250.0, 1100.0, "quay", 5.0, 0, "q");
    remove_water_crossings(&site, &mut g);
    let after = alive(&g);
    // Running the prune again must change nothing — proof it already ran.
    prune_largest(&mut g);
    assert_eq!(alive(&g), after, "remove_water_crossings did not prune last");
    assert!(after > 0);
}

/// The same property for [`clear_fort_zone`].
#[test]
fn fort_zone_prunes_last() {
    let ring = ring_poly(850.0, 620.0, 380.0, 300.0, 24);
    let wall = WallState {
        ring: Some(ring.clone()),
        gates: GATES(),
        epoch: 0,
        land_arc: Some(ring[..13].to_vec()),
        generation: None,
        history: Vec::new(),
    };
    let mut g = Graph::new();
    grid(&mut g, 350.0, 250.0, 7, 6, 120.0, 110.0, "street", 6.0, 1, true);
    g.add_street(850.0, 100.0, 850.0, 1150.0, "primary", 9.0, 0, "radial");
    let (polys, details) = fort_fixture();
    clear_fort_zone(&wall, "bastioned", Some(70.0), &mut g, &polys, &polys, &details);
    let after = alive(&g);
    prune_largest(&mut g);
    assert_eq!(alive(&g), after, "clear_fort_zone did not prune last");
    assert!(after > 0);
}
