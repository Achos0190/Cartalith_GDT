extends RefCounted
## Export speed through the REAL shell under the Fennara driver (the HV lane's conditions): HV content, 2048 px, phase totals logged.
func run(ctx) -> void:
	var app = ctx.get_scene_root()
	if not app.has_method("select_domain"):
		app = ctx.get_scene_root().find_child("App", true, false)
	var bridge = app.bridge
	var host = app.viewport
	for _i in 80:
		var hit := false
		for e in DriveCtx.dump_ui(ctx, "Continue without a world"):
			var r: Array = e["rect"]
			await DriveCtx.click(ctx, Vector2(r[0] + r[2] / 2.0, r[1] + r[3] / 2.0))
			hit = true
			break
		if hit:
			break
		await ctx.wait(0.25)
	await ctx.wait(0.5)
	var grid := int(OS.get_environment("EXP_GRID")) if OS.get_environment("EXP_GRID") != "" else 1024
	bridge.generate({"seed": 483920, "width_km": 800.0, "grid_w": grid, "grid_h": int(grid * 0.64), "archetype": "", "villages": true, "sea_level": 0.45}, true)
	await ctx.wait(0.2)
	while bool(bridge.generating):
		await ctx.wait(0.25)
	await ctx.wait(3.0)
	var runner = load("res://shell/export_overlay.gd").new()
	app.add_child(runner)
	var fr := [0]
	app.get_tree().process_frame.connect(func(): fr[0] += 1)
	var t0 := Time.get_ticks_msec()
	var r: Dictionary = await runner.run(bridge, ProjectSettings.globalize_path("user://exp_speed.png"), {"width": int(OS.get_environment("EXP_W")) if OS.get_environment("EXP_W") != "" else 2048, "format": "png", "content": {"rivers": true, "coast": OS.get_environment("EXP_COAST") != "0", "territory": false}}, host.overlay, [])
	ctx.log("frames %d fps_set %s vsync %s" % [fr[0], str(Engine.max_fps), str(DisplayServer.window_get_vsync_mode())])
	ctx.log("EXPORT err=%s ok=%s wall %.1f s phases %s" % [str(r.get("error")), str(r.get("ok")), (Time.get_ticks_msec() - t0) / 1000.0, str(runner.phase_ms)])
