extends Node
## CHIP-INK verifier -- `LARGE_ITEM_RULINGS.md` ruling B, the selected phone
## chip's ink, and the layout invariant that ruling exists to protect.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _chipink_probe.tscn \
##       -- --force-touch --nowelcome
##
## **Run WINDOWED, never `--headless`.** Legs C and D read the real
## framebuffer; the dummy rasteriser hands back a null texture from
## `get_texture().get_image()` and every pixel comparison would pass vacuously.
## The probe ABORTS rather than reporting a pass if it finds itself headless.
##
## **This probe was written against the state BEFORE the change and must fail
## there.** On the shipped tree the selected chip is a filled `--acc` slab with
## `accent_ink` type, so legs B and C both go red; that is what gives them
## authority once they turn green. Leg D (advance) passes in both states BY
## DESIGN -- it is the regression guard, not the proof of the change, and its
## job is to catch a future edit that buys ink with layout.
##
## ## What it measures, and why each leg is a different claim
##
##   A  REACH -- the prefs screen draws chip groups, and the chip being measured
##      is actually PAINTED: its drawn rect is intersected with every clipping
##      ancestor and with the Window's client rect, because `get_global_rect()`
##      is unclipped and a control scrolled out of a ScrollContainer reports a
##      full-size rect from inside a viewport that never draws it.
##
##   B  TREE INK -- the selected chip's `font_color`, read back through
##      `get_theme_color()` (what the Button resolves at draw time), against the
##      **canvas literal**, never against `DccTheme.c("accent")`: a token-vs-
##      token check moves on both sides at once and can see a wrong token but
##      never a wrong value.
##
##   C  DRAWN INK -- the same claim as pixels, because a theme override that is
##      set and never reaches the glyphs is this project's most repeated
##      failure. Three captures of one rect: as shipped, then with the ink
##      forced to pure red (POSITIVE CONTROL), then restored. The measurement is
##      palette-agnostic -- the pill's fill is the crop's modal colour and the
##      ink is the pixel furthest from it in luminance, so nothing here assumes
##      "the ink is the bright one", which is a dark-palette habit.
##
##   D  ADVANCE -- the ruling chose colour over weight because **Medium's
##      advance is identical to Regular**, so a selection change re-flows
##      nothing. Measured three ways on the real chips: the face's own
##      `get_string_size()` for each chip's own text, the chip's combined
##      minimum size with its face flipped under it, and a freshly built
##      selected/unselected pair over one identical string.
##
##   E  FILL -- the pair's second term. An ink is only legible against what is
##      behind it, so the fill is measured too: it must be the accent WASH and
##      not the solid accent slab `GUI_GAP_REGISTER` §48 (DS-02) removed
##      shell-wide, and the contrast ratio of the ruled ink over it is printed
##      for both palettes.
##
## Both palettes, forced AFTER the shell boots (a palette set before
## `app.tscn` is instantiated is inert -- the boot re-applies the saved mode)
## and restored to `light` at the end, which is what this machine boots.

# -- The canvas literals ------------------------------------------------------
#
# `design/mcp-2026-09-07/Cartalith Android.dc.html`, quoted rather than
# referenced by number alone:
#
#   AND:31    (dark root)  `--sur:#0d0e0f; … --acc:#e0a34a; --accInk:#16130c;
#                            --wash:rgba(224,163,74,.14)`
#   AND:1469  (light vars) `--sur:#f4f2ee; … --acc:#a4650f; --accInk:#f7f4ee;
#                            --wash:rgba(164,101,15,.10)`
#   AND:1356  (the MORE screens' own chip helper, which is the surface this
#             probe measures)
#             `const chip=(on)=>({bord:on?'var(--acc)':'var(--hair)',
#                                 col:on?'var(--acc)':'var(--sec)',
#                                 bg:on?'var(--wash)':'transparent'});`
#             consumed at AND:1362 as `opts:(r.opts||[]).map(v=>({v,...chip(v===r.cur)}))`
#
# So the canvas's selected chip is **accent border, accent TYPE, wash fill** --
# the text token differs from the fill token, which is the whole reason the
# ruling's "accent ink" cannot be read as `accent_ink`. `--accInk` is carried
# below only as the value the assertion must reject.
const ACC := {"dark": Color("#e0a34a"), "light": Color("#a4650f")}
const ACC_INK := {"dark": Color("#16130c"), "light": Color("#f7f4ee")}
const SUR := {"dark": Color("#0d0e0f"), "light": Color("#f4f2ee")}

