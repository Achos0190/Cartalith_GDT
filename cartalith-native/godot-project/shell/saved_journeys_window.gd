extends AcceptDialog

## Story ▸ JOURNEYS ▸ Saved journeys… -- milestone SM-6 of
## `STORY_MENUS_PROPOSAL.md` (§3.3, §6; owner Ruling CK, 2026-10-18).
##
## A list of every journey saved in this world, one row per engine `Journey`
## (`STORY_PLANNING_SCOPE.md` SP-1's entity, `bridge.journey_list()`), and a row
## press that opens it in the Journey planner. Before this window a saved
## journey was reachable only from the planner's own left-dock Journeys list,
## i.e. only once the planner was already open (§2: "Saved journeys exist as a
## store ... but no window lists them").
##
## **No mockup draws this window.** §3.3 marks the row P ("the list window is
## new (derived)"), and A2/A8 draw only the menu row and the phone MORE row. So
## it is derived from the DCC vocabulary the nearest shipped window already
## uses -- `travel_library_window.gd`'s head row (accent mono title, the menu
## path in ghost type, a right-aligned caption), its rule lines, and its
## two-pane-on-desktop / stacked-on-phone convention (one pane here: a list) --
## per the owner's 2026-08-25 rule ("where none exists, derive from the DCC
## canvases' own vocabulary", `DCC_SHELL_SCOPE.md`).
##
## **What a row shows is only what the engine stores** (`lib.rs::journey_list`):
## `name`, `party_preset` (a Travel library preset id, named through
## `tl_get("preset", id)` and shown as the raw id when the library no longer
## holds it), `start_year`, `km` (the route's length at save time) and `mode`
## (`land` / `water` / `mixed`, `project_bridge.rs::route_mode_key`). No
## duration, cost or arrival is drawn: those are the planner's computation
## over the current world, not stored fields, and reading them here would mean
## planning every journey on open.
##
## **What it must never do**: call `WorldGen` while `bridge.generating` (the
## worker owns it; `_refresh` says so instead of reading), write anything (the
## only verb is "open in the planner"; deleting stays in the planner's list,
## where it already keeps the planner's own copy in step -- `_delete_journey`),
## or show a value it did not read.
##
## Preloaded by `app.gd` (`SAVED_JOURNEYS_WINDOW`) rather than given a
## `class_name`, so adding it needs no `--headless --import` pass (CLAUDE.md:
## that pass strips `project.godot`'s `;` comments) -- `export_dialog.gd`'s
## precedent.

## The shell (`DccApp`), for `phone_fit`, `open_journey_planner` and the planner view.
var _host
## The `EngineBridge`; every engine read here goes through it.
var _bridge: EngineBridge
## The list body, cleared and refilled by `_refresh()`.
var _body: VBoxContainer
## The readout line under the head row ("N saved journeys"); set by `_refresh()`.
var _readout: Label
## Phone composition (`DccWidgets.phone_window`): rows become two lines.
var _phone := false

## Builds the frame once (head row, readout, scrolling list) and wires the
## refresh to the two bridge signals that change what the list can say. Call
## once, before `open()`. Must never be called twice: it appends to `self`.
func setup(host, bridge: EngineBridge) -> void:
	_host = host
	_bridge = bridge
	title = "Saved journeys"
	## A labelled judgement: five columns of short values plus the name read
	## on one line at 760; the floor keeps the name column above ~200 px.
	size = Vector2i(760, 520)
	min_size = Vector2i(560, 360)
	_phone = DccWidgets.phone_window(self, host)
	var outer := VBoxContainer.new()
	outer.add_theme_constant_override("separation", 0)
	add_child(outer)
	if _phone:
		DccWidgets.phone_head(outer, "Saved journeys", "saved in this world · tap one to plan it")
	else:
		var head_row := HBoxContainer.new()
		head_row.add_theme_constant_override("separation", 12)
		var head_pad := DccWidgets.pad(outer, 12, 6, 12, 6)
		head_pad.add_child(head_row)
		head_row.add_child(DccTheme.mono_label("→ SAVED JOURNEYS", "accent", DccTheme.FS_HEADER, 2, true))
		head_row.add_child(DccTheme.label("Story ▸ Saved journeys", "text_ghost", DccTheme.FS_TINY))
		head_row.add_child(DccTheme.spacer())
		head_row.add_child(DccTheme.label("saved in this world · open one in the planner", "text_ghost", DccTheme.FS_TINY))
		outer.add_child(DccTheme.rule())
	_readout = DccTheme.mono_label("", "text_dim", DccTheme.FS_SMALL, 0)
	_readout.name = "SjReadout"
	DccWidgets.pad(outer, 14, 8, 14, 4).add_child(_readout)
	var scroll := ScrollContainer.new()
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	outer.add_child(scroll)
	var pad := MarginContainer.new()
	for side in ["left", "right", "bottom"]:
		pad.add_theme_constant_override("margin_" + side, 14)
	pad.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	scroll.add_child(pad)
	_body = VBoxContainer.new()
	_body.name = "SjRows"
	_body.add_theme_constant_override("separation", 0)
	_body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	pad.add_child(_body)
	## A generation ending, or a project opening, changes the store; an open
	## window re-reads rather than showing the old world's journeys.
	bridge.generation_finished.connect(func(_ok: bool): if visible: _refresh())
	bridge.world_loaded.connect(func(): if visible: _refresh())

