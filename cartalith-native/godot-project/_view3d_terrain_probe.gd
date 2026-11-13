extends Node
## **3D-2 acceptance probe** (`3D_VIEWPORT_SCOPE.md` §5, "3D-2 · Basic: one
## displaced mesh, map-shading drape, orbit, camera-relative from the start").
## What it protects, one leg per line of the milestone and its bars:
##
## - **A. Entry.** `Window ▸ 3D view` enters 3D top-down at the 2D view's own
##   centre and scale, then tilts to Orbit (§3.9, Q9's answer); the gate ran;
##   terrain is drawn (most of the 3D frame is not background) at true scale
##   (Ruling CT) unless a preference says otherwise.
## - **B. Zoom gains detail at every level** (the owner's hard rule): at camera
##   heights equivalent to 2D's x4, x16 and x64 the window's level rises
##   strictly, and at each height the drawn level carries more near-field
##   detail (mean luminance gradient) than the same camera drawn two levels
##   coarser (the terrain's probe seam). Frames are saved for each.
## - **C. Displacement is real:** the same oblique view at 1x and 4x
##   exaggeration moves pixels (the VTF control, on the shipped tiles).
## - **D. Bar B4, the ground clamp:** a scripted fly path that orbits low past
##   the highest ground. Every frame the camera is above an independent point
##   sample of the drawn relief and no background shows in the lower 40 % of
##   the frame. **Positive control:** with the clamp off (`debug_no_clamp`) the
##   same path does go below the ground.
## - **E. Bar B5, depth:** two frames of a static camera, at the highest
##   altitude (world framed) and the lowest (closest, low pitch over a coast),
##   differ in 0 pixels.
## - **F. Basic's frame bar:** a static Basic view with vsync off, 60 warm-up
##   and 300 timed frames, median, min..max and p95; p95 <= 33 ms. A moving
##   camera and a level change are timed and printed (not barred).
## - **G. Controls:** desktop: the wheel zooms by `ZOOM_WHEEL_STEP`, a right
##   drag orbits, a left drag pans in the drag's direction, Top-down switches
##   to an orthographic camera; touch (`--force-touch`): the navpad's −, + and ⌖
##   tapped by synthesised touches at their drawn centres zoom the 3D camera by
##   `ZOOM_BUTTON_STEP` and frame the world, while the hidden 2D zoom is
##   untouched (Ruling EB item 4); one finger pans; two fingers pinch, twist and
##   tilt. Injected through `Input.parse_input_event`, upstream of every
##   control's filter; still not a finger on glass (`MISTAKES.md` row 155).
## - **H. Leaving 3D** gives back the map's 2D frame byte-identical to the one
##   before entering.
##
## MUST run windowed; refuses headless with exit 2. Declares and clears the
## `view3d` settings it asserts on.
##
##   godot --path . --resolution 1600x900 _view3d_terrain_probe.tscn -- --out DIR
##   godot --path . --resolution 1080x2340 _view3d_terrain_probe.tscn -- --force-touch --out DIR
##
## Flags (an unknown one aborts with exit 2): `--out DIR` (required),
## `--force-touch` (read by `DccShell`; switches leg G to touch), `--grid WxH`
## (default 512x384), `--seed N` (default 483920), `--timing-only` (legs A and F
## only, for the separate-process timing runs).
## Result line: `RESULT checks=N fail=M`. Exit 0 all held, 1 a check failed,
## 2 could not run.

const ARGS: Array[String] = ["--out", "--force-touch", "--grid", "--seed", "--timing-only"]
const View3DGate := preload("res://shell/view3d_gate.gd")
const View3D := preload("res://shell/view3d.gd")
const View3DCamera := preload("res://shell/view3d_camera.gd")

var app: Node
var vp: ViewportHost
var v: View3D
var _out := ""
var _checks := 0
var _fail := 0
var _touch := false
var _grid := Vector2i(512, 384)
var _seed := 483920
var _timing_only := false

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
			"--timing-only":
				_timing_only = true
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
	## Basic forced: since 3D-3, Standard and Full draw the quadtree
	## (`_view3d_quadtree_probe.gd`), and this probe measures Basic's patch.
	DccSettings.set_view3d_forced_tier(View3DGate.TIER_BASIC)
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
	var ok := await _leg_a()
	if ok:
		if _timing_only:
			await _leg_f()
		else:
			await _leg_b()
			await _leg_c()
			await _leg_d()
			await _leg_e()
			await _leg_f()
			await _leg_g()
			await _leg_i()
			await _leg_h()
	DccSettings.set_view3d_result({})
	DccSettings.set_view3d_forced_tier(-1)
	DccSettings.clear_view3d_exaggeration()
	print("RESULT checks=%d fail=%d" % [_checks, _fail])
	get_tree().quit(1 if _fail > 0 else 0)

# -- helpers --------------------------------------------------------------------

func _frames(n: int) -> void:
	for i in n:
		await RenderingServer.frame_post_draw

func _shot(name: String) -> Image:
	await _frames(3)
	var img := get_viewport().get_texture().get_image()
	if name != "":
		img.save_png(_out.path_join(name))
	return img

func _window_popup() -> PopupMenu:
	for ch in app.menu_bar_row.get_children():
		if ch is MenuButton:
			var found := _find_id((ch as MenuButton).get_popup(), DccMenus.ID_WIN_VIEW_3D)
			if found != null:
				return found
	return null

func _find_id(p: PopupMenu, id: int) -> PopupMenu:
	if p.get_item_index(id) >= 0:
		return p
	for c in p.get_children(true):
		if c is PopupMenu:
			var f := _find_id(c as PopupMenu, id)
			if f != null:
				return f
	return null

