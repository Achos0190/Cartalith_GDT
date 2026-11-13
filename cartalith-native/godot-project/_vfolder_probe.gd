extends Node
## Folder rename and move in the vault's file tree (`OUTSTANDING_WORK.md`'s vault row, "moving folders"): rename a
## folder and move one into another through the real tree menu, the prompts and the confirmation, then read the
## DISK and the crate, never the window's claim: every note at its new path, every link resolving, the entity
## bindings opening the right note, the remembered note folder following, the refusals writing nothing, the long
## press (touch) and F2 (desktop), and one undo step that puts it all back.
##
##   Godot_v4.7.2-stable_win64_console.exe --path . _vfolder_probe.tscn -- --vp 1600x1000
##   ... -- --vp 1080x2340 --density 2.8125 --force-touch   (the phone leg)
##
## **Windowed**, under a wiped user directory (`APPDATA=<empty dir>`), on a throw-away vault in TEMP. Exit 0 = every
## check held, 1 = a check failed, 2 = could not run.

const Ed := preload("res://shell/markdown_editor.gd")

var _vp: SubViewport
var app: Node
var _bridge
var vw: VaultWindow
var ed
var te: TextEdit
var sp
var _root := ""
var _touch := false
var _phone := false
var _n := 0
var _fails: Array = []


func _ok(label: String, cond: bool, detail: String = "") -> void:
	_n += 1
	if cond:
		print("VFOLD  ok  %s" % label)
	else:
		_fails.append(label)
		print("VFOLD  !!  %s   %s" % [label, detail])


func _arg(name: String, dflt: String) -> String:
	var args := OS.get_cmdline_user_args()
	var i := args.find(name)
	if i >= 0 and i + 1 < args.size():
		return String(args[i + 1])
	return dflt


func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame


func _until(cond: Callable, ms: int) -> int:
	var t := Time.get_ticks_msec()
	while Time.get_ticks_msec() - t < ms:
		if cond.call():
			return Time.get_ticks_msec() - t
		await get_tree().process_frame
	return -1


func _write(path: String, text: String) -> void:
	DirAccess.make_dir_recursive_absolute(path.get_base_dir())
	var f := FileAccess.open(path, FileAccess.WRITE)
	f.store_string(text)
	f.close()


func _disk(rel: String) -> String:
	return FileAccess.get_file_as_string(_root + "/" + rel)


func _exists(rel: String) -> bool:
	return FileAccess.file_exists(_root + "/" + rel)


func _find_all(n: Node, cls: String, out: Array) -> void:
	if n.is_class(cls):
		out.append(n)
	for c in n.get_children():
		_find_all(c, cls, out)


func _find_button(n: Node, text: String) -> Button:
	if n is Button and String((n as Button).text).to_lower() == text.to_lower() and (n as Button).is_visible_in_tree():
		return n as Button
	for c in n.get_children():
		var r := _find_button(c, text)
		if r != null:
			return r
	return null


func _named(n: String) -> Control:
	return sp.find_child(n.validate_node_name(), true, false) as Control if sp != null and is_instance_valid(sp) else null


func _tree() -> Tree:
	var ts: Array = []
	_find_all(vw, "Tree", ts)
	return ts[0] if not ts.is_empty() else null


func _leaf_in(it: TreeItem, rel: String) -> TreeItem:
	if it == null:
		return null
	if it.get_metadata(0) is String and String(it.get_metadata(0)) == rel:
		return it
	var c := it.get_first_child()
	while c != null:
		var r := _leaf_in(c, rel)
		if r != null:
			return r
		c = c.get_next()
	return null


## A real mouse press and release inside `win` (an embedded window) at `local`, with `button`.
func _mouse(win: Window, local: Vector2, pressed: bool, button: int = MOUSE_BUTTON_LEFT) -> void:
	var p := Vector2(win.position) + local * win.content_scale_factor
	var ev := InputEventMouseButton.new()
	ev.button_index = button
	ev.pressed = pressed
	ev.position = p
	ev.global_position = p
	if _touch:
		ev.device = InputEvent.DEVICE_ID_EMULATION
	_vp.push_input(ev)
	await _frames(2)


func _tap(win: Window, local: Vector2, button: int = MOUSE_BUTTON_LEFT) -> void:
	await _mouse(win, local, true, button)
	await _mouse(win, local, false, button)


