extends Node
## Width-floor probe -- `DccShell.phone_fit()`'s tap floor, measured on the
## axis a guard used to skip.
##
## `phone_fit()` **used to** raise `custom_minimum_size.y` for every tappable
## control unconditionally and `.x` only `if min_size.x > 0.0`, so a control
## that declared no width was never floored horizontally however many times the
## walk ran over it. That condition was removed 2026-09-06; both axes are now
## unconditional, and `phone_fit()`'s own comment carries the measurement. This
## probe is what measured the hole and is what keeps it shut: restore the
## condition and four of its assertions go red -- 174 controls with no floor
## applied and 149 narrow-but-not-short, in both the phone-tree census and the
## windows one. Verified by doing exactly that, 2026-09-06.
##
## **It does not use `_phonechrome_probe.gd`'s walk, and that is the point.**
## That walk counts only `MOUSE_FILTER_STOP` `BaseButton`s, and `phone_fit()`
## flips every plain `BaseButton` it visits to `MOUSE_FILTER_PASS` -- so a
## control leaves that walk's view at the moment it is fitted, which is the
## moment this floor is supposed to apply to it. The seam is recorded in
## `_phonechrome_probe.gd`'s own filter comment (26 buttons under STOP, 323
## under STOP-or-PASS). This walk filters on **nothing**: every `Control` under
## `_phone_root` is collected and classified afterwards, so a control that
## `phone_fit()` has touched is still in view.
##
## Run (headless is correct here -- every assertion is a laid-out size or a
## `get_combined_minimum_size()`, no rasterisation and no timing):
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . \
##     _widthfloor_probe.tscn -- --force-touch
##
## `--force-touch` is required for the same reason `_phonechrome_probe.gd`
## documents: `_phone` can never be true in this dev environment without it.

var _fail := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ok(name: String, got, want) -> void:
	var good: bool = str(got) == str(want)
	if not good:
		_fail += 1
	print("  ", "ok  " if good else "FAIL", " ", name, "   got=", got, " want=", want)

## Same exclusion `_phonechrome_probe.gd` makes and for the same reason:
## `PhoneMenuModel` is a permanently-`visible = false` clone of the DESKTOP
## menu bar, kept only so `PhoneMenu` can read its structure. It is never
## tappable on any phone composition.
func _collect(node: Node, out: Array) -> void:
	if node.name == "PhoneMenuModel":
		return
	if node is Control:
		out.append(node)
	for c in node.get_children():
		_collect(c, out)

func _describe(ctl: Control) -> String:
	var owner_script := "-"
	var n: Node = ctl.get_parent()
	while n != null:
		var s: Script = n.get_script() as Script
		if s != null:
			owner_script = s.resource_path.get_file()
			break
		n = n.get_parent()
	return "%s text=%s cms=%s size=%s fitted=%s built-by=%s" % [
		ctl.get_class(), str(ctl.get("text")), ctl.custom_minimum_size,
		ctl.size, ctl.has_meta("_phone_fitted"), owner_script]

## The exact set `phone_fit()`'s floor branch tests for, so the census below
## describes the real branch rather than a guess at what it covers.
func _is_tappable(ctl: Control) -> bool:
	return ctl is BaseButton or ctl is LineEdit or ctl is Range or ctl is TextEdit

## The three `phone_fit()` itself exempts from the scroll-propagation pass, and
## `_phonechrome_probe.gd` from its floor walk: each pops a `Window`-class
## popup on press rather than acting as a plain tap target.
func _is_popup_opener(ctl: Control) -> bool:
	return ctl is OptionButton or ctl is MenuButton or ctl is ColorPickerButton

