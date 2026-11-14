extends Node
## Ruling EW item 1 (the town wall hugs the built-up area): the before/after frame pair, in the real shell's City
## Viewer, in the map look.
##
##   Godot_v4.7.2-stable_win64_console.exe --path . --resolution 1600x1000 _wallhug_probe.tscn -- --out <dir> [--seed N] [--min-buildings N]
##
## **Windowed, never `--headless`** (it reads rendered pixels), from a wiped user dir (the harness refuses `--headless`).
##
## What it protects (each check names itself; the frames are the evidence the numbers in `STATUS.md` cannot give):
##   1. THE SWITCH IS LIVE: `urban_wall_hug` is a real parameter, ON in the shipped app's parameters, and writing it
##      changes the very next `urban_layouts()` of the same settlement (the layout is regenerated on demand, never stored).
##   2. THE WALL MOVED IN: for the chosen walled town the circuit's enclosed area is smaller with the switch on, the
##      town's own buildings are the same set within the wall lots' tolerance, and every building is still inside or
##      outside the ring (the wall cuts none).
##   3. THE FRAMES DIFFER: the two City Viewer frames differ in the wall region (a positive control), and each has many
##      distinct colours (it drew).
## The town is the walled `curtain` town with the largest circuit-area reduction among those of at least `--min-buildings`.
## Frames: `<out>/wallhug_off.png`, `<out>/wallhug_on.png`, `<out>/wallhug_pair.png` (off left, on right).
## Result: `RESULT: PASS|FAIL (<n> failed) checks=<n>`; exit 0 pass, 1 fail, 2 bad argument or watchdog.

var _out := ""
var _vp: SubViewport
var _app: Node
var _fails := 0
var _checks := 0

func _ok(cond: bool, what: String) -> void:
	_checks += 1
	print(("  PASS  %s" if cond else "  FAIL  %s") % what)
	if not cond:
		_fails += 1

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _area(ring: PackedVector2Array) -> float:
	var a := 0.0
	for i in ring.size():
		var p := ring[i]
		var q := ring[(i + 1) % ring.size()]
		a += p.x * q.y - q.x * p.y
	return absf(a) * 0.5

func _distinct(img: Image) -> int:
	var seen := {}
	for y in range(0, img.get_height(), 3):
		for x in range(0, img.get_width(), 3):
			var c := img.get_pixel(x, y)
			seen[Vector3i(int(c.r * 15.0), int(c.g * 15.0), int(c.b * 15.0))] = true
	return seen.size()

func _shot(rect: Rect2) -> Image:
	await RenderingServer.frame_post_draw
	var img: Image = _vp.get_texture().get_image()
	return img.get_region(Rect2i(rect))

## A building is cut when some vertices are inside the ring and some are not.
func _cut_buildings(layout: Dictionary) -> int:
	var ring: PackedVector2Array = layout.get("wall_ring", PackedVector2Array())
	var n := 0
	for b in layout.get("buildings", []):
		var poly: PackedVector2Array = b
		if poly.size() < 3:
			continue
		var inside := 0
		for v in poly:
			if Geometry2D.is_point_in_polygon(v, ring):
				inside += 1
		if inside > 0 and inside < poly.size():
			n += 1
	return n

