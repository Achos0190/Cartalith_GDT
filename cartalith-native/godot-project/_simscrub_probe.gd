extends Node
## **The phone sim strip's width budget, and the scrub track inside it.**
##
## `_tlfloor_probe.gd` found the defect and could only report it: its
## `_walk_phone_sim_strip()` is explicitly "REPORT ONLY, not this lane's file",
## because the strip is built by `dcc_shell.gd::_build_phone_sim_strip()` and
## that probe belongs to `app.gd`'s timeline row. This probe **asserts** on the
## same six controls, and it adds the thing a floor walk cannot give you: the
## **arithmetic of the row**, term by term, so the next person can see which
## term is largest rather than re-deriving it.
##
## ## What made this control different from the other five
##
## `_phone_sim_slider` is the only child of the strip at `SIZE_EXPAND_FILL`.
## Its siblings all declare a width (`_ptap(...)`, which floors at
## `PHONE_TAP_MIN * phone_scale`), so the slider is handed the **remainder** --
## and on a 1080 px screen the remainder was 96 px against a 115 px floor.
## Flooring the slider alone could not fix it: the row's minimum would simply
## have exceeded the room and something else would have shrunk. The fix had to
## change the layout, so this probe measures the layout and not just the sizes.
##
## ## What it asserts, and the limit of each assertion
##
## | # | Assertion | What it does NOT say |
## |---|---|---|
## | A1 | every target is `>= floor` on **both axes** | it is a threshold, not a pin -- a 115 x 900 control still passes |
## | A2 | the target count is exactly 6, of which exactly one is a `Range` | nothing about which six |
## | A3 | the strip's combined minimum width fits the width it is given | nothing about height |
## | A4 | the strip's rect lies inside `_phone_content_gap` | nothing about overlap with the undo chip, which is measured and printed instead |
## | A5 | the scrub spans the strip's full inner content width (its own row) | nothing about the track's *drawing* |
##
## A1 is the defect's own bar and A5 is the fix's shape: A1 alone would pass a
## 115 px scrub sharing a row, which is a target but not a track. **A floor is
## not a pin**, so both are needed and both are derived -- A1 from
## `DccTheme.PHONE_TAP_MIN * phone_scale()` and A5 from the strip's measured
## stylebox margins, never from a literal typed here.
##
## The `--tablet` leg is the control state: the strip is phone-only chrome, so
## at tablet density the assertion is that it does not exist at all. A pass
## there is what makes "phone-density defect" a measurement rather than an
## assumption that `_build_phone_shell()` never ran.
##
## ## Run (one density per process -- `DccTheme._touch` / `_phone_mode` are
## latched for the life of the process, so the legs cannot share a run)
##
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . \
##     _simscrub_probe.tscn -- --phone --force-touch
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . \
##     _simscrub_probe.tscn -- --phone-1440 --force-touch
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . \
##     _simscrub_probe.tscn -- --phone-landscape --force-touch
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . \
##     _simscrub_probe.tscn -- --tablet --force-touch
##
## Every flag is read by `_ready()` below and an unrecognised one is a hard
## refusal, not a default. There is no `--resolution` and none is read: the
## composition is chosen by the **SubViewport's** size, which `--headless`
## honours where the 64x64 root viewport does not.
##
## **`--headless` is correct here.** Nothing rasterises and nothing is timed:
## every number is a laid-out `size`, a `custom_minimum_size` or a stylebox
## content margin. (`ImageTexture.update()` being a no-op under the dummy
## driver is why pixel probes in this tree run windowed; this one reads no
## pixels.) Capture stderr anyway when quoting an error count -- `--headless`
## manufactures its own from the dummy texture storage at exit.

var _fail := 0
var _app: Node
var _vp: SubViewport
var _leg := ""
var _floor := 0.0
var _dens := ""

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

## The density this process actually booted into, read off `DccTheme` rather
## than off the flag that asked for it: a leg that fell into the wrong
## composition would otherwise report the wrong floor's verdict.
func _density() -> String:
	if DccTheme.is_phone():
		return "PHONE"
	if DccTheme.is_tablet():
		return "TABLET"
	if DccTheme.is_laptop():
		return "LAPTOP"
	return "DESKTOP"

