//! Milestone 10's tests.
//!
//! **Every wall in `golden.rs` is the reference engine's own**, captured by
//! slicing block 4 out of the frozen HTML and running it under a bare `vm`
//! context with no DOM. None of the seven functions ported here is on `UME`'s
//! public export or its `_test` one, so the capture adds them by a single
//! anchored replacement of the `return {` line, asserted to match exactly once;
//! a second anchored insertion — likewise asserted exactly once — reports the
//! gate list as it stands *before* the bastioned gate cap, so
//! [`the_golden_file_is_the_shape_it_claims_to_be`] can show the cap really
//! dropped a gate rather than merely producing few. The frozen file is never
//! written to.
//!
//! Everything is compared **bit for bit**: polylines by length, by the fnv1a of
//! their exact-64-bit dump, and (at 64 points or fewer) coordinate by
//! coordinate; provenance by the fnv1a of the string, which pins the em-dashes
//! and the `§` as surely as the wording.
//!
//! # The fixtures, and what each one exists for
//!
//! Milestone 5's rule — build the fixtures out of the geometry under test — is
//! most of this set:
//!
//! - **`riverTown` is the bank-following circuit** and the reference case for
//!   every wall on a channel: a land arc, a seven-point water wall, two spurs,
//!   a water-gate. `coastTown` / `bayTown` / `landlockedTown` are the other
//!   branch, where the hull never reaches the water and the curtain closes all
//!   round.
//! - **`riverHarbourMouth` puts the harbour on the town's own water frontage.**
//!   A probe run with no harbour yields the actual `_waterClosure`; the mouth
//!   goes on its midpoint, so the 48 m gap really splits the drawn wall in two.
//!   `riverHarbour` (the site's own harbour, which lands nowhere near the wall)
//!   and `riverHarbourFar` are the same town with the gap never firing, and
//!   `riverHarbourEmpty` is the empty-quay case, which the reference reads as
//!   "no drawn gap at all" because it tests `harbour.quay.length`.
//! - **`straitTown` and `islandTown` are the v1.04 needle guard**, and they are
//!   the reason the guard's exact aftermath is worth a test: it empties `water`
//!   but does not skip the block below it, so the closure ends up `Some([])`
//!   rather than `None` and a harbourless town gets `water_walls == [[]]` — an
//!   array holding one empty array. Both are asserted explicitly.
//! - **`bothBanksTown` is the bridge-town rule**, `far.len() > max(20, near ·
//!   0.32)`, reached by widening `bridgeheadDistance`/`bridgeheadProbability`
//!   until the far bank really fills. `riverThroughTown` reaches `spansWater`
//!   the other way, through `site.through`.
//! - **`pathsTown` / `pathsBigTown` set `g._fromPaths`**, which is the v1.01
//!   discount on bare degree-2 primary vertices. Without it the enceinte
//!   stretches along the arterial over empty land; the scope document's
//!   milestone 2, finding 2 names it, and these two are what hold it.
//! - **`ridgeTown` / `rollingTown` / `realRiverRidge` deflect the circuit onto
//!   crests**; `flatTown` has a real heightfield whose relief is 0.0072 field
//!   units, under the 0.01 floor, so the deflection block never engages at all.
//!   Milestone 7's warning — a raster-backed fixture must use a **normalised**
//!   heightfield — is why every grid here spans ~0.1 rather than metres.
//! - **The eight `fort*` cases plus nine `STAR` ones** cover the trace: 4, 5, 6,
//!   7, 8 and 9 bastions (both clamps of `max(4, min(9, round(arc/230)))`), wet
//!   and dry ditches, the canal-fed moat, the `doubleMoat` boundary from both
//!   sides at exactly 4 and exactly 5 bastions, and both of `applyStarFort`'s
//!   early returns, which no real town's circuit reaches.
//! - **`emptyGraph` is the refusal**: no primaries at all, so `builtMassHull`
//!   returns `null` and `buildWall` writes nothing.
//!
//! # The rasters travel as hashes
//!
//! 500 KB of `f64` literals would dwarf the rest of the golden, so each raster
//! is rebuilt here from the same closed-form expression the capture used and
//! checked against the fnv1a of the reference's own cells before any scenario
//! uses it. That makes a `js_sin` / `js_cos` / `js_exp` / `js_hypot` divergence
//! fail as itself rather than as an unexplained wall.
//!
//! # What the mutation sweep found
//!
//! Swept 2026-09-24: every constant, tuning comparator and boolean gate in
//! `fortify.rs`, one mutant at a time. The table — killed, survivors killed by
//! a fixture added for them, equivalent mutants with the reason, and the ones
//! still open — is in `URBAN_MORPHOLOGY_SCOPE.md` under milestone 10. The
//! fixtures written for the survivors are the last section of this file.

use super::*;
use crate::geom::{js_cos, js_exp, js_sin};
use crate::graph::Graph;
use crate::growth::{GrowOpts, RecordingWallBuilder, WallState, grow};
use crate::rng::fnv1a;
use crate::routes::{Anchors, build_primaries, build_primaries_from_paths, place_anchors};
use crate::rules::DEFAULT_RULES;
use crate::site::{Site, SiteOpts, TerrainCtx, WaterCtx, build_site};
use golden::{Case, PolySpec};

mod golden;

// ------------------------------------------------------------------ helpers --

/// Flat `[x, y, x, y, ...]` back into points.
fn pts(flat: &[f64]) -> Vec<Vec2> {
    assert_eq!(flat.len() % 2, 0, "a flat point list must have an even length");
    flat.chunks(2).map(|c| Vec2::new(c[0], c[1])).collect()
}

fn eq_bits(got: f64, want: f64, what: &str) {
    assert_eq!(got.to_bits(), want.to_bits(), "{what}: got {got:?}, want {want:?}");
}

/// The capture's canonical polyline serialisation: every coordinate as its
/// exact 64 bits.
fn poly_dump(p: &[Vec2]) -> String {
    p.iter()
        .map(|q| format!("{:016x},{:016x}", q.x.to_bits(), q.y.to_bits()))
        .collect::<Vec<_>>()
        .join(";")
}

fn check_poly(got: &[Vec2], want: &PolySpec, what: &str) {
    assert_eq!(got.len(), want.n, "{what}: length");
    assert_eq!(fnv1a(&poly_dump(got)), want.h, "{what}: exact-bit hash");
    if want.dumped {
        assert_eq!(want.pts.len(), got.len() * 2, "{what}: dumped coordinate count");
        for (i, q) in got.iter().enumerate() {
            eq_bits(q.x, want.pts[2 * i], &format!("{what}[{i}].x"));
            eq_bits(q.y, want.pts[2 * i + 1], &format!("{what}[{i}].y"));
        }
    }
}

/// Milestone 7's whole-graph serialisation, unchanged: this milestone re-asserts
/// the pre-wall graph so a failure below means "the wall diverged" rather than
/// "the town was not the town".
fn graph_dump(g: &Graph) -> String {
    let ns = g
        .nodes
        .iter()
        .map(|n| {
            let adj = n.adj.iter().map(usize::to_string).collect::<Vec<_>>().join(".");
            format!("{:016x},{:016x},{}", n.x.to_bits(), n.y.to_bits(), adj)
        })
        .collect::<Vec<_>>()
        .join(";");
    let es = g
        .edges
        .iter()
        .map(|e| {
            format!(
                "{},{},{},{:016x},{},{}",
                e.a,
                e.b,
                e.cls,
                e.w.to_bits(),
                e.epoch,
                u8::from(e.alive)
            )
        })
        .collect::<Vec<_>>()
        .join(";");
    format!("{ns}#{es}")
}

// ------------------------------------------------------------------ rasters --

const CELL: f64 = 25.0;
const MW: usize = 68;
const MH: usize = 50;

/// The capture's raster expressions, reproduced. Every transcendental goes
/// through the reference's own semantics, which is exactly what the hash gate
/// below checks.
fn water_raster(name: &str) -> (Vec<u8>, Vec<f64>) {
    let mut mask = vec![0u8; MW * MH];
    let mut dt = vec![0.0f64; MW * MH];
    for j in 0..MH {
        for i in 0..MW {
            let x = (i as f64 + 0.5) * CELL;
            let y = (j as f64 + 0.5) * CELL;
            let (wet, d) = match name {
                "waterRiver" => {
                    let d = (y - (780.0 + 46.0 * js_sin(x / 300.0))).abs();
                    (d < 24.0, d)
                }
                "waterCoast" => {
                    let s = 880.0 + 60.0 * js_sin(x / 520.0);
                    (y > s, (y - s).abs())
                }
                "waterStrait" => {
                    (y < 430.0 || y > 640.0, js_min((y - 430.0).abs(), (y - 640.0).abs()))
                }
                "waterIsland" => {
                    let d = js_hypot(x - 850.0, y - 600.0);
                    (d > 165.0, (d - 165.0).abs())
                }
                "waterLagoon" => {
                    let d = js_hypot(x - 1150.0, y - 830.0);
                    (d < 300.0, (d - 300.0).abs())
                }
                other => panic!("unknown water raster {other}"),
            };
            mask[j * MW + i] = u8::from(wet);
            dt[j * MW + i] = d / CELL;
        }
    }
    (mask, dt)
}

fn terrain_raster(name: &str) -> Vec<f64> {
    let mut grid = vec![0.0f64; MW * MH];
    for j in 0..MH {
        for i in 0..MW {
            let x = (i as f64 + 0.5) * CELL;
            let y = (j as f64 + 0.5) * CELL;
            grid[j * MW + i] = match name {
                "terrainRolling" => {
                    0.46 + 0.045 * js_sin(x / 410.0)
                        + 0.03 * js_cos(y / 360.0)
                        + 0.012 * js_sin((x + y) / 190.0)
                }
                "terrainRidge" => {
                    let t = (x - 900.0) * 0.6 + (y - 600.0) * 0.8;
                    0.40 + 0.22 * js_exp(-(t * t) / (2.0 * 210.0 * 210.0))
                        + 0.01 * js_sin(x / 300.0)
                }
                "terrainFlat" => 0.5 + 0.0021 * js_sin(x / 400.0) + 0.0015 * js_cos(y / 330.0),
                "terrainMid" => 0.5 + 0.005 * js_sin(x / 410.0) + 0.0022 * js_cos(y / 360.0),
                "terrainCrown" => {
                    let t = js_hypot(x - 390.0, y - 815.0) - 540.0;
                    0.40 + 0.22 * js_exp(-(t * t) / (2.0 * 110.0 * 110.0))
                }
                other => panic!("unknown terrain raster {other}"),
            };
        }
    }
    grid
}

fn water_ctx(name: &str) -> WaterCtx {
    let spec = golden::WATER_RASTERS
        .iter()
        .find(|w| w.name == name)
        .unwrap_or_else(|| panic!("no captured water raster {name}"));
    let (mask, dt) = water_raster(name);
    assert_eq!(
        fnv1a(&mask.iter().map(u8::to_string).collect::<Vec<_>>().join(",")),
        spec.mask_hash,
        "{name}: the rebuilt water mask is not the reference's"
    );
    assert_eq!(
        fnv1a(&dt.iter().map(|v| format!("{:016x}", v.to_bits())).collect::<Vec<_>>().join(",")),
        spec.dt_hash,
        "{name}: the rebuilt distance transform is not the reference's"
    );
    WaterCtx {
        mask,
        dt,
        mw: spec.mw,
        mh: spec.mh,
        cell_m: spec.cell_m,
        river_path: spec.river_path.map(pts),
        river_width_m: Some(spec.river_width_m),
        river_order: spec.river_order,
        sea_lake_cells: spec.sea_lake_cells,
    }
}

fn terrain_ctx(name: &str) -> TerrainCtx {
    let spec = golden::TERRAIN_RASTERS
        .iter()
        .find(|t| t.name == name)
        .unwrap_or_else(|| panic!("no captured terrain raster {name}"));
    let grid = terrain_raster(name);
    assert_eq!(
        fnv1a(&grid.iter().map(|v| format!("{:016x}", v.to_bits())).collect::<Vec<_>>().join(",")),
        spec.grid_hash,
        "{name}: the rebuilt heightfield is not the reference's"
    );
    // `Math.min(...grid)` / `Math.max(...grid)`, through the JS semantics.
    let h_min = grid.iter().copied().fold(f64::INFINITY, js_min);
    let h_max = grid.iter().copied().fold(f64::NEG_INFINITY, js_max);
    eq_bits(h_min, spec.h_min, &format!("{name}: hMin"));
    eq_bits(h_max, spec.h_max, &format!("{name}: hMax"));
    TerrainCtx { grid, mw: spec.mw, mh: spec.mh, cell_m: spec.cell_m, h_min, h_max }
}

// ---------------------------------------------------------------- scenarios --

/// Rebuild one scenario's town: `buildSite` → `placeAnchors` → a route builder
/// → `grow` with walls **off**, so the wall under test is built by one explicit
/// call rather than by the epoch loop.
fn town(c: &Case) -> (Site, Anchors, Graph, f64) {
    let opts = SiteOpts {
        water: c.water.map(water_ctx),
        terrain: c.terrain.map(terrain_ctx),
        economy: None,
    };
    let site = build_site(c.site_seed, c.wm, c.hm, c.kind, opts);
    let anchors = place_anchors(c.anchor_seed, &site);
    let mut g = Graph::new();
    if c.paths {
        let ps: Vec<Vec<Vec2>> = golden::INJECTED_PATHS.iter().map(|p| pts(p)).collect();
        build_primaries_from_paths(c.site_seed, &site, &anchors, &mut g, &ps);
    } else if !c.no_primaries {
        build_primaries(c.site_seed, &site, &anchors, &mut g);
    }
    let rules = c.rules.map(|(d, p)| {
        let mut r = DEFAULT_RULES;
        r.street.bridgehead_distance = d;
        r.street.bridgehead_probability = p;
        r
    });
    let mut ws = WallState::default();
    let grow_opts = GrowOpts {
        target_len: c.target_len,
        max_rf: c.max_rf,
        walls: false,
        wall_generations: false,
        settlement_age: None,
        harbour: None,
        rules,
        wall_style: None,
        fortified: false,
        wet_moat: false,
        pop: 0.0,
    };
    let mut recorder = RecordingWallBuilder::default();
    let placed = grow(
        c.grow_seed,
        &site,
        &anchors,
        &mut g,
        c.epochs,
        &mut ws,
        &grow_opts,
        &mut recorder,
    );
    assert!(recorder.calls.is_empty(), "{}: grow must not build a wall here", c.name);
    (site, anchors, g, placed)
}

