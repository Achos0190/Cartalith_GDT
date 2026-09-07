extends Node
## **WINDOW-CONFORM**: the three constructions this pass repointed, read back
## from the BUILT control rather than from the factory that built it.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _winconform_probe.tscn
##   ... _winconform_probe.tscn -- --force-touch    # tablet, 1600x1000
##   ... _winconform_probe.tscn -- --light          # the light palette
##
## **The app decides its own density from the viewport it is in**, not from
## `DccTheme.set_touch()`. At the headless default 1152 x 648 the aspect is
## 0.5625, under `DccShell._compute_layout_mode()`'s 0.6 phone threshold, so a
## `--force-touch` run without a frame silently measures the PHONE composition
## -- which is how the first version of `_wincensus_probe.gd` reported a
## conforming tablet slider that was really a phone one. The `SubViewport`
## below is the frame, and `app.is_phone()` is printed so the run says which
## composition it measured.
##
## **Every `want` is a literal or a public shell symbol named at its source,
## never the private expression under test.** `MODAL_CTL` and
## `MODAL_INSET_RADIUS` are `DccWidgets`' own published modal-header figures
## and are cited as the vocabulary being conformed TO, not as the value being
## checked against itself.

var app: Node
var _fail := 0
var _checks := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ok(label: String, got, want) -> void:
	_checks += 1
	var good: bool = str(got) == str(want)
	if not good:
		_fail += 1
	print("%s  %-46s got=%s want=%s" % ["ok  " if good else "FAIL", label, got, want])

func _find(n: Node, cls: String, text: String) -> Control:
	if n.get_class() == cls or (cls == "Button" and n is Button and n.get_class() == "Button"):
		if text == "" or (n is Button and String((n as Button).text) == text):
			return n as Control
	for c in n.get_children():
		var r := _find(c, cls, text)
		if r != null:
			return r
	return null

