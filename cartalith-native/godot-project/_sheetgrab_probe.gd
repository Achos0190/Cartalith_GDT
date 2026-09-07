extends Node
## Lane DRAWER+PREFS, 2026-09-07. The owner: "it seems an issue with dragging
## the drawer up in the sculpt menu." `_detent_probe.gd` drives that same drag
## and is GREEN -- it presses the grab handle's exact CENTRE. This probe asks
## the question that probe cannot: **how wide is the target, and what happens
## when a finger misses it.**
##
##   godot --headless --path . _sheetgrab_probe.tscn -- --force-touch --vp 1080x2340 --tag g1080
##   godot --headless --path . _sheetgrab_probe.tscn -- --force-touch --vp 1440x3200 --tag g1440
##
## Flags this probe actually reads, grepped from the body below:
##   `--vp WxH`      SubViewport size in physical px. Default 1080x2340.
##   `--tag NAME`    prefix on every output line. Default `grab`.
##   `--jitter`      add +-6 px of horizontal wobble to every drag sample, the
##                   shape `_gestclass_probe.gd` uses. Off by default so the
##                   two ladders can be compared.
##   `--force-touch` NOT read here -- `dcc_shell.gd` reads it from
##                   `OS.get_cmdline_user_args()`. Without it the shell boots
##                   desktop and every row below is void.
## Anything else after `--` aborts.
##
## Headless is sound: nothing here reads a pixel, only `Control.size`/`position`
## and the detent the shell reports.

var app: Node
var _vp: SubViewport
var _tag := "grab"
var _jitter := false
var _fail := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _log(s: String) -> void:
	print("[%s] %s" % [_tag, s])

func _check(ok: bool, what: String) -> void:
	if not ok:
		_fail += 1
	_log("  %-4s %s" % ["OK" if ok else "FAIL", what])

func _arg(name: String, dflt: String) -> String:
	var a := OS.get_cmdline_user_args()
	var i := a.find(name)
	if i >= 0 and i + 1 < a.size():
		return String(a[i + 1])
	return dflt

func _dp(px: float) -> float:
	return px / float(app.phone_scale())

func _sheet_h() -> float:
	return (app._phone_tool_sheet as Control).size.y

func _press(at: Vector2, down: bool) -> void:
	var e := InputEventMouseButton.new()
	e.button_index = MOUSE_BUTTON_LEFT
	e.pressed = down
	e.position = at
	e.global_position = at
	_vp.push_input(e, true)

func _motion(at: Vector2, rel: Vector2) -> void:
	var e := InputEventMouseMotion.new()
	e.position = at
	e.global_position = at
	e.relative = rel
	e.button_mask = MOUSE_BUTTON_MASK_LEFT
	_vp.push_input(e, true)

## Eight-sample upward drag, so the gesture has a shape a classifier can read
## rather than the two samples `_detent_probe.gd` sends. `--jitter` adds the
## horizontal wobble a real finger carries; a pure vertical drag is the one
## shape an axis verdict can never get wrong.
func _drag_up(from: Vector2, dy: float) -> Dictionary:
	var h0 := _sheet_h()
	_press(from, true)
	await _frames(1)
	var prev := from
	var h_mid := h0
	for i in range(1, 9):
		var t := float(i) / 8.0
		var jx := 0.0
		if _jitter:
			jx = 6.0 if (i % 2) == 0 else -6.0
		var at := Vector2(from.x + jx, from.y + dy * t)
		_motion(at, at - prev)
		prev = at
		await _frames(1)
		if i == 4:
			h_mid = _sheet_h()
	_press(prev, false)
	await _frames(1)
	await get_tree().create_timer(0.45).timeout
	await _frames(2)
	return {"h0": h0, "mid": h_mid, "end": _sheet_h(),
		"detent": String(app.phone_detent())}

func _reset_peek() -> void:
	app.set_phone_detent("peek")
	await get_tree().create_timer(0.45).timeout
	await _frames(2)

