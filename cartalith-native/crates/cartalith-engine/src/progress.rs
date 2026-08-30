//! Process-global generation-stage progress counter (the owner's locked
//! Android spec: "Generator: one Generate button + staged progress readout
//! (10 stages), stale-from note after edits").
//!
//! ## Why this is a `static`, not a method on `WorldGen`
//!
//! `EngineBridge.generate()` (`godot-project/shell/engine_bridge.gd`) runs
//! `generate_terrain` on a background `Thread` that holds `&mut WorldGen` --
//! via gdext's `Gd<T>::bind_mut()` -- for the call's whole duration. Any
//! `#[func]` reached from the main thread while that runs fails its own
//! `bind()` (`engine_bridge.gd`'s multi-GPU block documents measuring 360
//! such panics from one open menu during a single generation). So the
//! counter cannot live on `WorldGen`; it lives here instead, and
//! `cartalith-godot::GenerationProgress` -- a second, unrelated, stateless
//! `RefCounted` -- is free to read it every frame from the main thread while
//! the worker thread runs, exactly the way `_params_cache` in
//! `engine_bridge.gd` already serves reads for the same reason.
//!
//! Plain `std::sync::atomic`, no new dependency: an `AtomicU32` for the
//! current stage index and an `AtomicU64` run token, both `Relaxed` --
//! nothing here needs to synchronise with any other write, only to become
//! visible to a poller within a frame or two.
//!
//! ## The ten stages, and the banner -> stage mapping this module's callers
//! encode
//!
//! Derived by reading `generate_terrain_inner` end to end (`cartalith-engine/
//! src/lib.rs`, function starts at the line the doc comment on that function
//! names), not by counting its `// ---- name ----` banners --
//! `cartalith-porting-discipline`'s own rule, and load-bearing here: several
//! banners span more than one of these stages, and one stage's real work has
//! no banner of its own at all. `world_workspace.gd`'s own `STAGES` const
//! (`godot-project/shell/workspaces/world_workspace.gd`, ~line 64) is the ten
//! names' and the `needs`/`produces`/`gap` prose's source of truth; this
//! module only has to stay in the same order, which
//! `progress::tests::stage_names_match_the_declared_count` pins.
//!
//!  0. **Planet** -- no code of its own in `generate_terrain_inner`
//!     (`planet.g`/`axial_tilt_deg`/`rotation_hours` are read later, by
//!     craters and ocean currents, not computed here). Ticked through at
//!     `begin_run()`, which already resets the stage to this index.
//!  1. **Extent & scale** -- likewise no code of its own: `gw`/`gh` are call
//!     arguments, not computed. (This stage's own "land/sea split" prose
//!     names the one real per-generate computation that could be attributed
//!     to it -- `apply_world_structure_sea_level`, which only runs under a
//!     World-Structure archetype and only AFTER Tectonics/Volcanism have
//!     already produced the field its histogram reads. Bumping this stage
//!     there would walk the counter backward from 3/4 to 1, which this
//!     readout must never do -- see `advance`'s own monotonic contract -- so
//!     that correction is left running silently under whichever stage is
//!     current when it executes, and is disclosed here rather than silently
//!     reordered.) Ticks through immediately after Planet.
//!  2. **World structure** -- the `generate_continentality_field`/
//!     `world_structure_arg` block, immediately before the
//!     "buildTectonicSubstrate" banner (a no-op when `world_structure.
//!     enabled` is off, the default -- ticks through with nothing to show
//!     either).
//!  3. **Tectonics** -- from the "buildTectonicSubstrate" banner's real
//!     tectonics work (GPU-substrate setup, warp, `build_plates`/
//!     `assign_plates`, `compute_stress`, flexure, `base_field`,
//!     `age_field`, heterogeneity, `resistance_field`, the World-Structure
//!     orogeny pass) through the "height -> normalize" banner
//!     (`compute_height`/`normalize_field`) -- i.e. that whole banner's span
//!     once World Structure's own slice (stage 2, above) is subtracted from
//!     the front of it.
//!  4. **Volcanism & impacts** -- the "volcanism + craters" banner.
//!     `stamp_craters` always runs regardless of `volc_count`, so this stage
//!     always does real work, every generate.
//!  5. **Erosion** -- the light stream-power carve inside `carveRiverValleys`
//!     (`if p.carve_rivers`, on by default), or, when `carve_rivers` is off,
//!     ticked through alongside Hydrology and Climate right after the
//!     direct `flow_discharge` computation (see stage 7's note: with no
//!     carve pass there is nothing left for any of the three to compute
//!     beyond what already ran). The standalone `passes.*` block (velocity,
//!     glacial, coastal, hillslope, `evolveCoupled`, sediment fill, tidal
//!     flats -- all off by default) is this same stage's own work too, but
//!     needs no separate bump: by the time it can run, the counter is
//!     already at or past this index (`advance` is monotonic).
//!  6. **Hydrology** -- `build_channels`/`strahler_from_receivers`/
//!     `trace_river_polylines`/the channel-carve loop inside the carve
//!     block -- the `channels`/`stream_order`/`river_mask`/`river_floor`
//!     fields `WorldState` actually stores. (The earlier `flow_area` local,
//!     computed before Climate's own first pass, is *not* this stage's
//!     product: it is discarded, never written to `WorldState` -- see stage
//!     7's note on why it gets no bump of its own either.)
//!  7. **Climate** -- the post-carve refresh (temperature, rainfall,
//!     moisture correctors, ocean currents) that is climate's real, FINAL,
//!     stored state on the default `carve_rivers=true` path. **Deliberately
//!     not bumped at its first appearance**, well before Erosion: `STAGES[5]`
//!     ("Erosion") in `world_workspace.gd` states its own dependency as
//!     "needs -- 04 Tectonics, 08 Climate", i.e. the reference genuinely
//!     computes a priming climate pass before erosion so the carve has
//!     rainfall to read, then re-derives climate afterward once the valleys
//!     are cut. A linear ten-stage counter cannot show both without moving
//!     backward from 7 to 5/6, and this readout never does that -- so the
//!     priming pass runs silently under whichever stage is already current
//!     (Volcanism, usually), and only the refresh that produces the value
//!     actually kept counts as "Climate ran". Its rainfall/temperature ARE
//!     real by the time this stage's bump lands.
//!  8. **Ecology & biomes** / 9. **Resources & soils** -- no code in
//!     `generate_terrain_inner` at all, matching `STAGES[8]`/`STAGES[9]`'s
//!     own gap notes in `world_workspace.gd` ("Not parameterised ... no
//!     dials exist in cartalith-engine"). Both tick through together right
//!     before this function returns, via `finish()`.
//!
//!     **Caveat worth a human's attention, not silently folded into the gap
//!     note**: biome classification and soil/resource potentials genuinely
//!     ARE computed on every `generate()` -- in `cartalith-godot::
//!     compute_civilisation` (`build_biome_raster`, `build_soil_fertility`,
//!     `build_resource_potentials`), called right after
//!     `generate_terrain_inner` returns. That is real engine work, just
//!     outside the WORLD domain's ten-stage pipeline this readout
//!     represents -- `world_workspace.gd`'s own top-of-file doc comment
//!     draws exactly this line for Settlements/Infrastructure/Politics, and
//!     the same reasoning applies to Biomes/Resources' *civ*-layer
//!     computation. So "no dials" (what the gap note says) is accurate, and
//!     "no engine equivalent at all" (what a careless reading of it could
//!     suggest) would not be. This module does not reach into
//!     `compute_civilisation` to bump these two stages for real work done
//!     there, on the same "no code in this function" boundary the task that
//!     added this module was scoped to.
//!
//! Separately, and independent of the above: `05 Volcanism & impacts` DOES
//! carry real editable dials in this port (`params.rs`'s `"volcanism"`
//! group: `volc.count`, `volc.age`, `volc.provinces`, `crater.count`,
//! `crater.age`) -- only `10 Resources & soils` has neither a `groups` nor a
//! `keys` entry in `world_workspace.gd`'s `STAGES` table and is genuinely
//! non-editable. A claim that Volcanism is also non-editable does not match
//! the params table and should not be treated as ground truth.

