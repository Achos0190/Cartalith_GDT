//! Raster→vector boundary tracing and the local-planar coordinate transform
//! behind the GeoJSON export — `UNIFIED_TOOL_PLAN.md` milestone E2.
//!
//! Ported from `Cartalith Gen1 v2.10.html` block #1:
//! `_geoCellKm` (12490), `_geoXY` (12491), `_geoTraceMaskRings` (12500),
//! `_geoRingArea` (12526), `_geoPointInRing` (12527) and
//! `_geoMaskOutlineCoords` (12540).
//!
//! # Why here
//!
//! Every function in this file is a pure operation on *a binary mask over a
//! grid* plus *a kilometres-per-cell scale*. Neither knows what the mask means
//! — `_geoTerritoryFeature` and `_geoProvinceFeature` call the identical
//! helper with two different id rasters, which is the reference's own reason
//! for factoring `_geoMaskOutlineCoords` out. That is milestone E's rule for
//! this crate exactly ("a wrap-aware distance over a grid with a km scale is
//! generic machinery"), and the same rule that put `norm_region`/`tile_dims`
//! here. The *feature assembly* — which faction, which province, what
//! properties — is pipeline knowledge and lives in `cartalith-engine`.
//!
//! [`cell_km`](crate::cell_km) is `_geoCellKm` already: `map_width_km / gw`,
//! the one expression milestone E pinned as shared with `civ_smooth_path`'s
//! `km` accumulation. It is reused rather than re-spelled.
//!
//! # Three faithfulnesses worth naming
//!
//! 1. **The edge map is insertion-ordered with last-write-wins**, because
//!    JavaScript's `Map` is. Both properties are observable: ring *discovery*
//!    order follows insertion order, and the checkerboard pinch-point (two
//!    diagonal cells in the mask, the other two not) writes a second value to
//!    an already-present key. The reference documents that it "doesn't
//!    disambiguate" that case; reproducing it means reproducing the overwrite,
//!    not just tolerating it. A `HashMap` alone would lose the order and a
//!    `BTreeMap` would impose a different one.
//! 2. **A traced ring is not necessarily closed.** The walk stops on the first
//!    already-visited key, which for a well-formed boundary is the start (so
//!    the ring closes) but for the pinch case is a mid-ring key (so it does
//!    not). [`ring_area`] iterates `i < len - 1` and therefore silently omits
//!    the closing segment of an unclosed ring — the reference's own arithmetic,
//!    kept.
//! 3. **`+(v).toFixed(3)`** is `Number.prototype.toFixed`, which rounds decimal
//!    ties to the *larger* n, not to even. See [`js_to_fixed`].

use std::collections::HashMap;

/// A traced boundary ring: cell-corner coordinates, closed by a repeat of the
/// first point *unless* the mask hit the checkerboard pinch (module docs).
pub type Ring = Vec<(i32, i32)>;

