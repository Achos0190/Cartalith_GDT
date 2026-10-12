//! GF-4 (`GEOLOGY_FIRST_SCOPE.md` §3.2 item 2, §4.5; owner Rulings BH and
//! BJ): the coastal pass runs ahead of the river trace. The kernel tests are
//! `cartalith-erosion`'s `passes::gf4_rock_tests`; the pass's wiring through
//! the rock context is `lib.rs`'s
//! `gf4_rock_context_coastal_reads_rock_and_accounts_debris_as_regolith`;
//! the measurement is `cartalith-godot/tests/gf4_glacial_coastal.rs`.

use cartalith_engine::{generate_terrain, WorldParams};

/// Cells the river carve marked (`river_mask`) that ended below sea level,
/// and the number of carved cells (asserted non-zero by the caller, so the
/// count cannot pass vacuously).
fn sunk_river_cells(p: &WorldParams) -> (usize, usize) {
    let ws = generate_terrain(p);
    let rm = ws.river_mask.as_ref().expect("carve_rivers is on");
    let carved = rm.iter().filter(|&&m| m != 0).count();
    let sunk = (0..rm.len()).filter(|&i| rm[i] != 0 && (ws.field[i] as f64) < ws.sea_level).count();
    (sunk, carved)
}

/// Protects: GF-4's move of the coastal pass ahead of the trace. With the
/// pass on, no cell the carve marked as river is below sea level, because
/// the network is traced on the surface the pass leaves. Run on the tree
/// before the move (`e80ba61a`, the pass after the carve) this test failed:
/// 43 carved river cells below sea on seed 483920. A move back after the
/// trace turns it red.
#[test]
fn with_the_coastal_pass_on_no_carved_river_cell_is_sea() {
    for seed in [483_920, 314_159] {
        let mut p = WorldParams::defaults(256, 164, seed);
        p.passes.coastal = true;
        let (sunk, carved) = sunk_river_cells(&p);
        assert!(carved > 100, "seed {seed}: the fixture must carve rivers, carved {carved}");
        assert_eq!(sunk, 0, "seed {seed}: {sunk} carved river cells are below sea with the coastal pass on");
    }
}

/// Protects: the control for the test above. Without the coastal pass the
/// same worlds carve no river cell below sea either, so the guarantee above
/// is the carve's own (RV-1), kept by the move, not one the pass adds.
#[test]
fn without_the_coastal_pass_no_carved_river_cell_is_sea() {
    for seed in [483_920, 314_159] {
        let (sunk, carved) = sunk_river_cells(&WorldParams::defaults(256, 164, seed));
        assert!(carved > 100, "seed {seed}: the fixture must carve rivers, carved {carved}");
        assert_eq!(sunk, 0, "seed {seed}: {sunk} carved river cells are below sea without the coastal pass");
    }
}
