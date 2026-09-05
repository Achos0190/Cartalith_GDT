extends Node
## VERIFIER probe, round-2 Lane Culling. Independent of `_segcull_probe.gd`:
## its own fixtures, its own ground-truth oracle (a real Liang-Barsky
## segment/rect clip, NOT the same axis-aligned box test `_segment_chains`
## uses), and pixel controls that assert WHERE ink is and where it must not be
## -- a segment count is not evidence that the geometry still draws.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _vfy_r2_cull_probe.tscn
##      [-- --only-timing]

const W := 900
const H := 600
const BG := Color(0.08, 0.07, 0.06)

var _fails := 0

func _p(s: String) -> void:
	print("VCULL  %s" % s)

func _bad(s: String) -> void:
	_fails += 1
	_p("FAIL  %s" % s)

class _WholeRun extends "res://map_overlay.gd":
	## Per-segment culling OFF and nothing else: one chain over everything
	## `_run_offscreen` did not already reject.
	func _segment_chains(sp: PackedVector2Array, _k: float, _pad: float) -> Array[Vector2i]:
		if sp.size() < 2:
			return []
		return [Vector2i(0, sp.size() - 1)]

## Real segment-vs-rect intersection (Liang-Barsky), the oracle
## `_segment_chains`' bounding-box test is allowed to be over-inclusive
## against. `_segcull_probe.gd`'s "ground truth" is the SAME box test on a
## smaller rect, so it cannot see box-vs-line over-inclusion at all.
static func _seg_hits_rect(a: Vector2, b: Vector2, r: Rect2) -> bool:
	var d := b - a
	var t0 := 0.0
	var t1 := 1.0
	var p := [-d.x, d.x, -d.y, d.y]
	var q := [a.x - r.position.x, r.position.x + r.size.x - a.x,
			a.y - r.position.y, r.position.y + r.size.y - a.y]
	for i in 4:
		if absf(p[i]) < 1e-12:
			if q[i] < 0.0:
				return false
		else:
			var t: float = q[i] / p[i]
			if p[i] < 0.0:
				t0 = maxf(t0, t)
			else:
				t1 = minf(t1, t)
	return t0 <= t1

static func _way(pts: PackedVector2Array, wt: String) -> Dictionary:
	return {"points": pts, "brks": PackedInt32Array(), "way_type": wt,
		"name": wt, "km": 1.0, "manual": false}

## Long, strictly-increasing-x way. Same shape the lane's fixture uses so its
## 460/79/77 figures are re-derivable, rebuilt from its stated parameters
## rather than by calling its code.
static func _long(y: float, wt: String) -> Dictionary:
	var pts := PackedVector2Array()
	var x := -3000.0
	while x <= 3900.0:
		pts.append(Vector2(x, y + 80.0 * sin(x / 300.0)))
		x += 15.0
	return _way(pts, wt)

## Enters and leaves through the TOP. A culler that dropped the two off-screen
## points and joined what is left would paint a horizontal chord at y=100 from
## x=100 to x=800 that the real polyline never draws.
static func _chord_trap() -> Dictionary:
	return _way(PackedVector2Array([Vector2(100, 100), Vector2(100, -5000),
		Vector2(800, -5000), Vector2(800, 100)]), "highway")

## Four crossings of the bottom edge; the y=3000 links between them are the
## only thing culled.
static func _comb() -> Dictionary:
	return _way(PackedVector2Array([
		Vector2(150, 300), Vector2(150, 3000), Vector2(250, 3000), Vector2(250, 300),
		Vector2(350, 300), Vector2(350, 3000), Vector2(450, 3000), Vector2(450, 300),
	]), "highway")

## Both endpoints far outside, on opposite sides, crossing the middle.
static func _straddle() -> Dictionary:
	return _way(PackedVector2Array([Vector2(-2000, 300), Vector2(2900, 300)]), "highway")

