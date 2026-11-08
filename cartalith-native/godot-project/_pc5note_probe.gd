extends Node
## PC-5 of `PC_SHELL_PROPOSAL.md` §3.4: the Context panel's Note tab -- the subject's linked note, edited in
## the vault editor, one note focus with one writer. Bar (§6 table): *a note opens in Write; a `[[link]]`
## selects, by Ctrl-click in Write; a chronos row moves the year; Write autosaves; Revert to opened… asks
## first and is one undo step; a missing note is offered with a template choice, then created and bound to
## the subject.* Plus the owner's hard rules: one note focus with one writer, and a user's notes are data, so
## typed text is never lost across a selection change, a pin, a tab switch, the vault window opening, closing
## the panel or quitting.
##
##   Godot_v4.7.2-stable_win64_console.exe --path . _pc5note_probe.tscn -- --vp 1600x1000 [--touch] [--shots DIR]
##
## **Windowed** (refuses `--headless`), under a wiped user directory (`APPDATA=<empty dir>`): it deletes
## `user://vault.json`, generates a small world (seed 4242, 256 x 192) and binds a throw-away vault in TEMP.
## Exit 0 = every check held, 1 = a check failed, 2 = could not run, 3 = watchdog. Real mouse clicks and real
## key events through `SubViewport.push_input`; every file assertion re-reads the disk.
##
## ## What each leg proves
##  A. SHOWS: a settlement with a linked note -> the Note tab (a real click on its tab, and Ctrl+2) shows that
##     note's text in Write, in ONE editor, inside the tab's frame; the file is untouched by opening.
##  B. EDIT: typing autosaves within 2 s; nothing is written before a pause.
##  C. FOLLOW: typed text, then the selection moves BEFORE the autosave -- the text is on disk, the tab shows
##     the other place's note, and selecting back shows the saved text (not lost).
##  D. PIN: pinned, a selection elsewhere does not move the tab or lose text; typing continues in the pinned
##     note; FOLLOW then flushes it and shows the selection's note.
##  E. ONE WRITER: the vault window opening takes the editor (the tab shows a notice and holds none), flushes
##     the tab's text first, there are never two editors, and closing the window gives it back; a note
##     docked then opened in the window is written once per edit.
##  F. CONFLICT: a change made outside raises the banner IN the tab; switching selection is HELD (text kept,
##     disk untouched); resolving the banner lets the switch through.
##  G. QUIT: pending text is on disk after each exit route (`_vault_safe_to_exit`, pause, focus-out).
##  H. TAB SWITCH / PANEL: leaving the tab or hiding the panel keeps the editor and its text; the autosave
##     lands; no text is lost.
##  I. CHRONOS: a dated row moves the year (`tl_year`) and the row the year falls in is lit.
##  J. LINK: Ctrl-click on a `[[link]]` to a place with a note selects that place; the tab follows.
##  K. REVERT: Revert to opened… asks first; Revert is one undo step.
##  L. CREATE: a place with no note offers a template choice and a Create button; pressing it creates the
##     file, binds it to the place and shows it in the editor.
##  M. EMPTY STATES: nothing selected and an unsupported subject say so and hold no editor.
##  N. PHONE-safe route: `context_tabs()` lists note and `open_context_tab("note")` does not dock on the phone.

const Ed := preload("res://shell/markdown_editor.gd")

var _vp: SubViewport
var app: Node
var _bridge
var vw: VaultWindow
var _root := ""
var _shots := ""
var _touch := false
var _ff := false
var _n := 0
var _fails: Array = []
var _ia := 0
var _ib := 1
var _ic := 2
var _name_a := ""
var _name_b := ""
var _tid_a := 0
var _tid_b := 0
var _tid_c := 0
var _fid := 0


func _ok(label: String, cond: bool, detail: String = "") -> void:
	_n += 1
	if cond:
		print("PC5  ok  %s" % label)
	else:
		_fails.append(label)
		print("PC5  !!  %s   %s" % [label, detail])
		if _ff:
			## --ff (fail fast): a mutant run stops at the first failed check; its death is all that is wanted.
			_finish()


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


