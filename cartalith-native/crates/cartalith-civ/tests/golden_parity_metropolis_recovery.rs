//! Golden-parity tests for the two v0.75/v0.82 passes ported 2026-08-20 on
//! the owner's decision: `_civSelectMetropolises` (reference lines
//! **24961-24989**) and `_civApplyRecovery` (reference lines
//! **24619-24640**), plus the now-uncapped `_civTierForPopulation`
//! (line 24618) over the full six-entry `_CIV_TIER_FLOOR` (line 24617).
//!
//! Generated from a Node `vm.runInContext` extraction run against
//! `reference/Cartalith Gen1 v2.10.html` (harness itself transient, not
//! checked in, per this project's convention -- see `PARITY_TESTING.md` and
//! `golden_parity_timeline_collapse.rs`'s own header). Three slices, taken
//! **verbatim**: `mulberry32` (line 2291), the tier tables +
//! `_civTierForPopulation` + `_civApplyRecovery` (24614-24640), and
//! `_civSelectMetropolises` (24961-24989). Every number below is the real
//! reference's own output, read back out of the harness; nothing here was
//! transcribed or reimplemented by hand.
//!
//! **The line ranges were asserted, not assumed** -- this repo's own
//! hard-learned rule. The harness `must()`-checks a distinctive substring on
//! each boundary line before slicing, and that check earned its keep
//! immediately: `_civSelectMetropolises` ends on **24989**, not the 24988 a
//! first reading of the function index suggested. A slice ending one line
//! early would have produced a syntax error here, but the same class of
//! error at the *start* of a slice silently omits a definition. The harness
//! also probes that all four sliced functions are `typeof 'function'` and
//! that `_CIV_TIER_ORDER` really has six entries before emitting anything,
//! per the "watch for silently-empty golden output" rule.
//!
//! ## Why the RNG stream is checked, not just the outputs
//!
//! `_civApplyRecovery` draws from `rng` **once per settlement, before** the
//! abandonment test that may drop that settlement. Get that wrong and every
//! result still looks plausible while every later consumer of the same
//! stream silently desynchronises. So each recovery fixture also pins the
//! **next** value the stream yields after the call, against an independently
//! extracted table of raw `mulberry32` draws (`DRAWS_*` below): the fixture
//! that drops one of six settlements must leave the stream at draw 7, and
//! the phase-0 no-op must leave it at draw 1.

use cartalith_civ::timeline::CollapsePlace;
use cartalith_civ::{
    MetropolisOpts, SettlementKind, SettlementPlacement, civ_select_metropolises,
    timeline::{RecoveryOpts, RecoveryPhase, civ_apply_recovery, civ_tier_for_population},
};
use cartalith_rng::Mulberry32;

// ===================== `_civTierForPopulation`, uncapped =====================

/// The reference's own `_civTierForPopulation` answers over the full
/// six-entry floor table, sampled just below and exactly at every boundary.
/// The last three rows are the ones that used to read `"capital"` in this
/// port while the reference said `"metropolis"` -- the documented cap that
/// `TIMELINE_SCOPE.md` §9 recorded, and that porting
/// `_civSelectMetropolises` lifted.
#[test]
fn tier_for_population_matches_the_full_six_tier_reference_table() {
    // Protects: `civ_tier_for_population` over the reference's full six-entry
    // `_CIV_TIER_FLOOR` (lines 24617-24618): floors 150 / 800 / 5 000 / 30
    // 000 / 150 000 are inclusive lower bounds, each probed just below and
    // exactly at. A mutation of any floor, or a return to the old capital
    // cap, moves a row; the three top rows are the ones the cap used to fail.
    let cases: [(f64, SettlementKind); 13] = [
        (0.0, SettlementKind::Hamlet),
        (149.999, SettlementKind::Hamlet),
        (150.0, SettlementKind::Village),
        (799.999, SettlementKind::Village),
        (800.0, SettlementKind::Town),
        (4999.999, SettlementKind::Town),
        (5000.0, SettlementKind::City),
        (29999.999, SettlementKind::City),
        (30000.0, SettlementKind::Capital),
        (149_999.999, SettlementKind::Capital),
        (150_000.0, SettlementKind::Metropolis),
        (2_000_000.0, SettlementKind::Metropolis),
        (5_000_000.0, SettlementKind::Metropolis),
    ];
    for (pop, want) in cases {
        assert_eq!(civ_tier_for_population(pop), want, "pop={pop}");
    }
}

// ===================== `_civSelectMetropolises` =====================

/// Builds a `SettlementPlacement` at (x, y) of the given kind and faction,
/// with `capital` derived from the kind, `suit` 0.0 and `coastal` false.
/// Why: `civ_select_metropolises` reads only kind, faction and position, so
/// every other field is held at an inert value.
/// Must never: set `capital` independently of `kind`; the reference tests the
/// kind string.
fn p(x: usize, y: usize, kind: SettlementKind, faction: i32) -> SettlementPlacement {
    SettlementPlacement {
        x,
        y,
        suit: 0.0,
        faction,
        capital: kind == SettlementKind::Capital,
        kind,
        coastal: false,
    }
}

