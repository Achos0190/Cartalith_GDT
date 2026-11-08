extends Node
## MS-3 desktop-identity probe: the desktop shell must not move a pixel when the phone's domain sheet lands.
## Boots the real `app.tscn` at the window size (no phone), waits for it to settle, and either writes the
## framebuffer to `--out DIR/desk.png` or compares it with `--base DIR/desk.png` (and `--base2` for the second
## pristine capture, so run-to-run noise is measured, not assumed zero).
##   capture: godot --path . --resolution 1600x1000 _ms3desk_probe.tscn -- --out DIR
##   compare: godot --path . --resolution 1600x1000 _ms3desk_probe.tscn -- --base A --base2 B   (A, B pristine; this tree's capture is --out C)
## Windowed only (`ImageTexture` readback is a no-op headless, `MISTAKES.md` "Run a pixel probe"). Arguments read
## by THIS file: `--out`, `--base`, `--base2`; anything else aborts with exit 2. Prints `RESULT: ...`.

func _arg(name: String) -> String:
	var a := OS.get_cmdline_user_args()
	var k := a.find(name)
	return String(a[k + 1]) if k >= 0 and k + 1 < a.size() else ""

func _diff(a: Image, b: Image) -> int:
	if a.get_size() != b.get_size():
		return -1
	var n := 0
	for y in a.get_height():
		for x in a.get_width():
			if a.get_pixel(x, y) != b.get_pixel(x, y):
				n += 1
	return n

func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		if String(args[i]) in ["--out", "--base", "--base2"]:
			i += 2
		else:
			print("ABORT: unknown argument ", args[i])
			get_tree().quit(2)
			return
	var app: Node = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(3.0).timeout
	if "open_project_dialog" in app and app.open_project_dialog != null:
		app.open_project_dialog.hide()
	for k in 20:
		await get_tree().process_frame
	await RenderingServer.frame_post_draw
	var img := get_viewport().get_texture().get_image()
	var out := _arg("--out")
	if out != "":
		DirAccess.make_dir_recursive_absolute(out)
		img.save_png(out + "/desk.png")
		print("RESULT: WROTE %s (%s)" % [out + "/desk.png", str(img.get_size())])
	var base := _arg("--base")
	if base != "":
		var a := Image.load_from_file(base + "/desk.png")
		var d := _diff(img, a)
		print("RESULT: this capture vs base differs in %d pixels" % d)
		var b2 := _arg("--base2")
		if b2 != "":
			var d2 := _diff(a, Image.load_from_file(b2 + "/desk.png"))
			print("RESULT: base vs base2 (run-to-run noise) differs in %d pixels" % d2)
	get_tree().quit(0)
