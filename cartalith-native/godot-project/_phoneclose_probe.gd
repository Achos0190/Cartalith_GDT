extends Node
## Phone probe for two OUTSTANDING_WORK.md "Phone and touch" rows (2026-11-12):
##   1. "After a MORE window closes the tab stays lit" -- MORE is MENU since MS-2/MS-4. A MENU row that opens a window
##      closes the overlay (`PhoneMenu._activate()`), and `DccShell._phone_tab` stayed "menu", so the bar kept the MENU
##      pill lit over a map the window then sat on. Fixed by `PhoneMenu.closed` -> `DccShell._on_phone_menu_closed()`.
##   2. "Phone Close button is 29 px, under the 44 px floor" -- every visible button named Close (an `AcceptDialog`'s
##      OK renamed by `DccWidgets.phone_window()`, a window's own footer chip, a sheet's cross) is measured in physical
##      pixels against 44 x `phone_scale()` and then TAPPED with a real touch pair, which must close its window.
##
## It boots the real shell as a phone in a `--vp`-sized host `SubViewport` (as `_ms4_probe.gd` does) and drives real
## pointer events. Windows are opened by the live menu bar's own rows (every enabled, non-checkable item whose title
## ends in an ellipsis, minus a stated deny list), the way MENU opens them, not by calling window classes directly.
##
## Windowed, with a wiped user dir, under the shared lock (`MISTAKES.md`, "Run a pixel probe"):
##   godot --path . --resolution 1600x1000 _phoneclose_probe.tscn -- --vp 1080x2340 --density 2.8125 --force-touch
##   godot --path . --resolution 1600x1000 _phoneclose_probe.tscn -- --vp 1080x1920 --density 3 --force-touch   (360 dp)
## Arguments read by THIS file: `--vp WxH`. Accepted because the shell reads them: `--density D`, `--force-touch`.
## Anything else aborts with exit 2. Result: `RESULT: PASS|FAIL (<n> failed, <m> checks)`. Exit 0 pass, 1 fail, 2 bad
## argument or headless.
##
## **Expected values are literals**: the floor is 44 (the canvas's accessibility minimum; `DccTheme.PHONE_TAP_MIN` is
## asserted equal to the literal 44 first so the probe cannot be satisfied by lowering the constant), and the tab ink
## is compared with the shell's own `accent` token.

## Rows never activated: they leave the process, change the user's files, or open a native dialog this probe cannot close.
const DENY: Array = ["Exit", "Quit", "Open project", "Open", "Save", "Import", "Export", "Delete", "Clear", "Forget",
	"Replace", "Browse", "Reveal", "Rescan", "New world", "Generate", "Reset", "Revert", "Close world"]
var _fails := 0
var _checks := 0
var _vp: SubViewport
var _app: Node

func _ok(cond: bool, what: String) -> void:
	_checks += 1
	if not cond:
		_fails += 1
		print("FAIL: ", what)
	else:
		print("ok:   ", what)

func _frames(n: int) -> void:
	for k in n:
		await get_tree().process_frame

func _arg(name: String, dflt: String) -> String:
	var args := OS.get_cmdline_user_args()
	var k := args.find(name)
	return String(args[k + 1]) if k >= 0 and k + 1 < args.size() else dflt

func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		print("ABORT: run WINDOWED")
		get_tree().quit(2)
		return
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		var a := String(args[i])
		if a == "--force-touch":
			i += 1
		elif a in ["--vp", "--density"]:
			i += 2
		else:
			print("ABORT: unknown argument ", a)
			get_tree().quit(2)
			return
	var parts := _arg("--vp", "").split("x")
	if parts.size() != 2:
		print("ABORT: needs --vp WxH")
		get_tree().quit(2)
		return
	await _run(int(parts[0]), int(parts[1]))
	print("RESULT: %s (%d failed, %d checks)" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)

func _touch(p: Vector2, down: bool) -> void:
	var t := InputEventScreenTouch.new()
	t.index = 0
	t.pressed = down
	t.position = p
	_vp.push_input(t)
	var m := InputEventMouseButton.new()
	m.button_index = MOUSE_BUTTON_LEFT
	m.pressed = down
	m.position = p
	m.global_position = p
	m.button_mask = MOUSE_BUTTON_MASK_LEFT if down else 0
	_vp.push_input(m)
	await _frames(2 if down else 3)

