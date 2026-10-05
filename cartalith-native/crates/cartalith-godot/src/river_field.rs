//! **RIM-1: the river as water in the map's own per-pixel field**
//! (`RIVERS_IN_MAP_SCOPE.md` RIM-1/RIM-3/RIM-6, Ruling BU, 2026-09-28).
//!
//! RV-2 draws every river as a vector stroke over the finished map, in screen
//! pixels (`river_stroke.rs`, `map_overlay.gd::_draw_rivers_into`). RV-4 gave
//! the coast and every lake shore a different, better mechanism: a field the
//! base map's shader contours at zero (`shore_field.gdshaderinc`), so the
//! water is part of the map's own image with an antialiased sub-cell edge.
//! This module is that mechanism for rivers: [`build`] turns the same drawn
//! centrelines and per-point widths the stroke uses into a **distance field**
//! the shader (`map_shore.gdshader`) reads per screen pixel, so a river is
//! painted into the map exactly as a coast is, with no second draw call.
//!
//! **What a texel holds** (RGBA half-float, [`CHANNELS`] = 4):
//!
//! - `R`: `d`, the distance in grid cells from the texel centre to the nearest
//!   river centreline (a true point-to-segment distance, round caps), or
//!   [`SENTINEL`] beyond the band where no river is near. *Unsigned*: a
//!   signed perpendicular distance interpolates to zero between two rivers
//!   whose nearest sides differ, bridging a gap that is not there.
//! - `G`: `hw`, the half-width in grid cells of the segment `d` was measured
//!   to, at the foot of the perpendicular -- RV-2's width law
//!   (`river_half_width_profile`, settled at confluences), so the taper from
//!   spring to mouth survives. Not scaled by the preset's `river_width`: the
//!   shader does that, as [`river_stroke::river_px_width`] does.
//! - `B`: the order-1 weight, `1` where the nearest segment belongs to a run
//!   whose own order is 1 (a headwater trickle), else `0`. Interpolated, so a
//!   tributary meeting its trunk blends the two zoom rules instead of
//!   punching a hole where a sign flips.
//! - `A`: the plate-frame weight `1 - border_cover` at the texel's own
//!   position (`render::border_cover_f`), `1` across the interior and fading
//!   to `0` under the bare-paper margin the map's raster bakes over its own
//!   outer cells (`render::apply_border`). The shader multiplies the river's
//!   coverage and floodplain by it, so a painted river never lies on the
//!   frame -- the stroke path gets the same result by clipping to
//!   `map_overlay.gd::_interior_rect`. Held in the one four-channel half-float
//!   format every Vulkan implementation can sample; a three-channel one is
//!   widened by the driver anyway.
//!
//! Which segment a texel reports is the one with the least `d - hw * river_width`,
//! the signed distance to its **edge** at the preset's width: a thin stream
//! beside a trunk must not win a texel that is inside the trunk's body.
//!
//! **The shader's coverage law** is [`coverage`], and it is RV-2's stroke law,
//! so the two agree to within the antialiasing band: half-width in screen
//! pixels `max(hw * ppc * wm * river_width, 0.5)` (the 1 px floor,
//! [`river_stroke::MIN_STROKE_PX`]), alpha ramping linearly over the one pixel
//! fringe outside it ([`river_stroke::EDGE_FRINGE_PX`]), the order-1
//! de-emphasis ([`river_stroke::o1_deemphasis`]) on the width and the alpha,
//! and RIM-5's fade ([`river_stroke::o1_distance_fade`]) on the order-1 alpha
//! below [`river_stroke::O1_FADE_FULL_PPC`] pixels per cell -- a headwater stream thins out of a zoomed-out
//! map rather than popping, in this painted path only (the vector stroke, the
//! tiles and the export keep `o1_deemphasis`'s opening look until RIM-7).
//! The union with the shore field's own water coverage is how a river merges
//! into a lake or the sea with no seam: where the water covers a pixel the
//! river adds nothing, and the shore's contour is untouched
//! (`map_shore.gdshader`).
//!
//! **RIM-2: the optional bank outline** ([`bank_coverage`], `river_bank`).
//! A one-pixel line in the style's ink centred on the visible edge, read from
//! the same `R`/`G`/`B`/`A` -- no new channel -- and gated per preset
//! (`TerrainAppearance::river_bank`, 0 = off, the default, running no outline
//! code in the shader at all). It is measured from the very edge the water
//! uses ([`edge_px`]), so the two cannot disagree; it takes the order-1 alpha
//! (RIM-5's fade removes the outline with the stream) and the plate-frame
//! weight `A`; and it is clamped to the field's valid band so it thins to
//! nothing at a zoomed-out view instead of ending on a step. Screen painted
//! path only: tiles and export are RIM-7's.
//!
//! **Resolution and memory.** The field is supersampled by [`field_scale`]
//! (1 or 2 texels a cell per axis) because a bilinear read of an unsigned
//! distance overestimates it by up to half a texel at the centreline, which
//! would thin a one-pixel stream; at two texels a cell that error is a
//! quarter cell. It is capped at [`MAX_TEXELS`] (8 bytes each): a larger grid
//! gets no field and keeps the vector stroke (`WorldGen::river_field_texture`
//! returns `None`), never a coarser guess.
//!
//! Must never read or change anything a simulation reads: it is a function of
//! the drawn geometry (`RiverGeometry`, itself render-only) and the preset's
//! width. Generation is untouched.

use crate::river_stroke::{RiverGeometry, EDGE_FRINGE_PX, FIELD_VIEW_PX, MIN_STROKE_PX};
use rayon::prelude::*;

/// Half-float channels per texel (see the module doc).
pub const CHANNELS: usize = 4;

/// The `R` value of a texel no river reaches. Large enough that a bilinear mix
/// of it with any in-band distance (at most about 12 cells) stays far above
/// every edge the shader tests -- 64 is exact in half-float. A *distance*, not
/// "no river" encoded as a plausible small one (`MISTAKES.md`).
pub const SENTINEL: f32 = 64.0;

/// How far past a river's drawn edge a texel keeps its distance, in cells, at
/// the least. **Labelled judgement**: the shader needs `d` out to the edge
/// plus the antialiasing fringe at the coarsest view the base map is shown at.
/// A fit view puts a `gw`-cell map in at least [`FIELD_VIEW_PX`] pixels, so a
/// pixel is at most `gw / FIELD_VIEW_PX` cells (2.6 at 2048): the 1 px floor
/// and its fringe reach `1.5 * 2.6 = 3.8` cells. 3.5 is that rounded down; a
/// narrower view reads a one-pixel stream slightly thinner at its fringe,
/// never wrong-coloured, as the colour field's own band does
/// (`river_stroke::colour_field_pad_cells`).
pub const BAND_CELLS: f32 = 3.5;

