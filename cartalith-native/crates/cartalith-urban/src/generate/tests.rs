//! Milestone 16's tests.
//!
//! **A whole-subsystem golden, and it is captured rather than claimed.**
//! `golden.rs` holds the frozen reference engine's own `generate()` output for
//! 29 scenarios, taken by `cartalith-native/tools/um_capture.js` — a harness
//! this milestone had to *reconstruct*, because none was committed: each
//! earlier `golden.rs` was produced by a script that was thrown away and
//! survives only as prose in its own header. That harness is now in the tree
//! beside `ponycheck.js`, and it re-runs every assertion the convention in
//! `URBAN_MORPHOLOGY_SCOPE.md` names (the two structural boundary asserts, the
//! comment-balance check plus the orphan counter, the `mulberry32` negative
//! control) rather than trusting them.
//!
//! # The comparison, in the order it fails
//!
//! `hashModel` is the reference's own whole-model instrument and is **checked
//! last**, because it is the least informative failure this file can produce
//! and because it is coarse by construction: it rounds coordinates to the
//! centimetre and areas to a tenth of a square metre. What fails first is the
//! shape — counts, district and detail histograms, which branch each stage
//! took — then `computeMetrics`, which is eleven full-precision doubles read
//! off the **final** graph, i.e. after `lanePass`, `removeWaterCrossings`,
//! `privatizeAlleys` and `clearFortZone`. That readout is the strongest single
//! probe here: `totalLen` alone is a sum over every live segment.
//!
//! `hashModel` also hashes **no details at all**, so `detail_bbox`,
//! `waterway_pts`/`waterway_first` and the detail histogram are the only things
//! watching the hinterland half of the model. That is not theoretical: a
//! mutation to `buildWaterway`'s radius survived two sweeps before those columns
//! existed.
//!
//! # What this milestone is actually testing
//!
//! Not the stages — every one of them is golden-verified in its own module.
//! **The orchestration**: 29 calls in the reference's order, the branch
//! conditions between them, and the arithmetic `generate()` does itself. The
//! matrix is built to move each of those, over 29 scenarios:
//!
//! | arm | scenarios reaching it |
//! |---|---|
//! | organic / radial planning | 23 / 6 |
//! | walled / unwalled | 25 / 4 |
//! | star fort | 8 |
//! | canal-fed star fort (`fort.canalFed`) | 1 |
//! | harbour built / none / `'unnavigable'` | 25 / 4 / 1 |
//! | real river bridges (`detectRiverCrossings`) | 3 |
//! | markets / none | 25 / 4 |
//! | civic hall / none | 24 / 5 |
//! | churches / none | 19 / 10 |
//! | games building | 19 |
//! | ruined parcels | 1 |
//! | the Venus canal | 5 |
//! | a swept (`cleared`) parcel | 26 |
//!
//! The capture **refuses to write** unless every one of those is reached, and
//! refuses per scenario unless the graph, blocks, parcels and details are all
//! non-empty and `pop` is finite. Three subsystems in this project have shipped
//! a harness that produced silently empty output and passed every structural
//! check; that is what those gates are for. They earned their keep twice here —
//! see the note on `terrain_ctx` below.
//!
//! # The mutation sweep: 49 of 56 killed, and every survivor accounted for
//!
//! Golden-matching is necessary and not sufficient. Every constant, every
//! branch condition and every ordering in `generate()` was mutated by one unit
//! (or inverted) and the golden re-run. Five of the seven scenarios added after
//! the first sweep exist *because* a mutant survived it, and each is quantised
//! or sits exactly on a threshold: `popFloorClamp` (pop 100 → the 400 floor),
//! `churchRoundBoundary` (pop 6500, where `(pop/5.2)/500` is EXACTLY 2.5),
//! `hamletBoundary` (pop exactly 600), `ageFloorClamp` (age 10 → the 30 floor),
//! `landlockedFortDry` and `venusLandlockedCanal` (the only place `wetMoat` is
//! visible, since every other fort sits within 175 m of water and its ditch is
//! already wet), `hostRouteEndsOnly`, and `venusTinyCanal` (the only Venus town
//! small enough that the canal radius is `maxRF*0.95` rather than the box-edge
//! clamp). Adding them killed nine further mutants.
//!
//! **The seven survivors, each with its reason — none is an untested constant:**
//!
//! 1. `targetLen`'s 1600 floor → 1601. Sub-resolution: 1 m more budget across 8
//!    epochs never adds a segment. **1600 → 2600 is killed**, so the constant is
//!    pinned to within what `grow` can observe.
//! 2. The head count summed (`s + 5.2` per parcel) versus `5.2 * n`. The
//!    accumulated rounding over ~2 000 additions is ~1e-8 absolute against
//!    `Math.round`'s 0.5 — indistinguishable unless a sum lands within 1e-8 of a
//!    half. Written as the reference writes it anyway; do not "simplify" it.
//! 3. `fortified`'s `wallGates.scheme === 'organic'` guard. **Dead**: both live
//!    profiles set `'organic'`. Pinned instead by
//!    [`two_generate_guards_are_dead_against_the_live_profiles`].
//! 4. The lane-pass early break. Provably a no-op: `lanePass` is deterministic
//!    on `(seed, epochs, graph)`, so once it adds nothing it adds nothing again.
//!    The break is the reference's and is kept.
//! 5. `detectRiverCrossings` moved above the last two sweeps. See
//!    [`every_recorded_bridge_has_a_live_road_on_it`] for the mechanism and for
//!    the 180-run scan that found zero differing.
//! 6. `clearFortZone`'s `if(wallState.ring)` guard. Redundant: `clear_fort_zone`
//!    returns at its own first line without a ring. The reference has the same
//!    double guard.
//! 7. `profile.markets`. **Dead**, same as 3, and pinned by the same test.
//!
//! # What is NOT reached, stated rather than papered over
//!
//! - **`Crossings::Ford`.** It needs a through-town with a `bridgePt` where no
//!   live road crosses the real centreline, which in practice means a town
//!   degenerate enough to fail the shape gate. Zero of 29 scenarios reach it.
//! - **`harbourInvalid = 'cliff'`.** Only `'unnavigable'` is reached.
//! - **A `None` plaza.** Every scenario laid a live primary, so `build_plaza`
//!   never took its `if(!be) return null` arm here. `plaza`'s own goldens do.
//! - **`buildPrimariesFromPaths` on the radial branch** — unreachable by
//!   construction; that branch calls neither primary builder.
//!
//! # The two synthetic rasters
//!
//! [`water_ctx`] and [`terrain_ctx`] are built from **integer arithmetic and
//! one exact division**, on this side and in the capture script alike, so no
//! literal raster has to be embedded and the two cannot disagree in the last
//! bit. The terrain values are divided by 1000 for a real reason recorded in
//! the capture script: `_umTerrainCtx` samples the host's *normalised* 0..1
//! elevation field, and `slope()` multiplies its finite difference by 900 — a
//! fixture in metres makes every square metre of the box unbuildable, and the
//! reference then produces a 30-node, zero-block "town" that the shape gate
//! (correctly) refuses. Both fixtures were the reference behaving correctly on
//! a silly input; the gate is what turned "the golden passed" into "the golden
//! would have been vacuous".

