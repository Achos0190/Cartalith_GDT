//! Golden-parity tests for the two genuinely-ported functions behind
//! `PARITY_AUDIT.md` §5 items 7, 9 and 10:
//!
//! - `_civFactionColor` (reference **14577-14586**) -- the golden-angle hue
//!   rotation that colours any faction index past the hand-picked base
//!   palette, the thing that makes "add a faction" possible without a
//!   colour picker.
//! - `_civAgrarianRegionalTotal` (reference **23516-23528**) -- the
//!   `civPopEstimateOut` "Land sustains ≈ N" readout.
//!
//! # The harness
//!
//! Node `vm.runInContext` over two **line slices**, fresh per run, not
//! checked in -- the established practice for a function with no
//! block-level dependencies (milestones B/C's precedent; the whole-`<script>`
//! boundary `golden_parity_civ_tools.rs` uses is for code that reads the
//! block's own `let` globals, which neither of these two does past the
//! stubs listed below).
//!
//! **The first slice range was wrong on the first attempt** -- 14576 caught
//! the `];` closing `CIV_FACTIONS` and the boundary assertion said so
//! immediately. That is `CLAUDE.md`'s own "verify a scope document's line
//! ranges against the real reference before slicing" rule earning its keep
//! for the fifth time; the assertions below are the reason it failed loudly
//! instead of silently.
//!
//! Boundary assertions run before anything else: each slice must *start*
//! with its own `function` declaration line, must *end* on `}`, and must
//! contain its own real `return` statement (so a slice that lost its tail
//! cannot parse-and-pass).
//!
//! `_civAgrarianRegionalTotal` reads six globals; all six are supplied as
//! context properties (`GW`, `GH`, `field`, `state`,
//! `currentCarryingCapacity`, `currentAgrarianDensity`) rather than by
//! loading the whole block. That is safe here precisely *because* the slice
//! declares none of them -- the shadowed-`let` trap `golden_parity_civ_tools
//! .rs` documents needs the reference's own `let` in scope, and this slice
//! has no `let` at file scope at all.
//!
//! # Emptiness assertions
//!
//! The extraction refused to emit a golden unless `total > 0`,
//! `landKm2 > 0`, and the fixture actually had land cells -- the
//! silently-empty-golden failure mode `CLAUDE.md` records four subsystems
//! being bitten by. All three are re-asserted here against the port.
//!
//! # The fixture is shaped to reach the code
//!
//! 6x4, `sea = 0.42`. Five cells sit *below* sea (`i % 5 == 0`) so the
//! land skip is genuinely exercised (19 of 24 land cells, asserted).
//! `field` is deliberately non-uniform and starts *at* the sea threshold
//! for i=1 -- `0.42 + i*0.011` -- so the `<` (not `<=`) comparison is
//! pinned: cell 1's value is strictly above, and a mutation to `<=` would
//! not change the count, while a mutation to `>` would flip all 24.
//! `dens` is `3.5 + i*0.25`, distinct per cell, so summing the wrong cells
//! or dropping the per-cell multiply both move the answer.
//!
//! Two cases differing **only** in `mapWidthKm` (800 vs 1250) pin the
//! `cellKm = mapWidthKm / GW` division and the `cellKm2` square: the
//! totals differ by exactly `(1250/800)^2`, which a dropped square would
//! not reproduce.

use cartalith_civ::roster::civ_faction_color;
use cartalith_civ::timeline::civ_agrarian_regional_total;

/// The extraction's own fixture, rebuilt identically here.
fn fixture() -> (Vec<f32>, Vec<f32>) {
    let n = 6 * 4;
    let mut field = vec![0f32; n];
    let mut dens = vec![0f32; n];
    for i in 0..n {
        field[i] = if i % 5 == 0 { 0.10 } else { (0.42 + i as f64 * 0.011) as f32 };
        dens[i] = (3.5 + i as f64 * 0.25) as f32;
    }
    (field, dens)
}

#[test]
fn faction_color_matches_the_reference_at_every_hue_sector() {
    // Node `vm.runInContext` over reference lines 14577-14586.
    const GOLDEN: [(usize, (u8, u8, u8)); 13] = [
        (0, (198, 57, 57)),
        (1, (57, 198, 98)),
        (2, (139, 57, 198)),
        (3, (198, 180, 57)),
        (4, (57, 174, 198)),
        (5, (198, 57, 133)),
        (6, (92, 198, 57)),
        (7, (63, 57, 198)),
        (8, (198, 104, 57)),
        (12, (57, 127, 198)),
        (13, (198, 57, 86)),
        (40, (103, 198, 57)),
        (63, (198, 111, 57)),
    ];
    for (i, expected) in GOLDEN {
        assert_eq!(
            civ_faction_color(i),
            expected,
            "_civFactionColor({i}) diverged from the reference"
        );
    }
    // Negative control: adjacent indices must not collide, which is the
    // entire point of the golden angle.
    for i in 0..40usize {
        assert_ne!(civ_faction_color(i), civ_faction_color(i + 1));
    }
}

#[test]
fn agrarian_regional_total_matches_the_reference() {
    let (field, dens) = fixture();
    assert_eq!(
        field.iter().filter(|&&v| (v as f64) >= 0.42).count(),
        19,
        "fixture drifted -- the golden below was extracted against 19 land cells"
    );

    // Case 1: mapWidthKm 800, GW 6 -> cellKm 133.333..., cellKm2 17777.77...
    let out = civ_agrarian_regional_total(&dens, &field, 0.42, 800.0 / 6.0);
    assert_eq!(out.total, 2_186_667.0);
    assert_eq!(out.land_km2, 337_778.0);
    assert!(out.total > 0.0 && out.land_km2 > 0.0, "silently-empty golden");

    // Case 2: identical fixture, mapWidthKm 1250 -- pins cellKm and its square.
    let out2 = civ_agrarian_regional_total(&dens, &field, 0.42, 1250.0 / 6.0);
    assert_eq!(out2.total, 5_338_542.0);
    assert_eq!(out2.land_km2, 824_653.0);

    // The ratio is (1250/800)^2 exactly, modulo the two roundings -- a
    // dropped square would give 1250/800 instead.
    let ratio = out2.total / out.total;
    let expected = (1250.0f64 / 800.0).powi(2);
    assert!(
        (ratio - expected).abs() < 1e-4,
        "cellKm2 is not squared: ratio {ratio} vs {expected}"
    );
}

#[test]
fn agrarian_regional_total_skips_sub_sea_cells() {
    let (field, dens) = fixture();
    // Raising sea above every cell must give exactly zero -- the land gate
    // is a real gate, not decoration.
    let none = civ_agrarian_regional_total(&dens, &field, 1.0, 800.0 / 6.0);
    assert_eq!(none.total, 0.0);
    assert_eq!(none.land_km2, 0.0);
    // Dropping sea below every cell must include all 24.
    let all = civ_agrarian_regional_total(&dens, &field, 0.0, 800.0 / 6.0);
    assert!(all.total > civ_agrarian_regional_total(&dens, &field, 0.42, 800.0 / 6.0).total);
    assert_eq!(all.land_km2, ((24.0 * (800.0f64 / 6.0).powi(2)) as f64).round());
}
