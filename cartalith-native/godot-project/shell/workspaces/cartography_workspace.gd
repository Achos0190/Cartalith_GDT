extends Workspace
class_name CartographyWorkspace

## CARTO domain (§7): layer list, layer properties, colour ramp, stop editor.
##
## Presentation only. No control here may alter heightmap, climate, hydrology,
## biome classification, settlements, routes or seed, and none marks a
## generation stage stale -- which is why nothing in this file touches
## `bridge.mark_dirty()`.
##
## §7 specifies three panes and a ten-row layer stack. The renderer can honour
## five of those rows today; the rest (hand-drawn hillshade, colour relief as
## separate layers, opacity, blend mode, the ramp editor) need `render.rs`'s
## `TerrainAppearance` bound to Godot first.

## The five layers the shell can actually toggle, in §7's own draw order:
## topmost first, matching how the layer list reads.
const LIVE_LAYERS: Array = [
	{"id": "settlements", "label": "Settlements", "on": true},
	{"id": "roads", "label": "Ways & routes", "on": true},
	{"id": "sea_routes", "label": "Sea routes", "on": true},
	{"id": "provinces", "label": "Political — provinces", "on": false},
	{"id": "territory", "label": "Political — territory", "on": false},
]

func _build() -> void:
	var cat := DccWidgets.category(self, "Layers", categories, true)
	var body := DccWidgets.section(cat, "Visible layers")
	for layer in LIVE_LAYERS:
		DccWidgets.toggle(body, layer.label, layer.on,
			func(on: bool): app.viewport.set_layer_visible(layer.id, on))
	DccWidgets.note(body,
		"Terrain, hillshade and colour relief are one baked raster today, so "
		+ "they toggle together with the map itself rather than as separate rows.")

	var props := DccWidgets.category(self, "Layer properties", categories)
	DccWidgets.note(DccWidgets.section(props, "Fill · light · opacity"),
		"Spec §7's ramp picker, stop editor and lighting rig read and write "
		+ "render.rs's TerrainAppearance, which is implemented and settable in "
		+ "Rust but bound to no GDExtension method. Nothing here can be honest "
		+ "until that binding lands (UNIFIED_TOOL_PLAN.md milestone F).")

	var annot := DccWidgets.category(self, "Labels & annotation", categories)
	DccWidgets.note(DccWidgets.section(annot, "Authoring"),
		"cartalith-civ/src/labels.rs implements arc layout, hit boxes and drag "
		+ "handles for map labels. The design gives labels a layer row but no way "
		+ "to create one — see STRANDED_TOOLS.md row 13.")
