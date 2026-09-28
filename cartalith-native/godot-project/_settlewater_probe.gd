extends Control
## Windowed pixel proof for OUTSTANDING_WORK.md's "Settlements draw their own
## water on the map" row. Owner: settlements' own water fill on the MAIN map
## is either not needed (the base raster already shows it) or should be
## clipped below sea level; river connection is explicitly deferred.
##
## `map_overlay.gd`'s only `URBAN_DRAW.draw_layout()` call now passes
## `draw_water = false`. This probe proves the actual effect on real pixels:
## the SAME coastal settlement's SAME layout, at the SAME screen transform,
## drawn through the real main-map path (`_overlay` via `map_overlay.gd`,
## `draw_water = false`) versus the pre-fix path (`URBAN_DRAW.draw_layout()`
## called directly with `draw_water = true`, the old default) -- diffed
## against a true-blank baseline the same way `_settlepix_probe.gd` does
## (MISTAKES.md: "Assert on pixels").
##
## Godot_v4.7.1-stable_win64_console.exe --path . _settlewater_probe.tscn

const SEED := 24601
const SIZE_KM := 96.0
const GW := 96
const GH := 64
const PX_PER_CELL := 6.0
## The task's own zoom figure -- deep enough that a coastal town's water
## fill, if drawn, is many screen pixels across, not a sub-pixel sliver.
const ZOOM := 16.0

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


func _region_diff(base: Image, other: Image, center: Vector2, r: int) -> int:
	var count := 0
	var w := base.get_width()
	var h := base.get_height()
	for dy in range(-r, r + 1):
		for dx in range(-r, r + 1):
			var x := int(center.x) + dx
			var y := int(center.y) + dy
			if x < 0 or y < 0 or x >= w or y >= h:
				continue
			if base.get_pixel(x, y) != other.get_pixel(x, y):
				count += 1
	return count


func _ready() -> void:
	var gen := WorldGen.new()
	gen.generate_sized(SEED, SIZE_KM, GW, GH)
	var settlements: Array = gen.get_settlements()

	var target_idx := -1
	for i in settlements.size():
		var s: Dictionary = settlements[i]
		if bool(s.get("coastal", false)) and int(s.get("population", 0)) > 0:
			target_idx = i
			break
	if target_idx < 0:
		print("SETTLEWATER FAIL -- no coastal settlement generated at this seed")
		get_tree().quit(2)
		return

	var t: Dictionary = settlements[target_idx]
	print("SETTLEWATER target #%d \"%s\" coastal at (%.1f,%.1f)" % [
		target_idx, String(t.get("name", "?")), float(t["x"]), float(t["y"])])

	var layout_arr: Array = gen.urban_layouts(PackedInt32Array([target_idx]))
	if layout_arr.size() != 1:
		print("SETTLEWATER FAIL -- no layout for target")
		get_tree().quit(2)
		return
	var layout: Dictionary = layout_arr[0]
	var water_poly: PackedVector2Array = layout.get("water_poly", PackedVector2Array())
	var mask_runs: Array = layout.get("water_mask_runs", [])
	print("SETTLEWATER layout: water_poly=%d verts, water_mask_runs=%d runs" % [
		water_poly.size(), mask_runs.size()])
	_ok(water_poly.size() >= 3 or not mask_runs.is_empty(),
		"target's layout actually carries its own water geometry (else this proves nothing)")

	size = Vector2(GW, GH) * PX_PER_CELL
	var overlay := preload("res://map_overlay.gd").new()
	overlay.size = size
	overlay.position = Vector2.ZERO
	add_child(overlay)
	overlay.set_civ_data(settlements, [], [], GW, GH, 0.0)
	overlay.set_map_width_km(SIZE_KM)
	overlay.set_show_settlements(true)
	overlay.set_show_urban_layouts(true)
	overlay.set_urban_layouts(PackedInt32Array([target_idx]), layout_arr)

	var rect := Rect2(Vector2.ZERO, size)
	var center: Vector2 = overlay._cell_to_screen(Vector2(t["x"], t["y"]), rect)
	overlay.set_camera_zoom(ZOOM)
	await _settle()

	overlay.visible = false
	await _settle()
	var img_blank := get_viewport().get_texture().get_image()

	overlay.visible = true
	overlay.queue_redraw()
	await _settle()
	var img_after := get_viewport().get_texture().get_image()
	img_after.save_png("res://_settlewater_after.png")

	# -- the pre-fix path, same layout, same transform, called directly -------
	var to_screen := func(p: Vector2) -> Vector2: return overlay._cell_to_screen(p, rect)
	var m_scale: float = overlay._urban_m_scale(rect) if overlay.has_method("_urban_m_scale") else ZOOM
	# Draw the pre-fix (draw_water = true) pass on a throwaway child control so
	# it captures independently of the real overlay above.
	var before_layer := Control.new()
	before_layer.size = size
	add_child(before_layer)
	overlay.visible = false
	var draw_water_true := func() -> void:
		UrbanLayoutDraw.draw_layout(before_layer, layout, to_screen, m_scale, 1.0, 1.0, false, 1.0, true)
	before_layer.draw.connect(draw_water_true)
	before_layer.queue_redraw()
	await _settle()
	var img_before := get_viewport().get_texture().get_image()
	img_before.save_png("res://_settlewater_before.png")

	var r := 40
	var diff_after := _region_diff(img_blank, img_after, center, r)
	var diff_before := _region_diff(img_blank, img_before, center, r)
	print("SETTLEWATER diff near pin: after(draw_water=false, real map_overlay path)=%d px, before(draw_water=true)=%d px" % [
		diff_after, diff_before])

	_ok(diff_before > 0, "POSITIVE CONTROL: the old draw_water=true path paints real pixels near the town")
	_ok(diff_after < diff_before,
		"THE FIX: the real main-map path (draw_water=false) paints strictly fewer pixels there than the old path did")

	print("SETTLEWATER %s (%d failed)" % ["ALL PASS" if _fails == 0 else "SOME FAILED", _fails])
	get_tree().quit(1 if _fails > 0 else 0)
