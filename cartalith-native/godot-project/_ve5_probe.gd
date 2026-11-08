extends Node
## VE-5 of `VAULT_EDITOR_PROPOSAL.md` s.7: the side panel -- backlinks, the unlinked-mention Link
## button, the outline jump, the properties form and the tags index. Bar: *every listed mention opens
## its note at its line; a frontmatter edit leaves the other lines byte-identical.*
##
##   Godot_v4.7.2-stable_win64_console.exe --path . _ve5_probe.tscn -- --vp 1600x1000 [--touch] [--shots DIR]
##   ... -- --vp 1080x2340 --force-touch   (the phone leg)
##
## **Windowed.** Run under a wiped user directory (`APPDATA=<empty dir>`): it deletes `user://vault.json`
## and binds a throw-away vault in TEMP. Exit 0 = every check held, 1 = a check failed, 2 = could not run.
## Real clicks on real buttons, real typed keys, and every file assertion re-reads the disk.
##
## ## What each leg proves (data first: a user's notes are data)
##
## - **Links.** The backlinks listed are exactly the notes that link here (a wiki link and a Markdown
##   link), each opens its note with the caret on the link's line; the unlinked mentions are listed with
##   their excerpt; **Link** turns the mention into `[[Name]]` / `[[Name|as written]]`, changes nothing
##   else in that note byte for byte, goes through the hash guard (a read-only note is refused and left
##   alone), and the note then appears as a backlink. A save in the editor refreshes the index by itself.
## - **Outline.** Headings in order and indented by level; a click puts the caret on the heading's
##   line in Write, and scrolls Preview there; `[[#Heading]]` in the same note jumps too.
## - **Properties.** Every other line of the note is byte-identical after a scalar edit, a list-item
##   add and remove, a new property and a removed one; each is ONE undo step; the autosave writes the
##   result; a note whose properties cannot be read safely is shown but not editable.
## - **Tags.** The vault's tags with the number of NOTES (not mentions) that carry each, tapping one
##   lists its notes, each opens; a capped scan says it is partial.
## - **Forms.** The panel is beside the editor on a wide window and below it on a narrow one, can be
##   toggled, and its buttons are at least 44 px on touch.

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
		print("VE5  ok  %s" % label)
	else:
		_fails.append(label)
		print("VE5  !!  %s   %s" % [label, detail])


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
		print("VE5 dbg press %s rect=%s vw.pos=%s vw.size=%s scale=%s vis=%s sp=%s ed=%s" % [b.name, b.get_global_rect(), vw.position, vw.size, vw.content_scale_factor, b.is_visible_in_tree(), sp.get_global_rect() if sp != null else null, ed.get_global_rect() if ed != null else null])
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
	print("VE5 RESULT %d checks, %d failed%s" % [_n, _fails.size(), "" if _fails.is_empty() else " :: " + str(_fails)])
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
	_vp.get_texture().get_image().save_png(_shots + "/ve5_%s_%s.png" % [("phone" if _phone else ("touch" if _touch else "desk")), name])


func _lines(t: String) -> PackedStringArray:
	return t.split("\n")