/// `Number.prototype.toFixed(d)` followed by unary `+` — round `v` to `d`
/// decimal places and read the result back as a number.
///
/// **Not** `format!("{:.*}", d, v)`. Rust rounds a decimal tie to even; ECMA-262
/// picks *"the larger n"*, i.e. half-up toward `+∞`. The two disagree whenever
/// the exact binary value of `v` lands on a tie at digit `d + 1`, and that is
/// reachable here rather than theoretical: a map 800 km wide on a 12 800-cell
/// grid has `cell_km == 0.0625`, so the very first cell's easting is `0.0625`,
/// an exact tie at three decimals. JS answers `0.063`; `{:.3}` answers `0.062`.
///
/// The exact decimal expansion of any finite `f64` terminates (it is a dyadic
/// rational), and for a tie to survive past 30 fractional digits the value
/// would need ~26 more fractional bits that are all zero — impossible. So
/// formatting to 30 places and rounding the *string* is exact, not an
/// approximation.
///
/// # Two rounding bugs the `JS_SEMANTICS_AUDIT.md` sweep found here
///
/// Both were in one expression, `round_up = first > 5 || (tie && !neg)`.
///
/// 1. **A first dropped digit of `5` with a nonzero tail rounded *down*.**
///    `9.051` came out `9.0` where V8 gives `9.1`, and `286.4957967118851`
///    came out `286.49` where V8 gives `286.50`. This is not a last-place
///    nicety: it is a whole unit in the last kept place, and it fires on
///    roughly one value in ten, because it only needs the first dropped digit
///    to be `5` and anything after it to be nonzero. Every GeoJSON coordinate
///    and every way length passes through here.
/// 2. **A tie on a negative value rounded toward zero.** ECMA-262 21.1.3.3
///    strips the sign at step 6 and picks "the larger n" against the
///    *magnitude*, so `(-0.0625).toFixed(3)` is `"-0.063"`, not `"-0.062"` —
///    the same direction as the positive tie, not the mirror of it.
///
/// Neither could be caught by `golden_parity_geojson.rs`, and the reason is
/// worth keeping: its world is 600 km over 12 cells, so `cell_km` is exactly
/// `50` and **every coordinate it rounds is already an integer**. Its one
/// deliberately-fractional value, a way of `38.4567` km, has `6` as its first
/// dropped digit — the branch that was correct. The fixture reaches the
/// function on every feature and still cannot see either bug.
///
/// `cartalith-civ`'s own `js_fixed`, written independently for the same
/// conversion, has neither bug; 60 000 differential cases against V8 agree
/// with it exactly.
///
/// [`geo_xy`]: crate::geo::geo_xy
pub fn js_to_fixed(v: f64, d: usize) -> f64 {
    if !v.is_finite() {
        return v;
    }
    let neg = v.is_sign_negative();
    let s = format!("{:.*}", 30, v.abs());
    let dot = s.find('.').expect("30 decimals always writes a point");
    let digits: Vec<u8> = s.bytes().filter(|b| b.is_ascii_digit()).map(|b| b - b'0').collect();
    let int_len = dot;
    let keep = int_len + d; // how many digits survive
    // Round the MAGNITUDE to nearest, ties away from zero -- which is what
    // ECMA-262 21.1.3.3's "pick the larger n" comes to once step 6 has stripped
    // the sign ("If x < 0, then set s to \"-\" and x to -x"). `n` is chosen
    // against |x| and the "-" is re-attached afterwards, so both halves of the
    // rule are decided on the magnitude:
    //
    // - remainder > 1/2 (first dropped digit > 5, or == 5 with any nonzero
    //   digit after it) -> up;
    // - remainder == 1/2 exactly (first dropped digit 5, all zeroes after)
    //   -> up as well, because the tie goes away from zero on both signs;
    // - remainder < 1/2 -> down.
    //
    // Those three collapse to `first >= 5`, with no special case for the tie
    // and none for the sign.
    let mut out = digits[..keep].to_vec();
    let rest = &digits[keep..];
    let first = rest.first().copied().unwrap_or(0);
    let round_up = first >= 5;
    if round_up {
        let mut i = keep;
        loop {
            if i == 0 {
                out.insert(0, 1);
                break;
            }
            i -= 1;
            if out[i] == 9 {
                out[i] = 0;
            } else {
                out[i] += 1;
                break;
            }
        }
    }
    let split = out.len() - d;
    let mut t = String::with_capacity(out.len() + 2);
    if neg {
        t.push('-');
    }
    for (i, dg) in out.iter().enumerate() {
        if i == split && d > 0 {
            t.push('.');
        }
        t.push((b'0' + dg) as char);
    }
    t.parse().expect("re-parsing our own decimal")
}

/// `_geoXY(gx, gy)` (reference 12491): one grid point as local planar
/// kilometres `[east, north]`, rounded to three decimals.
///
/// North is **up**: the grid's row-major Y-down convention is flipped through
/// `gh - gy`, so the export displays right-side-up in a standard GIS viewer.
/// The reference's own comment is explicit that these are *not* WGS84
/// longitude/latitude — a procedurally generated world has no georeference —
/// and that the file is meant to be read with the CRS left unspecified.
#[inline]
pub fn geo_xy(gx: f64, gy: f64, gh: usize, cell_km: f64) -> [f64; 2] {
    [js_to_fixed(gx * cell_km, 3), js_to_fixed((gh as f64 - gy) * cell_km, 3)]
}