func _mk(script, roads: Array) -> Array:
	var vp := SubViewport.new()
	vp.size = Vector2i(W, H)
	vp.transparent_bg = false
	vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(vp)
	var bg := ColorRect.new()
	bg.size = Vector2(W, H)
	bg.color = BG
	vp.add_child(bg)
	var cam := Node2D.new()
	vp.add_child(cam)
	var ov := Control.new()
	ov.set_script(script)
	ov.size = Vector2(W, H)
	cam.add_child(ov)
	ov.set_civ_data([], roads, [], W, H, 0.0)
	ov.set_camera_zoom(1.0)
	return [vp, cam, ov]

## Ink = "differs from the flat background", which is palette-agnostic: the
## threshold does not depend on which theme booted (MISTAKES.md's palette rule).
static func _ink_mask(img: Image) -> PackedByteArray:
	var out := PackedByteArray()
	out.resize(W * H)
	for y in H:
		for x in W:
			var c := img.get_pixel(x, y)
			var lit := absf(c.r - BG.r) > 0.02 or absf(c.g - BG.g) > 0.02 \
				or absf(c.b - BG.b) > 0.02
			out[y * W + x] = 1 if lit else 0
	return out

static func _ink_in(mask: PackedByteArray, r: Rect2i) -> int:
	var n := 0
	for y in range(maxi(r.position.y, 0), mini(r.position.y + r.size.y, H)):
		for x in range(maxi(r.position.x, 0), mini(r.position.x + r.size.x, W)):
			n += mask[y * W + x]
	return n

func _diff(a: Image, b: Image) -> Array:
	var nd := 0
	var mx := 0
	var struct := 0
	for y in H:
		for x in W:
			var ca := a.get_pixel(x, y)
			var cb := b.get_pixel(x, y)
			var d := maxi(maxi(int(absf(ca.r - cb.r) * 255.0 + 0.5),
				int(absf(ca.g - cb.g) * 255.0 + 0.5)),
				int(absf(ca.b - cb.b) * 255.0 + 0.5))
			if d > 0:
				nd += 1
				mx = maxi(mx, d)
				if d > 16:
					struct += 1
	return [nd, mx, struct]

func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 900.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		_p("WATCHDOG")
		get_tree().quit(2))
	wd.start()
	if DisplayServer.get_name() == "headless":
		_p("ABORT: pixels/timing need a real display; re-run windowed.")
		get_tree().quit(2)
		return
	_p("palette: DccTheme.is_dark()=%s  (every control below is ink-vs-background, palette-agnostic)"
		% str(DccTheme.is_dark()))

	var args := OS.get_cmdline_user_args()
	var timing_only := args.has("--only-timing")

	if not timing_only:
		await _phase_counts()
		await _phase_pixels()
		await _phase_rasteriser()
	await _phase_timing()

	_p("RESULT: %s -- %d check(s) failed" % ["PASS" if _fails == 0 else "FAIL", _fails])
	get_tree().quit(1 if _fails else 0)