## The part of the 3D view no shell chrome covers, in window pixels: the view's
## rect inside the host's safe insets (on the phone the map runs edge to edge
## behind the app bar and the bottom sheet) and above the touch navpad. A first
## cut measured the whole view and read the phone's sheet as background.
func _rect3d() -> Rect2:
	var r := v.get_global_rect()
	var ins: Dictionary = vp._safe_insets
	r = r.grow_individual(-float(ins.get("left", 0.0)), -float(ins.get("top", 0.0)), -float(ins.get("right", 0.0)), -float(ins.get("bottom", 0.0)))
	var pad = vp._navpad
	if pad != null and pad.is_visible_in_tree():
		r.end.y = minf(r.end.y, pad.get_global_rect().position.y)
	return r * get_viewport().get_final_transform()

## The 2D map's own rect inside the host's safe insets, in window pixels: on
## the phone the host spans the whole screen, and its status bar's clock ticked
## between legs A and H (34 px, the first phone run of 2026-11-12).
func _rect2d() -> Rect2:
	var ins: Dictionary = vp._safe_insets
	return vp.get_global_rect().grow_individual(-float(ins.get("left", 0.0)), -float(ins.get("top", 0.0)),
		-float(ins.get("right", 0.0)), -float(ins.get("bottom", 0.0))) * get_viewport().get_final_transform()

## **Holes in the ground**: background pixels with drawn terrain above them in
## the same column. A heightfield seen from above it has no sky below its
## upper outline, so any such pixel is ground the camera sees through (a crack,
## a discarded backdrop, a camera inside a hill with back faces culled). Read
## below the card, every third column. Returns the count.
func _holes(img: Image) -> int:
	var card_bottom := 0
	if v._card != null and v._card.is_visible_in_tree():
		var cr: Rect2 = v._card.get_global_rect() * get_viewport().get_final_transform()
		card_bottom = int(cr.end.y - _rect3d().position.y) + 1
	var bg := SKY
	var n := 0
	for x in range(0, img.get_width(), 3):
		var seen := false
		for y in range(clampi(card_bottom, 0, img.get_height()), img.get_height()):
			var c := img.get_pixel(x, y)
			var is_bg := absi(c.r8 - bg.r8) <= 6 and absi(c.g8 - bg.g8) <= 6 and absi(c.b8 - bg.b8) <= 6
			if not is_bg:
				seen = true
			elif seen:
				n += 1
	return n

## Waits until no first-run coach mark is on screen (the phone shows three in
## turn, 3.6 s each), so the 2D captures of legs A and H are of the map, not of
## a hint that happened to be up for one of them.
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

func _crop(img: Image, r: Rect2) -> Image:
	var ri := Rect2i(r).intersection(Rect2i(Vector2i.ZERO, img.get_size()))
	return img.get_region(ri)

## The 3D view's background while this probe runs: magenta, a colour no
## terrain look draws. The probe first compared against the theme's own
## background, and on 2026-11-13 (3D-3's re-run) bright snow texels within 6
## levels of the light theme's (244, 242, 238) read as 1 273 "holes" at x16,
## 97 % of them isolated pixels inside the snow (`MISTAKES.md`: a pixel
## threshold is palette-bound).
const SKY := Color(1.0, 0.0, 1.0)

func _set_sky() -> void:
	for ch in v.subviewport().get_children():
		if ch is WorldEnvironment:
			(ch as WorldEnvironment).environment.background_color = SKY

## Pixels within 6 levels of the 3D background colour, as a fraction of `img`.
func _bg_fraction(img: Image) -> float:
	var bg := SKY
	var d := img.get_data()
	var n := 0
	var tot := 0
	for i in range(0, d.size(), 4 * 7):
		tot += 1
		if absi(d[i] - int(bg.r8)) <= 6 and absi(d[i + 1] - int(bg.g8)) <= 6 and absi(d[i + 2] - int(bg.b8)) <= 6:
			n += 1
	return float(n) / maxf(tot, 1)

## Mean absolute luminance step between horizontal and vertical neighbours, in
## 0..255 levels: LOD-D0's detail-per-pixel metric, over `r` of `img`.
func _detail(img: Image, r: Rect2) -> float:
	var c := _crop(img, r)
	var w := c.get_width()
	var h := c.get_height()
	var s := 0.0
	var n := 0
	for y in range(0, h - 1):
		for x in range(0, w - 1):
			var l := c.get_pixel(x, y).get_luminance()
			s += absf(l - c.get_pixel(x + 1, y).get_luminance()) + absf(l - c.get_pixel(x, y + 1).get_luminance())
			n += 2
	return s / maxf(n, 1) * 255.0

func _diff_px(a: Image, b: Image) -> int:
	if a.get_size() != b.get_size():
		return -1
	var n := 0
	var da := a.get_data()
	var db := b.get_data()
	for i in range(0, da.size(), 4):
		if da[i] != db[i] or da[i + 1] != db[i + 1] or da[i + 2] != db[i + 2]:
			n += 1
	return n

## Waits until the terrain has a drawn window and nothing pending (or times out).
func _settle(limit_s := 60.0) -> bool:
	var t0 := Time.get_ticks_msec()
	await _frames(3)
	while (v.terrain().filling() or v.terrain().window().is_empty()) and (Time.get_ticks_msec() - t0) < limit_s * 1000.0:
		await get_tree().process_frame
	await _frames(3)
	return not v.terrain().filling() and not v.terrain().window().is_empty()

