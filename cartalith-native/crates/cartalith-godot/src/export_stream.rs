//! The streaming export writer (`EXPORT_SCOPE.md` §7, milestone E2): band in,
//! file out.
//!
//! E1 ([`render::ExportBandPlan`] / [`render::bake_export_band`]) produces a
//! 16K/32K raster one full-width band at a time; this module writes each band
//! to disk as it arrives and drops it, so the finished raster **never exists
//! in memory**. Peak memory is one band (plus its apron) through the finishing
//! passes, not the image.
//!
//! Three formats, each a survivor of `EXPORT_SCOPE.md` §6.1's streaming
//! constraint: PNG (ruling 26's format), BigTIFF + Deflate, and since E5
//! baseline JPEG (Ruling AZ, 2026-09-28: "offer JPEG alongside PNG"; written
//! by [`write_jpeg`] through the `jpeg-encoder` crate).
//!
//! Since ET-E1 (`LARGE_ITEM_RULINGS.md` Ruling CB item 1) every PNG written
//! here carries a colour label: one `sRGB` chunk by default, or one `iCCP`
//! with the embedded Display P3 profile ([`display_p3_icc`]) for an export in
//! Display P3. The BigTIFF stays untagged and refuses Display P3
//! ([`check_format_space`]). A JPEG is untagged JFIF in sRGB and carries the
//! same Display P3 profile as APP2 segments in Display P3 ([`write_jpeg`]).
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
//! No encoder sees band boundaries. `png::StreamWriter` filters and
//! compresses per completed **row** whatever the `write` calls look like, the
//! TIFF strips are a fixed number of rows ([`TIFF_STRIP_TARGET_BYTES`])
//! re-cut from the band stream, and the JPEG encoder asks for one row at a
//! time ([`JpegRows`]) and cuts its own 8-row MCU rows from them, so a band
//! need not be a multiple of 8 rows. So one export yields the same file bytes at
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

use crate::render::{self, BakeFields, ColorSpace, ExportBandPlan, ExportRivers, RenderCtx};

// ---------------------------------------------------------------------------
// ET-E1: the colour label every exported PNG carries
// ---------------------------------------------------------------------------

/// The ICC profile header's creation date-and-time field: bytes 24..36, six
/// big-endian `u16`s (year, month, day, hours, minutes, seconds), per the ICC
/// specification's profile header layout (ICC.1, `dateTimeNumber`; from the
/// layout `moxcms`' own `writer.rs` encodes, read 2026-10-11, not from the ICC
/// text itself). `moxcms` stamps it with the clock at every `encode()`
/// (`ColorDateTime::now()`), which would make two identical P3 exports
/// different files; it is pinned to [`P3_PROFILE_DATE`] instead.
const ICC_DATE_FIELD: std::ops::Range<usize> = 24..36;

/// The fixed creation date written into the embedded P3 profile:
/// 2026-10-11 00:00:00, the day Ruling CB item 1 adopted it. A **labelled
/// judgement**: any fixed valid date would do; the point is that it is fixed.
/// The header's 16-byte profile ID (MD5) is all zeros in `moxcms`' output
/// ("not computed", which the ICC header permits), so changing the date
/// invalidates no checksum.
const P3_PROFILE_DATE: [u16; 6] = [2026, 10, 11, 0, 0, 0];

/// The D65 white `render::SRGB_TO_P3` is derived on (its doc: `.3127/.3290`),
/// the white the sRGB and Display P3 standards use, as `moxcms`' `XyY` with
/// luminance 1. A published constant, not a judgement.
const D65: moxcms::XyY = moxcms::XyY { x: 0.3127, y: 0.3290, yb: 1.0 };