## `_tlfloor_probe.gd::_is_target` and `_widthfloor_probe.gd::_is_tappable`'s
## list, verbatim, so the three walks cannot disagree about what a target is.
## **`Range` is in it and it is the whole reason this probe exists**: a
## `BaseButton`-only filter reports this strip as five targets when it has six,
## and the one it drops is the one that was broken.
func _is_target(c: Control) -> bool:
	if c is BaseButton or c is LineEdit or c is Range or c is TextEdit:
		return true
	return c.mouse_filter != Control.MOUSE_FILTER_IGNORE \
		and c.get_signal_connection_list("gui_input").size() > 0

func _who(c: Control) -> String:
	for handle in ["_phone_sim_play", "_phone_sim_slider", "_phone_sim_year"]:
		if _app.get(handle) == c:
			return "%s <%s>" % [c.get_class(), handle]
	var t := str(c.get("text")) if c is Button or c is Label else ""
	if t != "":
		return "%s \"%s\"" % [c.get_class(), t]
	return "%s <%s>" % [c.get_class(), str(c.name)]

func _strip() -> Control:
	return _app.get("_phone_sim_strip") as Control

func _collect(n: Node, out: Array) -> void:
	for c in n.get_children():
		if c is Control and _is_target(c as Control):
			out.append(c)
		_collect(c, out)

## What a physical pixel is worth in the 412 dp space every constant in
## `_build_phone_sim_strip()` is authored in. Printed beside every figure so a
## number can be compared against the canvas without arithmetic in the reader's
## head.
func _dp(px: float) -> float:
	return px / DccTheme.phone_scale()

## **The arithmetic, term by term.** Walks the strip's box containers and
## prints, for each level: the container's own width, its separation constant,
## every child's laid and minimum width, and the residual left over for
## whatever expands. This is the section that says *which term is largest*,
## which is the thing a size walk alone cannot tell you.
func _arithmetic(tag: String) -> void:
	var strip := _strip()
	var sb: StyleBox = strip.get_theme_stylebox("panel")
	var ml: float = sb.content_margin_left
	var mr: float = sb.content_margin_right
	var mt: float = sb.content_margin_top
	var mb: float = sb.content_margin_bottom
	print("\n--- strip arithmetic (%s), %s density, phone_scale %.3f ---"
		% [tag, _dens, DccTheme.phone_scale()])
	print("  PanelContainer %.0f x %.0f px (%.1f x %.1f dp)   content margins L/R %.0f/%.0f  T/B %.0f/%.0f"
		% [strip.size.x, strip.size.y, _dp(strip.size.x), _dp(strip.size.y), ml, mr, mt, mb])
	print("  inner content width = %.0f - %.0f - %.0f = %.0f px (%.1f dp)"
		% [strip.size.x, ml, mr, strip.size.x - ml - mr, _dp(strip.size.x - ml - mr)])
	_dump_box(strip, 1)

