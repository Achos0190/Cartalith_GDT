extends Node
## Why `_draw_dashed_polyline` was NOT collapsed into one `draw_multiline`.
##
##   Godot_v4.7.1-stable_win64.exe --path . _dashbatch_probe.tscn
##
## `MEMORY_OPTIMIZATION_SCOPE.md`'s 2026-08-25 Android diagnosis registered that
## collapse as its first lever: `_draw_dashed_polyline` emits one antialiased
## `draw_line` per dash, `urban_layout_draw.gd` had already made exactly that
## change for roof ink and recorded a 577 ms-a-redraw payoff, and the buffer
## memory tracked the drawn-object count. **Measured, the collapse buys nothing**
## -- and this probe is that measurement, kept so the lever is retired with a
## number rather than re-attempted.
##
## Two phases, both comparing N antialiased `draw_line`s against one
## `draw_multiline` over *identical* dash endpoints (the walk is shared, so the
## primitive is the only variable):
##
## 1. **Pixels.** Two `SubViewport`s, frames compared byte for byte, over every
##    dash pattern this shell draws, at the extremes of `set_way_scale()`'s own
##    0.2-2.5 clamp, inside the same `_crisp_begin()` 1/k transform the real
##    layer draws in, over a zigzag (dashes straddle vertices, which is why the
##    walk exists) and a straight run (they cannot). **108 of 108 identical** --
##    so the collapse would have been safe.
## 2. **Counters.** One arm at a time, reading Godot's own
##    `RENDER_TOTAL_OBJECTS_IN_FRAME` / `..._DRAW_CALLS_IN_FRAME`. **Identical.**
##    Godot's canvas renderer already batches adjacent same-material primitives
##    (a few hundred draw calls for hundreds of thousands of objects), and an
##    antialiased thick line expands to the same triangles either way. The
##    memory is the *geometry*, not the command count, so issuing the geometry
##    in one call rather than thousands moves nothing.
##
## What did move it was culling by viewport -- `map_overlay.gd`'s
## `_run_offscreen()`, checked by `_cull_probe`.

const W := 900
const H := 700

## dash / gap / width, in the same screen px `_draw_way_segment` passes, and the
## colour that goes with them. Sourced from `map_overlay.gd`'s own constants --
## the three dashed `WAY_STYLE` tiers, the sea lane, and both route weights.
const PATTERNS := [
	{"name": "road", "dash": 1.8, "gap": 1.3, "width": 0.7,
		"color": Color(0.627, 0.392, 0.235, 0.75)},
	{"name": "track", "dash": 1.3, "gap": 2.0, "width": 0.6,
		"color": Color(0.392, 0.471, 0.235, 0.75)},
	{"name": "ancient", "dash": 2.5, "gap": 1.3, "width": 0.65,
		"color": Color(0.471, 0.431, 0.392, 0.65)},
	{"name": "sea-lane", "dash": 2.6, "gap": 2.0, "width": 0.85,
		"color": Color(0.118, 0.510, 0.784, 0.7)},
	{"name": "route", "dash": 5.0, "gap": 3.0, "width": 1.5,
		"color": Color(0.784, 0.627, 0.235, 0.85)},
	{"name": "route-sel", "dash": 5.0, "gap": 3.0, "width": 2.5,
		"color": Color(1.0, 0.824, 0.314, 0.98)},
]
## `set_way_scale()`'s clamp floor, its rest value, and its ceiling.
const SCALES := [0.2, 1.0, 2.5]
## `_crisp_begin()`'s `k` -- the camera zoom the layer is drawn under.
const ZOOMS := [1.0, 4.0]


func _p(s: String) -> void:
	print("DASHBATCH  %s" % s)


