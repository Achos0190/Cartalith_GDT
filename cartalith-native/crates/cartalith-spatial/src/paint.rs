//! Categorical override painting — `UNIFIED_TOOL_PLAN.md` milestone C's
//! Biome-paint half.
//!
//! A direct port of the reference HTML engine's Cartography paint brush
//! (`reference/Cartalith Gen1 v2.10.html`, `_paintAt`/`getPaintLayer`/
//! `_paintSampleAt`, lines 4754-4795, plus the sparse persistence at
//! ~26230). The reference's own header states the contract exactly:
//!
//! > `paintBiome`/`paintSplat`/`paintTerrain` are lazily-allocated
//! > `Uint8Array(GW*GH)`, like `civTerritory`: **0 = unpainted** (render
//! > falls through to the existing procedural pipeline), else a **1-based
//! > index** into `CART_BIOMES` / `SPLAT_PAINT_SLOTS` / `CART_TERRAINS`
//! > respectively.
//!
//! ## Why this is in `cartalith-spatial` and not a domain crate
//!
//! Milestone A's split was: generic machinery here, pipeline knowledge in
//! `cartalith-engine`, subsystem-domain math in the owning subsystem crate.
//! A hard-edged categorical disc over a `u8` grid, gated by a caller-supplied
//! exclusion mask, is generic machinery with no Cartalith semantics in it at
//! all — this module never learns what a biome is, only that `0` means
//! "unpainted" and that the caller may veto cells. That is the same
//! caller-defined-flags precedent [`crate::QuadTree`] and [`crate::DirtyTracker`]
//! already set, and [`crate::pass`]'s own module doc anticipated this exact
//! type: *"a biome-paint disc, a territory-paint disc, and a 13-feature
//! landform stamp can all implement it"*.
//!
//! The palettes themselves (`CART_BIOMES`, `CART_TERRAINS`) stay where they
//! were ported, in `cartalith-civ`; a caller supplies indices into them.
//!
//! ## Two things reading the reference corrected
//!
//! 1. **There are three paint layers, not one.** `UNIFIED_TOOL_PLAN.md`
//!    describes only `paintBiome`. The reference ships biome, *splat* (asset-
//!    pack ground textures) and *terrain* (`CART_TERRAINS`, the "surface
//!    underfoot" palette) as three peer arrays through one brush, switched by
//!    `_paintLayer`. They differ only in which palette the value indexes, so
//!    one [`PaintStamp`] type serves all three and the caller owns which
//!    array it is applied to.
//! 2. **The read-time merge is not one thing.** See [`PaintLayer::merge_over`].

use std::sync::Arc;

use crate::pass::Stamp;
use crate::Region;

/// One hard-edged circular paint dab — the reference's `_paintAt` (line 4783),
/// re-shaped as a [`Stamp`] so a paint stroke gets milestone A's draft/commit/
/// discard and draft-scoped undo for free.
///
/// **Categorical data has no half-painted state**, which is the reference's
/// own stated reason (verbatim: *"unlike `sculpt()`/`brushHeight` there's no
/// soft falloff here"*) for this being a hard disc rather than the smoothstep
/// coverage every [`crate::pass::Stamp`] in `cartalith-terrain`'s sculpt
/// registry uses. Blending two palette indices would produce a meaningless
/// third index, not a blend.
///
/// **Divergence from the reference, and it is the only one:** `_paintAt`
/// writes the override array *immediately*, with no draft stage — the
/// reference has no pass buffer for paint at all. Routing it through
/// [`crate::PassBuffer`] is this port's addition, per `UNIFIED_TOOL_PLAN.md`'s
/// shared editing model, and it is purely additive: committing a buffer of
/// `PaintStamp`s in stack order produces exactly what the same sequence of
/// `_paintAt` calls would.
#[derive(Debug, Clone, PartialEq)]
pub struct PaintStamp {
    /// Disc centre in grid cells. Signed because a stroke legitimately runs
    /// off the edge of the map; [`PaintStamp::bounds`] clips.
    pub cx: i64,
    pub cy: i64,
    /// Radius in cells (`_paintRadius`, reference default 6). The gate is
    /// `hypot(dx, dy) > R` — **inclusive** at exactly `R`.
    ///
    /// One divergence, in a case the reference cannot reach: `_paintAt`
    /// iterates `for(let dy=-R; dy<=R; dy++)`, so a *fractional* `R` would
    /// step in fractional offsets and index the array at fractional
    /// positions — garbage. It never happens there because `_paintRadius`
    /// comes from an integer-step slider. Here the loop bound is
    /// `radius.floor()` while the distance test keeps the raw `radius`, so a
    /// fractional value degrades sensibly instead. For every integer radius,
    /// which is every radius the reference can produce, the two are
    /// identical.
    pub radius: f64,
    /// The 1-based palette index to write, or `0` to erase (`_paintErase`).
    pub value: u8,
    /// Cells where `mask[i] != 0` are skipped.
    ///
    /// In Cartography this is the water-body classification, and the gate is
    /// **hard, not a toggle**: the reference's own comment is explicit that
    /// `wb[i] !== 0` *"excludes BOTH ocean(1) and lake(2), never a bare
    /// `field[i] < sea` check, which misses above-sea-level lakes"*. Callers
    /// painting Cartography layers must pass it.
    ///
    /// `None` means "no gate". That is **not** a reference behaviour — the
    /// reference always gates. It exists because `UI_SHELL_DESIGN.md`'s tool
    /// options bar shows a *"respect water mask"* switch that the reference
    /// has no equivalent for (milestone B recorded the same mockup-vs-
    /// reference gap for Freehand raise/lower). Leaving the gate optional
    /// makes that switch buildable later without a redesign, flagged as a
    /// **new affordance** rather than parity (`DECISIONS.md` §7d).
    ///
    /// `Arc` so every dab in one stroke shares one classification array.
    pub mask: Option<Arc<[u8]>>,
}

