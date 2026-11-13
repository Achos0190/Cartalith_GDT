extends Node
## **3D-3 acceptance probe** (`3D_VIEWPORT_SCOPE.md` §5, "3D-3 · Standard
## geometry: quadtree, `L_3d` morph, layer arrays, culling"; bars §5.1).
## What it protects, one leg per line of the milestone and its bars:
##
## - **A. Entry and structure.** `Window ▸ 3D view` at a forced Standard (or
##   Full) tier draws the quadtree, not Basic's patch; the patch is the tier's
##   row (literal triangle counts); 2D's LOD tiles are parked while 3D shows
##   (§3.12); the card names the tiles and levels.
## - **B. B1, 2D/3D agreement, and B6, zoom gains detail.** At 2D zooms x4,
##   x16 and x64 (overlays hidden) the 2D frame is captured; the 3D view, top-
##   down orthographic at the same centre and scale, is captured over the same
##   pixels. Mean |dL*| <= 1.0 and p99 <= 4.0; the level index per pixel (the
##   debug view's mode 2) equals 2D's `round(L)` on >= 99.9 %; the 3D frame's
##   detail (mean luminance step) is >= 2D's at each zoom; the deepest drawn
##   level rises with the zoom.
## - **C. B2, no cracks.** The background is magenta; a scripted descent with
##   a yaw sweep through level changes counts magenta pixels with terrain above
##   them in the same column (a hole), every frame: 0.
## - **D. B3, no pops.** A slow dolly in: the mean absolute frame-to-frame
##   change T_i of every frame on which the drawn set changed is <= 1.5 x the
##   median of the ten frames around it. **Not** LOD-D0's depth-reprojected
##   T_i: the camera moves slowly enough that unreprojected frames differ
##   little, and that is said wherever this number is quoted.
## - **E. B4, the ground clamp,** on the quadtree's surface: a low orbit past
##   the highest ground, every frame above an independent bilinear sample of
##   the drawn node's vertex grid.
## - **F. Standard's (or Full's) frame and memory bars.** Static, orbiting and
##   zooming through level changes, 300 frames each after warm-up, vsync off:
##   median, min..max, p95 (bar p95 <= 16.7 ms desktop, 33 ms phone). Video
##   memory in 3D minus 2D's steady state at the same view <= the tier's §3.12
##   figure + 10 %.
## - **G. Controls with real input** (`Input.parse_input_event`): desktop wheel
##   and right drag; touch one-finger pan and two-finger pinch. The navpad's
##   −, + and ⌖ in 3D are `_view3d_terrain_probe.gd` leg G's (the camera is
##   shared, so the tier does not change them).
## - Touch here is a composition (`--force-touch`) on this OS: the tier row is
##   chosen by `View3DGate.phone_class(OS.get_name())`, so the phone's Standard
##   row (33², 64 layers) runs only on an Android device.
## - **H. Leaving 3D** gives back 2D's frame byte-identical once its parked
##   tiles have rebuilt.
##
## MUST run windowed; refuses headless with exit 2.
##
##   godot --path . --resolution 1600x900 _view3d_quadtree_probe.tscn -- --out DIR
##   godot --path . --resolution 1080x2340 _view3d_quadtree_probe.tscn -- --force-touch --out DIR
##
## Flags (an unknown one aborts with exit 2): `--out DIR` (required),
## `--force-touch`, `--grid WxH` (default 512x384), `--seed N` (default 483920),
## `--tier standard|full` (default standard), `--timing-only` (legs A and F),
## `--legs LETTERS` (run only those legs after A, e.g. `--legs bcd`; for the
## mutant runs).
## Result line: `RESULT checks=N fail=M`. Exit 0 all held, 1 a check failed,
## 2 could not run.

const ARGS: Array[String] = ["--out", "--force-touch", "--grid", "--seed", "--tier", "--timing-only", "--legs"]
const View3DGate := preload("res://shell/view3d_gate.gd")
const View3D := preload("res://shell/view3d.gd")
const View3DCamera := preload("res://shell/view3d_camera.gd")
## Triangles per patch by tier row, literals: 2(n-1)^2 surface + 8(n-1) skirt.
const PATCH_TRIS := {"standard_phone": 2304, "standard_desktop": 8704, "full": 33792}
## §3.12's GPU tile memory per row (MB, decimal), the memory bar's base.
const TIER_MB := {"standard_phone": 34.0, "standard_desktop": 136.0, "full": 272.0}

var app: Node
var vp: ViewportHost
var v: View3D
var _out := ""
var _checks := 0
var _fail := 0
var _touch := false
var _grid := Vector2i(512, 384)
var _seed := 483920
var _tier := View3DGate.TIER_STANDARD
var _timing_only := false
## The legs to run after A (`--legs`); every leg when empty.
var _legs := ""

func _ok(name: String, cond: bool, detail: String = "") -> void:
	_checks += 1
	if not cond:
		_fail += 1
	print("%s  %s  %s" % ["PASS" if cond else "FAIL", name, detail])

func _parse() -> bool:
	var a := OS.get_cmdline_user_args()
	var i := 0
	while i < a.size():
		match a[i]:
			"--out":
				_out = a[i + 1] if i + 1 < a.size() else ""
				i += 1
			"--force-touch":
				_touch = true
			"--grid":
				var p := (a[i + 1] if i + 1 < a.size() else "").split("x")
				if p.size() != 2:
					print("RESULT could not run: --grid wants WxH")
					return false
				_grid = Vector2i(int(p[0]), int(p[1]))
				i += 1
			"--seed":
				_seed = int(a[i + 1]) if i + 1 < a.size() else _seed
				i += 1
			"--tier":
				var t := a[i + 1] if i + 1 < a.size() else ""
				if t == "full":
					_tier = View3DGate.TIER_FULL
				elif t != "standard":
					print("RESULT could not run: --tier wants standard or full")
					return false
				i += 1
			"--timing-only":
				_timing_only = true
			"--legs":
				_legs = a[i + 1] if i + 1 < a.size() else ""
				if _legs == "":
					print("RESULT could not run: --legs wants letters")
					return false
				i += 1
			_:
				print("RESULT could not run: unknown argument %s -- accepted: %s" % [a[i], ", ".join(ARGS)])
				return false
		i += 1
	return _out != ""

