extends Node
## Ruling BS (owner, 2026-09-28): suggested resupply stops, end to end in the
## real shell -- plan a long overland route with a party whose supplies the
## pre-ruling engine refused outright, see the suggested stops, accept one,
## then the rest, and confirm the journey becomes feasible.
##
##   desktop: Godot_v4.7.1-stable_win64_console.exe --path . _jpresupply_probe.tscn
##   phone:   Godot_v4.7.1-stable_win64_console.exe --path . _jpresupply_probe.tscn -- --vp 1080x2340 --force-touch
##
## WINDOWED on purpose: its evidence is screenshots (`MISTAKES.md`: pixel
## probes run windowed). `--vp WxH` sets the window after boot, where nothing
## clamps it; any other user argument except `--force-touch` fails loudly.
##
## "Blocked before" is shown, not asserted from memory: the first land leg's
## GROSS mass over capacity (`capacity.total_mass / capacity.capacity`, the
## ratio the pre-ruling pre-loop test read) is printed beside
## `JP_LOAD_INVALID_RATIO` (1.50). Over it, the old engine refused the stage.
##
## Screenshots land in `user://jpresupply_<mode>_<step>.png`; each path is
## printed.

const SEEDS := [483920, 77021, 4242, 1234, 98765, 55555]
const DESKTOP := Vector2i(1600, 1000)

var app: Node
var mode := "desktop"

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
	await _frames(4)
	await RenderingServer.frame_post_draw
	var img := get_viewport().get_texture().get_image()
	var out := "user://jpresupply_%s_%s.png" % [mode, step]
	img.save_png(out)
	print("JPRS shot %s" % ProjectSettings.globalize_path(out))

## Scrolls the results dock so the Resupply stops section is on screen.
func _reveal_resupply() -> void:
	var body: Control = app.right_dock_body
	var head := _find(body, func(n): return n is Label and String((n as Label).text).to_lower().contains("resupply stops"))
	if head == null:
		print("JPRS   (no Resupply stops section drawn)")
		return
	var p: Node = head.get_parent()
	while p != null and not (p is ScrollContainer):
		p = p.get_parent()
	if p != null:
		var sc := p as ScrollContainer
		## Section heading to the top of the dock, not merely "somewhere in
		## view" -- the rows and buttons are below it.
		sc.scroll_vertical += int((head as Control).get_global_rect().position.y - sc.get_global_rect().position.y) - 8
	await _frames(3)

func _report(tag: String, jpv) -> Dictionary:
	var res: Dictionary = jpv._last_result
	var plan: Dictionary = res.get("plan", {})
	var v: Dictionary = res.get("verdict", {})
	var sugg: Array = plan.get("resupply_suggestions", [])
	var acc := 0
	for s in sugg:
		if bool((s as Dictionary).get("accepted", false)):
			acc += 1
	print("JPRS %s: dock_min.x=%.0f body_min.x=%.0f" % [tag, (app.right_dock as Control).get_combined_minimum_size().x,
		(app.right_dock_body as Control).get_combined_minimum_size().x])
	print("JPRS %s: verdict=%s total_days=%.1f resupply_stop_days=%.1f suggestions=%d accepted=%d supply_block=%s"
		% [tag, String(v.get("level", "?")), float(plan.get("total_days", -1.0)),
			float(plan.get("resupply_stop_days", 0.0)), sugg.size(), acc,
			String((plan.get("supply_block", {}) as Dictionary).get("reason", "none"))])
	for r in v.get("reasons", PackedStringArray()):
		print("JPRS     reason: %s" % String(r))
	for s in sugg:
		var d: Dictionary = s
		print("JPRS     stop: %s km %.0f +%.0f d cost %.1f accepted=%s" % [String(d.get("name", "")),
			float(d.get("at_km", 0.0)), float(d.get("days", 0.0)), float(d.get("cost", 0.0)), str(d.get("accepted", false))])
	return plan

