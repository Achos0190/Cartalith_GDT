extends Node
## **Every phone control that is NOT a `Range`, and what a vertical swipe that
## begins on it does.**
##
## `_rangeswipe_probe.gd` censused one base class and stopped where that class
## ended. This one starts from the observation the main loop made on glass --
## a swipe from (540,1600) to (540,1100) on the New World card opened the
## Archetype dropdown instead of scrolling the card, while the same swipe from
## the label column at x=200 scrolled normally -- and asks the same three
## questions of every other control class the phone tree actually holds:
##
##   1. does it act on touch-DOWN, or on release?
##   2. does it consume the drag, so the scroller never sees it?
##   3. is it reachable inside a live vertical scroller on a phone, and how many?
##
## `--census-only` answers 3 by walking the tree. The swipe legs answer 1 and 2
## by driving the real hit-test, each with a positive control -- a tap at the
## same point that MUST do the thing, so "the swipe changed nothing" cannot be
## confused with "the gesture never reached the control".
##
##   godot --path . _gestclass_probe.tscn -- --force-touch --vp 1080x2340 --tag g1080
##   godot --path . _gestclass_probe.tscn -- --force-touch --vp 1440x3168 --tag g1440
##   godot --path . _gestclass_probe.tscn -- --force-touch --vp 720x1600  --tag g720
##   godot --path . _gestclass_probe.tscn -- --force-touch --vp 800x1280  --tag tab800
##
## Flags this probe actually reads, grepped from the body below rather than
## assumed (`MISTAKES.md`, "Write a probe's usage header"):
##   `--vp WxH`      SubViewport size in physical px. Default 1080x2340.
##   `--tag NAME`    prefix on every output line. Default `gc`.
##   `--census-only` skip the swipe legs; print the inventory and quit.
##   `--force-touch` NOT read here -- `dcc_shell.gd` reads it out of
##                   `OS.get_cmdline_user_args()`, and without it the shell
##                   boots desktop and every line below is void.
## Any other `--flag` aborts rather than being silently ignored.
##
## **Run it windowed.** Nothing here samples a pixel, but every act depends on
## the GUI hit-test and on `ScrollContainer`'s touch drag, both of which want a
## real frame loop.

## `MOUSE_FILTER_STOP` is **0**, `PASS` 1, `IGNORE` 2 -- the reverse of the
## order the words are usually said in, and the first cut of this probe printed
## every `STOP` control as `IGNORE` for exactly that reason. Indexed by the
## enum, not by a guess at it.
const _FILTER := ["STOP", "PASS", "IGNORE"]

var app: Node
var _vp: SubViewport
var _tag := "gc"
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
	await _frames(4)

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

## The same path `_rangeswipe_probe.gd::_jitter()` drives: three samples whose
## sideways travel exceeds their downward travel -- the pivot a thumb makes
## before it slides -- and then straight down. Copied deliberately rather than
## imported, so this probe holds the arbitration to the identical standard.
func _jitter(dir: float, reach: float) -> Array:
	var out: Array = []
	var xs := [3.0, 6.0, 5.0, 8.0, 10.0, 6.0, 4.0, 2.0, 0.0, -2.0, -4.0, -4.0, -3.0]
	var ys := [1.0, 2.0, 4.0, 6.0, 10.0, 20.0, 40.0, 80.0, 140.0, 210.0, 290.0, 380.0, 468.0]
	var span: float = ys[ys.size() - 1]
	for i in xs.size():
		out.append(Vector2(float(xs[i]), dir * float(ys[i]) / span * reach))
	return out

# -- Tree walking ---------------------------------------------------------------

