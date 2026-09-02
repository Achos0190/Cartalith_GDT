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
## exactly four write buttons in this file, each behind a preview whose hash
## is handed back to the write — so a note edited in the user's own editor
## between the preview and the confirmation refuses instead of overwriting.
## The engine enforces that; this window's job is to never offer a write
## without having shown what it would do.
##
## ## "Confirm always" suppresses the dialog and never the guard
##
## The owner's 2026-08-25 direction ends *"the prompt should have an option to
## confirm always"*, and `_preview_dialog` carries it: a ticked checkbox sets
## one of `vault_write_prefs()`' three flags, and a set flag means the next
## write of that kind runs without stopping to ask. It does **not** mean the
## next write runs unchecked. Every caller still computes its `vault_preview_*`
## first and still hands that preview's `hash` to the write, because the hash
## *is* the guard — see `_preview_dialog`'s own comment for why the preview
## call could not be moved inside the dialog even if it looked tidier there.
##
## ## Search (§9, the same 2026-08-25 message's first sentence)
##
## `_build_search` is the panel half of `vault_search`. It reports what the
## engine actually did rather than only what it found: content search runs off
## the backlink index, so with no index only *names* were looked at, and
## answering that with a bare "no results" would be telling the user their
## vault does not contain a word nobody searched for.
##
## ## The map snapshot (§21, §22 — milestone 2, 2026-09-02)
##
## `_build_snapshots` is §21's immediate/local/regional crop, and it is the
## one section here that writes a file into the vault that is not a `.md`. Its
## own doc comment carries §22's "must not silently pollute the Markdown
## vault" and how the folder is accepted. The three Map checkboxes in
## `_build_feedback` appear only once a snapshot exists — `export::offer`
## filters on the value being there, so the block can never carry a link to an
## image that was never written.
##
## ## What is deliberately not here
##
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

## The attach form's current pick, and whether its "what does this note say?"
## readout is open. Closed by default because opening it opens the file: §31's
## rule is that browsing never reads, so a per-rebuild disk read has to be
## something the user asked for.
var _pick_file := ""
var _pick_heading := ""
var _pick_data := false

## The vault search. `_search_result` is the last `vault_search` answer held
## verbatim — `indexed`/`scanned`/`truncated` included, because the panel has
## to report those and not only `hits`.
var _search_query := ""
var _search_result := {}
var _search_box: VBoxContainer
var _search_open_rel := ""

## Whether the device-local write preferences have been pulled off disk yet.
var _prefs_loaded := false

## The Cartalith-feedback checkbox set (§20), by export-field key.
var _selected_fields := {}

## §22's proposed structure, and the folder the user is currently accepting.
##
## `_snapshot_dir` is session state and deliberately not persisted anywhere: it
## is a *choice being made*, not a setting, and §22's requirement is that the
## person sees the destination and presses Generate — which is only true if the
## field is on screen at the moment of the write. A remembered folder would
## turn the second snapshot into a silent write to a path nobody re-read.
const DEFAULT_SNAPSHOT_DIR := ".cartalith/maps"

## The snapshot's edge, in pixels. One number rather than a control: §21 says
## the *radius* may be configurable and says nothing about resolution, and 512
## is what a note renders inline at without a scrollbar in either Obsidian or a
## plain Markdown viewer.
const SNAPSHOT_PX := 512

var _snapshot_dir := ""

## The last index Refresh/Rebuild result, shown on the Index section until the
## next one. A one-shot line, not a persistent state: the numbers above it are
## the state.
var _index_feedback := ""

var _body: VBoxContainer
var _phone := false
var _phone_title: Label

## Emitted after anything that changes the link store, so the host can
## persist it. The window never touches the disk itself.
signal store_changed


func setup(a, b: EngineBridge) -> void:
	app = a
	bridge = b
	_load_prefs_once()
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
	_pick_data = false
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
	## Nulled, not left dangling: `_fill_search_results()` can be reached from a
	## callback that outlives the rebuild that freed the box it was writing into.
	_search_box = null


func _rebuild() -> void:
	_clear()
	var scoped := _kind != ""
	title = "Markdown vault — %s" % _entity_label if scoped else "Markdown vault"
	if _phone_title != null:
		_phone_title.text = (_entity_label if scoped else "Markdown vault").to_upper()

	var info := bridge.vault_info()
	var bound := bool(info.get("bound", false))
	_build_connection(info)
	## Search sits directly under the connection and above everything else in
	## both modes: the owner's sentence starts with finding the note, and in the
	## scoped view "find it, then attach it" is the order the two acts happen in.
	if bound:
		_build_search()
	if scoped:
		if bound:
			_build_create()
			_build_attach()
		_build_links()
		## Above the reader, and deliberately not inside it: a snapshot is a
		## picture of the *place*, so it exists whether or not a note is open,
		## and the Map checkboxes in `_build_feedback` appear only once one has
		## been generated. Ordering it here is what makes that sequence visible.
		if bound:
			_build_snapshots()
		if _reader_link != "":
			_build_reader()
			_build_feedback()
	else:
		_build_overview()
	_build_write_prefs()
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


# -- Searching (§9, the owner's 2026-08-25 direction) ----------------------

