extends Node
## Minimum sizes of the four inputs, printed so a BEFORE/AFTER run can be
## diffed. The claim under test is not invariance -- the lane says layout
## moved -- but MONOTONICITY: nothing grew, and no min WIDTH grew at all,
## which is the hazard when a child minimum propagates out through a
## scroll-disabled dock column.
##
##   Godot_v4.7.1 --headless --path . _vfy_minsz_probe.tscn [-- --force-touch]

func _ready() -> void:
	var touch := "--force-touch" in OS.get_cmdline_user_args()
	DccTheme.set_phone(false)
	DccTheme.set_touch(touch)
	DccTheme.apply_theme(true)
	var col := VBoxContainer.new()
	add_child(col)
	var cb := DccWidgets.toggle(col, "Grid", true, func(_v): pass)
	var ob := DccWidgets.choice(col, "Projection", ["equirect", "mercator"], 0, func(_i): pass)
	var sp := DccWidgets.number(col, "Seed", 0, 100, 1, 42, func(_v): pass)
	var le := LineEdit.new()
	le.text = "0.75"
	col.add_child(le)
	DccWidgets.well(le)
	await get_tree().process_frame
	await get_tree().process_frame
	print("=== _vfy_minsz  %s ===" % ("TABLET" if touch else "POINTER"))
	for pair in [[cb, "CheckBox"], [ob, "OptionButton"], [sp, "SpinBox"],
			[sp.get_line_edit(), "SpinBox.LineEdit"], [le, "well().LineEdit"]]:
		var c: Control = pair[0]
		print("%-18s min=%s  row_min=%s" % [pair[1],
			c.get_combined_minimum_size(),
			(c.get_parent() as Control).get_combined_minimum_size()
				if c.get_parent() is Control else "-"])
	print("row0 min = %s" % (col.get_child(0) as Control).get_combined_minimum_size())
	print("col  min = %s" % col.get_combined_minimum_size())
	get_tree().quit(0)
