extends Node
## Proves the new "pan_drag_hint" coach mark (Ruling BO / `OUTSTANDING_WORK.md`,
## "a drag on the map does nothing until the hand tool is armed, with no
## on-screen cue") actually renders on a fresh phone launch, not just that the
## id exists in `_COACH_MARKS`, AND that it (and the two pre-existing marks
## that share its `_show_phone_toast()` primitive) is actually legible --
## opaque, and not visually overlapped by anything drawn above it.
##
## Before this pass: `git show HEAD:.../dcc_shell.gd | grep pan_drag_hint`
## returns nothing -- the id, the toast text and this whole coach mark did not
## exist. After: `_coach_mark_ids()` names it, a toast carrying its exact text
## appears in `_phone_root`, and `_coach_mark_seen("pan_drag_hint")` flips to
## true once it has been shown -- the same persistence the other two marks
## already use, reused rather than reinvented.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _pandraghint_probe.tscn -- --force-touch
##
## Windowed on purpose, not --headless: this is a claim about what actually
## paints (MISTAKES.md "Assert on pixels" / "Claim something covered is now
## visible" -- reasoning from the scene tree is not evidence a Label is really
## drawn, but Godot's UI tree WILL contain the Label node either way, and that
## is the weaker, still-useful thing a tree walk checks -- presence of the
## right node with the right text, not a pixel diff. The pixel checks below
## are what actually verify legibility).
##
## **Root cause of the first reported screenshot ("faint text, a map label
## drawn on top")**: this probe's own first version screenshotted the instant
## `_find_label_with_text()` found the Label node -- which can be the very
## first frame the toast exists, while `_show_phone_toast()`'s own fade-in
## tween (`modulate:a`, 0 -> 1.0 over 0.18s) is still mid-flight. At partial
## alpha the WHOLE toast (background AND text) blends toward transparent, so
## the map underneath -- including a place-name label at that screen position
## -- shows through and the text loses contrast. Investigated rather than
## assumed: `DccTheme.panel("raised")`'s `StyleBoxFlat.bg_color` carries no
## alpha channel of its own (light theme `#fbfaf7`, dark `#17191a`, both fully
## opaque), and `text_bright` against `raised` is `#111210` on `#fbfaf7` in
## light / `#e8ebec` on `#17191a` in dark -- both comfortably over 4.5:1 at
## rest. There is no code defect in `_show_phone_toast()`'s style; the bug was
## this probe capturing before the tween settled. Fixed below by waiting for
## `wrap.modulate.a` to reach 1.0 before sampling or saving anything, and by
## verifying that wait empirically (not assuming a fixed delay is enough).
##
## Protects: a future edit to `_COACH_MARKS` (a typo in the id, a copy-paste
## that reuses "sheet_handle"'s anchor for this entry, or a text edit that
## drops the ✋ glyph) from silently breaking the one on-screen cue this
## defect has -- AND a future edit to `_show_phone_toast()`'s style (an actual
## alpha added to the panel, a text colour swapped for a low-contrast one, a
## z_index removed) from silently making any of the three toasts illegible
## again, for any of the three marks, without a screenshot being read to
## notice it.

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _log(s: String) -> void:
	print("[pandraghint] ", s)

var _fail := 0

func _check(ok: bool, what: String) -> void:
	if not ok:
		_fail += 1
	_log("  %-4s %s" % ["OK" if ok else "FAIL", what])

const HINT_TEXT := "A one-finger drag only moves the map while Pan (✋) is armed."
const BOTTOMBAR_TEXT := "MAP · GENERATE · PLAN switch tasks here — MORE reaches everything else."
const SHEETHANDLE_TEXT := "Drag this handle to expand tool options."

## `is_visible_in_tree()`, not just a text match on the node -- MISTAKES.md
## "Claim something covered is now visible": a tree walk finds a Label
## whether or not anything is actually painting it, and this probe's first
## run found exactly that false positive (the project-picker dialog was still
## up, painted OVER the toast, and the check passed anyway because the Label
## existed underneath it).
func _find_label_with_text(root: Node, needle: String) -> Label:
	if root is Label and String((root as Label).text) == needle \
			and (root as Label).is_visible_in_tree():
		return root as Label
	for c in root.get_children():
		var found := _find_label_with_text(c, needle)
		if found != null:
			return found
	return null

