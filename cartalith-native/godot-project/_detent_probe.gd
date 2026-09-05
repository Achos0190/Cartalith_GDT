extends Node
## Lane C, 2026-09-05. Measures the phone bottom sheet's three detents against
## `design/dcc-environment-2026-08-31/spec/06-phone.md` 5.1/5.2, and drives
## 5.3's drag through the SubViewport's own GUI routing rather than by calling
## the handler.
##
##   godot --headless --path . _detent_probe.tscn -- --force-touch --vp 1080x2340 --tag p1080
##   godot --headless --path . _detent_probe.tscn -- --force-touch --vp 1440x3200 --tag p1440
##   godot --headless --path . _detent_probe.tscn -- --force-touch --vp 720x1600 --tag p720
##
## Flags this probe actually reads, grepped from the body below, not assumed:
##   `--vp WxH`      SubViewport size in physical px. Default 1080x2340.
##   `--tag NAME`    prefix on every output line. Default `det`.
##   `--force-touch` NOT read here -- it is read by `dcc_shell.gd`'s
##                   `_read_layout_env()` neighbourhood via
##                   `OS.get_cmdline_user_args()`, and without it the shell
##                   boots desktop and every measurement below is void.
## Anything else after `--` is rejected loudly rather than ignored, because a
## silently-defaulted size measures the same box three times and calls it three
## densities.
##
## Headless is sound here: nothing in this probe reads a pixel. The rule it
## would otherwise break -- `ImageTexture.update()` is a no-op under
## `--headless` -- does not apply to `Control.size`, which the layout pass
## computes either way.

var app: Node
var _vp: SubViewport
var _tag := "det"
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
	var args := OS.get_cmdline_user_args()
	var i := args.find(name)
	if i >= 0 and i + 1 < args.size():
		return String(args[i + 1])
	return dflt

## Refuse to run on an argument this probe does not read -- MISTAKES.md's
## "write a probe's usage header" row.
func _reject_unknown_args() -> bool:
	var known := ["--force-touch", "--vp", "--tag"]
	var args := OS.get_cmdline_user_args()
	for a in args:
		var s := String(a)
		if s.begins_with("--") and not (s in known):
			_log("ABORT unknown flag %s -- this probe reads only %s" % [s, str(known)])
			return false
	return true

func _dp(px: float) -> float:
	return px / float(app.phone_scale())

func _sheet_h() -> float:
	return (app._phone_tool_sheet as Control).size.y

## One mouse event through `SubViewport.push_input()` -- the viewport's real
## hit-test and `gui_input` dispatch, not a direct call on the handler.
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

## Drag from `from` by `dy` (negative = upward = taller sheet), sampling the
## height mid-gesture so a gesture that moves nothing cannot pass silently.
func _drag(from: Vector2, dy: float) -> Dictionary:
	var h0 := _sheet_h()
	_press(from, true)
	await _frames(1)
	var mid := from + Vector2(0.0, dy * 0.5)
	_motion(mid, Vector2(0.0, dy * 0.5))
	await _frames(2)
	var h_mid := _sheet_h()
	var to := from + Vector2(0.0, dy)
	_motion(to, Vector2(0.0, dy * 0.5))
	await _frames(2)
	_press(to, false)
	await _frames(1)
	await get_tree().create_timer(0.45).timeout
	await _frames(2)
	return {"h0": h0, "mid": h_mid, "end": _sheet_h(),
		"detent": String(app.phone_detent())}

