extends Node
## Bisects `_cull_probe.gd`'s 13 mixed-network FAILs (see
## `_dashresidualcheck_probe.gd`) by way-type family: does the residual
## reproduce with ONLY dashed way types present (road/track/ancient), and is
## it absent with ONLY solid ones (highway/regional) -- the split
## `map_overlay.gd`'s own doc comment (`_segment_chains`, ~line 2762) already
## claims from a single bare-`draw_polyline` isolation, checked here instead
## through the full `_run_offscreen`/`_segment_chains`/`_draw_dashed_polyline`
## pipeline on `_cull_probe.gd`'s own network shape and the same on/off idiom.
##
## Same two-arm idiom as `_cull_probe.gd`: shipping script vs a subclass whose
## only difference is `_visible_local_rect()` returning everything (so
## `_segment_chains` sees an effectively infinite view and folds every way to
## one whole-run chain, same as `_NoCull` there). Windowed -- pixel probe.
##
##   Godot_v4.7.1-stable_win64.exe --path . _dashvssolid_probe.tscn

const W := 900
const H := 600

## Identical construction to `_cull_probe.gd::_roads()` (same seed, same
## per-type loop), filtered to one family so the two calls are otherwise the
## same fixture split in two.
static func _roads(types: Array) -> Array:
	var rng := RandomNumberGenerator.new()
	rng.seed = 20260825
	var out := []
	for t in ["highway", "regional", "road", "track", "ancient"]:
		for w in 10:
			var pts := PackedVector2Array()
			var x := rng.randf_range(0.0, 200.0)
			var y := rng.randf_range(0.0, 600.0)
			for s in 30:
				pts.append(Vector2(x, y))
				x += rng.randf_range(8.0, 34.0)
				y += rng.randf_range(-22.0, 22.0)
			if t in types:
				out.append({"points": pts, "brks": PackedInt32Array([13]), "way_type": t,
					"name": "w", "km": 100.0, "manual": false})
	return out


class _NoCull extends "res://map_overlay.gd":
	func _visible_local_rect() -> Rect2:
		return Rect2(-1e9, -1e9, 2e9, 2e9)


func _p(s: String) -> void:
	print("DASHVSSOLID  %s" % s)


func _pixel_diff(a: PackedByteArray, b: PackedByteArray) -> Dictionary:
	var max_delta := 0
	var diff_pixels := 0
	var n := a.size()
	var i := 0
	while i < n:
		var pixel_differs := false
		for c in 4:
			var d := absi(int(a[i + c]) - int(b[i + c]))
			if d > 0:
				pixel_differs = true
				if d > max_delta:
					max_delta = d
		if pixel_differs:
			diff_pixels += 1
		i += 4
	return {"max_delta": max_delta, "diff_pixels": diff_pixels}


func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 240.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		_p("WATCHDOG -- probe did not finish")
		get_tree().quit(2))
	wd.start()

	if DisplayServer.get_name() == "headless":
		_p("ABORT: pixel probe, needs the windowed binary -- see header")
		get_tree().quit(2)
		return

	for family in [{"tag": "SOLID-ONLY", "types": ["highway", "regional"]},
			{"tag": "DASHED-ONLY", "types": ["road", "track", "ancient"]}]:
		await _run_family(family["tag"], family["types"])
	_p("DONE")
	get_tree().quit(0)


func _run_family(tag: String, types: Array) -> void:
	var roads := _roads(types)
	var vps := []
	var cams := []
	var ovs := []
	for script in [load("res://map_overlay.gd"), _NoCull]:
		var vp := SubViewport.new()
		vp.size = Vector2i(W, H)
		vp.transparent_bg = false
		vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
		add_child(vp)
		var bg := ColorRect.new()
		bg.size = Vector2(W, H)
		bg.color = Color(0.08, 0.07, 0.06)
		vp.add_child(bg)
		var cam := Node2D.new()
		vp.add_child(cam)
		var ov := Control.new()
		ov.set_script(script)
		ov.size = Vector2(W, H)
		cam.add_child(ov)
		ov.set_civ_data([], roads, [], W, H, 0.0)
		vps.append(vp)
		cams.append(cam)
		ovs.append(ov)

	var blank_vp := SubViewport.new()
	blank_vp.size = Vector2i(W, H)
	blank_vp.transparent_bg = false
	blank_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(blank_vp)
	var blank_bg := ColorRect.new()
	blank_bg.size = Vector2(W, H)
	blank_bg.color = Color(0.08, 0.07, 0.06)
	blank_vp.add_child(blank_bg)
	for f in 3:
		await get_tree().process_frame
	await RenderingServer.frame_post_draw
	var blank: PackedByteArray = blank_vp.get_texture().get_image().get_data()

	var inked := 0
	var exact_count := 0
	var diff_count := 0
	var worst_delta := 0
	var worst_frac := 0.0
	## Same case set `_cull_probe.gd` uses -- the exact pans/zooms its own 13
	## fails were measured at.
	for z in [1.0, 2.0, 4.0, 8.0]:
		for pan in [Vector2.ZERO, Vector2(-400, -260), Vector2(-1600, -900), Vector2(300, 180)]:
			for i in 2:
				cams[i].scale = Vector2(z, z)
				cams[i].position = pan
				ovs[i].set_camera_zoom(z)
				ovs[i].queue_redraw()
			for f in 3:
				await get_tree().process_frame
			await RenderingServer.frame_post_draw
			var a: Image = vps[0].get_texture().get_image()
			var b: Image = vps[1].get_texture().get_image()
			var data_a := a.get_data()
			var data_b := b.get_data()
			var exact: bool = data_a == data_b
			var ink: bool = data_a != blank
			if ink:
				inked += 1
			if exact:
				exact_count += 1
			else:
				diff_count += 1
				var diff := _pixel_diff(data_a, data_b)
				var frac := float(diff["diff_pixels"]) / float(W * H)
				if diff["max_delta"] > worst_delta or (diff["max_delta"] == worst_delta and frac > worst_frac):
					worst_delta = diff["max_delta"]
					worst_frac = frac
				_p("  %-11s z%.0f pan(%5d,%5d) DIFFER  max_delta=%3d diff_px=%6d (%.4f%%)"
					% [tag, z, pan.x, pan.y, diff["max_delta"], diff["diff_pixels"], frac * 100.0])
	_p("%-11s: %d/16 exact, %d/16 differ, %d/16 inked -- worst max_delta=%d (%.4f%% px)"
		% [tag, exact_count, diff_count, inked, worst_delta, worst_frac * 100.0])

	for n in [blank_vp] + vps:
		n.queue_free()
	for n in cams:
		n.queue_free()
