//! **EF-9.4's continuous effective level** -- importance-driven refinement as a
//! per-position level, not a per-level tile selection
//! (`ELEVATION_FIELD_ARCHITECTURE_RESEARCH.md` §8.5).
//!
//! # Why this exists
//!
//! EF-9.4's first display model (2026-10-17/18) promoted whole `z0` tiles to
//! their `z0 + 1` children, re-selected at every level. Measured on seed 24601,
//! 512x384, zoom-while-pan (dumped frames 19 -> 20, §8.4): a region promoted at
//! level `z` shows `z + 1` by the end of the level, and one frame later, at
//! `z + 1`, its children drop from alpha 0.998 to 0.00002 because the selection
//! at the new level is a different set. A promotion decided separately at each
//! level makes the drawn level jump by one wherever the decision changes, and at
//! a level boundary the decision is re-taken everywhere at once. No edge or
//! colour rule fixes that; the D3 bar (level-boundary ratio <= 1.5) failed on 8
//! of 12 runs with it.
//!
//! # The model
//!
//! Strugar's CDLOD (*Continuous Distance-Dependent Level of Detail for Rendering
//! Heightmaps*, JGGT 14(4) 2009, cited in `docs/research/lod extra info.md` §3.3
//! and `TERRAIN_ARCHITECTURE_RESEARCH.md`) chooses the level of every vertex
//! from a continuous function of position (distance to the camera) and morphs
//! over the last part of each level's range, so two neighbouring nodes of
//! different levels agree where they meet. This module does the same with
//! **importance in place of distance**, in level units:
//!
//! ```text
//! lam_zoom      = log2(max(1, gw * px_per_cell / TILE_PX))      (lod_bridge's level_for_zoom, unrounded)
//! L(x, zoom)    = lam_zoom + gain(zoom) * BOOST_LEVELS * b(x)    (effective_level)
//! w_k(x, zoom)  = smoothstep01(L - k + 0.5)                      (level_weight; LOD-D3's morph at b = 0)
//! ```
//!
//! `b(x)` in `[0, 1]` is a **zoom-independent** field over the map
//! ([`BoostField`]): the fitted importance of EF-9.1, scored once per world on a
//! fixed footprint, mapped through a smooth ramp, and interpolated bilinearly.
//! `gain(zoom)` ([`entry_gain`]) is `0` at the LOD entry zoom and `1` one zoom
//! octave past it, so the layer still comes up exactly as it does without EF-9.
//!
//! The viewport draws three layers, back to front: level `z0 - 1` (or the base
//! map in the entry band), level `z0` and level `z0 + 1`, where `z0` is today's
//! drawn level. Each tile's alpha is its own level's `w_k` **per pixel**
//! (`lod_tile.gdshader`). [`layer_weights`] is the resulting composite.
//!
//! # The continuity condition
//!
//! The picture at a pixel is continuous in zoom and in position iff
//!
//! 1. `L` is continuous in `(x, zoom)` -- true because `lam_zoom`, `gain` and
//!    `b` are each continuous and nothing here depends on the drawn level;
//! 2. the composite is a function of `L` alone, independent of which `z0` is
//!    drawn, wherever two choices of `z0` are both valid -- proved below; and
//! 3. every tile with a non-zero weight anywhere in view is live (a missing tile
//!    is a hole in the sense LOD-D3 already measures; [`BoostField::tiles_over`]
//!    plus [`PREFETCH_LEVELS`] is how the shell asks for them in time).
//!
//! **Proof of 2.** With `a_k = w_k(L)`, the stack `z0` composites to
//! `C = a1·T(z0+1) + (1-a1)·(a0·T(z0) + (1-a0)·T(z0-1))`. Today's drawn level is
//! `z0 = round_half_up(lam_zoom)`, so `lam_zoom` is in `[z0 - 0.5, z0 + 0.5)`
//! and, with `0 <= gain·BOOST_LEVELS·b <= 1`, `L` is in `[z0 - 0.5, z0 + 1.5)`.
//! On that interval at most one of `a0`, `a1` is strictly between 0 and 1:
//! `L <= z0 + 0.5` gives `a1 = 0`, so `C = mix(T(z0-1), T(z0), a0)`; and
//! `L >= z0 + 0.5` gives `a0 = 1`, so `C = mix(T(z0), T(z0+1), a1)`. Both are
//! `P(L) = mix(T(k-1), T(k), smoothstep01(L - k + 0.5))` with `k = round(L)`,
//! a function of `L` only. At the level boundary `lam_zoom = z0 + 0.5` the
//! shell switches to stack `z0 + 1`; every pixel then has `L >= z0 + 0.5`, so
//! the old stack shows `mix(T(z0), T(z0+1), a1)` and the new one shows the same
//! expression (its bottom layer `T(z0)` is opaque under `T(z0+1)` at weight
//! `w_{z0+1}(L)`, and its `z0 + 2` layer has weight `w_{z0+2}(L) = 0` because
//! `L < z0 + 1.5`). Zooming out is the mirror case. `P` itself is continuous in
//! `L` because `smoothstep01(L - k + 0.5)` is `1` at `L = k + 0.5`, where
//! `P` hands over to `mix(T(k), T(k+1), 0) = T(k)` -- LOD-D3's own invariant
//! (`lod_bridge::morph_for_zoom`'s `the_morph_hands_over_exactly_at_the_level_boundary`).
//! So condition 2 holds, and the weights move by at most
//! `max |smoothstep01'| = 1.5` per level of `L` -- which the tests sweep.
//!
//! **Why `BOOST_LEVELS <= 1` is part of the condition.** With a boost above one
//! level `L` could reach `z0 + 2` and a fourth layer would be needed; the stack
//! above would then cut it off and the composite would no longer be `P(L)`.
//!
//! # What it must never do
//!
//! * Depend on the drawn level `z0` anywhere in `b` or `L` (that is the
//!   discrete model's defect). [`BoostField`] is built once per world from a
//!   fixed footprint.
//! * Change a tile's content (I1), a river threshold (I2), or ask for a tile
//!   deeper than the caller's cap (I3: [`plus_one_threshold`] is `None` when
//!   `z0 + 1 > z_cap`).
//! * Boost a tile with no score: an absent score maps to `b = 0`, which is
//!   today's display there, never a guessed value.

