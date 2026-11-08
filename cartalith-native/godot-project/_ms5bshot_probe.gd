extends Node
## MS-5 second step: a desktop-identity capture. Boots the shell at the window's own size, generates the fixed
## world below, waits for it to settle and saves the window's framebuffer to `--out PATH`. Run it on this tree and on a
## pristine `origin/main` tree, twice on the pristine one for run-to-run noise, and compare the PNGs byte for byte:
## the tool strip is the phone's, so a desktop capture must not move a pixel.
##   godot --path . --resolution 1600x1000 _ms5bshot_probe.tscn -- --out C:/cw/ms5b_shots/this.png
## Arguments read by THIS file: `--out PATH` (required). Anything else aborts with exit 2.

func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var out := ""
	var i := 0
	while i < args.size():
		if String(args[i]) == "--out" and i + 1 < args.size():
			out = String(args[i + 1])
			i += 2
		else:
			print("ABORT: unknown argument ", args[i])
			get_tree().quit(2)
			return
	if out.is_empty():
		print("ABORT: needs --out PATH")
		get_tree().quit(2)
		return
	var wd := Timer.new()
	wd.wait_time = 120.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("WATCHDOG TIMEOUT")
		get_tree().quit(3))
	wd.start()
	var app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.6).timeout
	var bridge = app.bridge
	bridge.generate({"seed": 24601, "width_km": 1200.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.42})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.5).timeout
	if "open_project_dialog" in app and app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await get_tree().create_timer(1.0).timeout
	DirAccess.make_dir_recursive_absolute(out.get_base_dir())
	var img: Image = get_viewport().get_texture().get_image()
	var err := img.save_png(out)
	print("saved %s (%d x %d) err=%d" % [out, img.get_width(), img.get_height(), err])
	get_tree().quit(0 if err == OK else 1)