func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		print("RESULT could not run: headless has no rasteriser")
		get_tree().quit(2)
		return
	if not _parse():
		print("RESULT could not run: need --out DIR")
		get_tree().quit(2)
		return
	DirAccess.make_dir_recursive_absolute(_out)
	DccSettings.set_view3d_result({})
	DccSettings.set_view3d_forced_tier(_tier)
	DccSettings.clear_view3d_exaggeration()
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(0.8).timeout
	vp = app.viewport
	var t0 := Time.get_ticks_msec()
	app.bridge.generate({"seed": _seed, "width_km": 1200.0, "grid_w": _grid.x, "grid_h": _grid.y,
		"archetype": "", "villages": true, "sea_level": 0.42})
	while app.bridge.generating:
		await get_tree().create_timer(0.25).timeout
	print("WORLD seed=%d grid=%dx%d generate_ms=%d" % [_seed, _grid.x, _grid.y, Time.get_ticks_msec() - t0])
	await get_tree().create_timer(0.6).timeout
	app.open_project_dialog.hide()
	await _frames(10)
	await _wait_no_coach()
	await _park_pointer()
	var ok := await _leg_a()
	if ok:
		if not _timing_only:
			if _run("b"):
				await _leg_b()
			if _run("c"):
				await _leg_c()
			if _run("d"):
				await _leg_d()
			if _run("e"):
				await _leg_e()
			if _run("g"):
				await _leg_g()
		if _run("f"):
			await _leg_f()
		if _run("h"):
			await _leg_h()
	DccSettings.set_view3d_result({})
	DccSettings.set_view3d_forced_tier(-1)
	DccSettings.clear_view3d_exaggeration()
	print("RESULT checks=%d fail=%d" % [_checks, _fail])
	get_tree().quit(1 if _fail > 0 else 0)

# -- helpers --------------------------------------------------------------------

func _run(leg: String) -> bool:
	return _legs == "" or _legs.contains(leg)

func _frames(n: int) -> void:
	for i in n:
		await RenderingServer.frame_post_draw

func _shot(name: String) -> Image:
	await _frames(3)
	var img := get_viewport().get_texture().get_image()
	if name != "":
		img.save_png(_out.path_join(name))
	return img

## The 3D SubViewport's own frame (no card, no navpad), saved when named.
func _shot3d(name: String) -> Image:
	await _frames(3)
	var img := v.subviewport().get_texture().get_image()
	if name != "":
		img.save_png(_out.path_join(name))
	return img

func _crop(img: Image, r: Rect2) -> Image:
	var ri := Rect2i(r).intersection(Rect2i(Vector2i.ZERO, img.get_size()))
	return img.get_region(ri)

func _rect2d() -> Rect2:
	var ins: Dictionary = vp._safe_insets
	return vp.get_global_rect().grow_individual(-float(ins.get("left", 0.0)), -float(ins.get("top", 0.0)),
		-float(ins.get("right", 0.0)), -float(ins.get("bottom", 0.0))) * get_viewport().get_final_transform()

func _wait_no_coach(limit_s := 30.0) -> void:
	var marks: Array = load("res://shell/dcc_shell.gd").get_script_constant_map().get("_COACH_MARKS", [])
	var texts: Array = marks.map(func(m): return String(m.get("text", "")))
	var t0 := Time.get_ticks_msec()
	while Time.get_ticks_msec() - t0 < limit_s * 1000.0:
		var up := false
		for l in app.find_children("*", "Label", true, false):
			if (l as Label).is_visible_in_tree() and texts.has((l as Label).text):
				up = true
				break
		if not up:
			return
		await get_tree().create_timer(0.25).timeout

func _park_pointer() -> void:
	if not _touch:
		var at := Vector2(4.0, get_viewport().get_visible_rect().size.y - 4.0) * get_viewport().get_final_transform().x.x
		_motion(at, Vector2.ZERO, 0)
		await _frames(4)

func _q():
	return v.quadtree()

## Waits until the quadtree has drawn something and nothing it asked for is
## outstanding, for `quiet` frames running (or times out).
func _settle(limit_s := 60.0, quiet := 8) -> bool:
	var t0 := Time.get_ticks_msec()
	var still := 0
	while Time.get_ticks_msec() - t0 < limit_s * 1000.0:
		await get_tree().process_frame
		var q = _q()
		if q != null and q.drawn_count > 0 and q._asked.is_empty():
			still += 1
			if still >= quiet:
				await _frames(3)
				return true
		else:
			still = 0
	return false

## Waits until the 3D view has placed its own entry pose (it does so once the
## root tile lands), so a pose the probe sets afterwards is not overwritten.
func _await_placed(limit_s := 60.0) -> void:
	var t0 := Time.get_ticks_msec()
	while Time.get_ticks_msec() - t0 < limit_s * 1000.0:
		await get_tree().process_frame
		if _q() != null and _q().world_known() and not v._place_pending and not v._frame_world_pending:
			break
	await get_tree().process_frame

## Waits until 2D's deep-zoom tiles have all landed.
func _settle_2d(limit_s := 60.0) -> void:
	var t0 := Time.get_ticks_msec()
	await _frames(4)
	while Time.get_ticks_msec() - t0 < limit_s * 1000.0:
		if vp.lod_pending() == 0:
			break
		await get_tree().process_frame
	await _frames(8)

