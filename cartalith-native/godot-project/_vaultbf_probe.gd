extends Node
## Owner Ruling BF (`LARGE_ITEM_RULINGS.md`, 2026-09-27): vault templates and
## Markdown follow Obsidian exactly. Drives the real app, the real vault window
## and a real fixture vault on disk that carries a real
## `.obsidian/templates.json` (folder `Templates`, a custom `dateFormat` and
## `timeFormat`); every file assertion re-reads the file from disk.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _vaultbf_probe.tscn -- --vp 1600x1000 --shots <dir>
##   Godot_v4.7.1-stable_win64_console.exe --path . _vaultbf_probe.tscn -- --vp 1080x2340 --force-touch --shots <dir>
##
## **Windowed, not `--headless`**: it saves screenshots, and the pixel path is a
## no-op under the dummy driver (`MISTAKES.md`). Flags it reads, grepped from
## the body below: `--vp WxH` (host viewport, default 1600x1000),
## `--force-touch` (checked against `DccTheme.is_phone()`), `--shots DIR`
## (where PNGs go; none when absent). Any other flag aborts.
##
## Every expected date, time and weekday is built here from Godot's own clock
## (`Time.get_datetime_dict_from_system()`) with `%02d` formatting and a
## weekday table typed below -- never from the engine's Moment formatter, which
## is what is under test. Exit 0 = every check held, 1 = a check failed,
## 2 = could not run. The RESULT line carries the check count.

const SEED := 552017
const DAYS := ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"]
const TEMPLATE := "---\ntags: [place]\nstatus: draft\n---\n# {{title}}\nCreated {{date}} at {{time}}, a {{date:dddd}}, for {{Place_Name}}.\nTemplater: <% tp.file.creation_date() %>\n"

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
		print("VAULTBF  ok  %s" % label)
	else:
		_fails.append(label)
		print("VAULTBF  !!  %s   %s" % [label, detail])


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


func _rm_tree(path: String) -> void:
	var d := DirAccess.open(path)
	if d == null:
		return
	d.include_hidden = true
	for f in d.get_files():
		DirAccess.remove_absolute(path + "/" + f)
	for sub in d.get_directories():
		_rm_tree(path + "/" + sub)
	DirAccess.remove_absolute(path)


func _shot(name: String) -> void:
	await _frames(6)
	await RenderingServer.frame_post_draw
	await get_tree().process_frame
	await RenderingServer.frame_post_draw
	if _shots == "":
		return
	var tag := "phone_" if _phone else ""
	_vp.get_texture().get_image().save_png(_shots + "/" + tag + name)
	print("VAULTBF  shot %s/%s%s" % [_shots, tag, name])


func _find_named(n: Node, target: String) -> Node:
	if n.name == target:
		return n
	for c in n.get_children():
		var r := _find_named(c, target)
		if r != null:
			return r
	return null


func _find_button_prefix(n: Node, prefix: String) -> Button:
	if n is Button and String((n as Button).text).to_lower().begins_with(prefix.to_lower()):
		return n as Button
	for c in n.get_children():
		var r := _find_button_prefix(c, prefix)
		if r != null:
			return r
	return null


func _find_all(n: Node, cls: String, out: Array) -> void:
	if n.is_class(cls):
		out.append(n)
	for c in n.get_children():
		_find_all(c, cls, out)


func _tree(vw: Node) -> Tree:
	var ts: Array = []
	_find_all(vw, "Tree", ts)
	return ts[0] as Tree if not ts.is_empty() else null


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


## Phone: a synthetic tap into an embedded window at phone scale does not
## route (`MISTAKES.md`), so the button's own signal stands in for it.
func _press(vw: Window, b: Button) -> void:
	if _phone:
		b.pressed.emit()
		await _frames(3)
	else:
		await _click(vw, b)


## `{date: "DD.MM.YYYY", time: "HH.mm", weekday: "Sunday"}` from Godot's clock.
func _clock() -> Dictionary:
	var t := Time.get_datetime_dict_from_system()
	return {
		"date": "%02d.%02d.%04d" % [int(t["day"]), int(t["month"]), int(t["year"])],
		"time": "%02d.%02d" % [int(t["hour"]), int(t["minute"])],
		"weekday": DAYS[int(t["weekday"])],
	}


