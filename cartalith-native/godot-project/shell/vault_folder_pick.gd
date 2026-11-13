extends RefCounted
## The folder chooser for a new vault note (owner, 2026-11-13: "make sure that from the note ... we also get the
## option to select where in the vault the note gets placed"). One implementation, used by the Context panel's
## Note tab (`ContextNote._build_create`) and by the vault window's "New note from a template" form, on desktop,
## tablet and phone alike.
##
## What it is: a dropdown of the connected vault's existing folders (derived from `vault_list_files`, so it is the
## same list the file tree shows, and nothing is walked a second time), a "New folder..." entry that reveals a name
## field, and a rewrite of the path field the form already had. The path stays the one source of truth the form
## creates from, so a user who types a path by hand still can; the folder control only edits its directory part.
##
## What it must never do: write anything. Choosing a folder changes text on screen. The folder (and the file) come
## into being when Create runs `vault_create_file` / `vault_create_from_template`, whose crate code creates missing
## parents (`create_dir_all`; on Android `vault_saf_handler.gd` makes each missing folder through
## `driver.create(.., true)`), and which still refuse an existing file.

const DccThemeRef := preload("res://shell/dcc_theme.gd")
const DccWidgetsRef := preload("res://shell/dcc_widgets.gd")
const Settings := preload("res://shell/dcc_settings.gd")

## The dropdown's last entry; also its probe-visible label.
const NEW_FOLDER_LABEL := "New folder..."
## The touch floor (dp) for the dropdown, the new-folder field and the popup's rows (the 44 px touch rule).
const TOUCH_MIN := 44


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


## Builds the control under `parent` and wires it to `path_edit` (the form's full-path field).
## `kind` keys the remembered folder; `default_rel` is the path the form suggested. Returns
## `{"choice": OptionButton, "new_edit": LineEdit, "folder": Callable}`; `folder` answers the folder currently
## chosen ("" is the vault root), or **null while "New folder..." has no valid name yet** (never a made-up one).
## Node names `NoteFolder` and `NoteNewFolder` are for probes.
static func build(parent: Control, bridge, kind: String, default_rel: String, path_edit: LineEdit) -> Dictionary:
	var default_folder := default_rel.get_base_dir()
	var list := folders(bridge.vault_list_files(0), default_folder)
	var labels: Array = []
	for f in list:
		labels.append("(vault root)" if String(f) == "" else String(f))
	labels.append(NEW_FOLDER_LABEL)
	var state := {"folder": default_folder}
	var new_edit := LineEdit.new()
	new_edit.name = "NoteNewFolder"
	new_edit.placeholder_text = "Folder name, e.g. Places/Coast"
	new_edit.visible = false
	new_edit.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	new_edit.tooltip_text = "A folder that does not exist yet. It is made when the note is created."
	var apply := func() -> void:
		var f = state["folder"]
		if f != null:
			path_edit.text = join(String(f), path_edit.text.get_file())
	var from_text := func(t: String) -> void:
		var c := clean_new_folder(t)
		state["folder"] = c if c != "" else null
		apply.call()
	var on_pick := func(i: int) -> void:
		if i >= list.size():
			new_edit.visible = true
			new_edit.grab_focus()
			from_text.call(new_edit.text)
		else:
			new_edit.visible = false
			state["folder"] = String(list[i])
			apply.call()
	var sel := initial_index(list, Settings.vault_note_folder(kind), default_folder)
	var ch := DccWidgetsRef.choice(parent, "Folder", labels, maxi(sel, 0), on_pick,
		"Where in the vault the note goes. The folders are the vault's own; New folder... makes one when the note is created.")
	ch.name = "NoteFolder"
	parent.add_child(new_edit)
	new_edit.text_changed.connect(from_text)
	if DccThemeRef.is_touch():
		ch.custom_minimum_size.y = maxi(int(ch.custom_minimum_size.y), TOUCH_MIN)
		new_edit.custom_minimum_size.y = TOUCH_MIN
		ch.get_popup().add_theme_constant_override("v_separation", 14)
	if sel >= 0:
		on_pick.call(sel)   ## the remembered folder, or the default: rewrites the path field to match
	return {"choice": ch, "new_edit": new_edit, "folder": func(): return state["folder"]}


## Remembers where a note of `kind` just went (the folder part of `rel`). Called by the form after a create
## that succeeded, never before.
static func remember(kind: String, rel: String) -> void:
	Settings.set_vault_note_folder(kind, rel.get_base_dir())