## Mean absolute luminance step between neighbours, 0..255: LOD-D0's detail
## per pixel.
func _detail(c: Image) -> float:
	var w := c.get_width()
	var h := c.get_height()
	var s := 0.0
	var n := 0
	for y in range(0, h - 1, 2):
		for x in range(0, w - 1, 2):
			var l := c.get_pixel(x, y).get_luminance()
			s += absf(l - c.get_pixel(x + 1, y).get_luminance()) + absf(l - c.get_pixel(x, y + 1).get_luminance())
			n += 2
	return s / maxf(n, 1) * 255.0

static func _lstar(c: Color) -> float:
	var lin := func(u: float) -> float: return u / 12.92 if u <= 0.04045 else pow((u + 0.055) / 1.055, 2.4)
	var y: float = 0.2126 * lin.call(c.r) + 0.7152 * lin.call(c.g) + 0.0722 * lin.call(c.b)
	var f := pow(y, 1.0 / 3.0) if y > 0.008856 else 7.787 * y + 16.0 / 116.0
	return 116.0 * f - 16.0

## The highest point of the root's relief, as a sample position (cells).
func _highest_cell() -> Vector2:
	var img: Image = _q()._ref_img
	var best := -1.0
	var at := Vector2i.ZERO
	for y in img.get_height():
		for x in img.get_width():
			var h := img.get_pixel(x, y).r
			if h > best:
				best = h
				at = Vector2i(x, y)
	return Vector2(at.x / float(img.get_width() - 1) * (_grid.x - 1), at.y / float(img.get_height() - 1) * (_grid.y - 1))

## A land cell near the world's centre (relief above sea + 0.05).
func _land_cell() -> Vector2:
	var img: Image = _q()._ref_img
	var sea: float = _q()._sea
	var best := INF
	var at := Vector2i(img.get_width() / 2, img.get_height() / 2)
	for y in range(0, img.get_height(), 2):
		for x in range(0, img.get_width(), 2):
			if img.get_pixel(x, y).r > sea + 0.05:
				var d := Vector2(x - img.get_width() / 2, y - img.get_height() / 2).length()
				if d < best:
					best = d
					at = Vector2i(x, y)
	return Vector2(at.x / float(img.get_width() - 1) * (_grid.x - 1), at.y / float(img.get_height() - 1) * (_grid.y - 1))

func _aim(cell: Vector2, span_m: float, pitch_deg: float, yaw_deg := 0.0) -> void:
	var c: View3DCamera = v.camera_state()
	c.set_ortho(false)
	c.pivot_x = cell.x * c.cell_m
	c.pivot_z = cell.y * c.cell_m
	c.dist = clampf(span_m * 0.5 / tan(deg_to_rad(View3DCamera.FOV_DEG) * 0.5), c.min_dist(), c.max_dist())
	c.pitch = deg_to_rad(pitch_deg)
	c.yaw = deg_to_rad(yaw_deg)
	c.tilt_left = 0.0

func _time(n: int, each: Callable) -> Dictionary:
	var ts := PackedFloat64Array()
	var last := Time.get_ticks_usec()
	for i in n:
		each.call(i)
		await get_tree().process_frame
		var now := Time.get_ticks_usec()
		ts.append((now - last) / 1000.0)
		last = now
	ts.sort()
	return {"frames": n, "median": ts[n / 2], "min": ts[0], "max": ts[n - 1], "p95": ts[mini(n - 1, int(n * 0.95))]}

func _mouse(pos: Vector2, button: int, pressed: bool) -> void:
	var e := InputEventMouseButton.new()
	e.position = pos
	e.global_position = pos
	e.button_index = button
	e.pressed = pressed
	Input.parse_input_event(e)

func _motion(pos: Vector2, rel: Vector2, mask: int) -> void:
	var e := InputEventMouseMotion.new()
	e.position = pos
	e.global_position = pos
	e.relative = rel
	e.button_mask = mask
	Input.parse_input_event(e)

func _touch_ev(idx: int, pos: Vector2, pressed: bool) -> void:
	var e := InputEventScreenTouch.new()
	e.index = idx
	e.position = pos
	e.pressed = pressed
	Input.parse_input_event(e)

func _drag_ev(idx: int, pos: Vector2, rel: Vector2) -> void:
	var e := InputEventScreenDrag.new()
	e.index = idx
	e.position = pos
	e.relative = rel
	Input.parse_input_event(e)

## Opens 3D through `Window ▸ 3D view` (the gate runs on the first entry).
func _enter() -> bool:
	var win: PopupMenu = null
	for ch in app.menu_bar_row.get_children():
		if ch is MenuButton:
			var p := (ch as MenuButton).get_popup()
			if _find_id(p, DccMenus.ID_WIN_VIEW_3D) != null:
				win = _find_id(p, DccMenus.ID_WIN_VIEW_3D)
	if win == null:
		return false
	win.id_pressed.emit(DccMenus.ID_WIN_VIEW_3D)
	v = vp.view3d()
	var t0 := Time.get_ticks_msec()
	while v != null and v.busy() and Time.get_ticks_msec() - t0 < 120000:
		await get_tree().process_frame
	return v != null

func _find_id(p: PopupMenu, id: int) -> PopupMenu:
	if p.get_item_index(id) >= 0:
		return p
	for c in p.get_children(true):
		if c is PopupMenu:
			var f := _find_id(c as PopupMenu, id)
			if f != null:
				return f
	return null

# -- legs -----------------------------------------------------------------------

var _fit_span := 0.0
var _skirt_share := 0.0
## The B2 skirt check's limit: a fraction of sampled pixels painted as skirt
## over the descent. A labelled judgement, about 9x what seed 483920 measured
## (0.00011, 2026-11-13), so a level boundary the skirts are papering over
## at scale fails it. It does NOT see the morph removed: that measured 0.00012
## (mutant m8), because with low-passed geometry the morph's visible share is
## about 125 pixels in 11 million on this path.
const SKIRT_SHARE_MAX := 0.001
var _mem_2d := 0.0