fn check_fort(got: &Fort, want: &golden::FortSpec, what: &str) {
    check_poly(&got.trace, &want.trace, &format!("{what}.trace"));
    assert_eq!(got.bastions.len(), want.bastions.len(), "{what}: bastion count");
    for (i, (b, w)) in got.bastions.iter().zip(want.bastions).enumerate() {
        eq_bits(b.salient.x, w.salient.0, &format!("{what}.bastions[{i}].salient.x"));
        eq_bits(b.salient.y, w.salient.1, &format!("{what}.bastions[{i}].salient.y"));
        check_poly(&b.outline, &w.outline, &format!("{what}.bastions[{i}].outline"));
        assert_eq!(b.demi, w.demi, "{what}.bastions[{i}].demi");
    }
    assert_eq!(got.curtains.len(), want.curtains.len(), "{what}: curtain count");
    for (i, (cu, w)) in got.curtains.iter().zip(want.curtains).enumerate() {
        for (g, e, n) in [
            (cu.a.x, w.0, "a.x"),
            (cu.a.y, w.1, "a.y"),
            (cu.b.x, w.2, "b.x"),
            (cu.b.y, w.3, "b.y"),
            (cu.mid.x, w.4, "mid.x"),
            (cu.mid.y, w.5, "mid.y"),
        ] {
            eq_bits(g, e, &format!("{what}.curtains[{i}].{n}"));
        }
    }
    check_poly(&got.counterscarp, &want.counterscarp, &format!("{what}.counterscarp"));
    check_poly(&got.ditch.outer, &want.ditch_outer, &format!("{what}.ditch.outer"));
    check_poly(&got.ditch.inner, &want.ditch_inner, &format!("{what}.ditch.inner"));
    check_poly(&got.covered_way, &want.covered_way, &format!("{what}.coveredWay"));
    match (&got.outer_moat, &want.outer_moat) {
        (Some(m), Some((o, i))) => {
            check_poly(&m.outer, o, &format!("{what}.outerMoat.outer"));
            check_poly(&m.inner, i, &format!("{what}.outerMoat.inner"));
        }
        (None, None) => {}
        _ => panic!("{what}: outerMoat presence differs"),
    }
    check_poly(&got.glacis.outer, &want.glacis_outer, &format!("{what}.glacis.outer"));
    check_poly(&got.glacis.inner, &want.glacis_inner, &format!("{what}.glacis.inner"));
    eq_bits(got.glacis_off, want.glacis_off, &format!("{what}.glacisOff"));
    assert_eq!(got.ravelins.len(), want.ravelins.len(), "{what}: ravelin count");
    for (i, (r, w)) in got.ravelins.iter().zip(want.ravelins).enumerate() {
        check_poly(r, w, &format!("{what}.ravelins[{i}]"));
    }
    assert_eq!(got.wet_ditch, want.wet_ditch, "{what}.wetDitch");
    assert_eq!(got.double_moat, want.double_moat, "{what}.doubleMoat");
    assert_eq!(got.canal_fed, want.canal_fed, "{what}.canalFed");
    assert_eq!(fnv1a(&got.prov), want.prov_hash, "{what}.prov");
}

#[test]
fn golden_every_wall_reproduces_the_reference_exactly() {
    for c in golden::GOLDEN {
        let what = c.name;
        let (site, anchors, g, placed) = town(c);

        // Milestones 5, 6 and 7 golden-verified everything up to this point.
        eq_bits(anchors.market.x, c.market.0, &format!("{what}: market.x"));
        eq_bits(anchors.market.y, c.market.1, &format!("{what}: market.y"));
        assert_eq!(g.nodes.len(), c.pre_nodes, "{what}: pre-wall node count");
        assert_eq!(g.edges.len(), c.pre_edges, "{what}: pre-wall edge count");
        assert_eq!(fnv1a(&graph_dump(&g)), c.pre_hash, "{what}: pre-wall graph");
        eq_bits(placed, c.placed, &format!("{what}: placed length"));

        // The two intermediates, in their own right.
        match (built_mass_hull(&site, &anchors, &g), &c.bmh) {
            (Some(bm), Some((hull, spans))) => {
                check_poly(&bm.hull, hull, &format!("{what}: builtMassHull.hull"));
                assert_eq!(bm.spans_water, *spans, "{what}: builtMassHull.spansWater");
            }
            (None, None) => {}
            (a, b) => panic!("{what}: builtMassHull presence differs ({}, {})", a.is_some(), b.is_some()),
        }
        check_poly(&town_bank(&site, &anchors), &c.bank, &format!("{what}: townBank"));

        // The wall.
        let harbour = c.harbour.map(|h| HarbourFront {
            pt: Vec2::new(h.pt.0, h.pt.1),
            quay: pts(h.quay),
        });
        let opts = FortOpts {
            wall_style: c.wall_style.map(str::to_string),
            fortified: c.fortified,
            wet_moat: c.wet_moat,
        };
        let mut ws = WallState::default();
        let before = graph_dump(&g);
        build_wall(c.wall_seed, &site, &anchors, &g, &mut ws, c.ep, harbour.as_ref(), &opts);
        assert_eq!(graph_dump(&g), before, "{what}: buildWall must not touch the graph");

        assert_eq!(ws.ring.is_some(), c.built, "{what}: a circuit was/was not built");
        if !c.built {
            // A refusal leaves the state exactly as it found it — and since the
            // integration pass folded the nine staged fields onto `WallState`,
            // this one comparison now covers all fifteen rather than six.
            assert_eq!(ws, WallState::default(), "{what}: a refusal must write nothing");
            continue;
        }

        check_poly(ws.ring.as_deref().expect("built"), c.ring.as_ref().expect("built"), &format!("{what}: ring"));
        check_poly(
            ws.land_arc.as_deref().expect("built"),
            c.land_arc.as_ref().expect("built"),
            &format!("{what}: landArc"),
        );
        assert_eq!(ws.epoch, c.ep, "{what}: epoch");

        let want_ww = c.water_walls.expect("built");
        assert_eq!(ws.water_walls.len(), want_ww.len(), "{what}: waterWalls run count");
        for (i, (run, w)) in ws.water_walls.iter().zip(want_ww).enumerate() {
            check_poly(run, w, &format!("{what}: waterWalls[{i}]"));
        }
        let want_spurs = c.spurs.expect("built");
        assert_eq!(ws.spurs.len(), want_spurs.len(), "{what}: spur count");
        for (i, (s, w)) in ws.spurs.iter().zip(want_spurs).enumerate() {
            eq_bits(s.a.x, w.0, &format!("{what}: spurs[{i}].a.x"));
            eq_bits(s.a.y, w.1, &format!("{what}: spurs[{i}].a.y"));
            eq_bits(s.b.x, w.2, &format!("{what}: spurs[{i}].b.x"));
            eq_bits(s.b.y, w.3, &format!("{what}: spurs[{i}].b.y"));
            assert_eq!(fnv1a(s.prov), w.4, "{what}: spurs[{i}].prov");
        }
        assert_eq!(Some(ws.spans_water), c.spans_water, "{what}: spansWater");
        assert_eq!(Some(ws.style.as_str()), c.style, "{what}: style");
        assert_eq!(Some(fnv1a(&ws.prov)), c.prov_hash, "{what}: prov");
        let cen = ws.centroid.expect("built");
        eq_bits(cen.x, c.centroid.expect("built").0, &format!("{what}: centroid.x"));
        eq_bits(cen.y, c.centroid.expect("built").1, &format!("{what}: centroid.y"));
        assert_eq!(Some(ws.terrain_deflected), c.terrain_deflected, "{what}: terrainDeflected");
        match (&ws.water_closure, &c.water_closure) {
            (Some(w), Some(spec)) => check_poly(w, spec, &format!("{what}: _waterClosure")),
            (None, None) => {}
            _ => panic!("{what}: _waterClosure presence differs"),
        }

        assert_eq!(ws.gates.len(), c.gates.len(), "{what}: gate count");
        for (i, (gt, w)) in ws.gates.iter().zip(c.gates).enumerate() {
            eq_bits(gt.pt.x, w.0, &format!("{what}: gates[{i}].x"));
            eq_bits(gt.pt.y, w.1, &format!("{what}: gates[{i}].y"));
            assert_eq!(gt.water, w.2, "{what}: gates[{i}].water");
            assert_eq!(fnv1a(&gt.prov), w.3, "{what}: gates[{i}].prov");
        }

        match (&ws.fort, c.fort) {
            (Some(f), Some(w)) => check_fort(f, w, &format!("{what}: fort")),
            (None, None) => {}
            _ => panic!("{what}: fort presence differs"),
        }
    }
}

#[test]
fn golden_apply_star_fort_on_hand_built_rings() {
    for s in golden::STAR {
        let what = s.name;
        let site = build_site(s.site_seed, 1700.0, 1250.0, s.site_kind, SiteOpts::default());
        // `style` is preset because `applyStarFort` is reached only through
        // `buildWall`, which has already tagged the circuit `'curtain'`; the
        // assertion below is that the trace overwrites that with `'bastioned'`.
        let mut ws = WallState {
            ring: Some(pts(s.ring_in)),
            gates: Vec::new(),
            epoch: 3,
            land_arc: None,
            generation: None,
            history: Vec::new(),
            style: "curtain".to_string(),
            ..WallState::default()
        };
        let opts = FortOpts { wall_style: None, fortified: true, wet_moat: s.wet_moat };
        apply_star_fort(s.seed, &site, &mut ws, &opts);

        assert_eq!(ws.style == "bastioned", s.applied, "{what}: did the trace apply?");
        if !s.applied {
            // Both early returns leave the ring exactly as they found it.
            assert_eq!(ws.ring.as_deref(), Some(pts(s.ring_in).as_slice()), "{what}: ring untouched");
            assert!(ws.land_arc.is_none(), "{what}: landArc untouched");
            assert!(ws.fort.is_none(), "{what}: no fort");
            continue;
        }
        check_poly(ws.ring.as_deref().expect("applied"), s.ring.as_ref().expect("applied"), &format!("{what}: ring"));
        check_poly(
            ws.land_arc.as_deref().expect("applied"),
            s.land_arc.as_ref().expect("applied"),
            &format!("{what}: landArc"),
        );
        let cen = ws.centroid.expect("applied");
        eq_bits(cen.x, s.centroid.expect("applied").0, &format!("{what}: centroid.x"));
        eq_bits(cen.y, s.centroid.expect("applied").1, &format!("{what}: centroid.y"));
        assert_eq!(Some(ws.water_walls.len()), s.water_walls, "{what}: waterWalls emptied");
        assert_eq!(Some(fnv1a(&ws.prov)), s.prov_hash, "{what}: prov");
        check_fort(ws.fort.as_ref().expect("applied"), s.fort.expect("applied"), &format!("{what}: fort"));
    }
}

// ------------------------------------------------------------- unit goldens --

/// The capture's own `densifyLoop` inputs, index-aligned with `golden::DENSIFY`.
fn densify_input(idx: usize) -> Vec<Vec2> {
    match idx {
        0 | 1 => pts(&[0., 0., 100., 0., 100., 60., 0., 60.]),
        2 => pts(&[10., 10., 13., 11., 11., 14.]),
        3 => pts(&[0., 0., 7., 0.]),
        4 => pts(&[5., 5.]),
        5 => Vec::new(),
        other => panic!("no densify input {other}"),
    }
}

#[test]
fn golden_densify_loop() {
    for d in golden::DENSIFY {
        let got = densify_loop(&densify_input(d.idx), d.step);
        check_poly(&got, &d.out, &format!("densifyLoop[{}]", d.idx));
    }
    // The capture's own shape, asserted rather than assumed: a square at step 8
    // really is resampled, and the step-longer-than-every-side case really is
    // one point per side.
    assert!(golden::DENSIFY[0].out.n > 30, "the resampled square is not resampled");
    assert_eq!(golden::DENSIFY[1].out.n, 4, "one sample per side");
    assert_eq!(golden::DENSIFY[5].out.n, 0, "an empty loop densifies to nothing");
}

/// The capture's own `nearestIdx` inputs.
fn nearest_input(idx: usize) -> Vec<Vec2> {
    match idx {
        0 | 1 => pts(&[0., 0., 10., 0., 10., 10., 0., 10.]),
        2 => pts(&[0., 0., 3., 4.]),
        3 => Vec::new(),
        4 => pts(&[0., 0., 1., 1.]),
        other => panic!("no nearest input {other}"),
    }
}

#[test]
fn golden_nearest_idx() {
    for n in golden::NEAREST {
        let got = nearest_idx(&nearest_input(n.idx), Vec2::new(n.p.0, n.p.1));
        assert_eq!(got, n.want, "nearestIdx[{}]", n.idx);
    }
    // The three behaviours worth naming: a four-way tie keeps the first index, an
    // empty list answers 0, and a NaN probe never beats `Infinity`.
    assert_eq!(golden::NEAREST[1].want, 0, "an equidistant tie keeps the first index");
    assert_eq!(golden::NEAREST[3].want, 0, "an empty list answers 0");
    assert_eq!(golden::NEAREST[4].want, 0, "a NaN distance never compares less");
}