/// A faction of `n` towns at `y=20`, so its capital clears `minFactionSize`.
fn towns(x0: usize, y: usize, n: usize, faction: i32) -> Vec<SettlementPlacement> {
    (0..n)
        .map(|i| p(x0 + i, y, SettlementKind::Town, faction))
        .collect()
}

/// Fixture 1: one faction of six, one capital, dominant betweenness.
#[test]
fn a_dominant_capital_of_a_large_polity_is_promoted() {
    // Protects: the positive path of `civ_select_metropolises` (reference
    // 24961-24989): a capital in a six-settlement faction with the maximum
    // betweenness is promoted. If this goes red the negative fixtures below
    // pass for the wrong reason.
    let mut places = vec![p(10, 10, SettlementKind::Capital, 1)];
    places.extend(towns(20, 10, 5, 1));
    let btw = [100.0, 5.0, 4.0, 3.0, 2.0, 1.0];
    assert_eq!(
        civ_select_metropolises(&places, &btw, 100.0, MetropolisOpts::default()),
        vec![0]
    );
}

/// Fixture 2: the same capital, one settlement short of `minFactionSize=6`.
/// The polity-size term is what makes this rule about administrative
/// capacity rather than pure centrality -- so it gets its own fixture, one
/// settlement below the boundary.
#[test]
fn a_faction_one_settlement_below_min_size_promotes_nothing() {
    // Protects: the default `min_faction_size` of 6: the identical fixture
    // with five settlements promotes nothing, so a mutation of the default to
    // 5 or lower turns this red.
    let mut places = vec![p(10, 10, SettlementKind::Capital, 1)];
    places.extend(towns(20, 10, 4, 1));
    let btw = [100.0, 5.0, 4.0, 3.0, 2.0];
    assert_eq!(
        civ_select_metropolises(&places, &btw, 100.0, MetropolisOpts::default()),
        Vec::<usize>::new()
    );
}

/// Fixture 3: `normB >= btwThr` is inclusive at exactly 0.85. Quantised on
/// purpose (85/100), so the boundary is representable and the comparison's
/// direction is genuinely under test rather than hidden in float noise.
#[test]
fn the_betweenness_threshold_is_inclusive_at_exactly_0_85() {
    // Protects: the default `btw_thr` of 0.85 and the direction of its
    // comparison: 84.9/100 is below it, 85/100 clears it (`>=`). A threshold
    // mutation or a `>` comparison turns one of the two asserts red.
    let mut places = vec![p(10, 10, SettlementKind::Capital, 1)];
    places.extend(towns(20, 10, 5, 1));
    let below = [84.9, 1.0, 1.0, 1.0, 1.0, 1.0];
    let at = [85.0, 1.0, 1.0, 1.0, 1.0, 1.0];
    assert_eq!(
        civ_select_metropolises(&places, &below, 100.0, MetropolisOpts::default()),
        Vec::<usize>::new(),
        "84.9/100 = 0.849 is below the threshold"
    );
    assert_eq!(
        civ_select_metropolises(&places, &at, 100.0, MetropolisOpts::default()),
        vec![0],
        "85/100 = 0.85 exactly clears it"
    );
}

/// Fixture 4: the reference's own `maxBtwF<=0` early return.
#[test]
fn a_non_positive_max_betweenness_promotes_nothing() {
    // Protects: the reference's `maxBtwF <= 0` early return. With every
    // betweenness 0 and a maximum of 0 the normalised ratio would divide by
    // zero; the port must return an empty list instead.
    let mut places = vec![p(10, 10, SettlementKind::Capital, 1)];
    places.extend(towns(20, 10, 5, 1));
    let btw = [0.0; 6];
    assert_eq!(
        civ_select_metropolises(&places, &btw, 0.0, MetropolisOpts::default()),
        Vec::<usize>::new()
    );
    assert!(civ_select_metropolises(&places, &btw, 0.0, MetropolisOpts::default()).is_empty());
}

/// Fixture 5: two capitals in one faction, `perFaction=1` -- the *higher*
/// betweenness one wins, and it is deliberately NOT the earlier index, so a
/// port that silently kept input order instead of sorting would fail here.
#[test]
fn per_faction_cap_keeps_the_more_central_of_two_capitals() {
    // Protects: the `per_faction` cap of 1 together with the betweenness
    // sort: the second capital (index 1, betweenness 100) must beat the first
    // (index 0, 90). A port that kept input order returns `[0]` and fails.
    let mut places = vec![
        p(10, 10, SettlementKind::Capital, 1),
        p(40, 10, SettlementKind::Capital, 1),
    ];
    places.extend(towns(20, 10, 5, 1));
    let btw = [90.0, 100.0, 1.0, 1.0, 1.0, 1.0, 1.0];
    assert_eq!(
        civ_select_metropolises(&places, &btw, 100.0, MetropolisOpts::default()),
        vec![1]
    );
}

