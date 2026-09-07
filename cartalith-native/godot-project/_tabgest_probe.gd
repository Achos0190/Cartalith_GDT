extends Node
## **Does the tablet's newly-reached gesture arbitration actually behave, or
## does it only census differently?**
##
## `_rangeswipe_probe.gd` and `_gestclass_probe.gd` both drive real swipes, and
## both gate every swipe leg on `app.is_phone()` -- so at `--vp 800x1280`
## (`phone=false tablet=true`) they degrade to a census and nothing drives the
## hit-test. The census proved `dcc_shell.gd`'s `tablet_arbitrate()` REACHES the
## controls; this probe is the other half: that what it attached still behaves
## at the tablet's own `unit` of 1.0, where the slop is 8 px rather than the
## phone's 8 x 2.621 = 21.
##
##   godot --path . _tabgest_probe.tscn -- --force-touch --vp 800x1280 --tag tg800
##   godot --path . _tabgest_probe.tscn -- --force-touch --vp 1280x800 --tag tgL
##
## Flags this probe actually reads, grepped from the body below rather than
## assumed (`MISTAKES.md`, "Write a probe's usage header"):
##   `--vp WxH`      SubViewport size in physical px. Default 800x1280.
##   `--tag NAME`    prefix on every output line. Default `tg`.
##   `--wobble F`    multiplier on the jitter path's sideways travel.
##                   Default **0.75**; see `_wobble` for why it is not 1.0.
##   `--force-touch` NOT read here -- `dcc_shell.gd` reads it out of
##                   `OS.get_cmdline_user_args()`; without it the shell boots
##                   pointer-desktop and every leg below is void. This probe
##                   ABORTS if the shell did not come up as a tablet, so a
##                   forgotten flag cannot pass as a green run.
## Any other `--flag` aborts rather than being silently ignored.
##
## **Every leg carries a positive control**, because "the value did not change"
## and "the gesture never arrived" look identical from the outside.
##
## Run it windowed: nothing here samples a pixel, but everything depends on the
## GUI hit-test and on `ScrollContainer`'s touch drag.

var app: Node
var _vp: SubViewport
var _tag := "tg"
var _fail := 0

## **The sideways amplitude of the jitter path, as a multiplier.**
##
## The path itself is `_rangeswipe_probe.gd`'s, authored against a 1080 px
## wide handset -- so its 10 px peak wobble is 10 PHONE pixels, about 0.63 mm
## on a 6T (1080 px across ~68 mm, 15.9 px/mm). Replayed unscaled on an 800 px
## wide 8" tablet (~172 mm, 4.65 px/mm) the same 10 px is **2.15 mm**, a wobble
## 3.4x larger than the phone was ever tested with. A raw-pixel replay is
## therefore not a fair test of the tablet slop, and this multiplier is how the
## crossover gets measured rather than argued about.
##
## **The sweep, at 800x1280, slider leg, this session:**
##
## | `--wobble` | peak dx | vertical swipe writes the slider? |
## |---|---|---|
## | 1.00 | 10.0 px | **yes** -- classified horizontal |
## | 0.75 | 7.5 px | no |
## | 0.60 / 0.50 / 0.40 / 0.29 | 6.0 / 5.0 / 4.0 / 2.9 px | no |
##
## The crossover sits between 7.5 and 10 px, which is where it must: `PgSlider`
## classifies at the first sample whose cumulative travel reaches `slop`, and
## `slop` on a tablet is **8** (`dcc_shell.gd`'s `tablet_arbitrate()` passes
## `unit` 1.0, and 8 authored px is 8 device px in a composition that applies no
## `content_scale_factor`).
##
## **So the 1.0 row is not a tablet defect, and this probe defaults to 0.75 to
## avoid reporting it as one.** In physical terms the tablet is the *stricter*
## of the two surfaces: 8 px on an 800 px wide 8" panel (~172 mm, 4.65 px/mm) is
## **1.72 mm** of sideways lead before a gesture is called horizontal, against
## the phone's 21 px on a 6T (15.9 px/mm) = **1.32 mm**. Replaying the phone's
## raw-pixel path here asks for a 2.15 mm wobble, which exceeds BOTH thresholds
## and would be read as horizontal on the phone too. 0.75 is 1.6 mm, just inside
## the tablet's own threshold.
##
## The physical figures assume an 8" 16:10 panel, because `DisplayServer.
## screen_get_dpi()` is not answerable off a real device -- `dcc_shell.gd`'s
## `_is_tablet_sized()` returns `false` off `OS.has_feature("mobile")` for that
## exact reason. Stated as an assumption, not as a measurement.
var _wobble := 0.75

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _log(s: String) -> void:
	print("[%s] %s" % [_tag, s])