func _ready() -> void:
	get_tree().create_timer(900.0).timeout.connect(func() -> void:
		push_error("wallhug probe watchdog")
		get_tree().quit(2))
	if DisplayServer.get_name() == "headless":
		push_error("_wallhug_probe must be windowed")
		get_tree().quit(2)
		return
	var seed_n := 483920
	var min_bld := 1500
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		match args[i]:
			"--out":
				_out = args[i + 1]
			"--seed":
				seed_n = int(args[i + 1])
			"--min-buildings":
				min_bld = int(args[i + 1])
			_:
				push_error("unknown argument " + args[i])
				get_tree().quit(2)
				return
		i += 2
	if _out == "":
		push_error("--out <dir> is required")
		get_tree().quit(2)
		return
	DirAccess.make_dir_recursive_absolute(_out)
	_vp = SubViewport.new()
	_vp.size = Vector2i(1600, 1000)
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	_app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(_app)
	await get_tree().create_timer(1.0).timeout
	var bridge = _app.bridge
	bridge.generate({"seed": seed_n, "width_km": 800.0, "grid_w": 384, "grid_h": 288, "archetype": "", "villages": true})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	await _frames(6)

	print("\n== 1. the switch is live ==")
	var info: Dictionary = bridge.param_info("urban_wall_hug")
	print("  param_info: %s" % [info])
	_ok(not info.is_empty(), "urban_wall_hug is a registered parameter")
	_ok(bool(bridge.param_get("urban_wall_hug")), "urban_wall_hug is ON in the app's parameters (a new world)")
	var places: Array = bridge.settlements()
	var best := -1
	var best_gain := 0.0
	var pairs := {}
	for k in places.size():
		bridge.param_set("urban_wall_hug", false)
		var off_l: Array = bridge.urban_layouts(PackedInt32Array([k]))
		bridge.param_set("urban_wall_hug", true)
		var on_l: Array = bridge.urban_layouts(PackedInt32Array([k]))
		if off_l.is_empty() or on_l.is_empty():
			continue
		var a: Dictionary = off_l[0]
		var b: Dictionary = on_l[0]
		var ra: PackedVector2Array = a.get("wall_ring", PackedVector2Array())
		var rb: PackedVector2Array = b.get("wall_ring", PackedVector2Array())
		if ra.size() < 6 or rb.size() < 6 or String(a.get("wall_style", "")) != "curtain":
			continue
		if (a.get("buildings", []) as Array).size() < min_bld:
			continue
		var gain := 1.0 - _area(rb) / maxf(1.0, _area(ra))
		pairs[k] = gain
		if gain > best_gain:
			best_gain = gain
			best = k
	print("  %d walled curtain towns of >= %d buildings; best circuit-area reduction %.1f%% at settlement #%d" % [pairs.size(), min_bld, best_gain * 100.0, best])
	_ok(best >= 0 and best_gain > 0.0, "the switch changes a walled town's circuit (the same settlement, the next layout call)")
	if best < 0:
		_finish()
		return

	print("\n== 2. the wall moved in and cuts nothing ==")
	bridge.param_set("urban_wall_hug", false)
	var off_layout: Dictionary = bridge.urban_layouts(PackedInt32Array([best]))[0]
	bridge.param_set("urban_wall_hug", true)
	var on_layout: Dictionary = bridge.urban_layouts(PackedInt32Array([best]))[0]
	var ba := (off_layout.get("buildings", []) as Array).size()
	var bb := (on_layout.get("buildings", []) as Array).size()
	print("  buildings off %d, on %d; ring area off %.0f m2, on %.0f m2" % [ba, bb, _area(off_layout["wall_ring"]), _area(on_layout["wall_ring"])])
	_ok(_area(on_layout["wall_ring"]) < _area(off_layout["wall_ring"]), "the enclosed area is smaller with the switch on")
	_ok(absf(float(bb - ba)) / float(ba) < 0.10, "the building count moves by under 10%% (%d -> %d)" % [ba, bb])
	_ok(_cut_buildings(on_layout) == 0, "the new wall cuts no building (%d cut)" % _cut_buildings(on_layout))

	print("\n== 3. the frames (City Viewer, map look) ==")
	var imgs := {}
	for state in ["off", "on"]:
		bridge.param_set("urban_wall_hug", state == "on")
		_app.open_city_viewer(best)
		await _frames(12)
		var cv = _app.city_viewer_window
		cv.size = Vector2i(1560, 960)
		cv.position = Vector2i(20, 20)
		await _frames(8)
		cv._look_picker.select(0)
		cv._look_picker.item_selected.emit(0)
		await _frames(3)
		cv._reset_view()
		await _frames(6)
		var rect := Rect2(cv._canvas.get_global_transform_with_canvas().origin + Vector2(cv.position), cv._canvas.size)
		var im := await _shot(rect)
		im.save_png(_out.path_join("wallhug_%s.png" % state))
		imgs[state] = im
		_ok(_distinct(im) > 40, "the %s frame drew (%d distinct colours)" % [state, _distinct(im)])
		cv.hide()
		await _frames(4)
	var a_img: Image = imgs["off"]
	var b_img: Image = imgs["on"]
	var diff := 0
	var total := 0
	for y in range(0, mini(a_img.get_height(), b_img.get_height()), 4):
		for x in range(0, mini(a_img.get_width(), b_img.get_width()), 4):
			total += 1
			var ca := a_img.get_pixel(x, y)
			var cb := b_img.get_pixel(x, y)
			if absf(ca.r - cb.r) > 0.02 or absf(ca.g - cb.g) > 0.02 or absf(ca.b - cb.b) > 0.02:
				diff += 1
	print("  frames differ on %.1f%% of sampled pixels" % (100.0 * diff / float(maxi(1, total))))
	_ok(diff > 0, "the two frames differ (control: the switch moves pixels)")
	var pair := Image.create(a_img.get_width() + b_img.get_width() + 20, maxi(a_img.get_height(), b_img.get_height()), false, Image.FORMAT_RGBA8)
	pair.fill(Color(1, 1, 1, 1))
	pair.blit_rect(a_img, Rect2i(0, 0, a_img.get_width(), a_img.get_height()), Vector2i(0, 0))
	pair.blit_rect(b_img, Rect2i(0, 0, b_img.get_width(), b_img.get_height()), Vector2i(a_img.get_width() + 20, 0))
	pair.save_png(_out.path_join("wallhug_pair.png"))
	_finish()

func _finish() -> void:
	print("RESULT: %s (%d failed) checks=%d" % ["PASS" if _fails == 0 else "FAIL", _fails, _checks])
	get_tree().quit(0 if _fails == 0 else 1)