func _leg_a() -> bool:
	if not await _enter():
		_ok("A: Window ▸ 3D view opens the 3D view", false)
		return false
	_ok("A: the forced tier is in force", v.tier() == _tier, "tier %s" % View3DGate.TIER_NAMES[v.tier()])
	var settled := await _settle()
	var q = _q()
	_ok("A: the quadtree draws", settled and q != null and q.drawn_count > 0, "" if q == null else "%d nodes, levels %d..%d, reason '%s'" % [q.drawn_count, q.level_min, q.level_max, q.reason])
	if q == null:
		return false
	_ok("A: Basic's patch is not drawn under it", not v.terrain()._win_mi.visible and not v.terrain()._back_mi.visible)
	var key: String = q.tier_key()
	var tris := int(q._mm.mesh.get_faces().size() / 3)
	_ok("A: the patch is the tier's row (%s)" % key, tris == int(PATCH_TRIS[key]), "%d triangles per patch" % tris)
	_ok("A: 2D's LOD tiles are parked while 3D shows (§3.12)", vp._lod_tiles.is_empty() and vp._lod_cache.is_empty(), "%d live, %d cached" % [vp._lod_tiles.size(), vp._lod_cache.size()])
	## And stay parked when 2D is asked to update while 3D shows (a resize or a
	## repaint reaches `_update_lod()`; called here as that stand-in). Freeing
	## the tiles on entry alone passed the check above with the park flag
	## mutated off (2026-11-13 mutant m3).
	vp._reset_view_2d()
	vp._zoom_at(vp.size * 0.5, 16.0)
	vp._update_lod()
	await _frames(20)
	_ok("A: a 2D update while 3D shows builds no 2D tile", vp._lod_tiles.is_empty() and vp.lod_pending() == 0, "%d live, %d pending" % [vp._lod_tiles.size(), vp.lod_pending()])
	vp._reset_view_2d()
	_ok("A: the card names the tiles and levels", v._info.text.find("tiles at level") >= 0, v._info.text)
	var c: View3DCamera = v.camera_state()
	_fit_span = maxf(c.world_h_m, c.world_w_m / (float(v.subviewport().size.x) / v.subviewport().size.y))
	await _shot("a_entry.png")
	return settled

## Sets 2D to `zoom` x fit about `cell`, zooming in by steps of 1.25 as a user
## does, and waits for its tiles. Not one jump: 2D blends its first LOD level
## with the base raster until the camera first changes level
## (`ViewportHost._lod_parents_due`, LOD-D3's entry band), so a jump straight
## to x4 compared 3D's parent-tile blend with 2D's base-raster one (measured
## 2026-11-12: x4 p99 |dL*| 5.2 over all pixels, 0.55 where no partner shows).
func _set_2d(cell: Vector2, zoom: float) -> void:
	vp._reset_view_2d()
	await _frames(2)
	vp.move_view_to(cell.x, cell.y)
	var z := 1.0
	while z < zoom:
		var f := minf(1.25, zoom / z)
		vp._zoom_at(vp.size * 0.5, f)
		vp.move_view_to(cell.x, cell.y)
		z *= f
		await _frames(2)
	await _settle_2d()

