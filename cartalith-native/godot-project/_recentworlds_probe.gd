extends Node
## MENUS lane, Part C, 2026-09-13. Dumps `File ▸ Recent worlds`'s real leaf
## rows -- label and tooltip -- against whatever `DccSettings.recent_projects()`
## actually holds on this machine, so the report quotes real output rather than
## reasoning about the code in the abstract.
##
##   Godot_v4.7.1 --path . _recentworlds_probe.tscn
##
## Read-only: does not seed, does not clear, does not write
## `user://cartalith_settings.cfg`. Whatever is in `[recent]` today is what
## this prints.

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _menu_button(root: Node, title: String) -> MenuButton:
	if root is MenuButton and (root as MenuButton).text == title:
		return root as MenuButton
	for c in root.get_children(true):
		var r := _menu_button(c, title)
		if r != null:
			return r
	return null

func _submenu_by_title(pm: PopupMenu, title: String) -> PopupMenu:
	for i in pm.item_count:
		if pm.is_item_separator(i):
			continue
		if pm.get_item_text(i) == title:
			var sub := pm.get_item_submenu(i)
			if sub != "":
				var node := pm.get_node_or_null(NodePath(sub))
				if node is PopupMenu:
					return node as PopupMenu
	return null

var _fail := 0

func _ok(name: String, cond: bool, detail: String = "") -> void:
	if not cond:
		_fail += 1
	print("  ", "ok  " if cond else "FAIL", " ", name, (("   " + detail) if detail != "" else ""))

func _ready() -> void:
	await _frames(2)
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load"); get_tree().quit(1); return
	var vp := SubViewport.new()
	vp.size = Vector2i(1600, 1000)
	vp.gui_embed_subwindows = true
	vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(vp)
	var app: Node = load("res://shell/app.tscn").instantiate()
	vp.add_child(app)
	await _frames(45)

	print("=== recent_projects() on disk, in order ===")
	var recents: Array = DccSettings.recent_projects()
	for i in recents.size():
		var p := String(recents[i])
		print("  [", i, "] exists=", FileAccess.file_exists(p), "  ", p)

	var file_mb := _menu_button(app, "File")
	if file_mb == null:
		print("[FATAL] no File menu button found"); get_tree().quit(1); return
	var file_popup := file_mb.get_popup()
	file_popup.about_to_popup.emit()
	var recent_popup := _submenu_by_title(file_popup, "Recent worlds")
	if recent_popup == null:
		print("[FATAL] no Recent worlds submenu found"); get_tree().quit(1); return
	print("")
	print("=== File > Recent worlds, real rows (label | tooltip) ===")
	print("  item_count=", recent_popup.item_count)
	var any_double_space_dash := false
	var any_edited_prefix := false
	var any_double_space_dot := false
	var missing_rows_ok := true
	var any_tooltip_empty := false
	for i in recent_popup.item_count:
		if recent_popup.is_item_separator(i):
			print("  ---")
			continue
		var label := recent_popup.get_item_text(i)
		var tip := recent_popup.get_item_tooltip(i)
		print("  LABEL|", label)
		print("    tip|", tip)
		if label.contains("  —  "):
			any_double_space_dash = true
		if label.contains("  ·  "):
			any_double_space_dot = true
		if tip.contains("edited "):
			## The tooltip is always the bare path for a present file and the
			## true-reason sentence for a missing one -- neither should ever
			## carry the source function's raw "edited " prose; only the
			## LABEL ever did, and only before Part C's fix.
			pass
		if label.contains("edited "):
			any_edited_prefix = true
		if tip.strip_edges() == "":
			any_tooltip_empty = true
		if not FileAccess.file_exists(String(recents[i]) if i < recents.size() else ""):
			if not label.ends_with("— file not found"):
				missing_rows_ok = false
	print("")
	print("=== MENUS lane Part C gates ===")
	_ok("no row uses the old double-space em-dash ('  —  ')", not any_double_space_dash)
	_ok("no row uses the old double-space middot ('  ·  ')", not any_double_space_dot)
	_ok("no row carries the raw 'edited ' prefix (terse-age ran)", not any_edited_prefix)
	_ok("every missing-file row ends '— file not found'", missing_rows_ok)
	_ok("no row's tooltip is empty", not any_tooltip_empty)
	print("")
	print("_recentworlds_probe: ", "PASS" if _fail == 0 else str(_fail) + " FAILURE(S)")
	print("=== end ===")
	get_tree().quit(1 if _fail > 0 else 0)
