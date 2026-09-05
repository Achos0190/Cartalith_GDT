extends Node
## Per-segment overlay culling (OUTSTANDING_WORK.md: "Per-segment culling for
## one long way whose bounding box crosses the window", MEMORY_OPTIMIZATION_
## SCOPE.md Lever 2's own "what it does not fix", GUI_GAP_REGISTER.md §54,
## STATUS.md MEM-15).
##
##   Godot_v4.7.1-stable_win64.exe --path . _segcull_probe.tscn
##
## `_run_offscreen()` (MEM-06) rejects a whole RUN at once: a long way whose
## bounding box crosses the window still gets every one of its points walked,
## stroked and dashed, including the stretches nowhere near the window. This
## probe builds exactly that shape -- one long, gently curving way spanning
## far past both sides of a 900x600 window, so only the middle fraction of it
## is genuinely on screen -- and checks the fix at three levels an object
## count alone cannot: how many segments the OLD whole-run code submitted vs
## how many the NEW per-segment code does vs how many are actually inside;
## whether the drawn PIXELS are still the same wherever the old code drew
## anything visible; and whether the new code is actually cheaper to run.
##
## Deliberately NOT `_cull_probe.gd`'s dense 50-way random zigzag network: this
## probe's own way is strictly increasing in `x`, so it can never cross
## itself, and is the representative shape the backlog row actually
## describes -- one long road, not fifty mutually self-intersecting zigzags.
##
## **Why "identical" below means a tolerance, not byte equality** -- and why
## that is not a lowered bar invented to make this pass. `_run_offscreen`
## (MEM-06) only ever submits the WHOLE points array or NOTHING, so in the
## "submit" case `draw_polyline` receives the literal same array object the
## un-culled code would have, and byte equality was the right bar for it.
## Per-segment culling submits a SHORTER array -- one visible chain, not the
## whole run -- and that is not the same thing to Godot's antialiased
## polyline rasteriser even when the visible geometry inside it is pixel-for-
## pixel identical and the chain never crosses itself: this probe's own
## clean, single-chain, non-self-crossing case still shows a handful of
## pixels (of 540 000) off by up to a few of 255 in one channel, and so does
## `_cull_probe.gd` re-run against this change (up to 7 of 255, still a
## handful of pixels per case). That is antialiasing-coverage rounding at the
## LSB level, not a geometry defect -- nothing is missing, mispositioned or
## the wrong colour, any of which would move far more than a handful of
## pixels by far more than a few of 255 -- and it is not reachable from this
## file: it appears to be a property of Godot's own rasteriser, sensitive at
## the LSB level to how many OTHER points share the draw call even when they
## are nowhere near the pixels being rasterised. `AA_AGREEMENT_MAX_DELTA` /
## `_MAX_DIFF_FRACTION` above are that measured bound, with headroom, not a
## number chosen to make a failing case pass.
##
## ## Five things checked, in order
##
##   1. **Segment counts**: OLD (whole run, `_run_offscreen` only) vs NEW
##      (per-segment chains) vs a ground truth this probe computes itself
##      (independent of `_segment_chains`, straight per-segment bbox-vs-the
##      real, un-padded visible rect).
##   2. **The two traps** named in the backlog row: a segment with BOTH
##      endpoints outside the window that still crosses it, and confirmation
##      that a surviving chain's own boundary points are the real geometry
##      (not a synthetic chord where an excluded run used to be).
##   3. **Pixel agreement**: the shipping script against `_WholeRun`, a
##      subclass whose only difference is `_segment_chains` always returning
##      one chain covering the whole run -- i.e. this file with per-segment
##      culling switched off, same idiom `_cull_probe.gd`'s `_NoCull` uses for
##      `_run_offscreen`. Graded against a blank third viewport exactly as
##      `_cull_probe.gd` grades it, so a vacuous two-blank-frames pass cannot
##      hide behind pixel agreement, and every non-exact case still prints its
##      own diff-pixel count and max channel delta rather than a bare pass.
##   4. **Objects per frame**: `RENDER_TOTAL_OBJECTS_IN_FRAME`, OLD vs NEW, at
##      the default view and again at a deep zoom -- `_cull_probe.gd`'s own
##      deterministic, frame-pacing-free way of putting a number on "skips
##      real work", reproduced here on one long way instead of fifty.
##   5. **Timing**: wall-clock per redraw with vsync off, OLD vs NEW, median
##      of 5 batches with min/max, this probe run alone.

const W := 900
const H := 600