## An independent sample of the drawn relief at `(x_m, z_m)`: bilinear over the
## window's relief image where the window covers the point, else the
## backdrop's; metres by the terrain's law. Not `surface_bound()` (the code under
## test): a point sample of the relief, the surface the patch approximates.
func _point_surface(x_m: float, z_m: float) -> float:
	var t = v.terrain()
	var cm: float = t.cell_m()
	var sx := x_m / cm
	var sy := z_m / cm
	var f: Dictionary = t._back
	var w: Dictionary = t._win
	if not w.is_empty() and sx >= float(w["x0"]) and sx <= float(w["x0"]) + float(w["w"]) and sy >= float(w["y0"]) and sy <= float(w["y0"]) + float(w["h"]):
		f = w
	var img: Image = f["himg"]
	var u := clampf((sx - float(f["x0"])) / float(f["w"]) * (img.get_width() - 1), 0.0, img.get_width() - 1.001)
	var q := clampf((sy - float(f["y0"])) / float(f["h"]) * (img.get_height() - 1), 0.0, img.get_height() - 1.001)
	var x0 := int(u)
	var y0 := int(q)
	var fx := u - x0
	var fy := q - y0
	var h := lerpf(lerpf(img.get_pixel(x0, y0).r, img.get_pixel(x0 + 1, y0).r, fx),
		lerpf(img.get_pixel(x0, y0 + 1).r, img.get_pixel(x0 + 1, y0 + 1).r, fx), fy)
	## The backdrop is drawn sunk; ignoring the sink overstates its surface,
	## which only makes this check stricter.
	return t.metres(h)

## The highest texel of the backdrop relief, as a sample position (cells).
func _highest_cell() -> Vector2:
	var b: Dictionary = v.terrain()._back
	var img: Image = b["himg"]
	var best := -1.0
	var at := Vector2i.ZERO
	for y in img.get_height():
		for x in img.get_width():
			var h := img.get_pixel(x, y).r
			if h > best:
				best = h
				at = Vector2i(x, y)
	return Vector2(float(b["x0"]) + at.x / float(img.get_width() - 1) * float(b["w"]),
		float(b["y0"]) + at.y / float(img.get_height() - 1) * float(b["h"]))

## The deepest backdrop texel (sea, flattened in 3D), as a sample position.
func _deep_cell() -> Vector2:
	var b: Dictionary = v.terrain()._back
	var img: Image = b["himg"]
	var lo := INF
	var at := Vector2i.ZERO
	## The relief image carries sea at exactly sea level, so the deepest is any
	## sea texel; take the one whose 9 x 9 neighbourhood is all sea, nearest
	## the centre, so no land rises beside the camera.
	var sea: float = v.terrain()._sea
	for y in range(4, img.get_height() - 4, 2):
		for x in range(4, img.get_width() - 4, 2):
			var all_sea := true
			for j in range(-4, 5, 2):
				for i in range(-4, 5, 2):
					if img.get_pixel(x + i, y + j).r > sea:
						all_sea = false
			if all_sea:
				var d := Vector2(x - img.get_width() / 2, y - img.get_height() / 2).length()
				if d < lo:
					lo = d
					at = Vector2i(x, y)
	return Vector2(float(b["x0"]) + at.x / float(img.get_width() - 1) * float(b["w"]),
		float(b["y0"]) + at.y / float(img.get_height() - 1) * float(b["h"]))

## A coastal cell: a backdrop texel at sea level beside one above it, nearest
## the world's centre.
func _coast_cell() -> Vector2:
	var b: Dictionary = v.terrain()._back
	var img: Image = b["himg"]
	var sea: float = v.terrain()._sea
	var best := INF
	var at := Vector2i(img.get_width() / 2, img.get_height() / 2)
	for y in range(1, img.get_height() - 1):
		for x in range(1, img.get_width() - 1):
			if img.get_pixel(x, y).r <= sea and img.get_pixel(x + 1, y).r > sea + 0.01:
				var d := Vector2(x - img.get_width() / 2, y - img.get_height() / 2).length()
				if d < best:
					best = d
					at = Vector2i(x, y)
	return Vector2(float(b["x0"]) + at.x / float(img.get_width() - 1) * float(b["w"]),
		float(b["y0"]) + at.y / float(img.get_height() - 1) * float(b["h"]))

## Puts the orbit camera at a pivot (cells) with a vertical ground span (m).
func _aim(cell: Vector2, span_m: float, pitch_deg: float, yaw_deg := 0.0) -> void:
	var c: View3DCamera = v.camera_state()
	c.set_ortho(false)
	c.pivot_x = cell.x * c.cell_m
	c.pivot_z = cell.y * c.cell_m
	c.dist = clampf(span_m * 0.5 / tan(deg_to_rad(View3DCamera.FOV_DEG) * 0.5), c.min_dist(), c.max_dist())
	c.pitch = deg_to_rad(pitch_deg)
	c.yaw = deg_to_rad(yaw_deg)
	c.tilt_left = 0.0

# -- legs -----------------------------------------------------------------------

var _before2d: Image
var _fit_span := 0.0
## `RENDER_VIDEO_MEM_USED` with the 2D map up, before 3D was ever opened.
var _mem_2d := 0.0

## Parks the pointer on the status bar (bottom-left corner), so the 2D map's
## hover readout and highlight are the same in the two frames leg H compares.
## The first desktop run with leg I differed in 10 702 px, all of them the hover
## readout of wherever the last synthesised (or the real) pointer rested.
func _park_pointer() -> void:
	if not _touch:
		var at := Vector2(4.0, get_viewport().get_visible_rect().size.y - 4.0) * get_viewport().get_final_transform().x.x
		_motion(at, Vector2.ZERO, 0)
		await _frames(4)

