extends Node
## **The timeline row's touch floor, DRAWN rather than declared.**
##
## `app.gd`'s strip declares a 44 px floor in three places -- `_tl_square()`
## grows the transport to `role_px("btn_min_h")`, `_build_timeline_scrub()`
## takes `maxf(timeline_track_h, btn_min_h)`, and `DccWidgets.text_button()`
## carries the phone tap floor -- and until this probe nobody had walked the row
## at touch density and read the **laid** heights back. That distinction has
## already cost this tree twice: a field declared to floor at 44 drew 22 px at
## tablet because `phone_fit()` only runs on a phone, and six chips drew 43 px
## against a 115 px floor while a probe printed the number and nobody read it.
##
## **What this probe asserts is a THRESHOLD, not the board's value, and that
## limit is stated rather than left for a reader to discover.** Every check is
## `>= floor on both axes`, so a 44 x 400 transport would still print `ok`,
## while `Timeline.dc.html` board H draws the three transport squares at
## exactly `44px x 44px`. The measured size is printed beside every verdict, so
## the number is on screen even where the assertion does not pin it.
##
## A threshold is the right shape here -- the floor is a MINIMUM and a taller
## control is not a defect -- but "a floor is not a pin" is a rule this project
## learned expensively: a cube changed to a square passed a whole crate green
## because the only assertion over it was `> 1/255`.
##
## So this probe reads `Control.size.y` after a real container sort, names the
## density beside every number, and **asserts** -- it does not merely print.
##
## Run (one density per process: `DccTheme._touch` / `_phone_mode` are latched
## for the life of the process, so the legs cannot share a run):
##
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . \
##     _tlfloor_probe.tscn -- --tablet --force-touch
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . \
##     _tlfloor_probe.tscn -- --tablet-portrait --force-touch
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . \
##     _tlfloor_probe.tscn -- --tablet-narrow --force-touch
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . \
##     _tlfloor_probe.tscn -- --phone --force-touch
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . \
##     _tlfloor_probe.tscn -- --pointer
##
## Every flag above is read by `_ready()` below and an unrecognised leg flag is
## a hard refusal, not a default -- `MISTAKES.md`'s "a probe's usage header is a
## claim about the probe's own code" row.
##
## **`--headless` is correct here and `--resolution` is not used.** Nothing
## rasterises and nothing is timed: every number is a laid-out `size.y` or a
## `custom_minimum_size.y`. The composition is chosen by the **SubViewport's**
## own size, which `--headless` honours (the 64x64 root viewport `--headless`
## reports is what makes `--resolution` useless, and is exactly why
## `_tapfloor_probe.gd` and `_tabletparity_probe.gd` host the shell in a
## `SubViewport` too).
##
## ## The densities, and the floor each one is judged against
##
## | leg | SubViewport | `is_*()` | floor | where the floor comes from |
## |---|---|---|---|---|
## | `--tablet` | 2560 x 1600 | `is_tablet()` | **44 px** | `ROLE["btn_min_h"] == [0, 44]`, tier A |
## | `--tablet-portrait` | 1600 x 2560 | `is_tablet()` | **44 px** | same |
## | `--tablet-narrow` | 1400 x 1000 | `is_tablet()` | **44 px** | same |
## | `--phone` | 1080 x 2340 | `is_phone()` | **115 px** | `PHONE_TAP_MIN (44) * phone_scale (2.621)` |
## | `--pointer` | 1920 x 1080 | neither | **none** | `role_px("btn_min_h")` is `0` on the desktop, which `ROLE`'s own note defines as "the design states no constraint" |
##
## Three tablet widths and not one, because the row's minimum is
## content-dependent (the six layer pills alone are ~610 px of un-clippable
## label at tablet type) and one width is one sample.
##
## The pointer leg is the control state: a set of heights with nothing to
## compare them against cannot tell you whose defect they are. Every figure in
## it is byte-identical before and after this pass's floors, which is what makes
## "touch-only" a measurement rather than an argument from `role_px` returning
## `0`.
##
## 2560 x 1600 is aspect 0.625, above `_PHONE_ASPECT_MAX` (0.6), so it
## classifies as tablet; 1080 x 2340 is 0.462 and classifies as phone. Both need
## `--force-touch`, because real touch hardware is never present in this dev
## environment (`dcc_shell.gd::_ready()`).

