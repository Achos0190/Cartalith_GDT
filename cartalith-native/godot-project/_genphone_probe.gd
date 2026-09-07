extends Node
## The phone GENERATE sheet, driven through the viewport's own hit-test.
##
## **Why this probe is shaped this way.** `MISTAKES.md`: *"`--force-touch` on
## the desktop is NOT the phone, and `pressed.emit()` is not a finger"*, and
## *"separate 'it renders', 'it can be operated' and 'it can be FOUND'"*. The
## owner could not reach the generation parameters on a real handset while a
## whole session of probes reported the phone shell healthy. So every act below
## is a `SubViewport.push_input()` mouse press at a rect **read off the live
## scene tree**, and the tab is located by hunting the visible caption
## `GENERATE` rather than by reading `_phone_tab_cells`. Nothing here calls
## `_pick_phone_tab()`, `pressed.emit()` or any builder directly.
##
##   godot --headless --path . _genphone_probe.tscn -- --force-touch --vp 1080x2340 --tag p1080
##   godot --headless --path . _genphone_probe.tscn -- --force-touch --vp 1440x3168 --tag p1440
##   godot --headless --path . _genphone_probe.tscn -- --force-touch --vp 800x1280  --tag tab800
##
## Flags this probe actually reads, grepped from the body below rather than
## assumed (`MISTAKES.md`, "Write a probe's usage header"):
##   `--vp WxH`      SubViewport size in physical px. Default 1080x2340.
##   `--tag NAME`    prefix on every output line. Default `gen`.
##   `--world`       also run a generation first, so the stale/resolved states
##                   and the seed row are exercised against a real world.
##   `--force-touch` NOT read here -- `dcc_shell.gd` reads it out of
##                   `OS.get_cmdline_user_args()`, and without it the shell
##                   boots desktop and every line below is void.
## Any other `--flag` aborts rather than being silently ignored.
##
## Headless is correct here: nothing below reads a pixel. Every measurement is
## `Control.size` / `get_global_rect()`, which the layout pass computes under
## the dummy driver exactly as it does windowed (`MISTAKES.md`'s headless row
## scopes itself to rasterising and timing).

var app: Node
var _vp: SubViewport
var _tag := "gen"
var _fail := 0
## Counted, never asserted in prose. A doc put this probe at "26 checks" while
## it ran 29 at 1080x2340 -- 31 `_check(` sites with two on untaken branches --
## and a count that disagrees with its own output is the first thing a later
## reader distrusts. `MISTAKES.md`, "Write a probe's usage header": the header
## is a claim about the probe's own code, so let the output make it instead.
var _checks := 0

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

func _flag(name: String) -> bool:
	return OS.get_cmdline_user_args().has(name)

func _reject_unknown_args() -> bool:
	var known := ["--force-touch", "--vp", "--tag", "--world"]
	for a in OS.get_cmdline_user_args():
		var s := String(a)
		if s.begins_with("--") and not (s in known):
			_log("ABORT unknown flag %s -- this probe reads only %s" % [s, str(known)])
			return false
	return true

func _dp(px: float) -> float:
	return px / maxf(0.001, float(app.phone_scale()))

# -- Real input, not a synthesised signal --------------------------------------

func _tap(at: Vector2) -> void:
	for down in [true, false]:
		var e := InputEventMouseButton.new()
		e.button_index = MOUSE_BUTTON_LEFT
		e.pressed = down
		e.position = at
		e.global_position = at
		_vp.push_input(e, true)
		await _frames(1)
	await _frames(3)

## Every visible `Control` under `root`, depth-first.
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

## The `Label` whose text is exactly `text`, anywhere on screen. Used to FIND a
## target the way a person does -- by reading it -- rather than by reaching for
## the node that happens to hold it.
func _find_label(text: String) -> Label:
	for c in _visible_controls(_vp):
		if c is Label and String((c as Label).text) == text:
			return c
	return null

func _pressable_ancestor(n: Node) -> Button:
	var p := n
	while p != null:
		if p is Button:
			return p
		p = p.get_parent()
	return null

func _slider_count(root: Node) -> int:
	var n := 0
	for c in _visible_controls(root):
		if c is HSlider:
			n += 1
	return n