var app: Node
var pm
var _fail := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _check(name: String, cond: bool, detail: String = "") -> void:
	print("CHIPINK %s  %s%s" % ["ok  " if cond else "FAIL", name,
		("  -- " + detail) if detail != "" else ""])
	if not cond:
		_fail += 1

func _all(n: Node, out: Array) -> void:
	for c in n.get_children():
		out.append(c)
		_all(c, out)

func _nodes() -> Array:
	var out: Array = []
	_all(pm._screen_body, out)
	return out

## Every chip group on the built screen: caption, chips, and which are selected.
##
## The selected chip is identified by its FONT RESOURCE (the medium face), not
## by its colour -- the colour is what legs B and C are testing, and an
## identifier that reads the thing under test can only ever agree with itself.
func _groups() -> Array:
	var out: Array = []
	for c in _nodes():
		if not (c is HFlowContainer):
			continue
		var col := (c as HFlowContainer).get_parent()
		if col == null:
			continue
		var cap := ""
		for s in col.get_children():
			if s is Label:
				cap = String((s as Label).text)
				break
		var chips: Array = []
		var on: Array = []
		for b in (c as HFlowContainer).get_children():
			if not (b is Button):
				continue
			chips.append(b)
			var f := (b as Button).get_theme_font("font")
			var base: Font = (f as FontVariation).base_font if f is FontVariation else null
			if base == DccTheme.FONT_MONO_MED:
				on.append(b)
		out.append({"caption": cap, "chips": chips, "on": on})
	return out

# -- Geometry -----------------------------------------------------------------

## The rect a control is actually PAINTED into.
##
## `get_global_rect()` is unclipped: a chip scrolled out of `_screen_scroll`
## still reports its full rect, and so does one inside a Window that pops wider
## than the viewport. Intersected with every clipping ancestor -- a
## `ScrollContainer` clips whatever its `clip_contents` says -- and with the
## Window's own client rect.
func _visible_rect(ctrl: Control) -> Rect2:
	var r := ctrl.get_global_rect()
	var n: Node = ctrl.get_parent()
	while n != null:
		if n is Control:
			var c := n as Control
			if c.clip_contents or c is ScrollContainer:
				r = r.intersection(c.get_global_rect())
		n = n.get_parent()
	var w := ctrl.get_window()
	if w != null:
		r = r.intersection(Rect2(Vector2.ZERO, Vector2(w.size)))
	return r

func _painted_fraction(ctrl: Control) -> float:
	var full := ctrl.get_global_rect()
	var a := full.size.x * full.size.y
	if a <= 0.0:
		return 0.0
	var v := _visible_rect(ctrl)
	return maxf(0.0, v.size.x * v.size.y) / a

# -- Capture ------------------------------------------------------------------

## One frame of the live framebuffer, cropped to `rect`.
##
## `RenderingServer.frame_post_draw` rather than a count of `process_frame`s:
## only at that point is the texture guaranteed to hold the frame just drawn,
## and a leg comparing two captures cannot afford to read one a frame early.
func _shot(rect: Rect2i, path: String) -> Image:
	await _frames(3)
	await RenderingServer.frame_post_draw
	var tex := get_viewport().get_texture()
	if tex == null:
		return null
	var img := tex.get_image()
	if img == null:
		return null
	var crop := img.get_region(rect)
	crop.convert(Image.FORMAT_RGBA8)
	crop.save_png(path)
	return crop

func _diff(a: Image, b: Image) -> int:
	if a == null or b == null or a.get_size() != b.get_size():
		return -1
	var n := 0
	for y in a.get_height():
		for x in a.get_width():
			if a.get_pixel(x, y) != b.get_pixel(x, y):
				n += 1
	return n