use super::*;
use crate::rules::{CULTURE_PROFILES, ParcelPatch, RulesPatch, SettlementPatch, StreetPatch};

mod golden;

// ------------------------------------------------------------- the fixtures --

const CELL_M: f64 = 22.0;
const MW: usize = 78;
const MH: usize = 57;

/// The river's row, per column — two integer steps down. Integer arithmetic
/// only; see this module's header.
fn river_row(i: usize) -> usize {
    27 + if i < 26 { 0 } else if i < 52 { 1 } else { 2 }
}

fn water_ctx(order: f64) -> WaterCtx {
    let mut mask = vec![0u8; MW * MH];
    let mut dt = vec![0.0f64; MW * MH];
    for j in 0..MH {
        for i in 0..MW {
            let d = (j as i64 - river_row(i) as i64).unsigned_abs() as usize;
            mask[j * MW + i] = u8::from(d <= 1);
            dt[j * MW + i] = if d <= 1 { 0.0 } else { (d - 1) as f64 };
        }
    }
    let mut river_path = Vec::new();
    let mut i = 0;
    while i < MW {
        river_path.push(Vec2::new(
            (i as f64 + 0.5) * CELL_M,
            (river_row(i) as f64 + 0.5) * CELL_M,
        ));
        i += 6;
    }
    WaterCtx {
        mask,
        dt,
        mw: MW,
        mh: MH,
        cell_m: CELL_M,
        river_path: Some(river_path),
        river_width_m: Some(26.0),
        river_order: order,
        sea_lake_cells: 0.0,
    }
}

fn terrain_ctx() -> TerrainCtx {
    let mut grid = vec![0.0f64; MW * MH];
    for j in 0..MH {
        for i in 0..MW {
            let a = if (i as f64) < MW as f64 / 2.0 { i } else { MW - i } * 2;
            let b = if (j as f64) < MH as f64 / 2.0 { j } else { MH - j };
            grid[j * MW + i] = (a + b) as f64 / 1000.0;
        }
    }
    TerrainCtx { grid, mw: MW, mh: MH, cell_m: CELL_M, h_min: 0.0, h_max: 106.0 / 1000.0 }
}

/// `opts.routeEnds` — integer metres, so nothing rounds through the fixture.
fn route_ends() -> Vec<Vec2> {
    vec![Vec2::new(0.0, 300.0), Vec2::new(1700.0, 900.0), Vec2::new(800.0, 0.0)]
}

/// `opts.primaryPaths`.
fn primary_paths() -> Vec<Vec<Vec2>> {
    vec![
        vec![
            Vec2::new(0.0, 400.0),
            Vec2::new(600.0, 560.0),
            Vec2::new(1100.0, 640.0),
            Vec2::new(1700.0, 720.0),
        ],
        vec![Vec2::new(850.0, 0.0), Vec2::new(830.0, 500.0), Vec2::new(900.0, 1250.0)],
    ]
}

/// `o_rules == 2`: a dead-end bias and nothing else, so `privatize_alleys` can
/// kill an edge on a town that also has a real river. That is what makes
/// `detect_river_crossings`' position **after** it observable — with no killed
/// edge, running it early and running it late give the same answer, which is
/// how that mutant survived the first sweep.
fn dead_end_patch() -> RulesPatch {
    RulesPatch {
        street: Some(StreetPatch { dead_end_bias: Some(0.38), ..StreetPatch::default() }),
        ..RulesPatch::default()
    }
}

/// `o_rules == 1`. `dead_end_bias` is the load-bearing field: both live profiles
/// and `DEFAULT_RULES` set it to 0, so without this `privatize_alleys` returns
/// at its first line in every other scenario.
fn rules_patch() -> RulesPatch {
    RulesPatch {
        street: Some(StreetPatch {
            segment_length_median: Some(46.0),
            dead_end_bias: Some(0.18),
            parallel_street_spacing: Some(31.0),
            ..StreetPatch::default()
        }),
        parcels: Some(ParcelPatch {
            plot_depth_variance: Some(0.4),
            subdivision_cap: Some(4.0),
            ..ParcelPatch::default()
        }),
        settlement: Some(SettlementPatch {
            carrying_capacity_weight: Some(0.0),
            ..SettlementPatch::default()
        }),
        meta: None,
    }
}

fn opt_str(s: &str) -> Option<String> {
    if s.is_empty() { None } else { Some(s.to_string()) }
}