## Two polylines a dash pattern behaves differently on, plus one built out of
## the degenerate cases the walk guards (a repeated vertex, a near-zero step).
static func _paths() -> Array:
	var zig := PackedVector2Array()
	for i in 24:
		zig.append(Vector2(60.0 + i * 33.0, 120.0 + (17.0 if i % 2 == 0 else -17.0)))
	var flat := PackedVector2Array([Vector2(60.0, 300.0), Vector2(840.0, 300.0)])
	var deg := PackedVector2Array([Vector2(60.0, 480.0), Vector2(60.0, 480.0),
		Vector2(400.0, 480.0), Vector2(400.0000001, 480.0), Vector2(840.0, 481.0)])
	return [zig, flat, deg]


## The dash walk, verbatim from `map_overlay.gd`, returning the endpoints rather
## than drawing them -- so both arms below stroke *identical* geometry and the
## only variable left is the primitive.
static func _dashes(points: PackedVector2Array, dash_len: float, gap_len: float) -> PackedVector2Array:
	var out := PackedVector2Array()
	var period := dash_len + gap_len
	var phase := 0.0
	for i in range(points.size() - 1):
		var p0 := points[i]
		var p1 := points[i + 1]
		var seg_vec := p1 - p0
		var seg_len := seg_vec.length()
		if seg_len <= 0.0:
			continue
		var dir := seg_vec / seg_len
		var traveled := 0.0
		while traveled < seg_len:
			var cycle_pos := fmod(phase, period)
			var on := cycle_pos < dash_len
			var remaining_in_state := (dash_len - cycle_pos) if on else (period - cycle_pos)
			var step := maxf(minf(remaining_in_state, seg_len - traveled), 0.001)
			if on:
				out.append(p0 + dir * traveled)
				out.append(p0 + dir * (traveled + step))
			traveled += step
			phase += step
	return out


