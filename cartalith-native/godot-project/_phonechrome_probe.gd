extends Node
## Phone-chrome probe -- verifies the three items this pass added to
## `shell/dcc_shell.gd` (the app bar's `⌕` search, the floating `↶` undo chip,
## the two coach-mark toasts) plus the `_ptap()` tap-floor fix a coordinator
## review found separately. Structural assertions only -- `--headless` uses
## the dummy rasteriser, so `texture_2d_get()` returns null and pixel checks
## are not meaningful here.
##
## Modelled on `_cmdindex_probe.gd`: a `SubViewport` sized to the target form
## factor rather than `--resolution`, since `DccShell._compute_layout_mode()`
## reads `get_viewport_rect().size`, which for a node inside a `SubViewport`
## is that viewport's own size, not the root window's. Portrait phone,
## 1080x2340 -- short/long = 0.46, comfortably under `_PHONE_ASPECT_MAX`
## (0.6), matching a real handset rather than sitting near the threshold.
##
## Run:
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . \
##     _phonechrome_probe.tscn -- --force-touch
##
## `--force-touch` is required: `_touch` (and therefore `_phone`) is decided
## in `DccShell._ready()` off `DisplayServer.is_touchscreen_available() and
## OS.has_feature("mobile")`, neither ever true in this headless dev
## environment, OR this cmdline flag -- the same override `_shot_phone.gd`
## already documents and relies on.

var _fail := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ok(name: String, got, want) -> void:
	var good: bool = str(got) == str(want)
	if not good:
		_fail += 1
	print("  ", "ok  " if good else "FAIL", " ", name, "   got=", got, " want=", want)

func _find_button_by_tooltip(root: Node, tip: String) -> Button:
	if root is Button and (root as Button).tooltip_text == tip:
		return root
	for c in root.get_children():
		var found := _find_button_by_tooltip(c, tip)
		if found != null:
			return found
	return null

## `PhoneMenuModel` excluded by name: `dcc_shell.gd`'s own comment at its
## construction site calls it out as a permanently-`visible = false` clone of
## the DESKTOP menu bar and status bar, kept only so `PhoneMenu` (a separate
## file) can read its structure -- never shown, never tappable, on any phone
## composition. A tap-floor check on it would be testing a control that
## cannot physically be tapped.
func _collect_controls(node: Node, out: Array) -> void:
	if node.name == "PhoneMenuModel":
		return
	if node is Control:
		out.append(node)
	for c in node.get_children():
		_collect_controls(c, out)

## An anonymous `@Button@4913` node path names nothing a reader can act on --
## the shell builds every phone control in code, so there are no scene names.
## Printed beside any violation so the report says WHAT the control is (its
## text, its tooltip, its class) and WHO built it (the nearest ancestor
## carrying a script), rather than a path that has to be re-walked by hand.
func _describe(ctl: Control) -> String:
	var chain: PackedStringArray = []
	var n: Node = ctl.get_parent()
	while n != null:
		var s: Script = n.get_script() as Script
		chain.append("%s(%s)%s" % [n.name, n.get_class(),
			"" if s == null else "<" + s.resource_path + ">"])
		n = n.get_parent()
	return "class=%s text=%s tooltip=%s cms=%s fitted=%s | parents: %s" % [
		ctl.get_class(), str(ctl.get("text")), ctl.tooltip_text,
		ctl.custom_minimum_size, ctl.has_meta("_phone_fitted"), " < ".join(chain)]

