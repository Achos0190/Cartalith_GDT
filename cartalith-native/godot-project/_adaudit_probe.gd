extends Node
## Lane AssetsData conformance audit -- enumerates the LIVE `Assets` and `Data`
## PopupMenus recursively and dumps every row.
##
##   godot --path . --resolution 1600x900 _adaudit_probe.tscn
##
## Windowed, not headless: this walks a real MenuBar built by `menus.gd` and
## fires `about_to_popup` on every popup so the refreshed rows (family counts,
## pack state) are the ones reported.

var _vp: SubViewport

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _accel(a: int) -> String:
	return OS.get_keycode_string(a) if a != 0 else "-"

func _walk(p: PopupMenu, path: String, depth: int) -> void:
	p.about_to_popup.emit()
	await _frames(2)
	print("%s[POPUP] %s   items=%d" % ["  ".repeat(depth), path, p.item_count])
	for i in p.item_count:
		var pad := "  ".repeat(depth + 1)
		if p.is_item_separator(i):
			print("%s%02d SEP  '%s'" % [pad, i, p.get_item_text(i)])
			continue
		var sub := p.get_item_submenu(i)
		var meta = p.get_item_metadata(i)
		print("%s%02d %s id=%-5d accel=%-14s '%s'%s%s%s" % [
			pad, i,
			("DIS " if p.is_item_disabled(i) else "EN  "),
			p.get_item_id(i), _accel(p.get_item_accelerator(i)),
			p.get_item_text(i),
			("  [sub=%s]" % sub) if sub != "" else "",
			("  [meta=%s]" % str(meta)) if meta != null else "",
			("  [check=%s]" % str(p.is_item_checked(i))) if p.is_item_checkable(i) or p.is_item_radio_checkable(i) else ""])
		var tt := p.get_item_tooltip(i)
		if tt != "":
			print("%s     TIP: %s" % [pad, tt.replace("\n", " ")])
	for i in p.item_count:
		var sub := p.get_item_submenu(i)
		if sub == "":
			continue
		var node := p.get_node_or_null(NodePath(sub))
		if node is PopupMenu:
			await _walk(node as PopupMenu, path + " > " + p.get_item_text(i), depth + 1)

func _gather(n: Node, out: Array) -> void:
	if n is MenuButton:
		out.append(n)
	for c in n.get_children(true):
		_gather(c, out)

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

	var buttons: Array = []
	_gather(app, buttons)
	var names := []
	for b in buttons:
		names.append(b.text)
	print("[BAR] ", names)

	for b in buttons:
		if b.text != "Assets" and b.text != "Data":
			continue
		var pm: PopupMenu = b.get_popup()
		print("\n########## MENU: %s ##########" % b.text)
		await _walk(pm, b.text, 0)

	print("\n[DONE]")
	get_tree().quit(0)
