//! The streaming export writer (`EXPORT_SCOPE.md` §7, milestone E2) —
//! `export_stream::export_banded` / `write_bands`.
//!
//! # The bar, and how each half of it is met
//!
//! §7: *"round-trip tests that decode with a different decoder than the
//! encoder and compare every byte at several band geometries including a
//! short final band."*
//!
//! - **A different decoder.** Both formats are read back by decoders written
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

#[path = "../src/export_stream.rs"]
mod export_stream;

use std::io::Read;
use std::path::PathBuf;

use export_stream::{StreamFormat, export_banded, write_bands};
use render::{BakeFields, ExportBandPlan, RenderCtx, RiverInk, TerrainAppearance};

/// `tests/export_bands.rs`'s fixture: a 61 × 43 closed-form world.
const GW: usize = 61;
const GH: usize = 43;
const SEA: f64 = 0.42;

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

/// `export_raster_png`'s closure, call for call.
fn monolithic(ctx: &RenderCtx, a: &TerrainAppearance, ink: Option<RiverInk<'_>>, w: usize, h: usize) -> Vec<u8> {
    let bf = BakeFields::new(ctx);
    let mut px = render::bake_rect(ctx, &bf, ink, w, h, 0, 0, w, h);
    render::apply_local_contrast(a, &mut px, w, h, ctx.world);
    let inf = render::build_grade_influence(ctx, w, h);
    render::apply_color_grade(a, &mut px, &inf);
    px
}

fn river_mask() -> Vec<u8> {
    (0..GW * GH).map(|i| u8::from((i % GW).is_multiple_of(3) || i % GW == i / GW)).collect()
}

fn scratch(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("cartalith_export_stream_{}_{name}", std::process::id()))
}

// ---------------------------------------------------------------------------
// The independent PNG decoder
// ---------------------------------------------------------------------------

fn be32(b: &[u8], at: usize) -> Result<u32, String> {
    b.get(at..at + 4).map(|s| u32::from_be_bytes([s[0], s[1], s[2], s[3]])).ok_or_else(|| format!("truncated at {at}"))
}

/// CRC-32 (ISO 3309, as PNG specifies it), bit by bit.
fn crc32(data: &[u8]) -> u32 {
    let mut c = 0xFFFF_FFFFu32;
    for &byte in data {
        c ^= byte as u32;
        for _ in 0..8 {
            c = if c & 1 != 0 { 0xEDB8_8320 ^ (c >> 1) } else { c >> 1 };
        }
    }
    !c
}

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

