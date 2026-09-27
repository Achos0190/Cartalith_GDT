extends Node
## Owner Ruling BE (`LARGE_ITEM_RULINGS.md`, 2026-09-27): one vault window for
## browsing and for an entity, with an Attached-to block and a functional
## Markdown editor. Drives the real app, the real vault window and a real
## vault folder on disk; every file assertion re-reads the file from disk.
## Grew out of `_vaultshow_probe.gd` (the owner's before-screenshots), which is
## left as it was: it pictures the two windows this ruling merged.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _vaultbe_probe.tscn -- --vp 1600x1000 --shots <dir>
##   Godot_v4.7.1-stable_win64_console.exe --path . _vaultbe_probe.tscn -- --vp 1080x2340 --force-touch --shots <dir>
##
## **Windowed, not `--headless`**: it saves screenshots, and the pixel path is
## a no-op under the dummy driver (`MISTAKES.md`). Flags it reads, grepped from
## the body below: `--vp WxH` (host viewport, default 1600x1000),
## `--force-touch` (read by the shell's layout probe; checked here against
## `DccTheme.is_phone()`), `--shots DIR` (where PNGs go; none when absent). Any
## other flag aborts.
##
## Exit 0 = every check held, 1 = a check failed, 2 = could not run. The
## RESULT line carries the check count.

const SEED := 552017

var _vp: SubViewport
var app: Node
var _bridge
var _root := ""
var _shots := ""
var _phone := false
var _n := 0
var _fails: Array = []


func _ok(label: String, cond: bool, detail: String = "") -> void:
	_n += 1
	if cond:
		print("VAULTBE  ok  %s" % label)
	else:
		_fails.append(label)
		print("VAULTBE  !!  %s   %s" % [label, detail])


func _arg(name: String, dflt: String) -> String:
	var args := OS.get_cmdline_user_args()
	var i := args.find(name)
	if i >= 0 and i + 1 < args.size():
		return String(args[i + 1])
	return dflt


func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame


func _write(path: String, text: String) -> void:
	var f := FileAccess.open(path, FileAccess.WRITE)
	f.store_string(text)
	f.close()


func _disk(rel: String) -> String:
	return FileAccess.get_file_as_string(_root + "/" + rel)


func _shot(name: String) -> void:
	await _frames(6)
	await RenderingServer.frame_post_draw
	await get_tree().process_frame
	await RenderingServer.frame_post_draw
	if _shots == "":
		return
	var tag := "phone_" if _phone else ""
	_vp.get_texture().get_image().save_png(_shots + "/" + tag + name)
	print("VAULTBE  shot %s/%s%s" % [_shots, tag, name])


func _find_named(n: Node, target: String) -> Node:
	if n.name == target:
		return n
	for c in n.get_children():
		var r := _find_named(c, target)
		if r != null:
			return r
	return null


## Case-blind: the phone fit sets action labels in capitals.
func _find_button(n: Node, text: String) -> Button:
	if n is Button and String((n as Button).text).to_lower() == text.to_lower():
		return n as Button
	for c in n.get_children():
		var r := _find_button(c, text)
		if r != null:
			return r
	return null


func _find_all(n: Node, cls: String, out: Array) -> void:
	if n.is_class(cls):
		out.append(n)
	for c in n.get_children():
		_find_all(c, cls, out)


func _labels(n: Node) -> String:
	var out: Array = []
	var ls: Array = []
	_find_all(n, "Label", ls)
	for l in ls:
		out.append(String((l as Label).text))
	return "\n".join(PackedStringArray(out))


func _leaf(tree: Tree, rel: String) -> TreeItem:
	return _leaf_in(tree.get_root(), rel)


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


func _tree(vw: Node) -> Tree:
	var ts: Array = []
	_find_all(vw, "Tree", ts)
	return ts[0] as Tree if not ts.is_empty() else null


