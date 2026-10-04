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
//! de-emphasis ([`river_stroke::o1_deemphasis`]) on the width and the alpha.
//! The union with the shore field's own water coverage is how a river merges
//! into a lake or the sea with no seam: where the water covers a pixel the
//! river adds nothing, and the shore's contour is untouched
//! (`map_shore.gdshader`).
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
fn segments(geom: &RiverGeometry, gw: usize, river_width: f32) -> Vec<Seg> {
    let s_min = gw as f32 / FIELD_VIEW_PX;
    let mut out = Vec::new();
    for run in &geom.runs {
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
pub fn build(geom: &RiverGeometry, gw: usize, gh: usize, river_width: f32, frame_cover: &(dyn Fn(f64, f64) -> f64 + Sync)) -> Option<RiverField> {
    let s = field_scale(gw, gh)?;
    let segs = segments(geom, gw, river_width);
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
    let (wm_o1, am_o1) = crate::river_stroke::o1_deemphasis(ppc);
    let o1 = texel[2].clamp(0.0, 1.0);
    let wm = 1.0 + (wm_o1 - 1.0) * o1;
    let am = 1.0 + (am_o1 - 1.0) * o1;
    let hw_px = (texel[1] * ppc * wm * river_width).max(0.5 * MIN_STROKE_PX);
    let d_px = texel[0] * ppc;
    (((hw_px + EDGE_FRINGE_PX - d_px) / EDGE_FRINGE_PX).clamp(0.0, 1.0), am)
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
