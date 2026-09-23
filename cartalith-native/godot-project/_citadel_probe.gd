extends Control
## Ruling I / AC: is the citadel VISIBLE on a real generated settlement -- walled,
## astride the curtain, with an opening on its town side?
##
## Windowed only -- `ImageTexture`/viewport capture is vacuous under
## `--headless` (MISTAKES.md "Run a pixel probe").
##
##   Godot_v4.7.1-stable_win64.exe --path . _citadel_probe.tscn -- <out_dir>
##
## Method: one real world (seed 24601), its largest settlement raised to
## population 15 000 through the Place Editor's own bridge call, drawn by
## `UrbanLayoutDraw.draw_layout` twice from the SAME dictionary -- once whole
## ("after") and once with every `citadel_*` key erased ("before": exactly what
## the renderer drew for this model before the citadel existed). The diff must
## be non-zero inside the citadel's screen box (the positive control: the draw
## path really paints it) and ZERO outside it (nothing else moved). Then three
## sampled pixels: an outer corner is tower ink, the inner-face midpoint (the
## gate) is NOT wall ink, and a point on the inner face away from the gate IS.
## The background is a fixed colour this file paints, so no theme is involved.

const SEED := 24601
const VP := Vector2(1000, 760)
const BG := Color(0.87, 0.89, 0.78)

var _layout: Dictionary = {}
var _scale := 1.0
var _offset := Vector2.ZERO
var _fails := 0


func _ok(cond: bool, what: String) -> void:
	print(("  PASS  " if cond else "  FAIL  ") + what)
	if not cond:
		_fails += 1


func _to_screen(p: Vector2) -> Vector2:
	return p * _scale + _offset


func _draw() -> void:
	draw_rect(Rect2(Vector2.ZERO, VP), BG)
	if _layout.is_empty():
		return
	UrbanLayoutDraw.draw_layout(self, _layout, Callable(self, "_to_screen"), _scale, 1.0,
		1.0, false, 1.0)


func _capture() -> Image:
	queue_redraw()
	await RenderingServer.frame_post_draw
	await get_tree().process_frame
	await RenderingServer.frame_post_draw
	return get_viewport().get_texture().get_image()


## Fit a model-space rectangle into the viewport with a margin.
func _fit(r: Rect2) -> void:
	_scale = minf(VP.x / r.size.x, VP.y / r.size.y) * 0.9
	_offset = VP * 0.5 - r.get_center() * _scale


func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var out_dir: String = args[0] if args.size() > 0 else OS.get_user_data_dir()
	get_window().size = Vector2i(VP)
	size = VP

	var gen := WorldGen.new()
	gen.generate_sized(SEED, 96.0, 96, 64)
	gen.recompute_civilisation()
	var settlements: Array = gen.get_settlements()
	var idx := -1
	var best := -1
	for i in settlements.size():
		var p := int(settlements[i].get("population", 0))
		if p > best:
			best = p
			idx = i
	print("CITADEL target #%d \"%s\" pop=%d" % [idx, settlements[idx].get("name", "?"), best])

	# Under the tier: no citadel keys at all.
	_ok(gen.civ_edit_settlement(idx, {"population": 9000}), "population set to 9000")
	var under: Dictionary = gen.urban_layouts(PackedInt32Array([idx]))[0]
	print("CITADEL pop 9000: pop_target=%s walls=%s citadel_wall present=%s" % [
		under.get("pop_target"), under.get("walls"), under.has("citadel_wall")])
	_ok(not under.has("citadel_wall"), "a 9 000 town has no citadel")

	_ok(gen.civ_edit_settlement(idx, {"population": 15000}), "population set to 15000")
	_layout = gen.urban_layouts(PackedInt32Array([idx]))[0]
	print("CITADEL pop 15000: pop_target=%s wall_style=%s keys=%s" % [
		_layout.get("pop_target"), _layout.get("wall_style"),
		_layout.keys().filter(func(k): return String(k).begins_with("citadel"))])
	_ok(_layout.has("citadel_wall"), "a 15 000 town has a citadel")
	if not _layout.has("citadel_wall"):
		print("CITADEL SOME FAILED (%d failed)" % _fails)
		get_tree().quit(1)
		return
	var wall: PackedVector2Array = _layout["citadel_wall"]
	var ring: PackedVector2Array = _layout["wall_ring"]
	_ok(Geometry2D.is_point_in_polygon(wall[0], ring) and Geometry2D.is_point_in_polygon(wall[1], ring),
		"inner face inside the town ring")
	_ok(not Geometry2D.is_point_in_polygon(wall[2], ring) and not Geometry2D.is_point_in_polygon(wall[3], ring),
		"outer face outside the town ring")

	# Two framings: the whole town, and the citadel close up.
	var box := Rect2(Vector2.ZERO, Vector2(_layout["wm"], _layout["hm"]))
	var cbox := Rect2(wall[0], Vector2.ZERO)
	for p in wall:
		cbox = cbox.expand(p)
	for view in [["town", box], ["close", cbox.grow(cbox.size.length() * 0.9)]]:
		_fit(view[1])
		var after := await _capture()
		var full := _layout
		var stripped := _layout.duplicate()
		for k in _layout.keys():
			if String(k).begins_with("citadel"):
				stripped.erase(k)
		_layout = stripped
		var before := await _capture()
		_layout = full
		after.save_png(out_dir.path_join("citadel_%s_after.png" % view[0]))
		before.save_png(out_dir.path_join("citadel_%s_before.png" % view[0]))

		var sbox := Rect2(_to_screen(wall[0]), Vector2.ZERO)
		for p in wall:
			sbox = sbox.expand(_to_screen(p))
		sbox = sbox.grow(float(_layout.get("citadel_tower_r", 6.0)) * _scale + 3.0)
		var inside := 0
		var outside := 0
		for y in after.get_height():
			for x in after.get_width():
				if after.get_pixel(x, y) != before.get_pixel(x, y):
					if sbox.has_point(Vector2(x, y)):
						inside += 1
					else:
						outside += 1
		print("CITADEL %s: scale=%.3f px/m, changed px inside citadel box=%d outside=%d" % [
			view[0], _scale, inside, outside])
		_ok(inside > 0, "%s: the citadel paints pixels (positive control)" % view[0])
		_ok(outside == 0, "%s: nothing outside the citadel moved" % view[0])

		if view[0] == "close":
			var ink := UrbanLayoutDraw.WALL_STONE
			var tower := UrbanLayoutDraw.WALL_TOWER
			var near := func(c: Color, want: Color) -> bool:
				return absf(c.r - want.r) < 0.03 and absf(c.g - want.g) < 0.03 and absf(c.b - want.b) < 0.03
			var px := func(p: Vector2) -> Color:
				var s := _to_screen(p)
				return after.get_pixel(int(s.x), int(s.y))
			var gate: Vector2 = _layout["citadel_gate"]
			var face_pt: Vector2 = wall[0].lerp(wall[1], 0.2)
			print("CITADEL close: corner=%s gate=%s inner-face=%s" % [
				px.call(wall[2]), px.call(gate), px.call(face_pt)])
			_ok(near.call(px.call(wall[2]), tower), "outer corner carries a tower")
			_ok(not near.call(px.call(gate), ink) and not near.call(px.call(gate), tower),
				"the inner gate is an opening, not wall")
			_ok(near.call(px.call(face_pt), ink), "the inner face away from the gate is wall")

	print("CITADEL %s (%d failed)" % ["ALL PASS" if _fails == 0 else "SOME FAILED", _fails])
	get_tree().quit(1 if _fails > 0 else 0)
