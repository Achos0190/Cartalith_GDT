extends Node
## VE-1 of `VAULT_EDITOR_PROPOSAL.md` s.7 (VE-0's measurements are `_ve0_measure_probe.gd`): links work.
## Bar: *every link in the four sample notes followed and returned from; unresolved and entity links
## never create silently.* Drives the real app, the real vault window and a real vault folder with
## real clicks and typed keys; every file assertion re-reads the disk.
##
##   Godot_v4.7.2-stable_win64_console.exe --path . _ve0_probe.tscn -- --vp 1600x1000 [--touch] [--shots DIR]
##
## **Windowed** (pixels and input routing). Flags, all read below: `--vp WxH` (default 1600x1000),
## `--touch` (the pointer events carry the emulated-mouse device id a finger's do, and the hold-peek
## is asserted instead of the hover-peek -- the window is NOT put in phone density; VE-8 owns the
## phone), `--shots DIR`. Any other flag aborts. Run it under a wiped user directory
## (`APPDATA=<empty dir>`): it deletes `user://vault.json` and binds a throw-away vault.
## Exit 0 = every check held, 1 = a check failed, 2 = could not run.
##
## Pointer note, measured in `_ve0_measure_probe.gd`: a SubViewport routes motion only after
## `NOTIFICATION_VP_MOUSE_ENTER`; without it every hover assertion fails (or, worse, a "stays hidden"
## assertion passes) vacuously, so a positive control hovers a known link first.
##
## Result block (2026-11-08, Godot 4.7.2, this machine): see STATUS.md "VE-0 and VE-1".

const SEED := 552017

var _vp: SubViewport
var app: Node
var _bridge
var _root := ""
var _shots := ""
var _touch := false
var _n := 0
var _fails: Array = []
var _hover_meta := ""


func _ok(label: String, cond: bool, detail: String = "") -> void:
	_n += 1
	if cond:
		print("VE0  ok  %s" % label)
	else:
		_fails.append(label)
		print("VE0  !!  %s   %s" % [label, detail])


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
	DirAccess.make_dir_recursive_absolute(path.get_base_dir())
	var f := FileAccess.open(path, FileAccess.WRITE)
	f.store_string(text)
	f.close()


func _disk(rel: String) -> String:
	return FileAccess.get_file_as_string(_root + "/" + rel)


func _exists(rel: String) -> bool:
	return FileAccess.file_exists(_root + "/" + rel)


func _shot(name: String) -> void:
	await _frames(6)
	await RenderingServer.frame_post_draw
	if _shots == "":
		return
	DirAccess.make_dir_recursive_absolute(_shots)
	_vp.get_texture().get_image().save_png(_shots + "/" + ("touch_" if _touch else "desk_") + name)


func _find_named(n: Node, target: String) -> Node:
	if n.name == target:
		return n
	for c in n.get_children():
		var r := _find_named(c, target)
		if r != null:
			return r
	return null


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


func _tree(vw: Node) -> Tree:
	var ts: Array = []
	_find_all(vw, "Tree", ts)
	return ts[0] as Tree if not ts.is_empty() else null


## One pointer event at a point in the window's own coordinates, through the host viewport. A finger
## is the emulated device (what Godot gives the emulated mouse a touch becomes).
func _mouse(vw: Window, local: Vector2, pressed: bool, ctrl: bool = false) -> void:
	var p := Vector2(vw.position) + local
	var ev := InputEventMouseButton.new()
	ev.button_index = MOUSE_BUTTON_LEFT
	ev.pressed = pressed
	ev.position = p
	ev.global_position = p
	ev.ctrl_pressed = ctrl
	if _touch:
		ev.device = InputEvent.DEVICE_ID_EMULATION
	_vp.push_input(ev)
	await _frames(2)


func _tap(vw: Window, local: Vector2, ctrl: bool = false) -> void:
	await _mouse(vw, local, true, ctrl)
	await _mouse(vw, local, false, ctrl)


