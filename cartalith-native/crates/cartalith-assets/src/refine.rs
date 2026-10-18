//! AP-7 (`ASSET_PACK_TOOLS_PROPOSAL.md` §6, Ruling CQ): the pixel half of
//! the REFINE verbs -- trim, pad, flip, tint jitter, outline, shadow and
//! harmonise-to-Look. Each is a pure function from one image to a new one;
//! **none ever writes its input**, which is what lets the caller store the
//! result as a new variant beside an untouched original (Ruling CQ Q7).
//! *Fit all* is the eighth verb and is transform-only, so it lives with the
//! session's item records (`cartalith-godot`'s `AssetLibrarySession::fit_all`),
//! not here.
//!
//! **Which pixels each verb takes** is the caller's choice and is documented
//! there; this module works on whatever it is given. Two kinds:
//!
//! - geometric and per-pixel colour verbs (trim, pad, flip, tint, harmonise)
//!   lose nothing at any size, so they can run on the source at full
//!   resolution;
//! - outline and shadow have a width in pixels, and only mean something at
//!   the size the art ships at, so they run on the bake.
//!
//! The colour verbs (tint, harmonise) **never change alpha** (byte-equal
//! alpha channel, AP-7's bar), and leave the RGB of fully transparent pixels
//! as it was. Every constant below is a labelled judgement unless it says
//! otherwise; none is a measurement.

use crate::raster::DecodedImage;
use crate::slicer::{BLANK_ALPHA_THRESHOLD, trim_transparent_edges};

/// The margin *Pad* adds on every side, as a fraction of the image's longer
/// side. Labelled judgement: one tenth gives a sprite that touched its
/// canvas edge a visible border at map size, without shrinking it past
/// recognition (the art ends up at 1/1.2 = 83 % of the cell).
pub const PAD_FRACTION: f64 = 0.1;

/// *Outline*'s width in pixels at the 256² sprite bake. Labelled judgement:
/// the largest marker glyph box is about 42 screen px (AP-3's measured
/// range, 10 to 42 px), where 6 px of 256 is one screen pixel; at the
/// smaller sizes it fades into the art's own edge rather than swamping it.
pub const OUTLINE_PX: usize = 6;

/// *Outline*'s ink: a near-black, not pure black, so it reads as an ink line
/// on dark terrain too. Labelled judgement.
pub const OUTLINE_RGB: [u8; 3] = [24, 22, 20];

/// *Shadow*'s offset (right, down) in pixels at the 256² bake: light from
/// the upper left, the convention of the map's own hillshade azimuth
/// (`sun_az_deg`, north-west). Labelled judgement for the length.
pub const SHADOW_OFFSET: (i64, i64) = (6, 6);

/// *Shadow*'s blur: the radius of each of three box-blur passes (three
/// passes of a box approximate a Gaussian; W. Wells, *Efficient synthesis of
/// Gaussian filters by cascaded uniform filters*, IEEE PAMI 8(2), 1986).
/// Labelled judgement for the radius.
pub const SHADOW_BLUR_PX: usize = 4;

/// *Shadow*'s opacity at full coverage. Labelled judgement.
pub const SHADOW_OPACITY: f64 = 0.45;

/// *Tint jitter*'s ranges: hue ± degrees, saturation and value × (1 ± this).
/// Labelled judgement: "slightly varied mountains" (proposal rank 9), small
/// enough that the variant reads as the same drawing, and inside AP-6's
/// ±10 % near-duplicate band on value.
pub const JITTER_HUE_DEG: f64 = 12.0;
/// See [`JITTER_HUE_DEG`].
pub const JITTER_SAT: f64 = 0.10;
/// See [`JITTER_HUE_DEG`].
pub const JITTER_VAL: f64 = 0.08;

/// *Harmonise*: how far the art's mean saturation moves toward the target
/// swatch's (0 = not at all, 1 = all the way). Labelled judgement: halfway
/// keeps a vivid icon readable on a muted map. The hue moves all the way;
/// value is left alone so line work keeps its contrast.
pub const HARMONISE_SAT_PULL: f64 = 0.5;

