extends Node
## The phone's vertical scale bar (owner, 2026-11-17: "Do the scale bar on phone. Maybe vertical instead of
## horizontal?"; `OUTSTANDING_WORK.md` row "Phone scale bar, drawn vertically"). `viewport_host.gd` used to hide the
## rule on the phone and keep only the label; it now draws the rule as a vertical bar down the edge opposite the
## dominant hand's row, label beside it, and falls back to the label-only corner when the band has no room.
##
##   godot --path . _phonescale_probe.tscn -- --vp 1080x2340 --force-touch --density 2.625 [--shot DIR]
##   godot --path . _phonescale_probe.tscn -- --vp 1700x1000 --force-touch --density=1.0      (tablet: horizontal)
##   godot --path . _phonescale_probe.tscn -- --vp 1600x900                                   (desktop: horizontal)
##
## Windowed, not headless (global rects and `--shot` frames need a real renderer). The form is read from
## `DccTheme` and the checks branch on it: phone asserts the vertical form and measures its rect against every other
## piece of chrome across zoom x detent x hand; tablet and desktop assert the horizontal bar is unchanged.
##
## Result: an assertion. Prints `PASS|FAIL  <what>  got= want=`, `FORM ...`, `MEASURE ...` rows (every clearance in
## px), with `--shot DIR` a PNG per case into DIR, then `[RESULT] pass=<n> fail=<n>`. Exit 0 when none failed.
## Clearances are measured against the nodes that draw where the owner named: the Layers button and context chip, the
## readout and coordinate labels, the navpad (zoom row), the tool sheet (and so its handle), the bottom bar, the tool
## strip and a live coach-tip toast.

var app: Node
var _vp: SubViewport
var _pass := 0
var _fail := 0
var _shot_dir := ""


func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame


func _ok(what: String, got: Variant, want: Variant) -> void:
	var good: bool = str(got) == str(want)
	_pass += 1 if good else 0
	_fail += 0 if good else 1
	print("%s  %s  got=%s want=%s" % ["PASS" if good else "FAIL", what, str(got), str(want)])


func _check(what: String, cond: bool) -> void:
	_ok(what, cond, true)


func _grect(c: Control) -> Rect2:
	return c.get_global_rect()


## Gap in px between two rects (negative = overlap depth along the shallower axis, so the test is `>= 0`).
func _clear(a: Rect2, b: Rect2) -> float:
	var dx := maxf(b.position.x - (a.position.x + a.size.x), a.position.x - (b.position.x + b.size.x))
	var dy := maxf(b.position.y - (a.position.y + a.size.y), a.position.y - (b.position.y + b.size.y))
	return maxf(dx, dy)


func _ready() -> void:
	var vp_size := Vector2i(1080, 2340)
	var a := OS.get_cmdline_user_args()
	var i := 0
	while i < a.size():
		if a[i] == "--vp" and i + 1 < a.size():
			var wh := String(a[i + 1]).split("x")
			vp_size = Vector2i(int(wh[0]), int(wh[1]))
			i += 2
		elif a[i] == "--shot" and i + 1 < a.size():
			_shot_dir = String(a[i + 1])
			i += 2
		elif a[i] == "--density" and i + 1 < a.size():
			i += 2   ## read by the shell itself
		else:
			i += 1   ## --force-touch, --density=D: the shell's own
	if _has("--force-touch"):
		Input.set_emulate_touch_from_mouse(true)
	_vp = SubViewport.new()
	_vp.size = vp_size
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await get_tree().create_timer(1.4).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	if "phone_project_picker" in app and app.phone_project_picker != null:
		app.phone_project_picker.hide()
	await _frames(4)
	var bridge = app.bridge
	bridge.generate({"seed": 483920, "width_km": 2000.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.45})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(1.0).timeout
	await _frames(8)
	var form := "phone" if DccTheme.is_phone() else ("tablet" if DccTheme.is_tablet() else "desktop")
	print("FORM %s vp=%s touch=%s" % [form, str(vp_size), str(DccTheme.is_touch())])
	if form == "phone":
		await _phone()
	else:
		await _flat(form)
	print("[RESULT] pass=%d fail=%d" % [_pass, _fail])
	get_tree().quit(1 if _fail > 0 else 0)


func _has(n: String) -> bool:
	return OS.get_cmdline_user_args().has(n)


func _snap(tag: String) -> void:
	if _shot_dir == "":
		return
	await _frames(3)
	DirAccess.make_dir_recursive_absolute(_shot_dir)
	var img := _vp.get_texture().get_image()
	if img != null:
		img.save_png(_shot_dir.path_join("scalebar_%s.png" % tag))