## **`include_internal = true`, and that is not a detail.** The first cut of
## this walk used the default `get_children()`, which omits every INTERNAL
## child -- 4 401 nodes against 5 512 at 1080x2340. What it hid was not
## padding: **12 `SpinBoxLineEdit`** (5 in `new_world_dialog.gd`, 5 in the
## slicer `AcceptDialog`, 2 in `asset_library_window.gd`) and a `TabBar` in
## `world_data_window.gd`'s `TabContainer` -- all `LineEdit`-derived, all
## acting on press, all live under live vertical scrollers, and all absent
## from **both** the before and after census numbers. The class this probe
## exists to count was undercounted by the walk that counts it.
##
## Measured on the surface that flagged it: a jittered vertical swipe on the
## New World **Seed** `SpinBox` gives scroll `0 -> 0` with its internal
## `SpinBoxLineEdit` focused -- which on Android raises the soft keyboard over
## the sheet -- while the label column at the same `y` scrolls `0 -> 62`.
func _walk(root: Node, out: Array) -> void:
	for c in root.get_children(true):
		out.append(c)
		_walk(c, out)

func _all(root: Node) -> Array:
	var out: Array = []
	_walk(root, out)
	return out

func _scroller_of(n: Node) -> ScrollContainer:
	var p: Node = n.get_parent()
	while p != null:
		if p is ScrollContainer:
			return p as ScrollContainer
		p = p.get_parent()
	return null

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
			var scr: Script = p.get_script()
			return "Window %s" % (scr.resource_path.get_file() if scr != null
				else p.get_class())
		p = p.get_parent()
	return "phone root"

## **Question 1, answered from the engine rather than from a class list.**
## For a `BaseButton` the answer is a property -- `ACTION_MODE_BUTTON_PRESS`
## fires on touch-DOWN and `ACTION_MODE_BUTTON_RELEASE` does not -- so it is
## read off the live node, not asserted from the class name. Measured on 4.7.1
## for the classes this shell builds: `OptionButton` 0 (PRESS),
## `MenuButton` 0 (PRESS), `CheckBox` 1, `ColorPickerButton` 1, `Button` 1.
##
## The text and item classes have no such property; each is named with the
## behaviour of its own C++ handler, and each is then MEASURED by a leg below
## rather than believed from this table.
func _acts_down(n: Control) -> bool:
	if n is BaseButton:
		return (n as BaseButton).action_mode == BaseButton.ACTION_MODE_BUTTON_PRESS
	## `LineEdit::gui_input` and `TextEdit::gui_input` both set the caret and
	## grab focus from the PRESS, which on Android also raises the soft
	## keyboard -- measured by `_text_leg()`.
	if n is LineEdit or n is TextEdit:
		return true
	## `ItemList` and `Tree` select from the press. Neither is constructed
	## anywhere in `shell/` (grep, 2026-09-07) -- kept so the census reports
	## one if a `.tscn` ever carries one, instead of silently skipping it.
	if n is ItemList or n is Tree or n is TabBar:
		return true
	return false

## **Question 2.** A `MOUSE_FILTER_STOP` control ends the event walk, so the
## `ScrollContainer` above it never sees the press and cannot arm its drag.
## `MOUSE_FILTER_PASS` forwards what the control does not accept -- which is
## the cooperation `dcc_shell.gd::phone_fit()` already relies on for plain
## `Button`s. `IGNORE` is not picked at all.
func _consumes_drag(n: Control) -> bool:
	return n.mouse_filter == Control.MOUSE_FILTER_STOP

## Whether this pass has given the node its own arbitration.
##
## **This read the meta key `pg_touch_hold`, which NOTHING in the project
## sets** -- it occurred exactly once in the tree, here, in the line that read
## it. So `_arbitrated()` was unconditionally `false` and every census row
## printed `arb=NO`, which reads as a measured negative and is not one. It
## changed no number, because the walk skips `Range` and so never reached a
## node `touch_slider()` had touched, but it is the exact shape this batch was
## told to hunt: something declared in a probe, never true, reading as
## coverage.
##
## Replaced with the two properties `touch_release_button()` actually writes.
## `action_mode` is the engine's own switch and `mouse_filter` is the other
## half -- mutation M2 showed neither alone is sufficient -- so a node that
## carries both is arbitrated by definition rather than by a marker somebody
## has to remember to set.
func _arbitrated(n: Control) -> bool:
	if n is BaseButton:
		return (n as BaseButton).action_mode == BaseButton.ACTION_MODE_BUTTON_RELEASE \
			and n.mouse_filter == Control.MOUSE_FILTER_PASS
	return false

