//! Milestone 6's tests. **Every one of the 29 scenarios is golden** — market,
//! provenance string, every route polyline, the whole resulting street graph
//! and the spatial index are the reference engine's own output, captured by
//! slicing block 4 out of the frozen HTML and running it under a bare `vm`
//! context with no DOM.
//!
//! None of the three functions is on `UME`'s public export or its `_test` one,
//! so the capture adds them — with `buildSite` and `makeGraph`, which the
//! fixtures need — to the returned object by a single anchored replacement of
//! the `return {` line, asserted to match exactly once. The reference file is
//! never touched.
//!
//! Everything is compared **bit for bit** through [`f64::to_bits`]: no
//! tolerances anywhere, including on the market, which comes out of
//! [`js_cos`](crate::geom::js_cos) / [`js_sin`](crate::geom::js_sin), and the
//! route polylines, which come out of `astar` and therefore
//! [`js_hypot`](crate::geom::js_hypot).
//!
//! The spatial index is pinned by the reference's **own** `fnv1a` over its own
//! grid dump rather than cell by cell. Milestone 2 golden-tested the index
//! itself; restating 400-odd cells per scenario here would add nothing but
//! 40,000 lines, and a hash over the exact same canonical string is exactly as
//! strict.
//!
//! # The fixtures, and the constant each one exists for
//!
//! Milestone 5's rule — *build the fixtures out of the geometry under test* —
//! is what most of this set is. The geometry here is the site, so the sites are
//! real ones from [`build_site`](crate::site::build_site) rather than
//! hand-drawn rasters, and the pairs below are chosen to separate constants
//! that a single site cannot:
//!
//! - **`river7` / `riverthrough7` share a seed.** The two kinds share
//!   `placeAnchors`' `[60, 240]` candidate band but *not* its `120 m` preferred
//!   distance — the score's ternary tests `'river'` alone. No single-kind
//!   fixture set can see that.
//! - **`river7` / `river7altAnchor` share a site.** The only pair that
//!   separates the `'anchors'` substream's seed from the `'site'` one.
//! - **`bay5` / `coast5` / `atoll5` share a seed.** Milestone 5's finding: a bay
//!   draws one fewer number than a coast, so their `routeEnds` diverge; and an
//!   unrecognised kind takes the coastline branch under its own name, which is
//!   what makes `placeAnchors`' string comparisons observable.
//! - **`landlocked3` / `landlocked17`** reach the market reference's **third**
//!   `||` arm — the literal `{Wm*0.52, Hm*0.42}` — which milestone 5 wrote
//!   forward as live rather than defensive: a landlocked site has neither a
//!   bridge nor a quay.
//! - **`tinyBox` / `narrowBox`** straddle the 80 m margin. At 150 m every one of
//!   the 400 candidates is rejected, so `best` stays `null` and the market is
//!   the `{ref.x, ref.y - 120}` fallback; at 180 m a candidate survives.
//!   Nothing with a full-size box can reach that line.
//! - **`nanCost`** is an all-NaN heightfield, so every cost cell is NaN, no
//!   relaxation ever fires and `astar` returns `null` for **every** route end.
//!   It is the only fixture that reaches `if(!path)return`.
//! - **`pathsDistExactly1` / `pathsDist1p25`** straddle
//!   `V.dist(pts[0], M) > 1`. The obvious "one ulp above 1" fixture does *not*
//!   work: the path is a metre offset added to the market, and
//!   `(386.6 + 1.0000000000000002) - 386.6` is exactly `1.0`. A boundary
//!   fixture built by offsetting a large coordinate has to clear **that
//!   coordinate's** ulp, not the constant's.
//! - **`pathsExit` / `pathsFirstOut` / `pathsTooShort` / `pathsEmptyList`** are
//!   the four ways `buildPrimariesFromPaths` drops input, and the last three
//!   are the only fixtures that leave `g._fromPaths` **false**.
//! - **`marginWinner`** is scanned: a site whose *winning* candidate sits 80-110
//!   m from a box edge. `midBox` makes the 80 m margin reject candidates, but
//!   raising it only removes ones that were losing anyway, so the constant stays
//!   invisible until the winner itself is inside the band.
//! - **`libmSensitive` / `libmMarket`** are scanned too, and the pair is the
//!   finding: seed 42's winning candidate has an angle where V8's `cos`/`sin`
//!   and the platform's differ, and that is *not enough* — a one-ulp cos error
//!   times a 240 m arm is 2.4e-14 against a market coordinate whose own ulp is
//!   5.7e-14, so it usually rounds away. Seed 212 moves the market's *y*
//!   (through `sin`) and leaves its *x* alone, so `libmMarketCos` (seed 543)
//!   exists for `cos`: each of the two needs its own scanned seed.
//! - **`shortDtWater`** is the only fixture that separates `js_max` from
//!   `f64::max`: a truncated `dt` array makes `riverDist` NaN, and a *real*
//!   heightfield alongside it keeps the slope finite, so the score's
//!   `Math.max(0, rd - 260)` is the only NaN in it.
//! - **`pathsBoxEdge`** straddles the 6 m box tolerance on **all four sides**
//!   with -5 / -6 / -7 offsets. The four `6.0`s in `inBox` are four separate
//!   constants and only the side a path leaves through can see its own.
//! - **`pathsCrossing`** makes two injected roads cross, so `addPolylineStreet`
//!   has to split — the only paths fixture that exercises milestone 2's
//!   planarity machinery.
//!
//! Mutation results, including every reported survivor and the invariant it
//! rests on, are in `URBAN_MORPHOLOGY_SCOPE.md`.

