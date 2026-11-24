extends RefCounted
## Labels cull/zoom census (2026-11-23, owner: "collision culling and adaptive per zoom seem to no longer work on the tab").
## Creates a many-label world through the engine call Create makes, zooms about the map centre with REAL
## `InputEventMagnifyGesture`s (the pinch the viewport reads) to x1, x2, x4, x8, x16 and at each step counts the
## generated labels the overlay would draw (`_label_hidden` false) and, among those inside the view, the pairs whose
## on-screen rectangles (font, tracking, `_label_unit`, rotation: `_draw_labels`' own model) intersect.
## Logs `CENSUS zoom=<z> drawn=<n> inview=<n> overlaps=<n>` and saves a frame per zoom to LBLC_SHOTS.
## Gate: LBLC_MAX_OVERLAP (default 0) at every zoom; a FAIL line fails the run.
## `LBLC_SEED` (default 4242), `LBLC_GRID` (default "512x320").

var _fails := 0
var _app: Node


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


func _poly(ov, lb: Dictionary, rect: Rect2) -> PackedVector2Array:
	var text: String = lb["text"]
	var font: Font = ov._label_font_for(lb)
	var fpx: int = ov._label_font_px(lb, rect)
	var lu: float = ov._label_unit(lb)
	var w: float = font.get_string_size(text, HORIZONTAL_ALIGNMENT_LEFT, -1, fpx).x + absf(float(lb.get("tracking_em", 0.0))) * fpx * maxf(0.0, text.length() - 1.0)
	var h: float = font.get_height(fpx)
	var pos: Vector2 = ov._point_to_screen(Vector2(lb["x"], lb["y"]), rect)
	var th := deg_to_rad(float(lb["angle"]))
	var hw := w * lu / 2.0
	var hh := h * lu / 2.0
	var pts := PackedVector2Array([Vector2(-hw, -hh), Vector2(hw, -hh), Vector2(hw, hh), Vector2(-hw, hh)])
	for i in 4:
		pts[i] = pos + pts[i].rotated(th)
	return pts


func _census(ctx, tag: String) -> Dictionary:
	var ov = _app.viewport.overlay
	ov._ensure_live_label_cull()   ## what the next `_draw_labels()` would do first
	var rect: Rect2 = ov._displayed_rect()
	var interior: Rect2 = ov._interior_rect(rect)
	var polys: Array = []
	var drawn := 0
	var by_class := {}
	for lb in ov._labels:
		if String(lb.get("text", "")) == "" or ov._label_hidden(lb):
			continue
		drawn += 1
		by_class[String(lb.get("class", "?"))] = int(by_class.get(String(lb.get("class", "?")), 0)) + 1
		## In view = the label's whole box meets the view (a big letter-spaced name centred off-screen still inks on it).
		var bx: Rect2 = ov._label_local_box(lb, rect)
		if not bx.intersects(interior) or not bx.intersects(ov._visible_local):
			continue
		polys.append(_poly(ov, lb, rect))
	var top_culled := 0   ## the top-ranked class (continental) is never suppressed by a lower one
	for lb in ov._labels:
		if bool(lb.get("generated", false)) and String(lb.get("class", "")) == "continental" and bool(lb.get(ov.LIVE_CULL_KEY, false)):
			top_culled += 1
	var overlaps := 0
	for i in polys.size():
		for j in range(i + 1, polys.size()):
			var bi := Rect2(polys[i][0], Vector2.ZERO)
			for p in polys[i]:
				bi = bi.expand(p)
			var bj := Rect2(polys[j][0], Vector2.ZERO)
			for p in polys[j]:
				bj = bj.expand(p)
			if not bi.intersects(bj):
				continue
			var inter: Array = Geometry2D.intersect_polygons(polys[i], polys[j])
			for q in inter:
				if absf(_area(q)) > 1.0:
					overlaps += 1
					break
	ctx.log("CENSUS %s zoom=%.2f drawn=%d inview=%d overlaps=%d ppc_live=%.4f classes=%s" % [tag, _app.viewport.zoom(), drawn, polys.size(), overlaps, _app.viewport.label_px_per_cell(), str(by_class)])
	return {"top_culled": top_culled, "overlaps": overlaps, "inview": polys.size(), "drawn": drawn}


