extends Node
## PC-3 phone guard: the phone keeps its shipped strip (CIVIL only, the TIMELINE button and the sim strip),
## so its captures must be byte-identical before and after PC-3 (`PC_SHELL_PROPOSAL.md` changes nothing on
## the phone). Captures WORLD, CIVIL, CARTO and CIVIL with the phone sim strip open at 393x852, and prints
## whether the strip is visible in each (it must be CIVIL only).
##   godot --path . --resolution 393x852 _pc3phone_probe.tscn -- --force-touch --shots <dir>
## Windowed, from a project copy with its own `config/name` and a wiped user dir (`MISTAKES.md`).
## The comparison is done by the caller (two baseline captures from origin/main must be equal to each other
## and to these), because only a build without PC-3 can say what "unchanged" is.
## Exit 0 captures taken, 2 bad argument, watchdog or not the phone.
var _shots := ""
var _app: Node
var _bridge

func _settle(n: int = 4) -> void:
	for _i in n:
		await get_tree().process_frame

func _cap(tag: String) -> void:
	await _settle(8)
	await RenderingServer.frame_post_draw
	var img := get_viewport().get_texture().get_image()
	var path := _shots.path_join("pc3_phone_%s.png" % tag)
	img.save_png(path)
	print("SHOT %s strip_visible=%s" % [path, str(_app.timeline_bar.is_visible_in_tree())])

func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var k := 0
	while k < args.size():
		if args[k] == "--force-touch":
			pass
		elif args[k] == "--shots" and k + 1 < args.size():
			k += 1
			_shots = args[k]
		else:
			print("ABORT: unrecognised argument %s" % args[k])
			get_tree().quit(2)
			return
		k += 1
	if _shots == "":
		print("ABORT: --shots <dir> is required")
		get_tree().quit(2)
		return
	DirAccess.make_dir_recursive_absolute(_shots)
	get_tree().create_timer(600.0).timeout.connect(func() -> void:
		print("WATCHDOG")
		get_tree().quit(2))
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.2).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _settle(6)
	if not DccTheme.is_phone():
		print("[FATAL] not the phone")
		get_tree().quit(2)
		return
	_bridge = _app.bridge
	_bridge.generate({"seed": 4242, "width_km": 2400.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.45})
	var t0 := Time.get_ticks_msec()
	await _settle(2)
	while bool(_bridge.generating) and Time.get_ticks_msec() - t0 < 240000:
		await _settle(10)
	await _settle(30)
	_bridge.civ_add_year(100)
	_app.tl_set_year(100, true)
	for d in ["world", "civilization", "cartography"]:
		_app.select_domain(d)
		await _cap(d)
	_app.select_domain("civilization")
	_app.set_phone_sim_strip_open(true)
	await _cap("civil_sim")
	print("RESULT: PASS (captures taken)")
	get_tree().quit(0)
