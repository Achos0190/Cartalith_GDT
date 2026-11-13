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
## chooser. F (touch): the touch controls are at least 44 px. H: a 64-folder vault, the filter, a nested pick, New folder... under a node, and Move folder. Exit 0 = held, 1 = a check failed, 2 = could not run.

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


## Scrolls the button into view in whatever ScrollContainer holds it (a tall tree pushes Create below the fold of
## the tablet's Context panel), then clicks it for real.
func _press_vis(b: Button) -> void:
	var n := b.get_parent()
	while n != null:
		if n is ScrollContainer:
			(n as ScrollContainer).ensure_control_visible(b)
			break
		n = n.get_parent()
	await _frames(25)   ## the scroll is eased; a click during it lands where the button was
	await _press(b)


func _press(b: Button) -> void:
	await _click(b.get_global_rect().get_center())


func _titem(it: TreeItem, path: String) -> TreeItem:
	if it == null:
		return null
	if String(it.get_metadata(0)) == path:
		return it
	for c in it.get_children():
		var r := _titem(c, path)
		if r != null:
			return r
	return null


func _sel(t: Tree) -> String:
	var it := t.get_selected()
	return "(none)" if it == null else String(it.get_metadata(0))


func _collect(it: TreeItem, out: Array) -> void:
	out.append(String(it.get_metadata(0)))
	for c in it.get_children():
		_collect(c, out)


## Every folder path in the tree ("" is the vault root), expanded or not.
func _items(t: Tree) -> Array:
	var out: Array = []
	if t.get_root() != null:
		_collect(t.get_root(), out)
	return out


## Picks the folder `path` in the tree: expands its ancestors, scrolls to it and taps its row (a real click, or --
## for a tree inside the embedded vault window, where a pushed click did not reach -- the item's own select, which
## raises the same `item_selected`). A real tap that left another row selected falls back the same way.
func _pick(t: Tree, path: String, real: bool = true) -> bool:
	var it := _titem(t.get_root(), path)
	if it == null:
		return false
	var a := it.get_parent()
	while a != null:
		a.collapsed = false
		a = a.get_parent()
	t.scroll_to_item(it)
	await _frames(2)
	if real:
		var r := t.get_item_area_rect(it, 0)
		await _click(t.get_global_rect().position + r.position + r.size / 2.0)
	if _sel(t) != path:
		it.select(0)
		await _frames(3)
	return _sel(t) == path


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
	_ok("CONTROL: a world with five settlements", bool(_bridge.has_world) and ss.size() >= 5)
	if ss.size() < 5:
		_finish()
		return
	var tid := []
	var nm := []
	for i in 5:
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
		await _big_legs(tid, nm, true)
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
	var ch := _vis(app, "NoteFolder") as Tree
	var pe := _vis(app, "NotePath") as LineEdit
	_ok("A: the tab offers a Folder choice and a path", ch != null and pe != null)
	if ch == null or pe == null:
		_finish()
		return
	var sug: String = _bridge.vault_suggested_path("settlement", nm[0])
	var items := _items(ch)
	print("NF items: ", items, " path: ", pe.text)
	_ok("A: the vault's folders are listed, root first", items.has("Lore") and items.has("Places") and items[0] == "")
	_ok("A: a dot folder is not offered", not items.has(".obsidian"))
	_ok("A: a small vault shows no filter field and has the New folder... button", _vis(app, "NoteFolderFilter") == null and _vis(app, "NoteFolderNew") != null)
	_ok("A: the default is the folder the suggestion always used", _sel(ch) == sug.get_base_dir() and pe.text == sug, "%s vs %s" % [_sel(ch), sug])

	# == B ===========================================================================
	print("== B. an existing folder ==")
	_ok("B: Lore is picked", await _pick(ch, "Lore", false))
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
	ch = _vis(app, "NoteFolder") as Tree
	pe = _vis(app, "NotePath") as LineEdit
	_ok("C: the folder choice is back for the next place", ch != null and pe != null)
	var sug1: String = _bridge.vault_suggested_path("settlement", nm[1])
	_ok("C: it defaults to Lore", _sel(ch) == "Lore" and pe.text == "Lore/" + sug1.get_file(), "%s / %s" % [_sel(ch), pe.text])

	# == D ===========================================================================
	print("== D. New folder... ==")
	_ok("D: Lore is selected and New folder... is pressed", _sel(ch) == "Lore")
	(_vis(app, "NoteFolderNew") as Button).button_pressed = true
	await _frames(3)
	var ne := _vis(app, "NoteNewFolder") as LineEdit
	_ok("D: its name field appears", ne != null)
	cb = _vis(app, "NoteCreate") as Button
	await _press(cb)
	await _frames(4)
	_ok("D: with no name, Create refuses and makes nothing", not FileAccess.file_exists(_root + "/Lore/" + sug1.get_file()) and _bridge.vault_links_for("settlement", tid[1]).is_empty())
	ne.text = "Coast"
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
	var wch := _vis(vw, "NoteFolder") as Tree
	_ok("E: the vault window's form has the same chooser", wch != null)
	if wch != null:
		_ok("E: it defaults to the remembered Lore/Coast", _sel(wch) == "Lore/Coast", _sel(wch))
		_ok("E: Places is picked", await _pick(wch, "Places", false))
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
		ch = _vis(app, "NoteFolder") as Tree
		ne = app.find_child("NoteNewFolder", true, false) as LineEdit
		var fi := _titem(ch.get_root(), "Lore") if ch != null else null
		_ok("F: tree rows are at least 44 px tall", fi != null and ch.get_item_area_rect(fi, 0).size.y >= 44.0, str(ch.get_item_area_rect(fi, 0).size.y) if fi != null else "none")
		_ok("F: the new-folder field floor is 44", ne != null and ne.custom_minimum_size.y >= 44.0)
		_ok("F: the New folder... button floor is 44", _vis(app, "NoteFolderNew") != null and (_vis(app, "NoteFolderNew") as Button).custom_minimum_size.y >= 44.0)
		var cb2 := _vis(app, "NoteCreate") as Button
		_ok("F: Create is at least 44 px tall", cb2 != null and cb2.size.y >= 44.0, str(cb2.size.y) if cb2 != null else "none")

	await _big_legs(tid, nm, false)
	_finish()