## The tap-floor walk, lifted out of section 4 so it can be run against more
## than one composition. Returns the violation count; prints every violation
## with `_describe()` beside it and every unlaid control separately.
func _tap_walk(app: Node, floor_px: float) -> int:
	var phone_root: Node = app.get("_phone_root")
	var violations := 0
	var unlaid := 0
	var checked := 0
	if phone_root != null:
		var all_controls: Array = []
		_collect_controls(phone_root, all_controls)
		for ctl in all_controls:
			if not (ctl is BaseButton):
				continue
			var bb := ctl as BaseButton
			## **This filter is narrower than the section heading sounds, and the
			## narrowing is not cosmetic.** `DccShell.phone_fit()` turns every plain
			## `BaseButton` it visits into `MOUSE_FILTER_PASS` (its own PH-05 branch:
			## a flick that starts on a button has to reach the `ScrollContainer`
			## above it), so `STOP` here selects, in practice, the buttons
			## `phone_fit()` has **not** touched -- the phone chrome `dcc_shell.gd`
			## sizes through `_ptap()`, which is what this section was written to
			## guard. Everything inside a fitted dock, sheet or window is excluded.
			## Measured by widening this one line to STOP-or-PASS -- and **the figure
			## has moved, so it is quoted here with both readings rather than as a
			## standing fact.** An earlier pass on 2026-09-06 wrote *26 buttons to 323,
			## 155 violations, 36 x 115 `CheckBox`es and 84 x 126 `Button`s*.
			## Re-measured the same way later the same day, after the shell had moved:
			## **336 buttons checked, 19 violations** against the same 115 px floor.
			## Left narrow either way -- 19 is still a shell-wide backlog across many
			## files, not something this probe can turn red on its own. **Do not read a
			## pass here as "every phone button clears 44 dp"**, and do not read it as
			## covering a *fitted* control at all: see `_fit_walk()` above for why it
			## structurally cannot.
			if bb.mouse_filter != Control.MOUSE_FILTER_STOP:
				continue
			## Same three exclusions as the walk at `dcc_shell.gd`'s own
			## `phone_fit()`-adjacent scroll-propagation pass (~line 1098):
			## each of the three pops a `Window`-class popup on press rather
			## than acting as a plain tap target, so "cleared 44 dp" is not
			## the property that matters for them the way it is for a
			## `Button`.
			if bb is OptionButton or bb is MenuButton or bb is ColorPickerButton:
				continue
			checked += 1
			var sz: Vector2 = bb.size
			## **A zero axis is UNMEASURED, not too small**, and reporting it as a
			## tap-floor violation is a false red. It means the control has never
			## had a real layout pass -- the state the warm-up above exists to
			## remove.
			##
			## Three such buttons were reported here from 2026-09-06 at
			## `(0.0, 115.0)`, a correct height against a width that had never
			## been resolved, and the cause recorded beside them -- that deleting
			## the `_set_panel_picker_open` warm-up with the `▤` glyph (owner
			## ruling 20) had stopped warming them -- was **wrong**. Measured
			## with `_sheetback_probe.gd --sub 1080x2340`: they are the `⋮`
			## overflow popover's three rows (`Save project` / `Theme` /
			## `Close world`, built by `_build_phone_overflow()` under
			## `_phone_overflow_pop`), and **nothing has ever warmed them**. The
			## warm-up above called `_set_overflow_open()`, which despite the
			## name opens `PhoneMenu`'s L2 root and never touches the popover;
			## the function that shows it is `_set_phone_overflow_open()`. The
			## picker deletion narrowed the count from 5 to 3, which is why it
			## looked like the cause. Adding that one call resolves all three:
			## `(0.0, 115.0)` -> `(601.0, 115.0)` at `_phone_scale` 2.6214,
			## clearing the 115 px floor on both axes.
			if sz.x <= 0.5 or sz.y <= 0.5:
				unlaid += 1
				print("  UNLAID   ", bb.get_path(), "  size=", sz,
					"  (never laid out -- not a floor violation)")
				print("            ", _describe(bb))
				continue
			if sz.x < floor_px - 0.5 or sz.y < floor_px - 0.5:
				violations += 1
				print("  VIOLATION ", bb.get_path(), "  size=", sz, "  floor=", floor_px)
				print("            ", _describe(bb))
	else:
		print("  SKIP -- no _phone_root")
	print("  info BaseButtons checked (STOP filter, non-popup): ", checked)
	return violations

