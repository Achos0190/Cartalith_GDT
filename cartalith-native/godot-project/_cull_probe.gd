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
## frames.
##
## Driven through a `Node2D` camera ancestor at four zooms and four pans,
## because that ancestor scale is exactly what the shipping code had no way of
## seeing before `_run_offscreen()` existed.
##
## ## Not raw byte equality -- a measured, cited tolerance (2026-09-13)
##
## This probe (2026-08-25) predates `_segment_chains` (per-segment culling,
## added later -- see `map_overlay.gd`'s own doc comment directly above that
## function). `_NoCull` below overrides only `_visible_local_rect()`; it does
## not touch `_segment_chains` directly -- but `_segment_chains` reads
## `_visible_local`, which `_visible_local_rect()` feeds, so `_NoCull`'s huge
## returned rect (`-1e9 .. 1e9`) makes `view.intersects(box)` true for every
## real segment and `_segment_chains` collapses to exactly one whole-run chain
## every time, unconditionally. The shipping arm, over the REAL visible rect,
## may still split the same run into one or more SHORTER chains. Godot's
## antialiased `draw_polyline` does not always rasterise the same visible
## geometry byte-identically when it arrives as a shorter, differently-chained
## array instead of the original longer one, even with no self-crossing
## anywhere in sight -- `_segcull_probe.gd` measured and documented this exact
## cause for its own per-segment-culling comparison, and shipped a tolerance
## for it. `AA_AGREEMENT_MAX_DELTA` / `AA_AGREEMENT_MAX_DIFF_FRACTION` below are
## REFERENCES to that probe's own constants (`preload`ed, not retyped), so the
## two can never silently drift apart. Measured on this exact fixture,
## 2026-09-13 (`_dashresidualcheck_probe.gd`, reading this probe's own saved
## captures): all 13 of 16 cases that fail raw byte equality differ by at most
## 1 of 255 in one channel (a flat 10x inside the shipped `AA_AGREEMENT_MAX_
## DELTA = 10` on every one of the 13) on between 1 and 158 of the frame's
## 540 000 pixels (0.0002%-0.03%; against the shipped `_MAX_DIFF_FRACTION =
## 0.001` = 540 px, that is ~3.4x inside the bound at the closest case and
## ~540x inside it at the furthest, not merely near it on either axis).
##
## **The other fix this could have been, and why it wasn't.** The backlog row
## that opened this also asked whether `_NoCull` should explicitly bypass
## `_segment_chains` instead, as "the honest comparison". It already does, in
## effect -- the huge-rect paragraph above IS that bypass, arrived at
## indirectly: an explicit override would return the identical one-whole-chain
## result in every case, so it changes nothing observable, and the comparison
## would still be "shipping's real, possibly-shorter chains" against "one
## whole-run chain" -- `_segcull_probe.gd`'s own Phase 3 shape, not a route back
## to byte identity. The change that WOULD restore true byte equality --
## overriding `_run_offscreen()` itself instead of `_visible_local_rect()`, so
## `_segment_chains` sees the identical real view on both arms -- was rejected
## for a sharper reason: it would make this probe BLIND to a `_segment_chains`
## regression, since both arms would then run the exact same chaining over the
## exact same view and could only ever diverge through `_run_offscreen`. Phase
## 3 below keeps that claim checked rather than argued: it proves the shipped
## tolerance still catches a real defect in either mechanism, on `_NoCull` as
## this file already had it.
##
## ## Three claims, and why pixel-equality alone is not the first of them
##
## Byte-equality of two captures is satisfied *perfectly* by two blank frames,
## so on its own it is a test that cannot fail: if `set_civ_data()` were ever to
## stop reaching the overlay, or the script were to stop drawing, both arms
## would render the bare `ColorRect` and the probe would report PASS over zero
## coverage. Every case is therefore graded against a THIRD viewport holding the
## background and nothing else (`_vp_blank`), and a case whose culled arm is
## byte-identical to that blank frame is reported INK=0 -- evidence, not a pass.
##
##   1. **No pixel moves beyond the tolerance above.** The shipping script
##      against a subclass of itself whose only difference is that
##      `_visible_local_rect()` returns everything, over every zoom/pan -- with
##      at least one case, and specifically the everything-on-screen baseline,
##      proven to have drawn ink.
##   2. **Something is actually skipped.** Phase 2 renders one arm per frame
##      and reads Godot's own `RENDER_TOTAL_OBJECTS_IN_FRAME`, which is the
##      number claim 1 is worthless without: culling that moves no pixel *and*
##      saves no object is culling that is not happening.
##   3. **The tolerance is not hiding a real defect.** Two mutants built on the
##      real `_run_offscreen` / `_segment_chains` via `super` -- an inverted
##      visible-rect test, and a dropped chain, the two examples named when
##      this tolerance was added -- measured against the shipping arm and
##      asserted to fall OUTSIDE the same tolerance claim 1 now uses.

