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

/// One circular paint dab — the reference's `_paintAt` (line 4783), re-shaped
/// as a [`Stamp`] so a paint stroke gets milestone A's draft/commit/discard
/// and draft-scoped undo for free.
///
/// **Categorical data has no half-painted state**, which is the reference's
/// own stated reason (verbatim: *"unlike `sculpt()`/`brushHeight` there's no
/// soft falloff here"*) for shipping a hard disc rather than the smoothstep
/// coverage every [`crate::pass::Stamp`] in `cartalith-terrain`'s sculpt
/// registry uses. **That reasoning is still correct, and this port has not
/// touched it**: no palette index is ever blended with another; every
/// painted cell still carries exactly one clean index, always. What this
/// port adds (`DECISIONS.md` §7k, owner ruling 2026-08-31,
/// `LARGE_ITEM_RULINGS.md` — the highest-severity row in
/// `UNWIRED_FUNCTIONS.md`, bound rather than deleted) is falloff at the
/// disc's own *edge* — which cells get touched at all — never at its
/// *interior* — what value a touched cell receives. See
/// [`PaintStamp::with_falloff`] for the mechanism.
///
/// **Two divergences from the reference, both disclosed, and this is the
/// complete list:**
///
/// 1. `_paintAt` writes the override array *immediately*, with no draft
///    stage — the reference has no pass buffer for paint at all. Routing it
///    through [`crate::PassBuffer`] is this port's addition, per
///    `UNIFIED_TOOL_PLAN.md`'s shared editing model, and it is purely
///    additive: committing a buffer of `PaintStamp`s in stack order
///    produces exactly what the same sequence of `_paintAt` calls would.
/// 2. The edge falloff itself. The reference has no falloff of any kind for
///    this brush, hard or soft. A stamp built through [`PaintStamp::new`]/
///    [`PaintStamp::ungated`] and never handed to [`PaintStamp::
///    with_falloff`] is the historical hard disc, bit-for-bit — this is a
///    strict superset of the old behaviour, not a replacement for it, and
///    every pre-existing golden/regression test for the hard-disc case
///    (this crate's own, `cartalith-civ`'s territory brush, which does not
///    use falloff at all, and `cartalith-godot`'s) keeps passing unchanged.
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
    ///
    /// The distance test itself uses [`js_hypot`], not `f64::hypot` — see
    /// that function for the measured rim-cell divergence this fixes.
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

    /// Falloff inputs for [`PaintStamp::apply`]'s own edge band,
    /// `0.0..=1.0` each — the `DCC_SHELL_SPEC.md` §4.5.2 `Hardness`/
    /// `Softness` sliders, verbatim and uncombined. **Not reference
    /// values** — there is nothing in the reference to port here; see this
    /// type's own doc. Set through [`PaintStamp::with_falloff`], never
    /// directly: [`PaintStamp::new`]/[`PaintStamp::ungated`] default both
    /// to the pair that keeps this a hard disc (`hardness: 1.0, softness:
    /// 0.0`), so every existing caller — including `cartalith-civ`'s
    /// territory brush, which this falloff was never meant to reach — is
    /// unaffected by these fields' existence unless it explicitly opts in.
    ///
    /// The two combine into one *softening* amount at read time
    /// (`PaintStamp::feather_width`): `((1.0 - hardness) +
    /// softness).clamp(0.0, 1.0)`. Moving either slider away from "fully
    /// hard" (`hardness = 1`, `softness = 0`) softens the edge a little —
    /// both push the same needle the same way, the plain-English meaning of
    /// both words, rather than one being forced into the other's exact
    /// inverse.
    pub hardness: f64,
    pub softness: f64,
}

impl PaintStamp {
    /// A gated dab — the Cartography default. `mask` is the water-body
    /// classification (`0` = land). Constructs with `hardness: 1.0,
    /// softness: 0.0` — the historical hard disc; see [`PaintStamp::
    /// with_falloff`] to opt into a soft edge.
    pub fn new(cx: i64, cy: i64, radius: f64, value: u8, mask: Arc<[u8]>) -> Self {
        Self { cx, cy, radius, value, mask: Some(mask), hardness: 1.0, softness: 0.0 }
    }

