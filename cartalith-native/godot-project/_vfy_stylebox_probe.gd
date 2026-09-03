extends SceneTree
## Measures the content margins Godot derives for the two styleboxes the
## place-editor trait chip now uses for `normal` vs `hover`, and the widget
## library's own chip pair, to see whether the label moves on hover.

func _initialize() -> void:
	var flat := DccTheme.flat(DccTheme.c("sunken"))
	var out := DccTheme.outline("border", "sunken")
	print("place_editor off-chip  normal flat()   L/T = %.1f/%.1f" % [
		flat.get_margin(SIDE_LEFT), flat.get_margin(SIDE_TOP)])
	print("place_editor off-chip  hover  outline() L/T = %.1f/%.1f" % [
		out.get_margin(SIDE_LEFT), out.get_margin(SIDE_TOP)])
	var rest := DccTheme.outline("line")
	rest.content_margin_left = 10
	rest.content_margin_top = 4
	var lit := DccTheme.outline("line", "line_soft")
	lit.content_margin_left = 10
	lit.content_margin_top = 4
	print("dcc_widgets action()   normal outline() L/T = %.1f/%.1f" % [
		rest.get_margin(SIDE_LEFT), rest.get_margin(SIDE_TOP)])
	print("dcc_widgets action()   hover  outline() L/T = %.1f/%.1f" % [
		lit.get_margin(SIDE_LEFT), lit.get_margin(SIDE_TOP)])
	quit(0)
