//! The vector / polygon geometry kernel — reference lines 28286-28362
//! (`V`, `polyArea` … `clipConvex`) plus `convexHull` (line 29639).
//!
//! Every function here is reachable from the reference's own `UME._test`
//! export, which is what makes this milestone golden-verifiable rather than
//! hand-checked. Ported literally: the operation order inside each expression
//! is the reference's, because these feed `insetPoly`'s area/self-intersection
//! guards and `buildBlocks`' accept/reject thresholds, where a last-bit
//! difference flips a block from kept to dropped.
//!
//! Two behaviours here look like bugs and are not being "fixed" — they are what
//! the reference does, they are pinned by golden tests, and downstream code is
//! tuned around them:
//!
//! - [`clip_convex`] intersects the subject edge against the clip *segment*,
//!   not the clip *line*. A subject vertex outside the clip polygon's extent
//!   therefore produces no crossing point at all, so clipping a shape that
//!   pokes past the clip window's corners can collapse to empty rather than
//!   returning the true intersection. `buildParcels`/`buildBuildings` only ever
//!   clip shapes that already sit inside their block, so it never bites there.
//! - [`inset_poly`] returns `None` — not a degenerate polygon — when the result
//!   has area below 15, or when it self-intersects and has at most 60 vertices.
//!   Those are the reference's own two rejection gates and `buildBlocks` reads
//!   the `None` as "this block cannot be built on".

/// `Math.hypot(x, y)` as V8 actually computes it — **not** `f64::hypot`.
///
/// This is a real, measured discrepancy, not a theoretical one. ECMA-262
/// specifies `Math.hypot` only as "implementation-approximated", and V8
/// implements it by scaling to the largest magnitude and Kahan-summing the
/// squared ratios: `max * sqrt(Σ (vᵢ/max)²)`. Rust's `f64::hypot` is a
/// different, more accurate algorithm. On inputs as ordinary as `(3, 3)` they
/// disagree by one ulp:
///
/// | | value |
/// |---|---|
/// | true 3√2 | 4.242640687119285146… |
/// | Rust `f64::hypot(3,3)` | 4.2426406871192847703 (correctly rounded) |
/// | V8 `Math.hypot(3,3)` | 4.2426406871192856585 (1 ulp high) |
///
/// It surfaced immediately: the first golden run of `dist_pt_seg` failed on
/// exactly that case. Every distance in this engine — `V.len`, `V.dist`,
/// `nearestNode`'s radius test, `attachPoint`'s 11 m snap, `rawEdge`'s 3.5 m
/// minimum — flows through it, and those are threshold comparisons where being
/// *more* accurate than the reference is the wrong answer
/// (`cartalith-rust-conventions`: match the JS engine, do not improve on it).
/// So V8's algorithm is reproduced here rather than delegated.
///
/// The infinity/NaN ordering is the spec's (`Math.hypot(∞, NaN)` is `∞`, not
/// `NaN`), which the scaling loop on its own would get wrong.
pub fn js_hypot(x: f64, y: f64) -> f64 {
    if x.is_infinite() || y.is_infinite() {
        return f64::INFINITY;
    }
    if x.is_nan() || y.is_nan() {
        return f64::NAN;
    }
    let (ax, ay) = (x.abs(), y.abs());
    let max = if ay > ax { ay } else { ax };
    if max == 0.0 {
        return 0.0;
    }
    // Kahan-compensated Σ (vᵢ/max)², in V8's own argument order.
    let mut sum = 0.0f64;
    let mut compensation = 0.0f64;
    for v in [ax, ay] {
        let n = v / max;
        let summand = n * n - compensation;
        let preliminary = sum + summand;
        compensation = (preliminary - sum) - summand;
        sum = preliminary;
    }
    sum.sqrt() * max
}

/// `Math.min(a, b)`, with JS semantics rather than Rust's.
///
/// The difference that matters: **JS propagates NaN, Rust absorbs it.**
/// `Math.min(0.70, NaN)` is `NaN`; `f64::min(0.70, NaN)` is `0.70`.
///
/// **One documented divergence, on signed zero.** `Math.min(+0, -0)` is `-0`
/// and `Math.max(+0, -0)` is `+0`; this returns whichever argument the `<`
/// comparison happens to land on, since `-0.0 < 0.0` is false. Only two of
/// `applyWildness`/`applyPlotChaos`'s eleven clamps have a zero bound
/// (`pierceChance` and `deadEndBias`, both `lo = 0`), and neither can reach a
/// `-0` argument: `0.10 * (2 - w)` is `-0` only if `2 - w` is `-0`, which
/// subtraction of two finite doubles never produces, and
/// `deadEndBias + (w - 1) * 0.15` is `+0` at `w == 1`. So the divergence is
/// unreachable, and handling it would be four lines of code for a case no
/// caller can construct. Recorded, not coded around.
///
/// Milestone 5 gave these a second set of call sites (`buildSite`'s raster
/// index clamps and channel-drift bounds, `terrainSuitability`'s flood score),
/// which is why they live here beside [`js_hypot`] rather than inside
/// `rules` where milestone 4 first needed them.
pub fn js_min(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        f64::NAN
    } else if b < a {
        b
    } else {
        a
    }
}

/// `Math.max(a, b)`, with JS semantics. See [`js_min`] for the NaN rule and the
/// signed-zero divergence.
pub fn js_max(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        f64::NAN
    } else if b > a {
        b
    } else {
        a
    }
}

/// `Math.round(x)`, with JS semantics — **ties go toward +infinity**, not away
/// from zero.
///
/// `f64::round` rounds `-2.5` to `-3`; `Math.round(-2.5)` is `-2`. The obvious
/// JS transliteration `(x + 0.5).floor()` is wrong in the other direction: at
/// `x = 0.49999999999999994` the addition rounds up to exactly `1.0` and the
/// floor gives `1`, where V8 gives `0`. Comparing the fractional part instead
/// gets both cases right.
///
/// The one difference left is `-0`: `Math.round(-0.5)` is `-0` and this returns
/// `+0`. Its only call site is `buildPrimaries`' `to_cell`, which immediately
/// indexes a raster with the result, so the sign of a zero cannot be observed.
pub fn js_round(x: f64) -> f64 {
    if !x.is_finite() {
        return x;
    }
    let f = x.floor();
    if x - f >= 0.5 { f + 1.0 } else { f }
}

