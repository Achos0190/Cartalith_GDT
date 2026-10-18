//! AP-4 (`ASSET_PACK_TOOLS_PROPOSAL.md` §6, Ruling CQ): make a ground tile
//! wrap without a seam.
//!
//! **Why it matters.** Splat channels and painted biome/terrain tiles are
//! sampled wrapped, nearest-neighbour, one texel per output pixel or cell
//! (`ASSET_PACK_TOOLS_PROPOSAL.md` §1.3), so a tile whose right edge does not
//! continue into its left edge draws a hard line every tile width across the
//! whole map. The checker is AP-2's [`seam_score`]; this module is the fixer.
//!
//! **The method** is the proposal's: offset by half, then cross-blend over a
//! band of `w/8`. Per axis that needs it (x first, then y on the x result):
//!
//! 1. roll the tile by half its size, so the old wrap seam lands in the
//!    middle and the old middle -- continuous by construction, it is the
//!    tile's own interior -- becomes the new wrap edge;
//! 2. inside a band `w/8` wide centred on that middle seam, blend from the
//!    rolled tile back to the unrolled one, reaching the unrolled tile
//!    exactly on the two seam columns, whose pixels are therefore a pair of
//!    the tile's own interior neighbours again.
//!
//! After both steps the wrap-edge pair and the centre-line pair of every row
//! (and column) are the same two source pixels, so the fixed axis scores an
//! edge-to-interior ratio of 1 by [`seam_score`]'s own definition, up to the
//! rounding of the blend. The cost is visible: inside the band the result is a
//! cross-fade of two offset copies (some ghosting on strongly structured art),
//! and the whole tile is shifted by half -- immaterial for a wrapped texture,
//! which has no origin on the map.
//!
//! **Never touches a tile that already wraps**: [`make_seamless`] returns no
//! image when [`seam_score`] does not call it a seam, so a seamless tile comes
//! back byte-identical (it is not re-blended "to be safe"). An axis that
//! already wraps is left alone even when the other is fixed.
//!
//! Must never write anything but pixels: this is a REFINE verb, and Ruling CQ
//! Q7 has the caller store the result as a new variant beside the original.

use crate::pack_report::{SEAM_RATIO, SeamScore, seam_score};
use crate::raster::DecodedImage;

/// What [`make_seamless`] found and did.
#[derive(Debug, Clone, PartialEq)]
pub struct SeamlessOutcome {
    /// [`seam_score`] of the input.
    pub before: SeamScore,
    /// [`seam_score`] of the result; equal to `before` when nothing was done.
    pub after: SeamScore,
    /// Whether the left-right wrap was blended.
    pub fixed_x: bool,
    /// Whether the top-bottom wrap was blended.
    pub fixed_y: bool,
    /// The seamless tile, or `None` when the input already wraps by
    /// [`SeamScore::is_seam`] (nothing to do, and nothing done).
    pub image: Option<DecodedImage>,
}

/// One axis's half of [`seam_score`]: the wrap-edge and centre-line means for
/// the left-right wrap (`horizontal == true`, averaged over rows) or the
/// top-bottom wrap (averaged over columns), in the same 8-bit RGB levels.
/// A tile under 2 px on that axis scores zero on both.
pub fn axis_seam_score(img: &DecodedImage, horizontal: bool) -> SeamScore {
    let (w, h) = (img.w as usize, img.h as usize);
    let (len, across) = if horizontal { (w, h) } else { (h, w) };
    if len < 2 || across == 0 {
        return SeamScore { edge: 0.0, interior: 0.0 };
    }
    let at = |along: usize, other: usize| -> usize { (if horizontal { other * w + along } else { along * w + other }) * 4 };
    let diff = |a: usize, b: usize| -> f64 {
        (0..3).map(|c| f64::from(img.rgba[a + c].abs_diff(img.rgba[b + c]))).sum::<f64>() / 3.0
    };
    let (mut edge, mut interior) = (0.0, 0.0);
    for o in 0..across {
        edge += diff(at(len - 1, o), at(0, o));
        interior += diff(at(len / 2 - 1, o), at(len / 2, o));
    }
    SeamScore { edge: edge / across as f64, interior: interior / across as f64 }
}

