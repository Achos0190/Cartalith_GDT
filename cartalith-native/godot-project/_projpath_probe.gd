extends Node
## A NEW world must be UNTITLED: it must never inherit the project file of the world it replaced.
## The data-loss hazard (OUTSTANDING_WORK.md 3.3 "Project path hazard", found on the OnePlus 6T,
## 2026-10-26): after File > New world from a saved world, `DccApp.current_project_path` stayed the
## OLD file, so the exit card named it, Save-as prefilled its name, and a plain Save wrote the new
## world over the saved one.
##
## Claim under test, one per `_check` below:
##   1. A world saved as A, then re-generated (WORLD > Generate route) KEEPS path A (the control).
##   2. New world, through a real click on Create, drops the path: `current_project_path == ""`.
##   3. The exit card says the world has never been saved and does not name A; the `last autosave`
##      row does not carry the previous world's time.
##   4. "Save and exit" (a real click) opens Save-as prefilled with the fresh `world_<seed>.ctl`,
##      never A's name.
##   5. A plain Save opens Save-as and writes NOTHING; A is byte-identical (md5) afterwards.
##   6. The autosave tick writes nothing (no `A.autosave.ctl`, A unchanged).
##   7. Choosing A's own name in Save-as raises the Overwrite confirmation, and A is still
##      byte-identical until it is answered.
##   8. A fresh name saves, becomes the project path, and A is still byte-identical.
##   9. A regenerate on that new project keeps ITS path (the control again).
##  10. Open A, then a heightmap import: also untitled (the other way into a different world).
##  11. Open A again: the path is A, and a Save from there writes A (the normal case still works).
##
## Real input, not `pressed.emit()` (MISTAKES.md: emit() is not a finger): Create, Save and exit and the
## overwrite dialog's button are `Input` mouse events at drawn centres. Setup steps that are not
## under test (generating world A, writing A, choosing the name in the Save-as dialog) call the
## shell directly and say so in the log.
##
## Run WINDOWED, in a project copy with its own `config/name` (the app writes `user://` settings and
## the worlds go to `user://Worlds`): MISTAKES.md "Isolate a probe's user dir".
##
##   godot --path <copy> --resolution 1600x900 res://_projpath_probe.tscn -- --tag desktop
##   godot --path <copy> --resolution 1280x800 res://_projpath_probe.tscn -- --tag tablet --force-touch
##   godot --path <copy> --resolution 540x1170 res://_projpath_probe.tscn -- --tag phone --force-touch
##
## Flags read below (grepped from the body): `--tag NAME` (line prefix, default `pp`) and
## `--force-touch` (read by `dcc_shell.gd`, not here). Any other `--flag` aborts.
## Result: `[tag] RESULT checks=N fail=M`; exit 0 when no check failed, 1 otherwise, 2 headless or
## unknown flag, 3 watchdog.

var app: Node
var bridge
var _tag := "pp"
var _checks := 0
var _fail := 0
var _dir := ""


func _log(s: String) -> void:
	print("[%s] %s" % [_tag, s])


func _check(ok: bool, what: String) -> void:
	_checks += 1
	if not ok:
		_fail += 1
	_log("  %-4s %s" % ["OK" if ok else "FAIL", what])


func _arg(name: String, dflt: String) -> String:
	var a := OS.get_cmdline_user_args()
	var i := a.find(name)
	return String(a[i + 1]) if i >= 0 and i + 1 < a.size() else dflt


func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame


func _walk(n: Node, out: Array) -> void:
	out.append(n)
	for c in n.get_children(true):
		_walk(c, out)


## Every visible Button under `root` whose text equals `text` (case-insensitive).
func _button(root: Node, text: String) -> Button:
	var all: Array = []
	_walk(root, all)
	for n in all:
		if n is Button and (n as Button).is_visible_in_tree() \
				and String((n as Button).text).strip_edges().to_lower() == text.to_lower():
			return n
	return null


