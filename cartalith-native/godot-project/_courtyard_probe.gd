extends Control
## Ruling H / AD: do the outermost ring's dense blocks DRAW as the owner's
## perimeter blocks -- a continuous ring of ranges round an open court -- on a
## real generated settlement?
##
## Windowed only -- viewport capture is vacuous under `--headless`
## (MISTAKES.md "Run a pixel probe").
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _courtyard_probe.tscn -- <out_dir> <tag>
##
## `<tag>` names the PNGs. The before/after comparison is across TWO ENGINE
## BUILDS, not two dictionaries: the ring changes the plat itself, so there is
## no key to erase. Run this once against HEAD's `cartalith_godot.dll` with tag
## `before` and once against the working tree's with tag `after`; the `after`
## run writes the close-up box to `<out_dir>/courtyard_box.txt` and a `before`
## run reads it, so both close-ups frame the same ground. Run `after` first.
##
## Method: one real world (seed 24601), its largest settlement set to 6 000 --
## a walled market town like the owner's plan, and under the citadel tier so
## no citadel confounds the frame. `building_courtyard` flags both this ring
## and the old per-lot M-BLD-3 courtyard plan, which only market and burgher
## plots near the centre take, so the ring is the flagged buildings in the
## outer 30% of the town's extent from its centre of mass (Ruling AD's own
## band). In the `after` run there must be some; the close-up centres on the
## farthest. The positive control is the before/after pixel diff inside that
## box, taken across the two runs' PNGs (the renderer paints its own ground
## over the whole frame, so an in-run "is it blank" count cannot fail).

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


func _fit(r: Rect2) -> void:
	_scale = minf(VP.x / r.size.x, VP.y / r.size.y) * 0.95
	_offset = VP * 0.5 - r.get_center() * _scale


func _centroid(poly: PackedVector2Array) -> Vector2:
	var c := Vector2.ZERO
	for p in poly:
		c += p
	return c / maxf(1.0, poly.size())


func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var out_dir: String = args[0] if args.size() > 0 else OS.get_user_data_dir()
	var tag: String = args[1] if args.size() > 1 else "after"
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
	_ok(gen.civ_edit_settlement(idx, {"population": 6000}), "population set to 6000")
	_layout = gen.urban_layouts(PackedInt32Array([idx]))[0]
	var blds: Array = _layout["buildings"]
	var cy: PackedByteArray = _layout["building_courtyard"]
	# The town's centre of mass stands in for the market: the layout's
	# `market` key is not in the building polygons' frame on every site.
	var market := Vector2.ZERO
	for b in blds:
		market += _centroid(b)
	market /= maxf(1.0, blds.size())
	var max_d := 0.0
	for b in blds:
		max_d = maxf(max_d, _centroid(b).distance_to(market))
	var ring: Array[Vector2] = []
	var far := market
	for i in blds.size():
		var c := _centroid(blds[i])
		if cy[i] == 1 and c.distance_to(market) >= 0.7 * max_d:
			ring.append(c)
			if c.distance_to(market) > far.distance_to(market):
				far = c
	print("COURTYARD %s: settlement #%d buildings=%d courtyard-flagged=%d in outer band=%d" % [
		tag, idx, blds.size(), Array(cy).count(1), ring.size()])

	var box_path := out_dir.path_join("courtyard_box.txt")
	var cbox: Rect2
	if tag == "after":
		_ok(ring.size() > 0, "the outer band has perimeter-block ranges")
		cbox = Rect2(far - Vector2(70, 53), Vector2(140, 106))
		var f := FileAccess.open(box_path, FileAccess.WRITE)
		f.store_string("%f %f %f %f" % [cbox.position.x, cbox.position.y, cbox.size.x, cbox.size.y])
		f.close()
	else:
		var s := FileAccess.get_file_as_string(box_path).split(" ")
		cbox = Rect2(float(s[0]), float(s[1]), float(s[2]), float(s[3]))

	var tbox := Rect2(Vector2.ZERO, Vector2(_layout["wm"], _layout["hm"]))
	for view in [["town", tbox], ["close", cbox]]:
		_fit(view[1])
		var img := await _capture()
		img.save_png(out_dir.path_join("courtyard_%s_%s.png" % [view[0], tag]))

	print("COURTYARD %s (%d failed)" % ["ALL PASS" if _fails == 0 else "SOME FAILED", _fails])
	get_tree().quit(1 if _fails > 0 else 0)
