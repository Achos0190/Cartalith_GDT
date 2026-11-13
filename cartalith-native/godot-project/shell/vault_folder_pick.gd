extends RefCounted
## The folder chooser for a new vault note (owner, 2026-11-13: "make sure that from the note ... we also get the
## option to select where in the vault the note gets placed"). One implementation, used by the Context panel's
## Note tab (`ContextNote._build_create`), by the vault window's "New note from a template" form and by the file
## tree's "Move folder..." (`ask_move`), on desktop, tablet and phone alike.
##
## What it is: a collapsible TREE of the connected vault's existing folders (derived from `vault_list_files`, so it
## is the same list the file tree shows, and nothing is walked a second time), a filter field (drawn once the vault
## has more than `FILTER_MIN_FOLDERS` folders, so a small vault looks as simple as the dropdown this replaced), a
## "New folder..." button that reveals a name field for a folder made UNDER the selected node, and a rewrite of the
## path field the form already had. The path stays the one source of truth the form creates from, so a user who
## types a path by hand still can; the chooser only edits its directory part. Owner, 2026-11-13: "a folder tree
## instead of a dropdown for a very large vault".
##
## What it must never do: write anything. Choosing a folder changes text on screen. The folder (and the file) come
## into being when Create runs `vault_create_file` / `vault_create_from_template`, whose crate code creates missing
## parents (`create_dir_all`; on Android `vault_saf_handler.gd` makes each missing folder through
## `driver.create(.., true)`), and which still refuse an existing file. Move's destination is only handed to the
## caller, which plans the move through the one link planner.

const DccThemeRef := preload("res://shell/dcc_theme.gd")
const DccWidgetsRef := preload("res://shell/dcc_widgets.gd")
const Settings := preload("res://shell/dcc_settings.gd")

## The "New folder..." button's text; also its probe-visible label.
const NEW_FOLDER_LABEL := "New folder..."
## The touch floor (dp) for tree rows, the new-folder field, the filter and the button (the 44 px touch rule).
const TOUCH_MIN := 44
## Labelled judgement: at or below this many folders the filter field is not drawn.
const FILTER_MIN_FOLDERS := 8
## Labelled judgement: the tree's height in dp. About five rows at 44 on touch, a compact well on desktop.
const TREE_H_DESKTOP := 150
const TREE_H_TOUCH := 230
## The vault root's row text.
const ROOT_LABEL := "(vault root)"


## True when `p` is `folder` or lies inside it ("" is never within anything: the root is not a folder to exclude).
## The one test behind Move's "not into itself or a descendant".
static func is_within(folder: String, p: String) -> bool:
	return folder != "" and (p == folder or p.begins_with(folder + "/"))


## `base` joined with the new-folder text, or null when the text is not a clean relative folder (never a made-up
## name). The name is relative to the selected node: "New folder... under the selected node".
static func new_folder_path(base: String, text: String):
	var c := clean_new_folder(text)
	return null if c == "" else join(base, c)


## The vault's folders from its file list: every ancestor folder of every file, sorted, with "" (the vault root)
## first. A folder with a segment starting "." (`.obsidian`, `.cartalith`) is app data, not a place for notes, and
## is skipped. `extra` (the kind's suggested folder, which a fresh vault may not have yet) is added with its
## ancestors when absent, so the default is always selectable.
static func folders(files: PackedStringArray, extra: String = "") -> Array:
	var seen := {"": true}
	for f in files:
		var dir := String(f).get_base_dir()
		while dir != "" and not seen.has(dir):
			seen[dir] = true
			dir = dir.get_base_dir()
	var add := extra
	while add != "" and not seen.has(add):
		seen[add] = true
		add = add.get_base_dir()
	var out: Array = []
	for k in seen:
		var hidden := false
		for seg in String(k).split("/"):
			if String(seg).begins_with("."):
				hidden = true
		if not hidden:
			out.append(k)
	out.sort()   ## "" sorts first
	return out


