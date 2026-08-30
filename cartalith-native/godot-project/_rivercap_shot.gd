extends Node
## Renders the same world at several map extents so a river-width cap can be
## chosen by looking rather than by arguing.
##
##   godot --headless --path . _rivercap_shot.tscn -- --tag cap16
##
## `--tag` names the output files, so three builds with three different caps
## produce three comparable sets. Everything else is held fixed: same seed,
## same grid, same params. The ONLY variable across a set is `map_width_km`,
## and across sets it is the cap.
##
## At and below 50 km both `terrain_detail_k` and `river_coarse_ease` are
## already saturated, so the river NETWORK is identical at 1/5/10/50 km --
## which is exactly what makes this a clean test of width alone.

const EXTENTS: Array[float] = [50.0, 10.0, 5.0, 1.0]
const GRID_W := 384
const GRID_H := 288

func _ready() -> void:
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load"); get_tree().quit(1); return

	var tag := "untagged"
	var args := OS.get_cmdline_user_args()
	for i in args.size():
		if args[i] == "--tag" and i + 1 < args.size():
			tag = args[i + 1]

	var out_dir := "user://rivercap"
	DirAccess.make_dir_recursive_absolute(out_dir)

	for km in EXTENTS:
		var wg: Object = ClassDB.instantiate("WorldGen")
		wg.set_params({"tect.plates": 9})
		wg.generate_sized(24601, km, GRID_W, GRID_H)
		var tex: Texture2D = wg.build_color_texture()
		if tex == null:
			print("[FAIL] no texture at %.0f km" % km)
			continue
		var img: Image = tex.get_image()
		var name := "%s/%s_%04dkm.png" % [out_dir, tag, int(km)]
		img.save_png(name)
		## Count river-tinted pixels the same way _riverwidth_probe does, so
		## the picture comes with a number rather than only an impression.
		var n := 0
		for y in img.get_height():
			for x in img.get_width():
				var c := img.get_pixel(x, y)
				if c.b > c.r + 0.22 and c.b > 0.45 and c.g > c.r:
					n += 1
		print("[SHOT] %-8s %5.0f km -> %6d river px  %s"
			% [tag, km, n, ProjectSettings.globalize_path(name)])

	get_tree().quit(0)