## The search field, and the three numbers beside the results that stop it
## lying.
##
## `vault_search` answers `{indexed, scanned, truncated, hits}`, and only
## `hits` is the part a naive panel would draw. The other three are the
## difference between "your vault does not contain that" and "nobody looked":
##
## - **`indexed` false** means the backlink index has never been built, so the
##   engine matched *names only*. An empty answer there is not an answer, and
##   the panel says so and offers the one press that fixes it.
## - **`scanned`** is how many notes were actually opened to confirm a content
##   match. It is the cost the user paid, and it is also the honest bound on
##   the search: notes past it were never looked at.
## - **`truncated`** means a cap cut the answer short. A capped search that
##   presents itself as complete is worse than one that admits it stopped.
##
## The results live in their own container, refilled in place. A `_rebuild()`
## per keystroke would be the obvious wiring and is the wrong one — it frees
## the `LineEdit` being typed into and takes the caret with it.
func _build_search() -> void:
	var sec := DccWidgets.section(_body, "Find a note")
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 8)
	row.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	sec.add_child(row)
	var field := LineEdit.new()
	field.placeholder_text = "a note's name, or a word inside one"
	field.text = _search_query
	field.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	DccWidgets.well(field)
	## Typing records the query and searches nothing; Enter or the button runs
	## it. §31 forbids walking the vault casually, and a search-as-you-type
	## field would open up to `max_reads` files on every letter.
	field.text_changed.connect(func(t: String): _search_query = t)
	field.text_submitted.connect(func(t: String):
		_search_query = t
		_run_search())
	row.add_child(field)
	var go := DccWidgets.action(row, "Search", _run_search)
	go.tooltip_text = "Names always. The text inside notes only once the content index has been built — which is the one thing in this window that reads the whole vault, and which the results below offer when it is missing."

	_search_box = VBoxContainer.new()
	_search_box.add_theme_constant_override("separation", 2)
	_search_box.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	sec.add_child(_search_box)
	_fill_search_results()


func _run_search() -> void:
	var q := _search_query.strip_edges()
	_search_open_rel = ""
	_search_result = {} if q == "" else bridge.vault_search(q, 0, 0)
	_fill_search_results()


func _fill_search_results() -> void:
	if _search_box == null or not is_instance_valid(_search_box):
		return
	for c in _search_box.get_children():
		_search_box.remove_child(c)
		c.queue_free()

	if _search_result.is_empty():
		DccWidgets.note(_search_box, "Nothing searched yet. Type a note's name, or — once the content index exists — a word from inside one, and press Enter.")
		_fit_search_box()
		return
	if not bool(_search_result.get("ok", false)):
		DccWidgets.note(_search_box, "Search: %s" % String(_search_result.get("error", "refused")))
		_fit_search_box()
		return

	var hits: Array = _search_result.get("hits", [])
	var indexed := bool(_search_result.get("indexed", false))
	var scanned := int(_search_result.get("scanned", 0))
	var truncated := bool(_search_result.get("truncated", false))
	## The engine's other silent narrowing, and the only one that is not in the
	## answer: under three characters it matches names and stops, because
	## confirming a two-letter query means opening the whole vault. A `scanned`
	## of 0 has to be explained by *something*, and this is the explanation the
	## user can act on.
	var short_query := _search_query.strip_edges().length() < 3

	if not indexed:
		DccWidgets.note(_search_box, "Names only — this vault has no content index, so the inside of a note was not looked at. %s" % (
			"Nothing here has that in its name." if hits.is_empty() else "There may be more inside the notes."))
		var build := DccWidgets.action(_search_box, "Build the content index, then search again", func():
			## `_refresh_index()` rebuilds the whole panel, which frees the box
			## this button lives in — so the re-search runs after it, against
			## the new one, and the query survives because it is window state
			## rather than the field's.
			_refresh_index()
			if _search_query.strip_edges() != "":
				_run_search())
		build.tooltip_text = "Reads every note in the vault once and keeps its size, modified time, links and a word fingerprint — never the prose. After that a refresh only re-opens the files that changed."
	elif short_query:
		DccWidgets.note(_search_box, "Names only — a query under three characters is not confirmed against the text of a note, because doing that means opening every one of them. %s" % [
			"Nothing has that in its name." if hits.is_empty() else "Add a letter to search inside the notes too."])
	elif hits.is_empty():
		DccWidgets.note(_search_box, "No match, in any note's name or in the %d note%s opened to check." % [scanned, "" if scanned == 1 else "s"])
	else:
		DccWidgets.note(_search_box, "%d match%s · %d note%s opened to confirm a match in the text." % [
			hits.size(), "" if hits.size() == 1 else "es", scanned, "" if scanned == 1 else "s"])
	if truncated:
		DccWidgets.note(_search_box, "%s Cut short by the cap: there are more matches, and notes past the scan limit were never opened. Narrow the query rather than reading this as the whole answer." % DccIcons.SYMBOLS["warn_tri"])

	for h in hits:
		var d: Dictionary = h
		var rel := String(d.get("rel", ""))
		var in_name := bool(d.get("in_name", false))
		## The shell's own two marks for a row that opens — `group()` uses the
		## first, every menu the second. Not a new glyph pair.
		var mark: String = DccIcons.SYMBOLS["caret"] if _search_open_rel == rel \
			else DccIcons.SYMBOLS["expand"]
		var open := DccWidgets.action(_search_box, "%s %s" % [mark, rel], func():
			_search_open_rel = "" if _search_open_rel == rel else rel
			_fill_search_results())
		open.alignment = HORIZONTAL_ALIGNMENT_LEFT
		open.tooltip_text = "Shows what this note holds — its frontmatter and its filled-in fields — without attaching anything."
		## `in_name` is the certain half: a name hit cost no read at all, a text
		## hit was confirmed by opening the file. Saying which is not decoration
		## — it is the difference between a match the engine is sure of and one
		## it narrowed to.
		var excerpt := String(d.get("excerpt", ""))
		DccWidgets.note(_search_box, "    %s%s" % [
			"in the name" if in_name else "in the text", "" if excerpt == "" else " · " + excerpt])
		if _search_open_rel == rel:
			var g := DccWidgets.group(_search_box, "what this note holds", true)
			_build_note_data(g, bridge.vault_file_data(rel),
				"No frontmatter and no filled-in template fields — this note is prose, which Cartalith reads and does not model.")
			if _kind != "":
				var use := DccWidgets.action(g, "Attach this one to %s…" % _entity_label, func():
					_pick_file = rel
					_pick_heading = ""
					_pick_data = false
					_rebuild())
				use.tooltip_text = "Selects it in Attach a note below, where the section and the attach itself are still yours to confirm. Nothing is attached by this button."
	_fit_search_box()