var _fail := 0
var _app: Node
var _leg := ""
var _floor := 0.0
var _dens := ""

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

## The density this process actually booted into, read back off `DccTheme`
## rather than off the flag that asked for it -- a leg that silently fell into
## the wrong composition would otherwise report the wrong floor's verdict.
func _density() -> String:
	if DccTheme.is_phone():
		return "PHONE"
	if DccTheme.is_tablet():
		return "TABLET"
	if DccTheme.is_laptop():
		return "LAPTOP"
	return "DESKTOP"

## **What counts as a finger target on this row.**
##
## `BaseButton` covers the collapsed strip's `hit`, the three transport squares,
## the three speed pills, the six layer toggles and the collapse chevron. The
## scrub track is none of those -- it is a bare `Control` that answers
## `gui_input` -- so the predicate is structural and covers both: anything that
## takes pointer input and has a handler on it.
##
## `DccTheme.spacer()` is a bare `Control` too, and it is excluded correctly
## rather than by a name test: it has no `gui_input` connection, so the second
## clause is false for it.
func _is_target(c: Control) -> bool:
	## `_widthfloor_probe.gd::_is_tappable`'s own list, verbatim, so the two
	## walks cannot disagree about what a target is. `Range` is in it and it
	## matters: the phone sim strip's year scrub is an `HSlider`, and a
	## `BaseButton`-only filter reported that strip as five targets when it has
	## six. Widening a filter is only ever suspect when it makes a number look
	## better; this one made it worse.
	if c is BaseButton or c is LineEdit or c is Range or c is TextEdit:
		return true
	return c.mouse_filter != Control.MOUSE_FILTER_IGNORE \
		and c.get_signal_connection_list("gui_input").size() > 0

## A short, findable identity for a control in the row -- its text where it has
## one, else its class plus the `_tl_*` handle `app.gd` holds it by.
func _who(c: Control) -> String:
	var t := str(c.get("text")) if c is Button or c is Label else ""
	if t != "":
		return "%s \"%s\"" % [c.get_class(), t]
	for handle in ["_tl_track", "_tl_head", "_tl_bubble", "_tl_phone_button",
			"_tl_play_button", "_tl_fwd_button", "_phone_sim_play",
			"_phone_sim_slider"]:
		if _app.get(handle) == c:
			return "%s <%s>" % [c.get_class(), handle]
	## The collapsed strip's expand target: a textless `Button` parented
	## straight to `timeline_row`. Named structurally rather than by its
	## `@Button@4748` runtime name, which means nothing between runs.
	if c is Button and c.get_parent() == _app.timeline_row:
		return "Button <collapsed strip hit>"
	return "%s <%s>" % [c.get_class(), str(c.name)]

## **The one exemption, its axis, and its reason -- stated here and nowhere
## else.**
##
## An exemption is a claim about the control and is checked like any other. The
## failure mode this probe exists to prevent is the count reaching zero by
## widening this function, so the arm below names an **approved drawing** that
## makes the floor the wrong bar for that control on that axis, not merely that
## flooring it would be inconvenient. There is exactly one arm, and it is
## height-only: nothing on this row is exempt from the WIDTH floor.
##
## **The scrub track is deliberately NOT an arm here, and that is a finding.**
## A track is dragged along its length rather than tapped at a point, so it is
## the obvious candidate for an exemption -- and it does not need one. Measured
## at three tablet widths: **2532 x 44**, **1572 x 44** and **1372 x 44** px,
## i.e. 111 408 / 69 168 / 60 368 px^2 of hit area, with the height at exactly
## the floor because `_build_timeline_scrub()` takes
## `maxf(role_px("timeline_track_h"), role_px("btn_min_h"))`. The 3 px rail and
## the marks inside it are `MOUSE_FILTER_IGNORE` `ColorRect`s and are not
## targets at all -- board H's own note, *"the row is the target, the rail is
## only its drawing"*. An exemption written for it would have been an untrue
## reason for a control that already clears the bar.
##
## Returns the reason, or `""` for "not exempt, judge it against the floor".
func _exempt_height(c: Control, form: String) -> String:
	## **The collapsed strip is 34 px on touch by design, and the design says so
	## in the same breath as it says the transport is 44.**
	## `design/proposed-2026-09-05-round2/Timeline.dc.html` board H is headed
	## *"H · touch density — §4.1's 34 px strip, and the transport floored at
	## 44"* and draws the collapsed row as `height:34px` beside three
	## `width:44px;height:44px` transport squares. So 34 is not an oversight the
	## board forgot to floor; it is the figure the board picked while flooring
	## its neighbour in the same drawing.
	##
	## `app.gd::_build_timeline_collapsed()` pins it to `role_px("h_status") - 2`
	## -- 34 at tablet -- and the row inside loses 1 px more to the bar's top
	## rule, which is the 33 measured below.
	##
	## **What the target actually is**, since a height alone would misdescribe
	## it: the whole strip is one `Button` at `SIZE_EXPAND_FILL`, so it is the
	## full width between the two docks. The area is printed beside every
	## verdict rather than asserted from memory.
	##
	## **Gated on touch, because the reason is about touch.** An exemption
	## printed at pointer density would be citing a touch board for a row no
	## touch floor is being applied to -- a true sentence in the wrong place,
	## which is how a reason stops being checkable.
	if form == "collapsed" and DccTheme.is_touch() \
			and c is Button and c.get_parent() == _app.timeline_row:
		return "board H draws §4.1's collapsed strip at 34 px on touch; the target is the full-width row"
	return ""

