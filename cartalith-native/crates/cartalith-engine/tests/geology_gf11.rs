//! GF-11 of `GEOLOGY_FIRST_SCOPE.md` (owner Rulings BM and BY): rock-aware
//! construction behind `WorldParams::relief_model`, on whole generated worlds.
//!
//! **What these tests protect.**
//! - The gate: the switch acts only with `geology_model` and
//!   `geology_processes` on; otherwise the world is bit-identical to the one
//!   without it (identity by control flow, §6.3).
//! - The stage is alive and in §4.16's place: on, the world moves, the column
//!   records the structural surface (the pre-erosion world's own field), the
//!   contact is the one the geology stage derived, and the light pass's
//!   stream power and rebound are not what lowered it.
//! - The clock scales the budget (§4.16): a different τ is a different world.
//! - A skip (world-structure on) is recorded with its reason and leaves the
//!   processes-on world, bit for bit.
//!
//! That the built stage is the arm GF-11's bars measured, bit for bit, is
//! `cartalith-godot`'s `gf0_geology_harness.rs`
//! (`gf11_built_stage_reproduces_the_measured_arm`), which holds the
//! measured prototype.

use cartalith_engine::{generate_terrain, ErosionPassParams, WorldParams, WorldState};
use cartalith_terrain::geology::{Construction, ConstructionSkip};

/// Fixture grid width: the size the other GF suites in this crate use
/// (`geology_gf1.rs`, `geology_gf2.rs`, `geology_gf7.rs`), not a scope value.
const GW: usize = 256;
/// Fixture grid height, paired with [`GW`]. Same provenance.
const GH: usize = 164;

/// An app-like world (`cartalith-godot`'s `params::defaults` divergences,
/// which this crate cannot import -- the list `geology_gf7.rs` keeps) with the
/// model on and the two switches as given.
fn app_params(seed: i32, processes: bool, relief: bool, age: f64) -> WorldParams {
    let mut p = WorldParams::defaults(GW, GH, seed);
    p.crater.physical_model = true;
    p.volc.exclude_transform = true;
    p.volc.edifice_model = true;
    p.integrate_drainage = true;
    p.tect.narrow_plate_base_blur = true;
    p.passes.glacial = true;
    p.geology_model = true;
    p.geology_processes = processes;
    p.relief_model = relief;
    p.geo_age = age;
    p
}

/// Bit-pattern view of an `f32` slice, for exact equality.
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
    let (ca, cb) = (a.geology.column(), b.geology.column());
    assert_eq!(ca.is_some(), cb.is_some(), "{what}: column presence");
    if let (Some(ca), Some(cb)) = (ca, cb) {
        assert_eq!(bits(&ca.regolith), bits(&cb.regolith), "{what}: regolith");
        assert_eq!(bits(&ca.contact), bits(&cb.contact), "{what}: contact");
        assert_eq!(ca.rock_top, cb.rock_top, "{what}: rock_top");
    }
}

/// The structural surface: the same world with the carve and every pass
/// off. `generate_terrain_inner` writes nothing to `field` after the geology
/// stage on that path (the processes switch off, so no construction either).
fn structural(seed: i32) -> WorldState {
    let mut p = app_params(seed, false, false, 1.0);
    p.carve_rivers = false;
    p.passes = ErosionPassParams::off();
    generate_terrain(&p)
}

/// Protects: §6.3's switch-off identity -- with `geology_processes` off (the
/// app today) or the model off (the parity baseline), turning `relief_model`
/// on changes nothing, bit for bit; and with the processes on, the switch
/// off is the processes-on world exactly (the construction block is skipped
/// by control flow).
#[test]
fn the_switch_is_inert_without_the_rock_gate() {
    for seed in [12345, 483920] {
        let app = generate_terrain(&app_params(seed, false, false, 1.0));
        assert_same(&app, &generate_terrain(&app_params(seed, false, true, 1.0)), &format!("seed {seed}: processes off"));
        let col = app.geology.column().expect("the model is on");
        assert_eq!(col.construction, Construction::Off, "no construction was requested");
    }
    let parity = WorldParams::defaults(GW, GH, 7);
    let on = generate_terrain(&WorldParams { relief_model: true, geology_processes: true, ..parity.clone() });
    assert_eq!(bits(&on.field), bits(&generate_terrain(&parity).field), "parity baseline: no column, so no construction");
}

