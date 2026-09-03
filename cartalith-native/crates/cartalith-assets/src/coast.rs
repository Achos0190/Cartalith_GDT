//! The coastline, as the *"snap sea marks to coast"* rule needs it.
//!
//! **Nothing here is ported.** `cartography_workspace.gd`'s third placement
//! rule (`cartalith-dcc-parts.js:393`'s `icoRules`) has no reference
//! implementation to transcribe — the reference has no sea-marks family and no
//! generated icon-placement pass at all — so this is a port addition under the
//! owner's 2026-09-02 ruling, written against the rule's own one-line
//! statement and nothing else.
//!
//! # What "the coast" is here, stated rather than assumed
//!
//! A **coast cell is a water cell with at least one land neighbour across an
//! edge** (4-neighbourhood). Not a land cell touching water, and not the
//! 8-neighbourhood.
//!
//! - *Water, not land*, because that is what the family is. Five of
//!   [`crate::PACK_SEAMARK_SLOTS`]' eight — buoy, anchorage, shipwreck, reef,
//!   shoal, whirlpool — are things that exist only in water; a lighthouse or a
//!   beacon drawn one cell offshore reads as standing on the point it marks,
//!   while a buoy drawn one cell inland reads as a mistake. The asymmetry
//!   decides it.
//! - *4-neighbourhood*, because a diagonal-only touch is where two water bodies
//!   pinch past each other, not a shore anything can be moored against. This
//!   also makes the definition agree with [`cartalith_noise`]-free arithmetic:
//!   no distance field, no epsilon, just four index reads.
//!
//! `sea` is the same threshold every other consumer of a height field in this
//! workspace uses — `h <= sea` is water — and the comparison is written that
//! way round on purpose: `!(h > sea)` and `h <= sea` differ on NaN, and a NaN
//! cell must read as *water it is not safe to place on* rather than as land.
//! See the workspace rule in `cartalith-rust-conventions`.
//!
//! # Why this lives in `cartalith-assets`
//!
//! It is asset placement, it needs nothing but a height field, and the sea-mark
//! family it serves is defined in `slots.rs` next door. The *rest* of the
//! generated placement pass — min spacing, label avoidance — lives in
//! `cartalith-godot`'s `icon_bridge` instead, because it reuses
//! `cartalith_civ::labels::LabelRect` and this crate does not depend on
//! `cartalith-civ`. That split is stated in `icon_bridge.rs`'s own module doc.

/// Whether cell `(x, y)` is water: `h <= sea`, and a non-finite height counts
/// as water so an unmeasurable cell is never offered as somewhere to stand.
///
/// Out-of-bounds is **water**, which is what makes the map edge behave like
/// open sea rather than like a wall of land: a sea mark near the edge does not
/// find a spurious shore there.
#[inline]
pub fn is_water(field: &[f32], gw: usize, gh: usize, sea: f64, x: i64, y: i64) -> bool {
    if x < 0 || y < 0 || x >= gw as i64 || y >= gh as i64 {
        return true;
    }
    let Some(&h) = field.get(y as usize * gw + x as usize) else {
        return true;
    };
    // `!(h > sea)`, spelled as the positive test: NaN falls to `true` either
    // way here, and this is the direction the rest of the workspace writes.
    !(h as f64 > sea)
}

/// Whether `(x, y)` is a **coast cell**: water, with a land 4-neighbour.
///
/// The predicate the ruling's snap test asserts. A sea mark that does not
/// satisfy this has not landed on a coast.
pub fn is_coast(field: &[f32], gw: usize, gh: usize, sea: f64, x: i64, y: i64) -> bool {
    if !is_water(field, gw, gh, sea, x, y) {
        return false;
    }
    [(1i64, 0i64), (-1, 0), (0, 1), (0, -1)]
        .into_iter()
        .any(|(dx, dy)| !is_water(field, gw, gh, sea, x + dx, y + dy))
}

