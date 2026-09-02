//! JS-semantics math: the operations where Rust's standard library and V8
//! disagree about what a floating-point expression means.
//!
//! # Why this crate exists
//!
//! `JS_SEMANTICS_AUDIT.md` catalogues **eight** measured divergences between
//! this port and `reference/Cartalith Gen1 v2.10.html`, each found by whichever
//! milestone happened to trip over it and each *after* the code had already
//! passed golden tests. By the time the audit ran, the JS-faithful replacements
//! had been written independently in **five** crates: seven copies of
//! `js_hypot`, seven of `js_round`, three of `js_min`/`js_max`, two of
//! `toFixed`, and one each of `js_exp`/`js_sin`/`js_cos`/`js_log`/`js_atan2`
//! that nothing outside their own crate could reach.
//!
//! The copies had already drifted apart in three measurable ways (audit §3),
//! and §2.2 showed that when two independent ports of one conversion exist, one
//! of them is wrong. This crate is the audit's recommendation #2, carried out:
//! **one implementation each, and the tests that pin it move with it.**
//!
//! # It is a leaf, deliberately
//!
//! `ARCHITECTURE.md` has dependencies running one way, in pipeline order. A
//! crate with *no* dependencies at all — not even a dev-dependency — is the
//! only shape that can reach all fifteen without disturbing that ordering:
//! `cartalith-urban` is allowed only `cartalith-rng`, and `cartalith-assets`
//! only `cartalith-io`/`-noise`, so neither can see `cartalith-spatial` or
//! `cartalith-hydrology`, where two of these helpers used to live. Nothing here
//! knows about `gdext`, pipeline state, or grids.
//!
//! # What "correct" means here
//!
//! Not "accurate" — *identical to V8*. Several of these functions are
//! measurably **worse** approximations than what Rust's standard library would
//! give, and that is the point (`cartalith-rust-conventions`: match the JS
//! engine, do not improve on it). Every expectation in this crate's tests was
//! read off `node` v24.19.0, never off a paraphrase of ECMA-262 — the audit
//! found a unit test that had been asserting a bug for two milestones because
//! it had been written the other way round.
//!
//! # The three copy disagreements the audit measured, and how they resolved
//!
//! 1. **`js_round`.** Six crates used `(x + 0.5).floor()`, which disagrees with
//!    V8 on exactly one double, `0.49999999999999994`. Consolidated onto
//!    [`js_round`]'s fractional-part form, which is V8's answer on that input
//!    and on every other. `cartalith-terrain`'s comment calling the additive
//!    form "the standard exact equivalent" is gone with the code.
//! 2. **`js_hypot`.** Three of four copies had lost the specification preamble,
//!    so `hypot(inf, 3)` returned NaN. The `js_atan2` fork repaired all three
//!    in place; the preamble is here, once, and [`js_hypot_n`] is the only
//!    compensated sum left in the workspace.
//! 3. **`js_min`/`js_max` on signed zero.** `Math.min(+0, -0)` is `-0` and
//!    `Math.max(+0, -0)` is `+0`, in **either** argument order. Every previous
//!    copy got one order right and the other wrong; see [`js_min`].

mod libm;

pub use libm::{js_acos, js_atan, js_atan2, js_cos, js_exp, js_log, js_log10, js_sin};


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
    js_hypot_n(&[x, y])
}

/// Three-argument `Math.hypot` — `renderHeightTileRGBA` is the reference's one
/// caller of the variadic form. A thin wrapper so it cannot drift from the
/// two-argument case, which is exactly what it was in `cartalith-terrain`.
#[inline]
pub fn js_hypot3(x: f64, y: f64, z: f64) -> f64 {
    js_hypot_n(&[x, y, z])
}

/// `Math.hypot(...vals)` for any argument count — the one compensated sum in
/// the workspace. [`js_hypot`] and [`js_hypot3`] are wrappers over it.
///
/// Arguments are taken with their signs; the magnitudes the algorithm needs are
/// computed here. (`cartalith-terrain`'s copy took pre-`abs`ed magnitudes and
/// its callers still pass them that way — `abs` of a magnitude is the identity,
/// so nothing moved.)
#[inline]
pub fn js_hypot_n(vals: &[f64]) -> f64 {
    // ECMA-262 21.3.2.18 pins this ahead of the scaling loop, and the loop
    // on its own gets both wrong: an infinite argument makes `v / max` a
    // NaN, and a NaN argument loses the `v > max` comparison so `max` stays
    // 0. `Math.hypot(inf, NaN)` is `inf`, not NaN -- infinity is checked
    // first. (`JS_SEMANTICS_AUDIT.md` §3.2 found this preamble missing from
    // three of the four copies of this function.)
    if vals.iter().any(|v| v.is_infinite()) {
        return f64::INFINITY;
    }
    if vals.iter().any(|v| v.is_nan()) {
        return f64::NAN;
    }
    let mut max = 0.0f64;
    for &v in vals {
        let a = v.abs();
        if a > max {
            max = a;
        }
    }
    if max == 0.0 {
        return 0.0;
    }
    // Kahan-compensated sum of (vi/max)^2, in V8's own argument order.
    let mut sum = 0.0f64;
    let mut compensation = 0.0f64;
    for &v in vals {
        let n = v.abs() / max;
        let summand = n * n - compensation;
        let preliminary = sum + summand;
        compensation = (preliminary - sum) - summand;
        sum = preliminary;
    }
    max * sum.sqrt()
}

/// `Math.min(a, b)`, with JS semantics rather than Rust's.
///
/// The difference that matters: **JS propagates NaN, Rust absorbs it.**
/// `Math.min(0.70, NaN)` is `NaN`; `f64::min(0.70, NaN)` is `0.70`.
///
/// **Signed zero, the audit's third copy disagreement (§3.3), now pinned.**
/// `Math.min(+0, -0)` is `-0` and `Math.max(+0, -0)` is `+0`, in **either**
/// argument order — for these two functions alone, `-0` counts as strictly
/// smaller than `+0`.
/// A plain `<` cannot see that (`-0.0 < 0.0` is false), so every previous copy
/// got one order right and the other wrong: `cartalith-urban`/`-civ`'s
/// `if b < a { b } else { a }` answered `min(-0, +0)` correctly and
/// `min(+0, -0)` wrongly; `cartalith-terrain::amplify`'s `if a < b { a } else
/// { b }` did the reverse. The `is_sign_negative` arm below is the four lines
/// that make both orders V8's. Expectations read off `node`.
///
/// It is still unobservable in the engine — no live site reads the sign of a
/// zero — but a single implementation that is right in one argument order and
/// wrong in the other is not a thing to keep once there is only one of it.
pub fn js_min(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        f64::NAN
    } else if a == 0.0 && b == 0.0 {
        // Both zeros: -0 wins, whichever side it is on.
        if a.is_sign_negative() { a } else { b }
    } else if b < a {
        b
    } else {
        a
    }
}