/// `Math.exp(x)`, with **V8's** result rather than the platform libm's.
///
/// The same finding as [`js_hypot`], arriving at milestone 5 and measured the
/// same way. Every height, every slope falloff and every log-normal draw in
/// this subsystem runs through `Math.exp`, and Rust's `f64::exp` delegates to
/// the platform's libm, which is **not** the function V8 calls:
///
/// | | disagreements with V8 |
/// |---|---|
/// | `f64::exp` (MSVC CRT) | **20,721 of 240,000** random arguments, all by one ulp |
/// | this function | **0 of 240,000** |
///
/// V8 calls `base::ieee754::exp`, which is FDLIBM's `__ieee754_exp` — argument
/// reduction to `[-0.5 ln2, 0.5 ln2]`, a degree-5 polynomial for the
/// `x - x*c` correction, and a `2^k` scale — transliterated below with its
/// integer bit twiddling intact. It is *less* accurate than a good modern libm
/// (it promises under one ulp, not correct rounding), and that is exactly the
/// point: `cartalith-rust-conventions`' rule is to match the JS engine, not to
/// improve on it. Milestone 5's very first golden run failed on a one-ulp
/// `exp`, the same way milestone 1's first `dist_pt_seg` run failed on a
/// one-ulp `hypot`.
///
/// **One measured special case.** Across 244,000 arguments — 240,000 random,
/// plus every half- and quarter-integer to +-20, plus `1.0` at +-1 and +-2 ulp
/// — V8 and FDLIBM agree everywhere except at exactly `x == 1.0`, where V8
/// returns the correctly-rounded `e` and FDLIBM returns one ulp above it.
/// Reproduced here because it was measured, not because its cause is known. It
/// is unreachable from the site model, whose `exp` arguments are all
/// `-(d^2)/(2*sigma^2)` and therefore never positive.
// The constants below are FDLIBM's own source text, quoted digit for digit so
// this can be diffed against `e_exp.c` by eye. Clippy would have them shortened
// to the same doubles written differently, and `INVLN2` replaced by
// `f64::consts::LOG2_E` -- both are the same value and both would destroy that
// property, which is the only defence this function has against a silent edit.
#[allow(clippy::excessive_precision, clippy::approx_constant)]
pub fn js_exp(x: f64) -> f64 {
    const O_THRESHOLD: f64 = 7.09782712893383973096e+02;
    const U_THRESHOLD: f64 = -7.45133219101941108420e+02;
    const LN2HI: [f64; 2] = [6.93147180369123816490e-01, -6.93147180369123816490e-01];
    const LN2LO: [f64; 2] = [1.90821492927058770002e-10, -1.90821492927058770002e-10];
    const INVLN2: f64 = 1.44269504088896338700e+00;
    const HALF: [f64; 2] = [0.5, -0.5];
    const P1: f64 = 1.66666666666666019037e-01;
    const P2: f64 = -2.77777777770155933842e-03;
    const P3: f64 = 6.61375632143793436117e-05;
    const P4: f64 = -1.65339022054652515390e-06;
    const P5: f64 = 4.13813679705723846039e-08;
    const TWOM1000: f64 = 9.33263618503218878990e-302;
    const HUGE: f64 = 1.0e300;

    // The measured divergence above. See the doc comment.
    if x == 1.0 {
        return std::f64::consts::E;
    }

    let bits = x.to_bits();
    let hx0 = (bits >> 32) as u32;
    let xsb = ((hx0 >> 31) & 1) as usize; // sign bit of x
    let hx = hx0 & 0x7fff_ffff; // high word of |x|

    let mut k: i32 = 0;
    let mut hi = 0.0f64;
    let mut lo = 0.0f64;
    let mut x = x;

    // filter out non-finite argument
    if hx >= 0x4086_2e42 {
        // |x| >= 709.78...
        if hx >= 0x7ff0_0000 {
            let lx = bits as u32;
            if ((hx & 0x000f_ffff) | lx) != 0 {
                return x + x; // NaN
            }
            return if xsb == 0 { x } else { 0.0 }; // exp(+-inf) = {inf, 0}
        }
        if x > O_THRESHOLD {
            return HUGE * HUGE; // overflow
        }
        if x < U_THRESHOLD {
            return TWOM1000 * TWOM1000; // underflow
        }
    }

    // argument reduction
    if hx > 0x3fd6_2e42 {
        // |x| > 0.5 ln2
        if hx < 0x3ff0_a2b2 {
            // ... and |x| < 1.5 ln2
            hi = x - LN2HI[xsb];
            lo = LN2LO[xsb];
            k = 1 - xsb as i32 - xsb as i32;
        } else {
            k = (INVLN2 * x + HALF[xsb]) as i32;
            let t = k as f64;
            hi = x - t * LN2HI[0]; // t*ln2HI is exact here
            lo = t * LN2LO[0];
        }
        x = hi - lo;
    } else if hx < 0x3e30_0000 {
        // |x| < 2^-28
        if HUGE + x > 1.0 {
            return 1.0 + x; // trigger inexact
        }
    } else {
        k = 0;
    }

    // x is now in the primary range
    let t = x * x;
    let twopk = if k >= -1021 {
        f64::from_bits(((0x3ff + k) as u64) << 52)
    } else {
        f64::from_bits(((0x3ff + k + 1000) as u64) << 52)
    };
    let c = x - t * (P1 + t * (P2 + t * (P3 + t * (P4 + t * P5))));
    if k == 0 {
        return 1.0 - ((x * c) / (c - 2.0) - x);
    }
    let y = 1.0 - ((lo - (x * c) / (2.0 - c)) - hi);
    if k >= -1021 {
        if k == 1024 {
            // the one k whose 2^k word would overflow to infinity; scale twice
            return y * 2.0 * f64::from_bits(0x7fe0_0000_0000_0000);
        }
        y * twopk
    } else {
        y * twopk * TWOM1000
    }
}

/// The two-word view of a `f64` FDLIBM is written against. Kept as three tiny
/// helpers rather than inlined, because the ported code below reads as the C it
/// is checked against only if `GET_HIGH_WORD` / `SET_HIGH_WORD` survive as
/// names.
fn hi_word(x: f64) -> u32 {
    (x.to_bits() >> 32) as u32
}

fn lo_word(x: f64) -> u32 {
    x.to_bits() as u32
}

fn from_words(h: u32, l: u32) -> f64 {
    f64::from_bits((u64::from(h) << 32) | u64::from(l))
}

/// FDLIBM `__kernel_sin(x, y, iy)` — `sin(x + y)` for `|x| <= pi/4`, with `y`
/// the tail of the argument reduction and `iy` flagging whether `y` is
/// significant. Not public: it is only correct inside that interval.
#[allow(clippy::excessive_precision)]
fn kernel_sin(x: f64, y: f64, iy: i32) -> f64 {
    const S1: f64 = -1.66666666666666324348e-01;
    const S2: f64 = 8.33333333332248946124e-03;
    const S3: f64 = -1.98412698298579493134e-04;
    const S4: f64 = 2.75573137070700676789e-06;
    const S5: f64 = -2.50507602534068634195e-08;
    const S6: f64 = 1.58969099521155010221e-10;

    let ix = hi_word(x) & 0x7fff_ffff;
    // |x| < 2^-27: sin(x) == x to double precision. The C is `if((int)x==0)`,
    // a truncation, and it is written that way here for the same reason.
    if ix < 0x3e40_0000 && (x as i32) == 0 {
        return x;
    }
    let z = x * x;
    let v = z * x;
    let r = S2 + z * (S3 + z * (S4 + z * (S5 + z * S6)));
    if iy == 0 {
        x + v * (S1 + z * r)
    } else {
        x - ((z * (0.5 * y - v * r) - y) - v * S1)
    }
}

/// FDLIBM `__kernel_cos(x, y)` — `cos(x + y)` for `|x| <= pi/4`.
///
/// The `qx` split in the second branch is FDLIBM's trick for keeping
/// `1 - z/2` exact where cancellation would otherwise cost bits; it is the part
/// a "cleaner" rewrite would drop, and dropping it moves the last bit.
#[allow(clippy::excessive_precision)]
fn kernel_cos(x: f64, y: f64) -> f64 {
    const C1: f64 = 4.16666666666666019037e-02;
    const C2: f64 = -1.38888888888741095749e-03;
    const C3: f64 = 2.48015872894767294178e-05;
    const C4: f64 = -2.75573143513906633035e-07;
    const C5: f64 = 2.08757232129817482790e-09;
    const C6: f64 = -1.13596475577881948265e-11;

    let ix = hi_word(x) & 0x7fff_ffff;
    if ix < 0x3e40_0000 && (x as i32) == 0 {
        return 1.0;
    }
    let z = x * x;
    let r = z * (C1 + z * (C2 + z * (C3 + z * (C4 + z * (C5 + z * C6)))));
    if ix < 0x3fd3_3333 {
        // |x| < 0.3
        1.0 - (0.5 * z - (z * r - x * y))
    } else {
        let qx = if ix > 0x3fe9_0000 {
            // x > 0.78125
            0.28125
        } else {
            // x/4, with the low word zeroed
            from_words(ix - 0x0020_0000, 0)
        };
        let hz = 0.5 * z - qx;
        let a = 1.0 - qx;
        a - (hz - (z * r - x * y))
    }
}