/// The nearest coast cell to `(x, y)` within `max_r` cells, or `None`.
///
/// Searched as expanding square rings, nearest ring first, and **within a ring
/// the closest cell by true squared distance wins** — a ring is a Chebyshev
/// shell, so its corners are further away than its edges and taking the first
/// hit in scan order would pull a mark diagonally when a nearer cell sat
/// straight ahead. Ties break on the lowest `(y, x)`, so the answer does not
/// depend on iteration order and a golden test can pin it.
///
/// `r = 0` is checked first, so a mark already on a coast cell does not move —
/// the snap is idempotent, which matters because the placement pass may run
/// twice over the same world.
pub fn snap_to_coast(
    field: &[f32],
    gw: usize,
    gh: usize,
    sea: f64,
    x: i64,
    y: i64,
    max_r: i64,
) -> Option<(i64, i64)> {
    if gw == 0 || gh == 0 {
        return None;
    }
    for r in 0..=max_r.max(0) {
        let mut best: Option<((i64, i64), i64)> = None;
        let mut consider = |cx: i64, cy: i64| {
            if cx < 0 || cy < 0 || cx >= gw as i64 || cy >= gh as i64 {
                return;
            }
            if !is_coast(field, gw, gh, sea, cx, cy) {
                return;
            }
            let d2 = (cx - x) * (cx - x) + (cy - y) * (cy - y);
            // `<` not `<=`: first-found wins a tie, and the scan below runs in
            // (y, x) order, so the tie-break is the documented one.
            if best.is_none_or(|(_, bd)| d2 < bd) {
                best = Some(((cx, cy), d2));
            }
        };
        if r == 0 {
            consider(x, y);
        } else {
            // Top and bottom edges of the ring, then the two sides without
            // their corners — every cell exactly once, in (y, x) order.
            for cx in (x - r)..=(x + r) {
                consider(cx, y - r);
            }
            for cy in (y - r + 1)..=(y + r - 1) {
                consider(x - r, cy);
                consider(x + r, cy);
            }
            for cx in (x - r)..=(x + r) {
                consider(cx, y + r);
            }
        }
        if let Some((cell, _)) = best {
            return Some(cell);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A `w`-wide, `h`-tall field: land where `x < land_cols`, water after.
    /// The shore therefore runs down the column `land_cols`, and every cell in
    /// it is a coast cell.
    fn vertical_shore(w: usize, h: usize, land_cols: usize) -> Vec<f32> {
        (0..w * h)
            .map(|i| if i % w < land_cols { 1.0 } else { 0.0 })
            .collect()
    }

    const SEA: f64 = 0.5;

    #[test]
    fn a_coast_cell_is_water_with_a_land_neighbour() {
        let f = vertical_shore(8, 4, 3);
        // Column 3 is the first water column and touches land at column 2.
        assert!(is_coast(&f, 8, 4, SEA, 3, 1));
        // Column 4 is water but has no land neighbour.
        assert!(!is_coast(&f, 8, 4, SEA, 4, 1));
        // Column 2 is land -- land is never a coast cell here, however much
        // water it touches. This is the asymmetry the module doc argues for.
        assert!(!is_coast(&f, 8, 4, SEA, 2, 1));
    }

    #[test]
    fn the_map_edge_is_open_sea_not_a_shore() {
        // All land: the only "land neighbour" a border cell could have is the
        // out-of-bounds side, and that reads as water, so nothing is coast.
        let f = vec![1.0f32; 6 * 6];
        for y in 0..6 {
            for x in 0..6 {
                assert!(!is_coast(&f, 6, 6, SEA, x, y), "({x},{y}) is land, not coast");
            }
        }
        // And the converse: an all-water map has no shore either.
        let w = vec![0.0f32; 6 * 6];
        assert_eq!(snap_to_coast(&w, 6, 6, SEA, 3, 3, 10), None);
    }

    #[test]
    fn snapping_pulls_an_inland_mark_out_to_the_water_side_of_the_shore() {
        let f = vertical_shore(10, 5, 4);
        // Deep inland at x=0: the nearest coast cell is (4, y), straight out.
        let hit = snap_to_coast(&f, 10, 5, SEA, 0, 2, 10).expect("a shore exists");
        assert_eq!(hit, (4, 2));
        assert!(is_coast(&f, 10, 5, SEA, hit.0, hit.1));
    }

    #[test]
    fn snapping_pulls_an_offshore_mark_back_in() {
        let f = vertical_shore(10, 5, 4);
        // Out at sea: the same column, from the other side.
        let hit = snap_to_coast(&f, 10, 5, SEA, 9, 2, 10).expect("a shore exists");
        assert_eq!(hit, (4, 2));
    }

    #[test]
    fn a_mark_already_on_the_coast_does_not_move() {
        let f = vertical_shore(10, 5, 4);
        assert_eq!(snap_to_coast(&f, 10, 5, SEA, 4, 3, 10), Some((4, 3)));
    }

    #[test]
    fn the_radius_is_a_real_limit_not_a_hint() {
        let f = vertical_shore(20, 3, 10);
        // The shore is at x=10; from x=0 that is 10 cells away.
        assert_eq!(snap_to_coast(&f, 20, 3, SEA, 0, 1, 9), None);
        assert_eq!(snap_to_coast(&f, 20, 3, SEA, 0, 1, 10), Some((10, 1)));
    }

    #[test]
    fn within_a_ring_the_nearest_cell_wins_not_the_first_scanned() {
        // One land cell at (5,5) makes its four edge-neighbours coast cells.
        // From (5, 2) the ring at r=2 contains (5,4) -- distance 2 -- and also
        // (4,5)/(6,5)/(5,6)... no: (4,5) is at Chebyshev distance 3. Build the
        // case explicitly instead: search from (3,5), whose r=2 ring holds the
        // coast cell (4,5) at true distance 1... also not in that ring.
        //
        // The real case is a diagonal corner competing with a straight edge.
        // Land fills the column x>=6 and also the single cell (4,3). From
        // (4,5): the r=2 Chebyshev ring holds (6,5)'s neighbour (5,5) at
        // distance... walk it as data rather than prose.
        let (w, h) = (9usize, 9usize);
        let mut f = vec![0.0f32; w * h];
        // A land block on the right half.
        for y in 0..h {
            for x in 6..w {
                f[y * w + x] = 1.0;
            }
        }
        // ...and one isolated land cell up and to the left.
        f[3 * w + 4] = 1.0;
        // From (4, 5): (4,4) is coast (touches the isolated land at (4,3)) and
        // is 1 away; (5,5) is coast (touches the block at (6,5)) and is also 1
        // away. Both sit in the r=1 ring; the (y,x)-order tie-break takes the
        // lower row first, so (4,4) wins -- deterministic, and pinned here so a
        // scan-order change cannot silently move every sea mark on the map.
        assert_eq!(snap_to_coast(&f, w, h, SEA, 4, 5, 6), Some((4, 4)));
        // One row further down, (4,4) is 2 away and (5,6) is 1 away, so the
        // block wins -- the nearest coast, not the nearest in scan order.
        assert_eq!(snap_to_coast(&f, w, h, SEA, 4, 6, 6), Some((5, 6)));
    }

    #[test]
    fn a_non_finite_cell_reads_as_water_and_never_as_a_place_to_stand() {
        // `!(h > sea)` vs `h <= sea` is the same on NaN; what matters is that
        // NaN does not become land and manufacture a shore out of nothing.
        let mut f = vec![0.0f32; 5 * 5];
        f[2 * 5 + 2] = f32::NAN;
        assert!(is_water(&f, 5, 5, SEA, 2, 2));
        for y in 0..5 {
            for x in 0..5 {
                assert!(!is_coast(&f, 5, 5, SEA, x, y));
            }
        }
    }
}