/// **The Display P3 ICC profile a P3 export embeds** (ET-E1), built once.
///
/// Built from `moxcms::ColorProfile::new_display_p3()` (its sRGB-shaped
/// parametric transfer curve on all three channels, its description and
/// copyright tags), with **three corrections**, each found by parsing the
/// profile back in `tests/export_stream.rs` and each needed so the profile
/// describes exactly the space `render::SRGB_TO_P3` re-encodes into (P3
/// primaries `.68/.32 .265/.69 .15/.06` on D65 `.3127/.3290`, sRGB's transfer
/// function -- that constant's own doc):
///
/// 1. **The white point.** `moxcms` builds its colorants on `WHITE_POINT_D65 =
///    white_point_from_temperature(6504)`, the CIE daylight-locus formula at
///    6504 K: x .312714, y .329118 -- measured 1.2e-4 off the D65 `.3127/.3290`
///    that sRGB, Display P3 and `SRGB_TO_P3` all use (the test's white check
///    failed at y .32912 before this). The colorants are rebuilt with
///    `update_rgb_colorimetry` on the standard D65 and `ColorPrimaries::
///    DISPLAY_P3`, and the media white point is set to the same D65.
/// 2. **The `chad` tag.** `moxcms`' RGB profiles carry `chromatic_adaptation:
///    Some(BRADFORD_D)` -- the bare Bradford cone-response matrix (`chad.rs`:
///    `[[0.8951, 0.2664, -0.1614], ...]`), not an adaptation. The ICC
///    `chromaticAdaptationTag` is the matrix taking the actual illuminant's XYZ
///    to the PCS (D50) XYZ (ICC.1, from memory of the specification, not
///    re-read for this change); a reader undoing the adaptation with the bare
///    cone matrix recovers red at x .573 y .422 instead of .68/.32 (measured
///    by the test before this correction). The tag is set to
///    `adaption_matrix_d(D65, D50)` -- the Bradford adaptation `moxcms` itself
///    adapts the colorants with (`adapt_to_d50_d`), so tag and colorants agree.
/// 3. **The `cicp` tag.** `new_display_p3` says colour primaries `Smpte431`
///    (ITU-T H.273 value 11, SMPTE RP 431-2: the DCI theatrical white, x .314
///    y .351 by `moxcms`' own table). Display P3 is value 12 (`Smpte432`,
///    SMPTE EG 432-1, D65). `update_rgb_colorimetry` clears the tag, so it is
///    restored with primaries 12 and `moxcms`' own transfer (`Srgb`), matrix
///    (`Bt709`) and range (`false`) values.
///
/// The header's clock-stamped creation date is then pinned
/// ([`P3_PROFILE_DATE`]) so the same export is the same file.
///
/// `Err` only if `moxcms` refuses to encode, which a fixed profile does not;
/// it is still an `Err`, never a panic, because this runs under a `#[func]`.
pub fn display_p3_icc() -> Result<&'static [u8], String> {
    static PROFILE: std::sync::OnceLock<Result<Vec<u8>, String>> = std::sync::OnceLock::new();
    PROFILE
        .get_or_init(|| {
            let mut p = moxcms::ColorProfile::new_display_p3();
            let cicp = p.cicp.map(|c| moxcms::CicpProfile { color_primaries: moxcms::CicpColorPrimaries::Smpte432, ..c });
            p.update_rgb_colorimetry(D65, moxcms::ColorPrimaries::DISPLAY_P3);
            p.media_white_point = Some(D65.to_xyzd());
            p.chromatic_adaptation = Some(moxcms::adaption_matrix_d(D65.to_xyz(), moxcms::Chromaticity::D50.to_xyz()));
            p.cicp = cicp;
            let mut bytes = p.encode().map_err(|e| format!("could not build the Display P3 profile: {e:?}"))?;
            if bytes.len() < ICC_DATE_FIELD.end {
                return Err("the Display P3 profile is shorter than an ICC header".into());
            }
            let date: Vec<u8> = P3_PROFILE_DATE.iter().flat_map(|v| v.to_be_bytes()).collect();
            bytes[ICC_DATE_FIELD].copy_from_slice(&date);
            Ok(bytes)
        })
        .as_ref()
        .map(|v| v.as_slice())
        .map_err(|e| e.clone())
}

/// The `cartalith_assets` label for an export in `space`: `sRGB` for
/// [`ColorSpace::Srgb`], the embedded [`display_p3_icc`] profile for
/// [`ColorSpace::DisplayP3`] -- the one mapping every PNG writer here uses, so
/// the label can never disagree with the pixels' encoding.
pub fn png_colour(space: ColorSpace) -> Result<cartalith_assets::raster::PngColour<'static>, String> {
    Ok(match space {
        ColorSpace::Srgb => cartalith_assets::raster::PngColour::Srgb,
        ColorSpace::DisplayP3 => cartalith_assets::raster::PngColour::IccProfile(display_p3_icc()?),
    })
}

/// Refuse a format that cannot carry `space`'s label. BigTIFF is written
/// without a colour tag (ET-E1 covers PNG; TIFF's ICC tag 34675 is not
/// written), so Display P3 numbers in one would be read as sRGB by every
/// consumer: refused rather than written mislabelled. sRGB BigTIFF stays as it
/// shipped (an untagged TIFF is conventionally read as sRGB). PNG and JPEG
/// carry both spaces ([`write_png`], [`write_jpeg`]).
///
/// There is no CMYK case to refuse here: `render::ColorSpace` has no CMYK
/// member, so a `"CMYK"` colour space is refused by the options parser as an
/// unknown name before it reaches a writer. A CMYK JPEG (Adobe APP14, four
/// components) is out of scope for E5; Ruling CB item 6's CMYK work is a
/// print-gamut *warning*, not a CMYK export.
pub fn check_format_space(format: StreamFormat, space: ColorSpace) -> Result<(), String> {
    if format == StreamFormat::BigTiff && space == ColorSpace::DisplayP3 {
        return Err("a Display P3 export must be PNG or JPEG -- this BigTIFF writer embeds no colour profile, so P3 numbers in it would be read as sRGB".into());
    }
    Ok(())
}

