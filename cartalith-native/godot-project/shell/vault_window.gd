extends AcceptDialog
class_name VaultWindow

## The Markdown Vault panel — `MARKDOWN_VAULT_INTEGRATION.md` §28 and §29,
## `MARKDOWN_VAULT_SCOPE.md` milestone 1.
##
## §28 asks for the vault to "appear in entity information panels rather than
## as an isolated utility", and it does: `place_editor_window.gd`'s KNOWLEDGE
## section and the Civilization dock's province and continent rows are the
## entry points, and each of them opens *this* window already scoped to that
## entity. What lives here is everything §28's sketch cannot fit in a dock
## column — the file browser, the reader/working copy (§29), the preview, and
## the two write actions.
##
## One window, three entity kinds, because §11's whole point is a generic
## `KnowledgeLink`: `open_for("settlement", tid, name)`,
## `open_for("province", id, name)`, `open_for("continent", rank, name)`.
## Opened with no entity it shows the whole link store instead.
##
## ## Every write here is explicit, and every write is previewed
##
## §17: *"Reading can be automatic/on-demand. Writing cannot."* There are
## exactly three write buttons in this file, each behind a preview whose hash
## is handed back to the write — so a note edited in the user's own editor
## between the preview and the confirmation refuses instead of overwriting.
## The engine enforces that; this window's job is to never offer a write
## without having shown what it would do.
##
## ## What is deliberately not here
##
## - **No map snapshot** (§21). It needs a crop of the current renderer at
##   three radii; `MARKDOWN_VAULT_SCOPE.md` holds it as milestone 2 rather
##   than shipping a button that writes a broken image link.
## - **No `obsidian://` link, no wikilink generation, no two-way sync.** The
##   first two are Obsidian-specific (owner, 2026-08-18: nothing may require
##   Obsidian); the third is §33's explicit V1 non-goal.
## - **No POI.** Not a ported concept — the same absence
##   `place_editor_window.gd`'s own footer states.

var app                       ## `DccApp`
var bridge: EngineBridge

## The entity this window is scoped to. Empty `kind` means the overview.
var _kind := ""
var _entity_id := 0
var _entity_label := ""

## The link currently open in the reader (§29). Empty means none.
var _reader_link := ""
var _reader_edit: TextEdit

## The New-note-from-a-template picker's current template (VA-02).
var _pick_template := ""

## The attach form's current pick.
var _pick_file := ""
var _pick_heading := ""

## The Cartalith-feedback checkbox set (§20), by export-field key.
var _selected_fields := {}

var _body: VBoxContainer
var _phone := false
var _phone_title: Label

## Emitted after anything that changes the link store, so the host can
## persist it. The window never touches the disk itself.
signal store_changed


func setup(a, b: EngineBridge) -> void:
	app = a
	bridge = b
	title = "Markdown vault"
	size = Vector2i(560, 720)
	min_size = Vector2i(380, 460)
	max_size = Vector2i(760, 900)
	_phone = DccWidgets.phone_window(self, a)

	var root := VBoxContainer.new()
	root.add_theme_constant_override("separation", 0)
	add_child(root)
	var scroll := ScrollContainer.new()
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	root.add_child(scroll)
	if _phone:
		_phone_title = DccWidgets.phone_head(root, "Markdown vault", "linked notes")
	var pad := MarginContainer.new()
	for side in ["left", "top", "right", "bottom"]:
		pad.add_theme_constant_override("margin_" + side, 12)
	pad.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	scroll.add_child(pad)
	_body = VBoxContainer.new()
	_body.add_theme_constant_override("separation", 4)
	_body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	pad.add_child(_body)


## Opens scoped to one entity. `kind` is `"settlement"`, `"province"` or
## `"continent"`; `entity_id` is that kind's own id (a settlement's **tid**,
## not its index — the index shifts when another settlement is deleted and a
## link must survive that).
func open_for(kind: String, entity_id: int, label: String) -> void:
	_kind = kind
	_entity_id = entity_id
	_entity_label = label
	_reader_link = ""
	_pick_file = ""
	_pick_heading = ""
	_selected_fields = {}
	_rebuild()
	if not DccWidgets.phone_present(self, app):
		popup_centered()


