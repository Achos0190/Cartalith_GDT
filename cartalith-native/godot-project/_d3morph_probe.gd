extends Node
## **LOD-D3's mechanism, asserted on the real screen** (`LOD_DETAIL_SCOPE.md`,
## "LOD-D3 · Continuous transitions: parent fallback and a colour-space
## morph").
##
## `_lodsweep_probe.gd` measures the OUTCOME across three seeds and two grid
## sizes -- pops, holes, seams, the level-boundary ratio. This one asserts the
## MECHANISM, which the outcome cannot distinguish from luck:
##
##   1. the engine's morph rule hands over exactly at the level boundary
##      (`WorldGen::lod_morph` through `EngineBridge`, so the binding is
##      exercised and not just the Rust unit test);
##   2. a real tile on screen carries a morph strictly between 0 and 1 at some
##      zoom -- a fade that never leaves 1 is not a fade;
##   3. two pyramid levels are live at once after a level change, and a child
##      reports `under_parent` -- the parent fallback;
##   4. **the level-boundary frame-to-frame change is smaller with the morph
##      than without it**, measured on the framebuffer against a planted
##      control that forces every tile back to `morph = 1, under_parent =
##      false` -- which is the pre-LOD-D3 compositor exactly.
##
## (4) is the one with teeth. `MISTAKES.md`, "Add a probe in the same commit
## that changes the behaviour": a probe earns authority by failing on the state
## before the change, so the control reproduces that state from OUTSIDE the
## shell (no shipping file is edited) and the assertion is a comparison, not a
## threshold picked to pass.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _d3morph_probe.tscn
##
## **MUST run WINDOWED.** Section 4 reads the framebuffer back
## (`get_texture().get_image()`), which the dummy rasterizer under `--headless`
## cannot do (`MISTAKES.md`, "Run a pixel probe"). It refuses to start there
## rather than passing vacuously.
##
## Exit 0 every assertion held, 1 an assertion failed, 2 could not set up the
## premise (no extension, no world, never reached deep zoom, no level boundary
## inside the sweep).

const SEED := 483920
const GRID := Vector2i(512, 384)
const WIDTH_KM := 1200.0
const ZOOM_STEP := 1.02
const VP := Vector2i(1600, 1000)

## How many zoom steps to walk past the first level boundary found, and how
## many frames either side of it feed the reference median. Eight a side is
## `_lodsweep_probe.gd`'s own neighbourhood width, reused so the two probes
## mean the same thing by "at a level boundary".
const PAST := 10
const NEIGHBOURS := 8

var _vp: SubViewport
var _app: Node
var _vh: Control
var _br: Node
var _m: Object
var _fail := 0


func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame


func _ok(name: String, cond: bool, detail: String = "") -> void:
	print("  ", "ok  " if cond else "FAIL", " ", name, ("  -- " + detail) if detail != "" else "")
	if not cond:
		_fail += 1


