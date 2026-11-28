extends RefCounted
## Row "labels hidden by the live cull are still hit-tested" (OUTSTANDING_WORK, labels-cull lane `da74d998`): a generated label the
## overlay does not draw (live collision cull, zoom floor, fade) must answer no tap. Creates a many-label world, zooms with REAL
## `InputEventMagnifyGesture`s, then for every generated row the overlay does not draw asks `auto_label_hit` at that row's own
## point, once as the shell used to (no skip list) and once as it now does (`undrawn_label_keys()`).
## The world's own labels never collide enough to cull live, so the collision is made on the OVERLAY's copy of the rows (one
## row moved onto another; the engine still holds the original points, which is what the tap is sent to).
## Bars: (1) at least one live-culled row is hit by its own key with no skip list (positive control: without it the next bar is
## vacuous); (2) with the skip list NO undrawn row is ever the hit; (3) a drawn row is still hit by its own key (the skip
## list does not blind the pick). A FAIL line fails the run. Never writes the owner's data.

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
	var size: Vector2 = _app.get_viewport().get_visible_rect().size
	var tag := OS.get_environment("SC_TAG") if OS.get_environment("SC_TAG") != "" else "form"
	if size.x < 700.0:
		await _click_entry(ctx, await _wait_for(ctx, "+ NEW WORLD"))
		await _click_entry(ctx, await _wait_for(ctx, "CANCEL"))
	else:
		await _click_entry(ctx, await _wait_for(ctx, "Continue without a world"))
	await ctx.wait(1.0)
	_app.bridge.generate({"seed": 4242, "width_km": 400.0, "grid_w": 256, "grid_h": 160, "archetype": "", "villages": true, "sea_level": 0.45}, true)
	while _app.bridge.generating:
		await _app.get_tree().create_timer(0.25).timeout
	await ctx.wait(2.0)
	var vp = _app.viewport
	var ov = vp.overlay
	## one real pinch of 1.5 so the run is at a zoom that is not the fit
	var ev := InputEventMagnifyGesture.new()
	ev.position = vp.global_position + vp.size * 0.5
	ev.factor = 1.5
	Input.parse_input_event(ev)
	await ctx.wait(1.0)
	var ppc: float = vp.label_px_per_cell()
	var zoom: float = vp.zoom()
	## the drawn generated rows with a key and a text, by rank
	var drawn: Array = []
	for lb in ov._labels:
		if bool(lb.get("generated", false)) and lb.has("key") and String(lb.get("text", "")) != "" and not ov._label_hidden(lb):
			drawn.append(lb)
	_check(ctx, "the world has at least 3 drawn generated labels", drawn.size() >= 3, "%d" % drawn.size())
	## move the LAST drawn row's overlay copy onto the first's; remember where the engine still has it
	var victim: Dictionary = drawn[drawn.size() - 1]
	var anchor: Dictionary = drawn[0]
	var vx := float(victim["x"])
	var vy := float(victim["y"])
	var vkey := String(victim["key"])
	victim["x"] = anchor["x"]
	victim["y"] = anchor["y"]
	ov._labels_rev += 1
	ov._ensure_live_label_cull()
	var culled := bool(victim.get(ov.LIVE_CULL_KEY, false))
	var undrawn: PackedStringArray = ov.undrawn_label_keys()
	_check(ctx, "the moved row is live-culled and in the undrawn list", culled and undrawn.has(vkey), "culled %s, list has it %s, undrawn %d" % [str(culled), str(undrawn.has(vkey)), undrawn.size()])
	var old: String = _app.bridge.auto_label_hit(vx, vy, ppc, zoom)
	var now: String = _app.bridge.auto_label_hit(vx, vy, ppc, zoom, undrawn)
	_check(ctx, "positive control: with no skip list the culled row answers its own tap", old == vkey, "hit '%s' vs key '%s'" % [old, vkey])
	_check(ctx, "with the skip list the culled row answers no tap", now != vkey, "hit '%s'" % now)
	var leaked := 0
	var drawn_ok := 0
	var drawn_n := 0
	for lb in ov._labels:
		if not bool(lb.get("generated", false)) or not lb.has("key") or String(lb.get("text", "")) == "":
			continue
		var k := String(lb["key"])
		var px := float(lb["x"]) if k != vkey else vx
		var py := float(lb["y"]) if k != vkey else vy
		var h: String = _app.bridge.auto_label_hit(px, py, ppc, zoom, undrawn)
		if undrawn.has(k):
			if h == k:
				leaked += 1
		elif not ov._label_hidden(lb):
			drawn_n += 1
			if h == k:
				drawn_ok += 1
	_check(ctx, "no undrawn row (live-culled, below its zoom floor, faded) is ever the hit", leaked == 0, "leaked %d of %d undrawn" % [leaked, undrawn.size()])
	_check(ctx, "a drawn row is still hit by its own key", drawn_ok > 0, "%d of %d" % [drawn_ok, drawn_n])
	ctx.log("%s zoom %.2f rows %d drawn %d undrawn %d" % [tag, zoom, ov._labels.size(), drawn_n, undrawn.size()])
	ctx.log("RESULT fails=%d" % _fails)
