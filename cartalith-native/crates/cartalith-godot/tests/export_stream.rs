//! The streaming export writer (`EXPORT_SCOPE.md` §7, milestone E2) —
//! `export_stream::export_banded` / `write_bands`.
//!
//! # The bar, and how each half of it is met
//!
//! §7: *"round-trip tests that decode with a different decoder than the
//! encoder and compare every byte at several band geometries including a
//! short final band."*
//!
//! - **JPEG (E5)** is lossy, so it is held to a measured error bound against
//!   the same monolithic render instead of byte equality, decoded by
//!   `jpeg-decoder` (image-rs), not by the encoder's code; its *file* is still
//!   byte-identical at every band plan.
//! - **A different decoder.** Both lossless formats are read back by decoders written
//!   in this file, sharing no code with the encoders: PNG chunks, CRCs and all
//!   five row filters; BigTIFF header, IFD, strips and the horizontal
//!   predictor. The one shared piece is DEFLATE itself, inflated with
//!   `flate2`. The PNG encoder deflates with `fdeflate` (`Compression::Fast`,
//!   the shipped encoder's setting), so that half is independent too; the
//!   TIFF encoder deflates through `flate2`, so for TIFF the inflate is the
//!   same library family and only the container and predictor are
//!   independent. Pillow (libpng / libtiff / zlib) was run over the same
//!   files outside `cargo test` to close that gap; it cannot be a dependency
//!   of this suite.
//! - **Every byte.** Each decode is compared in full against the monolithic
//!   render (`export_raster_png`'s own four calls, transcribed as in
//!   `tests/export_bands.rs`), not sampled.
//! - **Several band geometries, short final bands.** Every plan below except
//!   the controls leaves a final band shorter than the rest, and one control
//!   divides the height exactly.
//!
//! The decoders are checked in their turn: each must read a file written by a
//! *third* encoder (the shipped `image` PNG path, the `tiff` crate's own
//! whole-image writer), and must reject a corrupted one.

#[path = "../src/render.rs"]
mod render;

/// The streaming writer under test, compiled straight from `src/export_stream.rs` (via `#[path]`) because `cartalith-godot` is `cdylib`-only.
#[path = "../src/export_stream.rs"]
mod export_stream;

use std::io::Read;
use std::path::PathBuf;

use export_stream::{StreamFormat, export_banded, write_bands};
use render::{BakeFields, ColorSpace, ExportBandPlan, ExportRivers, RenderCtx, TerrainAppearance};

/// `tests/export_bands.rs`'s fixture: a 61 × 43 closed-form world.
const GW: usize = 61;
// Fixture grid height (cells); with `GW`, a 61x43 world shared with `tests/export_bands.rs`. Odd and non-square on purpose so band heights never divide it evenly by accident.
const GH: usize = 43;
/// Sea level for the fixture world. Judgement: the project's usual 0.42, so the sea and land paths both run.
const SEA: f64 = 0.42;

/// The closed-form synthetic world (height, temperature, rainfall, discharge) at `gw`x`gh`; same formulae as `tests/export_bands.rs` and `bake_raster.rs`, independent of the generator, shaped so both land and sea branches run.
fn fixture(gw: usize, gh: usize) -> (Vec<f32>, Vec<f32>, Vec<f32>, Vec<f32>) {
    let n = gw * gh;
    let (mut field, mut temp, mut rain, mut flow) = (vec![0f32; n], vec![0f32; n], vec![0f32; n], vec![0f32; n]);
    for y in 0..gh {
        for x in 0..gw {
            let (u, v) = (x as f64 / gw as f64, y as f64 / gh as f64);
            let i = y * gw + x;
            field[i] = (0.46 + 0.34 * ((u * 6.1).sin() * (v * 4.3).cos()) + 0.12 * ((u * 17.0 + v * 11.0).sin())) as f32;
            temp[i] = (28.0 - 44.0 * v + 6.0 * (u * 9.0).cos()) as f32;
            rain[i] = (0.5 + 0.5 * ((u * 5.0 + v * 3.0).sin())).clamp(0.0, 1.0) as f32;
            flow[i] = (1.0 + 40.0 * ((u * 13.0).sin() * (v * 7.0).sin()).abs()) as f32;
        }
    }
    (field, temp, rain, flow)
}

/// Every stage that can differ between a band and the whole raster on:
/// local contrast and the plate frame (both on at `default()`), a real grade
/// with two field-influence weights, and hachure.
fn appearance() -> TerrainAppearance {
    let mut a = TerrainAppearance::default().with_look(render::LOOK_ANTIQUE);
    a.grade_field_elevation = 0.5;
    a.grade_field_moisture = 0.3;
    a.npr.hachure = 0.6;
    a
}

/// `export_raster_png`'s closure, call for call (since RV-5,
/// `render::bake_and_finish`). `a` is the context's look, stated at the call.
fn monolithic(ctx: &RenderCtx, a: &TerrainAppearance, ink: ExportRivers<'_>, w: usize, h: usize) -> Vec<u8> {
    assert_eq!(a.local_contrast, ctx.appearance().local_contrast, "the look passed must be the context's");
    let bf = BakeFields::new(ctx);
    let inf = render::build_grade_influence(ctx, w, h);
    render::bake_and_finish(ctx, &bf, ink, w, h, &inf, render::ColorSpace::Srgb)
}

/// A river `Flag` mask over the `GW`x`GH` grid with real structure (every third column plus the diagonal), so tinted and untinted cells are adjacent in both axes and the river tint is in the path.
fn river_mask() -> Vec<u8> {
    (0..GW * GH).map(|i| u8::from((i % GW).is_multiple_of(3) || i % GW == i / GW)).collect()
}

/// A per-process, per-name file path in the OS temp directory. The pid keeps concurrent test processes from sharing a file; tests remove their file after reading it.
fn scratch(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("cartalith_export_stream_{}_{name}", std::process::id()))
}

// ---------------------------------------------------------------------------
// The independent PNG decoder
// ---------------------------------------------------------------------------

/// Big-endian u32 at `at`, or a `truncated` error instead of a panic.
fn be32(b: &[u8], at: usize) -> Result<u32, String> {
    b.get(at..at + 4).map(|s| u32::from_be_bytes([s[0], s[1], s[2], s[3]])).ok_or_else(|| format!("truncated at {at}"))
}

/// CRC-32 (ISO 3309, as PNG specifies it), bit by bit.
fn crc32(data: &[u8]) -> u32 {
    // CRC-32 initial value (all ones) and, below, the reflected polynomial 0xEDB88320, per ISO 3309 / the PNG specification.
    let mut c = 0xFFFF_FFFFu32;
    for &byte in data {
        c ^= byte as u32;
        for _ in 0..8 {
            c = if c & 1 != 0 { 0xEDB8_8320 ^ (c >> 1) } else { c >> 1 };
        }
    }
    !c
}

/// Inflate a zlib stream with `flate2`: the one piece the independent decoders share with the encoders (see the header).
fn inflate_zlib(data: &[u8]) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    flate2::read::ZlibDecoder::new(data).read_to_end(&mut out).map_err(|e| format!("inflate: {e}"))?;
    Ok(out)
}

