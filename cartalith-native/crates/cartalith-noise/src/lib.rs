//! hash/vnoise/fbm/ridged — hand-ported for parity, not `noise-rs`
//! (`PROVENANCE.md`).
//!
//! Faithful hand-port of the reference HTML's noise section (lines
//! 2292-2295, right after `mulberry32`). `hash` is the one function here
//! that needs real care: its middle step multiplies a 32-bit value by a
//! ~1.27e9 constant using plain JS `*` (not `Math.imul`), so the product
//! can reach ~2^61 — past `f64`'s exact 53-bit integer range. JS keeps
//! this as a rounded `f64` and only wraps to 32 bits (`>>>`) afterward;
//! that rounding is real, observed behaviour baked into every world this
//! engine has ever generated, not a bug to quietly fix
//! (`cartalith-rust-conventions`: match precision, don't improve it).

/// Mirrors ECMAScript's `ToUint32`: truncate toward zero, then wrap to
/// `[0, 2^32)`. Valid for any `x` whose truncated magnitude fits in `i64`
/// (true for every value this module produces — the largest intermediate,
/// `hash`'s prod-before-second-wrap step, tops out around 2^61).
fn to_uint32(x: f64) -> u32 {
    (x as i64) as u32
}

/// `hash(x, y, s)` — reference HTML: `(x|0)*374761393+(y|0)*668265263+(s|0)*362437`
/// then two 32-bit-wrap/xor/multiply rounds, `/4294967295` (note: `2^32-1`,
/// *not* `2^32` like `mulberry32`'s divisor — easy to transpose, verified
/// against golden data).
///
/// One sign subtlety cost a golden-test failure to find: JS's `^` returns
/// a *signed* int32 (`ToInt32`), and that signed value — not its unsigned
/// bit pattern — is what feeds the following plain `*`. `h2_bits as i32`
/// below is that re-interpretation; skipping it silently flips the sign
/// of roughly half of all outputs.
pub fn hash(x: i32, y: i32, s: i32) -> f64 {
    let h0 = (x as f64) * 374761393.0 + (y as f64) * 668265263.0 + (s as f64) * 362437.0;
    let h1 = to_uint32(h0);
    let h2_bits = h1 ^ (h1 >> 13);
    let h3 = (h2_bits as i32 as f64) * 1274126177.0;
    let h4 = to_uint32(h3);
    let h5 = h4 ^ (h4 >> 16);
    (h5 as f64) / 4294967295.0
}

/// `vnoise(x, y, s)` — bilinear value noise over `hash`'s lattice, with
/// the standard 3t²-2t³ smoothstep.
pub fn vnoise(x: f64, y: f64, s: i32) -> f64 {
    let xi = x.floor();
    let yi = y.floor();
    let xf = x - xi;
    let yf = y - yi;
    let u = xf * xf * (3.0 - 2.0 * xf);
    let v = yf * yf * (3.0 - 2.0 * yf);
    let xi = xi as i32;
    let yi = yi as i32;
    let a = hash(xi, yi, s);
    let b = hash(xi + 1, yi, s);
    let c = hash(xi, yi + 1, s);
    let d = hash(xi + 1, yi + 1, s);
    a * (1.0 - u) * (1.0 - v) + b * u * (1.0 - v) + c * (1.0 - u) * v + d * u * v
}

/// `fbm(x, y, s)` — 6-octave fractal Brownian motion over `vnoise`.
pub fn fbm(x: f64, y: f64, s: i32) -> f64 {
    let mut amp = 0.5;
    let mut freq = 1.0;
    let mut sum = 0.0;
    let mut nrm = 0.0;
    for o in 0..6 {
        sum += amp * vnoise(x * freq, y * freq, s + o * 131);
        nrm += amp;
        amp *= 0.5;
        freq *= 2.0;
    }
    sum / nrm
}

/// `ridged(x, y, s)` — 6-octave ridged multifractal over `vnoise`.
pub fn ridged(x: f64, y: f64, s: i32) -> f64 {
    let mut amp = 0.5;
    let mut freq = 1.0;
    let mut sum = 0.0;
    let mut nrm = 0.0;
    for o in 0..6 {
        let n = vnoise(x * freq, y * freq, s + o * 131);
        let n = 1.0 - (2.0 * n - 1.0).abs();
        sum += amp * n * n;
        nrm += amp;
        amp *= 0.5;
        freq *= 2.0;
    }
    sum / nrm
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crate_compiles_and_tests_run() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn deterministic_for_same_input() {
        assert_eq!(hash(3, -7, 42), hash(3, -7, 42));
        assert_eq!(fbm(1.5, -2.5, 42), fbm(1.5, -2.5, 42));
    }
}