/// The JPEG quality an export gets when the options name none: **90**, a
/// **labelled judgement**. It is the lowest quality at which `jpeg-encoder`
/// 0.7.1's own `Encoder::new` keeps full chroma (it switches to 4:2:0 below
/// 90, a choice [`write_jpeg`] overrides anyway), and on the IJG scale the
/// crate's quantisation tables use (`quantization.rs::get_with_quality`: the
/// Annex K tables scaled by `200 - 2q` percent) it divides every Annex K step
/// by five. What it costs in size and error on a real export is measured in
/// `EXPORT_SCOPE.md` E5; the number itself was not derived from those
/// measurements.
pub const JPEG_QUALITY_DEFAULT: u8 = 90;

/// The qualities a JPEG export accepts: `jpeg-encoder`'s own 1..=100 (its
/// `Encoder::new` doc). Anything outside is refused by the options parser, not
/// clamped, so a mistyped quality cannot become a plausible one.
pub const JPEG_QUALITY_RANGE: std::ops::RangeInclusive<u8> = 1..=100;

/// The file formats [`write_bands`] can stream.
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
    /// Baseline JPEG (one interleaved sequential scan, standard Huffman
    /// tables, 4:4:4 chroma) at `quality` ([`JPEG_QUALITY_RANGE`]), written
    /// by [`write_jpeg`]. Lossy: the one format here whose pixels do not
    /// decode back to the bands.
    Jpeg { quality: u8 },
}

impl StreamFormat {
    /// The JPEG format at [`JPEG_QUALITY_DEFAULT`], what `"format": "jpeg"`
    /// with no `quality` means.
    pub const JPEG: StreamFormat = StreamFormat::Jpeg { quality: JPEG_QUALITY_DEFAULT };

    /// True for every JPEG quality: the format's kind, ignoring its setting.
    pub fn is_jpeg(self) -> bool {
        matches!(self, StreamFormat::Jpeg { .. })
    }
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
/// `bf` and `rivers` are what `WorldGen::export_render_with` hands every
/// export (RV-5): the bake prologue with the screen's smooth-shore field
/// attached, and the vector rivers (or a loaded save's flag);
/// `ctx.appearance` and `ctx.world` are the look and wrap, exactly as for E1.
/// `bf` is built by the caller because the shore field needs the world's
/// water bodies, which a `RenderCtx` does not carry.
///
/// `space` is the export's colour space (ET-E1): it reaches both the pixels
/// (`render::bake_export_band`'s one finishing pass) and the file's label
/// ([`write_bands`]), from this one argument, so they cannot disagree. A
/// format that cannot carry the label is refused before the file is created
/// ([`check_format_space`]).
///
/// A failed export removes its partial file rather than leaving a truncated
/// image that looks finished. Every failure is an `Err` with a message the
/// caller can show; nothing here panics on bad input or I/O.
pub fn export_banded(ctx: &RenderCtx, bf: &BakeFields, rivers: ExportRivers<'_>, plan: &ExportBandPlan, format: StreamFormat, space: ColorSpace, path: &Path) -> Result<u64, String> {
    check_format_space(format, space)?;
    if let Some(dir) = path.parent()
        && !dir.as_os_str().is_empty()
    {
        std::fs::create_dir_all(dir).map_err(|e| format!("could not create {}: {e}", dir.display()))?;
    }
    let file = File::create(path).map_err(|e| format!("could not create {}: {e}", path.display()))?;
    // Built once per export and shared by every band (E1's contract).
    let cells = render::build_grade_influence_cells(ctx);
    let bands = plan.bands().map(|band| render::bake_export_band(ctx, bf, rivers, plan, band, &cells, space));
    let mut out = BufWriter::with_capacity(PNG_CHUNK_BYTES, file);
    let result = write_bands(&mut out, format, space, plan.w, plan.h, bands)
        .and_then(|()| out.flush().map_err(|e| format!("could not write {}: {e}", path.display())))
        .and_then(|()| out.get_ref().metadata().map(|m| m.len()).map_err(|e| format!("could not stat {}: {e}", path.display())));
    if result.is_err() {
        drop(out);
        let _ = std::fs::remove_file(path);
    }
    result
}

/// [`write_bands`] on its own thread, fed one band at a time — the E4 overlay
/// session's writer (`EXPORT_SCOPE.md` §7), where bands arrive from calls
/// across the gdext boundary rather than from a lazy iterator.
///
/// The encoder is the unchanged [`write_bands`]: a `Receiver<Vec<u8>>` is an
/// `IntoIterator<Item = Vec<u8>>` that ends when the sender is dropped, so the
/// file it writes is byte-identical to [`export_banded`]'s for the same bands.
/// The channel is `sync_channel(1)`: at most one band waits while one is being
/// encoded, so a fast producer blocks instead of queueing the image in memory.
///
/// **A file is either finished or removed.** [`Self::finish`] returns the
/// bytes written; a writer error, [`Self::abort`] and a drop without `finish`
/// all join the writer and delete the partial file — the same rule
/// [`export_banded`] applies inline, duplicated here because this writer
/// outlives any one call.
pub struct BandSink {
    tx: Option<std::sync::mpsc::SyncSender<Vec<u8>>>,
    writer: Option<std::thread::JoinHandle<Result<u64, String>>>,
    path: std::path::PathBuf,
}

impl BandSink {
    /// Create `path` (and its parent directory) and start the writer for a
    /// `w × h` image in `format`, labelled for colour space `space` (the bands
    /// pushed must already be encoded in it). A format that cannot carry the
    /// label is refused before any file is created.
    pub fn open(path: &Path, format: StreamFormat, space: ColorSpace, w: usize, h: usize) -> Result<BandSink, String> {
        check_format_space(format, space)?;
        if let Some(dir) = path.parent()
            && !dir.as_os_str().is_empty()
        {
            std::fs::create_dir_all(dir).map_err(|e| format!("could not create {}: {e}", dir.display()))?;
        }
        let file = File::create(path).map_err(|e| format!("could not create {}: {e}", path.display()))?;
        let (tx, rx) = std::sync::mpsc::sync_channel::<Vec<u8>>(1);
        let shown = path.display().to_string();
        let spawned = std::thread::Builder::new().name("cartalith-export-writer".into()).spawn(move || {
            let mut out = BufWriter::with_capacity(PNG_CHUNK_BYTES, file);
            write_bands(&mut out, format, space, w, h, rx)
                .and_then(|()| out.flush().map_err(|e| format!("could not write {shown}: {e}")))
                .and_then(|()| out.get_ref().metadata().map(|m| m.len()).map_err(|e| format!("could not stat {shown}: {e}")))
        });
        match spawned {
            Ok(writer) => Ok(BandSink { tx: Some(tx), writer: Some(writer), path: path.to_path_buf() }),
            Err(e) => {
                let _ = std::fs::remove_file(path);
                Err(format!("could not start the export writer: {e}"))
            }
        }
    }