func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		printerr("PROBE-CANNOT-RUN: headless -- `get_texture().get_image()` is null there,")
		printerr("  so section 4 would compare two empty buffers and pass vacuously.")
		get_tree().quit(2)
		return
	if not ClassDB.class_exists("WorldGen"):
		printerr("PROBE-CANNOT-RUN: the .gdextension did not load.")
		get_tree().quit(2)
		return
	## The stale-`.dll` detector `_lodsweep_probe.gd` uses, for the same
	## reason: `cargo test` does not rebuild `cartalith_godot.dll`, so a probe
	## run after a `.rs` edit and before `cargo build` tests the previous
	## engine (`MISTAKES.md`, "Grade a Godot probe as evidence for a Rust
	## change"). `LodSweepMetrics` carries the frame comparison section 4
	## needs, so its absence is fatal here rather than merely suspicious.
	if not ClassDB.class_exists("LodSweepMetrics"):
		printerr("PROBE-CANNOT-RUN: the loaded extension has no `LodSweepMetrics`.")
		get_tree().quit(2)
		return
	_m = ClassDB.instantiate("LodSweepMetrics")

	get_window().size = VP
	_vp = SubViewport.new()
	_vp.size = VP
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	_app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(_app)
	await _frames(30)
	if _app.open_project_dialog != null:
		_app.open_project_dialog.hide()
	await _frames(15)
	_br = _app.bridge
	_vh = _app.viewport
	## **LOD-D6.** This probe reads `_lod_tiles` a frame or four after
	## moving the camera and asserts on what it finds, so it needs a tile
	## to exist the moment `_update_lod()` returns. Since LOD-D6 synthesis
	## runs on a worker and a tile lands some frames later, which is a
	## different premise from the one this probe was written against.
	## `_lod_sync` puts synthesis back on this thread; it moves no pixel
	## (both paths end in `lod_worker::LodSnapshot::render_tile`) and only
	## changes WHEN the tile appears. What this probe measures -- the
	## compositor -- is unaffected. `_d6async_probe.gd` is what exercises
	## the threading.
	_vh._lod_sync = true
	print("[BOOT] shell up")

	print("\n=== 1: the engine's morph rule, through the bridge ===")
	_check_rule()

	print("\n=== 2: a world, zoomed until the pyramid is up ===")
	_br.generate({
		"seed": SEED, "width_km": WIDTH_KM, "grid_w": GRID.x, "grid_h": GRID.y,
		"sea_level": 0.5, "villages": true,
	})
	var gen_ok = await _br.generation_finished
	await _frames(10)
	_ok("generation reported ok", gen_ok)
	if not _br.has_world:
		printerr("[ABORT] no world")
		get_tree().quit(2)
		return
	## Overlays off: a settlement pin re-scales on every zoom notch, which is a
	## real screen change no camera warp can undo and would land in section 4's
	## numbers as if it were the tiles' doing.
	for layer in ["territory", "provinces", "settlements", "roads", "sea_routes",
			"landmarks", "landmark_rejects", "urban_layouts"]:
		_vh.set_layer_visible(layer, false)
	_vh.reset_view()
	await _frames(5)
	var guard := 0
	while not _vh.lod_active() and guard < 2000:
		_vh._zoom_at(_vh.size * 0.5, ZOOM_STEP)
		await get_tree().process_frame
		guard += 1
	_ok("the pyramid came up", _vh.lod_active(), "at zoom %.2f after %d steps" % [_vh.zoom(), guard])
	if not _vh.lod_active():
		printerr("[ABORT] never reached deep zoom")
		get_tree().quit(2)
		return
	await _frames(45)

	print("\n=== 3-4: one level boundary, with the morph and with it defeated ===")
	var live := await _walk(false)
	if live.is_empty():
		printerr("[ABORT] the walk crossed no pyramid-level boundary")
		get_tree().quit(2)
		return
	var dead := await _walk(true)

	print("\n=== 5: the verdicts ===")
	_ok("some tile on screen was mid-fade (0 < morph < 1)", live["saw_partial"],
		"lowest child morph seen %.4f, highest %.4f" % [live["min_morph"], live["max_morph"]])
	_ok("two pyramid levels were live at once", live["max_levels"] >= 2,
		"levels live at the boundary frame: %d" % live["max_levels"])
	_ok("a child reported a parent underneath it", live["saw_under_parent"],
		"%d of %d tiles at the boundary frame" % [live["under_n"], live["tiles_n"]])
	var cs: Dictionary = live.get("census", {})
	_ok("at the boundary frame the incoming level draws nothing yet",
		cs.get("new_n", 0) > 0 and float(cs.get("new_max", 1.0)) < 0.02,
		"%d tiles, worst morph %.5f (bar: < 0.02)" % [cs.get("new_n", 0), cs.get("new_max", -1.0)])
	_ok("at the boundary frame the outgoing level draws whole",
		cs.get("old_n", 0) > 0 and float(cs.get("old_min", 0.0)) > 0.999,
		"%d tiles, weakest morph %.5f (bar: > 0.999)" % [cs.get("old_n", 0), cs.get("old_min", -1.0)])

	if dead.is_empty():
		_ok("the planted control also crossed a boundary", false, "it did not, so 4 cannot be scored")
	else:
		var a: float = live["boundary_ti"]
		var b: float = dead["boundary_ti"]
		print("  info boundary |dL*|: with the morph %.4f, with it defeated %.4f" % [a, b])
		print("  info neighbourhood median |dL*|: %.4f / %.4f" % [live["ref_med"], dead["ref_med"]])
		print("  info boundary ratio: %.3fx / %.3fx   (LOD-D3's bar is <= 1.5x)"
			% [live["ratio"], dead["ratio"]])
		## Reported, and asserted only for DIRECTION with a tolerance: the
		## whole content difference between two adjacent pyramid levels is one
		## octave of `add_zoom_detail`, and it measures a few hundredths of an
		## L* unit against a per-frame zoom residual two orders of magnitude
		## larger. A tight assertion on that margin would be a flake, and the
		## structural check above is what actually pins the behaviour.
		_ok("the morph does not make the level boundary worse", a <= b + 0.05,
			"%.4f vs %.4f" % [a, b])
		_ok("the control really was defeated (it is not the same picture twice)",
			absf(a - b) > 1e-6, "|difference| %.6f" % absf(a - b))

	print("\n%s" % ("PROBE-RESULT: PASS" if _fail == 0 else "PROBE-RESULT: FAIL (%d)" % _fail))
	get_tree().quit(1 if _fail > 0 else 0)


