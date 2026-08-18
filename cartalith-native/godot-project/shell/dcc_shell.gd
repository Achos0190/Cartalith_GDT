extends Control
class_name DccShell

## The DCC editor frame (`DCC_SHELL_SPEC.md` §1-§3, §6, §7, §9, §11).
##
## Six regions in DOM order plus the two bars that bracket them, built in code
## rather than in a `.tscn` so that the geometry table in §1 is readable as a
## table here, and so five workspace modules can attach without five people
## editing one scene file.
##
## This script owns the *frame* only: region sizes, dock collapse, which
## workspace is active, and the status bar. It owns no world state and calls no
## engine method -- `EngineBridge` does that, and the workspaces read it. The
## load-bearing rule from `UI_SHELL_DESIGN.md`: the top bar is about the
## program, the map is about the world.

signal workspace_changed(id: String)
signal tool_changed(tool_id: String)

# -- Workspaces (§3) ----------------------------------------------------------
#
# Five domains on the rail. Generate / Simulate / Render / View are *not*
# menus in this shell -- that is the structural change this revision makes, and
# the reason the menu bar below is seven program menus and nothing else.

const DOMAINS: Array = [
	{"id": "world", "label": "World", "icon": "domain_world",
		"subtitle": "Terrain, hydrology, climate and ecology"},
	{"id": "civilization", "label": "Civilization", "icon": "domain_civ",
		"subtitle": "Settlements, factions, provinces and trade"},
	{"id": "infrastructure", "label": "Infrastructure", "icon": "domain_infra",
		"subtitle": "Roads, sea routes, bridges and journeys"},
	{"id": "cartography", "label": "Cartography", "icon": "domain_carto",
		"subtitle": "Layers, styles, labels and annotation"},
	{"id": "render", "label": "Render", "icon": "domain_render",
		"subtitle": "Lighting, materials, export and 3D"},
]

# -- Region handles -----------------------------------------------------------
#
# Everything a workspace module needs is reachable from here. Workspaces never
# reach past these into the frame's own containers.

var menu_bar_row: HBoxContainer
var tool_options_row: HBoxContainer
var rail_column: VBoxContainer
var left_dock: PanelContainer
var left_dock_title: Label
var left_dock_body: VBoxContainer      ## Workspace panels attach here.
var viewport_area: Control
var viewport_content: Control          ## The map surface; overlays are children.
var right_dock: PanelContainer
var right_dock_body: VBoxContainer
var timeline_row: HBoxContainer
var status_row: HBoxContainer

var _domain_buttons: Dictionary = {}   ## id -> Button
var _active_domain := "world"
var _left_collapsed := false
var _right_collapsed := false
var _left_width := float(DccTheme.W_LEFT_DOCK)
var _right_width := float(DccTheme.W_RIGHT_DOCK)
var _status_labels: Dictionary = {}    ## slot -> Label
var _collapse_buttons: Dictionary = {} ## "left"/"right" -> Button, so the chevron can flip
var _dock_readouts: Dictionary = {}    ## "left"/"right" -> the collapsed-state Label
var _workspace_panels: Dictionary = {} ## domain id -> Control
var _touch := false

# -- Build --------------------------------------------------------------------

func _ready() -> void:
	_touch = DisplayServer.is_touchscreen_available() and OS.has_feature("mobile")
	set_anchors_preset(Control.PRESET_FULL_RECT)
	var ground := ColorRect.new()
	ground.color = DccTheme.c("bg")
	ground.set_anchors_preset(Control.PRESET_FULL_RECT)
	ground.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(ground)

	var shell := VBoxContainer.new()
	shell.set_anchors_preset(Control.PRESET_FULL_RECT)
	shell.add_theme_constant_override("separation", 0)
	add_child(shell)

	shell.add_child(_build_menu_bar())
	shell.add_child(_build_tool_options_bar())

	var main_row := HBoxContainer.new()
	main_row.size_flags_vertical = Control.SIZE_EXPAND_FILL
	main_row.add_theme_constant_override("separation", 0)
	shell.add_child(main_row)

	main_row.add_child(_build_rail())
	main_row.add_child(_build_left_dock())
	main_row.add_child(_build_viewport())
	main_row.add_child(_build_right_dock())

	shell.add_child(_build_timeline())
	shell.add_child(_build_status_bar())

	_select_domain(_active_domain)

