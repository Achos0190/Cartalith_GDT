//! Tests for `pack::colour_import` (ET-E2), compiled by `lib.rs` only: they
//! need `export_stream::display_p3_icc` and `asset_bridge`, which the two
//! integration targets that `#[path]`-include `pack.rs` do not have.
//!
//! **Where the expected numbers come from.** Every converted pixel pinned
//! below was computed by an independent Python reference (not the code under
//! test, not `moxcms`): RGB->XYZ matrices derived from each space's published
//! xy primaries and the D65 white (0.3127, 0.3290), the source EOTF, a
//! straight XYZ->linear-sRGB matrix (both whites are D65, so no chromatic
//! adaptation), a per-channel clip to [0, 1], the IEC 61966-2-1 sRGB encode,
//! and round-half-up to 8 bits. Primaries: Display P3 .680/.320 .265/.690
//! .150/.060 (SMPTE EG 432-1); Rec. 2020 .708/.292 .170/.797 .131/.046
//! (ITU-R BT.2020); Adobe RGB (1998) .64/.33 .21/.71 .15/.06 with gamma
//! 563/256; sRGB .64/.33 .30/.60 .15/.06. **Tolerance: +-1 code per
//! channel**, because `moxcms` runs its tone curves through 8-bit-in /
//! 4096-entry tables and its SIMD paths differ by platform; a value the
//! reference CLIPS (0 or 255 from an out-of-range linear value) is pinned
//! exactly, since a clip has no rounding to disagree about.

use crate::pack::colour_import::*;
use cartalith_assets::raster::{PngColour, encode_png_rgb8};
use cartalith_assets::{DecodedImage, decode_png, encode_png};
use std::io::Write;

// ---------------------------------------------------------------------------
// Fixture builders (independent of the code under test where it matters)
// ---------------------------------------------------------------------------