/// *Harmonise* refuses art whose alpha-weighted mean chroma is below this
/// (0..1 of full scale): a grey drawing has no hue to move, and a hue read
/// off near-grey pixels is noise. Also the floor below which a palette
/// swatch counts as grey and is not a target. Labelled judgement.
pub const MIN_CHROMA: f64 = 0.04;

// ---------------------------------------------------------------------------
// HSV, in f64 on 0..1 channels (hue in degrees)
// ---------------------------------------------------------------------------

/// RGB (0..255) to (hue degrees 0..360, saturation 0..1, value 0..1).
fn rgb_to_hsv(r: u8, g: u8, b: u8) -> (f64, f64, f64) {
    let (r, g, b) = (f64::from(r) / 255.0, f64::from(g) / 255.0, f64::from(b) / 255.0);
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let c = max - min;
    let h = if c == 0.0 {
        0.0
    } else if max == r {
        60.0 * ((g - b) / c).rem_euclid(6.0)
    } else if max == g {
        60.0 * ((b - r) / c + 2.0)
    } else {
        60.0 * ((r - g) / c + 4.0)
    };
    let s = if max == 0.0 { 0.0 } else { c / max };
    (h, s, max)
}

/// The inverse of [`rgb_to_hsv`], rounded to the nearest level.
fn hsv_to_rgb(h: f64, s: f64, v: f64) -> [u8; 3] {
    let h = h.rem_euclid(360.0);
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;
    let (r, g, b) = match (h / 60.0) as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let q = |t: f64| ((t + m) * 255.0).round().clamp(0.0, 255.0) as u8;
    [q(r), q(g), q(b)]
}

/// Smallest signed difference `to - from` between two hues, in degrees
/// (-180..180].
fn hue_delta(from: f64, to: f64) -> f64 {
    let d = (to - from).rem_euclid(360.0);
    if d > 180.0 { d - 360.0 } else { d }
}

/// Apply `f(h, s, v) -> (h, s, v)` to every pixel with alpha above 0,
/// leaving alpha (every pixel) and the RGB of fully transparent pixels as
/// they were.
fn map_hsv(img: &DecodedImage, f: impl Fn(f64, f64, f64) -> (f64, f64, f64)) -> DecodedImage {
    let mut out = img.clone();
    for p in out.rgba.chunks_exact_mut(4) {
        if p[3] == 0 {
            continue;
        }
        let (h, s, v) = rgb_to_hsv(p[0], p[1], p[2]);
        let (h, s, v) = f(h, s, v);
        let rgb = hsv_to_rgb(h, s.clamp(0.0, 1.0), v.clamp(0.0, 1.0));
        p[..3].copy_from_slice(&rgb);
    }
    out
}

/// The art's mean hue in degrees: the circular mean of every pixel's hue
/// weighted by alpha × chroma (so transparent and grey pixels count for
/// nothing), with its mean saturation (alpha-weighted), or `None` when the
/// alpha-weighted mean chroma is below [`MIN_CHROMA`] -- grey art has no hue
/// to report, and the caller must say so rather than show a number.
pub fn mean_hue(img: &DecodedImage) -> Option<(f64, f64)> {
    let (mut sx, mut sy, mut chroma, mut alpha, mut sat) = (0.0, 0.0, 0.0, 0.0, 0.0);
    for p in img.rgba.chunks_exact(4) {
        let a = f64::from(p[3]) / 255.0;
        if a == 0.0 {
            continue;
        }
        let (h, s, v) = rgb_to_hsv(p[0], p[1], p[2]);
        let c = s * v;
        let w = a * c;
        sx += w * h.to_radians().cos();
        sy += w * h.to_radians().sin();
        chroma += w;
        alpha += a;
        sat += a * s;
    }
    if alpha == 0.0 || chroma / alpha < MIN_CHROMA {
        return None;
    }
    Some((sy.atan2(sx).to_degrees().rem_euclid(360.0), sat / alpha))
}

// ---------------------------------------------------------------------------
// The verbs
// ---------------------------------------------------------------------------

