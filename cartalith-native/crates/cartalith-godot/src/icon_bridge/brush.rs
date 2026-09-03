//! CARTO ▸ Icon — the **density brush**'s Godot surface.
//!
//! `UNIFIED_TOOL_PLAN.md` milestone E's last open half. That plan sized the
//! manual-icon row as three gaps — arming, rendering and persistence — and by
//! 2026-08-31 all three had shipped; `ops_bridge.rs`'s own re-read says so and
//! names what was actually left: *"no `#[func]` calls `icon_brush_rule` +
//! `icon_brush_stamp` against `self.icons` on a drag sample, and the Icon
//! tool-options row has no radius or density control."* This file is that
//! `#[func]`, and `cartography_workspace.gd` is those controls.
//!
//! ## Why the brush is the larger of the two manual paths
//!
//! `UNIFIED_TOOL_PLAN.md`'s own correction, kept here because it is the thing
//! a reader of `icon_place` would otherwise get wrong: the reference has
//! **three** placement paths, not two. Rule-driven autoplacement
//! (`place_map_icons_ruled`, wired next door in `generate.rs`); click-to-place
//! one icon (`place_manual_icon`, four lines, wired as `icon_place`); and
//! `_carIconBrushStamp`, which paints a blue-noise *stand* of icons under a
//! radius as the pointer drags. The third is by far the biggest of the manual
//! two and is what this file finally reaches.
//!
//! ## Three differences from the click path, all deliberate
//!
//! 1. **The brush has a sea-level gate and the click path does not.** Ported
//!    as written both times — `place_manual_icon`'s own doc comment explains
//!    the asymmetry (a hand-placed lighthouse is a legitimate thing to want; a
//!    painted forest standing in the sea is not).
//! 2. **The arm-time `scale` does not apply** — see
//!    [`super::IconEditor::brush_stamp`]'s own doc comment.
//! 3. **It is deliberately non-deterministic across repeats.** The reference
//!    uses `Math.random` and says why; this port advances one long-lived
//!    stream instead of hashing position, which gives the same property. See
//!    [`super::BRUSH_SEED`].
//!
//! ## Where the scatter rule comes from
//!
//! `icon_brush_rule` reads the very same `ScatterRuleTable` the autoplacement
//! pass does, keyed by the armed slot — which is the whole argument
//! `UNIFIED_TOOL_PLAN.md` gave for putting the manual icon in
//! `cartalith-assets` in the first place (*"a manual icon addresses the same
//! slot vocabulary, and `icon_brush_rule` reads the very same `ScatterRule`
//! table `place_map_icons_ruled` does"*). The table is rebuilt from the loaded
//! pack's manifest per stamp, exactly as `crate::pack::composite_map_icons`
//! rebuilds it per render, rather than cached on `WorldGen` where it could go
//! stale against a reloaded pack.

use godot::prelude::*;

use cartalith_assets::manual::icon_brush_rule;
use cartalith_assets::{autopopulate_scatter_rules, ScatterRuleTable};

use crate::{WorldGen, WorldSource};

#[godot_api(secondary)]
impl WorldGen {
    /// The density brush's own three controls — `carIconBrushChk` /
    /// `carIconBrushR` / `carIconBrushD` (reference lines 1654-1657).
    ///
    /// `radius` is clamped to the reference slider's own `2..60` and
    /// `density` to its own `0.05..2.00`
    /// ([`super::ICON_BRUSH_R_MIN`], [`super::ICON_BRUSH_DENSITY_MIN`]).
    /// Returns `false` when either was non-finite — in which case `on` still
    /// applies and the two numbers keep their previous values, so a bad
    /// slider cannot make the brush unturnoffable.
    ///
    /// `false` with nothing changed at all when there is no world yet: the
    /// editor is created by `absorb()`, so there is nowhere to keep this.
    #[func]
    fn icon_brush_set(&mut self, on: bool, radius: f64, density: f64) -> bool {
        let Some(icons) = self.icons.as_mut() else { return false };
        icons.set_brush(on, radius, density)
    }

    /// What the brush is currently set to — `on`, `radius`, `density`.
    ///
    /// Empty `Dictionary` when there is no world, matching `icon_armed`'s own
    /// convention for "there is nothing to report" rather than inventing a
    /// row of plausible defaults the engine is not actually holding.
    #[func]
    fn icon_brush(&self) -> VarDictionary {
        let Some(icons) = self.icons.as_ref() else { return VarDictionary::new() };
        vdict! {
            "on" => icons.brush.on,
            "radius" => icons.brush.r,
            "density" => icons.brush.density,
        }
    }

    /// One brush stamp at grid cell `(gx, gy)` — the number of icons it
    /// added.
    ///
    /// `0` is a real answer, not a failure code: a stamp whose darts all
    /// landed in water, or all inside the blue-noise spacing of icons already
    /// there, legitimately places nothing. The shell uses it only to decide
    /// whether the annotation layer needs redrawing, which is exactly what
    /// the reference's own `if(_carIconBrushStamp(gx,gy)) drawCivLayerAuto()`
    /// (line 9719) does with it.
    ///
    /// `0` also covers every "cannot stamp" case, and each of those is a
    /// state the shell can see for itself before it drags: no world
    /// (`has_world`), no editor, the brush switched off (`icon_brush`),
    /// nothing armed (`icon_armed`), or no asset pack (`has_asset_pack`).
    #[func]
    fn icon_brush_stamp(&mut self, gx: f64, gy: f64) -> i64 {
        let (gw, gh) = (self.gw.max(0) as usize, self.gh.max(0) as usize);
        if gw == 0 || gh == 0 {
            return 0;
        }
        let sea = self.sea_level;

        // Rebuilt per stamp from the live pack -- see the module doc.
        let Some(pack) = self.asset_pack.as_ref() else { return 0 };
        let mut table = ScatterRuleTable::default();
        autopopulate_scatter_rules(&mut table, &pack.manifest);

        // `self.source` and `self.icons` are borrowed as *fields* rather than
        // through `icon_gen_field()`, which borrows the whole of `self` and
        // would collide with the `&mut` below.
        let field: &[f32] = match self.source.as_ref() {
            Some(WorldSource::Generated(ws)) => &ws.field,
            Some(WorldSource::Loaded(save)) => &save.fields.heightmap,
            None => return 0,
        };
        let Some(icons) = self.icons.as_mut() else { return 0 };
        let Some(rule) = icon_brush_rule(icons.armed.as_ref().map(|a| &a.icon), &table) else {
            return 0;
        };
        icons.brush_stamp(&rule, field, gw, gh, sea, gx, gy) as i64
    }
}
