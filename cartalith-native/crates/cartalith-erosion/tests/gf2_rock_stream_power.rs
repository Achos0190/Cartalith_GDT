//! GF-2 of `GEOLOGY_FIRST_SCOPE.md` (owner Rulings BH and BJ): stream power
//! reads the exposed rock and switches to the substrate mid-solve once a cap
//! is breached; the caller's net-change rule strips regolith before bedrock
//! and writes deposits to regolith (§4.9); and rebound lifts the column.
//!
//! **What these tests protect.** Every expected value is a literal computed by
//! hand from §4.1's formula on a 3 x 3 fixture small enough to follow, never
//! the code's own constant read back (`MISTAKES.md`: never assert a constant
//! against itself). The legacy kernel's own goldens
//! (`golden_parity_streampower.rs`) protect the path these tests do not take.
//!
//! **The fixture.** A 3 x 3 grid whose eight border cells are priority-flood
//! seeds at height 0 and never move (their slope to every neighbour is not
//! positive, so they have no receiver). The centre drains to its first
//! orthogonal neighbour in scan order (index 1, `L = 1`), has drainage area
//! `A = 1`, and with `rain = 0`, `uplift = 0`, `deposit = 0` and `g = 1` its
//! implicit update is `z ← z / (1 + c)` with `c = K·κ^contrast`.

use cartalith_erosion::{account_regolith, kappa_multiplier, lift_column, stream_power_kernel_rock, StreamPowerParams, StreamPowerRock};
use cartalith_terrain::geology::{GeologyColumn, Rock, NO_LAYER};

const C: usize = 4; // the centre of the 3 x 3 grid

fn params(k: f64, iters: i32) -> StreamPowerParams {
    StreamPowerParams { k, uplift: 0.0, deposit: 0.0, climate_k: 0.0, iters, resist: 0.5, g: 1.0, world: false, sea: 0.0 }
}

/// A 3 x 3 column: every cell `top`, the centre optionally two-layer.
fn column(top: Rock, sub: Option<(Rock, f32)>, regolith_c: f32) -> GeologyColumn {
    let mut col = GeologyColumn {
        rock_top: vec![top as u8; 9],
        rock_sub: vec![NO_LAYER; 9],
        contact: vec![f32::NAN; 9],
        regolith: vec![0.0; 9],
        volcanic_setting: vec![0; 9],
    };
    if let Some((s, c)) = sub {
        col.rock_sub[C] = s as u8;
        col.contact[C] = c;
    }
    col.regolith[C] = regolith_c;
    col
}

fn run(field: &mut [f32], col: &mut GeologyColumn, k: f64, iters: i32, contrast: f64) {
    let zeros = [0f32; 9];
    let mut rock = StreamPowerRock { column: col, contrast, r_expose: 0.01 };
    stream_power_kernel_rock(field, &zeros, &zeros, 3, 3, &params(k, iters), &mut rock);
}

fn centre_peak() -> Vec<f32> {
    let mut f = vec![0f32; 9];
    f[C] = 0.5;
    f
}

fn close(a: f64, b: f64, what: &str) {
    assert!((a - b).abs() < 1e-6, "{what}: got {a}, want {b}");
}

/// Protects the κ lookup (§2.3's κ column raised to the contrast). Literals:
/// √0.30, √2.5, 2.0¹, 4.0¹, and x⁰ = 1 for every rock.
#[test]
fn kappa_multiplier_is_kappa_to_the_contrast() {
    close(kappa_multiplier(Rock::Granite, 0.5), 0.547_722_6, "granite, c = 0.5");
    close(kappa_multiplier(Rock::Shale, 0.5), 1.581_138_8, "shale, c = 0.5");
    close(kappa_multiplier(Rock::Tuff, 1.0), 2.0, "tuff, c = 1");
    close(kappa_multiplier(Rock::Unconsolidated, 1.0), 4.0, "unconsolidated, c = 1");
    close(kappa_multiplier(Rock::Gneiss, 1.0), 0.35, "gneiss, c = 1");
    for r in Rock::ALL {
        assert_eq!(kappa_multiplier(r, 0.0), 1.0, "{r:?}: c = 0 must be uniform rock");
    }
}

/// Protects the rock actually reaching the coefficient: one iteration on
/// granite (κ = 0.3) and on shale (κ = 2.5), contrast 1, K = 0.1.
/// Granite: 0.5 / 1.03 = 0.485 436 9; shale: 0.5 / 1.25 = 0.4.
#[test]
fn one_iteration_uses_the_exposed_rocks_kappa() {
    let mut f = centre_peak();
    run(&mut f, &mut column(Rock::Granite, None, 0.0), 0.1, 1, 1.0);
    close(f[C] as f64, 0.485_436_9, "granite");
    let mut f = centre_peak();
    run(&mut f, &mut column(Rock::Shale, None, 0.0), 0.1, 1, 1.0);
    close(f[C] as f64, 0.4, "shale");
    // The seeds never move.
    assert!(f.iter().enumerate().all(|(i, &v)| i == C || v == 0.0));
}

/// Protects §4.1's contact switch inside the solve. A granite cap over shale
/// with its contact at 0.49: iteration 1 erodes granite (0.5 → 0.485 436 9,
/// now below the contact), so iteration 2 must erode **shale**:
/// 0.485 436 9 / 1.25 = 0.388 349 5. Without the switch it would be
/// 0.485 436 9 / 1.03 = 0.471 297 9; a contact never reached must give that.
#[test]
fn a_breached_cap_switches_to_the_substrate_mid_solve() {
    let mut f = centre_peak();
    let mut col = column(Rock::Granite, Some((Rock::Shale, 0.49)), 0.0);
    run(&mut f, &mut col, 0.1, 2, 1.0);
    close(f[C] as f64, 0.388_349_5, "breached in iteration 1, shale in iteration 2");

    let mut f = centre_peak();
    let mut col = column(Rock::Granite, Some((Rock::Shale, 0.2)), 0.0);
    run(&mut f, &mut col, 0.1, 2, 1.0);
    close(f[C] as f64, 0.471_297_9, "contact never reached: granite both iterations");
    // The kernel moves the contact only with uplift, which is zero here.
    assert_eq!(col.contact[C], 0.2);
}

