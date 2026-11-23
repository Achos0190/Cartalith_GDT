extends RefCounted
## Ruling HU item 2 (owner, 2026-11-23): a live sea-level move, a sculpt, opening a save and a parameter write no longer
## rebuild the markers' table (`WorldGen.market_points`, every settlement's town origin and market) on the main thread;
## the engine answers the last table and builds the new one on a thread of its own, which `EngineBridge._market_poll`
## lands. Driven through the REAL shell on one form:
##   python -I <evidence>/run_drive.py desktop OUT 900 res://tools/marker_async_drive.gd [HU_SEED=… HU_GW=… HU_GH=…]
## (defaults 483920, 384, 288; `run_drive.py` holds `C:/cw/.win_lock` with an owner file). Per case it prints, from
## `WorldGen.market_points_stats()` ({main, worker, last_ms}), how many tables were built on the main thread, the
## shell's marker call right after the change (`EngineBridge.settlement_bank_offsets`, the call that used to build
## in place) and the longest frame in the four seconds after it. Checks (`ok` / `FAIL`), per case:
##   1. no table was built on the main thread;
##   2. a table landed from the job (the worker count rose) within 20 s;
##   3. once landed, the shell's markers are the exact ones (`set_markers_async(false)`, the table built in place);
##   4. the change moved the table's key at all (the control: the job ran because the world changed);
##   5. the map's markers stand where the landed table puts them (the overlay re-asked when it landed).
## Against a build without the job (no `market_points_poll`: the state before this ruling) checks 1 to 3 FAIL, which
## is the drive's mutant; that run is the BEFORE timing.

var _fails := 0


func _check(ctx, what: String, cond: bool, detail: String = "") -> void:
	if not cond:
		_fails += 1
	ctx.log("%s  %s%s" % ["ok  " if cond else "FAIL", what, ("  -- " + detail) if detail != "" else ""])


func _env(k: String, d: String) -> String:
	var v := OS.get_environment(k)
	return v if v != "" else d


func _past_welcome(ctx) -> void:
	for t in ["Continue without a world", "+ NEW WORLD"]:
		for e in DriveCtx.dump_ui(ctx, t):
			var r: Array = e["rect"]
			await DriveCtx.click(ctx, Vector2(r[0] + r[2] / 2.0, r[1] + r[3] / 2.0))
			await ctx.wait(0.8)
			for c in DriveCtx.dump_ui(ctx, "CANCEL"):
				var rc: Array = c["rect"]
				await DriveCtx.click(ctx, Vector2(rc[0] + rc[2] / 2.0, rc[1] + rc[3] / 2.0))
			return


## The longest frame, ms, over `secs` seconds of the shell running.
func _longest_frame(ctx, secs: float) -> float:
	var tree: SceneTree = ctx.get_scene_root().get_tree()
	var worst := 0.0
	var t_end := Time.get_ticks_msec() + int(secs * 1000.0)
	var last := Time.get_ticks_usec()
	while Time.get_ticks_msec() < t_end:
		await tree.process_frame
		var now := Time.get_ticks_usec()
		worst = maxf(worst, (now - last) / 1000.0)
		last = now
	return worst