func _ready() -> void:
	_tag = _arg("--tag", "grab")
	var known := ["--force-touch", "--vp", "--tag", "--jitter"]
	for a in OS.get_cmdline_user_args():
		var s := String(a)
		if s.begins_with("--") and not (s in known):
			_log("ABORT unknown flag %s -- this probe reads only %s" % [s, str(known)])
			get_tree().quit(2)
			return
	_jitter = "--jitter" in OS.get_cmdline_user_args()
	var parts: PackedStringArray = _arg("--vp", "1080x2340").split("x")
	if parts.size() != 2:
		_log("ABORT --vp wants WxH")
		get_tree().quit(2)
		return
	_vp = SubViewport.new()
	_vp.size = Vector2i(int(parts[0]), int(parts[1]))
	_vp.transparent_bg = false
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	Input.set_emulate_touch_from_mouse(true)
	app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await get_tree().create_timer(1.6).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(4)
	if not app.is_phone():
		_log("ABORT not phone mode -- pass --force-touch")
		get_tree().quit(2)
		return

	## The owner's own surface: the SCULPT tool options, not the boot default.
	app.arm_tool("sculpt")
	await _frames(4)
	await _reset_peek()

	var s: float = app.phone_scale()
	var grab: Control = app._phone_sheet_grab
	var sheet: Control = app._phone_tool_sheet
	var gr := grab.get_global_rect()
	_log("=== vp=%s scale=%.4f jitter=%s armed=%s ===" % [
		str(_vp.size), s, str(_jitter), String(app.armed_tool)])
	_log("[target]  the only gesture target this sheet has")
	_log("  grab row     y=%.0f..%.0f  h=%.1f px = %.2f dp" % [
		gr.position.y, gr.end.y, gr.size.y, _dp(gr.size.y)])
	_log("  sheet        y=%.0f..%.0f  h=%.1f px = %.2f dp" % [
		sheet.get_global_rect().position.y, sheet.get_global_rect().end.y,
		sheet.size.y, _dp(sheet.size.y)])
	## 44 dp is the floor `phone_fit()` enforces on every other tappable thing
	## in this shell (`DccTheme.PHONE_TAP_MIN`); 48 dp is Android's own. The
	## Android figure is a literal here, not read back from anything.
	_log("  floors       phone_fit tap min = %.0f dp, Android min = 48 dp"
		% DccTheme.PHONE_TAP_MIN)
	## Pinned from BOTH directions with one literal. Below 44 the target is
	## under the floor every other phone control here gets; above it the row
	## eats the `peek` sliver it is supposed to sit at the top of. `44.0` is
	## typed here, not read back from `DccTheme.PHONE_TAP_MIN` -- the shell
	## reads the constant, this asserts the number.
	_check(absf(_dp(gr.size.y) - 44.0) < 1.0,
		"grab row is the 44 dp floor phone_fit() applies to every other target")

	## -- the three gesture arbiters added this week, eliminated by STATE -----
	## `PgSlider` (8 dp slop), `touch_release_button()` (RELEASE + PASS) and
	## `PgField` (`focus_mode` parked) each convert one population:
	## `HSlider`, `BaseButton`, `LineEdit`/`SpinBox`. This asserts the grab
	## handle is in NONE of them, and that `_touch_arbitrate()`'s bare-`Control`
	## clause skipped it -- which is stronger than a mutation run, because it
	## holds for every gesture rather than the ones this probe drove.
	_log("[arbiters]  is the grab handle in any of the three populations?")
	_log("  class=%s script=%s filter=%d gui_input listeners=%d" % [
		grab.get_class(), str(grab.get_script()), grab.mouse_filter,
		grab.get_signal_connection_list("gui_input").size()])
	_check(not (grab is HSlider), "not an HSlider -- PgSlider cannot reach it")
	_check(not (grab is BaseButton),
		"not a BaseButton -- touch_release_button() cannot reach it")
	_check(not (grab is LineEdit) and not (grab is SpinBox),
		"not a text field -- PgField cannot reach it")
	_check(grab.mouse_filter == Control.MOUSE_FILTER_STOP,
		"still MOUSE_FILTER_STOP -- _touch_arbitrate()'s bare-Control clause skipped it")
	_check(grab.get_signal_connection_list("gui_input").size() == 1,
		"exactly one gui_input listener -- the sheet-raise handler")

	## -- the ladder: press at a series of offsets from the handle's centre ---
	## Each rung is one whole gesture from a cold `peek`. A rung that reaches
	## `half` raised the drawer; anything else did not.
	_log("[ladder]  one full drag per rung, from peek, dy = -(half - peek)")
	var half: float = app._phone_detent_height("half")
	var peek: float = app._phone_detent_height("peek")
	var dy := -(half - peek)
	var cy := gr.get_center().y
	var raised: Array[float] = []
	var offsets := [-60.0, -40.0, -26.0, -12.0, 0.0, 12.0, 26.0, 40.0, 60.0, 90.0]
	for off in offsets:
		await _reset_peek()
		var at := Vector2(gr.get_center().x, cy + float(off))
		var r: Dictionary = await _drag_up(at, dy)
		var ok: bool = String(r["detent"]) == "half"
		if ok:
			raised.append(float(off))
		var inside := "in " if gr.has_point(at) else "out"
		_log("  off %+6.0f px (%+6.2f dp) %s handle  mid=%7.1f end=%7.1f detent=%-4s %s" % [
			off, _dp(float(off)), inside, r["mid"], r["end"], r["detent"],
			"RAISED" if ok else "-"])
	## **The band is the grab rect, and that is what this asserts -- not the
	## spread of my own rungs.** The first version of this check reported
	## `last_raised - first_raised`, which is bounded by the offsets chosen
	## above and so measures the ladder rather than the target: it read 14.88 dp
	## against a 19.84 dp rect and 30.90 dp against a 43.87 dp one, wrong in the
	## same direction both times. The honest statement is an equality -- every
	## rung inside the drawn rect raises the sheet, and every rung outside it
	## does not -- which pins the hit region to the drawn region from both
	## sides. The rect's own height is pinned against a literal 44 above.
	_log("  raised at %d of %d rungs (rect is %.0f px tall, so rungs within "
		% [raised.size(), offsets.size(), gr.size.y]
		+ "+-%.1f px of centre should raise)" % (gr.size.y / 2.0))
	_check(raised.size() > 0, "positive control: at least one rung raises the drawer")
	var mismatched := 0
	for off in offsets:
		var want: bool = gr.has_point(Vector2(gr.get_center().x, cy + float(off)))
		if want != (float(off) in raised):
			mismatched += 1
			_log("  MISMATCH off %+.0f: inside=%s raised=%s" % [
				off, str(want), str(float(off) in raised)])
	_check(mismatched == 0,
		"the region that raises the drawer is exactly the region it draws")

	## -- what a miss does instead, which is the half that is invisible -------
	## A press that lands just BELOW the handle is inside the sheet body. If it
	## scrolls that body, the gesture is not merely lost -- it does something
	## else, which is what a user reads as "the drawer will not drag up".
	await _reset_peek()
	## **Both scrollers, named.** `col` holds two: `_phone_tool_scroll` (the
	## tool-options row) and `_phone_gen_scroll` (the GENERATE parameter
	## column), and `_refresh_phone_gen_panel()` shows exactly one. A loop that
	## takes the last `ScrollContainer` it finds silently measures whichever
	## happens to be second in the tree -- it took the GENERATE column and
	## reported its 1146 dp of content as if it were the sculpt sheet's.
	var scroll: ScrollContainer = app._phone_tool_scroll
	_log("[scrollers] tool vis=%s   gen vis=%s   tab=%s" % [
		str(scroll != null and scroll.visible),
		str(app._phone_gen_scroll != null and app._phone_gen_scroll.visible),
		app._phone_tab])
	if scroll == null:
		_log("[miss]    no ScrollContainer found under the sheet")
	else:
		_log("[miss]    sheet body scroller v=%d h=%d  v_max=%.0f h_max=%.0f at=%d" % [
			scroll.vertical_scroll_mode, scroll.horizontal_scroll_mode,
			scroll.get_v_scroll_bar().max_value, scroll.get_h_scroll_bar().max_value,
			scroll.scroll_vertical])
		_log("  scroll viewport h=%.1f px = %.2f dp   content h=%.1f px = %.2f dp" % [
			scroll.size.y, _dp(scroll.size.y),
			scroll.get_child(0).size.y, _dp(scroll.get_child(0).size.y)])
		var tor: Control = app.tool_options_row
		_log("  tool_options_row %s  children=%d" % [
			str(tor.get_global_rect()), tor.get_child_count()])
		var below := Vector2(gr.get_center().x, gr.end.y + 20.0)
		var v0 := scroll.scroll_vertical
		var hh0 := scroll.scroll_horizontal
		var r: Dictionary = await _drag_up(below, dy)
		_log("  drag 20 px below the handle  detent=%s  v %d->%d  h %d->%d" % [
			r["detent"], v0, scroll.scroll_vertical, hh0, scroll.scroll_horizontal])
	_log("RESULT %s  failures=%d" % ["PASS" if _fail == 0 else "FAIL", _fail])
	get_tree().quit(1 if _fail > 0 else 0)
