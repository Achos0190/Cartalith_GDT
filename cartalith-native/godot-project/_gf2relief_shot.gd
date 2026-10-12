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
## `--relief on|off` (optional, GF-13, `GEOLOGY_FIRST_SCOPE.md` §7 GF-13 and
## §4.16) sets `relief_model`, rock-aware construction (GF-11), through
## `set_params` before each generation and aborts if the key is rejected, so a
## shot labelled "relief on" cannot silently draw a world without it. Omitted,
## the parameter is left at the build's default (off in the app until the
## owner's switch-on decision). It acts only with `--processes on`, so GF-13's
## arm C passes both:
##   Godot_v4.7.1-stable_win64_console.exe --path . _gf2relief_shot.tscn -- --tag gf13_c1 --processes on --relief on
## `--zoom-at X,Y` (optional, repeatable, GF-13's deep-zoom frames) also writes,
## per seed, a `--zoom-size`-pixel square (default 512) of the world around
## grid cell (X, Y), `--zoom-radius` cells each side (default 32, so 65 cells
## across 512 px, about 7.9 px per cell against the fit image's 1). It is drawn
## by `WorldGen::export_snapshot_png`, the export's own `bake_rect` over the
## same fields: a magnified render of the generated grid, NOT the app's
## deep-zoom LOD tiles (which add synthesized sub-cell detail). The crop
## centres are given, never picked by the probe, so every arm is cropped at
## the same cells. A crop centre is a cell of one world, so pass `--seed N` (one
## of `SEEDS`) with it; `--seed` alone also restricts the fit shots to that seed.
## `--coastal on|off` (optional, GF-4, Ruling CH addendum) sets `passes.coastal`
## the same way, so the tidal marsh's accretion-only change can be seen on the
## app's own world with the coastal pass forced on (it is off in the app):
##   Godot_v4.7.1-stable_win64_console.exe --path . _gf2relief_shot.tscn -- --tag marsh_after --coastal on --seed 314159
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
	## "" = `--relief` not given: the build's default is left alone. Kept apart
	## from "off" so an omitted flag never writes a value nobody asked for.
	var relief := ""
	## "" = `--coastal` not given: the build's default is left alone.
	var coastal := ""
	## Deep-zoom crop centres in grid cells (`--zoom-at X,Y`, repeatable).
	var zooms: Array[Vector2i] = []
	var zoom_radius := 32
	var zoom_size := 512
	## `--seed N` (one of `SEEDS`): shoot that seed only, so a crop centre,
	## which is a cell of one world, is not applied to the other. -1 = both.
	var only_seed := -1
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		if args[i] == "--tag" and i + 1 < args.size():
			tag = args[i + 1]
			i += 2
		elif args[i] == "--coastal" and i + 1 < args.size() and args[i + 1] in ["on", "off"]:
			coastal = args[i + 1]
			i += 2
		elif args[i] == "--relief" and i + 1 < args.size() and args[i + 1] in ["on", "off"]:
			relief = args[i + 1]
			i += 2
		elif args[i] == "--zoom-at" and i + 1 < args.size() and args[i + 1].split(",").size() == 2 \
				and args[i + 1].split(",")[0].is_valid_int() and args[i + 1].split(",")[1].is_valid_int():
			var xy := args[i + 1].split(",")
			zooms.append(Vector2i(int(xy[0]), int(xy[1])))
			i += 2
		elif args[i] == "--seed" and i + 1 < args.size() and args[i + 1].is_valid_int() and int(args[i + 1]) in SEEDS:
			only_seed = int(args[i + 1])
			i += 2
		elif args[i] == "--zoom-radius" and i + 1 < args.size() and args[i + 1].is_valid_int() and int(args[i + 1]) > 0:
			zoom_radius = int(args[i + 1])
			i += 2
		elif args[i] == "--zoom-size" and i + 1 < args.size() and args[i + 1].is_valid_int() and int(args[i + 1]) > 0:
			zoom_size = int(args[i + 1])
			i += 2
		elif args[i] == "--processes" and i + 1 < args.size() and args[i + 1] in ["on", "off"]:
			processes = args[i + 1] == "on"
			i += 2
		elif args[i] == "--age" and i + 1 < args.size() and args[i + 1].is_valid_float():
			age = float(args[i + 1])
			i += 2
		else:
			print("[FATAL] unknown argument %s (read: --tag NAME, --processes on|off, --age TAU, --relief on|off, --coastal on|off, --seed N, --zoom-at X,Y, --zoom-radius N, --zoom-size PX)" % args[i])
			get_tree().quit(2); return
	if tag == "":
		print("[FATAL] --tag NAME is required"); get_tree().quit(2); return

	var out_dir := "user://gf2relief"
	DirAccess.make_dir_recursive_absolute(out_dir)
	var failed := false
	for sd in SEEDS:
		if only_seed >= 0 and sd != only_seed:
			continue
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
		if relief != "":
			var rep_rel: Dictionary = wg.set_params({"relief_model": relief == "on"})
			if not (rep_rel.get("rejected", PackedStringArray()) as PackedStringArray).is_empty():
				print("[FATAL] set_params rejected relief_model: %s" % [rep_rel])
				get_tree().quit(2); return
		if coastal != "":
			var rep_c: Dictionary = wg.set_params({"passes.coastal": coastal == "on"})
			if not (rep_c.get("rejected", PackedStringArray()) as PackedStringArray).is_empty():
				print("[FATAL] set_params rejected passes.coastal: %s" % [rep_c])
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
		print("[SHOT] %s seed %d processes %s age %s relief %s -> %s" % [tag, sd, "on" if processes else "off",
			("%.2f" % age) if age >= 0.0 else "default", relief if relief != "" else "default",
			ProjectSettings.globalize_path(path)])
		# Deep-zoom crops at the given cells (header). A crop the binding
		# refuses fails the run rather than leaving a gap in a before/after set.
		for z in zooms:
			var zpath := "%s/%s_%d_z%d_%d.png" % [out_dir, tag, sd, z.x, z.y]
			var zr: Dictionary = wg.export_snapshot_png(ProjectSettings.globalize_path(zpath), z.x, z.y, zoom_radius, zoom_size)
			if not bool(zr.get("ok", false)):
				print("[FAIL] seed %d zoom at %s: %s" % [sd, z, zr])
				failed = true
				continue
			print("[ZOOM] %s seed %d at %d,%d (centre used %s,%s; %s cells across %d px) -> %s" % [tag, sd, z.x, z.y,
				zr.get("center_x"), zr.get("center_y"), zr.get("cells_across"), zoom_size,
				ProjectSettings.globalize_path(zpath)])
	get_tree().quit(1 if failed else 0)
