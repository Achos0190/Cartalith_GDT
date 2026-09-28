//! The stream-power routing refresh (`GEOLOGY_FIRST_SCOPE.md` §5.11): the
//! `_refreshed` kernels rebuild the fill, receivers, drainage area and `Cc`
//! from the current surface every `k` iterations instead of once per call.
//!
//! **What these tests protect.** The oracle is independent of the refresh
//! code: with `uplift = 0` the kernel's uplift field is zero whatever the
//! surface, and on a fixture that stays inside `[0, 1]` the final clamp is a
//! no-op, so refreshing every `k` iterations of an `n`-iteration call must
//! equal `n / k` back-to-back calls of the **frozen**, golden-verified kernel
//! (`golden_parity_streampower.rs`) of `k` iterations each -- bit for bit.
//! And a refresh interval at or beyond the call's iteration count must be the
//! frozen kernel itself, which is what keeps every existing caller unmoved.
//!
//! **The fixture.** A 48 x 40 hash-noise surface over a tilted plane, in
//! `(0.2, 0.8)`, with deposition on, so the tree genuinely changes between
//! iterations (asserted, not assumed: [`the_fixture_is_one_the_refresh_changes`]).

use std::num::NonZeroUsize;

use cartalith_erosion::{
    stream_power_kernel, stream_power_kernel_refreshed, stream_power_kernel_rock, stream_power_kernel_rock_refreshed,
    StreamPowerParams, StreamPowerRock,
};
use cartalith_terrain::geology::{GeologyColumn, Rock, NO_LAYER};

const W: usize = 48;
const H: usize = 40;

/// Shorthand for a `NonZeroUsize` refresh interval; panics if `k == 0`, which
/// every call site in this file avoids by construction.
fn nz(k: usize) -> NonZeroUsize {
    NonZeroUsize::new(k).expect("test intervals are positive")
}

/// Deterministic hash noise in `[0, 1)`: no RNG crate, so the fixture cannot
/// drift with a dependency.
fn hash01(i: usize, salt: u32) -> f32 {
    let mut x = (i as u32).wrapping_mul(0x9E37_79B9) ^ salt.wrapping_mul(0x85EB_CA6B);
    x ^= x >> 16;
    x = x.wrapping_mul(0x7FEB_352D);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846C_A68B);
    x ^= x >> 16;
    (x >> 8) as f32 / (1u32 << 24) as f32
}

/// The shared 48x40 fixture: a tilted plane (rising in x and, more gently,
/// in y) plus deterministic hash noise, kept inside `(0.2, 0.8)` so the
/// kernel's own `[0, 1]` clamp never fires (module header's oracle premise).
fn surface() -> Vec<f32> {
    (0..W * H)
        .map(|i| {
            let (x, y) = ((i % W) as f32, (i / W) as f32);
            0.25 + 0.3 * (x / W as f32) + 0.1 * (y / H as f32) + 0.12 * hash01(i, 7)
        })
        .collect()
}

/// Deterministic hash-noise rainfall, independent of [`surface`]'s salt so
/// the two fields are not correlated.
fn rain() -> Vec<f32> {
    (0..W * H).map(|i| 0.5 + 0.5 * hash01(i, 11)).collect()
}

/// The shared `StreamPowerParams` fixture at a given iteration count.
/// `uplift = 0` is the oracle's premise (module header): the uplift term
/// is then identically zero on every cell, so it cannot mask a routing
/// difference between the frozen and refreshed kernels.
fn params(iters: i32) -> StreamPowerParams {
    // The app's own stream constants (k 0.012, deposit 0.3, climate_k 0.5,
    // `cartalith_engine::WorldParams::defaults()`), with a K large enough to
    // move this small fixture visibly, and uplift 0 (the oracle's premise).
    StreamPowerParams { k: 0.3, uplift: 0.0, deposit: 0.3, climate_k: 0.5, iters, resist: 0.5, g: 1.0, world: false, sea: 0.2 }
}

/// Bit patterns of an `f32` slice, so `assert_eq!` compares exactly
/// (including distinguishing `-0.0` from `0.0`) rather than through
/// `PartialEq`'s NaN-unfriendly equality.
fn bits(v: &[f32]) -> Vec<u32> {
    v.iter().map(|x| x.to_bits()).collect()
}

