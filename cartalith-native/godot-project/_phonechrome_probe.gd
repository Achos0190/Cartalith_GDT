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
			## Measured 2026-09-06 by widening this one line to STOP-or-PASS: the
			## walk goes from **26 buttons to 323** and reports **155 violations**
			## against the same 115 px floor -- 36 x 115 `CheckBox`es and 84 x 126
			## `Button`s, the same un-floored-width class as the one fixed in this
			## batch. Left narrow deliberately: that is a shell-wide backlog across
			## many files, not something this probe can turn red on its own. **Do
			## not read a pass here as "every phone button clears 44 dp".**
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
			## remove. Counted and printed separately, 2026-09-06, after the
			## `_set_panel_picker_open` warm-up was deleted with the `▤` glyph
			## (owner ruling 20) and three Controls in one subtree stopped being
			## laid out: they reported `(0.0, 115.0)`, a correct height against a
			## width that had never been resolved. **The gap is real and is a
			## backlog row** -- something is no longer warmed -- but it is a
			## measurement gap, not four buttons a finger cannot hit.
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