/// Decode an 8-bit RGB, non-interlaced PNG to packed RGB8. Every chunk CRC is
/// checked, IEND must be the last thing in the file.
fn decode_png(bytes: &[u8]) -> Result<(usize, usize, Vec<u8>), String> {
    if bytes.get(..8) != Some(b"\x89PNG\r\n\x1a\n".as_slice()) {
        return Err("not a PNG signature".into());
    }
    let (mut p, mut w, mut h, mut idat, mut ended) = (8usize, 0usize, 0usize, Vec::new(), false);
    while p < bytes.len() {
        let len = be32(bytes, p)? as usize;
        let body = bytes.get(p + 4..p + 8 + len).ok_or("chunk runs past the file")?;
        if crc32(body) != be32(bytes, p + 8 + len)? {
            return Err(format!("CRC mismatch in {:?} at {p}", String::from_utf8_lossy(&body[..4])));
        }
        let (kind, data) = (&body[..4], &body[4..]);
        match kind {
            b"IHDR" => {
                w = be32(data, 0)? as usize;
                h = be32(data, 4)? as usize;
                if data[8..13] != [8, 2, 0, 0, 0] {
                    return Err(format!("IHDR is not 8-bit RGB, deflate, no interlace: {:?}", &data[8..13]));
                }
            }
            b"IDAT" => idat.extend_from_slice(data),
            b"IEND" => ended = true,
            _ => {}
        }
        p += 12 + len;
        if ended {
            break;
        }
    }
    if !ended || p != bytes.len() {
        return Err("no IEND, or bytes after it".into());
    }
    let raw = inflate_zlib(&idat)?;
    let stride = w * 3;
    if raw.len() != h * (stride + 1) {
        return Err(format!("inflated {} bytes, expected {}", raw.len(), h * (stride + 1)));
    }
    let mut out = vec![0u8; w * h * 3];
    for y in 0..h {
        let line = &raw[y * (stride + 1)..(y + 1) * (stride + 1)];
        let (filter, src) = (line[0], &line[1..]);
        let (prev, cur) = out.split_at_mut(y * stride);
        let up: &[u8] = if y == 0 { &[] } else { &prev[(y - 1) * stride..] };
        let cur = &mut cur[..stride];
        for i in 0..stride {
            let a = if i >= 3 { cur[i - 3] as i16 } else { 0 };
            let b = up.get(i).map_or(0, |&v| v as i16);
            let c = if i >= 3 { up.get(i - 3).map_or(0, |&v| v as i16) } else { 0 };
            let pred = match filter {
                0 => 0,
                1 => a,
                2 => b,
                3 => (a + b) / 2,
                4 => {
                    let p = a + b - c;
                    let (pa, pb, pc) = ((p - a).abs(), (p - b).abs(), (p - c).abs());
                    if pa <= pb && pa <= pc {
                        a
                    } else if pb <= pc {
                        b
                    } else {
                        c
                    }
                }
                f => return Err(format!("row {y}: filter type {f}")),
            };
            cur[i] = src[i].wrapping_add(pred as u8);
        }
    }
    Ok((w, h, out))
}

// ---------------------------------------------------------------------------
// The independent BigTIFF decoder
// ---------------------------------------------------------------------------

/// Little-endian unsigned integer of `n` bytes at `at`, or a `truncated` error.
fn le(b: &[u8], at: usize, n: usize) -> Result<u64, String> {
    let s = b.get(at..at + n).ok_or_else(|| format!("truncated at {at}"))?;
    Ok(s.iter().rev().fold(0u64, |acc, &x| (acc << 8) | x as u64))
}

/// What the decoder saw, beyond the pixels.
struct TiffInfo {
    compression: u64,
    predictor: u64,
    rows_per_strip: usize,
    strips: usize,
}

/// Decode a little-endian BigTIFF holding one chunky 8-bit RGB image,
/// uncompressed or Deflate, with or without the horizontal predictor.
fn decode_bigtiff(b: &[u8]) -> Result<(usize, usize, Vec<u8>, TiffInfo), String> {
    if b.get(..2) != Some(b"II".as_slice()) || le(b, 2, 2)? != 43 || le(b, 4, 2)? != 8 || le(b, 6, 2)? != 0 {
        return Err("not a little-endian BigTIFF header".into());
    }
    let ifd = le(b, 8, 8)? as usize;
    let n = le(b, ifd, 8)? as usize;
    let mut tags = std::collections::HashMap::<u64, Vec<u64>>::new();
    for e in 0..n {
        let at = ifd + 8 + e * 20;
        let (tag, typ, count) = (le(b, at, 2)?, le(b, at + 2, 2)?, le(b, at + 4, 8)? as usize);
        let size = match typ {
            3 => 2,
            4 => 4,
            16 => 8,
            _ => continue, // RATIONAL etc.: nothing this decoder needs
        };
        let base = if count * size <= 8 { at + 12 } else { le(b, at + 12, 8)? as usize };
        tags.insert(tag, (0..count).map(|i| le(b, base + i * size, size)).collect::<Result<_, _>>()?);
    }
    if le(b, ifd + 8 + n * 20, 8)? != 0 {
        return Err("more than one IFD".into());
    }
    let one = |t: u64| tags.get(&t).and_then(|v| v.first().copied()).ok_or_else(|| format!("missing tag {t}"));
    let (w, h) = (one(256)? as usize, one(257)? as usize);
    if tags.get(&258) != Some(&vec![8, 8, 8]) || one(262)? != 2 || one(277)? != 3 || tags.get(&284).is_some_and(|v| v != &vec![1]) {
        return Err("not chunky 8-bit RGB".into());
    }
    let (compression, predictor) = (one(259)?, tags.get(&317).map_or(1, |v| v[0]));
    let rps = one(278)? as usize;
    let (offs, counts) = (tags.get(&273).ok_or("no StripOffsets")?, tags.get(&279).ok_or("no StripByteCounts")?);
    if offs.len() != counts.len() || offs.len() != h.div_ceil(rps) {
        return Err(format!("{} offsets, {} counts, {} strips expected", offs.len(), counts.len(), h.div_ceil(rps)));
    }
    let row = w * 3;
    let mut out = Vec::with_capacity(w * h * 3);
    for (s, (&o, &c)) in offs.iter().zip(counts).enumerate() {
        let data = b.get(o as usize..(o + c) as usize).ok_or("strip runs past the file")?;
        let mut strip = match compression {
            1 => data.to_vec(),
            8 | 32946 => inflate_zlib(data)?,
            m => return Err(format!("compression {m}")),
        };
        let rows = rps.min(h - s * rps);
        if strip.len() != rows * row {
            return Err(format!("strip {s}: {} bytes, expected {}", strip.len(), rows * row));
        }
        if predictor == 2 {
            for r in strip.chunks_exact_mut(row) {
                for i in 3..row {
                    r[i] = r[i].wrapping_add(r[i - 3]);
                }
            }
        } else if predictor != 1 {
            return Err(format!("predictor {predictor}"));
        }
        out.extend_from_slice(&strip);
    }
    Ok((w, h, out, TiffInfo { compression, predictor, rows_per_strip: rps, strips: offs.len() }))
}

/// Dispatch to the independent PNG or BigTIFF decoder and return `(width, height, packed RGB8)`.
fn decode(format: StreamFormat, bytes: &[u8]) -> Result<(usize, usize, Vec<u8>), String> {
    match format {
        StreamFormat::Png => decode_png(bytes),
        StreamFormat::BigTiff => decode_bigtiff(bytes).map(|(w, h, px, _)| (w, h, px)),
        StreamFormat::Jpeg { .. } => decode_jpeg(bytes).map(|(w, h, px, _)| (w, h, px)),
    }
}

/// Number of byte positions at which two equal-length buffers differ; asserts equal length so a truncated decode cannot look like "fewer differences".
fn differing(a: &[u8], b: &[u8]) -> usize {
    assert_eq!(a.len(), b.len());
    a.iter().zip(b).filter(|(x, y)| x != y).count()
}

// ---------------------------------------------------------------------------
// The round trips
// ---------------------------------------------------------------------------

