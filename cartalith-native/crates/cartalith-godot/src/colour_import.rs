//! Colour-managed PNG import for **asset images** -- `EDITING_TOOLS_SCOPE.md`
//! milestone ET-E2.
//!
//! # What it does
//!
//! [`decode_png_managed`] decodes a PNG exactly as
//! `cartalith_assets::raster::decode_png` always has (same `image` call, same
//! RGBA8 bytes), then reads the file's colour information and, when the image
//! is tagged with a colour space that is **not** sRGB, converts the pixels to
//! the engine's sRGB working space with `moxcms`. It returns a
//! [`ColourReport`] saying what it found and what it did, so the shell can
//! tell the user rather than guess.
//!
//! # Precedence (PNG Specification, Third Edition, W3C Recommendation
//! 24 June 2025, the colour-chunk precedence rules -- read via the scope
//! document's §2 summary row, not re-read here)
//!
//! `cICP` > `iCCP` > `sRGB` > `gAMA`+`cHRM`, else untagged. A chunk that is
//! present but cannot be used (an unsupported cICP code, a profile that will
//! not inflate or parse, a CMYK or grey profile on an RGB path, `gAMA`
//! without `cHRM`) is **reported as a warning and the next tier is tried**,
//! which is what the spec asks of a decoder that does not support a chunk.
//! Only chunks before the first `IDAT` count: the spec requires every colour
//! chunk to precede the image data, and the `png` crate's `read_info` stops
//! there too.
//!
//! # What never changes (the standing byte-identity constraint)
//!
//! - An image with no colour chunk decodes to today's bytes, by control flow:
//!   no profile is built and no transform runs.
//! - An `sRGB`-chunk image, a cICP image whose code points ARE sRGB
//!   (primaries 1, transfer 13, full range), an ICC profile that is
//!   sRGB-equivalent ([`icc_is_srgb_equivalent`]) and a `gAMA`/`cHRM` pair
//!   carrying the PNG spec's own sRGB compatibility values
//!   ([`gama_chrm_is_srgb`]) all decode to today's bytes the same way.
//! - **Heightmaps are never routed here.** A heightmap is data, not colour
//!   (`EDITING_TOOLS_SCOPE.md` §3.3 "Imports"); `cartalith-engine`'s
//!   `import.rs` calls `cartalith_assets::raster::decode_png_data`, which
//!   reads no colour chunk at all.
//!
//! # The conversion
//!
//! Relative colorimetric (the scope's named intent, "a judgement, matching
//! PhotoCraft's gamut path" -- the scope's own label, kept). Out-of-gamut
//! colours **clip per channel** to [0, 1] in the destination: that is
//! relative colorimetric's defined behaviour (ICC.1 rendering-intent
//! definitions, read via summary, not re-read here) and what `moxcms`'s
//! 8-bit transform does (tested in `colour_import_tests.rs`, which pins Display
//! P3 red (255,0,0) to sRGB (255,0,0) with an unclipped value of 1.225).
//!
//! **Black-point compensation, which the scope also names, is not available**:
//! `moxcms` 0.8.1's `TransformOptions` has the field commented out
//! (`// pub black_point_compensation: bool`, `transform.rs`). For a
//! matrix/TRC profile whose curves pass through zero -- every profile the
//! tests build, and Display P3, Adobe RGB, ProPhoto and Rec. 2020 as
//! published -- the source black is XYZ (0,0,0), the destination black is
//! the same, and BPC would be the identity. A LUT-based profile can have a
//! non-zero black, so converting one adds a warning saying BPC was not
//! applied.
//!
//! The destination is an sRGB profile built here ([`srgb_destination`]) from
//! the IEC 61966-2-1 primaries and the D65 white (0.3127, 0.3290) that
//! `render.rs`'s `SRGB_TO_P3` and ET-E1's `display_p3_icc` use -- **not**
//! `moxcms::ColorProfile::new_srgb()` as it stands, whose white is the
//! 6504 K daylight value 1.2e-4 away (ET-E1's finding, `STATUS.md`).
//!
//! # Where it is called
//!
//! `asset_bridge.rs` (`AssetLibrarySession::import_item_managed` and
//! `load_sheet_managed`, behind `WorldGen::as_import_item` / `as_load_sheet`)
//! and `pack.rs::load_pack_from_bytes` (every decoded pack image). This file
//! is declared inside `pack.rs` (`#[path]`) rather than in `lib.rs` because
//! two integration-test targets compile `pack.rs` standalone; its tests live
//! in `colour_import_tests.rs`, compiled by `lib.rs` only, because they need
//! `export_stream::display_p3_icc`.