func _phase_counts() -> void:
	_p("---- phase 1: chain counts vs a real segment/rect clip oracle")
	## `set_camera_zoom()` alone does NOT reproduce a zoomed view: it sets `k`,
	## which scales the points and `_visible_local` by the same factor, so the
	## on-screen fraction is unchanged. The zoom a user sees is the Node2D
	## ANCESTOR's scale, which is what `_visible_local_rect()` reads. Both are
	## set here; a first pass that set only the former measured 78 kept
	## segments at "zoom 16" -- the zoom-1 answer wearing a different label.
	for zoom in [1.0, 16.0]:
		var roads := [_long(200.0, "highway"), _long(450.0, "road")]
		var host := Node2D.new()
		host.scale = Vector2(zoom, zoom)
		## Keep the ways ON screen at every zoom. Scaling alone drops both of
		## them off the bottom (world y 200/450 becomes 3200/7200 screen px at
		## 16x), which measures "everything culled" -- a degenerate case, not a
		## deep zoom, and it is what a first pass here reported.
		host.position = Vector2(0.0, -325.0 * zoom + 300.0)
		add_child(host)
		var ov := Control.new()
		ov.set_script(load("res://map_overlay.gd"))
		ov.size = Vector2(W, H)
		host.add_child(ov)
		ov.set_civ_data([], roads, [], W, H, 0.0)
		ov.set_camera_zoom(zoom)
		ov._visible_local = ov._visible_local_rect()
		var rect: Rect2 = ov._displayed_rect()
		var k: float = maxf(ov._camera_zoom, 0.001)
		for road in roads:
			var pts: PackedVector2Array = road["points"]
			var style: Dictionary = ov.WAY_STYLE[road["way_type"]]
			var sp: PackedVector2Array = ov._stroke_points(pts, 0, pts.size(), rect, k)
			var pad: float = style["under_w"] * 0.5
			var old_n := 0 if ov._run_offscreen(sp, k, pad) else sp.size() - 1
			var chains: Array[Vector2i] = ov._segment_chains(sp, k, pad)
			var new_n := 0
			for c in chains:
				new_n += c.y - c.x
			var view := Rect2(ov._visible_local.position * k, ov._visible_local.size * k)
			var truth := 0
			for i in range(sp.size() - 1):
				if _seg_hits_rect(sp[i], sp[i + 1], view):
					truth += 1
			_p("zoom %-4.0f %-8s segments=%4d OLD=%4d NEW=%4d TRUE(clip)=%4d chains=%d"
				% [zoom, road["way_type"], sp.size() - 1, old_n, new_n, truth, chains.size()])
			if new_n < truth:
				_bad("zoom %.0f %s: NEW(%d) < TRUE(%d) -- real on-screen geometry dropped"
					% [zoom, road["way_type"], new_n, truth])
			if old_n <= new_n:
				_bad("zoom %.0f %s: no saving (OLD=%d NEW=%d)" % [zoom, road["way_type"], old_n, new_n])
		host.queue_free()
		await get_tree().process_frame

	## The three traps, at the chain level, before any pixel is drawn.
	var ov2 := Control.new()
	ov2.set_script(load("res://map_overlay.gd"))
	ov2.size = Vector2(W, H)
	add_child(ov2)
	for pair in [["chord_trap", _chord_trap()], ["comb", _comb()], ["straddle", _straddle()]]:
		var road: Dictionary = pair[1]
		ov2.set_civ_data([], [road], [], W, H, 0.0)
		ov2.set_camera_zoom(1.0)
		ov2._visible_local = ov2._visible_local_rect()
		var rect: Rect2 = ov2._displayed_rect()
		var pts: PackedVector2Array = road["points"]
		var style: Dictionary = ov2.WAY_STYLE[road["way_type"]]
		var sp: PackedVector2Array = ov2._stroke_points(pts, 0, pts.size(), rect, 1.0)
		var pad: float = style["under_w"] * 0.5
		var chains: Array[Vector2i] = ov2._segment_chains(sp, 1.0, pad)
		var kept := 0
		for c in chains:
			kept += c.y - c.x
		var view := Rect2(ov2._visible_local.position, ov2._visible_local.size)
		var truth := 0
		for i in range(sp.size() - 1):
			if _seg_hits_rect(sp[i], sp[i + 1], view):
				truth += 1
		_p("%-11s segments=%d kept=%d TRUE(clip)=%d chains=%s"
			% [pair[0], sp.size() - 1, kept, truth, str(chains)])
		if kept < truth:
			_bad("%s: dropped %d truly-visible segment(s)" % [pair[0], truth - kept])
	ov2.queue_free()
	await get_tree().process_frame


