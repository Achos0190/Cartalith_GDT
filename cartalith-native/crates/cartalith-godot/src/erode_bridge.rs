//! The reference's `erode()` op — `PARITY_AUDIT.md` §23 F11 — and the
//! force-lake post-pass — §23 F13.
//!
//! `dropletKernel` → `erodeThermal` → clamp → `isostaticRebound` (reference
//! line 3898; `erodeFinish` at 3892). All three kernels are ported in
//! `cartalith-erosion` with golden-parity coverage; nothing ever assembled
//! them, and `cartalith-engine` imports `isostatic_rebound` alone.
//!
//! **This is an op, not a generation stage.** In the reference it is a button
//! that mutates the existing `field` after generation — `state.erosion`'s
//! parameters are op parameters, and `generate()` does not run it. Putting it
//! here rather than in `generate_terrain` keeps that true, and keeps every
//! existing golden and every `generate()`-derived hash bit-identical.
//!
//! Its own file rather than `lib.rs` so this work cannot collide with the
//! other §23 agents editing that file — the same `#[godot_api(secondary)]`
//! split `project_bridge.rs` and `vault_bridge.rs` already use.
//!
//! # Why the op itself is in `cartalith-engine`, not here
//!
//! `cartalith-godot` **does not depend on `cartalith-erosion`**. Only
//! `cartalith-engine` does (`grep -l cartalith-erosion crates/*/Cargo.toml`
//! returns exactly two files: the erosion crate itself and the engine), and
//! `cartalith-engine`'s import of it is a private `use`, not a `pub use` —
//! `lib.rs:125` brings `isostatic_rebound` and friends into the engine's own
//! namespace and re-exports none of them. So `droplet_kernel`,
//! `erode_thermal`, `isostatic_rebound` and `DropletParams` are not merely
//! uncalled from this crate: they are **unnameable** from it. Rust 2018+
//! resolves `cartalith_erosion::…` only against a direct dependency.
//!
//! Rather than open this crate's dependency list, the assembly lives in
//! [`cartalith_engine::erode_op`] — which is the better home regardless:
//! *"cartalith-engine orchestrates; it does not compute"*, and
//! `ARCHITECTURE.md` wants this crate thin. Read that module for the op's
//! semantics, its reference provenance and the climate-coupling fallback.
//! What is left here is the `Variant` conversion, the undo/ledger push and
//! the staleness refresh — the three things the engine deliberately cannot do.
//!
//! # F13 · `apply_force_lake` — built, below
//!
//! `cartalith-civ` **is** a direct dependency, so the other half of this task
//! is here in full. [`WorldGen::apply_force_lake`] is the consumer
//! `WaterState::lake_mask`'s own doc comment has been waiting for since
//! `UNIFIED_TOOL_PLAN.md` milestone C wrote it: *"Pass this mask to it after
//! classifying, and a painted lake is a lake."*

use crate::{undo, CivData, WorldGen, WorldSource};
use cartalith_engine::erode_op::{erode_op as run_erode_op, ErodeOpts};
use cartalith_engine::staleness::PipelineStage;
use godot::prelude::*;

/// Reads the op's `Dictionary` argument, falling back to
/// [`ErodeOpts::default`] — `state.erosion`'s own literal, reference HTML line
/// 2268 — for every key the caller omits. Same shape as `lib.rs`'s
/// `slice_params_from`.
///
/// No clamping here: [`cartalith_engine::erode_op::erode_op`] runs
/// [`ErodeOpts::sanitized`] itself, so the guard lives with the code that
/// knows why it is needed rather than being restated at each call site.
fn erode_opts_from(opts: &VarDictionary) -> ErodeOpts {
    let d = ErodeOpts::default();
    let i_of = |k: &str, dv: i32| {
        opts.get(k).and_then(|v| v.try_to::<i64>().ok()).unwrap_or(dv as i64).clamp(i32::MIN as i64, i32::MAX as i64) as i32
    };
    let f_of = |k: &str, dv: f64| opts.get(k).and_then(|v| v.try_to::<f64>().ok()).unwrap_or(dv);
    ErodeOpts {
        droplets: i_of("droplets", d.droplets),
        inertia: f_of("inertia", d.inertia),
        capacity: f_of("capacity", d.capacity),
        min_slope: f_of("min_slope", d.min_slope),
        deposit: f_of("deposit", d.deposit),
        erode: f_of("erode", d.erode),
        evaporate: f_of("evaporate", d.evaporate),
        gravity: f_of("gravity", d.gravity),
        max_lifetime: i_of("max_lifetime", d.max_lifetime),
        init_speed: f_of("init_speed", d.init_speed),
        init_water: f_of("init_water", d.init_water),
        radius: i_of("radius", d.radius),
        talus: f_of("talus", d.talus),
        thermal_passes: i_of("thermal_passes", d.thermal_passes),
    }
}

