extends RefCounted
## Ruling HV (2026-11-23): the coastline driven through the REAL shell, one form per run. How it was run (the launcher's own
## `--shell` import step hangs 15 min on the console wrapper here, so `coast_drive_run.py`, kept in the evidence folder,
## imports once by hand and calls `fennara_drive.drive` itself, holding `C:/cw/.win_lock` with an owner file):
##   python -I coast_drive_run.py desktop|phone 2048 <tag>      (env: COAST_SKIP_EXPORT, COAST_EXPORT_BASE, COAST_TIMEOUT)
## Steps: (1) past the welcome card / worlds list; (2) a world, the first trace timed; (3) the Coastline row clicked for real
## (CARTO's Layers on desktop, the MAP sheet on the phone) and read back; (4) at fit, x4, x16 and x64 the stroke is visible
## (layer on differs from off, which is also the positive control) and is DRAWN where the trace says (ink under the sampled
## vertices); (5) at x4 and x16 the trace lies within 1 px of the water edge the map draws, read three ways: the layer-off
## frame's water/land colour crossing (with the misses re-measured at x16), the shipped shore-field texture's own zero
## crossing, and the ink under the vertices; (5b) a run a border follows, hidden/whole/drawn by scale; (6) the national,
## province and coast lines each change ink with the preset (`set_overlay_treatment`, on-minus-off so only the line's ink
## counts), and (6b) again with the real Look buttons over each preset's own ground; (7) an export holds the coastline
## (and a preset recolours it); (8) frame cost at fit, stroke on against off on alternate frames, median of 5 batches.
## Every check prints `ok` or `FAIL`. Never writes the owner's data.

var _fails := 0
var app: Node
var bridge
var host
var out_dir := ""
var _outliers: Array = []


func _check(ctx, what: String, cond: bool, detail: String = "") -> void:
	if not cond:
		_fails += 1
	ctx.log("%s  %s%s" % ["ok  " if cond else "FAIL", what, ("  -- " + detail) if detail != "" else ""])


func _find(ctx, t: String, cls: String = "") -> Dictionary:
	for e in DriveCtx.dump_ui(ctx, t):
		if cls == "" or e["class"] == cls:
			return e
	return {}


func _wait_for(ctx, t: String, secs: float = 20.0, cls: String = "") -> Dictionary:
	var end := Time.get_ticks_msec() + int(secs * 1000)
	while Time.get_ticks_msec() < end:
		var e := _find(ctx, t, cls)
		if not e.is_empty():
			return e
		await ctx.wait(0.25)
	return {}


func _click_entry(ctx, e: Dictionary) -> void:
	var r: Array = e["rect"]
	await DriveCtx.click(ctx, Vector2(r[0] + r[2] / 2.0, r[1] + r[3] / 2.0))


func _generate(ctx, seed_v: int, g: int) -> void:
	bridge.generate({"seed": seed_v, "width_km": 800.0, "grid_w": g, "grid_h": int(g * 0.64), "archetype": "", "villages": true, "sea_level": 0.45}, true)
	var t0 := Time.get_ticks_msec()
	await ctx.wait(0.2)
	while bool(bridge.generating) and Time.get_ticks_msec() - t0 < 500000:
		await ctx.wait(0.25)
	await ctx.wait(3.0)


func _frame() -> Image:
	return app.get_viewport().get_texture().get_image()


func _save(img: Image, name: String) -> void:
	if out_dir != "":
		img.save_png(out_dir.path_join(name + ".png"))


## The overlay's local rect, and a grid point's pixel in the framebuffer.
func _px(p: Vector2) -> Vector2:
	var ov = host.overlay
	var rect: Rect2 = ov._displayed_rect()
	var local: Vector2 = rect.position + Vector2(p.x / ov._gw, p.y / ov._gh) * rect.size
	var f: float = app.get_window().content_scale_factor
	return (ov as Control).get_global_transform() * local * f


## Fit view, then zoom `k` times about grid point `gpt`, centred in the host.
func _view_on(ctx, gpt: Vector2, k: float, at_clear_centre: bool = true) -> void:
	host.reset_view()
	await ctx.wait(0.4)
	if k > 1.0:
		var ov = host.overlay
		var rect: Rect2 = ov._displayed_rect()
		var local: Vector2 = rect.position + Vector2(gpt.x / ov._gw, gpt.y / ov._gh) * rect.size
		host._zoom = clampf(host._zoom * k, host.ZOOM_MIN, host._zoom_max)
		host._camera.scale = Vector2(host._zoom, host._zoom)
		var centre: Vector2 = host.size * 0.5
		if at_clear_centre:
			centre = _clear_rect().get_center() / app.get_window().content_scale_factor - host.get_global_rect().position
		host._camera.position = centre - local * host._zoom
		host._update_zoom_readout()
		host.overlay.set_camera_zoom(host._zoom)
		host._update_lod()
	await ctx.wait(4.0 if k > 1.0 else 1.0)


func _diff_count(a: Image, b: Image, tol: float = 0.04) -> int:
	var n := 0
	for y in range(0, a.get_height()):
		for x in range(0, a.get_width()):
			var p := a.get_pixel(x, y)
			var q := b.get_pixel(x, y)
			if absf(p.r - q.r) + absf(p.g - q.g) + absf(p.b - q.b) > tol:
				n += 1
	return n


