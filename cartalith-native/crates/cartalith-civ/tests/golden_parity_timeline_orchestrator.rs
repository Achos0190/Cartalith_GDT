//! Golden-parity tests for `TIMELINE_SCOPE.md` milestone 4's orchestrator:
//! `civ_simulate_timeline` (reference `_civSimulateTimeline`, lines
//! 24875-24892).
//!
//! Generated from a Node `vm.runInContext` extraction run against
//! `reference/Cartalith Gen1 v2.10.html` (harness itself transient, not
//! checked in, per this project's convention -- see `PARITY_TESTING.md` and
//! `golden_parity_timeline_collapse.rs`'s own header), slicing the
//! milestone-1 population-ceiling chain (reference lines 23407-23434 +
//! 23461-23512, skipping the real cached `currentAgrarianDensity` body in
//! between since the harness stubs that function directly -- same technique
//! `golden_parity_timeline_collapse.rs` already documents and justifies) and
//! the whole v0.85 stepper block PLUS the orchestrator itself (24614-24892)
//! into a `vm` context stubbed with `state`/`GW`/`GH`/`field` and
//! `currentAgrarianDensity`/`currentCarryingCapacity` returning
//! caller-supplied arrays. No reference source was transcribed or
//! reimplemented by hand for the function under test; every number below is
//! the real reference's own output, read back out of the harness.
//!
//! The reference's own `_civCollapseStep`/`_civRecoveryGrowthStep` filter
//! `places` down to `p.category==='settlement'` before doing anything --
//! the harness's own place fixtures all set `category:'settlement'`
//! (otherwise every step is a silent no-op, `n=0`, which is exactly what a
//! first pass of this harness caught before that field was added).
//!
//! Same grid setup as `golden_parity_timeline_collapse.rs`: `GW=GH=100`,
//! `mapWidthKm=1000` (`cellKm=10`), `seaLevel=0.42`, uniform all-land
//! `field=0.6`. The collapse fixtures reuse `dens=10`/km² uniform (this
//! file's own `base_places` is the exact HUB/DENSE/UNDEFENDED/FORTRESS
//! fixture `golden_parity_timeline_collapse.rs` already golden-verifies at
//! the single-step level -- this file proves the ORCHESTRATOR wires
//! multiple steps together correctly, not the step functions again); the
//! recovery fixture reuses `dens=300`/km² uniform and the same
//! ruins+fortified Town `golden_parity_timeline_collapse.rs`'s own
//! `recovery_growth_step_promotes_into_exchange_tier_and_clears_ruins` test
//! uses, split across two 50-year steps instead of one 100-year step (its
//! final pop/kind, 6211/City, match that test's own single-step number
//! exactly -- confirming the logistic step is genuinely being compounded
//! step-to-step by the orchestrator, not silently re-run from scratch).
//!
//! Fixture groups (`TIMELINE_SCOPE.md` §7 success criterion 1 -- "matches
//! ... to golden-parity tolerance on every field"):
//! - collapse, `mixed` character, 3 steps -- proves `baselineNormB` is
//!   captured ONLY at step 0 and threaded unchanged into steps 1 and 2 (a
//!   bug that re-captured it every step, or never captured it at all, would
//!   diverge from these exact numbers by step 2);
//! - collapse, `trade` character, 2 steps, a different severity (0.8) --
//!   an independent second collapse configuration;
//! - recovery, 2 steps -- proves the orchestrator's `cur=r.places` chaining
//!   works for the recovery branch too, and that a tier promotion crossing
//!   an exchange-tier floor mid-run is reflected in the next step's own
//!   `kind`;
//! - collapse with `opts.steps` omitted -- `Math.max(1,opts.steps||1)`
//!   must clamp to exactly 1 step, matching
//!   `golden_parity_timeline_collapse.rs`'s own `conflict` character number
//!   exactly (proving this is really the same single-step math, run once).

use cartalith_civ::SettlementKind;
use cartalith_civ::timeline::{
    CollapseCharacter, CollapsePlace, SimulateMode, SimulateTimelineOpts, SimulateWorldParams,
    TimelineStepStats, civ_simulate_timeline,
};

const GW: usize = 100;
const GH: usize = 100;
const MAP_WIDTH_KM: f64 = 1000.0;
const SEA: f64 = 0.42;

fn uniform(d: f32) -> Vec<f32> {
    vec![d; GW * GH]
}

fn place(
    tid: u64,
    x: usize,
    y: usize,
    kind: SettlementKind,
    pop: f64,
    fortified: bool,
) -> CollapsePlace {
    CollapsePlace {
        tid,
        x,
        y,
        kind,
        pop,
        fortified,
        ruins: false,
    }
}

fn base_places() -> Vec<CollapsePlace> {
    vec![
        place(1, 10, 50, SettlementKind::Hamlet, 50.0, false),
        place(2, 30, 50, SettlementKind::Hamlet, 1000.0, false),
        place(3, 60, 50, SettlementKind::Hamlet, 50.0, false),
        place(4, 90, 50, SettlementKind::Hamlet, 50.0, true),
    ]
}

