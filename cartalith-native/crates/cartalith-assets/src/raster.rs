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
#[cfg(test)]
mod tests {
    use super::*;

    fn solid(w: u32, h: u32, rgba: [u8; 4]) -> DecodedImage {
        let mut data = Vec::with_capacity((w * h * 4) as usize);
        for _ in 0..(w * h) {
            data.extend_from_slice(&rgba);
        }
        DecodedImage::new(w, h, data).unwrap()
    }

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

    #[test]
    fn encode_then_decode_round_trips_pixels_exactly() {
        // PNG is lossless, so a real round trip must reproduce every byte,
        // not merely something visually close.
        let img = solid(6, 5, [12, 200, 44, 255]);
        let bytes = encode_png(&img).unwrap();
        let back = decode_png(&bytes).unwrap();
        assert_eq!(back, img);
    }

    #[test]
    fn decode_png_rejects_non_png_bytes() {
        assert!(decode_png(b"not a png").is_err());
    }

    #[test]
    fn item_hash_is_deterministic_for_identical_pixels() {
        let a = solid(40, 40, [10, 20, 30, 255]);
        let b = solid(40, 40, [10, 20, 30, 255]);
        assert_eq!(item_hash(&a), item_hash(&b));
    }

    #[test]
    fn item_hash_differs_for_different_pixels() {
        let a = solid(40, 40, [10, 20, 30, 255]);
        let b = solid(40, 40, [200, 20, 30, 255]);
        assert_ne!(item_hash(&a), item_hash(&b));
    }

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

    #[test]
    fn render_item_produces_a_size_by_size_canvas() {
        let src = solid(10, 20, [255, 0, 0, 255]);
        let t = ItemTransform::default();
        let out = render_item(&src, &t, 64, false);
        assert_eq!((out.w, out.h), (64, 64));
        assert_eq!(out.rgba.len(), 64 * 64 * 4);
    }

    #[test]
    fn render_item_opaque_fills_the_backdrop_black() {
        let src = solid(4, 4, [255, 255, 255, 255]);
        let t = ItemTransform { scale: 0.1, pan_x: 0.0, pan_y: 0.0 };
        let out = render_item(&src, &t, 32, true);
        // Far corner is outside the tiny scaled sprite -> the opaque backdrop.
        assert_eq!(&out.rgba[0..4], &[0, 0, 0, 255]);
    }

    #[test]
    fn render_item_transparent_leaves_the_backdrop_empty() {
        let src = solid(4, 4, [255, 255, 255, 255]);
        let t = ItemTransform { scale: 0.1, pan_x: 0.0, pan_y: 0.0 };
        let out = render_item(&src, &t, 32, false);
        assert_eq!(&out.rgba[0..4], &[0, 0, 0, 0]);
    }

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
}