func _check(ok: bool, what: String) -> void:
	if not ok:
		_fail += 1
	_log("%s  %s" % ["PASS" if ok else "FAIL", what])

func _arg(name: String, dflt: String) -> String:
	var a := OS.get_cmdline_user_args()
	for i in a.size():
		if a[i] == name and i + 1 < a.size():
			return a[i + 1]
	return dflt

func _reject_unknown_args() -> bool:
	var known := ["--vp", "--tag", "--wobble", "--force-touch"]
	var takes := ["--vp", "--tag", "--wobble"]
	var a := OS.get_cmdline_user_args()
	var i := 0
	while i < a.size():
		if not a[i].begins_with("--"):
			i += 1
			continue
		if not known.has(a[i]):
			_log("ABORT unknown argument %s" % a[i])
			return false
		i += 2 if takes.has(a[i]) else 1
	return true

func _push_path(from: Vector2, offsets: Array) -> void:
	var down := InputEventMouseButton.new()
	down.button_index = MOUSE_BUTTON_LEFT
	down.pressed = true
	down.position = from
	down.global_position = from
	_vp.push_input(down, true)
	await _frames(1)
	var prev := from
	for o in offsets:
		var at: Vector2 = from + (o as Vector2)
		var mm := InputEventMouseMotion.new()
		mm.position = at
		mm.global_position = at
		mm.relative = at - prev
		mm.button_mask = MOUSE_BUTTON_MASK_LEFT
		_vp.push_input(mm, true)
		prev = at
		await _frames(1)
	var up := InputEventMouseButton.new()
	up.button_index = MOUSE_BUTTON_LEFT
	up.pressed = false
	up.position = prev
	up.global_position = prev
	_vp.push_input(up, true)
	await _frames(4)

## The same path `_rangeswipe_probe.gd` drives, restated here rather than
## imported so this probe does not depend on that one: the first three samples
## travel further sideways than downward, which is what a real thumb does and
## what a `slop` of 0 misclassifies as "this is horizontal, write the value".
func _jitter(dir: float, reach: float) -> Array:
	var out: Array = []
	var xs := [3.0, 6.0, 5.0, 8.0, 10.0, 6.0, 4.0, 2.0, 0.0, -2.0, -4.0, -4.0, -3.0]
	var ys := [1.0, 2.0, 4.0, 6.0, 10.0, 20.0, 40.0, 80.0, 140.0, 210.0, 290.0, 380.0, 468.0]
	var span: float = ys[ys.size() - 1]
	for i in xs.size():
		out.append(Vector2(float(xs[i]) * _wobble,
			dir * float(ys[i]) / span * reach))
	return out

func _walk(root: Node, out: Array) -> void:
	for c in root.get_children():
		out.append(c)
		_walk(c, out)

func _all(root: Node) -> Array:
	var out: Array = []
	_walk(root, out)
	return out

## The nearest ancestor `ScrollContainer` that actually scrolls vertically.
## Restated here rather than called on `DccWidgets`, so the measurement does
## not depend on the code under test to describe itself.
func _scroller_of(n: Node) -> ScrollContainer:
	var p: Node = n.get_parent()
	while p != null:
		if p is ScrollContainer and (p as ScrollContainer).vertical_scroll_mode \
				!= ScrollContainer.SCROLL_MODE_DISABLED:
			return p as ScrollContainer
		p = p.get_parent()
	return null

func _centre(c: Control) -> Vector2:
	return c.get_global_rect().get_center()

