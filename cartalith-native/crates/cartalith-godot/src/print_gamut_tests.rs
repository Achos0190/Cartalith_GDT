//! Tests for `print_gamut.rs` (ET-E3). Kept in their own file so the module
//! stays readable; compiled only under `cfg(test)` from the module's own
//! `#[path]` declaration.

use super::*;

/// Encoded sRGB of a linear value (IEC 61966-2-1), for building test colours
/// from [`InkModel`]'s linear output.
fn oetf(l: f64) -> f64 {
    if l <= 0.003_130_8 { 12.92 * l } else { 1.055 * l.powf(1.0 / 2.4) - 0.055 }
}

/// The test engine: the shipped synthetic profile, built once per test.
fn eng() -> &'static Engine {
    engine().expect("the synthetic CMYK profile builds, parses and transforms")
}

/// Protects: the CIEDE2000 implementation against G. Sharma, W. Wu,
/// E. N. Dalal (2005), Color Res. Appl. 30(1), Table 1's supplementary test
/// data -- the pairs that exercise the hue-mean wrap, the zero-chroma rule and
/// the rotation term -- to the table's four decimals. An independent
/// published expectation, not a value computed by the code under test.
#[test]
fn ciede2000_sharma_pairs() {
    let pairs: [([f64; 3], [f64; 3], f64); 12] = [
        ([50.0, 2.6772, -79.7751], [50.0, 0.0, -82.7485], 2.0425),
        ([50.0, 3.1571, -77.2803], [50.0, 0.0, -82.7485], 2.8615),
        ([50.0, 2.8361, -74.0200], [50.0, 0.0, -82.7485], 3.4412),
        ([50.0, -1.3802, -84.2814], [50.0, 0.0, -82.7485], 1.0000),
        ([50.0, 0.0, 0.0], [50.0, -1.0, 2.0], 2.3669),
        ([50.0, 2.4900, -0.0010], [50.0, -2.4900, 0.0009], 7.1792),
        ([50.0, 2.5, 0.0], [73.0, 25.0, -18.0], 27.1492),
        ([50.0, 2.5, 0.0], [61.0, -5.0, 29.0], 22.8977),
        ([50.0, 2.5, 0.0], [56.0, -27.0, -3.0], 31.9030),
        ([50.0, 2.5, 0.0], [58.0, 24.0, 15.0], 19.4535),
        ([60.2574, -34.0099, 36.2677], [60.4626, -34.1751, 39.4387], 1.2644),
        ([90.8027, -2.0831, 1.4410], [91.1528, -1.6435, 0.0447], 1.4441),
    ];
    for (a, b, want) in pairs {
        let got = ciede2000(a, b);
        assert!((got - want).abs() < 5e-5, "{a:?} vs {b:?}: got {got:.6}, Sharma 2005 gives {want}");
        // The formula is symmetric in its arguments (Sharma 2005, §"Mathematical observations").
        assert!((ciede2000(b, a) - want).abs() < 5e-5);
    }
}

/// Protects: the two Lindbloom matrices being typed correctly -- their
/// product must be the identity to the 7 places they are quoted to.
#[test]
fn matrices_are_inverse() {
    for r in 0..3 {
        for c in 0..3 {
            let v: f64 = (0..3).map(|k| XYZ_D50_TO_SRGB[r][k] * SRGB_TO_XYZ_D50[k][c]).sum();
            let want = if r == c { 1.0 } else { 0.0 };
            assert!((v - want).abs() < 1e-6, "[{r}][{c}] = {v}");
        }
    }
}

/// Protects: the shipped profile is a real ICC CMYK output profile that
/// `moxcms` parses back -- class `prtr`, data space CMYK, PCS Lab, both
/// relative-colorimetric LUTs present, and the description that says it is
/// synthetic. Reads the encoded bytes, not the struct that wrote them.
#[test]
fn synthetic_profile_parses_as_cmyk_output() {
    let icc = synthetic_cmyk_icc().expect("encodes");
    assert_eq!(&icc[36..40], b"acsp", "ICC magic at byte 36");
    assert_eq!(&icc[12..16], b"prtr", "output device class");
    assert_eq!(&icc[16..20], b"CMYK", "data colour space");
    assert_eq!(&icc[20..24], b"Lab ", "PCS");
    let p = ColorProfile::new_from_slice(&icc).expect("moxcms parses its own encoding");
    assert_eq!(p.color_space, DataColorSpace::Cmyk);
    assert_eq!(p.profile_class, ProfileClass::OutputDevice);
    assert!(p.lut_a_to_b_colorimetric.is_some() && p.lut_b_to_a_colorimetric.is_some());
    let desc = format!("{:?}", p.description);
    assert!(desc.contains("not a press characterisation"), "{desc}");
}

/// Protects: "never in gamut for not checked" at the engine layer -- bytes
/// that are not a CMYK profile build no engine (so `check` has nothing to
/// answer with), rather than a permissive one.
#[test]
fn non_cmyk_profile_builds_no_engine() {
    assert!(Engine::from_icc(b"not an icc profile", "x").is_err());
    let srgb = ColorProfile::new_srgb().encode().unwrap();
    let e = Engine::from_icc(&srgb, "x").err().expect("an RGB profile is refused");
    assert!(e.contains("not a CMYK"), "{e}");
    // A grey profile too: the guard is "CMYK only", not "anything but RGB".
    let grey = ColorProfile::new_gray_with_gamma(2.2).encode().unwrap();
    let e = Engine::from_icc(&grey, "x").err().expect("a grey profile is refused");
    assert!(e.contains("not a CMYK"), "{e}");
}