    /// An ungated dab. See [`PaintStamp::mask`] — this is the new affordance,
    /// not the reference's behaviour. Constructs with `hardness: 1.0,
    /// softness: 0.0`, same as [`PaintStamp::new`].
    pub fn ungated(cx: i64, cy: i64, radius: f64, value: u8) -> Self {
        Self { cx, cy, radius, value, mask: None, hardness: 1.0, softness: 0.0 }
    }

    /// True when this dab erases rather than paints.
    pub fn is_erase(&self) -> bool {
        self.value == 0
    }

    /// Opts this dab into a soft-edged falloff band at the outer rim of the
    /// disc — `DECISIONS.md` §7k, bound by the owner 2026-08-31
    /// (`LARGE_ITEM_RULINGS.md`) as a deliberate divergence from the
    /// reference, which has no falloff for this brush at all.
    ///
    /// `hardness`/`softness` are the two `DCC_SHELL_SPEC.md` §4.5.2 sliders,
    /// verbatim — not pre-combined by the caller. See [`PaintStamp::
    /// feather_width`] for how they turn into a band width, and this type's
    /// own doc for why never calling this method (or calling it with
    /// `hardness: 1.0, softness: 0.0`, [`PaintStamp::new`]/[`PaintStamp::
    /// ungated`]'s own construction default) is a documented, tested no-op:
    /// the historical hard disc, bit-for-bit.
    pub fn with_falloff(mut self, hardness: f64, softness: f64) -> Self {
        self.hardness = hardness;
        self.softness = softness;
        self
    }

    /// Width, in cells, of the probabilistic falloff band at the outer edge
    /// of the disc — `0.0` (the exact float, not merely small) means no
    /// band at all, and every cell inside `radius` paints unconditionally.
    ///
    /// [`PaintStamp::new`]/[`PaintStamp::ungated`] construct with
    /// `hardness: 1.0, softness: 0.0`: `(1.0 - 1.0) + 0.0` is `0.0` with no
    /// rounding (IEEE 754 subtraction of two equal finite operands is
    /// exact), so `softening` below is exactly `0.0`, and `0.0 * radius` is
    /// exactly `0.0` for any finite `radius` — every dab that never calls
    /// `with_falloff` gets a literal, not merely approximate, zero-width
    /// band, and [`PaintStamp::apply`] skips the falloff branch for it
    /// entirely rather than evaluating a probability that always comes out
    /// to 1.
    fn feather_width(&self) -> f64 {
        let softening = ((1.0 - self.hardness) + self.softness).clamp(0.0, 1.0);
        softening * self.radius
    }
}

/// Whether the cell at absolute grid position `(x, y)`, measured `dist`
/// cells from the stamp's own centre by the same [`js_hypot`] the disc's
/// own boundary test uses, survives a falloff band `width` cells wide at
/// the outer edge of a disc of the given `radius`.
///
/// The interior — everything closer than `radius - width` — always passes,
/// which is what keeps the disc's centre solid instead of fading it
/// uniformly; only the band itself is probabilistic. Inside the band the
/// pass probability ramps linearly from 1 (at the band's own inner edge) to
/// 0 (at `radius`, the disc's existing hard boundary), decided against
/// [`cell_dither`]: a deterministic hash of the cell's own absolute
/// position, not a per-frame random draw, so repainting the same spot with
/// the same brush settings keeps (or drops) exactly the same cells every
/// time. The brush stipples the map; it does not flicker.
fn passes_falloff(x: i64, y: i64, dist: f64, radius: f64, width: f64) -> bool {
    let inner = radius - width;
    if dist <= inner {
        return true;
    }
    let t = (radius - dist) / width;
    cell_dither(x, y) < t
}