## The results are built after `_rebuild()` has already run `phone_fit` over the
## window, so the walk has to be repeated on the new subtree or every row in it
## lands below §13's 44 dp floor. Cheap: `phone_fit` marks what it has visited.
func _fit_search_box() -> void:
	if _phone and _search_box != null and is_instance_valid(_search_box):
		app.phone_fit(_search_box, 1.0)


# -- What a note holds (`vault_file_data` / `vault_link_data`) --------------

## The two maps, drawn as two lists and never merged.
##
## `type: town` in the frontmatter and `**Type:** City` in the body are two
## authoring surfaces that can legitimately disagree, and merging them needs a
## precedence rule nobody asked for. Cartalith shows both and says which is
## which; deciding between them is the author's.
func _build_note_data(parent: Control, data: Dictionary, empty_note: String) -> void:
	if not bool(data.get("ok", false)):
		DccWidgets.note(parent, "Could not read this note: %s" % String(data.get("error", "")))
		return
	var frontmatter: Dictionary = data.get("frontmatter", {})
	var fields: Dictionary = data.get("fields", {})
	if frontmatter.is_empty() and fields.is_empty():
		DccWidgets.note(parent, empty_note)
		return
	if not frontmatter.is_empty():
		DccWidgets.note(parent, "Frontmatter")
		for k in frontmatter:
			DccWidgets.note(parent, "    %s: %s" % [String(k), String(frontmatter[k])])
	if not fields.is_empty():
		DccWidgets.note(parent, "Fields the author filled in")
		for k in fields:
			DccWidgets.note(parent, "    %s: %s" % [String(k), String(fields[k])])


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
			_pick_data = false
			_rebuild(),
		"Listed lazily and capped — the vault is never fully read into memory.")

	## §17's reading half, at the one moment it is worth the read: what this
	## note actually holds, *before* the user commits to attaching it. Opened by
	## request rather than drawn always, because `vault_file_data` opens the
	## file and this section is rebuilt on every pick change.
	if _pick_data:
		var g := DccWidgets.group(sec, "what %s holds" % _pick_file.get_file(), true)
		_build_note_data(g, bridge.vault_file_data(_pick_file),
			"No frontmatter and no filled-in template fields. Attaching still copies the prose — this readout is about the parts a program can read back.")
	else:
		DccWidgets.text_button(sec, "What does this note hold?", func():
			_pick_data = true
			_rebuild())

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
			## §14's own three-way prompt, all three now: Reload and Compare
			## are buttons; Keep is simply pressing neither, which is why
			## milestone 1 needed no button for it at all.
			var reload := DccWidgets.action(row, "Reload source", func(): _reload_link(lid))
			reload.tooltip_text = "Discards the Cartalith working copy and re-reads the section from the vault. The vault is not written."
			var link_rel := String(d.get("path", ""))
			var compare := DccWidgets.action(row, "Compare…", func(): _compare_link(lid, link_rel))
			compare.tooltip_text = "A line-by-line diff between %s as it is right now and your working copy, so you can judge before choosing Reload or Keep." % link_rel.get_file()
		var detach := DccWidgets.action(row, "Detach", func():
			bridge.vault_detach(lid)
			if lid == _reader_link:
				_reader_link = ""
			store_changed.emit()
			_rebuild())
		detach.tooltip_text = "Removes the link. The Markdown file is not touched — including any Cartalith block already written into it, which stays until you remove it explicitly."
	_build_entity_data(sec)


## The owner's sentence read back: *"The information then gets copied to a
## json."* This is where that copy comes out.
##
## It reads the copy and never the disk, which is the whole reason copying was
## worth doing — it still answers with the vault on a drive that is not plugged
## in (§27's Unbound). Two consequences worth stating rather than discovering:
##
## - **Not deduplicated.** Two notes on one settlement may disagree, so every
##   row carries the note it came from and the disagreement stays visible and
##   attributable instead of being silently resolved.
## - **Empty for a link made before 2026-08-25.** *Reload source* fills it. The
##   engine defaults the field rather than bumping a format version, so an old
##   sidecar loads and simply has nothing here yet.
func _build_entity_data(sec: Control) -> void:
	var rows := bridge.vault_entity_data(_kind, _entity_id)
	if rows.is_empty():
		return
	var g := DccWidgets.group(sec, "what the notes say", false)
	DccWidgets.note(g, "Copied out of the attached notes when each was attached or last reloaded, and readable with the vault disconnected. Cartalith holds this; it does not act on it — nothing here sets a population or a name in the world.")
	for r in rows:
		var d: Dictionary = r
		DccWidgets.note(g, "    %s: %s    (%s · %s)" % [
			String(d.get("key", "")), String(d.get("value", "")),
			String(d.get("origin", "")), String(d.get("rel", ""))])


# -- Compare (§14's third action, `MARKDOWN_VAULT_SCOPE.md` milestone 5) ----

## §14's "Reload source", factored out so the stale row's own button and
## Compare's own button (below) share one call rather than risk two copies
## drifting apart. Behaviour is unchanged from before this pass: discards
## the working copy, re-reads the section, and never touches the file.
func _reload_link(lid: String) -> void:
	var r := bridge.vault_reload_link(lid)
	if not bool(r.get("ok", false)):
		app.set_status("hint", "Reload: %s" % String(r.get("error", "")), "accent")
	else:
		store_changed.emit()
	_rebuild()


## A guard on the DP table the diff below builds, the same shape as
## `_run_search`'s own `truncated` — a vault note is realistically tens to a
## few hundred lines, so this is a fence against a mistakenly huge
## attachment, not a limit anyone should ever actually see.
const DIFF_MAX_CELLS := 1_000_000
## Lines of unchanged context kept on each side of a change before the run
## between two changes collapses to a count — `diff -U3`'s own idea: enough
## to place a change without echoing a whole unchanged note back at someone
## who already has it.
const DIFF_CONTEXT := 3

