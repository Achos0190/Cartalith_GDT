extends Node
## Temporary, untracked verification harness for the zoom behaviour of the
## animated Wind / Ocean-currents streak overlay (`shell/wind_fx_layer.gd`) --
## the owner's 2026-08-25 report that it "doesn't scale with zoom ... doesn't
## show finer patterns", and that the tip should be an arrowhead rather than a
## square pixel.
##
## Non-headless by necessity: the complaint is about pixels. Hosted in a
## `SubViewport` because Windows clamps a real window to the 1680x1002 work
## area (`_hidpi_probe.gd`'s idiom).
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _flowzoom_probe.tscn -- --out DIR
##
## At each zoom step and for each of the two views it reports:
##   particles on screen, objects in frame, draw calls, and a PNG crop.
## Objects-in-frame is the number that matters twice over: the arrowheads must
## cost O(1) of it, not O(particles) -- the mistake `map_overlay.gd`'s dashed
## polylines made (311 237 objects in a frame).

const VP := Vector2i(1600, 1000)
const ZOOMS := [1.0, 2.0, 4.0, 8.0, 16.0]

var _vp: SubViewport
var _out := ""

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _shot() -> Image:
	await RenderingServer.frame_post_draw
	return _vp.get_texture().get_image()

## Mean per-pixel difference between two frames -- a static raster reads ~0.
func _diff(a: Image, b: Image) -> float:
	var da := a.get_data()
	var db := b.get_data()
	var n: int = mini(da.size(), db.size())
	var total := 0.0
	var i := 0
	while i < n:
		total += absf(float(da[i]) - float(db[i]))
		i += 7
	return total / float(maxi(1, n / 7))

## Particles whose *head* projects inside the host's own rect. This is the
## quantity the owner is actually looking at: streaks per screen, not per grid.
func _on_screen(fx, host) -> int:
	## From the parent, not from `fx`, so the same probe runs against the
	## pre-fix script (which has no `_rect` of its own).
	var rect: Rect2 = fx.get_parent().displayed_rect()
	if rect.size.x <= 0.0:
		return -1
	var xf: Transform2D = fx.get_global_transform()
	var scr: Rect2 = (host as Control).get_global_rect()
	var sx: float = rect.size.x / float(fx._fw)
	var sy: float = rect.size.y / float(fx._fh)
	var hit := 0
	for i in range(fx._px.size()):
		var p: Vector2 = xf * (rect.position + Vector2(fx._px[i] * sx, fx._py[i] * sy))
		if scr.has_point(p):
			hit += 1
	return hit

## Objects and draw calls attributable to *this layer*, measured as on-minus-off
## at one fixed zoom. A single global baseline will not do: deep-zoom LOD
## tiling swaps tens of thousands of objects in and out on its own as the
## camera moves, and swamps the few the streaks cost.
func _cost(host, view: String) -> Array:
	host.set_debug_layer("off")
	await _frames(6)
	await _shot()
	var off_o := Performance.get_monitor(Performance.RENDER_TOTAL_OBJECTS_IN_FRAME)
	var off_d := Performance.get_monitor(Performance.RENDER_TOTAL_DRAW_CALLS_IN_FRAME)
	host.set_debug_layer(view)
	await _frames(6)
	await _shot()
	var on_o := Performance.get_monitor(Performance.RENDER_TOTAL_OBJECTS_IN_FRAME)
	var on_d := Performance.get_monitor(Performance.RENDER_TOTAL_DRAW_CALLS_IN_FRAME)
	return [on_o, on_o - off_o, on_d, on_d - off_d]

func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var i := args.find("--out")
	_out = args[i + 1] if i >= 0 and i + 1 < args.size() else ProjectSettings.globalize_path("user://")

	_vp = SubViewport.new()
	_vp.size = VP
	_vp.transparent_bg = false
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)

	var app: Node = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await get_tree().create_timer(0.8).timeout

	var bridge = app.bridge
	bridge.generate({
		"seed": 483920, "width_km": 1200.0, "grid_w": 512, "grid_h": 384,
		"archetype": "", "villages": true, "sea_level": 0.42,
	})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(0.6).timeout
	app.open_project_dialog.hide()
	await _frames(4)

	var host = app.viewport
	var fx = host.overlay.get_node_or_null("WindFxLayer")
	print("FLOWZOOM overlay node present: ", fx != null, "  vp=", VP)

	for view in ["wind", "ocean"]:
		host.set_debug_layer(view)
		await _frames(6)
		for z in ZOOMS:
			host.reset_view()
			await _frames(3)
			host.zoom_step(z / maxf(host.zoom(), 0.0001))
			await get_tree().create_timer(0.5).timeout
			await _frames(4)
			var c: Array = await _cost(host, view)
			var a: Image = await _shot()
			await get_tree().create_timer(0.25).timeout
			var b: Image = await _shot()
			print(("FLOWZOOM %s zoom=%.2f | on-screen=%d of %d"
				+ " | objects=%d (layer %+d) draw_calls=%d (layer %+d)"
				+ " | motion=%.4f fps=%.0f")
				% [view, host.zoom(), _on_screen(fx, host), fx._px.size(),
					c[0], c[1], c[2], c[3], _diff(a, b),
					Engine.get_frames_per_second()])
			b.save_png("%s/flowzoom_%s_z%02d.png" % [_out, view, int(z)])
			## A tight crop around the viewport centre, where an arrowhead is
			## actually legible at 1:1 rather than 3 px in a 1600 px frame.
			var r: Rect2i = Rect2i(VP / 2 - Vector2i(200, 125), Vector2i(400, 250))
			var crop: Image = b.get_region(r)
			crop.resize(r.size.x * 3, r.size.y * 3, Image.INTERPOLATE_NEAREST)
			crop.save_png("%s/flowzoom_%s_z%02d_crop.png" % [_out, view, int(z)])
	print("FLOWZOOM wrote to ", _out)
	get_tree().quit()
