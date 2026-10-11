//! The print (CMYK) gamut check behind the colour picker's warning --
//! `EDITING_TOOLS_SCOPE.md` milestone ET-E3, under Ruling CB item 6
//! (`LARGE_ITEM_RULINGS.md`: *"the owner's own ICC profile plus a labelled
//! generic fallback, CIEDE2000 with the threshold measured from the profile's
//! noise"*).
//!
//! # What it does
//!
//! [`check`] takes an sRGB colour, converts it sRGB -> CMYK -> sRGB through a
//! CMYK ICC profile with `moxcms` (relative colorimetric, the scope's intent),
//! and reports how far the round trip moved it (CIEDE2000, [`ciede2000`]),
//! whether that is within the profile's own noise ([`IN_GAMUT_MAX_DE00`]), and
//! the round-tripped colour itself as the **nearest printable** colour.
//!
//! # Which profile
//!
//! **This build ships one profile, [`synthetic_cmyk_icc`], generated here.**
//! It is a real ICC v4 output-class (`prtr`) CMYK profile -- `mAB` A2B0/A2B1
//! and `mBA` B2A0/B2A1 tags over a Lab PCS -- encoded by `moxcms`' own writer
//! and read back by `moxcms`' own parser before any transform is built, so the
//! check runs through exactly the ICC machinery a press profile would. What is
//! synthetic is the **data**: the colorimetry comes from [`InkModel`], a
//! documented analytic ink model (four inks, Yule-Nielsen spread, a flare
//! floor, GCR and a 300 % total-ink limit), not from measured press patches.
//!
//! **It is not FOGRA39, SWOP, GRACoL or any ISO 12647 characterisation, and
//! must never be described as one.** Its licence is this project's: every
//! byte is produced by code in this file. The dictionary carries
//! `source` = [`PROFILE_SOURCE_OWN_ICC`] and the picker's tooltip labels it,
//! so the warning reads as "against a generic coated-paper approximation",
//! never as a press proof. No analytic (no-ICC) fallback was needed: `moxcms`
//! writes, parses and transforms through this profile.
//!
//! **Not built here: loading the owner's own press `.icc`.** Ruling CB item 6
//! reads, at its plainest, as *the owner loads a press profile, with this one
//! as the labelled fallback*. This milestone's brief named one `#[func]`;
//! a loader is a second binding plus a file dialog, and is left to the
//! coordinator (reported as a doc note, not silently dropped). The engine is
//! shaped for it: [`Engine::from_icc`] takes any CMYK ICC bytes.
//!
//! # Never "in gamut" for "not checked"
//!
//! [`check`] returns `None` when no profile could be built or no transform
//! could be made; the `#[func]` (`WorldGen::color_print_gamut`) then returns
//! an **empty** dictionary, and the picker draws no glyph. It never returns
//! `in_gamut: true` for a colour it did not convert.
//!
//! # A property of the synthetic profile worth knowing
//!
//! [`InkModel`] multiplies per-channel transmittances in **linear sRGB**, each
//! in [0, 1], so every colour it can print is inside sRGB by construction. A
//! real press profile prints some cyans and greens sRGB cannot show; this one
//! cannot. So `nearest` here never needs clipping, which would not hold for a
//! loaded press profile (the clamp in [`Engine::round_trip`] exists for that).

use std::sync::{Arc, OnceLock};

use moxcms::{
    ColorProfile, DataColorSpace, Layout, LutMultidimensionalType, LutStore, LutWarehouse, Matrix3d, ProfileClass,
    ProfileText, RenderingIntent, ToneReprCurve, TransformF64Executor, TransformOptions, Vector3d, Xyzd,
};

/// The `source` value for a check made against [`synthetic_cmyk_icc`]:
/// this project's **own ICC profile** (the brief's vocabulary: `"own_icc"`
/// for a profile run through ICC machinery, `"generic"` reserved for an
/// analytic no-ICC fallback, which this build does not need). It is still a
/// synthesised profile, not a press characterisation, and the picker's
/// tooltip says so.
pub const PROFILE_SOURCE_OWN_ICC: &str = "own_icc";

/// The profile description written into the synthetic profile's `desc` tag.
/// Says what it is in the file itself, so an exported copy cannot be mistaken
/// for a press profile.
pub const PROFILE_DESCRIPTION: &str =
    "Cartalith generic coated CMYK (synthetic ink model, 300% TIL, GCR) - not a press characterisation";