/// Fixture 6: four eligible factions, `globalCap=3` -- the fourth is
/// dropped, and the three kept are the three most central.
#[test]
fn the_global_cap_truncates_a_fourth_eligible_faction() {
    // Protects: the default `global_cap` of 3: of four eligible factions the
    // least central is dropped and the three kept appear in descending
    // betweenness. A mutation of the cap changes the length of the result.
    let mut places = Vec::new();
    let mut btw = Vec::new();
    for f in 1..=4i32 {
        places.push(p(f as usize * 10, 5, SettlementKind::Capital, f));
        btw.push(100.0 - f64::from(f));
        for t in towns(f as usize * 10, 20, 5, f) {
            places.push(t);
            btw.push(1.0);
        }
    }
    assert_eq!(
        civ_select_metropolises(&places, &btw, 100.0, MetropolisOpts::default()),
        vec![0, 6, 12]
    );
}

/// Fixture 7: three capitals at *identical* normalised betweenness, so only
/// the `x`-then-`y` tie-break comparator decides the order -- and with
/// `globalCap=2`, decides who is dropped outright. Expected order is
/// `(10,3)` then `(10,9)` then `(30,5)`: x ascending first, y ascending to
/// break the remaining tie.
#[test]
fn the_tie_break_is_x_then_y_ascending() {
    // Protects: the comparator's tie-break on equal normalised betweenness: x
    // ascending, then y ascending. The first assert pins the whole order and
    // the second pins that the cap of 2 drops the last entry of that order. A
    // swapped or missing tie-break reorders the result.
    let mut places = vec![
        p(30, 5, SettlementKind::Capital, 1),
        p(10, 9, SettlementKind::Capital, 2),
        p(10, 3, SettlementKind::Capital, 3),
    ];
    let mut btw = vec![100.0, 100.0, 100.0];
    for f in 1..=3i32 {
        for t in towns(50 + f as usize * 5, 40, 5, f) {
            places.push(t);
            btw.push(0.0);
        }
    }
    assert_eq!(
        civ_select_metropolises(&places, &btw, 100.0, MetropolisOpts::default()),
        vec![2, 1, 0],
        "selection order, not merely membership"
    );
    assert_eq!(
        civ_select_metropolises(
            &places,
            &btw,
            100.0,
            MetropolisOpts {
                global_cap: 2,
                ..MetropolisOpts::default()
            }
        ),
        vec![2, 1],
        "the cap drops the tie-break's LAST entry"
    );
}

/// Fixture 8: every `opts` field simultaneously off its default, so a port
/// that hardcoded any one of the four defaults fails.
#[test]
fn every_opts_field_is_honoured_at_once() {
    // Protects: that `btw_thr`, `min_faction_size`, `per_faction` and
    // `global_cap` are all read from `MetropolisOpts`: every field is off its
    // default at once, so a port that hardcoded any one default changes the
    // answer. The closing assert shows the same input promotes nothing under
    // the defaults.
    let places = vec![
        p(10, 10, SettlementKind::Capital, 1),
        p(40, 10, SettlementKind::Capital, 1),
        p(70, 10, SettlementKind::Capital, 2),
        p(20, 30, SettlementKind::Town, 1),
        p(21, 30, SettlementKind::Town, 1),
        p(22, 30, SettlementKind::Town, 1),
        p(60, 30, SettlementKind::Town, 2),
        p(61, 30, SettlementKind::Town, 2),
        p(62, 30, SettlementKind::Town, 2),
    ];
    let btw = [100.0, 95.0, 90.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0];
    let opts = MetropolisOpts {
        btw_thr: 0.5,
        min_faction_size: 3,
        per_faction: 2,
        global_cap: 5,
    };
    assert_eq!(
        civ_select_metropolises(&places, &btw, 100.0, opts),
        vec![0, 1, 2]
    );
    // …and with the defaults restored, the very same input promotes nothing:
    // faction 2 has only 4 settlements, faction 1's second capital exceeds
    // perFaction=1, and 95/100 & 90/100 both clear btwThr only because the
    // 0.5 override lowered it -- but 100/100 does not, because faction 1 is
    // still below minFactionSize=6.
    assert_eq!(
        civ_select_metropolises(&places, &btw, 100.0, MetropolisOpts::default()),
        Vec::<usize>::new()
    );
}

/// Fixture 9: `kind!=='capital'` is checked before anything else -- a city
/// with maximal betweenness in a large polity is still never promoted.
/// Metropolis is a promotion *of a capital*, not of the most central place.
#[test]
fn a_city_is_never_eligible_however_central() {
    // Protects: the reference's `kind !== 'capital'` check: a city with the
    // maximum betweenness in a six-settlement faction is never promoted. Red
    // means metropolis promotion has stopped being a promotion of a capital.
    let mut places = vec![p(10, 10, SettlementKind::City, 1)];
    places.extend(towns(20, 10, 5, 1));
    let btw = [100.0, 1.0, 1.0, 1.0, 1.0, 1.0];
    assert_eq!(
        civ_select_metropolises(&places, &btw, 100.0, MetropolisOpts::default()),
        Vec::<usize>::new()
    );
}

