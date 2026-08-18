//! Golden-parity tests for `UNIFIED_TOOL_PLAN.md` milestone D -- the
//! Civilization tool group: `_civPaintTerritoryAt` (reference line 15964),
//! `_civDropPlace` (16051), `_civNearestOnWay`/`_civFindSnapTarget`/
//! `_civSnapPoint` (16011/16025/16043), `_civDijkstraPath` (25957) with all
//! three of its cost grids (`_civLandCostGrid` 21035, `_civWaterCostGrid`
//! 21051, `_civMixedCostGrid` 21090), the existing-way discount
//! (`_civWalkWayCells`/`_civMarkWaysOnGrid`, 21766/21757),
//! `_civJoinDijkstraSegs` (26052) and `_civCommitWay` (26072).
//!
//! # The harness
//!
//! Node `vm.runInContext`, fresh per this project's established practice
//! (not checked in). **Whole `<script>` blocks, not line slices**: blocks #1
//! (2084-14556) and #2 (14563-26720) concatenated, the same boundaries
//! `golden_parity_hierarchical_network.rs` documents. That is a materially
//! stronger boundary guarantee than milestones B/C's contiguous line
//! slices -- the delimiters are the real `<script>`/`</script>` tags, and
//! the harness asserts exactly that (the line before each slice *is*
//! `<script>`, the line after *is* `</script>`) rather than inferring a
//! top-level boundary from indentation.
//!
//! The block-comment balance assertion and Urban M2's orphan-`*/` counter
//! run on both blocks anyway, and earned their keep twice over -- both
//! times by being **wrong**, which is how a check of this kind proves it is
//! actually looking:
//!
//! 1. Two false orphans in block #2, both real comments. Cause: a crude
//!    "scan to the next quote of the same kind" string skipper
//!    desynchronises on **nested template literals**, of which this
//!    reference has many (`` `...${list.map(k => `...`)}...` ``). Fixed
//!    with a real template stack rather than by deleting the check.
//! 2. Then an unbalanced-backtick report, cause: **regex literals
//!    containing a bare `"`** (`k.replace(/"/g, '&quot;')`), read as a
//!    string opener. Fixed with a regex-literal skipper.
//!
//! Everything is driven from **inside** the context, milestone C's lesson:
//! `civWays`, `_civActiveFaction`, `_civTerRadius`, `_civWayWaypoints` and
//! `civTerritory` are all `let` declarations, which in a `vm` script are
//! lexical bindings, not properties of the context object -- assigning them
//! from the host would create a shadow the reference code never reads, and
//! the failure would be silently-empty output rather than an error.
//!
//! Six presentation-only functions (`_civRenderPlaceEditor`,
//! `_civRenderWayList`, `_civRenderJourneyList`, `_civUpdatePlannerPanel`,
//! `drawCivLayerAuto`, `renderNow`) are neutralised **inside** the context
//! by reassigning their bindings. Disclosed because it is a modification of
//! the reference environment, but note what it is *not*: no tool body is
//! transcribed or edited, and none of the six touches routing, placement or
//! paint state. `_civRenderPlaceEditor` in particular reaches
//! `_umSiteProfile`, which lives in script block #3 (the urban-morphology
//! block, 26722-28161) and is deliberately not loaded here -- loading it to
//! satisfy a UI call would pull in a whole unrelated subsystem.
//!
//! # Emptiness assertions -- because three subsystems have been bitten
//!
//! Journey Planner M5 got a silently-empty stage list from a slice that
//! parsed; milestone C got silently-empty paint output from a shadowed
//! global. Both passed every structural check. So the extraction asserted,
//! before any golden was written down: every "should route" path has >= 2
//! points and km > 0; every "should not route" path reports
//! `reachable === false` (a real negative control, not an absent
//! assertion); the territory brush painted a nonzero cell count; the drop
//! tool produced exactly one place; the unreachable commit produced a
//! non-empty warning. All of those are re-asserted here against the port.
//!
//! # The world under it is bit-identical
//!
//! Before any tool ran, the harness's `field`, water-body classification,
//! biome raster and Strahler river order were FNV-1a-64'd over their raw
//! bytes and compared against this port's own `generate_terrain` +
//! `build_water_bodies` + `build_biome_raster` + `fresh_river_order` for
//! the same parameters. All four hashes matched exactly in both cases, as
//! did the land/ocean/lake cell counts. Every routing golden below is
//! therefore a test of the tool code, not of an accidentally-different
//! world -- and both cases contain real ocean, real land and at least one
//! real lake, so the ocean-vs-lake distinction the water gates turn on is
//! genuinely exercised.
//!
//! Case 0 (`gw=24 gh=18 seed=24601 world=false`): a western landmass
//! (x 0-8), an ocean, and an eastern strip -- so a land route between two
//! western points is real, a land route to the east is genuinely
//! unreachable, and `mixed` connects them by crossing the water.
//! Case 1 (`gw=20 gh=16 seed=314159 world=true`): x-wrapping, 42 lake cells
//! and an ocean that is only connected *through the seam* -- both its land
//! and water routes come back with a seam break (`brks`), which is the
//! wrap-aware smoothing path nothing else in this file's fixtures reaches.

