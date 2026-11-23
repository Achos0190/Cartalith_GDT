extends RefCounted
## Shell batch A (2026-11-23), phone only: the sheet and tab state after a window opened from MENU closes.
## `_phone_tab` and `_phone_detent` before MENU, with the window open, and after it closes (a real tap on the window's Close).
## Python wrapper: `C:/cw/shellA_drive.py phone res://tools/shell_batch_a_scroll_drive.gd <out>`.
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

func _click_entry(ctx, e: Dictionary) -> void:
	var r: Array = e["rect"]
	await DriveCtx.click(ctx, Vector2(r[0] + r[2] / 2.0, r[1] + r[3] / 2.0))

func _shot(ctx, label: String) -> void:
	var img: Image = await ctx.frame(4096)
	var d := OS.get_environment("SHELLA_SHOTS")
	if img != null and d != "":
		DirAccess.make_dir_recursive_absolute(d)
		img.save_png("%s/%s.png" % [d, label])

func run(ctx) -> void:
	_app = ctx.get_scene_root()
	var nw := _find(ctx, "+ NEW WORLD")
	await _click_entry(ctx, nw)
	await ctx.wait(0.8)
	await _click_entry(ctx, _find(ctx, "CANCEL"))
	await ctx.wait(0.8)
	_app.bridge.generate({"seed": 4242, "width_km": 400.0, "grid_w": 256, "grid_h": 160,
		"archetype": "", "villages": false, "sea_level": 0.45}, false)
	while _app.bridge.generating:
		await _app.get_tree().create_timer(0.25).timeout
	await ctx.wait(1.0)
	for start in [["domain", "peek"], ["domain", "half"], ["map", "half"], ["domain", "full"]]:
		_app._pick_phone_tab(String(start[0]), false)
		_app._set_phone_detent(String(start[1]))
		await ctx.wait(0.8)
		var tab0: String = _app._phone_tab
		var det0: String = _app._phone_detent
		var sc0 := _sheet_scroller_h()
		ctx.log("start %s: tab=%s detent=%s sheet scroller %.0f px" % [str(start), tab0, det0, sc0])
		await _shot(ctx, "scroll_%s_%s_0before" % [start[0], start[1]])
		await DriveCtx.menu_open(ctx, "Assets", "")
		await ctx.wait(0.8)
		var row := _find(ctx, "Landmark types")
		_check(ctx, "%s/%s: MENU > Assets lists Landmark types" % start, not row.is_empty())
		await _click_entry(ctx, row)
		await ctx.wait(1.0)
		var close := _find(ctx, "Close", "Button")
		_check(ctx, "%s/%s: the window offers Close" % start, not close.is_empty())
		if not close.is_empty():
			await _click_entry(ctx, close)
		await ctx.wait(1.0)
		await _shot(ctx, "scroll_%s_%s_1after" % [start[0], start[1]])
		_check(ctx, "%s/%s: after the window closes the tab is as before (%s)" % [start[0], start[1], tab0], _app._phone_tab == tab0, _app._phone_tab)
		_check(ctx, "%s/%s: and the sheet detent is as before (%s)" % [start[0], start[1], det0], _app._phone_detent == det0, _app._phone_detent)
		var sc1 := _sheet_scroller_h()
		_check(ctx, "%s/%s: and the sheet's scroller is as tall as before (%.0f px)" % [start[0], start[1], sc0], absf(sc1 - sc0) < 2.0, "%.0f" % sc1)
	ctx.log("RESULT: %d failed" % _fails)
	if _fails > 0:
		ctx.error("FAIL %d checks" % _fails)


## Height of the tallest visible ScrollContainer under the phone sheet (the dock's own scroller mounted there), or 0.
func _sheet_scroller_h() -> float:
	var best := 0.0
	for n in _app._phone_sheet_col.find_children("*", "ScrollContainer", true, false):
		var c := n as Control
		if c.is_visible_in_tree():
			best = maxf(best, c.size.y)
	return best