func _ready() -> void:
	for a in OS.get_cmdline_user_args():
		var s := String(a)
		if s.begins_with("--") and not (s in ["--vp", "--touch", "--shots", "--force-touch", "--debug"]):
			print("VE5 ABORT unknown flag %s" % s)
			get_tree().quit(2)
			return
	if DisplayServer.get_name() == "headless":
		print("VE5 REFUSED: run windowed")
		get_tree().quit(2)
		return
	var wd := Timer.new()
	wd.wait_time = 500.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("VE5 WATCHDOG TIMEOUT")
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
	_root = OS.get_environment("TEMP").replace("\\", "/") + "/cartalith-ve5-%s-%d" % ["touch" if _touch else "desk", Time.get_ticks_msec()]
	var filler := ""
	for i in 70:
		filler += "Filler line %d of the history, long enough to be a line.\n" % i
	_write(_root + "/Q0first.md", "# Q0first\n")  # sorts first so selecting Rhosk is a real selection
	_write(_root + "/Rhosk.md", "---\ntitle: Rhosk, the Grey\nstatus: draft\ntags:\n  - town\n  - port\naliases: [Old Rhosk]\n---\n# Rhosk\n\nIntro line.\n\n## History\n\n" + filler + "\n## Trade\n\nSalt and grain.\n\n### Salt\n\nA grey crust.\n\nBack to [[#Trade]] now.\n")
	_write(_root + "/Linker.md", "# Linker\n\nfirst\nSee [[Rhosk]] here.\nand [[Rhosk#Trade]] too\n")
	_write(_root + "/Linker2.md", "# Linker2\n\n\nx\n\nA [markdown link](Rhosk.md) here.\n")
	_write(_root + "/Mentioner.md", "# Mentioner\n\nPara.\nThe old port of Rhosk is on the river.\nRhosk again later.\n")
	_write(_root + "/Mentioner2.md", "# Mentioner2\n\nthe city of rhosk grows\n")
	_write(_root + "/Mentioner3.md", "# Mentioner3\n\nRhosk, read only.\n")
	_write(_root + "/Tagged1.md", "A #town and a #river.\n#town again\n")
	_write(_root + "/Tagged2.md", "---\ntags: [River]\n---\nbody\n")
	var dup_fill := ""
	for i in 60:
		dup_fill += "- Part filler %d