use cartalith_civ::tools::*;
use cartalith_civ::{NamedSettlement, SettlementKind, SettlementPlacement};

fn fnv_u8(a: &[u8]) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in a {
        h ^= b as u64;
        h = h.wrapping_mul(0x100_0000_01b3);
    }
    format!("{h:016x}")
}

struct World {
    gw: usize,
    gh: usize,
    world: bool,
    map_width_km: f64,
    sea: f64,
    field: Vec<f32>,
    wb: Vec<u8>,
    biome: Vec<u8>,
    river_order: Vec<i16>,
}

fn build(gw: usize, gh: usize, seed: i32, world: bool, field0: f64) -> World {
    let mut p = cartalith_engine::WorldParams::defaults(gw, gh, seed);
    p.world = world;
    p.climate.w_iters = 12;
    let ws = cartalith_engine::generate_terrain(&p);
    assert!((ws.sea_level - 0.42f64).abs() < 1e-12, "sea_level mismatch, harness assumption broken");
    assert_eq!(ws.field[0] as f64, field0, "field[0] mismatch, harness assumption broken");
    assert_eq!(p.map_width_km, 800.0, "map_width_km mismatch, harness assumption broken");
    let wb = cartalith_civ::build_water_bodies(&ws.field, gw, gh, ws.sea_level, world, Some(&ws.rainfall));
    let biome = cartalith_civ::build_biome_raster(&wb.classification, &ws.temperature, &ws.rainfall);
    let river_order = cartalith_civ::fresh_river_order(&ws.field, &ws.flow_discharge, gw, gh, ws.sea_level, world, p.river_density, p.map_width_km);
    World { gw, gh, world, map_width_km: p.map_width_km, sea: ws.sea_level, field: ws.field, wb: wb.classification, biome, river_order }
}