/// The capture's own `cornerCut` inputs.
fn corner_input(idx: usize) -> Vec<Vec2> {
    match idx {
        0 | 1 => pts(&[0., 0., 100., 0., 100., 100., 0., 100.]),
        2 | 3 => pts(&[0., 0., 200., 0., 100., 8.]),
        4 => pts(&[0., 0., 50., 0., 50., 50.]),
        5 => pts(&[0., 0., 10., 0.]),
        6 => Vec::new(),
        other => panic!("no cornerCut input {other}"),
    }
}

#[test]
fn golden_corner_cut() {
    for c in golden::CORNER {
        let got = corner_cut(&corner_input(c.idx), c.min_ang, c.passes);
        check_poly(&got, &c.out, &format!("cornerCut[{}]", c.idx));
    }
    // A right angle is 1.5708 rad: under the engine's own 1.75 threshold and over
    // 1.0, so the same square answers differently on the two. That pins the
    // comparison direction, which no amount of hashing would.
    assert!(golden::CORNER[0].out.n > 4, "1.75 rad must cut a right angle");
    assert_eq!(golden::CORNER[1].out.n, 4, "1.0 rad must not");
}

#[test]
fn golden_js_acos_matches_v8_by_bulk_hash() {
    let mut dump = String::with_capacity(golden::ACOS_N * 17);
    for i in 0..golden::ACOS_N {
        if i > 0 {
            dump.push(',');
        }
        let x = -1.0 + (2.0 * i as f64) / (golden::ACOS_N as f64 - 1.0);
        dump.push_str(&format!("{:016x}", js_acos(x).to_bits()));
    }
    assert_eq!(fnv1a(&dump), golden::ACOS_HASH, "js_acos diverges from V8 somewhere in [-1, 1]");
}

#[test]
fn golden_js_acos_rows_and_domain_edges() {
    for &(x, want) in golden::ACOS_ROWS {
        eq_bits(js_acos(x), want, &format!("js_acos({x})"));
    }
    for &(x, want) in golden::ACOS_EDGE {
        let got = js_acos(x);
        if want.is_nan() {
            assert!(got.is_nan(), "js_acos({x}) should be NaN, got {got}");
        } else {
            eq_bits(got, want, &format!("js_acos({x})"));
        }
    }
    // The reason this function exists at all: the platform libm is *not* the
    // same function. If this ever stops finding disagreements the port is still
    // correct — but the claim in the module header would need re-measuring, so
    // it is asserted rather than believed.
    let disagreements =
        (0..40_000).filter(|&i| {
            let x = -1.0 + (2.0 * i as f64) / 39_999.0;
            js_acos(x).to_bits() != x.acos().to_bits()
        }).count();
    assert!(
        disagreements > 0,
        "js_acos and f64::acos agreed on all 40 000 arguments; re-measure the claim"
    );
}

// -------------------------------------------------------------- shape gates --

/// The emptiness / shape gate: four subsystems in this port have been bitten by
/// a golden that passed on an empty result, so what the fixtures *reach* is
/// asserted explicitly rather than assumed from the fact that they pass.
#[test]
fn the_golden_file_is_the_shape_it_claims_to_be() {
    let g = golden::GOLDEN;
    assert!(g.len() >= 30, "only {} scenarios", g.len());
    let by = |n: &str| g.iter().find(|c| c.name == n).unwrap_or_else(|| panic!("no case {n}"));

    let built = g.iter().filter(|c| c.built).count();
    assert!(built >= 25, "only {built} scenarios actually built a wall");
    assert!(g.iter().any(|c| !c.built), "no scenario reaches builtMassHull's refusal");
    assert!(!by("emptyGraph").built, "emptyGraph must not be walled");
    assert!(by("emptyGraph").bmh.is_none(), "emptyGraph's built mass must be None");

    // Both routes to `spansWater`, and the bridge-town one specifically.
    let spanning: Vec<&Case> = g.iter().filter(|c| c.spans_water == Some(true)).collect();
    assert!(spanning.len() >= 3, "only {} spanning circuits", spanning.len());
    assert!(
        spanning.iter().any(|c| c.kind == "river"),
        "no scenario reaches the bridge-town rule (a 'river' site spanning the water)"
    );
    assert!(
        spanning.iter().any(|c| c.kind == "riverthrough"),
        "no scenario reaches `site.through`"
    );
    // A spanning circuit has water-gates and no water wall at all.
    for c in &spanning {
        assert!(c.water_walls.is_some_and(<[PolySpec]>::is_empty), "{}: spanning waterWalls", c.name);
        assert!(c.spurs.is_some_and(<[_]>::is_empty), "{}: spanning spurs", c.name);
        assert!(c.gates.iter().any(|gt| gt.2), "{}: a spanning circuit needs a water-gate", c.name);
    }

    // The bank-following branch: a real water wall, two spurs, a closure.
    let bank_following: Vec<&Case> = g
        .iter()
        .filter(|c| c.spurs.is_some_and(|s| s.len() == 2))
        .collect();
    assert!(bank_following.len() >= 8, "only {} bank-following circuits", bank_following.len());
    assert!(
        by("riverTown").water_walls.expect("built").iter().any(|w| w.n > 1),
        "riverTown's drawn water wall is empty"
    );

    // The v1.04 needle guard's exact aftermath: `water` emptied but the block
    // below it not skipped, so the closure is `Some([])` and a harbourless town
    // gets one empty drawn run.
    let needled: Vec<&Case> = g
        .iter()
        .filter(|c| c.water_closure.as_ref().is_some_and(|w| w.n == 0))
        .collect();
    assert!(needled.len() >= 2, "only {} scenarios reach the v1.04 needle guard", needled.len());
    for c in &needled {
        assert_eq!(c.water_walls.expect("built").len(), 1, "{}: needle waterWalls", c.name);
        assert_eq!(c.water_walls.expect("built")[0].n, 0, "{}: needle waterWalls[0]", c.name);
        assert!(c.spurs.is_some_and(<[_]>::is_empty), "{}: needle spurs", c.name);
    }

    // The harbour mouth really splits a drawn wall in two.
    assert_eq!(
        by("riverHarbourMouth").water_walls.expect("built").len(),
        2,
        "the harbour gap did not split the water wall"
    );
    // ... and the same town without a mouth on the frontage keeps one run.
    assert_eq!(by("riverHarbourFar").water_walls.expect("built").len(), 1);
    // An empty quay is read as "no drawn gap", not as "no harbour": the same
    // single run, from the `else` arm.
    assert_eq!(by("riverHarbourEmpty").water_walls.expect("built").len(), 1);

    // The two `spurDepth` arms: a channel's `riverW/2 + 9` and a sea's flat 20 m.
    // `lagoonTown` is the only fixture in the set whose wall follows a **non-channel**
    // bank without tripping the needle guard, and it is why the 20 m arm is tested at
    // all -- a synthetic coast never gets its hull to the water.
    let lagoon = by("lagoonTown").spurs.expect("built");
    assert_eq!(lagoon.len(), 2, "lagoonTown must dip a spur at each end");
    let depth = Vec2::new(lagoon[0].0, lagoon[0].1).dist(Vec2::new(lagoon[0].2, lagoon[0].3));
    assert!((depth - 20.0).abs() < 1e-9, "the sea spur depth is 20 m, got {depth}");
    let river_spur = by("riverTown").spurs.expect("built");
    let rdepth =
        Vec2::new(river_spur[0].0, river_spur[0].1).dist(Vec2::new(river_spur[0].2, river_spur[0].3));
    // `riverW/2 + 9` on a seed-7 synthetic channel is 19.388 m — close enough to the
    // sea arm's flat 20 that only a fixture with a *real* sea bank separates them,
    // which is exactly what `lagoonTown` is for.
    assert!(
        (rdepth - 20.0).abs() > 1e-9,
        "the channel spur arm must not coincide with the sea one, got {rdepth}"
    );

    // Terrain deflection on and off, both with a real heightfield, and one site whose
    // relief lands *between* the 0.01 floor and twice it, so the floor's own value is
    // what decides.
    assert!(
        g.iter().filter(|c| c.terrain_deflected.is_some_and(|n| n > 0)).count() >= 3,
        "no scenario deflects the circuit onto higher ground"
    );
    assert_eq!(
        by("flatTown").terrain_deflected,
        Some(0),
        "flatTown must have a real heightfield and still not deflect"
    );
    let mid = golden::TERRAIN_RASTERS.iter().find(|t| t.name == "terrainMid").expect("terrainMid");
    let relief = mid.h_max - mid.h_min;
    assert!(
        (0.01..0.02).contains(&relief),
        "terrainMid's relief must straddle the 0.01 floor within one doubling, got {relief}"
    );
    assert!(
        by("midReliefTown").terrain_deflected.is_some_and(|n| n > 0),
        "midReliefTown must still deflect"
    );
    // Every ridge fixture picks the **inward** offsets, because a crest crossing the
    // town is nearer the centroid than the hull is. `crownTown` puts the crest in a
    // ring *outside* the built mass, which is the only way the positive offsets of
    // `[-60, -30, 30, 60]` are ever the best one.
    assert!(
        by("crownTown").terrain_deflected.is_some_and(|n| n >= 3),
        "crownTown must deflect outward onto the surrounding crest"
    );

    // The `cur.len() > 1` guard: a gap that leaves a single bank point on one flank
    // draws nothing there rather than a zero-length wall. Three mouths sit on a bank
    // vertex so that both the mid-loop and the end-of-loop guard fire.
    for n in ["riverHarbourDrop1", "riverHarbourDrop2", "riverHarbourDrop4"] {
        let ww = by(n).water_walls.expect("built");
        let drawn: usize = ww.iter().map(|w| w.n).sum();
        let closure = by(n).water_closure.as_ref().expect("built").n;
        assert!(ww.iter().all(|w| w.n > 1), "{n}: no drawn run may be a single point");
        assert!(
            drawn + 1 < closure,
            "{n}: the gap must swallow more than it draws ({drawn} of {closure})"
        );
    }

    // `_fromPaths`, which changes which nodes count as built mass at all.
    assert!(by("pathsTown").bmh.is_some() && by("pathsBigTown").bmh.is_some());

    // The style ternary: 'stone' and absent both fall through to 'curtain'.
    assert_eq!(by("styleStone").style, Some("curtain"));
    assert_eq!(by("riverTown").style, Some("curtain"));
    assert_eq!(by("stylePalisade").style, Some("palisade"));
    assert_eq!(by("styleDitch").style, Some("ditch"));
    // ... and a style tag changes nothing but the tag.
    assert_eq!(by("styleStone").ring.as_ref().map(|r| r.h), by("stylePalisade").ring.as_ref().map(|r| r.h));
    assert_eq!(by("riverTown").gates.len(), by("styleDitch").gates.len());

    // The bastioned trace.
    let forts: Vec<&Case> = g.iter().filter(|c| c.fort.is_some()).collect();
    assert!(forts.len() >= 6, "only {} bastioned circuits", forts.len());
    assert!(forts.iter().any(|c| c.fort.expect("fort").wet_ditch));
    assert!(forts.iter().any(|c| !c.fort.expect("fort").wet_ditch));
    assert!(forts.iter().any(|c| c.fort.expect("fort").canal_fed));
    assert!(forts.iter().all(|c| c.style == Some("bastioned")));
    // Every bastioned circuit's ring is the gorge — two points per bastion.
    for c in &forts {
        let f = c.fort.expect("fort");
        assert_eq!(c.ring.as_ref().expect("built").n, f.bastions.len() * 2, "{}: gorge", c.name);
        assert_eq!(c.land_arc.as_ref().expect("built").n, f.bastions.len() * 5, "{}: trace", c.name);
        assert!(c.water_walls.expect("built").is_empty(), "{}: the trace wraps every front", c.name);
    }
    // The gate cap really drops gates, and both its clamps are exercised.
    let capped = forts.iter().filter(|c| {
        let pre = c.pre_cap_gates.expect("built").iter().filter(|gt| !gt.2).count();
        pre > c.gates.iter().filter(|gt| !gt.2).count()
    }).count();
    assert!(capped >= 1, "the bastioned gate cap never dropped a gate in any scenario");
    let kept_counts: Vec<usize> =
        forts.iter().map(|c| c.gates.iter().filter(|gt| !gt.2).count()).collect();
    assert!(kept_counts.contains(&2), "no scenario keeps two land gates");
    assert!(kept_counts.contains(&3), "no scenario keeps three land gates");
    // `max(2, min(3, round(bastions / 3)))`: the **max** arm only binds under five
    // bastions (round(4/3) = 1), so a four-bastion town with more than one land gate
    // is the one fixture that can separate it. `min(3, …)` cannot be separated at all
    // — `nSeg` is capped at 9 and round(9/3) is 3 — which is asserted below rather
    // than left as a surviving mutant with no explanation.
    let four = forts.iter().find(|c| c.fort.expect("fort").bastions.len() == 4);
    let four = four.expect("no four-bastion fortified town: the cap's max(2) arm is untested");
    assert!(
        four.pre_cap_gates.expect("built").iter().filter(|gt| !gt.2).count() > 2,
        "{}: needs more than two land gates before the cap for max(2) to bind",
        four.name
    );
    assert_eq!(four.gates.iter().filter(|gt| !gt.2).count(), 2, "max(2, round(4/3)) is 2");
    for c in &forts {
        let n = c.fort.expect("fort").bastions.len();
        assert!(n <= 9, "{}: nSeg is capped at 9, so round(n/3) can never exceed 3", c.name);
    }

    // The star-fort units: both early returns, and the doubleMoat boundary from
    // both sides at exactly four and exactly five bastions.
    let s = golden::STAR;
    assert!(s.iter().filter(|f| !f.applied).count() >= 2, "neither early return is reached");
    let four = s.iter().find(|f| f.name == "fourWet").expect("fourWet");
    let five = s.iter().find(|f| f.name == "fiveWet").expect("fiveWet");
    assert_eq!(four.fort.expect("fort").bastions.len(), 4);
    assert_eq!(five.fort.expect("fort").bastions.len(), 5);
    assert!(four.fort.expect("fort").wet_ditch && !four.fort.expect("fort").double_moat);
    assert!(five.fort.expect("fort").wet_ditch && five.fort.expect("fort").double_moat);
    // `nSeg = max(4, min(9, round(arc / 230)))`: both clamps.
    assert!(s.iter().any(|f| f.fort.is_some_and(|x| x.bastions.len() == 4)), "the max(4) clamp");
    assert!(s.iter().any(|f| f.fort.is_some_and(|x| x.bastions.len() == 9)), "the min(9) clamp");
    // `outerMoat` is present exactly when `doubleMoat` is.
    for f in s.iter().filter_map(|f| f.fort) {
        assert_eq!(f.outer_moat.is_some(), f.double_moat, "outerMoat tracks doubleMoat");
    }
    // `minWaterD < 175` decides whether a ditch can be flooded (M-FOR-6). Two star
    // cases straddle it at 170.5 m and 180.5 m from the seed-7 river — close enough
    // that moving the constant either way flips one of them, which is the whole
    // point of building the fixture out of the geometry under test.
    let inside = s.iter().find(|f| f.name == "wetJustInside").expect("wetJustInside");
    let outside = s.iter().find(|f| f.name == "wetJustOutside").expect("wetJustOutside");
    assert!(inside.fort.expect("fort").wet_ditch, "170.5 m from water floods the ditch");
    assert!(!outside.fort.expect("fort").wet_ditch, "180.5 m from water does not");
    assert!(!inside.wet_moat && !outside.wet_moat, "neither may supply water explicitly");

    // The ravelin half-width is `min(32, curtainLen * 0.3)`. Every trace big enough
    // to be a town hits the 32 m cap, so `tinySquare` exists purely to make the
    // `* 0.3` arm the binding one.
    let tiny = s.iter().find(|f| f.name == "tinySquare").expect("tinySquare");
    let shortest = tiny
        .fort
        .expect("fort")
        .curtains
        .iter()
        .map(|c| Vec2::new(c.0, c.1).dist(Vec2::new(c.2, c.3)))
        .fold(f64::INFINITY, js_min);
    assert!(
        shortest * 0.3 < 32.0,
        "tinySquare's curtains ({shortest} m) must be short enough for the 0.3 arm to bind"
    );
    let big = s.iter().find(|f| f.name == "bigCircle").expect("bigCircle");
    let longest = big
        .fort
        .expect("fort")
        .curtains
        .iter()
        .map(|c| Vec2::new(c.0, c.1).dist(Vec2::new(c.2, c.3)))
        .fold(f64::NEG_INFINITY, js_max);
    assert!(longest * 0.3 > 32.0, "bigCircle's curtains must hit the 32 m cap");
}