/// FDLIBM `__ieee754_rem_pio2(x, y)` — `x mod pi/2`, returning the quadrant
/// count and writing the head/tail of the remainder into `y`.
///
/// # The one branch that is deliberately not ported
///
/// For `|x| >= 2^19 * pi/2` (about 8.2e5) FDLIBM switches to Payne-Hanek
/// reduction — `__kernel_rem_pio2`, a hundred-odd lines of multi-precision
/// integer arithmetic over a 66-word table of `2/pi`. **Every argument this
/// subsystem passes to a trig function is an angle**: `r.range(-PI, 0)`,
/// `i / n * 2 * PI`, an `atan2` result, a bearing. None of them can leave
/// `[-4*PI, 4*PI]`. Porting a hundred lines of bit-twiddling that no golden
/// could ever exercise would be dead code with a real chance of being silently
/// wrong, so the branch falls through to the platform libm and says so here.
/// It is the only input class on which [`js_sin`] / [`js_cos`] may differ from
/// V8, and it is the one class the engine cannot produce.
// `INVPIO2` is 2/pi and `eq_op` fires on FDLIBM's `x - x` idiom for "propagate
// the NaN, and turn an infinity into one". Both are the C's own text; rewriting
// either to please the lint would be rewriting the function under test.
#[allow(clippy::excessive_precision, clippy::approx_constant, clippy::eq_op)]
fn rem_pio2(x: f64, y: &mut [f64; 2]) -> i32 {
    const INVPIO2: f64 = 6.36619772367581382433e-01;
    const PIO2_1: f64 = 1.57079632673412561417e+00;
    const PIO2_1T: f64 = 6.07710050650619224932e-11;
    const PIO2_2: f64 = 6.07710050630396597660e-11;
    const PIO2_2T: f64 = 2.02226624879595063154e-21;
    const PIO2_3: f64 = 2.02226624871116645580e-21;
    const PIO2_3T: f64 = 8.47842766036889956997e-32;

    let hx = hi_word(x) as i32;
    let ix = hx & 0x7fff_ffff;

    // |x| <= pi/4 — nothing to reduce.
    if ix <= 0x3fe9_21fb {
        y[0] = x;
        y[1] = 0.0;
        return 0;
    }
    // |x| < 3*pi/4 — the n = +-1 case, done in 33+53 bit pi (or 33+33+53 right
    // at pi/2, where the first form loses too much).
    if ix < 0x4002_d97c {
        if hx > 0 {
            let mut z = x - PIO2_1;
            if ix != 0x3ff9_21fb {
                y[0] = z - PIO2_1T;
                y[1] = (z - y[0]) - PIO2_1T;
            } else {
                z -= PIO2_2;
                y[0] = z - PIO2_2T;
                y[1] = (z - y[0]) - PIO2_2T;
            }
            return 1;
        }
        let mut z = x + PIO2_1;
        if ix != 0x3ff9_21fb {
            y[0] = z + PIO2_1T;
            y[1] = (z - y[0]) + PIO2_1T;
        } else {
            z += PIO2_2;
            y[0] = z + PIO2_2T;
            y[1] = (z - y[0]) + PIO2_2T;
        }
        return -1;
    }
    // |x| <= 2^19 * pi/2 — medium size, up to three correction rounds.
    if ix <= 0x4139_21fb {
        let t = x.abs();
        let n = (t * INVPIO2 + 0.5) as i32;
        let fnn = f64::from(n);
        let mut r = t - fnn * PIO2_1;
        let mut w = fnn * PIO2_1T;
        let j = ix >> 20;
        y[0] = r - w;
        let mut i = j - ((hi_word(y[0]) as i32 >> 20) & 0x7ff);
        if i > 16 {
            // 2nd iteration, good to 118 bits
            let t2 = r;
            w = fnn * PIO2_2;
            r = t2 - w;
            w = fnn * PIO2_2T - ((t2 - r) - w);
            y[0] = r - w;
            i = j - ((hi_word(y[0]) as i32 >> 20) & 0x7ff);
            if i > 49 {
                // 3rd iteration, 151 bits
                let t3 = r;
                w = fnn * PIO2_3;
                r = t3 - w;
                w = fnn * PIO2_3T - ((t3 - r) - w);
                y[0] = r - w;
            }
        }
        y[1] = (r - y[0]) - w;
        if hx < 0 {
            y[0] = -y[0];
            y[1] = -y[1];
            return -n;
        }
        return n;
    }
    // Inf or NaN.
    if ix >= 0x7ff0_0000 {
        y[0] = x - x;
        y[1] = y[0];
        return 0;
    }
    // Unreachable: `js_sin` / `js_cos` divert everything past `HUGE_ARG` to the
    // platform libm before calling here, precisely so this branch never has to
    // exist. See the doc comment.
    unreachable!("rem_pio2 was called with |x| >= 2^19 * pi/2 ({x}); js_sin/js_cos divert those")
}

/// The first argument FDLIBM would send down the Payne-Hanek path — high word
/// `0x41392200`, i.e. just past `2^19 * pi/2`. Above this [`js_sin`] and
/// [`js_cos`] hand off to the platform libm; see [`rem_pio2`].
const HUGE_ARG_HI: u32 = 0x4139_21fb;

/// `Math.sin(x)` as **V8** computes it, not as the platform libm does.
///
/// The third measured V8 libm divergence in this port, after
/// [`js_hypot`] (milestone 1) and [`js_exp`] (milestone 5), and it was found
/// the same way — by measuring before trusting rather than after a golden
/// failed. `Math.sin` and `Math.cos` are the *third and fourth most used* math
/// functions in block 4 (27 and 26 call sites, behind only `Math.min`/`max`),
/// and `placeAnchors` — milestone 6's first function — calls both on every one
/// of its 400 candidate points.
///
/// | over 80,214 arguments spanning every reachable reduction branch | disagreements with V8 |
/// |---|---|
/// | `f64::sin` (the platform libm) | **1,942** |
/// | `f64::cos` | **2,160** |
/// | `js_sin` / `js_cos` | **0** / **0** |
///
/// V8 calls `base::ieee754::sin` / `cos`, which are FDLIBM's `__ieee754_sin` /
/// `__ieee754_cos`: reduce the argument mod pi/2 ([`rem_pio2`]), then evaluate
/// one of two degree-6 kernel polynomials ([`kernel_sin`], [`kernel_cos`])
/// according to the quadrant. Transliterated below with the bit manipulation
/// intact, for the reason `cartalith-rust-conventions` gives: match the JS
/// engine, do not improve on it.
///
/// See [`rem_pio2`] for the single input class this does not reproduce
/// (`|x| >= 2^19 * pi/2`, which no angle in this engine can reach).
// `x - x` is FDLIBM's own way of writing "NaN, with x's payload if it has one".
#[allow(clippy::eq_op)]
pub fn js_sin(x: f64) -> f64 {
    let ix = hi_word(x) & 0x7fff_ffff;
    if ix <= 0x3fe9_21fb {
        return kernel_sin(x, 0.0, 0);
    }
    if ix >= 0x7ff0_0000 {
        return x - x; // sin(Inf) and sin(NaN) are NaN
    }
    if ix > HUGE_ARG_HI {
        return x.sin(); // see `rem_pio2`: the Payne-Hanek branch is not ported
    }
    let mut y = [0.0f64; 2];
    match rem_pio2(x, &mut y) & 3 {
        0 => kernel_sin(y[0], y[1], 1),
        1 => kernel_cos(y[0], y[1]),
        2 => -kernel_sin(y[0], y[1], 1),
        _ => -kernel_cos(y[0], y[1]),
    }
}

/// `Math.cos(x)` as V8 computes it. See [`js_sin`] for the measurement and the
/// reasoning; this is the same reduction with the quadrant table rotated.
#[allow(clippy::eq_op)]
pub fn js_cos(x: f64) -> f64 {
    let ix = hi_word(x) & 0x7fff_ffff;
    if ix <= 0x3fe9_21fb {
        return kernel_cos(x, 0.0);
    }
    if ix >= 0x7ff0_0000 {
        return x - x;
    }
    if ix > HUGE_ARG_HI {
        return x.cos(); // see `rem_pio2`: the Payne-Hanek branch is not ported
    }
    let mut y = [0.0f64; 2];
    match rem_pio2(x, &mut y) & 3 {
        0 => kernel_cos(y[0], y[1]),
        1 => -kernel_sin(y[0], y[1], 1),
        2 => -kernel_cos(y[0], y[1]),
        _ => kernel_sin(y[0], y[1], 1),
    }
}

/// `Math.log(x)` as V8 computes it — FDLIBM's `__ieee754_log`.
///
/// The fifth divergence, and the reason it is ported *here* rather than at the
/// milestone that first needs it: [`crate::rng::Substream::norm`] is
/// `Math.sqrt(-2 * Math.log(u1)) * Math.cos(2 * PI * u2)`, so it runs through
/// **two** of these — and `norm` feeds `logn`, which draws every frontage
/// width, plot depth and building dimension in the town (five call sites in
/// block 4). Milestone 1 shipped `norm` on `f64::ln` and `f64::cos` with a
/// documented tolerance; milestone 5 removed the tolerance from `exp` and this
/// removes the last of it.
///
/// | over 60,009 arguments across the whole normal range | disagreements with V8 |
/// |---|---|
/// | `f64::ln` (the platform libm) | **1,647** |
/// | `js_log` | **0** |
///
/// `Math.sqrt` needs no such treatment: IEEE-754 mandates a correctly-rounded
/// square root, so V8's and Rust's agree by specification.
#[allow(clippy::excessive_precision, clippy::eq_op)]
pub fn js_log(mut x: f64) -> f64 {
    const LN2_HI: f64 = 6.93147180369123816490e-01;
    const LN2_LO: f64 = 1.90821492927058770002e-10;
    const TWO54: f64 = 1.80143985094819840000e+16;
    const LG1: f64 = 6.666666666666735130e-01;
    const LG2: f64 = 3.999999999940941908e-01;
    const LG3: f64 = 2.857142874366239149e-01;
    const LG4: f64 = 2.222219843214978396e-01;
    const LG5: f64 = 1.818357216161805012e-01;
    const LG6: f64 = 1.531383769920937332e-01;
    const LG7: f64 = 1.479819860511658591e-01;

    let mut hx = hi_word(x) as i32;
    let lx = lo_word(x);
    let mut k = 0i32;

    if hx < 0x0010_0000 {
        // subnormal, zero or negative
        if ((hx & 0x7fff_ffff) as u32 | lx) == 0 {
            return -TWO54 / 0.0; // log(+-0) = -inf
        }
        if hx < 0 {
            return (x - x) / 0.0; // log(negative) = NaN
        }
        k -= 54;
        x *= TWO54; // subnormal number, scale up
        hx = hi_word(x) as i32;
    }
    if hx >= 0x7ff0_0000 {
        return x + x; // Inf or NaN
    }
    k += (hx >> 20) - 1023;
    hx &= 0x000f_ffff;
    let i0 = (hx + 0x95f64) & 0x0010_0000;
    x = from_words((hx | (i0 ^ 0x3ff0_0000)) as u32, lo_word(x)); // normalise x or x/2
    k += i0 >> 20;
    let f = x - 1.0;
    let dk = f64::from(k);

    if (0x000f_ffff & (2 + hx)) < 3 {
        // |f| < 2^-20
        if f == 0.0 {
            if k == 0 {
                return 0.0;
            }
            return dk * LN2_HI + dk * LN2_LO;
        }
        let r = f * f * (0.5 - 0.33333333333333333 * f);
        if k == 0 {
            return f - r;
        }
        return dk * LN2_HI - ((r - dk * LN2_LO) - f);
    }

    let s = f / (2.0 + f);
    let z = s * s;
    let mut i = hx - 0x6147a;
    let w = z * z;
    let j = 0x6b851 - hx;
    let t1 = w * (LG2 + w * (LG4 + w * LG6));
    let t2 = z * (LG1 + w * (LG3 + w * (LG5 + w * LG7)));
    i |= j;
    let r = t2 + t1;
    if i > 0 {
        let hfsq = 0.5 * f * f;
        if k == 0 {
            f - (hfsq - s * (hfsq + r))
        } else {
            dk * LN2_HI - ((hfsq - (s * (hfsq + r) + dk * LN2_LO)) - f)
        }
    } else if k == 0 {
        f - s * (f - r)
    } else {
        dk * LN2_HI - ((s * (f - r) - dk * LN2_LO) - f)
    }
}

