extends AcceptDialog

## Story ▸ CONFLICTS & CAMPAIGNS ▸ Conflicts… -- milestone SM-6 of
## `STORY_MENUS_PROPOSAL.md` (§3.3, §6; owner Ruling CK, 2026-10-18).
##
## Every conflict in the world, with its year range, in one list. Before this
## window the conflict store (`conflict_bridge.rs`, SP-4) was reached four ways
## -- the CIVIL Conflict tool, the CARTO Conflict layer, the right dock's
## Conflict context and the Factions window's Conflicts section -- and none of
## them listed every conflict world-wide by year (§2: "No list of every
## conflict exists, and no menu row reaches them"). CIVIL ▸ Military ▸
## Conflicts lists them too, but inside a dock category, without sides or
## places.
##
## **No mockup draws this window** (§3.3 marks the row P, "a world-wide list
## window is new (derived)"). Derived from the DCC vocabulary of the nearest
## shipped windows: `travel_library_window.gd`'s head row and rules, and
## CIVIL ▸ Military ▸ Conflicts' own row grammar (`civilization_workspace.gd::
## _fill_conflicts`: `●` active in the cursor's year / `○` not, the kind label
## from `CONFLICT_KIND_LABELS`, "start–end" or "ongoing").
##
## **A row shows only what `conflict_list()` stores**: `name`, `kind`,
## `start_year`, `end_year` (absent = ongoing, never 0), `sides` (faction ids,
## named from `get_factions()`), `active` (judged by the engine at the Timeline
## cursor's year) and, when anchored, `anchor_name` (absent when the place is
## gone -- then "place missing", the right dock's own wording, `right_dock.gd`
## Conflict context). A row press is the shell's one conflict-selection path,
## `app.open_conflict(id)` (the right dock's Conflict context and the map's
## selected stroke), plus a move of the map view to the conflict's resolved
## points -- the "select on map" half §6 asks for.
##
## **What it must never do**: call `WorldGen` while `bridge.generating`; edit a
## conflict (editing is the right dock's, where a row press lands); or invent a
## year, side or place it did not read.
##
## Preloaded by `app.gd` (`CONFLICTS_WINDOW`), no `class_name` -- the same
## no-import-pass reason as `saved_journeys_window.gd`.

## The shell (`DccApp`): `open_conflict`, `viewport`, `phone_fit`, the Conflict tool.
var _host
## The `EngineBridge`; every engine read here goes through it.
var _bridge: EngineBridge
## The list body, cleared and refilled by `_refresh()`.
var _body: VBoxContainer
## "N conflicts · M active in year Y", set by `_refresh()`.
var _readout: Label
## The ALL / ACTIVE chip row, refilled by `_refresh()` (its year is live).
var _chips: HFlowContainer
## The filter: `"all"`, or `"active"` (active in the Timeline cursor's year).
var _filter := "all"
## Phone composition (`DccWidgets.phone_window`): rows become two lines.
var _phone := false

## Builds the frame once (head row, readout, chips, scrolling list) and wires a
## refresh to the bridge signals that change the store. Call once, before
## `open()`; it appends to `self`.
func setup(host, bridge: EngineBridge) -> void:
	_host = host
	_bridge = bridge
	title = "Conflicts"
	## A labelled judgement: six short columns plus the name on one line.
	size = Vector2i(860, 540)
	min_size = Vector2i(600, 360)
	_phone = DccWidgets.phone_window(self, host)
	var outer := VBoxContainer.new()
	outer.add_theme_constant_override("separation", 0)
	add_child(outer)
	if _phone:
		DccWidgets.phone_head(outer, "Conflicts", "every conflict in this world · tap one to select it")
	else:
		var head_row := HBoxContainer.new()
		head_row.add_theme_constant_override("separation", 12)
		DccWidgets.pad(outer, 12, 6, 12, 6).add_child(head_row)
		head_row.add_child(DccTheme.mono_label("◇ CONFLICTS", "accent", DccTheme.FS_HEADER, 2, true))
		head_row.add_child(DccTheme.label("Story ▸ Conflicts", "text_ghost", DccTheme.FS_TINY))
		head_row.add_child(DccTheme.spacer())
		head_row.add_child(DccTheme.label("every conflict · select one to edit it in the right dock", "text_ghost", DccTheme.FS_TINY))
		outer.add_child(DccTheme.rule())
	var top := VBoxContainer.new()
	top.add_theme_constant_override("separation", 6)
	DccWidgets.pad(outer, 14, 8, 14, 6).add_child(top)
	_readout = DccTheme.mono_label("", "text_dim", DccTheme.FS_SMALL, 0)
	_readout.name = "CfReadout"
	top.add_child(_readout)
	_chips = HFlowContainer.new()
	_chips.name = "CfChips"
	_chips.add_theme_constant_override("h_separation", 4)
	_chips.add_theme_constant_override("v_separation", 4)
	top.add_child(_chips)
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
	_body.name = "CfRows"
	_body.add_theme_constant_override("separation", 0)
	_body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	pad.add_child(_body)
	bridge.generation_finished.connect(func(_ok: bool): if visible: _refresh())
	bridge.world_loaded.connect(func(): if visible: _refresh())