func _leg_a() -> bool:
	await _wait_no_coach()
	await _park_pointer()
	_before2d = _crop(await _shot("00_2d_before.png"), _rect2d())
	var expect := vp.view_for_3d()
	_mem_2d = Performance.get_monitor(Performance.RENDER_VIDEO_MEM_USED)
	var win := _window_popup()
	var idx := win.get_item_index(DccMenus.ID_WIN_VIEW_3D) if win != null else -1
	_ok("A: Window ▸ 3D view row exists", idx >= 0)
	if idx < 0:
		return false
	win.id_pressed.emit(DccMenus.ID_WIN_VIEW_3D)
	v = vp.view3d()
	_set_sky()
	if v == null:
		_ok("A: the 3D view was built", false)
		return false
	var t0 := Time.get_ticks_msec()
	while v.busy() and Time.get_ticks_msec() - t0 < 120000:
		await get_tree().process_frame
	var r := DccSettings.view3d_result()
	_ok("A: the gate ran and measured at least Basic", int(r.get("tier", 0)) >= View3DGate.TIER_BASIC,
		"tier %s, cause '%s'" % [View3DGate.TIER_NAMES[int(r.get("tier", 0))], r.get("cause", "")])
	## The first frame with a world: the entry pose, before the tilt runs.
	t0 = Time.get_ticks_msec()
	while not v.terrain().world_known() and Time.get_ticks_msec() - t0 < 30000:
		await get_tree().process_frame
	await get_tree().process_frame
	var c: View3DCamera = v.camera_state()
	var cm := c.cell_m
	_ok("A: opens at the 2D view's centre", not expect.is_empty()
		and absf(c.pivot_x - (float(expect["centre"].x) - 0.5) * cm) < 1.0 and absf(c.pivot_z - (float(expect["centre"].y) - 0.5) * cm) < 1.0,
		"pivot %.1f,%.1f m vs 2D centre %s cells x %.3f m" % [c.pivot_x, c.pivot_z, str(expect.get("centre")), cm])
	var span_2d := float(v.subviewport().size.y) / float(expect.get("px_per_cell", 1.0)) * cm
	_ok("A: opens at the 2D view's ground scale", absf(c.view_span_m() - clampf(span_2d, c.min_ortho(), c.max_ortho())) < 0.01 * span_2d,
		"3D span %.0f m vs 2D %.0f m" % [c.view_span_m(), span_2d])
	_ok("A: opens straight down, then tilts", c.pitch > deg_to_rad(80.0) and c.tilt_left > 0.0, "pitch %.1f°" % rad_to_deg(c.pitch))
	## Literals, not the constants: §3.9's tilt is about 0.6 s and ends at 55°.
	await get_tree().create_timer(0.9).timeout
	_ok("A: the tilt ends at 55° within 0.9 s", absf(rad_to_deg(c.pitch) - 55.0) < 0.01, "pitch %.2f°" % rad_to_deg(c.pitch))
	var settled := await _settle()
	_ok("A: a tile window is drawn", settled, JSON.stringify(_win_summary()))
	_ok("A: true scale by default (Ruling CT)", is_equal_approx(v.terrain().exaggeration(), 1.0), "%.2f" % v.terrain().exaggeration())
	var img := _crop(await _shot("01_entry_orbit.png"), _rect3d())
	var bgf := _bg_fraction(img)
	_ok("A: terrain covers the 3D view", bgf < 0.35, "background %.1f %% of the 3D frame" % (bgf * 100.0))
	_ok("A: the card names the tile level", v._info.text.find("tile level") >= 0, v._info.text)
	_fit_span = maxf(c.world_h_m, c.world_w_m / (float(v.subviewport().size.x) / v.subviewport().size.y))
	return settled

func _win_summary() -> Dictionary:
	var w: Dictionary = v.terrain().window()
	if w.is_empty():
		return {"reason": v.terrain().reason}
	return {"z": w["z"], "c0": w["c0"], "r0": w["r0"], "k": w["k"], "mosaic": "%dx%d" % [w["mx"], w["my"]],
		"tiles_built": v.terrain().tiles_built, "last_fill_ms": snappedf(v.terrain().last_fill_ms, 0.1)}

## The near field: the lower-middle of the 3D frame, clear of the card.
func _near(img: Image) -> Rect2:
	var s := Vector2(img.get_size())
	return Rect2(s.x * 0.25, s.y * 0.55, s.x * 0.5, s.y * 0.35)

func _leg_b() -> void:
	var peak := _highest_cell()
	var zs: Array = []
	for k in [4, 16, 64]:
		_aim(peak + Vector2(3.0, 3.0), _fit_span / k, 50.0)
		await _settle()
		var z := int(v.terrain().window().get("z", -1))
		zs.append(z)
		var c: View3DCamera = v.camera_state()
		var ppc := c.cell_m / c.metres_per_px(float(v.subviewport().size.y))
		var z2d: int = app.bridge.lod_level_for_zoom(ppc)
		var n_tiles := 1 << z
		_ok("B: x%d: the window is one level below 2D's, 4 tiles a side" % k, z == maxi(z2d - 1, 0) and int(v.terrain().window()["k"]) == mini(4, n_tiles),
			"z %d, 2D z %d, k %d" % [z, z2d, int(v.terrain().window()["k"])])
		var img := _crop(await _shot("02_x%d_drawn_z%d.png" % [k, z]), _rect3d())
		var dd := _detail(img, _near(img))
		var holes := _holes(img)
		_ok("B: x%d: no hole in the drawn ground (no crack at the window's rim)" % k, holes == 0, "%d px" % holes)
		v.terrain().debug_level_offset = -2
		await _settle()
		var zc := int(v.terrain().window().get("z", -1))
		var imc := _crop(await _shot("02_x%d_coarse_z%d.png" % [k, zc]), _rect3d())
		var dc := _detail(imc, _near(imc))
		v.terrain().debug_level_offset = 0
		await _settle()
		print("DETAIL x%d z=%d %.3f vs z=%d %.3f (%.2fx) window %s" % [k, z, dd, zc, dc, dd / maxf(dc, 1e-6), JSON.stringify(_win_summary())])
		_ok("B: x%d: the drawn level carries more detail than two levels coarser" % k, zc < z and dd > dc,
			"z%d %.3f vs z%d %.3f" % [z, dd, zc, dc])
	_ok("B: the level rises at every zoom step", zs.size() == 3 and zs[0] < zs[1] and zs[1] < zs[2], str(zs))
	## Two more heights for the frames the scope asks to be looked at: the
	## world framed, and the closest view.
	vp.reset_view()
	await _settle()
	await _shot("02_world_framed.png")
	_aim(peak + Vector2(3.0, 3.0), 0.0, 35.0, 30.0)
	await _settle()
	await _shot("02_closest_z%d.png" % int(v.terrain().window().get("z", -1)))

