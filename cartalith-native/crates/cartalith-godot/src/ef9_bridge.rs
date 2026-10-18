//! **EF-9.4** -- importance-driven refinement, wired to the viewport
//! (`ELEVATION_FIELD_ARCHITECTURE_RESEARCH.md` §8.3, milestone EF-9.4).
//!
//! Its own file with a `#[godot_api(secondary)]` block, like
//! `campaign_bridge.rs`.
//!
//! **Since 2026-10-18 the viewport draws EF-9 through the continuous effective
//! level** (`cartalith_engine::continuous_lod`, §8.5): the `#[func]`s it calls
//! are [`WorldGen::lod_continuous_level`], [`WorldGen::lod_ef9_gain`],
//! [`WorldGen::lod_ef9_plus_one_threshold`], [`WorldGen::lod_ef9_boost`],
//! [`WorldGen::lod_ef9_boost_texture`] and [`WorldGen::lod_ef9_boost_tiles`].
//! The three below belong to the first, discrete display model (per-level
//! promotion), which the shell no longer calls; they remain callable, with
//! their engine functions and tests, and `_lodsweep_probe.gd` still times the
//! conformed synthesis. Removing them is an open row, not done here:
//!
//! - [`WorldGen::lod_ef9_select`] -- EF-9.2's selection
//!   (`cartalith_engine::subdivision::select_tiles`) for the view the shell
//!   is drawing, with EF-9.4's conform plan
//!   (`cartalith_engine::subdivision::conform_plan`) and the synthesis order;
//! - [`WorldGen::lod_request_tile_conformed`] /
//!   [`WorldGen::lod_synthesize_tile_conformed`] -- the asynchronous and
//!   synchronous tile paths for a tile drawn beside coarser ground (EF-9.3's
//!   geomorph, `lod_bridge::synthesize_tile_rgba_rivers_conformed`).
//!
//! # What it caches, and on what
//!
//! The plan's *"the importance features are cached per world"*: one
//! [`Ef9Cache`] on `WorldGen` holds the prepared rasters
//! (`cartalith_engine::importance::WorldImportanceField`, four full-grid
//! derivative passes), the Q2 policy inputs, and every tile score computed so
//! far, keyed by [`Ef9Cache::key_for`]. A key change (another world, a sculpt
//! or erode that moved the height stage, a sea-level change, a civ recompute
//! that moved the civ stage) rebuilds it whole on the next call.
//! **Keeping it current incrementally after an edit is EF-9.5**, not this
//! milestone: here an edit simply costs one rebuild.
//!
//! # What it must never do
//!
//! Change a tile's content at a given `(z, col, row, conform)` (I1), move a
//! river threshold (I2), or return a tile deeper than the caller's cap (I3:
//! `z_cap` is also clamped to `lod_bridge::MAX_LEVEL` here, so a bad caller
//! cannot exceed it). With the shell's toggle off none of this is called at
//! all -- the toggle-off display is today's by control flow in
//! `viewport_host.gd`, not by an identity this file would have to maintain.

use std::collections::HashMap;

use cartalith_engine::continuous_lod::{entry_gain, plus_one_threshold, BoostField};
use cartalith_engine::importance::{importance, policy_rank, tile_features, PolicyField, WorldImportanceField, FITTED_WEIGHTS};
use cartalith_engine::staleness::PipelineStage;
use cartalith_engine::subdivision::{conform_plan, select_tiles, Basis, ViewImportance, ViewRange};
use cartalith_spatial::pyramid::{pyramid_dims, pyramid_tile_bounds};
use godot::classes::image::Format;
use godot::classes::{Image, ImageTexture};
use godot::prelude::*;

use crate::{lod_bridge, lod_worker, WorldGen, WorldSource};

/// One world's EF-9 inputs, built lazily by [`WorldGen::lod_ef9_select`].
/// Main-thread only (it lives in a `RefCell`); nothing here crosses a thread.
#[derive(Default)]
pub(crate) struct Ef9Cache {
    /// [`Ef9Cache::key_for`] of the world these were built from; empty = never
    /// built.
    key: String,
    /// The prepared terrain/hydrology/geology rasters; `None` when the world
    /// has no usable field.
    field: Option<WorldImportanceField>,
    /// The Q2 settlement/road inputs; `None` when the world has no civ layer
    /// (no information -- never a count of zero).
    policy: Option<PolicyField>,
    /// `(z, col, row) -> (score, policy rank)`, filled as views are scored.
    /// Bounded by what the user has looked at: a view is a few dozen tiles.
    scores: HashMap<(u32, u32, u32), (Option<f64>, Option<u8>)>,
    /// The continuous model's `b(x)` (`continuous_lod::BoostField::build` over
    /// `field` with `FITTED_WEIGHTS`), built on first use and dropped with the
    /// rest of the cache when the key moves.
    boost: Option<BoostField>,
}

