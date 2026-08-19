extends Workspace
class_name RenderWorkspace

## Terrain appearance groups; right dock preview & quality -- formerly the
## standalone RENDER rail domain (§3).
##
## `render.rs` carries a full `TerrainAppearance` structure that is settable
## in Rust and bound to nothing -- its own doc comment says so. Until it is
## bound, quality tier (Preferences) is the only live control.
##
## **Domain merge (2026-08-20, owner instruction: "And render into carto").**
## RENDER no longer has its own rail button; this one-subject class is now
## composed into `CartographyWorkspace` (`cartography_workspace.gd`'s own
## `_render` field) as a nested `VBoxContainer` appended after CARTO's own
## three categories. `_nested` (set true by `cartography_workspace.gd` before
## `setup()`) skips this file's own TOOLS block -- CARTO's own already covers
## the three global tools (plus Icon/Label), and RENDER never had a
## domain-specific tool of its own to add to it. This also directly resolves
## the CA-01/RN-01 ambiguity `GUI_GAP_REGISTER.md` §8.6 flagged: CARTO and
## RENDER were both proposing to own the same future `set_appearance()`
## binding; merging the domains removes the split.
var _nested := false

func _build() -> void:
	## §4.5.1's three global tools -- RENDER owns no tool of its own (no
	## §4.5.x section names one), so this was always the whole TOOLS block.
	## Skipped when nested -- see this file's own class doc.
	if not _nested:
		DccWidgets.tools_block(self, app, app.tool_group)
	_not_built("Terrain appearance",
		"render.rs's TerrainAppearance is real but unbound to Godot; until it is, Preferences ▸ Render quality is the live control.")
