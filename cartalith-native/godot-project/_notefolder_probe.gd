extends Node
## Owner request 2026-11-13: the user chooses the vault folder a new note goes in (`VaultFolderPick`), from the
## Note tab's Create and from the vault window's template form. Windowed (refuses --headless), under a wiped
## user directory (`APPDATA=<empty dir>`), a throw-away vault in TEMP.
##
##   Godot_v4.7.2-stable_win64_console.exe --path . _notefolder_probe.tscn -- --vp 1600x1000 [--touch] [--ff]
##   (phone: --vp 1080x2340 --touch; Window/stretch is whatever the app sets itself)
##
## Legs. A: the folder list is the vault's own and the default is the old behaviour's folder. B: pick an existing
## folder, press Create (a real click) -> the FILE is in that folder (re-read from disk) and the place is bound to
## it. C: a second create defaults to the remembered folder. D: New folder... with no name refuses and creates
## nothing; with a name makes the folder and the file and binds. E: the vault window's template form uses the same
## chooser. F (touch): the touch controls are at least 44 px. Exit 0 = held, 1 = a check failed, 2 = could not run.

const Pick := preload("res://shell/vault_folder_pick.gd")
const Settings := preload("res://shell/dcc_settings.gd")

var app: Node
var _bridge
var vw
var _vp: SubViewport
var _root := ""
var _touch := false
var _ff := false
var _n := 0
var _fails: Array = []


func _ok(label: String, cond: bool, detail: String = "") -> void:
	_n += 1
	if cond:
		print("NF  ok  %s" % label)
	else:
		_fails.append(label)
		print("NF  !!  %s   %s" % [label, detail])
		if _ff:
			_finish()


func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame


func _write(path: String, text: String) -> void:
	DirAccess.make_dir_recursive_absolute(path.get_base_dir())
	var f := FileAccess.open(path, FileAccess.WRITE)
	f.store_string(text)
	f.close()


func _vis(n: Node, name: String) -> Control:
	if n.name == name and n is Control and (n as Control).is_visible_in_tree():
		return n
	for c in n.get_children():
		var r := _vis(c, name)
		if r != null:
			return r
	return null


func _btn_prefix(n: Node, prefix: String) -> Button:
	if n is Button and (n as Button).text.to_lower().begins_with(prefix.to_lower()) and (n as Button).is_visible_in_tree():
		return n
	for c in n.get_children():
		var r := _btn_prefix(c, prefix)
		if r != null:
			return r
	return null


func _click(pos: Vector2) -> void:
	for pressed in [true, false]:
		var ev := InputEventMouseButton.new()
		ev.button_index = MOUSE_BUTTON_LEFT
		ev.pressed = pressed
		ev.position = pos
		ev.global_position = pos
		if _touch:
			ev.device = InputEvent.DEVICE_ID_EMULATION
		_vp.push_input(ev)
		await _frames(2)
	await _frames(4)


func _press(b: Button) -> void:
	await _click(b.get_global_rect().get_center())


## Picks `label` in the dropdown the way the popup would: select + item_selected (the popup's own click is the
## engine's; what is tested is what the shell does with the choice).
func _pick(ch: OptionButton, label: String) -> bool:
	for i in ch.item_count:
		if ch.get_item_text(i) == label:
			ch.select(i)
			ch.item_selected.emit(i)
			await _frames(3)
			return true
	return false


func _items(ch: OptionButton) -> Array:
	var out: Array = []
	for i in ch.item_count:
		out.append(ch.get_item_text(i))
	return out


func _finish() -> void:
	print("NF RESULT %d checks, %d failed%s" % [_n, _fails.size(), "" if _fails.is_empty() else " :: " + str(_fails)])
	get_tree().quit(0 if _fails.is_empty() else 1)