## On screen, big enough to hit, and clear of both the chrome at the top and
## the bottom edge the swipe travels toward.
func _reachable(c: Control) -> bool:
	if not c.is_visible_in_tree():
		return false
	var r := c.get_global_rect()
	if r.size.x < 8.0 or r.size.y < 4.0:
		return false
	return r.position.y >= 60.0 and r.get_center().y <= float(_vp.size.y) - 200.0

func _ready() -> void:
	_tag = _arg("--tag", "tg")
	_wobble = float(_arg("--wobble", "0.75"))
	if not _reject_unknown_args():
		get_tree().quit(2)
		return
	var parts: PackedStringArray = _arg("--vp", "800x1280").split("x")
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
	if app.phone_project_picker != null and app.phone_project_picker.visible:
		app.phone_project_picker.hide()
	await _frames(8)

	_log("viewport %dx%d  phone=%s  tablet=%s  wobble=%.2f  peak_dx=%.1fpx" %
		[_vp.size.x, _vp.size.y, app.is_phone(), DccTheme.is_tablet(),
		_wobble, 10.0 * _wobble])
	if app.is_phone() or not DccTheme.is_tablet():
		_log("ABORT this probe is about the TABLET composition -- pass --force-touch and an aspect >= 0.6")
		get_tree().quit(2)
		return

	await _slider_legs()
	await _dropdown_legs()

	_log("RESULT %s fail=%d" % [_tag, _fail])
	get_tree().quit(1 if _fail > 0 else 0)

## **The inventory, printed whether or not a target is found.** A `FAIL  found
## a live ...` with nothing beside it cannot tell "the dock is empty" from "the
## dock is full and every row is off screen", and the first cut of this probe
## reported exactly that.
func _inventory(cls: String) -> void:
	if app.left_dock == null:
		_log("inventory %s: left_dock is null" % cls)
		return
	var total := 0
	var vis := 0
	var scrolled := 0
	var onscreen := 0
	var first_rect := ""
	for n in _all(app.left_dock):
		if not n.is_class(cls):
			continue
		total += 1
		var c := n as Control
		if not c.is_visible_in_tree():
			continue
		vis += 1
		if _scroller_of(n) == null:
			continue
		scrolled += 1
		if first_rect == "":
			first_rect = str(c.get_global_rect())
		if _reachable(c):
			onscreen += 1
	_log("inventory %s: total=%d visible=%d in-a-scroller=%d reachable=%d  first=%s"
		% [cls, total, vis, scrolled, onscreen, first_rect])

## **The dock is taller than the tablet, so a target has to be scrolled to
## before it can be swiped on.** Measured at 800x1280: of 214 `HSlider`s in the
## left dock 5 are visible, all 5 inside the scroller, and **0** land on screen
## at rest -- the first sits at y 1470 in a 1280 px viewport. So this scrolls
## the sheet to put the target at ~40 % height, which also leaves travel in
## both directions for the swipe legs to consume.
##
## `scroll_vertical` is written directly rather than by `ensure_control_visible`
## -- that call parks the control at the nearest EDGE, which is the one place a
## 300 px downward swipe has nowhere to go.
func _scroll_to(c: Control) -> void:
	var sc := _scroller_of(c)
	if sc == null:
		return
	var want: float = c.get_global_rect().get_center().y - sc.get_global_rect().position.y
	sc.scroll_vertical = int(round(float(sc.scroll_vertical) + want
		- sc.get_global_rect().size.y * 0.40))
	await _frames(3)

func _first_in_scroller(cls: String) -> Control:
	if app.left_dock == null:
		return null
	for n in _all(app.left_dock):
		if not n.is_class(cls):
			continue
		var c := n as Control
		if c.is_visible_in_tree() and _scroller_of(n) != null \
				and c.get_global_rect().size.x >= 8.0:
			return c
	return null

func _pick_slider() -> HSlider:
	if app.left_dock == null:
		return null
	for n in _all(app.left_dock):
		if n is HSlider and (n as HSlider).editable and _reachable(n as Control) \
				and _scroller_of(n) != null:
			return n as HSlider
	return null