func _phase_pixels() -> void:
	_p("---- phase 2: pixel controls -- does the geometry still DRAW")
	var cases := [
		## The two visible stubs run from y=100 UP off the top edge, so the
		## positive controls are y in [5,95] -- the first pass put them at
		## y=200..500, below the way's own start point, where BOTH arms
		## correctly drew nothing. A control that is blank in both arms is a
		## broken control, not a finding.
		["chord_trap", [_chord_trap()],
			[Rect2i(94, 5, 13, 90), Rect2i(794, 5, 13, 90)],
			[Rect2i(160, 90, 580, 21)]],
		["comb", [_comb()],
			[Rect2i(144, 400, 13, 150), Rect2i(444, 400, 13, 150), Rect2i(270, 294, 60, 13)],
			[Rect2i(170, 294, 60, 13), Rect2i(370, 294, 60, 13)]],
		["straddle", [_straddle()],
			[Rect2i(20, 294, 60, 13), Rect2i(820, 294, 60, 13), Rect2i(440, 294, 20, 13)],
			[]],
		["long_pair", [_long(200.0, "highway"), _long(450.0, "road")],
			[Rect2i(0, 100, 900, 400)], []],
		## Split by way type: `highway` has `dash == 0.0` and takes the plain
		## `draw_polyline` branch; `road` takes `_draw_dashed_polyline`. Whether
		## the residual follows the solid arm or the dashed one is the whole
		## question about its cause, and the lane's own fixture mixes them.
		["long_solid", [_long(200.0, "highway")], [Rect2i(0, 100, 900, 250)], []],
		["long_dashed", [_long(450.0, "road")], [Rect2i(0, 300, 900, 280)], []],
	]
	for case in cases:
		var cname: String = case[0]
		var roads: Array = case[1]
		var a := _mk(load("res://map_overlay.gd"), roads)
		var b := _mk(_WholeRun, roads)
		var blank := _mk(load("res://map_overlay.gd"), [])
		for f in 4:
			await get_tree().process_frame
		await RenderingServer.frame_post_draw
		var ia: Image = a[0].get_texture().get_image()
		var ib: Image = b[0].get_texture().get_image()
		var ibl: Image = blank[0].get_texture().get_image()
		if ia.get_data() == ibl.get_data():
			_bad("%s: the culled arm is byte-identical to an empty frame -- nothing drew" % cname)
		var ma := _ink_mask(ia)
		var mb := _ink_mask(ib)
		var d := _diff(ia, ib)
		_p("%-11s diff px=%d  max-delta=%d  delta>16=%d   ink(on)=%d ink(off)=%d"
			% [cname, d[0], d[1], d[2], _ink_in(ma, Rect2i(0, 0, W, H)),
				_ink_in(mb, Rect2i(0, 0, W, H))])
		if d[2] > 0:
			_bad("%s: %d pixel(s) differ by more than 16/255 -- structural, not antialiasing"
				% [cname, d[2]])
		for r in case[2]:
			var na := _ink_in(ma, r)
			var nb := _ink_in(mb, r)
			if na <= 0:
				_bad("%s: POSITIVE control %s has no ink with culling on (off has %d)"
					% [cname, str(r), nb])
			elif nb > 0 and (float(na) / float(nb) < 0.9 or float(na) / float(nb) > 1.1):
				_bad("%s: POSITIVE control %s ink %d(on) vs %d(off) -- more than 10 percent apart"
					% [cname, str(r), na, nb])
			else:
				_p("    +ctrl %-22s ink on=%-6d off=%-6d" % [str(r), na, nb])
		for r in case[3]:
			var na := _ink_in(ma, r)
			var nb := _ink_in(mb, r)
			if na > 0:
				_bad("%s: NEGATIVE control %s drew %d ink px with culling on -- a chord the polyline never had"
					% [cname, str(r), na])
			elif nb > 0:
				_bad("%s: NEGATIVE control %s blank with culling ON but %d px with it OFF -- the control is wrong"
					% [cname, str(r), nb])
			else:
				_p("    -ctrl %-22s blank in both arms" % str(r))
		for arr in [a, b, blank]:
			arr[0].queue_free()
		await get_tree().process_frame


