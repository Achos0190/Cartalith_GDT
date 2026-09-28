//! Golden-parity tests for road-corridor consolidation, classification,
//! naming, and Catmull-Rom smoothing -- `PHASE2_SCOPE.md` milestone 14:
//! `_civHierarchicalNetwork`'s consolidation tail (reference HTML lines
//! ~21670-21739), plus its helpers `rdpSimplify`/`catmullRomSample`/
//! `_civSmoothPath`/`_civTerrainValidTest`/`_civNearestValidPt` (lines
//! 8701/8790/21892/21843/21872). See `civ_consolidate_and_smooth_ways`'s
//! own doc comment in `src/lib.rs` for the full account.
//!
//! Not required for `civ_seed_villages` (milestone 15) -- that only needs
//! road-PROXIMITY distance, which milestone 12's raw unsmoothed edges
//! already provide -- required for anything that actually draws roads.
//!
//! Node `vm` harness: fresh per this project's established practice (not
//! checked in). Blocks #1 (2084-14556) + #2 (14563-26720) -- note the
//! `<script>` tag itself sits at 2083/14562, one line before code starts;
//! earlier milestones' own documented "2083-14556 + 14562-26720" ranges
//! include that tag and produce a syntax error if sliced literally, this
//! extraction shifted both starts by +1. `state.tect.seed` (not the dead
//! `state.seed`), `allocate()` with zero arguments.
//!
//! `_civHierarchicalNetwork(places, {})` was called directly (not
//! instrumented) since it *returns* the post-consolidation `ways` array
//! natively -- unlike milestone 12, which needed the raw pre-consolidation
//! `allEdges` and had to capture that mid-function. Settlement inputs
//! (`x`/`y`/`faction`/`name`/`pop`, all `kind:'capital'`) are the SAME
//! already-verified fixtures `golden_parity_settlement_naming.rs`'s own
//! case0/case1 established -- not re-derived. `field[0]` was cross-checked
//! against the Rust side and found ~9e-6 apart (well inside this crate's
//! `1e-4` convention) -- normal JS-vs-Rust cross-language float noise per
//! `PARITY_TESTING.md`, not a harness bug (both sides implement the same
//! formula, not the same binary).
//!
//! Case 0 exercises a genuine short-segment Catmull-Rom oversampling
//! quirk, not a synthetic corner case: the 2-cell path `[35,34]` produces
//! a 3-point smoothed output where the middle point rounds to coincide
//! exactly with the (float-precision-restored) start point --
//! `js_round(6.5)=7` (JS/this port's `Math.round`-equivalent rounds .5 up)
//! lands the interpolated midpoint back on the start cell. Confirmed real
//! by tracing the algorithm, not assumed a bug in the extraction. Case 1
//! (K5 complete graph, 10 edges) exercises corridor consolidation proper
//! (shared trunk segments claimed busiest-first, hidden 2-point ways for
//! fully-consolidated edges, both `highway` and `regional` classification)
//! across real, richly-connected data.
//!
//! Continuous point coordinates checked at `1e-4` (matches this crate's
//! established tolerance for continuous fields); `km`, integer/categorical
//! fields (name/type/aIdx/bIdx/hidden/way count/point count) checked
//! exactly.

// RV-1 (Ruling BD): this suite proves parity on the reference's world; see
// `pre_rv1_world.rs` for why the carve's six arrays are pinned back to it.
#[path = "../../cartalith-engine/tests/fixtures/pre_rv1_world.rs"]
mod pre_rv1_world;

fn named(x: usize, y: usize, faction: i32, name: &str, pop: u32) -> cartalith_civ::NamedSettlement {
    cartalith_civ::NamedSettlement {
        tid: 0,
        placement: cartalith_civ::SettlementPlacement {
            x,
            y,
            suit: 0.0,
            faction,
            capital: true,
            kind: cartalith_civ::SettlementKind::Capital,
            coastal: true,
        },
        name: name.to_string(),
        pop,
    }
}

