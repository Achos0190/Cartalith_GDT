//! LOD-D0's zoom-sweep metrics — `LOD_DETAIL_SCOPE.md`, "LOD-D0 · Zoom-sweep
//! harness — measurement only, no pixel change".
//!
//! This module is **instrumentation**: it reads framebuffer captures and camera
//! state and returns numbers. It renders nothing, it is not on any draw path,
//! and no shipping `.gd` file calls it. Its only caller is
//! `godot-project/_lodsweep_probe.gd`.
//!
//! **Why it is Rust and not GDScript.** The harness captures a frame per zoom
//! notch across three seeds, two grid sizes and three camera tests — several
//! thousand frames — and metric 1 compares two full-resolution images per
//! frame. A GDScript `Image.get_pixel` loop over that is hours; the same loops
//! here are milliseconds, and — the reason that matters more — the metric
//! *definitions* become unit-testable, so the thresholds every later LOD
//! milestone is graded against are pinned by `cargo test` rather than by a
//! probe nobody re-runs.
//!
//! ## The five metrics, as the scope defines them
//!
//! 1. **Temporal discontinuity `T_i`** — [`temporal_discontinuity`]. Warp frame
//!    *i* into frame *i+1* by the known camera transform (exact for a top-down
//!    2D camera: `screen = camera.position + local · zoom`), then mean
//!    `|ΔL*|` over the overlap. A **pop** is [`pops`]: `T_i` above
//!    [`POP_MEDIAN_FACTOR`] × its trailing [`POP_WINDOW`]-frame median *and*
//!    above [`POP_FLOOR_LSTAR`] L\* units.
//! 2. **Seam ratio** — [`seam_ratio`]. The tile-boundary column difference over
//!    the mean of its eight neighbouring columns.
//! 3. **Hole pixels** — [`hole_pixels`]. Pixels inside the map rect covered by
//!    neither a live tile nor a fallback.
//! 4. **Detail per screen pixel** — [`detail_per_pixel`]. `_mapsharp_probe.gd`'s
//!    own `_hf_energy`, reproduced statement for statement so the 0.0220 →
//!    0.0017 figures already quoted in `LOD_DETAIL_SCOPE.md` stay comparable.
//! 5. **Timing** — [`stats`], median with min..max. The samples are taken in the
//!    probe (they are wall-clock readings, not pixels); only the summary lives
//!    here, so no point estimate can be quoted by accident.
//!
//! ## Two deliberate unit choices, stated because they differ
//!
//! Metrics 1 and 2 are in **CIE L\***, because the scope's own pop threshold is
//! *"1.0 L\* units"* — a quantity that only exists in that space. Metric 4 is in
//! the **non-linear sRGB luma** `_hf_energy` uses, because the scope says it
//! *reuses* that statistic and its recorded baseline is in those units. Mixing
//! the two would silently re-base the one number this milestone exists to
//! preserve.
//!
//! ## Absence is absent
//!
//! Every entry point returns `ok: false` with a `reason` rather than a
//! plausible zero when it cannot measure — an empty overlap, a frame with no
//! tile boundaries on screen, a series too short to judge. `MISTAKES.md`:
//! *"never encode 'no value' as a plausible value"*. A `0.0` seam ratio and a
//! frame with no seams to measure are not the same fact, and the second must
//! not be able to pass the first's threshold.

use godot::classes::{IRefCounted, RefCounted};
use godot::prelude::*;

// ── The constants the thresholds are made of ────────────────────────────────
//
// Every one of these is a literal from `LOD_DETAIL_SCOPE.md` §LOD-D0 except
// where the doc comment says otherwise. They are `pub` so the tests can mutate
// them in a build and watch a test go red (`MISTAKES.md`: a test that pins a
// constant must assert a literal, never the constant against itself).

/// A pop is `T_i` above this multiple of its trailing median. Scope: *"`T_i >
/// 3 ×` its rolling 15-frame median"*.
pub const POP_MEDIAN_FACTOR: f64 = 3.0;

/// …**and** above this many L\* units. Scope: *"and `T_i > 1.0` L\* units"*.
/// The conjunction matters: a still frame's median is near zero, so the
/// multiple alone fires on noise.
pub const POP_FLOOR_LSTAR: f64 = 1.0;

/// The rolling median's window, in frames. Scope: *"its rolling 15-frame
/// median"*.
///
/// **Trailing, not centred** — the 15 frames *before* `i`. Judgment call,
/// recorded rather than hidden: a centred window would let the frames *after* a
/// pop raise the bar the pop is judged against, and a run of two adjacent pops
/// would then hide each other. The cost is that the first [`POP_WINDOW`] frames
/// of every sweep carry no verdict at all, which [`pops`] reports as `unjudged`
/// instead of silently calling them clean.
pub const POP_WINDOW: usize = 15;

/// Columns either side of a tile boundary that form its reference. Four a side
/// is the scope's *"eight neighbouring columns"*.
pub const SEAM_NEIGHBOURS_PER_SIDE: usize = 4;

/// The fewest usable neighbour columns a boundary needs before its ratio is
/// reported. A boundary within four columns of the crop edge, or flanked by
/// another boundary, loses some of its eight; below this it is dropped rather
/// than compared against a thinner reference.
pub const SEAM_MIN_NEIGHBOURS: usize = 4;

/// Pixel stride for metric 1. Every second pixel on each axis — a quarter of
/// the frame, which at 1600×1000 is still 400 000 samples per frame.
///
/// Metric 4 does **not** use this: `_hf_energy` strides rows by two and columns
/// by one, and [`detail_per_pixel`] reproduces that exactly.
pub const SAMPLE_STRIDE: usize = 2;

/// `_hf_energy`'s measurement box, as a fraction of the frame — the central
/// 30 %…70 % on both axes.
pub const DETAIL_BOX_LO: f64 = 0.3;
/// See [`DETAIL_BOX_LO`].
pub const DETAIL_BOX_HI: f64 = 0.7;

/// `_hf_energy`'s row stride.
pub const DETAIL_ROW_STRIDE: usize = 2;

// ── Colour ──────────────────────────────────────────────────────────────────

