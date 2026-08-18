//! Region selection geometry — `UNIFIED_TOOL_PLAN.md` milestone E,
//! the Region select/export tool's *selection* half.
//!
//! Two pure functions the reference itself already isolated and documented as
//! headless-testable:
//!
//! - `normRegion` (reference line 11569, its own comment: *"two drag corners
//!   (any order, any overshoot) → a clamped integer grid rect with a minimum
//!   size. Pure → headless-tested."*) — the whole drag-rectangle interaction's
//!   engine half. The interaction itself (`regionDrag`/`regionSel`, reference
//!   9583) is pointer routing and belongs to the shell.
//! - `tileDims` (reference line 11536) — aspect-preserving per-tile pixel dims
//!   for a tiled export, so a non-square selection never comes back squished.
//!
//! They live here rather than in an export crate because a clamped integer
//! rectangle over a grid is exactly the generic spatial machinery this crate
//! exists for: neither function knows what a heightmap is. The *export* built
//! on top of them (`cartalith_terrain::amplify`, `cartalith_io::tiles`,
//! `cartalith_engine::region_export`) is split out per milestone A's placement
//! rule — generic machinery here, subsystem math in the owning crate,
//! composition in the orchestrator.

use crate::Region;
use serde::{Deserialize, Serialize};

/// The reference's `regionSel` in continuous coordinates.
///
/// `refineTile` splits a [`Region`] into `cols`×`rows` sub-bounds by dividing
/// its width by the column count, which is **not** generally an integer — the
/// sub-bounds are fractional coarse-cell coordinates by construction, and
/// rounding them would break the exact seam agreement between adjacent tiles
/// that the whole tiling model rests on. So the amplification path takes this
/// type, not [`Region`].
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FloatRegion {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

impl Region {
    /// Widen to continuous coordinates.
    pub fn to_float(self) -> FloatRegion {
        FloatRegion { x: self.x as f64, y: self.y as f64, w: self.w as f64, h: self.h as f64 }
    }
}

/// `Math.round` — round half **up** (toward `+∞`), not Rust's round-half-away-
/// from-zero. The two disagree at exactly `-0.5`-style inputs, which
/// [`crate::region`]'s own callers cannot reach but the icon brush's
/// `Math.round(cx + cos(ang)*rad)` very much can.
#[inline]
pub fn js_round(v: f64) -> f64 {
    (v + 0.5).floor()
}

/// `normRegion(x0,y0,x1,y1,W,H,minW,minH)` (reference line 11569).
///
/// Corners may arrive in any order and may overshoot the grid in either
/// direction. `min_w`/`min_h` default to 8; the reference spells that
/// `minW = minW || 8`, so an explicit **zero** also becomes 8 (`0` is falsy in
/// JS) — reproduced here rather than tidied, since a caller passing 0 today
/// gets 8 and would silently start getting a zero-width region if this were
/// "fixed".
///
/// The clamp order is load-bearing and is not the obvious one: the minimum
/// size is applied **before** the far-edge clamp, so a selection that would be
/// pushed past the edge is *slid back* rather than shrunk — and only if it
/// still does not fit is the width finally reduced. A port that clamped first
/// would return a rectangle smaller than the requested minimum.
// The reference's own eight-argument signature, kept argument for argument.
#[allow(clippy::too_many_arguments)]
pub fn norm_region(
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
    grid_w: usize,
    grid_h: usize,
    min_w: Option<usize>,
    min_h: Option<usize>,
) -> Region {
    let min_w = match min_w {
        Some(v) if v != 0 => v as i64,
        _ => 8,
    };
    let min_h = match min_h {
        Some(v) if v != 0 => v as i64,
        _ => 8,
    };
    let (gw, gh) = (grid_w as i64, grid_h as i64);

    let mut x = f64::min(x0, x1).floor() as i64;
    let mut y = f64::min(y0, y1).floor() as i64;
    let mut w = (x1 - x0).abs().ceil() as i64;
    let mut h = (y1 - y0).abs().ceil() as i64;
    if x < 0 {
        x = 0;
    }
    if y < 0 {
        y = 0;
    }
    if w < min_w {
        w = min_w;
    }
    if h < min_h {
        h = min_h;
    }
    if x + w > gw {
        x = i64::max(0, gw - w);
        w = i64::min(w, gw - x);
    }
    if y + h > gh {
        y = i64::max(0, gh - h);
        h = i64::min(h, gh - y);
    }
    Region {
        x: x.max(0) as usize,
        y: y.max(0) as usize,
        w: w.max(0) as usize,
        h: h.max(0) as usize,
    }
}

/// Per-tile pixel dimensions from [`tile_dims`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TileDims {
    pub w: usize,
    pub h: usize,
}