impl PaintStamp {
    /// A gated dab — the Cartography default. `mask` is the water-body
    /// classification (`0` = land).
    pub fn new(cx: i64, cy: i64, radius: f64, value: u8, mask: Arc<[u8]>) -> Self {
        Self { cx, cy, radius, value, mask: Some(mask) }
    }

    /// An ungated dab. See [`PaintStamp::mask`] — this is the new affordance,
    /// not the reference's behaviour.
    pub fn ungated(cx: i64, cy: i64, radius: f64, value: u8) -> Self {
        Self { cx, cy, radius, value, mask: None }
    }

    /// True when this dab erases rather than paints.
    pub fn is_erase(&self) -> bool {
        self.value == 0
    }
}

impl Stamp for PaintStamp {
    type Cell = u8;

    fn bounds(&self, width: usize, height: usize) -> Region {
        if width == 0 || height == 0 || self.radius < 0.0 {
            return Region::new(0, 0, 0, 0);
        }
        // The reference scans the full `-R..=R` box and rejects per cell;
        // the box is what this stamp can touch, so it is also the bounds.
        // `R` is used unrounded here (not `ceil`) precisely because the
        // per-cell test is `hypot > R`: a cell at integer offset `ceil(R)`
        // can never pass it.
        let r = self.radius.floor() as i64;
        let x0 = (self.cx - r).clamp(0, width as i64 - 1);
        let x1 = (self.cx + r).clamp(0, width as i64 - 1);
        let y0 = (self.cy - r).clamp(0, height as i64 - 1);
        let y1 = (self.cy + r).clamp(0, height as i64 - 1);
        // A disc entirely off-grid still clamps to a 1x1 box, so re-check
        // that the unclamped box actually overlapped the grid at all.
        if self.cx + r < 0
            || self.cx - r > width as i64 - 1
            || self.cy + r < 0
            || self.cy - r > height as i64 - 1
        {
            return Region::new(0, 0, 0, 0);
        }
        Region::new(
            x0 as usize,
            y0 as usize,
            (x1 - x0 + 1) as usize,
            (y1 - y0 + 1) as usize,
        )
    }

    fn apply(&self, dst: &mut [u8], width: usize, height: usize) {
        if width == 0 || height == 0 {
            return;
        }
        let r = self.radius.floor() as i64;
        for dy in -r..=r {
            for dx in -r..=r {
                let (x, y) = (self.cx + dx, self.cy + dy);
                if x < 0 || x >= width as i64 || y < 0 || y >= height as i64 {
                    continue;
                }
                // JS `Math.hypot(dx,dy) > R` -- the comparison, and so the
                // exact set of rim cells, is on the raw radius.
                if (dx as f64).hypot(dy as f64) > self.radius {
                    continue;
                }
                let i = y as usize * width + x as usize;
                if let Some(m) = &self.mask
                    && m.get(i).is_some_and(|&v| v != 0)
                {
                    continue;
                }
                dst[i] = self.value;
            }
        }
    }
}