impl Ef9Cache {
    /// Everything the cached rasters and scores are a function of, derived from
    /// [`WorldImportanceField::build`]'s and [`PolicyField::new`]'s arguments
    /// (`MISTAKES.md`: a cache key is derived from the definition): which world
    /// (`world_epoch`), its height (the Height stage version: sculpt, erode,
    /// undo and redo all mark it), the grid, the sea level the land mask is
    /// cut at, the wrap flag and map width that fix the TPI radius, and the civ
    /// layer (the Civ stage version and whether one exists). Stream order,
    /// boundary type and the volcanic field change only with a new world.
    fn key_for(w: &WorldGen) -> String {
        format!(
            "e{};h{};c{};{}x{};s{};w{};km{};civ{}",
            w.world_epoch,
            w.stages.version(PipelineStage::Height.id(), 0),
            w.stages.version(PipelineStage::Civ.id(), 0),
            w.gw,
            w.gh,
            w.sea_level.to_bits(),
            w.world,
            w.map_width_km.to_bits(),
            w.civ.is_some(),
        )
    }
}

/// The view a call scores, refused (as `None`) unless it is a real rectangle
/// inside level `z0`'s own `2^z0 x 2^z0` grid -- the precondition
/// `ViewRange` documents as the caller's.
fn view_in_level(z0: i32, c0: i32, c1: i32, r0: i32, r1: i32) -> Option<ViewRange> {
    if !(0..=lod_bridge::MAX_LEVEL).contains(&z0) || c0 < 0 || r0 < 0 || c1 < c0 || r1 < r0 {
        return None;
    }
    let n = pyramid_dims(z0).cols as i32;
    (c1 < n && r1 < n).then(|| ViewRange { c0: c0 as u32, c1: c1 as u32, r0: r0 as u32, r1: r1 as u32 })
}

/// The string the shell reads for a [`Basis`]; the shell takes EF-9's path
/// only on `"ranked"` and today's path on every other answer.
fn basis_name(b: Basis) -> &'static str {
    match b {
        Basis::Disabled => "disabled",
        Basis::Mismatch => "mismatch",
        Basis::Absent => "absent",
        Basis::Flat => "flat",
        Basis::Ranked => "ranked",
    }
}

impl WorldGen {
    /// Rebuilds [`Ef9Cache`] when its key moved. `false` when there is no world
    /// to build from (the caller then answers `{}`).
    fn ef9_ensure(&self) -> bool {
        let key = Ef9Cache::key_for(self);
        let mut c = self.ef9.borrow_mut();
        if c.key == key && c.field.is_some() {
            return true;
        }
        let (gw, gh) = (self.gw.max(0) as usize, self.gh.max(0) as usize);
        if gw < 2 || gh < 2 {
            return false;
        }
        let field = match self.source.as_ref() {
            Some(WorldSource::Generated(ws)) if ws.field.len() == gw * gh => Some(WorldImportanceField::build(
                &ws.field,
                gw,
                gh,
                self.sea_level,
                self.world,
                self.map_width_km,
                ws.stream_order.as_deref(),
                &ws.boundary_type,
                &ws.volcanic_field,
            )),
            // A terrain-only save carries no stream order, boundary type or
            // volcanic field: those terms are reported missing, never zero.
            Some(WorldSource::Loaded(save)) if save.fields.heightmap.len() == gw * gh => Some(WorldImportanceField::build(
                &save.fields.heightmap,
                gw,
                gh,
                self.sea_level,
                self.world,
                self.map_width_km,
                None,
                &[],
                &[],
            )),
            _ => None,
        };
        let Some(field) = field else { return false };
        let policy = self.civ.as_ref().and_then(|civ| {
            let settlements = civ.settlements.iter().map(|s| (s.placement.x, s.placement.y)).collect();
            let ways: Vec<Vec<(f64, f64)>> = civ.ways.iter().filter(|w| !w.hidden).map(|w| w.pts.clone()).collect();
            PolicyField::new(gw, gh, settlements, &ways)
        });
        *c = Ef9Cache { key, field: Some(field), policy, scores: HashMap::new(), boost: None };
        true
    }