fn decode(format: StreamFormat, bytes: &[u8]) -> Result<(usize, usize, Vec<u8>), String> {
    match format {
        StreamFormat::Png => decode_png(bytes),
        StreamFormat::BigTiff => decode_bigtiff(bytes).map(|(w, h, px, _)| (w, h, px)),
    }
}

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
    let ink = Some(RiverInk::Flag(&mask));
    let (w, h) = render::bake_dims(width, GW, GH);
    let whole = monolithic(&ctx, &a, ink, w, h);
    let mut single: Option<Vec<u8>> = None;
    let mut short_finals = 0;
    for &rows in rows_per_band {
        let plan = ExportBandPlan::with_rows(&a, w, h, rows);
        let last = plan.bands().last().unwrap();
        short_finals += usize::from(last.rows < plan.rows_per_band);
        let path = scratch(&format!("{format:?}_{width}_{rows}"));
        let written = export_banded(&ctx, ink, &plan, format, &path).unwrap_or_else(|e| panic!("{format:?} {w}x{h} at {rows} rows/band: {e}"));
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

#[test]
fn png_round_trips_through_an_independent_decoder_at_every_band_geometry() {
    // 512 x 361: 361 = 19 * 19, so 19 rows/band divides exactly (the control);
    // every other plan leaves a short final band (61, 41, 28, 1 rows).
    round_trip(StreamFormat::Png, 512, &[usize::MAX, 100, 64, 37, 19, 5]);
    // 300 x 211, a prime height: every banded plan ends short.
    round_trip(StreamFormat::Png, 300, &[usize::MAX, 50, 16, 7]);
}

#[test]
fn bigtiff_round_trips_through_an_independent_decoder_at_every_band_geometry() {
    round_trip(StreamFormat::BigTiff, 512, &[usize::MAX, 100, 64, 37, 19, 5]);
    round_trip(StreamFormat::BigTiff, 300, &[usize::MAX, 50, 16, 7]);
}

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
    export_banded(&ctx, None, &plan, StreamFormat::BigTiff, &path).unwrap();
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
    let ink = Some(RiverInk::Flag(&mask));
    let (w, h) = render::bake_dims(2048, GW, GH);
    let whole = monolithic(&ctx, &a, ink, w, h);
    let shipped = cartalith_assets::raster::encode_png_rgb8(w as u32, h as u32, whole.clone()).unwrap();
    let (_, _, shipped_px) = decode_png(&shipped).expect("the independent decoder cannot read the shipped encoder's PNG");
    assert_eq!(differing(&shipped_px, &whole), 0, "the independent decoder misreads the shipped PNG");

    let one = ExportBandPlan::for_budget(&a, w, h, (w * h) as u64);
    assert_eq!((one.band_count(), one.apron), (1, 0), "a whole-raster budget is not one band");
    let banded = ExportBandPlan::for_budget(&a, w, h, (w * 300) as u64);
    assert!(banded.band_count() > 1 && !h.is_multiple_of(banded.rows_per_band));
    for (label, plan) in [("one band", one), ("banded", banded)] {
        for format in [StreamFormat::Png, StreamFormat::BigTiff] {
            let path = scratch(&format!("2k_{format:?}_{}", plan.band_count()));
            export_banded(&ctx, ink, &plan, format, &path).unwrap();
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
    write_bands(&mut stream, StreamFormat::Png, w, h, [px.clone()]).unwrap();
    let mut bad = stream.into_inner();
    assert_eq!(decode_png(&bad).unwrap().2, px);
    let mid = bad.len() / 2;
    bad[mid] ^= 0x40;
    assert!(decode_png(&bad).is_err(), "a flipped byte in the PNG went unnoticed");

    let mut stream = std::io::Cursor::new(Vec::new());
    write_bands(&mut stream, StreamFormat::BigTiff, w, h, [px.clone()]).unwrap();
    let mut bad = stream.into_inner();
    assert_eq!(decode_bigtiff(&bad).unwrap().2, px);
    // Inside the first strip's compressed data, which starts at byte 16.
    bad[40] ^= 0x40;
    assert!(decode_bigtiff(&bad).map_or(true, |d| d.2 != px), "a flipped byte in the BigTIFF went unnoticed");
}

/// Bands that do not tile the image are refused with a message, never
/// written as a short or padded file, and never a panic.
#[test]
fn bands_that_do_not_tile_the_image_are_refused() {
    let (w, h) = (16usize, 10usize);
    let row = w * 3;
    for format in [StreamFormat::Png, StreamFormat::BigTiff] {
        let sink = || std::io::Cursor::new(Vec::new());
        let short = write_bands(sink(), format, w, h, [vec![0u8; 4 * row], vec![0u8; 5 * row]]);
        let long = write_bands(sink(), format, w, h, [vec![0u8; 6 * row], vec![0u8; 5 * row]]);
        let ragged = write_bands(sink(), format, w, h, [vec![0u8; 4 * row + 3], vec![0u8; 6 * row - 3]]);
        let empty = write_bands(sink(), format, 0, h, Vec::<Vec<u8>>::new());
        println!("{format:?}: short {short:?} / long {long:?} / ragged {ragged:?} / empty {empty:?}");
        assert!(short.is_err() && long.is_err() && ragged.is_err() && empty.is_err());
        assert!(write_bands(sink(), format, w, h, [vec![0u8; 4 * row], vec![0u8; 6 * row]]).is_ok());
    }
    // An unwritable destination: a path *under a file*.
    let blocker = scratch("blocker");
    std::fs::write(&blocker, b"x").unwrap();
    let (field, temp, rain, flow) = fixture(GW, GH);
    let a = appearance();
    let ctx = RenderCtx::with_appearance(&field, &temp, &rain, Some(&flow), GW, GH, SEA, false, 55.0, 5.0, a.clone());
    let plan = ExportBandPlan::with_rows(&a, 64, 45, 16);
    let err = export_banded(&ctx, None, &plan, StreamFormat::Png, &blocker.join("map.png"));
    let _ = std::fs::remove_file(&blocker);
    println!("under a file: {err:?}");
    assert!(err.is_err());
}

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
    let bytes = export_banded(&ctx, None, &plan, format, &path).unwrap();
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
        std::fs::write(raw, monolithic(&ctx, &a, None, w, h)).unwrap();
    }
}
