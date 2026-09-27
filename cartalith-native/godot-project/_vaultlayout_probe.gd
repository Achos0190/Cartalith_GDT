extends Node
## Windowed proof for `OUTSTANDING_WORK.md`'s Vault Browser row's own last
## three "Still open, re-measured" gaps against the owner-approved mockup
## (`design/vault-browser-2026-09-21/Cartalith Vault Browser.dc.html`),
## re-measured after this pass:
##
##   1. search at the head of the tree column;
##   2. a one-line connection header;
##   3. outline and excerpt side by side (desktop) / stacked (phone).
##
## `_vaultbrowsegaps_probe.gd` (`b778f79`) is left untouched — its own scope is
## the backlinks/mentions counts, Centre on map and the excerpt's heading-
## marker strip, a different slice of the same window. This probe is scoped
## to the three gaps still open at that commit.
##
##   Godot_v4.7.1-stable_win64_console.exe --path . _vaultlayout_probe.tscn -- --vp 1280x800
##   Godot_v4.7.1-stable_win64_console.exe --path . _vaultlayout_probe.tscn -- --vp 1080x2340 --force-touch
##
## **Windowed, not `--headless`** (`MISTAKES.md`'s "run a pixel probe" row):
## this probe saves real framebuffer PNGs for visual comparison against the
## mockup, and `ImageTexture.update()`/`RenderingServer.frame_post_draw` are
## no-ops under `--headless` -- a headless run would pass every pixel-shaped
## check vacuously (there are none here that assert on colour, but the saved
## PNGs would be blank, which defeats the whole point of saving them).
##
## Flags this probe actually reads, grepped from the body below:
##   `--vp WxH`      the host SubViewport's size in physical px. Default
##                   1280x800, the mockup's own frame.
##   `--force-touch` read by `dcc_shell.gd`'s own layout-env probe, not here --
##                   this probe only reads `DccTheme.is_phone()` afterward to
##                   confirm which density it actually got, so a run that
##                   forgets the flag fails loudly instead of measuring the
##                   wrong density under the phone label.
## Any other `--flag` aborts rather than being silently ignored (MISTAKES.md's
## "write a probe's usage header" row).

const SEED := 552017

var _vp: SubViewport
var app: Node
var _bridge
var _root := ""
var _fails: Array = []


func _ok(label: String, cond: bool, detail: String = "") -> void:
	if cond:
		print("VAULTLAYOUT  OK  %s" % label)
	else:
		_fails.append(label)
		print("VAULTLAYOUT  !!  %s   %s" % [label, detail])


func _arg(name: String, dflt: String) -> String:
	var args := OS.get_cmdline_user_args()
	var i := args.find(name)
	if i >= 0 and i + 1 < args.size():
		return String(args[i + 1])
	return dflt


func _reject_unknown_args() -> bool:
	var known := ["--vp", "--force-touch"]
	for a in OS.get_cmdline_user_args():
		var s := String(a)
		if s.begins_with("--") and not (s in known):
			print("VAULTLAYOUT ABORT unknown flag %s -- this probe reads only %s" % [s, str(known)])
			return false
	return true


func _frames(n: int) -> void:
	for _i in n:
		await get_tree().process_frame


func _write(path: String, text: String) -> void:
	var f := FileAccess.open(path, FileAccess.WRITE)
	f.store_string(text)
	f.close()


## Every node under `n` whose own `.name` is `target`, depth-first, ignoring
## `owned` (every node this window builds at runtime is unowned).
func _find_named(n: Node, target: String) -> Node:
	if n.name == target:
		return n
	for c in n.get_children():
		var r := _find_named(c, target)
		if r != null:
			return r
	return null


func _find_typed(n: Node, cls: String, out: Array) -> void:
	if n.is_class(cls):
		out.append(n)
	for c in n.get_children():
		_find_typed(c, cls, out)


func _find_line_edit(n: Node, placeholder: String) -> LineEdit:
	var edits: Array = []
	_find_typed(n, "LineEdit", edits)
	for e in edits:
		if String((e as LineEdit).placeholder_text) == placeholder:
			return e as LineEdit
	return null


