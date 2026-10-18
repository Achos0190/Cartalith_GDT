//! AP-10 (Ruling CR, 2026-10-18): line art to a clean, solid sprite.
//!
//! **What it is for.** The owner's complaint about the HTML app's line art:
//! *"the lines weren't solid enough and often it became a blobby mess."*
//! `LINEART_METHODS_RESEARCH_2026-10-18.md` read the causes at the symbols
//! (a single-colour chroma key that leaves darker paper and every stroke's
//! light edge opaque; a straight-alpha CatmullRom bake; a point-sampled blit;
//! nearest-neighbour zoom) and measured a replacement on four synthetic
//! samples. `ASSET_PACK_TOOLS_PROPOSAL.md` §9 turned that into this
//! milestone, and Ruling CR approved it at the recommended defaults. This
//! module is that pipeline, in the order the ruling names:
//!
//! 1. **Ink extraction at full source resolution** ([`clean_ink`]): flatten
//!    the paper (luminance divided by a max-then-mean filter whose window is
//!    [`PAPER_WINDOW_FRACTION`] of the sprite), threshold with Otsu (1979),
//!    remove specks, close 1 px gaps **on scans only** ([`GapClosing`]),
//!    fill pinholes.
//! 2. **A signed distance field** of the cleaned mask ([`SdfPyramid`]): an
//!    exact Euclidean distance transform (Felzenszwalb & Huttenlocher 2012,
//!    written from the paper), one L8 channel per level, the top level sized
//!    so the thinnest stroke spans at least [`MIN_TEXELS_ACROSS_STROKE`]
//!    texels, every level sampled from the full-resolution field (built from
//!    the mask, not averaged from the level above: research Finding 5).
//! 3. **Rendering with a 1 px edge ramp and a stroke floor**
//!    ([`render_coverage`]): `alpha = clamp(d_out + δ + 0.5)`, where `d_out`
//!    is the signed distance in output pixels and
//!    `δ = max(0, (w_min − w_p2) / 2)` widens every stroke so its
//!    2nd-percentile width reaches `w_min` output pixels (Ruling CR Q-L1:
//!    "solid" at map scale, `w_min` the per-variant line weight, 0.5–1.5,
//!    default 1.0).
//! 4. **One ink colour** ([`LineArtOptions::ink`]), picked by the caller from
//!    the Look's palette (Ruling CQ Q4): RGB is that colour wherever alpha is
//!    non-zero, so no paper RGB survives and the halo is zero by construction.
//!
//! [`convert_line_art`] runs all four and returns the family-size coverage
//! bake (Ruling F: the pack stays 256² PNG). The caller stores it as a **new
//! variant** (Ruling CQ Q7); this module never sees, and so can never
//! overwrite, the original.
//!
//! **What it must never do.**
//! - Link or port potrace (GPL-2.0+; Ruling CR). Nothing here traces curves.
//! - Use randomness, timing, threads or hash-map iteration order: bar (h) is
//!   byte-identical output for the same input, pinned by a golden hash per
//!   fixture in `tests/lineart_ap10.rs`. Every step is integer morphology,
//!   an exact EDT or a fixed f64 quantiser, run in one fixed order.
//! - Persist the SDF. Ruling CR Q-L3: it is a derived cache rebuilt from the
//!   original, so it adds no save key and no pack key.
//! - Touch any map pixel by itself. Only a user pressing the REFINE verb
//!   calls it, and what it writes is a new library variant.
//!
//! **Provenance of the code.** Every algorithm here was written for this
//! crate from the cited papers and from the research lab's Python
//! (`cartalith-native/docs/evidence/lineart-2026-10-18/lineart_lab.py`,
//! this project's own), not copied from PhotoCraft; so no third-party notice
//! applies. The lab's functions are named beside each port so an auditor can
//! compare them line by line.

use crate::library::ItemTransform;
use crate::raster::{DecodedImage, render_item};

// ---------------------------------------------------------------------------
// Constants (each names its source)
// ---------------------------------------------------------------------------

/// The paper estimate's window, as a fraction of the sprite's side. The lab's
/// `normalise_paper` (`int(0.15 * L.shape[0]) | 1`). **Measured, research
/// Finding 1:** a 25 px window read the castle's 123 px solid door as paper
/// and hollowed it out (IoU 0.807); 15 % fixed it (1.000). It must be wider
/// than the largest solid ink area; illumination is low-frequency, so wide is
/// safe.
pub const PAPER_WINDOW_FRACTION: f64 = 0.15;

/// The window, as a fraction of the sprite's side, that separates a stroke
/// from a broad dark area in [`ink_by_two_scales`]: 21 px at 1024², 5 px at
/// 256². **Labelled judgement:** wider than every stroke in the five fixtures
/// (the widest is the castle's 8 px outline) and than a bold 2 % brush line,
/// narrower than the crease shadow it was added for (about 20 px at half
/// depth) and the castle's 123 px door. A stroke wider than this still
/// survives, through the wide path's sharp-edge test.
pub const STROKE_WINDOW_FRACTION: f64 = 0.02;

/// The mean boundary gradient (flattened luminance per pixel, central
/// difference) a broad dark area must reach to count as ink rather than
/// shading in [`ink_by_two_scales`]. **Labelled judgement, calibrated
/// 2026-10-19:** the castle's solid door measures 0.40 on the clean sample
/// and 0.20 on the bad scan; a 50 %-deep Gaussian shadow of σ 30 px at 1024²
/// measures 0.011 and the test's sharp block 0.50
/// (`a_broad_soft_shadow_is_not_ink`). The pencil scan's fold
/// never reaches this test: the stroke-scale closing already reads it as
/// paper.
pub const SHARP_EDGE_MIN: f64 = 0.08;

/// The flattened luminance is clipped to `[0, FLAT_CLIP]`, and Otsu's
/// histogram spans the same range. The lab's `np.clip(L / B, 0, 1.2)` and
/// `np.histogram(L, 256, (0, 1.2))`. Above 1 only where a pixel is brighter
/// than its local paper estimate (grain); 1.2 keeps that tail in the
/// histogram instead of piling it into the last bin.
pub const FLAT_CLIP: f64 = 1.2;

/// Otsu's histogram bin count. The lab's 256: one bin per 8-bit level of the
/// unflattened input.
pub const OTSU_BINS: usize = 256;

/// No pixel whose flattened luminance is at or above this is ink, whatever
/// Otsu says. The lab's `m &= Ln < 0.9` ("never call clean paper ink"): on a
/// page with almost no ink Otsu splits the paper's own grain, and this guard
/// keeps that split out of the mask. Labelled judgement from the lab.
pub const PAPER_CEILING: f64 = 0.9;

/// Ink components smaller than this many pixels are removed (dust), at a
/// 1024² source. The lab's `speck=24`; measured on the scan: 577 fragments
/// against 7 for today's key, 7 against 7 with this. Scaled by area for
/// other sizes ([`area_scaled`]).
pub const SPECK_PX_AT_1024: f64 = 24.0;

/// Enclosed paper regions (4-connected) smaller than this many pixels are
/// filled (pinholes left by ink starvation and grain), at 1024². The lab's
/// `pin=12`. Scaled by area like [`SPECK_PX_AT_1024`]. Small enough that the
/// castle's windows (about 1 000 px each at 1024²) stay open: castle holes
/// 14 of 14 in the research's table.
pub const PINHOLE_PX_AT_1024: f64 = 12.0;

/// The illumination range above which [`ScanEvidence`] calls an image a
/// scan: `(p99 − p1) / p99` of the paper estimate. **Labelled judgement,
/// calibrated on the fixtures** (`tests/lineart_ap10.rs`,
/// `scan_detection_separates_the_fixtures`): the clean and JPEG samples
/// measure 0.000, the bad scan 0.238 and the pencil scan 0.162 (2026-10-19),
/// so 0.05 sits far from both sides.
pub const SCAN_ILLUMINATION: f64 = 0.05;

/// The paper grain above which [`ScanEvidence`] calls an image a scan: the
/// median absolute deviation of the flattened luminance over paper pixels.
/// **Labelled judgement, calibrated on the same fixtures:** clean and JPEG
/// samples 0.0000, the bad scan 0.0087 and the pencil scan 0.0108
/// (2026-10-19).
pub const SCAN_GRAIN: f64 = 0.004;

/// The signed distance range one L8 level spans: 127 levels for this many
/// texels each side of the edge. The lab's `spread=8.0`. Wider than any
/// floor offset ever applied (δ ≤ 0.75 output px at ≤ 4 texels per px is
/// at most 3 texels), so the floor never runs off the end of the range.
pub const SDF_SPREAD_TEXELS: f64 = 8.0;

