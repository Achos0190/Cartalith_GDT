extends Node
## VE-5's copy of a note's links at the end of Preview, and VE-7's browse preview embeds (`VAULT_EDITOR_PROPOSAL.md` s.3.2, s.7).
##
##   godot --path . _ve7bare_probe.tscn -- --vp 1600x1000 [--touch] [--shots DIR]
##   ... -- --vp 1080x2340 --force-touch --density 2.625   (the phone leg)
##
## **Windowed**, with a scratch APPDATA; binds a throw-away vault in TEMP. Exit 0 pass, 1 a check failed, 2 could not run.
## What it protects (VE-7's Rust half): a picture embedded by BARE NAME (`![[crest.png]]`) resolves to the file in another folder
## of the vault, in the browse preview and in the editor's Preview, is drawn as an image (`[img` in the bbcode); two pictures of
## one name are named ambiguous rather than picked; a gif is said unsupported; a missing name is said missing. A control note
## embeds by full path. Scratch vault in TEMP.

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
		print("VB  ok  %s" % label)
	else:
		_fails.append(label)
		print("VB  !!  %s   %s" % [label, detail])


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
	print("VB RESULT %d checks, %d failed%s" % [_n, _fails.size(), "" if _fails.is_empty() else " :: " + str(_fails)])
	get_tree().quit(0 if _fails.is_empty() else 1)


func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		print("VB REFUSED: run windowed")
		get_tree().quit(2)
		return
	var wd := Timer.new()
	wd.wait_time = 240.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("VB WATCHDOG TIMEOUT")
		get_tree().quit(3))
	wd.start()
	var parts: PackedStringArray = _arg("--vp", "1600x900").split("x")
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

	_root = OS.get_environment("TEMP").replace("\\", "/") + "/cartalith-ve7bare-%d" % Time.get_ticks_msec()
	DirAccess.make_dir_recursive_absolute(_root + "/Assets/deep")
	DirAccess.make_dir_recursive_absolute(_root + "/Other")
	var img := Image.create(96, 64, false, Image.FORMAT_RGBA8)
	img.fill(Color(0.85, 0.15, 0.15))
	for x in range(8, 88):
		for y in range(8, 56):
			img.set_pixel(x, y, Color(0.95, 0.85, 0.2))
	img.save_png(_root + "/Assets/deep/crest.png")
	img.save_png(_root + "/Assets/twin.png")
	img.save_png(_root + "/Other/twin.png")
	_write(_root + "/Assets/anim.gif", "GIF89a")
	_write(_root + "/Notes/Bare.md", "# Bare

By bare name:

![[crest.png|120]]

Two of a name: ![[twin.png]]

A gif: ![[anim.gif]]

None: ![[nothing.png]]
")
	_write(_root + "/Notes/Path.md", "# Path

![[Assets/deep/crest.png]]
")
	var conn: Dictionary = _bridge.vault_connect(_root, "Chronicle")
	_ok("CONTROL: the fixture vault binds", bool(conn.get("ok", false)), str(conn))
	var o: Dictionary = _bridge.vault_embed_open("Notes/Bare.md", "crest.png", PackedStringArray(["Notes/Bare.md"]))
	_ok("the crate resolves the bare name", String(o.get("kind", "")) == "image" and String(o.get("path", "")) == "Assets/deep/crest.png", str(o))
	app.open_vault_browse()
	await _frames(4)
	vw = app.vault_window

	vw._select_file("Notes/Path.md")
	await _frames(6)
	var body := vw.find_child("VaultNoteRendered", true, false) as RichTextLabel
	_ok("CONTROL: a full-path picture embeds in browse", body != null and body.text.contains("[img"), body.text if body != null else "")

	vw._select_file("Notes/Bare.md")
	await _frames(6)
	body = vw.find_child("VaultNoteRendered", true, false) as RichTextLabel
	_ok("browse: the bare-name picture is drawn as an image", body != null and body.text.contains("[img"), body.text if body != null else "")
	var shown := body.get_parsed_text() if body != null else ""
	_ok("browse: two of one name are called ambiguous, not picked", shown.contains("could mean 2 files"), shown)
	_ok("browse: the gif is said unsupported", shown.to_lower().contains("gif"), shown)
	_ok("browse: the missing name is said missing", shown.to_lower().contains("nothing.png"), shown)
	await _shot("browse")

	vw._enter_note("Notes/Bare.md")
	await _frames(8)
	var ed = vw._editor
	_ok("the editor opens", ed != null)
	if ed != null:
		ed.set_mode("preview")
		await _frames(6)
		_ok("editor Preview: the bare-name picture is drawn as an image", ed.preview.text.contains("[img"), ed.preview.text)
		await _shot("preview")
	_finish()
