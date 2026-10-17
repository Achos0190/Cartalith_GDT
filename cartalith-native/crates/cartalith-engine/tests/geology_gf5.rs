//! GF-5 of `GEOLOGY_FIRST_SCOPE.md` (owner Rulings BH and BJ): karst
//! dissolution and permeability-reduced runoff behind
//! `WorldParams::geology_karst`, on whole generated worlds.
//!
//! **What these tests protect.**
//! - The gate: with the processes off (the app today) or the model off (the
//!   parity baseline) the switch changes nothing, bit for bit; it is part of
//!   the gated processes, not a parameter of its own.
//! - The switch is alive: with the processes on it moves the world (the
//!   field or the stream order), and a karst world is deterministic. The
//!   pass's own input surface is not observable from a `WorldState`, so
//!   which cells it lowered is measured in the harness, not here.
//!
//! Identity against a build without the switch (HEAD) cannot be asserted in
//! one binary; it is the hash printout of `cartalith-godot/tests/gf5_karst.rs`
//! `gf5_identity_hashes`, compared across two builds and recorded in
//! `GEOLOGY_FIRST_SCOPE.md` §7 "GF-5 measured". The pass's B5 exactness is
//! pinned on literal fixtures in `cartalith-erosion/src/karst.rs`.

use cartalith_engine::{generate_terrain, WorldParams, WorldState};

/// Fixture grid width. Source: the size the other GF fixture suites in this
/// crate use (`geology_gf7.rs`), so results are comparable; not a scope value.
const GW: usize = 256;
/// Fixture grid height, paired with [`GW`]. Same provenance.
const GH: usize = 164;