/// The thinnest stroke must span at least this many texels at the top SDF
/// level. **Measured, research Finding 4:** the mountain's 4 px hatching at
/// 1024² is 1 texel at 256² and broke at ×64 (15 components against 14), 2
/// texels at 512² and held (14 against 14).
pub const MIN_TEXELS_ACROSS_STROKE: f64 = 2.0;

/// The smallest top level the SDF is ever built at. The research's ×1 level
/// (128², 3.2 texels per output pixel at 40 px): every sprite gets at least
/// that, however coarse its strokes.
pub const SDF_TOP_MIN: u32 = 128;

/// The largest top level. The research's 512², which met every ×16 and ×64
/// bar on all four samples; bar (g) caps the stored SDF at 32 KB per sprite
/// and 512² measured 10.8–21.0 KB as PNG. A source whose strokes would ask
/// for more (a starved scan) is capped here: its thinnest strokes are then
/// carried by the floor, not the field.
pub const SDF_TOP_MAX: u32 = 512;

/// The smallest level kept below the top: the pyramid halves down to this.
/// 32² serves a sprite drawn at 8 px or less; nothing on the map is smaller.
pub const SDF_LEVEL_MIN: u32 = 32;

/// Read an SDF at no more than about this many texels per output pixel.
/// **Measured, research Finding 5:** the 512² level read at ×1 (12.8 per
/// pixel) aliased, with 4.6–8.6 px edges; the 128² level (3.2 per pixel)
/// gave 1.42–1.83 px.
pub const MAX_TEXELS_PER_PIXEL: f64 = 4.0;

/// The Gaussian applied to the mask before the 2× supersample the distance
/// field is measured on (the lab's `gaussian_filter(m, 0.8)` then
/// `zoom(.., 2) > 0.5`). It rounds the mask's pixel staircase into a contour
/// a half-pixel field can follow; σ 0.8 is the lab's labelled judgement.
pub const CONTOUR_SIGMA: f64 = 0.8;

/// The stroke-width percentile the floor lifts to the line weight. The lab's
/// `floor_delta` (`np.percentile(w, 2)`). **Measured, research Finding 6:**
/// the 10th percentile missed the castle's dashed line at ×4 (3 components
/// against 11); the 2nd caught it (7 against 11).
pub const FLOOR_PERCENTILE: f64 = 2.0;

/// Ruling CR Q-L1: the per-variant "line weight at map scale" control, in
/// output pixels.
pub const LINE_WEIGHT_MIN: f64 = 0.5;
/// See [`LINE_WEIGHT_MIN`].
pub const LINE_WEIGHT_MAX: f64 = 1.5;
/// Ruling CR Q-L1's default: every line at least 1 px at map scale.
pub const LINE_WEIGHT_DEFAULT: f64 = 1.0;

/// The working resolution is the source's own longer side, clamped to this
/// range ([`working_image`]). The floor (256) is the family bake size, so a
/// small source is never cleaned below the size it ships at; the cap (2048)
/// bounds the EDT's 2× supersample at 4096² (about 16.8 M f64 cells twice,
/// ~270 MB transient), a labelled judgement against the pack importer's own
/// 8 192 px per side image cap. Ruling CR's "at full source resolution"
/// holds for every source up to 2048.
pub const WORK_MIN: u32 = 256;
/// See [`WORK_MIN`].
pub const WORK_MAX: u32 = 2048;

// ---------------------------------------------------------------------------
// Options and reports
// ---------------------------------------------------------------------------

/// Whether [`clean_ink`] closes 1 px gaps. Ruling CR: *"scans-only 1 px gap
/// closing"*. Closing heals ink starvation on a scan but would bridge clean
/// art's close parallel lines (research Finding 3: r = 2 merged two hatch
/// lines; r = 1 is the largest kept), so it is off unless the image is a scan.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GapClosing {
    /// Close when [`ScanEvidence::is_scan`] says the image is a scan. The
    /// default.
    Auto,
    /// Never close (the user says this is clean digital art).
    Off,
    /// Always close by 1 px (the user says this is a scan).
    On,
}

/// What [`convert_line_art`] needs from its caller.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LineArtOptions {
    /// The one ink colour baked into every non-transparent pixel (sRGB, 8
    /// bit). Ruling CQ Q4: the caller picks it from the Look's palette.
    pub ink: [u8; 3],
    /// The minimum stroke width at the bake size, in output pixels; clamped
    /// to [`LINE_WEIGHT_MIN`]..=[`LINE_WEIGHT_MAX`].
    pub line_weight: f64,
    /// See [`GapClosing`].
    pub gap_closing: GapClosing,
}

impl Default for LineArtOptions {
    /// The research's ink (34, 28, 22), the default line weight, and
    /// automatic gap closing. A caller with a Look should always pass the
    /// Look's ink instead; this default exists for tests and for a caller
    /// with no Look loaded.
    fn default() -> Self {
        LineArtOptions { ink: [34, 28, 22], line_weight: LINE_WEIGHT_DEFAULT, gap_closing: GapClosing::Auto }
    }
}

/// The two measurements behind [`GapClosing::Auto`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScanEvidence {
    /// `(p99 − p1) / p99` of the paper estimate: how much the page's
    /// illumination falls off across the image. 0 for digital art.
    pub illumination: f64,
    /// The median absolute deviation of the flattened luminance over paper
    /// pixels: the paper's grain. 0 for digital art.
    pub grain: f64,
}

impl ScanEvidence {
    /// A scan by either measure ([`SCAN_ILLUMINATION`], [`SCAN_GRAIN`]).
    pub fn is_scan(&self) -> bool {
        self.illumination > SCAN_ILLUMINATION || self.grain > SCAN_GRAIN
    }
}

/// A binary raster, row-major. `true` is ink.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mask {
    pub w: usize,
    pub h: usize,
    pub on: Vec<bool>,
}

impl Mask {
    /// How many pixels are ink.
    pub fn count(&self) -> usize {
        self.on.iter().filter(|b| **b).count()
    }
}

/// What [`clean_ink`] found and did, for the inspector's readout and the
/// tests.
#[derive(Clone, Debug, PartialEq)]
pub struct CleanReport {
    /// Otsu's threshold on the flattened luminance (a bin's upper edge in
    /// `[0, FLAT_CLIP]`).
    pub threshold: f64,
    /// See [`ScanEvidence`].
    pub evidence: ScanEvidence,
    /// Whether the 1 px closing ran.
    pub closed: bool,
    /// Dark areas wider than a stroke that were dropped as shading (soft
    /// edges: a fold, a page curl's shadow), not ink. See
    /// [`SHARP_EDGE_MIN`].
    pub shading_rejected: usize,
    /// Ink components removed as specks.
    pub specks_removed: usize,
    /// Enclosed paper regions filled as pinholes.
    pub pinholes_filled: usize,
    /// 8-connected ink components left.
    pub components: usize,
    /// The share of the image that is ink after cleaning.
    pub ink_fraction: f64,
}

/// One level of the signed distance field: L8, row-major, `size × size`.
/// 128 is the edge, above is inside; one level is
/// `SDF_SPREAD_TEXELS / 127` texels.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sdf {
    pub size: u32,
    pub q: Vec<u8>,
}

/// Every level [`render_coverage`] may read, largest first, plus the stroke
/// statistic the floor needs.
#[derive(Clone, Debug, PartialEq)]
pub struct SdfPyramid {
    /// Largest first; each half the one before, down to [`SDF_LEVEL_MIN`].
    pub levels: Vec<Sdf>,
    /// The cleaned mask's side, in pixels.
    pub mask_size: u32,
    /// The [`FLOOR_PERCENTILE`]th-percentile stroke width of the cleaned
    /// mask, in mask pixels (`2·edt − 1` at the medial ridge, the lab's
    /// `ridge_widths`).
    pub stroke_p2: f64,
}

/// What [`convert_line_art`] made.
#[derive(Clone, Debug, PartialEq)]
pub struct LineArtResult {
    /// `out × out` RGBA: the ink colour wherever alpha > 0, black (0, 0, 0)
    /// under alpha 0, alpha the floored coverage.
    pub image: DecodedImage,
    pub clean: CleanReport,
    /// The top SDF level's side.
    pub sdf_top: u32,
    /// The floor's offset at the bake size, output pixels (δ above).
    pub floor_px: f64,
    /// The thinnest (2nd-percentile) stroke at the bake size before the
    /// floor, output pixels.
    pub stroke_p2_out: f64,
}

/// Why [`convert_line_art`] refused.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LineArtError {
    /// A zero-sized source.
    Empty,
    /// Cleaning left no ink: the image is blank paper, or so faint Otsu
    /// found nothing under [`PAPER_CEILING`].
    NoInk,
    /// Cleaning left no paper: the image is solid ink (nothing to outline).
    NoPaper,
}