## Which of `list` to select: the remembered folder for `kind` when there is one and the vault has it, otherwise
## `default_folder` (the folder the form used before this control existed). An index into `list`; the default is
## always in it when `folders()` was given it as `extra`.
static func initial_index(list: Array, remembered, default_folder: String) -> int:
	if remembered != null and list.has(String(remembered)):
		return list.find(String(remembered))
	return list.find(default_folder)


## `folder` joined to `file_name`: "" is the vault root, so no leading slash.
static func join(folder: String, file_name: String) -> String:
	return file_name if folder == "" else "%s/%s" % [folder, file_name]


## A new folder's text as a clean relative folder, or "" when it is not one (empty, a backslash or colon, a hidden
## or empty segment, `..`). The crate refuses unsafe paths anyway; this keeps the refusal on the same screen.
## Surrounding spaces and slashes are trimmed.
static func clean_new_folder(text: String) -> String:
	var t := text.strip_edges().trim_prefix("/").trim_suffix("/")
	if t == "" or t.contains("\\") or t.contains(":"):
		return ""
	for seg in t.split("/"):
		var s := String(seg).strip_edges()
		if s == "" or s.begins_with("."):
			return ""
	return t


## Fills `tree` with `list` (folder paths, "" first); items carry their path as metadata. A non-empty `filter`
## keeps only folders whose path contains it (case-insensitive) plus their ancestors, all expanded. With no filter
## every folder is collapsed except the root and the ancestors of `selected`, so a small vault reads as a short list.
## Folders in `excluded`'s subtree are drawn dim and are not selectable. Returns the item for `selected`, or null
## when it is filtered out.
static func fill_tree(tree: Tree, list: Array, selected: String, filter: String, excluded: String) -> TreeItem:
	tree.clear()
	var f := filter.strip_edges().to_lower()
	var keep := {"": true}
	for p in list:
		if f == "" or String(p).to_lower().contains(f):
			var d := String(p)
			while d != "" and not keep.has(d):
				keep[d] = true
				d = d.get_base_dir()
	var items := {}
	var root := tree.create_item()
	root.set_text(0, ROOT_LABEL)
	root.set_metadata(0, "")
	items[""] = root
	var order: Array = []
	for p in list:
		if String(p) != "" and keep.has(String(p)):
			order.append(String(p))
	order.sort()   ## a parent sorts before its children, so its item exists
	var found: TreeItem = root if selected == "" else null
	for p in order:
		var parent_item: TreeItem = items.get(p.get_base_dir(), root)
		var it := tree.create_item(parent_item)
		it.set_text(0, p.get_file())
		it.set_metadata(0, p)
		it.set_tooltip_text(0, p)
		items[p] = it
		if p == selected:
			found = it
	var touch := DccThemeRef.is_touch()
	for p in items:
		var it2: TreeItem = items[p]
		if touch:
			it2.set_custom_minimum_height(TOUCH_MIN)
		if excluded != "" and is_within(excluded, String(p)):
			it2.set_selectable(0, false)
			it2.set_custom_color(0, DccThemeRef.c("text_dim"))
			it2.set_tooltip_text(0, "A folder cannot be moved into itself or a folder inside it.")
		it2.collapsed = f == "" and String(p) != ""
	if f == "":
		var a := selected.get_base_dir() if selected != "" else ""
		while a != "":
			if items.has(a):
				(items[a] as TreeItem).collapsed = false
			a = a.get_base_dir()
	root.collapsed = false
	return found