/// Rebuild the exact `opts` object the capture ran, from the fixture columns of
/// the case itself — so the two matrices cannot drift apart.
fn opts_for(c: &golden::Case) -> GenOpts {
    GenOpts {
        culture: opt_str(c.o_culture),
        rules: match c.o_rules {
            1 => Some(rules_patch()),
            2 => Some(dead_end_patch()),
            _ => None,
        },
        site: opt_str(c.o_site),
        terrain_aware: c.o_terrain_aware,
        ruined: c.o_ruined,
        wall_generations: c.o_wall_generations,
        settlement_age: c.o_settlement_age,
        epochs: c.o_epochs,
        pop: c.o_pop,
        walls: c.o_walls,
        fortified: c.o_fortified,
        wall_style: opt_str(c.o_wall_style),
        faith: opt_str(c.o_faith),
        civic_style: opt_str(c.o_civic_style),
        harbour_defence: opt_str(c.o_harbour_defence),
        harbour_scale: c.o_harbour_scale,
        water: c.o_water_order.map(water_ctx),
        terrain: if c.o_terrain { Some(terrain_ctx()) } else { None },
        economy: opt_str(c.o_economy).map(|s| Economy {
            specialisation: Some(s),
            // Unread by this crate; the *bearing* travels in `ore_bearing`
            // because `Economy::ore_bearing` is a `bool` and the reference's
            // `oreBearing` is a nullable angle. See the module header.
            ore_bearing: true,
        }),
        ore_bearing: c.o_ore_bearing,
        route_ends: if c.o_route_ends { route_ends() } else { Vec::new() },
        primary_paths: if c.o_primary_paths { primary_paths() } else { Vec::new() },
    }
}

// ------------------------------------------------------------- the assertion --

