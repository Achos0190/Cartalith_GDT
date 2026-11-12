extends AcceptDialog
class_name VaultWindow

## MS-1 S4.5 (`window_represent.gd`, `S4_INVENTORY.md` section 18.8): the contract's shared helpers.
const WindowRepresent := preload("res://shell/window_represent.gd")

## The Markdown Vault panel — `MARKDOWN_VAULT_INTEGRATION.md` §28 and §29,
## `MARKDOWN_VAULT_SCOPE.md` milestone 1.
##
## §28 asks for the vault to "appear in entity information panels rather than
## as an isolated utility", and it does: `place_editor_window.gd`'s KNOWLEDGE
## section and the Civilization dock's province and continent rows are the
## entry points, and each of them opens *this* window already scoped to that
## entity. What lives here is everything §28's sketch cannot fit in a dock
## column — the file browser, the reader/working copy (§29), the preview, and
## the five write actions (see the section below).
##
## One window, six entity kinds (`cartalith-vault`'s `EntityKind`), because
## §11's whole point is a generic `KnowledgeLink`: `open_for("settlement", tid,
## name)`, `open_for("province", id, name)`, `open_for("continent", rank,
## name)`, and the same for `faction`, `culture` and `landmark`.
## Opened with no entity it shows the whole link store instead.
##
## ## One layout for browsing and for an entity (owner Ruling BE, 2026-09-27)
##
## `open_browse()` and `open_for(kind, id, label)` used to draw two different
## windows -- a tree with a raw `TextEdit`, and a long form (Vault / Find /
## Attach / Knowledge / Map snapshot / Write confirmations). With a vault
## connected they now draw the same one (`_build_browse`): the tree and its
## search on the left, attached notes marked; on the right an **Attached to**
## block (every link on the picked note, with Show on map / Open place editor
## / Detach, plus a frontmatter match offered as "by frontmatter, not
## attached"), the note itself -- a read-only rendering until **Open to edit**
## turns the same pane into `markdown_editor.gd` -- and a collapsed **More**
## holding the working copy, the map snapshot, the Cartalith block, field
## fill, the write confirmations, the index and Disconnect. `open_for`
## preselects the entity's first attached note, or shows its attach flow when
## it has none. With no vault connected, and for the overview, the old form
## is still what draws: there is no tree to put on the left.
##
## ## Every write here is explicit, and every write is previewed
##
## §17: *"Reading can be automatic/on-demand. Writing cannot."* Recounted
## 2026-09-27 (every `bridge.vault_write_*`/`vault_remove_block` call site in
## this file): there are exactly **five** buttons that write a Markdown
## file, each behind a preview whose hash is handed back to the write — so a
## note edited in the user's own editor between the preview and the
## confirmation refuses instead of overwriting. They are: **Save** (the
## Markdown editor, `_save_note`, `vault_write_file` -- its "preview" is the
## editor itself, and its hash is the one `vault_read_file_for_edit` returned
## when the note was opened), and, under **More**: **Insert updated
## section into source…** (`_confirm_section_write`, `vault_write_section`),
## **Preview & write Cartalith block…** (`_confirm_block_write`,
## `vault_write_block`), **Fill the note's own fields…**
## (`_confirm_field_fill`, `vault_write_field_fill`) and **Remove the
## Cartalith block…** (`_confirm_block_remove`, `vault_remove_block`).
## "Save local copy" in the reader is not one of them — it calls
## `vault_set_link_text`, which is memory-only and never touches the
## Markdown file (its own tooltip says so). The engine enforces the
## hash guard; this window's job is to never offer a write without having
## shown what it would do.
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
## `_build_feedback` appear only while the image is actually in the vault:
## `vault_export_fields` filters through `entity_values`, which supplies a Map
## value only for a snapshot that was written **and that nothing reported
## gone** (2026-09-05). So the block can never carry a link to an image that
## was never written, nor to one this device has looked for and not found.
##
## ## What is deliberately not here
##
## - **No `obsidian://` link, no wikilink generation, no two-way sync.** The
##   first two are Obsidian-specific (owner, 2026-08-18: nothing may require
##   Obsidian); the third is §33's explicit V1 non-goal.
## - **No POI.** Not a ported concept — the same absence
##   `place_editor_window.gd`'s own footer states.

## The shared Markdown editor script (`markdown_editor.gd`): `_build_editor_pane` instantiates it
## and `_build_browse_preview` uses its `style_rendered`.
const MdEditor := preload("res://shell/markdown_editor.gd")
## The side panel (VE-5): Links, Outline, Properties, Tags. Held without a `class_name`, as the editor is.
const SidePanel := preload("res://shell/vault_side_panel.gd")
## The private badge's cache (Ruling DP, VE-9): which notes are private, asked of the crate's one rule.
const Privacy := preload("res://shell/vault_privacy.gd")
const SafConnect := preload("res://shell/vault_saf_connect.gd")
## The window width from which the side panel sits to the RIGHT of the editor (px of the window, which
## are dp here). Below it the panel sits BELOW the editor, the "end of the note" form of the proposal's
## s.4. A labelled judgement: the editor wants 760 for Split, the tree column takes 220-300, so a 280
## panel beside a Split-capable editor needs about 1 300; 1 100 keeps Write comfortable (about 540) and
## gives up Split only on the narrower side of the line.
const SIDE_RIGHT_MIN_W := 1100

var app                       ## `DccApp`
## The `EngineBridge` given to `setup()`; every `vault_*` call in this window goes through it.
var bridge: EngineBridge
## Which notes are private, held so every list and link can say so (`vault_privacy.gd`). Built on first use,
## emptied when the window opens on a vault (`_forget_privacy`), and one note's answer dropped when it is saved.
var _privacy: RefCounted

## The entity this window is scoped to. Empty `kind` means the overview.
var _kind := ""
## Id of the scope entity within `_kind`; set by `open_for`, zeroed by `open_browse`.
var _entity_id := 0
## Display label of the scope entity; set by `open_for`, emptied by `open_browse`.
var _entity_label := ""

## The link currently open in the reader (§29). Empty means none.
var _reader_link := ""
## The reader's `TextEdit` (made in `_build_reader`); nulled by `_clear` so no stale node outlives
## a rebuild.
var _reader_edit: TextEdit

## The New-note-from-a-template picker's current template (VA-02).
var _pick_template := ""

## The picked note (the tree's selection, and the note pane's subject) and
## the section an attach copies ("" = the whole document).
var _pick_file := ""
## The section heading an attach would copy, "" meaning the whole document (see the comment above);
## reset by `_reset_note_state` and `_select_file`.
var _pick_heading := ""

## The Markdown editor's file state (`_open_editor`, `_save_note`).
## `_browse_path` is the file the editor is open on and `""` means closed; it
## is deliberately not the same variable as `_pick_file`, so picking a
## *different* note closes a stale editor rather than showing the old one's
## text under the new one's name. `_browse_hash` is what Save hands back to
## `vault_write_file`'s guard. `_browse_edit` is the editor's own `TextEdit`.
##
## `_browse_text` is held here rather than read back from the editor on every
## `_rebuild()`, because there is no working copy for a whole file to
## round-trip through the way `_build_reader` does via `vault_set_link_text`.
var _browse_path := ""
## Content hash of the open file, what Save hands back to `vault_write_file`'s guard (see the
## comment above); cleared by `_reset_note_state` and `_close_editor`.
var _browse_hash := ""
## The editor buffer held across rebuilds (see the comment above); `_is_dirty` compares it with
## `_browse_saved`.
var _browse_text := ""
## The Markdown editor's inner `TextEdit` (`ed.text_edit` in `_build_editor_pane`); nulled by
## `_clear`.
var _browse_edit: TextEdit

## Ruling BE's editor state. `_browse_saved` is the text the file held when it
## was opened or last saved -- what "unsaved" is measured against, so a
## rebuild (which recreates the editor from `_browse_text`) does not forget
## that the buffer is dirty. `_editor_mode` is Write or Preview, kept across a
## rebuild for the same reason.
var _browse_saved := ""
## The Markdown editor control built in `_build_editor_pane`; nulled by `_clear`.
var _editor: Control
## The side panel beside / below the editor and the header button that shows or hides it; nulled with the
## editor. `_side_user` is the person's choice (`-1` none yet: open on a wide window, closed otherwise,
## `0` closed, `1` open), kept across rebuilds. `_goto_pending` is a 0-based line the next editor built
## should scroll to (a backlink opened at its link), `-1` for none.
var _side_panel: Control
var _side_toggle: Button
var _side_user := -1
var _goto_pending := -1
## "write" or "preview" (see the comment above); reset to "write" by `_reset_note_state` and
## `_close_editor`.
var _editor_mode := "write"

## --- VE-4: autosave, the conflict banner and the session undo (VAULT_EDITOR_PROPOSAL.md s.3.3) ---
## Autosave fires this long after the last keystroke. A labelled judgement: the proposal and Ruling DY
## (VQ1) say "about a second"; short enough that "saved" is on screen well inside the 2 s bar, long
## enough that a pause between words does not write a half-sentence.
const AUTOSAVE_SEC := 1.0
## The session undo keeps checkpoints, not keystrokes: the first-opened text, then one per save or
## rebuild. Labelled judgements: 100 steps is more Ctrl+Z than anyone walks, and 4 million characters
## (about 8 MB) is a ceiling on one note's history. Past either, the OLDEST checkpoint after the
## first-opened one is dropped, never the first-opened one itself (that is what Revert returns to).
const HIST_MAX_CHECKPOINTS := 100
const HIST_MAX_CHARS := 4000000
## One-shot timer for the autosave, created in `setup`; restarted by every edit.
var _autosave_timer: Timer
## Per note (vault-relative path): the checkpoints, `[0]` being the note as first opened since
## Cartalith started. Memory only: never written to disk, gone on quit (Ruling DY, VQ1). Cleared on
## connect and disconnect, because a path means a different note in another vault.
var _hist: Dictionary = {}
## Why the last save did not land, `{kind, error}`, or `{}` when nothing blocks. Held here, not in
## the editor, so a rebuild (dragging across the fold) shows the same banner again.
var _save_block: Dictionary = {}
## The open file used CRLF line endings. A `TextEdit` holds LF, so the text is converted back on write:
## a Windows note must not change every line of itself the first time it is autosaved.
var _eol_crlf := false
## The time of the last successful save, shown in the footer; `""` before the first.
var _saved_stamp := ""
## How many times this window has called `vault_write_file` for the editor (a diagnostic a probe reads
## to prove that an unchanged note, or a note under a standing conflict, is not written to).
var _writes_attempted := 0
## The note the LIVE editor (`_editor` / `_browse_edit`) was built for. `_open_editor` assigns the new
## `_browse_path` and text BEFORE `_rebuild` frees the old editor, so for that moment the old editor is
## still alive under the new note's name. Anything that reads the editor's text (the sync in `_clear`,
## `_live_text`, `_write_now`) must check this equals `_browse_path`, or one note's text is
## written to or checkpointed under another note's name. A probe caught exactly that on this
## milestone's first run (Qconf1.md came out holding Qalpha.md's text).
var _editor_path := ""

## The notes a followed link was followed FROM, most recent last (VE-1): Back pops one. Cleared when
## a note is picked from the tree, searched for or opened from the map, because those start a new
## trail rather than continuing one.
var _link_back: Array = []
## Lower-cased entity name -> `{kind, id}` for the kinds `_entity_list` can list (settlement, province,
## continent, faction, culture and, since Ruling EI, landmark); built on the first link classed after a
## rebuild (`_clear` empties it), so a landmark name given while the window is open reads from the next open.
var _ent_names: Dictionary = {}

## The "Attach to…" picker (`_build_attach_picker`): open or not, the kind it
## lists, and its name filter.
var _attach_open := false
## Entity kind the Attach picker is listing, default "settlement"; set by the picker's segment
## buttons in `_build_attach_picker`.
var _attach_kind := "settlement"
## The Attach picker's name filter; cleared by `_reset_note_state` and written back into the field
## by `_build_attach_picker`.
var _attach_query := ""
## The Attach picker's result list, made in `_build_attach_picker` and nulled by `_clear`.
var _attach_list: VBoxContainer

## Whether the collapsed **More** section is open. Held here because every
## state change rebuilds the window, and a section that snapped shut on every
## press inside it would be unusable.
var _more_open := false

## The entity the More section's writes act on: the active link's entity when
## a note with links is picked, otherwise the window's own scope. The five
## guarded writes and the snapshot read these, never `_kind` directly, because
## a browsed note is not scoped to anything and may be attached to several.
var _lk := ""
## Entity id for the More section's writes (see the comment above); `_rebuild` seeds it from
## `_entity_id` and `_build_more` overrides it from the active link.
var _lid := 0
## Entity label for the More section's writes; seeded from `_entity_label` in `_rebuild`,
## overridden by the active link in `_build_more`.
var _llabel := ""

## The tree+preview split (owner request, 2026-09-21, "a Markdown Vault
## browser"): which pane is showing on a phone, where the split folds to a
## segmented switcher rather than sitting side by side — the now-retired (FH-2) `culture_profiles_
## window.gd::_build_phone_switcher()`'s pattern, adapted to this file's own
## idiom of rebuilding `_body` from scratch on every state change rather than
## keeping persistent panes to show/hide.
var _browse_phone_pane := "tree"

## The browse split's divider, as `HSplitContainer.split_offsets[0]` (see
## `_build_browse()`): 300 opens the tree column at the mockup's 300 px.
var _browse_split := 300

## True only for the standalone entry point (`open_browse()`): no entity, no
## Attach, just Search and the file browser/editor — as opposed to
## `open_overview()`'s `_kind == ""`, which lists every link in the store
## instead. Distinct flag rather than overloading `_kind == ""` because both
## are valid "no entity" states that draw different bodies.
var _browse_only := false

## The vault search. `_search_result` is the last `vault_search` answer held
## verbatim — `indexed`/`scanned`/`truncated` included, because the panel has
## to report those and not only `hits`.
var _search_query := ""
## Last `vault_search` answer verbatim (see the comment above); set in `_run_search`, `{}` when the
## query is empty, and read by `_fill_search_results`.
var _search_result := {}
## Container the search results are drawn into, made in `_build_search` and nulled by `_clear`.
var _search_box: VBoxContainer
## Relative path of the search hit currently expanded; cleared in `_run_search`, and
## `_fill_search_results` shows the caret on it.
var _search_open_rel := ""

## The standalone browser's own tree container (`_build_browse`), refreshed in
## place by `_refresh_browse_tree()` on every keystroke in
## `_build_browse_search()`'s field — distinct from `_search_box` above (the
## scoped Attach flow's content-search results) so a live keystroke in one
## mode can never free a node the other mode owns. `null` when no tree pane
## is on screen (the phone's preview pane, or before the first `_rebuild()`).
var _browse_tree_col: Control

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

## Destination folder text for the map snapshot; edited in `_build_snapshots` and, per the comment
## above, deliberately never persisted. An empty value falls back to `DEFAULT_SNAPSHOT_DIR`.
var _snapshot_dir := ""

## The last index Refresh/Rebuild result, shown on the Index section until the
## next one. A one-shot line, not a persistent state: the numbers above it are
## the state.
var _index_feedback := ""

## The overview's settlement-name filter for `_build_pick_entity()` (menus.gd
## 2026-09-21: the two menu rows that used to promise "Create a note from a
## template" and the index landed on this same unscoped panel with no way to
## actually reach either without an entity already linked. This is that way.)
var _pick_query := ""

## The `VBoxContainer` the window's contents are rebuilt into on every state change; created in
## `setup`.
var _body: VBoxContainer
## The `ScrollContainer` holding `_body`, taken from `setup`; `_rebuild` switches its vertical
## scroll mode.
var _scroll: ScrollContainer
## Whether the window is the phone presentation, set from `DccWidgets.phone_window` in `setup`.
var _phone := false
## The phone header label made by `DccWidgets.phone_head` in `setup`; `_rebuild` updates it, and it
## is null off-phone.
var _phone_title: Label

## Emitted after anything that changes the link store, so the host can
## persist it. The window never touches the disk itself.
signal store_changed

## PC-5. The window took the editor back from the Context panel's Note tab (it is being opened or
## rebuilt): the tab's editor is gone, the note's text is still held here, and the tab should re-dock.
signal dock_released
## PC-5. A write landed (`_after_save`): the Note tab retries a selection switch it held while the
## previous note could not be saved.
signal dock_saved


## Wires the window to its host (`app`) and the engine bridge (`b`), loads the
## write-confirmation preferences once, and builds the empty frame (title, size,
## scroll body); `_rebuild()` fills the body on every open. Call it once, before
## the window is shown. It must never touch the disk itself: persistence goes out
## through `store_changed`.
func setup(a, b: EngineBridge) -> void:
	app = a
	bridge = b
	_load_prefs_once()
	title = "Markdown vault"
	size = Vector2i(560, 720)
	min_size = Vector2i(380, 460)
	## No `max_size` (UX review, 2026-09-23). It was 760x900 -- a cap written
	## against `wrap_controls` running the window off the screen, which
	## `phone_window()` below already turns off at the cause. What the cap did
	## instead was stop a user drag at 760 px, while the owner-approved Vault
	## Browser mockup (2026-09-21) is drawn at 1280x800.
	## "Close", not "OK": nothing here is committed by pressing it -- every write
	## has its own button and preview. The four other plain windows that only
	## dismiss (`city_viewer_window.gd`, `world_data_window.gd`, ...) say Close.
	ok_button_text = "Close"
	_phone = DccWidgets.phone_window(self, a)
	## PW-3 (`POPOUT_WINDOWS_SCOPE.md`, Ruling DK Q2): the "Pop out / Dock" button in the button row. Desktop
	## form only. A move hides and re-shows THIS node under `PopoutHost.META_MOVING`; the unsaved-edit guard
	## (`_on_visibility_changed`) defers to the end of the frame and finds the window visible again, so a
	## move with unsaved edits neither asks nor loses them. Never calls `open_*`, so the note, its
	## unsaved text and the scroll are the ones the user left.
	PopoutHost.add_toggle_button(self)

	var root := VBoxContainer.new()
	root.add_theme_constant_override("separation", 0)
	add_child(root)
	var scroll := ScrollContainer.new()
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	_scroll = scroll
	## VE-8: with the soft keyboard up the window is only the part above it, so a field the person taps lower in the
	## page (the sheet's properties, a rename box) must be scrolled into view rather than left under the keys.
	scroll.follow_focus = true
	root.add_child(scroll)
	if _phone:
		_phone_title = DccWidgets.phone_head(root, "Markdown vault", "linked notes")
	var pad := MarginContainer.new()
	for side in ["left", "top", "right", "bottom"]:
		pad.add_theme_constant_override("margin_" + side, 12)
	pad.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	## Only matters while the scroll is not scrolling vertically (browse's
	## split): EXPAND is what lets a ScrollContainer hand its child its height.
	pad.size_flags_vertical = Control.SIZE_EXPAND_FILL
	scroll.add_child(pad)
	_body = VBoxContainer.new()
	_body.add_theme_constant_override("separation", 4)
	_body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_body.size_flags_vertical = Control.SIZE_EXPAND_FILL
	pad.add_child(_body)
	visibility_changed.connect(_on_visibility_changed)
	size_changed.connect(_on_size_changed)
	_autosave_timer = Timer.new()
	_autosave_timer.name = "VaultAutosave"
	_autosave_timer.one_shot = true
	_autosave_timer.wait_time = AUTOSAVE_SEC
	_autosave_timer.timeout.connect(_write_now)
	add_child(_autosave_timer)


## Opens scoped to one entity. `kind` is `"settlement"`, `"province"` or
## `"continent"`; `entity_id` is that kind's own id (a settlement's **tid**,
## not its index — the index shifts when another settlement is deleted and a
## link must survive that).
##
## Ruling BE: with a vault connected this is the browser layout with the
## entity's first attached note preselected -- or, when it has none, its attach
## flow (`_build_attach_flow_empty`).
func open_for(kind: String, entity_id: int, label: String) -> void:
	if _is_dirty():
		_guard_dirty(func(): open_for(kind, entity_id, label))
		return
	privacy().clear()
	_kind = kind
	_entity_id = entity_id
	_entity_label = label
	_browse_only = false
	_reset_note_state()
	_selected_fields = {}
	if kind != "":
		var links := bridge.vault_links_for(kind, entity_id)
		if not links.is_empty():
			_pick_file = String((links[0] as Dictionary).get("path", ""))
			_reader_link = String((links[0] as Dictionary).get("link_id", ""))
		## Preselected, so the phone opens on the note rather than the files.
		_browse_phone_pane = "preview"
	_rebuild()
	if not DccWidgets.phone_present(self, app):
		if kind != "" and bool(bridge.vault_info().get("bound", false)):
			_size_as_browser()
		## PW-3: re-decides native versus docked for this open (see `setup`'s note).
		PopoutHost.prepare(self)
		popup_centered()


## The browser folds to one pane (FILES / NOTE, the phone's switcher) when
## the window is narrower than the two panes' minimum. Measured by
## `_vaultgone_probe`'s width sweep: side by side, the browser's body needs
## 499 px, and the window may be dragged down to its 380 px `min_size`. 640 is
## that 499 plus the window's padding and the split's grabber, rounded up.
const TWO_PANE_MIN_W := 640


## True when the window should use the single-column (fold) layout: always on a
## phone, otherwise whenever the window is narrower than `TWO_PANE_MIN_W`.
## Read-only; `_on_size_changed()` rebuilds when this flips.
func _one_pane() -> bool:
	return _phone or size.x < TWO_PANE_MIN_W


## A drag across the fold width rebuilds, once per crossing.
func _on_size_changed() -> void:
	if not visible or _phone or not _unified():
		return
	var one := size.x < TWO_PANE_MIN_W
	if one != bool(get_meta("one_pane", one)):
		set_meta("one_pane", one)
		_rebuild.call_deferred()
	set_meta("one_pane", one)


## The editor, picker and link state every entry point starts from.
func _reset_note_state() -> void:
	_link_back.clear()
	_reader_link = ""
	_pick_file = ""
	_pick_heading = ""
	_browse_path = ""
	_browse_text = ""
	_browse_hash = ""
	_browse_saved = ""
	_editor_mode = "write"
	_save_block = {}
	if _autosave_timer != null:
		_autosave_timer.stop()
	_attach_open = false
	_attach_query = ""
	_more_open = false


## The approved mockup's frame (1280x800), never larger than 90% of the screen
## it opens on, and never smaller than a size the user already dragged it to
## this session.
func _size_as_browser() -> void:
	var room: Vector2 = app.get_viewport_rect().size * 0.9
	size = Vector2i(maxi(size.x, mini(1280, int(room.x))), maxi(size.y, mini(800, int(room.y))))


## Opens the overview: the vault connection and every link in the store.
func open_overview() -> void:
	open_for("", 0, "")


## The map's Find notes scope opens a note here (VE-9): the browse window, then the note in Write (Ruling DY, VQ4).
## A dirty note asks first, as every other way into the window does.
func open_note(rel: String) -> void:
	open_browse()
	if _browse_only:
		_enter_note(rel)


## The standalone browse-and-edit entry point (owner request, 2026-09-21):
## the same window, no entity/kind scope and no Attach — Search plus the file
## browser and its raw-text preview+edit panel (`_build_browse`), so a note
## can be read and written without first attaching it to anything.
##
## `open_for("", 0, "")` already exists as "the overview" (every link in the
## store), which is a different zero-entity view and not this one — hence the
## separate `_browse_only` flag rather than overloading `_kind == ""` a
## second way. Reusing this window rather than a new scene: `setup()`,
## `_clear()`/`_rebuild()`, the phone-fit passes and every `DccWidgets`/
## `DccTheme` helper this file already leans on would otherwise have to be
## duplicated into a second script for one extra body variant.
func open_browse() -> void:
	if _is_dirty():
		_guard_dirty(open_browse)
		return
	privacy().clear()
	_kind = ""
	_entity_id = 0
	_entity_label = ""
	_browse_only = true
	_reset_note_state()
	_browse_phone_pane = "tree"
	_selected_fields = {}
	## **Size BEFORE the build (2026-10-18).** `_rebuild()` chooses the layout from
	## `_one_pane()`, i.e. from `size.x` at that instant. This used to size the
	## window AFTER `_rebuild()`, so the first open on a desktop built the
	## folded one-pane layout against the window's default 560 px width, and
	## `_on_size_changed()` cannot repair it (it returns while the window is not
	## yet visible, and no further size change comes once it is shown) -- the
	## split with its `VaultTreeCol` only appeared after some later rebuild
	## (a note pick). Found by `_vaultlayout_probe.gd`'s "tree column exists
	## (desktop split)" check, which had been failing at HEAD.
	var as_window := not DccWidgets.phone_present(self, app)
	if as_window:
		_size_as_browser()
	_rebuild()
	if as_window:
		## PW-3: re-decides native versus docked for this open (see `setup`'s note).
		PopoutHost.prepare(self)
		popup_centered()


## Frees the body's children and nulls every cached reference into them (editors,
## search box, tree column) so a callback that outlives a rebuild finds `null`
## and bails instead of writing into a freed node. Must never be skipped before a
## rebuild.
func _clear() -> void:
	## VE-4: the live text is read back and checkpointed BEFORE the editor is freed, so a rebuild
	## (a drag across the fold width) keeps both the unsaved buffer and the undo history. The
	## `edited` mirror is deferred and may be one frame behind.
	if _browse_path != "" and _editor_path == _browse_path and _browse_edit != null and is_instance_valid(_browse_edit):
		_browse_text = _browse_edit.text
		_checkpoint_history()
	_editor_path = ""
	## PC-5: the window is taking the editor back (it is being opened, or rebuilt). The Note tab's editor is
	## freed here, after the checkpoint above, so there is never a docked editor and a window editor for one
	## note; the unsaved text stays in `_browse_text`, and the tab hears `dock_released` and re-docks.
	if _dock_node != null:
		_free_dock_node()
		dock_released.emit()
	for c in _body.get_children():
		_body.remove_child(c)
		c.queue_free()
	_reader_edit = null
	_browse_edit = null
	_editor = null
	_side_panel = null
	_side_toggle = null
	_ent_names = {}
	_attach_list = null
	## Nulled, not left dangling: `_fill_search_results()` can be reached from a
	## callback that outlives the rebuild that freed the box it was writing into.
	_search_box = null
	## Same reason: `_refresh_browse_tree()` (the tree-column search field's
	## live filter) is reached from a `LineEdit.text_changed` callback the
	## field's own owner keeps firing after a `_rebuild()` has freed this.
	_browse_tree_col = null
	## Same reason: `_scroll_to_pwn()` runs deferred, after a rebuild may
	## have freed the section it would scroll to.
	_pwn_section = null