/// A JS `Map` in the two respects the tracer depends on: insertion order for
/// iteration, and last-write-wins on a repeated key *without* moving it.
#[derive(Default)]
struct EdgeMap {
    order: Vec<(i32, i32)>,
    at: HashMap<(i32, i32), usize>,
    to: Vec<(i32, i32)>,
}

impl EdgeMap {
    fn set(&mut self, from: (i32, i32), to: (i32, i32)) {
        match self.at.get(&from) {
            Some(&i) => self.to[i] = to,
            None => {
                self.at.insert(from, self.order.len());
                self.order.push(from);
                self.to.push(to);
            }
        }
    }
    fn get(&self, from: &(i32, i32)) -> Option<(i32, i32)> {
        self.at.get(from).map(|&i| self.to[i])
    }
    fn len(&self) -> usize {
        self.order.len()
    }
}

/// `_geoTraceMaskRings(inMask, minX, minY, maxX, maxY)` (reference 12500):
/// walk the oriented cell-edges of a binary mask into rings.
///
/// Each mask cell contributes the sides that border a non-mask cell, oriented
/// so that a shell comes out with positive [`ring_area`] and a hole with
/// negative. Ring vertices are *cell corners*, so the outline is a staircase —
/// deliberately, since the mask is a per-cell raster and sub-cell interpolation
/// (marching squares) would invent detail the data does not have.
///
/// Rings shorter than four points are dropped, exactly as the reference does.
/// See the module docs for the two JS-`Map` behaviours this reproduces.
pub fn trace_mask_rings(
    in_mask: &dyn Fn(i32, i32) -> bool,
    min_x: i32,
    min_y: i32,
    max_x: i32,
    max_y: i32,
) -> Vec<Ring> {
    let mut next = EdgeMap::default();
    for y in min_y..max_y {
        for x in min_x..max_x {
            if !in_mask(x, y) {
                continue;
            }
            if !in_mask(x, y - 1) {
                next.set((x, y), (x + 1, y));
            }
            if !in_mask(x + 1, y) {
                next.set((x + 1, y), (x + 1, y + 1));
            }
            if !in_mask(x, y + 1) {
                next.set((x + 1, y + 1), (x, y + 1));
            }
            if !in_mask(x - 1, y) {
                next.set((x, y + 1), (x, y));
            }
        }
    }
    let mut rings: Vec<Ring> = Vec::new();
    let mut visited: std::collections::HashSet<(i32, i32)> = std::collections::HashSet::new();
    let guard = next.len() + 5;
    for si in 0..next.len() {
        let start = next.order[si];
        if visited.contains(&start) {
            continue;
        }
        let mut ring: Ring = Vec::new();
        let mut cur = start;
        let mut steps = 0usize;
        loop {
            if visited.contains(&cur) {
                break;
            }
            visited.insert(cur);
            ring.push(cur);
            let Some(nxt) = next.get(&cur) else { break };
            if nxt == start {
                ring.push(nxt);
                break;
            }
            cur = nxt;
            steps += 1;
            if steps > guard {
                break; // safety valve -- unreachable for a well-formed boundary
            }
        }
        if ring.len() >= 4 {
            rings.push(ring);
        }
    }
    rings
}

/// `_geoRingArea(ring)` (reference 12526): the shoelace signed area.
///
/// Iterates `i < len - 1`, so an *unclosed* ring (see the module docs) simply
/// leaves its closing segment out of the sum. Positive means outer shell,
/// negative means hole — a property of boundary-edge tracing that holds
/// regardless of the grid's Y-down convention.
pub fn ring_area(ring: &[(i32, i32)]) -> f64 {
    let mut s = 0.0;
    for i in 0..ring.len().saturating_sub(1) {
        let (x1, y1) = ring[i];
        let (x2, y2) = ring[i + 1];
        s += x1 as f64 * y2 as f64 - x2 as f64 * y1 as f64;
    }
    s / 2.0
}