## Simulates a real keystroke: sets the field's text and emits its own signal,
## the same idiom `_assetsearch_probe.gd::_type()` uses, rather than writing
## `_search_query` directly -- so a well that was built but never connected to
## `_refresh_browse_tree()` would fail here.
func _type(field: LineEdit, s: String) -> void:
	field.text = s
	field.text_changed.emit(s)
	await _frames(6)


## Every `TreeItem` label under `root`'s tree, depth-first, folders and leaves
## both -- so the probe can assert on names without assuming a fixed
## nesting depth.
func _tree_item_texts(item: TreeItem, out: Array) -> void:
	if item == null:
		return
	out.append(String(item.get_text(0)))
	var c := item.get_first_child()
	while c != null:
		_tree_item_texts(c, out)
		c = c.get_next()


func _find_tree(n: Node) -> Tree:
	if n is Tree:
		return n as Tree
	for c in n.get_children():
		var r := _find_tree(c)
		if r != null:
			return r
	return null


func _texts(n: Node, out: Array) -> void:
	if n is Label:
		out.append(String((n as Label).text))
	for c in n.get_children():
		_texts(c, out)


func _generate() -> void:
	_bridge.generate({
		"seed": SEED, "width_km": 2400.0, "grid_w": 384, "grid_h": 288,
		"archetype": "", "villages": true, "sea_level": 0.45,
	})
	while _bridge.generating:
		await get_tree().create_timer(0.25).timeout
	await get_tree().process_frame
	await get_tree().process_frame