/// A fast, deterministic `[0, 1)` value from a cell's own absolute grid
/// position — the mottled edge's own source of "randomness".
///
/// **Not a port.** No reference brush falloff exists to match (this whole
/// mechanism is new engine work, `DECISIONS.md` §7k), so there is no JS
/// behaviour this needs to reproduce and none of `cartalith-rust-
/// conventions`' precision-matching rules govern it — only a good, stable
/// avalanche is required. This is MurmurHash3's 64-bit finalizer
/// (`fmix64`, public domain): small, well known, and already avalanches
/// well from one multiply-xor-shift-multiply-xor-shift-multiply-xor-shift
/// pass. `cartalith_noise::hash` next door does the equivalent job for
/// terrain noise, but pulling it into this crate would add a
/// `cartalith-spatial` → `cartalith-noise` dependency edge for one
/// comparison, and this crate's own module doc is explicit about staying
/// free of dependencies it does not need.
///
/// Salted with a fixed odd constant before the mix so the origin cell
/// `(0, 0)` — whose raw key is `0`, and `fmix64(0) == 0` — does not become
/// the one grid cell that always deterministically passes every falloff
/// test; every other cell already avalanches away from its own key without
/// help.
fn cell_dither(x: i64, y: i64) -> f64 {
    let key = ((x as u32 as u64) << 32) | (y as u32 as u64);
    let mut h = key ^ 0x9E37_79B9_7F4A_7C15;
    h ^= h >> 33;
    h = h.wrapping_mul(0xff51_afd7_ed55_8ccd);
    h ^= h >> 33;
    h = h.wrapping_mul(0xc4ce_b9fe_1a85_ec53);
    h ^= h >> 33;
    (h >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
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
        let feather = self.feather_width();
        for dy in -r..=r {
            for dx in -r..=r {
                let (x, y) = (self.cx + dx, self.cy + dy);
                if x < 0 || x >= width as i64 || y < 0 || y >= height as i64 {
                    continue;
                }
                // JS `Math.hypot(dx,dy) > R` -- the comparison, and so the
                // exact set of rim cells, is on the raw radius, and on V8's
                // `Math.hypot` rather than Rust's (see `js_hypot`).
                let dist = js_hypot(dx as f64, dy as f64);
                if dist > self.radius {
                    continue;
                }
                // `feather == 0.0` -- the exact float, not merely small --
                // is the construction default, and skips this branch
                // (including the `passes_falloff` call, so no hash is ever
                // computed) entirely: see `feather_width`'s own doc for why
                // that keeps this loop bit-for-bit identical to the
                // pre-falloff code for every caller that never opted in.
                if feather > 0.0 && !passes_falloff(x, y, dist, self.radius, feather) {
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

// V8's `Math.hypot`, from `cartalith-jsmath`. `_paintAt`'s brush gate is
// `if(Math.hypot(dx,dy)>R) continue`, and `JS_SEMANTICS_AUDIT.md` §2.1 found
// that `f64::hypot` skips a different set of rim cells: the two disagree on
// 1 398 of the 4 096 integer offsets in `[0,64)^2`, and an exhaustive scan of
// every integer radius `1..=512` finds 25 radii where a painted cell actually
// changes, the first at `R = 125` on the Pythagorean triple `(35, 120)`. The
// exhaustive tests below are this module's own and stay here.
use cartalith_jsmath::js_hypot;

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

    /// The rim-cell divergence [`js_hypot`] exists for, at the smallest radius
    /// that can reach it.
    ///
    /// `(35, 120, 125)` is a Pythagorean triple, so the true distance from the
    /// brush centre to those cells is *exactly* the radius. V8's `Math.hypot`
    /// is one ulp high there — `125.00000000000001421` — so `_paintAt`'s
    /// `if(Math.hypot(dx,dy)>R) continue` **skips** them. Rust's `f64::hypot`
    /// is correctly rounded, returns exactly `125.0`, and paints them. Eight
    /// cells per stamp, on the exact rim the module doc claims to reproduce.
    ///
    /// Verified against V8 directly (`node -e "Math.hypot(35,120)"`), not
    /// against this port's own output.
    ///
    /// No pre-existing fixture could have caught this: every `PaintStamp` test
    /// in this crate, and `cartalith-civ`'s territory-brush golden, uses a
    /// radius of 6 or less, and an exhaustive scan of `1..=512` shows the
    /// first radius where the two algorithms disagree *about a cell* is 125.
    #[test]
    fn rim_cells_on_a_pythagorean_triple_follow_v8_not_rust_hypot() {
        const G: usize = 251; // 2*125 + 1
        const C: i64 = 125;
        let mut dst = vec![0u8; G * G];
        PaintStamp::ungated(C, C, 125.0, 7).apply(&mut dst, G, G);

        let at = |dx: i64, dy: i64| dst[((C + dy) as usize) * G + (C + dx) as usize];
        for (dx, dy) in [(35, 120), (120, 35)] {
            for (sx, sy) in [(1, 1), (1, -1), (-1, 1), (-1, -1)] {
                assert_eq!(
                    at(dx * sx, dy * sy),
                    0,
                    "cell ({}, {}) is at distance exactly 125; V8's Math.hypot                      reports 125.00000000000001421, so the reference skips it",
                    dx * sx,
                    dy * sy
                );
            }
        }
        // ... and the cells just inside it are still painted, so the test is
        // not passing because the whole stamp went missing.
        assert_eq!(at(35, 119), 7);
        assert_eq!(at(34, 120), 7);
        assert_eq!(at(0, 125), 7); // exactly on the rim, but not a triple
    }

    /// The other half of the same claim: for every radius the reference's own
    /// sliders can produce (`_paintRadius` 1..=40, `_civTerRadius` 1..=20),
    /// [`js_hypot`] and `f64::hypot` choose the *same* cells — so this change
    /// cannot have moved any existing golden.
    #[test]
    fn below_radius_125_the_two_hypots_agree_on_every_cell() {
        for r in 1..=124i64 {
            for dx in -r..=r {
                for dy in -r..=r {
                    let (a, b) = (dx as f64, dy as f64);
                    assert_eq!(
                        js_hypot(a, b) > r as f64,
                        a.hypot(b) > r as f64,
                        "r={r} dx={dx} dy={dy}"
                    );
                }
            }
        }
    }

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

    // ---- with_falloff (`DECISIONS.md` §7k) ----

    #[test]
    fn with_falloff_at_the_construction_defaults_is_bit_identical_to_the_hard_disc() {
        // The exact claim `DECISIONS.md` §7k rests on: `hardness=1.0,
        // softness=0.0` -- both the type's own construction default AND an
        // explicit `with_falloff` call at those values -- must reproduce
        // the untouched hard-disc output, cell for cell.
        let mut baseline = vec![0u8; W * H];
        PaintStamp::new(8, 8, 6.0, 5, land()).apply(&mut baseline, W, H);

        let mut explicit_default = vec![0u8; W * H];
        PaintStamp::new(8, 8, 6.0, 5, land())
            .with_falloff(1.0, 0.0)
            .apply(&mut explicit_default, W, H);

        assert_eq!(baseline, explicit_default);
        // A second radius, away from the 6.0 this test already uses and the
        // 3.0 `a_dab_is_a_hard_disc_with_no_falloff` uses above -- the
        // invariant is `feather_width() == 0.0`, not "holds at one radius".
        let mut baseline2 = vec![0u8; W * H];
        PaintStamp::new(8, 8, 4.0, 5, land()).apply(&mut baseline2, W, H);
        let mut explicit2 = vec![0u8; W * H];
        PaintStamp::new(8, 8, 4.0, 5, land()).with_falloff(1.0, 0.0).apply(&mut explicit2, W, H);
        assert_eq!(baseline2, explicit2);
    }

    #[test]
    fn a_softer_edge_keeps_the_interior_solid_but_mottles_the_rim() {
        // Radius 20 in a grid big enough to hold it -- the small radii
        // (3..6) the rest of this file uses don't leave enough rim cells
        // for "mottled, not merely smaller" to be a meaningful measurement.
        const G: usize = 48;
        const C: i64 = 24;
        const R: f64 = 20.0;
        let mut hard = vec![0u8; G * G];
        PaintStamp::ungated(C, C, R, 7).apply(&mut hard, G, G);

        let mut soft = vec![0u8; G * G];
        // hardness=0.4 -> softening = 0.6 -> a 12-cell-wide band (inner
        // edge at distance 8, disc's own edge at 20).
        PaintStamp::ungated(C, C, R, 7).with_falloff(0.4, 0.0).apply(&mut soft, G, G);

        // The deep interior -- inside distance ~7.07, well short of the
        // band's own inner edge at 8 -- is untouched by the falloff: every
        // cell the hard disc painted there, the soft one does too.
        for dy in -5i64..=5 {
            for dx in -5i64..=5 {
                let i = ((C + dy) as usize) * G + (C + dx) as usize;
                assert_eq!(soft[i], hard[i], "interior cell ({dx},{dy}) must be unaffected by falloff");
            }
        }

        // The band itself (distance in (8, 20]) is sampled over the full
        // annulus -- hundreds of cells, not one ray -- so "some painted,
        // some not" is a real structural check, not a coin flip this test
        // could get unlucky on.
        let mut band_painted = 0;
        let mut band_total = 0;
        for dy in -20i64..=20 {
            for dx in -20i64..=20 {
                let dist = js_hypot(dx as f64, dy as f64);
                if dist > 8.0 && dist <= R {
                    band_total += 1;
                    let i = ((C + dy) as usize) * G + (C + dx) as usize;
                    if soft[i] != 0 {
                        band_painted += 1;
                    }
                }
            }
        }
        assert!(band_total > 100, "sanity: the annulus should be a few hundred cells, got {band_total}");
        assert!(band_painted > 0, "some of the band must still be painted");
        assert!(band_painted < band_total, "and some of it must not -- a mottled edge, not a shrunk disc");

        // Falloff only ever *removes* cells the hard disc painted; it never
        // extends the disc past its own radius.
        for i in 0..G * G {
            if soft[i] != 0 {
                assert_ne!(hard[i], 0, "cell {i} painted soft but not hard -- falloff must not extend the disc");
            }
        }
    }

    #[test]
    fn softness_alone_softens_the_edge_even_at_full_hardness() {
        // hardness stays at its own "fully hard" default; softness alone
        // must still be able to open a falloff band -- the two sliders add,
        // rather than softness being hardness-in-disguise.
        const G: usize = 48;
        const C: i64 = 24;
        const R: f64 = 20.0;
        let mut hard = vec![0u8; G * G];
        PaintStamp::ungated(C, C, R, 7).apply(&mut hard, G, G);
        let mut soft = vec![0u8; G * G];
        PaintStamp::ungated(C, C, R, 7).with_falloff(1.0, 0.5).apply(&mut soft, G, G);
        assert_ne!(hard, soft, "softness=0.5 at hardness=1.0 must still feather the edge");
    }

    #[test]
    fn falloff_is_deterministic_across_repeated_applications() {
        // "a deterministic, position-seeded threshold so repeated passes
        // are stable" -- the property a per-frame random draw would not
        // have. Applying the same stamp twice (independent scratch buffers,
        // same stamp value reused) must paint exactly the same cells both
        // times.
        const G: usize = 48;
        let stamp = PaintStamp::ungated(24, 24, 20.0, 3).with_falloff(0.4, 0.0);
        let mut a = vec![0u8; G * G];
        let mut b = vec![0u8; G * G];
        stamp.apply(&mut a, G, G);
        stamp.apply(&mut b, G, G);
        assert_eq!(a, b, "the same stamp applied twice must paint exactly the same cells");
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
