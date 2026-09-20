//! Level-set contour tracing over a scalar grid — the shared
//! raster→polyline primitive **EF-6** asks for
//! (`ELEVATION_FIELD_ARCHITECTURE_RESEARCH.md` §5).
//!
//! One function, [`contour_polylines`]: given a scalar field over a grid and a
//! threshold, return the level set `field == level` as linked polylines in the
//! same pixel-space convention `trace_river_polylines` already returns (a cell
//! centre is `(col + 0.5, row + 0.5)`).
//!
//! # New capability, not a port — flagged, per `cartalith-porting-discipline`
//!
//! There is no reference function this ports and **no golden value to match**.
//! `Cartalith Gen1` renders elevation isolines (`viz.contours`, the *"contour
//! veins"* style, `Cartalith Gen1 v2.11.html:7954`) but does it **per pixel in
//! height space** — `d = |r/iv − round(r/iv)| · iv`, a darkening term, never a
//! point or a polyline — and its coastline is likewise only the land/water
//! decision each pixel makes for itself. So nothing here has a reference
//! answer to be checked against. The rule for this case is the skill's own:
//! flag it, explain why, do not assume it correct. Nothing here changes the
//! output of any existing function — this module is additive and has no caller
//! inside this crate.
//!
//! # What already existed, and why this is still not a duplicate
//!
//! EF-6's own text says *"there is no contour or coastline vectorization
//! anywhere in this codebase (grepped for `marching_squares`/`contour`/
//! `trace_coastline`, zero hits)."* That grep is true of those three names and
//! **misses two real tracers**, both checked at the symbol here rather than
//! taken from the document:
//!
//! - [`crate::geo::trace_mask_rings`] — the sibling of this function, in this
//!   crate, one module over. It traces a **binary mask**, and its own docs are
//!   explicit that the staircase is deliberate: *"sub-cell interpolation
//!   (marching squares) would invent detail the data does not have."* That
//!   reasoning is right for a mask and does not transfer to a **continuous**
//!   field. Where a coastline crosses a cell edge between `field = 0.39` and
//!   `field = 0.45` at `sea_level = 0.42`, the crossing fraction is *measured*
//!   data, not invented — so this module interpolates and that one does not,
//!   and both are correct for their input. `trace_mask_rings` is also a
//!   golden-pinned port (`_geoTraceMaskRings`, reference 12500) feeding the
//!   GeoJSON export; it is not touched.
//! - `cartalith_terrain::trace_boundaries` — a skeleton chain-walk that
//!   already vectorises `boundary_mask` into typed polylines. See
//!   `cartalith_terrain::vector`'s own header for what that means for EF-6's
//!   fault-line half, which is the part of EF-6 that turns out to be already
//!   built.
//!
//! # Why `cartalith-spatial`
//!
//! `UNIFIED_TOOL_PLAN.md` milestone B/C's placement rule, applied the way
//! [`cartalith_terrain::amplify`](../../cartalith_terrain/amplify/index.html)'s
//! and `cartalith-engine`'s `elevation` module each apply it to themselves:
//! subsystem-domain math belongs to the crate that owns the field;
//! `cartalith-engine` orchestrates and does not compute; `cartalith-spatial`
//! holds *generic machinery with no Cartalith-specific opinion*.
//!
//! This function knows nothing about height, sea level, plates or biomes — it
//! takes a `&[f32]`, a width, a height and an `f64`. The same call traces a
//! coastline, an isobath, a rainfall isohyet or a temperature isotherm, and it
//! could not tell you which it had just done. That is milestone C's test
//! exactly, and it is the same test that put `trace_mask_rings`,
//! `norm_region` and `tile_dims` in this crate. The *sea-level-aware*
//! half — "a coastline is the level set of the height field at
//! `WorldState::sea_level`" — is height-field knowledge and lives in
//! `cartalith_terrain::vector`, on the far side of this boundary.
//!
//! # The algorithm, and the two decisions inside it
//!
//! Marching squares over **cell centres**: each 2×2 block of adjacent cell
//! centres is one marching-squares cell, so a `w × h` field yields
//! `(w-1) × (h-1)` blocks and the traced contour lives strictly inside the
//! convex hull of the cell centres. Crossings are placed on block edges by
//! linear interpolation between the two centre values.
//!
//! **1. Saddle disambiguation is by the block's centre average.** Cases 5 and
//! 10 (the two diagonal configurations) have two valid readings, and this
//! module resolves them with the mean of the four corner values against
//! `level` — the standard asymptotic-decider approximation. Deterministic, and
//! stated because "marching squares" alone does not specify it.
//!
//! **2. Every segment is oriented with the above-`level` side on its visual
//! left** (this workspace's grids are row-major Y-down, so "visual left" of a
//! direction `(dx, dy)` is `(dy, -dx)`). That makes a closed ring around an
//! above-level island wind consistently, so [`crate::geo::ring_area`]'s sign
//! test answers *island or hole* on this output too, and it is what lets the
//! linker below assume each crossing is the start of exactly one segment and
//! the end of exactly one other.
//!
//! # Determinism, which is a property of the linking and not of the tracing
//!
//! Emitting segments is trivially deterministic (a scan in `y`-then-`x` order).
//! Linking them is where a contour tracer usually stops being deterministic,
//! because the obvious implementation keys crossings in a hash map and then
//! *iterates it* to find chain starts. This one never does: a crossing's
//! identity is an exact integer — the block edge it lies on — so the lookup is
//! a flat `Vec` index, not a hash, and every chain start is found by scanning
//! the segment list in emission order. Two calls on the same input return
//! byte-identical output, in the same order, in the same process or across
//! processes.
//!
//! # Three limits, stated rather than discovered later
//!
//! - **No X wrap.** A world map's east edge is its west edge, and this does not
//!   know that: a contour crossing the antimeridian comes back as two open
//!   chains ending at the grid edge. That matches what the renderer already
//!   does with rivers — `split_river_polylines` cuts them at the seam
//!   regardless — but a consumer measuring *length* or *closure* across the
//!   seam must handle it.
//! - **The traced line stops half a cell short of the grid border**, because
//!   the outermost cell centres are the outermost samples. A landmass running
//!   off the edge of the map yields an open chain, not a closed ring.
//! - **Transient memory is `2·w·h` entries of `u32` plus `2·w·h` of `bool`** —
//!   about 27 MB at this port's largest shipping world (2048 × 1311). It is
//!   freed on return. A tile-scoped caller pays a tile-scoped cost.

