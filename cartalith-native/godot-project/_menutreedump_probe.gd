extends Node
## Diagnostic only, TABLET lane 2026-09-13 -- not a gate, not part of any
## committed probe's assertions. Prints the tablet menu bar's full tree
## (titles, indentation by submenu depth) for the batch report's "menu tree"
## field. Read-only: boots one tablet composition, prints, quits 0 always.
##
##   Godot_v4.7.1 --path . --resolution 1600x900 _menutreedump_probe.tscn -- --force-touch

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _find_menu_bar(n: Node) -> Node:
	for c in n.get_children(true):
		if c is MenuButton:
			return n
	for c in n.get_children(true):
		var r := _find_menu_bar(c)
		if r != null:
			return r
	return null

func _dump(pm: PopupMenu, indent: int) -> void:
	for i in pm.item_count:
		if pm.is_item_separator(i):
			print("  ".repeat(indent), "---")
			continue
		var accel: int = pm.get_item_accelerator(i)
		var accel_s := ("  [" + OS.get_keycode_string(accel) + "]") if accel != 0 else ""
		var disabled := "  (disabled)" if pm.is_item_disabled(i) else ""
		print("  ".repeat(indent), "- ", pm.get_item_text(i), accel_s, disabled)
		var sub := pm.get_item_submenu(i)
		if sub != "":
			var node := pm.get_node_or_null(NodePath(sub))
			if node is PopupMenu:
				_dump(node as PopupMenu, indent + 1)

func _ready() -> void:
	await _frames(2)
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load"); get_tree().quit(1); return
	var vp := SubViewport.new()
	vp.size = Vector2i(2560, 1600)
	vp.gui_embed_subwindows = true
	vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(vp)
	var app: Node = load("res://shell/app.tscn").instantiate()
	vp.add_child(app)
	await _frames(45)
	print("=== tablet menu tree (force-touch=", "--force-touch" in OS.get_cmdline_user_args(), ") ===")
	var bar := _find_menu_bar(app)
	if bar == null:
		print("[FATAL] no menu bar found"); get_tree().quit(1); return
	for c in bar.get_children(true):
		if c is MenuButton:
			var mb := c as MenuButton
			print(mb.text, ":")
			_dump(mb.get_popup(), 1)
	get_tree().quit(0)