/// Maximum total ink (C+M+Y+K, each 0..1), as a fraction: 3.0 = 300 %.
/// **Source: the brief and the scope's OQ-2 option (b)** ("300 % total ink"),
/// which is the common coated-offset limit range (300-330 %); a labelled
/// judgement within it, not a measured press value.
pub const TOTAL_INK_LIMIT: f64 = 3.0;

/// The Yule-Nielsen *n* factor of the halftone spread model
/// (`T(x) = (1 - x + x * T_solid^(1/n))^n`; Yule & Nielsen 1951, "The
/// penetration of light into paper and its effect on halftone reproduction").
/// **A labelled judgement**: 2.0 sits in the 1.5-2.5 range coated-paper fits
/// commonly report; it bends the tone curve to model dot gain. No press was
/// measured.
const YULE_NIELSEN_N: f64 = 2.0;

/// Surface (first-surface) reflection floor, as a fraction of paper white:
/// the darkest any ink stack can get, `R = FLARE + (1 - FLARE) * product`.
/// **A labelled judgement**: 0.01 puts the 300 % rich black near L* 9, the
/// lightness a coated-offset rich black typically measures; without it the
/// model's black would be unphysically deep (L* < 1).
const FLARE: f64 = 0.01;

/// How far the black generated by GCR is removed from C, M and Y
/// (`C = c' - UCR * K`). **A labelled judgement**: 0.25 keeps some CMY under
/// K (a richer black) so the 300 % limit genuinely binds near the black
/// corner, as it does on a press, instead of never being reached.
const UCR: f64 = 0.25;

/// The four inks' solid colours (C, M, Y, K order) as CIE L*a*b* (D50).
/// **A labelled judgement shaped to resemble published coated-offset solids**
/// (cyan ~ 55/-37/-50, magenta ~ 48/74/-3, yellow ~ 89/-5/93, black ~ 16/0/0
/// are the round figures widely quoted for ISO 12647-2 coated paper); used
/// only to give the synthetic model plausible hues. Not a characterisation
/// and not copied from any profile's data.
const SOLIDS_LAB: [[f64; 3]; 4] = [[55.0, -37.0, -50.0], [48.0, 74.0, -3.0], [89.0, -5.0, 93.0], [16.0, 0.0, 0.0]];

/// Smallest per-channel solid transmittance allowed. A solid whose Lab falls
/// outside sRGB (the cyan and yellow above do) has a negative linear channel;
/// it is clamped to this rather than zero so overprints stay invertible.
/// **A labelled judgement**: 0.002 linear is about L* 2 on its own.
const MIN_TRANSMITTANCE: f64 = 0.002;

/// A2B CLUT grid points per CMYK axis. 11^4 = 14 641 nodes. **A labelled
/// judgement**: real coated profiles commonly use 9-17 here; the model is
/// smooth, and [`IN_GAMUT_MAX_DE00`] measures the result whatever it is.
const A2B_GRID: u8 = 11;

/// B2A CLUT grid points per Lab axis. 33^3 = 35 937 nodes, the size common in
/// press profiles' B2A tags. **A labelled judgement**, measured through
/// [`IN_GAMUT_MAX_DE00`] like the A2B grid.
const B2A_GRID: u8 = 33;

/// Points per axis of the coarse table that seeds [`invert`]. **A labelled
/// judgement for speed**: with the warm start, 9 finds the same minima as
/// 17 did (the noise-floor test measures the result) at a fraction of the
/// scan.
const COARSE: usize = 9;

/// The ICC PCS white, D50 (ICC.1:2010 §7.2.16, `0.9642, 1.0, 0.8249`).
const D50: [f64; 3] = [0.9642, 1.0, 0.8249];

/// Linear sRGB -> CIE XYZ (D50), the IEC 61966-2-1 primaries Bradford-adapted
/// from D65 to D50. **Source:** B. Lindbloom's published "sRGB to XYZ (D50)"
/// matrix (brucelindbloom.com, RGB/XYZ matrices), quoted to 7 places.
const SRGB_TO_XYZ_D50: [[f64; 3]; 3] = [
    [0.436_074_7, 0.385_064_9, 0.143_080_4],
    [0.222_504_5, 0.716_878_6, 0.060_616_9],
    [0.013_932_2, 0.097_104_5, 0.714_173_3],
];

