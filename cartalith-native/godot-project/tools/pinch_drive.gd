extends RefCounted
## Row "pinch zoom may be applied twice" (OUTSTANDING_WORK, labels-cull lane `da74d998`): sends REAL `InputEventMagnifyGesture`s of factor
## PINCH_F (default 1.3) through the engine's own entry (`Input.parse_input_event`, what Android's gesture handler calls) and through
## `Viewport.push_input`, and reads the zoom ratio back. One pinch of factor f must give zoom x f (to the 5 % the clamp allows).
## A FAIL line fails the run. Never writes the owner's data.

var _fails := 0
var _app: Node


func _find(ctx, t: String) -> Dictionary:
	for e in DriveCtx.dump_ui(ctx, t):
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


func _check(ctx, what: String, cond: bool, detail: String = "") -> void:
	if not cond:
		_fails += 1
	ctx.log("%s  %s  %s" % ["ok  " if cond else "FAIL", what, detail])


func run(ctx) -> void:
	_app = ctx.get_scene_root()
	var f := float(OS.get_environment("PINCH_F")) if OS.get_environment("PINCH_F") != "" else 1.3
	var size: Vector2 = _app.get_viewport().get_visible_rect().size
	if size.x < 700.0:
		await _click_entry(ctx, await _wait_for(ctx, "+ NEW WORLD"))
		await _click_entry(ctx, await _wait_for(ctx, "CANCEL"))
	else:
		await _click_entry(ctx, await _wait_for(ctx, "Continue without a world"))
	await ctx.wait(1.0)
	_app.bridge.generate({"seed": 4242, "width_km": 400.0, "grid_w": 256, "grid_h": 160, "archetype": "", "villages": true, "sea_level": 0.45}, true)
	while _app.bridge.generating:
		await _app.get_tree().create_timer(0.25).timeout
	await ctx.wait(1.5)
	var vp = _app.viewport
	var pivot: Vector2 = vp.global_position + vp.size * 0.5
	ctx.log("form canvas %s touch %s zoom0 %.4f" % [str(size), str(vp._touch), vp.zoom()])
	for route in ["parse_input_event", "push_input"]:
		for rep in 2:
			var before: float = vp.zoom()
			var ev := InputEventMagnifyGesture.new()
			ev.position = pivot
			ev.factor = f
			if route == "parse_input_event":
				Input.parse_input_event(ev)
			else:
				_app.get_viewport().push_input(ev)
			await ctx.wait(0.6)
			var ratio: float = vp.zoom() / before
			ctx.log("%s #%d zoom %.4f -> %.4f ratio %.4f (sent %.3f)" % [route, rep, before, vp.zoom(), ratio, f])
			_check(ctx, "%s #%d applies the factor once" % [route, rep], absf(ratio - f) < 0.02 * f, "ratio %.4f vs %.3f (twice would be %.3f)" % [ratio, f, f * f])
	## Positive control for the guard: two DIFFERENT pinches in one frame both apply (1.1 then 1.2 = 1.32).
	var b2: float = vp.zoom()
	for ff in [1.1, 1.2]:
		var e2 := InputEventMagnifyGesture.new()
		e2.position = pivot
		e2.factor = ff
		_app.get_viewport().push_input(e2)
	await ctx.wait(0.6)
	_check(ctx, "two different pinches in one frame both apply", absf(vp.zoom() / b2 - 1.32) < 0.02, "ratio %.4f vs 1.32" % (vp.zoom() / b2))
	## Diagnostic: how many times a bare node's `_input` sees one event of each kind sent through `Input.parse_input_event`.
	var counter := _Counter.new()
	_app.get_tree().root.add_child(counter)
	for kind in ["magnify", "pan", "motion", "key"]:
		var e: InputEvent
		match kind:
			"magnify":
				var m := InputEventMagnifyGesture.new()
				m.position = pivot
				m.factor = 1.0
				e = m
			"pan":
				var pg := InputEventPanGesture.new()
				pg.position = pivot
				e = pg
			"motion":
				var mm := InputEventMouseMotion.new()
				mm.position = pivot
				e = mm
			_:
				var k := InputEventKey.new()
				k.keycode = KEY_F24
				k.pressed = false
				e = k
		counter.n = 0
		Input.parse_input_event(e)
		await ctx.wait(0.4)
		ctx.log("DIAG %s sent once through Input.parse_input_event -> a bare node's _input saw %d" % [kind, counter.n])
	counter.queue_free()
	ctx.log("RESULT fails=%d" % _fails)


class _Counter extends Node:
	var n := 0
	func _input(event: InputEvent) -> void:
		if not (event is InputEventScreenTouch):
			n += 1