func _ready() -> void:
	if not _reject_unknown_args():
		get_tree().quit(2)
		return
	if DisplayServer.get_name() == "headless":
		push_error("_vaultlayout_probe: run WINDOWED. ImageTexture.update() is a "
			+ "no-op under --headless, and the saved PNGs would be blank.")
		print("VAULTLAYOUT REFUSED: headless")
		get_tree().quit(2)
		return

	var parts: PackedStringArray = _arg("--vp", "1280x800").split("x")
	if parts.size() != 2:
		print("VAULTLAYOUT ABORT --vp wants WxH")
		get_tree().quit(2)
		return
	var vp_size := Vector2i(int(parts[0]), int(parts[1]))
	var want_touch := "--force-touch" in OS.get_cmdline_user_args()

	if FileAccess.file_exists(VaultStore.PATH):
		DirAccess.remove_absolute(ProjectSettings.globalize_path(VaultStore.PATH))

	_vp = SubViewport.new()
	_vp.size = vp_size
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
	_bridge = app.bridge

	var phone := DccTheme.is_phone()
	print("VAULTLAYOUT vp=%s force_touch=%s phone=%s" % [str(vp_size), want_touch, phone])
	_ok("density matches the flag passed (force-touch <-> phone)", phone == want_touch,
		"pass --force-touch for a phone run, or drop it for desktop")

	await _generate()

	# -- a real vault on disk, shaped so a query has real parents to keep ------
	_root = OS.get_environment("TEMP").replace("\\", "/") + "/cartalith-vaultlayout-probe"
	DirAccess.make_dir_recursive_absolute(_root + "/Settlements")
	DirAccess.make_dir_recursive_absolute(_root + "/Events")
	DirAccess.make_dir_recursive_absolute(_root + "/Lore")
	_write(_root + "/Settlements/Aldenmoor.md",
		"---\ntype: settlement\ntid: 1\n---\n\n# Founding legend\n\nThe old song claims the ford chose its own name.\n\n## Customs Gate\n\nA river crossing grown into a customs town.\n")
	_write(_root + "/Settlements/Hammerfell.md",
		"---\ntype: settlement\ntid: 2\n---\n\n# Iron works\n\nFounded on the old garrison site.\n")
	_write(_root + "/Events/Salt-Road.md",
		"---\ntype: event\nyear: 398\n---\n\nThree names on the customs-gate list.\n")
	_write(_root + "/Lore/Unrelated.md", "Nothing to do with any settlement.\n")

	var conn: Dictionary = _bridge.vault_connect(_root, "ProbeVault")
	_ok("connect: the fixture folder binds", bool(conn.get("ok", false)), String(conn.get("error", "")))

	app.open_vault_browse()
	await _frames(4)
	var vw: VaultWindow = app.vault_window
	_ok("the standalone browser is open", vw != null and (vw as Window).visible)
	if vw == null:
		_finish()
		return

	var real_count: int = _bridge.vault_list_files(2000).size()
	_ok("CONTROL: the fixture really has 4 files", real_count == 4, "vault_list_files=%d" % real_count)

	# == 1. search at the head of the tree column ============================
	var tree_col := _find_named(vw, "VaultTreeCol")
	_ok("the tree column exists (desktop split)", tree_col != null or phone)
	var search_host: Node = tree_col if tree_col != null else vw
	var field := _find_line_edit(search_host, "Search notes…")
	_ok("the tree-column search field is drawn", field != null)
	if field == null:
		_finish()
		return
	var tree := _find_tree(vw)
	_ok("the tree is drawn", tree != null)
	if tree != null:
		_ok("the search field sits ABOVE the tree (head of the column)",
			field.get_global_rect().position.y < tree.get_global_rect().position.y,
			"field.y=%.1f tree.y=%.1f" % [field.get_global_rect().position.y, tree.get_global_rect().position.y])
		if tree_col != null:
			_ok("both the field and the tree are inside the tree column",
				tree_col.is_ancestor_of(field) and tree_col.is_ancestor_of(tree))

	## The "Find a note" content-search section (`_build_search`) must NOT be
	## drawn in browse mode any more -- the mockup draws nothing else in this
	## slot, and its own richer UI (a Search button, an index-build offer) has
	## no home in the mockup's bare filter box.
	var all_buttons: Array = []
	_find_typed(vw, "Button", all_buttons)
	var has_old_search_button := false
	for b in all_buttons:
		if String((b as Button).text) == "Search":
			has_old_search_button = true
	_ok("the old top-of-window \"Search\" button is gone in browse mode", not has_old_search_button)

	# == typing filters the tree, keeping parents visible; clearing restores ==
	_ok("before typing: every fixture folder is in the tree",
		_tree_has(tree, "Settlements") and _tree_has(tree, "Events") and _tree_has(tree, "Lore"))
	await _type(field, "Hammerfell")
	tree = _find_tree(vw)   ## `_refresh_browse_tree()` rebuilds the Tree node itself
	_ok("typing narrows to the matching leaf", _tree_has(tree, "Hammerfell.md"))
	_ok("its parent folder stays visible", _tree_has(tree, "Settlements"))
	_ok("a non-matching sibling leaf is gone", not _tree_has(tree, "Aldenmoor.md"))
	_ok("a non-matching folder is gone entirely", not _tree_has(tree, "Events") and not _tree_has(tree, "Lore"))
	await _type(field, "")
	tree = _find_tree(vw)
	_ok("clearing restores every fixture folder", _tree_has(tree, "Settlements")
		and _tree_has(tree, "Events") and _tree_has(tree, "Lore"))
	_ok("clearing restores every fixture leaf", _tree_has(tree, "Aldenmoor.md")
		and _tree_has(tree, "Hammerfell.md") and _tree_has(tree, "Salt-Road.md")
		and _tree_has(tree, "Unrelated.md"))

	# == 2. one-line connection header, real counts ===========================
	var texts: Array = []
	_texts(vw, texts)
	var joined := "\n".join(PackedStringArray(texts))
	var count_re := RegEx.new()
	count_re.compile("ProbeVault · (\\d+) note")
	var m := count_re.search(joined)
	_ok("the connection header names the vault and a note count", m != null, joined)
	if m != null:
		var shown := int(m.get_string(1))
		_ok("the header's count equals the bridge's own vault_list_files() count -- not a probe constant",
			shown == real_count, "header=%d bridge=%d" % [shown, real_count])
	_ok("the header line names the read-only/open-to-edit contract",
		joined.find("read-only preview") >= 0 and joined.find("open to edit") >= 0)

	# == 3. outline and excerpt: side by side (desktop) / stacked (phone) =====
	vw.set("_pick_file", "Settlements/Aldenmoor.md")
	if phone:
		## The phone fold shows one pane at a time (`_build_browse_phone_
		## switcher`'s segmented FILES/PREVIEW row) -- the tree pane is the
		## default, so the preview (where the outline/excerpt columns live)
		## has to be switched to explicitly, the same way a real tap on
		## PREVIEW would.
		vw.set("_browse_phone_pane", "preview")
	vw.call("_rebuild")
	await _frames(4)
	var outline_col := _find_named(vw, "VaultOutlineCol")
	var excerpt_col := _find_named(vw, "VaultExcerptCol")
	_ok("the outline column is drawn", outline_col != null)
	_ok("the excerpt column is drawn", excerpt_col != null)
	if outline_col != null and excerpt_col != null:
		var ro: Rect2 = (outline_col as Control).get_global_rect()
		var re: Rect2 = (excerpt_col as Control).get_global_rect()
		print("VAULTLAYOUT outline_rect=%s excerpt_rect=%s phone=%s" % [ro, re, phone])
		_ok("outline and excerpt rects do not overlap", not ro.intersects(re), "%s vs %s" % [ro, re])
		if phone:
			_ok("PHONE: stacked -- excerpt starts at or below where outline ends",
				re.position.y >= ro.position.y + ro.size.y - 1.0,
				"outline bottom=%.1f excerpt top=%.1f" % [ro.position.y + ro.size.y, re.position.y])
		else:
			_ok("DESKTOP: side by side -- they share a row (vertically overlapping y-ranges)",
				ro.position.y < re.position.y + re.size.y and re.position.y < ro.position.y + ro.size.y,
				"%s vs %s" % [ro, re])
			_ok("DESKTOP: outline sits to the LEFT of excerpt",
				ro.position.x + ro.size.x <= re.position.x + 1.0,
				"outline right=%.1f excerpt left=%.1f" % [ro.position.x + ro.size.x, re.position.x])

	# == 4. preview text reads at the mockup's own size, live ================
	# `OUTSTANDING_WORK.md`'s "Vault Browser preview text looks smaller than
	# the mockup" row: the main loop's review measured the outline/excerpt
	# body at 9-10 px against the mockup's 12 / 13.5 px (`ENV:99`, `ENV:106`
	# of `Cartalith Vault Browser.dc.html`). Asserted here against
	# `get_theme_font_size()` on the LIVE Label -- not the `DccTheme.FS_BODY`
	# constant the fix reads -- so a label that silently kept its own
	# override (or a sibling `note()` call this pass missed) still fails
	# loudly rather than the check asserting the constant against itself
	# (`MISTAKES.md`).
	if outline_col != null and excerpt_col != null:
		var outline_row: Label = null
		for c in (outline_col as Node).get_children():
			if c is Label and String((c as Label).text).strip_edges() != "" \
					and String((c as Label).text).strip_edges() != "Outline":
				outline_row = c as Label
				break
		_ok("an outline content row is drawn", outline_row != null)
		if outline_row != null:
			var fs := outline_row.get_theme_font_size("font_size")
			_ok("outline row reads at the mockup's 12px (DccTheme.FS_BODY)", fs == 12,
				"live font_size=%d text=%s" % [fs, outline_row.text])

		var excerpt_row: Label = null
		for c in (excerpt_col as Node).get_children():
			if c is Label and String((c as Label).text).strip_edges() != "" \
					and String((c as Label).text).strip_edges() != "Excerpt":
				excerpt_row = c as Label
				break
		_ok("an excerpt content row is drawn", excerpt_row != null)
		if excerpt_row != null:
			var fs2 := excerpt_row.get_theme_font_size("font_size")
			_ok("excerpt row reads at the closest DccTheme token to the mockup's 13.5px (FS_BODY=12)",
				fs2 == 12, "live font_size=%d text=%s" % [fs2, excerpt_row.text])

	# == screenshot, for a human to compare against the mockup ================
	await RenderingServer.frame_post_draw
	await get_tree().process_frame
	await RenderingServer.frame_post_draw
	var img := _vp.get_texture().get_image()
	var tag := "phone" if phone else "desktop"
	var out_path := "res://_vaultlayout_%s.png" % tag
	img.save_png(out_path)
	print("VAULTLAYOUT saved %s" % ProjectSettings.globalize_path(out_path))

	_finish()


func _tree_has(tree: Tree, text: String) -> bool:
	if tree == null:
		return false
	var texts: Array = []
	_tree_item_texts(tree.get_root(), texts)
	return texts.has(text)


func _finish() -> void:
	if _fails.is_empty():
		print("VAULTLAYOUT  ALL CHECKS PASSED")
	else:
		print("VAULTLAYOUT  %d FAILED: %s" % [_fails.size(), ", ".join(PackedStringArray(_fails))])
	get_tree().quit(0 if _fails.is_empty() else 1)