/// The level set `field == level`, as linked polylines in pixel space
/// (`col + 0.5`, `row + 0.5` for a cell centre).
///
/// A closed ring repeats its first point as its last; an open chain (one that
/// runs off the sampled area) does not. Runs of fewer than two points are
/// impossible by construction and none are emitted. Returns empty when the
/// field never crosses `level`, and when `w < 2` or `h < 2` — with one row or
/// one column there is no 2×2 block to march over, which is the honest answer
/// rather than a degenerate one.
///
/// Order is stable and meaningful: open chains first, in the emission order of
/// their heads, then closed rings in the emission order of their first
/// segment. Emission order is the `y`-then-`x` block scan.
///
/// `NaN` in the field behaves as below `level` (`>=` is false for `NaN`), so a
/// `NaN` region reads as a hole rather than poisoning the whole trace. That is
/// a consequence worth naming, not a designed feature: a field with `NaN` in
/// it has a defect upstream of here.
/// One marching-squares segment: the block edge it starts on, the one it ends
/// on, and the two interpolated points. The edges are the identity the linker
/// works in; the points are only carried so they are computed once.
type Seg = (usize, usize, (f64, f64), (f64, f64));

pub fn contour_polylines(field: &[f32], w: usize, h: usize, level: f64) -> Vec<Vec<(f64, f64)>> {
    if w < 2 || h < 2 || field.len() < w * h {
        return Vec::new();
    }

    // A crossing's identity is the block edge it lies on, and a block edge is
    // named by the pair of cell centres it separates: horizontal edge `(x, y)`
    // joins centres `(x, y)` and `(x+1, y)`; vertical edge `(x, y)` joins
    // `(x, y)` and `(x, y+1)`. Both index into `0..w*h`, with the vertical set
    // offset by `w*h`, so the whole edge space is a flat `Vec` index and the
    // linker below needs no hashing (see the module docs on determinism).
    let n = w * h;
    let h_edge = |x: usize, y: usize| y * w + x;
    let v_edge = |x: usize, y: usize| n + y * w + x;

    let at = |x: usize, y: usize| field[y * w + x] as f64;
    // Where the level crosses between two samples, as a fraction of the way
    // from `a` to `b`. `a` and `b` are on opposite sides of `level` at every
    // call site, so they differ and this never divides by zero.
    let frac = |a: f64, b: f64| (level - a) / (b - a);

    let mut segs: Vec<Seg> = Vec::new();

    for y in 0..h - 1 {
        for x in 0..w - 1 {
            let (tl, tr, br, bl) = (at(x, y), at(x + 1, y), at(x + 1, y + 1), at(x, y + 1));
            let case = (tl >= level) as usize
                | (((tr >= level) as usize) << 1)
                | (((br >= level) as usize) << 2)
                | (((bl >= level) as usize) << 3);
            if case == 0 || case == 15 {
                continue;
            }

            // The four block edges, in marching-squares order: 0 top, 1 right,
            // 2 bottom, 3 left. Each is computed only if the contour actually
            // uses it, which the case index already decided.
            let pt = |e: usize| -> ((f64, f64), usize) {
                match e {
                    0 => (
                        (x as f64 + 0.5 + frac(tl, tr), y as f64 + 0.5),
                        h_edge(x, y),
                    ),
                    1 => (
                        (x as f64 + 1.5, y as f64 + 0.5 + frac(tr, br)),
                        v_edge(x + 1, y),
                    ),
                    2 => (
                        (x as f64 + 0.5 + frac(bl, br), y as f64 + 1.5),
                        h_edge(x, y + 1),
                    ),
                    _ => (
                        (x as f64 + 0.5, y as f64 + 0.5 + frac(tl, bl)),
                        v_edge(x, y),
                    ),
                }
            };

            // Directed so the above-`level` side is on the segment's visual
            // left (module docs). The complement case is always the reverse of
            // its partner, which is what makes that property hold globally.
            let pairs: &[(usize, usize)] = match case {
                1 => &[(3, 0)],
                2 => &[(0, 1)],
                3 => &[(3, 1)],
                4 => &[(1, 2)],
                6 => &[(0, 2)],
                7 => &[(3, 2)],
                8 => &[(2, 3)],
                9 => &[(2, 0)],
                11 => &[(2, 1)],
                12 => &[(1, 3)],
                13 => &[(1, 0)],
                14 => &[(0, 3)],
                // The two saddles, resolved by the block's centre average.
                5 => {
                    if (tl + tr + br + bl) * 0.25 >= level {
                        &[(1, 0), (3, 2)]
                    } else {
                        &[(3, 0), (1, 2)]
                    }
                }
                _ => {
                    if (tl + tr + br + bl) * 0.25 >= level {
                        &[(0, 3), (2, 1)]
                    } else {
                        &[(0, 1), (2, 3)]
                    }
                }
            };

            for &(a, b) in pairs {
                let (pa, ka) = pt(a);
                let (pb, kb) = pt(b);
                segs.push((ka, kb, pa, pb));
            }
        }
    }

    if segs.is_empty() {
        return Vec::new();
    }

    // Link. Each crossing is the `from` of at most one segment and the `to` of
    // at most one other — a consequence of the consistent orientation above,
    // since a shared block edge is entered from one block and left into the
    // other. `or_insert` semantics are kept explicit for the degenerate input
    // that would break that (it would orphan a segment into its own run rather
    // than lose it).
    const NONE: u32 = u32::MAX;
    let mut from_of = vec![NONE; 2 * n];
    let mut is_to = vec![false; 2 * n];
    for (i, s) in segs.iter().enumerate() {
        if from_of[s.0] == NONE {
            from_of[s.0] = i as u32;
        }
        is_to[s.1] = true;
    }

    let mut used = vec![false; segs.len()];
    let mut out: Vec<Vec<(f64, f64)>> = Vec::new();

    let walk = |start: usize, used: &mut [bool]| -> Vec<(f64, f64)> {
        let mut pts = vec![segs[start].2];
        let mut cur = start;
        loop {
            used[cur] = true;
            pts.push(segs[cur].3);
            let nxt = from_of[segs[cur].1];
            if nxt == NONE || used[nxt as usize] {
                break;
            }
            cur = nxt as usize;
        }
        pts
    };

    // Open chains first: a head is a segment whose `from` nothing links into.
    for s in 0..segs.len() {
        if used[s] || is_to[segs[s].0] {
            continue;
        }
        let pl = walk(s, &mut used);
        out.push(pl);
    }
    // Then closed rings. A ring's walk stops when it re-reaches its own first
    // segment, having already pushed that segment's `from` point as its last
    // point — so the ring arrives closed and needs no explicit repeat.
    for s in 0..segs.len() {
        if used[s] {
            continue;
        }
        let pl = walk(s, &mut used);
        out.push(pl);
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A field that is `x - cut`, so the level set is the vertical line
    /// `x == cut` in cell-index space and `cut + 0.5` in pixel space.
    fn ramp(w: usize, h: usize, cut: f64) -> Vec<f32> {
        (0..w * h).map(|i| ((i % w) as f64 - cut) as f32).collect()
    }

    #[test]
    fn a_linear_ramp_traces_one_straight_chain_at_the_exact_crossing() {
        let (w, h) = (24usize, 9usize);
        let pls = contour_polylines(&ramp(w, h, 10.3), w, h, 0.0);
        assert_eq!(pls.len(), 1, "one crossing, one chain");
        // (h-1) blocks vertically, each contributing one segment, linked into
        // h points.
        assert_eq!(pls[0].len(), h, "one point per sampled row");
        for &(px, _) in &pls[0] {
            // Interpolating a field that is exactly linear in x is exact in
            // the arithmetic; the residual 1.2e-8 is the field's own `f32`
            // storage of `-0.3`, measured rather than assumed away. A tighter
            // bound here would be asserting `f32` round-trips a decimal.
            assert!((px - 10.8).abs() < 1e-6, "x = {px}, want 10.8");
        }
        // Open, not closed: it runs off the sampled area top and bottom.
        assert_ne!(pls[0][0], pls[0][pls[0].len() - 1]);
    }

    #[test]
    fn a_circle_is_traced_as_one_closed_ring_on_the_circle() {
        let (w, h, r) = (61usize, 61usize, 20.0f64);
        let (cx, cy) = (30.5f64, 30.5f64);
        // Positive inside the disc, negative outside — an island.
        let field: Vec<f32> = (0..w * h)
            .map(|i| {
                let (x, y) = ((i % w) as f64 + 0.5, (i / w) as f64 + 0.5);
                (r - ((x - cx).powi(2) + (y - cy).powi(2)).sqrt()) as f32
            })
            .collect();
        let pls = contour_polylines(&field, w, h, 0.0);
        assert_eq!(pls.len(), 1, "one island, one ring");
        let ring = &pls[0];
        assert!(ring.len() > 100, "{} points is too coarse for r=20", ring.len());
        assert_eq!(ring[0], ring[ring.len() - 1], "a ring closes");
        let worst = ring
            .iter()
            .map(|&(x, y)| (((x - cx).powi(2) + (y - cy).powi(2)).sqrt() - r).abs())
            .fold(0.0f64, f64::max);
        // The only error is the chord-vs-arc term of interpolating a field
        // that is linear in *radius* along a straight cell edge, which is
        // O(1/(8r)) ≈ 0.006 cells here. 0.02 is a ceiling with room, not a
        // fitted number.
        assert!(worst < 0.02, "worst radial error {worst} cells");
    }

    #[test]
    fn an_island_ring_winds_the_opposite_way_from_a_lake_ring() {
        let (w, h) = (41usize, 41usize);
        let disc = |sign: f64| -> Vec<f32> {
            (0..w * h)
                .map(|i| {
                    let (x, y) = ((i % w) as f64 + 0.5, (i / w) as f64 + 0.5);
                    (sign * (12.0 - ((x - 20.5).powi(2) + (y - 20.5).powi(2)).sqrt())) as f32
                })
                .collect()
        };
        let island = contour_polylines(&disc(1.0), w, h, 0.0);
        let lake = contour_polylines(&disc(-1.0), w, h, 0.0);
        let area = |r: &[(f64, f64)]| {
            let s: f64 = r.windows(2).map(|p| p[0].0 * p[1].1 - p[1].0 * p[0].1).sum();
            s / 2.0
        };
        let (a, b) = (area(&island[0]), area(&lake[0]));
        assert!(a * b < 0.0, "island {a} and lake {b} must wind oppositely");
    }

    #[test]
    fn the_same_field_traces_byte_identically_twice() {
        let (w, h) = (37usize, 29usize);
        // Something with several components and at least one saddle: a
        // product of sines crosses zero on a checkerboard of lobes.
        let field: Vec<f32> = (0..w * h)
            .map(|i| {
                let (x, y) = ((i % w) as f64, (i / w) as f64);
                ((x * 0.41).sin() * (y * 0.37).sin() - 0.12) as f32
            })
            .collect();
        let a = contour_polylines(&field, w, h, 0.0);
        let b = contour_polylines(&field, w, h, 0.0);
        assert!(a.len() > 3, "expected several components, got {}", a.len());
        assert_eq!(a, b);
    }

    #[test]
    fn a_field_that_never_crosses_the_level_traces_nothing() {
        let f = vec![1.0f32; 16 * 16];
        assert!(contour_polylines(&f, 16, 16, 0.0).is_empty(), "all above");
        assert!(contour_polylines(&f, 16, 16, 2.0).is_empty(), "all below");
    }

    #[test]
    fn a_degenerate_grid_traces_nothing_rather_than_panicking() {
        let f = ramp(8, 1, 3.5);
        assert!(contour_polylines(&f, 8, 1, 0.0).is_empty(), "one row");
        assert!(contour_polylines(&f, 1, 8, 0.0).is_empty(), "one column");
        assert!(contour_polylines(&[], 0, 0, 0.0).is_empty(), "empty");
        // Short buffer: refused, not indexed past the end.
        assert!(contour_polylines(&f, 8, 4, 0.0).is_empty(), "short buffer");
    }

    #[test]
    fn a_saddle_resolves_to_two_segments_and_links_both() {
        // A 2x2 field with the two diagonals on opposite sides of 0 is
        // marching-squares case 10 (TR and BL above), the ambiguous one. The
        // centre average is 0, which `>=` reads as above — so the two *below*
        // corners separate, giving two segments and two open chains.
        let field = vec![-1.0f32, 1.0, 1.0, -1.0];
        let pls = contour_polylines(&field, 2, 2, 0.0);
        assert_eq!(pls.len(), 2, "a saddle emits two runs");
        for pl in &pls {
            assert_eq!(pl.len(), 2, "each is one segment");
        }
    }
}
