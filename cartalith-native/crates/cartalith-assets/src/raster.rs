//! Milestone 6 (`ASSET_LIBRARY_SCOPE.md`): the first place this crate touches
//! pixels. Crate work (`image`) plus a thin port, matching how [`crate::archive`]
//! already reused the `zip` crate rather than reimplementing archive handling —
//! `PROVENANCE.md`'s "take a crate for anything downstream of the pixels" rule.
//!
//! Ported from `Cartalith Gen1 v2.10.html`:
//! - `finalizePackTexture` (line 12196) — pure arithmetic, no DOM, golden-verified.
//! - `fitToBottom` (line 26769) — pure arithmetic, no DOM, golden-verified.
//! - `drawItemOnly`/`renderItem`/`renderToCanvas` (`ThumbnailRenderer`, lines
//!   26750-26777) — the single shared render core the reference itself uses for
//!   thumbnails, the inspector preview, *and* pack-export bake (its own doc
//!   comment says so). Ported here as [`render_item`].
//! - `itemHash` (line 26913) — real content hash from decoded pixels, feeding
//!   [`crate::library::duplicate_groups`]/[`crate::library::slot_has_dupe`],
//!   which milestone 5 already implemented against a caller-supplied hash.
//! - `encodeItemPng` (line 27873), `AssetImporter.decodeBytes` (27057),
//!   `decodePackImage` (12229) — decode/encode, via the `image` crate.
//!
//! # Why `itemHash` does not need to (and cannot) match the reference byte for
//! byte
//!
//! Two things settle this, both found by reading rather than assumed:
//!
//! 1. **The hash is never serialized.** `_alExportEntries` writes
//!    `{img,name,t}` per item (line 27890) — no `hash` field — and
//!    `_alImportProject` *recomputes* `hash:itemHash(img,w,h)` fresh after
//!    decoding (line 27922), rather than reading one back from the file. A
//!    hash computed by this port is therefore never compared against one a
//!    browser produced; each runtime computes its own, from its own decode, on
//!    its own load. [`crate::library::ItemRecord`] already reflects this —
//!    milestone 5 shipped it with no `hash` field, before this milestone ever
//!    named the reason.
//! 2. **It could not match even if it needed to.** `itemHash` downsamples
//!    through `ctx.drawImage(img,0,0,32,32)` — a canvas resample whose exact
//!    kernel the HTML5 Canvas spec leaves implementation-defined. Two
//!    *browsers* are not obliged to produce the same 32×32 pixels for the same
//!    source image, so "matches the reference" was never a coherent bar for
//!    this function, only "matches itself" is.
//!
//! So [`item_hash`] is real, deterministic content hashing — same decoded
//! pixels in, same string out, every time, on every platform this binary
//! runs on — verified with real unit tests for that property, not
//! golden-verified against a captured browser run (none is possible; see
//! below).
//!
//! # Why `render_item`/`item_hash`/`decode_png`/`encode_png` are real unit
//! tests, not golden-parity ones
//!
//! Every prior milestone's golden tests work by lifting real reference
//! functions into a headless Node `vm.runInContext` sandbox and running them.
//! That technique has no `document`, no `HTMLCanvasElement`, no
//! `CanvasRenderingContext2D`, and no `Image`/`createImageBitmap` — so any
//! reference function that touches one (`itemHash`, `drawItemOnly`/
//! `renderItem`, `encodeItemPng`, `decodeBytes`, `decodePackImage`) simply
//! cannot execute there. `finalizePackTexture` and `fitToBottom` are the two
//! functions in this milestone's scope that touch *no* DOM API at all — pure
//! arithmetic over plain numbers/arrays — so those two, and only those two,
//! are golden-verified in `tests/golden_parity_raster.rs`, the same way as
//! every earlier milestone.

use crate::library::ItemTransform;
use image::imageops::FilterType;
use image::{Rgba, RgbaImage};
use std::fmt;

// ---------------------------------------------------------------------------
// DecodedImage
// ---------------------------------------------------------------------------

/// A decoded (or rendered) RGBA8 raster — the reference's `{img,w,h}`/
/// `ImageData`, minus the DOM handle. Straight (non-premultiplied) alpha,
/// row-major, top-to-bottom — `image`'s own layout, which is also
/// `getImageData`'s.
#[derive(Debug, Clone, PartialEq)]
pub struct DecodedImage {
    pub w: u32,
    pub h: u32,
    pub rgba: Vec<u8>,
}

impl DecodedImage {
    /// Build from raw dimensions and bytes, checking the one invariant every
    /// other function in this module relies on: `rgba.len() == w*h*4`.
    pub fn new(w: u32, h: u32, rgba: Vec<u8>) -> Result<Self, ImageError> {
        let expected = u64::from(w) * u64::from(h) * 4;
        if rgba.len() as u64 != expected {
            return Err(ImageError::BufferSize { expected, actual: rgba.len() as u64 });
        }
        Ok(DecodedImage { w, h, rgba })
    }

    /// Borrow as an `image` crate `RgbaImage` for its own algorithms
    /// (resize, overlay). Cheap to call repeatedly to avoid a second stored
    /// copy of the pixels in this module's own types; `image`'s buffer type
    /// needs owned data, so this does copy once per call.
    fn to_rgba_image(&self) -> RgbaImage {
        RgbaImage::from_raw(self.w, self.h, self.rgba.clone())
            .expect("DecodedImage invariant (rgba.len() == w*h*4) enforced at construction")
    }
}

/// What went wrong decoding, encoding, or constructing a [`DecodedImage`].
#[derive(Debug)]
pub enum ImageError {
    /// The `image` crate could not decode the bytes as a PNG.
    Decode(image::ImageError),
    /// The `image` crate could not encode a [`DecodedImage`] as a PNG.
    Encode(image::ImageError),
    /// [`DecodedImage::new`]'s own invariant failed: the buffer is not
    /// exactly `w*h*4` bytes.
    BufferSize { expected: u64, actual: u64 },
}

impl fmt::Display for ImageError {
    /// One line naming which operation failed and, for `BufferSize`, the
    /// expected-vs-actual byte counts.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ImageError::Decode(e) => write!(f, "PNG decode failed: {e}"),
            ImageError::Encode(e) => write!(f, "PNG encode failed: {e}"),
            ImageError::BufferSize { expected, actual } => {
                write!(f, "image buffer is {actual} bytes, expected {expected} (w*h*4)")
            }
        }
    }
}