fn affordance_inputs(
    ws: &cartalith_engine::WorldState,
    gw: usize,
    gh: usize,
    world: bool,
    map_width_km: f64,
    river_density: f64,
) -> (Vec<u8>, Vec<u8>, Vec<i16>) {
    let wb = cartalith_civ::build_water_bodies(&ws.field, gw, gh, ws.sea_level, world, Some(&ws.rainfall));
    let biome = cartalith_civ::build_biome_raster(&wb.classification, &ws.temperature, &ws.rainfall);
    let river_order = cartalith_civ::fresh_river_order(&ws.field, &ws.flow_discharge, gw, gh, ws.sea_level, world, river_density, map_width_km, ws.integrated_drainage);
    (wb.classification, biome, river_order)
}

fn assert_pts_match(actual: &[(f64, f64)], expected: &[(f64, f64)], label: &str) {
    assert_eq!(actual.len(), expected.len(), "{label}: point count mismatch: {actual:?} vs {expected:?}");
    for (i, (a, e)) in actual.iter().zip(expected.iter()).enumerate() {
        assert!((a.0 - e.0).abs() < 1e-4, "{label}: pt {i} x mismatch: {a:?} vs {e:?}");
        assert!((a.1 - e.1).abs() < 1e-4, "{label}: pt {i} y mismatch: {a:?} vs {e:?}");
    }
}

fn assert_way_type(actual: cartalith_civ::WayType, expected: &str, label: &str) {
    let matches = match (actual, expected) {
        (cartalith_civ::WayType::Highway, "highway") => true,
        (cartalith_civ::WayType::Regional, "regional") => true,
        (cartalith_civ::WayType::Road, "road") => true,
        (cartalith_civ::WayType::Track, "track") => true,
        _ => false,
    };
    assert!(matches, "{label}: type mismatch: {actual:?} vs {expected}");
}

#[test]
fn road_consolidation_case_0_short_segment_oversample() {
    // case0_region: gw=14 gh=11 seed=24601 world=false. Same (x,y,faction)
    // triples + real names/pop as golden_parity_settlement_naming.rs's
    // case0 and golden_parity_hierarchical_network.rs's case0 (1 edge,
    // path [35,34], the settlement at index 1 unreachable).
    let mut p = cartalith_engine::WorldParams::defaults(14, 11, 24601);
    p.world = false;
    p.climate.w_iters = 12;
    let mut ws = cartalith_engine::generate_terrain(&p);
    pre_rv1_world::pin(&mut ws, p.gw, p.gh, p.tect.seed, p.world);
    assert!((ws.field[0] - 0.8640472292900085f64 as f32).abs() < 1e-4, "field[0] mismatch, harness assumption broken");

    let (water_bodies, biome, river_order) = affordance_inputs(&ws, 14, 11, false, p.map_width_km, p.river_density);
    let places = vec![
        named(7, 2, 1, "Sevjuniana", 19465),
        named(9, 3, 2, "Hurngarngarnhaskcairn", 20094),
        named(6, 2, 3, "Ghalbahrghaltazdune", 22094),
    ];
    let placements: Vec<cartalith_civ::SettlementPlacement> = places.iter().map(|p| p.placement).collect();

    let net = cartalith_civ::civ_hierarchical_network_topology(
        &placements, 14, 11, ws.sea_level, &ws.field, &ws.flow_discharge, &river_order, &biome, &water_bodies, false, p.map_width_km,
    );
    assert_eq!(net.edges.len(), 1, "harness assumption broken: expected 1 edge");

    let ways = cartalith_civ::civ_consolidate_and_smooth_ways(&net, &places, &ws.field, &water_bodies, 14, 11, p.map_width_km);

    // Real extraction: _civHierarchicalNetwork({...places with real
    // names}, {}).ways for this exact seed/config, one way, a genuine
    // 3-point short-segment oversample (see module doc comment).
    assert_eq!(ways.len(), 1, "case0: way count mismatch");
    let w = &ways[0];
    assert_eq!(w.name, "Sevjuniana \u{2192} Ghalbahrghaltazdune", "case0: name mismatch");
    assert_way_type(w.way_type, "track", "case0 way0");
    assert_eq!((w.a_idx, w.b_idx, w.hidden), (0, 2, false), "case0: edge identity mismatch");
    assert_pts_match(&w.pts, &[(7.0, 2.0), (7.0, 2.0), (6.0, 2.0)], "case0 way0 pts");
    assert!((w.km - 57.142857142857146).abs() < 1e-4, "case0: km mismatch: {}", w.km);
    assert!(w.brks.is_empty(), "case0: brks should be empty");
}