/// One override grid: `0` = unpainted, else a 1-based palette index.
///
/// Lazily allocated, exactly like `getPaintLayer` — an unpainted layer costs
/// nothing, and a resolution change reallocates rather than serving the old
/// grid (the reference's own `arr.length !== GW*GH` guard, which its v0.148
/// comment records as a real bug fix).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PaintLayer {
    cells: Option<Vec<u8>>,
}

impl PaintLayer {
    pub const fn new() -> Self {
        Self { cells: None }
    }

    /// True when nothing has ever been painted (the array is unallocated).
    /// Note this stays `false` after every cell is erased back to `0` —
    /// same as the reference, whose array also survives being erased.
    pub fn is_unallocated(&self) -> bool {
        self.cells.is_none()
    }

    /// True when no cell currently carries an override.
    pub fn is_empty(&self) -> bool {
        self.cells.as_ref().is_none_or(|c| c.iter().all(|&v| v == 0))
    }

    pub fn cells(&self) -> Option<&[u8]> {
        self.cells.as_deref()
    }

    /// `getPaintLayer` (line 4765): the grid, allocated on demand and
    /// reallocated if `len` no longer matches the field.
    pub fn cells_mut(&mut self, len: usize) -> &mut [u8] {
        match &self.cells {
            Some(c) if c.len() == len => {}
            _ => self.cells = Some(vec![0u8; len]),
        }
        self.cells.as_mut().expect("just allocated")
    }

    /// `paintBiome = null` (line 3353): *"hand-painted Cartography overrides
    /// don't survive a terrain rebuild"*.
    ///
    /// **A real open question this port has and the reference did not**, left
    /// unanswered here deliberately: the reference only ever had one
    /// `generate()`, so "cleared on rebuild" was unambiguous. This port now
    /// has *incremental* terrain edits (milestone B), and whether a Sculpt
    /// commit that changes the temperature/moisture inputs under a painted
    /// cell should also clear that cell is a policy decision with no
    /// reference answer. `UNIFIED_TOOL_PLAN.md` flagged it; nothing here
    /// decides it, because the deciding caller (the shell, milestone F) does
    /// not exist yet. Clearing on full regenerate is the reference-faithful
    /// floor and is all this method does.
    pub fn clear(&mut self) {
        self.cells = None;
    }

    /// `_paintSampleAt` (line 4774): nearest-neighbour, clamped.
    ///
    /// Nearest and **not** bilinear on purpose — the reference's own comment:
    /// *"paint layers are categorical indices, so bilinear (`sampleArr`'s
    /// usual behaviour) would blend two unrelated palette entries into a
    /// meaningless third index."*
    pub fn sample_nearest(&self, wx: f64, wy: f64, gw: usize, gh: usize) -> u8 {
        let Some(cells) = &self.cells else { return 0 };
        if gw == 0 || gh == 0 {
            return 0;
        }
        // JS `Math.round` is half-up (toward +inf), not Rust's half-away-
        // from-zero. They differ only at negative halves, which the clamp
        // then swallows -- but match it anyway rather than rely on that.
        let ix = (wx + 0.5).floor().clamp(0.0, gw as f64 - 1.0) as usize;
        let iy = (wy + 0.5).floor().clamp(0.0, gh as f64 - 1.0) as usize;
        cells.get(iy * gw + ix).copied().unwrap_or(0)
    }

