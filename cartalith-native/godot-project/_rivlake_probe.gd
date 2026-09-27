extends Node
## Windowed proof that a river is no longer stroked across an above-sea lake
## (`OUTSTANDING_WORK.md` §2.5, 2026-09-24; the reference's
## `splitRiverPolylines`). Pixels, rivers on vs off:
##   1. non-vacuous: some drawn river's traced cells run through a lake
##      (`sample_cell`'s `water == "lake"`). Since RV-2 (2026-09-29) the cut is
##      `get_rivers()`' `pieces`, made once per crossing on the shoreline; the
##      per-point `lake_mask` this probe first read is gone;
##   2. at the middle of the longest lake stretch, rivers on/off changes ~no
##      pixels in a small box -- nothing is drawn over the open water;
##   3. control: at a dry point of the same river the same box DOES change,
##      so the river is drawn and the comparison can see it.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . --resolution 1600x1000 _rivlake_probe.tscn

var _fails := 0

func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame

func _check(what: String, cond: bool, detail: String = "") -> void:
	print("RIVLAKE %s  %s%s" % ["ok  " if cond else "FAIL", what,
		("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fails += 1

func _zoom_to(host, z: float, gx: float, gy: float) -> void:
	host.reset_view()
	await _frames(2)
	host.zoom_step(z / maxf(host.zoom(), 0.0001))
	await _frames(2)
	host.move_view_to(gx, gy)
	await get_tree().create_timer(1.5).timeout
	var guard := 0
	while host.lod_pending() > 0 and guard < 40:
		await get_tree().create_timer(0.2).timeout
		guard += 1

## Where grid point `g` lands on screen: the overlay's own mapping
## (`_point_to_screen(p, displayed_rect())`; the crisp pass multiplies by the
## zoom and draws under a 1/zoom transform, so the two cancel) through the
## overlay's global canvas transform. The control check below is what proves
## this mapping right.
func _screen_of(ov: Control, g: Vector2) -> Vector2i:
	return Vector2i(ov.get_global_transform_with_canvas() * ov._point_to_screen(g, ov.displayed_rect()))

## Until the deep-zoom pyramid has caught up (`lod_pending() == 0`) and the
## per-tile morph has run -- a condition, not a frame count.
func _settle_lod(host: Node) -> void:
	await _frames(6)
	var t0 := Time.get_ticks_msec()
	while host.lod_pending() > 0 and Time.get_ticks_msec() - t0 < 20000:
		await get_tree().process_frame
	await get_tree().create_timer(0.6).timeout
	await RenderingServer.frame_post_draw


## Worst per-channel difference at one pixel, in 0-255 levels.
func _worst(a: Image, b: Image, x: int, y: int) -> float:
	var pa := a.get_pixel(x, y)
	var pb := b.get_pixel(x, y)
	return maxf(absf(pa.r - pb.r), maxf(absf(pa.g - pb.g), absf(pa.b - pb.b))) * 255.0


## Pixels in the box that the Rivers switch moves by MORE than an identical
## re-render moves them. Three settled frames: `a` rivers on, `b` off, `c` on
## again. The switch rebuilds every deep-zoom tile, so `a` vs `c` is the
## control -- the same state rendered twice -- and a pixel counts only where
## `|a - b|` exceeds that pixel's own `|a - c|`. No tolerance is chosen: the
## control measures it.
func _box_diff(ov: Control, g: Vector2, half: int) -> int:
	var c := _screen_of(ov, g)
	print("RIVLAKE   grid %s -> screen %s" % [g, c])
	var host: Node = ov.get_parent()
	while host != null and not host.has_method("set_layer_visible"):
		host = host.get_parent()
	await _settle_lod(host)
	var a := get_viewport().get_texture().get_image()
	host.set_layer_visible("rivers", false)
	await _settle_lod(host)
	var b := get_viewport().get_texture().get_image()
	host.set_layer_visible("rivers", true)
	await _settle_lod(host)
	var cc := get_viewport().get_texture().get_image()
	var d := 0
	var worst := 0.0
	var noise := 0.0
	for y in range(c.y - half, c.y + half + 1):
		for x in range(c.x - half, c.x + half + 1):
			var dm := _worst(a, b, x, y)
			var dn := _worst(a, cc, x, y)
			worst = maxf(worst, dm)
			noise = maxf(noise, dn)
			if dm > dn:
				d += 1
	print("RIVLAKE   worst channel change in the box: switch %.1f levels, identical re-render (control) %.1f" % [worst, noise])
	return d

func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		print("RIVLAKE REFUSED: headless")
		get_tree().quit(2)
		return
	var wd := Timer.new()
	wd.wait_time = 300.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func(): print("RIVLAKE WATCHDOG"); get_tree().quit(3))
	wd.start()
	var app: Node = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.0).timeout
	var bridge: Node = app.bridge
	bridge.generate({"seed": 9137, "width_km": 2400.0, "grid_w": 384, "grid_h": 288,
		"archetype": "", "villages": true, "sea_level": 0.45})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	if app.open_project_dialog:
		app.open_project_dialog.hide()
	await _frames(6)
	var host: Node = app.viewport
	var ov: Control = host.overlay

	# 1. the longest lake stretch of any drawn river, in traced cells
	## The engine's own list (the overlay no longer holds one since 2026-09-27).
	var rivers: Array = bridge.rivers(1)
	var best_len := 0
	var best: Dictionary = {}
	var crossing := 0
	for r in rivers:
		var rd: Dictionary = r
		if not rd.has("widths") or rd.has("parallel_of"):
			continue
		var tp: PackedVector2Array = rd["points"]
		var run := 0
		var crossed := false
		for i in tp.size():
			var wet := String(bridge.sample_cell(int(tp[i].x), int(tp[i].y)).get("water", "")) == "lake"
			run = run + 1 if wet else 0
			crossed = crossed or wet
			if run > best_len:
				best_len = run
				best = {"lake": tp[i - run / 2]}
		if crossed:
			crossing += 1
	_check("some river crosses an above-sea lake", crossing > 0 and best_len >= 3,
		"%d rivers cross a lake; longest stretch %d traced cells" % [crossing, best_len])
	if best.is_empty():
		get_tree().quit(1)
		return
	## Control: a point well inside a drawn piece (3 render points from either
	## end) of the widest drawn river, so the stroke is thick enough to see at z12
	## -- and 10% of the grid in from every edge, where the map's frame is drawn
	## over the terrain (the widest point on this world first found sat at
	## x = 0.87, under the frame, and read 0 px).
	var dry := Vector2(-1, -1)
	var dry_w := -1.0
	for r in rivers:
		var rd: Dictionary = r
		if not rd.has("widths") or rd.has("parallel_of"):
			continue
		var ws: PackedFloat32Array = rd["widths"]
		var ps: PackedVector2Array = rd["render_points"]
		var pc: PackedInt32Array = rd["pieces"]
		for k in range(0, pc.size() - 1, 2):
			for i in range(pc[k] + 3, pc[k + 1] - 3):
				var q: Vector2 = ps[i]
				if q.x < ov._gw * 0.1 or q.x > ov._gw * 0.9 or q.y < ov._gh * 0.1 or q.y > ov._gh * 0.9:
					continue
				if ws[i] > dry_w:
					dry = ps[i]
					dry_w = ws[i]

	# 2. over the lake: nothing drawn
	var lp: Vector2 = best["lake"]
	await _zoom_to(host, 12.0, lp.x, lp.y)
	var lake_d := await _box_diff(ov, lp, 3)
	_check("over the lake, rivers on/off changes no pixels", lake_d == 0, "%d of 49 px" % lake_d)

	# 3. control: a dry point of the same river is drawn
	_check("control point found on a drawn river", dry.x >= 0, "%s, width %.2f cells" % [dry, dry_w])
	if dry.x >= 0:
		await _zoom_to(host, 12.0, dry.x, dry.y)
		var dry_d := await _box_diff(ov, dry, 3)
		_check("control: on dry ground the river IS drawn", dry_d > 0, "%d of 49 px" % dry_d)
	print("RIVLAKE %s  (%d failures)" % ["PASS" if _fails == 0 else "FAIL", _fails])
	get_tree().quit(0 if _fails == 0 else 1)
