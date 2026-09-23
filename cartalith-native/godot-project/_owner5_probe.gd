extends Node
## Reproduction harness for the owner's five rendering reports of 2026-09-23
## (chaotic rivers, lake flicker on zoom, parallel lines, the Village preset
## sticking, settlements missing at zoom). Windowed only -- pixels.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . [--resolution 1680x1000] _owner5_probe.tscn -- --out DIR [--tag T] [MODE]
##
## MODE (one of; none = the zoom ladder 1..32 on a town):
##   --presets  zoom ladder, then Watercolor -> Village -> Natural Vibrant -> Default
##   --lake     fixed-ground crops of one lake shore across 2x..12x (flicker)
##   --town     largest interior town at 8..200x: urban-layout alpha/cache/reveal
##   --rivdbg   z=32 on the capital: river/road strokes on vs off, per-width
##   --checker  zero each appearance tunable in turn; lattice energy in a crop

var app
var bridge
var _out := ""
var _tag := "run"

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _shot(name: String) -> void:
	await get_tree().create_timer(0.6).timeout
	await _frames(4)
	await RenderingServer.frame_post_draw
	var img := get_viewport().get_texture().get_image()
	img.save_png("%s/%s_%s.png" % [_out, _tag, name])
	print("SHOT ", name)

func _find(n: Node, script_file: String) -> Node:
	if n.get_script() != null and String(n.get_script().resource_path).ends_with(script_file):
		return n
	for c in n.get_children(true):
		var r := _find(c, script_file)
		if r != null:
			return r
	return null

func _zoom_to(host, z: float, gx: float, gy: float) -> void:
	host.reset_view()
	await _frames(2)
	host.zoom_step(z / maxf(host.zoom(), 0.0001))
	await _frames(2)
	host.move_view_to(gx, gy)
	await get_tree().create_timer(1.5).timeout
	var guard := 0
	while host.lod_pending() > 0 and guard < 40:
		await get_tree().create_timer(0.2).timeout
		guard += 1

func _is_lake(gx: int, gy: int) -> bool:
	var d: Dictionary = bridge.sample_cell(gx, gy)
	return String(d.get("water", "land")) == "lake"

## A lake cell whose whole 7x7-step neighbourhood is also lake, then a fine
## zoom sweep over it: the owner's "lakes flicker on zoom".
func _lake_sweep(host, g: Vector2i) -> void:
	var best := Vector2i(-1, -1)
	for y in range(8, g.y - 8, 6):
		for x in range(8, g.x - 8, 6):
			if not _is_lake(x, y):
				continue
			var ok := true
			for dy in [-18, 0, 18]:
				for dx in [-18, 0, 18]:
					if not _is_lake(clampi(x + dx, 0, g.x - 1), clampi(y + dy, 0, g.y - 1)):
						ok = false
			if ok:
				best = Vector2i(x, y)
				break
		if best.x >= 0:
			break
	print("lake interior cell ", best)
	if best.x < 0:
		return
	## Walk north to the shore: the owner's flicker is at the edge, not mid-lake.
	var sy := best.y
	while sy > 0 and _is_lake(best.x, sy):
		sy -= 1
	var focus := Vector2(best.x, sy)
	print("shore focus ", focus)
	var z := 2.0
	var n := 0
	while z <= 12.0:
		await _zoom_to(host, z, focus.x, focus.y)
		await get_tree().create_timer(0.3).timeout
		await RenderingServer.frame_post_draw
		var img := get_viewport().get_texture().get_image()
		## Where the focus cell actually landed on screen, so every crop covers
		## the same patch of ground whatever the camera did.
		var ov = host.overlay
		var sp: Vector2 = ov.get_global_transform() * ov._cell_to_screen(focus, ov.displayed_rect())
		## A fixed 24-cell ground window around the focus.
		var px_per_cell: float = ov.displayed_rect().size.x / float(g.x) * host.zoom()
		var half := int(12.0 * px_per_cell)
		var r := Rect2i(Vector2i(sp) - Vector2i(half, half), Vector2i(half * 2, half * 2))
		r = r.intersection(Rect2i(Vector2i.ZERO, img.get_size()))
		var crop := img.get_region(r)
		crop.resize(360, 360, Image.INTERPOLATE_BILINEAR)
		print("LAKE z=%.2f lod=%s focus_px=%s crop=%s" % [host.zoom(), str(host.lod_active()), str(sp), str(r)])
		crop.save_png("%s/%s_shore_%02d.png" % [_out, _tag, n])
		n += 1
		z *= 1.15