// ----------------------------------------------------- targeted unit tests --

#[test]
fn ring_crossings_is_milestone_sevens_and_is_not_ported_again() {
    // The scope document's correction 4: `ringCrossings` (reference line 29631)
    // is nominally this milestone's first function and was ported forward by
    // milestone 7 because `grow` calls it. This asserts the shared one behaves
    // as `buildWall`'s gate loop needs — every crossing of a closed ring, in
    // ring-vertex order — rather than re-implementing it here.
    let ring = pts(&[0., 0., 100., 0., 100., 100., 0., 100.]);
    let hits = crate::growth::ring_crossings(&ring, Vec2::new(-20., 50.), Vec2::new(120., 50.));
    assert_eq!(hits.len(), 2, "a chord across a square crosses twice");
    assert_eq!((hits[0].x, hits[0].y), (100.0, 50.0), "the x=100 side is ring edge 1");
    assert_eq!((hits[1].x, hits[1].y), (0.0, 50.0), "the x=0 side is ring edge 3");
    assert!(
        crate::growth::ring_crossings(&ring, Vec2::new(10., 10.), Vec2::new(20., 20.)).is_empty(),
        "a segment wholly inside crosses nothing"
    );
}

#[test]
fn the_ringroad_class_already_existed() {
    // The scope document files `'ringroad'` under this milestone. It arrived a
    // milestone early -- `supersedeWall` is milestone 7's and lays the demolished
    // land arc with it -- so there was nothing to extend and no parallel enum was
    // created. Asserted from the graph's own side.
    let mut g = Graph::new();
    let made = g.add_polyline_street(
        &pts(&[100., 100., 300., 140., 500., 120.]),
        "ringroad",
        7.5,
        4,
        "fixture",
    );
    assert!(!made.is_empty(), "a ring road must actually lay edges");
    assert!(g.edges.iter().any(|e| e.cls == "ringroad"), "the class survives into the graph");
}

#[test]
fn densify_loop_takes_max_one_not_the_ceiling_alone() {
    // `Math.max(1, Math.ceil(d / step))`: a side shorter than the step still
    // contributes exactly its own start point. Mutating the 1 to a 0 would drop
    // short sides entirely, which the goldens catch -- but only because this
    // shape exists in them, so it is stated here too.
    let tiny = pts(&[0., 0., 1., 0., 1., 1., 0., 1.]);
    assert_eq!(densify_loop(&tiny, 50.0).len(), 4, "one sample per side at minimum");
    assert_eq!(densify_loop(&tiny, 50.0)[0], Vec2::new(0.0, 0.0));
}

#[test]
fn corner_cut_stops_early_when_a_pass_cuts_nothing() {
    // `if (!cut) break;` -- an obtuse ring is returned unchanged however many
    // passes are asked for, and the result is the input, not a copy of a copy.
    let obtuse = pts(&[0., 0., 100., 0., 150., 60., 100., 120., 0., 120., -50., 60.]);
    let once = corner_cut(&obtuse, 1.0, 1);
    let many = corner_cut(&obtuse, 1.0, 40);
    assert_eq!(once, obtuse, "nothing under 1.0 rad, so nothing is cut");
    assert_eq!(many, obtuse, "and further passes cannot change that");
    // Zero or negative passes run the loop zero times.
    assert_eq!(corner_cut(&obtuse, 3.2, 0), obtuse);
    assert_eq!(corner_cut(&obtuse, 3.2, -3), obtuse);
}

#[test]
fn town_bank_reads_site_kind_not_the_sites_own_river_like_flag() {
    // A real-water site with a river centreline and `kind === 'coast'` is
    // river-*like* to `buildSite` (`site.river_like()` is true) and a coast to
    // `townBank`, which recomputes `rk` from the kind string. The two disagree,
    // and reproducing the reference means reading the kind.
    let mut w = water_ctx("waterRiver");
    w.river_path = Some(pts(&[100., 600., 800., 640., 1600., 620.]));
    let site = build_site(7, 1700.0, 1250.0, "coast", SiteOpts { water: Some(w), ..SiteOpts::default() });
    assert!(site.river_like(), "a river path makes buildSite call this river-like");
    assert_eq!(site.kind, "coast", "but its kind is a coast");
    let anchors = place_anchors(7, &site);
    let bank = town_bank(&site, &anchors);
    // The coast branch offsets by 5 m, the channel branch by riverW/2 + 5 = 29.
    let d = bank[1].dist(site.river[1]);
    assert!((d - 5.0).abs() < 1e-9, "the coast branch offsets by 5 m, got {d}");
}

#[test]
fn built_mass_hull_needs_eight_junctions_and_counts_only_live_ones() {
    // The `near.length < 8` refusal, from both sides, and the degree-2 rule: a
    // node hanging off one edge is a street end, not a place.
    let site = build_site(5, 1700.0, 1250.0, "landlocked", SiteOpts::default());
    let anchors = place_anchors(5, &site);
    let m = anchors.market;
    let mut g = Graph::new();
    // A fan of spokes: every rim node has degree 1, the hub has many.
    for i in 0..12 {
        let a = 2.0 * PI * f64::from(i) / 12.0;
        let (x, y) = (m.x + js_cos(a) * 120.0, m.y + js_sin(a) * 120.0);
        g.add_street(m.x, m.y, x, y, "street", 4.0, 1, "fixture spoke");
    }
    assert!(
        built_mass_hull(&site, &anchors, &g).is_none(),
        "a fan of dead-end spokes has one junction, not eight"
    );
    // Close the rim: now every rim node has degree 3.
    let rim: Vec<Vec2> = (0..=12)
        .map(|i| {
            let a = 2.0 * PI * f64::from(i) / 12.0;
            Vec2::new(m.x + js_cos(a) * 120.0, m.y + js_sin(a) * 120.0)
        })
        .collect();
    g.add_polyline_street(&rim, "street", 4.0, 1, "fixture rim");
    let bm = built_mass_hull(&site, &anchors, &g).expect("a closed rim is built mass");
    assert!(bm.hull.len() >= 3, "the hull of a ring of junctions is a polygon");
    assert!(!bm.spans_water, "a landlocked site never spans water");
    // The 10 % + 16 m growth reserve really inflates: every hull vertex is
    // further from the market than the 120 m rim it came from.
    for p in &bm.hull {
        assert!(p.dist(m) > 120.0, "the hull is inflated past the built mass");
    }
}

/// A closed ring of `n` junctions at `r` metres around the market, on a landlocked
/// site — the smallest thing `built_mass_hull` will look at.
fn ring_town(n: usize, r: f64) -> (Site, Anchors, Graph) {
    let site = build_site(5, 1700.0, 1250.0, "landlocked", SiteOpts::default());
    let anchors = place_anchors(5, &site);
    let m = anchors.market;
    let mut g = Graph::new();
    let poly: Vec<Vec2> = (0..=n)
        .map(|i| {
            let a = 2.0 * PI * i as f64 / n as f64;
            Vec2::new(m.x + js_cos(a) * r, m.y + js_sin(a) * r)
        })
        .collect();
    g.add_polyline_street(&poly, "street", 4.0, 1, "fixture ring");
    (site, anchors, g)
}

#[test]
fn built_mass_hull_refuses_at_seven_junctions_and_accepts_at_eight() {
    // `near.length < 8` is an exact integer count, and no *grown* town in the golden
    // lands on it — milestone 7 hit the same wall with `interior.len() >= 8` and
    // recorded it as unclosable there because the count is an output of the growth
    // loop. Here it is an input, so the boundary is constructible and is constructed.
    let (site7, a7, g7) = ring_town(7, 150.0);
    assert_eq!(g7.nodes.iter().filter(|n| n.adj.len() >= 2).count(), 7, "exactly seven junctions");
    assert!(built_mass_hull(&site7, &a7, &g7).is_none(), "seven junctions is not a town");
    let (site8, a8, g8) = ring_town(8, 150.0);
    assert_eq!(g8.nodes.iter().filter(|n| n.adj.len() >= 2).count(), 8, "exactly eight junctions");
    assert!(built_mass_hull(&site8, &a8, &g8).is_some(), "eight junctions is");
}

#[test]
fn built_mass_hull_refuses_when_the_percentile_cut_drops_a_node() {
    // The **second** `< 8`, on `pts` rather than on `near`: eight junctions qualify,
    // then `ds[floor(n * 0.85)] * 1.12` cuts one of them out and the hull is refused
    // after all. Seven at 150 m and one at 400 m does it — the cut lands at 168 m.
    let site = build_site(5, 1700.0, 1250.0, "landlocked", SiteOpts::default());
    let anchors = place_anchors(5, &site);
    let m = anchors.market;
    let mut g = Graph::new();
    let ring: Vec<Vec2> = (0..=7)
        .map(|i| {
            let a = 2.0 * PI * i as f64 / 7.0;
            Vec2::new(m.x + js_cos(a) * 150.0, m.y + js_sin(a) * 150.0)
        })
        .collect();
    g.add_polyline_street(&ring, "street", 4.0, 1, "fixture ring");
    // The eighth junction, far out: a two-hop spur straight outward from a ring
    // vertex, so its **first** node has degree 2 and counts while its tip has degree
    // 1 and does not. Exactly one far junction is what the boundary needs -- two
    // would push `floor(n * 0.85)` onto a far distance and the cut would not bite.
    g.add_polyline_street(
        &[Vec2::new(m.x + 150.0, m.y), Vec2::new(m.x + 420.0, m.y), Vec2::new(m.x + 560.0, m.y)],
        "street",
        4.0,
        1,
        "fixture spur",
    );
    let near = g.nodes.iter().filter(|n| n.adj.len() >= 2).count();
    assert_eq!(near, 8, "the fixture must clear the first gate with exactly eight");
    assert!(
        built_mass_hull(&site, &anchors, &g).is_none(),
        "the 85th-percentile cut must drop the outliers and refuse"
    );
}

#[test]
fn the_closed_ring_corner_cut_is_a_no_op_after_two_chaikin_passes() {
    // Two of this milestone's four `cornerCut` call sites take a **closed** ring that
    // has just been through `chaikin(chaikin(x, true), true)`, and neither its
    // `minAng` nor its pass count can be mutated to any effect. That is not a fixture
    // gap: subdividing a convex hull twice leaves every interior angle well over
    // 1.75 rad, so the first pass cuts nothing and `if (!cut) break` ends it. Proved
    // over every built-mass hull the golden actually produced, rather than asserted.
    let mut checked = 0usize;
    for c in golden::GOLDEN {
        let Some((hull, _)) = &c.bmh else { continue };
        if !hull.dumped {
            continue;
        }
        let h = pts(hull.pts);
        for base in [h.clone(), crate::geom::convex_hull(&h)] {
            let smooth = chaikin(&chaikin(&base, true), true);
            assert_eq!(
                corner_cut(&smooth, 1.75, 4),
                smooth,
                "{}: a double-chaikin ring must survive cornerCut untouched",
                c.name
            );
            checked += 1;
        }
    }
    assert!(checked >= 40, "only {checked} hulls checked");
    // The **open** land arc is the opposite case, and is why the constant is not
    // simply dead: the densified hull run has sharp ends and really is cut.
    let run = pts(&[0., 0., 100., 2., 200., 0., 205., 90., 100., 96., 3., 92.]);
    assert_ne!(corner_cut(&run, 1.75, 3), run, "an unsmoothed run must be cut");
}

