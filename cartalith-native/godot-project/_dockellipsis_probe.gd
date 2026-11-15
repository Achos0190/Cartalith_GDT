extends Node
## Right dock key/value label column (`right_dock.gd::_field_label_fit`, OUTSTANDING_WORK "Right dock: field labels clip
## without an ellipsis on tablet"). Windowed; run under a scratch APPDATA:
##
##   Godot_v4.7.2-stable_win64_console.exe --path . _dockellipsis_probe.tscn -- --vp 1600x900 --out <dir>
##   ... -- --vp 1700x1000 --force-touch --out <dir>
##
## Protects: (1) every Sample row's label ends in an ellipsis when too long (never `clip_text`'s silent cut) and carries the
## row's tooltip; (2) on touch, "Boundary + distance" (the 2026-11-15 spot-check's clipped label) FITS its column at the
## 14 px tablet text; (3) on the pointer band the column is exactly the literal 116 px (desktop unchanged). The asserted
## numbers are literals, not `_FIELD_LABEL_W` (a test that reads the constant holds for every value).
## Exit 0 = held, 1 = a check failed, 2 = could not run.

var _touch := false
var _out := ""
var _fails := 0
var _checks := 0


func _ok(what: String, cond: bool, detail: String = "") -> void:
	_checks += 1
	if not cond:
		_fails += 1
	print("DOCKELL %s  %s%s" % ["ok  " if cond else "FAIL", what, ("  -- " + detail) if detail != "" else ""])


func _labels(n: Node, out: Array) -> void:
	for c in n.get_children():
		if c is Label:
			out.append(c)
		_labels(c, out)


func _ready() -> void:
	var vp_size := Vector2i(1600, 900)
	var args := OS.get_cmdline_user_args()
	var i := 0
	while i < args.size():
		match String(args[i]):
			"--vp":
				var wh := String(args[i + 1]).split("x")
				vp_size = Vector2i(int(wh[0]), int(wh[1]))
				i += 2
			"--out":
				_out = String(args[i + 1])
				i += 2
			_:
				if String(args[i]) == "--force-touch":
					_touch = true
				i += 1
	if DisplayServer.get_name() == "headless":
		print("DOCKELL ABORT run windowed")
		get_tree().quit(2)
		return
	var vp := SubViewport.new()
	vp.size = vp_size
	vp.gui_embed_subwindows = true
	vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(vp)
	var app = load("res://shell/app.tscn").instantiate()
	vp.add_child(app)
	await get_tree().create_timer(1.6).timeout
	if app.get("open_project_dialog") != null:
		app.open_project_dialog.hide()
	for _f in 10:
		await get_tree().process_frame
	var dock: Node = app
	var labs: Array = []
	_labels(dock, labs)
	var target: Label = null
	var checked := 0
	var bad := 0
	for l: Label in labs:
		var row := l.get_parent()
		if not (row is HBoxContainer) or l.custom_minimum_size.x < 1.0:
			continue
		if l.text in ["Slope", "Aspect", "Plate + type", "Boundary + distance", "Resistance", "Lithology"]:
			checked += 1
			if l.text_overrun_behavior != TextServer.OVERRUN_TRIM_ELLIPSIS:
				bad += 1
			if l.text == "Boundary + distance":
				target = l
	_ok("Sample labels found", checked >= 4, "found %d" % checked)
	_ok("every Sample label elides with an ellipsis", bad == 0, "%d not trimmed" % bad)
	if target != null:
		var font := target.get_theme_font("font")
		var fs := target.get_theme_font_size("font_size")
		var w := font.get_string_size(target.text, HORIZONTAL_ALIGNMENT_LEFT, -1, fs).x
		var col := target.custom_minimum_size.x
		print("DOCKELL info touch=%s column=%.0f text_w=%.1f font=%d" % [_touch, col, w, fs])
		if _touch:
			_ok("tablet: 'Boundary + distance' fits its column", w <= col, "text %.1f exceeds column %.0f" % [w, col])
			_ok("tablet: column is the scaled literal 148", int(col) == 148, "column %.0f" % col)
		else:
			_ok("desktop: column is exactly 116", int(col) == 116, "column %.0f" % col)
		_ok("the row carries a tooltip", String(target.get_parent().tooltip_text) != "")
	else:
		_ok("'Boundary + distance' label found", false)
	if _out != "":
		await RenderingServer.frame_post_draw
		var name := "dock_labels_%s.png" % ("tablet" if _touch else "desktop")
		vp.get_texture().get_image().save_png(_out.path_join(name))
		print("DOCKELL shot %s" % name)
	print("DOCKELL ---- %d checks, %s ----" % [_checks, "PASS" if _fails == 0 else "%d FAILED" % _fails])
	get_tree().quit(1 if _fails > 0 else 0)
