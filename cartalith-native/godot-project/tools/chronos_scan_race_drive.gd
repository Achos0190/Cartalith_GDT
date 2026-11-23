extends RefCounted
## Ruling HP follow-up (2026-11-23): the chronos strip's vault scan must not touch the engine while a generate worker holds
## it (`chronos_items.gd::_start_scan`; gdext "Gd<T>::bind() failed, already bound" = SCRIPT ERROR 1337, seen in 5 of 11 phone
## Creates). The race is a timer firing mid-generate, so this makes it deterministic: with a world in place and a SECOND
## generate running, `_start_scan` is called on the live strip (what the 0.25 s / 1 s / 5 s timers do). Fixed code defers it to
## `generation_finished`; the mutant (the guard removed) panics. The drive's own exit code carries the verdict (a SCRIPT ERROR
## line fails it). Run on the phone form:
##   python -I cartalith-native/tools/fennara_drive.py GODOT --shell phone res://tools/chronos_scan_race_drive.gd --dll DLL --out DIR --root ROOT
## Checks: the strip exists (control: else nothing was tested), the second generate was running at the call, the scan is not
## started during it (`busy` false, nothing paged), and it did run after the finish (the deferred call landed).

const ChronosItemsO := preload("res://shell/chronos_items.gd")
var _fails := 0


func _check(ctx, what: String, cond: bool, detail: String = "") -> void:
	if not cond:
		_fails += 1
	ctx.log("%s  %s%s" % ["ok  " if cond else "FAIL", what, ("  -- " + detail) if detail != "" else ""])


func run(ctx) -> void:
	var app: Node = ctx.get_scene_root()
	var bridge = app.bridge
	var t0 := Time.get_ticks_msec()
	while Time.get_ticks_msec() - t0 < 20000:
		var hit := false
		for e in DriveCtx.dump_ui(ctx, "+ NEW WORLD"):
			hit = true
			await DriveCtx.click(ctx, Vector2(e["rect"][0] + e["rect"][2] / 2.0, e["rect"][1] + e["rect"][3] / 2.0))
			break
		if hit:
			break
		await ctx.wait(0.25)
	for e in DriveCtx.dump_ui(ctx, "CANCEL"):
		await DriveCtx.click(ctx, Vector2(e["rect"][0] + e["rect"][2] / 2.0, e["rect"][1] + e["rect"][3] / 2.0))
		break
	await ctx.wait(1.0)
	var req := {"seed": 4242, "width_km": 800.0, "grid_w": 256, "grid_h": 160, "archetype": "", "villages": true, "sea_level": 0.45}
	bridge.generate(req)
	while bridge.generating:
		await ctx.wait(0.25)
	await ctx.wait(4.0)   ## the auto-populate passes arm two frames after the finish
	var strip: Node = null
	for n in app.find_children("ChronosItems", "VBoxContainer", true, false):
		strip = n
	_check(ctx, "control: the phone's chronos strip exists", strip != null)
	if strip == null:
		ctx.error("no strip")
		return
	ChronosItemsO.invalidate_scan()
	req["grid_w"] = 1024   ## big enough that the worker is still inside the engine at the call
	req["grid_h"] = 640
	bridge.generate(req)
	var w := Time.get_ticks_msec()
	while not bridge.generating and Time.get_ticks_msec() - w < 3000:
		await ctx.wait(0.05)
	await ctx.wait(0.15)
	_check(ctx, "setup: the second generate is running at the call", bridge.generating)
	strip._start_scan()
	_check(ctx, "the scan did not start while generating", not bool(strip._scan.get("busy", false)) and int(strip._scan["next"]) == 0 and not bool(strip._scan["done"]) and String(strip._scan["error"]) == "")
	while bridge.generating:
		await ctx.wait(0.25)
	await ctx.wait(2.0)
	_check(ctx, "the scan ran once the generate finished", bool(strip._scan["done"]) or String(strip._scan["error"]) != "" or int(strip._scan["next"]) != 0, str(strip._scan.get("error", "")))
	if _fails > 0:
		ctx.error("%d check(s) failed" % _fails)
