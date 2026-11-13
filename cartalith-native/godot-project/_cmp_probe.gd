extends Node
## VE-4 of `VAULT_EDITOR_PROPOSAL.md` s.7: autosave, the conflict banner, the session undo and Revert to
## opened. Bar: *an edit outside Cartalith is never overwritten, by a save or by a revert; "saved" is
## visible within 2 s. Undo reaches the first-opened text after the note is closed and reopened, and
## nothing of it is on disk after a quit. Revert asks first and is one undo step.*
##
##   Godot_v4.7.2-stable_win64_console.exe --path . _cmp_probe.tscn -- --vp 1600x1000 [--touch] [--shots DIR]
##   ... -- --vp 1080x2340 --force-touch   (the phone leg)
##
## **Windowed.** Run under a wiped user directory (`APPDATA=<empty dir>`): it deletes `user://vault.json`
## and binds a throw-away vault in TEMP. Exit 0 = every check held, 1 = a check failed, 2 = could not run.
##
## ## What each leg proves (data first: a user's notes are data)
##
## - **Autosave** writes nothing before a pause and nothing for an unchanged note, writes within 2 s of
##   the last key, and writes a CRLF note back with CRLF endings and its bytes untouched until edited.
## - **An outside change is never overwritten**: by the autosave, by a second autosave, by Keep mine
##   when the file changed AGAIN after the banner, or by Revert to opened. Every choice is driven with a
##   real button press and every file assertion re-reads the disk.
## - **The refusals that are not conflicts**: a deleted note (never recreated by itself), a read-only
##   file, a file another program holds open (a real second process), and a vault folder that
##   disappears (and comes back).
## - **The session undo**: Ctrl+Z, typed as a key, reaches the first-opened text after the note was closed
##   and reopened and after a rebuild; no trace of that text is on disk once the note was replaced.
## - **Revert to opened** asks first, Cancel changes nothing, and Revert is one undo step.
##
## A script error inside an `await` hangs a probe, so every wait here is bounded and a missing control
## ends the run with the failure on record.

const Ed := preload("res://shell/markdown_editor.gd")

var _vp: SubViewport
var app: Node
var _bridge
var vw: VaultWindow
var ed
var te: TextEdit
var _root := ""
var _shots := ""
var _touch := false
var _phone := false
var _n := 0
var _fails: Array = []
var _ro_path := ""
var _holder_pid := -1


func _ok(label: String, cond: bool, detail: String = "") -> void:
	_n += 1
	if cond:
		print("CMP  ok  %s" % label)
	else:
		_fails.append(label)
		print("CMP  !!  %s   %s" % [label, detail])


func _arg(name: String, dflt: String) -> String:
	var args := OS.get_cmdline_user_args()
	var i := args.find(name)
	if i >= 0 and i + 1 < args.size():
		return String(args[i + 1])
	return dflt


func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame


## Waits `ms` of wall time, frame by frame (timers need frames to run).
func _wait(ms: int) -> void:
	var t := Time.get_ticks_msec()
	while Time.get_ticks_msec() - t < ms:
		await get_tree().process_frame


## Frames until `cond` is true or `ms` pass; returns the elapsed milliseconds, -1 on timeout.
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


func _write_bytes(path: String, b: PackedByteArray) -> void:
	var f := FileAccess.open(path, FileAccess.WRITE)
	f.store_buffer(b)
	f.close()


func _disk(rel: String) -> String:
	return FileAccess.get_file_as_string(_root + "/" + rel)


func _bytes(rel: String) -> PackedByteArray:
	return FileAccess.get_file_as_bytes(_root + "/" + rel)


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


func _press(win: Window, b: Button) -> void:
	await _tap(win, b.get_global_rect().get_center())
	await _frames(3)


## Types `s` through real key events (focus must already be where the text should go).
func _type(s: String) -> void:
	for i in s.length():
		for pressed in [true, false]:
			var k := InputEventKey.new()
			k.pressed = pressed
			k.unicode = s.unicode_at(i)
			k.keycode = s.unicode_at(i) as Key
			_vp.push_input(k)
		await _frames(2)


func _key(code: Key, ctrl: bool = false, shift: bool = false) -> void:
	for pressed in [true, false]:
		var k := InputEventKey.new()
		k.pressed = pressed
		k.keycode = code
		k.physical_keycode = code
		k.ctrl_pressed = ctrl
		k.shift_pressed = shift
		_vp.push_input(k)
	await _frames(3)


