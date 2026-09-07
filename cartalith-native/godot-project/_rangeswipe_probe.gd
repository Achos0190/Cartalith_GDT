extends Node
## **Every `Range` on the phone that sits inside a scroller, and what a
## vertical swipe with a sideways wobble does to it.**
##
## Two jobs, and the first is the reason the second exists:
##
## 1. **The census.** `PgSlider` closed the vertical-swipe-writes-the-parameter
##    defect at the two `_pg_*` construction sites in `world_workspace.gd`, and
##    nowhere else. This probe walks the LIVE phone tree and reports every
##    `Range` together with its nearest `ScrollContainer` ancestor, so the
##    inventory is a measurement rather than a grep of `HSlider.new()` -- which
##    would miss every `DccWidgets.slider()` and `DccWidgets.number()` row and
##    every `_slider_row()` in `phone_menu.gd`. Counted in the same edit that
##    writes it: `grep -rn 'DccWidgets.slider(' shell/ --include=*.gd |
##    grep -v ':[0-9]*:[[:space:]]*#' | wc -l` is **41** and the same for
##    `number(` is **13**, 2026-09-07.
## 2. **The jitter swipe.** `_nwsize_probe.gd`'s vertical leg drives a
##    *perfectly* vertical path (`delta.x == 0`), so it passes with
##    `PgSlider.slop` mutated to **0** -- at which point the first pixel of
##    motion picks the axis and a real thumb's sideways wobble writes the
##    parameter. `_jitter()` below leads with three samples whose x travel
##    exceeds their y travel, which is what a thumb actually does, and is the
##    check that pins the slop from BELOW.
##
##   godot --path . _rangeswipe_probe.tscn -- --force-touch --vp 1080x2340 --tag p1080
##   godot --path . _rangeswipe_probe.tscn -- --force-touch --vp 1440x3168 --tag p1440
##   godot --path . _rangeswipe_probe.tscn -- --force-touch --vp 800x1280  --tag tab800
##
## Flags this probe actually reads, grepped from the body below rather than
## assumed (`MISTAKES.md`, "Write a probe's usage header"):
##   `--vp WxH`      SubViewport size in physical px. Default 1080x2340.
##   `--tag NAME`    prefix on every output line. Default `rs`.
##   `--census-only` skip the swipe legs; print the inventory and quit.
##   `--force-touch` NOT read here -- `dcc_shell.gd` reads it out of
##                   `OS.get_cmdline_user_args()`, and without it the shell
##                   boots desktop and every line below is void.
## Any other `--flag` aborts rather than being silently ignored.
##
## **Run it windowed.** Nothing here samples a pixel, but every act depends on
## the GUI hit-test, which wants a real frame loop; `MISTAKES.md`'s headless row
## is scoped to rasterising and timing.

var app: Node
var _vp: SubViewport
var _tag := "rs"
var _fail := 0
var _drag_ends := 0

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

func _flag(name: String) -> bool:
	return OS.get_cmdline_user_args().has(name)

func _reject_unknown_args() -> bool:
	var known := ["--force-touch", "--vp", "--tag", "--census-only"]
	for a in OS.get_cmdline_user_args():
		var s := String(a)
		if s.begins_with("--") and not (s in known):
			_log("ABORT unknown flag %s -- this probe reads only %s" % [s, str(known)])
			return false
	return true

# -- Input, pushed through the viewport's own hit-test --------------------------

func _tap_at(at: Vector2) -> void:
	for down in [true, false]:
		var e := InputEventMouseButton.new()
		e.button_index = MOUSE_BUTTON_LEFT
		e.pressed = down
		e.position = at
		e.global_position = at
		_vp.push_input(e, true)
		await _frames(1)
	await _frames(3)

## A press, one motion per entry of `offsets` (each an offset from `from`),
## then a release. `offsets` rather than a straight `delta` because the whole
## point of the jitter leg is a path whose EARLY samples travel further
## sideways than down -- which a linear interpolation cannot express.
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

## **The path this probe exists for.** A thumb swiping down a list does not
## travel straight: the first few samples after touch-down carry more sideways
## travel than downward travel, because the thumb pivots before it slides. The
## first three offsets below are exactly that -- `(3,1) (6,2) (5,4)` -- and
## they are what a `slop` of 0 classifies as "this is a horizontal drag, write
## the value". A slop of 8 dp is still undecided at all three and has resolved
## downward by the time the travel passes it.
##
## `dir` is +1 to swipe DOWN the screen (content moves up) and -1 for up; the
## caller picks whichever end of the scroller has room.
func _jitter(dir: float, reach: float) -> Array:
	var out: Array = []
	var xs := [3.0, 6.0, 5.0, 8.0, 10.0, 6.0, 4.0, 2.0, 0.0, -2.0, -4.0, -4.0, -3.0]
	var ys := [1.0, 2.0, 4.0, 6.0, 10.0, 20.0, 40.0, 80.0, 140.0, 210.0, 290.0, 380.0, 468.0]
	var span: float = ys[ys.size() - 1]
	for i in xs.size():
		out.append(Vector2(float(xs[i]), dir * float(ys[i]) / span * reach))
	return out

