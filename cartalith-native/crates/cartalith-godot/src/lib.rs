//! Boundary layer between Godot and the engine crates (ARCHITECTURE.md).
//!
//! Phase 0 walking skeleton: proves a gdext-backed class loads in the Godot
//! editor and survives a Windows/Android export. No engine crate is wired in
//! yet — that starts in Phase 1 (MVP_SCOPE.md).

use godot::classes::{INode, Node};
use godot::init::{ExtensionLibrary, gdextension};
use godot::prelude::*;

struct CartalithExtension;

#[gdextension]
unsafe impl ExtensionLibrary for CartalithExtension {}

/// Placeholder GDExtension class for the Phase 0 walking skeleton.
#[derive(GodotClass)]
#[class(base=Node)]
struct WalkingSkeleton {
    base: Base<Node>,
}

#[godot_api]
impl INode for WalkingSkeleton {
    fn init(base: Base<Node>) -> Self {
        Self { base }
    }

    fn ready(&mut self) {
        godot_print!("cartalith-godot: WalkingSkeleton ready (Phase 0)");
    }
}

#[godot_api]
impl WalkingSkeleton {
    /// Round-trips a value through Rust so GDScript can confirm the
    /// extension is actually loaded, not just present on disk.
    #[func]
    fn ping(&self) -> GString {
        GString::from("cartalith-godot: pong")
    }
}