## **A combined minimum wider than the screen is only a defect where nothing
## scrolls.** `phone_fit()`'s own `wide` argument exists for the one subtree
## that is deliberately wider than 412 dp -- the phone tool sheet, wrapped in a
## `SCROLL_MODE_AUTO` `ScrollContainer` by `_build_phone_tool_sheet()` -- and a
## check that cannot tell the two apart would report the design as a fault.
## Returns the nearest ancestor `ScrollContainer` that actually scrolls
## horizontally, or `null`.
func _h_scroll_ancestor(ctl: Control) -> ScrollContainer:
	var n: Node = ctl
	while n != null:
		if n is ScrollContainer:
			var sc := n as ScrollContainer
			if sc.horizontal_scroll_mode != ScrollContainer.SCROLL_MODE_DISABLED:
				return sc
		n = n.get_parent()
	return null

## What the control's own text wants, in physical px, or -1 where the question
## does not apply. **The `clip_text` trap is why this is measured off the font
## rather than off `get_minimum_size()`:** `phone_fit()` sets `clip_text` on
## every expanding `Label` and `Button`, and `clip_text` collapses a control's
## reported minimum width to 1 (`MISTAKES.md`'s own row for it). So a clipped
## label reports no width requirement at all, and a check built on
## `get_minimum_size()` would report "nothing stopped fitting" for a label that
## had been reduced to an ellipsis. Asking the font directly is the measure
## that survives the clip.
func _text_width(c: Control) -> float:
	if not (c is Label or c is Button):
		return -1.0
	var txt := str(c.get("text"))
	if txt.is_empty():
		return -1.0
	var f: Font = c.get_theme_font("font")
	if f == null:
		return -1.0
	return f.get_string_size(txt, HORIZONTAL_ALIGNMENT_LEFT, -1,
		c.get_theme_font_size("font_size")).x

## **Two coordinate spaces reach this walk, and one number cannot describe
## both.** `phone_fit()` is called with `unit = _phone_scale` for anything in a
## dock -- those controls lay out in physical px, and their floor is
## `round(44 * _phone_scale)` = 115 -- and with `unit = 1.0` for a `Window` that
## has already set `content_scale_factor` (its own header says so, and ~22 of
## its call sites are that case). A control in one of those windows lays out in
## **dp**: its floor is a bare 44, and its `size.x` is a dp figure that the
## compositor multiplies on the way to the screen. Comparing that 44 against
## the dock's 115 says a correctly-floored dialog button is 62% undersized.
## Multiplying by the owning window's own scale factor puts every control in
## physical px, so one floor is the right bar for all of them.
func _phys_scale(c: Control) -> float:
	var n: Node = c
	while n != null:
		if n is Window:
			return maxf(0.0001, (n as Window).content_scale_factor)
		n = n.get_parent()
	return 1.0

## **A window that has never been presented has no density yet, so nothing in
## it can be measured against a physical floor.** `DccWidgets.phone_present()`
## is what sets `content_scale_factor`; before that a phone-fitted dialog sits
## at scale 1.0 carrying 44 dp minimums, and multiplying those by 1.0 reports
## every one of them as 44 physical px against a 115 px bar. Measured: 82 such
## controls in this run's tree, all of them in `AcceptDialog`s that were built
## and fitted at boot and never opened -- a number that says nothing about the
## floor and everything about which windows this probe happened to open.
## Excluded rather than counted, with the excluded population printed.
func _density_known(c: Control) -> bool:
	var n: Node = c
	while n != null:
		if n is Window:
			return (n as Window).visible or n == get_tree().root
		n = n.get_parent()
	return true

