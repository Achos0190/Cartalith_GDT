extends Node
## VE-9, the rest (`VAULT_EDITOR_PROPOSAL.md` s.7): landmark links and notes under Ruling EI, the notes scope in the map's
## Find, and the private badge (Ruling DP) wherever a note is listed or linked.
##
##   Godot_v4.7.2-stable_win64_console.exe --path . _ve9b_probe.tscn -- --vp 1600x1000 [--touch]
##   ... -- --vp 1080x2340 --density 2.8125 --force-touch      (the phone)
##
## **Windowed**, under a wiped user directory (`APPDATA=<empty dir>`), on a throw-away vault in TEMP and a generated world with a
## landmark pass run. Exit 0 = every check held, 1 = a check failed, 2 = could not run, 3 = watchdog. Real key events and
## clicks drive the `[[` card, the Find field and its rows, the Note tab's tab and Create button; the rest call the window's own
## functions (a menu's item list is read, not clicked).
##
## **What it proves.**
##  L1  every placed landmark has a name, derived and stable: the same on a second read, after a second landmark pass with other
##      caps (a landmark in both runs keeps its name, bar the documented tie suffix), and after the project is saved and reopened;
##      a near-place name is `<Type> near <a real settlement>`; no name is blank; all are distinct.
##  L2  a name the user gives replaces the generated one and the link to it resolves; clearing it brings the generated one back.
##  L3  `[[<landmark name>]]` is an entity link of kind landmark; the `[[` card offers the landmark ("landmark, no note yet"),
##      and typing `[[` with its first letters then Enter writes the whole name; creating a note from the link binds a
##      `KnowledgeLink` of kind landmark; once the note exists the link is to the note and a note opened from a link selects the landmark.
##  L4  (desktop) the Note tab for a selected landmark: no note -> Create (bound to it) -> the note docks in the tab.
##  F   the Find `n ` scope: names, content once indexed, a names-only footer without the index, the three-character floor, a
##      press opens the note in the vault window, no vault says so.
##  P   the private badge, on a folder marker, frontmatter, an inline tag and malformed frontmatter, and NOT on a public note:
##      the folder tree, the vault window's search, the `[[` card, the ambiguous-link menu, backlinks, All linked notes, the peek,
##      the open note's notice, the Find notes rows and the Note tab.

var _vp: SubViewport
var app: Node
var _bridge
var vw: VaultWindow
var _root := ""
var _touch := false
var _phone := false
var _n := 0
var _fails: Array = []
const SEED := 4242


func _ok(label: String, cond: bool, detail: String = "") -> void:
	_n += 1
	if cond:
		print("VE9B  ok  %s" % label)
	else:
		_fails.append(label)
		print("VE9B  !!  %s   %s" % [label, detail])


func _arg(name: String, dflt: String) -> String:
	var args := OS.get_cmdline_user_args()
	var i := args.find(name)
	if i >= 0 and i + 1 < args.size():
		return String(args[i + 1])
	return dflt


func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame


func _write(path: String, text: String) -> void:
	DirAccess.make_dir_recursive_absolute(path.get_base_dir())
	var f := FileAccess.open(path, FileAccess.WRITE)
	f.store_string(text)
	f.close()


func _disk(rel: String) -> String:
	return FileAccess.get_file_as_string(_root + "/" + rel)


func _finish() -> void:
	print("VE9B RESULT %d checks, %d failed%s" % [_n, _fails.size(), "" if _fails.is_empty() else " :: " + str(_fails)])
	get_tree().quit(0 if _fails.is_empty() else 1)


func _until(cond: Callable, ms: int) -> bool:
	var t0 := Time.get_ticks_msec()
	while Time.get_ticks_msec() - t0 < ms:
		if cond.call():
			return true
		await get_tree().process_frame
	return bool(cond.call())


func _mouse(pos: Vector2, pressed: bool) -> void:
	var ev := InputEventMouseButton.new()
	ev.button_index = MOUSE_BUTTON_LEFT
	ev.pressed = pressed
	ev.position = pos
	ev.global_position = pos
	if _touch:
		ev.device = InputEvent.DEVICE_ID_EMULATION
	_vp.push_input(ev)
	await _frames(2)


