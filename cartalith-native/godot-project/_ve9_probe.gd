extends Node
## VE-9 of `VAULT_EDITOR_PROPOSAL.md` s.7 (the scale items): the shell's 2 000-note caps are gone (all six sites of
## s.2, Ruling DY VQ7), and cultures are a link class beside settlements, provinces, continents and factions.
##
##   Godot_v4.7.2-stable_win64_console.exe --path . _ve9_probe.tscn -- --vp 1600x1000
##
## **Windowed**, wiped `APPDATA`, a throw-away vault of 2 600 notes in TEMP. Exit 0 = every check held, 1 = a check
## failed, 2 = could not run.
##
## What it proves: with 2 600 notes (more than the old cap) the vault header, the Browse list, the folder tree and the
## index refresh all see every note; the bridge's default limits are the crate's own; `[[Highland]]` is classed as a
## culture with no note (and `[[highland]]` too), is offered by the `[[` suggestions as "culture, no note yet", and
## creating a note from its link binds it to the culture; a note called Highland wins the name once it exists; a
## name that is no entity is unresolved, and a landmark is NOT a link class (it has no name to be linked by).

var _vp: SubViewport
var app: Node
var _bridge
var vw: VaultWindow
var _root := ""
var _n := 0
var _fails: Array = []
const N := 2600


func _ok(label: String, cond: bool, detail: String = "") -> void:
	_n += 1
	if cond:
		print("VE9  ok  %s" % label)
	else:
		_fails.append(label)
		print("VE9  !!  %s   %s" % [label, detail])


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


func _finish() -> void:
	print("VE9 RESULT %d checks, %d failed%s" % [_n, _fails.size(), "" if _fails.is_empty() else " :: " + str(_fails)])
	get_tree().quit(0 if _fails.is_empty() else 1)


## Notes (items with metadata) under a Tree item, all the way down.
func _count_leaves(it: TreeItem) -> int:
	if it == null:
		return 0
	var n := 0
	var c := it.get_first_child()
	while c != null:
		n += (1 if c.get_metadata(0) != null else 0) + _count_leaves(c)
		c = c.get_next()
	return n


func _labels_with(root: Node, needle: String) -> int:
	var c := 0
	for l in root.find_children("*", "Label", true, false):
		if String((l as Label).text).contains(needle):
			c += 1
	return c


