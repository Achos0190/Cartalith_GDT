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
//! map rather than popping, on every painted path (the vector stroke, which a
//! look without `rivers_as_water` still draws, keeps `o1_deemphasis`'s opening
//! look).
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
//! nothing at a zoomed-out view instead of ending on a step.
//!
//! **RIM-7: one law on every path** (2026-10-05). The screen paints from the
//! texture in the GPU (`map_shore.gdshader`, its own GLSL copy of the law,
//! held equal by the `the_shader_mirrors_*` tests). The deep-zoom tiles and
//! every export paint from [`PaintSource::raster`]: the SAME per-texel
//! evaluator the texture is built by ([`eval_window`] -- the nearest drawn
//! segment by least edge distance, within [`band_cells`]), run exactly at each
//! of their own pixels instead of read back bilinearly, and the SAME per-pixel
//! law ([`pixel_paint`]: [`coverage`] with the order-1 rules and RIM-5's fade,
//! the floodplain weight, [`bank_coverage`]), in the river colour the screen
//! samples (the colour field, `river_stroke::rasterize_colour_field_with`).
//! So a river has one width taper, one colour, one floodplain, one bank line
//! and one headwater fade at fit zoom, past the deep-zoom switch and in a
//! file. Which looks paint is one gate, [`painted`]; any other look keeps the
//! vector stroke on all three paths.
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

/// Where a raster's pixels sit in river space, for [`eval_window`]: pixel
/// `(i, j)` is sampled at `((i - ox) / sx, (j - oy) / sy)` -- the inverse of
/// `river_stroke::RasterMap`'s `raster = offset + point * scale`. The
/// texture's texel `i` (centre `(i + 0.5) / s`) is `sx = s, ox = -0.5`.
#[derive(Clone, Copy, Debug)]
struct Place {
    sx: f32,
    sy: f32,
    ox: f32,
    oy: f32,
}

