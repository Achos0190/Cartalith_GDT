extends Node
## Ruling ET's lake water balance through the real shell: the app boots with
## `lake_balance` ON, the New World dialog and World ▸ Hydrology publish the
## row, and a world generated with it off then on draws different water
## (`cargo test` cannot see the shell, `MISTAKES.md`).
##
## MUST run WINDOWED, under `C:/cw/.win_lock`:
##
##   Godot_v4.7.2-stable_win64_console.exe --path . --resolution 1600x1000 \
##       _lakebal_probe.tscn -- --out DIR [--seed 483920]
##
## Asserts: the row exists with default ON; generation with it off and on both
## finish; the two rendered frames differ (a switch that moved no pixel is
## inert, `MISTAKES.md`); the frame with it on is saved as the evidence.
## Prints PROBE-FAIL then exits 1 on a failed assertion, 0 when none; 2 when it
## cannot run (headless, generation never finished).

var _out := "user://lakebal/"
var _seed := 483920
var _fails := 0


func _ok(cond: bool, what: String) -> void:
	print(("  PASS  " if cond else "  PROBE-FAIL  ") + what)
	if not cond:
		_fails += 1


func _settle(n: int) -> void:
	for f in n:
		await RenderingServer.frame_post_draw


func _gen(br: Node, v: bool) -> Image:
	br.param_set("lake_balance", v)
	br.generate({"seed": _seed, "width_km": 1200.0, "grid_w": 1024, "grid_h": 656,
		"archetype": "", "villages": false, "sea_level": 0.42})
	var spins := 0
	while br.generating and spins < 2400:
		await get_tree().create_timer(0.25).timeout
		spins += 1
	if br.generating:
		return null
	await get_tree().create_timer(1.0).timeout
	return br.color_texture().get_image()


func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		printerr("PROBE-CANNOT-RUN: headless -- run windowed.")
		get_tree().quit(2)
		return
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		match args[i]:
			"--out":
				i += 1
				_out = args[i]
			"--seed":
				i += 1
				_seed = int(args[i])
		i += 1
	DirAccess.make_dir_recursive_absolute(_out)
	var app: Node = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.0).timeout
	var br: Node = app.bridge
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await get_tree().process_frame
	var row: Dictionary = br.param_info("lake_balance")
	_ok(not row.is_empty(), "the registry publishes lake_balance")
	_ok(br.param_get("lake_balance") == true, "the app boots with lake_balance ON")
	var off: Image = await _gen(br, false)
	if off == null:
		printerr("PROBE-CANNOT-RUN: generation never finished.")
		get_tree().quit(2)
		return
	_ok(br.param_get("lake_balance") == false, "the off world was generated with it off")
	var on: Image = await _gen(br, true)
	if on == null:
		printerr("PROBE-CANNOT-RUN: generation never finished.")
		get_tree().quit(2)
		return
	var differ := 0
	if off.get_size() == on.get_size():
		var a := off.get_data()
		var b := on.get_data()
		var px := off.get_width() * off.get_height()
		var step := off.get_data().size() / maxi(px, 1)
		for p in px:
			var k := p * step
			for c in mini(step, 3):
				if a[k + c] != b[k + c]:
					differ += 1
					break
	print("pixels differing off vs on: %d of %s" % [differ, str(off.get_size())])
	_ok(differ > 0, "the switch moves the drawn map")
	off.save_png(_out.path_join("probe_off.png"))
	on.save_png(_out.path_join("probe_on.png"))
	print("_lakebal_probe: %s; images in %s" % ["ALL PASS" if _fails == 0 else "PROBE-FAIL FAILURES (%d failures)" % _fails, _out])
	get_tree().quit(0 if _fails == 0 else 1)