/// The floodplain band beside a river's edge, in cells, is
/// `FLOODPLAIN_BASE_CELLS + FLOODPLAIN_PER_HALF_WIDTH * hw * river_width`
/// (RIM-3, `map_shore.gdshader`'s tint). **Labelled judgement**: a floodplain
/// scales with the channel that built it, and one cell is the least a
/// one-texel-per-cell map can show as a band. The band must fit inside the
/// field's own ([`band_cells`]): [`band_cells`] takes the larger of the two.
pub const FLOODPLAIN_BASE_CELLS: f32 = 1.0;
/// See [`FLOODPLAIN_BASE_CELLS`].
pub const FLOODPLAIN_PER_HALF_WIDTH: f32 = 1.0;

/// The most texels (each [`CHANNELS`] half-floats, 8 bytes) a field may hold.
/// **Labelled judgement**: 12.6 M texels is 100 MB -- one 2048x1311 map at two
/// texels a cell (10.7 M) and one 4096x2622 map at one (10.7 M), the two sizes
/// the owner works at. 8192x5240 would need 344 MB at one texel a cell and is
/// refused ([`field_scale`] `None`): its rivers keep the vector stroke.
pub const MAX_TEXELS: usize = 12_582_912;

/// Rows of texels one worker builds at a time. A labelled judgement: 32 rows of
/// a 4096-wide field are 1 MB of scratch, small beside the segments' own
/// per-chunk list.
const CHUNK_ROWS: usize = 32;

/// Texels per cell per axis for a `gw x gh` grid: the finest of 2 and 1 that
/// fits [`MAX_TEXELS`], or `None` when even 1 does not (the caller keeps the
/// stroke). See the module doc for why finer is better for thin rivers.
pub fn field_scale(gw: usize, gh: usize) -> Option<usize> {
    [2usize, 1].into_iter().find(|&s| gw.saturating_mul(s).saturating_mul(gh.saturating_mul(s)) <= MAX_TEXELS && gw > 0 && gh > 0)
}

/// The band, in cells, a texel keeps its distance over for a river of
/// half-width `hw` cells at the preset's `river_width`: the edge itself, plus
/// the larger of [`BAND_CELLS`] and the floodplain band. One definition for the
/// builder and the tests, so the field can never be narrower than the tint
/// that reads it.
pub fn band_cells(hw: f32, river_width: f32) -> f32 {
    let edge = hw * river_width;
    edge + BAND_CELLS.max(FLOODPLAIN_BASE_CELLS + FLOODPLAIN_PER_HALF_WIDTH * edge)
}

/// One straight piece of a drawn centreline, in river space (a cell's centre
/// is `x + 0.5`), with the half-width at each end.
#[derive(Clone, Copy, Debug)]
struct Seg {
    a: (f32, f32),
    b: (f32, f32),
    hwa: f32,
    hwb: f32,
    o1: f32,
}

impl Seg {
    /// Distance to the segment from `q`, and the half-width at the foot.
    fn nearest(&self, q: (f32, f32)) -> (f32, f32) {
        let (ex, ey) = (self.b.0 - self.a.0, self.b.1 - self.a.1);
        let l2 = ex * ex + ey * ey;
        let t = if l2 > 0.0 { (((q.0 - self.a.0) * ex + (q.1 - self.a.1) * ey) / l2).clamp(0.0, 1.0) } else { 0.0 };
        let (fx, fy) = (self.a.0 + ex * t, self.a.1 + ey * t);
        let d = ((q.0 - fx) * (q.0 - fx) + (q.1 - fy) * (q.1 - fy)).sqrt();
        (d, self.hwa + (self.hwb - self.hwa) * t)
    }
}

/// Every drawn span of the network as straight segments, including the mouth
/// extensions the stroke draws ([`river_stroke::DrawnRun::reach`]): an end
/// that meets water is carried on until a cap of the widest half-extent the
/// coarsest view draws lies wholly on water, so the river's end is never short
/// of the shore's own smooth contour -- the seam the stroke closed the same
/// way. A seam-crossing jump (more than half the map in `x`) is not a reach
/// (`split_river_polylines`' own rule, as `valley_shade::recut_run` has it).
///
/// `geoms` is the network and, after it, RIM-4's delta fans
/// ([`crate::river_delta`]; this field is the painted path's) -- every run of every geometry
/// listed is flattened the same way, in the order given, so a fan's segments
/// follow the network's and a tie still goes to the first segment drawn.
fn segments(geoms: &[&RiverGeometry], gw: usize, river_width: f32) -> Vec<Seg> {
    let s_min = gw as f32 / FIELD_VIEW_PX;
    let mut out = Vec::new();
    for run in geoms.iter().flat_map(|g| g.runs.iter()) {
        let n = run.pts.len();
        if run.widths.len() != n {
            continue;
        }
        let o1 = if run.own_order <= 1 { 1.0 } else { 0.0 };
        let hw: Vec<f32> = run.widths.iter().map(|&w| (w * 0.5).max(0.0)).collect();
        for (pk, &(s0, e0)) in run.pieces.iter().enumerate() {
            let e0 = e0.min(n);
            if e0 < s0 + 2 {
                continue;
            }
            let mut push = |a: (f32, f32), b: (f32, f32), ha: f32, hb: f32| {
                if (b.0 - a.0).abs() <= gw as f32 * 0.5 {
                    out.push(Seg { a, b, hwa: ha, hwb: hb, o1 });
                }
            };
            for k in s0..e0 - 1 {
                push(run.pts[k], run.pts[k + 1], hw[k], hw[k + 1]);
            }
            // The cap a raster draws is the half-width (floored at one pixel)
            // plus the fringe, in cells at the coarsest density.
            let cap = |h: f32| h.max(0.5 * MIN_STROKE_PX * s_min) + EDGE_FRINGE_PX * s_min;
            let (head, tail) = run.reach.get(pk).map_or((None, None), |r| (r.0.as_ref(), r.1.as_ref()));
            if let Some(r) = head {
                let x = r.extra(cap(hw[s0] * river_width));
                if x > 0.0 {
                    let q = (run.pts[s0].0 + r.dir.0 as f32 * x, run.pts[s0].1 + r.dir.1 as f32 * x);
                    push(q, run.pts[s0], hw[s0], hw[s0]);
                }
            }
            if let Some(r) = tail {
                let last = e0 - 1;
                let x = r.extra(cap(hw[last] * river_width));
                if x > 0.0 {
                    let q = (run.pts[last].0 + r.dir.0 as f32 * x, run.pts[last].1 + r.dir.1 as f32 * x);
                    push(run.pts[last], q, hw[last], hw[last]);
                }
            }
        }
    }
    out
}

