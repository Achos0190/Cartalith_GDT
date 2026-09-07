extends Node
## Capability probe, 2026-09-07. Answers ONE question by measurement:
## can a `PopupMenu` render a SINGLE item bold while its siblings stay regular?
##
##   godot --headless --path . _prefbold_probe.tscn
##
## Reads no flags. Anything after `--` is rejected loudly.
func _ready() -> void:
	for a in OS.get_cmdline_user_args():
		if String(a).begins_with("--"):
			print("[bold] ABORT unknown flag %s -- this probe reads none" % a)
			get_tree().quit(2)
			return
	var hits: Array[String] = []
	for m in ClassDB.class_get_method_list("PopupMenu", true):
		var n := String(m["name"])
		if n.begins_with("set_item_"):
			hits.append(n)
	print("[bold] PopupMenu set_item_* methods (%d):" % hits.size())
	print("[bold]   %s" % ", ".join(hits))
	var fonty: Array[String] = []
	for n in hits:
		if "font" in n or "bold" in n or "weight" in n or "style" in n:
			fonty.append(n)
	print("[bold] of those, any naming font/bold/weight/style: %s" % str(fonty))

	var tl := ThemeDB.get_default_theme().get_font_list("PopupMenu")
	print("[bold] PopupMenu theme font entries: %s" % str(tl))

	## Does an item carry its own font at all? Ask the instance.
	var pm := PopupMenu.new()
	add_child(pm)
	pm.add_radio_check_item("Alpha", 0)
	pm.add_radio_check_item("Beta", 1)
	pm.set_item_checked(1, true)
	var props: Array[String] = []
	for p in pm.get_property_list():
		var pn := String(p["name"])
		if pn.begins_with("item_") or "/font" in pn:
			props.append(pn)
	print("[bold] PopupMenu per-item properties exposed: %s" % str(props))

	## The one mechanism that IS per-item and does not need a font: the icon
	## column. Confirm it accepts a texture per row.
	print("[bold] set_item_icon present: %s" % str("set_item_icon" in hits))
	print("[bold] set_item_indent present: %s" % str("set_item_indent" in hits))
	## And whether the popup honours a whole-popup bold override at all,
	## which is the fallback shape if per-item is impossible.
	var f := SystemFont.new()
	f.font_names = PackedStringArray(["Segoe UI"])
	f.font_weight = 700
	pm.add_theme_font_override("font", f)
	print("[bold] whole-popup font override accepted: %s"
		% str(pm.has_theme_font_override("font")))
	print("[bold] RESULT informational")
	get_tree().quit(0)
