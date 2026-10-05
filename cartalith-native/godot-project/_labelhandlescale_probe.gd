extends Node
## Ruling AP follow-up (2026-10-05). **Protects:** a "fixed" label's on-canvas
## handles keep their proportion to its box at every camera zoom, and the handle
## `tool_overlay.gd` DRAWS sits exactly where the engine's `label_handles` hit
## circle is -- a handle must be clickable where it is drawn.
##
## Before the fix the handle radius floors (4 / 6 cells, stem 10) were
## constants in grid cells, so at zoom 30 a fixed label's resize handle was
## 6.4 cells = ~192 px across around a box of ~26 px. After it the whole handle
## set is the zoom-1 set scaled by `1 / zoom`, i.e. constant on screen.
##
## Windowed only: `--headless` never composites the canvas, so every pixel
## assertion would pass vacuously. Run from a scratch copy of the project:
##
##   Godot_v4.7.1-stable_win64_console.exe --path <copy> _labelhandlescale_probe.tscn
##
## # Method
## A real `WorldGen`; the viewport equals the grid (`px_per_cell == 1`, so the
## disclosed cell/pixel conflation contributes nothing). One size-20 fixed label
## at the grid centre, the camera zoomed about the viewport centre exactly as
## `viewport_host.gd` does (a FULL_RECT parent whose `scale` is the zoom), and a
## real `ToolOverlay` in that camera fed the three `label_handles` circles -- the
## same call `cartography_workspace.gd::_update_label_handles_overlay` makes.
## At zoom 1, 8 and 30 the framebuffer is captured and:
##   (a) each handle's expected disc (engine x, y, r mapped through the camera)
##       must be >= 90 % handle-coloured inside (r - 1.5 px);
##   (b) the ring just outside it (r + 1.5 .. r + 4 px), minus any point near
##       another handle's disc, must be <= 5 % handle-coloured -- a drawn handle
##       bigger than the engine circle would fail here;
##   (c) the on-screen resize radius is the same at every zoom (+-12 %), which is
##       the scaling itself (an unscaled floor would be x8 and x30).
## Prints PROBE-FAIL for each failure; exit status is 0 regardless, so grep.

const OVERLAY := preload("res://map_overlay.gd")
const TOOL_OVERLAY := preload("res://shell/tool_overlay.gd")

const SEED := 90210
const GW := 512
const GH := 512
const VP := Vector2i(GW, GH)
const GROUND := Color(0.42, 0.44, 0.40)
## `tool_overlay.gd::HANDLE_COLOR` (rgb), composited at its own 0.95 alpha over GROUND.
const HANDLE_FILL := Color(0.549, 0.816, 1.0)

var _fails := 0


func _ok(cond: bool, what: String) -> void:
	print(("  PASS  " if cond else "  PROBE-FAIL  ") + what)
	if not cond:
		_fails += 1


func _is_handle(c: Color) -> bool:
	var want := HANDLE_FILL * 0.95 + GROUND * 0.05
	return absf(c.r - want.r) < 0.05 and absf(c.g - want.g) < 0.05 and absf(c.b - want.b) < 0.05


