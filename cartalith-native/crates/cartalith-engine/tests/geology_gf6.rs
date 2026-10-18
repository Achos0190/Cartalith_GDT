//! GF-6 of `GEOLOGY_FIRST_SCOPE.md` (owner Ruling BH): the river carve's
//! rock-dependent valley width `ω` (§4.7) behind `WorldParams::geology_carve`,
//! on whole generated worlds.
//!
//! **What these tests protect.**
//! - The gate: with the processes off (the app today) or the model off (the
//!   parity baseline) the switch changes nothing, bit for bit.
//! - The switch is alive and points the right way: with the processes on,
//!   carved cells on weak rock (`ω > 1`) grow and on strong rock (`ω < 1`)
//!   shrink, and a carve-on world is deterministic.
//!
//! Identity against a build without the switch (HEAD) cannot be asserted in
//! one binary; it is the hash printout of `cartalith-civ/tests/gf6_carve.rs`
//! `gf6_carve_identity_hashes`, compared across two builds and recorded in
//! `GEOLOGY_FIRST_SCOPE.md` §7 "GF-6 measured: the carve half". `ω` itself is
//! pinned on literal fixtures in `cartalith-erosion/src/carve_width.rs`.

use cartalith_engine::{generate_terrain, WorldParams, WorldState};

/// Fixture grid width. Source: the size the other GF fixture suites in this
/// crate use (`geology_gf7.rs`), so results are comparable; not a scope value.
const GW: usize = 256;
/// Fixture grid height, paired with [`GW`]. Same provenance.
const GH: usize = 164;

/// An app-like world (`cartalith-godot`'s `params::defaults` divergences,
/// the list `geology_gf7.rs` keeps) with the model on and the processes and
/// carve switches as given.
fn app_params(seed: i32, processes: bool, carve: bool) -> WorldParams {
    let mut p = WorldParams::defaults(GW, GH, seed);
    p.crater.physical_model = true;
    p.volc.exclude_transform = true;
    p.volc.edifice_model = true;
    p.integrate_drainage = true;
    p.tect.narrow_plate_base_blur = true;
    p.passes.glacial = true;
    p.geology_model = true;
    p.geology_processes = processes;
    p.geology_carve = carve;
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
    match (a.geology.column(), b.geology.column()) {
        (Some(ca), Some(cb)) => {
            assert_eq!(bits(&ca.regolith), bits(&cb.regolith), "{what}: regolith");
            assert_eq!(bits(&ca.contact), bits(&cb.contact), "{what}: contact");
        }
        (None, None) => {}
        _ => panic!("{what}: one world has a column and the other does not"),
    }
}

/// Processes off, or the model off: the carve switch is inert, bit for bit.
#[test]
fn with_the_processes_off_the_carve_switch_changes_nothing() {
    // Protects: the gate -- geology_carve must not move a world whose
    // processes are off (the app) or that has no column (the parity path).
    for (gw, gh) in [(GW, GH), (512, 328)] {
        for seed in [12345, 483920] {
            let off = generate_terrain(&WorldParams { gw, gh, ..app_params(seed, false, false) });
            let on = generate_terrain(&WorldParams { gw, gh, ..app_params(seed, false, true) });
            assert_same(&off, &on, &format!("seed {seed} at {gw}x{gh}, processes off"));
        }
    }
    let parity = WorldParams::defaults(GW, GH, 7);
    let with = generate_terrain(&WorldParams { geology_carve: true, ..parity.clone() });
    assert_same(&generate_terrain(&parity), &with, "parity baseline");
}

/// Processes on: carved cells (`river_mask`) on weak rock grow and on strong
/// rock shrink, read against the switch-off world's own exposed rock; and a
/// carve-on world is deterministic.
#[test]
fn with_the_processes_on_valleys_widen_in_weak_rock_and_narrow_in_strong() {
    // Protects: the switch is wired at the carve's call site and the factor
    // points the way §4.7 says. Removing the call, or inverting the slope,
    // must turn this red (mutation-checked, GEOLOGY_FIRST_SCOPE.md §7).
    use cartalith_erosion::carve_width::valley_width_factor;
    use cartalith_terrain::geology::{m_to_norm, R_EXPOSE_M};
    let (gw, gh) = (512usize, 328usize);
    let (mut weak_off, mut weak_on, mut strong_off, mut strong_on) = (0usize, 0usize, 0usize, 0usize);
    for seed in [12345, 483920] {
        let p = WorldParams { gw, gh, ..app_params(seed, true, false) };
        let off = generate_terrain(&p);
        let on = generate_terrain(&WorldParams { geology_carve: true, ..p.clone() });
        let again = generate_terrain(&WorldParams { geology_carve: true, ..p.clone() });
        assert_same(&on, &again, &format!("seed {seed}, carve on twice"));
        let r = m_to_norm(R_EXPOSE_M, off.sea_level, p.peak_m) as f32;
        let col = off.geology.column().unwrap();
        let (mo, mn) = (off.river_mask.as_ref().unwrap(), on.river_mask.as_ref().unwrap());
        for i in 0..gw * gh {
            let Some(rock) = col.exposed(i, off.field[i], r) else { continue };
            let w = valley_width_factor(rock);
            let (a, b) = ((mo[i] != 0) as usize, (mn[i] != 0) as usize);
            if w > 1.0 + 1e-6 {
                weak_off += a;
                weak_on += b;
            } else if w < 1.0 - 1e-6 {
                strong_off += a;
                strong_on += b;
            }
        }
    }
    println!("carved cells on weak rock {weak_off} -> {weak_on}; on strong rock {strong_off} -> {strong_on}");
    assert!(weak_off >= 100 && strong_off >= 100, "too few carved cells to read: weak {weak_off}, strong {strong_off}");
    assert!(weak_on > weak_off, "weak-rock valleys did not widen: {weak_off} -> {weak_on}");
    assert!(strong_on < strong_off, "strong-rock valleys did not narrow: {strong_off} -> {strong_on}");
}