# -- Tree walking ---------------------------------------------------------------

func _walk(root: Node, out: Array) -> void:
	for c in root.get_children():
		out.append(c)
		_walk(c, out)

func _all(root: Node) -> Array:
	var out: Array = []
	_walk(root, out)
	return out

## The nearest `ScrollContainer` above `n`, or null. The same lookup
## `DccWidgets.PgSlider._scroll()` makes, restated here so the census does not
## depend on the class under test to describe itself.
func _scroller_of(n: Node) -> ScrollContainer:
	var p: Node = n.get_parent()
	while p != null:
		if p is ScrollContainer:
			return p as ScrollContainer
		p = p.get_parent()
	return null

## Which phone surface `n` belongs to, named by the node the shell itself
## keeps a reference to rather than by a class test -- so the census reads in
## the vocabulary of `dcc_shell.gd` and can be checked against it.
func _surface_of(n: Node) -> String:
	var p: Node = n
	while p != null:
		if p == app.left_dock:
			return "left_dock sheet"
		if p == app.right_dock:
			return "right_dock sheet"
		if p == app._phone_gen_scroll:
			return "GENERATE sheet"
		if p == app._phone_tool_scroll:
			return "tool-options sheet"
		if p is PhoneMenu:
			return "PhoneMenu (MORE)"
		if p is Window and p != get_tree().root and p != _vp:
			## Named by its SCRIPT, not by `p.name` -- an `AcceptDialog` built
			## in code is `@AcceptDialog@683`, which tells a reader nothing
			## about which surface it is.
			var scr: Script = p.get_script()
			return "Window %s" % (scr.resource_path.get_file() if scr != null
				else p.get_class())
		p = p.get_parent()
	return "phone root"

func _census(where: String) -> Dictionary:
	var rows: Dictionary = {}
	var hazard := 0
	var total := 0
	for n in _all(get_tree().root):
		if not (n is Range):
			continue
		total += 1
		var sc := _scroller_of(n)
		var scrolls: bool = sc != null \
			and sc.vertical_scroll_mode != ScrollContainer.SCROLL_MODE_DISABLED
		## A `ProgressBar` has no `gui_input` at all, so it cannot be written by
		## any gesture; it is counted and then excluded from the hazard, which
		## is a finding rather than an omission.
		var writable: bool = (n is Slider) or (n is SpinBox)
		var key := "%s | %s | scroller=%s | script=%s | writable=%s" % [
			_surface_of(n), n.get_class(),
			("none" if sc == null else ("vDISABLED" if not scrolls else "vSCROLLS")),
			("yes" if n.get_script() != null else "NO"),
			str(writable)]
		rows[key] = int(rows.get(key, 0)) + 1
		if scrolls and writable and n.get_script() == null:
			hazard += 1
	_log("-- census (%s): %d Range nodes" % [where, total])
	var keys: Array = rows.keys()
	keys.sort()
	for k in keys:
		_log("   %4d  %s" % [rows[k], k])
	_log("   UNARBITRATED writable Range inside a live vertical scroller: %d" % hazard)
	return {"total": total, "hazard": hazard}

func _first_slider(root: Node) -> HSlider:
	for n in _all(root):
		if n is HSlider and (n as HSlider).is_visible_in_tree():
			return n as HSlider
	return null

func _find_label(text: String) -> Label:
	for n in _all(_vp):
		if n is Label and (n as Label).is_visible_in_tree() \
				and String((n as Label).text) == text:
			return n as Label
	return null

## Find a visible caption reading exactly `text` -- `Label` or `Button`, since
## this shell draws both -- and tap the pressable cell it sits in. Returns
## whether anything was found, so a caller can say "not reached" instead of
## silently measuring the previous screen.
func _tap_caption(text: String) -> bool:
	var target: Control = _find_label(text)
	if target == null:
		for n in _all(_vp):
			if n is Button and (n as Button).is_visible_in_tree() \
					and String((n as Button).text) == text:
				target = n as Button
				break
	if target == null:
		_log("   (no visible caption `%s` -- surface not reached)" % text)
		return false
	## A `Button` ancestor where there is one; otherwise the caption's own rect,
	## because `phone_menu.gd` draws its drill rows as a `PanelContainer` with
	## its own `gui_input` rather than as a `Button` -- the first cut of this
	## helper required a `Button` and reported MORE > Simulation "unreachable"
	## when it is simply not built out of one. Either way the tap goes through
	## the viewport's hit-test at a rect read off the live tree.
	var cell := _pressable_ancestor(target)
	var at: Vector2 = (cell if cell != null else target).get_global_rect().get_center()
	await _tap_at(at)
	await _frames(8)
	return true

