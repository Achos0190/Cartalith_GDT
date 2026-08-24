//! The FDLIBM family: the transcendentals V8 does **not** take from the
//! platform libm.
//!
//! `Math.exp`, `Math.sin`, `Math.cos`, `Math.log` and `Math.atan2` are all
//! "implementation-approximated" in ECMA-262, and V8 does not delegate any of
//! them. It ships its own FDLIBM port in `src/base/ieee754.cc`, so the target
//! for this port is that file, not whatever `f64::exp` happens to resolve to
//! on the host. Rust's own results differ from V8's on 2-10 % of ordinary
//! arguments (`JS_SEMANTICS_AUDIT.md` §1.1), always by one ulp, and one ulp
//! is enough: divergence #1's history is a one-ulp `hypot` turning a four-node
//! road graph into a three-node one, and §2.3's is a one-ulp `atan2` steering
//! a river into the wrong cell.
//!
//! Every function below is transliterated with its integer bit twiddling
//! intact and its constants quoted digit for digit, so it can be diffed
//! against the C by eye. That is the only defence these have against a silent
//! "simplification", which is why `clippy::excessive_precision` and
//! `clippy::approx_constant` are allowed rather than obeyed.
/// `Math.exp(x)`, with **V8's** result rather than the platform libm's.
///
/// The same finding as [`crate::js_hypot`], arriving at milestone 5 and measured the
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
/// [`crate::js_hypot`] (milestone 1) and [`js_exp`] (milestone 5), and it was found
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
/// The fifth divergence. It was ported by `cartalith-urban` milestone 6 ahead
/// of the milestone that first needed it, because `cartalith-urban`'s
/// `Substream::norm` is
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
/// `Math.atan2`, as V8 actually computes it.
///
/// **This is not `f64::atan2`.** V8 does not call the platform libm for
/// `Math.atan2`; it ships its own copy of FDLIBM's `__ieee754_atan2` in
/// `src/base/ieee754.cc`, the same way it ships `__ieee754_exp` (which is
/// why [`js_exp`] exists). Measured against `node`
/// v24.19.0 over 240 000 arguments drawn in the ranges this engine really
/// uses, `f64::atan2` returns a different double on **40 824** of them —
/// 17.0 %, and the largest single divergence in this workspace
/// (`JS_SEMANTICS_AUDIT.md` §1.1). This function returns V8's double on
/// **0** of the same 240 000.
///
/// It matters here because `build_channels` feeds the result into a
/// discrete argmax that picks the cell a river flows into, not into a
/// shaded value — see that function's own note.
///
/// Two details are load-bearing and easy to drop:
///
/// * the specification preamble (both signed zeros, each infinity
///   quadrant, NaN). `JS_SEMANTICS_AUDIT.md` §3.2 records three of four
///   `js_hypot` copies having lost exactly this, so `hypot(inf, 3)`
///   returned NaN. Every case ECMA-262 21.3.2.8 pins is covered by a test
///   below, with each expectation read off `node`.
/// * `m &= 1` in the `|y/x| > 2**60` branch. That line is the FreeBSD msun
///   correction V8 carries and the original 1993 Sun fdlibm does not;
///   without it this function disagrees with V8 on 777 of the same
///   240 000 arguments (all of them `x` tiny and negative, `y` large),
///   returning one ulp above `pi/2` where V8 returns `pi/2`.
///
/// `js_atan` is public alongside it because `Math.atan` is a JS function in
/// its own right, and the next caller that wants one should not have to
/// re-transcribe fdlibm to get it.
// The FDLIBM constants are transcribed verbatim from V8's own source;
// `PI_O_2` and friends are deliberately those literals and not
// `std::f64::consts::FRAC_PI_2`, and their digit counts are fdlibm's.
#[allow(clippy::approx_constant, clippy::excessive_precision)]
mod atan {
    const ATANHI: [f64; 4] = [
        4.63647609000806093515e-01, // atan(0.5)hi
        7.85398163397448278999e-01, // atan(1.0)hi
        9.82793723247329054082e-01, // atan(1.5)hi
        1.57079632679489655800e+00, // atan(inf)hi
    ];
    const ATANLO: [f64; 4] = [
        2.26987774529616870924e-17, // atan(0.5)lo
        3.06161699786838301793e-17, // atan(1.0)lo
        1.39033110312309984516e-17, // atan(1.5)lo
        6.12323399573676603587e-17, // atan(inf)lo
    ];
    const AT: [f64; 11] = [
        3.33333333333329318027e-01,
        -1.99999999998764832476e-01,
        1.42857142725034663711e-01,
        -1.11111104054623557880e-01,
        9.09088713343650656196e-02,
        -7.69187620504482999495e-02,
        6.66107313738753120669e-02,
        -5.83357013379057348645e-02,
        4.97687799461593236017e-02,
        -3.65315727442169155270e-02,
        1.62858201153657823623e-02,
    ];

