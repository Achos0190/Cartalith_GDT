//! Golden-parity tests for `GUI_GAP_REGISTER.md` **CV-25**'s three ported
//! functions:
//!
//! - `_umWallSpec` (reference **22109-22132**) — the four-rung
//!   fortification ladder.
//! - `_umInferWalls` (**22134-22136**) — its boolean view, which
//!   `_civFactionAggregates`' `fortifiedCount` reads.
//! - `_civPlaceDefensibility` (**23802-23810**) — per-settlement defensive
//!   strength `0..1`.
//!
//! plus the two primitives they call, `_civTerrainRuggednessD` (**6318**)
//! and `_umInferAge` (**22096-22099**), and the `CIV_SETTLEMENT_CLASSES`
//! rank table (**14674-14685**).
//!
//! # The harness
//!
//! Node `vm.runInContext` over six line **slices**, run at extraction time
//! and not checked in — the established practice for functions with no
//! block-level `let` dependencies (`golden_parity_roster.rs`'s precedent).
//! `_umWallSpec` reads exactly three globals beyond its own argument
//! (`CIV_SETTLEMENT_CLASSES`, `field`, `state`) and `GW`/`GH`; all are
//! supplied as context properties, which is safe here because none of the
//! six slices declares any of them with `let` at file scope.
//!
//! **Two boundary assertions failed on the first attempt, and both were
//! real.** The `_umWallSpec` slice started at 22105, which is inside the
//! v1.17 provenance comment rather than at the `function` line (22109) —
//! `CLAUDE.md`'s "verify the line ranges against the real reference before
//! slicing" rule earning its keep again. And the four-rung assertion was
//! written as four `return 'x';` statements, which the reference does not
//! contain: `palisade` reaches its caller only through the two ternaries
//! `pop>=1200?'stone':'palisade'` and `rank>=1?'palisade':'ditch'`. Both
//! failed loudly rather than silently producing a short, plausible golden.
//!
//! # Shape assertions
//!
//! The extraction refused to emit a golden unless all four rungs appeared
//! across the cases, and unless `defensibility` and `walled` were each
//! genuinely differentiated (some zero, some not; some true, some false) —
//! the silently-empty-golden failure mode `CLAUDE.md` records four
//! subsystems being bitten by. All three are re-asserted here.
//!
//! # The fixture is shaped to reach the code
//!
//! 6x4, `sea = 0.42`, `field[i] = 0.42 + 0.58*(i/23)` so the relative
//! elevation `r` sweeps the whole `[0,1]` land band. Three cells are
//! overridden:
//!
//! - `field[8]` puts `r` at the ruggedness peak `0.35`, the only place the
//!   commanding-village rung can fire;
//! - `field[9]` aims at `terrainD == 0.9` exactly — and lands a hair
//!   *above* it once stored as `f32`, which is why the golden says `ditch`.
//!   That is not a slack case: it is the one that pins `field` being a
//!   `Float32Array` on both sides — an `f64` field puts the same cell just
//!   *below* the threshold and the answer becomes `none`. (The `>` in
//!   `terrainD>0.9` is provably unobservable at any precision; the crate's
//!   own `commanding_village_digs_in` records why and pins the constant
//!   from both sides instead.)
//! - `field[0]` sits below sea, so a negative `r` is exercised.
//!
//! Population and age cases sit just below **and** at each threshold
//! (`1199`/`1200`, `259`/`260`, `249`/`250`) so a mutated constant cannot
//! survive.

use cartalith_civ::SettlementKind;
use cartalith_civ::military::{
    WallPlace, civ_place_defensibility, civ_relative_elevation, um_wall_spec, um_infer_walls,
};
use cartalith_civ::urban_adapter::um_infer_age;

const GW: usize = 6;
const GH: usize = 4;
const SEA: f64 = 0.42;

