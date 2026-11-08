extends Node
## VE-6 of `VAULT_EDITOR_PROPOSAL.md` s.7: rename with link rewriting, and create-from-link binding the
## entity. Bar: *links survive the rename in every note that had one, aliases and headings kept, and the
## KnowledgeLink follows.*
##
##   Godot_v4.7.2-stable_win64_console.exe --path . _ve6_probe.tscn -- --vp 1600x1000 [--touch] [--shots DIR]
##   ... -- --vp 1080x2340 --force-touch   (the phone leg)
##
## **Windowed**, under a wiped user directory (`APPDATA=<empty dir>`), on a throw-away vault in TEMP. Exit 0 =
## every check held, 1 = a check failed, 2 = could not run. Real clicks on the side-panel buttons, real typed
## keys into the name prompt and the editor; dialog buttons by their `pressed` signal (the precedent of
## `_ve4_probe.gd`); the link menu's row by `id_pressed`. Every file assertion re-reads the disk.
##
## What it proves: a name already taken or unusable is refused in words with nothing changed; the plan names
## the count per note and warns that a read-only note cannot be rewritten BEFORE anything moves; an outside
## change while deciding refuses the whole plan and writes nothing; the confirmed rename moves the note,
## rewrites wiki, alias, heading and Markdown links and the note's own self-link, leaves code spans and
## unrelated notes byte-identical, never touches the read-only note and says so; the window keeps one note
## focus (the editor is for the new path, the session undo re-keyed); the KnowledgeLink and the backlink
## index follow; the Note tab docks the new path; Undo renames back and restores every file byte for byte;
## Ctrl+Z walks back across the rename to the first-opened text; and creating a note from an entity link
## binds a KnowledgeLink to it.

const Ed := preload("res://shell/markdown_editor.gd")

var _vp: SubViewport
var app: Node
var _bridge
var vw: VaultWindow
var ed
var te: TextEdit
var sp  ## the side panel
var _root := ""
var _shots := ""
var _touch := false
var _phone := false
var _n := 0
var _fails: Array = []
var _ro_paths: Array = []


func _ok(label: String, cond: bool, detail: String = "") -> void:
	_n += 1
	if cond:
		print("VE6  ok  %s" % label)
	else:
		_fails.append(label)
		print("VE6  !!  %s   %s" % [label, detail])


func _arg(name: String, dflt: String) -> String:
	var args := OS.get_cmdline_user_args()
	var i := args.find(name)
	if i >= 0 and i + 1 < args.size():
		return String(args[i + 1])
	return dflt


func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame


func _wait(ms: int) -> void:
	var t := Time.get_ticks_msec()
	while Time.get_ticks_msec() - t < ms:
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


func _mouse(win: Window, local: Vector2, pressed: bool) -> void:
	var p := Vector2(win.position) + local * win.content_scale_factor
	var ev := InputEventMouseButton.new()
	ev.button_index = MOUSE_BUTTON_LEFT
	ev.pressed = pressed
	ev.position = p
	ev.global_position = p
	if _touch:
		ev.device = InputEvent.DEVICE_ID_EMULATION
	_vp.push_input(ev)
	await _frames(2)


func _tap(win: Window, local: Vector2) -> void:
	await _mouse(win, local, true)
	await _mouse(win, local, false)


## A REAL click on `b`: scrolls it into view if its scroll container hides it, then taps its centre.
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
	if OS.get_cmdline_user_args().has("--debug"):
		print("VE6 dbg press %s rect=%s vw.pos=%s vw.size=%s scale=%s vis=%s sp=%s ed=%s" % [b.name, b.get_global_rect(), vw.position, vw.size, vw.content_scale_factor, b.is_visible_in_tree(), sp.get_global_rect() if sp != null else null, ed.get_global_rect() if ed != null else null])
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
	for p in _ro_paths:
		FileAccess.set_read_only_attribute(p, false)
	print("VE6 RESULT %d checks, %d failed%s" % [_n, _fails.size(), "" if _fails.is_empty() else " :: " + str(_fails)])
	get_tree().quit(0 if _fails.is_empty() else 1)


## Opens `rel` in the editor the way a link follow does, with the editor and panel handles refreshed.
func _open(rel: String) -> void:
	if _phone:
		vw._set_side(false)  ## on the phone the panel is a sheet over the note; opening a note by hand needs the note
	vw._link_back.clear()
	vw._enter_note(rel)
	await _frames(6)
	_grab()