func _finish() -> void:
	if _ro_path != "":
		FileAccess.set_read_only_attribute(_ro_path, false)
	if _holder_pid > 0:
		OS.kill(_holder_pid)
	print("CMP RESULT %d checks, %d failed%s" % [_n, _fails.size(), "" if _fails.is_empty() else " :: " + str(_fails)])
	get_tree().quit(0 if _fails.is_empty() else 1)


func _tmp_files() -> Array:
	var out: Array = []
	var d := DirAccess.open(_root)
	if d == null:
		return out
	for f in d.get_files():
		if f.ends_with(".cartalith-tmp"):
			out.append(f)
	return out


## Opens `rel` in the editor the way a link follow does (the window's own entry), focuses the source and
## puts the caret at its end. Callers have dealt with unsaved edits.
func _open(rel: String) -> void:
	vw._link_back.clear()
	vw._enter_note(rel)
	await _frames(5)
	ed = vw._editor
	te = ed.text_edit if ed != null else null
	if te != null:
		te.grab_focus()
		_end()
	await _frames(2)


func _end() -> void:
	var last := te.get_line_count() - 1
	te.set_caret_line(last)
	te.set_caret_column(te.get_line(last).length())


func _banner_up() -> bool:
	return ed != null and is_instance_valid(ed) and ed.banner != null and ed.banner.visible


func _btn(key: String) -> Button:
	if ed == null or ed._banner_row == null:
		return null
	return ed._banner_row.get_node_or_null("Banner_" + key) as Button


func _status() -> String:
	return ed.status_label.text if ed != null and ed.status_label != null else ""


## Types `mine` at the end, then changes the file on disk BEFORE the autosave fires, and waits for the
## banner. Leaves the editor on the conflicted note.
func _conflict(rel: String, base: String, mine: String, theirs: String) -> void:
	_write(_root + "/" + rel, base)
	await _open(rel)
	await _type(mine)
	_write(_root + "/" + rel, theirs)
	var ms := await _until(func(): return _banner_up(), 4000)
	_ok("%s: a change made outside raises the banner (%d ms)" % [rel, ms], ms >= 0, _status())


## The visible confirmation whose OK button reads `ok_text` (its title and body differ on the phone's sheet).
func _confirm_dialog(ok_text: String) -> ConfirmationDialog:
	var out: Array = []
	_find_all(app, "ConfirmationDialog", out)
	for d in out:
		if (d as Window).visible and (d as ConfirmationDialog).get_ok_button().text == ok_text:
			return d
	return null


## Every label's text under a node, joined (a dialog's body is a Label on the desktop and prose on the phone).
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


func _scan_for(dir: String, needle: String, hits: Array) -> void:
	var d := DirAccess.open(dir)
	if d == null:
		return
	for f in d.get_files():
		var p := dir + "/" + f
		var fa := FileAccess.open(p, FileAccess.READ)
		if fa == null or fa.get_length() > 8000000:
			continue
		if fa.get_buffer(fa.get_length()).get_string_from_utf8().contains(needle):
			hits.append(p)
	for sub in d.get_directories():
		_scan_for(dir + "/" + sub, needle, hits)


