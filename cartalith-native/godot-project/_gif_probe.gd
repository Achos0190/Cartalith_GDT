extends Node
## VE-5's copy of a note's links at the end of Preview, and VE-7's browse preview embeds (`VAULT_EDITOR_PROPOSAL.md` s.3.2, s.7).
##
##   godot --path . _gif_probe.tscn -- --vp 1600x1000 [--touch] [--shots DIR]
##   ... -- --vp 1080x2340 --force-touch --density 2.625   (the phone leg)
##
## **Windowed**, with a scratch APPDATA; binds a throw-away vault in TEMP. Exit 0 pass, 1 a check failed, 2 could not run.
## What it protects (the vault's GIF decoder): an embedded `.gif` is drawn in browse and in the editor's Preview, its frames advance
## (the registered texture takes the colour of each frame in turn and the Preview's pixels change), a GIF past the frame cap shows
## its first frame with a quiet note and never ticks, a truncated GIF says it could not be read, and the cycler stops while the
## window is hidden. Fixtures: `_gif_fixtures/` (anim 3 frames x 100 ms; many 250 frames; cut, a truncated anim).

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
		print("GP  ok  %s" % label)
	else:
		_fails.append(label)
		print("GP  !!  %s   %s" % [label, detail])


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
	print("GP RESULT %d checks, %d failed%s" % [_n, _fails.size(), "" if _fails.is_empty() else " :: " + str(_fails)])
	get_tree().quit(0 if _fails.is_empty() else 1)


func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		print("GP REFUSED: run windowed")
		get_tree().quit(2)
		return
	var wd := Timer.new()
	wd.wait_time = 240.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("GP WATCHDOG TIMEOUT")
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

	_root = OS.get_environment("TEMP").replace("\\", "/") + "/cartalith-gif-%d" % Time.get_ticks_msec()
	DirAccess.make_dir_recursive_absolute(_root + "/Assets/deep")
	DirAccess.make_dir_recursive_absolute(_root + "/Other")
	for n in ["anim", "many", "cut"]:
		DirAccess.make_dir_recursive_absolute(_root + "/Assets")
		var src := FileAccess.get_file_as_bytes("res://_gif_fixtures/%s.gif" % n)
		_ok("CONTROL: fixture %s.gif is present" % n, src.size() > 0)
		var f := FileAccess.open(_root + "/Assets/%s.gif" % n, FileAccess.WRITE)
		f.store_buffer(src)
		f.close()
	_write(_root + "/Notes/Gif.md", "# Gif

![[anim.gif]]

Many: ![[many.gif]]

Cut: ![[cut.gif]]
")
	var conn: Dictionary = _bridge.vault_connect(_root, "Chronicle")
	_ok("CONTROL: the fixture vault binds", bool(conn.get("ok", false)), str(conn))
	var g: Dictionary = _bridge.vault_read_gif("Assets/anim.gif")
	_ok("bridge: anim.gif is 3 frames of 16x16 at 100 ms", bool(g.get("ok", false)) and (g["frames"] as Array).size() == 3 and int(g["width"]) == 16 and (g["delays_ms"] as PackedInt32Array) == PackedInt32Array([100, 100, 100]) and not bool(g["capped"]), str(g.get("error", "")))
	_ok("bridge: frame colours are red, green, blue", Array(g["frames"]).map(func(b): return Color8(b[0], b[1], b[2], b[3])) == [Color8(255, 0, 0), Color8(0, 255, 0), Color8(0, 0, 255)])
	var m: Dictionary = _bridge.vault_read_gif("Assets/many.gif")
	_ok("bridge: many.gif is held to its first frame with a note", bool(m.get("ok", false)) and (m["frames"] as Array).size() == 1 and bool(m["capped"]) and String(m["note"]).contains("too many frames"), str(m))
	var c: Dictionary = _bridge.vault_read_gif("Assets/cut.gif")
	_ok("bridge: cut.gif is an error with words, not a crash", not bool(c.get("ok", true)) and String(c.get("error", "")) != "", str(c))
	_ok("bridge: a png path is refused", not bool(_bridge.vault_read_gif("Assets/anim.png").get("ok", true)))
	app.open_vault_browse()
	await _frames(4)
	vw = app.vault_window
	## The phone's browse is two panes; a note already picked (the only one) does not switch pane by itself.
	vw._pick_file = ""
	vw._select_file("Notes/Gif.md")
	await _frames(6)
	var body := vw.find_child("VaultNoteRendered", true, false) as RichTextLabel
	_ok("browse: the gif is drawn as an image", body != null and body.text.contains("gif.png[/img]"), body.text if body != null else "")
	var shown := body.get_parsed_text() if body != null else ""
	_ok("browse: the capped gif carries its quiet note", shown.contains("too many frames to animate"), shown)
	_ok("browse: the cut gif says it could not be read", shown.contains("could not be read") and shown.contains("cut.gif"), shown)
	await _shot("browse")

	vw._enter_note("Notes/Gif.md")
	await _frames(8)
	var ed = vw._editor
	_ok("the editor opens", ed != null)
	if ed == null:
		_finish()
		return
	ed.set_mode("preview")
	await _frames(6)
	_ok("Preview: the animated gif is an image", ed.preview.text.contains("gif.png[/img]"), ed.preview.text)
	_ok("only the animated gif is held by the cycler", vw._gifs.keys() == ["Assets/anim.gif"], str(vw._gifs.keys()))
	var seen := {}
	var shots: Array = []
	for _s in 40:
		await get_tree().create_timer(0.05).timeout
		var tex: ImageTexture = (vw._gifs["Assets/anim.gif"] as Dictionary)["tex"]
		seen[tex.get_image().get_pixel(0, 0).to_html(false)] = true
		if _s % 8 == 0:
			shots.append(_vp.get_texture().get_image())
	_ok("frames advance: the texture took all three colours in 2 s", seen.size() == 3, str(seen.keys()))
	var changed := 0
	for a in range(1, shots.size()):
		if (shots[a] as Image).get_data() != (shots[a - 1] as Image).get_data():
			changed += 1
	_ok("Preview redraws as frames advance (framebuffer differs between samples)", changed >= 2, "%d of %d sample pairs differ" % [changed, shots.size() - 1])
	await _shot("preview")
	vw.hide()
	await _frames(2)
	var ent: Dictionary = vw._gifs["Assets/anim.gif"]
	var i0: int = ent["i"]
	var left0: float = ent["left"]
	vw._process(0.35)
	_ok("hidden: a tick moves neither the frame nor its clock", int(ent["i"]) == i0 and float(ent["left"]) == left0, "%d/%s" % [int(ent["i"]), ent["left"]])
	await get_tree().create_timer(0.5).timeout
	_ok("hidden: and real time does not advance it either", int(ent["i"]) == i0)
	vw.show()
	await _frames(2)
	vw._process(0.35)
	_ok("shown: the same tick advances one frame (never a burst)", int(ent["i"]) == (i0 + 1) % 3, str(ent["i"]))
	_finish()