impl std::fmt::Display for LineArtError {
    /// One sentence the shell can show as the reason.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LineArtError::Empty => write!(f, "the image is empty"),
            LineArtError::NoInk => write!(f, "no ink found: the image reads as blank paper"),
            LineArtError::NoPaper => write!(f, "no paper found: the image reads as solid ink"),
        }
    }
}

// ---------------------------------------------------------------------------
// The pipeline
// ---------------------------------------------------------------------------

/// The square, transform-applied image the pipeline cleans: the item drawn by
/// [`render_item`] (its zoom, pan and fit, so the result lines up with what
/// the user framed and what the family bake shows) at the source's own longer
/// side clamped to [`WORK_MIN`]..=[`WORK_MAX`].
///
/// `family_size` is the size the transform's pan was authored for (the
/// family's bake size, `Family::size`): pan is in pixels of that canvas, so
/// it is scaled by `side / family_size` here, or a bottom-anchored icon's
/// `fit_to_bottom` pan would shift by a quarter of the move at 1024².
/// At the source's own size and the identity transform this is a 1:1 copy;
/// a non-square source is padded with transparency, which [`clean_ink`]
/// reads as paper.
pub fn working_image(img: &DecodedImage, transform: &ItemTransform, family_size: u32) -> DecodedImage {
    let side = img.w.max(img.h).clamp(WORK_MIN, WORK_MAX);
    let k = f64::from(side) / f64::from(family_size.max(1));
    let t = ItemTransform { scale: transform.scale, pan_x: transform.pan_x * k, pan_y: transform.pan_y * k };
    render_item(img, &t, side, false)
}

/// Run the whole of Ruling CR's pipeline on `work` (normally
/// [`working_image`]'s output) and return the `out × out` bake.
///
/// `Err` when there is nothing to convert ([`LineArtError`]); never panics on
/// any image, including 1 × 1 and fully transparent ones.
pub fn convert_line_art(work: &DecodedImage, opts: &LineArtOptions, out: u32) -> Result<LineArtResult, LineArtError> {
    if work.w == 0 || work.h == 0 || out == 0 {
        return Err(LineArtError::Empty);
    }
    let (mask, clean) = clean_ink(work, opts.gap_closing);
    let n_ink = mask.count();
    if n_ink == 0 {
        return Err(LineArtError::NoInk);
    }
    if n_ink == mask.on.len() {
        return Err(LineArtError::NoPaper);
    }
    let pyr = SdfPyramid::build(&mask);
    let weight = opts.line_weight.clamp(LINE_WEIGHT_MIN, LINE_WEIGHT_MAX);
    let alpha = render_coverage(&pyr, out, weight);
    let stroke_p2_out = pyr.stroke_p2 * f64::from(out) / f64::from(pyr.mask_size);
    let floor_px = floor_offset(stroke_p2_out, weight);
    let mut rgba = vec![0u8; (out as usize) * (out as usize) * 4];
    for (px, a) in rgba.chunks_exact_mut(4).zip(alpha.iter()) {
        let a8 = quantise_alpha(*a);
        if a8 > 0 {
            px[0] = opts.ink[0];
            px[1] = opts.ink[1];
            px[2] = opts.ink[2];
        }
        px[3] = a8;
    }
    let image = DecodedImage::new(out, out, rgba).expect("out*out*4 bytes by construction");
    Ok(LineArtResult { image, clean, sdf_top: pyr.levels[0].size, floor_px, stroke_p2_out })
}

/// Coverage in `[0, 1]` to an 8-bit alpha, rounding half away from zero.
fn quantise_alpha(a: f32) -> u8 {
    (f64::from(a.clamp(0.0, 1.0)) * 255.0).round() as u8
}

/// The floor's offset δ in output pixels for a sprite whose thinnest stroke
/// is `stroke_p2_out` output pixels: half of what it lacks of `weight`, never
/// negative (a stroke already wide enough is not thinned). The lab's
/// `floor_delta`.
pub fn floor_offset(stroke_p2_out: f64, weight: f64) -> f64 {
    ((weight - stroke_p2_out) / 2.0).max(0.0)
}

/// Step 1: the cleaned ink mask, at `img`'s own resolution.
///
/// The lab's `clean_mask(scan, close, speck=24, pin=12, method="otsu")`, in
/// its order: flatten → Otsu → paper ceiling → despeckle → (close) → pinhole
/// fill. Transparent pixels are composited over white first, so a sprite
/// that is already keyed reads its transparency as paper.
pub fn clean_ink(img: &DecodedImage, closing: GapClosing) -> (Mask, CleanReport) {
    let (w, h) = (img.w as usize, img.h as usize);
    let lum = luminance_over_white(img);
    let (flat, paper) = flatten_paper(&lum, w, h);
    let evidence = scan_evidence(&flat, &paper);
    let threshold = otsu_threshold(&flat);
    let (mut on, shading_rejected) = ink_by_two_scales(&lum, &flat, &paper, w, h, threshold);

    let area = (w * h) as f64;
    let speck = area_scaled(SPECK_PX_AT_1024, area);
    let pin = area_scaled(PINHOLE_PX_AT_1024, area);

    let (labels, sizes) = label_components(&on, w, h, true);
    let specks_removed = sizes.iter().filter(|s| (**s as f64) < speck).count();
    for (b, l) in on.iter_mut().zip(labels.iter()) {
        if *l != 0 && (sizes[(*l - 1) as usize] as f64) < speck {
            *b = false;
        }
    }

    let closed = match closing {
        GapClosing::On => true,
        GapClosing::Off => false,
        GapClosing::Auto => evidence.is_scan(),
    };
    if closed {
        on = erode3(&dilate3(&on, w, h), w, h);
    }

    let paper_mask: Vec<bool> = on.iter().map(|b| !*b).collect();
    let (plabels, psizes) = label_components(&paper_mask, w, h, false);
    let pinholes_filled = psizes.iter().filter(|s| (**s as f64) < pin).count();
    for (b, l) in on.iter_mut().zip(plabels.iter()) {
        if *l != 0 && (psizes[(*l - 1) as usize] as f64) < pin {
            *b = true;
        }
    }

    let (_, csizes) = label_components(&on, w, h, true);
    let ink = on.iter().filter(|b| **b).count();
    let report = CleanReport {
        threshold,
        evidence,
        closed,
        shading_rejected,
        specks_removed,
        pinholes_filled,
        components: csizes.len(),
        ink_fraction: if area > 0.0 { ink as f64 / area } else { 0.0 },
    };
    (Mask { w, h, on }, report)
}

/// A pixel-count threshold given at 1024², scaled by image area, never below
/// 1 (a threshold of 1 removes nothing, since every component has ≥ 1 px).
/// Labelled judgement: dust and pinholes are physical sizes on the page, so
/// they scale with area, not side.
fn area_scaled(at_1024: f64, area: f64) -> f64 {
    (at_1024 * area / (1024.0 * 1024.0)).max(1.0)
}

/// Rec. 709 luma of each pixel composited over white, on the 0–255 scale,
/// from the sRGB-encoded bytes as the lab's `lum` reads them (no
/// linearisation: the threshold is a statistic, not a colour operation).
fn luminance_over_white(img: &DecodedImage) -> Vec<f64> {
    img.rgba
        .chunks_exact(4)
        .map(|p| {
            let a = f64::from(p[3]) / 255.0;
            let over = |c: u8| f64::from(c) * a + 255.0 * (1.0 - a);
            0.2126 * over(p[0]) + 0.7152 * over(p[1]) + 0.0722 * over(p[2])
        })
        .collect()
}

/// The lab's `normalise_paper`: the paper estimate `B` (a maximum filter,
/// then a mean filter, both [`PAPER_WINDOW_FRACTION`] of the longer side,
/// made odd, with scipy's `reflect` boundary) and the flattened luminance
/// `clip(L / max(B, 1), 0, FLAT_CLIP)`. Returns `(flat, paper)`.
fn flatten_paper(lum: &[f64], w: usize, h: usize) -> (Vec<f64>, Vec<f64>) {
    let win = ((PAPER_WINDOW_FRACTION * w.max(h) as f64) as usize) | 1;
    let maxed = separable(lum, w, h, |line, out| sliding_max(line, win, out));
    let paper = separable(&maxed, w, h, |line, out| sliding_mean(line, win, out));
    let flat = lum.iter().zip(paper.iter()).map(|(l, b)| (l / b.max(1.0)).clamp(0.0, FLAT_CLIP)).collect();
    (flat, paper)
}