func _scaled(px: int) -> int:
	## §13: tablet and phone scale every fixed height, with a 44 px floor on
	## anything tappable. Windows is pointer-first and takes the raw value.
	if not _touch:
		return px
	return maxi(44, int(round(px * DccTheme.TOUCH_SCALE)))

# -- §2 Menu bar: program scope only ------------------------------------------

func _build_menu_bar() -> Control:
	var bar := PanelContainer.new()
	bar.custom_minimum_size.y = _scaled(DccTheme.H_MENU_BAR)
	bar.add_theme_stylebox_override("panel",
		DccTheme.panel("panel", {"bottom": 1}))

	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 0)
	var pad := MarginContainer.new()
	pad.add_theme_constant_override("margin_left", 14)
	pad.add_theme_constant_override("margin_right", 14)
	pad.add_child(row)
	bar.add_child(pad)

	var wordmark := DccTheme.label("C A R T A L I T H", "text_bright", DccTheme.FS_MENU)
	wordmark.custom_minimum_size.x = 128
	row.add_child(wordmark)

	menu_bar_row = HBoxContainer.new()
	menu_bar_row.add_theme_constant_override("separation", 0)
	row.add_child(menu_bar_row)

	row.add_child(DccTheme.spacer())

	## The readout cluster: world, pass state, and the three cost meters. §11
	## keeps these in the menu bar because they describe the *program's* load,
	## not the world's content.
	for slot in ["world", "pass", "cpu", "gpu", "mem"]:
		var l := DccTheme.label("", "text_faint", DccTheme.FS_READOUT)
		_status_labels["top_" + slot] = l
		row.add_child(l)
		var gap := Control.new()
		gap.custom_minimum_size.x = 22
		row.add_child(gap)
	return bar

## Register a program menu. The caller fills the PopupMenu through `on_built`,
## so this file never has to know what File contains.
func add_menu(title: String, on_built: Callable) -> MenuButton:
	var mb := MenuButton.new()
	mb.text = title
	mb.flat = true
	mb.focus_mode = Control.FOCUS_NONE
	mb.add_theme_font_size_override("font_size", DccTheme.FS_MENU)
	mb.add_theme_color_override("font_color", DccTheme.c("text_dim"))
	mb.add_theme_color_override("font_hover_color", DccTheme.c("text_bright"))
	mb.add_theme_stylebox_override("normal", DccTheme.inset(11, 9, 11, 9))
	mb.add_theme_stylebox_override("hover", DccTheme.inset(11, 9, 11, 9))
	mb.add_theme_stylebox_override("pressed", DccTheme.active_row())
	menu_bar_row.add_child(mb)
	var popup := mb.get_popup()
	style_popup(popup)
	on_built.call(popup)
	return mb

func style_popup(popup: PopupMenu) -> void:
	popup.add_theme_stylebox_override("panel", DccTheme.panel("raised",
		{"left": 1, "right": 1, "top": 1, "bottom": 1}))
	popup.add_theme_color_override("font_color", DccTheme.c("text"))
	popup.add_theme_color_override("font_disabled_color", DccTheme.c("text_ghost"))
	popup.add_theme_color_override("font_accelerator_color", DccTheme.c("text_faint"))
	popup.add_theme_font_size_override("font_size", DccTheme.FS_MENU)

# -- §4 Tool options bar ------------------------------------------------------