    /// [`Self::ef9_ensure`], then the boost field, built once per cache key.
    /// `false` when there is no world (or no usable field).
    fn ef9_boost_ready(&self) -> bool {
        if !self.ef9_ensure() {
            return false;
        }
        let mut c = self.ef9.borrow_mut();
        if c.boost.is_none() {
            let Some(field) = c.field.as_ref() else { return false };
            let b = BoostField::build(field, &FITTED_WEIGHTS);
            c.boost = Some(b);
        }
        true
    }
}

#[godot_api(secondary)]
impl WorldGen {
    /// **EF-9.2's selection for the shell's view**, with EF-9.4's conform plan.
    ///
    /// `z0` is the level the shell would draw today (`lod_level_for_zoom`,
    /// clamped as it clamps it), `z_cap` the deepest level it may draw
    /// (`min(lod_max_level, tier max_level)`; clamped again to
    /// `lod_bridge::MAX_LEVEL` here), and `c0..=c1 x r0..=r1` the `z0` tiles in
    /// view. Returns:
    ///
    /// - `basis` -- `"ranked"` when importance discriminated between the tiles
    ///   (the shell's EF-9 path), else `"absent"`/`"flat"`/... (the shell keeps
    ///   today's path; see `cartalith_engine::subdivision::Basis`);
    /// - `promoted` -- how many `z0` tiles were replaced by their children;
    /// - `tiles` -- `PackedInt32Array` of `z, col, row, mask` quadruples, the
    ///   exact cover (`Selection::tiles`) with each tile's conform mask;
    /// - `order` -- `PackedInt32Array`, indices into `tiles`, synthesise first
    ///   to last (importance, then Q2 policy rank, then nearness to the centre);
    /// - `fallbacks` -- `PackedInt32Array` of `col, row, mask` triples at `z0`:
    ///   each promoted tile, drawn whole under its children (see
    ///   `cartalith_engine::subdivision::ConformPlan`);
    /// - `terms_missing` -- `PackedStringArray`, the fitted terms this world
    ///   cannot supply (a loaded save: `stream_order_max`,
    ///   `boundary_fraction`, `volcanic_max`), and `policy` -- whether the Q2
    ///   settlement/road ranks were available.
    ///
    /// `{}` (every key absent) before a world, for a view outside the level, or
    /// mid-generation-safe refusals -- the shell reads an empty answer as
    /// "draw today's way". `budget` passed to `select_tiles` is unbounded:
    /// `PROMOTE_PERCENT` (25 % of the view's scored tiles) is the cap on
    /// promotions, and the shell's `tiles_per_update` caps synthesis per call
    /// (I3), as it does today.
    #[func]
    fn lod_ef9_select(&self, z0: i32, z_cap: i32, c0: i32, c1: i32, r0: i32, r1: i32) -> VarDictionary {
        let Some(view) = view_in_level(z0, c0, c1, r0, r1) else { return VarDictionary::new() };
        if !self.ef9_ensure() {
            return VarDictionary::new();
        }
        let mut c = self.ef9.borrow_mut();
        let Ef9Cache { field, policy, scores, .. } = &mut *c;
        let Some(field) = field.as_ref() else { return VarDictionary::new() };
        let (gw, gh) = (field.gw(), field.gh());
        let z0u = z0 as u32;
        let mut sv = Vec::with_capacity(view.len());
        let mut rv = Vec::with_capacity(view.len());
        for row in view.r0..=view.r1 {
            for col in view.c0..=view.c1 {
                let (s, r) = *scores.entry((z0u, col, row)).or_insert_with(|| {
                    let b = pyramid_tile_bounds(gw, gh, z0, col, row);
                    let s = importance(&tile_features(field, &b), &FITTED_WEIGHTS).score;
                    let r = policy.as_ref().and_then(|p| policy_rank(&p.inputs(&b)));
                    (s, r)
                });
                sv.push(s);
                rv.push(r);
            }
        }
        let ranks = policy.as_ref().map(|_| rv);
        let Some(vi) = ViewImportance::new(view, sv, ranks) else { return VarDictionary::new() };
        let cap = z_cap.clamp(0, lod_bridge::MAX_LEVEL) as u32;
        let sel = select_tiles(view, z0u, cap, usize::MAX, Some(&vi));
        let plan = conform_plan(view, z0u, &sel);
        let mut tiles = PackedInt32Array::new();
        for (t, m) in sel.tiles.iter().zip(&plan.masks) {
            for v in [t.z as i32, t.col as i32, t.row as i32, i32::from(*m)] {
                tiles.push(v);
            }
        }
        let order: PackedInt32Array = sel.order.iter().map(|&i| i as i32).collect();
        let mut fallbacks = PackedInt32Array::new();
        for (t, m) in &plan.fallbacks {
            for v in [t.col as i32, t.row as i32, i32::from(*m)] {
                fallbacks.push(v);
            }
        }
        let mut missing = PackedStringArray::new();
        for (name, present) in [
            ("stream_order_max", field.has_stream_order()),
            ("boundary_fraction", field.has_boundary()),
            ("volcanic_max", field.has_volcanic()),
        ] {
            if !present {
                missing.push(name);
            }
        }
        dict! {
            "basis" => basis_name(sel.basis),
            "promoted" => sel.promoted as i64,
            "tiles" => &tiles,
            "order" => &order,
            "fallbacks" => &fallbacks,
            "terms_missing" => &missing,
            "policy" => policy.is_some(),
        }
    }

