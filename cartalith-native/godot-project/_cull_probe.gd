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
## seeing before `_run_offscreen()` existed.
##
## ## Two claims, and why pixel-equality alone is not one of them
##
## Byte-equality of two captures is satisfied *perfectly* by two blank frames,
## so on its own it is a test that cannot fail: if `set_civ_data()` were ever to
## stop reaching the overlay, or the script were to stop drawing, both arms
## would render the bare `ColorRect` and the probe would report PASS over zero
## coverage. Every case is therefore graded against a THIRD viewport holding the
## background and nothing else (`_vp_blank`), and a case whose culled arm is
## byte-identical to that blank frame is reported INK=0 -- evidence, not a pass.
##
##   1. **No pixel moves.** The shipping script against a subclass of itself
##      whose only difference is that `_visible_local_rect()` returns
##      everything, over every zoom/pan, byte for byte -- with at least one
##      case, and specifically the everything-on-screen baseline, proven to
##      have drawn ink.
##   2. **Something is actually skipped.** Phase 2 renders one arm per frame
##      and reads Godot's own `RENDER_TOTAL_OBJECTS_IN_FRAME`, which is the
##      number claim 1 is worthless without: culling that moves no pixel *and*
##      saves no object is culling that is not happening.

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
## The same viewport and the same `ColorRect`, with no overlay child at all --
## i.e. the exact framebuffer "nothing was drawn" produces. Held as a byte
## string rather than an Image so the per-case test is one `==`.
var _blank_vp: SubViewport
var _blank: PackedByteArray = PackedByteArray()
var _fails := 0


func _bad(s: String) -> void:
	_fails += 1
	_p("FAIL  %s" % s)


func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 300.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		_p("WATCHDOG -- probe did not finish")
		get_tree().quit(2))
	wd.start()

	## This probe measures PIXELS, and its documented invocation is the windowed
	## binary for that reason. Under `--headless` Godot loads the dummy display
	## driver: `RenderingServer.frame_post_draw` never fires, so the first
	## `_capture()` blocks forever and the run dies at the watchdog above having
	## printed nothing. Measured 2026-09-01 on both this probe and
	## `_cull_probe.gd`, and on the committed version of this one -- which is why
	## the watchdog is NOT the thing to raise: the run is not slow, it is stopped.
	## Said out loud here, because a silent 5-minute hang reads as "slow machine".
	if DisplayServer.get_name() == "headless":
		_p("ABORT: this probe measures pixels and cannot run headless -- "
			+ "RenderingServer.frame_post_draw never fires with the dummy driver. "
			+ "Re-run with the windowed binary, as the header shows:")
		_p("  Godot_v4.7.1-stable_win64.exe --path . _cull_probe.tscn")
		get_tree().quit(2)
		return


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

	## The negative control for claim 1: same size, same background, no overlay.
	_blank_vp = SubViewport.new()
	_blank_vp.size = Vector2i(W, H)
	_blank_vp.transparent_bg = false
	_blank_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_blank_vp)
	var blank_bg := ColorRect.new()
	blank_bg.size = Vector2(W, H)
	blank_bg.color = Color(0.08, 0.07, 0.06)
	_blank_vp.add_child(blank_bg)

	for f in 3:
		await get_tree().process_frame
	await RenderingServer.frame_post_draw
	_blank = _blank_vp.get_texture().get_image().get_data()
	if _blank.is_empty():
		_bad("the blank reference frame captured nothing; the content test below "
			+ "would pass every case vacuously")

	var fails := 0
	var inked := 0
	var baseline_ink := false
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
			var data_a := a.get_data()
			var same: bool = data_a == b.get_data()
			## Ink, against the blank control -- NOT against the other arm. Two
			## blank frames are byte-identical, so `same` alone is satisfied by
			## a probe that drew nothing at all.
			var ink: bool = not _blank.is_empty() and data_a != _blank
			if ink:
				inked += 1
			if z == 1.0 and pan == Vector2.ZERO:
				baseline_ink = ink
			if not same:
				fails += 1
				a.save_png("user://cull_on_z%.0f_%d_%d.png" % [z, int(pan.x), int(pan.y)])
				b.save_png("user://cull_off_z%.0f_%d_%d.png" % [z, int(pan.x), int(pan.y)])
			_p("zoom %.0f pan (%5d,%5d) -> %-38s ink=%s" % [z, pan.x, pan.y,
				"identical" if same else "DIFFERENT (captures in user://)",
				"yes" if ink else "NO"])

	_fails += fails
	## At zoom 1 with the camera at the origin the generated network spans the
	## whole window by construction (`_roads()` starts every way inside
	## x<=200,y<=600 and walks +8..34 px per step for 30 steps), so a blank
	## frame there is not a legal outcome -- it is the overlay not drawing.
	if not baseline_ink:
		_bad("zoom 1 / pan (0,0) rendered the bare background: the overlay drew "
			+ "nothing, so every 'identical' above compares two empty frames")
	if inked == 0:
		_bad("no case drew a single pixel; the pixel comparison proved nothing")
	_p("content: %d of 16 cases drew ink" % inked)

	await _object_counts()

	_p("RESULT: %s" % ("PASS -- culling moves no pixel, and skips real work" if _fails == 0
		else "FAIL -- %d check(s) failed" % _fails))
	get_tree().quit(0 if _fails == 0 else 1)