## Every `Window` under `app`, with the combined minimum width of its content
## root -- the number that decides whether the window can fit the screen at
## all, since a `Window` is clamped up to its content's minimum and there is no
## scrollbar above it to reveal what spills.
func _window_report(node: Node, screen_w: float) -> void:
	if node is Window and node != node.get_tree().root:
		var w := node as Window
		var content := 0.0
		for c in w.get_children():
			if c is Control:
				content = maxf(content, (c as Control).get_combined_minimum_size().x)
		## In physical px, like everything else here: a content-scaled window
		## measures its own children in dp, and it is the scaled figure that
		## has to fit the screen. See `_phys_scale()`.
		content *= maxf(0.0001, w.content_scale_factor)
		## **How much of this window the assertions can actually see**, printed
		## rather than assumed. A window contributes nothing to them unless its
		## controls are phone-fitted AND laid out AND it is visible. Measured
		## on this probe's own three: the data manager gives 142 controls, all
		## fitted, 126 laid out; the credits dialog 8 and 8; the faction roster
		## **26 controls and 0 fitted** -- its two `phone_fit()` calls (`grep -n
		## phone_fit shell/faction_roster_window.gd`: `_build_detail`'s
		## `phone_fit(self, 1.0)` and `phone_fit(_list_body, 1.0)`) did not
		## reach the master-list state this probe opens it in, and the cause is
		## not investigated here. Of all that, **4** controls are tappable,
		## non-popup and laid out -- so the windows census adds 4 to the phone
		## tree's 492, and saying it "covers dialogs" would be a claim these
		## numbers do not make. What it does establish is the window-width
		## half: no window's content minimum moved past the screen.
		var kids: Array = []
		_collect(w, kids)
		var fit := 0
		var laid := 0
		for k in kids:
			if (k as Control).has_meta("_phone_fitted"):
				fit += 1
				if (k as Control).size.x > 0.5 and (k as Control).size.y > 0.5:
					laid += 1
		print("  window ", w.name, " visible=", w.visible, " size=", w.size,
			" scale=", w.content_scale_factor,
			" content min.x=", content, " physical px vs screen ", screen_w,
			"  controls=", kids.size(), " fitted=", fit, " fitted+laid=", laid,
			("  <-- OVER" if content > screen_w + 0.5 else ""))
	for c in node.get_children():
		_window_report(c, screen_w)