func _grab() -> void:
	ed = vw._editor
	te = ed.text_edit if ed != null else null
	sp = vw._side_panel


## Phone only: closes the sheet so the editor is the thing on screen (keys and carets go to it).
func _to_note() -> void:
	if _phone:
		vw._set_side(false)
		await _frames(3)
		_grab()


func _tab(key: String) -> void:
	if sp != null and is_instance_valid(sp) and not sp.visible and vw._side_toggle != null and vw._side_toggle.is_visible_in_tree():
		await _press(vw._side_toggle)  ## the phone closes the sheet when it takes you back to text
		_grab()
	var b := _named("SideTab_" + key)
	if b == null:
		sp.show_tab(key)
	else:
		await _press(b)
	await _frames(3)


## The visible, non-blank lines of the whole text split the way a byte-comparison wants them.
func _shot(name: String) -> void:
	if _shots == "":
		return
	DirAccess.make_dir_recursive_absolute(_shots)
	await _frames(4)
	_vp.get_texture().get_image().save_png(_shots + "/ve6_%s_%s.png" % [("phone" if _phone else ("touch" if _touch else "desk")), name])


func _lines(t: String) -> PackedStringArray:
	return t.split("\n")




## A visible ConfirmationDialog whose OK button says `ok_text`; `field` true wants one that holds a LineEdit
## (the name prompt), false one that does not (a confirmation). Null when none is showing.
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


func _texts_under(n: Node) -> String:
	var s := ""
	if n is Label:
		s += (n as Label).text + " "
	elif n is RichTextLabel:
		s += (n as RichTextLabel).get_parsed_text() + " "
	elif n is AcceptDialog:
		s += (n as AcceptDialog).dialog_text + " "
	for c in n.get_children():
		s += _texts_under(c)
	return s


func _field_of(d: Node) -> LineEdit:
	var les: Array = []
	_find_all(d, "LineEdit", les)
	for le in les:
		if (le as LineEdit).is_visible_in_tree():
			return le
	return null


## Real keys into the name prompt: select all, type `name`, Enter.
func _type_name_and_enter(name: String) -> void:
	## A real tap on the field first, so it holds the keyboard on every form (the phone shows the prompt as a
	## sheet that does not take focus by itself).
	var fld := _field_of(_dlg("Rename", true))
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
		if OS.get_cmdline_user_args().has("--debug"):
			print("VE6 dbg field window=%s embedded=%s focus=%s" % [w, w.is_embedded() if w != null else null, fld.has_focus()])
	await _key(KEY_A, true)
	await _type(name)
	await _key(KEY_ENTER)
	await _frames(4)


func _mode_str() -> String:
	return "phone" if _phone else ("touch" if _touch else "desk")


