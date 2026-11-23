extends RefCounted
## Ruling HQ item 4 (owner, 2026-11-23): the first marker draw on a new world no longer pays for every settlement's
## town origin and market on the main thread (`WorldGen.market_points`, 0.75 s at 234 settlements, Ruling HN's
## leftover (f)); the generation worker builds them in its warm-up (`WorldGen.warm_render_caches`). Driven through the
## REAL shell on one form:
##   python -I cartalith-native/tools/fennara_drive.py GODOT <project copy> res://tools/marker_cost_drive.gd \
##       --scene res://shell/app.tscn -- <the form's window and user args>
## (take `C:/cw/.win_lock` yourself, with an owner file, for the run). Inputs (environment): HQ_SEED, HQ_GW, HQ_GH
## (default 483920, 384, 288). It reads `WorldGen.market_points_stats()` ({main, worker, last_ms}: where the table was
## built, and the last build's cost):
##   1. after a generate through `EngineBridge.generate` (the worker path) and two seconds of the shell drawing its
##      markers, the table was built on the worker and never on the main thread (`ok` / `FAIL`);
##   2. after a live sea-level move (`EngineBridge.sea_level_live`, the Sea level control's call) and a redraw, it
##      prints where the rebuild landed and what it cost: a measurement, not a check (the move runs on the main
##      thread, Ruling AQ). HQ_MUTATE=nowarm generates with the shell's render warm-up off (`prewarm_render`), so
##      check 1 must FAIL.

var _fails := 0


func _check(ctx, what: String, cond: bool, detail: String = "") -> void:
	if not cond:
		_fails += 1
	ctx.log("%s  %s%s" % ["ok  " if cond else "FAIL", what, ("  -- " + detail) if detail != "" else ""])


func _env(k: String, d: String) -> String:
	var v := OS.get_environment(k)
	return v if v != "" else d


func _past_welcome(ctx, app) -> void:
	for t in ["Continue without a world", "+ NEW WORLD"]:
		for e in DriveCtx.dump_ui(ctx, t):
			var r: Array = e["rect"]
			await DriveCtx.click(ctx, Vector2(r[0] + r[2] / 2.0, r[1] + r[3] / 2.0))
			await ctx.wait(0.8)
			for c in DriveCtx.dump_ui(ctx, "CANCEL"):
				var rc: Array = c["rect"]
				await DriveCtx.click(ctx, Vector2(rc[0] + rc[2] / 2.0, rc[1] + rc[3] / 2.0))
			return


func run(ctx) -> void:
	var app = ctx.get_scene_root()
	if not ("bridge" in app):
		app = ctx.get_scene_root().find_child("App", true, false)
	var bridge = app.bridge
	await ctx.wait(1.5)
	await _past_welcome(ctx, app)
	if _env("HQ_MUTATE", "") == "nowarm":
		bridge.prewarm_render = false
	var req := {"seed": int(_env("HQ_SEED", "483920")), "width_km": 800.0, "grid_w": int(_env("HQ_GW", "384")), "grid_h": int(_env("HQ_GH", "288")), "archetype": "", "villages": true}
	var s0: Dictionary = bridge.world_gen.market_points_stats()
	bridge.generate(req)
	await ctx.wait(0.2)
	var t0 := Time.get_ticks_msec()
	while bool(bridge.generating) and Time.get_ticks_msec() - t0 < 900000:
		await ctx.wait(0.25)
	await ctx.wait(3.0)
	var places: Array = bridge.settlements()
	var on: bool = bool(bridge.world_gen.get_params().get("civ.bank_sites", false))
	var s1: Dictionary = bridge.world_gen.market_points_stats()
	ctx.log("world %d settlements, civ.bank_sites %s; table builds before %s, after the generate %s" % [places.size(), str(on), str(s0), str(s1)])
	_check(ctx, "the switch is on and the world has settlements (the control)", on and places.size() > 0)
	_check(ctx, "the markers' table was built on the worker", int(s1["worker"]) > int(s0["worker"]), str(s1))
	_check(ctx, "and not on the main thread at the first marker draw", int(s1["main"]) == int(s0["main"]), str(s1))
	## `EngineBridge.generate` sets the request's sea level, 0.5 by default (`EngineBridge.generate`).
	var lvl := 0.5
	var r: Dictionary = bridge.sea_level_live(lvl + 0.01)
	app.viewport.overlay.queue_redraw()
	await ctx.wait(3.0)
	var s2: Dictionary = bridge.world_gen.market_points_stats()
	## What the next marker draw asks (`settlement_bank_offsets`, one call a zoom step), timed on this thread.
	var tq := Time.get_ticks_usec()
	bridge.world_gen.settlement_bank_offsets(2.4, -1.0)
	var q_ms := (Time.get_ticks_usec() - tq) / 1000.0
	var s3: Dictionary = bridge.world_gen.market_points_stats()
	ctx.log("SEA LEVEL %.3f -> %.3f (changed %s): main-thread builds %d -> %d after the redraw, %d after a marker query of %.0f ms; last build %.0f ms" % [lvl, lvl + 0.01, str(r.get("changed", false)), int(s1["main"]), int(s2["main"]), int(s3["main"]), q_ms, float(s3["last_ms"])])
	## The same query again: the table is cached, so this is a zoom step's cost.
	tq = Time.get_ticks_usec()
	bridge.world_gen.settlement_bank_offsets(3.0, -1.0)
	ctx.log("A ZOOM STEP after it: %.1f ms" % ((Time.get_ticks_usec() - tq) / 1000.0))
	ctx.log("RESULT %s" % ("PASS" if _fails == 0 else "FAIL (%d)" % _fails))