/// Ink at two scales, so broad soft shading is not read as ink while a
/// broad solid fill still is. **An addition to the lab's single-scale
/// threshold, made while building AP-10** because the generated pencil scan
/// (`scan2`, `ap10_measure.py`) has a fold crease: its shadow, about 20 px
/// wide at 1024², fell under Otsu's threshold on the one wide paper estimate
/// and came out as a solid vertical line that also merged four hatch strokes
/// (components 10 against 14 at ×16 and ×64, ink mass 1.48).
///
/// - **Thin ink:** luminance against a grey closing of itself over a window
///   [`STROKE_WINDOW_FRACTION`] of the sprite (21 px at 1024²): the closing
///   erases every dark feature narrower than the window and keeps every
///   broader one, so `L / closing` is dark exactly on strokes, wherever they
///   sit, including on top of a shadow. Thin ink is
///   `L / closing < threshold`, with the paper ceiling still applied to the
///   flattened value.
/// - **Wide ink:** the closing itself against the wide paper estimate is
///   dark only where something broader than the window is dark: a solid fill
///   (the castle's door, research Finding 1) or shading. Each such
///   8-connected region is kept only when its boundary is sharp (mean
///   central-difference gradient of the flattened luminance at least
///   [`SHARP_EDGE_MIN`] per pixel): an inked fill has a pen edge, a shadow a
///   soft one.
///
/// Returns the mask and how many wide regions were dropped as shading.
fn ink_by_two_scales(lum: &[f64], flat: &[f64], paper: &[f64], w: usize, h: usize, threshold: f64) -> (Vec<bool>, usize) {
    let win = (((STROKE_WINDOW_FRACTION * w.max(h) as f64) as usize).max(3)) | 1;
    let dilated = separable(lum, w, h, |line, out| sliding_max(line, win, out));
    let closed = separable(&dilated, w, h, |line, out| sliding_min(line, win, out));
    let mut on: Vec<bool> = (0..w * h)
        .map(|i| (lum[i] / closed[i].max(1.0)).clamp(0.0, FLAT_CLIP) < threshold && flat[i] < PAPER_CEILING)
        .collect();
    let wide: Vec<bool> = (0..w * h).map(|i| (closed[i] / paper[i].max(1.0)).clamp(0.0, FLAT_CLIP) < threshold).collect();
    let (labels, sizes) = label_components(&wide, w, h, true);
    let mut grad_sum = vec![0f64; sizes.len()];
    let mut edge_n = vec![0u32; sizes.len()];
    let at = |x: i64, y: i64| flat[reflect(y, h) * w + reflect(x, w)];
    for y in 0..h {
        for x in 0..w {
            let l = labels[y * w + x];
            if l == 0 {
                continue;
            }
            let (xi, yi) = (x as i64, y as i64);
            let boundary = [(1i64, 0i64), (-1, 0), (0, 1), (0, -1)].iter().any(|(dx, dy)| {
                let (nx, ny) = (xi + dx, yi + dy);
                nx < 0 || ny < 0 || nx as usize >= w || ny as usize >= h || labels[ny as usize * w + nx as usize] != l
            });
            if boundary {
                let gx = (at(xi + 1, yi) - at(xi - 1, yi)).abs() / 2.0;
                let gy = (at(xi, yi + 1) - at(xi, yi - 1)).abs() / 2.0;
                grad_sum[(l - 1) as usize] += gx.max(gy);
                edge_n[(l - 1) as usize] += 1;
            }
        }
    }
    let keep: Vec<bool> = grad_sum.iter().zip(edge_n.iter()).map(|(g, n)| *n > 0 && g / f64::from(*n) >= SHARP_EDGE_MIN).collect();
    let rejected = keep.iter().filter(|k| !**k).count();
    for (b, l) in on.iter_mut().zip(labels.iter()) {
        if *l != 0 && keep[(*l - 1) as usize] {
            *b = true;
        }
    }
    (on, rejected)
}

/// [`ScanEvidence`] from the flattened luminance and the paper estimate.
/// Paper pixels for the grain are those at or above [`PAPER_CEILING`];
/// percentiles are the lab's (numpy's linear interpolation).
fn scan_evidence(flat: &[f64], paper: &[f64]) -> ScanEvidence {
    let mut b: Vec<f64> = paper.to_vec();
    b.sort_by(f64::total_cmp);
    let p99 = percentile_sorted(&b, 99.0);
    let p1 = percentile_sorted(&b, 1.0);
    let illumination = if p99 > 0.0 { (p99 - p1) / p99 } else { 0.0 };
    let mut on_paper: Vec<f64> = flat.iter().copied().filter(|v| *v >= PAPER_CEILING).collect();
    let grain = if on_paper.is_empty() {
        0.0
    } else {
        on_paper.sort_by(f64::total_cmp);
        let med = percentile_sorted(&on_paper, 50.0);
        let mut dev: Vec<f64> = on_paper.iter().map(|v| (v - med).abs()).collect();
        dev.sort_by(f64::total_cmp);
        percentile_sorted(&dev, 50.0)
    };
    ScanEvidence { illumination, grain }
}

/// numpy's default (`linear`) percentile of an ascending slice. 0 for an
/// empty one.
fn percentile_sorted(v: &[f64], p: f64) -> f64 {
    if v.is_empty() {
        return 0.0;
    }
    let pos = p / 100.0 * (v.len() - 1) as f64;
    let lo = pos.floor() as usize;
    let hi = (lo + 1).min(v.len() - 1);
    let t = pos - lo as f64;
    v[lo] + (v[hi] - v[lo]) * t
}

/// Otsu (1979) on a [`OTSU_BINS`]-bin histogram of `values` over
/// `[0, FLAT_CLIP]`, as the lab's `otsu`: the bin maximising the
/// between-class variance `w0·w1·(m0 − m1)²` (the first such bin on a tie,
/// numpy's `argmax`), answered as that bin's **upper edge** (see the
/// divergence note at the return). Ink is then `value < threshold`.
pub fn otsu_threshold(values: &[f64]) -> f64 {
    let mut hist = [0f64; OTSU_BINS];
    let scale = OTSU_BINS as f64 / FLAT_CLIP;
    for &v in values {
        let b = ((v * scale) as usize).min(OTSU_BINS - 1);
        hist[b] += 1.0;
    }
    let centre = |i: usize| (i as f64 + 0.5) / scale;
    let total: f64 = hist.iter().sum();
    let sum_all: f64 = hist.iter().enumerate().map(|(i, h)| h * centre(i)).sum();
    let (mut w0, mut s0) = (0.0f64, 0.0f64);
    let (mut best, mut best_i) = (f64::NEG_INFINITY, 0usize);
    for (i, h) in hist.iter().enumerate() {
        w0 += h;
        s0 += h * centre(i);
        let w1 = total - w0;
        let m0 = s0 / w0.max(1.0);
        let m1 = (sum_all - s0) / w1.max(1.0);
        let var = w0 * w1 * (m0 - m1) * (m0 - m1);
        if var > best {
            best = var;
            best_i = i;
        }
    }
    // The upper edge of the last bin in the dark class: class 0 is bins
    // `0..=best_i`, so every value in them is ink. **Deliberate divergence
    // from the lab's `otsu`**, which returned that bin's centre and so
    // dropped the dark class's own top half-bin (on a two-level image, the
    // whole dark level: `otsu_splits_two_levels`).
    (best_i as f64 + 1.0) / scale
}

// ---------------------------------------------------------------------------
// Step 2: the signed distance field
// ---------------------------------------------------------------------------

impl SdfPyramid {
    /// Build every level from `mask`.
    ///
    /// The lab's `sdf_from_mask`: blur the mask by [`CONTOUR_SIGMA`],
    /// supersample 2× bilinearly and threshold at 0.5, take the exact EDT
    /// inside and outside, `d = (inside − outside) / 2` in mask pixels; then
    /// each level samples `d` bilinearly at its own texel centres and
    /// quantises `128 + d_texels · 127 / SDF_SPREAD_TEXELS`.
    ///
    /// The top level is the smallest power of two giving the thinnest stroke
    /// ([`FLOOR_PERCENTILE`]) [`MIN_TEXELS_ACROSS_STROKE`] texels, clamped to
    /// [`SDF_TOP_MIN`]..=[`SDF_TOP_MAX`] and to at most twice the mask (the
    /// supersample's own resolution: more texels than that carry nothing).
    /// The stroke width used here is `2·edt` at the ridge (the full width;
    /// `stroke_p2` reports the lab's `2·edt − 1`), so a 4 px hatch asks for
    /// 512² at 1024², the research's measured level.
    pub fn build(mask: &Mask) -> SdfPyramid {
        let n = mask.w.max(mask.h);
        let widths = ridge_widths(mask);
        let stroke_p2 = percentile_sorted(&widths, FLOOR_PERCENTILE);
        let full_width = (stroke_p2 + 1.0).max(1.0);
        let want = (MIN_TEXELS_ACROSS_STROKE * n as f64 / full_width).ceil().max(1.0) as u32;
        let top = want.next_power_of_two().clamp(SDF_TOP_MIN, SDF_TOP_MAX).min(((2 * n) as u32).next_power_of_two());
        let field = signed_field_2x(mask);
        let mut levels = Vec::new();
        let mut r = top;
        loop {
            levels.push(field.level(r, n));
            if r / 2 < SDF_LEVEL_MIN || r <= 1 {
                break;
            }
            r /= 2;
        }
        SdfPyramid { levels, mask_size: n as u32, stroke_p2 }
    }