func _ready() -> void:
	for a in OS.get_cmdline_user_args():
		var s := String(a)
		if s.begins_with("--") and not (s in ["--vp", "--force-touch", "--shots"]):
			print("VAULTBF ABORT unknown flag %s" % s)
			get_tree().quit(2)
			return
	if DisplayServer.get_name() == "headless":
		print("VAULTBF REFUSED: run windowed")
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
	if not _bridge._has("vault_insert_template"):
		print("VAULTBF REFUSED: this .dll predates Ruling BF (stale build)")
		get_tree().quit(2)
		return

	_bridge.generate({"seed": SEED, "width_km": 2400.0, "grid_w": 384, "grid_h": 288,
		"archetype": "", "villages": true, "sea_level": 0.45})
	while _bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await _frames(4)
	var ss: Array = _bridge.settlements()
	_ok("CONTROL: the world has a settlement", ss.size() >= 1, str(ss.size()))
	if ss.is_empty():
		_finish()
		return
	var tid := int((ss[0] as Dictionary).get("tid", -1))
	var nm := String((ss[0] as Dictionary).get("name", "?"))

	# -- the fixture vault: real Obsidian settings, a lookalike outside them ----
	_root = OS.get_environment("TEMP").replace("\\", "/") + ("/cartalith-vaultbf-%s" % ("phone" if _phone else "desk"))
	_rm_tree(_root)
	_write(_root + "/.obsidian/templates.json", "{\"folder\":\"Templates\",\"dateFormat\":\"DD.MM.YYYY\",\"timeFormat\":\"HH.mm\"}")
	_write(_root + "/Templates/Place.md", TEMPLATE)
	_write(_root + "/Settlement Template.md", "## Old convention: [Name]\n")
	_write(_root + "/Notes/Journal.md", "---\ntags: [log]\n---\nFirst line.\n")
	var conn: Dictionary = _bridge.vault_connect(_root, "BF Fixture")
	_ok("CONTROL: the fixture vault binds", bool(conn.get("ok", false)), str(conn))

	# == 1. The folder comes from templates.json =================================
	var src: Dictionary = _bridge.vault_template_source()
	_ok("the source is Obsidian's settings, folder Templates",
		String(src.get("source", "")) == "obsidian" and String(src.get("folder", "")) == "Templates", str(src))
	var rels: Array = []
	for t in _bridge.vault_templates():
		rels.append(String((t as Dictionary).get("rel", "")))
	_ok("only the folder's template is offered -- not 'Settlement Template.md' outside it",
		rels == ["Templates/Place.md"], str(rels))

	# == 2. New note from a template, read back from disk ========================
	app.open_vault("settlement", tid, nm)
	await _frames(6)
	var vw: VaultWindow = app.vault_window
	var src_label := _find_named(vw, "TemplateSource") as Label
	_ok("the window says where the templates came from",
		src_label != null and String(src_label.text) == "templates from Templates/ (Obsidian settings)",
		String(src_label.text) if src_label != null else "<no label>")
	var suggested: String = _bridge.vault_suggested_path("settlement", nm)
	var create := _find_button_prefix(vw, "Create ")
	_ok("the Create button is there", create != null)
	if create == null:
		_finish()
		return
	await _shot("1_create_from_template.png")
	var before := _clock()
	await _press(vw, create)
	await _frames(4)
	var after := _clock()
	var made := _disk(suggested)
	print("VAULTBF created %s:\n%s" % [suggested, made])
	var stem := suggested.get_file().get_basename()
	_ok("the note exists on disk", made != "")
	_ok("{{title}} is the new note's name", made.find("\n# %s\n" % stem) >= 0, made)
	var dated := false
	for c in [before, after]:
		if made.find("Created %s at %s, a %s, for %s." % [c["date"], c["time"], c["weekday"], nm]) >= 0:
			dated = true
	_ok("{{date}}/{{time}} use the vault's DD.MM.YYYY / HH.mm, {{date:dddd}} the weekday, {{Place_Name}} the settlement",
		dated, "want e.g. Created %s at %s, a %s, for %s." % [after["date"], after["time"], after["weekday"], nm])
	_ok("Templater's <% %> survived verbatim", made.find("Templater: <% tp.file.creation_date() %>\n") >= 0)
	_ok("no placeholder is left unfilled", made.find("{{") < 0)
	_ok("CONTROL: the template itself is untouched", _disk("Templates/Place.md") == TEMPLATE)
	await _shot("2_created_note.png")
	vw.hide()
	await _frames(2)

	# == 3. Insert template into an open note ====================================
	app.open_vault_browse()
	await _frames(4)
	var tree := _tree(vw)
	var leaf := _leaf_in(tree.get_root(), "Notes/Journal.md") if tree != null else null
	_ok("the journal note is in the tree", leaf != null)
	if leaf == null:
		_finish()
		return
	leaf.select(0)
	await _frames(4)
	## Browsing opens on the first note, which here is already this one, so the
	## selection changes nothing; a phone user then taps NOTE.
	if _phone and vw._browse_phone_pane == "tree":
		var note_tab := _find_button_prefix(vw, "NOTE")
		if note_tab != null:
			note_tab.pressed.emit()
			await _frames(4)
	var open_btn := _find_button_prefix(vw, "Open to edit")
	_ok("Open to edit is offered", open_btn != null,
		"pick=%s pane=%s" % [vw._pick_file, vw._browse_phone_pane])
	if open_btn == null:
		_finish()
		return
	await _press(vw, open_btn)
	await _frames(6)
	var ed = _find_named(vw, "VaultMarkdownEditor")
	_ok("the editor opened", ed != null)
	if ed == null:
		_finish()
		return
	var missing: Array = []
	for k in ["bold", "italic", "underline", "strike", "highlight", "h0", "h1", "h2", "h3", "bullet",
			"number", "task", "quote", "callout", "code", "code_block", "link", "tag", "template", "write", "preview"]:
		if not (ed.buttons as Dictionary).has(k):
			missing.append(k)
	_ok("the toolbar carries every Ruling BE and BF control", missing.is_empty(), str(missing))
	var te: TextEdit = ed.text_edit
	te.set_caret_line(te.get_line_count() - 1)
	te.set_caret_column(te.get_line(te.get_line_count() - 1).length())
	await _press(vw, ed.buttons["template"])
	await _frames(3)
	var pop := _find_named(ed, "EditorTemplatePicker") as PopupMenu
	_ok("Insert template opens a picker listing the template",
		pop != null and pop.visible and pop.item_count == 1 and pop.get_item_text(0) == "Place",
		"%s %s" % [pop.visible if pop != null else "-", pop.get_item_text(0) if pop != null and pop.item_count > 0 else "-"])
	if pop != null and not _phone:
		## Where it opened, against the button's own rect lifted by the window's
		## position -- computed here, not through the editor's helper. 40 px of
		## slack: the popup's Window rect includes its drop-shadow margin
		## (measured 34 x 16 px), while the bug this guards against put it off
		## by the vault window's whole offset (160 x 64 px at this size).
		var b_rect: Rect2 = (ed.buttons["template"] as Button).get_global_rect()
		var want_top := Vector2(vw.position).y + b_rect.end.y
		var want_left := Vector2(vw.position).x + b_rect.position.x
		_ok("the picker opens under the Insert template button, not offset by the window",
			absf(float(pop.position.y) - want_top) <= 40.0 and absf(float(pop.position.x) - want_left) <= 40.0,
			"popup at %s, button bottom-left at (%d, %d)" % [str(pop.position), int(want_left), int(want_top)])
	await _shot("3_insert_template_picker.png")
	var ins_clock := _clock()
	if pop != null:
		pop.id_pressed.emit(pop.get_item_id(0))
		pop.hide()
	await _frames(3)
	## Concatenated, not `%`-formatted: the Templater tag is itself a `%`.
	var want := "---\ntags:\n  - log\n  - place\nstatus: draft\n---\nFirst line.\n# Journal\nCreated " \
		+ String(ins_clock["date"]) + " at " + String(ins_clock["time"]) + ", a " + String(ins_clock["weekday"]) \
		+ ", for Journal.\nTemplater: <% tp.file.creation_date() %>\n"
	_ok("inserted at the caret, {{title}} = this note, tags merged (list union), a new key added",
		te.text == want, "\ngot:\n%s\nwant:\n%s" % [te.text, want])
	await _press(vw, ed.save_button)
	await _frames(3)
	_ok("Save wrote exactly that to disk (re-read)", _disk("Notes/Journal.md") == te.text)
	await _shot("4_inserted.png")

	# == 4. Toolbar marks and Preview ===========================================
	te.set_caret_line(te.get_line_count() - 1)
	te.set_caret_column(0)
	te.insert_text_at_caret("> [!warning] Low bridge\n> Barges lower their masts.\n")
	te.insert_text_at_caret("- [ ] pay the toll\n- [x] cross\nThe ==salt== road #trade\n")
	await _frames(2)
	await _shot("5_write_with_marks.png")
	if _phone:
		await _press(vw, ed.buttons["preview"])
	else:
		te.grab_focus()
		await _frames(1)
		var k := InputEventKey.new()
		k.keycode = KEY_E
		k.ctrl_pressed = true
		k.pressed = true
		_vp.push_input(k)
	await _frames(3)
	var pv: RichTextLabel = ed.preview
	_ok("Ctrl+E (desktop) / Preview (phone) shows the rendering", pv.visible and not te.visible)
	var bb := String(pv.text)
	_ok("Preview renders the callout's title as its header", bb.find("[b]Low bridge[/b]") >= 0, bb)
	_ok("... and its body as prose, not an italic quote", bb.find("[indent]Barges lower their masts.[/indent]") >= 0)
	_ok("Preview renders the open and the done task", bb.find("☐  pay the toll") >= 0 and bb.find("☑  ") >= 0 and bb.find("[s]cross[/s]") >= 0)
	_ok("Preview renders the highlight", bb.find("[bgcolor=#") >= 0 and bb.find("]salt[/bgcolor]") >= 0)
	_ok("Preview renders the tag", bb.find("]#trade[/color]") >= 0)
	await _shot("6_preview.png")
	if not _phone:
		pv.grab_focus()
		await _frames(1)
		var k2 := InputEventKey.new()
		k2.keycode = KEY_E
		k2.ctrl_pressed = true
		k2.pressed = true
		_vp.push_input(k2)
		await _frames(3)
		_ok("Ctrl+E again returns to the source", te.visible and not pv.visible)
	ed.cancel_requested.emit()
	await _frames(3)
	vw.hide()
	await _frames(2)
	_finish()


func _finish() -> void:
	if FileAccess.file_exists(VaultStore.PATH):
		DirAccess.remove_absolute(ProjectSettings.globalize_path(VaultStore.PATH))
	print("VAULTBF  RESULT checks=%d failed=%d %s" % [_n, _fails.size(), str(_fails)])
	get_tree().quit(0 if _fails.is_empty() else 1)
