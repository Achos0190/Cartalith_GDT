extends RefCounted
## Owner finding F1 (2026-11-21): "Measure and export to journey made a 897 km measure into a 206 km Journey."
## Driven through the REAL shell, once per form:
##   python -I cartalith-native/tools/fennara_drive.py GODOT --shell desktop|tablet|phone res://tools/f1_measure_journey_drive.gd \
##       --dll DLL --out DIR            (F1_SEED, F1_GW, F1_GH, F1_MODES="distance,area,radius,bearing" in the environment)
## A 2048 x 1311 world, 800 km wide, is generated; for each Measure mode the points are placed with real mouse clicks on
## the map, the dock's "Plan a journey" button is pressed with a real click, and the measure's own length is compared
## with the committed route's and the planner's. Each line is `F1 ...`; a `FAIL` fails the run (launcher exit 1).
## F1_MUTATE=seed makes the check ignore the fix, so the checks must turn red (see the commit's mutant).

var _fails := 0
var app: Node
var bridge


func _check(ctx, what: String, cond: bool, detail: String = "") -> void:
	if not cond:
		_fails += 1
	ctx.log("%s  %s%s" % ["ok  " if cond else "FAIL", what, ("  -- " + detail) if detail != "" else ""])


func _env(k: String, d: String) -> String:
	var v := OS.get_environment(k)
	return v if v != "" else d


func _wait_for(ctx, t: String, secs: float = 20.0) -> Dictionary:
	var end := Time.get_ticks_msec() + int(secs * 1000)
	while Time.get_ticks_msec() < end:
		for e in DriveCtx.dump_ui(ctx, t):
			return e
		await ctx.wait(0.25)
	return {}


func _click_entry(ctx, e: Dictionary) -> void:
	var r: Array = e["rect"]
	await DriveCtx.click(ctx, Vector2(r[0] + r[2] / 2.0, r[1] + r[3] / 2.0))


func _past_welcome(ctx) -> void:
	if app.get_viewport().get_visible_rect().size.x < 700.0:
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


func _screen(gx: float, gy: float) -> Vector2:
	var ov = app.viewport.overlay
	var rect: Rect2 = ov._displayed_rect()
	return ov.get_global_transform_with_canvas() * ov._point_to_screen(Vector2(gx, gy), rect)


## `DccUnits.format_thousands` as the planner prints it: whole km, a space between thousands.
func _thousands(km: float) -> String:
	var n := str(int(round(km)))
	var out := ""
	while n.length() > 3:
		out = " " + n.substr(n.length() - 3) + out
		n = n.substr(0, n.length() - 3)
	return n + out


func _find_button(text: String) -> Button:
	for b in app.find_children("*", "Button", true, false):
		if (b as Button).text.to_lower() == text.to_lower() and (b as Button).is_visible_in_tree():
			return b
	return null


## Points as fractions of the grid: a long route over land and sea, several hundred km between points.
const ROUTE_DEFAULT := [Vector2(0.10, 0.30), Vector2(0.40, 0.70), Vector2(0.70, 0.30), Vector2(0.90, 0.80)]


## F1_ROUTE="x:y,x:y,..." (fractions of the grid) overrides the default chain.
func _route() -> Array:
	var spec := _env("F1_ROUTE", "")
	if spec == "":
		return ROUTE_DEFAULT
	var out: Array = []
	for t in spec.split(","):
		out.append(Vector2(float(t.get_slice(":", 0)), float(t.get_slice(":", 1))))
	return out


func run(ctx) -> void:
	app = ctx.get_scene_root()
	if not ("bridge" in app):
		app = ctx.get_scene_root().find_child("App", true, false)
	bridge = app.bridge
	var size: Vector2 = app.get_viewport().get_visible_rect().size
	var form := "phone" if size.x < 700.0 else ("tablet" if bool(DccTheme.is_tablet()) else "desktop")
	ctx.log("form %s, window %s" % [form, str(size)])
	await _past_welcome(ctx)
	bridge.generate({"seed": int(_env("F1_SEED", "7")), "width_km": 800.0, "grid_w": int(_env("F1_GW", "2048")),
		"grid_h": int(_env("F1_GH", "1311")), "archetype": "", "villages": true})
	await ctx.wait(0.2)
	var t0 := Time.get_ticks_msec()
	while bool(bridge.generating) and Time.get_ticks_msec() - t0 < 900000:
		await ctx.wait(0.5)
	await ctx.wait(2.0)
	if app.get("open_project_dialog") != null and app.open_project_dialog:
		app.open_project_dialog.hide()
	_check(ctx, "a world exists", bool(bridge.has_world))
	var gs: Vector2 = bridge.grid_size()
	ctx.log("grid %s, width %.1f km" % [str(gs), float(bridge.world_gen.get_map_width_km())])
	var modes: PackedStringArray = _env("F1_MODES", "distance,area,radius").split(",")
	for mode in modes:
		await _one_mode(ctx, form, mode, gs)
	ctx.log("F1 %s" % ("FAIL: %d check(s)" % _fails if _fails > 0 else "PASS"))