func _leg_b() -> void:
	## On the highest ground, so the frames carry relief and colour detail
	## (a first run centred on the land cell nearest the middle framed mostly
	## open sea at x16 and x64, which proves little about agreement).
	var land := _highest_cell() + Vector2(4.0, 4.0)
	var ef9: bool = vp._ef9_enabled()
	var prev_deep := -1
	for k in [4, 16, 64]:
		v.set_debug_levels(0)
		vp.set_3d(false)
		vp.overlay.visible = false
		await _set_2d(land, float(k))
		var r2 := _rect2d()
		var two := _crop(await _shot("b_2d_x%d.png" % k), r2)
		var expect := vp.view_for_3d()
		var lam: float = vp._bridge.lod_continuous_level(float(expect["px_per_cell"]))
		var gain: float = vp._ef9_gain if ef9 else 0.0
		vp.set_3d(true)
		await _await_placed()
		var c: View3DCamera = v.camera_state()
		c.enter_from_2d(expect["centre"], float(expect["px_per_cell"]), float(v.subviewport().size.y), false)
		v._refresh()
		await _settle()
		var three_full := await _shot3d("b_3d_x%d.png" % k)
		## The same pixels: the 3D view covers the host, so crop it by the 2D
		## rect's offset inside the host.
		var off := r2.position - v.get_global_rect().position * get_viewport().get_final_transform().x.x
		var three := _crop(three_full, Rect2(off, r2.size))
		var inner := Rect2(Vector2(two.get_size()) * 0.15, Vector2(two.get_size()) * 0.7)
		var a := _crop(two, inner)
		var b := _crop(three, inner)
		var dl := PackedFloat32Array()
		var own := PackedFloat32Array()
		var boost_img: Image = null
		if gain > 0.0 and vp._bridge.lod_ef9_boost_texture() != null:
			boost_img = vp._bridge.lod_ef9_boost_texture().get_image()
		for y in range(0, a.get_height(), 2):
			for x in range(0, a.get_width(), 2):
				var d := absf(_lstar(a.get_pixel(x, y)) - _lstar(b.get_pixel(x, y)))
				dl.append(d)
				## 2D's own-level weight here: 1 means no blend partner (base map
				## in 2D's entry band, parent tile in 3D) contributes.
				var lf := lam + gain * _boost_at(boost_img, inner.position + Vector2(x, y) + Vector2(0.5, 0.5), r2, expect)
				if smoothstep(0.0, 1.0, lf - floor(lf + 0.5) + 0.5) >= 0.999:
					own.append(d)
		dl.sort()
		var mean := 0.0
		for d in dl:
			mean += d
		mean /= maxf(dl.size(), 1)
		var p99 := dl[mini(dl.size() - 1, int(dl.size() * 0.99))]
		_ok("B1 x%d: mean |dL*| <= 1.0" % k, mean <= 1.0, "mean %.3f, p99 %.3f, %d px" % [mean, p99, dl.size()])
		_ok("B1 x%d: p99 |dL*| <= 4.0" % k, p99 <= 4.0, "p99 %.3f" % p99)
		own.sort()
		var om := 0.0
		for d in own:
			om += d
		print("B1 x%d own-level pixels only (2D weight 1, no blend partner): %d px, mean %.3f, p99 %.3f" % [k, own.size(), om / maxf(own.size(), 1), own[mini(own.size() - 1, int(own.size() * 0.99))] if own.size() > 0 else -1.0])
		var d2 := _detail(a)
		var d3 := _detail(b)
		_ok("B6 x%d: 3D detail >= 2D detail at the same px per cell" % k, d3 >= d2 * 0.98, "3D %.3f vs 2D %.3f" % [d3, d2])
		var q = _q()
		_ok("B6 x%d: the deepest drawn level rises with the zoom" % k, q.level_max > prev_deep, "levels %d..%d" % [q.level_min, q.level_max])
		prev_deep = q.level_max
		## The level index per pixel: debug mode 2 writes (j + 0.5) / 16.
		v.set_debug_levels(2)
		await _frames(4)
		var lv := _crop(_crop(await _shot3d(""), Rect2(off, r2.size)), inner)
		v.set_debug_levels(0)
		var agree := _level_agreement(lv, inner, r2, expect, lam, gain)
		## A pixel whose L is within 0.01 of a half-integer is a rounding tie
		## between the GPU's float32 and this f64 reference (on a boost plateau,
		## b = 1, a whole region can sit on one); it is reported, not counted.
		var away := (agree.y - float(_miss.get("tie", 0))) / maxf(agree.y, 1.0)
		var fr_away := (agree.x * agree.y) / maxf(agree.y - float(_miss.get("tie", 0)), 1.0)
		_ok("B1 x%d: level index per pixel equals 2D's on >= 99.9 %% away from rounding ties" % k, fr_away >= 0.999,
			"%.4f away from ties (%.4f raw) of %d px, decode %s; misses %s" % [fr_away, agree.x, int(agree.y), "srgb" if agree.z > 0.5 else "linear", JSON.stringify(_miss)])
		print("B1 x%d level share away from ties %.4f" % [k, away])
	vp.overlay.visible = true

## The boost field at a screen point of the B1 crop (`BoostField::sample`).
func _boost_at(img: Image, sp: Vector2, r2: Rect2, expect: Dictionary) -> float:
	if img == null:
		return 0.0
	var cell: Vector2 = expect["centre"] + (sp - r2.size * 0.5) / float(expect["px_per_cell"])
	var u := Vector2((cell.x - 0.5) / (_grid.x - 1.0), (cell.y - 0.5) / (_grid.y - 1.0))
	var bn := img.get_width()
	var p := u * bn - Vector2(0.5, 0.5)
	var i0 := Vector2i(floor(p.x), floor(p.y))
	var f := p - Vector2(i0)
	var ax := clampi(i0.x, 0, bn - 1)
	var ay := clampi(i0.y, 0, bn - 1)
	var bx := clampi(i0.x + 1, 0, bn - 1)
	var by := clampi(i0.y + 1, 0, bn - 1)
	return clampf(lerpf(lerpf(img.get_pixel(ax, ay).r, img.get_pixel(bx, ay).r, f.x), lerpf(img.get_pixel(ax, by).r, img.get_pixel(bx, by).r, f.x), f.y), 0.0, 1.0)

## The last `_level_agreement()`'s misses: 3D coarser / finer than 2D, and how
## many of them sit within 0.01 level of a rounding boundary (a float tie).
var _miss := {}

## Fraction of pixels whose drawn level equals 2D's `round(L)` there:
## `L = lam + gain * b(x)` (`continuous_lod::effective_level`), b sampled from
## the boost texture as `BoostField::sample`. Returns (fraction, pixels,
## 1 = sRGB decode).
func _level_agreement(lv: Image, inner: Rect2, r2: Rect2, expect: Dictionary, lam: float, gain: float) -> Vector3:
	var boost: Image = null
	var bn := 0
	if gain > 0.0:
		var bt: Texture2D = vp._bridge.lod_ef9_boost_texture()
		if bt != null:
			boost = bt.get_image()
			bn = boost.get_width()
	var ppc := float(expect["px_per_cell"])
	var centre: Vector2 = expect["centre"]
	var zcap: int = _q().max_level()
	var gw := float(_grid.x)
	var gh := float(_grid.y)
	var tables := [[], []]
	for j in 16:
		var lin := (j + 0.5) / 16.0
		tables[0].append(int(round(lin * 255.0)))
		tables[1].append(int(round(Color(lin, lin, lin).linear_to_srgb().r * 255.0)))
	var best := Vector3(0, 0, 0)
	for ti in 2:
		var n := 0
		var eq := 0
		var miss := {"coarser": 0, "finer": 0, "tie": 0}
		for y in range(0, lv.get_height(), 3):
			for x in range(0, lv.get_width(), 3):
				## Screen px -> cell coordinate -> sample -> index space.
				var sp := inner.position + Vector2(x, y) + Vector2(0.5, 0.5)
				var cell := centre + (sp - r2.size * 0.5) / ppc
				var u := Vector2((cell.x - 0.5) / (gw - 1.0), (cell.y - 0.5) / (gh - 1.0))
				if u.x < 0.0 or u.y < 0.0 or u.x > 1.0 or u.y > 1.0:
					continue
				var b := 0.0
				if boost != null:
					var p := u * bn - Vector2(0.5, 0.5)
					var i0 := Vector2i(floor(p.x), floor(p.y))
					var f := p - Vector2(i0)
					var ax := clampi(i0.x, 0, bn - 1)
					var ay := clampi(i0.y, 0, bn - 1)
					var bx := clampi(i0.x + 1, 0, bn - 1)
					var by := clampi(i0.y + 1, 0, bn - 1)
					b = lerpf(lerpf(boost.get_pixel(ax, ay).r, boost.get_pixel(bx, ay).r, f.x), lerpf(boost.get_pixel(ax, by).r, boost.get_pixel(bx, by).r, f.x), f.y)
				var lfull := lam + gain * clampf(b, 0.0, 1.0)
				var want := clampi(int(floor(lfull + 0.5)), 0, zcap)
				var val := lv.get_pixel(x, y).r8
				var got := -1
				var bd := 999
				for j in 16:
					var dd := absi(val - int(tables[ti][j]))
					if dd < bd:
						bd = dd
						got = j
				n += 1
				if got == want:
					eq += 1
				else:
					miss["coarser" if got < want else "finer"] += 1
					if absf(lfull - floor(lfull) - 0.5) < 0.01:
						miss["tie"] += 1
		var fr := float(eq) / maxf(n, 1)
		if fr > best.x:
			best = Vector3(fr, n, ti)
			_miss = miss
	return best