func _pressable_ancestor(n: Node) -> Button:
	var p := n
	while p != null:
		if p is Button:
			return p as Button
		p = p.get_parent()
	return null

func _count_drag_end(_c: bool) -> void:
	_drag_ends += 1

# -- The swipe leg --------------------------------------------------------------

## One slider, one jittered vertical swipe, three assertions. `label` names the
## surface for the report; `read` is a Callable returning the value to compare
## (the ENGINE's, where there is one -- a rebuild can free the node under test).
func _swipe_leg(label: String, slider: HSlider, scroll: ScrollContainer,
		read: Callable) -> void:
	_log("-- vertical swipe with cross-axis jitter: %s" % label)
	_check(slider != null and scroll != null, "found a slider and its scroller")
	if slider == null or scroll == null:
		return
	_check(slider.get_script() != null,
		"the slider carries the arbitration script (script=%s)"
			% ("yes" if slider.get_script() != null else "NO"))
	scroll.ensure_control_visible(slider)
	await _frames(4)
	if not slider.drag_ended.is_connected(_count_drag_end):
		slider.drag_ended.connect(_count_drag_end)
	var r := slider.get_global_rect()
	var v0: float = float(read.call())
	var s0 := scroll.scroll_vertical
	_drag_ends = 0
	var bar := scroll.get_v_scroll_bar()
	var room_up: float = bar.max_value - bar.page - float(s0)
	var dir := -1.0 if room_up > 200.0 else 1.0
	## The LEFT quarter of the track: a press there on a slider sitting high is
	## where Godot's own touch-down `set_as_ratio()` is loudest.
	var start := Vector2(r.position.x + r.size.x * 0.25, r.position.y + r.size.y * 0.5)
	var reach: float = minf(float(_vp.size.y) * 0.20, maxf(120.0, absf(room_up)))
	_log("   rect %.0f,%.0f %.0fx%.0f  scroll %d of %.0f  dir %s  reach %.0f"
		% [r.position.x, r.position.y, r.size.x, r.size.y, s0,
			bar.max_value - bar.page, "up" if dir < 0.0 else "down", reach])
	await _push_path(start, _jitter(dir, reach))
	var v1: float = float(read.call())
	_log("   value %s -> %s   scroll %d -> %d   drag_ended x%d"
		% [str(v0), str(v1), s0, scroll.scroll_vertical, _drag_ends])
	_check(v1 == v0,
		"%s: a jittered vertical swipe leaves the value byte-identical (%s vs %s)"
			% [label, str(v0), str(v1)])
	_check(_drag_ends == 0, "%s: and emits no drag_ended" % label)
	_check(scroll.scroll_vertical != s0,
		"%s: and DOES scroll the sheet (%d -> %d)"
			% [label, s0, scroll.scroll_vertical])

# -- SpinBox: is it a hazard at all? -------------------------------------------
#
# `SpinBox` is a `Range`, and Godot's own `SpinBox::gui_input` steps the value
# on press and then drags it -- which would be the identical defect one class
# over. Whether that is REACHABLE on this build is a question about the engine
# and about how `DccWidgets.number()` lays the control out, so it is measured
# here rather than reasoned about from C++ this repository does not contain.
#
# **With a positive control**, because "the value did not move" is worthless on
# its own: a tap at the same x must step the value by `step` if
# `SpinBox::gui_input` is receiving the event at all. If neither the tap nor
# the drag moves it, the control never sees the gesture and the whole class is
# not a hazard here -- which is a finding, not an omission.