func _leg_c() -> void:
	var peak := _highest_cell()
	_aim(peak + Vector2(4.0, 4.0), _fit_span / 16.0, 35.0)
	await _settle()
	var a := _crop(await _shot("03_exag_1.png"), _rect3d())
	v.terrain().set_exaggeration(4.0)
	await _frames(4)
	var b := _crop(await _shot("03_exag_4.png"), _rect3d())
	v.terrain().set_exaggeration(1.0)
	var n := _diff_px(a, b)
	_ok("C: 1x against 4x moves pixels (heights reach the vertices)", n > a.get_width() * a.get_height() / 100, "%d px moved" % n)

## Bar B4's fly path. Returns {below, sky_frames, clamped, frames, min_margin}.
func _fly(no_clamp: bool, tag: String) -> Dictionary:
	var c: View3DCamera = v.camera_state()
	c.debug_no_clamp = no_clamp
	var peak := _highest_cell()
	## At 4x (the slider's ceiling, Ruling EB item 6) so the high ground is as
	## tall as the view can make it, with the pivot one orbit radius east of
	## the summit at the closest distance and 8° pitch: the orbit then carries
	## the camera straight over the summit, low.
	v.terrain().set_exaggeration(4.0)
	_aim(peak, 0.0, 8.0, 0.0)
	var radius := c.dist * cos(deg_to_rad(8.0)) / c.cell_m
	_aim(peak + Vector2(radius, 0.0), 0.0, 8.0, 0.0)
	await _settle()
	var below := 0
	var sky := 0
	var clamped := 0
	var min_margin := INF
	## The clamp's own floor on the frames it acted: §3.9's "2 cells × 0.1"
	## (or more, for the near plane) above the drawn surface's bound.
	var min_clamped_hag := INF
	var frames := 0
	for i in 180:
		c.orbit(deg_to_rad(2.0), 0.0)
		c.pitch = deg_to_rad(PITCH_PATH(i))
		await RenderingServer.frame_post_draw
		frames += 1
		var p := c.position()
		var margin := float(p[1]) - _point_surface(p[0], p[2])
		min_margin = minf(min_margin, margin)
		if margin < 0.0:
			below += 1
		if v.last_clamp_steps > 0:
			clamped += 1
			min_clamped_hag = minf(min_clamped_hag, v.last_hag)
		if i % 6 == 0:
			var img := _crop(get_viewport().get_texture().get_image(), _rect3d())
			if _holes(img) > 0:
				sky += 1
				if sky == 1:
					img.save_png(_out.path_join("04_%s_first_hole_frame.png" % tag))
	c.debug_no_clamp = false
	v.terrain().set_exaggeration(1.0)
	return {"below": below, "sky_frames": sky, "clamped": clamped, "frames": frames, "min_margin": min_margin, "min_clamped_hag": min_clamped_hag}

## The fly path's pitch: from 12° down to the 5° floor and back up.
func PITCH_PATH(i: int) -> float:
	return 5.0 + 7.0 * absf(cos(i * PI / 90.0))

func _leg_d() -> void:
	var off := await _fly(true, "noclamp")
	print("B4 control (clamp off): ", JSON.stringify(off))
	_ok("D: control: without the clamp the path goes below the ground", int(off["below"]) > 0, JSON.stringify(off))
	var on := await _fly(false, "clamp")
	print("B4 (clamp on): ", JSON.stringify(on))
	_ok("D: B4: no frame below the drawn ground", int(on["below"]) == 0, "min margin %.1f m over %d frames" % [float(on["min_margin"]), int(on["frames"])])
	_ok("D: B4: no frame sees through the ground", int(on["sky_frames"]) == 0, JSON.stringify(on))
	_ok("D: the clamp acted on the path", int(on["clamped"]) > 0, "%d frames clamped" % int(on["clamped"]))
	var floor_m := 0.2 * v.camera_state().cell_m
	_ok("D: a clamped camera keeps 2 cells x 0.1 above the drawn ground (§3.9)", float(on["min_clamped_hag"]) >= floor_m - 0.01,
		"min %.1f m against %.1f m" % [float(on["min_clamped_hag"]), floor_m])

