//! GF-7 of `GEOLOGY_FIRST_SCOPE.md` (owner Ruling BJ): the geological clock,
//! on whole generated worlds.
//!
//! **What these tests protect.**
//! - The gate: with `geology_processes` off (the app today) τ changes nothing,
//!   bit for bit, at either end of its range -- the clock is part of the gated
//!   processes, not a parameter of its own.
//! - The clock is alive: with the processes on, τ = 4 moves the world, and
//!   deepens it (mean lowering from the pre-erosion surface rises with τ), so
//!   an inert wiring cannot pass.
//! - Determinism at a scaled age.
//!
//! τ = 1's identity against a world generated *without* the clock cannot be
//! asserted inside one binary (there is no clock-less build to call); it is a
//! hash comparison against HEAD `ec4e078`, recorded in §5.8. The count laws
//! are pinned by literal in `src/geo_clock.rs`'s tests.

use cartalith_engine::{generate_terrain, WorldParams, WorldState};

/// Fixture grid width. Source: matches the size other GF fixture suites in
/// this crate use (`geology_gf1.rs`/`geology_gf2.rs`) so results are
/// comparable across them -- not a scope-cited value.
const GW: usize = 256;
/// Fixture grid height, paired with [`GW`]. Same provenance.
const GH: usize = 164;

/// An app-like world (`cartalith-godot`'s `params::defaults` divergences,
/// which this crate cannot import -- the list `geology_gf2.rs` keeps) with
/// the model on and the processes switch as given.
fn app_params(seed: i32, processes: bool, age: f64) -> WorldParams {
    let mut p = WorldParams::defaults(GW, GH, seed);
    p.crater.physical_model = true;
    p.volc.exclude_transform = true;
    p.volc.edifice_model = true;
    p.integrate_drainage = true;
    p.tect.narrow_plate_base_blur = true;
    p.passes.glacial = true;
    p.geology_model = true;
    p.geology_processes = processes;
    p.geo_age = age;
    p
}

/// Bit-pattern view of an `f32` slice, for exact (not epsilon) equality --
/// two floats that print the same can still differ in the low bits.
fn bits(v: &[f32]) -> Vec<u32> {
    v.iter().map(|x| x.to_bits()).collect()
}

/// Every array a caller reads, compared bit for bit.
fn assert_same(a: &WorldState, b: &WorldState, what: &str) {
    assert_eq!(bits(&a.field), bits(&b.field), "{what}: field");
    assert_eq!(bits(&a.rainfall), bits(&b.rainfall), "{what}: rainfall");
    assert_eq!(bits(&a.temperature), bits(&b.temperature), "{what}: temperature");
    assert_eq!(bits(&a.flow_discharge), bits(&b.flow_discharge), "{what}: flow_discharge");
    assert_eq!(a.stream_order, b.stream_order, "{what}: stream_order");
    assert_eq!(a.river_mask, b.river_mask, "{what}: river_mask");
    let (ca, cb) = (a.geology.column().expect("column"), b.geology.column().expect("column"));
    assert_eq!(bits(&ca.regolith), bits(&cb.regolith), "{what}: regolith");
    assert_eq!(bits(&ca.contact), bits(&cb.contact), "{what}: contact");
}

/// Processes off: τ at both ends of its range is the τ = 1 world, bit for
/// bit, on the parity baseline and on an app-like world. This is what keeps
/// the app's default world fixed while the slider moves.
#[test]
fn with_the_processes_off_age_changes_nothing() {
    // Protects: the gate -- with geology_processes off, geo_age must not move
    // the world bit for bit, at either end of its range, on an app-like world
    // and on the bare parity baseline.
    for seed in [12345, 483920] {
        let base = generate_terrain(&app_params(seed, false, 1.0));
        for age in [0.25, 4.0] {
            assert_same(&base, &generate_terrain(&app_params(seed, false, age)), &format!("seed {seed} age {age}"));
        }
    }
    let parity = WorldParams::defaults(GW, GH, 7);
    let old = generate_terrain(&WorldParams { geo_age: 4.0, ..parity.clone() });
    let new = generate_terrain(&parity);
    assert_eq!(bits(&old.field), bits(&new.field), "parity baseline: geo_age has no effect without the model");
}