## **The complement of `_tap_walk()`, and the reason it exists is that
## `_tap_walk()` goes blind on success.**
##
## What the STOP filter protects, traced rather than assumed: it is a *scope*
## device in this probe and a *scroll* device in the shell, and the two got
## coupled by accident. In `dcc_shell.gd::phone_fit()` the `PASS` assignment is
## PH-05's fix -- `_scrolldrag_probe.gd` found that every point down the left
## sheet that failed to flick was a `Button` or an `HSlider`, because a `STOP`
## control ends Godot's event walk before the `ScrollContainer` above it sees
## the drag; `PASS` still picks (so tooltips and hover survive) and forwards
## what it does not handle, and `ScrollContainer`/`BaseButton` cancel the
## pending press past the deadzone. In *this* probe the same constant was
## chosen to keep the walk on the un-fitted phone chrome `_ptap()` sizes,
## because widening it reports 155 violations of a shell-wide backlog this
## probe cannot fix. Both reasons are good; together they mean **the moment
## `phone_fit()` succeeds on a control, that control leaves `_tap_walk()`**.
##
## The defect, as disclosed: deleting `journey_planner_view.gd`'s width fix
## entirely still yields `no tap-floor violations got=0` from `_tap_walk()`,
## because the button is fitted either way and fitted means `PASS`. The whole
## of that fix's coverage was four hand-written `_ok` lines naming one button;
## delete those four and nothing turned red. **Measured here rather than
## restated:** `_tap_walk()` checks 31 buttons with the planner open and this
## walk finds the planner's centre panel holds exactly 1 fitted `BaseButton`
## and 0 never-fitted ones -- so the two walks partition it, and the fitted
## side had no walk at all. Mutating the floor to `* 1.5` turns this red at
## `violations=1` while `_tap_walk()` stays at its own unrelated count;
## mutating the meta key drops `checked` to 0 and turns the emptiness
## assertion red. Both run 2026-09-06, restored in a `finally`.
##
## So this walks the other half of the partition -- `BaseButton`s carrying
## `phone_fit()`'s own `_phone_fitted` meta -- and asserts the floor on them
## structurally, over whatever the subtree happens to contain, rather than by
## name. It cannot go blind the way the STOP walk does: a `phone_fit()` that
## stopped fitting drops `checked` to 0, which is asserted separately, and a
## fitted button under the floor is a violation whether or not anyone thought
## to name it.
##
## Scoped to a subtree by the caller for the same reason `_tap_walk()` keeps
## its narrow filter: run over the whole phone root this reports the same
## shell-wide backlog, which is a real finding and not this probe's row.
## Returns `[checked, violations]`.
func _fit_walk(root: Node, floor_px: float) -> Array:
	var all_controls: Array = []
	_collect_controls(root, all_controls)
	var checked := 0
	var violations := 0
	var hidden := 0
	var unfitted := 0
	for ctl in all_controls:
		var c := ctl as Control
		if not (c is BaseButton):
			continue
		if not c.has_meta("_phone_fitted"):
			unfitted += 1
			continue
		## The same three popup-openers `_tap_walk()` exempts, for the same
		## reason, and the same UNLAID rule: a zero axis is unmeasured, not
		## too small.
		if c is OptionButton or c is MenuButton or c is ColorPickerButton:
			continue
		## A hidden control's `size` is whatever it was last laid out at, which
		## for a subtree that has never been shown is not a measurement of
		## anything. Counted and reported rather than silently dropped, so the
		## `checked` figure below can be read as what it is.
		if not c.is_visible_in_tree():
			hidden += 1
			continue
		var sz: Vector2 = c.size
		if sz.x <= 0.5 or sz.y <= 0.5:
			print("  UNLAID   ", c.get_path(), "  size=", sz)
			continue
		checked += 1
		if sz.x < floor_px - 0.5 or sz.y < floor_px - 0.5:
			violations += 1
			print("  VIOLATION ", c.get_path(), "  size=", sz, "  floor=", floor_px)
			print("            ", _describe(c))
	print("  info phone-fitted BaseButtons: checked=", checked, " violations=",
		violations, "  (hidden=", hidden, " never-fitted=", unfitted, ")")
	return [checked, violations]

