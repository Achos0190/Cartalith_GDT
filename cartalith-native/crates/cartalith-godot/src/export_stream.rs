//! The streaming export writer (`EXPORT_SCOPE.md` §7, milestone E2): band in,
//! file out.
//!
//! E1 ([`render::ExportBandPlan`] / [`render::bake_export_band`]) produces a
//! 16K/32K raster one full-width band at a time; this module writes each band
//! to disk as it arrives and drops it, so the finished raster **never exists
//! in memory**. Peak memory is one band (plus its apron) through the finishing
//! passes, not the image.
//!
//! Two formats, both `EXPORT_SCOPE.md` §6.1's survivors of the streaming
//! constraint: PNG (ruling 26's format) and BigTIFF + Deflate.
//!
//! # Not yet the user-facing path
//!
//! One `#[func]` reaches this: `WorldGen::export_image` (`export_raster.rs`),
//! which reads E3's options dictionary (`export_options.rs`) and hands
//! [`export_banded`] a [`RenderCtx`] from `WorldGen::export_render_with`. No
//! menu or dialog calls that yet (E5). The shipped `export_raster_png` does
//! not go through here and is unchanged.
//!
//! # The output does not depend on the band size
//!
//! Neither encoder sees band boundaries. `png::StreamWriter` filters and
//! compresses per completed **row** whatever the `write` calls look like, and
//! the TIFF strips are a fixed number of rows ([`TIFF_STRIP_TARGET_BYTES`])
//! re-cut from the band stream. So one export yields the same file bytes at
//! every band plan — `tests/export_stream.rs` asserts that, not only that the
//! pixels decode back.
//!
//! Compiled standalone by `tests/export_stream.rs` through the same `#[path]`
//! trick `tests/export_bands.rs` uses; `crate::render` resolves to that test's
//! own `mod render` there.
#![allow(dead_code)]

use std::fs::File;
use std::io::{BufWriter, Seek, Write};
use std::path::Path;

use crate::render::{self, ExportBandPlan, RenderCtx, RiverInk};

/// The two file formats [`write_bands`] can stream.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StreamFormat {
    /// RGB8 PNG, at the settings the shipped `export_raster_png` gets from
    /// `image` 0.25.10's `PngEncoder::new`, whose `CompressionType` default is
    /// **`Fast`**, not `Default` (`#[default] Fast` in `codecs/png.rs`), mapped
    /// to `png::Compression::Fast` (`fdeflate`'s ultra-fast mode) with
    /// `Filter::Adaptive`. So `export_raster.rs`' fitted file-size model and its
    /// measured timings describe this encoder too.
    Png,
    /// RGB8 BigTIFF, Deflate-compressed with the horizontal predictor.
    BigTiff,
}

/// Uncompressed bytes per TIFF strip, the tiff crate's own default order of
/// magnitude (it uses ~1 MB) brought down so a small export still gets several
/// strips. Rows per strip is this over one row's bytes, floored at one row.
pub const TIFF_STRIP_TARGET_BYTES: usize = 256 * 1024;

/// `png`'s IDAT chunk size. Its default is 4 KiB, which at 32K is ~50 000
/// chunk headers and as many small writes; 1 MiB is a fixed size so the file
/// stays independent of the band plan.
const PNG_CHUNK_BYTES: usize = 1 << 20;

/// Render `plan` band by band with [`render::bake_export_band`] and stream it
/// to `path` as `format`. Returns the bytes written.
///
/// `ink` is the same river ink `export_raster_png` is handed
/// (`WorldGen::river_ink`); `ctx.appearance` and `ctx.world` are the look and
/// wrap, exactly as for E1.
///
/// A failed export removes its partial file rather than leaving a truncated
/// image that looks finished. Every failure is an `Err` with a message the
/// caller can show; nothing here panics on bad input or I/O.
pub fn export_banded(ctx: &RenderCtx, ink: Option<RiverInk<'_>>, plan: &ExportBandPlan, format: StreamFormat, path: &Path) -> Result<u64, String> {
    if let Some(dir) = path.parent()
        && !dir.as_os_str().is_empty()
    {
        std::fs::create_dir_all(dir).map_err(|e| format!("could not create {}: {e}", dir.display()))?;
    }
    let file = File::create(path).map_err(|e| format!("could not create {}: {e}", path.display()))?;
    let bf = render::BakeFields::new(ctx);
    // Built once per export and shared by every band (E1's contract).
    let cells = render::build_grade_influence_cells(ctx);
    let bands = plan.bands().map(|band| render::bake_export_band(ctx, &bf, ink, plan, band, &cells));
    let mut out = BufWriter::with_capacity(PNG_CHUNK_BYTES, file);
    let result = write_bands(&mut out, format, plan.w, plan.h, bands)
        .and_then(|()| out.flush().map_err(|e| format!("could not write {}: {e}", path.display())))
        .and_then(|()| out.get_ref().metadata().map(|m| m.len()).map_err(|e| format!("could not stat {}: {e}", path.display())));
    if result.is_err() {
        drop(out);
        let _ = std::fs::remove_file(path);
    }
    result
}

