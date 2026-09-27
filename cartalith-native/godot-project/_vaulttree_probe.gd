extends Node
## Windowed verification for the Markdown Vault browser's tree+preview split
## (owner request, 2026-09-21) -- `vault_window.gd::_build_browse`,
## `_build_browse_tree` and `_build_browse_preview`.
##
## Run: godot4 --path . _vaulttree_probe.tscn   (WINDOWED -- no --headless;
## `_vaultbrowse_probe.gd`'s own header explains why.)
##
##   1. Boot, generate a small world (a settlement is needed to prove the
##      entity-scoped Attach flow, unchanged, at the end).
##   2. A real scratch vault with real folder structure: two subfolders and a
##      root-level file, one note carrying frontmatter and headings.
##   3. `open_vault_browse()`: confirm a `Tree` renders, and that it holds the
##      real folders/files split out of `vault_list_files()` -- not a fixture,
##      the same flat list the bridge actually returned.
##   4. Expand/collapse a folder TreeItem.
##   5. Select a file item and confirm the preview's frontmatter chips,
##      heading outline and excerpt match direct `vault_file_data` /
##      `vault_file_headings` / `vault_read_file_for_edit` calls for the same
##      file -- cross-checked, not eyeballed.
##   6. Press "Open to edit" and confirm it reaches the real editor for that
##      same file.
##   7. Confirm the entity-scoped Attach flow (`open_vault`, a different call
##      than `open_vault_browse`) still attaches: since Ruling BE (2026-09-27)
##      it is the same tree, with an "Attach to <entity>" button on the
##      picked note, and a real `vault_attach` behind it.

var _app: Node
var _fails: Array = []
const SEED := 40217
const NOTE_A := "Towns/Nareth.md"
const NOTE_B := "Towns/Old/Ferry.md"
const NOTE_ROOT := "root.md"
const NOTE_A_TEXT := "---\ntype: town\npopulation: 400\n---\n\n# Nareth\n\n## History\n\nFounded at the ford.\n\n## Trade\n\nRiver barges.\n\nA river town at the third ford, written by hand for the tree probe.\n"


func _ok(label: String, cond: bool, detail: String = "") -> void:
	if cond:
		print("VTREE    OK  %s" % label)
	else:
		_fails.append(label)
		print("VTREE    !!  %s   %s" % [label, detail])


func _write(path: String, text: String) -> void:
	DirAccess.make_dir_recursive_absolute(path.get_base_dir())
	var f := FileAccess.open(path, FileAccess.WRITE)
	f.store_string(text)
	f.close()


func _find(n: Node, cls) -> Node:
	if is_instance_of(n, cls):
		return n
	for c in n.get_children():
		var r := _find(c, cls)
		if r != null:
			return r
	return null


func _find_all(n: Node, cls, out: Array) -> void:
	if is_instance_of(n, cls):
		out.append(n)
	for c in n.get_children():
		_find_all(c, cls, out)


func _find_button(n: Node, text: String) -> Button:
	if n is Button and String((n as Button).text) == text:
		return n
	for c in n.get_children():
		var r := _find_button(c, text)
		if r != null:
			return r
	return null


## Every TreeItem under `root`, depth-first -- `Tree`'s own children API is on
## `TreeItem`, not on the `Tree` node, so this walk is the probe's equivalent
## of `_find_all` for the one control this project has never used before.
func _collect_items(item: TreeItem, out: Array) -> void:
	if item == null:
		return
	var c := item.get_first_child()
	while c != null:
		out.append(c)
		_collect_items(c, out)
		c = c.get_next()


func _find_item_by_text(root: TreeItem, text: String) -> TreeItem:
	var all: Array = []
	_collect_items(root, all)
	for it in all:
		if String((it as TreeItem).get_text(0)) == text:
			return it
	return null


func _find_item_by_metadata(root: TreeItem, rel: String) -> TreeItem:
	var all: Array = []
	_collect_items(root, all)
	for it in all:
		var md: Variant = (it as TreeItem).get_metadata(0)
		if typeof(md) == TYPE_STRING and String(md) == rel:
			return it
	return null


func _shot(name: String) -> void:
	await get_tree().process_frame
	var img := get_viewport().get_texture().get_image()
	var path := "res://%s.png" % name
	img.save_png(path)
	print("VTREE    shot: %s (%dx%d)" % [path, img.get_width(), img.get_height()])