func _build_tool_options_bar() -> Control:
	var bar := PanelContainer.new()
	bar.custom_minimum_size.y = _scaled(DccTheme.H_TOOL_OPTIONS)
	bar.add_theme_stylebox_override("panel",
		DccTheme.panel("panel_alt", {"bottom": 1}))
	tool_options_row = HBoxContainer.new()
	tool_options_row.add_theme_constant_override("separation", 14)
	var pad := MarginContainer.new()
	pad.add_theme_constant_override("margin_left", 14)
	pad.add_theme_constant_override("margin_right", 14)
	pad.add_child(tool_options_row)
	bar.add_child(pad)
	return bar

## Replace the bar's contents. §4: it holds the active tool's frequently-changed
## values and its commit/discard, and never a control belonging to another tool
## -- so switching tools clears it rather than appending to it.
func set_tool_options(build: Callable) -> void:
	for child in tool_options_row.get_children():
		tool_options_row.remove_child(child)
		child.queue_free()
	build.call(tool_options_row)

# -- §3 Domain rail -----------------------------------------------------------

func _build_rail() -> Control:
	var rail := PanelContainer.new()
	rail.custom_minimum_size.x = _scaled(DccTheme.W_RAIL_COLLAPSED)
	rail.add_theme_stylebox_override("panel", DccTheme.panel("panel", {"right": 1}))
	rail_column = VBoxContainer.new()
	rail_column.add_theme_constant_override("separation", 2)
	var pad := MarginContainer.new()
	pad.add_theme_constant_override("margin_top", 6)
	pad.add_child(rail_column)
	rail.add_child(pad)

	for d in DOMAINS:
		var b := Button.new()
		b.tooltip_text = "%s -- %s" % [d.label, d.subtitle]
		b.flat = true
		b.focus_mode = Control.FOCUS_NONE
		b.custom_minimum_size.y = _scaled(36)
		b.icon = DccIcons.get_icon(d.icon, 16 if not _touch else 20)
		b.expand_icon = false
		b.add_theme_stylebox_override("normal", DccTheme.empty())
		b.add_theme_stylebox_override("hover", DccTheme.flat(DccTheme.c("line_soft")))
		b.pressed.connect(_select_domain.bind(d.id))
		_domain_buttons[d.id] = b
		rail_column.add_child(b)
	return rail

func _select_domain(id: String) -> void:
	_active_domain = id
	for key in _domain_buttons:
		var b: Button = _domain_buttons[key]
		var on: bool = key == id
		b.modulate = DccTheme.c("accent") if on else DccTheme.c("text_faint")
		b.add_theme_stylebox_override("normal",
			DccTheme.active_row(false) if on else DccTheme.empty())
	for key in _workspace_panels:
		(_workspace_panels[key] as Control).visible = key == id
	for d in DOMAINS:
		if d.id == id:
			left_dock_title.text = String(d.label).to_upper()
			break
	workspace_changed.emit(id)

## A workspace module calls this once, from `_ready`, with the panel it wants in
## the left dock. Panels are built up front and hidden, not rebuilt on every
## switch -- §3 requires each domain's L2 open/closed state to persist.
func register_workspace(id: String, panel: Control) -> void:
	panel.visible = id == _active_domain
	panel.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_workspace_panels[id] = panel
	left_dock_body.add_child(panel)

func active_domain() -> String:
	return _active_domain

# -- §6 Docks -----------------------------------------------------------------

func _build_left_dock() -> Control:
	left_dock = PanelContainer.new()
	left_dock.custom_minimum_size.x = _left_width
	left_dock.add_theme_stylebox_override("panel", DccTheme.panel("panel", {"right": 1}))

	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 0)
	left_dock.add_child(col)

	var head := HBoxContainer.new()
	head.custom_minimum_size.y = 26
	left_dock_title = DccTheme.header("WORLD")
	head.add_child(left_dock_title)
	head.add_child(DccTheme.spacer())
	head.add_child(_collapse_button(true))
	var head_pad := MarginContainer.new()
	head_pad.add_theme_constant_override("margin_left", 12)
	head_pad.add_theme_constant_override("margin_right", 6)
	head_pad.add_child(head)
	col.add_child(head_pad)
	col.add_child(DccTheme.rule())
	col.add_child(_dock_readout("left"))

	var scroll := _scroll()
	left_dock_body = VBoxContainer.new()
	left_dock_body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	left_dock_body.add_theme_constant_override("separation", 0)
	scroll.add_child(left_dock_body)
	col.add_child(scroll)
	return left_dock