/// `{"ok": false, "reason": …}` with every count zeroed — [`WorldGen::erode_op`]'s
/// single refusal shape, so GDScript has one thing to check.
fn refuse_erode(reason: &str) -> VarDictionary {
    vdict! {
        "ok" => false, "reason" => reason,
        "cells_changed" => 0i64, "cells_lowered" => 0i64, "cells_raised" => 0i64,
        "climate_coupled" => false, "ms" => 0.0f64,
    }
}

/// [`cartalith_civ::apply_force_lake`] plus the two counts a caller needs to
/// tell "nothing was painted" from "everything painted was already water".
///
/// Free function rather than a method so the whole behaviour is exercised by
/// this file's own `#[cfg(test)]` block with no Godot runtime involved — the
/// same isolation `sculpt_bridge.rs`'s module doc argues for.
///
/// Returns `(newly_forced, lake_cells_after)`. `newly_forced` counts only
/// cells the call actually changed: a stamp dropped on a basin that already
/// pooled into a lake by itself is a no-op, and reporting it as work done
/// would make the UI claim a change that did not happen.
fn force_lakes(classification: &mut [u8], mask: &[u8]) -> (i64, i64) {
    let before = classification
        .iter()
        .zip(mask.iter())
        .filter(|&(&c, &m)| m != 0 && c != 2)
        .count() as i64;
    cartalith_civ::apply_force_lake(classification, mask);
    let lakes = classification.iter().filter(|&&c| c == 2).count() as i64;
    (before, lakes)
}

/// `{"ok": false, "reason": …}` with the two counts zeroed — one refusal
/// shape, so GDScript has exactly one thing to check.
fn refuse(reason: &str) -> VarDictionary {
    vdict! { "ok" => false, "reason" => reason, "forced" => 0i64, "lake_cells" => 0i64 }
}