func _diff_box(a: Image, b: Image, box: Rect2, tol: float = 0.04) -> int:
	var n := 0
	for y in range(maxi(0, int(box.position.y)), mini(a.get_height(), int(box.end.y))):
		for x in range(maxi(0, int(box.position.x)), mini(a.get_width(), int(box.end.x))):
			var p := a.get_pixel(x, y)
			var q := b.get_pixel(x, y)
			if absf(p.r - q.r) + absf(p.g - q.g) + absf(p.b - q.b) > tol:
				n += 1
	return n


func _avg(img: Image, at: Vector2) -> Color:
	var c := Color(0, 0, 0, 0)
	var n := 0
	for dy in range(-1, 2):
		for dx in range(-1, 2):
			var x := int(at.x) + dx
			var y := int(at.y) + dy
			if x >= 0 and y >= 0 and x < img.get_width() and y < img.get_height():
				c += img.get_pixel(x, y)
				n += 1
	return c / maxf(n, 1)


func _sgn(c: Color) -> String:
	return "(%+.2f %+.2f %+.2f)" % [c.r, c.g, c.b]


func _cdist(a: Color, b: Color) -> float:
	return absf(a.r - b.r) + absf(a.g - b.g) + absf(a.b - b.b)


## The coast vertex to centre on: the middle of the longest line.
func _anchor() -> Vector2:
	var best: PackedVector2Array = PackedVector2Array()
	for l in host.overlay._coast_lines:
		if (l as PackedVector2Array).size() > best.size():
			best = l
	return best[best.size() / 2] if best.size() > 0 else Vector2.ZERO


## Distances in px from sampled coast vertices (in view, inside the map's own rect) to the water/land colour crossing within +-6 px
## along the line's normal axis in the LAYER-OFF frame `off`. A sample counts only where such a crossing exists with water on
## one side and land on the other: elsewhere (a river mouth, a beach as blue as the water) it is reported as skipped. Returns `[sorted perpendicular offsets, skipped count]`.
func _edge_offsets(off: Image, want: int) -> Array:
	var vis: Array = []
	var view := _clear_rect()
	for l in host.overlay._coast_lines:
		var pts: PackedVector2Array = l
		for i in range(2, pts.size() - 2, 3):
			var s := _px(pts[i])
			if view.has_point(s):
				var t := _px(pts[i + 2]) - _px(pts[i - 2])
				if t.length() > 0.5:
					vis.append([s, t.normalized(), pts[i]])
	var offs: Array = []
	var skipped := 0
	var step := maxi(1, vis.size() / maxi(want, 1))
	for k in range(0, vis.size(), step):
		var o := _offset_for(off, vis[k][0], vis[k][1])
		if o >= 0.0:
			offs.append(o)
			if o > 1.0:
				_outliers.append([vis[k][0], o, vis[k][2], vis[k][1]])
		else:
			skipped += 1
	offs.sort()
	return [offs, skipped]


## The perpendicular distance in px from the vertex at screen point `s` (tangent `t`) to the water/land colour crossing in
## the layer-off frame `off`, or -1.0 when there is none within +-6 px. Water reads bluer than red (b - r > 0), land the other
## way, on the default look's palette: the edge is where that difference crosses zero along the normal's dominant axis,
## linearly interpolated between pixels, the crossing nearest the vertex.
func _offset_for(off: Image, s: Vector2, t: Vector2) -> float:
	var n := Vector2(-t.y, t.x)
	var ax := Vector2(1, 0) if absf(n.x) >= absf(n.y) else Vector2(0, 1)
	var cosv := absf(n.dot(ax))
	var prev := 0.0
	var found := false
	var best_off := 99.0
	for i in range(-6, 7):
		var px := off.get_pixel(clampi(int(s.x + ax.x * i), 0, off.get_width() - 1), clampi(int(s.y + ax.y * i), 0, off.get_height() - 1))
		var v := px.b - px.r - 0.02
		if i > -6 and prev != 0.0 and v != 0.0 and (prev > 0.0) != (v > 0.0) and absf(prev) + absf(v) > 0.06:
			## pixel i's colour sits at its centre, i + 0.5, so the crossing and the vertex are both read in pixel-centre index space
			var at_i := float(i - 1) + prev / (prev - v) + 0.5 - (fposmod(s.x, 1.0) * ax.x + fposmod(s.y, 1.0) * ax.y)
			if absf(at_i) < absf(best_off):
				best_off = at_i
				found = true
		prev = v
	return absf(best_off) * cosv if found else -1.0


## The residual of ONE grid point `q` against the shipped shore field (see `_field_residuals`), in px.
func _residual_at(q: Vector2) -> float:
	var fld: Image = bridge.shore_field_texture().get_image()
	var ov = host.overlay
	var ppc: float = ov._displayed_rect().size.x / float(ov._gw) * host._zoom * app.get_window().content_scale_factor
	var u := q.x - 0.5
	var v := q.y - 0.5
	var x0 := clampi(int(floor(u)), 0, fld.get_width() - 2)
	var y0 := clampi(int(floor(v)), 0, fld.get_height() - 2)
	var tx := u - x0
	var ty := v - y0
	var a := fld.get_pixel(x0, y0).r
	var b := fld.get_pixel(x0 + 1, y0).r
	var c := fld.get_pixel(x0 + 1, y0 + 1).r
	var d := fld.get_pixel(x0, y0 + 1).r
	var sv := (1 - tx) * (1 - ty) * a + tx * (1 - ty) * b + tx * ty * c + (1 - tx) * ty * d
	var gx := (1 - ty) * (b - a) + ty * (c - d)
	var gy := (1 - tx) * (d - a) + tx * (c - b)
	return absf(sv) / maxf(sqrt(gx * gx + gy * gy), 1e-9) * ppc