    /// Composite this layer over a computed classification, in place:
    /// `if painted[i] != 0 { base[i] = painted[i] }`.
    ///
    /// **This is where reading the reference corrected the plan.**
    /// `UNIFIED_TOOL_PLAN.md` describes one merge — *"the painted layer takes
    /// precedence over the computed classification, cell by cell"* — and
    /// expects an audit of *"every current `classify_biome` call site"*. The
    /// reference actually has **two different merges, at two different
    /// altitudes**, and the per-cell replace this method implements is the
    /// *rarer* of them:
    ///
    /// * **Replace (this method)** happens in exactly one place: the
    ///   Cartalith editor export (line 12435), which copies
    ///   `buildCartBiome()`/`buildCartTerrain()` and overwrites painted
    ///   cells before encoding.
    /// * **A 0.60 alpha tint** is what the renderer does (`landColorCore`
    ///   lines 7898-7900): the painted index's palette colour is blended over
    ///   the *fully shaded* procedural colour at weight `0.60`, deliberately
    ///   *"not a rewrite of the `materialWeights` mix ... so hillshade/AO/
    ///   crest/splat/haze still show through and painted cells don't read as
    ///   flat pasted stickers"*. That belongs to the renderer, not here.
    ///
    /// And the audit's real answer is that **no analysis consumer merges at
    /// all**: `buildEcoregions`, and every Journey Planner
    /// `currentCartBiome()` reader, take the unpainted classifier output.
    /// Painted overrides are presentation and export in the reference, never
    /// an input to simulation — so wiring them into `classify_biome`'s
    /// callers, as the plan's phrasing invites, would have changed behaviour
    /// the reference does not have.
    pub fn merge_over(&self, base: &mut [u8]) {
        let Some(cells) = &self.cells else { return };
        for (b, &p) in base.iter_mut().zip(cells.iter()) {
            if p != 0 {
                *b = p;
            }
        }
    }

    /// `_paintSyncToState`'s `enc` (line 26236): sparse `[index, value, ...]`
    /// pairs, skipping unpainted cells — the shape `state.cartoPaint` stores,
    /// copied from the `civTerritory` persistence pattern.
    pub fn encode_sparse(&self) -> Vec<u32> {
        let mut out = Vec::new();
        let Some(cells) = &self.cells else { return out };
        for (i, &v) in cells.iter().enumerate() {
            if v != 0 {
                out.push(i as u32);
                out.push(v as u32);
            }
        }
        out
    }