#[test]
fn road_consolidation_case_1_k5_corridor_sharing() {
    // case1_world_wrap: gw=16 gh=12 seed=314159 world=true. Same 5
    // (x,y,faction) triples + real names/pop as
    // golden_parity_settlement_naming.rs's case1 and
    // golden_parity_hierarchical_network.rs's case1 (K5, 10 edges).
    // Real corridor consolidation: several edges share trunk segments
    // (busiest-claimed-first), producing a mix of visible (highway/
    // regional) and hidden (fully-consolidated) ways.
    let mut p = cartalith_engine::WorldParams::defaults(16, 12, 314159);
    p.world = true;
    p.climate.w_iters = 12;
    let mut ws = cartalith_engine::generate_terrain(&p);
    pre_rv1_world::pin(&mut ws, p.gw, p.gh, p.tect.seed, p.world);
    assert!((ws.field[0] - 0.2477419376373291f64 as f32).abs() < 1e-4, "field[0] mismatch, harness assumption broken");

    let (water_bodies, biome, river_order) = affordance_inputs(&ws, 16, 12, true, p.map_width_km, p.river_density);
    let places = vec![
        named(9, 3, 1, "Sevjuniana", 20354),
        named(5, 8, 2, "Hurngarngarnhaskcairn", 20697),
        named(8, 9, 3, "Ghalbahrghaltazdune", 22698),
        named(10, 5, 4, "Orenelywash", 15972),
        named(4, 7, 5, "Taela'elorashade", 22508),
    ];
    let placements: Vec<cartalith_civ::SettlementPlacement> = places.iter().map(|p| p.placement).collect();

    let net = cartalith_civ::civ_hierarchical_network_topology(
        &placements, 16, 12, ws.sea_level, &ws.field, &ws.flow_discharge, &river_order, &biome, &water_bodies, true, p.map_width_km,
    );
    assert_eq!(net.edges.len(), 10, "harness assumption broken: expected K5 = 10 edges");

    let ways = cartalith_civ::civ_consolidate_and_smooth_ways(&net, &places, &ws.field, &water_bodies, 16, 12, p.map_width_km);

    // Real extraction: _civHierarchicalNetwork(...).ways, 10 ways (one per
    // edge -- some visible, some hidden where a busier edge already
    // claimed the whole shared corridor), ordered busiest-max-usage-first.
    assert_eq!(ways.len(), 10, "case1: way count mismatch");

    struct Expect {
        pts: Vec<(f64, f64)>,
        km: f64,
        name: &'static str,
        way_type: &'static str,
        a_idx: usize,
        b_idx: usize,
        hidden: bool,
    }
    // RE-BASELINED then RE-REVERTED, both 2026-09-21. `LARGE_ITEM_RULINGS.md`'s
    // Ruling Q moved this fixture's ways 0/2 (via
    // `civ_hierarchical_network_topology`'s edges, see
    // `golden_parity_hierarchical_network.rs`'s header) -- then Ruling T
    // special-cased this fixture's `world=true` map back to the original
    // size-primary `build_water_bodies` rule, which reverts
    // `civ_hierarchical_network_topology`'s own edges for this world back
    // to their pre-Ruling-Q shape (see that file's own header for the
    // full account), so `civ_consolidate_and_smooth_ways` reverts with
    // them. The array below is this file's own pre-Ruling-Q content (`git
    // show c6de2a2:crates/cartalith-civ/tests/golden_parity_road_consolidation.rs`),
    // re-run and confirmed against this crate's own current actual output.
    //
    // RE-BASELINED AGAIN, 2026-09-28 (ways 2 and 4 only): owner ruling,
    // follow-up to Ruling BT (`LARGE_ITEM_RULINGS.md`) -- corridor
    // consolidation's busiest-edge-first claim order (see
    // `civ_consolidate_and_smooth_ways`'s own doc comment) can leave a run
    // stranded mid-corridor, its visible end nowhere near its own A or B.
    // Way 2 (Sevjuniana -> Ghalbahrghaltazdune) used to end at (10.5, 5.5),
    // 4.3 cells short of B's (8, 9). Way 4 (Sevjuniana -> Taela'elorashade)
    // used to START at (5.5, 8.5), 6.5 cells short of A's (9, 3). Both were
    // faithful ports of the reference's own inherited behaviour -- this
    // fixture proves that behaviour was never actually downsampling-specific
    // (this world is 16x12, `sc == 1.0` throughout): the earlier diagnosis
    // in `STATUS.md`/`63bedc8d` that called it "visible only above the
    // 384-cell cap" described where the owner NOTICED it, not the bug's
    // actual scope. The extend-back pass below `civ_consolidate_and_smooth_ways`'s
    // claim loop now splices each stranded end back along its OWN edge's
    // own routed `path` -- toward its own settlement only, deliberately
    // overlapping the corridor a busier edge already claimed -- so way 2
    // now reaches (8, 9) exactly and way 4 now starts at (9, 3) exactly.
    // Re-run and confirmed against this crate's own current actual output;
    // every other way in this array (0/1/3/5-9) is untouched.
    let expected = [
        Expect { pts: vec![(10.0, 5.0), (9.0, 7.0), (8.0, 9.0)], km: 223.60679774997897, name: "Orenelywash \u{2192} Ghalbahrghaltazdune", way_type: "highway", a_idx: 3, b_idx: 2, hidden: false },
        Expect { pts: vec![(8.0, 9.0), (7.0, 9.0), (5.0, 8.0)], km: 161.80339887498948, name: "Ghalbahrghaltazdune \u{2192} Hurngarngarnhaskcairn", way_type: "highway", a_idx: 2, b_idx: 1, hidden: false },
        Expect { pts: vec![(9.0, 3.0), (10.0, 5.0), (11.0, 7.0), (9.0, 8.0), (8.0, 9.0)], km: 406.1208747436232, name: "Sevjuniana \u{2192} Ghalbahrghaltazdune", way_type: "highway", a_idx: 0, b_idx: 2, hidden: false },
        Expect { pts: vec![(9.0, 3.0), (5.0, 8.0)], km: 0.0, name: "Sevjuniana \u{2192} Hurngarngarnhaskcairn", way_type: "highway", a_idx: 0, b_idx: 1, hidden: true },
        Expect { pts: vec![(9.0, 3.0), (10.0, 5.0), (11.0, 7.0), (10.0, 8.0), (8.0, 9.0), (6.0, 9.0), (4.0, 7.0)], km: 647.5422309809327, name: "Sevjuniana \u{2192} Taela'elorashade", way_type: "highway", a_idx: 0, b_idx: 4, hidden: false },
        Expect { pts: vec![(5.0, 8.0), (10.0, 5.0)], km: 0.0, name: "Hurngarngarnhaskcairn \u{2192} Orenelywash", way_type: "highway", a_idx: 1, b_idx: 3, hidden: true },
        Expect { pts: vec![(8.0, 9.0), (4.0, 7.0)], km: 0.0, name: "Ghalbahrghaltazdune \u{2192} Taela'elorashade", way_type: "highway", a_idx: 2, b_idx: 4, hidden: true },
        Expect { pts: vec![(10.0, 5.0), (4.0, 7.0)], km: 0.0, name: "Orenelywash \u{2192} Taela'elorashade", way_type: "highway", a_idx: 3, b_idx: 4, hidden: true },
        Expect { pts: vec![(9.0, 3.0), (10.0, 5.0)], km: 0.0, name: "Sevjuniana \u{2192} Orenelywash", way_type: "regional", a_idx: 0, b_idx: 3, hidden: true },
        Expect { pts: vec![(5.0, 8.0), (4.0, 7.0)], km: 0.0, name: "Hurngarngarnhaskcairn \u{2192} Taela'elorashade", way_type: "regional", a_idx: 1, b_idx: 4, hidden: true },
    ];
    for (i, (w, e)) in ways.iter().zip(expected.iter()).enumerate() {
        let label = format!("case1 way{i}");
        assert_eq!(w.name, e.name, "{label}: name mismatch");
        assert_way_type(w.way_type, e.way_type, &label);
        assert_eq!((w.a_idx, w.b_idx, w.hidden), (e.a_idx, e.b_idx, e.hidden), "{label}: edge identity mismatch");
        assert_pts_match(&w.pts, &e.pts, &label);
        assert!((w.km - e.km).abs() < 1e-4, "{label}: km mismatch: {} vs {}", w.km, e.km);
    }
}

