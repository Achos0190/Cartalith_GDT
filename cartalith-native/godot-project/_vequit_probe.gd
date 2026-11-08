extends Node
## The VE-4 gap closed in the VE-5 session: **quitting the app flushes the vault editor's pending
## autosave**, and a standing conflict never silently drops the user's text. Also pins that a note
## autosave does not mark the WORLD dirty (`engine_bridge.gd::vault_write_file`).
##
##   Godot_v4.7.2-stable_win64_console.exe --path . _vequit_probe.tscn -- --route NAME [--vp 1600x1000] [--force-touch]
##
## **One route per process**, because a quit that really happens ends the process: the probe sends the
## real notification or calls the real exit path, reads the disk in the SAME frame (the flush is
## synchronous, before `quit()` takes effect at the end of the frame) and prints its verdict before the
## engine exits. Routes: `close` (the window manager's close request, which is what the title-bar X,
## Alt+F4 and a menu Quit arrive as), `back` (Android's back at the end of its navigation), `pause`
## (Android's `NOTIFICATION_APPLICATION_PAUSED`), `focus` (window focus-out), `clean` (nothing typed),
## `conflict_close` / `conflict_banner_close` / `conflict_pause` / `conflict_focus` (the file was changed
## outside, before / after the banner went up), `unwritable` (the vault folder is gone), `dirty` (the
## world's unsaved mark).
##
## **Windowed**, wiped `APPDATA`, a throw-away vault in TEMP. Exit 0 = every check held, 1 = a check
## failed, 2 = could not run, 3 = watchdog. Every file assertion re-reads the disk.

var _vp: SubViewport
var app: Node
var _bridge
var vw: VaultWindow
var ed
var te: TextEdit
var _root := ""
var _n := 0
var _fails: Array = []
var _route := ""


func _ok(label: String, cond: bool, detail: String = "") -> void:
	_n += 1
	if cond:
		print("VQ  ok  %s" % label)
	else:
		_fails.append(label)
		print("VQ  !!  %s   %s" % [label, detail])


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


func _find_button(n: Node, text: String) -> Button:
	if n is Button and String((n as Button).text).to_lower() == text.to_lower() and (n as Button).is_visible_in_tree():
		return n as Button
	for c in n.get_children():
		var r := _find_button(c, text)
		if r != null:
			return r
	return null


func _type(s: String) -> void:
	for i in s.length():
		for pressed in [true, false]:
			var k := InputEventKey.new()
			k.pressed = pressed
			k.unicode = s.unicode_at(i)
			k.keycode = s.unicode_at(i) as Key
			_vp.push_input(k)
		await _frames(2)


## Every `Qalpha (my edits ...).md` beside the note: the rescue copies.
func _copies() -> Array:
	var out: Array = []
	var d := DirAccess.open(_root)
	if d == null:
		return out
	for f in d.get_files():
		if f.begins_with("Qalpha (my edits"):
			out.append(f)
	return out


func _finish() -> void:
	print("VQ RESULT route=%s %d checks, %d failed%s" % [_route, _n, _fails.size(), "" if _fails.is_empty() else " :: " + str(_fails)])
	get_tree().quit(0 if _fails.is_empty() else 1)


func _banner_up() -> bool:
	return ed != null and is_instance_valid(ed) and ed.banner != null and ed.banner.visible


## Types `mine` at the end of Qalpha WITHOUT waiting for the autosave (the pause is 1 s; typing is
## quicker), so the text is exactly what a quit would find pending.
func _type_pending(mine: String) -> void:
	te.grab_focus()
	var last := te.get_line_count() - 1
	te.set_caret_line(last)
	te.set_caret_column(te.get_line(last).length())
	await _type(mine)