## A real click on `b`, which lives in the main viewport (the docked Note tab's frame), not in the window.
func _click_main(b: Control) -> void:
	var p := b.get_global_rect().get_center()
	for pressed in [true, false]:
		var ev := InputEventMouseButton.new()
		ev.button_index = MOUSE_BUTTON_LEFT
		ev.pressed = pressed
		ev.position = p
		ev.global_position = p
		_vp.push_input(ev)
		await _frames(2)
	await _frames(3)


## A REAL click on `b` (a control of the vault window).
func _press(b: Control) -> void:
	if b == null:
		_ok("a control to press exists", false)
		return
	var sc := b.get_parent()
	while sc != null and not (sc is ScrollContainer):
		sc = sc.get_parent()
	if sc != null:
		(sc as ScrollContainer).ensure_control_visible(b)
		await _frames(2)
	await _tap(vw, b.get_global_rect().get_center())
	await _frames(3)


func _type(s: String) -> void:
	for i in s.length():
		for pressed in [true, false]:
			var k := InputEventKey.new()
			k.pressed = pressed
			k.unicode = s.unicode_at(i)
			k.keycode = s.unicode_at(i) as Key
			_vp.push_input(k)
		await _frames(2)


func _key(code: Key, ctrl: bool = false) -> void:
	for pressed in [true, false]:
		var k := InputEventKey.new()
		k.pressed = pressed
		k.keycode = code
		k.physical_keycode = code
		k.ctrl_pressed = ctrl
		_vp.push_input(k)
	await _frames(3)


func _finish() -> void:
	print("VFOLD RESULT %d checks, %d failed%s" % [_n, _fails.size(), "" if _fails.is_empty() else " :: " + str(_fails)])
	get_tree().quit(0 if _fails.is_empty() else 1)


func _grab() -> void:
	ed = vw._editor
	te = ed.text_edit if ed != null else null
	sp = vw._side_panel


func _open(rel: String) -> void:
	if _phone:
		vw._set_side(false)
	vw._link_back.clear()
	vw._enter_note(rel)
	await _frames(6)
	_grab()


func _to_note() -> void:
	if _phone:
		vw._set_side(false)
		await _frames(3)
		_grab()


func _tab(key: String) -> void:
	if sp != null and is_instance_valid(sp) and not sp.visible and vw._side_toggle != null and vw._side_toggle.is_visible_in_tree():
		await _press(vw._side_toggle)
		_grab()
	var b := _named("SideTab_" + key)
	if b == null:
		sp.show_tab(key)
	else:
		await _press(b)
	await _frames(3)


func _dlg(ok_text: String, field: bool) -> ConfirmationDialog:
	var out: Array = []
	_find_all(app, "ConfirmationDialog", out)
	for d in out:
		var cd := d as ConfirmationDialog
		if cd.is_queued_for_deletion() or not cd.visible:
			continue
		if cd.get_ok_button().text != ok_text:
			continue
		var les: Array = []
		_find_all(cd, "LineEdit", les)
		var has_field := false
		for le in les:
			if (le as LineEdit).is_visible_in_tree() and not ((le.get_parent() is SpinBox)):
				has_field = true
		if has_field == field:
			return cd
	return null


func _wait_dlg(ok_text: String, field: bool, ms: int = 4000) -> ConfirmationDialog:
	var t := Time.get_ticks_msec()
	while Time.get_ticks_msec() - t < ms:
		var d := _dlg(ok_text, field)
		if d != null:
			return d
		await get_tree().process_frame
	return null


func _field_of(d: Node) -> LineEdit:
	var les: Array = []
	_find_all(d, "LineEdit", les)
	for le in les:
		if (le as LineEdit).is_visible_in_tree():
			return le
	return null


## Real keys into a prompt: tap its field, select all, type `name`, Enter.
func _type_name_and_enter(ok_text: String, name: String) -> void:
	var fld := _field_of(_dlg(ok_text, true))
	if fld != null:
		var w := fld.get_window()
		if w != null and w != get_tree().root and w.is_embedded():
			await _tap(w, fld.get_global_rect().get_center())
		else:
			var ev := InputEventMouseButton.new()
			ev.button_index = MOUSE_BUTTON_LEFT
			ev.position = fld.get_global_rect().get_center()
			ev.global_position = ev.position
			for pressed in [true, false]:
				ev.pressed = pressed
				_vp.push_input(ev.duplicate())
				await _frames(2)
	await _key(KEY_A, true)
	await _type(name)
	await _key(KEY_ENTER)
	await _frames(4)


