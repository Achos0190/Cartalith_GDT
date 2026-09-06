extends Node
## Two readouts of ONE fact: the `custom` family's fill, as the Assets menu
## draws it and as the Asset library's own rail draws it.

var _vp: SubViewport
var _app: Node

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _gather(n: Node, out: Array) -> void:
	if n is MenuButton:
		out.append(n)
	for c in n.get_children(true):
		_gather(c, out)

func _labels(n: Node, out: Array, d: int = 0) -> void:
	if d > 16:
		return
	if n is Label and String((n as Label).text).strip_edges() != "":
		out.append(String((n as Label).text))
	for c in n.get_children(true):
		_labels(c, out, d + 1)

func _ready() -> void:
	await _frames(2)
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load"); get_tree().quit(1); return
	_vp = SubViewport.new()
	_vp.size = Vector2i(1600, 900)
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	_app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(_app)
	await _frames(60)
	print("[BOOT] shell up, build ", DccShell.build_id())

	var buttons: Array = []
	_gather(_app, buttons)
	var assets: PopupMenu = null
	for b in buttons:
		if b.text == "Assets":
			assets = b.get_popup()
	var icf := assets.get_node_or_null(NodePath("IconFamilies")) as PopupMenu
	icf.about_to_popup.emit()
	await _frames(3)
	for i in icf.item_count:
		if icf.get_item_text(i).begins_with("Custom"):
			print("  MENU  row text : '", icf.get_item_text(i), "'")
			print("  MENU  tooltip  : '", icf.get_item_tooltip(i), "'")

	_app.open_asset_library("custom")
	await _frames(20)
	var t := []
	_labels(_app.asset_library_window, t)
	var idx := t.find("Custom icons")
	print("  RAIL  labels around 'Custom icons': ", str(t.slice(max(0, idx - 1), idx + 3)) if idx >= 0 else "NOT FOUND -> " + str(t.slice(0, 30)))
	print("\n[DONE]")
	get_tree().quit(0)