func _tap(p: Vector2) -> void:
	await _touch(p, true)
	await _touch(p, false)

func _settle() -> void:
	await get_tree().create_timer(0.3).timeout
	await _frames(3)

func _cell(id: String) -> Dictionary:
	return _app._phone_tab_cells.get(id, {}) as Dictionary

func _lit(id: String) -> bool:
	## The ink the bar actually draws: the caption's font colour is the accent only on the lit tab.
	var lbl := _cell(id).get("label") as Label
	return lbl != null and lbl.get_theme_color("font_color").is_equal_approx(DccTheme.c("accent"))

func _lit_set() -> Array:
	var out: Array = []
	for k in _app._phone_tab_cells.keys():
		if _lit(String(k)):
			out.append(String(k))
	return out

func _centre(id: String) -> Vector2:
	var b := _cell(id).get("button") as Button
	return b.get_global_rect().get_center() if b != null else Vector2(-1, -1)

# ---- Close-button census ------------------------------------------------------------------------------------

func _is_close(b: Button) -> bool:
	var t := b.text.strip_edges()
	return b.is_visible_in_tree() and not b.disabled and (t == "Close" or t.begins_with("Close ") or t == "✕"
		or b.tooltip_text == "Close" or b.accessibility_name == "Close" or b.accessibility_name == "Close menu")

func _scan(n: Node, out: Array) -> void:
	if n is Button and _is_close(n as Button):
		out.append(n)
	for c in n.get_children(true):
		_scan(c, out)

## Window units to physical pixels: a control inside an embedded `Window` is laid out in that window's content-scale
## units; one in the host viewport is already physical.
func _factor(b: Control) -> float:
	var w := b.get_viewport()
	if w is Window and w != _app.get_window():
		return (w as Window).content_scale_factor
	return 1.0

func _phys_h(b: Control) -> float:
	return b.size.y * _factor(b)

func _floor_px() -> float:
	return 44.0 * float(_app.phone_scale()) - 1.0

func _visible_windows(n: Node, out: Array) -> void:
	if n is Window and (n as Window).visible and n != _app.get_window():
		out.append(n)
	for c in n.get_children():
		_visible_windows(c, out)

func _hide_all_windows() -> void:
	var ws: Array = []
	_visible_windows(_app, ws)
	for w in ws:
		(w as Window).hide()

func _menu_rows(popup: PopupMenu, out: Array, path: String) -> void:
	popup.about_to_popup.emit()
	for i in popup.item_count:
		if popup.is_item_separator(i) or popup.is_item_disabled(i):
			continue
		var t := popup.get_item_text(i).strip_edges()
		var sn := popup.get_item_submenu(i)
		if sn != "":
			var sub := popup.get_node_or_null(NodePath(sn)) as PopupMenu
			if sub != null:
				_menu_rows(sub, out, path + " > " + t)
			continue
		if popup.is_item_checkable(i) or popup.is_item_radio_checkable(i):
			continue
		if not t.ends_with("…"):
			continue
		var denied := false
		for d in DENY:
			denied = denied or t.begins_with(String(d))
		if not denied:
			out.append({"popup": popup, "i": i, "t": path + " > " + t})

func _row_has_text(n: Node, want: String) -> bool:
	if n is Label and (n as Label).text == want:
		return true
	for c in n.get_children():
		if _row_has_text(c, want):
			return true
	return false

func _all_rows() -> Array:
	var rows: Array = []
	for c in _app.menu_bar_row.get_children():
		if c is MenuButton:
			_menu_rows((c as MenuButton).get_popup(), rows, (c as MenuButton).text)
	return rows

func _fire(r: Dictionary) -> void:
	var p: PopupMenu = r["popup"]
	p.id_pressed.emit(p.get_item_id(r["i"]))
	p.index_pressed.emit(r["i"])

func _run(vw: int, vh: int) -> void:
	DisplayServer.window_set_size(Vector2i(mini(vw, 1600), mini(vh, 1000)))
	_vp = SubViewport.new()
	_vp.size = Vector2i(vw, vh)
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	_app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(_app)
	await get_tree().create_timer(1.6).timeout
	if "open_project_dialog" in _app and _app.open_project_dialog != null:
		_app.open_project_dialog.hide()
	if "phone_project_picker" in _app and _app.phone_project_picker != null:
		_app.phone_project_picker.hide()
	await _frames(6)
	_ok(DccTheme.is_phone() and _app._phone_root != null, "boots as a phone")
	_ok(DccTheme.PHONE_TAP_MIN == 44, "the shell's tap floor constant is the literal 44")
	if not DccTheme.is_phone():
		return
	await _tab_row()
	await _close_row()

