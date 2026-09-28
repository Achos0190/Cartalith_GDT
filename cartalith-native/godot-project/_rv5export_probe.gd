extends Node
## RV-5: one banded export (`WorldGen::export_image`), at one width, so the
## host can watch the process's peak working set while it runs -- the same
## one-export-per-process rule `_exportbig_probe.gd` states, for the path the
## 16K/32K dialog actually takes (E1's bands + E2's streaming writer) rather
## than the monolithic `export_raster_png`.
##
##   godot --headless --path . _rv5export_probe.tscn -- --width 16384 --out C:/tmp/x.png
##   godot --headless --path . _rv5export_probe.tscn -- --width 32768 --out C:/tmp/y.png --delete
##   godot --headless --path . _rv5export_probe.tscn -- --width 16384 --out C:/tmp/m.png --mono
##
## `--width` and `--out` are required. `--seed` (default 483920, a world with
## river mouths and above-sea lakes on the app's 2048 x 1311 grid), `--mono`
## (run `export_raster_png` instead), `--delete` (remove the file after
## reporting its size) and `--no-rivers` (the banded export's `content.rivers`
## off -- a control for what the rivers alone put in the picture) are optional. Anything else ABORTS rather than being
## ignored (`MISTAKES.md`: a flag the probe does not read must fail loudly).
##
## Prints one `RUN` line: width, height, bytes on disk, bands, and the wall
## time the export call itself took. Peak memory is the host's to measure.
## Protects: nothing by assertion -- it is a measuring harness; the parity
## assertions are `export_raster.rs`'s `rv5_parity_tests`.

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
	var seed_value := 483920
	var out := ""
	var mono := false
	var delete := false
	var rivers := true
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
			"--out":
				out = String(args[i + 1]) if i + 1 < args.size() else ""
				i += 2
			"--mono":
				mono = true
				i += 1
			"--delete":
				delete = true
				i += 1
			"--no-rivers":
				rivers = false
				i += 1
			_:
				print("ABORT: unrecognised argument %s" % args[i])
				get_tree().quit(2)
				return
	if width <= 0 or out == "":
		print("ABORT: --width and --out are required")
		get_tree().quit(2)
		return
	get_tree().create_timer(3600.0).timeout.connect(func() -> void:
		print("\n==== WATCHDOG: the probe did not finish ====\n")
		get_tree().quit(2))
	bridge = load("res://shell/engine_bridge.gd").new()
	add_child(bridge)
	await get_tree().process_frame

	var tg := Time.get_ticks_msec()
	bridge.world_gen.generate_sized(seed_value, 1200.0, 2048, 1311)
	bridge.has_world = true
	print("  seed %d generated 2048 x 1311 in %.1f s" % [seed_value, (Time.get_ticks_msec() - tg) / 1000.0])
	print("  GENERATED -- starting the %s export now" % ("monolithic" if mono else "banded"))
	var t0 := Time.get_ticks_msec()
	var r: Dictionary = (bridge.world_gen.export_raster_png(out, width, false) if mono
		else bridge.world_gen.export_image(out, {"width": width, "content": {"rivers": rivers}}))
	var wall := (Time.get_ticks_msec() - t0) / 1000.0
	if not bool(r.get("ok", false)):
		print("  REFUSED/FAILED at %d: %s" % [width, String(r.get("error", "?"))])
		get_tree().quit(1)
		return
	print("  RUN %s %d -> %d x %d  %d bytes  bands %d  %.1f s wall"
		% ["MONO" if mono else "BAND", width, int(r.get("width", 0)), int(r.get("height", 0)), _size(out),
			int(r.get("bands", 1)), wall])
	if delete:
		DirAccess.remove_absolute(out)
	get_tree().quit(0)
