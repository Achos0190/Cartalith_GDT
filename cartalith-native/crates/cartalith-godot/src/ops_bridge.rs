//! Ported-and-unexposed engine capability — `PARITY_AUDIT.md` §23 F13.
//!
//! Each of these exists in `cartalith-*`, is tested there, and had no `#[func]`
//! and no caller: `extract_region_as_world`,
//! `estimate_regional_density_km2`, `arc_label_line_width`, the icon brush
//! (`icon_brush_rule` / `icon_brush_stamp`) and the asset-library operations
//! (`to_library_json`, `drop_collection`, `apply_library_file_with_items`,
//! `referenced_files`, `slot_paths`, `filled_count`).
//!
//! Its own file rather than `lib.rs` so this work cannot collide with the
//! other §23 agents editing that file — the same `#[godot_api(secondary)]`
//! split `project_bridge.rs` and `vault_bridge.rs` already use.
//!
//! **`apply_force_lake` was on this list and is not here.** It landed in
//! `erode_bridge.rs` instead, where it sits beside the other terrain op — the
//! two agents working §23 in parallel had both been given it, and a duplicate
//! inherent method is a hard `E0592` rather than anything silent. Recorded so
//! the next reader does not go looking for it under F13.
//!
//! **Two of the listed items are still unwired**, and the reason is the same
//! in each case: closing them properly needs a file this pass did not own, and
//! a half-wired control is worse than an unwired function because it looks
//! finished.
//!
//! * `extract_region_as_world` — its own doc comment says the orchestration
//!   around it (`allocate`, clearing the warp fields, cache invalidation,
//!   climate refresh, emptying the civ layer) is deliberately not ported. That
//!   is new `WorldGen` state in `lib.rs`.
//! * `referenced_files` / `slot_paths` — their one real consumer,
//!   `pack::load_pack_from_bytes`, iterates `manifest.icons`/`manifest.textures`
//!   by hand instead of calling them. Worth consolidating, in that file.
//!
//! ## Two of them stopped being unwired (re-read 2026-08-31)
//!
//! Both entries below used to sit in the list above. They were re-read against
//! the working tree rather than trusted, because a decline that has gone stale
//! is not a neutral comment: the icon-brush one was quoted into an owner ruling
//! that sized `UNIFIED_TOOL_PLAN.md` milestone E as three gaps when two of the
//! three had already shipped.
//!
//! * **`icon_brush_rule` / `icon_brush_stamp` — the decline claimed "there is
//!   no manual-icon tool in the shell at all: nothing arms it, nothing renders
//!   `state.mapIcons`, and nothing stores them". All three clauses are false.**
//!   Arming: `cartography_workspace.gd::_arm_icon_from_ui` calls
//!   `bridge.icon_arm(fam.key, ...)` against `icon_bridge`'s `#[func] icon_arm`.
//!   Rendering: `viewport_host.gd::refresh_annotations` calls
//!   `overlay.set_manual_icons(_bridge.icon_list())`, and `map_overlay.gd`
//!   draws that list. Storing: `project_bridge.rs` writes every placed icon to
//!   `annotations/icons.json` (`SLOT_ICONS`, in `project_save_with_documents`)
//!   and parses it back in `project_open` through `IconsDoc`/`IconDto`.
//!   Click-placement, selection, resize and delete are all live too
//!   (`icon_place`/`icon_hit_test`/`icon_handles`/`icon_resize`/`icon_delete`).
//!
//!   What is genuinely missing is **the brush, not the tool**: no `#[func]`
//!   calls `icon_brush_rule` + `icon_brush_stamp` against `self.icons` on a
//!   drag sample, and the Icon tool-options row has no radius or density
//!   control to drive [`cartalith_assets::manual::IconBrush`]'s
//!   `r`/`density` — `cartography_workspace.gd` builds exactly three
//!   sliders there -- `Scale`, `Rotation` and `Jitter` -- all three of
//!   which `_arm_icon_from_ui` feeds to
//!   `icon_arm` and none of which is a brush parameter. That is one
//!   remaining gap, over an existing tool — not the inside of a tool that
//!   does not exist.
//! * **`to_library_json` / `apply_library_file_with_items` — the decline said
//!   "they need `project_bridge.rs` to read and write the section". It does
//!   now.** `project_bridge.rs::asset_library_document_json` calls
//!   `to_library_json` and `asset_library_restore_document` calls
//!   `apply_library_file_with_items`, both `#[func]`, both over the
//!   `library/assets.json` slot. The one real limitation left is the one those
//!   two carry in their own doc comments and not this list's: item *pixels*
//!   have no channel in the project writer, so a restored library comes back
//!   with its slots, collections and scatter rules and `items == 0`.
//!
//! And one was judged **not a gap at all**: `filled_count` is already derived
//! from real engine data by `asset_library_window.gd`'s `_refresh_rail_counts`,
//! which counts `as_family_slots()`' per-slot `filled` flags. A second binding
//! would be a redundant FFI round-trip answering a question the shell can
//! already answer correctly.