/// The inverse of [`SRGB_TO_XYZ_D50`], same source (Lindbloom's "XYZ (D50)
/// to sRGB" matrix). `matrices_are_inverse` tests the pair.
const XYZ_D50_TO_SRGB: [[f64; 3]; 3] = [
    [3.133_856_1, -1.616_866_7, -0.490_614_6],
    [-0.978_768_4, 1.916_141_5, 0.033_454_0],
    [0.071_945_3, -0.228_991_4, 1.405_242_7],
];

/// The largest CIEDE2000 round-trip error a colour can show and still count
/// as **in gamut** for the synthetic profile.
///
/// **Measured, not adopted** (PhotoCraft's 3.0 and 4.0 are deliberately not
/// used, per the scope). `noise_floor_measurement` (a `#[test]`, run it with
/// `--nocapture` to print) round-trips colours the profile can print by
/// construction -- the [`InkModel`] output of every separation on a 21^3
/// grid of the GCR parameterisation, 9 261 colours -- through the full
/// sRGB -> CMYK -> sRGB path. Measured 2026-10-12 with this file's grids:
/// **median 0.28, p99 1.26, maximum 2.41 ΔE00**. The maximum is the 300 %
/// rich black itself (the gamut narrows to a point there and the B2A grid's
/// neighbouring nodes are clipped); the next worst is a near-white at 1.99,
/// and 9 of the 9 261 exceed 1.6. Most of the floor is interpolation:
/// `moxcms` resamples the pair into its own 33^3 (RGB -> CMYK) and 17^4
/// (CMYK -> RGB) grids (`lut_transforms.rs`, `GRID_SIZE`) on top of the
/// profile's CLUTs.
///
/// **The threshold is that maximum rounded up to the next half unit, 2.5** --
/// a labelled judgement on top of the measurement, so no colour the profile
/// prints is ever flagged, while saturated sRGB blue (0, 0, 255) measures
/// 16.3 and is flagged with a wide margin. The cost, stated: a colour less
/// than ~2.5 ΔE00 outside the gamut is not flagged, because it cannot be told
/// apart from the profile's own noise. The test pins the measured maximum
/// below this value, so a grid or model change that raises the floor turns
/// it red rather than silently flagging printable colours.
pub const IN_GAMUT_MAX_DE00: f64 = 2.5;

// ---------------------------------------------------------------------------
// The synthetic ink model
// ---------------------------------------------------------------------------

/// A four-ink subtractive model in linear sRGB: each ink is a per-channel
/// transmittance at full coverage, partial coverage follows Yule-Nielsen,
/// and inks multiply. Paper white is linear (1, 1, 1) (relative colorimetric:
/// the medium is the white), floored by [`FLARE`].
///
/// Exists so the shipped profile is generated by documented code rather than
/// carried as an opaque blob; never use it as a model of a real press.
struct InkModel {
    /// Solid transmittance per ink (C, M, Y, K), per linear sRGB channel.
    solid: [[f64; 3]; 4],
}

impl InkModel {
    /// Builds the model from [`SOLIDS_LAB`]: each solid's Lab -> linear sRGB,
    /// clamped to [[`MIN_TRANSMITTANCE`], 1].
    fn new() -> Self {
        let mut solid = [[0.0; 3]; 4];
        for (i, lab) in SOLIDS_LAB.iter().enumerate() {
            let rgb = mul3(&XYZ_D50_TO_SRGB, lab_to_xyz_d50(*lab));
            for c in 0..3 {
                solid[i][c] = rgb[c].clamp(MIN_TRANSMITTANCE, 1.0);
            }
        }
        Self { solid }
    }

    /// Linear sRGB reflectance of a CMYK coverage (each 0..1).
    fn linear_rgb(&self, cmyk: [f64; 4]) -> [f64; 3] {
        let mut out = [1.0; 3];
        for (i, &x) in cmyk.iter().enumerate() {
            let x = x.clamp(0.0, 1.0);
            for c in 0..3 {
                let t = (1.0 - x + x * self.solid[i][c].powf(1.0 / YULE_NIELSEN_N)).powf(YULE_NIELSEN_N);
                out[c] *= t;
            }
        }
        out.map(|v| FLARE + (1.0 - FLARE) * v)
    }