## A real left click at `b`'s drawn centre, through `Input` (hit-test, embedded dialog, focus).
func _real_click(b: Control, what: String) -> void:
	await _frames(20)
	var vp := b.get_viewport()
	var at: Vector2 = vp.get_final_transform() * (b.get_global_rect().get_center())
	if vp is Window and vp != get_tree().root:
		at += Vector2((vp as Window).position)
	_log("    real click on %s at %s" % [what, str(at)])
	var mm := InputEventMouseMotion.new()
	mm.position = at
	mm.global_position = at
	Input.parse_input_event(mm)
	await _frames(2)
	for down in [true, false]:
		var e := InputEventMouseButton.new()
		e.button_index = MOUSE_BUTTON_LEFT
		e.pressed = down
		e.position = at
		e.global_position = at
		Input.parse_input_event(e)
		await _frames(2)
	await _frames(4)


func _wait_generated() -> void:
	var waited := 0
	while waited < 4000 and (bridge.generating or waited < 6):
		await get_tree().process_frame
		waited += 1
	await _frames(10)


## The visible Save-as browser, or null.
func _save_dialog() -> Node:
	var all: Array = []
	_walk(app, all)
	for n in all:
		if n is DccBrowseDialog and (n as Window).visible:
			return n
	return null


func _window_titled(prefix: String) -> Window:
	var all: Array = []
	_walk(app, all)
	for n in all:
		if n is Window and (n as Window).visible and String((n as Window).title).begins_with(prefix):
			return n
	return null


## Every readable string under `root`: labels, button captions and tooltips.
func _texts(root: Node) -> PackedStringArray:
	var out := PackedStringArray()
	var all: Array = []
	_walk(root, all)
	for n in all:
		if n is Label:
			out.append(String((n as Label).text))
		if n is Button:
			out.append(String((n as Button).text))
		if n is Control and String((n as Control).tooltip_text) != "":
			out.append(String((n as Control).tooltip_text))
	return out


func _any_contains(texts: PackedStringArray, needle: String) -> bool:
	for t in texts:
		if t.contains(needle):
			return true
	return false


## Close every transient dialog this probe opened, so the next step starts from the bare shell.
func _dismiss_all() -> void:
	var all: Array = []
	_walk(app, all)
	for n in all:
		if n is DccBrowseDialog and (n as Window).visible:
			(n as Window).hide()
	for n in all:
		if n is AcceptDialog and (n as Window).visible and n != app.new_world_dialog \
				and n != app.open_project_dialog and n != app.phone_project_picker:
			(n as Window).hide()
	await _frames(4)


func _listing() -> String:
	var parts := PackedStringArray()
	var names := DirAccess.get_files_at(_dir)
	names.sort()
	for f in names:
		parts.append("%s:%s" % [f, FileAccess.get_md5(_dir.path_join(f))])
	return ",".join(parts)


## Drive the real Create. Desktop and tablet press the dialog's OK; a phone presses CREATE WORLD.
func _create_new_world(seed_value: int) -> void:
	app.open_new_world()
	await _frames(12)
	var nw = app.new_world_dialog
	nw.seed_input.value = seed_value
	nw.seed_input.value_changed.emit(float(seed_value))
	nw.grid_w_input.value = 128
	nw.grid_w_input.value_changed.emit(128.0)
	await _frames(3)
	var req: Dictionary = nw.request()
	_log("    new-world request seed=%s grid_w=%s phone=%s" % [str(req.get("seed")), str(req.get("grid_w")), app.is_phone()])
	var btn: Button = nw._phone_create if app.is_phone() else nw.get_ok_button()
	await _real_click(btn, "Create (%s)" % btn.text)
	await _wait_generated()