/// `V` (reference line 28286): a plain 2-D point/vector. `f64` throughout —
/// these are JS `Number`s and the engine's thresholds are tuned against their
/// exact values.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

/// Convenience constructor; `Vec2::new(x, y)` reads better than a struct
/// literal at the density these algorithms are written at.
impl Vec2 {
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
    /// `V.len` — `Math.hypot`, via [`js_hypot`].
    pub fn len(self) -> f64 {
        js_hypot(self.x, self.y)
    }
    /// `V.dist` — `Math.hypot`, via [`js_hypot`].
    pub fn dist(self, b: Vec2) -> f64 {
        js_hypot(self.x - b.x, self.y - b.y)
    }
    /// `V.norm` — divides by `hypot||1`, so the zero vector maps to itself
    /// rather than to NaN. Kept exactly, since several call sites rely on it.
    /// (JS `||` treats NaN as falsy too, so a NaN length also becomes 1.)
    pub fn norm(self) -> Vec2 {
        let h = js_hypot(self.x, self.y);
        let l = if h == 0.0 || h.is_nan() { 1.0 } else { h };
        Vec2::new(self.x / l, self.y / l)
    }
    /// `V.dot`
    pub fn dot(self, b: Vec2) -> f64 {
        self.x * b.x + self.y * b.y
    }
    /// `V.cross` — the 2-D scalar cross product.
    pub fn cross(self, b: Vec2) -> f64 {
        self.x * b.y - self.y * b.x
    }
    /// `V.lerp`
    pub fn lerp(self, b: Vec2, t: f64) -> Vec2 {
        Vec2::new(self.x + (b.x - self.x) * t, self.y + (b.y - self.y) * t)
    }
    /// `V.rot90`
    pub fn rot90(self) -> Vec2 {
        Vec2::new(-self.y, self.x)
    }
}

/// `V.add` — as the operator, so the dense ported geometry below reads as
/// arithmetic rather than as method chains.
impl std::ops::Add for Vec2 {
    type Output = Vec2;
    fn add(self, b: Vec2) -> Vec2 {
        Vec2::new(self.x + b.x, self.y + b.y)
    }
}
/// `V.sub`
impl std::ops::Sub for Vec2 {
    type Output = Vec2;
    fn sub(self, b: Vec2) -> Vec2 {
        Vec2::new(self.x - b.x, self.y - b.y)
    }
}
/// `V.mul` — vector times scalar.
impl std::ops::Mul<f64> for Vec2 {
    type Output = Vec2;
    fn mul(self, s: f64) -> Vec2 {
        Vec2::new(self.x * s, self.y * s)
    }
}

/// `polyArea` (line 28290) — signed shoelace area, positive for CCW.
pub fn poly_area(p: &[Vec2]) -> f64 {
    let n = p.len();
    let mut s = 0.0;
    for i in 0..n {
        let a = p[i];
        let b = p[(i + 1) % n];
        s += a.x * b.y - b.x * a.y;
    }
    s / 2.0
}

/// `polyCentroid` (line 28291) — area-weighted centroid, falling back to the
/// vertex mean when the signed area is degenerate (`|2A| < 1e-9`).
pub fn poly_centroid(p: &[Vec2]) -> Vec2 {
    let n = p.len();
    let (mut sx, mut sy, mut sa) = (0.0, 0.0, 0.0);
    for i in 0..n {
        let a = p[i];
        let b = p[(i + 1) % n];
        let c = a.x * b.y - b.x * a.y;
        sa += c;
        sx += (a.x + b.x) * c;
        sy += (a.y + b.y) * c;
    }
    if sa.abs() < 1e-9 {
        let (mut mx, mut my) = (0.0, 0.0);
        for q in p {
            mx += q.x;
            my += q.y;
        }
        return Vec2::new(mx / n as f64, my / n as f64);
    }
    Vec2::new(sx / (3.0 * sa), sy / (3.0 * sa))
}

/// `pointInPoly` (line 28295) — the standard crossing-number test, with the
/// reference's exact half-open edge convention (`(yi>pt.y) !== (yj>pt.y)`).
pub fn point_in_poly(pt: Vec2, p: &[Vec2]) -> bool {
    let mut inside = false;
    let n = p.len();
    if n == 0 {
        return false;
    }
    let mut j = n - 1;
    for i in 0..n {
        let (xi, yi) = (p[i].x, p[i].y);
        let (xj, yj) = (p[j].x, p[j].y);
        if ((yi > pt.y) != (yj > pt.y)) && (pt.x < (xj - xi) * (pt.y - yi) / (yj - yi) + xi) {
            inside = !inside;
        }
        j = i;
    }
    inside
}

/// What [`seg_int`] returns: the two parameters and the crossing point.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SegHit {
    pub t: f64,
    pub u: f64,
    pub pt: Vec2,
}

/// `segInt` (line 28298) — **segment**-segment intersection, `None` when the
/// denominator is under `1e-12` (parallel/collinear, including the overlapping
/// collinear case) or when either parameter falls outside `[-1e-9, 1+1e-9]`.
pub fn seg_int(p: Vec2, p2: Vec2, q: Vec2, q2: Vec2) -> Option<SegHit> {
    let r = Vec2::new(p2.x - p.x, p2.y - p.y);
    let s = Vec2::new(q2.x - q.x, q2.y - q.y);
    let den = r.cross(s);
    if den.abs() < 1e-12 {
        return None;
    }
    let qp = Vec2::new(q.x - p.x, q.y - p.y);
    let t = qp.cross(s) / den;
    let u = qp.cross(r) / den;
    if !(-1e-9..=1.0 + 1e-9).contains(&t) || !(-1e-9..=1.0 + 1e-9).contains(&u) {
        return None;
    }
    Some(SegHit { t, u, pt: Vec2::new(p.x + t * r.x, p.y + t * r.y) })
}

/// `distPtSeg` (line 28305) — point-to-segment distance, degenerate segments
/// falling back to point-to-point.
pub fn dist_pt_seg(pt: Vec2, a: Vec2, b: Vec2) -> f64 {
    let ab = b - a;
    let l2 = ab.x * ab.x + ab.y * ab.y;
    if l2 < 1e-12 {
        return pt.dist(a);
    }
    let mut t = ((pt.x - a.x) * ab.x + (pt.y - a.y) * ab.y) / l2;
    t = t.clamp(0.0, 1.0);
    pt.dist(Vec2::new(a.x + ab.x * t, a.y + ab.y * t))
}

/// `polySelfIntersects` (line 28309) — O(n²) all-pairs edge test skipping
/// adjacent pairs and the wrap-around pair. Quadratic on purpose: its only
/// caller ([`inset_poly`]) gates it behind a 60-vertex cap.
pub fn poly_self_intersects(p: &[Vec2]) -> bool {
    let n = p.len();
    for i in 0..n {
        for j in (i + 1)..n {
            if i.abs_diff(j) <= 1 || (i == 0 && j == n - 1) {
                continue;
            }
            if seg_int(p[i], p[(i + 1) % n], p[j], p[(j + 1) % n]).is_some() {
                return true;
            }
        }
    }
    false
}

