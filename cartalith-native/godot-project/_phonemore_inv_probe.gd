extends Node
## Ground-truth inventory of what the phone MORE tab reaches TODAY, before the
## §6.6 bespoke-screen rewrite replaces the drill list.
##
## The brief's deliverable is "enumerate what stops being reachable on the
## phone", and that cannot be enumerated from the design: it has to be walked
## off the live `MenuBar`, which is what `phone_menu.gd::_fill_root()` and
## `_fill_popup()` themselves read. So this dumps every non-separator item of
## every program menu, recursively through submenus, with the id, the disabled
## flag and the `_todo`/`_readout`/`_signpost` marker, and prints a machine
## readable line per row.
##
##   Godot_v4.7.1-stable_win64_console.exe --headless --path . _phonemore_inv_probe.tscn

var app: Node

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _kind(p: PopupMenu, i: int) -> String:
	if p.is_item_separator(i):
		return "sep"
	var meta = p.get_item_metadata(i)
	if typeof(meta) == TYPE_STRING:
		if String(meta) == DccMenus.META_SIGNPOST:
			return "signpost"
		if String(meta) == DccMenus.META_READOUT:
			return "readout"
	if p.get_item_submenu(i) != "":
		return "submenu"
	if p.is_item_disabled(i):
		return "todo"
	if p.is_item_radio_checkable(i):
		return "radio"
	if p.is_item_checkable(i):
		return "check"
	return "item"

func _walk(p: PopupMenu, trail: String, depth: int) -> void:
	if depth > 4:
		return
	for i in p.item_count:
		if p.is_item_separator(i):
			var st := p.get_item_text(i).strip_edges()
			if st != "":
				print("ROW\t%s\t-\tband\t%s" % [trail, st])
			continue
		var k := _kind(p, i)
		var txt := p.get_item_text(i).strip_edges()
		print("ROW\t%s\t%d\t%s\t%s" % [trail, p.get_item_id(i), k, txt])
		var sn := p.get_item_submenu(i)
		if sn != "":
			var sub := p.get_node_or_null(NodePath(sn)) as PopupMenu
			if sub != null:
				_walk(sub, trail + " > " + txt, depth + 1)

func _ready() -> void:
	app = load("res://shell/app.tscn").instantiate()
	add_child(app)
	await get_tree().create_timer(1.2).timeout
	if app.open_project_dialog != null:
		app.open_project_dialog.hide()
	await _frames(3)

	var row: HBoxContainer = app.menu_bar_row
	if row == null:
		print("INV  !! no menu_bar_row")
		get_tree().quit(1)
		return
	var menus := 0
	var rows := 0
	for child in row.get_children():
		if not (child is MenuButton):
			continue
		menus += 1
		var mb := child as MenuButton
		var p := mb.get_popup()
		## `phone_menu.gd::_fill_popup()` fires this before reading, so the
		## inventory must too or Recent worlds / GPU devices / Open windows
		## read as their build-time placeholder.
		p.about_to_popup.emit()
		print("MENU\t%s\t%d" % [mb.text, p.item_count])
		_walk(p, String(mb.text), 1)
	print("INV menus=%d" % menus)
	get_tree().quit(0)