func _ready() -> void:
	## A clean slate for the coach-mark assertions below: this repo's own
	## `user://cartalith_settings.cfg` may already carry `coach_marks.*=true`
	## from a prior dev session or probe run on this machine, which would make
	## "shown once, then marked seen" untestable (they would already read
	## seen before `_maybe_show_coach_marks()` ever ran this boot). Cleared
	## via a fresh `ConfigFile` at the same path `DccSettings.CONFIG_PATH`
	## names -- the identical mechanism `dcc_shell.gd`'s own
	## `_coach_mark_seen()`/`_set_coach_mark_seen()` use, not a new file.
	## Done BEFORE the app boots, since `_maybe_show_coach_marks()` runs
	## during `_build_phone_shell()`, inside `_ready()`, before this script
	## gets another chance to touch anything.
	var cfg := ConfigFile.new()
	cfg.load(DccSettings.CONFIG_PATH)
	if cfg.has_section("coach_marks"):
		cfg.erase_section("coach_marks")
		cfg.save(DccSettings.CONFIG_PATH)

	await _frames(2)
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load"); get_tree().quit(1); return
	if not ("--force-touch" in OS.get_cmdline_user_args()):
		print("[FATAL] run with `-- --force-touch` -- _phone can never be true without it")
		get_tree().quit(1); return

	var vp := SubViewport.new()
	vp.size = Vector2i(1080, 2340)
	vp.gui_embed_subwindows = true
	vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(vp)
	var app: Node = load("res://shell/app.tscn").instantiate()
	vp.add_child(app)
	await _frames(50)
	print("[BOOT] shell up, is_phone=", app.is_phone())
	if not bool(app.is_phone()):
		print("[FATAL] booted into the desktop/tablet composition, not phone -- ",
			"the 1080x2340 SubViewport should have forced it")
		get_tree().quit(1); return

	print("\n=== 1: the app bar has a ⌕ cell, and only because the index exists ===")
	_ok("place_search.gd exists in this checkout", app.call("_has_place_search"), true)
	var bar: Control = app.get("_phone_app_bar")
	_ok("the app bar was built", bar != null, true)
	var search_btn: Button = null
	if bar != null:
		search_btn = _find_button_by_tooltip(bar, "Search")
	_ok("a Search-tooltipped button sits in the app bar", search_btn != null, true)

	print("\n=== 2: opening it produces a focused text field ===")
	if search_btn != null:
		search_btn.pressed.emit()
		await _frames(3)
		var overlay: Control = app.get("_phone_search_overlay")
		_ok("the search overlay is visible after the tap", overlay != null and overlay.visible, true)
		var field: LineEdit = app.get("_phone_search_field")
		_ok("the search field exists", field != null, true)
		if field != null:
			_ok("the search field has focus", field.has_focus(), true)
		app.call("_set_search_open", false)
		await _frames(2)
	else:
		print("  SKIP -- no button to press")

	print("\n=== 3: the undo chip is HIDDEN with nothing to undo ===")
	var chip: Button = app.get("_phone_undo_chip")
	_ok("the undo chip exists", chip != null, true)
	_ok("can_undo() is false on a fresh boot with no edits", bool(app.bridge.can_undo()), false)
	if chip != null:
		## `_wire_phone_undo_chip()`'s own bridge lookup is deferred one
		## frame past `_ready()`'s end -- already well covered by the 50
		## frames above, but an explicit re-check keeps this assertion
		## honest about what it depends on rather than assuming timing.
		_ok("the chip is not visible", chip.visible, false)

	print("\n=== 4: every phone BaseButton clears the tap floor ===")
	## Cycle every overlay open-then-closed first. A `Control` subtree that
	## has never once been visible has never had a real container-sort pass
	## either -- measured directly: before this cycling was added, the panel
	## picker's own two rows read back `size=(0.0, 136.0)`, not merely
	## un-floored but structurally unsized, because nothing had ever asked
	## a `VBoxContainer` inside a `visible=false` subtree to lay out its
	## children's cross-axis width. Opening and closing each overlay once
	## gives every phone Control the one real layout pass its size needs to
	## mean anything; `_close_all_phone_overlays()` leaves the tree exactly
	## as it started.
	## `_set_panel_picker_open` was deleted 2026-09-06 with the `▤` glyph
	## (owner ruling 20: the app bar is `[world pill] · ⌕ · ⋮`). Calling it made
	## this probe error at load and then **hang** rather than fail, so sections
	## 4+ — the tap-floor walk it exists for — never ran. `_hidpi_probe.gd` and
	## `_jp16_probe.gd` carry the post-mortem for this same failure mode.
	app.call("_set_overflow_open", true); await _frames(2)
	## **A second, differently named function, and the difference is the whole
	## point.** `_set_overflow_open()` above opens `PhoneMenu`'s L2 root -- it
	## keeps that name only so `_shot_phone.gd --overflow` still works, and its
	## own doc comment says so. The `⋮` popover is `_phone_overflow_pop` and
	## `_set_phone_overflow_open()` is what shows it. Without this line its
	## three rows are the only phone subtree no warm-up reaches, and they were
	## reported as UNLAID for it. `_close_all_phone_overlays()` below hides the
	## popover again, so the restore is the same one the other three get.
	app.call("_set_phone_overflow_open", true); await _frames(2)
	app.call("_set_sheet_open", "left", true); await _frames(2)
	app.call("_set_sheet_open", "right", true); await _frames(2)
	app.call("_close_all_phone_overlays")
	await _frames(2)

	## The fix under test: `_ptap()` used to compare an UNSCALED 44 against
	## an already-scaled value, so it silently never raised anything below
	## 44 dp once `_phone_scale` moved off 1.0. `_pscale(44)` is `_ptap()`'s
	## own new floor expression re-derived here as the expectation, not a
	## second guess at what it should be.
	var floor_px: float = float(app.call("_pscale", 44))
	print("  info _phone_scale short-side reference: PHONE_REF_SHORT vs 1080 ",
		"-> _pscale(44) = ", floor_px, " physical px")
	## **The Journey planner is a fourth surface the overlay cycling above
	## cannot reach, and it is where this batch's one real violation lived.**
	## It is not an overlay, it is an armed tool: `journey_planner_view.gd`
	## runs its own `phone_fit(_center_panel, phone_scale())` from `_show()`,
	## so until the planner has been opened once its whole centre panel still
	## carries desktop-authored sizes and `visible = false`. Measured today,
	## on the route-map layer button: `(59, 51)` with `_phone_fitted` absent
	## before the first open, `(115, 115)` with it set immediately after.
	## Reporting the first of those as a tap-floor violation is the same false
	## red as the UNLAID entries below -- a control nothing has fitted yet, not
	## a control a finger cannot hit. Opened and closed here for exactly the
	## reason the three overlays are, and the restore is asserted, not assumed.
	var dom0: String = str(app.call("active_domain"))
	var mode0: String = str(app.call("active_mode", dom0))
	var tool0: String = str(app.get("armed_tool"))
	app.call("open_journey_planner")
	await _frames(20)

	## Asserted HERE, with the planner open, and not left to the walk below.
	## `phone_fit()` turns every plain `BaseButton` it touches into
	## `MOUSE_FILTER_PASS`, and the walk counts only `STOP` -- so the moment
	## this control is fitted it drops out of that walk entirely (measured:
	## 26 buttons checked before the planner opens, 25 after, and the one
	## that leaves is this button). Without these three lines the fix in
	## `journey_planner_view.gd` would be guarded by nothing at all.
	##
	## Both axes, because they failed for different reasons. `phone_fit()`
	## raises `custom_minimum_size.y` for every `BaseButton` unconditionally,
	## so the height was already 115; it raises `.x` only `if min_size.x >
	## 0.0`, so an icon-only button that declared no width stayed 35 px --
	## 13 dp at this composition's `phone_scale` of 2.621 -- in the shipped
	## app, for as long as the planner has existed. The width is the half the
	## fix adds and the half a one-axis assertion would miss.
	var jp_btn: Button = app.get("journey_planner_view").get("_route_map_layer_btn")
	print("  info route-map layer button, planner open: size=", jp_btn.size,
		" cms=", jp_btn.custom_minimum_size, " fitted=", jp_btn.has_meta("_phone_fitted"))
	_ok("the planner's subtree is phone-fitted once it opens",
		jp_btn.has_meta("_phone_fitted"), true)
	_ok("route-map layer button clears the tap floor on WIDTH",
		jp_btn.size.x >= floor_px - 0.5, true)
	_ok("route-map layer button clears the tap floor on HEIGHT",
		jp_btn.size.y >= floor_px - 0.5, true)
	## The box grew; the mark inside it has to grow with it. `phone_fit()`
	## re-rasterises icons only for `TOOL_GLYPH_META` buttons, so flooring
	## alone would leave the desktop 15 px glyph adrift in a 115 px cell --
	## the "15 px stays 15 physical px, about a millimetre, sitting in a 121
	## px cell" fault `_phone_fit_tool_button()` was written for. `> 15`
	## rather than a pinned size: the assertion is that the desktop raster is
	## not what shipped, and it dies the moment that line is removed.
	_ok("its glyph was re-rasterised for the floored box",
		jp_btn.icon != null and jp_btn.icon.get_height() > 15, true)
	print("  info glyph raster height, planner open: ",
		0 if jp_btn.icon == null else jp_btn.icon.get_height(), " px")

	## **The structural half, and the answer to "delete those four lines and
	## nothing turns red".** The four `_ok`s above name one button; this walks
	## every phone-fitted `BaseButton` in the same subtree and applies the same
	## floor, so a second icon-only button added to this panel tomorrow is
	## covered by construction rather than by somebody remembering. See
	## `_fit_walk()`'s header for why `_tap_walk()` structurally cannot see any
	## of them.
	##
	## `checked > 0` is not decoration: it is the assertion that keeps this from
	## going blind the way the STOP walk does. A `phone_fit()` that stopped
	## stamping `_phone_fitted`, or a planner that stopped calling it, would
	## empty this walk -- and an empty walk reports zero violations.
	var fit_res: Array = _fit_walk(app.get("journey_planner_view").get("_center_panel"),
		floor_px)
	_ok("the fitted walk actually saw fitted buttons (it cannot pass empty)",
		int(fit_res[0]) > 0, true)
	_ok("every phone-fitted button in the planner clears the tap floor",
		fit_res[1], 0)

	app.call("select_domain_mode", dom0, mode0)
	app.call("arm_tool", tool0)
	await _frames(10)
	_ok("the planner warm-up left the domain as it found it",
		app.call("active_domain"), dom0)
	_ok("the planner warm-up left the armed tool as it found it",
		app.get("armed_tool"), tool0)

	var violations: int = _tap_walk(app, floor_px)
	_ok("no tap-floor violations", violations, 0)

	print("\n=== 5: exactly two coach marks, both marked seen after showing once ===")
	var ids: Array = app.call("_coach_mark_ids")
	_ok("exactly two coach marks are defined", ids.size(), 2)
	## Both were already triggered once, at boot, by `_maybe_show_coach_
	## marks()`'s own deferred call inside `_build_phone_shell()` -- this
	## probe does not call anything to start them. The first is marked seen
	## synchronously when shown; the second only fires after a real 3.6 s
	## `SceneTreeTimer` (headless does not accelerate wall-clock timers), so
	## this waits long enough for that chain to finish rather than for a
	## frame count that says nothing about elapsed real time.
	await get_tree().create_timer(5.0).timeout
	for id in ids:
		_ok("coach mark '%s' is marked seen" % id, app.call("_coach_mark_seen", id), true)

	print("\n_phonechrome_probe: ", "PASS" if _fail == 0 else str(_fail) + " FAILURE(S)")
	get_tree().quit(1 if _fail > 0 else 0)