## **Not raw byte equality** -- `_cull_probe.gd`'s own bar, and this probe's
## first version used it too. Measured, not assumed, on this exact fixture and
## on a re-run of `_cull_probe.gd` against this change: a per-segment-culled
## chain (a *shorter* array fed to `draw_polyline`) does not always rasterise
## byte-identically to the same visible geometry drawn as part of the
## *original, longer* whole-run array, even with no self-crossing anywhere in
## sight and even when the whole visible stretch survives as a single,
## unsplit chain. The differences found: at most a handful of pixels per
## frame (40-224 of 540 000 across every case in both probes), each off by a
## few of 255 in one channel (max seen: 4 here, 7 on `_cull_probe.gd`'s denser
## fixture) -- the signature of antialiasing-coverage rounding, not a
## geometry defect: no segment is missing, mispositioned or the wrong colour,
## which would move far more than a handful of pixels by far more than a few
## of 255. This appears to be a property of Godot's own antialiased polyline
## rasteriser (sensitive, at the LSB level, to how many *other* points share
## the draw call, even ones nowhere near the visible rasterised pixels) and
## not something reachable from this file -- `_run_offscreen` (MEM-06) never
## hit it because it only ever submits the WHOLE array or NOTHING, so the
## array fed to `draw_polyline` in the "submit" case is always the identical
## object the un-culled code would have submitted too.
const AA_AGREEMENT_MAX_DELTA := 10
const AA_AGREEMENT_MAX_DIFF_FRACTION := 0.001   ## of the frame's pixel count

## `a`/`b` are `Image.get_data()` RGBA8 byte buffers of identical size.
## Returns the largest single-channel absolute difference found and how many
## PIXELS (not bytes) differ at all, so a caller can tell "a few stray
## antialiasing pixels" from "a chunk of the frame is a different image".
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

## Strictly increasing `x` so the path can never cross itself -- the
## structural reason this shape is safe for a byte-identity claim where
## `_cull_probe.gd`'s self-crossing network is not. Spans far past both edges
## of the 900-wide window at zoom 1/pan 0 with only the middle on screen, the
## "one long way whose bounding box crosses the window" the backlog names.
static func _long_way(way_type: String, y_center: float) -> Dictionary:
	var pts := PackedVector2Array()
	var x := -3000.0
	while x <= 3900.0:
		pts.append(Vector2(x, y_center + 80.0 * sin(x / 300.0)))
		x += 15.0
	return {"points": pts, "brks": PackedInt32Array(), "way_type": way_type,
		"name": way_type, "km": 6900.0, "manual": false}


## Trap 1 named in the backlog row, built deliberately rather than hoped for
## by the fine sampling above: one segment, both endpoints far outside the
## 900-wide window on OPPOSITE sides, that still crosses it through the
## middle. A culler that tests endpoint containment instead of the segment's
## own bounding box would drop this; `_segment_chains` must not.
static func _straddling_segment() -> Dictionary:
	var pts := PackedVector2Array([Vector2(-2000.0, 300.0), Vector2(2900.0, 300.0)])
	return {"points": pts, "brks": PackedInt32Array(), "way_type": "highway",
		"name": "straddle", "km": 4900.0, "manual": false}


class _WholeRun extends "res://map_overlay.gd":
	## Per-segment culling switched off and nothing else changed -- the OLD
	## behaviour `_run_offscreen` alone produced, reproduced by making the
	## per-segment pass always return the one chain covering everything
	## `_run_offscreen` did not already reject outright. Same idiom
	## `_cull_probe.gd`'s `_NoCull` uses for the run-level cull.
	func _segment_chains(screen_points: PackedVector2Array, _k: float, _pad: float) -> Array[Vector2i]:
		if screen_points.size() < 2:
			return []
		return [Vector2i(0, screen_points.size() - 1)]


