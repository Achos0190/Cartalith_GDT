extends Node
## Committed measurement harness for `EXPORT_SCOPE.md` E5's JPEG writer: file
## size and wall time of terrain-only `export_image` exports, PNG beside JPEG,
## on the grid every earlier export measurement used (2048 x 1311).
##
##   godot --headless --path . _exportjpeg_probe.tscn -- --seed 20260906 --widths 2048,4096,8192,16384 --qualities 90
##   godot --headless --path . _exportjpeg_probe.tscn -- --seed 20260906 --widths 4096 --qualities 50,70,80,90,95,100 --no-png
##
## Every flag is optional: `--seed` (default 20260906, the seed `EXPORT_SCOPE.md`
## §6.3's PNG table led with), `--widths` (comma list of `BAKE_WIDTHS` rungs,
## default 2048,4096,8192), `--qualities` (comma list of JPEG qualities 1..100,
## default 90) and `--no-png` (skip the PNG beside each width). Anything else
## aborts with exit 2 (`MISTAKES.md`: a probe header is a claim about its own
## parsing). **Never pass 32768 in a routine run** (the E5 brief; one 32K JPEG
## writes for minutes).
##
## `--headless` is right here: `export_image` is the engine's own banded
## renderer and streaming writer, with no `SubViewport` and nothing drawn.
## One world per process; the exports run one after another in this process,
## so their timings are sequential, not parallel. Peak memory is NOT measured
## here (that is `_exportfull_probe.gd`, polled from the host).
##
## Result: a measurement. Prints `  seed <n> generated 2048 x 1311 in <s> s`,
## then one `  RUN <png|jpeg> q<quality|-> <width> -> <w> x <h>  <bytes> bytes
## <B/px> bytes/px  <s> s wall` row per export, and `RESULT: PASS` when every
## export wrote a file (exit 0); `  FAILED ...` and `RESULT: FAIL` (exit 1) on a
## refused export; `ABORT:` (exit 2) on a bad flag; the watchdog exits 2. Every
## file is deleted after it is measured.

var bridge: Node

func _ints(csv: String) -> Array[int]:
	var out: Array[int] = []
	for p in csv.split(",", false):
		out.append(int(p))
	return out

func _size(path: String) -> int:
	var f := FileAccess.open(path, FileAccess.READ)
	if f == null:
		return -1
	var n := f.get_length()
	f.close()
	return n

func _ready() -> void:
	var seed_value := 20260906
	var widths: Array[int] = [2048, 4096, 8192]
	var qualities: Array[int] = [90]
	var with_png := true
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		match args[i]:
			"--seed":
				seed_value = int(args[i + 1]) if i + 1 < args.size() else 0
				i += 2
			"--widths":
				widths = _ints(args[i + 1] if i + 1 < args.size() else "")
				i += 2
			"--qualities":
				qualities = _ints(args[i + 1] if i + 1 < args.size() else "")
				i += 2
			"--no-png":
				with_png = false
				i += 1
			_:
				print("ABORT: unrecognised argument %s -- this probe reads --seed, --widths, --qualities and --no-png only" % args[i])
				get_tree().quit(2)
				return
	if widths.is_empty() or qualities.is_empty():
		print("ABORT: --widths and --qualities must each name at least one value")
		get_tree().quit(2)
		return
	get_tree().create_timer(3600.0).timeout.connect(func() -> void:
		print("WATCHDOG: the probe did not finish")
		get_tree().quit(2))
	bridge = load("res://shell/engine_bridge.gd").new()
	add_child(bridge)
	await get_tree().process_frame
	var dir := ProjectSettings.globalize_path("user://_exportjpeg_probe")
	DirAccess.make_dir_recursive_absolute(dir)
	var tg := Time.get_ticks_msec()
	bridge.world_gen.generate_sized(seed_value, 1200.0, 2048, 1311)
	bridge.has_world = true
	print("  seed %d generated 2048 x 1311 in %.1f s" % [seed_value, (Time.get_ticks_msec() - tg) / 1000.0])

	var failed := 0
	for w in widths:
		var runs: Array = []
		if with_png:
			runs.append({"format": "png"})
		for q in qualities:
			runs.append({"format": "jpeg", "quality": q})
		for r0 in runs:
			var opts: Dictionary = r0.duplicate()
			opts["width"] = w
			var ext := "png" if opts["format"] == "png" else "jpg"
			var p := dir.path_join("e_%d_%s.%s" % [w, str(opts.get("quality", "")), ext])
			var t0 := Time.get_ticks_msec()
			var r: Dictionary = bridge.world_gen.export_image(p, opts)
			var wall := (Time.get_ticks_msec() - t0) / 1000.0
			if not bool(r.get("ok", false)):
				print("  FAILED %s at %d: %s" % [opts, w, String(r.get("error", "?"))])
				failed += 1
				continue
			var px := int(r["width"]) * int(r["height"])
			var bytes := _size(p)
			print("  RUN %s q%s %6d -> %6d x %-6d %12d bytes  %.4f bytes/px  %.1f s wall" % [opts["format"],
				str(opts.get("quality", "-")), w, int(r["width"]), int(r["height"]), bytes, float(bytes) / float(px), wall])
			DirAccess.remove_absolute(p)
	print("RESULT: %s" % ("PASS" if failed == 0 else "FAIL"))
	get_tree().quit(0 if failed == 0 else 1)
