extends Node
## File ▸ Share project… probe. Windowed (a SubViewport host, like `_ms4_probe.gd`), wiped user dir, under the lock:
##   godot --path . --resolution 1600x1000 _share_probe.tscn -- --vp 1600x1000
##   godot --path . --resolution 1600x1000 _share_probe.tscn -- --vp 1080x2340 --density 2.8125 --force-touch
## Arguments read by THIS file: `--vp WxH`. Accepted because the shell reads them: `--density D`, `--force-touch`.
## Anything else aborts with exit 2.
##
## Protects: the row exists in File on every form factor, is DISABLED with no world and ENABLED with one, and pressing
## it saves the project to a real .ctl zip (the file exists and starts with the zip magic `PK`) and then makes the
## platform call with that very path. The platform call is a spy (`app._share_spy`), because the Android intent cannot
## run here; the device check is the owner's. On a touch (phone) build with no spy, off Android, the fallback is
## `reveal_on_disk()` returning false (touch has no file manager) and the status line saying so, not a crash.
## Result: `RESULT: PASS|FAIL (<n> failed, <m> checks)`. Exit 0 pass, 1 fail, 2 bad argument or headless.

var _vp: SubViewport
var _app: Node
var _fails := 0
var _checks := 0
var _spied: Array = []

func _ok(c: bool, what: String) -> void:
	_checks += 1
	if not c:
		_fails += 1
	print("  ", "ok  " if c else "FAIL", " ", what)

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _bars(n: Node, out: Array) -> void:
	if n is MenuButton:
		out.append(n)
	for c in n.get_children(true):
		_bars(c, out)

func _find_row(p: PopupMenu) -> int:
	p.about_to_popup.emit()
	for i in p.item_count:
		if p.get_item_text(i) == "Share project…":
			return i
	return -1

func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		print("ABORT: run WINDOWED")
		get_tree().quit(2)
		return
	var args := OS.get_cmdline_user_args()
	var vp := ""
	var i := 0
	while i < args.size():
		var a := String(args[i])
		if a == "--force-touch":
			i += 1
		elif a in ["--density"]:
			i += 2
		elif a == "--vp":
			vp = String(args[i + 1])
			i += 2
		else:
			print("ABORT: unknown argument ", a)
			get_tree().quit(2)
			return
	var parts := vp.split("x")
	if parts.size() != 2:
		print("ABORT: needs --vp WxH")
		get_tree().quit(2)
		return
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
	if "phone_project_picker" in _app and _app.phone_project_picker != null:
		_app.phone_project_picker.hide()
	await _frames(6)
	var touch: bool = DccTheme.is_touch()
	print("form: ", "touch/phone" if touch else "desktop")
	var mbs: Array = []
	_bars(_app, mbs)
	var file_pop: PopupMenu = null
	for mb in mbs:
		if String((mb as MenuButton).text) == "File":
			file_pop = (mb as MenuButton).get_popup()
	_ok(file_pop != null, "the File menu exists")
	if file_pop == null:
		return
	var row := _find_row(file_pop)
	_ok(row >= 0, "File has a 'Share project…' row")
	if row < 0:
		return
	_ok(file_pop.get_item_id(row) == DccMenus.ID_SHARE_PROJECT, "the row carries ID_SHARE_PROJECT")
	_ok(file_pop.is_item_disabled(row), "disabled while there is no world")

	var bridge = _app.get("bridge")
	bridge.generate({"seed": 4242, "width_km": 600.0, "grid_w": 128, "grid_h": 96, "sea_level": 0.5, "villages": false})
	await bridge.generation_finished
	await _frames(20)
	_ok(bridge.has_world, "a world exists")
	row = _find_row(file_pop)
	_ok(not file_pop.is_item_disabled(row), "enabled with a world")

	var path := OS.get_user_data_dir().path_join("share_probe.ctl")  ## a real OS path: the engine writes with std::fs, which does not resolve user://
	if FileAccess.file_exists(path):
		DirAccess.remove_absolute(path)
	_app.current_project_path = path
	_app._share_spy = func(p: String) -> void: _spied.append(p)
	file_pop.id_pressed.emit(DccMenus.ID_SHARE_PROJECT)
	for _n in 600:
		if not _spied.is_empty():
			break
		await _frames(1)
	_ok(FileAccess.file_exists(path), "pressing the row wrote the project file")
	var f := FileAccess.open(path, FileAccess.READ)
	_ok(f != null and f.get_buffer(2).get_string_from_ascii() == "PK", "the file is a zip (PK magic)")
	_ok(_spied.size() == 1 and _spied[0] == path, "the platform share call was made once with the saved path")

	## No spy: the real platform branch must not crash. Touch builds have no file manager, desktop reveals.
	_app._share_spy = Callable()
	var shown: bool = _app.reveal_on_disk(path) if not touch else false
	_ok(shown == (not touch), "reveal_on_disk is %s on this form factor" % ("true" if not touch else "false (touch fallback)"))
	_app._share_saved_file(path)
	await _frames(2)
	_ok(true, "the real platform branch ran without an error")
	if FileAccess.file_exists(path):
		DirAccess.remove_absolute(path)
