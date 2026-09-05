extends Node
## Does the new floor change WORLD ▸ Generation pipeline (mode a), where §3 asks
## for no gate and nine headers are on screen? Compared against CARTO, which the
## lane deliberately left without a floor.
var app: Node
func _f(n: int) -> void:
	for i in n:
		await get_tree().process_frame
func _open(id: String) -> Array:
	var p: Control = app.workspace_panel(id) as Control
	var out: Array = []
	for e in p.categories:
		var b: Control = e.get("body")
		if b != null and b.visible:
			out.append(String(e.get("title", "")))
	return out
func _press(id: String, title: String) -> void:
	var p: Control = app.workspace_panel(id) as Control
	for e in p.categories:
		if String(e.get("title", "")) == title:
			(e["button"] as Button).pressed.emit()
func _ready() -> void:
	var vp := SubViewport.new(); vp.size = Vector2i(1920, 1080)
	vp.gui_embed_subwindows = true; add_child(vp)
	app = load("res://shell/app.tscn").instantiate(); vp.add_child(app)
	await _f(60)
	if app.open_project_dialog != null: app.open_project_dialog.hide()
	await _f(4)
	app.select_domain_mode("world", "a")
	await _f(4)
	print("FLR world/a open before close: %s" % str(_open("world")))
	_press("world", "Generate")
	await _f(4)
	print("FLR world/a after closing the one open header: %s" % str(_open("world")))
	app.select_domain_mode("cartography", "style")
	await _f(4)
	print("FLR carto open before close: %s" % str(_open("cartography")))
	_press("cartography", "Layers")
	await _f(4)
	print("FLR carto after closing the one open header: %s" % str(_open("cartography")))
	app.select_domain_mode("civilization", "landmarks")
	await _f(4)
	_press("civilization", "Landmarks")
	await _f(4)
	print("FLR civil after closing the one open header: %s" % str(_open("civilization")))
	get_tree().quit(0)