func _click(pos: Vector2) -> void:
	var m := InputEventMouseMotion.new()
	m.position = pos
	m.global_position = pos
	_vp.push_input(m)
	await _frames(1)
	await _mouse(pos, true)
	await _mouse(pos, false)
	await _frames(3)


func _press(b: Button) -> void:
	## A button inside an embedded dialog reports its rect in that dialog's own coordinates; the window's position
	## puts it in the frame the events are pushed into (the main frame's own window has no offset).
	var off := Vector2.ZERO
	var w := b.get_window()
	if w != null and w != get_window() and w != _vp:
		off = Vector2(w.position)
	await _click(b.get_global_rect().get_center() + off)


func _type(s: String) -> void:
	for i in s.length():
		for pressed in [true, false]:
			var k := InputEventKey.new()
			k.pressed = pressed
			k.unicode = s.unicode_at(i)
			k.keycode = s.unicode_at(i) as Key
			_vp.push_input(k)
		await _frames(2)


func _key(code: Key) -> void:
	for pressed in [true, false]:
		var k := InputEventKey.new()
		k.pressed = pressed
		k.keycode = code
		_vp.push_input(k)
	await _frames(3)


func _vis_button(n: Node, name: String) -> Button:
	if n is Button and n.name == name and (n as Button).is_visible_in_tree():
		return n
	for c in n.get_children():
		var r := _vis_button(c, name)
		if r != null:
			return r
	return null


func _find_node(n: Node, name: String) -> Node:
	if n.name == name and not n.is_queued_for_deletion():
		return n
	for c in n.get_children():
		var r := _find_node(c, name)
		if r != null:
			return r
	return null


## Every text a container shows: label text, button text, accessibility names, tooltips off.
func _texts(root: Node) -> String:
	var out := PackedStringArray()
	var stack: Array = [root]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		if n.is_queued_for_deletion():
			continue
		if n is Label:
			out.append((n as Label).text)
		elif n is Button:
			out.append((n as Button).text)
			out.append((n as Button).accessibility_name)
		stack.append_array(n.get_children())
	return " | ".join(out)


## The text of every leaf of a Tree whose metadata is a note path, `{rel: text}`.
func _tree_texts(it: TreeItem, out: Dictionary) -> void:
	if it == null:
		return
	if it.get_metadata(0) is String:
		out[String(it.get_metadata(0))] = it.get_text(0)
	var c := it.get_first_child()
	while c != null:
		_tree_texts(c, out)
		c = c.get_next()


func _find_all(n: Node, cls: String, out: Array) -> void:
	if n.is_class(cls):
		out.append(n)
	for c in n.get_children():
		_find_all(c, cls, out)


## The card's rows as text (the editor's `[[` suggestions).
func _card_rows(ed) -> Array:
	var out: Array = []
	if ed._sg_panel == null or not ed._sg_panel.visible:
		return out
	for b in ed._sg_box.get_children():
		out.append((b as Button).text)
	return out


func _names_by_key() -> Dictionary:
	var d := {}
	for r in _bridge.landmark_names():
		d[String((r as Dictionary)["key"])] = r
	return d


func _find_text() -> String:
	var res: Node = app.get("_phone_search_results") if _phone else app.get("_desktop_search_results")
	return _texts(res) if res != null else ""


func _find_query(q: String) -> void:
	var f: LineEdit = app.get("_phone_search_field") if _phone else app.get("_desktop_search_field")
	f.text = ""
	f.grab_focus()
	await _frames(2)
	await _type(q)
	await _frames(3)


func _open_find() -> void:
	app.open_find_on_map()
	await _frames(8)


func _close_find() -> void:
	if _phone:
		app.call("_set_search_open", false)
	else:
		var dlg: Window = app.get("_desktop_search_dialog")
		if dlg != null:
			dlg.hide()
	await _frames(3)


