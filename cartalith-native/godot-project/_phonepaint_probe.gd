extends Node
## **Phone: a thumb can arm a paint tool AND reach its options row.**
##
## Protects (OUTSTANDING_WORK.md row "Phone: the Paint tool bar cannot be reached
## by thumb"; Ruling BR's Original slider is the control that was unreachable):
##  1. the Paint controls are reached by REAL taps through the bottom nav and the
##     GENERATE sheet's own segment -- this probe never writes `_phone_tab`, never
##     calls `_pick_phone_tab` / `_refresh_phone_gen_panel` / `arm_tool`, and
##     never calls a handler. `_paintorig_probe.gd`'s phone leg did, with a
##     "FORCED REVEAL (not user-reachable)" log, which is why this row stayed
##     open: a probe that writes the navigation state cannot see a missing route.
##     Remove the PAINT segment, or Original, or Size, from the sheet and this
##     probe goes RED (mutation-checked, see the run log in the lane report);
##  2. what is reached is usable: each control's DRAWN rect lies inside the
##     scroller's visible rect (`get_global_rect()` reports the UNCLIPPED rect --
##     MISTAKES.md), not clipped at the right edge, and at least 44 dp tall;
##  3. the controls are wired to the same state as the tool bar: Original moves
##     `paint_original.opacity` and the pixels actually drawn over the map, Size
##     moves the brush radius, Erase / Land only / Class / Target field write the
##     shared `_paint_brush` / `_paint_layer`, Splat (no raster) shows the reason
##     instead of a dead slider, and COMMIT / DISCARD enable on the pending draft;
##  3b. (2026-10-04, phone-paint remainders) three more routes, each by real taps,
##     drags and a real key press: (i) the map's mode chip reads `PAINT · DRAFT`
##     while Paint is armed and stops saying so when it is not
##     (`dcc_shell.gd::_refresh_viewport_context`, refreshed by `tool_armed`);
##     (ii) Original is drawn INERT (`editable == false`) with its reason once
##     another tool is armed -- an Escape key press disarms Paint to Inspect --
##     and a drag on it moves nothing, and ARM PAINT makes it live again;
##     (iii) Hardness and Softness sliders sit in the column, reach the shared
##     `_paint_brush`, and moving one does not move the other. Remove the chip
##     branch, the `tool_armed` hook, the inert gate or the two sliders and this
##     probe goes RED;
##  4. nothing else moved: the SCULPT and PIPELINE segments, and the MAP, PLAN
##     and MORE tabs, still show what they showed, and a segment chosen by the
##     user is not clobbered by re-tapping the tab.
##
## **Phone only.** On desktop and tablet the sheet is not built
## (`_phone_gen_scroll == null`); that leg asserts the absence and exits, rather
## than reporting a vacuous pass. Run windowed (pixel readback is a no-op under
## `--headless`):
##   godot --path . _phonepaint_probe.tscn -- --force-touch --vp 1080x2340 --tag phone --shotdir DIR
##
## Palette: both palettes are forced and the forced state is VERIFIED before any
## pixel threshold is read; the drawn-fill check compares against the live
## `DccTheme.c("accent")`, never a colour written for one palette.

var app: Node
var bridge: EngineBridge
var _vp: SubViewport
var _tag := "phonepaint"
var _shotdir := ""
var _fail := 0
var _checks := 0

const SEED := 4242
const GRID_W := 256
const GRID_H := 164
const SETTLE := 8   ## frames for the shell to lay out and redraw after an input

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _log(s: String) -> void:
	print("[%s] %s" % [_tag, s])

func _check(ok: bool, what: String) -> void:
	_checks += 1
	if not ok:
		_fail += 1
	_log("  %-4s %s" % ["OK" if ok else "FAIL", what])

func _arg(name: String, dflt: String) -> String:
	var args := OS.get_cmdline_user_args()
	var i := args.find(name)
	if i >= 0 and i + 1 < args.size():
		return String(args[i + 1])
	return dflt

func _reject_unknown_args() -> bool:
	var known := ["--force-touch", "--vp", "--tag", "--shotdir"]
	for a in OS.get_cmdline_user_args():
		var s := String(a)
		if s.begins_with("--") and not (s in known):
			_log("ABORT unknown flag %s -- this probe reads only %s" % [s, str(known)])
			return false
	return true

func _finish() -> void:
	_log("RESULT %s checks=%d fail=%d" % [_tag, _checks, _fail])
	get_tree().quit(1 if _fail > 0 else 0)

func _guard() -> bool:
	var img := Image.create_empty(2, 2, false, Image.FORMAT_RGBA8)
	img.fill(Color(0, 0, 0, 0))
	var t := ImageTexture.create_from_image(img)
	img.set_pixel(1, 1, Color(1, 0, 0, 1))
	t.update(img)
	return t.get_image().get_pixel(1, 1).a > 0.0

