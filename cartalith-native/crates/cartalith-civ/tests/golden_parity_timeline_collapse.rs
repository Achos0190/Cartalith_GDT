//! Golden-parity tests for `TIMELINE_SCOPE.md` milestone 3
//! (`cartalith_civ::timeline`): `_civSettlementStress` (reference lines
//! 24713-24723), `_civMortalityMigrationRates` (24726-24731),
//! `_civGravityMigrate` (24738-24778), `_civCollapseStep` (24785-24848) and
//! `_civRecoveryGrowthStep` (24852-24870).
//!
//! Generated from a Node `vm.runInContext` extraction run against
//! `reference/Cartalith Gen1 v2.10.html` (harness itself transient, not
//! checked in, per this project's convention -- see `PARITY_TESTING.md` and
//! `golden_parity_settlement_population.rs`'s/`golden_parity_timeline_graph.rs`'s
//! own headers), slicing the milestone-1 population-ceiling chain (lines
//! 23407-23512, verbatim) and the whole v0.85 stepper block (lines
//! 24614-24870, verbatim) into a `vm` context stubbed with `state`/`GW`/`GH`/
//! `field` and `currentAgrarianDensity`/`currentCarryingCapacity` returning
//! caller-supplied arrays -- the same "feed the pure per-cell-field
//! functions a known-good input directly" technique
//! `golden_parity_settlement_population.rs`'s own header already
//! documents and justifies. No reference source was transcribed or
//! reimplemented by hand for the functions under test; every number below
//! is the real reference's own output, read back out of the harness.
//!
//! `currentCarryingCapacity`'s return value is never actually read by any
//! fixture below -- `_civCollapseStep`/`_civRecoveryGrowthStep` only ever
//! take the branch that calls `_civSettlementPopulation`, which itself
//! only reads `currentAgrarianDensity()` (this port's own already-
//! documented dropped-dead-branch decision, `timeline.rs`'s own top-of-
//! milestone-3 doc comment) -- so the harness's stub value is an
//! unused placeholder, not a control on the results.
//!
//! All fixtures below share one grid setup unless noted otherwise:
//! `GW=GH=100`, `mapWidthKm=1000` (`cellKm=10`), `seaLevel=0.42`, uniform
//! all-land `field=0.6`, uniform `agrarianDensity=10`/km² -- chosen so
//! every settlement's catchment-population ceiling
//! (`_civSettlementPopulation`) is identical regardless of grid position
//! (uniform density -> the catchment mean is the density itself,
//! independent of catchment radius or edge clipping), and equals the exact
//! per-kind values `golden_parity_settlement_population.rs`'s own D=10
//! fixture already golden-verified (Hamlet 39, Village 137.5, Town 240,
//! City 960, Capital 1540) -- so this file leans on milestone 1's own
//! already-proven chain rather than re-deriving it, per this project's
//! "one test per pipeline stage" discipline.
//!
//! Fixture groups, per this project's own "shape fixtures to reach real
//! branches" discipline (`TIMELINE_SCOPE.md` §7 names every one of these):
//! - the abandonment floor, one cell below/at/above `_CIV_ABANDON_FLOOR=20`;
//! - a fortified-vs-unfortified pair at equal distance/headroom, proving
//!   `_CIV_FORTIFIED_BONUS` changes gravity-migration destination weighting;
//! - the gravity model's multi-pass saturation logic actually engaging
//!   (a near destination saturates, the remainder re-offered to a farther
//!   one), plus a genuine system-wide unplaced/diaspora-loss case;
//! - all four collapse characters (trade/disease/conflict/mixed) on the
//!   same base fixture, both at the raw stress-function level (with a
//!   caller-supplied baseline proving the `L` term) and end-to-end through
//!   `civ_collapse_step` (proving the character weighting changes real
//!   `failed` counts and survivor populations, not just an internal
//!   number nobody reads);
//! - a recovery step promoting a `ruins`-flagged settlement back into an
//!   exchange tier (clearing `ruins`, keeping `fortified`), contrasted with
//!   one that promotes into a NON-exchange tier and must NOT clear `ruins`.

use std::collections::HashMap;