## The folder tree control under `parent`. `list` is the folders ("" first), `initial` the folder selected first
## (its ancestors are expanded), `excluded` a folder whose subtree cannot be chosen ("" for none), `on_change` is
## called after the chosen folder or the new-folder name changes. Returns `{"tree", "filter" (or null), "new_edit",
## "new_button", "folder": Callable, "select": Callable}`: `folder` answers the folder now chosen ("" is the vault
## root) -- **null while "New folder..." is open with no valid name** -- and `select(path)` selects a folder as a tap
## would. Node names `NoteFolder` (the tree), `NoteFolderFilter`, `NoteFolderNew` and `NoteNewFolder` are for probes.
static func chooser(parent: Control, list: Array, initial: String, excluded: String, on_change: Callable) -> Dictionary:
	var touch := DccThemeRef.is_touch()
	var state := {"sel": initial, "silent": false, "filter": ""}
	var box := VBoxContainer.new()
	box.name = "NoteFolderBox"
	box.add_theme_constant_override("separation", 4)
	box.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	parent.add_child(box)
	var filter: LineEdit = null
	if list.size() > FILTER_MIN_FOLDERS:
		filter = LineEdit.new()
		filter.name = "NoteFolderFilter"
		filter.placeholder_text = "Filter folders"
		filter.clear_button_enabled = true
		if touch:
			filter.custom_minimum_size.y = TOUCH_MIN
		box.add_child(filter)
	var tree := Tree.new()
	tree.name = "NoteFolder"
	tree.hide_root = false
	tree.select_mode = Tree.SELECT_SINGLE
	tree.custom_minimum_size = Vector2(0, TREE_H_TOUCH if touch else TREE_H_DESKTOP)
	tree.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	## Same tokens as the file tree (`vault_window.gd`): the project theme has no Tree entries, so without this it
	## draws Godot's stock grey slab.
	var well := DccThemeRef.panel("sunken", {"left": 1, "right": 1, "top": 1, "bottom": 1})
	well.border_color = DccThemeRef.c("border")
	well.content_margin_left = 4
	well.content_margin_top = 4
	tree.add_theme_stylebox_override("panel", well)
	tree.add_theme_stylebox_override("focus", DccThemeRef.empty())
	tree.tooltip_text = "The vault's folders. Pick one."
	box.add_child(tree)
	var new_btn := Button.new()
	new_btn.name = "NoteFolderNew"
	new_btn.text = NEW_FOLDER_LABEL
	new_btn.toggle_mode = true
	new_btn.alignment = HORIZONTAL_ALIGNMENT_LEFT
	if touch:
		new_btn.custom_minimum_size.y = TOUCH_MIN
	new_btn.tooltip_text = "Makes a folder under the selected one. It is created when you confirm."
	box.add_child(new_btn)
	var new_edit := LineEdit.new()
	new_edit.name = "NoteNewFolder"
	new_edit.placeholder_text = "Folder name under the selected one, e.g. Coast"
	new_edit.visible = false
	new_edit.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	if touch:
		new_edit.custom_minimum_size.y = TOUCH_MIN
	new_edit.tooltip_text = "A folder that does not exist yet, inside the selected one."
	box.add_child(new_edit)

	var current := func():
		if new_edit.visible:
			return new_folder_path(String(state["sel"]), new_edit.text)
		return String(state["sel"])
	var changed := func() -> void:
		if on_change.is_valid():
			on_change.call()
	var refill := func() -> void:
		state["silent"] = true
		var it := fill_tree(tree, list, String(state["sel"]), String(state["filter"]), excluded)
		if it != null:
			it.select(0)
			tree.scroll_to_item(it)
		state["silent"] = false
	tree.item_selected.connect(func() -> void:
		if bool(state["silent"]):
			return
		var it := tree.get_selected()
		if it == null:
			return
		state["sel"] = String(it.get_metadata(0))
		changed.call())
	## A tap expands (owner brief): on touch the arrow is a small target, so the row itself toggles its children.
	tree.item_mouse_selected.connect(func(_pos: Vector2, button: int) -> void:
		var it := tree.get_selected()
		if touch and button == MOUSE_BUTTON_LEFT and it != null and it.get_child_count() > 0 and not bool(state["silent"]):
			it.collapsed = not it.collapsed)
	if filter != null:
		filter.text_changed.connect(func(t: String) -> void:
			state["filter"] = t
			refill.call())
	new_btn.toggled.connect(func(on: bool) -> void:
		new_edit.visible = on
		if on:
			new_edit.grab_focus()
		changed.call())
	new_edit.text_changed.connect(func(_t: String) -> void: changed.call())
	refill.call()
	return {"tree": tree, "filter": filter, "new_edit": new_edit, "new_button": new_btn, "folder": current,
		"select": func(p: String) -> void:
			state["sel"] = p
			refill.call()
			changed.call()}