## The pill's own fill: the modal colour of a band across the chip's vertical
## middle, inset 4 px so the rounded corners and the 1 px border are outside it.
## Glyphs live in that band too but never dominate it.
func _fill_colour(img: Image) -> Color:
	var h := img.get_height()
	var w := img.get_width()
	var counts := {}
	var best := Color(0, 0, 0, 0)
	var best_n := 0
	for y in range(maxi(0, h / 2 - 2), mini(h, h / 2 + 3)):
		for x in range(4, maxi(5, w - 4)):
			var p := img.get_pixel(x, y)
			var k := p.to_html()
			var n: int = int(counts.get(k, 0)) + 1
			counts[k] = n
			if n > best_n:
				best_n = n
				best = p
	return best

func _luma(c: Color) -> float:
	return 0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b

## The glyph ink: the pixel furthest from the fill in LUMINANCE, **inside the
## pill** rather than inside the crop.
##
## The inset is not cosmetic, and it took two measurements to get right:
##
##   1. A fully rounded pill leaves the crop's four corners showing bare
##      `--sur`. On the pre-change tree -- a solid `--acc` slab -- those corners
##      are further from the fill in luminance than the type is, so an un-inset
##      search returned the GROUND and the positive control read `f4f2ee`
##      instead of the red it had just been forced to.
##   2. A 6 px inset is still not enough, because the rounded edge cuts inward:
##      at a quarter-height into a 162x115 chip the border sits at x ~ 7.6. The
##      dark control leg then read `c38f42` -- an antialiased sample of the
##      chip's own **accent border** -- and the leg would have passed on the
##      shipped capture for the wrong reason, since that border is the accent
##      whether or not the ink ever reached a glyph.
##
## So the window is the chip's middle half horizontally and middle 40%
## vertically, which is inside the flat part of the pill on every side and is
## where centred type actually is.
##
## Palette-agnostic on purpose. "Take the brightest pixel" is a dark-palette
## habit -- on light the brightest thing in the crop is the ground.
func _ink_colour(img: Image, fill: Color) -> Color:
	var base := _luma(fill)
	var best := fill
	var best_d := -1.0
	var w := img.get_width()
	var h := img.get_height()
	for y in range(int(h * 0.30), maxi(int(h * 0.30) + 1, int(h * 0.70))):
		for x in range(int(w * 0.25), maxi(int(w * 0.25) + 1, int(w * 0.75))):
			var p := img.get_pixel(x, y)
			var d: float = absf(_luma(p) - base)
			if d > best_d:
				best_d = d
				best = p
	return best

func _dist(a: Color, b: Color) -> float:
	return Vector3(a.r - b.r, a.g - b.g, a.b - b.b).length()

func _rel_lum(c: Color) -> float:
	var ch := [c.r, c.g, c.b]
	var o := []
	for v in ch:
		o.append(v / 12.92 if v <= 0.04045 else pow((v + 0.055) / 1.055, 2.4))
	return 0.2126 * o[0] + 0.7152 * o[1] + 0.0722 * o[2]

func _contrast(a: Color, b: Color) -> float:
	var la := _rel_lum(a)
	var lb := _rel_lum(b)
	return (maxf(la, lb) + 0.05) / (minf(la, lb) + 0.05)

func _over(fg: Color, bg: Color) -> Color:
	var a := fg.a
	return Color(fg.r * a + bg.r * (1.0 - a), fg.g * a + bg.g * (1.0 - a),
		fg.b * a + bg.b * (1.0 - a), 1.0)

# -- Palette ------------------------------------------------------------------

## Forced AFTER the shell boots, through the same two calls
## `menus.gd::_apply_theme_mode()` makes, so the repaint path under test is the
## one a user's Theme tap takes.
func _force_palette(want_dark: bool) -> void:
	if DccTheme.is_dark() == want_dark:
		return
	var was_dark := DccTheme.is_dark()
	DccTheme.apply_theme(want_dark)
	app.rebuild_theme(was_dark)

func _key() -> String:
	return "dark" if DccTheme.is_dark() else "light"

# -- Run ----------------------------------------------------------------------

