extends Node
## Owner findings of 2026-11-21, batch 4 (F20, F21, F23, and the assistant sheet's layout): one windowed probe, one leg each.
##
##   godot --path . --resolution 1600x900 _fgui4_probe.tscn -- --leg all --tag after --out DIR
##   tablet: --resolution 1700x1000 ... -- --force-touch --density=1.0       phone: --resolution 450x975 ... --density 1.094
##
## Legs: `f20` (3D Full frames at x1, x4, x16, x64 and the primitive and frame cost, F20/F21: `--tag before|after` names the
## files; it ASSERTS only that the frames exist and that the geometry is the low-passed one), `f23` (2D rivers zoomed in: a
## frame at x16 and x64, the paint params, and the opacity of the river centre line), `assist` (the assistant sheet's right
## edge and the Context panel's reply height). **WINDOWED**, scratch APPDATA. Result lines `F4 ok|FAIL`; exit 0/1/2.

const View3DGate := preload("res://shell/view3d_gate.gd")
const View3DCamera := preload("res://shell/view3d_camera.gd")

var app: Node
var bridge
var vp
var v
var _checks := 0
var _fails := 0
var _out := ""
var _tag := "after"
var _form := "desktop"
var _fit_span := 0.0


func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame


func _arg(name: String, dflt: String) -> String:
	var args := OS.get_cmdline_user_args()
	var i := args.find(name)
	return String(args[i + 1]) if i >= 0 and i + 1 < args.size() else dflt