## The arms' own "nothing was drawn" framebuffer, captured with an empty
## `_case`. Held as bytes rather than an Image so the per-case test is one `!=`.
var _blank: PackedByteArray = PackedByteArray()
var _vp_a: SubViewport
var _vp_b: SubViewport
var _case := {}


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
		_p("  Godot_v4.7.1-stable_win64.exe --path . _dashbatch_probe.tscn")
		get_tree().quit(2)
		return


	_vp_a = _make_vp(_ArmLines.new())
	_vp_b = _make_vp(_ArmMultiline.new())

	var fails := 0
	var checks := 0
	var saved := 0

	## ------------------------------------------------------- the blank control
	## **Pixel-equality alone is a test that cannot fail**, and this probe did not
	## have this until 2026-09-01. Two frames that drew nothing are byte-identical,
	## so if `_case` stopped reaching the arms, or either `_draw()` stopped
	## emitting, all 108 cases would come back "identical" and the probe would
	## report the collapse safe over zero coverage -- which is the conclusion
	## `MEMORY_OPTIMIZATION_SCOPE.md` MEM-5 rests on. `_cull_probe.gd` grew the
	## same control for the same reason; this is that idiom.
	##
	## An empty `_case` is exactly the "nothing was drawn" framebuffer: both arms
	## open `_draw()` with `if c.is_empty(): return`, so this is the arms' own
	## blank output rather than a colour guessed at here.
	_case = {}
	var blank_pair := await _capture()
	_blank = (blank_pair[0] as Image).get_data()
	if _blank.is_empty():
		_p("FAIL  the blank reference captured nothing; the ink test below would "
			+ "pass every case vacuously")
		fails += 1
	var inked := 0
	var paths := _paths()
	for path_i in paths.size():
		for pat in PATTERNS:
			for sc in SCALES:
				for z in ZOOMS:
					## `* z` first, exactly as `_stroke_points` does: the walk
					## runs in screen px, inside the 1/k transform.
					var pts := PackedVector2Array()
					for p: Vector2 in paths[path_i]:
						pts.append(p * z)
					_case = {"color": pat["color"], "zoom": z,
						"width": float(pat["width"]) * sc,
						"dashes": _dashes(pts, float(pat["dash"]) * sc, float(pat["gap"]) * sc)}
					var img := await _capture()
					checks += 1
					## One `get_data()` per arm, reused for both tests. The
					## per-pixel `_compare()` walk below still runs only on a
					## case that already failed -- 630k GDScript `get_pixel`
					## pairs x 108 cases is minutes of wall clock and would push
					## this probe past its own watchdog.
					var data_a := (img[0] as Image).get_data()
					var same: bool = data_a == (img[1] as Image).get_data()
					## Ink against the blank control -- NOT against the other
					## arm, which is the mistake that made the equality check
					## vacuous. A case where the `draw_line` arm drew nothing
					## proves nothing about the primitive under test.
					if not _blank.is_empty() and data_a != _blank:
						inked += 1
					if same:
						continue
					var diff := _compare(img[0], img[1])
					fails += 1
					_p("DIFF  %-10s path%d scale %.1f zoom %.1f -> %d px differ, max channel delta %d/255" % [
						pat["name"], path_i, sc, z, diff[0], diff[1]])
					if saved < 2:
						saved += 1
						img[0].save_png("user://dashbatch_lines_%s_%d.png" % [pat["name"], path_i])
						img[1].save_png("user://dashbatch_multi_%s_%d.png" % [pat["name"], path_i])
						_p("      captures -> %s" % ProjectSettings.globalize_path("user://"))

	_p("%d / %d cases pixel-identical, %d drew ink" % [checks - fails, checks, inked])
	## Every case here is a real dash pattern over a real path at a real scale;
	## none of them is a legal blank. One blank is a bug in the harness, and
	## `inked == 0` means the 108 "identical" results above are 108 pairs of
	## empty frames.
	if inked < checks:
		_p("FAIL  %d of %d cases rendered the bare background: the arms drew "
			% [checks - inked, checks]
			+ "nothing, so those 'identical' results compare empty frames")
		fails += 1

	## Phase 2 -- the counters, one arm at a time so the per-frame monitors
	## belong to a single arm. A long dense path, so the count is big enough
	## that a real difference could not hide in it.
	var heavy := PackedVector2Array()
	for i in 900:
		heavy.append(Vector2(20.0 + i * 0.97, 350.0 + sin(i * 0.21) * 300.0))
	_case = {"color": PATTERNS[0]["color"], "zoom": 1.0, "width": 0.7,
		"dashes": _dashes(heavy, 1.8, 1.3)}
	_p("phase 2: %d dashes, one arm per frame" % (_case["dashes"].size() / 2))
	var counts := []
	for arm in [_vp_a, _vp_b]:
		for other in [_vp_a, _vp_b]:
			other.render_target_update_mode = SubViewport.UPDATE_DISABLED
		arm.render_target_update_mode = SubViewport.UPDATE_ALWAYS
		arm.get_child(1).queue_redraw()
		for i in 4:
			await get_tree().process_frame
		await RenderingServer.frame_post_draw
		await get_tree().process_frame
		counts.append([int(Performance.get_monitor(Performance.RENDER_TOTAL_OBJECTS_IN_FRAME)),
			int(Performance.get_monitor(Performance.RENDER_TOTAL_DRAW_CALLS_IN_FRAME))])
		_p("  %-12s objects %8d  draw calls %6d" % [
			"draw_line" if arm == _vp_a else "draw_multiline", counts[-1][0], counts[-1][1]])
	var same_counts: bool = counts[0] == counts[1]

	## **Phase 2 was a print, not a check.** `same_counts` was computed, reported
	## in the RESULT line, and then dropped: `quit()` gated on `fails`, which
	## only phase 1 could raise. So the half of this probe that MEM-5 was
	## declined on -- "the counters are identical, so batching buys nothing" --
	## could go false without the exit code moving. It is an assertion now, in
	## both directions:
	##
	##   - a non-trivial object count in each arm, because `0 == 0` satisfies
	##     equality perfectly and would mean neither arm rendered at all;
	##   - the equality itself, because if it ever stops holding then MEM-5's
	##     declined lever is worth revisiting and a green run would hide that.
	if int(counts[0][0]) < 100 or int(counts[1][0]) < 100:
		_p("FAIL  an arm rendered almost nothing (objects %d vs %d over %d dashes); "
			% [counts[0][0], counts[1][0], _case["dashes"].size() / 2]
			+ "the counter comparison below is between two empty frames")
		fails += 1
	elif not same_counts:
		_p("FAIL  the counters DIFFER (draw_line %s, draw_multiline %s). "
			% [str(counts[0]), str(counts[1])]
			+ "MEM-5 was declined on these being identical -- the lever is worth "
			+ "revisiting, and this header's claim is now false.")
		fails += 1

	_p("RESULT: %s; %s" % [
		"pixels identical" if fails == 0 else "%d CHECK(S) FAILED" % fails,
		"counters identical -- batching buys nothing" if same_counts
			else "counters DIFFER -- batching is worth revisiting"])
	get_tree().quit(0 if fails == 0 else 1)


