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

fn named(x: usize, y: usize, faction: i32, name: &str, pop: u32) -> cartalith_civ::NamedSettlement {
    cartalith_civ::NamedSettlement {
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
    let river_order = cartalith_civ::fresh_river_order(&ws.field, &ws.flow_discharge, gw, gh, ws.sea_level, world, river_density, map_width_km);
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
    let ws = cartalith_engine::generate_terrain(&p);
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
    let ws = cartalith_engine::generate_terrain(&p);
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
    let expected = [
        Expect { pts: vec![(10.0, 5.0), (9.0, 7.0), (8.0, 9.0)], km: 223.60679774997897, name: "Orenelywash \u{2192} Ghalbahrghaltazdune", way_type: "highway", a_idx: 3, b_idx: 2, hidden: false },
        Expect { pts: vec![(8.0, 9.0), (7.0, 9.0), (5.0, 8.0)], km: 161.80339887498948, name: "Ghalbahrghaltazdune \u{2192} Hurngarngarnhaskcairn", way_type: "highway", a_idx: 2, b_idx: 1, hidden: false },
        Expect { pts: vec![(9.0, 3.0), (10.0, 4.0), (10.5, 5.5)], km: 182.51407699364424, name: "Sevjuniana \u{2192} Ghalbahrghaltazdune", way_type: "highway", a_idx: 0, b_idx: 2, hidden: false },
        Expect { pts: vec![(9.0, 3.0), (5.0, 8.0)], km: 0.0, name: "Sevjuniana \u{2192} Hurngarngarnhaskcairn", way_type: "highway", a_idx: 0, b_idx: 1, hidden: true },
        Expect { pts: vec![(5.5, 8.5), (5.0, 8.0), (4.0, 7.0)], km: 141.4213562373095, name: "Sevjuniana \u{2192} Taela'elorashade", way_type: "highway", a_idx: 0, b_idx: 4, hidden: false },
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
