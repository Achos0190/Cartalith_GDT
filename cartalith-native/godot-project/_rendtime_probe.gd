extends Node
## Measurement harness for the render-rows batch (2026-11-23). Every figure is PRINTED; the only
## assertions are premises (a drag that committed, a mark that moved pixels), never a bound.
##  --drag   ET-D4: a 200-waypoint drag commit on a 2048-wide world (`EDITING_TOOLS_SCOPE.md` risk 1).
##  --stack  ET-C2 / Curves: one live stack edit at 2048 -- `adjust_set` + `color_texture()` (the
##           engine rebuild) and `viewport.refresh()` (the shell's whole map refresh), per stack set.
##  --halo   Colour paint section 4: local-contrast halo at a Replace edge (1024 world).
##  --seam   ET-C2 river-bank seam under a strong water-only stack (1024 world).
##
##   godot --headless --path . res://_rendtime_probe.tscn -- --drag --stack --runs 7
##
## Needs no pixels, so headless is allowed. Timings: run alone with the RELEASE
## `cartalith_godot.dll` (`MISTAKES.md`, "Quote a timing"); medians with min..max.

const WAYPOINTS := 200
## Matches the shell default `WAY_DRAG_SPACING_CELLS` (Ruling HJ).
const SPACING := 6.0

var _bridge: Node
var _app: Node
var _fail := 0
var _out := ""

func _stat(label: String, t: Array) -> void:
	var s := t.duplicate()
	s.sort()
	print("%s (ms): median %.1f min %.1f max %.1f over %d runs" % [label, s[s.size() / 2], s[0], s[s.size() - 1], s.size()])

func _gen(w: int, h: int, km: float) -> void:
	var t0 := Time.get_ticks_msec()
	_bridge.generate({"seed": 483920, "width_km": km, "grid_w": w, "grid_h": h, "archetype": "", "villages": true, "sea_level": 0.45})
	while _bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().process_frame
	print("generated %dx%d in %d ms" % [w, h, Time.get_ticks_msec() - t0])

func _ready() -> void:
	var runs := 5
	var want := {}
	var a := OS.get_cmdline_user_args()
	var i := 0
	while i < a.size():
		if a[i] == "--runs":
			runs = int(a[i + 1])
			i += 1
		elif a[i] == "--out":
			_out = String(a[i + 1])
			i += 1
		else:
			want[String(a[i]).trim_prefix("--")] = true
		i += 1
	get_tree().create_timer(1500.0).timeout.connect(func() -> void: get_tree().quit(2))
	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.0).timeout
	_bridge = _app.bridge
	if _app.open_project_dialog:
		_app.open_project_dialog.hide()
	if want.has("drag") or want.has("stack"):
		await _gen(2048, 1312, 4000.0)
		if want.has("drag"):
			_drag(runs)
		if want.has("stack"):
			await _stack(runs)
	if want.has("halo") or want.has("seam"):
		await _gen(1024, 656, 2400.0)
		if want.has("halo"):
			await _halo()
		if want.has("seam"):
			await _seam()
	print("RESULT fail=%d" % _fail)
	get_tree().quit(1 if _fail > 0 else 0)

## The longest horizontal run of non-water pixels in the colour texture (rows every 8, water =
## blue-dominant), as `Vector3i(y, x0, length)`: a corridor a straight drag stays on land along,
## so no leg is unreachable (an unreachable leg re-routes unguided over the whole map, a different
## and far dearer thing than the 200-waypoint cost the scope asks for).
func _longest_land_run() -> Vector3i:
	var img: Image = _bridge.color_texture().get_image()
	img.convert(Image.FORMAT_RGB8)
	var d: PackedByteArray = img.get_data()
	var w := img.get_width()
	var best := Vector3i(0, 0, 0)
	for y in range(img.get_height() / 4, img.get_height() * 3 / 4, 8):
		var run0 := -1
		for x in w + 1:
			var land := false
			if x < w:
				var o := (y * w + x) * 3
				land = not (d[o + 2] > d[o] + 15 and d[o + 2] >= d[o + 1] - 5)
			if land and run0 < 0:
				run0 = x
			elif not land and run0 >= 0:
				if x - run0 > best.z:
					best = Vector3i(y, run0, x - run0)
				run0 = -1
	return best