## One census, run twice (before and after a fix) against the same tree.
## Everything it prints is a count of laid-out controls -- a zero axis is
## UNMEASURED, not too small, the distinction `_phonechrome_probe.gd` had to
## learn, so those are counted apart and never reported as violations.
func _census(root: Node, floor_px: float, screen_w: float, tag: String) -> Dictionary:
	var all: Array = []
	_collect(root, all)
	var under := 0
	var unlaid := 0
	var tappable := 0
	var zero_cms := 0          ## the set the removed `min_size.x > 0.0` guard skipped
	var zero_cms_under := 0    ## ...of which, drawn below the floor
	var by_class := {}
	var tappable_by_class := {}
	var zero_by_class := {}
	var under_h := 0
	var fitted_n := 0
	var fit_under_unfloored := 0
	var fit_under_width_only := 0
	var fit_width_only_samples: Array = []
	var no_density := 0
	var dump: Array = []
	var worst_min_w := 0.0
	var worst_min_node := ""
	var overflow_scrolled := 0
	var overflow_unscrolled := 0
	var samples: Dictionary = {}
	for ctl in all:
		var c := ctl as Control
		## Physical px for every control, whichever space it lays out in --
		## see `_phys_scale()` for why one bar needs one space.
		var ps := _phys_scale(c)
		if not _density_known(c):
			no_density += 1
		var w_px := c.size.x * ps
		var h_px := c.size.y * ps
		var mx_px := c.custom_minimum_size.x * ps
		## **Every control, not only the tappable ones.** Flooring a minimum can
		## only grow the control it is set on -- but it takes that width out of a
		## shared row, so the regression to look for is a *sibling* getting
		## narrower. A census that only records the controls the fix touches
		## cannot see that, so the dump is the whole tree and the comparison is
		## done outside this process.
		dump.append({"p": str(c.get_path()), "c": c.get_class(),
			"t": str(c.get("text")), "w": c.size.x, "h": c.size.y,
			"mx": c.custom_minimum_size.x, "my": c.custom_minimum_size.y,
			"tw": _text_width(c), "ps": ps})
		var cmin: float = c.get_combined_minimum_size().x * ps
		if cmin > screen_w + 0.5:
			if _h_scroll_ancestor(c) != null:
				overflow_scrolled += 1
			else:
				overflow_unscrolled += 1
				print("      OVERFLOW (nothing scrolls it): cmin.x=", cmin, "  ", _describe(c))
		if cmin > worst_min_w:
			worst_min_w = cmin
			worst_min_node = _describe(c)
		if not _is_tappable(c) or _is_popup_opener(c):
			continue
		tappable += 1
		var tk: String = c.get_class()
		tappable_by_class[tk] = int(tappable_by_class.get(tk, 0)) + 1
		if c.custom_minimum_size.x <= 0.0:
			zero_cms += 1
			## **The classes a de-guarded floor would newly reach**, which is
			## the question "is it safe to remove the condition" restated as a
			## measurement. A `Range` here is the one `tablet_fit()`'s own
			## header refuses to floor.
			zero_by_class[tk] = int(zero_by_class.get(tk, 0)) + 1
		if w_px <= 0.5 or h_px <= 0.5:
			unlaid += 1
			continue
		## **The control measure for the width count.** The height floor has
		## been unconditional since this walk was written, so any control still
		## drawn under it is one whose parent ignores `custom_minimum_size`
		## altogether -- not something either half of this line can reach. Held
		## beside the width count so a width survivor can be attributed rather
		## than guessed at.
		if h_px < floor_px - 0.5:
			under_h += 1
		## **The assertion set: only controls `phone_fit()` has actually
		## walked.** A window that has never been presented has never been
		## fitted -- `asset_library_window.gd` and friends call `phone_fit()`
		## from their own `_popup_full()` -- so a narrow control inside one is
		## out of this line's reach and asserting on it would be asserting on
		## whether this probe happened to open that window. Measured: 14 of the
		## windows census's 689 tappable controls are unfitted, 3 of them under
		## the floor.
		if c.has_meta("_phone_fitted") and _density_known(c):
			fitted_n += 1
			if w_px < floor_px - 0.5:
				## The guard's own hole: the floor was never applied, so the
				## control's own declared minimum is still below it.
				if mx_px < floor_px - 0.5:
					fit_under_unfloored += 1
				## Narrow but NOT short. The height floor has been
				## unconditional since this walk was written, so a control
				## under both floors is one whose parent ignores
				## `custom_minimum_size` outright -- a different defect on both
				## axes, not this line's. One under the width floor ALONE is
				## this line's and nothing else's.
				if h_px >= floor_px - 0.5:
					fit_under_width_only += 1
					if fit_width_only_samples.size() < 8:
						fit_width_only_samples.append(_describe(c))
		if w_px < floor_px - 0.5:
			under += 1
			if c.custom_minimum_size.x <= 0.0:
				zero_cms_under += 1
			var k: String = c.get_class()
			by_class[k] = int(by_class.get(k, 0)) + 1
			if not samples.has(k):
				samples[k] = []
			if (samples[k] as Array).size() < 3:
				(samples[k] as Array).append(_describe(c))
	## `--dump=<name>` writes the whole-tree census to `user://` so a run before
	## a change and a run after it can be compared per control, outside this
	## process. Omitted entirely when the flag is absent -- an empty file would
	## be a plausible-looking "no controls".
	for a in OS.get_cmdline_user_args():
		if a.begins_with("--dump="):
			var path := "user://widthfloor_%s_%s.json" % [a.substr(7), tag]
			var f := FileAccess.open(path, FileAccess.WRITE)
			f.store_string(JSON.stringify(dump))
			f.close()
			print("  [", tag, "] dumped ", dump.size(), " controls -> ",
				ProjectSettings.globalize_path(path))
	print("  [", tag, "] controls=", all.size(), " tappable(non-popup)=", tappable,
		" laid-out-under-floor=", under, " unlaid=", unlaid,
		"  (same set under the HEIGHT floor, which has always been "
		+ "unconditional: ", under_h, ")")
	print("  [", tag, "] of the tappable set, custom_minimum_size.x == 0: ", zero_cms,
		"  (of the under-floor ones: ", zero_cms_under, ")")
	print("  [", tag, "] under-floor by class: ", by_class)
	print("  [", tag, "] whole tappable set by class: ", tappable_by_class)
	print("  [", tag, "] ...of which cms.x == 0 (what a de-guarded floor reaches): ",
		zero_by_class)
	print("  [", tag, "] widest combined minimum: ", worst_min_w, " px vs screen ",
		screen_w, "  -> over-screen: ", overflow_scrolled, " inside an h-scroller, ",
		overflow_unscrolled, " with nothing to scroll them")
	print("  [", tag, "] controls in a window with no density yet (excluded): ", no_density)
	print("  [", tag, "] phone_fit()-walked, laid out, density known: ", fitted_n,
		"  -> under the width floor with no floor applied: ", fit_under_unfloored,
		",  under the width floor but NOT the height floor: ", fit_under_width_only)
	for s2 in fit_width_only_samples:
		print("      narrow-but-not-short: ", s2)
	print("  [", tag, "] widest is: ", worst_min_node)
	for k in samples:
		for s in samples[k]:
			print("      under-floor: ", s)
	return {"under": under, "tappable": tappable, "unlaid": unlaid,
		"zero_cms": zero_cms, "overflow": overflow_unscrolled,
		"fitted": fitted_n, "unfloored": fit_under_unfloored,
		"width_only": fit_under_width_only,
		"overflow_scrolled": overflow_scrolled,
		"worst_min_w": worst_min_w, "by_class": by_class}