#[godot_api(secondary)]
impl WorldGen {
    /// The reference's `#erodeBtn` (`erode()`, reference HTML line 3898):
    /// particle hydraulic erosion over the finished surface, then thermal
    /// talus relaxation, the `[0,1]` clamp and isostatic rebound of the
    /// unloaded crust (`erodeFinish`, line 3892). `PARITY_AUDIT.md` §23 F11.
    ///
    /// A thin caller of [`cartalith_engine::erode_op::erode_op`] — read that
    /// module for the algorithm, the reference provenance and the
    /// climate-coupling fallback.
    ///
    /// An **opt-in** op, exactly as in the reference: it never runs during
    /// `generate()`, so a default world is bit-identical with or without this
    /// binding existing, and no `WorldParams` field, `world_key` input,
    /// golden fixture or `save_round_trip` assertion is touched.
    ///
    /// # Parameters
    ///
    /// `opts` carries the **op's own** parameters, not world parameters.
    /// Every key is optional and falls back to `state.erosion`'s literal
    /// (reference line 2268):
    ///
    /// | key | default | reference control |
    /// |---|---|---|
    /// | `droplets` | `60000` | `#drops` (slider 0-100, x1500) |
    /// | `erode` | `0.35` | `#estr` (/100) |
    /// | `deposit` | `0.30` | `#edep` (/100) |
    /// | `thermal_passes` | `8` | `#ethr` |
    /// | `talus` | `0.012` | `#etal` (/1000) |
    /// | `inertia` | `0.05` | none |
    /// | `capacity` | `4.0` | none |
    /// | `min_slope` | `0.01` | none |
    /// | `evaporate` | `0.02` | none |
    /// | `gravity` | `4.0` | none |
    /// | `max_lifetime` | `30` | none |
    /// | `init_speed` | `1.0` | none |
    /// | `init_water` | `1.0` | none |
    /// | `radius` | `3` | none |
    ///
    /// The nine with no reference control are exactly the nine the
    /// reference's own Erosion panel never exposed; they are accepted here as
    /// a superset, at the engine's own default.
    ///
    /// `g`, `ck` and `seed` are deliberately **not** accepted: `dropletParams`
    /// reads them from `state.planet.g`, `state.stream.climateK` and
    /// `state.tect.seed`, so the op takes them from this world's own live
    /// parameters and a caller cannot erode with a gravity or a seed that
    /// disagrees with the world being eroded.
    ///
    /// # Returns
    ///
    /// `ok` (bool), `cells_changed` / `cells_lowered` / `cells_raised` (int),
    /// `climate_coupled` (bool — false when this world carries no rainfall
    /// and the droplets spawned uniformly), `recomputed` / `still_stale`
    /// (`PackedStringArray`), `ms` (float), and `reason` (String, only when
    /// `ok` is false).
    ///
    /// # What it re-runs, and what it does not
    ///
    /// Flow and climate **are** recomputed, through the same staleness-graph
    /// path [`Self::sculpt_commit`] and [`Self::carve_fjords`] use — this
    /// port's `computeFlow(true)` + `refreshClimate()`, which is
    /// `erodeFinish`'s own tail. The **vector river network** (`channels`,
    /// `stream_order`, the carve-time `river_mask`) is not re-derived: the
    /// same documented ceiling `carve_fjords` carries, since re-deriving the
    /// vector network is not part of what `refresh_climate` does.
    #[func]
    fn erode_op(&mut self, opts: VarDictionary) -> VarDictionary {
        // A finalized world's baked atlas is the authoritative surface; an
        // edit under it would show in the live view and vanish at the next
        // zoom (`sculpt_commit`'s own reasoning, reference `applyFinalizedUI`).
        if let Err(msg) = self.bake.check(cartalith_engine::bake::Mutation::HeightEdit) {
            return refuse_erode(&msg);
        }
        let t0 = std::time::Instant::now();
        let eo = erode_opts_from(&opts);
        // The live world parameters -- `p.gw`/`gh`/`world`/`tect.seed`/
        // `tect.blur_r`/`planet.g`/`stream.climate_k` are exactly what the op
        // reads from `state`. `recompute_params()` is the same accessor
        // `mark_and_recompute` uses, so the op and the refresh that follows it
        // cannot disagree about which world they are working on. Taken before
        // the `source` borrow, since it needs `&self` whole.
        let p = self.recompute_params();
        let n = p.gw * p.gh;

        let Some(WorldSource::Generated(ws)) = self.source.as_mut() else {
            return refuse_erode(
                "Erode needs a generated world; a loaded save has no pipeline graph to refresh flow and climate through afterwards.",
            );
        };
        if n == 0 || ws.field.len() != n {
            return refuse_erode("No world.");
        }
        // Global heightmap undo at the same point `carve_fjords` pushes it:
        // below every refusal, so no undo step is spent on a call that changed
        // nothing. The reference's own `#erodeBtn` handler opens with
        // `pushUndo()` and has no such refusals to avoid.
        self.undo.push("Erode (droplet)", &ws.field);
        self.ledger.record(
            "height",
            "Erode (droplet)",
            format!("{} x {}", self.gw, self.gh),
            undo::EntryKind::HeightSnapshot,
        );
        let s = run_erode_op(ws, &p, &eo);

        // Droplets can start anywhere on the map and isostatic rebound is a
        // whole-field Gaussian blur, so this is never tile-local -- the whole
        // graph is marked, which is also all a whole-field recompute could act
        // on.
        let all_tiles = 0..self.stages.tile_count();
        let (recomputed, still_stale) = self.mark_and_recompute(PipelineStage::Height, all_tiles, "erode");
        vdict! {
            "ok" => true,
            "reason" => "",
            "cells_changed" => s.cells_changed as i64,
            "cells_lowered" => s.cells_lowered as i64,
            "cells_raised" => s.cells_raised as i64,
            "climate_coupled" => s.climate_coupled,
            "recomputed" => &recomputed,
            "still_stale" => &still_stale,
            "ms" => t0.elapsed().as_secs_f64() * 1000.0,
        }
    }

