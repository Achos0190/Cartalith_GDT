extends RefCounted
## TOWN_LAYOUT_SAVE_SCOPE.md TL-3/TL-6 (Rulings GZ and HA, 2026-11-22): frozen towns, driven through the REAL shell on one form.
## Run once per form, each from its own `--root` (own empty APPDATA):
##   python -I cartalith-native/tools/fennara_drive.py GODOT --shell desktop|tablet|phone res://tools/town_save_drive.gd --dll DLL --out DIR --root C:/cw/<root> --timeout 420
## Steps, each printing `ok` or `FAIL` (the launcher exits 1 on any FAIL):
##  1. a world (seed 4242, 256 x 192); the largest laid-out town is chosen and opened in the City Viewer; it is NOT frozen
##     (the positive control: no badge, the section offers Freeze this town);
##  2. a POI and a label are placed in it through the engine calls the CARTO tools make (`poi_add`, `label_create`); the
##     town is now frozen, the badge reads "Frozen", and undo history holds one step per mark;
##  3. the world is saved and reopened: the town's layout (building count and the first building's outline) is identical
##     and it is still frozen;
##  4. `urban_parallel_streets` is flipped on: the frozen town's layout is unchanged and the banner names the switch;
##  5. the banner's Re-lay is clicked (a real click): the before/after opens; Show before / Show after flip the plan (both
##     frames captured); Apply is clicked: the report line shows, the POI keeps its id and is anchored to the new town;
##  6. on a touch form every new control is at least 44 px tall in the window's own coordinates (44 dp: the scale is 1);
##     the banner's and badge's text colour against the panel is computed (WCAG ratio, at least 4.5);
##  7. TL-7: a collapse simulation records five years; scrubbing them, the frozen town never re-lays (same layout), its
##     anchors stay, and the chip is on exactly when the frozen and this year's populations differ by 2x or more.
##  TL-1's visual check (every form): the City Viewer's plan at zoom x1, x4 and x12 (the viewer's ceiling, `ZOOM_MAX`) is
##     captured before the marks (generated) and after (frozen), and the canvas pixels are diffed: the frozen town must
##     the quantum's 0.05 m must be under one screen pixel at every zoom (scope §4.3's criterion), and the visible
##     pixel change (a channel past 64/255) is reported and bounded at 5 % of the canvas, against the re-laid town (the
##     positive control) at over 20 %.
## Frames: `<out>/*.png`, one per step.

const SCRATCH := "C:/cw/tl_town_scratch"
var _fails := 0
var app: Node
var bridge
var _form := "desktop"
## The town as generated, before any mark froze it.
var _gen_layout: Dictionary = {}
## The viewer canvas at each zoom, keyed `"<tag>_<zoom>"`.
var _zoom_imgs := {}
## Canvas pixels per metre at each zoom (`_view_transform().scale`).
var _zoom_scale := {}
const ZOOMS := [1.0, 4.0, 12.0]


## The City Viewer's canvas at each of `ZOOMS` (centred on the fit view), stored under `tag`, a frame captured for each.
func _zoom_shots(ctx, tag: String) -> void:
	var cv = app.city_viewer_window
	for z in ZOOMS:
		cv._zoom = z
		cv._pan = Vector2.ZERO
		cv._canvas.queue_redraw()
		await ctx.wait(0.6)
		var img: Image = app.get_viewport().get_texture().get_image()
		var r: Rect2 = DriveCtx.rect_of(cv._canvas)
		var vp_scale: float = float(img.get_width()) / app.get_viewport().get_visible_rect().size.x
		var rr := Rect2i(Vector2i(r.position * vp_scale), Vector2i(r.size * vp_scale)).intersection(Rect2i(0, 0, img.get_width(), img.get_height()))
		_zoom_imgs["%s_%d" % [tag, int(z)]] = img.get_region(rr)
		_zoom_scale["%d" % int(z)] = float(cv._view_transform()["scale"])
		## The measured canvas itself, saved (a `ctx.capture` frame is taken asynchronously and may show a later zoom).
		(_zoom_imgs["%s_%d" % [tag, int(z)]] as Image).save_png(SCRATCH.path_join("zoom_%s_%s_x%d.png" % [_form, tag, int(z)]))
		ctx.capture("z_%s_%s_x%d" % [_form, tag, int(z)])
	cv._zoom = 1.0
	cv._canvas.queue_redraw()