func _leg_e() -> void:
	vp.reset_view()
	await _settle()
	await _frames(6)
	var a := await _shot("")
	await _frames(2)
	var b := await _shot("05_b5_highest.png")
	var n_hi := _diff_px(_crop(a, _rect3d()), _crop(b, _rect3d()))
	_ok("E: B5 at the highest altitude: two static frames identical", n_hi == 0, "%d px differ" % n_hi)
	_aim(_coast_cell(), 0.0, 6.0, 45.0)
	await _settle()
	await _frames(6)
	a = await _shot("")
	await _frames(2)
	b = await _shot("05_b5_lowest_coast.png")
	var n_lo := _diff_px(_crop(a, _rect3d()), _crop(b, _rect3d()))
	_ok("E: the near plane low down is 5 % of the height above ground (§3.11)", absf(v.camera3d().near - clampf(0.05 * v.last_hag, 1.0, 1000.0)) < 1e-3,
		"near %.3f m at %.1f m above ground" % [v.camera3d().near, v.last_hag])
	_ok("E: B5 at the lowest altitude over a coast: two static frames identical", n_lo == 0,
		"%d px differ, near %.2f m far %.0f m" % [n_lo, v.camera3d().near, v.camera3d().far])

## Frame times between process frames (`View3DGate._time()`'s method).
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

func c_pos() -> Array:
	return v.camera_state().position()

func _leg_f() -> void:
	DisplayServer.window_set_vsync_mode(DisplayServer.VSYNC_DISABLED)
	OS.low_processor_usage_mode = false
	var peak := _highest_cell()
	_aim(peak + Vector2(3.0, 3.0), _fit_span / 16.0, 45.0)
	await _settle()
	var mem := Performance.get_monitor(Performance.RENDER_VIDEO_MEM_USED)
	var pos := c_pos()
	var hag := v.last_hag
	_ok("F: the near plane is 5 % of the height above ground, 1 m..1 km (§3.11)", absf(v.camera3d().near - clampf(0.05 * hag, 1.0, 1000.0)) < 1e-3,
		"near %.3f m at %.1f m above ground" % [v.camera3d().near, hag])
	var farthest := 0.0
	var cs: View3DCamera = v.camera_state()
	for corner in [Vector2(0, 0), Vector2(cs.world_w_m, 0), Vector2(0, cs.world_h_m), Vector2(cs.world_w_m, cs.world_h_m)]:
		farthest = maxf(farthest, Vector3(float(pos[0]) - corner.x, float(pos[1]), float(pos[2]) - corner.y).length())
	_ok("F: the far plane reaches the farthest world corner", v.camera3d().far >= farthest, "far %.0f m, corner %.0f m" % [v.camera3d().far, farthest])
	for i in 60:
		await get_tree().process_frame
	var st := await _time(300, func(_i): pass)
	var prims := Performance.get_monitor(Performance.RENDER_TOTAL_PRIMITIVES_IN_FRAME)
	var draws := Performance.get_monitor(Performance.RENDER_TOTAL_DRAW_CALLS_IN_FRAME)
	## Two patches of 256 x 256 quads, two triangles each: §3.12's Basic row
	## (131 072 triangles) for the window, the same again for the backdrop.
	var tris := int(v.terrain()._mesh.get_faces().size() / 3)
	_ok("F: the patch is 257 x 257 vertices, 131 072 triangles (§3.12's Basic row)", tris == 131072, "%d triangles in the patch; %d primitives in the frame" % [tris, prims])
	print("FRAME static median=%.3f min=%.3f max=%.3f p95=%.3f ms primitives=%d draws=%d video_mem=%.1f MiB (2D before 3D %.1f MiB, +%.1f MiB) window=%s" % [
		st["median"], st["min"], st["max"], st["p95"], prims, draws, mem / 1048576.0, _mem_2d / 1048576.0, (mem - _mem_2d) / 1048576.0, JSON.stringify(_win_summary())])
	_ok("F: Basic's frame bar, static: p95 <= 33 ms", float(st["p95"]) <= 33.0, "p95 %.3f ms" % float(st["p95"]))
	var c: View3DCamera = v.camera_state()
	var mv := await _time(300, func(_i): c.orbit(deg_to_rad(0.5), 0.0))
	print("FRAME orbiting median=%.3f min=%.3f max=%.3f p95=%.3f ms" % [mv["median"], mv["min"], mv["max"], mv["p95"]])
	var built := v.terrain().tiles_built
	var zf := await _time(120, func(i): c.zoom(1.02 if i < 60 else 1.0 / 1.02))
	print("FRAME zooming (level changes, fills) median=%.3f min=%.3f max=%.3f p95=%.3f ms tiles_built=%d" % [
		zf["median"], zf["min"], zf["max"], zf["p95"], v.terrain().tiles_built - built])

## Desktop: synthesised mouse; touch: synthesised touches.
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

## A control's drawn centre in window pixels.
func _centre_of(ctl: Control) -> Vector2:
	return (ctl.get_global_rect() * get_viewport().get_final_transform()).get_center()

func _navpad_btn(tip: String) -> Button:
	var pad = vp._navpad
	if pad == null:
		return null
	for b in pad.find_children("*", "Button", true, false):
		if (b as Button).tooltip_text == tip and (b as Button).is_visible_in_tree():
			return b
	return null

func _tap(pos: Vector2) -> void:
	_touch_ev(0, pos, true)
	await _frames(2)
	_touch_ev(0, pos, false)
	await _frames(3)