func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		printerr("PROBE-CANNOT-RUN: headless server; the canvas is never composited.")
		get_tree().quit(2)
		return
	DisplayServer.window_set_size(VP)
	await get_tree().process_frame

	var root := Control.new()
	root.set_anchors_preset(Control.PRESET_FULL_RECT)
	add_child(root)
	var bg := ColorRect.new()
	bg.set_anchors_preset(Control.PRESET_FULL_RECT)
	bg.color = GROUND
	root.add_child(bg)
	var cam := Control.new()
	cam.set_anchors_preset(Control.PRESET_FULL_RECT)
	root.add_child(cam)
	var ov := Control.new()
	ov.set_script(OVERLAY)
	ov.set_anchors_preset(Control.PRESET_FULL_RECT)
	cam.add_child(ov)
	var tov: Control = TOOL_OVERLAY.new()
	tov.set_anchors_preset(Control.PRESET_FULL_RECT)
	tov.overlay = ov
	cam.add_child(tov)
	await get_tree().process_frame

	var gen := WorldGen.new()
	gen.generate_sized(SEED, float(GW), GW, GH)
	var idx: int = gen.label_create(float(GW) / 2.0, float(GH) / 2.0, "Aldebar")
	gen.label_set(idx, {"size": 20.0, "size_mode": "fixed"})
	var lb: Dictionary = gen.label_get(idx)
	ov.set_civ_data([], [], [], GW, GH, 0.0)
	tov.set_grid(GW, GH)
	var ppc: float = ov.label_px_per_cell()
	_ok(absf(ppc - 1.0) < 1e-6, "px_per_cell == 1.0 (viewport matches the grid)")
	var s := Vector2(VP)
	var resize_px := {}
	for z in [1.0, 8.0, 30.0]:
		cam.scale = Vector2(z, z)
		cam.position = s * 0.5 - (s * 0.5) * z
		ov.set_camera_zoom(z)
		ov.set_labels([lb])
		var h: Dictionary = gen.label_handles(idx, z, ppc)
		var circles: Array = []
		for key in ["resize", "rotate", "arc"]:
			circles.append(h[key])
		tov.set_handles(circles)
		await get_tree().process_frame
		ov.queue_redraw()
		tov.queue_redraw()
		await RenderingServer.frame_post_draw
		await RenderingServer.frame_post_draw
		var img: Image = get_viewport().get_texture().get_image()
		var path := "user://labelhandlescale_z%d.png" % int(z)
		img.save_png(path)
		print("z=%.0f image: %s" % [z, ProjectSettings.globalize_path(path)])

		## Expected screen discs: grid -> local (`_point_to_screen`) -> camera.
		var rect: Rect2 = ov.displayed_rect()
		var cell_px: float = rect.size.x / float(GW) * z
		var discs: Array = []
		for c in circles:
			var local: Vector2 = ov._point_to_screen(Vector2(c["x"], c["y"]), rect)
			discs.append({"p": cam.position + local * z, "r": float(c["r"]) * cell_px})
		resize_px[z] = discs[0]["r"]
		var names := ["resize", "rotate", "arc"]
		for i in discs.size():
			var d: Dictionary = discs[i]
			var inside := 0
			var inside_n := 0
			var ring := 0
			var ring_n := 0
			var x0 := int(d["p"].x - d["r"] - 6.0)
			var y0 := int(d["p"].y - d["r"] - 6.0)
			var x1 := int(d["p"].x + d["r"] + 6.0)
			var y1 := int(d["p"].y + d["r"] + 6.0)
			for y in range(maxi(y0, 0), mini(y1, VP.y - 1) + 1):
				for x in range(maxi(x0, 0), mini(x1, VP.x - 1) + 1):
					var p := Vector2(x + 0.5, y + 0.5)
					var dist: float = p.distance_to(d["p"])
					var in_other := false
					for j in discs.size():
						if j != i and p.distance_to(discs[j]["p"]) <= float(discs[j]["r"]) + 4.0:
							in_other = true
					if dist <= float(d["r"]) - 1.5:
						inside_n += 1
						if _is_handle(img.get_pixel(x, y)):
							inside += 1
					elif dist >= float(d["r"]) + 1.5 and dist <= float(d["r"]) + 4.0 and not in_other:
						ring_n += 1
						if _is_handle(img.get_pixel(x, y)):
							ring += 1
			var in_frac := float(inside) / float(maxi(inside_n, 1))
			var ring_frac := float(ring) / float(maxi(ring_n, 1))
			print("z=%.0f %s: centre=(%.1f,%.1f) r=%.2fpx  inside-handle=%.2f (%d px)  ring-handle=%.3f (%d px)"
				% [z, names[i], d["p"].x, d["p"].y, d["r"], in_frac, inside_n, ring_frac, ring_n])
			_ok(inside_n >= 4 and in_frac >= 0.90, "z=%.0f %s: the engine circle is filled with the drawn handle" % [z, names[i]])
			_ok(ring_frac <= 0.05, "z=%.0f %s: nothing handle-coloured outside the engine circle" % [z, names[i]])
	_ok(absf(float(resize_px[8.0]) / float(resize_px[1.0]) - 1.0) < 0.12,
		"on-screen resize radius at z=8 matches z=1 (%.2f vs %.2f px)" % [resize_px[8.0], resize_px[1.0]])
	_ok(absf(float(resize_px[30.0]) / float(resize_px[1.0]) - 1.0) < 0.12,
		"on-screen resize radius at z=30 matches z=1 (%.2f vs %.2f px)" % [resize_px[30.0], resize_px[1.0]])
	print("PROBE-RESULT: ", "PASS" if _fails == 0 else "FAIL (%d)" % _fails)
	get_tree().quit(0)
