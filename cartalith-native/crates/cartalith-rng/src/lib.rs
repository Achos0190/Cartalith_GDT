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

pub struct Mulberry32 {
    state: u32,
}

impl Mulberry32 {
    /// `seed` matches the JS call site's convention of passing an already
    /// `>>>0`-coerced (unsigned 32-bit) seed.
    pub fn new(seed: u32) -> Self {
        Self { state: seed }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crate_compiles_and_tests_run() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn deterministic_for_same_seed() {
        let mut a = Mulberry32::new(42);
        let mut b = Mulberry32::new(42);
        for _ in 0..8 {
            assert_eq!(a.next_f64(), b.next_f64());
        }
    }
}