use crate::{WorldGen, WorldSource};
use godot::prelude::*;

#[godot_api(secondary)]
impl WorldGen {
    /// `estimateRegionalDensityKm2` via `currentPopulationDensity`/
    /// `_civRegionalPopulation` (reference HTML lines 6217/6455/23297;
    /// `PARITY_AUDIT.md` §23 F13). Integrates the modeled persons/km² field
    /// over land into one world-level sanity total — the reference's
    /// **other** regional population figure, distinct from and never
    /// feeding into [`Self::civ_agrarian_regional_total`]'s settlement-
    /// sizing ceiling. `cartalith_civ::estimate_regional_density_km2`'s own
    /// doc comment says the same of the field this integrates: "additive to
    /// carrying capacity k, never feeds back into it." Read-only, like the
    /// reference's own `_civRegionalPopulation` ("never touches
    /// generate()/render").
    ///
    /// Empty on no generated world or no civilisation layer, matching
    /// `civ_agrarian_regional_total`'s own guard.
    ///
    /// **Recomputed fresh on every call.** Unlike `CivData::dens`
    /// (`civ_agrarian_regional_total`'s input, retained because
    /// `TIMELINE_SCOPE.md` milestone 5 already needed it for the collapse
    /// stepper), carrying capacity, water access and NPP are not retained
    /// anywhere on `CivData` — this mirrors `compute_civilisation`'s own
    /// soil/water-access/carrying-capacity block instead of reading a
    /// cache, reusing the one piece that *is* retained
    /// (`CivData::water_bodies`) so it does not re-run
    /// `build_water_bodies`'s flood fill. Comparable in cost to a real
    /// slice of `Recompute civilisation`'s own ~1-4s (see that button's own
    /// tooltip) — expected to be called from an explicit button press, not
    /// on every panel refresh.
    #[func]
    fn civ_regional_population(&self) -> VarDictionary {
        let (Some(WorldSource::Generated(ws)), Some(civ)) = (self.source.as_ref(), self.civ.as_ref()) else {
            return VarDictionary::new();
        };
        if self.gw <= 0 || self.gh <= 0 {
            return VarDictionary::new();
        }
        let (gw, gh) = (self.gw as usize, self.gh as usize);
        let sea = self.sea_level;
        let world = self.world;
        let map_width_km = self.map_width_km;

        let biome = cartalith_civ::build_biome_raster(&civ.water_bodies, &ws.temperature, &ws.rainfall);
        let soil_slope = cartalith_civ::build_slope_field(&ws.field, gw, gh, world);
        let lithology = cartalith_civ::build_lithology(
            &ws.field, &ws.age_field, &ws.volcanic_field, &ws.crust_field, &ws.resistance_field, &ws.rainfall, sea,
        );
        let soil = cartalith_civ::build_soil_fertility(&lithology, &ws.temperature, &ws.rainfall, &soil_slope, &ws.age_field);
        let flow_thresh = cartalith_hydrology::river_flow_thresh(gw, gh, gw, map_width_km);
        let water_access = cartalith_civ::build_water_access(&ws.flow_discharge, &ws.field, gw, gh, sea, flow_thresh);
        let wetland = if self.civ_options.biome_k {
            Some(cartalith_civ::build_wetland_mask(&civ.water_bodies, &ws.field, &ws.rainfall, &soil_slope, sea))
        } else {
            None
        };
        let carrying_cap = cartalith_civ::build_carrying_capacity(
            &soil, &water_access, Some(&biome), &ws.temperature, &ws.field, sea,
            if self.civ_options.biome_k { 1.0 } else { 0.0 }, wetland.as_deref(),
        );
        let npp = cartalith_civ::build_npp(&ws.temperature, &ws.rainfall, &ws.field, sea, 3000.0);
        let dens = cartalith_civ::estimate_regional_density_km2(
            &carrying_cap, &water_access, Some(&biome), Some(&npp), &ws.field, sea, wetland.as_deref(),
        );

        // `_civRegionalPopulation` (reference line 23297): uniform cellKm²
        // over land, plus the painted-territory share when one exists.
        let cell_km = map_width_km / gw as f64;
        let cell_km2 = cell_km * cell_km;
        let has_territory = civ.territory.len() == gw * gh;
        let mut total = 0.0f64;
        let mut land_cells = 0i64;
        let mut claimed = 0.0f64;
        for i in 0..dens.len() {
            if (ws.field[i] as f64) < sea {
                continue;
            }
            land_cells += 1;
            let p = dens[i] as f64 * cell_km2;
            total += p;
            if has_territory && civ.territory[i] > 0 {
                claimed += p;
            }
        }
        vdict! {
            "total" => total.round() as i64,
            "land_km2" => (land_cells as f64 * cell_km2).round() as i64,
            "claimed" => claimed.round() as i64,
        }
    }

