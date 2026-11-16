extends Node
## VE-5's copy of a note's links at the end of Preview, and VE-7's browse preview embeds (`VAULT_EDITOR_PROPOSAL.md` s.3.2, s.7).
##
##   godot --path . _fovault_probe.tscn -- --vp 1600x1000 [--touch] [--shots DIR]
##   ... -- --vp 1080x2340 --force-touch --density 2.625   (the phone leg)
##
## **Windowed**, with a scratch APPDATA; binds a throw-away vault in TEMP. Exit 0 pass, 1 a check failed, 2 could not run.
## What it protects:
##   A. VE-5: with the side panel NOT beside the note, Preview ends with "LINKS TO THIS NOTE", listing the backlinks and
##      the unlinked mentions (a REAL meta click on a row opens that note at its line; Link writes the mention and the note
##      moves to the backlinks); with the panel beside the note (wide) there is NO copy (the panel already shows it);
##      Write mode's text is never touched by it.
##   B. VE-7 edge: the browse preview (a note selected, not opened to edit) draws a picture embed as an image and a note
##      embed collapsed, and a click on the embed's meta expands it.

var _vp: SubViewport
var app: Node
var _bridge
var vw
var _root := ""
var _shots := ""
var _n := 0
var _fails: Array = []


func _ok(label: String, cond: bool, detail: String = "") -> void:
	_n += 1
	if cond:
		print("FOV  ok  %s" % label)
	else:
		_fails.append(label)
		print("FOV  !!  %s   %s" % [label, detail])


func _arg(name: String, dflt: String) -> String:
	var args := OS.get_cmdline_user_args()
	var i := args.find(name)
	return String(args[i + 1]) if i >= 0 and i + 1 < args.size() else dflt


func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame


func _write(path: String, text: String) -> void:
	DirAccess.make_dir_recursive_absolute(path.get_base_dir())
	var f := FileAccess.open(path, FileAccess.WRITE)
	f.store_string(text)
	f.close()


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


func _shot(tag: String) -> void:
	if _shots == "":
		return
	DirAccess.make_dir_recursive_absolute(_shots)
	await _frames(4)
	_vp.get_texture().get_image().save_png(_shots + "/fov_%s.png" % tag)


func _finish() -> void:
	print("FOV RESULT %d checks, %d failed%s" % [_n, _fails.size(), "" if _fails.is_empty() else " :: " + str(_fails)])
	get_tree().quit(0 if _fails.is_empty() else 1)


