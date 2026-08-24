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
## the same rule `journey_planner_view.gd` follows for `jp_options()`.
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

const FLOW_FX_SCRIPT := preload("res://shell/wind_fx_layer.gd")

var bridge: EngineBridge
var host: ViewportHost

var _list: VBoxContainer
var _legend: VBoxContainer
var _rows: Dictionary = {}   ## view id -> its Button

## Hotkey badges 1-8 (`DCC_SHELL_SPEC.md` §10: "grouped rows with hotkey
## badges: SURFACE (Relief 1, Biome 2, Political 3), TERRAIN FIELDS
## (Elevation 4, Slope 5, Flow accumulation 6), CLIMATE (Temperature 7,
## Rainfall 8)"). That grouping does not exist in this popover's own data:
## `LAYER_GROUPS` (`sample_bridge.rs`) is the *reference's* verbatim
## Base/Climate/Tectonics/Hydrology/Surface/Civilization order (this file's
## own header comment above explains why it stays that way), which has no
## "Relief" row at all (the closest is `off`, "No overlay (base map)") and
## puts Political under Civilization, last, not third. Re-sorting rows
## client-side to chase the spec's naming would scatter hotkeys 1-8 across
## non-adjacent groups with no visual grouping to match -- a bigger, riskier
## change than this badge itself. Badging the first 8 rows in their real,
## already-built order instead (`DCC_CONTROL_INDEX.md`'s own tolerance for
## "uncertain" mappings). Noted here and in `GUI_GAP_REGISTER.md`.
##
## **Only *available* rows are badged**, which is why `rebuild()` counts them
## itself rather than badging `LAYER_GROUPS`' first eight entries outright.
## The unavailable rows are the eleven permanent engine gaps (`GAP_LAYERS`,
## `sample_bridge.rs`) -- they are disabled on every world that will ever
## exist, so a digit spent on one is a digit that does nothing, forever.
## Counting positionally did exactly that: when the seven new Climate views
## landed (Wind, Ocean currents, ...), Köppen -- a gap row -- shifted into
## slot 4, and pressing `4` silently no-opped from then on. Skipping
## unavailable rows keeps all eight digits live, and keeps them stable
## against a future row landing in the middle of a group.
const HOTKEY_COUNT := 8
const HOTKEY_ACTIONS: Array[String] = [
	"layers_hotkey_1", "layers_hotkey_2", "layers_hotkey_3", "layers_hotkey_4",
	"layers_hotkey_5", "layers_hotkey_6", "layers_hotkey_7", "layers_hotkey_8",
]
var _hotkey_ids: Array = []   ## index 0-7 -> the row id badged with that digit.

func setup(b: EngineBridge, h: ViewportHost) -> void:
	bridge = b
	host = h
	_register_hotkeys()
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
		"Settlement, road, sea-route and town-layout visibility live in " +
		"Cartography ▸ Layers on the rail; way types are Cartography ▸ Roads & " +
		"routes and the two political layers are Cartography ▸ Political " +
		"display. All of those are vector overlays drawn from world data, not " +
		"field rasters, and they toggle rather than replace one another. Town " +
		"layouts draw themselves once the map spans under 24 km.")

	bridge.generation_finished.connect(func(_ok: bool): if visible: rebuild())
	bridge.world_loaded.connect(func(): if visible: rebuild())

	_attach_flow_fx()

## The Wind and Ocean-currents rows are the only two field views the reference
## also *animates* (`#windFxCanvas`, reference HTML lines 2113-2209): particle
## streaks advected along the flow field, over the static raster. `app.gd`
## builds exactly one of this popover and this runs from `setup()`, so the one
## overlay node is created once here rather than on each pick -- it is idle
## (invisible, holding no field) until `host.debug_view()` actually reads back
## one of those two, which it polls for itself. See `wind_fx_layer.gd`.
##
## Parented under `host.overlay`, not this popover: the streaks belong in the
## map's own zoomed/panned coordinate space, and `map_overlay.gd` is already
## the node that carries it (and publishes the letterbox fit rect the
## particles project through). Attached from here because `set_debug_layer`
## lives on `ViewportHost`, which is owner-reserved for concurrent work --
## this popover is the only other place that knows a field view was picked.
func _attach_flow_fx() -> void:
	if host == null or host.overlay == null:
		return
	## `preload`, not the `WindFxLayer` global class name: a global name only
	## resolves once the editor has rescanned and written
	## `.godot/global_script_class_cache.cfg`, so a fresh clone (or any
	## editor-less run, which is how this port's capture harnesses drive the
	## shell) would fail to parse this file. `viewport_host.gd`'s own
	## `OVERLAY_SCRIPT` preload is here for the same reason.
	var fx: Control = FLOW_FX_SCRIPT.new()
	fx.name = "WindFxLayer"
	fx.setup(bridge, host)
	host.overlay.add_child(fx)

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
	_hotkey_ids.clear()

	var groups := bridge.debug_layers()
	if groups.is_empty():
		DccWidgets.note(_list,
			"No field views: this build's engine has no debug_layers() binding.")
		_refresh_legend([])
		return

	var current := host.debug_view()
	var row_i := 0   ## Running count of *available* rows across every group --
		## `HOTKEY_ACTIONS`' own doc comment on why this badges the first 8 in
		## build order rather than the spec's own SURFACE/TERRAIN FIELDS/
		## CLIMATE grouping, and why a disabled row never consumes a digit.
	for g in groups:
		var group: Dictionary = g
		var body := DccWidgets.section(_list, String(group["group"]))
		for it in group["items"]:
			var item: Dictionary = it
			var hotkey := -1
			if bool(item["available"]) and row_i < HOTKEY_COUNT:
				hotkey = row_i
				_hotkey_ids.append(String(item["id"]))
				row_i += 1
			_rows[String(item["id"])] = _row(body, item, current, hotkey)
	_refresh_legend(_legend_for(current, groups))