/// One sRGB byte, linearised — IEC 61966-2-1's own piecewise curve.
fn linearize(c: u8) -> f64 {
    let s = c as f64 / 255.0;
    if s <= 0.04045 {
        s / 12.92
    } else {
        ((s + 0.055) / 1.055).powf(2.4)
    }
}

/// A 256-entry linearisation table. [`temporal_discontinuity`] calls
/// [`linearize`] three times per sampled pixel over millions of pixels, and
/// `powf` is the whole cost of the metric without it.
fn linear_lut() -> [f64; 256] {
    let mut t = [0.0f64; 256];
    for (i, v) in t.iter_mut().enumerate() {
        *v = linearize(i as u8);
    }
    t
}

/// CIE L\* of an sRGB triple, D65, `Yn = 1`. 0 (black) … 100 (white).
///
/// The rational form of the constants (`216/24389`, `24389/27`) is the CIE
/// definition rather than the rounded 0.008856 / 903.3 that most code carries;
/// they differ in the eighth digit, which is below anything this harness
/// measures, and the exact form is used because a threshold in *"L\* units"*
/// should not depend on which rounding its author remembered.
/// Single-pixel entry point, used by this module's tests and kept as the
/// readable definition the plane builder's LUT is an optimisation of. Nothing
/// on the hot path calls it, hence the allow.
#[allow(dead_code)]
pub fn lstar(r: u8, g: u8, b: u8) -> f64 {
    lstar_lut(r, g, b, &linear_lut())
}

fn lstar_lut(r: u8, g: u8, b: u8, lut: &[f64; 256]) -> f64 {
    let y = 0.2126 * lut[r as usize] + 0.7152 * lut[g as usize] + 0.0722 * lut[b as usize];
    if y > 216.0 / 24389.0 {
        116.0 * y.cbrt() - 16.0
    } else {
        (24389.0 / 27.0) * y
    }
}

/// `_mapsharp_probe.gd`'s `_lum` — Rec. 709 weights over **non-linear** sRGB in
/// 0…1, which is what `Color.r` hands a GDScript caller. Not a lightness and
/// not linear luminance; it is reproduced rather than improved so metric 4's
/// recorded baseline keeps meaning what it meant.
fn mapsharp_lum(r: u8, g: u8, b: u8) -> f64 {
    (0.2126 * r as f64 + 0.7152 * g as f64 + 0.0722 * b as f64) / 255.0
}

/// An L\* plane for an RGBA8 buffer, one `f64` per pixel.
fn lstar_plane(rgba: &[u8], w: usize, h: usize) -> Vec<f64> {
    let lut = linear_lut();
    let mut out = vec![0.0f64; w * h];
    for (i, px) in out.iter_mut().enumerate() {
        let o = i * 4;
        *px = lstar_lut(rgba[o], rgba[o + 1], rgba[o + 2], &lut);
    }
    out
}

// ── Metric 1 · temporal discontinuity ───────────────────────────────────────

/// A top-down 2D camera, in the space the capture is cropped to.
///
/// `ViewportHost`'s own contract (`viewport_host.gd::_zoom_at`) is
/// `screen = position + local · zoom`, with `position` measured from
/// `ViewportHost`'s origin — so when the capture is cropped to that node's rect,
/// these are the crop's own pixel coordinates and no further offset exists.
#[derive(Clone, Copy, Debug)]
pub struct Cam {
    pub px: f64,
    pub py: f64,
    pub zoom: f64,
}

/// Bilinear sample of an `f64` plane at `(x, y)` in pixel-centre coordinates.
fn sample_plane(plane: &[f64], w: usize, h: usize, x: f64, y: f64) -> f64 {
    let x0 = x.floor();
    let y0 = y.floor();
    let fx = x - x0;
    let fy = y - y0;
    let x0 = (x0 as isize).clamp(0, w as isize - 1) as usize;
    let y0 = (y0 as isize).clamp(0, h as isize - 1) as usize;
    let x1 = (x0 + 1).min(w - 1);
    let y1 = (y0 + 1).min(h - 1);
    let a = plane[y0 * w + x0];
    let b = plane[y0 * w + x1];
    let c = plane[y1 * w + x0];
    let d = plane[y1 * w + x1];
    let top = a + (b - a) * fx;
    let bot = c + (d - c) * fx;
    top + (bot - top) * fy
}

/// Metric 1. Warps `prev` (taken under camera `a`) into `cur`'s frame (camera
/// `b`) and returns `(mean |ΔL*|, overlap sample count)`.
///
/// The warp is the inverse of the camera's own relation, evaluated per
/// destination pixel: a pixel at crop coordinate `s` in `cur` was, in `prev`, at
/// `a.pos + (s − b.pos) · a.zoom / b.zoom`. **Exact for a top-down camera** —
/// there is no resampling model to choose and no approximation to disclose,
/// which is the reason the scope specifies this metric this way rather than as
/// a plain frame difference. A plain difference would report the zoom itself as
/// a discontinuity and could never distinguish a pop from motion.
///
/// `None` when the two frames do not overlap at all, or when a buffer is the
/// wrong length — never a zero, which would read as "perfectly continuous".
pub fn temporal_discontinuity(
    prev: &[u8],
    cur: &[u8],
    w: usize,
    h: usize,
    a: Cam,
    b: Cam,
) -> Option<(f64, usize)> {
    if w < 2 || h < 2 || prev.len() < w * h * 4 || cur.len() < w * h * 4 {
        return None;
    }
    if !(a.zoom.is_finite() && b.zoom.is_finite()) || a.zoom <= 0.0 || b.zoom <= 0.0 {
        return None;
    }
    let plane = lstar_plane(prev, w, h);
    let lut = linear_lut();
    let k = a.zoom / b.zoom;
    let mut acc = 0.0;
    let mut n = 0usize;
    let mut y = 0usize;
    while y < h {
        let sy = y as f64 + 0.5;
        let py = a.py + (sy - b.py) * k;
        if py >= 0.0 && py <= h as f64 - 1.0 {
            let mut x = 0usize;
            while x < w {
                let sx = x as f64 + 0.5;
                let px = a.px + (sx - b.px) * k;
                if px >= 0.0 && px <= w as f64 - 1.0 {
                    let o = (y * w + x) * 4;
                    let now = lstar_lut(cur[o], cur[o + 1], cur[o + 2], &lut);
                    acc += (sample_plane(&plane, w, h, px, py) - now).abs();
                    n += 1;
                }
                x += SAMPLE_STRIDE;
            }
        }
        y += SAMPLE_STRIDE;
    }
    if n == 0 {
        return None;
    }
    Some((acc / n as f64, n))
}