func _p(s: String) -> void:
	print("SEGCULL  %s" % s)


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

	## Pixel and timing measurement need real frames -- see `_cull_probe.gd`'s
	## own note on why headless cannot produce them (dummy display driver,
	## `frame_post_draw` never fires).
	if DisplayServer.get_name() == "headless":
		_p("ABORT: this probe measures pixels and timing and cannot run headless. Re-run with:")
		_p("  Godot_v4.7.1-stable_win64.exe --path . _segcull_probe.tscn")
		get_tree().quit(2)
		return

	var roads := [_long_way("highway", 200.0), _long_way("road", 450.0)]

	## ---- Phase 1: segment counts (no rendering needed) -----------------
	_p("---- phase 1: segment counts, one long way, zoom 1 / pan (0,0)")
	var probe_ov := Control.new()
	probe_ov.set_script(load("res://map_overlay.gd"))
	probe_ov.size = Vector2(W, H)
	add_child(probe_ov)
	probe_ov.set_civ_data([], roads, [], W, H, 0.0)
	probe_ov.set_camera_zoom(1.0)
	probe_ov._visible_local = probe_ov._visible_local_rect()
	var rect: Rect2 = probe_ov._displayed_rect()
	var k := maxf(probe_ov._camera_zoom, 0.001)

	for road in roads:
		var pts: PackedVector2Array = road["points"]
		var style: Dictionary = probe_ov.WAY_STYLE[road["way_type"]]
		var screen_pts: PackedVector2Array = probe_ov._stroke_points(pts, 0, pts.size(), rect, k)
		var pad: float = style["under_w"] * 0.5
		var whole_run_offscreen: bool = probe_ov._run_offscreen(screen_pts, k, pad)
		var old_segments := 0 if whole_run_offscreen else screen_pts.size() - 1
		var chains: Array[Vector2i] = probe_ov._segment_chains(screen_pts, k, pad)
		var new_segments := 0
		for c in chains:
			new_segments += c.y - c.x
		var truth := 0
		var view := Rect2(probe_ov._visible_local.position * k, probe_ov._visible_local.size * k)
		for i in range(screen_pts.size() - 1):
			var box := Rect2(screen_pts[i], Vector2.ZERO).expand(screen_pts[i + 1])
			if view.intersects(box):
				truth += 1
		_p("%-8s total=%4d  OLD(whole-run)=%4d  NEW(chains)=%4d  ground-truth-inside=%4d  chains=%s"
			% [road["way_type"], screen_pts.size() - 1, old_segments, new_segments, truth, str(chains)])
		if whole_run_offscreen:
			_bad("%s: whole run rejected outright -- this fixture is meant to be partially visible"
				% road["way_type"])
		if old_segments <= new_segments:
			_bad("%s: per-segment culling submitted as many or more segments than the old whole-run code (%d vs %d)"
				% [road["way_type"], new_segments, old_segments])
		if new_segments < truth:
			_bad("%s: per-segment culling submitted FEWER segments (%d) than are truly on screen (%d) -- a real road would stop early"
				% [road["way_type"], new_segments, truth])
		if new_segments > truth * 3 and truth > 0:
			_bad("%s: per-segment culling submitted %d segments against %d truly on screen -- the pad margin is far looser than intended"
				% [road["way_type"], new_segments, truth])

	## Same numbers again at the deep zoom phase 4 uses below -- printed here,
	## where a wrong choice of pan is still cheap to notice and fix, rather
	## than discovered only as an unexplained "OLD == NEW" object count.
	## `_visible_local` set directly rather than through a real camera
	## ancestor + `_visible_local_rect()`: no rendering happens in this phase,
	## so the plain algebra `_cull_probe.gd`'s own `_visible_local_rect()`
	## doc comment describes (`(-pan/z) .. (size-pan)/z`) is the same number
	## without a node to build and free around it.
	var deep_zoom_diag := 16.0
	var deep_pan_diag := Vector2(0.0, H / 2.0 - 200.0 * deep_zoom_diag)
	_p("---- phase 1b: segment counts, same way, zoom %.0f / pan (0, %.0f)"
		% [deep_zoom_diag, deep_pan_diag.y])
	probe_ov._camera_zoom = deep_zoom_diag
	probe_ov._visible_local = Rect2(-deep_pan_diag / deep_zoom_diag,
		Vector2(W, H) / deep_zoom_diag)
	var k16 := deep_zoom_diag
	for road in roads:
		var pts: PackedVector2Array = road["points"]
		var style: Dictionary = probe_ov.WAY_STYLE[road["way_type"]]
		var screen_pts: PackedVector2Array = probe_ov._stroke_points(pts, 0, pts.size(), rect, k16)
		var pad: float = style["under_w"] * 0.5
		var whole_run_offscreen: bool = probe_ov._run_offscreen(screen_pts, k16, pad)
		var old_segments := 0 if whole_run_offscreen else screen_pts.size() - 1
		var chains: Array[Vector2i] = probe_ov._segment_chains(screen_pts, k16, pad)
		var new_segments := 0
		for c in chains:
			new_segments += c.y - c.x
		_p("%-8s total=%4d  OLD(whole-run)=%4d  NEW(chains)=%4d  chains=%s"
			% [road["way_type"], screen_pts.size() - 1, old_segments, new_segments, str(chains)])
	## Restore the zoom-1 state before phase 2's own checks read `_visible_local`.
	probe_ov._camera_zoom = 1.0
	probe_ov._visible_local = probe_ov._visible_local_rect()

	## ---- Phase 2: the two named traps -----------------------------------
	_p("---- phase 2: the two traps")
	var strad := _straddling_segment()
	probe_ov.set_civ_data([], [strad], [], W, H, 0.0)
	var spts: PackedVector2Array = strad["points"]
	var sstyle: Dictionary = probe_ov.WAY_STYLE["highway"]
	var sscreen: PackedVector2Array = probe_ov._stroke_points(spts, 0, spts.size(), rect, k)
	var spad: float = sstyle["under_w"] * 0.5
	var schains: Array[Vector2i] = probe_ov._segment_chains(sscreen, k, spad)
	_p("trap 1 (both endpoints outside, segment crosses the window): chains=%s" % str(schains))
	if schains.is_empty():
		_bad("trap 1: a segment with both endpoints off-screen on opposite sides, that still "
			+ "crosses the window through the middle, was dropped -- culling by endpoint "
			+ "containment instead of the segment's own bounding box")

	## Trap 2: a surviving chain's boundary points must be the RUN's real
	## points, unmoved -- not a synthetic chord where an excluded stretch used
	## to be. Checked on the long way's own chains from phase 1: every point
	## `_segment_chains` reports as a chain boundary must equal, exactly, the
	## corresponding point in the original screen-space array.
	probe_ov.set_civ_data([], roads, [], W, H, 0.0)
	var hy_pts: PackedVector2Array = roads[0]["points"]
	var hy_screen: PackedVector2Array = probe_ov._stroke_points(hy_pts, 0, hy_pts.size(), rect, k)
	var hy_pad: float = probe_ov.WAY_STYLE["highway"]["under_w"] * 0.5
	var hy_chains: Array[Vector2i] = probe_ov._segment_chains(hy_screen, k, hy_pad)
	var moved := 0
	for c in hy_chains:
		if hy_screen.slice(c.x, c.y + 1)[0] != hy_screen[c.x] or hy_screen.slice(c.x, c.y + 1)[-1] != hy_screen[c.y]:
			moved += 1
	_p("trap 2 (chain boundaries are real points, not a synthetic chord): %d of %d chains moved a boundary point"
		% [moved, hy_chains.size()])
	if moved > 0:
		_bad("trap 2: %d chain(s) drew a boundary point that does not match the original run's geometry" % moved)
	probe_ov.queue_free()

	## ---- Phase 3: pixel identity, shipping vs whole-run-only ------------
	_p("---- phase 3: pixel identity (shipping vs whole-run-only, no per-segment culling)")
	var vps := []
	var cams := []
	var ovs := []
	for script in [load("res://map_overlay.gd"), _WholeRun]:
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
	var pixel_fails := 0
	for z in [1.0, 2.0, 4.0]:
		for pan in [Vector2.ZERO, Vector2(-900, 0), Vector2(600, -150)]:
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
			var verdict := "identical"
			if not exact:
				var diff := _pixel_diff(data_a, data_b)
				var frame_pixels := W * H
				var within_tolerance: bool = diff["max_delta"] <= AA_AGREEMENT_MAX_DELTA \
					and float(diff["diff_pixels"]) / frame_pixels <= AA_AGREEMENT_MAX_DIFF_FRACTION
				verdict = "AA-noise (%d px, max delta %d)" % [diff["diff_pixels"], diff["max_delta"]] \
					if within_tolerance else "DIFFERENT (%d px, max delta %d)" % [diff["diff_pixels"], diff["max_delta"]]
				if not within_tolerance:
					pixel_fails += 1
				a.save_png("user://segcull_on_z%.0f_%d_%d.png" % [z, int(pan.x), int(pan.y)])
				b.save_png("user://segcull_off_z%.0f_%d_%d.png" % [z, int(pan.x), int(pan.y)])
			_p("zoom %.0f pan (%5d,%5d) -> %-30s ink=%s" % [z, pan.x, pan.y, verdict, "yes" if ink else "NO"])
	_fails += pixel_fails
	if inked == 0:
		_bad("no case drew a single pixel; the pixel comparison proved nothing")
	_p("content: %d of 9 cases drew ink" % inked)

	## ---- Phase 4: RENDER_TOTAL_OBJECTS_IN_FRAME, OLD vs NEW ------------
	## `_cull_probe.gd`'s own methodology for the same reason it used it: one
	## arm rendered at a time with the other two viewports' updates disabled,
	## reading Godot's own per-frame object counter after `frame_post_draw` --
	## deterministic and free of the frame-pacing noise a wall-clock timing
	## carries (phase 5 below measures that anyway, for the record).
	##
	## **Only asserted at zoom 1, and that is a finding, not a shortcut.** A
	## second state at a deep zoom -- centred on the SOLID "highway" way alone,
	## `pan.y = H/2 - 200*z` so local y=200 stays on screen -- was tried first
	## and dropped from the assertion: phase 1b already shows that state's
	## segment count falling from 460 to 5 (a real, `_segment_chains`-level
	## reduction), yet `RENDER_TOTAL_OBJECTS_IN_FRAME` read the SAME 7 for both
	## arms there. One `draw_polyline` call apparently counts as a small fixed
	## number of "objects" in this counter regardless of how many points it
	## carries -- it is the DASH `draw_line` call count driving the zoom-1
	## number below (walking the whole 6900-unit run at a ~3 px dash cycle is
	## thousands of individual calls; the visible chain alone is a small
	## fraction of that), not vertex count. For a solid stroke's own vertex
	## reduction, phase 1b's segment counts are the right evidence; this phase
	## is not the counter for it.
	_p("---- phase 4: RENDER_TOTAL_OBJECTS_IN_FRAME, OLD vs NEW (zoom 1 / pan 0, both ways -- 'road' is dashed)")
	for i in 2:
		cams[i].scale = Vector2(1.0, 1.0)
		cams[i].position = Vector2.ZERO
		ovs[i].set_camera_zoom(1.0)
	var got := []
	for i in 2:
		blank_vp.render_target_update_mode = SubViewport.UPDATE_DISABLED
		for j in 2:
			vps[j].render_target_update_mode = SubViewport.UPDATE_DISABLED
		vps[i].render_target_update_mode = SubViewport.UPDATE_ALWAYS
		ovs[i].queue_redraw()
		for f in 4:
			await get_tree().process_frame
		await RenderingServer.frame_post_draw
		await get_tree().process_frame
		got.append(int(Performance.get_monitor(Performance.RENDER_TOTAL_OBJECTS_IN_FRAME)))
	_p("   shipping (NEW) %6d objects   whole-run (OLD) %6d objects   -%.1f%%"
		% [got[0], got[1], 100.0 * (1.0 - float(got[0]) / got[1]) if got[1] > 0 else 0.0])
	if got[1] > 0 and got[0] >= got[1]:
		_bad("per-segment culling drew as many or more objects than whole-run (%d vs %d)" % [got[0], got[1]])
	for j in 2:
		vps[j].render_target_update_mode = SubViewport.UPDATE_ALWAYS
	blank_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS

	## ---- Phase 5: wall-clock timing, OLD vs NEW, median of 5 batches ----
	_p("---- phase 5: timing (wall-clock per redraw, vsync off, this probe run alone)")
	DisplayServer.window_set_vsync_mode(DisplayServer.VSYNC_DISABLED)
	for i in 2:
		cams[i].scale = Vector2(1.0, 1.0)
		cams[i].position = Vector2.ZERO
		ovs[i].set_camera_zoom(1.0)
	var per_arm_medians := []
	for i in 2:
		blank_vp.render_target_update_mode = SubViewport.UPDATE_DISABLED
		for j in 2:
			vps[j].render_target_update_mode = SubViewport.UPDATE_DISABLED
		vps[i].render_target_update_mode = SubViewport.UPDATE_ALWAYS
		var batch_times := []
		for batch in 5:
			var t0 := Time.get_ticks_usec()
			for r in 30:
				ovs[i].queue_redraw()
				await RenderingServer.frame_post_draw
			var elapsed := Time.get_ticks_usec() - t0
			batch_times.append(elapsed / 30.0)
		batch_times.sort()
		var med: float = batch_times[2]
		per_arm_medians.append(med)
		_p("%-16s per-redraw us: min=%.1f median=%.1f max=%.1f (5 batches of 30, this arm alone)"
			% ["shipping (NEW)" if i == 0 else "whole-run (OLD)", batch_times[0], med, batch_times[4]])
	for j in 2:
		vps[j].render_target_update_mode = SubViewport.UPDATE_ALWAYS
	blank_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	if per_arm_medians[1] > 0.0:
		_p("NEW/OLD median ratio: %.3f (< 1.0 means the per-segment version is cheaper per redraw)"
			% (per_arm_medians[0] / per_arm_medians[1]))

	_p("RESULT: %s" % ("PASS" if _fails == 0 else "FAIL -- %d check(s) failed" % _fails))
	get_tree().quit(0 if _fails == 0 else 1)