/// Stream `(width, plans)` through `format`, decode every file independently,
/// and compare every byte with the monolithic render. Also asserts that the
/// file itself is the same bytes whatever the band plan.
fn round_trip(format: StreamFormat, width: usize, rows_per_band: &[usize]) {
    let (field, temp, rain, flow) = fixture(GW, GH);
    let a = appearance();
    let ctx = RenderCtx::with_appearance(&field, &temp, &rain, Some(&flow), GW, GH, SEA, true, 55.0, 5.0, a.clone());
    let mask = river_mask();
    let ink = ExportRivers { save_flag: Some(&mask), vector: None };
    let (w, h) = render::bake_dims(width, GW, GH);
    let whole = monolithic(&ctx, &a, ink, w, h);
    let mut single: Option<Vec<u8>> = None;
    let mut short_finals = 0;
    for &rows in rows_per_band {
        let plan = ExportBandPlan::with_rows(&a, w, h, rows);
        let last = plan.bands().last().unwrap();
        short_finals += usize::from(last.rows < plan.rows_per_band);
        let path = scratch(&format!("{format:?}_{width}_{rows}"));
        let written = export_banded(&ctx, &BakeFields::new(&ctx), ink, &plan, format, ColorSpace::Srgb, &path).unwrap_or_else(|e| panic!("{format:?} {w}x{h} at {rows} rows/band: {e}"));
        let file = std::fs::read(&path).unwrap();
        let _ = std::fs::remove_file(&path);
        assert_eq!(written, file.len() as u64, "reported size is not the file's");
        let (dw, dh, px) = decode(format, &file).unwrap_or_else(|e| panic!("{format:?} {w}x{h} at {rows} rows/band does not decode: {e}"));
        assert_eq!((dw, dh), (w, h));
        let d = differing(&px, &whole);
        println!(
            "{format:?} {w}x{h}: {} rows/band, {} bands (last {} rows), apron {}: {} bytes on disk, {d} differing of {}",
            plan.rows_per_band,
            plan.band_count(),
            last.rows,
            plan.apron,
            file.len(),
            whole.len()
        );
        assert_eq!(d, 0, "{format:?} {w}x{h} at {rows} rows/band: {d} bytes differ from the monolithic render");
        match &single {
            None => {
                assert_eq!(plan.band_count(), 1, "the first plan is the single-band control");
                single = Some(file);
            }
            Some(first) => assert!(file == *first, "{format:?} at {rows} rows/band wrote different file bytes from the single-band export"),
        }
    }
    assert!(short_finals >= 2, "only {short_finals} plans ended on a short band");
}

/// Protects: the streamed PNG, decoded by this file's own independent decoder, equals the monolithic render byte for byte at 512 and 300 pixels wide across six and four band plans (including short final bands), and the file bytes do not depend on the band plan.
#[test]
fn png_round_trips_through_an_independent_decoder_at_every_band_geometry() {
    // 512 x 361: 361 = 19 * 19, so 19 rows/band divides exactly (the control);
    // every other plan leaves a short final band (61, 41, 28, 1 rows).
    round_trip(StreamFormat::Png, 512, &[usize::MAX, 100, 64, 37, 19, 5]);
    // 300 x 211, a prime height: every banded plan ends short.
    round_trip(StreamFormat::Png, 300, &[usize::MAX, 50, 16, 7]);
}

/// Protects: the same round trip for BigTIFF (Deflate with horizontal predictor), decoded by the independent BigTIFF decoder, at the same widths and band plans.
#[test]
fn bigtiff_round_trips_through_an_independent_decoder_at_every_band_geometry() {
    round_trip(StreamFormat::BigTiff, 512, &[usize::MAX, 100, 64, 37, 19, 5]);
    round_trip(StreamFormat::BigTiff, 300, &[usize::MAX, 50, 16, 7]);
}

/// Protects: the BigTIFF writer really sets Deflate (8) and the horizontal predictor (2), cuts 170-row strips (256 KiB over 1536-byte rows) giving three strips at 361 rows, and compresses to under half the raw size, so strip re-cutting across 37-row bands is exercised.
///
/// The BigTIFF is really compressed, really predicted, and really cut into
/// several strips whose last is short, so the strip re-cutting across band
/// boundaries is in the path rather than one strip per file.
#[test]
fn the_bigtiff_is_deflated_predicted_and_multi_strip() {
    let (field, temp, rain, flow) = fixture(GW, GH);
    let a = appearance();
    let ctx = RenderCtx::with_appearance(&field, &temp, &rain, Some(&flow), GW, GH, SEA, false, 55.0, 5.0, a.clone());
    let (w, h) = render::bake_dims(512, GW, GH);
    let plan = ExportBandPlan::with_rows(&a, w, h, 37);
    let path = scratch("tiff_shape");
    export_banded(&ctx, &BakeFields::new(&ctx), ExportRivers::default(), &plan, StreamFormat::BigTiff, ColorSpace::Srgb, &path).unwrap();
    let file = std::fs::read(&path).unwrap();
    let _ = std::fs::remove_file(&path);
    let (_, _, px, info) = decode_bigtiff(&file).unwrap();
    println!("{w}x{h}: {} strips of {} rows, {} bytes on disk against {} raw", info.strips, info.rows_per_strip, file.len(), px.len());
    assert_eq!(info.compression, 8, "Compression tag is not Deflate");
    assert_eq!(info.predictor, 2, "Predictor tag is not horizontal");
    // 256 KiB / (512 * 3) = 170 rows per strip, so 361 rows is 3 strips, the
    // last of 21 rows -- and 37-row bands straddle every strip boundary.
    assert_eq!((info.rows_per_strip, info.strips), (170, 3));
    assert!(file.len() * 2 < px.len(), "{} bytes on disk for {} raw -- the strips are not compressed", file.len(), px.len());
}

/// Protects: at 2048 wide the streamed export (one band and banded, PNG and BigTIFF) decodes to the same pixels as the shipped `encode_png_rgb8` path and the monolithic render; it deliberately does not claim the PNG files are byte-identical.
///
/// **The streamed file is the shipped export's picture.** At 2048 wide the
/// shipped path (`export_raster_png`'s render, then
/// `cartalith_assets::raster::encode_png_rgb8`, i.e. `image`'s PNG encoder)
/// and the streamed path — both as one whole-raster band from
/// `for_budget`, and banded — decode to the same bytes, all equal to the
/// monolithic render.
///
/// The PNG **files** are not byte-identical, and nothing here claims they
/// are: at the same `Compression::Fast` the whole-image `write_image_data`
/// path and the `StreamWriter` path do not emit the same stream (measured
/// 2026-09-23 on this fixture: 1 271 456 streamed against 1 271 728 shipped).
/// The image is identical, which is the claim.
#[test]
fn the_streamed_2k_export_is_the_shipped_export_pixel_for_pixel() {
    let (field, temp, rain, flow) = fixture(GW, GH);
    let a = appearance();
    let ctx = RenderCtx::with_appearance(&field, &temp, &rain, Some(&flow), GW, GH, SEA, true, 55.0, 5.0, a.clone());
    let mask = river_mask();
    let ink = ExportRivers { save_flag: Some(&mask), vector: None };
    let (w, h) = render::bake_dims(2048, GW, GH);
    let whole = monolithic(&ctx, &a, ink, w, h);
    let shipped = cartalith_assets::raster::encode_png_rgb8(w as u32, h as u32, whole.clone(), cartalith_assets::raster::PngColour::Srgb).unwrap();
    let (_, _, shipped_px) = decode_png(&shipped).expect("the independent decoder cannot read the shipped encoder's PNG");
    assert_eq!(differing(&shipped_px, &whole), 0, "the independent decoder misreads the shipped PNG");

    let one = ExportBandPlan::for_budget(&a, w, h, (w * h) as u64);
    assert_eq!((one.band_count(), one.apron), (1, 0), "a whole-raster budget is not one band");
    let banded = ExportBandPlan::for_budget(&a, w, h, (w * 300) as u64);
    assert!(banded.band_count() > 1 && !h.is_multiple_of(banded.rows_per_band));
    for (label, plan) in [("one band", one), ("banded", banded)] {
        for format in [StreamFormat::Png, StreamFormat::BigTiff] {
            let path = scratch(&format!("2k_{format:?}_{}", plan.band_count()));
            export_banded(&ctx, &BakeFields::new(&ctx), ink, &plan, format, ColorSpace::Srgb, &path).unwrap();
            let file = std::fs::read(&path).unwrap();
            let _ = std::fs::remove_file(&path);
            let (_, _, px) = decode(format, &file).unwrap();
            let d = differing(&px, &shipped_px);
            println!(
                "{w}x{h} {format:?} {label}: {} bands of {} rows (apron {}), {} bytes (shipped PNG {}), {d} pixels bytes differ from the shipped export",
                plan.band_count(),
                plan.rows_per_band,
                plan.apron,
                file.len(),
                shipped.len()
            );
            assert_eq!(d, 0, "{format:?} {label} differs from the shipped 2K export");
        }
    }
}