## Shows the window with a fresh read: phone-shaped on a phone, centred elsewhere.
func open() -> void:
	_refresh()
	if not DccWidgets.phone_present(self, _host):
		popup_centered()
	if _phone:
		_host.phone_fit(self, 1.0)

## Rebuilds the chips and the list from `conflict_list()`, or draws the state
## that stops it: a generation running, no world, nothing recorded.
func _refresh() -> void:
	for c in _body.get_children():
		_body.remove_child(c)
		c.queue_free()
	for c in _chips.get_children():
		_chips.remove_child(c)
		c.queue_free()
	if bool(_bridge.generating):
		_readout.text = "Generating"
		DccWidgets.note(_body, "A generation is running; the conflict store is read when it finishes (the engine object belongs to the worker until then).")
		return
	if not bool(_bridge.has_world):
		_readout.text = "No world"
		DccWidgets.note(_body, "No world yet. Generate or open one, then draw a conflict with Story ▸ Draw a conflict.")
		return
	var all: Array = _bridge.conflict_list()
	var year := int(_bridge.get_civ_year())
	var active := all.filter(func(c): return bool((c as Dictionary).get("active", false)))
	_readout.text = "%d conflict%s · %d active in year %d" % [all.size(), "" if all.size() == 1 else "s",
		active.size(), year]
	if all.is_empty():
		DccWidgets.note(_body, "No conflict is recorded in this world. Draw a front, an arrow, a siege or a battle with the CIVIL Conflict tool; each one is listed here with its years and sides.")
		var b := DccWidgets.action(_body, "Draw a conflict", func():
			hide()
			_host.select_domain("civilization")
			_host.arm_tool("conflict"))
		b.name = "CfDraw"
		return
	var ca := DccWidgets.segment(_chips, "ALL · %d" % all.size(), func(): _set_filter("all"))
	DccWidgets.set_segment_on(ca, _filter == "all")
	var cb := DccWidgets.segment(_chips, "ACTIVE IN %d · %d" % [year, active.size()], func(): _set_filter("active"))
	cb.name = "CfActiveChip"
	DccWidgets.set_segment_on(cb, _filter == "active")
	cb.tooltip_text = "Only the conflicts whose years include the Timeline cursor's year (%d); the map draws only these." % year
	var rows: Array = active if _filter == "active" else all
	## Chronological: start year, then creation order (the store's own id).
	rows = rows.duplicate()
	rows.sort_custom(func(a: Dictionary, b: Dictionary) -> bool:
		if int(a.get("start_year", 0)) != int(b.get("start_year", 0)):
			return int(a.get("start_year", 0)) < int(b.get("start_year", 0))
		return int(a.get("id", 0)) < int(b.get("id", 0)))
	if rows.is_empty():
		DccWidgets.note(_body, "No conflict is active in year %d. Choose ALL, or move the Timeline cursor." % year)
		return
	var names := {}
	for f in _bridge.get_factions():
		names[int((f as Dictionary).get("id", -1))] = String((f as Dictionary).get("name", ""))
	if not _phone:
		_body.add_child(_header_row())
	for c in rows:
		_add_row(c as Dictionary, names, year)
	if _phone:
		_host.phone_fit(self, 1.0)

## A chip press: sets the filter and re-reads.
func _set_filter(f: String) -> void:
	_filter = f
	_refresh()

## The desktop column captions, in the row's own order and widths.
func _header_row() -> Control:
	var h := HBoxContainer.new()
	h.add_theme_constant_override("separation", 10)
	for spec in [["", 14], ["CONFLICT", 0], ["KIND", 56], ["YEARS", 96], ["SIDES", 190], ["PLACE", 120]]:
		var l := DccTheme.mono_label(String(spec[0]), "text_faint", DccTheme.FS_MICRO, 2)
		if int(spec[1]) == 0:
			l.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		else:
			l.custom_minimum_size.x = int(spec[1])
		h.add_child(l)
	return h