use super::*;
use crate::rng::fnv1a;
use crate::site::{SiteOpts, TerrainCtx, WaterCtx, build_site};
use golden::{Case, TerrainSpec, WaterSpec};

mod golden;

/// Flat `[x, y, x, y, ...]` back into points.
fn pts(flat: &[f64]) -> Vec<Vec2> {
    assert_eq!(flat.len() % 2, 0, "a flat point list must have an even length");
    flat.chunks(2).map(|c| Vec2::new(c[0], c[1])).collect()
}

fn water_ctx(s: &WaterSpec) -> WaterCtx {
    WaterCtx {
        mask: s.mask.to_vec(),
        dt: s.dt.to_vec(),
        mw: s.mw,
        mh: s.mh,
        cell_m: s.cell_m,
        river_path: s.river_path.map(pts),
        river_width_m: Some(s.river_width_m),
        river_order: s.river_order,
        sea_lake_cells: s.sea_lake_cells,
    }
}

fn terrain_ctx(s: &TerrainSpec) -> TerrainCtx {
    TerrainCtx {
        grid: s.grid.to_vec(),
        mw: s.mw,
        mh: s.mh,
        cell_m: s.cell_m,
        h_min: s.h_min,
        h_max: s.h_max,
    }
}

/// The reference's own grid serialisation: non-empty cells only, keyed
/// `"cx:cy"`, sorted by that **string**, joined `key=id,id;…`. The port keys the
/// same partition with `(i64, i64)`, so the ordering is re-derived rather than
/// assumed to match numeric order — the same care milestone 2's `grid_sorted`
/// takes.
fn grid_hash(g: &Graph) -> (usize, u32) {
    let mut cells: Vec<(String, &Vec<usize>)> = g
        .grid
        .iter()
        .filter(|(_, ids)| !ids.is_empty())
        .map(|(k, ids)| (format!("{}:{}", k.0, k.1), ids))
        .collect();
    cells.sort_by(|a, b| a.0.cmp(&b.0));
    let dump = cells
        .iter()
        .map(|(k, ids)| {
            let joined =
                ids.iter().map(usize::to_string).collect::<Vec<_>>().join(",");
            format!("{k}={joined}")
        })
        .collect::<Vec<_>>()
        .join(";");
    (cells.len(), fnv1a(&dump))
}