func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 300.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("WATCHDOG TIMEOUT")
		get_tree().quit(3))
	wd.start()

	if DisplayServer.get_name() == "headless":
		print("CHIPINK !! ABORT -- headless. Legs C/E read the framebuffer and "
			+ "get_image() is null under the dummy driver. Run windowed.")
		get_tree().quit(2)
		return

	var want := Vector2i(1080, 2400)
	DisplayServer.window_set_size(want)
	get_window().size = want
	get_tree().root.gui_embed_subwindows = true
	await _frames(4)

	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.4).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	## The phone picker is a separate full-screen Control and hiding the desktop
	## dialog is not enough on a handset -- a previous probe measured a rect of
	## the world browser for two runs while every tree assertion passed.
	if app.get("phone_project_picker") != null:
		app.phone_project_picker.hide()
	await _frames(4)

	pm = app.get("_phone_menu")
	if pm == null:
		print("CHIPINK !! no PhoneMenu -- run with -- --force-touch")
		get_tree().quit(1)
		return
	if not app.is_phone():
		print("CHIPINK !! not a handset composition -- nothing measured")
		get_tree().quit(1)
		return

	for want_dark in [false, true]:
		_force_palette(want_dark)
		await _frames(4)
		if DccTheme.is_dark() != want_dark:
			_check("palette forced to %s" % ("dark" if want_dark else "light"),
				false, "still %s -- thresholds are palette-bound, refusing" % _key())
			continue
		await _measure()

	await _remap_leg()

	## Restored: this machine boots `light`.
	_force_palette(false)
	await _frames(2)
	print("CHIPINK === %s (%d failing) ===" % ["PASS" if _fail == 0 else "FAIL", _fail])
	get_tree().quit(1 if _fail > 0 else 0)

## F: the REPAINT path, which the two passes above never touch.
##
## Each pass measures chips built fresh in its own palette, because
## `_push_screen()` rebuilds the body. The path a user actually takes is the
## other one: standing on this screen, tapping `Dark` on the Theme row runs
## `menus.gd::_apply_theme_mode()` -> `DccShell.rebuild_theme()` ->
## `_recolor_subtree()` over the chips ALREADY on screen. That walk remaps a
## stylebox colour only if it can find it in the outgoing palette table, so a
## colour written from anything but a token survives the flip unchanged and
## strands the chip wearing the other palette's amber. Nothing above would see
## that.
func _remap_leg() -> void:
	_force_palette(false)
	while pm._stack.size() > 1:
		pm._stack.pop_back()
	pm.open()
	pm._push_screen("prefs")
	await _frames(6)
	var b: Button = null
	for g in _groups():
		if (g["on"] as Array).size() == 1:
			b = (g["on"] as Array)[0]
			break
	if b == null:
		_check("[remap] a selected chip exists to flip under", false)
		return
	var before_col := b.get_theme_color("font_color")
	## The flip, with NO re-render of the screen.
	_force_palette(true)
	await _frames(4)
	if not is_instance_valid(b):
		_check("[remap] the chip survives rebuild_theme()", false,
			"freed -- the shell rebuilds rather than recolours, so this leg "
				+ "cannot apply and the fresh-build legs above are the whole story")
		return
	var col := b.get_theme_color("font_color")
	var sb := b.get_theme_stylebox("normal")
	var d: Color = ACC["dark"]
	print("   [remap] light->dark on the SAME chip [%s]: ink %s -> %s" % [
		b.text, before_col.to_html(false), col.to_html(false)])
	_check("[remap] a live palette flip repaints the chip's ink to dark --acc",
		_dist(col, d) < 0.02, "ink=%s want=%s" % [col.to_html(false), d.to_html(false)])
	if sb is StyleBoxFlat:
		var f := sb as StyleBoxFlat
		_check("[remap] and its wash and border with it",
			_dist(f.bg_color, d) < 0.02 and _dist(f.border_color, d) < 0.02,
			"bg=%s(a=%.3f) border=%s" % [f.bg_color.to_html(false), f.bg_color.a,
				f.border_color.to_html(false)])
	_force_palette(false)
	await _frames(3)