/// Every separation on a `n`^3 grid of the GCR parameters, as encoded sRGB:
/// colours the synthetic profile prints by construction.
fn printable_colours(n: usize) -> Vec<[f64; 3]> {
    let model = InkModel::new();
    let mut out = Vec::new();
    for i in 0..n {
        for j in 0..n {
            for k in 0..n {
                let d = (n - 1) as f64;
                let rgb = model.linear_rgb(separate([i as f64 / d, j as f64 / d, k as f64 / d]));
                out.push(rgb.map(oetf));
            }
        }
    }
    out
}

/// Protects: [`IN_GAMUT_MAX_DE00`] stays above the profile's own measured
/// round-trip noise on printable colours (the measurement its doc cites --
/// run with `--nocapture` to print it). If a grid or model change raises the
/// floor past the threshold this goes red, instead of the picker flagging
/// colours the profile prints.
#[test]
fn noise_floor_measurement() {
    let e = eng();
    let mut des: Vec<f64> = printable_colours(21).into_iter().map(|c| e.check(c).unwrap().delta_e).collect();
    des.sort_by(f64::total_cmp);
    let q = |p: f64| des[((des.len() - 1) as f64 * p).round() as usize];
    let max = *des.last().unwrap();
    println!(
        "print_gamut noise floor over {} printable colours: median {:.3}, p95 {:.3}, p99 {:.3}, max {:.3} dE00",
        des.len(),
        q(0.5),
        q(0.95),
        q(0.99),
        max
    );
    assert!(max < IN_GAMUT_MAX_DE00, "noise floor max {max:.3} reaches the threshold");
}

/// Protects: the scope's first literal acceptance -- sRGB (128, 128, 128)
/// is in gamut against the shipped profile. With `<=` inverted in
/// `Engine::check` this goes red.
#[test]
fn neutral_grey_is_in_gamut() {
    let g = eng().check([128.0 / 255.0; 3]).unwrap();
    println!("grey 128: dE00 {:.4}", g.delta_e);
    assert!(g.in_gamut, "grey dE00 {}", g.delta_e);
    assert_eq!(g.source, "own_icc");
}

/// Protects: the scope's second literal acceptance -- saturated sRGB blue
/// (0, 0, 255) is flagged. With `<=` inverted in `Engine::check` this goes
/// red. Confirmed against the profile before pinning (its round trip is
/// printed with `--nocapture`).
#[test]
fn saturated_blue_is_flagged() {
    let b = eng().check([0.0, 0.0, 1.0]).unwrap();
    println!("blue: dE00 {:.4}, nearest {:?}", b.delta_e, b.nearest);
    assert!(!b.in_gamut, "blue dE00 {}", b.delta_e);
    assert!(b.delta_e > 5.0 * IN_GAMUT_MAX_DE00, "blue is far outside, not marginal: {}", b.delta_e);
}

/// Protects: the swatch the picker offers is itself printable -- `nearest`
/// for out-of-gamut and in-gamut colours alike checks back in gamut.
#[test]
fn nearest_is_in_gamut() {
    let e = eng();
    let mut flagged = 0;
    let n = 6;
    for i in 0..n {
        for j in 0..n {
            for k in 0..n {
                let d = (n - 1) as f64;
                let c = [i as f64 / d, j as f64 / d, k as f64 / d];
                let g = e.check(c).unwrap();
                if !g.in_gamut {
                    flagged += 1;
                }
                let again = e.check(g.nearest).unwrap();
                assert!(again.in_gamut, "{c:?} -> nearest {:?} rechecks at dE00 {}", g.nearest, again.delta_e);
            }
        }
    }
    // Positive control: the sample must contain out-of-gamut colours, or
    // this test would pass on a profile that never clips.
    assert!(flagged > 20, "only {flagged} of {} sampled colours flagged", n * n * n);
}

/// Protects: the 300 % total-ink limit holds in what the separation emits,
/// and genuinely binds (positive control: the unlimited sum at the black
/// corner exceeds it, so the limit is not vacuous).
#[test]
fn total_ink_limit_holds_and_binds() {
    let unlimited = 3.0 * (1.0 - UCR) + 1.0;
    assert!(unlimited > TOTAL_INK_LIMIT, "premise: the black corner needs limiting");
    let s = separate([1.0, 1.0, 1.0]);
    let sum: f64 = s.iter().sum();
    assert!((sum - 3.0).abs() < 1e-12, "black corner total {sum}");
    for i in 0..=10 {
        for j in 0..=10 {
            for k in 0..=10 {
                let s = separate([i as f64 / 10.0, j as f64 / 10.0, k as f64 / 10.0]);
                assert!(s.iter().sum::<f64>() <= 3.0 + 1e-12);
            }
        }
    }
}

/// Protects: the B2A a*/b* input shaper reaching the parsed profile (the
/// workaround for `moxcms` 0.8.1's zero-entry-curve reader, see
/// `identity_curve`). With a zero-entry identity written first, the shaper
/// parses back as an empty curve and this goes red. 32768 is
/// `round(0.5 * 65535)`: a* = 0 lands on the middle grid node.
#[test]
fn shaper_survives_encoding() {
    let icc = synthetic_cmyk_icc().unwrap();
    let p = ColorProfile::new_from_slice(&icc).unwrap();
    let Some(LutWarehouse::Multidimensional(m)) = &p.lut_b_to_a_colorimetric else {
        panic!("B2A1 is an mBA");
    };
    assert_eq!(m.b_curves.len(), 3);
    for c in &m.b_curves[1..] {
        let ToneReprCurve::Lut(v) = c else { panic!("a sampled curve") };
        assert_eq!(v.len(), 256);
        assert_eq!(v[128], 32768);
    }
}