## Rows are tried until one commits with no relaxed leg: a relaxed (unreachable) leg re-routes
## unguided over the whole map, a much dearer thing than the 200-waypoint cost the scope asks about,
## and the colour test for "land" is only a heuristic. Every row's time and relaxed count is printed.
func _drag(runs: int) -> void:
	print("
== ET-D4: %d-waypoint drag commit ==" % WAYPOINTS)
	var g: Vector2i = _bridge.grid_size()
	var spacing := 3.0
	var clean_row := -1
	var cand: Array = []
	for k in 9:
		cand.append(int(g.y * (0.25 + 0.06 * k)))
	for y in cand:
		var x0 := 300.0
		_bridge.way_begin("road")
		_bridge.way_stroke_begin({"way_type": "road", "spacing": spacing})
		for k in WAYPOINTS * 3:
			_bridge.way_stroke_sample(x0 + float(k) * spacing / 3.0, float(y) + 0.5, k * 8)
		var t := Time.get_ticks_usec()
		var res: Dictionary = _bridge.way_stroke_end()
		var ms := (Time.get_ticks_usec() - t) / 1000.0
		print("row %d: end %.1f ms, committed %s, relaxed legs %s" % [y, ms, res.get("committed", false), res.get("relaxed_legs", "-")])
		if bool(res.get("committed", false)):
			_bridge.way_stroke_undo()
			if int(res.get("relaxed_legs", 0)) == 0 and int(res.get("unreachable_legs", 0)) == 0 and clean_row < 0:
				clean_row = y
	if clean_row < 0:
		_fail += 1
		print("FAIL premise: no row gave a drag with every leg guided")
		return
	var times: Array = []
	for r in runs + 1:
		_bridge.way_begin("road")
		_bridge.way_stroke_begin({"way_type": "road", "spacing": spacing})
		var n := 0
		for k in WAYPOINTS * 3:
			n = _bridge.way_stroke_sample(300.0 + float(k) * spacing / 3.0, float(clean_row) + 0.5, k * 8)
		var t := Time.get_ticks_usec()
		var res: Dictionary = _bridge.way_stroke_end()
		var ms := (Time.get_ticks_usec() - t) / 1000.0
		if r > 0:   ## run 0 is a warm-up
			times.append(ms)
		print("clean row %d run %d: %d samples held, end %.1f ms, reply %s" % [clean_row, r, n, ms, res])
		if bool(res.get("committed", false)):
			_bridge.way_stroke_undo()
	_stat("way_stroke_end, %d waypoints, every leg guided (row %d)" % [WAYPOINTS, clean_row], times)

## One stack set per row: the same live-edit path the Adjustments panel takes on every drag step
## (`adjust_panel.gd::_apply`: `adjust_set`, then the render workspace's `_refresh_map` =
## `viewport.refresh()`). Curves' point is moved each run so no cache can serve it.
func _curves(y: float) -> String:
	return JSON.stringify({"entries": [{"kind": {"curves": {"points": [[0.0, 0.0], [128.0, y], [255.0, 255.0]]}}, "enabled": true, "opacity": 1.0}]})

func _stack(runs: int) -> void:
	print("\n== live stack edit at 2048x1312 ==")
	var vp: Node = _app.viewport
	var sets := {"none": [], "terrain": ["terrain"], "water": ["water"], "terrain+water": ["terrain", "water"], "levels terrain": ["terrain"], "hsl terrain": ["terrain"]}
	for name in sets:
		var tex: Array = []
		var full: Array = []
		for r in runs + 1:
			for l in ["terrain", "water"]:
				_bridge.adjust_set(l, '{"entries":[]}')
			for l in sets[name]:
				var doc := _curves(100.0 + 3.0 * r)
				if name.begins_with("levels"):
					doc = JSON.stringify({"entries": [{"kind": {"levels": {"in_black": 0.0, "in_white": 255.0, "gamma": 1.0 + 0.05 * r, "out_black": 0.0, "out_white": 255.0}}, "enabled": true, "opacity": 1.0}]})
				elif name.begins_with("hsl"):
					doc = JSON.stringify({"entries": [{"kind": {"hsl": {"hue_deg": 10.0 + r, "saturation": 0.0, "lightness": 0.0}}, "enabled": true, "opacity": 1.0}]})
				_bridge.adjust_set(l, doc)
			var t := Time.get_ticks_usec()
			_bridge.color_texture()
			var a := (Time.get_ticks_usec() - t) / 1000.0
			t = Time.get_ticks_usec()
			vp.refresh()
			var b := (Time.get_ticks_usec() - t) / 1000.0
			if r > 0:   ## run 0 is a warm-up
				tex.append(a)
				full.append(b)
		_stat("%s: color_texture()" % name, tex)
		_stat("%s: viewport.refresh()" % name, full)
	for l in ["terrain", "water"]:
		_bridge.adjust_set(l, '{"entries":[]}')

func _lum_image(tex: Texture2D) -> PackedFloat32Array:
	var img := tex.get_image()
	img.convert(Image.FORMAT_RGB8)
	var d := img.get_data()
	var out := PackedFloat32Array()
	out.resize(d.size() / 3)
	for p in out.size():
		out[p] = 0.3 * d[p * 3] + 0.59 * d[p * 3 + 1] + 0.11 * d[p * 3 + 2]
	return out

## Distance-from-the-paint-edge bands, in cells: [-6,-3), [-3,0), [0,3), [3,6) and the rest.
func _halo() -> void:
	print("
== colour paint section 4: local-contrast halo at a Replace edge ==")
	var g: Vector2i = _bridge.grid_size()
	var setts: Array = _bridge.settlements()
	var cx := g.x / 2
	var cy := g.y / 2
	if not setts.is_empty():
		cx = int(setts[0]["x"])
		cy = int(setts[0]["y"])
	var km_per_cell := 2400.0 / float(g.x)
	var radius := 30.0
	var ring: Array = []
	for k in 96:
		var ang := TAU * float(k) / 96.0
		ring.append([(cx + 0.5 + radius * cos(ang)) * km_per_cell, (cy + 0.5 + radius * sin(ang)) * km_per_cell])
	_bridge.set_appearance({"local_contrast": 0.0})
	var base_off := _lum_image(_bridge.color_texture())
	for lc in [0.55, 1.0]:
		_bridge.colour_paint_set_document('{"version":1,"visible":true,"marks":[]}')   ## the reference carries no paint
		_bridge.set_appearance({"local_contrast": lc})
		var base_on := _lum_image(_bridge.color_texture())
		for rgb in [[200, 30, 30], [90, 120, 60], [150, 140, 100]]:
			var doc := {"version": 1, "visible": true, "marks": [{"shape": {"kind": "area", "ring_km": ring, "edge_km": 0.0},
				"rgb": rgb, "opacity": 1.0, "hardness": 1.0, "blend": "replace", "land_only": true}]}
			_bridge.set_appearance({"local_contrast": 0.0})
			var r1: Dictionary = _bridge.colour_paint_set_document(JSON.stringify(doc))
			var tex_off: Texture2D = _bridge.color_texture()
			var paint_off := _lum_image(tex_off)
			_bridge.set_appearance({"local_contrast": lc})
			var tex_on: Texture2D = _bridge.color_texture()
			var paint_on := _lum_image(tex_on)
			## halo(p) = the local-contrast effect WITH the paint minus the effect without it.
			var bands := [0.0, 0.0, 0.0, 0.0]
			var far: Array = []
			var moved := 0
			var edge_step := 0.0
			for y in g.y:
				for x in g.x:
					var p := y * g.x + x
					var d := Vector2(x - cx, y - cy).length() - radius
					var halo := (paint_on[p] - paint_off[p]) - (base_on[p] - base_off[p])
					if absf(paint_off[p] - base_off[p]) > 0.5:
						moved += 1
					if d >= -6.0 and d < -3.0: bands[0] = maxf(bands[0], absf(halo))
					elif d >= -3.0 and d < 0.0: bands[1] = maxf(bands[1], absf(halo))
					elif d >= 0.0 and d < 3.0: bands[2] = maxf(bands[2], absf(halo))
					elif d >= 3.0 and d < 6.0: bands[3] = maxf(bands[3], absf(halo))
					elif d > 12.0:
						far.append(absf(halo))
					if d >= -1.0 and d < 1.0:
						edge_step = maxf(edge_step, absf(paint_off[p] - base_off[p]))
			far.sort()
			print("local_contrast %.2f, Replace %s (set ok=%s, %d px moved, edge step up to %.1f levels): max |halo| by band [-6,-3) %.2f, [-3,0) %.2f, [0,3) %.2f, [3,6) %.2f levels; outside, more than 12 cells away: max %.2f, p99 %.2f"
				% [lc, rgb, r1.get("ok", false), moved, edge_step, bands[0], bands[1], bands[2], bands[3], far[far.size() - 1], far[int(far.size() * 0.99)]])
			if moved < 200:
				_fail += 1
				print("FAIL premise: the mark moved only %d pixels" % moved)
			if _out != "" and lc == 1.0 and rgb[0] == 150:
				for pair in [["lc_off", tex_off], ["lc_on", tex_on]]:
					var img: Image = (pair[1] as Texture2D).get_image()
					img.convert(Image.FORMAT_RGB8)
					var crop := img.get_region(Rect2i(cx - 50, cy - 50, 100, 100))
					crop.resize(500, 500, Image.INTERPOLATE_NEAREST)
					crop.save_png("%s/halo_%s.png" % [_out, pair[0]])
	_bridge.colour_paint_set_document('{"version":1,"visible":true,"marks":[]}')

## ET-C2's bank-seam risk (`EDITING_TOOLS_SCOPE.md`, ET-C2 risks): the shore shader blends base texels
## with `river_color` texels, so where the two disagree on the water weight a step shows across the
## bank. Measured on the data the shader reads: at every pixel where the plain river texture EQUALS the
## plain base (no river there, so the two must stay equal), the adjusted pair must be equal too, under a
## strong water-only stack. Positive controls: the stack moved the base at water pixels and the river
## texture at river pixels (an empty diff is a failure, not a pass).
func _seam() -> void:
	print("
== ET-C2 river-bank seam under a strong water-only stack ==")
	for l in ["terrain", "water"]:
		_bridge.adjust_set(l, '{"entries":[]}')
	var base: Image = _bridge.color_texture().get_image()
	var riv0: Image = _bridge.river_color_texture().get_image()
	var strong := JSON.stringify({"entries": [{"kind": {"hsl": {"hue_deg": 150.0, "saturation": 0.0, "lightness": 0.35}}, "enabled": true, "opacity": 1.0}]})
	_bridge.adjust_set("water", strong)
	var adj: Image = _bridge.color_texture().get_image()
	var riv1: Image = _bridge.river_color_texture().get_image()
	for im in [base, riv0, adj, riv1]:
		(im as Image).convert(Image.FORMAT_RGB8)
	var b0: PackedByteArray = base.get_data()
	var b1: PackedByteArray = adj.get_data()
	var r0: PackedByteArray = riv0.get_data()
	var r1: PackedByteArray = riv1.get_data()
	var n := b0.size() / 3
	var same := 0
	var worst := 0
	var over1 := 0
	var interior_over1 := 0
	var base_moved := 0
	var river_moved := 0
	for p in n:
		var o := p * 3
		if r0[o] == b0[o] and r0[o + 1] == b0[o + 1] and r0[o + 2] == b0[o + 2]:
			same += 1
			var d := maxi(absi(r1[o] - b1[o]), maxi(absi(r1[o + 1] - b1[o + 1]), absi(r1[o + 2] - b1[o + 2])))
			worst = maxi(worst, d)
			var px := p % base.get_width()
			var py := p / base.get_width()
			if d > 1 and px >= 16 and py >= 16 and px < base.get_width() - 16 and py < base.get_height() - 16:
				interior_over1 += 1
				if interior_over1 <= 8:
					print("  INTERIOR seam px (%d,%d): plain base %s river %s -> adjusted base %s river %s" % [px, py, [b0[o], b0[o + 1], b0[o + 2]], [r0[o], r0[o + 1], r0[o + 2]], [b1[o], b1[o + 1], b1[o + 2]], [r1[o], r1[o + 1], r1[o + 2]]])
			if d > 1:
				over1 += 1
				if over1 <= 6 or over1 % 2000 == 0:
					print("  seam px #%d (%d,%d): plain base %s river %s -> adjusted base %s river %s" % [over1, p % base.get_width(), p / base.get_width(),
						[b0[o], b0[o + 1], b0[o + 2]], [r0[o], r0[o + 1], r0[o + 2]], [b1[o], b1[o + 1], b1[o + 2]], [r1[o], r1[o + 1], r1[o + 2]]])
		else:
			if absi(r1[o] - r0[o]) + absi(r1[o + 1] - r0[o + 1]) + absi(r1[o + 2] - r0[o + 2]) > 6:
				river_moved += 1
		if absi(b1[o] - b0[o]) + absi(b1[o + 1] - b0[o + 1]) + absi(b1[o + 2] - b0[o + 2]) > 6:
			base_moved += 1
	print("pixels where plain river texture == plain base: %d of %d; after the stack the pair differs by at most %d levels, %d pixels by more than 1; %d of those are 16+ px inside the map border" % [same, n, worst, over1, interior_over1])
	print("positive controls: the stack moved %d base pixels (water) and %d river-texture pixels (river)" % [base_moved, river_moved])
	if base_moved < 1000 or river_moved < 100:
		_fail += 1
		print("FAIL premise: the stack moved too little to test a seam")
	_bridge.adjust_set("water", '{"entries":[]}')
