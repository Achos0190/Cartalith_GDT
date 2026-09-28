//! mulberry32, ported exactly (PARITY_TESTING.md)
//!
//! Faithful hand-port of the JS engine's `mulberry32` (reference HTML,
//! `<script>` block, `/* ===================== noise ===================== */`
//! section) — every seeded decision in the engine derives from this, so a
//! different PRNG (even a "better" one) would make every downstream
//! comparison fail for reasons unrelated to correctness. See
//! `PARITY_TESTING.md`: port and test this alone before anything depends
//! on it.
//!
//! JS source:
//! ```js
//! function mulberry32(a){ return function(){ a|=0; a=a+0x6D2B79F5|0; let t=Math.imul(a^a>>>15,1|a); t=t+Math.imul(t^t>>>7,61|t)^t; return ((t^t>>>14)>>>0)/4294967296; }; }
//! ```
//!
//! `Math.imul` is a 32-bit wrapping multiply; JS's `+`/`^`/`>>>` all operate
//! on the same 32-bit representation regardless of signedness (XOR and
//! wrapping multiplication are bit-identical whether the operand is
//! interpreted as `i32` or `u32`), so this ports directly onto `u32` with
//! `wrapping_add`/`wrapping_mul` — no signed/unsigned split needed. Divides
//! by `2^32` exactly (a `u32` cast to `f64` and division by a power of two
//! are both lossless), so results are bit-identical to the JS `Number`
//! output, not merely close.

/// The reference engine's `mulberry32` PRNG, carried as one `u32` word of
/// state. Every seeded decision downstream in the engine (tectonic seeds,
/// crater placement, volcanism, manual-placement names) derives from this
/// generator, so it must reproduce the JS closure's output bit-for-bit — see
/// the module doc above for why a different (even "better") PRNG would be
/// wrong here. Must never be swapped for `rand`'s `SmallRng` or any other
/// generator: golden tests in `tests/golden_parity.rs` pin this exact stream.
pub struct Mulberry32 {
    state: u32,
}

impl Mulberry32 {
    /// `seed` matches the JS call site's convention of passing an already
    /// `>>>0`-coerced (unsigned 32-bit) seed.
    pub fn new(seed: u32) -> Self {
        Self { state: seed }
    }

    /// The generator's whole position. `Mulberry32::new(rng.state())`
    /// continues the stream exactly where `rng` stands, so a stream that must
    /// outlive a session (a saved project's manual-placement names) can be
    /// stored as this one number.
    pub fn state(&self) -> u32 {
        self.state
    }

    /// One step of the generator, returning the same `[0, 1)` value the JS
    /// closure's own call would return.
    pub fn next_f64(&mut self) -> f64 {
        self.state = self.state.wrapping_add(0x6D2B79F5);
        let mut t = self.state;
        t = (t ^ (t >> 15)).wrapping_mul(t | 1);
        t = t.wrapping_add((t ^ (t >> 7)).wrapping_mul(t | 61)) ^ t;
        ((t ^ (t >> 14)) as f64) / 4294967296.0
    }
}

/// Unit tests for `Mulberry32`'s Rust-only behaviour (state save/restore,
/// determinism). Bit-for-bit agreement with the JS engine's own output is
/// tested separately in `tests/golden_parity.rs`, not here.
#[cfg(test)]
mod tests {
    use super::*;

    /// Sanity check that the crate builds and its test harness runs at all.
    #[test]
    fn crate_compiles_and_tests_run() {
        // Protects: nothing behavioural -- a canary that fails loudly if the
        // crate or its test harness stops compiling/running altogether.
        assert_eq!(2 + 2, 4);
    }

    /// A stream reconstructed from a saved `state()` word continues exactly
    /// where the original left off.
    #[test]
    fn a_stream_rebuilt_from_its_state_continues_it() {
        // Protects: `state()`/`new()` round-tripping the generator's position
        // exactly, which manual-placement name persistence across a saved
        // session depends on -- a drift here would desync a resumed stream
        // from the one that would have run uninterrupted.
        let mut a = Mulberry32::new(42);
        a.next_f64();
        a.next_f64();
        let mut b = Mulberry32::new(a.state());
        for _ in 0..8 {
            assert_eq!(a.next_f64(), b.next_f64());
        }
        // 42 + 10 * 0x6D2B79F5, wrapped: the position after ten draws.
        assert_eq!(a.state(), 1_135_788_988);
    }

    /// Two generators built from the same seed produce identical streams.
    #[test]
    fn deterministic_for_same_seed() {
        // Protects: two generators built from the same seed against any
        // hidden non-determinism (e.g. an accidental read of ambient state),
        // which the whole port's parity story depends on.
        let mut a = Mulberry32::new(42);
        let mut b = Mulberry32::new(42);
        for _ in 0..8 {
            assert_eq!(a.next_f64(), b.next_f64());
        }
    }
}