" % i
	_write(_root + "/Dups.md", "# Dups

## Part

first part

" + dup_fill + "
## Part

second part

" + dup_fill + "
end
")
	_write(_root + "/Bad.md", "---\na: 1\nnot a pair\nb: 2\n---\n# Bad\n")
	_write(_root + "/A/Twin.md", "# Twin A\n")
	_write(_root + "/B/Twin.md", "# Twin B\n")
	var conn: Dictionary = _bridge.vault_connect(_root, "Chronicle")
	_ok("CONTROL: the fixture vault binds", bool(conn.get("ok", false)), str(conn))
	_bridge.vault_refresh_backlinks(2000)
	app.open_vault_browse()
	await _frames(4)
	vw = app.vault_window
	var ts: Array = []
	_find_all(vw, "Tree", ts)
	var tree: Tree = ts[0] if not ts.is_empty() else null
	var leaf := _leaf_in(tree.get_root(), "Rhosk.md") if tree != null else null
	_ok("CONTROL: Rhosk.md is in the tree", leaf != null)
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
	await _press(open_btn)
	await _frames(4)
	_grab()
	_ok("the note opens in Write with a side panel built", ed != null and ed.mode() == "write" and sp != null)
	if sp == null:
		_finish()
		return
	_phone = ed._phone
	await _shot("note_first")
	print("VE5 info: viewport %s, phone=%s, touch=%s, window %s" % [_vp.size, _phone, _touch, vw.size])

	# == 0. Where the panel sits and whether it can be toggled =================
	## A literal, not `vw.SIDE_RIGHT_MIN_W`: asserting a constant against itself passes whatever it is set to.
	var wide: bool = (not _phone) and vw.size.x >= 1100
	if wide:
		_ok("wide window: the panel is open and to the RIGHT of the editor", sp.visible and sp.get_global_rect().position.x >= ed.get_global_rect().end.x - 2.0, "%s vs %s" % [sp.get_global_rect(), ed.get_global_rect()])
	else:
		_ok("narrow window or phone: the panel starts closed", not sp.visible)
	var toggle: Button = vw._side_toggle
	_ok("the header has a panel toggle", toggle != null and is_instance_valid(toggle))
	if toggle != null:
		var was: bool = sp.visible
		await _press(toggle)
		_grab()
		_ok("the toggle flips the panel", sp.visible != was)
		if _phone:
			_ok("phone: the open panel REPLACES the editor (it would not fit under it)", sp.visible and not ed.visible)
			await _press(_named("SideClose"))
			_ok("phone: the Note button brings the editor back", not sp.visible and ed.visible)
			vw._set_side(true)
			vw._rebuild()
			await _frames(5)
			_grab()
			_ok("phone: a rebuild while the sheet is open keeps the sheet and keeps the editor hidden", sp.visible and not ed.visible)
			vw._set_side(false)
			await _frames(3)
			_grab()
		else:
			await _press(vw._side_toggle)
			_grab()
			_ok("... and flips it back", sp.visible == was)
	if not sp.visible:
		await _press(vw._side_toggle)
		_grab()
		_ok("CONTROL: the panel is open for the rest of the run", sp.visible)
	if _phone:
		for k in ["links", "outline", "props", "tags"]:
			var tb := _named("SideTab_" + k)
			_ok("phone: tab %s is at least 44 px tall" % k, tb != null and tb.size.y >= 43.5, str(tb.size if tb != null else null))

	# == 1. Links =============================================================
	await _tab("links")
	await _shot("links_open")
	var back_rows := ["Linker.md", "Linker2.md"]
	for r in back_rows:
		_ok("Links lists the backlink %s" % r, _named("Backlink_" + r) != null)
	_ok("Links lists no non-linking note as a backlink", _named("Backlink_Mentioner.md") == null and _named("Backlink_Tagged1.md") == null)
	_ok("Links lists the unlinked mentions", _named("Mention_Mentioner.md") != null and _named("Mention_Mentioner2.md") != null and _named("Mention_Mentioner3.md") != null)
	_ok("... and a backlinking note is not also an unlinked mention", _named("Mention_Linker.md") == null)
	_ok("pure: backlink_line reads the wiki link's line", sp.backlink_line("Linker.md", "Rhosk.md") == 3, str(sp.backlink_line("Linker.md", "Rhosk.md")))
	_ok("pure: backlink_line reads a Markdown link's line", sp.backlink_line("Linker2.md", "Rhosk.md") == 5, str(sp.backlink_line("Linker2.md", "Rhosk.md")))
	_ok("pure: mention_line", sp.mention_line("Mentioner.md") == 3, str(sp.mention_line("Mentioner.md")))
	_ok("pure: mention_name derives the stem with - and _ as spaces", sp.mention_name("Places/Kel-Var_Landing.md") == "Kel Var Landing")
	_ok("pure: link_target is the bare name when it resolves to this note", sp.link_target("Linker.md", "Rhosk.md") == "Rhosk")
	_ok("pure: link_target is the path when the name is a twin", sp.link_target("Linker.md", "A/Twin.md") == "A/Twin", sp.link_target("Linker.md", "A/Twin.md"))

	# every listed backlink opens its note at the link's line
	var expect := {"Linker.md": 3, "Linker2.md": 5}
	for r in back_rows:
		await _tab("links")
		await _press(_named("Backlink_" + r))
		_grab()
		await _frames(6)
		_ok("clicking backlink %s opens that note" % r, vw._browse_path == r, vw._browse_path)
		_ok("... with the caret on the link's line %d" % expect[r], te != null and te.get_caret_line() == expect[r], str(te.get_caret_line() if te != null else -1))
		_ok("... and that line is on screen", te != null and te.get_first_visible_line() <= expect[r] and te.get_last_full_visible_line() >= expect[r])
		await _open("Rhosk.md")
	# every listed mention opens its note at its line
	var mexpect := {"Mentioner.md": 3, "Mentioner2.md": 2, "Mentioner3.md": 2}
	for r in mexpect:
		await _tab("links")
		await _press(_named("Mention_" + r))
		_grab()
		await _frames(6)
		_ok("clicking mention %s opens that note" % r, vw._browse_path == r, vw._browse_path)
		_ok("... with the caret on the mention's line %d" % mexpect[r], te != null and te.get_caret_line() == mexpect[r], str(te.get_caret_line() if te != null else -1))
		await _open("Rhosk.md")

	# Link: writes only the mention, through the guard
	await _tab("links")
	var m_before := _disk("Mentioner.md")
	await _press(_named("MentionLink_Mentioner.md"))
	var m_after := _disk("Mentioner.md")
	_ok("Link writes [[Rhosk]] over the first mention and nothing else", m_after == m_before.replace("port of Rhosk is", "port of [[Rhosk]] is"), m_after)
	_ok("... the later 'Rhosk again' is untouched (one mention per press)", m_after.contains("\nRhosk again later.\n"))
	await _tab("links")
	_ok("... and Mentioner is now a backlink, no longer a mention", _named("Backlink_Mentioner.md") != null and _named("Mention_Mentioner.md") == null)
	var m2_before := _disk("Mentioner2.md")
	await _press(_named("MentionLink_Mentioner2.md"))
	_ok("a mention in another case is written as [[Rhosk|rhosk]]", _disk("Mentioner2.md") == m2_before.replace("city of rhosk grows", "city of [[Rhosk|rhosk]] grows"), _disk("Mentioner2.md"))
	var ro := _root + "/Mentioner3.md"
	FileAccess.set_read_only_attribute(ro, true)
	_ro_paths.append(ro)
	await _tab("links")
	var m3_before := _disk("Mentioner3.md")
	await _press(_named("MentionLink_Mentioner3.md"))
	_ok("Link on a read-only note is refused and the note is untouched", _disk("Mentioner3.md") == m3_before, _disk("Mentioner3.md"))
	FileAccess.set_read_only_attribute(ro, false)
	_ro_paths.clear()

	# a save in the editor refreshes the index with no Refresh press
	await _open("Mentioner3.md")
	te.grab_focus()
	te.set_caret_line(te.get_line_count() - 1)
	te.set_caret_column(te.get_line(te.get_line_count() - 1).length())
	await _type("See [[Tagged2]] also.")
	var ms := await _until(func(): return _disk("Mentioner3.md").contains("[[Tagged2]]"), 3000)
	_ok("the autosave wrote the new link (%d ms)" % ms, ms >= 0)
	await _frames(4)
	await _open("Tagged2.md")
	await _tab("links")
	_ok("... and the linked note lists the new backlink without a manual refresh", _named("Backlink_Mentioner3.md") != null)
	await _open("Rhosk.md")

	# == 2. Outline ===========================================================
	await _tab("outline")
	var heads: Array = _bridge.vault_note_headings(te.text)
	_ok("pure: the crate sees the four headings", heads.size() == 4, str(heads.size()))
	var by_text := {}
	for h in heads:
		by_text[String((h as Dictionary)["text"])] = int((h as Dictionary)["line"])
	for t in ["Rhosk", "History", "Trade", "Salt"]:
		_ok("Outline lists %s" % t, _named("OutlineJump_%d" % int(by_text.get(t, -1))) != null)
	var salt_pad := _named("Outline_%d" % int(by_text["Salt"]))
	var trade_pad := _named("Outline_%d" % int(by_text["Trade"]))
	_ok("a level-3 heading is indented deeper than a level-2", salt_pad != null and trade_pad != null and (salt_pad.get_child(0) as Control).custom_minimum_size.x > (trade_pad.get_child(0) as Control).custom_minimum_size.x)
	_ok("CONTROL: Trade is far below the fold in Write", int(by_text["Trade"]) > 50)
	await _press(_named("OutlineJump_%d" % int(by_text["Trade"])))
	_grab()
	await _frames(6)
	_ok("Write: the caret is on Trade's line", te.get_caret_line() == int(by_text["Trade"]), str(te.get_caret_line()))
	_ok("... and the line is on screen", te.get_first_visible_line() <= int(by_text["Trade"]) and te.get_last_full_visible_line() >= int(by_text["Trade"]), "%d..%d" % [te.get_first_visible_line(), te.get_last_full_visible_line()])
	await _tab("outline")
	await _press(_named("OutlineJump_%d" % int(by_text["Rhosk"])))
	_ok("... and jumping back to the top works", te.get_caret_line() == int(by_text["Rhosk"]) and te.get_first_visible_line() <= int(by_text["Rhosk"]))
	# Preview
	ed.set_mode("preview")
	await _frames(6)
	var sb: ScrollBar = ed.preview.get_v_scroll_bar()
	var top0 := sb.value
	await _tab("outline")
	await _press(_named("OutlineJump_%d" % int(by_text["Trade"])))
	await _frames(6)
	_ok("Preview: the jump scrolls the preview down (%s -> %s)" % [top0, sb.value], sb.value > top0 + 100.0)
	var before_salt := sb.value
	await _tab("outline")
	await _press(_named("OutlineJump_%d" % int(by_text["Rhosk"])))
	await _frames(6)
	_ok("Preview: jumping to the first heading scrolls back up", sb.value < before_salt)
	ed.set_mode("write")
	await _frames(4)
	# two headings of one name: the preview jumps to the RIGHT one
	await _open("Dups.md")
	var dh: Array = _bridge.vault_note_headings(te.text)
	var parts_lines: Array = []
	for h in dh:
		if String((h as Dictionary)["text"]) == "Part":
			parts_lines.append(int((h as Dictionary)["line"]))
	_ok("CONTROL: Dups.md has two headings called Part", parts_lines.size() == 2, str(parts_lines))
	ed.set_mode("preview")
	await _frames(6)
	var dsb: ScrollBar = ed.preview.get_v_scroll_bar()
	await _tab("outline")
	await _press(_named("OutlineJump_%d" % int(parts_lines[0])))
	await _frames(6)
	var first_v := dsb.value
	await _tab("outline")
	await _press(_named("OutlineJump_%d" % int(parts_lines[1])))
	await _frames(6)
	_ok("Preview: the SECOND of two same-named headings scrolls further than the first (%s -> %s)" % [first_v, dsb.value], dsb.value > first_v + 200.0)
	ed.set_mode("write")
	await _frames(3)
	await _open("Rhosk.md")
	# the note's own [[#Trade]] link
	var ln := -1
	for i in te.get_line_count():
		if te.get_line(i).contains("[[#Trade]]"):
			ln = i
	_ok("CONTROL: the fixture has a [[#Trade]] link", ln >= 0)
	te.set_caret_line(0)
	te.set_caret_column(0)
	await _frames(2)
	vw._follow_link("Rhosk.md", "#Trade", ed)
	await _frames(6)
	_ok("[[#Trade]] in the same note jumps to Trade", te.get_caret_line() == int(by_text["Trade"]), str(te.get_caret_line()))
	vw._follow_link("Rhosk.md", "#No such heading", ed)
	await _frames(3)
	_ok("[[#missing]] does not move the caret", te.get_caret_line() == int(by_text["Trade"]))

	# == 3. Properties ========================================================
	await _tab("props")
	var f_status := _named("PropValue_status") as LineEdit
	_ok("Properties shows a scalar field (status = draft)", f_status != null and f_status.text == "draft")
	var f_title := _named("PropValue_title") as LineEdit
	_ok("... a scalar with a comma is read whole", f_title != null and f_title.text == "Rhosk, the Grey", f_title.text if f_title != null else "")
	_ok("... a list shows its items (tags: town, port)", _named("PropItem_tags_0") != null and _named("PropItem_tags_1") != null and (_named("PropItem_tags_0") as Button).text.begins_with("town"))
	_ok("... an inline list too (aliases: Old Rhosk)", _named("PropItem_aliases_0") != null)
	var t0 := te.text
	var pure: Dictionary = _bridge.vault_frontmatter_set(t0, "status", "review")
	_ok("pure: the bridge writes a String as one scalar", bool(pure.get("ok", false)) and String(pure.get("text", "")) == t0.replace("status: draft", "status: review"), str(pure.get("text", pure)).substr(0, 100))
	# a scalar edit, typed
	f_status.grab_focus()
	f_status.select_all()
	await _frames(2)
	await _type("review")
	await _key(KEY_ENTER)
	await _frames(6)
	var t1 := te.text
	var l0 := _lines(t0)
	var l1 := _lines(t1)
	var diff := 0
	var diff_line := -1
	for i in l0.size():
		if i < l1.size() and l0[i] != l1[i]:
			diff += 1
			diff_line = i
	_ok("a scalar edit changes exactly one line", l0.size() == l1.size() and diff == 1, "%d lines differ" % diff)
	_ok("... that line is status: review", diff_line >= 0 and l1[diff_line] == "status: review", l1[diff_line] if diff_line >= 0 else "")
	_ok("... and every other line is byte-identical", t1 == t0.replace("status: draft", "status: review"))
	# one undo step
	await _to_note()
	te.grab_focus()
	await _key(KEY_Z, true)
	_ok("Ctrl+Z undoes the whole property edit in one step", te.text == t0, te.text.substr(0, 120))
	# list add and remove
	await _tab("props")
	await _tab("props")
	var add_tags := _named("PropAdd_tags") as LineEdit
	add_tags.grab_focus()
	await _frames(2)
	await _type("harbour")
	await _key(KEY_ENTER)
	await _frames(6)
	_ok("adding a list item appends it as a block-list line", te.text == t0.replace("  - port\n", "  - port\n  - harbour\n"), te.text.substr(0, 160))
	await _tab("props")
	await _press(_named("PropItem_tags_0"))
	await _frames(4)
	_ok("tapping a list item removes it and keeps the rest byte-identical", te.text == t0.replace("  - port\n", "  - port\n  - harbour\n").replace("  - town\n", ""), te.text.substr(0, 160))
	# new property
	await _tab("props")
	var nk := _named("PropNewKey") as LineEdit
	var nv := _named("PropNewValue") as LineEdit
	nk.grab_focus()
	await _frames(2)
	await _type("region")
	nv.grab_focus()
	await _frames(2)
	await _type("Marches")
	await _press(_named("PropNewAdd"))
	await _frames(4)
	_ok("a new property is added at the end of the block", te.text.contains("aliases: [Old Rhosk]\nregion: Marches\n---\n"), te.text.substr(0, 220))
	# remove
	await _tab("props")
	await _press(_named("PropRemove_region"))
	await _frames(4)
	_ok("removing a property removes just its line", not te.text.contains("region:") and te.text.contains("aliases: [Old Rhosk]\n---\n"))
	# the autosave writes it, with the body untouched
	var d_ms := await _until(func(): return _disk("Rhosk.md") == te.text, 4000)
	_ok("the autosave wrote the edited properties (%d ms)" % d_ms, d_ms >= 0)
	var body_start := t0.find("# Rhosk")
	_ok("... and the note's body is byte-identical to before", _disk("Rhosk.md").substr(_disk("Rhosk.md").find("# Rhosk")) == t0.substr(body_start))
	# restore the fixture
	te.text = t0
	vw._on_editor_edited(ed)
	await _until(func(): return _disk("Rhosk.md") == t0, 4000)
	# a note without a block
	await _open("Linker.md")
	await _tab("props")
	_ok("a note with no properties says so and offers the add row", _named("PropNewKey") != null)
	var lk0 := te.text
	nk = _named("PropNewKey") as LineEdit
	nv = _named("PropNewValue") as LineEdit
	nk.grab_focus()
	await _frames(2)
	await _type("status")
	nv.grab_focus()
	await _frames(2)
	await _type("new")
	await _press(_named("PropNewAdd"))
	await _frames(4)
	_ok("adding the first property creates the block at the top", te.text == "---\nstatus: new\n---\n" + lk0, te.text.substr(0, 80))
	# unreadable properties
	await _open("Bad.md")
	await _tab("props")
	_ok("a note whose properties cannot be read safely is not editable", _named("PropNewKey") == null and (_named("PropValue_a") == null or not (_named("PropValue_a") as LineEdit).editable))

	await _tab("props")
	await _shot("props")
	await _tab("outline")
	await _shot("outline")
	# == 4. Tags ==============================================================
	await _open("Rhosk.md")
	await _tab("tags")
	var ix: Dictionary = _bridge.vault_tag_index(2000)
	_ok("pure: the crate's index counts notes (town: Rhosk and Tagged1 = 2)", _tag_count(ix, "town") == 2, str(_tag_count(ix, "town")))
	_ok("pure: river counts Tagged1 and Tagged2 once each = 2 (case-folded)", _tag_count(ix, "river") == 2)
	_ok("Tags lists town with its note count", _named("Tag_town") != null and (_named("Tag_town") as Button).text.ends_with("2"), (_named("Tag_town") as Button).text if _named("Tag_town") != null else "")
	_ok("... and no note list is open yet", _named("TagNote_Tagged1.md") == null)
	await _press(_named("Tag_town"))
	_ok("tapping a tag lists the notes that carry it", _named("TagNote_Tagged1.md") != null and _named("TagNote_Rhosk.md") != null)
	await _press(_named("TagNote_Tagged1.md"))
	_grab()
	await _frames(5)
	_ok("... and a listed note opens", vw._browse_path == "Tagged1.md", vw._browse_path)
	await _tab("tags")
	sp._tags = _bridge.vault_tag_index(1)
	sp.rebuild()
	await _frames(2)
	var partial := false
	var labels: Array = []
	_find_all(sp, "Label", labels)
	for l in labels:
		if (l as Label).text.begins_with("Partial: 1 of"):
			partial = true
	_ok("a capped scan says it is partial", partial)
	sp._tags = {}
	sp.rebuild()

	await _shot("tags")
	# a tag typed into a note appears in the index after the save, with no Rescan
	await _open("Tagged2.md")
	await _tab("tags")  ## scanned and cached BEFORE the edit, so a stale cache would show
	await _to_note()
	te.grab_focus()
	te.set_caret_line(te.get_line_count() - 1)
	te.set_caret_column(te.get_line(te.get_line_count() - 1).length())
	await _type(" #brandnew")
	var tms := await _until(func(): return _disk("Tagged2.md").contains("#brandnew"), 3000)
	_ok("the autosave wrote the new tag (%d ms)" % tms, tms >= 0)
	await _frames(6)
	await _tab("tags")
	_ok("a tag written in a note shows in the Tags tab after the save, with no Rescan", _named("Tag_brandnew") != null)
	# == 5. A narrow window puts the panel BELOW the editor ====================
	if not _phone:
		await _open("Rhosk.md")
		vw.size = Vector2i(1000, 800)
		await _frames(4)
		vw._rebuild()
		await _frames(6)
		_grab()
		var nar := vw.size.x < 1100
		_ok("CONTROL: the window is narrow now (%d)" % vw.size.x, nar)
		if nar and sp != null and not sp.visible:
			await _press(vw._side_toggle)
			_grab()
		_ok("narrow: the panel opens BELOW the editor", sp != null and sp.visible and sp.get_global_rect().position.y >= ed.get_global_rect().end.y - 2.0, "%s vs %s" % [sp.get_global_rect() if sp != null else null, ed.get_global_rect()])
	else:
		await _open("Rhosk.md")
		await _tab("links")
		_ok("phone: the panel fills the sheet and the editor is hidden", sp.visible and not ed.visible and sp.size.x >= 300.0, "%s" % sp.size)
		for b in ["Backlink_Linker.md"]:
			await _tab("links")
			var bb := _named(b)
			_ok("phone: %s is at least 44 px tall" % b, bb != null and bb.size.y >= 43.5, str(bb.size if bb != null else null))
	if _shots != "":
		DirAccess.make_dir_recursive_absolute(_shots)
		await _frames(3)
		_vp.get_texture().get_image().save_png(_shots + "/ve5_%s.png" % ("phone" if _phone else ("touch" if _touch else "desk")))
	_finish()


## The number of notes carrying `tag` in a `vault_tag_index` answer (case-insensitive), -1 if absent.
func _tag_count(ix: Dictionary, tag: String) -> int:
	for t in ix.get("tags", []):
		if String((t as Dictionary)["tag"]).to_lower() == tag.to_lower():
			return (t as Dictionary)["notes"].size()
	return -1
