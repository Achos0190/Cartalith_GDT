extends Node
## Ruling AN (OUTSTANDING_WORK.md §20): the three whole-raster correction
## stages -- local contrast, the colour grade, the output colour space -- run
## as ONE pass with one quantisation (render::finish_rgb). This drives the real
## screen path (build_color_texture) and the real export path
## (export_raster_png) on a generated world under the shipped default and under
## a graded wide-gamut look, and writes each to disk so the same probe run on
## the pre-AN build gives a before/after pair to diff.
##
## Windowed, not headless: it reads texture pixels, and ImageTexture.update is
## a no-op under --headless (MISTAKES.md, "Run a pixel probe").
##
##   godot --path . _finishpass_probe.tscn -- --tag after
##
## `--tag` names the output files; any other argument is refused.
## Committed, like every probe scene in this folder.

var bridge: Node
var fails := 0

func _ok(cond: bool, what: String) -> void:
	print(("  PASS  %s" if cond else "  FAIL  %s") % what)
	if not cond:
		fails += 1

func _distinct(img: Image) -> int:
	var seen := {}
	for i in range(4000):
		seen[img.get_pixel(i * 7919 % img.get_width(), i * 104729 % img.get_height()).to_rgba32()] = true
	return seen.size()

func _ready() -> void:
	var tag := ""
	var args := OS.get_cmdline_user_args()
	var k := 0
	while k < args.size():
		if args[k] == "--tag" and k + 1 < args.size():
			tag = args[k + 1]
			k += 2
		else:
			push_error("unknown argument %s" % args[k])
			get_tree().quit(2)
			return
	if tag == "":
		push_error("--tag <name> is required")
		get_tree().quit(2)
		return
	bridge = load("res://shell/engine_bridge.gd").new()
	add_child(bridge)
	await get_tree().process_frame
	var dir := ProjectSettings.globalize_path("user://_finishpass_probe")
	DirAccess.make_dir_recursive_absolute(dir)
	print("  scratch: %s" % dir)

	bridge.world_gen.generate_sized(20260824, 1200.0, 512, 328)
	bridge.has_world = true

	var shots := {}
	for case in [["default", "Quality tier", "sRGB"], ["antique_p3", "Antique Parchment", "Display P3"]]:
		_ok(bool(bridge.world_gen.set_look(case[1])), "look %s is offered" % case[1])
		_ok(bool(bridge.world_gen.set_color_space(case[2])), "space %s is offered" % case[2])
		var tex: ImageTexture = bridge.world_gen.build_color_texture()
		_ok(tex != null, "%s: build_color_texture returns a texture" % case[0])
		if tex == null:
			continue
		var img := tex.get_image()
		img.convert(Image.FORMAT_RGB8)
		shots[case[0]] = img
		var d := _distinct(img)
		_ok(d > 500, "%s: the screen raster is a real picture (%d distinct colours in 4000 samples)" % [case[0], d])
		img.save_png(dir.path_join("%s_screen_%s.png" % [tag, case[0]]))
		var r: Dictionary = bridge.world_gen.export_raster_png(dir.path_join("%s_export_%s.png" % [tag, case[0]]), 2048, false)
		_ok(bool(r.get("ok", false)), "%s: the 2K export runs (%s)" % [case[0], r.get("error", "")])
	# Positive control: the graded wide-gamut look really moves the screen.
	if shots.size() == 2:
		_ok(shots["default"].get_data() != shots["antique_p3"].get_data(), "positive control -- the graded P3 look moves the screen raster")
	bridge.world_gen.set_look("Quality tier")
	bridge.world_gen.set_color_space("sRGB")
	print("\n==== finish pass: %s ====\n" % ["OK" if fails == 0 else "SEE ABOVE"])
	get_tree().quit(1 if fails > 0 else 0)