func _slider_legs() -> void:
	_inventory("HSlider")
	var seed_s := _first_in_scroller("HSlider")
	if seed_s != null:
		await _scroll_to(seed_s)
		_inventory("HSlider")
	var s := _pick_slider()
	if s == null:
		_check(false, "found a live left-dock HSlider inside a vertical scroller")
		return
	var sc := _scroller_of(s)
	_check(s.get_script() != null,
		"the picked slider carries PgSlider -- tablet_arbitrate() reached it")

	## Leg 1, the defect itself: a jittered VERTICAL swipe must scroll the
	## sheet and must leave the parameter alone.
	var v0: float = s.value
	var y0: int = sc.scroll_vertical
	await _push_path(_centre(s), _jitter(1.0, 300.0))
	_log("vertical swipe on slider: value %.4f -> %.4f   scroll %d -> %d"
		% [v0, s.value, y0, sc.scroll_vertical])
	_check(is_equal_approx(v0, s.value),
		"a vertical swipe starting on the slider does NOT write it")
	_check(sc.scroll_vertical != y0,
		"a vertical swipe starting on the slider scrolls the sheet")

	## Leg 2, the positive control. Without it, leg 1 passes just as well when
	## the gesture never reached the control at all.
	## Re-seeded for the same reason the dropdown leg is: the swipe that just
	## passed moved the sheet 300 px, and in landscape (1280x800) that takes
	## every one of the five live sliders off screen.
	var seed2 := _first_in_scroller("HSlider")
	if seed2 != null:
		await _scroll_to(seed2)
	var s2 := _pick_slider()
	if s2 == null:
		_check(false, "positive control: a slider is still on screen after the scroll")
		return
	var v2: float = s2.value
	var wide: Array = []
	for dx in [4.0, 10.0, 24.0, 48.0, 80.0, 120.0]:
		wide.append(Vector2(dx, 1.0))
	await _push_path(_centre(s2), wide)
	_log("horizontal drag on slider: value %.4f -> %.4f" % [v2, s2.value])
	_check(not is_equal_approx(v2, s2.value),
		"POSITIVE CONTROL: a horizontal drag on the slider still writes it")

func _pick_option() -> OptionButton:
	if app.left_dock == null:
		return null
	for n in _all(app.left_dock):
		if n is OptionButton and not (n as OptionButton).disabled \
				and _reachable(n as Control) and _scroller_of(n) != null:
			return n as OptionButton
	return null

func _dropdown_legs() -> void:
	_inventory("OptionButton")
	var seed_o := _first_in_scroller("OptionButton")
	if seed_o != null:
		await _scroll_to(seed_o)
		_inventory("OptionButton")
	var o := _pick_option()
	if o == null:
		_check(false, "found a live left-dock OptionButton inside a vertical scroller")
		return
	var sc := _scroller_of(o)
	_check(o.action_mode == BaseButton.ACTION_MODE_BUTTON_RELEASE,
		"the picked dropdown acts on RELEASE -- touch_release_button() reached it")

	var sel0: int = o.selected
	var y0: int = sc.scroll_vertical
	await _push_path(_centre(o), _jitter(1.0, 300.0))
	var pop_open: bool = o.get_popup().visible
	_log("vertical swipe on dropdown: sel %d -> %d   scroll %d -> %d   popup=%s"
		% [sel0, o.selected, y0, sc.scroll_vertical, pop_open])
	_check(not pop_open, "a vertical swipe on the dropdown does NOT open its popup")
	_check(o.selected == sel0, "a vertical swipe on the dropdown does NOT change the selection")
	_check(sc.scroll_vertical != y0, "a vertical swipe on the dropdown scrolls the sheet")

	## Re-seeded, because the swipe that just passed moved the sheet 530 px and
	## took the one live dropdown off screen with it -- the first cut of this
	## probe scored that as a failed positive control.
	var seed2 := _first_in_scroller("OptionButton")
	if seed2 != null:
		await _scroll_to(seed2)
	var o2 := _pick_option()
	if o2 == null:
		_check(false, "positive control: a dropdown is still on screen after the scroll")
		return
	await _push_path(_centre(o2), [Vector2(0.0, 0.0)])
	var open2: bool = o2.get_popup().visible
	_check(open2, "POSITIVE CONTROL: a clean tap on the dropdown still opens it")
	if open2:
		o2.get_popup().hide()
		await _frames(2)