    /// `_paintSyncFromState`'s `dec` (line 26240). An empty pair list decodes
    /// to an unallocated layer, and out-of-range indices are dropped — both
    /// verbatim from the reference (`if(!pairs||!pairs.length) return null`,
    /// `if(pairs[k]<a.length)`).
    pub fn decode_sparse(pairs: &[u32], len: usize) -> Self {
        if pairs.is_empty() {
            return Self::new();
        }
        let mut cells = vec![0u8; len];
        for pair in pairs.chunks_exact(2) {
            let (i, v) = (pair[0] as usize, pair[1]);
            if i < len {
                cells[i] = v as u8;
            }
        }
        Self { cells: Some(cells) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DirtyTracker, PassBuffer};

    const W: usize = 16;
    const H: usize = 16;

    fn land() -> Arc<[u8]> {
        vec![0u8; W * H].into()
    }

    /// Rows 7..=9 are water (classification 2 = lake, deliberately not 1 —
    /// the gate is `!= 0`, not `== 1`).
    fn lake_band() -> Arc<[u8]> {
        let mut m = vec![0u8; W * H];
        for y in 7..=9 {
            for x in 0..W {
                m[y * W + x] = 2;
            }
        }
        m.into()
    }

    fn painted(dst: &[u8]) -> usize {
        dst.iter().filter(|&&v| v != 0).count()
    }

    #[test]
    fn a_dab_is_a_hard_disc_with_no_falloff() {
        let mut dst = vec![0u8; W * H];
        PaintStamp::new(8, 8, 3.0, 5, land()).apply(&mut dst, W, H);
        // Every painted cell carries the same index -- no intermediate
        // values exist, which is the whole point of categorical paint.
        assert!(dst.iter().all(|&v| v == 0 || v == 5));
        assert_eq!(dst[8 * W + 8], 5);
        // Exactly at the radius: painted (the test is `> R`, not `>= R`).
        assert_eq!(dst[8 * W + 11], 5);
        // Just past it: not.
        assert_eq!(dst[8 * W + 12], 0);
        // The corner of the bounding box is outside the disc.
        assert_eq!(dst[11 * W + 11], 0);
    }

    #[test]
    fn radius_one_paints_a_plus_not_a_square() {
        let mut dst = vec![0u8; W * H];
        PaintStamp::new(8, 8, 1.0, 3, land()).apply(&mut dst, W, H);
        assert_eq!(painted(&dst), 5);
        assert_eq!(dst[7 * W + 7], 0, "diagonal is hypot(1,1)=1.41 > 1");
    }

    #[test]
    fn the_mask_gate_excludes_lakes_not_just_ocean() {
        let mut dst = vec![0u8; W * H];
        PaintStamp::new(8, 8, 4.0, 5, lake_band()).apply(&mut dst, W, H);
        for y in 7..=9 {
            for x in 0..W {
                assert_eq!(dst[y * W + x], 0, "painted over water at ({x},{y})");
            }
        }
        assert!(painted(&dst) > 0, "everything outside the band still painted");
    }

    #[test]
    fn an_ungated_dab_paints_where_a_gated_one_would_not() {
        // The new affordance, kept honestly separate from the port.
        let mut gated = vec![0u8; W * H];
        PaintStamp::new(8, 8, 4.0, 5, lake_band()).apply(&mut gated, W, H);
        let mut free = vec![0u8; W * H];
        PaintStamp::ungated(8, 8, 4.0, 5).apply(&mut free, W, H);
        assert!(painted(&free) > painted(&gated));
    }

    #[test]
    fn erase_writes_zero_over_an_existing_index() {
        let mut dst = vec![0u8; W * H];
        PaintStamp::new(8, 8, 4.0, 5, land()).apply(&mut dst, W, H);
        let before = painted(&dst);
        let eraser = PaintStamp::new(8, 8, 2.0, 0, land());
        assert!(eraser.is_erase());
        eraser.apply(&mut dst, W, H);
        assert_eq!(dst[8 * W + 8], 0);
        assert!(painted(&dst) < before);
        assert_eq!(dst[8 * W + 12], 5, "outside the eraser, untouched");
    }

    #[test]
    fn a_later_dab_overwrites_an_earlier_one() {
        // Categorical: last write wins outright, it does not accumulate.
        let mut dst = vec![0u8; W * H];
        PaintStamp::new(8, 8, 4.0, 5, land()).apply(&mut dst, W, H);
        PaintStamp::new(8, 8, 2.0, 9, land()).apply(&mut dst, W, H);
        assert_eq!(dst[8 * W + 8], 9);
    }

    #[test]
    fn bounds_clip_to_the_grid_and_go_empty_when_fully_outside() {
        let s = PaintStamp::new(1, 1, 6.0, 2, land());
        assert_eq!(s.bounds(W, H), Region::new(0, 0, 8, 8));
        let off = PaintStamp::new(-40, -40, 3.0, 2, land());
        let b = off.bounds(W, H);
        assert!(b.w == 0 && b.h == 0);
    }

    #[test]
    fn a_dab_never_writes_outside_its_own_bounds() {
        // The contract PassBuffer's tile marking depends on.
        for (cx, cy) in [(0i64, 0i64), (8, 8), (15, 15), (-2, 8), (8, 20)] {
            let s = PaintStamp::new(cx, cy, 5.0, 7, land());
            let b = s.bounds(W, H);
            let mut dst = vec![0u8; W * H];
            s.apply(&mut dst, W, H);
            for y in 0..H {
                for x in 0..W {
                    if dst[y * W + x] != 0 {
                        assert!(
                            x >= b.x && x < b.x + b.w && y >= b.y && y < b.y + b.h,
                            "({cx},{cy}) wrote ({x},{y}) outside bounds {b:?}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn a_paint_stroke_runs_through_the_pass_buffer() {
        // The whole reason PaintStamp is a Stamp: draft, preview, discard,
        // commit and undo come from milestone A unchanged.
        let mut buf: PassBuffer<PaintStamp> = PassBuffer::new(W, H, 8);
        buf.push(PaintStamp::new(4, 4, 3.0, 5, land()));
        buf.push(PaintStamp::new(12, 12, 3.0, 6, land()));

        let base = vec![0u8; W * H];
        let mut scratch = vec![0u8; W * H];
        buf.preview_into(&base, &mut scratch);
        assert_eq!(scratch[4 * W + 4], 5);
        assert_eq!(scratch[12 * W + 12], 6);
        assert_eq!(base, vec![0u8; W * H], "preview never mutates");

        let mut layer = PaintLayer::new();
        let mut tracker = DirtyTracker::new(buf.tile_count());
        let summary = buf.commit(layer.cells_mut(W * H), &mut tracker, "biome_painted");
        assert_eq!(summary.stamps_applied, 2);
        assert_eq!(summary.tiles_marked, vec![0, 3], "opposite corner tiles");
        assert_eq!(layer.cells().unwrap()[4 * W + 4], 5);
    }

    #[test]
    fn discarding_a_paint_draft_leaves_the_layer_untouched() {
        let mut layer = PaintLayer::new();
        layer.cells_mut(W * H)[0] = 3;
        let before = layer.clone();
        let mut buf: PassBuffer<PaintStamp> = PassBuffer::new(W, H, 8);
        buf.push(PaintStamp::new(4, 4, 3.0, 5, land()));
        buf.discard();
        assert_eq!(layer, before);
    }

    // ---- the layer itself ----

    #[test]
    fn a_fresh_layer_allocates_nothing() {
        let l = PaintLayer::new();
        assert!(l.is_unallocated());
        assert!(l.is_empty());
        assert_eq!(l.cells(), None);
        assert_eq!(l.sample_nearest(4.0, 4.0, W, H), 0);
        assert!(l.encode_sparse().is_empty());
    }

    #[test]
    fn cells_mut_reallocates_when_the_resolution_changes() {
        // The reference's own v0.148 length guard: a resolution change must
        // never serve the old grid.
        let mut l = PaintLayer::new();
        l.cells_mut(W * H)[5] = 7;
        assert_eq!(l.cells().unwrap().len(), W * H);
        let bigger = l.cells_mut(W * H * 4);
        assert_eq!(bigger.len(), W * H * 4);
        assert!(bigger.iter().all(|&v| v == 0), "stale data must not survive");
    }

    #[test]
    fn cells_mut_keeps_the_grid_when_the_size_is_unchanged() {
        let mut l = PaintLayer::new();
        l.cells_mut(W * H)[5] = 7;
        assert_eq!(l.cells_mut(W * H)[5], 7);
    }

    #[test]
    fn merge_over_replaces_only_painted_cells() {
        let mut l = PaintLayer::new();
        {
            let c = l.cells_mut(4);
            c[1] = 9;
            c[3] = 2;
        }
        let mut base = vec![5u8, 5, 5, 5];
        l.merge_over(&mut base);
        assert_eq!(base, vec![5, 9, 5, 2]);
    }

    #[test]
    fn merge_over_an_unallocated_layer_is_a_no_op() {
        let mut base = vec![5u8, 5, 5, 5];
        PaintLayer::new().merge_over(&mut base);
        assert_eq!(base, vec![5, 5, 5, 5]);
    }

    #[test]
    fn sample_nearest_rounds_and_clamps() {
        let mut l = PaintLayer::new();
        {
            let c = l.cells_mut(W * H);
            c[4 * W + 4] = 8;
            c[0] = 3;
        }
        assert_eq!(l.sample_nearest(4.0, 4.0, W, H), 8);
        assert_eq!(l.sample_nearest(3.6, 4.4, W, H), 8, "rounds to (4,4)");
        assert_eq!(l.sample_nearest(-99.0, -99.0, W, H), 3, "clamped to (0,0)");
        assert_eq!(l.sample_nearest(1e6, 1e6, W, H), 0, "clamped to the far corner");
    }

    #[test]
    fn sparse_encoding_round_trips() {
        let mut l = PaintLayer::new();
        {
            let c = l.cells_mut(W * H);
            c[0] = 1;
            c[100] = 13;
            c[W * H - 1] = 7;
        }
        let pairs = l.encode_sparse();
        assert_eq!(pairs.len(), 6, "three painted cells -> three pairs");
        let back = PaintLayer::decode_sparse(&pairs, W * H);
        assert_eq!(back, l);
    }

    #[test]
    fn decoding_drops_indices_past_the_grid() {
        // Verbatim from the reference's `if(pairs[k] < a.length)` -- loading
        // a save made at a higher resolution must not panic.
        let l = PaintLayer::decode_sparse(&[2, 5, 9_999, 6], 4);
        assert_eq!(l.cells().unwrap(), &[0, 0, 5, 0]);
    }

    #[test]
    fn decoding_an_empty_pair_list_yields_an_unallocated_layer() {
        assert!(PaintLayer::decode_sparse(&[], 64).is_unallocated());
    }

    #[test]
    fn clear_drops_the_grid_entirely() {
        let mut l = PaintLayer::new();
        l.cells_mut(W * H)[0] = 4;
        l.clear();
        assert!(l.is_unallocated());
    }

    #[test]
    fn an_erased_layer_is_empty_but_still_allocated() {
        let mut l = PaintLayer::new();
        l.cells_mut(W * H)[0] = 4;
        l.cells_mut(W * H)[0] = 0;
        assert!(l.is_empty());
        assert!(!l.is_unallocated(), "the reference's array survives erasing too");
    }
}