const W := 900
const H := 600

## Not this file's own bound -- `_segcull_probe.gd`'s, referenced directly
## (never retyped) so the two probes can never silently drift apart. See the
## top-of-file docstring above for why this probe needs it too, and that
## file's own doc comment above `AA_AGREEMENT_MAX_DELTA` for the measurement
## and the antialiasing-coverage-rounding cause.
const _SegCullProbe := preload("res://_segcull_probe.gd")

## `a`/`b` are `Image.get_data()` RGBA8 byte buffers of identical size.
## Exact copy of `_segcull_probe.gd::_pixel_diff`'s semantics (kept as a literal
## copy rather than a cross-file call so this probe has no runtime dependency
## on that one beyond the two constants above): the largest single-channel
## absolute difference found, and how many PIXELS (not bytes) differ at all.
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


## True if `a` and `b` (same-size RGBA8 buffers) agree within
## `_segcull_probe.gd`'s own shipped AA tolerance -- exact equality always
## qualifies. Returns the diff dictionary too so a caller can report the
## magnitude on both a pass and a fail, not just a verdict.
func _within_aa_tolerance(a: PackedByteArray, b: PackedByteArray) -> Dictionary:
	if a == b:
		return {"within": true, "max_delta": 0, "diff_pixels": 0}
	var diff := _pixel_diff(a, b)
	var frame_pixels := W * H
	var within: bool = diff["max_delta"] <= _SegCullProbe.AA_AGREEMENT_MAX_DELTA \
		and float(diff["diff_pixels"]) / frame_pixels <= _SegCullProbe.AA_AGREEMENT_MAX_DIFF_FRACTION
	diff["within"] = within
	return diff


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
	## against itself. Deliberately left overriding only
	## `_visible_local_rect()`, not `_segment_chains` too -- see the
	## top-of-file docstring's "other fix this could have been" for why: this
	## exact shape is what keeps Phase 3 below able to catch a
	## `_segment_chains` regression, which a `_run_offscreen`-only override
	## would not.
	func _visible_local_rect() -> Rect2:
		return Rect2(-1e9, -1e9, 2e9, 2e9)


## Phase 3 mutant 1, named in the backlog row that added this phase: "the
## visible-rect test inverted". Built on the real `_run_offscreen` via `super`
## rather than a retyped copy of its box math, so this can never silently
## drift from what it inverts. Rejects exactly the runs the real function
## would keep on screen, and keeps exactly the ones it would reject -- a real,
## load-bearing culling defect (whole visible ways go missing), not an
## AA-scale rounding difference.
class _InvertedRunOffscreen extends "res://map_overlay.gd":
	func _run_offscreen(pts: PackedVector2Array, k: float, pad: float) -> bool:
		if pts.is_empty():
			return true
		return not super._run_offscreen(pts, k, pad)