func _measure() -> void:
	var pal := _key()
	var acc: Color = ACC[pal]
	var acc_ink: Color = ACC_INK[pal]
	var sur: Color = SUR[pal]
	print("CHIPINK --- palette=%s  acc=%s  accInk=%s ---" % [pal,
		acc.to_html(false), acc_ink.to_html(false)])

	while pm._stack.size() > 1:
		pm._stack.pop_back()
	pm.open()
	pm._push_screen("prefs")
	await _frames(6)

	var groups := _groups()
	_check("[%s] the prefs screen draws chip groups" % pal, groups.size() >= 3,
		"%d groups" % groups.size())
	if groups.is_empty():
		return

	## The Medium half of ruling B, pinned on its own rather than left implicit
	## in `_groups()`'s identification. Nine of the ten groups are radio sets
	## with exactly one checked item; `Relief exaggeration`'s chips are commands
	## rather than a radio set and correctly carry no selection at all.
	var singles := 0
	var many := PackedStringArray()
	for g in groups:
		var n: int = (g["on"] as Array).size()
		if n == 1:
			singles += 1
		elif n > 1:
			many.append("%s=%d" % [g["caption"], n])
	_check("[%s] every radio chip group lights exactly one MEDIUM-faced chip"
		% pal, singles >= 9 and many.is_empty(),
		"%d of %d groups single%s" % [singles, groups.size(),
			("; multi: " + " ".join(many)) if not many.is_empty() else ""])

	# -- B: the ink, in the tree ----------------------------------------------
	var probe_chip: Button = null
	for g in groups:
		var on: Array = g["on"]
		if on.size() != 1:
			continue
		var b := on[0] as Button
		var col := b.get_theme_color("font_color")
		_check("[%s] selected chip [%s] inks the CANVAS --acc %s" % [
				pal, b.text, acc.to_html(false)],
			_dist(col, acc) < 0.02,
			"drawn-property %s (accInk would be %s)" % [
				col.to_html(false), acc_ink.to_html(false)])
		if probe_chip == null and _painted_fraction(b) > 0.99:
			probe_chip = b
	for g in groups:
		for b in g["chips"]:
			if (g["on"] as Array).has(b):
				continue
			var c2 := (b as Button).get_theme_color("font_color")
			if _dist(c2, acc) < 0.02:
				_check("[%s] unselected chip [%s] does NOT take the accent ink"
					% [pal, (b as Button).text], false, c2.to_html(false))

	# -- A: the chip being measured is painted --------------------------------
	if probe_chip == null:
		_check("[%s] a fully painted selected chip exists to measure" % pal, false,
			"no selected chip is >99%% inside its clipping ancestors")
		return
	var vis := _visible_rect(probe_chip)
	print("   [%s] chip=[%s] unclipped=%s visible=%s painted=%.3f" % [
		pal, probe_chip.text, probe_chip.get_global_rect(), vis,
		_painted_fraction(probe_chip)])
	_check("[%s] chip [%s] is painted inside its container AND the window"
		% [pal, probe_chip.text], _painted_fraction(probe_chip) > 0.99,
		"%.3f" % _painted_fraction(probe_chip))

	# -- C: the ink, as drawn pixels ------------------------------------------
	var rect := Rect2i(int(vis.position.x), int(vis.position.y),
		int(vis.size.x), int(vis.size.y))
	var shot := await _shot(rect, "user://chipink_%s_shipped.png" % pal)
	if shot == null:
		_check("[%s] the framebuffer can be read at all" % pal, false,
			"get_image() returned null -- headless?")
		return
	var fill := _fill_colour(shot)
	var ink := _ink_colour(shot, fill)
	print("   [%s] DRAWN fill=%s  ink=%s  (acc=%s accInk=%s sur=%s)" % [
		pal, fill.to_html(false), ink.to_html(false), acc.to_html(false),
		acc_ink.to_html(false), sur.to_html(false)])

	## The discriminating form: the drawn ink must be NEARER the accent than the
	## reversed ink. An absolute tolerance alone would have to absorb the glyph
	## antialiasing; this comparison does not, and it is exactly the assertion
	## the pre-change tree fails.
	_check("[%s] the DRAWN glyph ink is the accent, not the reversed ink" % pal,
		_dist(ink, acc) < _dist(ink, acc_ink),
		"ink=%s  d(acc)=%.3f  d(accInk)=%.3f" % [ink.to_html(false),
			_dist(ink, acc), _dist(ink, acc_ink)])
	_check("[%s] the DRAWN glyph ink lands on the canvas --acc value" % pal,
		_dist(ink, acc) < 0.12, "ink=%s d=%.3f" % [ink.to_html(false), _dist(ink, acc)])

	# -- E: the pair's second term --------------------------------------------
	_check("[%s] the pill is the accent WASH, not the solid --acc slab" % pal,
		_dist(fill, sur) < _dist(fill, acc),
		"fill=%s  d(sur)=%.3f  d(acc)=%.3f" % [fill.to_html(false),
			_dist(fill, sur), _dist(fill, acc)])
	_check("[%s] the wash is actually painted (the fill is tinted off --sur)"
		% pal, _dist(fill, sur) > 0.004,
		"fill=%s sur=%s d=%.4f" % [fill.to_html(false), sur.to_html(false),
			_dist(fill, sur)])
	print("   [%s] CONTRAST ruled ink on drawn fill = %.2f:1  (ink on bare --sur = %.2f:1)"
		% [pal, _contrast(acc, fill), _contrast(acc, sur)])

	## The stylebox behind it, read back off the Button. `bg_color.rgb` is
	## pinned to the canvas `--acc` triple because `--wash` IS the accent hue at
	## low alpha in both palettes (`rgba(224,163,74,.14)` / `rgba(164,101,15,.10)`
	## at AND:31 / AND:1469); only the alpha weight differs between canvas and
	## shell token, so the alpha is bracketed rather than pinned.
	var sb := probe_chip.get_theme_stylebox("normal")
	if sb is StyleBoxFlat:
		var f := sb as StyleBoxFlat
		_check("[%s] the chip's normal box is washed in the canvas --acc hue" % pal,
			_dist(f.bg_color, acc) < 0.02 and f.bg_color.a > 0.05 and f.bg_color.a < 0.25,
			"bg=%s a=%.3f" % [f.bg_color.to_html(false), f.bg_color.a])
		_check("[%s] the chip's border is the canvas --acc" % pal,
			_dist(f.border_color, acc) < 0.02 and f.get_border_width(SIDE_LEFT) > 0,
			"border=%s w=%d" % [f.border_color.to_html(false),
				f.get_border_width(SIDE_LEFT)])
		## **Which wash weight, asserted against another widget rather than
		## against the token that set it.**
		##
		## No pixel test can adjudicate .09 against .16 from the Android canvas:
		## that canvas declares a single `--wash` (.14 dark / .10 light) and
		## spends it on both the hover class and the segment class, because it
		## has no `--wash2`. The rule that separates them is a shell rule --
		## §7's washed treatment, which `DccWidgets.set_segment_on()` builds for
		## every desktop and tablet segment -- so the invariant with real
		## content is that the phone chip and the desktop segment are ONE
		## treatment. Read off a throwaway segment in another file, so this is
		## not a constant asserted against itself.
		var ref := Button.new()
		add_child(ref)
		DccWidgets.set_segment_on(ref, true)
		var rsb := ref.get_theme_stylebox("normal")
		if rsb is StyleBoxFlat:
			var rf := rsb as StyleBoxFlat
			_check("[%s] the chip's wash is the SEGMENT on-state weight "
				% pal + "DccWidgets paints, not the hover weight",
				is_equal_approx(f.bg_color.a, rf.bg_color.a)
					and _dist(f.bg_color, rf.bg_color) < 0.01,
				"chip a=%.3f %s  vs set_segment_on a=%.3f %s" % [
					f.bg_color.a, f.bg_color.to_html(false),
					rf.bg_color.a, rf.bg_color.to_html(false)])
		ref.queue_free()

	## POSITIVE CONTROL. Without it a green leg above could equally mean "the
	## capture is stale" or "this crop is not the chip".
	var was := probe_chip.get_theme_color("font_color")
	probe_chip.add_theme_color_override("font_color", Color(1, 0, 0))
	var ctl := await _shot(rect, "user://chipink_%s_control.png" % pal)
	var ctl_ink := _ink_colour(ctl, _fill_colour(ctl)) if ctl != null else Color(0, 0, 0)
	var moved := _diff(shot, ctl)
	probe_chip.add_theme_color_override("font_color", was)
	await _frames(2)
	print("   [%s] CONTROL forced-red ink=%s  differing px=%d" % [
		pal, ctl_ink.to_html(false), moved])
	_check("[%s] the capture is LIVE (forcing the ink moves pixels)" % pal,
		moved > 0, "%d px" % moved)
	_check("[%s] the measured pixel really is the glyph ink (control reads red)"
		% pal, _dist(ctl_ink, Color(1, 0, 0)) < _dist(ctl_ink, acc),
		"control ink=%s" % ctl_ink.to_html(false))

	# -- D: the advance did not move ------------------------------------------
	#
	# The ruling's own reason. Three independent measurements, because each can
	# fail without the others: the FACE's advance, the laid CHIP's minimum, and
	# a selected/unselected pair built over one identical string.
	var f_med := DccTheme.mono(0, true)
	var f_reg := DccTheme.mono(0, false)
	var face_bad := PackedStringArray()
	var chip_bad := PackedStringArray()
	var n_chips := 0
	for g in groups:
		for node in g["chips"]:
			var b := node as Button
			n_chips += 1
			var fs := b.get_theme_font_size("font_size")
			var wm := f_med.get_string_size(b.text, HORIZONTAL_ALIGNMENT_LEFT, -1, fs).x
			var wr := f_reg.get_string_size(b.text, HORIZONTAL_ALIGNMENT_LEFT, -1, fs).x
			if not is_equal_approx(wm, wr):
				face_bad.append("[%s] med=%.2f reg=%.2f" % [b.text, wm, wr])
			## The same chip with the other face under it. Restored immediately.
			var keep := b.get_theme_font("font")
			var w0 := b.get_combined_minimum_size().x
			b.add_theme_font_override("font",
				f_reg if (g["on"] as Array).has(b) else f_med)
			var w1 := b.get_combined_minimum_size().x
			b.add_theme_font_override("font", keep)
			if not is_equal_approx(w0, w1):
				chip_bad.append("[%s] %.2f -> %.2f" % [b.text, w0, w1])
	_check("[%s] Medium's advance equals Regular's for all %d chip labels"
		% [pal, n_chips], face_bad.is_empty(), " ".join(face_bad))
	_check("[%s] flipping a chip's face moves its minimum width by 0 px" % pal,
		chip_bad.is_empty(), " ".join(chip_bad))

	## And the whole treatment, not just the face: two chips built through the
	## real factory over one identical string, one selected and one not.
	var host := HFlowContainer.new()
	pm._screen_body.add_child(host)
	var noop := func(): pass
	var a_on: Button = pm._chip("SAMPLE", true, noop)
	var a_off: Button = pm._chip("SAMPLE", false, noop)
	host.add_child(a_on)
	host.add_child(a_off)
	await _frames(3)
	print("   [%s] PAIR selected min=%s size=%s  unselected min=%s size=%s" % [
		pal, a_on.get_combined_minimum_size(), a_on.size,
		a_off.get_combined_minimum_size(), a_off.size])
	_check("[%s] a selected chip costs the same width as an unselected one" % pal,
		is_equal_approx(a_on.get_combined_minimum_size().x,
			a_off.get_combined_minimum_size().x),
		"%.2f vs %.2f" % [a_on.get_combined_minimum_size().x,
			a_off.get_combined_minimum_size().x])
	_check("[%s] and the same height" % pal,
		is_equal_approx(a_on.get_combined_minimum_size().y,
			a_off.get_combined_minimum_size().y),
		"%.2f vs %.2f" % [a_on.get_combined_minimum_size().y,
			a_off.get_combined_minimum_size().y])
	host.queue_free()
	await _frames(2)
