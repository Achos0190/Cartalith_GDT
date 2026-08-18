extends Workspace
class_name RenderWorkspace

## RENDER domain (§3): terrain appearance groups; right dock preview & quality.
##
## `render.rs` carries a full `TerrainAppearance` structure that is settable
## in Rust and bound to nothing -- its own doc comment says so. Until it is
## bound, quality tier (Preferences) is the only live control.

func _build() -> void:
	## §4.5.1's three global tools -- RENDER owns no tool of its own (no
	## §4.5.x section names one), so this is the whole TOOLS block.
	DccWidgets.tools_block(self, app, app.tool_group)
	_not_built("Terrain appearance",
		"render.rs's TerrainAppearance is real but unbound to Godot; until it is, Preferences ▸ Render quality is the live control.")