/// The normalisation `_civNetworkMetrics` applies (`btw /= (n-1)(n-2)`,
/// reference line 21990) cancels in `betweenness/maxBtw`, which is the only
/// way either value is read. This pins that reasoning rather than leaving it
/// as a comment: the same fixture, normalised and un-normalised, must agree.
#[test]
fn betweenness_normalisation_cancels_out() {
    // Protects: the claim that `_civNetworkMetrics`' `(n-1)(n-2)`
    // normalisation (reference line 21990) cancels in `betweenness / maxBtw`:
    // the same fixture raw and normalised must select identically. Red means
    // the selection reads an absolute betweenness somewhere.
    let mut places = vec![
        p(10, 10, SettlementKind::Capital, 1),
        p(40, 10, SettlementKind::Capital, 1),
    ];
    places.extend(towns(20, 10, 5, 1));
    let raw = [90.0, 100.0, 1.0, 1.0, 1.0, 1.0, 1.0];
    let n = places.len() as f64;
    let norm_div = (n - 1.0) * (n - 2.0);
    let normalised: Vec<f64> = raw.iter().map(|b| b / norm_div).collect();
    assert_eq!(
        civ_select_metropolises(&places, &raw, 100.0, MetropolisOpts::default()),
        civ_select_metropolises(
            &places,
            &normalised,
            100.0 / norm_div,
            MetropolisOpts::default()
        )
    );
}

// ===================== `_civApplyRecovery` =====================

/// Raw `mulberry32(seed)` draws, extracted from the reference's own line
/// 2291 by the same harness -- used only to prove how many values each
/// recovery call consumed (see this file's own header).
const DRAWS_1234: [f64; 2] = [0.073_294_978_123_158_22, 0.703_411_989_845_335_5];
const DRAWS_7: [f64; 7] = [
    0.011_704_753_153_026_104,
    0.061_958_257_574_588_06,
    0.976_907_632_779_33,
    0.699_028_705_712_407_8,
    0.521_445_268_532_261_3,
    0.405_521_688_051_521_8,
    0.466_232_632_519_677_3,
];
const DRAWS_99: [f64; 7] = [
    0.260_465_812_403_708_7,
    0.804_822_765_523_567_8,
    0.540_871_534_962_207_1,
    0.690_243_425_779_044_6,
    0.001_138_708_321_377_635,
    0.817_543_087_759_986_5,
    0.011_350_846_383_720_636,
];
const DRAWS_5: [f64; 3] = [
    0.689_774_910_919_368_3,
    0.772_743_273_293_599_5,
    0.219_763_010_274_618_86,
];
const DRAWS_3: [f64; 3] = [
    0.720_226_783_771_067_9,
    0.038_662_160_513_922_57,
    0.456_192_192_621_529_1,
];
const DRAWS_11: [f64; 3] = [
    0.511_587_048_647_925_3,
    0.529_946_408_234_536_6,
    0.608_118_564_123_287_8,
];

/// Builds a `CollapsePlace` of the given kind, population and port flag, with
/// `tid` 0, position (0, 0), `fortified` and `ruins` false.
/// Why: `civ_apply_recovery` reads kind, pop, port and the two flags, never
/// position, so position is inert here.
/// Must never: be given non-default `fortified`/`ruins` by default; the
/// idempotence test overrides them explicitly with struct-update syntax.
fn cp(kind: SettlementKind, pop: f64, port: bool) -> CollapsePlace {
    CollapsePlace {
        tid: 0,
        x: 0,
        y: 0,
        kind,
        pop,
        fortified: false,
        ruins: false,
        port,
    }
}

/// `(kind, pop, ruins, fortified)` -- the four fields the reference's own
/// pass writes.
type Row = (SettlementKind, f64, bool, bool);

/// Projects a roster to `(kind, pop, ruins, fortified)` tuples, the four
/// fields `_civApplyRecovery` writes.
/// Why: lets a fixture assert the whole output roster, including its order
/// and length, in one `assert_eq!`.
/// Must never: compare position or port; the pass does not write them.
fn rows(places: &[CollapsePlace]) -> Vec<Row> {
    places
        .iter()
        .map(|p| (p.kind, p.pop, p.ruins, p.fortified))
        .collect()
}

/// The reference's six-tier roster, one per tier, with the village a port.
fn roster() -> Vec<CollapsePlace> {
    vec![
        cp(SettlementKind::Metropolis, 400_000.0, false),
        cp(SettlementKind::Capital, 60_000.0, false),
        cp(SettlementKind::City, 12_000.0, false),
        cp(SettlementKind::Town, 3_000.0, false),
        cp(SettlementKind::Village, 400.0, true),
        cp(SettlementKind::Hamlet, 40.0, false),
    ]
}