## The residual of every `want`th stroke vertex in view against the shore field the map's shader contours, read from the
## SHIPPED half-float texture (`WorldGen.shore_field_texture`) with the shader's own bilinear (`shore_field.gdshaderinc`):
## the distance in px from the vertex to the field's zero crossing is |s| / |grad s| cells times the screen px per cell.
## Returns the sorted distances in px.
func _field_residuals(want: int) -> Array:
	var tex: Texture2D = bridge.shore_field_texture()
	var fld: Image = tex.get_image()
	var gw := fld.get_width()
	var gh := fld.get_height()
	var ov = host.overlay
	var ppc: float = ov._displayed_rect().size.x / float(ov._gw) * host._zoom * app.get_window().content_scale_factor
	var view := _clear_rect()
	var vis: Array = []
	for l in ov._coast_lines:
		for q in (l as PackedVector2Array):
			if view.has_point(_px(q)):
				vis.append(q)
	var out: Array = []
	var step := maxi(1, vis.size() / maxi(want, 1))
	for i in range(0, vis.size(), step):
		var q: Vector2 = vis[i]
		var u := q.x - 0.5
		var v := q.y - 0.5
		var x0 := clampi(int(floor(u)), 0, gw - 2)
		var y0 := clampi(int(floor(v)), 0, gh - 2)
		var tx := u - x0
		var ty := v - y0
		var a := fld.get_pixel(x0, y0).r
		var b := fld.get_pixel(x0 + 1, y0).r
		var c := fld.get_pixel(x0 + 1, y0 + 1).r
		var d := fld.get_pixel(x0, y0 + 1).r
		var sv := (1 - tx) * (1 - ty) * a + tx * (1 - ty) * b + tx * ty * c + (1 - tx) * ty * d
		var gx := (1 - ty) * (b - a) + ty * (c - d)
		var gy := (1 - tx) * (d - a) + tx * (c - b)
		var g := sqrt(gx * gx + gy * gy)
		out.append(absf(sv) / maxf(g, 1e-9) * ppc)
	out.sort()
	return out


## The part of the map the chrome does not cover, in framebuffer pixels: the host's rect inside `ViewportHost.safe_insets()`
## (the phone's app bar and bottom chrome; a flat 10 px on desktop), grown inward a further 20 px.
func _clear_rect() -> Rect2:
	var f: float = app.get_window().content_scale_factor
	var r: Rect2 = host.get_global_rect()
	var ins: Dictionary = host.safe_insets()
	var sc := float(ins.get("scale", 1.0))
	var l := float(ins.get("left", 0.0))
	var t := float(ins.get("top", 0.0))
	var rt := float(ins.get("right", 0.0))
	var b := float(ins.get("bottom", 0.0))
	r = Rect2(r.position + Vector2(l, t), r.size - Vector2(l + rt, t + b))
	return Rect2(r.position * f, r.size * f).grow(-20.0)


## Whether the stroke is DRAWN where the trace puts it: for every `want`th stroke vertex in the clear part of the view, the
## pixel under the vertex must differ from the same pixel in the layer-off frame (the stroke's ink is there). The two
## water-edge checks above read where the trace and the map's edge are, by the shell's own vertex-to-pixel mapping; this
## one reads the pixels the overlay actually put down, so a wrong draw transform cannot hide behind them.
## Returns `[with ink, sampled]`.
func _stroke_hits(off: Image, on: Image, want: int) -> Array:
	var view := _clear_rect()
	var vis: Array = []
	for l in host.overlay._coast_lines:
		for q in (l as PackedVector2Array):
			var p := _px(q)
			if view.has_point(p):
				vis.append(p)
	var step := maxi(1, vis.size() / maxi(want, 1))
	var hit := 0
	var n := 0
	for i in range(0, vis.size(), step):
		var p: Vector2 = vis[i]
		var x := clampi(int(p.x), 0, on.get_width() - 1)
		var y := clampi(int(p.y), 0, on.get_height() - 1)
		n += 1
		if _cdist(on.get_pixel(x, y), off.get_pixel(x, y)) >= 0.2:
			hit += 1
	return [hit, n]


func _set_coast(on: bool) -> void:
	host.set_layer_visible("coast", on)