## Shows the window with a fresh read: phone-shaped on a phone, centred
## elsewhere.
func open() -> void:
	_refresh()
	if not DccWidgets.phone_present(self, _host):
		popup_centered()
	if _phone:
		_host.phone_fit(self, 1.0)

## Rebuilds the list from `journey_list()`, or draws the state that stops it:
## a generation running, no world, nothing saved.
func _refresh() -> void:
	for c in _body.get_children():
		_body.remove_child(c)
		c.queue_free()
	if bool(_bridge.generating):
		_readout.text = "Generating"
		DccWidgets.note(_body, "A generation is running; the journey store is read when it finishes (the engine object belongs to the worker until then).")
		return
	if not bool(_bridge.has_world):
		_readout.text = "No world"
		DccWidgets.note(_body, "No world yet. Generate or open one, then plan and save a journey in the Journey planner.")
		return
	var rows: Array = _bridge.journey_list()
	_readout.text = "%d saved journey%s" % [rows.size(), "" if rows.size() == 1 else "s"]
	if rows.is_empty():
		DccWidgets.note(_body, "No journey is saved in this world. Plan one in the Journey planner and press Save journey; it is listed here and kept with the project.")
		var b := DccWidgets.action(_body, "Open the Journey planner", func():
			hide()
			_host.open_journey_planner())
		b.name = "SjOpenPlanner"
		return
	if not _phone:
		_body.add_child(_header_row())
	## Preset names looked up once per distinct id, not once per row.
	var preset_names := {}
	for r in rows:
		var pid := String((r as Dictionary).get("party_preset", ""))
		if pid != "" and not preset_names.has(pid):
			var p: Dictionary = _bridge.tl_get("preset", pid)
			preset_names[pid] = String(p.get("name", "")) if bool(p.get("ok", false)) else ""
	for r in rows:
		_add_row(r as Dictionary, preset_names)
	if _phone:
		_host.phone_fit(self, 1.0)

## The desktop column captions, in the row's own order and widths.
func _header_row() -> Control:
	var h := HBoxContainer.new()
	h.add_theme_constant_override("separation", 10)
	for spec in [["JOURNEY", 0], ["PARTY", 150], ["START", 60], ["LENGTH", 80], ["MODE", 60]]:
		var l := DccTheme.mono_label(String(spec[0]), "text_faint", DccTheme.FS_MICRO, 2)
		if int(spec[1]) == 0:
			l.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		else:
			l.custom_minimum_size.x = int(spec[1])
		h.add_child(l)
	return h

## One journey. The name is the button (a press opens it in the planner); the
## stored fields follow as columns on desktop and as one line under the name
## on a phone. `preset_names` maps a preset id to its library name, `""` when
## the library no longer has it (then the id itself is shown, never a guess).
func _add_row(r: Dictionary, preset_names: Dictionary) -> void:
	var id := int(r.get("id", -1))
	var nm := String(r.get("name", ""))
	if nm == "":
		nm = "(unnamed #%d)" % id
	var pid := String(r.get("party_preset", ""))
	var party := String(preset_names.get(pid, ""))
	if party == "":
		party = pid if pid != "" else "no party preset"
	var start := "year %d" % int(r.get("start_year", 0))
	var length := DccUnits.format(float(r.get("km", 0.0)))
	var mode := String(r.get("mode", ""))
	var row: BoxContainer = VBoxContainer.new() if _phone else HBoxContainer.new()
	row.name = "SjRow"
	row.set_meta("journey_id", id)
	row.add_theme_constant_override("separation", 2 if _phone else 10)
	_body.add_child(row)
	var b := Button.new()
	b.name = "SjOpen"
	b.text = nm
	b.alignment = HORIZONTAL_ALIGNMENT_LEFT
	b.focus_mode = Control.FOCUS_NONE
	b.flat = true
	b.add_theme_color_override("font_color", DccTheme.c("text"))
	b.add_theme_color_override("font_hover_color", DccTheme.c("accent"))
	b.add_theme_font_size_override("font_size", DccTheme.FS_SMALL)
	b.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	## Tablet: the touch floor every dock row takes (`role_px("row_min_h")`,
	## the same floor `DccWidgets.group` headers use); the phone's comes from
	## `phone_fit`. Desktop keeps the button's own height.
	if DccTheme.is_tablet():
		b.custom_minimum_size.y = DccTheme.role_px("row_min_h")
	b.tooltip_text = "Opens %s in the Journey planner: its party, plan and route as saved." % nm
	b.pressed.connect(_open_journey.bind(id))
	row.add_child(b)
	if _phone:
		row.add_child(DccTheme.mono_label("%s · %s · %s · %s" % [party, start, length, mode],
			"text_ghost", DccTheme.FS_TINY, 0))
	else:
		for spec in [[party, 150], [start, 60], [length, 80], [mode, 60]]:
			var l := DccTheme.mono_label(String(spec[0]), "text_dim", DccTheme.FS_TINY, 0)
			l.custom_minimum_size.x = int(spec[1])
			l.clip_text = true
			l.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
			l.tooltip_text = String(spec[0])
			l.mouse_filter = Control.MOUSE_FILTER_PASS
			row.add_child(l)
	_body.add_child(DccTheme.rule())

## Closes this window and opens journey `id` in the planner
## (`journey_planner_view.gd::open_saved`). Says so on the status line when the
## planner's list does not hold it, rather than opening on another journey.
func _open_journey(id: int) -> void:
	hide()
	var ok: bool = _host.journey_planner_view.open_saved(id)
	if not ok:
		_host.set_status("hint", "That journey is not in the planner's list; reopen the project to reload it.", "accent")
