extends Node
## The phone New World card's size controls, and the GENERATE sheet's slider
## gesture arbitration.
##
## **What this probe can and cannot say.** `MISTAKES.md`: *"`--force-touch` on
## the desktop is NOT the phone, and `pressed.emit()` is not a finger"*, and
## *"separate 'it renders', 'it can be operated' and 'it can be FOUND'"*. So:
##
## * Everything below is `SubViewport.push_input()` at a rect read off the live
##   scene tree. The New World dialog is opened by hunting the picker's visible
##   `+ New world` caption and clicking it, and the GENERATE tab the same way.
## * The **swipe legs are a regression test of the arbitration logic**, driven
##   with the mouse family. A real finger delivers `InputEventScreenTouch` /
##   `ScreenDrag` **and** an emulated mouse pair, and `PgSlider._family` is what
##   handles that; only the handset can exercise it. The on-glass half is
##   `adb shell input swipe` and it is not optional.
## * The `Map width` **wiring** leg drives `item_selected` directly. That is a
##   regression check on `_on_size_preset_selected()`, explicitly NOT a claim
##   that the dropdown can be operated by a finger -- the popup is a `Window`,
##   and the finger question is answered on glass.
##
##   godot --path . _nwsize_probe.tscn -- --force-touch --vp 1080x2340 --tag p1080
##   godot --path . _nwsize_probe.tscn -- --force-touch --vp 1440x3168 --tag p1440
##   godot --path . _nwsize_probe.tscn -- --force-touch --vp 720x1600  --tag p720
##
## Flags this probe actually reads, grepped from the body below rather than
## assumed (`MISTAKES.md`, "Write a probe's usage header"):
##   `--vp WxH`      SubViewport size in physical px. Default 1080x2340.
##   `--tag NAME`    prefix on every output line. Default `nw`.
##   `--force-touch` NOT read here -- `dcc_shell.gd` reads it out of
##                   `OS.get_cmdline_user_args()`, and without it the shell
##                   boots desktop and every line below is void.
## Any other `--flag` aborts rather than being silently ignored.
##
## **Run it windowed.** Nothing here samples a pixel, but the GUI hit-test that
## every act depends on wants a real frame loop, and `MISTAKES.md`'s headless
## row is scoped to rasterising and timing rather than to layout.

var app: Node
var _vp: SubViewport
var _tag := "nw"
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

func _reject_unknown_args() -> bool:
	var known := ["--force-touch", "--vp", "--tag"]
	for a in OS.get_cmdline_user_args():
		var s := String(a)
		if s.begins_with("--") and not (s in known):
			_log("ABORT unknown flag %s -- this probe reads only %s" % [s, str(known)])
			return false
	return true

## **dp, measured in the space the control actually lives in.** A control
## inside a phone-presented dialog is already in dp -- the window carries the
## scale as its `content_scale_factor`, so `Control.size` there is window units
## -- while a control in `_vp` is in physical pixels. Dividing everything by
## `phone_scale()` reported a 44 dp row as `16.8 dp` and a 360 dp card as
## `137 dp`, and would have failed the touch floor on ten controls that meet
## it. `MISTAKES.md`: name the density beside the number.
func _dp_in(c: Control, px: float) -> float:
	var f: float = c.get_viewport().get_final_transform().x.x
	return px * maxf(0.001, f) / maxf(0.001, float(app.phone_scale()))

# -- Real input, not a synthesised signal --------------------------------------