func _one_mode(ctx, form: String, mode: String, gs: Vector2) -> void:
	GlobalTools.set_measure_mode(app, mode)
	app.arm_tool("measure")
	await ctx.wait(0.5)
	await _fit_view(ctx)
	var route := _route()
	var cap := 2 if mode in ["radius", "bearing", "section", "vertical"] else route.size()
	var vis := _phone_visible(ctx) if form == "phone" else Rect2()
	for i in cap:
		var c: Vector2 = route[i]
		var at := _screen(c.x * gs.x, c.y * gs.y)
		if form == "phone":
			at = vis.position + Vector2(c.x, c.y) * vis.size
		await DriveCtx.click(ctx, at)
		await ctx.wait(0.1)
	var m: Dictionary = bridge.measure_result()
	var pts := GlobalTools.measure_points()
	ctx.log("F1 %s/%s: %d points, measure total %.1f km, straight %.1f km" % [form, mode, pts.size(), float(m.get("total_km", -1)), float(m.get("straight_line_km", -1))])
	# the phone keeps the Measure dock in its right sheet: Window > Context panel opens it (the same menu path a person takes)
	if form == "phone":
		var opened: bool = await DriveCtx.menu_open(ctx, "Window", "Context panel")
		ctx.log("F1 phone: Window > Context panel opened=%s" % str(opened))
		await ctx.wait(1.0)
	ctx.capture("%s_%s_1_measure" % [form, mode])
	var plan_btn := _find_button("Plan a journey")
	_check(ctx, "%s/%s the Measure dock offers Plan a journey" % [form, mode], plan_btn != null)
	if plan_btn == null:
		for b in app.find_children("*", "Button", true, false):
			if (b as Button).text.to_lower() == "plan a journey":
				ctx.log("F1 diag: Plan a journey exists, visible_in_tree=%s, path %s" % [(b as Control).is_visible_in_tree(), str(b.get_path())])
		return
	# the dock scrolls: bring the button on screen first (a click on a clipped button lands on the map instead)
	var anc: Node = plan_btn.get_parent()
	while anc != null and not (anc is ScrollContainer):
		anc = anc.get_parent()
	if anc != null:
		(anc as ScrollContainer).ensure_control_visible(plan_btn)
		await ctx.wait(0.4)
	await DriveCtx.click(ctx, DriveCtx.rect_of(plan_btn).get_center())
	await ctx.wait(4.0)
	var jv = app.journey_planner_view
	var rt: Dictionary = bridge.route_get(jv._route_index)
	var plan: Dictionary = jv._last_result.get("plan", {})
	var want := float(m.get("total_km", -1))
	var rkm := float(rt.get("km", -1))
	var pkm := float(plan.get("km", -1))
	ctx.log("F1 %s/%s: measured %.1f km | route %.1f km (%d pts, %d unreachable legs) | planner %.1f km" % [form, mode, want, rkm,
		(rt.get("points", PackedVector2Array()) as PackedVector2Array).size(), int(rt.get("unreachable_legs", -1)), pkm])
	await ctx.wait(3.0)
	ctx.capture("%s_%s_2b_planner_later" % [form, mode])
	var pw := app.find_child("PlannerWindow", true, false)
	ctx.log("F1 %s/%s: PlannerWindow node %s, visible %s" % [form, mode, str(pw != null), str(pw != null and (pw as Window).visible)])
	ctx.capture("%s_%s_2_planner" % [form, mode])
	# the planner itself, opened by a real click on the dock button, so the frame shows the route list with the new name
	var open_btn := _find_button("Open Journey Planner")
	if open_btn != null:
		var sc: Node = open_btn.get_parent()
		while sc != null and not (sc is ScrollContainer):
			sc = sc.get_parent()
		if sc != null:
			(sc as ScrollContainer).ensure_control_visible(open_btn)
			await ctx.wait(0.3)
		await DriveCtx.click(ctx, DriveCtx.rect_of(open_btn).get_center())
		await ctx.wait(2.5)
		ctx.capture("%s_%s_3_planner_open" % [form, mode])
	else:
		ctx.log("F1 info: no Open Journey Planner button on screen at this form")
	## 1. The planner's distance is the committed route's (nothing lost between them).
	_check(ctx, "%s/%s the planner's km equals the committed route's" % [form, mode], rkm > 0.0 and absf(pkm - rkm) <= 1.0, "route %.1f, planner %.1f" % [rkm, pkm])
	## 2. The route is a least-cost route: never shorter than the straight chain it came from (the owner's 206 < 897 is not this path).
	_check(ctx, "%s/%s the journey is never shorter than the measured chain" % [form, mode], want > 0.0 and rkm >= want * 0.99, "measure %.1f, route %.1f" % [want, rkm])
	## 3. F1's fix: the journey says what it came from and why its km differ. Literals, not the dock's own helpers.
	var label: String = {"distance": "Distance", "area": "Area", "radius": "Radius"}.get(mode, mode)
	var want_name := "From %s measure (%s km straight)" % [label, _thousands(want)]
	_check(ctx, "%s/%s the route is named for its measure" % [form, mode], String(rt.get("name", "")) == want_name, "name '%s', wanted '%s'" % [rt.get("name", ""), want_name])
	var shown := DriveCtx.dump_ui(ctx, want_name)
	ctx.log("F1 %s/%s: the name is drawn by %s" % [form, mode, str(shown.map(func(e): return e["path"]))])
	_check(ctx, "%s/%s the planner's route list shows that name" % [form, mode], not shown.is_empty())
	## The status bar tail slot is dropped by the tablet and phone fit pass, so the line is asserted where it is drawn.
	if form == "desktop":
		_check(ctx, "%s/%s the status line says why the km differ" % [form, mode], not DriveCtx.dump_ui(ctx, "Planned along terrain: %s km; as measured (straight between the points): %s km" % [_thousands(rkm), _thousands(want)]).is_empty())
	if form == "phone":
		app.phone_hide_right_sheet()
	# back to the map for the next mode: Escape closes the desktop planner window; the swap forms return to WORLD
	await DriveCtx.tap_key(ctx, KEY_ESCAPE)
	await ctx.wait(0.5)
	app.select_domain("world")
	await ctx.wait(0.5)


