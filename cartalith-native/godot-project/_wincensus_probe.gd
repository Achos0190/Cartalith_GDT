extends Node
## **WINDOW-CONFORM census.** Walks the four dialog/window surfaces this lane
## owns and reports every *built* control that carries no theme override of its
## own -- i.e. one drawing Godot's stock theme rather than this shell's.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _wincensus_probe.tscn
##   ... -- --force-touch          # tablet density
##   ... -- --light                # the light palette (the preference here)
##
## `DccTheme._touch` latches for the life of the process, so densities cannot
## share a run (`_btnfill_probe.gd` carries the same constraint).
##
## This probe **reports**; it does not assert conformance. Its job is to make
## the private-construction census a measurement instead of a grep, because a
## grep cannot see which builder actually ran.

var app: Node
var _rows: Array = []

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _path(n: Node, root: Node) -> String:
	var parts := PackedStringArray()
	var c := n
	while c != null and c != root:
		parts.append(c.get_class())
		c = c.get_parent()
	parts.reverse()
	return "/".join(parts)

const BTN_BOX := ["normal", "hover", "pressed", "disabled"]
const BTN_INK := ["font_color", "font_hover_color", "font_pressed_color",
	"font_disabled_color"]

func _walk(n: Node, root: Node, win: String) -> void:
	if n is Button:
		var b := n as Button
		var miss_box := PackedStringArray()
		for s in BTN_BOX:
			if not b.has_theme_stylebox_override(s):
				miss_box.append(s)
		var miss_ink := PackedStringArray()
		for s in BTN_INK:
			if not b.has_theme_color_override(s):
				miss_ink.append(s)
		if miss_box.size() > 0 or miss_ink.size() > 0:
			_rows.append({"win": win, "kind": b.get_class(), "flat": b.flat,
				"text": b.text.substr(0, 22), "miss_box": miss_box,
				"miss_ink": miss_ink,
				"min": b.custom_minimum_size, "size": b.size,
				"path": _path(b, root)})
	elif n is LineEdit:
		var le := n as LineEdit
		if not le.has_theme_stylebox_override("normal"):
			_rows.append({"win": win, "kind": "LineEdit", "flat": false,
				"text": le.placeholder_text.substr(0, 22),
				"miss_box": PackedStringArray(["normal"]),
				"miss_ink": PackedStringArray(),
				"min": le.custom_minimum_size, "size": le.size,
				"path": _path(le, root)})
	elif n is HSlider or n is VSlider:
		var s := n as Slider
		var t := 0.0
		if s.has_theme_stylebox_override("slider"):
			var sb := s.get_theme_stylebox("slider") as StyleBoxFlat
			t = sb.content_margin_top + sb.content_margin_bottom
		_rows.append({"win": win, "kind": "Slider", "flat": false,
			"text": "track=%.0f" % t,
			"miss_box": PackedStringArray() if s.has_theme_stylebox_override("slider") \
				else PackedStringArray(["slider"]),
			"miss_ink": PackedStringArray(),
			"min": s.custom_minimum_size, "size": s.size,
			"path": _path(s, root)})
	for c in n.get_children():
		_walk(c, root, win)

func _report(win: String) -> void:
	var n := 0
	for r in _rows:
		if r["win"] != win:
			continue
		n += 1
		print("  %-18s flat=%s min=%dx%d size=%dx%d  box-:%s ink-:%s  %s" \
			% [("'" + String(r["text"]) + "'"), r["flat"],
				(r["min"] as Vector2).x, (r["min"] as Vector2).y,
				(r["size"] as Vector2).x, (r["size"] as Vector2).y,
				r["miss_box"], r["miss_ink"], r["kind"]])
	if n == 0:
		print("  (every built control carries its own overrides)")

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
	print("=== _wincensus_probe density=%s theme=%s ===" \
		% ["tablet" if touch else "pointer", "dark" if dark else "light"])

	## **The app computes its own layout mode from the viewport it is in**, so
	## `DccTheme.set_touch()` alone is not the density: at the headless default
	## 1152 x 648 the aspect is 0.5625, under `_compute_layout_mode()`'s 0.6
	## phone threshold, and with `--force-touch` the whole census silently
	## measured the PHONE composition. Caught by printing `app.is_phone()`.
	## A `SubViewport` is not clamped to the desktop (`CAPTURE.md`), so the
	## frame is chosen here: 1920 x 1080 pointer, 1600 x 1000 touch (0.625,
	## clear of the threshold, which is what makes it a tablet and not a phone).
	var vp := SubViewport.new()
	vp.size = Vector2i(1600, 1000) if touch else Vector2i(1920, 1080)
	vp.gui_embed_subwindows = true
	vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(vp)
	app = load("res://shell/app.tscn").instantiate()
	vp.add_child(app)
	await get_tree().create_timer(1.6).timeout
	print("layout: is_phone=%s is_tablet=%s is_touch=%s" 		% [app.call("is_phone"), DccTheme.is_tablet(), DccTheme.is_touch()])
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(4)

	for spec in [["open_project_dialog", "OPD-gallery"],
			["world_data_window", "WDW"],
			["data_manager_window", "DMW"],
			["asset_library_window", "ALW"]]:
		var w: Node = app.get(String(spec[0]))
		if w == null:
			print("[%s] MISSING" % spec[1])
			continue
		w.call("open")
		await _frames(8)
		_rows.clear()
		_walk(w, w, String(spec[1]))
		print("[%s]  %d control(s) with a gap:" % [spec[1], _rows.size()])
		_report(String(spec[1]))
		w.call("hide")
		await _frames(2)

	## The picker composition is a second tree inside the same dialog.
	var opd: Node = app.open_project_dialog
	opd.call("open_welcome")
	await _frames(8)
	_rows.clear()
	_walk(opd, opd, "OPD-picker")
	print("[OPD-picker]  %d control(s) with a gap:" % _rows.size())
	_report("OPD-picker")
	opd.call("hide")
	await _frames(2)

	## The slicer modal is a child Window of the asset library.
	var alw: Node = app.asset_library_window
	alw.call("open", "", true)
	await _frames(12)
	_rows.clear()
	var sl: Node = alw.get("_slicer")
	if sl == null:
		print("[ALW-slicer] MISSING")
	else:
		_walk(sl, sl, "ALW-slicer")
		print("[ALW-slicer]  %d control(s) with a gap:" % _rows.size())
		_report("ALW-slicer")
		var cp: Node = alw.get("_slicer_chroma_color")
		if cp != null:
			print("  ColorPickerButton min=%s size=%s" \
				% [(cp as Control).custom_minimum_size, (cp as Control).size])
	alw.call("hide")
	await _frames(2)

	print("CENSUS done")
	get_tree().quit(0)