## Rebuilds the whole body from the bridge's current state: the title, the
## connection block, then the unified browser (Ruling BE: a bound vault with a
## scope entity or browse-only), the unbound scoped form, or the overview. Safe
## to call after any state change; it is the window's only redraw path and frees
## the focused field, so never call it per keystroke (see `_build_search`).
func _rebuild() -> void:
	_clear()
	var scoped := _kind != ""
	var header := _entity_label if scoped else ("Browse a note" if _browse_only else "Markdown vault")
	title = "Markdown vault — %s" % header if scoped or _browse_only else header
	if _phone_title != null:
		_phone_title.text = header.to_upper()

	var info := bridge.vault_info()
	var bound := bool(info.get("bound", false))
	## The More section's writes act on the scope entity until a picked note's
	## link says otherwise (`_build_more`).
	_lk = _kind
	_lid = _entity_id
	_llabel = _entity_label
	## Ruling BE: browse and entity view are one layout whenever there is a
	## vault to browse. `_reader_link` naming a link on another file (a caller
	## that set it directly, or the overview's rows) moves the pick to that
	## file rather than being silently ignored.
	var unified := bound and (scoped or _browse_only)
	if unified and _reader_link != "" and not _editing():
		var lp := _link_rel(_reader_link)
		if lp != "" and lp != _pick_file:
			_pick_file = lp
	## The browser is a two-pane layout, not a form: the window scroll stops
	## scrolling so `_build_browse()`'s split can take the window's whole
	## remaining height, and each pane scrolls on its own (the mockup's
	## shape). On the phone the one-pane fold still scrolls as a column --
	## except while editing, where the editor has to fill the sheet and
	## scrolls its own text.
	var split := unified and (not _one_pane() or _editing())
	_scroll.vertical_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED if split \
		else ScrollContainer.SCROLL_MODE_AUTO
	_build_connection(info)
	if unified:
		_build_browse()
	elif scoped:
		## No vault on this device: no tree to browse, so the scoped form --
		## the links and their cached text stay readable (§27 "Unbound").
		_build_links()
		if _reader_link != "":
			_build_reader(_body)
			_build_feedback(_body)
	elif not _browse_only:
		## Search sits directly under the connection in the overview: the
		## owner's sentence starts with finding the note.
		if bound:
			_build_search()
		_build_overview()
	## The browser keeps these under More; the overview and the unbound form
	## still draw them at the foot.
	if not unified and not _browse_only:
		_build_write_prefs(_body)
		_build_footer()
	if _phone:
		app.phone_fit(self, 1.0)


# -- Connection (§7) --------------------------------------------------------

## Builds the connection block for `info` (the vault status dictionary): a one-line header when
## bound and in the browser or a kind scope, otherwise the full "Vault" section (connected, known but
## not on this device, or none) with its Connect action.
func _build_connection(info: Dictionary) -> void:
	var bound: bool = bool(info.get("bound", false))
	## The standalone browser, connected: the owner-approved 2026-09-21
	## mockup draws a one-line connection header (`vault/ · 42 notes …
	## read-only preview · open to edit`) in place of this section's full
	## Connect/Disconnect block. Unbound still falls through to the full
	## section below — there is nothing to browse yet, and that block is
	## where "Connect vault…" lives.
	if bound and (_browse_only or _kind != ""):
		_build_connection_header(info)
		return
	var sec := DccWidgets.section(_body, "Vault")
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
			_hist.clear()
			_img_cache.clear()
			_rename_last = {}
			store_changed.emit()
			_rebuild())
		dis.tooltip_text = "Drops this device's binding. The links themselves survive — that is the difference between disconnecting and detaching."


## The standalone browser's one-line connection header (owner-approved
## mockup, `design/vault-browser-2026-09-21/Cartalith Vault Browser.dc.html`
## title bar: `vault/ · 42 notes` beside a right-aligned `read-only preview ·
## open to edit`). "Change vault…" is not in the mockup's own drawing, which
## has no other window to reach `_browse_vault()` from — `open_vault_browse()`
## is this window's only menu entry point (`menus.gd`), so a bound vault with
## no way to swap it here would have none anywhere. Kept to a single
## `text_button` so the desktop row stays one line.
##
## **On phone, two lines, not one** — measured (`_vaultlayout_probe.gd`'s own
## phone screenshot): the desktop row's four items (name+count, "Change
## vault…", a spacer, then the full "read-only preview · open to edit"
## sentence) at phone-scaled font ran past the right edge of a 1080-physical-
## px viewport, the same overflow `MISTAKES.md`'s "read a layout that
## overflows the screen" row warns about. `HBoxContainer` never wraps or
## clips a sibling on its own — splitting the row in two, the same
## `VBoxContainer` fold the outline/excerpt columns use above, is what keeps
## every word on screen rather than shrinking type until it happens to fit.
func _build_connection_header(info: Dictionary) -> void:
	var name := String(info.get("display_name", ""))
	var count := bridge.vault_list_files().size()
	var head_text := "%s · %d note%s" % [
		name if name != "" else "vault", count, "" if count == 1 else "s"]
	## Opened from an entity (Ruling BE): say which, in the header's own
	## right-hand slot, so the preselected note is not a mystery.
	var hint := "read-only preview · open to edit" if _kind == "" \
		else "opened from %s · %s" % [_entity_label, _kind]
	if _one_pane():
		var col := VBoxContainer.new()
		col.add_theme_constant_override("separation", 4)
		_body.add_child(col)
		var row1 := HBoxContainer.new()
		row1.add_theme_constant_override("separation", 10)
		col.add_child(row1)
		row1.add_child(DccTheme.mono_label(head_text, "text_dim", DccTheme.FS_SMALL, 1))
		var change := DccWidgets.text_button(row1, "Change vault…", _browse_vault)
		change.tooltip_text = "Connect a different folder. Cartalith reads only the folder you choose, and never writes to it without an explicit action and a preview."
		col.add_child(DccTheme.mono_label(hint, "text_ghost", DccTheme.FS_MICRO, 1))
		return
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 10)
	row.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_body.add_child(row)
	row.add_child(DccTheme.mono_label(head_text, "text_dim", DccTheme.FS_SMALL, 1))
	var change := DccWidgets.text_button(row, "Change vault…", _browse_vault)
	change.tooltip_text = "Connect a different folder. Cartalith reads only the folder you choose, and never writes to it without an explicit action and a preview."
	row.add_child(DccTheme.spacer())
	row.add_child(DccTheme.mono_label(hint, "text_ghost", DccTheme.FS_MICRO, 1))


## Opens the folder picker and binds the chosen folder as the vault. A refused
## connect is reported on the status line with the engine's reason; success emits
## `store_changed` so the host persists. Rebuilds either way. Never writes to the
## chosen folder: connecting only reads.
func _browse_vault() -> void:
	## Android: the OS folder grant (Ruling AV), connected through the SAF handler. The in-app browser
	## cannot read another app's files there (scoped storage), so it is the fallback only where the OS
	## dialog does not exist.
	if SafConnect.should_use_picker():
		SafConnect.pick("Select markdown vault folder", func(uri: String):
			_hist.clear()
			_img_cache.clear()
			_rename_last = {}
			var r: Dictionary = SafConnect.connect_android(bridge, uri, "")
			if not bool(r.get("ok", false)):
				app.set_status("hint", "Vault: %s" % String(r.get("error", "could not connect")), "accent")
			else:
				if r.has("warning"):
					app.set_status("hint", String(r["warning"]), "accent")
				store_changed.emit()
			_rebuild(),
			func(msg: String): app.set_status("hint", "Vault: %s" % msg, "accent"))
		return
	DccBrowseDialog.choose_folder(PopoutHost.dialog_host(app, self), "Select markdown vault folder", "",
		"Cartalith reads .md files from this folder and writes only where you tell it to.",
		func(path: String):
			_hist.clear()
			_img_cache.clear()
			_rename_last = {}
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


## Runs the search for `_search_query` (names, plus note text once the content
## index exists) and refills only the results box. An empty query clears the
## result. Called by Enter or the Search button, never per keystroke (§31: do not
## walk the vault casually).
func _run_search() -> void:
	var q := _search_query.strip_edges()
	_search_open_rel = ""
	_search_result = {} if q == "" else bridge.vault_search(q, 0, 0)
	## The tree already narrows live on names (`_refresh_browse_tree`), so a
	## run search -- Enter in the tree's field, which is what looks INSIDE the
	## notes -- only refills its results box, and never frees the field.
	_fill_search_results()


## Redraws `_search_box` from `_search_result`: the hits plus the numbers
## (`indexed`, `scanned`, `truncated`) that say whether an empty answer means
## "absent" or "not looked at". Returns at once if a rebuild has freed the box.
## Must never present a capped or names-only search as complete.
func _fill_search_results() -> void:
	if _search_box == null or not is_instance_valid(_search_box):
		return
	for c in _search_box.get_children():
		_search_box.remove_child(c)
		c.queue_free()

	var in_browser := _unified()
	if _search_result.is_empty():
		## The browser's field filters names as it is typed; its box stays
		## empty until Enter asks for more, rather than repeating the hint.
		if not in_browser:
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
		## In the browser a hit opens the note in the pane beside it -- the
		## pane already shows everything the old inline readout did.
		var open := DccWidgets.action(_search_box, privacy().badged(rel, "%s %s" % [mark, rel]), func():
			if in_browser:
				_select_file(rel)
				return
			_search_open_rel = "" if _search_open_rel == rel else rel
			_fill_search_results())
		open.alignment = HORIZONTAL_ALIGNMENT_LEFT
		open.tooltip_text = "Opens this note beside the tree." if in_browser \
			else "Shows what this note holds — its frontmatter and its filled-in fields — without attaching anything."
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
	_fit_search_box()


## Whether the window is drawing Ruling BE's browser layout right now.
func _unified() -> bool:
	return (_kind != "" or _browse_only) and bool(bridge.vault_info().get("bound", false))


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
## copies the author's own template with Obsidian's placeholders filled
## (Ruling BF: `{{title}}` is the new note's name, `{{date}}`/`{{time}}` today
## in the vault's own formats) and the entity's name in the owner's
## `{{…Name}}`/`[Name]` tokens -- every `[If applicable]` and `[Optional]`
## prompt, and Templater's `<% %>`, survives for the author.
##
## Templates come from the vault, not from this program. There is no registry
## and no bundled content: the folder is the one Obsidian's Templates plugin is
## set to (`.obsidian/templates.json`), else Templater's; only a vault with
## neither falls back to "a `.md` with *template* in its path", which is how
## the owner's own `design/vault-templates/` names them. The line under the
## heading says which applied.
func _build_create(parent: Control, open: bool = false) -> void:
	var templates := bridge.vault_templates()
	if templates.is_empty():
		return
	var sec := DccWidgets.group(parent, "New note from a template", open)
	var src := bridge.vault_template_source()
	if bool(src.get("ok", false)):
		var src_label := DccTheme.label(String(src.get("describe", "")), "text_ghost", DccTheme.FS_SMALL)
		src_label.name = "TemplateSource"
		src_label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		src_label.custom_minimum_size.x = 160
		sec.add_child(src_label)
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
		"The notes in the template folder your Obsidian settings name -- or, when none is set, every .md whose path contains \"template\". Cartalith ships none of its own -- your templates are yours.")

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
		_browse_phone_pane = "preview"
		_rebuild())
	create.tooltip_text = "Copies the template as Obsidian would -- {{title}} becomes the new note's name, {{date}} and {{time}} today's -- with %s in its name placeholders, then links it to this entity. Refuses if that path already exists -- nothing is ever overwritten." % _entity_label


# -- The browser: one layout for browsing and for an entity (Ruling BE) -----

## The bound body for both `open_browse()` and `open_for()`: a folder tree over
## the vault's flat file list on the left (search at its head, attached notes
## marked), and on the right the note pane (`_build_note_pane`) -- or, while
## editing, the Markdown editor filling that pane.
##
## Before Ruling BE this was the standalone browser only, and `open_for()` drew
## a separate form below a file dropdown. What that form held and where each
## piece went (`MISTAKES.md`'s "delete a surface" inventory, 2026-09-27):
##
## | the form's item | now |
## |---|---|
## | File dropdown (`_build_file_picker`) | the tree |
## | "What does this note hold?" | the frontmatter and field chips, always shown |
## | "Preview & edit this note…" (`_build_note_editor`) | Open to edit -> the Markdown editor |
## | Section dropdown + "Attach to X" (`_build_attach`) | Attached to: "Attach to X", with its Section choice |
## | Find a note (`_build_search`) | the tree's field: typing filters names, Enter searches inside notes |
## | Knowledge rows: status, Open, Detach, Reload, Compare | Attached to (status, Detach, Reload, Compare); Open -> More ▸ Working copy |
## | What the notes say (`_build_entity_data`) | More |
## | New note from a template | the attach flow (no notes yet) or More |
## | Map snapshot, Cartalith feedback, Write confirmations | More |
## | Vault: Connect a different folder / Disconnect | the header's "Change vault…"; More ▸ Disconnect |
##
## `_pick_file` stays the one selection variable. No new bridge call: the tree
## is built client-side from `vault_list_files()`'s flat paths, split on `/`.
func _build_browse() -> void:
	var sec := DccWidgets.section(_body, "Browse a note" if _kind == "" else "Notes")
	var files := bridge.vault_list_files()
	if files.is_empty():
		DccWidgets.note(sec, "No .md files found in this vault folder.")
		return
	if _pick_file != "" and not Array(files).has(_pick_file):
		_pick_file = ""
	## Browsing opens on the first note, as it always has. An entity with no
	## note opens on its attach flow instead (`_build_note_pane`).
	if _pick_file == "" and _kind == "":
		_pick_file = files[0]

	if _one_pane():
		if _editing():
			## The editor fills the sheet: no switcher, no tree, and the outer
			## scroll is off (`_rebuild`), so EXPAND reaches it.
			sec.size_flags_vertical = Control.SIZE_EXPAND_FILL
			sec.get_parent().size_flags_vertical = Control.SIZE_EXPAND_FILL
			_build_editor_pane(sec)
			return
		_build_browse_phone_switcher(sec)
		if _browse_phone_pane == "tree":
			_build_browse_search(sec)
			var tree_wrap := VBoxContainer.new()
			tree_wrap.name = "VaultTreeWrap"
			sec.add_child(tree_wrap)
			_browse_tree_col = tree_wrap
			_build_browse_tree(tree_wrap, files)
		else:
			_build_note_pane(sec)
		return

	## The section takes the window's remaining height (`_rebuild()` stopped
	## the outer scroll for this), and the two panes split it with Godot's
	## own draggable divider -- the mockup's 300 px tree column, resizable.
	sec.size_flags_vertical = Control.SIZE_EXPAND_FILL
	sec.get_parent().size_flags_vertical = Control.SIZE_EXPAND_FILL
	var row := HSplitContainer.new()
	row.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	row.size_flags_vertical = Control.SIZE_EXPAND_FILL
	## Measured on 4.7.1 with this child layout (tree column not expanding,
	## preview expanding): the offset lands as the tree column's WIDTH -- 80
	## drew it at its 220 px floor, a 150 px drag from there read back 370 --
	## so the mockup's 300 is 300, not +80. Applied on the first
	## real-width layout, not here: a split sorted while the window is still
	## hidden clamps a stored offset to its zero width and keeps the clamp.
	## Held on the window, because `_rebuild()` makes a new split on every
	## pick and a divider that jumped back each click would not be one.
	row.resized.connect(func():
		if row.size.x > 520.0 and not row.has_meta("placed"):
			row.set_meta("placed", true)
			row.set_deferred("split_offsets", PackedInt32Array([_browse_split])))
	row.dragged.connect(func(offset: int): _browse_split = offset)
	sec.add_child(row)
	var tree_col := VBoxContainer.new()
	tree_col.name = "VaultTreeCol"
	tree_col.custom_minimum_size.x = 220
	tree_col.size_flags_vertical = Control.SIZE_EXPAND_FILL
	row.add_child(tree_col)
	## Mockup order: the search box sits at the very head of the tree
	## column, above the tree itself. `tree_wrap` is a separate child so
	## `_refresh_browse_tree()` can clear and rebuild just the tree on every
	## keystroke without freeing the `LineEdit` above it.
	_build_browse_search(tree_col)
	var tree_wrap := VBoxContainer.new()
	tree_wrap.name = "VaultTreeWrap"
	tree_wrap.size_flags_vertical = Control.SIZE_EXPAND_FILL
	tree_col.add_child(tree_wrap)
	_browse_tree_col = tree_wrap
	_build_browse_tree(tree_wrap, files)
	if _editing():
		## Edit switches the SAME pane to the editor, and it fills it: no
		## scroller around it, the text area scrolls its own lines.
		var ed_col := VBoxContainer.new()
		ed_col.name = "VaultEditorPane"
		ed_col.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		ed_col.size_flags_vertical = Control.SIZE_EXPAND_FILL
		row.add_child(ed_col)
		_build_editor_pane(ed_col)
		return
	var preview_scroll := ScrollContainer.new()
	preview_scroll.name = "VaultNoteScroll"
	preview_scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	preview_scroll.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	preview_scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	preview_scroll.add_theme_stylebox_override("panel", DccTheme.empty())
	row.add_child(preview_scroll)
	var preview_col := VBoxContainer.new()
	preview_col.name = "VaultNotePane"
	preview_col.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	preview_scroll.add_child(preview_col)
	_build_note_pane(preview_col)


## The mockup's own "Search notes…" field, at the head of the tree column.
##
## Typing filters the tree live, on names, straight into
## `_refresh_browse_tree()` -- not a full `_rebuild()`, which would free this
## very `LineEdit` mid-type and take the caret with it. That is a client-side
## substring match over paths already in memory, so a keystroke costs no read.
## **Enter** runs `vault_search` -- the search that looks INSIDE notes, with
## its indexed/scanned/truncated honesty -- into `_search_box` under the field,
## the same results `_build_search()` draws in the overview. That keeps §9's
## content search in the browser without the separate "Find a note" section
## the mockup does not draw.
func _build_browse_search(parent: Control) -> void:
	var wrap := MarginContainer.new()
	for side in ["left", "top", "right", "bottom"]:
		wrap.add_theme_constant_override("margin_" + side, 6)
	parent.add_child(wrap)
	var field := LineEdit.new()
	field.placeholder_text = "Search notes…"
	field.text = _search_query
	field.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	field.tooltip_text = "Typing narrows the tree by name. Enter also searches inside the notes -- once the content index exists; the results say when it does not."
	DccWidgets.well(field)
	field.text_changed.connect(func(t: String):
		_search_query = t
		_refresh_browse_tree())
	field.text_submitted.connect(func(t: String):
		_search_query = t
		_run_search())
	wrap.add_child(field)
	_search_box = VBoxContainer.new()
	_search_box.name = "VaultSearchResults"
	_search_box.add_theme_constant_override("separation", 2)
	_search_box.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	parent.add_child(_search_box)
	_fill_search_results()


# -- The note pane ------------------------------------------------------------

## The right-hand pane (Ruling BE): the **Attached to** block, the note itself
## (read-only until Open to edit), then the collapsed **More**. An entity
## opened with no note gets its attach flow in the note's place.
func _build_note_pane(parent: Control) -> void:
	if _pick_file == "":
		if _kind != "":
			_build_attach_flow_empty(parent)
		else:
			DccWidgets.note(parent, "Select a file in the tree.")
		_build_more(parent)
		return
	var data := bridge.vault_file_data(_pick_file)
	var fm: Dictionary = data.get("frontmatter", {}) if bool(data.get("ok", false)) else {}
	_build_attached_to(parent, fm)
	_build_browse_preview(parent, data)
	_build_more(parent)


## `open_for()` on an entity with no note yet: what to do about it, in the
## note's own place, plus the template flow (the one act that needs no note to
## exist first).
func _build_attach_flow_empty(parent: Control) -> void:
	var sec := DccWidgets.section(parent, "Attach a note")
	sec.name = "VaultAttachFlow"
	DccWidgets.note(sec, "%s has no note attached yet. Pick one in the tree -- its Attached to block then offers Attach to %s -- or start a new one from a template." % [_entity_label, _entity_label])
	_build_create(sec, true)


## True while the note pane is the Markdown editor on the picked file.
func _editing() -> bool:
	return _browse_path != "" and _browse_path == _pick_file


## Whether the editor holds text that is not on disk. Measured against
## `_browse_saved`, not against the editor, so it survives a rebuild.
func _is_dirty() -> bool:
	return _browse_path != "" and _live_text() != _browse_saved


## Runs `then` once any unsaved edit is dealt with: at once when there is
## none, otherwise only after the user chooses to discard it. Keeping the
## edits is the dialog's Cancel, which runs `on_keep` and loses nothing.
func _guard_dirty(then: Callable, on_keep: Callable = Callable()) -> void:
	## VE-4: leaving a note saves it (through the same hash guard). Only a save that could NOT land
	## (a conflict, a deleted or read-only file, a vanished vault) falls through to the prompt.
	_write_now()
	if not _is_dirty():
		then.call()
		return
	DccWidgets.confirm(PopoutHost.dialog_host(app, self), "Unsaved edits",
		"%s has edits that are not saved. Discard them?" % _browse_path.get_file(),
		"Discard edits", func():
			_close_editor()
			then.call(), on_keep)


## Discards the in-memory editor session (path, text, saved copy, hash) and
## returns the editor to its default Write mode. Writes nothing and does not
## rebuild; the caller redraws. Callers have either saved or have the user's
## consent to discard (`_guard_dirty`, `_warn_unsaved`).
func _close_editor() -> void:
	if _autosave_timer != null:
		_autosave_timer.stop()
	_save_block = {}
	_saved_stamp = ""
	_browse_path = ""
	_browse_text = ""
	_browse_hash = ""
	_browse_saved = ""
	_editor_mode = "write"


## Picks a note (the tree, a search hit). A dirty editor asks first; keeping
## the edits rebuilds, which puts the tree's highlight back where it was.
func _select_file(rel: String) -> void:
	if rel == _pick_file:
		return
	_guard_dirty(func():
		_link_back.clear()
		_pick_file = rel
		_pick_heading = ""
		_reader_link = ""
		_attach_open = false
		_browse_path = ""
		_browse_phone_pane = "preview"
		_rebuild(), _rebuild)


# -- Attached to --------------------------------------------------------------

## Every link on the picked note, then a frontmatter match that is not a link,
## then the ways to make one. Read from the link store (`vault_all_links`,
## filtered to this path) -- the same store the old KNOWLEDGE list read through
## `vault_links_for`, seen from the note's side rather than the entity's.
func _build_attached_to(parent: Control, fm: Dictionary) -> void:
	var sec := DccWidgets.section(parent, "Attached to")
	sec.name = "VaultAttachedTo"
	var links := _links_for_file(_pick_file)
	var linked := {}
	for l in links:
		var d: Dictionary = l
		linked["%s:%d" % [String(d.get("entity_kind", "")), int(d.get("entity_id", 0))]] = true
		_build_attached_row(sec, d)

	var fm_ent := _frontmatter_entity(fm)
	var fm_key := "" if fm_ent.is_empty() else "%s:%d" % [String(fm_ent["kind"]), int(fm_ent["id"])]
	if fm_key != "" and not linked.has(fm_key):
		_build_frontmatter_row(sec, fm_ent)
	if links.is_empty() and (fm_key == "" or linked.has(fm_key)):
		DccWidgets.note(sec, "Not attached to anything. A note does not know about a place until it is attached to it.")

	var quick := _kind != "" and not linked.has("%s:%d" % [_kind, _entity_id]) and fm_key != "%s:%d" % [_kind, _entity_id]
	if quick or _attach_open:
		_build_section_choice(sec)
	if quick:
		var at := DccWidgets.action(sec, "Attach to %s" % _entity_label, func():
			_attach_entity(_kind, _entity_id, _entity_label), true)
		at.tooltip_text = "Reads the selection now and records the source's timestamp and content hash, so Cartalith can tell later whether the note changed."
	_build_attach_picker(sec)


## One link: the entity (name, kind, faction), the link's status, and what can
## be done from here.
func _build_attached_row(sec: Control, d: Dictionary) -> void:
	var kind := String(d.get("entity_kind", ""))
	var eid := int(d.get("entity_id", 0))
	var lid := String(d.get("link_id", ""))
	var ent := _resolve_entity(kind, eid)
	var name := String(ent.get("name", d.get("entity_label", "")))
	var box := VBoxContainer.new()
	box.name = "AttachedRow"
	box.set_meta("entity", "%s:%d" % [kind, eid])
	box.add_theme_constant_override("separation", 3)
	sec.add_child(box)
	var head := HBoxContainer.new()
	head.add_theme_constant_override("separation", 10)
	box.add_child(head)
	head.add_child(DccTheme.label(name, "text_bright", DccTheme.FS_BODY + 1))
	head.add_child(DccTheme.mono_label(_entity_line(kind, ent), "text_dim", DccTheme.FS_SMALL, 1))
	var sel := String(d.get("selection_label", ""))
	DccWidgets.note(box, "%s · %s" % [
		String(STATUS_TEXT.get(String(d.get("status", "")), String(d.get("status", "")))),
		sel if sel != "" else "whole document"])
	var acts := HFlowContainer.new()
	acts.add_theme_constant_override("h_separation", 6)
	acts.add_theme_constant_override("v_separation", 6)
	box.add_child(acts)
	_build_show_on_map(acts, kind, ent)
	if kind == "settlement":
		var pe := DccWidgets.action(acts, "Open place editor", func():
			app.open_place_editor(int(ent.get("index", 0))))
		if not ent.has("index"):
			pe.disabled = true
			pe.tooltip_text = "No settlement with tid %d exists in the currently generated world." % eid
		else:
			pe.tooltip_text = "Opens %s in the place editor." % name
	if String(d.get("status", "")) == "stale":
		## §14's three-way prompt: Reload and Compare are buttons; Keep is
		## pressing neither.
		var reload := DccWidgets.action(acts, "Reload source", func(): _reload_link(lid))
		reload.tooltip_text = "Discards the Cartalith working copy and re-reads the section from the vault. The vault is not written."
		var link_rel := String(d.get("path", ""))
		var compare := DccWidgets.action(acts, "Compare…", func(): _compare_link(lid, link_rel))
		compare.tooltip_text = "A line-by-line diff between %s as it is right now and your working copy, so you can judge before choosing Reload or Keep." % link_rel.get_file()
	var detach := DccWidgets.action(acts, "Detach", func():
		bridge.vault_detach(lid)
		if lid == _reader_link:
			_reader_link = ""
		store_changed.emit()
		_rebuild())
	detach.tooltip_text = "Removes the link. The Markdown file is not touched — including any Cartalith block already written into it, which stays until you remove it explicitly."
	sec.add_child(DccTheme.rule())