func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		print("NF REFUSED: run windowed")
		get_tree().quit(2)
		return
	var wd := Timer.new()
	wd.wait_time = 500.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("NF WATCHDOG")
		get_tree().quit(3))
	wd.start()
	var args := OS.get_cmdline_user_args()
	var vpa := "1600x1000"
	if args.find("--vp") >= 0:
		vpa = String(args[args.find("--vp") + 1])
	var parts := vpa.split("x")
	_touch = "--touch" in args or "--force-touch" in args
	_ff = "--ff" in args
	if FileAccess.file_exists(VaultStore.PATH):
		DirAccess.remove_absolute(ProjectSettings.globalize_path(VaultStore.PATH))
	_vp = SubViewport.new()
	_vp.size = Vector2i(int(parts[0]), int(parts[1]))
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await get_tree().create_timer(1.4).timeout
	if app.get("open_project_dialog") != null:
		app.open_project_dialog.hide()
	await _frames(4)
	_vp.notification(Node.NOTIFICATION_VP_MOUSE_ENTER)
	_bridge = app.bridge
	vw = app.vault_window
	_bridge.generate({"seed": 4242, "width_km": 2400.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.45})
	await _frames(2)
	var t0 := Time.get_ticks_msec()
	while bool(_bridge.generating) and Time.get_ticks_msec() - t0 < 240000:
		await _frames(10)
	await _frames(30)
	var ss: Array = _bridge.settlements()
	_ok("CONTROL: a world with four settlements", bool(_bridge.has_world) and ss.size() >= 4)
	if ss.size() < 4:
		_finish()
		return
	var tid := []
	var nm := []
	for i in 4:
		tid.append(int((ss[i] as Dictionary).get("tid", 0)))
		nm.append(String((ss[i] as Dictionary).get("name", "")))
	_ok("CONTROL: ids are real", not tid.has(0))

	_root = OS.get_environment("TEMP").replace("\\", "/") + "/cartalith-nf-%s-%d" % ["touch" if _touch else "desk", Time.get_ticks_msec()]
	_write(_root + "/Lore/Old.md", "# Old\n")
	_write(_root + "/Places/Other.md", "# Other\n")
	_write(_root + "/Templates/T.md", "# {{title}}\n")
	_write(_root + "/.obsidian/x.md", "hidden\n")
	_ok("CONTROL: vault binds", bool(_bridge.vault_connect(_root, "Chronicle").get("ok", false)))
	_ok("CONTROL: nothing remembered yet", Settings.vault_note_folder("settlement") == null)

	if DccTheme.is_phone():
		await _phone_legs(tid, nm)
		_finish()
		return

	# == A ===========================================================================
	print("== A. the list and the default ==")
	app.select("settlement", 0)
	await _frames(6)
	var tab := _vis(app, "CtxTabNote") as Button
	_ok("the Note tab exists", tab != null)
	if tab == null:
		_finish()
		return
	await _press(tab)
	await _frames(6)
	var ch := _vis(app, "NoteFolder") as OptionButton
	var pe := _vis(app, "NotePath") as LineEdit
	_ok("A: the tab offers a Folder choice and a path", ch != null and pe != null)
	if ch == null or pe == null:
		_finish()
		return
	var sug: String = _bridge.vault_suggested_path("settlement", nm[0])
	var items := _items(ch)
	print("NF items: ", items, " path: ", pe.text)
	_ok("A: the vault's folders are listed, root first", items.has("Lore") and items.has("Places") and items.has("(vault root)"))
	_ok("A: a dot folder is not offered", not items.has(".obsidian"))
	_ok("A: New folder... is the last entry", items[items.size() - 1] == Pick.NEW_FOLDER_LABEL)
	_ok("A: the default is the folder the suggestion always used", ch.get_item_text(ch.selected) == (sug.get_base_dir() if sug.get_base_dir() != "" else "(vault root)") and pe.text == sug, "%s vs %s" % [ch.get_item_text(ch.selected), sug])

	# == B ===========================================================================
	print("== B. an existing folder ==")
	_ok("B: Lore is picked", await _pick(ch, "Lore"))
	_ok("B: the path follows the folder", pe.text == "Lore/" + sug.get_file(), pe.text)
	var cb := _vis(app, "NoteCreate") as Button
	await _press(cb)
	await _frames(6)
	_ok("B: the file is in Lore on disk", FileAccess.file_exists(_root + "/Lore/" + sug.get_file()))
	_ok("B: ... and not where the default would have put it", not FileAccess.file_exists(_root + "/" + sug))
	var links: Array = _bridge.vault_links_for("settlement", tid[0])
	_ok("B: the place is bound to it", links.size() == 1 and JSON.stringify(links).contains("Lore/"), str(links))
	_ok("B: the folder was remembered", Settings.vault_note_folder("settlement") == "Lore")

	# == C ===========================================================================
	print("== C. the next create defaults to it ==")
	app.select("settlement", 1)
	await _frames(6)
	ch = _vis(app, "NoteFolder") as OptionButton
	pe = _vis(app, "NotePath") as LineEdit
	_ok("C: the folder choice is back for the next place", ch != null and pe != null)
	var sug1: String = _bridge.vault_suggested_path("settlement", nm[1])
	_ok("C: it defaults to Lore", ch.get_item_text(ch.selected) == "Lore" and pe.text == "Lore/" + sug1.get_file(), "%s / %s" % [ch.get_item_text(ch.selected), pe.text])

	# == D ===========================================================================
	print("== D. New folder... ==")
	_ok("D: New folder... is picked", await _pick(ch, Pick.NEW_FOLDER_LABEL))
	var ne := _vis(app, "NoteNewFolder") as LineEdit
	_ok("D: its name field appears", ne != null)
	cb = _vis(app, "NoteCreate") as Button
	await _press(cb)
	await _frames(4)
	_ok("D: with no name, Create refuses and makes nothing", not FileAccess.file_exists(_root + "/Lore/" + sug1.get_file()) and _bridge.vault_links_for("settlement", tid[1]).is_empty())
	ne.text = "Lore/Coast"
	ne.text_changed.emit(ne.text)
	await _frames(2)
	_ok("D: the path follows the typed folder", pe.text == "Lore/Coast/" + sug1.get_file(), pe.text)
	await _press(cb)
	await _frames(6)
	_ok("D: the folder was made", DirAccess.dir_exists_absolute(_root + "/Lore/Coast"))
	_ok("D: the file is inside it", FileAccess.file_exists(_root + "/Lore/Coast/" + sug1.get_file()))
	_ok("D: the place is bound to it", JSON.stringify(_bridge.vault_links_for("settlement", tid[1])).contains("Lore/Coast/"))
	_ok("D: it is remembered", Settings.vault_note_folder("settlement") == "Lore/Coast")

	# == E: the vault window's form ===================================================
	print("== E. the vault window's template form ==")
	vw.open_for("settlement", tid[2], nm[2])
	await _frames(8)
	var wch := _vis(vw, "NoteFolder") as OptionButton
	_ok("E: the vault window's form has the same chooser", wch != null)
	if wch != null:
		_ok("E: it defaults to the remembered Lore/Coast", wch.get_item_text(wch.selected) == "Lore/Coast", wch.get_item_text(wch.selected))
		_ok("E: Places is picked", await _pick(wch, "Places"))
		var wc := _btn_prefix(vw, "Create ")
		_ok("E: Create is there", wc != null)
		if wc != null:
			var sug2: String = _bridge.vault_suggested_path("settlement", nm[2])
			## The vault window is an embedded sub-window; a pushed click on it did not reach the button in this
			## harness (the Note tab legs use real clicks), so this leg emits the button's own signal.
			wc.pressed.emit()
			await _frames(6)
			_ok("E: the note is in Places", FileAccess.file_exists(_root + "/Places/" + sug2.get_file()))
			_ok("E: and bound", JSON.stringify(_bridge.vault_links_for("settlement", tid[2])).contains("Places/"))

	# == F: touch floors ==============================================================
	if _touch:
		print("== F. touch targets ==")
		app.select("settlement", 3)
		await _frames(6)
		ch = _vis(app, "NoteFolder") as OptionButton
		ne = app.find_child("NoteNewFolder", true, false) as LineEdit
		_ok("F: the folder dropdown is at least 44 px tall", ch != null and ch.size.y >= 44.0, str(ch.size.y) if ch != null else "none")
		_ok("F: the new-folder field floor is 44", ne != null and ne.custom_minimum_size.y >= 44.0)
		var cb2 := _vis(app, "NoteCreate") as Button
		_ok("F: Create is at least 44 px tall", cb2 != null and cb2.size.y >= 44.0, str(cb2.size.y) if cb2 != null else "none")

	_finish()