use cartalith_civ::SettlementKind;
use cartalith_civ::timeline::{
    CollapseCharacter, CollapsePlace, civ_collapse_step, civ_gravity_migrate,
    civ_mortality_migration_rates, civ_recovery_growth_step, civ_settlement_stress,
};

const GW: usize = 100;
const GH: usize = 100;
const MAP_WIDTH_KM: f64 = 1000.0;
const SEA: f64 = 0.42;

fn uniform_field(d10: f32) -> Vec<f32> {
    vec![d10; GW * GH]
}

fn place(
    tid: u64,
    x: usize,
    y: usize,
    kind: SettlementKind,
    pop: f64,
    fortified: bool,
    ruins: bool,
) -> CollapsePlace {
    CollapsePlace {
        tid,
        x,
        y,
        kind,
        pop,
        fortified,
        ruins,
    }
}

fn assert_close(actual: f64, expected: f64, label: &str) {
    const ATOL: f64 = 1e-6;
    const RTOL: f64 = 1e-6;
    let tol = ATOL + RTOL * expected.abs();
    assert!(
        (actual - expected).abs() <= tol,
        "{label}: got {actual}, expected {expected} (diff {}, tol {tol})",
        (actual - expected).abs()
    );
}

// ---------- _civMortalityMigrationRates ----------

#[test]
fn mortality_migration_rates_match_the_reference_across_characters_and_clamps() {
    let cases: [(f64, f64, CollapseCharacter, f64, f64); 6] = [
        (0.5, 0.5, CollapseCharacter::Mixed, 0.0375, 0.0625),
        (1.0, 1.0, CollapseCharacter::Mixed, 0.15, 0.25),
        (0.5, 0.5, CollapseCharacter::Disease, 0.0375, 0.0375),
        (0.5, 0.5, CollapseCharacter::Conflict, 0.0375, 0.0875),
        (0.0, 0.5, CollapseCharacter::Trade, 0.0, 0.0),
        // severity > 1 is a real reachable value (the UI's severity dial is
        // 0-100%, but nothing stops a caller passing more) -- both rates
        // still clamp at 0.95, not the ceiling constant times an unclamped
        // severity*stress product.
        (1.0, 2.0, CollapseCharacter::Mixed, 0.3, 0.5),
    ];
    for (stress, severity, character, want_m, want_g) in cases {
        let r = civ_mortality_migration_rates(stress, severity, character);
        assert_close(
            r.m,
            want_m,
            &format!("m({stress},{severity},{character:?})"),
        );
        assert_close(
            r.g,
            want_g,
            &format!("g({stress},{severity},{character:?})"),
        );
    }
}

// ---------- _civSettlementStress: character weighting, with a real L term ----------

