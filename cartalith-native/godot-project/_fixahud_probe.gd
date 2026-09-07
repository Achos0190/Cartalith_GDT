extends Node
## FIX-A, 2026-09-07. The viewport HUD against `Cartalith DCC Environment
## .dc.html` (`ENV:897-917`) and `Cartalith Tablet.dc.html` (`TAB:449-452`).
##
##   godot --path . _fixahud_probe.tscn -- --vp 1920x1080 --tag pc
##   godot --path . _fixahud_probe.tscn -- --vp 2560x1600 --tag tab --force-touch
##
## **Windowed, not headless, when `--shot` is passed** -- `MISTAKES.md`: the
## dummy driver never fires `frame_post_draw`, so `get_image()` is null and every
## pixel claim passes vacuously. The size and text assertions below are pure
## layout and run correctly either way; the probe says which it took.
##
## Hosted in a `SubViewport` for the same reason `_ph412_probe.gd` is: a
## `--resolution` above this machine's 1680x1050 work area is clamped, and three
## of the frames under test are wider than that.

var app: Node
var _vp: SubViewport
var _tag := "pc"
var _fail := 0
var _pass := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _arg(name: String, dflt: String) -> String:
	var a := OS.get_cmdline_user_args()
	var i := a.find(name)
	return String(a[i + 1]) if i >= 0 and i + 1 < a.size() else dflt

func _has(name: String) -> bool:
	return OS.get_cmdline_user_args().has(name)

func _ok(what: String, got, want) -> void:
	var good: bool = str(got) == str(want)
	if good:
		_pass += 1
	else:
		_fail += 1
	print("  %-4s %-46s got=%s want=%s" % ["ok" if good else "FAIL", what, str(got), str(want)])