func _area(p: PackedVector2Array) -> float:
	var a := 0.0
	for i in p.size():
		var q := p[(i + 1) % p.size()]
		a += p[i].x * q.y - q.x * p[i].y
	return a / 2.0


func _shot(ctx, label: String) -> void:
	var dir := OS.get_environment("LBLC_SHOTS")
	var img: Image = await ctx.frame(4096)
	if img != null and dir != "":
		DirAccess.make_dir_recursive_absolute(dir)
		img.save_png("%s/%s.png" % [dir, label])


func run(ctx) -> void:
	_app = ctx.get_scene_root()
	var seed_v := int(OS.get_environment("LBLC_SEED")) if OS.get_environment("LBLC_SEED") != "" else 4242
	var grid := (OS.get_environment("LBLC_GRID") if OS.get_environment("LBLC_GRID") != "" else "512x320").split("x")
	var tag := OS.get_environment("LBLC_TAG") if OS.get_environment("LBLC_TAG") != "" else "form"
	var max_ov := int(OS.get_environment("LBLC_MAX_OVERLAP")) if OS.get_environment("LBLC_MAX_OVERLAP") != "" else 0
	var size: Vector2 = _app.get_viewport().get_visible_rect().size
	if size.x < 700.0:
		await _click_entry(ctx, await _wait_for(ctx, "+ NEW WORLD"))
		await _click_entry(ctx, await _wait_for(ctx, "CANCEL"))
	else:
		await _click_entry(ctx, await _wait_for(ctx, "Continue without a world"))
	await ctx.wait(1.0)
	_app.bridge.generate({"seed": seed_v, "width_km": 400.0, "grid_w": int(grid[0]), "grid_h": int(grid[1]),
		"archetype": "", "villages": true, "sea_level": 0.45}, true)
	while _app.bridge.generating:
		await _app.get_tree().create_timer(0.25).timeout
	await ctx.wait(2.0)
	ctx.log("form canvas %s seed %d grid %s labels %d" % [str(size), seed_v, str(grid), _app.viewport.overlay._labels.size()])
	var cc := {}
	for lb in _app.viewport.overlay._labels:
		var k := "%s/%s" % [String(lb.get("class", "?")), "gen" if bool(lb.get("generated", false)) else "hand"]
		cc[k] = int(cc.get(k, 0)) + 1
	ctx.log("LABELS by class %s" % str(cc))
	var fails := 0
	var carto = null
	for w in _app._workspaces:
		if (w as Object).get_script().get_global_name() == "CartographyWorkspace":
			carto = w
	var ov = _app.viewport.overlay
	var a_total := 0
	var prev_ok := false
	for target in [1.0, 2.0, 4.0, 8.0, 16.0]:
		var guard := 0
		while absf(_app.viewport.zoom() / (target * _fit_zoom()) - 1.0) > 0.02 and guard < 12:
			## The harness's pinch lands twice (measured: factor 2.0 took 1.6 to 6.4), so the factor sent is the square root.
			var f: float = sqrt(clampf(target * _fit_zoom() / _app.viewport.zoom(), 0.25, 4.0))
			var ev := InputEventMagnifyGesture.new()
			ev.position = _pivot()
			ev.factor = f
			var before: float = _app.viewport.zoom()
			Input.parse_input_event(ev)
			var t0 := Time.get_ticks_msec()
			while is_equal_approx(_app.viewport.zoom(), before) and Time.get_ticks_msec() - t0 < 2000:
				await ctx.wait(0.1)
			if is_equal_approx(_app.viewport.zoom(), before):
				## The pivot sat on a control over the map (a pin, the layers button): the same pinch at the map's centre.
				ctx.log("   (pinch at %s did not zoom; retrying at the map centre)" % str(ev.position))
				ev.position = _app.viewport.global_position + _app.viewport.size * 0.5
				Input.parse_input_event(ev)
				t0 = Time.get_ticks_msec()
				while is_equal_approx(_app.viewport.zoom(), before) and Time.get_ticks_msec() - t0 < 3000:
					await ctx.wait(0.1)
			ctx.log("pinch step target=%.1f f=%.3f zoom %.3f -> %.3f (guard %d)" % [target, f, before, _app.viewport.zoom(), guard])
			guard += 1
		await ctx.wait(1.0)
		var carry_drawn := -1
		if prev_ok:
			var cc2 := _census(ctx, "%s_C_carry_x%d" % [tag, int(target)])   ## no regen: the previous zoom's state, new zoom
			carry_drawn = int(cc2["drawn"])
			if int(cc2["overlaps"]) > max_ov:
				fails += 1
				ctx.log("FAIL overlaps %d after a zoom with no relabel at x%d (cache not following the zoom)" % [cc2["overlaps"], int(target)])
		var a_zoom := 0
		for arm in ["A_head_nolive", "B_pre4fcf0d94", "C_fixed"]:
			if arm == "B_pre4fcf0d94":
				## The generation call as it was before 4fcf0d94: the cull boxed at the LIVE fit.
				ov.set_label_live_cull(false)
				_app.bridge.labels_generate({"cull": {"on": true, "advance_ratio": carto._label_advance_ratio(), "px_per_cell": _app.viewport.label_px_per_cell()}})
				_app.viewport.refresh_annotations()
			else:
				carto._regenerate_labels()   ## the shipped call: fixed reference fit
				ov.set_label_live_cull(arm == "C_fixed")
			await ctx.wait(0.6)
			var c := _census(ctx, "%s_%s_x%d" % [tag, arm, int(target)])
			if arm == "A_head_nolive":
				a_total += int(c["overlaps"])
				a_zoom = int(c["overlaps"])
			if arm != "B_pre4fcf0d94":
				await _shot(ctx, "%s_%s_x%d" % [tag, arm, int(target)])
			if arm == "C_fixed" and carry_drawn >= 0 and carry_drawn != int(c["drawn"]):
				fails += 1
				ctx.log("FAIL the live cull cache did not follow the zoom at x%d: %d drawn carried, %d after a relabel" % [int(target), carry_drawn, int(c["drawn"])])
			if arm == "C_fixed" and int(c["top_culled"]) > 0:
				fails += 1
				ctx.log("FAIL the continental label was live-suppressed at x%d (rank inverted)" % int(target))
			if arm == "C_fixed" and int(c["overlaps"]) > max_ov:
				fails += 1
				ctx.log("FAIL overlaps %d > %d at x%d (fixed arm)" % [c["overlaps"], max_ov, int(target)])

		## D: the panel toggle off must turn the live pass off with the engine's (overlaps back, never 0 while the head arm has some).
		carto._label_cull = false
		carto._regenerate_labels()
		await ctx.wait(0.6)
		var d := _census(ctx, "%s_D_toggle_off_x%d" % [tag, int(target)])
		carto._label_cull = true
		if int(d["overlaps"]) == 0 and a_zoom > 0:
			fails += 1
			ctx.log("FAIL toggle off left 0 overlaps while the head arm had %d at x%d" % [a_zoom, int(target)])
		carto._regenerate_labels()   ## back to the shipped state (toggle on) for the carry census at the next zoom
		await ctx.wait(0.6)
		prev_ok = true
	ctx.log("POSITIVE CONTROL head arm overlaps summed over zooms: %d" % a_total)
	ctx.log("labels cull drive %s: %d failing zoom(s)" % [tag, fails])


## Screen point (viewport-global) over the densest cluster of drawn-at-x16-eligible labels, so a zoom keeps labels in view.
func _pivot() -> Vector2:
	var ov = _app.viewport.overlay
	var rect: Rect2 = ov._displayed_rect()
	var best := Vector2.ZERO
	var best_n := -1
	var pts: Array = []
	for lb in ov._labels:
		if bool(lb.get("generated", false)) and String(lb.get("class", "")) != "settlement":
			pts.append(Vector2(lb["x"], lb["y"]))
	for p in pts:
		var n := 0
		for q in pts:
			if p.distance_to(q) < 25.0:
				n += 1
		if n > best_n:
			best_n = n
			best = p
	if best_n < 0:
		return _app.viewport.global_position + _app.viewport.size * 0.5
	return ov.get_global_transform() * ov._point_to_screen(best, rect)


var _fz := -1.0


func _fit_zoom() -> float:
	if _fz < 0.0:
		_fz = _app.viewport.zoom()
	return _fz