func _dump_box(n: Node, depth: int) -> void:
	for child in n.get_children():
		if child is BoxContainer:
			var box := child as BoxContainer
			var sep := float(box.get_theme_constant("separation"))
			var kids: Array = []
			for k in box.get_children():
				if k is Control and (k as Control).visible:
					kids.append(k)
			var axis := "H" if box is HBoxContainer else "V"
			print("%s%sBox %.0f x %.0f px, separation %.0f, %d visible children -> %.0f px of gaps"
				% ["  ".repeat(depth), axis, box.size.x, box.size.y, sep, kids.size(),
				   sep * maxf(0.0, kids.size() - 1)])
			var fixed := 0.0
			var expanding := 0.0
			for k in kids:
				var c := k as Control
				var exp_h := (c.size_flags_horizontal & Control.SIZE_EXPAND) != 0
				var along: float = c.size.x if axis == "H" else c.size.y
				var min_along: float = c.custom_minimum_size.x if axis == "H" \
					else c.custom_minimum_size.y
				if axis == "H":
					if exp_h:
						expanding += along
					else:
						fixed += along
				print("%s  %-34s laid %7.1f x %-7.1f px  (%5.1f x %5.1f dp)  min=(%.0f,%.0f)%s"
					% ["  ".repeat(depth), _who(c), c.size.x, c.size.y,
					   _dp(c.size.x), _dp(c.size.y),
					   c.custom_minimum_size.x, c.custom_minimum_size.y,
					   "  EXPAND_FILL" if exp_h else ""])
			if axis == "H":
				var gaps := sep * maxf(0.0, kids.size() - 1)
				print("%s  SUM: fixed %.0f + gaps %.0f = %.0f px of %.0f  -> %.0f px remainder for %s"
					% ["  ".repeat(depth), fixed, gaps, fixed + gaps, box.size.x,
					   box.size.x - fixed - gaps,
					   "the expanding child" if expanding > 0.0 else "nothing (no expander)"])
			print("%s  combined minimum %.0f x %.0f px, given %.0f x %.0f -> %s"
				% ["  ".repeat(depth), box.get_combined_minimum_size().x,
				   box.get_combined_minimum_size().y, box.size.x, box.size.y,
				   "fits" if box.size.x >= box.get_combined_minimum_size().x - 0.5
				   else "OVERFLOWS by %.0f px" % (box.get_combined_minimum_size().x - box.size.x)])
		_dump_box(child, depth + 1)

## A1/A2: the six targets, both axes, against the floor.
func _walk_targets(tag: String) -> Array:
	var strip := _strip()
	var targets: Array = []
	_collect(strip, targets)
	print("\n--- targets (%s), floor %.0f px = %.0f dp on BOTH axes, %s ---"
		% [tag, _floor, _dp(_floor), _dens])
	if targets.is_empty():
		_fail += 1
		print("  FAIL no target found in the strip -- the walk is broken, not the strip.")
		return targets
	var ranges := 0
	for t in targets:
		var c := t as Control
		if c is Range:
			ranges += 1
		var bad_w := c.size.x < _floor - 0.5
		var bad_h := c.size.y < _floor - 0.5
		var verdict := "ok    "
		if bad_w or bad_h:
			verdict = "UNDER%s%s" % ["W" if bad_w else "", "H" if bad_h else ""]
			_fail += 1
		print("    %-7s %-34s drawn %7.1f x %-7.1f px = %8.0f px^2  (%5.1f x %5.1f dp)  [%s]"
			% [verdict, _who(c), c.size.x, c.size.y, c.size.x * c.size.y,
			   _dp(c.size.x), _dp(c.size.y), _dens])
	## A2. The count is asserted because the walk that found this defect had a
	## `BaseButton`-only filter and reported **5** where there are 6 -- a
	## narrower filter here would make every other assertion vacuous for the
	## one control that was actually broken.
	if targets.size() != 6 or ranges != 1:
		_fail += 1
		print("  FAIL expected 6 targets of which exactly 1 is a Range; got %d targets, %d Range(s)"
			% [targets.size(), ranges])
	else:
		print("  6 targets, 1 of them a Range (the scrub) -- the filter sees the slider")
	return targets