## Builds the chooser under `parent` and wires it to `path_edit` (the form's full-path field).
## `kind` keys the remembered folder; `default_rel` is the path the form suggested. Returns the `chooser()` dict;
## its `folder` answers the folder currently chosen ("" is the vault root), or **null while "New folder..." has no
## valid name yet** (never a made-up one). The remembered folder for `kind` is selected when the vault still has
## it, otherwise the default.
static func build(parent: Control, bridge, kind: String, default_rel: String, path_edit: LineEdit) -> Dictionary:
	var default_folder := default_rel.get_base_dir()
	var list := folders(bridge.vault_list_files(0), default_folder)
	var sel := initial_index(list, Settings.vault_note_folder(kind), default_folder)
	var start := String(list[sel]) if sel >= 0 else default_folder
	var holder := {}
	var apply := func() -> void:
		if holder.is_empty():
			return
		var f = (holder["folder"] as Callable).call()
		if f != null:
			path_edit.text = join(String(f), path_edit.text.get_file())
	var c := chooser(parent, list, start, "", apply)
	holder.merge(c)
	apply.call()   ## the remembered folder, or the default: rewrites the path field to match
	return c


## The file tree's "Move folder..." destination dialog: the same tree, with `old`'s own subtree dim and not
## selectable, and "New folder..." for a destination that does not exist yet. `on_submit(dest)` gets the chosen
## folder ("" is the top level). OK stays disabled until a valid destination is chosen. Built by
## `DccWidgets.custom_dialog`, so it presents like every other prompt (phone sheet, desktop window, pop-out).
## Returns the chooser dict plus the dialog under "dialog", for probes.
static func ask_move(host: Node, bridge, old: String, on_submit: Callable, hint: String) -> Dictionary:
	var list := folders(bridge.vault_list_files(0), old.get_base_dir())
	var d: Dictionary = DccWidgetsRef.custom_dialog(host, "Move folder", "Move", 380)
	var dlg: ConfirmationDialog = d["dialog"]
	var body: VBoxContainer = d["body"]
	body.add_child(DccThemeRef.label("Move into folder", "text_dim", DccThemeRef.FS_SMALL))
	var holder := {}
	var sync := func() -> void:
		if holder.is_empty():
			return
		var f = (holder["folder"] as Callable).call()
		dlg.get_ok_button().disabled = f == null or is_within(old, String(f))
	var c := chooser(body, list, old.get_base_dir(), old, sync)
	holder.merge(c)
	if hint != "":
		DccWidgetsRef.note(body, hint)
	sync.call()
	dlg.confirmed.connect(func() -> void:
		var f = (c["folder"] as Callable).call()
		if f != null and not is_within(old, String(f)) and on_submit.is_valid():
			on_submit.call(String(f))
		if is_instance_valid(dlg):
			dlg.queue_free())
	dlg.canceled.connect(func() -> void:
		if is_instance_valid(dlg):
			dlg.queue_free())
	DccWidgetsRef.present_custom_dialog(d)
	c["dialog"] = dlg
	return c


## Remembers where a note of `kind` just went (the folder part of `rel`). Called by the form after a create
## that succeeded, never before.
static func remember(kind: String, rel: String) -> void:
	Settings.set_vault_note_folder(kind, rel.get_base_dir())