impl std::error::Error for ImageError {
    /// The wrapped `image` crate error, for `Decode`/`Encode`; `None` for
    /// `BufferSize`, which has no underlying cause to chain.
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ImageError::Decode(e) | ImageError::Encode(e) => Some(e),
            ImageError::BufferSize { .. } => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Decode / encode
// ---------------------------------------------------------------------------

/// Decode PNG bytes to RGBA8 — the reference's `AssetImporter.decodeBytes`/
/// `decodePackImage`, both of which feed a `Blob` typed `image/png` (every
/// pack entry and every `assetlib/img/N.png` project entry is always a PNG;
/// this port's export side only ever writes one). Format is asserted rather
/// than sniffed, so a non-PNG file is a real [`ImageError::Decode`] rather
/// than a silent guess.
pub fn decode_png(bytes: &[u8]) -> Result<DecodedImage, ImageError> {
    let img = image::load_from_memory_with_format(bytes, image::ImageFormat::Png)
        .map_err(ImageError::Decode)?;
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    Ok(DecodedImage { w, h, rgba: rgba.into_raw() })
}

/// Decode PNG bytes to RGBA8 **as data**: the numbers in the file, with every
/// colour chunk (`cICP`, `iCCP`, `sRGB`, `gAMA`, `cHRM`) ignored. The
/// heightmap import (`cartalith-engine`'s `import.rs`) calls this, because a
/// heightmap is elevation, not colour, and must never be colour-managed
/// (`EDITING_TOOLS_SCOPE.md` ET-E2).
///
/// Today it is exactly [`decode_png`], which reads no colour chunk either.
/// It is a separate name so that the colour-managed decode ET-E2 added for
/// asset images (`cartalith-godot`'s `colour_import::decode_png_managed`,
/// which lives there because `moxcms` is a dependency of that crate and not
/// of this one) can never be wired into the heightmap path by a rename. It
/// must never read a colour chunk.
pub fn decode_png_data(bytes: &[u8]) -> Result<DecodedImage, ImageError> {
    decode_png(bytes)
}

/// Encode RGBA8 as a PNG — the reference's `encodeItemPng`. Used both for the
/// project persistence path (re-encoding an item's own decoded pixels
/// verbatim, at its native size, no transform applied — exactly what
/// `encodeItemPng` does) and, on [`render_item`]'s output, for a pack
/// export's baked slot image.
pub fn encode_png(img: &DecodedImage) -> Result<Vec<u8>, ImageError> {
    let buf = img.to_rgba_image();
    let mut out = Vec::new();
    image::DynamicImage::ImageRgba8(buf)
        .write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
        .map_err(ImageError::Encode)?;
    Ok(out)
}

/// What colour information [`encode_png_rgb8`] writes into a PNG (ET-E1,
/// `EDITING_TOOLS_SCOPE.md` section 6; `LARGE_ITEM_RULINGS.md` Ruling CB item 1:
/// *"tag every exported PNG as sRGB by default ... Display P3 stays an explicit
/// export option; the heightmap PNG stays untagged"*).
///
/// Exactly one of the two colour chunks, or neither, never both: the W3C PNG
/// specification, third edition, says *"If the iCCP chunk is present, the sRGB
/// chunk should not be present"* and the reverse, and that both must come
/// before `PLTE` and `IDAT` (read via a fetched summary of
/// <https://www.w3.org/TR/png-3/>, 2026-10-11, not the full text). Every caller
/// names its variant: there is no default here, so a data raster cannot be
/// tagged by omission and a picture cannot be left untagged by omission.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PngColour<'a> {
    /// One `sRGB` chunk, rendering intent 0 (perceptual), and no `iCCP`. The
    /// default for every exported picture (Ruling CB item 1). Perceptual is a
    /// **labelled judgement** (`EDITING_TOOLS_SCOPE.md` ET-E1 names it one):
    /// the intent only matters to a consumer converting into a smaller gamut,
    /// and for a map picture a compressed-but-ordered gamut reads better than
    /// clipped saturated greens. No `gAMA`/`cHRM` fallback chunks are written;
    /// the `png` crate's own sRGB writer omits them too unless asked.
    Srgb,
    /// One `iCCP` chunk carrying these ICC profile bytes (zlib-deflated by the
    /// `png` crate, compression method 0, the only one PNG defines), and no
    /// `sRGB`. The pixels must already be encoded in that profile's space: this
    /// writes a label, it converts nothing. An empty profile is refused rather
    /// than written as an untagged file.
    IccProfile(&'a [u8]),
    /// No colour chunk at all, for a raster of **data** that is not a colour
    /// picture (the channel atlas's packed fields, a mask tile): the same
    /// reason Ruling CB gives for keeping the heightmap untagged. A label
    /// claiming these bytes are sRGB colours would be false.
    Data,
}

/// The complete `sRGB` chunk for rendering intent 0 (perceptual): length
/// `00 00 00 01`, type `sRGB`, the one data byte `00`, then the CRC-32 (ISO
/// 3309, as PNG specifies it) over type and data, `AE CE 1C E9`. A constant
/// because it is one: every byte follows from the PNG specification. Protected
/// by `the_srgb_chunk_constant_is_the_png_crates_own_bytes_and_crc` (an
/// independent bitwise CRC) and by the `png` decoder's own CRC check in every
/// round-trip test.
const SRGB_CHUNK_PERCEPTUAL: [u8; 13] = [0, 0, 0, 1, b's', b'R', b'G', b'B', 0, 0xAE, 0xCE, 0x1C, 0xE9];

/// Where the first chunk after `IHDR` starts: the 8-byte PNG signature plus
/// `IHDR`'s 25 bytes (4 length, 4 type, 13 data, 4 CRC). `IHDR` is required to
/// be first and is always 13 data bytes (PNG specification, IHDR), so this is
/// fixed; [`encode_png_rgb8`] still checks the bytes before splicing.
const AFTER_IHDR: usize = 8 + 4 + 4 + 13 + 4;

/// Encode a tightly-packed **RGB8** buffer as a PNG, taking ownership of the
/// bytes rather than copying them, labelled as `colour` says ([`PngColour`]).
///
/// # How each label is written, and why it cannot move a pixel
///
/// The image data is encoded exactly as before ET-E1: `image`'s
/// `PngEncoder::new` (its `Fast` compression, adaptive filter), which is what
/// `DynamicImage::write_to(.., ImageFormat::Png)` used. So `IDAT` is
/// byte-identical to the untagged encode and only the chunk list grows.
/// - [`PngColour::Srgb`]: `image` has no API for an `sRGB` chunk, so the
///   constant [`SRGB_CHUNK_PERCEPTUAL`] is spliced in directly after `IHDR`
///   (before `PLTE`/`IDAT`, as the specification requires). The splice checks
///   that `IHDR` really is where the format puts it and refuses otherwise.
/// - [`PngColour::IccProfile`]: `ImageEncoder::set_icc_profile`, which the
///   `png` crate writes as one compressed `iCCP` chunk after `IHDR`.
/// - [`PngColour::Data`]: nothing added; the pre-ET-E1 file exactly.
///
/// Must never write both chunks, and must never silently drop a requested
/// label (an empty profile is an error, not an untagged file).
///
/// [`encode_png`] is the right entry point for anything that already holds a
/// [`DecodedImage`]; this one exists for the export raster
/// (`TERRAIN_APPEARANCE_SCOPE.md`'s bake milestone), whose renderer produces
/// RGB8 and whose largest output is 32 768 px wide since owner ruling 15
/// (8192 before it). Routing that through
/// `DecodedImage` would mean widening to RGBA (+33%) and then
/// `to_rgba_image`'s own `clone()` (+100%) — roughly 470 MB of transient
/// allocation on a single 8K export against the 129 MB the pixels actually
/// occupy. An alpha channel that is `255` everywhere is not worth that, and
/// the PNG is smaller without it besides.
pub fn encode_png_rgb8(w: u32, h: u32, rgb: Vec<u8>, colour: PngColour<'_>) -> Result<Vec<u8>, ImageError> {
    use image::ImageEncoder;
    let expected = u64::from(w) * u64::from(h) * 3;
    if rgb.len() as u64 != expected {
        return Err(ImageError::BufferSize { expected, actual: rgb.len() as u64 });
    }
    let encode_err = |what: &str| {
        ImageError::Encode(image::ImageError::Parameter(image::error::ParameterError::from_kind(image::error::ParameterErrorKind::Generic(what.to_string()))))
    };
    let mut out = Vec::new();
    let mut enc = image::codecs::png::PngEncoder::new(&mut out);
    if let PngColour::IccProfile(profile) = colour {
        if profile.is_empty() {
            return Err(encode_err("an empty ICC profile would write an untagged PNG"));
        }
        enc.set_icc_profile(profile.to_vec()).map_err(|e| ImageError::Encode(image::ImageError::Unsupported(e)))?;
    }
    enc.write_image(&rgb, w, h, image::ExtendedColorType::Rgb8).map_err(ImageError::Encode)?;
    drop(rgb);
    if colour == PngColour::Srgb {
        // `IHDR` must be the first chunk, 13 data bytes long; anything else
        // is not a file this splice understands, so it refuses.
        if out.len() < AFTER_IHDR || out[8..16] != [0, 0, 0, 13, b'I', b'H', b'D', b'R'] {
            return Err(encode_err("the encoder did not put a 13-byte IHDR first"));
        }
        out.splice(AFTER_IHDR..AFTER_IHDR, SRGB_CHUNK_PERCEPTUAL);
    }
    Ok(out)
}

/// Encode a tightly-packed **16-bit grayscale** buffer as a PNG.
///
/// For the heightmap export: a height field is one channel of continuous
/// data, and 8 bits of it quantises a world's whole elevation range into 256
/// steps -- visible as terracing the moment anything downstream takes a
/// gradient. 16 bits is what the tools this format exists for (game engines,
/// terrain editors, `image`-reading DCC apps) expect.
///
/// PNG stores 16-bit samples big-endian, which `image` handles; the caller
/// passes native-endian `u16`s.
///
/// **Untagged, deliberately**: neither `sRGB` nor `iCCP` (`LARGE_ITEM_RULINGS.md`
/// Ruling CB item 1: *"the heightmap PNG stays untagged"*; ET-E1). A height
/// field is data, not colour, and a colour label invites a colour-managing
/// reader to convert it. Must never gain a colour chunk; protected by
/// `the_heightmap_png_carries_no_colour_chunk`.
///
/// Sibling of [`encode_png_rgb8`] and written the same way and for the same
/// reason -- it takes ownership rather than copying, because the largest
/// heightmap this ships is 32 768 px wide since ruling 15 and a needless clone
/// there is 2.06 GB — it was 8192 px and 128 MB when this was written.
pub fn encode_png_luma16(w: u32, h: u32, gray: Vec<u16>) -> Result<Vec<u8>, ImageError> {
    let expected = u64::from(w) * u64::from(h);
    if gray.len() as u64 != expected {
        return Err(ImageError::BufferSize { expected, actual: gray.len() as u64 });
    }
    let buf = image::ImageBuffer::<image::Luma<u16>, Vec<u16>>::from_raw(w, h, gray)
        .ok_or(ImageError::BufferSize { expected, actual: expected })?;
    let mut out = Vec::new();
    image::DynamicImage::ImageLuma16(buf)
        .write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
        .map_err(ImageError::Encode)?;
    Ok(out)
}

// ---------------------------------------------------------------------------
// itemHash
// ---------------------------------------------------------------------------

/// Real content hash from decoded pixels — the reference's `itemHash(img,w,h)`
/// (line 26913): downsample to 32×32, FNV-1a-with-stride-7 over the result,
/// append `-{w}x{h}` (the *original* dimensions, not the thumbnail's). See the
/// module docs for why this need not, and cannot, match the reference's own
/// output byte for byte — it only needs to be deterministic (same pixels in,
/// same string out) for [`crate::library::duplicate_groups`]'s dedup to work.
///
/// The downsample uses `image`'s `Triangle` (bilinear-equivalent) filter — a
/// concrete, deterministic choice standing in for the reference's own
/// implementation-defined canvas resample. The hash constants themselves (FNV
/// offset basis `0x811c9dc5`, prime `0x01000193`, stride 7, 32-bit wrapping
/// multiply) are ported verbatim; only the resampling kernel that produces the
/// bytes those constants hash is a deliberate substitution.
pub fn item_hash(img: &DecodedImage) -> String {
    let thumb = thumbnail_32(img);
    let mut hsh: u32 = 0x811c_9dc5;
    let mut i = 0usize;
    while i < thumb.len() {
        hsh ^= u32::from(thumb[i]);
        hsh = hsh.wrapping_mul(0x0100_0193);
        i += 7;
    }
    format!("{hsh:x}-{}x{}", img.w, img.h)
}

/// The fixed 32x32 RGBA resample [`item_hash`] hashes -- a solid-colour
/// buffer for a degenerate 0-width/height image, so hashing never divides
/// or samples out of bounds.
fn thumbnail_32(img: &DecodedImage) -> Vec<u8> {
    if img.w == 0 || img.h == 0 {
        return vec![0u8; 32 * 32 * 4];
    }
    image::imageops::resize(&img.to_rgba_image(), 32, 32, FilterType::Triangle).into_raw()
}

// ---------------------------------------------------------------------------
// fitToBottom
// ---------------------------------------------------------------------------

/// Anchor an item's display transform so its baked footprint's bottom edge
/// sits on the slot's vertical centre — the reference's `fitToBottom(item,
/// size)` (line 26769), applied on intake to every `anchor:'bottom'` family
/// (feature icons only). Mutates `panY`; leaves `panX`/`scale` untouched.
///
/// Golden-verified in `tests/golden_parity_raster.rs` (pure arithmetic, no
/// DOM — see the module docs on why this one qualifies and `item_hash` does
/// not).
pub fn fit_to_bottom(transform: &mut ItemTransform, w: u32, h: u32, size: u32) {
    let base_fit = f64::from(size) / f64::from(w.max(h));
    transform.pan_y = (f64::from(size) - f64::from(h) * base_fit * transform.scale) / 2.0;
}

// ---------------------------------------------------------------------------
// render_item (ThumbnailRenderer's shared core: thumbnail / preview / bake)
// ---------------------------------------------------------------------------

/// Composite one item onto a `size×size` canvas — the reference's
/// `drawItemOnly`/`renderItem` (`ThumbnailRenderer`'s shared core, lines
/// 26751-26762), which the reference itself uses for grid thumbnails, the
/// inspector preview, *and* a pack export's baked slot image alike (its own
/// module comment: "shared render core (thumbnails, inspector preview, export
/// bake)"). Scales `img` to fit `size` on its longer side, times the
/// transform's own `scale`, centres it, then offsets by `panX`/`panY`.
/// `opaque` pre-fills solid black before compositing (ground-texture bake, so
/// alpha flattens onto black rather than staying transparent); sprites keep
/// their own alpha over a transparent canvas.
///
/// The reference draws through `CanvasRenderingContext2D.drawImage` with
/// `imageSmoothingQuality:'high'` — an implementation-defined resampling
/// kernel no two browsers (or a Rust port) are obliged to agree on bit for
/// bit. This uses `image`'s `CatmullRom` filter as a concrete stand-in for
/// "high quality"; the *geometry* — position, size, alpha compositing via
/// source-over — is exact, only the resampling kernel is not
/// reference-identical (same caveat as [`item_hash`]'s downsample, and for the
/// same underlying reason).
pub fn render_item(img: &DecodedImage, transform: &ItemTransform, size: u32, opaque: bool) -> DecodedImage {
    let base_fit = f64::from(size) / f64::from(img.w.max(img.h));
    let dw = (f64::from(img.w) * base_fit * transform.scale).round();
    let dh = (f64::from(img.h) * base_fit * transform.scale).round();
    let dw_u = dw.max(0.0) as u32;
    let dh_u = dh.max(0.0) as u32;
    let dx = ((f64::from(size) - dw) / 2.0 + transform.pan_x).round() as i64;
    let dy = ((f64::from(size) - dh) / 2.0 + transform.pan_y).round() as i64;

    let bg = if opaque { Rgba([0, 0, 0, 255]) } else { Rgba([0, 0, 0, 0]) };
    let mut canvas = RgbaImage::from_pixel(size, size, bg);

    if dw_u > 0 && dh_u > 0 {
        let scaled = image::imageops::resize(&img.to_rgba_image(), dw_u, dh_u, FilterType::CatmullRom);
        image::imageops::overlay(&mut canvas, &scaled, dx, dy);
    }
    DecodedImage { w: size, h: size, rgba: canvas.into_raw() }
}

/// Which arithmetic [`render_item_with`] bakes in.
///
/// Ruling CR (2026-10-18, last paragraph) set this fix aside as "a separate,
/// cheaper change behind a switch"; `MISTAKES.md`'s "Change generated output"
/// row is why it is a switch. The default is [`BakeFilter::Straight`], today's
/// bytes, so every caller that does not opt in (this crate's own pack report,
/// near-duplicate finder and `lineart::working_image`, and every test pin) is
/// untouched; the app opts in through `AssetLibrarySession::bake_filter`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BakeFilter {
    /// [`render_item`] exactly: `CatmullRom` on **straight** alpha, then
    /// `imageops::overlay`. Not a fixed point: AP-1 measured every pixel
    /// with 0 < alpha < 255 moving RGB by one level on every bake
    /// (`ASSET_PACK_TOOLS_PROPOSAL.md` §10, AP-1), and straight-alpha
    /// filtering pulls a transparent texel's RGB (usually black) into the
    /// edge, a dark fringe.
    #[default]
    Straight,
    /// Premultiplied, in `f32` from decode to the final rounding: the source
    /// is premultiplied, resampled with the same `CatmullRom` kernel (or not
    /// resampled at all when the footprint already equals the source's
    /// size), composited source-over in premultiplied form, and quantised
    /// once. A bake at the source's own size and the identity transform
    /// therefore returns every visible pixel unchanged, and a second bake of
    /// a first bake's output returns it byte for byte (both tested).
    Premultiplied,
}