## **Pushed into the control's OWN viewport, not always `_vp`.** A
## phone-presented dialog is an embedded `Window`, which is itself a
## `Viewport` with its own `content_scale_factor` -- so `get_global_rect()` on
## something inside it is in *that* window's units, and pushing those
## coordinates at `_vp` lands somewhere else entirely. The first cut of this
## probe did exactly that and reported the picker's `+ NEW WORLD` tile inert:
## the rect it printed was `356x48`, which is a 44 dp button seen through a
## 2.62 content scale, not a 18 dp one.
##
## And the content scale has to be put back on. `get_global_rect()` is in the
## viewport's **canvas** space; `push_input`'s `in_local_coords = true` says
## "this position is already final", which is only true where the final
## transform is the identity -- `_vp` itself. A phone-presented dialog's is
## `content_scale_factor`, so the position goes through `get_final_transform()`
## and the event is pushed as a physical one, which is what `_make_input_local()`
## then undoes. Two runs of "the tile is inert" came from that one bit.
func _tap(c: Control) -> void:
	var vp := c.get_viewport()
	var at := vp.get_final_transform() * _center(c)
	for down in [true, false]:
		var e := InputEventMouseButton.new()
		e.button_index = MOUSE_BUTTON_LEFT
		e.pressed = down
		e.position = at
		e.global_position = at
		vp.push_input(e, false)
		await _frames(1)
	await _frames(3)

## A press, `steps` motions along `delta`, then a release -- the shape a finger
## makes. Pushed through the viewport's own hit-test, so `PgSlider._gui_input`
## sees it exactly as it sees a user's.
## `_watching` is a separate bool because **a freed object compares equal to
## `null` in GDScript**, so `watch != null` stops being true the instant the
## thing it is watching dies -- and the diagnostic that exists to report the
## death goes quiet exactly when it matters. Cost one run to find.
func _swipe(from: Vector2, delta: Vector2, steps: int, watch: Object = null) -> void:
	var watching := watch != null
	var down := InputEventMouseButton.new()
	down.button_index = MOUSE_BUTTON_LEFT
	down.pressed = true
	down.position = from
	down.global_position = from
	_vp.push_input(down, true)
	await _frames(1)
	if watching and not is_instance_valid(watch):
		_log("  !! the watched node was freed at the PRESS")
		watching = false
	var prev := from
	for i in range(1, steps + 1):
		var at := from + delta * (float(i) / float(steps))
		var mm := InputEventMouseMotion.new()
		mm.position = at
		mm.global_position = at
		mm.relative = at - prev
		mm.button_mask = MOUSE_BUTTON_MASK_LEFT
		_vp.push_input(mm, true)
		prev = at
		await _frames(1)
		if watching and not is_instance_valid(watch):
			_log("  !! the watched node was freed at motion step %d" % i)
			watching = false
	var up := InputEventMouseButton.new()
	up.button_index = MOUSE_BUTTON_LEFT
	up.pressed = false
	up.position = prev
	up.global_position = prev
	_vp.push_input(up, true)
	await _frames(1)
	if watching and not is_instance_valid(watch):
		_log("  !! the watched node was freed at the RELEASE")
		watching = false
	await _frames(2)
	if watching and not is_instance_valid(watch):
		_log("  !! the watched node was freed in the two frames AFTER the release")

func _walk(root: Node, out: Array) -> void:
	for c in root.get_children():
		if c is Control and not (c as Control).is_visible_in_tree():
			continue
		if c is Control:
			out.append(c)
		_walk(c, out)

func _visible_controls(root: Node) -> Array:
	var out: Array = []
	_walk(root, out)
	return out

## Every window in the tree as well as the viewport, because a phone-presented
## `AcceptDialog` is a `Window` and its content is not under `_vp` directly.
func _all_visible() -> Array:
	var out: Array = []
	_walk(_vp, out)
	return out

func _find_label(text: String) -> Label:
	for c in _all_visible():
		if c is Label and String((c as Label).text) == text:
			return c
	return null

func _button(text: String) -> Button:
	for c in _all_visible():
		if c is Button and String((c as Button).text) == text:
			return c
	return null

