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
//! **One of the listed items is still unwired**, and the reason is that
//! closing it properly needs a file this pass did not own, and a half-wired
//! control is worse than an unwired function because it looks finished.
//!
//! * `referenced_files` / `slot_paths` — their one real consumer,
//!   `pack::load_pack_from_bytes`, iterates `manifest.icons`/`manifest.textures`
//!   by hand instead of calling them. Worth consolidating, in that file.
//!
//! **`extract_region_as_world` is wired, here** ([`WorldGen::region_new_world`],
//! `LARGE_ITEM_RULINGS.md`'s *"schedule separately"* row). The decline above
//! used to say its orchestration — *"`allocate`, clearing the warp fields,
//! cache invalidation, climate refresh, emptying the civ layer"* — "is new
//! `WorldGen` state in `lib.rs`". Four of those five needed no new state at
//! all, which the decline could not have known without opening the reference's
//! own handler:
//!
//! * `allocate()` + `refreshClimate()` + `computeFlow(true)` over a supplied
//!   field, with `warpX`/`warpY` null, **is** [`cartalith_engine::import::
//!   infer_tectonics`] — the reference reaches it from this button too, via the
//!   `_setupOpen('calibrate')` call the handler ends on (reference line 13240)
//!   and `_suCalCommit`'s `inferTectonics()` (line 13830).
//! * **There are no warp fields to clear.** `warp_x`/`warp_y` are locals of
//!   `generate_terrain`, consumed by `compute_height` and never stored on
//!   `WorldState`; `infer_tectonics` already passes `None`/`None` and says so.
//! * Cache invalidation and emptying the civ layer are `release_world()` +
//!   `absorb()`, which between them already replace every world-scoped field on
//!   `WorldGen`.
//!
//! So the whole orchestration is the three-call sequence in
//! `region_new_world` below, and only two decisions in it are genuinely new:
//! the refusal floor it carries, and the primitives-in/`bool`-out shape the
//! worker thread forces (`landmark_run`'s rule -- see that function's own
//! doc, and this one's).
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
//!   **That left one gap, and it closed 2026-09-03 — this whole entry is now
//!   history, not a decline.** What was still missing was *the brush, not the
//!   tool*: no `#[func]` called `icon_brush_rule` + `icon_brush_stamp` against
//!   `self.icons` on a drag sample, and the Icon tool-options row had no
//!   radius or density control to drive
//!   [`cartalith_assets::manual::IconBrush`]'s `r`/`density` — it built
//!   exactly three sliders, `Scale`/`Rotation`/`Jitter`, none of which is a
//!   brush parameter. Both halves now exist: `icon_bridge/brush.rs` binds
//!   `icon_brush_set`/`icon_brush`/`icon_brush_stamp`, and
//!   `cartography_workspace.gd::_build_icon_brush_controls` adds the toggle
//!   and the two sliders behind it, routed through `_on_icon_click`/
//!   `_on_icon_drag`/`_on_icon_release` on the reference's own precedence.
//!   Verified end to end by `godot-project/_iconbrush_probe.tscn` (32
//!   assertions over a live `WorldGen`, ALL PASS), which is the only place the
//!   `#[func]` layer can be exercised at all.
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
        let wetland = if self.params.civ.biome_k {
            Some(cartalith_civ::build_wetland_mask(&civ.water_bodies, &ws.field, &ws.rainfall, &soil_slope, sea))
        } else {
            None
        };
        let carrying_cap = cartalith_civ::build_carrying_capacity(
            &soil, &water_access, Some(&biome), &ws.temperature, &ws.field, sea,
            if self.params.civ.biome_k { 1.0 } else { 0.0 }, wetland.as_deref(),
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

    /// `regionNewWorldBtn` (reference HTML line 13219): **replace the current
    /// world with a higher-resolution resample of the Region-select marquee.**
    ///
    /// Reads the same `self.infra.region` rect `region_set`/`region_get`/
    /// `region_export_tiles` do -- "two views of one rect", per §4.5.1's own
    /// note -- and the same `seed`/`sea` from this world's state rather than
    /// from `opts`, which is `region_export_tiles`' documented convention and
    /// is required here rather than merely consistent: the amplified field is
    /// real elevation in the **parent's** `[0, 1]` space, so the sea level it
    /// is classified against must be the one it was authored under.
    ///
    /// `tile_size` is the reference's `refSize` -- the long edge of the
    /// resampled world in cells. `<= 0` takes its default of **1024**, the
    /// reference's own `+document.getElementById('refSize').value||1024`, not
    /// `region_export_tiles`' 512. `detail_freq`/`detail_amp` `<= 0` take
    /// `AmplifyOpts`' 1.0/0.14, so a caller with nothing to say passes zeros.
    ///
    /// # This runs on a worker thread, and that is why it is primitives in,
    /// `bool` out
    ///
    /// [`WorldGen::landmark_run`]'s rule, and it applies here for the same
    /// measured reason: it is an amplify plus a full substrate inversion plus
    /// climate, flow and civilisation over the new grid -- seconds of work
    /// that froze the window when `landmark_run` shipped synchronous
    /// (2026-09-01, *"the new point of interest function seems to make the
    /// program freeze"*). `engine_bridge.gd` therefore calls this from a
    /// `Thread`, and **without gdext's `experimental-threads` feature every
    /// `Dictionary`/`Array`/`GString` operation goes through
    /// `sys::get_binding()`, whose `ensure_main_thread()` panics off the main
    /// thread**. The first cut of this function took a `Dictionary` of options
    /// and returned one -- which is that panic twice per press. The refusal
    /// reason is stashed in `WorldGen::region_error` (a plain `String`) and
    /// read back by [`Self::region_new_world_error`] on the main thread.
    ///
    /// On **every** `false` return the current world is left exactly as it
    /// was, because all five refusals (the finalize lock, no marquee, no
    /// world, a source shorter than its own grid, and a sub-4-cell resample)
    /// are checked before the first line that mutates anything.
    ///
    /// # What is here, and what is in the engine
    ///
    /// The pipeline half — resample, derive the new params, reconstruct the
    /// substrate, refresh climate — is
    /// [`cartalith_engine::region_export::region_as_new_world`], which carries
    /// the whole parity account of the reference's handler and is tested
    /// there. What is left here is the half `LARGE_ITEM_RULINGS.md` said must
    /// not be folded into GUI work: the **`WorldGen` state**.
    ///
    /// * [`WorldGen::release_world`] — the reference's `allocate()` (which
    ///   zeroes every field) plus `invalidateFieldCaches()` plus the civ clear.
    ///   Without it a knowledge link filed against the parent world's
    ///   `settlement:3` would resolve against the resampled world's by
    ///   coincidence. This bullet used to add "and the **only** path on this
    ///   struct that clears `vault.store.links`/`snapshots`", which was true
    ///   and was the bug: `import_heightmap` replaces a world without calling
    ///   it, so those two clears moved into `absorb` on 2026-09-03 and this
    ///   path now gets them twice, harmlessly.
    /// * [`WorldGen::absorb`] — the new `WorldState`, a civ layer over it, and
    ///   fresh `sculpt`/`icons`/`civ_tools`/`paint`/`labels`/`infra` at the new
    ///   dimensions, plus the `undo`/`redo`/`ledger`/`stages`/`civ_dirty`/
    ///   `bake.finalized`/`landmark_store` resets a world replacement needs.
    ///
    /// # Ordering: `release_world` runs last, unlike `generate_sized`
    ///
    /// `generate_sized` releases *before* generating, because
    /// `MEMORY_OPTIMIZATION_SCOPE.md` R1 measured +209.96 MiB from holding two
    /// worlds at once. It cannot be done here: this path **reads the outgoing
    /// `field`** to build the incoming one, so releasing first would drop its
    /// own input. The cost is one extra `WorldState` at the new dimensions,
    /// live across `region_as_new_world` — smaller than R1's number, since a
    /// region resample is one world rather than two full-resolution ones, and
    /// not avoidable without amplifying twice.
    ///
    /// This is also why a failure here cannot strand the user: nothing is
    /// released until every refusal has returned, so an `ok: false` leaves the
    /// parent world untouched and still selected.
    #[func]
    fn region_new_world(
        &mut self,
        tile_size: i64,
        ridged: bool,
        detail_freq: f64,
        detail_amp: f64,
    ) -> bool {
        // Not a `Dictionary` and not a `GString` -- see the threading note
        // above. Every early return goes through this, so there is one place
        // the reason is recorded and none where it is forgotten.
        let refuse = |s: &mut Self, reason: String| -> bool {
            s.region_error = reason;
            false
        };
        self.region_error.clear();

        // Replacing the world outright is the most complete form of
        // `Mutation::Generation`, the same reason `generate_sized` and
        // `load_save` both check it here.
        if let Err(msg) = self.bake.check(cartalith_engine::bake::Mutation::Generation) {
            return refuse(self, msg.to_string());
        }
        let Some(region) = self.infra.as_ref().and_then(|i| i.region) else {
            return refuse(self, "No region is selected. Drag the Region select marquee first.".into());
        };
        let (gw, gh) = (self.gw.max(0) as usize, self.gh.max(0) as usize);
        let short = match self.source.as_ref() {
            // `run_bake`'s own guard, for its reason: `amplify_region`
            // asserts on a short source, and an assert here is a process kill.
            Some(WorldSource::Generated(ws)) => gw == 0 || gh == 0 || ws.field.len() < gw * gh,
            Some(WorldSource::Loaded(save)) => gw == 0 || gh == 0 || save.fields.heightmap.len() < gw * gh,
            None => true,
        };
        if short {
            return refuse(self, "No world is loaded.".into());
        }

        let tile_size = if tile_size > 0 { tile_size as usize } else { 1024 };
        let amplify = cartalith_terrain::amplify::AmplifyOpts {
            seed: self.seed,
            detail_freq: if detail_freq > 0.0 { detail_freq } else { 1.0 },
            detail_amp: if detail_amp > 0.0 { detail_amp } else { 0.14 },
            sea: self.sea_level,
            ridged,
            // `z_base`/`zoom_detail_k` steer `add_zoom_detail`, which
            // `amplify_region` ignores -- `region_export_tiles`' own note.
            ..cartalith_terrain::amplify::AmplifyOpts::default()
        };
        // Re-borrowed after the guard above rather than held across it: the
        // `refuse` closure needs `&mut self`, so the field cannot stay
        // borrowed while the refusals run.
        let field: &[f32] = match self.source.as_ref() {
            Some(WorldSource::Generated(ws)) => &ws.field,
            Some(WorldSource::Loaded(save)) => &save.fields.heightmap,
            // Unreachable: `short` above is `true` for `None` and has already
            // returned. Answered rather than `unreachable!()`, because a panic
            // here crosses the gdext boundary.
            None => return refuse(self, "No world is loaded.".into()),
        };
        // `recompute_params()` -- the parent's own params with `world`
        // pinned, which is the reference's behaviour (its handler never
        // touches `state.world`, so a resample inherits the parent's wrap
        // geometry rather than being re-decided by whatever the dial says
        // now). `region_as_new_world` derives `gw`/`gh`/`map_width_km`/
        // `sea_level` from the resample and hands back what it used, so
        // there is no way for this caller to index the new state with the
        // parent's stride.
        let base = self.recompute_params();
        let (p, ws) = match cartalith_engine::region_export::region_as_new_world(
            field, gw, gh, &region, tile_size, &base, &amplify,
        ) {
            Ok(v) => v,
            Err((rw, rh)) => {
                let reason = format!(
                    "That selection resamples to {rw} x {rh} cells; the smallest world this engine builds is {n} x {n}. Select a less extreme aspect, or raise the tile size.",
                    n = cartalith_engine::region_export::MIN_REGION_WORLD_AXIS,
                );
                return refuse(self, reason);
            }
        };

        // Every refusal has returned; from here the world is replaced.
        // `release_world` runs *after* the resample and inference rather than
        // before, unlike `generate_sized`: those two produce a world from
        // parameters alone, this one reads the outgoing `field` to build the
        // incoming one, so dropping the source first would drop its input.
        let seed = self.seed;
        self.release_world();
        // `ORIGIN_REGION`: this field is an amplified crop of the *parent's*,
        // and `seed` above is the parent's own, so its parameter tuple is
        // unusually likely to land on one a generated world already has an
        // atlas under -- see `bake_bridge::world_key_signature`.
        self.absorb(ws, &p, seed, crate::bake_bridge::ORIGIN_REGION);
        true
    }

    /// Why the last [`Self::region_new_world`] returned `false`, or an empty
    /// string when it returned `true`.
    ///
    /// The main-thread half of that call, exactly as `landmark_last_run()` is
    /// `landmark_run()`'s: `region_new_world` runs on a worker thread and may
    /// not construct a `GString`, so it leaves a plain `String` behind and the
    /// shell reads it back here once the thread has been joined.
    ///
    /// Empty is the honest "nothing refused", not a stand-in for a message the
    /// engine failed to produce -- every `false` path sets a reason before
    /// returning.
    #[func]
    fn region_new_world_error(&self) -> GString {
        GString::from(self.region_error.as_str())
    }
}
