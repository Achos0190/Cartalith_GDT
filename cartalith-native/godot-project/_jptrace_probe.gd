extends Node
## Lane CalcTrace, measurement pass: what does a per-leg substitution trace
## actually reconcile to on a real planned journey?
##
##   Godot_v4.7.1-stable_win64_console.exe --path . --headless _jptrace_probe.tscn
##
## Arithmetic and dictionary shape only -- no pixel is read and no frame is
## timed, so `--headless` is the correct driver here (`MISTAKES.md`: "headless
## for logic and layout; windowed for anything that rasterises or times a
## frame"). The windowed pass that proves the SEVEN ROWS DRAW lives in
## `_jptraceshot_probe.gd`; this one establishes the numbers that pass has to
## agree with.
##
## Three seeds, because a reconciliation is content-dependent exactly as a
## panel width is (`MISTAKES.md`: "Report a layout measurement" -- one world is
## one sample). Every land and water leg of every seed is walked, not stage 0.

const SEEDS := [483920, 77021, 4242]

var app: Node

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ready() -> void:
	get_tree().root.gui_embed_subwindows = true
	await _frames(4)
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.2).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	if app.phone_project_picker != null:
		app.phone_project_picker.hide()
	await _frames(3)
	print("TRACE === phone=", app.is_phone(), " scale=", app.phone_scale(),
		" screen=", app.get_viewport_rect().size, " ===")

	var bridge = app.bridge
	for seed_v in SEEDS:
		bridge.generate({
			"seed": seed_v, "width_km": 2000.0, "grid_w": 256, "grid_h": 192,
			"archetype": "", "villages": true, "sea_level": 0.45,
		})
		var waited := 0
		while bridge.generating and waited < 3000:
			await get_tree().process_frame
			waited += 1
		await _frames(10)
		if not bridge.has_world:
			print("TRACE seed %d: generate FAILED" % seed_v)
			continue
		var gs: Vector2i = bridge.grid_size()
		bridge.route_begin("mixed")
		bridge.route_append_stop(gs.x * 0.20, gs.y * 0.30)
		bridge.route_append_stop(gs.x * 0.55, gs.y * 0.50)
		bridge.route_append_stop(gs.x * 0.82, gs.y * 0.72)
		var ridx: int = bridge.route_commit()

		app.open_journey_planner()
		await get_tree().create_timer(0.8).timeout
		var jpv = app.journey_planner_view
		jpv._refresh_route_choice()
		## Lighten the load so MOST legs plan. **It does not unblock the
		## journey, and an earlier version of this comment said it did.**
		## Measured on this probe's own output, all three seeds:
		## `total_days=-1.0000` still, and 6 of 35 legs remain BLOCKED
		## ("Overloaded 200% of capacity, 240 kg carried vs 120 kg rated") --
		## `jp_journey_plan_dict` reports `-1` because there is no honest total
		## for a blocked journey, and that is unchanged here.
		##
		## What this DOES buy is the 29 unblocked legs the reconciliation is
		## measured over, which is all this probe needs: the residual is a
		## per-leg property, not a journey-total one.
		jpv._plan_values["cargo_kg"] = 40.0
		jpv._compute()
		await _frames(12)
		_report(jpv, seed_v, ridx)

	print("TRACE === done ===")
	get_tree().quit()

func _report(jpv, seed_v: int, ridx: int) -> void:
	var res: Dictionary = jpv._last_result
	if not bool(res.get("ok", false)):
		print("TRACE seed %d: compute not ok: %s" % [seed_v, String(res.get("error", "?"))])
		return
	var plan: Dictionary = res.get("plan", {})
	var stages: Array = plan.get("stages", [])
	var results: Array = plan.get("results", [])
	print("TRACE ##### seed %d route=%d stages=%d km=%.4f travel_days=%.4f total_days=%.4f avg_km_day=%.4f #####"
		% [seed_v, ridx, stages.size(), float(plan.get("km", 0.0)),
			float(plan.get("travel_days", 0.0)), float(plan.get("total_days", 0.0)),
			float(plan.get("avg_km_day", 0.0))])
	var sum_days := 0.0
	var sum_km := 0.0
	for i in results.size():
		var r: Dictionary = results[i]
		var s: Dictionary = stages[i] if i < stages.size() else {}
		if bool(r.get("blocked", false)):
			print("TRACE   %02d BLOCKED %s" % [i + 1, String(r.get("blocked_reason", ""))])
			continue
		var km := float(r.get("km", 0.0))
		var dk := float(r.get("daily_km", 0.0))
		var dy := float(r.get("days", 0.0))
		sum_days += dy
		sum_km += km
		var calc: Dictionary = r.get("land", r.get("water", {}))
		var trace: Array = calc.get("trace", [])
		if trace.is_empty():
			print("TRACE   %02d NO TRACE -- binary predates the JP-05 binding" % [i + 1])
			continue
		var prod := 1.0
		var keys: Array[String] = []
		var six := 1.0
		var six_keys: Array[String] = []
		var rest := 1.0
		var rest_keys: Array[String] = []
		## The six the brief names, mapped onto the engine's own keys. "crossing"
		## is `sailing window` on water; nothing on a land leg carries that name.
		var SIX := ["terrain", "pace", "load", "weather", "sailing window"]
		for t in trace:
			var td: Dictionary = t
			var k := String(td.get("key", ""))
			var f := float(td.get("factor", 1.0))
			prod *= f
			keys.append("%s=%.6f[%s]" % [k, f, String(td.get("detail", ""))])
			if k in SIX:
				six *= f
				six_keys.append(k)
			else:
				rest *= f
				rest_keys.append(k)
		print("TRACE   %02d cat=%-5s km=%10.4f  daily_km=%9.4f  days=%8.4f  terms=%d"
			% [i + 1, String(r.get("cat", "?")), km, dk, dy, trace.size()])
		print("TRACE        prod(all)=%.9f  daily_km=%.9f  RESID_kmday=%.12f" % [prod, dk, prod - dk])
		print("TRACE        km/daily_km=%.9f  days=%.9f  RESID_days=%.12f" % [(km / dk if dk > 0.0 else -1.0), dy, (km / dk - dy if dk > 0.0 else 0.0)])
		print("TRACE        six=%s prod6=%.6f   rest=%s prod_rest=%.6f" % [str(six_keys), six, str(rest_keys), rest])
		var six_days := (km / (six) if six > 0.0 else -1.0)
		print("TRACE        km/prod6=%.4f d vs days=%.4f d  RATIO=%.4f" % [six_days, dy, (six_days / dy if dy > 0.0 else -1.0)])
		print("TRACE        terms: %s" % " ".join(keys))
	print("TRACE   sum(stage days)=%.6f  plan.travel_days=%.6f  RESID=%.12f"
		% [sum_days, float(plan.get("travel_days", 0.0)), sum_days - float(plan.get("travel_days", 0.0))])
	print("TRACE   sum(stage km)=%.6f  plan.km=%.6f  RESID=%.12f"
		% [sum_km, float(plan.get("km", 0.0)), sum_km - float(plan.get("km", 0.0))])
	print("TRACE   total_days=%.6f  travel+rest+layover=%.6f  rest=%d layover=%d"
		% [float(plan.get("total_days", 0.0)),
			float(plan.get("travel_days", 0.0)) + float(plan.get("rest_days", 0)) + float(plan.get("layover_days", 0)),
			int(plan.get("rest_days", 0)), int(plan.get("layover_days", 0))])