/// `chaikin` (line 28314) — one corner-cutting subdivision pass at the ¼ / ¾
/// parameters. Open polylines keep their endpoints; closed ones wrap.
pub fn chaikin(pts: &[Vec2], closed: bool) -> Vec<Vec2> {
    let n = pts.len();
    if n == 0 {
        return Vec::new();
    }
    let lim = if closed { n } else { n - 1 };
    let mut out = Vec::with_capacity(2 * lim + 2);
    if !closed {
        out.push(pts[0]);
    }
    for i in 0..lim {
        let a = pts[i];
        let b = pts[(i + 1) % n];
        out.push(a.lerp(b, 0.25));
        out.push(a.lerp(b, 0.75));
    }
    if !closed {
        out.push(pts[n - 1]);
    }
    out
}

/// `simplify` (line 28321) — iterative Douglas-Peucker. Fewer than three
/// points passes through unchanged.
pub fn simplify(pts: &[Vec2], tol: f64) -> Vec<Vec2> {
    if pts.len() < 3 {
        return pts.to_vec();
    }
    let mut keep = vec![0u8; pts.len()];
    keep[0] = 1;
    let last = pts.len() - 1;
    keep[last] = 1;
    let mut stack = vec![(0usize, last)];
    while let Some((a, b)) = stack.pop() {
        let mut mx = -1.0f64;
        let mut mi = usize::MAX;
        for i in (a + 1)..b {
            let d = dist_pt_seg(pts[i], pts[a], pts[b]);
            if d > mx {
                mx = d;
                mi = i;
            }
        }
        if mx > tol {
            keep[mi] = 1;
            stack.push((a, mi));
            stack.push((mi, b));
        }
    }
    pts.iter().enumerate().filter(|(i, _)| keep[*i] == 1).map(|(_, p)| *p).collect()
}

/// `ensureCCW` (line 28330) — reverse if the shoelace area is negative, so the
/// interior lies to the LEFT of every directed edge.
pub fn ensure_ccw(p: &[Vec2]) -> Vec<Vec2> {
    if poly_area(p) < 0.0 {
        let mut v = p.to_vec();
        v.reverse();
        v
    } else {
        p.to_vec()
    }
}

/// `insetPoly` (line 28332) — per-edge inward offset with miter joins.
/// `dists[i]` applies to edge `i -> i+1`, and a short `dists` clamps to its
/// last entry. CCW input expected.
///
/// Returns `None` on the reference's own three rejections: fewer than three
/// vertices, a result whose signed area is below 15, or (at 60 vertices or
/// fewer) a result that self-intersects.
pub fn inset_poly(poly: &[Vec2], dists: &[f64]) -> Option<Vec<Vec2>> {
    let n = poly.len();
    if n < 3 {
        return None;
    }
    // `Math.max(...dists)` — used twice below as the miter-runaway clamp.
    let dmax = dists.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let mut lines: Vec<(Vec2, Vec2)> = Vec::with_capacity(n); // (point, direction)
    for i in 0..n {
        let a = poly[i];
        let b = poly[(i + 1) % n];
        let d = (b - a).norm();
        let nl = Vec2::new(-d.y, d.x); // inward (left) normal for CCW
        let off = dists[i.min(dists.len() - 1)];
        lines.push((a + nl * off, d));
    }
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let l1 = lines[(i + n - 1) % n];
        let l2 = lines[i];
        let den = l1.1.cross(l2.1);
        let pt = if den.abs() < 1e-9 {
            l2.0
        } else {
            let t = (l2.0 - l1.0).cross(l2.1) / den;
            let mut pt = l1.0 + l1.1 * t;
            if pt.dist(poly[i]) > 4.0 * dmax + 6.0 {
                pt = poly[i] + (pt - poly[i]).norm() * (dmax * 1.8);
            }
            pt
        };
        out.push(pt);
    }
    if poly_area(&out) < 15.0 {
        return None;
    }
    if out.len() <= 60 && poly_self_intersects(&out) {
        return None;
    }
    Some(out)
}

/// `clipConvex` (line 28351) — Sutherland-Hodgman against a convex CCW clip
/// polygon. See this module's header for the segment-vs-line caveat, which is
/// reference behaviour and is pinned by a golden test.
pub fn clip_convex(subject: &[Vec2], clip: &[Vec2]) -> Vec<Vec2> {
    let mut out = subject.to_vec();
    let m = clip.len();
    let mut i = 0;
    while i < m && !out.is_empty() {
        let a = clip[i];
        let b = clip[(i + 1) % m];
        let inp = std::mem::take(&mut out);
        let inside = |p: Vec2| (b - a).cross(p - a) >= -1e-9;
        let ln = inp.len();
        for j in 0..ln {
            let cur = inp[j];
            let prv = inp[(j + ln - 1) % ln];
            let (ci, pi) = (inside(cur), inside(prv));
            if ci {
                if !pi && let Some(h) = seg_int(prv, cur, a, b) {
                    out.push(h.pt);
                }
                out.push(cur);
            } else if pi && let Some(h) = seg_int(prv, cur, a, b) {
                out.push(h.pt);
            }
        }
        i += 1;
    }
    out
}

/// `convexHull` (line 29639) — monotone chain. Collinear points are dropped
/// (`<= 0` on the cross product), and the sort is by x then y, stable, exactly
/// as JS's own stable `Array.prototype.sort` with that comparator.
pub fn convex_hull(pts: &[Vec2]) -> Vec<Vec2> {
    let mut p = pts.to_vec();
    // `(a,b)=>a.x-b.x||a.y-b.y`: JS treats a NaN comparator result as 0, i.e.
    // "keep the existing order" under a stable sort — `unwrap_or(Equal)` on a
    // stable `sort_by` is exactly that, not a papered-over NaN.
    p.sort_by(|a, b| {
        a.x.partial_cmp(&b.x)
            .filter(|o| o.is_ne())
            .unwrap_or_else(|| a.y.partial_cmp(&b.y).unwrap_or(std::cmp::Ordering::Equal))
    });
    if p.len() < 3 {
        return p;
    }
    let cr = |o: Vec2, a: Vec2, b: Vec2| (a.x - o.x) * (b.y - o.y) - (a.y - o.y) * (b.x - o.x);
    let mut lo: Vec<Vec2> = Vec::new();
    for q in &p {
        while lo.len() >= 2 && cr(lo[lo.len() - 2], lo[lo.len() - 1], *q) <= 0.0 {
            lo.pop();
        }
        lo.push(*q);
    }
    let mut up: Vec<Vec2> = Vec::new();
    for q in p.iter().rev() {
        while up.len() >= 2 && cr(up[up.len() - 2], up[up.len() - 1], *q) <= 0.0 {
            up.pop();
        }
        up.push(*q);
    }
    lo.pop();
    up.pop();
    lo.extend(up);
    lo
}

#[cfg(test)]
mod tests {
    use super::*;

    /// See `rng.rs`'s own test-module comment for how these expected values
    /// were captured: block 4 sliced contiguously out of the frozen reference,
    /// balance-asserted at both boundaries, run under Node `vm.runInContext`,
    /// every function reached through the reference's own `UME._test` export.
    /// Every value below is compared **exactly** — none of this module's
    /// arithmetic passes through a transcendental function whose last bit could
    /// legitimately differ between V8 and Rust.
    fn p(x: f64, y: f64) -> Vec2 {
        Vec2::new(x, y)
    }
    fn pts(v: &[(f64, f64)]) -> Vec<Vec2> {
        v.iter().map(|&(x, y)| p(x, y)).collect()
    }
    fn flat(v: &[Vec2]) -> Vec<(f64, f64)> {
        v.iter().map(|q| (q.x, q.y)).collect()
    }
    fn square() -> Vec<Vec2> {
        pts(&[(0., 0.), (10., 0.), (10., 10.), (0., 10.)])
    }
    fn cw_square() -> Vec<Vec2> {
        let mut v = square();
        v.reverse();
        v
    }
    fn tri() -> Vec<Vec2> {
        pts(&[(0., 0.), (8., 0.), (0., 6.)])
    }
    fn l_shape() -> Vec<Vec2> {
        pts(&[(0., 0.), (12., 0.), (12., 4.), (5., 4.), (5., 11.), (0., 11.)])
    }