func _move(vw: Window, local: Vector2) -> void:
	var p := Vector2(vw.position) + local
	var mm := InputEventMouseMotion.new()
	mm.position = p
	mm.global_position = p
	mm.relative = Vector2(2, 1)
	_vp.push_input(mm)


func _centre(c: Control) -> Vector2:
	return c.get_global_rect().get_center()


func _press(vw: Window, b: Button) -> void:
	await _tap(vw, _centre(b))
	await _frames(3)


## Types `s` through real key events (focus must already be in a text field).
func _type(s: String) -> void:
	for i in s.length():
		for pressed in [true, false]:
			var k := InputEventKey.new()
			k.pressed = pressed
			k.unicode = s.unicode_at(i)
			k.keycode = s.unicode_at(i) as Key
			_vp.push_input(k)
		await _frames(1)


## Where in `label` (window-local) a link whose meta is `wiki:<name>` can be hit, found by sweeping
## the pointer over the label and reading which meta it reports -- RichTextLabel exposes no link
## rectangles. Synchronous: hover is computed inside `push_input`, so no frame is awaited per probe.
func _link_pos(vw: Window, label: RichTextLabel, meta: String) -> Vector2:
	_hover_meta = ""
	var cb := func(m): _hover_meta = str(m)
	label.meta_hover_started.connect(cb)
	var r := label.get_global_rect()
	var found := Vector2(-1, -1)
	var y := r.position.y + 6
	while y < r.end.y and found.x < 0:
		var x := r.position.x + 4
		while x < r.end.x and found.x < 0:
			_move(vw, Vector2(x, y))
			if _hover_meta == meta:
				found = Vector2(x, y)
			x += 6
		y += 5
	label.meta_hover_started.disconnect(cb)
	_move(vw, Vector2(r.end.x - 2, r.end.y - 2))
	return found


func _menu(vw: Node) -> PopupMenu:
	var m := _find_named(vw, "LinkMenu")
	return m as PopupMenu if m != null and (m as PopupMenu).visible else null


func _menu_texts(m: PopupMenu) -> String:
	var out: Array = []
	for i in m.item_count:
		out.append(m.get_item_text(i))
	return " | ".join(PackedStringArray(out))


func _menu_pick(m: PopupMenu, contains: String) -> bool:
	for i in m.item_count:
		if m.get_item_text(i).find(contains) >= 0 and not m.is_item_disabled(i):
			m.id_pressed.emit(m.get_item_id(i))
			m.hide()
			return true
	return false


func _ed(vw: VaultWindow):
	return vw._editor


func _finish() -> void:
	print("VE0 RESULT %d checks, %d failed%s" % [_n, _fails.size(), "" if _fails.is_empty() else " :: " + str(_fails)])
	get_tree().quit(0 if _fails.is_empty() else 1)