func _ready() -> void:
	if DisplayServer.get_name() == "headless":
		push_error("_vaulttree_probe: run WINDOWED (no --headless) -- the dummy "
			+ "rasteriser returns a null viewport image, so screenshots would pass vacuously.")
		print("PROBE REFUSED: headless")
		get_tree().quit(2)
		return

	var wd := Timer.new()
	wd.wait_time = 300.0
	wd.one_shot = true
	add_child(wd)
	wd.timeout.connect(func():
		print("WATCHDOG TIMEOUT")
		get_tree().quit(3))
	wd.start()

	_app = load("res://shell/app.tscn").instantiate()
	add_child(_app)
	await get_tree().create_timer(1.0).timeout
	_app.open_project_dialog.hide()
	await get_tree().process_frame

	var bridge = _app.bridge
	bridge.generate({
		"seed": SEED, "width_km": 2000.0, "grid_w": 256, "grid_h": 192,
		"archetype": "", "villages": true, "sea_level": 0.45,
	})
	while bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().create_timer(0.5).timeout

	var settlements: Array = bridge.settlements()
	_ok("world: a settlement exists to scope the Attach check", settlements.size() > 0)
	if settlements.is_empty():
		_finish()
		return
	var s: Dictionary = settlements[0]
	var tid := int(s.get("tid", 0))
	var sname := String(s.get("name", ""))

	# -- a real scratch vault, real folders -------------------------------------
	var root := OS.get_environment("TEMP").replace("\\", "/") + "/cartalith-vaulttree-probe"
	DirAccess.make_dir_recursive_absolute(root)
	_write(root + "/" + NOTE_A, NOTE_A_TEXT)
	_write(root + "/" + NOTE_B, "# Ferry crossing\n\nA smaller note, one folder deeper.\n")
	_write(root + "/" + NOTE_ROOT, "# Root note\n\nLives directly in the vault folder.\n")
	var conn: Dictionary = bridge.vault_connect(root, "TreeProbeVault")
	_ok("connect: a real folder binds", bool(conn.get("ok", false)), String(conn.get("error", "")))

	# -- §3 open browse mode, find the Tree -------------------------------------
	_app.open_vault_browse()
	await get_tree().process_frame
	await get_tree().process_frame
	var vw := _find(_app, VaultWindow)
	_ok("panel: the vault window opened in browse mode", vw != null)
	if vw == null:
		_finish()
		return

	var tree := _find(vw, Tree) as Tree
	_ok("tree: a Tree control renders in browse mode", tree != null)
	if tree == null:
		_finish()
		return

	var real_files: PackedStringArray = bridge.vault_list_files(2000)
	_ok("tree: the vault actually has the three real files", real_files.size() >= 3, str(real_files))

	var item_a := _find_item_by_metadata(tree.get_root(), NOTE_A)
	var item_b := _find_item_by_metadata(tree.get_root(), NOTE_B)
	var item_root_note := _find_item_by_metadata(tree.get_root(), NOTE_ROOT)
	_ok("tree: %s is a leaf item" % NOTE_A, item_a != null)
	_ok("tree: %s is a leaf item" % NOTE_B, item_b != null)
	_ok("tree: %s (no subfolder) is a leaf item" % NOTE_ROOT, item_root_note != null)

	var folder_towns := _find_item_by_text(tree.get_root(), "Towns")
	var folder_old := _find_item_by_text(tree.get_root(), "Old")
	_ok("tree: a real folder ('Towns') appears, derived from the path split", folder_towns != null)
	_ok("tree: a nested real folder ('Towns/Old') appears", folder_old != null)
	if folder_towns != null:
		_ok("tree: a folder row is not itself selectable (only files are)", not folder_towns.is_selectable(0))

	# -- §4 expand/collapse a folder ---------------------------------------------
	if folder_towns != null:
		var was_collapsed := folder_towns.collapsed
		folder_towns.collapsed = true
		_ok("tree: a folder can be collapsed", folder_towns.collapsed)
		folder_towns.collapsed = false
		_ok("tree: and re-expanded", not folder_towns.collapsed)
		folder_towns.collapsed = was_collapsed
	await _shot("_vaulttree_probe_a_tree")

	# -- §5 select a file, cross-check the preview against direct bridge calls --
	if item_a != null:
		item_a.select(0)
		tree.item_selected.emit()
		await get_tree().process_frame
		await get_tree().process_frame

		_ok("select: picking the tree item updates the shared selection", vw._pick_file == NOTE_A, vw._pick_file)

		var direct_data: Dictionary = bridge.vault_file_data(NOTE_A)
		var direct_headings: Array = bridge.vault_file_headings(NOTE_A)
		var direct_read: Dictionary = bridge.vault_read_file_for_edit(NOTE_A)

		var labels: Array = []
		_collect_texts(vw, labels)
		var joined := "\n".join(PackedStringArray(labels))

		var frontmatter: Dictionary = direct_data.get("frontmatter", {})
		var fm_ok := true
		for k in frontmatter:
			var expect := "%s: %s" % [String(k), String(frontmatter[k])]
			if joined.find(expect) < 0:
				fm_ok = false
				_ok("preview: frontmatter chip for %s matches vault_file_data" % k, false, expect)
		if fm_ok and not frontmatter.is_empty():
			_ok("preview: every frontmatter chip matches a direct vault_file_data call", true)
		_ok("preview: frontmatter was non-empty to check against", not frontmatter.is_empty(), str(direct_data))

		var headings_ok := true
		for h in direct_headings:
			var d: Dictionary = h
			var title := String(d.get("title", ""))
			if joined.find(title) < 0:
				headings_ok = false
				_ok("preview: heading '%s' matches vault_file_headings" % title, false, joined)
		if headings_ok and not direct_headings.is_empty():
			_ok("preview: every heading matches a direct vault_file_headings call", true)
		_ok("preview: headings were non-empty to check against", not direct_headings.is_empty(), str(direct_headings))

		var excerpt: String = vw._first_lines(vw._strip_frontmatter(String(direct_read.get("text", ""))), 3)
		_ok("preview: the excerpt is drawn from the same read_for_edit text",
			excerpt != "" and joined.find(excerpt) >= 0, excerpt)

	# -- §6 "Open to edit" reaches the real editor ----------------------------
	## The tree preview's button has been "Open to edit" since 2026-09-26; this
	## still looked for the Attach form's label and failed at HEAD before
	## Ruling BE (measured 2026-09-27, baseline run).
	var open_btn := _find_button(vw, "Open to edit")
	_ok("editor: Open to edit is offered from the tree preview", open_btn != null)
	if open_btn != null:
		open_btn.pressed.emit()
		await get_tree().process_frame
		await get_tree().process_frame
		_ok("editor: it opened on the tree-selected file", vw._browse_path == NOTE_A, vw._browse_path)
		_ok("editor: the raw text is exactly the file on disk", vw._browse_edit != null
			and vw._browse_edit.text == NOTE_A_TEXT, str(vw._browse_edit.text if vw._browse_edit else "<null>"))
	await _shot("_vaulttree_probe_b_editor")

	# -- §7 the entity-scoped Attach flow, unchanged -----------------------------
	_app.open_vault("settlement", tid, sname)
	await get_tree().process_frame
	await get_tree().process_frame
	var vw3 := _find(_app, VaultWindow)
	_ok("attach: the same window reopens scoped to the settlement", vw3 == vw)
	var labels3: Array = []
	_collect_texts(vw3, labels3)
	var joined3 := "\n".join(PackedStringArray(labels3))
	_ok("attach: 'Attach a note' section is present, unlike browse mode", joined3.findn("Attach a note") >= 0, joined3)
	## Ruling BE (2026-09-27) inverted this: the entity view IS the browser
	## now, so the scoped Attach path is served by the same Tree.
	_ok("attach: the entity view shows the same Tree as browse (Ruling BE)",
		_find(vw3, Tree) != null, "no Tree under the entity-scoped view")

	vw3._pick_file = NOTE_A
	vw3._rebuild()
	await get_tree().process_frame
	var attach_btn := _find_button(vw3, "Attach to %s" % sname)
	_ok("attach: the Attach button is offered", attach_btn != null)
	if attach_btn != null:
		attach_btn.pressed.emit()
		await get_tree().process_frame
		await get_tree().process_frame
		var links: Array = bridge.vault_links_for("settlement", tid)
		_ok("attach: vault_attach actually linked the note", links.size() > 0, str(links))
		if not links.is_empty():
			var linked: Dictionary = links[0]
			_ok("attach: the link points at the tree-previewed file", String(linked.get("path", "")) == NOTE_A,
				String(linked.get("path", "")))
	await _shot("_vaulttree_probe_c_attach_unregressed")

	_finish()


func _collect_texts(n: Node, out: Array) -> void:
	if n is Label:
		out.append(String((n as Label).text))
	elif n is Button:
		out.append(String((n as Button).text))
	for c in n.get_children():
		_collect_texts(c, out)


func _finish() -> void:
	if _fails.is_empty():
		print("VTREE    ALL CHECKS PASSED")
	else:
		print("VTREE    %d FAILED: %s" % [_fails.size(), ", ".join(PackedStringArray(_fails))])
	get_tree().quit(0 if _fails.is_empty() else 1)