/// Protects: the decoders are themselves sound: the BigTIFF decoder reads a file from the `tiff` crate's own writer, and both decoders reject a file with one flipped byte (the BigTIFF check accepts either an error or wrong pixels).
///
/// The decoders earn their verdicts: each reads a file from an encoder that is
/// neither this module nor itself, and each refuses a corrupted file.
#[test]
fn the_independent_decoders_are_not_vacuous() {
    // A third-party BigTIFF: the tiff crate's own whole-image writer (the path
    // that DOES install its compressor), Deflate + horizontal predictor.
    let (w, h) = (97usize, 61usize);
    let px: Vec<u8> = (0..w * h * 3).map(|i| ((i * 7919) % 251) as u8 ^ (i / (w * 3)) as u8).collect();
    let mut cur = std::io::Cursor::new(Vec::new());
    tiff::encoder::TiffEncoder::new_big(&mut cur)
        .unwrap()
        .with_compression(tiff::encoder::Compression::Deflate(tiff::encoder::DeflateLevel::Balanced))
        .with_predictor(tiff::encoder::Predictor::Horizontal)
        .write_image::<tiff::encoder::colortype::RGB8>(w as u32, h as u32, &px)
        .unwrap();
    let (dw, dh, got, info) = decode_bigtiff(cur.get_ref()).expect("cannot read the tiff crate's own BigTIFF");
    assert_eq!((dw, dh, info.compression, info.predictor), (w, h, 8, 2));
    assert_eq!(got, px, "misreads the tiff crate's own BigTIFF");

    // Corruption is caught, not decoded into something plausible.
    let mut stream = std::io::Cursor::new(Vec::new());
    write_bands(&mut stream, StreamFormat::Png, ColorSpace::Srgb, w, h, [px.clone()]).unwrap();
    let mut bad = stream.into_inner();
    assert_eq!(decode_png(&bad).unwrap().2, px);
    let mid = bad.len() / 2;
    bad[mid] ^= 0x40;
    assert!(decode_png(&bad).is_err(), "a flipped byte in the PNG went unnoticed");

    let mut stream = std::io::Cursor::new(Vec::new());
    write_bands(&mut stream, StreamFormat::BigTiff, ColorSpace::Srgb, w, h, [px.clone()]).unwrap();
    let mut bad = stream.into_inner();
    assert_eq!(decode_bigtiff(&bad).unwrap().2, px);
    // Inside the first strip's compressed data, which starts at byte 16.
    bad[40] ^= 0x40;
    assert!(decode_bigtiff(&bad).map_or(true, |d| d.2 != px), "a flipped byte in the BigTIFF went unnoticed");
}

/// Protects: `write_bands` returns an error (no panic, no short or padded file) in every format, JPEG included (E5), for bands that are too few rows, too many rows, ragged within a row, empty, or followed by a band past the last row; the ragged band's own message survives; it accepts an exact tiling, and `export_banded` errors on an unwritable destination.
///
/// Bands that do not tile the image are refused with a message, never
/// written as a short or padded file, and never a panic.
#[test]
fn bands_that_do_not_tile_the_image_are_refused() {
    let (w, h) = (16usize, 10usize);
    let row = w * 3;
    for format in [StreamFormat::Png, StreamFormat::BigTiff, StreamFormat::JPEG] {
        let sink = || std::io::Cursor::new(Vec::new());
        let short = write_bands(sink(), format, ColorSpace::Srgb, w, h, [vec![0u8; 4 * row], vec![0u8; 5 * row]]);
        let long = write_bands(sink(), format, ColorSpace::Srgb, w, h, [vec![0u8; 6 * row], vec![0u8; 5 * row]]);
        let ragged = write_bands(sink(), format, ColorSpace::Srgb, w, h, [vec![0u8; 4 * row + 3], vec![0u8; 6 * row - 3]]);
        let empty = write_bands(sink(), format, ColorSpace::Srgb, 0, h, Vec::<Vec<u8>>::new());
        // E5: a band after the image is complete. The JPEG encoder stops
        // pulling at the last row, so only its writer's drain can see this.
        let trailing = write_bands(sink(), format, ColorSpace::Srgb, w, h, [vec![0u8; 10 * row], vec![0u8; row]]);
        println!("{format:?}: short {short:?} / long {long:?} / ragged {ragged:?} / empty {empty:?} / trailing {trailing:?}");
        assert!(short.is_err() && long.is_err() && ragged.is_err() && empty.is_err() && trailing.is_err());
        // The band stream's own message reaches the caller, not the I/O error
        // the JPEG writer raises to stop its encoder.
        assert!(ragged.unwrap_err().contains("whole number"), "{format:?}: the ragged band's own message was lost");
        assert!(write_bands(sink(), format, ColorSpace::Srgb, w, h, [vec![0u8; 4 * row], vec![0u8; 6 * row]]).is_ok());
    }
    // An unwritable destination: a path *under a file*.
    let blocker = scratch("blocker");
    std::fs::write(&blocker, b"x").unwrap();
    let (field, temp, rain, flow) = fixture(GW, GH);
    let a = appearance();
    let ctx = RenderCtx::with_appearance(&field, &temp, &rain, Some(&flow), GW, GH, SEA, false, 55.0, 5.0, a.clone());
    let plan = ExportBandPlan::with_rows(&a, 64, 45, 16);
    let err = export_banded(&ctx, &BakeFields::new(&ctx), ExportRivers::default(), &plan, StreamFormat::Png, ColorSpace::Srgb, &blocker.join("map.png"));
    let _ = std::fs::remove_file(&blocker);
    println!("under a file: {err:?}");
    assert!(err.is_err());
}