func _ready() -> void:
	for a in OS.get_cmdline_user_args():
		var s := String(a)
		if s.begins_with("--") and not (s in ["--vp", "--touch", "--shots"]):
			print("VE0 ABORT unknown flag %s" % s)
			get_tree().quit(2)
			return
	if DisplayServer.get_name() == "headless":
		print("VE0 REFUSED: run windowed")
		get_tree().quit(2)
		return
	var parts: PackedStringArray = _arg("--vp", "1600x1000").split("x")
	_shots = _arg("--shots", "")
	_touch = "--touch" in OS.get_cmdline_user_args()
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

	_bridge.generate({"seed": SEED, "width_km": 2400.0, "grid_w": 384, "grid_h": 288,
		"archetype": "", "villages": true, "sea_level": 0.45})
	while _bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await _frames(4)
	var ss: Array = _bridge.settlements()
	_ok("CONTROL: the world has settlements", ss.size() >= 2, str(ss.size()))
	if ss.size() < 2:
		_finish()
		return
	var ent_name := String((ss[0] as Dictionary).get("name", "?"))

	# == The four sample notes =================================================
	_root = OS.get_environment("TEMP").replace("\\", "/") + "/cartalith-ve0-%s" % ("touch" if _touch else "desk")
	if DirAccess.dir_exists_absolute(_root):
		OS.move_to_trash(_root)
	var kel := "---\ntype: place\n---\n# Kelvhold\n\nA hold on the river. See [[Marren]] and [[Marren#History|its history]], the [[Ghost Town]], [md](Marren.md), [out](https://example.org/x) and [[%s]].\n" % ent_name
	var mar := "# Marren\n\nA ferry town below [[Kelvhold|the hold]]. Salt barges pay by the hand-width.\n\n## History\n\nBack to [[Kelvhold#Lore]].\n"
	var ald := "---\naliases: [The Ferryman]\n---\nAldis rows between [[Marren]] and [[Twin]].\n"
	var thaw := "The thaw reached [[Kelvhold]] and [[Places/Marren#History]].\n"
	_write(_root + "/Places/Kelvhold.md", kel)
	_write(_root + "/Places/Marren.md", mar)
	_write(_root + "/People/Aldis.md", ald)
	_write(_root + "/Journal/Thaw.md", thaw)
	_write(_root + "/A/Twin.md", "Twin on the A road.\n")
	_write(_root + "/B/Twin.md", "Twin on the B road.\n")
	_write(_root + "/Templates/template-note.md", "# {{title}}\n\nCreated from a template.\n")
	var before := {}
	for rel in ["Places/Kelvhold.md", "Places/Marren.md", "People/Aldis.md", "Journal/Thaw.md"]:
		before[rel] = _disk(rel)
	var conn: Dictionary = _bridge.vault_connect(_root, "Chronicle")
	_ok("CONTROL: the fixture vault binds", bool(conn.get("ok", false)), str(conn))

	app.open_vault_browse()
	await _frames(4)
	var vw: VaultWindow = app.vault_window
	var tree := _tree(vw)
	var leaf := _leaf_in(tree.get_root(), "Places/Kelvhold.md") if tree != null else null
	_ok("CONTROL: the note is in the tree", leaf != null)
	if leaf == null:
		_finish()
		return
	leaf.select(0)
	await _frames(4)
	# The reader pane: its links are classed and followable too.
	var rendered := _find_named(vw, "VaultNoteRendered") as RichTextLabel
	_ok("reader pane: links carry wiki: metas", rendered != null and rendered.text.find("url=wiki:Marren") >= 0,
		rendered.text.substr(0, 300) if rendered != null else "<none>")
	var open_btn := _find_button(vw, "Open to edit")
	_ok("CONTROL: Open to edit is offered", open_btn != null)
	await _press(vw, open_btn)
	var ed = _ed(vw)
	_ok("a note opens in Write (Ruling DY, VQ4)", ed != null and ed.mode() == "write" and ed.text_edit.visible)
	_ok("no Back button before any link is followed", ed != null and not ed._back_button.visible)

	# == 1. Write: Ctrl/Cmd-click follows =====================================
	var te: TextEdit = ed.text_edit
	var line := -1
	var col := 0
	for i in te.get_line_count():
		var c := te.get_line(i).find("[[Marren]]")
		if c >= 0:
			line = i
			col = c + 3
	var rect := te.get_rect_at_line_column(line, col)
	var at: Vector2 = te.get_global_rect().position + Vector2(rect.position) + Vector2(3, rect.size.y * 0.5)
	await _tap(vw, at)    ## a plain click only moves the caret
	_ok("a plain click in Write follows nothing", vw._browse_path == "Places/Kelvhold.md", vw._browse_path)
	await _tap(vw, at, true)
	await _frames(4)
	_ok("Ctrl/Cmd-click on [[Marren]] in Write opens Marren, in Write",
		vw._browse_path == "Places/Marren.md" and _ed(vw) != null and _ed(vw).mode() == "write", vw._browse_path)
	ed = _ed(vw)
	_ok("... and Back is now offered", ed != null and ed._back_button.visible)
	await _press(vw, ed._back_button)
	await _frames(3)
	ed = _ed(vw)
	_ok("Back returns to Kelvhold", vw._browse_path == "Places/Kelvhold.md", vw._browse_path)
	_ok("... and Back is hidden again", ed != null and not ed._back_button.visible)

	# == 2. Preview: kinds look different, click follows ======================
	await _press(vw, ed.buttons["preview"])
	var pv: RichTextLabel = ed.preview
	var bb := pv.text
	_ok("a note link is accent and underlined", bb.find("[u][url=wiki:Marren]") >= 0, bb)
	_ok("an entity link with no note carries the glyph and no underline", bb.find("[url=wiki:%s]⌖ " % ent_name) >= 0 and bb.find("[u][url=wiki:%s]" % ent_name) < 0, bb)
	_ok("an unresolved link is dim italic", bb.find("[i][url=wiki:Ghost Town]") >= 0, bb)
	_ok("a Markdown link to a note is a note link", bb.find("[u][url=wiki:Marren]md") >= 0, bb)
	_ok("an external link is its own meta", bb.find("[url=ext:https://example.org/x]") >= 0, bb)
	await _shot("1_preview_kinds.png")
	var dir_before := _disk("Places/Kelvhold.md")

	# peek: positive control first (a known link), then the unresolved one must show none
	var pos := _link_pos(vw, pv, "wiki:Marren")
	_ok("CONTROL: the Marren link can be found by sweeping", pos.x >= 0)
	if pos.x >= 0:
		var peek := ed._peek as Control
		if _touch:
			await _mouse(vw, pos, true)
			await get_tree().create_timer(0.8).timeout
			_ok("touch: a held finger on a link shows the peek card", peek.visible and peek.modulate.a == 1.0 and peek.size.y < 260.0 and ed._peek_label.text.find("ferry town") >= 0, ed._peek_label.text)
			await _shot("2_peek_touch.png")
			await _mouse(vw, pos, false)
			await _frames(3)
			_ok("touch: lifting after a peek follows nothing and hides the card",
				not (is_instance_valid(peek) and peek.visible) and vw._browse_path == "Places/Kelvhold.md", vw._browse_path)
			if vw._browse_path != "Places/Kelvhold.md":
				## The rest of the run assumes this note is open; stop with the failure on record rather
				## than hang on a missing control (a script error inside an `await` never returns).
				_finish()
				return
		else:
			_move(vw, pos)
			await get_tree().create_timer(0.85).timeout
			_ok("a 650 ms hover on a link shows the peek card with the note's text", peek.visible and peek.modulate.a == 1.0 and peek.size.y < 260.0 and ed._peek_label.text.find("ferry town") >= 0, ed._peek_label.text)
			await _shot("2_peek_desktop.png")
			_move(vw, Vector2(5, 5))
			await _frames(3)
			_ok("moving away hides the card", not peek.visible)
	var gpos := _link_pos(vw, pv, "wiki:Ghost Town")
	_ok("CONTROL: the unresolved link can be found", gpos.x >= 0)
	if gpos.x >= 0:
		_move(vw, gpos)
		await get_tree().create_timer(0.85).timeout
		_ok("an unresolved link has no peek card", not ed._peek.visible)
		_move(vw, Vector2(5, 5))
		await _frames(2)

	# unresolved: follow only OFFERS
	if gpos.x >= 0:
		await _tap(vw, gpos)
		await _frames(4)
		var m := _menu(vw)
		_ok("following an unresolved link opens a menu", m != null)
		if m != null:
			var tx := _menu_texts(m)
			print("VE0 menu(unresolved): %s" % tx)
			_ok("... that names the missing note and offers a template", tx.find("No note called Ghost Town") >= 0 and tx.find("Create Ghost Town.md from") >= 0, tx)
			_ok("... and NOTHING was created by following it", not _exists("Ghost Town.md"))
			await _shot("3_menu_unresolved.png")
			m.hide()
			await _frames(3)
		_ok("dismissing the menu created nothing and left the note", not _exists("Ghost Town.md") and vw._browse_path == "Places/Kelvhold.md")
		await _tap(vw, gpos)
		await _frames(4)
		m = _menu(vw)
		if m != null:
			_ok("choosing Create makes the note from the template and opens it",
				_menu_pick(m, "Create Ghost Town.md from"))
			await _frames(5)
			_ok("... the file now exists with the template's text", _disk("Ghost Town.md").find("Created from a template.") >= 0, _disk("Ghost Town.md"))
			_ok("... and is open in Write with Back available", vw._browse_path == "Ghost Town.md" and _ed(vw) != null and _ed(vw).mode() == "write" and _ed(vw)._back_button.visible, vw._browse_path)
	await _press(vw, _ed(vw)._back_button)
	await _frames(3)
	ed = _ed(vw)
	_ok("Back from the new note returns to Kelvhold", vw._browse_path == "Places/Kelvhold.md")

	# entity: offers create + centre, creates nothing
	await _press(vw, ed.buttons["preview"])
	pv = ed.preview
	var epos := _link_pos(vw, pv, "wiki:%s" % ent_name)
	_ok("CONTROL: the entity link can be found", epos.x >= 0)
	if epos.x >= 0:
		var files_before: int = _bridge.vault_list_files(5000).size()
		await _tap(vw, epos)
		await _frames(4)
		var me := _menu(vw)
		_ok("following an entity link opens a menu", me != null)
		if me != null:
			var tx2 := _menu_texts(me)
			print("VE0 menu(entity): %s" % tx2)
			_ok("... that says it is a settlement with no note and offers Create and Centre on map",
				tx2.find("is a settlement with no note yet") >= 0 and tx2.find("Create ") >= 0 and tx2.find("Centre on map") >= 0, tx2)
			_ok("... and creating nothing so far", _bridge.vault_list_files(5000).size() == files_before)
			await _shot("4_menu_entity.png")
			me.hide()
			await _frames(3)

	# markdown link, followed in Preview
	var mpos := _link_pos(vw, pv, "wiki:Marren")
	await _tap(vw, mpos)
	await _frames(5)
	_ok("clicking a note link in Preview opens the note, in Write", vw._browse_path == "Places/Marren.md" and _ed(vw).mode() == "write", vw._browse_path)
	# Marren -> Kelvhold (alias link), then Back, Back stack depth
	ed = _ed(vw)
	await _press(vw, ed.buttons["preview"])
	var kpos := _link_pos(vw, ed.preview, "wiki:Kelvhold")
	_ok("CONTROL: [[Kelvhold|the hold]] found", kpos.x >= 0)
	await _tap(vw, kpos)
	await _frames(5)
	_ok("the aliased link opens Kelvhold", vw._browse_path == "Places/Kelvhold.md", vw._browse_path)
	_ok("Back trail holds two entries", vw._link_back.size() == 2, str(vw._link_back))
	await _press(vw, _ed(vw)._back_button)
	await _press(vw, _ed(vw)._back_button)
	await _frames(3)
	_ok("two Backs return to where the trail began", vw._browse_path == "Places/Kelvhold.md" and vw._link_back.is_empty(), "%s %s" % [vw._browse_path, str(vw._link_back)])

	# ambiguous: Aldis -> Twin
	vw._enter_note("People/Aldis.md")
	await _frames(5)
	ed = _ed(vw)
	await _press(vw, ed.buttons["preview"])
	var tpos := _link_pos(vw, ed.preview, "wiki:Twin")
	_ok("CONTROL: [[Twin]] found", tpos.x >= 0)
	await _tap(vw, tpos)
	await _frames(4)
	var ma := _menu(vw)
	_ok("an ambiguous name asks which note", ma != null and _menu_texts(ma).find("A/Twin.md") >= 0 and _menu_texts(ma).find("B/Twin.md") >= 0, _menu_texts(ma) if ma != null else "<no menu>")
	if ma != null:
		_menu_pick(ma, "B/Twin.md")
		await _frames(5)
		_ok("choosing opens that one", vw._browse_path == "B/Twin.md", vw._browse_path)

	# == 2a. A link to a heading of the SAME note goes nowhere new =============
	var trail := vw._link_back.size()
	_write(_root + "/Journal/Self.md", "# Intro

See [[#Intro]] above.
")
	vw._enter_note("Journal/Self.md")
	await _frames(5)
	ed = _ed(vw)
	await _press(vw, ed.buttons["preview"])
	var spos := _link_pos(vw, ed.preview, "wiki:#Intro")
	_ok("CONTROL: the same-note link can be found", spos.x >= 0)
	var ed_id: int = ed.get_instance_id()
	await _tap(vw, spos)
	await _frames(4)
	## Same editor object: a "follow" that re-opened the note would have rebuilt it.
	_ok("a same-note heading link opens no menu and leaves the note", _ed(vw) != null and _ed(vw).get_instance_id() == ed_id and _menu(vw) == null and vw._browse_path == "Journal/Self.md" and vw._link_back.size() == trail, vw._browse_path)
	vw._enter_note("Places/Kelvhold.md")
	await _frames(5)
	ed = _ed(vw)

	# == 2b. A failed lookup is NOT "no such note" =============================
	## While the engine is busy the resolver answers `{ok: false}`. Reading that as "unresolved"
	## would offer to create a note that may well exist, so the window must call it `unknown`.
	_bridge.generating = true
	var unk: Dictionary = vw._link_kind("Places/Kelvhold.md", "Marren")
	_bridge.generating = false
	_ok("a resolver that cannot answer is `unknown`, never `unresolved`", String(unk.get("kind", "")) == "unknown" and unk.has("why"), str(unk))
	_ok("... and with the engine free the same link is a note again", String(vw._link_kind("Places/Kelvhold.md", "Marren").get("kind", "")) == "note")

	# == 3. Never lose typed text =============================================
	vw._enter_note("Places/Kelvhold.md")
	await _frames(5)
	ed = _ed(vw)
	te = ed.text_edit
	await _tap(vw, _centre(te))
	te.set_caret_line(te.get_line_count() - 1)
	te.set_caret_column(te.get_line(te.get_line_count() - 1).length())
	await _type("ZZTYPED")
	_ok("CONTROL: typing reached the editor", te.text.find("ZZTYPED") >= 0, te.text.substr(te.text.length() - 40))
	await _press(vw, ed.buttons["preview"])
	var zpos := _link_pos(vw, _ed(vw).preview, "wiki:Marren")
	await _tap(vw, zpos)
	await _frames(5)
	var dlg: Array = []
	_find_all(app, "AcceptDialog", dlg)
	var asked := false
	for d in dlg:
		if (d as Window).visible and d != vw and (d as Window).title.find("Unsaved") >= 0:
			asked = true
	## VE-4 changed this: leaving a note saves it (through the hash guard) instead of asking, so a dirty note
	## no longer meets the Discard / Cancel card; only a save that was refused would.
	_ok("following a link from a dirty note saves it and follows, with no prompt", not asked and vw._browse_path == "Places/Marren.md", vw._browse_path)
	_ok("... the typed text reached the file on disk", _disk("Places/Kelvhold.md").find("ZZTYPED") >= 0, _disk("Places/Kelvhold.md").right(40))

	# == 4. The four sample notes are unchanged by all of the above ===========
	for d in dlg:
		if (d as Window).visible and d != vw:
			(d as Window).hide()
	for rel in ["Places/Marren.md", "People/Aldis.md", "Journal/Thaw.md"]:
		_ok("%s is byte-identical on disk" % rel, _disk(rel) == before[rel])
	await _shot("5_end.png")
	_finish()