#[test]
fn a_bastioned_style_without_a_fort_reads_the_bastion_count_as_zero() {
    // **No golden path, and deliberately so.** `opts.wallStyle === 'bastioned'` with
    // `opts.fortified` false makes the reference read `wallState.fort.bastions` off
    // an undefined `fort` and throw, which would abort the whole of `generate()`.
    // Nothing produces that state — `_umWallSpec` returns only
    // none/ditch/palisade/stone — so the port substitutes the documented non-fatal
    // reading (`bastions.length` of an absent fort is 0, hence the `|| 6` default)
    // rather than panicking across the gdext boundary. This test is what pins that
    // choice, and what makes the `|| 6` testable at all.
    // `landlockedTown`'s circuit, which the golden shows carrying three land gates --
    // one more than the `|| 6` default's cap, which is what makes the default
    // observable at all.
    let site = build_site(5, 1700.0, 1250.0, "landlocked", SiteOpts::default());
    let anchors = place_anchors(5, &site);
    let mut g = Graph::new();
    build_primaries(5, &site, &anchors, &mut g);
    let mut ws = WallState::default();
    let opts = GrowOpts { target_len: 3600.0, max_rf: 430.0, ..GrowOpts::default() };
    grow(5, &site, &anchors, &mut g, 8, &mut ws, &opts, &mut RecordingWallBuilder::default());

    let mut plain = WallState::default();
    build_wall(5, &site, &anchors, &g, &mut plain, 9, None, &FortOpts::default());
    assert_eq!(
        plain.gates.iter().filter(|g| !g.water).count(),
        3,
        "the uncapped circuit must have more land gates than the cap allows"
    );

    let mut ws = WallState::default();
    build_wall(
        5,
        &site,
        &anchors,
        &g,
        &mut ws,
        9,
        None,
        &FortOpts {
            wall_style: Some("bastioned".to_string()),
            fortified: false,
            wet_moat: false,
        },
    );
    assert_eq!(ws.style, "bastioned", "the style tag is taken verbatim");
    assert!(ws.fort.is_none(), "no trace was applied, so there is no fort");
    // `max(2, min(3, round(6 / 3)))` == 2 land gates kept, one fewer than the three
    // the same circuit produces untagged.
    assert_eq!(ws.gates.iter().filter(|g| !g.water).count(), 2, "the || 6 default caps at two");
    // ... and the gates that survive carry the bastioned wording, because the style
    // tag — not the fort — is what the provenance branch reads.
    assert!(
        ws.gates.iter().filter(|g| !g.water).all(|g| g.prov == GATE_PROV_BASTIONED),
        "a bastioned style tags its gates as curtain gates"
    );
}

#[test]
fn a_refusal_leaves_the_previous_circuit_standing() {
    // `model.wall` means "the active, outermost circuit" throughout, and
    // `buildWall` overwrites in place -- so a call that refuses must not clear
    // what is already there. That is what makes `grow`'s wall-permeability test
    // safe across a failed rebuild.
    let site = build_site(5, 1700.0, 1250.0, "landlocked", SiteOpts::default());
    let anchors = place_anchors(5, &site);
    let g = Graph::new(); // no nodes at all: builtMassHull refuses
    let ring = pts(&[100., 100., 400., 100., 400., 400., 100., 400.]);
    // Every field the previous circuit could have carried is set to something
    // non-default, so `ws == before` below is a real fifteen-field assertion
    // rather than a comparison of mostly-empty values. Before the integration
    // pass the nine on the second half lived in a separate `WallExtras` and
    // were checked by a separate, weaker `== WallExtras::default()`.
    let mut ws = WallState {
        ring: Some(ring.clone()),
        gates: vec![Gate { pt: Vec2::new(250., 100.), water: false, prov: "g".into() }],
        epoch: 2,
        land_arc: Some(ring.clone()),
        generation: Some(1),
        history: Vec::new(),
        water_walls: vec![ring.clone()],
        spurs: vec![Spur { a: Vec2::new(1., 2.), b: Vec2::new(3., 4.), prov: SPUR_PROV }],
        spans_water: true,
        style: "curtain".to_string(),
        prov: WALL_PROV_BANK.to_string(),
        fort: None,
        centroid: Some(Vec2::new(250., 250.)),
        terrain_deflected: 7,
        water_closure: Some(ring.clone()),
    };
    let before = ws.clone();
    build_wall(7, &site, &anchors, &g, &mut ws, 9, None, &FortOpts::default());
    assert_eq!(ws, before, "a refusal must leave the standing circuit alone");
}

#[test]
fn the_builder_keeps_a_supersession_s_extra_fields() {
    // Milestone 7's warning: `supersedeWall` copies six of `buildWall`'s nine
    // extra fields into its history record, and adding them to `WallState`
    // without adding them to `WallGeneration` would produce a silently lossy
    // history that every structural test still passes.
    //
    // Milestone 10 answered that with a `history_extras` vector on the builder,
    // hand-kept index-aligned with `WallState::history`. The integration pass
    // deleted both: the six fields are on `WallGeneration` now and
    // `supersede_wall` copies them in the same statement as the other four, so
    // the record IS the alignment. This test now reads them off
    // `ws.history[0]`, which is what milestone 7 asked for in the first place —
    // and it is a stronger assertion, because a `WallGeneration` cannot exist
    // without them the way a missing `history_extras` push could.
    let site = build_site(7, 1700.0, 1250.0, "river", SiteOpts::default());
    let anchors = place_anchors(7, &site);
    let mut g = Graph::new();
    build_primaries(7, &site, &anchors, &mut g);
    let mut ws = WallState::default();
    let opts = GrowOpts {
        target_len: 3600.0,
        max_rf: 430.0,
        walls: false,
        ..GrowOpts::default()
    };
    let mut builder = FortificationBuilder;
    grow(7, &site, &anchors, &mut g, 8, &mut ws, &opts, &mut RecordingWallBuilder::default());

    // First circuit: nothing retired yet.
    builder.build_wall(7, &site, &anchors, &mut g, &mut ws, 9, &opts);
    assert!(ws.ring.is_some(), "the real builder must build a real circuit");
    assert!(ws.history.is_empty(), "nothing has been superseded");
    let first = ws.clone();
    assert!(!first.prov.is_empty() && !first.style.is_empty());

    // Supersede it: `supersede_wall` pushes the history record and then calls
    // the builder, so the retiring circuit's six extra fields must land in
    // `history[0]` — and be the values the FIRST circuit had, not the second's.
    crate::growth::supersede_wall(7, &site, &anchors, &mut g, &mut ws, 12, &opts, &mut builder);
    assert_eq!(ws.history.len(), 1, "one circuit retired");
    let h = &ws.history[0];
    assert_eq!(h.water_walls, first.water_walls, "waterWalls");
    assert_eq!(h.spurs, first.spurs, "spurs");
    assert_eq!(h.spans_water, first.spans_water, "spansWater");
    assert_eq!(h.style, first.style, "style");
    assert_eq!(h.prov, first.prov, "prov");
    assert_eq!(h.fort, first.fort, "fort");
    // ... and the four the reference's object literal also picks.
    assert_eq!(h.ring, first.ring, "ring");
    assert_eq!(h.gates, first.gates, "gates");
    assert_eq!(h.land_arc, first.land_arc, "landArc");
    assert_eq!(h.epoch, first.epoch, "epoch");
    // The three the literal deliberately omits are NOT on `WallGeneration` at
    // all, which is the half of milestone 7's warning that is now unforgeable:
    // `_waterClosure`, `centroid` and `terrainDeflected` cannot be read back off
    // a history record because the reference does not put them there.
}

#[test]
fn wet_moat_is_an_input_nothing_in_the_reference_supplies() {
    // **The name is milestone 10's claim, and it is wrong** -- kept only so the
    // test stays greppable across the integration pass that disproved it. That
    // milestone grepped for `opts.wetMoat` and found its two consumers (29998,
    // 29999) and no producer; but a producer spells the KEY, not the read.
    // Reference line 31017 is one:
    //
    //   if(walls)buildWall(seed,site,anchors,g,wallState,1,harbour,
    //     {fortified,wetMoat:profile.waterway,wallStyle:opts.wallStyle});
    //
    // on the `profile.planning === 'radial'` branch, where `VENUS.waterway` is
    // true (line 28209) -- so every fortified Venus town gets one, and line
    // 31063 reads `wallState.fort.canalFed` back. Rename this when the radial
    // branch is wired; until then what it asserts -- the behavioural difference
    // `wetMoat` makes -- is unaffected and is the reason to keep it.
    let site = build_site(5, 1700.0, 1250.0, "landlocked", SiteOpts::default());
    let ring: Vec<Vec2> = (0..6)
        .map(|i| {
            let a = 2.0 * PI * f64::from(i) / 6.0;
            Vec2::new(850.0 + 300.0 * js_cos(a), 600.0 + 300.0 * js_sin(a))
        })
        .collect();
    let run = |wet_moat: bool| {
        let mut ws = WallState { ring: Some(ring.clone()), ..WallState::default() };
        apply_star_fort(4, &site, &mut ws, &FortOpts { wet_moat, ..FortOpts::default() });
        ws.fort.expect("the hexagon applies")
    };
    let dry = run(false);
    let wet = run(true);
    assert!(!dry.wet_ditch && !dry.canal_fed && !dry.double_moat);
    assert!(wet.wet_ditch && wet.canal_fed && wet.double_moat, "landlocked + wetMoat is canal-fed");
    // The doubled moat is not free: it pushes the glacis out by ditchW * 0.9.
    // Written as the reference's own expression rather than as a difference,
    // because `97.8 - 78.0` is 19.799999999999997 and comparing that against a
    // literal 19.8 would be this test's rounding error, not the port's.
    eq_bits(dry.glacis_off, 22.0 + 8.0 + 48.0, "the plain glacis offset");
    eq_bits(wet.glacis_off, (22.0 + 8.0 + 22.0 * 0.9) + 48.0, "the doubled-moat glacis offset");
    assert!(wet.glacis_off > dry.glacis_off);
}

// ------------------------------------------- mutation-sweep fixtures (2026-09-24) --
//
// Every test below exists because a mutant survived the 2026-09-24 sweep
// (`URBAN_MORPHOLOGY_SCOPE.md`, milestone 10). Each is built out of the geometry
// under test: a hand-laid junction graph, a site field overwritten to put one
// side of a comparison where the fixture needs it, and — where the survivor
// was a tie — an exact tie, asserted exact before it is used.

/// A closed ring of `n` points at radius `r` about `c`, starting at angle
/// `phase`, with the first point repeated at the end.
fn closed_ring(c: Vec2, n: usize, r: f64, phase: f64) -> Vec<Vec2> {
    (0..=n)
        .map(|i| {
            let a = phase + 2.0 * PI * (i % n) as f64 / n as f64;
            Vec2::new(c.x + js_cos(a) * r, c.y + js_sin(a) * r)
        })
        .collect()
}

/// The junctions `built_mass_hull` counts: at least two live edges.
fn junctions(g: &Graph) -> Vec<Vec2> {
    g.nodes.iter().filter(|n| g.live_degree(n.id) >= 2).map(|n| n.pt()).collect()
}

fn landlocked() -> (Site, Anchors) {
    let site = build_site(5, 1700.0, 1250.0, "landlocked", SiteOpts::default());
    let anchors = place_anchors(5, &site);
    (site, anchors)
}

fn extent(p: &[Vec2]) -> (f64, f64) {
    let (mut x0, mut x1, mut y0, mut y1) = (f64::INFINITY, f64::NEG_INFINITY, f64::INFINITY, f64::NEG_INFINITY);
    for q in p {
        x0 = x0.min(q.x);
        x1 = x1.max(q.x);
        y0 = y0.min(q.y);
        y1 = y1.max(q.y);
    }
    (x1 - x0, y1 - y0)
}

#[test]
fn nearest_idx_starts_from_infinity_not_from_any_finite_bound() {
    // Every candidate is more than a kilometre away and the nearest is not the
    // first: a finite starting bound would find nothing and answer 0.
    let pts = pts(&[5000., 0., 0., 0.]);
    assert_eq!(nearest_idx(&pts, Vec2::new(0.0, 5000.0)), 1);
}

#[test]
fn corner_cut_clamps_the_cosine_to_exactly_minus_one_and_one() {
    // A 0.1 rad needle under a 0.3 rad threshold: clamping the cosine anywhere
    // short of 1 would floor the angle above 0.3 and leave the needle uncut.
    let needle = vec![Vec2::new(0.0, 0.0), Vec2::new(100.0, 0.0), Vec2::new(100.0 * js_cos(0.1), 100.0 * js_sin(0.1))];
    assert_eq!(corner_cut(&needle, 0.3, 1).len(), 4, "only the needle vertex is cut");
    // A 3.06 rad near-straight vertex under a 3.0 rad threshold: clamping the
    // cosine anywhere short of -1 would cap the angle below 3.0 and cut it.
    let bent = pts(&[0., 0., 50., -2., 100., 0., 100., 100., 0., 100.]);
    assert_eq!(corner_cut(&bent, 3.0, 1).len(), 9, "four right angles cut, the 3.06 rad vertex kept");
}