## Desktop and tablet keep today's horizontal bar: wide, tick-high, the label above it, bottom-left.
func _flat(form: String) -> void:
	var vh = app.viewport
	var rule: Control = vh._scale_rule
	var lab: Label = vh._scale_label
	_check("%s: rule visible" % form, rule.visible)
	var fsb: StyleBox = lab.get_theme_stylebox("normal")
	_check("%s: the label keeps no scrim (section 5.4: ink on the map)" % form,
		not (fsb is StyleBoxFlat and (fsb as StyleBoxFlat).bg_color == DccTheme.c("hud_scrim")))
	_check("%s: not the vertical form" % form, not vh._scale_vertical)
	_ok("%s: rule width is w_scale_bar" % form, int(rule.size.x), DccTheme.role_px("w_scale_bar"))
	_ok("%s: rule height is h_scale_tick" % form, int(rule.size.y), DccTheme.role_px("h_scale_tick"))
	_ok("%s: label sits 3 above the rule" % form,
		int(round(rule.position.y - (lab.position.y + lab.size.y))), 3)
	_ok("%s: label left == rule left" % form, int(lab.position.x), int(rule.position.x))
	_check("%s: rule is in the bottom half" % form, rule.position.y > vh.size.y * 0.5)
	await _snap(form)


## The phone: for each zoom x detent x hand, the rule is vertical, sized and labelled by the same `_bar_km()`,
## on the edge opposite the row, and clear of everything else; at `full` it falls back to the label-only corner.
func _phone() -> void:
	var vh = app.viewport
	var shell = app
	if _vp.size.x > _vp.size.y:
		## Landscape swaps the bottom sheet for a right-hand dock and the bottom bar for a left rail, so every
		## clearance below (rule above the sheet's top edge, `full` falling back, the toast near the bottom bar) has a
		## different meaning there. Looked at in frames (`--shot`), not asserted; run portrait for the assertions.
		print("NOTE landscape phone: portrait assertions skipped, one frame per hand written with --shot")
		for hand in ["right", "left"]:
			DccSettings.set_dominant_hand(hand)
			await get_tree().create_timer(0.5).timeout
			await _frames(4)
			await _snap("landscape_%s" % hand)
		DccSettings.set_dominant_hand("right")
		_check("landscape: the rule is vertical or hidden, never a stale horizontal one",
			not vh._scale_rule.visible or vh._scale_rule.size.y > vh._scale_rule.size.x)
		return
	for hand in ["right", "left"]:
		DccSettings.set_dominant_hand(hand)
		for det in ["peek", "half", "full"]:
			shell.set_phone_detent(det)
			await get_tree().create_timer(0.5).timeout
			await _frames(4)
			for zf in [1.0, 4.0, 16.0]:
				vh.reset_view()
				await _frames(2)
				vh.zoom_step(zf)
				await _frames(3)
				var tag := "%s_%s_x%d" % [hand, det, int(zf)]
				await _phone_case(vh, shell, hand, det, tag)
		await _toast_case(vh, shell, hand)
	DccSettings.set_dominant_hand("right")


func _phone_case(vh, shell, hand: String, det: String, tag: String) -> void:
	var rule: Control = vh._scale_rule
	var lab: Label = vh._scale_label
	var bar_w := DccTheme.role_px("w_scale_bar")
	var tick := DccTheme.role_px("h_scale_tick")
	if det == "full":
		## No room under the corner chrome at `full`: label-only corner, rule hidden, nothing overlapping the sheet.
		_check("%s: falls back (rule hidden)" % tag, not rule.visible and not vh._scale_vertical)
		_check("%s: the label is still drawn" % tag, lab.text.contains("km") or lab.text.contains("m "))
		return
	_check("%s: vertical form" % tag, vh._scale_vertical and rule.visible)
	_ok("%s: rule is %d px long" % [tag, bar_w], int(rule.size.y), bar_w)
	_ok("%s: rule is %d px thick (the tick)" % [tag, tick], int(rule.size.x), tick)
	_check("%s: mouse ignored (map keeps every gesture)" % tag,
		rule.mouse_filter == Control.MOUSE_FILTER_IGNORE and lab.mouse_filter == Control.MOUSE_FILTER_IGNORE)
	## Same km as the horizontal form: recomputed independently of `_bar_km()`.
	var gs: Vector2i = shell.bridge.grid_size()
	var drawn_w: float = minf(vh.size.x, vh.size.y * float(gs.x) / float(gs.y))
	var indep: float = float(bar_w) * (2000.0 / float(gs.x)) / (drawn_w * maxf(0.01, vh.zoom()) / float(gs.x))
	_ok("%s: bar km independent (2 dp)" % tag, "%.2f" % vh._bar_km(), "%.2f" % indep)
	_check("%s: label still carries the per-cell segment" % tag, lab.text.contains("/ cell"))
	var sb: StyleBox = lab.get_theme_stylebox("normal")
	_check("%s: the label sits on the HUD scrim (it floats mid-map over any terrain)" % tag,
		sb is StyleBoxFlat and (sb as StyleBoxFlat).bg_color == DccTheme.c("hud_scrim"))
	_ok("%s: ... and takes the scrimmed siblings' ink, text_dim" % tag,
		lab.get_theme_color("font_color").to_html(), DccTheme.c("text_dim").to_html())
	var rr := _grect(rule)
	var lr := _grect(lab)
	var vr := _grect(vh)
	## Edge: opposite the row. Right-handed row is on the right, so the bar is on the left.
	if hand == "right":
		_check("%s: bar on the left edge" % tag, rr.position.x < vr.position.x + vr.size.x * 0.25)
		_check("%s: label right of the rule" % tag, lr.position.x >= rr.end.x)
	else:
		_check("%s: bar on the right edge" % tag, rr.end.x > vr.position.x + vr.size.x * 0.75)
		_check("%s: label left of the rule" % tag, lr.end.x <= rr.position.x)
	_check("%s: rule and label inside the map rect" % tag, vr.encloses(rr) and vr.encloses(lr))
	_check("%s: label is horizontal text, one line, vertically centred on the rule" % tag,
		not lab.text.contains("\n") and absf((lr.position.y + lr.size.y * 0.5) - (rr.position.y + rr.size.y * 0.5)) <= 1.5)
	## Clearances (px; >= 0 means no overlap).
	var others := {
		"layers_btn": vh._layers_btn, "context_chip": vh._vp_context, "readout": vh._readout_label,
		"coords": vh._coords_label, "navpad": vh._navpad,
		"tool_sheet": shell._phone_tool_sheet, "bottom_bar": shell._phone_menu_bar, "tool_strip": shell._tool_strip,
	}
	for k in others:
		var c: Control = others[k]
		if c == null or not is_instance_valid(c) or not c.is_visible_in_tree():
			continue
		for mine in [["rule", rr], ["label", lr]]:
			var gap := _clear(mine[1], _grect(c))
			print("MEASURE %s %s-%s gap=%.1f" % [tag, mine[0], k, gap])
			_check("%s: %s clear of %s" % [tag, mine[0], k], gap >= 0.0)
	## The sheet's handle sits at the sheet's top edge: the bar must end above it.
	var sheet_top: float = _grect(shell._phone_tool_sheet).position.y
	_check("%s: rule ends above the sheet's top edge (handle)" % tag, rr.end.y <= sheet_top)
	await _pixels(vh, hand, tag)
	await _snap(tag)


