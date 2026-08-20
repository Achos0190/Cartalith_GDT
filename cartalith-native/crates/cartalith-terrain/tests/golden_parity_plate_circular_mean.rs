//! `JS_SEMANTICS_AUDIT.md` §4.4's `-terrain:372`, closed.
//!
//! `build_plates`' `world` branch takes the circular mean of a plate's member
//! cells' x positions, `atan2(sum sin th, sum cos th)`, and scales it by `gw`.
//! The audit reported the site and deliberately did **not** change it, because
//! at the time only `js_atan2` existed and the divergence enters *upstream* of
//! `atan2`: Rust's `sin`/`cos` produce a different `(sum sin, sum cos)` pair
//! from V8's on 92 of 2 000 synthetic plates before `atan2` is ever called, so
//! swapping in `js_atan2` alone took the final `plate.x` from 98/2000
//! disagreeing to 7/2000 — "an improvement that leaves the site *differently*
//! wrong, which is worse than leaving it alone". Its instruction was to fix all
//! three together in the pass that lands `js_sin`/`js_cos`. This is that pass.
//!
//! # Why a hash rather than rows
//!
//! `cartalith-urban` milestone 6's first mutation sweep left 63 survivors
//! inside hand-picked libm goldens; what killed them was a bulk FNV-1a hash
//! over tens of thousands of results. The same technique is used here, at the
//! level of the whole expression rather than one function: 2 000 synthetic
//! plates, arguments drawn by the reference's own `mulberry32` so both sides
//! provably evaluate the same points, one hash per seed. `golden_parity_plates.rs`
//! cases 2 and 3 are the *feature* goldens for this code path and they pass
//! unmodified; this is the *branch* golden the audit's recommendation #4 asks
//! for, and it is what actually distinguishes the two implementations.
//!
//! The expectations are `node` v24.19.0's own output, from a script that
//! mirrors the loop below statement for statement.

/// `cartalith-rng`'s `Mulberry32`, which is the reference's own `mulberry32`.
/// Inlined so this file's argument sequence is visibly the same one the Node
/// script drew.
struct Mulberry32(u32);

impl Mulberry32 {
    fn next_f64(&mut self) -> f64 {
        self.0 = self.0.wrapping_add(0x6D2B79F5);
        let mut t = self.0;
        t = (t ^ (t >> 15)).wrapping_mul(t | 1);
        t = t.wrapping_add((t ^ (t >> 7)).wrapping_mul(t | 61)) ^ t;
        ((t ^ (t >> 14)) as f64) / 4294967296.0
    }
}

const GW: usize = 512;
const TAU: f64 = std::f64::consts::PI * 2.0;

/// The circular mean exactly as `build_plates` spells it, parameterised on the
/// three libm calls so the JS-faithful and the Rust-native forms can be run
/// over the identical argument stream.
fn sweep(seed: u32, n: usize, sin: fn(f64) -> f64, cos: fn(f64) -> f64, atan2: fn(f64, f64) -> f64) -> u32 {
    let mut r = Mulberry32(seed);
    let mut h: u32 = 0x811c_9dc5;
    for _ in 0..n {
        let m = 20 + (r.next_f64() * 400.0).floor() as usize;
        let (mut sxs, mut sxc) = (0.0f64, 0.0f64);
        for _ in 0..m {
            let x = (r.next_f64() * GW as f64).floor();
            let th = x / GW as f64 * TAU;
            sxs += sin(th);
            sxc += cos(th);
        }
        let px = (atan2(sxs, sxc) / TAU + 1.0) * GW as f64 % GW as f64;
        for b in px.to_le_bytes() {
            h ^= u32::from(b);
            h = h.wrapping_mul(0x0100_0193);
        }
    }
    h
}