#[test]
fn corner_cut_keeps_a_vertex_exactly_at_the_threshold() {
    // `angI < minAng`, strict: a square's corners have a dot product of exactly
    // zero, so their angle is exactly `js_acos(0)`.
    let right = js_acos(0.0);
    assert_eq!(right, 1.5707963267948966);
    let square = pts(&[0., 0., 100., 0., 100., 100., 0., 100.]);
    assert_eq!(corner_cut(&square, right, 1), square, "an angle equal to the threshold is not cut");
}

#[test]
fn town_bank_keeps_its_normal_when_the_market_lies_on_the_tangent() {
    // `nl.dot(market - p) < 0` flips the offset toward the market. With the
    // market on the centreline's own extension the dot product is exactly zero,
    // and zero does not flip.
    let line = pts(&[100., 600., 500., 600., 900., 600.]);
    let mut site = build_site(7, 1700.0, 1250.0, "river", SiteOpts::default());
    site.river = line.clone();
    site.river_w = 20.0;
    let anchors = Anchors { market: Vec2::new(1300.0, 600.0), prov: "fixture" };
    assert_eq!(town_bank(&site, &anchors)[1], Vec2::new(500.0, 615.0), "channel: riverW/2 + 5 along +y");
    let mut coast = build_site(7, 1700.0, 1250.0, "coast", SiteOpts::default());
    coast.river = line;
    coast.uses_real_water = true;
    assert_eq!(town_bank(&coast, &anchors)[1], Vec2::new(500.0, 605.0), "real coast: 5 m along +y");
}

#[test]
fn from_paths_discounts_only_all_primary_vertices_under_degree_three() {
    let (site, mut anchors) = landlocked();
    let m = Vec2::new(850.0, 625.0);
    anchors.market = m;
    // Eight rim junctions of degree exactly 3, every edge primary: counted,
    // because the discount is for degree < 3.
    let mut g = Graph::new();
    let rim = closed_ring(m, 8, 150.0, 0.1);
    g.add_polyline_street(&rim, "primary", 6.0, 1, "fixture rim");
    for p in &rim[..8] {
        g.add_street(m.x, m.y, p.x, p.y, "primary", 6.0, 1, "fixture spoke");
    }
    g.from_paths = true;
    let deg3 = g.nodes.iter().filter(|n| g.live_degree(n.id) == 3).count();
    assert_eq!(deg3, 8, "eight all-primary junctions of degree three");
    assert!(built_mass_hull(&site, &anchors, &g).is_some(), "degree-3 primary junctions are built mass");
    // Eight degree-2 junctions, each between one primary and one street:
    // counted, because the discount is for vertices whose edges are ALL primary.
    let mut g = Graph::new();
    let rim = closed_ring(m, 8, 150.0, 0.1);
    for (i, w) in rim.windows(2).enumerate() {
        let cls = if i % 2 == 0 { "primary" } else { "street" };
        g.add_street(w[0].x, w[0].y, w[1].x, w[1].y, cls, 6.0, 1, "fixture rim");
    }
    g.from_paths = true;
    assert_eq!(junctions(&g).len(), 8);
    assert!(built_mass_hull(&site, &anchors, &g).is_some(), "a mixed-class junction is built mass");
}

/// A `river`-kind synthetic site whose centreline is overwritten with a
/// horizontal line at `y`, spanning the box.
fn river_at(kind: &str, y: f64, w: f64) -> Site {
    let mut site = build_site(7, 1700.0, 1250.0, kind, SiteOpts::default());
    site.river = vec![Vec2::new(0.0, y), Vec2::new(850.0, y), Vec2::new(1700.0, y)];
    site.river_w = w;
    site
}

#[test]
fn built_mass_hull_keeps_a_junction_exactly_at_the_channel_margin() {
    // `riverDist(p) < riverW/2 + 14` sets a node aside; exactly at the margin it
    // stays. Eight junctions, one of them placed so the margin lands on it.
    let m = Vec2::new(850.0, 500.0);
    let rim = closed_ring(m, 8, 150.0, 0.0);
    let low = rim[2]; // angle pi/2: the southernmost
    let mut site = river_at("river", low.y + 20.0, 10.0);
    let rd = site.river_dist(low);
    assert_eq!(rd, 20.0, "the fixture's distance is exact");
    site.river_w = 2.0 * (rd - 14.0);
    assert_eq!(site.river_w / 2.0 + 14.0, rd, "the margin lands exactly on the junction");
    let anchors = Anchors { market: m, prov: "fixture" };
    let mut g = Graph::new();
    g.add_polyline_street(&rim, "street", 4.0, 1, "fixture ring");
    assert_eq!(junctions(&g).len(), 8);
    assert!(built_mass_hull(&site, &anchors, &g).is_some(), "the junction on the margin still counts");
}

#[test]
fn a_through_site_still_needs_eight_near_bank_junctions() {
    // `near.length < 8` is tested on the near bank alone, before the far bank
    // is folded in — so seven near junctions refuse even when a riverthrough
    // site's far bank would bring the mass to fifteen.
    let m = Vec2::new(850.0, 400.0);
    let site = river_at("riverthrough", m.y + 100.0, 20.0);
    assert!(site.through, "a riverthrough site spans the water by definition");
    let anchors = Anchors { market: m, prov: "fixture" };
    let build = |near_n: usize| {
        let mut g = Graph::new();
        g.add_polyline_street(&closed_ring(m, near_n, 40.0, 0.2), "street", 4.0, 1, "near");
        g.add_polyline_street(&closed_ring(Vec2::new(m.x, m.y + 200.0), 8, 40.0, 0.2), "street", 4.0, 1, "far");
        assert_eq!(junctions(&g).len(), near_n + 8);
        built_mass_hull(&site, &anchors, &g)
    };
    assert!(build(7).is_none(), "seven near-bank junctions refuse");
    assert!(build(8).is_some(), "eight are a town");
}

#[test]
fn the_bridge_town_rule_is_strictly_more_than_max_twenty_and_thirty_two_percent() {
    // `far > max(20, near * 0.32)`: both arms, at their exact integer boundaries.
    let m = Vec2::new(850.0, 300.0);
    let site = river_at("river", m.y + 300.0, 20.0);
    let anchors = Anchors { market: m, prov: "fixture" };
    let spans = |near_n: usize, near_r: f64, far_n: usize, far_r: f64, far_y: f64| {
        let mut g = Graph::new();
        g.add_polyline_street(&closed_ring(m, near_n, near_r, 0.05), "street", 4.0, 1, "near");
        g.add_polyline_street(&closed_ring(Vec2::new(m.x, m.y + far_y), far_n, far_r, 0.05), "street", 4.0, 1, "far");
        let js = junctions(&g);
        assert_eq!(js.iter().filter(|p| p.y < m.y + 300.0).count(), near_n, "near-bank junctions");
        assert_eq!(js.iter().filter(|p| p.y > m.y + 300.0).count(), far_n, "far-bank junctions");
        built_mass_hull(&site, &anchors, &g).expect("a town").spans_water
    };
    // 40 near: 40 * 0.32 = 12.8, so the floor of 20 binds.
    assert!(!spans(40, 90.0, 20, 50.0, 450.0), "twenty is not more than twenty");
    assert!(spans(40, 90.0, 21, 55.0, 450.0), "twenty-one is");
    // 100 near: 100 * 0.32 = 32, over the floor.
    assert!(spans(100, 200.0, 33, 70.0, 420.0), "thirty-three is more than thirty-two percent of a hundred");
}

#[test]
fn the_percentile_cut_keeps_a_junction_exactly_on_it() {
    // `d <= cut`, inclusive: seven junctions on a ring about the market and an
    // eighth placed exactly at `ds[floor(8 * 0.85)] * 1.12`. The market is the
    // origin so that distance along the axis is exact.
    let (site, mut anchors) = landlocked();
    let m = Vec2::new(0.0, 0.0);
    anchors.market = m;
    let mut g = Graph::new();
    let ring = closed_ring(m, 7, 150.0, 0.0);
    g.add_polyline_street(&ring, "street", 4.0, 1, "fixture ring");
    let mut ds: Vec<f64> = junctions(&g).iter().map(|p| p.dist(m)).collect();
    ds.sort_by(|a, b| js_num_cmp(*a, *b));
    assert_eq!(ds.len(), 7);
    let cut = ds[6] * 1.12; // floor(8 * 0.85) = 6 once the eighth joins, and it sorts last
    g.add_polyline_street(&[ring[0], Vec2::new(cut, 0.0), Vec2::new(cut + 140.0, 0.0)], "street", 4.0, 1, "spur");
    let js = junctions(&g);
    assert_eq!(js.len(), 8);
    assert!(js.iter().any(|p| p.dist(m) == cut), "the eighth junction sits exactly on the cut");
    assert!(built_mass_hull(&site, &anchors, &g).is_some(), "a junction on the cut is kept");
}

/// A rectangle of junctions, `2a` by `2b`, about `c`: corners plus points
/// along the long sides, every coordinate an exact integer.
fn rect_town(c: Vec2, a: f64, b: f64) -> Graph {
    let mut ring = Vec::new();
    for k in 0..=4 {
        ring.push(Vec2::new(c.x - a + 2.0 * a * k as f64 / 4.0, c.y - b));
    }
    for k in 0..=4 {
        ring.push(Vec2::new(c.x + a - 2.0 * a * k as f64 / 4.0, c.y + b));
    }
    ring.push(ring[0]);
    let mut g = Graph::new();
    g.add_polyline_street(&ring, "street", 4.0, 1, "fixture rect");
    g
}

#[test]
fn the_aspect_cap_is_for_real_water_only_and_compresses_the_long_axis() {
    let (mut site, mut anchors) = landlocked();
    let m = Vec2::new(850.0, 625.0);
    anchors.market = m;
    let g = rect_town(m, 400.0, 48.0);
    assert!(junctions(&g).len() >= 8);
    // Synthetic water: the hull keeps its aspect, 800 m and more across.
    let bm = built_mass_hull(&site, &anchors, &g).expect("a town");
    let (w, h) = extent(&bm.hull);
    assert!(w > 800.0 && h < 200.0, "uncapped: {w} x {h}");
    // Real water: capped at 2.4, by compressing along the long (x) axis.
    site.uses_real_water = true;
    let bm = built_mass_hull(&site, &anchors, &g).expect("a town");
    let (w2, h2) = extent(&bm.hull);
    assert!(w2 < 400.0, "the long axis is compressed: {w2}");
    assert!((h2 - h).abs() < 1e-6, "the short axis is not: {h2} vs {h}");
}

#[test]
fn the_aspect_cap_applies_to_a_three_vertex_hull() {
    // `hull.length >= 3`: a triangle is the smallest hull the cap reads, and it
    // is capped. Every junction lies exactly on one of the three sides, so the
    // hull is exactly the three corners.
    let (mut site, mut anchors) = landlocked();
    site.uses_real_water = true;
    let m = Vec2::new(850.0, 625.0);
    anchors.market = m;
    let (t1, t2, t3) = (Vec2::new(550.0, 665.0), Vec2::new(1150.0, 665.0), Vec2::new(850.0, 545.0));
    let mut ring = Vec::new();
    for (a, b) in [(t1, t2), (t2, t3), (t3, t1)] {
        for k in 0..4 {
            ring.push(a.lerp(b, k as f64 / 4.0));
        }
    }
    ring.push(t1);
    let mut g = Graph::new();
    g.add_polyline_street(&ring, "street", 4.0, 1, "fixture triangle");
    assert_eq!(junctions(&g).len(), 12);
    let bm = built_mass_hull(&site, &anchors, &g).expect("a town");
    assert_eq!(bm.hull.len(), 3, "the hull is the triangle");
    let (w, _) = extent(&bm.hull);
    assert!(w < 600.0, "the triangle's 600 m base is compressed: {w}");
}

/// A landlocked site with a real heightfield: a linear ramp rising along
/// `(dx, dy)` at 1e-4 field units per metre.
fn ramp_site(dx: f64, dy: f64) -> Site {
    let (mw, mh, cell) = (68usize, 50usize, 25.0);
    let mut grid = vec![0.0f64; mw * mh];
    for j in 0..mh {
        for i in 0..mw {
            let (x, y) = ((i as f64 + 0.5) * cell, (j as f64 + 0.5) * cell);
            grid[j * mw + i] = 0.5 + 1e-4 * (dx * x + dy * y);
        }
    }
    let h_min = grid.iter().copied().fold(f64::INFINITY, js_min);
    let h_max = grid.iter().copied().fold(f64::NEG_INFINITY, js_max);
    let t = TerrainCtx { grid, mw, mh, cell_m: cell, h_min, h_max };
    build_site(5, 1700.0, 1250.0, "landlocked", SiteOpts { terrain: Some(t), ..SiteOpts::default() })
}

fn deflected(site: &Site, m: Vec2, g: &Graph) -> WallState {
    let anchors = Anchors { market: m, prov: "fixture" };
    let mut ws = WallState::default();
    build_wall(1, site, &anchors, g, &mut ws, 1, None, &FortOpts::default());
    assert!(ws.ring.is_some(), "the fixture must build a circuit");
    ws
}

#[test]
fn terrain_deflection_needs_real_terrain_not_merely_a_relief_value() {
    // The ramp that deflects at relief 0.01 (below), with the flag cleared.
    let m = Vec2::new(850.0, 625.0);
    let mut site = ramp_site(1.0, 0.0);
    site.terrain_relief = 0.01;
    let mut g = Graph::new();
    g.add_polyline_street(&closed_ring(m, 8, 150.0, 0.0), "street", 4.0, 1, "ring");
    assert!(deflected(&site, m, &g).terrain_deflected > 0, "the flag set: it deflects");
    site.uses_real_terrain = false;
    assert_eq!(deflected(&site, m, &g).terrain_deflected, 0, "the flag cleared: it does not");
}