    /// CIE Lab (D50) of a CMYK coverage.
    fn lab(&self, cmyk: [f64; 4]) -> [f64; 3] {
        linear_srgb_to_lab(self.linear_rgb(cmyk))
    }
}

/// The separation: three GCR parameters (c', m', y', each 0..1) to the CMYK
/// the profile prints. Black is generated from the grey component
/// (`K = min(c', m', y')^2`, a labelled-judgement GCR curve: none in
/// highlights, heavy in shadows), [`UCR`] of it is removed from C, M and Y,
/// and C, M, Y are then scaled down so the total never exceeds
/// [`TOTAL_INK_LIMIT`].
fn separate(p: [f64; 3]) -> [f64; 4] {
    let g = p[0].min(p[1]).min(p[2]);
    let k = g * g;
    let mut cmy = p.map(|v| (v - UCR * k).max(0.0));
    let sum: f64 = cmy.iter().sum();
    if sum + k > TOTAL_INK_LIMIT && sum > 0.0 {
        let s = (TOTAL_INK_LIMIT - k) / sum;
        cmy = cmy.map(|v| v * s);
    }
    [cmy[0], cmy[1], cmy[2], k]
}

/// The B2A inversion for one Lab target: the separation whose printed Lab is
/// nearest (Euclidean in Lab, i.e. ΔE76 -- minimum-distance clipping, the
/// usual relative-colorimetric gamut clip) to `target`, returned as the GCR
/// parameters (see [`separate`]). The nearest entry of a coarse table, or
/// `warm` (the neighbouring grid node's answer) when that is nearer, seeds a
/// shrinking pattern search over the three parameters. The warm start exists
/// for speed: with the 9^3 coarse table it found the same B2A table as a
/// 17^3 table did (identical noise-floor figures) and a debug-build
/// generation fell from 7.2 s to 2.8 s (single runs on this machine,
/// 2026-10-12, an order of magnitude only).
fn invert(model: &InkModel, coarse: &[([f64; 3], [f64; 3])], target: [f64; 3], warm: Option<[f64; 3]>) -> [f64; 3] {
    let d2 = |a: [f64; 3]| (a[0] - target[0]).powi(2) + (a[1] - target[1]).powi(2) + (a[2] - target[2]).powi(2);
    let (mut best_p, mut best_d) = ([0.0; 3], f64::INFINITY);
    for (p, lab) in coarse {
        let d = d2(*lab);
        if d < best_d {
            best_d = d;
            best_p = *p;
        }
    }
    if let Some(w) = warm {
        let d = d2(model.lab(separate(w)));
        if d < best_d {
            best_d = d;
            best_p = w;
        }
    }
    let mut step = 1.0 / 16.0;
    while step > 1e-4 {
        let mut moved = false;
        for axis in 0..3 {
            for dir in [-1.0, 1.0] {
                let mut p = best_p;
                p[axis] = (p[axis] + dir * step).clamp(0.0, 1.0);
                let d = d2(model.lab(separate(p)));
                if d < best_d {
                    best_d = d;
                    best_p = p;
                    moved = true;
                }
            }
        }
        if !moved {
            step *= 0.5;
        }
    }
    best_p
}

/// An identity ICC curve as a two-entry `curv` (0 -> 0, 65535 -> 65535).
///
/// **Deliberately not the zero-entry `curv`** ICC.1 also defines as the
/// identity: `moxcms` 0.8.1's `read_trc_tag` returns early for a zero count
/// without setting the size it read, so `read_nested_tone_curves` does not
/// advance and **every later curve in the set is re-read as that same
/// identity** (found 2026-10-12: this file's a*/b* shaper, written after a
/// zero-entry L* curve, parsed back as identity and grey round-tripped
/// 0.47 a* off). `shaper_survives_encoding` pins it.
fn identity_curve() -> ToneReprCurve {
    ToneReprCurve::Lut(vec![0, 65535])
}

/// `n` [`identity_curve`]s, one per channel.
fn identity_curves(n: usize) -> Vec<ToneReprCurve> {
    (0..n).map(|_| identity_curve()).collect()
}

