extends Node
## Why the project picker draws its column at about half the screen width on
## the OnePlus 6T. Measures rather than reasons — the device screenshot showed
## content ending at ~52% with the window itself full width, and the layout
## chain has four candidates (`outer`, the `ScrollContainer`, its
## `MarginContainer`, `_list`).
##
##   Godot_v4.7.1 --path . --resolution 1600x900 --rendering-driver opengl3 _pickerwidth_probe.tscn -- --force-touch

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _walk(n: Node, depth: int, out: Array) -> void:
	if n is Control:
		var c := n as Control
		out.append("%s%s [%s]  size=%.0f x %.0f  minw=%.0f  hflags=%d" % [
			"  ".repeat(depth), n.name, n.get_class(),
			c.size.x, c.size.y, c.custom_minimum_size.x, c.size_flags_horizontal])
	if depth > 7:
		return
	for k in n.get_children(true):
		_walk(k, depth + 1, out)

func _ready() -> void:
	await _frames(2)
	if not ClassDB.class_exists("WorldGen"):
		print("[FATAL] extension did not load"); get_tree().quit(1); return
	var vp := SubViewport.new()
	vp.size = Vector2i(1080, 2340)
	vp.gui_embed_subwindows = true
	vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(vp)
	var app: Node = load("res://shell/app.tscn").instantiate()
	vp.add_child(app)
	await _frames(60)
	print("[BOOT] is_phone=", app.is_phone(), "  phone_scale=", app.phone_scale())

	var pk = app.get("phone_project_picker")
	if pk == null:
		print("[FATAL] no picker"); get_tree().quit(1); return
	if not pk.visible:
		pk.open()
	await _frames(20)
	print("[PICKER] visible=", pk.visible, "  window size=", pk.size,
		"  content_scale_factor=", pk.content_scale_factor)
	var rows: Array = []
	_walk(pk, 0, rows)
	for r in rows:
		print(r)
	get_tree().quit(0)