## §14's Compare. Diffs the source file as it stands right now against what
## the file would read **if the working copy were written into it** — not
## the raw working text against the raw file. That choice is what lets this
## function skip reimplementing heading/fence parsing to find the linked
## section's own boundaries a second time: `vault_preview_section_write`
## already builds exactly that document, through the same
## `markdown::replace_section` the real write uses, so every byte outside
## this link's own section is identical on both sides and the diff shows
## only what this link actually touches.
##
## Both calls this makes are read-only (`&self` on the Rust side) — unlike
## `vault_reload_link`, which updates the link's *stored* hash and timestamp
## as a side effect of re-reading. That update is exactly right for an
## explicit Reload and exactly wrong for "let me just look first": it would
## silently clear the very Stale status this button is offered from, so a
## Compare that merely looked would make the row stop saying the source had
## changed. This function never calls `vault_reload_link`, so looking never
## changes what a link reports — only the dialog's own Reload button, wired
## through `_reload_link` above, does that, and only once pressed.
func _compare_link(lid: String, rel: String) -> void:
	var p := bridge.vault_preview_section_write(lid)
	if not bool(p.get("ok", false)):
		app.set_status("hint", "Compare: %s" % String(p.get("error", "")), "accent")
		return
	## CRLF-normalised before splitting, for display only — nothing here is
	## written back. A Windows-authored note is CRLF throughout; the spliced
	## section came from the working copy's own line endings, which a
	## `TextEdit` normalises to `\n`, so without this every line of an
	## otherwise-identical section would show as changed on the ending alone.
	var source_text := bridge.vault_read_file(rel).replace("\r\n", "\n")
	var working_text := String(p.get("preview", "")).replace("\r\n", "\n")
	var old_lines := source_text.split("\n")
	var new_lines := working_text.split("\n")
	if old_lines.size() * new_lines.size() > DIFF_MAX_CELLS:
		_compare_dialog(lid, rel, [], true)
		return
	_compare_dialog(lid, rel, _lcs_diff(old_lines, new_lines), false)


## Longest-common-subsequence line diff — the classic O(n·m) DP table plus a
## backtrack, which is the whole algorithm a line-level diff needs and the
## only one this window builds: no third-party widget, no new crate. `old`
## is the source file as it reads right now; `new` is what it would read
## with the working copy written back — `_compare_link`'s own header says
## why those two and not the raw working text.
##
## One flat `PackedInt32Array` rather than an `Array` of rows, deliberately:
## `dp[i][j] = x` through a plain `Array` of packed arrays risks writing
## through a copy GDScript handed back rather than the stored element, and
## this project has already paid once for a silent-loss bug shaped exactly
## like that (`vault_store.gd`'s own KV-04 header). A single packed array
## has one level of indexing and no such question to get wrong.
func _lcs_diff(old: PackedStringArray, new: PackedStringArray) -> Array:
	var n := old.size()
	var m := new.size()
	var w := m + 1
	var dp := PackedInt32Array()
	dp.resize((n + 1) * w)
	for i in range(n - 1, -1, -1):
		for j in range(m - 1, -1, -1):
			dp[i * w + j] = (dp[(i + 1) * w + j + 1] + 1) if old[i] == new[j] \
				else maxi(dp[(i + 1) * w + j], dp[i * w + j + 1])
	var ops: Array = []
	var i := 0
	var j := 0
	while i < n and j < m:
		if old[i] == new[j]:
			ops.append({"op": "eq", "text": old[i]})
			i += 1
			j += 1
		elif dp[(i + 1) * w + j] >= dp[i * w + j + 1]:
			ops.append({"op": "del", "text": old[i]})
			i += 1
		else:
			ops.append({"op": "add", "text": new[j]})
			j += 1
	while i < n:
		ops.append({"op": "del", "text": old[i]})
		i += 1
	while j < m:
		ops.append({"op": "add", "text": new[j]})
		j += 1
	return ops


## §14's Compare dialog: the diff, and nothing to confirm. Looking cannot
## lose work by construction (see `_compare_link`'s own header), so the only
## actions are dismissing and the same Reload the stale row already offers,
## wired through the shared `_reload_link` rather than a second copy of it.
func _compare_dialog(lid: String, rel: String, ops: Array, too_large: bool) -> void:
	var dlg := AcceptDialog.new()
	dlg.title = "Compare — %s" % rel.get_file()
	dlg.size = Vector2i(680, 640)
	dlg.get_ok_button().text = "Close"
	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 6)
	dlg.add_child(col)
	DccWidgets.note(col,
		("%s as it reads on disk right now, against your working copy. " % rel.get_file())
		+ "\"+\" lines are only in your working copy — Reload source would discard them. "
		+ "\"-\" lines are only in the file — Reload source would bring them in.")
	var scroll := ScrollContainer.new()
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	col.add_child(scroll)
	var diff_col := VBoxContainer.new()
	diff_col.add_theme_constant_override("separation", 0)
	diff_col.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	scroll.add_child(diff_col)
	var has_diff := false
	for o in ops:
		if String((o as Dictionary).get("op", "")) != "eq":
			has_diff = true
			break
	if too_large:
		DccWidgets.note(diff_col,
			"This note is too large to diff line by line in this view. Reload source, or compare it in your own editor.")
	elif not has_diff:
		DccWidgets.note(diff_col,
			"No difference in what this link covers — the file changed somewhere else, or only its timestamp moved.")
	else:
		_build_diff_rows(diff_col, ops)
	var footer := HBoxContainer.new()
	footer.add_theme_constant_override("separation", 8)
	col.add_child(footer)
	var reload := DccWidgets.action(footer, "Reload source", func():
		dlg.queue_free()
		_reload_link(lid))
	reload.tooltip_text = "Discards the Cartalith working copy and re-reads the section from the vault. The vault is not written."
	dlg.confirmed.connect(dlg.queue_free)
	dlg.canceled.connect(dlg.queue_free)
	app.add_child(dlg)
	if _phone:
		app.phone_fit(dlg, 1.0)
	dlg.popup_centered()


