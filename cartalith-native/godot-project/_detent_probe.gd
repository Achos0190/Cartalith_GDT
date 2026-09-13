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

## Depth-first, first match -- used to find the handle pill wherever it sits
## under `_phone_sheet_grab`, rather than assuming a fixed nesting depth.
func _find_colorrect(root: Node) -> Control:
	if root is ColorRect:
		return root as Control
	for c in root.get_children():
		var found := _find_colorrect(c)
		if found != null:
			return found
	return null

## Every `Button` under `root` whose own text matches `label`, depth-first --
## mirrors `_navcut_probe.gd`'s own `_buttons_with_text` rather than assuming
## a fixed parent path (`.claude/resume-2026-09-12/peek_pin_2026-09-12.patch`'s
## own `_find_button`, the same technique, reused for the header proof below).
func _find_button(root: Node, label: String) -> Button:
	if root is Button and String((root as Button).text) == label:
		return root as Button
	for c in root.get_children():
		var found := _find_button(c, label)
		if found != null:
			return found
	return null

## Every `Button` under `root`, depth-first -- for counting rows a probe does
## not know the exact text of ahead of time (the mode-segment PIPELINE/SCULPT
## pair is one; the label wording is `world_workspace.gd`'s, not this probe's
## business to hard-code more of than the one already-asserted case needs).
func _all_buttons(root: Node, out: Array) -> void:
	if root is Button:
		out.append(root)
	for c in root.get_children():
		_all_buttons(c, out)

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

	## **Gate B3, and it has to run HERE -- cold boot, before anything else in
	## this file moves the detent.** The refuted header attempt
	## (`.claude/resume-2026-09-12/peek_pin_2026-09-12.patch`) covered 29 px of
	## the bottom nav and hid the coordinate/scale labels at exactly this
	## moment, because its taller peek pushed into the map's own corner
	## readouts before the user had done anything at all. Measured here as
	## drawn rects (`get_global_rect()`), windowed (this probe runs
	## `--headless`, sound for layout per this file's own header -- these are
	## `Control.size`/`position` reads, nothing rasterised) rather than
	## inferred from the scene tree, and stated as what it is: a synthetic
	## boot, not a finger on a device.
	var navpad: Control = app._phone_menu_bar
	var vh = app._find_viewport_host()
	var coords: Control = vh._coords_label if vh != null else null
	var scale_lbl: Control = vh._scale_label if vh != null else null
	var header0: Rect2 = (app._phone_sheet_grab as Control).get_global_rect()
	var sheet0: Rect2 = (app._phone_tool_sheet as Control).get_global_rect()
	_log("  [B3] cold boot: header=%s sheet=%s" % [str(header0), str(sheet0)])
	if navpad != null:
		var np: Rect2 = navpad.get_global_rect()
		_log("       navpad=%s  overlap=%s" % [str(np), str(sheet0.intersection(np))])
		_check(sheet0.intersection(np).size.y < 1.0,
			"B3: the sheet does not cover the bottom navpad at cold boot")
	if coords != null:
		var cr: Rect2 = coords.get_global_rect()
		_log("       coords=%s  overlap=%s" % [str(cr), str(sheet0.intersection(cr))])
		_check(sheet0.intersection(cr).size.y < 1.0 or sheet0.intersection(cr).size.x < 1.0,
			"B3: the sheet does not cover the coordinate label at cold boot")
	if scale_lbl != null:
		var sr: Rect2 = scale_lbl.get_global_rect()
		_log("       scale=%s  overlap=%s" % [str(sr), str(sheet0.intersection(sr))])
		_check(sheet0.intersection(sr).size.y < 1.0 or sheet0.intersection(sr).size.x < 1.0,
			"B3: the sheet does not cover the scale label at cold boot")

	## Gate B2: every outer-height change still routes through the one signal
	## the shared inset pass depends on. Not a new path -- this pass touched
	## the header's OWN content and its `custom_minimum_size` floor, never
	## `_phone_sheet_tween` or its `finished` connection
	## (`phone_insets_changed.emit()`) -- so a real regression here would mean
	## the floor above is fighting the tween, not merely riding along with it.
	## `[0]`, not a plain `int` -- GDScript lambdas capture an outer local BY
	## VALUE, so `func(): insets_fires += 1` over a bare `int` would mutate
	## only the lambda's own copy and this count would read back 0 no matter
	## what fired. An `Array`'s CONTENTS are still the same shared object.
	var insets_fires := [0]
	app.phone_insets_changed.connect(func(): insets_fires[0] += 1)
	app.set_phone_detent("full")
	await get_tree().create_timer(0.45).timeout
	await _frames(2)
	_log("  set_phone_detent(full)   detent=%s h=%.1f  phone_insets_changed x%d" % [
		String(app.phone_detent()), _sheet_h(), insets_fires[0]])
	_check(absf(_sheet_h() - full) < 2.0, "saved-layout route reaches full")
	_check(insets_fires[0] >= 1,
		"B2: the detent change to full emitted phone_insets_changed")

	## S-FULL, new 2026-09-13: the other half of S-HALF below -- does "full"
	## actually cover the map's corner readouts, or does the insets pass keep
	## up with it too? Answered, not assumed: at 1080x2340 the pushed-up
	## coordinate/scale labels land at y=311/319, and "full"'s own top edge is
	## y=251 -- ABOVE them, so the sheet still reaches past wherever the
	## insets pass can push a label before it runs out of safe area above.
	## The overlap measured is the label's FULL size, not a sliver.
	var sheet_full: Rect2 = (app._phone_tool_sheet as Control).get_global_rect()
	_log("  [S-FULL] sheet=%s" % str(sheet_full))
	if coords != null:
		var cr_f: Rect2 = coords.get_global_rect()
		_log("       coords=%s overlap=%s" % [str(cr_f), str(sheet_full.intersection(cr_f))])
		var ov_c: Vector2 = sheet_full.intersection(cr_f).size
		## Component-wise, not `Vector2 >=` -- that operator is LEXICOGRAPHIC
		## (compares x, only falls to y on a tie), not "both axes at least
		## this large", and an intersection can never exceed the label's own
		## size regardless, so this is really an equality check.
		_check(ov_c.x >= cr_f.size.x and ov_c.y >= cr_f.size.y,
			"S-FULL: full DOES fully cover the coordinate label (by design -- nowhere left to push it)")
	if scale_lbl != null:
		var sr_f: Rect2 = scale_lbl.get_global_rect()
		_log("       scale=%s overlap=%s" % [str(sr_f), str(sheet_full.intersection(sr_f))])
		var ov_s: Vector2 = sheet_full.intersection(sr_f).size
		_check(ov_s.x >= sr_f.size.x and ov_s.y >= sr_f.size.y,
			"S-FULL: full DOES fully cover the scale label (by design -- nowhere left to push it)")

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

	## S-HALF, new 2026-09-13: does "half" cover the map's own corner readouts,
	## the same question S3 below asks of "peek" and "full"? Nothing in this
	## file had ever checked it -- the S3 comment below used to just ASSERT
	## "half"/"full" both do, by the same reasoning that turned out to only
	## hold for one of them (see that comment's own correction). Re-read here,
	## not assumed from the B3 cold-boot capture: `phone_insets_changed`
	## (Gate B2 above) exists so `ViewportHost`'s HUD can reposition as the
	## sheet grows, so unlike the sheet's own bottom-anchored rect, these
	## labels' Y is not constant across detents.
	var sheet_half: Rect2 = (app._phone_tool_sheet as Control).get_global_rect()
	_log("  [S-HALF] sheet=%s" % str(sheet_half))
	if navpad != null:
		var np_h: Rect2 = navpad.get_global_rect()
		_log("       navpad=%s overlap=%s" % [str(np_h), str(sheet_half.intersection(np_h))])
		_check(sheet_half.intersection(np_h).size.y < 1.0,
			"S-HALF: half does not cover the bottom navpad")
	if coords != null:
		var cr_h: Rect2 = coords.get_global_rect()
		_log("       coords=%s overlap=%s" % [str(cr_h), str(sheet_half.intersection(cr_h))])
		_check(sheet_half.intersection(cr_h).size.y < 1.0 or sheet_half.intersection(cr_h).size.x < 1.0,
			"S-HALF: half does not cover the coordinate label -- the insets pass keeps it clear")
	if scale_lbl != null:
		var sr_h: Rect2 = scale_lbl.get_global_rect()
		_log("       scale=%s overlap=%s" % [str(sr_h), str(sheet_half.intersection(sr_h))])
		_check(sheet_half.intersection(sr_h).size.y < 1.0 or sheet_half.intersection(sr_h).size.x < 1.0,
			"S-HALF: half does not cover the scale label -- the insets pass keeps it clear")

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
	## Recursive, not `get_children()` -- `MISTAKES.md`'s "enumerate a surface"
	## row. The pill used to be `_phone_sheet_grab`'s only, direct child; since
	## 2026-09-13 the grab is the whole header block and the pill sits inside
	## its own `pill_row` sub-`Control`, two levels down. A shallow walk would
	## silently find nothing and read as "no pill" rather than "wrong depth".
	var pill: Control = _find_colorrect(app._phone_sheet_grab as Control)
	if pill == null:
		_check(false, "handle pill: the grab row has no ColorRect child")
	else:
		var vw := float(app.get_viewport_rect().size.x)
		var want := Vector2(roundf(42.0 * vw / 412.0), roundf(4.0 * vw / 412.0))
		_log("  handle pill  got=%.0fx%.0f want=%.0fx%.0f  (42 x 4 dp at %.0f px wide)"
			% [pill.size.x, pill.size.y, want.x, want.y, vw])
		_check(absf(pill.size.x - want.x) <= 1.0 and absf(pill.size.y - want.y) <= 1.0,
			"handle pill is 42 x 4 dp")

	# -- Part B, 2026-09-13: the header block's own proof ---------------------
	# `OUTSTANDING_WORK.md`'s "half a body row shows at peek, PIPELINE/SCULPT
	# chips cut at 52 of 115 px" -- `dcc_shell.gd::_build_phone_tool_sheet()`'s
	# own header has the fix and the sizing argument; this proves the OUTCOME.
	_log("[header]  sheetTitle/sheetSub/hSheetClose exist; peek shows none of the body")
	_check(app._phone_sheet_title != null and String(app._phone_sheet_title.text) != "",
		"sheetTitle is built and non-empty")
	_check(app._phone_sheet_subtitle != null and String(app._phone_sheet_subtitle.text) != "",
		"sheetSub is built and non-empty")
	var close_btn := _find_button(app._phone_sheet_grab as Control, "✕")
	_check(close_btn != null, "hSheetClose (the close circle) exists in the header")

	## What the user SEES, not only what the tree contains -- MISTAKES.md's own
	## row: a header check once passed with the whole header removed, and a
	## phone check once read a hidden label's text. `is_visible_in_tree()`, and
	## an intersection against the SHEET's own drawn rect rather than the
	## label's unclipped `get_global_rect()` alone, which proves nothing about
	## what is actually shown.
	var header_sheet_rect: Rect2 = (app._phone_tool_sheet as Control).get_global_rect()
	var title_rect: Rect2 = app._phone_sheet_title.get_global_rect()
	var sub_rect: Rect2 = app._phone_sheet_subtitle.get_global_rect()
	_log("  title  visible=%s rect=%s  |  sub  visible=%s rect=%s  |  sheet=%s" % [
		str(app._phone_sheet_title.is_visible_in_tree()), str(title_rect),
		str(app._phone_sheet_subtitle.is_visible_in_tree()), str(sub_rect),
		str(header_sheet_rect)])
	_check(app._phone_sheet_title.is_visible_in_tree(), "sheetTitle is visible in tree")
	_check(app._phone_sheet_subtitle.is_visible_in_tree(), "sheetSub is visible in tree")
	_check(title_rect.size.y > 1.0
			and header_sheet_rect.intersection(title_rect).size.y >= title_rect.size.y - 1.0,
		"sheetTitle's drawn rect is non-empty and inside the sheet's visible rect")
	_check(sub_rect.size.y > 1.0
			and header_sheet_rect.intersection(sub_rect).size.y >= sub_rect.size.y - 1.0,
		"sheetSub's drawn rect is non-empty and inside the sheet's visible rect")
	## `_pfont()`-scaled, not the raw `DccTheme.FS_SMALL`/`FS_MICRO` CSS px --
	## FAILS if the fonts are ever reverted to unscaled: 11/9.5 raw px is well
	## under 90% of `_pfont(11)`/`_pfont(9.5)` at any phone scale above 1.0
	## (this device: %.3f).
	_log("  title h=%.1f want>=%.1f (90%% of _pfont(11)=%d, scale=%.3f)  sub h=%.1f want>=%.1f (90%% of _pfont(9.5)=%d)" % [
		title_rect.size.y, float(app._pfont(11)) * 0.9, app._pfont(11), app.phone_scale(),
		sub_rect.size.y, float(app._pfont(9.5)) * 0.9, app._pfont(9.5)])
	_check(title_rect.size.y >= float(app._pfont(11)) * 0.9,
		"sheetTitle draws at >= 90% of the scaled font size, not the raw constant")
	_check(sub_rect.size.y >= float(app._pfont(9.5)) * 0.9,
		"sheetSub draws at >= 90% of the scaled font size, not the raw constant")

	## Title tracking, 2026-09-13: `.2em` of the ACTUAL rendered title size,
	## not a bare `2` -- read back off the real `FontVariation` the running
	## label holds, not re-derived from `dcc_shell.gd`'s own formula (that
	## would just assert the code against itself).
	var title_font: Font = app._phone_sheet_title.get_theme_font("font")
	if title_font is FontVariation:
		var spacing: int = (title_font as FontVariation).spacing_glyph
		var want_spacing: int = maxi(1, roundi(float(app._pfont(11)) * 0.2))
		_log("  title spacing_glyph=%d  want~=%d (.2em of _pfont(11)=%d)" % [
			spacing, want_spacing, app._pfont(11)])
		_check(spacing == want_spacing,
			"sheetTitle's FontVariation carries .2em of the SCALED font size, not a raw literal")
		_check(spacing > 2,
			"sheetTitle tracking is scaled (>2 raw px) at this phone's density, not the old bare DccTheme.mono(2)")
	else:
		_check(false, "sheetTitle's font is a FontVariation (spacing_glyph readable)")

	## Live GENERATE wiring, 2026-09-13: a real world so the printed seed is a
	## real number, not the "no world yet" fallback. `bridge.generate()` is the
	## same entry point every other probe in this project uses to raise one,
	## not a shortcut invented for this file.
	app.bridge.generate({"seed": 77341, "width_km": 600.0, "grid_w": 128, "grid_h": 96,
		"sea_level": 0.5, "villages": false})
	await app.bridge.generation_finished
	await _frames(10)
	_check(app.bridge.has_world, "a world landed for the header's live seed read")

	## S3, extended: B3's own cold-boot invariant (the sheet does not cover the
	## navpad/coords/scale), re-measured now that a world -- and its readouts --
	## exist. "half"/"full" are not RE-checked here -- their target heights
	## come from the same `_phone_detent_height()` formula this batch never
	## touched -- but they ARE checked, above (`S-HALF`/`S-FULL`), and **not
	## alike: only "full" covers the map's own corner readouts. "half" does
	## not.**
	##
	## **Corrected 2026-09-13 -- this comment used to claim both "by design",
	## and nothing in this file had ever measured "half" against these labels
	## to back that half of it.** `phone_insets_changed` (Gate B2 above) pushes
	## the coordinate/scale labels up as the sheet grows; at "half" that keeps
	## them fully clear (measured at 1080x2340: zero overlap, both labels,
	## the same invariant "peek" already holds). Only at "full" does the
	## sheet's own top edge land ABOVE wherever the insets pass has pushed the
	## labels to -- there is no more safe area left above them to retreat
	## into -- and there the overlap is the label's FULL size, not a sliver.
	## So the claim was right for "full" and simply untested for "half";
	## asserting non-coverage at "full" would indeed assert something false,
	## and asserting non-coverage at "half" is exactly the invariant that was
	## missing. "peek" is the one detent whose CONTENT changed this batch.
	app.set_phone_detent("peek")
	await get_tree().create_timer(0.45).timeout
	await _frames(2)
	var sheet_world: Rect2 = (app._phone_tool_sheet as Control).get_global_rect()
	if navpad != null:
		_check(sheet_world.intersection(navpad.get_global_rect()).size.y < 1.0,
			"S3: peek does not cover the bottom navpad, world generated")
	if coords != null:
		var cr2: Rect2 = coords.get_global_rect()
		_check(sheet_world.intersection(cr2).size.y < 1.0 or sheet_world.intersection(cr2).size.x < 1.0,
			"S3: peek does not cover the coordinate label, world generated")
	if scale_lbl != null:
		var sr2: Rect2 = scale_lbl.get_global_rect()
		_check(sheet_world.intersection(sr2).size.y < 1.0 or sheet_world.intersection(sr2).size.x < 1.0,
			"S3: peek does not cover the scale label, world generated")

	## Per-tab header strings against the canvas's own `titles` table
	## (`AND:1466`) and `_moreTitle()` (`AND:1208`) -- quoted in full in
	## `_refresh_phone_sheet_header()`'s own doc comment.
	app.select_domain_mode("world", "a")
	app._pick_phone_tab("gen")
	await _frames(2)
	_log("  [gen/pipe]   title=%s  sub=%s" % [
		app._phone_sheet_title.text, app._phone_sheet_subtitle.text])
	_check(String(app._phone_sheet_title.text) == "GENERATE", "GEN/pipe title is GENERATE")
	_check(String(app._phone_sheet_subtitle.text) == "pipeline · seed 77341",
		"GEN/pipe subtitle carries the live seed, verbatim against AND:1466's gen row")

	app.select_domain_mode("world", "b")
	await _frames(2)
	_log("  [gen/sculpt] title=%s  sub=%s" % [
		app._phone_sheet_title.text, app._phone_sheet_subtitle.text])
	_check(String(app._phone_sheet_title.text) == "GENERATE", "GEN/sculpt title is GENERATE")
	_check(String(app._phone_sheet_subtitle.text) == "sculpt · draft stamps",
		"GEN/sculpt subtitle matches AND:1466's gen row verbatim")
	app.select_domain_mode("world", "a")   ## leave pipe mode for the rest of this file

	app._pick_phone_tab("map")
	await _frames(2)
	_log("  [map]        title=%s  sub=%s" % [
		app._phone_sheet_title.text, app._phone_sheet_subtitle.text])
	_check(String(app._phone_sheet_title.text) == "MAP", "MAP title matches AND:1466")
	_check(String(app._phone_sheet_subtitle.text) == "layers · style · annotation",
		"MAP subtitle matches AND:1466's map row verbatim")

	app._pick_phone_tab("more")
	await _frames(2)
	_log("  [more]       title=%s  sub=%s  menu_open=%s" % [
		app._phone_sheet_title.text, app._phone_sheet_subtitle.text,
		str(app._phone_menu != null and app._phone_menu.is_open())])
	_check(String(app._phone_sheet_title.text) == "MORE", "MORE title matches AND:1208 root")
	_check(String(app._phone_sheet_subtitle.text) == "program · data · preferences",
		"MORE subtitle matches _moreTitle()'s root entry verbatim (AND:1208)")

	## PLAN's header, 2026-09-13: `journey_planner_view.gd::phone_header_info()`
	## is new, closing the gap this exact per-tab section's neighbourhood
	## already tests for MAP/GEN/MORE. Poked directly on the live instance
	## rather than driven through `_pick_phone_tab("plan")` -- that call also
	## fires `open_journey_planner()`'s full region takeover (arms a tool,
	## swaps the right dock), which is not what a header STRING assertion
	## needs and would leave more state to restore afterward than
	## `_phone_tab`/`_refresh_phone_sheet_header()` alone. `app.journey_planner_
	## view` is reached directly (this file already reaches `app._phone_tab`,
	## `app._phone_menu` etc. the same way).
	var jp = app.journey_planner_view
	_check(jp != null and jp.has_method("phone_header_info"),
		"journey_planner_view exposes phone_header_info()")
	if jp != null:
		var prior_tab: String = app._phone_tab
		app._phone_tab = "plan"
		jp._last_result = {}
		jp._isolated_stage = -1
		app._refresh_phone_sheet_header()
		_log("  [plan/none]  title=%s  sub=%s" % [
			app._phone_sheet_title.text, app._phone_sheet_subtitle.text])
		_check(String(app._phone_sheet_title.text) == "PLAN",
			"PLAN/no route: title falls through to PHONE_TABS' static PLAN")
		_check(String(app._phone_sheet_subtitle.text) == "Journey planner",
			"PLAN/no route: subtitle falls through to PHONE_TABS' static tip, not a manufactured \"journey · journey → journey\"")

		jp._last_result = {"ok": true, "plan": {"stops": [
			{"name": "Vhal Serai"}, {"name": "Amre Ford"}, {"name": "Port Amre"},
		]}}
		jp._isolated_stage = -1
		app._refresh_phone_sheet_header()
		_log("  [plan/route] title=%s  sub=%s" % [
			app._phone_sheet_title.text, app._phone_sheet_subtitle.text])
		_check(String(app._phone_sheet_title.text) == "PLAN",
			"PLAN/committed route, no stage isolated: title stays PLAN")
		_check(String(app._phone_sheet_subtitle.text) == "journey · Vhal Serai → Port Amre",
			"PLAN/committed route: subtitle is journey · first stop -> last stop, off plan.stops")

		jp._isolated_stage = 1
		app._refresh_phone_sheet_header()
		_log("  [plan/stage] title=%s  sub=%s" % [
			app._phone_sheet_title.text, app._phone_sheet_subtitle.text])
		_check(String(app._phone_sheet_title.text) == "PLAN · STAGE 2",
			"PLAN/stage 1 isolated (0-based): title is PLAN · STAGE 2, matching AND:1466's planView==='stage' branch")
		_check(String(app._phone_sheet_subtitle.text) == "journey · Vhal Serai → Port Amre",
			"PLAN/stage isolated: endpoints are unaffected by which stage is isolated")

		## Restore -- this poked `journey_planner_view`'s own state directly,
		## and every OTHER check below this point expects a clean planner.
		jp._last_result = {}
		jp._isolated_stage = -1
		app._phone_tab = prior_tab

	app._pick_phone_tab("gen")
	app.set_phone_detent("peek")
	await get_tree().create_timer(0.45).timeout
	await _frames(2)
	var gen_scroll: Control = app._phone_gen_scroll
	## `PIPELINE`, not a generic "first row" -- the exact chip
	## `OUTSTANDING_WORK.md` named, found the same way
	## `.claude/resume-2026-09-12/peek_pin_2026-09-12.patch`'s own `_find_button`
	## did for the same control.
	var seg_btn := _find_button(app._phone_tool_sheet as Control, "PIPELINE")
	_check(gen_scroll != null and gen_scroll.visible, "the GENERATE body scroll is the visible one")
	_check(seg_btn != null, "PIPELINE mode-segment chip exists under the sheet")
	if gen_scroll != null and seg_btn != null:
		var seg_full_h: float = seg_btn.get_global_rect().size.y
		## The scroll container's OWN rect is what actually clips -- the
		## chip's `get_global_rect()` is unclipped, natural-position geometry
		## regardless of what its ScrollContainer ancestor shows on screen
		## (`MISTAKES.md`: a `ScrollContainer` does not fold a child's minimum
		## into its own, and the converse holds too -- the child's rect does
		## not know it is being clipped). Intersecting the two is what "drawn
		## rect intersects the sheet's visible rect" actually has to mean.
		var seg_visible_peek: Rect2 = gen_scroll.get_global_rect().intersection(seg_btn.get_global_rect())
		_log("  peek: header h=%.1f  sheet h=%.1f  body scroll h=%.1f  PIPELINE chip %.1f of %.1f px visible" % [
			(app._phone_sheet_grab as Control).size.y, _sheet_h(), gen_scroll.size.y,
			seg_visible_peek.size.y, seg_full_h])
		_check(seg_visible_peek.size.y < 2.0,
			"peek: no body row's drawn rect meaningfully intersects the sheet's visible rect")

		for det in ["half", "full"]:
			app.set_phone_detent(det)
			await get_tree().create_timer(0.45).timeout
			await _frames(2)
			var seg_visible: Rect2 = gen_scroll.get_global_rect().intersection(seg_btn.get_global_rect())
			_log("  %s: PIPELINE chip %.1f of %.1f px visible" % [det, seg_visible.size.y, seg_full_h])
			_check(absf(seg_visible.size.y - seg_full_h) < 1.0,
				"%s: the mode-segment chip is fully visible (115/115-equivalent), not clipped" % det)

		app.set_phone_detent("peek")
		await get_tree().create_timer(0.45).timeout
		await _frames(2)

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