func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var i := args.find("--out")
	_out = args[i + 1] if i >= 0 else ProjectSettings.globalize_path("user://")
	var t := args.find("--tag")
	if t >= 0:
		_tag = args[t + 1]
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.2).timeout
	bridge = app.bridge
	bridge.generate({
		"seed": 483920, "width_km": 1200.0, "grid_w": 1024, "grid_h": 768,
		"archetype": "", "villages": true, "sea_level": 0.42,
	})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	if app.open_project_dialog:
		app.open_project_dialog.hide()
	await _frames(6)
	var host = app.viewport
	var st: Array = bridge.settlements()
	print("settlements=", st.size())
	if st.size() > 0:
		print("keys=", (st[0] as Dictionary).keys())
	var g: Vector2i = bridge.grid_size()
	var cx := g.x * 0.5
	var cy := g.y * 0.5
	for s in st:
		var d: Dictionary = s
		if String(d.get("tier", d.get("kind", ""))).to_lower() in ["city", "capital", "town"]:
			cx = float(d.get("x", cx)); cy = float(d.get("y", cy))
			print("focus settlement ", d.get("name", "?"), " at ", cx, ",", cy)
			break
	if "--town" in args:
		var best: Dictionary = {}
		for s in st:
			if float(s["x"]) < 60 or float(s["x"]) > g.x - 60 or float(s["y"]) < 60 or float(s["y"]) > g.y - 60:
				continue
			if best.is_empty() or int(s["population"]) > int(best["population"]):
				best = s
		print("town ", best.get("name"), " pop=", best.get("population"), " kind=", best.get("kind"), " at ", best["x"], ",", best["y"])
		for z in [8.0, 32.0, 64.0, 128.0, 200.0]:
			await _zoom_to(host, z, float(best["x"]), float(best["y"]))
			await get_tree().create_timer(1.5).timeout
			var ov = host.overlay
			var rect: Rect2 = ov.displayed_rect()
			print("TOWN zoom=%.1f span_km=%.2f alpha=%.2f layouts_cached=%d revealed=%d show=%s" % [
				host.zoom(), ov._urban_span_km(rect), ov._urban_layout_alpha(rect),
				ov._urban_layouts.size(), ov._urban_revealed.size(), str(ov._show_urban_layouts)])
			await _shot("town_z%03d" % int(z))
		get_tree().quit()
		return
	if "--checker" in args:
		## Which appearance stage paints the lattice seen in deep-zoom tiles:
		## zero each non-zero tunable in turn and measure high-frequency energy
		## in a land crop around the focus.
		var focus := Vector2(398, 668)
		var base: Dictionary = bridge.appearance()
		var hf := func() -> float:
			await RenderingServer.frame_post_draw
			var im := get_viewport().get_texture().get_image()
			var c := Vector2i(1015, 560)
			var e := 0.0
			var n2 := 0
			for yy in range(c.y - 120, c.y + 120):
				for xx in range(c.x - 200, c.x + 200):
					var l0 := im.get_pixel(xx, yy).get_luminance()
					var l1 := im.get_pixel(xx + 1, yy + 1).get_luminance()
					var l2 := im.get_pixel(xx + 1, yy).get_luminance()
					var l3 := im.get_pixel(xx, yy + 1).get_luminance()
					e += absf(l0 + l1 - l2 - l3)
					n2 += 1
			return e / n2
		var z0 := 16.0
		await _zoom_to(host, z0, focus.x, focus.y)
		var e0: float = await hf.call()
		print("CHECK baseline hf=%.5f" % e0)
		get_viewport().get_texture().get_image().save_png("%s/checker_base.png" % _out)
		for key in base.keys():
			var v = base[key]
			if typeof(v) != TYPE_FLOAT or float(v) <= 0.0:
				continue
			bridge.set_appearance({key: 0.0})
			host.refresh()
			await _zoom_to(host, z0, focus.x, focus.y)
			var e1: float = await hf.call()
			print("CHECK %s %.3f->0 hf=%.5f (%+.1f%%)" % [key, float(v), e1, 100.0 * (e1 - e0) / e0])
			bridge.set_appearance({key: v})
		get_tree().quit()
		return
	if "--rivdbg" in args:
		var ov = host.overlay
		var fpos := Vector2(398, 678)
		for r in ov._rivers:
			var near := false
			for p in r["points"]:
				if p.distance_to(fpos) < 6.0:
					near = true
					break
			if near:
				print("NEAR river idx=%d order=%d own=%d width=%s parallel=%s pts=%d rpts=%d" % [int(r["index"]), int(r["order"]), int(r.get("own_order", -1)), str(r.get("width_cells", "none")), str(r.get("parallel_of", "-")), r["points"].size(), r["render_points"].size()])
		for z in [32.0]:
			await _zoom_to(host, z, fpos.x, fpos.y)
			var rect: Rect2 = ov.displayed_rect()
			ov._visible_local = ov._visible_local_rect()
			var k: float = maxf(ov._camera_zoom, 0.001)
			var ppc: float = rect.size.x / float(ov._gw) * k
			var drawn := 0
			var widths: Array = []
			for r in ov._rivers:
				if not r.has("width_cells") or r.has("parallel_of"):
					continue
				var sp: PackedVector2Array = ov._stroke_points(r["render_points"], 0, r["render_points"].size(), rect, k)
				var w: float = float(r["width_cells"]) * ppc
				if ov._run_offscreen(sp, k, w * 0.5):
					continue
				if ov._segment_chains(sp, k, w * 0.5).size() > 0:
					drawn += 1
					widths.append(w)
			widths.sort()
			print("RIV z=%.2f cam=%.2f lod=%s rivers=%d drawn=%d median_w=%.2f vis_local=%s show=%s dbg=%s" % [z, ov._camera_zoom, str(host.lod_active()),
				ov._rivers.size(), drawn, widths[widths.size() / 2] if widths.size() > 0 else -1.0,
				str(ov._visible_local), str(ov._show_rivers), str(ov._debug_active)])
			await RenderingServer.frame_post_draw
			var a := get_viewport().get_texture().get_image()
			ov.set_show_rivers(false)
			await _frames(3)
			await RenderingServer.frame_post_draw
			var b := get_viewport().get_texture().get_image()
			ov.set_show_rivers(true)
			await _frames(3)
			var diff := 0
			for yy in range(0, a.get_height(), 2):
				for xx in range(0, a.get_width(), 2):
					if a.get_pixel(xx, yy) != b.get_pixel(xx, yy):
						diff += 1
			print("RIVDIFF z=%.2f changed_px(1/4 sampled)=%d" % [z, diff])
			var all_r: Array = ov._rivers
			var inview: Array = []
			for r in all_r:
				if not r.has("width_cells") or r.has("parallel_of"):
					continue
				var sp2: PackedVector2Array = ov._stroke_points(r["render_points"], 0, r["render_points"].size(), rect, k)
				var w2: float = float(r["width_cells"]) * ppc
				if ov._run_offscreen(sp2, k, w2 * 0.5) or ov._segment_chains(sp2, k, w2 * 0.5).is_empty():
					continue
				inview.append(int(r["index"]))
				var minseg := 1e9
				var nan := false
				for qi in range(sp2.size() - 1):
					minseg = minf(minseg, sp2[qi].distance_to(sp2[qi + 1]))
					if is_nan(sp2[qi].x) or is_nan(sp2[qi].y):
						nan = true
				print("INVIEW %d order=%d rp=%d minseg_px=%.4f nan=%s chains=%s" % [int(r["index"]), int(r["order"]), sp2.size(), minseg, str(nan), str(ov._segment_chains(sp2, k, w2 * 0.5))])
			for pick in []:
				for r in all_r:
					if int(r["index"]) == pick:
						ov._rivers = [r]
				ov.queue_redraw()
				await _frames(3)
				await RenderingServer.frame_post_draw
				var c := get_viewport().get_texture().get_image()
				var d2 := 0
				for yy in range(0, c.get_height(), 2):
					for xx in range(0, c.get_width(), 2):
						if c.get_pixel(xx, yy) != b.get_pixel(xx, yy):
							d2 += 1
				var r0: Dictionary = ov._rivers[0]
				var rp: PackedVector2Array = r0["render_points"]
				var mn := Vector2(1e9, 1e9)
				var mx := Vector2(-1e9, -1e9)
				for q in rp:
					mn = mn.min(q); mx = mx.max(q)
				print("ONLY river %d: changed=%d rp_bbox=%s..%s colors=%d rp=%d first=%s" % [pick, d2, str(mn), str(mx), r0["colors"].size(), rp.size(), str(r0["colors"][0])])
				c.save_png("%s/only_%d.png" % [_out, pick])
			for scale_w in [0.05, 0.25, 0.5, 1.0]:
				var mod: Array = []
				for r in all_r:
					var r2: Dictionary = r.duplicate()
					if r2.has("width_cells"):
						r2["width_cells"] = float(r2["width_cells"]) * scale_w
					mod.append(r2)
				ov._rivers = mod
				ov.queue_redraw()
				await _frames(3)
				await RenderingServer.frame_post_draw
				var c2 := get_viewport().get_texture().get_image()
				## Road ink: count brownish pixels (roads are the only brown strokes here).
				var roadpx := 0
				for yy in range(200, 980, 2):
					for xx in range(380, 1400, 2):
						var px := c2.get_pixel(xx, yy)
						if px.r8 > 120 and px.r8 < 200 and px.g8 < 110 and px.b8 < 70:
							roadpx += 1
				print("WIDTHSCALE %.2f road_px=%d" % [scale_w, roadpx])
				c2.save_png("%s/wscale_%.2f.png" % [_out, scale_w])
			ov._rivers = []
			ov.queue_redraw()
			await _frames(3)
			await RenderingServer.frame_post_draw
			var c3 := get_viewport().get_texture().get_image()
			var roadpx0 := 0
			for yy in range(200, 980, 2):
				for xx in range(380, 1400, 2):
					var px := c3.get_pixel(xx, yy)
					if px.r8 > 120 and px.r8 < 200 and px.g8 < 110 and px.b8 < 70:
						roadpx0 += 1
			print("NORIVERS road_px=%d" % roadpx0)
			ov._rivers = all_r
			a.save_png("%s/rivdbg_z%.2f.png" % [_out, z])
			b.save_png("%s/rivdbg_off_z%.2f.png" % [_out, z])
		get_tree().quit()
		return
	if "--lake" in args:
		await _lake_sweep(host, g)
		get_tree().quit()
		return
	await _shot("z01_full")
	for z in [2.0, 4.0, 8.0, 16.0, 32.0]:
		await _zoom_to(host, z, cx, cy)
		print("zoom=", host.zoom(), " lod_active=", host.lod_active())
		await _shot("z%02d" % int(z))
	if "--presets" in args:
		app.select_domain_category("cartography", "Style")
		await _frames(8)
		var rw = _find(app, "render_workspace.gd")
		host.reset_view()
		await _frames(4)
		for idx in [4, 6, 0, 1]:  # Watercolor, Village, Natural Vibrant, Default
			rw._apply_preset(idx)
			await get_tree().create_timer(2.0).timeout
			await _shot("preset%d_%s" % [idx, String(RenderWorkspace.STYLE_PRESETS[idx][0])])
		print("appearance after Default: ", bridge.world_gen.appearance() if bridge.world_gen.has_method("appearance") else "n/a")
	get_tree().quit()