/// Run one scenario end to end from its captured inputs.
fn run(c: &Case) -> (Anchors, Vec<Route>, Graph) {
    let opts = SiteOpts {
        water: c.water.map(water_ctx),
        terrain: c.terrain.map(terrain_ctx),
        economy: None,
    };
    let site = build_site(c.site_seed, c.wm, c.hm, c.kind, opts);

    // Milestone 5 golden-verified `build_site`; re-asserting two of its outputs
    // here is what says this scenario is testing the site the capture tested,
    // not merely a site.
    let ends: Vec<f64> = site.route_ends.iter().flat_map(|p| [p.x, p.y]).collect();
    assert_eq!(
        ends.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
        c.route_ends.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
        "{}: site.routeEnds",
        c.name
    );
    assert_eq!(site.bridge_pt.is_some(), c.has_bridge, "{}: site.bridgePt presence", c.name);
    assert_eq!(site.harbour.pt.is_some(), c.has_quay, "{}: site.harbour.pt presence", c.name);

    let anchors = place_anchors(c.anchor_seed, &site);
    let mut g = Graph::new();
    let routes = match c.paths {
        None => build_primaries(c.site_seed, &site, &anchors, &mut g),
        Some(paths) => {
            let ps: Vec<Vec<Vec2>> = paths.iter().map(|p| pts(p)).collect();
            build_primaries_from_paths(c.site_seed, &site, &anchors, &mut g, &ps)
        }
    };
    (anchors, routes, g)
}

fn eq_bits(got: f64, want: f64, what: &str) {
    assert_eq!(got.to_bits(), want.to_bits(), "{what}: got {got:?}, want {want:?}");
}

#[test]
fn golden_every_scenario_reproduces_the_reference_exactly() {
    for c in golden::GOLDEN {
        let (anchors, routes, g) = run(c);
        let what = c.name;

        eq_bits(anchors.market.x, c.market.0, &format!("{what}: market.x"));
        eq_bits(anchors.market.y, c.market.1, &format!("{what}: market.y"));
        assert_eq!(anchors.prov, c.prov, "{what}: market provenance");

        assert_eq!(routes.len(), c.route_pts.len(), "{what}: route count");
        for (i, (rt, want)) in routes.iter().zip(c.route_pts).enumerate() {
            let want = pts(want);
            assert_eq!(rt.pts.len(), want.len(), "{what}: route {i} point count");
            for (j, (a, b)) in rt.pts.iter().zip(&want).enumerate() {
                eq_bits(a.x, b.x, &format!("{what}: route {i} pt {j} x"));
                eq_bits(a.y, b.y, &format!("{what}: route {i} pt {j} y"));
            }
        }
        let got_i: Vec<i64> =
            routes.iter().map(|r| r.i.map_or(-1, |v| v as i64)).collect();
        assert_eq!(got_i, c.route_i, "{what}: route indices (-1 = the reference pushed no `i`)");

        assert_eq!(g.nodes.len(), c.nodes.len(), "{what}: node count (= the reference's nextN)");
        for (n, (x, y, adj)) in g.nodes.iter().zip(c.nodes) {
            eq_bits(n.x, *x, &format!("{what}: node {} x", n.id));
            eq_bits(n.y, *y, &format!("{what}: node {} y", n.id));
            assert_eq!(&n.adj, adj, "{what}: node {} adjacency", n.id);
        }

        assert_eq!(g.edges.len(), c.edges.len(), "{what}: edge count (= the reference's nextE)");
        for (e, &(a, b, cls, w, epoch, alive)) in g.edges.iter().zip(c.edges) {
            assert_eq!(
                (e.a, e.b, e.cls, e.w, e.epoch, e.alive),
                (a, b, cls, w, epoch, alive),
                "{what}: edge {}",
                e.id
            );
        }

        let (cells, hash) = grid_hash(&g);
        assert_eq!(cells, c.grid_cells, "{what}: populated grid cell count");
        assert_eq!(hash, c.grid_hash, "{what}: grid contents (fnv1a over the reference's own dump)");

        assert_eq!(g.from_paths, c.from_paths, "{what}: g._fromPaths");
    }
}

