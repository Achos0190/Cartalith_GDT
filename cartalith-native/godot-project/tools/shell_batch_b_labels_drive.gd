extends RefCounted
## Shell batch B (2026-11-23), row "Generated label list differs between two Creates of one seed (view-dependent
## `cull.px_per_cell`)". The scripted session that makes two Creates of ONE seed through the engine call the Create button makes
## (`EngineBridge.generate(request, true)`), waits for everything the shell does on `generation_finished`, and compares the
## generated label list (`labels_render_list`, generated rows only) of the two. Any form.
## Run: `python C:/cw/shellB_run.py tablet res://tools/shell_batch_b_labels_drive.gd <out> SHELLB_SHOTS=<dir>`.
##   L1  the first Create after boot and the second Create of the same seed give the SAME generated labels (name, position, class)
##   L2  re-running the labelling pass once the layout has settled changes nothing (so the list the Create made was not made
##       from a stale fit)
##   L3  the live fit is the same after both Creates (a control for L1/L2: nothing else moved)
##   L5  the same seed with the window 20 % narrower (a different live fit) gives the SAME list: the cull no longer reads the view
## `SHELLB_SEED` (default 4242) and `SHELLB_GRID` ("256x160") pick the world.

var _fails := 0
var _app: Node


func _check(ctx, what: String, cond: bool, detail: String = "") -> void:
	if not cond:
		_fails += 1
	ctx.log("%s  %s%s" % ["ok  " if cond else "FAIL", what, ("  -- " + detail) if detail != "" else ""])


func _find(ctx, t: String, cls: String = "") -> Dictionary:
	for e in DriveCtx.dump_ui(ctx, t):
		if cls == "" or e["class"] == cls:
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


func _shot(ctx, label: String) -> void:
	var dir := OS.get_environment("SHELLB_SHOTS")
	var img: Image = await ctx.frame(4096)
	if img != null and dir != "":
		DirAccess.make_dir_recursive_absolute(dir)
		img.save_png("%s/%s.png" % [dir, label])


func _create(ctx, seed_v: int, gw: int, gh: int) -> void:
	_app.bridge.generate({"seed": seed_v, "width_km": 400.0, "grid_w": gw, "grid_h": gh,
		"archetype": "", "villages": true, "sea_level": 0.45}, true)
	while _app.bridge.generating:
		await _app.get_tree().create_timer(0.25).timeout
	await ctx.wait(1.5)


## The generated rows as sorted "class|text|x|y" strings: order-independent, position to 0.01 of a cell.
func _generated() -> Array:
	var out := []
	for r in _app.bridge.world_gen.labels_render_list():
		if bool(r.get("generated", false)):
			out.append("%s|%s|%.2f|%.2f" % [String(r.get("class", "")), String(r.get("text", "")), float(r.get("x", 0.0)), float(r.get("y", 0.0))])
	out.sort()
	return out


func _diff(a: Array, b: Array) -> String:
	var only_a := a.filter(func(s): return not b.has(s))
	var only_b := b.filter(func(s): return not a.has(s))
	return "only in first %d (e.g. %s), only in second %d (e.g. %s)" % [only_a.size(), str(only_a.slice(0, 2)), only_b.size(), str(only_b.slice(0, 2))]


func run(ctx) -> void:
	_app = ctx.get_scene_root()
	var seed_v := int(OS.get_environment("SHELLB_SEED")) if OS.get_environment("SHELLB_SEED") != "" else 4242
	var grid := (OS.get_environment("SHELLB_GRID") if OS.get_environment("SHELLB_GRID") != "" else "256x160").split("x")
	var size: Vector2 = _app.get_viewport().get_visible_rect().size
	if size.x < 700.0:
		var nw := await _wait_for(ctx, "+ NEW WORLD")
		await _click_entry(ctx, nw)
		await _click_entry(ctx, await _wait_for(ctx, "CANCEL"))
	else:
		var cont := await _wait_for(ctx, "Continue without a world")
		await _click_entry(ctx, cont)
	await ctx.wait(1.0)
	var gw := int(grid[0])
	var gh := int(grid[1])
	ctx.log("form canvas %s, seed %d, grid %dx%d" % [str(size), seed_v, gw, gh])
	var carto = null
	for w in _app._workspaces:
		if (w as Object).get_script().get_global_name() == "CartographyWorkspace":
			carto = w
	_check(ctx, "setup: the CARTO workspace is built", carto != null)
	await _create(ctx, seed_v, gw, gh)
	var first := _generated()
	await _shot(ctx, "labels_wide_%s" % ("phone" if size.x < 700.0 else ("tablet" if size.x < 1650.0 or true else "desktop")))
	var ppc1: float = _app.viewport.label_px_per_cell()
	ctx.log("   create 1: %d generated labels, settled px/cell %.4f" % [first.size(), ppc1])
	await _create(ctx, seed_v, gw, gh)
	var second := _generated()
	var ppc2: float = _app.viewport.label_px_per_cell()
	ctx.log("   create 2: %d generated labels, settled px/cell %.4f" % [second.size(), ppc2])
	_check(ctx, "control: the world is not empty of labels", first.size() >= 10, str(first.size()))
	_check(ctx, "L1 two Creates of one seed give the same generated labels", first == second, _diff(first, second))
	carto._regenerate_labels()
	var third := _generated()
	_check(ctx, "L2 re-running the labelling pass on the settled layout changes nothing (after Create 2)", third == second, _diff(second, third))
	_check(ctx, "L2 ... nor after Create 1's list", third == first, _diff(first, third))
	_check(ctx, "L3 the settled fit is positive and the same for both Creates", ppc1 > 0.0 and is_equal_approx(ppc1, ppc2), "%f vs %f" % [ppc1, ppc2])
	# A different grid between two Creates: the second must not use the first world's fit.
	await _create(ctx, seed_v, gw / 2, gh / 2)
	var small := _generated()
	await _create(ctx, seed_v, gw, gh)
	var back := _generated()
	_check(ctx, "L4 Create at another grid then back at the first gives the first list", back == first, _diff(first, back))
	# L5 (a measurement, not a pass/fail): the SAME seed with the window a different size (a narrower map area).
	var w_before := float(_app.viewport.size.x)
	var win := _app.get_window()
	var old_size := win.size
	win.size = Vector2i(int(old_size.x * 0.8), old_size.y)
	await ctx.wait(1.5)
	var w_after := float(_app.viewport.size.x)
	await _create(ctx, seed_v, gw, gh)
	var narrow := _generated()
	var ppc3: float = _app.viewport.label_px_per_cell()
	await _shot(ctx, "labels_narrow_%s" % ("phone" if size.x < 700.0 else "tablet"))
	_check(ctx, "control: the window change moved the live fit (so L5 is a real change of view)", absf(ppc3 - ppc1) > 0.1 * ppc1, "%f vs %f" % [ppc1, ppc3])
	_check(ctx, "L5 the same seed in a narrower window gives the same generated labels", narrow == first,
		"map width %.0f -> %.0f px, %d -> %d labels, %s" % [w_before, w_after, first.size(), narrow.size(), _diff(first, narrow)])
	for row in first:
		ctx.log("LBL " + String(row))
	ctx.log("   advance ratio %.4f, cull on %s" % [carto._label_advance_ratio(), str(carto._label_cull)])
	ctx.log("   list digest (compare across forms): %d labels, hash %d" % [first.size(), str(first).hash()])
	win.size = old_size
	ctx.log("shell batch B labels drive: %d failure(s)" % _fails)