## Every target under `timeline_row`, deepest-first order irrelevant -- the row
## is three levels at most.
func _collect(n: Node, out: Array) -> void:
	for c in n.get_children():
		if c is Control and _is_target(c as Control):
			out.append(c)
		_collect(c, out)

## Walk the row and report. `form` is `collapsed` / `expanded` / `phone`, and
## every printed number carries `_dens` beside it.
##
## **Both axes are judged.** A tap floor is a floor on the target, and the
## defect this probe found at tablet was on the axis a height-only walk cannot
## see: the transport squares drew 36 x 44 and the collapse chevron 7 x 44.
func _walk(form: String) -> void:
	var row: Control = _app.timeline_row
	var bar: Control = _app.timeline_bar
	print("\n--- %s form, %s density, floor %.0f px on BOTH axes ---" % [form, _dens, _floor])
	print("  timeline_bar.visible=%s  bar.size.y=%.0f  row.size.y=%.0f  row.min.x=%.0f  children=%d"
		% [str(bar.visible), bar.size.y, row.size.y,
		   row.get_combined_minimum_size().x, row.get_child_count()])
	if not bar.visible:
		_fail += 1
		print("  FAIL the timeline bar is not visible -- nothing was measured")
		return
	var targets: Array = []
	_collect(row, targets)
	if targets.is_empty():
		_fail += 1
		print("  FAIL no tappable control found in timeline_row -- the walk is broken,")
		print("       not the row. A count of zero is never a pass here.")
		return
	var under := 0
	var exempted := 0
	for t in targets:
		var c := t as Control
		var why := _exempt_height(c, form)
		var h := c.size.y
		var w := c.size.x
		var bad_h := h < _floor - 0.5 and why == ""
		var bad_w := w < _floor - 0.5
		var verdict := "ok    "
		if bad_h or bad_w:
			verdict = "UNDER%s%s" % ["H" if bad_h else "", "W" if bad_w else ""]
			under += 1
			_fail += 1
		elif why != "":
			verdict = "EXEMPT"
			exempted += 1
		print("    %-7s %-34s drawn %7.1f x %-6.1f px = %8.0f px^2  min=(%.0f,%.0f)  [%s]%s"
			% [verdict, _who(c), w, h, w * h,
			   c.custom_minimum_size.x, c.custom_minimum_size.y, _dens,
			   ("  -- exempt on height: " + why) if why != "" else ""])
	print("  %d target(s): %d under the %.0f px floor at %s, %d exempted (height only)"
		% [targets.size(), under, _floor, _dens, exempted])
	## **Does the row still fit?** `_build_timeline_layers()`'s own header says
	## this row's minimum "becomes the window's floor", so a floor that grows a
	## control has to be checked against the space the row is actually given --
	## not asserted (the overflow, where there is one, predates every floor
	## applied here), but printed, so nobody has to take the width on trust.
	print("  row has %.0f px, wants %.0f px  -> %s"
		% [row.size.x, row.get_combined_minimum_size().x,
		   "fits" if row.size.x >= row.get_combined_minimum_size().x - 0.5
		   else "OVERFLOWS by %.0f px" % (row.get_combined_minimum_size().x - row.size.x)])