/// The Rust half of the capture's emptiness / shape gate. A truncated or
/// silently-emptied `golden.rs` still parses and still passes every `zip` above
/// — `zip` stops at the shorter side — so the suite has to assert its own
/// inputs are the right shape. Four subsystems in this project have shipped a
/// harness that produced silently empty output and passed every structural
/// check; this is the check that would have caught it.
#[test]
fn the_golden_file_is_the_shape_it_claims_to_be() {
    let all = golden::GOLDEN;
    assert!(all.len() >= 38, "only {} scenarios in the golden file", all.len());

    let primaries = all.iter().filter(|c| c.paths.is_none()).count();
    let from_paths = all.iter().filter(|c| c.paths.is_some()).count();
    assert!(primaries >= 18, "only {primaries} buildPrimaries scenarios");
    assert!(from_paths >= 12, "only {from_paths} buildPrimariesFromPaths scenarios");

    let edges: usize = all.iter().map(|c| c.edges.len()).sum();
    let route_pts: usize = all.iter().map(|c| c.route_pts.iter().map(|r| r.len() / 2).sum::<usize>()).sum();
    assert!(edges >= 500, "only {edges} edges in the whole golden file");
    assert!(route_pts >= 700, "only {route_pts} route points in the whole golden file");

    for c in all {
        assert!(c.market.0.is_finite() && c.market.1.is_finite(), "{}: market is not finite", c.name);
        assert!(c.prov.len() > 40, "{}: provenance string is empty", c.name);
        assert_eq!(c.route_pts.len(), c.route_i.len(), "{}: route arrays disagree", c.name);
        for r in c.route_pts {
            assert!(r.len() >= 4, "{}: a route has under 2 points", c.name);
        }
        for &(_, _, cls, w, epoch, _) in c.edges {
            assert_eq!((cls, w, epoch), ("primary", 7.0, 0), "{}: a non-primary edge", c.name);
        }
        assert_eq!(c.from_paths, c.paths.is_some() && !c.route_pts.is_empty(), "{}: _fromPaths", c.name);
    }

    // The specific behaviours the pairs above exist to pin, asserted on the
    // captured data itself so a re-capture that silently loses one fails here
    // rather than passing a weaker suite.
    let find = |n: &str| all.iter().find(|c| c.name == n).unwrap_or_else(|| panic!("scenario {n} is gone"));
    let tiny = find("tinyBox");
    assert_eq!(
        (tiny.market.0, tiny.market.1),
        (tiny.wm * 0.52, tiny.hm * 0.42 - 120.0),
        "tinyBox no longer takes the best-is-none market fallback"
    );
    for n in ["midBox", "midBoxRiver", "marginWinner"] {
        let c = find(n);
        assert_ne!(
            (c.market.0, c.market.1),
            (c.wm * 0.52, c.hm * 0.42 - 120.0),
            "{n} now takes the market fallback, so it no longer straddles the 80 m margin"
        );
    }
    // `shortDtWater` isolates `js_max`'s NaN rule: its `riverDist` is NaN, its
    // relief is real (so the slope is finite), and the score's
    // `Math.max(0, rd - 260)` is therefore the only NaN in it. Under JS
    // semantics that poisons every candidate and the market falls back; under
    // `f64::max` the NaN is absorbed, a candidate wins, and the market moves.
    // The routes must survive, or the slope field is NaN too and the fixture
    // is proving nothing.
    assert!(!find("shortDtWater").route_pts.is_empty(), "shortDtWater lost its routes");
    assert!(!find("landlocked3").has_bridge && !find("landlocked3").has_quay,
        "landlocked3 gained a bridge or a quay, so the third `||` arm is no longer live");
    assert!(find("river7").has_bridge, "river7 lost its bridgePt");
    assert!(find("coast5").has_quay, "coast5 lost its quay");
    assert_ne!(find("river7").market, find("riverthrough7").market,
        "river and riverthrough now produce the same market");
    assert_ne!(find("river7").market, find("river7altAnchor").market,
        "the anchors seed no longer changes anything");
    // Milestone 5's finding 4: a bay draws one fewer number than a coast (it
    // reuses its own indent centre instead of drawing a harbour abscissa), so
    // the two share a seed and still land on *different* endpoints — same
    // count, different values. `atoll5` shares the seed too and must match
    // `coast5` exactly, which is what says an unknown kind takes the coastline
    // branch rather than a branch of its own.
    assert_ne!(find("bay5").route_ends, find("coast5").route_ends,
        "bay and coast on one seed no longer diverge");
    assert_eq!(find("atoll5").route_ends, find("coast5").route_ends,
        "an unknown kind no longer takes the coastline branch");
    assert!(find("nanCost").route_pts.is_empty(), "nanCost no longer reaches astar's null path");
    assert_eq!(find("pathsDistExactly1").route_pts[0][0], find("pathsDistExactly1").market.0 + 1.0,
        "pathsDistExactly1 is being unshifted, so `> 1` now reads as `>= 1`");
    assert_eq!(find("pathsDist1p25").route_pts[0][0], find("pathsDist1p25").market.0,
        "pathsDist1p25 is not being unshifted, so the 1 m boundary is not straddled");
    assert!(find("pathsFirstOut").route_pts.is_empty(), "pathsFirstOut produced a route");
    assert!(find("pathsOnlyMarket").route_pts.is_empty(), "pathsOnlyMarket produced a route");
    assert!(find("pathsCrossing").nodes.len() >= 5, "pathsCrossing no longer splits");
    assert!(all.iter().any(|c| c.paths.is_some() && !c.from_paths), "no scenario leaves _fromPaths false");
    assert!(all.iter().any(|c| c.from_paths), "no scenario sets _fromPaths");
}