// ── Metric 1b · the pop rule ────────────────────────────────────────────────

/// Median of a slice, by sorting a copy. `None` for an empty slice.
pub fn median(v: &[f64]) -> Option<f64> {
    if v.is_empty() {
        return None;
    }
    let mut s: Vec<f64> = v.to_vec();
    s.sort_by(|p, q| p.partial_cmp(q).unwrap_or(std::cmp::Ordering::Equal));
    let m = s.len() / 2;
    Some(if s.len() % 2 == 1 { s[m] } else { 0.5 * (s[m - 1] + s[m]) })
}

/// The indices of the frames that count as **pops**, and how many frames could
/// be judged at all.
///
/// Returns `(pop indices, judged, unjudged)`. The first [`POP_WINDOW`] entries
/// of any series are `unjudged`: there is no trailing window to compare them
/// against, and calling them clean would let a defect planted in the first
/// second of a sweep pass silently.
pub fn pops(series: &[f64]) -> (Vec<usize>, usize, usize) {
    let mut hits = Vec::new();
    let mut judged = 0usize;
    for i in 0..series.len() {
        if i < POP_WINDOW {
            continue;
        }
        let Some(med) = median(&series[i - POP_WINDOW..i]) else { continue };
        judged += 1;
        if series[i] > POP_MEDIAN_FACTOR * med && series[i] > POP_FLOOR_LSTAR {
            hits.push(i);
        }
    }
    let unjudged = series.len() - judged;
    (hits, judged, unjudged)
}

// ── Metric 2 · seam ratio ───────────────────────────────────────────────────

/// The mean `|ΔL*|` down column `x` against column `x − 1`.
fn column_step(plane: &[f64], w: usize, h: usize, x: usize) -> f64 {
    let mut acc = 0.0;
    for y in 0..h {
        acc += (plane[y * w + x] - plane[y * w + x - 1]).abs();
    }
    acc / h as f64
}

/// Metric 2. The tile-boundary column difference over the mean of its eight
/// neighbours, per boundary column, summarised as `(max, median, n)`.
///
/// `cols` are crop-local screen columns where a tile edge falls; the probe reads
/// them off the live `Sprite2D` rects rather than recomputing the half-texel
/// inset `viewport_host.gd::_lod_tile_rect` exists to justify.
///
/// A boundary is **dropped** — not scored zero — when it is too near the crop
/// edge to have four neighbours a side, when fewer than
/// [`SEAM_MIN_NEIGHBOURS`] of its eight are free of other boundaries, or when
/// the neighbour mean is zero (a flat region, where any ratio is meaningless).
/// `None` when that leaves nothing to report.
///
/// **The unseamed baseline is 1 only where neighbouring columns are alike.**
/// Terrain is; a synthetic comb is not — a stripe of period 2 puts a full step
/// on every even column and none on the odd ones, so its eight neighbours
/// average half the boundary's step and a *perfectly correct* frame ratios 2.0.
/// The D2/D3 bar of ≤ 1.5 is written against real renders for that reason, and
/// a fixture built to exercise this function has to be built the same way.
pub fn seam_ratio(
    rgba: &[u8],
    w: usize,
    h: usize,
    cols: &[i32],
) -> Option<(f64, f64, usize)> {
    if w < 2 * SEAM_NEIGHBOURS_PER_SIDE + 3 || h == 0 || rgba.len() < w * h * 4 {
        return None;
    }
    let plane = lstar_plane(rgba, w, h);
    let lo = SEAM_NEIGHBOURS_PER_SIDE + 1;
    let hi = w - SEAM_NEIGHBOURS_PER_SIDE - 1;
    let is_boundary = |x: usize| cols.iter().any(|c| *c >= 0 && *c as usize == x);

    let mut ratios: Vec<f64> = Vec::new();
    let mut seen: Vec<usize> = Vec::new();
    for c in cols {
        if *c < 0 {
            continue;
        }
        let b = *c as usize;
        if b < lo || b >= hi || seen.contains(&b) {
            continue;
        }
        seen.push(b);
        let mut refs = Vec::new();
        for d in 1..=SEAM_NEIGHBOURS_PER_SIDE {
            for x in [b - d, b + d] {
                if !is_boundary(x) {
                    refs.push(column_step(&plane, w, h, x));
                }
            }
        }
        if refs.len() < SEAM_MIN_NEIGHBOURS {
            continue;
        }
        let mean: f64 = refs.iter().sum::<f64>() / refs.len() as f64;
        if mean <= 0.0 {
            continue;
        }
        ratios.push(column_step(&plane, w, h, b) / mean);
    }
    if ratios.is_empty() {
        return None;
    }
    let max = ratios.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let med = median(&ratios)?;
    Some((max, med, ratios.len()))
}

// ── Metric 3 · hole pixels ──────────────────────────────────────────────────

/// An axis-aligned rect in crop pixels.
#[derive(Clone, Copy, Debug)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

impl Rect {
    fn covers(&self, px: f64, py: f64) -> bool {
        px >= self.x && px < self.x + self.w && py >= self.y && py < self.y + self.h
    }
}

/// Metric 3. Pixels whose centre lies inside `map` (clipped to the crop) and
/// inside no rect of `covers` — today the live tiles, and from LOD-D3 the live
/// tiles plus their parent fallbacks.
///
/// Returns `(hole pixels, map pixels)`. The denominator is returned because the
/// count alone is unreadable: 4 000 holes is most of a small map rect and a
/// rounding edge on a large one.
///
/// **This is a count of what the LOD layer failed to cover, not of transparent
/// screen.** Until LOD-D3 the base raster still shows under every hole, so the
/// user sees a magnified coarse cell rather than nothing — which is the defect
/// D3 is scheduled to remove, and the reason the metric is defined on coverage
/// rather than on alpha.
pub fn hole_pixels(w: usize, h: usize, map: Rect, covers: &[Rect]) -> (u64, u64) {
    let x0 = map.x.floor().max(0.0) as usize;
    let y0 = map.y.floor().max(0.0) as usize;
    let x1 = ((map.x + map.w).ceil().max(0.0) as usize).min(w);
    let y1 = ((map.y + map.h).ceil().max(0.0) as usize).min(h);
    let mut holes = 0u64;
    let mut inside = 0u64;
    for y in y0..y1 {
        let py = y as f64 + 0.5;
        for x in x0..x1 {
            let px = x as f64 + 0.5;
            if !map.covers(px, py) {
                continue;
            }
            inside += 1;
            if !covers.iter().any(|r| r.covers(px, py)) {
                holes += 1;
            }
        }
    }
    (holes, inside)
}