## Every piece of text a person can READ on this surface -- `Label.text` and
## `BaseButton.text` both. The first cut of this collected Labels only and
## reported the PIPELINE/SCULPT segment missing; it is drawn as two `Button`s,
## so the probe was wrong and the build was right.
func _labels(root: Node) -> Array:
	var out: Array = []
	for c in _visible_controls(root):
		if c is Label:
			out.append(String((c as Label).text))
		elif c is Button:
			out.append(String((c as Button).text))
	return out

## Scroll `target` into the sheet's own viewport before tapping it. A tap at a
## rect the scroller is CLIPPING lands on whatever is behind the sheet -- which
## is exactly what the first run of this probe did to the `04 Tectonics` header
## (the column is 2 936 px in a 975 px sheet, so group 04 starts below the
## fold) and it reported the header inert when it had simply never been hit.
func _reveal(target: Control) -> void:
	app._phone_gen_scroll.ensure_control_visible(target)
	await _frames(3)

## The row `VBoxContainer` that `_pg_range_field()` builds around one parameter,
## found from the label it draws. Needed because "a `+` on screen" is not "the
## `+` belonging to the parameter under test" -- the first run tapped Planet's
## and then asserted about `tect.plates`.
func _row_of(label_text: String) -> Control:
	var l := _find_label(label_text)
	if l == null:
		return null
	return l.get_parent().get_parent() as Control

func _button_in(root: Node, text: String) -> Button:
	for c in _visible_controls(root):
		if c is Button and String((c as Button).text) == text:
			return c
	return null

