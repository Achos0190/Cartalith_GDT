extends Workspace
class_name RenderWorkspace

## RENDER domain (§3): terrain appearance groups; right dock preview & quality.
##
## `render.rs` carries a full `TerrainAppearance` structure that is settable
## in Rust and bound to nothing -- its own doc comment says so. Until it is
## bound, quality tier (Preferences) is the only live control.

func _build() -> void:
	_not_built("Terrain appearance",
		"render.rs's TerrainAppearance is real but unbound to Godot; until it is, Preferences ▸ Render quality is the live control.")