use crate::importance::{importance, tile_features, ImportanceWeights, WorldImportanceField};
use crate::subdivision::{PROMOTE_PERCENT, SCORE_FLOOR};
use cartalith_spatial::pyramid::{pyramid_dims, pyramid_tile_bounds};

// ------------------------------------------------------------------ constants

/// How many pyramid levels the boost may add: **1**. Q1's default (§8.6: promote
/// only, at most `+1`), and the bound the continuity proof needs (a larger
/// boost would need a fourth layer; module header).
pub const BOOST_LEVELS: f64 = 1.0;

/// Coarse cells across one boost texel: **8**. A labelled judgement. The
/// fitted score aggregates slope and land over a footprint (EF-9.1), so the
/// footprint must be fixed for `b` to be zoom-independent; 8 cells is the
/// footprint of a z6 tile at 512 wide and a z8 tile at 2048 wide, inside the
/// z5..z10 range EF-9.0 measured and fitted on. Not swept.
pub const BOOST_FOOTPRINT_CELLS: f64 = 8.0;

/// Half-width, in percentiles, of the ramp that maps a score to `b`: **12.5**.
/// The ramp is centred on EF-9.2's own cut (the top [`PROMOTE_PERCENT`] of
/// scored tiles), so a texel at the 75th percentile of the world's scores gets
/// `b = 0.5`, one below the 62.5th gets 0 and one above the 87.5th gets 1. A
/// labelled judgement: wide enough that `b` is a ramp rather than a mask, narrow
/// enough that the bottom 62.5 % of the world draws exactly as without EF-9.
pub const RAMP_HALF_WIDTH_PERCENT: f64 = 12.5;

/// How far ahead, in levels of `L`, the shell asks for a `z0 + 1` tile before
/// its weight leaves 0: **0.25**. A labelled judgement: at the D0 harness's zoom
/// step of 1.02 per frame (`log2(1.02)` = 0.0286 levels) it is about nine frames
/// of lead, against tile syntheses measured at 7-12 ms each (§8.4, metric 5).
pub const PREFETCH_LEVELS: f64 = 0.25;

// ------------------------------------------------------------- the weights