fn assert_places(
    step_label: &str,
    places: &[CollapsePlace],
    want: &[(u64, f64, SettlementKind, bool)],
) {
    assert_eq!(
        places.len(),
        want.len(),
        "{step_label}: place count, got {places:?}"
    );
    for (p, (tid, pop, kind, fortified)) in places.iter().zip(want) {
        assert_eq!(p.tid, *tid, "{step_label}: tid");
        assert_eq!(p.pop, *pop, "{step_label}: pop (tid={tid})");
        assert_eq!(p.kind, *kind, "{step_label}: kind (tid={tid})");
        assert_eq!(
            p.fortified, *fortified,
            "{step_label}: fortified (tid={tid})"
        );
    }
}

// ---------- collapse, mixed character, 3 steps -- baseline-normB threading ----------

#[test]
fn simulate_timeline_collapse_mixed_three_steps_matches_the_reference() {
    let dens = uniform(10.0);
    let field = uniform(0.6);
    let opts = SimulateTimelineOpts {
        mode: SimulateMode::Collapse,
        steps: 3,
        step_years: 10,
        character: CollapseCharacter::Mixed,
        severity: 0.5,
        k_nearest: 0,
        max_link_km: 0.0,
        rate: 0.0,
        world: SimulateWorldParams {
            dens: &dens,
            field: &field,
            gw: GW,
            gh: GH,
            sea: SEA,
            world_wrap: false,
            map_width_km: MAP_WIDTH_KM,
        },
    };
    let snaps = civ_simulate_timeline(&base_places(), &opts);
    assert_eq!(snaps.len(), 3);

    assert_places(
        "step0",
        &snaps[0].places,
        &[
            (1, 22.0, SettlementKind::Hamlet, false),
            (2, 339.0, SettlementKind::Hamlet, false),
            (4, 39.0, SettlementKind::Hamlet, true),
        ],
    );
    let TimelineStepStats::Collapse(s0) = snaps[0].stats else {
        panic!("collapse mode must produce Collapse stats")
    };
    assert_eq!(
        (s0.died, s0.migrated, s0.unplaced, s0.failed),
        (365, 0, 368, 1)
    );

    assert_places(
        "step1",
        &snaps[1].places,
        &[
            (1, 27.0, SettlementKind::Hamlet, false),
            (2, 115.0, SettlementKind::Hamlet, false),
            (4, 30.0, SettlementKind::Hamlet, true),
        ],
    );
    let TimelineStepStats::Collapse(s1) = snaps[1].stats else {
        panic!("collapse mode must produce Collapse stats")
    };
    assert_eq!(
        (s1.died, s1.migrated, s1.unplaced, s1.failed),
        (122, 17, 107, 0)
    );

    assert_places(
        "step2",
        &snaps[2].places,
        &[
            (1, 23.0, SettlementKind::Hamlet, false),
            (2, 39.0, SettlementKind::Hamlet, false),
            (4, 31.0, SettlementKind::Hamlet, true),
        ],
    );
    let TimelineStepStats::Collapse(s2) = snaps[2].stats else {
        panic!("collapse mode must produce Collapse stats")
    };
    assert_eq!(
        (s2.died, s2.migrated, s2.unplaced, s2.failed),
        (49, 21, 30, 0)
    );
}

// ---------- collapse, trade character, 2 steps, a different severity ----------

#[test]
fn simulate_timeline_collapse_trade_two_steps_matches_the_reference() {
    let dens = uniform(10.0);
    let field = uniform(0.6);
    let opts = SimulateTimelineOpts {
        mode: SimulateMode::Collapse,
        steps: 2,
        step_years: 10,
        character: CollapseCharacter::Trade,
        severity: 0.8,
        k_nearest: 0,
        max_link_km: 0.0,
        rate: 0.0,
        world: SimulateWorldParams {
            dens: &dens,
            field: &field,
            gw: GW,
            gh: GH,
            sea: SEA,
            world_wrap: false,
            map_width_km: MAP_WIDTH_KM,
        },
    };
    let snaps = civ_simulate_timeline(&base_places(), &opts);
    assert_eq!(snaps.len(), 2);

    assert_places(
        "step0",
        &snaps[0].places,
        &[
            (1, 22.0, SettlementKind::Hamlet, false),
            (2, 406.0, SettlementKind::Hamlet, false),
            (3, 20.0, SettlementKind::Hamlet, false),
            (4, 39.0, SettlementKind::Hamlet, true),
        ],
    );
    let TimelineStepStats::Collapse(s0) = snaps[0].stats else {
        panic!("collapse mode must produce Collapse stats")
    };
    assert_eq!(
        (s0.died, s0.migrated, s0.unplaced, s0.failed),
        (317, 0, 346, 0)
    );

    assert_places(
        "step1",
        &snaps[1].places,
        &[
            (1, 27.0, SettlementKind::Hamlet, false),
            (2, 165.0, SettlementKind::Hamlet, false),
            (3, 27.0, SettlementKind::Hamlet, false),
            (4, 30.0, SettlementKind::Hamlet, true),
        ],
    );
    let TimelineStepStats::Collapse(s1) = snaps[1].stats else {
        panic!("collapse mode must produce Collapse stats")
    };
    assert_eq!(
        (s1.died, s1.migrated, s1.unplaced, s1.failed),
        (131, 36, 107, 0)
    );
}