func _ready() -> void:
	_tag = _arg("--tag", "gen")
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
	## **The one act in this probe that is NOT a finger, and it must be said.**
	## Everything below reaches its target with a `SubViewport.push_input()`
	## press at a rect read off the live tree. This line does not: it hides the
	## project picker by calling it. **So the picker leg of the tap path is
	## proven on glass only, never here** -- which is fine, and saying so is the
	## point, because the whole reason this probe exists is that it does not
	## fake its way past the touch path.
	##
	## It is not laziness either: a probe tap cannot reach a control inside an
	## embedded `AcceptDialog` sub-window at `content_scale_factor` 2.62 --
	## canvas coordinates, physical coordinates and `get_final_transform()`
	## were all tried and `gui_get_hovered_control()` stays null. Taps into the
	## main viewport route normally; this is specific to the sub-window.
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(4)

	_log("viewport %dx%d  phone=%s  scale=%.3f" %
		[_vp.size.x, _vp.size.y, app.is_phone(), app.phone_scale()])

	if not app.is_phone():
		## Tablet and desktop never build `_build_phone_tool_sheet()` at all,
		## so there is no GENERATE sheet to measure. Say so rather than
		## reporting a vacuous pass -- and assert the panel really is absent,
		## which is the one claim this leg CAN make.
		_check(app._phone_gen_scroll == null,
			"not a phone: the GENERATE sheet is not built (left dock is the surface here)")
		_log("RESULT %s checks=%d fail=%d (non-phone leg)" % [_tag, _checks, _fail])
		get_tree().quit(1 if _fail > 0 else 0)
		return

	if _flag("--world"):
		app._run_pipeline()
		var waited := 0
		while app.bridge.generating and waited < 2400:
			await get_tree().process_frame
			waited += 1
		await _frames(8)
		_log("world generated: has_world=%s (%d frames)" % [app.bridge.has_world, waited])

	# -- 0. The boot state, before anything is touched -------------------------
	#
	# `_phone_tab` starts at `"gen"`, so a phone comes up with GENERATE already
	# lit and no tab press ever happening. On the handset that used to draw the
	# one-line tool-options strip under a lit GENERATE tab, which is the defect
	# the owner reported. Asserted here FIRST, because every check below taps
	# the tab and a tap is exactly what hid it.

	_check(app._phone_gen_scroll != null and app._phone_gen_scroll.visible,
		"at launch, with nothing tapped, the GENERATE column is already up")
	var boot_sliders := _slider_count(app._phone_gen_scroll)
	_check(boot_sliders > 0,
		"and it already carries real parameter sliders: %d" % boot_sliders)

	# -- 1. Can it be FOUND? ---------------------------------------------------

	var caption := _find_label("GENERATE")
	_check(caption != null, "a visible caption reading 'GENERATE' exists at launch")
	if caption == null:
		_log("RESULT %s checks=%d fail=%d" % [_tag, _checks, _fail])
		get_tree().quit(1)
		return
	var tab := _pressable_ancestor(caption)
	_check(tab != null, "that caption sits inside a pressable cell")
	var r: Rect2 = tab.get_global_rect()
	_check(Rect2(Vector2.ZERO, Vector2(_vp.size)).encloses(r),
		"the cell is fully on screen: %s in %s" % [r, _vp.size])
	_check(r.size.x >= 44.0 * app.phone_scale() and r.size.y >= 44.0 * app.phone_scale(),
		"the cell clears 44 dp on BOTH axes: %.1f x %.1f dp" % [_dp(r.size.x), _dp(r.size.y)])

	# -- 2. One tap on it, through the viewport --------------------------------

	var before := _slider_count(app._phone_tool_sheet)
	## ONE tap. The phone boots at `peek` with GENERATE already lit, so
	## `_pick_phone_tab`'s `elif _phone_detent == "peek"` arm is the one that
	## fires and a single press lifts the sheet to `half`. A second press would
	## take the first arm and drop it straight back -- measured, after this
	## probe briefly tapped twice and reported the lift broken.
	await _tap(r.get_center())
	_check(String(app._phone_tab) == "gen", "the tap selected the GENERATE tab")
	_check(String(app.active_domain()) == "world", "and the WORLD domain with it")
	_check(app._phone_gen_scroll != null and app._phone_gen_scroll.visible,
		"the GENERATE column is up")
	_check(app._phone_tool_scroll != null and not app._phone_tool_scroll.visible,
		"and the one-line tool-options strip is not")
	_check(String(app.phone_detent()) == "half",
		"the sheet lifted to `half`: %s" % app.phone_detent())

	var texts := _labels(app._phone_tool_sheet)
	_check(texts.has("PIPELINE") and texts.has("SCULPT"),
		"the mode segment is drawn: %s" % [texts.slice(0, 6)])
	_check(texts.has("SEED"), "the SEED row is drawn")
	## The eight canvas groups plus the two this engine parameterises that the
	## prototype does not -- asserted by NAME against `STAGES`, so a renamed
	## stage fails here rather than silently drawing the wrong caption.
	var missing: Array = []
	for st in app._workspace_panels["world"].STAGES:
		if not texts.has(String(st["name"])):
			missing.append(String(st["name"]))
	_check(missing.is_empty(), "every pipeline stage has a group header; missing=%s" % [missing])

	# -- 3. Are the sliders reachable by tapping a header? ---------------------

	## Not "sliders appeared": since the boot fix they are there from launch, so
	## the discriminating claim is that a tab round trip does not LOSE them.
	var after_tab := _slider_count(app._phone_tool_sheet)
	_check(after_tab > 0 and after_tab >= before,
		"a tab round trip keeps the sliders: %d -> %d" % [before, after_tab])

	var tect := _find_label("Tectonics")
	_check(tect != null, "the '04 Tectonics' header is on screen")
	if tect != null:
		var head := _pressable_ancestor(tect)
		_check(head != null, "that header is pressable")
		await _reveal(head)
		var n0 := _slider_count(app._phone_tool_sheet)
		await _tap(head.get_global_rect().get_center())
		var n1 := _slider_count(app._phone_tool_sheet)
		_check(n1 > n0, "tapping it opened the group: %d -> %d sliders" % [n0, n1])
		var opened := _labels(app._phone_tool_sheet)
		## `Plates` and `Drift` are `tect.plates`/`tect.vel`'s own
		## `param_info.label`, straight off `params.rs`. Asserting the ENGINE's
		## strings -- not ones this probe invented -- is what makes this a test
		## of the wiring rather than of the probe.
		_check(opened.has("Plates") and opened.has("Drift"),
			"and it drew the engine's own labels for that group")

		# -- 4. Does a control WRITE the engine? ---------------------------
		var row := _row_of("Plates")
		_check(row != null, "the Plates row is reachable")
		if row != null:
			await _reveal(row)
			var plus := _button_in(row, "+")
			_check(plus != null, "that row carries a `+` stepper")
			if plus != null:
				var was = app.bridge.param_get("tect.plates")
				await _tap(plus.get_global_rect().get_center())
				var now = app.bridge.param_get("tect.plates")
				_check(now != null and now != was,
					"tapping it moved that parameter: tect.plates %s -> %s" % [was, now])
				var stale: int = app._workspace_panels["world"]._stale_from_stage
				## Stage 04 is `STAGES[3]`, zero-based -- the edit must mark
				## the pipeline stale from there, which is what the group's own
				## `stale` caption and the REGENERATE label both read.
				_check(stale == 3,
					"and marked the pipeline stale from stage 04: _stale_from_stage=%d" % stale)

	# -- 5. Every tappable control, BOTH axes, against the floor ---------------
	#
	# `MISTAKES.md`: *"say TARGET, not height"* -- a height-only walk reported a
	# row green while a 7 px chevron shipped on the width axis.

	var floor_px: float = 44.0 * app.phone_scale()
	var small: Array = []
	for c in _visible_controls(app._phone_gen_scroll):
		if not (c is BaseButton or c is HSlider):
			continue
		var s: Vector2 = (c as Control).size
		if s.x <= 0.0 or s.y <= 0.0:
			continue
		if s.x + 0.5 < floor_px or s.y + 0.5 < floor_px:
			var what: String = String((c as Button).text) if c is Button else c.get_class()
			small.append("%s %.0fx%.0f (%.1fx%.1f dp)" % [what, s.x, s.y, _dp(s.x), _dp(s.y)])
	_check(small.is_empty(), "every tappable control clears 44 dp on both axes; under=%s" % [small])

	# -- 6. Is the sheet actually full? ---------------------------------------
	#
	# The owner's complaint was "a lot of white space". A column that is taller
	# than the sheet is a column with content in it; one that is shorter is the
	# defect being fixed.

	var col_h: float = app._phone_gen_col.get_combined_minimum_size().y
	var sheet_h: float = (app._phone_tool_sheet as Control).size.y
	_check(col_h > sheet_h,
		"the GENERATE column overflows its sheet (so it scrolls, not pads): %.0f vs %.0f px"
			% [col_h, sheet_h])
	## And the scroller can reveal that overflow -- `MISTAKES.md`'s
	## "read a layout that overflows the screen" row: a DISABLED axis folds the
	## child minimum into the parent with no scrollbar to show it.
	_check(app._phone_gen_scroll.vertical_scroll_mode == ScrollContainer.SCROLL_MODE_AUTO,
		"the column's vertical scroll is AUTO, not DISABLED")
	_check(app._phone_gen_col.get_combined_minimum_size().x <= float(_vp.size.x),
		"and it does not force the sheet wider than the screen: %.0f vs %d"
			% [app._phone_gen_col.get_combined_minimum_size().x, _vp.size.x])

	# -- 7. Re-tap collapses, as the canvas's hTab says -----------------------

	await _tap(r.get_center())
	_check(String(app.phone_detent()) == "peek",
		"re-tapping the lit tab drops it to peek: %s" % app.phone_detent())

	# -- 8. Can the list be SCROLLED by a finger? -----------------------------
	#
	# Measured on the handset first and only then written here: a drag that
	# begins on a group header used to move nothing, because a `Button` at the
	# default `MOUSE_FILTER_STOP` swallows the drag and the `ScrollContainer`
	# never sees it -- on a list that is mostly header rows, that leaves the
	# ~14 dp gutter as the only place a scroll can start. The fix is
	# `MOUSE_FILTER_PASS` on every button in the column, and this is the
	# assertion that keeps it: **not** a call to `scroll_vertical`, which is
	# what a programmatic reveal does and what could not have caught it.

	## Buttons `PASS`, sliders `STOP`, everything else `IGNORE`. Asserted over
	## the WHOLE subtree and not just the buttons, because the first fix did
	## only the buttons and changed nothing on the handset -- the group boxes
	## and their padding were the blocker.
	var wrong: Array = []
	var seen := 0
	for c in _visible_controls(app._phone_gen_scroll):
		if c == app._phone_gen_col or c is ScrollContainer:
			continue
		seen += 1
		var want: int = Control.MOUSE_FILTER_STOP if c is Range 			else (Control.MOUSE_FILTER_PASS if c is BaseButton else Control.MOUSE_FILTER_IGNORE)
		if (c as Control).mouse_filter != want:
			wrong.append("%s=%d want %d" % [c.get_class(), (c as Control).mouse_filter, want])
	_check(seen > 0 and wrong.is_empty(),
		"all %d controls in the column let the drag reach the scroller; wrong=%s"
			% [seen, wrong.slice(0, 6)])

	_log("RESULT %s checks=%d fail=%d" % [_tag, _checks, _fail])
	get_tree().quit(1 if _fail > 0 else 0)