func _leg_g() -> void:
	var c: View3DCamera = v.camera_state()
	var r := _rect3d()
	var mid := r.get_center() + Vector2(0, r.size.y * 0.1)
	_aim(_highest_cell() + Vector2(3.0, 3.0), _fit_span / 8.0, 50.0)
	await _settle()
	var zoom2d: float = vp._zoom
	if not _touch:
		var d0 := c.dist
		_mouse(mid, MOUSE_BUTTON_WHEEL_UP, true)
		_mouse(mid, MOUSE_BUTTON_WHEEL_UP, false)
		await _frames(2)
		_ok("G: the wheel zooms in by ZOOM_WHEEL_STEP", absf(d0 / c.dist - 1.15) < 1e-6, "ratio %.6f" % (d0 / c.dist))
		var y0 := c.yaw
		_mouse(mid, MOUSE_BUTTON_RIGHT, true)
		for i in 4:
			_motion(mid + Vector2(25 * (i + 1), 0), Vector2(25, 0), MOUSE_BUTTON_MASK_RIGHT)
			await _frames(1)
		_mouse(mid + Vector2(100, 0), MOUSE_BUTTON_RIGHT, false)
		await _frames(2)
		_ok("G: a right drag orbits", absf(rad_to_deg(wrapf(c.yaw - y0, -PI, PI)) + 100.0 * 0.25) < 0.5, "yaw moved %.2f°" % rad_to_deg(wrapf(c.yaw - y0, -PI, PI)))
		## §3.9's pitch range, 5° to 90°, by dragging far past each end, over
		## the deepest sea (flattened, so the ground clamp never lifts the
		## camera there and the pitch read is the drag's alone).
		_aim(_deep_cell(), _fit_span / 16.0, 50.0)
		await _settle()
		for dy in [-2000.0, 2000.0]:
			_mouse(mid, MOUSE_BUTTON_RIGHT, true)
			_motion(mid + Vector2(0, dy * 0.01), Vector2(0, dy), MOUSE_BUTTON_MASK_RIGHT)
			_mouse(mid, MOUSE_BUTTON_RIGHT, false)
			await _frames(2)
			var want := 5.0 if dy < 0 else 90.0
			_ok("G: a right drag %s stops at %.0f°" % ["up" if dy < 0 else "down", want], absf(rad_to_deg(c.pitch) - want) < 0.01,
				"pitch %.2f°, clamp steps %d" % [rad_to_deg(c.pitch), v.last_clamp_steps])
		_aim(_highest_cell() + Vector2(3.0, 3.0), _fit_span / 8.0, 50.0)
		await _settle()
		c.yaw = 0.0
		var px := c.pivot_x
		_mouse(mid, MOUSE_BUTTON_LEFT, true)
		for i in 4:
			_motion(mid + Vector2(25 * (i + 1), 0), Vector2(25, 0), MOUSE_BUTTON_MASK_LEFT)
			await _frames(1)
		_mouse(mid + Vector2(100, 0), MOUSE_BUTTON_LEFT, false)
		await _frames(2)
		_ok("G: a left drag to the right moves the pivot west (the ground follows)", c.pivot_x < px, "%.1f -> %.1f m" % [px, c.pivot_x])
		v._mode_btn.pressed.emit()
		await _frames(3)
		_ok("G: Top-down switches to an orthographic camera", v.camera3d().projection == Camera3D.PROJECTION_ORTHOGONAL and c.ortho, v._mode_btn.text)
		await _shot("06_topdown.png")
		v._mode_btn.pressed.emit()
		await _frames(2)
		_ok("G: Orbit switches back", not c.ortho and v.camera3d().projection == Camera3D.PROJECTION_PERSPECTIVE)
		## The desktop's zoom and fit commands reach the same two functions
		## the touch row's buttons call.
		var d2 := c.dist
		vp.zoom_step(ViewportHost.ZOOM_BUTTON_STEP)
		_ok("G: ViewportHost.zoom_step() moves the 3D camera", absf(d2 / c.dist - ViewportHost.ZOOM_BUTTON_STEP) < 1e-6, "ratio %.6f" % (d2 / c.dist))
		vp.reset_view()
		_ok("G: ViewportHost.reset_view() frames the 3D world", absf(c.pivot_x - c.world_w_m * 0.5) < 1.0 and absf(c.pivot_z - c.world_h_m * 0.5) < 1.0)
		_ok("G: none of it moved the hidden 2D zoom", is_equal_approx(float(vp._zoom), zoom2d), "%.4f vs %.4f" % [vp._zoom, zoom2d])
	else:
		var plus := _navpad_btn("Zoom in")
		var minus := _navpad_btn("Zoom out")
		var fit := _navpad_btn("Reset view")
		_ok("G: the navpad's −, + and ⌖ are on screen over 3D", plus != null and minus != null and fit != null)
		if plus != null and minus != null and fit != null:
			var d0 := c.dist
			await _tap(_centre_of(plus))
			_ok("G: + moves the 3D camera in by ZOOM_BUTTON_STEP", absf(d0 / c.dist - ViewportHost.ZOOM_BUTTON_STEP) < 1e-6, "ratio %.6f" % (d0 / c.dist))
			d0 = c.dist
			await _tap(_centre_of(minus))
			_ok("G: − moves it out by the same step", absf(c.dist / d0 - ViewportHost.ZOOM_BUTTON_STEP) < 1e-6, "ratio %.6f" % (c.dist / d0))
			await _tap(_centre_of(fit))
			_ok("G: ⌖ frames the world", absf(c.pivot_x - c.world_w_m * 0.5) < 1.0 and absf(c.pivot_z - c.world_h_m * 0.5) < 1.0,
				"pivot %.0f,%.0f of %.0f x %.0f m" % [c.pivot_x, c.pivot_z, c.world_w_m, c.world_h_m])
			_ok("G: the hidden 2D zoom did not move", is_equal_approx(float(vp._zoom), zoom2d), "%.4f vs %.4f" % [vp._zoom, zoom2d])
		_aim(_highest_cell() + Vector2(3.0, 3.0), _fit_span / 8.0, 50.0)
		await _settle()
		c.yaw = 0.0
		var px := c.pivot_x
		_touch_ev(0, mid, true)
		for i in 4:
			_drag_ev(0, mid + Vector2(25 * (i + 1), 0), Vector2(25, 0))
			await _frames(1)
		_touch_ev(0, mid + Vector2(100, 0), false)
		await _frames(2)
		_ok("G: one finger pans (the ground follows)", c.pivot_x < px, "%.1f -> %.1f m" % [px, c.pivot_x])
		var d1 := c.dist
		var a := mid - Vector2(80, 0)
		var b := mid + Vector2(80, 0)
		_touch_ev(0, a, true)
		_touch_ev(1, b, true)
		await _frames(1)
		for i in 4:
			a -= Vector2(20, 0)
			b += Vector2(20, 0)
			_drag_ev(0, a, Vector2(-20, 0))
			_drag_ev(1, b, Vector2(20, 0))
			await _frames(1)
		_touch_ev(0, a, false)
		_touch_ev(1, b, false)
		await _frames(2)
		_ok("G: spreading two fingers zooms in", c.dist < d1, "%.0f -> %.0f m (fingers 160 -> 320 px)" % [d1, c.dist])
		var y1 := c.yaw
		var p1 := c.pitch
		a = mid - Vector2(80, 0)
		b = mid + Vector2(80, 0)
		_touch_ev(0, a, true)
		_touch_ev(1, b, true)
		await _frames(1)
		for i in 4:
			var ang := deg_to_rad(5.0 * (i + 1))
			var na := mid - Vector2(80, 0).rotated(ang)
			var nb := mid + Vector2(80, 0).rotated(ang)
			_drag_ev(0, na, na - a)
			_drag_ev(1, nb, nb - b)
			a = na
			b = nb
			await _frames(1)
		_touch_ev(0, a, false)
		_touch_ev(1, b, false)
		await _frames(2)
		_ok("G: twisting two fingers turns the camera", absf(rad_to_deg(wrapf(c.yaw - y1, -PI, PI)) + 20.0) < 1.0, "yaw moved %.2f°" % rad_to_deg(wrapf(c.yaw - y1, -PI, PI)))
		a = mid - Vector2(80, 0)
		b = mid + Vector2(80, 0)
		_touch_ev(0, a, true)
		_touch_ev(1, b, true)
		await _frames(1)
		for i in 4:
			a += Vector2(0, -20)
			b += Vector2(0, -20)
			_drag_ev(0, a, Vector2(0, -20))
			_drag_ev(1, b, Vector2(0, -20))
			await _frames(1)
		_touch_ev(0, a, false)
		_touch_ev(1, b, false)
		await _frames(2)
		_ok("G: two fingers moving up together tilt toward the horizon", c.pitch < p1, "pitch %.1f° -> %.1f°" % [rad_to_deg(p1), rad_to_deg(c.pitch)])
		_ok("G: none of it moved the hidden 2D zoom", is_equal_approx(float(vp._zoom), zoom2d))