    /// `ctx.lineWidth = Math.max(1, sizePx * 0.16)` — the arc-label halo
    /// stroke (`cartalith_civ::labels::arc_label_line_width`,
    /// `PARITY_AUDIT.md` §23 F13).
    ///
    /// **The one existing call site deliberately does not use this, and that
    /// is the right answer rather than an unfinished one.** `map_overlay.gd`'s
    /// `_draw_labels` (reference `drawArcLabel`, HTML line ~15244) computes
    /// `maxi(1, int(font_px * 0.16))` inline. Routing it through here was
    /// considered and rejected on 2026-08-26: that file holds no bridge
    /// reference at all — it is a pure renderer fed data — and the expression
    /// sits inside a per-label draw loop, so calling the engine would add an
    /// FFI round-trip per label per frame to save one multiplication. That is
    /// strictly worse than the duplication.
    ///
    /// The binding is kept for callers that are not in a draw loop, and this
    /// paragraph exists so a later reachability audit does not re-flag the
    /// GDScript copy as drift. The two are one constant apart; if `0.16` ever
    /// moves, both move.
    #[func]
    fn arc_label_line_width(&self, size_px: f64) -> f64 {
        cartalith_civ::labels::arc_label_line_width(size_px)
    }

    /// `AssetCollections::drop_collection` (`PARITY_AUDIT.md` §23 F13): drop
    /// a whole collection by name. The Collections rail
    /// (`asset_library_window.gd`, AS-12) can create a collection
    /// (`as_batch_collect`) and browse it (`as_collections`) but had no way
    /// to remove one outright — only per-uid removal existed, and only as a
    /// side effect of `as_batch_delete`/`remove_custom_slot` dropping a uid
    /// from every collection it was in.
    ///
    /// A no-op, not an error, for an unknown `name` — `AssetCollections::
    /// remove`'s own "the collection itself is dropped once it becomes
    /// empty" rule already treats a missing collection as equivalent to an
    /// empty one.
    #[func]
    fn as_drop_collection(&mut self, name: GString) -> VarDictionary {
        self.asset_library.db.collections.drop_collection(&name.to_string());
        vdict! { "ok" => true }
    }
}