    /// The level [`render_coverage`] reads for an `out`-pixel render: the
    /// largest whose texels per output pixel are at most
    /// [`MAX_TEXELS_PER_PIXEL`], or the smallest level if none is that
    /// coarse.
    pub fn level_for(&self, out: u32) -> &Sdf {
        self.levels
            .iter()
            .find(|l| f64::from(l.size) / f64::from(out.max(1)) <= MAX_TEXELS_PER_PIXEL)
            .unwrap_or_else(|| self.levels.last().expect("build always pushes at least one level"))
    }
}

/// The signed distance on the mask's 2× supersample, in mask pixels,
/// positive inside. Row-major, `side × side` with `side = 2·max(w, h)`.
struct SignedField {
    side: usize,
    d: Vec<f64>,
}

impl SignedField {
    /// One quantised level of `r × r` texels, for a mask of side `n`.
    fn level(&self, r: u32, n: usize) -> Sdf {
        let r_us = r as usize;
        let mut q = Vec::with_capacity(r_us * r_us);
        let step = self.side as f64 / f64::from(r);
        let to_texels = f64::from(r) / n as f64;
        for y in 0..r_us {
            let sy = (y as f64 + 0.5) * step - 0.5;
            for x in 0..r_us {
                let sx = (x as f64 + 0.5) * step - 0.5;
                let d = bilinear(&self.d, self.side, self.side, sx, sy) * to_texels;
                q.push((128.0 + d * 127.0 / SDF_SPREAD_TEXELS).round().clamp(0.0, 255.0) as u8);
            }
        }
        Sdf { size: r, q }
    }
}

/// Build the [`SignedField`]: blur, 2× bilinear supersample, threshold at
/// 0.5, exact EDT each side. A non-square mask is treated as padded with
/// paper to its longer side.
fn signed_field_2x(mask: &Mask) -> SignedField {
    let n = mask.w.max(mask.h);
    let mut m = vec![0f64; n * n];
    for y in 0..mask.h {
        for x in 0..mask.w {
            if mask.on[y * mask.w + x] {
                m[y * n + x] = 1.0;
            }
        }
    }
    let blurred = gaussian_blur(&m, n, n, CONTOUR_SIGMA);
    let side = 2 * n;
    let mut up = vec![false; side * side];
    for y in 0..side {
        let sy = (y as f64 + 0.5) / 2.0 - 0.5;
        for x in 0..side {
            let sx = (x as f64 + 0.5) / 2.0 - 0.5;
            // A mask pixel the blur took below 0.5 at its own centre is a
            // stroke thinner than the blur (a 1 px line peaks at 0.499 under
            // σ 0.8). The lab's threshold would delete it; here the pixel
            // keeps its whole 2 × 2 block, so no ink the cleaner kept is lost
            // to the contour smoothing. **Deliberate divergence from the
            // lab's `sdf_from_mask`**, which never met such a stroke (its
            // samples' thinnest is 4 px at 1024²); a 256² pack sprite with
            // 1 px lines does (`the_floor_makes_a_thin_line_one_pixel_solid`).
            let (px, py) = (x / 2, y / 2);
            let thin_ink = m[py * n + px] > 0.5 && blurred[py * n + px] <= 0.5;
            up[y * side + x] = thin_ink || bilinear(&blurred, n, n, sx, sy) > 0.5;
        }
    }
    let inside = edt(&up, side, side, false);
    let outside = edt(&up, side, side, true);
    let d = inside.iter().zip(outside.iter()).map(|(i, o)| (i - o) / 2.0).collect();
    SignedField { side, d }
}

/// The stroke widths along the mask's medial ridge, ascending: `2·e − 1`
/// wherever `e` (the EDT inside the ink) is a local maximum of its 3 × 3
/// neighbourhood. The lab's `ridge_widths`. Never empty: an inkless mask
/// answers `[0.0]`, as the lab does.
pub fn ridge_widths(mask: &Mask) -> Vec<f64> {
    let (w, h) = (mask.w, mask.h);
    let e = edt(&mask.on, w, h, false);
    let mut out = Vec::new();
    for y in 0..h {
        for x in 0..w {
            let v = e[y * w + x];
            if !mask.on[y * w + x] || v <= 0.0 {
                continue;
            }
            let mut is_max = true;
            'n: for dy in -1i64..=1 {
                for dx in -1i64..=1 {
                    let (nx, ny) = (reflect(x as i64 + dx, w), reflect(y as i64 + dy, h));
                    if e[ny * w + nx] > v {
                        is_max = false;
                        break 'n;
                    }
                }
            }
            if is_max {
                out.push(2.0 * v - 1.0);
            }
        }
    }
    if out.is_empty() {
        out.push(0.0);
    }
    out.sort_by(f64::total_cmp);
    out
}

/// The exact Euclidean distance transform (Felzenszwalb & Huttenlocher,
/// *Distance Transforms of Sampled Functions*, Theory of Computing 8, 2012):
/// for each pixel of the set (`on == !of_background`), the distance to the
/// nearest pixel outside it; 0 outside the set. With `of_background` false it
/// is scipy's `distance_transform_edt(on)`; true gives
/// `distance_transform_edt(~on)`. Only pixels inside the image count as
/// "outside" (scipy's convention), so a set filling the whole image has no
/// finite distance and answers [`EDT_NONE`].
pub fn edt(on: &[bool], w: usize, h: usize, of_background: bool) -> Vec<f64> {
    const INF: f64 = 1e20;
    let mut g = vec![0f64; w * h];
    for (gv, b) in g.iter_mut().zip(on.iter()) {
        let in_set = *b != of_background;
        *gv = if in_set { INF } else { 0.0 };
    }
    let mut buf_f = vec![0f64; w.max(h)];
    let mut buf_d = vec![0f64; w.max(h)];
    let mut v = vec![0usize; w.max(h)];
    let mut z = vec![0f64; w.max(h) + 1];
    for y in 0..h {
        buf_f[..w].copy_from_slice(&g[y * w..(y + 1) * w]);
        dt_1d(&buf_f[..w], &mut buf_d[..w], &mut v, &mut z);
        g[y * w..(y + 1) * w].copy_from_slice(&buf_d[..w]);
    }
    for x in 0..w {
        for y in 0..h {
            buf_f[y] = g[y * w + x];
        }
        dt_1d(&buf_f[..h], &mut buf_d[..h], &mut v, &mut z);
        for y in 0..h {
            g[y * w + x] = buf_d[y];
        }
    }
    g.iter().map(|s| if *s >= INF / 2.0 { EDT_NONE } else { s.sqrt() }).collect()
}

/// [`edt`]'s answer for a pixel with no pixel outside its set anywhere in the
/// image. Large enough to saturate every quantiser in this module.
pub const EDT_NONE: f64 = 1e9;

/// Felzenszwalb & Huttenlocher's 1-D squared distance transform (their
/// Algorithm 1, the lower envelope of parabolas rooted at each sample):
/// `d[q] = min_p ((q − p)² + f[p])`. `v` and `z` are scratch of length
/// `n` and `n + 1`.
fn dt_1d(f: &[f64], d: &mut [f64], v: &mut [usize], z: &mut [f64]) {
    let n = f.len();
    if n == 0 {
        return;
    }
    let mut k = 0usize;
    v[0] = 0;
    z[0] = f64::NEG_INFINITY;
    z[1] = f64::INFINITY;
    for q in 1..n {
        let qf = q as f64;
        loop {
            let p = v[k];
            let pf = p as f64;
            let s = ((f[q] + qf * qf) - (f[p] + pf * pf)) / (2.0 * qf - 2.0 * pf);
            if s <= z[k] {
                if k == 0 {
                    // Cannot happen for finite inputs (z[0] is -inf), kept so
                    // the loop is total.
                    break;
                }
                k -= 1;
            } else {
                k += 1;
                v[k] = q;
                z[k] = s;
                z[k + 1] = f64::INFINITY;
                break;
            }
        }
    }
    k = 0;
    for (q, dq) in d.iter_mut().enumerate().take(n) {
        let qf = q as f64;
        while z[k + 1] < qf {
            k += 1;
        }
        let p = v[k] as f64;
        *dq = (qf - p) * (qf - p) + f[v[k]];
    }
}

// ---------------------------------------------------------------------------
// Step 3: rendering
// ---------------------------------------------------------------------------