## Opens the overview: the vault connection and every link in the store.
func open_overview() -> void:
	open_for("", 0, "")


func _clear() -> void:
	for c in _body.get_children():
		_body.remove_child(c)
		c.queue_free()
	_reader_edit = null


func _rebuild() -> void:
	_clear()
	var scoped := _kind != ""
	title = "Markdown vault — %s" % _entity_label if scoped else "Markdown vault"
	if _phone_title != null:
		_phone_title.text = (_entity_label if scoped else "Markdown vault").to_upper()

	var info := bridge.vault_info()
	_build_connection(info)
	if scoped:
		if bool(info.get("bound", false)):
			_build_create()
			_build_attach()
		_build_links()
		if _reader_link != "":
			_build_reader()
			_build_feedback()
	else:
		_build_overview()
	_build_footer()
	if _phone:
		app.phone_fit(self, 1.0)


# -- Connection (§7) --------------------------------------------------------

func _build_connection(info: Dictionary) -> void:
	var sec := DccWidgets.section(_body, "Vault")
	var bound: bool = bool(info.get("bound", false))
	var name := String(info.get("display_name", ""))
	if bound:
		DccWidgets.note(sec, "✓ Connected — %s\n%s" % [name, String(info.get("root", ""))])
	elif name != "":
		## §27 "Unbound": the project knows the vault, this device does not.
		DccWidgets.note(sec, "● %s is known to this project but not connected on this device.\nReconnect it to read or write; links and cached text stay readable meanwhile." % name)
	else:
		DccWidgets.note(sec, "No Markdown vault connected. Any folder of .md files works — Obsidian is one such folder, and nothing here requires it.")

	var connect_btn := DccWidgets.action(sec, "Connect vault…" if not bound else "Connect a different folder…", _browse_vault)
	connect_btn.tooltip_text = "Cartalith reads only the folder you choose here, and never writes to it without an explicit action and a preview."
	if bound:
		var dis := DccWidgets.action(sec, "Disconnect", func():
			bridge.vault_disconnect()
			store_changed.emit()
			_rebuild())
		dis.tooltip_text = "Drops this device's binding. The links themselves survive — that is the difference between disconnecting and detaching."


func _browse_vault() -> void:
	DccBrowseDialog.choose_folder(app, "Select markdown vault folder", "",
		"Cartalith reads .md files from this folder and writes only where you tell it to.",
		func(path: String):
			var r := bridge.vault_connect(path, "")
			if not bool(r.get("ok", false)):
				app.set_status("hint", "Vault: %s" % String(r.get("error", "could not connect")), "accent")
			else:
				store_changed.emit()
			_rebuild())


# -- Creating a note (§16/§17, `GUI_GAP_REGISTER.md` VA-02) -----------------