/// Runs the frozen, golden-verified `stream_power_kernel` — routing built
/// once per call, exactly as every existing caller uses it.
fn frozen(f: &mut [f32], iters: i32) {
    let zeros = vec![0f32; W * H];
    let resist: Vec<f32> = (0..W * H).map(|i| hash01(i, 3)).collect();
    stream_power_kernel(f, &zeros, &resist, &rain(), W, H, &params(iters));
}

/// Runs `stream_power_kernel_refreshed`, rebuilding fill/receivers/drainage
/// area/`Cc` every `every` iterations instead of once per call.
fn refreshed(f: &mut [f32], iters: i32, every: usize) {
    let zeros = vec![0f32; W * H];
    let resist: Vec<f32> = (0..W * H).map(|i| hash01(i, 3)).collect();
    stream_power_kernel_refreshed(f, &zeros, &resist, &rain(), W, H, &params(iters), nz(every));
}

/// Interior cells strictly lower than all eight neighbours and
/// above `sea`: closed pits, the cells a later fill turns into 1-cell lakes.
fn pits(f: &[f32], sea: f32) -> usize {
    let mut n = 0;
    for y in 1..H - 1 {
        for x in 1..W - 1 {
            let i = y * W + x;
            if f[i] <= sea {
                continue;
            }
            let lowest = (-1i64..=1)
                .flat_map(|dy| (-1i64..=1).map(move |dx| (dx, dy)))
                .filter(|&(dx, dy)| dx != 0 || dy != 0)
                .all(|(dx, dy)| f[((y as i64 + dy) as usize) * W + (x as i64 + dx) as usize] > f[i]);
            n += lowest as usize;
        }
    }
    n
}

/// Protects the oracle's premise: on this fixture the refresh is not a no-op,
/// and the surface stays inside `[0, 1]` (so the clamp between chained calls
/// changes nothing). Without it, the chained-call equalities below could pass
/// on a fixture where routing never changes, and prove nothing.
#[test]
fn the_fixture_is_one_the_refresh_changes() {
    // Protects: the oracle's own premise -- the fixture actually moves under
    // refreshing (not a no-op fixture that would pass vacuously) and stays
    // inside [0, 1] throughout, so the final clamp cannot be masking a
    // difference the chained-call equalities below would otherwise miss.
    let (mut a, mut b) = (surface(), surface());
    frozen(&mut a, 12);
    refreshed(&mut b, 12, 1);
    let differ = a.iter().zip(&b).filter(|(x, y)| x.to_bits() != y.to_bits()).count();
    assert!(differ > 100, "refreshing every iteration moved only {differ} cells of {}", W * H);
    assert!(a.iter().chain(&b).all(|v| (0.0..=1.0).contains(v)), "the fixture left [0, 1], so the clamp is not a no-op");
}

/// Protects the refresh schedule: rebuilding before iterations `k, 2k, ...`
/// and only then. Oracle: `n / k` back-to-back frozen calls of `k` iterations
/// (the frozen kernel routes once per call, so a call boundary *is* a
/// refresh). Checked for `k` = 1, 2, 3 and 4 over 12 iterations.
#[test]
fn refresh_every_k_equals_back_to_back_frozen_calls_of_k() {
    // Protects: stream_power_kernel_refreshed's refresh schedule -- a
    // refresh every k iterations of one n-iteration call must equal n/k
    // back-to-back frozen calls of k iterations each, bit for bit, checked
    // at k = 1, 2, 3 and 4 over 12 iterations.
    for k in [1usize, 2, 3, 4] {
        let mut chained = surface();
        for _ in 0..12 / k {
            frozen(&mut chained, k as i32);
        }
        let mut r = surface();
        refreshed(&mut r, 12, k);
        assert_eq!(bits(&r), bits(&chained), "refresh every {k} diverged from {} frozen calls of {k}", 12 / k);
    }
}

/// Protects the gate every existing caller relies on: an interval at or
/// beyond the iteration count never refreshes, so the kernel is the frozen
/// one bit for bit (the app's 9-iteration light pass is the case that
/// matters: `refresh_every >= 9` there is today's world).
#[test]
fn an_interval_at_or_beyond_the_iteration_count_is_the_frozen_kernel() {
    // Protects: every existing caller's assumption -- a refresh interval at
    // or beyond the call's own iteration count never fires, so the refreshed
    // kernel is bit-for-bit the frozen one (the gate the app's 9-iteration
    // light pass relies on).
    let mut want = surface();
    frozen(&mut want, 9);
    for k in [9usize, 10, 1000] {
        let mut r = surface();
        refreshed(&mut r, 9, k);
        assert_eq!(bits(&r), bits(&want), "refresh every {k} of 9 iterations is not the frozen kernel");
    }
}