## A frontmatter `type` + `tid` (or `id`) that names a real entity of this
## world, and no link to it: shown for what it is -- a claim the note makes
## about itself, which Cartalith has not acted on -- with the one press that
## makes it a link.
func _build_frontmatter_row(sec: Control, fm_ent: Dictionary) -> void:
	var kind := String(fm_ent["kind"])
	var eid := int(fm_ent["id"])
	var ent: Dictionary = fm_ent["entity"]
	var name := String(ent.get("name", ""))
	var box := VBoxContainer.new()
	box.name = "FrontmatterRow"
	box.add_theme_constant_override("separation", 3)
	sec.add_child(box)
	var head := HBoxContainer.new()
	head.add_theme_constant_override("separation", 10)
	box.add_child(head)
	head.add_child(DccTheme.label(name, "text", DccTheme.FS_BODY + 1))
	head.add_child(DccTheme.mono_label(_entity_line(kind, ent), "text_dim", DccTheme.FS_SMALL, 1))
	DccWidgets.note(box, "by frontmatter, not attached -- this note's own type/%s fields name it, but no link exists, so Cartalith has copied nothing from it." % ("tid" if kind == "settlement" else "id"))
	var acts := HFlowContainer.new()
	acts.add_theme_constant_override("h_separation", 6)
	box.add_child(acts)
	var at := DccWidgets.action(acts, "Attach", func(): _attach_entity(kind, eid, name), true)
	at.tooltip_text = "Attaches this whole note to %s." % name
	_build_show_on_map(acts, kind, ent)
	sec.add_child(DccTheme.rule())


## "SETTLEMENT · Kaldrune Dominion". The faction is named, never numbered, and
## a kind without one says nothing rather than inventing "unclaimed".
func _entity_line(kind: String, ent: Dictionary) -> String:
	var parts: Array = [kind.to_upper()]
	if ent.is_empty():
		parts.append("not in the current world")
	elif ent.has("faction"):
		parts.append(_faction_name(int(ent["faction"])))
	return " · ".join(PackedStringArray(parts))


## Adds the "Show on map" button for an entity. Enabled only when the entity
## resolved with an `x` position; otherwise it is disabled with a tooltip giving
## the reason for this `kind` (absent from the world, a kind this window does not
## look up, or no readable capital). Never moves the view to a made-up point.
func _build_show_on_map(parent: Control, kind: String, ent: Dictionary) -> void:
	var b := DccWidgets.action(parent, "Show on map", func():
		app.viewport.move_view_to(float(ent.get("x", 0.0)), float(ent.get("y", 0.0))))
	if ent.has("x"):
		b.tooltip_text = "Moves the map view to %s." % String(ent.get("name", ""))
		return
	b.disabled = true
	if ent.is_empty():
		b.tooltip_text = "Nothing by this id exists in the currently generated world, so there is no place to show."
	elif kind in ["culture", "landmark"]:
		b.tooltip_text = "This window resolves map positions for settlements, provinces, continents and factions only; a %s is not looked up here." % kind
	else:
		b.tooltip_text = "This %s has no position this window can read (a province needs a capital settlement, a faction a capital)." % kind


## Adds the "Section" choice for an attach: the whole document or one heading of
## `_pick_file` (§11's priority order), stored in `_pick_heading`. Arbitrary byte
## ranges are deliberately not offered; an offset goes stale as soon as the author
## edits above it.
func _build_section_choice(parent: Control) -> void:
	## §11's own priority order: whole document first, then a heading section.
	var headings := bridge.vault_file_headings(_pick_file)
	var h_labels: Array = ["Whole document"]
	var h_values: Array = [""]
	for h in headings:
		var d: Dictionary = h
		var lvl := int(d.get("level", 1))
		h_labels.append("%s%s" % ["  ".repeat(maxi(0, lvl - 1)), String(d.get("title", ""))])
		h_values.append(String(d.get("title", "")))
	DccWidgets.choice(parent, "Section", h_labels, maxi(0, h_values.find(_pick_heading)),
		func(i: int): _pick_heading = String(h_values[i]),
		"What the attach copies. Arbitrary text ranges are not offered: a byte offset stops pointing at the right paragraph the moment the author edits the text above it.")


## "Attach to…": any entity of this world, by kind and name.
func _build_attach_picker(sec: Control) -> void:
	if not _attach_open:
		var b := DccWidgets.text_button(sec, "Attach to…", func():
			_attach_open = true
			_rebuild())
		b.name = "AttachToOpen"
		b.alignment = HORIZONTAL_ALIGNMENT_LEFT
		b.tooltip_text = "Attach this note to any settlement, province, continent or faction."
		return
	var g := VBoxContainer.new()
	g.name = "AttachPicker"
	g.add_theme_constant_override("separation", 4)
	sec.add_child(g)
	g.add_child(DccTheme.mono_label("ATTACH TO…", "text_faint", DccTheme.FS_HEADER, 2))
	var kinds := bridge.vault_entity_kinds()
	var seg := HFlowContainer.new()
	seg.add_theme_constant_override("h_separation", 4)
	g.add_child(seg)
	for k in ["settlement", "province", "continent", "faction"]:
		if not kinds.has(k):
			continue
		var kk := String(k)
		var sb := DccWidgets.segment(seg, kk.capitalize(), func():
			_attach_kind = kk
			_rebuild())
		DccWidgets.set_segment_on(sb, kk == _attach_kind)
	var field := LineEdit.new()
	field.placeholder_text = "name"
	field.text = _attach_query
	field.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	DccWidgets.well(field)
	field.text_changed.connect(func(t: String):
		_attach_query = t
		_fill_attach_list())
	g.add_child(field)
	_attach_list = VBoxContainer.new()
	_attach_list.name = "AttachPickerList"
	_attach_list.add_theme_constant_override("separation", 2)
	g.add_child(_attach_list)
	_fill_attach_list()
	DccWidgets.text_button(g, "Cancel", func():
		_attach_open = false
		_rebuild())


## Refilled in place on every keystroke -- a rebuild would free the field.
func _fill_attach_list() -> void:
	if _attach_list == null or not is_instance_valid(_attach_list):
		return
	for c in _attach_list.get_children():
		_attach_list.remove_child(c)
		c.queue_free()
	var ents := _entity_list(_attach_kind)
	if ents.is_empty():
		DccWidgets.note(_attach_list, "No %s in the current world -- generate one first." % _attach_kind)
		return
	var linked := {}
	for l in _links_for_file(_pick_file):
		linked["%s:%d" % [String((l as Dictionary).get("entity_kind", "")), int((l as Dictionary).get("entity_id", 0))]] = true
	var q := _attach_query.strip_edges().to_lower()
	var shown := 0
	var matched := 0
	for e in ents:
		var d: Dictionary = e
		var nm := String(d["name"])
		if q != "" and nm.to_lower().find(q) < 0:
			continue
		matched += 1
		if shown >= 12:
			continue
		var eid := int(d["id"])
		var already := linked.has("%s:%d" % [_attach_kind, eid])
		var kind := _attach_kind
		var b := DccWidgets.action(_attach_list, nm + ("  (attached)" if already else ""), func():
			_attach_entity(kind, eid, nm))
		b.alignment = HORIZONTAL_ALIGNMENT_LEFT
		b.disabled = already
		shown += 1
	if matched == 0:
		DccWidgets.note(_attach_list, "No %s matches \"%s\"." % [_attach_kind, _attach_query.strip_edges()])
	elif matched > shown:
		DccWidgets.note(_attach_list, "%d more -- type to narrow." % (matched - shown))
	if _phone:
		app.phone_fit(_attach_list, 1.0)


## Attaches the picked note (`_pick_file`, `_pick_heading`) to the entity
## `kind`/`eid`, labelled `label`. On success it opens the new link in the reader
## and emits `store_changed`; a refusal is reported on the status line. Closes the
## attach picker and rebuilds either way.
func _attach_entity(kind: String, eid: int, label: String) -> void:
	var r := bridge.vault_attach(kind, eid, label, _pick_file, _pick_heading)
	if not bool(r.get("ok", false)):
		app.set_status("hint", "Attach: %s" % String(r.get("error", "refused")), "accent")
	else:
		_reader_link = String(r.get("link_id", ""))
		store_changed.emit()
		app.set_status("hint", "%s attached to %s." % [_pick_file.get_file(), label], "text")
	_attach_open = false
	_rebuild()


## Every link in the store whose file is `rel` (any entity's), or an empty array
## for an empty path. Scans `vault_all_links()`; read-only.
func _links_for_file(rel: String) -> Array:
	var out: Array = []
	if rel == "":
		return out
	for l in bridge.vault_all_links():
		if String((l as Dictionary).get("path", "")) == rel:
			out.append(l)
	return out


## The file a link points at, `""` for a link id the store does not hold.
func _link_rel(link_id: String) -> String:
	for l in bridge.vault_all_links():
		var d: Dictionary = l
		if String(d.get("link_id", "")) == link_id:
			return String(d.get("path", ""))
	return ""


## `{kind, id, entity}` when the note's own frontmatter names an entity this
## world has -- `type:` one of the vault's entity kinds, and `tid:` (or `id:`)
## an integer that resolves. `{}` otherwise, never a guess.
func _frontmatter_entity(fm: Dictionary) -> Dictionary:
	var kind := String(fm.get("type", "")).strip_edges().to_lower()
	if kind == "" or not bridge.vault_entity_kinds().has(kind):
		return {}
	var id_str := String(fm.get("tid", fm.get("id", ""))).strip_edges()
	if not id_str.is_valid_int():
		return {}
	var ent := _resolve_entity(kind, int(id_str))
	if ent.is_empty():
		return {}
	return {"kind": kind, "id": int(id_str), "entity": ent}


## One entity of the current world: `{name, faction, x, y, index}`, each key
## present only when this world actually has it (`x`/`y` a map position,
## `index` an index into `bridge.settlements()` for the place editor). `{}` when
## nothing by that id exists -- which is also the answer for a kind this window
## does not look up, and the callers say which. A landmark (Ruling EI) is found by its vault entity id in
## `bridge.landmark_names()` and carries `{name, x, y}` (its cell; no `faction`, no `index`).
func _resolve_entity(kind: String, id: int) -> Dictionary:
	match kind:
		"landmark":
			for n in bridge.landmark_names():
				var d: Dictionary = n
				if int(d.get("id", -1)) == id:
					var out := {"name": String(d.get("name", ""))}
					## The key is `kind@x,y`; the cell is read from it rather than guessed, and left out when
					## it does not parse (a landmark with no position offers no Centre on map).
					var cell := String(d.get("key", "")).get_slice("@", 1).split(",")
					if cell.size() == 2 and cell[0].is_valid_int() and cell[1].is_valid_int():
						out["x"] = float(int(cell[0]))
						out["y"] = float(int(cell[1]))
					return out
		"label", "icon":
			## Ruling EQ item 2: a hand-placed label or a placed icon is filed by its cell, so it is found by
			## asking the engine for each item's vault id (`vault_label_entity_id` / `vault_icon_entity_id`)
			## and matching. `{name, x, y}` (its grid position, which Show on map centres on); `{}` when it is
			## no longer placed, never a guessed position.
			var items: Array = bridge.label_list() if kind == "label" else bridge.icon_list()
			for it in items:
				var d: Dictionary = it
				var idx := int(d.get("index", -1))
				var got: Dictionary = bridge.vault_label_entity_id(idx) if kind == "label" else bridge.vault_icon_entity_id(idx)
				if bool(got.get("ok", false)) and got.has("entity_id") and int(got["entity_id"]) == id:
					return {"name": String(got.get("label", "")), "x": float(d.get("x", 0.0)), "y": float(d.get("y", 0.0))}
		"culture":
			## VE-9. A culture is addressed by its `CIV_CULTURES` index (`get_cultures`' `id`), has no map
			## position (so no Centre on map), and its name is the title-cased key the engine gives it.
			for c in bridge.get_cultures():
				var d: Dictionary = c
				if int(d.get("id", -1)) == id:
					return {"name": String(d.get("name", ""))}
		"settlement":
			var ss := bridge.settlements()
			for i in ss.size():
				var s: Dictionary = ss[i]
				if int(s.get("tid", -1)) == id:
					return {"name": String(s.get("name", "")), "faction": int(s.get("faction", 0)),
						"x": float(s.get("x", 0)), "y": float(s.get("y", 0)), "index": i}
		"province":
			for p in bridge.provinces():
				var d: Dictionary = p
				if int(d.get("id", -1)) == id:
					var out := {"name": String(d.get("name", "")), "faction": int(d.get("faction", 0))}
					var ci := int(d.get("capital_settlement_index", -1))
					var ss := bridge.settlements()
					if ci >= 0 and ci < ss.size():
						out["x"] = float((ss[ci] as Dictionary).get("x", 0))
						out["y"] = float((ss[ci] as Dictionary).get("y", 0))
					return out
		"continent":
			for c in bridge.continents():
				var d: Dictionary = c
				if int(d.get("id", -1)) == id:
					return {"name": String(d.get("name", "")), "faction": int(d.get("faction", 0)),
						"x": float(d.get("cx", 0.0)), "y": float(d.get("cy", 0.0))}
		"faction":
			for f in bridge.get_factions():
				var d: Dictionary = f
				if int(d.get("id", -1)) == id:
					var out := {"name": String(d.get("name", "")), "faction": id}
					for s in bridge.settlements():
						var sd: Dictionary = s
						if int(sd.get("faction", -1)) == id and bool(sd.get("capital", false)):
							out["x"] = float(sd.get("x", 0))
							out["y"] = float(sd.get("y", 0))
							break
					return out
	return {}


## `[{id, name}]` for the Attach-to picker.
func _entity_list(kind: String) -> Array:
	var out: Array = []
	match kind:
		"settlement":
			for s in bridge.settlements():
				out.append({"id": int((s as Dictionary).get("tid", 0)), "name": String((s as Dictionary).get("name", "?"))})
		"province":
			for p in bridge.provinces():
				out.append({"id": int((p as Dictionary).get("id", 0)), "name": String((p as Dictionary).get("name", "?"))})
		"continent":
			for c in bridge.continents():
				out.append({"id": int((c as Dictionary).get("id", 0)), "name": String((c as Dictionary).get("name", "?"))})
		"faction":
			for f in bridge.get_factions():
				out.append({"id": int((f as Dictionary).get("id", 0)), "name": String((f as Dictionary).get("name", "?"))})
		"culture":
			## VE-9: the seven cultures of `CIV_CULTURES`, which exist before any world does.
			for c in bridge.get_cultures():
				out.append({"id": int((c as Dictionary).get("id", 0)), "name": String((c as Dictionary).get("name", "?"))})
		"landmark":
			## Ruling EI: the generated (or user-given) name of every placed landmark.
			for n in bridge.landmark_names():
				out.append({"id": int((n as Dictionary).get("id", 0)), "name": String((n as Dictionary).get("name", "?"))})
	return out


## A faction's roster name. `0` is the engine's "held by no faction"
## (`get_continents()`' own doc), said as such; an id the roster does not
## carry is said as that too, never as a plausible name.
func _faction_name(fid: int) -> String:
	if fid <= 0:
		return "no faction"
	for f in bridge.get_factions():
		var d: Dictionary = f
		if int(d.get("id", -1)) == fid:
			return String(d.get("name", "faction %d" % fid))
	return "faction %d (not in the roster)" % fid


# -- Links: classing, following, Back, peek (VE-1) ------------------------------

## What a link `note_part` (the text before any `#heading` or `^block`) written in the note at
## `from_rel` names, as `{kind}`: `note` (plus `path`, or `candidates` when more than one note could
## mean it), `entity` (a settlement, province, continent, faction, culture or landmark of this world that has no
## note by that name; plus `ent_kind` and `id`; cultures since VE-9, landmarks since Ruling EI), `unresolved` (nothing by that name), or `unknown` (the
## resolver could not answer -- generating, or no vault bound -- plus `why`). `unknown` is its own
## answer on purpose: reading a failed lookup as `unresolved` would offer to CREATE a note that may
## well exist (`MISTAKES.md`: never encode "no value" as a plausible value).
func _link_kind(from_rel: String, note_part: String) -> Dictionary:
	var r := bridge.vault_resolve_link(from_rel, note_part)
	if not bool(r.get("ok", false)):
		return {"kind": "unknown", "why": String(r.get("error", "the vault could not be read"))}
	match String(r.get("status", "")):
		"unique":
			return {"kind": "note", "path": String(r.get("path", ""))}
		"ambiguous":
			return {"kind": "note", "candidates": r.get("candidates", [])}
	var e: Dictionary = _entity_names().get(note_part.strip_edges().to_lower(), {})
	if not e.is_empty():
		return {"kind": "entity", "ent_kind": String(e["kind"]), "id": int(e["id"]), "name": String(e["name"])}
	return {"kind": "unresolved"}


## How many notes the browse tree asks the crate about per pass (`_badge_tree`): a folder marker costs no read,
## any other note one, so a vault of thousands is badged over a few frames instead of in one stall. A
## labelled judgement (a read is well under a millisecond on the machines measured).
const PRIV_CHUNK := 400
## Bumped by every tree build, so a pass belonging to a tree that has been replaced stops.
var _priv_pass := 0


## Ruling DP (VE-9): draws the private badge on the browse tree's note rows. The first chunk is decided before the
## tree is first drawn; the rest follow a frame at a time. A note that could not be decided (unreadable) keeps no
## badge, and a pass that decides nothing stops rather than ask again forever.
func _badge_tree(leaves: Array) -> void:
	_priv_pass += 1
	var mine := _priv_pass
	var rels: Array = []
	for lv in leaves:
		rels.append(String((lv as Array)[1]))
	while not rels.is_empty():
		var n: int = privacy().decide(rels, PRIV_CHUNK)
		for lv in leaves:
			var item: TreeItem = (lv as Array)[0]
			var rel := String((lv as Array)[1])
			if is_instance_valid(item) and privacy().is_private(rel) and not item.get_text(0).ends_with(Privacy.SUFFIX):
				item.set_text(0, item.get_text(0) + Privacy.SUFFIX)
				var tip := item.get_tooltip_text(0)
				item.set_tooltip_text(0, ("%s%sPrivate: %s." % [tip, " " if tip != "" else "", privacy().reason(rel)]))
		if n == 0:
			return
		rels = privacy().pending(rels)
		if rels.is_empty():
			return
		await get_tree().process_frame
		if mine != _priv_pass or not is_inside_tree():
			return


## The private-note cache (`vault_privacy.gd`), shared with the side panel and the map's Find notes scope
## (Ruling DP: a private note is shown as private wherever a note is listed or linked).
func privacy() -> RefCounted:
	if _privacy == null:
		_privacy = Privacy.new(bridge)
	return _privacy


## `_ent_names`, built on first use. The first entity of a name wins (names are not unique across
## kinds; a note link would be unambiguous, an entity link is only a courtesy).
func _entity_names() -> Dictionary:
	if _ent_names.is_empty():
		## Cultures next-to-last (VE-9): a culture is a generic word ("Highland"), so a settlement, province,
		## continent or faction of that name must win the link. Landmarks last (Ruling EI): their names are
		## "Waterfall near Vhal Serai", so a real place or note of the same words still wins.
		for kind in ["settlement", "province", "continent", "faction", "culture", "landmark"]:
			for ent in _entity_list(kind):
				var n := String((ent as Dictionary).get("name", "")).strip_edges()
				var key := n.to_lower()
				if n != "" and n != "?" and not _ent_names.has(key):
					_ent_names[key] = {"kind": kind, "id": int((ent as Dictionary).get("id", 0)), "name": n}
	return _ent_names


## The order the Rust ranker's classes sort in (`suggest::MatchKind`), for merging notes and places
## into one list. Index = rank; an unknown class sorts last rather than first.
const _MATCH_ORDER := ["exact", "prefix", "word_start", "substring", "subsequence"]


## The editor's `[[` suggestions (VE-2): ranked rows for `mode` `note` (notes by name and alias, then the
## world's places that have no note, then "Link to new note") or `heading` (the headings of `note`, or of
## this note when `note` is empty). At most `limit` rows, never the whole vault. A row is `{kind, label,
## insert, detail?, alias?, enabled?}`; a line that only explains (`kind` `info`, disabled) stands in for
## an answer the vault could not give, rather than an empty card that reads as "nothing".
func _suggest_rows(mode: String, note: String, query: String, limit: int) -> Array:
	if mode == "heading":
		return _suggest_headings(note.strip_edges(), query.strip_edges(), limit)
	var q := query.strip_edges()
	var ranked: Array = []
	var seen_names := {}
	## The notes, with headroom: a second note of the same name ranks next to the first, so asking for
	## three times the rows is how a duplicate is seen without a resolver call per row (each of which
	## would list the vault again).
	var notes: Array = bridge.vault_suggest_notes(q, limit * 3)
	var dup := {}
	for n in notes:
		var k := String((n as Dictionary).get("name", "")).to_lower()
		dup[k] = int(dup.get(k, 0)) + 1
	for n in notes:
		var d: Dictionary = n
		var path := String(d.get("path", ""))
		if path == _browse_path:
			continue
		var name := String(d.get("name", ""))
		seen_names[name.to_lower()] = true
		## A name shared by two notes is written as its path, so the link names one of them.
		var target := name if int(dup.get(name.to_lower(), 0)) < 2 else path.trim_suffix(".md")
		var row := {"kind": "note", "insert": target, "label": name, "path": path,
			"detail": path.get_base_dir() if path.get_base_dir() != "" else "",
			"cls": _MATCH_ORDER.find(String(d.get("match", ""))), "sort": String(d.get("matched", name)).to_lower(), "tie": 0}
		if d.has("alias"):
			row["label"] = "%s  →  %s" % [String(d["alias"]), name]
			row["alias"] = String(d["alias"])
		ranked.append(row)
	var cands: Array = []
	for e in _entity_names().values():
		cands.append({"name": String((e as Dictionary)["name"]), "kind": String((e as Dictionary)["kind"])})
	for s in bridge.vault_suggest(q, cands, limit * 3):
		var sd: Dictionary = s
		var ename := String(sd.get("name", ""))
		if seen_names.has(ename.to_lower()):
			continue
		ranked.append({"kind": "entity", "insert": ename, "label": ename,
			"detail": "%s, no note yet" % String(sd.get("kind", "place")),
			"cls": _MATCH_ORDER.find(String(sd.get("match", ""))), "sort": String(sd.get("matched", ename)).to_lower(), "tie": 1})
	ranked.sort_custom(func(a: Dictionary, b: Dictionary) -> bool:
		var ca := int(a["cls"]) if int(a["cls"]) >= 0 else 99
		var cb := int(b["cls"]) if int(b["cls"]) >= 0 else 99
		if ca != cb:
			return ca < cb
		## Within a class a note comes before a place with no note (the places are the second source,
		## and an empty query would otherwise open on whichever world names sort first), then by name.
		if int(a["tie"]) != int(b["tie"]):
			return int(a["tie"]) < int(b["tie"])
		return String(a["sort"]) < String(b["sort"]))
	var exact := false
	for r in ranked:
		if q != "" and int((r as Dictionary)["cls"]) == 0:
			exact = true
	var make_new := q != "" and not exact
	var keep := limit - (1 if make_new else 0)
	var rows: Array = ranked.slice(0, maxi(keep, 0))
	## Ruling DP: a private note is offered as private (at most `limit` notes are asked about, never the vault).
	for r in rows:
		var rd: Dictionary = r
		if String(rd.get("kind", "")) == "note" and rd.has("path"):
			var pr := String(rd["path"])
			privacy().decide([pr])
			if privacy().is_private(pr):
				rd["detail"] = (String(rd.get("detail", "")) + Privacy.SUFFIX).strip_edges(true, false)
				rd["private"] = true
	if make_new:
		rows.append({"kind": "create", "insert": q, "label": "Link to new note “%s”" % q})
	return rows


## Heading rows for `[[note#query`: the headings of the note `note` names (this note when empty), ranked.
## A note that cannot be read, is ambiguous or does not exist answers with one disabled line saying so.
func _suggest_headings(note: String, q: String, limit: int) -> Array:
	var text := _browse_text
	var shown := "this note"
	if note != "":
		shown = note
		var r := bridge.vault_resolve_link(_browse_path, note)
		if not bool(r.get("ok", false)):
			return [_info_row("Cannot list headings now: %s" % String(r.get("error", "the vault could not be read")))]
		match String(r.get("status", "")):
			"unique":
				text = bridge.vault_read_file(String(r.get("path", "")))
			"ambiguous":
				return [_info_row("%s could mean several notes; write its folder too" % note)]
			_:
				return [_info_row("No note called %s" % note)]
	var cands: Array = []
	for h in bridge.vault_note_headings(text):
		cands.append({"name": String((h as Dictionary).get("text", "")), "kind": "heading"})
	if cands.is_empty():
		return [_info_row("No headings in %s" % shown)]
	var rows: Array = []
	for s in bridge.vault_suggest(q, cands, limit):
		var name := String((s as Dictionary).get("name", ""))
		## `[ ] | # ^` end or split a link; Obsidian writes them as spaces in a heading link.
		var safe := name
		for ch in ["[", "]", "|", "#", "^"]:
			safe = safe.replace(ch, " ")
		rows.append({"kind": "heading", "label": name, "insert": " ".join(safe.split(" ", false))})
	if rows.is_empty():
		return [_info_row("No heading in %s matches “%s”" % [shown, q])]
	return rows


## A suggestion line that only says something (`enabled` false).
func _info_row(msg: String) -> Dictionary:
	return {"kind": "info", "label": msg, "insert": "", "enabled": false}