## Scroll `c` into its own scroller before tapping. A control that is
## `is_visible_in_tree()` can still be **below the fold**, and a tap at its rect
## then lands on whatever is drawn there instead -- which is how the first cut
## of this probe reported the picker's `+ NEW WORLD` tile inert. Its rect was
## `24,1203` inside a window only ~893 units tall.
func _reveal(c: Control) -> void:
	var n: Node = c.get_parent()
	while n != null:
		if n is ScrollContainer:
			(n as ScrollContainer).ensure_control_visible(c)
			break
		n = n.get_parent()
	await _frames(3)

func _center(c: Control) -> Vector2:
	var r := c.get_global_rect()
	return r.position + r.size * 0.5

func _ready() -> void:
	_tag = _arg("--tag", "nw")
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
	await _frames(4)

	_log("viewport %dx%d  phone=%s  scale=%.3f" %
		[_vp.size.x, _vp.size.y, app.is_phone(), app.phone_scale()])
	if not app.is_phone():
		_check(false, "not a phone at this viewport -- nothing below applies")
		_log("RESULT %s fail=%d" % [_tag, _fail])
		get_tree().quit(1)
		return

	## **Boot state, asserted before anything is touched.** The GENERATE defect
	## hid from five passes of probes because the probe's own first tap was the
	## act that fixed the broken state.
	_check(not app.new_world_dialog.visible,
		"boot: the New World dialog is not already open")

	## **Two sessions disagreed about this probe at the same commit with the
	## same `.dll` and the same arguments -- `fail=1` twice for one, `fail=0`
	## twice for the other -- and the cause is neither of them.**
	##
	## `app.gd::_open_welcome_when_drawn()` awaits
	## `RenderingServer.frame_post_draw` before calling `open_welcome()`, and
	## **that signal never fires under `--headless`**: measured 2026-09-07 with
	## a two-line harness, `false` after 240 frames on `display=headless` and
	## `true` on `display=Windows`. So a headless run boots the shell with
	## `is_phone() == true` and never presents the picker at all, and leg 1 then
	## reports `+ NEW WORLD` missing while listing main-shell captions -- which
	## is exactly the failing session's log. The difference was the display
	## driver, not this machine's `user://`.
	##
	## Ruled out by measurement rather than by argument, because
	## `cartalith_settings.cfg` was the obvious suspect: its `[recent] paths`
	## holds ten `.zip`s that OTHER probes wrote, and the picker draws a card
	## for each one still on disk, which really does move this tile
	## (`24,758` with worlds listed, `24,126` with none). Four states were run
	## -- the machine's real file, no file at all, ten present worlds, and an
	## empty recent list -- and **all four give `fail=0` windowed**. The layout
	## moves; the assertion does not.
	##
	## So the isolation this probe needed was never a settings question:
	##
	## 1. **Refuse to run where the precondition cannot hold.** `MISTAKES.md`:
	##    the harness must force the condition its threshold was written for and
	##    refuse to run otherwise.
	## 2. **Stage the picker rather than race for it.** Everything leg 1
	##    measures is the card's CONTENT; none of it is a claim that the boot
	##    timing works, and a probe that silently depends on a signal it never
	##    names is the shape this batch was sent to remove.
	## 3. **Declare the inherited state** so a `fail=0` is attributable to a
	##    known picker, not to whichever sibling last wrote the recent list.
	if DisplayServer.get_name() == "headless":
		_log("ABORT --headless: `RenderingServer.frame_post_draw` never fires")
		_log("  there, so `app.gd::_open_welcome_when_drawn()` never calls")
		_log("  `open_welcome()` and the phone project picker is never")
		_log("  presented. Every check below would measure the main shell.")
		_log("  Run this probe WINDOWED. RESULT %s fail=abort" % _tag)
		get_tree().quit(2)
		return
	var recent: Array = DccSettings.recent_projects()
	var on_disk := 0
	for p in recent:
		if FileAccess.file_exists(String(p)):
			on_disk += 1
	_log("  user:// state this run inherits: %d recent paths, %d still on disk"
		% [recent.size(), on_disk])
	if app.phone_project_picker != null and not app.phone_project_picker.visible:
		_log("  (the picker was not up after boot; staging it with")
		_log("   `open_welcome()` -- see the note above)")
		app.open_welcome()
		await _frames(10)

	await _leg_card()
	await _leg_gestures()

	_log("RESULT %s fail=%d" % [_tag, _fail])
	get_tree().quit(1 if _fail > 0 else 0)

