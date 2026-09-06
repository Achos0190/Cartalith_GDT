extends Node
## Lane AssetsData follow-ups: status-line text for the two Assets rows that
## open no window, the two `route`-kind Data panes, and an accelerator sweep
## over the WHOLE menu bar (a collision is only visible bar-wide).

var _vp: SubViewport
var _app: Node

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _texts(n: Node, out: Array, depth: int = 0) -> void:
	if depth > 14:
		return
	if n is Label and String((n as Label).text).strip_edges() != "":
		out.append(String((n as Label).text))
	elif n is Button and String((n as Button).text).strip_edges() != "":
		out.append("[btn] " + String((n as Button).text))
	for c in n.get_children(true):
		_texts(c, out, depth + 1)

func _status() -> String:
	var out := []
	var bar = _app.get("_status_labels")
	if bar is Dictionary:
		for k in bar:
			var v = bar[k]
			if v is Label:
				out.append("%s=%s" % [k, (v as Label).text])
	return ", ".join(out)

func _gather(n: Node, out: Array) -> void:
	if n is MenuButton:
		out.append(n)
	for c in n.get_children(true):
		_gather(c, out)

func _walk_accel(p: PopupMenu, menu: String, path: String, out: Array) -> void:
	for i in p.item_count:
		if p.is_item_separator(i):
			continue
		var a := p.get_item_accelerator(i)
		if a != 0:
			out.append([OS.get_keycode_string(a), menu, path + p.get_item_text(i)])
		var sub := p.get_item_submenu(i)
		if sub != "":
			var node := p.get_node_or_null(NodePath(sub))
			if node is PopupMenu:
				_walk_accel(node as PopupMenu, menu, path + p.get_item_text(i) + " > ", out)

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
	print("[BOOT] shell up")

	print("\n=== 1: the two Assets rows that opened no window ===")
	print("  status before          : ", _status())
	_app.asset_library_window.apply_to_map_now()
	await _frames(10)
	print("  after Apply library    : ", _status())
	_app.asset_library_window.export_pack_now()
	await _frames(10)
	print("  after Export pack .zip : ", _status())
	var vis := []
	for c in _app.get_children(true):
		if c is Window and (c as Window).visible:
			vis.append("%s'%s'" % [c.get_class(), (c as Window).title])
	print("  visible dialogs on app : ", str(vis))

	print("\n=== 2: the two `route`-kind Data panes ===")
	for rid in ["import_assets", "export_assets"]:
		_app.open_data_manager_route(rid)
		await _frames(12)
		var dm = _app.get("data_manager_window")
		var t := []
		_texts(dm.get("_pane_body"), t)
		var f := []
		_texts(dm.get("_pane_footer"), f)
		print("  --- ", rid, "  title=", str(dm.get("_pane_title").text))
		for s in t:
			print("      body | ", s.replace("\n", " / "))
		for s in f:
			print("      foot | ", s.replace("\n", " / "))
	_app.data_manager_window.hide()

	print("\n=== 3: accelerator sweep, whole bar ===")
	var buttons: Array = []
	_gather(_app, buttons)
	var accel: Array = []
	for b in buttons:
		_walk_accel(b.get_popup(), b.text, "", accel)
	accel.sort_custom(func(a, c): return String(a[0]) < String(c[0]))
	var seen := {}
	for row in accel:
		var k := String(row[0])
		print("  %-18s %s ▸ %s" % [k, row[1], row[2]])
		if seen.has(k):
			print("      *** COLLISION with ", seen[k])
		else:
			seen[k] = "%s ▸ %s" % [row[1], row[2]]
	print("  total accelerators: ", accel.size(), "  distinct: ", seen.size())

	print("\n[DONE]")
	get_tree().quit(0)