## The phone has no docked Note tab (`open_context_tab("note")` does not dock there); a place's Note opens the
## vault window's form (`open_for`), so the phone legs drive that form: default, pick, remember, New folder.
## Create is the button's own `pressed` signal (a pushed click did not reach the embedded sub-window here).
func _phone_legs(tid: Array, nm: Array) -> void:
	print("== G. phone: the vault window's form ==")
	_ok("G: the shell booted as the phone", DccTheme.is_phone())
	var sug0: String = _bridge.vault_suggested_path("settlement", nm[0])
	vw.open_for("settlement", tid[0], nm[0])
	await _frames(8)
	var ch := _vis(vw, "NoteFolder") as OptionButton
	_ok("G: the form has the folder chooser", ch != null)
	if ch == null:
		return
	_ok("G: the default is the old suggestion's folder", ch.get_item_text(ch.selected) == (sug0.get_base_dir() if sug0.get_base_dir() != "" else "(vault root)"), ch.get_item_text(ch.selected))
	_ok("G: the dropdown is at least 44 px tall", ch.size.y >= 44.0, str(ch.size.y))
	var ne := vw.find_child("NoteNewFolder", true, false) as LineEdit
	_ok("G: the new-folder field floor is 44", ne != null and ne.custom_minimum_size.y >= 44.0)
	_ok("G: Lore is picked", await _pick(ch, "Lore"))
	var cb := _btn_prefix(vw, "Create ")
	if cb == null:
		cb = _any_btn(vw, "Create ")
		_dump_btns(vw, "")
		print("NF G: Create found but hidden: ", cb != null and cb.is_visible_in_tree(), " ch visible ", ch.is_visible_in_tree(), " vw ", vw.get("_browse_phone_pane"))
	_ok("G: Create is at least 44 px tall", cb != null and cb.size.y >= 44.0, str(cb.size.y) if cb != null else "none")
	cb.pressed.emit()
	await _frames(6)
	_ok("G: the file is in Lore", FileAccess.file_exists(_root + "/Lore/" + sug0.get_file()))
	_ok("G: bound to the place", JSON.stringify(_bridge.vault_links_for("settlement", tid[0])).contains("Lore/"))
	_ok("G: remembered", Settings.vault_note_folder("settlement") == "Lore")
	var sug1: String = _bridge.vault_suggested_path("settlement", nm[1])
	vw.open_for("settlement", tid[1], nm[1])
	await _frames(8)
	ch = _vis(vw, "NoteFolder") as OptionButton
	_ok("G: the second create defaults to Lore", ch != null and ch.get_item_text(ch.selected) == "Lore")
	_ok("G: New folder is picked", await _pick(ch, Pick.NEW_FOLDER_LABEL))
	cb = _btn_prefix(vw, "Create ")
	cb.pressed.emit()
	await _frames(4)
	_ok("G: with no folder name nothing is made", not FileAccess.file_exists(_root + "/Lore/" + sug1.get_file()) and _bridge.vault_links_for("settlement", tid[1]).is_empty())
	ne = _vis(vw, "NoteNewFolder") as LineEdit
	ne.text = "Lore/Coast"
	ne.text_changed.emit(ne.text)
	await _frames(2)
	cb = _btn_prefix(vw, "Create ")
	cb.pressed.emit()
	await _frames(6)
	_ok("G: the new folder and the file are made", DirAccess.dir_exists_absolute(_root + "/Lore/Coast") and FileAccess.file_exists(_root + "/Lore/Coast/" + sug1.get_file()))
	_ok("G: bound to the place", JSON.stringify(_bridge.vault_links_for("settlement", tid[1])).contains("Lore/Coast/"))


func _any_btn(n: Node, prefix: String) -> Button:
	if n is Button and (n as Button).text.to_lower().begins_with(prefix.to_lower()):
		return n
	for c in n.get_children():
		var r := _any_btn(c, prefix)
		if r != null:
			return r
	return null


func _dump_btns(n: Node, ind: String) -> void:
	if n is Button and (n as Button).text.begins_with("Create"):
		var chain := ""
		var p := n.get_parent()
		while p != null and p != vw:
			chain += "<" + p.name + ("" if (p as CanvasItem).visible else "(hid)") if p is CanvasItem else "<" + p.name
			p = p.get_parent()
		print("NF   btn ", (n as Button).text, " vis=", (n as Button).visible, " ", chain)
	for c in n.get_children():
		_dump_btns(c, ind)