#[test]
fn the_world_wrap_circular_mean_matches_v8_and_rusts_own_libm_does_not() {
    let js = (
        cartalith_jsmath::js_sin as fn(f64) -> f64,
        cartalith_jsmath::js_cos as fn(f64) -> f64,
        cartalith_jsmath::js_atan2 as fn(f64, f64) -> f64,
    );
    let rust = (
        f64::sin as fn(f64) -> f64,
        f64::cos as fn(f64) -> f64,
        (|y: f64, x: f64| y.atan2(x)) as fn(f64, f64) -> f64,
    );
    // `js_atan2` alone -- the partial fix the audit refused, kept here so the
    // "differently wrong" claim is a measurement in this repository rather than
    // a sentence in a document.
    let partial = (
        f64::sin as fn(f64) -> f64,
        f64::cos as fn(f64) -> f64,
        cartalith_jsmath::js_atan2 as fn(f64, f64) -> f64,
    );

    // `node` v24.19.0, from the mirrored script.
    for (seed, want) in [(0x00b1_a7e5_u32, 0xca0b_c8f8_u32), (0x0051_ade7, 0x151a_a3ae)] {
        let got = sweep(seed, 2000, js.0, js.1, js.2);
        assert_eq!(got, want, "seed {seed:#010x}: V8 hash over 2000 plates");

        let native = sweep(seed, 2000, rust.0, rust.1, rust.2);
        assert_ne!(native, want, "seed {seed:#010x}: this row exists to discriminate");

        let half = sweep(seed, 2000, partial.0, partial.1, partial.2);
        assert_ne!(half, want, "seed {seed:#010x}: js_atan2 alone does not reach V8");
        assert_ne!(half, native, "seed {seed:#010x}: ...and it is not a no-op either");
    }
}

/// The per-plate counts behind the hash, so a failure says *how far* rather
/// than only *that*. These are the shape of the audit's own 92/2000 and
/// 98/2000 figures, re-measured here.
#[test]
fn the_partial_fix_is_an_improvement_that_still_leaves_the_site_wrong() {
    let mut n = 0usize;
    let (mut native_bad, mut partial_bad, mut pair_bad) = (0usize, 0usize, 0usize);
    let mut r_js = Mulberry32(0x00b1_a7e5);
    let mut r_rs = Mulberry32(0x00b1_a7e5);
    for _ in 0..2000 {
        let m = 20 + (r_js.next_f64() * 400.0).floor() as usize;
        let _ = r_rs.next_f64();
        let (mut js_s, mut js_c, mut rs_s, mut rs_c) = (0.0f64, 0.0, 0.0, 0.0);
        for _ in 0..m {
            let u = r_js.next_f64();
            let _ = r_rs.next_f64();
            let th = (u * GW as f64).floor() / GW as f64 * TAU;
            js_s += cartalith_jsmath::js_sin(th);
            js_c += cartalith_jsmath::js_cos(th);
            rs_s += th.sin();
            rs_c += th.cos();
        }
        if (js_s, js_c) != (rs_s, rs_c) {
            pair_bad += 1;
        }
        let want = (cartalith_jsmath::js_atan2(js_s, js_c) / TAU + 1.0) * GW as f64 % GW as f64;
        if (rs_s.atan2(rs_c) / TAU + 1.0) * GW as f64 % GW as f64 != want {
            native_bad += 1;
        }
        if (cartalith_jsmath::js_atan2(rs_s, rs_c) / TAU + 1.0) * GW as f64 % GW as f64 != want {
            partial_bad += 1;
        }
        n += 1;
    }
    assert_eq!(n, 2000);
    // The audit's finding: the divergence is already present in the summed
    // pair, before atan2 is reached.
    assert!(pair_bad > 0, "the (sum sin, sum cos) pair must already differ");
    // ...so js_atan2 alone helps and does not finish the job.
    assert!(partial_bad > 0, "js_atan2 alone still disagrees with V8 on some plates");
    assert!(
        partial_bad < native_bad,
        "js_atan2 alone should improve on Rust's own libm: {partial_bad} vs {native_bad}"
    );
    eprintln!("pair {pair_bad}/2000, native {native_bad}/2000, js_atan2-only {partial_bad}/2000");
}