/// FNV-1a 64 over a byte slice: a compact, stable pixel fingerprint.
fn fnv(b: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &x in b {
        h ^= x as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

/// The ten test colours, in this order, as a 5x2 RGB8 image.
const COLOURS: [[u8; 3]; 10] = [
    [255, 0, 0],
    [0, 255, 0],
    [0, 0, 255],
    [255, 255, 255],
    [0, 0, 0],
    [128, 128, 128],
    [200, 100, 50],
    [10, 20, 30],
    [60, 180, 90],
    [240, 200, 40],
];

/// [`COLOURS`] as packed RGB8.
fn colour_rgb() -> Vec<u8> {
    COLOURS.iter().flatten().copied().collect()
}

/// An untagged (no colour chunk) 5x2 PNG of [`COLOURS`].
fn untagged_png() -> Vec<u8> {
    encode_png_rgb8(5, 2, colour_rgb(), PngColour::Data).expect("encode")
}

/// zlib-compress `data` (flate2, a dev-dependency) for an `iCCP` payload.
fn zlib(data: &[u8]) -> Vec<u8> {
    let mut e = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    e.write_all(data).unwrap();
    e.finish().unwrap()
}

/// An `iCCP` chunk body: keyword, NUL, compression method 0, zlib stream
/// (PNG-3 §11.3.2.4).
fn iccp_body(profile: &[u8]) -> Vec<u8> {
    let mut v = b"test profile\0\0".to_vec();
    v.extend(zlib(profile));
    v
}

/// Insert `chunks` straight after IHDR, each with a correct CRC unless its
/// `crc` override is given. IHDR is always the first chunk and 13 bytes long
/// (PNG-3 §11.2.1), so it ends at byte 8 + 8 + 13 + 4 = 33.
fn splice(png: &[u8], chunks: &[(&[u8; 4], Vec<u8>, Option<u32>)]) -> Vec<u8> {
    assert_eq!(&png[12..16], b"IHDR", "fixture must start with IHDR");
    let mut out = png[..33].to_vec();
    for (kind, data, crc) in chunks {
        out.extend((data.len() as u32).to_be_bytes());
        out.extend(*kind);
        out.extend(data);
        out.extend(crc.unwrap_or_else(|| png_crc32(&[*kind, data])).to_be_bytes());
    }
    out.extend(&png[33..]);
    out
}

/// The ET-E1 Display P3 profile, as a P3 export embeds it.
fn p3_profile() -> &'static [u8] {
    crate::export_stream::display_p3_icc().expect("the P3 profile builds")
}

/// A P3-tagged (iCCP) PNG of [`COLOURS`], written by ET-E1's own encoder.
fn p3_png() -> Vec<u8> {
    encode_png_rgb8(5, 2, colour_rgb(), PngColour::IccProfile(p3_profile())).expect("encode")
}

/// A cICP chunk body: primaries, transfer, matrix 0, full range.
fn cicp(primaries: u8, transfer: u8) -> Vec<u8> {
    vec![primaries, transfer, 0, 1]
}

/// A `cHRM` body from eight values already times 100 000.
fn chrm(v: [u32; 8]) -> Vec<u8> {
    v.iter().flat_map(|x| x.to_be_bytes()).collect()
}

/// The sRGB `cHRM` values of the PNG specification.
const SRGB_CHRM: [u32; 8] = [31270, 32900, 64000, 33000, 30000, 60000, 15000, 6000];

/// RGB triples of a decoded RGBA8 image.
fn rgb_of(img: &DecodedImage) -> Vec<[u8; 3]> {
    let v: Vec<[u8; 3]> = img.rgba.chunks_exact(4).map(|p| [p[0], p[1], p[2]]).collect();
    assert!(!v.is_empty(), "decoded image is empty");
    v
}

/// Assert `got` equals the reference within +-1 per channel, and EXACTLY
/// where the reference value is a clip (0 or 255 reached from outside the
/// gamut, flagged in `clipped`).
fn assert_near(got: &[[u8; 3]], want: &[[u8; 3]], clipped: &[[bool; 3]], what: &str) {
    assert_eq!(got.len(), want.len(), "{what}: pixel count");
    for (i, ((g, w), c)) in got.iter().zip(want).zip(clipped).enumerate() {
        for ch in 0..3 {
            let d = (g[ch] as i32 - w[ch] as i32).abs();
            if c[ch] {
                assert_eq!(g[ch], w[ch], "{what}: pixel {i} channel {ch} must clip exactly: got {g:?}, reference {w:?}");
            } else {
                assert!(d <= 1, "{what}: pixel {i} channel {ch}: got {g:?}, reference {w:?}");
            }
        }
    }
}

/// The independent reference for Display P3 -> sRGB over [`COLOURS`] (see
/// the file header). The unclipped linear values were, in order:
/// (1.22494,-0.04206,-0.01964) (-0.22494,1.04206,-0.07864)
/// (-0.0,0.0,1.09827) (1,1,1) (0,0,0) (0.21586 x3) (0.67884,0.10851,0.01367)
/// (0.00214,0.00716,0.01365) (-0.04731,0.47371,0.07551)
/// (0.93745,0.56522,-0.03923).
const P3_TO_SRGB: [[u8; 3]; 10] = [
    [255, 0, 0],
    [0, 255, 0],
    [0, 0, 255],
    [255, 255, 255],
    [0, 0, 0],
    [128, 128, 128],
    [215, 93, 31],
    [7, 20, 31],
    [0, 183, 78],
    [248, 198, 0],
];
/// Which channels of [`P3_TO_SRGB`] the reference clipped (linear value
/// outside [0, 1] before the clip).
const P3_CLIPPED: [[bool; 3]; 10] = [
    [true, true, true],
    [true, true, true],
    [true, false, true],
    [false; 3],
    [false; 3],
    [false; 3],
    [false; 3],
    [false; 3],
    [true, false, false],
    [false, false, true],
];

/// Rec. 2020 -> sRGB over [`COLOURS`], same reference. Unclipped linear:
/// (1.66049,-0.12455,-0.01815) (-0.58764,1.1329,-0.10058)
/// (-0.07285,-0.00835,1.11873) ... (200,100,50)->(0.88186,0.07217,0.01238)
/// (10,20,30)->(-2e-05,0.00744,0.01377) (60,180,90)->(-0.20062,0.51059,0.06766)
/// (240,200,40)->(1.10594,0.54563,-0.05017).
const R2020_TO_SRGB: [[u8; 3]; 10] = [
    [255, 0, 0],
    [0, 255, 0],
    [0, 0, 255],
    [255, 255, 255],
    [0, 0, 0],
    [128, 128, 128],
    [241, 76, 29],
    [0, 21, 31],
    [0, 189, 74],
    [255, 195, 0],
];
/// Which channels of [`R2020_TO_SRGB`] the reference clipped. (10,20,30)'s
/// red is -2e-5 linear: clipped, but by less than a code, so it is left in
/// the +-1 band rather than pinned.
const R2020_CLIPPED: [[bool; 3]; 10] = [
    [true, true, true],
    [true, true, true],
    [true, true, true],
    [false; 3],
    [false; 3],
    [false; 3],
    [false; 3],
    [false; 3],
    [true, false, false],
    [true, false, true],
];

/// Adobe RGB (1998) -> sRGB, same reference.
const ADOBE_TO_SRGB: [[u8; 3]; 10] = [
    [255, 0, 0],
    [0, 255, 0],
    [0, 0, 255],
    [255, 255, 255],
    [0, 0, 0],
    [129, 129, 129],
    [227, 100, 42],
    [0, 12, 24],
    [0, 181, 83],
    [254, 201, 0],
];
/// Which channels of [`ADOBE_TO_SRGB`] the reference clipped.
const ADOBE_CLIPPED: [[bool; 3]; 10] = [
    [true, false, false],
    [true, false, true],
    [false, false, true],
    [false; 3],
    [false; 3],
    [false; 3],
    [false; 3],
    [true, false, false],
    [true, false, false],
    [false, false, true],
];

/// gamma 1/0.55556 with sRGB primaries -> sRGB, same reference (no clips).
const G18_TO_SRGB: [[u8; 3]; 10] = [
    [255, 0, 0],
    [0, 255, 0],
    [0, 0, 255],
    [255, 255, 255],
    [0, 0, 0],
    [146, 146, 146],
    [210, 119, 65],
    [10, 26, 40],
    [77, 193, 109],
    [243, 210, 53],
];

// ---------------------------------------------------------------------------
// The chunk walk
// ---------------------------------------------------------------------------

/// Protects: the CRC the chunk walk uses to decide a colour chunk is
/// readable. Literals: IEND's CRC 0xAE426082 (every PNG ends with it) and
/// the sRGB-intent-0 chunk's 0xAECE1CE9 (ET-E1's pinned constant, read
/// from the png crate's own writer).
#[test]
fn png_crc32_matches_published_chunk_crcs() {
    assert_eq!(png_crc32(&[b"IEND", b""]), 0xAE42_6082);
    assert_eq!(png_crc32(&[b"sRGB", &[0]]), 0xAECE_1CE9);
}

/// Protects: the walker's framing (length, type, data, CRC), its stop at
/// IEND, and that a corrupted CRC is reported rather than trusted.
#[test]
fn png_chunks_walks_a_real_file_and_flags_bad_crcs() {
    let png = untagged_png();
    let list = png_chunks(&png).expect("a valid PNG walks");
    let kinds: Vec<&[u8; 4]> = list.iter().map(|c| &c.kind).collect();
    assert_eq!(kinds.first(), Some(&b"IHDR"));
    assert_eq!(kinds.last(), Some(&b"IEND"));
    assert!(kinds.contains(&b"IDAT"));
    assert!(list.iter().all(|c| c.crc_ok), "every chunk of a clean file has a good CRC");
    assert_eq!(list[0].data.len(), 13, "IHDR is 13 bytes");

    let bad = splice(&png, &[(b"sRGB", vec![0], Some(0xDEAD_BEEF))]);
    let list = png_chunks(&bad).unwrap();
    let srgb = list.iter().find(|c| &c.kind == b"sRGB").expect("the spliced chunk is found");
    assert!(!srgb.crc_ok, "a wrong CRC must be flagged");
}

/// Protects: the walker never panics or reads past the buffer.
#[test]
fn png_chunks_refuses_a_missing_signature_and_a_truncated_file() {
    assert!(png_chunks(b"not a png").is_err());
    let png = untagged_png();
    assert!(png_chunks(&png[..40]).is_err(), "a chunk running past the end is an error");
}

/// Protects: the chunk field layouts (cICP 4, sRGB 1, gAMA 4, cHRM 32
/// bytes), the IHDR colour type, the stop at the first IDAT (PNG-3 puts
/// every colour chunk before it), and duplicate reporting.
#[test]
fn colour_chunks_reads_fields_stops_at_idat_and_reports_duplicates() {
    let png = splice(
        &untagged_png(),
        &[
            (b"cICP", cicp(12, 13), None),
            (b"sRGB", vec![1], None),
            (b"sRGB", vec![2], None),
            (b"gAMA", 45455u32.to_be_bytes().to_vec(), None),
            (b"cHRM", chrm(SRGB_CHRM), None),
        ],
    );
    let c = colour_chunks(&png_chunks(&png).unwrap());
    assert_eq!(c.cicp, Some(Ok([12, 13, 0, 1])));
    assert_eq!(c.srgb, Some(Ok(1)), "the FIRST sRGB is used");
    assert_eq!(c.duplicates, vec!["sRGB"]);
    assert_eq!(c.gama, Some(Ok(45455)));
    assert_eq!(c.chrm, Some(Ok(SRGB_CHRM)));
    assert_eq!(c.colour_type, Some(2), "RGB");
    assert!(c.iccp.is_none());

    // A colour chunk after IDAT does not count.
    let png = untagged_png();
    let iend = png.len() - 12;
    let mut late = png[..iend].to_vec();
    late.extend(1u32.to_be_bytes());
    late.extend(b"sRGB");
    late.push(0);
    late.extend(png_crc32(&[b"sRGB", &[0]]).to_be_bytes());
    late.extend(&png[iend..]);
    let c = colour_chunks(&png_chunks(&late).unwrap());
    assert!(c.srgb.is_none() && !c.any(), "an sRGB chunk after IDAT is ignored");

    // Wrong lengths and values are recorded as errors, not defaults.
    let png = splice(
        &untagged_png(),
        &[(b"sRGB", vec![7], None), (b"gAMA", vec![0, 0, 0, 0], None), (b"cICP", vec![1, 13, 0], None)],
    );
    let c = colour_chunks(&png_chunks(&png).unwrap());
    assert!(matches!(c.srgb, Some(Err(_))));
    assert!(matches!(c.gama, Some(Err(_))));
    assert!(matches!(c.cicp, Some(Err(_))));
}

// ---------------------------------------------------------------------------
// Byte identity where nothing should move
// ---------------------------------------------------------------------------

/// FNV-1a of `decode_png(untagged_png()).rgba` as HEAD's `decode_png`
/// produces it (`decode_png`'s body is unchanged by ET-E2; computed by
/// running it on this fixture, 2026-10-11).
const UNTAGGED_RGBA_FNV: u64 = 0x232d_f214_a7db_8da9;

/// Protects: the standing constraint (no colour chunk -> today's bytes)
/// and the report's honesty: "untagged (treated as sRGB)", not "sRGB".
#[test]
fn an_untagged_image_imports_byte_identical_and_says_it_was_assumed() {
    let png = untagged_png();
    let (img, r) = decode_png_managed(&png).unwrap();
    assert_eq!(img.rgba, decode_png(&png).unwrap().rgba);
    assert_eq!(fnv(&img.rgba), UNTAGGED_RGBA_FNV);
    assert_eq!(
        r,
        ColourReport {
            source: "untagged (treated as sRGB)".into(),
            tagged: false,
            tag: None,
            converted: false,
            intent: None,
            warnings: vec![]
        }
    );
}

/// Protects: an sRGB-chunk image (every ET-E1 default export) re-imports
/// with today's bytes and is reported as sRGB, not converted.
#[test]
fn an_srgb_tagged_image_imports_byte_identical() {
    let png = encode_png_rgb8(5, 2, colour_rgb(), PngColour::Srgb).unwrap();
    let (img, r) = decode_png_managed(&png).unwrap();
    assert_eq!(fnv(&img.rgba), UNTAGGED_RGBA_FNV);
    assert_eq!((r.source.as_str(), r.tag, r.converted, r.intent), ("sRGB", Some("sRGB"), false, None));
    assert!(r.warnings.is_empty());
}

/// Protects: an embedded sRGB ICC profile and cICP 1/13 are read as sRGB
/// with today's bytes (byte identity by control flow), and Display P3 is
/// NOT taken for sRGB by the equivalence test.
#[test]
fn an_srgb_equivalent_icc_profile_and_srgb_cicp_do_not_convert() {
    let srgb_icc = moxcms::ColorProfile::new_srgb().encode().unwrap();
    let png = encode_png_rgb8(5, 2, colour_rgb(), PngColour::IccProfile(&srgb_icc)).unwrap();
    let (img, r) = decode_png_managed(&png).unwrap();
    assert_eq!(fnv(&img.rgba), UNTAGGED_RGBA_FNV);
    assert!(!r.converted && r.tag == Some("iCCP") && r.source.ends_with("(sRGB-equivalent)"), "{r:?}");

    let png = splice(&untagged_png(), &[(b"cICP", cicp(1, 13), None)]);
    let (img, r) = decode_png_managed(&png).unwrap();
    assert_eq!(fnv(&img.rgba), UNTAGGED_RGBA_FNV);
    assert_eq!((r.source.as_str(), r.tag, r.converted), ("sRGB (cICP)", Some("cICP"), false));

    let p3 = moxcms::ColorProfile::new_from_slice(p3_profile()).unwrap();
    assert!(!icc_is_srgb_equivalent(&p3), "Display P3 must not pass as sRGB");
    assert!(icc_is_srgb_equivalent(&srgb_destination()));
}

// ---------------------------------------------------------------------------
// Conversions, against the independent reference
// ---------------------------------------------------------------------------

/// Protects: the conversion itself (skip it and pure P3 green stays
/// (0,255,0) but (200,100,50) stays (200,100,50), not (215,93,31)), the
/// relative-colorimetric clip (P3 red (255,0,0) is linear sRGB
/// (1.225,-0.042,-0.020): it must land on (255,0,0) exactly), and the
/// report.
#[test]
fn a_display_p3_iccp_image_converts_to_the_reference_srgb_and_clips() {
    let (img, r) = decode_png_managed(&p3_png()).unwrap();
    assert_near(&rgb_of(&img), &P3_TO_SRGB, &P3_CLIPPED, "P3 iCCP");
    assert_eq!(rgb_of(&img)[0], [255, 0, 0], "P3 red clips to sRGB red");
    assert_eq!(rgb_of(&img)[8][0], 0, "P3 (60,180,90)'s red goes negative in sRGB and clips to 0");
    assert!(r.converted);
    assert_eq!(r.intent, Some("relative colorimetric"));
    assert_eq!(r.tag, Some("iCCP"));
    assert!(r.warnings.is_empty(), "{:?}", r.warnings);
}

/// Protects: cICP primaries 12 (Display P3) with the sRGB transfer reaches
/// the same pixels as the iCCP profile, both against the reference.
#[test]
fn a_cicp_p3_image_converts_like_the_iccp_one() {
    let png = splice(&untagged_png(), &[(b"cICP", cicp(12, 13), None)]);
    let (img, r) = decode_png_managed(&png).unwrap();
    assert_near(&rgb_of(&img), &P3_TO_SRGB, &P3_CLIPPED, "P3 cICP");
    let (icc_img, _) = decode_png_managed(&p3_png()).unwrap();
    for (a, b) in rgb_of(&img).iter().zip(rgb_of(&icc_img)) {
        for ch in 0..3 {
            assert!((a[ch] as i32 - b[ch] as i32).abs() <= 1, "cICP {a:?} vs iCCP {b:?}");
        }
    }
    assert_eq!((r.source.as_str(), r.tag, r.converted), ("Display P3 (cICP)", Some("cICP"), true));
}

/// Protects: cICP primaries 9 (Rec. 2020).
#[test]
fn a_cicp_rec2020_image_converts_to_the_reference() {
    let png = splice(&untagged_png(), &[(b"cICP", cicp(9, 13), None)]);
    let (img, r) = decode_png_managed(&png).unwrap();
    assert_near(&rgb_of(&img), &R2020_TO_SRGB, &R2020_CLIPPED, "Rec. 2020 cICP");
    assert!(r.converted);
}

/// Protects: a non-P3 matrix/TRC ICC profile with its own gamma.
#[test]
fn an_adobe_rgb_icc_image_converts_to_the_reference() {
    let adobe = moxcms::ColorProfile::new_adobe_rgb().encode().unwrap();
    let png = encode_png_rgb8(5, 2, colour_rgb(), PngColour::IccProfile(&adobe)).unwrap();
    let (img, r) = decode_png_managed(&png).unwrap();
    assert_near(&rgb_of(&img), &ADOBE_TO_SRGB, &ADOBE_CLIPPED, "Adobe RGB");
    assert!(r.converted);
}

/// Protects: tier 4. gamma 1/0.55556 with sRGB primaries converts to the
/// reference; the PNG spec's sRGB pair (45455 + sRGB cHRM) does not.
#[test]
fn gama_and_chrm_convert_unless_they_are_the_srgb_compatibility_pair() {
    let png = splice(
        &untagged_png(),
        &[(b"gAMA", 55556u32.to_be_bytes().to_vec(), None), (b"cHRM", chrm(SRGB_CHRM), None)],
    );
    let (img, r) = decode_png_managed(&png).unwrap();
    assert_near(&rgb_of(&img), &G18_TO_SRGB, &[[false; 3]; 10], "gAMA 0.55556");
    assert_eq!((r.tag, r.converted), (Some("gAMA+cHRM"), true));

    let png = splice(
        &untagged_png(),
        &[(b"gAMA", 45455u32.to_be_bytes().to_vec(), None), (b"cHRM", chrm(SRGB_CHRM), None)],
    );
    let (img, r) = decode_png_managed(&png).unwrap();
    assert_eq!(fnv(&img.rgba), UNTAGGED_RGBA_FNV);
    assert_eq!((r.tag, r.converted), (Some("gAMA+cHRM"), false));
    assert!(gama_chrm_is_srgb(45455, SRGB_CHRM) && !gama_chrm_is_srgb(55556, SRGB_CHRM));
}

/// Protects: no primaries -> no guess. The pixels stay today's, the
/// report says why.
#[test]
fn gama_alone_is_warned_and_not_converted() {
    let png = splice(&untagged_png(), &[(b"gAMA", 55556u32.to_be_bytes().to_vec(), None)]);
    let (img, r) = decode_png_managed(&png).unwrap();
    assert_eq!(fnv(&img.rgba), UNTAGGED_RGBA_FNV);
    assert!(!r.converted && r.tagged && r.tag.is_none());
    assert_eq!(r.source, "unusable colour information (treated as sRGB)");
    assert_eq!(r.warnings, vec!["gAMA without cHRM: primaries unknown, not converted".to_string()]);
}

// ---------------------------------------------------------------------------
// Precedence (PNG-3: cICP > iCCP > sRGB > gAMA/cHRM)
// ---------------------------------------------------------------------------

/// Protects: tier 1 beats tiers 2 and 3.
#[test]
fn cicp_wins_over_srgb_and_iccp() {
    let png = splice(&untagged_png(), &[(b"cICP", cicp(12, 13), None), (b"sRGB", vec![0], None)]);
    let (img, r) = decode_png_managed(&png).unwrap();
    assert_eq!((r.tag, r.converted), (Some("cICP"), true));
    assert_near(&rgb_of(&img), &P3_TO_SRGB, &P3_CLIPPED, "cICP over sRGB");

    // cICP sRGB over a P3 iCCP: the cICP says sRGB, so nothing converts.
    let png = splice(&p3_png(), &[(b"cICP", cicp(1, 13), None)]);
    let (img, r) = decode_png_managed(&png).unwrap();
    assert_eq!((r.tag, r.converted), (Some("cICP"), false));
    assert_eq!(fnv(&img.rgba), UNTAGGED_RGBA_FNV);
}

/// Protects: tier 2 beats tier 3, tier 3 beats tier 4.
#[test]
fn iccp_wins_over_srgb_and_srgb_wins_over_gama_chrm() {
    let png = splice(&p3_png(), &[(b"sRGB", vec![0], None)]);
    let (img, r) = decode_png_managed(&png).unwrap();
    assert_eq!((r.tag, r.converted), (Some("iCCP"), true));
    assert_near(&rgb_of(&img), &P3_TO_SRGB, &P3_CLIPPED, "iCCP over sRGB");

    let png = splice(
        &untagged_png(),
        &[(b"sRGB", vec![0], None), (b"gAMA", 55556u32.to_be_bytes().to_vec(), None), (b"cHRM", chrm(SRGB_CHRM), None)],
    );
    let (img, r) = decode_png_managed(&png).unwrap();
    assert_eq!((r.tag, r.converted), (Some("sRGB"), false));
    assert_eq!(fnv(&img.rgba), UNTAGGED_RGBA_FNV);
}

/// Protects: an HDR (PQ) cICP is not tone-mapped and not silently
/// ignored: it warns and the next tier (here a P3 iCCP) decides.
#[test]
fn an_unsupported_cicp_falls_through_with_a_warning() {
    let png = splice(&p3_png(), &[(b"cICP", cicp(9, 16), None)]);
    let (img, r) = decode_png_managed(&png).unwrap();
    assert_eq!((r.tag, r.converted), (Some("iCCP"), true));
    assert_eq!(r.warnings.len(), 1);
    assert!(r.warnings[0].contains("HDR"), "{:?}", r.warnings);
    assert_near(&rgb_of(&img), &P3_TO_SRGB, &P3_CLIPPED, "fall-through to iCCP");
}

// ---------------------------------------------------------------------------
// Profiles that cannot be used
// ---------------------------------------------------------------------------

/// Protects: a profile that will not inflate, one that inflates to
/// garbage, and one with a bad CRC each warn, keep today's pixels and
/// never fail the import.
#[test]
fn a_corrupt_iccp_is_a_warning_and_untagged_treatment() {
    for (what, chunk, needle) in [
        ("not zlib", {
            let mut v = b"bad\0\0".to_vec();
            v.extend(b"this is not a zlib stream");
            (v, None)
        }, "could not be decompressed"),
        ("garbage profile", (iccp_body(b"definitely not an ICC profile, just bytes"), None), "could not be read"),
        ("bad CRC", (iccp_body(p3_profile()), Some(0x1234_5678)), "bad CRC"),
    ] {
        let png = splice(&untagged_png(), &[(b"iCCP", chunk.0, chunk.1)]);
        let (img, r) = decode_png_managed(&png).unwrap_or_else(|e| panic!("{what}: import failed: {e}"));
        assert_eq!(fnv(&img.rgba), UNTAGGED_RGBA_FNV, "{what}: pixels moved");
        assert!(!r.converted && r.tagged && r.tag.is_none(), "{what}: {r:?}");
        assert_eq!(r.source, "unusable colour information (treated as sRGB)", "{what}");
        assert_eq!(r.warnings.len(), 1, "{what}: {:?}", r.warnings);
        assert!(r.warnings[0].contains(needle), "{what}: {:?}", r.warnings);
    }
}

/// Protects: a CMYK profile on an RGB PNG (invalid per PNG-3, seen in the
/// wild) is named and left alone, never pushed through an RGB transform.
/// Built by relabelling an encoded RGB profile's header colour-space
/// field (ICC.1 header bytes 16..20) as 'CMYK'.
#[test]
fn a_cmyk_profile_is_warned_and_not_converted() {
    let mut icc = moxcms::ColorProfile::new_srgb().encode().unwrap();
    icc[16..20].copy_from_slice(b"CMYK");
    let parsed = moxcms::ColorProfile::new_from_slice(&icc).expect("moxcms reads the relabelled header");
    assert_eq!(parsed.color_space, moxcms::DataColorSpace::Cmyk, "fixture premise");
    let png = encode_png_rgb8(5, 2, colour_rgb(), PngColour::IccProfile(&icc)).unwrap();
    let (img, r) = decode_png_managed(&png).unwrap();
    assert_eq!(fnv(&img.rgba), UNTAGGED_RGBA_FNV);
    assert!(!r.converted);
    assert_eq!(r.warnings.len(), 1);
    assert!(r.warnings[0].starts_with("CMYK profile"), "{:?}", r.warnings);
}

// ---------------------------------------------------------------------------
// Alpha, 16-bit
// ---------------------------------------------------------------------------

/// Protects: alpha is not colour.
#[test]
fn alpha_survives_a_conversion_untouched() {
    let rgba: Vec<u8> = COLOURS.iter().enumerate().flat_map(|(i, c)| [c[0], c[1], c[2], (i * 25) as u8]).collect();
    let base = encode_png(&DecodedImage::new(5, 2, rgba.clone()).unwrap()).unwrap();
    let png = splice(&base, &[(b"iCCP", iccp_body(p3_profile()), None)]);
    let (img, r) = decode_png_managed(&png).unwrap();
    assert!(r.converted);
    let alpha: Vec<u8> = img.rgba.chunks_exact(4).map(|p| p[3]).collect();
    assert_eq!(alpha, rgba.chunks_exact(4).map(|p| p[3]).collect::<Vec<_>>());
    assert_near(&rgb_of(&img), &P3_TO_SRGB, &P3_CLIPPED, "RGBA P3");
}

/// Protects: 16-bit is handled (reduced to 8 bits by decode_png exactly
/// as before, then converted), not skipped. Values are v*257 so the
/// reduction is exact.
#[test]
fn a_sixteen_bit_p3_image_converts_like_its_eight_bit_twin() {
    let mut buf = Vec::new();
    {
        let mut enc = png::Encoder::new(&mut buf, 5, 2);
        enc.set_color(png::ColorType::Rgb);
        enc.set_depth(png::BitDepth::Sixteen);
        let mut w = enc.write_header().unwrap();
        let data: Vec<u8> = colour_rgb().iter().flat_map(|&v| (v as u16 * 257).to_be_bytes()).collect();
        w.write_image_data(&data).unwrap();
    }
    let png = splice(&buf, &[(b"iCCP", iccp_body(p3_profile()), None)]);
    let (img, r) = decode_png_managed(&png).unwrap();
    assert!(r.converted);
    let (eight, _) = decode_png_managed(&p3_png()).unwrap();
    assert_eq!(img.rgba, eight.rgba);
}

// ---------------------------------------------------------------------------
// Heightmaps are data
// ---------------------------------------------------------------------------

/// A coloured 64x32 heightmap ramp (red and blue differ, so a colour
/// conversion WOULD move it), untagged.
fn coloured_heightmap() -> Vec<u8> {
    let (w, h) = (64u32, 32u32);
    let mut rgb = Vec::new();
    for y in 0..h {
        for x in 0..w {
            rgb.extend([(x * 4) as u8, (60 + y * 5) as u8, (255 - x * 3) as u8]);
        }
    }
    encode_png_rgb8(w, h, rgb, PngColour::Data).unwrap()
}

/// Protects: ET-E2's "heightmaps are never converted". The positive
/// control shows the tag WOULD move these pixels through the managed
/// decode, so the equality below is not vacuous.
#[test]
fn a_p3_tagged_heightmap_imports_to_the_same_field_as_its_untagged_twin() {
    let plain = coloured_heightmap();
    let tagged = splice(&plain, &[(b"iCCP", iccp_body(p3_profile()), None)]);
    let cicp_tagged = splice(&plain, &[(b"cICP", cicp(12, 13), None)]);
    let (managed, _) = decode_png_managed(&tagged).unwrap();
    assert_ne!(managed.rgba, decode_png(&plain).unwrap().rgba, "positive control: the tag is live");

    let a = cartalith_engine::import::decode_heightmap(&plain, 48).unwrap();
    assert!(!a.0.is_empty());
    assert_eq!(a, cartalith_engine::import::decode_heightmap(&tagged, 48).unwrap());
    assert_eq!(a, cartalith_engine::import::decode_heightmap(&cicp_tagged, 48).unwrap());

    let base = cartalith_engine::WorldParams::defaults(48, 999, 12345);
    let x = cartalith_engine::import::import_heightmap(&plain, &base).unwrap();
    let y = cartalith_engine::import::import_heightmap(&tagged, &base).unwrap();
    assert_eq!(x.state.field, y.state.field);
}

// ---------------------------------------------------------------------------
// The import paths
// ---------------------------------------------------------------------------

/// Protects: the two asset entry points behind `as_import_item` and
/// `as_load_sheet` run the managed decode, store converted pixels, and
/// hand the report back.
#[test]
fn the_asset_library_import_and_the_slicer_convert_and_report() {
    let mut s = crate::asset_bridge::AssetLibrarySession::new();
    let uid = s.add_custom_slot("p3", None);
    let r = s.import_item_managed(&uid, "p3".into(), &p3_png()).unwrap();
    assert!(r.converted);
    assert_near(&rgb_of(s.image(&uid, 0).unwrap()), &P3_TO_SRGB, &P3_CLIPPED, "library item");

    let (w, h, r) = s.load_sheet_managed("sheet.png".into(), &p3_png()).unwrap();
    assert_eq!((w, h), (5, 2));
    assert!(r.converted);
    let r = s.import_item_managed(&uid, "plain".into(), &untagged_png()).unwrap();
    assert!(!r.converted && !r.tagged);
    assert_eq!(fnv(&s.image(&uid, 1).unwrap().rgba), UNTAGGED_RGBA_FNV);
}

/// Protects: asset-pack images (the scope's "asset-pack images") go
/// through the managed decode, and the pack keeps one report per tagged
/// image, keyed by its archive path.
#[test]
fn a_loaded_pack_converts_tagged_images_and_lists_them() {
    let manifest = br#"{"schema":2,"name":"p3","structures":{"trait":{"mining":["structures/trait/mining_01.png","structures/trait/mining_02.png"]}}}"#;
    let p3 = p3_png();
    let plain = untagged_png();
    let zip = cartalith_assets::zip_store_bytes(&[
        ("pack.json", manifest.as_slice()),
        ("structures/trait/mining_01.png", p3.as_slice()),
        ("structures/trait/mining_02.png", plain.as_slice()),
    ])
    .unwrap();
    let loaded = crate::pack::load_pack_from_bytes(zip).unwrap();
    let v = &loaded.traits["mining"];
    assert_eq!(v.len(), 2);
    assert_near(&rgb_of(&v[0]), &P3_TO_SRGB, &P3_CLIPPED, "pack trait art");
    assert_eq!(fnv(&v[1].rgba), UNTAGGED_RGBA_FNV);
    assert_eq!(loaded.colour.len(), 1, "only the tagged image is listed");
    assert_eq!(loaded.colour[0].0, "structures/trait/mining_01.png");
    assert!(loaded.colour[0].1.converted);
}