## The one act in this window that puts a file in the vault that was not there
## before, and the only one that needs no preview -- because it cannot destroy
## anything. An existing path is refused outright by the engine.
##
## Registered as unbacked because "cartalith-vault attaches to notes that
## already exist and refuses a heading that does not -- deliberately". That
## boundary is about *editing*: the machine block is the only thing Cartalith
## rewrites unattended (§23). Creating a file is a different act, and this one
## copies the author's own template verbatim, substituting nothing but the
## entity's name -- every `[If applicable]` and `[Optional]` prompt survives
## for the author to answer.
##
## Templates come from the vault, not from this program. There is no registry
## and no bundled content: a `.md` with "template" in its path is a template,
## which is exactly how the owner's own `design/vault-templates/` names them.
func _build_create() -> void:
	var templates := bridge.vault_templates()
	if templates.is_empty():
		return
	var sec := DccWidgets.group(_body, "New note from a template", false)
	var labels: Array = []
	var rels: Array = []
	for t in templates:
		var d: Dictionary = t
		labels.append(String(d.get("label", "?")))
		rels.append(String(d.get("rel", "")))
	if _pick_template == "" or not rels.has(_pick_template):
		_pick_template = String(rels[0])
	DccWidgets.choice(sec, "Template", labels, maxi(0, rels.find(_pick_template)),
		func(i: int): _pick_template = String(rels[i]),
		"Every .md in this vault whose path contains \"template\". Cartalith ships none of its own -- your templates are yours.")

	var suggested := bridge.vault_suggested_path(_kind, _entity_label)
	var path_edit := LineEdit.new()
	path_edit.text = suggested
	path_edit.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 8)
	row.custom_minimum_size.y = 24
	row.tooltip_text = "Where the new note goes, relative to the vault folder. The suggestion follows the %s/{name}.md convention; edit it if your vault is arranged differently." % _kind.capitalize()
	var lab := DccTheme.mono_label("Path", "text_dim", DccTheme.FS_SMALL, 0)
	lab.custom_minimum_size.x = DccWidgets.ROW_LABEL_W
	row.add_child(lab)
	row.add_child(path_edit)
	sec.add_child(row)

	var create := DccWidgets.action(sec, "Create %s" % suggested.get_file(), func():
		var rel := path_edit.text.strip_edges()
		if rel == "":
			app.set_status("hint", "Give the new note a path.", "accent")
			return
		var r := bridge.vault_create_from_template(_pick_template, rel, _entity_label)
		if not bool(r.get("ok", false)):
			app.set_status("hint", "Create: %s" % String(r.get("error", "refused")), "accent")
			_rebuild()
			return
		## Attach is a separate act with its own validation -- but the user
		## asked for a note *for this entity*, so run it, and say if it fails
		## rather than leaving an orphan note they cannot see.
		var a := bridge.vault_attach(_kind, _entity_id, _entity_label, rel, "")
		if bool(a.get("ok", false)):
			_reader_link = String(a.get("link_id", ""))
			store_changed.emit()
			app.set_status("hint", "%s created from %s and linked to %s." % [rel, _pick_template, _entity_label], "text")
		else:
			app.set_status("hint", "%s created, but could not be linked: %s" % [rel, String(a.get("error", ""))], "accent")
		_pick_file = rel
		_rebuild())
	create.tooltip_text = "Copies the template verbatim with %s substituted for its name placeholder, then links it to this entity. Refuses if that path already exists -- nothing is ever overwritten." % _entity_label


# -- Attaching (§11, §12, §13) ---------------------------------------------

func _build_attach() -> void:
	var sec := DccWidgets.section(_body, "Attach a note")
	var files := bridge.vault_list_files(2000)
	if files.is_empty():
		DccWidgets.note(sec, "No .md files found in this vault folder.")
		return
	if _pick_file == "":
		_pick_file = files[0]
	var labels: Array = []
	for f in files:
		labels.append(f)
	DccWidgets.choice(sec, "File", labels, maxi(0, Array(files).find(_pick_file)),
		func(i: int):
			_pick_file = files[i]
			_pick_heading = ""
			_rebuild(),
		"Listed lazily and capped — the vault is never fully read into memory.")

	## §11's own priority order: whole document first, then a heading section.
	var headings := bridge.vault_file_headings(_pick_file)
	var h_labels: Array = ["Whole document"]
	var h_values: Array = [""]
	for h in headings:
		var d: Dictionary = h
		var lvl := int(d.get("level", 1))
		h_labels.append("%s%s" % ["  ".repeat(maxi(0, lvl - 1)), String(d.get("title", ""))])
		h_values.append(String(d.get("title", "")))
	DccWidgets.choice(sec, "Section", h_labels, maxi(0, h_values.find(_pick_heading)),
		func(i: int): _pick_heading = String(h_values[i]),
		"Arbitrary text ranges are not offered: a byte offset stops pointing at the right paragraph the moment the author edits the text above it.")

	var attach := DccWidgets.action(sec, "Attach to %s" % _entity_label, func():
		var r := bridge.vault_attach(_kind, _entity_id, _entity_label, _pick_file, _pick_heading)
		if not bool(r.get("ok", false)):
			app.set_status("hint", "Attach: %s" % String(r.get("error", "refused")), "accent")
		else:
			_reader_link = String(r.get("link_id", ""))
			store_changed.emit()
		_rebuild(), true)
	attach.tooltip_text = "Reads the selection now and records the source's timestamp and content hash, so Cartalith can tell later whether the note changed."