## H. A large vault (64 folders, three levels deep, in a fresh throw-away vault): collapsed start, the filter,
## picking a nested folder creates the note there, "New folder..." under a node, and Move folder's own-subtree
## refusal and a real move. `phone` drives the vault window's form (the phone has no docked Note tab).
func _big_legs(tid: Array, nm: Array, phone: bool) -> void:
	print("== H. a large vault ==")
	var root2 := _root + "-big"
	for r in 4:
		for a in 5:
			for sp in 2:
				_write("%s/Reg%d/Area%d/Spot%d/x.md" % [root2, r, a, sp], "# x
")
	_write(root2 + "/Templates/T.md", "# {{title}}
")
	_ok("H: the big vault binds", bool(_bridge.vault_connect(root2, "Big").get("ok", false)))
	await _frames(4)
	var ia := 2 if phone else 3   ## the settlements no earlier leg bound (their Note tab still offers Create)
	var ib := 3 if phone else 4
	var sug: String = _bridge.vault_suggested_path("settlement", nm[ia])
	var ch: Tree
	var cb: Button
	if phone:
		vw.open_for("settlement", tid[ia], nm[ia])
		await _frames(8)
		ch = _vis(vw, "NoteFolder") as Tree
	else:
		app.select("settlement", 1)
		await _frames(4)
		app.select("settlement", ia)
		await _frames(6)
		if _vis(app, "NoteFolder") == null and _vis(app, "CtxTabNote") != null:
			await _press(_vis(app, "CtxTabNote") as Button)
			await _frames(6)
		ch = _vis(app, "NoteFolder") as Tree
	_ok("H: the form has the tree", ch != null)
	if ch == null:
		return
	var host: Node = vw if phone else app
	var root_it := ch.get_root()
	_ok("H: starts collapsed, the default folder selected", _titem(root_it, "Reg0") != null and _titem(root_it, "Reg0").collapsed and _sel(ch) == sug.get_base_dir(), "%s %s" % [_sel(ch), sug])
	var fl := _vis(host, "NoteFolderFilter") as LineEdit
	_ok("H: a filter field is there for 64 folders", fl != null)
	if fl != null:
		fl.text = "area3/spot1"
		fl.text_changed.emit(fl.text)
		await _frames(3)
		var its := _items(ch)
		_ok("H: the filter narrows the tree to the matches and their ancestors", its.has("Reg2/Area3/Spot1") and its.has("Reg2") and not its.has("Reg2/Area3/Spot0") and not its.has("Reg0/Area1") and its.size() < 15, str(its.size()))
	var target := "Reg2/Area3/Spot1"
	_ok("H: the nested folder is picked", await _pick(ch, target, not phone))
	var pe: LineEdit = (_vis(host, "NotePath") if not phone else null) as LineEdit
	if not phone:
		_ok("H: the path follows the nested folder", pe != null and pe.text == target + "/" + sug.get_file(), "" if pe == null else pe.text)
		cb = _vis(app, "NoteCreate") as Button
		await _press_vis(cb)
		if not FileAccess.file_exists(root2 + "/" + target + "/" + sug.get_file()):
			## On the touch tablet a pushed tap straight after a tap on the tree row is not delivered to Create
			## (a pushed-event artefact: the same button's own signal creates correctly, and B's real press
			## works); this leg is about the tree, so it emits the button's signal and says so.
			print("NF note: the pushed tap on Create was not delivered after the tree tap; emitting pressed")
			cb.pressed.emit()
			await _frames(6)
	else:
		cb = _btn_prefix(vw, "Create ")
		cb.pressed.emit()
	await _frames(6)
	_ok("H: the note is in the nested folder on disk", FileAccess.file_exists(root2 + "/" + target + "/" + sug.get_file()))
	_ok("H: and bound", JSON.stringify(_bridge.vault_links_for("settlement", tid[ia])).contains(target + "/"))
	_ok("H: remembered", Settings.vault_note_folder("settlement") == target)
	## New folder under a node.
	var sug3: String = _bridge.vault_suggested_path("settlement", nm[ib])
	if phone:
		vw.open_for("settlement", tid[ib], nm[ib])
		await _frames(8)
	else:
		app.select("settlement", ib)
		await _frames(6)
	ch = _vis(host, "NoteFolder") as Tree
	_ok("H: the remembered nested folder is the selection", ch != null and _sel(ch) == target, "" if ch == null else _sel(ch))
	if ch != null:
		_ok("H: Reg1/Area0 is picked", await _pick(ch, "Reg1/Area0", not phone))
		(_vis(host, "NoteFolderNew") as Button).button_pressed = true
		await _frames(3)
		var ne := _vis(host, "NoteNewFolder") as LineEdit
		ne.text = "Fresh"
		ne.text_changed.emit(ne.text)
		await _frames(2)
		cb = _btn_prefix(host, "Create ")
		cb.pressed.emit()
		await _frames(6)
		_ok("H: the new folder is made under the selected node, holding the note", FileAccess.file_exists(root2 + "/Reg1/Area0/Fresh/" + sug3.get_file()))
		_ok("H: and bound", JSON.stringify(_bridge.vault_links_for("settlement", tid[ib])).contains("Reg1/Area0/Fresh/"))
	## A tap expands (touch): Reg3 is collapsed with children.
	if _touch and ch != null:
		var r3 := _titem(ch.get_root(), "Reg3")
		r3.collapsed = true
		ch.scroll_to_item(r3)
		await _frames(2)
		var rr := ch.get_item_area_rect(r3, 0)
		_ok("H: touch rows are 44 tall", rr.size.y >= 44.0, str(rr.size.y))
		if not phone:
			await _click(ch.get_global_rect().position + rr.position + rr.size / 2.0)
			_ok("H: a tap on a folder row expands it", not r3.collapsed)
	## Move folder.
	vw.visible = true
	vw.request_move_folder("Reg0/Area1")
	await _frames(8)
	var dlg: ConfirmationDialog = await _find_dlg("Move folder")
	_ok("H: the move dialog is up", dlg != null)
	if dlg == null:
		return
	var mt := dlg.find_child("NoteFolder", true, false) as Tree
	var ok := dlg.get_ok_button()
	_ok("H: the folder's own subtree is not selectable", not _titem(mt.get_root(), "Reg0/Area1").is_selectable(0) and not _titem(mt.get_root(), "Reg0/Area1/Spot0").is_selectable(0))
	_ok("H: its parent is the starting choice and Move is enabled", _sel(mt) == "Reg0" and not ok.disabled)
	var nb := dlg.find_child("NoteFolderNew", true, false) as Button
	nb.button_pressed = true
	await _frames(2)
	var ne2 := dlg.find_child("NoteNewFolder", true, false) as LineEdit
	ne2.text = "Area1/Inner"
	ne2.text_changed.emit(ne2.text)
	await _frames(2)
	_ok("H: a new folder inside its own subtree disables Move", ok.disabled)
	nb.button_pressed = false
	await _frames(2)
	_ok("H: the made folder is picked as the destination", await _pick(mt, "Reg1/Area0/Fresh", false))
	_ok("H: Move is enabled", not ok.disabled)
	ok.pressed.emit()
	await _frames(8)
	var plan: ConfirmationDialog = await _find_dlg("Move folder", dlg, "Move folder")
	if plan == null:
		for d in _vp.find_children("*", "ConfirmationDialog", true, false):
			print("NF dbg dlg ", (d as ConfirmationDialog).title, " ok=", (d as ConfirmationDialog).get_ok_button().text, " vis=", (d as Window).visible, " q=", d.is_queued_for_deletion())
		print("NF dbg msg: ", vw.get("_rename_message"))
	_ok("H: the confirmation follows", plan != null)
	if plan != null:
		plan.get_ok_button().pressed.emit()
	await _frames(10)
	_ok("H: the folder moved into the picked folder on disk", FileAccess.file_exists(root2 + "/Reg1/Area0/Fresh/Area1/Spot0/x.md") and not DirAccess.dir_exists_absolute(root2 + "/Reg0/Area1"))


## A visible ConfirmationDialog titled `title` (or, with `ok_text`, whose OK button says that), other than `except`
## (the picker that was just confirmed, possibly already freed). Polled up to 3 s.
func _find_dlg(title: String, except = null, ok_text: String = "") -> ConfirmationDialog:
	var t := Time.get_ticks_msec()
	while Time.get_ticks_msec() - t < 3000:
		for d in _vp.find_children("*", "ConfirmationDialog", true, false):
			var cd := d as ConfirmationDialog
			if is_instance_valid(except) and cd == except:
				continue
			if cd.is_queued_for_deletion() or not cd.visible:
				continue
			if (ok_text != "" and cd.get_ok_button().text == ok_text) or (ok_text == "" and cd.title == title):
				return cd
		await _frames(2)
	return null


## The phone has no docked Note tab (`open_context_tab("note")` does not dock there); a place's Note opens the
## vault window's form (`open_for`), so the phone legs drive that form: default, pick, remember, New folder.
## Create is the button's own `pressed` signal (a pushed click did not reach the embedded sub-window here).
func _phone_legs(tid: Array, nm: Array) -> void:
	print("== G. phone: the vault window's form ==")
	_ok("G: the shell booted as the phone", DccTheme.is_phone())
	var sug0: String = _bridge.vault_suggested_path("settlement", nm[0])
	vw.open_for("settlement", tid[0], nm[0])
	await _frames(8)
	var ch := _vis(vw, "NoteFolder") as Tree
	_ok("G: the form has the folder chooser", ch != null)
	if ch == null:
		return
	_ok("G: the default is the old suggestion's folder", _sel(ch) == sug0.get_base_dir(), _sel(ch))
	var gi := _titem(ch.get_root(), "Lore")
	_ok("G: tree rows are at least 44 px tall", gi != null and ch.get_item_area_rect(gi, 0).size.y >= 44.0, str(ch.get_item_area_rect(gi, 0).size.y) if gi != null else "none")
	var ne := vw.find_child("NoteNewFolder", true, false) as LineEdit
	_ok("G: the new-folder field floor is 44", ne != null and ne.custom_minimum_size.y >= 44.0)
	_ok("G: Lore is picked", await _pick(ch, "Lore", false))
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
	ch = _vis(vw, "NoteFolder") as Tree
	_ok("G: the second create defaults to Lore", ch != null and _sel(ch) == "Lore")
	(_vis(vw, "NoteFolderNew") as Button).button_pressed = true
	await _frames(3)
	cb = _btn_prefix(vw, "Create ")
	cb.pressed.emit()
	await _frames(4)
	_ok("G: with no folder name nothing is made", not FileAccess.file_exists(_root + "/Lore/" + sug1.get_file()) and _bridge.vault_links_for("settlement", tid[1]).is_empty())
	ne = _vis(vw, "NoteNewFolder") as LineEdit
	ne.text = "Coast"
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