## The navpad's zoom-out button by real clicks until the HUD chip reads a zoom of at most 150%, so most of the map is on
## screen and the points' fractions land on it (the phone opens zoomed to 338%, a fifth of the map). The zoom is read back
## from the chip, not assumed.
func _fit_view(ctx) -> void:
	for n in 20:
		var zoom := 9999.0
		for e in DriveCtx.dump_ui(ctx, "zoom "):
			var t := str(e["text"])
			var k := t.find("zoom ")
			if k >= 0:
				zoom = float(t.substr(k + 5).get_slice("%", 0))
		if zoom <= 150.0:
			ctx.log("F1 view zoom %.0f%% after %d zoom-out clicks" % [zoom, n])
			return
		var clicked := false
		for e in DriveCtx.dump_ui(ctx):
			if str(e["tooltip"]).to_lower() == "zoom out" and str(e["class"]).contains("Button"):
				var r: Array = e["rect"]
				await DriveCtx.click(ctx, Vector2(r[0] + r[2] / 2.0, r[1] + r[3] / 2.0))
				await ctx.wait(0.6)
				clicked = true
				break
		if not clicked:
			ctx.log("F1 no zoom-out button found (the map is left at zoom %.0f%%)" % zoom)
			return


## The phone's map is a strip of the screen between the title cluster and the zoom buttons, and the fitted map is wider than the
## screen: grid fractions land off-screen. The visible clickable region is the displayed map's rect clipped to the window, below
## its top edge and above the zoom buttons, inset 10 px; the route's fractions are then taken of that region.
func _phone_visible(ctx) -> Rect2:
	var ov = app.viewport.overlay
	var rect: Rect2 = ov._displayed_rect()
	var tl: Vector2 = ov.get_global_transform_with_canvas() * ov._point_to_screen(Vector2.ZERO, rect)
	var win: Vector2 = app.get_viewport().get_visible_rect().size
	var bottom := win.y
	var right_edge := win.x - 10.0
	for e in DriveCtx.dump_ui(ctx):
		if str(e["tooltip"]).to_lower() == "zoom out":
			bottom = float((e["rect"] as Array)[1]) - 10.0
			right_edge = float((e["rect"] as Array)[0]) - 10.0
	var top := maxf(tl.y, 0.0) + 10.0
	return Rect2(Vector2(10.0, top), Vector2(right_edge - 10.0, bottom - top))
