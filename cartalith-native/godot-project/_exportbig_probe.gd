extends Node
## One export, at one width, so the host can watch the process while it runs.
##
##   godot --headless --path . _exportbig_probe.tscn -- --width 16384
##   godot --headless --path . _exportbig_probe.tscn -- --width 32768 --seed 7
##   godot --headless --path . _exportbig_probe.tscn -- --width 32768 --heightmap
##
## `--width` is required; `--seed` (default 20260906) and `--heightmap` (which
## runs `export_heightmap_png` instead of `export_raster_png`, the other
## consumer of the same `BAKE_WIDTHS` ladder) are optional. All are read from
## `OS.get_cmdline_user_args()`, i.e. after the `--`, and **anything else
## aborts** rather than being ignored -- `MISTAKES.md`'s rule that a probe
## header is a claim about the probe's own parsing, and that a flag it does not
## read must fail loudly instead of silently measuring the default three times.
##
## Separate from `_export16k_probe.gd` because the interesting number here is
## the one this process cannot see: peak resident memory. One export per
## process, so the host's poll of the OS working set is attributable to it and
## not to a generate that ran twenty seconds earlier.
##
## Grid 2048 x 1311 -- `app.gd`'s default and the grid `EXPORT_SCOPE.md`
## section 6's dimensions come from.
##
## Committed, like every probe scene in this folder -- `STATUS.md`'s F8 row
## (`e1f18ca`, "Test harnesses committed").

var bridge: Node

func _size(path: String) -> int:
	var f := FileAccess.open(path, FileAccess.READ)
	if f == null:
		return -1
	var n := f.get_length()
	f.close()
	return n

func _ready() -> void:
	var width := -1
	var seed_value := 20260906
	var heightmap := false
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		match args[i]:
			"--width":
				width = int(args[i + 1]) if i + 1 < args.size() else -1
				i += 2
			"--seed":
				seed_value = int(args[i + 1]) if i + 1 < args.size() else 0
				i += 2
			"--heightmap":
				heightmap = true
				i += 1
			_:
				print("ABORT: unrecognised argument %s -- this probe reads --width, --seed and --heightmap only" % args[i])
				get_tree().quit(2)
				return
	if width <= 0:
		print("ABORT: --width is required, e.g. -- --width 16384")
		get_tree().quit(2)
		return

	get_tree().create_timer(3600.0).timeout.connect(func() -> void:
		push_error("export-big probe watchdog: _ready never finished")
		print("\n==== WATCHDOG: the probe did not finish ====\n")
		get_tree().quit(2))
	bridge = load("res://shell/engine_bridge.gd").new()
	add_child(bridge)
	await get_tree().process_frame

	var dir := ProjectSettings.globalize_path("user://_exportbig_probe")
	DirAccess.make_dir_recursive_absolute(dir)
	print("  scratch: %s" % dir)
	print("  memory_info: %s" % str(OS.get_memory_info()))

	var tg := Time.get_ticks_msec()
	bridge.world_gen.generate_sized(seed_value, 1200.0, 2048, 1311)
	bridge.has_world = true
	print("  seed %d generated 2048 x 1311 in %.1f s" % [seed_value, (Time.get_ticks_msec() - tg) / 1000.0])

	var e: Dictionary = bridge.world_gen.export_raster_estimate(width)
	print("  estimate %d -> %d x %d, %.1f MP, peak %.0f MB, keys %s"
		% [width, int(e.get("width", 0)), int(e.get("height", 0)),
			float(e.get("pixels", 0)) / 1e6, float(e.get("peak_bytes", 0)) / 1048576.0,
			str(e.keys())])
	print("  GENERATED -- starting the %s export now" % ("heightmap" if heightmap else "raster"))

	var p := dir.path_join("big_%s%d.png" % ["hm" if heightmap else "", width])
	var t0 := Time.get_ticks_msec()
	var r: Dictionary = (bridge.world_gen.export_heightmap_png(p, width) if heightmap
		else bridge.world_gen.export_raster_png(p, width, false))
	var wall := (Time.get_ticks_msec() - t0) / 1000.0
	if not bool(r.get("ok", false)):
		print("  REFUSED/FAILED at %d: %s" % [width, String(r.get("error", "?"))])
		get_tree().quit(1)
		return
	var px := int(r.get("width", 0)) * int(r.get("height", 0))
	var bytes := _size(p)
	print("  RUN %s %6d -> %6d x %-6d %12d bytes  %.4f bytes/px  %.1f s wall"
		% ["HM " if heightmap else "RGB", width, int(r.get("width", 0)), int(r.get("height", 0)), bytes,
			float(bytes) / float(px), wall])
	print("  EXPORT DONE -- deleting the file")
	DirAccess.remove_absolute(p)
	get_tree().quit(0)