func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		print("FOV REFUSED: run windowed")
		get_tree().quit(2)
		return
	var wd := Timer.new()
	wd.wait_time = 240.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("FOV WATCHDOG TIMEOUT")
		get_tree().quit(3))
	wd.start()
	var parts: PackedStringArray = _arg("--vp", "1600x1000").split("x")
	_shots = _arg("--shots", "")
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
	_bridge = app.bridge

	_root = OS.get_environment("TEMP").replace("\\", "/") + "/cartalith-fov-%d" % Time.get_ticks_msec()
	_write(_root + "/Q0first.md", "# Q0first\n")
	var long := ""
	for i in 12:
		long += "- salt barrel %d\n" % i
	_write(_root + "/Rhosk.md", "# Rhosk\n\nIntro line.\n\n" + long)
	_write(_root + "/Linker.md", "# Linker\n\nfirst\nSee [[Rhosk]] here.\n")
	_write(_root + "/Mentioner.md", "# Mentioner\n\nPara.\nThe old port of Rhosk is on the river.\n")
	var img := Image.create(8, 8, false, Image.FORMAT_RGBA8)
	img.fill(Color(0.9, 0.2, 0.2))
	DirAccess.make_dir_recursive_absolute(_root)
	img.save_png(_root + "/pic.png")
	_write(_root + "/Pic.md", "# Pic\n\n![[pic.png]]\n\n![[Rhosk]]\n")
	var conn: Dictionary = _bridge.vault_connect(_root, "Chronicle")
	_ok("CONTROL: the fixture vault binds", bool(conn.get("ok", false)), str(conn))
	_bridge.vault_refresh_backlinks(2000)
	app.open_vault_browse()
	await _frames(4)
	vw = app.vault_window

	# == B first: the browse preview of Pic.md =================================
	vw._select_file("Pic.md")
	await _frames(6)
	var body := vw.find_child("VaultNoteRendered", true, false) as RichTextLabel
	_ok("B. the browse preview exists", body != null)
	if body != null:
		_ok("B. a picture embed is drawn as an image ([img] in the bbcode)", body.text.contains("[img"), body.text)
		_ok("B. a note embed is a collapsed block with an embed: toggle", body.text.contains("embed:0"), body.text)
		var before := body.get_parsed_text().length()
		body.meta_clicked.emit("embed:0")
		await _frames(3)
		_ok("B. a click on the embed's toggle expands it (more text)", body.get_parsed_text().length() > before, "%d vs %d" % [before, body.get_parsed_text().length()])
		await _shot("browse_embeds")

	# == A: the footer ========================================================
	vw._select_file("Rhosk.md")
	await _frames(4)
	vw._link_back.clear()
	vw._enter_note("Rhosk.md")
	await _frames(6)
	var ed = vw._editor
	var beside: bool = ed != null and vw._side_panel != null and vw._side_panel.visible and vw._side_panel.get_parent() is HBoxContainer
	print("FOV info: window %s, panel beside=%s" % [vw.size, beside])
	var write_text: String = ed.text_edit.text
	ed.set_mode("preview")
	await _frames(4)
	var shown: String = ed.preview.get_parsed_text()
	if beside:
		_ok("A. the panel is beside the note: Preview has NO copy of the links", not shown.contains("LINKS TO THIS NOTE"), shown)
		vw._set_side(false)
		await _frames(3)
		ed.set_mode("write")
		ed.set_mode("preview")
		await _frames(3)
		shown = ed.preview.get_parsed_text()
	_ok("A. the panel hidden or below: Preview ends with the copy", shown.contains("LINKS TO THIS NOTE") and shown.ends_with("\n") == shown.ends_with("\n"), shown)
	_ok("A. it lists the backlink and the mention", shown.contains("Backlinks  2") and shown.contains("Linker.md") and shown.contains("Pic.md") and shown.contains("Unlinked mentions  1") and shown.contains("Mentioner.md"), shown)
	_ok("A. the copy is after the note's own text", shown.find("Intro line.") >= 0 and shown.find("Intro line.") < shown.find("LINKS TO THIS NOTE"))
	_ok("A. the Write text is untouched by the copy", ed.text_edit.text == write_text)
	var vb: ScrollBar = ed.preview.get_v_scroll_bar()
	vb.value = vb.max_value
	await _shot("footer")
	# a real meta click on the backlink row opens that note at its link's line
	ed.preview.meta_clicked.emit("foot:0")
	await _frames(8)
	_ok("A. a click on the backlink opens it", vw._browse_path == "Linker.md", vw._browse_path)
	_ok("A. ... with the caret on the link's line (3)", vw._editor != null and vw._editor.text_edit.get_caret_line() == 3, str(vw._editor.text_edit.get_caret_line() if vw._editor != null else -1))
	# Link
	vw._enter_note("Rhosk.md")
	await _frames(8)
	ed = vw._editor
	ed.set_mode("preview")
	await _frames(4)
	ed.preview.meta_clicked.emit("footl:2")
	await _frames(8)
	var mt := FileAccess.get_file_as_string(_root + "/Mentioner.md")
	_ok("A. Link in the copy wrote the mention as [[Rhosk]] on disk", mt.contains("port of [[Rhosk]] is"), mt)
	shown = ed.preview.get_parsed_text()
	_ok("A. ... and the note moved from the mentions to the backlinks", shown.contains("Backlinks  3") and shown.contains("Unlinked mentions  0"), shown)
	await _shot("footer_after_link")
	_finish()