/// Protects the effect the refresh exists for (§5.10's diagnosis): on a long
/// run the frozen tree leaves closed pits that the refreshed one does not.
/// Literal counts, measured on this fixture at 36 iterations (τ = 4's light
/// pass length), so a change in either kernel shows up as a changed count.
#[test]
fn a_long_run_leaves_fewer_pits_when_the_routing_is_refreshed() {
    // Protects: the effect the refresh exists for (§5.10) -- over a long run
    // the refreshed kernel leaves strictly fewer closed pits than the frozen
    // one, against literal pit counts measured on this fixture so a change
    // in either kernel's behaviour shows up as a changed count, not just a
    // direction.
    let start = surface();
    let mut a = start.clone();
    frozen(&mut a, 36);
    let mut b = start.clone();
    refreshed(&mut b, 36, 1);
    let (pa, pb) = (pits(&a, 0.2), pits(&b, 0.2));
    assert_eq!((pits(&start, 0.2), pa, pb), (PITS_START, PITS_FROZEN, PITS_REFRESHED));
    assert!(pb < pa, "refreshing did not reduce the pits: frozen {pa}, refreshed {pb}");
}

// Measured on the fixture above, 2026-09-28 (the first run of this test):
// the noise leaves 182 single-cell pits; 36 frozen iterations drain all but
// 34; refreshing every iteration leaves 31. A small effect on a 48 x 40
// fixture, but a mutation that stops the refresh firing reads 34 here.
const PITS_START: usize = 182;
const PITS_FROZEN: usize = 34;
const PITS_REFRESHED: usize = 31;

/// Granite everywhere except the centre third of the columns, which is
/// sandstone over shale with the contact 0.01 below the surface, so the rock
/// path's per-iteration multiplier (and its contact switch) has something to
/// read.
fn column(field: &[f32]) -> GeologyColumn {
    let mut col = GeologyColumn {
        rock_top: vec![Rock::Granite as u8; W * H],
        rock_sub: vec![NO_LAYER; W * H],
        contact: vec![f32::NAN; W * H],
        regolith: vec![0.0; W * H],
        volcanic_setting: vec![0; W * H],
    };
    for i in 0..W * H {
        if (W / 3..2 * W / 3).contains(&(i % W)) {
            col.rock_top[i] = Rock::Sandstone as u8;
            col.rock_sub[i] = Rock::Shale as u8;
            col.contact[i] = field[i] - 0.01;
        }
    }
    col
}

/// Protects the rock entry point's refresh the same way: every-`k` equals
/// back-to-back frozen rock calls of `k` iterations, bit for bit, on the
/// field and on the column (whose contact the kernel may write).
#[test]
fn rock_refresh_every_k_equals_back_to_back_frozen_rock_calls() {
    // Protects: the rock-aware entry point's refresh schedule matches the
    // plain kernel's (previous tests) -- every-k rock calls equal chained
    // frozen rock calls of k iterations, bit for bit, on both the field AND
    // the column's contact (which the rock kernel may write), and the
    // fixture is confirmed non-vacuous (refreshed differs from one
    // 12-iteration frozen rock call).
    let zeros = vec![0f32; W * H];
    let rn = rain();
    for k in [1usize, 3] {
        let mut f1 = surface();
        let mut c1 = column(&f1);
        for _ in 0..12 / k {
            stream_power_kernel_rock(&mut f1, &zeros, &rn, W, H, &params(k as i32), &mut StreamPowerRock { column: &mut c1, contrast: 0.5, r_expose: 0.001 });
        }
        let mut f2 = surface();
        let mut c2 = column(&f2);
        stream_power_kernel_rock_refreshed(&mut f2, &zeros, &rn, W, H, &params(12), &mut StreamPowerRock { column: &mut c2, contrast: 0.5, r_expose: 0.001 }, nz(k));
        assert_eq!(bits(&f2), bits(&f1), "rock refresh every {k} diverged from chained rock calls");
        assert_eq!(bits(&c2.contact), bits(&c1.contact), "rock refresh every {k}: contact diverged");
        let mut f3 = surface();
        let mut c3 = column(&f3);
        stream_power_kernel_rock(&mut f3, &zeros, &rn, W, H, &params(12), &mut StreamPowerRock { column: &mut c3, contrast: 0.5, r_expose: 0.001 });
        assert_ne!(bits(&f2), bits(&f3), "rock refresh every {k} changed nothing on this fixture");
    }
}
