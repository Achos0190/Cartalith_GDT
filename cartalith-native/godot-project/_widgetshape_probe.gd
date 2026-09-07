extends Node
## AUDIT-WIDGETS (2026-09-07). Measures the *shape* every `DccWidgets` factory
## resolves to -- fill, border, corner radius, content margins, min height --
## rather than reading it off the source, so a stylebox that arrives from
## `theme/dark_theme.tres` (`project.godot` `gui/theme/custom`) instead of from
## the factory is reported as what the user actually sees.
##
##   godot4 --path . --resolution 1280x800 _widgetshape_probe.tscn
##
## Written for the desktop density only: the question the owner asked ("all
## buttons and inputs are visually not the same") was asked about the PC shell.
var out: PackedStringArray = []

func _sb(n: Control, slot: String) -> StyleBox:
	var sb := n.get_theme_stylebox(slot)
	return sb

func _dump(name: String, n: Control, slot: String) -> void:
	var sb := _sb(n, slot)
	if sb == null:
		out.append("%-14s %s  <no stylebox>" % [name, slot])
		return
	var line := "%-14s %-9s cls=%s" % [name, slot, sb.get_class()]
	if sb is StyleBoxFlat:
		var f := sb as StyleBoxFlat
		line += " radius=%d/%d/%d/%d" % [f.corner_radius_top_left, f.corner_radius_top_right,
			f.corner_radius_bottom_right, f.corner_radius_bottom_left]
		line += " bg=%s a=%.2f" % [f.bg_color.to_html(false), f.bg_color.a]
		line += " bord=%d,%s a=%.2f" % [f.border_width_left, f.border_color.to_html(false), f.border_color.a]
	line += " pad=%.0f,%.0f,%.0f,%.0f" % [sb.content_margin_left, sb.content_margin_top,
		sb.content_margin_right, sb.content_margin_bottom]
	line += " minh=%.0f fs=%d" % [n.custom_minimum_size.y, n.get_theme_font_size("font_size")]
	out.append(line)

func _ready() -> void:
	var root := VBoxContainer.new()
	add_child(root)
	await get_tree().process_frame

	var a := DccWidgets.action(root, "Run", func(): pass, false)
	var ap := DccWidgets.action(root, "Run", func(): pass, true)
	var ch := DccWidgets.chip(root, "chip", func(): pass)
	var sg := DccWidgets.segment(root, "seg", func(): pass)
	var sgo := DccWidgets.segment(root, "segon", func(): pass)
	DccWidgets.set_segment_on(sgo, true)
	var tb := DccWidgets.text_button(root, "verb", func(): pass)
	var mb := DccWidgets.modal_button(root, "Cancel", func(): pass, false)
	var le := LineEdit.new(); root.add_child(le); DccWidgets.well(le)
	var le2 := LineEdit.new(); root.add_child(le2)          # unstyled control
	var sl := DccWidgets.slider(root, "slider", 0, 1, 0.01, 0.5, "", func(_v): pass)
	var tg := DccWidgets.toggle(root, "toggle", true, func(_v): pass)
	var co := DccWidgets.choice(root, "choice", ["a", "b"], 0, func(_i): pass)
	var nu := DccWidgets.number(root, "number", 0, 10, 1, 5, func(_v): pass)
	await get_tree().process_frame

	out.append("== FACTORY-BUILT BUTTONS ==")
	_dump("action", a, "normal"); _dump("action", a, "hover")
	_dump("action/prim", ap, "normal")
	_dump("chip", ch, "normal")
	_dump("segment", sg, "normal")
	_dump("segment/on", sgo, "normal")
	_dump("text_button", tb, "normal")
	_dump("modal_button", mb, "normal")
	out.append("== FIELDS ==")
	_dump("well(LineEdit)", le, "normal"); _dump("well(LineEdit)", le, "focus")
	_dump("bare LineEdit", le2, "normal")
	out.append("== STOCK CONTROLS THE FACTORIES DO NOT STYLE ==")
	_dump("toggle CheckBox", tg, "normal")
	_dump("choice OptButton", co, "normal")
	var nu_le := nu.get_line_edit()
	_dump("number LineEdit", nu_le, "normal")
	out.append("number SpinBox minh=%.0f  laid=%.0fx%.0f" % [nu.custom_minimum_size.y, nu.size.x, nu.size.y])
	var s: HSlider = sl["slider"]
	out.append("== SLIDER ==")
	_dump("slider track", s, "slider")
	_dump("slider fill", s, "grabber_area")
	var gr := s.get_theme_icon("grabber")
	out.append("slider grabber tex=%s size=%s  minsize=%s" % [
		("null" if gr == null else gr.get_class()),
		("n/a" if gr == null else str(gr.get_size())), str(s.custom_minimum_size)])
	out.append("== LAID SIZES (1280x800 desktop) ==")
	for pair in [["action", a], ["chip", ch], ["segment", sg], ["toggle", tg],
			["choice", co], ["number", nu]]:
		var cc: Control = pair[1]
		out.append("%-14s laid=%.0fx%.0f" % [pair[0], cc.size.x, cc.size.y])
	out.append("== FALLBACK THEME ==")
	out.append("project theme=%s" % ProjectSettings.get_setting("gui/theme/custom", "<none>"))

	for l in out:
		print(l)
	get_tree().quit()