#[test]
fn whole_subsystem_matches_reference() {
    assert_eq!(golden::CASES.len(), 29, "the golden lost cases");
    for c in golden::CASES {
        let t = generate(c.seed, &opts_for(c));
        let n = c.name;

        // The scalars `generate()` derives itself — these fail first because a
        // wrong clamp makes every later number wrong for one reason.
        assert_eq!(t.pop_target, c.pop_target, "{n}: popTarget");
        assert_eq!(t.settlement_age, c.settlement_age, "{n}: settlementAge");
        assert_eq!(t.epochs, c.epochs, "{n}: epochs");
        assert_eq!(t.walls, c.walls, "{n}: walls");
        assert_eq!(t.fortified, c.fortified, "{n}: fortified");
        assert_eq!(t.fort_requested, c.fort_requested, "{n}: fortRequested");
        assert_eq!(t.culture, c.culture, "{n}: culture");
        assert_eq!(t.site.kind, c.site_kind, "{n}: site.kind");
        assert_eq!(t.through, c.through, "{n}: through");
        assert_eq!(t.pop, c.pop, "{n}: pop");
        assert_eq!(t.wm, SITE_WM, "{n}: Wm");
        assert_eq!(t.hm, SITE_HM, "{n}: Hm");
        assert_eq!(t.fort_min, FORT_MIN, "{n}: fortMin");

        // Shape. A count mismatch localises far better than a hash mismatch.
        assert_eq!(t.graph.nodes.len(), c.nodes, "{n}: nodes");
        assert_eq!(t.graph.edges.len(), c.live_edges, "{n}: live edges");
        assert_eq!(t.blocks.len(), c.blocks, "{n}: blocks");
        assert_eq!(t.parcels.len(), c.parcels, "{n}: parcels");
        assert_eq!(t.buildings.len(), c.buildings, "{n}: buildings");
        assert_eq!(t.churches.len(), c.churches, "{n}: churches");
        assert_eq!(t.markets.len(), c.markets, "{n}: markets");
        assert_eq!(t.games.len(), c.games, "{n}: games");
        assert_eq!(t.details.len(), c.details, "{n}: details");
        assert_eq!(
            t.parcels.iter().filter(|p| p.ruined).count(),
            c.ruined_parcels,
            "{n}: ruined parcels"
        );
        assert_eq!(
            t.parcels.iter().filter(|p| p.cleared).count(),
            c.cleared_parcels,
            "{n}: cleared parcels"
        );
        assert_eq!(histogram(t.parcels.iter().map(|p| p.district)), c.district_counts, "{n}: districts");
        assert_eq!(histogram(t.details.iter().map(|d| d.kind)), c.detail_kinds, "{n}: detail kinds");

        // Which branch each optional stage took.
        assert_eq!(t.plaza.is_some(), c.has_plaza, "{n}: plaza present");
        assert_eq!(
            t.plaza.as_ref().map(|p| (p.center.x, p.center.y)),
            c.plaza_center,
            "{n}: plaza centre"
        );
        assert_eq!(t.harbour.is_some(), c.has_harbour, "{n}: harbour present");
        assert_eq!(t.site.harbour_invalid, c.harbour_invalid, "{n}: harbourInvalid");
        assert_eq!(t.civic.is_some(), c.has_civic, "{n}: civic present");
        assert_eq!(t.civic.as_ref().map(|v| v.style.as_str()), c.civic_style, "{n}: civic style");
        assert_eq!(t.wall.ring.as_ref().map(Vec::len), c.wall_ring, "{n}: wall ring");
        assert_eq!(t.wall.gates.len(), c.wall_gates, "{n}: wall gates");
        assert_eq!(t.wall.style, c.wall_style, "{n}: wall style");
        assert_eq!(t.wall.epoch, c.wall_epoch, "{n}: wall epoch");
        assert_eq!(t.wall.fort.is_some(), c.has_fort, "{n}: fort present");
        let f = t.wall.fort.as_ref();
        assert_eq!(f.is_some_and(|f| f.canal_fed), c.fort_canal_fed, "{n}: fort canalFed");
        assert_eq!(f.is_some_and(|f| f.wet_ditch), c.fort_wet_ditch, "{n}: fort wetDitch");
        assert_eq!(f.is_some_and(|f| f.double_moat), c.fort_double_moat, "{n}: fort doubleMoat");
        assert_eq!(
            f.is_some_and(|f| f.outer_moat.is_some()),
            c.fort_outer_moat,
            "{n}: fort outerMoat"
        );
        assert_eq!(f.map_or(0, |f| f.bastions.len()), c.fort_bastions, "{n}: fort bastions");
        assert_eq!(f.map_or(0, |f| f.trace.len()), c.fort_trace, "{n}: fort trace");
        assert_eq!(f.map_or(0, |f| f.ravelins.len()), c.fort_ravelins, "{n}: fort ravelins");
        assert_eq!(f.map_or(0.0, |f| f.glacis_off), c.fort_glacis_off, "{n}: fort glacisOff");
        assert_eq!(t.wall.generation.unwrap_or(1).max(1), c.wall_generation, "{n}: wall generation");
        assert_eq!(t.wall.history.len(), c.wall_history, "{n}: wall history");
        // `hashModel` hashes no details, so this is the only thing watching them.
        assert_eq!(detail_bbox(&t.details), c.detail_bbox, "{n}: detail bbox");
        assert_eq!(
            t.site.route_ends.iter().map(|p| (p.x, p.y)).collect::<Vec<_>>(),
            c.route_ends,
            "{n}: site.routeEnds"
        );
        let canal = t.details.iter().find(|d| d.kind == "waterway");
        assert_eq!(
            canal.map_or(0, |d| match &d.geom {
                DetailGeom::Poly(p) => p.len(),
                _ => 0,
            }),
            c.waterway_pts,
            "{n}: canal vertices"
        );
        assert_eq!(
            canal.and_then(|d| match &d.geom {
                DetailGeom::Poly(p) => p.first().map(|q| (q.x, q.y)),
                _ => None,
            }),
            c.waterway_first,
            "{n}: canal first vertex"
        );
        assert_eq!(t.site.bridges.as_ref().map(Vec::len), c.bridges, "{n}: bridges");
        assert_eq!(t.site.ford.is_some(), c.has_ford, "{n}: ford");

        // `computeMetrics` — eleven full-precision doubles over the FINAL graph.
        assert_eq!(t.metrics.nodes, c.m_nodes, "{n}: metrics.nodes");
        assert_eq!(t.metrics.edges, c.m_edges, "{n}: metrics.edges");
        assert_eq!(t.metrics.total_len, c.m_total_len, "{n}: metrics.totalLen");
        assert_eq!(t.metrics.dead_end_share, c.m_dead_end_share, "{n}: metrics.deadEndShare");
        assert_eq!(t.metrics.deg3_share, c.m_deg3_share, "{n}: metrics.deg3Share");
        assert_eq!(t.metrics.deg4_share, c.m_deg4_share, "{n}: metrics.deg4Share");
        assert_eq!(t.metrics.mean_deg, c.m_mean_deg, "{n}: metrics.meanDeg");
        assert_eq!(t.metrics.median_seg, c.m_median_seg, "{n}: metrics.medianSeg");
        assert_eq!(t.metrics.meshedness, c.m_meshedness, "{n}: metrics.meshedness");
        assert_eq!(
            t.metrics.median_block_area, c.m_median_block_area,
            "{n}: metrics.medianBlockArea"
        );
        assert_eq!(t.metrics.median_frontage, c.m_median_frontage, "{n}: metrics.medianFrontage");

        // Written-out anchors at both ends of each list.
        assert_eq!((t.anchors.market.x, t.anchors.market.y), c.market, "{n}: market");
        assert_eq!(t.anchors.prov, c.market_prov, "{n}: market prov");
        let fe = &t.graph.edges[0];
        let le = &t.graph.edges[t.graph.edges.len() - 1];
        assert_eq!((fe.a, fe.b, fe.cls, fe.w), c.first_edge, "{n}: first edge");
        assert_eq!((le.a, le.b, le.cls, le.w), c.last_edge, "{n}: last edge");
        let fnd = &t.graph.nodes[0];
        let lnd = &t.graph.nodes[t.graph.nodes.len() - 1];
        assert_eq!((fnd.x, fnd.y), c.first_node, "{n}: first node");
        assert_eq!((lnd.x, lnd.y), c.last_node, "{n}: last node");
        let fp = &t.parcels[0];
        let lp = &t.parcels[t.parcels.len() - 1];
        assert_eq!((fp.par.id.as_str(), fp.par.area, fp.district), c.first_parcel, "{n}: first parcel");
        assert_eq!((lp.par.id.as_str(), lp.par.area, lp.district), c.last_parcel, "{n}: last parcel");

        // And finally the reference's own whole-model hash.
        assert_eq!(hash_model(&t), c.hash, "{n}: hashModel");
    }
}

/// `(minX, minY, maxX, maxY)` over every point of every detail — the capture's
/// `detailBBox`, point for point.
fn detail_bbox(details: &[Detail]) -> (f64, f64, f64, f64) {
    let (mut x0, mut y0, mut x1, mut y1) =
        (f64::INFINITY, f64::INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY);
    let mut put = |p: Vec2| {
        if p.x < x0 {
            x0 = p.x;
        }
        if p.y < y0 {
            y0 = p.y;
        }
        if p.x > x1 {
            x1 = p.x;
        }
        if p.y > y1 {
            y1 = p.y;
        }
    };
    for d in details {
        match &d.geom {
            DetailGeom::Point(p) => put(*p),
            DetailGeom::Seg(a, b) => {
                put(*a);
                put(*b);
            }
            DetailGeom::Poly(poly) => {
                for q in poly {
                    put(*q);
                }
            }
        }
    }
    (x0, y0, x1, y1)
}

/// `(key, count)` pairs sorted by key — the same shape the capture emits.
fn histogram<'a>(it: impl Iterator<Item = &'a str>) -> Vec<(&'a str, usize)> {
    let mut m: std::collections::BTreeMap<&str, usize> = std::collections::BTreeMap::new();
    for k in it {
        *m.entry(k).or_insert(0) += 1;
    }
    m.into_iter().collect()
}