## A3/A4/A5: does the strip fit the width it has, stay inside its parent, and
## does the scrub get its own full-width row?
func _walk_geometry() -> void:
	var strip := _strip()
	var gap: Control = _app.get("_phone_content_gap") as Control
	var sb: StyleBox = strip.get_theme_stylebox("panel")
	var inner: float = strip.size.x - sb.content_margin_left - sb.content_margin_right
	var slider: Control = _app.get("_phone_sim_slider") as Control
	print("\n--- geometry, %s ---" % _dens)
	## A3
	var want: float = strip.get_combined_minimum_size().x
	print("  A3 strip wants %.0f px, has %.0f px -> %s" % [want, strip.size.x,
		"fits" if strip.size.x >= want - 0.5 else "OVERFLOWS by %.0f px" % (want - strip.size.x)])
	if strip.size.x < want - 0.5:
		_fail += 1
	## A4 -- the height budget. The strip grows UPWARD (`PRESET_BOTTOM_WIDE`
	## with `GROW_DIRECTION_BEGIN`), so a taller strip eats into the map, and
	## the bound that matters is `_phone_content_gap`, which already ends above
	## the tool sheet and the bottom bar.
	var sr := strip.get_global_rect()
	var gr := gap.get_global_rect()
	print("  A4 strip rect y %.0f..%.0f inside _phone_content_gap y %.0f..%.0f (gap is %.0f px tall) -> %s"
		% [sr.position.y, sr.end.y, gr.position.y, gr.end.y, gr.size.y,
		   "inside" if sr.position.y >= gr.position.y - 0.5 and sr.end.y <= gr.end.y + 0.5
		   else "OUTSIDE"])
	if sr.position.y < gr.position.y - 0.5 or sr.end.y > gr.end.y + 0.5:
		_fail += 1
	print("     strip uses %.0f of the gap's %.0f px (%.0f%%); %.0f px of map left above it"
		% [sr.size.y, gr.size.y, 100.0 * sr.size.y / maxf(1.0, gr.size.y),
		   sr.position.y - gr.position.y])
	## A5 -- the scrub gets its own row. Compared against the strip's measured
	## inner content width, never a literal: this is the fix's shape, and A1
	## alone would pass a 115 px scrub sharing a row with five siblings.
	print("  A5 scrub %.0f px of a %.0f px inner width (%.1f dp of track over %d years = %.2f years/px) -> %s"
		% [slider.size.x, inner, _dp(slider.size.x),
		   int(slider.max_value - slider.min_value),
		   (slider.max_value - slider.min_value) / maxf(1.0, slider.size.x),
		   "own row" if slider.size.x >= inner - 1.0 else "SHARES ITS ROW"])
	if slider.size.x < inner - 1.0:
		_fail += 1
	## Reported, not asserted: the undo chip is bottom-anchored in the same
	## container and is another lane's control. Whether the two overlap is a
	## finding to hand on, not this probe's verdict.
	var chip: Control = _app.get("_phone_undo_chip") as Control
	if chip != null:
		var was: bool = chip.visible
		chip.visible = true
		await _frames(4)
		var cr := chip.get_global_rect()
		print("  -- undo chip rect %.0f,%.0f %.0fx%.0f vs strip %.0f,%.0f %.0fx%.0f -> %s (REPORTED, not asserted)"
			% [cr.position.x, cr.position.y, cr.size.x, cr.size.y,
			   sr.position.x, sr.position.y, sr.size.x, sr.size.y,
			   "OVERLAPS" if cr.intersects(sr) else "clear"])
		chip.visible = was
		await _frames(2)

## The year readout is the one term in the row whose width is content-dependent,
## and a shell with no world draws the **shorter** of its two strings. One
## string is one sample: this re-measures with the widest text the label can
## ever carry, so the budget above is a worst case rather than a lucky one.
##
## `TL_YEAR_MIN` is -400, so "YEAR -400" (9 glyphs) is the widest `YEAR %d` can
## be; "NO WORLD" is 8. Both are set here rather than reasoned about.
func _widest_year() -> void:
	var label: Control = _app.get("_phone_sim_year") as Control
	if label == null:
		_fail += 1
		print("\n  FAIL _phone_sim_year is null")
		return
	var was: String = str(label.get("text"))
	print("\n--- year readout, both strings, %s ---" % _dens)
	for s in ["NO WORLD", "YEAR %d" % int(_app.get("TL_YEAR_MIN")),
			"YEAR %d" % int(_app.get("TL_YEAR_MAX"))]:
		label.set("text", s)
		await _frames(6)
		print("    %-10s label %6.1f px (%5.1f dp)   strip minimum %.0f px, has %.0f -> %s"
			% [s, label.size.x, _dp(label.size.x),
			   _strip().get_combined_minimum_size().x, _strip().size.x,
			   "fits" if _strip().size.x >= _strip().get_combined_minimum_size().x - 0.5
			   else "OVERFLOWS"])
		if _strip().size.x < _strip().get_combined_minimum_size().x - 0.5:
			_fail += 1
	label.set("text", was)
	await _frames(4)

