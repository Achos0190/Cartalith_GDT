extends RefCounted
## Item 1 of `OUTSTANDING_WORK.md` (2026-11-22): the docked assistant (the Context panel's Ask tab) on a 1000 px tablet had
## its Send row below the fold, inside the dock's own scroll. Run on tablet (and desktop): opens the Ask tab with a real
## click on the ASK tab chip, then checks that Reset / Stop / Send and Save to note are INSIDE the dock's visible frame
## without scrolling anything, and stay there after the dock's scroll is moved to its end and back. Captures frames.
var _fails := 0


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


func _inside(ctx, what: String, btn: Control, frame: Rect2) -> void:
	var r := DriveCtx.rect_of(btn)
	_check(ctx, "%s is inside the dock's visible frame (%.0f..%.0f of %.0f..%.0f)" % [what, r.position.y, r.end.y, frame.position.y, frame.end.y],
		btn.is_visible_in_tree() and r.position.y >= frame.position.y - 1.0 and r.end.y <= frame.end.y + 1.0)


func run(ctx) -> void:
	var app = ctx.get_scene_root()
	if not app.has_method("open_context_tab"):
		app = ctx.get_scene_root().find_child("App", true, false)
	var cont := await _wait_for(ctx, "Continue without a world")
	if not cont.is_empty():
		await _click_entry(ctx, cont)
	await ctx.wait(1.0)
	app.bridge.generate({"seed": 483920, "width_km": 800.0, "grid_w": 256, "grid_h": 164, "archetype": "", "villages": true, "sea_level": 0.45})
	var t0 := Time.get_ticks_msec()
	await ctx.wait(0.2)
	while bool(app.bridge.generating) and Time.get_ticks_msec() - t0 < 300000:
		await ctx.wait(0.25)
	await ctx.wait(2.0)
	app.select("settlement", 0)
	await ctx.wait(1.0)
	var chip := _find(ctx, "ASK", "Button")
	_check(ctx, "the ASK tab chip is there", not chip.is_empty())
	if not chip.is_empty():
		await _click_entry(ctx, chip)
	await ctx.wait(1.5)
	var dock = app.right_dock_ctrl
	var aw = app.assistant_window
	_check(ctx, "the assistant's content is docked in the Ask tab", aw.is_docked())
	var scroll: ScrollContainer = dock._ask_scroll
	var frame: Rect2 = DriveCtx.rect_of(dock._ask_frame)
	ctx.log("ask frame %s, form scroll content height %.0f" % [str(frame), (dock._ask_body as Control).get_combined_minimum_size().y])
	## A fresh view at the top: the input's focus scrolls the dock to the bottom on its own (`follow_focus`), which
	## would show Send for the wrong reason. Stop is hidden while nothing runs, so it is not asserted.
	scroll.scroll_vertical = 0
	await ctx.wait(0.5)
	for nm in ["send", "tokens_reset", "save"]:
		var b: Control = aw.controls.get(nm)
		_check(ctx, "'%s' exists" % nm, b != null)
		if b != null:
			_inside(ctx, nm, b, frame)
	await ctx.capture("ask_docked")
	scroll.scroll_vertical = 100000
	await ctx.wait(0.5)
	for nm in ["send", "save"]:
		var b2: Control = aw.controls.get(nm)
		if b2 != null:
			_inside(ctx, "%s after scrolling the form to its end" % nm, b2, frame)
	await ctx.capture("ask_docked_scrolled")
	## The foot goes home when the window form opens, and back to the dock after (nothing is lost or orphaned).
	aw.open_window()
	await ctx.wait(1.0)
	_check(ctx, "the window form holds the action rows again", (aw.get("_foot_pad") as Node).get_parent() == aw.get("_host"))
	aw.hide()
	await ctx.wait(0.5)
	dock.refresh_ask(true)
	await ctx.wait(1.0)
	_check(ctx, "back in the dock, the rows are in its footer", (aw.get("_foot_pad") as Node).get_parent() == dock.get("_ask_foot"))
	ctx.log("ASSISTANT DOCK %s" % ("FAIL: %d check(s)" % _fails if _fails > 0 else "PASS"))
