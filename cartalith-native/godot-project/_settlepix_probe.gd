extends Control
## Lane SETTLE: "the settlements layer owns ZERO pixels at deep zoom when
## `urban_layouts` is on" -- does a revealed urban layout actually replace its
## own pin's pixels, or does the pin's suppression (`_urban_revealed`, gated
## at `map_overlay.gd:1984`) leave nothing drawn there at all?
##
## Windowed only -- `--headless` returns a null texture and
## `RenderingServer.frame_post_draw` never fires under it (MISTAKES.md).
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _settlepix_probe.tscn
##
## Method: instantiate a bare `MapOverlay` (no ancestor camera transform, so
## this control's local px == screen px) fed a real generated world's civ
## data, and diff a small box around one settlement's pin position against a
## true "nothing drawn" baseline (`_overlay.visible = false`) at three states:
##   1. shallow zoom (span > UM_FADE_FAR_KM -> alpha 0, urban layer off) --
##      the POSITIVE CONTROL: the pin must paint pixels here, or the diff
##      method itself is broken.
##   2. deep zoom (span <= UM_FADE_NEAR_KM -> alpha 1, `_urban_revealed` set
##      for the target) -- the actual question: does anything else paint the
##      same spot once the pin stands down?
## This is a palette-agnostic pixel-owns-it test (MISTAKES.md "Assert on
## pixels": a threshold check against one hardcoded color proves nothing; a
## diff against a captured true-blank frame does).

const SEED := 24601
const SIZE_KM := 96.0
const GW := 96
const GH := 64
const PX_PER_CELL := 6.0
const SAMPLE_R := 10

var _gen: WorldGen
var _overlay: Control
var _target_idx := -1
var _target_pos := Vector2.ZERO
var _fails := 0


func _ok(cond: bool, what: String) -> void:
	if cond:
		print("  PASS  %s" % what)
	else:
		_fails += 1
		print("  FAIL  %s" % what)


func _settle() -> void:
	await RenderingServer.frame_post_draw
	await get_tree().process_frame
	await RenderingServer.frame_post_draw


func _on_needed(indices: PackedInt32Array) -> void:
	_overlay.set_urban_layouts(indices, _gen.urban_layouts(indices))


## Count of pixels inside a `(2r+1)^2` box around `center` that differ between
## `base` (the true-blank capture) and `other`. Local px == screen px here
## (the probe control carries no ancestor camera scale), so `center` -- taken
## from `MapOverlay._cell_to_screen()`, the exact function `_draw()` itself
## uses -- lands at the same pixel in every captured image.
func _region_diff(base: Image, other: Image, center: Vector2, r: int) -> Dictionary:
	var count := 0
	var total := 0
	var w := base.get_width()
	var h := base.get_height()
	for dy in range(-r, r + 1):
		for dx in range(-r, r + 1):
			var x := int(center.x) + dx
			var y := int(center.y) + dy
			if x < 0 or y < 0 or x >= w or y >= h:
				continue
			total += 1
			if base.get_pixel(x, y) != other.get_pixel(x, y):
				count += 1
	return {"diff": count, "total": total}