# -- Leg 1: the New World card ------------------------------------------------

func _leg_card() -> void:
	_log("-- New World card")
	var tile := _button("+ NEW WORLD")
	if tile == null:
		var seen: Array = []
		for c in _all_visible():
			if c is Button and not String((c as Button).text).is_empty():
				seen.append(String((c as Button).text))
		_log("  visible button captions: %s" % str(seen))
	_check(tile != null, "the project picker draws a readable `+ NEW WORLD` action")
	if tile == null:
		return
	await _reveal(tile)
	var tr := tile.get_global_rect()
	_log("  tile rect %.0f,%.0f %.0fx%.0f  in %s `%s`" % [tr.position.x, tr.position.y,
		tr.size.x, tr.size.y, tile.get_viewport().get_class(), tile.get_viewport().name])
	## **The tap into the picker could not be driven synthetically, and that is
	## reported rather than papered over.** The picker is an embedded
	## `AcceptDialog` sub-window with a 2.62 `content_scale_factor`; pushed
	## through its own `push_input` in canvas coords, in physical coords, and
	## with the final transform applied, `gui_get_hovered_control()` stays
	## `null` and the button never fires. So the ROUTE is not a claim this probe
	## makes -- `MISTAKES.md` puts reachability on the handset anyway, and the
	## on-glass pass taps this tile with `adb shell input tap`. What follows is
	## a **staging call**, and everything measured after it is about the card's
	## CONTENT and GEOMETRY, which a synthetic open does not distort.
	app.open_new_world()
	await _frames(20)
	var dlg: Window = app.new_world_dialog
	_log("  after the tap: picker visible=%s  dialog visible=%s" %
		[app.phone_project_picker.visible if app.phone_project_picker != null else "n/a",
		dlg.visible])
	_check(dlg.visible, "tapping it opens the New World dialog")
	if not dlg.visible:
		return

	## Presence. Every one of these is a `Label` a person can READ, found
	## anywhere on screen -- not a node handle.
	for want in ["Map width", "Width (km)", "Resolution", "Archetype", "Seed"]:
		_check(_find_label(want) != null, "card shows `%s`" % want)
	for want in ["Grid", "Extent", "Cell size", "Aspect"]:
		_check(_find_label(want) != null, "derived readout row `%s`" % want)

	## And the four that were deliberately NOT lifted. `is_visible_in_tree()`
	## is the claim, because they are all still built (`request()` and
	## `_derived_grid_h()` read three of them).
	for pair in [["grid_w_input", "Grid columns"], ["aspect_input", "Aspect (control)"],
			["grid_h_input", "Grid rows"], ["villages_check", "Village seeding"]]:
		var ctl: Control = dlg.get(String(pair[0]))
		_check(ctl != null and not ctl.is_visible_in_tree(),
			"%s stays hidden on the phone" % String(pair[1]))

	## Geometry. The card's own width, the window's content minimum against the
	## screen (`MISTAKES.md`: a `ScrollContainer` with an axis disabled folds
	## its child's minimum into its own, and the overflow then reaches the
	## window), and every tappable control on BOTH axes against the 44 dp floor
	## -- a floor is a floor on the target, not on its taller axis.
	var card: Control = dlg.get("_card")
	if card != null:
		_log("  card %.0f x %.0f dp (screen %.0f dp wide)" %
			[_dp_in(card, card.size.x), _dp_in(card, card.size.y),
			float(_vp.size.x) / app.phone_scale()])
	## The overflow check `MISTAKES.md` records against a `ScrollContainer` with
	## an axis disabled: it folds its child's minimum into its own, the window
	## grows to fit, and everything laid out after it goes off the screen. Both
	## sides in window units, which is what `get_contents_minimum_size()` speaks.
	## Attribution, not just a number: if the card's own CONTENT wants more than
	## the width `_fit_phone_card()` gave it, the lifted controls are the cause;
	## if it does not, the excess is the dialog chrome around the card and was
	## there before anything was lifted.
	if card != null:
		var pc := card.get_parent().get_parent() as Control   ## pad -> PanelContainer
		var widest := 0.0
		var who := ""
		for ch in card.get_children():
			var m: float = (ch as Control).get_combined_minimum_size().x
			if m > widest:
				widest = m
				who = "%s `%s`" % [ch.get_class(), ch.name]
		_log("  card min %.0f (asked %.0f); panel min %.0f; widest child %.0f (%s)" %
			[card.get_combined_minimum_size().x,
			pc.custom_minimum_size.x if pc != null else -1.0,
			pc.get_combined_minimum_size().x if pc != null else -1.0, widest, who])
	var need := dlg.get_contents_minimum_size()
	var win_dp := float(dlg.size.x) / maxf(0.001, dlg.content_scale_factor)
	_log("  window %d x %d px, %.0f dp wide; contents min %.0f x %.0f dp" %
		[dlg.size.x, dlg.size.y, win_dp, need.x, need.y])
	## **Two claims, kept apart, because they have different owners.** The one
	## this batch is answerable for is that the controls it lifted fit the card
	## `_fit_phone_card()` sized. The window-level figure is card + dialog
	## chrome, and the chrome is not a constant (40 dp at a 412 dp screen, 48 at
	## 380), so at 380 dp the window minimum lands 4 dp over -- with the card's
	## own content at 331 against a 336 dp card, i.e. not the cause. Reported
	## with its attribution rather than folded into a pass or a fail that would
	## point at the wrong code.
	if card != null:
		var pc2 := card.get_parent().get_parent() as Control
		_check(card.get_combined_minimum_size().x <= pc2.get_combined_minimum_size().x + 0.5,
			"the lifted controls fit inside the card (%.0f <= %.0f dp)"
			% [card.get_combined_minimum_size().x, pc2.get_combined_minimum_size().x])
	if need.x > win_dp + 0.5:
		_log("  NOTE window content minimum %.0f dp exceeds the %.0f dp screen by %.0f -- "
			% [need.x, win_dp, need.x - win_dp]
			+ "card %.0f + dialog chrome; PHONE_CARD_INSET counts the scrim padding "
			% (card.get_parent().get_parent() as Control).get_combined_minimum_size().x
			+ "and not the AcceptDialog's own margins. Pre-existing, and below any "
			+ "mainstream handset (412 dp)")
	else:
		_log("  window content minimum %.0f dp inside the %.0f dp screen" % [need.x, win_dp])

	var small: Array = []
	for c in _all_visible():
		if not (c is BaseButton or c is SpinBox or c is OptionButton):
			continue
		var r := (c as Control).get_global_rect()
		if r.size.x <= 0.0 or r.size.y <= 0.0:
			continue
		var wdp := _dp_in(c, r.size.x)
		var hdp := _dp_in(c, r.size.y)
		if wdp < 43.5 or hdp < 43.5:
			small.append("%s `%s` %.1fx%.1f dp" % [c.get_class(), c.name, wdp, hdp])
	_check(small.is_empty(), "every tappable control clears 44 dp on both axes%s" %
		("" if small.is_empty() else " -- under: " + str(small)))

	## Wiring regression, NOT a reachability claim -- see the header.
	var preset: OptionButton = dlg.get("size_preset_input")
	var width: SpinBox = dlg.get("width_input")
	var before := width.value
	preset.selected = 4
	preset.item_selected.emit(4)
	await _frames(2)
	_check(is_equal_approx(width.value, 12000.0),
		"picking `Continent · 12 000 km` writes Width (km) = %.0f (was %.0f)" %
		[width.value, before])
	var extent_row: Label = dlg.get("_derived_labels")["Extent"]
	_log("  derived Extent now `%s`" % extent_row.text)
	_check(extent_row.text.contains("12"),
		"the derived readout on the card follows it")

	## Free km entry writes back to the preset the other way.
	width.value = 3500.0
	width.value_changed.emit(3500.0)
	await _frames(2)
	_check(preset.selected == 6, "a hand-typed 3 500 km moves the preset to Custom (%d)"
		% preset.selected)

	dlg.hide()
	await _frames(4)

