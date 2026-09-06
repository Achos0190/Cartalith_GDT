extends Node
## ADVERSARIAL CROSS-CHECK -- independent enumeration of ALL SEVEN menus.
## Counts rows/separators per menu, tallies accelerators bar-wide, and dumps
## the specific rows three lanes disputed. Run windowed:
##   godot --path . --resolution 1600x900 _xcheck_probe.tscn

var _vp: SubViewport
var _accels := {}      # accel_int -> Array[String] paths
var _counts := {}      # menu -> {rows, seps, leafrows, leafseps}
var _fail := 0
var _app: Node

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ck(label: String, got, want) -> void:
	var ok: bool = (got == want)
	if not ok:
		_fail += 1
	print("[%s] %s  got=%s want=%s" % ["PASS" if ok else "FAIL", label, str(got), str(want)])

func _walk(p: PopupMenu, path: String, top: String, depth: int, dump: bool) -> void:
	p.about_to_popup.emit()
	await _frames(2)
	var c: Dictionary = _counts.get(top, {"rows": 0, "seps": 0, "top_rows": 0, "top_seps": 0, "popups": 0})
	c["popups"] = int(c["popups"]) + 1
	for i in p.item_count:
		if p.is_item_separator(i):
			c["seps"] = int(c["seps"]) + 1
			if depth == 0: c["top_seps"] = int(c["top_seps"]) + 1
		else:
			c["rows"] = int(c["rows"]) + 1
			if depth == 0: c["top_rows"] = int(c["top_rows"]) + 1
			var a := p.get_item_accelerator(i)
			if a != 0:
				var lst: Array = _accels.get(a, [])
				lst.append("%s > %s" % [path, p.get_item_text(i)])
				_accels[a] = lst
		if dump:
			var meta = p.get_item_metadata(i)
			print("  %s%02d %s id=%-4d %s '%s'%s%s" % ["  ".repeat(depth), i,
				("SEP " if p.is_item_separator(i) else ("DIS " if p.is_item_disabled(i) else "EN  ")),
				p.get_item_id(i),
				(OS.get_keycode_string(p.get_item_accelerator(i)) if p.get_item_accelerator(i) != 0 else "-"),
				p.get_item_text(i),
				("  [meta=%s]" % str(meta)) if meta != null else "",
				("  [sub=%s]" % p.get_item_submenu(i)) if p.get_item_submenu(i) != "" else ""])
			var tt := p.get_item_tooltip(i)
			if tt != "":
				print("  %s     TIP: %s" % ["  ".repeat(depth), tt.replace("\n", " ")])
	_counts[top] = c
	for i in p.item_count:
		var sub := p.get_item_submenu(i)
		if sub == "": continue
		var node := p.get_node_or_null(NodePath(sub))
		if node is PopupMenu:
			await _walk(node as PopupMenu, path + " > " + p.get_item_text(i), top, depth + 1, dump)

func _gather(n: Node, out: Array) -> void:
	if n is MenuButton: out.append(n)
	for c in n.get_children(true): _gather(c, out)