/// Encode a `w × h` RGB8 image arriving as consecutive full-width row bands
/// (each `rows · w · 3` bytes, top to bottom) into `out`.
///
/// Each band is written and dropped before the next is pulled, so `bands`
/// can be a lazy renderer. The bands must cover exactly `w · h · 3` bytes in
/// whole rows; anything else is an `Err`, never a short or padded file.
pub fn write_bands<W: Write + Seek>(out: W, format: StreamFormat, w: usize, h: usize, bands: impl IntoIterator<Item = Vec<u8>>) -> Result<(), String> {
    let (Ok(w32), Ok(h32)) = (u32::try_from(w), u32::try_from(h)) else {
        return Err(format!("{w}x{h} is too large for an image header"));
    };
    if w == 0 || h == 0 {
        return Err(format!("degenerate export dimensions {w}x{h}"));
    }
    let total = w as u64 * h as u64 * 3;
    let mut seen = 0u64;
    // Refuse a band that is not whole rows or that runs past the image,
    // before it reaches an encoder.
    let mut checked = bands.into_iter().map(|band| {
        seen += band.len() as u64;
        if !band.len().is_multiple_of(w * 3) {
            Err(format!("a band of {} bytes is not a whole number of {w}-pixel rows", band.len()))
        } else if seen > total {
            Err(format!("the bands run past the {w}x{h} image"))
        } else {
            Ok(band)
        }
    });
    match format {
        StreamFormat::Png => write_png(out, w32, h32, &mut checked)?,
        StreamFormat::BigTiff => write_bigtiff(out, w, w32, h32, &mut checked)?,
    }
    drop(checked);
    if seen != total {
        return Err(format!("the bands cover {seen} of the image's {total} bytes"));
    }
    Ok(())
}

fn write_png<W: Write>(out: W, w: u32, h: u32, bands: &mut dyn Iterator<Item = Result<Vec<u8>, String>>) -> Result<(), String> {
    let png_err = |e: png::EncodingError| format!("PNG encode failed: {e}");
    let mut enc = png::Encoder::new(out, w, h);
    enc.set_color(png::ColorType::Rgb);
    enc.set_depth(png::BitDepth::Eight);
    enc.set_compression(png::Compression::Fast);
    enc.set_filter(png::Filter::Adaptive);
    let mut writer = enc.write_header().map_err(png_err)?;
    {
        let mut stream = writer.stream_writer_with_size(PNG_CHUNK_BYTES).map_err(png_err)?;
        for band in bands {
            stream.write_all(&band?).map_err(|e| format!("PNG encode failed: {e}"))?;
        }
        // Errors (rather than silently padding) when fewer than `w · h` pixels
        // arrived, which `write_bands` then reports with the byte count.
        stream.finish().map_err(png_err)?;
    }
    // Writes IEND. Explicit rather than left to `Drop`, which would swallow
    // an I/O error on the last chunk.
    writer.finish().map_err(png_err)
}