/// Render `pyr` at `out × out` with the stroke floor for `weight` output
/// pixels: coverage in `[0, 1]`, row-major.
///
/// The lab's `render_sdf(q, H, delta_out)`: sample the chosen level
/// ([`SdfPyramid::level_for`]) bilinearly at each output pixel centre
/// (clamped at the border), convert texels to output pixels, add the floor
/// offset ([`floor_offset`]) and half a pixel, clamp. The result is a one
/// pixel edge ramp centred on the (floored) contour: Valve's alpha-tested
/// magnification (Green 2007) with the threshold replaced by a ramp.
pub fn render_coverage(pyr: &SdfPyramid, out: u32, weight: f64) -> Vec<f32> {
    let lvl = pyr.level_for(out);
    let r = lvl.size as usize;
    let field: Vec<f64> = lvl.q.iter().map(|q| (f64::from(*q) - 128.0) * SDF_SPREAD_TEXELS / 127.0).collect();
    let stroke_out = pyr.stroke_p2 * f64::from(out) / f64::from(pyr.mask_size);
    let delta = floor_offset(stroke_out, weight);
    let step = f64::from(lvl.size) / f64::from(out);
    let to_out = f64::from(out) / f64::from(lvl.size);
    let o = out as usize;
    let mut a = Vec::with_capacity(o * o);
    for y in 0..o {
        let sy = (y as f64 + 0.5) * step - 0.5;
        for x in 0..o {
            let sx = (x as f64 + 0.5) * step - 0.5;
            let d = bilinear(&field, r, r, sx, sy) * to_out;
            a.push((d + delta + 0.5).clamp(0.0, 1.0) as f32);
        }
    }
    a
}

// ---------------------------------------------------------------------------
// Shared raster helpers
// ---------------------------------------------------------------------------

/// Bilinear sample of a `w × h` field at continuous pixel coordinates (pixel
/// centres at integers), clamped at the border (scipy `mode="nearest"`).
fn bilinear(f: &[f64], w: usize, h: usize, x: f64, y: f64) -> f64 {
    let x = x.clamp(0.0, (w - 1) as f64);
    let y = y.clamp(0.0, (h - 1) as f64);
    let x0 = x.floor() as usize;
    let y0 = y.floor() as usize;
    let x1 = (x0 + 1).min(w - 1);
    let y1 = (y0 + 1).min(h - 1);
    let fx = x - x0 as f64;
    let fy = y - y0 as f64;
    let top = f[y0 * w + x0] * (1.0 - fx) + f[y0 * w + x1] * fx;
    let bot = f[y1 * w + x0] * (1.0 - fx) + f[y1 * w + x1] * fx;
    top * (1.0 - fy) + bot * fy
}

/// scipy's `reflect` boundary (`d c b a | a b c d | d c b a`) for an index
/// that may lie up to one full length outside `0..n`; folded repeatedly for
/// anything further.
fn reflect(i: i64, n: usize) -> usize {
    let n = n as i64;
    if n == 1 {
        return 0;
    }
    let period = 2 * n;
    let mut m = i.rem_euclid(period);
    if m >= n {
        m = period - 1 - m;
    }
    m as usize
}

/// Apply a 1-D filter along every row, then every column.
fn separable(src: &[f64], w: usize, h: usize, f: impl Fn(&[f64], &mut [f64])) -> Vec<f64> {
    let mut tmp = vec![0f64; w * h];
    let mut out_line = vec![0f64; w.max(h)];
    for y in 0..h {
        f(&src[y * w..(y + 1) * w], &mut out_line[..w]);
        tmp[y * w..(y + 1) * w].copy_from_slice(&out_line[..w]);
    }
    let mut out = vec![0f64; w * h];
    let mut col = vec![0f64; h];
    for x in 0..w {
        for y in 0..h {
            col[y] = tmp[y * w + x];
        }
        f(&col, &mut out_line[..h]);
        for y in 0..h {
            out[y * w + x] = out_line[y];
        }
    }
    out
}

/// A centred sliding-window maximum of odd width `win` with the `reflect`
/// boundary (scipy `maximum_filter1d`), by a monotonic deque over the
/// reflected line: O(n) whatever the window.
fn sliding_max(line: &[f64], win: usize, out: &mut [f64]) {
    let n = line.len();
    let r = (win / 2) as i64;
    let at = |i: i64| line[reflect(i, n)];
    let mut dq: std::collections::VecDeque<(i64, f64)> = std::collections::VecDeque::new();
    let mut next = -r;
    for (i, o) in out.iter_mut().enumerate().take(n) {
        let hi = i as i64 + r;
        while next <= hi {
            let v = at(next);
            while dq.back().is_some_and(|b| b.1 <= v) {
                dq.pop_back();
            }
            dq.push_back((next, v));
            next += 1;
        }
        while dq.front().is_some_and(|f| f.0 < i as i64 - r) {
            dq.pop_front();
        }
        *o = dq.front().map(|f| f.1).unwrap_or(0.0);
    }
}

/// A centred sliding-window minimum, as [`sliding_max`] on the negated line.
fn sliding_min(line: &[f64], win: usize, out: &mut [f64]) {
    let neg: Vec<f64> = line.iter().map(|v| -v).collect();
    sliding_max(&neg, win, out);
    for v in out.iter_mut() {
        *v = -*v;
    }
}

/// A centred sliding-window mean of odd width `win` with the `reflect`
/// boundary (scipy `uniform_filter1d`), by a running sum.
fn sliding_mean(line: &[f64], win: usize, out: &mut [f64]) {
    let n = line.len();
    let r = (win / 2) as i64;
    let at = |i: i64| line[reflect(i, n)];
    let mut sum: f64 = (-r..=r).map(at).sum();
    for (i, o) in out.iter_mut().enumerate().take(n) {
        *o = sum / win as f64;
        let i = i as i64;
        sum += at(i + r + 1) - at(i - r);
    }
}

/// A separable Gaussian of `sigma` with radius `round(4σ)` (scipy's default
/// `truncate=4.0`, radius `int(4σ + 0.5)`) and the `reflect` boundary.
fn gaussian_blur(src: &[f64], w: usize, h: usize, sigma: f64) -> Vec<f64> {
    let radius = (4.0 * sigma + 0.5) as i64;
    let mut k: Vec<f64> = (-radius..=radius).map(|i| (-0.5 * (i as f64 / sigma).powi(2)).exp()).collect();
    let s: f64 = k.iter().sum();
    for v in &mut k {
        *v /= s;
    }
    separable(src, w, h, |line, out| {
        let n = line.len();
        for (i, o) in out.iter_mut().enumerate().take(n) {
            let mut acc = 0.0;
            for (j, kv) in k.iter().enumerate() {
                acc += kv * line[reflect(i as i64 + j as i64 - radius, n)];
            }
            *o = acc;
        }
    })
}

/// 3 × 3 binary dilation, outside the image counting as paper (scipy
/// `binary_dilation(.., ones((3,3)), border_value=0)`).
fn dilate3(on: &[bool], w: usize, h: usize) -> Vec<bool> {
    morph3(on, w, h, true)
}

/// 3 × 3 binary erosion, outside the image counting as paper (scipy
/// `binary_erosion(.., ones((3,3)), border_value=0)`): ink touching the
/// border erodes there, as the lab's `binary_closing` does.
fn erode3(on: &[bool], w: usize, h: usize) -> Vec<bool> {
    morph3(on, w, h, false)
}

/// The shared body of [`dilate3`] (`any`) and [`erode3`] (`all`).
fn morph3(on: &[bool], w: usize, h: usize, any: bool) -> Vec<bool> {
    let mut out = vec![false; w * h];
    for y in 0..h {
        for x in 0..w {
            let mut acc = !any;
            for dy in -1i64..=1 {
                for dx in -1i64..=1 {
                    let (nx, ny) = (x as i64 + dx, y as i64 + dy);
                    let v = nx >= 0 && ny >= 0 && (nx as usize) < w && (ny as usize) < h && on[ny as usize * w + nx as usize];
                    if any {
                        acc |= v;
                    } else {
                        acc &= v;
                    }
                }
            }
            out[y * w + x] = acc;
        }
    }
    out
}