/// Protects: nothing by assertion -- an `#[ignore]`d manual harness that streams one large export and prints the plan so a host-measured peak memory can be read against it.
///
/// **The peak-memory harness, not a test.** Streams one large export of a
/// closed-form world at the app's own `2048 × 1311` grid, so the plan's apron
/// and band arithmetic are the real ones. Peak resident set is measured by the
/// host polling this process, as `_exportbig_probe.gd` did for the monolithic
/// path; this prints the plan so the number can be read against it.
///
/// `CARTALITH_STREAM_WIDTH` (default 32768), `CARTALITH_STREAM_FORMAT`
/// (`png` | `tiff`, default png), `CARTALITH_STREAM_ROWS` (rows per band,
/// default 1024), `CARTALITH_STREAM_OUT` (default a temp file, removed after),
/// `CARTALITH_STREAM_MONO_RAW` (optional: also write the monolithic render's
/// raw RGB8 bytes there, for an external decoder to compare against).
#[test]
#[ignore = "minutes long and allocates ~1-2 GB; run by hand with --ignored"]
fn streamed_export_peak_memory_harness() {
    let env = |k: &str, d: &str| std::env::var(k).unwrap_or_else(|_| d.to_string());
    let width: usize = env("CARTALITH_STREAM_WIDTH", "32768").parse().unwrap();
    let rows: usize = env("CARTALITH_STREAM_ROWS", "1024").parse().unwrap();
    let format = match env("CARTALITH_STREAM_FORMAT", "png").as_str() {
        "png" => StreamFormat::Png,
        "tiff" => StreamFormat::BigTiff,
        f => panic!("CARTALITH_STREAM_FORMAT={f}: expected png or tiff"),
    };
    let (gw, gh) = (2048usize, 1311usize);
    let (field, temp, rain, flow) = fixture(gw, gh);
    let a = TerrainAppearance::default();
    let ctx = RenderCtx::with_appearance(&field, &temp, &rain, Some(&flow), gw, gh, SEA, false, 55.0, 5.0, a.clone());
    let (w, h) = render::bake_dims(width, gw, gh);
    let plan = ExportBandPlan::with_rows(&a, w, h, rows);
    let path = std::env::var("CARTALITH_STREAM_OUT").map(PathBuf::from).unwrap_or_else(|_| scratch("peak"));
    let started = std::time::Instant::now();
    let bytes = export_banded(&ctx, &BakeFields::new(&ctx), ExportRivers::default(), &plan, format, ColorSpace::Srgb, &path).unwrap();
    println!(
        "PEAK-HARNESS {format:?} {w}x{h}: {} bands of {} rows, apron {}, largest rendered band {} rows = {} px; {} bytes written in {:.1} s",
        plan.band_count(),
        plan.rows_per_band,
        plan.apron,
        plan.bands().map(|b| b.rows + b.top + b.bottom).max().unwrap(),
        plan.bands().map(|b| b.rows + b.top + b.bottom).max().unwrap() * w,
        bytes,
        started.elapsed().as_secs_f64()
    );
    if std::env::var("CARTALITH_STREAM_OUT").is_err() {
        let _ = std::fs::remove_file(&path);
    }
    // For an external decoder to compare against: the monolithic render's
    // raw RGB8 bytes. Only sensible at a size the monolithic path can hold.
    if let Ok(raw) = std::env::var("CARTALITH_STREAM_MONO_RAW") {
        std::fs::write(raw, monolithic(&ctx, &a, ExportRivers::default(), w, h)).unwrap();
    }
}

// ---------------------------------------------------------------------------
// ET-E1: the colour label (`EDITING_TOOLS_SCOPE.md` section 6, Ruling CB item 1)
// ---------------------------------------------------------------------------

use export_stream::png_chunks::{assert_display_p3_profile, iccp_profile, labels};

/// The 37 x 23 RGB fixture `cartalith-assets`' colour tests and the HEAD
/// scratch run use (an odd size, every channel varying).
fn label_fixture() -> (usize, usize, Vec<u8>) {
    let (w, h) = (37usize, 23usize);
    (w, h, (0..(w * h * 3) as u32).map(|i| ((i * 7919 + i / 111 * 13) % 251) as u8).collect())
}

/// FNV-1a 64 (standard offset basis and prime): a change detector only.
fn fnv(b: &[u8]) -> u64 {
    assert!(!b.is_empty(), "an empty buffer hashes to the basis");
    b.iter().fold(0xcbf2_9ce4_8422_2325u64, |h, &x| (h ^ x as u64).wrapping_mul(0x0000_0100_0000_01b3))
}

/// FNV-1a 64 of HEAD's (`608d4435`) `write_png` output on [`label_fixture`]
/// (one band), produced by a scratch crate running HEAD's function verbatim
/// (`git show HEAD:…/export_stream.rs`) against `png` 0.18.1 -- never by the
/// code under test. It printed `fnv=0xda7ba4ee9c24a1ba len=3036`. The band
/// split does not matter: the module doc's own claim, tested above.
const HEAD_STREAM_FNV: u64 = 0xda7b_a4ee_9c24_a1ba;

/// The `sRGB` chunk for rendering intent 0, every byte from the PNG
/// specification: length 1, type, intent 0 (perceptual), CRC-32 `AECE1CE9`
/// (checked independently in `cartalith-assets`' raster tests).
const SRGB_CHUNK: [u8; 13] = [0, 0, 0, 1, b's', b'R', b'G', b'B', 0, 0xAE, 0xCE, 0x1C, 0xE9];

/// Write `px` through the streaming writer in `space`, in three bands.
fn stream(w: usize, h: usize, px: &[u8], space: ColorSpace) -> Result<Vec<u8>, String> {
    let row = w * 3;
    let mut out = std::io::Cursor::new(Vec::new());
    let bands = vec![px[..5 * row].to_vec(), px[5 * row..17 * row].to_vec(), px[17 * row..].to_vec()];
    write_bands(&mut out, StreamFormat::Png, space, w, h, bands)?;
    Ok(out.into_inner())
}

/// Protects: the default streamed PNG -- exactly one `sRGB` chunk (intent
/// perceptual) straight after `IHDR`, no `iCCP`, the same pixels, and
/// otherwise HEAD's file byte for byte (re-baselined 2026-10-11 under Ruling CB
/// item 1: the file gains one chunk, its pixels do not move).
#[test]
fn a_default_streamed_png_is_head_s_file_plus_one_srgb_chunk() {
    let (w, h, px) = label_fixture();
    let file = stream(w, h, &px, ColorSpace::Srgb).unwrap();
    assert_eq!(labels(&file), (1, 0), "one sRGB, no iCCP");
    assert_eq!(&file[33..46], &SRGB_CHUNK, "the sRGB chunk must follow IHDR and say perceptual");
    assert_eq!(decode_png(&file).unwrap().2, px, "the pixels moved");
    let mut head = file.clone();
    head.drain(33..46);
    assert_eq!(fnv(&head), HEAD_STREAM_FNV, "apart from the sRGB chunk the file is no longer HEAD's");
}

/// Protects: the Display P3 streamed PNG -- exactly one `iCCP`, no `sRGB`; the
/// embedded profile is `display_p3_icc()` and parses back as Display P3
/// (primaries, D65 white, sRGB-shaped curve, cicp 12); the pixels are written
/// as given (the label converts nothing); and two encodes are the same bytes
/// (the profile's clock-stamped date is pinned).
#[test]
fn a_p3_streamed_png_embeds_the_display_p3_profile_and_no_srgb() {
    let (w, h, px) = label_fixture();
    let file = stream(w, h, &px, ColorSpace::DisplayP3).unwrap();
    assert_eq!(labels(&file), (0, 1), "one iCCP, no sRGB");
    let icc = iccp_profile(&file).expect("an iCCP profile");
    assert_eq!(icc.as_slice(), export_stream::display_p3_icc().unwrap(), "the embedded profile is not the P3 profile");
    assert_display_p3_profile(&icc);
    assert_eq!(decode_png(&file).unwrap().2, px, "the label moved pixels");
    assert!(stream(w, h, &px, ColorSpace::DisplayP3).unwrap() == file, "two identical P3 exports differ");
    // The header date is the pinned literal 2026-10-11 00:00:00 (six
    // big-endian u16s at bytes 24..36), not moxcms' clock: within one process
    // the profile is cached, so only this literal can see a clock stamp.
    assert_eq!(&icc[24..36], &[0x07, 0xEA, 0, 10, 0, 11, 0, 0, 0, 0, 0, 0], "the profile date is not pinned");
}