/// Four settlements, same graph (so `normBNow` is identical for everyone
/// under every character), differing only in pop (`D`'s population-rank
/// half) and fortification (`V`), plus a caller-supplied `baselineNormB`
/// that gives ONLY the first settlement (`HUB`, tid=1) a real trade-
/// dependency loss (`L`): its baseline (1.0) is far above its current
/// `normB` (0.0), while every other settlement's baseline exactly equals
/// its own current `normB` (`L=0` for them). This is a synthetic baseline
/// chosen to exercise the `L` term directly (not derived from an actual
/// prior simulated step, which is milestone 4's orchestrator's job to wire
/// up automatically) -- `_civSettlementStress`'s real signature takes an
/// arbitrary caller-supplied map, so this is a legitimate, disclosed test
/// of that signature, not a hidden deviation from the reference.
///
/// `normBNow=[0,0,1,0]` and the pop/fortification profile below are the
/// REAL output of `_civProximityAdjacency`/`_civBetweennessFromAdjacency`
/// run against this fixture's own positions through the harness (a
/// 4-node graph where the third settlement is the sole bridge between the
/// other three, per `maxLinkKm=cellKm*GW*0.5=500km` excluding the two
/// longest pairs) -- not hand-derived, read back from the harness the same
/// as everything else in this file.
#[test]
fn settlement_stress_character_weighting_matches_the_reference() {
    let hub = place(1, 10, 50, SettlementKind::Hamlet, 50.0, false, false);
    let dense = place(2, 30, 50, SettlementKind::Hamlet, 1000.0, false, false);
    let undefended = place(3, 60, 50, SettlementKind::Hamlet, 50.0, false, false);
    let fortress = place(4, 90, 50, SettlementKind::Hamlet, 50.0, true, false);
    let places = [hub, dense, undefended, fortress];
    let norm_b_now = [0.0, 0.0, 1.0, 0.0];
    let max_pop_now = 1000.0;
    let baseline: HashMap<u64, f64> = [(1, 1.0), (2, 0.0), (3, 1.0), (4, 0.0)]
        .into_iter()
        .collect();

    let cases: [(CollapseCharacter, [f64; 4]); 4] = [
        (
            CollapseCharacter::Trade,
            [0.9512499999999999, 0.275, 0.27625, 0.07625],
        ),
        (
            CollapseCharacter::Disease,
            [0.3175, 0.6, 0.6174999999999999, 0.0925],
        ),
        (
            CollapseCharacter::Conflict,
            [0.95125, 0.8250000000000001, 0.82625, 0.24125],
        ),
        (CollapseCharacter::Mixed, [0.75625, 0.525, 0.53125, 0.12625]),
    ];
    for (character, want) in cases {
        for (i, p) in places.iter().enumerate() {
            let stress =
                civ_settlement_stress(p, norm_b_now[i], Some(&baseline), max_pop_now, character);
            assert_close(stress, want[i], &format!("{character:?} stress[{i}]"));
        }
    }
    // The point of the fixture: trade/conflict/mixed all rank HUB (index 0)
    // as most-stressed (its lost centrality dominates via wL, or its lack
    // of fortification dominates via wV); disease inverts this -- HUB drops
    // to second-LOWEST, and the two genuinely dense/connected settlements
    // (DENSE by population, UNDEFENDED by centrality) become the most
    // stressed instead. This is the design doc's own "disease is the
    // opposite direction from trade-collapse" claim, proven in real numbers.
    let disease = [0.3175, 0.6, 0.6174999999999999, 0.0925];
    let trade = [0.9512499999999999, 0.275, 0.27625, 0.07625];
    assert!(
        trade[0] > trade[1] && trade[0] > trade[2],
        "trade must rank HUB above DENSE/UNDEFENDED"
    );
    assert!(
        disease[0] < disease[1] && disease[0] < disease[2],
        "disease must rank HUB below DENSE/UNDEFENDED -- the inversion"
    );
}

// ---------- _civGravityMigrate ----------

/// Two destinations at EQUAL distance (400km) and equal headroom (500) from
/// one origin -- the only difference is A's explicit `fortified` trait.
/// A single saturation pass (neither destination's headroom binds) is
/// enough to isolate `_CIV_FORTIFIED_BONUS`'s effect cleanly: A receives
/// exactly `1.5x` what B receives (`bonusFactor` 1.5 vs 1.0), not merely
/// "more".
#[test]
fn gravity_migrate_fortified_bonus_changes_destination_weighting() {
    let origin = place(1, 10, 50, SettlementKind::Town, 0.0, false, false);
    let fortified_dest = place(2, 50, 50, SettlementKind::Town, 0.0, true, false);
    let unfortified_dest = place(3, 10, 90, SettlementKind::Town, 0.0, false, false);
    let places = [origin, fortified_dest, unfortified_dest];
    let migrants = [300.0, 0.0, 0.0];
    let cap_field = [0.0, 500.0, 500.0];

    let r = civ_gravity_migrate(&places, |i| migrants[i], &cap_field, 10.0, GW as f64, false);
    assert_close(r.received[1], 180.0, "fortified destination received");
    assert_close(r.received[2], 120.0, "unfortified destination received");
    assert_close(r.unplaced, 0.0, "unplaced (single pass, no saturation)");
    assert_close(
        r.received[1] / r.received[2],
        1.5,
        "exactly the fortified bonus ratio",
    );
}