/// Phase 0 is the reference's `band==null` no-op: the roster comes back
/// untouched **and the RNG is not advanced at all** -- which is exactly what
/// makes "Recovery phase: Stable" byte-identical to not running the pass.
#[test]
fn phase_stable_is_a_no_op_that_draws_nothing() {
    // Protects: the reference's `band == null` no-op for phase 0: the roster
    // is returned unchanged and the RNG is not advanced (the next draw must
    // be `DRAWS_1234[0]`), so a Stable timeline stays byte-identical to not
    // running the pass.
    let places = vec![
        cp(SettlementKind::Capital, 60_000.0, false),
        cp(SettlementKind::Town, 3_000.0, false),
        cp(SettlementKind::Hamlet, 40.0, false),
    ];
    let mut rng = Mulberry32::new(1234);
    let out = civ_apply_recovery(
        &places,
        RecoveryPhase::Stable,
        &mut rng,
        RecoveryOpts::default(),
    );
    assert_eq!(rows(&out), rows(&places));
    assert_eq!(
        rng.next_f64(),
        DRAWS_1234[0],
        "the stream must be untouched"
    );
}

/// The full roster through phase I (Survival, band 0.04-0.10). The hamlet is
/// abandoned -- unanchored and below `dropThresh=18` -- and every surviving
/// urban nucleus demotes into its own ruins. The `nextDraw` assertion is the
/// load-bearing one: **six** draws were consumed for six inputs, even though
/// only five came back.
#[test]
fn phase_survival_collapses_and_prunes_the_roster() {
    // Protects: `civ_apply_recovery` at phase I (band 0.04 to 0.10; reference
    // 24619-24640) on the six-tier roster with seed 7: scaled populations,
    // tier demotions into ruins, and the abandonment of the unanchored
    // hamlet. The final assert pins that six draws were consumed for six
    // inputs although five came back.
    let mut rng = Mulberry32::new(7);
    let out = civ_apply_recovery(
        &roster(),
        RecoveryPhase::Survival,
        &mut rng,
        RecoveryOpts::default(),
    );
    assert_eq!(
        rows(&out),
        vec![
            (SettlementKind::City, 16281.0, true, true),
            (SettlementKind::Town, 2623.0, true, true),
            (SettlementKind::Town, 1183.0, true, true),
            (SettlementKind::Village, 246.0, true, true),
            (SettlementKind::Hamlet, 29.0, false, false),
        ]
    );
    assert_eq!(
        rng.next_f64(),
        DRAWS_7[6],
        "one draw per INPUT settlement, including the one that was dropped"
    );
}

/// Phase II (Subsistence, 0.10-0.30) on the same roster and the same seed --
/// the prune still applies (`phase<=2`), and the hamlet still fails it.
#[test]
fn phase_subsistence_on_the_same_roster_and_seed() {
    // Protects: phase II (band 0.10 to 0.30) on the same roster and seed: the
    // band's scaling and the `phase <= 2` abandonment prune, which still
    // removes the hamlet. Red on the last assert means the draw count depends
    // on the phase.
    let mut rng = Mulberry32::new(7);
    let out = civ_apply_recovery(
        &roster(),
        RecoveryPhase::Subsistence,
        &mut rng,
        RecoveryOpts::default(),
    );
    assert_eq!(
        rows(&out),
        vec![
            (SettlementKind::Capital, 40936.0, true, true),
            (SettlementKind::City, 6743.0, true, true),
            (SettlementKind::Town, 3545.0, true, true),
            (SettlementKind::Village, 719.0, true, true),
            (SettlementKind::Hamlet, 82.0, false, false),
        ]
    );
    assert_eq!(rng.next_f64(), DRAWS_7[6]);
}

/// Phase III (Regional, 0.30-0.70): the prune is off (`phase>2`), so the
/// hamlet survives at 18 -- and the city/town/village all keep their tier,
/// so `ruins`/`fortified` stay clear on exactly those three. That split is
/// what proves `demoted` gates the ruins flag rather than `was_urban` alone.
#[test]
fn phase_regional_keeps_every_settlement_and_only_ruins_the_demoted() {
    // Protects: phase III (band 0.30 to 0.70): the prune is off above phase
    // 2, so the hamlet survives at 18, and only the demoted settlements get
    // `ruins`/`fortified`. The split between demoted and kept-tier rows shows
    // that `demoted`, not `was_urban` alone, gates the flags.
    let mut rng = Mulberry32::new(7);
    let out = civ_apply_recovery(
        &roster(),
        RecoveryPhase::Regional,
        &mut rng,
        RecoveryOpts::default(),
    );
    assert_eq!(
        rows(&out),
        vec![
            (SettlementKind::Capital, 121_873.0, true, true),
            (SettlementKind::City, 19487.0, true, true),
            (SettlementKind::City, 8289.0, false, false),
            (SettlementKind::Town, 1739.0, false, false),
            (SettlementKind::Village, 203.0, false, false),
            (SettlementKind::Hamlet, 18.0, false, false),
        ]
    );
    assert_eq!(rng.next_f64(), DRAWS_7[6]);
}