func _ready() -> void:
	_route = _arg("--route", "")
	if _route == "":
		print("VQ ABORT: --route NAME is required")
		get_tree().quit(2)
		return
	if DisplayServer.get_name() == "headless":
		print("VQ REFUSED: run windowed")
		get_tree().quit(2)
		return
	var wd := Timer.new()
	wd.wait_time = 120.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("VQ WATCHDOG TIMEOUT")
		get_tree().quit(3))
	wd.start()
	var parts: PackedStringArray = _arg("--vp", "1600x1000").split("x")
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

	_root = OS.get_environment("TEMP").replace("\\", "/") + "/cartalith-vequit-%s-%d" % [_route, Time.get_ticks_msec()]
	_write(_root + "/Q0first.md", "# Q0first\n")
	_write(_root + "/Qalpha.md", "# Qalpha\n\nbase text\n")
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
	if open_btn == null:
		_ok("CONTROL: Open to edit is offered", false)
		_finish()
		return
	open_btn.pressed.emit()
	await _frames(5)
	ed = vw._editor
	if ed == null:
		_ok("CONTROL: the note opens in the editor", false)
		_finish()
		return
	te = ed.text_edit
	var base := _disk("Qalpha.md")
	_ok("CONTROL: the note on disk is the fixture", base == "# Qalpha\n\nbase text\n", base)

	match _route:
		"close", "back", "pause", "focus":
			await _type_pending(" QUIT")
			_ok("CONTROL: the typed text is pending, not yet on disk", _disk("Qalpha.md") == base and vw._is_dirty(), _disk("Qalpha.md"))
			_fire(_route)
			var d := _disk("Qalpha.md")
			_ok("%s: the pending text is on disk the moment the route returns" % _route, d == base + " QUIT", d)
			_ok("%s: no rescue copy was needed" % _route, _copies().is_empty(), str(_copies()))
			_ok("%s: the editor is clean afterwards" % _route, not vw._is_dirty())
		"clean":
			var w0: int = vw._writes_attempted
			var m0 := FileAccess.get_modified_time(_root + "/Qalpha.md")
			_fire("close")
			_ok("clean: quitting a note nobody edited writes nothing", vw._writes_attempted == w0 and _disk("Qalpha.md") == base)
			_ok("clean: ... and the file's time did not move", FileAccess.get_modified_time(_root + "/Qalpha.md") == m0)
			_ok("clean: ... and makes no copy", _copies().is_empty())
		"conflict_close", "conflict_banner_close", "conflict_pause", "conflict_focus":
			await _type_pending(" MINE")
			_write(_root + "/Qalpha.md", "# Qalpha\n\nTHEIRS changed outside\n")
			if _route != "conflict_close":
				var ms := await _until(func(): return _banner_up(), 4000)
				_ok("the outside change raises the banner (%d ms)" % ms, ms >= 0)
			var theirs := _disk("Qalpha.md")
			var kind := _route.trim_prefix("conflict_")
			_fire(kind if kind != "banner_close" else "close")
			_ok("%s: THEIR file is untouched" % _route, _disk("Qalpha.md") == theirs, _disk("Qalpha.md"))
			if kind == "focus":
				_ok("focus-out with a conflict makes no copy (leaving the window is not leaving the app)", _copies().is_empty(), str(_copies()))
				_ok("... and the text is still in the editor", te.text.ends_with(" MINE"))
			else:
				var cs := _copies()
				_ok("%s: exactly one rescue copy was made" % _route, cs.size() == 1, str(cs))
				if cs.size() == 1:
					_ok("%s: the copy is MY full text" % _route, _disk(String(cs[0])) == "# Qalpha\n\nbase text\n MINE", _disk(String(cs[0])))
				if kind == "pause":
					_fire("pause")
					_fire("pause")
					_ok("pause again: the same text is not copied twice", _copies().size() == 1, str(_copies()))
					te.insert_text_at_caret("!")
					_fire("pause")
					_ok("pause after more typing: the new text gets its own copy", _copies().size() == 2, str(_copies()))
					_ok("... and THEIR file is still untouched", _disk("Qalpha.md") == theirs)
			_ok("%s: my text is still in the editor" % _route, te.text.contains(" MINE"))
		"unwritable":
			await _type_pending(" LOST?")
			_write(_root + "/Qalpha.md", "# Qalpha\n\nTHEIRS\n")
			var ms2 := await _until(func(): return _banner_up(), 4000)
			_ok("the outside change raises the banner (%d ms)" % ms2, ms2 >= 0)
			var gone := _root + "_gone"
			var dirs := DirAccess.open(_root.get_base_dir())
			_ok("CONTROL: the vault folder can be moved away", dirs.rename(_root.get_file(), gone.get_file()) == OK)
			## The banner is taken down by hand first, so that it can only be back because the exit gate
			## brought it back (`surface_for_exit`), not because it was still up from the refusal.
			ed.hide_banner()
			_ok("CONTROL: the banner is down before the close request", not _banner_up())
			## The window is hidden without its own close handler running (that handler would save and ask
			## by itself, which is a different path), so only the exit gate can bring it back.
			vw.visibility_changed.disconnect(vw._on_visibility_changed)
			vw.hide()
			vw.visibility_changed.connect(vw._on_visibility_changed)
			await _frames(2)
			_ok("CONTROL: the vault window is hidden before the close request", not vw.visible)
			app._close_requested()
			_ok("a close that cannot make the text safe does not go ahead the first time", app._vault_exit_warned)
			await _frames(3)
			_ok("... the vault window shows why (banner back up)", vw.visible and _banner_up())
			_ok("... and my text is still in the editor", is_instance_valid(te) and te.text.ends_with(" LOST?"))
			## The escape hatch (the invariant `_close_requested` documents): the SECOND request, with the
			## folder still gone, is let through, so a vault that can never be written cannot make the app
			## un-closeable.
			_ok("a second close request is allowed through while the folder is still gone", app._vault_safe_to_exit())
			dirs.rename(gone.get_file(), _root.get_file())
			_ok("the folder is back and THEIR file was never touched", _disk("Qalpha.md") == "# Qalpha

THEIRS
")
			app._close_requested()
		"dirty":
			## The world's unsaved mark: a note autosave is not a world edit. `has_world` is set only for
			## this check, so the real `mark_world_dirty` runs.
			_bridge.has_world = true
			_bridge.world_dirty = false
			await _type_pending(" X")
			var ms3 := await _until(func(): return _disk("Qalpha.md") != base, 3000)
			_ok("the autosave wrote the note (%d ms)" % ms3, ms3 >= 0)
			_ok("... and the world is NOT marked unsaved by it", not _bridge.world_dirty)
			_bridge.has_world = false
		_:
			_ok("known route", false, _route)
	_finish()


## The exits, each the real path. `close` is `NOTIFICATION_WM_CLOSE_REQUEST` sent to the app's node (the
## window manager's request reaches the main window's tree exactly so); the others likewise.
func _fire(route: String) -> void:
	match route:
		"close":
			app.notification(Node.NOTIFICATION_WM_CLOSE_REQUEST)
		"back":
			app._back_exhausted()
		"pause":
			app.notification(Node.NOTIFICATION_APPLICATION_PAUSED)
		"focus":
			app.notification(Node.NOTIFICATION_WM_WINDOW_FOCUS_OUT)