## **The two options that lost, measured rather than argued.**
##
## A layout decision made on taste ages badly and cannot be re-checked. These
## are the numbers the choice was made on, taken from the shipped controls'
## own minimum widths, so they stay true as those minimums move.
##
## **Option 1, wrap the row with an `HFlowContainer`.** The shell has the
## precedent (the search chips), and the phone sheet has vertical room. It
## fails on a mechanism rather than on taste: a `FlowContainer` wraps on its
## children's **minimum** widths, and the scrub's declared minimum is 0 (it is
## the expander). So the row's minimum fits and the flow lays exactly one line
## -- the same starved slider, in a different container. Declaring the floor on
## the scrub *does* make it wrap, and then it wraps in the wrong place: the
## break lands wherever the arithmetic puts it, which strands the last control
## alone on line 2 and still leaves the track sharing line 1.
##
## **Option 2, shrink the three speed pills.** They are three of the row's five
## `_ptap()` consumers, so they are where the width is. This prints what each
## pill would have to shrink to for the scrub to reach the floor, in dp, beside
## the 44 dp floor it would be breaking -- one violation traded for three, and
## the arithmetic says by how much.
func _losing_options() -> void:
	var strip := _strip()
	var sb: StyleBox = strip.get_theme_stylebox("panel")
	var inner: float = strip.size.x - sb.content_margin_left - sb.content_margin_right
	var slider: Control = _app.get("_phone_sim_slider") as Control
	var play: Control = _app.get("_phone_sim_play") as Control
	var year: Control = _app.get("_phone_sim_year") as Control
	## The canvas's own order (`06-phone.md` §6.2): play · YEAR · slider ·
	## ×1 ×10 ×100 · ✕. Widths are read off the shipped controls -- the year
	## readout has no declared minimum, so its content width stands in for it.
	var pills: Array = []
	var close: Control = null
	for t in _collect_targets():
		var c := t as Control
		if c is Button and str((c as Button).text).begins_with("×"):
			pills.append(c)
		elif c is Button and c != play:
			close = c
	var sep := float(_row_separation())
	print("\n--- the two options that lost, %s (inner width %.0f px, separation %.0f) ---"
		% [_dens, inner, sep])

	## -- Option 1 -------------------------------------------------------------
	for scrub_min in [0.0, _floor]:
		var mins: Array = [play.custom_minimum_size.x, maxf(year.custom_minimum_size.x,
			year.get_combined_minimum_size().x), scrub_min]
		for p in pills:
			mins.append((p as Control).custom_minimum_size.x)
		mins.append(close.custom_minimum_size.x)
		var host := Control.new()
		host.position = Vector2(0.0, float(_vp.size.y) * 2.0)  ## off-screen
		host.size = Vector2(inner, 900.0)
		_vp.add_child(host)
		var flow := HFlowContainer.new()
		flow.add_theme_constant_override("h_separation", int(sep))
		flow.add_theme_constant_override("v_separation", int(sep))
		flow.set_anchors_preset(Control.PRESET_FULL_RECT)
		host.add_child(flow)
		var stand_ins: Array = []
		for i in mins.size():
			var s := Control.new()
			s.custom_minimum_size = Vector2(float(mins[i]), _floor)
			if i == 2:
				s.size_flags_horizontal = Control.SIZE_EXPAND_FILL
			flow.add_child(s)
			stand_ins.append(s)
		await _frames(8)
		var names: PackedStringArray = ["play", "YEAR", "scrub", "×1", "×10", "×100", "✕"]
		var layout: PackedStringArray = []
		## **Which control the break strands, read off the laid positions rather
		## than inferred from the widths.** Arithmetic over the printed widths
		## gives the same answer here, but "the break lands after the sixth" is
		## a claim about `FlowContainer`'s packing and it costs one `position.y`
		## to measure instead of reason about.
		var rows: Dictionary = {}
		for i in stand_ins.size():
			var c := stand_ins[i] as Control
			var nm: String = names[i] if i < names.size() else "?"
			layout.append("%s %.0f" % [nm, c.size.x])
			var key := int(round(c.position.y))
			if not rows.has(key):
				rows[key] = PackedStringArray()
			var bucket: PackedStringArray = rows[key]
			bucket.append(nm)
			rows[key] = bucket
		print("  option 1 (HFlowContainer), scrub minimum %.0f px: %d line(s), scrub laid %.0f px (%.1f dp)"
			% [scrub_min, flow.get_line_count(), (stand_ins[2] as Control).size.x,
			   _dp((stand_ins[2] as Control).size.x)])
		print("           widths: %s" % ", ".join(layout))
		var ys: Array = rows.keys()
		ys.sort()
		for y in ys:
			print("           line at y=%d: %s" % [y, ", ".join(rows[y])])
		host.queue_free()
		await _frames(4)

	## -- Option 2 -------------------------------------------------------------
	var deficit: float = _floor - slider.size.x
	if deficit <= 0.0:
		## After the fix the scrub is no longer starved, so the deficit this
		## option existed to close is gone; the pill arithmetic is printed from
		## the pre-fix figure instead of silently reading zero.
		print("  option 2 (shrink the pills): the scrub is %.0f px, already at or over the %.0f px floor,"
			% [slider.size.x, _floor])
		print("           so there is no deficit left to take out of the pills. The figure this option")
		print("           lost on was the pre-fix one -- see the arithmetic block above.")
		return
	var per_pill: float = deficit / maxf(1.0, float(pills.size()))
	var shrunk: float = (pills[0] as Control).size.x - per_pill
	print("  option 2 (shrink the pills): the scrub is %.0f px short of the %.0f px floor;"
		% [deficit, _floor])
	print("           split over %d pills that is %.1f px each -> %.1f px = %.1f dp per pill,"
		% [pills.size(), per_pill, shrunk, _dp(shrunk)])
	print("           against the same %d dp floor. One violation traded for three, and the scrub"
		% DccTheme.PHONE_TAP_MIN)
	print("           reaches the floor exactly -- %.1f dp of track, not a length." % _dp(_floor))