/// `place_anchors` consumes exactly two draws per candidate, **before** any of
/// the four rejection tests — so the substream advances by 800 regardless of
/// the site's shape.
///
/// Not a golden (nothing exports the draw count) but a real property, and one
/// milestone 16 will need when it reasons about `generate()`'s draw order:
/// re-drawing the sequence by hand must land on the same market.
#[test]
fn place_anchors_draws_exactly_800_regardless_of_the_site() {
    use crate::rng::stream;
    for c in golden::GOLDEN {
        let opts = SiteOpts {
            water: c.water.map(water_ctx),
            terrain: c.terrain.map(terrain_ctx),
            economy: None,
        };
        let site = build_site(c.site_seed, c.wm, c.hm, c.kind, opts);
        let river_kind = site.kind == "river" || site.kind == "riverthrough";
        let band = if river_kind { [60.0, 240.0] } else { [60.0, 180.0] };

        // Re-derive the 400 candidates from a fresh substream and find the best
        // by the reference's own rule, then check it against the real call.
        let mut r = stream(c.anchor_seed, "anchors");
        let reference = site
            .bridge_pt
            .or(site.harbour.pt)
            .unwrap_or_else(|| Vec2::new(site.wm * 0.52, site.hm * 0.42));
        let mut draws = 0usize;
        let mut best: Option<Vec2> = None;
        let mut bs = f64::NEG_INFINITY;
        for _ in 0..400 {
            let ang = r.range(-std::f64::consts::PI, 0.0);
            let d = r.range(band[0], band[1]);
            draws += 2;
            let p = Vec2::new(
                reference.x + crate::geom::js_cos(ang) * d,
                reference.y + crate::geom::js_sin(ang) * d,
            );
            if p.x < 80.0 || p.y < 80.0 || p.x > site.wm - 80.0 || p.y > site.hm - 80.0 {
                continue;
            }
            if site.is_water(p) {
                continue;
            }
            let rd = site.river_dist(p);
            if rd < site.river_w / 2.0 + 30.0 {
                continue;
            }
            let score = -(site.slope(p) * 4.0)
                - (d - (if site.kind == "river" { 120.0 } else { 100.0 })).abs() / 60.0
                - crate::geom::js_max(0.0, rd - 260.0) / 120.0;
            if score > bs {
                bs = score;
                best = Some(p);
            }
        }
        assert_eq!(draws, 800, "{}: the candidate loop is not 400 x 2 draws", c.name);
        let market = best.unwrap_or_else(|| Vec2::new(reference.x, reference.y - 120.0));
        eq_bits(market.x, c.market.0, &format!("{}: re-derived market.x", c.name));
        eq_bits(market.y, c.market.1, &format!("{}: re-derived market.y", c.name));
    }
}