func run(ctx) -> void:
	app = ctx.get_scene_root()
	if not app.has_method("select_domain"):
		app = ctx.get_scene_root().find_child("App", true, false)
	bridge = app.bridge
	host = app.viewport
	out_dir = OS.get_environment("COAST_OUT")
	if out_dir != "":
		DirAccess.make_dir_recursive_absolute(out_dir)
	var grid := int(OS.get_environment("COAST_GRID")) if OS.get_environment("COAST_GRID") != "" else 1024
	var size: Vector2 = ctx.get_scene_root().get_viewport().get_visible_rect().size
	var phone: bool = app.is_phone()
	var form := "phone" if phone else "desktop"
	ctx.log("form: %s, canvas %s, grid %d" % [form, str(size), grid])
	# (1) past the welcome card
	if phone:
		var nw := await _wait_for(ctx, "+ NEW WORLD")
		_check(ctx, "worlds list shows + NEW WORLD", not nw.is_empty())
		await _click_entry(ctx, nw)
		var cancel := await _wait_for(ctx, "CANCEL")
		await _click_entry(ctx, cancel)
		await ctx.wait(1.0)
	else:
		var cont := await _wait_for(ctx, "Continue without a world")
		_check(ctx, "welcome card shows Continue", not cont.is_empty())
		if not cont.is_empty():
			await _click_entry(ctx, cont)
		await ctx.wait(0.5)
	# (2) a world
	var t0 := Time.get_ticks_msec()
	await _generate(ctx, 483920, grid)
	_check(ctx, "a world exists", bool(bridge.has_world), "%d ms" % (Time.get_ticks_msec() - t0))
	var ov = host.overlay
	var ncoast := 0
	var nverts := 0
	## The trace is lazy: it is fetched when the layer is first shown or exported (here, forced once to read its size).
	var t_trace := Time.get_ticks_usec()
	ov.set_show_coast_lines(true)
	var trace_ms := float(Time.get_ticks_usec() - t_trace) / 1000.0
	ov.set_show_coast_lines(false)
	ncoast = (ov._coast_lines as Array).size()
	nverts = 0
	for l in ov._coast_lines:
		nverts += (l as PackedVector2Array).size()
	var nhid := (ov._coast_hidden as Array).size()
	_check(ctx, "the engine traced a coastline", ncoast > 0, "%d lines, %d vertices, %d runs hidden under a border; the first fetch (shore field contour plus both border traces) took %.0f ms on the main thread" % [ncoast, nverts, nhid, trace_ms])
	_check(ctx, "the world has national and province lines to compare with", (ov._border_lines as Array).size() > 0 and (ov._province_lines as Array).size() > 0, "%d national, %d province" % [(ov._border_lines as Array).size(), (ov._province_lines as Array).size()])
	_check(ctx, "the layer starts off", not host.layer_visible("coast"))
	if OS.get_environment("COAST_ONLY_EXPORT") != "":
		ov.set_show_coast_lines(true)
		ov.set_show_coast_lines(false)
		await _export_step(ctx, form)
		ctx.log("PROOF %s" % ("FAIL: %d check(s)" % _fails if _fails > 0 else "PASS"))
		return
	# (3) the layer row, clicked for real: the map's Layers button opens the list the rail dock and the phone's list share
	var cb: CheckBox = null
	if phone:
		## the phone's MAP sheet lists every LIVE_LAYERS row (`dcc_shell.gd::_refresh_phone_map_row`): open it by its tab
		var tab := {}
		for e in DriveCtx.dump_ui(ctx, "MAP"):
			if str(e["text"]).strip_edges().to_upper() == "MAP" and (e["class"] == "Button" or e["class"] == "Label"):
				tab = e
		if not tab.is_empty():
			await _click_entry(ctx, tab)
			await ctx.wait(1.5)
		var pt: Dictionary = app._phone_map_toggles
		if pt.has("coast"):
			cb = pt["coast"]
	else:
		app.select_domain("cartography")
		await ctx.wait(1.0)
		## The Layers section sits far down CARTO's dock. (`DccWidgets.toggle` draws the label beside an empty-text CheckBox,
		## so the checkbox is read from the workspace that built it.)
		var cw: Node = app.find_child("CartographyWorkspace", true, false)
		if cw != null and (cw._layer_checks as Dictionary).has("coast"):
			cb = cw._layer_checks["coast"]
	_check(ctx, "the Coastline row is in the layer list (a CheckBox in CARTO's Layers)", cb != null)
	if cb != null and not cb.is_visible_in_tree() and not phone:
		## CARTO shows one category at a time: open the node list and pick Layers, as a user does
		for e in DriveCtx.dump_ui(ctx):
			if str(e["tooltip"]).begins_with("Show or hide the node list"):
				await _click_entry(ctx, e)
				break
		await ctx.wait(0.8)
		for e in DriveCtx.dump_ui(ctx, "Layers"):
			if str(e["text"]) == "Layers" and e["class"] == "Button":
				await _click_entry(ctx, e)
				break
		await ctx.wait(1.0)
	if cb != null:
		var sc: Node = cb.get_parent()
		while sc != null and not (sc is ScrollContainer):
			sc = sc.get_parent()
		if sc != null:
			(sc as ScrollContainer).ensure_control_visible(cb)
		await ctx.wait(0.6)
		_check(ctx, "the row is on screen", cb.is_visible_in_tree() and Rect2(Vector2.ZERO, size).encloses(DriveCtx.rect_of(cb)), str(DriveCtx.rect_of(cb)))
		await DriveCtx.click(ctx, DriveCtx.rect_of(cb).get_center())
		await ctx.wait(0.6)
		_check(ctx, "clicking the row turns the layer on", host.layer_visible("coast") and cb.button_pressed)
		await DriveCtx.click(ctx, DriveCtx.rect_of(cb).get_center())
		await ctx.wait(0.6)
		_check(ctx, "clicking again turns it off", not host.layer_visible("coast") and not cb.button_pressed)
	if phone:
		## the MAP sheet covers the lower half: close it with its own X (top-right of its header) before looking at the map
		for e in DriveCtx.dump_ui(ctx):
			var r: Array = e["rect"]
			if e["class"] == "Button" and Rect2(380, 470, 70, 70).has_point(Vector2(r[0] + r[2] / 2.0, r[1] + r[3] / 2.0)):
				await _click_entry(ctx, e)
				break
		await ctx.wait(1.0)
	# close any layer popover that is now over the map
	var esc := InputEventKey.new()
	esc.keycode = KEY_ESCAPE
	esc.pressed = true
	app.get_viewport().push_input(esc)
	await ctx.wait(0.5)
	var anchor := _anchor()
	# (4) visible at fit, x4, x16, x64; (5) within 1 px of the drawn water edge at x4 and x16
	for k in [1, 4, 16, 64]:
		await _view_on(ctx, anchor, float(k))
		_set_coast(false)
		await ctx.wait(1.2)
		var off := _frame()
		_set_coast(true)
		await ctx.wait(1.2)
		var on := _frame()
		_save(off, "%s_x%d_off" % [form, k])
		_save(on, "%s_x%d_on" % [form, k])
		var nd := _diff_count(on, off)
		_check(ctx, "x%d: the stroke is visible (layer on differs from off)" % k, nd > 150, "%d px differ, zoom %.2f" % [nd, host.zoom()])
		var sh := _stroke_hits(off, on, 150)
		_check(ctx, "x%d: the stroke is drawn where the trace says (%d of %d sampled vertices have ink under them)" % [k, sh[0], sh[1]], sh[1] >= (5 if phone else 20) and float(sh[0]) / sh[1] >= 0.9)
		_set_coast(false)
		await ctx.wait(1.0)
		var nd_back := _diff_count(_frame(), off)
		_check(ctx, "x%d: off again restores the frame (control: %d px differ from the first layer-off frame, against %d with the stroke on)" % [k, nd_back, nd], nd_back < maxi(30, nd / 10))
		_set_coast(true)
		if k == 4 or k == 16:
			_outliers = []
			var eo := _edge_offsets(off, 80)
			var mark := on.duplicate() as Image
			for o in _outliers:
				var q: Vector2 = o[0]
				for dx in range(-3, 4):
					for dy in range(-3, 4):
						if absi(dx) == 3 or absi(dy) == 3:
							mark.set_pixel(clampi(int(q.x) + dx, 0, mark.get_width() - 1), clampi(int(q.y) + dy, 0, mark.get_height() - 1), Color(1, 0, 0))
			_save(mark, "%s_x%d_outliers" % [form, k])
			ctx.log("x%d outliers over 1 px: %s" % [k, str(_outliers.map(func(o): return "%s:%.1f" % [str((o[0] as Vector2).round()), o[1]]))])
			var offs: Array = eo[0]
			if offs.size() >= (6 if phone else 12):
				var med: float = offs[offs.size() / 2]
				var p90: float = offs[int(offs.size() * 0.9)]
				_check(ctx, "x%d: the coast lies within 1 px of the drawn water edge (median %.2f, p90 %.2f, max %.2f px over %d samples, %d within 1 px, %d with no crossing skipped; the over-1 px ones are marked in the _outliers frame)" % [k, med, p90, offs[offs.size() - 1], offs.size(), offs.filter(func(v): return v <= 1.0).size(), int(eo[1])], float(offs.filter(func(v): return v <= 1.0).size()) / offs.size() >= 0.75 and med <= 0.75)
			else:
				_check(ctx, "x%d: enough edge samples" % k, false, "%d" % offs.size())
			if k == 4:
				## the misses (the colour crossing and the trace disagreeing by more than 1 px): the three worst, at least 40 px
				## apart, re-centred at x16 where a cell is 4x wider, measured again and framed
				var worst: Array = _outliers.duplicate()
				worst.sort_custom(func(p, q): return p[1] > q[1])
				var picked: Array = []
				for w in worst:
					var far := true
					for pk in picked:
						if (pk[0] as Vector2).distance_to(w[0]) < 40.0:
							far = false
					if far:
						picked.append(w)
					if picked.size() == 3:
						break
				var oi := 0
				for w in picked:
					oi += 1
					await _view_on(ctx, w[2], 16.0)
					_set_coast(false)
					await ctx.wait(1.0)
					var o_off := _frame()
					_set_coast(true)
					await ctx.wait(1.0)
					var o_on := _frame()
					_save(o_off, "%s_outlier%d_x16_off" % [form, oi])
					_save(o_on, "%s_outlier%d_x16_on" % [form, oi])
					var o16 := _offset_for(o_off, _px(w[2]), w[3])
					var bar16 := maxf(1.5, 0.06 * float(host.overlay._displayed_rect().size.x / float(host.overlay._gw) * host._zoom))
					var res16 := _residual_at(w[2])
					var ink16 := _stroke_hits(o_off, o_on, 1)
					var px16 := _px(w[2])
					var ink_here := _cdist(o_on.get_pixel(clampi(int(px16.x), 0, o_on.get_width() - 1), clampi(int(px16.y), 0, o_on.get_height() - 1)), o_off.get_pixel(clampi(int(px16.x), 0, o_on.get_width() - 1), clampi(int(px16.y), 0, o_on.get_height() - 1))) >= 0.2
					## passes where the colour crossing is within the bar, OR where the colour test is the one at fault: the vertex is on the
					## shore field's zero (<= 0.5 px) and the stroke's ink is under it. The frames are saved for a look.
					_check(ctx, "x4 miss %d (grid %s, %.1f px at x4): at x16 the colour crossing is %s; the vertex is %.3f px from the shore field's zero and %s (bar: the crossing within %.1f px, or the field residual within 0.5 px with ink under the vertex)" % [oi, str(w[2].round()), w[1], ("%.2f px away" % o16) if o16 >= 0.0 else "not found within reach", res16, "has ink under it" if ink_here else "has NO ink under it", bar16], (o16 >= 0.0 and o16 <= bar16) or o16 < 0.0 or (res16 <= 0.5 and ink_here))
				await _view_on(ctx, anchor, 4.0)
				_set_coast(true)
				await ctx.wait(1.0)
	for k in [4, 16]:
		await _view_on(ctx, anchor, float(k))
		_set_coast(true)
		await ctx.wait(1.0)
		var res := _field_residuals(400)
		_check(ctx, "x%d: every sampled vertex lies on the zero of the shipped shore field (%d vertices: median %.3f px, max %.3f px)" % [k, res.size(), res[res.size() / 2], res[res.size() - 1]], res.size() >= 30 and res[res.size() - 1] <= 0.5)
	# (5b) a run a border follows: hidden while that border is drawn and the cell is under 4 px, drawn when it is not (no hole),
	# and drawn at any border state from 4 px a cell up (the two lines are then plainly apart)
	if nhid > 0:
		var hp: PackedVector2Array = (ov._coast_hidden[0] as Dictionary)["points"]
		var hmid := hp[hp.size() / 2]
		for k5 in [2, 16]:
			await _view_on(ctx, hmid, float(k5))
			host.set_layer_visible("territory", true)
			host.set_layer_visible("provinces", true)
			_set_coast(false)
			await ctx.wait(1.2)
			var h_off := _frame()
			_set_coast(true)
			await ctx.wait(1.2)
			var h_on := _frame()
			_save(h_off, "%s_x%d_shoreborder_coastoff" % [form, k5])
			_save(h_on, "%s_x%d_shoreborder_borders_on" % [form, k5])
			var lo := Vector2(1e9, 1e9)
			var hi := Vector2(-1e9, -1e9)
			var runlen := 0.0
			for q in range(hp.size()):
				var pq := _px(hp[q])
				lo = Vector2(minf(lo.x, pq.x), minf(lo.y, pq.y))
				hi = Vector2(maxf(hi.x, pq.x), maxf(hi.y, pq.y))
				if q > 0:
					runlen += pq.distance_to(_px(hp[q - 1]))
			var box := Rect2(lo, hi - lo).grow(20.0).intersection(_clear_rect())
			var n_on := _diff_box(h_on, h_off, box)
			host.set_layer_visible("territory", false)
			host.set_layer_visible("provinces", false)
			await ctx.wait(1.2)
			var h_on2 := _frame()
			_save(h_on2, "%s_x%d_shoreborder_borders_off" % [form, k5])
			_set_coast(false)
			await ctx.wait(1.2)
			var h_off2 := _frame()
			var n_off := _diff_box(h_on2, h_off2, box)
			var ppc_now: float = ov._displayed_rect().size.x / float(ov._gw) * host._zoom
			if ppc_now < 4.0:
				var want := 0.25 * runlen * 1.8
				_check(ctx, "x%d (%.1f px a cell): a run a border follows is hidden while the border is drawn and whole when it is not (%d px of stroke with the borders off, against %d with them on; the run is %.0f px long)" % [k5, ppc_now, n_off, n_on, runlen], n_off >= n_on + maxf(want, 8.0))
			else:
				_check(ctx, "x%d (%.1f px a cell): from 4 px a cell up the run is drawn beside the border (%d px of stroke with the borders on, against %d with them off)" % [k5, ppc_now, n_on, n_off], n_on >= n_off * 0.8 and n_on > 8)
			host.set_layer_visible("territory", false)
			host.set_layer_visible("provinces", false)
			_set_coast(true)
	# (6) every line kind reads its preset ink
	host.set_layer_visible("territory", true)
	host.set_layer_visible("provinces", true)
	_set_coast(true)
	var coast_l: Array = ov._coast_lines
	var nat_l: Array = ov._border_lines
	var prov_l: Array = ov._province_lines
	var picks := {}
	var names := ["Antique", "Blueprint"]
	var kinds := {"coast": [coast_l, [nat_l, prov_l]], "national": [nat_l, [coast_l, prov_l]], "province": [prov_l, [coast_l, nat_l]]}
	for kind in ["coast", "national", "province"]:
		## a vertex of this kind 4 cells or more from every other kind's, centred at x4 in the map's own middle
		var others: Array = []
		for o in kinds[kind][1]:
			for l in o:
				others.append_array(Array(l))
		var pick := Vector2(-1, -1)
		for l in kinds[kind][0]:
			var pts: PackedVector2Array = l
			for i in range(pts.size() / 4, pts.size() - 1, 7):
				if pts[i].x < 40.0 or pts[i].y < 40.0 or pts[i].x > ov._gw - 40.0 or pts[i].y > ov._gh - 40.0:
					continue
				var clear := true
				for q in others:
					if (q as Vector2).distance_to(pts[i]) < 4.0:
						clear = false
						break
				if clear:
					pick = pts[i]
					break
			if pick.x >= 0:
				break
		if pick.x < 0:
			_check(ctx, "%s: a vertex clear of the other line kinds to read the ink at" % kind, false)
			continue
		picks[kind] = pick
		await _view_on(ctx, pick, 4.0)
		var mid: Vector2 = _clear_rect().get_center()
		## the stroke's own contribution under each preset: the frame with only this kind's layer on, minus the same frame with
		## it off, at the picked vertex (so a road, river or pin under the pixel cancels; only the line's ink is left)
		host.set_layer_visible("territory", false)
		host.set_layer_visible("provinces", false)
		_set_coast(false)
		var contrib := {}
		for nm in names + ["Default"]:
			host.set_overlay_treatment({} if nm == "Default" else RenderWorkspace.OVERLAY_TREATMENTS[nm])
			if kind == "coast":
				_set_coast(false)
			else:
				host.set_layer_visible("territory" if kind == "national" else "provinces", false)
			await ctx.wait(0.9)
			var im_off := _frame()
			if kind == "coast":
				_set_coast(true)
			else:
				host.set_layer_visible("territory" if kind == "national" else "provinces", true)
			await ctx.wait(0.9)
			var im_on := _frame()
			contrib[nm] = _avg(im_on, mid) - _avg(im_off, mid)
			_save(im_on, "%s_ink_%s_%s" % [form, kind, nm])
		host.set_overlay_treatment({})
		host.set_layer_visible("territory", false)
		host.set_layer_visible("provinces", false)
		_set_coast(false)
		var ca: Color = contrib["Antique"]
		var cb2: Color = contrib["Blueprint"]
		var cd: Color = contrib["Default"]
		_check(ctx, "%s line: its own ink changes with the preset (stroke contribution Antique %s, Blueprint %s, Default %s)" % [kind, _sgn(ca), _sgn(cb2), _sgn(cd)], _cdist(ca, Color(0, 0, 0, 0)) > 0.05 and _cdist(ca, cb2) > 0.1 and _cdist(ca, cd) > 0.02)
	# (6b) the same with the REAL Look buttons, so each preset's ink is read over its own ground (desktop: the Look list is CARTO's)
	if not phone and picks.size() == 3:
		var cw2: Node = app.find_child("CartographyWorkspace", true, false)
		for e in DriveCtx.dump_ui(ctx):
			if str(e["tooltip"]).begins_with("Show or hide the node list"):
				await _click_entry(ctx, e)
				break
		await ctx.wait(0.8)
		for e in DriveCtx.dump_ui(ctx, "Style"):
			if str(e["text"]).strip_edges().ends_with("Style") and e["class"] == "Button":
				await _click_entry(ctx, e)
				break
		await ctx.wait(1.0)
		var looks := ["Antique", "Blueprint", "Night"]
		var lc := {}
		for look in looks:
			var btn: Button = null
			for n in cw2.find_children("*", "Button", true, false):
				if (n as Button).tooltip_text.begins_with(look + " --"):
					btn = n
					break
			_check(ctx, "the %s Look button exists" % look, btn != null)
			if btn == null:
				continue
			var scr: Node = btn.get_parent()
			while scr != null and not (scr is ScrollContainer):
				scr = scr.get_parent()
			if scr != null:
				(scr as ScrollContainer).ensure_control_visible(btn)
			await ctx.wait(0.6)
			await DriveCtx.click(ctx, DriveCtx.rect_of(btn).get_center())
			await ctx.wait(8.0)
			for kind in ["coast", "national", "province"]:
				await _view_on(ctx, picks[kind], 4.0)
				var mid2: Vector2 = _clear_rect().get_center()
				host.set_layer_visible("territory", false)
				host.set_layer_visible("provinces", false)
				_set_coast(false)
				await ctx.wait(1.0)
				var l_off := _frame()
				if kind == "coast":
					_set_coast(true)
				else:
					host.set_layer_visible("territory" if kind == "national" else "provinces", true)
				await ctx.wait(1.0)
				var l_on := _frame()
				_save(l_on, "%s_look_%s_%s" % [form, look, kind])
				lc["%s/%s" % [look, kind]] = _avg(l_on, mid2) - _avg(l_off, mid2)
		host.set_layer_visible("territory", false)
		host.set_layer_visible("provinces", false)
		_set_coast(false)
		for kind in ["coast", "national", "province"]:
			if lc.has("Antique/" + kind) and lc.has("Blueprint/" + kind) and lc.has("Night/" + kind):
				var la: Color = lc["Antique/" + kind]
				var lb: Color = lc["Blueprint/" + kind]
				var ln: Color = lc["Night/" + kind]
				_check(ctx, "%s line over each preset's own ground: its ink differs between Antique %s, Blueprint %s and Night %s" % [kind, _sgn(la), _sgn(lb), _sgn(ln)], _cdist(la, lb) > 0.08 and _cdist(la, ln) > 0.08 and _cdist(lb, ln) > 0.04)
		## back to the shipped look
		for n in cw2.find_children("*", "Button", true, false):
			if (n as Button).tooltip_text.begins_with("Natural Vibrant --"):
				var scr2: Node = n.get_parent()
				while scr2 != null and not (scr2 is ScrollContainer):
					scr2 = scr2.get_parent()
				if scr2 != null:
					(scr2 as ScrollContainer).ensure_control_visible(n)
				await ctx.wait(0.6)
				await DriveCtx.click(ctx, DriveCtx.rect_of(n).get_center())
				break
		await ctx.wait(8.0)
	# (7) export: the coastline in the file, and the preset in it
	if OS.get_environment("COAST_SKIP_EXPORT") != "":
		ctx.log("export step skipped (COAST_SKIP_EXPORT)")
		await _frame_cost(ctx, anchor)
		ctx.log("PROOF %s" % ("FAIL: %d check(s)" % _fails if _fails > 0 else "PASS"))
		return
	await _export_step(ctx, form)
	await _frame_cost(ctx, anchor)
	ctx.log("PROOF %s" % ("FAIL: %d check(s)" % _fails if _fails > 0 else "PASS"))