/// Protects: the extend-back pass added to `civ_consolidate_and_smooth_ways`
/// (owner ruling, 2026-09-28, follow-up to Ruling BT). A synthetic,
/// deliberately minimal K3-shaped topology: a busier edge A->P1 claims cells
/// reaching THREE cells into a later, quieter edge P1->P2's own path before
/// that later edge is ever processed -- exactly the "later edge's
/// near-settlement cells are pre-claimed" shape the follow-up named. Without
/// extend-back, P1->P2's visible run would start 3 cells from P1 (nowhere
/// near it); with it, the run must be spliced back along its OWN routed
/// path (not re-routed, not stolen from way A) until it reaches P1 exactly,
/// while its far end (already touching P2) is untouched.
#[test]
fn extend_back_reaches_a_settlement_whose_own_cells_a_busier_edge_already_claimed() {
    // gw=13, gh=2 (routing grid forces `rh >= 2`; row y=1 is the only row
    // used). Three settlements on one straight line so path cell ids are
    // just `13 + x`.
    let (gw, gh) = (13usize, 2usize);
    let field = vec![1.0f32; gw * gh]; // all land
    let water_bodies = vec![0u8; gw * gh];
    let p0 = cartalith_civ::NamedSettlement {
        tid: 0,
        placement: cartalith_civ::SettlementPlacement { x: 0, y: 1, suit: 0.0, faction: 1, capital: false, kind: cartalith_civ::SettlementKind::Hamlet, coastal: false },
        name: "P0".into(),
        pop: 0,
    };
    let p1 = cartalith_civ::NamedSettlement {
        tid: 0,
        placement: cartalith_civ::SettlementPlacement { x: 5, y: 1, suit: 0.0, faction: 1, capital: false, kind: cartalith_civ::SettlementKind::Hamlet, coastal: false },
        name: "P1".into(),
        pop: 0,
    };
    let p2 = cartalith_civ::NamedSettlement {
        tid: 0,
        placement: cartalith_civ::SettlementPlacement { x: 12, y: 1, suit: 0.0, faction: 1, capital: false, kind: cartalith_civ::SettlementKind::Hamlet, coastal: false },
        name: "P2".into(),
        pop: 0,
    };
    let places = vec![p0, p1, p2];

    // Edge A (busier: max usage 100) -- P0(x=0) to "P1-ish", but its own
    // path runs THREE cells past P1's own cell (x=5) to x=8, so claiming it
    // eats into edge B's own near-P1 cells. Edge B (quieter: max usage 10)
    // is P1(x=5) to P2(x=12); its first 3 path cells (x=5,6,7 -- path
    // indices 0,1,2) are exactly what A's claim swallows.
    let a_path: Vec<usize> = (0..=8).map(|x| gw + x).collect(); // row 1, x=0..8
    let b_path: Vec<usize> = (5..=12).map(|x| gw + x).collect(); // row 1, x=5..12
    let mut usage_count = vec![0u16; gw * gh];
    usage_count[a_path[0]] = 100; // A is busiest -> processed first, claims first
    usage_count[b_path[b_path.len() - 1]] = 10; // B is quieter
    let topology = cartalith_civ::HierarchicalNetworkResult {
        edges: vec![
            cartalith_civ::RoadEdge { a: 0, b: 1, path: a_path },
            cartalith_civ::RoadEdge { a: 1, b: 2, path: b_path },
        ],
        usage_count,
        degree_of: vec![0; places.len()],
    };

    let ways = cartalith_civ::civ_consolidate_and_smooth_ways(&topology, &places, &field, &water_bodies, gw, gh, 100.0);
    let b_way = ways.iter().find(|w| w.a_idx == 1 && w.b_idx == 2).expect("edge P1->P2 must still emit a way");
    assert!(!b_way.hidden, "edge B has unclaimed cells of its own (x=9..12): it must draw, not fall back to a hidden straight line");
    let (p1_pt, p2_pt) = ((5.0f64, 1.0f64), (12.0f64, 1.0f64));
    let start = *b_way.pts.first().expect("a drawn way is never empty");
    let end = *b_way.pts.last().unwrap();
    let d = |p: (f64, f64), q: (f64, f64)| ((p.0 - q.0).powi(2) + (p.1 - q.1).powi(2)).sqrt();
    assert!(d(start, p1_pt) < 1e-6, "extend-back must reach P1 exactly, got {start:?} (was 3 cells short before this fix)");
    assert!(d(end, p2_pt) < 1e-6, "the untouched end must still reach P2 exactly, got {end:?}");
}