/// The offset-and-blend along one axis (see the module doc). `len` is the
/// tile's size on that axis; the band is `len / 8` wide, at least 2 px.
fn blend_axis(img: &DecodedImage, horizontal: bool) -> DecodedImage {
    let (w, h) = (img.w as usize, img.h as usize);
    let len = if horizontal { w } else { h };
    let half = len / 2;
    // The rolled tile's seam sits between `c - 1` and `c`.
    let c = len - half;
    // Half the band. `len / 8` across, judged by the proposal; floored at 1 so
    // a tiny tile still blends its two seam pixels.
    let hb = ((len / 16) as f64).max(1.0);
    let mut out = vec![0u8; img.rgba.len()];
    for y in 0..h {
        for x in 0..w {
            let along = if horizontal { x } else { y };
            let rolled_along = (along + half) % len;
            let (rx, ry) = if horizontal { (rolled_along, y) } else { (x, rolled_along) };
            // Distance from the seam in pixel centres; the two seam pixels
            // are 0.5 away and take weight exactly 1 (the unrolled tile).
            let d = (along as f64 + 0.5 - c as f64).abs();
            let wgt = (1.0 - (d - 0.5) / hb).clamp(0.0, 1.0);
            let o = (y * w + x) * 4;
            let r = (ry * w + rx) * 4;
            for k in 0..4 {
                let v = f64::from(img.rgba[r + k]) * (1.0 - wgt) + f64::from(img.rgba[o + k]) * wgt;
                out[o + k] = (v + 0.5).floor().clamp(0.0, 255.0) as u8;
            }
        }
    }
    DecodedImage::new(img.w, img.h, out).expect("same dimensions as the input")
}