/// An app-like world (`cartalith-godot`'s `params::defaults` divergences,
/// the list `geology_gf7.rs` keeps) with the model on and the processes and
/// karst switches as given.
fn app_params(seed: i32, processes: bool, karst: bool) -> WorldParams {
    let mut p = WorldParams::defaults(GW, GH, seed);
    p.crater.physical_model = true;
    p.volc.exclude_transform = true;
    p.volc.edifice_model = true;
    p.integrate_drainage = true;
    p.tect.narrow_plate_base_blur = true;
    p.passes.glacial = true;
    p.geology_model = true;
    p.geology_processes = processes;
    p.geology_karst = karst;
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

/// Processes off, or the model off: the karst switch is inert, bit for bit.
#[test]
fn with_the_processes_off_the_karst_switch_changes_nothing() {
    // Protects: the gate -- geology_karst must not move a world whose
    // processes are off (the app) or that has no column (the parity path).
    // 512 x 328 as well as the fixture grid: at 256 x 164 no land exposes
    // limestone, so a dissolution gate that leaked would have nothing to
    // dissolve there and this test could not see it (a mutant survived).
    for (gw, gh) in [(GW, GH), (512, 328)] {
        for seed in [12345, 483920] {
            let off = generate_terrain(&WorldParams { gw, gh, ..app_params(seed, false, false) });
            let on = generate_terrain(&WorldParams { gw, gh, ..app_params(seed, false, true) });
            assert_same(&off, &on, &format!("seed {seed} at {gw}x{gh}, processes off"));
        }
    }
    let parity = WorldParams::defaults(GW, GH, 7);
    let with = generate_terrain(&WorldParams { geology_karst: true, ..parity.clone() });
    assert_same(&generate_terrain(&parity), &with, "parity baseline");
}

/// Processes on: the switch moves the world on at least one of two seeds
/// whose column exposes limestone or permeable rock, so an inert wiring
/// cannot pass; and a karst world is deterministic.
#[test]
fn with_the_processes_on_the_karst_switch_moves_the_world() {
    // Protects: the switch is genuinely wired (dissolution and the runoff
    // seeding the network), not an inert knob; and determinism with it on.
    let mut moved_any = false;
    for seed in [12345, 483920] {
        let off = generate_terrain(&app_params(seed, true, false));
        let on = generate_terrain(&app_params(seed, true, true));
        let moved = bits(&off.field) != bits(&on.field) || off.stream_order != on.stream_order;
        moved_any |= moved;
        let again = generate_terrain(&app_params(seed, true, true));
        assert_same(&on, &again, &format!("seed {seed}, karst on twice"));
    }
    assert!(moved_any, "geology_karst moved neither world");
}

/// Both halves of the switch reach the world, each by its own signature:
/// dissolution lowers exposed limestone away from the rivers (the carve
/// cannot be what moved those cells), and permeable outcrops carry fewer
/// channel cells, because their runoff seeds less flow.
#[test]
fn dissolution_lowers_limestone_and_permeable_rock_loses_channels() {
    // Protects: each half is wired at its call site -- removing the
    // dissolution call, or seeding the network with the rain instead of the
    // runoff, must turn this red (mutation-checked, GEOLOGY_FIRST_SCOPE.md §7
    // "GF-5 measured").
    use cartalith_terrain::geology::{m_to_norm, Rock, R_EXPOSE_M};
    let (mut low_sum, mut low_n) = (0.0f64, 0usize);
    let (mut perm_off, mut perm_on) = (0usize, 0usize);
    // 512 x 328: at the 256 x 164 fixture no land exposes limestone (the
    // cover facies needs structural lows a coarse grid does not resolve).
    let (gw, gh) = (512usize, 328usize);
    for seed in [12345, 483920, 24601] {
        let p = WorldParams { gw, gh, ..app_params(seed, true, false) };
        let off = generate_terrain(&p);
        let on = generate_terrain(&WorldParams { geology_karst: true, ..p.clone() });
        let sea = off.sea_level;
        let mpu = p.peak_m / (1.0 - sea);
        let r = m_to_norm(R_EXPOSE_M, sea, p.peak_m) as f32;
        let (co, cn) = (off.geology.column().unwrap(), on.geology.column().unwrap());
        let (mo, mn) = (off.river_mask.as_ref().unwrap(), on.river_mask.as_ref().unwrap());
        let (ho, hn) = (&off.channels.as_ref().unwrap().chan, &on.channels.as_ref().unwrap().chan);
        for i in 0..gw * gh {
            if off.field[i] as f64 <= sea {
                continue;
            }
            if co.exposed(i, off.field[i], r) == Some(Rock::Limestone) && mo[i] == 0 && mn[i] == 0 {
                low_sum += (off.field[i] - on.field[i]) as f64 * mpu;
                low_n += 1;
            }
            let high = |c: &cartalith_terrain::geology::GeologyColumn, z: f32| {
                c.exposed(i, z, r).is_some_and(|k| k.props().permeability == cartalith_terrain::geology::Permeability::High)
            };
            if ho[i] != 0 && high(co, off.field[i]) {
                perm_off += 1;
            }
            if hn[i] != 0 && high(cn, on.field[i]) {
                perm_on += 1;
            }
        }
    }
    let mean = low_sum / low_n.max(1) as f64;
    println!("limestone off-river cells {low_n}, mean lowering {mean:.3} m; channel cells on high-permeability rock {perm_off} -> {perm_on}");
    assert!(low_n >= 100, "the fixture worlds expose too little limestone: {low_n}");
    assert!(mean > 1.0, "dissolution did not lower limestone: mean {mean} m");
    assert!(perm_on < perm_off, "permeable rock did not lose channel cells: {perm_off} -> {perm_on}");
}

/// GF-7's τ inside GF-5's budget (§4.12's karst row): exactly 1 at τ = 1 and
/// whenever the clock's gate is off, and τ itself (linear) otherwise.
#[test]
fn the_karst_budget_is_linear_in_age_and_one_when_the_clock_is_off() {
    // Protects: the dissolution depth scales with geological age only when
    // the clock acts, linearly, and is today's unscaled expression at τ = 1
    // or with the processes off. Literals, not the clock's own constants.
    use cartalith_engine::geo_clock::GeoClock;
    assert_eq!(GeoClock::new(true, 1.0).karst_budget_factor(), 1.0);
    assert_eq!(GeoClock::new(false, 4.0).karst_budget_factor(), 1.0);
    assert_eq!(GeoClock::new(true, 4.0).karst_budget_factor(), 4.0);
    assert_eq!(GeoClock::new(true, 0.25).karst_budget_factor(), 0.25);
    assert_eq!(GeoClock::new(true, 2.0).karst_budget_factor(), 2.0);
}