/// [`render_item`] with an explicit [`BakeFilter`]. Same geometry for both
/// filters (footprint, centring, pan, the `opaque` black backdrop), so the
/// two differ only in the pixels on soft or resampled alpha.
///
/// **Fully opaque sources take [`render_item`]'s own code under either
/// filter** (identity by control flow, not arithmetic): premultiplying by
/// 255 is the identity, so there is nothing for the fix to change, and the
/// ground textures' and opaque sprites' bytes cannot move. Must never change
/// an alpha-255 source's output.
pub fn render_item_with(img: &DecodedImage, transform: &ItemTransform, size: u32, opaque: bool, filter: BakeFilter) -> DecodedImage {
    if filter == BakeFilter::Straight || img.rgba.chunks_exact(4).all(|p| p[3] == 255) {
        return render_item(img, transform, size, opaque);
    }
    // The same footprint arithmetic as `render_item`, statement for statement.
    let base_fit = f64::from(size) / f64::from(img.w.max(img.h));
    let dw = (f64::from(img.w) * base_fit * transform.scale).round();
    let dh = (f64::from(img.h) * base_fit * transform.scale).round();
    let dw_u = dw.max(0.0) as u32;
    let dh_u = dh.max(0.0) as u32;
    let dx = ((f64::from(size) - dw) / 2.0 + transform.pan_x).round() as i64;
    let dy = ((f64::from(size) - dh) / 2.0 + transform.pan_y).round() as i64;

    // Premultiplied canvas in [0, 1]; the backdrop is opaque black or clear.
    let n = (size as usize) * (size as usize);
    let mut canvas = vec![[0.0f32, 0.0, 0.0, if opaque { 1.0 } else { 0.0 }]; n];
    if dw_u > 0 && dh_u > 0 {
        let mut src = image::Rgba32FImage::new(img.w, img.h);
        for (p, s) in src.pixels_mut().zip(img.rgba.chunks_exact(4)) {
            let a = f32::from(s[3]) / 255.0;
            *p = Rgba([f32::from(s[0]) / 255.0 * a, f32::from(s[1]) / 255.0 * a, f32::from(s[2]) / 255.0 * a, a]);
        }
        // No resample when the footprint is the source's own size: a kernel
        // evaluated at its own taps is the identity in exact arithmetic, but
        // not guaranteed bit-exact in f32, and this is the round-trip case.
        let scaled = if (dw_u, dh_u) == (img.w, img.h) { src } else { image::imageops::resize(&src, dw_u, dh_u, FilterType::CatmullRom) };
        for (sy, row) in scaled.rows().enumerate() {
            let cy = dy + sy as i64;
            if cy < 0 || cy >= i64::from(size) {
                continue;
            }
            for (sx, p) in row.enumerate() {
                let cx = dx + sx as i64;
                if cx < 0 || cx >= i64::from(size) {
                    continue;
                }
                // CatmullRom rings: clamp alpha into [0, 1] and colour into
                // [0, alpha], the premultiplied invariant, before compositing.
                let a = p[3].clamp(0.0, 1.0);
                let s = [p[0].clamp(0.0, a), p[1].clamp(0.0, a), p[2].clamp(0.0, a), a];
                let d = &mut canvas[cy as usize * size as usize + cx as usize];
                for c in 0..4 {
                    d[c] = s[c] + d[c] * (1.0 - a);
                }
            }
        }
    }
    // Quantise once: alpha to the nearest level, colour un-premultiplied by
    // the float alpha (not the rounded one, which is what keeps an identity
    // bake exact); RGB under alpha 0 is 0, the decoder's own convention.
    let mut rgba = Vec::with_capacity(n * 4);
    for p in &canvas {
        let a8 = (p[3] * 255.0).round().clamp(0.0, 255.0) as u8;
        for c in 0..3 {
            rgba.push(if a8 == 0 || p[3] <= 0.0 { 0 } else { (p[c] / p[3] * 255.0).round().clamp(0.0, 255.0) as u8 });
        }
        rgba.push(a8);
    }
    DecodedImage { w: size, h: size, rgba }
}