/// *Trim*: crop to the box of pixels above the slicer's blank threshold
/// ([`trim_transparent_edges`], the same rule the slicer's "trim" uses).
/// `None` when there is nothing to trim -- the box is already the whole
/// image, or the image is blank -- so the caller writes no variant.
pub fn trim(img: &DecodedImage) -> Option<DecodedImage> {
    let out = trim_transparent_edges(img);
    (out.w != img.w || out.h != img.h).then_some(out)
}

/// *Pad*: the image centred on a transparent canvas [`PAD_FRACTION`] of its
/// longer side larger on every side. Lossless: the art's own pixels are
/// copied, not resampled.
pub fn pad(img: &DecodedImage) -> DecodedImage {
    let m = (f64::from(img.w.max(img.h)) * PAD_FRACTION).ceil() as usize;
    let (w, h) = (img.w as usize, img.h as usize);
    let (nw, nh) = (w + 2 * m, h + 2 * m);
    let mut out = vec![0u8; nw * nh * 4];
    for y in 0..h {
        let (s, d) = (y * w * 4, ((y + m) * nw + m) * 4);
        out[d..d + w * 4].copy_from_slice(&img.rgba[s..s + w * 4]);
    }
    DecodedImage::new(nw as u32, nh as u32, out).expect("sized above")
}

/// *Flip*: the left-right mirror. Lossless and its own inverse.
pub fn flip_horizontal(img: &DecodedImage) -> DecodedImage {
    let (w, h) = (img.w as usize, img.h as usize);
    let mut out = vec![0u8; img.rgba.len()];
    for y in 0..h {
        for x in 0..w {
            let (s, d) = ((y * w + x) * 4, (y * w + (w - 1 - x)) * 4);
            out[d..d + 4].copy_from_slice(&img.rgba[s..s + 4]);
        }
    }
    DecodedImage::new(img.w, img.h, out).expect("same dimensions")
}

/// The three numbers one *tint jitter* applied.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Jitter {
    /// Hue rotation, degrees, within ±[`JITTER_HUE_DEG`].
    pub hue_deg: f64,
    /// Saturation factor, within 1 ± [`JITTER_SAT`].
    pub sat: f64,
    /// Value factor, within 1 ± [`JITTER_VAL`].
    pub val: f64,
}

impl Jitter {
    /// The jitter for `seed`: three draws from a SplitMix64 stream (S. Vigna,
    /// public domain; the standard seeding generator), each mapped to its
    /// range. Deterministic, so the same seed always makes the same variant.
    pub fn from_seed(seed: u64) -> Jitter {
        let mut z = seed;
        let mut next = || {
            z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
            let mut x = z;
            x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
            x ^= x >> 31;
            // Top 53 bits to [0, 1), then to [-1, 1).
            (x >> 11) as f64 / (1u64 << 53) as f64 * 2.0 - 1.0
        };
        Jitter { hue_deg: next() * JITTER_HUE_DEG, sat: 1.0 + next() * JITTER_SAT, val: 1.0 + next() * JITTER_VAL }
    }
}

/// *Tint jitter*: rotate hue and scale saturation and value by `j`. Alpha is
/// byte-identical to the input's.
pub fn tint_jitter(img: &DecodedImage, j: Jitter) -> DecodedImage {
    map_hsv(img, |h, s, v| (h + j.hue_deg, s * j.sat, v * j.val))
}

/// Straight-alpha "source over destination" for one pixel, both RGBA8.
fn over(src: [u8; 4], dst: [u8; 4]) -> [u8; 4] {
    let sa = f64::from(src[3]) / 255.0;
    let da = f64::from(dst[3]) / 255.0;
    let oa = sa + da * (1.0 - sa);
    if oa == 0.0 {
        return [0, 0, 0, 0];
    }
    let mut out = [0u8; 4];
    for c in 0..3 {
        let v = (f64::from(src[c]) * sa + f64::from(dst[c]) * da * (1.0 - sa)) / oa;
        out[c] = v.round().clamp(0.0, 255.0) as u8;
    }
    out[3] = (oa * 255.0).round().clamp(0.0, 255.0) as u8;
    out
}