/// BigTIFF, Deflate, horizontal predictor, fixed-height strips.
///
/// # Why not `ImageEncoder::write_strip`
///
/// `EXPORT_SCOPE.md` §6.1 names `TiffEncoder::new_big()` →
/// `ImageEncoder::rows_per_strip()` / `write_strip()` as the streaming API.
/// **In `tiff` 0.11.3 that path writes a corrupt file under compression**: the
/// writer's compressor is installed only by the whole-image
/// `ImageEncoder::write_data` (`self.encoder.writer.set_compression(..)`,
/// through a private field), so `write_strip` called directly emits the strip
/// **uncompressed** while the IFD still says `Compression = Deflate`. So the
/// strips are predicted and compressed here, with the crate's own public
/// [`Deflate`](tiff::encoder::compression::Deflate) compressor, and written
/// through the crate's `DirectoryEncoder` — whose writer is left at its
/// default pass-through, so the bytes land exactly as compressed.
fn write_bigtiff<W: Write + Seek>(out: W, w: usize, w32: u32, h32: u32, bands: &mut dyn Iterator<Item = Result<Vec<u8>, String>>) -> Result<(), String> {
    use tiff::encoder::compression::{CompressionAlgorithm, Deflate, DeflateLevel};
    use tiff::encoder::{Rational, TiffEncoder};
    use tiff::tags::{CompressionMethod, PhotometricInterpretation, PlanarConfiguration, Predictor, ResolutionUnit, SampleFormat, Tag};

    let tiff_err = |e: tiff::TiffError| format!("TIFF encode failed: {e}");
    let row_bytes = w * 3;
    let rows_per_strip = (TIFF_STRIP_TARGET_BYTES / row_bytes).clamp(1, h32 as usize);
    let strip_bytes = rows_per_strip * row_bytes;

    let mut tif = TiffEncoder::new_big(out).map_err(tiff_err)?;
    let mut dir = tif.image_directory().map_err(tiff_err)?;
    let mut deflate = Deflate::with_level(DeflateLevel::Balanced);
    let (mut offsets, mut counts) = (Vec::<u64>::new(), Vec::<u64>::new());
    let (mut predicted, mut packed) = (Vec::with_capacity(strip_bytes), Vec::new());
    let mut emit = |dir: &mut tiff::encoder::DirectoryEncoder<'_, W, tiff::encoder::TiffKindBig>, strip: &[u8]| -> Result<(), String> {
        // TIFF's horizontal predictor for 8-bit chunky RGB: each sample minus
        // the same sample of the pixel to its left, modulo 256, row by row.
        predicted.clear();
        for row in strip.chunks_exact(row_bytes) {
            predicted.extend_from_slice(&row[..3]);
            predicted.extend(row[3..].iter().zip(row).map(|(&b, &a)| b.wrapping_sub(a)));
        }
        packed.clear();
        deflate.write_to(&mut packed, &predicted).map_err(|e| format!("TIFF deflate failed: {e}"))?;
        offsets.push(dir.write_data(packed.as_slice()).map_err(tiff_err)?);
        counts.push(packed.len() as u64);
        Ok(())
    };

    // Strips are cut at fixed rows, independent of where bands end: the
    // remainder of a band waits in `pending` for the next one.
    let mut pending: Vec<u8> = Vec::with_capacity(strip_bytes);
    for band in bands {
        let band = band?;
        let mut rest = band.as_slice();
        if !pending.is_empty() {
            let take = (strip_bytes - pending.len()).min(rest.len());
            pending.extend_from_slice(&rest[..take]);
            rest = &rest[take..];
            if pending.len() == strip_bytes {
                emit(&mut dir, &pending)?;
                pending.clear();
            }
        }
        while rest.len() >= strip_bytes {
            emit(&mut dir, &rest[..strip_bytes])?;
            rest = &rest[strip_bytes..];
        }
        pending.extend_from_slice(rest);
    }
    if !pending.is_empty() {
        // The short final strip. `write_bands` has already refused a band
        // that runs past the image and reports a short one after this returns.
        emit(&mut dir, &pending)?;
    }

    let rgb = [8u16, 8, 8];
    let uint = [SampleFormat::Uint.to_u16(); 3];
    dir.write_tag(Tag::ImageWidth, w32).map_err(tiff_err)?;
    dir.write_tag(Tag::ImageLength, h32).map_err(tiff_err)?;
    dir.write_tag(Tag::BitsPerSample, &rgb[..]).map_err(tiff_err)?;
    dir.write_tag(Tag::Compression, CompressionMethod::Deflate.to_u16()).map_err(tiff_err)?;
    dir.write_tag(Tag::PhotometricInterpretation, PhotometricInterpretation::RGB.to_u16()).map_err(tiff_err)?;
    dir.write_tag(Tag::StripOffsets, &offsets[..]).map_err(tiff_err)?;
    dir.write_tag(Tag::SamplesPerPixel, 3u16).map_err(tiff_err)?;
    dir.write_tag(Tag::RowsPerStrip, rows_per_strip as u32).map_err(tiff_err)?;
    dir.write_tag(Tag::StripByteCounts, &counts[..]).map_err(tiff_err)?;
    dir.write_tag(Tag::XResolution, Rational { n: 1, d: 1 }).map_err(tiff_err)?;
    dir.write_tag(Tag::YResolution, Rational { n: 1, d: 1 }).map_err(tiff_err)?;
    dir.write_tag(Tag::PlanarConfiguration, PlanarConfiguration::Chunky.to_u16()).map_err(tiff_err)?;
    dir.write_tag(Tag::ResolutionUnit, ResolutionUnit::None.to_u16()).map_err(tiff_err)?;
    dir.write_tag(Tag::Predictor, Predictor::Horizontal.to_u16()).map_err(tiff_err)?;
    dir.write_tag(Tag::SampleFormat, &uint[..]).map_err(tiff_err)?;
    dir.finish().map_err(tiff_err)
}
