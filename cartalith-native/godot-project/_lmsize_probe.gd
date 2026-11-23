extends Node
## Landmark/pin size law (owner 2026-11-23): every map pin kind sizes through `MapOverlay._pin_head_r`, the settlement pins' law, so a
## landmark and a settlement pin read at a CONSTANT ratio at every zoom. Measures the on-screen head radius (px = local r x camera zoom)
## of a settlement (town, city), landmarks (local, regional, continental), a POI, the sample pin and a placed icon at x1..x64 and asserts
## each landmark/settlement ratio varies <= 5 percent. Windowed, scratch APPDATA. Args as `_pinstrip_probe.gd`. Exit 0 held, 1 failed.
## Mutant: `tools/lmsize_mutant.py` puts the old zoom law back on the landmark head; this probe must then fail.

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
		await get_tree().create_timer(0.1).timeout
	await bridge.landmark_run()
	await _frames(6)
	var t0 := Time.get_ticks_msec()
	while app.busy_overlay != null and app.busy_overlay.visible and Time.get_ticks_msec() - t0 < 30000:
		await _frames(10)
	await _frames(20)
	DccSettings.set_marker_style("pins")
	app.select_domain("cartography")
	await _frames(8)
	var ov = app.viewport.overlay
	var rect: Rect2 = ov._displayed_rect()
	_ok("displayed rect is non-empty", rect.size.x > 0.0, str(rect))
	var unit := float(DccTheme.phone_scale()) if DccTheme.is_phone() else 1.0
	var lms := {
		"local": {"class": "local", "importance": 0.5},
		"regional": {"class": "regional", "importance": 0.5},
		"continental": {"class": "continental", "importance": 0.5}}
	var ic := {"family": "poi", "scale": 1.0, "x": 10.0, "y": 10.0}
	var zs := [1.0, 2.0, 4.0, 8.0, 16.0, 64.0]
	var rows := {}
	for z in zs:
		ov.set_camera_zoom(z)
		var row := {
			"town": float(ov._settlement_head_r("town", rect)) * z,
			"city": float(ov._settlement_head_r("city", rect)) * z,
			"lm_local": float(ov._landmark_pin_r(lms["local"])) * z,
			"lm_regional": float(ov._landmark_pin_r(lms["regional"])) * z,
			"lm_continental": float(ov._landmark_pin_r(lms["continental"])) * z,
			"poi": float(ov._poi_pin_r()) * z,
			"sample": float(ov.sample_pin_screen_radius()),
			"icon": float(ov._icon_radius(ic)) * z}
		rows[z] = row
		_p("SIZE %s x%d town=%.2f city=%.2f lm_local=%.2f lm_regional=%.2f lm_cont=%.2f poi=%.2f sample=%.2f icon=%.2f (px, unit %.3f)" % [
			_form, int(z), row["town"], row["city"], row["lm_local"], row["lm_regional"], row["lm_continental"], row["poi"], row["sample"], row["icon"], unit])
	## The bar: landmark/settlement ratio constant across zoom within 5 percent, for each landmark class against the city pin.
	for key in ["lm_local", "lm_regional", "lm_continental", "poi", "sample"]:
		var lo := 1e9
		var hi := 0.0
		for z in zs:
			var q: float = float(rows[z][key]) / float(rows[z]["city"])
			lo = minf(lo, q)
			hi = maxf(hi, q)
		_p("RATIO %s %s/city min=%.4f max=%.4f spread=%.2f%%" % [_form, key, lo, hi, 100.0 * (hi - lo) / lo])
		_ok("%s/city ratio constant x1..x64 within 5 percent" % key, (hi - lo) / lo <= 0.05, "%.4f..%.4f" % [lo, hi])
	## Positive control: the ratio is a real number (a vanishing or NaN head would pass a spread test).
	_ok("landmark head is at least the floor", float(rows[1.0]["lm_local"]) >= 9.0 * unit - 0.01, str(rows[1.0]["lm_local"]))
	_ok("continental landmark outsizes local", float(rows[1.0]["lm_continental"]) > float(rows[1.0]["lm_local"]))
	## Frames at x1, x4, x16 about a settlement (so its pin and a landmark's stand together): the real camera path.
	var host = app.viewport
	for z in [1.0, 4.0, 16.0]:
		host.reset_view()
		await _frames(10)
		var r0: Rect2 = ov.displayed_rect()
		var pivot: Vector2 = host.size * 0.5
		if not ov._settlements.is_empty():
			var st: Dictionary = ov._settlements[ov._settlements.size() / 2]
			pivot = host.get_global_transform().affine_inverse() * (ov.get_global_transform() * ov._cell_to_screen(Vector2(st["x"], st["y"]), r0))
		if z > 1.0:
			host._zoom_at(pivot, z)
		await _frames(12)
		await get_tree().create_timer(1.5).timeout
		await _shot("x%d.png" % int(z))
	_finish()


func _finish() -> void:
	_p("== done: %d failed ==" % _fails)
	get_tree().quit(1 if _fails > 0 else 0)