#[test]
fn terrain_deflection_engages_at_exactly_the_relief_floor_and_on_a_triangle() {
    let m = Vec2::new(850.0, 625.0);
    let mut site = ramp_site(1.0, 0.0);
    assert!(site.uses_real_terrain);
    let mut g = Graph::new();
    g.add_polyline_street(&closed_ring(m, 8, 150.0, 0.0), "street", 4.0, 1, "ring");
    // `relief >= 0.01`, inclusive.
    site.terrain_relief = 0.01;
    assert!(deflected(&site, m, &g).terrain_deflected > 0, "relief exactly 0.01 engages");
    site.terrain_relief = 0.0099;
    assert_eq!(deflected(&site, m, &g).terrain_deflected, 0, "just under it does not");
    // `hull.length >= 3`: a three-vertex hull is deflected too.
    site.terrain_relief = 0.01;
    let (t1, t2, t3) = (Vec2::new(700.0, 725.0), Vec2::new(1000.0, 725.0), Vec2::new(850.0, 465.0));
    let mut ring = Vec::new();
    for (a, b) in [(t1, t2), (t2, t3), (t3, t1)] {
        for k in 0..4 {
            ring.push(a.lerp(b, k as f64 / 4.0));
        }
    }
    ring.push(t1);
    let mut tg = Graph::new();
    tg.add_polyline_street(&ring, "street", 4.0, 1, "triangle");
    let anchors = Anchors { market: m, prov: "fixture" };
    assert_eq!(built_mass_hull(&site, &anchors, &tg).expect("a town").hull.len(), 3);
    assert!(deflected(&site, m, &tg).terrain_deflected > 0, "a triangle's vertices are deflected");
}

#[test]
fn terrain_deflection_prices_distance_at_three_point_three_e_minus_four() {
    // On a linear ramp every offset gains in proportion to it, so the best is
    // +60 whenever `60 * slope * ux > 60 * 3.3e-4 * relief + 0.015 * relief`.
    // With the relief overwritten the threshold on `ux` is known exactly:
    // 1e-4 * 60 * ux > relief * (60 * 3.3e-4 + 0.015) is ux > 0.9 at
    // relief = 0.006 / (0.0198 + 0.015) — so the hull vertex at 25 degrees
    // (ux = 0.906) deflects, and at a price of 3.4e-4 it would not (ux > 0.915).
    let m = Vec2::new(850.0, 625.0);
    let mut site = ramp_site(1.0, 0.0);
    site.terrain_relief = 1e-4 * 60.0 * 0.9 / (60.0 * 3.3e-4 + 0.015);
    let mut g = Graph::new();
    g.add_polyline_street(&closed_ring(m, 12, 150.0, 25f64.to_radians()), "street", 4.0, 1, "ring");
    let anchors = Anchors { market: m, prov: "fixture" };
    let hull = built_mass_hull(&site, &anchors, &g).expect("a town").hull;
    let c0 = crate::geom::poly_centroid(&hull);
    // A west-side vertex gains by stepping inward (-60), an east-side one by
    // stepping outward (+60): both are priced the same, on |ux|.
    let ux: Vec<f64> = hull.iter().map(|v| (*v - c0).norm().x.abs()).collect();
    assert_eq!(ux.iter().filter(|&&u| u > 0.9).count(), 4, "four vertices clear 0.9: {ux:?}");
    assert_eq!(ux.iter().filter(|&&u| u > 0.9155).count(), 2, "two of them clear 0.9155");
    assert_eq!(deflected(&site, m, &g).terrain_deflected, 4);
}

#[test]
fn terrain_deflection_keeps_twenty_metres_off_the_far_box_edges() {
    // `q.x > wm - 20` and `q.y > hm - 20`: on a ramp rising east (south) the
    // easternmost (southernmost) candidate is the +60 m offset of the extreme
    // hull vertex, and it is the one chosen. Put the box edge 20.5 m past it:
    // inside the margin, so nothing changes.
    let m = Vec2::new(850.0, 625.0);
    for (dx, dy) in [(1.0, 0.0), (0.0, 1.0)] {
        let mut site = ramp_site(dx, dy);
        site.terrain_relief = 0.01;
        let mut g = Graph::new();
        g.add_polyline_street(&closed_ring(m, 8, 150.0, 0.0), "street", 4.0, 1, "ring");
        let free = deflected(&site, m, &g);
        let anchors = Anchors { market: m, prov: "fixture" };
        let hull = built_mass_hull(&site, &anchors, &g).expect("a town").hull;
        let c0 = crate::geom::poly_centroid(&hull);
        let far = hull
            .iter()
            .map(|v| *v + (*v - c0).norm() * 60.0)
            .fold(f64::NEG_INFINITY, |acc, q| acc.max(if dx > 0.0 { q.x } else { q.y }));
        if dx > 0.0 {
            site.wm = far + 20.5;
        } else {
            site.hm = far + 20.5;
        }
        let boxed = deflected(&site, m, &g);
        assert!(free.terrain_deflected > 0);
        assert_eq!(boxed.ring, free.ring, "a candidate 20.5 m inside the edge is still allowed ({dx},{dy})");
    }
}

/// `riverTown`'s own circuit, rebuilt with a harbour of the fixture's choosing.
fn river_town_with(harbour: Option<&HarbourFront>) -> WallState {
    let c = golden::GOLDEN.iter().find(|c| c.name == "riverTown").expect("riverTown");
    let (site, anchors, g, _) = town(c);
    let mut ws = WallState::default();
    build_wall(c.wall_seed, &site, &anchors, &g, &mut ws, c.ep, harbour, &FortOpts::default());
    ws
}

#[test]
fn the_harbour_mouth_is_a_strict_48_metre_gap_and_needs_a_quay() {
    let plain = river_town_with(None);
    let water = plain.water_closure.clone().expect("riverTown follows its bank");
    assert!(water.len() >= 5, "a real water walk");
    assert_eq!(plain.water_walls.len(), 1);
    let k = water.len() / 2;
    let w = water[k];
    // The mouth straight across the channel from bank point `k`, at a chosen
    // distance, with every other bank point farther away.
    let runs = |d: f64, quay: bool| {
        let s = if w.y < 780.0 { 1.0 } else { -1.0 };
        let pt = Vec2::new(w.x, w.y + s * d);
        let nearest = water.iter().map(|q| q.dist(pt)).fold(f64::INFINITY, f64::min);
        assert_eq!(nearest, pt.dist(w), "bank point k is the nearest");
        assert_eq!(pt.dist(w), d, "the distance is exact");
        let h = HarbourFront { quay: if quay { vec![pt] } else { Vec::new() }, pt };
        river_town_with(Some(&h)).water_walls.len()
    };
    assert_eq!(runs(47.5, true), 2, "47.5 m is inside the gap");
    assert_eq!(runs(48.0, true), 1, "exactly 48 m is not");
    assert_eq!(runs(48.5, true), 1, "48.5 m is not");
    assert_eq!(runs(0.0, false), 1, "an empty quay draws no gap, wherever its point is");
}

#[test]
fn an_empty_wall_style_is_the_legacy_curtain() {
    let (site, anchors) = landlocked();
    let mut g = Graph::new();
    build_primaries(5, &site, &anchors, &mut g);
    let mut ws = WallState::default();
    let opts = GrowOpts { target_len: 3600.0, max_rf: 430.0, ..GrowOpts::default() };
    grow(5, &site, &anchors, &mut g, 8, &mut ws, &opts, &mut RecordingWallBuilder::default());
    let mut ws = WallState::default();
    let fo = FortOpts { wall_style: Some(String::new()), ..FortOpts::default() };
    build_wall(5, &site, &anchors, &g, &mut ws, 9, None, &fo);
    assert_eq!(ws.style, "curtain");
}

/// A landlocked ring town and one primary straight south through its wall at
/// `x`: the wall and that one land gate.
fn gated(site: &Site, x_off: f64) -> (WallState, Vec2) {
    let m = Vec2::new(850.0, 400.0);
    let mut g = Graph::new();
    g.add_polyline_street(&closed_ring(m, 8, 150.0, 0.2), "street", 4.0, 1, "ring");
    g.add_street(m.x + x_off, m.y, m.x + x_off, m.y + 420.0, "primary", 6.0, 1, "primary");
    let anchors = Anchors { market: m, prov: "fixture" };
    let mut ws = WallState::default();
    build_wall(1, site, &anchors, &g, &mut ws, 1, None, &FortOpts::default());
    assert_eq!(ws.gates.len(), 1, "one gate");
    let cp = ws.gates[0].pt;
    (ws, cp)
}

#[test]
fn a_shoreline_gate_is_a_water_gate_within_24_metres_of_the_straddling_vertex() {
    let (base, _) = landlocked();
    let (ws0, cp) = gated(&base, 10.0);
    assert!(!ws0.gates[0].water, "no shoreline, no water-gate");
    let water = |line: Vec<Vec2>| {
        let mut site = base.clone();
        site.river = line;
        let (ws, _) = gated(&site, 10.0);
        assert_eq!(ws.ring, ws0.ring, "the shoreline must not move the circuit");
        ws.gates[0].water
    };
    let (near, far) = (cp.y + 10.0, cp.y + 100.0);
    // `|p.y - b| < 24`, strict, against the vertex straddling p.x.
    assert!(!water(vec![Vec2::new(0.0, cp.y + 24.5), Vec2::new(1700.0, cp.y + 24.5)]), "24.5 m is dry");
    assert_eq!((cp.y - (cp.y + 24.0)).abs(), 24.0);
    assert!(!water(vec![Vec2::new(0.0, cp.y + 24.0), Vec2::new(1700.0, cp.y + 24.0)]), "exactly 24 m is dry");
    assert!(water(vec![Vec2::new(0.0, cp.y + 23.5), Vec2::new(1700.0, cp.y + 23.5)]), "23.5 m is wet");
    // The straddling vertex is the first with `p.x <= q.x` whose predecessor has
    // `p.x > prev.x`: a vertex exactly at p.x is chosen, and its successor is not.
    assert!(water(vec![Vec2::new(0.0, far), Vec2::new(cp.x, near), Vec2::new(1700.0, far)]), "q.x == p.x is the one");
    // Vertex 0 needs no predecessor test.
    assert!(water(vec![Vec2::new(cp.x + 50.0, near), Vec2::new(cp.x + 100.0, far)]), "vertex 0 straddles alone");
    // Past every vertex, the first vertex's y stands.
    assert!(water(vec![Vec2::new(cp.x - 200.0, cp.y + 23.5), Vec2::new(cp.x - 100.0, far)]), "the first y stands");
}

#[test]
fn a_channel_gate_is_a_water_gate_strictly_within_half_width_plus_22() {
    let m_y = 400.0;
    let mut site = river_at("river", m_y + 500.0, 20.0);
    let (ws0, cp) = gated(&site, 10.0);
    assert!(!ws0.gates[0].water);
    let run = |site: &Site| {
        let (ws, _) = gated(site, 10.0);
        assert_eq!(ws.ring, ws0.ring, "moving the channel must not move the circuit");
        ws.gates[0].water
    };
    // riverW = 20: the margin is 32.
    site.river = vec![Vec2::new(0.0, cp.y + 32.5), Vec2::new(1700.0, cp.y + 32.5)];
    assert!(!run(&site), "32.5 m is dry");
    site.river = vec![Vec2::new(0.0, cp.y + 30.0), Vec2::new(1700.0, cp.y + 30.0)];
    let rd = site.river_dist(cp);
    assert_eq!(rd, 30.0);
    site.river_w = 2.0 * (rd - 22.0);
    assert_eq!(site.river_w / 2.0 + 22.0, rd);
    assert!(!run(&site), "exactly on the margin is dry");
}

#[test]
fn two_land_gates_merge_strictly_inside_40_metres() {
    // A square town's wall runs flat along its south side, so two vertical
    // primaries cross it at the same y and exactly their x offset apart.
    let (site, _) = landlocked();
    let m = Vec2::new(850.0, 400.0);
    let count = |half: f64| {
        let mut g = rect_town(m, 150.0, 150.0);
        g.add_street(m.x - half, m.y, m.x - half, m.y + 420.0, "primary", 6.0, 1, "primary a");
        g.add_street(m.x + half, m.y, m.x + half, m.y + 420.0, "primary", 6.0, 1, "primary b");
        let anchors = Anchors { market: m, prov: "fixture" };
        let mut ws = WallState::default();
        build_wall(1, &site, &anchors, &g, &mut ws, 1, None, &FortOpts::default());
        if ws.gates.len() == 2 {
            assert_eq!(ws.gates[0].pt.y, ws.gates[1].pt.y, "both cross the flat side");
            assert_eq!(ws.gates[0].pt.dist(ws.gates[1].pt), 2.0 * half, "exactly {} apart", 2.0 * half);
        }
        ws.gates.len()
    };
    assert_eq!(count(20.25), 2, "40.5 m apart: two gates");
    assert_eq!(count(20.0), 2, "exactly 40 m apart: still two");
    assert_eq!(count(19.75), 1, "39.5 m apart: merged");
}

/// A fortified landlocked ring town with radial primaries at `angles` (radians,
/// from the town centre) and the market pulled 40 m toward `pull`: the angles
/// of the land gates the bastioned cap keeps, measured as the code measures
/// them, from the circuit's centroid.
fn capped_gate_angles(angles: &[f64], pull: f64) -> Vec<f64> {
    let (site, _) = landlocked();
    let c = Vec2::new(850.0, 625.0);
    let mut g = Graph::new();
    g.add_polyline_street(&closed_ring(c, 12, 150.0, 0.13), "street", 4.0, 1, "ring");
    for &a in angles {
        g.add_street(c.x, c.y, c.x + 520.0 * js_cos(a), c.y + 520.0 * js_sin(a), "primary", 6.0, 1, "spoke");
    }
    let anchors = Anchors { market: Vec2::new(c.x + 40.0 * js_cos(pull), c.y + 40.0 * js_sin(pull)), prov: "fixture" };
    let mut ws = WallState::default();
    build_wall(1, &site, &anchors, &g, &mut ws, 1, None, &FortOpts { fortified: true, ..FortOpts::default() });
    assert_eq!(ws.style, "bastioned");
    let n = ws.fort.as_ref().expect("fort").bastions.len();
    assert!((4..=6).contains(&n), "{n} bastions: a cap of two");
    let cen = ws.centroid.expect("built");
    ws.gates.iter().filter(|g| !g.water).map(|g| js_atan2(g.pt.y - cen.y, g.pt.x - cen.x)).collect()
}