    /// Hand the writer the next band (whole rows, top to bottom). Blocks while
    /// the previous band is still waiting. If the writer has already failed,
    /// its error is returned and the file removed; the sink is then spent.
    pub fn push(&mut self, band: Vec<u8>) -> Result<(), String> {
        let Some(tx) = self.tx.as_ref() else {
            return Err("the export writer is already closed".into());
        };
        if tx.send(band).is_ok() {
            return Ok(());
        }
        // The receiver is gone, so the writer returned: collect why.
        match self.close() {
            Err(e) => Err(e),
            Ok(_) => Err("the export writer finished before every band arrived".into()),
        }
    }

    /// Close the band stream and wait for the file. `Err` (file removed) when
    /// the bands did not cover the image exactly, or on any I/O error.
    pub fn finish(mut self) -> Result<u64, String> {
        self.close()
    }

    /// Stop now and remove the partial file.
    pub fn abort(mut self) {
        self.discard();
    }

    /// Drop the sender, join the writer, and remove the file unless it
    /// finished cleanly.
    fn close(&mut self) -> Result<u64, String> {
        self.tx = None;
        let result = match self.writer.take() {
            None => Err("the export writer is already closed".into()),
            Some(h) => h.join().unwrap_or_else(|_| Err("the export writer panicked".into())),
        };
        if result.is_err() {
            let _ = std::fs::remove_file(&self.path);
        }
        result
    }

    /// Close and remove the file whatever the writer reports. The join comes
    /// first: Windows will not delete a file another thread holds open.
    fn discard(&mut self) {
        if self.tx.is_none() && self.writer.is_none() {
            return;
        }
        self.tx = None;
        if let Some(h) = self.writer.take() {
            let _ = h.join();
        }
        let _ = std::fs::remove_file(&self.path);
    }
}

impl Drop for BandSink {
    /// A sink dropped without [`BandSink::finish`] is an abandoned export.
    fn drop(&mut self) {
        self.discard();
    }
}

/// Encode a `w × h` RGB8 image arriving as consecutive full-width row bands
/// (each `rows · w · 3` bytes, top to bottom) into `out`.
///
/// Each band is written and dropped before the next is pulled, so `bands`
/// can be a lazy renderer. The bands must cover exactly `w · h · 3` bytes in
/// whole rows; anything else is an `Err`, never a short or padded file.
///
/// `space` is the colour space the bands are encoded in, and decides the PNG's
/// label (ET-E1, [`write_png`]); a BigTIFF in Display P3 is refused
/// ([`check_format_space`]).
pub fn write_bands<W: Write + Seek>(out: W, format: StreamFormat, space: ColorSpace, w: usize, h: usize, bands: impl IntoIterator<Item = Vec<u8>>) -> Result<(), String> {
    check_format_space(format, space)?;
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
        StreamFormat::Png => write_png(out, w32, h32, space, &mut checked)?,
        StreamFormat::BigTiff => write_bigtiff(out, w, w32, h32, &mut checked)?,
        StreamFormat::Jpeg { quality } => write_jpeg(out, w, h, quality, space, &mut checked)?,
    }
    drop(checked);
    if seen != total {
        return Err(format!("the bands cover {seen} of the image's {total} bytes"));
    }
    Ok(())
}