// ------------------------------------------------------- unit-level pinning --

/// `hashModel` pushes `String(Math.round(x))`, and JS prints `-0` as `"0"`.
///
/// The one place a plain `format!("{v}")` would diverge: Rust's `Display` for
/// `-0.0` is `"-0"`. Reachable — `js_round(-0.4)` is `-0.0`, which a node at
/// `x = -0.004` produces after the `*100`.
#[test]
fn js_int_str_matches_js_string_of_round() {
    assert_eq!(js_int_str(js_round(-0.4)), "0");
    assert_eq!(js_int_str(-0.0), "0");
    assert_eq!(js_int_str(0.0), "0");
    assert_eq!(js_int_str(js_round(-0.5)), "0", "JS rounds -0.5 to -0, which prints as 0");
    assert_eq!(js_int_str(js_round(0.5)), "1");
    assert_eq!(js_int_str(js_round(-1.5)), "-1", "JS rounds half toward +Infinity");
    assert_eq!(js_int_str(f64::NAN), "NaN");
    assert_eq!(js_int_str(f64::INFINITY), "Infinity");
    assert_eq!(js_int_str(f64::NEG_INFINITY), "-Infinity");
    assert_eq!(js_int_str(-12345.0), "-12345");
}

/// The empty options object really is the reference's own defaults, and
/// `generate(seed)` with no options produces the same town as
/// `generate(seed, {})`.
#[test]
fn defaults_are_the_references_defaults() {
    let t = generate(12345, &GenOpts::default());
    assert_eq!(t.pop_target, 5000.0);
    assert_eq!(t.epochs, 8);
    assert_eq!(t.settlement_age, 300.0);
    assert_eq!(t.site.kind, "river");
    assert_eq!(t.culture, "medieval");
    assert!(t.walls, "opts.walls !== false, so an absent key is walled");
    assert!(!t.fortified);
    // The first golden case is this exact call.
    assert_eq!(hash_model(&t), golden::CASES[0].hash);
}

/// `opts.walls` is tested with `!== false`, so only an explicit `false`
/// disables the enclosure — a mutation to `!opts.walls.unwrap_or(true)` would
/// pass every other test here.
#[test]
fn walls_is_a_strict_false_test() {
    let walled = generate(4242, &GenOpts { pop: Some(9000.0), ..GenOpts::default() });
    let asked = generate(
        4242,
        &GenOpts { pop: Some(9000.0), walls: Some(true), ..GenOpts::default() },
    );
    let off = generate(
        4242,
        &GenOpts { pop: Some(9000.0), walls: Some(false), ..GenOpts::default() },
    );
    assert!(walled.walls && asked.walls && !off.walls);
    assert_eq!(hash_model(&walled), hash_model(&asked), "absent and true must agree");
    assert!(off.wall.ring.is_none(), "walls:false must leave no circuit");
}

/// The anachronism guard: a bastioned trace needs the request **and** the
/// population **and** an enclosure **and** an `'organic'` gate scheme. The
/// population test is `>=`, and `fortMinBoundary` in the matrix sits exactly on
/// it — this pins the other side of the boundary.
#[test]
fn fortified_needs_all_four_conditions() {
    let at = |pop: f64, walls: Option<bool>| {
        generate(
            1000,
            &GenOpts { pop: Some(pop), fortified: true, walls, ..GenOpts::default() },
        )
        .fortified
    };
    assert!(at(FORT_MIN, None), "exactly at fortMin is fortified (>=, not >)");
    assert!(!at(FORT_MIN - 1.0, None), "one below is not");
    assert!(!at(FORT_MIN, Some(false)), "an unwalled town cannot be bastioned");
    assert!(
        !generate(1000, &GenOpts { pop: Some(FORT_MIN), ..GenOpts::default() }).fortified,
        "unrequested is not fortified"
    );
}

/// Two of `generate()`'s guards are **dead against today's profile roster**, and
/// no golden over `generate()` can pin them — a mutation deleting either
/// survives the whole matrix. This pins the *reason* instead, so the day a third
/// profile arrives the failure lands here rather than silently re-animating an
/// untested branch.
///
/// - `fortified` requires `profile.wallGates.scheme === 'organic'` — the
///   anachronism guard, since a trace italienne answers c.1500 gunpowder
///   artillery.
/// - `buildMarkets` runs only when `profile.markets` — a hook for a commerce
///   that did not run through a chartered square.
#[test]
fn two_generate_guards_are_dead_against_the_live_profiles() {
    assert_eq!(CULTURE_PROFILES.len(), 2, "a third profile can re-animate both guards below");
    for p in CULTURE_PROFILES {
        assert_eq!(p.wall_gates_scheme, "organic", "{}: the anachronism guard is inert", p.id);
        assert!(p.markets, "{}: the buildMarkets guard is inert", p.id);
    }
    // And `profile.noWalls`, which line 30955 reads and no profile defines — see
    // the module header. There is no such field to assert; what can be asserted
    // is that this port's `walls` really is `opts.walls !== false` alone, which
    // `walls_is_a_strict_false_test` above does.
}

/// v2.73, direct rather than via the golden: a village-tier settlement's plaza
/// is a green with no market cross, and a chartered town's is a market place
/// with one — checked positively on both sides, not just "no error".
#[test]
fn a_village_gets_a_green_and_a_chartered_town_gets_a_market_cross() {
    let village = generate(4242, &GenOpts { pop: Some(450.0), ..GenOpts::default() });
    assert_eq!(village.pop_target, 450.0, "below the chartered-town population");
    let vp = village.plaza.as_ref().expect("a village still has an open centre");
    assert_eq!(vp.kind, crate::plaza::PlazaKind::Green, "450 people is a village, not a chartered town");
    assert!(
        !village.details.iter().any(|d| d.kind == "cross"),
        "a village green must carry no market cross"
    );

    let town = generate(4242, &GenOpts { pop: Some(9000.0), ..GenOpts::default() });
    assert_eq!(town.pop_target, 9000.0, "well over the chartered-town population");
    let tp = town.plaza.as_ref().expect("a chartered town has a plaza");
    assert_eq!(tp.kind, crate::plaza::PlazaKind::Market, "9000 people is a chartered town");
    assert!(
        town.details.iter().any(|d| d.kind == "cross"),
        "a chartered town's plaza must carry its market cross"
    );
}

