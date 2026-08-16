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

/// `pvnoise(x, y, s, pX)` — `vnoise`'s world-wrap sibling: the x lattice
/// coordinate wraps mod `pX` so the noise tiles exactly on a cylinder
/// (used when `state.world` is set).
pub fn pvnoise(x: f64, y: f64, s: i32, p_x: i32) -> f64 {
    let xi = x.floor();
    let yi = y.floor();
    let xf = x - xi;
    let yf = y - yi;
    let u = xf * xf * (3.0 - 2.0 * xf);
    let v = yf * yf * (3.0 - 2.0 * yf);
    let xi = xi as i32;
    let yi = yi as i32;
    // JS `((xi%pX)+pX)%pX` — Euclidean mod, since JS `%` keeps the
    // dividend's sign (can be negative) unlike Rust's `rem_euclid` default.
    let px = ((xi % p_x) + p_x) % p_x;
    let px1 = (px + 1) % p_x;
    let a = hash(px, yi, s);
    let b = hash(px1, yi, s);
    let c = hash(px, yi + 1, s);
    let d = hash(px1, yi + 1, s);
    a * (1.0 - u) * (1.0 - v) + b * u * (1.0 - v) + c * (1.0 - u) * v + d * u * v
}

/// `pfbm(x, y, s, pX)` — `fbm` over `pvnoise`; `pX` doubles each octave
/// alongside frequency, same as the reference.
pub fn pfbm(x: f64, y: f64, s: i32, p_x: i32) -> f64 {
    let mut amp = 0.5;
    let mut freq = 1.0;
    let mut sum = 0.0;
    let mut nrm = 0.0;
    let mut p = p_x.max(2);
    for o in 0..6 {
        sum += amp * pvnoise(x * freq, y * freq, s + o * 131, p);
        nrm += amp;
        amp *= 0.5;
        freq *= 2.0;
        p = (p * 2).max(2);
    }
    sum / nrm
}

/// `pridged(x, y, s, pX)` — `ridged` over `pvnoise`.
pub fn pridged(x: f64, y: f64, s: i32, p_x: i32) -> f64 {
    let mut amp = 0.5;
    let mut freq = 1.0;
    let mut sum = 0.0;
    let mut nrm = 0.0;
    let mut p = p_x.max(2);
    for o in 0..6 {
        let n = pvnoise(x * freq, y * freq, s + o * 131, p);
        let n = 1.0 - (2.0 * n - 1.0).abs();
        sum += amp * n * n;
        nrm += amp;
        amp *= 0.5;
        freq *= 2.0;
        p = (p * 2).max(2);
    }
    sum / nrm
}

// ===================== GPU-safe noise (GPU_LAYER_INTEGRATION_SCOPE.md milestone 1) =====================
//
// `hash`/`vnoise` above are the reference-matching CPU primitives every
// existing golden-parity test in this workspace depends on — untouched by
// what follows. `gpu_hash`/`gpu_vnoise` are a DELIBERATE REDESIGN for the
// GPU path specifically (`DECISIONS.md` §7a: "principled equivalence," not
// bit-parity, for GPU/optimized paths where JS-matching is impractical).
// The GPU-compute pilot (`GPU_COMPUTE_PILOT_SCOPE.md`, `cartalith-gpu`)
// found `hash`'s JS semantics depend on IEEE-754 *double*-precision
// rounding at an intermediate magnitude (~2^61) that `f32` cannot
// represent and that WGSL cannot even attempt (`naga` has no working
// `f64` support on this toolchain) — so no faithful GPU port of `hash`
// exists. This is a fresh function instead, chosen for a property `hash`
// never needed: every operation must be *exactly* representable and
// behave *identically* in both Rust and WGSL.
//
// Construction: single-round PCG3D (Mark Jarzynski & Marc Olano, "Hash
// Functions for GPU Rendering," Journal of Computer Graphics Techniques,
// vol. 9, no. 3, 2020, https://www.jcgt.org/published/0009/03/02/) — a
// hash designed specifically for GPU shaders, using only `u32` multiply/
// add/xor/shift. `u32` wraps on overflow by specification in both Rust
// (`wrapping_mul`/`wrapping_add`, used explicitly below so release builds
// don't panic on overflow) and WGSL (native `u32` arithmetic wraps per
// the WGSL spec) — there is no float-precision regime gap here at all,
// unlike `hash`'s JS-vs-f32 problem, because no float appears until the
// very last step (converting the final `u32` to `[0,1)`).