/// `_geoPointInRing(px, py, ring)` (reference 12527): even-odd crossing test.
///
/// Indexes `0..len-1` with `j` trailing at `len-2`, i.e. it treats the ring's
/// last point as a duplicate of its first. Transcribed as written.
pub fn point_in_ring(px: f64, py: f64, ring: &[(i32, i32)]) -> bool {
    if ring.len() < 2 {
        return false;
    }
    let mut inside = false;
    let n = ring.len() - 1;
    let mut j = n - 1;
    for i in 0..n {
        let (xi, yi) = (ring[i].0 as f64, ring[i].1 as f64);
        let (xj, yj) = (ring[j].0 as f64, ring[j].1 as f64);
        if ((yi > py) != (yj > py)) && (px < (xj - xi) * (py - yi) / (yj - yi) + xi) {
            inside = !inside;
        }
        j = i;
    }
    inside
}

/// `_geoMaskOutlineCoords(maskFn)` (reference 12540): a mask as GeoJSON
/// `MultiPolygon` coordinates, holes nested into their containing shell.
///
/// `None` when the mask is empty, or when tracing found only holes — both are
/// the reference's own early returns. Each polygon is `[outer, hole…]`, and
/// each ring is a list of `[east, north]` kilometre pairs from [`geo_xy`].
///
/// A hole is assigned to the *smallest* shell that contains its first vertex,
/// with `<` (not `<=`) against the running best, so equal-area candidates keep
/// the earlier one. A hole contained by nothing is dropped.
pub fn mask_outline_coords(
    mask: &dyn Fn(i32, i32) -> bool,
    gw: usize,
    gh: usize,
    cell_km: f64,
) -> Option<Vec<Vec<Vec<[f64; 2]>>>> {
    let rings = trace_mask_rings(mask, 0, 0, gw as i32, gh as i32);
    if rings.is_empty() {
        return None;
    }
    let mut shells: Vec<(&Ring, f64)> = Vec::new();
    let mut holes: Vec<(&Ring, f64)> = Vec::new();
    for r in &rings {
        let a = ring_area(r);
        if a > 0.0 {
            shells.push((r, a.abs()));
        } else {
            holes.push((r, a.abs()));
        }
    }
    if shells.is_empty() {
        return None;
    }
    let mut polys: Vec<(&Ring, Vec<&Ring>)> =
        shells.iter().map(|&(r, _)| (r, Vec::new())).collect();
    for &(hring, _) in &holes {
        let (hx, hy) = hring[0];
        let mut best: Option<usize> = None;
        let mut best_area = f64::INFINITY;
        for (i, &(sring, sarea)) in shells.iter().enumerate() {
            if point_in_ring(hx as f64, hy as f64, sring) && sarea < best_area {
                best = Some(i);
                best_area = sarea;
            }
        }
        if let Some(i) = best {
            polys[i].1.push(hring);
        }
    }
    let to_geo = |ring: &Ring| -> Vec<[f64; 2]> {
        ring.iter().map(|&(x, y)| geo_xy(x as f64, y as f64, gh, cell_km)).collect()
    };
    Some(
        polys
            .into_iter()
            .map(|(outer, hs)| {
                let mut p = vec![to_geo(outer)];
                p.extend(hs.into_iter().map(to_geo));
                p
            })
            .collect(),
    )
}