## Share of pixels (every second one each way, over the two images' common top-left area) whose any channel differs by
## more than `tol` (0..1). A visible change is `tol` 0.25 (64/255): the 0.05 m quantum moves an edge by well under a
## pixel at the viewer's zooms, which flips anti-aliasing by a few levels only (measured 2026-11-22: 5-6 % of pixels
## past 8/255 at x1 and x4, 0.07-0.25 % past 64/255, those being the new pin and label).
func _diff_share(a: Image, b: Image, tol := 0.25) -> float:
	if a == null or b == null:
		return 1.0
	var w := mini(a.get_width(), b.get_width())
	var h := mini(a.get_height(), b.get_height())
	var n := 0
	for y in range(0, h, 2):
		for x in range(0, w, 2):
			var p := a.get_pixel(x, y)
			var q := b.get_pixel(x, y)
			if absf(p.r - q.r) > tol or absf(p.g - q.g) > tol or absf(p.b - q.b) > tol:
				n += 1
	return float(n) / float(maxi(1, ((w + 1) / 2) * ((h + 1) / 2)))


func _check(ctx, what: String, cond: bool, detail: String = "") -> void:
	if not cond:
		_fails += 1
	ctx.log("%s  %s%s" % ["ok  " if cond else "FAIL", what, ("  -- " + detail) if detail != "" else ""])


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


## The visible control named `n` under the City Viewer, or null.
func _named(n: String) -> Control:
	var c := app.city_viewer_window.find_child(n, true, false) as Control
	return c if c != null and c.is_visible_in_tree() else null


func _click_named(ctx, n: String) -> bool:
	var c := _named(n)
	if c == null:
		return false
	var r := DriveCtx.rect_of(c)
	await DriveCtx.click(ctx, r.get_center())
	await ctx.wait(0.8)
	return true


func _past_welcome(ctx) -> void:
	var size: Vector2 = ctx.get_scene_root().get_viewport().get_visible_rect().size
	if size.x < 700.0:
		var nw := await _wait_for(ctx, "+ NEW WORLD")
		if not nw.is_empty():
			await _click_entry(ctx, nw)
			var cancel := await _wait_for(ctx, "CANCEL")
			if not cancel.is_empty():
				await _click_entry(ctx, cancel)
		await ctx.wait(1.0)
		return
	var cont := await _wait_for(ctx, "Continue without a world")
	if not cont.is_empty():
		await _click_entry(ctx, cont)
		await ctx.wait(0.6)


func _generate(ctx) -> void:
	bridge.generate({"seed": 4242, "width_km": 800.0, "grid_w": 256, "grid_h": 192, "archetype": "", "villages": true, "sea_level": 0.45})
	await ctx.wait(0.2)
	var t0 := Time.get_ticks_msec()
	while bool(bridge.generating) and Time.get_ticks_msec() - t0 < 300000:
		await ctx.wait(0.25)
	await ctx.wait(2.0)


## A compact fingerprint of a served layout: building count, edge count and the first building's outline.
func _sig(l: Dictionary) -> String:
	var b: Array = l.get("buildings", [])
	return "%d|%d|%s" % [b.size(), int(l.get("edge_count", -1)), str(b[0]) if not b.is_empty() else ""]


func _layout(i: int) -> Dictionary:
	var got: Array = bridge.urban_layouts(PackedInt32Array([i]))
	return got[0] if not got.is_empty() else {}