/// Composite `art` over a one-colour layer whose alpha is `layer_alpha`
/// (0..1 per pixel). Where the art is fully opaque it is unchanged.
fn art_over_layer(art: &DecodedImage, layer_alpha: &[f64], rgb: [u8; 3]) -> DecodedImage {
    let mut out = art.clone();
    for (i, p) in out.rgba.chunks_exact_mut(4).enumerate() {
        let a = (layer_alpha[i].clamp(0.0, 1.0) * 255.0).round() as u8;
        let o = over([p[0], p[1], p[2], p[3]], [rgb[0], rgb[1], rgb[2], a]);
        p.copy_from_slice(&o);
    }
    out
}

/// *Outline*: an [`OUTLINE_RGB`] line [`OUTLINE_PX`] wide around the art,
/// drawn **behind** it: the layer's alpha at a pixel is the largest art alpha
/// within a disc of that radius (a grey-scale dilation), so soft edges keep
/// a soft outline. Same size as the input; art reaching the canvas edge has
/// its outline clipped there (Pad first to avoid that).
pub fn outline(img: &DecodedImage) -> DecodedImage {
    let (w, h) = (img.w as usize, img.h as usize);
    let r = OUTLINE_PX as i64;
    let alpha = |x: usize, y: usize| f64::from(img.rgba[(y * w + x) * 4 + 3]) / 255.0;
    // The disc's offsets, once.
    let disc: Vec<(i64, i64)> = (-r..=r).flat_map(|dy| (-r..=r).map(move |dx| (dx, dy))).filter(|(dx, dy)| dx * dx + dy * dy <= r * r).collect();
    let mut layer = vec![0.0f64; w * h];
    for y in 0..h {
        for x in 0..w {
            let mut m = 0.0f64;
            for &(dx, dy) in &disc {
                let (sx, sy) = (x as i64 + dx, y as i64 + dy);
                if sx >= 0 && sy >= 0 && (sx as usize) < w && (sy as usize) < h {
                    m = m.max(alpha(sx as usize, sy as usize));
                    if m >= 1.0 {
                        break;
                    }
                }
            }
            layer[y * w + x] = m;
        }
    }
    art_over_layer(img, &layer, OUTLINE_RGB)
}

/// One horizontal and one vertical box-blur pass of radius `r` over a
/// `w × h` field, with the field treated as zero outside.
fn box_blur(f: &[f64], w: usize, h: usize, r: usize) -> Vec<f64> {
    let n = (2 * r + 1) as f64;
    let mut tmp = vec![0.0; w * h];
    for y in 0..h {
        for x in 0..w {
            let (a, b) = (x.saturating_sub(r), (x + r).min(w - 1));
            tmp[y * w + x] = (a..=b).map(|i| f[y * w + i]).sum::<f64>() / n;
        }
    }
    let mut out = vec![0.0; w * h];
    for y in 0..h {
        for x in 0..w {
            let (a, b) = (y.saturating_sub(r), (y + r).min(h - 1));
            out[y * w + x] = (a..=b).map(|j| tmp[j * w + x]).sum::<f64>() / n;
        }
    }
    out
}

/// *Shadow*: a black drop shadow behind the art -- its alpha moved by
/// [`SHADOW_OFFSET`], blurred by three box passes of [`SHADOW_BLUR_PX`], at
/// [`SHADOW_OPACITY`]. Same size as the input; a shadow running off the
/// canvas is clipped there.
pub fn drop_shadow(img: &DecodedImage) -> DecodedImage {
    let (w, h) = (img.w as usize, img.h as usize);
    let (ox, oy) = SHADOW_OFFSET;
    let mut f = vec![0.0f64; w * h];
    for y in 0..h {
        for x in 0..w {
            let (sx, sy) = (x as i64 - ox, y as i64 - oy);
            if sx >= 0 && sy >= 0 && (sx as usize) < w && (sy as usize) < h {
                f[y * w + x] = f64::from(img.rgba[(sy as usize * w + sx as usize) * 4 + 3]) / 255.0;
            }
        }
    }
    for _ in 0..3 {
        f = box_blur(&f, w, h, SHADOW_BLUR_PX);
    }
    for v in &mut f {
        *v *= SHADOW_OPACITY;
    }
    art_over_layer(img, &f, [0, 0, 0])
}