/// Connected components of the `true` pixels, 8-connected when `eight`, else
/// 4-connected (scipy `label` with `ones((3,3))` or its default cross).
/// Returns per-pixel labels (0 = not in the set, else 1-based) and each
/// component's size, in first-pixel raster order, so the numbering is
/// deterministic.
pub fn label_components(on: &[bool], w: usize, h: usize, eight: bool) -> (Vec<u32>, Vec<u32>) {
    let mut labels = vec![0u32; w * h];
    let mut sizes = Vec::new();
    let mut stack = Vec::new();
    for start in 0..w * h {
        if !on[start] || labels[start] != 0 {
            continue;
        }
        let id = sizes.len() as u32 + 1;
        let mut size = 0u32;
        labels[start] = id;
        stack.push(start);
        while let Some(p) = stack.pop() {
            size += 1;
            let (x, y) = ((p % w) as i64, (p / w) as i64);
            for dy in -1i64..=1 {
                for dx in -1i64..=1 {
                    if (dx == 0 && dy == 0) || (!eight && dx != 0 && dy != 0) {
                        continue;
                    }
                    let (nx, ny) = (x + dx, y + dy);
                    if nx < 0 || ny < 0 || nx as usize >= w || ny as usize >= h {
                        continue;
                    }
                    let q = ny as usize * w + nx as usize;
                    if on[q] && labels[q] == 0 {
                        labels[q] = id;
                        stack.push(q);
                    }
                }
            }
        }
        sizes.push(size);
    }
    (labels, sizes)
}

#[cfg(test)]
mod tests {
    //! Unit tests on small shapes built here. The four research samples and
    //! the pencil scan, with their numeric bars, are `tests/lineart_ap10.rs`.
    use super::*;

    /// A white image with black rectangles `(x0, y0, x1, y1)` (exclusive).
    fn page(n: u32, rects: &[(u32, u32, u32, u32)]) -> DecodedImage {
        let mut rgba = vec![255u8; (n * n * 4) as usize];
        for &(x0, y0, x1, y1) in rects {
            for y in y0..y1 {
                for x in x0..x1 {
                    let i = ((y * n + x) * 4) as usize;
                    rgba[i..i + 3].copy_from_slice(&[0, 0, 0]);
                }
            }
        }
        DecodedImage::new(n, n, rgba).unwrap()
    }

    /// Protects the EDT's exactness: every distance equals a brute-force
    /// minimum over all background pixels, on a mask with irregular shapes
    /// (a jump flood, `build_coast_sdf`'s method, would fail this beyond one
    /// seed: `MISTAKES.md`, "Write an oracle").
    #[test]
    fn edt_is_exact_against_brute_force() {
        let (w, h) = (23usize, 17usize);
        let on: Vec<bool> = (0..w * h).map(|i| ((i * 7919) % 13) < 9 || (i % w > 5 && i % w < 15 && i / w > 3)).collect();
        let d = edt(&on, w, h, false);
        for y in 0..h {
            for x in 0..w {
                let want = if !on[y * w + x] {
                    0.0
                } else {
                    let mut best = f64::INFINITY;
                    for yy in 0..h {
                        for xx in 0..w {
                            if !on[yy * w + xx] {
                                let dd = ((x as f64 - xx as f64).powi(2) + (y as f64 - yy as f64).powi(2)).sqrt();
                                best = best.min(dd);
                            }
                        }
                    }
                    best
                };
                assert!((d[y * w + x] - want).abs() < 1e-9, "({x},{y}) {} vs {want}", d[y * w + x]);
            }
        }
    }

    /// Protects the EDT's "no background anywhere" answer: a full set has no
    /// finite distance, and says so with [`EDT_NONE`] rather than a
    /// plausible-looking number (`MISTAKES.md`, "no value as a plausible value").
    #[test]
    fn edt_of_a_full_set_is_none_not_a_distance() {
        let d = edt(&[true; 9], 3, 3, false);
        assert!(d.iter().all(|v| *v == 1e9));
    }

    /// Protects Otsu: on a two-level image the threshold lands between the
    /// levels, so every dark pixel is ink and no light one is.
    #[test]
    fn otsu_splits_two_levels() {
        let v: Vec<f64> = (0..1000).map(|i| if i % 4 == 0 { 0.2 } else { 1.0 }).collect();
        let t = otsu_threshold(&v);
        assert!(t > 0.2 && t < 1.0, "{t}");
        assert_eq!(v.iter().filter(|x| **x < t).count(), 250);
    }

    /// Protects the paper flattening (research Finding 1): a solid block far
    /// larger than a narrow window, under a strong illumination gradient,
    /// stays solid, and the gradient's paper is not called ink.
    #[test]
    fn flattening_keeps_a_large_solid_area_and_drops_the_gradient() {
        let n = 200u32;
        let mut img = page(n, &[(60, 60, 140, 140)]);
        for y in 0..n {
            for x in 0..n {
                let i = ((y * n + x) * 4) as usize;
                if img.rgba[i] == 255 {
                    let v = (255.0 * (1.0 - 0.3 * f64::from(x) / f64::from(n))) as u8;
                    img.rgba[i..i + 3].copy_from_slice(&[v, v, v]);
                }
            }
        }
        let (m, r) = clean_ink(&img, GapClosing::Off);
        let inside = (60..140).flat_map(|y| (60..140).map(move |x| (x, y))).filter(|(x, y)| m.on[y * 200 + x]).count();
        assert_eq!(inside, 80 * 80, "the block was hollowed");
        assert_eq!(m.count(), 80 * 80, "paper was called ink");
        assert_eq!(r.components, 1);
        assert!(r.evidence.illumination > SCAN_ILLUMINATION, "a 30 % fall-off is scan-like: {:?}", r.evidence);
    }

    /// Protects despeckle and pinhole fill: a 2 × 2 dust speck goes, a 2 × 2
    /// pinhole inside a stroke is filled, and a 30 × 30 window stays open.
    /// Thresholds at 128² are 24·(128/1024)² = 0.375 → 1 and 0.19 → 1, so
    /// this runs at 1024² where the 24 / 12 px limits apply.
    #[test]
    fn specks_go_pinholes_fill_windows_stay() {
        let n = 1024u32;
        let mut img = page(n, &[(100, 100, 400, 400), (700, 700, 702, 702)]);
        // A 2x2 pinhole and a 30x30 window inside the block.
        for (x0, y0, s) in [(150u32, 150u32, 2u32), (250, 250, 30)] {
            for y in y0..y0 + s {
                for x in x0..x0 + s {
                    let i = ((y * n + x) * 4) as usize;
                    img.rgba[i..i + 3].copy_from_slice(&[255, 255, 255]);
                }
            }
        }
        let (m, r) = clean_ink(&img, GapClosing::Off);
        assert!(!m.on[701 * 1024 + 701], "speck kept");
        assert!(m.on[150 * 1024 + 150], "pinhole not filled");
        assert!(!m.on[260 * 1024 + 260], "window filled");
        assert_eq!(r.specks_removed, 1);
        assert_eq!(r.pinholes_filled, 1);
    }

    /// Protects scans-only closing: on clean art Auto does not close, so two
    /// strokes 1 px apart stay two; forced On merges them.
    #[test]
    fn auto_does_not_close_clean_art() {
        let img = page(256, &[(50, 50, 200, 56), (50, 57, 200, 63)]);
        let (m_auto, r_auto) = clean_ink(&img, GapClosing::Auto);
        assert!(!r_auto.closed);
        assert_eq!(r_auto.components, 2);
        assert!(!m_auto.on[56 * 256 + 100]);
        let (_, r_on) = clean_ink(&img, GapClosing::On);
        assert!(r_on.closed);
        assert_eq!(r_on.components, 1);
    }

    /// Protects the stroke floor (Ruling CR Q-L1): a 1 px line at 256²
    /// rendered at 32 px (0.125 px wide unfloored) is unbroken at alpha > 0.5
    /// in every column it crosses and carries about one pixel of ink per
    /// column (0.93 measured; bilinear sampling of the 128² level shaves the
    /// tent's peak, hence the 0.85 bar), and weight 1.5 carries clearly more
    /// ink than 0.5. Without the floor the line is 0.125 px of tone.
    #[test]
    fn the_floor_makes_a_thin_line_one_pixel_solid() {
        let img = page(256, &[(20, 128, 236, 129)]);
        let (m, _) = clean_ink(&img, GapClosing::Off);
        let p = SdfPyramid::build(&m);
        let a = render_coverage(&p, 32, 1.0);
        // Along the line's middle, the column sum of coverage is >= ~1 px.
        for x in 4..28 {
            let col: f32 = (0..32).map(|y| a[y * 32 + x]).sum();
            assert!(col >= 0.85, "column {x} carries {col} px of ink");
            assert!((0..32).any(|y| a[y * 32 + x] > 0.5), "column {x} is broken");
        }
        let faint: f32 = render_coverage(&p, 32, 0.5).iter().sum();
        let bold: f32 = render_coverage(&p, 32, 1.5).iter().sum();
        assert!(bold > faint * 1.5, "{bold} vs {faint}");
    }

