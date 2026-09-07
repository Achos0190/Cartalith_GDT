extends Node
## VERIFIER probe: are the inputs INDEPENDENT of the project theme?
##
## Method: build the four inputs, snapshot every theme item that
## `res://theme/dark_theme.tres` defines for their classes, then install a
## HOSTILE theme at the scene root -- which outranks the project default theme
## in Godot's lookup order -- and re-read. Anything that MOVES is still being
## supplied by the project theme, not by the widget.
##
##   Godot_v4.7.1 --headless --path . _vfy_indep_probe.tscn [-- --force-touch]

const ITEMS := {
	"CheckBox": {
		"styles": ["normal", "hover", "pressed", "disabled", "focus"],
		"colors": ["font_color", "font_hover_color", "font_pressed_color",
			"font_disabled_color", "checkbox_checked_color", "checkbox_unchecked_color"],
		"font_sizes": ["font_size"],
		"icons": ["checked", "unchecked", "checked_disabled", "unchecked_disabled"]},
	"OptionButton": {
		"styles": ["normal", "hover", "pressed", "disabled", "focus"],
		"colors": ["font_color", "font_hover_color", "font_pressed_color",
			"font_disabled_color"],
		"font_sizes": ["font_size"], "icons": []},
	"LineEdit": {
		"styles": ["normal", "focus", "read_only"],
		"colors": ["font_color", "font_placeholder_color", "font_uneditable_color",
			"caret_color", "selection_color"],
		"font_sizes": ["font_size"], "icons": []},
	"SpinBox": {
		"styles": ["up_background", "up_background_hovered", "up_background_pressed",
			"up_background_disabled", "down_background", "down_background_hovered",
			"down_background_pressed", "down_background_disabled"],
		"colors": ["up_icon_modulate", "up_hover_icon_modulate",
			"up_pressed_icon_modulate", "up_disabled_icon_modulate",
			"down_icon_modulate", "down_hover_icon_modulate",
			"down_pressed_icon_modulate", "down_disabled_icon_modulate"],
		"font_sizes": [], "icons": []},
}

var moved: Array[String] = []
var checked := 0

func sig(c: Control, kind: String, n: String, cls: String) -> String:
	match kind:
		"styles":
			var s := c.get_theme_stylebox(n)
			if s == null: return "null"
			if s is StyleBoxFlat:
				var f := s as StyleBoxFlat
				return "flat:%s:r%d:b%d:m%d,%d" % [f.bg_color.to_html(), \
					f.corner_radius_top_left, f.border_width_left, \
					f.content_margin_left, f.content_margin_top]
			return s.get_class()
		"colors": return c.get_theme_color(n).to_html()
		"font_sizes": return str(c.get_theme_font_size(n))
		"icons":
			var t := c.get_theme_icon(n)
			return "null" if t == null else "%s:%dx%d" % [t.get_class(), \
				t.get_width(), t.get_height()]
	return "?"

func snap(c: Control, cls: String) -> Dictionary:
	var d := {}
	for kind in ITEMS[cls]:
		for n in ITEMS[cls][kind]:
			d["%s/%s/%s" % [cls, kind, n]] = sig(c, kind, n, cls)
	return d

func hostile() -> Theme:
	## Every item dark_theme.tres defines, redefined to something unmissable.
	var t := Theme.new()
	var sb := StyleBoxFlat.new()
	sb.bg_color = Color(1, 0, 1)          # magenta
	sb.set_corner_radius_all(31)
	sb.set_border_width_all(7)
	sb.content_margin_left = 41
	sb.content_margin_right = 41
	sb.content_margin_top = 43
	sb.content_margin_bottom = 43
	var ico := ImageTexture.create_from_image(
		Image.create(63, 61, false, Image.FORMAT_RGBA8))
	for cls in ITEMS:
		for n in ITEMS[cls]["styles"]:
			t.set_stylebox(n, cls, sb)
		for n in ITEMS[cls]["colors"]:
			t.set_color(n, cls, Color(0, 1, 0))
		for n in ITEMS[cls]["font_sizes"]:
			t.set_font_size(n, cls, 47)
		for n in ITEMS[cls]["icons"]:
			t.set_icon(n, cls, ico)
	return t