/// What *Harmonise* measured and did.
#[derive(Debug, Clone, PartialEq)]
pub struct Harmonised {
    /// The input's mean hue (degrees), per [`mean_hue`].
    pub hue_before: f64,
    /// The target swatch's hue (degrees).
    pub target_hue: f64,
    /// The target swatch itself.
    pub target_rgb: [u8; 3],
    /// The result's mean hue (degrees), measured on the result.
    pub hue_after: f64,
    /// The harmonised image; alpha byte-identical to the input's.
    pub image: DecodedImage,
}

/// Why *Harmonise* did nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HarmoniseRefusal {
    /// The art's mean chroma is below [`MIN_CHROMA`]: it is grey.
    NoColour,
    /// No palette swatch has chroma at or above [`MIN_CHROMA`].
    GreyPalette,
}

impl HarmoniseRefusal {
    /// The reason in words, for the shell.
    pub fn text(self) -> &'static str {
        match self {
            HarmoniseRefusal::NoColour => "this art is grey: it has no hue to move toward the Look",
            HarmoniseRefusal::GreyPalette => "the Look's palette has no coloured swatch to move toward",
        }
    }
}

/// *Harmonise to the Look*: find the palette swatch whose hue is nearest the
/// art's mean hue (grey swatches skipped), rotate every pixel's hue by the
/// difference, and move the mean saturation [`HARMONISE_SAT_PULL`] of the
/// way toward the swatch's. Value and alpha are untouched (alpha
/// byte-identical). The rotation is uniform, so the result's mean hue lands
/// on the swatch's up to 8-bit rounding (AP-7's bar: within 10°).
pub fn harmonise(img: &DecodedImage, palette: &[[u8; 3]]) -> Result<Harmonised, HarmoniseRefusal> {
    let (hue, sat) = mean_hue(img).ok_or(HarmoniseRefusal::NoColour)?;
    let target = palette
        .iter()
        .filter_map(|&rgb| {
            let (h, s, v) = rgb_to_hsv(rgb[0], rgb[1], rgb[2]);
            (s * v >= MIN_CHROMA).then_some((rgb, h, s))
        })
        .min_by(|a, b| hue_delta(hue, a.1).abs().total_cmp(&hue_delta(hue, b.1).abs()))
        .ok_or(HarmoniseRefusal::GreyPalette)?;
    let (rgb, th, ts) = target;
    let rot = hue_delta(hue, th);
    let k = if sat > 0.0 { ((sat + HARMONISE_SAT_PULL * (ts - sat)) / sat).clamp(0.5, 2.0) } else { 1.0 };
    let image = map_hsv(img, |h, s, v| (h + rot, s * k, v));
    let hue_after = mean_hue(&image).map(|(h, _)| h).unwrap_or(th);
    Ok(Harmonised { hue_before: hue, target_hue: th, target_rgb: rgb, hue_after, image })
}

/// Whether any pixel of `img` is above the slicer's blank threshold -- the
/// refine verbs that draw around art (outline, shadow) have nothing to draw
/// around otherwise.
pub fn has_ink(img: &DecodedImage) -> bool {
    img.rgba.chunks_exact(4).any(|p| p[3] > BLANK_ALPHA_THRESHOLD)
}

/// AP-7's pixel bars, verb by verb.
#[cfg(test)]
mod tests {
    use super::*;

    fn img(w: u32, h: u32, f: impl Fn(u32, u32) -> [u8; 4]) -> DecodedImage {
        let mut v = Vec::with_capacity((w * h * 4) as usize);
        for y in 0..h {
            for x in 0..w {
                v.extend_from_slice(&f(x, y));
            }
        }
        DecodedImage::new(w, h, v).unwrap()
    }

    /// A 256² sprite: a red-orange disc, shaded left to right, with a soft
    /// (anti-aliased) edge, on transparency.
    fn disc() -> DecodedImage {
        img(256, 256, |x, y| {
            let d = ((x as f64 - 128.0).powi(2) + (y as f64 - 140.0).powi(2)).sqrt();
            let a = (90.0 - d).clamp(0.0, 1.0);
            [(150 + x / 3) as u8, (60 + x / 6) as u8, 30, (a * 255.0).round() as u8]
        })
    }