func _case(ctx, app, bridge, name: String, change: Callable) -> void:
	var wg = bridge.world_gen
	var has_job: bool = wg.has_method("market_points_poll")
	var s0: Dictionary = wg.market_points_stats()
	await change.call()
	## What the shell does next: the map asks for its markers (`MapOverlay.bank_stale`, as a repaint or a zoom does).
	var tq := Time.get_ticks_usec()
	bridge.settlement_bank_offsets(2.4, -1.0)
	var q_ms := (Time.get_ticks_usec() - tq) / 1000.0
	app.viewport.overlay.bank_stale()
	var worst := await _longest_frame(ctx, 4.0)
	var t0 := Time.get_ticks_msec()
	while int(wg.market_points_stats()["worker"]) == int(s0["worker"]) and Time.get_ticks_msec() - t0 < 20000:
		await ctx.wait(0.1)
	await ctx.wait(0.3)
	var s1: Dictionary = wg.market_points_stats()
	ctx.log("%s: main-thread builds %d -> %d, job tables %d -> %d (last %.0f ms); the shell's marker call %.1f ms; longest frame in 4 s %.0f ms" % [name, int(s0["main"]), int(s1["main"]), int(s0["worker"]), int(s1["worker"]), float(s1["last_ms"]), q_ms, worst])
	_check(ctx, name + ": no table built on the main thread", int(s1["main"]) == int(s0["main"]), str(s1))
	_check(ctx, name + ": a table landed from the job", int(s1["worker"]) > int(s0["worker"]), str(s1))
	var got: PackedVector2Array = bridge.settlement_bank_offsets(2.4, -1.0)
	if has_job:
		wg.set_markers_async(false)
	var exact: PackedVector2Array = wg.settlement_bank_offsets(2.4, -1.0)
	var s2: Dictionary = wg.market_points_stats()
	_check(ctx, name + ": the markers are the exact ones once landed", got.size() > 0 and got == exact, "%d markers" % got.size())
	_check(ctx, name + ": the change moved the table (the control)", int(s1["worker"]) > int(s0["worker"]) or int(s1["main"]) > int(s0["main"]) or int(s2["main"]) > int(s1["main"]), str(s2))
	## 5. The map took the landed table: the offsets the overlay stamped on its markers are the exact ones for the
	## density it draws at (`EngineBridge._market_poll` re-asks through `bank_params_changed`).
	var ov = app.viewport.overlay
	var ppc: float = ov._bank_ppc
	var at_ppc: PackedVector2Array = wg.settlement_bank_offsets(ppc, -1.0) if ppc > 0.0 else PackedVector2Array()
	var off := 0
	for i in mini(ov._settlements.size(), at_ppc.size()):
		var b = (ov._settlements[i] as Dictionary).get("bank")
		if not (b is Vector2) or (b as Vector2).distance_to(at_ppc[i]) > 1e-4:
			off += 1
	_check(ctx, name + ": the map's markers stand where the landed table puts them", ppc > 0.0 and at_ppc.size() > 0 and off == 0, "%d of %d markers elsewhere at ppc %.3f" % [off, at_ppc.size(), ppc])
	if has_job:
		wg.set_markers_async(true)


func run(ctx) -> void:
	var app = ctx.get_scene_root()
	if not ("bridge" in app):
		app = ctx.get_scene_root().find_child("App", true, false)
	var bridge = app.bridge
	await ctx.wait(1.5)
	await _past_welcome(ctx)
	var req := {"seed": int(_env("HU_SEED", "483920")), "width_km": 800.0, "grid_w": int(_env("HU_GW", "384")), "grid_h": int(_env("HU_GH", "288")), "archetype": "", "villages": true}
	bridge.generate(req)
	await ctx.wait(0.2)
	var t0 := Time.get_ticks_msec()
	while bool(bridge.generating) and Time.get_ticks_msec() - t0 < 900000:
		await ctx.wait(0.25)
	await ctx.wait(3.0)
	var on: bool = bool(bridge.world_gen.get_params().get("civ.bank_sites", false))
	_check(ctx, "the switch is on and the world has settlements (the control)", on and bridge.settlements().size() > 0, "%d settlements" % bridge.settlements().size())
	## Saved now, opened after the sculpt: the opened world differs from the one on screen, as a real open does.
	var path := OS.get_environment("HN_OUT").path_join("hu_marker_drive.zip")
	bridge.save_project(path)
	await _case(ctx, app, bridge, "SEA LEVEL 0.50 -> 0.51", func() -> void:
		bridge.sea_level_live(0.51))
	await _case(ctx, app, bridge, "SCULPT a stroke, committed", func() -> void:
		var gs: Vector2i = bridge.grid_size()
		bridge.sculpt_begin_stroke()
		for k in 12:
			bridge.sculpt_add_point(gs.x * (0.3 + 0.02 * k), gs.y * 0.5)
		bridge.sculpt_end_stroke()
		bridge.sculpt_commit("Ruling HU drive"))
	await _case(ctx, app, bridge, "OPEN the save made before the sea-level move", func() -> void:
		bridge.load_save(path))
	var hug: bool = bool(bridge.world_gen.get_params().get("urban_wall_hug", false))
	await _case(ctx, app, bridge, "PARAMETER WRITE urban_wall_hug", func() -> void:
		bridge.param_write("urban_wall_hug", not hug))
	ctx.log("RESULT %s" % ("PASS" if _fails == 0 else "FAIL (%d)" % _fails))