/// Protects: the assets encoder (`encode_png_rgb8`, behind the whole-file,
/// tile, crop and preview paths) labels exactly as the streaming writer does:
/// the same single `sRGB` chunk by default and the same embedded P3 profile in
/// P3, through the one mapping `export_stream::png_colour`.
#[test]
fn the_assets_encoder_labels_like_the_streaming_writer() {
    let (w, h, px) = label_fixture();
    let s = cartalith_assets::raster::encode_png_rgb8(w as u32, h as u32, px.clone(), export_stream::png_colour(ColorSpace::Srgb).unwrap()).unwrap();
    assert_eq!(labels(&s), (1, 0));
    assert_eq!(&s[33..46], &SRGB_CHUNK);
    let p = cartalith_assets::raster::encode_png_rgb8(w as u32, h as u32, px.clone(), export_stream::png_colour(ColorSpace::DisplayP3).unwrap()).unwrap();
    assert_eq!(labels(&p), (0, 1));
    assert_eq!(iccp_profile(&p).unwrap().as_slice(), export_stream::display_p3_icc().unwrap());
    assert_eq!(decode_png(&p).unwrap().2, px);
}

/// Protects: a BigTIFF in Display P3 is refused (the TIFF writer embeds no
/// profile, so P3 numbers would be read as sRGB) by every entry point, and
/// `export_banded` leaves no file; an sRGB BigTIFF is still accepted.
#[test]
fn a_p3_bigtiff_is_refused_everywhere() {
    let (w, h, px) = label_fixture();
    let mut out = std::io::Cursor::new(Vec::new());
    assert!(write_bands(&mut out, StreamFormat::BigTiff, ColorSpace::DisplayP3, w, h, [px.clone()]).is_err());
    assert!(out.get_ref().is_empty(), "a refused write still wrote bytes");
    assert!(export_stream::check_format_space(StreamFormat::BigTiff, ColorSpace::DisplayP3).is_err());
    assert!(export_stream::check_format_space(StreamFormat::BigTiff, ColorSpace::Srgb).is_ok());
    assert!(export_stream::check_format_space(StreamFormat::Png, ColorSpace::DisplayP3).is_ok());
    let (field, temp, rain, flow) = fixture(GW, GH);
    let a = appearance();
    let ctx = RenderCtx::with_appearance(&field, &temp, &rain, Some(&flow), GW, GH, SEA, false, 55.0, 5.0, a.clone());
    let plan = ExportBandPlan::with_rows(&a, 64, 45, 16);
    let path = scratch("p3_tiff");
    assert!(export_banded(&ctx, &BakeFields::new(&ctx), ExportRivers::default(), &plan, StreamFormat::BigTiff, ColorSpace::DisplayP3, &path).is_err());
    assert!(!path.exists(), "a refused P3 BigTIFF left a file");
}

/// The screen texture's bytes for the fixture world in `space`:
/// `build_color_texture`'s quantised `cell_color` raster, then
/// `render::finish_screen_texture` -- the function the product's screen path
/// calls (no river colour field, no asset pack).
fn screen(ctx: &RenderCtx, world: bool, space: ColorSpace) -> Vec<u8> {
    let q = |v: f64| (v.clamp(0.0, 1.0) * 255.0) as u8;
    let mut bytes = vec![0u8; GW * GH * 3];
    for i in 0..GW * GH {
        let c = render::cell_color(ctx, i % GW, i / GW);
        bytes[i * 3..i * 3 + 3].copy_from_slice(&[q(c.0), q(c.1), q(c.2)]);
    }
    let inf = render::build_grade_influence(ctx, GW, GH);
    render::finish_screen_texture(ctx, &mut bytes, None, GW, GH, world, &inf, space);
    assert!(!bytes.is_empty());
    bytes
}

/// Protects: ET-E1's "a P3 export's pixels equal the screen's in P3 mode,
/// byte for byte". At the grid's own width the export (`bake_and_finish`, and
/// the streamed banded file decoded) equals the screen texture in sRGB -- the
/// control, which must hold or the P3 leg proves nothing -- and in Display P3;
/// and P3 really moved pixels (a positive control).
#[test]
fn the_p3_export_is_the_screen_s_p3_encode_byte_for_byte() {
    let (field, temp, rain, flow) = fixture(GW, GH);
    let a = appearance();
    let world = false;
    let ctx = RenderCtx::with_appearance(&field, &temp, &rain, Some(&flow), GW, GH, SEA, world, 55.0, 5.0, a.clone());
    let (w, h) = render::bake_dims(GW, GW, GH);
    assert_eq!((w, h), (GW, GH), "premise: the export is at the grid's own size");
    let bf = BakeFields::new(&ctx);
    let inf = render::build_grade_influence(&ctx, w, h);
    let mut seen = Vec::new();
    for space in [ColorSpace::Srgb, ColorSpace::DisplayP3] {
        let export = render::bake_and_finish(&ctx, &bf, ExportRivers::default(), w, h, &inf, space);
        let scr = screen(&ctx, world, space);
        let d = differing(&export, &scr);
        println!("{space:?}: export vs screen differ in {d} of {} bytes", scr.len());
        assert_eq!(d, 0, "{space:?}: the export is not the screen");
        let path = scratch(&format!("p3_screen_{space:?}"));
        let plan = ExportBandPlan::with_rows(&a, w, h, 9);
        export_banded(&ctx, &bf, ExportRivers::default(), &plan, StreamFormat::Png, space, &path).unwrap();
        let file = std::fs::read(&path).unwrap();
        let _ = std::fs::remove_file(&path);
        assert_eq!(decode_png(&file).unwrap().2, scr, "{space:?}: the streamed, banded file is not the screen");
        assert_eq!(labels(&file), if space == ColorSpace::Srgb { (1, 0) } else { (0, 1) });
        seen.push(scr);
    }
    let moved = differing(&seen[0], &seen[1]);
    assert!(moved > 0, "Display P3 moved no byte -- the P3 leg would pass vacuously");
}

// ---------------------------------------------------------------------------
// E5: the JPEG writer (Ruling AZ, `EXPORT_SCOPE.md` E5)
// ---------------------------------------------------------------------------

/// What [`jpeg_markers`] read from a JPEG's header segments.
struct JpegInfo {
    /// Every marker byte before the scan, in file order (`0xC0` = SOF0).
    markers: Vec<u8>,
    /// The APP2 `ICC_PROFILE` payloads reassembled in sequence order, or
    /// `None` when the file has none.
    icc: Option<Vec<u8>>,
    /// Each component's `(id, horizontal, vertical)` sampling from the SOF.
    sampling: Vec<(u8, u8, u8)>,
}