    fn alpha(img: &DecodedImage) -> Vec<u8> {
        img.rgba.chunks_exact(4).map(|p| p[3]).collect()
    }

    /// Protects: HSV round trip is exact on 8-bit input (every grey and a
    /// sweep of colours), so the colour verbs at identity change nothing.
    #[test]
    fn hsv_round_trip_is_exact() {
        for r in (0..=255).step_by(5) {
            for g in (0..=255).step_by(15) {
                for b in (0..=255).step_by(15) {
                    let (h, s, v) = rgb_to_hsv(r as u8, g as u8, b as u8);
                    assert_eq!(hsv_to_rgb(h, s, v), [r as u8, g as u8, b as u8]);
                }
            }
        }
        let d = disc();
        assert_eq!(tint_jitter(&d, Jitter { hue_deg: 0.0, sat: 1.0, val: 1.0 }), d);
    }

    /// Protects: trim crops exactly to the inked box and writes nothing when
    /// there is nothing to trim; pad adds the stated margin losslessly.
    #[test]
    fn trim_and_pad() {
        let d = disc();
        let t = trim(&d).unwrap();
        assert!(t.w < 256 && t.h < 256);
        assert!(trim(&t).is_none(), "a trimmed image has nothing left to trim");
        assert!(trim(&img(4, 4, |_, _| [0, 0, 0, 0])).is_none(), "blank: nothing to trim");
        let p = pad(&t);
        let m = (f64::from(t.w.max(t.h)) * 0.1).ceil() as u32;
        assert_eq!((p.w, p.h), (t.w + 2 * m, t.h + 2 * m));
        assert_eq!(trim(&p).unwrap(), t, "pad then trim is the identity");
    }

    /// Protects: flip mirrors and is its own inverse.
    #[test]
    fn flip_is_a_mirror() {
        let d = disc();
        let f = flip_horizontal(&d);
        assert_ne!(f, d);
        assert_eq!(&f.rgba[0..4], &d.rgba[255 * 4..256 * 4]);
        assert_eq!(flip_horizontal(&f), d);
    }

    /// Protects: tint jitter is deterministic per seed, stays in its ranges,
    /// differs between seeds, and leaves alpha byte-equal.
    #[test]
    fn tint_jitter_is_seeded_bounded_and_keeps_alpha() {
        let (a, b) = (Jitter::from_seed(7), Jitter::from_seed(8));
        assert_eq!(a, Jitter::from_seed(7));
        assert_ne!(a, b);
        for s in 0..200 {
            let j = Jitter::from_seed(s);
            assert!(j.hue_deg.abs() <= JITTER_HUE_DEG && (j.sat - 1.0).abs() <= JITTER_SAT && (j.val - 1.0).abs() <= JITTER_VAL);
        }
        let mut d = disc();
        // A fully transparent pixel with colour data: the verbs leave it be.
        d.rgba[0..4].copy_from_slice(&[10, 200, 30, 0]);
        let t = tint_jitter(&d, a);
        assert_ne!(t.rgba, d.rgba);
        assert_eq!(alpha(&t), alpha(&d));
        assert_eq!(&t.rgba[0..4], &[10, 200, 30, 0], "RGB under alpha 0 untouched");
    }

    /// Protects: outline puts ink only where the art was not (fully opaque
    /// art pixels unchanged), reaches OUTLINE_PX beyond the edge and no
    /// further.
    #[test]
    fn outline_is_behind_the_art_and_six_px_wide() {
        let sq = img(64, 64, |x, y| if (20..44).contains(&x) && (20..44).contains(&y) { [200, 10, 10, 255] } else { [0, 0, 0, 0] });
        let o = outline(&sq);
        let at = |im: &DecodedImage, x: usize, y: usize| -> [u8; 4] { im.rgba[(y * 64 + x) * 4..(y * 64 + x) * 4 + 4].try_into().unwrap() };
        assert_eq!(at(&o, 30, 30), [200, 10, 10, 255], "art over the outline is unchanged");
        // Literal 6 (the documented width), not OUTLINE_PX: a test against
        // the constant itself would pass for any width.
        assert_eq!(at(&o, 14, 30), [24, 22, 20, 255], "6 px out: ink");
        assert_eq!(at(&o, 13, 30)[3], 0, "7 px out: clear");
    }