# -- Real input ------------------------------------------------------------------

func _mouse(at: Vector2, pressed: bool) -> void:
	var e := InputEventMouseButton.new()
	e.button_index = MOUSE_BUTTON_LEFT
	e.pressed = pressed
	e.position = at
	e.global_position = at
	_vp.push_input(e, true)

## A tap: press and release at one point, through the viewport's own hit-test.
func _tap(at: Vector2) -> void:
	_mouse(at, true)
	await _frames(1)
	_mouse(at, false)
	await _frames(SETTLE)

## A horizontal press-and-drag ending at `to`, the way a thumb sets a slider:
## down on the track, a few motion samples, up. The first samples travel
## sideways, so `PgSlider` classifies it as a value drag, not a scroll.
func _drag_to(from: Vector2, to: Vector2) -> void:
	_mouse(from, true)
	await _frames(1)
	var prev := from
	for k in range(1, 5):
		var at := from.lerp(to, float(k) / 4.0)
		var mm := InputEventMouseMotion.new()
		mm.position = at
		mm.global_position = at
		mm.relative = at - prev
		mm.button_mask = MOUSE_BUTTON_MASK_LEFT
		_vp.push_input(mm, true)
		prev = at
		await _frames(1)
	_mouse(to, false)
	await _frames(SETTLE)

# -- Finding what a person reads ----------------------------------------------------

func _walk(root: Node, out: Array) -> void:
	for c in root.get_children():
		if c is Control and not (c as Control).is_visible_in_tree():
			continue
		if c is Control:
			out.append(c)
		_walk(c, out)

func _visible(root: Node) -> Array:
	var out: Array = []
	_walk(root, out)
	return out

## The visible `Button` under `root` whose text is exactly `text`, or null.
func _button(root: Node, text: String) -> Button:
	for c in _visible(root):
		if c is Button and String((c as Button).text) == text:
			return c
	return null

## The visible `Label` under `root` whose text starts with `prefix`, or null.
func _label(root: Node, prefix: String) -> Label:
	for c in _visible(root):
		if c is Label and String((c as Label).text).begins_with(prefix):
			return c
	return null

## A bottom-nav cell, found by reading its caption (never by `_phone_tab_cells`).
func _nav_cell(caption: String) -> Button:
	var l := _label(app._phone_menu_bar, caption)
	var p: Node = l
	while p != null:
		if p is Button:
			return p
		p = p.get_parent()
	return null

## The sheet column's slider whose label reads `label`: the `Paint column`'s
## slider sits under a head row whose first child is that label.
func _slider(label: String) -> HSlider:
	for c in _visible(app._phone_gen_scroll):
		if c is HSlider:
			var wrap := (c as Node).get_parent()
			if wrap != null and wrap.get_child_count() > 0:
				var head := wrap.get_child(0)
				if head.get_child_count() > 0 and head.get_child(0) is Label \
						and (head.get_child(0) as Label).text == label:
					return c
	return null

func _texts() -> Array:
	var out: Array = []
	for c in _visible(app._phone_gen_scroll):
		if c is Label:
			out.append(String((c as Label).text))
		elif c is Button:
			out.append(String((c as Button).text))
	return out

## Wait for the sheet's detent tween to land: the scroller is taller than a peek
## strip (>= 300 px, the half detent is ~700 px on a 2340 px screen) and its
## height is stable across 3 consecutive frames. Bounded at 40 frames, so a stuck
## sheet FAILS the checks after it instead of hanging the run. Stability alone is
## not enough: the height can sit still for a few frames before the tween starts.
func _sheet_settled() -> void:
	var last := -1.0
	var same := 0
	for i in 40:
		await get_tree().process_frame
		var h: float = app._phone_gen_scroll.get_global_rect().size.y
		same = same + 1 if is_equal_approx(h, last) else 0
		last = h
		if same >= 3 and h >= 300.0:
			return

func _pg_dp_px(dp: float) -> int:
	return int(round(dp * float(app.phone_scale())))

func _dp(px: float) -> float:
	return px / maxf(0.001, float(app.phone_scale()))

## `get_global_rect()` is the UNCLIPPED rect; the claim "reachable" needs the
## part the scroller actually draws. True when `c` lies wholly inside `clip`.
func _inside(c: Control, clip: Control) -> bool:
	var cr := c.get_global_rect()
	var kr := clip.get_global_rect()
	return cr.position.x >= kr.position.x - 0.5 and cr.end.x <= kr.end.x + 0.5 \
		and cr.position.y >= kr.position.y - 0.5 and cr.end.y <= kr.end.y + 0.5