use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};

pub const PLANET: usize = 0;
pub const EXTENT_SCALE: usize = 1;
pub const WORLD_STRUCTURE: usize = 2;
pub const TECTONICS: usize = 3;
pub const VOLCANISM: usize = 4;
pub const EROSION: usize = 5;
pub const HYDROLOGY: usize = 6;
pub const CLIMATE: usize = 7;
pub const ECOLOGY_BIOMES: usize = 8;
pub const RESOURCES_SOILS: usize = 9;

/// `world_workspace.gd`'s own `STAGES` names, in the same order -- kept here
/// as the Rust-side copy of that vocabulary so `cartalith-godot::
/// GenerationProgress` can hand a human-readable name back to GDScript
/// without either side hardcoding the other's table.
pub const STAGE_NAMES: [&str; 10] = [
    "Planet",
    "Extent & scale",
    "World structure",
    "Tectonics",
    "Volcanism & impacts",
    "Erosion",
    "Hydrology",
    "Climate",
    "Ecology & biomes",
    "Resources & soils",
];

pub const STAGE_COUNT: usize = STAGE_NAMES.len();

static RUN_TOKEN: AtomicU64 = AtomicU64::new(0);
/// `0..STAGE_COUNT` while a run is in progress; `STAGE_COUNT` once `finish()`
/// has landed for the run `RUN_TOKEN` currently names.
static STAGE: AtomicU32 = AtomicU32::new(0);