    /// **The zoom's continuous pyramid level** (`lod_bridge::continuous_level`
    /// for this world's width): the unrounded number `lod_level_for_zoom`
    /// rounds and `lod_morph` fades over, which the continuous model adds its
    /// boost to. `0.0` before a world.
    #[func]
    fn lod_continuous_level(&self, px_per_cell: f64) -> f64 {
        lod_bridge::continuous_level(px_per_cell, self.gw.max(0) as usize)
    }

    /// The boost's fade-in after LOD entry (`continuous_lod::entry_gain`):
    /// `0` at `zoom_entry`, `1` one zoom octave past it, `0` for an unknown
    /// entry zoom.
    #[func]
    fn lod_ef9_gain(&self, zoom: f64, zoom_entry: f64) -> f64 {
        entry_gain(zoom, zoom_entry)
    }

    /// The boost above which a `z0 + 1` tile must be live
    /// (`continuous_lod::plus_one_threshold`), or `null` when none is needed
    /// (at the cap -- I3 --, before any gain, or past every boost). `null`,
    /// never a sentinel number: every value in `[0, 1)` is a real threshold.
    #[func]
    fn lod_ef9_plus_one_threshold(&self, lam_zoom: f64, gain: f64, z0: i32, z_cap: i32) -> Variant {
        let cap = z_cap.min(lod_bridge::MAX_LEVEL);
        plus_one_threshold(lam_zoom, gain, z0, cap).map_or(Variant::nil(), |t| t.to_variant())
    }

    /// This world's boost field, described: `key` (the cache key it was built
    /// under -- the shell refetches the texture when it changes), `n` (texels
    /// per axis), `any` (whether any texel is boosted; `false` = EF-9 draws
    /// today's picture). `{}` before a world.
    #[func]
    fn lod_ef9_boost(&self) -> VarDictionary {
        if !self.ef9_boost_ready() {
            return VarDictionary::new();
        }
        let c = self.ef9.borrow();
        let Some(b) = c.boost.as_ref() else { return VarDictionary::new() };
        dict! { "key" => c.key.as_str(), "n" => b.n() as i64, "any" => b.any() }
    }

    /// The boost field as an `n x n` `FORMAT_RF` texture, texel `(i, j)` at
    /// pyramid index-space `((i + 0.5) / n, (j + 0.5) / n)`, for
    /// `lod_tile.gdshader`'s `boost_tex` (which samples it with `texelFetch`, so
    /// no filter enters). `null` before a world.
    #[func]
    fn lod_ef9_boost_texture(&self) -> Option<Gd<ImageTexture>> {
        if !self.ef9_boost_ready() {
            return None;
        }
        let c = self.ef9.borrow();
        let b = c.boost.as_ref()?;
        let n = b.n() as i32;
        let bytes: Vec<u8> = b.values().iter().flat_map(|v| v.to_le_bytes()).collect();
        let image = Image::create_from_data(n, n, false, Format::RF, &PackedByteArray::from(bytes))?;
        ImageTexture::create_from_image(&image)
    }