## Counts holes: magenta pixels with terrain above them in the same column
## whose view ray meets sea level INSIDE the world rectangle. A heightfield
## seen from above has no background below its outline, so such a pixel is
## ground the camera sees through. The ray test is there because a view past
## the world's edge shows background below the edge legitimately (seed 24601's
## highest ground is near its edge; the first run counted 17 201 such pixels).
func _magenta_holes(img: Image) -> int:
	var cam: Camera3D = v.camera3d()
	var c: View3DCamera = v.camera_state()
	var pos := c.position()
	var n := 0
	for x in range(0, img.get_width(), 4):
		var seen := false
		for y in range(0, img.get_height(), 2):
			var col := img.get_pixel(x, y)
			var mag := col.r8 > 235 and col.g8 < 20 and col.b8 > 235
			if not mag:
				seen = true
			elif seen:
				var o := cam.project_ray_origin(Vector2(x + 0.5, y + 0.5))
				var d := cam.project_ray_normal(Vector2(x + 0.5, y + 0.5))
				if d.y >= -1e-6:
					continue
				## Camera-relative: sea level is y = -camera height.
				var t := (-float(pos[1]) - o.y) / d.y
				var gx := float(pos[0]) + o.x + d.x * t
				var gz := float(pos[2]) + o.z + d.z * t
				if gx > c.cell_m and gz > c.cell_m and gx < c.world_w_m - 2.0 * c.cell_m and gz < c.world_h_m - 2.0 * c.cell_m:
					n += 1
	return n

func _leg_c() -> void:
	var env: Environment = null
	for ch in v.subviewport().get_children():
		if ch is WorldEnvironment:
			env = (ch as WorldEnvironment).environment
	var bg := env.background_color
	env.background_color = Color(1, 0, 1)
	var peak := _highest_cell()
	var holes := 0
	var frames := 0
	var lv := {}
	for i in 48:
		var t := i / 47.0
		_aim(peak + Vector2(6.0, 6.0), _fit_span / pow(2.0, 1.0 + 6.0 * t), lerpf(60.0, 20.0, t), 360.0 * t)
		await _frames(2)
		var img := await _shot3d("c_path_%02d.png" % i if i % 12 == 0 else "")
		holes += _magenta_holes(img)
		frames += 1
		lv["%d-%d" % [_q().level_min, _q().level_max]] = true
	## The same path again with skirts painted green (debug mode 3): how much
	## skirt shows, i.e. how much gap between nodes the skirts are covering
	## (see `SKIRT_SHARE_MAX` for what it can and cannot see).
	v.set_debug_levels(3)
	var green := 0
	var total := 0
	for i in 48:
		var t := i / 47.0
		_aim(peak + Vector2(6.0, 6.0), _fit_span / pow(2.0, 1.0 + 6.0 * t), lerpf(60.0, 20.0, t), 360.0 * t)
		await _frames(2)
		var img := await _shot3d("c_skirt_%02d.png" % i if i % 24 == 0 else "")
		var d := img.get_data()
		for p in range(0, d.size(), 4 * 3):
			total += 1
			if d[p] < 10 and d[p + 1] > 245 and d[p + 2] < 10:
				green += 1
	v.set_debug_levels(0)
	_skirt_share = float(green) / maxf(total, 1)
	print("SKIRT share of sampled pixels over the path: %.5f (%d of %d)" % [_skirt_share, green, total])
	_ok("B2: skirt shows on at most %.4f of pixels (gaps the skirts cover)" % SKIRT_SHARE_MAX, _skirt_share <= SKIRT_SHARE_MAX, "%.5f" % _skirt_share)
	env.background_color = bg
	_ok("B2: no hole on any frame of a descent through level changes", holes == 0, "%d hole px over %d frames, level spans %s" % [holes, frames, str(lv.keys())])
	_ok("B2: the path crossed level changes", lv.size() >= 3, str(lv.keys()))