func _build_right_dock() -> Control:
	right_dock = PanelContainer.new()
	right_dock.custom_minimum_size.x = _right_width
	right_dock.add_theme_stylebox_override("panel", DccTheme.panel("panel", {"left": 1}))

	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 0)
	right_dock.add_child(col)

	var head := HBoxContainer.new()
	head.custom_minimum_size.y = 26
	head.add_child(_collapse_button(false))
	head.add_child(DccTheme.header("LAYERS"))
	head.add_child(DccTheme.spacer())
	var head_pad := MarginContainer.new()
	head_pad.add_theme_constant_override("margin_left", 6)
	head_pad.add_theme_constant_override("margin_right", 12)
	head_pad.add_child(head)
	col.add_child(head_pad)
	col.add_child(DccTheme.rule())
	col.add_child(_dock_readout("right"))

	var scroll := _scroll()
	right_dock_body = VBoxContainer.new()
	right_dock_body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	right_dock_body.add_theme_constant_override("separation", 0)
	scroll.add_child(right_dock_body)
	col.add_child(scroll)
	return right_dock

## Godot's default theme draws a rounded, outlined panel behind every
## ScrollContainer. §11 is explicit that regions are separated by hairlines
## only, with radius 0 everywhere, so the panel is removed rather than
## restyled -- the dock around it already draws the one border there should be.
func _scroll() -> ScrollContainer:
	var s := ScrollContainer.new()
	s.size_flags_vertical = Control.SIZE_EXPAND_FILL
	s.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	s.add_theme_stylebox_override("panel", DccTheme.empty())
	return s

func _collapse_button(is_left: bool) -> Button:
	var b := Button.new()
	b.flat = true
	b.focus_mode = Control.FOCUS_NONE
	b.text = DccIcons.SYMBOLS["collapse"] if is_left else DccIcons.SYMBOLS["expand"]
	b.add_theme_color_override("font_color", DccTheme.c("text_faint"))
	b.custom_minimum_size = Vector2(_scaled(20), _scaled(20))
	b.pressed.connect(_toggle_dock.bind(is_left))
	_collapse_buttons["left" if is_left else "right"] = b
	return b

## §6's last line: "collapsed, the dock keeps its primary readout visible --
## elevation for Sample, layer dots for Layers, stamp count for the stack." So a
## collapsed dock is not an empty 40 px strip; it is a strip that still says the
## one thing you collapsed it in order to keep watching.
##
## The label lives outside the ScrollContainer precisely because collapsing
## hides that container -- putting the readout inside it would hide the thing
## the rule exists to preserve.
func _dock_readout(side: String) -> Control:
	var l := DccTheme.label("", "text_dim", DccTheme.FS_TINY)
	l.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	l.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	var pad := MarginContainer.new()
	pad.add_theme_constant_override("margin_left", 4)
	pad.add_theme_constant_override("margin_right", 4)
	pad.add_theme_constant_override("margin_top", 8)
	pad.add_child(l)
	pad.visible = false
	_dock_readouts[side] = l
	return pad

## Whatever the dock's current context considers its one essential number. Kept
## up to date whether or not the dock is collapsed, so collapsing never reveals
## a stale value.
func set_dock_readout(side: String, text: String) -> void:
	if not _dock_readouts.has(side):
		push_error("DccShell: no dock readout for side '%s'" % side)
		return
	(_dock_readouts[side] as Label).text = text

func is_dock_collapsed(side: String) -> bool:
	return _left_collapsed if side == "left" else _right_collapsed

