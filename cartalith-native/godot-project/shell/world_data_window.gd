extends AcceptDialog
class_name WorldDataWindow

## Data ▸ World data tables (`DCC_SHELL_SPEC.md` §2.4/§9): the settlement,
## province and economy tables, sortable and filterable.
##
## §9's Data manager is a five-route window this is one route of; the rest
## (import, export, sources, conversion, validation) has no engine behind it
## yet and is not faked here.
##
## Was a placeholder ("Being ported from main.gd's world-data dialog") behind
## a live, enabled `Data ▸ World data tables…` menu item -- found in the
## 2026-08-19 GUI audit as a real dead end: the one menu item in this shell
## that opened a window and delivered nothing, when the data it promised
## (`bridge.settlements()`/`provinces()`/`trade_balances()`) was already real
## and already read in full elsewhere (`civilization_workspace.gd`'s own
## Settlements/Politics/Economy categories, capped at a top-N summary; this
## window is the uncapped, filterable table those categories point at). Three
## tabs, one filter field shared across them (filters by name substring,
## case-insensitive). "Sortable" is Settlements sorted by population
## descending, unconditionally -- the reference's own most useful ordering,
## not a full per-column sort UI with a clickable header, which the spec's
## own three-field table doesn't name by control.

var bridge: EngineBridge

var _tabs: TabContainer
var _filter := ""
var _settlements_body: VBoxContainer
var _provinces_body: VBoxContainer
var _trade_body: VBoxContainer
var _sort_by_pop := true   ## Settlements tab only.

## Phone (§13) -- PH-12. Nothing here is a two-column composition, so the fault
## was purely density: a 760x620 desktop card drawn at native resolution inside
## a 1440x3168 panel, 20 px table rows (about 1 mm on a 510 ppi screen) and a
## 29 dp OK button for a way out.
##
## The one thing a content scale does not answer is the **six-column** table.
## Six `clip_text` columns across 393 dp is ~55 dp each -- wide enough for
## "Popul…" and nothing else. `_row()`/`_header_row()` therefore lay a row out
## on two lines there, name over the rest, which keeps every column rather than
## dropping the ones that fit worst.
##
## The other half is **how many rows** get built at all. Measured on a
## generated world (parallel phone sweep, 2026-08-25): the settlements tab is
## **1 470 individual `Label` nodes** across 240 rows, and the only way through
## them is an ~8 px scrollbar. Two lines per row would have made that 1 700.
## A phone therefore builds `PHONE_ROW_CAP` rows and says how many it left,
## with a button that reveals another page -- the same shape a mobile list
## takes everywhere, and honest about what it is not showing rather than
## silently truncating. The filter field above it is the real answer for a
## specific settlement, and the note says so.
var _phone := false
const PHONE_ROW_CAP := 50
var _row_cap := PHONE_ROW_CAP

## Returns true when this row must not be built. Call once per row that has
## already passed the name filter, so the cap counts *matching* rows -- capping
## before the filter would hide the very row a search was for.
func _capped(shown: int) -> bool:
	return _phone and shown > _row_cap

## The "…and N more" foot a capped table ends with. `total` is how many rows
## passed the filter, `built` how many were drawn.
func _cap_note(body: VBoxContainer, built: int, total: int) -> void:
	if not _phone or total <= built:
		return
	body.add_child(DccTheme.rule())
	DccWidgets.note(body, "Showing %d of %d. Filter by name above to narrow, or:" % [built, total])
	var more := Button.new()
	more.text = "Show %d more" % mini(PHONE_ROW_CAP, total - built)
	more.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	more.pressed.connect(func():
		_row_cap += PHONE_ROW_CAP
		_rebuild())
	body.add_child(more)

func setup(b: EngineBridge) -> void:
	bridge = b
	title = "World data"
	size = Vector2i(760, 620)
	min_size = Vector2i(560, 400)
	ok_button_text = "Close"
	_phone = DccWidgets.phone_window(self, get_parent())

	var outer := VBoxContainer.new()
	outer.add_theme_constant_override("separation", 6)
	add_child(outer)
	if _phone:
		DccWidgets.phone_head(outer, "World data", "settlements · provinces · economy")

	var filter_row := HBoxContainer.new()
	filter_row.add_theme_constant_override("separation", 8)
	var search := LineEdit.new()
	search.placeholder_text = "filter by name"
	if _phone:
		search.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	else:
		search.custom_minimum_size.x = 220
	search.text_changed.connect(func(t: String): _filter = t.to_lower(); _rebuild())
	filter_row.add_child(search)
	outer.add_child(filter_row)
	outer.add_child(DccTheme.rule())

	_tabs = TabContainer.new()
	_tabs.size_flags_vertical = Control.SIZE_EXPAND_FILL
	outer.add_child(_tabs)
	## PH-12: a `TabContainer`'s tab strip is drawn by an INTERNAL `TabBar`, so
	## `DccShell.phone_fit()` -- which walks `get_children()` -- never reaches
	## it, exactly as it never reaches `AcceptDialog`'s button bar. A tab has no
	## height property either; its height is the font plus the stylebox's own
	## vertical content margins, so those are the knob. Measured stock: 26 dp,
	## for the control that switches between the three tables.
	if _phone:
		for state in ["tab_selected", "tab_unselected", "tab_hovered", "tab_focus"]:
			var sb: StyleBox = _tabs.get_theme_stylebox(state, "TabContainer").duplicate()
			var pad: float = maxf(0.0, (DccTheme.PHONE_TAP_MIN - DccTheme.FS_BODY) * 0.5)
			sb.content_margin_top = pad
			sb.content_margin_bottom = pad
			_tabs.add_theme_stylebox_override(state, sb)

	_settlements_body = _build_tab("Settlements")
	_provinces_body = _build_tab("Provinces")
	_trade_body = _build_tab("Economy")

	bridge.generation_finished.connect(func(_ok: bool): if visible: _rebuild())
	bridge.world_loaded.connect(func(): if visible: _rebuild())

