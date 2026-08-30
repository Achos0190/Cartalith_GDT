//! A stateless, separately-instantiated read-only window onto
//! `cartalith_engine::progress`'s process-global generation-stage counter
//! (`cartalith-engine/src/progress.rs`) -- the owner's locked Android spec's
//! staged generation readout ("Generator: one Generate button + staged
//! progress readout (10 stages)").
//!
//! **Deliberately not a method on `WorldGen`.** `EngineBridge.generate()`
//! (`engine_bridge.gd`) runs `generate_terrain` on a background `Thread`
//! that holds `&mut WorldGen` -- via gdext's `Gd<T>::bind_mut()` -- for the
//! call's whole duration; any `#[func]` reached from the main thread
//! meanwhile fails its own `bind()` (see `engine_bridge.gd`'s multi-GPU
//! block for a measured account of exactly that failure: 360 panics from
//! one open menu during a single generation). `GenerationProgress` carries
//! no state of its own and never touches `WorldGen`, so
//! `GenerationProgress.new()` can be created and polled every frame from
//! the main thread while the worker thread runs, reading straight through
//! to the plain `std::sync::atomic` counter in `cartalith-engine`.
//!
//! No `project.godot` edit needed to register this class: a GDExtension
//! class is exported through the `.gdextension` manifest, the same as
//! `WorldGen`/`WalkingSkeleton` above it in `lib.rs` -- unlike a GDScript
//! `class_name`, which does need the editor's import pass (`CLAUDE.md`'s own
//! "Constraints" section).

use godot::classes::{IRefCounted, RefCounted};
use godot::prelude::*;

#[derive(GodotClass)]
#[class(base=RefCounted)]
struct GenerationProgress {
    base: Base<RefCounted>,
}

#[godot_api]
impl IRefCounted for GenerationProgress {
    fn init(base: Base<RefCounted>) -> Self {
        Self { base }
    }
}

#[godot_api]
impl GenerationProgress {
    /// `{run_token, stage, stage_count, stage_name, done}` --
    /// `cartalith_engine::progress::snapshot()` read straight through.
    ///
    /// `stage` is clamped to `0..stage_count-1` even once the run is
    /// `done`, so a caller that only wants "which row to show as current"
    /// never has to special-case the one-past-the-end sentinel
    /// `cartalith_engine::progress::finish()` stores internally -- `done`
    /// carries that fact separately. `run_token` is `0` before any
    /// `generate_terrain` call this process has made, and increases by
    /// exactly one on every call after that -- compare it to the last value
    /// seen to tell a fresh run's reading from a stale one apart (this is
    /// the "run token so a stale reading from a previous run cannot be
    /// mistaken for this one" the spec calls for).
    #[func]
    fn snapshot(&self) -> VarDictionary {
        let (token, stage) = cartalith_engine::progress::snapshot();
        let count = cartalith_engine::progress::STAGE_COUNT;
        let done = stage >= count;
        let shown = stage.min(count - 1);
        vdict! {
            "run_token" => token as i64,
            "stage" => shown as i64,
            "stage_count" => count as i64,
            "stage_name" => cartalith_engine::progress::STAGE_NAMES[shown],
            "done" => done,
        }
    }
}