func _make_vp(arm: Control) -> SubViewport:
	var vp := SubViewport.new()
	vp.size = Vector2i(W, H)
	vp.transparent_bg = false
	vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(vp)
	var bg := ColorRect.new()
	bg.size = Vector2(W, H)
	bg.color = Color.BLACK
	vp.add_child(bg)
	arm.size = Vector2(W, H)
	arm.set_meta("probe", self)
	vp.add_child(arm)
	return vp


func case() -> Dictionary:
	return _case


func _capture() -> Array:
	_vp_a.get_child(1).queue_redraw()
	_vp_b.get_child(1).queue_redraw()
	for i in 3:
		await get_tree().process_frame
	await RenderingServer.frame_post_draw
	return [_vp_a.get_texture().get_image(), _vp_b.get_texture().get_image()]


## Whole-buffer equality first -- 630k `get_pixel` pairs per case in GDScript is
## minutes of wall clock, and the answer for an identical frame is one `==`.
## The per-pixel walk only runs on a case that already failed, where its cost
## buys the two numbers worth reporting.
func _compare(a: Image, b: Image) -> Array:
	if a.get_data() == b.get_data():
		return [0, 0]
	var n := 0
	var worst := 0
	for y in H:
		for x in W:
			var pa := a.get_pixel(x, y)
			var pb := b.get_pixel(x, y)
			if pa == pb:
				continue
			n += 1
			worst = maxi(worst, int(round(maxf(maxf(absf(pa.r - pb.r), absf(pa.g - pb.g)),
				absf(pa.b - pb.b)) * 255.0)))
	return [n, worst]


## Both arms enter and leave the same `_crisp_begin()`/`_crisp_end()` transform
## the real layer draws inside, so the comparison covers the transform
## interaction and not only the primitive in isolation.
class _ArmLines extends Control:
	## The frozen pre-2026-08-25 form: one antialiased `draw_line` per dash.
	func _draw() -> void:
		var c: Dictionary = get_meta("probe").case()
		if c.is_empty():
			return
		var k: float = c["zoom"]
		draw_set_transform(Vector2.ZERO, 0.0, Vector2(1.0 / k, 1.0 / k))
		var d: PackedVector2Array = c["dashes"]
		for i in range(0, d.size(), 2):
			draw_line(d[i], d[i + 1], c["color"], c["width"], true)
		draw_set_transform(Vector2.ZERO, 0.0, Vector2.ONE)


class _ArmMultiline extends Control:
	## The shipping form: the same endpoints, one call.
	func _draw() -> void:
		var c: Dictionary = get_meta("probe").case()
		if c.is_empty():
			return
		var k: float = c["zoom"]
		draw_set_transform(Vector2.ZERO, 0.0, Vector2(1.0 / k, 1.0 / k))
		var d: PackedVector2Array = c["dashes"]
		if not d.is_empty():
			draw_multiline(d, c["color"], c["width"], true)
		draw_set_transform(Vector2.ZERO, 0.0, Vector2.ONE)
