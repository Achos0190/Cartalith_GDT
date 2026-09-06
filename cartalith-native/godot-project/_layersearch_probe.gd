extends Node
## Committed probe for CA-09 -- the Layers popover's search field
## (`shell/layers_popover.gd`).
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _layersearch_probe.tscn
##
## **Run WINDOWED, not `--headless`.** Nothing here asserts on pixels, but the
## field-focus check (§5) needs a real window to give a `LineEdit` focus, and
## `open()` calls `popup()` on a subwindow. Density is printed in §0 beside
## every count, per `MISTAKES.md`'s "name the density" row -- this machine
## boots light/pointer and the row heights differ on tablet.
##
## Committed, like every probe scene in this folder -- `STATUS.md`'s F8 row
## ("Test harnesses committed"): kept as the evidence for the pass that wrote
## it, not deleted after it.
##
## What it proves, in order:
##   0. density and the two list sizes, measured from the lists themselves
##   1. empty query shows EVERYTHING, in two bands, with the full count
##   2. a query narrows BOTH bands, and the bands stay separate
##   3. a query matching one list draws one band, never an empty band pair
##   4. no match says so ONCE
##   5. the hotkey property: digits 1-8 mean the same view before and after a
##      filter, INCLUDING a digit whose row the filter dropped
##   6. a focused field keeps its own digits

var _app: Node
var _bridge
var _pop
var _host
var _fail := 0


func _p(s: String) -> void:
	print("LAYERSEARCH  %s" % s)


func _bad(s: String) -> void:
	_fail += 1
	print("LAYERSEARCH  FAIL  %s" % s)


func _ok(cond: bool, s: String) -> void:
	if cond:
		_p("ok    %s" % s)
	else:
		_bad(s)


func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame


func _walk(n: Node, out: Array) -> void:
	out.append(n)
	for c in n.get_children(true):
		_walk(c, out)


## Every band header the popover currently draws, in tree order. A band header
## is `DccTheme.header(title, "")` -- mono caps with NO sigil -- and an engine
## group heading is the same call WITH the `§`, so the two are told apart by
## the sigil exactly the way a reader tells them apart.
func _bands() -> PackedStringArray:
	var out := PackedStringArray()
	var nodes: Array = []
	_walk(_pop, nodes)
	for n in nodes:
		if n is Label and not (n is LineEdit):
			var t := String((n as Label).text)
			if t == t.to_upper() and t.strip_edges() != "" and not t.begins_with("§") \
					and (t == "VISIBLE LAYERS" or t == "DATA OVERLAYS"):
				out.append(t)
	return out


## The `§`-sigil group headings under DATA OVERLAYS (Base, Climate, ...).
func _groups_drawn() -> PackedStringArray:
	var out := PackedStringArray()
	var nodes: Array = []
	_walk(_pop, nodes)
	for n in nodes:
		if n is Label and String((n as Label).text).begins_with("§"):
			out.append(String((n as Label).text))
	return out


## Rows in the VISIBLE LAYERS band: `DccWidgets.toggle()` builds a `CheckBox`,
## and nothing else in this popover does, so the class alone identifies them.
func _live_rows() -> PackedStringArray:
	var out := PackedStringArray()
	var nodes: Array = []
	_walk(_pop, nodes)
	for n in nodes:
		if n is CheckBox:
			var row := (n as CheckBox).get_parent()
			if row != null and row.get_child_count() > 0 and row.get_child(0) is Label:
				out.append(String((row.get_child(0) as Label).text))
	return out


## Rows in the DATA OVERLAYS band: plain `Button`s that are not `CheckBox`,
## not the phone Close, and carry text.
func _overlay_rows() -> PackedStringArray:
	var out := PackedStringArray()
	var nodes: Array = []
	_walk(_pop, nodes)
	for n in nodes:
		if n is Button and not (n is CheckBox) and not (n is OptionButton):
			var t := String((n as Button).text).strip_edges()
			if t != "" and t != "Close":
				out.append(t)
	return out


## The `CheckBox` of one VISIBLE LAYERS row, by the label beside it.
func _live_check(label: String) -> CheckBox:
	var nodes: Array = []
	_walk(_pop, nodes)
	for n in nodes:
		if n is CheckBox:
			var row := (n as CheckBox).get_parent()
			if row != null and row.get_child_count() > 0 and row.get_child(0) is Label \
					and String((row.get_child(0) as Label).text) == label:
				return n as CheckBox
	return null