/// Processes on: τ = 4 moves the world, and the mean lowering of land from
/// the pre-erosion surface rises strictly with τ (B12's incision clause, in
/// miniature). The pre-erosion surface is the same world with the carve and
/// the passes off, which τ cannot reach.
#[test]
fn with_the_processes_on_older_worlds_are_more_eroded() {
    // Protects: the clock is genuinely wired, not an inert knob -- tau=4 must
    // move a processes-on world, and mean land lowering from the pre-erosion
    // surface must rise strictly with tau.
    let seed = 483920;
    let mut pre_p = app_params(seed, true, 1.0);
    pre_p.carve_rivers = false;
    pre_p.passes = cartalith_engine::ErosionPassParams::off();
    let pre = generate_terrain(&pre_p);
    let sea = pre.sea_level as f32;
    let lowering = |age: f64| {
        let ws = generate_terrain(&app_params(seed, true, age));
        let (mut sum, mut n) = (0.0f64, 0usize);
        for (a, b) in pre.field.iter().zip(ws.field.iter()) {
            if *a > sea {
                sum += (*a - *b) as f64;
                n += 1;
            }
        }
        (sum / n.max(1) as f64, ws)
    };
    let (l_half, _) = lowering(0.5);
    let (l1, w1) = lowering(1.0);
    let (l4, w4) = lowering(4.0);
    assert_ne!(bits(&w1.field), bits(&w4.field), "tau = 4 must move a processes-on world");
    assert!(l_half < l1 && l1 < l4, "mean land lowering must rise with tau: {l_half} {l1} {l4}");
}

/// Every clocked call site reads the clock, not the raw count. At τ = 0.25
/// the floors make different raw counts run the same effective count
/// (arithmetic, `src/geo_clock.rs`): `stream.iters` 10 and 14 both give 4
/// light-pass, 4 evolve and 4 sediment-fill iterations; glacial 2 and 4 both
/// give 1 pass; coastal 2 and 3 both give 1 pass. So with every pass on, the
/// two worlds must be bit-identical -- and a call site that bypassed the
/// clock would run its raw count and split them. At τ = 1 the same two
/// parameter sets must differ, which proves the knobs are live (a test whose
/// knobs did nothing could not fail).
#[test]
fn every_clocked_call_site_runs_the_clocks_count() {
    // Protects: every clocked call site reads GeoClock's effective count, not
    // its own raw parameter -- a bypassed call site would split two worlds
    // the floor arithmetic says must be bit-identical at tau=0.25.
    let world = |age: f64, iters: i32, glacial: i32, coastal: i32| {
        let mut p = app_params(12345, true, age);
        p.passes.coastal = true;
        p.passes.sediment_fill = true;
        p.passes.evolve_cycles = 1;
        p.stream.iters = iters;
        p.passes.glacial_passes = glacial;
        p.passes.coastal_passes = coastal;
        generate_terrain(&p)
    };
    let a = world(0.25, 10, 2, 2);
    let b = world(0.25, 14, 4, 3);
    assert_same(&a, &b, "tau 0.25: equal effective counts");
    // Each knob alone at tau = 1 must move the world.
    let base = world(1.0, 10, 2, 2);
    for (label, other) in [("iters", world(1.0, 14, 2, 2)), ("glacial", world(1.0, 10, 4, 2)), ("coastal", world(1.0, 10, 2, 3))] {
        assert_ne!(bits(&base.field), bits(&other.field), "tau 1: the {label} knob must be live");
    }
}

/// A scaled age is as deterministic as the default one.
#[test]
fn a_scaled_age_is_deterministic() {
    // Protects: determinism at a scaled (non-default) tau -- the same seed and
    // age must generate bit-identical worlds twice.
    let p = app_params(24601, true, 2.5);
    assert_same(&generate_terrain(&p), &generate_terrain(&p), "same seed twice at tau 2.5");
}