func _leg_d() -> void:
	var land := _land_cell()
	_aim(land, _fit_span / 3.0, 45.0)
	await _settle()
	var ts: Array = []
	var changed: Array = []
	var prev := await _shot3d("")
	var sig := [_q().drawn_count, _q().level_min, _q().level_max]
	var c: View3DCamera = v.camera_state()
	for i in 90:
		c.zoom(1.02)
		await _frames(2)
		var img := await _shot3d("")
		var d := 0.0
		var a := prev.get_data()
		var b := img.get_data()
		var n := 0
		for p in range(0, a.size(), 4 * 13):
			d += absi(a[p] - b[p]) + absi(a[p + 1] - b[p + 1]) + absi(a[p + 2] - b[p + 2])
			n += 3
		ts.append(d / maxf(n, 1))
		var s2 := [_q().drawn_count, _q().level_min, _q().level_max]
		changed.append(s2 != sig)
		sig = s2
		prev = img
	var pops := 0
	var events := 0
	for i in ts.size():
		if not changed[i]:
			continue
		events += 1
		var nb: Array = []
		for j in range(maxi(0, i - 5), mini(ts.size(), i + 6)):
			if j != i:
				nb.append(ts[j])
		nb.sort()
		var med: float = nb[nb.size() / 2]
		if float(ts[i]) > 1.5 * med:
			pops += 1
			print("POP frame %d T=%.3f neighbours median %.3f" % [i, ts[i], med])
	_ok("B3: no pop at a selection change (T_i <= 1.5 x neighbours, unreprojected)", pops == 0, "%d of %d change frames" % [pops, events])
	_ok("B3: the dolly changed the selection", events >= 3, "%d change frames" % events)

## An independent sample of the drawn surface under (x_m, z_m): bilinear in
## the deepest drawn node's vertex grid (not `surface_bound()`'s maximum).
func _point_surface(x_m: float, z_m: float) -> float:
	var q = _q()
	var cm: float = q._cell_m
	var sx: float = x_m / ((_grid.x - 1) * cm)
	var sz: float = z_m / ((_grid.y - 1) * cm)
	var best := -INF
	for key in q.drawn:
		var n := float(1 << key.x)
		var u: float = sx * n - key.y
		var w: float = sz * n - key.z
		if u < 0.0 or w < 0.0 or u > 1.0 or w > 1.0:
			continue
		var g: PackedFloat32Array = q._tiles[key]["geo"]
		var nn: int = q._n
		var fu := clampf(u * (nn - 1), 0.0, nn - 1.001)
		var fv := clampf(w * (nn - 1), 0.0, nn - 1.001)
		var i0 := int(fu)
		var j0 := int(fv)
		var a := lerpf(g[j0 * nn + i0], g[j0 * nn + i0 + 1], fu - i0)
		var b := lerpf(g[(j0 + 1) * nn + i0], g[(j0 + 1) * nn + i0 + 1], fu - i0)
		best = maxf(best, q.metres(lerpf(a, b, fv - j0)))
	return best

func _leg_e() -> void:
	var peak := _highest_cell()
	v.set_height(4.0)
	var below := 0
	var minm := INF
	var c: View3DCamera = v.camera_state()
	for i in 120:
		_aim(peak + Vector2(2.0, 2.0), 3000.0, 8.0, i * 3.0)
		await get_tree().process_frame
		await get_tree().process_frame
		var p := c.position()
		var s := _point_surface(p[0], p[2])
		if s > -INF:
			minm = minf(minm, float(p[1]) - s)
			if float(p[1]) < s:
				below += 1
	_ok("B4: no frame below the drawn surface (quadtree, 4x, 8° pitch)", below == 0, "%d below, min margin %.1f m" % [below, minm])
	await _shot3d("e_low_orbit_4x.png")
	v.set_height(1.0)

func _leg_g() -> void:
	var c: View3DCamera = v.camera_state()
	_aim(_land_cell(), _fit_span / 4.0, 50.0)
	await _frames(4)
	var centre := v.get_global_rect().get_center() * get_viewport().get_final_transform().x.x
	if not _touch:
		var d0 := c.dist
		_mouse(centre, MOUSE_BUTTON_WHEEL_UP, true)
		_mouse(centre, MOUSE_BUTTON_WHEEL_UP, false)
		await _frames(2)
		_ok("G: the wheel zooms the 3D camera in", c.dist < d0 * 0.99, "%.0f -> %.0f m" % [d0, c.dist])
		var y0 := c.yaw
		_mouse(centre, MOUSE_BUTTON_RIGHT, true)
		for i in 10:
			_motion(centre + Vector2(10 * (i + 1), 0), Vector2(10, 0), MOUSE_BUTTON_MASK_RIGHT)
			await get_tree().process_frame
		_mouse(centre + Vector2(100, 0), MOUSE_BUTTON_RIGHT, false)
		await _frames(2)
		_ok("G: a right drag orbits", absf(wrapf(c.yaw - y0, -PI, PI)) > deg_to_rad(5.0), "yaw %.1f° -> %.1f°" % [rad_to_deg(y0), rad_to_deg(c.yaw)])
	else:
		var px0 := Vector2(c.pivot_x, c.pivot_z)
		_touch_ev(0, centre, true)
		for i in 10:
			_drag_ev(0, centre + Vector2(0, 12 * (i + 1)), Vector2(0, 12))
			await get_tree().process_frame
		_touch_ev(0, centre + Vector2(0, 120), false)
		await _frames(2)
		_ok("G: one finger pans", Vector2(c.pivot_x, c.pivot_z).distance_to(px0) > 1.0, "pivot moved %.0f m" % Vector2(c.pivot_x, c.pivot_z).distance_to(px0))
		var d0 := c.dist
		var a := centre - Vector2(60, 0)
		var b := centre + Vector2(60, 0)
		_touch_ev(0, a, true)
		_touch_ev(1, b, true)
		for i in 10:
			_drag_ev(0, a - Vector2(8 * (i + 1), 0), Vector2(-8, 0))
			_drag_ev(1, b + Vector2(8 * (i + 1), 0), Vector2(8, 0))
			await get_tree().process_frame
		_touch_ev(0, a - Vector2(80, 0), false)
		_touch_ev(1, b + Vector2(80, 0), false)
		await _frames(2)
		_ok("G: two fingers pinch the 3D camera in", c.dist < d0 * 0.9, "%.0f -> %.0f m" % [d0, c.dist])
	await _settle()
	_ok("G: the quadtree still draws after the gestures", _q().drawn_count > 0)