/// Neither route builder touches the RNG.
///
/// Both take a `seed` and neither reads it — verified by grep over the
/// reference's two bodies and asserted here from the other side: running each
/// with a wildly different seed must produce a byte-identical graph. Milestone
/// 16 needs this when it reasons about `generate()`'s overall draw order.
#[test]
fn neither_route_builder_reads_its_seed() {
    for c in golden::GOLDEN {
        let opts = SiteOpts {
            water: c.water.map(water_ctx),
            terrain: c.terrain.map(terrain_ctx),
            economy: None,
        };
        let site = build_site(c.site_seed, c.wm, c.hm, c.kind, opts);
        let anchors = place_anchors(c.anchor_seed, &site);
        let mut a = Graph::new();
        let mut b = Graph::new();
        match c.paths {
            None => {
                build_primaries(c.site_seed, &site, &anchors, &mut a);
                build_primaries(c.site_seed.wrapping_add(0x9e37_79b9), &site, &anchors, &mut b);
            }
            Some(paths) => {
                let ps: Vec<Vec<Vec2>> = paths.iter().map(|p| pts(p)).collect();
                build_primaries_from_paths(c.site_seed, &site, &anchors, &mut a, &ps);
                build_primaries_from_paths(
                    c.site_seed.wrapping_add(0x9e37_79b9),
                    &site,
                    &anchors,
                    &mut b,
                    &ps,
                );
            }
        }
        assert_eq!(a.nodes, b.nodes, "{}: the seed changed the nodes", c.name);
        assert_eq!(a.edges, b.edges, "{}: the seed changed the edges", c.name);
    }
}

/// The trail reinforcement is **order-dependent by design**, and this is the
/// test that says so out loud.
///
/// `buildPrimaries` runs `astar` once per external route endpoint over a *copy*
/// of the cost raster with the already-used cells multiplied by `0.45`, so each
/// run inherits the previous run's exact cell set (Helbing 1997, the
/// reference's own citation). Reversing `site.routeEnds` must therefore change
/// the result on a site where the routes actually share cells — if it did not,
/// the reinforcement would be doing nothing and the `0.45` would be untested.
#[test]
fn reversing_the_route_ends_changes_the_town() {
    let mut differed = 0;
    for c in golden::GOLDEN.iter().filter(|c| c.paths.is_none() && c.route_pts.len() >= 3) {
        let opts = SiteOpts {
            water: c.water.map(water_ctx),
            terrain: c.terrain.map(terrain_ctx),
            economy: None,
        };
        let mut site = build_site(c.site_seed, c.wm, c.hm, c.kind, opts);
        let anchors = place_anchors(c.anchor_seed, &site);
        site.route_ends.reverse();
        let mut g = Graph::new();
        let routes = build_primaries(c.site_seed, &site, &anchors, &mut g);
        let flat: Vec<u64> =
            routes.iter().flat_map(|r| r.pts.iter().flat_map(|p| [p.x.to_bits(), p.y.to_bits()])).collect();
        let want: Vec<u64> = c
            .route_pts
            .iter()
            .rev()
            .flat_map(|r| r.iter().map(|v| v.to_bits()))
            .collect();
        if flat != want {
            differed += 1;
        }
    }
    assert!(
        differed >= 5,
        "reversing routeEnds changed only {differed} scenarios — the 0.45 reinforcement is not being exercised"
    );
}