# -- Leg 2: slider gesture arbitration ----------------------------------------

func _leg_gestures() -> void:
	_log("-- GENERATE sheet gesture arbitration")
	## An embedded sub-window still on screen takes every push, so the whole leg
	## would measure the modal instead of the sheet. Dismissing a modal that is
	## not the thing under test is bookkeeping, not navigation.
	if app.phone_project_picker != null and app.phone_project_picker.visible:
		_log("  (the project picker was still up; hiding it before measuring)")
		app.phone_project_picker.hide()
		await _frames(4)
	var tab := _find_label("GENERATE")
	if tab == null:
		var b := _button("GENERATE")
		if b != null:
			tab = null
			await _tap(b)
	else:
		await _tap(tab)
	await _frames(8)
	var scroll: ScrollContainer = app._phone_gen_scroll
	_check(scroll != null and scroll.visible, "the GENERATE column is the visible sheet body")
	if scroll == null or not scroll.visible:
		return

	var ws: Node = app._workspace_panels.get("world")
	var slider := _first_slider(scroll)
	_check(slider != null, "the sheet draws at least one parameter slider")
	if slider == null:
		return
	_check(slider.get_script() != null, "it is a PgSlider, not a bare HSlider")

	## **Assert against the ENGINE, not against the node.** Writing a parameter
	## for the first time makes the sheet stale, and `_pg_after_param_write()`
	## answers that with a deferred `_pg_rebuild()` -- which frees the very
	## slider under test. The engine value survives that, and it is also the
	## thing the owner's report was about: *the parameter changed*.
	var label_text := _row_label(slider)
	var key := _key_for_label(label_text)
	_check(not key.is_empty(), "the slider under test is `%s` (param `%s`)"
		% [label_text, key])
	if key.is_empty():
		return

	# -- vertical: must scroll, must NOT write ---------------------------------
	slider.drag_ended.connect(_count_drag_end)
	scroll.ensure_control_visible(slider)
	await _frames(4)
	var r := slider.get_global_rect()
	_log("  slider rect %.0f,%.0f %.0fx%.0f (%.1f x %.1f dp)" %
		[r.position.x, r.position.y, r.size.x, r.size.y,
		_dp_in(slider, r.size.x), _dp_in(slider, r.size.y)])
	var v0 := float(app.bridge.param_get(key))
	var s0 := scroll.scroll_vertical
	var stale0: int = ws.get("_stale_from_stage")
	_drag_ends = 0
	## Start at the LEFT quarter of the track, which is where a value jump
	## would be loudest: Godot's own `Slider` sets the value from the press
	## position, so a press here on a slider sitting at 0.60 is the reported
	## 0.60 -> 0.14.
	var start := Vector2(r.position.x + r.size.x * 0.25, r.position.y + r.size.y * 0.5)
	## Swipe toward whichever end of the scroll actually has room. `s0` was 0 on
	## one run and already at the bottom stop on the next, and a swipe into a
	## clamp measures nothing.
	var bar := scroll.get_v_scroll_bar()
	var room_up: float = bar.max_value - bar.page - float(s0)
	var dir := -1.0 if room_up > 200.0 else 1.0
	_log("  scroll %d of %.0f (page %.0f), swiping %s" %
		[s0, bar.max_value - bar.page, bar.page, "up" if dir < 0.0 else "down"])
	await _swipe(start, Vector2(0, dir * _vp.size.y * 0.20), 12, slider)
	var v1 := float(app.bridge.param_get(key))
	_log("  vertical:   %s  %s -> %s   scroll %d -> %d   drag_ended x%d   slider alive=%s value=%s" %
		[key, str(v0), str(v1), s0, scroll.scroll_vertical, _drag_ends,
		is_instance_valid(slider), str(slider.value) if is_instance_valid(slider) else "-"])
	_check(v1 == v0, "a vertical swipe starting ON the slider leaves `%s` byte-identical" % key)
	_check(is_instance_valid(slider) and slider.value == v0,
		"and leaves the slider itself on the same value")
	_check(scroll.scroll_vertical != s0, "and scrolls the sheet (%d -> %d)"
		% [s0, scroll.scroll_vertical])
	_check(_drag_ends == 0, "and emits no drag_ended, so nothing is written to the engine")
	_check(int(ws.get("_stale_from_stage")) == stale0,
		"and marks no stage stale (_stale_from_stage %d)" % stale0)

	# -- horizontal: must write (the positive control) -------------------------
	## Twice. The first drag is what takes the sheet from clean to stale, and
	## that transition legitimately rebuilds the column -- so the scroll-did-not-
	## move half of the claim is made on the second drag, over a sheet that is
	## already stale and therefore repaints in place.
	for pass_i in 2:
		slider = _first_slider(scroll)
		if slider == null:
			_check(false, "the slider is gone after pass %d" % pass_i)
			return
		if not slider.drag_ended.is_connected(_count_drag_end):
			slider.drag_ended.connect(_count_drag_end)
		scroll.ensure_control_visible(slider)
		await _frames(4)
		r = slider.get_global_rect()
		var v2 := float(app.bridge.param_get(key))
		var s2 := scroll.scroll_vertical
		_drag_ends = 0
		var frac := 0.15 if pass_i == 0 else 0.85
		var span := (r.size.x * 0.7) if pass_i == 0 else (-r.size.x * 0.7)
		var lo := Vector2(r.position.x + r.size.x * frac, r.position.y + r.size.y * 0.5)
		await _swipe(lo, Vector2(span, 0), 12)
		var v3 := float(app.bridge.param_get(key))
		_log("  horizontal %d: %s  %s -> %s   scroll %d -> %d   drag_ended x%d" %
			[pass_i, key, str(v2), str(v3), s2, scroll.scroll_vertical, _drag_ends])
		_check(v3 != v2, "pass %d: a horizontal drag on the same slider still writes `%s`"
			% [pass_i, key])
		_check(_drag_ends == 1, "pass %d: and emits exactly one drag_ended (%d)"
			% [pass_i, _drag_ends])
		if pass_i == 1:
			_check(scroll.scroll_vertical == s2, "and does not scroll the sheet")

func _count_drag_end(_c: bool) -> void:
	_drag_ends += 1

func _first_slider(root: Node) -> HSlider:
	for c in _visible_controls(root):
		if c is HSlider:
			return c
	return null

## The label `_pg_range_field()` draws to the left of the value readout.
func _row_label(slider: Control) -> String:
	var wrap := slider.get_parent().get_parent() as Control
	if wrap == null:
		return ""
	for c in wrap.get_children():
		if c is HBoxContainer:
			for g in (c as Control).get_children():
				if g is Label:
					return String((g as Label).text)
	return ""

## `param_info(k).label` back to `k`, so the probe names the parameter the
## engine names rather than inventing a mapping of its own.
func _key_for_label(label_text: String) -> String:
	for k in app.bridge.param_keys():
		if String(app.bridge.param_info(String(k)).get("label", "")) == label_text:
			return String(k)
	return ""