## A collapsed dock shrinks to the rail width rather than disappearing, and
## swaps its body for the readout above.
func _toggle_dock(is_left: bool) -> void:
	var dock := left_dock if is_left else right_dock
	var side := "left" if is_left else "right"
	var collapsed := not (_left_collapsed if is_left else _right_collapsed)
	dock.custom_minimum_size.x = float(DccTheme.W_RAIL_COLLAPSED) if collapsed else (_left_width if is_left else _right_width)
	for child in dock.get_child(0).get_children():
		if child is ScrollContainer:
			child.visible = not collapsed
	(_dock_readouts[side] as Label).get_parent().visible = collapsed
	if is_left:
		## The title has no room at 40 px; the chevron is all that fits, and it
		## is the only affordance for getting the dock back.
		left_dock_title.visible = not collapsed
		_left_collapsed = collapsed
	else:
		_right_collapsed = collapsed
	var btn: Button = _collapse_buttons.get(side)
	if btn != null:
		var open_glyph: String = DccIcons.SYMBOLS["collapse"] if is_left else DccIcons.SYMBOLS["expand"]
		var shut_glyph: String = DccIcons.SYMBOLS["expand"] if is_left else DccIcons.SYMBOLS["collapse"]
		btn.text = shut_glyph if collapsed else open_glyph

# -- §9 Viewport --------------------------------------------------------------

func _build_viewport() -> Control:
	var area := PanelContainer.new()
	area.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	area.add_theme_stylebox_override("panel", DccTheme.flat(DccTheme.c("bg")))
	viewport_area = area
	viewport_content = Control.new()
	viewport_content.clip_contents = true
	area.add_child(viewport_content)
	return area

# -- §10 Timeline bar ---------------------------------------------------------

func _build_timeline() -> Control:
	var bar := PanelContainer.new()
	bar.custom_minimum_size.y = _scaled(DccTheme.H_TIMELINE)
	bar.add_theme_stylebox_override("panel", DccTheme.panel("panel", {"top": 1}))
	timeline_row = HBoxContainer.new()
	timeline_row.add_theme_constant_override("separation", 14)
	var pad := MarginContainer.new()
	pad.add_theme_constant_override("margin_left", 14)
	pad.add_theme_constant_override("margin_right", 14)
	pad.add_theme_constant_override("margin_top", 8)
	pad.add_theme_constant_override("margin_bottom", 8)
	pad.add_child(timeline_row)
	bar.add_child(pad)
	return bar

# -- §11 Status bar -----------------------------------------------------------

func _build_status_bar() -> Control:
	var bar := PanelContainer.new()
	bar.custom_minimum_size.y = _scaled(DccTheme.H_STATUS)
	bar.add_theme_stylebox_override("panel", DccTheme.panel("panel_alt", {"top": 1}))
	status_row = HBoxContainer.new()
	status_row.add_theme_constant_override("separation", 18)
	var pad := MarginContainer.new()
	pad.add_theme_constant_override("margin_left", 14)
	pad.add_theme_constant_override("margin_right", 14)
	pad.add_child(status_row)
	bar.add_child(pad)

	for slot in ["pass", "stale", "autosave", "atlas"]:
		var l := DccTheme.label("", "text_faint", DccTheme.FS_SMALL)
		_status_labels[slot] = l
		status_row.add_child(l)
	status_row.add_child(DccTheme.spacer())
	var hint := DccTheme.label("", "text_ghost", DccTheme.FS_SMALL)
	_status_labels["hint"] = hint
	status_row.add_child(hint)
	return bar

## Set one status slot. Slots: pass, stale, autosave, atlas, hint, and the menu
## bar's top_world / top_pass / top_cpu / top_gpu / top_mem.
func set_status(slot: String, text: String, token: String = "text_faint") -> void:
	if not _status_labels.has(slot):
		push_error("DccShell: no status slot '%s'" % slot)
		return
	var l: Label = _status_labels[slot]
	l.text = text
	l.add_theme_color_override("font_color", DccTheme.c(token))