/// Make `img` wrap without a seam, or say it already does.
///
/// Only when [`seam_score`] calls the tile a seam ([`SeamScore::is_seam`]):
/// each axis whose own edge exceeds [`SEAM_RATIO`] × its own interior is
/// blended ([`axis_seam_score`]); if, at the floor's margin, neither axis
/// alone does, both are. A tile under 4 px on a side is returned untouched
/// (no room for a band).
pub fn make_seamless(img: &DecodedImage) -> SeamlessOutcome {
    let before = seam_score(img);
    let untouched = SeamlessOutcome { before, after: before, fixed_x: false, fixed_y: false, image: None };
    if !before.is_seam() || img.w < 4 || img.h < 4 {
        return untouched;
    }
    let sx = axis_seam_score(img, true);
    let sy = axis_seam_score(img, false);
    let mut fixed_x = sx.edge > SEAM_RATIO * sx.interior;
    let mut fixed_y = sy.edge > SEAM_RATIO * sy.interior;
    if !fixed_x && !fixed_y {
        fixed_x = true;
        fixed_y = true;
    }
    let mut cur = img.clone();
    if fixed_x {
        cur = blend_axis(&cur, true);
    }
    if fixed_y {
        cur = blend_axis(&cur, false);
    }
    SeamlessOutcome { before, after: seam_score(&cur), fixed_x, fixed_y, image: Some(cur) }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A `w`×`h` opaque tile from `f(x, y) -> [r, g, b]`.
    fn tile(w: u32, h: u32, f: impl Fn(u32, u32) -> [u8; 3]) -> DecodedImage {
        let mut v = Vec::with_capacity((w * h * 4) as usize);
        for y in 0..h {
            for x in 0..w {
                let [r, g, b] = f(x, y);
                v.extend_from_slice(&[r, g, b, 255]);
            }
        }
        DecodedImage::new(w, h, v).unwrap()
    }

    /// Deterministic value noise in `[0, 1)`, not periodic in any tile size
    /// used here: a hash of the lattice point, bilinearly interpolated.
    fn noise(x: f64, y: f64, seed: u32) -> f64 {
        let h = |i: i64, j: i64| -> f64 {
            let mut n = (i as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ (j as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F) ^ u64::from(seed);
            n ^= n >> 31;
            n = n.wrapping_mul(0xBF58_476D_1CE4_E5B9);
            n ^= n >> 29;
            (n >> 11) as f64 / (1u64 << 53) as f64
        };
        let (x0, y0) = (x.floor(), y.floor());
        let (fx, fy) = (x - x0, y - y0);
        let (i, j) = (x0 as i64, y0 as i64);
        let a = h(i, j) * (1.0 - fx) + h(i + 1, j) * fx;
        let b = h(i, j + 1) * (1.0 - fx) + h(i + 1, j + 1) * fx;
        a * (1.0 - fy) + b * fy
    }

    /// The generated "packs" AP-4's bar is checked on (the proposal: the
    /// reference fixture plus at least two generated packs; the fixture's
    /// ground tiles are flat colours, which already wrap and are covered by
    /// the untouched test below). Each is a ground tile with a real seam.
    fn seamed_tiles() -> Vec<(&'static str, DecodedImage)> {
        vec![
            ("horizontal ramp", tile(512, 512, |x, _| [(x / 2) as u8, 90, 40])),
            ("vertical ramp", tile(512, 512, |_, y| [40, (y / 2) as u8, 90])),
            ("diagonal gradient + noise", tile(512, 512, |x, y| {
                let n = noise(f64::from(x) / 23.0, f64::from(y) / 23.0, 3) * 60.0;
                [((x + y) / 5) as u8, (60.0 + n) as u8, (120.0 - n) as u8]
            })),
            ("multi-octave noise, grass-like", tile(512, 512, |x, y| {
                let (fx, fy) = (f64::from(x), f64::from(y));
                let n = 0.5 * noise(fx / 64.0, fy / 64.0, 7) + 0.3 * noise(fx / 16.0, fy / 16.0, 8) + 0.2 * noise(fx / 4.0, fy / 4.0, 9);
                [(40.0 + 50.0 * n) as u8, (90.0 + 90.0 * n) as u8, (30.0 + 30.0 * n) as u8]
            })),
            ("offset brick", tile(512, 512, |x, y| {
                let row = y / 37;
                let xs = x + if row % 2 == 1 { 23 } else { 0 };
                if y % 37 < 3 || xs % 71 < 3 { [70, 60, 55] } else { [160, 90 + (row % 5) as u8 * 9, 70] }
            })),
            ("small 64 px noise tile", tile(64, 64, |x, y| {
                let n = noise(f64::from(x) / 7.0, f64::from(y) / 7.0, 11);
                [(200.0 * n) as u8, (150.0 * n) as u8, 60]
            })),
        ]
    }

    /// Protects: AP-4's acceptance bar on every generated seamed tile -- each
    /// is a seam before (so the test is not vacuous) and, made seamless,
    /// scores edge <= 1.25 x interior; the output keeps size and alpha.
    #[test]
    fn made_seamless_tiles_meet_the_bar() {
        for (name, t) in seamed_tiles() {
            let out = make_seamless(&t);
            assert!(out.before.is_seam(), "{name}: the fixture must start with a seam ({:?})", out.before);
            let img = out.image.as_ref().unwrap_or_else(|| panic!("{name}: a seamed tile is changed"));
            assert_eq!((img.w, img.h), (t.w, t.h));
            assert!(img.rgba.chunks(4).all(|p| p[3] == 255), "{name}: opaque stays opaque");
            let s = seam_score(img);
            assert_eq!(s, out.after);
            assert!(s.edge <= SEAM_RATIO * s.interior + 1e-9, "{name}: {s:?} (before {:?})", out.before);
            assert!(!s.is_seam(), "{name}: still a seam {s:?}");
        }
    }

    /// Protects: a tile that already wraps comes back untouched -- no image,
    /// so the caller stores nothing and the original's bytes are the answer --
    /// for a periodic pattern, a flat colour (the reference fixture's ground
    /// tiles are flat), and periodic waves.
    #[test]
    fn an_already_seamless_tile_is_left_untouched() {
        let periodic = tile(512, 512, |x, y| [((x % 64) * 3) as u8, ((y % 32) * 7) as u8, 50]);
        let flat = tile(512, 512, |_, _| [93, 140, 61]);
        // Integer-frequency sines are periodic in 256 and have the same
        // slope magnitude at the wrap edge as at the centre line, which is
        // what the score compares. (A noise field sampled on a torus is
        // periodic too, but its local slope at column 0 and column 128
        // differ, and it measured edge 3.09 against interior 1.74 -- a seam by
        // the score's ratio although nothing is discontinuous. The score is
        // local by design; that case is a known false positive, not this
        // test's subject.)
        let wrapped_waves = tile(256, 256, |x, y| {
            let (a, b) = (f64::from(x) / 256.0 * std::f64::consts::TAU, f64::from(y) / 256.0 * std::f64::consts::TAU);
            let v = 128.0 + 50.0 * (3.0 * a).sin() + 30.0 * (5.0 * b).cos() + 20.0 * (2.0 * a + 7.0 * b).sin();
            [v as u8, 100, 100]
        });
        for (name, t) in [("periodic", periodic), ("flat", flat), ("periodic waves", wrapped_waves)] {
            let out = make_seamless(&t);
            assert!(!out.before.is_seam(), "{name}: fixture must already wrap ({:?})", out.before);
            assert_eq!(out.image, None, "{name}: untouched");
            assert!(!out.fixed_x && !out.fixed_y);
            assert_eq!(out.after, out.before);
        }
    }

    /// Protects: only the axis that seams is blended -- a horizontal ramp is
    /// fixed in x and its rows (which already wrap) are not rolled, so the
    /// output equals the x pass alone, and its first row's colour band is
    /// where the half-offset puts it (literal: column 0 is source column 256,
    /// value 256/2 = 128).
    #[test]
    fn only_the_seamed_axis_is_blended() {
        let ramp = tile(512, 512, |x, _| [(x / 2) as u8, 90, 40]);
        let out = make_seamless(&ramp);
        assert!(out.fixed_x && !out.fixed_y, "{out:?}");
        let img = out.image.unwrap();
        assert_eq!(&img.rgba[0..4], &[128, 90, 40, 255], "column 0 is the source's column 256");
        // Every row is identical: no y pass ran.
        let row0 = &img.rgba[0..512 * 4];
        assert!(img.rgba.chunks(512 * 4).all(|r| r == row0));
        // The two seam columns are the unrolled source's own (255 and 256 of
        // a 512 ramp: values 127 and 128).
        assert_eq!(img.rgba[255 * 4], 127);
        assert_eq!(img.rgba[256 * 4], 128);
    }

    /// Protects: the band is `w/8` wide -- for a 512 tile, pixels more than
    /// 32.5 px from the seam are the rolled tile untouched, and pixels inside
    /// are blended (literals on the ramp).
    #[test]
    fn the_blend_band_is_an_eighth_of_the_tile() {
        let ramp = tile(512, 512, |x, _| [(x / 2) as u8, 0, 0]);
        let img = make_seamless(&ramp).image.unwrap();
        // Column 256 - 34 = 222 is outside the band: rolled = source col 478 -> 239.
        assert_eq!(img.rgba[222 * 4], 239);
        // Column 256 - 33: d = 32.5, weight 0 -> rolled exactly: source 479 -> 239.
        assert_eq!(img.rgba[223 * 4], 239);
        // Column 256 - 17: d = 16.5, weight 0.5 -> (rolled 495/2=247 + own 239/2=119) / 2 = 183.
        assert_eq!(img.rgba[239 * 4], 183);
    }

    /// Protects: the per-axis split adds back up to `seam_score`, so the
    /// fixer and the checker measure the same thing.
    #[test]
    fn axis_scores_sum_to_the_seam_score() {
        let t = seamed_tiles().remove(2).1;
        let (sx, sy, s) = (axis_seam_score(&t, true), axis_seam_score(&t, false), seam_score(&t));
        let n = f64::from(t.w + t.h);
        assert!((sx.edge * f64::from(t.h) + sy.edge * f64::from(t.w) - s.edge * n).abs() < 1e-6);
        assert!((sx.interior * f64::from(t.h) + sy.interior * f64::from(t.w) - s.interior * n).abs() < 1e-6);
    }
}