// ---------------------------------------------------------------------------
// finalizePackTexture's inverse means
// ---------------------------------------------------------------------------

/// Per-channel inverse mean over a decoded texture's pixels — the reference's
/// `finalizePackTexture(w,h,data)` (line 12196). "Inverse means", literally:
/// the mean of each of R/G/B across every pixel, clamped so it is never
/// treated as less than 1 (`Math.max(1,mean)`, so an almost-black slot cannot
/// blow the reciprocal up past 1), then reciprocated. `n===0` (no pixels)
/// answers `[0,0,0]` for all three, matching the reference's own `n?...:0`.
///
/// Used only by the `textures` (splat-channel) family: the live splat blend
/// modulates a procedural material ramp by `texel/mean`, so a texture's own
/// average brightness factors back out and only its relative variation
/// contributes. `biomes`/`terrains` deliberately **skip** this (reference
/// line 12246) — they are sampled as true colour against the painted
/// Cartography layers, not splat-modulated, and calling this for those two
/// families would be wrong. Consuming this in the renderer is milestone 7's
/// job; this is only the arithmetic.
///
/// Golden-verified in `tests/golden_parity_raster.rs` (pure arithmetic, no
/// DOM).
pub fn finalize_pack_texture_inv_mean(w: u32, h: u32, rgba: &[u8]) -> [f64; 3] {
    let n = u64::from(w) * u64::from(h);
    if n == 0 {
        return [0.0, 0.0, 0.0];
    }
    let mut sr = 0u64;
    let mut sg = 0u64;
    let mut sb = 0u64;
    for px in rgba.chunks_exact(4) {
        sr += u64::from(px[0]);
        sg += u64::from(px[1]);
        sb += u64::from(px[2]);
    }
    let n_f = n as f64;
    [
        1.0 / (sr as f64 / n_f).max(1.0),
        1.0 / (sg as f64 / n_f).max(1.0),
        1.0 / (sb as f64 / n_f).max(1.0),
    ]
}