/// Streams RGB8 bands into a PNG encoder (`png` crate, `Compression::Fast`,
/// adaptive filter, `PNG_CHUNK_BYTES` writer chunks) so a 16K/32K export never holds
/// the whole image. Fast compression is a size-for-speed judgement; the encoder
/// errors, rather than padding, when fewer than `w x h` pixels arrive. Must never
/// buffer all bands.
///
/// **The colour label (ET-E1, Ruling CB item 1)**, written by the `png` crate
/// itself before `IDAT`: [`ColorSpace::Srgb`] is one `sRGB` chunk, rendering
/// intent perceptual (a labelled judgement, `cartalith_assets::raster::
/// PngColour::Srgb`'s doc), and no `iCCP`, `gAMA` or `cHRM` (`png` writes those
/// fallbacks only when asked); [`ColorSpace::DisplayP3`] is one `iCCP` holding
/// [`display_p3_icc`], deflated by `png` (compression method 0), and no `sRGB`.
/// Never both: the PNG specification (third edition) says a file with one
/// should not carry the other. `png::Encoder::with_info` over
/// `Info::with_size` is what `Encoder::new` builds, so the image data is
/// encoded exactly as before; only the chunk list grows.
fn write_png<W: Write>(out: W, w: u32, h: u32, space: ColorSpace, bands: &mut dyn Iterator<Item = Result<Vec<u8>, String>>) -> Result<(), String> {
    let png_err = |e: png::EncodingError| format!("PNG encode failed: {e}");
    let mut info = png::Info::with_size(w, h);
    match space {
        ColorSpace::Srgb => info.srgb = Some(png::SrgbRenderingIntent::Perceptual),
        ColorSpace::DisplayP3 => info.icc_profile = Some(std::borrow::Cow::Borrowed(display_p3_icc()?)),
    }
    let mut enc = png::Encoder::with_info(out, info).map_err(png_err)?;
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

/// Baseline JPEG from a band stream, through `jpeg-encoder` 0.7.1, holding
/// one band and one 8-row MCU row at a time -- never the image.
///
/// # How it streams
///
/// `jpeg_encoder::Encoder::encode_image` takes an `ImageBuffer` and, on the
/// interleaved path this function forces (no progressive scans, no optimised
/// Huffman tables, a sampling factor that `supports_interleaved`), asks it for
/// rows **in increasing order**, eight per MCU row, repeating the last row to
/// pad the final MCU row (`encoder.rs::encode_image_interleaved`, read
/// 2026-10-18). [`JpegRows`] answers each request from the current band and
/// pulls the next band only when the request passes its end, so the bands are
/// consumed as the PNG writer consumes them. The crate's other two paths
/// (`encode_image_sequential`, `encode_image_progressive`) collect every block
/// of the image first; they are what `set_progressive(true)` or
/// `set_optimized_huffman_tables(true)` would select, and **must never be
/// turned on here** -- at 32K that is gigabytes of coefficients.
///
/// # The settings, and why
///
/// - **4:4:4 chroma** (`SamplingFactor::R_4_4_4`), at every quality. The
///   crate's own default is 4:2:0 below quality 90; a map's routes, borders,
///   label text and pins are 1-2 px coloured lines, and 4:2:0 stores one
///   chroma sample per 2 x 2 pixels, so a 1 px red route over green land is
///   averaged with its neighbours' hue. `tests/export_stream.rs`
///   (`jpeg_444_keeps_a_one_pixel_coloured_line`) measures the difference on
///   such a line rather than leaving this as an argument.
/// - **Quality** as given (`StreamFormat::Jpeg`), default
///   [`JPEG_QUALITY_DEFAULT`].
/// - **Colour label:** [`ColorSpace::Srgb`] writes plain JFIF with no ICC
///   profile, which colour-managed readers take as sRGB -- the same convention
///   [`check_format_space`] relies on for an untagged BigTIFF. JPEG has no
///   equivalent of PNG's `sRGB` chunk, so this is a **labelled judgement**,
///   not a specification rule; embedding an sRGB profile instead would need
///   its own profile check like [`display_p3_icc`]'s. [`ColorSpace::DisplayP3`]
///   embeds [`display_p3_icc`] -- the profile a P3 PNG carries, byte for byte
///   -- as `ICC_PROFILE` APP2 segments (`Encoder::add_icc_profile`, which its
///   comment says follows the ICC's "Embedding ICC profiles in JFIF files"
///   layout; not re-read against the ICC text here).
/// - **Deterministic bytes:** the crate's scalar DCT, never the `simd`
///   feature's AVX2 one (`Cargo.toml`), no restart markers, no clock.
///
/// # Failures
///
/// A JPEG frame header holds 16-bit dimensions, so a side over 65 535 px is
/// refused before anything is written (no `BAKE_WIDTHS` rung reaches it). A
/// band error, or bands that end early, cannot be returned from
/// `fill_buffers`; [`JpegRows`] records it and answers with filler, and the
/// [`JpegOut`] writer then fails the next write, so the encoder stops within
/// one buffered write instead of encoding the rest of a 32K image. The
/// recorded message is what this returns.
fn write_jpeg<W: Write>(out: W, w: usize, h: usize, quality: u8, space: ColorSpace, bands: &mut dyn Iterator<Item = Result<Vec<u8>, String>>) -> Result<(), String> {
    let (Ok(w16), Ok(h16)) = (u16::try_from(w), u16::try_from(h)) else {
        return Err(format!("{w}x{h} is too large for a JPEG -- its frame header holds at most 65535 px a side"));
    };
    if !JPEG_QUALITY_RANGE.contains(&quality) {
        return Err(format!("JPEG quality {quality} is outside {JPEG_QUALITY_RANGE:?}"));
    }
    let failed: Failure = Default::default();
    let jpeg_err = |e: jpeg_encoder::EncodingError| format!("JPEG encode failed: {e}");
    let mut enc = jpeg_encoder::Encoder::new(JpegOut { inner: out, failed: failed.clone() }, quality);
    enc.set_sampling_factor(jpeg_encoder::SamplingFactor::R_4_4_4);
    enc.set_progressive(false);
    enc.set_optimized_huffman_tables(false);
    if space == ColorSpace::DisplayP3 {
        enc.add_icc_profile(display_p3_icc()?).map_err(jpeg_err)?;
    }
    let rows = JpegRows { w, w16, h16, feed: std::cell::RefCell::new(JpegFeed { bands: &mut *bands, band: Vec::new(), first_row: 0 }), failed: failed.clone() };
    let encoded = enc.encode_image(rows);
    // The band stream's own error outranks the I/O error `JpegOut` raised
    // because of it.
    if let Some(e) = failed.borrow_mut().take() {
        return Err(e);
    }
    encoded.map_err(jpeg_err)?;
    // Pull whatever the encoder did not need, so a band past the image's last
    // row reaches `write_bands`' "run past the image" check as it does for PNG.
    for band in bands {
        band?;
    }
    Ok(())
}

/// The first error of a JPEG encode, shared by [`JpegRows`] (which records
/// it) and [`JpegOut`] (which then refuses every write). `Rc`, not `Arc`: the
/// whole encode runs on one thread.
type Failure = std::rc::Rc<std::cell::RefCell<Option<String>>>;

/// The row source [`write_jpeg`] hands the encoder: answers
/// `fill_buffers(y)` from the band holding row `y`, pulling bands from the
/// stream only as the rows pass their end and dropping each band once its
/// last row is read, so it holds at most one band and none while it waits. `RefCell` because `ImageBuffer::
/// fill_buffers` takes `&self`.
struct JpegRows<'a> {
    w: usize,
    w16: u16,
    h16: u16,
    feed: std::cell::RefCell<JpegFeed<'a>>,
    failed: Failure,
}