/// Phase IV (Mature, 0.70-1.00): nothing demotes at all -- the metropolis
/// stays a metropolis. Without the uncapped tier table this row would read
/// `Capital` and silently pass a hand-written expectation.
#[test]
fn phase_mature_demotes_nothing_and_keeps_the_metropolis() {
    // Protects: phase IV (band 0.70 to 1.00): nothing demotes and no flag is
    // set, and the metropolis stays a metropolis. A return to the old
    // capital-capped tier table would show `Capital` in row 0.
    let mut rng = Mulberry32::new(7);
    let out = civ_apply_recovery(
        &roster(),
        RecoveryPhase::Mature,
        &mut rng,
        RecoveryOpts::default(),
    );
    assert_eq!(
        rows(&out),
        vec![
            (SettlementKind::Metropolis, 281_405.0, false, false),
            (SettlementKind::Capital, 43115.0, false, false),
            (SettlementKind::City, 11917.0, false, false),
            (SettlementKind::Town, 2729.0, false, false),
            (SettlementKind::Village, 343.0, false, false),
            (SettlementKind::Hamlet, 33.0, false, false),
        ]
    );
    assert_eq!(rng.next_f64(), DRAWS_7[6]);
}

/// Six unanchored hamlets straddling `dropThresh=18`: the first is abandoned
/// (its scaled population rounds to 8, below 18), the rest survive -- one of
/// them at exactly 18, pinning the `pop < dropThresh` comparison as strict.
#[test]
fn unanchored_hamlets_below_the_drop_threshold_are_abandoned() {
    // Protects: the default `drop_thresh` of 18 and the strictness of `pop <
    // dropThresh`: one hamlet scaled to 8 is dropped and the one scaled to
    // exactly 18 survives. A `<=` comparison drops the 18 and changes the row
    // count.
    let places: Vec<CollapsePlace> = (1..=6)
        .map(|i| cp(SettlementKind::Hamlet, f64::from(i) * 100.0, false))
        .collect();
    let mut rng = Mulberry32::new(99);
    let out = civ_apply_recovery(
        &places,
        RecoveryPhase::Survival,
        &mut rng,
        RecoveryOpts::default(),
    );
    assert_eq!(
        rows(&out),
        vec![
            (SettlementKind::Hamlet, 18.0, false, false),
            (SettlementKind::Hamlet, 22.0, false, false),
            (SettlementKind::Hamlet, 33.0, false, false),
            (SettlementKind::Hamlet, 20.0, false, false),
            (SettlementKind::Hamlet, 53.0, false, false),
        ],
        "the 18 survivor is the strictness proof: `pop < dropThresh`, not `<=`"
    );
    assert_eq!(rng.next_f64(), DRAWS_99[6]);
}

/// The identical fixture with `port` set on every entry: "survivors cluster
/// on water" (reference line 24631), so the one that was abandoned above is
/// now anchored and survives at the `max(8, pop)` floor.
#[test]
fn a_port_anchors_a_hamlet_the_prune_would_otherwise_take() {
    // Protects: the port anchor (reference line 24631, survivors cluster on
    // water): the same six hamlets as the previous test, all ports, are all
    // kept, and the one that was abandoned survives at the `max(8, pop)`
    // floor.
    let places: Vec<CollapsePlace> = (1..=6)
        .map(|i| cp(SettlementKind::Hamlet, f64::from(i) * 100.0, true))
        .collect();
    let mut rng = Mulberry32::new(99);
    let out = civ_apply_recovery(
        &places,
        RecoveryPhase::Survival,
        &mut rng,
        RecoveryOpts::default(),
    );
    assert_eq!(
        rows(&out),
        vec![
            (SettlementKind::Hamlet, 8.0, false, false),
            (SettlementKind::Hamlet, 18.0, false, false),
            (SettlementKind::Hamlet, 22.0, false, false),
            (SettlementKind::Hamlet, 33.0, false, false),
            (SettlementKind::Hamlet, 20.0, false, false),
            (SettlementKind::Hamlet, 53.0, false, false),
        ]
    );
    assert_eq!(rng.next_f64(), DRAWS_99[6]);
}

