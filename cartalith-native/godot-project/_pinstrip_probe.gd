extends Node
## Pin-palette PP-6 follow-up (coordinator review of `phone_phone_sheet_half.png`): the phone's tool strip must sit ABOVE the map's
## pins with an opaque backing, its armed chip must read at the phone label size ("PINS", not "POI"), and a pin must be the same
## physical size (in dp) on the phone as on the tablet and desktop. Windowed (refuses --headless), under a scratch APPDATA.
##
##   PHONE:   `-- --vp 1080x2340 --force-touch --density 2.625 --out <dir> --tag <name>`
##   TABLET:  `-- --vp 1700x1000 --force-touch --density=1.0 --out <dir> --tag <name>`
##   DESKTOP: `-- --vp 1700x1000 --out <dir> --tag <name>`
## Every form prints `PINDP <form> landmark_dp=<..> poi_dp=<..>`: the pin HEAD radius in dp (screen px / px-per-dp), the number the
## three forms are compared on. Frames are `<tag>_<form>_strip.png`. Exit 0 = held, 1 = failed, 2 = could not run.

var app: Node
var _vp: SubViewport
var _fails := 0
var _out := ""
var _tag := "real"
var _form := "desktop"


func _p(s: String) -> void:
	print(s)


func _ok(name: String, cond: bool, detail: String = "") -> void:
	if cond:
		_p("PASS " + name)
	else:
		_fails += 1
		_p("FAIL %s  %s" % [name, detail])


func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame


func _shot(name: String) -> void:
	if _out == "":
		return
	await _frames(3)
	await RenderingServer.frame_post_draw
	_vp.get_texture().get_image().save_png(_out.path_join("%s_%s_%s" % [_tag, _form, name]))
	_p("shot %s_%s_%s" % [_tag, _form, name])


## The ancestor path from `n` up to the root, as a list of nodes (n first).
func _chain(n: Node) -> Array:
	var out: Array = []
	while n != null:
		out.append(n)
		n = n.get_parent()
	return out


## Draw order of two canvas items in one tree: true when `a` draws AFTER (above) `b`. Compares tree order below their lowest common
## ancestor, and an effective z_index first (z_index is absolute-accumulating, so it dominates sibling order).
func _draws_above(a: CanvasItem, b: CanvasItem) -> bool:
	var za := _eff_z(a)
	var zb := _eff_z(b)
	if za != zb:
		return za > zb
	var ca := _chain(a)
	var cb := _chain(b)
	for x in ca:
		if cb.has(x):
			var ia: Node = ca[ca.find(x) - 1] if ca.find(x) > 0 else null
			var ib: Node = cb[cb.find(x) - 1] if cb.find(x) > 0 else null
			if ia == null or ib == null:
				return ia != null   # one contains the other: the descendant draws above
			return ia.get_index() > ib.get_index()
	return false


func _eff_z(c: CanvasItem) -> int:
	var z := 0
	var n: Node = c
	while n is CanvasItem:
		var ci := n as CanvasItem
		z += ci.z_index
		if not ci.z_as_relative:
			break
		n = n.get_parent()
	return z