## `ops` collapsed to context around each change — `diff -U3`'s own idea: a
## run of unchanged lines longer than `DIFF_CONTEXT` on both sides shows only
## its two edges, with a count standing in for the rest.
func _build_diff_rows(container: Control, ops: Array) -> void:
	var n := ops.size()
	var i := 0
	while i < n:
		var d: Dictionary = ops[i]
		if String(d.get("op", "")) != "eq":
			_diff_row(container, d)
			i += 1
			continue
		var j := i
		while j < n and String((ops[j] as Dictionary).get("op", "")) == "eq":
			j += 1
		var run := j - i
		if run <= DIFF_CONTEXT * 2:
			for k in range(i, j):
				_diff_row(container, ops[k])
		else:
			for k in range(i, i + DIFF_CONTEXT):
				_diff_row(container, ops[k])
			var hidden := run - DIFF_CONTEXT * 2
			DccWidgets.note(container, "    %s %d unchanged line%s" % [
				DccIcons.SYMBOLS["overflow"], hidden, "" if hidden == 1 else "s"])
			for k in range(j - DIFF_CONTEXT, j):
				_diff_row(container, ops[k])
		i = j


## One diff line. `block` carries the risk side — content only in the
## working copy, which Reload source would throw away — because this
## palette has no green to pair with a red the way a version-control diff
## usually would: `DccTheme`'s own header records that `--good` is declared
## in the design canvas and used nowhere in it, so it was never imported
## here. `accent` marks the opposite case (only in the file) rather than a
## colour this shell does not have, and the leading `+`/`-` plus the dialog's
## own legend above carry the meaning too, so it is not colour-only.
func _diff_row(container: Control, d: Dictionary) -> void:
	var op := String(d.get("op", "eq"))
	var text := String(d.get("text", ""))
	var prefix := "  "
	var token := "text_dim"
	match op:
		"add":
			prefix = "+ "
			token = "block"
		"del":
			prefix = "- "
			token = "accent"
	var lbl := DccTheme.mono_label(prefix + text, token, DccTheme.FS_SMALL)
	lbl.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	lbl.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	if op == "eq":
		container.add_child(lbl)
		return
	var wrap := PanelContainer.new()
	var sb := DccTheme.flat(Color(DccTheme.c(token), 0.09))
	sb.content_margin_left = 4
	sb.content_margin_right = 4
	sb.content_margin_top = 1
	sb.content_margin_bottom = 1
	wrap.add_theme_stylebox_override("panel", sb)
	wrap.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	wrap.add_child(lbl)
	container.add_child(wrap)


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

	## The per-link view of the same copy `_build_entity_data` shows for the
	## whole entity. Memory only — `vault_link_data` reads the link, not the
	## file — so unlike the attach readout this one costs nothing to draw.
	var g := DccWidgets.group(sec, "what this note holds", false)
	_build_note_data(g, bridge.vault_link_data(_reader_link),
		"Nothing structured was copied from this note. Either it has no frontmatter and no filled-in fields, or the link predates Cartalith copying them — Reload source above fills it in that case.")


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
			_rebuild(),
		"section")


# -- Cartalith feedback (§18-§20, §23) -------------------------------------

# -- The map snapshot (§21, §22) -------------------------------------------

## §21's immediate/local/regional crop of the live renderer, and §22's
## explicit acceptance of where it goes.
##
## ## What "user-accepted location" is, here
##
## §22 is emphatic — *"the user must explicitly accept the proposed structure
## or choose another location"*, and *"the integration must not silently
## pollute the Markdown Vault"*. So the folder is a visible, editable field
## prefilled with §22's own proposed `.cartalith/maps`, and nothing is written
## until a Generate button is pressed with that folder on screen. There is no
## default-on, no background generation and no first-run write.
##
## ## Inside the vault, and why that is not a shortcut
##
## The folder is relative to the vault root and `vault_snapshot` refuses
## anything that escapes it (`FsVault::resolve`, the same containment check
## that refuses `..` for a note). That is not laziness about §22's
## "user-selected location": the path Cartalith writes into the note has to be
## one the note can still resolve on another machine, and an absolute path to
## somewhere else on this disk would be a §5 violation living in the user's
## own file, where nothing here could later correct it.
func _build_snapshots() -> void:
	var radii := bridge.vault_snapshot_radii(_kind, _entity_id)
	if radii.is_empty():
		return
	var sec := DccWidgets.group(_body, "Map snapshot", false)
	## `cells` is 0 when the world does not say how wide it is in km, which is
	## the one case a radius cannot be scaled honestly. Said out loud rather
	## than silently falling back to a cell count that would mean a different
	## distance in every world.
	var scaled := int((radii[0] as Dictionary).get("cells", 0)) > 0
	if not scaled:
		DccWidgets.note(sec, "No world is loaded, or it does not say how wide it is in kilometres — so \"local\" has no distance to mean. Generate a world first.")
		return

	var dir_edit := LineEdit.new()
	dir_edit.text = _snapshot_dir if _snapshot_dir != "" else DEFAULT_SNAPSHOT_DIR
	dir_edit.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	dir_edit.text_changed.connect(func(t: String): _snapshot_dir = t)
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 8)
	row.custom_minimum_size.y = 24
	row.tooltip_text = "Where snapshots go, relative to the vault folder. §22's own proposal is .cartalith/maps — change it if your vault is arranged differently. Nothing is written until you press a Generate button."
	var lab := DccTheme.mono_label("Folder", "text_dim", DccTheme.FS_SMALL, 0)
	lab.custom_minimum_size.x = DccWidgets.ROW_LABEL_W
	row.add_child(lab)
	row.add_child(dir_edit)
	sec.add_child(row)

	for r in radii:
		var d: Dictionary = r
		var radius := String(d.get("radius", ""))
		var have := String(d.get("path", ""))
		var caption := "%s — %d km across, %d cells" % [
			String(d.get("label", radius)), int(round(float(d.get("km", 0.0)) * 2.0)), int(d.get("cells", 0)) * 2 + 1]
		DccWidgets.note(sec, caption + ("\n✓ %s" % have if have != "" else "\n○ not generated"))
		var b := DccWidgets.action(sec, ("Regenerate " if have != "" else "Generate ") + radius, func():
			_generate_snapshot(radius, dir_edit.text.strip_edges()))
		b.tooltip_text = "Crops the map you are looking at — the same renderer, the same look — around this entity and writes a %d px PNG into the folder above. Regenerating replaces that file, so a note already pointing at it shows the new picture." % SNAPSHOT_PX