## Renames through the whole flow after the prompt is up: name, Enter, confirm. True when the confirmation came.
func _finish_rename(name: String, confirm_text: String = "Rename") -> bool:
	var pd := await _wait_dlg("Rename", true)
	if pd == null:
		return false
	await _type_name_and_enter("Rename", name)
	var cd := await _wait_dlg(confirm_text, false)
	if cd == null:
		return false
	cd.get_ok_button().pressed.emit()
	await _frames(8)
	return true


func _vis_button(n: Node, nm: String) -> Button:
	if n is Button and n.name == nm and (n as Button).is_visible_in_tree():
		return n
	for c in n.get_children():
		var r := _vis_button(c, nm)
		if r != null:
			return r
	return null


func _links_of(rel: String) -> Array:
	var out: Array = []
	for l in _bridge.vault_all_links():
		if String((l as Dictionary).get("path", "")) == rel:
			out.append(l)
	return out


## Every `[[target]]` in every note of the vault on disk resolves to exactly one note (the crate's resolver).
func _all_links_resolve(label: String) -> void:
	var rx := RegEx.new()
	rx.compile("\\[\\[([^\\]|#]+)")
	var bad: Array = []
	var seen := 0
	for rel in _bridge.vault_list_files():
		for m in rx.search_all(_disk(String(rel))):
			seen += 1
			var r: Dictionary = _bridge.vault_resolve_link(String(rel), m.get_string(1))
			if String(r.get("status", "")) != "unique":
				bad.append("%s -> %s" % [rel, m.get_string(1)])
	_ok("%s: all %d links resolve to one note" % [label, seen], bad.is_empty() and seen > 0, str(bad))


## The path and status of each binding, keyed by the entity label.
func _bindings() -> Dictionary:
	var out := {}
	for l in _bridge.vault_all_links():
		out[String((l as Dictionary).get("entity_label", ""))] = l
	return out


func _folder_item(it: TreeItem, name: String) -> TreeItem:
	if it == null:
		return null
	if it.get_text(0) == name and String(it.get_metadata(0)) == "":
		return it
	var c := it.get_first_child()
	while c != null:
		var r := _folder_item(c, name)
		if r != null:
			return r
		c = c.get_next()
	return null


func _tree_menu_on(folder: String) -> PopupMenu:
	var tree := _tree()
	var it := _folder_item(tree.get_root(), folder.get_file())
	if it == null:
		return null
	var rect := tree.get_item_area_rect(it, 0)
	await _tap(vw, tree.get_global_rect().position + rect.position + Vector2(20, rect.size.y * 0.5), MOUSE_BUTTON_RIGHT)
	return vw.find_child("TreeMenu", true, false) as PopupMenu


## All the text a dialog shows: its own `dialog_text` plus every visible label (the phone's dialog form puts its body in labels).
func _dlg_text(d: Node) -> String:
	var parts: Array = [String((d as AcceptDialog).dialog_text)]
	var ls: Array = []
	_find_all(d, "Label", ls)
	_find_all(d, "RichTextLabel", ls)
	for l in ls:
		parts.append(String(l.get("text")))
	return " ".join(PackedStringArray(parts))


func _show_tree() -> void:
	if _phone:
		## The phone's one pane shows the editor while the open note is the picked one; un-picking it (the
		## note stays open in the window's state, so the folder move still has to follow it) shows the tree.
		vw._pick_file = ""
		vw._browse_phone_pane = "tree"
	vw._rebuild()
	await _frames(6)


