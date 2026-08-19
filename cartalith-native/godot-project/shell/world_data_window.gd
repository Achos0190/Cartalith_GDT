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

func setup(b: EngineBridge) -> void:
	bridge = b
	title = "World data"
	size = Vector2i(760, 620)
	min_size = Vector2i(560, 400)

	var outer := VBoxContainer.new()
	outer.add_theme_constant_override("separation", 6)
	add_child(outer)

	var filter_row := HBoxContainer.new()
	filter_row.add_theme_constant_override("separation", 8)
	var search := LineEdit.new()
	search.placeholder_text = "filter by name"
	search.custom_minimum_size.x = 220
	search.text_changed.connect(func(t: String): _filter = t.to_lower(); _rebuild())
	filter_row.add_child(search)
	outer.add_child(filter_row)
	outer.add_child(DccTheme.rule())

	_tabs = TabContainer.new()
	_tabs.size_flags_vertical = Control.SIZE_EXPAND_FILL
	outer.add_child(_tabs)

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

func _row(body: VBoxContainer, cols: Array, tooltip: String = "") -> void:
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 10)
	row.custom_minimum_size.y = 20
	row.tooltip_text = tooltip
	for i in cols.size():
		var l := DccTheme.mono_label(String(cols[i]), "text_dim" if i == 0 else "text", DccTheme.FS_SMALL)
		l.clip_text = true
		l.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		l.size_flags_stretch_ratio = 2.0 if i == 0 else 1.0
		row.add_child(l)
	body.add_child(row)

func _header_row(body: VBoxContainer, cols: Array) -> void:
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 10)
	for i in cols.size():
		var l := DccTheme.mono_label(String(cols[i]), "text_faint", DccTheme.FS_MICRO, 1, true)
		l.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		l.size_flags_stretch_ratio = 2.0 if i == 0 else 1.0
		row.add_child(l)
	body.add_child(row)
	body.add_child(DccTheme.rule())

func _rebuild() -> void:
	_rebuild_settlements()
	_rebuild_provinces()
	_rebuild_trade()

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
	for r in rows:
		var d: Dictionary = r
		var name := String(d.get("name", "?"))
		if _filter != "" and name.to_lower().find(_filter) < 0:
			continue
		shown += 1
		_row(body, [name, String(d.get("kind", "?")).capitalize(), str(int(d.get("population", 0))),
			str(int(d.get("faction", 0))), "yes" if d.get("coastal", false) else "no",
			"yes" if d.get("capital", false) else "no"])
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
	for p in provinces:
		var d: Dictionary = p
		var name := String(d.get("name", "?"))
		if _filter != "" and name.to_lower().find(_filter) < 0:
			continue
		shown += 1
		var cap_idx := int(d.get("capital_settlement_index", -1))
		var cap_name := "—"
		if cap_idx >= 0 and cap_idx < settlements.size():
			cap_name = String((settlements[cap_idx] as Dictionary).get("name", "—"))
		_row(body, [name, str(int(d.get("faction", 0))), cap_name])
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
		_row(body, [name, ", ".join(ex) if ex.size() > 0 else "—", ", ".join(im) if im.size() > 0 else "—"])
	if shown == 0:
		DccWidgets.note(body, "No settlement matches \"%s\" with a trade relationship." % _filter if _filter != "" else "No settlement carries a trade relationship.")

func open() -> void:
	_rebuild()
	popup_centered()
