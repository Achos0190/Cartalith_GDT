extends RefCounted
## Ruling HV's leftover (OUTSTANDING_WORK): on a world whose provinces do not follow the terrain (`civ.border_friction` 0) the RASTER
## province line (`build_province_boundary_texture`) also outlines the coast, beside the Coastline stroke. With the Coastline layer on
## that outline must go, and nothing else of the line may. Real shell, one form per run, windowed (frames are saved to COAST_OUT).
## Bars: (1) positive control: the world has a raster province texture at all; (2) the inland texture has fewer inked cells than the
## plain one, none that the plain one lacks, and every cell it dropped has water within 2 cells (shore field > 0); (3) the province
## lines between provinces survive (inland ink > 0); (4) the layer round trip: coast on -> the live texture IS the inland one, coast
## off -> the plain one again, through `set_layer_visible` (the call the Layers row makes); (5) frames at x4 on a coast with the
## provinces layer on, coast off and coast on, which differ. A FAIL line fails the run. Never writes the owner's data.

var _fails := 0
var app: Node
var bridge
var host


func _check(ctx, what: String, cond: bool, detail: String = "") -> void:
	if not cond:
		_fails += 1
	ctx.log("%s  %s  %s" % ["ok  " if cond else "FAIL", what, detail])


func _find(ctx, t: String) -> Dictionary:
	for e in DriveCtx.dump_ui(ctx, t):
		return e
	return {}


func _wait_for(ctx, t: String, secs: float = 20.0) -> Dictionary:
	var end := Time.get_ticks_msec() + int(secs * 1000)
	while Time.get_ticks_msec() < end:
		var e := _find(ctx, t)
		if not e.is_empty():
			return e
		await ctx.wait(0.25)
	return {}


func _click_entry(ctx, e: Dictionary) -> void:
	var r: Array = e["rect"]
	await DriveCtx.click(ctx, Vector2(r[0] + r[2] / 2.0, r[1] + r[3] / 2.0))


func _inked(img: Image) -> int:
	var n := 0
	for y in img.get_height():
		for x in img.get_width():
			if img.get_pixel(x, y).a > 0.0:
				n += 1
	return n