/// [`JpegRows`]' mutable half: the band stream, the band now held, and the
/// image row its first row is.
struct JpegFeed<'a> {
    bands: &'a mut dyn Iterator<Item = Result<Vec<u8>, String>>,
    band: Vec<u8>,
    first_row: usize,
}

impl jpeg_encoder::ImageBuffer for JpegRows<'_> {
    /// RGB is encoded as YCbCr, as a JFIF file's three components are.
    fn get_jpeg_color_type(&self) -> jpeg_encoder::JpegColorType {
        jpeg_encoder::JpegColorType::Ycbcr
    }

    /// The image width, already checked to fit the frame header.
    fn width(&self) -> u16 {
        self.w16
    }

    /// The image height, already checked to fit the frame header.
    fn height(&self) -> u16 {
        self.h16
    }

    /// Row `y` as Y, Cb and Cr samples (`jpeg_encoder::rgb_to_ycbcr`, the
    /// crate's own fixed-point full-range BT.601 conversion), one per pixel
    /// per plane. After a failure, filler (Y 0, Cb and Cr 128) so the buffers
    /// keep the length the encoder indexes by; that output is never kept
    /// ([`write_jpeg`]'s doc). Never panics on a short or failed stream.
    fn fill_buffers(&self, y: u16, buffers: &mut [Vec<u8>; 4]) {
        let mut feed = self.feed.borrow_mut();
        let row_bytes = self.w * 3;
        let y = y as usize;
        let mut ok = self.failed.borrow().is_none();
        while ok {
            let held = feed.band.len() / row_bytes;
            if y < feed.first_row {
                *self.failed.borrow_mut() = Some(format!("the JPEG encoder asked for row {y} after row {}", feed.first_row));
                ok = false;
            } else if y < feed.first_row + held {
                break;
            } else {
                match feed.bands.next() {
                    Some(Ok(band)) => {
                        feed.first_row += held;
                        feed.band = band;
                    }
                    Some(Err(e)) => {
                        *self.failed.borrow_mut() = Some(e);
                        ok = false;
                    }
                    None => {
                        *self.failed.borrow_mut() = Some(format!("the bands cover {} of the image's {} rows", feed.first_row + held, self.h16));
                        ok = false;
                    }
                }
            }
        }
        if !ok {
            buffers[0].extend(std::iter::repeat_n(0, self.w));
            buffers[1].extend(std::iter::repeat_n(128, self.w));
            buffers[2].extend(std::iter::repeat_n(128, self.w));
            return;
        }
        let at = (y - feed.first_row) * row_bytes;
        for px in feed.band[at..at + row_bytes].chunks_exact(3) {
            let (luma, cb, cr) = jpeg_encoder::rgb_to_ycbcr(px[0], px[1], px[2]);
            buffers[0].push(luma);
            buffers[1].push(cb);
            buffers[2].push(cr);
        }
        // Drop a band as soon as its last row is read (unless it holds the
        // image's last row, which the encoder repeats to pad the final MCU
        // row): otherwise it stays alive while the next band is waited for,
        // one band more than the PNG writer holds. Measured 2026-10-18 at
        // 16K through the overlay session: without this, the export's host
        // working-set rise was ~100 MB (about one 16384 x 2294 RGB band)
        // above PNG's (`EXPORT_SCOPE.md` E5).
        let held = feed.band.len() / row_bytes;
        if y + 1 == feed.first_row + held && y + 1 < self.h16 as usize {
            feed.first_row += held;
            feed.band = Vec::new();
        }
    }
}