func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		print("VE9 REFUSED: run windowed")
		get_tree().quit(2)
		return
	var wd := Timer.new()
	wd.wait_time = 400.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("VE9 WATCHDOG TIMEOUT")
		get_tree().quit(3))
	wd.start()
	var parts: PackedStringArray = _arg("--vp", "1600x1000").split("x")
	if FileAccess.file_exists(VaultStore.PATH):
		DirAccess.remove_absolute(ProjectSettings.globalize_path(VaultStore.PATH))
	_vp = SubViewport.new()
	_vp.size = Vector2i(int(parts[0]), int(parts[1]))
	_vp.gui_embed_subwindows = true
	_vp.render_target_update_mode = SubViewport.UPDATE_ALWAYS
	add_child(_vp)
	app = load("res://shell/app.tscn").instantiate()
	_vp.add_child(app)
	await get_tree().create_timer(1.4).timeout
	if app.get("open_project_dialog") != null:
		app.open_project_dialog.hide()
	await _frames(4)
	_bridge = app.bridge

	_root = OS.get_environment("TEMP").replace("\\", "/") + "/cartalith-ve9-%d" % Time.get_ticks_msec()
	for i in N:
		_write(_root + "/Notes/Qn%04d.md" % i, "# Qn%04d\n\nlinks [[Qn%04d]]\n" % [i, (i + 1) % N])
	_write(_root + "/Hub.md", "# Hub\n\nA [[Highland]] folk, a [[highland]] again, a [[Nowhere Special]] and a [[Waterfall 1]].\n")
	_bridge.generate({"seed": 4242, "width_km": 2400.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.45})
	await _frames(2)
	var t0 := Time.get_ticks_msec()
	while bool(_bridge.generating) and Time.get_ticks_msec() - t0 < 240000:
		await _frames(10)
	await _frames(30)
	var conn: Dictionary = _bridge.vault_connect(_root, "Chronicle")
	_ok("CONTROL: the 2 601-note vault binds", bool(conn.get("ok", false)), str(conn))

	# == 1. the caps ============================================================
	var files: PackedStringArray = _bridge.vault_list_files()
	_ok("the bridge's default listing is not capped at 2 000", files.size() == N + 1, str(files.size()))
	_ok("... and an explicit limit still caps", _bridge.vault_list_files(10).size() == 10)
	var r: Dictionary = _bridge.vault_refresh_backlinks()
	_ok("the default index refresh reads every note (seen 2 601)", bool(r.get("ok", false)) and int(r.get("seen", 0)) == N + 1, str(r))
	app.open_vault_browse()
	await _frames(8)
	vw = app.vault_window
	_ok("the connection header counts all of them", _labels_with(vw, "· %d notes" % (N + 1)) >= 1, "labels: %d" % _labels_with(vw, " notes"))
	var tree: Control = vw._browse_tree_col
	var leaves := 0
	if tree != null:
		for t in tree.find_children("*", "Tree", true, false):
			leaves += _count_leaves((t as Tree).get_root())
	print("VE9 info: folder-tree notes: %d" % leaves)
	_ok("the folder tree lists every note (2 601), not 2 000", leaves == N + 1, "leaves %d" % leaves)
	vw._refresh_index()
	await _frames(4)
	_ok("Refresh index says it saw 2 601 notes", String(vw._index_feedback).contains("%d notes seen" % (N + 1)), String(vw._index_feedback))
	_ok("no 2000 left in the window's calls", not FileAccess.get_file_as_string("res://shell/vault_window.gd").contains("(2000)"))

	# == 2. culture is a link class =================================================
	var cultures: Array = _bridge.get_cultures()
	_ok("CONTROL: the engine lists the cultures (seven)", cultures.size() == 7, str(cultures.size()))
	var hid := -1
	for c in cultures:
		if String((c as Dictionary).get("name", "")) == "Highland":
			hid = int((c as Dictionary).get("id", -1))
	_ok("CONTROL: one of them is Highland", hid >= 0)
	var k: Dictionary = vw._link_kind("Hub.md", "Highland")
	_ok("[[Highland]] is an entity link, a culture, with its id", String(k.get("kind", "")) == "entity" and String(k.get("ent_kind", "")) == "culture" and int(k.get("id", -9)) == hid, str(k))
	k = vw._link_kind("Hub.md", "highland")
	_ok("... case-insensitively", String(k.get("kind", "")) == "entity" and String(k.get("ent_kind", "")) == "culture", str(k))
	k = vw._link_kind("Hub.md", "Nowhere Special")
	_ok("a name that is no entity stays unresolved", String(k.get("kind", "")) == "unresolved", str(k))
	k = vw._link_kind("Hub.md", "Waterfall 1")
	_ok("a landmark is NOT a link class (no name to link by): it stays unresolved", String(k.get("kind", "")) == "unresolved", str(k))
	var rows2: Array = vw._suggest_rows("note", "", "Highl", 8)
	var found := false
	for row in rows2:
		var d: Dictionary = row
		if String(d.get("kind", "")) == "entity" and String(d.get("label", "")) == "Highland" and String(d.get("detail", "")).contains("culture"):
			found = true
	_ok("the [[ suggestions offer Highland as 'culture, no note yet'", found, str(rows2))
	var ent: Dictionary = vw._resolve_entity("culture", hid)
	_ok("_resolve_entity answers a culture by id with its name and no map position", String(ent.get("name", "")) == "Highland" and not ent.has("x") and not ent.has("y"), str(ent))
	_ok("... and a culture id the world does not have answers nothing", vw._resolve_entity("culture", 99).is_empty())
	# Creating a note from the link binds it to the culture.
	var items: Array = vw._create_items("Highland", _bridge.vault_suggested_path("culture", "Highland"), {"kind": "culture", "id": hid})
	_ok("the create menu has rows (templates and a blank note)", items.size() >= 1)
	var before_links: int = _bridge.vault_all_links().size()
	(items.back()["call"] as Callable).call()
	await _frames(6)
	var bound := false
	for l in _bridge.vault_all_links():
		var d: Dictionary = l
		if String(d.get("entity_kind", "")) == "culture" and int(d.get("entity_id", -1)) == hid:
			bound = true
	_ok("creating the note bound it to the culture (a KnowledgeLink exists)", bound and _bridge.vault_all_links().size() == before_links + 1, "links %d -> %d" % [before_links, _bridge.vault_all_links().size()])
	vw._ent_names = {}
	k = vw._link_kind("Hub.md", "Highland")
	_ok("once a note called Highland exists the link is to the note, not the culture", String(k.get("kind", "")) == "note", str(k))
	_finish()