func _generate_snapshot(radius: String, subdir: String) -> void:
	var r := bridge.vault_snapshot(_kind, _entity_id, radius, subdir, SNAPSHOT_PX)
	if not bool(r.get("ok", false)):
		app.set_status("hint", "Snapshot: %s" % String(r.get("error", "refused")), "accent")
		return
	## The store changed — the snapshot's path is filed on it, and that is what
	## rides `vault.json` into the project archive.
	store_changed.emit()
	app.set_status("hint", "%s map written to %s (%d x %d)." % [
		radius, String(r.get("rel", "")), int(r.get("width", 0)), int(r.get("height", 0))], "text")
	_rebuild()


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
		## A Map field's value is the Markdown image that goes in the note,
		## `![](path)`. Shown here as the path alone: the checkbox is a list of
		## what this entity has, and `![](` in front of every map row is
		## syntax the reader has to skip past to find the answer. The note gets
		## the value verbatim either way -- this trims the *label*, never what
		## `vault_block_body` writes.
		var shown := String(values.get(key, ""))
		if shown.begins_with("![](") and shown.ends_with(")"):
			shown = shown.substr(4, shown.length() - 5)
		DccWidgets.toggle(host, "%s — %s" % [String(d.get("label", key)), shown],
			bool(_selected_fields[key]),
			func(v: bool): _selected_fields[key] = v)

	var preview := DccWidgets.action(sec, "Preview & write Cartalith block…", _confirm_block_write, true)
	preview.tooltip_text = "Writes a <!-- CARTALITH:BEGIN --> block into the linked note, replacing an earlier one if it is there. Plain Markdown — it renders the same in any editor."

	var fill := DccWidgets.action(sec, "Fill the note's own fields…", _confirm_field_fill)
	fill.tooltip_text = "The owner's 2026-08-18 amendment: Cartalith may also populate the author's own template fields (Type, Location, Size / Population). A field you have already filled is never overwritten — it is reported as skipped."

	var remove := DccWidgets.action(sec, "Remove the Cartalith block…", _confirm_block_remove)
	remove.tooltip_text = "Takes the block back out of the note, leaving every other byte alone. Previewed and confirmed like a write, because it is one — and it is the only act here that never learns to stop asking."


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
			_rebuild(),
		"block")


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
			_rebuild(),
		"field_fill")


## §32's "stale Cartalith block": the block taken back out of the note.
##
## The same preview-then-confirm treatment as every write here, because it is
## equally destructive — it edits the author's file — and two simplifications
## that are worth stating rather than hiding:
##
## 1. **There is no `vault_preview_block_remove`.** The hash a removal needs is
##    the note's content hash as it is right now, which is exactly what
##    `vault_preview_block` returns: both compute it from the bytes they just
##    read, in the same way. So the write preview is called for its `hash` and
##    its `action`, and `action == "inserted"` — no block for this entity in
##    that note — is the precondition check, at no extra read. The body handed
##    to it is irrelevant to both, since a block is found by its entity key.
## 2. **The span shown is located by this file**, from §23's public
##    `<!-- CARTALITH:BEGIN entity="…" -->` / `<!-- CARTALITH:END -->` markers.
##    Display only: the removal itself is the engine's own `block::remove`
##    under the hash guard, so a mis-slice here can show the wrong text and
##    cannot take out the wrong bytes. When the markers are not found the whole
##    note is shown and the dialog says that is what happened, rather than
##    presenting an empty pane as though there were nothing to remove.
##
## No "confirm always" checkbox, deliberately: `vault_set_write_pref` takes
## three names and removal is not one of them, and a fourth flag is an engine
## change. Given the act, always asking is also the right default.
func _confirm_block_remove() -> void:
	var rel := _link_path()
	if rel == "":
		return
	var body := bridge.vault_block_body(_kind, _entity_id, _selected_keys())
	var p := bridge.vault_preview_block(rel, _kind, _entity_id, body)
	if not bool(p.get("ok", false)):
		app.set_status("hint", "Preview: %s" % String(p.get("error", "")), "accent")
		return
	if String(p.get("action", "")) == "inserted":
		app.set_status("hint", "%s holds no Cartalith block for %s — there is nothing to remove." % [rel, _entity_label], "text_ghost")
		return
	var text := bridge.vault_read_file(rel)
	var span := _block_span(text, String(p.get("entity_key", "")))
	_preview_dialog("Remove the Cartalith block",
		span if span != "" else text,
		("These are the bytes that will be removed. Everything else in %s is written back unchanged." % rel) if span != ""
			else ("Cartalith could not locate the block's markers in %s to show them on their own, so the whole note is above. The removal itself is the engine's, and it takes out only the delimited block for %s." % [rel, _entity_label]),
		func():
			var r := bridge.vault_remove_block(rel, _kind, _entity_id, String(p.get("hash", "")))
			if not bool(r.get("ok", false)):
				app.set_status("hint", "Removal refused: %s" % String(r.get("error", "")), "accent")
			elif bool(r.get("removed", false)):
				app.set_status("hint", "Cartalith block removed from %s." % rel, "text_ghost")
			else:
				app.set_status("hint", "%s held no Cartalith block for %s." % [rel, _entity_label], "text_ghost")
			_rebuild(),
		"", "Remove from Markdown")


## The block's own bytes, for the preview above. `""` when the markers are not
## where this file expects them — which the caller reports rather than papers
## over. Character offsets throughout, which is self-consistent: `find` and
## `substr` are both in characters, so a note with non-ASCII prose above the
## block still slices correctly even though the engine works in bytes.
func _block_span(text: String, entity_key: String) -> String:
	if entity_key == "":
		return ""
	var begin := text.find("<!-- CARTALITH:BEGIN entity=\"%s\"" % entity_key)
	if begin < 0:
		return ""
	var end_marker := "<!-- CARTALITH:END -->"
	var end := text.find(end_marker, begin)
	if end < 0:
		return ""
	return text.substr(begin, end + end_marker.length() - begin)


