extends RefCounted
## Ruling HY (2026-11-24): a New World resets the numeric generation rule `civ.border_friction` to the app default, as it
## already does the on/off rules (`params::restore_new_world_rules`). Driven through the REAL New World screen:
##   python -I cartalith-native/tools/fennara_drive.py GODOT --shell desktop|tablet|phone res://tools/new_world_rules_drive.gd \
##       --dll DLL --out DIR --root ROOT --timeout 400
## Steps: past the start card; the open world's table is set to what an older save loads (`civ.border_friction` 0, the
## `LOADS_WHEN_ABSENT` value); read back 0 (positive control); File > New world; the screen's own Create at 512 columns;
## after the world exists `civ.border_friction` reads the app's 0.6 (a literal, not the constant read back).
## Mutant: `new_world_rules_mutant.py`.

var _fails := 0
var app: Node
var bridge


func _check(ctx, what: String, cond: bool, detail: String = "") -> void:
	if not cond:
		_fails += 1
	ctx.log("%s  %s%s" % ["ok  " if cond else "FAIL", what, ("  -- " + detail) if detail != "" else ""])


func _find(ctx, t: String, cls: String = "") -> Dictionary:
	for e in DriveCtx.dump_ui(ctx, t):
		if cls == "" or e["class"] == cls:
			return e
	return {}


func _wait_for(ctx, t: String, secs: float = 20.0, cls: String = "") -> Dictionary:
	var end := Time.get_ticks_msec() + int(secs * 1000)
	while Time.get_ticks_msec() < end:
		var e := _find(ctx, t, cls)
		if not e.is_empty():
			return e
		await ctx.wait(0.25)
	return {}


func _click_entry(ctx, e: Dictionary) -> void:
	var r: Array = e["rect"]
	await DriveCtx.click(ctx, Vector2(r[0] + r[2] / 2.0, r[1] + r[3] / 2.0))


func run(ctx) -> void:
	var root: Node = ctx.get_scene_root()
	app = root if root.has_method("select_domain") else root.find_child("App", true, false)
	bridge = app.bridge
	var size: Vector2 = root.get_viewport().get_visible_rect().size
	var phone := size.x < 700.0
	if phone:
		var nw := await _wait_for(ctx, "+ NEW WORLD")
		_check(ctx, "worlds list shows + NEW WORLD", not nw.is_empty())
		await _click_entry(ctx, nw)
	else:
		var cont := await _wait_for(ctx, "Continue without a world")
		_check(ctx, "start card shows Continue", not cont.is_empty())
		await _click_entry(ctx, cont)
		await ctx.wait(0.5)
		# an older save loads border friction 0: put the live table there before the New World screen reads it
		bridge.param_set("civ.border_friction", 0.0)
		_check(ctx, "positive control: the table holds the older save's 0", is_zero_approx(float(bridge.param_get("civ.border_friction"))), str(bridge.param_get("civ.border_friction")))
		_check(ctx, "File > New world opens", await DriveCtx.menu_open(ctx, "File", "New world"))
	if phone:
		bridge.param_set("civ.border_friction", 0.0)
		_check(ctx, "positive control: the table holds the older save's 0", is_zero_approx(float(bridge.param_get("civ.border_friction"))), str(bridge.param_get("civ.border_friction")))
	await ctx.wait(1.5)
	var dlg: Node = null
	for n in root.get_viewport().find_children("*", "", true, false):
		if n.get("_factions_spin") != null and n.has_method("_on_resolution_selected"):
			dlg = n
			break
	_check(ctx, "the New World screen is up", dlg != null)
	if dlg == null:
		ctx.log("HY FAIL")
		return
	(dlg.get("resolution_input") as OptionButton).select(0)
	dlg.call("_on_resolution_selected", 0)
	await ctx.wait(0.5)
	var create: Control = dlg.get("_phone_create") if phone else (dlg as AcceptDialog).get_ok_button()
	_check(ctx, "the create button is visible", create != null and create.is_visible_in_tree())
	await ctx.capture("new_world_screen")
	await DriveCtx.click(ctx, DriveCtx.rect_of(create).get_center())
	var t0 := Time.get_ticks_msec()
	await ctx.wait(0.5)
	while (bool(bridge.generating) or bridge.settlements().size() == 0) and Time.get_ticks_msec() - t0 < 300000:
		await ctx.wait(0.5)
	await ctx.wait(2.0)
	var v := float(bridge.param_get("civ.border_friction"))
	_check(ctx, "the new world starts at the app's border friction 0.6, not the older save's 0", is_equal_approx(v, 0.6), str(v))
	await ctx.capture("after_create")
	ctx.log("HY %s" % ("FAIL: %d check(s)" % _fails if _fails > 0 else "PASS"))