## Follows `target` (the text between the brackets, `#heading` included) written in `from_rel`.
## A link to a note opens it in Write (Ruling DY, VQ4) and pushes the current note onto the Back
## trail; one that could mean several notes asks which; a link to a world entity with no note, and a
## link to nothing, only OFFER to create a note (VQ2, VQ5): creating is a menu item the user picks,
## never a side effect of following. `anchor` is the control the menu opens beside.
func _follow_link(from_rel: String, target: String, anchor: Control) -> void:
	var note_part := target.get_slice("#", 0).get_slice("^", 0).strip_edges()
	if note_part == "":
		_jump_to_heading(target.get_slice("#", 1).get_slice("^", 0).strip_edges())
		return
	var info := _link_kind(from_rel, note_part)
	match String(info["kind"]):
		"note":
			if info.has("path"):
				_open_linked(String(info["path"]))
			else:
				var items: Array = []
				for c in info.get("candidates", []):
					var path := String(c)
					items.append({"label": privacy().badged(path, path), "call": func(): _open_linked(path)})
				_show_link_menu("%s could mean several notes" % note_part, items, anchor)
		"entity":
			var ent := _resolve_entity(String(info["ent_kind"]), int(info["id"]))
			var items2: Array = _create_items(String(info["name"]), bridge.vault_suggested_path(String(info["ent_kind"]), String(info["name"])),
				{"kind": String(info["ent_kind"]), "id": int(info["id"])})
			if ent.has("x") and ent.has("y"):
				items2.append({"label": "Centre on map", "call": func():
					app.viewport.move_view_to(float(ent["x"]), float(ent["y"]))})
			_show_link_menu("%s is a %s with no note yet" % [info["name"], info["ent_kind"]], items2, anchor)
		"unresolved":
			var rel := note_part if note_part.ends_with(".md") else note_part + ".md"
			_show_link_menu("No note called %s" % note_part, _create_items(note_part.get_file().trim_suffix(".md"), rel), anchor)
		_:
			app.set_status("hint", "Cannot follow that link now: %s" % String(info.get("why", "")), "accent")


## `[[#Heading]]` in the open note: moves the editor to that heading (VE-5's outline). The match is
## case-insensitive on the heading text the crate reports; no heading of that name says so and moves nothing.
func _jump_to_heading(name: String) -> void:
	if _editor == null or not is_instance_valid(_editor):
		return
	var want := name.to_lower()
	for h in bridge.vault_note_headings(_live_text()):
		var hd: Dictionary = h
		if String(hd.get("text", "")).strip_edges().to_lower() == want and want != "":
			_editor.goto_line(int(hd.get("line", 0)), String(hd.get("text", "")))
			return
	app.set_status("hint", "No heading called \"%s\" in this note." % name, "accent")


## Menu rows that create the note `rel` from each of the vault's templates (the creator never
## overwrites), then a blank note, so a vault with no templates is not a dead end.
##
## `entity` (VE-6) is `{kind, id}` when the link names a world entity: the new note is then bound to it
## (`vault_attach`, a `KnowledgeLink`), so the map, the Inspect panel and the Note tab find it. A link to
## nothing has no entity and creates a plain note. A blank note is always offered, so a vault with no
## templates is not a dead end. Binding belongs to the person's one choice of this menu row, never a second
## unasked act; if it is refused the note is still created and the status line says it is not linked.
func _create_items(name: String, rel: String, entity: Dictionary = {}) -> Array:
	var items: Array = []
	var made := func(r: Dictionary, how: String):
		if not bool(r.get("ok", false)):
			app.set_status("hint", "Create refused: %s" % String(r.get("error", "")), "accent")
			return
		var path := String(r.get("path", rel))
		if not entity.is_empty():
			var a := bridge.vault_attach(String(entity["kind"]), int(entity["id"]), name, path, "")
			if bool(a.get("ok", false)):
				store_changed.emit()
				app.set_status("hint", "%s created%s and linked to %s." % [path, how, name], "text")
			else:
				app.set_status("hint", "%s created%s, but could not be linked to %s: %s" % [path, how, name, String(a.get("error", ""))], "accent")
		_open_linked(path)
	for t in bridge.vault_templates():
		var td: Dictionary = t
		var trel := String(td.get("rel", ""))
		items.append({"label": "Create %s from %s" % [rel, String(td.get("label", trel))], "call": func():
			made.call(bridge.vault_create_from_template(trel, rel, name), " from %s" % String(td.get("label", trel)))})
	items.append({"label": "Create %s as a blank note" % rel, "call": func():
		var r := bridge.vault_create_file(rel, "# %s\n" % name)
		if bool(r.get("ok", false)):
			r["path"] = rel
		made.call(r, " blank")})
	return items


## Hides (or shows) this dialog's own button bar (VE-8). On the phone the editor pins its keyboard row to the
## window's bottom edge, which is where the bar sits; Godot picks the bar's internal controls before any content, so
## a bar left in place would take the touches meant for the row (a Close under a finger meant for Bold).
func _set_bar_hidden(on: bool) -> void:
	var ok := get_ok_button()
	if ok != null and ok.get_parent() is Control:
		(ok.get_parent() as Control).visible = not on


## Opens `path` in Write, from a link. A dirty editor asks first (`_guard_dirty`), so following a
## link never discards typing; keeping the edits leaves everything as it was.
##
## Docked (PC-5): a note that belongs to a place or faction is offered to `dock_link_select` first (a link
## "selects what it names", `PC_SHELL_PROPOSAL.md` §3.4), so the map, Inspect and Ask follow; the tab
## answers false when it is pinned or the note names nothing selectable, and the note then opens in the tab.
func _open_linked(path: String, line: int = -1) -> void:
	if _dock_host != null and dock_link_select.is_valid():
		var ent := _entity_of_note(path)
		if not ent.is_empty():
			var sel_kind := String(ent["kind"])
			var sel_id := int(ent["id"])
			_guard_dirty(func():
				_goto_pending = line
				if not bool(dock_link_select.call(sel_kind, sel_id, path)):
					_goto_pending = -1
					_open_linked_plain(path, line))
			return
	_open_linked_plain(path, line)


## `_open_linked`'s body: guard, push the Back trail, enter the note.
func _open_linked_plain(path: String, line: int = -1) -> void:
	_guard_dirty(func():
		if _pick_file != "" and _pick_file != path:
			_link_back.append(_pick_file)
		## Set only once the guard has let this through, so a cancelled open leaves no stale line behind.
		_goto_pending = line
		_enter_note(path))


## The place or faction a note is linked to, as `{kind, id}` in `DccApp.selection`'s vocabulary (a
## settlement's id there is its INDEX, found from the link's `tid`), or `{}` when the note is linked to
## neither (or the link names a thing this world no longer holds). Reads the link store, never a guess.
func _entity_of_note(rel: String) -> Dictionary:
	for l in bridge.vault_all_links():
		var d: Dictionary = l
		if String(d.get("path", "")) != rel:
			continue
		match String(d.get("entity_kind", "")):
			"settlement":
				var ss: Array = bridge.settlements()
				for i in ss.size():
					if int((ss[i] as Dictionary).get("tid", -1)) == int(d.get("entity_id", -2)):
						return {"kind": "settlement", "id": i}
			"faction":
				return {"kind": "faction", "id": int(d.get("entity_id", -1))}
			"landmark":
				## `DccApp.selection`'s landmark id is an index into `bridge.landmarks()`; the link holds the
				## vault entity id, so the index is found through the names (same order). A landmark no longer
				## placed gives nothing, and the note then opens in the tab.
				var names: Array = bridge.landmark_names()
				for i in names.size():
					if int((names[i] as Dictionary).get("id", -2)) == int(d.get("entity_id", -1)):
						return {"kind": "landmark", "id": i}
	return {}


## Back to the note the last followed link came from. Asks first if there are unsaved edits.
func _go_back() -> void:
	if _link_back.is_empty():
		return
	var prev := String(_link_back.back())
	_guard_dirty(func():
		_link_back.pop_back()
		_enter_note(prev))


## Makes `path` the picked note and opens it in the editor, in Write (the reader pane is skipped:
## Ruling DY, VQ4). Unguarded -- callers have dealt with unsaved edits.
func _enter_note(path: String) -> void:
	if _dock_host != null:
		## PC-5: a link followed inside the Note tab replaces the note in the tab, not in the window.
		_dock_follow(path)
		return
	_pick_file = path
	_pick_heading = ""
	_reader_link = ""
	_attach_open = false
	_browse_path = ""
	_browse_phone_pane = "preview"
	_open_editor()


## An external link is handed to the operating system, for the schemes a note can reasonably hold.
## Anything else (`file:`, `javascript:`, a custom scheme) is refused and said so.
func _open_external(url: String) -> void:
	var low := url.to_lower()
	if low.begins_with("http://") or low.begins_with("https://") or low.begins_with("mailto:"):
		OS.shell_open(url)
	else:
		app.set_status("hint", "Not opened: only http, https and mailto links leave the app.", "accent")


## BBCode for the peek card of link `target` written in `from_rel`: the note's name and its first
## lines (frontmatter skipped, at most `PEEK_LINES` non-empty lines and `PEEK_CHARS` characters), or
## a one-line description for a world entity or an ambiguous name; `""` when there is nothing to
## show (an unresolved link), so no card appears for it.
func _peek_for(from_rel: String, target: String) -> String:
	var note_part := target.get_slice("#", 0).get_slice("^", 0).strip_edges()
	if note_part == "":
		return ""
	var info := _link_kind(from_rel, note_part)
	match String(info["kind"]):
		"note":
			if not info.has("path"):
				return "[b]%s[/b]\nCould mean %d notes." % [note_part, (info.get("candidates", []) as Array).size()]
			var path := String(info["path"])
			privacy().decide([path])
			var priv_tag := "  [i]%s[/i]" % Privacy.WORD if privacy().is_private(path) else ""
			var text := MdEditor.strip_frontmatter(bridge.vault_read_file(path))
			var lines := PackedStringArray()
			var chars := 0
			for ln in text.split("\n"):
				if ln.strip_edges() == "":
					continue
				lines.append(ln)
				chars += ln.length()
				if lines.size() >= PEEK_LINES or chars >= PEEK_CHARS:
					break
			return priv_tag.strip_edges() + ("\n" if priv_tag != "" else "") + MdEditor.to_bbcode("\n".join(lines))
		"entity":
			return "[b]%s[/b]\nA %s of this world with no note yet." % [info["name"], info["ent_kind"]]
	return ""


## Where a decoded vault picture is registered so a Preview `[img]` can find it by name (VE-7). `BBCode`'s `[img]`
## loads a path through the resource cache, and `Resource.take_over_path` puts a texture in that cache under a path
## no file needs to exist at. The name carries the file's stamp, so a changed picture is a new name, never a stale hit.
const IMG_REG_PREFIX := "res://__vault_embed__/"
## Decoded pictures by vault path: `{stamp, src, w, h, tex}`. Held so the texture stays alive (the cache is weak) and so
## Split's re-render every 0.2 s does not decode the file again. Memory only.
var _img_cache: Dictionary = {}


## A picture of the vault for Preview: the crate's `vault_embed_open` answer `r` (kind `image`) in, `{src, w, h}` out,
## or `{}` when the bytes cannot be read or decoded (Preview then says so). The crate re-checks the path, the
## extension, containment under the vault folder and the size cap inside `vault_read_image`.
func _embed_image(r: Dictionary) -> Dictionary:
	var rel := String(r.get("path", ""))
	var stamp := String(r.get("stamp", ""))
	var hit: Variant = _img_cache.get(rel)
	if hit != null and String((hit as Dictionary)["stamp"]) == stamp:
		return {"src": hit["src"], "w": hit["w"], "h": hit["h"]}
	var bytes: PackedByteArray = bridge.vault_read_image(rel)
	if bytes.is_empty():
		return {}
	var img := Image.new()
	var ok := false
	## The extension's decoder first, then the others: a `.png` that is really a JPEG still draws.
	var ext := rel.get_extension().to_lower()
	var order: Array = ["png", "jpg", "webp", "bmp", "svg"]
	if ext == "jpeg":
		ext = "jpg"
	if order.has(ext):
		order.erase(ext)
		order.push_front(ext)
	for kind in order:
		var e := ERR_FILE_UNRECOGNIZED
		match String(kind):
			"png": e = img.load_png_from_buffer(bytes)
			"jpg": e = img.load_jpg_from_buffer(bytes)
			"webp": e = img.load_webp_from_buffer(bytes)
			"bmp": e = img.load_bmp_from_buffer(bytes)
			"svg": e = img.load_svg_from_buffer(bytes)
		if e == OK and not img.is_empty():
			ok = true
			break
	if not ok:
		return {}
	var tex := ImageTexture.create_from_image(img)
	var src := "%s%s_%s.png" % [IMG_REG_PREFIX, rel.md5_text(), stamp.replace(":", "_")]
	tex.take_over_path(src)
	_img_cache[rel] = {"stamp": stamp, "src": src, "w": img.get_width(), "h": img.get_height(), "tex": tex}
	return {"src": src, "w": img.get_width(), "h": img.get_height()}


## A chronos row was clicked in Preview (VE-7): move Cartalith's year to the row's through the shell's one writer,
## `tl_set_year` (PC-2), which clamps to the world's own range (`tl_range`, PC-3). Says what happened: no world,
## or a year past the range and where it went instead. Nothing here writes the note or the world.
func _on_chronos_year(year: int) -> void:
	if app == null or not app.has_method("tl_set_year"):
		return
	if not app.tl_available():
		app.set_status("hint", "Generate a world first: the year belongs to a world, and there is none to move.", "accent")
		return
	app.tl_set_year(year)
	var now: int = app.tl_year()
	if now == year:
		app.set_status("hint", "Timeline moved to year %d." % year, "accent")
	else:
		app.set_status("hint", "Year %d is outside this world's range; the timeline went to %d." % [year, now], "accent")


## Peek card limits: a labelled judgement (enough to recognise a note, short enough to read in the
## second the card is up).
const PEEK_LINES := 8
const PEEK_CHARS := 420


## Opens a menu of `items` (`[{label, call, disabled?}]`) beside `anchor`, under `heading` as a
## disabled first row. A row's `call` runs when it is chosen; dismissing the menu runs nothing.
func _show_link_menu(heading: String, items: Array, anchor: Control) -> void:
	var pm := PopupMenu.new()
	pm.name = "LinkMenu"
	pm.add_item(heading)
	pm.set_item_disabled(0, true)
	pm.add_separator()
	for i in items.size():
		var it: Dictionary = items[i]
		pm.add_item(String(it["label"]), i)
		if bool(it.get("disabled", false)):
			pm.set_item_disabled(pm.item_count - 1, true)
	pm.id_pressed.connect(func(id: int):
		var c: Callable = (items[id] as Dictionary)["call"]
		if c.is_valid():
			c.call())
	pm.popup_hide.connect(pm.queue_free, CONNECT_DEFERRED)
	add_child(pm)
	var r := MdEditor.anchor_rect(anchor)
	r.size = Vector2(24, 24)
	r.position += Vector2(48, 48)
	DccWidgets.popup_anchored(pm, r, 360)


# -- The Note tab's host of the editor (PC-5, Ruling DY) ----------------------

## **One note focus, one writer.** The Context panel's Note tab (`context_note.gd`) shows a place's linked
## note in the SAME editor and through the SAME writer as this window: it does not build an editor of its
## own, it asks this window to build one into a frame the tab owns. That is how the hazard VE-4 found (a
## second editor alive under another note's name) is closed by construction rather than by care:
##
##   * `_editor`, `_browse_edit`, `_browse_path`, `_browse_hash`, `_browse_saved` and `_hist` exist once.
##     At any moment the editor is in the window's body, or in the tab's frame (`_dock_host`), or nowhere.
##   * `dock_note` refuses while this window is shown (`visible`), and `_clear` frees the docked editor
##     before the window builds its own, so the two are never alive together.
##   * Every write, the autosave timer, Save now, leaving a note, the quit flush (`flush_for_exit`) and the
##     banner's choices, is `_write_now` or `_on_banner_choice`, whichever frame the editor sits in.
##   * Changing the note flushes the old one through `_write_now` first; if that is refused (a conflict, a
##     deleted or read-only file, a vanished vault) the old editor STAYS, with its banner, and the switch
##     is reported as held. Typed text is never dropped to make room for another note.
var _dock_host: Control = null
## The box `_build_editor_pane(…, true)` made inside `_dock_host`; freed with the editor.
var _dock_node: Control = null
## `(kind, id, path) -> bool`: set by the Note tab. Called when a link followed in the docked editor names a
## place or faction: true when the tab selected it (the panel follows), false to open the note in the tab.
var dock_link_select: Callable = Callable()


## Whether the editor is built into the Note tab right now.
func is_docked() -> bool:
	return _dock_host != null and _dock_node != null and is_instance_valid(_dock_node) \
		and _editor != null and is_instance_valid(_editor)


## The note the Note tab's editor holds, `""` when nothing is docked.
func docked_path() -> String:
	return _browse_path if is_docked() else ""


## Whether the note held here (docked or not) has text that is not on disk. The Note tab asks it to say
## "not saved yet" on a held switch.
func dock_dirty() -> bool:
	return _is_dirty()


## Docks note `rel` into `host` (a container the Note tab owns). Returns `{ok: true}` or `{ok: false, why}`
## with `why` one of:
##   `window`  this window is shown, so it owns the editor (its modal state must not be raced);
##   `held`    the note held here could not be saved (plus `path`, the note whose editor stays in `host`);
##   `read`    `rel` could not be read (the reason is on the status line, nothing changed).
## Never writes anything but through `_write_now`; never forces; never drops text. Docking the note that is
## already docked in `host` is a no-op that answers ok.
func dock_note(rel: String, host: Control) -> Dictionary:
	if visible:
		return {"ok": false, "why": "window"}
	if rel == "" or host == null or not is_instance_valid(host):
		return {"ok": false, "why": "read"}
	if is_docked() and _dock_host == host and _browse_path == rel:
		return {"ok": true}
	## A note held with no live editor and text not on disk (the window was rebuilt over a dirty dock):
	## its text is the user's. Put it back in front of them, with its banner, and report the switch held.
	var retained := _browse_path != "" and (_editor == null or not is_instance_valid(_editor)) and _browse_text != _browse_saved
	if retained:
		_dock_into(host, _browse_path)
		return {"ok": false, "why": "held", "path": _browse_path}
	if _browse_path != "":
		_write_now()
		if _is_dirty():
			if not is_docked():
				_dock_into(host, _browse_path)
			return {"ok": false, "why": "held", "path": _browse_path}
	_free_dock_node()
	_clear()
	_reset_note_state()
	_pick_file = rel
	if not _load_note_state(rel):
		_pick_file = ""
		return {"ok": false, "why": "read"}
	_editor_mode = "write"
	_dock_into(host, rel)
	return {"ok": true}


## Builds the editor for the state this window holds (`_browse_path`) into `host`. The state is already
## loaded; this only hosts it.
func _dock_into(host: Control, rel: String) -> void:
	_free_dock_node()
	_dock_host = host
	_pick_file = rel
	_build_editor_pane(host, true)


## A link followed inside the docked editor: the old note was flushed by `_guard_dirty`; the new one
## replaces it in the same frame. A note that cannot be read leaves the old editor where it was.
func _dock_follow(path: String) -> void:
	var host := _dock_host
	## `_load_note_state` changes nothing when the read fails, so the old editor stays on screen.
	if not _load_note_state(path):
		return
	_dock_into(host, path)


## Frees the docked editor's node and every reference to it. The unsaved-text state (`_browse_text`,
## `_browse_saved`) is not touched: a caller that discards has done `_close_editor`.
func _free_dock_node() -> void:
	if _dock_node != null and is_instance_valid(_dock_node):
		if _browse_path != "" and _editor_path == _browse_path and _browse_edit != null and is_instance_valid(_browse_edit):
			_browse_text = _browse_edit.text
			_checkpoint_history()
		var par := _dock_node.get_parent()
		if par != null:
			par.remove_child(_dock_node)
		_dock_node.queue_free()
	_dock_node = null
	_dock_host = null
	_editor = null
	_browse_edit = null
	_editor_path = ""


## The Note tab no longer shows a note (the subject has none, the tab was removed): flush it and let the
## editor go. Returns true when nothing is at risk. A refused save keeps the editor and answers false.
func release_dock() -> bool:
	if _dock_node == null:
		return true
	_write_now()
	if _is_dirty():
		return false
	_free_dock_node()
	_close_editor()
	return true


## The header's open-in-window button: the note goes to the vault window for split view, the outline, backlinks
## and rename. The window's own open guards the unsaved text (a refused save asks first).
func _open_docked_in_window() -> void:
	var rel := _browse_path
	if rel == "":
		return
	_write_now()
	if _is_dirty():
		app.set_status("hint", "Resolve the message above first: this note is not saved yet.", "accent")
		return
	open_browse()
	_enter_note(rel)


# -- The editor (Ruling BE) -----------------------------------------------------

## Reads the whole file (`vault_read_file_for_edit`, which returns its hash)
## and turns the note pane into the editor. The hash is what Save hands back.
func _open_editor() -> void:
	if not _load_note_state(_pick_file):
		return
	_editor_mode = "write"
	_browse_phone_pane = "preview"
	_rebuild()


## Reads `rel` and makes it the note this window holds (path, text, saved copy, hash, line-ending style,
## session-undo seed), writing nothing and building nothing. Returns false, with the reason on the status
## line and the window's state untouched, when the file cannot be read. The one reader behind both hosts
## of the editor: the browser pane (`_open_editor`) and the Context panel's Note tab (`dock_note`, PC-5), so
## a note is read, hashed and checkpointed the same way wherever it is edited.
func _load_note_state(rel: String) -> bool:
	var r := bridge.vault_read_file_for_edit(rel)
	if not bool(r.get("ok", false)):
		app.set_status("hint", "Read: %s" % String(r.get("error", "")), "accent")
		return false
	_browse_path = rel
	var raw := String(r.get("text", ""))
	## Line endings (VE-4): a CRLF note stays CRLF on write. This was `raw.contains("<LF>")` (a real line break
	## inside the quotes, from a tool that rewrote the escape), which is true for any note of two lines and
	## converted every LF note to CRLF on its first autosave; it is the escape again.
	_eol_crlf = raw.contains("\r\n")
	_browse_text = _norm_eol(raw)
	_browse_saved = _browse_text
	_browse_hash = String(r.get("hash", ""))
	_save_block = {}
	_saved_stamp = ""
	_autosave_timer.stop()
	_note_opened(_browse_path, _browse_text)
	return true


## Builds the Markdown editor (`MdEditor`) for the open note into `parent` and
## wires its signals: edits mirror into `_browse_text` and arm the autosave (`_on_editor_edited`), Save
## goes to `_save_note`, Revert to opened… to `_on_revert_requested`, the save banner's buttons to
## `_on_banner_choice`. The session undo is replayed into the new editor from `_hist` (VE-4). `[[`
## suggestions are ranked and capped by `_suggest_rows` (VE-2); the template inserter writes nothing
## itself, its result waits in the editor for the hash-guarded autosave. Shows "editing…" if the session
## is already dirty, and the banner again if a save is being refused.
##
## `docked` (PC-5): the editor is being built into the Context panel's Note tab, not the window's body. The
## same editor, the same signals and the same writer (`_write_now`) -- there is still exactly one editor and
## one `_browse_path`; what differs is the host (`_dock_host`, `_dock_node`), no side panel (the tab is a
## narrow frame; the full panel stays in the window, reached by the header's open-in-window button), and a
## link followed by Ctrl-click re-docks instead of rebuilding the window (`_enter_note`).
func _build_editor_pane(parent: Control, docked: bool = false) -> void:
	## Ruling DP (VE-9): the open note says so when it is private, whichever host it is in. Asked afresh (the
	## text may have just been saved with or without a marker), never assumed from an earlier list.
	privacy().forget(_browse_path)
	privacy().decide([_browse_path])
	if privacy().is_private(_browse_path):
		var pn := DccTheme.label("Private note: %s. Cartalith withholds it from the assistant's knowledge and place prose." % privacy().reason(_browse_path), "accent", DccTheme.FS_TINY)
		pn.name = "VaultPrivateNotice"
		pn.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		pn.custom_minimum_size.x = 160
		parent.add_child(pn)
	## VE-5: the editor and its side panel share one host, side by side on a wide window and stacked on a
	## narrow one (`SIDE_RIGHT_MIN_W`). The host fills the pane exactly as the bare editor did.
	var wide := (not _phone) and size.x >= SIDE_RIGHT_MIN_W and not docked
	var host: BoxContainer = HBoxContainer.new() if wide else VBoxContainer.new()
	host.name = "VaultEditorHost"
	host.add_theme_constant_override("separation", 10)
	host.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	host.size_flags_vertical = Control.SIZE_EXPAND_FILL
	parent.add_child(host)
	var ed = MdEditor.new()
	ed.name = "VaultMarkdownEditor"
	host.add_child(ed)
	var self_rel := _browse_path
	## The note-link picker's list is gone (VE-2): `[[` suggestions come from `_suggest_rows`, ranked and
	## capped, so nothing here lists the vault for the editor any more.
	## Insert template (Ruling BF): the vault's templates, and an inserter whose
	## `{{title}}` is this note's own name, as Obsidian's is. It writes
	## nothing; the result waits in the editor for Save's hash-guarded write.
	var note_title := self_rel.get_file().get_basename()
	## Links (VE-1): the editor holds no vault knowledge, so it is handed the classifier, the
	## link-under-the-caret reader and the peek text. `_browse_path` is read at call time, so a link in
	## whichever note is open is resolved from THAT note.
	ed.link_info = func(t: String) -> Dictionary: return _link_kind(_browse_path, t)
	ed.link_at = func(text: String, offset: int) -> Dictionary: return bridge.vault_link_at(text, offset)
	## VE-8: the soft keyboard's height for the phone's keyboard row. The shell polls it once
	## (`DccShell._process` keeps `_phone_kb_height`, which is what already sizes this window above the
	## keyboard); this reads that cache so there is no second `virtual_keyboard_get_height()` poll. -1 is
	## "this display has no soft keyboard to ask", never 0: the editor then falls back to the text's focus.
	ed.keyboard_mode_changed.connect(_set_bar_hidden)
	ed.keyboard_height = func() -> int:
		if not DisplayServer.has_feature(DisplayServer.FEATURE_VIRTUAL_KEYBOARD):
			return -1
		var kb = app.get("_phone_kb_height")
		return -1 if kb == null else int(kb)
	ed.peek_source = func(t: String) -> String: return _peek_for(_browse_path, t)
	## VE-7: what Preview draws beyond text. The crate decides every embed (`vault_embed_open`: a cycle, the
	## depth cap, a path that leaves the vault); this only reads through it. The chain starts at the open
	## note, and each embedded note is resolved from itself (`stack.back()`).
	ed.note_path = self_rel
	ed.embed_source = func(inner: String, stack: Array) -> Dictionary:
		var from := String(stack.back()) if not stack.is_empty() else _browse_path
		return bridge.vault_embed_open(from, inner, PackedStringArray(stack))
	ed.embed_image = func(r: Dictionary) -> Dictionary: return _embed_image(r)
	ed.chronos_source = func(t: String) -> Dictionary: return bridge.vault_note_chronos(t)
	ed.task_toggler = func(t: String, line: int) -> Dictionary: return bridge.vault_toggle_task(t, line)
	ed.chronos_year_requested.connect(_on_chronos_year)
	ed.suggest_source = func(mode: String, note: String, query: String, limit: int) -> Array:
		return _suggest_rows(mode, note, query, limit)
	## VE-3: the colouring's links and the footer's counts come from the crate through the bridge, so
	## the editor keeps no parser of its own. A build without the functions answers -1 and the footer
	## leaves that figure out.
	ed.link_spans = func(line: String) -> Array: return bridge.vault_link_spans(line)
	ed.word_counter = func(text: String) -> int: return bridge.vault_word_count(text)
	ed.link_counter = func(text: String) -> int: return bridge.vault_outgoing_link_count(text)
	var templates_source := func() -> Array: return bridge.vault_templates()
	var inserter := func(rel: String, text: String, caret: int) -> Dictionary:
		return bridge.vault_insert_template(rel, text, caret, note_title)
	ed.setup(_browse_path, _browse_text, _browse_saved, _phone, Callable(), _editor_mode,
		templates_source, inserter)
	_editor = ed
	_browse_edit = ed.text_edit
	_editor_path = _browse_path
	## The session undo: rebuild the editor's history from the checkpoints (VE-4).
	var snaps: Array = (_hist.get(_browse_path, []) as Array).duplicate()
	if snaps.is_empty() or String(snaps.back()) != _browse_text:
		snaps.append(_browse_text)
	ed.seed_history(snaps)
	ed.set_back_visible(not _link_back.is_empty())
	ed.link_followed.connect(func(t: String): _follow_link(_browse_path, t, ed))
	ed.external_link_followed.connect(_open_external)
	ed.back_requested.connect(_go_back)
	ed.edited.connect(func(): _on_editor_edited(ed))
	ed.mode_changed.connect(func(m: String): _editor_mode = m)
	ed.save_requested.connect(_save_note)
	ed.revert_requested.connect(_on_revert_requested)
	ed.banner_choice.connect(_on_banner_choice)
	if docked:
		_dock_node = host
		_side_panel = null
		_side_toggle = null
		var open_btn := ed.add_header_button("⧉", "Open this note in the vault window: split view, outline, backlinks, properties and rename.", _open_docked_in_window)
		open_btn.name = "NoteOpenInWindow"
		if _goto_pending >= 0:
			var l := _goto_pending
			_goto_pending = -1
			ed.goto_line.call_deferred(l)
	else:
		_build_side_panel(host, ed, wide)
	if not _save_block.is_empty():
		_show_block()
	elif _is_dirty():
		ed.set_status("editing…", false)
	elif _saved_stamp != "":
		ed.set_status("saved · %s" % _saved_stamp, false)


