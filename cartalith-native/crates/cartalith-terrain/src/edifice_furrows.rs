//! **Tile-time radial furrows on volcano flanks** (owner Ruling CO item 3,
//! 2026-10-18; measured as variant (b) in `EDIFICE_AND_STREAKS_2026-10-18.md`
//! at the repository root): the erosion rills the generation grid cannot
//! hold, synthesised by the deep-zoom tile path as the tile's own resolution
//! can carry them.
//!
//! # Why
//!
//! The generation grid can hold a gully no finer than about two cells. The
//! shipped flank cuts them at four cells, so at x16-x64 the tile path's
//! bilinear up-sample shows them as soft radial bands with no gully inside,
//! and the kilometre zoom-detail law (`amplify::KmZoomDetail`) adds isotropic
//! value noise there, not furrows. This module adds the furrows the grid could
//! not hold, octave by octave, as the tile can carry them: the same "complete
//! the spectrum at the scale that can hold it" rule the zoom law follows, with
//! oriented (downslope) noise instead of isotropic. Owner requirement: zoom
//! gains detail at every level.
//!
//! # The believability pass (Ruling CO item 3)
//!
//! The measured variant (b) read "slightly combed": dense, evenly spaced,
//! straight radial hairlines; and on small cones (a radius of a few cells) its
//! radial noise lattice was no longer than its angular one, so the ridged
//! noise drew short isotropic strokes. Candidate laws were compared on
//! 71077345 (the dome and a small cone), 24601 and 314159 at x16 and x64
//! against tiles without furrows (`STATUS.md`, 2026-10-18 Ruling CO (3)
//! entry; frames in `cartalith-native/docs/evidence/pitfill-2026-10-18/`).
//! What was kept:
//!
//! - **A fixed rill aspect** ([`FURROW_ASPECT`]): each octave's radial noise
//!   lattice is a fixed multiple of its own angular spacing instead of a
//!   fixed fraction of the edifice radius, so a rill is long and thin on a
//!   small cone and on a shield alike, and is broken into dashes on a large
//!   shield rather than drawn as a ray across the whole flank.
//! - **A gentle meander** ([`FURROW_WARP`], [`FURROW_WARP_RFREQ`]): the
//!   angular coordinate is displaced by a fraction of the octave's own
//!   spacing, varying slowly downslope, so lines wander instead of radiating
//!   as straight spokes.
//! - **Metres converted with `peak_m / (1 - sea)`**, the engine's own metres
//!   per normalised unit, not `peak_m` (the measured build over-scaled every
//!   furrow by `1 / (1 - sea)`, 1.72x at the app's sea level 0.42).
//!
//! Tried and dropped (no dual path, `DECISIONS.md` §7p): a steeper octave
//! fall-off (relief per octave `2^-1.25k` and `2^-1.5k`: the furrows all but
//! vanished), a slope gate (shield flanks are gentle, so it removed the dome's
//! furrows), random on/off masking along each rill (less detail, no visible
//! gain) and a strong, fast warp (it turned small-cone rills into loops).
//!
//! # What it reads, and why only that
//!
//! Only the [`EdificeFrame`] recovered from `volcanic_field` -- which
//! `WorldState` keeps and the LOD path already has -- because the per-edifice
//! records the stamps make are transient. That recovery is approximate (the
//! centre to the nearest cell, the radius as a median), which is fine for
//! **new** detail: it fixes where furrows run (downslope from the recovered
//! centre), not the generated gullies' exact phase.
//!
//! # What it must never do
//!
//! - Run on the parity path or outside the deep-zoom tiles: its only app
//!   caller is `cartalith-godot`'s `render::add_tile_furrows`, from
//!   `lod_bridge::tile_heights`, scaled by the look's `edifice_furrows`
//!   tunable, which `TerrainAppearance::js_reference` pins at `0.0`. The base
//!   raster, the fit view, the export bake and the info tiles do not call it.
//! - Add an octave the generation grid already holds, or one finer than the
//!   tile can hold ([`resolved_weight`] on both).
//! - Move the coastline: water texels are skipped and an excursion toward sea
//!   level is capped by `amplify`'s own `clamp_toward_sea`.
//! - Depend on anything but the texel's coarse coordinate, the frame and the
//!   options, which keeps two tiles seam-exact on a shared edge.

use cartalith_noise::pvnoise;
use cartalith_spatial::FloatRegion;

use crate::edifice_pits::EdificeFrame;