fn has_near(angles: &[f64], a: f64) -> bool {
    angles.iter().any(|&x| {
        let d = (x - a).abs();
        d.min(2.0 * PI - d) < 0.1
    })
}

#[test]
fn the_bastioned_cap_spreads_its_gates_at_least_0_9_rad_apart() {
    // Three primaries: two near the market (at 0 and at `b`) and one opposite.
    // With `b` under 0.9 rad the second near gate clashes with the first and
    // the opposite one is kept instead; over 0.9 rad both near ones are kept.
    let kept = capped_gate_angles(&[0.0, 0.85, PI], 0.425);
    assert_eq!(kept.len(), 2);
    assert!(has_near(&kept, PI), "0.85 rad clashes: the opposite gate is kept {kept:?}");
    let kept = capped_gate_angles(&[0.0, 0.95, PI], 0.475);
    assert_eq!(kept.len(), 2);
    assert!(!has_near(&kept, PI), "0.95 rad does not clash {kept:?}");
    // Across the ±pi seam: gates at ±2.1 rad are 2.08 rad apart the short way,
    // not 4.2, and do not clash.
    let kept = capped_gate_angles(&[2.1, -2.1, 0.0], PI);
    assert_eq!(kept.len(), 2);
    assert!(!has_near(&kept, 0.0), "the seam is wrapped {kept:?}");
}

#[test]
fn a_star_fort_applies_to_a_triangular_circuit() {
    let (site, _) = landlocked();
    let mut ws = WallState { ring: Some(pts(&[500., 500., 900., 500., 700., 800.])), ..WallState::default() };
    apply_star_fort(4, &site, &mut ws, &FortOpts::default());
    assert_eq!(ws.style, "bastioned", "a three-point hull is enough");
    assert!(ws.fort.is_some_and(|f| f.bastions.len() >= 4));
}

#[test]
fn a_ditch_floods_strictly_within_175_metres_of_the_waterline() {
    // A hexagon's trace; then a straight shoreline laid `d` metres south of
    // its southernmost point, so the trace's nearest approach is exactly `d`.
    let ring: Vec<Vec2> = closed_ring(Vec2::new(850.0, 450.0), 6, 250.0, 0.0)[..6].to_vec();
    let fort = |site: &Site, wet_moat: bool| {
        let mut ws = WallState { ring: Some(ring.clone()), ..WallState::default() };
        apply_star_fort(4, site, &mut ws, &FortOpts { wet_moat, ..FortOpts::default() });
        ws.fort.expect("the hexagon applies")
    };
    let mut coast = build_site(7, 1700.0, 1250.0, "coast", SiteOpts::default());
    coast.river = vec![Vec2::new(0.0, 5000.0), Vec2::new(1700.0, 5000.0)];
    let trace = fort(&coast, false).trace;
    let low = trace.iter().copied().fold(Vec2::new(0.0, f64::NEG_INFINITY), |a, p| if p.y > a.y { p } else { a });
    let at = |d: f64| {
        let mut s = coast.clone();
        s.river = vec![Vec2::new(0.0, low.y + d), Vec2::new(1700.0, low.y + d)];
        let near = trace.iter().map(|p| s.river_dist(*p)).fold(f64::INFINITY, f64::min);
        assert_eq!(near, d, "the trace's nearest approach is exactly {d}");
        s
    };
    assert!(!fort(&at(177.5), false).wet_ditch, "177.5 m is dry");
    assert!(!fort(&at(175.0), false).wet_ditch, "exactly 175 m is dry");
    assert!(fort(&at(172.5), false).wet_ditch, "172.5 m floods");
    // `!(d < 175) && wetMoat`: a supplied moat past the waterline's reach is canal-fed.
    let f = fort(&at(177.5), true);
    assert!(f.wet_ditch && f.canal_fed, "177.5 m with wetMoat is canal-fed");
    // `site.noWater` skips the measurement outright, whatever `river` holds.
    let mut dry = at(100.0);
    assert!(fort(&dry, false).wet_ditch, "100 m floods");
    dry.no_water = true;
    assert!(!fort(&dry, false).wet_ditch, "a noWater site never measures");
}

#[test]
fn the_builder_passes_wall_style_and_wet_moat_through() {
    let (site, anchors) = landlocked();
    let mut g = Graph::new();
    build_primaries(5, &site, &anchors, &mut g);
    let mut ws = WallState::default();
    let base = GrowOpts { target_len: 3600.0, max_rf: 430.0, walls: false, ..GrowOpts::default() };
    grow(5, &site, &anchors, &mut g, 8, &mut ws, &base, &mut RecordingWallBuilder::default());
    let mut b = FortificationBuilder;
    let mut ws = WallState::default();
    let opts = GrowOpts { wall_style: Some("palisade".into()), ..base.clone() };
    b.build_wall(5, &site, &anchors, &mut g, &mut ws, 9, &opts);
    assert_eq!(ws.style, "palisade");
    let mut ws = WallState::default();
    let opts = GrowOpts { fortified: true, wet_moat: true, ..base };
    b.build_wall(5, &site, &anchors, &mut g, &mut ws, 9, &opts);
    let f = ws.fort.expect("fortified");
    assert!(f.wet_ditch && f.canal_fed, "landlocked with wetMoat is canal-fed");
}

/// The circuit `build_wall` traces round `hull` on dry land: two closed Chaikin
/// passes, the corner cut and the simplify — the all-land branch, written out.
fn all_land_ring(hull: &[Vec2]) -> Vec<Vec2> {
    simplify(&corner_cut(&chaikin(&chaikin(hull, true), true), 1.75, 4), 2.0)
}

/// A landlocked site whose 1 m heightfield is a circular crest of radius
/// `crest` about `c0` (flat past 100 m either side), so bilinear sampling
/// cannot move it and every hull vertex steps straight onto it.
fn crest_site(c0: Vec2, crest: f64) -> Site {
    let (mw, mh) = (700usize, 700usize);
    let mut grid = vec![0.0f64; mw * mh];
    for j in 0..mh {
        for i in 0..mw {
            let r = Vec2::new(i as f64 + 0.5, j as f64 + 0.5).dist(c0);
            grid[j * mw + i] = 0.5 - 2e-6 * js_min((r - crest) * (r - crest), 1e4);
        }
    }
    let h_min = grid.iter().copied().fold(f64::INFINITY, js_min);
    let h_max = grid.iter().copied().fold(f64::NEG_INFINITY, js_max);
    let t = TerrainCtx { grid, mw, mh, cell_m: 1.0, h_min, h_max };
    let site = build_site(5, 1700.0, 1250.0, "landlocked", SiteOpts { terrain: Some(t), ..SiteOpts::default() });
    assert!(site.terrain_relief >= 0.01);
    site
}

/// An eight-junction ring town at `m`: its graph, hull, hull centroid and the
/// hull's (common) vertex radius.
fn octagon_town(m: Vec2) -> (Graph, Vec<Vec2>, Vec2, f64) {
    let mut g = Graph::new();
    g.add_polyline_street(&closed_ring(m, 8, 150.0, 0.0), "street", 4.0, 1, "ring");
    let (flat, _) = landlocked();
    let hull = built_mass_hull(&flat, &Anchors { market: m, prov: "fixture" }, &g).expect("a town").hull;
    let c0 = crate::geom::poly_centroid(&hull);
    let r_v = hull[0].dist(c0);
    assert!(hull.iter().all(|v| (v.dist(c0) - r_v).abs() < 1e-6), "a regular hull");
    (g, hull, c0, r_v)
}

#[test]
fn terrain_deflection_can_choose_the_thirty_metre_inward_step() {
    // A crest 30 m inside every hull vertex: every vertex steps exactly -30.
    let m = Vec2::new(350.0, 350.0);
    let (g, hull, c0, r_v) = octagon_town(m);
    let site = crest_site(c0, r_v - 30.0);
    let ws = deflected(&site, m, &g);
    assert_eq!(ws.terrain_deflected as usize, hull.len(), "every vertex steps onto the crest");
    let moved: Vec<Vec2> = hull.iter().map(|v| *v + (*v - c0).norm() * -30.0).collect();
    assert_eq!(ws.ring.as_deref(), Some(all_land_ring(&moved).as_slice()), "each by exactly -30 m");
}

#[test]
fn terrain_deflection_keeps_twenty_metres_off_the_near_box_edges() {
    // `q.x < 20` and `q.y < 20`: a crest 60 m outside every hull vertex, and
    // the town placed so that its westernmost (northernmost) vertex's +60
    // candidate is 20.5 m from the box edge — still allowed, so every vertex
    // steps exactly +60.
    for west in [true, false] {
        let m0 = Vec2::new(350.0, 350.0);
        let far = |m: Vec2| {
            let (_, hull, c0, _) = octagon_town(m);
            hull.iter()
                .map(|v| *v + (*v - c0).norm() * 60.0)
                .map(|q| if west { q.x } else { q.y })
                .fold(f64::INFINITY, f64::min)
        };
        let d = 20.5 - far(m0);
        let m = if west { Vec2::new(m0.x + d, m0.y) } else { Vec2::new(m0.x, m0.y + d) };
        assert!((far(m) - 20.5).abs() < 1e-6, "the extreme candidate is 20.5 m from the edge");
        let (g, hull, c0, r_v) = octagon_town(m);
        let site = crest_site(c0, r_v + 60.0);
        let ws = deflected(&site, m, &g);
        assert_eq!(ws.terrain_deflected as usize, hull.len(), "west {west}: every vertex steps");
        let moved: Vec<Vec2> = hull.iter().map(|v| *v + (*v - c0).norm() * 60.0).collect();
        assert_eq!(ws.ring.as_deref(), Some(all_land_ring(&moved).as_slice()), "west {west}: each by +60 m");
    }
}

#[test]
fn a_hull_vertex_exactly_at_the_channel_margin_is_water() {
    // `isLand` on a channel is `riverDist > riverW/2 + 1`, strict. The
    // octagon's southernmost hull vertex is one of the densified samples; put
    // the channel so that it sits exactly on the margin and it is water, which
    // sends the circuit down the bank-following branch.
    let m = Vec2::new(850.0, 400.0);
    let (g, hull, _, _) = octagon_town(m);
    let v = hull.iter().copied().fold(Vec2::new(0.0, f64::NEG_INFINITY), |a, p| if p.y > a.y { p } else { a });
    let mut site = river_at("river", v.y + 20.0, 10.0);
    let d = site.river_dist(v);
    site.river_w = 2.0 * (d - 1.0);
    assert_eq!(site.river_w / 2.0 + 1.0, d, "the margin lands exactly on the vertex");
    assert!(densify_loop(&hull, 8.0).contains(&v), "the vertex is a sample");
    let anchors = Anchors { market: m, prov: "fixture" };
    assert!(built_mass_hull(&site, &anchors, &g).is_some_and(|b| b.hull == hull), "the channel moves no junction");
    let mut ws = WallState::default();
    build_wall(1, &site, &anchors, &g, &mut ws, 1, None, &FortOpts::default());
    assert!(ws.ring.is_some());
    assert!(ws.water_closure.is_some(), "one sample on the margin is water: the wall follows the bank");
    assert_eq!(ws.spurs.len(), 2);
}

#[test]
fn two_river_crossings_merge_into_one_water_gate_only_inside_40_metres() {
    // A riverthrough town spans the water by definition, and its water-gates
    // are wherever the centreline crosses the circuit. A square notch in the
    // centreline crosses the flat south side twice, 40.5 m apart.
    let m = Vec2::new(850.0, 400.0);
    let g = rect_town(m, 150.0, 150.0);
    let anchors = Anchors { market: m, prov: "fixture" };
    let mut site = river_at("riverthrough", 5000.0, 20.0);
    let mut ws = WallState::default();
    build_wall(1, &site, &anchors, &g, &mut ws, 1, None, &FortOpts::default());
    let ring = ws.ring.expect("built");
    let yb = ring.iter().map(|p| p.y).fold(f64::NEG_INFINITY, f64::max);
    let flat: Vec<f64> = ring.iter().filter(|p| p.y == yb).map(|p| p.x).collect();
    assert!(flat.len() >= 2, "a flat south side");
    let (x0, x1) = (flat.iter().copied().fold(f64::INFINITY, f64::min), flat.iter().copied().fold(f64::NEG_INFINITY, f64::max));
    let xa = (x0 + 10.0).floor();
    assert!(xa + 40.5 < x1 - 10.0, "the notch fits the flat side: {x0}..{x1}");
    let mut gates = |gap: f64| {
        site.river = vec![
            Vec2::new(xa, yb + 60.0),
            Vec2::new(xa, yb - 10.0),
            Vec2::new(xa + gap, yb - 10.0),
            Vec2::new(xa + gap, yb + 60.0),
        ];
        let mut ws = WallState::default();
        build_wall(1, &site, &anchors, &g, &mut ws, 1, None, &FortOpts::default());
        assert_eq!(ws.ring.as_deref(), Some(ring.as_slice()), "the notch does not move the circuit");
        ws.gates.iter().filter(|gt| gt.water).map(|gt| gt.pt).collect::<Vec<_>>()
    };
    let two = gates(40.5);
    assert_eq!(two.len(), 2, "40.5 m apart: two water-gates");
    assert!((two[0].dist(two[1]) - 40.5).abs() < 1e-9);
    assert_eq!(gates(39.5).len(), 1, "39.5 m apart: one");
}