impl World {
    fn ctx<'a>(&'a self, places: &'a [NamedSettlement], ways: &'a [WayRef<'a>]) -> RouteContext<'a> {
        RouteContext {
            field: &self.field,
            water_bodies: &self.wb,
            biome: Some(&self.biome),
            river_order: Some(&self.river_order),
            places,
            ways,
            gw: self.gw,
            gh: self.gh,
            sea: self.sea,
            world: self.world,
            map_width_km: self.map_width_km,
        }
    }
}

fn town(x: usize, y: usize, kind: SettlementKind, faction: i32, pop: u32) -> NamedSettlement {
    NamedSettlement {
        placement: SettlementPlacement { x, y, suit: 0.0, faction, capital: kind == SettlementKind::Capital, kind, coastal: false },
        name: String::new(),
        pop,
    }
}

/// The reference's own `_civDijkstraPath` result, transcribed from the
/// harness dump.
fn check(label: &str, got: &DijkstraPath, pts: &[(f64, f64)], brks: &[usize], km: f64, reachable: bool) {
    assert_eq!(got.reachable, reachable, "{label}: reachable");
    assert_eq!(got.brks, brks, "{label}: brks");
    assert_eq!(got.pts, pts, "{label}: pts");
    assert_eq!(got.km.to_bits(), km.to_bits(), "{label}: km (bit-exact) got {} want {km}", got.km);
    if reachable {
        assert!(got.pts.len() >= 2 && got.km > 0.0, "{label}: emptiness assertion -- a reachable route must be a real path");
    }
}

/// The L-shaped detour way the harness builds for the existing-way
/// discount fixture: down the start column, then across to the target row.
fn detour_way(a: (usize, usize), b: (usize, usize)) -> Vec<(f64, f64)> {
    let mut pts = Vec::new();
    let step: isize = if a.1 <= b.1 { 1 } else { -1 };
    let mut y = a.1 as isize;
    loop {
        pts.push((a.0 as f64, y as f64));
        if y == b.1 as isize {
            break;
        }
        y += step;
    }
    let sx: isize = if a.0 <= b.0 { 1 } else { -1 };
    let mut x = a.0 as isize + sx;
    while x != b.0 as isize + sx {
        pts.push((x as f64, b.1 as f64));
        x += sx;
    }
    pts
}

// ===================== case 0: region, no wrap =====================

const C0_LAND_A: (usize, usize) = (2, 2);
const C0_LAND_B: (usize, usize) = (5, 15);
const C0_MID: (f64, f64) = (4.0, 9.0);
const C0_OCEAN_A: (usize, usize) = (12, 2);
const C0_OCEAN_B: (usize, usize) = (16, 10);
const C0_FAR_LAND: (usize, usize) = (22, 2);

fn case0() -> World {
    build(24, 18, 24601, false, 0.7889490723609924f64 as f32 as f64)
}

#[test]
fn case0_territory_brush_matches_civ_paint_territory_at() {
    let w = case0();
    // The reference paints faction 3 at radius 4, then faction 5 at radius
    // 2 overlapping it -- so the second stroke must genuinely overwrite,
    // which a "first writer wins" bug would fail.
    let mut layer = vec![0u8; w.gw * w.gh];
    {
        use cartalith_spatial::pass::Stamp;
        cartalith_spatial::paint::PaintStamp::ungated((w.gw / 3) as i64, (w.gh / 3) as i64, 4.0, 3).apply(&mut layer, w.gw, w.gh);
        cartalith_spatial::paint::PaintStamp::ungated((w.gw / 3 + 3) as i64, (w.gh / 3) as i64, 2.0, 5).apply(&mut layer, w.gw, w.gh);
    }
    assert_eq!(layer.iter().filter(|&&v| v == 3).count(), 39, "faction 3 cell count");
    assert_eq!(layer.iter().filter(|&&v| v == 5).count(), 13, "faction 5 cell count");
    assert_eq!(layer.iter().filter(|&&v| v != 0).count(), 52, "emptiness assertion: the brush must actually paint");
    assert_eq!(fnv_u8(&layer), "876069c36c92a8a9", "whole-raster FNV-1a-64 against the reference's own civTerritory");

    // The port's own addition (DECISIONS.md 7d): the same painted layer as
    // an override on an algorithmic base the reference never had.
    let mut base = vec![7i32; w.gw * w.gh];
    merge_territory_paint(&mut base, &layer);
    assert_eq!(base.iter().filter(|&&v| v == 7).count(), w.gw * w.gh - 52);
    assert_eq!(base.iter().filter(|&&v| v == 3).count(), 39);
}

#[test]
fn case0_drop_place_matches_civ_drop_place() {
    let w = case0();
    // `_civZoomPickR` returned 1.0 in the harness (zoom 1), so the pick
    // radius is its base value.
    let pick_r = civ_place_pick_radius(w.gw);
    assert_eq!(pick_r, 5.0);
    let mut places: Vec<NamedSettlement> = Vec::new();

    let d = civ_drop_place(&places, C0_LAND_A.0, C0_LAND_A.1, pick_r, &w.field, &w.wb, w.gw, w.gh, w.sea, 2, SettlementKind::Town, 0.0);
    let DropPlace::Placed(s) = d else { panic!("land drop must place") };
    assert_eq!((s.placement.x, s.placement.y), C0_LAND_A);
    assert_eq!(s.placement.faction, 2);
    assert_eq!(s.name, "");
    assert_eq!(s.pop, 1000);
    places.push(*s);
    assert_eq!(places.len(), 1, "emptiness assertion: the drop tool must actually append");

    // Ocean: refused, list unchanged.
    assert_eq!(
        civ_drop_place(&places, C0_OCEAN_A.0, C0_OCEAN_A.1, pick_r, &w.field, &w.wb, w.gw, w.gh, w.sea, 2, SettlementKind::Town, 0.0),
        DropPlace::Water
    );
    assert_eq!(places.len(), 1);

    // Re-clicking the same cell selects rather than stacking a second one.
    assert_eq!(
        civ_drop_place(&places, C0_LAND_A.0, C0_LAND_A.1, pick_r, &w.field, &w.wb, w.gw, w.gh, w.sea, 2, SettlementKind::Town, 0.0),
        DropPlace::Selected(0)
    );

    // A second, distant land click places a second settlement.
    let d = civ_drop_place(&places, C0_LAND_B.0, C0_LAND_B.1, pick_r, &w.field, &w.wb, w.gw, w.gh, w.sea, 2, SettlementKind::Town, 0.0);
    let DropPlace::Placed(s) = d else { panic!("second land drop must place") };
    assert_eq!((s.placement.x, s.placement.y), C0_LAND_B);
    places.push(*s);
    assert_eq!(places.len(), 2);
}

#[test]
fn case0_snapping_matches_civ_find_snap_target() {
    let w = case0();
    let places = vec![town(C0_LAND_A.0, C0_LAND_A.1, SettlementKind::Town, 1, 1000), town(C0_LAND_B.0, C0_LAND_B.1, SettlementKind::Capital, 1, 1000)];
    let pts = vec![(C0_LAND_A.0 as f64, C0_LAND_A.1 as f64), (C0_LAND_B.0 as f64, C0_LAND_B.1 as f64)];
    let ways = vec![WayRef { pts: &pts, brks: &[], sea: false, hidden: false }];
    let r = civ_snap_radius(w.gw);
    assert_eq!(r, 5.0);

    // Exactly on the pin: the place wins the tie against the way that
    // starts at the same coordinate.
    let t = civ_find_snap_target(&places, &ways, 2.0, 2.0, r).unwrap();
    assert_eq!(t.kind, SnapKind::Place(0));
    assert_eq!((t.x, t.y, t.d2), (2.0, 2.0, 0.0));

    // A cell off the pin, but beside the way: the way's projection wins.
    let t = civ_find_snap_target(&places, &ways, 3.0, 3.0, r).unwrap();
    assert_eq!(t.kind, SnapKind::Way(0));
    assert_eq!(t.x, 2.2696629213483144);
    assert_eq!(t.y, 3.168539325842697);
    assert_eq!(t.d2, 0.5617977528089891);
    assert_eq!(civ_snap_point(&places, &ways, 3.0, 3.0, r), (2.2696629213483144, 3.168539325842697));

    // Mid-way, exactly on the line.
    let t = civ_find_snap_target(&places, &ways, 3.5, 8.5, r).unwrap();
    assert_eq!(t.kind, SnapKind::Way(0));
    assert_eq!((t.x, t.y, t.d2), (3.5, 8.5, 0.0));

    // Out at the corner: only the nearer place is in range at all.
    let t = civ_find_snap_target(&places, &ways, 0.0, 0.0, r).unwrap();
    assert_eq!(t.kind, SnapKind::Place(0));
    assert_eq!(t.d2, 8.0);
    assert_eq!(civ_snap_point(&places, &ways, 0.0, 0.0, r), (2.0, 2.0));
}

#[test]
fn case0_land_route_matches_civ_dijkstra_path() {
    let w = case0();
    let ctx = w.ctx(&[], &[]);
    let got = civ_dijkstra_path(&ctx, C0_LAND_A.0 as f64, C0_LAND_A.1 as f64, C0_LAND_B.0 as f64, C0_LAND_B.1 as f64, RouteMode::Land);
    check(
        "case0 land",
        &got,
        &[(2.0, 2.0), (3.0, 5.0), (5.0, 7.0), (6.0, 10.0), (5.0, 12.0), (5.0, 15.0)],
        &[],
        479.63501408609125,
        true,
    );
}

/// The negative control, and the finding that corrected the plan: an
/// unreachable land target does **not** come back as a start-to-end
/// straight line. `_civSmoothPath` splits runs at any `|dx| > gw/2` jump --
/// unconditionally, world mode or not -- and the reconstruction's
/// start-point/target-cell pair is exactly such a jump here, so the run
/// holding the start (length 1) is dropped entirely and the drawn stub
/// sits at the *target* end. See the module notes in `UNIFIED_TOOL_PLAN.md`.
#[test]
fn case0_unreachable_land_route_is_a_stub_at_the_target_not_a_straight_line() {
    let w = case0();
    let ctx = w.ctx(&[], &[]);
    let got = civ_dijkstra_path(&ctx, C0_LAND_A.0 as f64, C0_LAND_A.1 as f64, C0_OCEAN_B.0 as f64, C0_OCEAN_B.1 as f64, RouteMode::Land);
    check("case0 land unreachable", &got, &[(16.5, 10.5), (17.0, 12.0), (16.0, 10.0)], &[], 0.0, false);
    assert!(!got.pts.contains(&(2.0, 2.0)), "the start point really is absent -- this is the reference's behaviour, not a port bug");
}

#[test]
fn case0_water_route_matches_civ_dijkstra_path() {
    let w = case0();
    let ctx = w.ctx(&[], &[]);
    let got = civ_dijkstra_path(&ctx, C0_OCEAN_A.0 as f64, C0_OCEAN_A.1 as f64, C0_OCEAN_B.0 as f64, C0_OCEAN_B.1 as f64, RouteMode::Water);
    check("case0 water", &got, &[(12.0, 2.0), (12.0, 4.0), (13.0, 7.0), (14.0, 8.0), (16.0, 10.0)], &[], 313.49727824292216, true);

    let bad = civ_dijkstra_path(&ctx, C0_OCEAN_A.0 as f64, C0_OCEAN_A.1 as f64, C0_LAND_A.0 as f64, C0_LAND_A.1 as f64, RouteMode::Water);
    check(
        "case0 water unreachable",
        &bad,
        &[(12.0, 2.0), (10.0, 2.0), (9.0, 0.0), (9.0, 0.0), (2.0, 2.0)],
        &[],
        141.20226591665966,
        false,
    );
}

#[test]
fn case0_mixed_route_crosses_the_ocean() {
    let w = case0();
    let ctx = w.ctx(&[], &[]);
    let got = civ_dijkstra_path(&ctx, C0_LAND_A.0 as f64, C0_LAND_A.1 as f64, C0_FAR_LAND.0 as f64, C0_FAR_LAND.1 as f64, RouteMode::Mixed);
    check(
        "case0 mixed",
        &got,
        &[(2.0, 2.0), (5.0, 2.0), (8.0, 2.0), (11.0, 2.0), (13.0, 2.0), (16.0, 2.0), (19.0, 2.0), (22.0, 2.0)],
        &[],
        666.6666666666667,
        true,
    );
    // Land mode refuses the same pair, which is what makes this meaningful.
    assert!(!civ_dijkstra_path(&ctx, C0_LAND_A.0 as f64, C0_LAND_A.1 as f64, C0_FAR_LAND.0 as f64, C0_FAR_LAND.1 as f64, RouteMode::Land).reachable);
}

#[test]
fn case0_settlement_gravity_bends_the_route() {
    let w = case0();
    let places = vec![town(C0_LAND_A.0, (C0_LAND_A.1 + 4).min(w.gh - 1), SettlementKind::City, 1, 5000)];
    let ctx = w.ctx(&places, &[]);
    let got = civ_dijkstra_path(&ctx, C0_LAND_A.0 as f64, C0_LAND_A.1 as f64, C0_LAND_B.0 as f64, C0_LAND_B.1 as f64, RouteMode::Land);
    check(
        "case0 gravity",
        &got,
        &[(2.0, 2.0), (3.0, 5.0), (3.0, 7.0), (4.0, 10.0), (4.0, 12.0), (5.0, 15.0)],
        &[],
        449.5610993501713,
        true,
    );
}

#[test]
fn case0_existing_way_discount_pulls_the_route_onto_it() {
    let w = case0();
    let pts = detour_way(C0_LAND_A, C0_LAND_B);
    let ways = vec![WayRef { pts: &pts, brks: &[], sea: false, hidden: false }];
    let ctx = w.ctx(&[], &ways);
    let got = civ_dijkstra_path(&ctx, C0_LAND_A.0 as f64, C0_LAND_A.1 as f64, C0_LAND_B.0 as f64, C0_LAND_B.1 as f64, RouteMode::Land);
    check(
        "case0 existing way",
        &got,
        &[(2.0, 2.0), (2.0, 5.0), (2.0, 8.0), (2.0, 10.0), (3.0, 13.0), (4.0, 14.0), (5.0, 15.0)],
        &[],
        466.356826163819,
        true,
    );
}

#[test]
fn case0_join_and_commit_match_the_reference() {
    let w = case0();
    let ctx = w.ctx(&[], &[]);
    let wps = [(C0_LAND_A.0 as f64, C0_LAND_A.1 as f64), C0_MID, (C0_LAND_B.0 as f64, C0_LAND_B.1 as f64)];
    let expect_pts = vec![(2.0, 2.0), (3.0, 4.0), (3.0, 7.0), (4.0, 9.0), (4.0, 11.0), (5.0, 13.0), (5.0, 15.0)];
    let expect_km: f64 = 456.9401310833123;

    let j = civ_join_dijkstra_segs(&ctx, &wps, RouteMode::Land);
    assert_eq!(j.pts, expect_pts, "join pts");
    assert!(j.brks.is_empty());
    assert_eq!(j.km.to_bits(), expect_km.to_bits(), "join km");
    assert_eq!(j.unreachable_legs, 0);

    let c = civ_commit_way(&ctx, &wps, ManualWayType::Road).unwrap();
    assert_eq!(c.unreachable_legs, 0);
    assert_eq!(c.way.pts, expect_pts);
    assert!(c.way.brks.is_empty());
    assert_eq!(c.way.km.to_bits(), expect_km.to_bits());
    assert!(!c.way.sea);
    assert_eq!(c.way.way_type, ManualWayType::Road);
    assert_eq!(c.way.name, "");

    // A sea lane commits over water and is tagged as such.
    let sea = civ_commit_way(
        &ctx,
        &[(C0_OCEAN_A.0 as f64, C0_OCEAN_A.1 as f64), (C0_OCEAN_B.0 as f64, C0_OCEAN_B.1 as f64)],
        ManualWayType::SeaLane,
    )
    .unwrap();
    assert!(sea.way.sea);
    assert_eq!(sea.unreachable_legs, 0);
    assert_eq!(sea.way.pts, vec![(12.0, 2.0), (12.0, 4.0), (13.0, 7.0), (14.0, 8.0), (16.0, 10.0)]);
    assert_eq!(sea.way.km.to_bits(), 313.49727824292216f64.to_bits());

    // The v1.99 warning path: the way is still produced, with a count the
    // shell turns into the reference's own alert.
    let bad = civ_commit_way(&ctx, &[(C0_LAND_A.0 as f64, C0_LAND_A.1 as f64), (C0_OCEAN_B.0 as f64, C0_OCEAN_B.1 as f64)], ManualWayType::Road).unwrap();
    assert_eq!(bad.unreachable_legs, 1, "emptiness/negative control: the warning must actually fire");
    assert_eq!(bad.way.pts, vec![(16.5, 10.5), (17.0, 12.0), (16.0, 10.0)]);
}

// ===================== case 1: world wrap =====================

const C1_LAND_A: (usize, usize) = (18, 15);
const C1_LAND_B: (usize, usize) = (1, 15);
const C1_MID: (f64, f64) = (10.0, 15.0);
const C1_OCEAN_A: (usize, usize) = (2, 2);
const C1_OCEAN_B: (usize, usize) = (18, 2);
const C1_FAR_LAND: (usize, usize) = (10, 2);

fn case1() -> World {
    build(20, 16, 314159, true, 0.3003617823123932f64 as f32 as f64)
}

#[test]
fn case1_territory_brush_matches_civ_paint_territory_at() {
    let w = case1();
    let mut layer = vec![0u8; w.gw * w.gh];
    {
        use cartalith_spatial::pass::Stamp;
        cartalith_spatial::paint::PaintStamp::ungated((w.gw / 3) as i64, (w.gh / 3) as i64, 4.0, 3).apply(&mut layer, w.gw, w.gh);
        cartalith_spatial::paint::PaintStamp::ungated((w.gw / 3 + 3) as i64, (w.gh / 3) as i64, 2.0, 5).apply(&mut layer, w.gw, w.gh);
    }
    assert_eq!(layer.iter().filter(|&&v| v != 0).count(), 52);
    assert_eq!(fnv_u8(&layer), "7502e8708d4cf429");
}

/// The wrap case: both the land route and the water route cross the x seam,
/// which is the only path through `civ_smooth_path`'s run-splitting that
/// produces a real `brks` entry. Nothing in case 0 reaches it.
#[test]
fn case1_wrapped_routes_carry_a_seam_break() {
    let w = case1();
    let ctx = w.ctx(&[], &[]);

    let land = civ_dijkstra_path(&ctx, C1_LAND_A.0 as f64, C1_LAND_A.1 as f64, C1_LAND_B.0 as f64, C1_LAND_B.1 as f64, RouteMode::Land);
    check(
        "case1 land (wrapped)",
        &land,
        &[(18.0, 15.0), (19.0, 15.0), (19.5, 15.5), (0.5, 15.5), (1.0, 15.0), (1.0, 15.0)],
        &[3],
        136.5685424949238,
        true,
    );

    let water = civ_dijkstra_path(&ctx, C1_OCEAN_A.0 as f64, C1_OCEAN_A.1 as f64, C1_OCEAN_B.0 as f64, C1_OCEAN_B.1 as f64, RouteMode::Water);
    check(
        "case1 water (wrapped)",
        &water,
        &[(2.0, 2.0), (1.0, 2.0), (0.5, 2.5), (19.5, 2.5), (19.0, 2.0), (18.0, 2.0)],
        &[3],
        176.5685424949238,
        true,
    );
}

#[test]
fn case1_land_and_water_negative_controls() {
    let w = case1();
    let ctx = w.ctx(&[], &[]);
    let land = civ_dijkstra_path(&ctx, C1_LAND_A.0 as f64, C1_LAND_A.1 as f64, C1_OCEAN_B.0 as f64, C1_OCEAN_B.1 as f64, RouteMode::Land);
    check(
        "case1 land unreachable",
        &land,
        &[(18.0, 15.0), (17.0, 11.0), (17.0, 9.0), (19.0, 6.0), (18.0, 5.0), (18.0, 2.0)],
        &[],
        610.6390435628963,
        false,
    );
    let water = civ_dijkstra_path(&ctx, C1_OCEAN_A.0 as f64, C1_OCEAN_A.1 as f64, C1_LAND_A.0 as f64, C1_LAND_A.1 as f64, RouteMode::Water);
    check("case1 water unreachable", &water, &[(18.5, 15.5), (16.0, 13.0), (18.0, 15.0)], &[], 0.0, false);
}

#[test]
fn case1_mixed_route_matches_the_reference() {
    let w = case1();
    let ctx = w.ctx(&[], &[]);
    let got = civ_dijkstra_path(&ctx, C1_LAND_A.0 as f64, C1_LAND_A.1 as f64, C1_FAR_LAND.0 as f64, C1_FAR_LAND.1 as f64, RouteMode::Mixed);
    check(
        "case1 mixed",
        &got,
        &[(18.0, 15.0), (19.0, 13.0), (19.5, 11.5), (0.5, 10.5), (2.0, 9.0), (4.0, 7.0), (6.0, 5.0), (8.0, 4.0), (10.0, 2.0)],
        &[3],
        664.3079547644414,
        true,
    );
}

#[test]
fn case1_join_and_commit_match_the_reference() {
    let w = case1();
    let ctx = w.ctx(&[], &[]);
    let wps = [(C1_LAND_A.0 as f64, C1_LAND_A.1 as f64), C1_MID, (C1_LAND_B.0 as f64, C1_LAND_B.1 as f64)];
    let expect = vec![(18.0, 15.0), (15.0, 15.0), (13.0, 15.0), (10.0, 15.0), (7.0, 15.0), (4.0, 15.0), (1.0, 15.0)];

    let j = civ_join_dijkstra_segs(&ctx, &wps, RouteMode::Land);
    assert_eq!(j.pts, expect);
    assert!(j.brks.is_empty(), "with a midpoint the two legs meet exactly and no pen is lifted");
    assert_eq!(j.km.to_bits(), 680.0f64.to_bits());
    assert_eq!(j.unreachable_legs, 0);

    let c = civ_commit_way(&ctx, &wps, ManualWayType::Road).unwrap();
    assert_eq!(c.way.pts, expect);
    assert_eq!(c.way.km.to_bits(), 680.0f64.to_bits());

    let sea = civ_commit_way(
        &ctx,
        &[(C1_OCEAN_A.0 as f64, C1_OCEAN_A.1 as f64), (C1_OCEAN_B.0 as f64, C1_OCEAN_B.1 as f64)],
        ManualWayType::SeaLane,
    )
    .unwrap();
    assert!(sea.way.sea);
    assert_eq!(sea.way.brks, vec![3], "the sea lane wraps the seam too");
    assert_eq!(sea.way.km.to_bits(), 176.5685424949238f64.to_bits());
}

/// A cross-check that the two cases are not accidentally the same fixture
/// twice -- a harness carrying a copy-paste error would not be caught by
/// any single-case assertion.
#[test]
fn the_two_cases_really_are_different_worlds() {
    let a = case0();
    let b = case1();
    assert_ne!(fnv_u8(&a.wb), fnv_u8(&b.wb));
    assert_ne!(fnv_u8(&a.biome), fnv_u8(&b.biome));
    assert!(a.wb.contains(&2) && b.wb.contains(&2), "both fixtures contain a real lake, so the ocean-vs-lake gates are exercised");
}