/// Walk a JPEG's segments from SOI to SOS by the ITU-T T.81 B.1 layout --
/// `FF xx`, a big-endian length that counts itself, the payload -- written
/// against the standard, sharing no code with the encoder or `jpeg-decoder`.
/// Reassembles APP2 `ICC_PROFILE\0` chunks by their 1-based sequence number
/// (ICC.1 B.4). Panics on a malformed header; asserts at least one marker.
fn jpeg_markers(b: &[u8]) -> JpegInfo {
    assert_eq!(&b[..2], &[0xFF, 0xD8], "no SOI");
    assert_eq!(&b[b.len() - 2..], &[0xFF, 0xD9], "no EOI at the end");
    let (mut p, mut markers, mut chunks, mut sampling) = (2usize, Vec::new(), Vec::<(u8, u8, Vec<u8>)>::new(), Vec::new());
    loop {
        assert_eq!(b[p], 0xFF, "segment at {p} does not start with FF");
        let m = b[p + 1];
        let len = u16::from_be_bytes([b[p + 2], b[p + 3]]) as usize;
        let body = &b[p + 4..p + 2 + len];
        markers.push(m);
        if m == 0xE2 && body.starts_with(b"ICC_PROFILE\0") {
            chunks.push((body[12], body[13], body[14..].to_vec()));
        }
        if (0xC0..=0xC2).contains(&m) {
            for c in 0..body[5] as usize {
                let at = 6 + 3 * c;
                sampling.push((body[at], body[at + 1] >> 4, body[at + 1] & 0x0F));
            }
        }
        if m == 0xDA {
            break;
        }
        p += 2 + len;
    }
    assert!(!markers.is_empty(), "a JPEG with no segments proves nothing");
    chunks.sort_by_key(|c| c.0);
    let icc = (!chunks.is_empty()).then(|| {
        let n = chunks[0].1 as usize;
        assert_eq!(chunks.len(), n, "ICC chunk count disagrees with the chunks' own total");
        for (i, c) in chunks.iter().enumerate() {
            assert_eq!((c.0 as usize, c.1 as usize), (i + 1, n), "ICC chunks out of sequence");
        }
        chunks.into_iter().flat_map(|c| c.2).collect()
    });
    JpegInfo { markers, icc, sampling }
}

/// Decode a JPEG with `jpeg-decoder` (image-rs; not the encoder's code) to
/// `(width, height, RGB8, embedded ICC profile)`.
fn decode_jpeg(b: &[u8]) -> Result<(usize, usize, Vec<u8>, Option<Vec<u8>>), String> {
    let mut d = jpeg_decoder::Decoder::new(std::io::Cursor::new(b));
    let px = d.decode().map_err(|e| format!("jpeg-decoder: {e}"))?;
    let info = d.info().ok_or("no frame info")?;
    if info.pixel_format != jpeg_decoder::PixelFormat::RGB24 {
        return Err(format!("decoded as {:?}, not RGB24", info.pixel_format));
    }
    Ok((info.width as usize, info.height as usize, px, d.icc_profile()))
}

/// `(mean absolute error per channel byte, PSNR in dB, largest single error)`
/// of `got` against `want`; asserts equal non-zero lengths.
fn jpeg_error(got: &[u8], want: &[u8]) -> (f64, f64, u8) {
    assert!(!want.is_empty() && got.len() == want.len(), "{} vs {} bytes", got.len(), want.len());
    let (mut abs, mut sq, mut max) = (0u64, 0u64, 0u8);
    for (&g, &w) in got.iter().zip(want) {
        let d = g.abs_diff(w);
        abs += d as u64;
        sq += (d as u64) * (d as u64);
        max = max.max(d);
    }
    let n = want.len() as f64;
    let mse = sq as f64 / n;
    let psnr = if mse == 0.0 { f64::INFINITY } else { 10.0 * (255.0f64 * 255.0 / mse).log10() };
    (abs as f64 / n, psnr, max)
}

/// The JPEG bars [`a_jpeg_export_decodes_within_its_measured_error_at_every_band_geometry`]
/// holds the q90 fixture export to: mean absolute error at most this many
/// levels per channel byte, and PSNR at least [`JPEG_Q90_MIN_PSNR`]. **Measured,
/// then given margin** -- see that test's doc for the run that set them.
const JPEG_Q90_MAX_MAE: f64 = 1.2;

/// The PSNR floor, in dB, beside [`JPEG_Q90_MAX_MAE`].
const JPEG_Q90_MIN_PSNR: f64 = 42.0;

/// Protects: the JPEG export (q90, sRGB) decodes, with an independent
/// decoder, to the monolithic render's size and within a measured error bound,
/// at six band plans of which four are not multiples of the 8-row MCU (100,
/// 37, 19, 5) and most end short; **and the file is the same bytes at every
/// plan** -- the encoder pulls rows, so band boundaries cannot reach it
/// (`export_stream.rs` module doc). Also: baseline (SOF0, never progressive
/// SOF2), one scan, 4:4:4 (every component 1x1), JFIF APP0, no ICC in sRGB.
///
/// The bars: measured 2026-10-18 by this test's first run -- 512 x 361, 45 445
/// bytes (0.246 B/px), **MAE 1.034, PSNR 43.44 dB, largest error 40 levels**,
/// identical at all six plans -- then [`JPEG_Q90_MAX_MAE`] 1.2 and
/// [`JPEG_Q90_MIN_PSNR`] 42.0 set with a margin around it (labelled judgement,
/// not derived from the JPEG standard). The largest single error is not bounded:
/// DCT ringing at a hard edge is what JPEG does, and the mean is the claim.
#[test]
fn a_jpeg_export_decodes_within_its_measured_error_at_every_band_geometry() {
    let (field, temp, rain, flow) = fixture(GW, GH);
    let a = appearance();
    let ctx = RenderCtx::with_appearance(&field, &temp, &rain, Some(&flow), GW, GH, SEA, true, 55.0, 5.0, a.clone());
    let mask = river_mask();
    let ink = ExportRivers { save_flag: Some(&mask), vector: None };
    let (w, h) = render::bake_dims(512, GW, GH);
    let whole = monolithic(&ctx, &a, ink, w, h);
    let mut first: Option<Vec<u8>> = None;
    for rows in [usize::MAX, 100, 64, 37, 19, 5] {
        let plan = ExportBandPlan::with_rows(&a, w, h, rows);
        let path = scratch(&format!("jpeg_{rows}"));
        let written = export_banded(&ctx, &BakeFields::new(&ctx), ink, &plan, StreamFormat::JPEG, ColorSpace::Srgb, &path).unwrap();
        let file = std::fs::read(&path).unwrap();
        let _ = std::fs::remove_file(&path);
        assert_eq!(written, file.len() as u64);
        let (dw, dh, px, icc) = decode_jpeg(&file).unwrap();
        assert_eq!((dw, dh), (w, h));
        assert!(icc.is_none(), "an sRGB JPEG carries an ICC profile");
        let (mae, psnr, max) = jpeg_error(&px, &whole);
        println!("JPEG q90 {w}x{h}: {} bands of {} rows, {} bytes ({:.3} B/px), MAE {mae:.3}, PSNR {psnr:.2} dB, max {max}", plan.band_count(), plan.rows_per_band, file.len(), file.len() as f64 / (w * h) as f64);
        assert!(mae <= JPEG_Q90_MAX_MAE && psnr >= JPEG_Q90_MIN_PSNR, "q90 error MAE {mae:.3} / PSNR {psnr:.2} outside the measured bar");
        match &first {
            None => {
                assert_eq!(plan.band_count(), 1, "the first plan is the single-band control");
                let info = jpeg_markers(&file);
                assert!(info.markers.contains(&0xC0) && !info.markers.contains(&0xC2), "not baseline: {:x?}", info.markers);
                assert_eq!(info.markers.iter().filter(|&&m| m == 0xDA).count(), 1);
                assert_eq!(info.markers.first(), Some(&0xE0), "no JFIF APP0 first");
                // Factors only: `jpeg-encoder` numbers its components 0, 1, 2
                // (T.81 allows any ids; JFIF's 1, 2, 3 is a convention decoders do not need).
                assert_eq!(info.sampling.iter().map(|c| (c.1, c.2)).collect::<Vec<_>>(), vec![(1, 1); 3], "not 4:4:4: {:?}", info.sampling);
                first = Some(file);
            }
            Some(f) => assert!(file == *f, "the JPEG at {rows} rows/band is not the single-band file's bytes"),
        }
    }
}