/// `detectRiverCrossings` must run on the FINAL graph. Nothing in its signature
/// can enforce that, so this asserts the consequence the reference's own comment
/// names: every recorded bridge sits on a **live** edge.
///
/// **This is a property test, not a proof of the ordering, and the difference is
/// worth stating.** Moving `detect_river_crossings` above `privatizeAlleys` and
/// `clearFortZone` does not change any result in this matrix — 180 further runs
/// off the capture harness (60 seeds × three real-water shapes, fortified and
/// bastioned, dead-end bias 0.4) found **zero** differing. The mechanism is
/// specific: after `removeWaterCrossings` every surviving edge that crosses the
/// real centreline is `cls == "primary"`, and `privatizeAlleys` filters
/// `cls == "street"`, so it can never kill one; `clearFortZone` can in principle,
/// and did not in any run. The order here is the reference's, and it is right;
/// what this file cannot claim is that a golden would catch reversing it.
#[test]
fn every_recorded_bridge_has_a_live_road_on_it() {
    let mut checked = 0;
    for c in golden::CASES {
        let t = generate(c.seed, &opts_for(c));
        let Some(bridges) = &t.site.bridges else { continue };
        assert!(!bridges.is_empty(), "{}: an empty bridge list should be None", c.name);
        for b in bridges {
            // The crossing was found on some live edge, so some live edge must
            // still pass within a segment's reach of it.
            let near = t.graph.edges.iter().any(|e| {
                let a = t.graph.nodes[e.a].pt();
                let z = t.graph.nodes[e.b].pt();
                crate::geom::dist_pt_seg(b.pt, a, z) < 1e-9
            });
            assert!(near, "{}: bridge at {:?} has no live road on it", c.name, b.pt);
            checked += 1;
        }
    }
    assert!(checked > 0, "no scenario produced a bridge — this test proved nothing");
}

// ----------------------------------------------- lots against the wall (Ruling H) --

/// The departure `crate::wallside` makes, pinned on real towns rather than on
/// its own helpers: the golden above only sees counts and a hash, which would
/// pass on lots placed anywhere.
#[test]
fn lots_back_onto_the_wall_and_the_faubourg_survives_the_rampart_sweep() {
    use crate::geom::point_in_poly;
    use crate::growth::dist_to_line;
    use crate::wallside::WallBacking;
    let case = |name: &str| golden::CASES.iter().find(|c| c.name == name).expect("case");

    let c = case("riverDefault");
    let t = generate(c.seed, &opts_for(c));
    assert_eq!(t.wall.style, "curtain");
    let ring = t.wall.ring.as_ref().expect("walled");
    // `dist_to_line` walks an open polyline; an all-land curtain's arc is the
    // whole ring, so close it or a lot on the seam measures to the far side.
    let mut arc = t.wall.land_arc.clone().expect("walled");
    if Some(&arc) == t.wall.ring.as_ref() {
        arc.push(arc[0]);
    }
    let arc = &arc;
    let lots = |wb| t.parcels.iter().filter(move |p| p.par.wall_backing == wb).collect::<Vec<_>>();
    let (inside, outside, behind) =
        (lots(WallBacking::Inside), lots(WallBacking::Outside), lots(WallBacking::OutsideRow));
    // Literals from the re-derived golden (`district_counts` faubourg 86 =
    // a first row of 43, exactly the single-row pass's, plus 43 behind it), not
    // from the code under test. 55 behind until the 2026-09-23 grain pass
    // (`wallside::taper_end_chance`, `BACK_JITTER`) re-drew the rows behind.
    assert_eq!((inside.len(), outside.len(), behind.len()), (43, 43, 43));

    // The rows behind: a CLUSTER, not a line (owner, 2026-09-22 follow-up).
    // Each lot wholly outside, spared by the sweep and built, with its back
    // line off the wall face — and in more than one band of distance from it.
    let mut bands = std::collections::BTreeSet::new();
    for p in &behind {
        assert_eq!(p.district, "faubourg", "{}", p.par.id);
        assert!(!p.cleared, "{}: the rampart sweep must spare every faubourg row", p.par.id);
        assert!(p.par.poly.iter().all(|q| !point_in_poly(*q, ring)), "{}", p.par.id);
        assert!(t.buildings.iter().any(|b| b.parcel == p.par.id), "{}: faubourg lot unbuilt", p.par.id);
        let d = dist_to_line(p.par.poly[3].lerp(p.par.poly[2], 0.5), arc);
        // Behind a first row at least 6 m deep; within 5 rows of <= 12 m and
        // 3.5 m lanes.
        assert!((2.25 + 6.0 - 1.0..=2.25 + 5.0 * 15.5).contains(&d), "{}: back line {d} m out", p.par.id);
        bands.insert((d / 10.0) as u32);
    }
    assert!(bands.len() >= 3, "a cluster spans >= 3 ten-metre bands off the wall, got {bands:?}");

    for p in &outside {
        assert_eq!(p.district, "faubourg", "{}", p.par.id);
        assert!(!p.cleared, "{}: the rampart sweep must spare a faubourg lot", p.par.id);
        // The back line sits on the curtain's drawn face — half of its 4.5 m
        // stroke out from the centreline — less at most `MAX_BOW` (1 m) where
        // the straight back line spans a bend in the wall.
        let back = p.par.poly[3].lerp(p.par.poly[2], 0.5);
        let d = dist_to_line(back, arc);
        assert!((1.25 - 1e-9..=2.25 + 1e-9).contains(&d), "{}: back line {d} m from the wall", p.par.id);
        assert!(p.par.poly.iter().all(|q| !point_in_poly(*q, ring)), "{}", p.par.id);
        // And it is built on, against that face. Removing the sweep exemption
        // in `generate()` deletes these buildings, so this fails with it.
        let b = t.buildings.iter().find(|b| b.parcel == p.par.id);
        let b = b.unwrap_or_else(|| panic!("{}: faubourg lot unbuilt", p.par.id));
        assert!(b.poly.iter().any(|q| dist_to_line(*q, arc) < 2.25 + 0.5), "{}", p.par.id);
    }
    for p in &inside {
        assert!(p.par.poly.iter().all(|q| point_in_poly(*q, ring)), "{}", p.par.id);
        let back = p.par.poly[3].lerp(p.par.poly[2], 0.5);
        let d = dist_to_line(back, arc);
        assert!((2.25 - 1e-9..=3.25 + 1e-9).contains(&d), "{}: back line {d} m from the wall", p.par.id);
    }
    // Churches stay off wall lots (one-lot precincts otherwise).
    for ch in &t.churches {
        assert!(ch.yard.iter().all(|id| !id.starts_with("wallin") && !id.starts_with("faub")));
    }

    // A glacis stays clear, on either planning branch.
    for name in ["wallStyleBastioned", "venusFortCanal"] {
        let c = case(name);
        let t = generate(c.seed, &opts_for(c));
        assert!(t.wall.ring.is_some(), "{name} is walled");
        assert_eq!(t.wall.style, "bastioned", "{name}");
        assert!(t.parcels.iter().all(|p| p.par.wall_backing == WallBacking::No), "{name}");
    }
}