/// The built field: `w x h` texels of [`CHANNELS`] half-float bit patterns
/// (`render::f16_bits`), row-major, `scale` texels per cell per axis.
pub struct RiverField {
    pub w: usize,
    pub h: usize,
    pub scale: usize,
    /// How many straight segments the network flattened to (mouth extensions
    /// included): the work the build binned, for `WorldGen::river_paint_stats`.
    pub segments: usize,
    /// `w * h * CHANNELS` half-float bit patterns, little-endian when written
    /// out for a texture.
    pub bits: Vec<u16>,
}

/// Builds the field for a `gw x gh` grid at the preset's `river_width`
/// and the plate frame's `frame_cover(x, y)` (`render::border_cover_f` at a
/// continuous cell position; `0` everywhere for a map with no frame), which
/// becomes the `A` channel (the plate-frame weight, `1 - cover`, so a painted
/// river fades under the baked frame as the stroke's own law does). `None` when the grid
/// is over [`MAX_TEXELS`] (see [`field_scale`]) or there is nothing to draw.
///
/// Exact, not approximate: every segment writes the true point-to-segment
/// distance to each texel within its band ([`band_cells`]), keeping per texel
/// the least `d - hw * river_width`. Parallel over bands of [`CHUNK_ROWS`]
/// rows (each segment is listed in the bands its box reaches), so the cost is
/// the segments' own area, never the grid's per segment -- the scope's "needs
/// a spatial index" risk answered by binning, measured in
/// `WorldGen::river_field_stats`.
///
/// Deterministic: ties go to the first segment in draw order, and every band
/// visits its segments in order.
#[cfg_attr(not(test), allow(dead_code))] // RIM-4: the shell calls the `_with` form; this is the fans-off identity the tests compare it to.
pub fn build(geom: &RiverGeometry, gw: usize, gh: usize, river_width: f32, frame_cover: &(dyn Fn(f64, f64) -> f64 + Sync)) -> Option<RiverField> {
    build_with(geom, None, gw, gh, river_width, frame_cover)
}

/// [`build`] over the network **plus** RIM-4's delta fans (`fans`,
/// [`crate::river_delta::delta_fans`]): the fans' runs are flattened after
/// the network's and compete for each texel by the same least-edge rule, so a
/// fan can only ADD river coverage -- never remove or narrow a texel the
/// network alone covers (`river_delta::tests::a_fan_only_adds_coverage_and_only_near_its_mouth`). `fans = None` is
/// [`build`] exactly: the same segments in the same order, hence a
/// bit-identical field -- the off switch's contract
/// (`river_delta::tests::the_off_switch_is_the_plain_field`). Must never be handed fans the
/// other paths do not draw: since RIM-7 the deep-zoom tiles and the export
/// stroke the SAME fans (`river_stroke::rasterize_with`), and one gate
/// (`river_delta::fans_drawn`) decides for all three, so the caller passes
/// what `WorldGen::delta_fans_for` returns and nothing else.
pub fn build_with(
    geom: &RiverGeometry,
    fans: Option<&RiverGeometry>,
    gw: usize,
    gh: usize,
    river_width: f32,
    frame_cover: &(dyn Fn(f64, f64) -> f64 + Sync),
) -> Option<RiverField> {
    let s = field_scale(gw, gh)?;
    let segs = match fans {
        Some(f) => segments(&[geom, f], gw, river_width),
        None => segments(&[geom], gw, river_width),
    };
    if segs.is_empty() {
        return None;
    }
    let (w, h) = (gw * s, gh * s);
    let sf = s as f32;
    let nchunks = h.div_ceil(CHUNK_ROWS);
    let mut bins: Vec<Vec<u32>> = vec![Vec::new(); nchunks];
    let reach_of = |sg: &Seg| band_cells(sg.hwa.max(sg.hwb), river_width);
    for (i, sg) in segs.iter().enumerate() {
        let r = reach_of(sg);
        // Texel `j`'s centre is at `(j + 0.5) / sf`.
        let y0 = (((sg.a.1.min(sg.b.1) - r) * sf - 0.5).floor().max(0.0)) as usize;
        let y1 = ((((sg.a.1.max(sg.b.1) + r) * sf - 0.5).ceil()).max(0.0) as usize).min(h - 1);
        if y0 > y1 {
            continue;
        }
        for c in y0 / CHUNK_ROWS..=y1 / CHUNK_ROWS {
            bins[c].push(i as u32);
        }
    }
    let sentinel_bits = crate::render::f16_bits(SENTINEL);
    let mut bits = vec![0u16; w * h * CHANNELS];
    bits.par_chunks_mut(CHUNK_ROWS * w * CHANNELS).zip(bins.par_iter()).enumerate().for_each(|(c, (out, bin))| {
        let rows = out.len() / (w * CHANNELS);
        let row0 = c * CHUNK_ROWS;
        let mut best = vec![f32::INFINITY; rows * w];
        // d, hw, o1 per texel; `d` starts at the sentinel.
        let mut val = vec![[SENTINEL, 0.0f32, 0.0f32]; rows * w];
        for &si in bin {
            let sg = &segs[si as usize];
            let r = reach_of(sg);
            let x0 = (((sg.a.0.min(sg.b.0) - r) * sf - 0.5).floor().max(0.0)) as usize;
            let x1 = ((((sg.a.0.max(sg.b.0) + r) * sf - 0.5).ceil()).max(0.0) as usize).min(w - 1);
            let y0 = (((sg.a.1.min(sg.b.1) - r) * sf - 0.5).floor().max(0.0)) as usize;
            let y1 = ((((sg.a.1.max(sg.b.1) + r) * sf - 0.5).ceil()).max(0.0) as usize).min(h - 1);
            for j in y0.max(row0)..=y1.min(row0 + rows - 1) {
                let qy = (j as f32 + 0.5) / sf;
                let base = (j - row0) * w;
                for i in x0..=x1 {
                    let (d, hw) = sg.nearest(((i as f32 + 0.5) / sf, qy));
                    if d > band_cells(hw, river_width) {
                        continue;
                    }
                    let e = d - hw * river_width;
                    if e < best[base + i] {
                        best[base + i] = e;
                        val[base + i] = [d, hw, sg.o1];
                    }
                }
            }
        }
        for (k, v) in val.iter().enumerate() {
            let o = k * CHANNELS;
            out[o] = if v[0] == SENTINEL { sentinel_bits } else { crate::render::f16_bits(v[0]) };
            out[o + 1] = crate::render::f16_bits(v[1]);
            out[o + 2] = crate::render::f16_bits(v[2]);
            // Texel `j`'s centre in cell-index space (a cell's centre is its
            // index), the space `border_cover_f` is defined in.
            let (ti, tj) = (k % w, row0 + k / w);
            let cx = (ti as f64 + 0.5) / s as f64 - 0.5;
            let cy = (tj as f64 + 0.5) / s as f64 - 0.5;
            out[o + 3] = crate::render::f16_bits((1.0 - frame_cover(cx, cy)).clamp(0.0, 1.0) as f32);
        }
    });
    Some(RiverField { w, h, scale: s, segments: segs.len(), bits })
}