# ---- Row 1: the tab -----------------------------------------------------------------------------------------

func _tab_row() -> void:
	var before := _lit_set()
	_ok(before == ["domain"], "at rest exactly the domain tab is lit (%s)" % str(before))
	await _tap(_centre("menu"))
	await _settle()
	_ok(_app._phone_menu.is_open() and _lit_set() == ["menu"], "a tap on MENU opens it and lights only MENU (%s)" % str(_lit_set()))
	## MENU's root has no cross of its own: the way out is the lit tab itself.
	await _tap(_centre("menu"))
	await _settle()
	_ok(not _app._phone_menu.is_open(), "a second tap on the lit MENU tab closes MENU")
	_ok(_lit_set() == ["domain"], "closing MENU by its tab un-lights it (%s)" % str(_lit_set()))
	## A screen inside MENU has a cross; a real tap on it closes the whole overlay.
	await _tap(_centre("menu"))
	await _settle()
	var entered := false
	for c in _app._phone_menu._screen_body.get_children():
		if c is PanelContainer and _row_has_text(c, "Help"):
			await _tap(_host_centre(c))
			await _settle()
			entered = true
			break
	_ok(entered, "the Help row is on MENU's root and a real tap on it enters it")
	var cross: Button = _app._phone_menu._screen_close
	_ok(cross.is_visible_in_tree(), "the screen's cross is on screen")
	await _tap(_host_centre(cross))
	await _settle()
	_ok(not _app._phone_menu.is_open(), "a real tap on the screen's cross closes MENU")
	_ok(_lit_set() == ["domain"], "closing MENU by a screen's cross un-lights it (%s)" % str(_lit_set()))
	## Android back from the root closes it too.
	await _tap(_centre("menu"))
	await _settle()
	_app._phone_menu.go_back()
	await _settle()
	_ok(_lit_set() == ["domain"], "closing MENU by back un-lights it (%s)" % str(_lit_set()))
	var rows := _all_rows()
	var r: Dictionary = {}
	for x in rows:
		if String(x["t"]).begins_with("Help"):
			r = x
			break
	_ok(not r.is_empty(), "Help has a window-opening row (%d rows overall)" % rows.size())
	if r.is_empty():
		return
	var pm: Node = _app._phone_menu
	await _tap(_centre("menu"))
	await _settle()
	_fire(r)
	pm.close()  ## what `_activate()` does after the signals
	await _settle()
	var ws: Array = []
	_visible_windows(_app, ws)
	_ok(not ws.is_empty(), "'%s' opened a window" % r["t"])
	_ok(not pm.is_open() and _lit_set() == ["domain"], "window up, MENU shut: the bar lights the domain tab, not MENU (%s)" % str(_lit_set()))
	_hide_all_windows()
	await _settle()
	_ok(_lit_set() == ["domain"], "after the window closes only the domain tab is lit (%s)" % str(_lit_set()))
	await _tap(_centre("map"))
	await _settle()
	_ok(_lit_set() == ["map"], "MAP lit after a tap (%s)" % str(_lit_set()))
	await _tap(_centre("menu"))
	await _settle()
	_fire(r)
	pm.close()
	await _settle()
	_hide_all_windows()
	await _settle()
	_ok(_lit_set() == ["map"], "MAP comes back lit after a MENU row's window closes (%s)" % str(_lit_set()))
	await _tap(_centre("story"))
	await _settle()
	_ok(_lit_set() == ["story"], "a tap on STORY lights only STORY (%s)" % str(_lit_set()))
	await _tap(_centre("story"))
	await _settle()
	_ok(_lit_set() == ["map"], "a second tap on STORY closes it and gives MAP back (%s)" % str(_lit_set()))
	await _tap(_centre("domain"))
	await _settle()

# ---- Row 2: the Close buttons -------------------------------------------------------------------------------