/// Protects §2.5's exposure rule inside the kernel and §4.9's division of
/// labour. Regolith thicker than `R_EXPOSE` reads unconsolidated (κ = 4):
/// with K = 0.01, 0.5 / 1.04 = 0.480 769 2 -- granite's κ would give
/// 0.5 / 1.003 instead. And the kernel never writes regolith: the caller
/// accounts the call's net change (`account_regolith`, below).
#[test]
fn thick_regolith_reads_unconsolidated_and_the_kernel_leaves_it_alone() {
    let mut f = centre_peak();
    let mut col = column(Rock::Granite, None, 0.02);
    run(&mut f, &mut col, 0.01, 1, 1.0);
    close(f[C] as f64, 0.480_769_2, "unconsolidated kappa 4");
    assert_eq!(col.regolith[C], 0.02, "the kernel must not write regolith (§4.9: the caller does)");
    // Thinner than R_EXPOSE (0.01 here): the bedrock shows through.
    let mut f = centre_peak();
    let mut col = column(Rock::Granite, None, 0.005);
    run(&mut f, &mut col, 0.01, 1, 1.0);
    close(f[C] as f64, 0.498_504_5, "granite under a thin mantle: 0.5 / 1.003");
}

/// Protects the rock path's arithmetic against an oracle it does not
/// compute: the **legacy** kernel. At contrast 0 every κ^c is exactly 1, and
/// with `resist = 0` the legacy factor `max(0.05, 1 − 0)` is exactly 1 too, so
/// the two kernels must agree bit for bit -- with deposition on, over several
/// iterations, on a rough surface with pits. Any drift in how the rock path
/// assembles the coefficient, or any write it makes to the field beyond the
/// legacy update, turns this red.
#[test]
fn at_contrast_zero_the_rock_kernel_is_the_legacy_kernel() {
    let (w, h) = (24usize, 20usize);
    let n = w * h;
    let mut seed = 0x2545_f491_u64;
    let mut rnd = || {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((seed >> 33) as f64 / (1u64 << 31) as f64) as f32
    };
    let pre: Vec<f32> = (0..n).map(|i| 0.4 + 0.3 * ((i % w) as f32 / w as f32) + 0.2 * rnd()).collect();
    let zeros = vec![0f32; n];
    let rain = vec![0.3f32; n];
    let p = StreamPowerParams { k: 0.05, uplift: 0.0, deposit: 0.3, climate_k: 0.5, iters: 5, resist: 0.0, g: 1.0, world: false, sea: 0.2 };
    let mut legacy = pre.clone();
    cartalith_erosion::stream_power_kernel(&mut legacy, &zeros, &zeros, &rain, w, h, &p);
    let mut col = GeologyColumn {
        rock_top: vec![Rock::Shale as u8; n],
        rock_sub: vec![NO_LAYER; n],
        contact: vec![f32::NAN; n],
        regolith: vec![0.0; n],
        volcanic_setting: vec![0; n],
    };
    let mut f = pre.clone();
    stream_power_kernel_rock(&mut f, &zeros, &rain, w, h, &p, &mut StreamPowerRock { column: &mut col, contrast: 0.0, r_expose: 0.01 });
    assert_ne!(legacy, pre, "the fixture must erode, or the comparison proves nothing");
    assert_eq!(f, legacy, "contrast 0 must reproduce the legacy kernel at resist 0 bit for bit");
}

/// Protects §4.2: the contact rises by exactly what rebound added (0.02 here),
/// a single-layer cell's NaN contact stays NaN, and regolith is untouched.
#[test]
fn rebound_lifts_the_contact_by_its_own_increment() {
    let mut col = column(Rock::Granite, Some((Rock::Shale, 0.4)), 0.003);
    let before = vec![0.5f32; 9];
    let mut after = before.clone();
    after[C] = 0.52;
    after[0] = 0.51;
    lift_column(&mut col, &before, &after);
    close(col.contact[C] as f64, 0.42, "contact + rebound increment");
    assert!(col.contact[0].is_nan(), "a single-layer contact must stay NaN, never a plausible elevation");
    assert_eq!(col.regolith[C], 0.003, "rebound does not change regolith thickness");
}

/// Protects `account_regolith`'s two modes: a lowering always strips (and
/// floors at 0); a rise is deposit only when the caller says so.
#[test]
fn account_regolith_strips_and_optionally_deposits() {
    let mut col = column(Rock::Granite, None, 0.01);
    let before = vec![0.5f32; 9];
    let mut after = before.clone();
    after[C] = 0.496; // lowered 0.004
    after[1] = 0.53; // raised 0.03
    account_regolith(&mut col, &before, &after, false);
    close(col.regolith[C] as f64, 0.006, "stripped by the lowering");
    assert_eq!(col.regolith[1], 0.0, "a rise is not deposit unless the caller says so");
    account_regolith(&mut col, &before, &after, true);
    close(col.regolith[C] as f64, 0.002, "stripped again");
    close(col.regolith[1] as f64, 0.03, "a rise from routing is deposit");
    after[C] = 0.4; // lowered 0.1, more than is left
    account_regolith(&mut col, &before, &after, true);
    assert_eq!(col.regolith[C], 0.0, "stripping floors at bare rock");
}