func _ready() -> void:
	_tag = _arg("--tag", "pp")
	for a in OS.get_cmdline_user_args():
		if String(a).begins_with("--") and not (String(a) in ["--tag", "--force-touch"]):
			_log("ABORT unknown flag %s" % a)
			get_tree().quit(2)
			return
	if DisplayServer.get_name() == "headless":
		_log("ABORT run windowed")
		get_tree().quit(2)
		return
	var wd := Timer.new()
	wd.wait_time = 540.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("WATCHDOG TIMEOUT")
		get_tree().quit(3))
	wd.start()

	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.6).timeout
	bridge = app.bridge
	for w in [app.open_project_dialog, app.phone_project_picker]:
		if w != null and w.visible:
			w.hide()
	await _frames(6)
	_dir = DccSettings.storage_root("projects")
	DirAccess.make_dir_recursive_absolute(_dir)
	for f in DirAccess.get_files_at(_dir):
		DirAccess.remove_absolute(_dir.path_join(f))
	_log("viewport %s phone=%s dir=%s" % [str(get_viewport().get_visible_rect().size), app.is_phone(), _dir])

	# -- setup (NOT under test): world A, saved through the real Save-as dialog's own confirm ------
	bridge.generate({"seed": 11111, "width_km": 1000.0, "grid_w": 128, "grid_h": 96,
		"archetype": "", "villages": true, "sea_level": 0.5})
	await _wait_generated()
	_check(bridge.has_world and app.current_project_path == "", "setup: world A generated, untitled")
	app.save_project()
	await _frames(10)
	var dlg = _save_dialog()
	_check(dlg != null, "setup: Save on an untitled world opens Save as")
	if dlg == null:
		_finish()
		return
	_check(dlg._name_edit.text == "world_11111.ctl",
		"setup: the untitled default name is world_<seed>.ctl (got '%s')" % dlg._name_edit.text)
	dlg._name_edit.text = "svA.ctl"
	dlg._confirm()   # setup step: choosing the name is not under test here
	var tries := 0
	while app.current_project_path == "" and tries < 900:
		await get_tree().process_frame
		tries += 1
	var path_a: String = _dir.path_join("svA.ctl")
	_check(app.current_project_path == path_a, "setup: A saved, project path is A (%s)" % app.current_project_path)
	await _frames(10)
	var md5_a := FileAccess.get_md5(path_a)
	_check(md5_a != "", "setup: A exists on disk, md5 %s" % md5_a)
	var seed_a: int = bridge.world_gen.get_seed()

	# -- 1. control: a plain re-generate keeps the path --------------------------------------------
	app._run_pipeline()
	await _wait_generated()
	_check(app.current_project_path == path_a,
		"1. a regenerate on the open project KEEPS its path (%s)" % app.current_project_path)

	# -- 2. New world, real click on Create ---------------------------------------------------------
	app._last_autosave_at = "12:34"   # a previous-world autosave time, to prove the card drops it
	await _create_new_world(22222)
	var seed_b: int = bridge.world_gen.get_seed()
	_check(seed_b == 22222 and seed_b != seed_a, "2. the new world has a different seed (%d vs %d)" % [seed_b, seed_a])
	_check(app.current_project_path == "",
		"2. New world is UNTITLED: project path is '' (got '%s')" % app.current_project_path)

	# -- 3. the exit card -----------------------------------------------------------------------------
	var card = app.confirm_unsaved_world("Exit Cartalith", "Exit the app?", "Discard and exit",
		"Save and exit", func(): pass)
	await _frames(12)
	var t := _texts(card)
	_check(not _any_contains(t, "svA"), "3. the exit card does not name A")
	_check(_any_contains(t, "never been saved"), "3. the exit card says the new world has never been saved")
	_check(not _any_contains(t, "12:34"), "3. the exit card does not carry the previous world's autosave time")

	# -- 4. Save and exit (real click) opens Save-as with a fresh name --------------------------------
	var sb := _button(card, "Save and exit")
	_check(sb != null, "4. the exit card has a visible 'Save and exit' button")
	if sb != null:
		await _real_click(sb, "Save and exit")
	await _frames(12)
	dlg = _save_dialog()
	_check(dlg != null, "4. Save and exit on an untitled world opens Save as, writing nothing yet")
	if dlg != null:
		_check(dlg._name_edit.text == "world_22222.ctl",
			"4. Save-as is prefilled world_22222.ctl, not A's name (got '%s')" % dlg._name_edit.text)
	_check(FileAccess.get_md5(path_a) == md5_a, "4. A is byte-identical after Save and exit's route")
	await _dismiss_all()

	# -- 5. plain Save -----------------------------------------------------------------------------------
	var before := _listing()
	app.save_project()
	await _frames(12)
	dlg = _save_dialog()
	_check(dlg != null, "5. a plain Save on the new world opens Save as instead of writing")
	if dlg != null:
		_check(dlg._name_edit.text == "world_22222.ctl", "5. ... prefilled world_22222.ctl (got '%s')" % dlg._name_edit.text)
	await _frames(20)
	_check(_listing() == before, "5. a plain Save wrote nothing: A is byte-identical and no new file")
	await _dismiss_all()

	# -- 6. autosave ----------------------------------------------------------------------------------------
	bridge.world_dirty = true
	app._autosave_tick()
	await _frames(20)
	_check(_listing() == before, "6. the autosave tick wrote nothing (no A.autosave.ctl, A untouched)")
	_check(not FileAccess.file_exists(_dir.path_join("svA.autosave.ctl")), "6. svA.autosave.ctl does not exist")

	# -- 7. A's own name in Save-as raises the overwrite confirmation ------------------------------------------
	app.save_project()
	await _frames(10)
	dlg = _save_dialog()
	if dlg != null:
		dlg._name_edit.text = "svA.ctl"
		dlg._confirm()   # setup: choosing the name; the confirmation is what is under test
		await _frames(14)
		var ow := _window_titled("Overwrite")
		_check(ow != null, "7. choosing A's name raises the Overwrite confirmation")
		_check(FileAccess.get_md5(path_a) == md5_a, "7. A is still byte-identical before the question is answered")
		if ow != null:
			var cancel := _button(ow, "Cancel")
			if cancel != null:
				await _real_click(cancel, "Overwrite > Cancel")
		await _frames(10)
		_check(FileAccess.get_md5(path_a) == md5_a, "7. Cancel left A byte-identical")
	else:
		_check(false, "7. Save as did not open")
	await _dismiss_all()

	# -- 8. a fresh name saves, and A is untouched ------------------------------------------------------------------
	app.save_project()
	await _frames(10)
	dlg = _save_dialog()
	if dlg != null:
		dlg._name_edit.text = "svB.ctl"
		dlg._confirm()
	var path_b: String = _dir.path_join("svB.ctl")
	tries = 0
	while app.current_project_path != path_b and tries < 900:
		await get_tree().process_frame
		tries += 1
	_check(app.current_project_path == path_b, "8. a fresh name saves: project path is B (%s)" % app.current_project_path)
	_check(FileAccess.file_exists(path_b), "8. B exists on disk")
	_check(FileAccess.get_md5(path_a) == md5_a, "8. A is still byte-identical after B was saved")
	await _frames(10)

	# -- 9. control: re-generate on the new project keeps B ------------------------------------------------------------
	app._run_pipeline()
	await _wait_generated()
	_check(app.current_project_path == path_b, "9. a regenerate on B keeps path B (%s)" % app.current_project_path)

	# -- 10. Open A, then a heightmap import is untitled too ----------------------------------------------------------------
	_check(app.open_recent_project(path_a), "10. Open A succeeds")
	await _frames(10)
	_check(app.current_project_path == path_a, "10. after Open, the project path is A")
	var img := Image.create(64, 40, false, Image.FORMAT_L8)
	for y in 40:
		for x in 64:
			var v := 0.5 + 0.4 * sin(float(x) * 0.2) * cos(float(y) * 0.25)
			img.set_pixel(x, y, Color(v, v, v))
	var hm := OS.get_user_data_dir().path_join("pp_hm.png")
	img.save_png(hm)
	if bridge.import_api:
		app._import_heightmap_at(hm)
		await _wait_generated()
		_check(app.current_project_path == "", "10. a heightmap import is UNTITLED (got '%s')" % app.current_project_path)
		_check(FileAccess.get_md5(path_a) == md5_a, "10. A is byte-identical after the import")
	else:
		_log("  SKIP heightmap import: this build has no import_api")

	# -- 11. Open A again: Save writes A (the normal case) ---------------------------------------------------------------------------
	_check(app.open_recent_project(path_a), "11. Open A again")
	await _frames(10)
	bridge.world_dirty = true
	app.save_project()
	await _frames(30)
	_check(app.current_project_path == path_a, "11. Save on an opened project keeps path A and writes it (no dialog)")
	_check(_save_dialog() == null, "11. ... without asking for a name")
	_finish()


func _finish() -> void:
	_log("RESULT checks=%d fail=%d" % [_checks, _fail])
	get_tree().quit(1 if _fail > 0 else 0)