/// The encoder's output, which fails every write once [`JpegRows`] has
/// recorded an error -- the only way to stop `encode_image` early, since
/// `fill_buffers` cannot return one.
struct JpegOut<W> {
    inner: W,
    failed: Failure,
}

impl<W: Write> Write for JpegOut<W> {
    /// Forwards to the file unless the band stream has failed.
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if self.failed.borrow().is_some() {
            return Err(std::io::Error::other("the export's band stream failed"));
        }
        self.inner.write(buf)
    }

    /// Forwards to the file.
    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
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

/// ET-E1's test helpers: a PNG chunk walker (lengths, types, CRCs, `IEND`
/// last) and the Display P3 profile check, shared by `export_raster.rs`'s and
/// `export_session.rs`'s unit tests and `tests/export_stream.rs`. Written
/// against the PNG and ICC layouts, not against the writers above.
#[cfg(test)]
pub(crate) mod png_chunks {
    /// CRC-32 (ISO 3309, as PNG specifies it), bit by bit: reflected
    /// polynomial `0xEDB88320`, initial value and final XOR all ones.
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

    /// Every chunk in `png` as `(type, data)`, in file order. Panics on a bad
    /// signature, a CRC mismatch, or bytes after `IEND`; asserts non-empty.
    pub(crate) fn chunks(png: &[u8]) -> Vec<([u8; 4], Vec<u8>)> {
        assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n", "not a PNG signature");
        let (mut p, mut out) = (8usize, Vec::new());
        while p < png.len() {
            let len = u32::from_be_bytes(png[p..p + 4].try_into().unwrap()) as usize;
            let body = &png[p + 4..p + 8 + len];
            assert_eq!(crc32(body), u32::from_be_bytes(png[p + 8 + len..p + 12 + len].try_into().unwrap()), "CRC mismatch at {p}");
            let kind: [u8; 4] = body[..4].try_into().unwrap();
            out.push((kind, body[4..].to_vec()));
            p += 12 + len;
            if &kind == b"IEND" {
                break;
            }
        }
        assert_eq!(p, png.len(), "bytes after IEND, or no IEND");
        assert!(!out.is_empty(), "a PNG with no chunks proves nothing");
        out
    }

    /// How many chunks of type `kind` the file holds.
    pub(crate) fn count(png: &[u8], kind: &[u8; 4]) -> usize {
        chunks(png).iter().filter(|(k, _)| k == kind).count()
    }

    /// The `sRGB`/`iCCP` census the ET-E1 acceptance names, as `(sRGB, iCCP)`.
    pub(crate) fn labels(png: &[u8]) -> (usize, usize) {
        (count(png, b"sRGB"), count(png, b"iCCP"))
    }