## Builds the side panel into `host` beside / below `ed` and the header button that shows it (VE-5).
## Everything it needs from this window is a Callable, so the panel knows neither the window nor the
## editor. It is built hidden-or-shown per `_side_wanted`; its content is drawn only while it is shown.
func _build_side_panel(host: BoxContainer, ed, wide: bool) -> void:
	var panel = SidePanel.new()
	panel.bridge = bridge
	panel.phone = _phone
	panel.keyboard_height = func() -> int:
		if not DisplayServer.has_feature(DisplayServer.FEATURE_VIRTUAL_KEYBOARD):
			return -1
		var kb = app.get("_phone_kb_height")
		return -1 if kb == null else int(kb)
	panel.keyboard_mode_changed.connect(_set_bar_hidden)
	panel.live_text = func() -> String: return _live_text()
	panel.note_rel = func() -> String: return _browse_path
	## On the phone the panel REPLACES the editor (a sheet over the note, `_set_side`): anything that takes the
	## person back to text (a backlink, a heading) closes it first.
	panel.open_note = func(rel: String, line: int):
		_set_side(false)
		_open_linked(rel, line)
	panel.jump_to = func(line: int, heading: String):
		_set_side(false)
		ed.goto_line(line, heading)
	panel.on_close = func(): _set_side(false)
	panel.apply_text = func(t: String): ed.replace_whole_text(t)
	panel.say = func(msg: String): app.set_status("hint", msg, "text_ghost")
	panel.rename_note = _request_rename
	panel.last_rename = func() -> Dictionary: return _rename_last
	panel.undo_rename = _undo_rename
	panel.private_suffix = func(rel: String) -> String:
		privacy().decide([rel])
		return privacy().suffix(rel)
	if wide:
		panel.custom_minimum_size.x = 300
		panel.size_flags_horizontal = Control.SIZE_FILL
	else:
		## Narrow window: below the editor. Phone: instead of the editor (it would not fit under it).
		panel.custom_minimum_size.y = 0 if _phone else 240
		panel.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	panel.size_flags_vertical = Control.SIZE_EXPAND_FILL if (wide or _phone) else Control.SIZE_FILL
	host.add_child(panel)
	panel.setup()
	_side_panel = panel
	panel.visible = _side_wanted()
	ed.visible = not (_phone and panel.visible)
	_side_toggle = ed.add_header_button("Panel", "Shows or hides the panel of backlinks, outline, properties and tags.", _toggle_side_panel)
	_side_toggle.name = "SidePanelToggle"
	if panel.visible:
		panel.rebuild()
	if _goto_pending >= 0:
		var l := _goto_pending
		_goto_pending = -1
		ed.goto_line.call_deferred(l)


## Whether the side panel is shown: the person's last choice, else open on a wide window only.
func _side_wanted() -> bool:
	if _side_user >= 0:
		return _side_user == 1
	return (not _phone) and size.x >= SIDE_RIGHT_MIN_W


func _toggle_side_panel() -> void:
	_set_side(not _side_wanted())


## Shows or hides the side panel and remembers the choice. On the phone the editor is hidden while the
## panel is shown (the panel is a sheet over the note, with a "Note" button back).
func _set_side(on: bool) -> void:
	_side_user = 1 if on else 0
	if _side_panel == null or not is_instance_valid(_side_panel):
		return
	_side_panel.visible = on
	if _editor != null and is_instance_valid(_editor):
		_editor.visible = not (_phone and on)
	if on:
		_side_panel.rebuild()
		if _phone and _editor != null and is_instance_valid(_editor):
			_editor.text_edit.release_focus()


## After a note was written: the backlink index follows (only if it was built -- a first build is a
## whole-vault read and is the person's to ask for), then the panel redraws. Deferred so a keystroke
## pause is never what waits for it. "The index refreshes on save and on rename, not only on a manual
## Refresh" (proposal s.3.2).
func _after_save() -> void:
	## Its frontmatter may have gained or lost a private marker: ask again next time it is listed.
	privacy().forget(_browse_path)
	dock_saved.emit()
	if bool(bridge.vault_backlink_stats().get("built", false)):
		bridge.vault_refresh_backlinks()
	if _side_panel != null and is_instance_valid(_side_panel):
		_side_panel.on_saved()


## Ctrl+S and the Save now button: the user's own save, answered by the same path as the autosave
## (`_write_now`), plus a status-bar line so the press is acknowledged.
func _save_note() -> void:
	if _editor == null or _browse_edit == null:
		return
	if _write_now():
		app.set_status("hint", "%s saved." % _browse_path.get_file(), "text_ghost")
	else:
		app.set_status("hint", "Save refused: %s" % String(_save_block.get("error", "")), "accent")


## The text as the editor holds it right now, falling back to the mirror when no editor is built.
## The mirror (`_browse_text`) is fed by a deferred signal and can be a frame behind a keystroke.
func _live_text() -> String:
	if _browse_edit != null and is_instance_valid(_browse_edit) and _editor_path == _browse_path:
		return _browse_edit.text
	return _browse_text


## Line endings as the editor holds them: LF. See `_eol_crlf`.
static func _norm_eol(t: String) -> String:
	return t.replace("\r\n", "\n")


## The text as it goes to disk: back to CRLF when the file was CRLF when read.
func _to_disk(t: String) -> String:
	return t.replace("\n", "\r\n") if _eol_crlf else t


## The one place a note is written (VE-4). Called by the autosave timer, by Save now, by leaving the
## note and by closing the window. Returns true when nothing unsaved remains afterwards.
##
## **The data-safety contract.** It writes only through `vault_write_file`, whose hash guard refuses
## when the file is not what was last read here; it never forces; a refusal keeps the text in the
## editor and puts a banner up (`_show_block`). It writes nothing when the text equals what was last
## saved (the crate also skips an identical write, so the file's time does not move). It never
## recreates a note that was deleted: that is a banner choice.
func _write_now() -> bool:
	if _autosave_timer != null:
		_autosave_timer.stop()
	if _browse_path == "" or _browse_edit == null or not is_instance_valid(_browse_edit) or _editor_path != _browse_path:
		return not _is_dirty()
	var text: String = _browse_edit.text
	_browse_text = text
	if text == _browse_saved:
		return true
	_writes_attempted += 1
	var r := bridge.vault_write_file(_browse_path, _to_disk(text), _browse_hash)
	if bool(r.get("ok", false)):
		_browse_saved = text
		_browse_hash = String(r.get("hash", ""))
		_save_block = {}
		_saved_stamp = Time.get_time_string_from_system()
		_checkpoint_history()
		if _editor != null:
			_editor.hide_banner()
			_editor.mark_saved(text, _saved_stamp)
		_after_save.call_deferred()
		return true
	_save_block = {"kind": String(r.get("kind", "io")), "error": String(r.get("error", "")), "actual": String(r.get("actual", ""))}
	_show_block()
	return false


## The text last saved as a rescue copy at exit, and where (`flush_for_exit`). One slot: the same text
## is never copied twice, so a pause that repeats while a conflict stands makes one file, not one per pause.
var _rescued_text := ""
var _rescued_rel := ""


## **Quitting must never drop typed text** (the VE-4 gap, closed in the VE-5 session). Called by
## `DccApp` before the app can end (the window's close request, Android's back at the end of its
## navigation, `NOTIFICATION_APPLICATION_PAUSED`, and window focus-out). It saves through `_write_now`,
## the only writer, so the hash guard holds and nothing is forced. If that save is REFUSED (a conflict, a
## deleted or read-only note) the user's text is not lost and not forced over theirs either: with
## `rescue` it is written beside the note as `Name (my edits <stamp>).md` through `vault_create_file`,
## which never overwrites (the same copy name Keep both uses), and the banner stays as it was.
## Returns true when nothing unsaved remains at risk (saved, or rescued), false when it could not be
## made safe (the vault folder is gone, or a generation holds the engine); the caller then keeps the app
## open and shows the window (`surface_for_exit`). `rescue` is false for a plain focus-out: leaving the
## window for a moment is not leaving the app, so no copy is made, only the ordinary guarded save.
func flush_for_exit(rescue: bool) -> bool:
	_write_now()
	if not _is_dirty():
		return true
	if not rescue:
		return false
	var text := _live_text()
	if text == _rescued_text and _rescued_rel != "" and _file_exists(_rescued_rel):
		return true
	var copy_rel := _conflict_copy_rel(_browse_path)
	if copy_rel == "":
		return false
	var c := bridge.vault_create_file(copy_rel, _to_disk(text))
	if not bool(c.get("ok", false)):
		return false
	_rescued_text = text
	_rescued_rel = copy_rel
	app.set_status("hint", "Your unsaved text was saved as %s." % copy_rel, "text")
	return true


## Brings the window back with its banner so the person sees why the app did not close
## (`flush_for_exit` returned false). The phone shows it as its sheet, which is already present.
func surface_for_exit() -> void:
	if _dock_host != null:
		## PC-5: the unsaved note is in the Context panel's Note tab; bring that forward, not an empty window.
		app.open_context_tab("note")
		_show_block()
		return
	if not visible and not DccWidgets.phone_present(self, app):
		popup()
	_show_block()


## An edit reached the editor: mirror it, and either arm the autosave or, when the text is back to
## what is saved (an undo all the way back), say so. A conflict or a deleted note pauses the timer:
## writing again would only be refused again, and the banner is already asking.
func _on_editor_edited(ed) -> void:
	_browse_text = ed.text_edit.text
	if _side_panel != null and is_instance_valid(_side_panel):
		_side_panel.on_text_changed()
	if _browse_text == _browse_saved:
		_autosave_timer.stop()
		if _save_block.is_empty():
			ed.set_status("saved · %s" % _saved_stamp if _saved_stamp != "" else "saved", false)
		return
	var k := String(_save_block.get("kind", ""))
	if k == "conflict" or k == "missing":
		ed.set_status("Not saved -- see above.", true)
		return
	if k == "":
		ed.set_status("editing…", false)
	_autosave_timer.start(AUTOSAVE_SEC)


## The banner for `_save_block`. One sentence on what happened and that nothing was lost, and the
## choices that fit that kind of refusal; a conflict never offers a bare "overwrite".
func _show_block() -> void:
	if _editor == null or _save_block.is_empty():
		return
	var k := String(_save_block["kind"])
	var msg := ""
	var choices: Array = []
	match k:
		"conflict":
			msg = "This note was changed outside Cartalith after you opened it, so nothing was saved and nothing of theirs was overwritten. Your text is still here."
			choices = [
				{"key": "keep_mine", "label": "Keep mine (replace theirs)"},
				{"key": "take_theirs", "label": "Take theirs"},
				{"key": "keep_both", "label": "Keep both"}]
		"missing":
			msg = "This note was deleted outside Cartalith. Your text is still here and is not saved anywhere."
			choices = [
				{"key": "recreate", "label": "Recreate with mine"},
				{"key": "discard", "label": "Close note"}]
		"unavailable":
			msg = "The vault folder is not available (moved, unplugged or deleted), so nothing was saved. Your text is still here."
			choices = [{"key": "retry", "label": "Try again"}]
		"readonly":
			msg = "This note could not be written: it is read-only or another program has it locked. Not saved; your text is still here."
			choices = [{"key": "retry", "label": "Try again"}]
		_:
			msg = "Not saved: %s Your text is still here." % String(_save_block.get("error", ""))
			choices = [{"key": "retry", "label": "Try again"}]
	_editor.show_banner(msg, choices)
	_editor.set_status("Not saved.", true)


## A banner button. Each choice that overwrites anything does so through the hash guard, after
## re-reading the file, so a second outside change since the banner went up raises the banner again.
func _on_banner_choice(key: String) -> void:
	if _editor == null or _browse_edit == null:
		return
	match key:
		"retry":
			_save_block = {}
			_write_now()
		"keep_mine":
			## Written against the hash of the version the banner told the person about (`actual`), not a
			## fresh read: if the file changed AGAIN since the banner went up, the guard refuses and the
			## banner returns, so a version the person never saw is never replaced.
			var seen := String(_save_block.get("actual", ""))
			if seen == "":
				_editor.set_status("Cannot tell which version to replace; use Take theirs or Keep both.", true)
				return
			_browse_hash = seen
			_save_block = {}
			_write_now()
		"take_theirs":
			_take_theirs()
		"keep_both":
			var copy_rel := _conflict_copy_rel(_browse_path)
			if copy_rel == "":
				_editor.set_status("Could not choose a name for the copy; nothing changed.", true)
				return
			var c := bridge.vault_create_file(copy_rel, _to_disk(_browse_edit.text))
			if not bool(c.get("ok", false)):
				_editor.set_status("Could not save your version as a copy: %s Nothing changed." % String(c.get("error", "")), true)
				return
			_take_theirs()
			app.set_status("hint", "Your version was saved as %s." % copy_rel, "text")
		"recreate":
			var c2 := bridge.vault_create_file(_browse_path, _to_disk(_browse_edit.text))
			if bool(c2.get("ok", false)):
				_browse_saved = _browse_edit.text
				_browse_hash = String(c2.get("hash", ""))
				_save_block = {}
				_saved_stamp = Time.get_time_string_from_system()
				_checkpoint_history()
				_editor.hide_banner()
				_editor.mark_saved(_browse_saved, _saved_stamp)
			elif String(c2.get("kind", "")) == "exists":
				## Someone put a note back while the banner was up: that is a conflict now, and the
				## stale hash makes the guard say so.
				_save_block = {"kind": "conflict", "error": ""}
				_show_block()
			else:
				_save_block = {"kind": String(c2.get("kind", "io")), "error": String(c2.get("error", ""))}
				_show_block()
		"discard":
			_hist.erase(_browse_path)
			_close_editor()
			_pick_file = ""
			_rebuild()
	## PC-5: a choice that leaves nothing unsaved (Take theirs, Keep both) writes nothing, so `_after_save`
	## never fires; the Note tab is told here so a selection switch it held can go ahead.
	if _dock_host != null and not _is_dirty():
		dock_saved.emit.call_deferred()


## Load the version on disk into the editor as ONE undo step (so Ctrl+Z brings the user's text
## back), and treat it as saved. Used by Take theirs and Keep both.
func _take_theirs() -> void:
	var r := bridge.vault_read_file_for_edit(_browse_path)
	if not bool(r.get("ok", false)):
		_save_block = {"kind": String(r.get("kind", "io")), "error": String(r.get("error", ""))}
		_show_block()
		return
	var raw := String(r.get("text", ""))
	var theirs := _norm_eol(raw)
	_checkpoint_history()
	_eol_crlf = raw.contains("\r\n")
	_editor.replace_whole_text(theirs)
	_browse_text = theirs
	_browse_saved = theirs
	_browse_hash = String(r.get("hash", ""))
	_save_block = {}
	_autosave_timer.stop()
	_saved_stamp = Time.get_time_string_from_system()
	_checkpoint_history()
	_editor.hide_banner()
	_editor.mark_saved(theirs, _saved_stamp)


## A name for the copy Keep both makes, beside the note: `Name (my edits 2026-11-08 14-03-22).md`,
## with ` 2`, ` 3` ... if that exists too. `""` when ten names are all taken (never overwrite).
func _conflict_copy_rel(rel: String) -> String:
	var stamp := Time.get_datetime_string_from_system().replace("T", " ").replace(":", "-")
	var dir := rel.get_base_dir()
	var base := rel.get_file().get_basename()
	for n in range(1, 11):
		var nm := "%s (my edits %s%s).md" % [base, stamp, "" if n == 1 else " %d" % n]
		var cand := nm if dir == "" else "%s/%s" % [dir, nm]
		if not _file_exists(cand):
			return cand
	return ""


## Whether `rel` is a note in the vault (an empty note reads as `""`, so a read alone cannot tell).
func _file_exists(rel: String) -> bool:
	var r := bridge.vault_read_file_for_edit(rel)
	return bool(r.get("ok", false))


# -- The session undo (VE-4, Ruling DY VQ1) -------------------------------------

## A note was opened from disk: start its history with the text as first opened this session, or,
## on a later open, add the text now on disk as a checkpoint if it differs from where the history
## left off (the note changed outside Cartalith meanwhile, so Ctrl+Z steps back to ours).
func _note_opened(rel: String, text: String) -> void:
	var h: Array = _hist.get(rel, [])
	if h.is_empty():
		_hist[rel] = [text]
	elif String(h.back()) != text:
		h.append(text)
		_trim_history(h)


## Append the live text to the open note's history when it differs from the last checkpoint.
func _checkpoint_history() -> void:
	if _browse_path == "":
		return
	var cur := _live_text()
	var h: Array = _hist.get(_browse_path, [])
	if h.is_empty():
		h = [cur]
	elif String(h.back()) != cur:
		h.append(cur)
		_trim_history(h)
	_hist[_browse_path] = h


## Keep the history inside its caps by dropping the oldest checkpoint AFTER the first-opened one.
func _trim_history(h: Array) -> void:
	var chars := 0
	for t in h:
		chars += String(t).length()
	while h.size() > 2 and (h.size() > HIST_MAX_CHECKPOINTS or chars > HIST_MAX_CHARS):
		chars -= String(h[1]).length()
		h.remove_at(1)


## The note moved from `old_rel` to `new_rel`: its history goes with it (VE-6 calls this after a
## rename; the proposal says the session undo "follows a rename").
func history_follow_rename(old_rel: String, new_rel: String) -> void:
	if _hist.has(old_rel):
		_hist[new_rel] = _hist[old_rel]
		_hist.erase(old_rel)
	if _browse_path == old_rel:
		_browse_path = new_rel


# -- Rename (VE-6, Ruling DY VQ3) -----------------------------------------------

## The last rename that moved a note: `{old, new, written: [{path, links}], failed: [{path, error}]}`, or
## `{}`. Memory only. It lets the Properties tab offer the undo and lets a probe read the outcome. Cleared
## when the vault is connected or disconnected (the paths would mean another vault).
var _rename_last: Dictionary = {}
## The last sentence the rename flow told the person (status line or report), kept for the probe.
var _rename_message := ""


## Says `msg` on the status line and keeps it as `_rename_message`.
func _rename_say(msg: String, accent: bool = true) -> void:
	_rename_message = msg
	app.set_status("hint", msg, "accent" if accent else "text")


## What the person typed, as a vault path: `\` becomes `/`, and `.md` is added unless it is already the end.
## A folder in the name moves the note there. The crate refuses every name that is not a safe note path.
static func rename_target(typed: String) -> String:
	var t := typed.strip_edges().replace("\\", "/")
	return t if t.to_lower().ends_with(".md") else t + ".md"


## Properties > Rename...: asks for the new name, then plans (`_rename_plan`). The open note is saved first
## through the one writer (`_write_now`), so the plan reads what is on disk and no unsaved typing can be
## lost or overwritten by the move. Only the window's editor renames (the docked Note tab has no side
## panel; its open-in-window button brings the note here).
func _request_rename() -> void:
	if _browse_path == "" or _editor == null or not is_instance_valid(_editor) or _editor_path != _browse_path or _dock_host != null:
		return
	if not _write_now():
		_rename_say("Not renamed: this note is not saved yet. Resolve the message above first.")
		return
	var old := _browse_path
	DccWidgets.prompt(PopoutHost.dialog_host(app, self), "Rename note", "New name (a folder in it moves the note)", old.trim_suffix(".md"), "Rename",
		func(typed: String): _rename_plan(old, rename_target(typed), false),
		"Links to this note in your other notes are rewritten. You are told how many first.")


## Plans renaming `old` to `new_rel` (the crate's `vault_rename_plan`, which writes nothing) and asks the
## person to confirm with the count. `undo` only changes the wording. A refusal (name taken, unsafe path,
## a name Windows cannot hold) is said and nothing else happens. The note must still be the open one and
## saved; the plan's hash covers it and every note the plan would write.
func _rename_plan(old: String, new_rel: String, undo: bool) -> void:
	if _browse_path != old or _editor_path != _browse_path:
		_rename_say("Not renamed: the note on screen is no longer %s." % old)
		return
	if not _write_now():
		_rename_say("Not renamed: this note is not saved yet. Resolve the message above first.")
		return
	var plan := bridge.vault_rename_plan(old, new_rel)
	if not bool(plan.get("ok", false)):
		_rename_say("Not renamed: %s." % String(plan.get("error", "refused")).trim_suffix("."))
		return
	var locked := _rename_unwritable(plan)
	var verb := "Undo rename" if undo else "Rename"
	DccWidgets.confirm(PopoutHost.dialog_host(app, self), verb if undo else "Rename note", _rename_describe(old, new_rel, plan, locked), verb,
		func(): _rename_run(old, new_rel, plan, undo), _refocus_editor.call_deferred, 460)


## The notes the plan would write that are read-only on this device, found BEFORE anything moves so the
## confirmation can say so. Best effort and only for a folder vault on a desktop (an Android tree URI has no
## file attribute to read): the authoritative answer is the crate's own refusal after the move, which
## `_rename_run` reports in any case.
func _rename_unwritable(plan: Dictionary) -> Array:
	var out: Array = []
	var root := String(bridge.vault_info().get("root", ""))
	if root == "" or root.contains("://"):
		return out
	for f in plan.get("files", []):
		var rel := String((f as Dictionary).get("path", ""))
		var abs := root.path_join(rel)
		if FileAccess.file_exists(abs) and FileAccess.get_read_only_attribute(abs):
			out.append(rel)
	return out


## The confirmation's text: what moves, how many links change in which notes, what is left alone, what
## could not be checked, and which notes cannot be written. Every number is the crate's, never counted here.
func _rename_describe(old: String, new_rel: String, plan: Dictionary, locked: Array) -> String:
	var lines := PackedStringArray()
	lines.append("%s will become %s." % [old, new_rel])
	var others: Array = []
	var self_links := 0
	for f in plan.get("files", []):
		var fd: Dictionary = f
		if String(fd.get("path", "")) == old:
			self_links = int(fd.get("links", 0))
		else:
			others.append(fd)
	var other_links := int(plan.get("links", 0)) - self_links
	if others.is_empty() and self_links == 0:
		lines.append("No link in any note names it, so no link changes.")
	else:
		lines.append("%d link%s in %d other note%s will be rewritten%s:" % [
			other_links, "" if other_links == 1 else "s", others.size(), "" if others.size() == 1 else "s",
			" (and %d in this note itself)" % self_links if self_links > 0 else ""])
		for i in mini(others.size(), 8):
			lines.append("   %s (%d)" % [String((others[i] as Dictionary).get("path", "")), int((others[i] as Dictionary).get("links", 0))])
		if others.size() > 8:
			lines.append("   and %d more" % (others.size() - 8))
	var amb: Array = plan.get("ambiguous", [])
	if not amb.is_empty():
		lines.append("%d link%s could mean more than one note and will be left alone:" % [amb.size(), "" if amb.size() == 1 else "s"])
		for i in mini(amb.size(), 4):
			var a: Dictionary = amb[i]
			lines.append("   %s, line %d: [[%s]]" % [String(a.get("path", "")), int(a.get("line", 0)) + 1, String(a.get("target", ""))])
	if int(plan.get("unreadable", 0)) > 0:
		lines.append("%d note%s could not be opened, so their links were NOT checked." % [int(plan["unreadable"]), "" if int(plan["unreadable"]) == 1 else "s"])
	if int(plan.get("knowledge_links", 0)) > 0:
		lines.append("%d link%s from the map follow the note." % [int(plan["knowledge_links"]), "" if int(plan["knowledge_links"]) == 1 else "s"])
	if not locked.is_empty():
		lines.append("READ-ONLY, so the links in them cannot be rewritten and will still name the old name: %s. The note still moves, and you are told afterwards. Make them writable first to avoid that." % ", ".join(PackedStringArray(locked)))
	lines.append("If any of these notes changes on disk before you confirm, nothing is written and you are asked again. A note that cannot be written is never forced: it is listed afterwards, and the rename can be undone.")
	return "\n".join(lines)