    /// Protects: the shadow falls to the lower right (SHADOW_OFFSET), is
    /// translucent, and does not touch fully opaque art.
    #[test]
    fn shadow_falls_down_right_and_stays_behind() {
        let sq = img(64, 64, |x, y| if (16..40).contains(&x) && (16..40).contains(&y) { [10, 200, 10, 255] } else { [0, 0, 0, 0] });
        let s = drop_shadow(&sq);
        let a = |x: usize, y: usize| s.rgba[(y * 64 + x) * 4 + 3];
        assert_eq!(&s.rgba[(20 * 64 + 20) * 4..(20 * 64 + 20) * 4 + 4], &[10, 200, 10, 255]);
        assert!(a(42, 42) > 0, "below-right of the art: shadow");
        assert!(a(42, 42) < 255);
        assert_eq!(a(10, 10), 0, "above-left: none");
        let max = s.rgba.chunks_exact(4).filter(|p| p[1] < 100).map(|p| p[3]).max().unwrap();
        assert!(f64::from(max) <= SHADOW_OPACITY * 255.0 + 1.0, "never darker than SHADOW_OPACITY: {max}");
    }

    /// Protects: AP-7's harmonise bar. The result's mean hue is within 10° of
    /// the nearest coloured palette swatch, the alpha channel is byte-equal,
    /// grey swatches are not targets, and grey art is refused in words.
    #[test]
    fn harmonise_bar_hue_within_ten_degrees_alpha_untouched() {
        let d = disc();
        let (hue0, _) = mean_hue(&d).unwrap();
        // A green, a blue, a grey and a gold: the gold is nearest the disc's
        // orange; the grey would be "nearest" by RGB distance but has no hue.
        let palette = [[60, 140, 50], [40, 80, 200], [128, 128, 128], [222, 172, 78]];
        let h = harmonise(&d, &palette).unwrap();
        assert_eq!(h.target_rgb, [222, 172, 78]);
        assert!(hue_delta(h.hue_after, h.target_hue).abs() <= 10.0, "{h:?}");
        assert!((h.hue_before - hue0).abs() < 1e-9);
        assert_eq!(alpha(&h.image), alpha(&d), "alpha byte-equal");
        // Saturation moves HARMONISE_SAT_PULL (half) of the way to the
        // swatch's: the disc is more saturated than the gold, so it drops,
        // and lands near the midpoint.
        let (_, s0) = mean_hue(&d).unwrap();
        let (_, s1) = mean_hue(&h.image).unwrap();
        let (_, ts, _) = rgb_to_hsv(222, 172, 78);
        assert!(s0 > ts && s1 < s0, "{s0} -> {s1} toward {ts}");
        assert!((s1 - (s0 + ts) / 2.0).abs() < 0.03, "{s0} -> {s1}, midpoint {}", (s0 + ts) / 2.0);
        let grey = img(16, 16, |_, _| [90, 90, 90, 255]);
        assert_eq!(harmonise(&grey, &palette).unwrap_err(), HarmoniseRefusal::NoColour);
        assert_eq!(harmonise(&d, &[[128, 128, 128]]).unwrap_err(), HarmoniseRefusal::GreyPalette);
        // A far target: the bar holds for a 120° move too.
        let far = harmonise(&d, &[[40, 80, 200]]).unwrap();
        assert!(hue_delta(far.hue_after, far.target_hue).abs() <= 10.0, "{:?}", (far.hue_before, far.target_hue, far.hue_after));
        assert_eq!(alpha(&far.image), alpha(&d));
    }

    /// Protects: the circular mean straddles 0°/360° correctly (a naive
    /// arithmetic mean of 350° and 10° would say 180°).
    #[test]
    fn mean_hue_wraps() {
        let two = img(2, 1, |x, _| if x == 0 { [255, 0, 43, 255] } else { [255, 43, 0, 255] });
        let (h, _) = mean_hue(&two).unwrap();
        assert!(hue_delta(h, 0.0).abs() < 1.0, "{h}");
    }
}