#[cfg(test)]
/// Half-float bit pattern back to `f32` (the inverse of `render::f16_bits`,
/// for [`RiverField::texel`]). Handles subnormals, infinity and NaN.
pub fn f16_to_f32(b: u16) -> f32 {
    let sign = if b & 0x8000 != 0 { -1.0f32 } else { 1.0 };
    let exp = ((b >> 10) & 0x1f) as i32;
    let man = (b & 0x3ff) as f32;
    match exp {
        0 => sign * man * 2f32.powi(-24),
        0x1f => if man == 0.0 { sign * f32::INFINITY } else { f32::NAN },
        _ => sign * (1.0 + man / 1024.0) * 2f32.powi(exp - 15),
    }
}

#[cfg(test)]
impl RiverField {
    /// The four channels of texel `(i, j)`, clamped to the field.
    pub fn texel(&self, i: i64, j: i64) -> [f32; 4] {
        let i = i.clamp(0, self.w as i64 - 1) as usize;
        let j = j.clamp(0, self.h as i64 - 1) as usize;
        let o = (j * self.w + i) * CHANNELS;
        [f16_to_f32(self.bits[o]), f16_to_f32(self.bits[o + 1]), f16_to_f32(self.bits[o + 2]), f16_to_f32(self.bits[o + 3])]
    }

    /// Bilinear read at river-space position `(x, y)` in cells (a cell's centre
    /// is `x + 0.5`): the GPU's own filtering of the texture
    /// (`map_shore.gdshader`'s `texture(river_field, UV)`), so a test or a
    /// probe measures the number the shader sees.
    pub fn sample(&self, x: f32, y: f32) -> [f32; 4] {
        let sf = self.scale as f32;
        let (px, py) = (x * sf - 0.5, y * sf - 0.5);
        let (fx, fy) = (px.floor(), py.floor());
        let (tx, ty) = (px - fx, py - fy);
        let (i, j) = (fx as i64, fy as i64);
        let (t00, t10, t01, t11) = (self.texel(i, j), self.texel(i + 1, j), self.texel(i, j + 1), self.texel(i + 1, j + 1));
        let mut out = [0.0; 4];
        for c in 0..4 {
            let top = t00[c] + (t10[c] - t00[c]) * tx;
            let bot = t01[c] + (t11[c] - t01[c]) * tx;
            out[c] = top + (bot - top) * ty;
        }
        out
    }
}

#[cfg(test)]
/// **The shader's river coverage law, mirrored** -- `map_shore.gdshader`'s
/// river block, written once more here so a test can pin it against
/// [`river_stroke::rasterize`] and a probe can name the number it should see.
/// `texel` is a bilinear read ([`RiverField::sample`]); `ppc` the screen's
/// pixels per grid cell; `river_width` the preset's multiplier. Returns
/// `(coverage 0..1, alpha multiplier)`: the stroke's own antialiased
/// coverage and the order-1 alpha, whose product is what composites.
///
/// The 1 px floor and the one-pixel fringe are [`river_stroke::river_px_width`]
/// and `river_stroke::stroke_mesh`'s: the stroke is `hw_px` to its inner
/// edge, then alpha falls linearly to zero over [`EDGE_FRINGE_PX`] more.
pub fn coverage(texel: [f32; 4], ppc: f32, river_width: f32) -> (f32, f32) {
    let (hw_px, am) = edge_px(texel, ppc, river_width);
    let d_px = texel[0] * ppc;
    (((hw_px + EDGE_FRINGE_PX - d_px) / EDGE_FRINGE_PX).clamp(0.0, 1.0), am)
}

#[cfg(test)]
/// The two numbers [`coverage`] and [`bank_coverage`] share, so the outline
/// can never be measured from a different edge than the water: the half-width
/// in screen pixels (`hw_px`, with the order-1 narrowing and the 1 px floor)
/// and the order-1 alpha multiplier (`am`, with RIM-5's headwater fade).
fn edge_px(texel: [f32; 4], ppc: f32, river_width: f32) -> (f32, f32) {
    let (wm_o1, am_o1) = crate::river_stroke::o1_deemphasis(ppc);
    let o1 = texel[2].clamp(0.0, 1.0);
    let wm = 1.0 + (wm_o1 - 1.0) * o1;
    // RIM-5: the order-1 share also fades out as the map zooms out.
    let am = 1.0 + (am_o1 * crate::river_stroke::o1_distance_fade(ppc) - 1.0) * o1;
    let hw_px = (texel[1] * ppc * wm * river_width).max(0.5 * MIN_STROKE_PX);
    (hw_px, am)
}

/// **RIM-2: the full-strength width of the bank outline, in screen pixels.**
/// `map_shore.gdshader` keeps its own copy (`BANK_WIDTH_PX`; a test reads it
/// from the shader's text and holds the two equal). **Labelled judgement**: a
/// one-pixel pen line is what an engraved or inked atlas runs along a bank --
/// anything wider reads as a second, darker river rather than an edge, and
/// the line is antialiased over one more pixel (`1 + BANK_WIDTH_PX` px in all).
#[cfg(test)]
pub const BANK_WIDTH_PX: f32 = 1.0;

/// **RIM-2: how much of the field's valid band the outline may not use, in
/// cells.** The field's texels hold the true distance only out to
/// [`band_cells`] past the edge and the [`SENTINEL`] beyond it, and a
/// bilinear read within one texel of that limit mixes the two -- a distance far
/// too large, which would cut the outline off with a visible step. One cell is
/// the width of a texel of the coarser (1 texel a cell) field, the worst case.
/// **Labelled judgement**, mirrored in the shader as `BANK_BAND_MARGIN_CELLS`.
#[cfg(test)]
pub const BANK_BAND_MARGIN_CELLS: f32 = 1.0;