func _census(where: String) -> Dictionary:
	var rows: Dictionary = {}
	var hazard := 0
	var eaters := 0
	var total := 0
	var per_class: Dictionary = {}
	var eat_class: Dictionary = {}
	for n in _all(get_tree().root):
		if not (n is Control) or n is Range:
			continue
		var c := n as Control
		var down := _acts_down(c)
		var eats := _consumes_drag(c)
		if not (down or eats):
			continue        ## Not a candidate at all; not counted.
		total += 1
		var sc := _scroller_of(c)
		var scrolls: bool = sc != null \
			and sc.vertical_scroll_mode != ScrollContainer.SCROLL_MODE_DISABLED
		var live := true
		if c is BaseButton:
			live = not (c as BaseButton).disabled
		elif c is LineEdit:
			live = (c as LineEdit).editable
		elif c is TextEdit:
			live = (c as TextEdit).editable
		var am := -1
		if c is BaseButton:
			am = (c as BaseButton).action_mode
		var key := "%s | %s | scroller=%s | down=%s | filter=%s | arb=%s | live=%s" % [
			_surface_of(c), c.get_class(),
			("none" if sc == null else ("vDISABLED" if not scrolls else "vSCROLLS")),
			("YES(am=%d)" % am) if down else ("no(am=%d)" % am if am >= 0 else "no"),
			_FILTER[c.mouse_filter],
			("yes" if _arbitrated(c) else "NO"),
			str(live)]
		rows[key] = int(rows.get(key, 0)) + 1
		if scrolls and live and down and not _arbitrated(c):
			hazard += 1
			per_class[c.get_class()] = int(per_class.get(c.get_class(), 0)) + 1
		elif scrolls and live and eats:
			## **The second mechanism, reported separately.** A control that
			## acts on RELEASE is not the §1.14 defect -- nothing is written
			## silently -- but a `MOUSE_FILTER_STOP` one still ends the event
			## walk, so the scroller above it never arms its drag and that
			## patch of the sheet cannot be scrolled at all. PH-05 converted
			## most `BaseButton`s to `PASS` for this reason; whatever is left
			## here is what that conversion does not reach.
			eaters += 1
			eat_class[c.get_class()] = int(eat_class.get(c.get_class(), 0)) + 1
	_log("-- census (%s): %d non-Range candidates" % [where, total])
	var keys: Array = rows.keys()
	keys.sort()
	for k in keys:
		_log("   %4d  %s" % [rows[k], k])
	_log("   UNARBITRATED touch-DOWN control inside a live vertical scroller: %d  [%s]"
		% [hazard, _tally(per_class)])
	_log("   RELEASE-acting but MOUSE_FILTER_STOP in a live vertical scroller: %d  [%s]"
		% [eaters, _tally(eat_class)])
	return {"total": total, "hazard": hazard, "eaters": eaters}

func _tally(d: Dictionary) -> String:
	var pk: Array = d.keys()
	pk.sort()
	var out := ""
	for k in pk:
		out += " %s=%d" % [k, d[k]]
	return out.strip_edges()

func _find_label(text: String) -> Label:
	for n in _all(_vp):
		if n is Label and (n as Label).is_visible_in_tree() \
				and String((n as Label).text) == text:
			return n as Label
	return null

func _pressable_ancestor(n: Node) -> Button:
	var p := n
	while p != null:
		if p is Button:
			return p as Button
		p = p.get_parent()
	return null

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
	var cell := _pressable_ancestor(target)
	var at: Vector2 = (cell if cell != null else target).get_global_rect().get_center()
	await _tap_at(at)
	await _frames(8)
	return true