func _ready() -> void:
	var vp_size := Vector2i(1700, 1000)
	var touch := false
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		var a := String(args[i])
		if a == "--vp":
			var wh := String(args[i + 1]).split("x")
			vp_size = Vector2i(int(wh[0]), int(wh[1]))
			i += 2
		elif a == "--out":
			_out = String(args[i + 1])
			i += 2
		elif a == "--tag":
			_tag = String(args[i + 1])
			i += 2
		elif a == "--density":
			i += 2
		elif a == "--force-touch":
			touch = true
			i += 1
		elif a.begins_with("--density="):
			i += 1
		else:
			_p("ABORT unknown argument %s" % a)
			get_tree().quit(2)
			return
	if DisplayServer.get_name() == "headless":
		_p("ABORT run windowed")
		get_tree().quit(2)
		return
	var wd := get_tree().create_timer(600.0)
	wd.timeout.connect(func():
		_p("WATCHDOG")
		get_tree().quit(3))
	_vp = SubViewport.new()
	_vp.size = vp_size
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await get_tree().create_timer(1.4).timeout
	if app.get("open_project_dialog") != null:
		app.open_project_dialog.hide()
	await _frames(4)
	_vp.notification(Node.NOTIFICATION_VP_MOUSE_ENTER)
	_form = "phone" if DccTheme.is_phone() else ("tablet" if DccTheme.is_tablet() and DccTheme.is_touch() else "desktop")
	_p("form: %s, viewport %s" % [_form, str(vp_size)])
	var bridge = app.bridge
	bridge.generate({"seed": 1200, "width_km": 1200.0, "grid_w": 512, "grid_h": 384,
		"archetype": "", "villages": true, "sea_level": 0.42})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(0.8).timeout
	_ok("a world generated", bridge.has_world)
	var ov = app.viewport.overlay
	await bridge.landmark_run()
	await _frames(6)
	var t0 := Time.get_ticks_msec()
	while app.busy_overlay != null and app.busy_overlay.visible and Time.get_ticks_msec() - t0 < 30000:
		await _frames(10)
	await _frames(20)
	DccSettings.set_marker_style("pins")
	for k in ["tavern", "harbour", "castle"]:
		DccSettings.pin_recent_push(k)
	app.select_domain("cartography")
	await _frames(8)
	app.arm_tool("inspect")
	await _frames(4)
	## Pin head radius in dp: local radius x the camera's zoom is screen px; a dp is the phone's scale on the phone, 1 elsewhere.
	var unit := float(DccTheme.phone_scale()) if DccTheme.is_phone() else 1.0
	var lms: Array = bridge.landmarks()
	var lm_dp := 0.0
	if not lms.is_empty():
		lm_dp = ov._landmark_pin_r(lms[0]) * float(ov._camera_zoom) / unit
	var poi_dp: float = ov._poi_pin_r() * float(ov._camera_zoom) / unit
	_p("PINDP %s landmark_dp=%.2f poi_dp=%.2f unit=%.3f zoom=%.3f" % [_form, lm_dp, poi_dp, unit, float(ov._camera_zoom)])
	## Item 3: a pin is the same physical size on every form. Measured 2026-11-15 (desktop 14.28 / 9.53, tablet 13.75 / 9.08, phone
	## 15.02 / 9.92 dp, landmark head / POI head); a double scale on the phone would read about 2.6x, so the band is +-20 percent
	## of 14.0 and 9.5, the head floor (`PIN_HEAD_MIN_PX` 9 dp) plus the class radius, not a tight pin.
	## 2026-11-23 (`_lmsize_probe.gd`): landmark heads now follow the settlement pins' law (`_pin_head_r`), so the band is the floor
	## (9 dp, a local landmark) to the metropolis-tier floor (14.4 dp) plus the continental class's larger size, 9..18 dp.
	_ok("landmark pin head %.2f dp is within 9.0..18.0 (the same on every form)" % lm_dp, lm_dp >= 8.9 and lm_dp <= 18.1)
	_ok("POI pin head %.2f dp is within 7.6..11.4 (the same on every form)" % poi_dp, poi_dp >= 7.6 and poi_dp <= 11.4)
	if _form == "phone":
		app.set_phone_detent("peek")
		await _frames(20)
		app.call_arming_from_map(func() -> void: app.arm_tool("poi"))
		await _frames(24)
		var strip := app.find_child("ToolStrip", true, false) as PanelContainer
		_ok("the tool strip exists and is visible", strip != null and strip.visible)
		if strip != null:
			var pr := strip.get_global_rect()
			## Item 1: the strip draws above the map's pin layer, and its backing is opaque.
			_ok("strip draws above the map overlay (pin layer)", _draws_above(strip, ov))
			var sb := strip.get_theme_stylebox("panel") as StyleBoxFlat
			_ok("strip backing is opaque (alpha 1)", sb != null and sb.bg_color.a >= 0.999, str(sb.bg_color if sb != null else null))
			## Pixel proof: no pin-white shows between the recents; sample the strip's own interior corners (padding area).
			await _shot("strip.png")
			var img := _vp.get_texture().get_image()
			var bg: Color = sb.bg_color if sb != null else Color.BLACK
			var bad := 0
			var n := 0
			## Row 2's empty ends (clear of the rounded corners, 37 px at this scale, and of the recents tiles).
			for fx in [0.06, 0.94]:
				for fy in [0.80, 0.86]:
					var p := Vector2i(int(pr.position.x + pr.size.x * fx), int(pr.position.y + pr.size.y * fy))
					n += 1
					var c := img.get_pixelv(p)
					if absf(c.r - bg.r) + absf(c.g - bg.g) + absf(c.b - bg.b) > 0.06:
						bad += 1
			_ok("strip corners show the backing colour, not the map (%d/%d off)" % [bad, n], bad == 0)
		## Item 2: the armed chip reads PINS at no less than the phone label floor.
		var fan = app.get("_tool_fan")
		var label := String(fan.pill_label()) if fan != null else ""
		_ok("armed chip label is PINS", label == "PINS", label)
		var pill = fan._pill if fan != null else null
		var fs := int(pill.label_font_size()) if pill != null and pill.has_method("label_font_size") else -1
		var floor_fs: int = app.phone_font(DccTheme.FS_SMALL)
		_ok("armed chip label size %d >= phone label floor %d" % [fs, floor_fs], fs >= floor_fs)
	else:
		_p("(strip legs are phone-only)")
	## Item 3 is judged by the caller from the three PINDP lines; the phone's must match the tablet's within 15 percent.
	_finish()


func _finish() -> void:
	_p("== done: %d failed ==" % _fails)
	get_tree().quit(1 if _fails > 0 else 0)