## Depth-first search for the node whose script is `script_file` -- the same
## helper `_newsurf_probe.gd` uses to reach a workspace.
func _find(n: Node, script_file: String) -> Node:
	if n.get_script() != null and String(n.get_script().resource_path).ends_with(script_file):
		return n
	for c in n.get_children(true):
		var r := _find(c, script_file)
		if r != null:
			return r
	return null


func _notes() -> String:
	var parts := PackedStringArray()
	var nodes: Array = []
	_walk(_pop._list, nodes)
	for n in nodes:
		if n is Label:
			parts.append(String((n as Label).text))
	return "\n".join(parts)


func _count_text() -> String:
	return String(_pop._count.text)


func _type(q: String) -> void:
	_pop._field.text = q
	## `text_changed` does not fire on a programmatic assignment -- the same
	## note `DccShell._set_search_query()` carries. The rebuild is the update.
	_pop._query = q
	_pop.rebuild()
	await _frames(2)


## Fires the real dispatch path rather than calling `_on_pick()`: an
## `InputEventKey` on the physical digit, through `_input()`, exactly as
## `_register_hotkeys()` bound it.
func _press_digit(d: int) -> void:
	var ev := InputEventKey.new()
	ev.physical_keycode = KEY_1 + d
	ev.pressed = true
	_pop._input(ev)
	await _frames(2)


func _generate(seed: int, gw: int, gh: int, km: float) -> void:
	_bridge.generate({
		"seed": seed, "width_km": km, "grid_w": gw, "grid_h": gh,
		"archetype": "", "villages": true, "sea_level": 0.45,
	})
	while _bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(0.8).timeout