/// `tileDims(sel, cols, rows, ts)` (reference line 11536).
///
/// The reference's own comment: *"a tile spanning stepX×stepY coarse cells
/// gets its LONGER coarse edge = ts px, the shorter scaled to match — so the
/// assembled cols·tileW × rows·tileH image always keeps the selection's true
/// shape regardless of how many tiles you choose (no squish on non-square
/// selections). All tiles in a uniform region share the same dims."*
///
/// The `max(2, …)` floor on the short edge is why no shipped caller ever
/// reaches [`crate::region`]'s companion division-by-zero in
/// `amplifyRegion` — see that function's own note on the `outW == 1` case.
pub fn tile_dims(sel: &Region, cols: usize, rows: usize, ts: usize) -> TileDims {
    let aspect = (sel.w as f64 / cols as f64) / (sel.h as f64 / rows as f64);
    if aspect >= 1.0 {
        TileDims { w: ts, h: f64::max(2.0, js_round(ts as f64 / aspect)) as usize }
    } else {
        TileDims { w: f64::max(2.0, js_round(ts as f64 * aspect)) as usize, h: ts }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordered_corners_pass_through() {
        assert_eq!(norm_region(3.0, 4.0, 20.0, 18.0, 64, 48, None, None),
                   Region { x: 3, y: 4, w: 17, h: 14 });
    }

    #[test]
    fn corner_order_does_not_matter() {
        let a = norm_region(3.0, 4.0, 20.0, 18.0, 64, 48, None, None);
        let b = norm_region(20.0, 18.0, 3.0, 4.0, 64, 48, None, None);
        assert_eq!(a, b);
    }

    #[test]
    fn negative_overshoot_clamps_to_the_origin() {
        let r = norm_region(-9.0, -4.0, 12.0, 9.0, 64, 48, None, None);
        assert_eq!(r.x, 0);
        assert_eq!(r.y, 0);
    }

    #[test]
    fn a_tap_still_yields_the_minimum_size() {
        let r = norm_region(5.0, 5.0, 5.0, 5.0, 64, 48, None, None);
        assert_eq!(r.w, 8);
        assert_eq!(r.h, 8);
    }

    #[test]
    fn an_explicit_zero_minimum_means_eight_like_the_reference() {
        // `minW = minW || 8` -- 0 is falsy in JS.
        let with_zero = norm_region(5.0, 5.0, 5.0, 5.0, 64, 48, Some(0), Some(0));
        let with_none = norm_region(5.0, 5.0, 5.0, 5.0, 64, 48, None, None);
        assert_eq!(with_zero, with_none);
        assert_eq!(with_zero.w, 8);
    }

    #[test]
    fn a_selection_wider_than_the_grid_is_clipped_to_it() {
        let r = norm_region(0.0, 0.0, 200.0, 200.0, 64, 48, None, None);
        assert_eq!(r, Region { x: 0, y: 0, w: 64, h: 48 });
    }

    #[test]
    fn the_minimum_slides_the_rect_back_rather_than_shrinking_it() {
        // Dragged 1x1 at the far corner: min 16 does not fit past the edge, so
        // x slides back to 64-16 and the width is kept -- not clipped to 1.
        let r = norm_region(63.0, 47.0, 64.0, 48.0, 64, 48, Some(16), Some(16));
        assert_eq!(r.w, 16);
        assert_eq!(r.x, 64 - 16);
    }

    #[test]
    fn fractional_corners_floor_the_origin_and_ceil_the_extent() {
        let r = norm_region(1.7, 2.2, 9.4, 8.9, 64, 48, None, None);
        assert_eq!(r.x, 1);
        assert_eq!(r.y, 2);
        assert_eq!(r.w, 8); // ceil(7.7) = 8
        assert_eq!(r.h, 8); // ceil(6.7) = 7, raised to the 8 minimum
    }

    #[test]
    fn tile_dims_keeps_the_long_coarse_edge_at_the_tile_size() {
        let wide = tile_dims(&Region { x: 0, y: 0, w: 100, h: 20 }, 2, 1, 1024);
        assert_eq!(wide.w, 1024);
        assert!(wide.h < 1024);
        let tall = tile_dims(&Region { x: 0, y: 0, w: 20, h: 100 }, 1, 2, 1024);
        assert_eq!(tall.h, 1024);
        assert!(tall.w < 1024);
    }

    #[test]
    fn tile_dims_floors_the_short_edge_at_two_pixels() {
        let d = tile_dims(&Region { x: 0, y: 0, w: 1000, h: 3 }, 1, 1, 8);
        assert_eq!(d.h, 2);
        let d2 = tile_dims(&Region { x: 0, y: 0, w: 3, h: 1000 }, 1, 1, 8);
        assert_eq!(d2.w, 2);
    }

    #[test]
    fn a_square_selection_takes_the_aspect_ge_one_branch() {
        let d = tile_dims(&Region { x: 0, y: 0, w: 33, h: 33 }, 3, 3, 256);
        assert_eq!(d, TileDims { w: 256, h: 256 });
    }

    #[test]
    fn js_round_is_half_up_not_half_away_from_zero() {
        assert_eq!(js_round(0.5), 1.0);
        assert_eq!(js_round(1.5), 2.0);
        // Rust's own f64::round would give -1.0 here; JS gives -0.
        assert_eq!(js_round(-0.5), 0.0);
        assert_eq!(js_round(-1.5), -1.0);
    }

    #[test]
    fn to_float_widens_without_moving_the_rect() {
        let r = Region { x: 4, y: 6, w: 20, h: 14 };
        let f = r.to_float();
        assert_eq!((f.x, f.y, f.w, f.h), (4.0, 6.0, 20.0, 14.0));
    }
}
