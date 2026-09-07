extends Node
## ENUMERATION ONLY. For CheckBox/OptionButton/LineEdit/SpinBox: every colour
## theme item the class actually consumes (the DEFAULT theme's list is the
## authority on what the class reads), and where a DccWidgets-built instance
## resolves it from -- a local override, `dark_theme.tres`, or Godot's default.

var _vp: SubViewport
var _root: VBoxContainer

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _chain(klass: String) -> Array:
	var out: Array = []
	var k := klass
	while k != "" and k != "Object":
		out.append(k)
		k = ClassDB.get_parent_class(k)
	return out

func _hex(col: Color) -> String:
	return "#%02x%02x%02x a=%.2f" % [int(round(col.r * 255)), int(round(col.g * 255)),
		int(round(col.b * 255)), col.a]

func _report(tag: String, ctl: Control, klass: String, proj: Theme) -> void:
	var items: Array = ThemeDB.get_default_theme().get_color_list(klass)
	items.sort()
	print("\n--- ", tag, "  (", klass, ")  items the class consumes: ", items.size())
	for item in items:
		var origin := ""
		if ctl.has_theme_color_override(item):
			origin = "OVERRIDE"
		else:
			for k in _chain(klass):
				if proj.has_color(item, k):
					origin = "RESOURCE:" + k
					break
			if origin == "":
				origin = "default"
		var resolved: Color = ctl.get_theme_color(item, klass)
		if origin.begins_with("RESOURCE"):
			print("  [FROM-RES] ", item, "  ", origin, "  = ", _hex(resolved))
		elif origin == "OVERRIDE":
			print("  [ovr     ] ", item, "  = ", _hex(resolved))
		else:
			print("  [default ] ", item, "  = ", _hex(resolved))

func _ready() -> void:
	await _frames(2)
	var proj: Theme = load("res://theme/dark_theme.tres")
	print("[BOOT] godot ", Engine.get_version_info()["string"],
		"  DccTheme.is_dark=", DccTheme.is_dark(),
		"  project theme=", ProjectSettings.get_setting("gui/theme/custom"))
	print("[BOOT] ThemeDB.project_theme is dark_theme.tres: ",
		ThemeDB.get_project_theme() == proj)

	_vp = SubViewport.new()
	_vp.size = Vector2i(600, 500)
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	_root = VBoxContainer.new()
	_root.custom_minimum_size = Vector2(560, 460)
	_vp.add_child(_root)

	var cb := DccWidgets.toggle(_root, "Toggle", true, func(_v): pass)
	var ob := DccWidgets.choice(_root, "Choice", ["A", "B"], 0, func(_i): pass)
	var sp := DccWidgets.number(_root, "Number", 0.0, 10.0, 1.0, 4.0, func(_v): pass)
	await _frames(4)

	_report("toggle()  CheckBox", cb, "CheckBox", proj)
	_report("choice()  OptionButton", ob, "OptionButton", proj)
	_report("number()  SpinBox", sp, "SpinBox", proj)
	_report("number()  SpinBox.get_line_edit()", sp.get_line_edit(), "LineEdit", proj)

	await _extra()
	print("\n[DONE]")
	get_tree().quit(0)

## Appended: what Godot's OWN default theme supplies for the two modulates,
## and what a BARE `CheckBox` (one built outside `DccWidgets.toggle()`, with
## text and the stock check glyph) resolves -- the split that decides whether
## the six `CheckBox` font items are load-bearing or inert.
func _extra() -> void:
	var d := ThemeDB.get_default_theme()
	print("\n--- Godot default theme, CheckBox modulates")
	for n in ["checkbox_checked_color", "checkbox_unchecked_color"]:
		print("  default ", n, " = ", _hex(d.get_color(n, "CheckBox")))
	var bare := CheckBox.new()
	bare.text = "blocked"
	_root.add_child(bare)
	var tog := DccWidgets.toggle(_root, "switch", true, func(_v): pass)
	await _frames(4)
	print("  bare CheckBox   text='", bare.text, "'  icon override 'checked': ",
		bare.has_theme_icon_override("checked"))
	print("  toggle CheckBox text='", tog.text, "'  icon override 'checked': ",
		tog.has_theme_icon_override("checked"))