/// The extraction's own fixture, rebuilt identically here. `f32` storage is
/// load-bearing — see this file's header.
fn fixture() -> Vec<f32> {
    let mut field = vec![0f32; GW * GH];
    for (i, v) in field.iter_mut().enumerate() {
        *v = (0.42 + 0.58 * (i as f64 / 23.0)) as f32;
    }
    field[8] = (0.42 + 0.58 * 0.35) as f32;
    field[9] = (0.42 + 0.58 * (0.35 + 0.025)) as f32;
    field[0] = 0.10;
    field
}

/// `(x, y)` of the cell at linear index `i`, as the extraction's `at()`.
fn at(i: usize) -> (f64, f64) {
    ((i % GW) as f64, (i / GW) as f64)
}

fn place(
    field: &[f32],
    cell: usize,
    kind: SettlementKind,
    pop: f64,
) -> WallPlace<'static> {
    let (x, y) = at(cell);
    WallPlace {
        walls_override: None,
        kind,
        pop,
        fortified_trait: false,
        age_override: None,
        specialisation: None,
        relative_elevation: civ_relative_elevation(field, GW, GH, SEA, x, y),
    }
}

/// One extracted row: label, expected spec, expected walled, expected
/// defensibility. Compared exactly — every value here is a `Math.max`/
/// `Math.min` of sums and products with no reordering on either side, so
/// there is no genuine language difference for a tolerance to absorb.
#[test]
fn wall_spec_and_defensibility_match_the_reference() {
    let f = fixture();

    // r at cell 1 (the lowland cell most cases sit on), asserted so a
    // fixture drift cannot quietly move every golden below.
    let r1 = civ_relative_elevation(&f, GW, GH, SEA, 1.0, 0.0);
    assert!((r1 - 0.043_478_277_222_863_58).abs() < 1e-15, "fixture drifted: r1 = {r1}");
    let r8 = civ_relative_elevation(&f, GW, GH, SEA, 2.0, 1.0);
    assert!((r8 - 0.350_000_044_395_183_7).abs() < 1e-15, "fixture drifted: r8 = {r8}");

    let mut seen_specs = std::collections::BTreeSet::new();
    let mut check = |label: &str, p: WallPlace, spec: &str, walled: bool, def: f64| {
        assert_eq!(um_wall_spec(&p), spec, "{label}: _umWallSpec");
        assert_eq!(um_infer_walls(&p), walled, "{label}: _umInferWalls");
        let got = civ_place_defensibility(p.relative_elevation, um_infer_walls(&p));
        assert!(
            (got - def).abs() < 1e-15,
            "{label}: _civPlaceDefensibility {got} vs reference {def}"
        );
        seen_specs.insert(spec.to_string());
    };

    // --- the explicit override, which outranks every rung -----------------
    let mut p = place(&f, 1, SettlementKind::Hamlet, 10.0);
    p.walls_override = Some(true);
    check("override_true_on_a_hamlet", p, "stone", true, 0.4);

    let mut p = place(&f, 1, SettlementKind::Metropolis, 900_000.0);
    p.walls_override = Some(false);
    check("override_false_on_a_metropolis", p, "none", false, 0.0);

    // --- rank >= 3 is always stone, however small -------------------------
    check("city", place(&f, 1, SettlementKind::City, 1.0), "stone", true, 0.4);
    check("capital", place(&f, 1, SettlementKind::Capital, 1.0), "stone", true, 0.4);
    check("metropolis", place(&f, 1, SettlementKind::Metropolis, 1.0), "stone", true, 0.4);

    // --- the town rung: three independent ways to earn stone --------------
    let mut young = place(&f, 1, SettlementKind::Town, 100.0);
    young.age_override = Some(30.0);
    check("young_poor_town", young, "palisade", true, 0.4);

    let mut nearly = place(&f, 1, SettlementKind::Town, 1199.0);
    nearly.age_override = Some(259.0);
    check("town_pop_1199_age_259", nearly, "palisade", true, 0.4);

    let mut wealthy = place(&f, 1, SettlementKind::Town, 1200.0);
    wealthy.age_override = Some(30.0);
    check("town_pop_1200_age_30", wealthy, "stone", true, 0.4);

    let mut old = place(&f, 1, SettlementKind::Town, 100.0);
    old.age_override = Some(260.0);
    check("town_age_260", old, "stone", true, 0.4);

    let mut threatened = place(&f, 1, SettlementKind::Town, 100.0);
    threatened.age_override = Some(30.0);
    threatened.fortified_trait = true;
    check("town_fortified_trait", threatened, "stone", true, 0.4);

    // No `umAge`: the inferred age (289 for pop 900) crosses 260 on its own.
    check(
        "town_inferred_age_900",
        place(&f, 1, SettlementKind::Town, 900.0),
        "stone",
        true,
        0.4,
    );

    // --- the garrison specialisation outranks the tier --------------------
    let mut g = place(&f, 1, SettlementKind::Hamlet, 1200.0);
    g.specialisation = Some("garrison");
    check("garrison_hamlet_1200", g, "stone", true, 0.4);
    let mut g = place(&f, 1, SettlementKind::Hamlet, 1199.0);
    g.specialisation = Some("garrison");
    check("garrison_hamlet_1199", g, "palisade", true, 0.4);

    // --- threat below town rank ------------------------------------------
    let mut v = place(&f, 1, SettlementKind::Village, 50.0);
    v.fortified_trait = true;
    check("fortified_village", v, "palisade", true, 0.4);
    let mut h = place(&f, 1, SettlementKind::Hamlet, 50.0);
    h.fortified_trait = true;
    check("fortified_hamlet", h, "ditch", true, 0.4);

    // --- command of ground ------------------------------------------------
    check(
        "commanding_village_250_at_r035",
        place(&f, 8, SettlementKind::Village, 250.0),
        "ditch",
        true,
        0.999_999_893_451_559_2,
    );
    check(
        "commanding_village_249_at_r035",
        place(&f, 8, SettlementKind::Village, 249.0),
        "none",
        false,
        0.599_999_893_451_559_1,
    );
    // See the header: `f32` storage puts this a hair above 0.9, not on it.
    check(
        "village_250_at_terrainD_exactly_09",
        place(&f, 9, SettlementKind::Village, 250.0),
        "ditch",
        true,
        0.940_000_049_327_981_9,
    );

    // --- and most places honestly have nothing ----------------------------
    check("plain_hamlet", place(&f, 1, SettlementKind::Hamlet, 40.0), "none", false, 0.0);
    check(
        "plain_village_lowland",
        place(&f, 1, SettlementKind::Village, 400.0),
        "none",
        false,
        0.0,
    );

    // The extraction's own shape assertions, re-asserted against the port.
    assert_eq!(
        seen_specs,
        ["ditch", "none", "palisade", "stone"]
            .iter()
            .map(|s| s.to_string())
            .collect::<std::collections::BTreeSet<_>>(),
        "silently-narrow golden: not every rung was reached"
    );
}

/// `_umInferAge`, which the town rung's age test leans on. Already covered
/// by a unit test in `urban_adapter`; pinned here against the reference's
/// own numbers because a drift in it silently moves the wall ladder.
#[test]
fn infer_age_matches_the_reference() {
    for (pop, expected) in
        [(0.0, 60.0), (1.0, 60.0), (100.0, 60.0), (900.0, 289.0), (1000.0, 300.0), (5000.0, 468.0), (1e9, 1000.0)]
    {
        assert_eq!(um_infer_age(pop), expected, "_umInferAge({pop})");
    }
}

/// Negative control: the ladder must not be a constant. If every rung
/// collapsed to one answer the test above would still pass its per-case
/// assertions only by accident, so assert the spread directly.
#[test]
fn the_ladder_is_actually_a_ladder() {
    let f = fixture();
    let specs: Vec<&str> = [
        SettlementKind::Hamlet,
        SettlementKind::Village,
        SettlementKind::Town,
        SettlementKind::City,
    ]
    .iter()
    .map(|&k| um_wall_spec(&place(&f, 1, k, 40.0)))
    .collect();
    assert_eq!(specs, vec!["none", "none", "palisade", "stone"]);
}