## Does `draw_polyline` rasterise the SAME visible geometry identically when
## the array carries extra, far-off-screen points? Nothing from `map_overlay`
## is involved -- if this differs, the residual `_cull_probe` now reports is a
## Godot property and no amount of padding or dash-phase work closes it; if it
## does NOT differ, the lane's stated cause is wrong and the residual is theirs.
class _Poly extends Control:
	var pts := PackedVector2Array()
	var w := 2.3
	func _draw() -> void:
		if pts.size() >= 2:
			draw_polyline(pts, Color(0.824, 0.569, 0.216, 0.98), w, true)

func _phase_rasteriser() -> void:
	_p("---- phase 4: is the residual Godot's rasteriser? (no map_overlay involved)")
	## A straight run across the window with a long, far-off-screen tail on
	## each end. The visible stretch is byte-for-byte the same geometry in
	## both arms; only the array length differs.
	var full := PackedVector2Array()
	var vis := PackedVector2Array()
	var x := -6000.0
	while x <= 6000.0:
		var p := Vector2(x, 300.0 + 60.0 * sin(x / 250.0))
		full.append(p)
		if x >= -120.0 and x <= 1020.0:
			vis.append(p)
		x += 15.0
	var arms := []
	for src in [full, vis]:
		var vp := SubViewport.new()
		vp.size = Vector2i(W, H)
		vp.transparent_bg = false
		vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
		add_child(vp)
		var bg := ColorRect.new()
		bg.size = Vector2(W, H)
		bg.color = BG
		vp.add_child(bg)
		var pl := _Poly.new()
		pl.size = Vector2(W, H)
		pl.pts = src
		vp.add_child(pl)
		arms.append(vp)
	for f in 4:
		await get_tree().process_frame
	await RenderingServer.frame_post_draw
	var ia: Image = arms[0].get_texture().get_image()
	var ib: Image = arms[1].get_texture().get_image()
	var d := _diff(ia, ib)
	var ink := _ink_in(_ink_mask(ia), Rect2i(0, 0, W, H))
	_p("full(%d pts) vs visible-slice(%d pts): diff px=%d max-delta=%d delta>16=%d  ink=%d"
		% [full.size(), vis.size(), d[0], d[1], d[2], ink])
	if ink == 0:
		_bad("phase 4 drew nothing -- the comparison would be vacuous")
	if d[0] == 0:
		_p("    => IDENTICAL. Godot rasterises the same visible geometry the same way")
		_p("       regardless of array length, so the lane's stated cause for the")
		_p("       _cull_probe residual is REFUTED and the residual is map_overlay's.")
	else:
		_p("    => DIFFERENT at %d px, max %d/255. The lane's stated cause is reproduced"
			% [d[0], d[1]])
		_p("       outside their code entirely: no chain boundary, no dash phase, no pad.")
	for a in arms:
		a.queue_free()
	await get_tree().process_frame


func _phase_timing() -> void:
	_p("---- phase 3: redraw wall-clock, this process alone")
	DisplayServer.window_set_vsync_mode(DisplayServer.VSYNC_DISABLED)
	var roads := [_long(200.0, "highway"), _long(450.0, "road")]
	var out := {}
	for arm in [["NEW", load("res://map_overlay.gd")], ["OLD", _WholeRun]]:
		var t := _mk(arm[1], roads)
		var ov: Control = t[2]
		for f in 3:
			await get_tree().process_frame
		var samples := []
		for batch in 7:
			var t0 := Time.get_ticks_usec()
			for i in 30:
				ov.queue_redraw()
				await get_tree().process_frame
			samples.append(float(Time.get_ticks_usec() - t0) / 30.0)
		samples.sort()
		out[arm[0]] = samples
		_p("%s  per-redraw us: min=%.0f median=%.0f max=%.0f  (7 batches x 30 redraws)"
			% [arm[0], samples[0], samples[3], samples[6]])
		t[0].queue_free()
		await get_tree().process_frame
	_p("TIMING-RATIO median NEW/OLD = %.3f" % (out["NEW"][3] / out["OLD"][3]))
