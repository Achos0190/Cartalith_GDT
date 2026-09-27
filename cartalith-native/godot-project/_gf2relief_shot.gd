extends Node
## GF-2 (`GEOLOGY_FIRST_SCOPE.md` §7): the same two worlds rendered before and
## after stream power starts reading rock, so "does relief now track rock" is
## answered by looking, beside the harness's numbers.
##
## MUST run WINDOWED -- `ImageTexture.update()` is a no-op under `--headless`
## (`MISTAKES.md`), so a headless run could save stale or empty pixels:
##   Godot_v4.7.1-stable_win64_console.exe --path . _gf2relief_shot.tscn -- --tag before
##
## `--tag NAME` (required) names the output files so a before and an after
## set sit side by side. `--processes on|off` (optional, default off: the
## app's own world) sets `geology_processes` through `set_params` before each
## generation, and aborts if the key is rejected, so a shot labelled "on"
## cannot silently draw the app's gated-off world. GF-3 (§5.7) takes its
## before and after shots with `--processes on`:
##   Godot_v4.7.1-stable_win64_console.exe --path . _gf2relief_shot.tscn -- --tag gf3_before --processes on
## `--age TAU` (optional, GF-7, `GEOLOGY_FIRST_SCOPE.md` §4.12) sets the
## geological age `geo.age` the same way, and aborts if the key is rejected or
## clamped, so a shot labelled with an age is that age. Omitted, it is left at
## the default 1.0. It acts only with `--processes on` (the clock is gated
## with the processes), which is why §5.8's shots pass both:
##   Godot_v4.7.1-stable_win64_console.exe --path . _gf2relief_shot.tscn -- --tag gf7_age4 --processes on --age 4
## An unknown argument aborts (exit 2) rather than being ignored
## (`MISTAKES.md`, "Write a probe's usage header").
## Held fixed across tags: seeds, extent, grid, the app's default parameters
## (plus the `--processes` value). The other variable is the engine build the
## probe loads. The GF-2 processes are gated off in `params::defaults()`
## (`geology_processes`, `GEOLOGY_FIRST_SCOPE.md` §5.6), so without
## `--processes on` a current build draws the GF-1 world; GF-2's recorded
## "after" shots were taken before that gate, with them on.
##
## Exit status: 0 both shots written; 1 a shot failed; 2 could not run.

## The GF-0 harness's first and last seeds (§5.1).
const SEEDS: Array[int] = [483920, 314159]
## §5.2's bars apply at 800 km.
const WIDTH_KM := 800.0
## The app's default grid (`engine_bridge.gd`: 2048 and round(2048 x 0.64)).
const GRID_W := 2048
const GRID_H := 1311

func _ready() -> void:
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load"); get_tree().quit(2); return

	var tag := ""
	var processes := false
	## -1.0 = "not given": outside `geo.age`'s 0.25..4 range, so never a real
	## age; the probe then leaves the parameter alone rather than writing a
	## value nobody asked for.
	var age := -1.0
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		if args[i] == "--tag" and i + 1 < args.size():
			tag = args[i + 1]
			i += 2
		elif args[i] == "--processes" and i + 1 < args.size() and args[i + 1] in ["on", "off"]:
			processes = args[i + 1] == "on"
			i += 2
		elif args[i] == "--age" and i + 1 < args.size() and args[i + 1].is_valid_float():
			age = float(args[i + 1])
			i += 2
		else:
			print("[FATAL] unknown argument %s (read: --tag NAME, --processes on|off, --age TAU)" % args[i])
			get_tree().quit(2); return
	if tag == "":
		print("[FATAL] --tag NAME is required"); get_tree().quit(2); return

	var out_dir := "user://gf2relief"
	DirAccess.make_dir_recursive_absolute(out_dir)
	var failed := false
	for sd in SEEDS:
		var wg: Object = ClassDB.instantiate("WorldGen")
		# Set explicitly either way, so the shot's world is the one its flag
		# names; a rejected key aborts rather than drawing the default world.
		var rep: Dictionary = wg.set_params({"geology_processes": processes})
		if not (rep.get("rejected", PackedStringArray()) as PackedStringArray).is_empty():
			print("[FATAL] set_params rejected geology_processes: %s" % [rep])
			get_tree().quit(2); return
		if age >= 0.0:
			var rep_age: Dictionary = wg.set_params({"geo.age": age})
			if not (rep_age.get("rejected", PackedStringArray()) as PackedStringArray).is_empty() \
					or not (rep_age.get("clamped", PackedStringArray()) as PackedStringArray).is_empty():
				print("[FATAL] set_params did not take geo.age %s as given: %s" % [age, rep_age])
				get_tree().quit(2); return
		wg.generate_sized(sd, WIDTH_KM, GRID_W, GRID_H)
		var tex: Texture2D = wg.build_color_texture()
		if tex == null:
			print("[FAIL] no texture for seed %d" % sd)
			failed = true
			continue
		var img: Image = tex.get_image()
		if img == null or img.get_width() != GRID_W:
			print("[FAIL] seed %d: image missing or not %d wide" % [sd, GRID_W])
			failed = true
			continue
		var path := "%s/%s_%d.png" % [out_dir, tag, sd]
		img.save_png(path)
		print("[SHOT] %s seed %d processes %s age %s -> %s" % [tag, sd, "on" if processes else "off",
			("%.2f" % age) if age >= 0.0 else "default", ProjectSettings.globalize_path(path)])
	get_tree().quit(1 if failed else 0)
