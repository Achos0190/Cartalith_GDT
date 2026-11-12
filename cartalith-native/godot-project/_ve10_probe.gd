extends Node
## The vault editor's open edges (`VAULT_EDITOR_PROPOSAL.md` s.7, VE-5 and VE-6): rename from the file tree and from
## the docked Note tab (both through the ONE rename flow), the Properties tab's entity binding (Change, Unlink,
## Attach) and Private switch, and a text search from a tag.
##
##   Godot_v4.7.2-stable_win64_console.exe --path . _ve10_probe.tscn -- --vp 1600x1000 [--touch] [--shots DIR]
##   ... -- --vp 1080x2340 --force-touch   (the phone leg)
##
## **Windowed**, under a wiped user directory (`APPDATA=<empty dir>`), on a throw-away vault in TEMP. Exit 0 = every
## check held, 1 = a check failed, 2 = could not run. Real mouse presses (a right click on a tree row, clicks on the
## panel's buttons and the docked tab's button), real typed keys into the name prompts, F2 as a real key event;
## dialog OK buttons by their `pressed` signal and the tree menu's row by `id_pressed` (the precedent of
## `_ve6_probe.gd`). Every file assertion re-reads the disk.

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
		print("VE10  ok  %s" % label)
	else:
		_fails.append(label)
		print("VE10  !!  %s   %s" % [label, detail])


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
	print("VE10 RESULT %d checks, %d failed%s" % [_n, _fails.size(), "" if _fails.is_empty() else " :: " + str(_fails)])
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