use cartalith_assets::{DecodedImage, ImageError, decode_png};
use moxcms::{
    Chromaticity, ColorPrimaries, ColorProfile, DataColorSpace, Layout, ProfileText, RenderingIntent, ToneReprCurve,
    TransformOptions, XyY,
};

/// The D65 white every profile this module builds uses: (0.3127, 0.3290), the
/// four-decimal value of IEC 61966-2-1 and SMPTE EG 432-1, and the white
/// `render.rs`'s `SRGB_TO_P3` and `export_stream.rs`'s `D65` already use.
const D65: XyY = XyY { x: 0.3127, y: 0.3290, yb: 1.0 };

/// The IEC 61966-2-1 sRGB tone curve as an ICC parametric curve (type 3:
/// `Y = (aX+b)^g` for `X >= d`, else `cX`): g 2.4, a 1/1.055, b 0.055/1.055,
/// c 1/12.92, d 0.04045. Same five numbers `moxcms`'s `new_srgb` writes.
fn srgb_trc() -> ToneReprCurve {
    ToneReprCurve::Parametric(vec![2.4, 1.0 / 1.055, 0.055 / 1.055, 1.0 / 12.92, 0.04045])
}

/// The intent every conversion here uses, as the report spells it.
pub const INTENT_NAME: &str = "relative colorimetric";

/// The largest per-entry difference between a profile's colorant matrix and
/// [`srgb_destination`]'s for which the profile still counts as sRGB
/// ([`icc_is_srgb_equivalent`]). **A labelled judgement**: 2e-3 in a D50 XYZ
/// colorant moves a full-scale linear value by at most ~2e-3, about a quarter
/// of an 8-bit code after the sRGB encode (slope ~0.44 at the top). Published
/// sRGB profiles (the HP/Microsoft v2 profile, the ICC's v4 ones) differ from
/// each other by ~1e-4 to 1e-3 in these entries, so all of them pass, and
/// Display P3 (red X 0.515 vs 0.436) fails by two orders of magnitude.
pub const SRGB_COLORANT_TOLERANCE: f64 = 2e-3;

/// What [`decode_png_managed`] found and did, for the import result's
/// `colour` dictionary. Every field is a fact about this file; nothing is
/// filled with a plausible default.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColourReport {
    /// The colour space the pixels were read as, in words: e.g. `"sRGB"`,
    /// `"Display P3 (cICP)"`, an ICC profile's own description, `"untagged
    /// (treated as sRGB)"` or `"unusable colour information (treated as
    /// sRGB)"`. Always present: even an untagged image has an answer, and the
    /// string says it was assumed.
    pub source: String,
    /// Whether the file carried any colour chunk at all (before `IDAT`).
    pub tagged: bool,
    /// The chunk that decided `source` (`"cICP"`, `"iCCP"`, `"sRGB"` or
    /// `"gAMA+cHRM"`). `None` when nothing usable decided it -- untagged, or
    /// every chunk present was unusable.
    pub tag: Option<&'static str>,
    /// Whether the pixels were changed by a conversion.
    pub converted: bool,
    /// The rendering intent of the conversion, `Some` only when `converted`.
    pub intent: Option<&'static str>,
    /// Everything the caller should be told: a chunk that could not be used
    /// and why, a CMYK or grey profile left unconverted, a conversion that
    /// failed, BPC not applied. Empty when there is nothing to say.
    pub warnings: Vec<String>,
}

impl ColourReport {
    /// The report for a file whose pixels were read as sRGB without
    /// conversion, decided by `tag` (or by nothing, when `tag` is `None`).
    fn as_srgb(source: String, tagged: bool, tag: Option<&'static str>, warnings: Vec<String>) -> Self {
        ColourReport { source, tagged, tag, converted: false, intent: None, warnings }
    }
}

// ---------------------------------------------------------------------------
// The chunk walk
// ---------------------------------------------------------------------------