func _ready() -> void:
	_tag = _arg("--tag", "det")
	if not _reject_unknown_args():
		get_tree().quit(2)
		return
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

	var s: float = app.phone_scale()
	_log("=== vp=%s phone=%s scale=%.4f REF_SHORT=%.1f ===" % [
		str(_vp.size), str(app.is_phone()), s, DccTheme.PHONE_REF_SHORT])
	if not app.is_phone():
		_log("ABORT not phone mode -- pass --force-touch")
		get_tree().quit(2)
		return

	# -- the reserve the detents subtract ------------------------------------
	var reserve: float = app._phone_nav_reserve()
	var bar: Control = app._phone_menu_bar
	var tl: Control = app.timeline_bar
	_log("[reserve]  spec navH = 84 dp (06-phone.md 3: 66 tab row + 18 gesture)")
	_log("  safe_bottom      px=%7.1f dp=%6.2f" % [
		float(app._safe_bottom()), _dp(float(app._safe_bottom()))])
	_log("  bottom nav       px=%7.1f dp=%6.2f vis=%s" % [
		float(app._ptap(DccTheme.H_PHONE_BOTTOM_NAV)),
		_dp(float(app._ptap(DccTheme.H_PHONE_BOTTOM_NAV))),
		str(bar != null and bar.visible)])
	_log("  timeline         px=%7.1f vis=%s" % [
		0.0 if tl == null else tl.size.y, str(tl != null and tl.visible)])
	_log("  nav_reserve      px=%7.1f dp=%6.2f" % [reserve, _dp(reserve)])

	# -- 5.2 -----------------------------------------------------------------
	var fh: float = float(_vp.size.y) - reserve
	var fh_dp := _dp(fh)
	var peek: float = app._phone_detent_height("peek")
	var half: float = app._phone_detent_height("half")
	var full: float = app._phone_detent_height("full")
	_log("[detents]  fh = vp.y - nav_reserve = %.1f px = %.2f dp" % [fh, fh_dp])
	_log("  peek  px=%7.1f dp=%7.2f   spec 66" % [peek, _dp(peek)])
	_log("  half  px=%7.1f dp=%7.2f   spec round(fh*0.46) = %.0f" % [
		half, _dp(half), round(fh_dp * 0.46)])
	_log("  full  px=%7.1f dp=%7.2f   spec fh-96          = %.0f" % [
		full, _dp(full), fh_dp - 96.0])
	_check(absf(_dp(peek) - 66.0) < 1.0, "peek within 1 dp of 66")
	_check(absf(_dp(half) - round(fh_dp * 0.46)) < 1.0,
		"half within 1 dp of round(fh*0.46)")
	_check(absf(_dp(full) - (fh_dp - 96.0)) < 1.0, "full within 1 dp of fh-96")
	_check(peek < half and half < full, "peek < half < full")

	# -- every way in ---------------------------------------------------------
	_log("[entrances]")
	_log("  boot             detent=%s h=%.1f" % [String(app.phone_detent()), _sheet_h()])
	_check(String(app.phone_detent()) == "peek",
		"boots at peek (divergence, stated at _phone_detent)")

	app.set_phone_detent("full")
	await get_tree().create_timer(0.45).timeout
	await _frames(2)
	_log("  set_phone_detent(full)   detent=%s h=%.1f" % [
		String(app.phone_detent()), _sheet_h()])
	_check(absf(_sheet_h() - full) < 2.0, "saved-layout route reaches full")

	app.set_phone_detent("nonsense")
	_check(String(app.phone_detent()) == "full", "setter rejects an unknown detent")

	app.set_phone_detent("peek")
	await get_tree().create_timer(0.45).timeout
	await _frames(2)

	## Tab route: a workspace tab lifts peek -> half; re-tapping it drops to peek.
	app._pick_phone_tab("gen")
	await get_tree().create_timer(0.45).timeout
	await _frames(2)
	_log("  tab tap gen              detent=%s h=%.1f" % [
		String(app.phone_detent()), _sheet_h()])
	_check(String(app.phone_detent()) == "half", "tab tap lifts peek -> half")
	app._pick_phone_tab("gen")
	await get_tree().create_timer(0.45).timeout
	await _frames(2)
	_log("  re-tap gen               detent=%s h=%.1f" % [
		String(app.phone_detent()), _sheet_h()])
	_check(String(app.phone_detent()) == "peek", "re-tap drops to peek")

	## MORE: the tab that does NOT drive the sheet.
	app._pick_phone_tab("more")
	await get_tree().create_timer(0.45).timeout
	await _frames(2)
	_log("  tab tap more             detent=%s h=%.1f menu_open=%s" % [
		String(app.phone_detent()), _sheet_h(),
		str(app._phone_menu != null and app._phone_menu.is_open())])
	_check(String(app.phone_detent()) == "peek", "MORE leaves the detent alone")
	app._pick_phone_tab("gen")
	app.set_phone_detent("peek")
	await get_tree().create_timer(0.45).timeout
	await _frames(2)

	# -- 5.3, through real routing -------------------------------------------
	_log("[drag]  SubViewport.push_input, hit-tested -- not a direct handler call")
	var grab: Control = app._phone_sheet_grab
	_log("  grab rect=%s h=%.1f px = %.2f dp (5.3 grabs the whole header block)" % [
		str(grab.get_global_rect()), grab.size.y, _dp(grab.size.y)])
	var c := grab.get_global_rect().get_center()
	var r1: Dictionary = await _drag(c, -(half - peek))
	_log("  up by half-peek          h0=%.1f mid=%.1f end=%.1f detent=%s" % [
		r1["h0"], r1["mid"], r1["end"], r1["detent"]])
	_check(absf(float(r1["mid"]) - float(r1["h0"])) > 20.0,
		"positive control: the sheet tracked the finger mid-gesture")
	_check(String(r1["detent"]) == "half", "release snapped to half")

	var c2: Vector2 = (app._phone_sheet_grab as Control).get_global_rect().get_center()
	var r2: Dictionary = await _drag(c2, -(full - half))
	_log("  up by full-half          h0=%.1f mid=%.1f end=%.1f detent=%s" % [
		r2["h0"], r2["mid"], r2["end"], r2["detent"]])
	_check(String(r2["detent"]) == "full", "release snapped to full")

	var c3: Vector2 = (app._phone_sheet_grab as Control).get_global_rect().get_center()
	var r3: Dictionary = await _drag(c3, full)
	_log("  down past dismiss        h0=%.1f mid=%.1f end=%.1f detent=%s" % [
		r3["h0"], r3["mid"], r3["end"], r3["detent"]])
	_check(String(r3["detent"]) == "peek",
		"below 44 dp collapses to peek, never to gone (divergence, stated)")

	## Negative control: the same gesture above the handle must move nothing.
	var away := Vector2(c3.x, maxf(4.0, c3.y - 200.0))
	var h_before := _sheet_h()
	var r4: Dictionary = await _drag(away, -300.0)
	_log("  off-handle drag at %s h0=%.1f end=%.1f" % [str(away), r4["h0"], r4["end"]])
	_check(absf(float(r4["end"]) - h_before) < 2.0,
		"negative control: a drag off the handle moves the sheet 0 px")

	## The handle pill's own geometry. Added 2026-09-05 after a verifier
	## mutated `_pscale(42)` -> `_pscale(40)` in `dcc_shell.gd` and this probe
	## still reported PASS: the one constant the detent row introduced was
	## asserted by nothing.
	##
	## The expected figure is recomputed here from the probe's OWN literals --
	## 42 x 4 reference dp over the 412 dp canvas width -- rather than read back
	## from `_pscale()`, so this is not a constant asserted against itself. A
	## mutant of either literal in the shell moves the measured px and fails.
	var pill: Control = null
	for ch in (app._phone_sheet_grab as Control).get_children():
		if ch is ColorRect:
			pill = ch as Control
			break
	if pill == null:
		_check(false, "handle pill: the grab row has no ColorRect child")
	else:
		var vw := float(app.get_viewport_rect().size.x)
		var want := Vector2(roundf(42.0 * vw / 412.0), roundf(4.0 * vw / 412.0))
		_log("  handle pill  got=%.0fx%.0f want=%.0fx%.0f  (42 x 4 dp at %.0f px wide)"
			% [pill.size.x, pill.size.y, want.x, want.y, vw])
		_check(absf(pill.size.x - want.x) <= 1.0 and absf(pill.size.y - want.y) <= 1.0,
			"handle pill is 42 x 4 dp")

	# -- what else the shared inset pass moves --------------------------------
	_log("[shared]  _apply_phone_orientation() is the shared pass, not apply_insets()")
	var menu: Node = app._phone_menu
	var probe_nodes := {
		"app bar": app._phone_app_bar, "bottom nav": bar,
		"timeline": tl, "left dock": app.left_dock, "right dock": app.right_dock,
		"phone menu": menu,
	}
	var before := {}
	for k in probe_nodes:
		var n: Control = probe_nodes[k]
		before[k] = Rect2() if n == null else n.get_global_rect()
	app.set_phone_detent("full")
	await get_tree().create_timer(0.45).timeout
	await _frames(3)
	for k in probe_nodes:
		var n: Control = probe_nodes[k]
		if n == null:
			_log("  %-14s ABSENT" % k)
			continue
		var d: Rect2 = n.get_global_rect()
		_log("  %-14s before=%s after=%s moved=%s" % [
			k, str(before[k]), str(d), str(before[k] != d)])

	_log("RESULT %s  failures=%d" % ["PASS" if _fail == 0 else "FAIL", _fail])
	get_tree().quit(0 if _fail == 0 else 1)
