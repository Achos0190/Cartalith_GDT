extends Node
## Lane FileEdit — enumerate the File and Edit menus FROM THE LIVE `PopupMenu`,
## recursively into submenus, and press every live row.
##
##   godot --headless --path . _fileedit_probe.tscn
##
## Audit only: it changes nothing on disk except the settings a pressed row is
## meant to change, and it restores those. Every number it prints is read off
## the built menu, not off `menus.gd`'s source.

var _vp: SubViewport
var _app: Node
var _fail := 0
var _rows := 0

func _frames(n: int) -> void:
	for i in n:
		await get_tree().process_frame

func _ok(name: String, got, want) -> void:
	var good: bool = str(got) == str(want)
	if not good:
		_fail += 1
	print("  ", "ok  " if good else "FAIL", " ", name, "   got=", got, " want=", want)

func _gather(n: Node, out: Array) -> void:
	if n is MenuButton:
		out.append(n)
	for c in n.get_children(true):
		_gather(c, out)

func _accel(p: PopupMenu, i: int) -> String:
	var a := p.get_item_accelerator(i)
	if a == 0:
		return "-"
	return OS.get_keycode_string(a)

func _kind(p: PopupMenu, i: int) -> String:
	if p.is_item_separator(i):
		return "SEP"
	if p.get_item_submenu(i) != "":
		return "SUB"
	var meta = p.get_item_metadata(i)
	var m := String(meta) if typeof(meta) == TYPE_STRING else ""
	if m == DccMenus.META_READOUT:
		return "READOUT"
	if m == DccMenus.META_SIGNPOST:
		return "SIGNPOST"
	if p.is_item_checkable(i):
		return "CHECK"
	if p.is_item_radio_checkable(i):
		return "RADIO"
	return "item"

func _walk(p: PopupMenu, depth: int, path: String) -> void:
	for i in p.item_count:
		var kind := _kind(p, i)
		var text := p.get_item_text(i)
		var pad := "  ".repeat(depth)
		if kind == "SEP":
			print("   %s[sep] %s" % [pad, text])
			continue
		_rows += 1
		var id := p.get_item_id(i)
		var dis := p.is_item_disabled(i)
		var tip := p.get_item_tooltip(i).replace("\n", " ")
		if tip.length() > 150:
			tip = tip.substr(0, 150) + "…"
		var chk := ""
		if p.is_item_checkable(i) or p.is_item_radio_checkable(i):
			chk = " [x]" if p.is_item_checked(i) else " [ ]"
		print("   %s%-8s id=%-4d %-12s %s%s" % [pad, kind, id, "DISABLED" if dis else "enabled",
			text, chk])
		print("   %s         accel=%s" % [pad, _accel(p, i)])
		if tip != "":
			print("   %s         why: %s" % [pad, tip])
		var sub := p.get_item_submenu(i)
		if sub != "":
			var node := p.get_node_or_null(NodePath(sub))
			if node is PopupMenu:
				(node as PopupMenu).about_to_popup.emit()
				await _frames(1)
				await _walk(node as PopupMenu, depth + 1, path + " ▸ " + text)
			else:
				print("   %s         !! submenu node '%s' NOT FOUND" % [pad, sub])
				_fail += 1