/// `was_urban` is `town|city|capital|metropolis` -- **wider** than
/// `civ_is_exchange_tier`'s `city|capital|metropolis`, which is the easy
/// thing to get wrong. A town and a village of identical population, drawn
/// consecutively from one stream: the town anchors, the village does not.
#[test]
fn town_counts_as_urban_for_anchoring_but_village_does_not() {
    // Protects: the width of `was_urban` (`town|city|capital|metropolis`),
    // which is wider than `civ_is_exchange_tier`'s `city|capital|metropolis`:
    // a town below the threshold survives as an urban nucleus while an
    // identical village is dropped. The last assert pins that both entries
    // drew.
    let places = vec![
        cp(SettlementKind::Town, 150.0, false),
        cp(SettlementKind::Village, 150.0, false),
    ];
    let mut rng = Mulberry32::new(5);
    let out = civ_apply_recovery(
        &places,
        RecoveryPhase::Survival,
        &mut rng,
        RecoveryOpts::default(),
    );
    assert_eq!(
        rows(&out),
        vec![(SettlementKind::Hamlet, 12.0, true, true)],
        "the town survives at 12 -- below dropThresh, but anchored by being urban"
    );
    assert_eq!(
        rng.next_f64(),
        DRAWS_5[2],
        "both entries drew, including the dropped village"
    );
}

/// `opts.dropThresh` really is the knob: the same seed and roster at 40
/// instead of 18 leaves only the largest hamlet standing.
#[test]
fn a_custom_drop_threshold_is_honoured() {
    // Protects: `RecoveryOpts::drop_thresh` being read at all: with 40
    // instead of 18 only the largest hamlet (53) survives. A hardcoded 18
    // keeps more rows.
    let places: Vec<CollapsePlace> = (1..=6)
        .map(|i| cp(SettlementKind::Hamlet, f64::from(i) * 100.0, false))
        .collect();
    let mut rng = Mulberry32::new(99);
    let out = civ_apply_recovery(
        &places,
        RecoveryPhase::Survival,
        &mut rng,
        RecoveryOpts { drop_thresh: 40.0 },
    );
    assert_eq!(
        rows(&out),
        vec![(SettlementKind::Hamlet, 53.0, false, false)]
    );
    assert_eq!(rng.next_f64(), DRAWS_99[6]);
}

/// …and it is ignored outright above phase II, however large -- the
/// reference's `(phase<=2) ? ... : 0` gate, tested with a threshold that
/// would otherwise abandon every settlement in the world.
#[test]
fn the_drop_threshold_is_ignored_above_phase_two() {
    // Protects: the `(phase <= 2) ? dropThresh : 0` gate: a threshold of 1e9
    // (a labelled sentinel meaning abandon everything) must still keep all
    // three hamlets at phase III, each at the floor of 8.
    let places = vec![
        cp(SettlementKind::Hamlet, 10.0, false),
        cp(SettlementKind::Hamlet, 12.0, false),
        cp(SettlementKind::Hamlet, 14.0, false),
    ];
    let mut rng = Mulberry32::new(99);
    let out = civ_apply_recovery(
        &places,
        RecoveryPhase::Regional,
        &mut rng,
        RecoveryOpts { drop_thresh: 1e9 },
    );
    assert_eq!(
        rows(&out),
        vec![
            (SettlementKind::Hamlet, 8.0, false, false),
            (SettlementKind::Hamlet, 8.0, false, false),
            (SettlementKind::Hamlet, 8.0, false, false),
        ]
    );
    assert_eq!(rng.next_f64(), DRAWS_99[3]);
}

/// The `max(8, pop)` floor, and the ordering detail behind it: the tier
/// decision reads the **unfloored** rounded population. Both towns here
/// scale to below 8 and land on the floor, and both are classified `Hamlet`
/// from the pre-floor value.
#[test]
fn the_population_floor_is_eight_and_applies_after_the_tier_decision() {
    // Protects: the `max(8, pop)` floor and its order of operations: both
    // towns scale below 8, are classified `Hamlet` from the unfloored rounded
    // population, and only then raised to 8. Applying the floor first would
    // classify them differently.
    let places = vec![
        cp(SettlementKind::Town, 60.0, true),
        cp(SettlementKind::Town, 20.0, true),
    ];
    let mut rng = Mulberry32::new(3);
    let out = civ_apply_recovery(
        &places,
        RecoveryPhase::Survival,
        &mut rng,
        RecoveryOpts::default(),
    );
    assert_eq!(
        rows(&out),
        vec![
            (SettlementKind::Hamlet, 8.0, true, true),
            (SettlementKind::Hamlet, 8.0, true, true),
        ]
    );
    assert_eq!(rng.next_f64(), DRAWS_3[2]);
}

/// A metropolis is a real input tier to this pass, not just an output one:
/// two of them collapse into cities inside their own ruins.
#[test]
fn a_metropolis_demotes_into_its_ruins() {
    // Protects: that `Metropolis` is a valid input tier: two metropolises
    // collapse to cities with `ruins` and `fortified` set. Red means the
    // input tier table was capped again.
    let places = vec![
        cp(SettlementKind::Metropolis, 200_000.0, false),
        cp(SettlementKind::Metropolis, 300_000.0, false),
    ];
    let mut rng = Mulberry32::new(11);
    let out = civ_apply_recovery(
        &places,
        RecoveryPhase::Survival,
        &mut rng,
        RecoveryOpts::default(),
    );
    assert_eq!(
        rows(&out),
        vec![
            (SettlementKind::City, 14139.0, true, true),
            (SettlementKind::City, 21539.0, true, true),
        ]
    );
    assert_eq!(rng.next_f64(), DRAWS_11[2]);
}