func _ready() -> void:
	var touch := "--force-touch" in OS.get_cmdline_user_args()
	DccTheme.set_phone(false)
	DccTheme.set_touch(touch)
	DccTheme.apply_theme(true)
	print("=== _vfy_indep_probe  density=%s ===" % ("TABLET" if touch else "POINTER"))
	print("project theme = %s" % ProjectSettings.get_setting("gui/theme/custom"))
	## Does the ENGINE itself define the CheckBox icon-modulate colours? If it
	## does, they modulate the switch textures this batch installed.
	var dt := ThemeDB.get_default_theme()
	for n in ["checkbox_checked_color", "checkbox_unchecked_color"]:
		print("engine default theme has CheckBox/%s = %s" % [n, dt.has_color(n, "CheckBox")])

	var col := VBoxContainer.new()
	add_child(col)
	var cb := DccWidgets.toggle(col, "Grid", true, func(_v): pass)
	var ob := DccWidgets.choice(col, "Projection", ["equirect", "mercator"], 0, func(_i): pass)
	var spin := DccWidgets.number(col, "Seed", 0, 100, 1, 42, func(_v): pass)
	var le := LineEdit.new()
	col.add_child(le)
	DccWidgets.well(le)

	var targets := [[cb, "CheckBox"], [ob, "OptionButton"],
		[spin, "SpinBox"], [spin.get_line_edit(), "LineEdit"], [le, "LineEdit"]]
	var names := ["toggle()", "choice()", "number()", "number().LineEdit", "well()"]

	var before := []
	for t in targets:
		before.append(snap(t[0], t[1]))

	## **The hostile theme is written INTO `dark_theme.tres` itself.**
	## `load()` returns the same cached `Theme` instance `ThemeDB` holds for
	## `gui/theme/custom`, so mutating it is literally the project theme
	## changing under the widgets -- not a proxy for it.
	var proj := load("res://theme/dark_theme.tres") as Theme
	var h := hostile()
	for cls in ITEMS:
		for n in ITEMS[cls]["styles"]:
			proj.set_stylebox(n, cls, h.get_stylebox(n, cls))
		for n in ITEMS[cls]["colors"]:
			proj.set_color(n, cls, h.get_color(n, cls))
		for n in ITEMS[cls]["font_sizes"]:
			proj.set_font_size(n, cls, h.get_font_size(n, cls))
		for n in ITEMS[cls]["icons"]:
			proj.set_icon(n, cls, h.get_icon(n, cls))
	await get_tree().process_frame

	## Control: something with NO overrides must move, or the swap did nothing.
	var bare := CheckBox.new()
	col.add_child(bare)
	await get_tree().process_frame
	var bsb := bare.get_theme_stylebox("normal", "CheckBox")
	var bare_moved := bsb is StyleBoxFlat 		and (bsb as StyleBoxFlat).bg_color.is_equal_approx(Color(1, 0, 1))
	print("CONTROL  a bare CheckBox DID take the hostile theme: %s  (got %s)"
		% [bare_moved, bsb.get_class() if bsb else "null"])
	if not bare_moved:
		moved.append("CONTROL FAILED -- the hostile theme was not in force at all")

	for i in targets.size():
		var after := snap(targets[i][0], targets[i][1])
		for k in before[i]:
			checked += 1
			if before[i][k] != after[k]:
				moved.append("%s %s: %s -> %s" % [names[i], k, before[i][k], after[k]])

	print("-- items compared: %d --" % checked)
	if moved.is_empty():
		print("INDEPENDENT: every item held under a hostile project-level theme")
	else:
		print("DEPENDENT on the project theme in %d item(s):" % moved.size())
		for m in moved:
			print("   ", m)
	print("=== indep: %d moved ===" % moved.size())
	get_tree().quit(1 if not moved.is_empty() else 0)