/// The eight-byte PNG signature (PNG-3 §5.2).
const PNG_SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1a, b'\n'];

/// One chunk as it sits in the file: its four-byte type, its data, and
/// whether its stored CRC matches (PNG-3 §5.3, CRC over type + data).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chunk<'a> {
    /// The chunk type, e.g. `*b"iCCP"`.
    pub kind: [u8; 4],
    /// The chunk's data field, `length` bytes.
    pub data: &'a [u8],
    /// `true` when the stored CRC equals the computed one.
    pub crc_ok: bool,
}

/// The PNG CRC-32 (ISO 3309 / ITU-T V.42 polynomial 0xEDB88320, reflected,
/// initial and final XOR 0xFFFFFFFF: PNG-3 Annex D's sample code), bitwise
/// rather than table-driven because a colour chunk is a few bytes and an
/// iCCP a few kilobytes.
pub fn png_crc32(parts: &[&[u8]]) -> u32 {
    let mut c: u32 = 0xFFFF_FFFF;
    for part in parts {
        for &b in *part {
            c ^= b as u32;
            for _ in 0..8 {
                c = if c & 1 != 0 { 0xEDB8_8320 ^ (c >> 1) } else { c >> 1 };
            }
        }
    }
    c ^ 0xFFFF_FFFF
}

/// Walk `bytes` as a PNG chunk stream up to and including `IEND`. `Err` for a
/// missing signature or a chunk whose declared length runs past the end of
/// the buffer -- a truncated file, which the pixel decode will reject too.
/// It never panics: every slice is bounds-checked.
pub fn png_chunks(bytes: &[u8]) -> Result<Vec<Chunk<'_>>, String> {
    if bytes.len() < 8 || bytes[..8] != PNG_SIGNATURE {
        return Err("not a PNG (signature missing)".into());
    }
    let mut out = Vec::new();
    let mut i = 8usize;
    while i < bytes.len() {
        let header = bytes.get(i..i + 8).ok_or("truncated chunk header")?;
        let len = u32::from_be_bytes([header[0], header[1], header[2], header[3]]) as usize;
        let kind = [header[4], header[5], header[6], header[7]];
        let data_end = (i + 8).checked_add(len).ok_or("chunk length overflows")?;
        let data = bytes.get(i + 8..data_end).ok_or("chunk data runs past the end of the file")?;
        let crc_bytes = bytes.get(data_end..data_end + 4).ok_or("chunk CRC runs past the end of the file")?;
        let stored = u32::from_be_bytes([crc_bytes[0], crc_bytes[1], crc_bytes[2], crc_bytes[3]]);
        out.push(Chunk { kind, data, crc_ok: stored == png_crc32(&[&kind, data]) });
        i = data_end + 4;
        if &kind == b"IEND" {
            break;
        }
    }
    Ok(out)
}

/// The colour chunks of one file, first occurrence of each before the first
/// `IDAT`. `None` = absent. A present chunk with a bad CRC or the wrong
/// length is still recorded (so it can be reported), as `Err(reason)`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ColourChunks {
    /// `cICP`: colour primaries, transfer function, matrix coefficients,
    /// video-full-range flag (PNG-3 §11.3.2.6; H.273 code points).
    pub cicp: Option<Result<[u8; 4], String>>,
    /// Whether an `iCCP` chunk is present. Its profile comes from the `png`
    /// crate (which inflates it), not from here.
    pub iccp: Option<Result<(), String>>,
    /// `sRGB`: the rendering-intent byte (0-3).
    pub srgb: Option<Result<u8, String>>,
    /// `gAMA`: the image gamma times 100 000 (e.g. 45455 for 1/2.2).
    pub gama: Option<Result<u32, String>>,
    /// `cHRM`: white x, y, red x, y, green x, y, blue x, y, each times 100 000.
    pub chrm: Option<Result<[u32; 8], String>>,
    /// Colour chunks seen more than once; the first is used (the `png`
    /// crate rejects the duplicate), and each is reported.
    pub duplicates: Vec<&'static str>,
    /// The IHDR colour type (PNG-3 §11.2.1): 0 grey, 2 RGB, 3 palette,
    /// 4 grey+alpha, 6 RGBA. `None` if IHDR is missing or short.
    pub colour_type: Option<u8>,
}