## The rule is really DRAWN, in the ink the code names, along its whole length: a node can be laid out right and paint
## nothing (a draw call dropped, the wrong colour, a spine on the wrong side). Reads the frame (windowed only: headless
## never presents) at the spine's middle and at the end tick, expecting `text_secondary` (drawn opaque), with a negative
## control a few px into the map beside it that must NOT be that ink.
func _pixels(vh, hand: String, tag: String) -> void:
	await RenderingServer.frame_post_draw
	var img := _vp.get_texture().get_image()
	if img == null:
		_check("%s: a frame to read (run windowed)" % tag, false)
		return
	var rr := _grect(vh._scale_rule)
	var ink: Color = DccTheme.c("text_secondary")
	var on_right := hand == "left"
	var spine_x := rr.end.x - 1.0 if on_right else rr.position.x + 1.0
	var mid := rr.position.y + rr.size.y * 0.5
	var spine := img.get_pixel(int(spine_x), int(mid))
	var tick_px := img.get_pixel(int(rr.end.x - 1.0 if on_right else rr.position.x + 1.0), int(rr.position.y + 1.0))
	var away := img.get_pixel(int(rr.position.x - 8.0 if on_right else rr.end.x + 8.0), int(mid))
	var d_spine := absf(spine.r - ink.r) + absf(spine.g - ink.g) + absf(spine.b - ink.b)
	var d_tick := absf(tick_px.r - ink.r) + absf(tick_px.g - ink.g) + absf(tick_px.b - ink.b)
	var d_away := absf(away.r - ink.r) + absf(away.g - ink.g) + absf(away.b - ink.b)
	print("MEASURE %s ink distance spine=%.3f tick=%.3f map=%.3f" % [tag, d_spine, d_tick, d_away])
	_check("%s: the spine is painted in text_secondary" % tag, d_spine < 0.06)
	_check("%s: the end tick is painted in text_secondary" % tag, d_tick < 0.06)
	_check("%s: beside the rule is map, not the ink (negative control)" % tag, d_away > 0.06)


## A live coach-tip toast (the shell's own `_show_phone_toast()` pointed at the bottom bar, as the first-run tip is)
## must not meet the bar: it sits above the sheet, left-aligned, so the check is a real measurement, not an assumption.
func _toast_case(vh, shell, hand: String) -> void:
	shell.set_phone_detent("peek")
	await get_tree().create_timer(0.5).timeout
	vh.reset_view()
	await _frames(3)
	var w = shell._show_phone_toast("Swipe the tab sideways to change domain, hold it to open the fan.", shell._phone_menu_bar, 30.0)
	await _frames(4)
	_check("coach toast exists", w != null)
	if w != null:
		var tr := _grect(w)
		for mine in [["rule", _grect(vh._scale_rule)], ["label", _grect(vh._scale_label)]]:
			var gap := _clear(mine[1], tr)
			print("MEASURE toast_%s %s-toast gap=%.1f" % [hand, mine[0], gap])
			_check("toast_%s: %s clear of the coach tip" % [hand, mine[0]], gap >= 0.0)
		await _snap("toast_%s" % hand)
		w.queue_free()