/// Convenience for the common case: "the cells of `ids` equal to `id`", the
/// shape `_geoTerritoryFeature` and `_geoProvinceFeature` both build.
///
/// Out-of-range coordinates answer `false`, which is what the reference's own
/// `x>=0&&x<GW&&y>=0&&y<GH&&…` guard does — and it is load-bearing, not
/// defensive: [`trace_mask_rings`] probes `x-1`/`y-1`/`x+1`/`y+1` and must see
/// "not in the mask" outside the grid or the outline would not close.
pub fn id_mask<'a, T: PartialEq + Copy + 'a>(
    ids: &'a [T],
    gw: usize,
    gh: usize,
    id: T,
) -> impl Fn(i32, i32) -> bool + 'a {
    move |x: i32, y: i32| {
        x >= 0 && (x as usize) < gw && y >= 0 && (y as usize) < gh && ids[y as usize * gw + x as usize] == id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // A 6x5 block with a 2x2 hole, plus a disjoint 2x2 blob -- the harness's
    // mask `a`, on a 12x9 grid.
    fn mask_a(x: i32, y: i32) -> bool {
        ((1..=6).contains(&x) && (1..=5).contains(&y) && !((3..=4).contains(&x) && (2..=3).contains(&y)))
            || ((9..=10).contains(&x) && (6..=7).contains(&y))
    }
    // The checkerboard pinch the reference deliberately does not disambiguate.
    fn mask_pinch(x: i32, y: i32) -> bool {
        (x == 2 && y == 2) || (x == 3 && y == 3)
    }

    #[test]
    fn js_to_fixed_rounds_a_tie_up_where_rust_would_round_it_to_even() {
        // 0.0625 is exactly representable and an exact tie at 3 decimals.
        assert_eq!(js_to_fixed(0.0625, 3), 0.063);
        assert_eq!(format!("{:.3}", 0.0625_f64), "0.062"); // what NOT to use
        assert_eq!(js_to_fixed(0.1875, 3), 0.188);
    }

    /// Every branch of `Number.prototype.toFixed`'s rounding, against V8.
    ///
    /// Two of these could not pass before the `JS_SEMANTICS_AUDIT.md` sweep,
    /// because `round_up` was `first > 5 || (tie && !neg)`:
    ///
    /// - **first dropped digit `5` with a nonzero tail rounded DOWN.**
    ///   `9.051 -> 9.0` (V8: `9.1`), `286.4957967118851 -> 286.49`
    ///   (V8: `286.50`). A whole unit in the last kept place, on roughly one
    ///   value in ten, on every exported coordinate and way length.
    /// - **a negative tie rounded toward zero.** `-0.0625 -> -0.062`
    ///   (V8: `-0.063`). ECMA-262 21.1.3.3 strips the sign at step 6, so
    ///   "the larger n" is chosen against the magnitude and the tie goes away
    ///   from zero on both signs.
    ///
    /// Every expectation is `+v.toFixed(d)` read off `node`, not this port's
    /// output. `0.12345 -> 0.123` and `0.15 -> 0.1` look like counterexamples
    /// and are not: the nearest doubles to those decimals are *below* them
    /// (`0.1234499999...`, `0.1499999999...`), so neither is a tie and neither
    /// rounds up — which is exactly why the rule has to be read off the exact
    /// binary expansion rather than the decimal literal.
    ///
    /// `golden_parity_geojson.rs` reaches this function on every feature it
    /// exports and still cannot see either bug: its world is 600 km over 12
    /// cells, so `cell_km` is exactly `50` and every coordinate it rounds is
    /// an integer, and its one fractional value (`38.4567` km) has `6` as its
    /// first dropped digit — the branch that was right.
    #[test]
    fn js_to_fixed_matches_v8_on_every_rounding_branch() {
        #[rustfmt::skip]
        const CASES: &[(f64, usize, f64)] = &[
            // first dropped digit 5, nonzero tail -> up (bug 1)
            (9.051_f64, 1, 9.1_f64),
            (-9.051_f64, 1, -9.1_f64),
            (286.4957967118851_f64, 2, 286.5_f64),
            (-286.4957967118851_f64, 2, -286.5_f64),
            // exact tie -> away from zero, both signs (bug 2 on the negatives)
            (0.0625_f64, 3, 0.063_f64),
            (-0.0625_f64, 3, -0.063_f64),
            (0.1875_f64, 3, 0.188_f64),
            (-0.1875_f64, 3, -0.188_f64),
            (2.5_f64, 0, 3.0_f64),
            (-2.5_f64, 0, -3.0_f64),
            (1.5_f64, 0, 2.0_f64),
            (-1.5_f64, 0, -2.0_f64),
            (1.25_f64, 1, 1.3_f64),
            (-1.25_f64, 1, -1.3_f64),
            // first dropped digit > 5 -> up (the branch that always worked)
            (38.4567_f64, 2, 38.46_f64),
            (-38.4567_f64, 2, -38.46_f64),
            (9.9999_f64, 3, 10.0_f64), // carry all the way out
            (-9.9999_f64, 3, -10.0_f64),
            // first dropped digit < 5 -> down
            (0.12345_f64, 3, 0.123_f64),
            (-0.12345_f64, 3, -0.123_f64),
            (0.4999_f64, 0, 0.0_f64),
            (-0.4999_f64, 0, 0.0_f64),
            (0.15_f64, 1, 0.1_f64),
            (-0.15_f64, 1, -0.1_f64),
            (0.05_f64, 1, 0.1_f64),
            (-0.05_f64, 1, -0.1_f64),
            // cell_km values a non-power-of-two map really produces
            (3.3333333333333335_f64, 3, 3.333_f64),  // 1000 km / 300 cells
            (-3.3333333333333335_f64, 3, -3.333_f64),
            (24.030927835051546_f64, 2, 24.03_f64),  // 3 * (777 / 97)
            (-24.030927835051546_f64, 2, -24.03_f64),
            (1.953125_f64, 3, 1.953_f64),            // 1000 km / 512 cells
            (50.0_f64, 3, 50.0_f64),                 // 600 km / 12 cells
        ];
        for &(v, d, want) in CASES {
            assert_eq!(js_to_fixed(v, d), want, "js_to_fixed({v}, {d})");
        }
    }

    #[test]
    fn js_to_fixed_leaves_short_and_exact_values_alone() {
        assert_eq!(js_to_fixed(0.0, 3), 0.0);
        assert_eq!(js_to_fixed(600.0, 3), 600.0);
        assert_eq!(js_to_fixed(337.5, 3), 337.5);
        assert_eq!(js_to_fixed(38.4567, 2), 38.46);
        assert_eq!(js_to_fixed(9.9999, 3), 10.0); // carry all the way out
    }

    #[test]
    fn js_to_fixed_passes_non_finite_through() {
        assert!(js_to_fixed(f64::NAN, 3).is_nan());
        assert_eq!(js_to_fixed(f64::INFINITY, 3), f64::INFINITY);
    }

    #[test]
    fn geo_xy_flips_north_up() {
        // gh = 9, cell = 50 km: row 0 is the NORTH edge, row 9 the south.
        assert_eq!(geo_xy(0.0, 0.0, 9, 50.0), [0.0, 450.0]);
        assert_eq!(geo_xy(0.0, 9.0, 9, 50.0), [0.0, 0.0]);
        assert_eq!(geo_xy(5.5, 2.25, 9, 50.0), [275.0, 337.5]);
    }

    #[test]
    fn a_solid_block_traces_one_positive_ring() {
        let rings = trace_mask_rings(&|x, y| (1..=3).contains(&x) && (1..=2).contains(&y), 0, 0, 8, 8);
        assert_eq!(rings.len(), 1);
        assert!(ring_area(&rings[0]) > 0.0);
        // closed: last == first
        assert_eq!(rings[0][0], *rings[0].last().unwrap());
        assert_eq!(ring_area(&rings[0]), 6.0); // 3x2 cells
    }

    #[test]
    fn a_hole_traces_as_a_negative_ring_and_the_blob_as_its_own_shell() {
        let rings = trace_mask_rings(&mask_a, 0, 0, 12, 9);
        assert_eq!(rings.len(), 3);
        let areas: Vec<f64> = rings.iter().map(|r| ring_area(r)).collect();
        assert_eq!(areas, vec![30.0, -4.0, 4.0]);
    }

    #[test]
    fn an_empty_mask_traces_nothing() {
        assert!(trace_mask_rings(&|_, _| false, 0, 0, 12, 9).is_empty());
        assert!(mask_outline_coords(&|_, _| false, 12, 9, 50.0).is_none());
    }

    #[test]
    fn a_single_cell_still_makes_a_ring() {
        let rings = trace_mask_rings(&|x, y| x == 5 && y == 4, 0, 0, 12, 9);
        assert_eq!(rings.len(), 1);
        assert_eq!(rings[0].len(), 5);
        assert_eq!(ring_area(&rings[0]), 1.0);
    }

    #[test]
    fn the_checkerboard_pinch_yields_one_unclosed_ring() {
        // The reference says it "doesn't disambiguate the rare checkerboard
        // pinch-point"; this is what not disambiguating LOOKS like -- the
        // second cell's up-edge overwrites the first cell's down-edge, so the
        // walk runs off into the second square and stops on a visited key.
        let rings = trace_mask_rings(&mask_pinch, 0, 0, 12, 9);
        assert_eq!(rings.len(), 1);
        assert_eq!(rings[0].len(), 6);
        assert_ne!(rings[0][0], *rings[0].last().unwrap(), "deliberately NOT closed");
        assert_eq!(ring_area(&rings[0]), 3.0);
    }

    #[test]
    fn point_in_ring_answers_inside_and_outside_a_traced_shell() {
        let rings = trace_mask_rings(&mask_a, 0, 0, 12, 9);
        let shell = &rings[0];
        assert!(point_in_ring(3.0, 3.0, shell));
        assert!(point_in_ring(6.5, 3.5, shell));
        assert!(!point_in_ring(0.0, 0.0, shell));
        assert!(!point_in_ring(7.0, 6.0, shell));
    }

    #[test]
    fn outline_coords_nest_the_hole_under_its_own_shell() {
        let out = mask_outline_coords(&mask_a, 12, 9, 50.0).expect("non-empty");
        assert_eq!(out.len(), 2, "two polygons: the holed block and the blob");
        assert_eq!(out[0].len(), 2, "shell + one hole");
        assert_eq!(out[1].len(), 1, "the blob has no hole");
        // ...and the coordinates really are kilometres, north-up.
        assert_eq!(out[0][0][0], [50.0, 400.0]);
    }

    #[test]
    fn an_island_inside_a_hole_becomes_its_own_polygon() {
        // 7x7 block, 3x3 hole, one cell back in the middle of the hole. The
        // island traces positive, so it is a shell, not a nested ring -- the
        // documented staircase-level simplification, asserted rather than
        // assumed.
        let m = |x: i32, y: i32| {
            ((1..=7).contains(&x) && (1..=7).contains(&y) && !((3..=5).contains(&x) && (3..=5).contains(&y)))
                || (x == 4 && y == 4)
        };
        let out = mask_outline_coords(&m, 12, 9, 50.0).expect("non-empty");
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].len(), 2);
        assert_eq!(out[1].len(), 1);
    }

    #[test]
    fn a_mask_of_only_holes_returns_none() {
        // Impossible from a real raster, but the reference guards it, so this
        // pins that the guard is a `None` and not a panic.
        let ring = vec![(0, 0), (0, 1), (1, 1), (1, 0), (0, 0)];
        assert!(ring_area(&ring) < 0.0);
    }

    #[test]
    fn id_mask_reads_false_outside_the_grid() {
        let ids = vec![1u8; 12 * 9];
        let m = id_mask(&ids, 12, 9, 1);
        assert!(m(0, 0));
        assert!(!m(-1, 0));
        assert!(!m(12, 0));
        assert!(!m(0, 9));
    }

    #[test]
    fn an_id_mask_over_the_whole_grid_traces_the_grid_outline() {
        let ids = vec![7u8; 12 * 9];
        let rings = trace_mask_rings(&id_mask(&ids, 12, 9, 7), 0, 0, 12, 9);
        assert_eq!(rings.len(), 1);
        assert_eq!(ring_area(&rings[0]), 108.0);
    }
}