/// PCG3D (Jarzynski & Olano 2020): mixes three `u32` lanes together.
/// Pure wrapping-`u32` arithmetic — bit-identical in Rust and WGSL by
/// construction, since both specify `u32` overflow behaviour identically.
fn pcg3d(mut v: [u32; 3]) -> [u32; 3] {
    v[0] = v[0].wrapping_mul(1664525).wrapping_add(1013904223);
    v[1] = v[1].wrapping_mul(1664525).wrapping_add(1013904223);
    v[2] = v[2].wrapping_mul(1664525).wrapping_add(1013904223);
    v[0] = v[0].wrapping_add(v[1].wrapping_mul(v[2]));
    v[1] = v[1].wrapping_add(v[2].wrapping_mul(v[0]));
    v[2] = v[2].wrapping_add(v[0].wrapping_mul(v[1]));
    v[0] ^= v[0] >> 16;
    v[1] ^= v[1] >> 16;
    v[2] ^= v[2] >> 16;
    v[0] = v[0].wrapping_add(v[1].wrapping_mul(v[2]));
    v[1] = v[1].wrapping_add(v[2].wrapping_mul(v[0]));
    v[2] = v[2].wrapping_add(v[0].wrapping_mul(v[1]));
    v
}

/// GPU-safe hash: `(x, y, s)` reinterpreted bit-for-bit as `u32` (Rust's
/// `as u32` cast on an `i32` and WGSL's `bitcast<u32>` both do a
/// bit-pattern reinterpret, not a value-preserving conversion — the two
/// are identical by construction), mixed via [`pcg3d`], first lane
/// returned. No operation here can lose precision the way `hash`'s
/// f64-magnitude product could.
pub fn gpu_hash(x: i32, y: i32, s: i32) -> u32 {
    pcg3d([x as u32, y as u32, s as u32])[0]
}

/// `u32 -> f32` is a standard, fully-specified IEEE-754 round-to-nearest
/// conversion in both Rust (`as f32`) and WGSL (`f32(...)`) — unlike the
/// `f32 -> u32` direction the original GPU pilot hit (implementation-
/// defined/saturating for out-of-range values), converting a valid,
/// already-in-range `u32` to `f32` has no platform-dependent behaviour to
/// worry about.
fn gpu_hash_to_unit_f32(h: u32) -> f32 {
    (h as f32) / (u32::MAX as f32)
}

/// GPU-safe value noise: same bilinear-lattice-plus-smoothstep shape as
/// `vnoise`, over [`gpu_hash`] instead of `hash`. Deliberately all-`f32`
/// (not promoted to `f64` mid-computation) so this function's arithmetic
/// matches the GPU shader operation-for-operation, giving the tightest
/// achievable CPU/GPU agreement rather than introducing a second,
/// unrelated precision gap between two "GPU-safe" implementations.
pub fn gpu_vnoise(x: f32, y: f32, s: i32) -> f32 {
    let xi = x.floor();
    let yi = y.floor();
    let xf = x - xi;
    let yf = y - yi;
    let u = xf * xf * (3.0 - 2.0 * xf);
    let v = yf * yf * (3.0 - 2.0 * yf);
    let xii = xi as i32;
    let yii = yi as i32;
    let a = gpu_hash_to_unit_f32(gpu_hash(xii, yii, s));
    let b = gpu_hash_to_unit_f32(gpu_hash(xii + 1, yii, s));
    let c = gpu_hash_to_unit_f32(gpu_hash(xii, yii + 1, s));
    let d = gpu_hash_to_unit_f32(gpu_hash(xii + 1, yii + 1, s));
    a * (1.0 - u) * (1.0 - v) + b * u * (1.0 - v) + c * (1.0 - u) * v + d * u * v
}