/// An already-`fortified`, already-`ruins` metropolis: the reference guards
/// the trait push with `!p.traits.includes('fortified')`, which in this
/// port's boolean representation is simply idempotent. Pinned so the
/// representation change stays honest.
#[test]
fn setting_ruins_and_fortified_is_idempotent() {
    // Protects: the boolean form of the reference's
    // `!p.traits.includes('fortified')` guard: a place that is already ruined
    // and fortified comes out with both flags true and the same result as the
    // fresh one.
    let places = vec![CollapsePlace {
        fortified: true,
        ruins: true,
        ..cp(SettlementKind::Metropolis, 200_000.0, false)
    }];
    let mut rng = Mulberry32::new(11);
    let out = civ_apply_recovery(
        &places,
        RecoveryPhase::Survival,
        &mut rng,
        RecoveryOpts::default(),
    );
    assert_eq!(
        rows(&out),
        vec![(SettlementKind::City, 14139.0, true, true)]
    );
    assert_eq!(rng.next_f64(), DRAWS_11[1]);
}

/// An empty roster returns an empty roster and draws nothing, at every
/// phase -- the reference's `!places.length` early return.
#[test]
fn an_empty_roster_is_returned_empty_at_every_phase() {
    // Protects: the reference's `!places.length` early return at all five
    // phases: an empty roster returns empty and the stream is untouched (the
    // next draw is `DRAWS_1234[0]`).
    for phase in [
        RecoveryPhase::Stable,
        RecoveryPhase::Survival,
        RecoveryPhase::Subsistence,
        RecoveryPhase::Regional,
        RecoveryPhase::Mature,
    ] {
        let mut rng = Mulberry32::new(1234);
        assert!(civ_apply_recovery(&[], phase, &mut rng, RecoveryOpts::default()).is_empty());
        assert_eq!(
            rng.next_f64(),
            DRAWS_1234[0],
            "phase {phase:?} drew from the stream"
        );
    }
}

/// One row of [`recovery_phase_indices_and_bands_match_the_reference_tables`]:
/// variant, the reference's numeric phase, `_CIV_RECOVERY_FRAC[phase]`, and
/// `_CIV_RECOVERY_NAME[phase]`.
type PhaseRow = (RecoveryPhase, u8, Option<(f64, f64)>, &'static str);

#[test]
fn recovery_phase_indices_and_bands_match_the_reference_tables() {
    // Protects: `RecoveryPhase`'s index, band and name against the
    // reference's `_CIV_RECOVERY_FRAC` and `_CIV_RECOVERY_NAME` tables, and
    // `from_index_clamped`'s clamping at both ends (-7 gives Stable, 99 gives
    // Mature). An off-by-one index would move the `phase <= 2` prune onto the
    // wrong phases.
    let table: [PhaseRow; 5] = [
        (RecoveryPhase::Stable, 0, None, "Stable"),
        (
            RecoveryPhase::Survival,
            1,
            Some((0.04, 0.10)),
            "I · Survival",
        ),
        (
            RecoveryPhase::Subsistence,
            2,
            Some((0.10, 0.30)),
            "II · Subsistence",
        ),
        (
            RecoveryPhase::Regional,
            3,
            Some((0.30, 0.70)),
            "III · Regional",
        ),
        (RecoveryPhase::Mature, 4, Some((0.70, 1.00)), "IV · Mature"),
    ];
    for (phase, idx, band, name) in table {
        assert_eq!(phase.index(), idx);
        assert_eq!(phase.frac_band(), band);
        assert_eq!(phase.name(), name);
        assert_eq!(RecoveryPhase::from_index_clamped(i64::from(idx)), phase);
    }
    assert_eq!(RecoveryPhase::from_index_clamped(-7), RecoveryPhase::Stable);
    assert_eq!(RecoveryPhase::from_index_clamped(99), RecoveryPhase::Mature);
}

/// One draw per settlement, exactly -- checked directly rather than only
/// through the fixtures above, because it is the property every one of them
/// depends on.
#[test]
fn exactly_one_draw_is_consumed_per_input_settlement() {
    // Protects: the draw-count invariant every fixture depends on: for 0 to 5
    // inputs the stream advances by exactly one draw per input, checked
    // against a reference stream. A draw taken after the abandonment test
    // desynchronises every later consumer.
    for n in 0..6usize {
        let places: Vec<CollapsePlace> = (0..n)
            .map(|_| cp(SettlementKind::Capital, 60_000.0, false))
            .collect();
        let mut rng = Mulberry32::new(7);
        let _ = civ_apply_recovery(
            &places,
            RecoveryPhase::Mature,
            &mut rng,
            RecoveryOpts::default(),
        );
        let mut reference = Mulberry32::new(7);
        for _ in 0..n {
            reference.next_f64();
        }
        assert_eq!(rng.next_f64(), reference.next_f64(), "n={n}");
    }
}