func _ready() -> void:
	for a in OS.get_cmdline_user_args():
		var s := String(a)
		if s.begins_with("--") and not (s in ["--vp", "--touch", "--shots", "--force-touch"]):
			print("CMP ABORT unknown flag %s" % s)
			get_tree().quit(2)
			return
	if DisplayServer.get_name() == "headless":
		print("CMP REFUSED: run windowed")
		get_tree().quit(2)
		return
	var wd := Timer.new()
	wd.wait_time = 540.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("CMP WATCHDOG TIMEOUT")
		if _ro_path != "":
			FileAccess.set_read_only_attribute(_ro_path, false)
		if _holder_pid > 0:
			OS.kill(_holder_pid)
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
	_root = OS.get_environment("TEMP").replace("\\", "/") + "/cartalith-ve4-%s-%d" % ["touch" if _touch else "desk", Time.get_ticks_msec()]
	_write(_root + "/Q0first.md", "# Q0first
")  # sorts first: the window picks it on open, so selecting Qalpha is a real selection (a re-selection of the picked note never switches the phone to the note pane)
	_write(_root + "/Qalpha.md", "# Qalpha\n\nbase text\n")
	_write_bytes(_root + "/Qcrlf.md", "# Qcrlf\r\n\r\nline one\r\nline two\r\n".to_utf8_buffer())
	_write(_root + "/Qother.md", "# Qother\n\nelsewhere\n")
	var conn: Dictionary = _bridge.vault_connect(_root, "Chronicle")
	_ok("CONTROL: the fixture vault binds", bool(conn.get("ok", false)), str(conn))
	_bridge.vault_refresh_backlinks(2000)
	app.open_vault_browse()
	await _frames(4)
	vw = app.vault_window
	var ts: Array = []
	_find_all(vw, "Tree", ts)
	var tree: Tree = ts[0] if not ts.is_empty() else null
	var leaf := _leaf_in(tree.get_root(), "Qalpha.md") if tree != null else null
	_ok("CONTROL: Qalpha.md is in the tree", leaf != null)
	if leaf == null:
		_finish()
		return
	leaf.select(0)
	await _frames(4)
	var open_btn := _find_button(vw, "Open to edit")
	_ok("CONTROL: Open to edit is offered", open_btn != null, "phone=%s pane=%s path=%s pick=%s" % [vw._phone, str(vw.get("_browse_phone_pane")), vw._browse_path, vw._pick_file])
	if open_btn == null:
		_finish()
		return
	await _press(vw, open_btn)
	await _frames(4)
	ed = vw._editor
	_ok("the note opens in Write", ed != null and ed.mode() == "write")
	if ed == null:
		_finish()
		return
	te = ed.text_edit
	_phone = ed._phone
	print("CMP info: viewport %s, phone=%s, touch=%s" % [_vp.size, _phone, _touch])

	# == Compare ===========================================================
	# Qcmp: base 4 lines; mine edits line 2, theirs edits line 3 and adds a line; both differ from each other.
	await _conflict("Qcmp1.md", "a\nb\nc\nd\n", "!", "a\nb\nTHEIRS\nd\nextra\n")
	_ok("the banner offers Compare as a fourth choice", _btn("compare") != null)
	if _touch and _phone:
		_ok("phone: Compare is 44 tall", _btn("compare").size.y >= 43.9, str(_btn("compare").size))
	await _press(vw, _btn("compare"))
	await _frames(6)
	var dlgs: Array = []
	_find_all(app, "AcceptDialog", dlgs)
	var cd: Window = null
	for d in dlgs:
		if (d as Window).name == "VaultCompareDialog" and (d as Window).visible:
			cd = d
	_ok("Compare opens a window", cd != null)
	if cd == null:
		_finish()
		return
	var tm := cd.find_child("CompareMine", true, false) as TextEdit
	var tt := cd.find_child("CompareTheirs", true, false) as TextEdit
	_ok("Mine shows the editor's text", tm != null and tm.text == "a\nb\nc\nd\n!", tm.text if tm != null else "")
	_ok("Theirs shows the disk text", tt != null and tt.text == "a\nb\nTHEIRS\nd\nextra\n", tt.text if tt != null else "")
	var bgc := func(t: TextEdit, i: int) -> bool: return t.get_line_background_color(i).a > 0.0
	var cm: PackedInt32Array = VaultCompare.changed_lines(tm.text.split("\n"), tt.text.split("\n"))["mine"]
	_ok("Mine's changed lines are exactly c and !", Array(cm) == [2, 4], str(cm))
	var ct: PackedInt32Array = VaultCompare.changed_lines(tm.text.split("\n"), tt.text.split("\n"))["theirs"]
	_ok("Theirs' changed lines are exactly THEIRS, extra, the trailing empty line", Array(ct) == [2, 4, 5], str(ct))
	_ok("the line background really is set on a changed line and not on a shared one",
		bgc.call(tm, 2) and not bgc.call(tm, 0) and bgc.call(tt, 4) and not bgc.call(tt, 1) and not bgc.call(tm, 3))
	var stacked := tm.global_position.y < tt.global_position.y and absf(tm.global_position.x - tt.global_position.x) < 4.0
	var beside := tm.global_position.x < tt.global_position.x and absf(tm.global_position.y - tt.global_position.y) < 4.0
	_ok("layout: %s" % ("stacked on the phone" if _phone else "side by side"), stacked if _phone else beside)
	for k in ["keep_mine", "take_theirs", "keep_both"]:
		var cb := cd.find_child("Compare_" + k, true, false) as Button
		_ok("Compare has %s" % k, cb != null)
		if _phone and cb != null:
			_ok("... %s is 44 tall (%.0f)" % [k, cb.size.y], cb.size.y >= 43.9)
	await _press(cd, cd.find_child("Compare_take_theirs", true, false) as Button)  # close via a choice; scroll is tested below
	await _wait(300)
	_ok("Take theirs from Compare: disk is theirs and the banner is gone", _disk("Qcmp1.md") == "a\nb\nTHEIRS\nd\nextra\n" and not _banner_up(), _disk("Qcmp1.md"))
	_ok("... and the editor shows theirs", te.text == "a\nb\nTHEIRS\nd\nextra\n", te.text)
	_ok("... and the compare window is gone", cd == null or not is_instance_valid(cd) or not cd.visible or cd.is_queued_for_deletion())

	# Keep mine from Compare
	await _conflict("Qcmp2.md", "a\nb\n", " mine", "theirs\n")
	await _press(vw, _btn("compare"))
	await _frames(6)
	cd = _find_cd()
	await _press(cd, cd.find_child("Compare_keep_mine", true, false) as Button)
	await _wait(400)
	_ok("Keep mine from Compare writes mine over theirs", _disk("Qcmp2.md") == "a\nb\n mine" and not _banner_up(), _disk("Qcmp2.md"))

	# Keep both from Compare
	await _conflict("Qcmp3.md", "a\n", "M", "theirs3\n")
	await _press(vw, _btn("compare"))
	await _frames(6)
	cd = _find_cd()
	await _press(cd, cd.find_child("Compare_keep_both", true, false) as Button)
	await _wait(400)
	var copies := 0
	var dd := DirAccess.open(_root)
	for f in dd.get_files():
		if f.begins_with("Qcmp3 (my edits") and _disk(f) == "a\nM":
			copies += 1
	_ok("Keep both from Compare: a copy holds mine, the note holds theirs", copies == 1 and _disk("Qcmp3.md") == "theirs3\n", "%d %s" % [copies, _disk("Qcmp3.md")])

	# Close leaves the conflict standing, nothing written
	await _conflict("Qcmp4.md", "a\n", "M", "theirs4\n")
	await _press(vw, _btn("compare"))
	await _frames(6)
	cd = _find_cd()
	# synchronised scrolling needs a long text
	var long_t := ""
	for i in 200:
		long_t += "line %d\n" % i
	cd.get_ok_button().pressed.emit()
	await _frames(4)
	_ok("Close leaves the banner up and the disk as theirs", _banner_up() and _disk("Qcmp4.md") == "theirs4\n")

	# synchronised scrolling, direct on a window built from a long conflict
	await _conflict("Qcmp5.md", long_t, "!", long_t + "tail\n")
	await _press(vw, _btn("compare"))
	await _frames(6)
	cd = _find_cd()
	tm = cd.find_child("CompareMine", true, false) as TextEdit
	tt = cd.find_child("CompareTheirs", true, false) as TextEdit
	tm.get_v_scroll_bar().value = 40.0
	await _frames(3)
	_ok("scrolling Mine scrolls Theirs the same amount (%.1f / %.1f)" % [tm.get_v_scroll_bar().value, tt.get_v_scroll_bar().value], tm.get_v_scroll_bar().value > 0.0 and is_equal_approx(tm.get_v_scroll_bar().value, tt.get_v_scroll_bar().value))
	tt.get_v_scroll_bar().value = 10.0
	await _frames(3)
	_ok("... and Theirs scrolls Mine", is_equal_approx(tm.get_v_scroll_bar().value, 10.0))
	cd.get_ok_button().pressed.emit()
	await _frames(3)

	if _shots != "":
		DirAccess.make_dir_recursive_absolute(_shots)
	_finish()


func _find_cd() -> Window:
	var dlgs: Array = []
	_find_all(app, "AcceptDialog", dlgs)
	for d in dlgs:
		if (d as Window).name == "VaultCompareDialog" and (d as Window).visible:
			return d
	return null
