//! The reference's `erode()` op — `PARITY_AUDIT.md` §23 F11.
//!
//! `dropletKernel` → `erodeThermal` → clamp → `isostaticRebound` (reference
//! line 3894). All three kernels are ported in `cartalith-erosion` with
//! golden-parity coverage; nothing ever assembled them, and
//! `cartalith-engine` imports `isostatic_rebound` alone.
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

use crate::WorldGen;
use godot::prelude::*;

#[godot_api(secondary)]
impl WorldGen {}