// ---------------------------------------------------------------------------
// Unit tests (real unit tests, not golden-parity -- see the module docs)
// ---------------------------------------------------------------------------
/// [`DecodedImage`]'s invariant, the PNG round trip, [`item_hash`] and
/// [`render_item`]'s pixel arithmetic.
#[cfg(test)]
mod tests {
    use super::*;

    /// A `w`x`h` [`DecodedImage`] filled with one RGBA colour.
    fn solid(w: u32, h: u32, rgba: [u8; 4]) -> DecodedImage {
        let mut data = Vec::with_capacity((w * h * 4) as usize);
        for _ in 0..(w * h) {
            data.extend_from_slice(&rgba);
        }
        DecodedImage::new(w, h, data).unwrap()
    }

    /// Protects: a buffer not exactly `w*h*4` bytes fails construction with
    /// `BufferSize` carrying the real expected/actual byte counts.
    #[test]
    fn decoded_image_rejects_a_mismatched_buffer() {
        let err = DecodedImage::new(4, 4, vec![0u8; 10]).unwrap_err();
        match err {
            ImageError::BufferSize { expected, actual } => {
                assert_eq!(expected, 64);
                assert_eq!(actual, 10);
            }
            other => panic!("expected BufferSize, got {other:?}"),
        }
    }

    /// Protects: PNG is lossless, so encode-then-decode must reproduce
    /// every byte, not merely something visually close.
    #[test]
    fn encode_then_decode_round_trips_pixels_exactly() {
        // PNG is lossless, so a real round trip must reproduce every byte,
        // not merely something visually close.
        let img = solid(6, 5, [12, 200, 44, 255]);
        let bytes = encode_png(&img).unwrap();
        let back = decode_png(&bytes).unwrap();
        assert_eq!(back, img);
    }

    /// Protects: garbage bytes fail decode with an error, not a panic.
    #[test]
    fn decode_png_rejects_non_png_bytes() {
        assert!(decode_png(b"not a png").is_err());
    }

    /// Protects: identical pixels hash identically -- the property
    /// `duplicate_groups`/`slot_has_dupe` in `library.rs` depend on.
    #[test]
    fn item_hash_is_deterministic_for_identical_pixels() {
        let a = solid(40, 40, [10, 20, 30, 255]);
        let b = solid(40, 40, [10, 20, 30, 255]);
        assert_eq!(item_hash(&a), item_hash(&b));
    }