func _ready() -> void:
	for a in OS.get_cmdline_user_args():
		var s := String(a)
		if s.begins_with("--") and not (s in ["--vp", "--touch", "--shots", "--force-touch", "--debug"]):
			print("VE6 ABORT unknown flag %s" % s)
			get_tree().quit(2)
			return
	if DisplayServer.get_name() == "headless":
		print("VE6 REFUSED: run windowed")
		get_tree().quit(2)
		return
	var wd := Timer.new()
	wd.wait_time = 420.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("VE6 WATCHDOG TIMEOUT")
		for p in _ro_paths:
			FileAccess.set_read_only_attribute(p, false)
		get_tree().quit(3))
	wd.start()
	var parts: PackedStringArray = _arg("--vp", "1600x1000").split("x")
	_shots = _arg("--shots", "")
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

	# == The fixture vault (names no generated place can share) ================
	_root = OS.get_environment("TEMP").replace("\\", "/") + "/cartalith-ve6-%s-%d" % [_mode_str(), Time.get_ticks_msec()]
	_bridge.generate({"seed": 4242, "width_km": 2400.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.45})
	await _frames(2)
	var t0 := Time.get_ticks_msec()
	while bool(_bridge.generating) and Time.get_ticks_msec() - t0 < 240000:
		await _frames(10)
	await _frames(30)
	var sets: Array = _bridge.settlements()
	_ok("CONTROL: the stand-in world has two settlements (a small generated world)", sets.size() >= 2, str(sets.size()))
	if sets.size() < 2:
		_finish()
		return
	var s0: Dictionary = sets[0]
	var s1: Dictionary = sets[1]
	var orig_target := "---\naliases: [Brin]\n---\n# Brindle\n\nIntro.\n\n## Trade\n\nSalt. See [[Brindle]] and [[#Trade]].\n"
	var orig_l1 := "# L1\n\none [[Brindle]] two\na [[Brindle|the grey town]] b\nc [[Brindle#Trade]] d\ne [md](Brindle.md) f\n"
	var orig_l2 := "# L2\n\nx [[Brindle#Trade|shown]] y\n`[[Brindle]]` stays in code\n"
	var orig_l3 := "# L3\n\nro [[Brindle]]\n"
	var orig_other := "# Other\n\nno links here\n"
	_write(_root + "/Brindle.md", orig_target)
	_write(_root + "/L1.md", orig_l1)
	_write(_root + "/L2.md", orig_l2)
	_write(_root + "/L3.md", orig_l3)
	_write(_root + "/Other.md", orig_other)
	_write(_root + "/Taken.md", "# Taken\n")
	_write(_root + "/Hub.md", "# Hub\n\nsee [[%s]] here\n" % String(s1.get("name", "")))
	var conn: Dictionary = _bridge.vault_connect(_root, "Chronicle")
	_ok("CONTROL: the fixture vault binds", bool(conn.get("ok", false)), str(conn))
	_bridge.vault_refresh_backlinks(2000)
	var att: Dictionary = _bridge.vault_attach("settlement", int(s0.get("tid", 0)), String(s0.get("name", "")), "Brindle.md", "")
	_ok("CONTROL: Brindle.md is bound to a settlement (a KnowledgeLink)", bool(att.get("ok", false)), str(att))
	app.open_vault_browse()
	await _frames(4)
	vw = app.vault_window
	var ts: Array = []
	_find_all(vw, "Tree", ts)
	var tree: Tree = ts[0] if not ts.is_empty() else null
	var leaf := _leaf_in(tree.get_root(), "Brindle.md") if tree != null else null
	_ok("CONTROL: Brindle.md is in the tree", leaf != null)
	if leaf == null:
		_finish()
		return
	leaf.select(0)
	await _frames(4)
	var open_btn := _find_button(vw, "Open to edit")
	if open_btn != null:
		await _press(open_btn)
	else:
		## The phone's one-pane browser did not offer the button in this build of the probe's fixture (a
		## generated world precedes it); entering the note by the same call a link follow makes is the fallback.
		print("VE6 info: no Open to edit button; entering the note as a link follow does")
		vw._enter_note("Brindle.md")
	await _frames(4)
	_grab()
	_ok("CONTROL: the note opens in Write with a side panel", ed != null and sp != null)
	if sp == null:
		_finish()
		return
	_phone = ed._phone
	if not sp.visible:
		await _press(vw._side_toggle)
		_grab()
	print("VE6 info: viewport %s, phone=%s, touch=%s, window %s" % [_vp.size, _phone, _touch, vw.size])

	# Type into the note so the session undo has something to walk back over: a real key press, saved by the rename.
	await _to_note()
	te.grab_focus()
	te.set_caret_line(te.get_line_count() - 1)
	te.set_caret_column(te.get_line(te.get_line_count() - 1).length())
	await _type("ZZ")
	var typed_text: String = te.text
	_ok("typing reached the editor", typed_text == orig_target + "ZZ" or typed_text.ends_with("ZZ"), typed_text)

	# == 1. Properties offers Rename, and refusals change nothing ===============
	await _tab("props")
	var rb := _named("NoteRename")
	_ok("Properties has a Rename… button", rb != null)
	if _phone and rb != null:
		_ok("phone: the Rename button is at least 44 px tall", rb.size.y >= 43.5, str(rb.size))
	_ok("no undo is offered before a rename", _named("NoteRenameUndo") == null)
	var before := {}
	for f in ["Brindle.md", "L1.md", "L2.md", "L3.md", "Other.md", "Taken.md"]:
		before[f] = _disk(f)

	# a name that is taken is refused and nothing moves
	await _press(rb)
	var pd := await _wait_dlg("Rename", true)
	_ok("pressing Rename… opens a name prompt holding the current name", pd != null and _field_of(pd) != null and _field_of(pd).text == "Brindle", "" if pd == null else str(_field_of(pd).text if _field_of(pd) != null else "<nofield>"))
	_ok("... and saved the note first (ZZ is on disk)", _disk("Brindle.md").ends_with("ZZ"), _disk("Brindle.md"))
	before["Brindle.md"] = _disk("Brindle.md")
	await _type_name_and_enter("Taken")
	_ok("a name already in the vault is refused, in words", vw._rename_message.contains("already exists"), vw._rename_message)
	_ok("... and nothing was moved or rewritten", _disk("Brindle.md") == before["Brindle.md"] and _disk("L1.md") == before["L1.md"] and _disk("Taken.md") == before["Taken.md"] and not FileAccess.file_exists(_root + "/Taken2.md"))
	_ok("... and no confirmation was raised", _dlg("Rename", false) == null)
	await _press(_named("NoteRename"))
	pd = await _wait_dlg("Rename", true)
	await _type_name_and_enter("bad:name")
	_ok("a name Windows cannot hold is refused", vw._rename_message.contains("Not renamed"), vw._rename_message)
	_ok("... and nothing changed", _disk("Brindle.md") == before["Brindle.md"] and _disk("L1.md") == before["L1.md"])

	# == 2. The plan states the count, an outside change refuses the whole plan =
	var ro := _root + "/L3.md"
	FileAccess.set_read_only_attribute(ro, true)
	_ro_paths.append(ro)
	await _press(_named("NoteRename"))
	pd = await _wait_dlg("Rename", true)
	await _type_name_and_enter("Quillon")
	var cd := await _wait_dlg("Rename", false)
	_ok("a confirmation appears with the plan", cd != null)
	var said := _texts_under(cd) if cd != null else ""
	_ok("... naming the note's new path", said.contains("Brindle.md will become Quillon.md"), said)
	_ok("... saying 6 links in 3 other notes", said.contains("6 links in 3 other notes"), said)
	_ok("... and 1 link in the note itself", said.contains("1 in this note itself"), said)
	_ok("... listing each note with its count", said.contains("L1.md (4)") and said.contains("L2.md (1)") and said.contains("L3.md (1)"), said)
	_ok("... saying the one KnowledgeLink follows", said.contains("1 link from the map follow"), said)
	var ro_at := said.find("READ-ONLY")
	var ro_sentence := said.substr(ro_at, said.find(". The note still moves") - ro_at) if ro_at >= 0 else ""
	_ok("... warning that exactly L3.md is read-only BEFORE anything moves", ro_at >= 0 and ro_sentence.contains("L3.md") and not ro_sentence.contains("L1.md") and not ro_sentence.contains("L2.md"), ro_sentence)
	_ok("... and that nothing has moved yet", FileAccess.file_exists(_root + "/Brindle.md") and not FileAccess.file_exists(_root + "/Quillon.md"))
	## An outside change to a note the plan reads, made while the confirmation is open.
	_write(_root + "/L2.md", orig_l2 + "OUTSIDE EDIT\n")
	if cd != null:
		cd.get_ok_button().pressed.emit()
	await _frames(6)
	_ok("an outside change while deciding refuses the whole rename, in words", vw._rename_message.contains("nothing was written") or vw._rename_message.contains("Nothing was moved"), vw._rename_message)
	_ok("... Brindle.md is still there and Quillon.md is not", FileAccess.file_exists(_root + "/Brindle.md") and not FileAccess.file_exists(_root + "/Quillon.md"))
	_ok("... no note was rewritten (L1 and L3 as they were, L2 as the outside edit left it)", _disk("L1.md") == orig_l1 and _disk("L3.md") == orig_l3 and _disk("L2.md") == orig_l2 + "OUTSIDE EDIT\n")
	cd = await _wait_dlg("Rename", false)
	_ok("the plan is offered again, with the outside edit now included", cd != null)

	# == 3. Confirm: the move, the rewrites, the read-only report ================
	var first_opened: String = vw._hist.get("Brindle.md", [""])[0]
	var self_text_before_rename: String = te.text
	if cd != null:
		cd.get_ok_button().pressed.emit()
	var rd := await _wait_dlg("Undo the rename", false, 6000)
	await _frames(4)
	_ok("the note moved: Quillon.md exists and Brindle.md does not", FileAccess.file_exists(_root + "/Quillon.md") and not FileAccess.file_exists(_root + "/Brindle.md"))
	_ok("L1: every form rewritten (wiki, alias, heading link, Markdown link), the rest byte-identical", _disk("L1.md") == "# L1\n\none [[Quillon]] two\na [[Quillon|the grey town]] b\nc [[Quillon#Trade]] d\ne [md](Quillon.md) f\n", _disk("L1.md"))
	_ok("L2: the heading link and alias kept, code span untouched, the OUTSIDE edit kept", _disk("L2.md") == "# L2\n\nx [[Quillon#Trade|shown]] y\n`[[Brindle]]` stays in code\nOUTSIDE EDIT\n", _disk("L2.md"))
	_ok("L3 (read-only): NOT rewritten, byte-identical", _disk("L3.md") == orig_l3, _disk("L3.md"))
	_ok("Other.md and Taken.md untouched", _disk("Other.md") == orig_other and _disk("Taken.md") == before["Taken.md"])
	var want_self := (before["Brindle.md"] as String).replace("See [[Brindle]]", "See [[Quillon]]")
	_ok("the renamed note's own self-link followed it, its alias and heading link kept", _disk("Quillon.md") == want_self, _disk("Quillon.md"))
	_ok("the report names the note that was not rewritten and why", rd != null and vw._rename_message.contains("L3.md") and vw._rename_message.contains("NOT rewritten"), vw._rename_message)
	_ok("... and says how many were rewritten", vw._rename_message.contains("6 links in 3 notes"), vw._rename_message)
	_ok("the window's record lists exactly L3.md as failed", (vw._rename_last.get("failed", []) as Array).size() == 1 and String((vw._rename_last["failed"][0] as Dictionary).get("path", "")) == "L3.md", str(vw._rename_last))
	if rd != null:
		rd.get_cancel_button().pressed.emit()
		await _frames(6)
	_ok("one note focus: the window holds Quillon.md and the editor was built for it", vw._browse_path == "Quillon.md" and vw._editor_path == vw._browse_path and vw._pick_file == "Quillon.md", "%s %s" % [vw._browse_path, vw._editor_path])
	_grab()
	_ok("the editor shows the note as it now is on disk, clean (nothing to write)", te != null and te.text == want_self and not vw._is_dirty(), "" if te == null else te.text)
	_ok("the session undo followed the rename (history keyed to the new path, not the old)", vw._hist.has("Quillon.md") and not vw._hist.has("Brindle.md"))
	_ok("the KnowledgeLink follows: the settlement's note is Quillon.md", _link_path(int(s0.get("tid", 0))) == "Quillon.md", _link_path(int(s0.get("tid", 0))))
	_ok("... and the note still resolves to its settlement", not vw._entity_of_note("Quillon.md").is_empty())
	_ok("the backlink index sees L1 and L2 pointing at Quillon.md", _backlink_sources("Quillon.md").has("L1.md") and _backlink_sources("Quillon.md").has("L2.md"), str(_backlink_sources("Quillon.md")))
	_ok("... and L3, whose link was not rewritten, no longer points at it", not _backlink_sources("Quillon.md").has("L3.md"))
	await _tab("props")
	_ok("the Properties tab now offers the undo", _named("NoteRenameUndo") != null)
	await _shot("renamed")

	# == 4. The Note tab follows the new path ===================================
	vw._write_now()
	vw.hide()
	await _frames(3)
	var host := Control.new()
	host.custom_minimum_size = Vector2(400, 300)
	app.add_child(host)
	var dr: Dictionary = vw.dock_note("Quillon.md", host)
	_ok("the Note tab docks the renamed note", bool(dr.get("ok", false)) and vw._browse_path == "Quillon.md" and vw._editor_path == vw._browse_path and vw.is_docked(), str(dr))
	_ok("... showing the text on disk", vw._browse_edit != null and vw._browse_edit.text == _disk("Quillon.md"))
	vw.release_dock()
	host.queue_free()
	await _frames(3)
	app.open_vault_browse()
	await _frames(4)
	vw = app.vault_window
	vw._enter_note("Quillon.md")
	await _frames(6)
	_grab()
	if _phone:
		vw._set_side(true)
		await _frames(3)
		_grab()
	await _tab("props")

	# == 5. Undo ================================================================
	var ub := _named("NoteRenameUndo")
	_ok("the undo is on the Properties tab", ub != null)
	var pre_undo_quillon := _disk("Quillon.md")
	if ub != null:
		await _press(ub)
	var ud := await _wait_dlg("Undo rename", false)
	_ok("undo asks first, with the reverse plan", ud != null and _texts_under(ud).contains("Quillon.md will become Brindle.md"), "" if ud == null else _texts_under(ud))
	_ok("... and has written nothing yet", FileAccess.file_exists(_root + "/Quillon.md") and not FileAccess.file_exists(_root + "/Brindle.md"))
	if ud != null:
		ud.get_ok_button().pressed.emit()
	await _until(func(): return FileAccess.file_exists(_root + "/Brindle.md"), 5000)
	await _frames(6)
	_ok("undo moves the note back", FileAccess.file_exists(_root + "/Brindle.md") and not FileAccess.file_exists(_root + "/Quillon.md"))
	_ok("... and every file is byte-identical to before the rename (L1, L3, Other)", _disk("L1.md") == orig_l1 and _disk("L3.md") == orig_l3 and _disk("Other.md") == orig_other, _disk("L1.md"))
	_ok("... L2 is as it was with the outside edit kept", _disk("L2.md") == orig_l2 + "OUTSIDE EDIT\n", _disk("L2.md"))
	_ok("... and the note itself as it was before the rename", _disk("Brindle.md") == before["Brindle.md"], _disk("Brindle.md"))
	_ok("the KnowledgeLink is back on Brindle.md", _link_path(int(s0.get("tid", 0))) == "Brindle.md", _link_path(int(s0.get("tid", 0))))
	_ok("the window holds Brindle.md again, one note focus", vw._browse_path == "Brindle.md" and vw._editor_path == vw._browse_path and vw._hist.has("Brindle.md") and not vw._hist.has("Quillon.md"))
	_grab()
	await _to_note()
	te.grab_focus()
	# Ctrl+Z walks back across both renames to the text as first opened.
	var guard := 0
	while te.text != first_opened and guard < 8:
		await _key(KEY_Z, true)
		guard += 1
	_ok("Ctrl+Z walks back across the rename and its undo to the text as first opened", te.text == first_opened, "%d presses: %s" % [guard, te.text])
	await _until(func(): return _disk("Brindle.md") == first_opened, 4000)
	_ok("... and the autosave wrote that to Brindle.md through the guard (the new path)", _disk("Brindle.md") == first_opened, _disk("Brindle.md"))
	_ok("no stray Quillon.md after the walk back", not FileAccess.file_exists(_root + "/Quillon.md"))

	# == 6. Create from an entity link binds it ================================
	vw._write_now()
	var sname := String(s1.get("name", ""))
	await _open("Hub.md")
	_ok("Hub.md is open", vw._browse_path == "Hub.md")
	var info: Dictionary = vw._link_kind("Hub.md", sname)
	_ok("the link names a world entity with no note", String(info.get("kind", "")) == "entity", str(info))
	var want_rel: String = _bridge.vault_suggested_path(String(info.get("ent_kind", "settlement")), sname)
	_ok("CONTROL: the suggested note does not exist yet", not FileAccess.file_exists(_root + "/" + want_rel), want_rel)
	_ok("CONTROL: the entity has no KnowledgeLink yet", _bridge.vault_links_for("settlement", int(s1.get("tid", 0))).size() == 0)
	vw._follow_link("Hub.md", sname, vw._editor)
	await _frames(4)
	var pm: PopupMenu = vw.find_child("LinkMenu", true, false) as PopupMenu
	_ok("following it OFFERS to create (a menu), and creates nothing by itself", pm != null and not FileAccess.file_exists(_root + "/" + want_rel))
	var blank_id := -1
	var rows: Array = []
	if pm != null:
		for i in pm.item_count:
			rows.append(pm.get_item_text(i))
			if pm.get_item_text(i).contains("as a blank note"):
				blank_id = pm.get_item_id(i)
	_ok("the menu offers a blank note (and a template row per template)", blank_id >= 0, str(rows))
	if pm != null and blank_id >= 0:
		pm.id_pressed.emit(blank_id)
	await _frames(8)
	_ok("the note was created", FileAccess.file_exists(_root + "/" + want_rel), want_rel)
	_ok("... holding a title line", _disk(want_rel) == "# %s\n" % sname, _disk(want_rel))
	var ls: Array = _bridge.vault_links_for("settlement", int(s1.get("tid", 0)))
	_ok("a KnowledgeLink now ties the entity to the new note", ls.size() == 1 and String((ls[0] as Dictionary).get("path", "")) == want_rel, str(ls))
	_ok("... and the note resolves to its settlement", not vw._entity_of_note(want_rel).is_empty())
	_ok("... and it opened in Write", vw._browse_path == want_rel)

	await _shot("final")
	_finish()


func _link_path(tid: int) -> String:
	var ls: Array = _bridge.vault_links_for("settlement", tid)
	return String((ls[0] as Dictionary).get("path", "")) if ls.size() == 1 else "<%d links>" % ls.size()


func _backlink_sources(rel: String) -> Array:
	_bridge.vault_refresh_backlinks(2000)
	var out: Array = []
	for b in _bridge.vault_file_backlinks(rel):
		out.append(String((b as Dictionary).get("rel", "")))
	return out