func _ready() -> void:
	await _frames(2)
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load")
		get_tree().quit(1)
		return
	var argv := OS.get_cmdline_user_args()
	var touch := "--force-touch" in argv
	var dark := not ("--light" in argv)
	DccTheme.set_phone(false)
	DccTheme.set_touch(touch)
	DccTheme.apply_theme(dark)
	print("=== _winconform_probe density=%s theme=%s ===" \
		% ["tablet" if touch else "pointer", "dark" if dark else "light"])

	var vp := SubViewport.new()
	vp.size = Vector2i(1600, 1000) if touch else Vector2i(1920, 1080)
	vp.gui_embed_subwindows = true
	vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(vp)
	app = load("res://shell/app.tscn").instantiate()
	vp.add_child(app)
	await get_tree().create_timer(1.6).timeout
	_ok("frame is not the phone composition", app.call("is_phone"), false)
	_ok("density resolved", DccTheme.is_tablet(), touch)

	# ── 1. open_project_dialog._head_close() ────────────────────────────────
	var opd: Node = app.open_project_dialog
	opd.call("open")
	await _frames(8)
	var close := _find(opd, "Button", "✕") as Button
	_ok("OPD head close found", close != null, true)
	if close != null:
		## `flat` suppresses every stylebox, which is what left this control
		## with no hover ground at all -- the measured "before".
		_ok("OPD close is not flat", close.flat, false)
		for st in ["normal", "hover", "pressed", "disabled"]:
			_ok("OPD close has %-8s box" % st,
				close.has_theme_stylebox_override(st), true)
		for ink in ["font_color", "font_hover_color", "font_pressed_color",
				"font_disabled_color"]:
			_ok("OPD close has %s" % ink, close.has_theme_color_override(ink), true)
		## Conformed to `DccWidgets.modal_card()`'s own closer: a `MODAL_CTL`
		## box, `empty` at rest, a `MODAL_INSET_RADIUS` `line_soft` hover.
		_ok("OPD close min box", close.custom_minimum_size,
			Vector2(DccWidgets.MODAL_CTL, DccWidgets.MODAL_CTL))
		var hov := close.get_theme_stylebox("hover") as StyleBoxFlat
		_ok("OPD close hover ground", hov.bg_color, DccTheme.c("line_soft"))
		_ok("OPD close hover radius", hov.corner_radius_top_left,
			DccWidgets.MODAL_INSET_RADIUS)
		var nor := close.get_theme_stylebox("normal")
		_ok("OPD close rest draws nothing", nor is StyleBoxEmpty, true)
		## **A minimum is not a laid-out size, and this one MOVED.** Measured
		## 34 x 35 before this change and 24 x 24 after, and the 11 px is not
		## an accident either way: `flat = true` suppressed the stylebox but
		## not the stock theme's *content margins*, so the old figure was
		## Godot's default `Button` padding leaking through the very hole this
		## repair closes. 24 is `MODAL_CTL`, the shell's own published header
		## control, and it is what the two sibling windows' `Close ✕` chips
		## already lay out at (69 x 24, measured by `_wincensus_probe.gd`).
		## Pinned as a literal so a later drift fails here rather than passing
		## against whatever the box happens to compute.
		print("     OPD close laid out %.0f x %.0f (before: 34 x 35)"
			% [close.size.x, close.size.y])
		_ok("OPD close laid out at MODAL_CTL", close.size, Vector2(24, 24))
	opd.call("hide")
	await _frames(2)

	# ── 1b. the picker FRAME against the canvas ─────────────────────────────
	## **The one surface in this lane the current canvas actually draws.**
	## `Cartalith DCC Environment.dc.html` has two screens -- `scr:'picker'`
	## and `scr:'app'` -- and no Asset library, Data manager or World data
	## window anywhere in it (`state.scr` is enumerated at line 1275 and
	## branched at 1897; `grep -i "asset library"` returns 0 in the PC and
	## Tablet canvases and 2 in `cartalith-dcc-parts.js`, both menu strings).
	## So this block is the lane's only true canvas conformance, and the other
	## three windows are measured against the shell's vocabulary instead.
	##
	## Literals, `Cartalith DCC Environment.dc.html` lines 29-51, desktop
	## density (`--m1:10px --m2:9px --btnH:28px`, line 25):
	##   frame  `gap:34px;padding:40px`
	##   mark   `gap:8px`, `500 20px` mono `letter-spacing:.34em`
	##   cards  `gap:16px`, card `width:252px`, thumb `height:130px`
	##   acts   `gap:10px`
	opd.call("open_welcome")
	await _frames(8)
	var pick: Control = opd.get("_picker")
	_ok("picker composition built", pick != null, true)
	if pick != null:
		var frame := pick as MarginContainer
		for side in ["left", "top", "right", "bottom"]:
			_ok("picker frame pad %-6s" % side,
				frame.get_theme_constant("margin_" + side), 40)
		var col := frame.get_child(0) as VBoxContainer
		_ok("picker block gap", col.get_theme_constant("separation"), 34)
		var mark := col.get_child(0) as VBoxContainer
		_ok("picker wordmark gap", mark.get_theme_constant("separation"), 8)
		var word := mark.get_child(0) as Label
		_ok("picker wordmark size", word.get_theme_font_size("font_size"), 20)
		var tiles: HFlowContainer = opd.get("_picker_tiles")
		_ok("picker card gap x", tiles.get_theme_constant("h_separation"), 16)
		_ok("picker card gap y", tiles.get_theme_constant("v_separation"), 16)
		## The action row is the block after the cards.
		var acts := col.get_child(2) as HFlowContainer
		_ok("picker action gap", acts.get_theme_constant("h_separation"), 10)
		print("     picker column laid out %.0f x %.0f" % [col.size.x, col.size.y])
	opd.call("hide")
	await _frames(2)

	# ── 2. asset_library _sort_button through _style_option() ───────────────
	var alw: Node = app.asset_library_window
	alw.call("open")
	await _frames(10)
	var sort: OptionButton = alw.get("_sort_button")
	_ok("sort button found", sort != null, true)
	if sort != null:
		_ok("sort has disabled box", sort.has_theme_stylebox_override("disabled"), true)
		_ok("sort has font_disabled_color",
			sort.has_theme_color_override("font_disabled_color"), true)
		## Geometry preserved: the factory pins FS_TINY and this site restores
		## the window bar's own FS_SMALL on the line after.
		_ok("sort font size held at FS_SMALL",
			sort.get_theme_font_size("font_size"), DccTheme.FS_SMALL)
		_ok("sort kept hover ink",
			sort.has_theme_color_override("font_hover_color"), true)
		## The two OptionButtons in this window must now agree on their state
		## set -- that agreement is the whole repair.
		## `_style_option(ob, false)` is the quiet variant -- `line` on the
		## four live states, `line_soft` on `disabled`. Pinned so the accent
		## flag cannot be flipped without failing here.
		var snorm := sort.get_theme_stylebox("normal") as StyleBoxFlat
		_ok("sort rest border token", snorm.border_color, DccTheme.c("line"))
		var sdis := sort.get_theme_stylebox("disabled") as StyleBoxFlat
		_ok("sort disabled border token", sdis.border_color, DccTheme.c("line_soft"))
		_ok("sort rest pad_x", snorm.content_margin_left, 9.0)
		print("     sort laid out %.0f x %.0f (pointer before: 140 x 24)"
			% [sort.size.x, sort.size.y])
	alw.call("hide")
	await _frames(2)

	# ── 3. the slicer's chroma swatch ───────────────────────────────────────
	alw.call("open", "", true)
	await _frames(14)
	var cp: ColorPickerButton = alw.get("_slicer_chroma_color")
	_ok("chroma swatch found", cp != null, true)
	if cp != null:
		for st in ["normal", "hover", "pressed", "disabled"]:
			_ok("chroma has %-8s box" % st, cp.has_theme_stylebox_override(st), true)
		var b := cp.get_theme_stylebox("normal") as StyleBoxFlat
		_ok("chroma border colour", b.border_color, DccTheme.c("line"))
		_ok("chroma border width", b.get_border_width(SIDE_TOP), 1)
		## Pinned because it is a DERIVED figure with no canvas behind it, and
		## because mutating it to 0 and to 9/4 left every other check green --
		## the control's own `custom_minimum_size` is the larger term, so the
		## margins move no pixel of geometry and nothing else could see them.
		_ok("chroma swatch inset x", b.content_margin_left, 2.0)
		_ok("chroma swatch inset y", b.content_margin_top, 2.0)
		## The authored pair, pinned as a literal: the margins must not have
		## grown the control past it.
		_ok("chroma min unchanged", cp.custom_minimum_size, Vector2(38, 22))
		_ok("chroma laid out at its minimum", cp.size, Vector2(38, 22))
		## Reported, not asserted -- `DccWidgets.text_button()` floors only on
		## a phone, so the slicer's own way out is under §13's 44 on a tablet
		## and the fix is in a file this lane does not own.
		var x := _find(alw.get("_slicer"), "Button", "✕") as Button
		if x != null:
			print("     GAP slicer close ✕ laid out %.0f x %.0f (§13 floor 44)"
				% [x.size.x, x.size.y])
	alw.call("hide")
	await _frames(2)

	print("WC done -- %d check(s), %d failure(s)" % [_checks, _fail])
	get_tree().quit(1 if _fail > 0 else 0)