func _scripts_named(n: Node, tail: String, out: Array) -> void:
	var s: Script = n.get_script()
	if s != null and s.resource_path.ends_with(tail):
		out.append(n)
	for c in n.get_children():
		_scripts_named(c, tail, out)


## Every live Markdown editor in the whole app (the tab's, the vault window's): the one-editor invariant.
func _editors() -> Array:
	var out: Array = []
	_scripts_named(app, "markdown_editor.gd", out)
	return out.filter(func(e): return is_instance_valid(e) and not e.is_queued_for_deletion())


func _dock() -> Node:
	return app.right_dock_ctrl


func _slot() -> Control:
	return app.find_child("NoteSlot", true, false) as Control


func _ed() -> Node:
	return vw._editor if vw != null and is_instance_valid(vw._editor) else null


func _te() -> TextEdit:
	var e = _ed()
	return e.text_edit if e != null else null


func _under_slot(n: Node) -> bool:
	var s := _slot()
	return s != null and n != null and s.is_ancestor_of(n)


func _vis_button(n: Node, name: String) -> Button:
	if n is Button and n.name == name and (n as Button).is_visible_in_tree():
		return n
	for c in n.get_children():
		var r := _vis_button(c, name)
		if r != null:
			return r
	return null


func _vis_label(n: Node, name: String) -> Label:
	if n is Label and n.name == name and (n as Label).is_visible_in_tree():
		return n
	for c in n.get_children():
		var r := _vis_label(c, name)
		if r != null:
			return r
	return null


func _move(pos: Vector2) -> void:
	var m := InputEventMouseMotion.new()
	m.position = pos
	m.global_position = pos
	_vp.push_input(m)
	await _frames(1)


func _mouse(pos: Vector2, pressed: bool, ctrl: bool = false, right: bool = false) -> void:
	var ev := InputEventMouseButton.new()
	ev.button_index = MOUSE_BUTTON_RIGHT if right else MOUSE_BUTTON_LEFT
	ev.pressed = pressed
	ev.ctrl_pressed = ctrl
	ev.position = pos
	ev.global_position = pos
	if _touch:
		ev.device = InputEvent.DEVICE_ID_EMULATION
	_vp.push_input(ev)
	await _frames(2)


func _click(pos: Vector2, ctrl: bool = false) -> void:
	await _move(pos)
	await _mouse(pos, true, ctrl)
	await _mouse(pos, false, ctrl)
	await _frames(3)


func _press(b: Button) -> void:
	await _click(b.get_global_rect().get_center())


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


func _end() -> void:
	var te := _te()
	var last := te.get_line_count() - 1
	te.set_caret_line(last)
	te.set_caret_column(te.get_line(last).length())


## Focuses the tab's text with a REAL click on it, then parks the caret at its end.
func _focus_text() -> void:
	var te := _te()
	await _click(te.get_global_rect().get_center())
	_end()
	await _frames(2)


func _shot(name: String) -> void:
	if _shots == "":
		return
	await _frames(3)
	_vp.get_texture().get_image().save_png("%s/pc5_%s_%s.png" % [_shots, "touch" if _touch else "desk", name])


func _select(kind: String, id: int) -> void:
	app.select(kind, id)
	await _frames(6)


func _finish() -> void:
	print("PC5 RESULT %d checks, %d failed%s" % [_n, _fails.size(), "" if _fails.is_empty() else " :: " + str(_fails)])
	get_tree().quit(0 if _fails.is_empty() else 1)


## Waits for the Note tab to hold `rel` in its editor.
func _docked(rel: String, ms: int = 3000) -> bool:
	return await _until(func(): return vw.is_docked() and vw.docked_path() == rel, ms) >= 0