// ── Metric 4 · detail per screen pixel ──────────────────────────────────────

/// Metric 4. `_mapsharp_probe.gd::_hf_energy`, statement for statement: mean
/// absolute adjacent-pixel luma step along rows, over the central
/// [`DETAIL_BOX_LO`]…[`DETAIL_BOX_HI`] box, rows strided by
/// [`DETAIL_ROW_STRIDE`] and columns not strided at all.
///
/// The asymmetric stride is not a bug being copied for its own sake: it is what
/// produced the `0.0220` and `0.0017` figures `LOD_DETAIL_SCOPE.md` states as
/// the defect this whole plan exists for, and a "tidier" symmetric version would
/// return a different number for the same picture, quietly orphaning the
/// baseline.
///
/// `None` when the box is degenerate.
pub fn detail_per_pixel(rgba: &[u8], w: usize, h: usize) -> Option<f64> {
    if w == 0 || h == 0 || rgba.len() < w * h * 4 {
        return None;
    }
    let x0 = (w as f64 * DETAIL_BOX_LO) as usize;
    let x1 = (w as f64 * DETAIL_BOX_HI) as usize;
    let y0 = (h as f64 * DETAIL_BOX_LO) as usize;
    let y1 = (h as f64 * DETAIL_BOX_HI) as usize;
    if x1 <= x0 + 1 || y1 <= y0 {
        return None;
    }
    let lum_at = |x: usize, y: usize| {
        let o = (y * w + x) * 4;
        mapsharp_lum(rgba[o], rgba[o + 1], rgba[o + 2])
    };
    let mut acc = 0.0;
    let mut n = 0usize;
    let mut y = y0;
    while y < y1 {
        let mut prev = lum_at(x0, y);
        for x in (x0 + 1)..x1 {
            let l = lum_at(x, y);
            acc += (l - prev).abs();
            n += 1;
            prev = l;
        }
        y += DETAIL_ROW_STRIDE;
    }
    if n == 0 {
        return None;
    }
    Some(acc / n as f64)
}

/// `_mapsharp_probe.gd::_mean_abs_diff` — mean absolute **luma** difference
/// between two frames, every second pixel on each axis.
///
/// A second difference function beside [`temporal_discontinuity`], and not a
/// duplicate of it: `LOD_DETAIL_SCOPE.md`'s LOD-D2 acceptance quotes *"hiding
/// the LOD layer at zoom 16 moves mean |dL| by 0.0083"*, and that 0.0083 is in
/// these units, from this function's shape, over the whole window. Measuring it
/// in L\* would produce a number about sixty times larger that could not be
/// compared with the recorded one, and nothing would say so.
///
/// There is no camera warp here because there is nothing to warp: the two
/// frames are the same view with a layer shown and hidden.
pub fn mean_abs_luma_diff(a: &[u8], b: &[u8], w: usize, h: usize) -> Option<f64> {
    if w == 0 || h == 0 || a.len() < w * h * 4 || b.len() < w * h * 4 {
        return None;
    }
    let mut acc = 0.0;
    let mut n = 0usize;
    let mut y = 0usize;
    while y < h {
        let mut x = 0usize;
        while x < w {
            let o = (y * w + x) * 4;
            acc += (mapsharp_lum(a[o], a[o + 1], a[o + 2]) - mapsharp_lum(b[o], b[o + 1], b[o + 2])).abs();
            n += 1;
            x += 2;
        }
        y += 2;
    }
    if n == 0 {
        return None;
    }
    Some(acc / n as f64)
}

// ── Metric 5 · summarising a timing series ──────────────────────────────────

/// Median with min..max — the only shape a timing may be quoted in here
/// (`MISTAKES.md`, *"Quote a timing or a benchmark"*). `None` for an empty
/// series, so a harness that measured nothing cannot print a zero.
pub fn stats(v: &[f64]) -> Option<(f64, f64, f64, usize)> {
    let med = median(v)?;
    let lo = v.iter().cloned().fold(f64::INFINITY, f64::min);
    let hi = v.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    Some((med, lo, hi, v.len()))
}

// ── The `#[func]` surface ───────────────────────────────────────────────────

/// `_lodsweep_probe.gd`'s calculator. A `RefCounted` of its own rather than
/// methods on `WorldGen`: none of this reads world state, and a measurement
/// class the shell never instantiates is easier to show is inert than five more
/// methods on the class the shell holds.
///
/// Registered by the extension, so GDScript reaches it as
/// `ClassDB.instantiate("LodSweepMetrics")` with no editor import pass — see
/// `progress_bridge.rs`'s own header on why a Rust class differs from a
/// GDScript `class_name` here. The probe treats a missing class as
/// PROBE-CANNOT-RUN, which doubles as its stale-`.dll` detector.
#[derive(GodotClass)]
#[class(base=RefCounted)]
struct LodSweepMetrics {
    base: Base<RefCounted>,
}

#[godot_api]
impl IRefCounted for LodSweepMetrics {
    fn init(base: Base<RefCounted>) -> Self {
        Self { base }
    }
}

fn fail(reason: &str) -> VarDictionary {
    vdict! { "ok" => false, "reason" => reason }
}

/// Groups of four `f32` read as rects — the probe's `Sprite2D` rects, flattened
/// so no typed `Array<Rect2>` has to cross the boundary.
fn rects_from(flat: &PackedFloat32Array) -> Vec<Rect> {
    flat.as_slice()
        .chunks_exact(4)
        .map(|c| Rect { x: c[0] as f64, y: c[1] as f64, w: c[2] as f64, h: c[3] as f64 })
        .collect()
}