# -- Linked notes (§28) -----------------------------------------------------

const STATUS_TEXT := {
	"connected": "✓ Connected",
	"stale": "● Source changed",
	"local_changes": "● Local changes",
	"cached": "● Cached — source unavailable",
	"missing": "✕ Source missing",
	"unbound": "● Vault not connected on this device",
}


func _build_links() -> void:
	var sec := DccWidgets.section(_body, "Knowledge")
	var links := bridge.vault_links_for(_kind, _entity_id)
	if links.is_empty():
		DccWidgets.note(sec, "No notes attached to %s yet." % _entity_label)
		return
	for l in links:
		var d: Dictionary = l
		var lid := String(d.get("link_id", ""))
		var row := DccWidgets.group(sec, "%s — %s" % [String(d.get("path", "")), String(d.get("selection_label", ""))])
		DccWidgets.note(row, String(STATUS_TEXT.get(String(d.get("status", "")), String(d.get("status", "")))))
		var open := DccWidgets.action(row, "Open" if lid != _reader_link else "Close", func():
			_reader_link = "" if lid == _reader_link else lid
			_rebuild())
		open.tooltip_text = "Shows the imported text and, once it diverges, the write-back action."
		if String(d.get("status", "")) == "stale":
			## §14's own three-way prompt, minus Compare — see the footer.
			var reload := DccWidgets.action(row, "Reload source", func():
				var r := bridge.vault_reload_link(lid)
				if not bool(r.get("ok", false)):
					app.set_status("hint", "Reload: %s" % String(r.get("error", "")), "accent")
				else:
					store_changed.emit()
				_rebuild())
			reload.tooltip_text = "Discards the Cartalith working copy and re-reads the section from the vault. The vault is not written."
		var detach := DccWidgets.action(row, "Detach", func():
			bridge.vault_detach(lid)
			if lid == _reader_link:
				_reader_link = ""
			store_changed.emit()
			_rebuild())
		detach.tooltip_text = "Removes the link. The Markdown file is not touched — including any Cartalith block already written into it, which stays until you remove it explicitly."


# -- Reader / working copy (§29) -------------------------------------------

func _build_reader() -> void:
	var sec := DccWidgets.section(_body, "Working copy")
	_reader_edit = TextEdit.new()
	_reader_edit.text = bridge.vault_link_text(_reader_link)
	_reader_edit.custom_minimum_size.y = 200
	_reader_edit.wrap_mode = TextEdit.LINE_WRAPPING_BOUNDARY
	## Commit on focus-loss rather than per keystroke, the same reason
	## `place_editor_window.gd`'s name field gives: every commit is a
	## `Dictionary` round trip into the engine, and a rebuild mid-word would
	## steal focus.
	_reader_edit.focus_exited.connect(func():
		if _reader_edit != null:
			bridge.vault_set_link_text(_reader_link, _reader_edit.text)
			store_changed.emit())
	sec.add_child(_reader_edit)

	var save := DccWidgets.action(sec, "Save local copy", func():
		bridge.vault_set_link_text(_reader_link, _reader_edit.text)
		store_changed.emit()
		_rebuild())
	save.tooltip_text = "Stores the edit on the Cartalith side only. The Markdown file is untouched until you use the action below."

	var write := DccWidgets.action(sec, "Insert updated section into source…", func():
		bridge.vault_set_link_text(_reader_link, _reader_edit.text)
		_confirm_section_write(), true)
	write.tooltip_text = "§15's one write-back path: replaces only this section, previewed first, and refuses outright if the file changed since the preview."