func _check(name: String, cond: bool, detail: String = "") -> void:
	_checks += 1
	print("F4 %s  %s%s" % ["ok  " if cond else "FAIL", name, ("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fails += 1


func _shot(name: String) -> Image:
	await _frames(3)
	await RenderingServer.frame_post_draw
	var img := get_viewport().get_texture().get_image()
	if _out != "":
		img.save_png(_out.path_join("f4_%s_%s_%s.png" % [_form, _tag, name]))
		print("F4 SHOT f4_%s_%s_%s.png" % [_form, _tag, name])
	return img


func _generate() -> void:
	bridge.generate({"seed": 483920, "width_km": 1200.0, "grid_w": 512, "grid_h": 384, "archetype": "", "villages": true, "sea_level": 0.42})
	var t0 := Time.get_ticks_msec()
	await _frames(2)
	while bool(bridge.generating) and Time.get_ticks_msec() - t0 < 240000:
		await _frames(10)
	await _frames(30)


func _ready() -> void:
	get_tree().create_timer(1200.0).timeout.connect(func() -> void:
		print("F4 WATCHDOG")
		get_tree().quit(2))
	if DisplayServer.get_name() == "headless":
		print("F4 FATAL headless")
		get_tree().quit(2)
		return
	_out = _arg("--out", "")
	_tag = _arg("--tag", "after")
	if _out != "":
		DirAccess.make_dir_recursive_absolute(_out)
	var leg := _arg("--leg", "all")
	if leg == "all" or leg == "f20":
		DccSettings.set_view3d_result({})
		DccSettings.set_view3d_forced_tier(View3DGate.TIER_FULL)
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.6).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	if "phone_project_picker" in app and app.phone_project_picker != null:
		app.phone_project_picker.hide()
	await _frames(6)
	bridge = app.bridge
	vp = app.viewport
	var vs := get_tree().root.size
	_form = ("phone" if DccTheme.is_phone() else ("tablet" if DccTheme.is_tablet() else "desktop")) + "_%dx%d" % [vs.x, vs.y]
	print("F4 form=%s" % _form)
	await _generate()
	for l in ["f23", "f23diag", "assist", "f20"]:
		if leg == l or (leg == "all" and l != "f23diag"):
			print("F4 --- leg %s" % l)
			await call("_leg_" + l)
	DccSettings.set_view3d_result({})
	DccSettings.set_view3d_forced_tier(-1)
	print("F4 RESULT %d checks, %d failed (%s)" % [_checks, _fails, _form])
	get_tree().quit(1 if _fails > 0 else 0)


# -- shared ---------------------------------------------------------------------------------------------------------
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
	var t0 := Time.get_ticks_msec()
	while vp.lod_pending() > 0 and Time.get_ticks_msec() - t0 < 60000:
		await get_tree().process_frame
	await _frames(8)


## The traced river with the most points, and its middle point (cells).
func _river_cell() -> Vector2:
	var best: PackedVector2Array = PackedVector2Array()
	for r in bridge.rivers(1):
		if r.has("parallel_of"):
			continue
		if r["points"].size() > best.size():
			best = r["points"]
	return best[best.size() / 2] if best.size() > 0 else Vector2(256, 192)


# -- F23 ------------------------------------------------------------------------------------------------------------
func _leg_f23() -> void:
	var rc := _river_cell()
	print("F4 river cell %s" % str(rc))
	## The Layers rows: rivers always draw, so there is no row, and an off request is ignored.
	var ids: Array = []
	for l in CartographyWorkspace.LIVE_LAYERS:
		ids.append(String((l as Dictionary)["id"]))
	_check("Layers has no Rivers row", not ids.has("rivers"), str(ids))
	app.layers_popover.open()
	await _frames(8)
	var all := ""
	var rivers_rows := 0
	for c in app.layers_popover.find_children("*", "Control", true, false):
		var t := ""
		if c is Label:
			t = (c as Label).text
		elif c is Button:
			t = (c as Button).text
		all += t + "|"
		if t.strip_edges().to_lower() == "rivers":
			rivers_rows += 1
	_check("the Layers popover lists layers but no row named Rivers", all.contains("Landmarks") and rivers_rows == 0, "%d rows; %s" % [rivers_rows, all.left(160)])
	app.layers_popover.hide()
	vp.set_layer_visible("rivers", false)
	await _frames(4)
	_check("an off request for rivers is ignored (still drawn)", vp.layer_visible("rivers") and bridge.rivers_in_map())
	for k in [4, 16, 64]:
		await _set_2d(rc, float(k))
		await _shot("rivers_x%d" % k)
		var n0 := 0
		var np := 0
		var lum := PackedFloat32Array()
		for key in vp._lod_tiles:
			var sp := vp._lod_tiles[key] as Sprite2D
			if sp == null or sp.texture == null:
				continue
			var im := sp.texture.get_image()
			for y in range(0, im.get_height(), 2):
				for x in range(0, im.get_width(), 2):
					var c := im.get_pixel(x, y)
					if c.a < 0.01:
						n0 += 1
						lum.append(c.get_luminance() * 255.0)
					elif c.a < 0.99:
						np += 1
		var mean := 0.0
		for l in lum:
			mean += l
		mean /= maxf(lum.size(), 1)
		var sd := 0.0
		for l in lum:
			sd += (l - mean) * (l - mean)
		sd = sqrt(sd / maxf(lum.size(), 1))
		print("F4 x%d tile river pixels: %d full-river (alpha 0), %d partial; full-river luminance mean %.1f sd %.2f" % [k, n0, np, mean, sd])
	vp.overlay.visible = true


## Diagnostic (not a bar): the luminance sd inside a wide river at x64 under each river look key switched off, to name
## what lets the ground show through the water.
func _leg_f23diag() -> void:
	var rc := _river_cell()
	for v in [{}, {"river_turbidity": 0.0}, {"river_wet_bank": 0.0}, {"river_floodplain_arid": 0.0}, {"local_contrast": 0.0}, {"river_turbidity": 0.0, "river_wet_bank": 0.0, "river_floodplain_arid": 0.0, "local_contrast": 0.0}]:
		bridge.set_appearance(v)
		await _set_2d(rc, 64.0)
		var im := await _shot("diag")
		var c := im.get_region(Rect2i(390, 330, 210, 40))
		var m := 0.0
		for y in c.get_height():
			for x in c.get_width():
				m += c.get_pixel(x, y).get_luminance() * 255.0
		m /= c.get_width() * c.get_height()
		var sd := 0.0
		for y in c.get_height():
			for x in c.get_width():
				var d := c.get_pixel(x, y).get_luminance() * 255.0 - m
				sd += d * d
		print("F4 diag %s: river crop mean %.1f sd %.2f" % [str(v), m, sqrt(sd / (c.get_width() * c.get_height()))])
	bridge.drop_appearance_overrides(PackedStringArray(["river_turbidity", "river_wet_bank", "river_floodplain_arid", "local_contrast"]))


# -- assistant ------------------------------------------------------------------------------------------------------
func _leg_assist() -> void:
	app.open_assistant_panel()
	await _frames(10)
	var aw = app.assistant_window
	aw._reply = "Kelvhold was walled in 1204. " + "A long reply line that wraps over several lines of the reply box. ".repeat(6)
	aw._reply_label = aw._add_msg("Assistant", aw._reply, "text_secondary", true)
	## The phone's status row also carries the "saved with the world" note when the conversation is stored with the world.
	if aw.controls.has("saved_with_world"):
		(aw.controls["saved_with_world"] as Control).visible = true
	await _frames(8)
	await _shot("assistant")
	var sw: float = get_tree().root.size.x / maxf(get_tree().root.content_scale_factor, 0.001)
	## The right edge of every visible control under the assistant, in root-viewport coordinates.
	var worst := 0.0
	var who := ""
	## The window's OWN viewport: the phone sheet is laid out in dp and scaled up, so the root's width is the wrong yardstick.
	var vis_w: float = aw.get_visible_rect().size.x if aw.visible else get_viewport().get_visible_rect().size.x
	if not aw.visible and app.get("right_dock_ctrl") != null:
		vis_w = float(app.right_dock_ctrl.get("_ask_frame").get_global_rect().end.x)
	for c in aw._host.find_children("*", "Control", true, false):
		var ctl := c as Control
		if not ctl.is_visible_in_tree():
			continue
		var r := ctl.get_global_rect()
		if ctl is Button or ctl is Label:
			if r.end.x > worst:
				worst = r.end.x
				who = "%s(%s)" % [ctl.name, (ctl as Button).text if ctl is Button else (ctl as Label).text.left(24)]
	print("F4 assistant: widest right edge %.1f of viewport %.1f by %s; body minimum width %.1f" % [worst, vis_w, who, aw._body.get_combined_minimum_size().x])
	for ch in aw._body.get_children():
		if ch is Control and (ch as Control).visible and (ch as Control).get_combined_minimum_size().x > vis_w * 0.8:
			print("F4   wide child %s (%s) min %.1f" % [ch.name, ch.get_class(), (ch as Control).get_combined_minimum_size().x])
	_check("no assistant button or label runs past the right edge", worst <= vis_w + 0.5, "%.1f > %.1f by %s" % [worst, vis_w, who])
	for want in ["Settings", "Profile", "Turn on Assistant"]:
		var b: Control = null
		for c in aw._host.find_children("*", "Button", true, false):
			if (c as Button).text.to_lower().begins_with(want.to_lower()) and (c as Control).is_visible_in_tree():
				b = c
		if b != null:
			var r := b.get_global_rect()
			_check("%s is on screen" % want, r.position.x >= -0.5 and r.end.x <= vis_w + 0.5, str(r))
		else:
			print("F4 note: no visible '%s' button" % want)
	## Every task, the Edit task's "Clear region" row among them (owner finding: "Settings, Clear region" clipped on a 393 px
	## phone): no button or label ends past the window's own width, and on touch every button meets the 44 dp floor
	## (`DccTheme.PHONE_TAP_MIN` on the phone, the `btn_min_h` role on the tablet).
	var floor_px := 0.0
	if DccTheme.is_phone():
		floor_px = float(DccTheme.PHONE_TAP_MIN)
	elif DccTheme.is_tablet():
		floor_px = float(DccTheme.role_px("btn_min_h"))
	for t in ["prose", "story", "note", "commands", "edit"]:
		aw._pick_task(t)
		await _frames(8)
		var tw := 0.0
		var twho := ""
		var short := ""
		for c in aw._host.find_children("*", "Control", true, false):
			if not (c is Button or c is Label) or not (c as Control).is_visible_in_tree():
				continue
			var cr := (c as Control).get_global_rect()
			if cr.end.x > tw:
				tw = cr.end.x
				twho = "%s(%s)" % [c.name, (c as Button).text if c is Button else (c as Label).text.left(20)]
			if c is Button and floor_px > 0.0 and cr.size.y + 0.5 < floor_px and (c as Button).text != "Close":
				short += " %s=%.0f" % [(c as Button).text, cr.size.y]
		_check("task %s: nothing past the right edge" % t, tw <= vis_w + 0.5, "%.1f of %.1f by %s" % [tw, vis_w, twho])
		_check("task %s: every button meets the %d px floor" % [t, int(floor_px)], short == "", short)
		if t == "edit":
			await _shot("assistant_edit")
			_check("Edit shows Clear region on screen", aw.controls["region_clear"].is_visible_in_tree() and (aw.controls["region_clear"] as Control).get_global_rect().end.x <= vis_w + 0.5)
	aw._pick_task("prose")
	if not DccTheme.is_phone() and not DccTheme.is_tablet():
		var h: float = aw._scroll.size.y
		print("F4 reply area height %.1f" % h)
		_check("the reply area is at least 160 px tall", h >= 160.0, "%.1f" % h)


# -- F20 / F21 ------------------------------------------------------------------------------------------------------
func _q():
	return v.quadtree()


func _settle(limit_s := 90.0, quiet := 8) -> bool:
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
	return Vector2(at.x / float(img.get_width() - 1) * 511.0, at.y / float(img.get_height() - 1) * 383.0)


func _aim(cell: Vector2, span_m: float, pitch_deg: float, yaw_deg := 0.0) -> void:
	var c: View3DCamera = v.camera_state()
	c.set_ortho(false)
	c.pivot_x = cell.x * c.cell_m
	c.pivot_z = cell.y * c.cell_m
	c.dist = clampf(span_m * 0.5 / tan(deg_to_rad(View3DCamera.FOV_DEG) * 0.5), c.min_dist(), c.max_dist())
	c.pitch = deg_to_rad(pitch_deg)
	c.yaw = deg_to_rad(yaw_deg)
	c.tilt_left = 0.0


func _find_id(p: PopupMenu, id: int) -> PopupMenu:
	if p.get_item_index(id) >= 0:
		return p
	for c in p.get_children(true):
		if c is PopupMenu:
			var f := _find_id(c as PopupMenu, id)
			if f != null:
				return f
	return null


func _enter() -> bool:
	var win: PopupMenu = null
	for ch in app.menu_bar_row.get_children():
		if ch is MenuButton:
			var f := _find_id((ch as MenuButton).get_popup(), DccMenus.ID_WIN_VIEW_3D)
			if f != null:
				win = f
	if win == null:
		return false
	win.id_pressed.emit(DccMenus.ID_WIN_VIEW_3D)
	v = vp.view3d()
	var t0 := Time.get_ticks_msec()
	while v != null and v.busy() and Time.get_ticks_msec() - t0 < 120000:
		await get_tree().process_frame
	return v != null


func _leg_f20() -> void:
	DisplayServer.window_set_vsync_mode(DisplayServer.VSYNC_DISABLED)
	OS.low_processor_usage_mode = false
	if not await _enter():
		_check("3D opens", false)
		return
	_check("Full is forced", v.tier() == View3DGate.TIER_FULL)
	await _settle()
	var t0 := Time.get_ticks_msec()
	while not (_q() != null and _q().world_known() and not v._place_pending and not v._frame_world_pending) and Time.get_ticks_msec() - t0 < 60000:
		await get_tree().process_frame
	var c: View3DCamera = v.camera_state()
	_fit_span = maxf(c.world_h_m, c.world_w_m / (float(v.subviewport().size.x) / v.subviewport().size.y))
	var peak := _highest_cell() + Vector2(4.0, 4.0)
	for k in [1, 4, 16, 64]:
		_aim(peak, _fit_span / k, 40.0, 30.0)
		await _settle()
		await _frames(30)
		var img: Image = v.subviewport().get_texture().get_image()
		img.save_png(_out.path_join("f4_%s_%s_3d_x%d.png" % [_form, _tag, k]))
		var ts := PackedFloat64Array()
		var last := Time.get_ticks_usec()
		for i in 120:
			await get_tree().process_frame
			var now := Time.get_ticks_usec()
			ts.append((now - last) / 1000.0)
			last = now
		ts.sort()
		var q = _q()
		print("F4 3D x%d  frame median %.2f ms (min %.2f max %.2f)  primitives %d  draws %d  nodes %d levels %d..%d" % [k, ts[60], ts[0], ts[119],
			int(Performance.get_monitor(Performance.RENDER_TOTAL_PRIMITIVES_IN_FRAME)), int(Performance.get_monitor(Performance.RENDER_TOTAL_DRAW_CALLS_IN_FRAME)),
			q.drawn_count, q.level_min, q.level_max])
	_check("frames taken", true)