## Runs the confirmed rename through the crate (`vault_rename`): the note moves first, then each rewrite goes
## through the hash guard, so a note that changed or is read-only is left as it is and listed. **The
## contract, stated once:** nothing is overwritten and nothing is forced; the move and the rewrites are NOT
## one atomic step, so a rewrite that is refused leaves that note's links naming the old name and the report
## says exactly which; the rename is undoable (`_undo_rename`), and the undo is a rename back, not a
## byte-restore of the files. A plan whose notes changed since the confirmation is refused whole and planned
## again.
func _rename_run(old: String, new_rel: String, plan: Dictionary, undo: bool) -> void:
	if _browse_path != old or _editor_path != _browse_path or not _write_now():
		_rename_say("Not renamed: the note changed while you were deciding. Nothing was moved.")
		return
	var r := bridge.vault_rename(old, new_rel, String(plan.get("hash", "")))
	if not bool(r.get("ok", false)):
		if bool(r.get("conflict", false)):
			_rename_say("A note changed while you were deciding, so nothing was written. Here is the plan again.")
			_rename_plan.call_deferred(old, new_rel, undo)
		else:
			_rename_say("Not renamed: %s." % String(r.get("error", "refused")).trim_suffix("."))
		return
	var failed: Array = r.get("failed", [])
	_rename_last = {} if undo else {"old": old, "new": new_rel, "written": r.get("written", []), "failed": failed}
	_rename_follow(old, new_rel)
	store_changed.emit()
	var rewritten := int(r.get("links", 0))
	var wrote: Array = r.get("written", [])
	if failed.is_empty():
		_rename_say("%s %s to %s; %d link%s in %d note%s rewritten." % [
			"Renamed back" if undo else "Renamed", old, new_rel, rewritten, "" if rewritten == 1 else "s", wrote.size(), "" if wrote.size() == 1 else "s"], false)
		return
	var lines := PackedStringArray()
	lines.append("%s is now %s, and %d link%s in %d note%s %s rewritten." % [old, new_rel, rewritten, "" if rewritten == 1 else "s", wrote.size(), "" if wrote.size() == 1 else "s", "was" if wrote.size() == 1 else "were"])
	lines.append("NOT rewritten (they still name the old name until you fix or undo):")
	for f in failed:
		lines.append("   %s: %s" % [String((f as Dictionary).get("path", "")), String((f as Dictionary).get("error", ""))])
	_rename_message = "\n".join(lines)
	app.set_status("hint", "Renamed %s, but %d note%s could not be updated." % [new_rel, failed.size(), "" if failed.size() == 1 else "s"], "accent")
	var dlg := DccWidgets.confirm(PopoutHost.dialog_host(app, self), "Renamed, with problems", _rename_message,
		"Undo the rename" if not undo else "OK", func():
			if not undo:
				_undo_rename()
			else:
				_refocus_editor.call_deferred(), _refocus_editor.call_deferred, 460)
	if dlg != null and not undo:
		dlg.cancel_button_text = "Keep it renamed"


## The open note was just moved from `old` to `new_rel` by `vault_rename`. This window's state follows
## it so there is still ONE note and ONE writer: the session undo and the Back trail are re-keyed
## (`history_follow_rename`), the text and hash are read again (a note that links to itself was rewritten),
## and the editor is rebuilt from that state with its undo replayed, on the tab the person was on.
## `_editor_path` is cleared first so the old editor, alive until the rebuild, is never read as the new note.
func _rename_follow(old: String, new_rel: String) -> void:
	var tab_was := String(_side_panel.tab) if _side_panel != null and is_instance_valid(_side_panel) else ""
	history_follow_rename(old, new_rel)
	_pick_file = new_rel
	for i in _link_back.size():
		if String(_link_back[i]) == old:
			_link_back[i] = new_rel
	_editor_path = ""
	var rr := bridge.vault_read_file_for_edit(new_rel)
	if bool(rr.get("ok", false)):
		var raw := String(rr.get("text", ""))
		_eol_crlf = raw.contains("\r\n")
		_browse_text = _norm_eol(raw)
		_browse_saved = _browse_text
		_browse_hash = String(rr.get("hash", ""))
		_save_block = {}
		_checkpoint_history()
	_rebuild()
	if tab_was != "" and _side_panel != null and is_instance_valid(_side_panel):
		_side_panel.show_tab(tab_was)
	_refocus_editor.call_deferred()


## Undoes the last rename: renames the note back through the same plan, confirmation and guard. It is a
## rename, so it puts every link that now names the new name back to naming the old one, and a note whose
## rewrite had been refused (it still names the old name) is simply right again; it is NOT a byte-restore
## of every file, and it is refused if the old name has been taken since.
func _undo_rename() -> void:
	if _rename_last.is_empty():
		return
	if _browse_path != String(_rename_last.get("new", "")):
		_rename_say("Open %s to undo its rename." % String(_rename_last.get("new", "")))
		return
	_rename_plan(_browse_path, String(_rename_last.get("old", "")), true)


## Keyboard focus back to the text, after a dialog that promised "Ctrl+Z brings it back" closed. Two
## steps, both needed: an embedded dialog that closes leaves NO embedded window holding the keyboard (a
## measured fact: with the vault window visible, typed keys and Ctrl+Z reached nothing until the window
## was given focus again), so the window takes it back first and then the text. Deferred, because the
## dialog frees itself at the end of the frame the confirm ran in.
func _refocus_editor() -> void:
	if _browse_edit == null or not is_instance_valid(_browse_edit):
		return
	if _dock_host != null:
		## Docked (PC-5): the editor lives in the main window's tree, which is the one holding the keyboard.
		_browse_edit.grab_focus()
		return
	if not visible:
		return
	grab_focus()
	_browse_edit.grab_focus()


## Revert to opened… (Ruling DY, VQ6): asks first, says what is lost, and is one undo step. The
## write that follows goes through the same hash guard as any save.
func _on_revert_requested() -> void:
	var h: Array = _hist.get(_browse_path, [])
	if h.is_empty() or _editor == null or _browse_edit == null:
		return
	var first := String(h[0])
	if _live_text() == first:
		app.set_status("hint", "%s is already as it was when you first opened it." % _browse_path.get_file(), "text_ghost")
		return
	DccWidgets.confirm(PopoutHost.dialog_host(app, self), "Revert to opened",
		"%s will go back to how it was when you first opened it this session. Everything you changed since is replaced. Ctrl+Z brings it back." % _browse_path.get_file(),
		"Revert", func():
			if _editor == null or _browse_edit == null or not is_instance_valid(_browse_edit):
				return
			_editor.replace_whole_text(first)
			_browse_text = first
			_write_now()
			_refocus_editor.call_deferred())


## A close with unsaved edits: the window comes straight back and asks. Every
## way out of an `AcceptDialog` -- Close, the title bar's X, Escape -- ends in
## a hide, so this one handler covers all of them.
func _on_visibility_changed() -> void:
	if visible:
		return
	## VE-4: closing the window saves first; the prompt below appears only if that save was refused.
	_write_now()
	if not _is_dirty():
		return
	_warn_unsaved.call_deferred()


## Deferred half of `_on_visibility_changed`: if the window was hidden while
## holding unsaved edits, brings it back (unless a phone sheet is present) and
## asks whether to discard them. Discard drops the edits and hides; Cancel leaves
## the window open with the edits intact. Deferred so it runs after the hide that
## triggered it has finished.
func _warn_unsaved() -> void:
	if visible or not _is_dirty():
		return
	if not DccWidgets.phone_present(self, app):
		popup()
	DccWidgets.confirm(PopoutHost.dialog_host(app, self), "Unsaved edits",
		"%s has edits that are not saved. Close anyway and discard them?" % _browse_path.get_file(),
		"Discard and close", func():
			_close_editor()
			hide())


# -- More ------------------------------------------------------------------------

## Screenshot 3's advanced sections, folded (Ruling BE). They act on ONE link
## -- the entity whose block, fields, working copy and snapshot these are --
## chosen here when the note carries several.
func _build_more(parent: Control) -> void:
	var more := DccWidgets.group(parent, "More", _more_open)
	more.name = "VaultMore"
	var hdr := parent.get_child(parent.get_child_count() - 2) as Button
	if hdr != null:
		hdr.name = "VaultMoreHeader"
		## Connected after `group()`'s own toggle, so it reads the new state.
		hdr.pressed.connect(func(): _more_open = more.visible)

	var links := _links_for_file(_pick_file)
	if not links.is_empty():
		var active := {}
		for l in links:
			if String((l as Dictionary).get("link_id", "")) == _reader_link:
				active = l
		if active.is_empty():
			for l in links:
				var d: Dictionary = l
				if String(d.get("entity_kind", "")) == _kind and int(d.get("entity_id", 0)) == _entity_id:
					active = d
		if active.is_empty():
			active = links[0]
		_reader_link = String(active.get("link_id", ""))
		_lk = String(active.get("entity_kind", ""))
		_lid = int(active.get("entity_id", 0))
		_llabel = String(active.get("entity_label", ""))
		if links.size() > 1:
			var labels: Array = []
			var ids: Array = []
			for l in links:
				var d: Dictionary = l
				labels.append("%s (%s) — %s" % [String(d.get("entity_label", "")),
					String(d.get("entity_kind", "")), String(d.get("selection_label", ""))])
				ids.append(String(d.get("link_id", "")))
			DccWidgets.choice(more, "Acts on", labels, maxi(0, ids.find(_reader_link)),
				func(i: int):
					_reader_link = String(ids[i])
					_rebuild(),
				"This note is attached to more than one entity; everything in More acts on the link picked here.")
		DccWidgets.note(more, "Acts on the link to %s (%s)." % [_llabel, _lk])
		_build_reader(more)
		_build_entity_data(more)
		_build_snapshots(more)
		_build_feedback(more)
	else:
		DccWidgets.note(more, "The working copy, the Cartalith block, field fill and section insert act on a link -- attach this note to an entity (Attached to, above) to reach them.")
		## A snapshot is a picture of the place, not of a note, so an entity
		## view offers it with no note attached.
		if _kind != "":
			_build_snapshots(more)
	if _kind != "" and _pick_file != "":
		_build_create(more)
	_build_write_prefs(more)
	_build_index(more)
	var vs := DccWidgets.section(more, "Vault")
	var dis := DccWidgets.action(vs, "Disconnect", func():
		bridge.vault_disconnect()
		_hist.clear()
		_img_cache.clear()
		_rename_last = {}
		store_changed.emit()
		_rebuild())
	dis.tooltip_text = "Drops this device's binding. The links themselves survive — that is the difference between disconnecting and detaching."


## Clears and rebuilds only the tree column's tree — not the search field
## beside it, and not the rest of the window — so `_build_browse_search()`'s
## field survives its own `text_changed` signal. `null` (no tree pane on
## screen: the phone's preview pane, or a state before the first `_rebuild()`)
## is a no-op, not an error: `_clear()` nulls this on every rebuild and the
## phone's preview pane never sets it.
func _refresh_browse_tree() -> void:
	if _browse_tree_col == null or not is_instance_valid(_browse_tree_col):
		return
	for c in _browse_tree_col.get_children():
		_browse_tree_col.remove_child(c)
		c.queue_free()
	_build_browse_tree(_browse_tree_col, bridge.vault_list_files())
	if _phone:
		app.phone_fit(_browse_tree_col, 1.0)


## The phone fold: the now-retired (FH-2) `culture_profiles_window.gd::_build_phone_switcher()`'s
## segmented-row look, wired through `_rebuild()` rather than persistent-pane
## visibility — this file rebuilds `_body` from scratch on every state change
## already (`_search_open_rel`, `_attach_open`, `_browse_path`, …), so a third
## toggle following the same idiom is the small addition, not a second
## show/hide mechanism living beside it.
func _build_browse_phone_switcher(parent: Control) -> void:
	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 0)
	parent.add_child(row)
	## "NOTE", not "PREVIEW", since Ruling BE: the pane holds the note's
	## attachments and its editor as well as the read-only view.
	for spec in [["tree", "FILES"], ["preview", "NOTE"]]:
		var key := String(spec[0])
		var b := Button.new()
		b.text = String(spec[1])
		b.focus_mode = Control.FOCUS_NONE
		b.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		b.add_theme_font_override("font", DccTheme.mono(0))
		b.add_theme_font_size_override("font_size", DccTheme.FS_MICRO)
		var on := _browse_phone_pane == key
		b.add_theme_color_override("font_color", DccTheme.c("accent") if on else DccTheme.c("text_dim"))
		b.pressed.connect(func():
			_browse_phone_pane = key
			_rebuild())
		row.add_child(b)


## The folder tree. Built fresh on every `_rebuild()` from `vault_list_files`'s
## flat, `/`-separated paths — no new bridge call, and no state kept across
## rebuilds beyond `_pick_file` itself, which is what a picked item writes to.
##
## Filtered by `_search_query` (case-insensitive substring on the whole
## relative path) rather than drawn and then hidden node-by-node: a filtered
## build is also what keeps a folder that holds no matching file out of the
## tree entirely, instead of an empty-looking row a post-hoc `visible = false`
## walk would have to reason about separately.
func _build_browse_tree(parent: Control, files: PackedStringArray) -> void:
	var q := _search_query.strip_edges().to_lower()
	var shown: Array = []
	for f in files:
		if q == "" or String(f).to_lower().find(q) >= 0:
			shown.append(String(f))

	var tree := Tree.new()
	tree.hide_root = true
	tree.custom_minimum_size = Vector2(200, 260)
	tree.size_flags_vertical = Control.SIZE_EXPAND_FILL
	## The shell's only `Tree`, and the project theme has no `Tree` entries to
	## remap -- so it drew Godot's stock grey slab, which on the light palette
	## is a mid-grey block in a paper-coloured window. Tokens instead: an input
	## well (`sunken` + `border`), body ink, and the accent wash every active
	## row in this shell already uses for "selected".
	var well := DccTheme.panel("sunken", {"left": 1, "right": 1, "top": 1, "bottom": 1})
	well.border_color = DccTheme.c("border")
	well.content_margin_left = 4
	well.content_margin_top = 4
	tree.add_theme_stylebox_override("panel", well)
	tree.add_theme_stylebox_override("focus", DccTheme.empty())
	var sel := DccTheme.flat(DccTheme.c("accent_wash"))
	for box in ["selected", "selected_focus", "hovered", "hovered_selected", "hovered_selected_focus"]:
		tree.add_theme_stylebox_override(box, sel)
	for box in ["cursor", "cursor_unfocused"]:
		tree.add_theme_stylebox_override(box, DccTheme.empty())
	tree.add_theme_color_override("font_color", DccTheme.c("text"))
	tree.add_theme_color_override("font_hovered_color", DccTheme.c("text_bright"))
	tree.add_theme_color_override("font_selected_color", DccTheme.c("text_bright"))
	tree.add_theme_color_override("guide_color", Color(0, 0, 0, 0))
	tree.add_theme_font_size_override("font_size", DccTheme.FS_BODY)
	## Ruling BE's link marker: an accent dot on every note the link store
	## holds a link for, with the entities named in its tooltip. The mockup's
	## own file dot, spent on the one fact about a file this window adds.
	var linked := {}   # rel -> Array of "Label (kind)"
	for l in bridge.vault_all_links():
		var ld: Dictionary = l
		var lrel := String(ld.get("path", ""))
		if not linked.has(lrel):
			linked[lrel] = []
		(linked[lrel] as Array).append("%s (%s)" % [String(ld.get("entity_label", "")), String(ld.get("entity_kind", ""))])
	var dot: Texture2D = DccWidgets._round_dot(8, DccTheme.c("accent")) if not linked.is_empty() else null
	var root := tree.create_item()
	var folders := {"": root}    # folder path ("" = root) -> TreeItem
	var leaves: Array = []       # [TreeItem, rel] for the private badge (Ruling DP, VE-9)
	for rel in shown:
		var parts := String(rel).split("/")
		var cur_path := ""
		var cur_item: TreeItem = root
		for i in range(parts.size() - 1):
			cur_path = String(parts[i]) if cur_path == "" else cur_path + "/" + String(parts[i])
			if not folders.has(cur_path):
				var fi := tree.create_item(cur_item)
				fi.set_text(0, String(parts[i]))
				fi.set_selectable(0, false)
				folders[cur_path] = fi
			cur_item = folders[cur_path]
		var leaf := tree.create_item(cur_item)
		leaf.set_text(0, String(parts[parts.size() - 1]))
		leaf.set_metadata(0, rel)
		leaves.append([leaf, rel])
		if linked.has(rel):
			leaf.set_icon(0, dot)
			leaf.set_tooltip_text(0, "Attached to %s" % ", ".join(PackedStringArray(linked[rel])))
		if rel == _pick_file:
			leaf.select(0)
	tree.item_selected.connect(func():
		var it := tree.get_selected()
		if it == null:
			return
		var rel := String(it.get_metadata(0))
		if rel == "" or rel == _pick_file:
			return
		## Synchronous, as it always was: the rebuild frees this Tree with
		## `queue_free()`, which waits for the signal to return.
		_select_file(rel))
	parent.add_child(tree)
	_badge_tree(leaves)
	if shown.is_empty():
		DccWidgets.note(parent, "No file matches \"%s\"." % _search_query.strip_edges())


## Drops a leading YAML frontmatter block (`---` … `---`) before the excerpt
## is taken from it. `vault_file_data` already parses that block into its own
## `frontmatter` map, shown as chips above — without this, the excerpt's
## first lines were the fence and the raw `key: value` pairs a second time,
## which is not the *body* the mockup asks for.
func _strip_frontmatter(text: String) -> String:
	if not text.begins_with("---"):
		return text
	var lines := text.split("\n")
	if lines.is_empty() or String(lines[0]).strip_edges() != "---":
		return text
	for i in range(1, lines.size()):
		if String(lines[i]).strip_edges() == "---":
			return "\n".join(PackedStringArray(lines.slice(i + 1)))
	return text


## First `n` non-blank lines of `text`, joined with a space — the excerpt's
## own trim. Deliberately not a character-count truncation: a heading or a
## blank line at the very top of a note would otherwise dominate a fixed
## character budget with nothing readable in it.
##
## ATX heading markers (`#` through `######`, then a space) are stripped per
## line before joining (`OUTSTANDING_WORK.md`'s Vault Browser row: "the
## excerpt shows raw `## ` markers" against the mockup, which draws prose
## only). The outline above already shows headings as headings; the excerpt
## is meant to read as the note's own prose, so a heading line that happens
## to fall in the first `n` lines reads as text, not as a second, markerless
## copy of the outline.
func _first_lines(text: String, n: int) -> String:
	var out: Array = []
	for raw in text.split("\n"):
		var t := String(raw).strip_edges()
		if t == "":
			continue
		out.append(_strip_heading_marker(t))
		if out.size() >= n:
			break
	return " ".join(PackedStringArray(out))


## `"## The Old Quarter"` -> `"The Old Quarter"`. Only a real ATX marker (1-6
## `#` then a space) is stripped, so a line that merely starts with `#` for
## some other reason (a hashtag in prose) is left alone.
func _strip_heading_marker(line: String) -> String:
	var i := 0
	while i < line.length() and i < 6 and line[i] == "#":
		i += 1
	if i > 0 and i < line.length() and line[i] == " ":
		return line.substr(i + 1).strip_edges()
	return line


## The note itself, read-only: frontmatter (and the author's filled-in
## template fields) as chips, the heading outline beside a short excerpt that
## keeps headings out, **Open to edit** and Centre on map, the backlinks line,
## and then the whole note rendered (`markdown_editor.gd::to_bbcode`, the same
## renderer the editor's Preview uses). Ruling BE asked for a rendered note;
## the 2026-09-21 pass had kept this a structured summary only.
##
## **Backlinks/unlinked mentions, closed 2026-09-26** (`OUTSTANDING_WORK.md`
## "Record approved Vault Browser mockup + close its remaining gaps").
## `vault_entity_backlinks`/`vault_entity_mentions` are keyed by `(kind,
## entity_id)`, which a browsed, unattached file has neither of — the gap this
## header used to describe. `vault_file_backlinks`/`vault_file_mentions`
## (`vault_bridge.rs`) are the path-keyed wrappers over
## `cartalith_vault::backlinks::Backlinks::backlinks_to` and the new
## `cartalith_vault::VaultSession::file_mentions`, both drawn below. Same
## `built` gate as the Index panel (`_build_index`): with no backlink index,
## these say so rather than reporting a false zero.
##
## **"Centre on map"** (`design/vault-browser-2026-09-21/Cartalith Vault
## Browser.dc.html`, beside "Open to edit") reuses the one camera call every
## other window in this shell already reaches through --
## `app.viewport.move_view_to(gx, gy)` (`place_editor_window.gd`'s "Focus
## camera on settlement", `faction_roster_window.gd`'s "Focus camera on
## capital", `global_tools.gd`, `place_search.gd`) -- rather than inventing a
## second one. There is no entity scope here to read a position off (a
## browsed file is not `open_for()`'d), so `_build_browse_centre` derives one
## from the note's own frontmatter (`type: settlement`, `tid: N`) and
## `bridge.settlements()`, the same list `faction_roster_window.gd::
## _capital_of` already filters client-side. **Disabled with the reason**,
## never invented, when the frontmatter names no settlement or that tid is
## not in the currently generated world.
func _build_browse_preview(parent: Control, data: Dictionary) -> void:
	var g := DccWidgets.group(parent, _pick_file.get_file(), true)
	g.name = "VaultNote"

	var frontmatter: Dictionary = {}
	if not bool(data.get("ok", false)):
		DccWidgets.note(g, "Could not read this note: %s" % String(data.get("error", "")))
	else:
		frontmatter = data.get("frontmatter", {})
		if frontmatter.is_empty():
			DccWidgets.note(g, "No frontmatter.")
		else:
			var flow := HFlowContainer.new()
			flow.add_theme_constant_override("h_separation", 6)
			flow.add_theme_constant_override("v_separation", 6)
			for k in frontmatter:
				## Display-only chip: `DccWidgets.chip`'s own `on_press` guards
				## the connect with `is_valid()`, so an empty `Callable()` here
				## leaves the chip inert rather than silently wired to nothing.
				DccWidgets.chip(flow, "%s: %s" % [String(k), String(frontmatter[k])], Callable())
			g.add_child(flow)
		## The author's filled-in template fields (`**Type:** City`), the other
		## half of what the removed "What does this note hold?" readout showed.
		## A second chip row and never merged with the first: the two can
		## disagree, and deciding between them is the author's.
		var fields: Dictionary = data.get("fields", {})
		if not fields.is_empty():
			var fflow := HFlowContainer.new()
			fflow.name = "VaultFieldChips"
			fflow.add_theme_constant_override("h_separation", 6)
			fflow.add_theme_constant_override("v_separation", 6)
			for k in fields:
				DccWidgets.chip(fflow, "%s: %s" % [String(k), String(fields[k])], Callable())
			g.add_child(fflow)

	var headings := bridge.vault_file_headings(_pick_file)
	var read := bridge.vault_read_file_for_edit(_pick_file)

	## Outline and excerpt side by side (owner-approved mockup: `grid-
	## template-columns:220px 1fr`), stacked on phone — the same
	## `BoxContainer`-axis-swap idiom the now-retired (FH-2) `culture_profiles_window.gd::
	## _build_body()` already uses for its own desktop/phone fold
	## (`VBoxContainer.new() if _phone else HBoxContainer.new()`), so one call
	## site builds both instead of a separate phone branch duplicating the
	## content. Named nodes (`VaultOutlineCol`/`VaultExcerptCol`) so a probe
	## can measure their rects directly rather than guessing which `Control`
	## is which from the tree shape.
	var cols: BoxContainer = VBoxContainer.new() if _one_pane() else HBoxContainer.new()
	cols.name = "VaultOutlineExcerptRow"
	cols.add_theme_constant_override("separation", 12 if _one_pane() else 24)
	cols.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	g.add_child(cols)

	var outline_col := VBoxContainer.new()
	outline_col.name = "VaultOutlineCol"
	if not _one_pane():
		outline_col.custom_minimum_size.x = 180
		outline_col.size_flags_horizontal = Control.SIZE_FILL
	else:
		outline_col.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	cols.add_child(outline_col)
	DccWidgets.note(outline_col, "Outline")
	## The mockup draws the heading list itself at `font:12px` (`ENV:99`), not
	## the section label's `10.5px` -- `note()`'s own default (`FS_MICRO`, 9)
	## is that caption size, right for "Outline" above but too small for the
	## rows read as prose here. `_vaultfont_probe`-equivalent: measured live
	## against the mockup by `_vaultlayout_probe.gd`'s font-size block.
	if headings.is_empty():
		DccWidgets.note(outline_col, "    (no headings)") \
			.add_theme_font_size_override("font_size", DccTheme.FS_BODY)
	else:
		for h in headings:
			var d: Dictionary = h
			var lvl := int(d.get("level", 1))
			DccWidgets.note(outline_col, "    %s%s" % ["  ".repeat(maxi(0, lvl - 1)), String(d.get("title", ""))]) \
				.add_theme_font_size_override("font_size", DccTheme.FS_BODY)

	var excerpt_col := VBoxContainer.new()
	excerpt_col.name = "VaultExcerptCol"
	excerpt_col.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	cols.add_child(excerpt_col)
	## The mockup's excerpt body is `font:13.5px/1.7 'Inter'` (`ENV:106`) --
	## prose, not a caption. Godot sizes are integers and this codebase's own
	## convention (`dcc_theme.gd`'s `fs_shortcut`/`fs_timeline` comments) rounds
	## a fractional mockup figure down to the nearest token rather than
	## inventing a literal, so `FS_BODY` (12) is the closest existing
	## `DccTheme` token below 13.5 -- the same one `DccWidgets.modal_prose()`
	## already reads prose in elsewhere.
	##
	## **Shown only when the read failed (`OUTSTANDING_WORK.md`'s Vault editor
	## polish row).** The 2026-09-21 mockup drew outline + excerpt because it
	## predates Ruling BE, which is what added the whole rendered note below
	## (`ENV:106` never anticipated that section existing at all). Once the
	## read succeeds, the full render a few rows down already carries this
	## excerpt's exact opening sentence -- same words, same order -- so the
	## excerpt stopped being a preview of something not yet shown and became a
	## second, plainer copy of the first few lines of the thing directly
	## beneath it. The mockup's *outline* column is unaffected and still
	## always drawn; only the now-redundant excerpt text is dropped. A failed
	## read has no full render to fall back on, so the excerpt (as "Could not
	## read: …") still does real work there and stays.
	if not bool(read.get("ok", false)):
		DccWidgets.note(excerpt_col, "Excerpt")
		DccWidgets.note(excerpt_col, "    Could not read: %s" % String(read.get("error", ""))) \
			.add_theme_font_size_override("font_size", DccTheme.FS_BODY)

	## Mockup order: the two buttons (Open to edit, Centre on map) directly
	## under the excerpt, then the backlinks/mentions line beneath them — all
	## in the excerpt column, matching the mockup's own right-hand stack.
	var open_btn := DccWidgets.action(excerpt_col, "Open to edit", _open_editor, true)
	open_btn.tooltip_text = "Turns this pane into the Markdown editor. Reads the whole file; Save writes it back and refuses if it changed on disk since this read."
	_build_browse_centre(excerpt_col, frontmatter)
	_build_browse_backlinks(excerpt_col)

	## The note, rendered. `border-left:2px` + `padding-left:14px` is the
	## mockup's own excerpt treatment (`ENV:106`), reused for the full body.
	if bool(read.get("ok", false)):
		DccWidgets.note(g, "Note")
		var body := RichTextLabel.new()
		body.name = "VaultNoteRendered"
		body.bbcode_enabled = true
		body.fit_content = true
		body.selection_enabled = true
		body.scroll_active = false
		body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		MdEditor.style_rendered(body)
		var edge := StyleBoxFlat.new()
		edge.bg_color = Color(0, 0, 0, 0)
		edge.border_width_left = 2
		edge.border_color = DccTheme.c("border")
		edge.content_margin_left = 14
		edge.content_margin_top = 4
		edge.content_margin_bottom = 4
		body.add_theme_stylebox_override("normal", edge)
		## Links (VE-1): classed by kind and followable, like the editor's Preview.
		var from_rel := _pick_file
		body.meta_underlined = false
		body.text = MdEditor.to_bbcode(MdEditor.strip_frontmatter(String(read.get("text", ""))), "", "",
			func(t: String) -> Dictionary: return _link_kind(from_rel, t))
		body.meta_clicked.connect(func(meta: Variant):
			var m := str(meta)
			if m.begins_with("wiki:"):
				_follow_link(from_rel, m.substr(5), body)
			elif m.begins_with("ext:"):
				_open_external(m.substr(4)))
		g.add_child(body)