## **Where to push an event so it lands on `ctl`.**
##
## A control inside an embedded subwindow does NOT report the probe viewport's
## coordinates: `get_global_rect()` is in that `Window`'s own content space,
## and `DccWidgets.phone_present()` sets `content_scale_factor` to
## `phone_scale()` (2.621 at 1080x2340), so the two spaces differ by both a
## translation and a scale. Pushing the raw rect centre put every New World
## gesture roughly a third of the way up the screen, onto the map behind the
## card -- the first cut of this probe reported the dropdown "unchanged by a
## swipe" and its own positive control caught it, which is the whole reason the
## tap leg is there.
##
## `Window.get_final_transform()` is the content-space -> window-space
## transform (the content scale), and an embedded window's `position` is
## relative to its embedder. Composed, that is the point to push.
func _screen_pt(ctl: Control, local: Vector2) -> Vector2:
	var win := ctl.get_window()
	var p: Vector2 = ctl.get_global_transform() * local
	if win != null and win != _vp and win.is_embedded():
		p = win.get_final_transform() * p + Vector2(win.position)
	return p

## What the hit-test actually reports under `at` -- the check that turns "the
## swipe changed nothing" into "the swipe reached the control and changed
## nothing". Asked of the control's OWN viewport, because an embedded window
## runs its own GUI pick.
func _hovered_under(ctl: Control, at: Vector2) -> String:
	var hover := InputEventMouseMotion.new()
	hover.position = at
	hover.global_position = at
	_vp.push_input(hover, true)
	await _frames(2)
	var vp: Viewport = ctl.get_viewport()
	var over := vp.gui_get_hovered_control() if vp != null else null
	return "<none>" if over == null else "%s(%s)" % [over.get_class(), over.name]

func _first_visible(root: Node, cls: String) -> Control:
	for n in _all(root):
		if n is Control and n.get_class() == cls and (n as Control).is_visible_in_tree():
			var sc := _scroller_of(n)
			if sc != null and sc.vertical_scroll_mode != ScrollContainer.SCROLL_MODE_DISABLED:
				return n as Control
	return null

# -- The swipe legs -------------------------------------------------------------