/// The B2A input shaper for a* and b*: piecewise linear, mapping the encoded
/// value of a* = 0 (128/255) to exactly 0.5, so with an odd grid the neutral
/// axis lies **on** CLUT nodes. Without it (measured on this file's first
/// build, 2026-10-12) the nodes sit at a* = -3.8 and +4.1, grey and white were
/// interpolated across clipped out-of-gamut nodes, and printable colours round
/// tripped at up to 2.5 ΔE00 -- paper white itself at ~0.5. As 256 entries
/// so the kink falls exactly on entry 128. Returns `(curve, node -> a*)`'s
/// curve; [`shaped_ab`] is its inverse at the grid nodes.
fn ab_shaper_curve() -> ToneReprCurve {
    ToneReprCurve::Lut(
        (0..256)
            .map(|i| {
                let s = if i <= 128 { i as f64 / 256.0 } else { 0.5 + (i - 128) as f64 / 254.0 };
                q16(s)
            })
            .collect(),
    )
}

/// The a* (or b*) a B2A grid node at shaped coordinate `s` (0..1) stands for:
/// the inverse of [`ab_shaper_curve`] (`s = 0.5` is exactly a* = 0).
fn shaped_ab(s: f64) -> f64 {
    if s <= 0.5 { s * 256.0 - 128.0 } else { (s - 0.5) * 254.0 }
}

/// A normalised [0, 1] value as a 16-bit CLUT entry.
fn q16(v: f64) -> u16 {
    (v.clamp(0.0, 1.0) * 65535.0).round() as u16
}

/// Lab in the ICC v4 normalised encoding `moxcms` reads (`Lab::to_pcs_xyz`:
/// L/100, (a+128)/255, (b+128)/255).
fn lab_norm(lab: [f64; 3]) -> [f64; 3] {
    [lab[0] / 100.0, (lab[1] + 128.0) / 255.0, (lab[2] + 128.0) / 255.0]
}