/// Protects: the stage runs, in §4.16's place, and records what it should.
/// With both switches on the world moves against the processes-on world;
/// the column's record is `Built` and holds the structural surface (the
/// carve-off, passes-off world's field, bit for bit); the contact equals the
/// structural world's except where glacial rebound lifts it (construction
/// itself never writes it), checked as: no contact cell moved downward; and
/// the regolith holds the fill (some cell gained).
#[test]
fn the_stage_runs_and_records_the_structural_surface() {
    for seed in [12345, 483920] {
        let s = structural(seed);
        let proc_on = generate_terrain(&app_params(seed, true, false, 1.0));
        let relief = generate_terrain(&app_params(seed, true, true, 1.0));
        assert_ne!(bits(&relief.field), bits(&proc_on.field), "seed {seed}: construction must move the world");
        let col = relief.geology.column().expect("column");
        match &col.construction {
            Construction::Built { structural_surface } => {
                assert_eq!(bits(structural_surface), bits(&s.field), "seed {seed}: the record must be the structural surface")
            }
            other => panic!("seed {seed}: expected Built, got {other:?}"),
        }
        let c0 = s.geology.column().expect("column");
        let down = (0..c0.len()).filter(|&i| !c0.contact[i].is_nan() && col.contact[i] < c0.contact[i]).count();
        assert_eq!(down, 0, "seed {seed}: no contact may move down (construction never writes it; only rebound lifts it)");
        assert!(col.regolith.iter().any(|&r| r > 0.0), "seed {seed}: the fill or the passes must leave regolith");
        // Land was lowered on balance: the mean field of the constructed
        // world's land is below the structural surface's.
        let sea = relief.sea_level;
        let land: Vec<usize> = (0..s.field.len()).filter(|&i| s.field[i] as f64 > sea).collect();
        let mean = |f: &[f32]| land.iter().map(|&i| f[i] as f64).sum::<f64>() / land.len() as f64;
        assert!(mean(&relief.field) < mean(&s.field), "seed {seed}: construction must lower land on balance");
    }
}

/// Protects: the clock reaches the budget (§4.16: "τ scales the construction
/// budget") -- at τ = 2 the constructed world differs from τ = 1's, and
/// lowers land further on balance.
#[test]
fn the_clock_scales_the_budget() {
    let seed = 483920;
    let s = structural(seed);
    let one = generate_terrain(&app_params(seed, true, true, 1.0));
    let two = generate_terrain(&app_params(seed, true, true, 2.0));
    assert_ne!(bits(&one.field), bits(&two.field), "tau must move a constructed world");
    let sea = one.sea_level;
    let land: Vec<usize> = (0..s.field.len()).filter(|&i| s.field[i] as f64 > sea).collect();
    let mean = |f: &[f32]| land.iter().map(|&i| f[i] as f64).sum::<f64>() / land.len() as f64;
    assert!(mean(&two.field) < mean(&one.field), "an older constructed world is lower on balance");
}

/// Protects: the world-structure skip -- recorded with its reason, the field
/// untouched by construction, and the light pass still running, so the world
/// is exactly the processes-on world with world-structure on.
#[test]
fn a_skipped_construction_is_recorded_and_leaves_the_processes_world() {
    let seed = 12345;
    let mut on = app_params(seed, true, true, 1.0);
    on.world_structure.enabled = true;
    let mut off = on.clone();
    off.relief_model = false;
    let a = generate_terrain(&on);
    assert_eq!(a.geology.column().expect("column").construction, Construction::Skipped(ConstructionSkip::WorldStructureOn));
    assert_same(&a, &generate_terrain(&off), "world-structure skip");
}