func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 900.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func(): _p("WATCHDOG"); get_tree().quit(3))
	wd.start()

	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.0).timeout
	_bridge = _app.bridge
	await _generate(483920, 384, 288, 2400.0)
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _frames(8)
	_pop = _app.layers_popover
	_host = _app.viewport

	# ================================================================ §0 setup
	var vp := get_viewport().get_visible_rect().size
	_p("=== §0 density and the two lists ===")
	_p("viewport %dx%d  is_tablet=%s  is_phone=%s  headless=%s" % [
		int(vp.x), int(vp.y), str(DccTheme.is_tablet()),
		str(_app.has_method("is_phone") and _app.is_phone()),
		str(DisplayServer.get_name() == "headless")])
	if DisplayServer.get_name() == "headless":
		_bad("run WINDOWED -- see this probe's header")

	var groups: Array = _bridge.debug_layers()
	var overlay_total := 0
	for g in groups:
		overlay_total += ((g as Dictionary)["items"] as Array).size()
	var live_total: int = CartographyWorkspace.LIVE_LAYERS.size()
	_p("LIVE_LAYERS=%d  debug_layers items=%d  groups=%d  total=%d" % [
		live_total, overlay_total, groups.size(), live_total + overlay_total])
	_ok(live_total > 0 and overlay_total > 0, "both lists are non-empty")

	_pop.open()
	await _frames(4)

	# ====================================================== §1 empty = all
	_p("=== §1 an empty query shows everything ===")
	_p("bands=%s" % str(_bands()))
	_p("count=%s" % _count_text())
	_p("live rows=%d  overlay rows=%d  group headings=%d" % [
		_live_rows().size(), _overlay_rows().size(), _groups_drawn().size()])
	_ok(_bands().size() == 2, "two bands drawn")
	_ok(_bands()[0] == "VISIBLE LAYERS" and _bands()[1] == "DATA OVERLAYS",
		"VISIBLE LAYERS first, DATA OVERLAYS second")
	_ok(_live_rows().size() == live_total, "every LIVE_LAYERS row drawn")
	_ok(_overlay_rows().size() == overlay_total, "every overlay row drawn")
	_ok(_count_text() == "%d of %d layers" % [
		live_total + overlay_total, live_total + overlay_total],
		"count reads N of N with an empty field")
	_ok(_groups_drawn().size() == groups.size(),
		"the engine's own group headings survive, all %d" % groups.size())

	## The bands are NOT one flat list: every live row is a CheckBox and every
	## overlay row is a plain Button, so a row cannot inherit the other band's
	## behaviour by construction, not by convention.
	_ok(not _live_rows().is_empty() and not _overlay_rows().is_empty(),
		"the two bands are drawn from two different row factories")

	var hotkeys_before: Array = (_pop._hotkey_ids as Array).duplicate()
	_p("hotkeys 1-8 before any filter: %s" % str(hotkeys_before))
	_ok(hotkeys_before.size() == 8, "eight digits are live")

	# ============================================ §2 a query narrows both bands
	_p("=== §2 \"political\" narrows both bands ===")
	await _type("political")
	_p("bands=%s" % str(_bands()))
	_p("live=%s" % str(_live_rows()))
	_p("overlay=%s" % str(_overlay_rows()))
	_p("count=%s" % _count_text())
	_ok(_bands().size() == 2, "both bands still drawn, still separate")
	_ok(_live_rows().size() == 2, "2 live rows survive (provinces, territory)")
	_ok(_overlay_rows().size() == 1, "1 overlay row survives (Political control)")
	_ok(_count_text() == "3 of %d layers" % [live_total + overlay_total],
		"count is 3 of the whole pool, not 3 of 3")
	_ok(_groups_drawn().size() == 1,
		"only the group that still has a row keeps its heading")

	_p("--- id-only match: \"popdensity\" is an id, never a printed label ---")
	await _type("popdensity")
	_p("live=%s overlay=%s count=%s" % [
		str(_live_rows()), str(_overlay_rows()), _count_text()])
	_ok(_overlay_rows().size() == 1 and _live_rows().is_empty(),
		"matching on the engine id alone finds the row")

	# ======================================= §3 one band, never an empty pair
	_p("=== §3 a one-list query draws ONE band ===")
	await _type("town layouts")
	_p("bands=%s live=%s overlay=%s count=%s" % [
		str(_bands()), str(_live_rows()), str(_overlay_rows()), _count_text()])
	_ok(_bands().size() == 1 and _bands()[0] == "VISIBLE LAYERS",
		"only the band with rows is drawn")
	_ok(_overlay_rows().is_empty(), "no empty DATA OVERLAYS band")

	await _type("strahler")
	_p("bands=%s live=%s overlay=%s" % [
		str(_bands()), str(_live_rows()), str(_overlay_rows())])
	_ok(_bands().size() == 1 and _bands()[0] == "DATA OVERLAYS",
		"and the same the other way round")

	# ========================================== §4 no match says so ONCE
	_p("=== §4 a query matching nothing ===")
	await _type("zzzz")
	var notes := _notes()
	_p("bands=%s count=%s" % [str(_bands()), _count_text()])
	_p("notes=%s" % notes.replace("\n", " | "))
	_ok(_bands().is_empty(), "no band pair drawn over an empty result")
	_ok(notes.findn("No layer matches") >= 0, "the no-match sentence is shown")
	_ok(notes.countn("No layer matches") == 1, "said once, not per band")
	_ok(_count_text() == "0 of %d layers" % [live_total + overlay_total],
		"count reads 0 of the pool")

	# ============================================ §5 THE HOTKEY PROPERTY
	_p("=== §5 digits 1-8 survive a filter ===")
	await _type("")
	_ok((_pop._hotkey_ids as Array) == hotkeys_before,
		"unfiltered digits unchanged after a round trip through the field")

	## Control: with no filter, digit 4 reaches the view it is badged with.
	var want: String = String(hotkeys_before[3])
	_host.set_debug_layer("off")
	await _frames(2)
	await _press_digit(3)
	_p("no filter: pressed 4 -> debug_view=%s (badge says %s)" % [
		_host.debug_view(), want])
	_ok(_host.debug_view() == want, "digit 4 reaches its badged view unfiltered")

	## Now filter that row OUT and press the same digit. This is the whole
	## point: `_hotkey_ids` is built over the unfiltered list, so the digit
	## still means the same view even with no row on screen to click.
	await _type("political")
	_p("filtered: hotkeys=%s" % str(_pop._hotkey_ids))
	_ok((_pop._hotkey_ids as Array) == hotkeys_before,
		"filtering did NOT rebind the eight digits")
	var drawn := _overlay_rows()
	_p("rows on screen while filtered: %s" % str(drawn))
	_host.set_debug_layer("off")
	await _frames(2)
	await _press_digit(3)
	_p("filtered: pressed 4 -> debug_view=%s (want %s)" % [_host.debug_view(), want])
	_ok(_host.debug_view() == want,
		"digit 4 still reaches its view with its row filtered off screen")

	## And every one of the eight, not just the fourth.
	var reached := 0
	for i in 8:
		_host.set_debug_layer("off")
		await _frames(1)
		await _press_digit(i)
		if _host.debug_view() == String(hotkeys_before[i]):
			reached += 1
		else:
			_bad("digit %d -> %s, wanted %s (filtered)" % [
				i + 1, _host.debug_view(), String(hotkeys_before[i])])
	_p("all eight digits under a filter: %d/8 reached their view" % reached)

	# ================================= §6 a focused field keeps its digits
	_p("=== §6 a focused field owns its own keystrokes ===")
	await _type("")
	_host.set_debug_layer("off")
	await _frames(2)
	_pop._field.grab_focus()
	await _frames(3)
	_p("field has_focus=%s" % str(_pop._field.has_focus()))
	if not _pop._field.has_focus():
		_p("note: focus not granted in this run -- the guard could not be exercised")
	else:
		await _press_digit(3)
		_p("focused: pressed 4 -> debug_view=%s (want off)" % _host.debug_view())
		_ok(_host.debug_view() == "off",
			"a digit typed into the field does not swap the layer")
		_pop._field.release_focus()
		await _frames(2)
		await _press_digit(3)
		_ok(_host.debug_view() == want, "and the digit works again once focus leaves")

	# ============================ §7 the field's own size, at this density
	##
	## Named beside the density, per `MISTAKES.md`'s "judge a measurement
	## against a floor" row: 44 is a TOUCH requirement, so on a pointer run the
	## number below is a drawn height with no floor to fail. Re-run this probe
	## as `--resolution 1080x2400 ... -- --force-touch` for the touch figure.
	_p("=== §7 the filter field, measured ===")
	await _type("")
	await _frames(3)
	var phone: bool = _app.has_method("is_phone") and _app.is_phone()
	_p("field h=%.1f  min_y=%.1f  density=%s  PHONE_TAP_MIN=%d" % [
		_pop._field.size.y, _pop._field.custom_minimum_size.y,
		("touch/phone" if phone else ("touch/tablet" if DccTheme.is_tablet()
			else "pointer")), DccTheme.PHONE_TAP_MIN])
	if phone:
		_ok(_pop._field.size.y >= float(DccTheme.PHONE_TAP_MIN),
			"the field meets §13's tap floor on a phone")
	elif DccTheme.is_tablet():
		## Measured at 22.0 before the `btn_min_h` floor was added -- see the
		## `DccWidgets.well(_field)` block in `layers_popover.gd`.
		_ok(_pop._field.size.y >= float(DccTheme.role_px("btn_min_h")),
			"the field meets the touch floor (%d px) on a tablet"
			% DccTheme.role_px("btn_min_h"))

	# ================= §8 one switch, two surfaces -- not two switches
	##
	## The VISIBLE LAYERS band writes through `ViewportHost.set_layer_visible()`
	## rather than holding its own state, so CARTO's rail dock -- which listens
	## to `layer_visibility_changed` -- has to follow it. This is the check that
	## makes "the same switch as Cartography ▸ Layers" in the foot note a
	## measured claim rather than an intention.
	_p("=== §8 the popover's toggle moves CARTO's own checkbox ===")
	var carto := _find(_app, "cartography_workspace.gd")
	if carto == null:
		_bad("no cartography workspace to check against")
	else:
		var dock_cb: CheckBox = (carto._layer_checks as Dictionary).get("settlements")
		var pop_cb := _live_check("Settlements")
		if dock_cb == null or pop_cb == null:
			_bad("missing a Settlements checkbox: dock=%s popover=%s" % [
				str(dock_cb != null), str(pop_cb != null)])
		else:
			var was: bool = _host.layer_visible("settlements")
			_p("before: engine=%s dock=%s popover=%s" % [
				str(was), str(dock_cb.button_pressed), str(pop_cb.button_pressed)])
			_ok(pop_cb.button_pressed == was, "the popover row starts on the engine's value")
			pop_cb.button_pressed = not was
			await _frames(3)
			_p("after: engine=%s dock=%s" % [
				str(_host.layer_visible("settlements")), str(dock_cb.button_pressed)])
			_ok(_host.layer_visible("settlements") == (not was),
				"the popover row wrote through to ViewportHost")
			_ok(dock_cb.button_pressed == (not was),
				"CARTO ▸ Layers followed it without a world change")
			_host.set_layer_visible("settlements", was)
			await _frames(2)

	# ============================================================== verdict
	if _fail == 0:
		_p("ALL PASS")
	else:
		_p("FAILURES: %d" % _fail)
	get_tree().quit(0 if _fail == 0 else 1)