## The toast's own outer wrap: `_show_phone_toast()` builds
## `PanelContainer -> Label`, so the Label's direct parent is the node whose
## `modulate.a` drives the fade and whose `get_global_rect()` is the whole
## toast's footprint (background included), not just the text's.
func _toast_wrap(label: Label) -> Control:
	return label.get_parent() as Control

## sRGB relative luminance (WCAG 2.1 formula) -- the same definition the
## 4.5:1 contrast ratio requirement is defined in terms of, so this must match
## it exactly rather than use a cheaper approximation (plain average, or
## `Color.get_luminance()`, which is a *different*, non-WCAG formula Godot
## exposes for tonemapping and would silently check the wrong ratio).
func _srgb_channel(v: float) -> float:
	return v / 12.92 if v <= 0.03928 else pow((v + 0.055) / 1.055, 2.4)

func _rel_luminance(c: Color) -> float:
	return 0.2126 * _srgb_channel(c.r) + 0.7152 * _srgb_channel(c.g) \
		+ 0.0722 * _srgb_channel(c.b)

func _contrast_ratio(a: Color, b: Color) -> float:
	var la := _rel_luminance(a)
	var lb := _rel_luminance(b)
	var hi := maxf(la, lb)
	var lo := minf(la, lb)
	return (hi + 0.05) / (lo + 0.05)

## Waits (bounded) for `wrap.modulate.a` to reach full opacity -- the fade-in
## tween's own target -- polling rather than a fixed sleep, since a slow frame
## could stretch the tween's real wall-clock duration past its nominal 0.18s.
func _await_full_opacity(wrap: Control, max_frames: int = 30) -> bool:
	for i in max_frames:
		if not is_instance_valid(wrap):
			return false
		if wrap.modulate.a >= 0.99:
			return true
		await _frames(1)
	return is_instance_valid(wrap) and wrap.modulate.a >= 0.99

## Checks one already-fully-opaque, already-screenshotted toast: samples the
## rendered frame (not theme tokens -- MISTAKES.md "Write an oracle" reasoning
## applies here too: the CLAIM is what actually painted, not what the style
## sheet says should have) at (a) a background corner of the toast's own rect,
## clear of the label's own bounding box, and (b) the darkest pixel inside the
## label's rect (the glyph ink, wherever a scan finds the most contrast-
## bearing point, rather than assuming the exact centre lands on a stroke).
## Protects: a background sample that does not resemble a solid theme colour
## means something else painted over (or under, showing through) the panel at
## that pixel -- the empirical replacement for reasoning about z-order/tree
## order from source, which MISTAKES.md's own "reasoning from the scene graph
## proves nothing under an opaque overlay" row rules out as sufficient by
## itself.
func _check_toast_legible(img: Image, wrap: Control, label: Label, tag: String) -> void:
	var wr := wrap.get_global_rect()
	var lr := label.get_global_rect()
	var iw := img.get_width()
	var ih := img.get_height()

	## Background sample: TOP-CENTRE of the wrap's rect, 4px down from its top
	## edge -- not the corner. First version of this probe sampled 6px in from
	## the top-LEFT corner and that is exactly where it broke: the panel's own
	## `corner_radius_all` (`_pscale(14)`, `_show_phone_toast()`) rounds that
	## corner, so a pixel 6px in from a rounded 14px-radius corner falls
	## OUTSIDE the panel's own fill, in the transparent cutout -- sampling
	## whatever is behind the toast (the map) rather than the toast itself.
	## That is a bug in the SAMPLE POINT, not evidence the map paints over the
	## toast -- the contrast check above, sampling the SAME wrong point,
	## still passed (9.58:1 etc.) only because the map pixel it hit happened
	## to differ enough from the dark text ink, which is exactly the kind of
	## false pass MISTAKES.md warns a control-less pixel assertion produces.
	## Top-centre, in x, is never inside either rounded corner arc as long as
	## the wrap is wider than twice the radius (true for every toast here --
	## `custom_minimum_size.x = _pscale(220)` against a `_pscale(14)` radius),
	## and 4px down is inside the panel's straight top edge, above the
	## label's own top content margin (`_pscale(9)`), so it is never text ink.
	var bx := int(clampf(wr.position.x + wr.size.x * 0.5, 0, iw - 1))
	var by := int(clampf(wr.position.y + 4, 0, ih - 1))
	var bg := img.get_pixel(bx, by)

	## Text sample: the darkest pixel found scanning the label's own rect --
	## the ink, wherever the scan lands on it, rather than an assumed centre.
	var darkest := bg
	var darkest_lum := 2.0
	var steps := 24
	for xi in steps:
		for yi in steps:
			var px := int(clampf(lr.position.x + lr.size.x * (float(xi) / float(steps - 1)), 0, iw - 1))
			var py := int(clampf(lr.position.y + lr.size.y * (float(yi) / float(steps - 1)), 0, ih - 1))
			var p := img.get_pixel(px, py)
			var lum := _rel_luminance(p)
			if lum < darkest_lum:
				darkest_lum = lum
				darkest = p

	var ratio := _contrast_ratio(bg, darkest)
	_log("  %s bg=%s ink=%s contrast=%.2f:1" % [tag, bg, darkest, ratio])
	_check(ratio >= 4.5, "%s text-vs-background contrast is >= 4.5:1 (got %.2f:1)" % [tag, ratio])

	## Background-purity check: the sampled corner should be close to one of
	## the two theme `raised` tokens (light `#fbfaf7`, dark `#17191a`) -- not
	## an exact match (anti-aliasing, the panel's own hairline border at
	## `line`, JPEG-free PNG banding are all fine), but within a tolerance
	## that a MAP pixel (this world's terrain/water palette, verified by eye
	## in `pan_drag_hint.png` to run blue/green/tan, nothing near either
	## `raised` token) or a label's outline stroke bleeding through would blow
	## past. This is the actual "is anything drawn above/through the toast"
	## check the coordinator asked for, done empirically against the rendered
	## frame rather than by reasoning about the scene tree.
	var light_raised := Color("#fbfaf7")
	var dark_raised := Color("#17191a")
	var close_to_theme := bg.is_equal_approx(light_raised) \
		or (absf(bg.r - light_raised.r) < 0.12 and absf(bg.g - light_raised.g) < 0.12 \
			and absf(bg.b - light_raised.b) < 0.12) \
		or (absf(bg.r - dark_raised.r) < 0.12 and absf(bg.g - dark_raised.g) < 0.12 \
			and absf(bg.b - dark_raised.b) < 0.12)
	_check(close_to_theme,
		"%s background sample %s is close to a theme 'raised' token (nothing painted over/through it)" % [tag, bg])