/// A close destination (headroom 50) and a far destination (headroom
/// 2000): a single proportional pass would over-allocate to the close one
/// (its raw distance-weighted share exceeds 50), so the algorithm MUST
/// cap it and re-offer the clipped remainder to the far one on a later
/// pass -- exercising the up-to-4-pass saturation loop, not just the
/// single-pass common case the fortified-bonus fixture above covers.
#[test]
fn gravity_migrate_saturates_the_near_destination_and_reoffers_the_remainder() {
    let origin = place(1, 10, 50, SettlementKind::Town, 0.0, false, false);
    let near = place(2, 20, 50, SettlementKind::Town, 0.0, false, false);
    let far = place(3, 90, 50, SettlementKind::Town, 0.0, false, false);
    let places = [origin, near, far];
    let migrants = [1000.0, 0.0, 0.0];
    let cap_field = [0.0, 50.0, 2000.0];

    let r = civ_gravity_migrate(&places, |i| migrants[i], &cap_field, 10.0, GW as f64, false);
    assert_close(
        r.received[1],
        50.0,
        "near destination saturates at its headroom",
    );
    assert_close(
        r.received[2],
        950.0,
        "far destination absorbs the reoffered remainder",
    );
    assert_close(r.unplaced, 0.0, "combined headroom covers every migrant");
}

/// Same near/far shape, but now BOTH destinations' combined headroom (150)
/// is less than the migrant pool (1000) -- proves the system-wide
/// unplaced/diaspora-loss statistic actually accumulates what remaining
/// headroom cannot absorb, not just "some number came out of the loop".
#[test]
fn gravity_migrate_reports_unplaced_diaspora_loss_when_headroom_is_exhausted() {
    let origin = place(1, 10, 50, SettlementKind::Town, 0.0, false, false);
    let near = place(2, 20, 50, SettlementKind::Town, 0.0, false, false);
    let far = place(3, 90, 50, SettlementKind::Town, 0.0, false, false);
    let places = [origin, near, far];
    let migrants = [1000.0, 0.0, 0.0];
    let cap_field = [0.0, 50.0, 100.0];

    let r = civ_gravity_migrate(&places, |i| migrants[i], &cap_field, 10.0, GW as f64, false);
    assert_close(r.received[1], 50.0, "near destination fully saturated");
    assert_close(r.received[2], 100.0, "far destination fully saturated");
    assert_close(
        r.unplaced,
        850.0,
        "everything neither could absorb is unplaced",
    );
}

// ---------- _civCollapseStep: abandonment floor ----------

/// A single isolated settlement (no other settlement to migrate to, so
/// every migrant becomes unplaced diaspora loss and `stayers` alone
/// determines the new population) at three starting populations chosen so
/// the post-mortality/migration population lands at exactly one cell
/// below, at, and one cell above `CIV_ABANDON_FLOOR=20`. `mixed` character,
/// `severity=0.5`, `stepYears=1` (no compounding, so the survival
/// multiplier is a single fixed constant across all three cases -- the
/// only thing that varies is the starting population).
#[test]
fn collapse_step_abandonment_floor_boundary_matches_the_reference() {
    let dens = uniform_field(10.0);
    let field = uniform_field(0.6);

    // pop0=21 -> newPop=19 (one below the floor): abandoned.
    let below = [place(1, 50, 50, SettlementKind::Hamlet, 21.0, false, false)];
    let r_below = civ_collapse_step(
        &below,
        CollapseCharacter::Mixed,
        0.5,
        1,
        0,
        0.0,
        None,
        &dens,
        &field,
        GW,
        GH,
        SEA,
        false,
        MAP_WIDTH_KM,
    );
    assert!(r_below.places.is_empty(), "pop0=21 must be abandoned");
    assert_eq!(r_below.stats.failed, 1);
    assert_eq!(r_below.stats.died, 1);
    assert_eq!(r_below.stats.unplaced, 1);

    // pop0=22 -> newPop=20 (exactly at the floor): survives (the check is
    // strictly `<`, not `<=`).
    let at = [place(1, 50, 50, SettlementKind::Hamlet, 22.0, false, false)];
    let r_at = civ_collapse_step(
        &at,
        CollapseCharacter::Mixed,
        0.5,
        1,
        0,
        0.0,
        None,
        &dens,
        &field,
        GW,
        GH,
        SEA,
        false,
        MAP_WIDTH_KM,
    );
    assert_eq!(r_at.places.len(), 1, "pop0=22 -> newPop=20 must survive");
    assert_eq!(r_at.places[0].pop, 20.0);
    assert_eq!(r_at.stats.failed, 0);

    // pop0=23 -> newPop=21 (one above the floor): survives.
    let above = [place(1, 50, 50, SettlementKind::Hamlet, 23.0, false, false)];
    let r_above = civ_collapse_step(
        &above,
        CollapseCharacter::Mixed,
        0.5,
        1,
        0,
        0.0,
        None,
        &dens,
        &field,
        GW,
        GH,
        SEA,
        false,
        MAP_WIDTH_KM,
    );
    assert_eq!(r_above.places.len(), 1, "pop0=23 -> newPop=21 must survive");
    assert_eq!(r_above.places[0].pop, 21.0);
    assert_eq!(r_above.stats.failed, 0);
}