// ---------- recovery, 2 steps -- logistic growth compounded across the orchestrator's own chaining ----------

#[test]
fn simulate_timeline_recovery_two_steps_matches_the_reference() {
    let dens = uniform(300.0);
    let field = uniform(0.6);
    let start = vec![CollapsePlace {
        tid: 9,
        x: 50,
        y: 50,
        kind: SettlementKind::Town,
        pop: 300.0,
        fortified: true,
        ruins: true,
    }];
    let opts = SimulateTimelineOpts {
        mode: SimulateMode::Recovery,
        steps: 2,
        step_years: 50,
        character: CollapseCharacter::Mixed, // unread in recovery mode
        severity: 0.0,                       // unread in recovery mode
        k_nearest: 0,
        max_link_km: 0.0,
        rate: 0.05,
        world: SimulateWorldParams {
            dens: &dens,
            field: &field,
            gw: GW,
            gh: GH,
            sea: SEA,
            world_wrap: false,
            map_width_km: MAP_WIDTH_KM,
        },
    };
    let snaps = civ_simulate_timeline(&start, &opts);
    assert_eq!(snaps.len(), 2);

    // Step 0 (50 years): still Town, ruins still set (Town isn't an exchange tier).
    assert_eq!(snaps[0].places.len(), 1);
    let p0 = &snaps[0].places[0];
    assert_eq!(p0.pop, 2424.0);
    assert_eq!(p0.kind, SettlementKind::Town);
    assert!(p0.ruins, "step0: not yet promoted into an exchange tier");
    assert!(p0.fortified);
    let TimelineStepStats::Recovery(r0) = snaps[0].stats else {
        panic!("recovery mode must produce Recovery stats")
    };
    assert_eq!(r0.grew, 1);

    // Step 1 (100 years total): promoted to City (an exchange tier) -- ruins clears,
    // fortified stays. Matches golden_parity_timeline_collapse.rs's own single-100-year-step
    // number (6211/City) exactly, confirming the orchestrator's step-to-step chaining is
    // equivalent to running the same total duration in one step.
    assert_eq!(snaps[1].places.len(), 1);
    let p1 = &snaps[1].places[0];
    assert_eq!(p1.pop, 6211.0);
    assert_eq!(p1.kind, SettlementKind::City);
    assert!(
        !p1.ruins,
        "step1: promotion into an exchange tier clears ruins"
    );
    assert!(p1.fortified, "fortified is never cleared");
    let TimelineStepStats::Recovery(r1) = snaps[1].stats else {
        panic!("recovery mode must produce Recovery stats")
    };
    assert_eq!(r1.grew, 1);
}

// ---------- opts.steps omitted (0) clamps to exactly 1 step ----------

#[test]
fn simulate_timeline_clamps_zero_steps_to_one_and_matches_the_single_step_reference_number() {
    let dens = uniform(10.0);
    let field = uniform(0.6);
    let opts = SimulateTimelineOpts {
        mode: SimulateMode::Collapse,
        steps: 0, // reference: Math.max(1,opts.steps||1) -- 0 is falsy too, clamps to 1
        step_years: 10,
        character: CollapseCharacter::Conflict,
        severity: 0.5,
        k_nearest: 0,
        max_link_km: 0.0,
        rate: 0.0,
        world: SimulateWorldParams {
            dens: &dens,
            field: &field,
            gw: GW,
            gh: GH,
            sea: SEA,
            world_wrap: false,
            map_width_km: MAP_WIDTH_KM,
        },
    };
    let snaps = civ_simulate_timeline(&base_places(), &opts);
    // Exactly one step, matching golden_parity_timeline_collapse.rs's own
    // `collapse_step_character_changes_which_settlements_fail`'s `conflict` case verbatim.
    assert_eq!(snaps.len(), 1);
    assert_places(
        "step0",
        &snaps[0].places,
        &[
            (2, 111.0, SettlementKind::Hamlet, false),
            (4, 27.0, SettlementKind::Hamlet, true),
        ],
    );
    let TimelineStepStats::Collapse(s0) = snaps[0].stats else {
        panic!("collapse mode must produce Collapse stats")
    };
    assert_eq!(
        (s0.died, s0.migrated, s0.unplaced, s0.failed),
        (527, 0, 473, 2)
    );
}
