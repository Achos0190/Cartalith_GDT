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
	## Through `DccWidgets.action()` rather than a bare `Button`: a raw Button
	## draws Godot's stock rounded grey pill, which is not a shape this design
	## has anywhere. Caught by screenshot 2026-08-25.
	var more := DccWidgets.action(body,
		"Show %d more" % mini(PHONE_ROW_CAP, total - built), func():
			_row_cap += PHONE_ROW_CAP
			_rebuild())
	more.size_flags_horizontal = Control.SIZE_EXPAND_FILL

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
	## Without this the whole table sizes to its own minimum inside the
	## `ScrollContainer` and stops at about 54 % of the width -- measured
	## 774 px of 1440 on the phone capture, with every row rule ending in mid
	## air. `body` had the flag; the margin between it and the scroll did not.
	pad.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	for side in ["left", "top", "right", "bottom"]:
		pad.add_theme_constant_override("margin_" + side, 12)
	scroll.add_child(pad)
	var body := VBoxContainer.new()
	body.add_theme_constant_override("separation", 2)
	body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	pad.add_child(body)
	_tabs.add_child(scroll)
	return body

## `20655` -> `20 655`. A thin-space group, because the phone row's whole point
## is that a number is read rather than parsed, and 240 populations in a column
## of unbroken digits is the thing that makes a list look like a dump.
static func _thousands(n: int) -> String:
	var s := str(absi(n))
	var out := ""
	for i in s.length():
		if i > 0 and (s.length() - i) % 3 == 0:
			out += " "
		out += s[i]
	return ("-" if n < 0 else "") + out

func _clear(body: VBoxContainer) -> void:
	for c in body.get_children():
		body.remove_child(c)
		c.queue_free()

## **Phone list row, replacing PH-12's wrapped table (2026-08-25).**
##
## PH-12 kept all six columns and broke them over two lines. It measured well
## -- nothing clipped, nothing under the tap floor -- and it still read as a
## spreadsheet someone had folded in half: a bare `Class Population Faction
## Coastal Capital` band under a column header that no longer sat over any
## column, and five unlabelled values a reader had to count along to decode.
##
## There is **no canvas for this window**, on phone or anywhere else, so the
## replacement is derived rather than matched -- from the one phone list the
## design does draw, `design/Cartalith Android Phone.dc.html` screen
## `03 Category`, whose row is, verbatim:
##
##   display:flex; align-items:center; min-height:52px; padding:0 16px;
##   gap:12px; border-top:1px solid rgba(255,255,255,.06)
##     <span style="flex:1">Droplet hydraulic
##       <span style="display:block;font:9.5px 'IBM Plex Mono';color:#5f6468">
##         4 dials · last run 12 s ago</span></span>
##     <span style="color:#5f6468">›</span>
##
## So: a prose primary line, one Plex secondary line at 9.5 px in `#5f6468`
## carrying everything else as `·`-joined prose, a 52 dp minimum, a 16 dp
## gutter and a `.06` rule above. The five remaining settlement columns become
## that secondary line, each carrying its own word rather than its position
## ("capital · coastal" instead of a `yes` in the fifth slot). The chevron is
## the one thing deliberately NOT copied: it promises a drill-down these rows
## do not have, and drawing an affordance that does nothing is the failure
## this whole pass exists to find.
##
## The column header is dropped on phone with the columns -- see
## `_header_row()`.
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

## The `·`-joined secondary line. `subtitle` is passed in already worded by the
## caller, because only the caller knows that a `Capital` column's `yes` means
## the word "capital" and its `no` means the word is simply absent.
func _phone_row(body: VBoxContainer, primary: String, subtitle: String,
		tooltip: String) -> void:
	var rule := ColorRect.new()
	rule.color = DccTheme.c("line_soft")
	rule.custom_minimum_size.y = 1
	body.add_child(rule)

	var stack := VBoxContainer.new()
	stack.add_theme_constant_override("separation", 2)
	stack.tooltip_text = tooltip
	stack.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	## `min-height:52px` with `padding:0 16px` -- 52 dp of row and no vertical
	## padding at all, so the gutter is horizontal only. The tap floor plus 8
	## is exactly the canvas's 52.
	stack.custom_minimum_size.y = DccTheme.PHONE_TAP_MIN + 8
	var name_l := DccTheme.label(primary, "text", DccTheme.FS_BODY)
	name_l.clip_text = true
	name_l.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	stack.add_child(name_l)
	var sub := DccTheme.mono_label(subtitle, "text_ghost", DccTheme.FS_MICRO)
	sub.clip_text = true
	sub.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	stack.add_child(sub)

	var pad := MarginContainer.new()
	pad.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	pad.add_theme_constant_override("margin_left", 16)
	pad.add_theme_constant_override("margin_right", 16)
	pad.add_child(stack)
	body.add_child(pad)

func _row(body: VBoxContainer, cols: Array, tooltip: String = "",
		subtitle: String = "") -> void:
	if _phone:
		_phone_row(body, String(cols[0]),
			subtitle if subtitle != "" else " · ".join(cols.slice(1)), tooltip)
		return
	var flat := _cells(cols, 0, false)
	flat.tooltip_text = tooltip
	body.add_child(flat)

## Desktop keeps its column header. Phone drops it: `_row()` no longer draws
## columns there, so a header would label a table that isn't on screen -- which
## is exactly what the previous phone treatment did.
func _header_row(body: VBoxContainer, cols: Array) -> void:
	if _phone:
		return
	body.add_child(_cells(cols, 0, true))
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
		## The phone subtitle words its own columns. `Coastal: yes` and
		## `Capital: yes` are table cells; "coastal" and "capital" are facts,
		## and a fact that is false is simply not stated -- which is how the
		## canvas's own summary lines read ("4 dials · last run 12 s ago",
		## "dynamic lithology on"), and it drops two thirds of the noise.
		var kind_lc := String(d.get("kind", "?")).to_lower()
		var facts := PackedStringArray([
			kind_lc,
			"pop %s" % _thousands(int(d.get("population", 0))),
			"faction %d" % int(d.get("faction", 0)),
		])
		if d.get("coastal", false):
			facts.append("coastal")
		## `GUI_GAP_REGISTER.md` §50: the capital flag used to append
		## unconditionally, so a settlement whose *class* already is "capital"
		## read "capital · pop 20 708 · faction 4 · capital" -- the same word
		## twice. Drop it exactly when it would only repeat facts[0].
		if d.get("capital", false) and kind_lc != "capital":
			facts.append("capital")
		_row(body, [name, String(d.get("kind", "?")).capitalize(), str(int(d.get("population", 0))),
			str(int(d.get("faction", 0))), "yes" if d.get("coastal", false) else "no",
			"yes" if d.get("capital", false) else "no"],
			"", " · ".join(facts))
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
		var psub := "faction %d" % int(d.get("faction", 0))
		if cap_name != "—":
			psub += " · capital %s" % cap_name
		_row(body, [name, str(int(d.get("faction", 0))), cap_name], "", psub)
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
		var tparts := PackedStringArray()
		if ex.size() > 0:
			tparts.append("exports " + ", ".join(ex))
		if im.size() > 0:
			tparts.append("imports " + ", ".join(im))
		_row(body, [name, ", ".join(ex) if ex.size() > 0 else "—",
			", ".join(im) if im.size() > 0 else "—"], "", " · ".join(tparts))
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