func _ready() -> void:
	var wd := Timer.new()
	wd.wait_time = 60.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("WATCHDOG TIMEOUT")
		get_tree().quit(3))
	wd.start()

	var want := Vector2i(1080, 2340)
	DisplayServer.window_set_size(want)
	get_window().size = want
	get_tree().root.gui_embed_subwindows = true
	await _frames(4)

	## Clear ANY prior "seen" state for the three coach marks so this run sees
	## the real first-launch sequence, not a config file a prior probe left
	## behind on this machine -- `DccSettings.CONFIG_PATH` is `user://`, which
	## persists across runs.
	var app: Node = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.6).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(4)

	if not app.is_phone():
		_log("ABORT not phone -- pass --force-touch")
		get_tree().quit(2)
		return

	_check(app._coach_mark_ids().has("pan_drag_hint"),
		"_COACH_MARKS names the new pan_drag_hint id")

	## Generate a world and let `phone_project_picker.gd` self-dismiss on
	## `generation_finished` BEFORE triggering the coach-mark sequence --
	## deliberately NOT the natural cold-boot order (`_maybe_show_coach_marks()`
	## already fired once, unconditionally, back when `add_child(app)` built the
	## phone shell, well before any world existed).
	##
	## **Found by this probe's own earlier run**: with the natural order, the
	## FIRST mark (`bottombar_tabs`) fires at ~t0, while `phone_project_picker`
	## is still the only thing on screen (real cold-boot behaviour -- every
	## launch opens on the picker) -- so its toast paints, entirely correctly,
	## UNDER a full-screen modal that was already open before it existed. This
	## is real and pre-existing (this pass changed neither the picker nor
	## `bottombar_tabs`), not a regression from `pan_drag_hint`, and not what
	## the coordinator asked this probe to check -- their ask was the
	## TOAST'S OWN style (opacity/contrast/what draws over it once the toast
	## is the thing actually meant to be on screen), which a picker that
	## legitimately owns the whole screen at boot is a different question from.
	## Re-triggering explicitly, over an already-clean map, isolates that
	## question from this unrelated cold-boot race -- see the note left in
	## this pass's report for the coordinator on the race itself.
	var bridge = app.bridge
	bridge.generate({
		"seed": 483920, "width_km": 1200.0, "grid_w": 512, "grid_h": 384,
		"archetype": "", "villages": true, "sea_level": 0.42,
	})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(0.6).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	if app.phone_project_picker != null:
		app.phone_project_picker.hide()
	await _frames(2)
	_check((app.open_project_dialog == null or not app.open_project_dialog.visible)
			and (app.phone_project_picker == null or not app.phone_project_picker.visible),
		"no project-picker dialog is left covering the shell after generation")

	## Clear "seen" state and re-run the sequence NOW, over the clean map --
	## `_show_next_coach_mark()`, `_coach_mark_seen()`/`_set_coach_mark_seen()`
	## are `dcc_shell.gd`'s own reflectable methods (`_coach_mark_ids()`'s own
	## doc comment: a `const` is not reflectable but a `func` is regardless of
	## the underscore convention), so this is the same call
	## `_maybe_show_coach_marks()` itself makes, not a re-implementation.
	var settings_script = load("res://shell/dcc_settings.gd")
	var cfg := ConfigFile.new()
	cfg.load(settings_script.CONFIG_PATH)
	if cfg.has_section("coach_marks"):
		cfg.erase_section("coach_marks")
	cfg.save(settings_script.CONFIG_PATH)
	app.call("_show_next_coach_mark", 0)

	## Polls from this restart (t=0'), the same clock the coach-mark sequence
	## runs on, checking for ALL THREE marks' text every frame -- so whichever
	## one is currently on screen gets caught and verified the moment its own
	## fade-in finishes, regardless of exact timing. `caught` tracks which ids
	## have already been handled so a mark is not screenshotted/checked twice
	## if it happens to still be findable a frame later.
	var caught := {}
	var out_dir := "C:/Users/Vincent/AppData/Local/Temp/claude/C--Users-Vincent-Cartalith-GDT/00ffe296-8aa1-4e76-b7ec-e81351b1a7b7/scratchpad/uidesign/"
	var wanted := [
		{"tag": "bottombar_tabs", "text": BOTTOMBAR_TEXT, "file": "coach_bottombar_tabs.png"},
		{"tag": "sheet_handle", "text": SHEETHANDLE_TEXT, "file": "coach_sheet_handle.png"},
		{"tag": "pan_drag_hint", "text": HINT_TEXT, "file": "pan_drag_hint.png"},
	]
	var seen_toast := false   ## specifically pan_drag_hint, for the existing checks below
	for i in 200:   ## ~20s at 3-frame steps, comfortably past t+7.2s+2.8s
		for w in wanted:
			var tag: String = w["tag"]
			if caught.has(tag):
				continue
			var lbl := _find_label_with_text(app._phone_root, String(w["text"]))
			if lbl == null:
				continue
			var wrap := _toast_wrap(lbl)
			if wrap == null:
				continue
			## Capture at FULL opacity, not mid-fade -- the coordinator's own
			## instruction, and the fix for the root cause this file's header
			## documents. `_await_full_opacity()` polls rather than assuming a
			## fixed delay is enough.
			var opaque := await _await_full_opacity(wrap)
			if not is_instance_valid(lbl) or not is_instance_valid(wrap):
				continue   ## faded/freed while we waited -- try again next pass
			_check(opaque, "%s toast reached full opacity (modulate.a=1.0) before capture" % tag)
			await _frames(1)
			var img := get_viewport().get_texture().get_image()
			img.save_png(out_dir + String(w["file"]))
			_log("saved " + out_dir + String(w["file"]))
			_check_toast_legible(img, wrap, lbl, tag)
			caught[tag] = true
			if tag == "pan_drag_hint":
				seen_toast = true
		if caught.size() >= wanted.size():
			break
		await _frames(3)

	for w in wanted:
		_check(caught.has(String(w["tag"])), "%s toast was seen and verified" % String(w["tag"]))

	## Give the toast's own tween (seconds + fade) time to finish, then confirm
	## it persisted as seen -- proves `_set_coach_mark_seen()` ran, not just
	## that a Label matching the string happened to exist somewhere.
	await get_tree().create_timer(4.0).timeout
	_check(app._coach_mark_seen("pan_drag_hint"),
		"pan_drag_hint is persisted as seen after its toast finished")

	_log("RESULT %s failures=%d" % ["PASS" if _fail == 0 else "FAIL", _fail])
	get_tree().quit(1 if _fail > 0 else 0)
