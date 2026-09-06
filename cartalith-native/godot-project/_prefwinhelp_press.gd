extends Node
## LANE PrefsWinHelp -- DOES THE ROW DO ANYTHING? (press probe)
##
##   godot --path . --resolution 1600x900 _prefwinhelp_press.tscn
##
## Emits `id_pressed` on the real popups and asserts an OBSERVABLE state change
## outside the menu. A row whose press moves nothing is the failure this
## register keeps finding. Deliberately skips rows that touch the filesystem
## (Clear caches, atlas import/export, Documentation) -- those are audited by
## reading their handler, not by firing them.
var _vp: SubViewport
var _fail := 0
var _app: Node

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ok(nm: String, got, want) -> void:
	var good: bool = str(got) == str(want)
	if not good:
		_fail += 1
	print("  ", "ok  " if good else "FAIL", " ", nm, "   got=", got, " want=", want)

func _find(n: Node, out: Array) -> void:
	if n is MenuButton:
		out.append(n)
	for c in n.get_children(true):
		_find(c, out)

func _menu(title: String) -> PopupMenu:
	var bar: Array = []
	_find(_app, bar)
	for mb in bar:
		if mb.text == title:
			return mb.get_popup()
	return null

func _sub(p: PopupMenu, label: String) -> PopupMenu:
	for i in p.item_count:
		if p.get_item_text(i) == label:
			var s := p.get_item_submenu(i)
			if s != "":
				return p.get_node_or_null(NodePath(s)) as PopupMenu
	return null