## Bring `c` fully into the scroller's visible rect. DISCLOSED STAND-IN: this
## writes `scroll_vertical` (as `_genphone_probe.gd` does); a stock `ScrollContainer`
## scrolls on a real touch drag, which `SubViewport.push_input` does not drive
## (measured: neither mouse nor ScreenDrag samples moved it), and
## `_tabgest_probe.gd` / `_gestclass_probe.gd` own the swipe-arbitration claim.
## Only the SCROLL POSITION is set here -- never navigation state -- and every
## tap that follows is a real press/release on the control's drawn rect, so a
## control that does not exist, or is clipped sideways, still fails.
func _reach(c: Control) -> bool:
	var scroll := app._phone_gen_scroll as ScrollContainer
	if not is_instance_valid(c):
		return false
	var cr := c.get_global_rect()
	var kr := scroll.get_global_rect()
	if cr.position.y < kr.position.y:
		scroll.scroll_vertical = maxi(0, scroll.scroll_vertical - int(kr.position.y - cr.position.y) - 8)
	elif cr.end.y > kr.end.y:
		scroll.scroll_vertical += int(cr.end.y - kr.end.y) + 8
	await _frames(3)
	return is_instance_valid(c) and _inside(c, scroll)

func _save_shot(name: String) -> void:
	if _shotdir == "":
		return
	var out := "%s/%s_%s.png" % [_shotdir, _tag, name]
	_vp.get_texture().get_image().save_png(out)
	_log("  shot -> %s" % out)

func _map_shot() -> Image:
	var full := _vp.get_texture().get_image()
	var r := Rect2i(app.viewport.get_global_rect()).intersection(Rect2i(Vector2i.ZERO, full.get_size()))
	var img := full.get_region(r)
	img.convert(Image.FORMAT_RGBA8)
	return img

func _mad(a: Image, b: Image) -> float:
	var x := a.duplicate() as Image
	var y := b.duplicate() as Image
	x.resize(96, 96, Image.INTERPOLATE_BILINEAR)
	y.resize(96, 96, Image.INTERPOLATE_BILINEAR)
	var da := x.get_data()
	var db := y.get_data()
	var tot := 0
	for i in da.size():
		tot += absi(int(da[i]) - int(db[i]))
	return float(tot) / float(da.size())

## Fraction of the pixels along `c`'s vertical centre line that are within a
## small distance of `ink`. A slider's filled track is the accent colour, so a
## slider set to 100 % must read mostly accent along its own width.
func _ink_fraction(c: Control, ink: Color) -> float:
	var img := _vp.get_texture().get_image()
	var r := c.get_global_rect()
	var y := int(r.position.y + r.size.y * 0.5)
	var hit := 0
	var n := 0
	for x in range(int(r.position.x), int(r.end.x)):
		if x < 0 or x >= img.get_width() or y < 0 or y >= img.get_height():
			continue
		n += 1
		var p := img.get_pixel(x, y)
		if absf(p.r - ink.r) + absf(p.g - ink.g) + absf(p.b - ink.b) < 0.18:
			hit += 1
	return float(hit) / maxf(1.0, float(n))

## The map's mode chip (`viewport_host.gd::_vp_context`), or null.
func _chip() -> Label:
	return app.viewport.get("_vp_context") as Label

## A real key press and release through the viewport's own input pipeline
## (`_unhandled_key_input` on the app), the way a hardware keyboard on a tablet
## or an adb `input keyevent` reaches it.
func _key(code: Key) -> void:
	for pressed in [true, false]:
		var e := InputEventKey.new()
		e.keycode = code
		e.physical_keycode = code
		e.pressed = pressed
		_vp.push_input(e, true)
		await _frames(2)
	await _frames(SETTLE)

func _force_palette(want_dark: bool) -> void:
	if DccTheme.is_dark() == want_dark:
		return
	var was_dark := DccTheme.is_dark()
	DccTheme.apply_theme(want_dark)
	app.rebuild_theme(was_dark)

# -- Main ------------------------------------------------------------------------------

func _ready() -> void:
	_tag = _arg("--tag", "phonepaint")
	_shotdir = _arg("--shotdir", "")
	if not _reject_unknown_args():
		get_tree().quit(2)
		return
	if not _guard():
		_log("ABORT run WINDOWED -- ImageTexture readback is a no-op under --headless")
		get_tree().quit(2)
		return
	var parts: PackedStringArray = _arg("--vp", "1080x2340").split("x")
	if parts.size() != 2:
		_log("ABORT --vp wants WxH")
		get_tree().quit(2)
		return
	_vp = SubViewport.new()
	_vp.size = Vector2i(int(parts[0]), int(parts[1]))
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await get_tree().create_timer(1.6).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(6)
	bridge = app.bridge
	_log("viewport %dx%d  phone=%s  scale=%.3f" % [_vp.size.x, _vp.size.y, app.is_phone(), app.phone_scale()])

	if not app.is_phone():
		_check(app._phone_gen_scroll == null, "not a phone: the GENERATE sheet is not built, so there is no PAINT segment to reach")
		_finish()
		return

	bridge.generate({"seed": SEED, "width_km": 800.0, "grid_w": GRID_W, "grid_h": GRID_H,
		"sea_level": 0.5, "villages": true})
	await bridge.generation_finished
	await _frames(20)
	_check(bridge.has_world, "a world landed")
	var po = app.paint_original
	if not bridge.has_world or po == null:
		_finish()
		return

	for want_dark in [false, true]:
		_force_palette(want_dark)
		await _frames(6)
		if DccTheme.is_dark() != want_dark:
			_check(false, "palette forced to %s -- refusing to read palette-bound thresholds" % ("dark" if want_dark else "light"))
			continue
		await _run_leg("dark" if want_dark else "light")
	_force_palette(false)
	_finish()

