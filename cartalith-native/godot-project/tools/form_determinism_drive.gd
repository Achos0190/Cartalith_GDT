extends RefCounted
## Form determinism (2026-11-23, OUTSTANDING "Same request, different world by form"). One New World Create of one
## seed through the real dialog (File > New world, the dialog's own Create button), then a dump of what decided the world:
## the dialog's request, the full parameter table before and after the run, the auto-populate plan, and digests of the
## heightmap (16-bit PNG export), the rivers, the settlements and the generated labels. Run by
## `cartalith-native/tools/form_determinism.py` once per form, each with a fresh APPDATA; `FORMDET_OUT=<dir>` receives
## `<form>.json`, which that tool diffs across forms (the check is the cross-form diff, not anything inside one run).
## `FORMDET_SEED` (default 4242), `FORMDET_GRID` ("256x160").
## F1  the Create ran and made a world with settlements and labels (a control: an empty world diffs equal everywhere).

var _fails := 0


func _check(ctx, what: String, cond: bool, detail: String = "") -> void:
	if not cond:
		_fails += 1
	ctx.log("%s  %s%s" % ["ok  " if cond else "FAIL", what, ("  -- " + detail) if detail != "" else ""])


func _wait_text(ctx, t: String, secs: float = 20.0) -> Dictionary:
	var end := Time.get_ticks_msec() + int(secs * 1000)
	while Time.get_ticks_msec() < end:
		for e in DriveCtx.dump_ui(ctx, t):
			return e
		await ctx.wait(0.25)
	return {}


func _click_entry(ctx, e: Dictionary) -> void:
	var r: Array = e["rect"]
	await DriveCtx.click(ctx, Vector2(r[0] + r[2] / 2.0, r[1] + r[3] / 2.0))


## A Dictionary/Array as canonical text (sorted keys), so equal content hashes equal.
func _canon(v) -> String:
	if v is Dictionary:
		var keys: Array = (v as Dictionary).keys()
		keys.sort_custom(func(a, b): return str(a) < str(b))
		var parts := []
		for k in keys:
			parts.append("%s:%s" % [str(k), _canon(v[k])])
		return "{" + ",".join(parts) + "}"
	if v is Array:
		var parts := []
		for x in v:
			parts.append(_canon(x))
		return "[" + ",".join(parts) + "]"
	return var_to_str(v)


func _labels(bridge) -> Array:
	var out := []
	for r in bridge.world_gen.labels_render_list():
		if bool(r.get("generated", false)):
			out.append("%s|%s|%.2f|%.2f" % [String(r.get("class", "")), String(r.get("text", "")), float(r.get("x", 0.0)), float(r.get("y", 0.0))])
	out.sort()
	return out


func run(ctx) -> void:
	var app: Node = ctx.get_scene_root()
	var bridge = app.bridge
	var seed_v := int(OS.get_environment("FORMDET_SEED")) if OS.get_environment("FORMDET_SEED") != "" else 4242
	var grid := (OS.get_environment("FORMDET_GRID") if OS.get_environment("FORMDET_GRID") != "" else "256x160").split("x")
	var size: Vector2 = app.get_viewport().get_visible_rect().size
	var form := "phone" if DccTheme.is_phone() else ("tablet" if DccTheme.is_touch() else "desktop")
	if size.x < 700.0:
		await _click_entry(ctx, await _wait_text(ctx, "+ NEW WORLD"))
		await _click_entry(ctx, await _wait_text(ctx, "CANCEL"))
	else:
		await _click_entry(ctx, await _wait_text(ctx, "Continue without a world"))
	await ctx.wait(1.0)
	var params_boot: Dictionary = bridge.world_gen.get_params()
	ctx.log("boot: use_gpu %s, stored choice %d" % [str(params_boot.get("use_gpu")), DccSettings.gpu_enabled()])
	var opened: bool = await DriveCtx.menu_open(ctx, "File", "New world")
	await ctx.wait(1.0)
	var dlg = app.new_world_dialog
	if not dlg.visible:
		ctx.log("note: File > New world did not open the dialog on this form; opening it as the picker's tile does")
		app.open_new_world()
		await ctx.wait(1.0)
	_check(ctx, "setup: the New world dialog is open", dlg.visible, "menu row clicked: %s" % str(opened))
	## The seed and the grid as a user sets them: values written into the dialog's own inputs (the field the request reads).
	dlg.seed_input.value = seed_v
	dlg.grid_w_input.value = int(grid[0])
	dlg.grid_h_input.value = int(grid[1])
	var request: Dictionary = dlg.generate_request()
	var plan: Array = dlg.auto_populate_plan()
	## Desktop and tablet: the dialog's footer button. Phone: §6.7's CREATE WORLD (the footer is hidden there).
	var ok_btn: Button = dlg.get_ok_button() if dlg.get_ok_button().is_visible_in_tree() else dlg._phone_create
	if dlg._phone_create != null and is_instance_valid(dlg._phone_create):
		## The phone card scrolls: bring CREATE WORLD on screen the way a thumb would leave it, then click it.
		dlg._phone_create.grab_focus()
		await ctx.wait(0.3)
	await DriveCtx.click(ctx, DriveCtx.rect_of(ok_btn).get_center())
	await ctx.wait(0.5)
	var t0 := Time.get_ticks_msec()
	while not bridge.generating and Time.get_ticks_msec() - t0 < 3000:
		await ctx.wait(0.1)
	_check(ctx, "setup: a real click on Create started the run", bridge.generating or bridge.has_world)
	while bridge.generating:
		await ctx.wait(0.25)
	await ctx.wait(4.0)   ## the auto-populate passes Create arms run two frames after the finish
	var params_after: Dictionary = bridge.world_gen.get_params()
	var out_dir := OS.get_environment("FORMDET_OUT")
	DirAccess.make_dir_recursive_absolute(out_dir)
	var png := "%s/%s_height.png" % [out_dir, form]
	var ex: Dictionary = bridge.world_gen.export_heightmap_png(png, 2048)
	var setl: Array = bridge.world_gen.get_settlements()
	var rivers: Array = bridge.world_gen.get_rivers(0)
	var labels := _labels(bridge)
	var dump := {
		"form": form, "canvas": [size.x, size.y], "seed": seed_v,
		"request": request, "auto_populate_plan": plan,
		"params_boot": params_boot, "params_after": params_after,
		"hash": {
			"height_png_md5": FileAccess.get_md5(png) if bool(ex.get("ok", false)) else "export failed: %s" % str(ex),
			"rivers_md5": _canon(rivers).md5_text(), "rivers": rivers.size(),
			"settlements_md5": _canon(setl).md5_text(), "settlements": setl.size(),
			"labels_md5": _canon(labels).md5_text(), "labels": labels.size(),
		},
		"settlements_list": setl.map(func(s): return "%s|%.4f|%.4f" % [String(s.get("name", "")), float(s.get("x", 0.0)), float(s.get("y", 0.0))]),
		"labels_list": labels,
	}
	var f := FileAccess.open("%s/%s.json" % [out_dir, form], FileAccess.WRITE)
	f.store_string(JSON.stringify(dump, "  ", true))
	f.close()
	ctx.log("form %s: height %s, rivers %d, settlements %d, labels %d" % [form, dump["hash"]["height_png_md5"], rivers.size(), setl.size(), labels.size()])
	_check(ctx, "F1 the world has settlements and labels", setl.size() > 0 and labels.size() >= 10, "%d / %d" % [setl.size(), labels.size()])
	if _fails > 0:
		ctx.error("%d check(s) failed" % _fails)