/// **The shipped CMYK profile, generated.** Builds an ICC v4 output-device
/// (`prtr`) CMYK profile over a Lab PCS from [`InkModel`] and [`separate`],
/// and returns its bytes as `moxcms`' writer encodes them.
///
/// - A2B0 = A2B1 (`mAB`): identity A and B curves around an
///   [`A2B_GRID`]^4 16-bit CLUT of the model's Lab at each CMYK node.
/// - B2A0 = B2A1 (`mBA`): identity curves around a [`B2A_GRID`]^3 16-bit CLUT
///   holding, at each Lab node, [`invert`]'s separation (so GCR and the
///   [`TOTAL_INK_LIMIT`] hold in everything the profile emits).
/// - `wtpt` D50 (the media white is the PCS white: relative colorimetric),
///   `desc` [`PROFILE_DESCRIPTION`], `cprt` stating it is synthetic.
///
/// Cost: one run each on this machine (2026-10-12, an order of magnitude
/// only): ~0.3 s in a release build, ~3 s in a debug build -- paid once per
/// process, on the first [`check`], which is why [`engine`] caches it.
///
/// Deterministic: same code, same bytes, except the header date `moxcms`
/// stamps at every `encode()` (see `export_stream.rs`'s note on it).
/// `Err` only if `moxcms` refuses to encode.
pub fn synthetic_cmyk_icc() -> Result<Vec<u8>, String> {
    let model = InkModel::new();

    let g = A2B_GRID as usize;
    let mut a2b = Vec::with_capacity(g.pow(4) * 3);
    for c in 0..g {
        for m in 0..g {
            for y in 0..g {
                for k in 0..g {
                    let n = (g - 1) as f64;
                    let lab = model.lab([c as f64 / n, m as f64 / n, y as f64 / n, k as f64 / n]);
                    a2b.extend(lab_norm(lab).map(q16));
                }
            }
        }
    }

    // The coarse seed table for the inversion: COARSE^3 separations and their Lab.
    let cs = COARSE;
    let mut coarse = Vec::with_capacity(cs.pow(3));
    for i in 0..cs {
        for j in 0..cs {
            for l in 0..cs {
                let n = (cs - 1) as f64;
                let p = [i as f64 / n, j as f64 / n, l as f64 / n];
                coarse.push((p, model.lab(separate(p))));
            }
        }
    }
    let b = B2A_GRID as usize;
    let mut b2a = Vec::with_capacity(b.pow(3) * 4);
    for li in 0..b {
        for ai in 0..b {
            let mut warm = None;
            for bi in 0..b {
                let n = (b - 1) as f64;
                let lab = [li as f64 / n * 100.0, shaped_ab(ai as f64 / n), shaped_ab(bi as f64 / n)];
                let p = invert(&model, &coarse, lab, warm);
                warm = Some(p);
                b2a.extend(separate(p).map(q16));
            }
        }
    }

    let mut grid_a = [0u8; 16];
    grid_a[..4].fill(A2B_GRID);
    let a2b_lut = LutWarehouse::Multidimensional(LutMultidimensionalType {
        num_input_channels: 4,
        num_output_channels: 3,
        grid_points: grid_a,
        clut: Some(LutStore::Store16(a2b)),
        a_curves: identity_curves(4),
        b_curves: identity_curves(3),
        m_curves: Vec::new(),
        matrix: Matrix3d::IDENTITY,
        bias: Vector3d::default(),
    });
    let mut grid_b = [0u8; 16];
    grid_b[..3].fill(B2A_GRID);
    let b2a_lut = LutWarehouse::Multidimensional(LutMultidimensionalType {
        num_input_channels: 3,
        num_output_channels: 4,
        grid_points: grid_b,
        clut: Some(LutStore::Store16(b2a)),
        a_curves: identity_curves(4),
        b_curves: vec![identity_curve(), ab_shaper_curve(), ab_shaper_curve()],
        m_curves: Vec::new(),
        matrix: Matrix3d::IDENTITY,
        bias: Vector3d::default(),
    });

    let mut p = ColorProfile::default();
    p.color_space = DataColorSpace::Cmyk;
    p.pcs = DataColorSpace::Lab;
    p.profile_class = ProfileClass::OutputDevice;
    p.rendering_intent = RenderingIntent::RelativeColorimetric;
    p.white_point = Xyzd::new(D50[0], D50[1], D50[2]);
    p.media_white_point = Some(Xyzd::new(D50[0], D50[1], D50[2]));
    p.lut_a_to_b_perceptual = Some(a2b_lut.clone());
    p.lut_a_to_b_colorimetric = Some(a2b_lut);
    p.lut_b_to_a_perceptual = Some(b2a_lut.clone());
    p.lut_b_to_a_colorimetric = Some(b2a_lut);
    p.description = Some(ProfileText::PlainString(PROFILE_DESCRIPTION.into()));
    p.copyright = Some(ProfileText::PlainString(
        "Synthetic profile generated by Cartalith (print_gamut.rs); no measured press data. Not FOGRA/SWOP.".into(),
    ));
    // `moxcms`' writer stamps the header version (at least v4) itself.
    p.encode().map_err(|e| format!("moxcms could not encode the synthetic CMYK profile: {e:?}"))
}

// ---------------------------------------------------------------------------
// The check
// ---------------------------------------------------------------------------

/// The two transforms a check needs, built once from one CMYK profile.
pub struct Engine {
    to_cmyk: Arc<TransformF64Executor>,
    to_rgb: Arc<TransformF64Executor>,
    /// What the profile is, for the dictionary's `source` key.
    pub source: &'static str,
}

impl Engine {
    /// Parses `icc` with `moxcms` and builds sRGB -> CMYK and CMYK -> sRGB
    /// transforms (relative colorimetric, `f64`). `Err` when the bytes do not
    /// parse, are not a CMYK profile, or `moxcms` cannot transform with them.
    pub fn from_icc(icc: &[u8], source: &'static str) -> Result<Self, String> {
        let cmyk = ColorProfile::new_from_slice(icc).map_err(|e| format!("profile does not parse: {e:?}"))?;
        if cmyk.color_space != DataColorSpace::Cmyk {
            return Err(format!("not a CMYK profile ({:?})", cmyk.color_space));
        }
        let srgb = crate::pack::colour_import::srgb_destination();
        let opts = TransformOptions { rendering_intent: RenderingIntent::RelativeColorimetric, ..Default::default() };
        let to_cmyk = srgb
            .create_transform_f64(Layout::Rgb, &cmyk, Layout::Rgba, opts)
            .map_err(|e| format!("sRGB -> CMYK transform: {e:?}"))?;
        let to_rgb = cmyk
            .create_transform_f64(Layout::Rgba, &srgb, Layout::Rgb, opts)
            .map_err(|e| format!("CMYK -> sRGB transform: {e:?}"))?;
        Ok(Self { to_cmyk, to_rgb, source })
    }