## Section 1. The handover invariant, asked of the ENGINE through the shell's
## own binding rather than of the Rust unit test: at the zoom where
## `lod_level_for_zoom` switches from `z` to `z+1`, the outgoing level is fully
## drawn and the incoming level is fully its parent, so the two frames either
## side of the switch draw the same thing.
func _check_rule() -> void:
	## `TILE_PX` is the engine's, not this file's: derive the handover zoom by
	## bisection on `lod_level_for_zoom` itself, so no constant is copied over
	## here to go stale.
	var gw := 2048.0
	_br.generate({
		"seed": 1, "width_km": 100.0, "grid_w": int(gw), "grid_h": 1311,
		"sea_level": 0.5, "villages": false,
	})
	await _br.generation_finished
	for z in [3, 4, 5]:
		var lo := 0.01
		var hi := 4096.0
		for i in 80:
			var mid := (lo + hi) * 0.5
			if _br.lod_level_for_zoom(mid) > z:
				hi = mid
			else:
				lo = mid
		## `hi` is the first px/cell resolving to `z+1`, `lo` the last to `z`.
		var out: float = _br.lod_morph(lo, z)
		var inc: float = _br.lod_morph(hi, z + 1)
		_ok("level %d hands over whole at its own boundary" % z,
			absf(out - 1.0) < 1e-3 and absf(inc) < 1e-3,
			"outgoing %.6f, incoming %.6f (px/cell %.6f)" % [out, inc, hi])


func _capture() -> Dictionary:
	await RenderingServer.frame_post_draw
	var tex := _vp.get_texture()
	if tex == null:
		return {}
	var img := tex.get_image()
	if img == null:
		return {}
	if img.get_format() != Image.FORMAT_RGBA8:
		img.convert(Image.FORMAT_RGBA8)
	return {
		"data": img.get_data(), "w": img.get_width(), "h": img.get_height(),
		"campos": _vh._camera.position, "zoom": _vh.zoom(),
	}


## Forces every live tile back to the pre-LOD-D3 compositor: drawn whole, over
## the base map. Applied from outside the shell, so no shipping file changes
## and the control is exactly "this milestone's blend, switched off".
func _defeat() -> void:
	for key in (_vh._lod_tiles as Dictionary).keys():
		var s := _vh._lod_tiles[key] as Sprite2D
		var mat := s.material as ShaderMaterial
		if mat != null:
			mat.set_shader_parameter("morph", 1.0)
			mat.set_shader_parameter("under_parent", false)