/// The score's third term is **not observable on any site this engine can
/// build**, and this test is what says so rather than the mutation report
/// waving at it.
///
/// `- Math.max(0, rd - 260) / 120` penalises a candidate for sitting far out of
/// the flood band. But the candidate is drawn at most 240 m from the reference
/// point, and on every watered site that reference point sits **on** the water
/// — the bridge, or the quay — so `riverDist` can never exceed the draw. And on
/// a landlocked site the river is a dummy segment at `(-1e4, -1e4)`, so every
/// candidate's `rd` is around 14,000: the term is a large *constant* offset
/// that shifts every score equally and cannot move the argmax.
///
/// So the term is either identically zero or a near-constant, per site. Both
/// the `260` and the `0` inside the `js_max` are therefore genuine equivalent
/// mutants, and this asserts the invariant they rest on across every fixture
/// rather than asserting the dead branch.
#[test]
fn the_flood_band_penalty_is_dead_on_every_site_the_engine_builds() {
    use crate::rng::stream;
    let (mut zero_sites, mut constant_sites) = (0, 0);
    for c in golden::GOLDEN {
        let opts = SiteOpts {
            water: c.water.map(water_ctx),
            terrain: c.terrain.map(terrain_ctx),
            economy: None,
        };
        let site = build_site(c.site_seed, c.wm, c.hm, c.kind, opts);
        let river_kind = site.kind == "river" || site.kind == "riverthrough";
        let band = if river_kind { [60.0, 240.0] } else { [60.0, 180.0] };
        let reference = site
            .bridge_pt
            .or(site.harbour.pt)
            .unwrap_or_else(|| Vec2::new(site.wm * 0.52, site.hm * 0.42));

        let mut r = stream(c.anchor_seed, "anchors");
        let mut terms: Vec<f64> = Vec::new();
        for _ in 0..400 {
            let ang = r.range(-std::f64::consts::PI, 0.0);
            let d = r.range(band[0], band[1]);
            let p = Vec2::new(
                reference.x + crate::geom::js_cos(ang) * d,
                reference.y + crate::geom::js_sin(ang) * d,
            );
            if p.x < 80.0 || p.y < 80.0 || p.x > site.wm - 80.0 || p.y > site.hm - 80.0 {
                continue;
            }
            if site.is_water(p) {
                continue;
            }
            let rd = site.river_dist(p);
            if rd < site.river_w / 2.0 + 30.0 || rd.is_nan() {
                continue;
            }
            terms.push(crate::geom::js_max(0.0, rd - 260.0) / 120.0);
        }
        if terms.is_empty() {
            continue;
        }
        let max = terms.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let min = terms.iter().copied().fold(f64::INFINITY, f64::min);
        if max == 0.0 {
            zero_sites += 1;
        } else {
            // A landlocked dummy river: the term is huge and its spread across
            // candidates is small next to the 260 m the constant would have to
            // move to matter.
            assert!(
                min > 96.0 / 120.0,
                "{}: the flood-band penalty straddles its own threshold (min {min}, max {max}) —                  the 260 is observable after all and should be mutation-tested, not argued away",
                c.name
            );
            constant_sites += 1;
        }
    }
    assert!(zero_sites >= 15, "only {zero_sites} sites zero the penalty outright");
    assert!(constant_sites >= 4, "only {constant_sites} sites reach it as a constant");
}