/// The furrows' gully spacing at mid-flank, km. **A labelled judgement**, not
/// a measurement of real volcanoes: it keeps the shipped stamp's look where
/// the app generates (4 cells at 800 km / 2048 is 1.56 km), and it sits inside
/// the order of radial-valley spacing seen on dissected stratocones and shield
/// flanks (about 1-3 km between barrancas; no source opened in this pass).
/// Moved here from the retired variant (a) (`EDIFICE_AND_STREAKS_2026-10-18.md`;
/// Ruling CO), whose `EDIFICE_GULLY_SPACING_KM_PROPOSED` it was.
pub const FURROW_SPACING_KM: f64 = 1.5;

/// [`resolved_weight`]'s fade, in cells of **noise-lattice spacing**: weight 0
/// at or below `LO`, 1 at or above `HI`. Value noise with a lattice spacing of
/// `L` cells carries most of its energy at wavelengths of about `2L`, and a
/// grid holds nothing shorter than two cells (Nyquist), so `L = 1` is the hard
/// floor. `LO = 1.5` and `HI = 3` keep a margin above it. **A labelled
/// judgement**, the same kind as the zoom law's one-octave Nyquist margin
/// (`amplify::ZOOM_DETAIL_KM_NYQUIST_MARGIN`).
const RESOLVE_LO_CELLS: f64 = 1.5;
const RESOLVE_HI_CELLS: f64 = 3.0;