func _ready() -> void:
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
	print("[BOOT] shell up, is_phone=", app.is_phone(), " phone_scale=", app.phone_scale())
	if not bool(app.is_phone()):
		print("[FATAL] booted into the desktop/tablet composition, not phone")
		get_tree().quit(1); return

	## The same warm-up `_phonechrome_probe.gd` performs, and for the reason it
	## records: a subtree that has never been visible has never had a real
	## container-sort pass, so its sizes read back as 0 and mean nothing.
	app.call("_set_overflow_open", true); await _frames(2)
	app.call("_set_sheet_open", "left", true); await _frames(2)
	app.call("_set_sheet_open", "right", true); await _frames(2)
	app.call("_close_all_phone_overlays")
	await _frames(2)
	var dom0: String = str(app.call("active_domain"))
	var mode0: String = str(app.call("active_mode", dom0))
	var tool0: String = str(app.get("armed_tool"))
	app.call("open_journey_planner")
	await _frames(20)

	var floor_px: float = float(app.call("_pscale", 44))
	var screen_w := 1080.0
	print("\n=== census: every Control under _phone_root, no mouse_filter filter ===")
	print("  info floor = _pscale(44) = ", floor_px, " physical px at phone_scale ",
		app.phone_scale(), " (density: 1080 x 2340 portrait handset, ",
		1080.0 / app.phone_scale(), " dp wide)")
	var c := _census(app.get("_phone_root"), floor_px, screen_w, "now")

	## **`_phone_root` is not `phone_fit()`'s whole surface, and the part it
	## leaves out is the part with the known failure mode.** Roughly 22 of the
	## call sites pass a `Window` (`app.gd`'s credits dialog, the roster, the
	## data manager, the asset library, `browse_dialog.gd`), and a `Window`
	## **cannot be narrower than its content's combined minimum** -- which is
	## exactly how the faction roster once grew past a 393 dp screen and pushed
	## its Add/Remove row 1 750 px below the bottom (`phone_fit()`'s own
	## `clip_text` comment carries that post-mortem). A width floor adds to
	## that minimum on every button in the window, so this is where a floor
	## fix could pay for 174 narrow buttons with an unreachable dialog.
	## Censused separately, rooted at `app` so the walk descends into the
	## embedded sub-windows (`gui_embed_subwindows = true` above), and with
	## every `Window`'s own content minimum reported against the screen.
	print("\n=== windows: the other half of phone_fit()'s surface ===")
	app.call("open_faction_roster"); await _frames(10)
	app.call("open_data_manager"); await _frames(10)
	app.call("open_credits"); await _frames(10)
	_window_report(app, screen_w)
	var cw := _census(app, floor_px, screen_w, "windows")

	print("\n=== the guard, stated as a measurement ===")
	## Not an assertion on a constant: this reads the live tree and says how
	## many controls the removed `min_size.x > 0.0` branch could not reach. It is the
	## row's own number.
	print("  info tappable controls the width floor cannot reach: ", c["zero_cms"],
		" of ", c["tappable"])

	print("\n=== assertions ===")
	## **Not "nothing is narrow" -- "nothing is narrow because of THIS line".**
	## The blunt form (`under == 0`) cannot go green and never could: 25
	## controls in the phone tree are drawn under BOTH floors, at 36 x 28
	## against a declared minimum of 115 x 115, because their parent ignores
	## `custom_minimum_size` outright. The unconditional height floor has never
	## reached them either, so they are a separate defect and an assertion that
	## folds them in would be permanently red for a reason the fix cannot
	## touch. The two below split that apart.
	##
	## First: a control `phone_fit()` walked, drawn under the floor, whose own
	## declared minimum is ALSO under the floor -- i.e. the floor was never
	## applied to it. That is the guard's hole and nothing else.
	## Measured with the guard in place: 174 (phone tree) / 235 (with windows).
	_ok("every fitted control has the width floor applied to it",
		c["unfloored"], 0)
	## Second, and it is the one that survives someone "fixing" the first by
	## writing a minimum somewhere else: a control under the WIDTH floor but
	## clearing the HEIGHT floor. The height floor is unconditional, so a
	## control that clears it is one the walk reached and sized successfully --
	## and if it is still narrow, the width half of that same line is the only
	## thing that can be responsible. Measured with the guard in place: 149.
	_ok("nothing is narrow while being tall enough", c["width_only"], 0)
	## The other half, and the reason the fix must be narrow: flooring a width
	## adds to every ancestor's combined minimum, and a combined minimum wider
	## than the screen is the overflow class `MISTAKES.md` records three
	## instances of. If a fix trades N narrow buttons for one overflowing row
	## it has not helped.
	##
	## Counted only where **nothing scrolls the control horizontally** -- the
	## phone tool sheet is deliberately wider than the screen and rides a
	## `SCROLL_MODE_AUTO` `ScrollContainer` (`phone_fit()`'s own `wide`
	## argument exists for it), so an undiscriminating count would call the
	## design a fault. Measured at HEAD before the fix: 0 unscrolled,
	## 2 inside an h-scroller.
	_ok("nothing overflows the screen with no scroller to reveal it",
		c["overflow"], 0)

	## **The same two, over the windows census.** `phone_fit()`'s other ~22
	## call sites are `Window`s, and a `Window` is clamped up to its content's
	## combined minimum with nothing above it to scroll -- so a width floor is
	## capable of widening a dialog past the screen, which is the failure the
	## roster once shipped. Asserted separately from the phone tree because the
	## surface is different and so is the consequence of getting it wrong.
	_ok("windows: every fitted control has the width floor applied to it",
		cw["unfloored"], 0)
	_ok("windows: nothing is narrow while being tall enough",
		cw["width_only"], 0)
	_ok("windows: nothing overflows the screen with no scroller to reveal it",
		cw["overflow"], 0)

	## Restore, and assert the restore rather than assuming it -- same
	## discipline as `_phonechrome_probe.gd`'s planner warm-up.
	app.call("select_domain_mode", dom0, mode0)
	app.call("arm_tool", tool0)
	await _frames(10)
	_ok("the planner warm-up left the domain as it found it",
		app.call("active_domain"), dom0)
	_ok("the planner warm-up left the armed tool as it found it",
		app.get("armed_tool"), tool0)

	print("\n_widthfloor_probe: ", "PASS" if _fail == 0 else str(_fail) + " FAILURE(S)")
	get_tree().quit(1 if _fail > 0 else 0)