## A real left click at a control's centre, through the host viewport -- the
## embedded window routes it like any mouse input. Desktop only: a synthetic
## tap into an embedded window at phone content scale does not route
## (`MISTAKES.md`, "plan a phone probe that must reach a DIALOG").
func _click(vw: Window, c: Control) -> void:
	var p := Vector2(vw.position) + c.get_global_rect().get_center()
	for pressed in [true, false]:
		var ev := InputEventMouseButton.new()
		ev.button_index = MOUSE_BUTTON_LEFT
		ev.pressed = pressed
		ev.position = p
		ev.global_position = p
		_vp.push_input(ev)
		await _frames(2)


func _press(vw: Window, b: Button) -> void:
	if _phone:
		b.pressed.emit()
		await _frames(3)
	else:
		await _click(vw, b)


func _confirm_dialogs() -> Array:
	var out: Array = []
	_find_all(app, "ConfirmationDialog", out)
	var vis: Array = []
	for d in out:
		if (d as Window).visible:
			vis.append(d)
	return vis


func _ready() -> void:
	for a in OS.get_cmdline_user_args():
		var s := String(a)
		if s.begins_with("--") and not (s in ["--vp", "--force-touch", "--shots"]):
			print("VAULTBE ABORT unknown flag %s" % s)
			get_tree().quit(2)
			return
	if DisplayServer.get_name() == "headless":
		print("VAULTBE REFUSED: run windowed")
		get_tree().quit(2)
		return
	var parts: PackedStringArray = _arg("--vp", "1600x1000").split("x")
	_shots = _arg("--shots", "")
	if _shots != "":
		DirAccess.make_dir_recursive_absolute(_shots)
	var want_touch := "--force-touch" in OS.get_cmdline_user_args()
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
	_bridge = app.bridge
	_phone = DccTheme.is_phone()
	_ok("density matches the flag (force-touch <-> phone)", _phone == want_touch)

	_bridge.generate({"seed": SEED, "width_km": 2400.0, "grid_w": 384, "grid_h": 288,
		"archetype": "", "villages": true, "sea_level": 0.45})
	while _bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await _frames(4)

	var ss: Array = _bridge.settlements()
	_ok("CONTROL: the world has at least two settlements", ss.size() >= 2, str(ss.size()))
	if ss.size() < 2:
		_finish()
		return
	var s0: Dictionary = ss[0]
	var s1: Dictionary = ss[1]
	var tid := int(s0.get("tid", -1))
	var nm := String(s0.get("name", "?"))
	var tid1 := int(s1.get("tid", -1))
	var nm1 := String(s1.get("name", "?"))
	## The faction's roster name, read from the roster -- not from the window.
	var fac_name := ""
	for f in _bridge.get_factions():
		if int((f as Dictionary).get("id", -1)) == int(s0.get("faction", 0)):
			fac_name = String((f as Dictionary).get("name", ""))
	print("VAULTBE settlement tid=%d name=%s faction=%s" % [tid, nm, fac_name])

	_root = OS.get_environment("TEMP").replace("\\", "/") + ("/cartalith-vaultbe-%s" % ("phone" if _phone else "desk"))
	DirAccess.make_dir_recursive_absolute(_root + "/Settlements")
	DirAccess.make_dir_recursive_absolute(_root + "/Events")
	var rel := "Settlements/%s.md" % nm
	_write(_root + "/" + rel,
		"---\ntype: settlement\ntid: %d\nfounded: 212\nruler: Hale Amaret\n---\n\n# Founding legend\n\nThe old song claims the ford chose its own name. The first wardens kept a tally of every raft that crossed.\n\n## Customs Gate\n\nA river crossing grown into a customs town. Salt barges pay by the hand-width of their draught.\n\n- **Size / Population:**\n\n## Notable people\n\n- Hale Amaret, reeve since 391\n- Orsa of the Weir, toll-keeper\n" % tid)
	_write(_root + "/Events/Salt-Road.md",
		"---\ntype: event\nyear: 398\n---\n\nThree names on the customs-gate list of [[%s]].\n" % nm)
	var conn: Dictionary = _bridge.vault_connect(_root, "Chronicle")
	_ok("CONTROL: the fixture vault binds", bool(conn.get("ok", false)), str(conn))

	# == 1. Browse: the tree, the frontmatter match, no link yet ==============
	app.open_vault_browse()
	await _frames(4)
	var vw: VaultWindow = app.vault_window
	if _phone:
		## The tree pane is what a phone opens on.
		var tree0 := _tree(vw)
		_ok("phone: browse opens on the FILES pane with a tree", tree0 != null)
	var tree := _tree(vw)
	var leaf := _leaf(tree, rel) if tree != null else null
	_ok("the note is in the tree", leaf != null)
	if leaf == null:
		_finish()
		return
	_ok("CONTROL: an unattached note carries no link marker", leaf.get_icon(0) == null)
	await _shot("1_browse_tree.png")
	leaf.select(0)          ## emits item_selected, the tree's own signal
	await _frames(4)
	_ok("picking the tree item picks the note", vw._pick_file == rel, vw._pick_file)
	var fm_row := _find_named(vw, "FrontmatterRow")
	_ok("frontmatter tid resolves to a real settlement, shown as not attached",
		fm_row != null and _labels(fm_row).find(nm) >= 0 and _labels(fm_row).find("by frontmatter, not attached") >= 0,
		_labels(fm_row) if fm_row != null else "<no row>")
	_ok("no Attached row yet", _find_named(vw, "AttachedRow") == null)
	var rendered := _find_named(vw, "VaultNoteRendered") as RichTextLabel
	_ok("the note is rendered, heading as a heading and not as '#'",
		rendered != null and rendered.text.find("[b]Founding legend[/b]") >= 0 and rendered.text.find("# Founding") < 0,
		rendered.text.substr(0, 200) if rendered != null else "<none>")
	await _shot("2_browse_note.png")

	# == 2. Attach through the frontmatter row's own button ===================
	var attach_btn := _find_button(fm_row, "Attach")
	_ok("the frontmatter row offers Attach", attach_btn != null)
	if attach_btn != null:
		await _press(vw, attach_btn)
		await _frames(4)
	var links: Array = _bridge.vault_links_for("settlement", tid)
	_ok("the store now holds exactly one link for that settlement, to this note",
		links.size() == 1 and String((links[0] as Dictionary).get("path", "")) == rel, str(links))
	var row := _find_named(vw, "AttachedRow")
	var row_text := _labels(row) if row != null else ""
	_ok("Attached to names the bridge's own settlement (name from bridge.settlements())",
		row != null and row_text.find(nm) >= 0 and String(row.get_meta("entity", "")) == "settlement:%d" % tid, row_text)
	_ok("... with its kind and its faction's roster name",
		row_text.find("SETTLEMENT") >= 0 and fac_name != "" and row_text.find(fac_name) >= 0, row_text)
	_ok("the frontmatter row is gone once it is a link", _find_named(vw, "FrontmatterRow") == null)
	var show := _find_button(row, "Show on map") if row != null else null
	var pe := _find_button(row, "Open place editor") if row != null else null
	var det := _find_button(row, "Detach") if row != null else null
	_ok("the row offers Show on map, Open place editor and Detach, all enabled",
		show != null and pe != null and det != null and not show.disabled and not pe.disabled and not det.disabled)
	await _shot("3_attached_to.png")
	if _phone:
		## The phone shows one pane: go back to FILES the way a user does.
		var files_btn := _find_button(vw, "FILES")
		if files_btn != null:
			files_btn.pressed.emit()
			await _frames(4)
	tree = _tree(vw)
	var leaf2 := _leaf(tree, rel) if tree != null else null
	_ok("the tree now marks the note as attached", leaf2 != null and leaf2.get_icon(0) != null
		and leaf2.get_tooltip_text(0).find(nm) >= 0)
	var other := _leaf(tree, "Events/Salt-Road.md") if tree != null else null
	_ok("CONTROL: the unattached note stays unmarked", other != null and other.get_icon(0) == null)
	await _shot("3b_tree_marked.png")
	vw.hide()
	await _frames(2)

	# == 3. open_for preselects the entity's note; an entity with none ========
	app.open_vault("settlement", tid, nm)
	await _frames(6)
	_ok("open_for preselects that entity's attached note", vw._pick_file == rel, vw._pick_file)
	_ok("... and the note pane shows it attached", _find_named(vw, "AttachedRow") != null)
	if not _phone:
		var t3 := _tree(vw)
		var sel := t3.get_selected() if t3 != null else null
		_ok("... and the tree highlights it", sel != null and String(sel.get_metadata(0)) == rel)
	vw.hide()
	await _frames(2)
	app.open_vault("settlement", tid1, nm1)
	await _frames(6)
	_ok("an entity with no note opens on its attach flow", vw._pick_file == "" and _find_named(vw, "VaultAttachFlow") != null,
		"pick=%s" % vw._pick_file)
	await _shot("4_entity_without_note.png")
	vw.hide()
	await _frames(2)
	app.open_vault("settlement", tid, nm)
	await _frames(6)

	# == 4. The editor: type, bold through the toolbar, Save, re-read disk ====
	var open_btn := _find_button(vw, "Open to edit")
	_ok("the note pane offers Open to edit", open_btn != null)
	if open_btn == null:
		_finish()
		return
	await _press(vw, open_btn)
	await _frames(6)
	var ed = _find_named(vw, "VaultMarkdownEditor")
	_ok("Open to edit turns the pane into the Markdown editor", ed != null)
	if ed == null:
		_finish()
		return
	var te: TextEdit = ed.text_edit
	_ok("the editor holds exactly the file on disk", te.text == _disk(rel))
	_ok("the pane has no second form under it (no Attached to while editing)",
		_find_named(vw, "VaultAttachedTo") == null)
	var names := ["bold", "italic", "underline", "strike", "h0", "h1", "h2", "h3",
		"bullet", "number", "quote", "code", "code_block", "link", "write", "preview"]
	var missing: Array = []
	for k in names:
		if not (ed.buttons as Dictionary).has(k):
			missing.append(k)
	_ok("the toolbar carries every ruled control", missing.is_empty(), str(missing))
	## Type: caret to the end, insert a line the way a keyboard does.
	te.set_caret_line(te.get_line_count() - 1)
	te.set_caret_column(te.get_line(te.get_line_count() - 1).length())
	te.insert_text_at_caret("\nThe toll is paid in salt.")
	await _frames(2)
	_ok("typing reaches the window's buffer", vw._browse_text.ends_with("The toll is paid in salt."))
	var at := te.text.rfind("salt")
	ed.select_offsets(at, at + 4)
	await _frames(1)
	await _press(vw, ed.buttons["bold"])
	await _frames(2)
	_ok("the real Bold button wrapped the selection", te.text.ends_with("The toll is paid in **salt**."), te.text.right(40))
	if not _phone:
		## Ctrl+I through the focused text area, the shortcut path.
		var tpos := te.text.rfind("toll")
		ed.select_offsets(tpos, tpos + 4)
		te.grab_focus()
		await _frames(1)
		var k := InputEventKey.new()
		k.keycode = KEY_I
		k.ctrl_pressed = true
		k.pressed = true
		_vp.push_input(k)
		await _frames(2)
		_ok("Ctrl+I italicises the selection", te.text.ends_with("The *toll* is paid in **salt**."), te.text.right(40))
	await _shot("5_editor_write.png")
	await _press(vw, ed.buttons["preview"])
	await _frames(3)
	var pv: RichTextLabel = ed.preview
	_ok("Preview renders the bold", pv.visible and not te.visible and pv.text.find("[b]salt[/b]") >= 0, pv.text.right(80))
	await _shot("6_editor_preview.png")
	await _press(vw, ed.buttons["write"])
	await _frames(2)
	_ok("Write comes back to the source", te.visible and not pv.visible)
	await _press(vw, ed.save_button)
	await _frames(3)
	var disk := _disk(rel)
	_ok("Save wrote the bold to disk (re-read from the file)", disk.find("paid in **salt**.") >= 0, disk.right(60))
	_ok("the editor stays open and clean after Save", _find_named(vw, "VaultMarkdownEditor") != null and not vw._is_dirty())

	# == 5. The hash guard: a change behind the editor refuses Save ===========
	var behind := disk + "\nEdited in another program.\n"
	_write(_root + "/" + rel, behind)
	te.insert_text_at_caret("\nMine, typed after the other edit.")
	await _frames(2)
	await _press(vw, ed.save_button)
	await _frames(3)
	_ok("Save refuses: the file on disk is exactly the other program's version", _disk(rel) == behind)
	_ok("... the typed text survives in the editor", te.text.find("Mine, typed after the other edit.") >= 0)
	_ok("... and the editor says it was refused", String(ed.status_label.text).findn("refused") >= 0, ed.status_label.text)
	await _shot("7_editor_refused.png")

	# == 6. Closing with unsaved edits warns ==================================
	vw.hide()
	await _frames(4)
	var dlgs := _confirm_dialogs()
	var warn: ConfirmationDialog = null
	for d in dlgs:
		if String((d as ConfirmationDialog).get_ok_button().text) == "Discard and close":
			warn = d
	_ok("closing with unsaved edits brings the window back and asks", vw.visible and warn != null)
	if warn != null:
		warn.get_ok_button().pressed.emit()
		await _frames(4)
	_ok("Discard and close closes it with the edits dropped", not vw.visible and not vw._is_dirty())
	_ok("CONTROL: discarding wrote nothing", _disk(rel) == behind)

	# == 7. More: the five guarded writes are all still reachable =============
	app.open_vault("settlement", tid, nm)
	await _frames(6)
	var more := _find_named(vw, "VaultMore") as Control
	var more_hdr := _find_named(vw, "VaultMoreHeader") as Button
	_ok("More exists and starts collapsed", more != null and not more.visible)
	if more_hdr != null:
		## It sits under the note, so a user scrolls to it first -- and so must
		## a real click, or it lands on whatever the pane shows instead.
		var sc0 := _find_named(vw, "VaultNoteScroll") as ScrollContainer
		if sc0 != null:
			sc0.ensure_control_visible(more_hdr)
			await _frames(3)
		await _press(vw, more_hdr)
		await _frames(3)
	more = _find_named(vw, "VaultMore") as Control
	_ok("pressing More opens it", more != null and more.visible)
	var four := ["Insert updated section into source…", "Preview & write Cartalith block…",
		"Fill the note's own fields…", "Remove the Cartalith block…"]
	for label in four:
		var b := _find_button(more, label) if more != null else null
		_ok("More holds '%s', enabled" % label, b != null and not b.disabled)
	_ok("the fifth, Save, is the editor's (checked above)", true)
	## The screenshot shows More's contents: scroll its header to the top.
	var scroll := _find_named(vw, "VaultNoteScroll") as ScrollContainer
	var hdr2 := _find_named(vw, "VaultMoreHeader") as Control
	if scroll == null:
		scroll = vw._scroll   ## the phone's one scrolling column
	if scroll != null and hdr2 != null:
		await _frames(2)
		scroll.scroll_vertical = int(hdr2.global_position.y - scroll.get_child(0).global_position.y)
	await _shot("8_more.png")
	## Each write still stops at its preview: press two and see the dialog.
	for pair in [["Preview & write Cartalith block…", "Write Cartalith block"],
			["Insert updated section into source…", "Insert updated section"],
			["Fill the note's own fields…", "Fill the note's own fields"]]:
		var b2 := _find_button(vw, String(pair[0]))
		if b2 == null:
			_ok("preview for %s" % pair[0], false, "button gone")
			continue
		b2.pressed.emit()
		await _frames(4)
		var found: ConfirmationDialog = null
		for d in _confirm_dialogs():
			if (d as Window).title == String(pair[1]):
				found = d
		_ok("'%s' stops at its preview dialog" % pair[0], found != null)
		if found != null:
			found.get_cancel_button().pressed.emit()
			await _frames(3)
	_ok("CONTROL: cancelling the previews wrote nothing", _disk(rel) == behind)
	vw.hide()
	await _frames(2)
	_finish()


func _finish() -> void:
	## Leave no vault binding behind: the next probe to boot reads this file,
	## and a "Chronicle" left in it renamed `_vault_probe`'s ProbeVault once.
	if FileAccess.file_exists(VaultStore.PATH):
		DirAccess.remove_absolute(ProjectSettings.globalize_path(VaultStore.PATH))
	print("VAULTBE  RESULT checks=%d failed=%d %s" % [_n, _fails.size(), str(_fails)])
	get_tree().quit(0 if _fails.is_empty() else 1)