## §23 rule 5 and §16 step 4-5, in one place so no write path can skip them.
##
## `pref_key` is the "confirm always" flag this dialog answers to — one of
## `vault_write_prefs()`' three names, `"section"`, `"block"` or
## `"field_fill"`. Passing one is what puts the checkbox on the dialog; passing
## `""` means this act always asks.
##
## ## The preference suppresses the dialog. It does not suppress the guard.
##
## Every caller computes its `vault_preview_*` **before** reaching here and
## closes over the `hash` that came back. That ordering is not tidiness: the
## hash is the entire write guard, so the preview is not the dialog's to skip.
## When the preference is set this function calls `on_confirm` straight
## through — the same closure, carrying the same hash, computed from the file
## as it was moments ago — and the engine still compares that hash against the
## file it is about to write. A note edited in the user's own editor in between
## refuses with "the file changed", whether or not anybody was asked.
##
## Written the other way round — the preview moved inside the `if` — a
## preference would have turned a safety mechanism into a rubber stamp, which
## is what `vault_write_prefs`' own doc comment warns a caller not to do.
##
## ## Two of the three flags sit against a line in the spec
##
## §24 asks the user to confirm a new block's *insertion location*, and §23's
## own header calls field fill *"offered and explicitly confirmed, never
## silent"*. Both are honoured here anyway, because the owner asked for the
## option by name and because what makes each safe is engine-side and is not a
## preference: the `expect_hash` comparison, and `FieldFill::OnlyIfEmpty`
## refusing an occupied field whether or not anybody is watching.
## `MARKDOWN_VAULT_SCOPE.md` milestone 6 records the same tension for the
## engine half; it is written down here rather than left as a silent
## contradiction in the UI half.
func _preview_dialog(dialog_title: String, preview: String, note: String,
		on_confirm: Callable, pref_key: String = "",
		ok_text: String = "Write to Markdown") -> void:
	if pref_key != "" and bool(bridge.vault_write_prefs().get(pref_key, false)):
		on_confirm.call()
		return
	var dlg := ConfirmationDialog.new()
	dlg.title = dialog_title
	dlg.size = Vector2i(620, 620)
	dlg.get_ok_button().text = ok_text
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
	var always: CheckBox = null
	if pref_key != "":
		## The toggle carries no callback that does anything: its value is read
		## once, on confirm, and never while the dialog is open. Ticking the box
		## and then pressing Cancel must leave the preference exactly as it was
		## — a user who backs out of *this* write has not agreed to stop being
		## asked about the next one.
		always = DccWidgets.toggle(col, "Don't ask again", false, func(_v: bool): pass,
			"Stops the preview appearing for this kind of write. The check that refuses a note edited since the preview is not a preference and stays on — Cartalith will still show you this dialog again if you turn the option back off under Write confirmations.")
	dlg.confirmed.connect(func():
		if always != null and always.button_pressed:
			bridge.vault_set_write_pref(pref_key, true)
			_save_prefs()
		on_confirm.call()
		dlg.queue_free())
	dlg.canceled.connect(dlg.queue_free)
	app.add_child(dlg)
	## The dialog is built outside `_rebuild()`, so the window's own phone pass
	## has never seen it — the same reason `app.gd`'s credits dialog fits itself
	## after `add_child`. Without this the checkbox row lands under §13's 44 dp
	## floor on a handset.
	##
	## It reaches the dialog's *content* and not its button bar: `get_children()`
	## skips internal children, and Write/Cancel are `AcceptDialog`'s own. That
	## bar is `DccWidgets._floor_dialog_bar`'s job, which runs from
	## `phone_window()` for this window and has never run for these preview
	## dialogs — unchanged by this pass, and not verified on a handset here.
	if _phone:
		app.phone_fit(dlg, 1.0)
	dlg.popup_centered()


# -- "Confirm always", and where it is turned back off ----------------------

const PREF_LABELS := {
	"section": "Insert updated section",
	"block": "Write Cartalith block",
	"field_fill": "Fill the note's own fields",
}

## Device state, and the one place a preference can be switched back on.
##
## A "don't ask again" that can only ever be set is a trap: the checkbox is on
## a dialog the preference itself has just stopped appearing. So the three
## flags are listed wherever this window is open, folded, in both the scoped
## and the overview modes.
##
## Three flags rather than one because replacing a section, regenerating the
## machine-owned block and writing into the author's own template lines are
## three different risks — a person may well never want to be asked about the
## middle one and always want to be asked about the last.
func _build_write_prefs() -> void:
	var prefs := bridge.vault_write_prefs()
	## Empty means an engine without the preference surface at all. Drawing
	## three toggles that silently do nothing would be worse than drawing none.
	if prefs.is_empty():
		return
	## Open when any confirmation is switched off, folded when all three are at
	## their safe default. A disarmed prompt is a thing the user should be able
	## to see without going looking; three unticked boxes are not.
	var any_off := bool(prefs.get("section", false)) or bool(prefs.get("block", false)) \
		or bool(prefs.get("field_fill", false))
	var sec := DccWidgets.group(_body, "Write confirmations", any_off)
	DccWidgets.note(sec, "Ticked means Cartalith stops showing the preview before that write. It does not stop checking: a note edited since Cartalith last read it still refuses, asked or not. These stay on this device — one person's \"stop asking me\" does not travel with a project.")
	for key in ["section", "block", "field_fill"]:
		DccWidgets.toggle(sec, String(PREF_LABELS[key]), bool(prefs.get(key, false)),
			func(v: bool):
				bridge.vault_set_write_pref(key, v)
				_save_prefs(),
			"Off is the default and the safe direction: every write of this kind is previewed and confirmed.")


# -- The preferences on disk ------------------------------------------------

