extends Node
## Lane AssetsData -- PRESSES every row of the live `Assets` and `Data` menus
## and reports what changed. A row that silently does nothing is the failure
## mode this audit exists to find, so the assertion is "state moved", not
## "no error printed".
##
##   godot --path . --resolution 1600x900 _adpress_probe.tscn

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

func _wins() -> Array:
	var out := []
	for name in ["asset_library_window", "data_manager_window", "vault_window",
			"world_data_window", "travel_library_window"]:
		var w = _app.get(name)
		if w != null and w is Window:
			if (w as Window).visible:
				out.append(name)
	return out

func _closeall() -> void:
	for name in ["asset_library_window", "data_manager_window", "vault_window",
			"world_data_window", "travel_library_window"]:
		var w = _app.get(name)
		if w != null and w is Window:
			(w as Window).hide()
	## Stock dialogs the shell may have parented onto app
	for c in _app.get_children(true):
		if c is AcceptDialog or c is FileDialog or c is ConfirmationDialog:
			if (c as Window).visible:
				(c as Window).hide()

## Every Window in the tree that is visible, by class -- catches stock dialogs
## the shell does not hold a named handle for.
func _all_visible_windows(n: Node, out: Array, depth: int = 0) -> void:
	if depth > 16:
		return
	if n is Window and (n as Window).visible and n != _vp:
		out.append("%s<%s>'%s'" % [n.name, n.get_class(), (n as Window).title])
	for c in n.get_children(true):
		_all_visible_windows(c, out, depth + 1)

func _gather(n: Node, out: Array) -> void:
	if n is MenuButton:
		out.append(n)
	for c in n.get_children(true):
		_gather(c, out)

func _press(pm: PopupMenu, idx: int, label: String) -> void:
	await _closeall()
	await _frames(3)
	var dom_before := str(_app.get("_active_domain")) + "/" + str(_app.get("_domain_mode"))
	pm.id_pressed.emit(pm.get_item_id(idx))
	await _frames(12)
	var vis := []
	_all_visible_windows(_app, vis)
	var dm = _app.get("data_manager_window")
	var sel := ""
	if dm != null:
		sel = str(dm.get("_selected_id"))
	var dom_after := str(_app.get("_active_domain")) + "/" + str(_app.get("_domain_mode"))
	print("  PRESS '%s'  -> windows=%s  dm_route=%s  domain %s->%s" % [
		label, str(vis), sel, dom_before, dom_after])

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

	var buttons: Array = []
	_gather(_app, buttons)
	var assets: PopupMenu = null
	var data: PopupMenu = null
	for b in buttons:
		if b.text == "Assets":
			assets = b.get_popup()
		elif b.text == "Data":
			data = b.get_popup()

	print("\n=== ASSETS top level ===")
	for i in assets.item_count:
		if assets.is_item_separator(i) or assets.get_item_submenu(i) != "" or assets.is_item_disabled(i):
			continue
		await _press(assets, i, assets.get_item_text(i))

	var ap := assets.get_node_or_null(NodePath("AssetPack")) as PopupMenu
	print("\n=== ASSETS > Asset pack ===")
	for i in ap.item_count:
		if ap.is_item_separator(i) or ap.is_item_disabled(i):
			continue
		await _press(ap, i, "Asset pack > " + ap.get_item_text(i))

	print("\n=== ASSETS > Icon families (first) / Texture sets (first) ===")
	var icf := assets.get_node_or_null(NodePath("IconFamilies")) as PopupMenu
	await _press(icf, 0, "Icon families > " + icf.get_item_text(0))
	var txs := assets.get_node_or_null(NodePath("TextureSets")) as PopupMenu
	await _press(txs, 0, "Texture sets > " + txs.get_item_text(0))

	print("\n=== ASSETS > Landmark types ===")
	var lt := assets.get_node_or_null(NodePath("LandmarkTypes")) as PopupMenu
	for i in lt.item_count:
		if lt.is_item_separator(i) or lt.is_item_disabled(i) or lt.get_item_submenu(i) != "":
			continue
		await _press(lt, i, "Landmark types > " + lt.get_item_text(i))
	var fam0 := lt.get_node_or_null(NodePath("LandmarkFam0")) as PopupMenu
	await _press(fam0, 0, "LandmarkFam0 leaf > " + fam0.get_item_text(0))
	await _press(fam0, fam0.item_count - 2, "LandmarkFam0 > " + fam0.get_item_text(fam0.item_count - 2))

	print("\n=== DATA ===")
	for i in data.item_count:
		if data.is_item_separator(i) or data.is_item_disabled(i):
			continue
		await _press(data, i, data.get_item_text(i))

	print("\n=== Definitions pane content ===")
	await _closeall()
	_app.open_data_manager_route("val_defs")
	await _frames(15)
	var dm = _app.get("data_manager_window")
	var body = dm.get("_pane_body")
	var t := []
	_texts(body, t)
	print("  pane_title=", str(dm.get("_pane_title").text))
	print("  label/button count=", t.size())
	for s in t:
		print("    | ", s.replace("\n", " / "))

	print("\n[DONE]")
	get_tree().quit(0)