func _spinbox_leg() -> void:
	_log("-- SpinBox: does a press-and-vertical-drag write it?")
	var sb: SpinBox = null
	for n in _all(get_tree().root):
		if n is SpinBox and (n as SpinBox).is_visible_in_tree() and (n as SpinBox).editable:
			var sc := _scroller_of(n)
			if sc != null and sc.vertical_scroll_mode != ScrollContainer.SCROLL_MODE_DISABLED:
				sb = n as SpinBox
				break
	if sb == null:
		_log("   (no visible editable SpinBox inside a live vertical scroller on this screen)")
		return
	var sc2 := _scroller_of(sb)
	sc2.ensure_control_visible(sb)
	await _frames(4)
	var r := sb.get_global_rect()
	_log("   %s  rect %.0fx%.0f  step %s  value %s"
		% [sb.get_class(), r.size.x, r.size.y, str(sb.step), str(sb.value)])
	## **The structural half of the answer, and the decisive one.**
	## `SpinBox::gui_input` only ever receives what its `LineEdit` child does
	## not, so if the child covers the whole control there is no strip for a
	## press-and-drag to land on and the class cannot be written by a gesture at
	## all. That is a rect comparison rather than three hopeful taps -- the taps
	## below stay as the behavioural confirmation.
	var le := sb.get_line_edit()
	var lr := le.get_global_rect() if le != null else Rect2()
	_log("   its LineEdit covers %.0fx%.0f of %.0fx%.0f -- uncovered strip %.0f px wide"
		% [lr.size.x, lr.size.y, r.size.x, r.size.y, r.size.x - lr.size.x])
	## Three x offsets from the right edge, because the `LineEdit` child covers
	## everything left of the spin-button strip and the strip's width is a theme
	## figure this probe does not know. One of them is inside it if it is
	## reachable at all.
	for inset_v in [3.0, 8.0, 18.0]:
		var inset := float(inset_v)
		var x := r.position.x + r.size.x - inset
		var y := r.position.y + r.size.y * 0.5
		var v_tap := sb.value
		await _tap_at(Vector2(x, y))
		var after_tap := sb.value
		var v_drag := sb.value
		await _push_path(Vector2(x, y), _jitter(1.0, 200.0))
		_log("   inset %.0f px:  tap %s -> %s   then jittered swipe %s -> %s"
			% [inset, str(v_tap), str(after_tap), str(v_drag), str(sb.value)])
		_check(sb.value == v_drag,
			"SpinBox at inset %.0f: a jittered vertical swipe leaves it byte-identical"
				% inset)
		if after_tap != v_tap:
			_log("   ^ the tap DID step it, so `SpinBox::gui_input` is live at this x")
			sb.value = v_tap

# -- Boot ----------------------------------------------------------------------