#[godot_api]
impl LodSweepMetrics {
    /// Metric 1. `{ok, mean_dlstar, overlap_px}`, or `{ok: false, reason}`.
    ///
    /// `prev`/`cur` are RGBA8 buffers of the same `w × h` crop;
    /// `pos_*`/`zoom_*` are `ViewportHost._camera.position` and
    /// `ViewportHost.zoom()` as they stood when each was captured.
    #[func]
    fn temporal(
        &self,
        prev: PackedByteArray,
        cur: PackedByteArray,
        w: i64,
        h: i64,
        pos_prev: Vector2,
        zoom_prev: f64,
        pos_cur: Vector2,
        zoom_cur: f64,
    ) -> VarDictionary {
        if w <= 0 || h <= 0 {
            return fail("degenerate crop");
        }
        let a = Cam { px: pos_prev.x as f64, py: pos_prev.y as f64, zoom: zoom_prev };
        let b = Cam { px: pos_cur.x as f64, py: pos_cur.y as f64, zoom: zoom_cur };
        match temporal_discontinuity(prev.as_slice(), cur.as_slice(), w as usize, h as usize, a, b) {
            Some((m, n)) => vdict! { "ok" => true, "mean_dlstar" => m, "overlap_px" => n as i64 },
            None => fail("no overlap, or a buffer shorter than its stated size"),
        }
    }

    /// Metric 1b. `{ok, indices, judged, unjudged, window, factor, floor}` over
    /// a whole `T_i` series. The rule's own three constants come back with the
    /// verdict so a report cannot quote a pop count beside the wrong threshold.
    #[func]
    fn pops(&self, series: PackedFloat64Array) -> VarDictionary {
        let (hits, judged, unjudged) = pops(series.as_slice());
        let idx: PackedInt32Array = hits.iter().map(|i| *i as i32).collect();
        vdict! {
            "ok" => true,
            "indices" => &idx,
            "judged" => judged as i64,
            "unjudged" => unjudged as i64,
            "window" => POP_WINDOW as i64,
            "factor" => POP_MEDIAN_FACTOR,
            "floor" => POP_FLOOR_LSTAR,
        }
    }

    /// Metric 2. `{ok, max, median, n}` — `n` is how many boundary columns were
    /// scorable, which is not the number handed in.
    #[func]
    fn seam(&self, rgba: PackedByteArray, w: i64, h: i64, cols: PackedInt32Array) -> VarDictionary {
        if w <= 0 || h <= 0 {
            return fail("degenerate crop");
        }
        match seam_ratio(rgba.as_slice(), w as usize, h as usize, cols.as_slice()) {
            Some((max, med, n)) => {
                vdict! { "ok" => true, "max" => max, "median" => med, "n" => n as i64 }
            }
            None => fail("no scorable tile boundary in this frame"),
        }
    }

    /// Metric 3. `{ok, holes, map_px, frac}`. `map_rect` and the flattened
    /// `tiles` are in the same crop-pixel space as the capture.
    #[func]
    fn holes(&self, w: i64, h: i64, map_rect: Rect2, tiles: PackedFloat32Array) -> VarDictionary {
        if w <= 0 || h <= 0 {
            return fail("degenerate crop");
        }
        let map = Rect {
            x: map_rect.position.x as f64,
            y: map_rect.position.y as f64,
            w: map_rect.size.x as f64,
            h: map_rect.size.y as f64,
        };
        let (holes, inside) = hole_pixels(w as usize, h as usize, map, &rects_from(&tiles));
        if inside == 0 {
            return fail("the map rect does not intersect the crop");
        }
        vdict! {
            "ok" => true,
            "holes" => holes as i64,
            "map_px" => inside as i64,
            "frac" => holes as f64 / inside as f64,
        }
    }

    /// Metric 4. `{ok, detail}`.
    #[func]
    fn detail(&self, rgba: PackedByteArray, w: i64, h: i64) -> VarDictionary {
        if w <= 0 || h <= 0 {
            return fail("degenerate crop");
        }
        match detail_per_pixel(rgba.as_slice(), w as usize, h as usize) {
            Some(d) => vdict! { "ok" => true, "detail" => d },
            None => fail("the measurement box is degenerate for this crop"),
        }
    }

    /// `_mean_abs_diff`'s units, for the LOD-D2 bars that are quoted in them.
    /// `{ok, mean_dluma}`.
    #[func]
    fn luma_diff(&self, a: PackedByteArray, b: PackedByteArray, w: i64, h: i64) -> VarDictionary {
        if w <= 0 || h <= 0 {
            return fail("degenerate crop");
        }
        match mean_abs_luma_diff(a.as_slice(), b.as_slice(), w as usize, h as usize) {
            Some(d) => vdict! { "ok" => true, "mean_dluma" => d },
            None => fail("a buffer is shorter than its stated size"),
        }
    }