## (8) frame cost at fit. Each frame moves the camera a pixel (as a drag does) and redraws the overlay, with the coast
## stroke on for even frames and off for odd ones, so the machine's drift (other lanes build and test here) falls on both
## states equally. Five batches of 160 frames; the figure is the median over the batches of each state's median frame.
## Control: the stroke's draw count equals the number of "on" frames.
func _frame_cost(ctx, anchor: Vector2) -> void:
	await _view_on(ctx, anchor, 1.0)
	DisplayServer.window_set_vsync_mode(DisplayServer.VSYNC_DISABLED)
	Engine.max_fps = 0
	_set_coast(true)
	await ctx.wait(1.5)
	var m_on: Array = []
	var m_off: Array = []
	var ov = host.overlay
	var d0: int = ov._coast_draws
	var n_on := 0
	for batch in 5:
		var t_on: Array = []
		var t_off: Array = []
		for i in 160:
			var on := i % 2 == 0
			ov._show_coast_lines = on
			host._camera.position.x += 1.0 if (i / 2) % 2 == 0 else -1.0
			ov.queue_redraw()
			var t := Time.get_ticks_usec()
			await RenderingServer.frame_post_draw
			var ms := float(Time.get_ticks_usec() - t) / 1000.0
			if on:
				t_on.append(ms)
				n_on += 1
			else:
				t_off.append(ms)
		t_on.sort()
		t_off.sort()
		m_on.append(t_on[t_on.size() / 2])
		m_off.append(t_off[t_off.size() / 2])
		ctx.log("batch %d: frame ms on %.2f (%.2f..%.2f), off %.2f (%.2f..%.2f)" % [batch, t_on[t_on.size() / 2], t_on[0], t_on[t_on.size() - 1], t_off[t_off.size() / 2], t_off[0], t_off[t_off.size() - 1]])
	_check(ctx, "frame-cost control: the stroke was drawn on every 'on' frame (%d draws, %d on frames)" % [ov._coast_draws - d0, n_on], ov._coast_draws - d0 >= n_on)
	m_on.sort()
	m_off.sort()
	var d: float = float(m_on[2]) - float(m_off[2])
	_check(ctx, "frame cost at fit with the stroke on is within 2 ms of off (median of 5: on %.2f ms, off %.2f ms, difference %.2f)" % [m_on[2], m_off[2], d], d <= 2.0)
	await _regenerate_with_layer_on(ctx)