## Claim 2. `RENDER_TOTAL_OBJECTS_IN_FRAME` is per *frame*, not per viewport, so
## one arm is rendered at a time with everything else's update disabled and the
## reading taken after `frame_post_draw` -- the idiom `_dashbatch_probe.gd`'s
## phase 2 established for exactly this monitor.
##
## Two camera states, because the interesting number is the difference between
## them and not either one alone:
##
##   all-visible   zoom 1, origin      -- culling may reject nothing here, so
##                                        the arms are allowed to be equal.
##   deep pan      zoom 8, (-1600,-900) -- the whole network is outside the
##                                        window. If the culled arm does not
##                                        draw STRICTLY fewer objects here then
##                                        `_run_offscreen()` is not running, and
##                                        the pixel-identity above is identity
##                                        between two arms doing the same work.
func _object_counts() -> void:
	_p("---- phase 2: drawn objects per arm")
	for state in [{"z": 1.0, "pan": Vector2.ZERO, "tag": "all-visible", "strict": false},
			{"z": 8.0, "pan": Vector2(-1600, -900), "tag": "deep pan", "strict": true}]:
		var got := []
		for i in 2:
			_cams[i].scale = Vector2(state["z"], state["z"])
			_cams[i].position = state["pan"]
			_ovs[i].set_camera_zoom(state["z"])
		for i in 2:
			_blank_vp.render_target_update_mode = SubViewport.UPDATE_DISABLED
			for j in 2:
				_vps[j].render_target_update_mode = SubViewport.UPDATE_DISABLED
			_vps[i].render_target_update_mode = SubViewport.UPDATE_ALWAYS
			_ovs[i].queue_redraw()
			for f in 4:
				await get_tree().process_frame
			await RenderingServer.frame_post_draw
			await get_tree().process_frame
			got.append(int(Performance.get_monitor(Performance.RENDER_TOTAL_OBJECTS_IN_FRAME)))
		_p("   %-12s culling on %6d objects   culling off %6d objects" % [
			state["tag"], got[0], got[1]])
		if got[0] == 0 or got[1] == 0:
			_bad("%s: an arm drew 0 objects; the counter measured nothing" % state["tag"])
			continue
		if got[0] > got[1]:
			_bad("%s: culling DREW MORE (%d > %d) -- the reject test costs more "
				% [state["tag"], got[0], got[1]] + "than the work it saves")
		if state["strict"] and got[0] >= got[1]:
			_bad("%s: culling saved nothing (%d vs %d) with the whole network "
				% [state["tag"], got[0], got[1]]
				+ "off screen -- `_run_offscreen()` is not rejecting anything")
	for j in 2:
		_vps[j].render_target_update_mode = SubViewport.UPDATE_ALWAYS
	_blank_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