    #[test]
    fn golden_js_hypot_differs_from_rust_hypot() {
        // Values captured from the same Node run. The (3,3) row is the one
        // that broke `dist_pt_seg` before `js_hypot` existed, and the assert
        // below states plainly that Rust's own `hypot` gives a DIFFERENT
        // answer — so nobody "simplifies" this back to `f64::hypot` later.
        for (x, y, want) in [
            (3.0, 3.0, 4.242640687119286),
            (3.0, 4.0, 5.0),
            (0.1, 0.2, 0.223606797749979),
            (1e-300, 1e-300, 1.4142135623730952e-300),
            (1e300, 1e300, 1.4142135623730952e300),
            (0.0, 0.0, 0.0),
            (0.0, 5.0, 5.0),
            (-7.0, 0.0, 7.0),
            (1.0, 1.0, std::f64::consts::SQRT_2),
            (2.5, 7.25, 7.668930825088984),
            (123.456, 789.012, 798.612121170221),
            (1e-320, 0.0, 1e-320),
        ] {
            assert_eq!(js_hypot(x, y), want, "js_hypot({x}, {y})");
        }
        assert_ne!(js_hypot(3.0, 3.0), 3.0f64.hypot(3.0));
        // Math.hypot(±∞, NaN) is ∞ per spec, ahead of the NaN rule.
        assert!(js_hypot(f64::INFINITY, f64::NAN).is_infinite());
        assert!(js_hypot(f64::NAN, 1.0).is_nan());
    }

    #[test]
    fn golden_js_exp_differs_from_rust_exp() {
        // Captured from the same Node run. Every row below is one where the
        // platform `f64::exp` gives a DIFFERENT answer, which the assert at the
        // end states plainly -- the same device `js_hypot` carries, for the same
        // reason: this is not an accuracy improvement waiting to be made.
        const CASES: [(f64, f64); 8] = [
            (-2.449164366815239, 0.08636572642093979),
            (-1.529806137084961, 0.21657764962095738),
            (-2.3961539268493652, 0.0910675329974808),
            (-1.8253226280212402, 0.16116563918983867),
            (-2.2492117881774902, 0.10548233422645462),
            (-1.816183090209961, 0.16264537037235),
            (-1.0032005310058594, 0.366703913772948),
            (-1.1515216827392578, 0.3161553150757277),
        ];
        let mut differ = 0;
        for (x, want) in CASES {
            assert_eq!(js_exp(x), want, "js_exp({x})");
            if x.exp() != want {
                differ += 1;
            }
        }
        assert_eq!(differ, CASES.len(), "these rows exist to discriminate; f64::exp now agrees on some");

        // and the rest of the domain, including the special case measured at
        // exactly 1.0 and the reduction-branch boundaries
        for (x, want) in [
            (0.0, 1.0),
            (-0.0, 1.0),
            (1.0, std::f64::consts::E),
            (-1.0, 0.36787944117144233),
            (2.0, 7.38905609893065),
            (0.5, 1.6487212707001282),
            (1e-30, 1.0),
            (709.78, 1.7928227943945155e308),
            (-745.2, 0.0),
        ] {
            assert_eq!(js_exp(x), want, "js_exp({x})");
        }
        assert!(js_exp(709.79).is_infinite());
        assert!(js_exp(f64::INFINITY).is_infinite());
        assert_eq!(js_exp(f64::NEG_INFINITY), 0.0);
        assert!(js_exp(f64::NAN).is_nan());
    }

    // `cos(pi/4)` really is one ulp above `FRAC_1_SQRT_2` in V8, which is the
    // whole point of the row; naming the constant would replace a captured
    // value with a different number.
    #[allow(clippy::approx_constant)]
    #[test]
    fn golden_js_sin_and_js_cos_differ_from_rust_sin_and_cos() {
        // Captured from the same Node run, two per reduction branch per
        // function: |x| <= pi/4 (straight into the kernel polynomial),
        // pi/4 < |x| < 3pi/4 (rem_pio2's n = +-1 special case), and the medium
        // branch with its up-to-three correction rounds. Every row is one where
        // the PLATFORM libm gives a different answer, asserted at the end so
        // nobody replaces this with `f64::sin`.
        const SIN: [(f64, f64); 6] = [
            (-0.1127985306084156, -0.11255948389078078),
            (-0.6115948781371117, -0.5741739696670092),
            (1.9793160259723663, 0.9177098794782881),
            (-1.1484385468065739, -0.9121249937795464),
            (-122.20573341473937, -0.31112834633397485),
            (-175.44943382963538, 0.46156164512934583),
        ];
        const COS: [(f64, f64); 6] = [
            (-0.6561320275068283, 0.7923578350894903),
            (0.6255884654819965, 0.8106186695614905),
            (-1.4369313605129719, 0.13346551813019575),
            (-1.9593198783695698, -0.37882242070802385),
            (178.00320917740464, -0.48225258690906103),
            (30.05973929539323, 0.21296549854266927),
        ];
        let mut differ = 0;
        for (x, want) in SIN {
            assert_eq!(js_sin(x), want, "js_sin({x})");
            if x.sin() != want {
                differ += 1;
            }
        }
        for (x, want) in COS {
            assert_eq!(js_cos(x), want, "js_cos({x})");
            if x.cos() != want {
                differ += 1;
            }
        }
        assert_eq!(
            differ,
            SIN.len() + COS.len(),
            "these rows exist to discriminate; the platform sin/cos now agrees on some"
        );

        // and the rest of the domain: exact quadrant boundaries, the tiny-x
        // shortcut, and the non-finite rules.
        for (x, s, c) in [
            (0.0, 0.0, 1.0),
            (std::f64::consts::PI, 1.2246467991473532e-16, -1.0),
            (-std::f64::consts::PI, -1.2246467991473532e-16, -1.0),
            (std::f64::consts::FRAC_PI_2, 1.0, 6.123233995736766e-17),
            (-std::f64::consts::FRAC_PI_2, -1.0, 6.123233995736766e-17),
            (std::f64::consts::FRAC_PI_4, 0.7071067811865475, 0.7071067811865476),
            (std::f64::consts::TAU, -2.4492935982947064e-16, 1.0),
            (1e-30, 1e-30, 1.0),
            (1.0, 0.8414709848078965, 0.5403023058681398),
        ] {
            assert_eq!(js_sin(x), s, "js_sin({x})");
            assert_eq!(js_cos(x), c, "js_cos({x})");
        }
        assert_eq!(js_sin(-0.0), -0.0);
        assert!(js_sin(f64::INFINITY).is_nan());
        assert!(js_cos(f64::NEG_INFINITY).is_nan());
        assert!(js_sin(f64::NAN).is_nan());
        assert!(js_cos(f64::NAN).is_nan());
        // Past 2^19 * pi/2 the Payne-Hanek branch is deliberately not ported
        // and the platform libm answers instead (see `rem_pio2`). Asserting
        // that here is what stops a later reader assuming full coverage.
        let huge = 4.0e6;
        assert_eq!(js_sin(huge), huge.sin());
        assert_eq!(js_cos(huge), huge.cos());
    }

    #[test]
    fn golden_js_log_differs_from_rust_ln() {
        const CASES: [(f64, f64); 4] = [
            (2.6795544533384854, 0.9856505319476128),
            (0.43189338049297177, -0.8395765256136656),
            (0.5566417494533674, -0.5858334247022867),
            (1.6544200123691382, 0.5034505017101765),
        ];
        let mut differ = 0;
        for (x, want) in CASES {
            assert_eq!(js_log(x), want, "js_log({x})");
            if x.ln() != want {
                differ += 1;
            }
        }
        assert_eq!(differ, CASES.len(), "these rows exist to discriminate; f64::ln now agrees on some");

        for (x, want) in [
            (1.0, 0.0),
            (2.0, std::f64::consts::LN_2),
            (0.5, -std::f64::consts::LN_2),
            (std::f64::consts::E, 1.0),
            (1e-320, -736.8272408909739), // subnormal: the two54 rescale branch
            (1e300, 690.7755278982137),
        ] {
            assert_eq!(js_log(x), want, "js_log({x})");
        }
        assert_eq!(js_log(0.0), f64::NEG_INFINITY);
        assert_eq!(js_log(-0.0), f64::NEG_INFINITY);
        assert!(js_log(-1.0).is_nan());
        assert!(js_log(f64::NAN).is_nan());
        assert_eq!(js_log(f64::INFINITY), f64::INFINITY);
    }