func _leg_f() -> void:
	DisplayServer.window_set_vsync_mode(DisplayServer.VSYNC_DISABLED)
	OS.low_processor_usage_mode = false
	var land := _land_cell()
	## 2D's steady state at the view the 3D one starts from (2D at x16).
	vp.set_3d(false)
	await _set_2d(land, 16.0)
	var mem2d := Performance.get_monitor(Performance.RENDER_VIDEO_MEM_USED)
	var expect := vp.view_for_3d()
	vp.set_3d(true)
	await _await_placed()
	var c: View3DCamera = v.camera_state()
	c.enter_from_2d(expect["centre"], float(expect["px_per_cell"]), float(v.subviewport().size.y), false)
	c.set_ortho(false)
	c.pitch = deg_to_rad(45.0)
	await _settle()
	for i in 60:
		await get_tree().process_frame
	var mem3d := Performance.get_monitor(Performance.RENDER_VIDEO_MEM_USED)
	var q = _q()
	var bar := 33.0 if View3DGate.phone_class(OS.get_name()) else 16.7
	var st := await _time(300, func(_i): pass)
	var prims := Performance.get_monitor(Performance.RENDER_TOTAL_PRIMITIVES_IN_FRAME)
	var draws := Performance.get_monitor(Performance.RENDER_TOTAL_DRAW_CALLS_IN_FRAME)
	print("FRAME static median=%.3f min=%.3f max=%.3f p95=%.3f ms primitives=%d draws=%d nodes=%d levels=%d..%d resident=%d select_ms=%.2f" % [
		st["median"], st["min"], st["max"], st["p95"], prims, draws, q.drawn_count, q.level_min, q.level_max, q.resident(), q.last_select_ms])
	_ok("F: %s frame bar, static: p95 <= %.1f ms" % [View3DGate.TIER_NAMES[_tier], bar], float(st["p95"]) <= bar, "p95 %.3f ms" % float(st["p95"]))
	var mv := await _time(300, func(_i): c.orbit(deg_to_rad(0.5), 0.0))
	print("FRAME orbiting median=%.3f min=%.3f max=%.3f p95=%.3f ms" % [mv["median"], mv["min"], mv["max"], mv["p95"]])
	_ok("F: frame bar, orbiting: p95 <= %.1f ms" % bar, float(mv["p95"]) <= bar, "p95 %.3f ms" % float(mv["p95"]))
	var up0: int = q.tiles_uploaded
	var zf := await _time(300, func(i): c.zoom(1.015 if i < 150 else 1.0 / 1.015))
	print("FRAME zooming (level changes) median=%.3f min=%.3f max=%.3f p95=%.3f ms tiles_uploaded=%d" % [zf["median"], zf["min"], zf["max"], zf["p95"], q.tiles_uploaded - up0])
	_ok("F: frame bar, zooming through levels: p95 <= %.1f ms" % bar, float(zf["p95"]) <= bar, "p95 %.3f ms, %d tiles landed" % [float(zf["p95"]), q.tiles_uploaded - up0])
	var mb := (mem3d - mem2d) / 1e6
	var cap: float = float(TIER_MB[q.tier_key()]) * 1.1
	print("MEMORY 2D steady %.1f MB, 3D steady %.1f MB, 3D above 2D %.1f MB (bar %.1f MB), layers %d" % [mem2d / 1e6, mem3d / 1e6, mb, cap, q.layer_cap()])
	_ok("F: 3D video memory above 2D's steady state <= the tier's §3.12 figure + 10 %", mb <= cap, "%.1f MB vs %.1f MB" % [mb, cap])
	var peak := _highest_cell() + Vector2(4.0, 4.0)
	for k in [4, 16, 64]:
		_aim(peak, _fit_span / k, 40.0, 30.0)
		await _settle()
		await _shot3d("f_oblique_x%d.png" % k)
	v.set_debug_levels(1)
	await _frames(4)
	await _shot3d("f_oblique_levels.png")
	v.set_debug_levels(0)

func _leg_h() -> void:
	## The same deep-zoom 2D view before and after a visit to 3D: 3D parks
	## 2D's tiles (§3.12), so leaving must rebuild them to the same picture.
	## A first cut compared against the session's opening view after a
	## `_reset_view_2d()`, which on the touch composition is not the opening
	## view (453 577 px differed for that reason alone, 2026-11-13).
	vp.set_3d(false)
	await _set_2d(_highest_cell() + Vector2(4.0, 4.0), 16.0)
	await _park_pointer()
	var before := _crop(await _shot("h_2d_before.png"), _rect2d())
	var tiles_before: int = vp._lod_tiles.size()
	vp.set_3d(true)
	await _settle()
	_ok("H: 2D's tiles are freed while 3D shows", vp._lod_tiles.is_empty(), "%d live before, %d in 3D" % [tiles_before, vp._lod_tiles.size()])
	vp.set_3d(false)
	await _settle_2d()
	await _park_pointer()
	var after := _crop(await _shot("h_2d_after.png"), _rect2d())
	var n := 0
	var a := before.get_data()
	var b := after.get_data()
	if a.size() != b.size():
		n = -1
	else:
		for i in range(0, a.size(), 4):
			if a[i] != b[i] or a[i + 1] != b[i + 1] or a[i + 2] != b[i + 2]:
				n += 1
	_ok("H: leaving 3D rebuilds 2D's deep-zoom frame byte-identical", n == 0 and tiles_before > 0, "%d px differ; %d tiles before, %d after" % [n, tiles_before, vp._lod_tiles.size()])