/// Ruling AA (owner, 2026-09-23): the radial plan carries the wall lots and the
/// faubourg too. Pinned on a real Venus town, not on `wallside`'s helpers — and
/// against the three things the change must NOT do: move a street, move a
/// street-platted lot, or re-band a Venus building on one.
#[test]
fn a_radial_town_builds_against_its_wall_without_moving_its_wedge_blocks() {
    use crate::geom::point_in_poly;
    use crate::wallside::WallBacking;
    let c = golden::CASES.iter().find(|c| c.name == "venusRadial").expect("case");
    let t = generate(c.seed, &opts_for(c));
    assert_eq!(t.culture, "venus");
    assert_eq!(t.wall.style, "curtain");
    let ring = t.wall.ring.as_ref().expect("walled");
    let n = |wb| t.parcels.iter().filter(|p| p.par.wall_backing == wb).count();
    // Literals from the re-derived golden, not from the code under test.
    assert_eq!(
        (n(WallBacking::Inside), n(WallBacking::Outside), n(WallBacking::OutsideRow)),
        // 133 behind until the 2026-09-23 grain pass re-drew the rows behind.
        (64, 74, 138)
    );
    // The street-platted lots are exactly the 738 the reference placed, in
    // order, and the wedge blocks are its 42 — the wall lots are appended.
    assert_eq!(t.blocks.len(), 42);
    assert!(t.parcels[..738].iter().all(|p| p.par.wall_backing == WallBacking::No));
    assert_eq!(t.parcels[737].par.id, "par758");
    for p in &t.parcels[738..] {
        let outside = p.par.wall_backing.is_faubourg();
        assert_eq!(p.district == "faubourg", outside, "{}", p.par.id);
        assert!(p.par.poly.iter().all(|q| point_in_poly(*q, ring) != outside), "{}", p.par.id);
        if outside {
            assert!(!p.cleared, "{}: the rampart sweep must spare the faubourg", p.par.id);
        }
    }
    // Every wall lot takes the wall grammar, never a Venus band: no pavilion,
    // warehouse, apartment, courtyard house or machiya stands on one.
    let wall_ids: std::collections::HashSet<&str> =
        t.parcels[738..].iter().map(|p| p.par.id.as_str()).collect();
    let on_wall: Vec<_> = t.buildings.iter().filter(|b| wall_ids.contains(b.parcel.as_str())).collect();
    assert!(!on_wall.is_empty());
    for b in &on_wall {
        // `outbuilding`: the faubourg's front range beside a gate (2026-09-23).
        assert!(matches!(b.kind, "main" | "lean-to" | "outbuilding"), "{}: {} on a wall lot", b.id, b.kind);
    }
}

/// Ruling AC's tier over the whole golden matrix: exactly the one organic,
/// curtain-walled case at or above `CITADEL_MIN_POP` (10 000) gets a citadel.
/// The other six at or above it are bastioned (`bayFortGenerations` 18 000,
/// `realWaterThroughFort` 11 000, `wallStyleBastioned` 14 000,
/// `landlockedFortDry` 12 000) or radial (`venusFortCanal` 16 000,
/// `venusLandlockedCanal` 14 000), and `whole_subsystem_matches_reference`
/// holds every one of them byte-identical.
#[test]
fn only_the_largest_curtain_walled_golden_case_gets_a_citadel() {
    let got: Vec<&str> = golden::CASES
        .iter()
        .filter(|c| generate(c.seed, &opts_for(c)).citadel.is_some())
        .map(|c| c.name)
        .collect();
    assert_eq!(got, vec!["coastHarbourChain"]);
}

/// Ruling AD's perimeter blocks (`crate::courtyard`), pinned against the golden
/// set: every organic case that plats an outermost dense block takes it, and
/// the six Venus plans never do. The two organic cases without one are named,
/// so a change that silently stops reaching them fails here.
#[test]
fn the_outermost_dense_blocks_become_perimeter_blocks_on_the_organic_plan() {
    let none = [
        "landlockedHamlet",
        "unnavigableStem",
        "venusRadial",
        "venusFortCanal",
        "venusThroughBridges",
        "venusLandlockedCanal",
        "venusSmall",
        "venusTinyCanal",
    ];
    let mut rung = Vec::new();
    for c in golden::CASES {
        let t = generate(c.seed, &opts_for(c));
        let blocks: std::collections::BTreeSet<&str> = t
            .parcels
            .iter()
            .filter(|p| p.par.courtyard_ring)
            .map(|p| p.par.block.as_str())
            .collect();
        assert_eq!(blocks.is_empty(), none.contains(&c.name), "{}: {} ring blocks", c.name, blocks.len());
        if !blocks.is_empty() {
            rung.push(c.name);
        }
    }
    assert_eq!(rung.len(), 21);
}