func run(ctx) -> void:
	app = ctx.get_scene_root()
	if not app.has_method("select_domain"):
		app = ctx.get_scene_root().find_child("App", true, false)
	bridge = app.bridge
	host = app.viewport
	var out_dir := OS.get_environment("COAST_OUT")
	if out_dir != "":
		DirAccess.make_dir_recursive_absolute(out_dir)
	var phone: bool = app.is_phone()
	var form := "phone" if phone else "desktop"
	if phone:
		await _click_entry(ctx, await _wait_for(ctx, "+ NEW WORLD"))
		await _click_entry(ctx, await _wait_for(ctx, "CANCEL"))
	else:
		await _click_entry(ctx, await _wait_for(ctx, "Continue without a world"))
	await ctx.wait(1.0)
	## A generate at another seed (or New World) puts the rules back to the app's (Ruling EV/HY), so: make the world, set friction 0,
	## then regenerate the SAME seed, which keeps the table (the older save's case).
	var req := {"seed": 483920, "width_km": 800.0, "grid_w": 512, "grid_h": 328, "archetype": "", "villages": true, "sea_level": 0.45}
	for pass_i in 2:
		if pass_i == 1:
			bridge.param_set("civ.border_friction", 0.0)
		bridge.generate(req, pass_i == 0)
		await ctx.wait(0.2)
		while bool(bridge.generating):
			await ctx.wait(0.25)
		await ctx.wait(3.0)
	_check(ctx, "friction is 0 (a literal)", is_zero_approx(float(bridge.param_get("civ.border_friction"))))
	var tex: Texture2D = host.province_view.texture
	_check(ctx, "positive control: this world has a raster province texture", tex != null)
	if tex == null:
		ctx.log("RESULT fails=%d" % _fails)
		return
	var plain: Image = bridge.province_boundary_texture(false).get_image()
	var inland: Image = bridge.province_boundary_texture(true).get_image()
	var shore: Image = bridge.shore_field_texture().get_image()
	var pn := _inked(plain)
	var inn := _inked(inland)
	_check(ctx, "the inland texture drops ink (the coast outline)", inn < pn - 50, "plain %d, inland %d inked cells" % [pn, inn])
	_check(ctx, "the province-to-province line survives", inn > 0 and inn * 4 > pn, "inland %d of plain %d" % [inn, pn])
	var added := 0
	var far := 0
	var gw := plain.get_width()
	var gh := plain.get_height()
	for y in gh:
		for x in gw:
			var a_p := plain.get_pixel(x, y).a
			var a_i := inland.get_pixel(x, y).a
			if a_i > 0.0 and a_p <= 0.0:
				added += 1
			if a_p > 0.0 and a_i <= 0.0:
				var wet := false
				for dy in range(-2, 3):
					for dx in range(-2, 3):
						if shore.get_pixel(clampi(x + dx, 0, shore.get_width() - 1), clampi(y + dy, 0, shore.get_height() - 1)).r > 0.0:
							wet = true
				if not wet:
					far += 1
	_check(ctx, "no cell gains ink", added == 0, "%d gained" % added)
	_check(ctx, "every dropped cell has water within 2 cells", far == 0, "%d dropped with no water near" % far)
	host.set_layer_visible("provinces", true)
	host.set_layer_visible("coast", false)
	await ctx.wait(0.8)
	var live_off: Image = host.province_view.texture.get_image()
	_check(ctx, "coast off: the live province texture is the plain one", live_off.get_data() == plain.get_data())
	host.set_layer_visible("coast", true)
	await ctx.wait(0.8)
	var live_on: Image = host.province_view.texture.get_image()
	_check(ctx, "coast on: the live province texture is the inland one", live_on.get_data() == inland.get_data())
	host.refresh()
	await ctx.wait(1.0)
	_check(ctx, "coast on, then a full refresh (a regenerate): the live province texture is still the inland one", (host.province_view.texture.get_image().get_data()) == inland.get_data())
	# frames at x4 about a coastal run
	var coast: Dictionary = bridge.coast_lines()
	var kept: Array = coast.get("kept", [])
	_check(ctx, "the world has a coast trace", kept.size() > 0)
	var anchor := Vector2(gw * 0.5, gh * 0.5)
	var best_len := 0
	for l in kept:
		if (l as PackedVector2Array).size() > best_len:
			best_len = (l as PackedVector2Array).size()
			anchor = (l as PackedVector2Array)[best_len / 2]
	host.reset_view()
	await ctx.wait(0.4)
	var ov = host.overlay
	var rect: Rect2 = ov._displayed_rect()
	var local: Vector2 = rect.position + Vector2(anchor.x / ov._gw, anchor.y / ov._gh) * rect.size
	host._zoom = clampf(host._zoom * 4.0, host.ZOOM_MIN, host._zoom_max)
	host._camera.scale = Vector2(host._zoom, host._zoom)
	host._camera.position = host.size * 0.5 - local * host._zoom
	host._update_zoom_readout()
	host.overlay.set_camera_zoom(host._zoom)
	host._update_lod()
	await ctx.wait(4.0)
	host.set_layer_visible("coast", false)
	await ctx.wait(1.5)
	var fa: Image = app.get_viewport().get_texture().get_image()
	host.set_layer_visible("coast", true)
	await ctx.wait(1.5)
	var fb: Image = app.get_viewport().get_texture().get_image()
	if out_dir != "":
		fa.save_png(out_dir.path_join("%s_raster_coast_off.png" % form))
		fb.save_png(out_dir.path_join("%s_raster_coast_on.png" % form))
	var nd := 0
	for y in range(0, fa.get_height(), 2):
		for x in range(0, fa.get_width(), 2):
			var c1 := fa.get_pixel(x, y)
			var c2 := fb.get_pixel(x, y)
			if absf(c1.r - c2.r) + absf(c1.g - c2.g) + absf(c1.b - c2.b) > 0.08:
				nd += 1
	_check(ctx, "frames: coast on differs from off (the stroke, and no raster outline beside it)", nd > 100, "%d sampled px differ, zoom %.2f" % [nd, host.zoom()])
	ctx.log("RESULT fails=%d" % _fails)
