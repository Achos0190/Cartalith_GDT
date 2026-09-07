extends Node
## PHONE-PREFS verifier -- the phone Preferences screen's chip rows.
##
## **Not `_prefbold_probe.gd`, and the distinction is the whole lane.** That
## probe asked whether a desktop `PopupMenu` can bold ONE item, measured the 25
## `set_item_*` methods, and correctly answered no. It is still in the tree and
## still right. The owner was not on a `PopupMenu`: `_fill_prefs()` expands each
## value group through `_chips()` into `_chip()`, a `Button` that sets its own
## face, so the same request is a one-argument change there. This probe measures
## THAT surface.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _prefchip_probe.tscn \
##       -- --force-touch --nowelcome
##
## Run WINDOWED, never `--headless`: leg C is a pixel count off the real
## framebuffer, and the dummy rasteriser hands back a null texture from
## `get_texture().get_image()` ("texture_2d_get: Parameter t is null").
##
## Four legs, and they are four different claims:
##
##   A  CAPTION -- does a chip group's caption still carry the value the chips
##      below it already light? `menus.gd::_stamp_pref_values()` rewrites each
##      Preferences submenu ROW to `base + PREF_VALUE_SEP + picked`, and
##      `_rest_of()` hands that rewritten text to `_expand_sub()` as the caption.
##      Theme and Units escape because `_fill_prefs()` passes their captions
##      itself. Asserted as: no caption contains `PREF_VALUE_SEP` at all, plus a
##      pin of the whole ordered caption set against `menus.gd`'s own literals.
##
##   B  OVERRIDE -- is the selected chip's font resource actually the medium
##      face, and is every other chip still regular? Read back through
##      `get_theme_font("font")`, which is what the Button resolves at draw
##      time, not the Callable that set it.
##
##   C  DRAWN -- three captures of the same screen rect, one change between
##      each: medium (as shipped) -> regular -> regular with a wrecked font
##      colour, compared as differing-pixel counts. The third is a POSITIVE
##      CONTROL and is why this leg can be believed: a zero between the first
##      two could equally mean "the weight never reaches the glyphs" or "this
##      probe is reading a stale framebuffer", and only the control separates
##      them. It earned its place -- the first two runs reported 0 for BOTH,
##      because the framebuffer was showing `phone_project_picker.gd` and not
##      this screen at all, while every tree assertion above it passed.
##
##   D  SWEEP -- `_unstamped()` lives in `_expand_sub()`, which every screen in
##      the MORE stack reaches, so "prefs is right" is not the claim that needs
##      proving. All ten screens are opened and every caption printed, then
##      checked for emptiness, edge whitespace and a surviving stamp.
##
## **A world is generated first, and legs A and D both need it.** With no world
## `menus.gd::_refresh_undo_budget_menu()` takes its `step <= 0` branch and
## writes `256 MB`; with one it writes `256 MB — 5 steps here`. The stale-stamp
## half of the defect exists only in the second state, and this probe reported
## green on it until it generated.