## **The TRANSPORT row's separation, and not merely the first box's.** The strip
## is a column whose first `BoxContainer` is that column, so a `BoxContainer`
## test picks the 16 px inter-row gap instead of the 26 px inter-control one and
## silently re-measures the option-1 replica with the wrong constant -- it
## reported one line where the real separation gives two. Matched on
## `HBoxContainer` and searched depth-first for that reason.
func _row_separation(n: Node = null) -> int:
	var here: Node = n if n != null else _strip()
	for c in here.get_children():
		if c is HBoxContainer:
			return (c as HBoxContainer).get_theme_constant("separation")
		var deeper := _row_separation(c)
		if deeper > 0:
			return deeper
	return 0

func _collect_targets() -> Array:
	var out: Array = []
	_collect(_strip(), out)
	return out

func _ready() -> void:
	await _frames(2)
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load"); get_tree().quit(1); return
	var args := OS.get_cmdline_user_args()
	var legs := 0
	var vp_size := Vector2i(1080, 2340)
	for a in args:
		match a:
			"--phone":
				## The OnePlus 6T this build is tested on, and the density every
				## figure in `_build_phone_sim_strip()`'s own comments is quoted
				## at: 1080 / 412 = 2.621.
				_leg = "phone"; vp_size = Vector2i(1080, 2340); legs += 1
			"--phone-1440":
				## The second density, because "a defect is phone-only" is a
				## claim about one number until the other one is tallied.
				## 1440 x 3168 is aspect 0.455, still phone; scale 3.495.
				_leg = "phone"; vp_size = Vector2i(1440, 3168); legs += 1
			"--phone-landscape":
				## **The composition the height budget can actually fail in.**
				## Portrait hands `_phone_content_gap` 1 594 px and the strip
				## takes 288 of it; landscape has only 1 080 px of screen to
				## start with, and this row grew by 131 px. A4's containment
				## check is the assertion that could break there, so it is
				## measured rather than reasoned about. 2340 x 1080 is the same
				## 0.462 aspect rotated, so it still classifies phone and keeps
				## `_phone_scale` at 2.621.
				_leg = "phone"; vp_size = Vector2i(2340, 1080); legs += 1
			"--tablet":
				## The control state: this strip is phone-only chrome.
				_leg = "tablet"; vp_size = Vector2i(2560, 1600); legs += 1
			"--force-touch":
				pass
			_:
				print("[FATAL] unrecognised argument '%s' -- this probe reads only" % a,
					" --phone / --phone-1440 / --phone-landscape / --tablet / --force-touch")
				get_tree().quit(1); return
	if legs != 1:
		print("[FATAL] pass exactly one of --phone / --phone-1440 / --phone-landscape / --tablet (got ",
			legs, ")")
		get_tree().quit(1); return
	if not ("--force-touch" in args):
		print("[FATAL] every leg needs --force-touch: _touch can never be true in this",
			" dev environment without it (dcc_shell.gd::_ready)")
		get_tree().quit(1); return

	var vp := SubViewport.new()
	vp.size = vp_size
	vp.gui_embed_subwindows = true
	add_child(vp)
	_vp = vp
	_app = load("res://shell/app.tscn").instantiate()
	vp.add_child(_app)
	await _frames(60)
	if _app.open_project_dialog != null:
		_app.open_project_dialog.hide()
	await _frames(4)

	_dens = _density()
	var want: String = {"phone": "PHONE", "tablet": "TABLET"}[_leg]
	if _dens != want:
		print("[FATAL] --%s booted into %s, not %s -- every floor below would be" % [_leg, _dens, want],
			" judged against the wrong bar")
		get_tree().quit(1); return
	## Derived from the same expression `DccShell._ptap()` uses --
	## `_pscale(maxf(PHONE_TAP_MIN, px))` = `round(44 * phone_scale)` -- so this
	## cannot pass for every value of the constant.
	_floor = round(float(DccTheme.PHONE_TAP_MIN) * DccTheme.phone_scale())
	print("[BOOT] leg=%s  viewport=%dx%d  density=%s  phone_scale=%.3f  floor=%.0f px (%d dp)"
		% [_leg, vp_size.x, vp_size.y, _dens, DccTheme.phone_scale(), _floor,
		   DccTheme.PHONE_TAP_MIN])

	if _dens == "TABLET":
		## The control state. `_build_phone_sim_strip()` is only called from
		## `_build_phone_shell()`, so at tablet density there is nothing to
		## measure -- and saying so is what makes the phone legs' numbers a
		## density finding rather than an accident of which composition booted.
		var strip := _strip()
		print("\n--- tablet control state ---")
		print("  _phone_sim_strip is %s (phone-only chrome; _build_phone_shell() did not run)"
			% ("null, as expected" if strip == null else "NOT null: " + str(strip)))
		if strip != null:
			_fail += 1
		print("\n_simscrub_probe (%s): %s" % [_dens, "clean" if _fail == 0 else str(_fail) + " FAILURE(S)"])
		get_tree().quit(1 if _fail > 0 else 0)
		return

	## The strip is CIVIL chrome and starts hidden; `set_phone_sim_strip_open()`
	## is its own opener and is what the phone timeline row calls.
	_app.select_domain_mode("civilization", "factions")
	await _frames(10)
	var strip := _strip()
	if strip == null:
		print("[FATAL] _phone_sim_strip is null on a PHONE boot -- nothing to measure")
		get_tree().quit(1); return
	_app.call("set_phone_sim_strip_open", true)
	await _frames(14)
	if not strip.visible:
		print("[FATAL] the strip did not open -- every number below would be a hidden node's")
		get_tree().quit(1); return

	_arithmetic("as built, no world")
	_walk_targets("as built, no world")
	await _walk_geometry()
	await _widest_year()
	await _losing_options()

	print("\n_simscrub_probe (%s): %s"
		% [_dens, "clean" if _fail == 0 else str(_fail) + " FAILURE(S)"])
	get_tree().quit(1 if _fail > 0 else 0)