// ---------- _civCollapseStep: all four characters change the fail order ----------

/// The same HUB/DENSE/UNDEFENDED/FORTRESS base fixture as the raw-stress
/// test above, run end-to-end through `civ_collapse_step` at `severity=0.5`,
/// `stepYears=10`, WITHOUT a baseline (`t=0` -- `L=0` for everyone, so this
/// isolates `D`/`V`'s contribution specifically, the realistic case for a
/// simulation's very first step). Proves the character weight triple
/// changes real, observable output -- which settlements fail, and their
/// exact surviving populations -- not just an internal stress number.
#[test]
fn collapse_step_character_changes_which_settlements_fail() {
    let dens = uniform_field(10.0);
    let field = uniform_field(0.6);
    let base = || {
        [
            place(1, 10, 50, SettlementKind::Hamlet, 50.0, false, false),
            place(2, 30, 50, SettlementKind::Hamlet, 1000.0, false, false),
            place(3, 60, 50, SettlementKind::Hamlet, 50.0, false, false),
            place(4, 90, 50, SettlementKind::Hamlet, 50.0, true, false),
        ]
    };
    let run = |character: CollapseCharacter| {
        civ_collapse_step(
            &base(),
            character,
            0.5,
            10,
            0,
            0.0,
            None,
            &dens,
            &field,
            GW,
            GH,
            SEA,
            false,
            MAP_WIDTH_KM,
        )
    };

    // trade: nobody fails -- L=0 for everyone at t=0, and trade weights L
    // heaviest (0.70), so with no loss to measure the whole system is
    // comparatively stable this step.
    let trade = run(CollapseCharacter::Trade);
    assert_eq!(trade.stats.failed, 0, "trade: nobody fails at t=0");
    assert_eq!(trade.stats.died, 209);
    assert_eq!(trade.stats.unplaced, 267);
    let trade_tids: Vec<u64> = trade.places.iter().map(|p| p.tid).collect();
    assert_eq!(trade_tids, vec![1, 2, 3, 4]);

    // disease: only UNDEFENDED (tid=3, the graph's sole high-centrality
    // bridge -- disease weights D=connectivity heaviest, 0.70) fails.
    let disease = run(CollapseCharacter::Disease);
    assert_eq!(disease.stats.failed, 1);
    assert_eq!(disease.stats.died, 400);
    assert_eq!(disease.stats.unplaced, 255);
    let disease_tids: Vec<u64> = disease.places.iter().map(|p| p.tid).collect();
    assert_eq!(disease_tids, vec![1, 2, 4], "UNDEFENDED (tid=3) is gone");

    // conflict: HUB (tid=1, unfortified) AND UNDEFENDED (tid=3, unfortified)
    // both fail; DENSE and FORTRESS (tid=4, fortified) survive -- conflict
    // weights V=undefended-violence heaviest (0.80), and fortification is
    // what saves FORTRESS despite sharing HUB's low pop/centrality profile.
    let conflict = run(CollapseCharacter::Conflict);
    assert_eq!(conflict.stats.failed, 2);
    assert_eq!(conflict.stats.died, 527);
    assert_eq!(conflict.stats.unplaced, 473);
    let conflict_tids: Vec<u64> = conflict.places.iter().map(|p| p.tid).collect();
    assert_eq!(conflict_tids, vec![2, 4], "only DENSE and FORTRESS survive");

    // mixed: only UNDEFENDED fails (same survivor set as disease here, but
    // different exact numbers -- the blended weights are not a simple
    // average of the other three outcomes).
    let mixed = run(CollapseCharacter::Mixed);
    assert_eq!(mixed.stats.failed, 1);
    assert_eq!(mixed.stats.died, 365);
    assert_eq!(mixed.stats.unplaced, 368);
    let mixed_tids: Vec<u64> = mixed.places.iter().map(|p| p.tid).collect();
    assert_eq!(mixed_tids, vec![1, 2, 4], "UNDEFENDED (tid=3) is gone");

    // Exact surviving populations, per character (also read back from the
    // harness, not hand-derived).
    assert_eq!(
        trade.places.iter().map(|p| p.pop).collect::<Vec<_>>(),
        vec![30.0, 572.0, 29.0, 43.0]
    );
    assert_eq!(
        disease.places.iter().map(|p| p.pop).collect::<Vec<_>>(),
        vec![33.0, 398.0, 44.0]
    );
    assert_eq!(
        conflict.places.iter().map(|p| p.pop).collect::<Vec<_>>(),
        vec![111.0, 27.0]
    );
    assert_eq!(
        mixed.places.iter().map(|p| p.pop).collect::<Vec<_>>(),
        vec![22.0, 339.0, 39.0]
    );
}