func _ready() -> void:
	_gen = WorldGen.new()
	_gen.generate_sized(SEED, SIZE_KM, GW, GH)
	var settlements: Array = _gen.get_settlements()
	print("SETTLEPIX world %dx%d @ %.0f km, %d settlements" % [GW, GH, SIZE_KM, settlements.size()])

	## Pick a real (non-addon, pop > 0) settlement, PREFERRING the interior so
	## the plate-edge clip cannot drop it -- but the centre band is a preference,
	## not a guarantee, and at this seed it matches nothing.
	##
	## **Said plainly because the comment used to claim otherwise.** A verifier
	## found the stated rationale was not what ran: no settlement here falls in
	## 0.3-0.7 (the five sit at grid y=5,1,4,57,2), so the filter matched nothing
	## and the fallback silently took #0 at cy=0.078 -- 33 px from the top edge,
	## the opposite of “comfortably inside the interior”. The measurement is
	## unaffected (the 328/441 positive control proves the pin drew), but a probe
	## whose comment describes a path it did not take is evidence about the wrong
	## run. **So the probe now PRINTS which branch chose the target**, and the
	## reader never has to infer it.
	for i in settlements.size():
		var s: Dictionary = settlements[i]
		if int(s.get("population", 0)) <= 0:
			continue
		var cx: float = float(s["x"]) / GW
		var cy: float = float(s["y"]) / GH
		if cx > 0.3 and cx < 0.7 and cy > 0.3 and cy < 0.7:
			_target_idx = i
			break
	var _pick := "interior band"
	if _target_idx < 0:
		_pick = "FALLBACK (no settlement in the 0.3-0.7 band)"
		for i in settlements.size():
			if int(settlements[i].get("population", 0)) > 0:
				_target_idx = i
				break
	print("SETTLEPIX target chosen by: %s" % _pick)
	if _target_idx < 0:
		print("  FAIL  no populated settlement generated -- cannot test")
		get_tree().quit(2)
		return

	var t: Dictionary = settlements[_target_idx]
	print("SETTLEPIX target #%d \"%s\" kind=%s pop=%d faction=%d at (%.1f,%.1f)" % [
		_target_idx, String(t.get("name", "?")), String(t["kind"]), int(t["population"]),
		int(t.get("faction", 0)), float(t["x"]), float(t["y"])])

	size = Vector2(GW, GH) * PX_PER_CELL
	_overlay = preload("res://map_overlay.gd").new()
	_overlay.size = size
	_overlay.position = Vector2.ZERO
	add_child(_overlay)
	_overlay.urban_layouts_needed.connect(_on_needed)
	_overlay.set_civ_data(settlements, [], [], GW, GH, 0.0)
	_overlay.set_map_width_km(SIZE_KM)
	_overlay.set_show_settlements(true)
	_overlay.set_show_urban_layouts(true)

	var rect := Rect2(Vector2.ZERO, size)
	_target_pos = _overlay._cell_to_screen(Vector2(t["x"], t["y"]), rect)
	print("SETTLEPIX pin position ~ (%.1f, %.1f), control %s" % [_target_pos.x, _target_pos.y, size])
	await _settle()

	# -- baseline: overlay hidden, nothing drawn at all ---------------------
	_overlay.visible = false
	await _settle()
	var img_blank := get_viewport().get_texture().get_image()

	# -- shallow zoom: span way past UM_FADE_FAR_KM, urban layer off --------
	_overlay.visible = true
	_overlay.set_camera_zoom(1.0)
	_overlay.queue_redraw()
	await _settle()
	var img_shallow := get_viewport().get_texture().get_image()
	var span_shallow: float = _overlay._urban_span_km(rect)
	var alpha_shallow: float = _overlay._urban_layout_alpha(rect)
	var revealed_shallow: bool = _overlay._urban_revealed.has(_target_idx)

	# -- deep zoom: span inside UM_FADE_NEAR_KM, target should be revealed --
	_overlay.set_camera_zoom(12.0)
	var revealed_deep := false
	for attempt in 5:
		_overlay.queue_redraw()
		await _settle()
		revealed_deep = _overlay._urban_revealed.has(_target_idx)
		if revealed_deep:
			break
	var span_deep: float = _overlay._urban_span_km(rect)
	var alpha_deep: float = _overlay._urban_layout_alpha(rect)
	var img_deep := get_viewport().get_texture().get_image()

	img_blank.save_png("res://_settlepix_blank.png")
	img_shallow.save_png("res://_settlepix_shallow.png")
	img_deep.save_png("res://_settlepix_deep.png")

	var d_shallow := _region_diff(img_blank, img_shallow, _target_pos, SAMPLE_R)
	var d_deep := _region_diff(img_blank, img_deep, _target_pos, SAMPLE_R)
	print("SETTLEPIX shallow: span=%.2fkm alpha=%.2f revealed=%s diff=%d/%d px" % [
		span_shallow, alpha_shallow, revealed_shallow, d_shallow["diff"], d_shallow["total"]])
	print("SETTLEPIX deep:    span=%.2fkm alpha=%.2f revealed=%s diff=%d/%d px" % [
		span_deep, alpha_deep, revealed_deep, d_deep["diff"], d_deep["total"]])

	_ok(alpha_shallow <= 0.0, "shallow zoom: urban layer off (alpha<=0)")
	_ok(not revealed_shallow, "shallow zoom: target NOT in _urban_revealed")
	_ok(d_shallow["diff"] > 0, "shallow zoom: pin painted pixels (positive control -- diff method works)")
	_ok(alpha_deep >= 1.0, "deep zoom: urban layer fully opaque (alpha>=1.0)")
	_ok(revealed_deep, "deep zoom: target IS in _urban_revealed (pin suppressed)")
	_ok(d_deep["diff"] > 0, "deep zoom: something OTHER than the pin painted the same spot")

	## -- Part B, 2026-09-13: parcel/market/farmland/water fill triangulation
	## at z64 (`urban_layout_draw.gd::_fill_ground_polygon()`'s own doc
	## comment: the block-ground fill is fixed; the parcel district fill
	## beside it fails from the same transform-precision cause, but only past
	## a DEEPER zoom than this file's own z12 above -- parcel/market fills are
	## gated `detail >= 1.0` (`map_overlay.gd::_draw_urban_layouts()`:
	## `box_px >= URBAN_FINE_BOX_PX` = 620), and z12's box_px here is ~122.
	## Water and farmland fills carry no such gate and already ran at z12
	## above; repeated here too because z64 is where the brief asked to look,
	## not because their own gate needs it.
	##
	## Godot logs "Invalid polygon data, triangulation failed" to this
	## PROCESS's stderr and skips the draw -- nothing inside a script can
	## intercept the engine's own logger, so the count is read by whoever
	## invokes this probe, from its captured stderr. `PART_B_BEGIN`/
	## `PART_B_END` bound exactly the lines to grep between, so a failure from
	## any earlier phase of this same file (there should be none -- the block
	## fix already covers z12) cannot be miscounted as this one's.
	print("SETTLEPIX PART_B_BEGIN")
	_overlay.set_camera_zoom(64.0)
	## Computed from THIS probe's own SIZE_KM/size.x, not asserted against
	## itself: a different map area or width gets a different box_px at the
	## same zoom (`_urban_m_scale()`'s own ratio), so this is what tells the
	## reader whether z64 actually crosses the gate on THIS run, not merely
	## claims it does.
	var box_px_z64: float = 1.7 * 1000.0 * (size.x / (SIZE_KM * 1000.0)) * 64.0
	print("PART_B z64: box_px=%.1f  URBAN_FINE_BOX_PX=620  crosses=%s" % [
		box_px_z64, str(box_px_z64 >= 620.0)])
	for draw_i in 3:
		_overlay.queue_redraw()
		await _settle()
	var img_z64 := get_viewport().get_texture().get_image()
	img_z64.save_png("res://_settlepix_z64.png")
	print("SETTLEPIX PART_B_END")

	## B1: are the RAW (model-space, pre-transform) parcel polygons
	## themselves malformed, or -- as with the block fix -- is this the
	## transform collapsing a valid small polygon at a large absolute offset?
	## Read straight off `_urban_layouts`, the exact dictionary
	## `map_overlay.gd` draws from -- no re-derivation of its transform, which
	## is exactly what would risk drifting from what it actually does.
	var layout = _overlay._urban_layouts.get(_target_idx)
	if layout == null:
		_ok(false, "PART_B B1: target settlement's layout is not loaded -- cannot check")
	else:
		var parcels: Array = layout.get("parcels", [])
		var bad := 0
		for p in parcels:
			var poly: PackedVector2Array = p
			if poly.size() < 3:
				continue
			var flagged := false
			for a in poly.size():
				var pa: Vector2 = poly[a]
				if is_nan(pa.x) or is_nan(pa.y):
					flagged = true
					break
				for b in range(a + 1, poly.size()):
					if pa.distance_to(poly[b]) < 0.001:
						flagged = true
						break
				if flagged:
					break
			if flagged:
				bad += 1
		print("PART_B B1: %d parcels, %d with a NaN or near-duplicate vertex (raw model space)" % [
			parcels.size(), bad])
		_ok(bad == 0, "B1: no parcel's RAW polygon is malformed -- any triangulation failure is the transform")

	print("SETTLEPIX %s (%d failed)" % ["ALL PASS" if _fails == 0 else "SOME FAILED", _fails])
	get_tree().quit(1 if _fails > 0 else 0)