    /// The **bulk** libm golden: one FNV-1a hash per function over tens of
    /// thousands of V8 results, with the arguments drawn by the reference's own
    /// `mulberry32` so both sides provably evaluate the same points.
    ///
    /// This test exists because of a mutation result, and the result is worth
    /// stating. The hand-picked tables above — a dozen discriminating rows per
    /// function, chosen exactly the way [`js_exp`]'s and [`js_hypot`]'s were —
    /// left **63 of milestone 6's mutations alive** inside these three
    /// functions: every reduction threshold, every `y[0]`/`y[1]` slot, both
    /// correction-round triggers and the whole `kernel_cos` `qx` split were
    /// untested. A dozen rows cannot cover branchy bit-manipulation; they cover
    /// a dozen paths through it. The hash covers every branch the engine can
    /// reach, in four lines of golden.
    ///
    /// The bands are chosen to enter each branch on purpose: `|x| <= pi/4`
    /// straight into the kernel polynomial, `|x| < 3pi/4` for `rem_pio2`'s
    /// `n = +-1` case, and two medium bands — one small enough that the first
    /// correction round suffices, one large enough to need the second and
    /// third. `log` gets the ordinary range, a band hugging `1.0` (the
    /// `|f| < 2^-20` shortcut), the top and bottom of the normal range, and a
    /// subnormal band for the `two54` rescale.
    #[test]
    fn golden_js_sin_cos_log_hash_over_every_reduction_branch() {
        use cartalith_rng::Mulberry32;

        fn fold(mut h: u32, x: f64) -> u32 {
            for b in x.to_le_bytes() {
                h ^= u32::from(b);
                h = h.wrapping_mul(0x0100_0193);
            }
            h
        }
        const N: usize = 6000;
        const FNV_OFFSET: u32 = 0x811c_9dc5;

        fn sweep(seed: u32, bands: &[fn(f64) -> f64], f: fn(f64) -> f64) -> (u32, usize, usize) {
            let mut r = Mulberry32::new(seed);
            let mut h = FNV_OFFSET;
            let (mut n, mut finite) = (0, 0);
            for band in bands {
                for _ in 0..N {
                    let v = f(band(r.next_f64()));
                    h = fold(h, v);
                    n += 1;
                    if v.is_finite() {
                        finite += 1;
                    }
                }
            }
            (h, n, finite)
        }

        let trig: [fn(f64) -> f64; 9] = [
            |u| (u - 0.5) * std::f64::consts::FRAC_PI_2,
            |u| (u - 0.5) * (3.0 * std::f64::consts::PI),
            |u| (u - 0.5) * 800.0,
            |u| (u - 0.5) * 1.6e6,
            // `rem_pio2`'s SECOND correction round fires when the exponent gap
            // exceeds 16, around |x| ~ 2^17. Neither band above lands there —
            // 800 gives a gap of ~9 and 1.6e6 gives ~20 — so `i > 16` was
            // invisible until this band existed.
            |u| (u - 0.5) * 3e5,
            // ...and the gap is `exponent(x) - exponent(y0)`, so 3e5 gives ~19-23
            // and 800 gives ~10-14. A gap of exactly 17 needs
            // |x| in [2^16, 2^17) — 1.2e5 is a whole binade short of it.
            |u| (u - 0.5) * 2.4e5,
            // The THIRD round needs a gap over 49, i.e. a remainder below about
            // 2^-32 — and an *added* offset cannot produce one, because at
            // |x| ~ 6400 the ulp is already 1e-12 and anything smaller is
            // absorbed. The remainder that is that small is the representation
            // error of `fl(k * pi/2)` itself, so these bands are exact multiples
            // with no offset at all.
            |u| (u * 4096.0).floor() * std::f64::consts::FRAC_PI_2,
            |u| (u * 1e6).floor() * std::f64::consts::FRAC_PI_2,
            // ...and one whose reduced remainder is around 1e-9 rather than
            // 1e-13, because that is the only band that enters the kernels'
            // own `|x| < 2^-27` shortcut. Dropping it resurrected two mutants
            // that a previous round had killed, which is why both are here.
            |u| (u * 4096.0).floor() * std::f64::consts::FRAC_PI_2
                + ((u * 4096.0) % 1.0 - 0.5) * 1e-9,
        ];
        let logs: [fn(f64) -> f64; 5] = [
            |u| u * 4.0,
            |u| 1.0 + (u - 0.5) * 1e-6,
            |u| u * 1e308,
            |u| u * 1e-305,
            |u| u * 1e-320,
        ];

        // Captured from the reference under Node: `Math.sin`/`Math.cos`/
        // `Math.log` over the identical argument sequence.
        for (name, seed, hash, count, want, bands) in [
            ("sin", 0x0005_1ade_u32, 0x9282_6171_u32, 54_000, js_sin as fn(f64) -> f64, &trig[..]),
            ("cos", 0x00c0_512e, 0x9788_c272, 54_000, js_cos as fn(f64) -> f64, &trig[..]),
            ("log", 0x0010_9a11, 0x6495_44e2, 30_000, js_log as fn(f64) -> f64, &logs[..]),
        ] {
            let (got, n, finite) = sweep(seed, bands, want);
            // The emptiness / shape gate: a hash proves nothing if the sweep
            // silently produced a wall of NaN or ran zero iterations.
            assert_eq!(n, count, "{name}: wrong argument count");
            assert!(finite * 100 >= n * 99, "{name}: only {finite}/{n} results are finite");
            assert_ne!(got, FNV_OFFSET, "{name}: the hash never advanced");
            assert_eq!(got, hash, "{name}: V8 hash over {n} arguments");
        }
    }

    /// `js_round` is JS's rounding rule, not Rust's, and the two differ in both
    /// directions from the obvious transliterations.
    #[test]
    fn js_round_is_neither_f64_round_nor_floor_of_x_plus_half() {
        for (x, want) in [
            (2.5, 3.0),
            (-2.5, -2.0),   // f64::round gives -3
            (-0.5, 0.0),    // f64::round gives -1
            (0.49999999999999994, 0.0), // (x + 0.5).floor() gives 1
            (2.4, 2.0),
            (-2.6, -3.0),
            (0.0, 0.0),
            (1e300, 1e300),
        ] {
            assert_eq!(js_round(x), want, "js_round({x})");
        }
        assert_ne!(js_round(-2.5), (-2.5f64).round());
        assert_ne!(js_round(0.49999999999999994), (0.49999999999999994f64 + 0.5).floor());
        assert!(js_round(f64::NAN).is_nan());
        assert_eq!(js_round(f64::INFINITY), f64::INFINITY);
    }

    #[test]
    fn golden_poly_area() {
        assert_eq!(poly_area(&square()), 100.0);
        assert_eq!(poly_area(&cw_square()), -100.0);
        assert_eq!(poly_area(&tri()), 24.0);
        assert_eq!(poly_area(&l_shape()), 83.0);
        // collinear -> exactly zero, and a self-crossing bowtie's lobes cancel
        assert_eq!(poly_area(&pts(&[(1., 1.), (3., 1.), (5., 1.)])), 0.0);
        assert_eq!(poly_area(&pts(&[(0., 0.), (10., 10.), (10., 0.), (0., 10.)])), 0.0);
    }

    #[test]
    fn golden_poly_centroid() {
        assert_eq!(flat(&[poly_centroid(&square())]), [(5.0, 5.0)]);
        assert_eq!(flat(&[poly_centroid(&cw_square())]), [(5.0, 5.0)]);
        assert_eq!(flat(&[poly_centroid(&tri())]), [(2.6666666666666665, 2.0)]);
        assert_eq!(
            flat(&[poly_centroid(&l_shape())]),
            [(4.524096385542169, 4.319277108433735)]
        );
        // degenerate (zero signed area) -> vertex mean, the reference's fallback
        assert_eq!(flat(&[poly_centroid(&pts(&[(1., 1.), (3., 1.), (5., 1.)]))]), [(3.0, 1.0)]);
    }

    #[test]
    fn golden_point_in_poly() {
        // includes the reference's asymmetric edge convention: the (0,0) corner
        // and the x=10 edge of the same square answer differently.
        for (pt, poly, want) in [
            (p(5., 5.), square(), true),
            (p(-1., 5.), square(), false),
            (p(0., 0.), square(), true),
            (p(10., 5.), square(), false),
            (p(2., 2.), tri(), true),
            (p(7., 5.), tri(), false),
            (p(3., 8.), l_shape(), true),
            (p(8., 8.), l_shape(), false),
        ] {
            assert_eq!(point_in_poly(pt, &poly), want, "point_in_poly({pt:?})");
        }
    }

    #[test]
    fn golden_seg_int() {
        assert_eq!(
            seg_int(p(0., 0.), p(10., 10.), p(0., 10.), p(10., 0.)),
            Some(SegHit { t: 0.5, u: 0.5, pt: p(5., 5.) })
        );
        assert_eq!(seg_int(p(0., 0.), p(10., 0.), p(0., 5.), p(10., 5.)), None); // parallel
        assert_eq!(
            seg_int(p(0., 0.), p(10., 0.), p(5., 0.), p(5., 5.)),
            Some(SegHit { t: 0.5, u: 0.0, pt: p(5., 0.) })
        ); // T-junction: u lands exactly on 0
        assert_eq!(seg_int(p(0., 0.), p(4., 0.), p(6., 0.), p(10., 0.)), None); // collinear
        assert_eq!(seg_int(p(0., 0.), p(1., 1.), p(2., 2.), p(3., 3.)), None); // collinear, apart
        let h = seg_int(p(0., 0.), p(3., 7.), p(1., 6.), p(9., 2.)).expect("oblique crossing");
        assert_eq!(h.t, 0.7647058823529411);
        assert_eq!(h.u, 0.16176470588235295);
        assert_eq!((h.pt.x, h.pt.y), (2.2941176470588234, 5.352941176470588));
    }