## **Moved to `vault_store.gd` on 2026-08-26** (`PARITY_AUDIT.md` §23). The
## paragraph that used to sit here said this window wrote the sidecar, that
## `VaultStore` should, and that moving it was "a rename and two call sites".
## It was. `VaultStore.PREFS_PATH` carries the reasoning for the separate file;
## `VaultStore.load_prefs_into()` carries the verbatim-text rule.
##
## The window keeps these two names because it is the surface that toggles a
## preference, and `_prefs_loaded` because `setup()` must have the values
## before `_build_write_prefs()` draws three checkboxes from them. It no longer
## touches the disk itself, which is now true of every piece of vault state.
##
## `VaultStore.load_into()` also loads them, unbranched, so preferences are
## restored on a session where this window is never built at all.
func _load_prefs_once() -> void:
	if _prefs_loaded:
		return
	_prefs_loaded = true
	VaultStore.load_prefs_into(bridge)


func _save_prefs() -> void:
	VaultStore.save_prefs_from(bridge)


# -- Overview ---------------------------------------------------------------

func _build_overview() -> void:
	_build_index()
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


# -- The backlink index (`GUI_GAP_REGISTER.md` VA-01) ------------------------

## The index panel, and the answer to the register's own question about it.
##
## The register frames backlinks as a choice between an on-demand index that
## stalls a large vault and a persistent one that goes stale behind a folder
## the user edits elsewhere. It is a false pair: a `stat` is not a read, so
## Refresh walks the listing, compares each note's `(modified, len)` against
## what the index holds, and opens **only the files that moved**. Ten edits in
## Obsidian cost ten reads.
##
## Everything on this panel is a number the engine measured on this vault --
## no estimates, and no progress bar for a pass that is over before one could
## draw.
func _build_index() -> void:
	var info := bridge.vault_info()
	if not bool(info.get("bound", false)):
		return
	var sec := DccWidgets.section(_body, "Index")
	var st := bridge.vault_backlink_stats()
	if not bool(st.get("built", false)):
		DccWidgets.note(sec,
			"Not built. Building it reads every note in this vault once and keeps, per note, "
			+ "its size and modified time, the links it points at, and a 64-bit word "
			+ "fingerprint — never the prose. After that, a refresh only re-opens the files "
			+ "that changed.")
	else:
		DccWidgets.note(sec, "%d notes · %d links · %d Cartalith blocks · %s" % [
			int(st.get("notes", 0)), int(st.get("links", 0)), int(st.get("entities", 0)),
			String.humanize_size(int(st.get("bytes", 0)))])
		var broken := int(st.get("broken", 0))
		var orphans := int(st.get("orphans", 0))
		DccWidgets.note(sec, "%d links point at a note that does not exist · %d notes nothing links to"
			% [broken, orphans])
	if _index_feedback != "":
		DccWidgets.note(sec, _index_feedback)
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 4)
	var refresh := DccWidgets.action(row, "Refresh index", _refresh_index)
	refresh.tooltip_text = "Stats every note and re-reads only the ones whose size or modified time changed. Safe to press whenever; on an untouched vault it opens nothing at all."
	var rebuild := DccWidgets.action(row, "Rebuild", func():
		bridge.vault_rebuild_backlinks()
		_refresh_index())
	rebuild.tooltip_text = "Throws the index away and reads every note again. Only needed if the index was written by an older build that parsed links differently."
	_body.add_child(row)
	if bool(st.get("built", false)):
		_build_index_report()

func _refresh_index() -> void:
	var r := bridge.vault_refresh_backlinks(2000)
	if bool(r.get("ok", false)):
		VaultStore.save_index_from(bridge)
		_index_feedback = "Index: %d notes seen, %d re-read, %d dropped%s." % [
			int(r.get("seen", 0)), int(r.get("reread", 0)), int(r.get("dropped", 0)),
			"" if int(r.get("unreadable", 0)) == 0
				else ", %d unreadable" % int(r.get("unreadable", 0))]
	else:
		_index_feedback = "Index: %s" % String(r.get("error", "no vault connected"))
	_rebuild()

## `Data ▸ Missing & orphan notes report…`, which VA-01 has had disabled
## waiting for exactly this index. One index answers both questions, so there
## is one panel and not two walks.
func _build_index_report() -> void:
	var rep := bridge.vault_backlink_report(40)
	var broken: Array = rep.get("broken", [])
	var orphans: PackedStringArray = rep.get("orphans", PackedStringArray())
	if broken.is_empty() and orphans.is_empty():
		return
	var g := DccWidgets.group(_body, "Missing & orphan notes", false)
	if not broken.is_empty():
		DccWidgets.note(g, "Links that point at no note:")
		for b in broken:
			var d: Dictionary = b
			DccWidgets.note(g, "    %s → %s" % [String(d.get("source", "")), String(d.get("target", ""))])
	if orphans.size() > 0:
		DccWidgets.note(g, "Notes nothing links to:")
		for o in orphans:
			DccWidgets.note(g, "    %s" % o)
	DccWidgets.note(g,
		"Read-only, deliberately: Cartalith will not create a note to satisfy a broken link "
		+ "or delete an orphan. Both are the author's to decide, and the vault's boundary is "
		+ "that Cartalith never rewrites a note's body.")


func _build_footer() -> void:
	DccWidgets.note(_body,
		"Not built here, each for a stated reason: the map snapshot (§21) — it needs a crop of "
		+ "the live renderer at three radii, held as its own milestone rather than shipped as a "
		+ "broken image link; two-way sync and an Obsidian plugin — an explicit V1 non-goal and a "
		+ "deferred wish. Compare-with-source (§14) is built now — the diff sits beside a stale "
		+ "link's Reload source button. Links themselves ride inside a saved project: File ▸ Save "
		+ "writes them into the project archive's own vault.json, beside the settlements and "
		+ "factions the same archive already carries, and opening that project restores them "
		+ "working. A copy is also kept beside your Cartalith profile, as the fallback for links "
		+ "made before anything has been saved.")