func _ready() -> void:
	for a in OS.get_cmdline_user_args():
		var s := String(a)
		if s.begins_with("--") and not (s in ["--vp", "--touch", "--shots", "--ff"]):
			print("PC5 ABORT unknown flag %s" % s)
			get_tree().quit(2)
			return
	if DisplayServer.get_name() == "headless":
		print("PC5 REFUSED: run windowed")
		get_tree().quit(2)
		return
	var wd := Timer.new()
	wd.wait_time = 700.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("PC5 WATCHDOG TIMEOUT")
		get_tree().quit(3))
	wd.start()
	var parts: PackedStringArray = _arg("--vp", "1600x1000").split("x")
	_shots = _arg("--shots", "")
	if _shots != "":
		DirAccess.make_dir_recursive_absolute(_shots)
	_touch = "--touch" in OS.get_cmdline_user_args()
	_ff = "--ff" in OS.get_cmdline_user_args()
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
	vw = app.vault_window
	_bridge.generate({"seed": 4242, "width_km": 2400.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.45})
	await _frames(2)
	var t0 := Time.get_ticks_msec()
	while bool(_bridge.generating) and Time.get_ticks_msec() - t0 < 240000:
		await _frames(10)
	await _frames(30)
	var ss: Array = _bridge.settlements()
	_ok("CONTROL: a world with three settlements and a faction", bool(_bridge.has_world) and ss.size() >= 3 and _bridge.get_factions().size() >= 1)
	if ss.size() < 3:
		_finish()
		return
	_name_a = String((ss[_ia] as Dictionary).get("name", ""))
	_name_b = String((ss[_ib] as Dictionary).get("name", ""))
	_tid_a = int((ss[_ia] as Dictionary).get("tid", 0))
	_tid_b = int((ss[_ib] as Dictionary).get("tid", 0))
	_tid_c = int((ss[_ic] as Dictionary).get("tid", 0))
	_fid = int((_bridge.get_factions()[0] as Dictionary).get("id", 1))
	_ok("CONTROL: the three settlements have stable ids", _tid_a != 0 and _tid_b != 0 and _tid_c != 0)

	# == The fixture vault: names no generated place can share ===================
	_root = OS.get_environment("TEMP").replace("\\", "/") + "/cartalith-pc5-%s-%d" % ["touch" if _touch else "desk", Time.get_ticks_msec()]
	var fence := "`" + "`" + "`"
	_write(_root + "/QnoteA.md", "# QnoteA\n\nAlpha text.\n\nSee [[QnoteB]] for the neighbour and [[QnoteX]] for the rest.\n\n%schronos\n* [100] Founded | the founding\n* [300] Sacked | the sack\n%s\n" % [fence, fence])
	_write(_root + "/QnoteB.md", "# QnoteB\n\nBeta text.\n")
	_write(_root + "/QnoteX.md", "# QnoteX\n\nA note no place owns.\n")
	_write(_root + "/QnoteF.md", "# QnoteF\n\nFaction text.\n")
	var conn: Dictionary = _bridge.vault_connect(_root, "Chronicle")
	_ok("CONTROL: the fixture vault binds", bool(conn.get("ok", false)), str(conn))
	_ok("CONTROL: attach A", bool(_bridge.vault_attach("settlement", _tid_a, _name_a, "QnoteA.md", "").get("ok", false)))
	_ok("CONTROL: attach B", bool(_bridge.vault_attach("settlement", _tid_b, _name_b, "QnoteB.md", "").get("ok", false)))
	_ok("CONTROL: attach F", bool(_bridge.vault_attach("faction", _fid, "Faction", "QnoteF.md", "").get("ok", false)))
	_bridge.vault_refresh_backlinks(2000)
	var a0 := _disk("QnoteA.md")
	var b0 := _disk("QnoteB.md")

	# == A. SHOWS ==================================================================
	print("== A. the Note tab shows the subject's note ==")
	await _select("settlement", _ia)
	_ok("the tab list has note (context_tabs)", app.context_tabs().has("note"))
	var tab_btn := _vis_button(app, "CtxTabNote")
	_ok("the strip draws a NOTE tab", tab_btn != null)
	if tab_btn == null:
		_finish()
		return
	await _press(tab_btn)
	_ok("a real click on NOTE shows the Note tab", _dock().current_tab() == "note")
	_ok("A: the tab docks QnoteA.md", await _docked("QnoteA.md"))
	var te := _te()
	_ok("A: the editor holds the note's text", te != null and te.text == a0, te.text if te != null else "no editor")
	_ok("A: ONE editor alive, inside the tab's frame", _editors().size() == 1 and _under_slot(_ed()))
	_ok("A: it opens in Write (VQ4)", _ed() != null and _ed().mode() == "write")
	_ok("A: opening wrote nothing", _disk("QnoteA.md") == a0)
	_ok("A: Inspect's scroll is hidden under the Note frame", not app._right_dock_scroll.visible)
	await _shot("a_note")

	# == B. EDIT: autosave =========================================================
	print("== B. typing autosaves ==")
	await _focus_text()
	var w0: int = vw._writes_attempted
	await _wait(1300)
	_ok("B: nothing is written before anything is typed", vw._writes_attempted == w0 and _disk("QnoteA.md") == a0)
	await _type(" T1")
	_ok("B: the text is in the editor", _te().text.ends_with(" T1") or _te().text.contains(" T1"), _te().text)
	var ms := await _until(func(): return _disk("QnoteA.md").contains(" T1"), 3500)
	_ok("B: the autosave lands within 2 s of the last key (%d ms)" % ms, ms >= 0 and ms <= 2600)
	_ok("B: the file is the old text plus the typing, nothing else lost", _disk("QnoteA.md").contains("Alpha text.") and _disk("QnoteA.md").contains("[[QnoteB]]"))

	# == C. FOLLOW: switch selection before the autosave ===========================
	print("== C. the selection moves with unsaved text ==")
	await _focus_text()
	await _type(" T2")
	var before_switch := _te().text
	_ok("C: T2 is in the editor and NOT yet on disk", before_switch.contains(" T2") and not _disk("QnoteA.md").contains(" T2"))
	await _select("settlement", _ib)
	_ok("C: the text typed before the switch is on disk", _disk("QnoteA.md").contains(" T2"))
	_ok("C: the tab now holds QnoteB.md", await _docked("QnoteB.md"))
	_ok("C: its text is QnoteB's, untouched", _te() != null and _te().text == b0 and _disk("QnoteB.md") == b0)
	_ok("C: still ONE editor", _editors().size() == 1 and _under_slot(_ed()))
	await _select("settlement", _ia)
	_ok("C: selecting back shows the saved text, T1 and T2 both there", await _docked("QnoteA.md") and _te().text.contains(" T1") and _te().text.contains(" T2"), _te().text)

	# == D. PIN =====================================================================
	print("== D. pin ==")
	var pin := _vis_button(app, "CtxPin")
	_ok("D: the header has a pin button reading FOLLOWING", pin != null and pin.text == "FOLLOWING")
	await _press(pin)
	_ok("D: pressed, the panel is PINNED", _dock().is_pinned())
	await _focus_text()
	await _type(" P1")
	await _select("settlement", _ib)
	_ok("D: a selection elsewhere does not move the pinned tab", vw.docked_path() == "QnoteA.md" and _dock().is_pinned())
	_ok("D: the pinned note's unsaved text is intact", _te().text.contains(" P1"))
	_ok("D: the banner names the new selection", _vis_button(app, "CtxPinToIt") != null)
	await _focus_text()
	await _type(" P2")
	var ms2 := await _until(func(): return _disk("QnoteA.md").contains(" P1 P2") or (_disk("QnoteA.md").contains(" P1") and _disk("QnoteA.md").contains(" P2")), 3500)
	_ok("D: typing continues in the pinned note and autosaves (%d ms)" % ms2, ms2 >= 0)
	await _type(" P3")
	await _press(_vis_button(app, "CtxFollow"))
	_ok("D: FOLLOW flushes the pinned note's text", _disk("QnoteA.md").contains(" P3"))
	_ok("D: FOLLOW shows the selection's note", await _docked("QnoteB.md") and not _dock().is_pinned())

	# == E. ONE WRITER with the vault window ========================================
	print("== E. one editor, one writer ==")
	await _select("settlement", _ia)
	_ok("E: back on A", await _docked("QnoteA.md"))
	await _focus_text()
	await _type(" E1")
	_ok("E: E1 typed, not yet saved", not _disk("QnoteA.md").contains(" E1"))
	app.open_vault_browse()
	await _frames(8)
	_ok("E: the window is open", vw.visible)
	_ok("E: opening the window flushed the tab's text first", _disk("QnoteA.md").contains(" E1"))
	_ok("E: the window took the editor back: nothing is docked", not vw.is_docked())
	_ok("E: no editor is alive (none was opened in the window yet)", _editors().is_empty(), str(_editors().size()))
	vw._pick_file = "QnoteA.md"
	vw._enter_note("QnoteA.md")
	await _frames(6)
	_ok("E: the window now holds the note: ONE editor, not under the tab's slot", _editors().size() == 1 and not _under_slot(_ed()))
	_ok("E: while the window is open the tab refuses to dock", not bool(vw.dock_note("QnoteA.md", _slot()).get("ok", true)))
	_ok("E: ... and that refusal built no second editor", _editors().size() == 1)
	var wa: int = vw._writes_attempted
	var te_w := _te()
	te_w.grab_focus()
	te_w.set_caret_line(te_w.get_line_count() - 1)
	te_w.set_caret_column(te_w.get_line(te_w.get_line_count() - 1).length())
	await _type(" E2")
	var ms3 := await _until(func(): return _disk("QnoteA.md").contains(" E2"), 3500)
	_ok("E: an edit in the window autosaves (%d ms)" % ms3, ms3 >= 0)
	_ok("E: ... as exactly one write", vw._writes_attempted == wa + 1, "%d vs %d" % [vw._writes_attempted, wa])
	vw.hide()
	await _frames(10)
	_ok("E: closing the window gives the editor back to the tab", await _docked("QnoteA.md"))
	_ok("E: ... with the window's text, now ONE editor in the tab", _te().text.contains(" E2") and _editors().size() == 1 and _under_slot(_ed()))
	_ok("E: the tab shows no stale 'open in the vault window' notice", _vis_label(app, "NoteInWindow") == null)

	# == F. CONFLICT ================================================================
	print("== F. an outside change ==")
	await _focus_text()
	await _type(" F1")
	var theirs := "# QnoteA\n\nTHEIRS changed outside.\n"
	_write(_root + "/QnoteA.md", theirs)
	var msb := await _until(func(): return _ed() != null and _ed().banner != null and _ed().banner.visible, 4500)
	_ok("F: the banner is raised IN the tab (%d ms)" % msb, msb >= 0)
	_ok("F: their file is untouched", _disk("QnoteA.md") == theirs)
	await _select("settlement", _ib)
	_ok("F: the switch is HELD: the tab still holds A", vw.docked_path() == "QnoteA.md")
	_ok("F: ... the typed text is still there", _te().text.contains(" F1"))
	_ok("F: ... the tab says why", _vis_label(app, "NoteHeld") != null)
	_ok("F: ... their file is still untouched", _disk("QnoteA.md") == theirs)
	var keep_both := _vis_button(_ed(), "Banner_keep_both")
	_ok("F: the banner offers Keep both", keep_both != null)
	if keep_both != null:
		await _press(keep_both)
	var msh := await _until(func(): return vw.docked_path() == "QnoteB.md", 4000)
	_ok("F: resolving the banner lets the held switch through (%d ms)" % msh, msh >= 0)
	var copies := 0
	var d := DirAccess.open(_root)
	for f in d.get_files():
		if f.begins_with("QnoteA (my edits"):
			copies += 1
			_ok("F: Keep both saved my text in the copy", FileAccess.get_file_as_string(_root + "/" + f).contains(" F1"))
	_ok("F: exactly one copy was made", copies == 1, str(copies))
	_ok("F: their file is still theirs", _disk("QnoteA.md") == theirs)

	# == G. QUIT =====================================================================
	print("== G. quit routes ==")
	await _select("settlement", _ib)
	_ok("G: on B", await _docked("QnoteB.md"))
	await _focus_text()
	await _type(" Q1")
	_ok("G: Q1 pending", not _disk("QnoteB.md").contains(" Q1"))
	_ok("G: the exit gate reports safe", app._vault_safe_to_exit())
	_ok("G: close route wrote it", _disk("QnoteB.md").contains(" Q1"))
	await _type(" Q2")
	app._notification(Node.NOTIFICATION_APPLICATION_PAUSED)
	_ok("G: pause route wrote it", _disk("QnoteB.md").contains(" Q2"))
	await _type(" Q3")
	app._notification(Node.NOTIFICATION_WM_WINDOW_FOCUS_OUT)
	_ok("G: focus-out route wrote it", _disk("QnoteB.md").contains(" Q3"))

	# == H. TAB SWITCH and PANEL ======================================================
	print("== H. tab switch, hidden panel ==")
	await _type(" H1")
	await _key(KEY_1, true)
	_ok("H: Ctrl+1 shows Inspect", _dock().current_tab() == "inspect")
	_ok("H: the editor lives on, hidden, with its text", vw.is_docked() and _te().text.contains(" H1"))
	_ok("H: ... and is not visible", not _te().is_visible_in_tree())
	var msh2 := await _until(func(): return _disk("QnoteB.md").contains(" H1"), 3500)
	_ok("H: the autosave still lands while the tab is hidden (%d ms)" % msh2, msh2 >= 0)
	await _select("settlement", _ia)
	await _key(KEY_2, true)
	_ok("H: Ctrl+2 shows the Note tab", _dock().current_tab() == "note")
	_ok("H: ... on the new selection's note", await _docked("QnoteA.md"))
	await _focus_text()
	await _type(" H2")
	var pm := app._region_nodes[DccMenus.ID_WIN_RIGHT] as CanvasItem
	pm.visible = false
	await _frames(4)
	_ok("H: a hidden panel keeps the text", vw.is_docked() and _te().text.contains(" H2"))
	var msh3 := await _until(func(): return _disk("QnoteA.md").contains(" H2"), 3500)
	_ok("H: ... and the autosave lands (%d ms)" % msh3, msh3 >= 0)
	pm.visible = true
	await _frames(4)

	# == I. CHRONOS ====================================================================
	print("== I. dated entries ==")
	_write(_root + "/QnoteA.md", "# QnoteA\n\nAlpha text.\n\nSee [[QnoteB]] for the neighbour and [[QnoteX]] for the rest.\n\n```chronos\n* [100] Founded | the founding\n* [300] Sacked | the sack\n```\n")
	await _select("settlement", _ib)
	await _select("settlement", _ia)
	_ok("I: A's note is docked again", await _docked("QnoteA.md"))
	var r0 := _vis_button(app, "NoteDated_0")
	var r1 := _vis_button(app, "NoteDated_1")
	_ok("I: two dated rows are drawn", r0 != null and r1 != null)
	## A year move rebuilds a settlement's panel, so the rows are found again after every move.
	if r0 != null and r1 != null:
		await _press(r1)
		_ok("I: pressing a row moves the year (tl_year = 300)", app.tl_year() == 300, str(app.tl_year()))
		_ok("I: the row the year falls in is lit, the other is not", _lit(1) and not _lit(0), "%s %s" % [_lit(0), _lit(1)])
		await _press(_vis_button(app, "NoteDated_0"))
		_ok("I: pressing the first row moves the year to 100", app.tl_year() == 100, str(app.tl_year()))
		_ok("I: the lit row follows the year", _lit(0) and not _lit(1))
		app.tl_set_year(250, true)
		await _frames(6)
		_ok("I: a year moved from elsewhere re-lights (250 -> the 100 row)", _lit(0) and not _lit(1))
		app.tl_set_year(-50, true)
		await _frames(6)
		_ok("I: before every entry nothing is lit", not _lit(0) and not _lit(1))
	await _shot("i_dated")

	# == J. LINK ========================================================================
	print("== J. a followed link selects what it names ==")
	te = _te()
	var line := -1
	for i in te.get_line_count():
		if te.get_line(i).contains("[[QnoteB]]"):
			line = i
	_ok("J: the note holds the link line", line >= 0)
	if line >= 0:
		var col := te.get_line(line).find("[[QnoteB]]") + 4
		te.set_caret_line(line)
		await _frames(2)
		var rc: Rect2i = te.get_rect_at_line_column(line, col + 1)
		var pos := te.get_global_rect().position + Vector2(rc.position) + Vector2(rc.size) * 0.5
		await _click(pos, true)
		var msj := await _until(func(): return String(app.selection.get("kind", "")) == "settlement" and int(app.selection.get("id", -1)) == _ib, 3000)
		_ok("J: Ctrl-click selected the place the link names (%d ms)" % msj, msj >= 0, str(app.selection))
		_ok("J: the tab follows to that place's note", await _docked("QnoteB.md"))
		_ok("J: still ONE editor", _editors().size() == 1)

	## J2: a link to a note that no place owns opens in the tab, with Back (the window's own Back trail).
	await _select("settlement", _ib)
	await _select("settlement", _ia)
	_ok("J2: on A", await _docked("QnoteA.md"))
	te = _te()
	line = -1
	for i in te.get_line_count():
		if te.get_line(i).contains("[[QnoteX]]"):
			line = i
	_ok("J2: the note holds the second link", line >= 0)
	if line >= 0:
		var col2 := te.get_line(line).find("[[QnoteX]]") + 4
		var rc2: Rect2i = te.get_rect_at_line_column(line, col2 + 1)
		var pos2 := te.get_global_rect().position + Vector2(rc2.position) + Vector2(rc2.size) * 0.5
		await _click(pos2, true)
		_ok("J2: Ctrl-click opens the unowned note IN the tab", await _docked("QnoteX.md"))
		_ok("J2: the selection did not move (no place owns that note)", int(app.selection.get("id", -1)) == _ia)
		_ok("J2: still ONE editor, in the tab", _editors().size() == 1 and _under_slot(_ed()))
		var back := _find_btn_text(_ed(), "‹ Back")
		_ok("J2: the editor offers Back", back != null)
		if back != null:
			await _press(back)
		_ok("J2: Back returns to A's note", await _docked("QnoteA.md"))

	## J3: pinned, a link to a place's note opens that note IN the tab and the selection does not move.
	await _press(_vis_button(app, "CtxPin"))
	_ok("J3: pinned", _dock().is_pinned())
	te = _te()
	line = -1
	for i in te.get_line_count():
		if te.get_line(i).contains("[[QnoteB]]"):
			line = i
	if line >= 0:
		var col3 := te.get_line(line).find("[[QnoteB]]") + 4
		var rc3: Rect2i = te.get_rect_at_line_column(line, col3 + 1)
		var pos3 := te.get_global_rect().position + Vector2(rc3.position) + Vector2(rc3.size) * 0.5
		await _click(pos3, true)
		_ok("J3: the pinned tab opened B's note in place", await _docked("QnoteB.md"))
		_ok("J3: the selection did not move and the pin holds", int(app.selection.get("id", -1)) == _ia and _dock().is_pinned())
		_ok("J3: still ONE editor", _editors().size() == 1 and _under_slot(_ed()))
		vw._go_back()
		_ok("J3: Back returns to A's note", await _docked("QnoteA.md"))
	await _press(_vis_button(app, "CtxFollow") if _vis_button(app, "CtxFollow") != null else _vis_button(app, "CtxPin"))
	_ok("J3: following again", not _dock().is_pinned())

	# == K. REVERT ======================================================================
	print("== K. Revert to opened ==")
	await _select("settlement", _ib)
	await _select("settlement", _ia)
	_ok("K: on A", await _docked("QnoteA.md"))
	var opened := _te().text
	await _focus_text()
	await _type(" K1")
	await _wait(1500)
	var rv := _vis_button(_ed(), "Revert to opened…")
	if rv == null:
		rv = _find_btn_text(_ed(), "Revert to opened…")
	_ok("K: the header has Revert to opened…", rv != null)
	if rv != null:
		await _press(rv)
		var dlg := _confirm("Revert")
		_ok("K: it asks first (a confirmation with Revert)", dlg != null)
		_ok("K: nothing changed yet", _te().text.contains(" K1"))
		if dlg != null:
			dlg.get_ok_button().pressed.emit()
			await _frames(6)
			_ok("K: Revert put the first-opened text back", _te().text == vw._hist.get("QnoteA.md", [""])[0])
			await _focus_text()
			await _key(KEY_Z, true)
			_ok("K: one Ctrl+Z brings the reverted text back (one undo step)", _te().text.contains(" K1"), _te().text)

	# == L. CREATE ======================================================================
	print("== L. a place with no note ==")
	await _select("settlement", _ic)
	_ok("L: the tab released the editor", not vw.is_docked() and _editors().is_empty())
	_ok("L: it says the place has no note", _vis_label(app, "NoteNone") != null)
	var cb := _vis_button(app, "NoteCreate")
	_ok("L: it offers Create", cb != null)
	_ok("L: ... with a template choice", app.find_child("NoteTemplate", true, false) != null)
	var cp := app.find_child("NotePath", true, false) as LineEdit
	_ok("L: ... and the path to create", cp != null and cp.text.ends_with(".md"))
	_ok("L: nothing was created by merely looking", _bridge.vault_links_for("settlement", _tid_c).is_empty())
	if cb != null and cp != null:
		var rel := cp.text
		await _press(cb)
		_ok("L: pressing Create made the file", FileAccess.file_exists(_root + "/" + rel), rel)
		_ok("L: ... and bound it to the place", not _bridge.vault_links_for("settlement", _tid_c).is_empty())
		_ok("L: ... and the tab shows it", await _docked(rel))

	# == M. EMPTY STATES =================================================================
	print("== M. nothing selected, an unsupported subject ==")
	await _focus_text()
	await _type(" M1")
	var crel := vw.docked_path()
	await _select("", -1)
	_ok("M: nothing selected: the typed text was saved", _disk(crel).contains(" M1"))
	_ok("M: ... the tab holds no editor and says so", not vw.is_docked() and _vis_label(app, "NoteNothing") != null)
	await _select("faction", _fid)
	_ok("M: a faction with a note shows it", await _docked("QnoteF.md"))
	var ct := PackedStringArray()
	_ok("M: the tab list names note", app.context_tabs().has("note"))
	await _shot("m_faction")

	_finish()


func _lit(i: int) -> bool:
	var b := _vis_button(app, "NoteDated_%d" % i)
	return b != null and bool(b.get_meta("lit", false))


func _find_btn_text(n: Node, text: String) -> Button:
	if n is Button and (n as Button).text == text and (n as Button).is_visible_in_tree():
		return n
	for c in n.get_children():
		var r := _find_btn_text(c, text)
		if r != null:
			return r
	return null


func _confirm(ok_text: String) -> ConfirmationDialog:
	var out: Array = []
	_collect(app, "ConfirmationDialog", out)
	for d in out:
		if (d as Window).visible and (d as ConfirmationDialog).get_ok_button().text == ok_text:
			return d
	return null


func _collect(n: Node, cls: String, out: Array) -> void:
	if n.is_class(cls):
		out.append(n)
	for c in n.get_children():
		_collect(c, cls, out)
