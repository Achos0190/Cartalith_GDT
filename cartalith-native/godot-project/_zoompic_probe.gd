extends SceneTree
## **Pin pictograms at deep zoom, as the map draws them** (zoom-regression fix, 2026-11-23): every mapped kind's pin is drawn at map
## coordinates near (831, 643) on a canvas scaled x39, the numbers the overlay hands `PinMarker.draw` at the deepest notch of
## `_zoomcost_probe` (head radius 14 screen px = 0.36 local). Before the fix the engine logged `Invalid polygon data,
## triangulation failed` for the small cut and fill polygons (house door, hut) and did not draw them: run this on the tree before
## and after and compare the PNG and the count of those lines in the log. MUST run windowed (a SubViewport draws nothing headless):
##   godot --path . --script res://_zoompic_probe.gd -- --out DIR
## Prints `zoompic: <n> pins drawn` and writes `DIR/zoompic.png`. Exit 0 whatever it saw (a measurement; the error count is the
## engine's own log line, counted by whoever runs it).
const Pin := preload("res://shell/pin_marker.gd")
const Z := 39.0
const BASE := Vector2(831.0, 643.0)

class Canvas extends Control:
	func _draw() -> void:
		var kinds: Array = Pin.KIND_PICTOGRAM.keys()
		for i in kinds.size():
			var tip := BASE + Vector2(float(i % 12) * 0.62, float(i / 12) * 0.8 + 0.5)
			Pin.draw(self, tip, 14.0 / Z, String(kinds[i]), false, 1.0, 1.0, Pin.INK, 0.0, "", Color(0.85, 0.7, 0.3), Color(0.1, 0.1, 0.1),
				1.0 / Z, 0.0)

func _initialize() -> void:
	var out := "."
	var args := OS.get_cmdline_user_args()
	for i in args.size():
		if args[i] == "--out" and i + 1 < args.size():
			out = args[i + 1]
	var vp := SubViewport.new()
	vp.size = Vector2i(560, 220)
	vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	root.add_child(vp)
	var bg := ColorRect.new()
	bg.color = Color(0.2, 0.3, 0.35)
	bg.size = Vector2(560, 220)
	vp.add_child(bg)
	var c := Canvas.new()
	c.scale = Vector2(Z, Z)
	c.position = -BASE * Z + Vector2(20, 10)
	c.size = Vector2(2000, 2000)
	vp.add_child(c)
	for k in 6:
		await process_frame
	DirAccess.make_dir_recursive_absolute(out)
	vp.get_texture().get_image().save_png(out.path_join("zoompic.png"))
	print("zoompic: %d pins drawn" % Pin.KIND_PICTOGRAM.size())
	quit(0)