## **What the floors cost the row in width, measured on the shipped nodes.**
##
## Not arithmetic and not a replica: this puts the three transport squares, the
## three speed pills and the collapse chevron back to the exact minimum their
## own factory hands out (`_menu_square()`'s `Vector2(MENU_CTL[1],
## MENU_CTL[1])`, `segment()`'s `role_px("chip_min_h")` on `y` alone, and
## `text_button()`'s nothing-at-all off a phone), re-sorts the real
## `timeline_row`, reads its combined minimum, and puts the floors back. Every
## "before" figure is read off `DccShell`/`DccTheme` rather than typed, so this
## cannot drift from what those factories actually do.
##
## The row's own minimum is what `app.gd`'s `_build_timeline_layers()` header
## calls "the window's floor", which is why the cost is worth a number rather
## than a shrug.
func _width_cost() -> void:
	var row: Control = _app.timeline_row
	var after: float = row.get_combined_minimum_size().x
	var saved: Dictionary = {}
	var sq := float(DccShell.MENU_CTL[1] if DccTheme.is_touch() else DccShell.MENU_CTL[0])
	for b in _app.get("_tl_transport"):
		var c := b as Control
		saved[c] = c.custom_minimum_size
		if c is Button and str(c.get("text")).begins_with("×"):
			c.custom_minimum_size = Vector2(c.custom_minimum_size.x,
				float(DccTheme.role_px("chip_min_h")))
		elif c is Button:
			c.custom_minimum_size = Vector2(sq, sq)
	var chev: Control = null
	for t in _collect_targets():
		if t is Button and str((t as Button).text) == DccIcons.SYMBOLS["chevron"]:
			chev = t as Control
	if chev != null:
		saved[chev] = chev.custom_minimum_size
		chev.custom_minimum_size = Vector2.ZERO
	await _frames(6)
	var before: float = row.get_combined_minimum_size().x
	for c in saved:
		(c as Control).custom_minimum_size = saved[c]
	await _frames(6)
	var restored: float = row.get_combined_minimum_size().x
	print("  row combined minimum x at %s: %.0f px WITHOUT the floors -> %.0f px WITH them (+%.0f)"
		% [_dens, before, after, after - before])
	if absf(restored - after) > 0.5:
		_fail += 1
		print("  FAIL the restore left the row at %.0f px, not the %.0f it started at"
			% [restored, after])

func _collect_targets() -> Array:
	var out: Array = []
	_collect(_app.timeline_row, out)
	return out

## The phone's view of the very controls this row carries on desktop and tablet:
## `06-phone.md` §6.2's floating sim strip, which holds the play button, the
## speed pills and the year slider. **Built by `dcc_shell.gd`, which this lane
## does not own** -- so this section measures and reports and never asserts. A
## number here that is under the floor is a finding to hand on, not a failure of
## `app.gd`.
func _walk_phone_sim_strip() -> void:
	var strip: Control = _app.get("_phone_sim_strip") as Control
	print("\n--- phone sim strip (dcc_shell.gd -- REPORT ONLY, not this lane's file) ---")
	if strip == null:
		print("  _phone_sim_strip is null -- nothing built")
		return
	_app.call("set_phone_sim_strip_open", true)
	await _frames(12)
	var targets: Array = []
	_collect(strip, targets)
	print("  strip.visible=%s  strip.size=%.0f x %.0f  targets=%d"
		% [str(strip.visible), strip.size.x, strip.size.y, targets.size()])
	var under := 0
	for t in targets:
		var c := t as Control
		var short_axis := minf(c.size.x, c.size.y)
		if short_axis < _floor - 0.5:
			under += 1
		print("    %-6s %-34s drawn %7.1f x %-6.1f px  min=(%.0f,%.0f)  [%s]"
			% ["under" if short_axis < _floor - 0.5 else "ok",
			   _who(c), c.size.x, c.size.y,
			   c.custom_minimum_size.x, c.custom_minimum_size.y, _dens])
	print("  %d of %d under the %.0f px floor at %s -- reported, NOT asserted"
		% [under, targets.size(), _floor, _dens])