    /// The one `iCCP` chunk's profile, inflated: profile name, NUL,
    /// compression method (must be 0, zlib deflate -- the only one PNG
    /// defines), then the zlib stream. `None` when there is no `iCCP`.
    pub(crate) fn iccp_profile(png: &[u8]) -> Option<Vec<u8>> {
        use std::io::Read;
        let (_, data) = chunks(png).into_iter().find(|(k, _)| k == b"iCCP")?;
        let nul = data.iter().position(|&b| b == 0).expect("iCCP has no NUL after its name");
        assert!((1..=79).contains(&nul), "iCCP profile name must be 1..79 bytes");
        assert_eq!(data[nul + 1], 0, "iCCP compression method must be 0");
        let mut out = Vec::new();
        flate2::read::ZlibDecoder::new(&data[nul + 2..]).read_to_end(&mut out).expect("iCCP inflates");
        Some(out)
    }

    /// Asserts `icc` parses (with `moxcms`, the reader) as a Display P3
    /// profile: the P3 primaries `.68/.32 .265/.69 .15/.06` and the D65 white
    /// `.3127/.3290` within 1e-4 (the scope's bar), recovered from the
    /// D50-adapted colorants through the profile's own `chad` matrix; sRGB's
    /// IEC 61966-2-1 parametric curve on all three channels (g 2.4, a 1/1.055,
    /// b 0.055/1.055, c 1/12.92, d 0.04045, within 1e-4: `s15Fixed16` stores
    /// to 1.5e-5); and the `cicp` primaries tag = 12 (ITU-T H.273, SMPTE EG
    /// 432-1, Display P3 on D65). Every expected value is a published literal.
    pub(crate) fn assert_display_p3_profile(icc: &[u8]) {
        let p = moxcms::ColorProfile::new_from_slice(icc).expect("moxcms parses the profile");
        let chad = p.chromatic_adaptation.expect("a chad tag").v;
        let inv = inverse(chad);
        let xy = |c: [f64; 3]| {
            let v = mul(inv, c);
            let s = v[0] + v[1] + v[2];
            (v[0] / s, v[1] / s)
        };
        let near = |got: (f64, f64), want: (f64, f64), what: &str| {
            assert!((got.0 - want.0).abs() < 1e-4 && (got.1 - want.1).abs() < 1e-4, "{what}: {got:?} vs {want:?}");
        };
        let col = |c: moxcms::Xyzd| [c.x, c.y, c.z];
        near(xy(col(p.red_colorant)), (0.680, 0.320), "red");
        near(xy(col(p.green_colorant)), (0.265, 0.690), "green");
        near(xy(col(p.blue_colorant)), (0.150, 0.060), "blue");
        // The white the colorants sum to is the D50 PCS white; un-adapted it
        // must be D65.
        let sum = [0, 1, 2].map(|i| col(p.red_colorant)[i] + col(p.green_colorant)[i] + col(p.blue_colorant)[i]);
        near(xy(sum), (0.3127, 0.3290), "white");
        let want = [2.4, 1.0 / 1.055, 0.055 / 1.055, 1.0 / 12.92, 0.04045];
        for (name, trc) in [("red", &p.red_trc), ("green", &p.green_trc), ("blue", &p.blue_trc)] {
            match trc {
                Some(moxcms::ToneReprCurve::Parametric(v)) => {
                    assert_eq!(v.len(), 5, "{name}: parametric type 3 has five parameters");
                    for (g, w) in v.iter().zip(want) {
                        assert!((*g as f64 - w).abs() < 1e-4, "{name} TRC {v:?} is not the sRGB curve");
                    }
                }
                other => panic!("{name} TRC is not parametric: {other:?}"),
            }
        }
        assert_eq!(p.cicp.expect("a cicp tag").color_primaries as u8, 12, "cicp primaries must be H.273 12 (Display P3, D65)");
    }

    /// `m · v` for a row-major 3x3.
    fn mul(m: [[f64; 3]; 3], v: [f64; 3]) -> [f64; 3] {
        [0, 1, 2].map(|r| m[r][0] * v[0] + m[r][1] * v[1] + m[r][2] * v[2])
    }

    /// Inverse of a 3x3 by the adjugate; the `chad` matrix is far from
    /// singular (it is close to identity).
    fn inverse(m: [[f64; 3]; 3]) -> [[f64; 3]; 3] {
        let c = |r0: usize, c0: usize, r1: usize, c1: usize| m[r0][c0] * m[r1][c1] - m[r0][c1] * m[r1][c0];
        let det = m[0][0] * c(1, 1, 2, 2) - m[0][1] * c(1, 0, 2, 2) + m[0][2] * c(1, 0, 2, 1);
        assert!(det.abs() > 1e-9, "singular chad");
        [
            [c(1, 1, 2, 2) / det, -c(0, 1, 2, 2) / det, c(0, 1, 1, 2) / det],
            [-c(1, 0, 2, 2) / det, c(0, 0, 2, 2) / det, -c(0, 0, 1, 2) / det],
            [c(1, 0, 2, 1) / det, -c(0, 0, 2, 1) / det, c(0, 0, 1, 1) / det],
        ]
    }
}