func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	var size := DESKTOP
	var i := 0
	while i < args.size():
		var a := String(args[i])
		if a == "--force-touch":
			pass
		elif a == "--vp" and i + 1 < args.size():
			var wh := String(args[i + 1]).split("x")
			size = Vector2i(int(wh[0]), int(wh[1]))
			mode = "phone"
			i += 1
		else:
			print("JPRS FAIL: unknown argument %s" % a)
			get_tree().quit(2)
			return
		i += 1
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
	print("JPRS === mode=%s phone=%s touch=%s screen=%s ===" % [mode, app.is_phone(), DccTheme.is_touch(), str(app.get_viewport_rect().size)])
	if mode == "phone" and not app.is_phone():
		print("JPRS FAIL: --vp/--force-touch did not produce phone mode")
		get_tree().quit(1)
		return

	var bridge = app.bridge
	for seed_v in SEEDS:
		bridge.generate({"seed": seed_v, "width_km": 2000.0, "grid_w": 256, "grid_h": 192,
			"archetype": "", "villages": true, "sea_level": 0.45})
		var waited := 0
		while bridge.generating and waited < 3000:
			await get_tree().process_frame
			waited += 1
		await _frames(10)
		if not bridge.has_world:
			print("JPRS seed %d: generate FAILED" % seed_v)
			continue
		## Settlement to settlement, as a real journey runs: among the first
		## 60, the pair whose straight-line distance is nearest 70% of the map
		## width (~1 400 km) -- long enough to outrun the carried provisions.
		var gs: Vector2i = bridge.grid_size()
		var want := gs.x * 0.70
		var setts: Array = bridge.settlements()
		var best := [-1, -1, INF]
		for a in mini(60, setts.size()):
			for b in range(a + 1, mini(60, setts.size())):
				var pa := Vector2(float(setts[a].get("x", 0)), float(setts[a].get("y", 0)))
				var pb := Vector2(float(setts[b].get("x", 0)), float(setts[b].get("y", 0)))
				if absf(pa.distance_to(pb) - want) < float(best[2]):
					best = [a, b, absf(pa.distance_to(pb) - want)]
		if int(best[0]) < 0:
			print("JPRS seed %d: fewer than two settlements" % seed_v)
			continue
		bridge.route_begin("land")
		bridge.route_append_stop(float(setts[best[0]].get("x", 0)), float(setts[best[0]].get("y", 0)))
		bridge.route_append_stop(float(setts[best[1]].get("x", 0)), float(setts[best[1]].get("y", 0)))
		var ridx: int = bridge.route_commit()
		if ridx < 0:
			print("JPRS seed %d: route did not commit" % seed_v)
			continue
		## The planner's own entry point with a route -- what the dock's "Plan
		## a journey" button calls; it arms the tool and shows the results.
		app.open_journey_planner_with_route(ridx)
		await _frames(14)
		var jpv = app.journey_planner_view
		## Twenty walkers, no cargo, 50 days of food, no animals: 600 kg of
		## porter capacity against ~1 t of food -- refused by the pre-ruling
		## engine (gross, unforaged: 1.67x or more). Foraging actively, as
		## a party this under-carried would -- the ruling's "foraging first".
		## Manual carriage, or the auto-picker would add animals and hide it.
		jpv._carriage_auto = false
		for k in ["donkey", "mule", "camel", "horse", "carts", "wagons", "travois", "sleds"]:
			jpv._plan_values[k] = 0
		jpv._plan_values["transport"] = "Walking"
		jpv._plan_values["group_size"] = 20
		jpv._plan_values["cargo_kg"] = 0.0
		jpv._plan_values["supply_days"] = 50
		jpv._plan_values["foraging"] = "Active"
		jpv._resupply_accepted.clear()
		jpv._rebuild_party_form()
		## Phone: the results are the RIGHT sheet (the planner itself raises
		## the left one, the party form). Close left, raise right, as a user
		## flipping to the results would.
		if mode == "phone":
			app._set_sheet_open("left", false)
			app._set_sheet_open("right", true)
			await _frames(6)
		jpv._compute()
		app.right_dock_ctrl.refresh_journey()
		await _frames(10)
		var plan := _report("seed %d BEFORE" % seed_v, jpv)
		var bi := int(plan.get("blocked_idx", -1))
		if bi >= 0:
			print("JPRS   stage %d blocked: %s" % [bi + 1, String((plan.get("results", [])[bi] as Dictionary).get("blocked_reason", ""))])
		for r in plan.get("results", []):
			var land: Dictionary = (r as Dictionary).get("land", {})
			if not land.is_empty():
				var cap: Dictionary = land.get("capacity", {})
				print("JPRS   first land leg: gross mass/capacity = %.2f (pre-ruling refused above 1.50); carry_days=%s capped=%s"
					% [float(cap.get("total_mass", 0.0)) / maxf(1.0, float(cap.get("capacity", 1.0))),
						str(land.get("carry_days", "omitted")), str(land.get("carry_capped", false))])
				break
		var sugg: Array = plan.get("resupply_suggestions", [])
		if sugg.is_empty() or not (plan.get("supply_block", {}) as Dictionary).is_empty():
			## Still useful evidence (a block names its stretch), but the
			## accept-to-feasible path needs a route with somewhere to stop.
			await _reveal_resupply()
			await _shot("0_seed%d_%s" % [seed_v, "blocked" if sugg.size() > 0 else "nostop"])
			print("JPRS seed %d: %s -- trying the next seed" % [seed_v,
				"a stretch with no settlement blocks it" if sugg.size() > 0 else "no suggestions"])
			continue
		await _reveal_resupply()
		await _shot("1_suggested")

		## Accept ONE through the real button.
		var btn := _find(app.right_dock_body, func(n): return n is Button and String((n as Button).text).to_lower() == "accept") as Button
		if btn == null:
			print("JPRS FAIL: no accept button drawn")
			get_tree().quit(1)
			return
		btn.pressed.emit()
		await _frames(10)
		app.right_dock_ctrl.refresh_journey()
		await _frames(6)
		plan = _report("seed %d AFTER ONE ACCEPT" % seed_v, jpv)
		await _reveal_resupply()
		await _shot("2_one_accepted")

		## Then every remaining one, the same way.
		for guard in 20:
			btn = _find(app.right_dock_body, func(n): return n is Button and String((n as Button).text).to_lower() == "accept") as Button
			if btn == null:
				break
			btn.pressed.emit()
			await _frames(8)
			app.right_dock_ctrl.refresh_journey()
			await _frames(4)
		plan = _report("seed %d AFTER ALL ACCEPTED" % seed_v, jpv)
		var lvl := String((jpv._last_result.get("verdict", {}) as Dictionary).get("level", ""))
		print("JPRS RESULT: %s" % ("FEASIBLE (verdict %s, %.1f calendar days)" % [lvl, float(plan.get("total_days", -1.0))]
			if lvl != "blocked" and lvl != "severe" and float(plan.get("total_days", -1.0)) >= 0.0
			else "NOT FEASIBLE (verdict %s)" % lvl))
		await _reveal_resupply()
		await _shot("3_all_accepted")
		## Control for the width figure: the same result with the resupply
		## data emptied (the group then draws nothing), so the delta the
		## section adds to the dock is measured, not assumed.
		var live: Dictionary = jpv._last_result.get("plan", {})
		live["resupply_suggestions"] = []
		live["resupply_stop_days"] = 0.0
		app.right_dock_ctrl.refresh_journey()
		await _frames(6)
		print("JPRS control (no resupply group): dock_min.x=%.0f body_min.x=%.0f" % [
			(app.right_dock as Control).get_combined_minimum_size().x,
			(app.right_dock_body as Control).get_combined_minimum_size().x])
		## The map, with the stops drawn on the route.
		get_tree().quit(0)
		return
	print("JPRS FAIL: no seed produced a route needing resupply stops")
	get_tree().quit(1)
