extends Node
## The deletion of the unreachable `QuickBarPhoneList` sheet (2026-11-16; MS-11's drawer superseded it, nothing opened it). Windowed,
## scratch user dir.
##
##   godot --path . _qbplist_probe.tscn -- --vp 1600x900                                       (desktop)
##   ... -- --vp 1700x1000 --density=1.0 --force-touch                                         (tablet)
##   ... -- --vp 1080x2340 --density 2.625 --force-touch                                       (phone)
##
## What it protects:
##   G  it is GONE: no script file, no registered global class, no `open_list` / `list_sheet` on the phone's controller, no
##      `quick_bar_phone_list` field on the app, and the class-swap list (`REPRESENTED_WINDOWS`, `LAZY_WINDOWS`) does not name it;
##   S  (phone) the row text it used to own still reaches the drawer (`QuickDrawer._sub_for`, moved there): a live pin's second
##      line is `<menu> menu`, a pin whose command is gone says why, the Assistant row is absent while the assistant is off.
## Exit 0 = held, 1 = a check failed, 2 = could not run.

var _fails := 0
var _checks := 0
var _vp: SubViewport
var _app: Node


func _ok(cond: bool, what: String) -> void:
	_checks += 1
	print(("  ok    " if cond else "  FAIL  ") + what)
	if not cond:
		_fails += 1


func _frames(n: int = 3) -> void:
	for _i in n:
		await get_tree().process_frame


func _arg(name: String, dflt: String) -> String:
	var a := OS.get_cmdline_user_args()
	for i in a.size():
		if a[i] == name and i + 1 < a.size():
			return a[i + 1]
	return dflt


func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		var a := String(args[i])
		if a == "--force-touch" or a.begins_with("--density="):
			i += 1
		elif a in ["--vp", "--density"]:
			i += 2
		else:
			print("ABORT: unrecognised argument %s" % a)
			get_tree().quit(2)
			return
	var parts := _arg("--vp", "").split("x")
	if parts.size() != 2:
		print("ABORT: needs --vp WxH")
		get_tree().quit(2)
		return
	get_tree().create_timer(600.0).timeout.connect(func() -> void:
		print("WATCHDOG")
		get_tree().quit(2))
	await _run(int(parts[0]), int(parts[1]))
	print("RESULT: %s (%d failed, %d checks)" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)


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
	await _frames(6)
	print("== G. the sheet is gone ==")
	_ok(not FileAccess.file_exists("res://shell/quick_bar_phone_list.gd") and not ResourceLoader.exists("res://shell/quick_bar_phone_list.gd"),
		"the script file is gone")
	var names: Array = []
	for g in ProjectSettings.get_global_class_list():
		names.append(String(g["class"]))
	_ok(not names.has("QuickBarPhoneList"), "no global class QuickBarPhoneList is registered")
	var pq = _app.phone_quick
	_ok(pq == null or (not pq.has_method("open_list") and not pq.has_method("list_sheet")), "the phone controller has no open_list / list_sheet")
	_ok(not ("quick_bar_phone_list" in _app), "the app has no quick_bar_phone_list field")
	_ok(not _app.REPRESENTED_WINDOWS.has("quick_bar_phone_list") and not _app.LAZY_WINDOWS.has("quick_bar_phone_list"),
		"the class-swap lists do not name it")
	var found := false
	for n in _vp.find_children("QuickBarPhoneList", "", true, false):
		found = true
	_ok(not found, "no node named QuickBarPhoneList is in the tree")
	if not DccTheme.is_phone():
		return
	print("== S. the drawer still words its rows ==")
	var bridge = _app.bridge
	bridge.generate({"seed": 24601, "width_km": 1200.0, "grid_w": 256, "grid_h": 192, "archetype": "", "villages": true, "sea_level": 0.42})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	_app._pick_phone_tab("map")
	_app.set_phone_detent("peek")
	await get_tree().create_timer(0.4).timeout
	var idx := CommandIndex.new()
	idx.build(_app, bridge)
	for k in DccSettings.quick_bar_value("bar").duplicate():
		DccSettings.quick_bar_remove("bar", String(k["k"]))
	DccSettings.quick_bar_add("bar", "menu:%d" % DccMenus.ID_VAULT, "Markdown vault")
	DccSettings.quick_bar_add("bar", "menu:999999", "Gone Command")
	_app.phone_quick.refresh()
	await _frames(6)
	var dr = _app.phone_quick.drawer()
	_ok(dr != null, "the drawer is mounted")
	if dr == null:
		return
	var rows: Array = dr.rows()
	_ok(rows.size() >= 3 and rows[0].text == "Markdown vault\nRecords menu", "a live pin's second line is its menu: %s" % (str(rows[0].text) if rows.size() > 0 else "none"))
	_ok(rows.size() >= 3 and rows[1].text == "Gone Command\nNo longer available. The pin is kept; remove it in Customise.", "a pin whose command is gone says why: %s" % (str(rows[1].text) if rows.size() > 1 else "none"))
	_ok(dr.find_child("Assistant", true, false) == null, "no Assistant row while the assistant is off")