func _ready() -> void:
	for a in OS.get_cmdline_user_args():
		var s := String(a)
		if s.begins_with("--") and not (s in ["--vp", "--touch", "--shots", "--force-touch", "--debug"]):
			print("VE10 ABORT unknown flag %s" % s)
			get_tree().quit(2)
			return
	if DisplayServer.get_name() == "headless":
		print("VE10 REFUSED: run windowed")
		get_tree().quit(2)
		return
	var wd := Timer.new()
	wd.wait_time = 420.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("VE10 WATCHDOG TIMEOUT")
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

	_root = OS.get_environment("TEMP").replace("\\", "/") + "/cartalith-ve10-%d" % Time.get_ticks_msec()
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

	# == Fixture: Brindle is the OPEN note and links to Wren, the note the tree will rename ===================
	var brindle := "---\naliases: [Brin]\n---\n# Brindle\n\nIntro.\n\n## Trade\n\nSalt goes to [[Wren]] and [[Wren#Sec|the section]].\n"
	var wren := "# Wren\n\n## Sec\n\nback to [[Brindle]] #saltroad\n"
	var hub := "# Hub\n\nsee [[Wren]] here\n"
	_write(_root + "/Brindle.md", brindle)
	_write(_root + "/Wren.md", wren)
	_write(_root + "/Hub.md", hub)
	_write(_root + "/Salt1.md", "# Salt1\n\nthe #saltroad runs east\n")
	_write(_root + "/Salt2.md", "---\ntags: [saltroad]\n---\n# Salt2\n\nbody\n")
	_write(_root + "/Prose.md", "# Prose\n\nA walk on the saltroad, written without the tag mark.\n")
	_write(_root + "/Private/Hidden.md", "# Hidden\n\nin a private folder\n")
	var conn: Dictionary = _bridge.vault_connect(_root, "Chronicle")
	_ok("CONTROL: the fixture vault binds", bool(conn.get("ok", false)), str(conn))
	_bridge.vault_refresh_backlinks(2000)
	var att: Dictionary = _bridge.vault_attach("settlement", int(s0.get("tid", 0)), String(s0.get("name", "")), "Brindle.md", "")
	_ok("CONTROL: Brindle.md is bound to settlement 0", bool(att.get("ok", false)), str(att))
	var att_h: Dictionary = _bridge.vault_attach("settlement", int(s0.get("tid", 0)), String(s0.get("name", "")), "Brindle.md", "Trade")
	_ok("CONTROL: ... and a second link selects the Trade heading", bool(att_h.get("ok", false)), str(att_h))
	app.open_vault_browse()
	await _frames(4)
	vw = app.vault_window
	await _open("Brindle.md")
	_ok("CONTROL: Brindle.md is open in Write", ed != null and vw._browse_path == "Brindle.md")
	if ed == null:
		_finish()
		return
	_phone = ed._phone
	print("VE10 info: viewport %s, phone=%s, touch=%s, window %s" % [_vp.size, _phone, _touch, vw.size])

	# == 1. Rename a note that is NOT open, from the file tree ====================================================
	print("== 1. file tree ==")
	## Edit the open note first: the flow must keep that text (it saves before it moves, and reads the file again
	## after, because the move rewrites links inside the note).
	await _to_note()
	te.grab_focus()
	te.set_caret_line(te.get_line_count() - 1)
	te.set_caret_column(te.get_line(te.get_line_count() - 1).length())
	await _type("pre")
	var tree := _tree()
	var wleaf := _leaf_in(tree.get_root(), "Wren.md") if tree != null else null
	## The two-pane browser (desktop, tablet) draws the tree; the phone's one-pane browser draws none beside an open note, which
	## the next branch already relied on. Both halves are asserted, so neither form can pass for the other.
	_ok("1: the tree lists Wren.md on desktop and tablet, and the phone's one-pane browser draws no tree", (wleaf != null) == (not _phone))
	if _phone or wleaf == null:
		print("VE10 info: the tree is not on screen in this form; the tree menu is driven by its own call")
		vw.request_rename_of("Wren.md")
	else:
		var rect := tree.get_item_area_rect(wleaf, 0)
		await _tap(vw, tree.get_global_rect().position + rect.position + Vector2(20, rect.size.y * 0.5), MOUSE_BUTTON_RIGHT)
		var pm := vw.find_child("TreeMenu", true, false) as PopupMenu
		_ok("1: a right click on a note opens the tree menu", pm != null and pm.item_count == 1 and pm.get_item_text(0) == "Rename...")
		_ok("1: ... and did NOT open that note (Brindle is still the one open)", vw._browse_path == "Brindle.md", vw._browse_path)
		if pm != null:
			pm.id_pressed.emit(0)
	await _frames(4)
	var pd_up := _dlg("Rename", true)
	_ok("1: the name prompt is up, holding the note's name", pd_up != null and _field_of(pd_up) != null and _field_of(pd_up).text == "Wren", "" if pd_up == null else str(_field_of(pd_up).text))
	var ok1 := await _finish_rename("Wren Keep")
	_ok("1: the confirmation named the plan and was accepted", ok1)
	_ok("1: Wren.md moved to Wren Keep.md on disk", _exists("Wren Keep.md") and not _exists("Wren.md"))
	_ok("1: the moved note kept its text", _disk("Wren Keep.md") == wren.replace("[[Brindle]]", "[[Brindle]]"))
	_ok("1: Hub.md links were rewritten", _disk("Hub.md") == "# Hub\n\nsee [[Wren Keep]] here\n", _disk("Hub.md"))
	var b_expect := "---\naliases: [Brin]\n---\n# Brindle\n\nIntro.\n\n## Trade\n\nSalt goes to [[Wren Keep]] and [[Wren Keep#Sec|the section]].\npre"
	_ok("1: the OPEN note's links on disk were rewritten (alias and heading kept)", _disk("Brindle.md") == b_expect, _disk("Brindle.md"))
	_grab()
	_ok("1: the window still holds Brindle.md (a note that was not renamed stays open)", vw._browse_path == "Brindle.md" and ed != null)
	_ok("1: ... and the editor shows the REWRITTEN text, not the stale one", te != null and te.text == b_expect, "" if te == null else te.text)
	var hash_now: String = String(_bridge.vault_read_file_for_edit("Brindle.md").get("hash", ""))
	_ok("1: ... with the hash of what is on disk now", vw._browse_hash == hash_now)
	await _to_note()
	te.grab_focus()
	te.set_caret_line(te.get_line_count() - 1)
	te.set_caret_column(te.get_line(te.get_line_count() - 1).length())
	await _type("Q")
	var ms := await _until(func(): return _disk("Brindle.md").ends_with("Q"), 3500)
	_ok("1: typing after the rename autosaves with no conflict (%d ms)" % ms, ms >= 0 and vw._save_block.is_empty(), str(vw._save_block))
	_ok("1: the undo record names the moved note, not the open one", String(vw._rename_last.get("new", "")) == "Wren Keep.md" and String(vw._rename_last.get("old", "")) == "Wren.md", str(vw._rename_last))
	## Undo from here (the open note is Brindle, not the moved one).
	vw._undo_rename()
	var okU := false
	var ud := await _wait_dlg("Undo rename", false)
	if ud != null:
		ud.get_ok_button().pressed.emit()
		okU = true
	await _frames(8)
	_ok("1: Undo renames it back though it is not the open note", okU and _exists("Wren.md") and not _exists("Wren Keep.md"))
	_ok("1: ... and the links are back (Hub.md byte for byte)", _disk("Hub.md") == hub, _disk("Hub.md"))

	# == 2. F2 on the selected tree row ===========================================================================
	print("== 2. F2 ==")
	tree = _tree()
	if not _phone and tree != null:
		wleaf = _leaf_in(tree.get_root(), "Wren.md")
		wleaf.select(0)
		await _frames(6)
		_grab()
		tree = _tree()
		if tree != null:
			tree.grab_focus()
		await _key(KEY_F2)
		var p2 := await _wait_dlg("Rename", true, 3000)
		_ok("2: F2 on a selected note raises the rename prompt", p2 != null)
		if p2 != null:
			p2.get_cancel_button().pressed.emit()
			await _frames(4)
		await _open("Brindle.md")
	else:
		print("VE10 info: F2 leg skipped on the phone (no hardware keyboard on the tree)")

	# == 3. Rename from the docked Note tab =======================================================================
	print("== 3. docked tab ==")
	vw.hide()
	await _frames(8)
	app.select("settlement", 0)  ## select() takes the settlement's INDEX in settlements() (s0 is index 0), not its tid
	await _frames(6)
	var tab_btn := _vis_button(app, "CtxTabNote")
	## The Context panel and its Note tab are desktop and tablet; the phone's form of the panel is the selection sheet
	## (`DccApp.open_context_tab()`'s own doc), which has no tab strip, so on the phone the tab must be ABSENT.
	_ok("3: the Context panel draws a NOTE tab for the selected place (the phone has no tab strip)", (tab_btn != null) == (not _phone))
	if tab_btn != null:
		await _click_main(tab_btn)
	_ok("3: the tab docks Brindle.md (desktop and tablet)", _phone or await _until(func(): return vw.is_docked() and vw.docked_path() == "Brindle.md", 3000) >= 0)
	var slot := app.find_child("NoteSlot", true, false) as Control
	var rbtn := _vis_button(slot, "NoteRename") if slot != null else null
	_ok("3: the docked editor has a Rename button (desktop and tablet)", _phone or (rbtn != null and rbtn.is_visible_in_tree()))
	if rbtn != null:
		await _click_main(rbtn)
		var ok3 := await _finish_rename("Brindle Dock")
		_ok("3: renamed from the docked tab through the same prompt and confirmation", ok3)
		_ok("3: the note moved on disk", _exists("Brindle Dock.md") and not _exists("Brindle.md"))
		_ok("3: the tab follows it: docked on the new path", await _until(func(): return vw.is_docked() and vw.docked_path() == "Brindle Dock.md", 3000) >= 0)
		var eds: Array = []
		_find_all(slot, "TextEdit", eds)
		_ok("3: ... with exactly ONE editor, inside the tab", eds.size() == 1)
		_ok("3: the map's links to the note followed the move (both)", _links_of("Brindle Dock.md").size() == 2, str(_links_of("Brindle Dock.md").size()))
		vw._undo_rename()
		var ud3 := await _wait_dlg("Undo rename", false)
		if ud3 != null:
			ud3.get_ok_button().pressed.emit()
		await _frames(8)
		_ok("3: Undo from the docked tab renames it back", _exists("Brindle.md") and not _exists("Brindle Dock.md"))
		_ok("3: ... and the tab shows the original name again", await _until(func(): return vw.is_docked() and vw.docked_path() == "Brindle.md", 3000) >= 0)
	vw.release_dock()
	await _frames(4)
	app.open_vault_browse()
	await _frames(6)
	await _open("Brindle.md")

	# == 4. The entity binding ====================================================================================
	print("== 4. entity binding ==")
	await _tab("props")
	var rows: Array = []
	for l in _links_of("Brindle.md"):
		rows.append(l)
	var bind_nodes: Array = []
	if sp != null:
		for c in sp.find_children("Bind_*", "VBoxContainer", true, false):
			bind_nodes.append(c)
	_ok("4: Properties lists one row per link (two)", bind_nodes.size() == 2 and rows.size() == 2, "%d rows, %d links" % [bind_nodes.size(), rows.size()])
	var txt_all := ""
	for nd in bind_nodes:
		var ls: Array = []
		_find_all(nd, "Label", ls)
		for l in ls:
			txt_all += (l as Label).text + " | "
	_ok("4: a row names the kind, the place and the link state", txt_all.contains("Settlement") and txt_all.contains(String(s0.get("name", "?"))) and (txt_all.contains("stale") or txt_all.contains("connected") or txt_all.contains("cached") or txt_all.contains("local changes")), txt_all)
	if _phone:
		var any_btn := _named("BindChange")
		_ok("4: phone: Change... is at least 44 px tall", any_btn != null and any_btn.size.y >= 43.5, "" if any_btn == null else str(any_btn.size))
	# Change the heading-selecting link to s1: the heading must be kept, the old link gone.
	var hl: Dictionary = {}
	for l in rows:
		if String((l as Dictionary).get("selection_label", "")) == "## Trade":
			hl = l
	var disk_before := _disk("Brindle.md")
	var res: Dictionary = vw.rebind_note_entity(String(hl.get("link_id", "")), String(s1.get("name", "")).to_upper())
	_ok("4: Change... to another place (typed in capitals) succeeds", bool(res.get("ok", false)), str(res))
	var after := _links_of("Brindle.md")
	var moved := false
	var old_gone := true
	for l in after:
		var ld: Dictionary = l
		if int(ld.get("entity_id", -1)) == int(s1.get("tid", -2)) and String(ld.get("selection_label", "")) == "## Trade":
			moved = true
		if String(ld.get("link_id", "")) == String(hl.get("link_id", "x")):
			old_gone = false
	_ok("4: the new link points at the new place and keeps the Trade heading", moved, str(after))
	_ok("4: the old link is gone and the count is unchanged", old_gone and after.size() == 2, str(after.size()))
	_ok("4: the note's text was not touched by re-linking", _disk("Brindle.md") == disk_before)
	var bad: Dictionary = vw.rebind_note_entity(String((after[0] as Dictionary).get("link_id", "")), "No Such Place Anywhere")
	_ok("4: an unknown place is refused in words and changes nothing", not bool(bad.get("ok", true)) and _links_of("Brindle.md").size() == 2)
	var same: Dictionary = vw.rebind_note_entity(String(hl.get("link_id", "")), String(s0.get("name", "")))
	var ids_before := []
	for l in _links_of("Brindle.md"):
		ids_before.append(String((l as Dictionary).get("link_id", "")))
	_ok("4: re-pointing a link at the place it already has does not delete it", _links_of("Brindle.md").size() >= 2, str(same))
	# Unlink by a real click.
	await _frames(4)
	var un := _named("BindUnlink")
	_ok("4: an Unlink button is on each row", un != null)
	var count_before := _links_of("Brindle.md").size()
	if un != null:
		await _press(un)
	_ok("4: a real click on Unlink removes exactly one link", _links_of("Brindle.md").size() == count_before - 1, "%d -> %d" % [count_before, _links_of("Brindle.md").size()])
	_ok("4: ... and never touches the note", _disk("Brindle.md") == disk_before)
	# Attach (the note now may have 1 link): the Attach button raises the prompt.
	var atb := _named("BindAttach")
	_ok("4: 'Attach to a place...' is offered", atb != null)
	if atb != null:
		await _press(atb)
		var ap := await _wait_dlg("Attach", true)
		_ok("4: ... and raises a place-name prompt", ap != null)
		if ap != null:
			await _type_name_and_enter("Attach", String(s1.get("name", "")))
			_ok("4: typing a place's name and Enter attaches the note to it", _links_of("Brindle.md").size() == count_before, str(_links_of("Brindle.md").size()))

	# == 5. The Private switch ====================================================================================
	print("== 5. private ==")
	await _tab("props")
	var sw := _named("PrivateSwitch") as CheckButton
	_ok("5: the Properties tab has a Private switch, off for a public note", sw != null and not sw.button_pressed)
	await _to_note()
	var before_text: String = te.text
	await _tab("props")
	sw = _named("PrivateSwitch") as CheckButton
	if sw != null:
		await _press(sw)
	await _frames(4)
	_grab()
	var after_on: String = te.text
	_ok("5: switching it on adds `private: true` to the front matter", after_on.contains("private: true"), after_on)
	_ok("5: ... and leaves every other line byte-identical", after_on.replace("private: true\n", "") == before_text, after_on)
	var ms5 := await _until(func(): return _disk("Brindle.md").contains("private: true"), 3500)
	_ok("5: ... and the autosave writes it (%d ms)" % ms5, ms5 >= 0)
	await _tab("props")
	sw = _named("PrivateSwitch") as CheckButton
	_ok("5: the switch now reads on", sw != null and sw.button_pressed)
	if sw != null:
		await _press(sw)
	await _frames(4)
	_grab()
	_ok("5: switching it off removes the property and restores the original text", te.text == before_text, te.text)
	# A note private for another reason: the switch cannot lift that, and says so.
	await _open("Private/Hidden.md")
	await _tab("props")
	sw = _named("PrivateSwitch") as CheckButton
	_ok("5: a note in Private/ shows the switch off", sw != null and not sw.button_pressed)
	var texts := ""
	if sp != null:
		var ls: Array = []
		_find_all(sp, "Label", ls)
		for l in ls:
			texts += (l as Label).text + " | "
	_ok("5: ... and says it is already private and why", texts.contains("Private already, by"), texts)

	# == 6. A text search from a tag ==============================================================================
	print("== 6. tag search ==")
	await _open("Brindle.md")
	await _tab("tags")
	var tg := _named("Tag_saltroad")
	_ok("6: the Tags tab lists #saltroad", tg != null)
	if tg != null:
		await _press(tg)
		var sb := _named("TagSearch_saltroad")
		_ok("6: an open tag offers 'Search the text for #saltroad'", sb != null)
		if sb != null:
			await _press(sb)
	_ok("6: the search ran for #saltroad", vw._search_query == "#saltroad", vw._search_query)
	## The browser's own field exists in the two-pane browser only; the phone's one-pane form has none, and the results box shows the hits.
	var field_live: bool = vw._search_field != null and is_instance_valid(vw._search_field)
	_ok("6: ... in the browser's own field (the phone's one-pane browser has none)", (field_live and vw._search_field.text == "#saltroad") if not _phone else not field_live)
	var hits: Array = (vw._search_result as Dictionary).get("hits", [])
	var rels: Array = []
	for h in hits:
		rels.append(String((h as Dictionary).get("rel", "")))
	_ok("6: it finds the inline-tag notes (Salt1, Wren)", rels.has("Salt1.md") and rels.has("Wren.md"), str(rels))
	_ok("6: ... and not the prose note that only says the word", not rels.has("Prose.md"), str(rels))
	tree = _tree()
	_ok("6: the file tree still lists every note (a #tag is not a name filter); the phone's one-pane browser draws no tree",
		(tree != null and _leaf_in(tree.get_root(), "Hub.md") != null) if not _phone else (tree == null or _leaf_in(tree.get_root(), "Hub.md") == null))

	_finish()