func _ready() -> void:
	await _frames(2)
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load"); get_tree().quit(1); return
	var args := OS.get_cmdline_user_args()
	var legs := 0
	var vp_size := Vector2i(1920, 1080)
	for a in args:
		match a:
			"--tablet":
				_leg = "tablet"; vp_size = Vector2i(2560, 1600); legs += 1
			"--tablet-narrow":
				## The third tablet sample, and the one that can fail: 1400 x
				## 1000 is the size `_tlheight_probe.gd` already uses for its
				## own touch leg, and at 1400 px the two 400 px docks and the
				## 48 px rail leave the row about 550 px of the ~1100 it wants.
				## Sampled because one width is one sample and this row's
				## minimum is content-dependent -- `MISTAKES.md`'s "a
				## single-sample layout number is the same error class as a
				## single-sample timing".
				_leg = "tablet"; vp_size = Vector2i(1400, 1000); legs += 1
			"--tablet-portrait":
				## The second tablet sample. One size is one sample, and the
				## row's minimum is content-dependent -- the six layer pills
				## alone are ~610 px of un-clippable label at tablet type. A
				## 1600 x 2560 slate is 0.625 aspect, so it still classifies as
				## tablet, and it is the narrowest real tablet this row has to
				## survive.
				_leg = "tablet"; vp_size = Vector2i(1600, 2560); legs += 1
			"--phone":
				_leg = "phone"; vp_size = Vector2i(1080, 2340); legs += 1
			"--pointer":
				_leg = "pointer"; vp_size = Vector2i(1920, 1080); legs += 1
			"--force-touch":
				pass
			_:
				print("[FATAL] unrecognised argument '%s' -- this probe reads only" % a,
					" --tablet / --phone / --pointer / --force-touch")
				get_tree().quit(1); return
	if legs != 1:
		print("[FATAL] pass exactly one of --tablet / --phone / --pointer (got ", legs, ")")
		get_tree().quit(1); return
	if _leg != "pointer" and not ("--force-touch" in args):
		print("[FATAL] --%s needs --force-touch: _touch can never be true in this" % _leg,
			" dev environment without it (dcc_shell.gd::_ready)")
		get_tree().quit(1); return

	var vp := SubViewport.new()
	vp.size = vp_size
	vp.gui_embed_subwindows = true
	add_child(vp)
	_app = load("res://shell/app.tscn").instantiate()
	vp.add_child(_app)
	await _frames(60)
	if _app.open_project_dialog != null:
		_app.open_project_dialog.hide()
	await _frames(4)

	_dens = _density()
	var want: String = {"tablet": "TABLET", "phone": "PHONE", "pointer": "DESKTOP"}[_leg]
	if _dens != want:
		_fail += 1
		print("[FATAL] --%s booted into %s, not %s -- every floor below would be" % [_leg, _dens, want],
			" judged against the wrong bar")
		get_tree().quit(1); return
	## The floor, derived from the same table the shell resolves it from --
	## never a literal here, or this probe would pass for every value of it.
	if _dens == "PHONE":
		_floor = round(float(DccTheme.PHONE_TAP_MIN) * DccTheme.phone_scale())
	else:
		_floor = float(DccTheme.role_px("btn_min_h"))
	print("[BOOT] leg=%s  viewport=%dx%d  density=%s  phone_scale=%.3f  floor=%.0f px"
		% [_leg, vp_size.x, vp_size.y, _dens, DccTheme.phone_scale(), _floor])
	print("       role_px(btn_min_h)=%d  role_px(chip_min_h)=%d  role_px(timeline_track_h)=%d  role_px(h_status)=%d"
		% [DccTheme.role_px("btn_min_h"), DccTheme.role_px("chip_min_h"),
		   DccTheme.role_px("timeline_track_h"), DccTheme.role_px("h_status")])

	## The strip is only visible in CIVIL (`app.gd::_on_workspace_changed`).
	_app.select_domain_mode("civilization", "factions")
	await _frames(10)

	if _dens == "PHONE":
		_walk("phone")
		await _walk_phone_sim_strip()
	else:
		_walk("collapsed")
		_app.set("_tl_expanded", true)
		_app.call("_fill_timeline_strip")
		await _frames(12)
		_walk("expanded")
		await _width_cost()

	print("\n_tlfloor_probe (%s): %s" % [_dens, "clean" if _fail == 0 else str(_fail) + " FAILURE(S)"])
	get_tree().quit(1 if _fail > 0 else 0)