    /// Protects: different pixel content changes the hash.
    #[test]
    fn item_hash_differs_for_different_pixels() {
        let a = solid(40, 40, [10, 20, 30, 255]);
        let b = solid(40, 40, [200, 20, 30, 255]);
        assert_ne!(item_hash(&a), item_hash(&b));
    }

    /// Protects: the `-{w}x{h}` suffix, so same-content-different-size
    /// items never collide regardless of what the pixel hash half computes.
    #[test]
    fn item_hash_differs_for_different_dimensions_even_with_identical_downsample() {
        // The `-{w}x{h}` suffix means same-content-different-size items never
        // collide, exactly as the reference's own string concatenation
        // guarantees regardless of what the pixel hash half computes.
        let a = solid(32, 32, [5, 5, 5, 255]);
        let b = solid(64, 64, [5, 5, 5, 255]);
        assert_ne!(item_hash(&a), item_hash(&b));
        assert!(item_hash(&a).ends_with("-32x32"));
        assert!(item_hash(&b).ends_with("-64x64"));
    }

    /// Protects: the output canvas is always exactly `size`x`size`,
    /// regardless of the source image's own dimensions.
    #[test]
    fn render_item_produces_a_size_by_size_canvas() {
        let src = solid(10, 20, [255, 0, 0, 255]);
        let t = ItemTransform::default();
        let out = render_item(&src, &t, 64, false);
        assert_eq!((out.w, out.h), (64, 64));
        assert_eq!(out.rgba.len(), 64 * 64 * 4);
    }

    /// Protects: `opaque=true` fills the area outside the scaled sprite
    /// with solid black, not transparent.
    #[test]
    fn render_item_opaque_fills_the_backdrop_black() {
        let src = solid(4, 4, [255, 255, 255, 255]);
        let t = ItemTransform { scale: 0.1, pan_x: 0.0, pan_y: 0.0 };
        let out = render_item(&src, &t, 32, true);
        // Far corner is outside the tiny scaled sprite -> the opaque backdrop.
        assert_eq!(&out.rgba[0..4], &[0, 0, 0, 255]);
    }

    /// Protects: `opaque=false` leaves the area outside the scaled sprite
    /// fully transparent (alpha 0), the counterpart to the opaque case above.
    #[test]
    fn render_item_transparent_leaves_the_backdrop_empty() {
        let src = solid(4, 4, [255, 255, 255, 255]);
        let t = ItemTransform { scale: 0.1, pan_x: 0.0, pan_y: 0.0 };
        let out = render_item(&src, &t, 32, false);
        assert_eq!(&out.rgba[0..4], &[0, 0, 0, 0]);
    }

    /// Protects: a square item at the default transform (scale 1, no pan)
    /// fills the whole canvas edge to edge with its own colour.
    #[test]
    fn render_item_centres_a_square_item_at_default_transform() {
        // A square item at scale 1 fills the whole canvas edge to edge.
        let src = solid(8, 8, [1, 2, 3, 255]);
        let t = ItemTransform::default();
        let out = render_item(&src, &t, 16, false);
        for chunk in out.rgba.chunks_exact(4) {
            assert_eq!(chunk, &[1, 2, 3, 255]);
        }
    }

    /// A `n`x`n` soft-edged sprite: an RGB gradient whose alpha is a radial
    /// falloff, so every alpha level 0..=255 occurs and both the dark-fringe
    /// and the drift cases are reached. Fixture values.
    fn soft_sprite(n: u32) -> DecodedImage {
        let mut v = Vec::with_capacity((n * n * 4) as usize);
        let c = f64::from(n) / 2.0;
        for y in 0..n {
            for x in 0..n {
                let r = ((f64::from(x) + 0.5 - c).hypot(f64::from(y) + 0.5 - c)) / c;
                let a = ((1.0 - r) * 2.0 * 255.0).clamp(0.0, 255.0) as u8;
                v.extend_from_slice(&[(x * 255 / n) as u8, (y * 255 / n) as u8, 200, a]);
            }
        }
        DecodedImage::new(n, n, v).unwrap()
    }

    /// Protects: the premultiplied bake at the source's own size and the
    /// identity transform returns every visible pixel byte for byte (the
    /// round trip AP-1 found drifting), alpha included. Positive control:
    /// the straight bake of the same sprite does move visible RGB, so the
    /// fixture reaches the drift.
    #[test]
    fn the_premultiplied_identity_bake_keeps_every_visible_pixel() {
        let src = soft_sprite(64);
        let t = ItemTransform::default();
        let pre = render_item_with(&src, &t, 64, false, BakeFilter::Premultiplied);
        let straight = render_item_with(&src, &t, 64, false, BakeFilter::Straight);
        let visible = |o: &DecodedImage| src.rgba.chunks_exact(4).zip(o.rgba.chunks_exact(4)).filter(|(a, _)| a[3] > 0).filter(|(a, b)| a != b).count();
        assert_eq!(visible(&pre), 0, "premultiplied identity bake moved a visible pixel");
        assert!(visible(&straight) > 0, "positive control: the straight bake should drift on this fixture");
    }

    /// Protects: the premultiplied bake is a fixed point -- baking a bake
    /// again (identity transform at the bake's size) changes no byte -- for a
    /// source that had to be resampled first (96 -> 64), so the second pass
    /// starts from resampled, already-quantised pixels.
    #[test]
    fn the_premultiplied_bake_of_a_bake_is_byte_identical() {
        let t = ItemTransform::default();
        let once = render_item_with(&soft_sprite(96), &t, 64, false, BakeFilter::Premultiplied);
        let twice = render_item_with(&once, &t, 64, false, BakeFilter::Premultiplied);
        assert_eq!(twice, once);
    }

    /// Protects: a fully opaque source bakes byte-identically under both
    /// filters, resampled and on the black backdrop alike (the switch must
    /// never move a ground texture or an opaque sprite).
    #[test]
    fn an_opaque_source_bakes_identically_under_both_filters() {
        let mut src = soft_sprite(40);
        for p in src.rgba.chunks_exact_mut(4) {
            p[3] = 255;
        }
        for (size, opaque, scale) in [(32, true, 1.0), (32, false, 0.7), (64, true, 1.3)] {
            let t = ItemTransform { scale, pan_x: 3.0, pan_y: -2.0 };
            assert_eq!(
                render_item_with(&src, &t, size, opaque, BakeFilter::Premultiplied),
                render_item_with(&src, &t, size, opaque, BakeFilter::Straight),
                "size {size} opaque {opaque} scale {scale}"
            );
        }
    }