    /// Metric 5. `{ok, median, min, max, n}`.
    #[func]
    fn stats(&self, series: PackedFloat64Array) -> VarDictionary {
        match stats(series.as_slice()) {
            Some((m, lo, hi, n)) => {
                vdict! { "ok" => true, "median" => m, "min" => lo, "max" => hi, "n" => n as i64 }
            }
            None => fail("empty series"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A solid RGBA8 buffer.
    fn solid(w: usize, h: usize, r: u8, g: u8, b: u8) -> Vec<u8> {
        let mut v = Vec::with_capacity(w * h * 4);
        for _ in 0..w * h {
            v.extend_from_slice(&[r, g, b, 255]);
        }
        v
    }

    /// Vertical stripes of period `p`, alternating `a`/`b` grey.
    fn stripes(w: usize, h: usize, p: usize, a: u8, b: u8) -> Vec<u8> {
        let mut v = Vec::with_capacity(w * h * 4);
        for _y in 0..h {
            for x in 0..w {
                let c = if (x / p) % 2 == 0 { a } else { b };
                v.extend_from_slice(&[c, c, c, 255]);
            }
        }
        v
    }

    // ── colour ──────────────────────────────────────────────────────────────

    /// Literals, not the constants — the two ends of the scale and one interior
    /// point checked against an independent reference (sRGB mid-grey 128 is
    /// L\* ≈ 53.585).
    #[test]
    fn lstar_spans_zero_to_a_hundred() {
        assert!(lstar(0, 0, 0).abs() < 1e-12);
        assert!((lstar(255, 255, 255) - 100.0).abs() < 1e-9);
        assert!((lstar(128, 128, 128) - 53.5850).abs() < 1e-3);
        // Monotone in a grey ramp, which is what makes "L* units" a scale.
        for v in 1u8..=255 {
            assert!(lstar(v, v, v) > lstar(v - 1, v - 1, v - 1));
        }
    }

    /// The linear-segment branch is real, not decoration: below the 0.04045
    /// knee the curve is the straight one, and a build that dropped the branch
    /// would land elsewhere.
    #[test]
    fn lstar_uses_the_linear_segment_near_black() {
        let y = 10.0 / 255.0 / 12.92;
        assert!((lstar(10, 10, 10) - (24389.0 / 27.0) * y).abs() < 1e-9);
    }

    // ── metric 1 ────────────────────────────────────────────────────────────

    /// A frame against itself with an unchanged camera is exactly zero. The
    /// positive control is beside it: a genuinely different frame is not.
    #[test]
    fn identical_frames_have_zero_discontinuity() {
        let w = 64;
        let h = 48;
        let a = solid(w, h, 90, 120, 60);
        let cam = Cam { px: 0.0, py: 0.0, zoom: 1.0 };
        let (d, n) = temporal_discontinuity(&a, &a, w, h, cam, cam).expect("overlap");
        assert_eq!(d, 0.0);
        assert!(n > 0, "the metric must have sampled something");

        // Blue carries 7 % of the luminance, so swapping only the blue channel
        // is a 4.7 L* move — below the pop floor, and a reminder that "the
        // picture changed" and "the lightness changed" are different facts.
        let bluer = solid(w, h, 90, 120, 200);
        let (d_blue, _) = temporal_discontinuity(&a, &bluer, w, h, cam, cam).expect("overlap");
        assert!(d_blue > 1.0 && d_blue < 10.0, "blue-only swing, got {d_blue}");

        let white = solid(w, h, 255, 255, 255);
        let (d2, _) = temporal_discontinuity(&a, &white, w, h, cam, cam).expect("overlap");
        assert!(d2 > 20.0, "a visibly different frame must score, got {d2}");
    }

    /// **The point of the warp.** A pure pan of a textured frame is *not* a
    /// discontinuity, and a plain frame difference would call it one. The warp
    /// takes it back to (near) zero; the same pair compared without the warp
    /// scores heavily.
    #[test]
    fn a_pure_pan_is_not_a_discontinuity() {
        let w = 96;
        let h = 64;
        let src = stripes(w, h, 4, 40, 210);
        // Frame two is frame one shifted 2 px, i.e. the camera moved +2. **Not
        // a whole number of stripe periods** — an 8 px shift of a 4 px stripe
        // reproduces the image exactly, and the unwarped control below would
        // then read 0 and prove nothing. Caught by the control failing.
        const SHIFT: isize = 2;
        let mut moved = vec![0u8; w * h * 4];
        for y in 0..h {
            for x in 0..w {
                let sx = (x as isize - SHIFT).rem_euclid(w as isize) as usize;
                let o = (y * w + x) * 4;
                let s = (y * w + sx) * 4;
                moved[o..o + 4].copy_from_slice(&src[s..s + 4]);
            }
        }
        let a = Cam { px: 0.0, py: 0.0, zoom: 1.0 };
        let b = Cam { px: SHIFT as f64, py: 0.0, zoom: 1.0 };
        let (warped, n) = temporal_discontinuity(&src, &moved, w, h, a, b).expect("overlap");
        let (naive, _) = temporal_discontinuity(&src, &moved, w, h, a, a).expect("overlap");
        assert!(n > 0);
        assert!(warped < 0.5, "a warped pan should vanish, got {warped}");
        assert!(naive > 20.0, "the unwarped comparison should be large, got {naive}");
    }

    /// Absence is absent: two frames whose overlap is empty return `None`, not
    /// a zero that would read as perfect continuity.
    #[test]
    fn no_overlap_is_none_not_zero() {
        let w = 32;
        let h = 32;
        let a = solid(w, h, 10, 10, 10);
        let ca = Cam { px: 0.0, py: 0.0, zoom: 1.0 };
        let cb = Cam { px: 10_000.0, py: 10_000.0, zoom: 1.0 };
        assert!(temporal_discontinuity(&a, &a, w, h, ca, cb).is_none());
        // And a short buffer is refused rather than read past.
        assert!(temporal_discontinuity(&a[..16], &a, w, h, ca, ca).is_none());
    }

    // ── metric 1b · the pop rule ────────────────────────────────────────────

    /// Both clauses are load-bearing, asserted against literals so mutating
    /// either constant turns this red.
    #[test]
    fn the_pop_rule_needs_both_of_its_clauses() {
        // A quiet series with one 30x spike well above 1.0 L*.
        let mut s = vec![0.4f64; 40];
        s[30] = 12.0;
        let (hits, judged, unjudged) = pops(&s);
        assert_eq!(hits, vec![30]);
        assert_eq!(unjudged, 15, "the first window's frames carry no verdict");
        assert_eq!(judged, 25);

        // 30x its median but only 0.006 L* — below the floor, so not a pop.
        let mut tiny = vec![0.0002f64; 40];
        tiny[30] = 0.006;
        assert!(pops(&tiny).0.is_empty(), "the L* floor must veto a tiny spike");

        // 3.0 L* on a busy series whose median is 2.0 — over the floor but only
        // 1.5x the median, so not a pop.
        let mut busy = vec![2.0f64; 40];
        busy[30] = 3.0;
        assert!(pops(&busy).0.is_empty(), "the median multiple must veto it");
    }

    /// The multiple is **3**, bracketed from both sides. A 2.5× spike is not a
    /// pop and a 3.5× one is — which is what makes the constant load-bearing
    /// rather than merely present. (Mutating it to 2.0 survived every other
    /// test in this module; this is the one that kills it.)
    #[test]
    fn the_median_multiple_is_three_not_two_or_four() {
        let mut just_under = vec![1.0f64; 40];
        just_under[30] = 2.5; // > the 1.0 L* floor, 2.5× the median
        assert!(pops(&just_under).0.is_empty(), "2.5× must not be a pop at a 3× rule");

        let mut just_over = vec![1.0f64; 40];
        just_over[30] = 3.5;
        assert_eq!(pops(&just_over).0, vec![30], "3.5× must be a pop at a 3× rule");
    }

    /// The window is trailing, and its length is exactly [`POP_WINDOW`]: a
    /// spike at index 15 is judged, one at 14 is not.
    #[test]
    fn the_window_is_trailing_and_fifteen_long() {
        let mut early = vec![0.3f64; 30];
        early[14] = 40.0;
        assert!(pops(&early).0.is_empty(), "index 14 has no full trailing window");

        let mut edge = vec![0.3f64; 30];
        edge[15] = 40.0;
        assert_eq!(pops(&edge).0, vec![15]);
    }

    #[test]
    fn a_series_shorter_than_the_window_judges_nothing() {
        let (hits, judged, unjudged) = pops(&[5.0, 9.0, 1.0]);
        assert!(hits.is_empty());
        assert_eq!(judged, 0);
        assert_eq!(unjudged, 3, "unjudged frames are reported, not counted clean");
    }

    // ── metric 2 ────────────────────────────────────────────────────────────

    /// On a uniformly textured frame every column steps the same, so the ratio
    /// at any column is 1. A column given a real discontinuity scores far above
    /// it — the metric's teeth.
    /// **Period-1 stripes, deliberately.** A period-*2* stripe has a full step
    /// on even columns and none on odd ones, so its eight neighbours average
    /// half the boundary's own step and an unseamed frame ratios **2.0** — the
    /// first version of this test used one and the metric correctly said so.
    /// The ratio is 1 when neighbouring columns are statistically alike, which
    /// terrain is and a synthetic comb is not; the same caveat is on
    /// [`seam_ratio`] itself.
    #[test]
    fn seam_ratio_is_one_without_a_seam_and_large_with_one() {
        let w = 64;
        let h = 32;
        let even = stripes(w, h, 1, 100, 104);
        let (max, med, n) = seam_ratio(&even, w, h, &[32]).expect("scorable");
        assert_eq!(n, 1);
        assert!((max - 1.0).abs() < 1e-9, "even texture must ratio 1, got {max}");
        assert!((med - 1.0).abs() < 1e-9);

        // Plant a seam: everything from column 32 on is lifted by 60 levels.
        let mut seamed = even.clone();
        for y in 0..h {
            for x in 32..w {
                let o = (y * w + x) * 4;
                for c in 0..3 {
                    seamed[o + c] = seamed[o + c].saturating_add(60);
                }
            }
        }
        let (max2, _, _) = seam_ratio(&seamed, w, h, &[32]).expect("scorable");
        assert!(max2 > 1.5, "a planted seam must exceed the D2/D3 bar of 1.5, got {max2}");
    }

    /// A boundary with no usable neighbours is dropped, and a frame of nothing
    /// but boundaries reports `None` rather than a comfortable 1.0.
    #[test]
    fn unscorable_boundaries_are_dropped_not_scored() {
        let w = 64;
        let h = 16;
        let img = stripes(w, h, 2, 100, 140);
        // Column 1 is inside the four-a-side margin.
        assert!(seam_ratio(&img, w, h, &[1]).is_none());
        // Every column a boundary: no neighbour is free, so nothing is scorable.
        let all: Vec<i32> = (0..w as i32).collect();
        assert!(seam_ratio(&img, w, h, &all).is_none());
        // A flat frame has a zero neighbour mean — a ratio there is undefined.
        assert!(seam_ratio(&solid(w, h, 77, 77, 77), w, h, &[32]).is_none());
    }

    /// **Four a side, exactly.** The margin a boundary needs is the constant
    /// itself, so column 4 is unscorable and column 5 is scorable — and the
    /// same one column in from the right edge. Three a side would move both
    /// answers, and nothing else in this module notices.
    #[test]
    fn the_seam_margin_is_four_columns_a_side() {
        let w = 64;
        let h = 16;
        let img = stripes(w, h, 1, 100, 104);
        assert!(seam_ratio(&img, w, h, &[4]).is_none(), "column 4 has only three columns to its left");
        assert!(seam_ratio(&img, w, h, &[5]).is_some(), "column 5 has exactly four");
        assert!(seam_ratio(&img, w, h, &[w as i32 - 5]).is_none(), "column w-5 has only three to its right");
        assert!(seam_ratio(&img, w, h, &[w as i32 - 6]).is_some(), "column w-6 has exactly four");
    }

    /// **A boundary needs four free neighbours**, and a cluster of boundaries
    /// starves the ones inside it. In a run of six adjacent-ish boundaries only
    /// the two **outermost** keep four free neighbours each; the four inside
    /// have three and are dropped. Relaxing the minimum to one scores all six,
    /// which is what makes this test the one that kills that mutant.
    #[test]
    fn a_starved_boundary_is_dropped_not_scored_on_three_neighbours() {
        let w = 64;
        let h = 16;
        let img = stripes(w, h, 1, 100, 104);
        let cols = [16i32, 12, 13, 14, 15, 17];
        let (_, _, n) = seam_ratio(&img, w, h, &cols).expect("two of them are scorable");
        assert_eq!(n, 2, "only columns 12 and 17 keep four free neighbours");
    }

    // ── metric 3 ────────────────────────────────────────────────────────────

    #[test]
    fn holes_count_only_uncovered_map_pixels() {
        let map = Rect { x: 0.0, y: 0.0, w: 10.0, h: 10.0 };
        let full = Rect { x: 0.0, y: 0.0, w: 10.0, h: 10.0 };
        assert_eq!(hole_pixels(10, 10, map, &[full]), (0, 100));
        assert_eq!(hole_pixels(10, 10, map, &[]), (100, 100));
        // Half covered.
        let half = Rect { x: 0.0, y: 0.0, w: 5.0, h: 10.0 };
        assert_eq!(hole_pixels(10, 10, map, &[half]), (50, 100));
        // Ground outside the map rect is not a hole.
        let small = Rect { x: 2.0, y: 2.0, w: 4.0, h: 4.0 };
        assert_eq!(hole_pixels(10, 10, small, &[]), (16, 16));
    }

    /// The map rect is clipped to the crop, so a map larger than the screen
    /// reports the screen's worth and not a number that includes off-screen
    /// ground the user cannot see a hole in.
    #[test]
    fn the_map_rect_is_clipped_to_the_crop() {
        let huge = Rect { x: -50.0, y: -50.0, w: 500.0, h: 500.0 };
        assert_eq!(hole_pixels(8, 8, huge, &[]), (64, 64));
    }

    // ── metric 4 ────────────────────────────────────────────────────────────

    /// A flat frame has no detail; stripes do; and doubling the stripe period —
    /// exactly what magnifying a fixed source does — halves the statistic. That
    /// last relation is the one the whole milestone is about.
    #[test]
    fn detail_falls_as_a_fixed_source_is_magnified() {
        let w = 200;
        let h = 200;
        assert_eq!(detail_per_pixel(&solid(w, h, 60, 60, 60), w, h), Some(0.0));
        let fine = detail_per_pixel(&stripes(w, h, 1, 40, 200), w, h).expect("value");
        let coarse = detail_per_pixel(&stripes(w, h, 2, 40, 200), w, h).expect("value");
        let coarser = detail_per_pixel(&stripes(w, h, 4, 40, 200), w, h).expect("value");
        assert!(fine > coarse && coarse > coarser);
        assert!((coarse / coarser - 2.0).abs() < 0.1, "period 2 vs 4: {coarse} / {coarser}");
    }

    /// The box really is the central 30–70 %: detail outside it does not count.
    #[test]
    fn detail_reads_only_the_central_box() {
        let w = 100;
        let h = 100;
        let mut img = solid(w, h, 60, 60, 60);
        // Loud noise confined to the left 25 % of every row.
        for y in 0..h {
            for x in 0..25 {
                let o = (y * w + x) * 4;
                let c = if x % 2 == 0 { 0u8 } else { 255u8 };
                img[o] = c;
                img[o + 1] = c;
                img[o + 2] = c;
            }
        }
        assert_eq!(detail_per_pixel(&img, w, h), Some(0.0));

        // …and the far side of the box. Noise confined to columns 70–79 is
        // outside a 0.7 upper edge and inside a 0.8 one.
        let mut right = solid(w, h, 60, 60, 60);
        for y in 0..h {
            for x in 70..80 {
                let o = (y * w + x) * 4;
                let c = if x % 2 == 0 { 0u8 } else { 255u8 };
                right[o] = c;
                right[o + 1] = c;
                right[o + 2] = c;
            }
        }
        assert_eq!(detail_per_pixel(&right, w, h), Some(0.0), "the box stops at 70 %");
    }

    /// **Rows are strided by two, from an even first row.** A frame textured
    /// only on even rows reads at full strength; any other stride mixes in the
    /// blank odd rows and halves it. `_hf_energy`'s asymmetric stride is a
    /// property of the recorded baseline, so it is pinned rather than tidied.
    #[test]
    fn detail_strides_rows_by_two() {
        let w = 100;
        let h = 100;
        let mut img = solid(w, h, 128, 128, 128);
        for y in (0..h).step_by(2) {
            for x in 0..w {
                let o = (y * w + x) * 4;
                let c = if x % 2 == 0 { 0u8 } else { 255u8 };
                img[o] = c;
                img[o + 1] = c;
                img[o + 2] = c;
            }
        }
        // y0 = int(100 * 0.3) = 30, even — so a stride of two reads even rows only.
        let d = detail_per_pixel(&img, w, h).expect("value");
        assert!((d - 1.0).abs() < 1e-12, "even rows only should read 1.0, got {d}");
    }

    /// The exact value for a known picture, independently computed: a period-1
    /// black/white stripe steps `(255-0)/255 = 1.0` every column, so the mean is
    /// 1.0 — which also pins the luma weights' normalisation.
    #[test]
    fn detail_of_a_full_swing_stripe_is_one() {
        let w = 100;
        let h = 100;
        let d = detail_per_pixel(&stripes(w, h, 1, 0, 255), w, h).expect("value");
        assert!((d - 1.0).abs() < 1e-12, "got {d}");
    }

    /// Luma, not L\*: the two differ by more than a constant, and the LOD-D2
    /// bar is quoted in the first. Black against white is exactly 1.0 here and
    /// 100 in L\*, so a harness that mixed them would be off by ~60x on a
    /// mid-grey pair with nothing failing.
    #[test]
    fn luma_diff_is_in_mapsharps_units_not_lstar() {
        let w = 40;
        let h = 40;
        let black = solid(w, h, 0, 0, 0);
        let white = solid(w, h, 255, 255, 255);
        // Not `== 1.0`: the Rec. 709 weights sum to 0.9999999999999999 in f64,
        // which is the third decimal of nothing and still not an equality.
        let full = mean_abs_luma_diff(&black, &white, w, h).expect("value");
        assert!((full - 1.0).abs() < 1e-12, "got {full}");
        assert_eq!(mean_abs_luma_diff(&black, &black, w, h), Some(0.0));

        let mid = solid(w, h, 128, 128, 128);
        let d_luma = mean_abs_luma_diff(&black, &mid, w, h).expect("value");
        assert!((d_luma - 128.0 / 255.0).abs() < 1e-12, "got {d_luma}");
        // The same pair in L* is the 53.6 the colour test pins — two scales.
        assert!(lstar(128, 128, 128) / d_luma > 50.0);

        assert!(mean_abs_luma_diff(&black[..8], &white, w, h).is_none());
    }

    // ── metric 5 ────────────────────────────────────────────────────────────

    #[test]
    fn stats_are_median_min_max_and_a_count() {
        assert_eq!(stats(&[]), None);
        let (m, lo, hi, n) = stats(&[3.0, 1.0, 2.0, 100.0]).expect("value");
        assert_eq!((m, lo, hi, n), (2.5, 1.0, 100.0, 4));
        assert_eq!(median(&[5.0]), Some(5.0));
        assert_eq!(median(&[]), None);
    }
}