func _find_popup(buttons: Array, name: String) -> PopupMenu:
	for b in buttons:
		if b.text == name: return b.get_popup()
	return null

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
	await _frames(90)
	print("[BOOT] shell up; build=", _app.build_id() if _app.has_method("build_id") else "?")

	var buttons: Array = []
	_gather(_app, buttons)
	var names := []
	for b in buttons: names.append(b.text)
	print("[BAR] %d buttons: %s" % [buttons.size(), str(names)])
	_ck("bar has 7 MenuButtons", buttons.size(), 7)

	for b in buttons:
		print("\n########## MENU: %s ##########" % b.text)
		await _walk(b.get_popup(), b.text, b.text, 0, true)

	print("\n########## ROW COUNTS ##########")
	for k in _counts.keys():
		var c: Dictionary = _counts[k]
		print("%-13s top_rows=%-4d top_seps=%-3d ALL_rows=%-4d ALL_seps=%-3d popups=%d" % [
			k, c["top_rows"], c["top_seps"], c["rows"], c["seps"], c["popups"]])

	print("\n########## ACCELERATORS ##########")
	var dupes := 0
	for a in _accels.keys():
		var lst: Array = _accels[a]
		if lst.size() > 1:
			dupes += 1
			print("  COLLISION %s -> %s" % [OS.get_keycode_string(a), str(lst)])
		else:
			print("  %-22s %s" % [OS.get_keycode_string(a), lst[0]])
	_ck("distinct accelerators", _accels.size(), 18)
	_ck("accelerator collisions", dupes, 0)

	print("\n########## DISPUTED ROWS ##########")
	# --- File storage rows: metadata + ids ---
	var filep := _find_popup(buttons, "File")
	filep.about_to_popup.emit(); await _frames(2)
	var store_meta := 0
	var idmap := {}
	var iddupes := []
	for i in filep.item_count:
		if filep.is_item_separator(i): continue
		var id := filep.get_item_id(i)
		if idmap.has(id): iddupes.append("id=%d : '%s' AND '%s'" % [id, idmap[id], filep.get_item_text(i)])
		else: idmap[id] = filep.get_item_text(i)
		var t := filep.get_item_text(i)
		if t.begins_with("projects") or t.begins_with("tile atlas") or t.begins_with("asset packs") or t.begins_with("exports"):
			if filep.get_item_metadata(i) != null: store_meta += 1
			print("  STORAGE ROW '%s' meta=%s disabled=%s" % [t, str(filep.get_item_metadata(i)), str(filep.is_item_disabled(i))])
	_ck("storage rows carrying META_READOUT", store_meta, 4)
	print("  FILE DUPLICATE IDS (%d): %s" % [iddupes.size(), str(iddupes)])

	# --- Close project position ---
	var close_i := -1; var storage_i := -1
	for i in filep.item_count:
		if filep.get_item_text(i) == "Close project": close_i = i
		if filep.is_item_separator(i) and filep.get_item_text(i) == "STORAGE LOCATIONS": storage_i = i
	print("  Close project index=%d ; STORAGE LOCATIONS separator index=%d" % [close_i, storage_i])
	_ck("Close project before STORAGE band (newest canvas)", close_i < storage_i, true)

	# --- Recent worlds leaves ---
	var rec := filep.get_node_or_null(NodePath("RecentWorlds"))
	if rec == null:
		for i in filep.item_count:
			if filep.get_item_submenu(i) != "" and filep.get_item_text(i).begins_with("Recent"):
				rec = filep.get_node_or_null(NodePath(filep.get_item_submenu(i)))
	if rec is PopupMenu:
		rec.about_to_popup.emit(); await _frames(2)
		print("  RECENT leaves=%d first='%s'" % [rec.item_count, rec.get_item_text(0) if rec.item_count > 0 else ""])

	# --- Edit Delete ---
	var editp := _find_popup(buttons, "Edit")
	editp.about_to_popup.emit(); await _frames(2)
	for i in editp.item_count:
		var t := editp.get_item_text(i)
		if t == "Delete" or t.begins_with("Cut") or t.begins_with("Copy") or t.begins_with("Paste") or t.begins_with("Select all"):
			print("  EDIT '%s' disabled=%s tip='%s'" % [t, str(editp.is_item_disabled(i)), editp.get_item_tooltip(i).substr(0, 70)])
	var dsel = _app.delete_selection() if _app.has_method("delete_selection") else null
	print("  delete_selection() returned: %s" % str(dsel))
	var del_dis := true
	for i in editp.item_count:
		if editp.get_item_text(i) == "Delete": del_dis = editp.is_item_disabled(i)
	_ck("Delete disabled when delete_selection() is false", del_dis, true)

	print("\n[DONE] failures=%d" % _fail)
	get_tree().quit(0)
