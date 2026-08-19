extends PopupPanel
class_name LayersPopover

## The map canvas's Layers popover (`DCC_SHELL_SPEC.md` §9's layers button),
## ported from the reference's own canvas popover (`buildLayersPopover`,
## reference HTML line 13657) rather than invented.
##
## **What the layers button used to do.** It emitted `layers_button_pressed`
## and `app.gd` answered by selecting the Cartography domain, whose left dock
## has a "Visible layers" section. That was a stand-in for this: a button
## labelled Layers that jumped the whole workspace rather than opening
## anything. The stand-in is replaced, not extended -- but nothing it reached
## is removed: `cartography_workspace.gd`'s own toggles and
## `ViewportHost.set_layer_visible()` are untouched, still on the rail, and
## the note at the foot of this popover points at them.
##
## **The grouping is the reference's, verbatim.** `LAYER_GROUPS` in
## `sample_bridge.rs` keeps the original's Base / Climate / Tectonics /
## Hydrology / Surface / Civilization headings and their order, restricted to
## the views this port can actually draw from state generation already
## retains, plus four it adds for Sample fields the reference never had a
## view for (elevation, slope, aspect, resistance -- each says so in its own
## hint). The engine owns that table; this file never restates it, which is
## the same rule `journey_planner_window.gd` follows for `jp_options()`.
##
## **Rows that cannot work are disabled, not hidden.** A view whose one input
## this world lacks (Strahler order without river extraction, biomes/terrain/
## control on a loaded save) comes back `available: false` from the engine
## and is drawn greyed with its reason in the tooltip -- §6's own "fields
## from stale stages read —" rule, applied to a picker.
##
## Like the reference's, this popover stays open across picks: it holds many
## independent choices plus an opacity slider, and closing on every click
## would make comparing two views needlessly tedious. Click outside or press
## Escape to dismiss.

var bridge: EngineBridge
var host: ViewportHost

var _list: VBoxContainer
var _legend: VBoxContainer
var _rows: Dictionary = {}   ## view id -> its Button

func setup(b: EngineBridge, h: ViewportHost) -> void:
	bridge = b
	host = h
	add_theme_stylebox_override("panel",
		DccTheme.panel("panel", {"left": 1, "right": 1, "top": 1, "bottom": 1}))

	var outer := VBoxContainer.new()
	outer.add_theme_constant_override("separation", 0)
	outer.custom_minimum_size = Vector2(228, 0)
	add_child(outer)

	var scroll := ScrollContainer.new()
	scroll.custom_minimum_size = Vector2(228, 420)
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	outer.add_child(scroll)

	_list = VBoxContainer.new()
	_list.add_theme_constant_override("separation", 0)
	_list.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	scroll.add_child(_list)

	outer.add_child(DccTheme.rule())

	var foot := VBoxContainer.new()
	foot.add_theme_constant_override("separation", 2)
	var pad := MarginContainer.new()
	pad.add_theme_constant_override("margin_left", 10)
	pad.add_theme_constant_override("margin_right", 10)
	pad.add_theme_constant_override("margin_top", 6)
	pad.add_theme_constant_override("margin_bottom", 8)
	pad.add_child(foot)
	outer.add_child(pad)

	_legend = VBoxContainer.new()
	_legend.add_theme_constant_override("separation", 1)
	foot.add_child(_legend)

	DccWidgets.slider(foot, "Opacity", 0.0, 100.0, 1.0,
		host.debug_opacity() * 100.0, "%",
		func(v: float): host.set_debug_opacity(v / 100.0),
		"Blends the active field raster over the base map, so terrain reads " +
		"through it. The reference's own #dbgOpacity.")

	DccWidgets.note(foot,
		"Settlement, road, sea-route, territory and province visibility live " +
		"in Cartography ▸ Layers on the rail -- those are vector overlays " +
		"drawn from world data, not field rasters, and they toggle rather " +
		"than replace one another.")

	bridge.generation_finished.connect(func(_ok: bool): if visible: rebuild())
	bridge.world_loaded.connect(func(): if visible: rebuild())

## Anchored under the viewport's own layers button rather than at a guessed
## corner offset -- the button moves with `set_safe_insets()` on phone.
func open() -> void:
	rebuild()
	var r := host.layers_button_rect()
	popup(Rect2i(Vector2i(r.position.x, r.position.y + r.size.y + 4), Vector2i(228, 0)))

func rebuild() -> void:
	for child in _list.get_children():
		_list.remove_child(child)
		child.queue_free()
	_rows.clear()

	var groups := bridge.debug_layers()
	if groups.is_empty():
		DccWidgets.note(_list,
			"No field views: this build's engine has no debug_layers() binding.")
		_refresh_legend([])
		return

	var current := host.debug_view()
	for g in groups:
		var group: Dictionary = g
		var body := DccWidgets.section(_list, String(group["group"]))
		for it in group["items"]:
			var item: Dictionary = it
			_rows[String(item["id"])] = _row(body, item, current)
	_refresh_legend(_legend_for(current, groups))

func _row(parent: Control, item: Dictionary, current: String) -> Button:
	var id := String(item["id"])
	var available: bool = bool(item["available"])
	var b := Button.new()
	b.text = String(item["label"])
	b.flat = true
	b.focus_mode = Control.FOCUS_NONE
	b.alignment = HORIZONTAL_ALIGNMENT_LEFT
	b.custom_minimum_size.y = 22
	b.add_theme_font_size_override("font_size", DccTheme.FS_SMALL)
	b.tooltip_text = String(item["hint"]) if available else \
		String(item["hint"]) + "\n\nNot available for this world."
	b.disabled = not available
	b.add_theme_color_override("font_color",
		DccTheme.c("text" if available else "text_ghost"))
	b.add_theme_color_override("font_disabled_color", DccTheme.c("text_ghost"))
	if id == current:
		b.add_theme_stylebox_override("normal", DccTheme.active_row())
		b.add_theme_color_override("font_color", DccTheme.c("accent"))
	if available:
		b.pressed.connect(_on_pick.bind(id))
	parent.add_child(b)
	return b

func _on_pick(id: String) -> void:
	host.set_debug_layer(id)
	## Rebuilt rather than re-styled in place: `set_debug_layer` is allowed
	## to refuse (a view the engine could not draw falls back to "off"), and
	## reading `debug_view()` back is the only honest way to know which row
	## should be lit.
	rebuild()

func _legend_for(view: String, groups: Array) -> Array:
	for g in groups:
		var group: Dictionary = g
		for it in group["items"]:
			var item: Dictionary = it
			if String(item["id"]) == view:
				return item["legend"]
	return []

func _refresh_legend(entries: Array) -> void:
	for child in _legend.get_children():
		_legend.remove_child(child)
		child.queue_free()
	for e in entries:
		var entry: Dictionary = e
		var row := HBoxContainer.new()
		row.add_theme_constant_override("separation", 6)
		var sw := ColorRect.new()
		sw.color = Color8(int(entry["r"]), int(entry["g"]), int(entry["b"]))
		sw.custom_minimum_size = Vector2(11, 11)
		sw.size_flags_vertical = Control.SIZE_SHRINK_CENTER
		row.add_child(sw)
		row.add_child(DccTheme.label(String(entry["label"]), "text_dim", DccTheme.FS_MICRO))
		_legend.add_child(row)