func _build_tab(label_text: String) -> VBoxContainer:
	var scroll := ScrollContainer.new()
	scroll.name = label_text
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	var pad := MarginContainer.new()
	for side in ["left", "top", "right", "bottom"]:
		pad.add_theme_constant_override("margin_" + side, 12)
	scroll.add_child(pad)
	var body := VBoxContainer.new()
	body.add_theme_constant_override("separation", 2)
	body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	pad.add_child(body)
	_tabs.add_child(scroll)
	return body

func _clear(body: VBoxContainer) -> void:
	for c in body.get_children():
		body.remove_child(c)
		c.queue_free()

## PH-12's two-line phone row. The name gets its own full-width line and the
## remaining columns share the one under it, so a six-column settlement row is
## five ~70 dp cells instead of six ~55 dp ones, with the name -- the column
## every filter and every lookup is keyed on -- unclipped.
##
## `_row()` and `_header_row()` build the SAME two-line shape, which is what
## keeps the header over its own cells; a header that stayed one line would sit
## over nothing.
func _cells(cols: Array, from: int, header: bool) -> HBoxContainer:
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 10)
	if not header:
		row.custom_minimum_size.y = 20
	for i in range(from, cols.size()):
		var l: Label
		if header:
			l = DccTheme.mono_label(String(cols[i]), "text_faint", DccTheme.FS_MICRO, 1, true)
		else:
			l = DccTheme.mono_label(String(cols[i]), "text_dim" if i == 0 else "text", DccTheme.FS_SMALL)
			l.clip_text = true
		l.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		l.size_flags_stretch_ratio = 2.0 if i == 0 else 1.0
		row.add_child(l)
	return row

func _row(body: VBoxContainer, cols: Array, tooltip: String = "") -> void:
	if not (_phone and cols.size() > 3):
		var flat := _cells(cols, 0, false)
		flat.tooltip_text = tooltip
		body.add_child(flat)
		return
	var stack := VBoxContainer.new()
	stack.add_theme_constant_override("separation", 0)
	stack.tooltip_text = tooltip
	var name_l := DccTheme.mono_label(String(cols[0]), "text_bright", DccTheme.FS_SMALL)
	name_l.clip_text = true
	stack.add_child(name_l)
	stack.add_child(_cells(cols, 1, false))
	body.add_child(stack)

func _header_row(body: VBoxContainer, cols: Array) -> void:
	if not (_phone and cols.size() > 3):
		body.add_child(_cells(cols, 0, true))
		body.add_child(DccTheme.rule())
		return
	var stack := VBoxContainer.new()
	stack.add_theme_constant_override("separation", 0)
	stack.add_child(DccTheme.mono_label(String(cols[0]), "text_faint", DccTheme.FS_MICRO, 1, true))
	stack.add_child(_cells(cols, 1, true))
	body.add_child(stack)
	body.add_child(DccTheme.rule())

func _rebuild() -> void:
	_rebuild_settlements()
	_rebuild_provinces()
	_rebuild_trade()
	## PH-12: every row above is a fresh node, and this runs on a filter
	## keystroke, on Show-more, and on a generate finishing. Idempotent by
	## meta-flag (`DccShell.phone_fit`), so it only touches what was just made.
	if _phone and get_parent() != null and get_parent().has_method("phone_fit"):
		get_parent().phone_fit(self, 1.0)