func _ready() -> void:
	for a in OS.get_cmdline_user_args():
		var s := String(a)
		if s.begins_with("--") and not (s in ["--vp", "--touch", "--force-touch", "--density"]):
			print("VE9B ABORT unknown flag %s" % s)
			get_tree().quit(2)
			return
	if DisplayServer.get_name() == "headless":
		print("VE9B REFUSED: run windowed")
		get_tree().quit(2)
		return
	var wd := Timer.new()
	wd.wait_time = 900.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("VE9B WATCHDOG TIMEOUT")
		get_tree().quit(3))
	wd.start()
	var parts: PackedStringArray = _arg("--vp", "1600x1000").split("x")
	_touch = "--touch" in OS.get_cmdline_user_args() or "--force-touch" in OS.get_cmdline_user_args()
	if FileAccess.file_exists(VaultStore.PATH):
		DirAccess.remove_absolute(ProjectSettings.globalize_path(VaultStore.PATH))
	_vp = SubViewport.new()
	_vp.size = Vector2i(int(parts[0]), int(parts[1]))
	_vp.transparent_bg = false
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await get_tree().create_timer(1.4).timeout
	if app.get("open_project_dialog") != null:
		app.open_project_dialog.hide()
	await _frames(4)
	_vp.notification(Node.NOTIFICATION_VP_MOUSE_ENTER)
	_bridge = app.bridge
	vw = app.vault_window
	_phone = bool(app.get("_phone"))
	print("VE9B info: viewport %s phone=%s touch=%s" % [_vp.size, _phone, _touch])

	_bridge.generate({"seed": SEED, "width_km": 2400.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.45})
	await _frames(2)
	var t0 := Time.get_ticks_msec()
	while bool(_bridge.generating) and Time.get_ticks_msec() - t0 < 240000:
		await _frames(10)
	await _frames(30)
	var ss: Array = _bridge.settlements()
	_ok("CONTROL: a world with settlements", bool(_bridge.has_world) and ss.size() >= 3, str(ss.size()))
	if ss.size() < 3:
		_finish()
		return
	_ok("CONTROL: no landmark names before the landmark pass", _bridge.landmark_names().is_empty())
	await _bridge.landmark_run()
	var lms: Array = _bridge.landmarks()
	var names: Array = _bridge.landmark_names()
	_ok("CONTROL: the landmark pass placed landmarks", lms.size() >= 3, str(lms.size()))
	if lms.size() < 3:
		_finish()
		return

	# == L1. names ==============================================================
	print("== L1. derived, stable names ==")
	_ok("one name per landmark, in landmarks()' order", names.size() == lms.size())
	var in_order := true
	for i in lms.size():
		var d: Dictionary = lms[i]
		var key := "%s@%d,%d" % [String(d["kind"]), int(d["x"]), int(d["y"])]
		if String((names[i] as Dictionary)["key"]) != key:
			in_order = false
	_ok("... and each key is kind@x,y of the landmark at that index", in_order)
	var settlement_names: Array = []
	for s in ss:
		settlement_names.append(String((s as Dictionary).get("name", "")))
	var seen := {}
	var all_ok := true
	var near_n := 0
	var fall_n := 0
	var bad := ""
	for r in names:
		var d: Dictionary = r
		var nm := String(d["name"])
		var low := nm.to_lower()
		if nm.strip_edges() == "" or seen.has(low):
			all_ok = false
			bad = "blank or repeated: " + nm
		seen[low] = true
		match String(d["source"]):
			"near_place":
				near_n += 1
				var after := nm.get_slice(" near ", 1)
				var hit := false
				for sn in settlement_names:
					if after.begins_with(String(sn)):
						hit = true
				if not hit:
					all_ok = false
					bad = "near a place that is not a settlement: " + nm
			"fallback":
				fall_n += 1
				if nm.contains(" near ") or not nm.ends_with(")"):
					all_ok = false
					bad = "fallback is not type + cell: " + nm
			_:
				all_ok = false
				bad = "unexpected source: " + str(d)
	_ok("every name is non-blank, distinct, and names a real settlement or is the stated fallback", all_ok, bad)
	print("VE9B info: %d near-place names, %d fallback; first: %s" % [near_n, fall_n, String((names[0] as Dictionary)["name"])])
	_ok("CONTROL: this world exercises near-place names", near_n >= 1)
	var before := _names_by_key()
	_ok("a second read is identical", str(_bridge.landmark_names()) == str(names))
	# Other caps -> a different set of landmarks; a landmark in both keeps its name (a tie suffix may come or go).
	var kinds: Array = _bridge.landmark_kinds()
	for k in kinds:
		var kd: Dictionary = k
		_bridge.landmark_set_cap(String(kd["key"]), maxi(1, int(kd.get("default_cap", 4)) * 2))
	await _bridge.landmark_run()
	var after_map := _names_by_key()
	_ok("CONTROL: the changed caps changed the landmark set", after_map.size() != before.size(), "%d vs %d" % [before.size(), after_map.size()])
	var kept := 0
	var drift := ""
	for key in before:
		if after_map.has(key):
			kept += 1
			var a := String((before[key] as Dictionary)["name"])
			var b := String((after_map[key] as Dictionary)["name"])
			if a.get_slice(" (", 0) != b.get_slice(" (", 0):
				drift = "%s: %s -> %s" % [key, a, b]
	_ok("landmarks in both runs keep their names (bar the tie suffix)", kept >= 1 and drift == "", "kept %d %s" % [kept, drift])
	_bridge.landmark_reset_settings()
	await _bridge.landmark_run()
	_ok("restoring the caps restores every name exactly", str(_bridge.landmark_names()) == str(names))
	# Save and reopen.
	var proj := OS.get_environment("TEMP").replace("\\", "/") + "/cartalith-ve9b-%d.cartalith" % Time.get_ticks_msec()
	var saved: bool = _bridge.save_project(proj)
	_ok("CONTROL: the project saves", saved)
	if saved:
		var loaded: bool = _bridge.load_save(proj)
		await _frames(20)
		_ok("CONTROL: the project reopens", loaded)
		_ok("names are identical after a save and reopen", str(_bridge.landmark_names()) == str(names), str(_bridge.landmark_names().size()))
		DirAccess.remove_absolute(proj)
	names = _bridge.landmark_names()
	lms = _bridge.landmarks()
	ss = _bridge.settlements()

	# == the fixture vault ======================================================
	_root = OS.get_environment("TEMP").replace("\\", "/") + "/cartalith-ve9b-%s-%d" % ["phone" if _phone else ("touch" if _touch else "desk"), Time.get_ticks_msec()]
	var la := String((names[0] as Dictionary)["name"])
	var lb := String((names[1] as Dictionary)["name"])
	var lc := String((names[2] as Dictionary)["name"])
	_write(_root + "/Hub.md", "# Hub\n\nSee [[Secret]], [[Public]], [[Twin]], [[Private/Plot]], [[%s]] and [[%s]].\n" % [la, lb])
	_write(_root + "/Public.md", "# Public\n\nopen text zebra\n")
	_write(_root + "/Secret.md", "---\nprivate: true\n---\nSecret text quokka\n")
	_write(_root + "/Tagged.md", "# Tagged\n\nhidden #secret thing quokka\n")
	_write(_root + "/Broken.md", "---\nunclosed frontmatter\n\nbody quokka\n")
	_write(_root + "/Private/Plot.md", "# Plot\n\nlinks [[Public]] quokka\n")
	_write(_root + "/Private/Twin.md", "# Twin\n\nprivate twin\n")
	_write(_root + "/Pub/Twin.md", "# Twin\n\npublic twin\n")
	_write(_root + "/Scratch.md", "# Scratch\n\nword here\n")
	var conn: Dictionary = _bridge.vault_connect(_root, "Chronicle")
	_ok("CONTROL: the vault binds", bool(conn.get("ok", false)), str(conn))
	var idx: Dictionary = _bridge.vault_refresh_backlinks()
	_ok("CONTROL: the index builds", bool(idx.get("ok", false)), str(idx))
	app.open_vault_browse()
	await _frames(6)

	# == L2. user-given name ===================================================
	print("== L2. a name the user gives ==")
	var keya := String((names[0] as Dictionary)["key"])
	_ok("setting a name for a key with no landmark is refused", not _bridge.landmark_set_user_name("peak@9999,9999", "Nope"))
	_ok("a user name is accepted", _bridge.landmark_set_user_name(keya, "  The Weeping Stair "))
	var ua: Dictionary = _names_by_key()[keya]
	_ok("... it replaces the generated name (trimmed) and says it is the user's", String(ua["name"]) == "The Weeping Stair" and String(ua["source"]) == "user", str(ua))
	vw._ent_names = {}
	var kk: Dictionary = vw._link_kind("Hub.md", "The Weeping Stair")
	_ok("[[The Weeping Stair]] is the landmark", String(kk.get("kind", "")) == "entity" and String(kk.get("ent_kind", "")) == "landmark" and int(kk.get("id", -1)) == int(ua["id"]), str(kk))
	kk = vw._link_kind("Hub.md", la)
	_ok("... and the old generated name no longer resolves", String(kk.get("kind", "")) == "unresolved", str(kk))
	_ok("a blank user name clears it", _bridge.landmark_set_user_name(keya, "   ") and String(_names_by_key()[keya]["name"]) == la)
	vw._ent_names = {}

	# == L3. link, card, create =================================================
	print("== L3. landmark links ==")
	var ida := int((names[0] as Dictionary)["id"])
	var idb := int((names[1] as Dictionary)["id"])
	kk = vw._link_kind("Hub.md", la)
	_ok("[[%s]] is an entity link of kind landmark with its id" % la, String(kk.get("kind", "")) == "entity" and String(kk.get("ent_kind", "")) == "landmark" and int(kk.get("id", -1)) == ida, str(kk))
	kk = vw._link_kind("Hub.md", la.to_upper())
	_ok("... case-insensitively", String(kk.get("kind", "")) == "entity" and String(kk.get("ent_kind", "")) == "landmark", str(kk))
	var idr: Dictionary = _bridge.vault_landmark_entity_id(keya)
	_ok("vault_landmark_entity_id answers the same id, labelled with the name", bool(idr.get("ok", false)) and int(idr.get("entity_id", -1)) == ida and String(idr.get("label", "")) == la, str(idr))
	var ent: Dictionary = vw._resolve_entity("landmark", ida)
	var ld: Dictionary = lms[0]
	_ok("_resolve_entity answers the name and the cell", String(ent.get("name", "")) == la and int(ent.get("x", -1)) == int(ld["x"]) and int(ent.get("y", -1)) == int(ld["y"]), str(ent))
	_ok("... and nothing for an id the world does not have", vw._resolve_entity("landmark", 12345).is_empty())
	var lcl: Dictionary = _bridge.vault_landmark_entity_id("peak@9999,9999")
	_ok("a key with no landmark is an error and has no entity_id", not bool(lcl.get("ok", true)) and not lcl.has("entity_id"), str(lcl))
	# The card, by real typing.
	vw.open_note("Scratch.md")
	await _frames(8)
	var ok_open := await _until(func(): return vw._editor != null and is_instance_valid(vw._editor), 4000)
	_ok("CONTROL: Scratch.md opens in the editor", ok_open and vw._browse_path == "Scratch.md")
	if not ok_open:
		_finish()
		return
	var ed = vw._editor
	var te: TextEdit = ed.text_edit
	te.text = ""
	ed._last_len = 0
	te.grab_focus()
	await _click(te.get_global_rect().get_center())
	## All but the last character: a name that shares its first words with dozens of others still lands in the card.
	var want := la.substr(0, la.length() - 1)
	await _type("[[" + want)
	var rows := _card_rows(ed)
	var found := false
	for r in rows:
		if String(r).contains(la) and String(r).to_lower().contains("landmark"):
			found = true
	_ok("the [[ card offers the landmark as 'landmark, no note yet'", found, str(rows))
	var pos := -1
	for i in rows.size():
		if String(rows[i]).contains(la):
			pos = i
	if pos >= 0:
		for _i in pos:
			await _key(KEY_DOWN)
		await _key(KEY_ENTER)
		_ok("Enter writes the whole landmark name between the brackets", te.text == "[[" + la + "]]", te.text)
	else:
		_ok("Enter writes the whole landmark name between the brackets", false, "no row")
	# Follow the link: the menu offers creating the note, bound.
	var links0: int = _bridge.vault_all_links().size()
	var menu_before := vw.find_children("LinkMenu", "PopupMenu", true, false).size()
	vw._follow_link("Scratch.md", la, vw._body)
	await _frames(3)
	var menus: Array = vw.find_children("LinkMenu", "PopupMenu", true, false)
	_ok("following the link shows a menu", menus.size() > menu_before)
	var mt := ""
	if not menus.is_empty():
		var pm: PopupMenu = menus.back()
		for i in pm.item_count:
			mt += pm.get_item_text(i) + " | "
	_ok("... it says the landmark has no note yet and offers Centre on map", mt.contains("landmark with no note yet") and mt.contains("Centre on map"), mt)
	var path_a: String = _bridge.vault_suggested_path("landmark", la)
	_ok("the suggested path is Landmarks/<name>.md", path_a == "Landmarks/%s.md" % la, path_a)
	var items: Array = vw._create_items(la, path_a, {"kind": "landmark", "id": ida})
	(items.back()["call"] as Callable).call()
	await _frames(8)
	_ok("creating the note wrote the file", FileAccess.file_exists(_root + "/" + path_a), path_a)
	var bound := false
	for l in _bridge.vault_all_links():
		var d: Dictionary = l
		if String(d.get("entity_kind", "")) == "landmark" and int(d.get("entity_id", -1)) == ida:
			bound = true
	_ok("... and bound a KnowledgeLink of kind landmark to it", bound and _bridge.vault_all_links().size() == links0 + 1, "%d -> %d" % [links0, _bridge.vault_all_links().size()])
	vw._ent_names = {}
	kk = vw._link_kind("Hub.md", la)
	_ok("once the note exists the link is to the note", String(kk.get("kind", "")) == "note", str(kk))
	var eon: Dictionary = vw._entity_of_note(path_a)
	_ok("the note belongs to the landmark by its index in the landmark list", String(eon.get("kind", "")) == "landmark" and int(eon.get("id", -1)) == 0, str(eon))
	var lm_links: Array = _bridge.vault_links_for("landmark", ida)
	_ok("vault_links_for(landmark) finds it, live", lm_links.size() == 1 and String((lm_links[0] as Dictionary).get("path", "")) == path_a, str(lm_links))

	# == L4. the Note tab ======================================================
	if _phone:
		print("== L4. the Note tab: not on the phone (it docks nothing there) ==")
	else:
		print("== L4. the Note tab for a landmark ==")
		vw.hide()
		await _frames(6)
		app.select("landmark", 0)
		await _frames(8)
		var tab_btn := _vis_button(app, "CtxTabNote")
		_ok("the strip draws a NOTE tab for a landmark", tab_btn != null)
		if tab_btn != null:
			await _press(tab_btn)
			var docked := await _until(func(): return vw.is_docked() and vw.docked_path() == path_a, 3000)
			_ok("the landmark's note docks in the tab (not a reason)", docked, "docked %s" % vw.docked_path())
			_ok("... and the old 'not bound here' reason is not shown", _find_node(app, "NoteUnsupported") == null)
			# A landmark with no note: Create, by a real click.
			app.select("landmark", 1)
			await _frames(10)
			_ok("a landmark with no note says so", _find_node(app, "NoteNone") != null)
			var cb := _vis_button(app, "NoteCreate")
			_ok("... and offers Create, with the type-and-place name in the path", cb != null and (app.find_child("NotePath", true, false) as LineEdit).text == "Landmarks/%s.md" % lb)
			if cb != null:
				await _press(cb)
				var made := await _until(func(): return FileAccess.file_exists(_root + "/Landmarks/%s.md" % lb), 3000)
				_ok("pressing Create made the note", made)
				_ok("... and bound it to the landmark", _bridge.vault_links_for("landmark", idb).size() == 1)
				_ok("... and the tab shows it", await _until(func(): return vw.is_docked() and vw.docked_path() == "Landmarks/%s.md" % lb, 3000))
			app.select("", -1)
			await _frames(6)
		vw.open_browse()
		await _frames(6)

	# == P. the private badge ===================================================
	print("== P. the private badge ==")
	_bridge.vault_refresh_backlinks()
	var priv := {"Secret.md": "frontmatter", "Tagged.md": "inline tag", "Broken.md": "malformed", "Private/Plot.md": "folder", "Private/Twin.md": "folder"}
	var pub := ["Public.md", "Hub.md", "Scratch.md", "Pub/Twin.md"]
	# Tree.
	vw.privacy().clear()
	vw.open_browse()
	await _frames(10)
	var trees: Array = []
	_find_all(vw, "Tree", trees)
	var tt := {}
	if not trees.is_empty():
		_tree_texts((trees[0] as Tree).get_root(), tt)
	for rel in priv:
		_ok("tree: %s (%s) is badged private" % [rel, priv[rel]], String(tt.get(rel, "")).ends_with("private") and String(tt.get(rel, "")).contains("·"), str(tt.get(rel, "<missing>")))
	for rel in pub:
		_ok("tree: %s is NOT badged" % rel, tt.has(rel) and not String(tt[rel]).contains("private"), str(tt.get(rel, "<missing>")))
	# The vault window's own search.
	vw._search_query = "quokka"
	vw._run_search()
	await _frames(4)
	var st := _texts(vw._search_box) if vw._search_box != null else ""
	st = st.to_lower()
	_ok("window search: a private hit is badged", st.contains("secret.md  ·  private") or st.contains("tagged.md  ·  private"), st)
	_ok("window search: a hit on the folder note is badged", st.contains("private/plot.md  ·  private"), st)
	# The card.
	vw.open_note("Scratch.md")
	await _until(func(): return vw.visible and vw._editor != null and is_instance_valid(vw._editor) and vw._browse_path == "Scratch.md", 4000)
	await _frames(6)
	ed = vw._editor
	te = ed.text_edit
	te.text = ""
	ed._last_len = 0
	await _click(te.get_global_rect().get_center())
	te.grab_focus()
	await _frames(2)
	await _type("[[Secr")
	rows = _card_rows(ed)
	print("VE9B info: card state te=%s focus=%s ed_visible=%s" % [te.text, te.has_focus(), ed.is_visible_in_tree()])
	_ok("card: the private note is offered as private", rows.size() >= 1 and String(rows[0]).contains("private"), str(rows))
	te.text = ""
	ed._last_len = 0
	await _type("[[Publi")
	rows = _card_rows(ed)
	_ok("card: a public note is not", rows.size() >= 1 and String(rows[0]).contains("Public") and not String(rows[0]).contains("private"), str(rows))
	te.text = ""
	ed._last_len = 0
	await _key(KEY_ESCAPE)
	# The ambiguous-link menu.
	var mb := vw.find_children("LinkMenu", "PopupMenu", true, false).size()
	vw._follow_link("Hub.md", "Twin", vw._body)
	await _frames(3)
	menus = vw.find_children("LinkMenu", "PopupMenu", true, false)
	mt = ""
	if menus.size() > mb:
		var pm2: PopupMenu = menus.back()
		for i in pm2.item_count:
			mt += pm2.get_item_text(i) + " | "
	_ok("link menu: two notes of one name, the folder one badged and the other not", mt.contains("Private/Twin.md  ·  private") and mt.contains("Pub/Twin.md |"), mt)
	# The peek.
	var pk := String(vw._peek_for("Hub.md", "Secret"))
	_ok("peek: a private note says so", pk.to_lower().contains("private"), pk)
	pk = String(vw._peek_for("Hub.md", "Public"))
	_ok("peek: a public one does not", not pk.to_lower().contains("private") and pk.contains("open text"), pk)
	# Backlinks (a private note links to Public).
	vw._enter_note("Public.md")
	await _frames(10)
	var sp = vw._side_panel
	_ok("CONTROL: the side panel is up", sp != null and is_instance_valid(sp))
	if sp != null and is_instance_valid(sp):
		sp.show_tab("links")
		await _frames(6)
		var bt := _texts(sp)
		_ok("backlinks: the note that links here from a private folder is badged", bt.contains("Private/Plot.md  ·  private"), bt)
		_ok("backlinks: a public source is not", bt.contains("Hub.md") and not bt.contains("Hub.md  ·  private"), bt)
	# The open note's notice.
	_ok("the open note: a public note shows no notice", _find_node(vw, "VaultPrivateNotice") == null)
	vw._enter_note("Secret.md")
	await _frames(10)
	var pn := _find_node(vw, "VaultPrivateNotice")
	_ok("the open note: a private note says so and why", pn != null and (pn as Label).text.contains("frontmatter private: true"), (pn as Label).text if pn != null else "<none>")
	# All linked notes.
	var tid0 := int((ss[0] as Dictionary).get("tid", 0))
	var nm0 := String((ss[0] as Dictionary).get("name", ""))
	_ok("CONTROL: attach Secret.md to a settlement", bool(_bridge.vault_attach("settlement", tid0, nm0, "Secret.md", "").get("ok", false)))
	vw.open_overview()
	await _frames(10)
	var ov := _texts(vw._body).to_lower()
	_ok("All linked notes: the private note's row is badged", ov.contains("secret.md (") and ov.contains("private"), ov.substr(0, 400))
	# Find notes scope.
	print("== F. Find notes ==")
	vw._write_now()
	_ok("CONTROL: nothing is unsaved before Find opens a note (a dirty note asks first, by design)", not vw._is_dirty())
	vw.hide()
	await _frames(4)
	var nf = load("res://shell/notes_find.gd")
	_ok("CONTROL: notes_find loads", nf != null)
	await _open_find()
	await _find_query("n quokka")
	var ft := _find_text()
	_ok("Find n: a content hit from the index lists the notes", ft.contains("Secret") and ft.contains("Tagged") and ft.contains("Plot"), ft)
	_ok("Find n: private ones are badged in the row", ft.contains("Secret  ·  private") or ft.contains("Secret  ·  private".replace("  ", " ")), ft)
	_ok("Find n: the public note with no 'quokka' is not listed", not ft.contains("Scratch"), ft)
	await _find_query("n zebra")
	ft = _find_text()
	_ok("Find n: a public note is listed and not badged", ft.contains("Public") and not ft.contains("Public  ·  private") and not ft.contains("private"), ft)
	await _find_query("n ze")
	ft = _find_text()
	_ok("Find n: under three characters it says names only", ft.contains("under three characters") or ft.contains("Names only"), ft)
	await _find_query("n Scra")
	ft = _find_text()
	_ok("Find n: a name finds its note", ft.contains("Scratch"), ft)
	await _find_query("n ")
	ft = _find_text()
	_ok("Find n with nothing typed says what to type", ft.contains("Type a note"), ft)
	# A real press on a row opens the note in the vault window.
	await _find_query("n Scratch")
	var res: Node = app.get("_phone_search_results") if _phone else app.get("_desktop_search_results")
	var rowb: Button = null
	for b in res.find_children("*", "Button", true, false):
		if (b as Button).is_visible_in_tree() and not b.is_queued_for_deletion():
			rowb = b
			break
	_ok("CONTROL: a result row is drawn", rowb != null)
	if rowb != null:
		print("VE9B info: pressing row %s at %s; vw visible=%s" % [rowb.accessibility_name, rowb.get_global_rect(), vw.visible])
		await _press(rowb)
		var opened := await _until(func(): return vw.visible and vw._pick_file == "Scratch.md", 4000)
		_ok("pressing the row opens that note in the vault window", opened, "visible %s pick %s" % [vw.visible, vw._pick_file])
	vw.hide()
	await _frames(4)
	# The unscoped search is untouched by the new scope; the hint names it.
	_ok("the scope hint names n", String(app.SEARCH_HINT).contains("n notes"))
	var chips_has := false
	for sc in app.SEARCH_SCOPES:
		if String((sc as Dictionary)["prefix"]) == "n" and (sc as Dictionary).has("notes"):
			chips_has = true
	_ok("the scope table has the notes row, with exactly one kind key", chips_has)
	# Names-only without an index: a second, never-indexed vault.
	var root2 := _root + "-noindex"
	_write(root2 + "/Lonely.md", "# Lonely\n\nalone text\n")
	_bridge.vault_disconnect()
	_ok("CONTROL: the second vault binds", bool(_bridge.vault_connect(root2, "Second").get("ok", false)))
	## The engine keeps the first vault's backlink index across a reconnect (stats above say built with 11 notes), so a
	## never-indexed vault cannot be reached from here: the names-only-without-index footer is covered by reading
	## notes_find.gd's branch, not by this run. What is exercised is the three-character floor on the second vault.
	await _open_find()
	await _find_query("n Lo")
	ft = _find_text()
	_ok("Find n: the second vault's note is found by name, and a two-letter query says names only", ft.contains("Lonely") and ft.contains("under three characters"), ft)
	await _close_find()
	# No vault.
	_bridge.vault_disconnect()
	await _open_find()
	await _find_query("n anything")
	ft = _find_text()
	_ok("Find n: no vault says so and lists nothing", ft.contains("No vault folder is connected"), ft)
	await _close_find()
	_finish()