/// How much of a noise octave with lattice spacing `l` (in grid or texel
/// units) a raster with unit spacing can hold: a smoothstep from 0 at
/// [`RESOLVE_LO_CELLS`] to 1 at [`RESOLVE_HI_CELLS`]. NaN reads 0 (nothing is
/// resolved).
pub fn resolved_weight(l: f64) -> f64 {
    if l.is_nan() {
        return 0.0;
    }
    let t = ((l - RESOLVE_LO_CELLS) / (RESOLVE_HI_CELLS - RESOLVE_LO_CELLS)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// The mean of one ridged octave, `E[(1 - |2·n - 1|)^2]` with `n` a `pvnoise`
/// sample. **Measured**, not derived: for a uniform `n` it would be `1/3`, but
/// `pvnoise`'s smoothstep blend of four lattice values pulls `n` toward 0.5
/// and raises it. `ridge_mean_is_the_measured_one` re-measures it over 2^20
/// samples and fails if it moves by 0.005. Subtracted so a furrow octave is
/// zero-mean and the flank's mean height does not move.
pub const RIDGE_SQ_MEAN: f64 = 0.4701;

/// The furrow law's knobs. Every field is required; there is no `Default`,
/// because each one is a scale the caller must state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FurrowOpts {
    /// Kilometres per coarse cell.
    pub km_per_cell: f64,
    /// Gully spacing at mid-flank, km ([`FURROW_SPACING_KM`] in the app).
    pub spacing_km: f64,
    /// `WorldParams::peak_m`, the height that normalised 1.0 stands for.
    pub peak_m: f64,
    /// Sea level, normalised.
    pub sea: f64,
}

/// Furrow relief per unit spacing: an octave with spacing `s` km has relief
/// amplitude `FURROW_DEPTH_PER_SPACING * s`. Depth proportional to spacing is
/// the self-similar (`H = 1`) spectrum the zoom law ships
/// (`amplify::ZOOM_DETAIL_KM_OCTAVE_GAIN`), so every octave adds the same
/// slope. `0.1` puts a 1.5 km gully at 150 m, near the shipped gully term's
/// own ~117 m on the reference's mid-class stratocone (0.45 x 4 000 m x 0.10
/// rough x 0.65). **A labelled judgement**.
pub const FURROW_DEPTH_PER_SPACING: f64 = 0.1;

/// The finest octave this law adds, counted from the mid-flank spacing: 2^-8
/// of 1.5 km is about 6 m. A cost cap, like `KmZoomDetail::max_octaves`.
pub const FURROW_MAX_OCTAVES: i32 = 9;

/// The flank band furrows live in, as a fraction of the recovered radius: 0
/// inside `t < FURROW_T_TOP` (the summit and its depression; the recovered
/// frame does not know the caldera's own rim) and at the toe, peaking between.
/// **A labelled judgement**: between the shipped shapes' rims (0.12-0.38).
pub const FURROW_T_TOP: f64 = 0.25;

/// The rill aspect: each octave's radial noise lattice is `FURROW_ASPECT`
/// times the furrow spacing at that octave, so a rill is about that many
/// times longer than it is wide whatever the edifice's size. **A labelled
/// judgement** from the believability pass (4 and 6 both read as rills; 6
/// kept the shield's dashes longer). The measured build's radius-keyed
/// lattice gave aspects from about 0.8 (a 7-cell cone: short isotropic
/// strokes) to about 24 (the 83 km dome: straight rays).
pub const FURROW_ASPECT: f64 = 6.0;

/// The meander: the angular coordinate is displaced by up to `FURROW_WARP`
/// of the octave's own angular lattice, by a noise whose radial frequency is
/// `FURROW_WARP_RFREQ` of the rill's own, so a line wanders by about a third
/// of a spacing over two rill lengths. **A labelled judgement**: a warp of
/// 1.2 at three times the rill's radial frequency drew loops on small cones;
/// 0.3 at half its frequency kept the lines running downslope.
pub const FURROW_WARP: f64 = 0.3;
/// See [`FURROW_WARP`].
pub const FURROW_WARP_RFREQ: f64 = 0.5;

/// The furrow height (normalised units, zero-mean) at coarse coordinate
/// `(cx, cy)` for a tile whose texels are `texel_cells` coarse cells apart.
/// Zero off every recovered edifice, on the summit (`t < FURROW_T_TOP`) and
/// at the toe.
///
/// Octave `k` (spacing `s_k = spacing_km / 2^k`, relief
/// `FURROW_DEPTH_PER_SPACING * s_k`) is weighted by `max(0, w_tile - w_grid)`:
/// the share the tile can hold beyond what the generation grid already holds,
/// both from [`resolved_weight`] on the octave's noise-lattice spacing (the
/// smaller of its angular and radial spacing, in texels and in cells). So an
/// octave the generated field already carries is never added twice, and one
/// finer than the tile is never drawn.
pub fn edifice_furrow_at(frame: &EdificeFrame, cx: f64, cy: f64, texel_cells: f64, o: &FurrowOpts) -> f64 {
    let Some(e) = frame.owner_near(cx, cy) else { return 0.0 };
    if !(o.km_per_cell > 0.0 && o.spacing_km > 0.0 && o.peak_m > 0.0 && texel_cells > 0.0 && o.sea < 1.0) {
        return 0.0;
    }
    let (dx, dy) = (cx - e.cx, cy - e.cy);
    let d = dx.hypot(dy);
    let t = d / e.r;
    if !(FURROW_T_TOP..1.0).contains(&t) {
        return 0.0;
    }
    let u = (t - FURROW_T_TOP) / (1.0 - FURROW_T_TOP);
    let envelope = 4.0 * u * (1.0 - u);
    // The engine's metres per normalised unit (`metresPerUnit = peakM / (1 -
    // seaLevel)`), so `relief_m` below is metres on the ground.
    let mpu = o.peak_m / (1.0 - o.sea);
    let sp_cells = o.spacing_km / o.km_per_cell;
    let g = ((std::f64::consts::PI * e.r / sp_cells).round().clamp(6.0, 1.0e6)) as i32;
    let ang = (dy.atan2(dx) * std::f64::consts::FRAC_1_PI * 0.5 + 0.5) * g as f64;
    let mut sum = 0.0;
    for k in 0..FURROW_MAX_OCTAVES {
        let f = f64::powi(2.0, k);
        // The octave's radial lattice: FURROW_ASPECT times its spacing, so a
        // rill's length is a fixed multiple of its width (module doc).
        let rad_lat = FURROW_ASPECT * sp_cells / f;
        let yk = d / (FURROW_ASPECT * sp_cells) * f;
        let lat = (std::f64::consts::TAU * d / (g as f64 * f)).min(rad_lat);
        let w = (resolved_weight(lat / texel_cells) - resolved_weight(lat)).max(0.0);
        if w <= 0.0 {
            continue;
        }
        // The angular period is the octave's gully count, so the noise closes
        // across the bearing's +/-pi seam (`furrows_close_across_the_seam`).
        let per = (g as f64 * f).min(i32::MAX as f64) as i32;
        let salt = e.salt.wrapping_add(k.wrapping_mul(131));
        let x = ang * f + FURROW_WARP * (2.0 * pvnoise(ang * f, yk * FURROW_WARP_RFREQ + 7.1, salt ^ 0x3c1, per) - 1.0);
        let n = pvnoise(x, yk, salt, per);
        let ridge = 1.0 - (2.0 * n - 1.0).abs();
        // Incised, like the shipped gully term (`1 - 2·pridged`): a high
        // ridge is a furrow floor. Zero-mean, so the flank's mean height does
        // not move.
        let relief_m = FURROW_DEPTH_PER_SPACING * (o.spacing_km / f) * 1000.0;
        sum += -(ridge * ridge - RIDGE_SQ_MEAN) * 2.0 * relief_m / mpu * w;
    }
    sum * envelope
}

/// [`edifice_furrow_at`] over a tile carrying `amplify::amplify_region_padded`'s
/// halo, in place, with `add_zoom_detail_padded`'s own texel-to-coarse mapping
/// (`b` is the tile's coarse footprint, `w`/`h` the tile without the halo).
/// Water texels are skipped; an excursion toward sea level is capped at half
/// the headroom (`amplify::clamp_toward_sea`). `strength` scales the furrows
/// (the look's `edifice_furrows`); `0.0`, or NaN, returns before touching a
/// texel, so a tile with the tunable at zero is the tile without furrows by
/// control flow.
#[allow(clippy::too_many_arguments)]
pub fn add_edifice_furrows_padded(data: &mut [f32], w: usize, h: usize, pad: usize, frame: &EdificeFrame, b: &FloatRegion, o: &FurrowOpts, strength: f64) {
    if !(strength > 0.0) {
        return;
    }
    let (pw, ph) = (w + 2 * pad, h + 2 * pad);
    assert!(data.len() >= pw * ph, "add_edifice_furrows_padded: data is too short");
    let texel_cells = if w > 1 { b.w / (w as f64 - 1.0) } else { b.w };
    let padf = pad as f64;
    for oy in 0..ph {
        let cy = b.y + if h > 1 { (oy as f64 - padf) / (h as f64 - 1.0) * b.h } else { 0.0 };
        for ox in 0..pw {
            let i = oy * pw + ox;
            let base = data[i] as f64;
            if base < o.sea {
                continue;
            }
            let cx = b.x + if w > 1 { (ox as f64 - padf) / (w as f64 - 1.0) * b.w } else { 0.0 };
            let s = strength * edifice_furrow_at(frame, cx, cy, texel_cells, o);
            if s != 0.0 {
                data[i] = (base + crate::amplify::clamp_toward_sea(base, s, o.sea)) as f32;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A synthetic `volcanic_field`: two cones, `(1 - t)(1 - age)`, the
    /// stamp's own mark.
    fn two_cones(gw: usize, gh: usize) -> Vec<f32> {
        let mut v = vec![0f32; gw * gh];
        for (cx, cy, r, age) in [(40.0, 40.0, 30.0, 0.2), (110.0, 60.0, 20.0, 0.0)] {
            for y in 0..gh {
                for x in 0..gw {
                    let d = (x as f64 - cx).hypot(y as f64 - cy);
                    if d <= r {
                        let c = ((1.0 - d / r) * (1.0 - age)) as f32;
                        let i = y * gw + x;
                        v[i] = v[i].max(c);
                    }
                }
            }
        }
        v
    }

    fn opts() -> FurrowOpts {
        FurrowOpts { km_per_cell: 800.0 / 2048.0, spacing_km: 1.5, peak_m: 4000.0, sea: 0.42 }
    }

    /// Protects: the mean subtracted from every furrow octave is the noise's
    /// real mean (measured over 2^20 samples at incommensurate coordinates),
    /// so the furrows do not shift the flank's mean height.
    #[test]
    fn ridge_mean_is_the_measured_one() {
        let mut r = 0.0f64;
        let n = 1usize << 20;
        for k in 0..n {
            let (x, y) = ((k % 1024) as f64 * 0.618_034 + 0.13, (k / 1024) as f64 * 0.414_214 + 0.71);
            let q = 1.0 - (2.0 * pvnoise(x, y, 4242, 1 << 20) - 1.0).abs();
            r += q * q;
        }
        let r = r / n as f64;
        assert!((r - RIDGE_SQ_MEAN).abs() < 0.005, "ridge mean measured {r}");
        assert_eq!(resolved_weight(1.5), 0.0);
        assert_eq!(resolved_weight(3.0), 1.0);
        assert_eq!(resolved_weight(f64::NAN), 0.0);
    }

    /// Protects: the band rule. At a texel spacing equal to the coarse cell
    /// (x1-ish) nothing is added -- the grid already holds what that texel
    /// could -- and at a fine texel (1/16 cell) furrows appear, zero-mean
    /// round the flank, and nothing is added on the summit or off the cone.
    #[test]
    fn furrows_fill_only_what_the_grid_could_not_hold() {
        let (gw, gh) = (160usize, 110usize);
        let f = EdificeFrame::from_volcanic_field(&two_cones(gw, gh), gw, gh);
        let o = opts();
        // Eleven rings across the mid-flank (t = 0.4..0.73), 720 bearings
        // each: one ring is too few independent rills to test a mean on.
        let ring = |texel: f64| -> Vec<f64> {
            (0..11)
                .flat_map(|j| {
                    let r = 12.0 + 1.0 * j as f64;
                    (0..720).map(move |k| {
                        let a = k as f64 / 720.0 * std::f64::consts::TAU;
                        (r, a)
                    })
                })
                .map(|(r, a)| edifice_furrow_at(&f, 40.0 + r * a.cos(), 40.0 + r * a.sin(), texel, &o))
                .collect()
        };
        assert!(ring(1.0).iter().all(|&s| s == 0.0), "a cell-sized texel must add nothing");
        let fine = ring(1.0 / 16.0);
        let mean = fine.iter().sum::<f64>() / fine.len() as f64;
        let rms = (fine.iter().map(|s| s * s).sum::<f64>() / fine.len() as f64).sqrt();
        assert!(rms > 1e-4, "fine texels should carry furrows, rms {rms}");
        assert!(mean.abs() < 0.25 * rms, "furrows should be about zero-mean: mean {mean}, rms {rms}");
        assert_eq!(edifice_furrow_at(&f, 40.0, 41.0, 1.0 / 16.0, &o), 0.0, "summit");
        assert_eq!(edifice_furrow_at(&f, 150.0, 5.0, 1.0 / 16.0, &o), 0.0, "off every cone");
    }

    /// Protects: the angular noise closes across the bearing's +/-pi seam
    /// (due west of the centre), like the shipped gully term's `pridged`
    /// period: no radial scar where `atan2` wraps. The noise's period must be
    /// on the angular axis for this to hold.
    #[test]
    fn furrows_close_across_the_seam() {
        let (gw, gh) = (160usize, 110usize);
        let f = EdificeFrame::from_volcanic_field(&two_cones(gw, gh), gw, gh);
        let o = opts();
        let (mut jump, mut step) = (0.0f64, 0.0f64);
        for k in 0..40 {
            let r = 12.0 + k as f64 * 0.3;
            let e = 1e-6;
            let p = |a: f64| edifice_furrow_at(&f, 40.0 + r * a.cos(), 40.0 + r * a.sin(), 1.0 / 16.0, &o);
            jump = jump.max((p(std::f64::consts::PI - e) - p(-std::f64::consts::PI + e)).abs());
            step = step.max((p(1.0) - p(1.0 + 2.0 * e)).abs());
        }
        // Across the seam the change must be no bigger than an ordinary step
        // of the same angular size elsewhere.
        assert!(jump <= 2.0 * step + 1e-12, "a scar at the seam: {jump} (an ordinary step of the same size is {step})");
    }

    /// Protects: the furrows run radially -- along a ray the furrow height
    /// varies far less than across the ray at the same radius.
    #[test]
    fn furrows_are_radial() {
        let (gw, gh) = (160usize, 110usize);
        let f = EdificeFrame::from_volcanic_field(&two_cones(gw, gh), gw, gh);
        let o = opts();
        let tex = 1.0 / 16.0;
        let (mut along, mut across) = (0.0, 0.0);
        for k in 0..400 {
            let a = k as f64 * 0.0157;
            let r0 = 17.0;
            let p = |r: f64, da: f64| {
                let (cc, ss) = ((a + da).cos(), (a + da).sin());
                edifice_furrow_at(&f, 40.0 + r * cc, 40.0 + r * ss, tex, &o)
            };
            along += (p(r0 + 0.05, 0.0) - p(r0, 0.0)).powi(2);
            across += (p(r0, 0.05 / r0) - p(r0, 0.0)).powi(2);
        }
        assert!(across > 4.0 * along, "across {across} vs along {along}");
    }

    /// One cone of radius `r` cells at `(cx, cy)` on a `gw x gh` grid.
    fn one_cone(gw: usize, gh: usize, cx: f64, cy: f64, r: f64) -> Vec<f32> {
        (0..gw * gh)
            .map(|i| {
                let d = ((i % gw) as f64 - cx).hypot((i / gw) as f64 - cy);
                if d <= r { (1.0 - d / r) as f32 } else { 0.0 }
            })
            .collect()
    }

    /// Protects: the believability pass's fixed rill aspect. On a small cone
    /// (radius 8 cells, about 3 km at the app's 800 km cell) the furrows still
    /// run downslope -- the height varies far less along a ray than across it
    /// -- where the measured build's radius-keyed radial lattice (`r / 2.2`)
    /// was no longer than the angular one and drew short isotropic strokes.
    #[test]
    fn small_cone_furrows_still_run_downslope() {
        let (gw, gh) = (40usize, 40usize);
        let f = EdificeFrame::from_volcanic_field(&one_cone(gw, gh, 20.0, 20.0, 8.0), gw, gh);
        assert_eq!(f.edifices.len(), 1, "{:?}", f.edifices);
        let o = opts();
        let tex = 1.0 / 32.0;
        let (mut along, mut across) = (0.0, 0.0);
        for k in 0..720 {
            let a = k as f64 * std::f64::consts::TAU / 720.0;
            let r0 = 4.6;
            let p = |r: f64, da: f64| edifice_furrow_at(&f, 20.0 + r * (a + da).cos(), 20.0 + r * (a + da).sin(), tex, &o);
            along += (p(r0 + 0.03, 0.0) - p(r0, 0.0)).powi(2);
            across += (p(r0, 0.03 / r0) - p(r0, 0.0)).powi(2);
        }
        assert!(across > 0.0, "no furrow on the small cone");
        assert!(across > 4.0 * along, "small cone: across {across} vs along {along}");
    }

    /// Protects: furrow relief is metres on the ground, converted with the
    /// engine's own metres per unit `peak_m / (1 - sea)`: the same point at
    /// sea 0.42 carries 0.58 of the normalised relief it carries at sea 0 (the
    /// measured build divided by `peak_m` alone, which made the ratio 1).
    #[test]
    fn relief_uses_the_engines_metres_per_unit() {
        let (gw, gh) = (160usize, 110usize);
        let f = EdificeFrame::from_volcanic_field(&two_cones(gw, gh), gw, gh);
        let at = |sea: f64| -> f64 {
            let o = FurrowOpts { sea, ..opts() };
            (0..360).map(|k| {
                let a = k as f64 * 0.0174;
                edifice_furrow_at(&f, 40.0 + 17.0 * a.cos(), 40.0 + 17.0 * a.sin(), 1.0 / 16.0, &o).abs()
            }).sum()
        };
        let (s0, s42) = (at(0.0), at(0.42));
        assert!(s0 > 0.0);
        assert!((s42 / s0 - 0.58).abs() < 1e-9, "ratio {}", s42 / s0);
    }

    /// Protects: `strength` 0 (the parity look) and NaN return before
    /// touching a texel, so such a tile is the furrow-free tile by control
    /// flow; strength 1 moves land texels on the flank and no water texel.
    #[test]
    fn zero_strength_moves_no_texel() {
        let (gw, gh) = (160usize, 110usize);
        let f = EdificeFrame::from_volcanic_field(&two_cones(gw, gh), gw, gh);
        let o = opts();
        let (w, h, pad) = (64usize, 64usize, 2usize);
        let b = FloatRegion { x: 50.0, y: 32.0, w: 4.0, h: 4.0 };
        let mut base: Vec<f32> = vec![0.6; (w + 2 * pad) * (h + 2 * pad)];
        for v in base.iter_mut().step_by(7) {
            *v = 0.3; // some water texels
        }
        for st in [0.0, f64::NAN, -1.0] {
            let mut d = base.clone();
            add_edifice_furrows_padded(&mut d, w, h, pad, &f, &b, &o, st);
            assert_eq!(d, base, "strength {st} moved a texel");
        }
        let mut d = base.clone();
        add_edifice_furrows_padded(&mut d, w, h, pad, &f, &b, &o, 1.0);
        assert!(d.iter().zip(&base).any(|(a, b)| a != b), "strength 1 moved nothing");
        assert!(d.iter().zip(&base).all(|(a, b)| *b >= 0.42 || a == b), "a water texel moved");
    }
}
