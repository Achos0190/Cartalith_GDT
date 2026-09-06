extends Node
## `Preferences` / `Window` / `Help` menu enumeration — LANE PrefsWinHelp audit.
##
##   godot --path . --resolution 1600x900 _prefwinhelp_probe.tscn
##
## Walks the LIVE `MenuBar` (never the design set) and dumps every row of the
## three audited menus, recursively into submenus: index, id, label,
## accelerator, disabled, checkable/checked, metadata marker, tooltip.
##
## Two passes per menu: as BUILT, then again after `about_to_popup` fires —
## several rows in `menus.gd` rewrite their own text/disabled state there, so
## the built state is not what a user sees.
var _vp: SubViewport

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _accel_text(a: int) -> String:
	if a == 0:
		return "-"
	return OS.get_keycode_string(a)

func _gather(n: Node, out: Array) -> void:
	if n is MenuButton:
		out.append(n)
	for c in n.get_children(true):
		_gather(c, out)

func _dump(popup: PopupMenu, prefix: String, depth: int) -> void:
	for i in popup.item_count:
		var pad := "  ".repeat(depth)
		if popup.is_item_separator(i):
			print("%s%s[%d] --SEP-- '%s'" % [pad, prefix, i, popup.get_item_text(i)])
			continue
		var kind := "item"
		if popup.is_item_radio_checkable(i):
			kind = "radio"
		elif popup.is_item_checkable(i):
			kind = "check"
		var meta = popup.get_item_metadata(i)
		var sub := popup.get_item_submenu(i)
		var tip := popup.get_item_tooltip(i)
		print("%s%s[%d] id=%d %-5s dis=%s chk=%s acc=%s meta=%s sub='%s' | %s" % [
			pad, prefix, i, popup.get_item_id(i), kind,
			"Y" if popup.is_item_disabled(i) else "n",
			"Y" if popup.is_item_checked(i) else "n",
			_accel_text(popup.get_item_accelerator(i)),
			str(meta) if meta != null else "-",
			sub, popup.get_item_text(i)])
		if tip != "":
			print("%s      TIP: %s" % [pad, tip])
		if sub != "":
			var node := popup.get_node_or_null(NodePath(sub))
			if node is PopupMenu:
				_dump(node as PopupMenu, prefix, depth + 1)
			else:
				print("%s      !! submenu node '%s' NOT FOUND" % [pad, sub])

func _fire(popup: PopupMenu, depth: int) -> void:
	popup.about_to_popup.emit()
	for i in popup.item_count:
		var sub := popup.get_item_submenu(i)
		if sub != "":
			var node := popup.get_node_or_null(NodePath(sub))
			if node is PopupMenu:
				_fire(node as PopupMenu, depth + 1)

func _ready() -> void:
	await _frames(2)
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load"); get_tree().quit(1); return
	_vp = SubViewport.new()
	_vp.size = Vector2i(1600, 900)
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	var app: Node = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await _frames(60)
	print("[BOOT] shell up")
	var bar: Array = []
	_gather(app, bar)
	var names: Array = []
	for mb in bar:
		names.append(mb.text)
	print("[BAR] %d menu buttons: %s" % [bar.size(), str(names)])
	var want := ["Preferences", "Window", "Help"]
	for mb in bar:
		if not (mb.text in want):
			continue
		var pm: PopupMenu = mb.get_popup()
		print("\n\n########## %s — AS BUILT (%d rows) ##########" % [mb.text, pm.item_count])
		_dump(pm, "", 0)
		_fire(pm, 0)
		await _frames(3)
		print("\n---------- %s — AFTER about_to_popup (%d rows) ----------" % [mb.text, pm.item_count])
		_dump(pm, "", 0)
	print("\n[DONE]")
	get_tree().quit(0)