## One control, one jittered vertical swipe, then a plain tap at the same
## point. `read` returns the state that must NOT move on the swipe and MUST
## move on the tap -- the positive control, without which "nothing happened" is
## indistinguishable from "the gesture never arrived".
func _gesture_leg(label: String, ctl: Control, read: Callable,
		expect_tap_moves: bool = true) -> void:
	_log("-- vertical swipe with cross-axis jitter: %s" % label)
	if ctl == null:
		_check(false, "%s: found the control" % label)
		return
	var scroll := _scroller_of(ctl)
	if scroll == null:
		_check(false, "%s: found its scroller" % label)
		return
	scroll.ensure_control_visible(ctl)
	await _frames(6)
	var r := ctl.get_global_rect()
	var s0 := scroll.scroll_vertical
	var v0: Variant = read.call()
	var bar := scroll.get_v_scroll_bar()
	## **Swipe towards whichever end has room**, and say how much there was.
	## `_rangeswipe_probe.gd`'s rule -- swipe up only when more than 200 px of
	## range remains -- is right for the left dock sheet and wrong for a short
	## card: the New World card scrolls 62 px, so that rule chose "finger down"
	## at `scroll_vertical == 0`, where nothing can move, and the leg reported
	## "does not scroll" for a reason that had nothing to do with the control.
	var room_up: float = bar.max_value - bar.page - float(s0)   ## finger UP
	var room_down: float = float(s0)                            ## finger DOWN
	var dir := -1.0 if room_up >= room_down else 1.0
	var room: float = maxf(room_up, room_down)
	_check(room > 8.0, "%s: its scroller has somewhere to go (%.0f px)" % [label, room])
	var reach: float = minf(float(_vp.size.y) * 0.20, maxf(120.0, room))
	var start := _screen_pt(ctl, ctl.size * 0.5)
	var under := await _hovered_under(ctl, start)
	_log("   %s rect %.0f,%.0f %.0fx%.0f  filter=%s  push at %.0f,%.0f  under=%s"
		% [ctl.get_class(), r.position.x, r.position.y, r.size.x, r.size.y,
			_FILTER[ctl.mouse_filter], start.x, start.y, under])
	_log("   scroll %d of %.0f  dir %s  reach %.0f"
		% [s0, bar.max_value - bar.page, "up" if dir < 0.0 else "down", reach])
	## **The gesture has to arrive before anything it does means anything.**
	## `MISTAKES.md`: a probe that synthesises input downstream of the hit-test
	## proves nothing about reachability, so this asserts the pick.
	_check(under.begins_with(ctl.get_class()),
		"%s: the push point picks the control itself (under=%s)" % [label, under])
	await _push_path(start, _jitter(dir, reach))
	var v1: Variant = read.call()
	var s1 := scroll.scroll_vertical
	_log("   state %s -> %s   scroll %d -> %d" % [str(v0), str(v1), s0, s1])
	_check(str(v1) == str(v0),
		"%s: a jittered vertical swipe leaves the state unchanged (%s vs %s)"
			% [label, str(v0), str(v1)])
	_check(s1 != s0, "%s: and DOES scroll the container (%d -> %d)" % [label, s0, s1])
	## The positive control. Without it the two checks above pass on a control
	## the gesture never reached at all.
	if not expect_tap_moves:
		return
	var v2: Variant = read.call()
	await _tap_at(_screen_pt(ctl, ctl.size * 0.5))
	var v3: Variant = read.call()
	_log("   plain tap at the same point: %s -> %s" % [str(v2), str(v3)])
	_check(str(v3) != str(v2),
		"%s: and a plain tap at the same point STILL acts (%s -> %s)"
			% [label, str(v2), str(v3)])

# -- Boot ----------------------------------------------------------------------