## `bridge.settlements()` is the whole vault-side of the mockup's "Centre on
## map" -- every field it returns (`_capital_of` above), read here instead of
## invented: a browsed file carries no entity scope of its own, so the only
## honest position is one this note's own frontmatter names and the current
## world actually has. `app.viewport.move_view_to` is the same call every
## other "Focus/Centre" button in this shell already uses -- see this
## function's own doc comment above `_build_browse_preview` for the roll
## call -- so this adds no new engine surface, only a new caller of it.
func _build_browse_centre(g: Control, frontmatter: Dictionary) -> void:
	var reason := ""
	var target := {}
	var kind_str := String(frontmatter.get("type", ""))
	if kind_str != "settlement":
		reason = "Only wired for a settlement's own note (frontmatter \"type: settlement\") -- this note's frontmatter says \"%s\"." % (kind_str if kind_str != "" else "(no type field)")
	else:
		var tid_str := String(frontmatter.get("tid", ""))
		if not tid_str.is_valid_int():
			reason = "This note's frontmatter has no numeric \"tid\" field to resolve a position from."
		else:
			var tid := int(tid_str)
			for s in bridge.settlements():
				var d: Dictionary = s
				if int(d.get("tid", -1)) == tid:
					target = d
					break
			if target.is_empty():
				reason = "No settlement with tid %d exists in the currently generated world -- the note may describe an earlier one, or none has been generated yet." % tid

	var centre := DccWidgets.action(g, "Centre on map", func():
		app.viewport.move_view_to(float(int(target.get("x", 0))), float(int(target.get("y", 0)))))
	if target.is_empty():
		centre.disabled = true
		centre.tooltip_text = reason
	else:
		centre.tooltip_text = "Moves the map view to %s (%d, %d)." % [String(target.get("name", "")), int(target.get("x", 0)), int(target.get("y", 0))]


## The backlinks/unlinked-mentions readout for a browsed file (see this
## file's own handoff note above `_build_browse_preview`). Gated on the same
## `built` flag `_build_index` reads, so an unbuilt index reads as "not built"
## rather than as a real zero -- `MISTAKES.md`'s "never encode 'no value' as
## a plausible value".
func _build_browse_backlinks(g: Control) -> void:
	var stats := bridge.vault_backlink_stats()
	if not bool(stats.get("built", false)):
		DccWidgets.note(g, "Backlinks: index not built yet -- More ▸ Index builds it.")
		return
	var back := bridge.vault_file_backlinks(_pick_file)
	var mentions := bridge.vault_file_mentions(_pick_file, 12)
	DccWidgets.note(g, "%d backlink%s · %d unlinked mention%s" % [
		back.size(), "" if back.size() == 1 else "s",
		mentions.size(), "" if mentions.size() == 1 else "s"])
	for b in back:
		var bd: Dictionary = b
		DccWidgets.note(g, "    %s → links here (%s)" % [String(bd.get("rel", "")), String(bd.get("form", ""))])
	for m in mentions:
		var md: Dictionary = m
		DccWidgets.note(g, "    %s names it, unlinked · %s" % [String(md.get("rel", "")), String(md.get("excerpt", ""))])


# -- Linked notes (§28) -----------------------------------------------------

## Link status code from the vault store -> the label shown for it, with a leading mark;
## `_build_attached_row` and `_build_links` fall back to the raw code for anything not listed.
const STATUS_TEXT := {
	"connected": "✓ Connected",
	"stale": "● Source changed",
	"local_changes": "● Local changes",
	"cached": "● Cached — source unavailable",
	"missing": "✕ Source missing",
	"unbound": "● Vault not connected on this device",
}


## The scoped form's "Knowledge" section: one group per note attached to the
## scope entity (`_kind`/`_entity_id`), with its `STATUS_TEXT` line and the
## Open/Close, Reload and compare actions for a stale link. Notes show their
## cached text even when the vault is unbound (§27), so this never needs a live
## connection to draw.
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
	var rows := bridge.vault_entity_data(_lk, _lid)
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
	## `phone_window()` FIRST, and this site is why the protocol is worth
	## naming. It had the second and third calls -- `app.phone_fit(dlg, 1.0)`
	## and a bare `popup_centered()` -- and not the first, so the fit ran
	## against a window that had never been shaped for a phone. **That is worse
	## than omitting it**: an absent fit leaves a desktop dialog, a fit without
	## the shaping produces a scaled-up desktop dialog and looks deliberate.
	##
	## It also sets `ok_button_text` to "Close", which is what this dialog
	## already wanted, so the `get_ok_button()` line below it stays correct on
	## both platforms rather than being overwritten on one.
	var phone := DccWidgets.phone_window(dlg, app)
	dlg.get_ok_button().text = "Close"
	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 6)
	dlg.add_child(col)
	## `phone_window()` drops the decoration, so the title has to be drawn
	## inside the content or the dialog opens nameless -- see its own header,
	## "each window carries its own titled header inside the content".
	if phone:
		DccWidgets.phone_head(col, dlg.title, "")
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
	## Ruling EA item 11: a window popped out to another monitor opens its dialogs on that monitor
	## (`PopoutHost.dialog_host`); docked, this is `app` exactly as before.
	var dlg_want := dlg.size   # read BEFORE it joins a pop-out: an embedded window is clamped to its embedder on entry
	var dlg_host := PopoutHost.dialog_host(app, self)
	dlg_host.add_child(dlg)
	if phone:
		app.phone_fit(dlg, 1.0)
	## `phone_present()` is the third call, and it returns false off-phone --
	## so `popup_centered()` is the desktop path rather than a fallback that
	## also fires on a phone and fights the presentation.
	if not DccWidgets.phone_present(dlg, app):
		PopoutHost.present(dlg, dlg_host, dlg_want)


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

## Working-copy section of a linked note: an editable text field backed by
## `bridge.vault_link_text` (committed on focus loss), a Save local copy action, an "Insert updated
## section into source..." action that writes the section back to the vault file after a preview
## and a hash-checked confirmation (`_confirm_section_write`), and the "what this note holds" group.
## Building it writes nothing.
func _build_reader(parent: Control) -> void:
	var sec := DccWidgets.section(parent, "Working copy")
	_reader_edit = TextEdit.new()
	_reader_edit.text = bridge.vault_link_text(_reader_link)
	_reader_edit.custom_minimum_size.y = 200
	_reader_edit.wrap_mode = TextEdit.LINE_WRAPPING_BOUNDARY
	## The shell's field ground, not Godot's stock grey slab (Ruling BE's "no
	## grey box" applies to the working copy under More too).
	for st in ["normal", "focus", "read_only"]:
		_reader_edit.add_theme_stylebox_override(st, DccTheme.field_box(st, 12, 8))
	_reader_edit.add_theme_color_override("font_color", DccTheme.c("text_bright"))
	_reader_edit.add_theme_color_override("caret_color", DccTheme.c("accent"))
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
##
## ## Three states per radius, and three different things to say (2026-09-05)
##
## A filed path used to be the whole story here: `path != ""` drew a tick and
## labelled the button *Regenerate*. It is not, because the file lives outside
## this process and a file manager can take it away. `vault_snapshot_radii`
## now separates the cases and the row renders each differently:
##
## | row | shown | button |
## |---|---|---|
## | no `path` | `○ not generated` | *Generate* |
## | `path`, no `missing` | `✓ <path>` | *Regenerate* |
## | `path` + `missing` | `✕ <path> — written before, and not in the vault now` | *Generate … again* |
##
## The middle row is two states the panel cannot tell apart and does not
## pretend to: `missing` is omitted both when the image is there and when this
## device cannot check (a vault that is not on this filesystem). Omitted is
## also why the branch is `has()` and not `get(…, false)` — see the loop.
##
## *Regenerate* is deliberately not the word on the third row. Regenerating
## replaces a picture the note already renders; the third row's file is gone,
## so the act is putting it back, and the row says what that changes — a note
## pointing at it currently shows nothing, and Cartalith feedback offers no
## Map checkbox for it until it exists again.
func _build_snapshots(parent: Control) -> void:
	var radii := bridge.vault_snapshot_radii(_lk, _lid)
	if radii.is_empty():
		return
	var sec := DccWidgets.group(parent, "Map snapshot", false)
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

	var base_tip := "Crops the map you are looking at — the same renderer, the same look — around this entity and writes a %d px PNG into the folder above. Regenerating replaces that file, so a note already pointing at it shows the new picture." % SNAPSHOT_PX

	for r in radii:
		var d: Dictionary = r
		var radius := String(d.get("radius", ""))
		var have := String(d.get("path", ""))
		var caption := "%s — %d km across, %d cells" % [
			String(d.get("label", radius)), int(round(float(d.get("km", 0.0)) * 2.0)), int(d.get("cells", 0)) * 2 + 1]
		## `has("missing")`, never `get("missing", false)`: the engine omits
		## the key rather than sending `false`, because there is no value of it
		## that means *unknown* — see `vault_snapshot_radii`'s own table. A
		## filed path is a record that a snapshot was **written**; whether the
		## image is still there is the separate question `snapshot_on_disk`
		## answers, and it answers `None` — key omitted — for a vault this
		## device cannot look inside. So the tick below means "filed, and
		## nothing said it was gone", which is the strongest true statement
		## the panel has, and a wrongly-ticked row is only reachable on a
		## project written on a desktop and opened where the vault is not on
		## this filesystem.
		var status := "\n○ not generated"
		var verb := "Generate " + radius
		var tip := base_tip
		if have != "":
			if d.has("missing"):
				## Not *Regenerate*: there is nothing there to replace. The
				## file was written and something outside Cartalith removed
				## it, so the act on offer is putting it back — and it is
				## worth offering rather than hiding, because the Map checkbox
				## in Cartalith feedback stays away until it is back
				## (`vault_export_fields` filters through `entity_values`,
				## which drops a Map field whose image is gone).
				##
				## The condition on that repair is the folder field above, and
				## it is stated because the code branches on it: `rel` is
				## `{subdir}/{entity key}_{radius}.png` built from whatever
				## that field holds at the moment Generate is pressed, so
				## typing a different folder writes a new image somewhere else
				## and leaves the old path as dead as it was.
				status = "\n✕ %s — written before, and not in the vault now. A note already pointing at it shows nothing, and Cartalith feedback stops offering this map until the file is back." % have
				verb = "Generate %s again" % radius
				tip = base_tip + " This one was written and is gone — something outside Cartalith moved or deleted it. Generating again writes into the folder above: leave that at %s and the image comes back at the path a note is already pointing at, change it and the new image goes somewhere else with the old path still empty." % have.get_base_dir()
			else:
				status = "\n✓ %s" % have
				verb = "Regenerate " + radius
		DccWidgets.note(sec, caption + status)
		var b := DccWidgets.action(sec, verb, func():
			_generate_snapshot(radius, dir_edit.text.strip_edges()))
		b.tooltip_text = tip


## Writes a map snapshot of the scope entity at `radius` into `subdir` of the
## vault and files its path on the link store. A refusal is reported and nothing
## changes. Success emits `store_changed` (the path rides `vault.json` into the
## project archive) and rebuilds. Image size is fixed at `SNAPSHOT_PX`.
func _generate_snapshot(radius: String, subdir: String) -> void:
	var r := bridge.vault_snapshot(_lk, _lid, radius, subdir, SNAPSHOT_PX)
	if not bool(r.get("ok", false)):
		app.set_status("hint", "Snapshot: %s" % String(r.get("error", "refused")), "accent")
		return
	## The store changed — the snapshot's path is filed on it, and that is what
	## rides `vault.json` into the project archive.
	store_changed.emit()
	app.set_status("hint", "%s map written to %s (%d x %d)." % [
		radius, String(r.get("rel", "")), int(r.get("width", 0)), int(r.get("height", 0))], "text")
	_rebuild()


## The "Cartalith feedback" section: the field checkboxes for the entity's
## exportable values (all ticked by default) and the preview-and-write, field-fill
## and remove-block buttons. Map snapshots live in `_build_snapshots`. Says so, rather than drawing an empty form, when the
## entity exports nothing. Every write here is behind a hash-guarded preview.
func _build_feedback(parent: Control) -> void:
	var sec := DccWidgets.section(parent, "Cartalith feedback")
	var fields := bridge.vault_export_fields(_lk, _lid)
	if fields.is_empty():
		DccWidgets.note(sec, "Nothing to export for this entity — generate a world first, or this entity no longer resolves.")
		return
	var values := bridge.vault_entity_values(_lk, _lid)
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


## The export field keys currently ticked in `_selected_fields`, in the
## dictionary's order. Passed to `vault_block_body` so the block holds only what
## the user chose.
func _selected_keys() -> PackedStringArray:
	var out := PackedStringArray()
	for k in _selected_fields:
		if bool(_selected_fields[k]):
			out.append(String(k))
	return out


## The active link's file. Through `_link_rel`, which reads the whole store:
## the More section's link can belong to any entity, not only the scope.
func _link_path() -> String:
	return _link_rel(_reader_link)


## Previews, then (on confirm) writes the entity's Cartalith block into the
## active link's note. Shows the engine's preview with a note on whether the block
## is inserted or replaced; the preview's hash goes back with the write so a note
## edited in between is refused, not overwritten. Does nothing without an active
## link file. Honours the "confirm always" preference via `_preview_dialog`.
func _confirm_block_write() -> void:
	var rel := _link_path()
	if rel == "":
		return
	var body := bridge.vault_block_body(_lk, _lid, _selected_keys())
	var p := bridge.vault_preview_block(rel, _lk, _lid, body)
	if not bool(p.get("ok", false)):
		app.set_status("hint", "Preview: %s" % String(p.get("error", "")), "accent")
		return
	var action := String(p.get("action", ""))
	_preview_dialog("Write Cartalith block", String(p.get("preview", "")),
		("A new block will be inserted below the note's title." if action == "inserted"
			else "The existing Cartalith block will be replaced. Nothing outside it changes."),
		func():
			var r := bridge.vault_write_block(rel, _lk, _lid, body, String(p.get("hash", "")))
			if bool(r.get("ok", false)):
				app.set_status("hint", "Cartalith block %s." % String(r.get("action", "written")), "text_ghost")
			else:
				app.set_status("hint", "Write refused: %s" % String(r.get("error", "")), "accent")
			_rebuild(),
		"block")


## Previews, then (on confirm) fills the note's own template fields with the
## entity's values. Lists each field's outcome; if the note has none Cartalith can
## fill it says so on the status line instead of opening an empty dialog. A field
## the user already filled is skipped, never overwritten. Hash-guarded like every
## write here.
func _confirm_field_fill() -> void:
	var rel := _link_path()
	if rel == "":
		return
	var p := bridge.vault_preview_field_fill(rel, _lk, _lid, false)
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
			var r := bridge.vault_write_field_fill(rel, _lk, _lid, false, String(p.get("hash", "")))
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
	var body := bridge.vault_block_body(_lk, _lid, _selected_keys())
	var p := bridge.vault_preview_block(rel, _lk, _lid, body)
	if not bool(p.get("ok", false)):
		app.set_status("hint", "Preview: %s" % String(p.get("error", "")), "accent")
		return
	if String(p.get("action", "")) == "inserted":
		app.set_status("hint", "%s holds no Cartalith block for %s — there is nothing to remove." % [rel, _llabel], "text_ghost")
		return
	var text := bridge.vault_read_file(rel)
	var span := _block_span(text, String(p.get("entity_key", "")))
	_preview_dialog("Remove the Cartalith block",
		span if span != "" else text,
		("These are the bytes that will be removed. Everything else in %s is written back unchanged." % rel) if span != ""
			else ("Cartalith could not locate the block's markers in %s to show them on their own, so the whole note is above. The removal itself is the engine's, and it takes out only the delimited block for %s." % [rel, _llabel]),
		func():
			var r := bridge.vault_remove_block(rel, _lk, _lid, String(p.get("hash", "")))
			if not bool(r.get("ok", false)):
				app.set_status("hint", "Removal refused: %s" % String(r.get("error", "")), "accent")
			elif bool(r.get("removed", false)):
				app.set_status("hint", "Cartalith block removed from %s." % rel, "text_ghost")
			else:
				app.set_status("hint", "%s held no Cartalith block for %s." % [rel, _llabel], "text_ghost")
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
	## `phone_window()` first -- the same missing first call as `_compare_dialog`
	## above, and the file's own comment below already knew: it records that
	## `_floor_dialog_bar` "runs from `phone_window()` for this window and has
	## never run for these preview dialogs".
	##
	## Order matters HERE in a way it did not there: `phone_window()` sets
	## `ok_button_text` to "Close", and this dialog's caller supplies its own
	## ("Write to Markdown"). Setting the caller's text AFTER the call is what
	## keeps the verb; doing it before would have silently renamed the primary
	## action on phones only.
	var phone := DccWidgets.phone_window(dlg, app)
	dlg.get_ok_button().text = ok_text
	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 6)
	dlg.add_child(col)
	if phone:
		DccWidgets.phone_head(col, dialog_title, "")
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
	var dlg_want := dlg.size
	var dlg_host := PopoutHost.dialog_host(app, self)   # Ruling EA item 11, as in `_compare_dialog`
	dlg_host.add_child(dlg)
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
	if phone:
		app.phone_fit(dlg, 1.0)
	## Third call. `phone_present()` returns false off-phone, so the
	## desktop keeps `popup_centered()` as its own path rather than as a
	## fallback that would also fire on a phone.
	if not DccWidgets.phone_present(dlg, app):
		PopoutHost.present(dlg, dlg_host, dlg_want)


# -- "Confirm always", and where it is turned back off ----------------------

## Write-preference key -> the toggle label `_build_write_prefs` shows for it; the three keys are
## those the comment below says can be switched back on.
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
func _build_write_prefs(parent: Control) -> void:
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
	var sec := DccWidgets.group(parent, "Write confirmations", any_off)
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


## Persists the write-confirmation preferences through `VaultStore`. Call after
## every `vault_set_write_pref` so a preference survives a restart; preferences
## are per device and do not ride the project archive.
func _save_prefs() -> void:
	VaultStore.save_prefs_from(bridge)


# -- Overview ---------------------------------------------------------------

## The unscoped body: the index, the entity picker and the "All linked notes" list, one button per
## link that opens that entity's note via `open_for`. Shows a note when nothing is linked.
func _build_overview() -> void:
	## SM-5 (`STORY_MENUS_PROPOSAL.md` §5): above the Index, so above its
	## Missing & orphan notes group, as A5 draws it.
	_build_places_without_notes(_body)
	_build_index(_body)
	_build_pick_entity()
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
		var b := DccWidgets.action(sec, "%s %s — %s (%s)%s" % [
			kind.capitalize(), label, String(d.get("path", "")),
			String(STATUS_TEXT.get(String(d.get("status", "")), "")),
			privacy().badged(String(d.get("path", "")), "")],
			func(): open_for(kind, eid, label))
		b.alignment = HORIZONTAL_ALIGNMENT_LEFT


## Reaches `_build_create()` — the template-creation flow (`GUI_GAP_REGISTER.md`
## VA-02) — and the attach flow from the unscoped overview, for an entity that
## does not have a note linked yet, which "All linked notes" above cannot show
## (it lists links, not entities). Both only build when `_rebuild()` sees an
## entity scope, so picking any row here and re-entering through `open_for()`
## is what makes them reachable, not a duplicate of them.
##
## Provinces and continents are listed in full — the same two dictionaries and
## the same `id`/`name` keys `civilization_workspace.gd::_fill_knowledge()`
## already reads for its own "Linked notes" panel, so this reuses a shape
## proven correct there rather than guessing a settlement-sized one. Settlements
## are not listed in full (there can be hundreds); a name-substring filter over
## `bridge.settlements()`, on the same "type it, press Enter/Search" idiom
## `_build_search()` above already uses, finds one without walking the whole
## list on every keystroke.
func _build_pick_entity() -> void:
	var sec := DccWidgets.section(_body, "Attach or create a note")
	DccWidgets.note(sec,
		"Pick a settlement, province or continent to open its own Attach and "
		+ "Create-a-note-from-a-template sections — whether or not it already "
		+ "has a note linked. An entity's own panel (its Linked notes row) opens "
		+ "the same place.")

	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 8)
	row.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	sec.add_child(row)
	var field := LineEdit.new()
	field.placeholder_text = "settlement name"
	field.text = _pick_query
	field.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	DccWidgets.well(field)
	field.text_changed.connect(func(t: String): _pick_query = t)
	field.text_submitted.connect(func(t: String):
		_pick_query = t
		_rebuild())
	row.add_child(field)
	DccWidgets.action(row, "Find settlements", func():
		_rebuild())

	var q := _pick_query.strip_edges().to_lower()
	if q != "":
		var matches: Array = []
		for s in bridge.settlements():
			var d: Dictionary = s
			var name := String(d.get("name", ""))
			if name.to_lower().find(q) >= 0:
				matches.append(d)
		if matches.is_empty():
			DccWidgets.note(sec, "No settlement matches \"%s\"." % _pick_query.strip_edges())
		else:
			var sg := DccWidgets.group(sec, "Settlements", true)
			var shown := 0
			for d in matches:
				if shown >= 30:
					DccWidgets.note(sg, "%d more match — narrow the search to see them." % (matches.size() - shown))
					break
				var tid := int(d.get("tid", 0))
				var name := String(d.get("name", "?"))
				var b := DccWidgets.action(sg, name, func(): open_for("settlement", tid, name))
				b.alignment = HORIZONTAL_ALIGNMENT_LEFT
				shown += 1

	var provinces := bridge.provinces()
	if not provinces.is_empty():
		var pg := DccWidgets.group(sec, "Provinces", false)
		for p in provinces:
			var d: Dictionary = p
			var pid := int(d.get("id", 0))
			var pname := String(d.get("name", "?"))
			var b := DccWidgets.action(pg, pname, func(): open_for("province", pid, pname))
			b.alignment = HORIZONTAL_ALIGNMENT_LEFT

	var continents := bridge.continents()
	if not continents.is_empty():
		var cg := DccWidgets.group(sec, "Continents", false)
		for c in continents:
			var d: Dictionary = c
			var cid := int(d.get("id", 0))
			var cname := String(d.get("name", "?"))
			var b := DccWidgets.action(cg, cname, func(): open_for("continent", cid, cname))
			b.alignment = HORIZONTAL_ALIGNMENT_LEFT