    /// The level-`z` tiles in `c0..=c1 x r0..=r1` whose boost is above `thr`
    /// somewhere in their footprint (`BoostField::tiles_over`, a conservative
    /// bound), as `col, row` pairs. Empty before a world or for a view outside
    /// level `z` (`view_in_level`'s guard).
    #[func]
    fn lod_ef9_boost_tiles(&self, z: i32, c0: i32, c1: i32, r0: i32, r1: i32, thr: f64) -> PackedInt32Array {
        let mut out = PackedInt32Array::new();
        let Some(v) = view_in_level(z, c0, c1, r0, r1) else { return out };
        if !self.ef9_boost_ready() {
            return out;
        }
        let c = self.ef9.borrow();
        let Some(b) = c.boost.as_ref() else { return out };
        for (col, row) in b.tiles_over(z as u32, v.c0, v.c1, v.r0, v.r1, thr) {
            out.push(col as i32);
            out.push(row as i32);
        }
        out
    }

    /// [`WorldGen::lod_request_tile`] for EF-9.4's conformed tile: `conform`
    /// is the tile's 8-neighbour mask from [`Self::lod_ef9_select`] (`0` is
    /// exactly `lod_request_tile`). `false` for a mask outside `0..=255`, and
    /// on every refusal `lod_request_tile` has.
    #[func]
    fn lod_request_tile_conformed(&self, z: i32, col: i32, row: i32, conform: i32) -> bool {
        let Ok(mask) = u8::try_from(conform) else { return false };
        self.lod_worker.request_conformed(z, col, row, mask, lod_worker::budget_for_tier(self.quality).max_in_flight)
    }

    /// [`WorldGen::lod_synthesize_tile`] for EF-9.4's conformed tile, on the
    /// calling thread -- the path the shell takes against a pool that could not
    /// be built, and the one a probe can time. `null` exactly where
    /// `lod_synthesize_tile` is, and for a mask outside `0..=255`.
    #[func]
    fn lod_synthesize_tile_conformed(&self, z: i32, col: i32, row: i32, conform: i32) -> Option<Gd<ImageTexture>> {
        let mask = u8::try_from(conform).ok()?;
        let snap = self.lod_snapshot()?;
        let (rgba, w, h) = self.lod_worker.tile_conformed(&snap, z, col, row, mask)?;
        let packed = PackedByteArray::from(rgba);
        let image = Image::create_from_data(w as i32, h as i32, false, Format::RGBA8, &packed)?;
        ImageTexture::create_from_image(&image)
    }
}

/// Unit tests of the pure helpers; the `#[func]`s need a live `WorldGen`, which
/// the windowed `_lodsweep_probe.gd` drives (`--ef9 on|off`).
#[cfg(test)]
mod tests {
    use super::*;

    /// Protects: the view guard -- a rectangle reversed, negative, past the
    /// level's `2^z` edge or at a level outside `0..=MAX_LEVEL` is refused, and
    /// a real one round-trips. A guard that let `c1 == 2^z` through would hand
    /// `select_tiles` an index past its level (the caller's precondition).
    #[test]
    fn the_view_must_lie_inside_its_level() {
        assert_eq!(view_in_level(3, 1, 4, 0, 7), Some(ViewRange { c0: 1, c1: 4, r0: 0, r1: 7 }));
        assert_eq!(view_in_level(3, 0, 8, 0, 0), None, "c1 == 2^3");
        assert_eq!(view_in_level(3, 0, 0, 0, 8), None, "r1 == 2^3");
        assert_eq!(view_in_level(3, 2, 1, 0, 0), None, "reversed");
        assert_eq!(view_in_level(3, -1, 1, 0, 0), None, "negative");
        assert_eq!(view_in_level(-1, 0, 0, 0, 0), None, "level below 0");
        assert_eq!(view_in_level(lod_bridge::MAX_LEVEL + 1, 0, 0, 0, 0), None, "level past MAX_LEVEL");
        assert!(view_in_level(0, 0, 0, 0, 0).is_some(), "the whole map at level 0");
    }

    /// Protects: the basis strings the shell branches on -- only `"ranked"`
    /// takes EF-9's path, so a renamed variant must not silently send every
    /// view down it (or none).
    #[test]
    fn the_basis_names_are_the_shells_literals() {
        assert_eq!(basis_name(Basis::Ranked), "ranked");
        assert_eq!(
            [Basis::Disabled, Basis::Mismatch, Basis::Absent, Basis::Flat].map(basis_name),
            ["disabled", "mismatch", "absent", "flat"]
        );
    }
}