/// GLSL `smoothstep(0.0, 1.0, x)`: clamp, then `x²(3 − 2x)`. The same curve as
/// `cartalith_godot::lod_bridge`'s private `smoothstep01` (LOD-D3's morph), and
/// `lod_tile.gdshader` evaluates the same expression per pixel.
#[must_use]
pub fn smoothstep01(x: f64) -> f64 {
    let t = x.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// The drawn level for a continuous zoom level: `round_half_up(lam_zoom)`
/// clamped to `[0, z_cap]` -- `cartalith_spatial::pyramid::pyramid_level_for_zoom`'s
/// own rounding (`js_round`), so the shell's `lod_level_for_zoom` and this
/// agree for every `lam_zoom` it can produce.
#[must_use]
pub fn drawn_level(lam_zoom: f64, z_cap: i32) -> i32 {
    let r = cartalith_jsmath::js_round(lam_zoom);
    f64::max(0.0, f64::min(f64::from(z_cap), r)) as i32
}

/// `L(x, zoom)`: the continuous zoom level plus the importance boost. `b` and
/// `gain` are clamped to `[0, 1]`, so `L` never exceeds `lam_zoom + BOOST_LEVELS`
/// -- the bound the continuity proof needs.
#[must_use]
pub fn effective_level(lam_zoom: f64, gain: f64, b: f64) -> f64 {
    lam_zoom + gain.clamp(0.0, 1.0) * BOOST_LEVELS * b.clamp(0.0, 1.0)
}

/// How far level `k` has faded in at effective level `lam`: LOD-D3's morph with
/// `L` in place of the zoom's level (`lod_bridge::morph_for_zoom` is this with
/// `b = 0`).
#[must_use]
pub fn level_weight(lam: f64, k: i32) -> f64 {
    smoothstep01(lam - f64::from(k) + 0.5)
}

/// The boost's fade-in after LOD entry: `smoothstep01(log2(zoom / zoom_entry))`,
/// `0` at the entry zoom and `1` one zoom octave (one pyramid level) past it.
///
/// Why: the layer comes up at `_update_lod()`'s entry gate against the base
/// map, at whatever morph the zoom then has (LOD-D3's entry rule). A boost at
/// full strength there would bring boosted ground up a whole level further in
/// the same frame. Ramping it over one level makes the entry exactly today's
/// and keeps `L` continuous in zoom. `0` for a non-positive or non-finite
/// `zoom_entry` (no entry gate known = no boost, today's display).
#[must_use]
pub fn entry_gain(zoom: f64, zoom_entry: f64) -> f64 {
    if !(zoom_entry > 0.0) || !zoom.is_finite() || zoom <= 0.0 {
        return 0.0;
    }
    smoothstep01((zoom / zoom_entry).log2())
}

/// **The composite** of the three-layer stack the viewport draws at drawn level
/// `z0`, for one pixel at effective level `lam`: `(level, contribution)` for
/// `z0 - 1` (the bottom layer: the parent level, or the base map in the entry
/// band, reported as `z0 - 1` either way), `z0` and, when `z0 + 1 <= z_cap`,
/// `z0 + 1`. Contributions are non-negative and sum to 1.
///
/// This is the shader's arithmetic, stated once so the tests can check the
/// continuity condition on it: `z0 + 1` drawn with alpha `a1` over `z0` drawn
/// with alpha `a0` over an opaque bottom.
#[must_use]
pub fn layer_weights(lam: f64, z0: i32, z_cap: i32) -> Vec<(i32, f64)> {
    let a0 = level_weight(lam, z0);
    let a1 = if z0 < z_cap { level_weight(lam, z0 + 1) } else { 0.0 };
    let mut out = vec![(z0 - 1, (1.0 - a0) * (1.0 - a1)), (z0, a0 * (1.0 - a1))];
    if z0 < z_cap {
        out.push((z0 + 1, a1));
    }
    out
}

/// The boost value above which a `z0 + 1` tile must be live, or `None` when no
/// `z0 + 1` tile is needed: `z0 + 1` past the cap (I3), no gain yet, or a
/// threshold at or above 1 (no `b` reaches it).
///
/// A `z0 + 1` tile has a non-zero weight somewhere iff
/// `gain·BOOST_LEVELS·max(b) > z0 + 0.5 - lam_zoom`; asking [`PREFETCH_LEVELS`]
/// early gives the synthesis time to land before the weight leaves 0. Clamped
/// at `0`, never below: a tile whose `b` is 0 everywhere has weight 0 at this
/// level whatever the zoom (`L = lam_zoom < z0 + 0.5`), so asking for it would
/// be a prefetch of the next level for the whole view, not a refinement.
#[must_use]
pub fn plus_one_threshold(lam_zoom: f64, gain: f64, z0: i32, z_cap: i32) -> Option<f64> {
    if z0 >= z_cap {
        return None;
    }
    let g = gain.clamp(0.0, 1.0) * BOOST_LEVELS;
    if g <= 0.0 {
        return None;
    }
    let t = ((f64::from(z0) + 0.5 - lam_zoom - PREFETCH_LEVELS) / g).max(0.0);
    (t < 1.0).then_some(t)
}

// ---------------------------------------------------------- the boost field

/// The level whose tiles span about [`BOOST_FOOTPRINT_CELLS`] coarse cells
/// across a `gw`-wide world: `round(log2((gw - 1) / 8))`, clamped to `0..=10`
/// (`lod_bridge::MAX_LEVEL`'s value, restated because the engine cannot name the
/// bridge). 512 wide gives 6, 2048 wide gives 8.
#[must_use]
pub fn boost_level(gw: usize) -> u32 {
    if gw < 2 {
        return 0;
    }
    let l = ((gw as f64 - 1.0) / BOOST_FOOTPRINT_CELLS).log2();
    cartalith_jsmath::js_round(l).clamp(0.0, 10.0) as u32
}

/// `b(x)`: one value in `[0, 1]` per tile of [`boost_level`], `n x n` of them,
/// row-major, read as a texture whose texel `(i, j)` sits at the **centre** of
/// that tile in pyramid index space, `((i + 0.5) / n, (j + 0.5) / n)`.
///
/// Index space (`u = col / 2^z` at any level `z`) is used rather than cell space
/// because the shell's tile rects are exactly index-space intervals: a tile
/// `(z, c, r)` covers `[c, c + 1] / 2^z` on each axis, at every level, so one
/// texture serves every level and a parent's footprint is exactly the union of
/// its children's.
#[derive(Clone, Debug, PartialEq)]
pub struct BoostField {
    n: usize,
    values: Vec<f32>,
}

impl BoostField {
    /// Maps one score per texel to `b`: a smoothstep ramp from the
    /// `100 - PROMOTE_PERCENT - RAMP_HALF_WIDTH_PERCENT` to the
    /// `100 - PROMOTE_PERCENT + RAMP_HALF_WIDTH_PERCENT` percentile of the present
    /// scores (nearest rank), with the low end raised to [`SCORE_FLOOR`] so a
    /// world whose upper scores are below the training average (mostly sea)
    /// boosts nothing.
    ///
    /// An absent (`None`) or non-finite score gives `b = 0` -- today's display
    /// there -- and is excluded from the percentiles; no present scores, or a
    /// top end at or below the floor, gives `b = 0` everywhere. A zero-width
    /// ramp (all scores in the band equal) is a step at that score.
    ///
    /// # Panics
    /// If `scores.len() != n * n`.
    #[must_use]
    pub fn from_scores(n: usize, scores: &[Option<f64>]) -> Self {
        assert_eq!(scores.len(), n * n, "one score per texel");
        let mut present: Vec<f64> = scores.iter().filter_map(|s| s.filter(|v| v.is_finite())).collect();
        if present.is_empty() {
            return Self { n, values: vec![0.0; n * n] };
        }
        present.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
        let centre = 1.0 - f64::from(PROMOTE_PERCENT) / 100.0;
        let half = RAMP_HALF_WIDTH_PERCENT / 100.0;
        let rank = |p: f64| {
            let i = ((p * present.len() as f64).ceil() as usize).clamp(1, present.len()) - 1;
            present[i]
        };
        let lo = rank(centre - half).max(SCORE_FLOOR);
        let hi = rank(centre + half);
        if hi <= SCORE_FLOOR {
            return Self { n, values: vec![0.0; n * n] };
        }
        let values = scores
            .iter()
            .map(|s| match s.filter(|v| v.is_finite()) {
                None => 0.0,
                Some(v) if hi > lo => smoothstep01((v - lo) / (hi - lo)) as f32,
                Some(v) => f32::from(u8::from(v > lo)),
            })
            .collect();
        Self { n, values }
    }

    /// Scores every tile of [`boost_level`] with `weights` (EF-9.1's
    /// `tile_features` + `importance`, unchanged) and maps them with
    /// [`Self::from_scores`]. Cost: `4^level` footprint aggregations of about 64
    /// cells each, once per world.
    #[must_use]
    pub fn build(field: &WorldImportanceField, weights: &ImportanceWeights) -> Self {
        let (gw, gh) = (field.gw(), field.gh());
        let level = boost_level(gw);
        let n = pyramid_dims(level as i32).cols as usize;
        let mut scores = Vec::with_capacity(n * n);
        for row in 0..n as u32 {
            for col in 0..n as u32 {
                let b = pyramid_tile_bounds(gw, gh, level as i32, col, row);
                scores.push(importance(&tile_features(field, &b), weights).score);
            }
        }
        Self::from_scores(n, &scores)
    }

    /// Texels per axis.
    #[must_use]
    pub fn n(&self) -> usize {
        self.n
    }

    /// The texel values, row-major, for the shell's `FORMAT_RF` texture.
    #[must_use]
    pub fn values(&self) -> &[f32] {
        &self.values
    }

    /// Whether any texel is boosted; `false` means EF-9 draws exactly today's
    /// picture for this world.
    #[must_use]
    pub fn any(&self) -> bool {
        self.values.iter().any(|&v| v > 0.0)
    }

    fn at(&self, i: i64, j: i64) -> f64 {
        let m = self.n as i64 - 1;
        f64::from(self.values[(j.clamp(0, m) * self.n as i64 + i.clamp(0, m)) as usize])
    }

    /// `b` at index-space `(u, v)`: bilinear between texel centres, clamped at
    /// the edges -- the arithmetic `lod_tile.gdshader`'s `boost_at` does with
    /// four `texelFetch`es (manual, so no GPU's 8-bit filter weights enter).
    /// Continuous in `(u, v)`.
    #[must_use]
    pub fn sample(&self, u: f64, v: f64) -> f64 {
        if self.n == 0 {
            return 0.0;
        }
        let (px, py) = (u * self.n as f64 - 0.5, v * self.n as f64 - 0.5);
        let (i0, j0) = (px.floor(), py.floor());
        let (fx, fy) = (px - i0, py - j0);
        let (i0, j0) = (i0 as i64, j0 as i64);
        let top = self.at(i0, j0) * (1.0 - fx) + self.at(i0 + 1, j0) * fx;
        let bot = self.at(i0, j0 + 1) * (1.0 - fx) + self.at(i0 + 1, j0 + 1) * fx;
        top * (1.0 - fy) + bot * fy
    }

    /// An upper bound on [`Self::sample`] over the index-space box
    /// `[u0, u1] x [v0, v1]`: the largest texel any point of the box
    /// interpolates from. Never below the true maximum (a bilinear value is a
    /// convex combination of its four texels), so a tile it rejects has `b`
    /// below the threshold everywhere.
    #[must_use]
    pub fn max_over(&self, u0: f64, u1: f64, v0: f64, v1: f64) -> f64 {
        if self.n == 0 {
            return 0.0;
        }
        let m = self.n as i64 - 1;
        let lo = |t: f64| ((t * self.n as f64 - 0.5).floor() as i64).clamp(0, m);
        let hi = |t: f64| ((t * self.n as f64 - 0.5).floor() as i64 + 1).clamp(0, m);
        let mut best = 0.0f64;
        for j in lo(v0)..=hi(v1) {
            for i in lo(u0)..=hi(u1) {
                best = best.max(self.at(i, j));
            }
        }
        best
    }

    /// The tiles of level `z` in `c0..=c1 x r0..=r1` whose [`Self::max_over`]
    /// footprint `[c, c + 1] / 2^z x [r, r + 1] / 2^z` is strictly above `thr`,
    /// as `(col, row)` in row-major order.
    #[must_use]
    pub fn tiles_over(&self, z: u32, c0: u32, c1: u32, r0: u32, r1: u32, thr: f64) -> Vec<(u32, u32)> {
        let n = f64::from(pyramid_dims(z as i32).cols);
        let mut out = Vec::new();
        for r in r0..=r1 {
            for c in c0..=c1 {
                let (u0, v0) = (f64::from(c) / n, f64::from(r) / n);
                if self.max_over(u0, u0 + 1.0 / n, v0, v0 + 1.0 / n) > thr {
                    out.push((c, r));
                }
            }
        }
        out
    }
}

/// Tests of the display model's weights (the continuity condition) and of the
/// boost field's sampling and selection.
#[cfg(test)]
mod tests {
    use super::*;

    /// The per-level contributions of `layer_weights`, as a function of
    /// `lam_zoom` and `b`, with the drawn level chosen exactly as the shell
    /// chooses it -- the whole display model in one call.
    fn display(lam_zoom: f64, b: f64, z_cap: i32) -> std::collections::BTreeMap<i32, f64> {
        let z0 = drawn_level(lam_zoom, z_cap);
        let lam = effective_level(lam_zoom, 1.0, b);
        let mut m = std::collections::BTreeMap::new();
        for (k, w) in layer_weights(lam, z0, z_cap) {
            *m.entry(k).or_insert(0.0) += w;
        }
        m
    }

    /// Largest change of any level's contribution between two displays (a level
    /// absent from one side counts as 0 there).
    fn max_delta(a: &std::collections::BTreeMap<i32, f64>, b: &std::collections::BTreeMap<i32, f64>) -> f64 {
        let mut d = 0.0f64;
        for k in a.keys().chain(b.keys()) {
            d = d.max((a.get(k).copied().unwrap_or(0.0) - b.get(k).copied().unwrap_or(0.0)).abs());
        }
        d
    }

    /// Protects: **the continuity condition across level boundaries**. Sweeps
    /// the zoom level over four boundaries in steps of 1e-4 for boosts from 0 to
    /// 1 and asserts no level's contribution moves by more than
    /// `2 * 1.5 * step` per step (smoothstep's slope bound, twice for the two
    /// layers a pixel can be between, plus rounding). The discrete model this
    /// replaces moved by ~1 in one step at every boundary where a promotion
    /// changed; dropping the `+ 0.5` in `level_weight` moves by 0.5.
    #[test]
    fn no_weight_jumps_across_level_boundaries_for_any_boost() {
        let step = 1e-4;
        let bound = 3.0 * step + 1e-12;
        for &b in &[0.0, 0.1, 0.37, 0.5, 0.83, 0.999, 1.0] {
            let mut prev = display(2.6, b, 10);
            let mut lam = 2.6 + step;
            let mut worst = 0.0f64;
            while lam < 6.7 {
                let cur = display(lam, b, 10);
                worst = worst.max(max_delta(&prev, &cur));
                prev = cur;
                lam += step;
            }
            assert!(worst <= bound, "b = {b}: a weight moved {worst} in one step of {step} (bound {bound})");
        }
    }

    /// Protects: **the proof's step 2** -- at the boundary the old stack and the
    /// new one composite identically for every boost, so the switch itself
    /// changes no pixel. Exact equality of the two `layer_weights` after
    /// dropping zero contributions.
    #[test]
    fn the_old_and_new_stack_agree_exactly_at_the_boundary() {
        for z in 3..8 {
            let lam_zoom = f64::from(z) + 0.5;
            for i in 0..=20 {
                let b = f64::from(i) / 20.0;
                let lam = effective_level(lam_zoom, 1.0, b);
                let nz = |v: Vec<(i32, f64)>| v.into_iter().filter(|&(_, w)| w != 0.0).collect::<Vec<_>>();
                assert_eq!(nz(layer_weights(lam, z, 10)), nz(layer_weights(lam, z + 1, 10)), "z {z}, b {b}");
            }
        }
    }

    /// Protects: **continuity in space** -- at a fixed zoom, the composite is
    /// Lipschitz in `b` (and `b` is continuous in position), so neighbouring
    /// pixels with nearby boosts never differ by more than 3 x their boost
    /// difference. A per-tile (discrete) boost would fail at every tile edge.
    #[test]
    fn the_composite_is_continuous_in_the_boost() {
        let db = 1e-4;
        for &lz in &[3.5, 3.9, 4.2, 4.49, 4.5, 4.51] {
            let mut prev = display(lz, 0.0, 10);
            let mut b = db;
            while b <= 1.0 {
                let cur = display(lz, b, 10);
                assert!(max_delta(&prev, &cur) <= 3.0 * db + 1e-12, "lam_zoom {lz}, b {b}");
                prev = cur;
                b += db;
            }
        }
    }

    /// Protects: **identity at `b = 0`** -- with no boost the drawn level's
    /// weight is LOD-D3's morph (`smoothstep01(lam_zoom - z0 + 0.5)`, the
    /// literal expression of `lod_bridge::morph_for_zoom`) and the `z0 + 1` layer
    /// contributes exactly 0, so an unboosted pixel draws today's picture.
    #[test]
    fn no_boost_is_todays_morph() {
        for i in 0..400 {
            let lz = 2.5 + f64::from(i) * 0.01;
            let z0 = drawn_level(lz, 10);
            let w = layer_weights(effective_level(lz, 1.0, 0.0), z0, 10);
            let t = lz - f64::from(z0) + 0.5;
            let tc = t.clamp(0.0, 1.0);
            let morph = tc * tc * (3.0 - 2.0 * tc);
            assert_eq!(w[1], (z0, morph), "lam_zoom {lz}");
            assert_eq!(w[2], (z0 + 1, 0.0), "lam_zoom {lz}");
        }
    }

    /// Protects: contributions are a partition of unity, and the cap removes the
    /// `z0 + 1` layer (I3) without breaking it.
    #[test]
    fn weights_sum_to_one_and_respect_the_cap() {
        for i in 0..200 {
            let lam = 3.5 + f64::from(i) * 0.01;
            for (z0, cap) in [(4, 10), (4, 4)] {
                let w = layer_weights(lam, z0, cap);
                let s: f64 = w.iter().map(|&(_, v)| v).sum();
                assert!((s - 1.0).abs() < 1e-12, "sum {s}");
                assert!(w.iter().all(|&(k, v)| k <= cap && v >= 0.0));
            }
        }
        assert_eq!(layer_weights(5.0, 4, 4).len(), 2, "no z0 + 1 layer at the cap");
    }

    /// Protects: the literal constants (`MISTAKES.md`: assert a literal, not the
    /// constant against itself) and the drawn level's rounding (half up, like
    /// `js_round`), so a changed constant is a deliberate edit of this test too.
    #[test]
    fn constants_and_rounding_are_the_recorded_values() {
        assert_eq!(BOOST_LEVELS, 1.0);
        assert_eq!(BOOST_FOOTPRINT_CELLS, 8.0);
        assert_eq!(RAMP_HALF_WIDTH_PERCENT, 12.5);
        assert_eq!(PREFETCH_LEVELS, 0.25);
        assert_eq!(drawn_level(4.5, 10), 5);
        assert_eq!(drawn_level(4.4999, 10), 4);
        assert_eq!(drawn_level(12.0, 10), 10);
        assert_eq!(boost_level(512), 6);
        assert_eq!(boost_level(2048), 8);
        assert_eq!(boost_level(1), 0);
    }

    /// Protects: the continuity proof's bound -- `L` never exceeds
    /// `lam_zoom + BOOST_LEVELS` nor falls below `lam_zoom`, whatever `b` or
    /// `gain` a caller passes (an unclamped `b` of 1.7 would need a fourth
    /// layer the shell does not draw).
    #[test]
    fn the_effective_level_is_clamped_to_one_level_of_boost() {
        assert_eq!(effective_level(4.0, 1.0, 1.7), 5.0);
        assert_eq!(effective_level(4.0, 1.0, -0.3), 4.0);
        assert_eq!(effective_level(4.0, 2.0, 1.0), 5.0);
        assert_eq!(effective_level(4.0, 0.5, 0.5), 4.25);
    }

    /// Protects: the entry ramp -- 0 at the entry zoom and below, 1 one octave
    /// past it, 0.5 at half an octave, and 0 (no boost) for an unknown entry.
    #[test]
    fn the_boost_fades_in_over_one_octave_after_entry() {
        assert_eq!(entry_gain(2.2, 2.2), 0.0);
        assert_eq!(entry_gain(1.0, 2.2), 0.0);
        assert_eq!(entry_gain(4.4, 2.2), 1.0);
        assert!((entry_gain(2.2 * 2f64.sqrt(), 2.2) - 0.5).abs() < 1e-12);
        assert_eq!(entry_gain(8.0, 0.0), 0.0);
        assert_eq!(entry_gain(8.0, f64::NAN), 0.0);
    }

    /// Protects: the `z0 + 1` request rule -- `None` at the cap and before any
    /// gain, the prefetch lead, the clamp at 0, and `None` once no boost can
    /// reach the threshold.
    #[test]
    fn plus_one_tiles_are_asked_for_ahead_of_their_weight() {
        assert_eq!(plus_one_threshold(4.0, 1.0, 4, 4), None, "cap");
        assert_eq!(plus_one_threshold(4.0, 0.0, 4, 10), None, "no gain");
        // L reaches z0 + 0.5 = 4.5 at b = 0.5 from lam_zoom 4.0; asked 0.25 early.
        assert_eq!(plus_one_threshold(4.0, 1.0, 4, 10), Some(0.25));
        assert_eq!(plus_one_threshold(4.4, 1.0, 4, 10), Some(0.0), "clamped at 0");
        assert_eq!(plus_one_threshold(3.5, 0.5, 4, 10), None, "threshold 1.5: unreachable");
    }

    /// A 4 x 4 field with one texel at 1 (column 2, row 1), the rest 0.
    fn one_hot() -> BoostField {
        let mut values = vec![0.0f32; 16];
        values[4 + 2] = 1.0;
        BoostField { n: 4, values }
    }

    /// Protects: bilinear sampling between texel centres -- the value at a
    /// texel centre is the texel, half-way between two is their mean, and the
    /// edges clamp. Hand values on `one_hot()`.
    #[test]
    fn sampling_is_bilinear_between_texel_centres() {
        let f = one_hot();
        assert_eq!(f.sample(2.5 / 4.0, 1.5 / 4.0), 1.0);
        assert_eq!(f.sample(3.0 / 4.0, 1.5 / 4.0), 0.5);
        assert_eq!(f.sample(3.0 / 4.0, 2.0 / 4.0), 0.25);
        assert_eq!(f.sample(0.0, 0.0), 0.0);
        assert_eq!(f.sample(1.0, 1.0), 0.0);
    }

    /// Protects: `max_over` is never below the sampled maximum (a rejected tile
    /// really is below the threshold everywhere), on a dense sweep of boxes,
    /// and `tiles_over` picks exactly the level-2 tiles that touch the hot
    /// texel's support.
    #[test]
    fn max_over_bounds_the_field_and_tiles_over_uses_it() {
        let f = one_hot();
        for k in 0..16 {
            let (u0, v0) = (f64::from(k % 4) / 4.0, f64::from(k / 4) / 4.0);
            let mut true_max = 0.0f64;
            for a in 0..=32 {
                for c in 0..=32 {
                    true_max = true_max.max(f.sample(u0 + f64::from(a) / 128.0, v0 + f64::from(c) / 128.0));
                }
            }
            assert!(f.max_over(u0, u0 + 0.25, v0, v0 + 0.25) >= true_max, "box {k}");
        }
        // Texel (2, 1)'s bilinear support is index space (1.5/4, 3.5/4) x (0.5/4, 2.5/4):
        // level-2 tiles with columns 1..=3 and rows 0..=2.
        let got = f.tiles_over(2, 0, 3, 0, 3, 0.0);
        let want: Vec<(u32, u32)> = (0..=2).flat_map(|r| (1..=3).map(move |c| (c, r))).collect();
        assert_eq!(got, want);
        assert!(f.tiles_over(2, 0, 3, 0, 3, 1.0).is_empty(), "strictly above");
    }

    /// Protects: the score-to-boost map -- the ramp's ends at the 62.5th and
    /// 87.5th percentiles (nearest rank), an absent score never boosted, the
    /// floor, and an all-absent or all-low world boosting nothing (today's
    /// display, by value).
    #[test]
    fn scores_map_to_a_ramp_around_the_promotion_cut() {
        // 16 scores 1..=16: nearest rank 62.5 % -> 10, 87.5 % -> 14.
        let s: Vec<Option<f64>> = (1..=16).map(|v| Some(f64::from(v))).collect();
        let f = BoostField::from_scores(4, &s);
        assert_eq!(f.values()[9], 0.0, "score 10 is the low end");
        assert_eq!(f.values()[13], 1.0, "score 14 is the high end");
        assert_eq!(f.values()[11], 0.5, "score 12 is mid-ramp");
        assert_eq!(f.values()[0], 0.0);
        let mut s2 = s.clone();
        s2[15] = None;
        assert_eq!(BoostField::from_scores(4, &s2).values()[15], 0.0, "absent is not boosted");
        assert!(!BoostField::from_scores(4, &vec![None; 16]).any());
        let low: Vec<Option<f64>> = (1..=16).map(|v| Some(-f64::from(v))).collect();
        assert!(!BoostField::from_scores(4, &low).any(), "below the floor");
        // The floor raises the low end: with scores -10..=5 the 62.5 % rank is -1,
        // raised to the floor 0, and the 87.5 % rank is 3.
        let mixed: Vec<Option<f64>> = (-10..=5).map(|v| Some(f64::from(v))).collect();
        let fm = BoostField::from_scores(4, &mixed);
        assert_eq!(fm.values()[10], 0.0, "score 0 is at the floor");
        assert_eq!(fm.values()[13], 1.0, "score 3 is the high end");
        assert!(fm.values()[11] > 0.0 && fm.values()[11] < 1.0, "score 1 is mid-ramp");
    }
}
