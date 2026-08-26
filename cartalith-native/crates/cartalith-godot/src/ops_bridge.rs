//! Ported-and-unexposed engine capability — `PARITY_AUDIT.md` §23 F13.
//!
//! Each of these exists in `cartalith-*`, is tested there, and had no `#[func]`
//! and no caller: `extract_region_as_world`, `apply_force_lake`,
//! `estimate_regional_density_km2`, `arc_label_line_width`, the icon brush
//! (`icon_brush_rule` / `icon_brush_stamp`) and the asset-library operations
//! (`to_library_json`, `drop_collection`, `apply_library_file_with_items`,
//! `referenced_files`, `slot_paths`, `filled_count`).
//!
//! Its own file rather than `lib.rs` so this work cannot collide with the
//! other §23 agents editing that file — the same `#[godot_api(secondary)]`
//! split `project_bridge.rs` and `vault_bridge.rs` already use.

use crate::WorldGen;
use godot::prelude::*;

#[godot_api(secondary)]
impl WorldGen {}