/// `Math.max(a, b)`, with JS semantics. See [`js_min`] for the NaN rule and
/// the signed-zero rule.
pub fn js_max(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        f64::NAN
    } else if a == 0.0 && b == 0.0 {
        // Both zeros: +0 wins, whichever side it is on.
        if a.is_sign_negative() { b } else { a }
    } else if b > a {
        b
    } else {
        a
    }
}

/// The reference's own `smoothstep(a, b, x)` (reference HTML line 7569):
/// `t = clamp01((x - a) / ((b - a) || 1e-6)); return t*t*(3 - 2*t)`.
///
/// Not a V8-vs-Rust divergence like the rest of this crate, but the same
/// *problem*: four independent ports of one reference function, with **three
/// different answers** for a degenerate band. `||` is JS truthiness, so the
/// `1e-6` substitutes for `0`, `-0` **and** `NaN` — and only
/// `cartalith-terrain::sculpt`'s copy said so. `cartalith-climate`'s and
/// `cartalith-godot::render`'s guarded `== 0.0` and let a NaN width through;
/// `cartalith-civ`'s had no guard at all, so a zero-width band divided by zero
/// there — which the clamp absorbs to `0`/`1` on either side but not *at* the
/// band, where `0/0` is NaN and the reference's `1e-6` ramp is `0`. No live
/// call site reaches any of that — every one of them passes
/// constant literal bounds — which is exactly the position §3.2's `js_hypot`
/// and §3.3's `js_min` were in when they were consolidated here, and the same
/// reason it is safe to do: no golden can move, and there is no longer a copy
/// that can silently lose the rule.
///
/// `cartalith-terrain::sculpt::cliff` genuinely reaches `b - a == 0`
/// (`smoothstep(-transW, transW, sd)` with `transW == 0`), so the guard is not
/// a defensive flourish.
pub fn smoothstep(a: f64, b: f64, x: f64) -> f64 {
    let d = b - a;
    let d = if d == 0.0 || d.is_nan() { 1e-6 } else { d };
    let t = ((x - a) / d).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
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
/// **The form matters, and this is the one the audit measured.** Six crates
/// carried `(x + 0.5).floor()` instead, and a sweep of 3 million random values
/// plus every double within 3 ulp of every half-integer in +-50 found exactly
/// one input where the two differ: `x = 0.49999999999999994`, the largest
/// double below `0.5`, where `x + 0.5` rounds up to exactly `1.0` and the floor
/// gives `1` where V8 gives `0` (`JS_SEMANTICS_AUDIT.md` §3.1). Comparing the
/// fractional part gets that case and the negative-tie case both right, so it
/// is what all seven former call sites now share.
///
/// The one difference left is `-0`: `Math.round(-0.5)` is `-0` and this returns
/// `+0`. Every call site in the workspace either indexes a raster with the
/// result or clamps it, so the sign of a zero is not observable from any of
/// them.
pub fn js_round(x: f64) -> f64 {
    if !x.is_finite() {
        return x;
    }
    let f = x.floor();
    if x - f >= 0.5 { f + 1.0 } else { f }
}

/// JS `x || 0`. **`NaN` is falsy in JS**, so `NaN || 0` is `0` -- not `NaN`,
/// which is what a plain Rust read of the same field would give. The
/// reference guards every per-place number this way (`p.pop||0`,
/// `p.tradeVolume||0`, `p.economicImportance||0`, and both sides of
/// `_civFactionCapital`'s comparison), and the effect is real: a `NaN`
/// population is absorbed at the place, so it can never reach the power
/// clamp downstream. Dropping this would let one bad settlement turn a
/// whole faction's aggregate row into `NaN`s the reference never produces.
pub fn js_num_or_zero(x: f64) -> f64 {
    if x.is_nan() { 0.0 } else { x }
}

/// JS truthiness of a number used as a divide-by guard (`maxPop ? a/maxPop
/// : 0`): `0`, `-0` and `NaN` are all falsy. `x != 0.0` alone would take the
/// true branch on `NaN` and divide by it.
pub fn js_truthy_num(x: f64) -> bool {
    x != 0.0 && !x.is_nan()
}

/// JS `Number.prototype.toFixed`: "pick the larger n" on a tie, i.e. round
/// half away from zero, where Rust's `{:.N}` rounds half to even. Only ever
/// used for `cartalith-civ`'s verdict/blocked-message text, but those strings
/// are compared against the reference's own output by that crate's goldens, so
/// the tie-breaking rule has to match.
///
/// The tie is decided on the value's **exact** decimal expansion, never by
/// scaling. Milestone 6 found the earlier `(v*10^d + 0.5).floor()` form
/// fabricating ties: `61.5/30` is `2.0499999999999998`, which JS renders
/// `"2.0"`, but `2.0499999999999998 * 10` rounds to exactly `20.5` in `f64` and
/// the `+0.5` then carried it to `"2.1"`. Rust's own `{:.N}` already prints the
/// correctly-rounded exact decimal, so all this has to do is spot a genuine tie
/// and step it away from zero instead of to even.
pub fn js_fixed(v: f64, digits: u32) -> String {
    if !v.is_finite() {
        // JS spells these `"NaN"`, `"Infinity"` and `"-Infinity"`; Rust's `{}`
        // spells the last two `inf` and `-inf`. This crate's whole job is the
        // former, so it is written out rather than delegated. Found by the
        // mutation sweep: dropping the guard entirely left a `.expect()` that
        // panics on an infinity and no test noticed, and writing the test the
        // sweep asked for is what exposed the spelling.
        return if v.is_nan() {
            "NaN".to_string()
        } else if v > 0.0 {
            "Infinity".to_string()
        } else {
            "-Infinity".to_string()
        };
    }
    let d = digits as usize;
    // A double is a dyadic rational, so its decimal expansion terminates; a tie
    // at place `d+1` therefore means digit `d+1` is 5 and the expansion ends
    // there. Eighteen guard digits is far past the point where two neighbouring
    // doubles could both look like one.
    let exact = format!("{:.*}", d + 18, v.abs());
    let frac = &exact[exact.find('.').expect("a fractional part was requested") + 1..];
    let tie = frac.as_bytes()[d] == b'5' && frac.as_bytes()[d + 1..].iter().all(|&b| b == b'0');
    if !tie {
        return format!("{:.*}", d, v);
    }
    // Away from zero: increment the last kept decimal digit, carrying.
    let dot = exact.find('.').expect("checked above");
    let mut kept: Vec<u8> = exact[..dot].bytes().chain(frac[..d].bytes()).collect();
    let mut i = kept.len();
    loop {
        if i == 0 {
            kept.insert(0, b'1');
            break;
        }
        i -= 1;
        if kept[i] == b'9' {
            kept[i] = b'0';
        } else {
            kept[i] += 1;
            break;
        }
    }
    let split = kept.len() - d;
    let int_part = String::from_utf8(kept[..split].to_vec()).expect("ascii digits");
    let frac_part = String::from_utf8(kept[split..].to_vec()).expect("ascii digits");
    let sign = if v < 0.0 { "-" } else { "" };
    if d == 0 { format!("{sign}{int_part}") } else { format!("{sign}{int_part}.{frac_part}") }
}

/// `Number.prototype.toFixed(d)` followed by unary `+` — round `v` to `d`
/// decimal places and read the result back as a number.
///
/// **Not** `format!("{:.*}", d, v)`. Rust rounds a decimal tie to even; ECMA-262
/// picks *"the larger n"*, i.e. half-up toward `+∞`. The two disagree whenever
/// the exact binary value of `v` lands on a tie at digit `d + 1`, and that is
/// reachable here rather than theoretical: a map 800 km wide on a 12 800-cell
/// grid has `cell_km == 0.0625`, so the very first cell's easting is `0.0625`,
/// an exact tie at three decimals. JS answers `0.063`; `{:.3}` answers `0.062`.
///
/// The exact decimal expansion of any finite `f64` terminates (it is a dyadic
/// rational), and for a tie to survive past 30 fractional digits the value
/// would need ~26 more fractional bits that are all zero — impossible. So
/// formatting to 30 places and rounding the *string* is exact, not an
/// approximation.
///
/// # Two rounding bugs the `JS_SEMANTICS_AUDIT.md` sweep found here
///
/// Both were in one expression, `round_up = first > 5 || (tie && !neg)`.
///
/// 1. **A first dropped digit of `5` with a nonzero tail rounded *down*.**
///    `9.051` came out `9.0` where V8 gives `9.1`, and `286.4957967118851`
///    came out `286.49` where V8 gives `286.50`. This is not a last-place
///    nicety: it is a whole unit in the last kept place, and it fires on
///    roughly one value in ten, because it only needs the first dropped digit
///    to be `5` and anything after it to be nonzero. Every GeoJSON coordinate
///    and every way length passes through here.
/// 2. **A tie on a negative value rounded toward zero.** ECMA-262 21.1.3.3
///    strips the sign at step 6 and picks "the larger n" against the
///    *magnitude*, so `(-0.0625).toFixed(3)` is `"-0.063"`, not `"-0.062"` —
///    the same direction as the positive tie, not the mirror of it.
///
/// Neither could be caught by `golden_parity_geojson.rs`, and the reason is
/// worth keeping: its world is 600 km over 12 cells, so `cell_km` is exactly
/// `50` and **every coordinate it rounds is already an integer**. Its one
/// deliberately-fractional value, a way of `38.4567` km, has `6` as its first
/// dropped digit — the branch that was correct. The fixture reaches the
/// function on every feature and still cannot see either bug.
///
/// [`js_fixed`], written independently for the same conversion in
/// `cartalith-civ`, has neither bug; 60 000 differential cases against V8 agree
/// with it exactly. Both are kept: one returns the string `toFixed` produces
/// and the other the number `+x.toFixed(d)` coerces to, and a test below runs
/// them against each other across the whole rounding surface.
pub fn js_to_fixed(v: f64, d: usize) -> f64 {
    if !v.is_finite() {
        return v;
    }
    let neg = v.is_sign_negative();
    let s = format!("{:.*}", 30, v.abs());
    let dot = s.find('.').expect("30 decimals always writes a point");
    let digits: Vec<u8> = s.bytes().filter(|b| b.is_ascii_digit()).map(|b| b - b'0').collect();
    let int_len = dot;
    let keep = int_len + d; // how many digits survive
    // Round the MAGNITUDE to nearest, ties away from zero -- which is what
    // ECMA-262 21.1.3.3's "pick the larger n" comes to once step 6 has stripped
    // the sign ("If x < 0, then set s to \"-\" and x to -x"). `n` is chosen
    // against |x| and the "-" is re-attached afterwards, so both halves of the
    // rule are decided on the magnitude:
    //
    // - remainder > 1/2 (first dropped digit > 5, or == 5 with any nonzero
    //   digit after it) -> up;
    // - remainder == 1/2 exactly (first dropped digit 5, all zeroes after)
    //   -> up as well, because the tie goes away from zero on both signs;
    // - remainder < 1/2 -> down.
    //
    // Those three collapse to `first >= 5`, with no special case for the tie
    // and none for the sign.
    let mut out = digits[..keep].to_vec();
    let rest = &digits[keep..];
    let first = rest.first().copied().unwrap_or(0);
    let round_up = first >= 5;
    if round_up {
        let mut i = keep;
        loop {
            if i == 0 {
                out.insert(0, 1);
                break;
            }
            i -= 1;
            if out[i] == 9 {
                out[i] = 0;
            } else {
                out[i] += 1;
                break;
            }
        }
    }
    let split = out.len() - d;
    let mut t = String::with_capacity(out.len() + 2);
    if neg {
        t.push('-');
    }
    for (i, dg) in out.iter().enumerate() {
        if i == split && d > 0 {
            t.push('.');
        }
        t.push((b'0' + dg) as char);
    }
    t.parse().expect("re-parsing our own decimal")
}

/// ECMA-262 `ToUint8Clamp` — what storing into a `Uint8ClampedArray` does.
///
/// Clamp to `[0, 255]`, NaN to `0`, then round to nearest with **ties to
/// even**. Not `as u8` (which truncates) and not `round()` (which breaks ties
/// away from zero).
#[inline]
pub fn u8_clamped(v: f64) -> u8 {
    if v.is_nan() || v <= 0.0 {
        return 0;
    }
    if v >= 255.0 {
        return 255;
    }
    let f = v.floor();
    if f + 0.5 < v {
        (f + 1.0) as u8
    } else if v < f + 0.5 {
        f as u8
    } else if (f as u64) % 2 == 1 {
        (f + 1.0) as u8
    } else {
        f as u8
    }
}


#[cfg(test)]
mod tests {
    use super::*;

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

        // `cartalith-rng`'s `Mulberry32`, inlined. This crate takes no
        // dependency at all (see the crate docs), and this is the reference's
        // own four-line generator; the hashes below are the proof it is the
        // same stream, since they were captured against the reference driving
        // its own `mulberry32` over the identical argument sequence.
        struct Mulberry32 {
            state: u32,
        }
        impl Mulberry32 {
            fn new(seed: u32) -> Self {
                Mulberry32 { state: seed }
            }
            fn next_f64(&mut self) -> f64 {
                self.state = self.state.wrapping_add(0x6D2B79F5);
                let mut t = self.state;
                t = (t ^ (t >> 15)).wrapping_mul(t | 1);
                t = t.wrapping_add((t ^ (t >> 7)).wrapping_mul(t | 61)) ^ t;
                ((t ^ (t >> 14)) as f64) / 4294967296.0
            }
        }

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


    /// The **bulk** goldens for `js_exp` and `js_atan2`, the two functions this
    /// catalogue had only ever pinned with hand-picked rows.
    ///
    /// This test exists because of a mutation result, exactly as the
    /// `sin`/`cos`/`log` hash above does. The first sweep of the consolidated
    /// crate left **206 of 439 mutants alive**, and **101 of them were inside
    /// `js_exp` and `js_atan`/`js_atan2`** — every reduction threshold, both
    /// `|y/x|` shortcuts, the whole `id` table of `atan`'s argument reduction.
    /// Those two arrived at milestones that predated the hash technique, so
    /// they had a dozen discriminating rows each, and `cartalith-urban`
    /// milestone 6 already measured what a dozen rows are worth against branchy
    /// bit manipulation: nothing. The bands below are chosen to enter each
    /// branch on purpose.
    ///
    /// **`exp`:** below `2^-28` (the `1 + x` shortcut), inside `0.5 ln2` (the
    /// `k == 0` path), across `0.5 ln2` and `1.5 ln2` (both reduction forms),
    /// the ordinary range, across both overflow and underflow thresholds, deep
    /// into the subnormal scale where the `2^-1000` rescale fires, just below
    /// the overflow threshold, and hugging `1.0` where V8 and FDLIBM disagree.
    ///
    /// **`atan2`:** one band per fdlibm `atan` reduction interval — `< 2^-27`,
    /// `< 7/16`, `[7/16, 11/16)`, `[11/16, 19/16)`, `[19/16, 2.4375)`,
    /// `[2.4375, 2^66)` — plus `|y/x| > 2^60` (the `m &= 1` branch the audit
    /// found missing from the first transcription) and `< -2^60`, and a band
    /// hugging a ratio of exactly 1. Each draw also picks a scale and one of
    /// the four sign quadrants, so every arm of the final `match m` is entered
    /// on every band.
    #[test]
    fn golden_js_exp_and_js_atan2_hash_over_every_reduction_branch() {
        // `cartalith-rng`'s `Mulberry32`, inlined -- see the crate docs on why
        // this crate takes no dependency, even a dev one.
        struct M(u32);
        impl M {
            fn f(&mut self) -> f64 {
                self.0 = self.0.wrapping_add(0x6D2B79F5);
                let mut t = self.0;
                t = (t ^ (t >> 15)).wrapping_mul(t | 1);
                t = t.wrapping_add((t ^ (t >> 7)).wrapping_mul(t | 61)) ^ t;
                ((t ^ (t >> 14)) as f64) / 4294967296.0
            }
        }
        fn fold(mut h: u32, x: f64) -> u32 {
            for b in x.to_le_bytes() {
                h ^= u32::from(b);
                h = h.wrapping_mul(0x0100_0193);
            }
            h
        }
        const N: usize = 6000;
        const FNV_OFFSET: u32 = 0x811c_9dc5;

        let exp_bands: [fn(f64) -> f64; 8] = [
            |u| (u - 0.5) * 1e-9,
            |u| (u - 0.5) * 0.69,
            |u| (u - 0.5) * 2.0,
            |u| (u - 0.5) * 40.0,
            |u| (u - 0.5) * 1400.0,
            |u| -(u * 745.0),
            |u| u * 709.0,
            |u| 1.0 + (u - 0.5) * 1e-12,
        ];
        let ratio_bands: [fn(f64) -> f64; 9] = [
            |u| u * 1e-9,
            |u| u * 0.4,
            |u| 0.4375 + u * 0.25,
            |u| 0.6875 + u * 0.5,
            |u| 1.1875 + u * 1.25,
            |u| 2.4375 + u * 1000.0,
            |u| 2.0f64.powi(67) * (1.0 + u),
            |u| 2.0f64.powi(-70) * (1.0 + u),
            |u| 1.0 + (u - 0.5) * 1e-9,
        ];

        // exp
        let mut r = M(0x00e7_9911);
        let (mut h, mut n, mut finite) = (FNV_OFFSET, 0usize, 0usize);
        for band in exp_bands {
            for _ in 0..N {
                let v = js_exp(band(r.f()));
                h = fold(h, v);
                n += 1;
                if v.is_finite() {
                    finite += 1;
                }
            }
        }
        assert_eq!(n, 48_000, "exp: wrong argument count");
        assert_eq!(finite, 48_000, "exp: {finite}/{n} finite");
        assert_ne!(h, FNV_OFFSET, "exp: the hash never advanced");
        assert_eq!(h, 0xe2b9_d14c, "exp: V8 hash over {n} arguments");

        // atan2
        let mut r = M(0x00a7_a422);
        let (mut h, mut n, mut finite) = (FNV_OFFSET, 0usize, 0usize);
        for band in ratio_bands {
            for _ in 0..N {
                let ratio = band(r.f());
                let s = 0.5 + r.f() * 9.5;
                let q = (r.f() * 4.0).floor() as u32;
                let x = s * if q & 1 != 0 { -1.0 } else { 1.0 };
                let y = ratio * s * if q & 2 != 0 { -1.0 } else { 1.0 };
                let v = js_atan2(y, x);
                h = fold(h, v);
                n += 1;
                if v.is_finite() {
                    finite += 1;
                }
            }
        }
        assert_eq!(n, 54_000, "atan2: wrong argument count");
        assert_eq!(finite, 54_000, "atan2: {finite}/{n} finite");
        assert_ne!(h, FNV_OFFSET, "atan2: the hash never advanced");
        assert_eq!(h, 0x17c9_ce4e, "atan2: V8 hash over {n} arguments");
    }

    /// Non-finite inputs to both `toFixed` ports, and the `u8_clamped` rows a
    /// mutation sweep showed the table above was missing.
    ///
    /// `js_to_fixed_passes_non_finite_through` came from `cartalith-spatial`;
    /// its `js_fixed` half is new, and the sweep is what asked for it — dropping
    /// either guard leaves a `.expect()` that panics on an infinity, and nothing
    /// noticed. The `u8_clamped` rows close the one real gap the sweep found in
    /// that function: **every value in the table had an even floor**, so
    /// inverting the round-down comparison fell through to the ties-to-even arm
    /// and produced the same answer. `1.2` and `3.1` have odd floors and do not.
    #[test]
    fn non_finite_to_fixed_and_the_odd_floor_rounding_the_sweep_found_untested() {
        assert!(js_to_fixed(f64::NAN, 3).is_nan());
        assert_eq!(js_to_fixed(f64::INFINITY, 3), f64::INFINITY);
        assert_eq!(js_to_fixed(f64::NEG_INFINITY, 3), f64::NEG_INFINITY);
        // `node`: NaN.toFixed(3) === "NaN", (1/0).toFixed(3) === "Infinity",
        // (-1/0).toFixed(3) === "-Infinity" -- and Rust's `{}` would have
        // written the last two `inf` and `-inf`, which is the divergence this
        // row exists to pin.
        assert_eq!(js_fixed(f64::NAN, 3), "NaN");
        assert_eq!(js_fixed(f64::INFINITY, 3), "Infinity");
        assert_eq!(js_fixed(f64::NEG_INFINITY, 3), "-Infinity");

        // Odd floor, fraction below the midpoint: must round DOWN, not to even.
        assert_eq!(u8_clamped(1.2), 1);
        assert_eq!(u8_clamped(3.1), 3);
        assert_eq!(u8_clamped(1.9), 2);
        // ...and the exact midpoint just above, which the `f + 0.5` comparison
        // decides rather than the tie rule.
        assert_eq!(u8_clamped(0.5000000000000001), 1);
        assert_eq!(u8_clamped(2.5000000000000004), 3);
    }

    /// `Math.hypot`'s specification preamble (ECMA-262 21.3.2.18), which the
    /// bare scaling loop gets wrong in both directions: an infinite argument
    /// makes `v / max` a NaN, and a NaN argument loses the `v > max`
    /// comparison so `max` stays 0.
    ///
    /// Moved here unchanged from `cartalith-civ`, `cartalith-assets` and
    /// `cartalith-terrain`, which each grew an identical copy of it when the
    /// `js_atan2` fork repaired the three copies that had lost the preamble
    /// (`JS_SEMANTICS_AUDIT.md` §3.2). Every expectation is `node` v24.19.0's
    /// own output. There is one implementation now, so there is one test.
    #[test]
    fn js_hypot_follows_the_spec_on_infinity_and_nan() {
        assert_eq!(js_hypot(f64::INFINITY, 3.0), f64::INFINITY);
        assert_eq!(js_hypot(3.0, f64::INFINITY), f64::INFINITY);
        assert_eq!(js_hypot(f64::NEG_INFINITY, 3.0), f64::INFINITY);
        // Infinity wins over NaN, in either argument order.
        assert_eq!(js_hypot(f64::INFINITY, f64::NAN), f64::INFINITY);
        assert_eq!(js_hypot(f64::NAN, f64::INFINITY), f64::INFINITY);
        assert!(js_hypot(f64::NAN, 0.0).is_nan());
        assert!(js_hypot(0.0, f64::NAN).is_nan());
        assert!(js_hypot(f64::NAN, f64::NAN).is_nan());
        // The ordinary path is unchanged: V8's one-ulp-high 3root2.
        assert_eq!(js_hypot(0.0, 0.0), 0.0);
        assert_eq!(js_hypot(3.0, 3.0), 4.242640687119286);
        // The variadic and three-argument forms inherit the same guard --
        // `JS_SEMANTICS_AUDIT.md` §3.2 names `js_hypot3` as the entry point a
        // two-argument-only fix would have missed.
        assert_eq!(js_hypot3(f64::INFINITY, 3.0, f64::NAN), f64::INFINITY);
        assert!(js_hypot3(1.0, f64::NAN, 2.0).is_nan());
        assert_eq!(js_hypot_n(&[]), 0.0);
        assert_eq!(js_hypot_n(&[3.0, 3.0]), js_hypot(3.0, 3.0));
        // Signs are taken here, not by the caller: the old `cartalith-terrain`
        // `js_hypot_n` required pre-`abs`ed magnitudes.
        assert_eq!(js_hypot_n(&[-3.0, 3.0]), js_hypot(3.0, 3.0));
        assert_eq!(js_hypot3(-2.0, -3.0, -6.0), 7.0);
    }

    /// `Math.hypot` on exactly-representable Pythagorean answers, moved from
    /// `cartalith-terrain::sculpt` and `::tile_render` (which tested the two-
    /// and three-argument forms separately against the same rule).
    #[test]
    fn js_hypot_matches_the_pythagorean_answer_on_exact_cases() {
        assert_eq!(js_hypot(3.0, 4.0), 5.0);
        assert_eq!(js_hypot(0.0, 0.0), 0.0);
        assert_eq!(js_hypot(-3.0, 4.0), 5.0);
        assert_eq!(js_hypot(5.0, 0.0), 5.0);
        assert_eq!(js_hypot3(2.0, 3.0, 6.0), 7.0);
        assert_eq!(js_hypot3(0.0, 0.0, 0.0), 0.0);
        assert_eq!(js_hypot3(1.0, 4.0, 8.0), 9.0);
        assert_eq!(js_hypot3(0.0, 3.0, 4.0), 5.0);
    }

    /// NaN propagation, the audit's divergence #3. `f64::min`/`f64::max`
    /// *absorb* a NaN and return the other operand; `Math.min`/`Math.max`
    /// propagate it. Moved from `cartalith-civ` and
    /// `cartalith-terrain::amplify`, which each tested the same rule.
    #[test]
    fn js_min_max_propagate_nan_where_rusts_own_would_not() {
        assert!(js_min(1.0, f64::NAN).is_nan());
        assert!(js_min(f64::NAN, 1.0).is_nan());
        assert!(js_max(0.0, f64::NAN).is_nan());
        assert!(js_max(f64::NAN, 0.0).is_nan());
        assert!(f64::min(1.0, f64::NAN) == 1.0, "this is the Rust behaviour being avoided");
        assert!(f64::max(1.0, f64::NAN) == 1.0, "and this one");
        assert_eq!(js_min(1.0, 0.5), 0.5);
        assert_eq!(js_min(0.5, 1.0), 0.5);
        assert_eq!(js_max(0.0, 3.0), 3.0);
        assert_eq!(js_max(3.0, 0.0), 3.0);
        assert!(js_truthy_num(1.0) && !js_truthy_num(0.0) && !js_truthy_num(-0.0) && !js_truthy_num(f64::NAN));
        assert_eq!(js_num_or_zero(f64::NAN), 0.0);
        assert_eq!(js_num_or_zero(-3.5), -3.5);
    }

    /// The `||1e-6` is JS truthiness, so it substitutes for a zero width, a
    /// negative-zero width **and** a NaN width. Of the four copies this
    /// function replaced, one had the whole rule, two had only the `== 0.0`
    /// half, and one had no guard at all — so this asserts the degenerate
    /// cases explicitly and asserts that the unguarded form really did produce
    /// something else, which is what stops a future copy-paste losing it again.
    #[test]
    fn smoothstep_substitutes_1e_6_for_a_zero_or_nan_width_the_way_js_truthiness_does() {
        // The ordinary band: endpoints pinned, midpoint at the cubic's centre.
        assert_eq!(smoothstep(0.0, 1.0, -1.0), 0.0);
        assert_eq!(smoothstep(0.0, 1.0, 2.0), 1.0);
        assert_eq!(smoothstep(0.0, 1.0, 0.5), 0.5);
        // Descending bands are legal and used (`smoothstep(1.0, -6.0, t)`).
        assert_eq!(smoothstep(1.0, -6.0, 2.0), 0.0);
        assert_eq!(smoothstep(1.0, -6.0, -7.0), 1.0);

        // Zero width: `1e-6` makes it a near-step, not a division by zero.
        assert_eq!(smoothstep(0.0, 0.0, 1.0), 1.0);
        assert_eq!(smoothstep(0.0, 0.0, -1.0), 0.0);
        // The one input where the no-guard-at-all copy really did differ:
        // exactly *at* a zero-width band it computed `0/0`.
        assert_eq!(smoothstep(0.0, 0.0, 0.0), 0.0);
        assert!(((0.0f64 - 0.0) / (0.0 - 0.0)).is_nan(), "what the unguarded copy computed there");
        // -0 width is falsy in JS too, and `-0.0 - 0.0 == -0.0`, which `== 0.0`
        // does catch -- so this one the `== 0.0` copies also got right.
        assert_eq!(smoothstep(0.0, -0.0, 1.0), 1.0);
        // A NaN width is falsy in JS; the two `== 0.0` copies let it through
        // and returned NaN. (Both endpoints NaN is NaN either way -- `x - a`
        // poisons `t` before the width is ever consulted.)
        assert_eq!(smoothstep(0.0, f64::NAN, 1.0), 1.0);
        assert_eq!(smoothstep(0.0, f64::NAN, -1.0), 0.0);
        assert!(((1.0 - 0.0) / (f64::NAN - 0.0)).is_nan(), "what the `== 0.0`-only copies computed there");
        assert!(smoothstep(f64::NAN, f64::NAN, 1.0).is_nan(), "and NaN in really is NaN out");
    }

    /// The audit's third copy disagreement (§3.3), resolved.
    ///
    /// `Math.min(+0, -0)` is `-0` and `Math.max(+0, -0)` is `+0` — in **both**
    /// argument orders. Every previous copy got exactly one order right, and
    /// which one depended on whether it was written `if b < a` or `if a < b`.
    /// The eight expectations below are `node` v24.19.0's, read with
    /// `Object.is(x, -0)` so a `-0` cannot be mistaken for a `+0` in the
    /// printing.
    #[test]
    fn js_min_max_pick_the_v8_signed_zero_in_either_argument_order() {
        let neg = |x: f64| x == 0.0 && x.is_sign_negative();
        let pos = |x: f64| x == 0.0 && x.is_sign_positive();

        assert!(neg(js_min(0.0, -0.0)), "Math.min(+0, -0) is -0");
        assert!(neg(js_min(-0.0, 0.0)), "...in the other order too");
        assert!(pos(js_max(0.0, -0.0)), "Math.max(+0, -0) is +0");
        assert!(pos(js_max(-0.0, 0.0)), "...in the other order too");
        assert!(neg(js_min(-0.0, -0.0)));
        assert!(neg(js_max(-0.0, -0.0)));
        assert!(pos(js_min(0.0, 0.0)));
        assert!(pos(js_max(0.0, 0.0)));
        // A zero against a nonzero still goes by magnitude, sign preserved.
        assert!(neg(js_min(-0.0, 5.0)));
        assert!(neg(js_min(2.0, -0.0)));
        assert!(neg(js_max(-0.0, -5.0)));

        // The two forms this replaces, each shown failing one order.
        let urban = |a: f64, b: f64| if b < a { b } else { a };
        let terrain = |a: f64, b: f64| if a < b { a } else { b };
        assert!(pos(urban(0.0, -0.0)), "the -urban/-civ form answered +0 here");
        assert!(pos(terrain(-0.0, 0.0)), "the -terrain form answered +0 there");
    }

    /// ECMA-262 `ToUint8Clamp` — what storing into a `Uint8ClampedArray` does,
    /// which is neither `as u8` (truncation) nor `.round()` (ties away from
    /// zero). Moved unchanged from `cartalith-terrain::tile_render`, whose
    /// module doc records that a naive `as u8` costs a whole colour level.
    #[test]
    fn u8_clamped_rounds_ties_to_even_and_clamps_both_ends() {
        assert_eq!(u8_clamped(0.5), 0, "tie -> even");
        assert_eq!(u8_clamped(1.5), 2, "tie -> even");
        assert_eq!(u8_clamped(2.5), 2, "tie -> even");
        assert_eq!(u8_clamped(3.5), 4);
        assert_eq!(u8_clamped(0.4999), 0);
        assert_eq!(u8_clamped(0.5001), 1);
        assert_eq!(u8_clamped(127.5), 128);
        assert_eq!(u8_clamped(128.5), 128);
        assert_eq!(u8_clamped(-1.0), 0);
        assert_eq!(u8_clamped(-0.4), 0);
        assert_eq!(u8_clamped(255.4), 255);
        assert_eq!(u8_clamped(300.0), 255);
        assert_eq!(u8_clamped(f64::NAN), 0);
        assert_eq!(u8_clamped(254.5), 254);
        // The two conversions this is not.
        assert_ne!(u8_clamped(2.5), 2.5f64.round() as u8);
        assert_ne!(u8_clamped(1.5), 1.5f64 as u8);
    }

    /// The two `toFixed` ports agree with each other across the whole rounding
    /// surface.
    ///
    /// `js_fixed` (from `cartalith-civ`) returns the string; `js_to_fixed`
    /// (from `cartalith-spatial::geo`) returns the `+`-coerced number. They
    /// were written independently for the same conversion, `JS_SEMANTICS_AUDIT.md`
    /// §3.4 found one of them wrong, and both have since been repaired — so
    /// now that they sit in one file, running them against each other is free
    /// and is exactly the check that would have caught either bug.
    ///
    /// 200 000 values over five bands, plus the audit's own named
    /// counterexamples. A disagreement here means one of the two has drifted.
    #[test]
    fn the_two_to_fixed_ports_agree_with_each_other_everywhere() {
        struct M(u32);
        impl M {
            fn f(&mut self) -> f64 {
                self.0 = self.0.wrapping_add(0x6D2B79F5);
                let mut t = self.0;
                t = (t ^ (t >> 15)).wrapping_mul(t | 1);
                t = t.wrapping_add((t ^ (t >> 7)).wrapping_mul(t | 61)) ^ t;
                ((t ^ (t >> 14)) as f64) / 4294967296.0
            }
        }
        let bands: [fn(f64) -> f64; 5] = [
            |u| u,
            |u| (u - 0.5) * 2000.0,
            // Values that land on or beside a decimal tie, which is the branch
            // both bugs lived in.
            |u| (u * 1e6).floor() / 16.0,
            |u| (u * 1e4).floor() / 1000.0,
            |u| (u - 0.5) * 1e-3,
        ];
        let mut r = M(0x00c0_ffee);
        let mut n = 0usize;
        for band in bands {
            for _ in 0..10_000 {
                let v = band(r.f());
                for d in 0..4usize {
                    let s = js_fixed(v, d as u32);
                    let want: f64 = s.parse().expect("js_fixed emits a number");
                    assert_eq!(
                        js_to_fixed(v, d),
                        want,
                        "js_to_fixed({v:?}, {d}) vs js_fixed -> {s:?}"
                    );
                    n += 1;
                }
            }
        }
        assert_eq!(n, 200_000, "the sweep did not run");

        // `JS_SEMANTICS_AUDIT.md` §2.2's own rows, expectations off `node`.
        assert_eq!(js_to_fixed(9.051, 1), 9.1);
        assert_eq!(js_to_fixed(286.4957967118851, 2), 286.5);
        assert_eq!(js_to_fixed(-0.0625, 3), -0.063);
        assert_eq!(js_fixed(-0.0625, 3), "-0.063");
        assert_eq!(js_fixed(9.051, 1), "9.1");
        // ...and the two that look like counterexamples and are not, because
        // the nearest double to each decimal is BELOW it.
        assert_eq!(js_to_fixed(0.12345, 3), 0.123);
        assert_eq!(js_to_fixed(0.15, 1), 0.1);
        assert_eq!(js_fixed(0.15, 1), "0.1");
        // Milestone 6's fabricated tie: 61.5/30 is 2.0499999999999998, and
        // `(v*10 + 0.5).floor()` carried it to "2.1".
        assert_eq!(js_fixed(61.5 / 30.0, 1), "2.0");
        assert_eq!(js_to_fixed(61.5 / 30.0, 1), 2.0);
    }

    /// Ordinary arguments, chosen to cross every branch of FDLIBM's
    /// `atan` argument reduction (|x| < 2^-27, < 0.4375, the three
    /// reduced intervals, >= 2^66) and every quadrant of `atan2`.
    #[test]
    fn js_atan2_matches_v8_on_every_branch() {
        let cases: [(f64, f64, u64); 44] = [
            (1.0, 1.0, 0x3fe921fb54442d18),
            (1.0, -1.0, 0x4002d97c7f3321d2),
            (-1.0, 1.0, 0xbfe921fb54442d18),
            (-1.0, -1.0, 0xc002d97c7f3321d2),
            (3.0, 4.0, 0x3fe4978fa3269ee1),
            (-3.0, 4.0, 0xbfe4978fa3269ee1),
            (3.0, -4.0, 0x4003fc176b7a8560),
            (-3.0, -4.0, 0xc003fc176b7a8560),
            (0.5, 0.4375, 0x3feb434ee31013fc),
            (0.5, 1.1875, 0x3fd9816449b6fd53),
            (0.5, 2.4375, 0x3fc9e5acd4944285),
            (-0.5, 0.4375, 0xbfeb434ee31013fc),
            (1.5, -1.0, 0x400145385fa3af72),
            (1.5, 2.4375, 0x3fe1a72859945683),
            (3.0, -2.0, 0x400145385fa3af72),
            (3.0, 2.4375, 0x3fec6e6d2171bf19),
            (std::f64::consts::PI, 1.0, 0x3ff433b8a322ddd2),
            (std::f64::consts::PI, -1.0, 0x3ffe103e05657c5f),
            (std::f64::consts::PI, -0.5, 0x3ffba87553cd6603),
            (-std::f64::consts::PI, 2.4375, 0xbfed266448697762),
            (1e-9, 1.0, 0x3e112e0be826d695),
            (1.0, 1e-9, 0x3ff921fb53ff74e9),
            (2.0, 1.0, 0x3ff1b6e192ebbe44),
            (1.0, 2.0, 0x3fddac670561bb4f),
            (7.0, 1.0, 0x3ff6dcc57bb565fd),
            (1.0, 7.0, 0x3fc229aec47638dc),
            // |y/x| > 2^60 with x negative: the `m &= 1` branch. Drop that
            // line and these four come back one ulp high.
            (0.0625, -1e-300, 0x3ff921fb54442d18),
            (1e300, -1e-300, 0x3ff921fb54442d18),
            (-1e300, -1e-300, 0xbff921fb54442d18),
            (1e300, 1e-300, 0x3ff921fb54442d18),
            (-1e300, 1e-300, 0xbff921fb54442d18),
            // subnormal and near-subnormal operands
            (1e-300, -2.2250738585072014e-308, 0x3ff921fb5a3d3c3b),
            (5e-324, 2.0, 0x0000000000000000),
            (-5e-324, 2.0, 0x8000000000000000),
            (2.2250738585072014e-308, -5e-324, 0x3ff921fb54442d1a),
            (1.0, 1e-320, 0x3ff921fb54442d18),
            (-1.0, 1e-320, 0xbff921fb54442d18),
            // |y|/x < -2^60: the `z = 0.0` branch
            (1e-300, 1e300, 0x0000000000000000),
            (-1e-300, 1e300, 0x8000000000000000),
            (1e-300, -1e300, 0x400921fb54442d18),
            (-1e-300, -1e300, 0xc00921fb54442d18),
            (123456789.5, -1e-8, 0x3ff921fb54442d19),
            (-123456789.5, -1e-8, 0xbff921fb54442d19),
            // x == 1.0 exactly: fdlibm short-circuits straight to atan(y)
            (0.4374999999999999, 1.0, 0x3fda64eec3cc23fb),
        ];
        for (y, x, want) in cases {
            let got = js_atan2(y, x);
            assert_eq!(
                got.to_bits(),
                want,
                "js_atan2({y:e}, {x:e}) = {got:e} ({:#018x}), V8 gives {:#018x}",
                got.to_bits(),
                want
            );
        }
    }

    /// The cases ECMA-262 21.3.2.8 pins by name: both signed zeros in
    /// every combination, an infinity in each quadrant, and NaN.
    ///
    /// `JS_SEMANTICS_AUDIT.md` §3.2 found three of the four `js_hypot`
    /// copies had quietly lost exactly this preamble, so `hypot(inf, 3)`
    /// returned NaN. This test is here so the same cannot happen to
    /// `js_atan2` unnoticed.
    #[test]
    fn js_atan2_matches_v8_on_the_spec_pinned_edge_cases() {
        let cases: [(f64, f64, u64); 26] = [
            (0.0, 0.0, 0x0000000000000000),
            (0.0, -0.0, 0x400921fb54442d18),
            (-0.0, 0.0, 0x8000000000000000),
            (-0.0, -0.0, 0xc00921fb54442d18),
            (0.0, 1.0, 0x0000000000000000),
            (0.0, -1.0, 0x400921fb54442d18),
            (-0.0, 1.0, 0x8000000000000000),
            (-0.0, -1.0, 0xc00921fb54442d18),
            (1.0, 0.0, 0x3ff921fb54442d18),
            (-1.0, 0.0, 0xbff921fb54442d18),
            (1.0, -0.0, 0x3ff921fb54442d18),
            (-1.0, -0.0, 0xbff921fb54442d18),
            (f64::INFINITY, f64::INFINITY, 0x3fe921fb54442d18),
            (f64::INFINITY, f64::NEG_INFINITY, 0x4002d97c7f3321d2),
            (f64::NEG_INFINITY, f64::INFINITY, 0xbfe921fb54442d18),
            (f64::NEG_INFINITY, f64::NEG_INFINITY, 0xc002d97c7f3321d2),
            (f64::INFINITY, 1.0, 0x3ff921fb54442d18),
            (f64::INFINITY, -1.0, 0x3ff921fb54442d18),
            (f64::NEG_INFINITY, 1.0, 0xbff921fb54442d18),
            (f64::NEG_INFINITY, -1.0, 0xbff921fb54442d18),
            (1.0, f64::INFINITY, 0x0000000000000000),
            (1.0, f64::NEG_INFINITY, 0x400921fb54442d18),
            (-1.0, f64::INFINITY, 0x8000000000000000),
            (-1.0, f64::NEG_INFINITY, 0xc00921fb54442d18),
            (0.0, f64::INFINITY, 0x0000000000000000),
            (-0.0, f64::NEG_INFINITY, 0xc00921fb54442d18),
        ];
        for (y, x, want) in cases {
            let got = js_atan2(y, x);
            assert_eq!(got.to_bits(), want, "js_atan2({y}, {x}) bits");
        }
        // The signed zeros above are load-bearing, not decoration: this is
        // the exact branch `build_channels` reaches on a left-right
        // symmetric cell, where `gx` is `0.0` and `-gx` is `-0.0`.
        // `x` is a zero of either sign and `y` is not, so this is the
        // `(ix|lx) == 0` branch: the answer is +-pi/2 and the sign comes
        // from `y` alone, never from the zero's sign.
        assert_eq!(js_atan2(-0.25, -0.0).to_bits(), 0xbff921fb54442d18);
        assert_eq!(js_atan2(0.25, -0.0).to_bits(), 0x3ff921fb54442d18);
        assert_eq!(js_atan2(-0.25, 0.0).to_bits(), 0xbff921fb54442d18);
        assert_eq!(js_atan2(0.25, 0.0).to_bits(), 0x3ff921fb54442d18);

        // JS has one observable NaN, so these are compared as `is_nan`.
        for (y, x) in [
            (f64::NAN, 1.0),
            (1.0, f64::NAN),
            (f64::NAN, f64::NAN),
            (f64::NAN, f64::INFINITY),
            (f64::INFINITY, f64::NAN),
            (f64::NAN, 0.0),
            (0.0, f64::NAN),
        ] {
            assert!(js_atan2(y, x).is_nan(), "js_atan2({y}, {x}) should be NaN");
        }
    }
}
