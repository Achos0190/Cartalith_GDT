extends RefCounted
## Headless helper (2026-11-23): generated-label counts per class at several `cull.px_per_cell` values, cull on and off,
## for one seed. Logs `DUMP ppc=<p> cull=<on> total=<n> classes={...}`. No window, so no lock.

func run(ctx) -> void:
	var app: Node = ctx.get_scene_root()
	for _i in 40:
		await ctx.wait(0.25)
	var gw := 768
	var gh := 480
	app.bridge.generate({"seed": 4242, "width_km": 400.0, "grid_w": gw, "grid_h": gh, "archetype": "", "villages": true, "sea_level": 0.45}, true)
	while app.bridge.generating:
		await app.get_tree().create_timer(0.25).timeout
	await ctx.wait(2.0)
	var carto = null
	for w in app._workspaces:
		if (w as Object).get_script().get_global_name() == "CartographyWorkspace":
			carto = w
	for on in [true, false]:
		for ppc in [0.0, 0.3, 0.6, 1.0, 1.3, 1.67, 2.0, 4.0]:
			var cull := {"on": on, "advance_ratio": carto._label_advance_ratio()}
			if ppc > 0.0:
				cull["px_per_cell"] = ppc
			var res: Dictionary = app.bridge.labels_generate({"cull": cull})
			var cls := {}
			for r in app.bridge.world_gen.labels_render_list():
				if bool(r.get("generated", false)):
					cls[String(r.get("class", ""))] = int(cls.get(String(r.get("class", "")), 0)) + 1
			ctx.log("DUMP ppc=%.2f cull=%s total=%s ok=%s classes=%s" % [ppc, str(on), str(res.get("total")), str(res.get("ok")), str(cls)])
	ctx.log("dump done")