var app: Node
var pm
var _fail := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _check(name: String, cond: bool, detail: String = "") -> void:
	print("PREF %s  %s%s" % ["ok  " if cond else "FAIL", name,
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

## Every chip group on the built screen: caption, chips, and the selected ones.
##
## Read off the built tree rather than off `_chips()`'s arguments: the caption
## is a `mono_label` and the chips are the `HFlowContainer`'s children, which is
## the shape `_chips()` actually produces.
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
			## The on-chip is the one wearing `accent_ink`; `_chip()` gives
			## every other chip `text_dim`. Read from the drawn colour rather
			## than from a re-derived `is_item_checked()`, so this leg cannot
			## agree with itself.
			if (b as Button).get_theme_color("font_color").is_equal_approx(
					DccTheme.c("accent_ink")):
				on.append(b)
		out.append({"caption": cap, "chips": chips, "on": on})
	return out

## One frame of the live framebuffer, cropped to `rect` and saved.
##
## `RenderingServer.frame_post_draw` rather than a count of `process_frame`s:
## the texture is only guaranteed to hold the frame just drawn at that point,
## and a leg that compares two captures cannot afford to read one of them a
## frame early. Two extra process frames first so the theme-override
## notification has been dispatched and the Button has re-laid-out.
func _shot(rect: Rect2i, path: String) -> Image:
	await _frames(3)
	await RenderingServer.frame_post_draw
	var img := get_viewport().get_texture().get_image()
	img.save_png(path.replace(".png", "_full.png"))
	var crop := img.get_region(rect)
	crop.convert(Image.FORMAT_RGBA8)
	crop.save_png(path)
	return crop

## Pixels where two equally-sized crops disagree.
##
## No assumption about which colour is pill and which is glyph -- the two crops
## are the same rect of the same screen one frame apart, so every differing
## pixel is the one thing that was changed between them.
func _diff(a: Image, b: Image) -> int:
	if a.get_size() != b.get_size():
		return -1
	var n := 0
	for y in a.get_height():
		for x in a.get_width():
			if a.get_pixel(x, y) != b.get_pixel(x, y):
				n += 1
	return n

func _rect_of(ctrl: Control) -> Rect2i:
	var r := ctrl.get_global_rect()
	return Rect2i(int(r.position.x), int(r.position.y), int(r.size.x), int(r.size.y))

func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 240.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("WATCHDOG TIMEOUT")
		get_tree().quit(3))
	wd.start()

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
	## **And the phone picker, which is not that dialog.** Hiding
	## `open_project_dialog` is what every desktop-shaped probe here does and it
	## is not enough on a handset: `app.gd::open_welcome()` routes to
	## `phone_project_picker.gd` instead, a full-screen `Control` that covers the
	## whole window. Leg C read a rect of THAT for two runs -- every tree
	## assertion passed while the framebuffer showed the world browser, which is
	## exactly the "a green probe behind every defect" shape. Caught by making
	## the capture save a full frame and looking at it.
	if app.get("phone_project_picker") != null:
		app.phone_project_picker.hide()
	await _frames(4)

	pm = app.get("_phone_menu")
	if pm == null:
		print("PREF !! no PhoneMenu -- run with -- --force-touch")
		get_tree().quit(1)
		return
	if not app.is_phone():
		print("PREF !! not a handset composition -- nothing measured")
		get_tree().quit(1)
		return

	## A world first, and it is leg D that needs it: the `sim` screen's `Speed`
	## group and `travel`'s `Type` group only carry a checked chip once there is
	## something to run, and a group with no single checked chip never reaches
	## the strip at all. Without this the sweep walks nine screens carrying two
	## chip groups between them, neither of them in the state the fix touches.
	app._run_pipeline()
	var waited := 0
	while app.bridge.generating and waited < 4000:
		await get_tree().process_frame
		waited += 1
	await _frames(8)
	_check("a world exists, so the sweep's chip groups are live", app.bridge.has_world)

	pm.open()
	pm._push_screen("prefs")
	await _frames(6)

	var groups := _groups()
	print("PREF groups=", groups.size(), "  palette=", "dark" if DccTheme.is_dark() else "light")
	_check("the prefs screen draws chip groups at all", groups.size() >= 3,
		"%d" % groups.size())

	# -- A: caption vs its own selected chip -----------------------------------
	for g in groups:
		var cap := String(g["caption"])
		var on: Array = g["on"]
		var labels := PackedStringArray()
		for b in g["chips"]:
			labels.append(String((b as Button).text))
		print("   group cap=[%s]  chips=[%s]  on=%s" % [
			cap, ", ".join(labels),
			String((on[0] as Button).text) if on.size() == 1 else str(on.size())])
		if on.size() != 1:
			continue
		var val := String((on[0] as Button).text)
		## `contains(sep)`, not `contains(val)` and not `ends_with(sep + val)`.
		##
		## The first is too loose -- it reports a defect on `Render quality`,
		## whose AUTHORED label ends in the word its `quality` tier is also
		## called; the caption is right and the test was wrong. The second is
		## too tight, and that is the interesting one: it passed on a caption
		## reading `Undo history   256 MB` above a lit `256 MB — 5 steps here`
		## chip, because the stamp and the chip word the same value differently
		## when the parent row is stamped before the submenu refreshes itself.
		## `PREF_VALUE_SEP` appearing at all is the invariant: `menus.gd` writes
		## those three spaces only to separate a label from a readout, and this
		## screen draws the readout as chips.
		_check("caption [%s] carries no value stamp (chip is [%s])" % [cap, val],
			not cap.contains(DccMenus.PREF_VALUE_SEP))

	# -- A2: the captions are the AUTHORED labels, exactly -----------------------
	#
	# `A` alone only says the value is gone; it cannot see a strip that took the
	# separator's whitespace with it, or one that ate a word. This pins the whole
	# ordered set against `menus.gd`'s own authored literals -- `_todo(p,
	# "Multi-GPU mode", ...)` / `_todo(p, "CPU worker threads", ...)` /
	# `_todo(p, "VRAM budget", ...)` / `_todo(p, "Fallback when VRAM full", ...)`
	# at menus.gd:2629-2640, `add_submenu_item("Render quality", "QualityTiers")`
	# at :2663 and `add_submenu_item("Undo history", "UndoBudget")` at :2799 --
	# plus `_fill_prefs()`'s own two literals. A different source file, so this is
	# not a constant asserted against itself.
	var got := PackedStringArray()
	for g in groups:
		got.append(String(g["caption"]))
	var want_caps := PackedStringArray([
		"Theme", "Units", "Multi-GPU mode", "CPU worker threads", "VRAM budget",
		"Fallback when VRAM full", "Render quality", "Relief exaggeration",
		"Tiled LOD", "Undo history"])
	_check("every caption is exactly its authored label, in order",
		got == want_caps, "got [%s]" % " | ".join(got))

	# -- B: the override resolves to the medium face ---------------------------
	var probe_chip: Button = null
	for g in groups:
		var on: Array = g["on"]
		if on.size() != 1:
			continue
		var b := on[0] as Button
		var f := b.get_theme_font("font")
		var base: Font = (f as FontVariation).base_font if f is FontVariation else null
		_check("selected chip [%s] resolves to the MEDIUM face" % b.text,
			base == DccTheme.FONT_MONO_MED,
			"base=%s" % (base.resource_path if base != null else "<not a FontVariation>"))
		if probe_chip == null:
			probe_chip = b
	for g in groups:
		for b in g["chips"]:
			if (g["on"] as Array).has(b):
				continue
			var f2 := (b as Button).get_theme_font("font")
			var base2: Font = (f2 as FontVariation).base_font if f2 is FontVariation else null
			if base2 != DccTheme.FONT_MONO:
				_check("unselected chip [%s] stays REGULAR" % (b as Button).text, false,
					"base=%s" % (base2.resource_path if base2 != null else "?"))

	# -- C: the drawn proof ----------------------------------------------------
	#
	# Three captures of the SAME rect, in order, with only one thing changed
	# between each: medium (as shipped) -> regular -> regular with a wrecked
	# font colour. Compared as differing-pixel counts between crops, so nothing
	# rests on a guess about which colour the pill is.
	#
	# The third capture is a POSITIVE CONTROL and it is the reason this leg can
	# be believed at all: a zero between the first two could equally mean "the
	# weight never reaches the glyphs" or "this probe is re-reading a stale
	# framebuffer". If the colour change moves pixels and the weight change does
	# not, the capture pipeline is live and the finding is real.
	if probe_chip == null:
		_check("a selected chip exists to measure", false)
	else:
		var rect := _rect_of(probe_chip)
		print("   TREE chip vis=", probe_chip.is_visible_in_tree(),
			" rect=", probe_chip.get_global_rect(),
			" xform=", probe_chip.get_global_transform_with_canvas())
		var anc := probe_chip.get_parent()
		while anc != null:
			if anc is CanvasLayer or anc is SubViewport or anc == app:
				print("   TREE ancestor ", anc.name, " (", anc.get_class(), ")")
			anc = anc.get_parent()
		print("   TREE viewport=", get_viewport(), " size=", get_viewport().get_visible_rect(),
			" chip viewport=", probe_chip.get_viewport(),
			" size=", probe_chip.get_viewport().get_visible_rect())
		var med := await _shot(rect, "user://prefchip_medium.png")
		probe_chip.add_theme_font_override("font", DccTheme.mono(0))
		var reg := await _shot(rect, "user://prefchip_regular.png")
		probe_chip.add_theme_color_override("font_color", Color(0, 1, 0))
		var ctl := await _shot(rect, "user://prefchip_control.png")

		var d_weight := _diff(med, reg)
		var d_colour := _diff(reg, ctl)
		print("   DRAWN chip=[%s] rect=%s  px=%d" % [
			probe_chip.text, rect, rect.size.x * rect.size.y])
		print("   DRAWN medium-vs-regular differing px = %d" % d_weight)
		print("   DRAWN control (regular-vs-green ink) differing px = %d" % d_colour)
		_check("the capture pipeline is live (the control moves pixels)",
			d_colour > 0, "%d" % d_colour)
		_check("the MEDIUM face reaches the drawn glyphs (weight moves pixels)",
			d_weight > 0, "%d" % d_weight)
		print("   saved ", ProjectSettings.globalize_path("user://prefchip_medium.png"))

	# -- D: the sweep ----------------------------------------------------------
	#
	# `_unstamped()` sits in `_expand_sub()`, which every screen in the MORE
	# stack reaches -- so a fix aimed at Preferences runs over all of them, and
	# "prefs is right" is not the claim that needs proving. Every screen with a
	# chip group is opened and every caption printed, so a caption this change
	# ate anywhere is visible rather than inferred.
	#
	# Two assertions, and the second is the one with teeth: no caption may carry
	# a stamp (the fix worked), and every caption must be NON-EMPTY and free of
	# edge whitespace (the fix did not overreach). A strip that fires on a
	# caption that was never stamped is the failure mode this leg exists for.
	print("PREF --- sweep ---")
	for id in ["more", "project", "civ", "data", "sim", "assets", "travel",
			"landmarks", "help", "gestures"]:
		while pm._stack.size() > 1:
			pm._stack.pop_back()
		pm._push_screen(id)
		await _frames(4)
		var caps := PackedStringArray()
		var bad := PackedStringArray()
		for g in _groups():
			var cap := String(g["caption"])
			caps.append(cap)
			if cap == "" or cap != cap.strip_edges():
				bad.append("[" + cap + "]")
			if cap.contains(DccMenus.PREF_VALUE_SEP):
				bad.append("stamped [" + cap + "]")
		print("   %-10s groups=%d  [%s]" % [id, caps.size(), " | ".join(caps)])
		_check("screen '%s' has no empty, padded or stamped caption" % id,
			bad.is_empty(), " ".join(bad))

	print("PREF === %s (%d failing) ===" % ["PASS" if _fail == 0 else "FAIL", _fail])
	get_tree().quit(1 if _fail > 0 else 0)