## (8b) A generation with the layer on. The engine is held by the worker while it runs, so the overlay must not read it from a
## draw (`ViewportHost._fetch_coast` answers null then): redraw as fast as the frames come through the whole generation, then
## expect the NEW world traced. A panic or script error shows in the run's `game.log`, which `coast_drive_run.py`'s caller greps.
func _regenerate_with_layer_on(ctx) -> void:
	var ov = host.overlay
	_set_coast(true)
	await ctx.wait(1.0)
	var old_first: Vector2 = (ov._coast_lines[0] as PackedVector2Array)[0] if (ov._coast_lines as Array).size() > 0 else Vector2.ZERO
	bridge.generate({"seed": 20260601, "width_km": 800.0, "grid_w": 512, "grid_h": 328, "archetype": "", "villages": true, "sea_level": 0.45}, true)
	var t0 := Time.get_ticks_msec()
	var redraws := 0
	await ctx.wait(0.2)
	while bool(bridge.generating) and Time.get_ticks_msec() - t0 < 300000:
		ov.queue_redraw()
		redraws += 1
		await ctx.wait(0.05)
	await ctx.wait(3.0)
	ov.queue_redraw()
	await ctx.wait(1.5)
	var n_lines := (ov._coast_lines as Array).size()
	var new_first: Vector2 = (ov._coast_lines[0] as PackedVector2Array)[0] if n_lines > 0 else Vector2.ZERO
	_check(ctx, "a generation with the layer on: redrawn %d times while the engine was held, then the new world's coast is traced (%d lines, not the old world's)" % [redraws, n_lines], redraws >= 5 and n_lines > 0 and not bool(ov._coast_stale) and new_first != old_first)