func _ready() -> void:
	var parts: PackedStringArray = _arg("--vp", "1920x1080").split("x")
	_tag = _arg("--tag", "pc")
	_vp = SubViewport.new()
	_vp.size = Vector2i(int(parts[0]), int(parts[1]))
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	if _has("--force-touch"):
		Input.set_emulate_touch_from_mouse(true)
	app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await get_tree().create_timer(1.4).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	if "phone_project_picker" in app and app.phone_project_picker != null:
		app.phone_project_picker.hide()
	await _frames(4)

	var bridge = app.bridge
	bridge.generate({
		"seed": 483920, "width_km": 2000.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.45,
	})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.2).timeout
	await _frames(8)

	var vh = app.viewport
	print("=== %s vp=%s touch=%s phone=%s dark=%s ===" % [_tag, str(_vp.size),
		str(DccTheme.is_touch()), str(DccTheme.is_phone()), str(DccTheme.is_dark())])

	# -- A. The Layers button, ENV:900 --------------------------------------
	print("-- A. Layers button (ENV:900 width/height:var(--tool), r8, 1px --hair)")
	var btn: Button = vh._layers_btn
	var want_tool: int = 44 if DccTheme.is_touch() else 30
	_ok("--tool -> button min size", str(btn.custom_minimum_size), str(Vector2(want_tool, want_tool)))
	_ok("laid size", str(btn.size), str(Vector2(want_tool, want_tool)))
	_ok("flat is off so the box is drawn at all", btn.flat, false)
	if _has("--diag"):
		for nm in ["normal", "normal_mirrored", "hover", "hover_mirrored",
				"pressed", "pressed_mirrored", "hover_pressed",
				"hover_pressed_mirrored", "disabled", "disabled_mirrored",
				"focus"]:
			var d: StyleBox = btn.get_theme_stylebox(nm)
			print("     diag %-24s min=%s over=%s" % [nm,
				str(d.get_minimum_size()) if d != null else "nil",
				str(btn.has_theme_stylebox_override(nm))])
		print("     diag offs L%.1f T%.1f R%.1f B%.1f parent=%s anchors=%.1f/%.1f" % [
			btn.offset_left, btn.offset_top, btn.offset_right, btn.offset_bottom,
			btn.get_parent().get_class(), btn.anchor_left, btn.anchor_right])
		print("     diag icon=%s combined=%s h_sep=%d" % [
			str(btn.icon.get_size()), str(btn.get_combined_minimum_size()),
			btn.get_theme_constant("h_separation")])
	var sb: StyleBox = btn.get_theme_stylebox("normal")
	_ok("normal box is a StyleBoxFlat", sb is StyleBoxFlat, true)
	if sb is StyleBoxFlat:
		var f := sb as StyleBoxFlat
		## Asserted at tablet too, not only at pointer: `_apply_touch_scale()`
		## early-returns at scale 1.0, so the canvas box survives there and only
		## the phone gets the navpad pill instead.
		if not DccTheme.is_phone():
			_ok("border-radius:8px", f.corner_radius_top_left, 8)
			_ok("border:1px", f.border_width_left, 1)
			_ok("border is --hair", f.border_color.to_html(), DccTheme.c("line").to_html())
			_ok("background is --pan", f.bg_color.to_html(), DccTheme.c("panel").to_html())
	## The open state, ENV:1959 `layersBtnBg: layersOpen ? var(--wash2) : var(--pan)`.
	if not DccTheme.is_phone():
		vh.set_layers_open(true)
		var op: StyleBox = btn.get_theme_stylebox("normal")
		_ok("open fill is --wash2", (op as StyleBoxFlat).bg_color.to_html(),
			DccTheme.c("accent_wash_2").to_html())
		vh.set_layers_open(false)
		_ok("closed fill returns to --pan",
			(btn.get_theme_stylebox("normal") as StyleBoxFlat).bg_color.to_html(),
			DccTheme.c("panel").to_html())

	# -- B. The scale bar, ENV:915-916 / TAB:449-451 -------------------------
	print("-- B. scale bar (label over a ruled bar, column gap 3)")
	var rule: Control = vh._scale_rule
	var lab: Label = vh._scale_label
	var want_w: int = 84 if DccTheme.is_touch() else 120
	var want_t: int = 4 if DccTheme.is_touch() else 5
	_ok("rule exists and is visible", rule != null and rule.visible, true)
	_ok("rule width", int(rule.size.x), want_w)
	_ok("rule height (tick)", int(rule.size.y), want_t)
	_ok("label sits above the rule, gap 3",
		int(round(rule.position.y - (lab.position.y + lab.size.y))), 3)
	_ok("label left edge == rule left edge", int(lab.position.x), int(rule.position.x))
	_ok("label no longer says 'across'", lab.text.contains("across"), false)
	_ok("label keeps the per-cell segment", lab.text.contains("/ cell"), true)
	print("     label text = '%s'" % lab.text)
	## The bar's own arithmetic, checked against an independent recomputation
	## rather than against the function that produced it.
	var gs: Vector2i = bridge.grid_size()
	var drawn_w: float = minf(vh.size.x, vh.size.y * float(gs.x) / float(gs.y))
	var indep: float = float(want_w) * (2000.0 / float(gs.x)) \
		/ (drawn_w * maxf(0.01, vh.zoom()) / float(gs.x))
	_ok("bar km recomputed independently (2 dp)",
		"%.2f" % vh._bar_km(), "%.2f" % indep)

	# -- C. The top-right readout, ENV:914 ----------------------------------
	print("-- C. zoom readout (ENV:914 is one line: equirect · zoom · field)")
	var ro: Label = vh._readout_label
	_ok("no '2D · ' prefix", ro.text.begins_with("2D"), false)
	_ok("starts at 'equirect'", ro.text.begins_with("equirect"), true)
	_ok("single line", ro.text.contains("\n"), false)
	print("     readout text = '%s'" % ro.text)

	if _has("--shot"):
		await _frames(4)
		DirAccess.make_dir_recursive_absolute("user://fixahud")
		var img := _vp.get_texture().get_image()
		if img == null:
			print("  SHOT ABORTED -- null image; run windowed, not --headless")
		else:
			var p := "user://fixahud/%s.png" % _tag
			img.save_png(p)
			print("  shot %s" % ProjectSettings.globalize_path(p))

	print("[RESULT] %s pass=%d failures=%d" % [_tag, _pass, _fail])
	get_tree().quit(1 if _fail > 0 else 0)
