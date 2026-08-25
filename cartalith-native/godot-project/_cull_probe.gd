extends Node
## The runnable check behind `_run_offscreen()` (2026-08-25).
##
##   Godot_v4.7.1-stable_win64.exe --path . _cull_probe.tscn
##
## `map_overlay.gd` now skips a way, sea lane or route whose whole run falls
## outside the window. The claim is that this is free: what it skips is outside
## the viewport, which discarded it anyway. That claim is only worth as much as
## a frame comparison, so this renders the real shipping script beside a
## subclass of it whose only difference is that `_visible_local_rect()` returns
## everything -- i.e. the same file with culling off -- and compares the two
## frames byte for byte.
##
## Driven through a `Node2D` camera ancestor at four zooms and four pans,
## because that ancestor scale is exactly what the shipping code had no way of
## seeing before `_run_offscreen()` existed. Reports the drawn-object count for
## each arm alongside, which is the whole point of the change.

const W := 900
const H := 600

## A way that leaves the window at zoom 1 and a network dense enough that a
## bounding box test has something to reject.
static func _roads() -> Array:
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
			out.append({"points": pts, "brks": PackedInt32Array([13]), "way_type": t,
				"name": "w", "km": 100.0, "manual": false})
	return out


class _NoCull extends "res://map_overlay.gd":
	## Culling off, and nothing else changed. The comparison is this file
	## against itself.
	func _visible_local_rect() -> Rect2:
		return Rect2(-1e9, -1e9, 2e9, 2e9)


func _p(s: String) -> void:
	print("CULL  %s" % s)


var _vps := []
var _cams := []
var _ovs := []


func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 300.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		_p("WATCHDOG -- probe did not finish")
		get_tree().quit(2))
	wd.start()

	var roads := _roads()
	var routes := [{"render_points": roads[0]["points"], "render_brks": PackedInt32Array(),
		"points": roads[0]["points"], "brks": PackedInt32Array(), "name": "r", "km": 1.0}]
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
		ov.set_manual_routes(routes)
		_vps.append(vp)
		_cams.append(cam)
		_ovs.append(ov)

	var fails := 0
	for z in [1.0, 2.0, 4.0, 8.0]:
		for pan in [Vector2.ZERO, Vector2(-400, -260), Vector2(-1600, -900), Vector2(300, 180)]:
			for i in 2:
				_cams[i].scale = Vector2(z, z)
				_cams[i].position = pan
				_ovs[i].set_camera_zoom(z)
				_ovs[i].queue_redraw()
			for f in 3:
				await get_tree().process_frame
			await RenderingServer.frame_post_draw
			var a: Image = _vps[0].get_texture().get_image()
			var b: Image = _vps[1].get_texture().get_image()
			var same: bool = a.get_data() == b.get_data()
			if not same:
				fails += 1
				a.save_png("user://cull_on_z%.0f_%d_%d.png" % [z, int(pan.x), int(pan.y)])
				b.save_png("user://cull_off_z%.0f_%d_%d.png" % [z, int(pan.x), int(pan.y)])
			_p("zoom %.0f pan (%5d,%5d) -> %s" % [z, pan.x, pan.y,
				"identical" if same else "DIFFERENT (captures in user://)"])

	_p("RESULT: %s" % ("PASS -- culling moves no pixel" if fails == 0
		else "FAIL -- %d frames differ" % fails))
	get_tree().quit(0 if fails == 0 else 1)