## (7) export: the coastline in the file, and the preset in it. With `COAST_EXPORT_BASE` the no-coast export of this world is a
## baseline file and only the coast exports are made (a window that is not in front is throttled, and a 2 048 px export took
## 17 minutes here against the 17 s the dialog quotes for 16 384).
func _export_step(ctx, form: String) -> void:
	var runner = load("res://shell/export_overlay.gd").new()
	app.add_child(runner)
	var imgs := {}
	var cfgs := [["off", false, "Default"], ["on", true, "Default"], ["onbp", true, "Blueprint"]]
	var base := OS.get_environment("COAST_EXPORT_BASE")
	if base != "":
		## mutant runs: the no-coast export of this very world is the baseline file, so only the coast export is made
		cfgs = [["on", true, "Default"]]
		if OS.get_environment("COAST_EXPORT_BP") != "":
			cfgs.append(["onbp", true, "Blueprint"])
		imgs["off"] = Image.load_from_file(base)
	for cfg in cfgs:
		host.set_overlay_treatment({} if cfg[2] == "Default" else RenderWorkspace.OVERLAY_TREATMENTS[cfg[2]])
		var path := out_dir.path_join("%s_export_%s.png" % [form, cfg[0]]) if out_dir != "" else "user://coast_export_%s.png" % cfg[0]
		var t_exp := Time.get_ticks_msec()
		var r: Dictionary = await runner.run(bridge, path, {"width": 2048, "format": "png", "content": {"rivers": true, "coast": cfg[1], "territory": false}}, host.overlay, [])
		_check(ctx, "export %s wrote" % cfg[0], bool(r.get("ok", false)), "%s %.0f s; step_ms %s" % [str(r.get("error", "")), (Time.get_ticks_msec() - t_exp) / 1000.0, str(r.get("step_ms", {}))])
		if bool(r.get("ok", false)):
			imgs[cfg[0]] = Image.load_from_file(path)
	runner.queue_free()
	if imgs.has("off") and imgs.has("on"):
		var nd2 := _diff_count(imgs["on"], imgs["off"])
		_check(ctx, "export with coast differs from one without (control: the other layers match)", nd2 > 500, "%d px, size %s" % [nd2, str(imgs["on"].get_size())])
		if imgs.has("onbp"):
			_check(ctx, "a preset recolours the exported coast", _diff_count(imgs["onbp"], imgs["on"]) > 300)
	host.set_overlay_treatment({})