# -- Places without notes (SM-5, Ruling CK) -----------------------------------
#
# `STORY_MENUS_PROPOSAL.md` §5, mockup `design/story-menus-2026-10-17/` A5:
# the places that have no linked note, so a writer can see what is still
# undocumented. The vault's Index (VA-01) answers the opposite question -- links
# that point at no note, notes nothing points at -- and neither answers this.
#
# **How the list is built.** §5 named one `vault_entity_summary(kind, id)` call
# per entity. A link store row already carries `entity_kind` and `entity_id`
# (`vault_bridge.rs::vault_all_links`), and `vault_entity_summary`'s
# `link_count` is `links_for(kind, id).len()` over that same store, so one
# `vault_all_links()` read plus the kind's own entity list gives the same
# answer. `_pwnotes_probe` asserts the two agree entity by entity, before and
# after a real attach. Neither is costly: measured 2026-10-18 (`_pwnotes_probe`,
# debug DLL, medians of 5 across three runs) at 1.7 ms (1.63..1.98) for this
# route and 0.8 ms (0.74..0.88) for the per-entity count alone, on worlds of 215
# and 240 settlements -- the placement model caps the count
# (`cartalith-civ::lib.rs` `max_places`, clamped 8..40, plus village add-ons),
# so a 4096x2048 world held the same 240 as a 2048 one. The whole overview
# opening, this section included, measured 38.4 ms (37.7..38.7) on that world.
#
# **Never off the main thread, and it does not need to be.** `WorldGen` is owned
# by the main thread except while `bridge.generating`, when the worker owns it;
# no other thread may call it. At the cost above, the read stays synchronous;
# every read asks `generating` first, and the list is built only while the
# overview is drawn -- never per frame.

## Entity kinds the view offers, in A5's chip order: `[vault kind, chip label,
## plural noun]`. Filtered at draw time against `bridge.vault_entity_kinds()`, so
## a build whose vault cannot link a kind never offers it.
const PWN_KINDS: Array = [["settlement", "Settlements", "settlements"],
	["province", "Provinces", "provinces"], ["continent", "Continents", "continents"],
	["faction", "Factions", "factions"]]
## Settlement classes, largest first: the engine's own vocabulary
## (`journey_bridge.rs::settlement_kind_key`). The rank is the list's sort key
## (A5: "Sorted by class, then population"); the chip row draws only the classes
## the current world holds, so no chip can filter to a class that is not there.
const PWN_CLASSES: Array = ["metropolis", "capital", "city", "town", "village", "hamlet"]
## Chip captions for `PWN_CLASSES`, index-parallel (A5's plural, upper-case).
const PWN_CLASS_CHIPS: Array = ["METROPOLISES", "CAPITALS", "CITIES", "TOWNS", "VILLAGES", "HAMLETS"]
## Rows drawn before "Show more". A labelled judgement: a page a reader scans in
## one scroll, and a bound on the controls one rebuild creates (each row is a
## label and two buttons; a world's hamlets alone can run to hundreds).
const PWN_PAGE := 40

## The kind the view lists (one of `PWN_KINDS`' first column).
var _pwn_kind := "settlement"
## The class filter, `""` for All (settlements only).
var _pwn_class := ""
## The "On a journey" filter (settlements only): keep places a saved journey passes.
var _pwn_journey := false
## The "In a conflict" filter (settlements only): keep places a conflict touches.
var _pwn_conflict := false
## Rows drawn so far; grows by `PWN_PAGE` per "Show more", reset by a filter change.
var _pwn_shown := PWN_PAGE
## Set by `open_places_without_notes()`: scroll the section into view once drawn.
var _pwn_focus := false
## The section body, held so `_pwn_focus` can scroll to it; nulled by `_clear`'s
## rebuild (re-made on every draw).
var _pwn_section: Control
## The last list built over a world (`_pwn_compute`'s answer plus `kind`), kept
## so the generating state can draw it DASHED rather than empty (A5: "The list is
## dashed, not emptied"). Never read for anything but that state.
var _pwn_last: Dictionary = {}

## The menu entry (Records ▸ RECORDS, signposted from Story ▸ NOTES, `menus.gd::ID_PLACES_
## WITHOUT_NOTES`): the overview, scrolled to this section, at a width the
## table fits (A5's 720 px frame, never more than 90% of the screen).
func open_places_without_notes() -> void:
	_pwn_focus = true
	_pwn_shown = PWN_PAGE
	open_overview()
	if not _phone:
		var room: Vector2 = app.get_viewport_rect().size * 0.9
		size = Vector2i(maxi(size.x, mini(760, int(room.x))), size.y)

## `{total, rows}` for one entity kind: `total` is how many entities of `kind`
## the world holds, `rows` those with no link in the store, each `{id, name,
## faction}` plus, for settlements, `class`, `population`, `x`, `y` and `index`
## (into `bridge.settlements()`). `faction` is the engine's id (`0` = none) and
## `faction_name` its roster name, read here once so the dashed (generating)
## redraw needs no engine read to name it. Three engine reads per call.
## Must not be called while `bridge.generating` -- the wrappers answer `[]` then,
## which would read as "this world has no places".
func _pwn_compute(kind: String) -> Dictionary:
	var linked := {}
	for l in bridge.vault_all_links():
		var d: Dictionary = l
		if String(d.get("entity_kind", "")) == kind:
			linked[int(d.get("entity_id", 0))] = true
	var rows: Array = []
	var total := 0
	var fnames := {}
	for f in bridge.get_factions():
		fnames[int((f as Dictionary).get("id", -1))] = String((f as Dictionary).get("name", ""))
	match kind:
		"settlement":
			var ss := bridge.settlements()
			for i in ss.size():
				var s: Dictionary = ss[i]
				total += 1
				var tid := int(s.get("tid", -1))
				if linked.has(tid):
					continue
				rows.append({"id": tid, "name": String(s.get("name", "")),
					"class": String(s.get("kind", "")), "population": int(s.get("population", 0)),
					"faction": int(s.get("faction", 0)), "x": float(s.get("x", 0)),
					"y": float(s.get("y", 0)), "index": i})
			## A5: "Sorted by class, then population" (largest first), then name
			## so equal rows keep one order between rebuilds.
			rows.sort_custom(func(a: Dictionary, b: Dictionary) -> bool:
				var ra := PWN_CLASSES.find(String(a["class"]))
				var rb := PWN_CLASSES.find(String(b["class"]))
				if ra != rb:
					return ra < rb
				if int(a["population"]) != int(b["population"]):
					return int(a["population"]) > int(b["population"])
				return String(a["name"]) < String(b["name"]))
		"province", "continent", "faction":
			var src: Array = bridge.provinces() if kind == "province" \
				else (bridge.continents() if kind == "continent" else bridge.get_factions())
			for e in src:
				var d: Dictionary = e
				var id := int(d.get("id", 0))
				## Faction 0 is the engine's "held by no faction" sentinel
				## (`_faction_name`), not a faction a note could be about.
				if kind == "faction" and id <= 0:
					continue
				total += 1
				if linked.has(id):
					continue
				var row := {"id": id, "name": String(d.get("name", ""))}
				if kind == "faction":
					row["faction"] = id
				elif d.has("faction"):
					row["faction"] = int(d["faction"])
				if kind == "continent" and d.has("cx"):
					row["x"] = float(d["cx"])
					row["y"] = float(d.get("cy", 0.0))
				rows.append(row)
			rows.sort_custom(func(a: Dictionary, b: Dictionary) -> bool:
				return String(a["name"]) < String(b["name"]))
	## `_faction_name`'s own wording for 0 and for an id the roster lacks.
	for r in rows:
		if (r as Dictionary).has("faction"):
			var fid := int(r["faction"])
			r["faction_name"] = "no faction" if fid <= 0 else String(fnames.get(fid, "faction %d" % fid))
	return {"total": total, "rows": rows}

## The section: readout, filter chips, the table, and its four states (A5): no
## vault connected (with the connect button -- every place would be "without a
## note"), no world, generating (the last list dashed, its buttons disabled),
## and every place linked. Reads the engine only when no generation is running.
func _build_places_without_notes(parent: Control) -> void:
	var sec := DccWidgets.section(parent, "Places without notes")
	sec.name = "PlacesWithoutNotes"
	_pwn_section = sec
	if _pwn_focus:
		_pwn_focus = false
		_scroll_to_pwn.call_deferred()
	var bound := bool(bridge.vault_info().get("bound", false))
	if not bound:
		_pwn_state(sec, "No vault connected",
			"Connect a folder of .md files first. Any folder works; Obsidian is one. Until then every place is \"without a note\", and the list would be the whole world.")
		var b := DccWidgets.action(_pwn_last_card, "Connect a vault folder…", _browse_vault, true)
		b.name = "PwnConnect"
		## A5 draws it as a compact button centred in the card, not a full-width bar.
		b.size_flags_horizontal = Control.SIZE_SHRINK_CENTER
		## `action()` wraps its label, which drops its minimum width to 0; shrunk
		## to its minimum, the phone drew it one letter per line. One line.
		b.autowrap_mode = TextServer.AUTOWRAP_OFF
		return
	if bool(bridge.generating):
		_pwn_state(sec, "While a generation runs",
			"The list is read when the generation finishes: the engine object belongs to the worker until then. Shown below is the last list, dashed.")
		if not _pwn_last.is_empty():
			_pwn_table(sec, _pwn_last, true)
		return
	if not bool(bridge.has_world):
		_pwn_state(sec, "No world yet",
			"Generate or open one. Generating places its settlements; CIVIL ▸ Populate re-runs the placement.")
		return
	var kinds := bridge.vault_entity_kinds()
	if not Array(kinds).has(_pwn_kind):
		_pwn_kind = "settlement"
	var t0 := Time.get_ticks_usec()
	var got := _pwn_compute(_pwn_kind)
	## Kept as a meta, not printed: `_pwnotes_probe` reads it to time the
	## live path; nothing in the shell reads it.
	set_meta("pwn_compute_usec", Time.get_ticks_usec() - t0)
	got["kind"] = _pwn_kind
	_pwn_last = got
	_pwn_filters(sec, kinds)
	_pwn_table(sec, got, false)

## The column of the card `_pwn_state` drew last, so a caller can put a button
## inside it (the unbound card's Connect). Nulled with the rest by `_clear`'s rebuild.
var _pwn_last_card: VBoxContainer

## One empty-state card: a title and its sentence, centred (A5's right column;
## A5 draws the border dashed, which a `StyleBoxFlat` cannot, so it is solid).
func _pwn_state(parent: Control, title: String, text: String) -> void:
	var card := PanelContainer.new()
	card.name = "PwnState"
	var sb := DccWidgets.box("border", "", 14, 10)
	sb.set_border_width_all(1)
	card.add_theme_stylebox_override("panel", sb)
	var col := VBoxContainer.new()
	col.add_theme_constant_override("separation", 4)
	card.add_child(col)
	_pwn_last_card = col
	var t := DccTheme.label(title, "text_bright", DccTheme.FS_BODY)
	t.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	col.add_child(t)
	var l := DccTheme.label(text, "text_dim", DccTheme.FS_SMALL)
	l.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	l.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	l.custom_minimum_size.x = 160
	col.add_child(l)
	parent.add_child(card)

## The two chip rows (A5): class (settlements only, the classes this world
## holds) and kind, plus On a journey / In a conflict (settlements only, each
## disabled with its reason when the store holds no journey or no conflict --
## a filter that can only answer "none" is not offered as live).
func _pwn_filters(sec: Control, kinds: PackedStringArray) -> void:
	if _pwn_kind == "settlement":
		var present := {}
		for s in bridge.settlements():
			present[String((s as Dictionary).get("kind", ""))] = true
		var crow := HFlowContainer.new()
		crow.name = "PwnClassChips"
		crow.add_theme_constant_override("h_separation", 4)
		crow.add_theme_constant_override("v_separation", 4)
		sec.add_child(crow)
		var all := DccWidgets.segment(crow, "ALL", func(): _pwn_set("class", ""))
		DccWidgets.set_segment_on(all, _pwn_class == "")
		for ci in PWN_CLASSES.size():
			var cc := String(PWN_CLASSES[ci])
			if not present.has(cc):
				continue
			var b := DccWidgets.segment(crow, String(PWN_CLASS_CHIPS[ci]), func(): _pwn_set("class", cc))
			DccWidgets.set_segment_on(b, _pwn_class == cc)
	var krow := HFlowContainer.new()
	krow.name = "PwnKindChips"
	krow.add_theme_constant_override("h_separation", 4)
	krow.add_theme_constant_override("v_separation", 4)
	sec.add_child(krow)
	for k in PWN_KINDS:
		if not Array(kinds).has(String(k[0])):
			continue
		var kk := String(k[0])
		var b := DccWidgets.segment(krow, String(k[1]).to_upper(), func(): _pwn_set("kind", kk))
		DccWidgets.set_segment_on(b, _pwn_kind == kk)
	if _pwn_kind != "settlement":
		return
	var has_journeys := not bridge.journey_list().is_empty()
	var has_conflicts := not bridge.conflict_list().is_empty()
	if not has_journeys:
		_pwn_journey = false
	if not has_conflicts:
		_pwn_conflict = false
	var jb := DccWidgets.segment(krow, "ON A JOURNEY", func(): _pwn_set("journey", not _pwn_journey))
	jb.name = "PwnJourneyChip"
	DccWidgets.set_segment_on(jb, _pwn_journey)
	jb.disabled = not has_journeys
	jb.tooltip_text = "Keeps the places a saved journey passes (the planner's own stop test, civ_settlement_journey_passes)." \
		if has_journeys else "No journey is saved in this world, so no place is on one. Save one from the Journey planner."
	var cb := DccWidgets.segment(krow, "IN A CONFLICT", func(): _pwn_set("conflict", not _pwn_conflict))
	cb.name = "PwnConflictChip"
	DccWidgets.set_segment_on(cb, _pwn_conflict)
	cb.disabled = not has_conflicts
	cb.tooltip_text = "Keeps the places a conflict is attached to, directly or through their province (conflicts_touching_settlement)." \
		if has_conflicts else "No conflict is recorded in this world. Draw one with Story ▸ Draw a conflict."

## A chip press: sets one filter, resets the page, rebuilds.
func _pwn_set(what: String, value) -> void:
	match what:
		"kind": _pwn_kind = String(value)
		"class": _pwn_class = String(value)
		"journey": _pwn_journey = bool(value)
		"conflict": _pwn_conflict = bool(value)
	_pwn_shown = PWN_PAGE
	_rebuild()
	_scroll_to_pwn.call_deferred()

## Scrolls the window to the section once it has a layout (deferred by callers).
func _scroll_to_pwn() -> void:
	if _pwn_section != null and is_instance_valid(_pwn_section) and _scroll != null:
		_scroll.ensure_control_visible(_pwn_section)

## The readout and the rows of `got` (`_pwn_compute`'s answer plus `kind`), after
## the class / journey / conflict filters. `dashed` draws them disabled (the
## generating state), and then reads nothing from the engine.
func _pwn_table(sec: Control, got: Dictionary, dashed: bool) -> void:
	var kind := String(got.get("kind", "settlement"))
	var noun := "places"
	for k in PWN_KINDS:
		if String(k[0]) == kind:
			noun = String(k[2])
	var rows: Array = got.get("rows", [])
	var total := int(got.get("total", 0))
	if not dashed and kind == "settlement":
		rows = rows.filter(func(r: Dictionary) -> bool:
			if _pwn_class != "" and String(r["class"]) != _pwn_class:
				return false
			if _pwn_journey and bridge.civ_settlement_journey_passes(int(r["id"])).is_empty():
				return false
			if _pwn_conflict and bridge.conflicts_touching_settlement(int(r["id"])).is_empty():
				return false
			return true)
	var without := int(got.get("rows", []).size())
	var readout := DccTheme.mono_label("%d of %d %s have no linked note" % [without, total, noun],
		"text", DccTheme.FS_SMALL, 0)
	readout.name = "PwnReadout"
	sec.add_child(readout)
	if total == 0:
		_pwn_state(sec, "No %s in this world" % noun,
			"This world holds no %s, so there is nothing to list." % noun)
		return
	if without == 0:
		_pwn_state(sec, "Every place has a note",
			"%d of %d %s are linked. Other kinds are counted separately; switch the filter above to see them." % [total, total, noun])
		return
	if rows.is_empty():
		DccWidgets.note(sec, "No %s without a note matches these filters." % noun)
		return
	var templates_ok := (not dashed) and not bridge.vault_templates().is_empty()
	var list := VBoxContainer.new()
	list.name = "PwnRows"
	list.add_theme_constant_override("separation", 0)
	sec.add_child(list)
	if not _phone:
		var head := HBoxContainer.new()
		head.add_theme_constant_override("separation", 8)
		list.add_child(head)
		var hn := DccTheme.mono_label("PLACE", "text_faint", DccTheme.FS_MICRO, 2)
		hn.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		head.add_child(hn)
		if kind == "settlement":
			var hc := DccTheme.mono_label("CLASS", "text_faint", DccTheme.FS_MICRO, 2)
			hc.custom_minimum_size.x = 76
			head.add_child(hc)
		if kind != "faction":
			var hf := DccTheme.mono_label("FACTION", "text_faint", DccTheme.FS_MICRO, 2)
			hf.custom_minimum_size.x = 120
			head.add_child(hf)
		var hl := DccTheme.mono_label("LINK", "text_faint", DccTheme.FS_MICRO, 2)
		hl.custom_minimum_size.x = 250
		head.add_child(hl)
	var shown := mini(rows.size(), _pwn_shown)
	for i in shown:
		_pwn_row(list, kind, rows[i], templates_ok, dashed)
	if rows.size() > shown:
		var more := DccWidgets.action(sec, "Show %d more (%d not shown)" % [mini(PWN_PAGE, rows.size() - shown), rows.size() - shown],
			func():
				_pwn_shown += PWN_PAGE
				_rebuild()
				_scroll_to_pwn.call_deferred())
		more.name = "PwnMore"
		more.disabled = dashed
	DccWidgets.note(sec,
		"New from template… writes Settlements/{name}.md (or the kind's own folder) from one of your templates and links it (the existing VA-02 flow). Attach existing… opens the place's own view, where any note in the tree can be attached. A row leaves this list once a link exists.")

## One table row: the name (a press moves the map to it, when the place has a
## position), class and faction, then the two link actions. Phone: two lines
## and the actions under them, so nothing runs past a 393 dp column.
func _pwn_row(list: Control, kind: String, r: Dictionary, templates_ok: bool, dashed: bool) -> void:
	var nm := String(r.get("name", ""))
	if nm == "":
		nm = "(unnamed #%d)" % int(r.get("id", 0))
	var id := int(r.get("id", 0))
	var cls := String(r.get("class", ""))
	var fac := String(r.get("faction_name", "")) if kind != "faction" else ""
	var row: BoxContainer = VBoxContainer.new() if _phone else HBoxContainer.new()
	row.name = "PwnRow"
	row.set_meta("pwn_id", id)
	row.add_theme_constant_override("separation", 8 if not _phone else 2)
	list.add_child(row)
	var name_ctl: Control
	if r.has("x") and not dashed:
		var nb := DccWidgets.text_button(row, nm, func():
			app.viewport.move_view_to(float(r["x"]), float(r["y"]))
			if _phone:
				hide())
		nb.tooltip_text = "Moves the map view to %s." % nm
		## A5 draws the name flush left, as a table cell; `text_button`
		## centres by default.
		nb.alignment = HORIZONTAL_ALIGNMENT_LEFT
		name_ctl = nb
	else:
		var nl := DccTheme.label(nm, "text_dim" if dashed else "text", DccTheme.FS_SMALL)
		nl.tooltip_text = "" if dashed else "This %s has no map position this list reads." % kind
		row.add_child(nl)
		name_ctl = nl
	name_ctl.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	if _phone:
		var sub := " · ".join(PackedStringArray([cls, fac].filter(func(s): return s != "")))
		if sub != "":
			row.add_child(DccTheme.mono_label(sub, "text_ghost", DccTheme.FS_TINY, 0))
	else:
		if kind == "settlement":
			var cl := DccTheme.mono_label(cls, "text_dim", DccTheme.FS_TINY, 0)
			cl.custom_minimum_size.x = 76
			row.add_child(cl)
		if kind != "faction":
			var fl := DccTheme.mono_label(fac, "text_ghost", DccTheme.FS_TINY, 0)
			fl.custom_minimum_size.x = 120
			row.add_child(fl)
	var acts := HBoxContainer.new()
	acts.add_theme_constant_override("separation", 6)
	row.add_child(acts)
	var newb := DccWidgets.chip(acts, "New from template…", func(): open_for(kind, id, nm))
	newb.name = "PwnNew"
	newb.disabled = dashed or not templates_ok
	newb.tooltip_text = "Opens %s's own view on its New-note-from-a-template block." % nm if templates_ok \
		else ("A generation is running." if dashed else
			"Your vault has no templates: Cartalith ships none, and looks in the template folder your Obsidian settings name, or at any .md whose path contains \"template\".")
	var att := DccWidgets.chip(acts, "Attach existing…", func():
		open_for(kind, id, nm)
		## The phone's one-pane fold opens an entity on its note pane; to
		## attach, the user needs the tree.
		if _phone:
			_browse_phone_pane = "tree"
			_rebuild())
	att.name = "PwnAttach"
	att.disabled = dashed
	att.tooltip_text = "A generation is running." if dashed \
		else "Opens %s's own view: pick a note in the tree, then Attach to %s." % [nm, nm]
	if not _phone:
		list.add_child(DccTheme.rule())

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
func _build_index(parent: Control) -> void:
	var info := bridge.vault_info()
	if not bool(info.get("bound", false)):
		return
	var sec := DccWidgets.section(parent, "Index")
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
	parent.add_child(row)
	if bool(st.get("built", false)):
		_build_index_report(parent)

## Refreshes the backlink index (stats every note, re-reads only changed ones),
## saves it through `VaultStore`, records a one-line outcome in `_index_feedback`
## (or the engine's error), and rebuilds the window. Reads at most the crate's `LIST_CAP`
## notes (12 000; VE-9, Ruling DY VQ7: it was a shell-side 2 000 chosen to match the list calls) and never
## writes to the vault.
func _refresh_index() -> void:
	privacy().clear()
	var r := bridge.vault_refresh_backlinks()
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
func _build_index_report(parent: Control) -> void:
	var rep := bridge.vault_backlink_report(40)
	var broken: Array = rep.get("broken", [])
	var orphans: PackedStringArray = rep.get("orphans", PackedStringArray())
	if broken.is_empty() and orphans.is_empty():
		return
	var g := DccWidgets.group(parent, "Missing & orphan notes", false)
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


## The overview's footer note: what the vault window deliberately does not do
## (two-way sync, an Obsidian plugin) and where links are persisted. Prose only;
## keep it in step with `VaultStore` and the snapshot and compare features it
## names.
func _build_footer() -> void:
	DccWidgets.note(_body,
		"Not built here, each for a stated reason: two-way sync and an Obsidian plugin — an "
		+ "explicit V1 non-goal and a deferred wish. The map snapshot (§21) is built, in an "
		+ "entity's own view: three radii cropped from the live renderer, and a row that says so "
		+ "when its image has gone missing from the vault. Compare-with-source (§14) is built now — the diff sits beside a stale "
		+ "link's Reload source button. Links themselves ride inside a saved project: File ▸ Save "
		+ "writes them into the project archive's own vault.json, beside the settlements and "
		+ "factions the same archive already carries, and opening that project restores them "
		+ "working. A copy is also kept beside your Cartalith profile, as the fallback for links "
		+ "made before anything has been saved.")

## MS-1 S4.1 (`S4_INVENTORY.md` §8, Q-S4-2's default): a class swap waits while the open note holds text
## that is not on disk (`_is_dirty()`, measured against `_browse_saved`). `DccShell.quiescent_blockers()`
## asks this only while the window is visible. A pure query: it never saves, discards or prompts.
## MS-1 S4.5 batch 5: this is now the ONLY thing the vault holds a swap for (`represent_capture()` carries the rest), and a
## text the capture's own `_write_now()` could save is saved before the swap runs, so what waits is a refused or conflicted save.
func quiescent_blockers() -> Array[String]:
	if _is_dirty():
		return ["a note has unsaved text"]
	return []

## MS-1 S4.5 (`window_represent.gd`, `S4_INVENTORY.md` section 18.8): what the user set, read BEFORE the swap unmounts
## anything. **One writer:** the text is flushed through `_write_now()`, the same call the autosave timer and a close use,
## so there is never a second path that writes a note (a refused or conflicted save leaves the text dirty, and
## `quiescent_blockers()` then holds the swap: unsaved text is never dropped). Carried: the scope (an entity, the browser, or the
## overview), the note and the reader's pick, the mode (Write, Split or Preview), the phone's pane, the scroll, the session undo
## (`_hist`, the checkpoints back to the note as first opened: memory only, so a rebuilt window would otherwise lose it) and the
## link trail. A closed window carries `open: false` and the same session undo, because a Note tab that docks a note after the
## swap reads `_hist` too. A bag holds no node; strings and arrays are copied.
func represent_capture() -> Dictionary:
	_write_now()
	return {
		"open": visible, "kind": _kind, "entity": _entity_id, "label": _entity_label, "browse_only": _browse_only,
		"path": _browse_path if not is_docked() else "", "pick": _pick_file, "reader": _reader_link, "heading": _pick_heading,
		"mode": _editor_mode, "pane": _browse_phone_pane, "hist": _hist.duplicate(true), "back": _link_back.duplicate(true),
		"scroll": WindowRepresent.scroll_value(_body),
	}

## On the NEW window, after the mount: the session undo and the trail first (a note opened next seeds its editor from them),
## then, when it was open, the same scope through the window's own openers (which clear the privacy state and rebuild, as any
## open does), the note through `_enter_note()`, the mode through `_editor_mode` (read by the editor's `setup()`) and a rebuild.
## A note that has since gone from disk is left unopened and the status line says why (`_load_note_state()`).
func represent_restore(bag: Dictionary) -> void:
	_hist = (bag["hist"] as Dictionary).duplicate(true)
	if not bool(bag["open"]):
		return
	if String(bag["kind"]) != "":
		open_for(String(bag["kind"]), int(bag["entity"]), String(bag["label"]))
	elif bool(bag["browse_only"]):
		open_browse()
	else:
		open_overview()
	if String(bag["path"]) != "":
		_enter_note(String(bag["path"]))
	elif String(bag["pick"]) != "":
		_pick_file = String(bag["pick"])
		_reader_link = String(bag["reader"])
		_pick_heading = String(bag["heading"])
	_link_back = (bag["back"] as Array).duplicate(true)
	_editor_mode = String(bag["mode"])
	_browse_phone_pane = String(bag["pane"]) if _phone else "preview" if String(bag["path"]) != "" else _browse_phone_pane
	_rebuild()
	await WindowRepresent.restore_scroll(_body, int(bag["scroll"]))