## One full pass under the forced palette. Every leg starts from a clean slate
## (tool disarmed, Original off, GENERATE tab) reached by taps and by the shell's
## own disarm, so the second palette proves the same route, not a leftover.
func _run_leg(theme: String) -> void:
	_log("== palette %s ==" % theme)
	var po = app.paint_original
	var ws = app._workspace_panels["world"]
	var scroll: Control = app._phone_gen_scroll

	# 0. Boot state: nothing armed, Original off -------------------------------------
	app.arm_tool("inspect")   ## slate reset only; not the route under test
	await _frames(SETTLE)
	po.set_percent(0.0)
	ws._paint_brush["erase"] = false
	ws._paint_brush["radius"] = 6.0
	ws._paint_brush["land_only"] = true
	ws._pg_mode = "pipe"
	await _frames(SETTLE)
	_check(app.armed_tool == "inspect", "[%s] starts with no tool armed" % theme)
	_check(not app.viewport.paint_original_state()["visible"], "[%s] and no Original layer on the map" % theme)

	# 1. Reach the sheet by tapping the nav ------------------------------------------------
	var gen_cell := _nav_cell("GENERATE")
	_check(gen_cell != null, "[%s] a bottom-nav cell reads GENERATE" % theme)
	if gen_cell == null:
		return
	## Tapping the ACTIVE tab collapses the sheet to peek and tapping a peeking
	## sheet's tab lifts it to half (`_pick_phone_tab`). Real taps, repeated until
	## the sheet is MEASURED open on GENERATE. Measured, not read from
	## `phone_detent()`: after a palette rebuild the shell reports "half" while the
	## rebuilt sheet is still laid out as a peek strip (a pre-existing
	## `rebuild_theme` mismatch, reported in the lane notes), and a probe that
	## trusted the variable would test a sheet nobody can see.
	for attempt in 3:
		await _tap(gen_cell.get_global_rect().get_center())
		await _sheet_settled()
		if String(app._phone_tab) == "gen" and scroll.get_global_rect().size.y >= 300.0:
			break
	_check(String(app._phone_tab) == "gen" and scroll.visible, "[%s] tapping GENERATE shows the sheet column" % theme)
	_check(String(app.phone_detent()) != "peek" and scroll.get_global_rect().size.y >= 300.0,
		"[%s] at a usable detent (%s), the sheet MEASURES %.0f px tall, not a peek strip" % [theme, app.phone_detent(), scroll.get_global_rect().size.y])

	# 2. The PAINT segment exists, is a real 44 dp+ target, and is tappable ------------------
	var paint_seg := _button(scroll, "PAINT")
	_check(paint_seg != null, "[%s] the GENERATE sheet carries a PAINT segment" % theme)
	if paint_seg == null:
		return
	_check(_inside(paint_seg, scroll), "[%s] the PAINT segment is drawn inside the sheet: %s in %s" % [theme, paint_seg.get_global_rect(), scroll.get_global_rect()])
	_check(_dp(paint_seg.size.y) >= 43.5, "[%s] PAINT is at least 44 dp tall (%.1f dp)" % [theme, _dp(paint_seg.size.y)])
	var seg_pipe := _button(scroll, "PIPELINE")
	var seg_sculpt := _button(scroll, "SCULPT")
	_check(seg_pipe != null and seg_sculpt != null, "[%s] PIPELINE and SCULPT are still there beside it" % theme)
	await _tap(paint_seg.get_global_rect().get_center())

	# 3. Tapping PAINT arms the tool and shows the column --------------------------------------
	_check(app.armed_tool == "paint", "[%s] a tap on PAINT armed the paint tool" % theme)
	var texts := _texts()
	_check("TARGET FIELD" in texts and "BRUSH" in texts, "[%s] the Paint column is up: %s" % [theme, str(texts.slice(0, 8))])
	var sub: Label = app._phone_sheet_subtitle
	_check(sub.text == "paint · draft cells", "[%s] the sheet subtitle names Paint: '%s'" % [theme, sub.text])
	var chip := _chip()
	_check(chip != null and chip.visible and chip.text == "PAINT · DRAFT",
		"[%s] the map's mode chip reads PAINT · DRAFT with Paint armed: '%s'" % [theme, chip.text if chip != null else "(no chip)"])
	var size_sl := _slider("Size")
	var orig_sl := _slider("Original")
	_check(size_sl != null, "[%s] the Size slider is on the sheet" % theme)
	_check(orig_sl != null, "[%s] the Original slider is on the sheet" % theme)
	if size_sl == null or orig_sl == null:
		return
	_check(await _reach(orig_sl), "[%s] Original can be scrolled wholly into the visible sheet" % theme)
	var kr := scroll.get_global_rect()
	var orr := orig_sl.get_global_rect()
	_check(orr.end.x <= kr.end.x + 0.5 and orr.end.x <= float(_vp.size.x) - 1.0,
		"[%s] Original is not clipped at the right edge: slider ends x=%.0f, sheet ends x=%.0f, screen %d" % [theme, orr.end.x, kr.end.x, _vp.size.x])
	_check(orr.size.x >= 200.0, "[%s] and has room to drag: %.0f px wide (%.0f dp)" % [theme, orr.size.x, _dp(orr.size.x)])
	_check(_dp(orr.size.y) >= 43.5, "[%s] and is at least 44 dp tall (%.1f dp)" % [theme, _dp(orr.size.y)])
	_check(is_equal_approx(orig_sl.value, 0.0), "[%s] Original starts at 0 (off)" % theme)

	# 4. Original, by dragging the thumb -------------------------------------------------------------
	var s0 := _map_shot()
	var s0b := _map_shot()
	_check(_mad(s0, s0b) == 0.0, "[%s] negative control: two idle captures of the map are identical" % theme)
	var r := orig_sl.get_global_rect()
	var y := r.position.y + r.size.y * 0.5
	await _drag_to(Vector2(r.position.x + 4.0, y), Vector2(r.position.x + r.size.x * 0.5, y))
	_check(absf(po.opacity - 0.5) <= 0.15 and po.opacity > 0.0,
		"[%s] dragging Original to mid-track set the opacity: %.2f" % [theme, po.opacity])
	var st: Dictionary = app.viewport.paint_original_state()
	_check(bool(st["visible"]) and bool(st["has_texture"]), "[%s] the layer is now drawn over the map" % theme)
	var s50 := _map_shot()
	_check(_mad(s0, s50) > 0.0, "[%s] the map's drawn pixels changed (%.3f)" % [theme, _mad(s0, s50)])
	_save_shot("%s_a_original_mid" % theme)
	r = orig_sl.get_global_rect()
	await _drag_to(Vector2(r.position.x + r.size.x * 0.5, y), Vector2(r.end.x + 30.0, y))
	_check(po.opacity >= 0.95, "[%s] dragging past the right end pins Original at 100 %% (%.2f)" % [theme, po.opacity])
	var fill := _ink_fraction(orig_sl, DccTheme.c("accent"))
	_check(fill > 0.5, "[%s] the slider's filled track is DRAWN across its width (%.0f %% accent pixels), so it is not clipped" % [theme, fill * 100.0])
	_check(_mad(s0, _map_shot()) > _mad(s0, s50), "[%s] 100 %% differs more from 0 than 50 %% does (monotonic)" % theme)
	_save_shot("%s_b_original_full" % theme)
	await _drag_to(Vector2(r.position.x + r.size.x * 0.5, y), Vector2(r.position.x - 30.0, y))
	_check(po.opacity <= 0.001 and not bool(app.viewport.paint_original_state()["visible"]),
		"[%s] dragging back past the left end turns it off and hides the layer (%.2f)" % [theme, po.opacity])
	var mad_back := _mad(s0, _map_shot())
	_check(mad_back == 0.0, "[%s] and the map's pixels are restored exactly (MAD %.4f)" % [theme, mad_back])
	await _drag_to(Vector2(r.position.x + 4.0, y), Vector2(r.position.x + r.size.x * 0.5, y))

	# 5. Size, Erase, Land only ---------------------------------------------------------------------------
	_check(await _reach(size_sl), "[%s] Size can be reached" % theme)
	var sr := size_sl.get_global_rect()
	var sy := sr.position.y + sr.size.y * 0.5
	await _drag_to(Vector2(sr.position.x + 4.0, sy), Vector2(sr.position.x + sr.size.x * 0.75, sy))
	var radius := float(ws._paint_brush["radius"])
	_check(radius > 20.0 and radius < 40.0, "[%s] dragging Size to ~75 %% set the brush radius (%.0f cells)" % [theme, radius])
	## Hardness / Softness: real drags, same shared state, independent of each other.
	var hard_sl := _slider("Hardness")
	var soft_sl := _slider("Softness")
	_check(hard_sl != null and soft_sl != null, "[%s] Hardness and Softness sliders are in the Paint column" % theme)
	if hard_sl != null and soft_sl != null:
		_check(await _reach(hard_sl), "[%s] Hardness can be scrolled wholly into the visible sheet" % theme)
		var hr := hard_sl.get_global_rect()
		_check(_dp(hr.size.y) >= 43.5 and hr.size.x >= 200.0 and hr.end.x <= scroll.get_global_rect().end.x + 0.5,
			"[%s] Hardness is at least 44 dp tall (%.1f dp), has room (%.0f px) and is not clipped at the right" % [theme, _dp(hr.size.y), hr.size.x])
		_check(is_equal_approx(float(ws._paint_brush["hardness"]), 1.0) and is_equal_approx(float(ws._paint_brush["softness"]), 0.0),
			"[%s] the brush starts at the dock's own defaults (hardness 1.00, softness 0.00)" % theme)
		var hy := hr.position.y + hr.size.y * 0.5
		await _drag_to(Vector2(hr.end.x - 4.0, hy), Vector2(hr.position.x + hr.size.x * 0.4, hy))
		var hv := float(ws._paint_brush["hardness"])
		_check(hv > 0.2 and hv < 0.6, "[%s] dragging Hardness to ~40 %% of the track wrote the shared brush (%.2f)" % [theme, hv])
		_check(is_equal_approx(float(ws._paint_brush["softness"]), 0.0), "[%s] and left Softness alone (%.2f)" % [theme, float(ws._paint_brush["softness"])])
		var hread := _label(scroll, "%.2f" % hv)
		_check(hread != null, "[%s] the Hardness readout shows two decimals, like the dock (%.2f)" % [theme, hv])
		soft_sl = _slider("Softness")
		_check(await _reach(soft_sl), "[%s] Softness can be reached" % theme)
		var ur := soft_sl.get_global_rect()
		var uy := ur.position.y + ur.size.y * 0.5
		await _drag_to(Vector2(ur.position.x + 4.0, uy), Vector2(ur.position.x + ur.size.x * 0.6, uy))
		var sv := float(ws._paint_brush["softness"])
		_check(sv > 0.35 and sv < 0.85, "[%s] dragging Softness to ~60 %% wrote the shared brush (%.2f)" % [theme, sv])
		_check(is_equal_approx(float(ws._paint_brush["hardness"]), hv), "[%s] and left Hardness alone (%.2f)" % [theme, float(ws._paint_brush["hardness"])])
		_save_shot("%s_d_hardness_softness" % theme)
		## Put the brush back so the rest of the leg (and the next palette) starts clean.
		ws._paint_brush["hardness"] = 1.0
		ws._paint_brush["softness"] = 0.0
		ws._sync_paint_brush()
	var erase := _button(scroll, "ERASE")
	_check(erase != null, "[%s] an ERASE chip is on the sheet" % theme)
	if erase != null:
		_check(await _reach(erase), "[%s] ERASE can be reached" % theme)
		_check(_dp(erase.size.y) >= 43.5, "[%s] ERASE is at least 44 dp tall (%.1f dp)" % [theme, _dp(erase.size.y)])
		await _tap(erase.get_global_rect().get_center())
		_check(bool(ws._paint_brush["erase"]), "[%s] tapping ERASE latched erase in the shared brush" % theme)
		_check(bool(app.tool_bar._paint_state()["erase"]), "[%s] and the tool bar reads the same state" % theme)
		var erase2 := _button(scroll, "ERASE")
		if erase2 != null:
			await _tap(erase2.get_global_rect().get_center())
		_check(not bool(ws._paint_brush["erase"]), "[%s] a second tap unlatches it" % theme)
	var land := _button(scroll, "LAND ONLY")
	_check(land != null, "[%s] a LAND ONLY chip is on the sheet" % theme)
	if land != null:
		await _reach(land)
		await _tap(land.get_global_rect().get_center())
		_check(not bool(ws._paint_brush["land_only"]), "[%s] tapping LAND ONLY switched it off (it started on)" % theme)

	# 6. Target field and class chips ------------------------------------------------------------------------
	var terrain := _button(scroll, "Terrain")
	_check(terrain != null, "[%s] a Terrain target chip is on the sheet" % theme)
	if terrain != null:
		await _reach(terrain)
		await _tap(terrain.get_global_rect().get_center())
		_check(String(ws._paint_layer) == "terrain", "[%s] tapping Terrain switched the shared paint layer" % theme)
		po.set_percent(60.0)   ## state for the pixel check below; the control path was proven in step 4
		await _frames(SETTLE)
		var tbytes: PackedByteArray = app.viewport.paint_original_node().texture.get_image().get_data()
		_check(tbytes == bridge.debug_texture("cterrain").get_image().get_data(), "[%s] on Terrain the Original layer is the engine's cterrain raster" % theme)
	var splat := _button(scroll, "Splat")
	_check(splat != null, "[%s] a Splat target chip is on the sheet" % theme)
	if splat != null:
		await _reach(splat)
		await _tap(splat.get_global_rect().get_center())
		_check(_slider("Original") == null, "[%s] on Splat there is no Original slider" % theme)
		var why := _label(scroll, "Original · no original")
		_check(why != null, "[%s] a reason stands in for it: '%s'" % [theme, why.text if why != null else "(none)"])
		_check(not bool(app.viewport.paint_original_state()["visible"]), "[%s] and the layer is hidden on Splat" % theme)
	var biome := _button(scroll, "Biome")
	if biome != null:
		await _reach(biome)
		await _tap(biome.get_global_rect().get_center())
	_check(String(ws._paint_layer) == "biome", "[%s] tapping Biome returns to the biome layer" % theme)
	var pal := bridge.get_paint_palette("biome")
	var second: Dictionary = pal[1]
	var cls := _button(scroll, String(second.get("label", "?")))
	_check(cls != null, "[%s] a class chip reads '%s'" % [theme, String(second.get("label", "?"))])
	if cls != null:
		await _reach(cls)
		await _tap(cls.get_global_rect().get_center())
		_check(int(ws._paint_brush["value"]) == int(second.get("index", -1)), "[%s] tapping it set the brush class to index %d" % [theme, int(second.get("index", -1))])

	# 7. COMMIT / DISCARD follow the pending draft ------------------------------------------------------------------
	var commit := _button(scroll, "%s COMMIT" % DccIcons.SYMBOLS["tick"])
	var discard := _button(scroll, "DISCARD")
	_check(commit != null and discard != null, "[%s] COMMIT and DISCARD are on the sheet" % theme)
	if commit != null and discard != null:
		await _reach(commit)
		_check(commit.disabled and discard.disabled, "[%s] with nothing pending both are disabled" % theme)
		## A map stroke is a map gesture, not part of this route: stand in for it
		## with the workspace's own stroke handlers, then release as a finger lifts.
		ws._paint_click(float(GRID_W) * 0.5, float(GRID_H) * 0.5)
		ws._paint_release(0.0, 0.0, true)
		await _frames(SETTLE)
		commit = _button(scroll, "%s COMMIT" % DccIcons.SYMBOLS["tick"])
		_check(commit != null and not commit.disabled and bridge.paint_draft_count() > 0,
			"[%s] after a stroke COMMIT is enabled without leaving the sheet (pending %d)" % [theme, bridge.paint_draft_count()])
		if commit != null:
			await _reach(commit)
			await _tap(commit.get_global_rect().get_center())
		_check(bridge.paint_draft_count() == 0, "[%s] tapping COMMIT committed the draft" % theme)
		commit = _button(scroll, "%s COMMIT" % DccIcons.SYMBOLS["tick"])
		_check(commit != null and commit.disabled, "[%s] and COMMIT is disabled again" % theme)

	# 8. ARM PAINT drops the sheet to peek; the tab lifts it again ----------------------------------------------------
	var arm := _button(scroll, "%s ARM PAINT · DRAW ON THE MAP" % DccIcons.SYMBOLS["add"])
	_check(arm != null, "[%s] an ARM PAINT button is on the sheet" % theme)
	if arm != null:
		await _reach(arm)
		await _tap(arm.get_global_rect().get_center())
		_check(app.armed_tool == "paint" and String(app.phone_detent()) == "peek",
			"[%s] ARM PAINT leaves Paint armed and drops the sheet to peek (%s)" % [theme, app.phone_detent()])
		await _tap(gen_cell.get_global_rect().get_center())
		_check(String(app.phone_detent()) != "peek", "[%s] tapping GENERATE lifts it again (%s)" % [theme, app.phone_detent()])

	# 8b. Another tool armed while the PAINT column is up: Original goes inert --------------------------------------------
	## Escape is a real key press that disarms Paint to Inspect (`_escape_action`)
	## without leaving the sheet or touching the segment, which is the exact state
	## the lane row describes: PAINT column on screen, Paint not the armed tool.
	po.set_percent(60.0)   ## a live value to prove the inert slider does not move
	await _frames(SETTLE)
	await _key(KEY_ESCAPE)
	_check(app.armed_tool == "inspect", "[%s] Escape (a real key press) disarmed Paint (armed=%s)" % [theme, app.armed_tool])
	_check("TARGET FIELD" in _texts(), "[%s] the PAINT column is still the one on screen" % theme)
	chip = _chip()
	_check(chip != null and chip.text != "PAINT · DRAFT",
		"[%s] the chip stops saying PAINT once Paint is disarmed: '%s'" % [theme, chip.text if chip != null else "(no chip)"])
	var inert := _slider("Original")
	_check(inert != null and not inert.editable, "[%s] with Paint disarmed the Original slider is drawn inert (editable=%s)" % [theme, str(inert.editable) if inert != null else "no slider"])
	var note := _label(scroll, "Shows the generator's own layer")
	_check(note != null and note.text.contains("not armed"), "[%s] and the note beside it says why" % theme)
	_check(not bool(app.viewport.paint_original_state()["visible"]), "[%s] and the Original layer is not drawn on the map" % theme)
	if inert != null:
		await _reach(inert)
		var ir := inert.get_global_rect()
		await _drag_to(Vector2(ir.position.x + 4.0, ir.position.y + ir.size.y * 0.5), Vector2(ir.end.x - 4.0, ir.position.y + ir.size.y * 0.5))
		_check(is_equal_approx(po.opacity, 0.6), "[%s] a drag across the inert slider moved nothing: opacity %.2f (still 0.60)" % [theme, po.opacity])
	_save_shot("%s_e_original_inert" % theme)
	var arm2 := _button(scroll, "%s ARM PAINT · DRAW ON THE MAP" % DccIcons.SYMBOLS["add"])
	_check(arm2 != null, "[%s] ARM PAINT is still on the sheet to re-arm" % theme)
	if arm2 != null:
		await _reach(arm2)
		await _tap(arm2.get_global_rect().get_center())
		_check(app.armed_tool == "paint", "[%s] tapping ARM PAINT re-armed Paint" % theme)
		chip = _chip()
		_check(chip != null and chip.text == "PAINT · DRAFT", "[%s] and the chip reads PAINT · DRAFT again: '%s'" % [theme, chip.text if chip != null else "(no chip)"])
		await _tap(gen_cell.get_global_rect().get_center())
		await _sheet_settled()
		var live := _slider("Original")
		_check(live != null and live.editable, "[%s] and Original is live again (editable=%s)" % [theme, str(live.editable) if live != null else "no slider"])

	# 9. Screenshot at a usable detent with Original showing ----------------------------------------------------------------------
	po.set_percent(0.0)
	var o2 := _slider("Original")
	if o2 != null:
		await _reach(o2)
		var r2 := o2.get_global_rect()
		await _drag_to(Vector2(r2.position.x + 4.0, r2.position.y + r2.size.y * 0.5), Vector2(r2.position.x + r2.size.x * 0.6, r2.position.y + r2.size.y * 0.5))
	## Frame the screenshot so BRUSH, Size and Original (label + readout + track)
	## are all in view, the way a person who had just scrolled to them would see it.
	var sz := _slider("Size")
	if sz != null:
		var sc := scroll as ScrollContainer
		sc.scroll_vertical += int(sz.get_parent().get_global_rect().position.y - scroll.get_global_rect().position.y) - _pg_dp_px(48.0)
		await _frames(SETTLE)
	_save_shot("%s_c_sheet_original_visible" % theme)

	# 10. No regression: the other segments and tabs ------------------------------------------------------------------------------
	seg_sculpt = _button(scroll, "SCULPT")
	await _reach(seg_sculpt)
	await _tap(seg_sculpt.get_global_rect().get_center())
	var tx := _texts()
	_check("GEOLOGICAL FEATURE" in tx or tx.any(func(s): return s.begins_with("Generate a world") or s.begins_with("No sculpt")),
		"[%s] SCULPT still shows the sculpt column: %s" % [theme, str(tx.slice(0, 4))])
	_check(sub.text == "sculpt · draft stamps", "[%s] and its subtitle is back to sculpt: '%s'" % [theme, sub.text])
	_check(app.armed_tool == "paint", "[%s] choosing SCULPT does not disarm Paint (the canvas's own behaviour)" % theme)
	seg_pipe = _button(scroll, "PIPELINE")
	await _reach(seg_pipe)
	await _tap(seg_pipe.get_global_rect().get_center())
	tx = _texts()
	_check("SEED" in tx, "[%s] PIPELINE still shows the SEED row" % theme)
	await _tap(gen_cell.get_global_rect().get_center())
	_check("SEED" in _texts() and not ("TARGET FIELD" in _texts()),
		"[%s] re-tapping GENERATE does not clobber the user's PIPELINE choice with Paint" % theme)
	for cap in ["MAP", "PLAN", "MORE"]:
		var cell := _nav_cell(cap)
		_check(cell != null, "[%s] a bottom-nav cell reads %s" % [theme, cap])
		if cell != null:
			await _tap(cell.get_global_rect().get_center())
			_check(not scroll.visible or String(app._phone_tab) == "gen",
				"[%s] %s takes the GENERATE column away (tab=%s)" % [theme, cap, app._phone_tab])
	app.arm_tool("inspect")   ## slate reset for the next palette's leg, as at step 0
	await _frames(SETTLE)