/// `GPU_LAYER_INTEGRATION_SCOPE.md` milestone 2: 6-octave fbm over
/// [`gpu_vnoise`], same octave-combining shape as [`fbm`] (amp/freq
/// halving/doubling, `s + o*131` per-octave seed offset) but all-`f32`
/// throughout — mirrors `gpu_vnoise`'s own reasoning: matching this
/// function's arithmetic operation-for-operation against its WGSL
/// counterpart gives the tightest achievable CPU/GPU agreement, rather
/// than promoting to `f64` mid-computation and introducing a second,
/// unrelated precision gap. Not periodic (no `gpu_pfbm` sibling yet) —
/// `compute_warp`'s `world=true`/`pfbm` branch is deliberately out of
/// scope for this milestone, see `GPU_LAYER_INTEGRATION_SCOPE.md`.
pub fn gpu_fbm(x: f32, y: f32, s: i32) -> f32 {
    let mut amp = 0.5f32;
    let mut freq = 1.0f32;
    let mut sum = 0.0f32;
    let mut nrm = 0.0f32;
    for o in 0..6 {
        sum += amp * gpu_vnoise(x * freq, y * freq, s + o * 131);
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

    #[test]
    fn gpu_hash_deterministic_for_same_input() {
        assert_eq!(gpu_hash(3, -7, 42), gpu_hash(3, -7, 42));
        assert_eq!(gpu_vnoise(1.5, -2.5, 42), gpu_vnoise(1.5, -2.5, 42));
    }

    #[test]
    fn gpu_fbm_deterministic_and_in_range() {
        let a = gpu_fbm(1.5, -2.5, 42);
        let b = gpu_fbm(1.5, -2.5, 42);
        assert_eq!(a, b);
        assert!((0.0..=1.0).contains(&a), "gpu_fbm output {a} out of [0,1]");
    }

    #[test]
    fn gpu_hash_output_range() {
        // gpu_hash_to_unit_f32 divides by u32::MAX, so the theoretical
        // range is [0,1] inclusive (not the old hash's near-open [0,1)
        // via /4294967295.0 -- same divisor, same inclusive endpoint).
        for (x, y, s) in [(0, 0, 0), (100, -100, 5), (i32::MAX, i32::MIN, 1), (-1, -1, -1)] {
            let v = gpu_hash_to_unit_f32(gpu_hash(x, y, s));
            assert!((0.0..=1.0).contains(&v), "gpu_hash({x},{y},{s}) -> {v} out of [0,1]");
        }
    }

    #[test]
    fn gpu_hash_differs_from_neighbouring_cells() {
        // Not a statistical test suite -- just a sanity check this isn't
        // degenerate (e.g. accidentally constant, or trivially periodic
        // at small lattice offsets) before it's trusted as a noise source.
        let base = gpu_hash(10, 10, 1);
        let neighbours =
            [gpu_hash(11, 10, 1), gpu_hash(10, 11, 1), gpu_hash(9, 10, 1), gpu_hash(10, 9, 1), gpu_hash(10, 10, 2)];
        for n in neighbours {
            assert_ne!(base, n, "adjacent lattice points/seeds must not collide");
        }
    }

    #[test]
    fn gpu_vnoise_is_continuous_at_lattice_boundaries() {
        // At an exact integer lattice point, gpu_vnoise(x,y,s) must equal
        // gpu_hash_to_unit_f32(gpu_hash(x,y,s)) directly (u=v=0, so the
        // bilinear blend collapses to corner `a` alone) -- catches an
        // inverted or mis-indexed corner before it reaches golden testing.
        let x = 7i32;
        let y = -3i32;
        let s = 99i32;
        let expected = gpu_hash_to_unit_f32(gpu_hash(x, y, s));
        let actual = gpu_vnoise(x as f32, y as f32, s);
        assert_eq!(expected, actual, "vnoise at an exact lattice point must equal that corner's hash exactly");
    }
}