/// Call once, at the very start of a `generate_terrain` run. Bumps the run
/// token (so a poller can tell this run's readings from the previous run's
/// last ones apart -- see this module's own doc comment) and resets the
/// stage to [`PLANET`].
pub fn begin_run() {
    RUN_TOKEN.fetch_add(1, Ordering::Relaxed);
    STAGE.store(PLANET as u32, Ordering::Relaxed);
}

/// Move the counter forward to `stage`. A no-op if the counter is already at
/// or past `stage` -- `fetch_max`, not `store` -- so calling this from more
/// than one branch of an `if`/`else` (or safety-net calls from a later block
/// that may or may not have been reached first) can never walk the readout
/// backward, whichever branch a given run actually takes.
pub fn advance(stage: usize) {
    debug_assert!(stage < STAGE_COUNT, "stage {stage} out of range (0..{STAGE_COUNT})");
    STAGE.fetch_max(stage as u32, Ordering::Relaxed);
}

/// Marks the run complete: one past the last valid stage index, so a reader
/// can tell "on the last stage" (`STAGE_COUNT - 1`) apart from "finished".
pub fn finish() {
    STAGE.store(STAGE_COUNT as u32, Ordering::Relaxed);
}

/// `(run_token, stage)` -- `stage` is `0..STAGE_COUNT` mid-run and
/// `STAGE_COUNT` once `finish()` has landed. `run_token` is `0` before the
/// first `generate_terrain` call this process has ever made, and increases
/// by exactly one on every call after that.
pub fn snapshot() -> (u64, usize) {
    (RUN_TOKEN.load(Ordering::Relaxed), STAGE.load(Ordering::Relaxed) as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stage_names_match_the_declared_count() {
        assert_eq!(STAGE_NAMES.len(), STAGE_COUNT);
        assert_eq!(STAGE_NAMES[PLANET], "Planet");
        assert_eq!(STAGE_NAMES[EROSION], "Erosion");
        assert_eq!(STAGE_NAMES[RESOURCES_SOILS], "Resources & soils");
    }

    #[test]
    fn advance_never_moves_the_counter_backward() {
        begin_run();
        advance(CLIMATE);
        advance(EROSION); // an out-of-order call, e.g. from a stray branch
        let (_, stage) = snapshot();
        assert_eq!(stage, CLIMATE, "a lower stage must never overwrite a higher one");
    }

    #[test]
    fn begin_run_resets_the_stage_and_bumps_the_token() {
        begin_run();
        advance(RESOURCES_SOILS);
        finish();
        let (t1, s1) = snapshot();
        assert_eq!(s1, STAGE_COUNT);
        begin_run();
        let (t2, s2) = snapshot();
        assert!(t2 > t1, "the token must increase so a poller can tell the runs apart");
        assert_eq!(s2, PLANET);
    }

    #[test]
    fn finish_is_distinguishable_from_the_last_real_stage() {
        begin_run();
        advance(RESOURCES_SOILS);
        let (_, mid) = snapshot();
        assert_eq!(mid, RESOURCES_SOILS);
        finish();
        let (_, done) = snapshot();
        assert_ne!(done, RESOURCES_SOILS);
        assert_eq!(done, STAGE_COUNT);
    }
}