func _ready() -> void:
	_tag = _arg("--tag", "rs")
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
	if app.phone_project_picker != null and app.phone_project_picker.visible:
		app.phone_project_picker.hide()
	await _frames(6)

	_log("viewport %dx%d  phone=%s  tablet=%s  scale=%.3f" %
		[_vp.size.x, _vp.size.y, app.is_phone(), DccTheme.is_tablet(), app.phone_scale()])

	## **The boot state, before any input.** `MISTAKES.md`: the GENERATE bug hid
	## from five passes of probes because the probe's own tap fixed the state.
	if app.is_phone():
		_check(app._phone_gen_scroll != null and app._phone_gen_scroll.visible,
			"at launch, nothing tapped, the GENERATE column is already up")

	var before := _census("boot, docks unopened")

	## Open both dock sheets so their content is on screen and measurable. The
	## sheets are BUILT at boot either way (`_build_left_dock(true)`), so the
	## census above already counted them -- this is about the swipe leg needing
	## a visible, laid-out slider, not about the inventory.
	if app.is_phone():
		app._set_sheet_open("left", true)
		await _frames(8)
		_log("   (left sheet opened by calling `_set_sheet_open` -- this is the")
		_log("    census and the gesture logic, NOT a claim that the route is")
		_log("    reachable by a finger; that claim is made on glass.)")
	_census("left sheet open")

	## **Leg 1, while the left sheet is still open.** The 214 `HSlider`s the
	## census just counted there are `DccWidgets.slider()` rows -- the population
	## `PgSlider` did not cover before this pass -- so this is the leg that says
	## whether attaching by `set_script()` from `phone_fit()` actually works.
	if not _flag("--census-only") and app.is_phone() and app.left_dock != null:
		var sl := _first_slider(app.left_dock)
		var sc := _scroller_of(sl) if sl != null else null
		if sl != null and sc != null:
			await _swipe_leg("left dock", sl, sc, func(): return sl.value)
		else:
			_check(false, "the left sheet draws a slider inside its scroller")

	## The phone surfaces that build their content only when entered, so the boot
	## census structurally cannot see them: PLAN (the journey planner, which fills
	## the left dock), MORE's root and MORE > Simulation -- where
	## `phone_menu.gd::_slider_row()` draws the Year cursor. Reached by hunting
	## the visible caption and tapping it, the way a person does.
	if app.is_phone():
		app._set_sheet_open("left", false)
		await _frames(4)
		if await _tap_caption("PLAN"):
			_census("after tapping PLAN")
			## The planner's form is the shell's densest `SpinBox` population and
			## the only one reachable without opening a window, so the SpinBox
			## question is asked here rather than at the end. The sheet has to
			## be OPEN for it: the census counts a node wherever it is parented,
			## but a swipe needs a laid-out, visible rect, and the first cut of
			## this leg reported "no visible editable SpinBox" for exactly that
			## reason while the census above was counting twelve of them.
			if not _flag("--census-only"):
				app._set_sheet_open("left", true)
				await _frames(8)
				await _spinbox_leg()
				app._set_sheet_open("left", false)
				await _frames(4)
		if await _tap_caption("MORE"):
			_census("after tapping MORE")
		## `Simulation` is a MORE drill row -- the caption is read off `ROOT_ROWS`
		## rather than guessed, because the first cut of this line hunted
		## "Simulate", found nothing and reported the screen unreached.
		##
		## **This leg reaches the screen and finds NO slider on it, and the
		## label used to say otherwise.** It read "the Year slider's screen",
		## which is a claim about what was walked, and in this probe's own boot
		## state it is false: `tl_available()` is `false` without a world, so
		## `_fill_sim()` draws `_missing_row("Year")` and the four
		## `phone_menu.gd::_slider_row()` sites contribute nothing to the
		## census. **So the census total below is a LOWER BOUND taken in one
		## state, not the enumeration of the hazard** -- a world-loaded boot
		## would find more, and the four `_slider_row()` sliders are converted
		## on the strength of a code walk rather than an observation.
		## Recorded here rather than in prose only, because a committed probe
		## that names a surface is read as having walked it.
		if await _tap_caption("Simulation"):
			_census("MORE > Simulation (no Range in a world-less boot: tl_available() is false)")

	## The Layers popover is a `PopupPanel`, so no `Control` walk reaches it and
	## the boot census sees it UNFITTED -- `layers_popover.gd::_phone_fit()`
	## returns early until `phone_present()` has run, which happens inside
	## `open()`. Opened by calling `open()` and saying so: this is the census and
	## the fitter, NOT a claim that the route is reachable by a finger.
	if app.is_phone() and app.layers_popover != null:
		app.layers_popover.open()
		await _frames(10)
		_census("Layers popover open (its own `_phone_fit()` has now run)")
		app.layers_popover.hide()
		await _frames(4)

	if _flag("--census-only"):
		_log("RESULT %s fail=%d (census only)" % [_tag, _fail])
		get_tree().quit(1 if _fail > 0 else 0)
		return

	# -- Leg 2: the GENERATE sheet (the already-fixed `_pg_*` population) -------
	if app.is_phone():
		app._set_sheet_open("left", false)
		await _frames(4)
		var cap := _find_label("GENERATE")
		if cap != null:
			var cell := _pressable_ancestor(cap)
			if cell != null:
				await _tap_at(cell.get_global_rect().get_center())
		await _frames(8)
		var gs: ScrollContainer = app._phone_gen_scroll
		if gs != null and gs.visible:
			var g := _first_slider(gs)
			if g != null:
				var ws: Node = app._workspace_panels.get("world")
				var key := _key_for(g, ws)
				if key.is_empty():
					await _swipe_leg("GENERATE sheet", g, gs, func(): return g.value)
				else:
					_log("   (asserting against the ENGINE param `%s`, since a" % key)
					_log("    write rebuilds the column and frees the node)")
					await _swipe_leg("GENERATE sheet", g, gs,
						func(): return float(app.bridge.param_get(key)))
			else:
				_check(false, "the GENERATE sheet draws a slider")

	_log("RESULT %s fail=%d  census_hazard_before=%d" % [_tag, _fail, before["hazard"]])
	get_tree().quit(1 if _fail > 0 else 0)

## The engine parameter key a GENERATE-sheet slider writes, found from the row
## label the sheet draws beside it. Empty when it cannot be resolved, in which
## case the caller falls back to the node's own value.
func _key_for(slider: HSlider, ws: Node) -> String:
	if ws == null or not ws.has_method("get"):
		return ""
	var row := slider.get_parent()
	while row != null and not (row is VBoxContainer):
		row = row.get_parent()
	if row == null:
		return ""
	var text := ""
	for n in _all(row):
		if n is Label:
			text = String((n as Label).text)
			break
	if text.is_empty():
		return ""
	for k in app.bridge.param_keys():
		var info: Dictionary = app.bridge.param_info(String(k))
		if String(info.get("label", "")) == text:
			return String(k)
	return ""