    /// sRGB (encoded, 0..1) -> CMYK -> sRGB, clamped to [0, 1]. `None` if a
    /// transform reports an error.
    fn round_trip(&self, rgb: [f64; 3]) -> Option<[f64; 3]> {
        let mut cmyk = [0.0f64; 4];
        self.to_cmyk.transform(&rgb, &mut cmyk).ok()?;
        let mut back = [0.0f64; 3];
        self.to_rgb.transform(&cmyk, &mut back).ok()?;
        Some(back.map(|v| if v.is_finite() { v.clamp(0.0, 1.0) } else { 0.0 }))
    }

    /// The full check for one colour; see [`Gamut`].
    pub fn check(&self, rgb: [f64; 3]) -> Option<Gamut> {
        let rgb = rgb.map(|v| v.clamp(0.0, 1.0));
        let nearest = self.round_trip(rgb)?;
        let delta_e = ciede2000(srgb_to_lab(rgb), srgb_to_lab(nearest));
        Some(Gamut { delta_e, in_gamut: delta_e <= IN_GAMUT_MAX_DE00, nearest, source: self.source })
    }
}

/// One colour's print-gamut answer. Only ever built from a real round trip.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Gamut {
    /// CIEDE2000 between the colour and its CMYK round trip.
    pub delta_e: f64,
    /// `delta_e <= IN_GAMUT_MAX_DE00`.
    pub in_gamut: bool,
    /// The round-tripped colour (sRGB, encoded, 0..1): what the profile would
    /// print, i.e. the nearest printable colour under relative colorimetric.
    pub nearest: [f64; 3],
    /// Which profile answered (`"own_icc"` for the synthetic one).
    pub source: &'static str,
}

/// The process-wide engine on the synthetic profile, built on first use.
/// `None` -- and every check absent -- if generating, parsing or transforming
/// through the profile failed; the failure is not retried.
fn engine() -> Option<&'static Engine> {
    static ENGINE: OnceLock<Option<Engine>> = OnceLock::new();
    ENGINE
        .get_or_init(|| {
            let icc = synthetic_cmyk_icc().ok()?;
            Engine::from_icc(&icc, PROFILE_SOURCE_OWN_ICC).ok()
        })
        .as_ref()
}

/// The check the `#[func]` makes: `None` when no CMYK profile is available
/// (the caller then returns an empty dictionary -- never "in gamut").
pub fn check(rgb: [f64; 3]) -> Option<Gamut> {
    engine()?.check(rgb)
}

// ---------------------------------------------------------------------------
// Colour maths
// ---------------------------------------------------------------------------

fn mul3(m: &[[f64; 3]; 3], v: [f64; 3]) -> [f64; 3] {
    [
        m[0][0] * v[0] + m[0][1] * v[1] + m[0][2] * v[2],
        m[1][0] * v[0] + m[1][1] * v[1] + m[1][2] * v[2],
        m[2][0] * v[0] + m[2][1] * v[1] + m[2][2] * v[2],
    ]
}

/// IEC 61966-2-1 sRGB decode (encoded -> linear).
fn srgb_eotf(v: f64) -> f64 {
    if v <= 0.040_45 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) }
}

/// CIE Lab (D50) of a linear sRGB triple. CIE 15:2004's Lab, with the
/// exact 216/24389 and 24389/27 constants (the CIE's own rational forms).
fn linear_srgb_to_lab(rgb: [f64; 3]) -> [f64; 3] {
    let xyz = mul3(&SRGB_TO_XYZ_D50, rgb);
    let f = |t: f64| if t > 216.0 / 24389.0 { t.cbrt() } else { (24389.0 / 27.0 * t + 16.0) / 116.0 };
    let fx = f(xyz[0] / D50[0]);
    let fy = f(xyz[1] / D50[1]);
    let fz = f(xyz[2] / D50[2]);
    [116.0 * fy - 16.0, 500.0 * (fx - fy), 200.0 * (fy - fz)]
}

/// CIE Lab (D50) of an encoded sRGB triple.
pub fn srgb_to_lab(rgb: [f64; 3]) -> [f64; 3] {
    linear_srgb_to_lab(rgb.map(srgb_eotf))
}