// ---------- _civRecoveryGrowthStep: ruins clearing on exchange-tier promotion ----------

/// A `ruins`+`fortified` Town (simulating a former City that collapsed
/// down to Town, still fortified in its own ruins) with a high enough local
/// agrarian density (300/km², vs 10 elsewhere in this file) that its OWN
/// Town-kind catchment ceiling (`24*300=7200`) clears the City floor
/// (`5000`) -- so 100 years of 5%/yr logistic regrowth from a low starting
/// population (300) crosses all the way into City tier in one step.
/// Promotion into an EXCHANGE tier (city/capital) is what clears `ruins`;
/// `fortified` is never cleared, even on promotion.
#[test]
fn recovery_growth_step_promotes_into_exchange_tier_and_clears_ruins() {
    let dens = uniform_field(300.0);
    let field = uniform_field(0.6);
    let places = [place(9, 50, 50, SettlementKind::Town, 300.0, true, true)];

    let r = civ_recovery_growth_step(
        &places,
        0.05,
        100,
        &dens,
        &field,
        GW,
        GH,
        SEA,
        false,
        MAP_WIDTH_KM,
    );
    assert_eq!(r.stats.grew, 1);
    let p = &r.places[0];
    assert_eq!(p.pop, 6211.0);
    assert_eq!(p.kind, SettlementKind::City);
    assert!(!p.ruins, "promotion into an exchange tier must clear ruins");
    assert!(p.fortified, "fortified is never cleared, even on promotion");
}

/// Same shape, lower density (100/km², so Village's own catchment ceiling
/// -- `13.75*100=1375` -- clears the Town floor (800) but NOT the City
/// floor): promotes Village -> Town over 100 years, but Town is NOT an
/// exchange tier, so `ruins` must stay set.
#[test]
fn recovery_growth_step_promotion_into_a_non_exchange_tier_keeps_ruins() {
    let dens = uniform_field(100.0);
    let field = uniform_field(0.6);
    let places = [place(9, 50, 50, SettlementKind::Village, 100.0, true, true)];

    let r = civ_recovery_growth_step(
        &places,
        0.05,
        100,
        &dens,
        &field,
        GW,
        GH,
        SEA,
        false,
        MAP_WIDTH_KM,
    );
    let p = &r.places[0];
    assert_eq!(p.pop, 1266.0);
    assert_eq!(p.kind, SettlementKind::Town);
    assert!(
        p.ruins,
        "promotion into a NON-exchange tier must not clear ruins"
    );
    assert!(p.fortified);
}