/// The faubourg's grain (owner, 2026-09-23: *"arranged quite neatly … often
/// the poor district away from a gate and more rich people and buildings
/// closer to the gate. (this isn't 100% a rule...)"*), on every walled golden
/// town that has a faubourg and a land gate.
///
/// - Every faubourg lot carries a `gate_quality` in 0..=1, and no other lot
///   carries one.
/// - The gradient: quality falls with distance to the nearest land gate
///   (compared by thirds, and on the pooled lots), and the near third is more
///   BUILT — building area over lot area — than the far third. Not per lot:
///   the noise is meant to break it lot to lot.
/// - The noise: somewhere a lot farther from its gate outranks a nearer one.
/// - The wobble: neighbouring lots in a row behind the first no longer share
///   one back line.
#[test]
fn the_faubourg_is_ragged_and_better_off_beside_its_gate() {
    use crate::geom::poly_area;
    use crate::growth::dist_to_line;
    use crate::wallside::WallBacking;
    let (mut towns, mut near_q, mut far_q, mut near_b, mut far_b) = (0, 0.0, 0.0, 0.0, 0.0);
    let (mut pairs, mut wobbling, mut inversions) = (0usize, 0usize, 0usize);
    for c in golden::CASES {
        let t = generate(c.seed, &opts_for(c));
        for p in &t.parcels {
            assert_eq!(
                p.par.gate_quality.is_some(),
                p.par.wall_backing.is_faubourg(),
                "{}: {} carries a gate quality iff it is a faubourg lot",
                c.name,
                p.par.id
            );
            if let Some(q) = p.par.gate_quality {
                assert!((0.0..=1.0).contains(&q), "{}: {} quality {q}", c.name, p.par.id);
            }
        }
        let gates: Vec<_> = t.wall.gates.iter().filter(|g| !g.water).map(|g| g.pt).collect();
        let faub: Vec<_> = t.parcels.iter().filter(|p| p.par.wall_backing.is_faubourg()).collect();
        if gates.is_empty() || faub.len() < 9 {
            continue;
        }
        towns += 1;
        // (gate distance, quality, built share)
        let mut rows: Vec<(f64, f64, f64)> = faub
            .iter()
            .map(|p| {
                let cen = crate::geom::poly_centroid(&p.par.poly);
                let g = gates.iter().map(|q| q.dist(cen)).fold(f64::INFINITY, f64::min);
                let built: f64 = t
                    .buildings
                    .iter()
                    .filter(|b| b.parcel == p.par.id)
                    .map(|b| poly_area(&b.poly).abs())
                    .sum();
                (g, p.par.gate_quality.unwrap_or(f64::NAN), built / p.par.area)
            })
            .collect();
        rows.sort_by(|a, b| a.0.total_cmp(&b.0));
        let n = rows.len() / 3;
        let mean = |it: &[(f64, f64, f64)], f: fn(&(f64, f64, f64)) -> f64| it.iter().map(f).sum::<f64>() / it.len() as f64;
        let (near, far) = (&rows[..n], &rows[rows.len() - n..]);
        near_q += mean(near, |r| r.1);
        far_q += mean(far, |r| r.1);
        near_b += mean(near, |r| r.2);
        far_b += mean(far, |r| r.2);
        // Two lots of one cluster (within 40 m of each other along the wall
        // from the same gate side): the farther one sometimes ranks higher.
        for w in rows.windows(2) {
            if w[1].0 - w[0].0 < 5.0 && w[1].1 > w[0].1 + 0.05 {
                inversions += 1;
            }
        }
        // The wobble, on the rows behind: lots whose back-line midpoints are
        // under 10 m apart and within 3 m of each other off the wall.
        let mut arc = t.wall.land_arc.clone().expect("walled");
        if Some(&arc) == t.wall.ring.as_ref() {
            arc.push(arc[0]);
        }
        let backs: Vec<_> = faub
            .iter()
            .filter(|p| p.par.wall_backing == WallBacking::OutsideRow)
            .map(|p| {
                let m = p.par.poly[3].lerp(p.par.poly[2], 0.5);
                (m, dist_to_line(m, &arc))
            })
            .collect();
        for i in 0..backs.len() {
            for j in i + 1..backs.len() {
                let dd = (backs[i].1 - backs[j].1).abs();
                if backs[i].0.dist(backs[j].0) < 10.0 && dd < 3.0 {
                    pairs += 1;
                    if dd > 0.4 {
                        wobbling += 1;
                    }
                }
            }
        }
    }
    let k = towns as f64;
    let (near_q, far_q, near_b, far_b) = (near_q / k, far_q / k, near_b / k, far_b / k);
    println!(
        "{towns} towns: quality near/far {near_q:.3}/{far_q:.3}; built near/far {near_b:.3}/{far_b:.3}; \
         wobbling {wobbling}/{pairs}; inversions {inversions}"
    );
    assert!(towns >= 10, "the goldens hold {towns} gated faubourg towns");
    assert!(near_q > far_q + 0.25, "quality falls off the gate: {near_q} vs {far_q}");
    assert!(near_b > far_b * 1.15, "more built beside the gate: {near_b} vs {far_b}");
    assert!(inversions > 0, "not 100% a rule: a farther lot sometimes outranks a nearer one");
    assert!(wobbling * 3 > pairs, "back lines wobble: {wobbling} of {pairs} neighbouring pairs");
}