    /// Protects level selection (research Finding 5): no render reads more
    /// than 4 texels per output pixel when a coarser level exists.
    #[test]
    fn level_choice_respects_four_texels_per_pixel() {
        let img = page(1024, &[(100, 100, 104, 900)]);
        let (m, _) = clean_ink(&img, GapClosing::Off);
        let p = SdfPyramid::build(&m);
        assert_eq!(p.levels[0].size, 512, "a 4 px stroke at 1024 asks for 512");
        assert_eq!(p.level_for(40).size, 128);
        assert_eq!(p.level_for(160).size, 512);
        assert_eq!(p.level_for(256).size, 512);
        assert_eq!(p.level_for(8).size, 32);
        assert_eq!(p.levels.last().unwrap().size, SDF_LEVEL_MIN);
    }

    /// Protects the variant's colour contract (Ruling CQ Q4, halo 0): every
    /// pixel with alpha is exactly the ink, every pixel without is (0,0,0,0).
    #[test]
    fn the_bake_is_one_ink_colour() {
        let img = page(512, &[(100, 100, 400, 110), (100, 200, 110, 400)]);
        let opts = LineArtOptions { ink: [12, 34, 56], ..LineArtOptions::default() };
        let r = convert_line_art(&img, &opts, 256).unwrap();
        let mut inked = 0;
        for p in r.image.rgba.chunks_exact(4) {
            if p[3] > 0 {
                inked += 1;
                assert_eq!(&p[..3], &[12, 34, 56]);
            } else {
                assert_eq!(p, &[0, 0, 0, 0]);
            }
        }
        assert!(inked > 1000, "{inked}");
    }

    /// Protects the refusals: blank paper, solid ink and empty images give
    /// a reason, not a blank variant.
    #[test]
    fn nothing_to_convert_is_an_error_not_a_blank_variant() {
        let o = LineArtOptions::default();
        assert_eq!(convert_line_art(&page(64, &[]), &o, 32).unwrap_err(), LineArtError::NoInk);
        assert_eq!(convert_line_art(&page(64, &[(0, 0, 64, 64)]), &o, 32).unwrap_err(), LineArtError::NoPaper);
        let empty = DecodedImage::new(0, 0, vec![]).unwrap();
        assert_eq!(convert_line_art(&empty, &o, 32).unwrap_err(), LineArtError::Empty);
        let one = DecodedImage::new(1, 1, vec![0, 0, 0, 0]).unwrap();
        assert!(convert_line_art(&one, &o, 32).is_err());
    }

    /// Protects transparency-as-paper: a keyed sprite (black strokes on
    /// alpha 0) cleans exactly as the same strokes on white.
    #[test]
    fn transparency_reads_as_paper() {
        let white = page(128, &[(20, 20, 108, 26)]);
        let mut keyed = white.clone();
        for p in keyed.rgba.chunks_exact_mut(4) {
            if p[0] == 255 {
                p.copy_from_slice(&[0, 0, 0, 0]);
            }
        }
        assert_eq!(clean_ink(&white, GapClosing::Off).0, clean_ink(&keyed, GapClosing::Off).0);
    }

    /// Protects the two-scale ink test (`ink_by_two_scales`): on a 1024²
    /// page, a broad soft shadow (Gaussian, σ 30 px, 50 % deep) is dropped as
    /// shading, while a sharp 200 px solid block (wider than the stroke
    /// window, so it reaches the sharp-edge test) and a 4 px line crossing
    /// the shadow are both kept whole.
    #[test]
    fn a_broad_soft_shadow_is_not_ink() {
        let n = 1024u32;
        let mut img = page(n, &[(700, 100, 900, 300), (100, 600, 900, 604)]);
        for y in 0..n {
            for x in 0..n {
                let i = ((y * n + x) * 4) as usize;
                let shade = 1.0 - 0.5 * (-((f64::from(x) - 400.0) / 30.0).powi(2) / 2.0).exp();
                for c in 0..3 {
                    img.rgba[i + c] = (f64::from(img.rgba[i + c]) * shade).round() as u8;
                }
            }
        }
        let (m, r) = clean_ink(&img, GapClosing::Off);
        assert!(r.shading_rejected >= 1, "{r:?}");
        assert!(!m.on[300 * 1024 + 400], "the shadow's centre was called ink");
        assert!((700..900).all(|x| (100..300).all(|y| m.on[y * 1024 + x])), "the solid block was not kept whole");
        assert!((100..900).all(|x| m.on[601 * 1024 + x]), "the line was broken where it crosses the shadow");
        assert_eq!(r.components, 2, "{r:?}");
    }

    /// Protects `working_image`'s pan scaling: a pan authored on the 256²
    /// family canvas moves the same fraction of the sprite at the 1024²
    /// working size (64 px family pan = 256 px working pan).
    #[test]
    fn working_image_scales_the_pan_to_the_working_size() {
        let src = page(1024, &[(500, 500, 524, 524)]);
        let t = ItemTransform { scale: 1.0, pan_x: 64.0, pan_y: 0.0 };
        let w = working_image(&src, &t, 256);
        assert_eq!((w.w, w.h), (1024, 1024));
        let dark = |x: u32, y: u32| w.rgba[((y * 1024 + x) * 4) as usize + 3] > 0 && w.rgba[((y * 1024 + x) * 4) as usize] < 128;
        assert!(dark(512 + 256, 512), "the block did not move by 256 px");
        assert!(!dark(512 + 64, 512), "the pan was applied unscaled");
    }

    /// A 512² page of even light with deterministic paper grain (±`amp`
    /// levels, a fixed integer hash: no RNG) and an optional horizontal mark
    /// of grey `mark`.
    fn grainy_page(amp: i32, mark: Option<u8>) -> DecodedImage {
        let n = 512u32;
        let mut rgba = vec![255u8; (n * n * 4) as usize];
        for y in 0..n {
            for x in 0..n {
                let hsh = (x.wrapping_mul(73_856_093) ^ y.wrapping_mul(19_349_663)) % 1000;
                let v = (235 + (hsh as i32 * (2 * amp + 1) / 1000) - amp).clamp(0, 255) as u8;
                let v = match mark {
                    Some(m) if (250..256).contains(&y) && (100..400).contains(&x) => m,
                    _ => v,
                };
                let i = ((y * n + x) * 4) as usize;
                rgba[i..i + 3].copy_from_slice(&[v, v, v]);
            }
        }
        DecodedImage::new(n, n, rgba).unwrap()
    }

    /// Protects the grain half of scan detection ([`SCAN_GRAIN`]): an evenly
    /// lit page with ±6 levels of grain (no illumination fall-off, so only
    /// grain can say "scan") is detected and closed; the same page without
    /// grain is not. Both fixture scans also have a fall-off, so without
    /// this test the grain threshold was unprotected (a mutant at 0.04
    /// survived, 2026-10-19).
    #[test]
    fn grain_alone_reads_as_a_scan() {
        let (_, grainy) = clean_ink(&grainy_page(6, Some(30)), GapClosing::Auto);
        assert!(grainy.evidence.illumination <= SCAN_ILLUMINATION, "{:?}", grainy.evidence);
        assert!(grainy.evidence.is_scan() && grainy.closed, "{:?}", grainy.evidence);
        let (_, smooth) = clean_ink(&grainy_page(0, Some(30)), GapClosing::Auto);
        assert!(!smooth.evidence.is_scan(), "{:?}", smooth.evidence);
    }

    /// Protects the paper ceiling ([`PAPER_CEILING`]): on a grainy page whose
    /// only mark is fainter than the ceiling, Otsu splits the paper's grain,
    /// and the ceiling is what keeps that split from becoming "ink". The
    /// answer is "no ink", not a variant of speckle.
    #[test]
    fn grain_under_the_ceiling_is_never_ink() {
        let o = LineArtOptions::default();
        let err = convert_line_art(&grainy_page(6, Some(222)), &o, 64).unwrap_err();
        assert_eq!(err, LineArtError::NoInk);
    }

    /// Protects the reflect boundary (scipy's `reflect`, not `mirror`).
    #[test]
    fn reflect_matches_scipy() {
        let got: Vec<usize> = (-3i64..7).map(|i| reflect(i, 4)).collect();
        assert_eq!(got, vec![2, 1, 0, 0, 1, 2, 3, 3, 2, 1]);
    }

    /// Protects the sliding filters against a direct window computation.
    #[test]
    fn sliding_filters_match_a_direct_window() {
        let line: Vec<f64> = (0..37).map(|i| ((i * 31) % 17) as f64).collect();
        let mut mx = vec![0.0; 37];
        let mut mn = vec![0.0; 37];
        sliding_max(&line, 9, &mut mx);
        sliding_mean(&line, 9, &mut mn);
        for i in 0..37i64 {
            let win: Vec<f64> = (i - 4..=i + 4).map(|j| line[reflect(j, 37)]).collect();
            assert_eq!(mx[i as usize], win.iter().cloned().fold(f64::MIN, f64::max));
            assert!((mn[i as usize] - win.iter().sum::<f64>() / 9.0).abs() < 1e-9);
        }
    }
}