    /// `buildWaterBodies`' `opts.forceLake` (reference HTML lines 5808-5809),
    /// applied to the live classification — `PARITY_AUDIT.md` §23 F13.
    ///
    /// The Sculpt editor's **Lake** stamp accumulates a `lake_mask`
    /// (`cartalith_engine::sculpt_commit::WaterState::lake_mask`, built by
    /// every `sculpt_commit` that deposits one). Nothing consumed it:
    /// `sculpt_commit` writes `river_mask`/`river_floor` back onto the
    /// `WorldState` and stops there, and `CivData::water_bodies` — the
    /// classification the Settlement tool, the route planner, the trade and
    /// military layers and the Journey Planner all read — was computed once,
    /// during `compute_civilisation`, from the terrain alone. So a painted
    /// lake was terrain that happened to be lower, and nothing in the port
    /// treated it as water.
    ///
    /// This is the missing edge. It reclassifies every cell the mask marks as
    /// a lake (`2`), unconditionally — that is the reference's own semantic:
    /// a user-deposited lake is a lake whether or not its floor ends up below
    /// sea level or its basin catches enough rain to pool.
    ///
    /// **An opt-in op, like [`Self::carve_fjords`].** It never runs during
    /// `generate()`, changes no height, marks nothing stale, and touches
    /// nothing a golden test or a `world_key` reads.
    ///
    /// Returns `ok` (bool), `forced` (int, cells this call actually changed —
    /// `0` with `ok: true` means every painted cell was already water),
    /// `lake_cells` (int, lake cells in the classification afterwards) and
    /// `reason` (String, only when `ok` is false).
    ///
    /// # What it does not reach
    ///
    /// The Biome-paint editor captured its own land-only gate as an
    /// `Arc<[u8]>` copy of this array at generation time
    /// (`paint_bridge::PaintEditor::water_mask`, deliberately cached — see
    /// that field's own doc comment for the 417 ms it saves). That copy is
    /// **not** refreshed here, because `PaintEditor` exposes no setter for it
    /// and this task owns neither file; until one exists the brush will still
    /// paint a forced lake as land. Recorded rather than hidden.
    ///
    /// The forcing is also lost on the next full `compute_civilisation`,
    /// which rebuilds `water_bodies` from `build_water_bodies` — same ceiling
    /// every manual civ edit already has.
    #[func]
    fn apply_force_lake(&mut self) -> VarDictionary {
        let Some(sculpt) = self.sculpt.as_ref() else {
            return refuse("Force lake needs a generated world with a Sculpt session; a loaded save has no draft.");
        };
        let Some(mask) = sculpt.water.lake_mask.as_ref() else {
            return refuse("No lake has been stamped yet — commit a Lake stamp in Sculpt first.");
        };
        // `self.sculpt` and `self.civ` are disjoint fields, so the mask
        // borrow stays live across the mutable one. `CivData` is a private
        // type of the crate root; this module is a descendant of it, which
        // is the whole reason the `#[godot_api(secondary)]` split works.
        let Some(civ): Option<&mut CivData> = self.civ.as_mut() else {
            return refuse("This world has no civilisation layer, so there is no water-body classification to force.");
        };
        if civ.water_bodies.is_empty() {
            return refuse("This world's water-body classification is empty.");
        }
        let (forced, lake_cells) = force_lakes(&mut civ.water_bodies, mask);
        vdict! { "ok" => true, "reason" => "", "forced" => forced, "lake_cells" => lake_cells }
    }
}

#[cfg(test)]
mod tests {
    use super::force_lakes;

    /// The reference's own `if(force[i]) out[i]=2` — ocean and land alike
    /// become lake, and the count reports only the cells that moved.
    #[test]
    fn forcing_reclassifies_land_and_ocean_and_counts_only_changes() {
        //            land ocean lake land
        let mut cls = [0u8, 1, 2, 0];
        let mask = [1u8, 1, 1, 0];
        let (forced, lakes) = force_lakes(&mut cls, &mask);
        assert_eq!(cls, [2, 2, 2, 0], "every masked cell is a lake, whatever it was");
        assert_eq!(forced, 2, "cell 2 was already a lake; cell 3 is unmasked");
        assert_eq!(lakes, 3);
    }

    /// Idempotent: pressing the button twice is not two changes.
    #[test]
    fn a_second_call_forces_nothing_new() {
        let mut cls = [0u8, 0, 1, 1];
        let mask = [1u8, 0, 1, 0];
        let (first, _) = force_lakes(&mut cls, &mask);
        let snapshot = cls;
        let (second, lakes) = force_lakes(&mut cls, &mask);
        assert_eq!(first, 2);
        assert_eq!(second, 0, "nothing left to force");
        assert_eq!(cls, snapshot, "and nothing left to change");
        assert_eq!(lakes, 2);
    }

    /// `apply_force_lake`'s own documented tolerance: a mask shorter than the
    /// classification simply does not force the tail, exactly as the
    /// reference's `if(force[i])` on a short `Uint8Array` does not.
    #[test]
    fn a_short_mask_leaves_the_tail_alone() {
        let mut cls = [0u8, 0, 0, 0];
        let mask = [1u8, 1];
        let (forced, lakes) = force_lakes(&mut cls, &mask);
        assert_eq!(cls, [2, 2, 0, 0]);
        assert_eq!((forced, lakes), (2, 2));
    }

    /// An empty mask is the state before the first Lake stamp is committed,
    /// and it must be a clean no-op rather than a panic.
    #[test]
    fn an_empty_mask_is_a_no_op() {
        let mut cls = [0u8, 1, 2];
        let (forced, lakes) = force_lakes(&mut cls, &[]);
        assert_eq!(cls, [0, 1, 2]);
        assert_eq!((forced, lakes), (0, 1));
    }
}