func _dump(p: PopupMenu, title: String) -> void:
	p.about_to_popup.emit()
	await _frames(2)
	print("")
	print("--- %s : %d top-level entries ---" % [title, p.item_count])
	await _walk(p, 0, title)

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

	var mbs: Array = []
	_gather(_app, mbs)
	var titles: Array = []
	for mb in mbs:
		titles.append(String((mb as MenuButton).text))
	print("[BAR] %d MenuButtons: %s" % [mbs.size(), ", ".join(titles)])

	var file_pop: PopupMenu = null
	var edit_pop: PopupMenu = null
	for mb in mbs:
		match String((mb as MenuButton).text):
			"File": file_pop = (mb as MenuButton).get_popup()
			"Edit": edit_pop = (mb as MenuButton).get_popup()
	_ok("File menu found", file_pop != null, true)
	_ok("Edit menu found", edit_pop != null, true)
	if file_pop == null or edit_pop == null:
		get_tree().quit(1); return

	print("")
	print("############ PASS A — NO WORLD ############")
	_rows = 0
	await _dump(file_pop, "File")
	var file_rows_a := _rows
	_rows = 0
	await _dump(edit_pop, "Edit")
	var edit_rows_a := _rows
	print("")
	print("[COUNT-A] File rows=%d  Edit rows=%d" % [file_rows_a, edit_rows_a])

	print("")
	print("############ generating a world ############")
	var bridge = _app.get("bridge")
	bridge.generate({"seed": 4242, "width_km": 600.0, "grid_w": 256, "grid_h": 192,
		"sea_level": 0.5, "villages": true})
	await bridge.generation_finished
	await _frames(30)
	_ok("a world landed", bridge.has_world, true)
	_ok("save_api reports a writer", bridge.save_api, true)

	print("")
	print("############ PASS B — WORLD, NEVER SAVED ############")
	_rows = 0
	await _dump(file_pop, "File")
	var file_rows_b := _rows
	_rows = 0
	await _dump(edit_pop, "Edit")
	var edit_rows_b := _rows
	print("")
	print("[COUNT-B] File rows=%d  Edit rows=%d" % [file_rows_b, edit_rows_b])
	_ok("File row count is stable across passes", file_rows_a, file_rows_b)
	_ok("Edit row count is stable across passes", edit_rows_a, edit_rows_b)

	print("")
	print("############ EXPORT — what the bar says about it, measured ############")
	## File owns save; the brief asks whether any row still implies a TILED
	## export lives under Data (ruling 29). Read off the live menus, not off
	## the ROUTES table, because the badge is concatenated at build time.
	for mb in mbs:
		var mn := String((mb as MenuButton).text)
		var pp: PopupMenu = (mb as MenuButton).get_popup()
		pp.about_to_popup.emit()
		await _frames(1)
		for i in pp.item_count:
			if pp.is_item_separator(i):
				continue
			var t := pp.get_item_text(i)
			var lt := t.to_lower()
			if lt.find("export") >= 0 or lt.find("tile") >= 0:
				print("  %-12s %-40s  %s  tip=%s" % [mn, t,
					"DISABLED" if pp.is_item_disabled(i) else "enabled",
					pp.get_item_tooltip(i).replace("
", " ").substr(0, 90)])

	print("")
	print("############ ACCELERATORS — every menu in the bar ############")
	## Collisions are a bar-wide question, not a File/Edit one: two menus can
	## each bind Ctrl+D and neither file knows about the other. Godot runs the
	## accelerators of every PopupMenu in the tree, so the second one drawn is
	## the one that never fires.
	var seen := {}
	for mb in mbs:
		var mname := String((mb as MenuButton).text)
		await _accel_scan((mb as MenuButton).get_popup(), mname, seen)
	var collisions := 0
	for k in seen.keys():
		var owners: Array = seen[k]
		if owners.size() > 1:
			collisions += 1
			print("  COLLISION %s  <- %s" % [k, ", ".join(owners)])
	print("  info %d distinct accelerators bound across the bar" % seen.size())
	_ok("no accelerator is bound twice in the menu bar", collisions, 0)

	print("")
	print("############ PRESS — every live id reaches a handler ############")
	## Every id the two dispatchers claim to handle, and the host method each
	## one calls. A row that presses into a `match` with no arm, or onto a
	## `_host` that has no such method, is the silently-inert failure this
	## register keeps finding.
	var host_methods := {
		"open_new_world": DccMenus.ID_NEW_WORLD,
		"open_project_picker": DccMenus.ID_OPEN_PROJECT,
		"save_project": DccMenus.ID_SAVE,
		"save_project_as": DccMenus.ID_SAVE_AS,
		"apply_autosave_setting": DccMenus.ID_AUTOSAVE,
		"revert_to_saved": DccMenus.ID_REVERT,
		"close_project": DccMenus.ID_CLOSE,
		"open_storage_locations": DccMenus.ID_STORAGE,
		"show_project_on_disk": DccMenus.ID_SHOW_ON_DISK,
		"open_recent_project": -1,
		"undo_last": DccMenus.ID_UNDO,
		"open_undo_history": DccMenus.ID_UNDO_HISTORY,
		"delete_selection": DccMenus.ID_DELETE,
		"clear_selection": DccMenus.ID_DESELECT,
		"open_find_on_map": DccMenus.ID_FIND_ON_MAP,
		"active_domain": -1,
	}
	for m in host_methods.keys():
		_ok("host has %s()" % m, _app.has_method(String(m)), true)

	## Presses that are observable and safe. Each asserts a state change, so a
	## row wired to nothing fails rather than passing silently.
	print("")
	print("--- press: File ▸ Autosave (check item) ---")
	var before_auto: bool = DccSettings.autosave_enabled()
	file_pop.id_pressed.emit(DccMenus.ID_AUTOSAVE)
	await _frames(2)
	_ok("Autosave flipped the stored bit", DccSettings.autosave_enabled(), not before_auto)
	file_pop.id_pressed.emit(DccMenus.ID_AUTOSAVE)
	await _frames(2)
	_ok("…and flipping back restores it", DccSettings.autosave_enabled(), before_auto)

	print("")
	print("--- press: File ▸ Autosave interval radios ---")
	var auto_sub: PopupMenu = file_pop.get_node_or_null(NodePath("AutosaveInterval"))
	_ok("AutosaveInterval submenu node exists", auto_sub != null, true)
	if auto_sub != null:
		var before_min: int = DccSettings.autosave_minutes()
		var before_on: bool = DccSettings.autosave_enabled()
		for i in DccMenus.AUTOSAVE_MINUTES.size():
			auto_sub.id_pressed.emit(DccMenus.ID_AUTOSAVE_FIRST + i)
			await _frames(2)
			_ok("radio %d min writes the setting" % DccMenus.AUTOSAVE_MINUTES[i],
				DccSettings.autosave_minutes(), DccMenus.AUTOSAVE_MINUTES[i])
		auto_sub.id_pressed.emit(DccMenus.ID_AUTOSAVE_OFF)
		await _frames(2)
		_ok("Off writes autosave_enabled=false", DccSettings.autosave_enabled(), false)
		DccSettings.set_autosave_minutes(before_min)
		DccSettings.set_autosave_enabled(before_on)

	print("")
	print("--- press: Edit ▸ Deselect / Select all / Undo history / Find on map ---")
	_app.select_domain("cartography")
	await _frames(6)
	var lbl: int = bridge.label_create(30.0, 40.0, "Probe A")
	_ok("a label to select", lbl >= 0, true)
	bridge.label_select(lbl)
	await _frames(2)
	edit_pop.id_pressed.emit(DccMenus.ID_SELECT_ALL)
	await _frames(4)
	_ok("Select all selected something", bridge.label_get_selection().size() > 0, true)
	edit_pop.id_pressed.emit(DccMenus.ID_DESELECT)
	await _frames(4)
	_ok("Deselect cleared it", bridge.label_get_selection().size(), 0)

	var before_ctx = _app.right_dock_ctrl
	edit_pop.id_pressed.emit(DccMenus.ID_UNDO_HISTORY)
	await _frames(6)
	print("  info right dock after Undo history: ", _app.right_dock_ctrl)

	var win_before := _count_windows(_app)
	edit_pop.id_pressed.emit(DccMenus.ID_FIND_ON_MAP)
	await _frames(8)
	var win_after := _count_windows(_app)
	print("  info visible windows before=%d after=%d" % [win_before, win_after])
	_ok("Find on map opened something", win_after > win_before, true)

	print("")
	print("--- press: Edit ▸ Delete with NOTHING selected ---")
	_app.clear_selection()
	await _frames(4)
	var deleted: bool = _app.delete_selection()
	print("  info Delete row is enabled=%s while delete_selection() returns %s"
		% [not edit_pop.is_item_disabled(edit_pop.get_item_index(DccMenus.ID_DELETE)), deleted])
	_ok("Delete does nothing when nothing is selected", deleted, false)
	edit_pop.id_pressed.emit(DccMenus.ID_DELETE)
	await _frames(4)

	print("")
	print("--- the four STORAGE LOCATIONS rows, as the searchable index sees them ---")
	var idx := CommandIndex.new()
	idx.build(_app, _app.get("bridge"))
	var storage_hits := 0
	for r in idx.all():
		var t := String(r["title"])
		if String(r["group"]) != "File":
			continue
		if t.begins_with("projects") or t.begins_with("tile atlas") 				or t.begins_with("asset packs") or t.begins_with("exports"):
			storage_hits += 1
			print("  INDEXED  title=%s  kind=%s  available=%s  why=%s"
				% [t, String(r["kind"]), r["available"], String(r["why"])])
	_ok("no storage-root row is indexed as a command", storage_hits, 0)

	print("")
	print("--- press: Edit ▸ Reset generation parameters ---")
	## A parameter that really takes a fractional value, found by writing one
	## and reading it back -- an integer-typed key silently rounds, which would
	## make the "it moved" control vacuous and the reset assertion meaningless.
	var pk := ""
	var def_v := 0.0
	for k in bridge.param_keys():
		var key := String(k)
		var dv = bridge.param_default(key)
		if typeof(dv) != TYPE_FLOAT and typeof(dv) != TYPE_INT:
			continue
		var base := float(dv)
		if not bridge.param_set(key, base + 0.13):
			continue
		await _frames(1)
		if abs(float(bridge.param_get(key)) - base) > 0.01:
			pk = key
			def_v = base
			break
		bridge.param_set(key, base)
	_ok("found a parameter that really moved off its default", pk != "", true)
	print("  info pinning %s (default %s, now %s)" % [pk, def_v, bridge.param_get(pk)])
	edit_pop.id_pressed.emit(DccMenus.ID_EDIT_RESET_PARAMS)
	await _frames(6)
	_ok("Reset generation parameters put it back",
		abs(float(bridge.param_get(pk)) - def_v) < 0.0001, true)

	print("")
	print("_fileedit_probe: ", "PASS" if _fail == 0 else str(_fail) + " FAILURE(S)")
	get_tree().quit(1 if _fail > 0 else 0)

func _count_windows(n: Node) -> int:
	var c := 0
	if n is Window and (n as Window).visible:
		c += 1
	for k in n.get_children(true):
		c += _count_windows(k)
	return c

func _accel_scan(p: PopupMenu, menu_name: String, seen: Dictionary) -> void:
	for i in p.item_count:
		if p.is_item_separator(i):
			continue
		var a := p.get_item_accelerator(i)
		if a != 0:
			var key := OS.get_keycode_string(a)
			if not seen.has(key):
				seen[key] = []
			seen[key].append("%s > %s" % [menu_name, p.get_item_text(i)])
		var sub := p.get_item_submenu(i)
		if sub != "":
			var node := p.get_node_or_null(NodePath(sub))
			if node is PopupMenu:
				await _accel_scan(node as PopupMenu, menu_name, seen)