/// **The shader's bank outline law, mirrored** (RIM-2): the outline's weight
/// at a pixel whose bilinear field read is `texel`, with `bank` the preset's
/// opacity for it ([`crate::render::TerrainAppearance::river_bank`], 0 = off).
/// Returns the weight the ink is mixed in by, `0..=bank`.
///
/// The line is centred on the river's visible edge -- the 50 % point of
/// [`coverage`]'s fringe, `hw_px + EDGE_FRINGE_PX / 2` -- with `e` the
/// signed distance from it in screen pixels (positive outside). Its full
/// reach on each side is `BANK_WIDTH_PX / 2 + 0.5` and its weight falls
/// linearly to zero over that, so a line of width 1 is the one-pixel
/// antialiased stroke the rest of the river is. Two clamps, each so the line
/// can never be where it has no business:
///
/// - **inward** it reaches at most `hw_px - 0.5` into the water (0 for the
///   1 px floor stream): a stream with no body gets no ink over its bed, so a
///   thin river is outlined from the outside only and never turned into a
///   dark line;
/// - **outward** it reaches at most to the field's valid band less
///   [`BANK_BAND_MARGIN_CELLS`] (`(hw * river_width + BAND_CELLS - margin) *
///   ppc - hw_px - 0.5` px past the edge; [`BAND_CELLS`] is the band's
///   guaranteed least), so at a zoomed-out view where the band is narrower than a
///   pixel the line thins to nothing instead of ending on a step.
///
/// It takes the order-1 alpha `am` (so RIM-5's headwater fade removes the
/// outline with the water: no orphan outline on a faded stream) and the
/// plate-frame weight `texel[3]` (none on the bare-paper margin) -- the same
/// two factors [`coverage`]'s composite takes.
#[cfg(test)]
pub fn bank_coverage(texel: [f32; 4], ppc: f32, river_width: f32, bank: f32) -> f32 {
    if bank <= 0.0 {
        return 0.0;
    }
    let (hw_px, am) = edge_px(texel, ppc, river_width);
    let d_px = texel[0] * ppc;
    let e = d_px - (hw_px + 0.5 * EDGE_FRINGE_PX);
    let reach = 0.5 * BANK_WIDTH_PX + 0.5;
    let room = ((texel[1] * river_width + BAND_CELLS - BANK_BAND_MARGIN_CELLS) * ppc - hw_px - 0.5 * EDGE_FRINGE_PX).max(0.0);
    let outward = reach.min(room);
    let inward = reach.min((hw_px - 0.5).max(0.0));
    let ring = if e < 0.0 { (inward + e).clamp(0.0, 1.0) } else { (outward - e).clamp(0.0, 1.0) };
    ring * bank.clamp(0.0, 1.0) * am * texel[3]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::river_stroke::{self, DrawnRun};

    /// A straight east-west run along `y`, from `x0` to `x1`, `width` cells
    /// wide throughout, points a quarter cell apart as the render spline's.
    fn run(y: f32, x0: f32, x1: f32, width: f32, own_order: i16) -> DrawnRun {
        let n = ((x1 - x0) * 4.0) as usize + 1;
        let pts: Vec<(f32, f32)> = (0..n).map(|i| (x0 + i as f32 * 0.25, y)).collect();
        DrawnRun {
            widths: vec![width; n],
            colors: vec![[0.1, 0.3, 0.8, 1.0]; n],
            orders: vec![own_order.max(1); n],
            discharge: vec![1.0; n],
            pieces: vec![(0, n)],
            reach: Vec::new(),
            own_order,
            pts,
        }
    }

    fn geom(runs: Vec<DrawnRun>) -> RiverGeometry {
        RiverGeometry { runs }
    }

    /// Protects: the supersampling rule -- 2 texels a cell while the field fits
    /// the budget, 1 above, none above that -- which decides both a thin
    /// stream's accuracy and the memory the field may take.
    #[test]
    fn the_scale_follows_the_texel_budget() {
        assert_eq!(field_scale(2048, 1311), Some(2));
        assert_eq!(field_scale(4096, 2622), Some(1));
        assert_eq!(field_scale(8192, 5240), None);
        assert_eq!(field_scale(0, 10), None);
    }

    /// Protects: the distance is the exact point-to-segment distance, the
    /// width is carried to the texel, and a texel beyond the band reads the
    /// sentinel (never a plausible small distance).
    #[test]
    fn a_straight_river_stores_its_exact_distance_and_width() {
        let g = geom(vec![run(20.0, 4.0, 36.0, 2.0, 3)]);
        let f = build(&g, 48, 40, 1.0, &|_, _| 0.0).expect("a field");
        assert_eq!((f.w, f.h, f.scale), (96, 80, 2));
        // The texel at x = 20.25, y = 20.75: 0.75 cells below the line.
        let t = f.texel(40, 41);
        assert!((t[0] - 0.75).abs() < 2e-3, "distance {t:?}");
        assert!((t[1] - 1.0).abs() < 1e-3, "half-width {t:?}");
        assert_eq!(t[2], 0.0, "not order 1");
        // 10 cells away: out of the band.
        assert_eq!(f.texel(40, 41 + 20)[0], SENTINEL);
        // Past the end of the line by 6 cells: a round cap's distance, then the sentinel.
        assert_eq!(f.texel(2 * 44, 41)[0], SENTINEL);
        let near_end = f.texel(2 * 36 + 3, 40); // 1.5+0.25 beyond x = 36
        assert!(near_end[0] > 1.5 && near_end[0] < 2.1, "{near_end:?}");
    }

    /// Protects: the order-1 weight reaches the field (the zoom rule's input)
    /// and a thin stream beside a trunk loses the texels inside the trunk's
    /// body (the least `d - hw` wins, not the least `d`).
    #[test]
    fn the_wider_river_owns_the_texels_inside_its_body() {
        // A trunk 6 wide on y = 20, a trickle 0.5 wide on y = 21.5 (inside the trunk's edge at y = 23).
        let g = geom(vec![run(21.5, 4.0, 36.0, 0.5, 1), run(20.0, 4.0, 36.0, 6.0, 3)]);
        let f = build(&g, 48, 40, 1.0, &|_, _| 0.0).expect("a field");
        // y = 21.75 is 0.25 from the trickle and 1.75 from the trunk's line.
        let t = f.texel(40, 43);
        assert!((t[1] - 3.0).abs() < 1e-3, "the trunk owns it: {t:?}");
        assert_eq!(t[2], 0.0);
        // Outside the trunk the trickle is the nearest edge: y = 24.25 is 2.75 from it
        // and 4.25 from the trunk's line (1.25 from its edge): the trunk still wins on edge distance.
        let u = f.texel(40, 2 * 24 + 1);
        assert!((u[1] - 3.0).abs() < 1e-3, "{u:?}");
        // A lone trickle reports its own order-1 weight.
        let g1 = geom(vec![run(20.0, 4.0, 36.0, 0.5, 1)]);
        let f1 = build(&g1, 48, 40, 1.0, &|_, _| 0.0).expect("a field");
        assert_eq!(f1.texel(40, 41)[2], 1.0);
    }

    /// Protects: the shader law (`coverage`) paints what RV-2's stroke paints
    /// -- the same screen pixels, within the antialiasing band -- at fit and
    /// deep base-view densities, for a thin and a wide river. This is the
    /// scope's "coverage vs stroke" bar, on a fixture; the probe repeats it on
    /// the owner's world.
    #[test]
    fn the_field_covers_what_the_stroke_covers() {
        let a = crate::render::TerrainAppearance::default();
        for (width, ppc) in [(0.4f32, 0.5f32), (0.4, 2.0), (3.0, 0.5), (3.0, 2.0), (8.0, 1.0)] {
            let g = geom(vec![run(20.3, 4.0, 60.0, width, 3)]);
            let f = build(&g, 64, 40, a.river_width as f32, &|_, _| 0.0).expect("a field");
            let (pw, ph) = ((64.0 * ppc) as usize, (40.0 * ppc) as usize);
            let layer = river_stroke::rasterize(&g, &a, pw, ph, river_stroke::RasterMap { scale: (ppc, ppc), offset: (0.0, 0.0) });
            let (mut stroke_px, mut field_px, mut diff) = (0.0f64, 0.0f64, 0.0f64);
            for py in 0..ph {
                for px in 0..pw {
                    // `RiverLayer` samples pixel `(x, y)` at exactly `(x, y)` (its
                    // `fill_triangles` contract), so the field is read there.
                    let (x, y) = (px as f32 / ppc, py as f32 / ppc);
                    if x < 8.0 || x > 56.0 {
                        continue; // away from the caps, which the two end differently
                    }
                    let s = layer.at(px, py).map_or(0.0, |p| p[3]) as f64;
                    let (c, am) = coverage(f.sample(x, y), ppc, a.river_width as f32);
                    let fv = (c * am) as f64;
                    stroke_px += s;
                    field_px += fv;
                    diff += (s - fv).abs();
                }
            }
            assert!(stroke_px > 5.0, "width {width} ppc {ppc}: the stroke drew nothing");
            // A straight line is the field's best case (measured: totals equal
            // and per-pixel difference under 0.1 px of coverage); the curved,
            // real network is the probe's bar. 2 percent leaves room for the
            // half-float rounding of the stored distance and nothing more.
            let rel = (field_px - stroke_px).abs() / stroke_px;
            assert!(rel < 0.02 && diff / stroke_px < 0.02, "width {width} ppc {ppc}: stroke {stroke_px:.1} field {field_px:.1} ({:.1}%), per-pixel diff {diff:.1}", rel * 100.0);
        }
    }

    /// Protects: RIM-5 in the painted path -- a headwater (order-1) river's
    /// alpha is 0 at and below `O1_GONE_PPC`, the opening 0.4 from
    /// `O1_FADE_FULL_PPC` up, continuous between; a higher-order river is untouched at
    /// every density; and the fade is the order-1 SHARE (a half-weight texel
    /// fades half as far), never a threshold that pops a stream in.
    #[test]
    fn a_headwater_stream_fades_out_as_the_map_zooms_out() {
        let a = crate::render::TerrainAppearance::default();
        let rw = a.river_width as f32;
        let head = build(&geom(vec![run(20.0, 4.0, 60.0, 0.6, 1)]), 64, 40, rw, &|_, _| 0.0).expect("a field");
        let trunk = build(&geom(vec![run(20.0, 4.0, 60.0, 0.6, 3)]), 64, 40, rw, &|_, _| 0.0).expect("a field");
        let (th, tt) = (head.sample(30.0, 20.0), trunk.sample(30.0, 20.0));
        assert!(th[2] > 0.99 && tt[2] < 0.01, "positive control: the fixtures are order 1 and order 3 ({} {})", th[2], tt[2]);
        let alpha = |t: [f32; 4], ppc: f32| coverage(t, ppc, rw).1;
        assert_eq!(alpha(th, river_stroke::O1_GONE_PPC), 0.0, "a headwater stream is gone at O1_GONE_PPC");
        assert_eq!(alpha(th, 0.1), 0.0, "and below it");
        assert!(alpha(th, 0.5) > 0.0 && alpha(th, 0.5) < 0.4, "mid-ramp is partly faded");
        assert!((alpha(th, river_stroke::O1_FADE_FULL_PPC) - 0.4).abs() < 1e-6, "from O1_FADE_FULL_PPC it is the opening alpha");
        for ppc in [0.1, 0.2, 0.5, 0.8, 3.0] {
            assert_eq!(alpha(tt, ppc), 1.0, "a trunk is never faded (ppc {ppc})");
        }
        let (mut prev, mut max_step) = (0.0f32, 0.0f32);
        for i in 0..=100 {
            let al = alpha(th, i as f32 * 0.01);
            max_step = max_step.max((al - prev).abs());
            prev = al;
        }
        assert!(max_step < 0.02, "a 0.01 pixel-per-cell step moved the alpha by {max_step}");
        let half = [th[0], th[1], 0.5, th[3]];
        let want = 1.0 + (0.4 * crate::river_stroke::o1_distance_fade(0.6) - 1.0) * 0.5;
        assert!((alpha(half, 0.6) - want).abs() < 1e-6, "the fade applies to the order-1 share only");
    }

    /// Protects: the shader's RIM-5 mirror -- `map_shore.gdshader` carries its
    /// own copy of the two fade thresholds (GLSL cannot import Rust), so the
    /// copy is read from the shader's text and held equal to
    /// `river_stroke`'s, and the order-1 alpha line must apply the fade. A
    /// retuned Rust constant with a forgotten shader one fails here.
    #[test]
    fn the_shader_mirrors_the_headwater_fade_thresholds() {
        let src = include_str!("../../../godot-project/shell/map_shore.gdshader");
        let konst = |name: &str| -> f32 {
            let key = format!("const float {name} = ");
            let at = src.find(&key).unwrap_or_else(|| panic!("the shader declares no `{name}`")) + key.len();
            src[at..].split(';').next().unwrap().trim().parse().expect("a float literal")
        };
        assert_eq!(konst("O1_GONE_PPC"), river_stroke::O1_GONE_PPC);
        assert_eq!(konst("O1_FADE_FULL_PPC"), river_stroke::O1_FADE_FULL_PPC);
        assert!(src.contains("smoothstep(O1_GONE_PPC, O1_FADE_FULL_PPC, ppc)"), "the shader's fade is not the smoothstep the Rust law is");
        assert!(src.contains("mix(1.0, dm.y * o1_distance_fade(ppc), o1)"), "the shader's order-1 alpha does not apply the fade");
    }

    /// Protects: a river whose width the preset scales (`river_width`) is
    /// built for that width, so its field edge is the stroke's edge at any
    /// slider value -- the build and the shader read the same `river_width`.
    #[test]
    fn the_band_widens_with_the_preset_width() {
        assert!(band_cells(1.0, 3.0) > band_cells(1.0, 1.0));
        assert!(band_cells(0.2, 1.0) >= BAND_CELLS);
    }

    /// Protects: the `A` channel is `1 - frame_cover` at the texel's own
    /// continuous cell position (so a river under the plate frame fades out
    /// with the frame and is untouched in the interior), and a map with no
    /// frame (cover 0 everywhere) stores 1 across the whole field.
    #[test]
    fn the_alpha_channel_is_the_plate_frame_weight() {
        let g = geom(vec![run(20.0, 2.0, 46.0, 2.0, 2)]);
        let framed = build(&g, 48, 40, 1.0, &|x, _| if x < 10.0 { 1.0 } else { 0.0 }).expect("a field");
        let s = framed.scale as f32;
        let at = |x: f32, y: f32| framed.sample(x * s, y * s)[3];
        assert!(at(4.0, 20.0) < 0.01, "inside the frame margin the river weight is 0");
        assert!((at(30.0, 20.0) - 1.0).abs() < 0.01, "in the interior it is 1");
        let bare = build(&g, 48, 40, 1.0, &|_, _| 0.0).expect("a field");
        assert!((0..bare.h as i64).all(|j| (0..bare.w as i64).all(|i| (bare.texel(i, j)[3] - 1.0).abs() < 1e-3)));
    }

    // ---- RIM-2: the bank outline -------------------------------------------------

    /// One screen column of a straight river through a built field: the pixels
    /// at `py` rows `0..ph`, each reading the field at its own cell position
    /// (the shader's per-pixel read), as `(py, signed e, texel, ring weight)`
    /// with `e` the pixel's distance from the river's visible edge in screen
    /// pixels (negative inside the water). The river lies on `y = 20`.
    fn column(width: f32, ppc: f32, rw: f32, bank: f32) -> Vec<(usize, f32, [f32; 4], f32)> {
        let f = build(&geom(vec![run(20.0, 2.0, 62.0, width, 3)]), 64, 40, rw, &|_, _| 0.0).expect("a field");
        let ph = (40.0 * ppc) as usize;
        (0..ph)
            .map(|py| {
                let t = f.sample(30.0, py as f32 / ppc);
                let (hw_px, _) = edge_px(t, ppc, rw);
                let e = t[0] * ppc - (hw_px + 0.5 * EDGE_FRINGE_PX);
                (py, e, t, bank_coverage(t, ppc, rw, bank))
            })
            .collect()
    }

    /// Protects: the off switch and the outline's presence. With `bank` 0 not
    /// one pixel of any column is inked (so a preset with no opinion draws the
    /// river exactly as before); with `bank` > 0 the same column has ink --
    /// the positive control that fails if the outline is dropped, and the peak
    /// is bounded by the preset's own strength (a bank of 0.6 never inks
    /// beyond 0.6).
    #[test]
    fn the_outline_is_off_at_zero_and_drawn_above_it() {
        // 0.5 px a cell is left out on purpose: with one sample per pixel the ring
        // there is thinner than the sampling and can fall between rows (phase, not a
        // dropped outline); `the_outline_never_reaches_past_the_fields_valid_band`
        // covers that density with a river whose ring does land on a row.
        for ppc in [1.0f32, 4.0, 16.0] {
            let off = column(3.0, ppc, 1.0, 0.0);
            assert!(off.iter().all(|c| c.3 == 0.0), "ppc {ppc}: bank 0 inked a pixel");
            let on = column(3.0, ppc, 1.0, 0.6);
            let peak = on.iter().map(|c| c.3).fold(0.0f32, f32::max);
            assert!(peak > 0.0, "ppc {ppc}: bank 0.6 drew no outline at all");
            assert!(peak <= 0.6 + 1e-6, "ppc {ppc}: the ring exceeds the preset's strength ({peak})");
        }
    }

    /// Protects: the outline's SHAPE on a river wide enough to have a body
    /// (3 cells wide at 4 px a cell; the edge is measured from the stored
    /// field, not assumed). A one-pixel line centred on each bank:
    /// (1) nothing leaks into the interior (any pixel more than one pixel
    /// inside the edge is untouched) or beyond the line (more than one pixel
    /// outside it is untouched); (2) each bank carries ink -- a hat function
    /// of half-width 1 sampled at unit pixel spacing sums to exactly the
    /// strength whatever its phase, so each bank's total is `bank` (a wider
    /// or narrower line moves it: this is what holds `BANK_WIDTH_PX` to one
    /// pixel); (3) the water bed between the banks is not inked.
    #[test]
    fn the_outline_is_a_one_pixel_line_on_each_bank_and_nowhere_else() {
        let (ppc, bank) = (4.0f32, 1.0f32);
        let col = column(3.0, ppc, 1.0, bank);
        let (mut up, mut down) = (0.0f32, 0.0f32);
        for &(py, e, _, b) in &col {
            if !(-1.0 - 1e-3..=1.0 + 1e-3).contains(&e) {
                assert_eq!(b, 0.0, "py {py}: ink {b} at e = {e} px, beyond the one-pixel line");
            }
            if (py as f32) < 20.0 * ppc {
                up += b;
            } else {
                down += b;
            }
        }
        assert!(col.iter().any(|c| c.1 < -3.0), "positive control: the fixture has water body to leak into");
        for (name, sum) in [("upper", up), ("lower", down)] {
            assert!((sum - bank).abs() < 0.05, "the {name} bank carries {sum} of ink, expected one pixel's worth ({bank})");
        }
    }

    /// Protects: a thin stream is outlined from the OUTSIDE only. A stream at
    /// the 1 px floor has no body to ink: no pixel inside its visible edge
    /// (e < 0) may take any ink, so a one-pixel river is never turned into a
    /// dark line (the failure the inward clamp exists for).
    #[test]
    fn a_floor_width_stream_takes_no_ink_over_its_bed() {
        for ppc in [1.0f32, 2.0, 4.0] {
            let col = column(0.1, ppc, 1.0, 1.0);
            let hw_px = col.iter().map(|c| edge_px(c.2, ppc, 1.0).0).fold(0.0f32, f32::max);
            assert!((hw_px - 0.5).abs() < 1e-3, "positive control: the stream is at the 1 px floor ({hw_px})");
            assert!(col.iter().all(|c| c.1 >= 0.0 || c.3 == 0.0), "ppc {ppc}: ink inside the water of a floor stream");
            assert!(col.iter().any(|c| c.3 > 0.0), "ppc {ppc}: and it still has its outer outline");
        }
    }

    /// Protects: the band clamp. The field's distance is the sentinel beyond
    /// `band_cells`, and a read within a texel of that mixes it in; at a
    /// zoomed-out view the outline must therefore stop short of the band by
    /// `BANK_BAND_MARGIN_CELLS` and thin to nothing rather than end on a step.
    /// For every pixel of a sweep of densities, any ink lies at a stored
    /// distance of at most `band_cells - margin` cells, and a density whose
    /// band cannot hold a pixel (0.2 px a cell or less) draws none.
    #[test]
    fn the_outline_never_reaches_past_the_fields_valid_band() {
        for ppc in [0.1f32, 0.2, 0.3, 0.4, 0.5, 0.8, 1.0, 2.0, 8.0] {
            let col = column(0.8, ppc, 1.0, 1.0);
            let mut ink = 0usize;
            for &(py, _, t, b) in &col {
                if b > 0.0 {
                    ink += 1;
                    let limit = band_cells(t[1], 1.0) - BANK_BAND_MARGIN_CELLS;
                    assert!(t[0] <= limit + 1e-3, "ppc {ppc} py {py}: ink at {} cells, past {limit}", t[0]);
                }
            }
            if ppc <= 0.2 {
                assert_eq!(ink, 0, "ppc {ppc}: the band cannot hold the line, yet {ink} pixels are inked");
            }
        }
        // And at the owner world's fit density the line is thinned, not absent.
        assert!(column(0.8, 0.5, 1.0, 1.0).iter().any(|c| c.3 > 0.0), "no outline at 0.5 px a cell");
    }

    /// The outline's peak weight (at the visible edge) for a fabricated texel:
    /// half-width `hw` cells, order-1 weight `o1`, frame weight `frame`.
    fn peak(hw: f32, o1: f32, frame: f32, ppc: f32, bank: f32) -> f32 {
        let (hw_px, _) = edge_px([0.0, hw, o1, frame], ppc, 1.0);
        bank_coverage([(hw_px + 0.5) / ppc, hw, o1, frame], ppc, 1.0, bank)
    }

    /// Protects: the outline takes the same two factors the water does. RIM-5's
    /// headwater fade removes it with the stream (zero at and below
    /// `O1_GONE_PPC`, the opening 0.4 at `O1_FADE_FULL_PPC`, a trunk untouched),
    /// so a faded stream leaves no orphan outline; and the plate-frame weight
    /// scales it, so none lies on the bare-paper margin.
    #[test]
    fn the_outline_fades_with_the_headwater_and_stops_at_the_frame() {
        // A wide headwater (10 cells half-width) keeps room in the band at 0.2 px/cell,
        // so the zero below is the FADE, not the band clamp.
        let (hw_px, _) = edge_px([0.0, 10.0, 1.0, 1.0], river_stroke::O1_GONE_PPC, 1.0);
        let room = (10.0 + BAND_CELLS - BANK_BAND_MARGIN_CELLS) * river_stroke::O1_GONE_PPC - hw_px - 0.5;
        assert!(room > 0.5, "positive control: the band alone would leave the line room ({room})");
        assert_eq!(peak(10.0, 1.0, 1.0, river_stroke::O1_GONE_PPC, 1.0), 0.0, "a faded headwater keeps no outline");
        assert_eq!(peak(10.0, 1.0, 1.0, 0.1, 1.0), 0.0);
        let at_full = peak(10.0, 1.0, 1.0, river_stroke::O1_FADE_FULL_PPC, 1.0);
        assert!((at_full - 0.4).abs() < 1e-5, "from O1_FADE_FULL_PPC the order-1 alpha 0.4 scales it ({at_full})");
        assert!((peak(10.0, 0.0, 1.0, river_stroke::O1_FADE_FULL_PPC, 1.0) - 1.0).abs() < 1e-5, "a trunk's line is full strength");
        let mid = peak(10.0, 1.0, 1.0, 0.5, 1.0);
        assert!(mid > 0.0 && mid < at_full, "mid-fade is partial ({mid})");
        assert!((peak(10.0, 0.0, 0.5, 2.0, 1.0) - 0.5).abs() < 1e-5, "half frame weight, half the ink");
        assert_eq!(peak(10.0, 0.0, 0.0, 2.0, 1.0), 0.0, "no outline on the frame margin");
    }

    /// Protects: the shader's RIM-2 mirror -- `map_shore.gdshader` keeps its own
    /// copies of the line's width, the band margin and the field's least band
    /// (GLSL cannot import Rust), each read from the shader's text and held
    /// equal to the Rust's; and the composite must be gated on the strength so
    /// OFF runs no outline code (what keeps off identical to before, bar a 1-LSB single-pixel residue measured in 5 of 32 probe frames).
    #[test]
    fn the_shader_mirrors_the_bank_outline_constants() {
        let src = include_str!("../../../godot-project/shell/map_shore.gdshader");
        let konst = |name: &str| -> f32 {
            let key = format!("const float {name} = ");
            let at = src.find(&key).unwrap_or_else(|| panic!("the shader declares no `{name}`")) + key.len();
            src[at..].split(';').next().unwrap().trim().parse().expect("a float literal")
        };
        assert_eq!(konst("BANK_WIDTH_PX"), BANK_WIDTH_PX);
        assert_eq!(konst("BANK_BAND_MARGIN_CELLS"), BANK_BAND_MARGIN_CELLS);
        assert_eq!(konst("FIELD_BAND_CELLS"), BAND_CELLS);
        assert!(src.contains("if (river_bank > 0.0)"), "the shader's outline is not gated on the strength");
        // Two composites (the shore branch and the all-land branch), each gated.
        assert_eq!(src.matches("if (bnk > 0.0)").count(), 2, "each shader composite must be gated on the outline weight");
    }

    /// Protects: the default is a literal zero -- OFF -- not merely whatever
    /// the constant is today, in the shipped default and the reference
    /// appearance alike (a bank outline can never leak into either by a
    /// retuned default), and the tunable's range is the 0..1 strength.
    #[test]
    fn the_outline_defaults_to_off() {
        assert_eq!(crate::render::TerrainAppearance::default().river_bank, 0.0);
        assert_eq!(crate::render::TerrainAppearance::js_reference().river_bank, 0.0);
        let (key, lo, hi, _) = crate::render::TerrainAppearance::TUNABLE.iter().find(|t| t.0 == "river_bank").copied().expect("river_bank is a tunable");
        assert_eq!((key, lo, hi), ("river_bank", 0.0, 1.0));
    }

    /// Protects: `f16_to_f32` inverts `render::f16_bits` over the values the
    /// field stores (so the tests and the probe read what the GPU reads).
    #[test]
    fn the_half_float_round_trips() {
        for v in [0.0f32, 0.25, 0.75, 1.0, 3.5, 11.9, 64.0] {
            let b = crate::render::f16_bits(v);
            assert!((f16_to_f32(b) - v).abs() <= v.abs() * 1e-3 + 1e-6, "{v}");
        }
    }
}