/// CIE XYZ (D50) of a CIE Lab (D50) triple, inverting [`linear_srgb_to_lab`]'s Lab.
fn lab_to_xyz_d50(lab: [f64; 3]) -> [f64; 3] {
    let fy = (lab[0] + 16.0) / 116.0;
    let fx = fy + lab[1] / 500.0;
    let fz = fy - lab[2] / 200.0;
    let finv = |f: f64| if f.powi(3) > 216.0 / 24389.0 { f.powi(3) } else { (116.0 * f - 16.0) * 27.0 / 24389.0 };
    [finv(fx) * D50[0], finv(fy) * D50[1], finv(fz) * D50[2]]
}

/// CIEDE2000 colour difference (kL = kC = kH = 1), as specified in G. Sharma,
/// W. Wu, E. N. Dalal, "The CIEDE2000 color-difference formula: implementation
/// notes, supplementary test data, and mathematical observations", Color
/// Research & Application 30(1), 2005 -- including its notes on the hue-angle
/// mean and difference when a chroma is zero. Tested against that paper's
/// supplementary pairs (`ciede2000_sharma_pairs`).
pub fn ciede2000(lab1: [f64; 3], lab2: [f64; 3]) -> f64 {
    use std::f64::consts::PI;
    let [l1, a1, b1] = lab1;
    let [l2, a2, b2] = lab2;
    let c1 = a1.hypot(b1);
    let c2 = a2.hypot(b2);
    let cbar = (c1 + c2) / 2.0;
    let cbar7 = cbar.powi(7);
    let g = 0.5 * (1.0 - (cbar7 / (cbar7 + 25f64.powi(7))).sqrt());
    let a1p = (1.0 + g) * a1;
    let a2p = (1.0 + g) * a2;
    let c1p = a1p.hypot(b1);
    let c2p = a2p.hypot(b2);
    let hue = |b: f64, a: f64| {
        if b == 0.0 && a == 0.0 {
            0.0
        } else {
            let h = b.atan2(a).to_degrees();
            if h < 0.0 { h + 360.0 } else { h }
        }
    };
    let h1p = hue(b1, a1p);
    let h2p = hue(b2, a2p);
    let dlp = l2 - l1;
    let dcp = c2p - c1p;
    let dhp = if c1p * c2p == 0.0 {
        0.0
    } else {
        let d = h2p - h1p;
        if d > 180.0 {
            d - 360.0
        } else if d < -180.0 {
            d + 360.0
        } else {
            d
        }
    };
    let dhp_big = 2.0 * (c1p * c2p).sqrt() * (dhp.to_radians() / 2.0).sin();
    let lbarp = (l1 + l2) / 2.0;
    let cbarp = (c1p + c2p) / 2.0;
    let hbarp = if c1p * c2p == 0.0 {
        h1p + h2p
    } else if (h1p - h2p).abs() <= 180.0 {
        (h1p + h2p) / 2.0
    } else if h1p + h2p < 360.0 {
        (h1p + h2p + 360.0) / 2.0
    } else {
        (h1p + h2p - 360.0) / 2.0
    };
    let t = 1.0 - 0.17 * (hbarp - 30.0).to_radians().cos() + 0.24 * (2.0 * hbarp).to_radians().cos()
        + 0.32 * (3.0 * hbarp + 6.0).to_radians().cos()
        - 0.20 * (4.0 * hbarp - 63.0).to_radians().cos();
    let dtheta = 30.0 * (-((hbarp - 275.0) / 25.0).powi(2)).exp();
    let cbarp7 = cbarp.powi(7);
    let rc = 2.0 * (cbarp7 / (cbarp7 + 25f64.powi(7))).sqrt();
    let lb50 = (lbarp - 50.0).powi(2);
    let sl = 1.0 + 0.015 * lb50 / (20.0 + lb50).sqrt();
    let sc = 1.0 + 0.045 * cbarp;
    let sh = 1.0 + 0.015 * cbarp * t;
    let rt = -(2.0 * dtheta * PI / 180.0).sin() * rc;
    let tl = dlp / sl;
    let tc = dcp / sc;
    let th = dhp_big / sh;
    (tl * tl + tc * tc + th * th + rt * tc * th).sqrt()
}

#[cfg(test)]
#[path = "print_gamut_tests.rs"]
mod tests;