## **I. The HEIGHT row** (§7 Q4's design, Rulings CT and EB items 5-6): on
## the PC and the tablet it sits between the info line and the actions, and a
## click at each end of its track (synthesised mouse or touch) sets 4x and 1x
## on the drawn mesh and in the stored preference. On the phone it is absent
## (its home is MAP ▸ View, unbuilt).
func _leg_i() -> void:
	var s: HSlider = v._height_slider
	if DccTheme.is_phone():
		_ok("I: no HEIGHT row in the phone's card", s == null)
		return
	_ok("I: the HEIGHT row exists and is shown", s != null and s.is_visible_in_tree())
	if s == null:
		return
	var info_y := v._info.get_global_rect().end.y
	var act_y := v._back_btn.get_global_rect().position.y
	var sy := s.get_global_rect().get_center().y
	_ok("I: between the info line and the actions", sy > info_y and sy < act_y, "info end %.0f, slider %.0f, actions %.0f" % [info_y, sy, act_y])
	_ok("I: 1x to 4x in steps of 0.25", s.min_value == 1.0 and s.max_value == 4.0 and s.step == 0.25, "%.2f..%.2f step %.2f" % [s.min_value, s.max_value, s.step])
	if DccTheme.is_touch():
		_ok("I: 44 dp tall on touch", s.get_global_rect().size.y >= 44.0 * maxf(v._unit, 1.0) - 0.5, "%.1f px" % s.get_global_rect().size.y)
	var r := s.get_global_rect() * get_viewport().get_final_transform()
	for want in [4.0, 1.0]:
		var at := Vector2(r.position.x + (r.size.x - 2.0 if want == 4.0 else 2.0), r.get_center().y)
		if _touch:
			await _tap(at)
		else:
			_mouse(at, MOUSE_BUTTON_LEFT, true)
			await _frames(1)
			_mouse(at, MOUSE_BUTTON_LEFT, false)
			await _frames(2)
		_ok("I: a press at the track's %s end sets %.0fx on the mesh and in Preferences" % ["right" if want == 4.0 else "left", want],
			is_equal_approx(v.terrain().exaggeration(), want) and is_equal_approx(DccSettings.view3d_exaggeration(-1.0), want),
			"mesh %.2f, stored %.2f, readout %s" % [v.terrain().exaggeration(), DccSettings.view3d_exaggeration(-1.0), v._height_value.text])
	await _shot("08_height_row.png")

func _leg_h() -> void:
	vp.set_3d(false)
	await _wait_no_coach()
	await _park_pointer()
	var after := _crop(await _shot("07_2d_after.png"), _rect2d())
	var n := _diff_px(_before2d, after)
	_ok("H: leaving 3D gives back the 2D map byte-identical", n == 0, "%d px differ" % n)