    /// FDLIBM `atan` — `Math.atan(x)`. [`js_atan2`] reaches it for its primary
    /// branch, and V8 reaches it directly for `Math.atan`, so it is public in
    /// its own right here rather than private as it was in
    /// `cartalith-hydrology`, where nothing but `js_atan2` could see it.
    pub fn js_atan(x: f64) -> f64 {
        let hx = (x.to_bits() >> 32) as u32 as i32;
        let ix = hx & 0x7fff_ffff;
        if ix >= 0x4410_0000 {
            // |x| >= 2^66: atan(x) is pi/2 to the last bit (or x is NaN).
            let low = x.to_bits() as u32;
            if ix > 0x7ff0_0000 || (ix == 0x7ff0_0000 && low != 0) {
                return x + x;
            }
            return if hx > 0 { ATANHI[3] + ATANLO[3] } else { -ATANHI[3] - ATANLO[3] };
        }
        let mut x = x;
        let id: i32;
        if ix < 0x3fdc_0000 {
            // |x| < 0.4375 -- no argument reduction.
            if ix < 0x3e40_0000 {
                // |x| < 2^-27: atan(x) == x. (fdlibm's `huge+x>one` guard
                // exists only to raise the inexact flag, which JS cannot
                // observe.)
                return x;
            }
            id = -1;
        } else {
            x = x.abs();
            if ix < 0x3ff3_0000 {
                if ix < 0x3fe6_0000 {
                    // 7/16 <= |x| < 11/16
                    id = 0;
                    x = (2.0 * x - 1.0) / (2.0 + x);
                } else {
                    // 11/16 <= |x| < 19/16
                    id = 1;
                    x = (x - 1.0) / (x + 1.0);
                }
            } else if ix < 0x4003_8000 {
                // 19/16 <= |x| < 2.4375
                id = 2;
                x = (x - 1.5) / (1.0 + 1.5 * x);
            } else {
                // 2.4375 <= |x| < 2^66
                id = 3;
                x = -1.0 / x;
            }
        }
        let z = x * x;
        let w = z * z;
        // The odd/even split of sum(aT[i] * z^(i+1)) fdlibm uses; the
        // grouping is part of the result, not an optimisation
        // (`cartalith-rust-conventions`: do not reassociate).
        let s1 = z * (AT[0] + w * (AT[2] + w * (AT[4] + w * (AT[6] + w * (AT[8] + w * AT[10])))));
        let s2 = w * (AT[1] + w * (AT[3] + w * (AT[5] + w * (AT[7] + w * AT[9]))));
        if id < 0 {
            x - x * (s1 + s2)
        } else {
            let z = ATANHI[id as usize] - ((x * (s1 + s2) - ATANLO[id as usize]) - x);
            if hx < 0 { -z } else { z }
        }
    }

    const TINY: f64 = 1.0e-300;
    const PI_O_4: f64 = 7.8539816339744827900e-01;
    const PI_O_2: f64 = 1.5707963267948965580e+00;
    const PI: f64 = 3.1415926535897931160e+00;
    const PI_LO: f64 = 1.2246467991473531772e-16;

    /// `Math.atan2(y, x)` — argument order is JS's, i.e. the same as
    /// `y.atan2(x)`, so a call site converts by moving the receiver.
    pub fn js_atan2(y: f64, x: f64) -> f64 {
        let (xb, yb) = (x.to_bits(), y.to_bits());
        let hx = (xb >> 32) as u32 as i32;
        let lx = xb as u32;
        let hy = (yb >> 32) as u32 as i32;
        let ly = yb as u32;
        let ix = (hx & 0x7fff_ffff) as u32;
        let iy = (hy & 0x7fff_ffff) as u32;

        // x or y is NaN. `(l | -l) >> 31` is fdlibm's branch-free "low
        // word is nonzero", which pushes an infinity's high word past
        // 0x7ff00000 exactly when the mantissa is set.
        if (ix | ((lx | lx.wrapping_neg()) >> 31)) > 0x7ff0_0000
            || (iy | ((ly | ly.wrapping_neg()) >> 31)) > 0x7ff0_0000
        {
            return x + y;
        }
        if (hx.wrapping_sub(0x3ff0_0000) as u32 | lx) == 0 {
            return js_atan(y); // x == 1.0
        }
        // 2*sign(x) + sign(y), i.e. the quadrant.
        let mut m = ((hy >> 31) & 1) | ((hx >> 30) & 2);

        if (iy | ly) == 0 {
            return match m {
                0 | 1 => y, // atan2(+-0, +anything) = +-0
                2 => PI + TINY, // atan2(+0, -anything) = pi
                _ => -PI - TINY, // atan2(-0, -anything) = -pi
            };
        }
        if (ix | lx) == 0 {
            return if hy < 0 { -PI_O_2 - TINY } else { PI_O_2 + TINY };
        }
        if ix == 0x7ff0_0000 {
            if iy == 0x7ff0_0000 {
                return match m {
                    0 => PI_O_4 + TINY,
                    1 => -PI_O_4 - TINY,
                    2 => 3.0 * PI_O_4 + TINY,
                    _ => -3.0 * PI_O_4 - TINY,
                };
            }
            return match m {
                0 => 0.0,
                1 => -0.0,
                2 => PI + TINY,
                _ => -PI - TINY,
            };
        }
        if iy == 0x7ff0_0000 {
            return if hy < 0 { -PI_O_2 - TINY } else { PI_O_2 + TINY };
        }

        let k = (iy as i32 - ix as i32) >> 20;
        let z = if k > 60 {
            // |y/x| > 2^60. `m &= 1` is the FreeBSD msun correction V8
            // carries: without it the m=2/m=3 arms below re-add pi to a
            // value that is already +-pi/2, landing one ulp off V8.
            m &= 1;
            PI_O_2 + 0.5 * PI_LO
        } else if hx < 0 && k < -60 {
            0.0 // 0 > |y|/x > -2^60
        } else {
            js_atan((y / x).abs())
        };
        match m {
            0 => z,
            1 => -z,
            2 => PI - (z - PI_LO),
            _ => (z - PI_LO) - PI,
        }
    }
}

pub use atan::{js_atan, js_atan2};
