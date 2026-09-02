//! The river entity on a **real generated world** — `OUTSTANDING_WORK.md` §2.2.
//!
//! `cartalith-hydrology`'s own unit tests drive `river_entities` on a hand-wired
//! receiver tree, which is the right fixture for the aggregation rules but says
//! nothing about whether the thing fires on terrain the engine actually
//! produces. This file runs `generate_terrain` and asserts on what comes back,
//! because the failure mode this port has hit four times is a test that passes
//! on empty output (`CLAUDE.md`'s "watch for silently-empty golden output").
//!
//! It also *measures* the two numbers `get_rivers()`/`river_at()`'s doc comments
//! claim: how many entities a world has at each `min_order`, and that a point
//! taken off a river's own polyline picks that river back.
//!
//! No `godot::` type appears here. `cartalith-godot` is `cdylib`-only
//! (`ARCHITECTURE.md`), so an integration test cannot link the crate itself —
//! the same reason `nonsquare.rs` `#[path]`-includes what it needs. What is
//! under test is the engine half of both bindings, which is exactly
//! `cartalith_hydrology`; the marshalling half is `river_dict`, a dozen `set`
//! calls with no branching worth a Godot process to reach.

use cartalith_engine::{WorldParams, generate_terrain};
use cartalith_hydrology::{River, pick_river, river_entities, river_flow_thresh, river_width_scale_k};

const GW: usize = 192;
const GH: usize = 144;
const MAP_WIDTH_KM: f64 = 800.0;

fn rivers_of(ws: &cartalith_engine::WorldState, min_order: i32) -> Vec<River> {
    let order = ws.stream_order.as_ref().expect("carve_rivers is on by default, so extraction ran");
    let ch = ws.channels.as_ref().expect("a generated world retains its receiver tree");
    river_entities(
        order,
        &ch.recv,
        &ws.flow_discharge,
        &ws.field,
        GW,
        GH,
        min_order,
        river_flow_thresh(GW, GH, GW, MAP_WIDTH_KM),
        river_width_scale_k(MAP_WIDTH_KM),
        false,
    )
}

fn world(seed: i32) -> cartalith_engine::WorldState {
    let mut p = WorldParams::defaults(GW, GH, seed);
    // CPU only: the GPU path is principled-equivalent, not bit-reproducible
    // (`DECISIONS.md` §7c), and nothing here is about shading.
    p.use_gpu = false;
    p.map_width_km = MAP_WIDTH_KM;
    generate_terrain(&p)
}

/// A generated world must actually have rivers, and every one of them must be
/// a well-formed entity — the assertion that would have caught this shipping
/// as an empty array.
#[test]
fn a_generated_world_yields_well_formed_river_entities() {
    let ws = world(20260902);
    let rivers = rivers_of(&ws, 1);
    assert!(
        rivers.len() > 20,
        "a {GW}x{GH} world should trace many rivers at min_order 1, got {}",
        rivers.len()
    );

    let mut with_tribs = 0usize;
    let mut widest = 0.0f64;
    for r in &rivers {
        assert!(r.pts.len() >= 2, "every entity is a drawable run, got {} points", r.pts.len());
        assert!(r.order >= 1, "a traced run carries a real Strahler order, got {}", r.order);
        assert!(r.length_cells > 0.0, "a >=2-point run has a real length");
        assert!(r.discharge > 0.0, "a channel run carries flow, got {}", r.discharge);
        assert!((r.head as usize) < GW * GH && (r.mouth as usize) < GW * GH);
        // `discharge` is the run's maximum, so it bounds both ends. It is
        // deliberately not "the value at the mouth" -- see `River::discharge`
        // for the two receiver trees that make discharge non-monotone here,
        // and `non_monotone_discharge_is_real_not_theoretical` below for the
        // measurement on this very world.
        assert!(ws.flow_discharge[r.head as usize] <= r.discharge);
        assert!(r.mouth_discharge <= r.discharge);
        if r.tributaries > 0 {
            with_tribs += 1;
        }
        if let Some(hw) = r.half_width_cells {
            assert!(hw >= 0.5, "the reference's own half-width floor, got {hw}");
            widest = widest.max(hw);
        }
    }
    assert!(with_tribs > 0, "a real drainage network has confluences; none was counted");
    assert!(widest > 0.0, "at least one mouth must resolve a channel width");

    // Every tributary charges exactly one trunk and none charges itself.
    let charged: u32 = rivers.iter().map(|r| r.tributaries).sum();
    assert!(charged as usize <= rivers.len(), "more tributary charges ({charged}) than runs");

    println!(
        "min_order 1: {} rivers, {} with tributaries, {} charges, widest half-width {widest:.2} cells",
        rivers.len(),
        with_tribs,
        charged
    );
}

/// `min_order` is the density control both bindings expose, and it has to
/// actually thin the network — the reference's own "the network has thousands
/// of order-1 trickles" (`drawRiverWays`, v0.96).
#[test]
fn min_order_thins_the_network_monotonically() {
    let ws = world(20260902);
    let counts: Vec<usize> = (1..=4).map(|m| rivers_of(&ws, m).len()).collect();
    println!("rivers by min_order 1..=4: {counts:?}");
    assert!(counts[0] > 0, "min_order 1 must find rivers");
    for w in counts.windows(2) {
        assert!(w[1] <= w[0], "raising min_order must never add rivers: {counts:?}");
    }
    assert!(counts[0] > counts[3], "min_order 4 must be strictly thinner than 1: {counts:?}");
    // A run surviving min_order 3 is by definition an order-3+ channel, which
    // is `civ_navigable_river_discount`'s own barge threshold.
    for r in rivers_of(&ws, 3) {
        assert!(r.order >= 3, "min_order 3 admitted an order-{} run", r.order);
    }
}