## §16's seven steps, as one dialog: preview, then an explicit confirmation
## carrying the hash the preview was computed from.
func _confirm_section_write() -> void:
	var p := bridge.vault_preview_section_write(_reader_link)
	if not bool(p.get("ok", false)):
		app.set_status("hint", "Preview: %s" % String(p.get("error", "")), "accent")
		return
	_preview_dialog("Insert updated section", String(p.get("preview", "")),
		"Only the linked section is replaced. Everything else in the file is written back byte for byte.",
		func():
			var r := bridge.vault_write_section(_reader_link, String(p.get("hash", "")))
			if bool(r.get("ok", false)):
				app.set_status("hint", "Section written back to the vault.", "text_ghost")
				store_changed.emit()
			else:
				app.set_status("hint", "Write refused: %s" % String(r.get("error", "")), "accent")
			_rebuild())


# -- Cartalith feedback (§18-§20, §23) -------------------------------------

func _build_feedback() -> void:
	var sec := DccWidgets.section(_body, "Cartalith feedback")
	var fields := bridge.vault_export_fields(_kind, _entity_id)
	if fields.is_empty():
		DccWidgets.note(sec, "Nothing to export for this entity — generate a world first, or this entity no longer resolves.")
		return
	var values := bridge.vault_entity_values(_kind, _entity_id)
	DccWidgets.note(sec, "Cartalith owns a delimited block in the note and nothing outside it. Only fields this entity actually has are listed.")

	var group := ""
	var host: Control = sec
	for f in fields:
		var d: Dictionary = f
		var key := String(d.get("key", ""))
		if String(d.get("group", "")) != group:
			group = String(d.get("group", ""))
			host = DccWidgets.group(sec, group)
		if not _selected_fields.has(key):
			_selected_fields[key] = true
		DccWidgets.toggle(host, "%s — %s" % [String(d.get("label", key)), String(values.get(key, ""))],
			bool(_selected_fields[key]),
			func(v: bool): _selected_fields[key] = v)

	var preview := DccWidgets.action(sec, "Preview & write Cartalith block…", _confirm_block_write, true)
	preview.tooltip_text = "Writes a <!-- CARTALITH:BEGIN --> block into the linked note, replacing an earlier one if it is there. Plain Markdown — it renders the same in any editor."

	var fill := DccWidgets.action(sec, "Fill the note's own fields…", _confirm_field_fill)
	fill.tooltip_text = "The owner's 2026-08-18 amendment: Cartalith may also populate the author's own template fields (Type, Location, Size / Population). A field you have already filled is never overwritten — it is reported as skipped."


func _selected_keys() -> PackedStringArray:
	var out := PackedStringArray()
	for k in _selected_fields:
		if bool(_selected_fields[k]):
			out.append(String(k))
	return out


func _link_path() -> String:
	for l in bridge.vault_links_for(_kind, _entity_id):
		var d: Dictionary = l
		if String(d.get("link_id", "")) == _reader_link:
			return String(d.get("path", ""))
	return ""


func _confirm_block_write() -> void:
	var rel := _link_path()
	if rel == "":
		return
	var body := bridge.vault_block_body(_kind, _entity_id, _selected_keys())
	var p := bridge.vault_preview_block(rel, _kind, _entity_id, body)
	if not bool(p.get("ok", false)):
		app.set_status("hint", "Preview: %s" % String(p.get("error", "")), "accent")
		return
	var action := String(p.get("action", ""))
	_preview_dialog("Write Cartalith block", String(p.get("preview", "")),
		("A new block will be inserted below the note's title." if action == "inserted"
			else "The existing Cartalith block will be replaced. Nothing outside it changes."),
		func():
			var r := bridge.vault_write_block(rel, _kind, _entity_id, body, String(p.get("hash", "")))
			if bool(r.get("ok", false)):
				app.set_status("hint", "Cartalith block %s." % String(r.get("action", "written")), "text_ghost")
			else:
				app.set_status("hint", "Write refused: %s" % String(r.get("error", "")), "accent")
			_rebuild())