func _ready() -> void:
	for a in OS.get_cmdline_user_args():
		var s := String(a)
		if s.begins_with("--") and not (s in ["--vp", "--touch", "--force-touch", "--density", "--debug"]):
			print("VFOLD ABORT unknown flag %s" % s)
			get_tree().quit(2)
			return
	if DisplayServer.get_name() == "headless":
		print("VFOLD REFUSED: run windowed")
		get_tree().quit(2)
		return
	var wd := Timer.new()
	wd.wait_time = 420.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("VFOLD WATCHDOG TIMEOUT")
		get_tree().quit(3))
	wd.start()
	var parts: PackedStringArray = _arg("--vp", "1600x1000").split("x")
	_touch = "--touch" in OS.get_cmdline_user_args() or "--force-touch" in OS.get_cmdline_user_args()
	if FileAccess.file_exists(VaultStore.PATH):
		DirAccess.remove_absolute(ProjectSettings.globalize_path(VaultStore.PATH))
	_vp = SubViewport.new()
	_vp.size = Vector2i(int(parts[0]), int(parts[1]))
	_vp.transparent_bg = false
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

	_root = OS.get_environment("TEMP").replace("\\", "/") + "/cartalith-vfolder-%d" % Time.get_ticks_msec()
	_bridge.generate({"seed": 4242, "width_km": 2400.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.45})
	await _frames(2)
	var t0 := Time.get_ticks_msec()
	while bool(_bridge.generating) and Time.get_ticks_msec() - t0 < 240000:
		await _frames(10)
	await _frames(30)
	var sets: Array = _bridge.settlements()
	_ok("CONTROL: the stand-in world has two settlements", sets.size() >= 2, str(sets.size()))
	if sets.size() < 2:
		_finish()
		return
	var s0: Dictionary = sets[0]
	var s1: Dictionary = sets[1]

	# == Fixture: three folders (Places with a nested Deep, Atlas, Other) and cross-links =========================
	var kelv := "# Kelvhold\n\nSee [[Ford]] and [[Journal/Log]].\n"
	var ford := "# Ford\n\nBack to [[Kelvhold]] and [Log](../../Journal/Log.md).\n"
	var log_text := "# Log\n\n[[Places/Kelvhold]] [[Places/Deep/Ford|the ford]] [[Index]]\n"
	_write(_root + "/Places/Kelvhold.md", kelv)
	_write(_root + "/Places/Deep/Ford.md", ford)
	_write(_root + "/Journal/Log.md", log_text)
	_write(_root + "/Atlas/Index.md", "# Index\n\nthe atlas\n")
	_write(_root + "/Other/Stray.md", "# Stray\n\nlinks [[Kelvhold]]\n")
	var conn: Dictionary = _bridge.vault_connect(_root, "Chronicle")
	_ok("CONTROL: the fixture vault binds", bool(conn.get("ok", false)), str(conn))
	_bridge.vault_refresh_backlinks(2000)
	var a0: Dictionary = _bridge.vault_attach("settlement", int(s0.get("tid", 0)), String(s0.get("name", "")), "Places/Kelvhold.md", "")
	var a1: Dictionary = _bridge.vault_attach("settlement", int(s1.get("tid", 0)), String(s1.get("name", "")), "Places/Deep/Ford.md", "")
	_ok("CONTROL: two settlements are bound to notes inside Places", bool(a0.get("ok", false)) and bool(a1.get("ok", false)), str(a0) + str(a1))
	DccSettings.set_vault_note_folder("settlement", "Places/Deep")
	DccSettings.set_vault_note_folder("landmark", "Journal")
	_all_links_resolve("CONTROL: before")
	app.open_vault_browse()
	await _frames(4)
	vw = app.vault_window
	await _open("Places/Deep/Ford.md")
	_ok("CONTROL: a note inside the folder is open", vw._browse_path == "Places/Deep/Ford.md")
	_phone = ed._phone if ed != null else false
	print("VFOLD info: viewport %s, phone=%s, touch=%s" % [_vp.size, _phone, _touch])
	await _show_tree()
	var tree := _tree()
	_ok("CONTROL: the file tree is on screen", tree != null)
	if tree == null:
		_finish()
		return
	var names0 := {}
	for k in ["Places", "Deep", "Atlas", "Other"]:
		names0[k] = _folder_item(tree.get_root(), k) != null
	_ok("CONTROL: the tree draws folder rows (selectable, with no note path)", names0["Places"] and names0["Deep"] and names0["Atlas"] and names0["Other"], str(names0))

	# == 1. Rename a folder from the tree menu ====================================================================
	print("== 1. rename ==")
	var pm := await _tree_menu_on("Places")
	_ok("1: a right click on a folder opens its own menu (Rename folder..., Move folder...)", pm != null and pm.item_count == 2 and pm.get_item_text(0) == "Rename folder..." and pm.get_item_text(1) == "Move folder...", "" if pm == null else str(pm.item_count))
	_ok("1: ... and did not open anything", vw._browse_path == "Places/Deep/Ford.md", vw._browse_path)
	if pm != null:
		pm.id_pressed.emit(0)
	await _frames(4)
	var pd := _dlg("Rename", true)
	_ok("1: the name prompt is up, holding the folder's name", pd != null and _field_of(pd) != null and _field_of(pd).text == "Places", "" if pd == null else _field_of(pd).text)
	var ok1 := await _finish_rename("Realms")
	_ok("1: the confirmation came and was accepted", ok1)
	_ok("1: every note is at its new path and the old folder is gone", _exists("Realms/Kelvhold.md") and _exists("Realms/Deep/Ford.md") and not _exists("Places/Kelvhold.md") and not DirAccess.dir_exists_absolute(_root + "/Places"))
	_ok("1: the moved notes kept their text, link for link", _disk("Realms/Kelvhold.md") == kelv and _disk("Realms/Deep/Ford.md") == ford, _disk("Realms/Deep/Ford.md"))
	_ok("1: the link from outside was rewritten (path links shortened, the by-name link left alone)", _disk("Journal/Log.md") == "# Log\n\n[[Kelvhold]] [[Ford|the ford]] [[Index]]\n", _disk("Journal/Log.md"))
	_ok("1: a by-name link in an untouched note is byte for byte", _disk("Other/Stray.md") == "# Stray\n\nlinks [[Kelvhold]]\n")
	_all_links_resolve("1")
	var b := _bindings()
	_ok("1: both bindings point at the moved notes and are Connected", b.size() == 2 and String((b[String(s0["name"])] as Dictionary).get("path", "")) == "Realms/Kelvhold.md" and String((b[String(s1["name"])] as Dictionary).get("path", "")) == "Realms/Deep/Ford.md" and String((b[String(s0["name"])] as Dictionary).get("status", "")) == "connected", str(b))
	_ok("1: a binding opens the right note (read through the vault by the binding's own path)", String(_bridge.vault_read_file(String((b[String(s0["name"])] as Dictionary).get("path", ""))) ).begins_with("# Kelvhold"))
	_ok("1: the remembered folders followed (Places/Deep -> Realms/Deep; an unrelated one untouched)", DccSettings.vault_note_folder("settlement") == "Realms/Deep" and DccSettings.vault_note_folder("landmark") == "Journal", str(DccSettings.vault_note_folder("settlement")))
	_ok("1: the OPEN note followed its folder and still shows its text", vw._browse_path == "Realms/Deep/Ford.md", vw._browse_path)
	_ok("1: the undo record is a folder rename", bool(vw._rename_last.get("dir", false)) and String(vw._rename_last.get("old", "")) == "Places")

	# == 2. Move a folder into another folder =====================================================================
	print("== 2. move ==")
	await _show_tree()
	pm = await _tree_menu_on("Realms")
	if pm != null:
		pm.id_pressed.emit(1)
	await _frames(4)
	var mp := await _wait_dlg("Move", true)
	_ok("2: the move prompt is up", mp != null)
	if mp != null:
		await _type_name_and_enter("Move", "Atlas")
	var mc := await _wait_dlg("Move folder", false)
	_ok("2: the confirmation says Move folder", mc != null)
	if mc != null:
		var shown := _dlg_text(mc)
		_ok("2: ... and names the notes and the links", "with the 2 notes in it" in shown and "other note" in shown, shown.left(200))
		mc.get_ok_button().pressed.emit()
	await _frames(8)
	_ok("2: the folder is inside Atlas on disk, beside Atlas's own note", _exists("Atlas/Realms/Kelvhold.md") and _exists("Atlas/Realms/Deep/Ford.md") and _exists("Atlas/Index.md") and not DirAccess.dir_exists_absolute(_root + "/Realms"))
	_ok("2: a relative link out of the folder was re-pointed one level deeper", _disk("Atlas/Realms/Deep/Ford.md") == "# Ford\n\nBack to [[Kelvhold]] and [Log](../../../Journal/Log.md).\n", _disk("Atlas/Realms/Deep/Ford.md"))
	_ok("2: Log.md still reads the same (every name is still unique)", _disk("Journal/Log.md") == "# Log\n\n[[Kelvhold]] [[Ford|the ford]] [[Index]]\n")
	_all_links_resolve("2")
	b = _bindings()
	_ok("2: the bindings follow again", String((b[String(s0["name"])] as Dictionary).get("path", "")) == "Atlas/Realms/Kelvhold.md" and String((b[String(s1["name"])] as Dictionary).get("path", "")) == "Atlas/Realms/Deep/Ford.md", str(b))
	_ok("2: the remembered folder followed", DccSettings.vault_note_folder("settlement") == "Atlas/Realms/Deep", str(DccSettings.vault_note_folder("settlement")))
	_ok("2: the open note followed", vw._browse_path == "Atlas/Realms/Deep/Ford.md", vw._browse_path)
	var snapshot := {}
	for rel in _bridge.vault_list_files():
		snapshot[String(rel)] = _disk(String(rel))

	# == 3. The refusals write nothing ==============================================================================
	print("== 3. refusals ==")
	vw._rename_plan("Atlas", "Atlas/Realms/Atlas", false, true)
	await _frames(4)
	_ok("3: a folder into its own descendant is refused with a clear message", "into itself" in vw._rename_message, vw._rename_message)
	vw._rename_plan("Atlas/Realms", "Atlas/Realms/Deep/Realms", false, true)
	await _frames(4)
	_ok("3: ... into a deeper descendant too", "into itself" in vw._rename_message, vw._rename_message)
	vw._rename_plan("Other", "Atlas", false, true)
	await _frames(4)
	_ok("3: a name that already exists is refused", "already exists" in vw._rename_message, vw._rename_message)
	vw._rename_plan("Other", "Bad:Name", false, true)
	await _frames(4)
	_ok("3: a name Windows cannot hold is refused", "Windows" in vw._rename_message, vw._rename_message)
	vw._rename_plan("Nowhere", "Z", false, true)
	await _frames(4)
	_ok("3: a folder that is not there is refused", "not a folder" in vw._rename_message, vw._rename_message)
	var same := true
	for rel in _bridge.vault_list_files():
		same = same and snapshot.get(String(rel), null) == _disk(String(rel))
	_ok("3: no note changed (same files, same bytes)", same and _bridge.vault_list_files().size() == snapshot.size())

	# == 4. Long press (touch) and F2 (desktop) ====================================================================
	print("== 4. gestures ==")
	await _show_tree()
	tree = _tree()
	var it := _folder_item(tree.get_root(), "Other")
	var rect := tree.get_item_area_rect(it, 0)
	var at := tree.get_global_rect().position + rect.position + Vector2(20, rect.size.y * 0.5)
	await _mouse(vw, at, true)
	await get_tree().create_timer(0.8).timeout
	await _frames(3)
	pm = vw.find_child("TreeMenu", true, false) as PopupMenu
	_ok("4: a long press on a folder row opens the folder menu", pm != null and pm.item_count == 2)
	await _mouse(vw, at, false)
	if pm != null:
		pm.hide()
	await _frames(4)
	await _show_tree()
	tree = _tree()
	it = _folder_item(tree.get_root(), "Other")
	rect = tree.get_item_area_rect(it, 0)
	at = tree.get_global_rect().position + rect.position + Vector2(20, rect.size.y * 0.5)
	await _mouse(vw, at, true)
	await _mouse(vw, at + Vector2(0, 3), false)
	await get_tree().create_timer(0.8).timeout
	_ok("4: a quick tap on a folder row opens no menu", vw.find_child("TreeMenu", true, false) == null or not (vw.find_child("TreeMenu", true, false) as PopupMenu).visible)
	await _show_tree()
	tree = _tree()
	it = _folder_item(tree.get_root(), "Other")
	if it != null:
		it.select(0)
	await _frames(4)
	tree.grab_focus()
	await _key(KEY_F2)
	await _frames(3)
	var f2 := _dlg("Rename", true)
	_ok("4: F2 on the selected folder asks for its new name (a hardware key: desktop and tablet; the phone has none)", _phone or (f2 != null and _field_of(f2) != null and _field_of(f2).text == "Other"), "" if f2 == null else _field_of(f2).text)
	if f2 != null:
		f2.hide()
	await _frames(3)

	# == 5. One undo step ===========================================================================================
	print("== 5. undo ==")
	_ok("5: the one undo record is the move", bool(vw._rename_last.get("dir", false)) and String(vw._rename_last.get("old", "")) == "Realms" and String(vw._rename_last.get("new", "")) == "Atlas/Realms", str(vw._rename_last))
	vw._undo_rename()
	var ud := await _wait_dlg("Undo rename", false)
	_ok("5: the undo asks once", ud != null)
	if ud != null:
		ud.get_ok_button().pressed.emit()
	await _frames(8)
	_ok("5: the folder is back at its first move's origin (Realms) with its notes", _exists("Realms/Kelvhold.md") and _exists("Realms/Deep/Ford.md") and not DirAccess.dir_exists_absolute(_root + "/Atlas/Realms"))
	_ok("5: the relative link out was put back", _disk("Realms/Deep/Ford.md") == ford, _disk("Realms/Deep/Ford.md"))
	_all_links_resolve("5")
	b = _bindings()
	_ok("5: the bindings followed back", String((b[String(s0["name"])] as Dictionary).get("path", "")) == "Realms/Kelvhold.md", str(b))
	_ok("5: the remembered folder followed back", DccSettings.vault_note_folder("settlement") == "Realms/Deep", str(DccSettings.vault_note_folder("settlement")))
	_finish()