func _rebuild_settlements() -> void:
	var body := _settlements_body
	_clear(body)
	if not bridge.has_world:
		DccWidgets.note(body, "No world generated -- File ▸ New world… to begin.")
		return
	var rows: Array = bridge.settlements().duplicate()
	if _sort_by_pop:
		rows.sort_custom(func(a, b): return int((a as Dictionary).get("population", 0)) > int((b as Dictionary).get("population", 0)))
	var head := HBoxContainer.new()
	head.add_theme_constant_override("separation", 8)
	head.add_child(DccTheme.mono_label("%d settlements" % rows.size(), "text_faint", DccTheme.FS_MICRO))
	head.add_child(DccTheme.spacer())
	body.add_child(head)
	_header_row(body, ["Name", "Class", "Population", "Faction", "Coastal", "Capital"])
	var shown := 0
	var built := 0
	for r in rows:
		var d: Dictionary = r
		var name := String(d.get("name", "?"))
		if _filter != "" and name.to_lower().find(_filter) < 0:
			continue
		shown += 1
		if _capped(shown):
			continue
		built += 1
		_row(body, [name, String(d.get("kind", "?")).capitalize(), str(int(d.get("population", 0))),
			str(int(d.get("faction", 0))), "yes" if d.get("coastal", false) else "no",
			"yes" if d.get("capital", false) else "no"])
	_cap_note(body, built, shown)
	if shown == 0:
		DccWidgets.note(body, "No settlement matches \"%s\"." % _filter if _filter != "" else "No settlements.")

func _rebuild_provinces() -> void:
	var body := _provinces_body
	_clear(body)
	if not bridge.has_world:
		DccWidgets.note(body, "No world generated -- File ▸ New world… to begin.")
		return
	var provinces := bridge.provinces()
	var settlements := bridge.settlements()
	var head := HBoxContainer.new()
	head.add_theme_constant_override("separation", 8)
	head.add_child(DccTheme.mono_label("%d provinces" % provinces.size(), "text_faint", DccTheme.FS_MICRO))
	head.add_child(DccTheme.spacer())
	body.add_child(head)
	_header_row(body, ["Name", "Faction", "Capital settlement"])
	var shown := 0
	var built := 0
	for p in provinces:
		var d: Dictionary = p
		var name := String(d.get("name", "?"))
		if _filter != "" and name.to_lower().find(_filter) < 0:
			continue
		shown += 1
		if _capped(shown):
			continue
		built += 1
		var cap_idx := int(d.get("capital_settlement_index", -1))
		var cap_name := "—"
		if cap_idx >= 0 and cap_idx < settlements.size():
			cap_name = String((settlements[cap_idx] as Dictionary).get("name", "—"))
		_row(body, [name, str(int(d.get("faction", 0))), cap_name])
	_cap_note(body, built, shown)
	if shown == 0:
		DccWidgets.note(body, "No province matches \"%s\"." % _filter if _filter != "" else "No provinces.")

## `get_trade_balances()`'s own doc comment: "one Dictionary per entry in
## get_settlements(), same order/index" -- there is no settlement-name field
## inside the trade dict itself, so the name is read positionally from
## `settlements[i]`, not invented.
func _rebuild_trade() -> void:
	var body := _trade_body
	_clear(body)
	if not bridge.has_world:
		DccWidgets.note(body, "No world generated -- File ▸ New world… to begin.")
		return
	var settlements := bridge.settlements()
	var balances := bridge.trade_balances()
	if balances.is_empty():
		DccWidgets.note(body, "No trade balances -- generate a world first.")
		return
	DccWidgets.note(body,
		"civ_resource_trade_balance's own hinterland term (ECONOMY_SCOPE.md) -- goods flow "
		+ "per settlement, not a faction-level aggregation (population, tax, the five-axis "
		+ "power heuristic), which remains unstarted real future scope.")
	body.add_child(DccTheme.rule())
	_header_row(body, ["Settlement", "Exports", "Imports"])
	var shown := 0
	var built := 0
	for i in mini(settlements.size(), balances.size()):
		var s: Dictionary = settlements[i]
		var name := String(s.get("name", "?"))
		if _filter != "" and name.to_lower().find(_filter) < 0:
			continue
		var t: Dictionary = balances[i]
		var ex: PackedStringArray = t.get("exports", PackedStringArray())
		var im: PackedStringArray = t.get("imports", PackedStringArray())
		if ex.is_empty() and im.is_empty():
			continue
		shown += 1
		if _capped(shown):
			continue
		built += 1
		_row(body, [name, ", ".join(ex) if ex.size() > 0 else "—", ", ".join(im) if im.size() > 0 else "—"])
	_cap_note(body, built, shown)
	if shown == 0:
		DccWidgets.note(body, "No settlement matches \"%s\" with a trade relationship." % _filter if _filter != "" else "No settlement carries a trade relationship.")

## `tab`, if given, selects that tab by its title ("Settlements" -- the
## default TabContainer selection, "Provinces", "Economy") rather than always
## landing on the first one -- RD-03's Settlement ▸ Economy button opens
## straight to the Economy tab instead of making the caller re-click after
## the window opens. Mirrors `DataManagerWindow.open(group)`'s own "scope to
## X, empty picks the default" shape.
func open(tab: String = "") -> void:
	## PH-12: the cap is a per-visit state, not a preference. Opening the window
	## again starts at one page, the same way a re-opened list does everywhere.
	_row_cap = PHONE_ROW_CAP
	_rebuild()
	if tab != "":
		for i in _tabs.get_tab_count():
			if _tabs.get_tab_title(i) == tab:
				_tabs.current_tab = i
				break
	if DccWidgets.phone_present(self, get_parent()):
		return
	popup_centered()
