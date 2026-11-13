extends Node
## Ruling EU items 2 and 3 (owner, 2026-11-12), end to end in the real shell:
## a planned route joins the ways it rides and is drawn on their own line, and
## the planner offers river stretches one by one, each with a recommended hull,
## which the user accepts, re-hulls and gives up again through the planner's own
## buttons, pressed with real input events (mouse on the desktop, screen touches
## on the phone).
##
##   desktop: Godot_v4.7.2-stable_win64_console.exe --path . _jpwayriver_probe.tscn -- --out <dir>
##   phone:   Godot_v4.7.2-stable_win64_console.exe --path . _jpwayriver_probe.tscn -- --out <dir> --vp 1080x2340 --force-touch
##
## WINDOWED on purpose (`MISTAKES.md`: pixel probes run windowed); refuses to
## run headless. `--out` is required; `--vp WxH` sets the window after boot and
## selects the phone form, `--force-touch` is accepted for the shell; any other
## argument fails loudly (exit 2).
##
## What it does. Generates seed 912345 at 1024 x 656, 1 200 km (a main world of
## the way census, `way_census::MAIN`), then walks town pairs of the most
## populous places, nearest first. Per pair it commits the route with the way
## join OFF and ON (`set_route_join_ways`) through the planner's own commit
## (`JourneyPlannerView._commit_stops`) and measures, for each, the share of the
## route's points that lie ON a generated way's own line (within 0.001 cell of
## its stored polyline: a spliced point is a way point or a projection onto one).
## The first pair whose share rises by 0.20 or more is the join evidence: map
## frames at x4 with the join off and on, and the planner with its "On ways"
## section. The first pair the planner offers a river stretch on is the river
## evidence: the offer, then "use river" pressed, then a different hull picked,
## then "walk instead" pressed.
##
## Prints `JPWR ...` lines; `JPWR PASS` and exit 0 when every check held, a
## `JPWR FAIL: ...` line and exit 1 on the first that did not, exit 2 when it
## cannot run.

const DESKTOP := Vector2i(1600, 1000)
const SEED := 912345
const GRID := Vector2i(1024, 656)
const KM := 1200.0

var app: Node
var mode := "desktop"
var touch := false
var out_dir := ""

func _p(s: String) -> void:
	print("JPWR " + s)

func _fail(s: String) -> void:
	print("JPWR FAIL: " + s)
	get_tree().quit(1)

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _find(node: Node, pred: Callable) -> Node:
	if pred.call(node):
		return node
	for c in node.get_children():
		var f := _find(c, pred)
		if f != null:
			return f
	return null

func _shot(step: String) -> void:
	await _frames(6)
	await RenderingServer.frame_post_draw
	var img := get_viewport().get_texture().get_image()
	var path := "%s/jpwr_%s_%s.png" % [out_dir, mode, step]
	img.save_png(path)
	_p("shot %s" % path)

## Where the planner's results draw: the planner window on the desktop (PC-8),
## the right dock / sheet in the swap form.
func _results_root() -> Control:
	var jpv = app.journey_planner_view
	if jpv.has_method("in_window") and jpv.in_window():
		return jpv.window().results_body
	return app.right_dock_body

## Scrolls `c` to near the top of the ScrollContainer that holds it.
func _reveal(c: Control) -> void:
	var sc: Node = c.get_parent()
	while sc != null and not (sc is ScrollContainer):
		sc = sc.get_parent()
	if sc != null:
		var s := sc as ScrollContainer
		s.scroll_vertical += int(c.get_global_rect().position.y - s.get_global_rect().position.y) - 12
		await _frames(4)

## Scrolls the results so the section headed `title` (lower case) is at the top.
func _reveal_section(title: String) -> void:
	var head := _find(_results_root(), func(n): return n is Label and String((n as Label).text).to_lower().contains(title)) as Control
	if head != null:
		await _reveal(head)

## A real click (desktop) or tap (phone) at the centre of `c`, after scrolling it
## into view: a mouse motion then a press and a release (or a touch down and up)
## fed to `Input.parse_input_event`, the path the OS's own input takes, at the
## control's position in the root window (an embedded window's own position
## added: the desktop planner is one).
func _press(c: Control) -> void:
	await _reveal(c)
	var pos := c.get_global_rect().get_center()
	var w := c.get_window()
	if w != get_tree().root:
		pos += Vector2(w.position)
	if not touch:
		var mv := InputEventMouseMotion.new()
		mv.position = pos
		mv.global_position = pos
		Input.parse_input_event(mv)
		await _frames(3)
		_p("press %s at %s: hovered %s" % [c.text if c is Button else c.name, str(pos), str(c.get_viewport().gui_get_hovered_control())])
	for down in [true, false]:
		if touch:
			var t := InputEventScreenTouch.new()
			t.index = 0
			t.position = pos
			t.pressed = down
			Input.parse_input_event(t)
		else:
			var m := InputEventMouseButton.new()
			m.button_index = MOUSE_BUTTON_LEFT
			m.position = pos
			m.global_position = pos
			m.pressed = down
			Input.parse_input_event(m)
		await _frames(3)