    /// Protects: no dark fringe -- a white sprite whose RGB under alpha 0 is
    /// black, shrunk 4x, keeps every visible pixel white in the premultiplied
    /// bake, where the straight bake darkens the soft edge (the positive
    /// control: its darkest visible pixel is well below white).
    #[test]
    fn the_premultiplied_bake_has_no_dark_fringe() {
        let mut src = soft_sprite(128);
        for p in src.rgba.chunks_exact_mut(4) {
            let on = p[3] > 0;
            p[0] = if on { 255 } else { 0 };
            p[1] = p[0];
            p[2] = p[0];
        }
        let t = ItemTransform::default();
        let darkest = |o: &DecodedImage| o.rgba.chunks_exact(4).filter(|p| p[3] >= 8).map(|p| p[0].min(p[1]).min(p[2])).min().unwrap();
        assert_eq!(darkest(&render_item_with(&src, &t, 32, false, BakeFilter::Premultiplied)), 255);
        assert!(darkest(&render_item_with(&src, &t, 32, false, BakeFilter::Straight)) < 200, "positive control");
    }

    /// Protects: the opaque-backdrop premultiplied bake flattens a soft
    /// pixel onto black as `round(c * a / 255)`, the source-over value. A
    /// 1x1 source of (200, 100, 50, 128) at its own size: 100.39, 50.2, 25.1.
    #[test]
    fn the_premultiplied_opaque_bake_flattens_onto_black() {
        let src = DecodedImage::new(1, 1, vec![200, 100, 50, 128]).unwrap();
        let out = render_item_with(&src, &ItemTransform::default(), 1, true, BakeFilter::Premultiplied);
        assert_eq!(out.rgba, vec![100, 50, 25, 255]);
    }
}

/// [`encode_png_luma16`]'s real 16-bit depth, not a truncated 8-bit encode.
#[cfg(test)]
mod luma16_tests {
    use super::*;

    /// Protects: 16 bits, not 8 -- the whole reason this encoder exists. A
    /// gradient of 1024 distinct levels must survive the round trip; at 8
    /// bits it would collapse to 256 and this fails.
    #[test]
    fn sixteen_bit_levels_survive_the_round_trip() {
        let (w, h) = (1024usize, 1usize);
        let src: Vec<u16> = (0..w).map(|i| (i * 64) as u16).collect();
        let png = encode_png_luma16(w as u32, h as u32, src.clone()).expect("encode");
        let back = decode_png(&png).expect("decode");
        assert_eq!((back.w as usize, back.h as usize), (w, h));

        let distinct: std::collections::BTreeSet<u8> =
            (0..w).map(|i| back.rgba[i * 4]).collect();
        assert!(
            distinct.len() > 200,
            "decode collapsed the ramp to {} levels; the encoder or the decoder is 8-bit",
            distinct.len()
        );
        // Grayscale means R == G == B, and opaque.
        for i in 0..w {
            let p = i * 4;
            assert_eq!(back.rgba[p], back.rgba[p + 1], "not grayscale at {i}");
            assert_eq!(back.rgba[p + 1], back.rgba[p + 2], "not grayscale at {i}");
            assert_eq!(back.rgba[p + 3], 255, "not opaque at {i}");
        }
    }

    /// Protects: a buffer not exactly `w*h` u16s fails with `BufferSize`
    /// carrying the real expected/actual counts, not a panic.
    #[test]
    fn a_wrong_sized_buffer_is_refused_rather_than_panicking() {
        let e = encode_png_luma16(4, 4, vec![0u16; 15]);
        assert!(matches!(e, Err(ImageError::BufferSize { expected: 16, actual: 15 })), "{e:?}");
    }

    /// Protects: Ruling CB item 1's exception -- the heightmap PNG carries
    /// neither `sRGB` nor `iCCP` (it is data). Fails if anyone routes the
    /// 16-bit encoder through a colour label.
    #[test]
    fn the_heightmap_png_carries_no_colour_chunk() {
        let png = encode_png_luma16(9, 5, (0..45u16).map(|i| i * 1000).collect()).expect("encode");
        let kinds = super::colour_tests::chunk_types(&png);
        assert_eq!(kinds.first().map(|k| &k[..]), Some(&b"IHDR"[..]));
        assert!(!kinds.iter().any(|k| k == b"sRGB" || k == b"iCCP" || k == b"gAMA" || k == b"cHRM" || k == b"cICP"), "{kinds:?}");
    }
}

/// ET-E1 (`EDITING_TOOLS_SCOPE.md` section 6, Ruling CB item 1): the colour
/// labels [`encode_png_rgb8`] writes. Chunk lists are read by this module's
/// own parser (lengths, types, CRCs), not by the encoder that wrote them.
#[cfg(test)]
pub(crate) mod colour_tests {
    use super::*;

    /// CRC-32 (ISO 3309, as PNG specifies it), bit by bit: reflected
    /// polynomial `0xEDB88320`, initial value and final XOR all ones. Written
    /// here rather than borrowed so a CRC check cannot share a bug with the
    /// constant it checks.
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

    /// Every chunk type in `png`, in file order, checking the signature, each
    /// length and each CRC, and that the file ends exactly at `IEND`. Panics
    /// on a malformed file; asserts the list is non-empty (`MISTAKES.md`:
    /// a helper returning a collection asserts non-empty).
    pub(crate) fn chunk_types(png: &[u8]) -> Vec<[u8; 4]> {
        assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n", "not a PNG signature");
        let (mut p, mut out) = (8usize, Vec::new());
        while p < png.len() {
            let len = u32::from_be_bytes(png[p..p + 4].try_into().unwrap()) as usize;
            let body = &png[p + 4..p + 8 + len];
            let crc = u32::from_be_bytes(png[p + 8 + len..p + 12 + len].try_into().unwrap());
            assert_eq!(crc32(body), crc, "CRC mismatch in chunk at {p}");
            out.push(body[..4].try_into().unwrap());
            p += 12 + len;
            if &body[..4] == b"IEND" {
                break;
            }
        }
        assert_eq!(p, png.len(), "bytes after IEND, or no IEND");
        assert!(!out.is_empty(), "a PNG with no chunks proves nothing");
        out
    }

    /// How many chunks of type `kind` the file holds.
    pub(crate) fn count(png: &[u8], kind: &[u8; 4]) -> usize {
        chunk_types(png).iter().filter(|k| *k == kind).count()
    }

    /// A 37 x 23 RGB fixture with every channel varying (an odd size, so no
    /// row length is a round number); deterministic, generator-free.
    pub(crate) fn fixture() -> (u32, u32, Vec<u8>) {
        let (w, h) = (37u32, 23u32);
        let px = (0..w * h * 3).map(|i| ((i * 7919 + i / 111 * 13) % 251) as u8).collect();
        (w, h, px)
    }