/// Protects: quality is honoured and monotone on the fixture: higher quality
/// gives a strictly larger file and a strictly smaller error, across
/// 30 / 60 / 90 / 100 (measured 2026-10-18 on 512 x 361: 13 891 / 19 307 /
/// 38 212 / 138 753 bytes, MAE 2.59 / 1.65 / 0.89 / 0.43); and the default
/// `StreamFormat::JPEG` is quality 90 (the literal, so mutating
/// `JPEG_QUALITY_DEFAULT` turns this red).
#[test]
fn jpeg_quality_moves_size_up_and_error_down() {
    assert_eq!(StreamFormat::JPEG, StreamFormat::Jpeg { quality: 90 }, "the default JPEG quality is 90");
    let (field, temp, rain, flow) = fixture(GW, GH);
    let a = appearance();
    let ctx = RenderCtx::with_appearance(&field, &temp, &rain, Some(&flow), GW, GH, SEA, false, 55.0, 5.0, a.clone());
    let (w, h) = render::bake_dims(512, GW, GH);
    let whole = monolithic(&ctx, &a, ExportRivers::default(), w, h);
    let mut prev: Option<(usize, f64)> = None;
    for q in [30u8, 60, 90, 100] {
        let mut out = std::io::Cursor::new(Vec::new());
        write_bands(&mut out, StreamFormat::Jpeg { quality: q }, ColorSpace::Srgb, w, h, [whole.clone()]).unwrap();
        let file = out.into_inner();
        let (mae, psnr, _) = jpeg_error(&decode_jpeg(&file).unwrap().2, &whole);
        println!("q{q}: {} bytes, MAE {mae:.3}, PSNR {psnr:.2}", file.len());
        if let Some((len, e)) = prev {
            assert!(file.len() > len && mae < e, "q{q}: {} bytes / MAE {mae:.3} against the previous {len} / {e:.3}", file.len());
        }
        prev = Some((file.len(), mae));
    }
}

/// Protects: the 4:4:4 choice (`write_jpeg`'s doc): a 1 px red line on green
/// survives a q90 export with its hue, where 4:2:0 at the same quality --
/// encoded here straight through `jpeg-encoder` -- does not. Measured on the
/// line's pixels only (selected by the INPUT geometry, not by the error):
/// mean |R - G| of the decoded line, which is 255 - 120 = 135 in the source.
/// Measured 2026-10-18: **4:4:4 keeps 132.7; 4:2:0 gives -120.0** -- the red
/// line decodes greener than red, its hue averaged into the green around it.
/// Bars: 4:4:4 keeps at least 100 and beats 4:2:0 by at least 20; judgement
/// bounds far inside the measured gap.
#[test]
fn jpeg_444_keeps_a_one_pixel_coloured_line() {
    let (w, h) = (96usize, 64usize);
    let mut px = vec![0u8; w * h * 3];
    let line = |x: usize, y: usize| x == 37 || y == 21;
    for y in 0..h {
        for x in 0..w {
            let c = if line(x, y) { [255, 120, 0] } else { [40, 160, 60] };
            px[(y * w + x) * 3..][..3].copy_from_slice(&c);
        }
    }
    let hue = |dec: &[u8]| {
        let (mut sum, mut n) = (0f64, 0f64);
        for y in 0..h {
            for x in 0..w {
                if line(x, y) {
                    let p = &dec[(y * w + x) * 3..][..3];
                    sum += p[0] as f64 - p[1] as f64;
                    n += 1.0;
                }
            }
        }
        sum / n
    };
    let mut ours = std::io::Cursor::new(Vec::new());
    write_bands(&mut ours, StreamFormat::JPEG, ColorSpace::Srgb, w, h, [px.clone()]).unwrap();
    let mut sub = Vec::new();
    let mut enc = jpeg_encoder::Encoder::new(&mut sub, 90);
    enc.set_sampling_factor(jpeg_encoder::SamplingFactor::R_4_2_0);
    enc.encode(&px, w as u16, h as u16, jpeg_encoder::ColorType::Rgb).unwrap();
    let (keep, lose) = (hue(&decode_jpeg(ours.get_ref()).unwrap().2), hue(&decode_jpeg(&sub).unwrap().2));
    println!("1 px line R-G (source 135): 4:4:4 {keep:.1}, 4:2:0 {lose:.1}");
    assert!(keep >= 100.0, "4:4:4 kept only {keep:.1} of the line's 135 R-G");
    assert!(keep - lose >= 20.0, "4:2:0 ({lose:.1}) is not measurably worse than 4:4:4 ({keep:.1}) -- the reason for 4:4:4 would be unsupported");
}

/// Protects: a Display P3 JPEG embeds exactly the P3 PNG's profile
/// (`display_p3_icc`, byte for byte) as APP2 `ICC_PROFILE` segments -- read
/// both by this file's own segment walker and by `jpeg-decoder` -- which parses
/// as Display P3; an sRGB JPEG has none; the pixels' encoding is the caller's
/// (the writer converts nothing: the same bands give the same scan in both
/// spaces); two P3 encodes are the same bytes.
#[test]
fn a_p3_jpeg_embeds_the_display_p3_profile() {
    let (w, h, px) = label_fixture();
    let enc = |space| {
        let mut out = std::io::Cursor::new(Vec::new());
        write_bands(&mut out, StreamFormat::JPEG, space, w, h, [px.clone()]).unwrap();
        out.into_inner()
    };
    let (srgb, p3) = (enc(ColorSpace::Srgb), enc(ColorSpace::DisplayP3));
    assert!(jpeg_markers(&srgb).icc.is_none(), "an sRGB JPEG carries a profile");
    let icc = jpeg_markers(&p3).icc.expect("a P3 JPEG has no ICC_PROFILE");
    assert_eq!(icc.as_slice(), export_stream::display_p3_icc().unwrap(), "the embedded profile is not the P3 profile");
    assert_display_p3_profile(&icc);
    let (_, _, p3_px, p3_icc) = decode_jpeg(&p3).unwrap();
    assert_eq!(p3_icc.as_deref(), Some(icc.as_slice()), "jpeg-decoder reads a different profile");
    assert_eq!(p3_px, decode_jpeg(&srgb).unwrap().2, "the label changed the encoded pixels");
    assert!(enc(ColorSpace::DisplayP3) == p3, "two identical P3 JPEGs differ");
}

/// Protects: a JPEG wider or taller than its 16-bit frame header can hold is
/// refused before anything is written (no band is pulled), and so is a quality
/// outside 1..=100; 65 535 itself is not refused by the size check.
#[test]
fn jpeg_limits_are_refused_up_front() {
    let mut out = std::io::Cursor::new(Vec::new());
    let wide = write_bands(&mut out, StreamFormat::JPEG, ColorSpace::Srgb, 65_536, 8, Vec::<Vec<u8>>::new());
    assert!(wide.as_ref().is_err_and(|e| e.contains("65535")), "{wide:?}");
    assert!(out.get_ref().is_empty(), "a refused JPEG wrote bytes");
    for q in [0u8, 101] {
        let r = write_bands(std::io::Cursor::new(Vec::new()), StreamFormat::Jpeg { quality: q }, ColorSpace::Srgb, 8, 8, [vec![0u8; 8 * 8 * 3]]);
        assert!(r.is_err(), "quality {q} was accepted");
    }
    // 65 535 x 1 encodes: the bound is the header's, not one less.
    assert!(write_bands(std::io::Cursor::new(Vec::new()), StreamFormat::JPEG, ColorSpace::Srgb, 65_535, 1, [vec![0u8; 65_535 * 3]]).is_ok());
}