impl ColourChunks {
    /// Whether any colour chunk is present at all.
    pub fn any(&self) -> bool {
        self.cicp.is_some() || self.iccp.is_some() || self.srgb.is_some() || self.gama.is_some() || self.chrm.is_some()
    }
}

/// Read the four big-endian `u32`s or fixed-length fields of the colour
/// chunks out of `chunks`, stopping at the first `IDAT` (PNG-3 requires every
/// colour chunk before it). Lengths are the spec's: cICP 4, sRGB 1, gAMA 4,
/// cHRM 32.
pub fn colour_chunks(chunks: &[Chunk<'_>]) -> ColourChunks {
    let mut c = ColourChunks::default();
    for ch in chunks {
        let k = &ch.kind;
        if k == b"IDAT" {
            break;
        }
        if k == b"IHDR" {
            c.colour_type = ch.data.get(9).copied();
            continue;
        }
        let (name, slot_taken): (&'static str, bool) = match k {
            b"cICP" => ("cICP", c.cicp.is_some()),
            b"iCCP" => ("iCCP", c.iccp.is_some()),
            b"sRGB" => ("sRGB", c.srgb.is_some()),
            b"gAMA" => ("gAMA", c.gama.is_some()),
            b"cHRM" => ("cHRM", c.chrm.is_some()),
            _ => continue,
        };
        if slot_taken {
            c.duplicates.push(name);
            continue;
        }
        let bad = |why: &str| format!("{name} chunk {why}");
        match name {
            "cICP" => {
                c.cicp = Some(if !ch.crc_ok {
                    Err(bad("has a bad CRC"))
                } else if ch.data.len() != 4 {
                    Err(bad("is not 4 bytes"))
                } else {
                    Ok([ch.data[0], ch.data[1], ch.data[2], ch.data[3]])
                })
            }
            "iCCP" => c.iccp = Some(if ch.crc_ok { Ok(()) } else { Err(bad("has a bad CRC")) }),
            "sRGB" => {
                c.srgb = Some(if !ch.crc_ok {
                    Err(bad("has a bad CRC"))
                } else if ch.data.len() != 1 || ch.data[0] > 3 {
                    Err(bad("is not one byte of 0-3"))
                } else {
                    Ok(ch.data[0])
                })
            }
            "gAMA" => {
                c.gama = Some(if !ch.crc_ok {
                    Err(bad("has a bad CRC"))
                } else if ch.data.len() != 4 {
                    Err(bad("is not 4 bytes"))
                } else {
                    let g = u32::from_be_bytes([ch.data[0], ch.data[1], ch.data[2], ch.data[3]]);
                    if g == 0 { Err(bad("is zero")) } else { Ok(g) }
                })
            }
            _ => {
                c.chrm = Some(if !ch.crc_ok {
                    Err(bad("has a bad CRC"))
                } else if ch.data.len() != 32 {
                    Err(bad("is not 32 bytes"))
                } else {
                    let mut v = [0u32; 8];
                    for (n, slot) in v.iter_mut().enumerate() {
                        let b = &ch.data[n * 4..n * 4 + 4];
                        *slot = u32::from_be_bytes([b[0], b[1], b[2], b[3]]);
                    }
                    Ok(v)
                })
            }
        }
    }
    c
}

/// The decompressed `iCCP` profile, via the `png` crate's own `read_info`
/// (which inflates it). `None` when the chunk is absent **or** could not be
/// inflated: `png` 0.18.1's `parse_iccp` discards an inflate error
/// (`let _ = self.parse_iccp_raw()`), so the caller tells the two apart with
/// [`ColourChunks::iccp`] (present) -- present-but-`None` is "unreadable".
fn inflated_icc(bytes: &[u8]) -> Result<Option<Vec<u8>>, String> {
    let decoder = png::Decoder::new(std::io::Cursor::new(bytes));
    let reader = decoder.read_info().map_err(|e| format!("the PNG header could not be read for colour information ({e})"))?;
    Ok(reader.info().icc_profile.as_ref().map(|p| p.to_vec()))
}

// ---------------------------------------------------------------------------
// Profiles
// ---------------------------------------------------------------------------

/// The sRGB destination: `moxcms`'s sRGB tone curve with colorants rebuilt
/// from the IEC 61966-2-1 primaries (BT.709 xy) and [`D65`], so the
/// destination white is exactly the white every source profile built here
/// uses (see the module doc for why not `new_srgb()`'s own).
pub fn srgb_destination() -> ColorProfile {
    let mut p = ColorProfile::new_srgb();
    p.update_rgb_colorimetry(D65, ColorPrimaries::BT_709);
    p.red_trc = Some(srgb_trc());
    p.green_trc = Some(srgb_trc());
    p.blue_trc = Some(srgb_trc());
    p
}

/// A matrix/TRC profile with `primaries`, [`D65`] and `trc` on all three
/// channels.
fn rgb_profile(primaries: ColorPrimaries, white: XyY, trc: ToneReprCurve) -> ColorProfile {
    let mut p = ColorProfile::new_srgb();
    p.update_rgb_colorimetry(white, primaries);
    p.red_trc = Some(trc.clone());
    p.green_trc = Some(trc.clone());
    p.blue_trc = Some(trc);
    p
}

/// The IEC 61966-2-1 sRGB encode (linear to non-linear), for the TRC check
/// in [`icc_is_srgb_equivalent`] only.
fn srgb_oetf(l: f64) -> f64 {
    if l <= 0.003_130_8 { 12.92 * l } else { 1.055 * l.powf(1.0 / 2.4) - 0.055 }
}

/// Whether an ICC profile is sRGB in everything an 8-bit import can see, so
/// converting through it would only add rounding noise: an RGB matrix/TRC
/// profile with no LUT tags, colorants within [`SRGB_COLORANT_TOLERANCE`] of
/// [`srgb_destination`]'s, and each channel's TRC mapping every 8-bit code
/// `i` to a linear value that the sRGB encode rounds back to `i`. A labelled
/// judgement: it exists so a file carrying an embedded sRGB profile (common
/// from Photoshop) imports byte-identical to today, as the standing
/// constraint asks, rather than shifting by a code here and there.
pub fn icc_is_srgb_equivalent(p: &ColorProfile) -> bool {
    if p.color_space != DataColorSpace::Rgb || !p.is_matrix_shaper() {
        return false;
    }
    if p.lut_a_to_b_perceptual.is_some() || p.lut_a_to_b_colorimetric.is_some() || p.lut_a_to_b_saturation.is_some() {
        return false;
    }
    let a = p.colorant_matrix();
    let b = srgb_destination().colorant_matrix();
    for r in 0..3 {
        for c in 0..3 {
            if (a.v[r][c] - b.v[r][c]).abs() > SRGB_COLORANT_TOLERANCE {
                return false;
            }
        }
    }
    for trc in [&p.red_trc, &p.green_trc, &p.blue_trc] {
        let Ok(table) = p.build_8bit_lin_table(trc) else { return false };
        for (i, &lin) in table.iter().enumerate() {
            let back = (srgb_oetf(lin.clamp(0.0, 1.0) as f64) * 255.0 + 0.5).floor() as usize;
            if back != i {
                return false;
            }
        }
    }
    true
}

/// An ICC profile's own description, or `"unnamed ICC profile"`. The text is
/// the file's claim, shown as the source name; it is never used to decide
/// anything.
fn profile_name(p: &ColorProfile) -> String {
    let text = match &p.description {
        Some(ProfileText::PlainString(s)) => s.clone(),
        Some(ProfileText::Localizable(v)) => v.first().map(|l| l.value.clone()).unwrap_or_default(),
        Some(ProfileText::Description(d)) => d.ascii_string.clone(),
        None => String::new(),
    };
    let t = text.trim().trim_end_matches('\0').trim();
    if t.is_empty() { "unnamed ICC profile".into() } else { t.chars().take(80).collect() }
}

/// The cICP code points this module converts, as a source profile and a name.
/// Supported: colour primaries 1 (BT.709 = sRGB), 9 (BT.2020) and 12 (SMPTE
/// EG 432-1, Display P3 D65), each with transfer 13 (the sRGB curve,
/// IEC 61966-2-1) and the full-range flag set (H.273 Tables 2 and 3; PNG-3
/// says RGB cICP uses matrix 0 and full range). Everything else is `Err` with
/// the reason and the caller falls through: PQ (16) and HLG (18) are HDR, and
/// tone-mapping them is ET-E4's deferred HDR work (Ruling CB item 5);
/// narrow range and the BT.709-family transfers (1, 6, 14, 15) are left out
/// because how a still image should invert a camera OETF is ambiguous, a
/// labelled judgement not to guess.
///
/// `Ok(None)` means "this IS sRGB": no conversion.
fn cicp_source(c: [u8; 4]) -> Result<Option<(ColorProfile, String)>, String> {
    let [prim, transfer, matrix, full] = c;
    if matrix != 0 {
        return Err(format!("cICP matrix coefficients {matrix} (PNG requires 0): cICP ignored"));
    }
    if full != 1 {
        return Err("cICP narrow (video) range is not supported: cICP ignored".into());
    }
    if transfer == 16 || transfer == 18 {
        return Err(format!("cICP transfer {transfer} is HDR (PQ/HLG), which import does not tone-map: cICP ignored"));
    }
    if transfer != 13 {
        return Err(format!("cICP transfer {transfer} is not supported (only 13, the sRGB curve): cICP ignored"));
    }
    match prim {
        1 => Ok(None),
        9 => Ok(Some((rgb_profile(ColorPrimaries::BT_2020, D65, srgb_trc()), "Rec. 2020 (cICP)".into()))),
        12 => Ok(Some((rgb_profile(ColorPrimaries::DISPLAY_P3, D65, srgb_trc()), "Display P3 (cICP)".into()))),
        p => Err(format!("cICP colour primaries {p} are not supported (1, 9 and 12 are): cICP ignored")),
    }
}

/// The PNG specification's sRGB compatibility values: the `gAMA` (45455) and
/// `cHRM` (white 31270/32900, red 64000/33000, green 30000/60000, blue
/// 15000/6000) an sRGB writer is asked to emit beside the `sRGB` chunk
/// (PNG-3, the `sRGB` chunk's section; read via summary). A pair carrying
/// them is read as sRGB with no conversion, even though gamma 1/0.45455 is
/// not exactly the piecewise sRGB curve, because that is what the writer
/// meant. Tolerance: gAMA within 10 units (1e-4) and each cHRM value within
/// 100 units (1e-3) -- a labelled judgement covering writers that rounded.
pub fn gama_chrm_is_srgb(gama: u32, chrm: [u32; 8]) -> bool {
    const SRGB_CHRM: [u32; 8] = [31270, 32900, 64000, 33000, 30000, 60000, 15000, 6000];
    gama.abs_diff(45455) <= 10 && chrm.iter().zip(SRGB_CHRM).all(|(&a, b)| a.abs_diff(b) <= 100)
}

/// A source profile from `gAMA` + `cHRM`: chromaticities from `cHRM`, and a
/// pure power curve with exponent `100000 / gama` (the `gAMA` value is the
/// encoding gamma, so decoding raises to its reciprocal: PNG-3's gAMA
/// section). `Err` when the chromaticities do not make a valid RGB space.
fn gama_chrm_profile(gama: u32, chrm: [u32; 8]) -> Result<ColorProfile, String> {
    let f = |v: u32| v as f32 / 100_000.0;
    let ch = |n: usize| Chromaticity::new(f(chrm[n]), f(chrm[n + 1]));
    if chrm.chunks(2).any(|xy| xy[1] == 0) {
        return Err("cHRM has a zero y chromaticity: gAMA/cHRM ignored".into());
    }
    let white = ch(0).to_xyyb();
    let primaries = ColorPrimaries { red: ch(2), green: ch(4), blue: ch(6) };
    let p = rgb_profile(primaries, white, ToneReprCurve::Parametric(vec![100_000.0 / gama as f32]));
    let m = p.colorant_matrix();
    if m.v.iter().flatten().any(|v| !v.is_finite()) {
        return Err("cHRM chromaticities do not form an RGB space: gAMA/cHRM ignored".into());
    }
    Ok(p)
}

// ---------------------------------------------------------------------------
// Resolution and conversion
// ---------------------------------------------------------------------------

/// What [`resolve`] decided: the report, and the source profile to convert
/// from when (and only when) the pixels must change.
pub struct Resolution {
    /// The report as of resolution (`converted`/`intent` are set by the
    /// conversion step).
    pub report: ColourReport,
    /// `Some(profile)` when the pixels need converting from it to sRGB.
    pub convert_from: Option<ColorProfile>,
}

/// Apply PNG-3's precedence to `chunks` (and the inflated profile `icc`).
/// Pure: no pixels. See the module doc for the tier rules.
pub fn resolve(chunks: &ColourChunks, icc: Option<&[u8]>) -> Resolution {
    let mut warnings: Vec<String> = chunks.duplicates.iter().map(|n| format!("duplicate {n} chunk: the first was used")).collect();
    let tagged = chunks.any();
    let grey = matches!(chunks.colour_type, Some(0) | Some(4));
    let done = |report: ColourReport, from: Option<ColorProfile>| Resolution { report, convert_from: from };

    // Tier 1: cICP.
    if let Some(c) = &chunks.cicp {
        match c.as_ref().map_err(Clone::clone).and_then(|&c| cicp_source(c)) {
            Ok(None) => return done(ColourReport::as_srgb("sRGB (cICP)".into(), tagged, Some("cICP"), warnings), None),
            Ok(Some((p, name))) => return done(ColourReport::as_srgb(name, tagged, Some("cICP"), warnings), Some(p)),
            Err(why) => warnings.push(why),
        }
    }

    // Tier 2: iCCP.
    if let Some(present) = &chunks.iccp {
        match (present, icc) {
            (Err(why), _) => warnings.push(format!("{why}: iCCP ignored")),
            (Ok(()), None) => warnings.push("iCCP profile could not be decompressed: iCCP ignored".into()),
            (Ok(()), Some(bytes)) => match ColorProfile::new_from_slice(bytes) {
                Err(e) => warnings.push(format!("iCCP profile could not be read ({e:?}): iCCP ignored")),
                Ok(p) => {
                    let name = profile_name(&p);
                    match p.color_space {
                        DataColorSpace::Cmyk => warnings.push(format!("CMYK profile \"{name}\": not converted")),
                        DataColorSpace::Gray => warnings.push(format!("grey profile \"{name}\": not converted")),
                        DataColorSpace::Rgb if grey => {
                            warnings.push(format!("RGB profile \"{name}\" on a greyscale PNG (invalid per PNG-3): not converted"))
                        }
                        DataColorSpace::Rgb if icc_is_srgb_equivalent(&p) => {
                            return done(ColourReport::as_srgb(format!("{name} (sRGB-equivalent)"), tagged, Some("iCCP"), warnings), None);
                        }
                        DataColorSpace::Rgb => {
                            if !p.is_matrix_shaper() {
                                warnings.push("LUT-based profile: black-point compensation is not available (moxcms 0.8.1), converted without it".into());
                            }
                            return done(ColourReport::as_srgb(name, tagged, Some("iCCP"), warnings), Some(p));
                        }
                        other => warnings.push(format!("unrecognised profile colour space {other:?} in \"{name}\": not converted")),
                    }
                }
            },
        }
    }

    // Tier 3: sRGB.
    if let Some(s) = &chunks.srgb {
        match s {
            Ok(_) => return done(ColourReport::as_srgb("sRGB".into(), tagged, Some("sRGB"), warnings), None),
            Err(why) => warnings.push(format!("{why}: sRGB ignored")),
        }
    }

    // Tier 4: gAMA + cHRM.
    match (&chunks.gama, &chunks.chrm) {
        (Some(Ok(g)), Some(Ok(c))) => {
            if gama_chrm_is_srgb(*g, *c) {
                return done(ColourReport::as_srgb("sRGB (gAMA/cHRM compatibility values)".into(), tagged, Some("gAMA+cHRM"), warnings), None);
            }
            match gama_chrm_profile(*g, *c) {
                Ok(p) => {
                    let name = format!("gAMA {:.5} / cHRM white {:.4},{:.4}", *g as f64 / 100_000.0, c[0] as f64 / 100_000.0, c[1] as f64 / 100_000.0);
                    return done(ColourReport::as_srgb(name, tagged, Some("gAMA+cHRM"), warnings), Some(p));
                }
                Err(why) => warnings.push(why),
            }
        }
        (Some(g), Some(c)) => {
            for e in [g.as_ref().err(), c.as_ref().err()].into_iter().flatten() {
                warnings.push(format!("{e}: gAMA/cHRM ignored"));
            }
        }
        (Some(_), None) => warnings.push("gAMA without cHRM: primaries unknown, not converted".into()),
        (None, Some(_)) => warnings.push("cHRM without gAMA: transfer unknown, not converted".into()),
        (None, None) => {}
    }

    let source = if tagged { "unusable colour information (treated as sRGB)" } else { "untagged (treated as sRGB)" };
    done(ColourReport::as_srgb(source.into(), tagged, None, warnings), None)
}

/// Convert `img`'s RGBA8 pixels in place from `src` to [`srgb_destination`],
/// relative colorimetric, alpha untouched. Options: `prefer_fixed_point:
/// false` (a one-off import favours `moxcms`'s float path over its Q2.13 one;
/// a judgement) and `allow_use_cicp_transfer: false` (a profile's TRC tags
/// are its normative colorimetry in ICC.1's matrix/TRC model, read via
/// summary; an embedded `cicp` tag must not override them -- ET-E1 found
/// `moxcms`'s own P3 profile carrying a wrong one).
pub fn convert_to_srgb(img: &mut DecodedImage, src: &ColorProfile) -> Result<(), String> {
    let options = TransformOptions {
        rendering_intent: RenderingIntent::RelativeColorimetric,
        allow_use_cicp_transfer: false,
        prefer_fixed_point: false,
        ..TransformOptions::default()
    };
    let transform = src
        .create_transform_8bit(Layout::Rgba, &srgb_destination(), Layout::Rgba, options)
        .map_err(|e| format!("could not build the conversion ({e:?})"))?;
    let mut out = vec![0u8; img.rgba.len()];
    transform.transform(&img.rgba, &mut out).map_err(|e| format!("the conversion failed ({e:?})"))?;
    // Alpha is not colour: carry it over byte for byte whatever the
    // transform did with it. moxcms 0.8.1's Rgba->Rgba transform already
    // preserves alpha (a mutant removing this copy survived
    // `alpha_survives_a_conversion_untouched`, 2026-10-11), so this line is
    // a guard against a future moxcms changing that, not current behaviour.
    for (o, i) in out.chunks_exact_mut(4).zip(img.rgba.chunks_exact(4)) {
        o[3] = i[3];
    }
    img.rgba = out;
    Ok(())
}

/// Decode `bytes` as a PNG to RGBA8 sRGB, colour-managed. The pixel decode is
/// `decode_png`'s, unchanged; `Err` only where `decode_png` errs (an
/// unreadable PNG). Nothing about colour can fail the import: every problem
/// becomes a warning in the report and the pixels are kept as decoded.
///
/// A 16-bit PNG is handled: `decode_png` reduces it to 8 bits exactly as
/// before, and the conversion runs on those 8-bit values (up to one code
/// coarser than a 16-bit transform would be; a judgement, since every
/// consumer downstream is RGBA8).
pub fn decode_png_managed(bytes: &[u8]) -> Result<(DecodedImage, ColourReport), ImageError> {
    let mut img = decode_png(bytes)?;
    let (chunks, mut early) = match png_chunks(bytes) {
        Ok(list) => (colour_chunks(&list), Vec::new()),
        Err(why) => (ColourChunks::default(), vec![format!("colour information could not be read ({why})")]),
    };
    let icc = if matches!(chunks.iccp, Some(Ok(()))) {
        match inflated_icc(bytes) {
            Ok(p) => p,
            Err(why) => {
                early.push(why);
                None
            }
        }
    } else {
        None
    };
    let Resolution { mut report, convert_from } = resolve(&chunks, icc.as_deref());
    early.append(&mut report.warnings);
    report.warnings = early;
    if let Some(src) = convert_from {
        match convert_to_srgb(&mut img, &src) {
            Ok(()) => {
                report.converted = true;
                report.intent = Some(INTENT_NAME);
            }
            Err(why) => report.warnings.push(format!("{why}: pixels kept as decoded and read as sRGB")),
        }
    }
    Ok((img, report))
}
