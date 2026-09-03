extends Node
## Dumps every `available: false` row of `CommandIndex` with its group and its
## stated reason, so the reasons can be audited against the code one by one.
##
## Also walks the live `MenuBar` itself and lists every row that is DISABLED
## AND SILENT -- the rows `command_index.gd::_walk_popup` drops as chrome. A
## real command that lands in that set disappears from the index entirely,
## which is the same defect as a false reason with the volume turned to zero.
##
##   godot --headless --path . --resolution 1600x900 _cmdunavail_probe.tscn

var _vp: SubViewport

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _gather(n: Node, out: Array) -> void:
	if n is MenuButton:
		out.append(n)
	for c in n.get_children(true):
		_gather(c, out)

func _walk(p: PopupMenu, menu: String, out: Array) -> void:
	for i in p.item_count:
		var text := p.get_item_text(i)
		if text.strip_edges() == "" or p.is_item_separator(i):
			continue
		var sub := p.get_item_submenu(i)
		if sub != "":
			var node := p.get_node_or_null(NodePath(sub))
			if node is PopupMenu:
				_walk(node as PopupMenu, menu, out)
			continue
		if not p.is_item_disabled(i):
			continue
		if p.get_item_tooltip(i).strip_edges() != "":
			continue
		var meta = p.get_item_metadata(i)
		var marker := String(meta) if typeof(meta) == TYPE_STRING else ""
		out.append("%s | %s | marker=%s" % [menu, text, marker if marker != "" else "(none)"])

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
	await _frames(50)
	print("[BOOT] shell up")

	var idx := CommandIndex.new()
	idx.build(app, app.get("bridge"))
	var total := idx.size()
	var params := 0
	var menus := 0
	var readouts := 0
	var unavailable := 0
	for r in idx.all():
		match String(r["kind"]):
			"param": params += 1
			"menu": menus += 1
			"readout": readouts += 1
		if not bool(r["available"]):
			unavailable += 1
	print("TOTALS total=%d params=%d menu=%d readout=%d unavailable=%d"
		% [total, params, menus, readouts, unavailable])
	print("--- unavailable rows ---")
	var n := 0
	for r in idx.all():
		if bool(r["available"]):
			continue
		n += 1
		print("[%02d] %s | %s | kind=%s" % [n, String(r["group"]), String(r["title"]), String(r["kind"])])
		print("     why: %s" % String(r["why"]))
	print("--- readout rows ---")
	for r in idx.all():
		if String(r["kind"]) == "readout":
			print("  RO  %s | %s" % [String(r["group"]), String(r["title"])])

	print("--- disabled AND silent (dropped from the index as chrome) ---")
	var buttons: Array = []
	_gather(app, buttons)
	var silent: Array = []
	for mb in buttons:
		var pop: PopupMenu = (mb as MenuButton).get_popup()
		if pop != null:
			_walk(pop, String((mb as MenuButton).text), silent)
	print("  count: %d" % silent.size())
	for s in silent:
		print("  DROP  %s" % s)
	get_tree().quit(0)