func _confirm_field_fill() -> void:
	var rel := _link_path()
	if rel == "":
		return
	var p := bridge.vault_preview_field_fill(rel, _kind, _entity_id, false)
	if not bool(p.get("ok", false)):
		app.set_status("hint", "Preview: %s" % String(p.get("error", "")), "accent")
		return
	var lines: Array = []
	for e in p.get("report", []):
		var d: Dictionary = e
		lines.append("%s — %s" % [String(d.get("field", "")), String(d.get("outcome", "")).replace("_", " ")])
	if lines.is_empty():
		app.set_status("hint", "This note has none of the template fields Cartalith can fill.", "text_ghost")
		return
	_preview_dialog("Fill the note's own fields", String(p.get("preview", "")),
		"\n".join(PackedStringArray(lines)) + "\n\nA field you had already filled is skipped, never overwritten.",
		func():
			var r := bridge.vault_write_field_fill(rel, _kind, _entity_id, false, String(p.get("hash", "")))
			if bool(r.get("ok", false)):
				app.set_status("hint", "Template fields filled.", "text_ghost")
			else:
				app.set_status("hint", "Write refused: %s" % String(r.get("error", "")), "accent")
			_rebuild())


## §23 rule 5 and §16 step 4-5, in one place so no write path can skip them.
func _preview_dialog(dialog_title: String, preview: String, note: String, on_confirm: Callable) -> void:
	var dlg := ConfirmationDialog.new()
	dlg.title = dialog_title
	dlg.size = Vector2i(620, 620)
	dlg.get_ok_button().text = "Write to Markdown"
	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 6)
	dlg.add_child(col)
	DccWidgets.note(col, note)
	var te := TextEdit.new()
	te.text = preview
	te.editable = false
	te.custom_minimum_size.y = 460
	te.size_flags_vertical = Control.SIZE_EXPAND_FILL
	col.add_child(te)
	dlg.confirmed.connect(func():
		on_confirm.call()
		dlg.queue_free())
	dlg.canceled.connect(dlg.queue_free)
	app.add_child(dlg)
	dlg.popup_centered()


# -- Overview ---------------------------------------------------------------

func _build_overview() -> void:
	var sec := DccWidgets.section(_body, "All linked notes")
	var links := bridge.vault_all_links()
	if links.is_empty():
		DccWidgets.note(sec, "Nothing is linked yet. Open a settlement, province or continent and attach a note from there — the vault belongs in the entity's own panel, not in a utility window.")
		return
	for l in links:
		var d: Dictionary = l
		var kind := String(d.get("entity_kind", ""))
		var eid := int(d.get("entity_id", 0))
		var label := String(d.get("entity_label", ""))
		var b := DccWidgets.action(sec, "%s %s — %s (%s)" % [
			kind.capitalize(), label, String(d.get("path", "")),
			String(STATUS_TEXT.get(String(d.get("status", "")), ""))],
			func(): open_for(kind, eid, label))
		b.alignment = HORIZONTAL_ALIGNMENT_LEFT


func _build_footer() -> void:
	DccWidgets.note(_body,
		"Not built here, each for a stated reason: the map snapshot (§21) — it needs a crop of "
		+ "the live renderer at three radii, held as its own milestone rather than shipped as a "
		+ "broken image link; Compare-with-source (§14) — no diff view exists in this shell yet, "
		+ "so a changed source offers Reload or Keep, which are the two actions that cannot lose "
		+ "work; two-way sync and an Obsidian plugin — an explicit V1 non-goal and a deferred "
		+ "wish. Links are stored beside your Cartalith profile, not inside the .zip save: the "
		+ "save format carries no civilisation layer, so a link inside one would come back "
		+ "pointing at settlements that no longer exist.")