    /// FNV-1a 64 (standard offset basis and prime): a change detector only.
    fn fnv(b: &[u8]) -> u64 {
        assert!(!b.is_empty(), "an empty buffer hashes to the basis");
        b.iter().fold(0xcbf2_9ce4_8422_2325u64, |h, &x| (h ^ x as u64).wrapping_mul(0x0000_0100_0000_01b3))
    }

    /// Protects: the spliced constant really is the PNG `sRGB` chunk for
    /// intent 0 -- length 1, type, data byte 0, and a CRC this test computes
    /// independently. A wrong CRC would make strict decoders reject every
    /// default export.
    #[test]
    fn the_srgb_chunk_constant_is_the_png_crates_own_bytes_and_crc() {
        let c = SRGB_CHUNK_PERCEPTUAL;
        assert_eq!(&c[..4], &[0, 0, 0, 1], "length");
        assert_eq!(&c[4..8], b"sRGB", "type");
        assert_eq!(c[8], 0, "rendering intent 0 = perceptual (PNG specification, sRGB)");
        assert_eq!(u32::from_be_bytes(c[9..13].try_into().unwrap()), crc32(&c[4..9]), "CRC");
        assert_eq!(AFTER_IHDR, 33, "signature 8 + IHDR 25");
    }

    /// Protects: the default label -- exactly one `sRGB`, no `iCCP`, the
    /// `sRGB` chunk before `IDAT`, the decoded pixels equal to the input, and
    /// the file otherwise byte-identical to today's untagged encode, whose
    /// hash was produced by HEAD's own `encode_png_rgb8` (`DynamicImage::
    /// write_to`) on this fixture, run from a scratch crate before ET-E1
    /// edited this file (re-baselined 2026-10-11 under Ruling CB item 1: the
    /// file gains one chunk, its pixels do not move).
    #[test]
    fn a_default_png_carries_one_srgb_chunk_and_today_s_pixels() {
        let (w, h, px) = fixture();
        let tagged = encode_png_rgb8(w, h, px.clone(), PngColour::Srgb).expect("encode");
        let kinds = chunk_types(&tagged);
        assert_eq!(count(&tagged, b"sRGB"), 1, "{kinds:?}");
        assert_eq!(count(&tagged, b"iCCP"), 0, "{kinds:?}");
        let srgb_at = kinds.iter().position(|k| k == b"sRGB").unwrap();
        let idat_at = kinds.iter().position(|k| k == b"IDAT").unwrap();
        assert!(srgb_at == 1 && srgb_at < idat_at, "sRGB must follow IHDR and precede IDAT: {kinds:?}");
        assert_eq!(decode_png(&tagged).expect("a strict decoder reads it").rgba.chunks_exact(4).flat_map(|p| [p[0], p[1], p[2]]).collect::<Vec<_>>(), px, "the decoded pixels moved");
        // The rest of the file is exactly the untagged encode.
        let mut stripped = tagged.clone();
        stripped.drain(AFTER_IHDR..AFTER_IHDR + 13);
        let data = encode_png_rgb8(w, h, px, PngColour::Data).expect("encode");
        assert_eq!(stripped, data, "tagging changed something other than the one chunk");
        assert_eq!(fnv(&data), HEAD_UNTAGGED_FNV, "the untagged encode is no longer HEAD's");
    }

    /// FNV-1a 64 of HEAD's (`608d4435`) `encode_png_rgb8` output on
    /// [`fixture`], produced by a scratch crate calling `image` 0.25.10's
    /// `DynamicImage::write_to(.., Png)` exactly as HEAD did -- never by the
    /// code under test (it printed `fnv=0x03a45927be6d69eb len=2644`).
    const HEAD_UNTAGGED_FNV: u64 = 0x03a4_5927_be6d_69eb;

    /// Protects: the Display P3 label -- exactly one `iCCP`, no `sRGB`, the
    /// profile bytes round-trip through a decoder unchanged, pixels unmoved;
    /// and an empty profile is refused rather than silently written untagged.
    /// (The profile's primaries are checked in `cartalith-godot`, which owns
    /// the real P3 profile; any non-empty bytes exercise the container here.)
    #[test]
    fn an_icc_png_carries_one_iccp_and_no_srgb() {
        use image::ImageDecoder;
        let (w, h, px) = fixture();
        let profile: Vec<u8> = (0..300u32).map(|i| (i * 31 % 256) as u8).collect();
        let tagged = encode_png_rgb8(w, h, px.clone(), PngColour::IccProfile(&profile)).expect("encode");
        assert_eq!(count(&tagged, b"iCCP"), 1);
        assert_eq!(count(&tagged, b"sRGB"), 0);
        let mut dec = image::codecs::png::PngDecoder::new(std::io::Cursor::new(&tagged)).expect("decoder");
        assert_eq!(dec.icc_profile().expect("icc").as_deref(), Some(&profile[..]), "the profile did not round-trip");
        assert_eq!(decode_png(&tagged).unwrap().rgba.chunks_exact(4).flat_map(|p| [p[0], p[1], p[2]]).collect::<Vec<_>>(), px);
        assert!(encode_png_rgb8(w, h, px, PngColour::IccProfile(&[])).is_err(), "an empty profile must be refused");
    }

    /// Protects: a data raster stays untagged -- neither chunk.
    #[test]
    fn a_data_png_carries_no_colour_chunk() {
        let (w, h, px) = fixture();
        let png = encode_png_rgb8(w, h, px, PngColour::Data).expect("encode");
        assert_eq!((count(&png, b"sRGB"), count(&png, b"iCCP")), (0, 0));
    }

    /// Protects: ET-E2's "heightmaps are never converted" at the data
    /// decode itself -- an sRGB-tagged and an ICC-tagged file decode to the
    /// same RGBA as their untagged twin.
    #[test]
    fn the_data_decode_ignores_colour_chunks() {
        let (w, h, px) = fixture();
        let plain = decode_png_data(&encode_png_rgb8(w, h, px.clone(), PngColour::Data).unwrap()).unwrap();
        assert!(!plain.rgba.is_empty());
        let profile: Vec<u8> = (0..300u32).map(|i| (i * 31 % 256) as u8).collect();
        for colour in [PngColour::Srgb, PngColour::IccProfile(&profile)] {
            let tagged = encode_png_rgb8(w, h, px.clone(), colour).unwrap();
            assert_eq!(decode_png_data(&tagged).unwrap(), plain);
        }
    }
}