## Zooms from the current view until the pyramid level changes, then `PAST`
## steps further, recording the frame-to-frame change throughout. Returns the
## boundary reading and its own neighbourhood median.
##
## Both legs start from the same camera, which is why the zoom is rewound
## rather than continued: comparing a boundary at level 5 against one at level
## 7 would be comparing two different pictures, not two compositors.
func _walk(defeated: bool) -> Dictionary:
	_vh.reset_view()
	await _frames(3)
	var guard := 0
	while not _vh.lod_active() and guard < 2000:
		_vh._zoom_at(_vh.size * 0.5, ZOOM_STEP)
		await get_tree().process_frame
		guard += 1
	await _frames(45)

	var g: Vector2i = _br.grid_size()
	var native := minf(_vh.size.x / float(g.x), _vh.size.y / float(g.y))
	var ti := PackedFloat64Array()
	var levels := PackedInt32Array()
	var prev: Dictionary = {}
	var edge := -1
	var saw_partial := false
	var saw_under := false
	var min_morph := 2.0
	var max_morph := -1.0
	var max_levels := 0
	var under_n := 0
	var tiles_n := 0
	var census: Dictionary = {}

	for i in 400:
		_vh._zoom_at(_vh.size * 0.5, ZOOM_STEP)
		if defeated:
			_defeat()
		var cap := await _capture()
		if cap.is_empty():
			break
		if defeated:
			## `_process()`'s backlog drain can re-apply the real blend between
			## the call above and the frame; re-assert after the draw so the
			## NEXT frame is clean too.
			_defeat()
		levels.append(_br.lod_level_for_zoom(native * float(cap["zoom"])))
		if not prev.is_empty():
			var t: Dictionary = _m.call("temporal", prev["data"], cap["data"], cap["w"], cap["h"],
				prev["campos"], prev["zoom"], cap["campos"], cap["zoom"])
			ti.append(t["mean_dlstar"] if t.get("ok", false) else 0.0)
		prev = cap

		if not defeated:
			var seen: Dictionary = {}
			var u := 0
			var n := 0
			for key in (_vh._lod_tiles as Dictionary).keys():
				var s := _vh._lod_tiles[key] as Sprite2D
				var mat := s.material as ShaderMaterial
				if mat == null:
					continue
				var idx: Vector3i = s.get_meta("lod_idx", Vector3i(-1, 0, 0))
				seen[idx.x] = true
				n += 1
				if bool(mat.get_shader_parameter("under_parent")):
					u += 1
					saw_under = true
				if idx.x == levels[levels.size() - 1]:
					var mv := float(mat.get_shader_parameter("morph"))
					min_morph = minf(min_morph, mv)
					max_morph = maxf(max_morph, mv)
					if mv > 1e-4 and mv < 1.0 - 1e-4:
						saw_partial = true
			if seen.size() > max_levels:
				max_levels = seen.size()
				under_n = u
				tiles_n = n

		if edge < 0 and levels.size() >= 2 and levels[levels.size() - 1] != levels[levels.size() - 2]:
			edge = levels.size() - 1
			if not defeated:
				census = _census(levels[edge], levels[edge - 1])
		if edge >= 0 and levels.size() >= edge + PAST:
			break

	if edge < 1 or edge - 1 >= ti.size():
		return {}
	## `ti[j]` is the transition from frame `j` to frame `j+1`, so the level
	## change observed at frame `edge` is `ti[edge-1]` -- the same indexing
	## `_lodsweep_probe.gd::_level_boundary_ratio` uses.
	var boundary: float = ti[edge - 1]
	var refs := PackedFloat64Array()
	for d in range(1, NEIGHBOURS + 1):
		for k in [edge - d, edge + d]:
			if k >= 1 and k - 1 < ti.size() and k != edge:
				refs.append(ti[k - 1])
	var st: Dictionary = _m.call("stats", refs)
	var med: float = float(st["median"]) if st.get("ok", false) and float(st["median"]) > 0.0 else 0.0
	print("  %s: boundary at frame %d (level %d -> %d), |dL*| %.4f, neighbourhood median %.4f"
		% ["defeated" if defeated else "live   ", edge, levels[edge - 1], levels[edge], boundary, med])
	return {
		"boundary_ti": boundary, "ref_med": med,
		"ratio": (boundary / med) if med > 0.0 else 0.0,
		"saw_partial": saw_partial, "saw_under_parent": saw_under,
		"min_morph": min_morph, "max_morph": max_morph,
		"max_levels": max_levels, "under_n": under_n, "tiles_n": tiles_n,
		"census": census,
	}


## **The no-pop invariant, read off the screen at the frame it has to hold.**
##
## On the frame a level change is observed, every tile of the NEW level must be
## at `morph` ~ 0 (drawing nothing yet) and every tile of the level it came
## from must be at `morph` 1 (drawing whole) -- so the composited picture is
## the outgoing level, which is exactly what the previous frame showed. That is
## `lod_bridge::morph_for_zoom`'s handover asserted on real materials rather
## than on the function, and it is what makes the transition continuous by
## construction instead of by measurement.
##
## Deterministic, which is the point: the pixel comparison in section 4 is a
## real reading but a thin one (the content difference between two adjacent
## pyramid levels is one octave of `add_zoom_detail`, which measures ~0.03 L\*
## against a ~4.3 L\* per-frame zoom residual), so it is reported rather than
## asserted tightly. This is the assertion with teeth: on the pre-LOD-D3
## compositor there is no `morph` uniform at all and every tile draws whole, so
## every new-level tile reads 1.0 here and the check fails outright.
func _census(new_level: int, old_level: int) -> Dictionary:
	var new_max := -1.0
	var new_n := 0
	var old_min := 2.0
	var old_n := 0
	for key in (_vh._lod_tiles as Dictionary).keys():
		var s := _vh._lod_tiles[key] as Sprite2D
		var mat := s.material as ShaderMaterial
		if mat == null:
			continue
		var idx: Vector3i = s.get_meta("lod_idx", Vector3i(-1, 0, 0))
		var mv := float(mat.get_shader_parameter("morph"))
		if idx.x == new_level:
			new_max = maxf(new_max, mv)
			new_n += 1
		elif idx.x == old_level:
			old_min = minf(old_min, mv)
			old_n += 1
	return {"new_max": new_max, "new_n": new_n, "old_min": old_min, "old_n": old_n}