func run(ctx) -> void:
	app = ctx.get_scene_root()
	if not ("bridge" in app):
		app = ctx.get_scene_root().find_child("App", true, false)
	bridge = app.bridge
	var size: Vector2 = app.get_viewport().get_visible_rect().size
	_form = "phone" if size.x < 700.0 else ("tablet" if bool(DccTheme.is_tablet()) else "desktop")
	ctx.log("form %s, window %s" % [_form, str(size)])
	DirAccess.make_dir_recursive_absolute(SCRATCH)
	await _past_welcome(ctx)
	await _generate(ctx)
	_check(ctx, "a world exists", bool(bridge.has_world))

	# 1. the largest laid-out town (a town has gates and many buildings: a real re-lay)
	var places: Array = bridge.settlements()
	var best := -1
	var best_n := 0
	for i in places.size():
		var l := _layout(i)
		var n: int = (l.get("buildings", []) as Array).size()
		if n > best_n:
			best_n = n
			best = i
	_check(ctx, "a town lays out", best >= 0 and best_n > 50, "index %d, %d buildings" % [best, best_n])
	if best < 0:
		return
	var s: Dictionary = places[best]
	var gx := float(s["x"]) + 0.5
	var gy := float(s["y"]) + 0.5
	_check(ctx, "positive control: the town is not frozen before any mark", not bool(bridge.town_state(best).get("frozen", true)))
	app.open_city_viewer(best)
	await ctx.wait(2.0)
	_check(ctx, "the viewer offers Freeze this town while unfrozen", _named("FreezeTown") != null)
	_check(ctx, "no Frozen badge before any mark", _named("FrozenBadge") == null)
	ctx.capture("1_%s_unfrozen" % _form)
	await _zoom_shots(ctx, "generated")

	# 2. a POI and a label in the town freeze it
	_gen_layout = _layout(best)
	var before_sig := _sig(_gen_layout)
	var cell_km: float = float(bridge.world_crs().get("cell_km", 1.0))
	var off := 0.12 / maxf(cell_km, 0.001)   # 120 m from the cell centre: inside the 850 m town radius
	var poi: int = bridge.poi_add(gx + off, gy, "tavern", "Drive Tavern")
	_check(ctx, "a POI is placed in the town", poi >= 0)
	var lab: int = bridge.label_create(gx - off, gy + off, "Drive Label")
	_check(ctx, "a label is placed in the town", lab >= 0)
	await ctx.wait(1.0)
	var st: Dictionary = bridge.town_state(best)
	_check(ctx, "the town is frozen after the marks", bool(st.get("frozen", false)), str(st))
	_check(ctx, "the POI is anchored to it", int(st.get("anchors", 0)) >= 1, str(st))
	# The frozen town is served from its stored form, quantised at 0.1 m: the same town, every vertex within 0.05 m of the
	# generated one it was frozen from (TL-1's bar), and from here on byte-stable.
	var gen_l := _gen_layout
	var fro_l := _layout(best)
	var worst := 0.0
	var gb: Array = gen_l.get("buildings", [])
	var fb: Array = fro_l.get("buildings", [])
	for k in mini(gb.size(), 50):
		var a: PackedVector2Array = gb[k]
		var b: PackedVector2Array = fb[k]
		for j in mini(a.size(), b.size()):
			worst = maxf(worst, maxf(absf(a[j].x - b[j].x), absf(a[j].y - b[j].y)))
	_check(ctx, "the frozen town is the generated one within 0.05 m (same counts)", gb.size() == fb.size() and int(gen_l.get("edge_count", -1)) == int(fro_l.get("edge_count", -2)) and worst <= 0.0501, "worst %.4f m over 50 buildings" % worst)
	before_sig = _sig(fro_l)
	await _zoom_shots(ctx, "frozen")
	for zz in ZOOMS:
		var sh := _diff_share(_zoom_imgs["generated_%d" % int(zz)], _zoom_imgs["frozen_%d" % int(zz)])
		var aa := _diff_share(_zoom_imgs["generated_%d" % int(zz)], _zoom_imgs["frozen_%d" % int(zz)], 0.032)
		var px := 0.05 * float(_zoom_scale.get("%d" % int(zz), 0.0))
		## Scope §4.3's own criterion: the quantum's worst error (0.05 m, asserted by TL-1's census) is under a pixel at
		## the viewer's deepest zoom. The pixel diff beside it is the observed effect (anti-aliasing levels on edges; the
		## new pin and label), bounded loosely and set against the re-laid town's below.
		_check(ctx, "x%d: 0.05 m is %.2f px on screen (under 1 px); %.2f %% of the canvas changes visibly, %.2f %% by anti-aliasing levels" % [int(zz), px, 100.0 * sh, 100.0 * aa], px > 0.0 and px < 1.0 and sh < 0.05)
	var badge := _named("FrozenBadge")
	_check(ctx, "the viewer shows the Frozen badge", badge != null and String((badge as Label).text).begins_with("Frozen"), "" if badge == null else String((badge as Label).text))
	if _form == "phone" and badge != null:
		# Ruling HL: the badge sits under the title, not in the scrolling side column, so it is on screen without scrolling.
		var in_scroll := false
		var n: Node = badge.get_parent()
		while n != null:
			if n is ScrollContainer:
				in_scroll = true
			n = n.get_parent()
		var br: Rect2 = (badge as Control).get_global_rect()
		_check(ctx, "phone: the Frozen badge is outside any ScrollContainer and visible on screen", not in_scroll and (badge as Control).is_visible_in_tree() and br.position.y >= 0.0 and br.end.y <= size.y and br.size.y > 0.0, "scroll=%s rect=%s" % [str(in_scroll), str(br)])
	ctx.capture("2_%s_frozen" % _form)

	# 3. save and reopen: the same layout, still frozen
	var path := SCRATCH.path_join("town_%s.zip" % _form)
	var r: Dictionary = bridge.project_save_with_documents(path, app._project_documents())
	_check(ctx, "the world saves", bool(r.get("ok", false)), str(r))
	var z := ZIPReader.new()
	var has_slot := z.open(path) == OK and z.get_files().has("entities/town_layouts.json")
	if z.get_files().size() > 0:
		z.close()
	_check(ctx, "the save holds entities/town_layouts.json", has_slot)
	var ok: bool = await app.open_project_busy(path)
	await ctx.wait(3.0)
	var t_end := Time.get_ticks_msec() + 40000
	while not _find(ctx, "OPENING PROJECT").is_empty() and Time.get_ticks_msec() < t_end:
		await ctx.wait(0.5)
	_check(ctx, "the save reopens", ok)
	_check(ctx, "after reopening the town is still frozen", bool(bridge.town_state(best).get("frozen", false)))
	_check(ctx, "after reopening the layout is identical", _sig(_layout(best)) == before_sig, _sig(_layout(best)).substr(0, 40) + " vs " + before_sig.substr(0, 40))
	var re_reasons: Array = Array(bridge.town_state(best, true).get("reasons", PackedStringArray()))
	_check(ctx, "after reopening nothing reports the town as changed (the cheap tier is empty)", re_reasons.is_empty(), str(re_reasons))
	app.open_city_viewer(best)
	await ctx.wait(2.0)
	ctx.capture("3_%s_reopened" % _form)

	# 4. flip a switch: the frozen town stays, the banner says why
	var w: Dictionary = bridge.param_write("urban_parallel_streets", true)
	_check(ctx, "urban_parallel_streets is switched on", bool(w.get("ok", false)), str(w))
	await ctx.wait(2.0)
	_check(ctx, "the frozen town is unchanged after the switch", _sig(_layout(best)) == before_sig)
	var mm := _named("MismatchText")
	_check(ctx, "the banner reports the mismatch by name", mm != null and String((mm as Label).text).contains("parallel streets is now on"), "" if mm == null else String((mm as Label).text))
	ctx.capture("4_%s_mismatch" % _form)

	# 5. Re-lay: before/after, Apply, pins re-attach
	_check(ctx, "the banner's Re-lay is clicked", await _click_named(ctx, "BannerRelay"))
	var pv := _named("PreviewText")
	_check(ctx, "the before/after opens on AFTER", pv != null and String((pv as Label).text).contains("AFTER"), "" if pv == null else String((pv as Label).text))
	ctx.capture("5a_%s_after" % _form)
	_check(ctx, "Show before is clicked", await _click_named(ctx, "PreviewFlip"))
	pv = _named("PreviewText")
	_check(ctx, "the plan now shows BEFORE", pv != null and String((pv as Label).text).contains("BEFORE"))
	_check(ctx, "BEFORE is the frozen town", _sig(app.city_viewer_window._layout) == before_sig)
	ctx.capture("5b_%s_before" % _form)
	if _form != "desktop":
		for n in ["PreviewFlip", "PreviewApply", "PreviewCancel"]:
			var c := _named(n)
			_check(ctx, "%s is at least 44 px tall on %s" % [n, _form], c != null and DriveCtx.rect_of(c).size.y >= 44.0 - 0.5, "" if c == null else str(DriveCtx.rect_of(c).size))
	var poi_id := int((bridge.poi_list()[poi] as Dictionary).get("id", -1)) if poi < (bridge.poi_list() as Array).size() else -1
	_check(ctx, "Apply is clicked", await _click_named(ctx, "PreviewApply"))
	await ctx.wait(1.5)
	var rep := _named("RelayReport")
	_check(ctx, "the re-lay report shows", rep != null and String((rep as Label).text).begins_with("Re-laid"), "" if rep == null else String((rep as Label).text))
	var after_sig := _sig(_layout(best))
	_check(ctx, "the town is now drawn by the current rules (a different layout)", after_sig != before_sig)
	var still := false
	for p in bridge.poi_list():
		if int((p as Dictionary).get("id", -2)) == poi_id:
			still = true
	_check(ctx, "the POI kept its id through the re-lay (nothing deleted)", still and poi_id > 0)
	var st2: Dictionary = bridge.town_state(best)
	_check(ctx, "the re-laid town is frozen again with the POI anchored", bool(st2.get("frozen", false)) and int(st2.get("anchors", 0)) >= 1, str(st2))
	ctx.capture("5c_%s_applied" % _form)
	await _zoom_shots(ctx, "relaid")
	for zz in ZOOMS:
		var sh := _diff_share(_zoom_imgs["frozen_%d" % int(zz)], _zoom_imgs["relaid_%d" % int(zz)])
		_check(ctx, "x%d positive control: the re-laid town differs visibly (%.1f %% of the canvas)" % [int(zz), 100.0 * sh], sh > 0.20)
	bridge.param_write("urban_parallel_streets", false)
	await ctx.wait(1.0)

	# 6. contrast of the new text (palette-bound: measured on the palette this run booted with)
	var fg: Color = DccTheme.c("text")
	var bg: Color = DccTheme.c("panel")
	var lum := func(c: Color) -> float:
		var ch := func(v: float) -> float: return v / 12.92 if v <= 0.03928 else pow((v + 0.055) / 1.055, 2.4)
		return 0.2126 * ch.call(c.r) + 0.7152 * ch.call(c.g) + 0.0722 * ch.call(c.b)
	var la: float = lum.call(fg)
	var lb: float = lum.call(bg)
	var ratio: float = (maxf(la, lb) + 0.05) / (minf(la, lb) + 0.05)
	_check(ctx, "banner and badge text contrast at least 4.5:1", ratio >= 4.5, "%.2f (text %s on panel %s)" % [ratio, fg.to_html(false), bg.to_html(false)])
	if _form != "desktop":
		for n in ["RelayButton", "BannerRelay", "BannerKeep"]:
			var c := _named(n)
			if c != null:
				_check(ctx, "%s is at least 44 px tall on %s" % [n, _form], DriveCtx.rect_of(c).size.y >= 44.0 - 0.5, str(DriveCtx.rect_of(c).size))

	# 7. TL-7: scrub five recorded years
	var sig_now := _sig(_layout(best))
	var anchors_now := int(bridge.town_state(best).get("anchors", 0))
	var y0 := int(bridge.get_civ_year())
	var sim: Dictionary = bridge.civ_run_collapse_simulation({"mode": "collapse", "character": "mixed", "severity": 0.9,
		"start_year": y0, "duration": 50, "step_years": 10, "confirm_overwrite": true})
	_check(ctx, "a collapse simulation records the years", bool(sim.get("ok", false)), str(sim).substr(0, 120))
	var chips := 0
	var nochips := 0
	for k in 5:
		var y := y0 + 10 * (k + 1)
		app.tl_set_year(y, true)
		await ctx.wait(0.8)
		var stk: Dictionary = bridge.town_state(best)
		var p0 := float(stk.get("pop", 0))
		var p1 := float(stk.get("pop_now", 0))
		var want := p0 > 0.0 and p1 > 0.0 and (p0 / p1 >= 2.0 or p1 / p0 >= 2.0)
		if bool(stk.get("chip", false)):
			chips += 1
		else:
			nochips += 1
		_check(ctx, "year %d: the frozen town is not re-laid" % y, _sig(_layout(best)) == sig_now)
		_check(ctx, "year %d: its anchors stay (%d)" % [y, anchors_now], int(stk.get("anchors", -1)) == anchors_now)
		_check(ctx, "year %d: chip %s at pop %d frozen vs %d now" % [y, str(stk.get("chip", false)), int(p0), int(p1)], bool(stk.get("chip", false)) == want)
	ctx.log("chip on in %d of 5 years, off in %d" % [chips, nochips])
	ctx.capture("7_%s_year_scrub" % _form)
	ctx.log("RESULT %s" % ("FAIL: %d check(s)" % _fails if _fails > 0 else "PASS"))