func _collect(node: Node, pred: Callable, out: Array) -> void:
	if pred.call(node):
		out.append(node)
	for c in node.get_children():
		_collect(c, pred, out)

func _button(text: String) -> Button:
	return _find(_results_root(), func(n): return n is Button and (n as Button).is_visible_in_tree() and String((n as Button).text).to_lower() == text) as Button

## The share of `pts` lying on one of `ways` (each a PackedVector2Array of
## stored control points): within 0.001 cell of a segment.
func _on_way_share(pts: PackedVector2Array, ways: Array) -> float:
	if pts.is_empty():
		return 0.0
	var on := 0
	for p in pts:
		var hit := false
		for w in ways:
			var wp: PackedVector2Array = w
			for k in range(wp.size() - 1):
				if Geometry2D.get_closest_point_to_segment(p, wp[k], wp[k + 1]).distance_to(p) < 0.001:
					hit = true
					break
			if hit:
				break
		if hit:
			on += 1
	return float(on) / float(pts.size())

func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		printerr("PROBE-CANNOT-RUN: headless -- run windowed.")
		get_tree().quit(2)
		return
	var args := OS.get_cmdline_user_args()
	var size := DESKTOP
	var i := 0
	while i < args.size():
		var a := String(args[i])
		if a == "--force-touch":
			touch = true
		elif a == "--vp" and i + 1 < args.size():
			var wh := String(args[i + 1]).split("x")
			size = Vector2i(int(wh[0]), int(wh[1]))
			mode = "phone"
			i += 1
		elif a == "--out" and i + 1 < args.size():
			out_dir = String(args[i + 1])
			i += 1
		else:
			print("JPWR FAIL: unknown argument %s" % a)
			get_tree().quit(2)
			return
		i += 1
	if out_dir == "":
		print("JPWR FAIL: --out <dir> is required")
		get_tree().quit(2)
		return
	get_tree().create_timer(1500.0).timeout.connect(func() -> void:
		printerr("PROBE-CANNOT-RUN: watchdog"); get_tree().quit(2))
	DisplayServer.window_set_size(size)
	get_window().size = size
	get_tree().root.gui_embed_subwindows = true
	await _frames(4)
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.5).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	if app.phone_project_picker != null:
		app.phone_project_picker.hide()
	await _frames(3)
	_p("=== mode=%s phone=%s touch=%s screen=%s ===" % [mode, app.is_phone(), DccTheme.is_touch(), str(app.get_viewport_rect().size)])
	if mode == "phone" and not app.is_phone():
		_fail("--vp/--force-touch did not produce phone mode")
		return
	var bridge = app.bridge
	var wg = bridge.world_gen
	if not wg.has_method("set_route_join_ways"):
		_fail("this binary has no set_route_join_ways (stale DLL?)")
		return
	bridge.generate({"seed": SEED, "width_km": KM, "grid_w": GRID.x, "grid_h": GRID.y,
		"archetype": "", "villages": true, "metropolis": false, "sea_level": 0.42})
	var spins := 0
	while bridge.generating and spins < 4000:
		await get_tree().create_timer(0.25).timeout
		spins += 1
	await _frames(10)
	if not bridge.has_world:
		_fail("generate failed")
		return
	var ways: Array = []
	for w in 20000:
		var d: Dictionary = bridge.way_get("generated", w)
		if d.is_empty():
			break
		if not bool(d.get("hidden", false)):
			ways.append(d.get("points", PackedVector2Array()))
	_p("generated ways: %d" % ways.size())
	if ways.size() < 3:
		_fail("no network")
		return
	var setts: Array = bridge.settlements()
	var idx: Array = range(setts.size())
	idx.sort_custom(func(a, b): return int(setts[a].get("population", 0)) > int(setts[b].get("population", 0)))
	idx = idx.slice(0, 16)
	var pairs: Array = []
	for a in idx.size():
		for b in range(a + 1, idx.size()):
			var pa := Vector2(float(setts[idx[a]].get("x", 0)), float(setts[idx[a]].get("y", 0)))
			var pb := Vector2(float(setts[idx[b]].get("x", 0)), float(setts[idx[b]].get("y", 0)))
			var dd := pa.distance_to(pb)
			if dd >= 0.03 * GRID.x and dd <= 0.25 * GRID.x:
				pairs.append([dd, pa, pb])
	pairs.sort_custom(func(x, y): return float(x[0]) < float(y[0]))
	var jpv = app.journey_planner_view
	var join_done := false
	var river_done := false
	var best_offer := [-1, 0.0]   ## [route index, longest offer's route km]
	for k in mini(pairs.size(), 10):
		var pa: Vector2 = pairs[k][1]
		var pb: Vector2 = pairs[k][2]
		var stops := [{"x": pa.x, "y": pa.y}, {"x": pb.x, "y": pb.y}]
		wg.set_route_join_ways(false)
		var off_i: int = jpv._commit_stops(stops)
		wg.set_route_join_ways(true)
		var on_i: int = jpv._commit_stops(stops)
		if off_i < 0 or on_i < 0:
			_p("pair %d: did not commit" % k)
			continue
		var off_r: Dictionary = bridge.route_get(off_i)
		var on_r: Dictionary = bridge.route_get(on_i)
		var s_off := _on_way_share(off_r.get("points", PackedVector2Array()), ways)
		var s_on := _on_way_share(on_r.get("points", PackedVector2Array()), ways)
		var join: Dictionary = on_r.get("join", {})
		_p("pair %d (%.0f cells): on-way share off %.2f -> on %.2f; join %s; off has join key: %s" % [k, float(pairs[k][0]), s_off, s_on, str(join), str(off_r.has("join"))])
		if off_r.has("join"):
			_fail("a route committed with the join off reports a join")
			return
		if join.is_empty():
			_fail("a route committed with the join on has no join report")
			return
		if float(join.get("cost_ratio_max", 1.0)) > 1.0 + float(join.get("margin", 0.15)) + 1e-9:
			_fail("a joined leg costs more than the margin")
			return
		app.open_journey_planner_with_route(on_i)
		await _frames(14)
		if mode == "phone":
			app._set_sheet_open("left", false)
			app._set_sheet_open("right", true)
			await _frames(6)
		jpv._compute()
		app.right_dock_ctrl.refresh_journey()
		await _frames(10)
		var res: Dictionary = jpv._last_result
		if not res.has("way_spans"):
			_fail("jp_compute returned no way_spans key")
			return
		if not join_done and s_on - s_off >= 0.20:
			join_done = true
			_p("JOIN evidence on pair %d: way_spans=%d" % [k, (res["way_spans"] as Array).size()])
			if (res["way_spans"] as Array).is_empty():
				_fail("a joined route has no way spans")
				return
			if _find(_results_root(), func(n): return n is Label and String((n as Label).text).to_lower().contains("on ways")) == null:
				_fail("no On ways section drawn")
				return
			await _shot("join_planner")
			if mode == "desktop":
				## Main map at x4 on the route's middle: join off, then on.
				## Framed where the unjoined line strays farthest from the joined
				## one (the parallel stroke the ruling names), at x4 and x16.
				var offp: PackedVector2Array = off_r["points"]
				var onp: PackedVector2Array = on_r["points"]
				var mid: Vector2 = onp[onp.size() / 2]
				var worst := 0.0
				for q in offp:
					var dmin := INF
					for kk in range(onp.size() - 1):
						dmin = minf(dmin, Geometry2D.get_closest_point_to_segment(q, onp[kk], onp[kk + 1]).distance_to(q))
					if dmin > worst and dmin < 3.0:
						worst = dmin
						mid = q
				_p("map frames centred on %s, where the unjoined line is %.2f cells off the joined one" % [str(mid), worst])
				jpv.window().hide()
				for z in [4.0, 16.0]:
					for which in [["off", off_i], ["on", on_i]]:
						app.viewport.overlay.set_manual_routes([bridge.route_get(int(which[1]))])
						app.viewport.reset_view()
						await _frames(3)
						app.viewport.zoom_step(z)
						app.viewport.move_view_to(mid.x, mid.y)
						await get_tree().create_timer(2.0).timeout
						await _shot("join_map_x%d_%s" % [int(z), which[0]])
				app.viewport.overlay.set_manual_routes(app.viewport.manual_routes())
				app.open_journey_planner_with_route(on_i)
				await _frames(14)
		var offers: Array = res.get("river_offers", [])
		_p("pair %d: river offers %d note=%s" % [k, offers.size(), String(res.get("river_offers_note", ""))])
		for o0 in offers:
			if float((o0 as Dictionary).get("route_km", 0.0)) > float(best_offer[1]):
				best_offer = [on_i, float((o0 as Dictionary).get("route_km", 0.0))]
	## The river evidence on the route with the longest offer of the pairs walked.
	if int(best_offer[0]) >= 0:
		river_done = true
		app.open_journey_planner_with_route(int(best_offer[0]))
		await _frames(14)
		if mode == "phone":
			app._set_sheet_open("left", false)
			app._set_sheet_open("right", true)
			await _frames(6)
		jpv._compute()
		app.right_dock_ctrl.refresh_journey()
		await _frames(10)
		var res: Dictionary = jpv._last_result
		var offers: Array = res.get("river_offers", [])
		var oi := 0
		for q in offers.size():
			if float((offers[q] as Dictionary).get("route_km", 0.0)) > float((offers[oi] as Dictionary).get("route_km", 0.0)):
				oi = q
		var before := float((res.get("plan", {}) as Dictionary).get("total_days", -1.0))
		var o: Dictionary = offers[oi]
		_p("RIVER offer: %s %.0f km saves %.2f d (%s) by %s; hulls %d" % [String(o.get("dir", "")), float(o.get("route_km", 0.0)),
			float(o.get("saving_days", 0.0)), String(o.get("saving_basis", "")), String(o.get("recommended", "")), (o.get("hulls", []) as Array).size()])
		if not (float(o.get("saving_days", 0.0)) > 0.0):
			_fail("an offer with no saving was shown")
			return
		## The offer's own "use river" button: the row after its note.
		var uses: Array = []
		_collect(_results_root(), func(n): return n is Button and (n as Button).is_visible_in_tree() and String((n as Button).text).to_lower() == "use river", uses)
		var use: Button = uses[oi] if oi < uses.size() else null
		if use == null:
			_fail("no use river button drawn")
			return
		await _reveal_section("river stretches")
		_p("use river button %s (h=%.0f px)" % [str(use.get_global_rect()), use.size.y])
		await _shot("river_offered")
		await _press(use)
		await _frames(12)
		app.right_dock_ctrl.refresh_journey()
		await _frames(6)
		res = jpv._last_result
		var plan: Dictionary = res.get("plan", {})
		var applied: Array = plan.get("river_legs_applied", [])
		var leg_stage := -1
		var stages: Array = plan.get("stages", [])
		for si in stages.size():
			if (stages[si] as Dictionary).has("river_leg"):
				leg_stage = si
		var after := float(plan.get("total_days", -1.0))
		_p("after use river: applied=%d leg stage=%d total_days %.2f -> %.2f (offer said %.2f); vessel=%s" % [applied.size(), leg_stage,
			before, after, float(o.get("days_with", -1.0)), String(((plan.get("results", []) as Array)[leg_stage] as Dictionary).get("eff", {}).get("vessel", "")) if leg_stage >= 0 else "-"])
		if applied.size() != 1 or leg_stage < 0:
			_fail("use river did not add a river leg")
			return
		if absf(after - float(o.get("days_with", -1.0))) > 0.01:
			_fail("the planned days (%.3f) differ from the offer's (%.3f)" % [after, float(o.get("days_with", -1.0))])
			return
		await _reveal_section("river stretches")
		await _shot("river_accepted")
		## Pick a different hull for the accepted leg through its menu.
		var menu := _find(_results_root(), func(n): return n is OptionButton and String((n as OptionButton).tooltip_text).begins_with("Hull for this river stretch")) as OptionButton
		if menu != null:
			var alt := -1
			for it in menu.item_count:
				if it != menu.selected and not menu.is_item_disabled(it):
					alt = it
					break
			if alt >= 0:
				menu.select(alt)
				menu.item_selected.emit(alt)
				await _frames(12)
				var plan2: Dictionary = jpv._last_result.get("plan", {})
				var v2 := String(((plan2.get("results", []) as Array)[leg_stage] as Dictionary).get("eff", {}).get("vessel", "")) if leg_stage < (plan2.get("results", []) as Array).size() else "-"
				_p("re-hulled: %s, total_days %.2f" % [v2, float(plan2.get("total_days", -1.0))])
				await _shot("river_rehulled")
		var walk := _button("walk instead")
		if walk == null:
			_fail("no walk instead button drawn")
			return
		await _press(walk)
		await _frames(12)
		app.right_dock_ctrl.refresh_journey()
		await _frames(4)
		var back: Dictionary = jpv._last_result.get("plan", {})
		_p("after walk instead: applied=%d total_days %.2f" % [(back.get("river_legs_applied", []) as Array).size(), float(back.get("total_days", -1.0))])
		if back.has("river_legs_applied"):
			_fail("walk instead left a leg applied")
			return
		if absf(float(back.get("total_days", -1.0)) - before) > 1e-6:
			_fail("walking again did not restore the plan's days")
			return
	if not join_done:
		_fail("no pair showed the join (share +0.20)")
		return
	if not river_done:
		_fail("no pair was offered a river stretch")
		return
	_p("PASS")
	get_tree().quit(0)