## Phase 3 mutant 2, the row's other named example: "a segment dropped". Runs
## the real `_segment_chains` via `super` and then discards the chain it found
## LAST, simulating the off-by-one of forgetting that function's own trailing
## `if run_start >= 0: chains.append(...)` -- a run still on screen at its very
## last point loses that final visible stretch.
class _DropLastChain extends "res://map_overlay.gd":
	func _segment_chains(screen_points: PackedVector2Array, k: float, pad: float) -> Array[Vector2i]:
		var chains := super._segment_chains(screen_points, k, pad)
		if chains.size() > 0:
			chains.remove_at(chains.size() - 1)
		return chains


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
			var data_b := b.get_data()
			var exact: bool = data_a == data_b
			var diff := _within_aa_tolerance(data_a, data_b)
			var same: bool = diff["within"]
			## Ink, against the blank control -- NOT against the other arm. Two
			## blank frames are byte-identical, so `same` alone is satisfied by
			## a probe that drew nothing at all.
			var ink: bool = not _blank.is_empty() and data_a != _blank
			if ink:
				inked += 1
			if z == 1.0 and pan == Vector2.ZERO:
				baseline_ink = ink
			if not exact:
				## Saved whenever bytes differ at all, even a within-tolerance
				## AA-noise case -- `_segcull_probe.gd`'s own convention, kept
				## so a future run can still see the magnitude, not just the
				## verdict.
				a.save_png("user://cull_on_z%.0f_%d_%d.png" % [z, int(pan.x), int(pan.y)])
				b.save_png("user://cull_off_z%.0f_%d_%d.png" % [z, int(pan.x), int(pan.y)])
			if not same:
				fails += 1
			var verdict := "identical"
			if not exact:
				verdict = "AA-noise (%d px, max delta %d)" % [diff["diff_pixels"], diff["max_delta"]] \
					if same else "DIFFERENT (%d px, max delta %d)" % [diff["diff_pixels"], diff["max_delta"]]
			_p("zoom %.0f pan (%5d,%5d) -> %-38s ink=%s" % [z, pan.x, pan.y, verdict,
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
	await _mutant_guard(roads, routes)

	_p("RESULT: %s" % ("PASS -- culling moves no pixel beyond the shipped AA tolerance, skips real "
			+ "work, and that tolerance still catches a real defect" if _fails == 0
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


## Builds one throwaway SubViewport+camera+overlay for `script`, feeds it the
## same fixture the main matrix uses, positions the camera at `z`/`pan`, waits
## for a real frame and returns its captured RGBA8 bytes. Freed before
## returning -- this runs after every other phase's own viewports are done
## with, so nothing else needs it to stay alive, and `RENDER_TOTAL_OBJECTS_
## IN_FRAME` (phase 2, already read by this point) is not disturbed by an
## extra viewport existing briefly.
func _capture_arm(script, roads: Array, routes: Array, z: float, pan: Vector2) -> PackedByteArray:
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
	cam.scale = Vector2(z, z)
	cam.position = pan
	vp.add_child(cam)
	var ov := Control.new()
	ov.set_script(script)
	ov.size = Vector2(W, H)
	cam.add_child(ov)
	ov.set_civ_data([], roads, [], W, H, 0.0)
	ov.set_manual_routes(routes)
	ov.set_camera_zoom(z)
	ov.queue_redraw()
	for f in 3:
		await get_tree().process_frame
	await RenderingServer.frame_post_draw
	var data := vp.get_texture().get_image().get_data()
	vp.queue_free()
	return data


## Phase 3: does this probe's shipped AA tolerance still catch a REAL culling
## defect, or has it been loosened enough to hide one? See the top-of-file
## docstring's "other fix this could have been" for why this exists and what
## it stands in for. Two mutants (`_InvertedRunOffscreen`, `_DropLastChain`,
## both defined above via `super` so neither can drift from what it mutates),
## each measured against a fresh shipping capture at two cases already proven
## to carry real ink: the all-visible baseline (zoom 1, pan 0) and one of the
## main matrix's own AA-noise cases (zoom 2, pan -400,-260) -- so a mutant's
## divergence is checked against a background that already carries some
## legitimate AA noise of its own, not a clean case that would flatter the
## tolerance.
func _mutant_guard(roads: Array, routes: Array) -> void:
	_p("---- phase 3: mutant regression guard (does the AA tolerance still catch a real defect?)")
	var cases := [{"z": 1.0, "pan": Vector2.ZERO, "tag": "z1 pan(0,0)"},
		{"z": 2.0, "pan": Vector2(-400, -260), "tag": "z2 pan(-400,-260)"}]
	var mutants := [["inverted _run_offscreen", _InvertedRunOffscreen],
		["dropped last chain", _DropLastChain]]
	for c in cases:
		var ship_data := await _capture_arm(load("res://map_overlay.gd"), roads, routes, c["z"], c["pan"])
		for m in mutants:
			var label: String = m[0]
			var mut_data: PackedByteArray = await _capture_arm(m[1], roads, routes, c["z"], c["pan"])
			var diff := _within_aa_tolerance(ship_data, mut_data)
			_p("   %-24s @ %-16s max_delta=%3d diff_px=%6d/%d (%.4f%%) -> %s"
				% [label, c["tag"], diff["max_delta"], diff["diff_pixels"], W * H,
					100.0 * float(diff["diff_pixels"]) / (W * H),
					"escaped (within tolerance)" if diff["within"] else "caught (outside tolerance)"])
			if diff["within"]:
				_bad("%s @ %s: this real culling defect stayed inside the AA tolerance -- the bar is too loose"
					% [label, c["tag"]])
				var tag: String = c["tag"].replace(" ", "_").replace("(", "").replace(")", "").replace(",", "_")
				var lbl: String = label.replace(" ", "_")
				Image.create_from_data(W, H, false, Image.FORMAT_RGBA8, ship_data) \
					.save_png("user://cull_mutant_ship_%s_%s.png" % [tag, lbl])
				Image.create_from_data(W, H, false, Image.FORMAT_RGBA8, mut_data) \
					.save_png("user://cull_mutant_bad_%s_%s.png" % [tag, lbl])
