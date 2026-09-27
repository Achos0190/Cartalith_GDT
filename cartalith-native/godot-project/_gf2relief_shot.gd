extends Node
## GF-2 (`GEOLOGY_FIRST_SCOPE.md` §7): the same two worlds rendered before and
## after stream power starts reading rock, so "does relief now track rock" is
## answered by looking, beside the harness's numbers.
##
## MUST run WINDOWED -- `ImageTexture.update()` is a no-op under `--headless`
## (`MISTAKES.md`), so a headless run could save stale or empty pixels:
##   Godot_v4.7.1-stable_win64_console.exe --path . _gf2relief_shot.tscn -- --tag before
##
## `--tag` is the only argument read, and names the output files so a before
## and an after set sit side by side. An unknown argument aborts (exit 2)
## rather than being ignored (`MISTAKES.md`, "Write a probe's usage header").
## Held fixed across tags: seeds, extent, grid, the app's default parameters.
## The only variable is the engine build the probe loads. Since the GF-2
## processes are gated off in `params::defaults()` (`geology_processes`,
## `GEOLOGY_FIRST_SCOPE.md` §5.6), a current build draws the GF-1 world; the
## recorded "after" shots were taken before that gate, with them on.
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
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		if args[i] == "--tag" and i + 1 < args.size():
			tag = args[i + 1]
			i += 2
		else:
			print("[FATAL] unknown argument %s (only --tag NAME is read)" % args[i])
			get_tree().quit(2); return
	if tag == "":
		print("[FATAL] --tag NAME is required"); get_tree().quit(2); return

	var out_dir := "user://gf2relief"
	DirAccess.make_dir_recursive_absolute(out_dir)
	var failed := false
	for sd in SEEDS:
		var wg: Object = ClassDB.instantiate("WorldGen")
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
		print("[SHOT] %s seed %d -> %s" % [tag, sd, ProjectSettings.globalize_path(path)])
	get_tree().quit(1 if failed else 0)