    #[test]
    fn golden_dist_pt_seg() {
        assert_eq!(dist_pt_seg(p(5., 5.), p(0., 0.), p(10., 0.)), 5.0);
        assert_eq!(dist_pt_seg(p(-3., 4.), p(0., 0.), p(10., 0.)), 5.0); // clamped to t=0
        assert_eq!(dist_pt_seg(p(13., 4.), p(0., 0.), p(10., 0.)), 5.0); // clamped to t=1
        assert_eq!(dist_pt_seg(p(2., 2.), p(1., 1.), p(1., 1.)), std::f64::consts::SQRT_2); // degenerate
        assert_eq!(dist_pt_seg(p(3., 9.), p(0., 0.), p(6., 6.)), 4.242640687119286);
    }

    #[test]
    fn golden_ensure_ccw() {
        let want = flat(&square());
        assert_eq!(flat(&ensure_ccw(&square())), want);
        assert_eq!(flat(&ensure_ccw(&cw_square())), want);
    }

    #[test]
    fn golden_chaikin() {
        assert_eq!(
            flat(&chaikin(&tri(), false)),
            [(0., 0.), (2., 0.), (6., 0.), (6., 1.5), (2., 4.5), (0., 6.)]
        );
        assert_eq!(
            flat(&chaikin(&square(), true)),
            [
                (2.5, 0.), (7.5, 0.), (10., 2.5), (10., 7.5),
                (7.5, 10.), (2.5, 10.), (0., 7.5), (0., 2.5)
            ]
        );
        assert_eq!(
            flat(&chaikin(&l_shape(), true)),
            [
                (3., 0.), (9., 0.), (12., 1.), (12., 3.), (10.25, 4.), (6.75, 4.),
                (5., 5.75), (5., 9.25), (3.75, 11.), (1.25, 11.), (0., 8.25), (0., 2.75)
            ]
        );
        // buildSite's own idiom: chaikin(chaikin(pts,false),false)
        let once = chaikin(&pts(&[(0., 0.), (3., 5.), (9., 1.), (12., 7.)]), false);
        assert_eq!(
            flat(&chaikin(&once, false)),
            [
                (0., 0.), (0.1875, 0.3125), (0.5625, 0.9375), (1.125, 1.875), (1.875, 3.125),
                (2.8125, 3.8125), (3.9375, 3.9375), (5.25, 3.5), (6.75, 2.5), (8.0625, 2.125),
                (9.1875, 2.375), (10.125, 3.25), (10.875, 4.75), (11.4375, 5.875),
                (11.8125, 6.625), (12., 7.)
            ]
        );
    }

    #[test]
    fn golden_simplify() {
        let line: Vec<Vec2> =
            (0..=20).map(|i| p(i as f64 * 5.0, (i as f64 / 3.0).sin() * 9.0)).collect();
        assert_eq!(
            flat(&simplify(&line, 0.5)),
            [
                (0., 0.), (15., 7.573238863271069), (25., 8.958671619765884),
                (35., 6.507772935644921), (60., -6.811222457771354), (70., -8.990594253881355),
                (80., -7.31996452410822), (100., 3.3673610751409795)
            ]
        );
        assert_eq!(
            flat(&simplify(&line, 3.0)),
            [
                (0., 0.), (25., 8.958671619765884), (70., -8.990594253881355),
                (100., 3.3673610751409795)
            ]
        );
        assert_eq!(flat(&simplify(&line, 30.0)), [(0., 0.), (100., 3.3673610751409795)]);
        assert_eq!(flat(&simplify(&pts(&[(0., 0.), (1., 1.)]), 1.0)), [(0., 0.), (1., 1.)]);
    }

    #[test]
    fn golden_inset_poly() {
        assert_eq!(
            flat(&inset_poly(&square(), &[3., 3., 3., 3.]).expect("uniform inset")),
            [(3., 3.), (7., 3.), (7., 7.), (3., 7.)]
        );
        assert_eq!(
            flat(&inset_poly(&square(), &[1., 2., 3., 4.]).expect("per-edge inset")),
            [(4., 1.), (8., 1.), (8., 7.), (4., 7.)]
        );
        // short `dists` clamps to its last entry, so one value insets uniformly
        assert_eq!(
            flat(&inset_poly(&square(), &[2.]).expect("clamped dists")),
            [(2., 2.), (8., 2.), (8., 8.), (2., 8.)]
        );
        // the reference's three rejections, all `None`:
        assert!(inset_poly(&square(), &[6., 6., 6., 6.]).is_none()); // inverted
        assert!(inset_poly(&ensure_ccw(&tri()), &[1., 1., 1.]).is_none()); // area < 15
        assert!(inset_poly(&ensure_ccw(&l_shape()), &[2.; 6]).is_none()); // reflex corner
        assert!(inset_poly(&pts(&[(0., 0.), (1., 0.)]), &[1., 1.]).is_none()); // n < 3
    }

    #[test]
    fn golden_clip_convex() {
        // The segment-vs-line caveat, pinned: a square overlapping the clip
        // window's corner yields the reference's own partial result, and a
        // triangle poking outside collapses to EMPTY rather than clipping.
        assert_eq!(
            flat(&clip_convex(&square(), &pts(&[(4., 4.), (14., 4.), (14., 14.), (4., 14.)]))),
            [(4., 7.6), (10., 4.), (10., 10.), (4., 10.)]
        );
        assert!(
            clip_convex(&ensure_ccw(&tri()), &pts(&[(1., 1.), (6., 1.), (6., 6.), (1., 6.)]))
                .is_empty()
        );
        assert!(
            clip_convex(&square(), &pts(&[(50., 50.), (60., 50.), (60., 60.), (50., 60.)]))
                .is_empty()
        );
        // a fully-contained subject passes through untouched
        assert_eq!(
            flat(&clip_convex(&pts(&[(3., 3.), (6., 3.), (6., 6.), (3., 6.)]), &square())),
            [(3., 3.), (6., 3.), (6., 6.), (3., 6.)]
        );
    }

    #[test]
    fn golden_convex_hull() {
        assert_eq!(
            flat(&convex_hull(&pts(&[
                (0., 0.), (5., 1.), (10., 0.), (10., 10.),
                (5., 5.), (0., 10.), (2., 6.), (8., 3.)
            ]))),
            [(0., 0.), (10., 0.), (10., 10.), (0., 10.)]
        );
        // collinear input collapses to its two extremes
        assert_eq!(
            flat(&convex_hull(&pts(&[(0., 0.), (1., 1.), (2., 2.), (3., 3.)]))),
            [(0., 0.), (3., 3.)]
        );
        // fewer than three points short-circuits, returning the SORTED input
        assert_eq!(flat(&convex_hull(&pts(&[(1., 2.), (3., 4.)]))), [(1., 2.), (3., 4.)]);
        assert_eq!(
            flat(&convex_hull(&pts(&[(0., 0.), (0., 0.), (4., 0.), (4., 4.), (0., 4.), (2., 2.)]))),
            [(0., 0.), (4., 0.), (4., 4.), (0., 4.)]
        );
    }

    #[test]
    fn poly_self_intersects_matches_its_only_caller_s_needs() {
        // No golden path: `polySelfIntersects` is not on the reference's
        // `_test` export, so this is a real unit test of the ported logic,
        // documented as such (same precedent as territory/provinces).
        assert!(!poly_self_intersects(&square()));
        assert!(!poly_self_intersects(&l_shape()));
        assert!(poly_self_intersects(&pts(&[(0., 0.), (10., 10.), (10., 0.), (0., 10.)])));
        // adjacent edges share a vertex and must not count as a crossing
        assert!(!poly_self_intersects(&tri()));
    }

    #[test]
    fn vector_helpers_hold_the_reference_s_zero_length_convention() {
        // `V.norm` divides by `hypot||1`, so the zero vector maps to itself
        // instead of to NaN — several call sites depend on that, so it is
        // asserted rather than left implicit.
        let z = Vec2::default().norm();
        assert_eq!((z.x, z.y), (0.0, 0.0));
        assert_eq!(p(3., 4.).len(), 5.0);
        assert_eq!(p(1., 0.).rot90(), p(0., 1.));
        assert_eq!(p(1., 2.).cross(p(3., 4.)), -2.0);
        assert_eq!(p(1., 2.).dot(p(3., 4.)), 11.0);
        assert_eq!(p(0., 0.).lerp(p(10., 20.), 0.25), p(2.5, 5.0));
    }
}