## One conflict: `●`/`○` (active in `year` or not), the name as the button,
## then kind, years, sides and place -- columns on desktop, one line under the
## name on a phone. `names` maps faction id to name; an id the roster lacks is
## shown as "faction N", never dropped.
func _add_row(c: Dictionary, names: Dictionary, year: int) -> void:
	var id := int(c.get("id", 0))
	var nm := String(c.get("name", ""))
	if nm == "":
		nm = "(unnamed #%d)" % id
	var on := bool(c.get("active", false))
	var kind := String(CivilizationWorkspace.CONFLICT_KIND_LABELS.get(String(c.get("kind", "")), "?"))
	var years := "%d–%s" % [int(c.get("start_year", 0)),
		str(int(c["end_year"])) if c.has("end_year") else "ongoing"]
	var side_names: PackedStringArray = []
	for f in Array(c.get("sides", PackedInt32Array())):
		var n := String(names.get(int(f), ""))
		side_names.append(n if n != "" else "faction %d" % int(f))
	var sides := " vs ".join(side_names) if not side_names.is_empty() else "no side named"
	var place := ""
	if c.has("anchor_kind"):
		place = String(c.get("anchor_name", "")) if bool(c.get("anchor_resolved", false)) else "place missing"
	else:
		place = "not attached"
	var row: BoxContainer = VBoxContainer.new() if _phone else HBoxContainer.new()
	row.name = "CfRow"
	row.set_meta("conflict_id", id)
	row.add_theme_constant_override("separation", 2 if _phone else 10)
	_body.add_child(row)
	var mark := "● " if on else "○ "
	var b := Button.new()
	b.name = "CfSelect"
	b.text = (mark + nm) if _phone else nm
	b.alignment = HORIZONTAL_ALIGNMENT_LEFT
	b.focus_mode = Control.FOCUS_NONE
	b.flat = true
	b.add_theme_color_override("font_color", DccTheme.c("text" if on else "text_dim"))
	b.add_theme_color_override("font_hover_color", DccTheme.c("accent"))
	b.add_theme_font_size_override("font_size", DccTheme.FS_SMALL)
	b.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	## Tablet: the touch floor every dock row takes (`role_px("row_min_h")`,
	## the same floor `DccWidgets.group` headers use); the phone's comes from
	## `phone_fit`. Desktop keeps the button's own height.
	if DccTheme.is_tablet():
		b.custom_minimum_size.y = DccTheme.role_px("row_min_h")
	b.tooltip_text = ("Active in %d -- drawn on the map. " if on else "Not active in %d -- hidden on the map until the year cursor enters its range. ") % year \
		+ "Selects it on the map and opens it in the right dock, where its sides and years are edited."
	b.pressed.connect(_select.bind(id, c.get("points", PackedVector2Array())))
	if _phone:
		row.add_child(b)
		row.add_child(DccTheme.mono_label("%s · %s · %s · %s" % [kind, years, sides, place],
			"text_ghost", DccTheme.FS_TINY, 0))
	else:
		var m := DccTheme.mono_label(mark.strip_edges(), "accent" if on else "text_ghost", DccTheme.FS_SMALL, 0)
		m.custom_minimum_size.x = 14
		row.add_child(m)
		row.add_child(b)
		for spec in [[kind, 56], [years, 96], [sides, 190], [place, 120]]:
			var l := DccTheme.mono_label(String(spec[0]), "text_dim", DccTheme.FS_TINY, 0)
			l.custom_minimum_size.x = int(spec[1])
			l.clip_text = true
			l.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
			l.tooltip_text = String(spec[0])
			l.mouse_filter = Control.MOUSE_FILTER_PASS
			row.add_child(l)
	_body.add_child(DccTheme.rule())

## Closes the window, selects conflict `id` (`app.open_conflict`: the right
## dock's Conflict context and the map's selected stroke, plus the phone's
## right sheet) in CIVIL, and centres the view on its resolved points. A
## conflict with no points moves nothing.
func _select(id: int, points: PackedVector2Array) -> void:
	hide()
	_host.select_domain("civilization")
	_host.open_conflict(id)
	if not points.is_empty() and _host.viewport != null:
		var sum := Vector2.ZERO
		for p in points:
			sum += p
		var mid := sum / float(points.size())
		_host.viewport.move_view_to(mid.x, mid.y)