func _ready() -> void:
	_tag = _arg("--tag", "gc")
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
	## The gate `ScrollContainer`'s own touch drag sits behind -- see
	## `_scrolldrag_probe.gd`'s header, which verified this against 4.7.1.
	Input.set_emulate_touch_from_mouse(true)
	app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await get_tree().create_timer(1.6).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	if app.phone_project_picker != null and app.phone_project_picker.visible:
		app.phone_project_picker.hide()
	await _frames(6)

	_log("viewport %dx%d  phone=%s  tablet=%s  scale=%.3f  touchscreen=%s" %
		[_vp.size.x, _vp.size.y, app.is_phone(), DccTheme.is_tablet(),
			app.phone_scale(), str(DisplayServer.is_touchscreen_available())])

	var before := _census("boot, docks unopened")

	if app.is_phone():
		app._set_sheet_open("left", true)
		await _frames(8)
		_log("   (left sheet opened by calling `_set_sheet_open` -- staging, not")
		_log("    a claim that the route is reachable by a finger)")
		_census("left sheet open")

	# -- Leg 1: the surface the defect was observed on -------------------------
	#
	# `new_world_dialog.gd`: six `OptionButton`s and four `SpinBox`es in a card
	# that MUST be scrolled to reach CREATE WORLD.
	var dlg: Window = app.new_world_dialog
	if dlg != null:
		## `app.gd::open_new_world()`'s own route, not `popup_centered()`: on a
		## phone the dialog is presented by `DccWidgets.phone_present()`, which
		## is what fills the screen and runs `phone_fit()` over the card.
		## Calling the menu handler rather than the presenter so the probe
		## exercises the shell's path and not a shortcut around it.
		app.open_new_world()
		await _frames(20)
		_census("New World dialog open")
		if not _flag("--census-only"):
			var ob := _first_visible(dlg, "OptionButton") as OptionButton
			if ob != null:
				var pop := ob.get_popup()
				await _gesture_leg("New World OptionButton", ob,
					func(): return "sel=%d popup=%s" % [ob.selected, str(pop.visible)])
				if pop.visible:
					pop.hide()
					await _frames(4)
			else:
				_check(false, "the New World card draws an OptionButton in its scroller")
		if dlg.has_method("hide"):
			dlg.hide()
		await _frames(6)

	# -- The states the boot census structurally cannot see -------------------
	#
	# `MISTAKES.md`: **a live-tree census is a LOWER BOUND taken in ONE state,
	# and the state has to be named.** §1.15's slider census walked a world-less
	# boot and silently missed four `phone_menu.gd::_slider_row()` sites,
	# because `tl_available()` is false without a world and that screen draws
	# `_missing_row("Year")` instead. Both extra states below are reported with
	# their own number rather than folded into the first.
	## **`--census-only`, and that is a real restriction rather than tidiness.**
	## Reaching these states leaves the MORE overlay standing over the docks and
	## a generated world under them, and the swipe legs below need a control
	## they can push at a known point. Run this way round the first time, the
	## left-dock leg pushed at a rect the MORE screen was covering and reported
	## `under=VBoxContainer` -- caught by the pick assertion, which is what that
	## assertion is for.
	if _flag("--census-only") and app.is_phone():
		app._set_sheet_open("left", false)
		await _frames(4)
		if await _tap_caption("PLAN"):
			_census("after tapping PLAN (the planner form now exists)")
		if await _tap_caption("MORE"):
			_census("after tapping MORE")
		if await _tap_caption("Simulation"):
			_census("MORE > Simulation")

	## **The world-loaded state.** Everything above is a world-LESS boot, which
	## is the state `_rangeswipe_probe.gd` was measured in and the state whose
	## incompleteness that probe's own comment records. A generate is the only
	## way to reach the other one, so it is run here and the census retaken --
	## and if it does not finish inside the timeout, the run says so rather than
	## reporting the world-less number as if it were the whole answer.
	if _flag("--census-only") and app.is_phone() and app.bridge != null:
		_log("-- generating a world to retake the census in the other state")
		var done := [false]
		app.bridge.generation_finished.connect(func(ok: bool): done[0] = true,
			CONNECT_ONE_SHOT)
		app._run_pipeline()
		var waited := 0.0
		while not done[0] and waited < 180.0:
			await get_tree().create_timer(0.5).timeout
			waited += 0.5
		if done[0]:
			await _frames(30)
			_log("   (generate finished after %.1fs)" % waited)
			_census("WORLD LOADED, docks as they stand")
			app._set_sheet_open("left", true)
			await _frames(10)
			_census("WORLD LOADED, left sheet open")
			app._set_sheet_open("left", false)
			await _frames(4)
			if await _tap_caption("MORE"):
				if await _tap_caption("Simulation"):
					_census("WORLD LOADED, MORE > Simulation")
		else:
			_log("   NOT MEASURED: the generate did not finish in %.0fs, so every"
				% waited)
			_log("   census above is the world-LESS state and nothing here says")
			_log("   what a world-loaded one holds.")

	if _flag("--census-only"):
		_log("RESULT %s fail=%d (census only)" % [_tag, _fail])
		get_tree().quit(1 if _fail > 0 else 0)
		return

	# -- Leg 2: the same class in the left dock sheet, the biggest population --
	if app.is_phone() and app.left_dock != null:
		app._set_sheet_open("left", true)
		await _frames(10)
		var ob2 := _first_visible(app.left_dock, "OptionButton") as OptionButton
		if ob2 != null:
			var pop2 := ob2.get_popup()
			await _gesture_leg("left dock OptionButton", ob2,
				func(): return "sel=%d popup=%s" % [ob2.selected, str(pop2.visible)])
			if pop2.visible:
				pop2.hide()
				await _frames(4)
		else:
			_check(false, "the left sheet draws an OptionButton in its scroller")

		# -- Leg 3: `ColorPickerButton` -- RELEASE-acting, and `MOUSE_FILTER_STOP`.
		#
		# It is NOT the touch-DOWN defect: `action_mode` is 1, measured. The
		# question it is here to answer is the SECOND one -- whether a `STOP`
		# control that phone_fit() deliberately leaves at `STOP` still lets the
		# sheet scroll under it. Its own picker is a `PopupPanel`, so `visible`
		# on that popup is the state to watch.
		## The seven the census counts in the left sheet live in the CARTO and
		## RENDER panels, and `_select_domain()` shows exactly one panel at a
		## time -- so the first cut of this leg searched the dock, found none
		## visible and printed "no ColorPickerButton", which would have read as
		## "the class is not here". Switching domain until one is visible is
		## what makes the skip mean what it says.
		var cpb: ColorPickerButton = null
		for dom in ["cartography", "render", "world", "civilization"]:
			app.select_domain(dom)
			await _frames(10)
			cpb = _first_visible(get_tree().root, "ColorPickerButton") as ColorPickerButton
			if cpb != null:
				_log("   (found a visible ColorPickerButton in domain `%s`)" % dom)
				break
		if cpb != null:
			var pk := cpb.get_popup()
			await _gesture_leg("left dock ColorPickerButton", cpb,
				func(): return "colour=%s popup=%s" % [str(cpb.color), str(pk.visible)],
				false)
			if pk.visible:
				pk.hide()
				await _frames(4)
		else:
			_log("   (no ColorPickerButton visible with a live vertical scroller in")
			_log("    any of the four domains -- the census counts 7 in the left")
			_log("    sheet, so this is a staging gap, NOT an absence)")

		# -- Leg 4: `CheckBox` -- the negative control of the whole census.
		#
		# `action_mode` 1 and `MOUSE_FILTER_PASS` (phone_fit() converts it), so
		# it should already cooperate with the scroller exactly as a plain
		# `Button` does. Measured rather than assumed, because "the class is
		# fine" is the claim most worth being wrong about.
		app.select_domain("world")
		await _frames(10)
		var cb2 := _first_visible(get_tree().root, "CheckBox") as CheckBox
		if cb2 != null:
			await _gesture_leg("left dock CheckBox", cb2,
				func(): return str(cb2.button_pressed))
		else:
			_log("   (no CheckBox inside the left sheet's scroller)")

		# -- Leg 5: `LineEdit` -- the other touch-DOWN class the census found.
		#
		# It writes no value on a press, but it takes FOCUS from one, and on
		# Android focus on a `LineEdit` raises the soft keyboard over the sheet
		# the swipe was trying to scroll. Focus is the state asserted.
		var le := _first_visible(get_tree().root, "LineEdit") as LineEdit
		if le == null and app.asset_library_window != null:
			## The census counts 3 in this window's scroller and 1 in the left
			## sheet; none is visible in the boot state, so the window is opened
			## rather than the class being reported absent.
			app.asset_library_window.open()
			await _frames(20)
			le = _first_visible(get_tree().root, "LineEdit") as LineEdit
			if le != null:
				_log("   (found a visible LineEdit by opening the asset library)")
		if le != null:
			await _gesture_leg("left dock LineEdit", le,
				func(): return "focus=%s caret=%d" % [str(le.has_focus()), le.caret_column])
			le.release_focus()
		else:
			_log("   (no LineEdit visible with a live vertical scroller in this")
			_log("    state -- a staging gap, NOT an absence: the census counts 4)")
		if app.asset_library_window != null and app.asset_library_window.visible:
			app.asset_library_window.hide()
			await _frames(4)
		app._set_sheet_open("left", false)
		await _frames(4)

	if _flag("--census-only"):
		_log("RESULT %s fail=%d (census only)" % [_tag, _fail])
		get_tree().quit(1 if _fail > 0 else 0)
		return

	_log("RESULT %s fail=%d  census_hazard_at_boot=%d" % [_tag, _fail, before["hazard"]])
	get_tree().quit(1 if _fail > 0 else 0)
