extends Node
## VE-4 of `VAULT_EDITOR_PROPOSAL.md` s.7: autosave, the conflict banner, the session undo and Revert to
## opened. Bar: *an edit outside Cartalith is never overwritten, by a save or by a revert; "saved" is
## visible within 2 s. Undo reaches the first-opened text after the note is closed and reopened, and
## nothing of it is on disk after a quit. Revert asks first and is one undo step.*
##
##   Godot_v4.7.2-stable_win64_console.exe --path . _ve4_probe.tscn -- --vp 1600x1000 [--touch] [--shots DIR]
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
		print("VE4  ok  %s" % label)
	else:
		_fails.append(label)
		print("VE4  !!  %s   %s" % [label, detail])


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
	print("VE4 RESULT %d checks, %d failed%s" % [_n, _fails.size(), "" if _fails.is_empty() else " :: " + str(_fails)])
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
			print("VE4 ABORT unknown flag %s" % s)
			get_tree().quit(2)
			return
	if DisplayServer.get_name() == "headless":
		print("VE4 REFUSED: run windowed")
		get_tree().quit(2)
		return
	var wd := Timer.new()
	wd.wait_time = 540.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("VE4 WATCHDOG TIMEOUT")
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
	print("VE4 info: viewport %s, phone=%s, touch=%s" % [_vp.size, _phone, _touch])

	# == 1. Autosave: nothing early, nothing for nothing, saved within 2 s ======
	var t0 := _disk("Qalpha.md")
	var w0: int = vw._writes_attempted
	te.grab_focus()
	_end()
	await _wait(1400)
	_ok("opening a note and leaving it alone writes nothing", vw._writes_attempted == w0 and _disk("Qalpha.md") == t0)
	await _type(" more")
	_ok("a key puts 'editing…' in the status at once", _status().begins_with("editing"), _status())
	var t_key := Time.get_ticks_msec()
	await _wait(500)
	_ok("0.5 s after the last key nothing is written (autosave waits for a pause)", _disk("Qalpha.md") == t0 and vw._writes_attempted == w0)
	var ms := await _until(func(): return _disk("Qalpha.md") != t0, 3000)
	var since_key := Time.get_ticks_msec() - t_key
	_ok("the note is on disk within 2 s of the last key (%d ms)" % since_key, ms >= 0 and since_key <= 2000)
	_ok("... and is exactly what was typed", _disk("Qalpha.md") == t0 + " more", _disk("Qalpha.md"))
	_ok("... and 'saved · time' is on screen (%s)" % _status(), _status().begins_with("saved") and _status().contains(":"))
	_ok("no temp file is left beside the note", _tmp_files().is_empty(), str(_tmp_files()))
	var saved_text := _disk("Qalpha.md")
	var w1: int = vw._writes_attempted
	await _type("x")
	await _key(KEY_BACKSPACE)
	await _wait(1600)
	_ok("typing then deleting back to the saved text writes nothing", vw._writes_attempted == w1 and _disk("Qalpha.md") == saved_text, "%d vs %d" % [vw._writes_attempted, w1])

	# pressing Save on a note with nothing unsaved is a no-op too (the same path the autosave and leaving use)
	await _press(vw, ed.save_button)
	await _wait(200)
	_ok("Save on a clean note writes nothing", vw._writes_attempted == w1 and _disk("Qalpha.md") == saved_text, "%d vs %d" % [vw._writes_attempted, w1])

	# == 2. CRLF: bytes untouched until edited, then every ending stays CRLF ====
	var crlf0 := _bytes("Qcrlf.md")
	await _open("Qcrlf.md")
	_ok("CONTROL: the CRLF note opens clean", not vw._is_dirty(), JSON.stringify([te.text, vw._browse_saved, vw._browse_text]))
	await _wait(1500)
	_ok("a CRLF note opened and left alone is not rewritten (bytes identical)", _bytes("Qcrlf.md") == crlf0)
	await _type("Z")
	await _until(func(): return _bytes("Qcrlf.md") != crlf0, 3000)
	var cb := _bytes("Qcrlf.md")
	var lf := 0
	var cr := 0
	for i in cb.size():
		if cb[i] == 10:
			lf += 1
		elif cb[i] == 13:
			cr += 1
	_ok("an edited CRLF note keeps CRLF on every line (%d LF, %d CR)" % [lf, cr], lf == 4 and cr == 4 and cb.get_string_from_utf8() == "# Qcrlf\r\n\r\nline one\r\nline two\r\nZ")

	# == 3. A change made outside is never overwritten ==========================
	await _conflict("Qconf1.md", "one\n", " mine", "theirs\n")
	var wc: int = vw._writes_attempted
	_ok("the autosave did not overwrite theirs", _disk("Qconf1.md") == "theirs\n", _disk("Qconf1.md"))
	_ok("the banner names the three choices", _btn("keep_mine") != null and _btn("take_theirs") != null and _btn("keep_both") != null)
	_ok("the editor still holds my text", te.text == "one\n mine")
	_ok("the status says it is not saved", _status().begins_with("Not saved"), _status())
	if _touch:
		for k in ["keep_mine", "take_theirs", "keep_both"]:
			var b := _btn(k)
			## 44 is the phone shell's floor (VE3's phone leg); the desktop shell with touch emulation
			## keeps its 26 px buttons by design, so only the phone leg is held to it.
			_ok("%s: the %s button is %.0f tall" % ["phone" if _phone else "touch", k, b.size.y if b != null else 0.0], b != null and (b.size.y >= 43.9 or not _phone))
	await _type("!")
	await _wait(1700)
	_ok("further typing does not retry the write while the conflict stands", vw._writes_attempted == wc and _disk("Qconf1.md") == "theirs\n", "%d vs %d" % [vw._writes_attempted, wc])
	# the window rebuilds (a drag across the fold) with the banner and the text intact
	vw._rebuild()
	await _frames(5)
	ed = vw._editor
	te = ed.text_edit
	_ok("a rebuild brings the banner back", _banner_up())
	_ok("... and the unsaved text", te.text == "one\n mine!", te.text)
	# Keep mine, after the file changed AGAIN: refuses, banner returns, the newer outside version survives
	_write(_root + "/Qconf1.md", "theirs again\n")
	await _press(vw, _btn("keep_mine"))
	await _wait(300)
	_ok("Keep mine after a SECOND outside change does not overwrite it", _disk("Qconf1.md") == "theirs again\n", _disk("Qconf1.md"))
	_ok("... and the banner is up again", _banner_up())
	# Keep mine on the version the banner now describes
	await _press(vw, _btn("keep_mine"))
	await _wait(300)
	_ok("Keep mine then replaces it with mine", _disk("Qconf1.md") == "one\n mine!", _disk("Qconf1.md"))
	_ok("... the banner is gone and the status says saved", not _banner_up() and _status().begins_with("saved"), _status())

	# Take theirs: the editor shows theirs, disk untouched, Ctrl+Z brings mine back
	await _conflict("Qconf2.md", "one\n", " mine", "theirs two\n")
	await _press(vw, _btn("take_theirs"))
	await _wait(300)
	_ok("Take theirs shows theirs in the editor", te.text == "theirs two\n", te.text)
	_ok("... leaves the disk as theirs", _disk("Qconf2.md") == "theirs two\n")
	_ok("... clears the banner", not _banner_up())
	te.grab_focus()
	await _key(KEY_Z, true)
	_ok("Ctrl+Z after Take theirs brings my text back", te.text == "one\n mine", te.text)
	await _key(KEY_Z, true, true)
	_ok("... and Ctrl+Shift+Z redoes theirs", te.text == "theirs two\n", te.text)

	# Keep both: my text beside the note, theirs in the editor
	await _conflict("Qconf3.md", "one\n", " mine", "theirs three\n")
	await _press(vw, _btn("keep_both"))
	await _wait(300)
	var copies: Array = []
	var dd := DirAccess.open(_root)
	for f in dd.get_files():
		if f.begins_with("Qconf3 (my edits"):
			copies.append(f)
	_ok("Keep both writes one copy beside the note", copies.size() == 1, str(copies))
	_ok("... holding my text", copies.size() == 1 and _disk(String(copies[0])) == "one\n mine")
	_ok("... and theirs stays on the note and in the editor", _disk("Qconf3.md") == "theirs three\n" and te.text == "theirs three\n")

	# == 4. A note deleted outside is never recreated by itself ================
	_write(_root + "/Qgone.md", "gone base\n")
	await _open("Qgone.md")
	await _type(" mine")
	DirAccess.remove_absolute(_root + "/Qgone.md")
	ms = await _until(func(): return _banner_up(), 4000)
	_ok("a deleted note raises a banner (%d ms)" % ms, ms >= 0, _status())
	await _wait(1500)
	_ok("the autosave did not recreate the deleted note", not FileAccess.file_exists(_root + "/Qgone.md"))
	_ok("the choices are Recreate and Close note", _btn("recreate") != null and _btn("discard") != null and _btn("keep_mine") == null)
	await _press(vw, _btn("recreate"))
	await _wait(300)
	_ok("Recreate writes my text as a new note", _disk("Qgone.md") == "gone base\n mine", _disk("Qgone.md"))
	_ok("... and the banner goes", not _banner_up())
	# a second delete, answered with Close note
	await _type("!")
	DirAccess.remove_absolute(_root + "/Qgone.md")
	await _until(func(): return _banner_up(), 4000)
	await _press(vw, _btn("discard"))
	await _wait(300)
	_ok("Close note closes the editor and writes nothing", vw._editor == null and not FileAccess.file_exists(_root + "/Qgone.md"))

	# == 5. A read-only note stays as it is ====================================
	_write(_root + "/Qro.md", "ro base\n")
	_ro_path = _root + "/Qro.md"
	await _open("Qro.md")
	FileAccess.set_read_only_attribute(_ro_path, true)
	await _type(" mine")
	ms = await _until(func(): return _banner_up(), 4000)
	_ok("a read-only note raises a banner (%d ms)" % ms, ms >= 0, _status())
	_ok("... the note was not replaced", _disk("Qro.md") == "ro base\n")
	_ok("... no temp file is left", _tmp_files().is_empty(), str(_tmp_files()))
	_ok("... the choice is Try again, and the text is still here", _btn("retry") != null and te.text == "ro base\n mine")
	FileAccess.set_read_only_attribute(_ro_path, false)
	_ro_path = ""
	await _press(vw, _btn("retry"))
	await _wait(300)
	_ok("Try again, once it is writable, saves", _disk("Qro.md") == "ro base\n mine" and not _banner_up(), _disk("Qro.md"))

	# == 6. A file another program holds open (a real second process) ==========
	if not _touch:
		_write(_root + "/Qlock.md", "lock base\n")
		var script := "$f=[System.IO.File]::Open('%s','Open','Read','Read'); Start-Sleep -Seconds 40; $f.Close()" % (_root + "/Qlock.md").replace("/", "\\")
		_holder_pid = OS.create_process("powershell.exe", ["-NoProfile", "-NonInteractive", "-Command", script])
		await _open("Qlock.md")
		await _wait(2500)
		await _type(" mine")
		ms = await _until(func(): return _banner_up(), 5000)
		_ok("a note another program holds open raises a banner (%d ms)" % ms, ms >= 0, _status())
		_ok("... the held note is unchanged and no temp file is left", _disk("Qlock.md") == "lock base\n" and _tmp_files().is_empty(), str(_tmp_files()))
		_ok("... and my text is still in the editor", te.text == "lock base\n mine")
		OS.kill(_holder_pid)
		_holder_pid = -1
		await _wait(1500)
		if _btn("retry") != null:
			await _press(vw, _btn("retry"))
			await _wait(300)
		_ok("once the other program lets go, Try again saves", _disk("Qlock.md") == "lock base\n mine", _disk("Qlock.md"))

	# == 7. The vault folder disappears, and comes back ========================
	_write(_root + "/Qvan.md", "van base\n")
	await _open("Qvan.md")
	var moved := _root + "_moved"
	DirAccess.rename_absolute(_root, moved)
	await _type(" mine")
	ms = await _until(func(): return _banner_up(), 4000)
	_ok("a vanished vault folder raises a banner (%d ms)" % ms, ms >= 0, _status())
	_ok("... the folder was not recreated by a write", not DirAccess.dir_exists_absolute(_root))
	_ok("... my text is still in the editor", te.text == "van base\n mine")
	DirAccess.rename_absolute(moved, _root)
	await _press(vw, _btn("retry"))
	await _wait(300)
	_ok("when the folder returns, Try again saves", _disk("Qvan.md") == "van base\n mine" and not _banner_up(), _disk("Qvan.md"))

	# == 8. Leaving a note, and closing the window, save first ==================
	await _open("Qalpha.md")
	await _type(" leave")
	vw._select_file("Qother.md")
	await _frames(5)
	_ok("opening another note saves the first one, with no prompt", _disk("Qalpha.md").ends_with(" more leave") and _confirm_dialog("Discard edits") == null, _disk("Qalpha.md"))
	await _open("Qother.md")
	await _type(" closing")
	vw.hide()
	await _frames(5)
	_ok("closing the window saves the open note, with no prompt", _disk("Qother.md").ends_with(" closing") and _confirm_dialog("Discard and close") == null, _disk("Qother.md"))
	app.open_vault_browse()
	await _frames(4)

	# == 9. The session undo ====================================================
	_write(_root + "/Qundo.md", "ORIGMARKER_77 first text\n")
	await _open("Qundo.md")
	te.grab_focus()
	await _key(KEY_A, true)
	await _type("fresh one")
	await _until(func(): return _disk("Qundo.md") == "fresh one", 3000)
	_ok("the note was replaced and saved", _disk("Qundo.md") == "fresh one", _disk("Qundo.md"))
	_end()
	await _type(" two")
	await _until(func(): return _disk("Qundo.md") == "fresh one two", 3000)
	vw._select_file("Qother.md")
	await _frames(5)
	await _open("Qundo.md")
	_ok("reopened: the saved text is shown", te.text == "fresh one two", te.text)
	te.grab_focus()
	var steps := 0
	for _i in 12:
		var before := te.text
		await _key(KEY_Z, true)
		if te.text == before:
			break
		steps += 1
	_ok("Ctrl+Z after a close and reopen reaches the first-opened text (%d steps)" % steps, te.text == "ORIGMARKER_77 first text\n" and steps >= 2, te.text)
	# and again after a rebuild, which also frees the TextEdit
	await _key(KEY_Z, true, true)
	var redone := te.text
	vw._rebuild()
	await _frames(5)
	ed = vw._editor
	te = ed.text_edit
	te.grab_focus()
	_ok("a rebuild keeps the text", te.text == redone)
	for _i in 12:
		var before2 := te.text
		await _key(KEY_Z, true)
		if te.text == before2:
			break
	_ok("Ctrl+Z after a rebuild still reaches the first-opened text", te.text == "ORIGMARKER_77 first text\n", te.text)
	# nothing of it is on disk: the note no longer holds the marker (autosave writes the undone text, so
	# put the replacement back and let it save, then scan every file)
	await _key(KEY_Z, true, true)
	await _key(KEY_Z, true, true)
	await _until(func(): return not _disk("Qundo.md").contains("ORIGMARKER_77"), 4000)
	await _wait(1500)
	var hits: Array = []
	_scan_for(_root, "ORIGMARKER_77", hits)
	_scan_for(OS.get_user_data_dir().replace("\\", "/"), "ORIGMARKER_77", hits)
	_ok("no file under the vault or the user directory holds the first-opened text (%s)" % str(hits), hits.is_empty(), str(hits))

	# == 10. Revert to opened ===================================================
	var rb: Button = ed.revert_button
	_ok("the header offers Revert to opened…, and Cancel is gone", rb != null and rb.text.to_lower().begins_with("revert to opened") and _find_button(ed, "Cancel") == null, "rb=%s cancel=%s" % [str(rb.text if rb != null else "<null>"), str(_find_button(ed, "Cancel"))])
	var edited_now := te.text
	var w_rev: int = vw._writes_attempted
	await _press(vw, rb)
	var dlg := _confirm_dialog("Revert")
	_ok("Revert asks first, naming what is lost", dlg != null and _texts_under(dlg).contains("Ctrl+Z"), "" if dlg == null else _texts_under(dlg))
	_ok("... and has written nothing yet", te.text == edited_now and vw._writes_attempted == w_rev)
	if dlg != null:
		dlg.get_cancel_button().pressed.emit()
		await _frames(4)
	_ok("Cancel changes nothing", te.text == edited_now and _disk("Qundo.md") == edited_now)
	await _press(vw, rb)
	dlg = _confirm_dialog("Revert")
	if dlg != null:
		dlg.get_ok_button().pressed.emit()
		await _frames(4)
	_ok("Revert puts back the first-opened text in the editor", te.text == "ORIGMARKER_77 first text\n", te.text)
	await _until(func(): return _disk("Qundo.md") == "ORIGMARKER_77 first text\n", 3000)
	_ok("... and on disk", _disk("Qundo.md") == "ORIGMARKER_77 first text\n", _disk("Qundo.md"))
	_ok("after Revert the keyboard is back in the text", te.has_focus())
	await _key(KEY_Z, true)
	_ok("ONE Ctrl+Z brings back everything the revert replaced", te.text == edited_now, te.text)
	await _until(func(): return _disk("Qundo.md") == edited_now, 3000)
	# the revert of a note changed outside Cartalith raises the banner and overwrites nothing
	_write(_root + "/Qundo.md", "outside change\n")
	await _press(vw, rb)
	dlg = _confirm_dialog("Revert")
	if dlg != null:
		dlg.get_ok_button().pressed.emit()
		await _frames(4)
	ms = await _until(func(): return _banner_up(), 3000)
	_ok("Revert to opened over an outside change raises the conflict banner", ms >= 0 and _btn("keep_mine") != null)
	_ok("... and the outside change is untouched on disk", _disk("Qundo.md") == "outside change\n", _disk("Qundo.md"))
	await _press(vw, _btn("take_theirs"))
	await _frames(3)
	var w_same: int = vw._writes_attempted
	await _press(vw, rb)
	dlg = _confirm_dialog("Revert")
	_ok("Revert is asked even when theirs was just taken", dlg != null)
	if dlg != null:
		dlg.get_cancel_button().pressed.emit()
		await _frames(3)

	# == 11. The history's caps and a rename ===================================
	var h: Array = ["first"]
	for i in 149:
		h.append("c%d" % i)
	vw._trim_history(h)
	_ok("150 checkpoints are cut to exactly 100, keeping the first-opened and the latest", h.size() == 100 and h[0] == "first" and h[99] == "c148", "%d %s %s" % [h.size(), h[0], h[h.size() - 1]])
	var big: Array = ["A".repeat(1000000), "B".repeat(1000000), "C".repeat(2000000)]
	vw._trim_history(big)
	_ok("exactly 4,000,000 characters is within the cap", big.size() == 3)
	big.append("D")
	vw._trim_history(big)
	_ok("one character over drops the oldest after the first, never the first", big.size() == 3 and String(big[0]).begins_with("A") and String(big[1]).begins_with("C"), str(big.size()))
	vw._hist["Qren.md"] = ["old", "new"]
	vw.history_follow_rename("Qren.md", "Qren2.md")
	_ok("the history follows a rename", not vw._hist.has("Qren.md") and vw._hist.has("Qren2.md") and (vw._hist["Qren2.md"] as Array)[0] == "old")
	vw._hist.clear()

	if _shots != "":
		DirAccess.make_dir_recursive_absolute(_shots)
	_finish()