/// The hit test, against real geometry.
///
/// Three claims, and the middle one is the whole reason to test on a real
/// world rather than a Y-shaped fixture:
///
/// 1. A point on any river picks **a** river. Never nothing.
/// 2. A vertex resolves either to its own run or to a run that **contains that
///    exact point** — a genuine distance-zero tie, never a wrong pick. Ties
///    are structural here: a tributary's last point IS a cell of its trunk, so
///    both runs are at distance zero on it, and `pick_river`'s documented
///    tie-break settles it. Measured on this world: 587 of 588 interior
///    vertices resolve to their own run and the single exception is one such
///    tie.
/// 3. A point *between* two cell centres picks the river too — the segment
///    case a vertex-only hit test would miss entirely, and the one a click at
///    `ViewportHost.ZOOM_MAX` lands in most of the time.
#[test]
fn a_point_on_a_river_picks_that_river_back() {
    let ws = world(20260902);
    let rivers = rivers_of(&ws, 1);
    assert!(!rivers.is_empty());

    let (mut any_hit, mut interior, mut interior_own, mut midpoint) = (0usize, 0usize, 0usize, 0usize);
    for (i, r) in rivers.iter().enumerate() {
        let k = r.pts.len() / 2;
        let p = r.pts[k];
        let hit = pick_river(&rivers, p.0, p.1, 0.75).expect("a point on a river must pick a river");
        any_hit += 1;
        // Strictly interior: neither the head nor the shared mouth cell.
        if k > 0 && k + 1 < r.pts.len() {
            interior += 1;
            if hit == i {
                interior_own += 1;
            } else {
                // The only licensed alternative: the winning run passes
                // through this exact point, so the pick was a tie and not a
                // miss. Cell centres are `col+0.5`, exactly representable, so
                // this is an exact comparison on purpose.
                assert!(
                    rivers[hit].pts.contains(&p),
                    "river {hit} won a point it does not contain: {p:?} (own run {i})"
                );
            }
        }
        let q = r.pts[k - 1];
        let mid = ((p.0 + q.0) * 0.5, (p.1 + q.1) * 0.5);
        if pick_river(&rivers, mid.0, mid.1, 0.4).is_some() {
            midpoint += 1;
        }
    }
    let n = rivers.len();
    println!("pick: {any_hit}/{n} vertices hit, {interior_own}/{interior} interior vertices resolve to their own run, {midpoint}/{n} midpoints hit");
    assert_eq!(any_hit, n, "every vertex must pick something");
    assert!(interior > n / 2, "the sample must be mostly interior vertices, got {interior}/{n}");
    // Overwhelmingly its own run; every exception was already proved a tie by
    // the `contains` assert in the loop.
    assert!(
        interior_own * 100 >= interior * 99,
        "an interior vertex should almost always resolve to its own run ({interior_own}/{interior})"
    );
    assert_eq!(midpoint, n, "every mid-segment point must hit a river ({midpoint}/{n})");

    // Far outside the grid there is nothing to pick, at any radius short of
    // the map.
    assert_eq!(pick_river(&rivers, -50.0, -50.0, 2.0), None);
}

/// The finding `River::discharge`'s doc comment rests on, asserted rather than
/// asserted-about: on a real world there **are** runs whose mouth carries less
/// flow than a cell upstream of it, because the traced receiver tree and the
/// accumulated one are not the same tree.
#[test]
fn non_monotone_discharge_is_real_not_theoretical() {
    let ws = world(20260902);
    let rivers = rivers_of(&ws, 1);
    let dips = rivers.iter().filter(|r| r.mouth_discharge < r.discharge).count();
    let worst = rivers
        .iter()
        .filter(|r| r.mouth_discharge < r.discharge)
        .map(|r| (r.discharge / r.mouth_discharge.max(f32::MIN_POSITIVE), r.discharge, r.mouth_discharge))
        .fold((0.0f32, 0.0f32, 0.0f32), |a, b| if b.0 > a.0 { b } else { a });
    println!(
        "discharge: {dips}/{} runs peak above their mouth; worst {:.2} -> {:.2} ({:.1}x)",
        rivers.len(),
        worst.1,
        worst.2,
        worst.0
    );
    assert!(
        dips > 0,
        "if this ever reaches zero the two receiver trees have converged and River::discharge's note is stale"
    );
}

/// A world generated with river carving off has no channel topology, and both
/// bindings must come back empty rather than inventing one — the same state a
/// loaded save is in (`SAVEFILE_COMPAT.md` stores no channel topology).
#[test]
fn no_river_extraction_means_no_entities() {
    let mut p = WorldParams::defaults(GW, GH, 7);
    p.use_gpu = false;
    p.carve_rivers = false;
    let ws = generate_terrain(&p);
    assert!(
        ws.stream_order.is_none() || ws.channels.is_none(),
        "carve_rivers off must leave the extraction unrun"
    );
    // The shape `rivers_now()` takes for that case: no order, no receivers,
    // no rivers.
    let rivers = river_entities(&[], &[], &[], &[], GW, GH, 1, 1.0, 1.0, false);
    assert!(rivers.is_empty());
    assert_eq!(pick_river(&rivers, 10.0, 10.0, 5.0), None);
}