/// **The one per-texel decision** (RIM-1, shared since RIM-7): for every
/// pixel of the window `win = (x0, y0, w, h)` of a raster of `raster = (W, H)`
/// pixels placed by `pl`, keep the segment of `ids` with the least
/// `d - hw * river_width` -- the signed distance to its EDGE at the preset's
/// width, so a thin stream beside a trunk never wins a pixel inside the
/// trunk's body -- among those whose band ([`band_cells`]) reaches the pixel.
/// `best` (the winning edge distance, `INFINITY` = none) and `val` (`[d, hw,
/// o1]`, `d` = [`SENTINEL`] = none) are the window's, row-major, `w * h`.
///
/// `ids` must be in draw order (ascending): a tie goes to the first, which is
/// what keeps the texture deterministic and the fans behind the network
/// (`build_with`). Each segment visits only its own reach box clipped to the
/// window, so the cost is the segments' area, never the window's per segment.
///
/// Called by [`build_with`] (the screen's texture, a window of whole rows)
/// and [`PaintSource::raster`] (a tile's or an export's own pixels, block by
/// block). Must never be given a different law for the one or the other:
/// that would let a river be wider on a tile than on the screen.
#[allow(clippy::too_many_arguments)]
fn eval_window(segs: &[Seg], ids: &[u32], pl: Place, raster: (usize, usize), win: (usize, usize, usize, usize), river_width: f32, best: &mut [f32], val: &mut [[f32; 3]]) {
    let (w, h) = raster;
    let (wx0, wy0, ww, wh) = win;
    if w == 0 || h == 0 || ww == 0 || wh == 0 {
        return;
    }
    for &si in ids {
        let sg = &segs[si as usize];
        let r = band_cells(sg.hwa.max(sg.hwb), river_width);
        let x0 = (((sg.a.0.min(sg.b.0) - r) * pl.sx + pl.ox).floor().max(0.0)) as usize;
        let x1 = ((((sg.a.0.max(sg.b.0) + r) * pl.sx + pl.ox).ceil()).max(0.0) as usize).min(w - 1);
        let y0 = (((sg.a.1.min(sg.b.1) - r) * pl.sy + pl.oy).floor().max(0.0)) as usize;
        let y1 = ((((sg.a.1.max(sg.b.1) + r) * pl.sy + pl.oy).ceil()).max(0.0) as usize).min(h - 1);
        for j in y0.max(wy0)..=y1.min(wy0 + wh - 1) {
            let qy = (j as f32 - pl.oy) / pl.sy;
            let base = (j - wy0) * ww;
            for i in x0.max(wx0)..=x1.min(wx0 + ww - 1) {
                let (d, hw) = sg.nearest(((i as f32 - pl.ox) / pl.sx, qy));
                if d > band_cells(hw, river_width) {
                    continue;
                }
                let e = d - hw * river_width;
                let k = base + i - wx0;
                if e < best[k] {
                    best[k] = e;
                    val[k] = [d, hw, sg.o1];
                }
            }
        }
    }
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
/// the least `d - hw * river_width` -- [`eval_window`], the decision the tiles
/// and the exports make at their own pixels (RIM-7). Parallel over bands of [`CHUNK_ROWS`]
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
    // Texel `i`'s centre is river-space `(i + 0.5) / s` (see `Place`).
    let pl = Place { sx: sf, sy: sf, ox: -0.5, oy: -0.5 };
    bits.par_chunks_mut(CHUNK_ROWS * w * CHANNELS).zip(bins.par_iter()).enumerate().for_each(|(c, (out, bin))| {
        let rows = out.len() / (w * CHANNELS);
        let row0 = c * CHUNK_ROWS;
        let mut best = vec![f32::INFINITY; rows * w];
        // d, hw, o1 per texel; `d` starts at the sentinel.
        let mut val = vec![[SENTINEL, 0.0f32, 0.0f32]; rows * w];
        eval_window(&segs, bin, pl, (w, h), (0, row0, w, rows), river_width, &mut best, &mut val);
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

/// **RIM-3's floodplain band, the shader's own ramp**: a river narrower than
/// `FLOODPLAIN_EDGE_LO` cells (half-width at the preset's width) has no
/// floodplain, one wider than `FLOODPLAIN_EDGE_HI` the full tint, smooth
/// between (`smoothstep(0.35, 1.0, edge)` in `map_shore.gdshader`). **Labelled
/// judgement**, tuned with the tint on the owner's world (2026-10-04): a
/// trickle cuts no valley floor. Mirrored in the shader's text and held equal
/// by `the_shader_mirrors_the_floodplain_tint`.
pub const FLOODPLAIN_EDGE_LO: f32 = 0.35;
/// See [`FLOODPLAIN_EDGE_LO`].
pub const FLOODPLAIN_EDGE_HI: f32 = 1.0;

/// The floodplain tint's strength: the shader's `floodplain_strength` uniform
/// at its declared default, which the shell never sets (only a probe does, to
/// measure the water alone). Read from the shader's text by
/// `the_shader_mirrors_the_floodplain_tint`, so a shipped default that moved
/// would fail rather than leave the tiles at the old one.
pub const FLOODPLAIN_STRENGTH: f32 = 1.0;

/// GLSL's `smoothstep`: 0 at or below `e0`, 1 at or above `e1`, the cubic
/// `t^2 (3 - 2t)` between. Written out because the law it serves is the
/// shader's, edge cases included (`e0 == e1` divides by zero there too; the
/// callers never pass it).
fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// **RIM-3's floodplain weight at a pixel** -- `map_shore.gdshader`'s `fp`:
/// full inside and at the river's edge, fading to nothing over
/// `FLOODPLAIN_BASE_CELLS + FLOODPLAIN_PER_HALF_WIDTH * edge` cells past it,
/// none on an order-1 stream (`1 - o1`), ramped in with the river's width
/// ([`FLOODPLAIN_EDGE_LO`]..[`FLOODPLAIN_EDGE_HI`]), times the strength and the
/// plate-frame weight `texel[3]`. In cells, so the band is the same ground at
/// every zoom. The water drawn over it hides it where the river covers
/// ([`pixel_paint`]'s caller multiplies by `1 - cov`, as the shader's
/// `mix(floodplain(land, fp), river, rcov)` does).
pub fn floodplain_weight(texel: [f32; 4], river_width: f32) -> f32 {
    let o1 = texel[2].clamp(0.0, 1.0);
    let edge = texel[1] * river_width;
    let e = texel[0] - edge;
    let band = FLOODPLAIN_BASE_CELLS + FLOODPLAIN_PER_HALF_WIDTH * edge;
    (1.0 - smoothstep(0.0, band, e)) * (1.0 - o1) * smoothstep(FLOODPLAIN_EDGE_LO, FLOODPLAIN_EDGE_HI, edge) * FLOODPLAIN_STRENGTH * texel[3]
}

/// What the painted river draws at one pixel ([`pixel_paint`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PixelPaint {
    /// The water's coverage, `0..=1`: [`coverage`] times its order-1 alpha
    /// (with RIM-5's fade) times the plate-frame weight -- the shader's `rcov`.
    pub cov: f32,
    /// The floodplain tint's weight before the water hides it
    /// ([`floodplain_weight`]) -- the shader's `fp`.
    pub floodplain: f32,
    /// The RIM-2 bank line's weight, `0..=bank` ([`bank_coverage`]) -- the
    /// shader's `bnk`; 0 whenever the preset has no outline.
    pub bank: f32,
}

/// **RIM-7: the painted river at one pixel, the screen's law** -- what
/// `map_shore.gdshader` computes from a texel, for the rasters that are not
/// the screen: `texel` the field's four channels at the pixel (`[d, hw, o1,
/// frame weight]`), `ppc` the raster's pixels per cell, `river_width` the
/// preset's width multiplier, `bank` its outline strength (0 = off). One
/// function, so the tile and the export cannot weigh a pixel differently from
/// each other; the shader is held to it by the mirror tests.
pub fn pixel_paint(texel: [f32; 4], ppc: f32, river_width: f32, bank: f32) -> PixelPaint {
    let (c, am) = coverage(texel, ppc, river_width);
    PixelPaint { cov: c * am * texel[3], floodplain: floodplain_weight(texel, river_width), bank: bank_coverage(texel, ppc, river_width, bank) }
}

/// **The one gate: does look `a` paint its rivers on a `gw x gh` map?**
/// (`rivers_as_water` and the smooth shore it merges into, and a grid the
/// field fits -- [`field_scale`]). The screen builds the field exactly then
/// (`WorldGen::build_color_texture`), and since RIM-7 the deep-zoom tiles and
/// the export paint exactly then too ([`PaintSource`]); in every other case
/// all three draw the vector stroke, as before. RIM-4's fans take the same
/// gate (`river_delta::fans_drawn`). Must never be answered differently by
/// the three paths -- that is a river drawn two ways across the zoom switch.
pub fn painted(a: &crate::render::TerrainAppearance, gw: usize, gh: usize) -> bool {
    a.smooth_shores && a.rivers_as_water && field_scale(gw, gh).is_some()
}

/// Side, in cells, of the square bins [`PaintSource`] files its segments in.
/// **Labelled judgement**: a tile's or an export block's window is 32 pixels,
/// 2 to 32 cells across between the deep zooms and a grid-resolution export,
/// so a bin of 8 cells keeps the candidate list near the window's own while
/// the index stays small (a few thousand bins on the owner's world). A wrong
/// value costs time, never a pixel: every segment whose reach box meets the
/// window is found whatever the bin size (`a_raster_at_the_texture_density_is_the_texture`).
const BIN_CELLS: f32 = 8.0;

/// **RIM-7: the painted river for every raster that is not the screen** --
/// the drawn network's segments (with RIM-4's fans, as the screen's field has
/// them) in a spatial index, the screen's river colour field, and the look's
/// width, outline and frame. [`Self::raster`] paints any raster with them: a
/// deep-zoom tile (`lod_bridge::synthesize_tile_rgba_rivers`) or an export
/// rectangle (`export_raster::with_export_rivers`), at that raster's own
/// pixels. Built once per look and network (`WorldGen::river_paint_source`)
/// and shared, read-only, by every worker.
///
/// Must never exist for a look [`painted`] refuses: that look draws the
/// stroke on screen, so a painted tile would be the seam this type exists to
/// remove.
pub struct PaintSource {
    segs: Vec<Seg>,
    river_width: f32,
    gw: usize,
    gh: usize,
    /// Segment ids per bin, row-major `bins_w x bins_h`, each segment filed in
    /// every bin its reach box ([`band_cells`]) touches, in draw order.
    bins: Vec<Vec<u32>>,
    bins_w: usize,
    bins_h: usize,
    /// The screen's river colour field (`river_stroke::rasterize_colour_field_with`,
    /// one pixel per cell, premultiplied, the preset's colour and opacity):
    /// the colour the screen's `river_color` texture is rendered from.
    colour: std::sync::Arc<crate::render::RiverLayer>,
    bank: f32,
    bank_rgb: [f32; 3],
    /// The look, for the plate frame's cover (`render::border_cover_f`).
    appearance: crate::render::TerrainAppearance,
}

impl PaintSource {
    /// Index `geom` and, after it, `fans` (the order [`build_with`] flattens
    /// them in) for look `a` on a `gw x gh` map, with `colour` the river
    /// colour field built from the same geometry and look. `None` when the
    /// look does not paint ([`painted`]) or there is nothing to draw.
    pub fn new(
        geom: &RiverGeometry,
        fans: Option<&RiverGeometry>,
        gw: usize,
        gh: usize,
        a: &crate::render::TerrainAppearance,
        colour: std::sync::Arc<crate::render::RiverLayer>,
    ) -> Option<PaintSource> {
        if !painted(a, gw, gh) {
            return None;
        }
        let river_width = crate::river_stroke::river_width_factor(a);
        let segs = match fans {
            Some(f) => segments(&[geom, f], gw, river_width),
            None => segments(&[geom], gw, river_width),
        };
        if segs.is_empty() {
            return None;
        }
        let bins_w = ((gw as f32 / BIN_CELLS).ceil() as usize).max(1);
        let bins_h = ((gh as f32 / BIN_CELLS).ceil() as usize).max(1);
        let mut bins: Vec<Vec<u32>> = vec![Vec::new(); bins_w * bins_h];
        for (i, sg) in segs.iter().enumerate() {
            let r = band_cells(sg.hwa.max(sg.hwb), river_width);
            let (bx0, bx1) = (bin_of(sg.a.0.min(sg.b.0) - r, bins_w), bin_of(sg.a.0.max(sg.b.0) + r, bins_w));
            let (by0, by1) = (bin_of(sg.a.1.min(sg.b.1) - r, bins_h), bin_of(sg.a.1.max(sg.b.1) + r, bins_h));
            for by in by0..=by1 {
                for bx in bx0..=bx1 {
                    bins[by * bins_w + bx].push(i as u32);
                }
            }
        }
        let ink = |v: f64| (v / 255.0).clamp(0.0, 1.0) as f32;
        Some(PaintSource {
            segs,
            river_width,
            gw,
            gh,
            bins,
            bins_w,
            bins_h,
            colour,
            bank: a.river_bank.clamp(0.0, 1.0) as f32,
            bank_rgb: [ink(a.river_ink_r), ink(a.river_ink_g), ink(a.river_ink_b)],
            appearance: a.clone(),
        })
    }

    /// How many straight segments the network and fans flattened to.
    pub fn segment_count(&self) -> usize {
        self.segs.len()
    }

    /// The segment ids whose bins meet the river-space rectangle
    /// `[x0, x1] x [y0, y1]`, ascending (draw order), each once.
    fn ids_in(&self, x0: f32, y0: f32, x1: f32, y1: f32) -> Vec<u32> {
        let (bx0, bx1) = (bin_of(x0, self.bins_w), bin_of(x1, self.bins_w));
        let (by0, by1) = (bin_of(y0, self.bins_h), bin_of(y1, self.bins_h));
        let mut ids: Vec<u32> = Vec::new();
        for by in by0..=by1 {
            for bx in bx0..=bx1 {
                ids.extend_from_slice(&self.bins[by * self.bins_w + bx]);
            }
        }
        ids.sort_unstable();
        ids.dedup();
        ids
    }

    /// The river colour at river-space `q`: the colour field read bilinearly
    /// (its pixel `x` sits on cell `x`'s centre, river-space `x + 0.5`), as the
    /// screen's linear filter reads the texture rendered from it. Premultiplied
    /// `[r, g, b, a]` on `land_color`'s `0..=255` scale; `None` where the field
    /// holds no river colour.
    fn colour_at(&self, q: (f32, f32)) -> Option<[f32; 4]> {
        let (fx, fy) = (q.0 - 0.5, q.1 - 0.5);
        let (x0, y0) = (fx.floor(), fy.floor());
        let (tx, ty) = (fx - x0, fy - y0);
        let px = |x: f32, y: f32| -> [f32; 4] {
            if x < 0.0 || y < 0.0 {
                return [0.0; 4];
            }
            self.colour.at(x as usize, y as usize).unwrap_or([0.0; 4])
        };
        let (c00, c10, c01, c11) = (px(x0, y0), px(x0 + 1.0, y0), px(x0, y0 + 1.0), px(x0 + 1.0, y0 + 1.0));
        let mut out = [0.0f32; 4];
        for k in 0..4 {
            let top = c00[k] + (c10[k] - c00[k]) * tx;
            let bot = c01[k] + (c11[k] - c01[k]) * tx;
            out[k] = top + (bot - top) * ty;
        }
        (out[3] > 0.0).then_some(out)
    }

    /// One [`crate::render::RIVER_BLOCK`] block `(bx, by)` of a `w x h`
    /// raster placed by `pl`: the block's size and [`eval_window`]'s `best`
    /// and `val` over it, from the segments the index files near it. `None`
    /// when no segment's bin meets the block. The one evaluation
    /// [`Self::raster`] paints from (and the tests read back).
    #[allow(clippy::type_complexity)]
    fn block_texels(&self, pl: Place, w: usize, h: usize, bx: usize, by: usize) -> Option<(usize, usize, Vec<f32>, Vec<[f32; 3]>)> {
        use crate::render::RIVER_BLOCK as B;
        let (wx0, wy0) = (bx * B, by * B);
        if wx0 >= w || wy0 >= h {
            return None;
        }
        let (ww, wh) = (B.min(w - wx0), B.min(h - wy0));
        let q = |i: f32, j: f32| ((i - pl.ox) / pl.sx, (j - pl.oy) / pl.sy);
        let (lo, hi) = (q(wx0 as f32, wy0 as f32), q((wx0 + ww - 1) as f32, (wy0 + wh - 1) as f32));
        let ids = self.ids_in(lo.0, lo.1, hi.0, hi.1);
        if ids.is_empty() {
            return None;
        }
        let mut best = vec![f32::INFINITY; ww * wh];
        let mut val = vec![[SENTINEL, 0.0f32, 0.0f32]; ww * wh];
        eval_window(&self.segs, &ids, pl, (w, h), (wx0, wy0, ww, wh), self.river_width, &mut best, &mut val);
        Some((ww, wh, best, val))
    }

    /// Test access: the winning `[d, hw, o1]` at every pixel of a `w x h`
    /// raster placed by `map`, through [`Self::block_texels`] block by block
    /// exactly as [`Self::raster`] evaluates it; `None` where no band reaches.
    #[cfg(test)]
    pub fn texels(&self, map: crate::river_stroke::RasterMap, w: usize, h: usize) -> Vec<Option<[f32; 3]>> {
        use crate::render::RIVER_BLOCK as B;
        let pl = Place { sx: map.scale.0, sy: map.scale.1, ox: map.offset.0, oy: map.offset.1 };
        let mut out = vec![None; w * h];
        for by in 0..h.div_ceil(B) {
            for bx in 0..w.div_ceil(B) {
                if let Some((ww, wh, best, val)) = self.block_texels(pl, w, h, bx, by) {
                    for ly in 0..wh {
                        for lx in 0..ww {
                            if best[ly * ww + lx].is_finite() {
                                out[(by * B + ly) * w + bx * B + lx] = Some(val[ly * ww + lx]);
                            }
                        }
                    }
                }
            }
        }
        out
    }

    /// **The painted river in a `w x h` raster placed by `map`** -- a tile at
    /// its own resolution or an export rectangle at the export's: per pixel,
    /// [`eval_window`]'s winner (the decision the screen's texture holds per
    /// texel), evaluated exactly at the pixel, then [`pixel_paint`] at the
    /// raster's own pixels per cell. The water goes in the layer's colour
    /// plane (the river colour, [`Self::colour_at`], times its coverage), for
    /// `render::land_color` to composite as it composites a stroke; the
    /// floodplain weight (already reduced by the water's coverage) and the
    /// bank line's go in its post plane (`RiverLayer::post_at`), for the
    /// caller to draw after its finishing pass.
    ///
    /// A pixel no segment's band reaches is never painted -- not a texel at
    /// the [`SENTINEL`] distance, which at a few hundredths of a pixel per
    /// cell would read as coverage. Parallel over [`crate::render::RIVER_BLOCK`]
    /// blocks; deterministic (each block's segments in draw order).
    pub fn raster(&self, map: crate::river_stroke::RasterMap, w: usize, h: usize) -> crate::render::RiverLayer {
        use crate::render::RIVER_BLOCK as B;
        let mut layer = crate::render::RiverLayer::new(w, h);
        layer.set_bank_rgb(self.bank_rgb);
        let pl = Place { sx: map.scale.0, sy: map.scale.1, ox: map.offset.0, oy: map.offset.1 };
        if w == 0 || h == 0 || !(pl.sx > 0.0 && pl.sy > 0.0 && pl.ox.is_finite() && pl.oy.is_finite()) {
            return layer;
        }
        let ppc = map.px_per_cell();
        let (nbx, nby) = (w.div_ceil(B), h.div_ceil(B));
        // One finished block of the raster: its origin cell and the optional per-texel paint and colour rows.
        type Block = (usize, usize, Option<Box<[[f32; 4]]>>, Option<Box<[[f32; 2]]>>);
        let blocks: Vec<Block> = (0..nbx * nby)
            .into_par_iter()
            .filter_map(|bi| {
                let (bx, by) = (bi % nbx, bi / nbx);
                let (wx0, wy0) = (bx * B, by * B);
                let (ww, wh, best, val) = self.block_texels(pl, w, h, bx, by)?;
                let q = |i: f32, j: f32| ((i - pl.ox) / pl.sx, (j - pl.oy) / pl.sy);
                let mut col: Option<Box<[[f32; 4]]>> = None;
                let mut post: Option<Box<[[f32; 2]]>> = None;
                for ly in 0..wh {
                    for lx in 0..ww {
                        let k = ly * ww + lx;
                        if !best[k].is_finite() {
                            continue;
                        }
                        let v = val[k];
                        let p = q((wx0 + lx) as f32, (wy0 + ly) as f32);
                        // The frame weight at the pixel's own cell-index
                        // position, as `build_with` stores it per texel.
                        let fw = (1.0 - crate::render::border_cover_f(&self.appearance, p.0 as f64 - 0.5, p.1 as f64 - 0.5, self.gw, self.gh)).clamp(0.0, 1.0) as f32;
                        let paint = pixel_paint([v[0], v[1], v[2], fw], ppc, self.river_width, self.bank);
                        let li = ly * B + lx;
                        if paint.cov > 0.0 {
                            if let Some(c) = self.colour_at(p) {
                                let rv = [c[0] * paint.cov, c[1] * paint.cov, c[2] * paint.cov, c[3] * paint.cov];
                                if rv[3] > 0.0 {
                                    col.get_or_insert_with(|| vec![[0.0f32; 4]; B * B].into_boxed_slice())[li] = rv;
                                }
                            }
                        }
                        let fp = paint.floodplain * (1.0 - paint.cov.clamp(0.0, 1.0));
                        if fp > 0.0 || paint.bank > 0.0 {
                            post.get_or_insert_with(|| vec![[0.0f32; 2]; B * B].into_boxed_slice())[li] = [fp, paint.bank];
                        }
                    }
                }
                (col.is_some() || post.is_some()).then_some((bx, by, col, post))
            })
            .collect();
        for (bx, by, c, p) in blocks {
            layer.put_block(bx, by, c, p);
        }
        layer
    }
}

/// The bin of river-space coordinate `v` among `n` bins of [`BIN_CELLS`],
/// clamped to the index (a reach past the map's edge files in the edge bin,
/// and a raster past the edge -- a tile's halo -- asks it).
fn bin_of(v: f32, n: usize) -> usize {
    if !(v > 0.0) {
        return 0;
    }
    ((v / BIN_CELLS) as usize).min(n - 1)
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

/// **The river coverage law** -- `map_shore.gdshader`'s river block in Rust.
/// Since RIM-7 it is not only a mirror for the tests: [`pixel_paint`] draws the
/// deep-zoom tiles' and the exports' painted rivers with it. `texel` is the
/// field's four channels at the pixel (a bilinear read on screen,
/// [`RiverField::sample`]; an exact evaluation on a tile or in an export,
/// [`PaintSource::raster`]); `ppc` the raster's pixels per grid cell;
/// `river_width` the preset's multiplier. Returns `(coverage 0..1, alpha
/// multiplier)`: the stroke's own antialiased coverage and the order-1 alpha,
/// whose product (times the plate-frame weight `texel[3]`) is what composites.
///
/// The 1 px floor and the one-pixel fringe are [`river_stroke::river_px_width`]
/// and `river_stroke::stroke_mesh`'s: the stroke is `hw_px` to its inner
/// edge, then alpha falls linearly to zero over [`EDGE_FRINGE_PX`] more.
pub fn coverage(texel: [f32; 4], ppc: f32, river_width: f32) -> (f32, f32) {
    let (hw_px, am) = edge_px(texel, ppc, river_width);
    let d_px = texel[0] * ppc;
    (((hw_px + EDGE_FRINGE_PX - d_px) / EDGE_FRINGE_PX).clamp(0.0, 1.0), am)
}

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
pub const BANK_WIDTH_PX: f32 = 1.0;

/// **RIM-2: how much of the field's valid band the outline may not use, in
/// cells.** The field's texels hold the true distance only out to
/// [`band_cells`] past the edge and the [`SENTINEL`] beyond it, and a
/// bilinear read within one texel of that limit mixes the two -- a distance far
/// too large, which would cut the outline off with a visible step. One cell is
/// the width of a texel of the coarser (1 texel a cell) field, the worst case.
/// **Labelled judgement**, mirrored in the shader as `BANK_BAND_MARGIN_CELLS`.
pub const BANK_BAND_MARGIN_CELLS: f32 = 1.0;

/// **The bank outline law** (RIM-2; `map_shore.gdshader`'s, and since RIM-7
/// the tiles' and the exports' through [`pixel_paint`]): the outline's weight
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

    // ---- RIM-7: one law on every path -------------------------------------------

    /// A run that bends (a sine of amplitude 3 cells), its width ramping from
    /// `w0` to `w1` cells, so a test sees curvature, a taper and joins.
    fn bendy(y0: f32, x0: f32, x1: f32, w0: f32, w1: f32, own_order: i16) -> DrawnRun {
        let mut r = run(y0, x0, x1, w0, own_order);
        let n = r.pts.len();
        for (i, p) in r.pts.iter_mut().enumerate() {
            p.1 = y0 + 3.0 * (p.0 * 0.13).sin();
            r.widths[i] = w0 + (w1 - w0) * i as f32 / n as f32;
        }
        r
    }

    /// The fixture network: a tapering trunk, an order-1 stream beside it and
    /// a second river, on a 100 x 60 map, and one fan off the trunk.
    fn network() -> (RiverGeometry, RiverGeometry) {
        let g = geom(vec![bendy(20.0, 3.0, 90.0, 0.3, 4.0, 3), bendy(31.0, 10.0, 60.0, 0.2, 0.6, 1), bendy(45.5, 2.5, 80.0, 1.0, 1.5, 2)]);
        let f = geom(vec![bendy(25.0, 70.0, 95.0, 0.5, 0.5, 3)]);
        (g, f)
    }

    /// The look every RIM-7 test paints with: the default (which paints its
    /// rivers) with a bank outline on, so all three of the law's outputs move.
    fn look() -> crate::render::TerrainAppearance {
        crate::render::TerrainAppearance { river_bank: 0.6, ..crate::render::TerrainAppearance::default() }
    }

    /// Test helper: builds a `PaintSource` over `g` (and optional fans `f`) for the given look and grid, so each test names only what it varies.
    fn source(g: &RiverGeometry, f: Option<&RiverGeometry>, a: &crate::render::TerrainAppearance, gw: usize, gh: usize) -> PaintSource {
        let colour = std::sync::Arc::new(crate::river_stroke::rasterize_colour_field_with(g, f, a, gw, gh));
        PaintSource::new(g, f, gw, gh, a, colour).expect("the look paints and the network is not empty")
    }

    /// Protects: **the tiles and the export decide "is this pixel river
    /// water" by the very evaluation the screen's texture holds.** A raster
    /// placed exactly on the texture's texels (`RasterMap { scale: s, offset:
    /// -0.5 }`) and evaluated block by block through the spatial index gives,
    /// at every texel, the distance, half-width and order-1 weight
    /// `build_with` stored -- bit for bit in half-float -- and no texel more or
    /// fewer. A bin that missed a segment, a block seam, a different tie-break
    /// or a different band would each break it. Both field densities (2 and 1
    /// texels a cell), the fans included.
    #[test]
    fn a_raster_at_the_texture_density_is_the_texture() {
        let (g, f) = network();
        let a = look();
        let rw = crate::river_stroke::river_width_factor(&a);
        for (gw, gh) in [(100usize, 60usize), (2200, 1500)] {
            let field = build_with(&g, Some(&f), gw, gh, rw, &|_, _| 0.0).expect("a field");
            let src = source(&g, Some(&f), &a, gw, gh);
            let sf = field.scale as f32;
            let map = crate::river_stroke::RasterMap { scale: (sf, sf), offset: (-0.5, -0.5) };
            // The fixture lies in the top-left 100 x 60 cells: compare that window.
            let (w, h) = ((100.0 * sf) as usize, (60.0 * sf) as usize);
            let t = src.texels(map, w, h);
            let (mut hits, mut same) = (0usize, 0usize);
            for j in 0..h {
                for i in 0..w {
                    let o = (j * field.w + i) * CHANNELS;
                    let stored = &field.bits[o..o + 3];
                    match t[j * w + i] {
                        Some(v) => {
                            hits += 1;
                            let b = [crate::render::f16_bits(v[0]), crate::render::f16_bits(v[1]), crate::render::f16_bits(v[2])];
                            assert_eq!(&b[..], stored, "texel ({i}, {j}) of {gw}x{gh}: raster {v:?}");
                            same += 1;
                        }
                        None => assert_eq!(stored[0], crate::render::f16_bits(SENTINEL), "texel ({i}, {j}) of {gw}x{gh}: the field has a river the raster lacks"),
                    }
                }
            }
            assert!(hits > 2000, "positive control: the fixture reaches {hits} texels");
            assert_eq!(hits, same);
        }
    }

    /// Protects: `pixel_paint` is the shader's law term for term -- coverage
    /// times the order-1 alpha times the frame weight, the floodplain weight,
    /// the bank line -- with each term's own literal anchor: full floodplain
    /// at the edge of a 1-cell half-width river, half of it one cell out (the
    /// smoothstep's midpoint over a 2-cell band), none on an order-1 stream
    /// or a river at the narrow end of the ramp (0.35 cells), half at the
    /// ramp's midpoint (0.675 cells), and the frame weight scaling every term.
    #[test]
    fn a_pixel_is_weighed_by_the_shaders_law() {
        // At the edge of a trunk of half-width 1 cell, preset width 1.
        let at_edge = pixel_paint([1.0, 1.0, 0.0, 1.0], 4.0, 1.0, 0.0);
        assert_eq!(at_edge.floodplain, 1.0);
        let one_out = pixel_paint([2.0, 1.0, 0.0, 1.0], 4.0, 1.0, 0.0);
        assert!((one_out.floodplain - 0.5).abs() < 1e-6, "{one_out:?}");
        assert_eq!(pixel_paint([3.0, 1.0, 0.0, 1.0], 4.0, 1.0, 0.0).floodplain, 0.0, "past the band");
        assert_eq!(pixel_paint([1.0, 1.0, 1.0, 1.0], 4.0, 1.0, 0.0).floodplain, 0.0, "an order-1 stream has no floodplain");
        assert_eq!(pixel_paint([0.35, 0.35, 0.0, 1.0], 4.0, 1.0, 0.0).floodplain, 0.0, "a river at the ramp's narrow end has none");
        assert!((pixel_paint([0.675, 0.675, 0.0, 1.0], 4.0, 1.0, 0.0).floodplain - 0.5).abs() < 1e-6, "half at the ramp's midpoint");
        // Coverage: the stroke law (inside = 1, the fringe linear), times the frame.
        let inside = pixel_paint([0.2, 1.0, 0.0, 1.0], 4.0, 1.0, 0.0);
        assert_eq!(inside.cov, 1.0);
        let fringe = pixel_paint([1.125, 1.0, 0.0, 1.0], 4.0, 1.0, 0.0); // 0.5 px past the 4 px edge
        assert!((fringe.cov - 0.5).abs() < 1e-6, "{fringe:?}");
        let framed = pixel_paint([0.2, 1.0, 0.0, 0.5], 4.0, 1.0, 0.6);
        assert!((framed.cov - 0.5).abs() < 1e-6 && (framed.floodplain - 0.5).abs() < 1e-6);
        // The same three numbers the mirrored shader functions give.
        for t in [[0.3f32, 0.8, 0.0, 1.0], [1.4, 0.8, 1.0, 1.0], [2.2, 3.0, 0.3, 0.7]] {
            for ppc in [0.5f32, 1.0, 3.7, 16.0] {
                let p = pixel_paint(t, ppc, 1.3, 0.4);
                let (c, am) = coverage(t, ppc, 1.3);
                assert_eq!(p.cov, c * am * t[3]);
                assert_eq!(p.bank, bank_coverage(t, ppc, 1.3, 0.4));
                assert_eq!(p.floodplain, floodplain_weight(t, 1.3));
            }
        }
        assert_eq!(pixel_paint([0.2, 1.0, 0.0, 1.0], 4.0, 1.0, 0.0).bank, 0.0, "no outline at strength 0");
    }

    /// Protects: the shader's floodplain mirror -- `map_shore.gdshader` keeps
    /// its own copies of the tint (`land * MUL + ADD`), the width ramp, the
    /// band and the strength's default (GLSL cannot import Rust), each read
    /// from the shader's text and held equal to the Rust the tiles and the
    /// export draw with. A tint retuned on one side only fails here.
    #[test]
    fn the_shader_mirrors_the_floodplain_tint() {
        let src = include_str!("../../../godot-project/shell/map_shore.gdshader");
        let (m, d) = (crate::render::FLOODPLAIN_MUL, crate::render::FLOODPLAIN_ADD);
        let tint = format!("land * vec3({:?}, {:?}, {:?}) + vec3({:?}, {:?}, {:?})", m[0], m[1], m[2], d[0], d[1], d[2]);
        assert!(src.contains(&tint), "the shader's tint is not `{tint}`");
        assert!(src.contains(&format!("smoothstep({:?}, {:?}, edge)", FLOODPLAIN_EDGE_LO, FLOODPLAIN_EDGE_HI)), "the shader's width ramp differs");
        assert!(src.contains(&format!("smoothstep(0.0, {:?} + edge, e)", FLOODPLAIN_BASE_CELLS)), "the shader's band base differs");
        assert_eq!(FLOODPLAIN_PER_HALF_WIDTH, 1.0, "the shader's band is `1.0 + edge`: a per-half-width other than 1 is not expressible there");
        assert!(src.contains(&format!("uniform float floodplain_strength = {:?};", FLOODPLAIN_STRENGTH)), "the shader's strength default differs");
    }

    /// Protects: the post-finish stage the tiles and the export draw after
    /// their grade -- the floodplain tint, then the bank line, in the shader's
    /// order. Nothing to draw returns the colour bit for bit; the full tint is
    /// `rgb * MUL + ADD` (a literal: mid grey 0.5 goes to 0.492/0.549/0.474);
    /// the full bank is the ink whatever the tint did first (the bank comes
    /// last); and a pixel's land share scales both (`river_post_px`).
    #[test]
    fn the_post_stage_is_the_floodplain_then_the_bank() {
        use crate::render::{river_post_px, river_post_rgb};
        let g = [0.5, 0.5, 0.5];
        assert_eq!(river_post_rgb(g, 0.0, 0.0, [1.0, 0.0, 0.0]), g);
        let t = river_post_rgb(g, 1.0, 0.0, [1.0, 0.0, 0.0]);
        for (k, want) in [0.492f64, 0.549, 0.474].into_iter().enumerate() {
            assert!((t[k] - want).abs() < 1e-12, "{t:?}");
        }
        assert_eq!(river_post_rgb(g, 1.0, 1.0, [1.0, 0.0, 0.0]), [1.0, 0.0, 0.0], "the bank is drawn after the tint");
        assert_eq!(river_post_px([128, 128, 128], [1.0, 0.0], 0.0, [0.0; 3]), [128, 128, 128], "on water nothing is drawn");
        assert_eq!(river_post_px([128, 128, 128], [0.0, 1.0], 1.0, [0.0, 0.0, 0.0]), [0, 0, 0]);
        assert_eq!(river_post_px([128, 128, 128], [0.0, 1.0], 0.5, [0.0, 0.0, 0.0]), [64, 64, 64], "half land, half the bank");
    }

    /// Protects: the gate. A look paints its rivers exactly when it has
    /// `rivers_as_water` and the smooth shore and the grid fits the field --
    /// the shipped default does, the reference look and a stroke look do not,
    /// and 8192 x 5240 keeps the stroke -- and only then is there a
    /// `PaintSource` (a stroke look gets none, so its tiles and exports stroke).
    #[test]
    fn the_painted_gate_is_the_screens() {
        let a = crate::render::TerrainAppearance::default();
        assert!(painted(&a, 2048, 1311) && painted(&a, 4096, 2622));
        assert!(!painted(&a, 8192, 5240));
        assert!(!painted(&crate::render::TerrainAppearance { rivers_as_water: false, ..a.clone() }, 2048, 1311));
        assert!(!painted(&crate::render::TerrainAppearance { smooth_shores: false, ..a.clone() }, 2048, 1311));
        assert!(!painted(&crate::render::TerrainAppearance::js_reference(), 2048, 1311));
        let (g, _) = network();
        let colour = std::sync::Arc::new(crate::river_stroke::rasterize_colour_field_with(&g, None, &a, 100, 60));
        let stroke_look = crate::render::TerrainAppearance { rivers_as_water: false, ..a.clone() };
        assert!(PaintSource::new(&g, None, 100, 60, &stroke_look, colour.clone()).is_none());
        assert!(PaintSource::new(&g, None, 100, 60, &a, colour.clone()).is_some());
        assert!(PaintSource::new(&geom(vec![]), None, 100, 60, &a, colour).is_none(), "nothing to draw is no source, never an empty one");
    }

    /// Screen-pixel coverage of a straight river under the shader law, read
    /// three ways: `texture` = the stored field read bilinearly (what the GPU
    /// does below the deep-zoom switch), `raster` = `PaintSource::raster`'s
    /// layer (what a tile or an export composites), its water alpha over the
    /// river colour's own, and `stroke` = RV-2's vector stroke
    /// (`river_stroke::rasterize`, exact geometry, the same width law).
    /// Returns their totals over the river's middle.
    fn three_ways(width: f32, ppc: f32) -> (f64, f64, f64) {
        let a = look();
        let rw = crate::river_stroke::river_width_factor(&a);
        let g = geom(vec![run(20.3, 4.0, 60.0, width, 3)]);
        // The look's own plate frame, as `build_color_texture` builds the
        // field and as `PaintSource::raster` weighs each pixel.
        let field = build(&g, 64, 40, rw, &|x, y| crate::render::border_cover_f(&a, x, y, 64, 40)).expect("a field");
        let src = source(&g, None, &a, 64, 40);
        let (pw, ph) = ((64.0 * ppc) as usize, (40.0 * ppc) as usize);
        // Pixel (x, y) sits on river-space (x, y) / ppc, as `both` reads the texture.
        let layer = src.raster(crate::river_stroke::RasterMap { scale: (ppc, ppc), offset: (0.0, 0.0) }, pw, ph);
        let opacity = src.colour_at((30.0, 20.3)).expect("the colour field holds the river")[3] as f64;
        let stroke = crate::river_stroke::rasterize(&g, &a, pw, ph, crate::river_stroke::RasterMap { scale: (ppc, ppc), offset: (0.0, 0.0) });
        let (mut tex, mut ras, mut stk) = (0.0f64, 0.0f64, 0.0f64);
        for py in 0..ph {
            for px in 0..pw {
                let (x, y) = (px as f32 / ppc, py as f32 / ppc);
                // Away from the caps, which the three end differently, and
                // off the plate frame, which the stroke does not fade by.
                if !(8.0..=56.0).contains(&x) || crate::render::border_cover_f(&a, x as f64 - 0.5, y as f64 - 0.5, 64, 40) > 0.0 {
                    continue;
                }
                let p = pixel_paint(field.sample(x, y), ppc, rw, 0.0);
                tex += p.cov as f64;
                ras += layer.at(px, py).map_or(0.0, |c| c[3] as f64 / opacity);
                stk += stroke.at(px, py).map_or(0.0, |c| c[3] as f64);
            }
        }
        (tex, ras, stk)
    }

    /// Protects: **a river is as wide on a tile and in an export as on the
    /// screen.** For a thin and a wide river at the densities the screen's
    /// texture is shown at (0.5, and about 1 px a cell either side of the
    /// deep-zoom switch) the water a raster paints (`PaintSource::raster`,
    /// exact per pixel) totals what the shader paints from the bilinear
    /// texture within 3 % -- the half-float and bilinear error of the
    /// texture, nothing more. Deeper (4, 16 px a cell, tiles and exports only)
    /// the texture's quarter-cell bilinear error is no longer sub-pixel, so
    /// the reference is the exact geometry under the same law, RV-2's stroke:
    /// within 2 % there. A raster with a different width law, floor or
    /// fringe fails either bar.
    #[test]
    fn a_raster_paints_the_water_the_screen_paints() {
        for width in [0.4f32, 3.0] {
            for ppc in [0.5f32, 0.97, 1.07] {
                let (tex, ras, _) = three_ways(width, ppc);
                assert!(tex > 5.0, "positive control: width {width} ppc {ppc} drew nothing");
                let rel = (ras - tex).abs() / tex;
                assert!(rel < 0.03, "width {width} ppc {ppc}: texture {tex:.1} px, raster {ras:.1} px ({:.1} %)", rel * 100.0);
            }
            for ppc in [4.0f32, 16.0] {
                let (_, ras, stk) = three_ways(width, ppc);
                assert!(stk > 5.0, "positive control: width {width} ppc {ppc} stroked nothing");
                let rel = (ras - stk).abs() / stk;
                assert!(rel < 0.02, "width {width} ppc {ppc}: stroke {stk:.1} px, raster {ras:.1} px ({:.1} %)", rel * 100.0);
            }
        }
    }

    /// Protects: the raster's post plane -- the floodplain beside a wide
    /// river and the bank line on its edge are there (positive control), the
    /// floodplain weight is hidden under the water (`fp * (1 - cov)`: zero
    /// where the water is full), the bank is absent with the strength at 0,
    /// and a pixel no band reaches carries nothing at all (no colour, no
    /// post) -- never a sentinel distance read as coverage, which at a
    /// hundredth of a pixel per cell would paint the whole raster.
    #[test]
    fn the_raster_carries_the_tint_and_the_line_and_nothing_off_its_band() {
        let (g, _) = (geom(vec![run(20.0, 4.0, 60.0, 3.0, 3)]), ());
        let a = look();
        let src = source(&g, None, &a, 64, 40);
        let ppc = 4.0f32;
        let layer = src.raster(crate::river_stroke::RasterMap { scale: (ppc, ppc), offset: (0.0, 0.0) }, 256, 160);
        let col = 120; // x = 30 cells
        let mut fp_max = 0.0f32;
        let mut bank_max = 0.0f32;
        for py in 0..160 {
            if let Some(q) = layer.post_at(col, py) {
                fp_max = fp_max.max(q[0]);
                bank_max = bank_max.max(q[1]);
                if let Some(c) = layer.at(col, py) {
                    let cov = c[3] / src.colour_at((30.0, py as f32 / ppc)).map_or(1.0, |k| k[3]);
                    if cov >= 0.999 {
                        assert_eq!(q[0], 0.0, "py {py}: the tint shows through full water");
                    }
                }
            }
        }
        assert!(fp_max > 0.5 && bank_max > 0.1, "the tint ({fp_max}) and the line ({bank_max}) must be there");
        assert!(layer.at(col, 4).is_none() && layer.post_at(col, 4).is_none(), "16 cells away nothing is drawn");
        let plain = PaintSource::new(&g, None, 64, 40, &crate::render::TerrainAppearance { river_bank: 0.0, ..a.clone() }, src.colour.clone()).expect("source");
        let l0 = plain.raster(crate::river_stroke::RasterMap { scale: (ppc, ppc), offset: (0.0, 0.0) }, 256, 160);
        assert!((0..160).all(|py| l0.post_at(col, py).is_none_or(|q| q[1] == 0.0)), "strength 0 draws no line");
        // A raster at 0.02 px a cell, where the sentinel distance would read as
        // coverage (1.5 - 64 * 0.02 = 0.22), on a 1500 x 1000 map with no plate
        // frame and a colour layer that holds a colour EVERYWHERE -- so neither
        // the frame's cover past the edge nor the colour field's own band can
        // hide a wrongly painted pixel: only the pixel over the river may be
        // painted.
        let (bw, bh) = (1500usize, 1000usize);
        let mut everywhere = crate::render::RiverLayer::new(bw, bh);
        everywhere.fill_triangles(&[(-1.0, -1.0), (bw as f32 + 1.0, -1.0), (bw as f32 + 1.0, bh as f32 + 1.0), (-1.0, bh as f32 + 1.0)], &[[0.2, 0.4, 0.9, 1.0]; 4], &[0, 1, 2, 0, 2, 3]);
        let unframed = crate::render::TerrainAppearance { border_width_frac: 0.0, ..a.clone() };
        let wide = PaintSource::new(&g, None, bw, bh, &unframed, std::sync::Arc::new(everywhere)).expect("source");
        // Pixel (0, 0) sits on the river at (30, 20); every other pixel is 50+ cells off.
        let far = wide.raster(crate::river_stroke::RasterMap { scale: (0.02, 0.02), offset: (-0.6, -0.4) }, 64, 64);
        assert!(far.at(0, 0).is_some(), "positive control: the river's own pixel is painted");
        assert!((0..64).all(|y| (0..64).all(|x| (x, y) == (0, 0) || far.at(x, y).is_none())), "a pixel off every band was painted");
    }

    /// Protects: the river colour a raster paints is the colour field's own,
    /// read where the screen's linear filter reads it -- the field's pixel `x`
    /// sits on cell `x`'s centre (river-space `x + 0.5`), so a read exactly
    /// there returns that pixel bit for bit, and halfway to the next pixel
    /// their mean. A half-cell slip (reading `x` at `x`) would blend in the
    /// neighbour and fail. The fixture's colour field varies along the river.
    #[test]
    fn a_raster_takes_its_colour_where_the_screen_does() {
        let a = look();
        let mut r = run(20.0, 4.0, 60.0, 3.0, 3);
        let n = r.colors.len();
        for (i, c) in r.colors.iter_mut().enumerate() {
            *c = [i as f32 / n as f32, 0.3, 1.0 - i as f32 / n as f32, 1.0];
        }
        let g = geom(vec![r]);
        let src = source(&g, None, &a, 64, 40);
        let (x, y) = (30usize, 20usize);
        let here = src.colour.at(x, y).expect("the field holds the river");
        let next = src.colour.at(x + 1, y).expect("and its neighbour");
        assert_ne!(here, next, "premise: the colour varies along the river");
        assert_eq!(src.colour_at((x as f32 + 0.5, y as f32 + 0.5)), Some(here));
        let mid = src.colour_at((x as f32 + 1.0, y as f32 + 0.5)).expect("a colour");
        for k in 0..4 {
            assert!((mid[k] - 0.5 * (here[k] + next[k])).abs() < 1e-4, "{mid:?} is not the mean of {here:?} and {next:?}");
        }
    }

    /// Protects: **the seam at the deep-zoom switch.** The same bending river
    /// rastered at the densities either side of the switch on the owner's
    /// window (0.97 and 1.07 px a cell) puts its water where the river is:
    /// in every column, the coverage-weighted centre lies within a tenth of a
    /// cell of the drawn centreline (`20 + 3 sin(0.13 x)`) at both densities,
    /// and the mean water per column in cells (the width) agrees to 10 %. A
    /// raster misplaced by half a pixel, or a different width rule on one
    /// side, fails.
    #[test]
    fn a_river_does_not_jump_at_the_deep_zoom_switch() {
        let a = look();
        let g = geom(vec![bendy(20.0, 3.0, 60.0, 1.5, 1.5, 3)]);
        let src = source(&g, None, &a, 64, 40);
        let sweep = |ppc: f32| -> f64 {
            let (w, h) = ((64.0 * ppc) as usize, (40.0 * ppc) as usize);
            let layer = src.raster(crate::river_stroke::RasterMap { scale: (ppc, ppc), offset: (0.0, 0.0) }, w, h);
            let (mut width, mut n) = (0.0f64, 0usize);
            for px in 0..w {
                let x = px as f64 / ppc as f64;
                if !(10.0..=50.0).contains(&x) {
                    continue;
                }
                let (mut m, mut my) = (0.0f64, 0.0f64);
                for py in 0..h {
                    let c = layer.at(px, py).map_or(0.0, |c| c[3] as f64);
                    m += c;
                    my += c * py as f64;
                }
                assert!(m > 0.5, "ppc {ppc} x {x:.2}: no water in the column");
                let (centre, want) = (my / m / ppc as f64, 20.0 + 3.0 * (0.13 * x).sin());
                assert!((centre - want).abs() < 0.1, "ppc {ppc} x {x:.2}: centre {centre:.3} cells, the river is at {want:.3}");
                width += m / ppc as f64;
                n += 1;
            }
            width / n as f64
        };
        let (lo, hi) = (sweep(0.97), sweep(1.07));
        assert!((hi / lo - 1.0).abs() < 0.10, "width {lo:.3} -> {hi:.3} cells across the switch");
    }
}