func _row(parent: Control, item: Dictionary, current: String, hotkey: int = -1) -> Button:
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
	var font_color := DccTheme.c("text" if available else "text_ghost")
	if id == current:
		b.add_theme_stylebox_override("normal", DccTheme.active_row())
		b.add_theme_color_override("font_color", DccTheme.c("accent"))
		font_color = DccTheme.c("accent")
	if available:
		b.pressed.connect(_on_pick.bind(id))
	parent.add_child(b)
	if hotkey >= 0:
		_add_hotkey_badge(b, hotkey, font_color, id == current)
	return b

## The mockup's own badge markup (`design/Cartalith DCC Shell.dc.html`, the
## Layers popover's Relief/Biome/Political/... rows): `font:9px 'IBM Plex
## Mono'`, `border:1px solid currentColor`, `padding:0 4px`, opacity .75 on
## the active row and .55 otherwise -- reproduced directly rather than
## invented, down to the opacity split. `currentColor` becomes `font_color`
## (the same colour the row's own label was just set to) at reduced alpha,
## since Godot StyleBox/Label colours have no "inherit the text colour"
## concept.
##
## A child of the row `Button`, not a sibling in a wrapping `HBoxContainer`:
## a `Button` is not itself a layout container, but it *is* a plain
## `Control`, so anchoring a child directly to its right edge reaches the
## same visual result -- flush against the row's own right edge, inside its
## already-existing background/hover/active stylebox -- without splitting
## the row's hit box in two. Anchors are computed from `get_minimum_size()`
## rather than baked via `set_anchors_preset()`, the same trap
## `ViewportHost._chrome()`'s own doc comment names: a preset call bakes
## offsets from the control's size *at that moment*, which is zero before
## the button has ever been laid out.
func _add_hotkey_badge(button: Button, hotkey: int, font_color: Color, is_active: bool) -> void:
	var badge := Label.new()
	badge.text = str(hotkey + 1)
	badge.add_theme_font_override("font", DccTheme.mono())
	badge.add_theme_font_size_override("font_size", DccTheme.FS_MICRO)
	var badge_color := font_color
	badge_color.a = 0.75 if is_active else 0.55
	badge.add_theme_color_override("font_color", badge_color)
	var sb := StyleBoxFlat.new()
	sb.bg_color = Color(0, 0, 0, 0)
	sb.border_color = badge_color
	sb.set_border_width_all(1)
	sb.content_margin_left = 4
	sb.content_margin_right = 4
	badge.add_theme_stylebox_override("normal", sb)
	badge.mouse_filter = Control.MOUSE_FILTER_IGNORE
	button.add_child(badge)

	badge.anchor_left = 1.0
	badge.anchor_right = 1.0
	badge.anchor_top = 0.5
	badge.anchor_bottom = 0.5
	badge.grow_horizontal = Control.GROW_DIRECTION_BEGIN
	var sz := badge.get_minimum_size()
	badge.offset_right = -10.0
	badge.offset_left = -10.0 - sz.x
	badge.offset_top = -sz.y * 0.5
	badge.offset_bottom = sz.y * 0.5

## Runs once -- `app.gd` builds exactly one `LayersPopover` and never frees
## it, so `setup()` (its one-time init point, matching every other method in
## this file) only ever calls this once per session. Guarded on `has_action`
## anyway, cheaply, rather than trusting that.
##
## Registered at runtime rather than declared in `project.godot`: this
## popover is the only place these eight digits mean anything, and `_input()`
## below already scopes them to "popover visibly open," so a project-wide
## `[input]` entry would only invite a second, unintended consumer.
func _register_hotkeys() -> void:
	for i in range(HOTKEY_ACTIONS.size()):
		var action := HOTKEY_ACTIONS[i]
		if InputMap.has_action(action):
			continue
		InputMap.add_action(action)
		var ev := InputEventKey.new()
		ev.physical_keycode = KEY_1 + i
		InputMap.action_add_event(action, ev)

## Hotkey badges 1-8 pick the same row their click already does. Scoped to
## "popover visibly open" by the `visible` guard below -- a `PopupPanel`
## stays in the scene tree while hidden (this node is never freed), so
## without that check the digit keys would fire from anywhere in the shell,
## which is not what a popover-local hotkey means.
func _input(event: InputEvent) -> void:
	if not visible:
		return
	for i in range(_hotkey_ids.size()):
		if event.is_action_pressed(HOTKEY_ACTIONS[i]):
			var id: String = _hotkey_ids[i]
			var row: Button = _rows.get(id)
			if row != null and not row.disabled:
				_on_pick(id)
			get_viewport().set_input_as_handled()
			return

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