func _ready() -> void:
	await _frames(2)
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load")
		get_tree().quit(1)
		return
	_vp = SubViewport.new()
	_vp.size = Vector2i(1600, 900)
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	_app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(_app)
	await _frames(60)
	print("[BOOT] shell up")

	var win := _menu("Window")
	var pref := _menu("Preferences")
	var help := _menu("Help")

	print("\n=== WINDOW - the five region toggles ===")
	for rid in [60, 61, 62, 63, 64]:
		var idx := win.get_item_index(rid)
		win.id_pressed.emit(rid)
		await _frames(2)
		var after_checked := win.is_item_checked(idx)
		win.id_pressed.emit(rid)
		await _frames(2)
		_ok("region %d toggled then restored (check state moved)" % rid,
			after_checked != win.is_item_checked(idx), true)

	print("\n=== WINDOW - Diagnostics overlay (66) ===")
	var vis0: bool = _app.resource_overlay.visible
	win.id_pressed.emit(66)
	await _frames(2)
	var vis1: bool = _app.resource_overlay.visible
	_ok("Diagnostics overlay visibility flipped", vis1 != vis0, true)
	win.id_pressed.emit(66)
	await _frames(2)
	_ok("and flipped back", _app.resource_overlay.visible, vis0)

	print("\n=== WINDOW - Workspace submenu ===")
	var wsp := _sub(win, "Workspace")
	var d0: String = _app.active_domain()
	wsp.id_pressed.emit(1)
	await _frames(4)
	var d1: String = _app.active_domain()
	_ok("Workspace row 1 changed the active domain", d1 != d0, true)
	print("     info domain %s -> %s" % [d0, d1])
	wsp.id_pressed.emit(0)
	await _frames(4)
	_ok("Workspace row 0 restored it", _app.active_domain(), d0)

	print("\n=== WINDOW - Layouts (seeded) ===")
	var lay := _sub(win, "Layouts")
	lay.about_to_popup.emit()
	await _frames(2)
	var names := []
	for i in lay.item_count:
		if not lay.is_item_separator(i):
			names.append(lay.get_item_text(i))
	print("     info layout rows: ", names)
	_ok("four task layouts are seeded", DccSettings.layout_names().size() >= 4, true)
	var dom_before: String = _app.active_domain()
	lay.id_pressed.emit(643)
	await _frames(6)
	print("     info after pressing layout row id 643: domain=", _app.active_domain())
	_ok("a saved layout press moved the domain", _app.active_domain() != dom_before, true)
	lay.id_pressed.emit(640)
	await _frames(4)

	print("\n=== PREFERENCES - Units radio (54/55/604) ===")
	var un := _sub(pref, "Units")
	var u0 := DccSettings.units_mode()
	un.id_pressed.emit(55)
	await _frames(2)
	_ok("Miles selected -> units_mode()", DccSettings.units_mode(), "mi")
	_ok("DccUnits.suffix() followed", DccUnits.suffix(), "mi")
	un.id_pressed.emit(604)
	await _frames(2)
	_ok("Nautical miles selected", DccSettings.units_mode(), "nmi")
	_ok("suffix is the symbol, not the key", DccUnits.suffix(), "nm")
	un.id_pressed.emit(54)
	await _frames(2)
	_ok("Kilometres restored", DccSettings.units_mode(), "km")
	_ok("mode restored to what it was", DccSettings.units_mode(), u0)

	print("\n=== PREFERENCES - Theme radio (51/52/58) ===")
	var th := _sub(pref, "Theme")
	var t0 := DccSettings.theme_mode()
	th.id_pressed.emit(51)
	await _frames(4)
	_ok("Dark selected -> theme_mode()", DccSettings.theme_mode(), "dark")
	_ok("DccTheme.is_dark() followed", DccTheme.is_dark(), true)
	th.id_pressed.emit(52)
	await _frames(4)
	_ok("Light selected", DccSettings.theme_mode(), "light")
	_ok("DccTheme.is_dark() followed", DccTheme.is_dark(), false)
	th.id_pressed.emit(58)
	await _frames(4)
	print("     info Follow system -> mode=%s  dark=%s  os_supported=%s"
		% [DccSettings.theme_mode(), DccTheme.is_dark(), DisplayServer.is_dark_mode_supported()])
	_ok("Follow system stored", DccSettings.theme_mode(), "system")
	th.id_pressed.emit(52 if t0 == "light" else 51)
	await _frames(4)

	print("\n=== PREFERENCES - Chunk debug overlay (74/75/76/80) ===")
	var dbg := _sub(pref, "Chunk debug overlay")
	for did in [74, 75, 76, 80]:
		var i := dbg.get_item_index(did)
		var c0 := dbg.is_item_checked(i)
		dbg.id_pressed.emit(did)
		await _frames(2)
		_ok("debug row %d flipped its check" % did, dbg.is_item_checked(i) != c0, true)
		dbg.id_pressed.emit(did)
		await _frames(2)

	print("\n=== PREFERENCES - Render quality / Undo budget / Relief exag ===")
	var q := _sub(pref, "Render quality")
	var q0 = _app.bridge.quality_tier()
	q.id_pressed.emit(0)
	await _frames(2)
	_ok("quality tier changed", _app.bridge.quality_tier() != q0, true)
	print("     info tier %s -> %s" % [q0, _app.bridge.quality_tier()])
	var ub := _sub(pref, "Undo history")
	var b0: Dictionary = _app.bridge.undo_stats()
	ub.id_pressed.emit(0)
	await _frames(2)
	var b1: Dictionary = _app.bridge.undo_stats()
	_ok("undo budget changed", int(b1.get("budget_bytes", -1)) != int(b0.get("budget_bytes", -2)), true)
	print("     info budget %s -> %s" % [b0.get("budget_bytes"), b1.get("budget_bytes")])
	var ex := _sub(pref, "Relief exaggeration")
	print("     info relief exag before: ", ("%s" % DccSettings.relief_exaggeration_default()) if DccSettings.has_relief_exaggeration_default() else "(unset)")
	ex.id_pressed.emit(621)
	await _frames(2)
	print("     info relief exag after rung 621: ", ("%s" % DccSettings.relief_exaggeration_default()) if DccSettings.has_relief_exaggeration_default() else "(unset)")

	print("\n=== PREFERENCES - GPU acceleration checkbox (50) ===")
	var gi := pref.get_item_index(50)
	var g0: bool = bool(_app.bridge.param_get("use_gpu"))
	pref.id_pressed.emit(50)
	await _frames(2)
	_ok("use_gpu param flipped", bool(_app.bridge.param_get("use_gpu")) != g0, true)
	_ok("and the row's own check followed", pref.is_item_checked(gi), not g0)
	pref.id_pressed.emit(50)
	await _frames(2)

	print("\n=== HELP - the dialog rows ===")
	for pair in [[72, "shortcuts_dialog"], [73, "gen_info_dialog"]]:
		help.id_pressed.emit(int(pair[0]))
		await _frames(4)
		var n = _app.get(String(pair[1]))
		_ok("Help id %d opened %s" % [int(pair[0]), pair[1]], n != null and n.visible, true)
		if n != null and n.visible:
			n.hide()
		await _frames(2)
	for hid in [70, 71, 700]:
		var before := _count_visible_dialogs()
		help.id_pressed.emit(hid)
		await _frames(6)
		var after := _count_visible_dialogs()
		_ok("Help id %d put a window on screen" % hid, after > before, true)
		_hide_all_dialogs()
		await _frames(2)

	print("\n[RESULT] failures=", _fail)
	get_tree().quit(0)

func _count_visible_dialogs() -> int:
	var n := 0
	for w in _all_windows(_app, []):
		if (w as Window).visible:
			n += 1
	return n

func _hide_all_dialogs() -> void:
	for w in _all_windows(_app, []):
		if (w as Window).visible:
			(w as Window).hide()

func _all_windows(n: Node, out: Array) -> Array:
	if n is Window:
		out.append(n)
	for c in n.get_children(true):
		_all_windows(c, out)
	return out