func _close_row() -> void:
	var at_rest: Array = []
	_scan(_app, at_rest)
	for b in at_rest:
		_ok(_phys_h(b) >= _floor_px(), "shell at rest: Close '%s' is %.1f px tall, floor %.1f" % [(b as Button).text, _phys_h(b), _floor_px()])
	await _tap(_centre("menu"))
	await _settle()
	var in_menu: Array = []
	_scan(_app, in_menu)
	for b in in_menu:
		_ok(_phys_h(b) >= _floor_px(), "MENU open: Close '%s' is %.1f px tall, floor %.1f" % [(b as Button).text, _phys_h(b), _floor_px()])
	_app._phone_menu.close()
	await _settle()
	var rows := _all_rows()
	print("window rows found: ", rows.size())
	var seen := [0, 0]  ## windows opened, windows closed by a tap
	for r in rows:
		_hide_all_windows()
		_fire(r)
		await _settle()
		await _probe_windows(String(r["t"]), seen)
	## Windows no menu row reaches by an ellipsis title: opened through the shell's own entry points, on a small world.
	var bridge = _app.get("bridge")
	bridge.generate({"seed": 24601, "width_km": 600.0, "grid_w": 128, "grid_h": 96, "archetype": "", "villages": true, "sea_level": 0.45})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	bridge.civ_populate()
	await _frames(10)
	var direct: Array = [
		["open_assistant_panel", []], ["open_assistant_settings", []], ["open_quick_bar_window", []],
		["open_place_editor", [0]], ["open_city_viewer", [0]], ["open_faction_roster", []], ["open_settlement_types", []],
		["open_saved_journeys", []], ["open_conflicts_list", []], ["open_credits", []], ["open_about", []],
		["open_data_manager", []], ["open_asset_library", []], ["open_places_without_notes", []],
		["open_travel_library", []], ["open_world_data", []], ["open_gen_info", []], ["open_shortcuts", []],
		["open_storage_locations", []], ["open_export_dialog", []], ["open_context_tab_window", ["note"]]]
	for d in direct:
		_hide_all_windows()
		if _app.has_method(String(d[0])):
			_app.callv(String(d[0]), d[1])
		else:
			print("note: no such entry point ", d[0])
			continue
		await _settle()
		await _probe_windows("direct " + String(d[0]), seen)
	print("windows opened: %d, closed by a tap: %d" % [seen[0], seen[1]])
	_ok(seen[0] >= 25, "the sweep reached at least 25 windows (%d)" % seen[0])
	_ok(seen[1] == seen[0], "every opened window with a Close was closed by a real tap on it (%d of %d)" % [seen[1], seen[0]])
	_hide_all_windows()

## Measure and tap the Close buttons of whatever windows are open now; `seen` is [opened, closed].
func _probe_windows(name: String, seen: Array) -> void:
	var ws: Array = []
	_visible_windows(_app, ws)
	if ws.is_empty():
		print("note: '%s' opened no window" % name)
		return
	seen[0] += 1
	var found: Array = []
	for w in ws:
		_scan(w, found)
	if found.is_empty():
		print("note: '%s' opened a window with no Close-named button" % name)
		seen[1] += 1
		return
	for b in found:
		_ok(_phys_h(b) >= _floor_px(), "%s: Close '%s' is %.1f px tall, floor %.1f" % [name, (b as Button).text, _phys_h(b), _floor_px()])
	## Try the buttons spelled "Close" first, then the bare crosses (an asset-library card holds several crosses that
	## are row removers, so a cross is tried only when nothing else closed the window).
	var order: Array = []
	for b in found:
		if (b as Button).text.begins_with("Close"):
			order.append(b)
	for b in found:
		if not order.has(b):
			order.append(b)
	var closed := false
	for b in order:
		await _tap(_host_centre(b))
		await _settle()
		var left: Array = []
		_visible_windows(_app, left)
		closed = left.size() < ws.size()
		if closed:
			break
	seen[1] += 1 if closed else 0
	_ok(closed, "%s: a real tap on Close closes the window" % name)

## A control inside an embedded window, in host-viewport pixels: the window's origin plus the control's centre scaled
## by the window's content-scale factor.
func _host_centre(b: Control) -> Vector2:
	var w := b.get_viewport()
	if w is Window and w != _app.get_window():
		return Vector2((w as Window).position) + b.get_global_rect().get_center() * (w as Window).content_scale_factor
	return b.get_global_rect().get_center()
