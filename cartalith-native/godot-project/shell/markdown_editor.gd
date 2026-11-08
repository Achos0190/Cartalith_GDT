extends VBoxContainer
## The vault's Markdown editor -- owner Ruling BE (`LARGE_ITEM_RULINGS.md`,
## 2026-09-27): *"give the editor basic word editor functions, bold,
## understriped, text size, header, like a functional markdown editor."*
##
## Loaded by `vault_window.gd` with `preload()`, deliberately without a
## `class_name`: a new global class needs a `godot --headless --import` pass,
## and that pass strips `project.godot`'s `;` comments (`MISTAKES.md`). Nothing
## outside the vault window builds one.
##
## ## What it is
##
## A header row (file name, Write / Preview, Revert to opened…, Save), a save banner, a toolbar, and one
## body that is either the source (`text_edit`) or its rendering (`preview`).
## It never touches the disk: Save, Revert and the banner's choices are signals, and the vault
## window answers them through `vault_write_file`'s hash guard (autosave, the conflict banner and the
## session undo are VE-4; `vault_window.gd::_write_now` is the one place a note is written). So this file
## cannot write a note, and the guard cannot be bypassed from here.
##
## ## The markup, and the two defaults the ruling states
##
## - **Underline writes `<u>…</u>`.** Markdown has no underline; Obsidian and
##   most renderers accept the HTML tag, and it round-trips as plain text.
## - **"Text size" is the heading level.** Markdown stores no font size, so the
##   picker is Normal / H1 / H2 / H3 -- it rewrites the caret line's `#`
##   prefix and nothing else.
##
## ## Obsidian's syntax, exactly (owner Ruling BF, 2026-09-27)
##
## Every mark the toolbar writes is Obsidian's own, so a note edited here reads
## identically there: `**bold**`, `*italic*`, `~~strike~~`, `==highlight==`,
## `[[note]]` / `[[note|shown text]]`, `#tag`, `- [ ] ` tasks and
## `> [!note]` callouts. Underline stays `<u>…</u>` because Obsidian has none of
## its own. Preview reads the same set plus `[[note#heading]]` (shown as
## "note > heading", as Obsidian's reading view shows it) and `![[embed]]`.
## With no host hooks an embed is the labelled placeholder ("embedded: name"); with them (VE-7, below)
## it is the embedded note, picture or a notice saying why not. A callout is drawn in its type's colour
## with a glyph and a bar (VE-7, `callout_style`).
## **Ctrl+E** flips Write / Preview, as Obsidian's reading-view toggle does.
##
## ## Links (VE-1, `VAULT_EDITOR_PROPOSAL.md`)
##
## The editor knows no vault. The host sets `link_info`, `link_at` and `peek_source` before `setup()`;
## Preview then draws each link by kind and a click emits `link_followed`, as does Ctrl/Cmd-click in
## Write. A hover (650 ms) or a finger hold (500 ms) shows a peek card. The Back button only emits
## `back_requested`; the host owns the trail. Nothing here opens, creates or saves a file.
##
## ## `[[` suggestions (VE-2)
##
## Typing `[[` auto-pairs to `[[]]` and opens a suggestion card at the caret (`suggest_context`,
## `suggest_apply`: pure, probe-pinned). The editor holds no vault; the host's `suggest_source` returns
## the ranked rows (notes by name and alias, world entities with no note, headings after `#`, a "link to
## new note" row). The card is never the whole vault: at most `SUGGEST_ROWS` rows (`SUGGEST_ROWS_PHONE`
## on the phone), re-ranked on every keystroke. Up / Down move, Enter or Tab accept, Escape closes the
## card (not the window), a click or tap on a row accepts. Rows are 44 px on touch. Accepting writes
## the link text and nothing else: it creates no note and saves nothing itself (VQ2; the window's autosave saves the note a second later).
## **Insert template** puts one of the vault's templates at the caret, filled
## and with its properties merged (`cartalith_vault::template::insert`, reached
## through the `template_inserter` the host passes to `setup`).
##
## ## Write colouring, Split, smart keys, find and replace, word count (VE-3)
##
## - **Colour.** `markdown_highlighter.gd` colours the source by column (headings, dimmed markers,
##   links, tags, code). Colour only: a `SyntaxHighlighter` cannot change a glyph's size or weight, so
##   the caret and the colours cannot drift apart, and Preview / Split carry the rendering.
## - **Split** shows Write and Preview side by side while the editor is at least `SPLIT_MIN_DP`
##   (760) wide and not the phone; the button appears at exactly that width. Preview follows the
##   text a moment after typing stops (`SPLIT_REFRESH_SEC`).
## - **Smart keys.** Enter continues a list, an (unticked) task, a numbered item or a quote and ends
##   the list on an empty one; Tab / Shift+Tab indent / outdent a list line; typing `**` pairs.
##   Ctrl/Cmd+K link, +L task, +S save (an explicit save: this file still never writes), +F find,
##   +H find and replace. Each is a pure `static func` (`smart_enter`, `find_matches`, `replace_all`).
## - **Find and replace** is a bar under the toolbar. Replace all is ONE undo step.
## - **Footer.** Word count and outgoing links, from the host's `word_counter` / `link_counter`
##   (the Rust crate); a figure the host cannot give is left out, never shown as 0.
##
## ## The save banner and the session undo (VE-4)
##
## - **The banner** (`show_banner` / `hide_banner`, signal `banner_choice`) is a blocking state above the
##   toolbar: a conflict, a deleted note, a read-only file, a vanished vault folder. The host decides the
##   words and the choices; this file only draws them (44 dp buttons on the phone) and emits the key.
## - **`replace_whole_text`** swaps the whole text as ONE undo step (a revert, taking the version on disk).
##   **`seed_history`** replays the host's checkpoints so Ctrl+Z reaches the first-opened text after a close
##   and reopen or a rebuild; neither keeps a `TextEdit`'s own history. The status line reads
##   `editing…` / `saved · time` (`mark_saved`).
##
## ## Tables, pictures, embeds, task ticking and chronos in Preview (VE-7)
##
## - **Tables** (GFM pipe tables: a header row, a delimiter row of the same width, then rows) draw as a `[table]`
##   with the header bold and washed and each column aligned as its delimiter said (`table_aligns`,
##   `split_table_row`; no host needed).
## - **Embeds** go through the host's `embed_source` (the crate's `vault_embed_open`, which owns every rule:
##   a cycle, the depth cap, a path that leaves the vault). A note embed shows its first section (or the
##   `#Heading` or `#^block` it names) under a header, collapsed past `EMBED_COLLAPSED_LINES` lines with a
##   click to expand, read-only (its tasks are not clickable, and it is edited by opening it). A picture draws as
##   `[img]` at its own width, or the `|N` the note asked for, capped at `IMG_MAX_PX`. Anything else (missing,
##   ambiguous, a cycle, too deep, outside the vault, unsupported, too large) is ONE warn-coloured line saying
##   why: never blank, never a dialog. `EMBED_HARD_LIMIT` is a backstop under the crate's rule so a wrong hook cannot
##   make Preview recurse without end. A picture in a sentence stays in it; a note in a sentence is a link.
## - **Task ticking.** With a `task_toggler`, the box of a task in the open note is a link; a click asks the
##   crate to flip that line and writes the result back as ONE undo step through `replace_whole_text`. The editor
##   still never writes a file: the host's autosave does, through the one writer.
## - **Chronos.** A ` ```chronos ` block (parsed by the crate through `chronos_source`) draws as a timeline: a count, a
##   text axis (`chronos_axis`), then one row per entry in year order. A click on a row emits
##   `chronos_year_requested(year)`; the host moves the shell's year (`tl_set_year`). Lines the block could not
##   read are listed, never dropped.
##
## Every transform is a `static func` over a plain `String` and two character
## offsets, returning `{"text", "from", "to"}`. That is what lets
## `_mdedit_probe.gd` pin each one with exact strings and no UI at all, and it
## is why the buttons below are thin: read the selection as offsets, call the
## transform, write the result back as ONE undoable edit, restore the
## selection.

## The Save button was pressed. This file never writes a file; the vault window answers it through
## `vault_write_file`'s hash guard.
signal save_requested
## "Revert to opened…" was pressed (VE-4, Ruling DY VQ6; it replaced Cancel). The host asks first, then
## calls `replace_whole_text` with the note as first opened this session; nothing here reverts the text.
signal revert_requested
## A button of the save banner (`show_banner`) was pressed; `key` is the choice's key. The host answers
## it (Keep mine, Take theirs, Keep both, Recreate, Retry...); the editor never touches the file.
signal banner_choice(key: String)
## Emitted after any edit, typed or made by a toolbar button.
signal edited
## Emitted when Write / Preview flips, so a host that rebuilds can keep it.
signal mode_changed(mode: String)
## A link was followed (VE-1): a click on a rendered link in Preview, or Ctrl/Cmd-click on one in
## Write. `target` is the link's own text -- `Note`, `Note#Heading`, `Folder/Note` -- exactly as
## written between the brackets (alias dropped); the host resolves it and decides what following
## means. Nothing here opens a file or creates one.
signal link_followed(target: String)
## A rendered external link (`http`, `https` or `mailto`) was clicked. Separate from `link_followed`
## because an external address is never resolved against the vault.
signal external_link_followed(url: String)
## A chronos row in Preview was clicked (VE-7): the host moves Cartalith's year to `year` through the shell's one
## writer (`tl_set_year`, PC-2), which clamps it to the world's range (PC-3). The editor moves nothing itself.
signal chronos_year_requested(year: int)
## The Back button was pressed (VE-1): return to the note the last followed link came from.
signal back_requested

## Host hook for `[[` suggestions (VE-2): `suggest_source(mode, note, query, limit) -> Array`.
## `mode` is `note` (`query` is the text after `[[`) or `heading` (`note` is the target written before
## `#`, `""` for this note; `query` is the text after it). Each row is a Dictionary:
## `{label, insert, kind, detail?, alias?, enabled?}` -- `insert` is the text written as the link's
## target (or the heading), `alias` the shown text to append as `|alias` when an alias matched, `kind`
## one of `note`, `entity`, `heading`, `create`, `info`. A row with `enabled` false is shown and
## cannot be accepted (an honest "nothing here" line). Without it nothing is suggested.
var suggest_source: Callable = Callable()

## Host hooks for links (VE-1), plain vars set between `new()` and `setup()` so a note that opens in
## Preview already draws its links by kind. The editor holds no vault knowledge of its own.
## `link_info(target: String) -> Dictionary` answers `{kind}` with `note`, `entity` (a world entity
## with no note) or `unresolved`; without it every link draws as a note link and nothing is classed.
var link_info: Callable = Callable()
## `link_at(text: String, offset: int) -> Dictionary`, the Rust `vault_link_at` (`{found, target?}`);
## Ctrl/Cmd-click in Write needs it.
var link_at: Callable = Callable()
## `peek_source(target: String) -> String`: BBCode for the peek card, or `""` for none (an
## unresolved link has nothing to show).
var peek_source: Callable = Callable()
## Footer hooks (VE-3), set by the host like the three above. `word_counter(text) -> int` is the
## crate's `vault_word_count` (frontmatter excluded); `link_counter(text) -> int` counts the links that
## name another note (`vault_outgoing_links`). A negative answer means "unavailable": without a hook, or
## with one, that figure is simply not shown.
var word_counter: Callable = Callable()
var link_counter: Callable = Callable()
## Host hook: `link_spans(line: String) -> Array`, `vault_link_spans`' rows for one line, so the
## highlighter colours links from the crate and not from a second parser here.
var link_spans: Callable = Callable()

## Host hooks for what Preview draws beyond text (VE-7), set between `new()` and `setup()` like the link hooks.
## `note_path` is the open note's vault path (the root of the embed chain); `embed_source(inner, stack) ->
## Dictionary` is the crate's `vault_embed_open` for the note at `stack.back()`; `embed_image(result) ->
## {src, w, h}` decodes and registers a picture (`{}` when it cannot); `chronos_source(text) -> Dictionary` is
## `vault_note_chronos`; `task_toggler(text, line) -> Dictionary` is `vault_toggle_task`. Without `embed_source`
## or `note_path` embeds stay the labelled placeholder; without `chronos_source` a chronos block stays code;
## without `task_toggler` a task's box is not clickable.
var note_path := ""
var embed_source: Callable = Callable()
var embed_image: Callable = Callable()
var chronos_source: Callable = Callable()
var task_toggler: Callable = Callable()
## Embed keys the user expanded (key -> true), kept across re-renders; the keys come from the last render.
var _embeds_open := {}
var _last_embed_keys: Array = []

## Narrowest editor, in dp, at which Split is offered (`VAULT_EDITOR_PROPOSAL.md` s.3.1, s.4). The
## editor's width in control units IS dp: a window's `content_scale_factor` is already divided out.
const SPLIT_MIN_DP := 760
## Pause after the last keystroke before Split's Preview re-renders (a labelled judgement: a render is
## a full `to_bbcode` pass, and typing should not wait on it).
const SPLIT_REFRESH_SEC := 0.2
## Pause before the footer's counts are recomputed after an edit (the crate counts the whole note).
const FOOTER_REFRESH_SEC := 0.25
## What Tab inserts to indent a list line, and the most spaces Shift+Tab removes from one that is
## indented with spaces. Obsidian's defaults are tabs, four wide (labelled judgement).
const INDENT := "\t"
const OUTDENT_SPACES := 4

## Pointer hover dwell before a peek shows (`VAULT_EDITOR_PROPOSAL.md` s.3.2: 650 ms), and the hold
## that stands in for it under a finger (500 ms; `_ve0_measure_probe.gd` Q2: a still finger produces
## no motion event, so Godot never starts a hover for it). Both are labelled judgements.
const PEEK_HOVER_SEC := 0.65
const PEEK_HOLD_SEC := 0.5

## Rows the suggestion card shows at most, desktop and phone (labelled judgements: eight 28 px rows
## are about a quarter of a 1000 px window; six 44 px rows fit above a soft keyboard). This cap is
## what keeps the card from ever being the whole vault.
const SUGGEST_ROWS := 8
const SUGGEST_ROWS_PHONE := 6
## Row height in px on pointer and on touch. 44 is the touch-target floor the shell's own touch
## audits use (`MOBILE_SHELL_PROPOSAL.md`); 28 is a labelled judgement for a pointer.
const SUGGEST_ROW_PX := 28
const SUGGEST_ROW_TOUCH_PX := 44

## Bold face for rendered bold and for the toolbar's `B` glyph (the project's Fira Sans file;
## `style_rendered()` also uses it for bold-italic, as there is no separate face).
const FONT_BOLD := preload("res://fonts/FiraSans-Bold.ttf")
## Italic face for rendered emphasis and the toolbar's `I` glyph.
const FONT_ITALIC := preload("res://fonts/FiraSans-Italic.ttf")
## Write-mode colouring (VE-3); a script and not a `class_name`, as this file is.
const Highlighter := preload("res://shell/markdown_highlighter.gd")

## Rendered heading sizes. Not `DccTheme` tokens -- the theme has no heading
## scale below `FS_MODAL_TITLE` (16) and `FS_HERO` (26); these sit on that
## ladder: H1 at the modal title's 16 + 4, H2 at 16, H3 at 14.
const HEADING_PX := {1: 20, 2: 16, 3: 14, 4: 13, 5: 12, 6: 12}

## The Write-mode `TextEdit`; `is_dirty` compares its text with `_saved_text`.
var text_edit: TextEdit
## The Preview-mode `RichTextLabel`; `set_mode` swaps it with `text_edit` and carries focus across.
var preview: RichTextLabel
## The status line beside the buttons (word-smart autowrap, `text_ghost` colour), created in
## `setup`; its text and colour are set by `set_status`.
var status_label: Label
## The header's Save button, made in `_build_header`; it emits `save_requested`.
var save_button: Button
## The header's "Revert to opened…" button, made in `_build_header`; it emits `revert_requested`.
var revert_button: Button
## VE-4's save banner (a conflict, a deleted note, a read-only file, a vanished vault): hidden until
## `show_banner`. Built in `setup` between the header and the toolbar so it is the first thing read.
var banner: PanelContainer
var _banner_label: Label
var _banner_row: HFlowContainer

## Every toolbar control, by key -- `bold`, `italic`, `underline`, `strike`,
## `highlight`, `h0`..`h3`, `bullet`, `number`, `task`, `quote`, `callout`,
## `code`, `code_block`, `link`, `tag`, `template`, `write`, `preview`. A probe
## presses the real button through this map.
var buttons := {}

## Callable given to `setup` that returns the vault's note paths. Kept for hosts that still pass it;
## VE-2's suggestions come from `suggest_source`, which the host ranks itself.
var _files_source: Callable
## "write" or "preview"; any value other than "preview" given to `set_mode` becomes "write", and
## `mode()` returns it.
var _mode := "write"
## Whether the host is the phone presentation, given to `setup`; `_build_header` branches on it.
var _phone := false
## The text the file held when last read or written, given to `setup`; the dirty check measures
## against it.
var _saved_text := ""
## Back button in the header (VE-1); hidden until the host has somewhere to go back to.
var _back_button: Button
## The header row (title, Back, Write / Split / Preview, Revert, Save), kept so the host can add a button
## to it (`add_header_button`).
var _header_row: HFlowContainer
## The peek card, its dwell timer, the link it is for, and whether a finger's hold produced it (the
## release that ends such a hold must not also follow the link).
var _peek: PanelContainer
var _peek_label: RichTextLabel
var _peek_timer: Timer
var _peek_target := ""
var _peek_by_hold := false
var _swallow_click := false
## The suggestion card (VE-2): a top-level panel named `EditorSuggest` holding one button per row,
## the rows last shown, the highlighted one, the context they answer, and whether a `[[` edit opened
## it (a caret move alone never opens it, only keeps or closes an open one).
var _sg_panel: PanelContainer
var _sg_box: VBoxContainer
var _sg_items: Array = []
var _sg_index := 0
var _sg_ctx: Dictionary = {}
var _sg_active := false
## Character count after the previous edit; a +1 step is how a typed `[` is told from a paste (and
## from the editor's own multi-character writes, which Godot reports to `text_changed` later, in a
## deferred call -- so a flag held while writing would already be cleared).
var _last_len := 0
## Whether the last pointer input was a finger (an emulated mouse) -- sizes the rows.
var _touch_seen := false
## The row holding Write's `text_edit` and Preview's `preview`, side by side in Split (VE-3).
var _body: HBoxContainer
## The source colouring, and the wish for Split that survives the window being too narrow for it (a
## rebuild at the fold, or setup before the first layout pass, must not lose it).
var _hl
var _split_wanted := false
## The mode Ctrl+E returns to from Preview (Write, or Split when that is where the note was).
var _before_preview := "write"
var _split_timer: Timer
var _footer_timer: Timer
## The footer's count label.
var _footer_label: Label
## The find bar (VE-3), its fields, buttons and state. `_find_matches` is the hits for the current
## query as `Vector2i(from, to)` character offsets, `_find_case` the match-case toggle.
var _find_bar: HFlowContainer
var _find_edit: LineEdit
var _replace_edit: LineEdit
var _find_count: Label
var _find_replace_box: HBoxContainer
var _find_case_button: Button
var _find_case := false
var _find_matches: Array = []
var _find_open := false
## A message that outlives the edit that caused it ("Replaced 5"): the replace changes the text, which
## recounts the hits, and the recount must not wipe the answer. Cleared when the query changes.
var _find_notice := ""
## Owners of the window's Escape (the suggestion card, the find bar): `dialog_close_on_escape` stays
## off while any holds it.
var _esc_owners := {}
## Callable given to `setup` that returns the template choices; `template_choices` calls it.
var _templates_source: Callable
## Callable given to `setup` that inserts a chosen template; `insert_template` refuses when it is
## not valid.
var _template_inserter: Callable
## The template picker `PopupMenu` (named "EditorTemplatePicker"), created in `_build_toolbar`.
var _template_popup: PopupMenu
## Relative paths listed by the template picker, reset by `template_choices`; a picked popup id
## indexes it.
var _template_rels: PackedStringArray = PackedStringArray()


## `initial` is the working text; `saved` is what the file held when it was
## last read or written (the dirty check compares against it). `files_source`
## is no longer read (VE-2 replaced the note-link picker with `suggest_source`); it stays in the
## signature so existing callers keep working.
## `templates_source` returns the vault's templates (`[{rel, label}]`) and
## `template_inserter(rel, text, caret)` returns `{ok, text, caret}` or
## `{ok: false, error}`; without both, Insert template says so when pressed.
func setup(file_label: String, initial: String, saved: String, phone: bool,
		files_source: Callable, mode: String = "write",
		templates_source: Callable = Callable(), template_inserter: Callable = Callable()) -> void:
	_phone = phone
	_files_source = files_source
	_templates_source = templates_source
	_template_inserter = template_inserter
	_saved_text = saved
	size_flags_horizontal = Control.SIZE_EXPAND_FILL
	size_flags_vertical = Control.SIZE_EXPAND_FILL
	add_theme_constant_override("separation", 6)

	_split_wanted = mode == "split"
	_build_header(file_label)
	_build_banner()
	_build_toolbar()
	_build_find()
	_build_body(initial)

	status_label = DccTheme.label("", "text_ghost", DccTheme.FS_SMALL)
	status_label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	status_label.custom_minimum_size.x = 160
	status_label.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	var status_row := HBoxContainer.new()
	status_row.add_theme_constant_override("separation", 8)
	status_row.add_child(status_label)
	## The footer (VE-3): word count and outgoing links, right of the status. Empty until the host's
	## counters answer; recomputed a moment after each edit.
	_footer_label = DccTheme.label("", "text_ghost", DccTheme.FS_SMALL)
	_footer_label.name = "EditorFooter"
	_footer_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_RIGHT
	status_row.add_child(_footer_label)
	add_child(status_row)

	_split_timer = Timer.new()
	_split_timer.one_shot = true
	_split_timer.wait_time = SPLIT_REFRESH_SEC
	_split_timer.timeout.connect(_refresh_split_preview)
	add_child(_split_timer)
	_footer_timer = Timer.new()
	_footer_timer.one_shot = true
	_footer_timer.wait_time = FOOTER_REFRESH_SEC
	_footer_timer.timeout.connect(_refresh_footer)
	add_child(_footer_timer)
	resized.connect(_on_resized)

	set_mode(mode)
	_refresh_heading_lights()
	_refresh_footer()


## Adds a button to the header, just before Revert to opened… (VE-5: the host's side-panel toggle). It
## is the host's button and the host owns what it does; the editor only gives it a place and the touch
## floor. Returns it.
func add_header_button(text: String, tip: String, on_press: Callable) -> Button:
	var b := DccWidgets.action(_header_row, text, on_press)
	b.tooltip_text = tip
	_header_row.move_child(b, revert_button.get_index())
	_touch_floor(b)
	return b


## Brings 0-based `line` into view and puts the caret at its start (VE-5: a backlink's link, an
## unlinked mention, an outline heading). In Write and Split the caret moves and the line is centred a
## frame later (the `TextEdit` has no layout until it has been drawn once, so centring now would use a
## zero-height viewport). In Preview and Split the preview is scrolled to `heading`'s own line when a
## heading text is given: the preview keeps no line map, so the heading is found by its text at the
## start of a rendered line, counting how many identical headings come before it in the source, and
## nothing scrolls if the rendered text does not contain it (a heading with inline markup), rather
## than scrolling somewhere wrong.
func goto_line(line: int, heading: String = "") -> void:
	if text_edit == null:
		return
	var m := mode()
	if m != "preview":
		var l := clampi(line, 0, text_edit.get_line_count() - 1)
		text_edit.deselect()
		text_edit.set_caret_line(l)
		text_edit.set_caret_column(0)
		text_edit.grab_focus()
	if m != "write" and heading != "" and preview != null:
		var k := 0
		var lines := text_edit.text.split("\n")
		for i in mini(line, lines.size()):
			if lines[i].strip_edges().lstrip("#").strip_edges() == heading:
				k += 1
		var parsed := preview.get_parsed_text()
		var from := 0
		var hit := -1
		var seen := 0
		while true:
			var idx := parsed.find(heading, from)
			if idx < 0:
				break
			from = idx + maxi(heading.length(), 1)
			if idx == 0 or parsed[idx - 1] == "\n":
				if seen == k:
					hit = idx
					break
				seen += 1
		if hit >= 0:
			preview.scroll_to_line(preview.get_character_line(hit))
	if m != "preview":
		await get_tree().process_frame
		if is_instance_valid(text_edit):
			text_edit.center_viewport_to_caret()


## Whether the source differs from the text last read or written (`_saved_text`). False before
## `setup()` builds the source.
func is_dirty() -> bool:
	return text_edit != null and text_edit.text != _saved_text


## Record `text` as what the file now holds, so `is_dirty()` goes false, and say "saved" (VE-4: with
## the time when `stamp` is given, so a person can see THAT it saved, which is what Obsidian's forum
## users ask for). Call only after the host's write succeeded; calling it early hides unsaved edits
## from the dirty check.
func mark_saved(text: String, stamp: String = "") -> void:
	_saved_text = text
	set_status("saved · %s" % stamp if stamp != "" else "saved", false)


## Replace the whole text as ONE undo step (a revert, or taking the version on disk), keeping the
## caret on a line that exists. The same shape as `_commit`, but it does not move focus or the
## selection, and it resets the auto-pair length tracker: this is not typing, and a one-character
## difference must not look like a typed `[`.
func replace_whole_text(t: String) -> void:
	if t == text_edit.text:
		return
	var cl := text_edit.get_caret_line()
	text_edit.begin_complex_operation()
	var last := text_edit.get_line_count() - 1
	text_edit.remove_text(0, 0, last, text_edit.get_line(last).length())
	text_edit.insert_text(t, 0, 0)
	text_edit.end_complex_operation()
	_last_len = t.length()
	cl = clampi(cl, 0, text_edit.get_line_count() - 1)
	text_edit.set_caret_line(cl)
	text_edit.set_caret_column(mini(text_edit.get_caret_column(), text_edit.get_line(cl).length()))
	_hl.refresh()
	_footer_timer.start()
	if _mode == "split":
		_split_timer.start()


## Rebuild the undo history from `snaps` (VE-4's session undo): `snaps[0]` is the note as first
## opened this session, each later entry a checkpoint, and the last is the text to show. Every step
## from one entry to the next is one undo step, so Ctrl+Z walks back to `snaps[0]` even though this
## editor was built after the note was closed, or after a rebuild (dragging across the fold width),
## neither of which keeps a `TextEdit`'s own history. The caller passes it right after `setup`.
func seed_history(snaps: Array) -> void:
	if snaps.size() < 2 or text_edit == null:
		return
	text_edit.text = String(snaps[0])
	text_edit.clear_undo_history()
	for i in range(1, snaps.size()):
		replace_whole_text(String(snaps[i]))
	text_edit.set_caret_line(0)
	text_edit.set_caret_column(0)
	_last_len = text_edit.text.length()


## The phone's 44 dp touch floor for a button this file adds (VE-4). `DccWidgets.action` floors only the
## tablet shell (its `btn_min_h`); on the phone it is 26 high, which a thumb misses. 44 is the shell's
## own touch-target floor (`SUGGEST_ROW_TOUCH_PX`, and VE-3's find bar on the phone).
func _touch_floor(b: Button) -> void:
	if _phone:
		b.custom_minimum_size.y = maxf(b.custom_minimum_size.y, float(SUGGEST_ROW_TOUCH_PX))


## Show the save banner: `msg` and one button per entry of `choices` (`[{key, label}]`). A
## button emits `banner_choice(key)`. Replaces any banner already showing. 44 dp buttons on touch
## come from `DccWidgets.action`.
func show_banner(msg: String, choices: Array) -> void:
	if banner == null:
		return
	_banner_label.text = msg
	for c in _banner_row.get_children():
		_banner_row.remove_child(c)
		c.queue_free()
	for ch in choices:
		var d: Dictionary = ch
		var key := String(d["key"])
		var b := DccWidgets.action(_banner_row, String(d["label"]), func(): banner_choice.emit(key))
		b.name = "Banner_" + key
		_touch_floor(b)
	banner.visible = true


## Hide the save banner.
func hide_banner() -> void:
	if banner != null:
		banner.visible = false


## The banner is a blocking state, in the accent ground the shell uses for a refusal, above the
## toolbar: read first, not hidden in the footer.
func _build_banner() -> void:
	banner = PanelContainer.new()
	banner.name = "SaveBanner"
	var sb := DccTheme.outline("accent", "panel")
	sb.set_content_margin_all(8)
	banner.add_theme_stylebox_override("panel", sb)
	var vb := VBoxContainer.new()
	vb.add_theme_constant_override("separation", 6)
	banner.add_child(vb)
	_banner_label = DccTheme.label("", "text_bright", DccTheme.FS_SMALL)
	_banner_label.name = "SaveBannerText"
	_banner_label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_banner_label.custom_minimum_size.x = 160
	_banner_label.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	vb.add_child(_banner_label)
	_banner_row = HFlowContainer.new()
	_banner_row.name = "SaveBannerChoices"
	_banner_row.add_theme_constant_override("h_separation", 6)
	_banner_row.add_theme_constant_override("v_separation", 6)
	vb.add_child(_banner_row)
	banner.visible = false
	add_child(banner)


## The status line: `refused` draws it in the accent. Used for "editing…" / "saved · time" and for
## the one-line reasons a save did not land.
func set_status(msg: String, refused: bool) -> void:
	if status_label == null:
		return
	status_label.text = msg
	status_label.add_theme_color_override("font_color",
		DccTheme.c("accent") if refused else DccTheme.c("text_ghost"))


## The current mode, `write`, `split` or `preview`.
func mode() -> String:
	return _mode


## Whether Split is on offer: never on the phone (one view at a time, s.4), and otherwise only while
## the editor is at least `SPLIT_MIN_DP` wide. Pure over the two inputs so the boundary can be pinned.
static func split_offered(phone: bool, width_dp: float) -> bool:
	return not phone and width_dp >= float(SPLIT_MIN_DP)


## Flip Write / Split / Preview (anything else means write; Split asked for while it is not on offer
## becomes Write, remembered in `_split_wanted` so a window that grows wide enough brings it back).
## Re-renders the BBCode when Preview is on screen, keeps focus on the visible pane only if the
## editor already had it, lights the matching segment, and disables every formatting button in
## Preview, where there is no selection to act on (Split keeps the source, so they stay live).
## Emits `mode_changed`.
func set_mode(m: String) -> void:
	var want := m if m in ["preview", "split"] else "write"
	_split_wanted = want == "split"
	if want == "split" and not split_offered(_phone, size.x):
		want = "write"
	if want == "preview" and _mode != "preview":
		_before_preview = _mode
	_mode = want
	var writing := _mode != "preview"
	## Ctrl+E is read from whichever of the two has focus, so the one on screen
	## must take it -- but only when the editor already had focus (asked before
	## hiding, which drops it); a rebuild must not steal focus from the window.
	var had_focus := text_edit.has_focus() or preview.has_focus()
	text_edit.visible = writing
	preview.visible = _mode != "write"
	if preview.visible:
		_render_preview()
	if not writing:
		_find_close(false)
	_hide_peek()
	_sg_hide()
	if had_focus:
		(text_edit if writing else preview).grab_focus.call_deferred()
	for k in ["write", "split", "preview"]:
		if buttons.has(k):
			DccWidgets.set_segment_on(buttons[k], k == _mode)
	if buttons.has("split"):
		(buttons["split"] as Button).visible = split_offered(_phone, size.x)
	## Formatting acts on the source; in Preview there is no selection to act on.
	for k in buttons:
		if k in ["write", "split", "preview"]:
			continue
		(buttons[k] as Button).disabled = not writing
	mode_changed.emit(_mode)


## Fills Preview from the source. The scroll position is kept, since Split re-renders as you type and
## a label that jumped to the top on every refresh would be unreadable.
func _render_preview() -> void:
	var bar := preview.get_v_scroll_bar()
	var keep := bar.value if bar != null else 0.0
	preview.text = _render_text()
	if bar != null and keep > 0.0:
		bar.set_deferred("value", keep)


## The hooks `to_bbcode` renders under, from the host's callables (VE-7). An embed needs a vault
## (`note_path` set, and `embed_source`); a task is clickable only with a toggler.
func _render_hooks() -> Dictionary:
	var h := {"open": _embeds_open, "stack": [note_path] if note_path != "" else []}
	if embed_source.is_valid() and note_path != "":
		h["embed"] = embed_source
		h["image"] = embed_image
	if chronos_source.is_valid():
		h["chronos"] = chronos_source
	if task_toggler.is_valid():
		h["ticks"] = true
	return h


## The source rendered as BBCode under the host's hooks; remembers which note embeds it drew, so a click on one
## (`embed:<n>`) names the right key.
func _render_text() -> String:
	var t := to_bbcode(text_edit.text, "", "", link_info, _render_hooks())
	_last_embed_keys = embed_keys.duplicate()
	return t


## Split's Preview follows the source a moment after typing stops.
func _refresh_split_preview() -> void:
	if _mode == "split":
		_render_preview()


## The editor was resized: Split appears or goes at exactly `SPLIT_MIN_DP`. A note in Split that is
## squeezed below it falls back to Write, and returns to Split when the room does.
func _on_resized() -> void:
	if text_edit == null or not buttons.has("split"):
		return
	var offered := split_offered(_phone, size.x)
	(buttons["split"] as Button).visible = offered
	if not offered and _mode == "split":
		var keep := _split_wanted
		set_mode("write")
		_split_wanted = keep
	elif offered and _split_wanted and _mode == "write":
		set_mode("split")


# -- Building ---------------------------------------------------------------

## Desktop: one row -- title, Write / Preview, Revert to opened…, Save. Phone: the title
## on its own line, because five things in one row do not fit 1080 px at phone
## density, and an `HBoxContainer` never wraps a sibling on its own.
func _build_header(file_label: String) -> void:
	var title := DccTheme.mono_label("EDITING  %s" % file_label, "text_dim", DccTheme.FS_SMALL, 1)
	title.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	## An `HFlowContainer` since VE-5: the side-panel toggle made the row wider than a 1000 px window's
	## editor column (Save was cut off); a flow wraps its last buttons to a second line instead.
	var row := HFlowContainer.new()
	row.name = "EditorHeader"
	_header_row = row
	row.add_theme_constant_override("h_separation", 8)
	row.add_theme_constant_override("v_separation", 6)
	if _phone:
		title.autowrap_mode = TextServer.AUTOWRAP_ARBITRARY
		title.custom_minimum_size.x = 120
		add_child(title)
		add_child(row)
		row.add_child(DccTheme.spacer())
	else:
		add_child(row)
		row.add_child(title)
	_back_button = DccWidgets.action(row, "‹ Back", func(): back_requested.emit())
	_back_button.name = "EditorBack"
	_back_button.tooltip_text = "Returns to the note you followed the last link from."
	_back_button.visible = false
	## Write / Preview as a lit segment pair -- the DCC vocabulary's "one of a
	## set is lit" control (`DccWidgets.segment`).
	buttons["write"] = DccWidgets.segment(row, "Write", func(): set_mode("write"))
	buttons["split"] = DccWidgets.segment(row, "Split", func(): set_mode("split"))
	buttons["split"].tooltip_text = "Write and Preview side by side (offered when the note pane is at least %d dp wide)." % SPLIT_MIN_DP
	buttons["split"].visible = false
	buttons["preview"] = DccWidgets.segment(row, "Preview", func(): set_mode("preview"))
	revert_button = DccWidgets.action(row, "Revert to opened…", func(): revert_requested.emit())
	revert_button.tooltip_text = "Replaces the note with how it was the first time you opened it this session. Asks first; Ctrl+Z brings your edits back."
	_touch_floor(revert_button)
	save_button = DccWidgets.action(row, "Save", func(): save_requested.emit(), true)
	_touch_floor(save_button)
	save_button.tooltip_text = "The note saves itself about a second after you stop typing. This writes it at once (Ctrl+S). Refuses, and writes nothing, if the file changed on disk since it was opened here."


## Build the formatting toolbar (one scrolling row on a phone, a wrapping flow on desktop) and the
## two pickers. Every button is registered in `buttons` by key and calls a thin `apply_*`; keep the
## keys in step with that map's doc, which the probe relies on.
func _build_toolbar() -> void:
	var bar: Container
	if _phone:
		## A horizontally scrolling row on the phone -- the ruling's "scrollable
		## row". Vertical scrolling is off, which folds the row's height into the
		## scroller (the height is wanted); horizontal is AUTO, so the row's
		## width never propagates up the tree (`MISTAKES.md`'s disabled-axis
		## row is about exactly that).
		var sc := ScrollContainer.new()
		sc.name = "EditorToolbarScroll"
		sc.vertical_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
		sc.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_AUTO
		sc.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		add_child(sc)
		var hb := HBoxContainer.new()
		hb.add_theme_constant_override("separation", 6)
		hb.name = "EditorToolbarRow"
		sc.add_child(hb)
		bar = hb
	else:
		var fl := HFlowContainer.new()
		fl.add_theme_constant_override("h_separation", 4)
		fl.add_theme_constant_override("v_separation", 4)
		add_child(fl)
		fl.name = "EditorToolbar"
		bar = fl

	_tool(bar, "bold", "B", "Bold (Ctrl+B) -- wraps the selection in **…**, or unwraps it.",
		func(): apply_inline("**", "**"))
	buttons["bold"].add_theme_font_override("font", FONT_BOLD)
	_tool(bar, "italic", "I", "Italic (Ctrl+I) -- *…*", func(): apply_inline("*", "*"))
	buttons["italic"].add_theme_font_override("font", FONT_ITALIC)
	_tool(bar, "underline", "U", "Underline (Ctrl+U) -- Markdown has none, so this writes <u>…</u>, which Obsidian renders.",
		func(): apply_inline("<u>", "</u>"))
	_style_glyph_chip(buttons["underline"], "[u]U[/u]")
	_tool(bar, "strike", "S", "Strikethrough -- ~~…~~", func(): apply_inline("~~", "~~"))
	_style_glyph_chip(buttons["strike"], "[s]S[/s]")
	_tool(bar, "highlight", "==H==", "Highlight -- ==…==, Obsidian's highlight.", func(): apply_inline("==", "=="))
	_sep(bar)
	## The "text size" picker. Markdown stores no font size, so size is the
	## heading level of the caret's line.
	_tool(bar, "h0", "Normal", "Body text -- removes the line's # prefix.", func(): apply_heading(0))
	_tool(bar, "h1", "H1", "Heading 1 -- the largest size.", func(): apply_heading(1))
	_tool(bar, "h2", "H2", "Heading 2", func(): apply_heading(2))
	_tool(bar, "h3", "H3", "Heading 3", func(): apply_heading(3))
	_sep(bar)
	_tool(bar, "bullet", "• List", "Bulleted list -- every selected line.", func(): apply_lines("bullet"))
	_tool(bar, "number", "1. List", "Numbered list -- every selected line.", func(): apply_lines("number"))
	_tool(bar, "task", "☐ Task", "Task -- writes - [ ] on every selected line, Obsidian's checkbox.", func(): apply_lines("task"))
	_tool(bar, "quote", "Quote", "Quote -- every selected line.", func(): apply_lines("quote"))
	_tool(bar, "callout", "[!note]", "Callout -- Obsidian's > [!note] block, around the selected lines.", func(): apply_callout())
	_sep(bar)
	_tool(bar, "code", "`code`", "Inline code -- `…`", func(): apply_inline("`", "`"))
	_tool(bar, "code_block", "```", "Code block -- fences the selected lines.", func(): apply_code_block())
	_tool(bar, "link", "[[ ]]", "Link -- writes [[ ]] at the caret (around the selection as the shown text) and suggests notes, headings and places as you type.", func(): apply_link())
	_tool(bar, "tag", "#", "Tag -- #tag. Obsidian's tags hold no spaces, so a selected phrase is hyphenated.", func(): apply_tag())
	_sep(bar)
	_tool(bar, "template", "Insert template", "Insert one of this vault's templates at the caret, as Obsidian's Insert template does: {{title}}, {{date}} and {{time}} filled, its properties merged into this note's.", func(): _open_template_picker())
	_sep(bar)
	_tool(bar, "find", "Find", "Find in this note (Ctrl+F). Ctrl+H, or Replace… in the bar, adds replace.", func(): _find_open_bar(false))

	_template_popup = PopupMenu.new()
	_template_popup.name = "EditorTemplatePicker"
	DccWidgets.style_popup(_template_popup)
	_template_popup.id_pressed.connect(func(id: int):
		if id >= 0 and id < _template_rels.size():
			insert_template(_template_rels[id]))
	add_child(_template_popup)


## Add one toolbar chip: a segment button with tooltip and a 28 px minimum width (a labelled
## judgement so one-letter chips stay a tappable target), registered in `buttons` under `key`.
func _tool(bar: Control, key: String, label_text: String, tip: String, cb: Callable) -> Button:
	var b := DccWidgets.segment(bar, label_text, cb)
	b.tooltip_text = tip
	b.custom_minimum_size.x = maxf(b.custom_minimum_size.x, 28)
	buttons[key] = b
	return b


## Draws a chip's glyph the way a word processor draws its own U/S buttons --
## underlined, struck through -- rather than a bare capital letter (Vault
## editor polish, `OUTSTANDING_WORK.md`). `DccWidgets.chip()`/`segment()` build
## a plain `Button`, and `Button.text` is not BBCode -- there is no Godot
## `Label`/`Button` property for "draw this text underlined" short of a custom
## font, so the glyph is drawn by a `RichTextLabel` overlaid on the button
## instead. The overlay's `mouse_filter = MOUSE_FILTER_IGNORE` sends every
## click straight through to the `Button` beneath it, so the press callback,
## the tooltip and the hit rect are all still the button's own -- accessible
## the same way every other chip is, `custom_minimum_size` untouched, no probe
## coordinate needs to move. The button's own text is kept (not cleared): it
## still sizes the chip and still carries the accessible name, only its ink is
## made transparent so the plain "U"/"S" it would otherwise draw does not show
## through beneath the styled glyph.
func _style_glyph_chip(b: Button, bbcode: String) -> void:
	var col: Color = b.get_theme_color("font_color")
	var transparent := Color(col.r, col.g, col.b, 0.0)
	b.add_theme_color_override("font_color", transparent)
	b.add_theme_color_override("font_hover_color", transparent)
	b.add_theme_color_override("font_disabled_color", transparent)
	var rt := RichTextLabel.new()
	rt.name = "GlyphOverlay"
	rt.bbcode_enabled = true
	rt.text = bbcode
	rt.fit_content = false
	rt.scroll_active = false
	rt.selection_enabled = false
	rt.focus_mode = Control.FOCUS_NONE
	rt.mouse_filter = Control.MOUSE_FILTER_IGNORE
	rt.set_anchors_preset(Control.PRESET_FULL_RECT)
	rt.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	rt.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	rt.add_theme_font_size_override("normal_font_size", b.get_theme_font_size("font_size"))
	rt.add_theme_color_override("default_color", col)
	b.add_child(rt)


## Add a thin vertical rule between toolbar groups.
func _sep(bar: Control) -> void:
	var r := DccTheme.rule(true)
	r.custom_minimum_size = Vector2(1, 18)
	bar.add_child(r)


## Build the source `TextEdit` (wrapping, shell field styling, caret and shortcut hooks) and the
## hidden `RichTextLabel` preview sharing its minimum height (360 px phone, 280 desktop -- labelled
## judgements). Edits emit `edited`; the preview is filled only when entering Preview.
func _build_body(initial: String) -> void:
	text_edit = TextEdit.new()
	text_edit.name = "EditorSource"
	text_edit.text = initial
	text_edit.wrap_mode = TextEdit.LINE_WRAPPING_BOUNDARY
	text_edit.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	text_edit.size_flags_vertical = Control.SIZE_EXPAND_FILL
	text_edit.custom_minimum_size.y = 360 if _phone else 280
	text_edit.placeholder_text = "Write Markdown here."
	## No grey slab: Godot's stock TextEdit draws a mid-grey block that reads as
	## a foreign widget on either palette. The shell's field vocabulary instead
	## -- `field_box`, the same ground `DccWidgets.well()` gives a LineEdit, at a
	## writing-surface padding -- with the prose face for prose.
	for st in ["normal", "focus", "read_only"]:
		text_edit.add_theme_stylebox_override(st, DccTheme.field_box(st, 14, 10))
	text_edit.add_theme_font_size_override("font_size", DccTheme.FS_BODY + 1)
	text_edit.add_theme_color_override("font_color", DccTheme.c("text_bright"))
	text_edit.add_theme_color_override("font_readonly_color", DccTheme.c("text_ghost"))
	text_edit.add_theme_color_override("font_placeholder_color", DccTheme.c("text_ghost"))
	text_edit.add_theme_color_override("caret_color", DccTheme.c("accent"))
	text_edit.add_theme_color_override("font_selected_color", DccTheme.c("text_bright"))
	text_edit.add_theme_color_override("selection_color", Color(DccTheme.c("accent"), 0.30))
	text_edit.add_theme_color_override("current_line_color", Color(0, 0, 0, 0))
	text_edit.add_theme_constant_override("line_spacing", 5)
	text_edit.text_changed.connect(_on_text_changed)
	text_edit.caret_changed.connect(_on_caret_changed)
	text_edit.focus_exited.connect(_sg_hide)
	text_edit.gui_input.connect(_on_source_input)
	## Write and Preview share one row so Split can show both (VE-3); each keeps its own visibility.
	_body = HBoxContainer.new()
	_body.name = "EditorBody"
	_body.add_theme_constant_override("separation", 10)
	_body.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	_body.size_flags_vertical = Control.SIZE_EXPAND_FILL
	add_child(_body)
	_body.add_child(text_edit)
	## The colouring. Links come from the host's crate-backed hook and the find bar's hits from here.
	_hl = Highlighter.new()
	_hl.link_source = func(line: String) -> Array:
		return link_spans.call(line) if link_spans.is_valid() else []
	_hl.match_source = _matches_in_line
	text_edit.syntax_highlighter = _hl
	## A text colour the highlighter does not override: the body role.
	text_edit.add_theme_color_override("font_color", _hl.color_of(Highlighter.BODY))
	text_edit.theme_changed.connect(_on_theme_changed)
	_last_len = initial.length()
	_build_suggest()

	preview = RichTextLabel.new()
	preview.name = "EditorPreview"
	preview.bbcode_enabled = true
	preview.selection_enabled = true
	preview.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	preview.size_flags_vertical = Control.SIZE_EXPAND_FILL
	preview.custom_minimum_size.y = text_edit.custom_minimum_size.y
	style_rendered(preview)
	var pv := DccTheme.field_box("read_only", 14, 10)
	preview.add_theme_stylebox_override("normal", pv)
	preview.visible = false
	preview.focus_mode = Control.FOCUS_ALL
	preview.gui_input.connect(_on_preview_input)
	## Classed links draw their own underline (a note link underlined, the others not), so the
	## label's blanket meta underline is off whenever the host classes links.
	preview.meta_underlined = not link_info.is_valid()
	preview.meta_clicked.connect(_on_meta_clicked)
	preview.meta_hover_started.connect(_on_meta_hover_started)
	preview.meta_hover_ended.connect(_on_meta_hover_ended)
	_body.add_child(preview)
	_build_peek()


## The rendered-Markdown look, shared with the vault window's read-only note
## pane so the preview and the page read the same.
static func style_rendered(r: RichTextLabel) -> void:
	r.add_theme_color_override("default_color", DccTheme.c("text"))
	r.add_theme_font_override("bold_font", FONT_BOLD)
	r.add_theme_font_override("italics_font", FONT_ITALIC)
	r.add_theme_font_override("bold_italics_font", FONT_BOLD)
	r.add_theme_font_override("mono_font", DccTheme.FONT_MONO)
	for k in ["normal_font_size", "bold_font_size", "italics_font_size", "bold_italics_font_size"]:
		r.add_theme_font_size_override(k, DccTheme.FS_BODY + 1)
	r.add_theme_font_size_override("mono_font_size", DccTheme.FS_SMALL)
	r.add_theme_constant_override("line_separation", 4)
	r.add_theme_color_override("selection_color", Color(DccTheme.c("accent"), 0.30))


# -- Selection plumbing -------------------------------------------------------

## Convert a (line, column) pair in `text_edit` into a flat character offset, counting one newline
## per line break. The inverse of `_line_col()`.
func _offset(line: int, col: int) -> int:
	var o := 0
	for i in range(line):
		o += text_edit.get_line(i).length() + 1
	return o + col


## Convert a flat offset into (line, column), the inverse of `_offset()`. An offset past the end
## lands on the last line.
static func _line_col(text: String, offset: int) -> Vector2i:
	var line := 0
	var start := 0
	var i := text.find("\n")
	while i >= 0 and i < offset:
		line += 1
		start = i + 1
		i = text.find("\n", start)
	return Vector2i(line, offset - start)


## The selection as `[from, to]` character offsets; the caret twice when
## nothing is selected.
func selection_offsets() -> Vector2i:
	if text_edit.has_selection():
		var a := _offset(text_edit.get_selection_from_line(), text_edit.get_selection_from_column())
		var b := _offset(text_edit.get_selection_to_line(), text_edit.get_selection_to_column())
		return Vector2i(mini(a, b), maxi(a, b))
	var c := _offset(text_edit.get_caret_line(), text_edit.get_caret_column())
	return Vector2i(c, c)


## Selects `[from, to]` (offsets) -- or places the caret, when they are equal.
func select_offsets(from: int, to: int) -> void:
	var a := _line_col(text_edit.text, from)
	var b := _line_col(text_edit.text, to)
	text_edit.deselect()
	text_edit.set_caret_line(a.x)
	text_edit.set_caret_column(a.y)
	if to != from:
		text_edit.select(a.x, a.y, b.x, b.y)
	_refresh_heading_lights()


## One undoable edit: the whole text swapped inside a complex operation, so
## Ctrl+Z takes back a toolbar action in one step.
func _commit(r: Dictionary) -> void:
	var t := String(r["text"])
	if t != text_edit.text:
		text_edit.begin_complex_operation()
		var last := text_edit.get_line_count() - 1
		text_edit.remove_text(0, 0, last, text_edit.get_line(last).length())
		text_edit.insert_text(t, 0, 0)
		text_edit.end_complex_operation()
	select_offsets(int(r["from"]), int(r["to"]))
	text_edit.grab_focus()


## Toolbar action: wrap or unwrap the selection in `open`/`close` through `wrap_inline()` as one
## undoable edit.
func apply_inline(open: String, close: String) -> void:
	var s := selection_offsets()
	_commit(wrap_inline(text_edit.text, s.x, s.y, open, close))


## Toolbar action: set the caret line's heading level (0 removes it) through `set_heading()`.
func apply_heading(level: int) -> void:
	var s := selection_offsets()
	_commit(set_heading(text_edit.text, s.x, s.y, level))


## Toolbar action: toggle a `bullet`, `number`, `task` or `quote` prefix on every selected line
## through `prefix_lines()`.
func apply_lines(kind: String) -> void:
	var s := selection_offsets()
	_commit(prefix_lines(text_edit.text, s.x, s.y, kind))


## Toolbar action: fence the selected lines as a code block through `code_block()`.
func apply_code_block() -> void:
	var s := selection_offsets()
	_commit(code_block(text_edit.text, s.x, s.y))


## Write a `[[note]]` link to the vault file `rel` at the selection through `note_link()`. Nothing in
## the shell calls it since VE-2 (the link button opens `[[` suggestions instead); the transform and
## this entry point stay because `_mdedit_probe.gd` pins `note_link()`.
func insert_note_link(rel: String) -> void:
	var s := selection_offsets()
	_commit(note_link(text_edit.text, s.x, s.y, rel))


## Toolbar action: wrap the selected lines in an Obsidian callout through `callout()`.
func apply_callout() -> void:
	var s := selection_offsets()
	_commit(callout(text_edit.text, s.x, s.y))


## Toolbar action: turn the selection into a `#tag` (spaces hyphenated) through `tag()`.
func apply_tag() -> void:
	var s := selection_offsets()
	_commit(tag(text_edit.text, s.x, s.y))


## Inserts the template at `rel` at the caret through the host's inserter --
## the engine fills it and merges its properties; this file only places the
## result, as one undoable edit. A selection is replaced, as typing over it
## would be.
func insert_template(rel: String) -> void:
	if not _template_inserter.is_valid():
		set_status("Insert template is not available here.", false)
		return
	var s := selection_offsets()
	var base := text_edit.text.substr(0, s.x) + text_edit.text.substr(s.y)
	var r: Dictionary = _template_inserter.call(rel, base, s.x)
	if not bool(r.get("ok", false)):
		set_status("Insert template: %s" % String(r.get("error", "refused")), false)
		return
	var c := int(r.get("caret", s.x))
	_commit({"text": String(r.get("text", base)), "from": c, "to": c})
	set_status("Inserted %s -- not saved yet." % rel.get_file(), false)


## The picker's rows, `[{rel, label}]` exactly as offered.
func template_choices() -> Array:
	return _templates_source.call() if _templates_source.is_valid() else []


## Fill and show the Insert-template popup from `template_choices()`, with a disabled placeholder
## row when the vault has none. Remembers the rel paths in `_template_rels` for the id-to-path
## lookup.
func _open_template_picker() -> void:
	var ts := template_choices()
	_template_popup.clear()
	_template_rels = PackedStringArray()
	if ts.is_empty():
		_template_popup.add_item("No templates in this vault")
		_template_popup.set_item_disabled(0, true)
	for i in ts.size():
		var d: Dictionary = ts[i]
		_template_rels.append(String(d.get("rel", "")))
		_template_popup.add_item(String(d.get("label", d.get("rel", "?"))), i)
	DccWidgets.popup_anchored(_template_popup, anchor_rect(buttons["template"]), 320)


## `c`'s rect in the space its popups open in. A popup under an embedded
## window (the vault window is one) is embedded one level up, beside that
## window, so the window's own position has to be added -- a bare
## `get_global_rect()` is window-local and put both pickers up and to the left
## of their buttons by exactly the window's offset (seen in `_vaultbf_probe`'s
## screenshot, 2026-09-27).
static func anchor_rect(c: Control) -> Rect2:
	var r := c.get_global_rect()
	var w := c.get_window()
	if w != null and w.is_embedded():
		r.position += Vector2(w.position)
	return r


## Shows or hides the header's Back button (VE-1). The host owns the history; this is only the
## button.
func set_back_visible(on: bool) -> void:
	if _back_button != null:
		_back_button.visible = on


## Re-renders Preview from the source with the current `link_info` (a host whose answers changed,
## say after a note was created, calls this). No effect in Write.
func refresh_preview() -> void:
	if preview != null and _mode == "preview":
		preview.text = _render_text()


# -- Links: follow and peek (VE-1) ---------------------------------------------

## A rendered link was clicked. The meta is `wiki:<target>` or `ext:<url>` (see `_inline_text`);
## anything else -- a plain `[url=...]` from a caller that did not class links -- is ignored rather
## than guessed at. A click that ends a finger's peek hold is swallowed once.
func _on_meta_clicked(meta: Variant) -> void:
	if _swallow_click:
		_swallow_click = false
		return
	var m := str(meta)
	_hide_peek()
	if m.begins_with("task:"):
		_tick_task(int(m.substr(5)))
	elif m.begins_with("year:"):
		chronos_year_requested.emit(int(m.substr(5)))
	elif m.begins_with("embed:"):
		var n := int(m.substr(6))
		if n >= 0 and n < _last_embed_keys.size():
			var key := String(_last_embed_keys[n])
			_embeds_open[key] = not bool(_embeds_open.get(key, false))
			_render_preview()
	elif m.begins_with("wiki:"):
		link_followed.emit(m.substr(5))
	elif m.begins_with("ext:"):
		external_link_followed.emit(m.substr(4))


## Ticks or unticks the task on 0-based `line` of the source, from a click on its box in Preview (VE-7). The
## crate decides what a task is (`task_toggler`); the text goes back as ONE undo step through `replace_whole_text`,
## so Ctrl+Z un-ticks it, and the host's autosave writes it like any edit. A line that is no longer a task (the
## text moved since the render) is left alone and Preview is simply redrawn.
func _tick_task(line: int) -> void:
	if task_toggler.is_valid():
		var r: Dictionary = task_toggler.call(text_edit.text, line)
		if bool(r.get("ok", false)):
			replace_whole_text(String(r["text"]))
	_render_preview()


## The pointer (or a held finger, see `_on_preview_input`) is over a link: start the dwell that
## shows the peek. Only `wiki:` links peek; an external address has nothing in the vault to show.
func _on_meta_hover_started(meta: Variant) -> void:
	var m := str(meta)
	if not m.begins_with("wiki:") or not peek_source.is_valid():
		return
	_peek_target = m.substr(5)
	_peek_timer.start(PEEK_HOLD_SEC if _peek_by_hold else PEEK_HOVER_SEC)


func _on_meta_hover_ended(_meta: Variant) -> void:
	if _peek_by_hold:
		return
	_hide_peek()


func _build_peek() -> void:
	_peek_timer = Timer.new()
	_peek_timer.one_shot = true
	_peek_timer.timeout.connect(_show_peek)
	add_child(_peek_timer)
	_peek = PanelContainer.new()
	_peek.name = "LinkPeek"
	_peek.top_level = true
	_peek.visible = false
	_peek.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_peek.custom_minimum_size = Vector2(300, 0)
	## A hairline-outlined card on the panel ground, so it reads as floating above the note rather than
	## as more of it (a `field_box` here was the same colour as the page and showed no edge).
	var card := DccTheme.panel("panel", {"left": 1, "right": 1, "top": 1, "bottom": 1})
	card.content_margin_left = 12
	card.content_margin_right = 12
	card.content_margin_top = 8
	card.content_margin_bottom = 8
	_peek.add_theme_stylebox_override("panel", card)
	_peek_label = RichTextLabel.new()
	_peek_label.bbcode_enabled = true
	_peek_label.fit_content = true
	_peek_label.scroll_active = false
	_peek_label.mouse_filter = Control.MOUSE_FILTER_IGNORE
	style_rendered(_peek_label)
	_peek.add_child(_peek_label)
	add_child(_peek)


## Fills and places the peek card beside the pointer, clamped to the editor's own rect so it never
## leaves the window. Nothing to show (`peek_source` returned `""`) shows nothing.
func _show_peek() -> void:
	if _peek_target == "" or not peek_source.is_valid():
		return
	var body := String(peek_source.call(_peek_target))
	if body == "":
		return
	_peek_label.text = body
	## A fit-content label reports its height only after a layout pass at its real width, and a
	## top-level panel keeps whatever size it was last given (seen in the first capture: a card
	## stretched to the window's foot). So: show it transparent, let one frame lay it out, shrink it
	## to its content, then place it and make it opaque.
	_peek.modulate.a = 0.0
	_peek.size = Vector2(300, 0)
	_peek.visible = true
	await get_tree().process_frame
	if not _peek.visible or _peek_target == "":
		return
	_peek.reset_size()
	var r := get_global_rect()
	var at := get_global_mouse_position() + Vector2(14, 18)
	at.x = clampf(at.x, r.position.x, maxf(r.position.x, r.end.x - _peek.size.x))
	at.y = clampf(at.y, r.position.y, maxf(r.position.y, r.end.y - _peek.size.y))
	_peek.global_position = at
	_peek.modulate.a = 1.0


func _hide_peek() -> void:
	if _peek_timer != null:
		_peek_timer.stop()
	if _peek != null:
		_peek.visible = false
	_peek_target = ""
	_peek_by_hold = false


# -- `[[` suggestions (VE-2) ---------------------------------------------------

## Where the caret sits relative to an open `[[` on one line, as a pure function (probe-pinned):
## `{}` when it is not inside one, else `{mode, open, qstart, note, query}`. `open` is the column just
## after `[[`; `mode` is `note` (the text after `[[` is the query, `qstart == open`), `heading` (after
## `Note#`: `note` is the target written before the `#`, `qstart` the column after it) or `alias`
## (after `|`, or after `^` for a block: free text, nothing to suggest). A `]` between `[[` and the
## caret closes the link, so a caret after a finished link is outside it.
static func suggest_context(line: String, col: int) -> Dictionary:
	var prefix := line.substr(0, col)
	var i := prefix.rfind("[[")
	if i < 0:
		return {}
	var inner := prefix.substr(i + 2)
	if inner.contains("]") or inner.contains("["):
		return {}
	var open := i + 2
	if inner.contains("|") or inner.contains("^"):
		return {"mode": "alias", "open": open, "qstart": col, "note": inner, "query": ""}
	var h := inner.find("#")
	if h >= 0:
		return {"mode": "heading", "open": open, "qstart": open + h + 1,
			"note": inner.substr(0, h), "query": inner.substr(h + 1)}
	return {"mode": "note", "open": open, "qstart": open, "note": "", "query": inner}


## Writes a chosen suggestion into `line` for the context `ctx` (from `suggest_context`) with the
## caret at `col`: `{line, caret}`. The target (`insert`, plus `|alias` when `alias` is given and the
## link has no shown text yet) replaces everything from `qstart` up to the link's own `|` or `]]`; a
## `]]` already after the caret (the auto-pair's, or a link being edited) is kept and the caret goes
## after it, otherwise `]]` is added. Never touches another line.
static func suggest_apply(line: String, col: int, ctx: Dictionary, insert: String, alias: String = "") -> Dictionary:
	var qstart := int(ctx["qstart"])
	var after := line.substr(col)
	var c := after.find("]]")
	var mid := insert
	if c >= 0 and not after.substr(0, c).contains("[["):
		var pipe := after.substr(0, c).find("|")
		var seg_end := col + (pipe if pipe >= 0 else c)
		if pipe < 0 and alias != "" and String(ctx["mode"]) == "note":
			mid += "|" + alias
		var closer_end := col + c + 2
		return {"line": line.substr(0, qstart) + mid + line.substr(seg_end),
			"caret": qstart + mid.length() + (closer_end - seg_end)}
	if alias != "" and String(ctx["mode"]) == "note":
		mid += "|" + alias
	return {"line": line.substr(0, qstart) + mid + "]]" + after, "caret": qstart + mid.length() + 2}


## The link button's transform: no selection writes `[[]]` with the caret between; a one-line
## selection becomes the link's shown text, `[[|selected]]`, with the caret before the `|`.
## A selection that spans lines is left alone and `[[]]` is written at its start (an alias cannot
## hold a line break).
static func link_wrap(text: String, from: int, to: int) -> Dictionary:
	var sel := text.substr(from, to - from)
	if from == to or sel.contains("\n"):
		return {"text": text.substr(0, from) + "[[]]" + text.substr(from), "from": from + 2, "to": from + 2}
	return {"text": text.substr(0, from) + "[[|" + sel + "]]" + text.substr(to), "from": from + 2, "to": from + 2}


## Toolbar action: write a link (see `link_wrap`) and open its suggestions.
func apply_link() -> void:
	var s := selection_offsets()
	_commit(link_wrap(text_edit.text, s.x, s.y))


func _sg_limit() -> int:
	return SUGGEST_ROWS_PHONE if _phone else SUGGEST_ROWS


## Whether the card is on screen.
func _sg_visible() -> bool:
	return _sg_panel != null and _sg_panel.visible


## The card: a hairline panel, top-level so it floats over the note and is placed in window
## coordinates. Created hidden; rows are rebuilt on every keystroke by `_sg_render`.
func _build_suggest() -> void:
	_sg_panel = PanelContainer.new()
	_sg_panel.name = "EditorSuggest"
	_sg_panel.top_level = true
	_sg_panel.visible = false
	_sg_panel.mouse_filter = Control.MOUSE_FILTER_STOP
	var card := DccTheme.panel("panel", {"left": 1, "right": 1, "top": 1, "bottom": 1})
	card.content_margin_left = 4
	card.content_margin_right = 4
	card.content_margin_top = 4
	card.content_margin_bottom = 4
	_sg_panel.add_theme_stylebox_override("panel", card)
	_sg_box = VBoxContainer.new()
	_sg_box.add_theme_constant_override("separation", 0)
	_sg_panel.add_child(_sg_box)
	add_child(_sg_panel)


## Typing: a lone `[` that made `[[` auto-pairs to `[[]]` (caret kept between), then the card is
## brought up to date. Paste and every other multi-character edit change the length by more than
## one and are left alone, as is a third `[`.
func _on_text_changed() -> void:
	edited.emit()
	var n := text_edit.text.length()
	var typed_one := n == _last_len + 1
	_last_len = n
	_hl.refresh()
	_footer_timer.start()
	if _mode == "split":
		_split_timer.start()
	if _find_open:
		_find_refresh(false)
	if _mode == "preview":
		return
	if typed_one and not text_edit.has_selection():
		var cl := text_edit.get_caret_line()
		var col := text_edit.get_caret_column()
		var line := text_edit.get_line(cl)
		if col >= 2 and line.substr(col - 2, 2) == "[[" and not (col >= 3 and line.substr(col - 3, 1) == "[") \
				and line.substr(col, 2) != "]]":
			text_edit.insert_text_at_caret("]]")
			text_edit.set_caret_column(col)
			_last_len += 2
		elif unpairs_third_star(line, col):
			## `***` (a rule, or bold-italic) typed after `**|**` took its pair: the third `*` removes the
			## closers this editor added, leaving exactly the three stars the user typed.
			text_edit.remove_text(cl, col, cl, col + 2)
			_last_len -= 2
		elif overtypes_star(line, col):
			## The closing `*` of a pair this editor made is typed OVER, not added (as Obsidian's
			## closing bracket is): remove the character just typed and step past the existing one.
			text_edit.remove_text(cl, col - 1, cl, col)
			text_edit.set_caret_column(col)
			_last_len -= 1
		elif should_pair_bold(line, col):
			## `**` pairs like `[[` does (s.3.3): the caret stays between, and typing the closer over it
			## is left alone by `should_pair_bold` (an opener earlier on the line makes this a closer).
			text_edit.insert_text_at_caret("**")
			text_edit.set_caret_column(col)
			_last_len += 2
	_sg_update(true)


func _on_caret_changed() -> void:
	_refresh_heading_lights()
	_sg_update(false)


## Re-reads the context at the caret and refreshes (or closes) the card. `from_edit` is true for a
## typed or written change, the only thing that opens the card; a caret move only keeps or closes an
## open one, so clicking into a finished link does not pop suggestions up over it.
func _sg_update(from_edit: bool) -> void:
	if text_edit == null or _mode == "preview" or not suggest_source.is_valid():
		_sg_hide()
		return
	if not from_edit and not _sg_active:
		return
	var ctx := suggest_context(text_edit.get_line(text_edit.get_caret_line()), text_edit.get_caret_column())
	if ctx.is_empty() or String(ctx["mode"]) == "alias":
		_sg_hide()
		return
	var items: Array = suggest_source.call(String(ctx["mode"]), String(ctx["note"]), String(ctx["query"]), _sg_limit())
	if items.is_empty():
		_sg_hide()
		return
	if items.size() > _sg_limit():
		items = items.slice(0, _sg_limit())
	_sg_ctx = ctx
	_sg_active = true
	_sg_items = items
	_sg_index = 0
	_sg_render()
	_sg_place()


func _sg_hide() -> void:
	_sg_active = false
	_sg_items = []
	if _sg_panel != null:
		_sg_panel.visible = false
	_esc_hold("suggest", false)


## Takes (`on`) or gives back the window's own Escape for `key`. The vault window is an
## `AcceptDialog`, and Escape would close it before the editor saw the key, so the suggestion card and
## the find bar each hold it while open; it is given back only when no one holds it.
func _esc_hold(key: String, on: bool) -> void:
	var w := get_window()
	if w == null or not ("dialog_close_on_escape" in w):
		return
	if on:
		if _esc_owners.is_empty():
			w.set("dialog_close_on_escape", false)
		_esc_owners[key] = true
	elif _esc_owners.erase(key) and _esc_owners.is_empty():
		w.set("dialog_close_on_escape", true)


## An editor freed with the card or the bar open must not leave the window deaf to Escape.
func _exit_tree() -> void:
	for k in _esc_owners.keys():
		_esc_hold(String(k), false)


func _sg_row_px() -> int:
	return SUGGEST_ROW_TOUCH_PX if (_touch_seen or _phone) else SUGGEST_ROW_PX


## Row text by kind: a note is its name and where it lives; an entity carries the pin glyph the
## link renderer uses for "a place with no note"; a heading a `#`; the new-note row a `+`.
static func _sg_text(it: Dictionary) -> String:
	var label := String(it.get("label", ""))
	var detail := String(it.get("detail", ""))
	var t := label
	match String(it.get("kind", "")):
		"entity":
			t = "⌖ " + label
		"heading":
			t = "# " + label
		"create":
			t = "+ " + label
	if detail != "":
		t += "   · " + detail
	return t


func _sg_render() -> void:
	for c in _sg_box.get_children():
		_sg_box.remove_child(c)
		c.queue_free()
	var px := _sg_row_px()
	for i in _sg_items.size():
		var it: Dictionary = _sg_items[i]
		var b := Button.new()
		b.name = "SuggestRow%d" % i
		## Not `flat`: a flat button draws no stylebox, and the highlight IS a stylebox.
		b.focus_mode = Control.FOCUS_NONE
		b.alignment = HORIZONTAL_ALIGNMENT_LEFT
		b.clip_text = true
		b.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
		b.custom_minimum_size = Vector2(0, px)
		b.text = _sg_text(it)
		b.disabled = not bool(it.get("enabled", true))
		b.add_theme_font_size_override("font_size", DccTheme.FS_BODY)
		for col in ["font_color", "font_hover_color", "font_pressed_color", "font_focus_color"]:
			b.add_theme_color_override(col, DccTheme.c("text_bright"))
		b.add_theme_color_override("font_disabled_color", DccTheme.c("text_ghost"))
		b.pressed.connect(_sg_accept.bind(i))
		b.mouse_entered.connect(_sg_hover.bind(i))
		_sg_box.add_child(b)
	_sg_style()


## Marks the highlighted row (the one Enter takes).
func _sg_style() -> void:
	for i in _sg_box.get_child_count():
		var b := _sg_box.get_child(i) as Button
		var on := i == _sg_index
		for st in ["normal", "hover", "pressed", "disabled"]:
			b.add_theme_stylebox_override(st, DccTheme.flat(Color(DccTheme.c("accent"), 0.24 if on else 0.0), 4))


func _sg_hover(i: int) -> void:
	if i != _sg_index and i < _sg_items.size():
		_sg_index = i
		_sg_style()


## Sizes the card from its row count (no layout frame to wait for) and puts it under the caret,
## flipping above when it would run past the bottom of the window (less the soft keyboard on the
## phone), clamped sideways into the window.
func _sg_place() -> void:
	var w := clampi(int(text_edit.size.x) - 16, 200, 400)
	var h := _sg_items.size() * _sg_row_px() + 10
	_sg_panel.custom_minimum_size = Vector2(w, h)
	_sg_panel.size = Vector2(w, h)
	var rect := Rect2(text_edit.get_rect_at_line_column(text_edit.get_caret_line(), text_edit.get_caret_column()))
	if rect.position.x < 0:
		rect = Rect2(text_edit.get_caret_draw_pos(), Vector2(0, 20))
	var base := text_edit.get_global_rect().position + rect.position
	var vp := get_viewport().get_visible_rect()
	var bottom := vp.end.y
	if _phone:
		bottom -= float(DisplayServer.virtual_keyboard_get_height()) / maxf(get_window().content_scale_factor, 0.01)
	var y := base.y + rect.size.y + 4
	if y + h > bottom:
		y = maxf(vp.position.y + 2, base.y - h - 4)
	_sg_panel.global_position = Vector2(clampf(base.x, vp.position.x + 4, maxf(vp.position.x + 4, vp.end.x - w - 4)), y)
	_sg_panel.visible = true
	_esc_hold("suggest", true)


## Writes row `i` as the link (see `suggest_apply`) in ONE undo step, closes the card and returns
## focus to the text. A disabled row is a statement ("no headings"), not a choice, and does nothing.
func _sg_accept(i: int) -> void:
	if i < 0 or i >= _sg_items.size():
		return
	var it: Dictionary = _sg_items[i]
	if not bool(it.get("enabled", true)):
		return
	var cl := text_edit.get_caret_line()
	var line := text_edit.get_line(cl)
	var res := suggest_apply(line, text_edit.get_caret_column(), _sg_ctx, String(it.get("insert", "")), String(it.get("alias", "")))
	_sg_hide()
	text_edit.begin_complex_operation()
	text_edit.remove_text(cl, 0, cl, line.length())
	text_edit.insert_text(String(res["line"]), cl, 0)
	text_edit.set_caret_line(cl)
	text_edit.set_caret_column(int(res["caret"]))
	text_edit.end_complex_operation()
	_last_len = text_edit.text.length()
	text_edit.grab_focus()


## Keys for the open card, and the pair-delete: Up / Down move, Enter or Tab accept, Escape closes the
## card. Backspace between an empty `[[|]]` removes the whole pair. Also notes whether the last
## pointer was a finger. True when the event was used.
func _sg_input(ev: InputEvent) -> bool:
	var mb := ev as InputEventMouseButton
	if mb != null and mb.pressed:
		_touch_seen = mb.device == InputEvent.DEVICE_ID_EMULATION
	elif ev is InputEventScreenTouch:
		_touch_seen = true
	var k := ev as InputEventKey
	if k == null or not k.pressed or k.ctrl_pressed or k.meta_pressed or k.alt_pressed:
		return false
	if _sg_visible():
		match k.keycode:
			KEY_DOWN, KEY_UP:
				_sg_index = posmod(_sg_index + (1 if k.keycode == KEY_DOWN else -1), _sg_items.size())
				_sg_style()
				return true
			KEY_ENTER, KEY_KP_ENTER, KEY_TAB:
				if not k.echo:
					_sg_accept(_sg_index)
				return true
			KEY_ESCAPE:
				_sg_hide()
				return true
	if k.keycode == KEY_BACKSPACE and not text_edit.has_selection():
		var cl := text_edit.get_caret_line()
		var col := text_edit.get_caret_column()
		var line := text_edit.get_line(cl)
		if col >= 2 and line.substr(col - 2, 2) == "[[" and line.substr(col, 2) == "]]":
			text_edit.remove_text(cl, col - 2, cl, col + 2)
			text_edit.set_caret_column(col - 2)
			return true
		if col >= 2 and line.substr(col - 2, 2) == "**" and line.substr(col, 2) == "**":
			## An empty `**|**` pair goes whole, as an empty `[[|]]` does.
			text_edit.remove_text(cl, col - 2, cl, col + 2)
			text_edit.set_caret_column(col - 2)
			return true
	return false


## Ctrl+E from the rendered view goes back to the source (Obsidian's
## reading-view toggle is the same key both ways). A finger pressed on the rendered text also
## arrives here as an emulated left button: a hold on a link is the touch peek, so the press sends
## the one motion Godot would otherwise never produce for a still finger (`PEEK_HOLD_SEC`).
func _on_preview_input(ev: InputEvent) -> void:
	var mb := ev as InputEventMouseButton
	if mb != null and mb.button_index == MOUSE_BUTTON_LEFT:
		if mb.pressed and mb.device == InputEvent.DEVICE_ID_EMULATION:
			_peek_by_hold = true
			var mm := InputEventMouseMotion.new()
			mm.position = mb.global_position
			mm.global_position = mb.global_position
			mm.relative = Vector2(1, 0)
			preview.get_viewport().push_input.call_deferred(mm)
		elif not mb.pressed and _peek_by_hold:
			## The hold ends. If it had produced the card, the release must not also follow the link.
			_swallow_click = _peek != null and _peek.visible
			_hide_peek()
			if _swallow_click:
				get_tree().create_timer(0.2).timeout.connect(func(): _swallow_click = false)
	var k := ev as InputEventKey
	if k == null or not k.pressed or k.echo:
		return
	if (k.ctrl_pressed or k.meta_pressed) and not k.alt_pressed and not k.shift_pressed and k.keycode == KEY_E:
		set_mode(_before_preview)
		preview.accept_event()


## Keyboard shortcuts in Write mode: Ctrl/Cmd+E flips to Preview, +B bold, +I italic, +U underline,
## +K link, +L task, +S save, +F find, +H find and replace; Enter and Tab are the smart keys
## (`_smart_key`). Any other key, any Alt or Shift combination, and key repeats fall through
## untouched. Accepts the event only for the keys it handled.
func _on_source_input(ev: InputEvent) -> void:
	if _sg_input(ev):
		text_edit.accept_event()
		return
	var mb := ev as InputEventMouseButton
	if mb != null and mb.pressed and mb.button_index == MOUSE_BUTTON_LEFT \
			and (mb.ctrl_pressed or mb.meta_pressed) and link_at.is_valid():
		## Ctrl/Cmd-click follows the link under the pointer (VE-1; Obsidian's gesture in its source
		## view). The caret has not moved yet, so the position is read from the click itself.
		var line := text_edit.get_line_column_at_pos(Vector2i(mb.position))
		var info: Dictionary = link_at.call(text_edit.text, _offset(line.y, line.x))
		if bool(info.get("found", false)) and info.has("target") and not bool(info.get("external", false)):
			var t := String(info["target"])
			if info.has("heading"):
				t += "#" + String(info["heading"])
			link_followed.emit(t)
			text_edit.accept_event()
		return
	var k := ev as InputEventKey
	if k == null or not k.pressed:
		return
	if not (k.ctrl_pressed or k.meta_pressed) and not k.alt_pressed and _smart_key(k):
		text_edit.accept_event()
		return
	if k.echo:
		return
	if not (k.ctrl_pressed or k.meta_pressed) or k.alt_pressed or k.shift_pressed:
		return
	match k.keycode:
		KEY_E:
			set_mode("preview")
		KEY_B:
			apply_inline("**", "**")
		KEY_I:
			apply_inline("*", "*")
		KEY_U:
			apply_inline("<u>", "</u>")
		KEY_K:
			apply_link()
		KEY_L:
			apply_lines("task")
		KEY_S:
			## An explicit save, the user's own: this file still writes nothing (the host answers).
			save_requested.emit()
		KEY_F:
			_find_open_bar(false)
		KEY_H:
			_find_open_bar(true)
		_:
			return
	text_edit.accept_event()


## Light the Normal/H1/H2/H3 segment matching the heading level at the caret. Safe before the
## toolbar exists. H4 to H6 light nothing, since the picker has no such size.
func _refresh_heading_lights() -> void:
	if text_edit == null or not buttons.has("h0"):
		return
	var lvl := heading_level_at(text_edit.text, selection_offsets().x)
	## H4-H6 light nothing: the picker offers three levels, and lighting H3
	## for an H5 line would claim a size the line does not have.
	for i in range(4):
		DccWidgets.set_segment_on(buttons["h%d" % i], i == lvl)


# -- The transforms (pure) ----------------------------------------------------

## The run of `ch` ending just before `pos` (backwards) -- `**x` at pos 2 is 2.
static func _run_before(text: String, pos: int, ch: String) -> int:
	var n := 0
	while pos - n - 1 >= 0 and text[pos - n - 1] == ch:
		n += 1
	return n


## The run of `ch` starting at `pos` (forwards); the mirror of `_run_before()`.
static func _run_after(text: String, pos: int, ch: String) -> int:
	var n := 0
	while pos + n < text.length() and text[pos + n] == ch:
		n += 1
	return n


## Whether a marker made of one repeated character (`*`, `**`, `~~`, `` ` ``)
## is present given the runs on each side. A single `*` next to `**` is not
## italic -- it is the edge of a bold -- so a one-character marker needs an odd
## run (`*x*`, `***x***`); a two-character marker needs at least two.
static func _run_holds(run_a: int, run_b: int, k: int) -> bool:
	if run_a < k or run_b < k:
		return false
	if k == 1:
		return run_a % 2 == 1 and run_b % 2 == 1
	return true


## Whether `marker` is one character repeated (`*`, `**`, `~~`), which selects the run-counting
## toggle in `wrap_inline()` rather than a plain substring test; false for an empty marker.
static func _is_repeat(marker: String) -> bool:
	if marker.is_empty():
		return false
	for i in marker.length():
		if marker[i] != marker[0]:
			return false
	return true


## Wraps `[from, to]` in `open`…`close`; toggles it off when the selection is
## already wrapped (markers just outside it, or at its own two ends). With no
## selection it inserts the pair and puts the caret between -- and pressing
## again with the caret still between an empty pair removes it.
static func wrap_inline(text: String, from: int, to: int, open: String, close: String) -> Dictionary:
	if from > to:
		var t0 := from
		from = to
		to = t0
	var sel := text.substr(from, to - from)
	var ol := open.length()
	var cl := close.length()
	var repeat := _is_repeat(open) and open == close
	# 1. markers just outside the selection
	var outside := false
	if from >= ol and to + cl <= text.length():
		if repeat:
			outside = _run_holds(_run_before(text, from, open[0]), _run_after(text, to, open[0]), ol)
		else:
			outside = text.substr(from - ol, ol) == open and text.substr(to, cl) == close
	if outside:
		var t := text.substr(0, from - ol) + sel + text.substr(to + cl)
		return {"text": t, "from": from - ol, "to": to - ol}
	# 2. markers at the selection's own ends
	if sel.length() >= ol + cl:
		var inside := false
		if repeat:
			inside = _run_holds(_run_after(sel, 0, open[0]), _run_before(sel, sel.length(), open[0]), ol) \
				and sel.length() > ol + cl
		else:
			inside = sel.begins_with(open) and sel.ends_with(close)
		if inside:
			var inner := sel.substr(ol, sel.length() - ol - cl)
			return {"text": text.substr(0, from) + inner + text.substr(to),
				"from": from, "to": from + inner.length()}
	# 3. wrap
	return {"text": text.substr(0, from) + open + sel + close + text.substr(to),
		"from": from + ol, "to": to + ol}


## Offset of the first character of the line holding `pos` (0 for `pos <= 0`).
static func _line_start(text: String, pos: int) -> int:
	if pos <= 0:
		return 0
	var i := text.rfind("\n", pos - 1)
	return i + 1


## Offset of the newline ending the line holding `pos`, or the text length on the last line.
static func _line_end(text: String, pos: int) -> int:
	var i := text.find("\n", pos)
	return text.length() if i < 0 else i


## The ATX prefix length of `line` (`"## "` -> 3), 0 when it has none.
static func _heading_prefix(line: String) -> Vector2i:
	var n := 0
	while n < line.length() and n < 6 and line[n] == "#":
		n += 1
	if n == 0:
		return Vector2i(0, 0)
	if n == line.length():
		return Vector2i(n, n)
	if line[n] == " ":
		var m := n
		while m < line.length() and line[m] == " ":
			m += 1
		return Vector2i(n, m)
	return Vector2i(0, 0)


## The heading level of the line holding `pos` (0 = body text).
static func heading_level_at(text: String, pos: int) -> int:
	var ls := _line_start(text, pos)
	return _heading_prefix(text.substr(ls, _line_end(text, ls) - ls)).x


## Rewrites the `#` prefix of the line holding `from`. Level 0 removes it.
static func set_heading(text: String, from: int, to: int, level: int) -> Dictionary:
	var ls := _line_start(text, from)
	var le := _line_end(text, ls)
	var line := text.substr(ls, le - ls)
	var old_p := _heading_prefix(line).y
	var body := line.substr(old_p)
	var new_prefix := "" if level <= 0 else "#".repeat(clampi(level, 1, 6)) + " "
	var t := text.substr(0, ls) + new_prefix + body + text.substr(le)
	var d := new_prefix.length() - old_p
	var nf := from + d if from >= ls + old_p else ls + new_prefix.length()
	var nt := to
	if to <= le:
		nt = to + d if to >= ls + old_p else ls + new_prefix.length()
	else:
		nt = to + d
	return {"text": t, "from": nf, "to": nt}


## Regexes (as strings, compiled where used) that recognise an existing list or quote prefix, so
## `prefix_lines()` can toggle or swap it. The leading group keeps indentation. `_TASK_RE` below
## must be tried before `_BULLET_RE`, since a task also begins with a bullet marker.
const _BULLET_RE := "^(\\s*)[-*+] "
## Regex string for an ordered-list prefix (`1.` or `1)`), compiled in `prefix_lines`.
const _NUMBER_RE := "^(\\s*)\\d+[.)] "
## Regex string for a block-quote prefix (`>` and an optional space), compiled in `prefix_lines`.
const _QUOTE_RE := "^> ?"
## Obsidian's task: a list marker, then a one-character box -- `[ ]` open, any
## other character (`[x]`, `[/]`, …) a status.
const _TASK_RE := "^(\\s*)[-*+] \\[.\\] "


## Toggles a list or quote prefix on every line `[from, to]` touches. Blank
## lines are left alone. If every non-blank line already carries the prefix it
## is removed; otherwise it is added (a bullet list becomes numbered rather
## than doubly marked). Returns the affected lines as the new selection.
static func prefix_lines(text: String, from: int, to: int, kind: String) -> Dictionary:
	if from > to:
		var t0 := from
		from = to
		to = t0
	var ls := _line_start(text, from)
	var end_pos := to
	## A selection that ends at the very start of a line does not include it.
	if to > from and to > 0 and text[to - 1] == "\n":
		end_pos = to - 1
	var le := _line_end(text, end_pos)
	var block := text.substr(ls, le - ls)
	var lines := block.split("\n")
	var bullet := RegEx.create_from_string(_BULLET_RE)
	var number := RegEx.create_from_string(_NUMBER_RE)
	var quote := RegEx.create_from_string(_QUOTE_RE)
	var task := RegEx.create_from_string(_TASK_RE)
	var mine: RegEx = {"bullet": bullet, "number": number, "quote": quote, "task": task}[kind]
	var all_have := true
	var any := false
	for l in lines:
		if String(l).strip_edges() == "":
			continue
		any = true
		if mine.search(String(l)) == null:
			all_have = false
	## The caret on an empty line: start the list (or quote) there.
	if not any:
		var p: String = {"bullet": "- ", "number": "1. ", "quote": "> ", "task": "- [ ] "}[kind]
		var t1 := text.substr(0, ls) + p + block + text.substr(le)
		return {"text": t1, "from": ls + p.length() + block.length(), "to": ls + p.length() + block.length()}
	var out := PackedStringArray()
	var n := 0
	for l in lines:
		var s := String(l)
		if s.strip_edges() == "" or not any:
			out.append(s)
			continue
		if all_have:
			var m := mine.search(s)
			var indent := "" if kind == "quote" else m.get_string(1)
			out.append(indent + s.substr(m.get_end()))
			continue
		n += 1
		if kind == "quote":
			out.append("> " + s)
			continue
		## Swap list styles rather than stacking them.
		var indent := ""
		var rest := s
		for rx in [task, bullet, number]:
			var m2: RegExMatch = (rx as RegEx).search(s)
			if m2 != null:
				indent = m2.get_string(1)
				rest = s.substr(m2.get_end())
				break
		if indent == "" and rest == s:
			var k := 0
			while k < s.length() and (s[k] == " " or s[k] == "\t"):
				k += 1
			indent = s.substr(0, k)
			rest = s.substr(k)
		var marker: String = {"bullet": "- ", "task": "- [ ] "}.get(kind, "%d. " % n)
		out.append(indent + marker + rest)
	var new_block := "\n".join(out)
	var t := text.substr(0, ls) + new_block + text.substr(le)
	return {"text": t, "from": ls, "to": ls + new_block.length()}


## Fences the lines `[from, to]` touches in ```` ``` ````, or removes the fence
## when those lines already sit between two fence lines. With no selection on
## an empty line it inserts an empty fenced block with the caret inside.
static func code_block(text: String, from: int, to: int) -> Dictionary:
	if from > to:
		var t0 := from
		from = to
		to = t0
	var ls := _line_start(text, from)
	var end_pos := to
	if to > from and text[to - 1] == "\n":
		end_pos = to - 1
	var le := _line_end(text, end_pos)
	# already fenced? the line above and the line below are both ```
	if ls >= 1 and le < text.length():
		var above_s := _line_start(text, ls - 1)
		var above := text.substr(above_s, ls - 1 - above_s)
		var below_e := _line_end(text, le + 1)
		var below := text.substr(le + 1, below_e - le - 1)
		if above.strip_edges().begins_with("```") and below.strip_edges() == "```":
			var inner := text.substr(ls, le - ls)
			var t := text.substr(0, above_s) + inner + text.substr(below_e)
			return {"text": t, "from": above_s, "to": above_s + inner.length()}
	var block := text.substr(ls, le - ls)
	var fenced := "```\n" + block + "\n```"
	var t2 := text.substr(0, ls) + fenced + text.substr(le)
	if block == "":
		return {"text": t2, "from": ls + 4, "to": ls + 4}
	return {"text": t2, "from": ls + 4, "to": ls + 4 + block.length()}


## `[[Note]]` for a vault path (`Settlements/Aldenmoor.md` -> `Aldenmoor`, the
## form Obsidian resolves by name). A selection becomes the link's shown text,
## `[[Note|selection]]`. The caret lands after the link.
static func note_link(text: String, from: int, to: int, rel: String) -> Dictionary:
	if from > to:
		var t0 := from
		from = to
		to = t0
	var target := rel.get_file()
	if target.to_lower().ends_with(".md"):
		target = target.substr(0, target.length() - 3)
	var sel := text.substr(from, to - from)
	var link := "[[%s]]" % target if sel == "" or sel == target else "[[%s|%s]]" % [target, sel]
	var t := text.substr(0, from) + link + text.substr(to)
	return {"text": t, "from": from + link.length(), "to": from + link.length()}


## A callout's header line: `> [!type]`, the space after `>` optional.
const _CALLOUT_RE := "^> ?\\[!([^\\]]+)\\]([+-]?)\\s*(.*)$"


## Obsidian's callout. On an empty line it writes `> [!note] ` with the caret
## after it, for a title. Over lines it writes a `> [!note]` header line and
## quotes every line below it. With the caret anywhere in an existing callout
## (a `> [!…]` header joined to the caret line by `>` lines) it removes the
## header and the quoting -- the same toggle every other button makes.
static func callout(text: String, from: int, to: int) -> Dictionary:
	if from > to:
		var t0 := from
		from = to
		to = t0
	var head_re := RegEx.create_from_string(_CALLOUT_RE)
	var ls := _line_start(text, from)
	var top := ls
	var head := -1
	while true:
		var line := text.substr(top, _line_end(text, top) - top)
		if not line.begins_with(">"):
			break
		if head_re.search(line) != null:
			head = top
			break
		if top == 0:
			break
		top = _line_start(text, top - 1)
	if head >= 0:
		var end := _line_end(text, head)
		while end < text.length():
			var nxt := end + 1
			var ne := _line_end(text, nxt)
			if not text.substr(nxt, ne - nxt).begins_with(">"):
				break
			end = ne
		var rows := text.substr(head, end - head).split("\n")
		var body := PackedStringArray()
		for i in range(1, rows.size()):
			var r := String(rows[i]).substr(1)
			if r.begins_with(" "):
				r = r.substr(1)
			body.append(r)
		var inner := "\n".join(body)
		return {"text": text.substr(0, head) + inner + text.substr(end), "from": head, "to": head + inner.length()}
	var end_pos := to
	if to > from and text[to - 1] == "\n":
		end_pos = to - 1
	var le := _line_end(text, end_pos)
	var block := text.substr(ls, le - ls)
	if block.strip_edges() == "":
		var h := "> [!note] "
		return {"text": text.substr(0, ls) + h + text.substr(le), "from": ls + h.length(), "to": ls + h.length()}
	var out := PackedStringArray(["> [!note]"])
	for l in block.split("\n"):
		out.append("> " + String(l))
	var nb := "\n".join(out)
	return {"text": text.substr(0, ls) + nb + text.substr(le), "from": ls, "to": ls + nb.length()}


## `#tag` at the caret, or over a selection. Obsidian's tags hold no spaces
## (its help suggests camelCase, snake_case or kebab-case), so whitespace in a
## selected phrase becomes `-`. The selection stays on the tag's text.
static func tag(text: String, from: int, to: int) -> Dictionary:
	if from > to:
		var t0 := from
		from = to
		to = t0
	var sel := text.substr(from, to - from).strip_edges()
	var tag_name := RegEx.create_from_string("\\s+").sub(sel, "-", true)
	var t := text.substr(0, from) + "#" + tag_name + text.substr(to)
	return {"text": t, "from": from + 1, "to": from + 1 + tag_name.length()}


# -- Rendering (Markdown -> BBCode) -------------------------------------------

## Drops a leading YAML frontmatter block -- the vault window shows it as
## chips, so rendering it again as prose would say it twice.
static func strip_frontmatter(text: String) -> String:
	var lines := text.split("\n")
	if lines.is_empty() or String(lines[0]).strip_edges() != "---":
		return text
	for i in range(1, lines.size()):
		if String(lines[i]).strip_edges() == "---":
			return "\n".join(PackedStringArray(lines.slice(i + 1)))
	return text


## Escape the one BBCode-significant character, `[`, as `[lb]` so note text can never open a tag in
## the preview.
static func _esc(s: String) -> String:
	return s.replace("[", "[lb]")


## A small Markdown subset rendered as `RichTextLabel` BBCode: ATX headings,
## bold, italic, bold-italic, `<u>` underline, `~~` strikethrough,
## `==highlight==`, inline code, fenced code, bulleted and numbered lists,
## `- [ ]`/`- [x]` tasks, quotes, `> [!type] title` callouts, rules,
## `[[wikilinks]]` (with `|alias` and `#heading`), `![[embeds]]` as a labelled
## placeholder, `#tags` and `[text](url)` links. It is a reading view, not a
## CommonMark renderer; anything it does not recognise is shown as the text it
## is. `accent_hex`/`dim_hex` default to the theme's; a probe passes literals.
##
## `link_info` (VE-1) is the host's classifier, `(target: String) -> {kind}` with `note`, `entity` or
## `unresolved`. With it, every wikilink is drawn by kind and carries a `wiki:<target>` meta that
## `_on_meta_clicked` follows: a note link accent and underlined; a link to a world entity with no
## note accent, not underlined, with a leading `\u2316`; an unresolved link dim and italic. Without it
## the output is exactly what it was before VE-1 (`[url=<target>]`, one colour), which
## `_mdedit_probe.gd` pins. BBCode has no dotted or dashed underline, so the three kinds differ by
## colour, underline, italic and glyph rather than by line style (disclosed against
## `VAULT_EDITOR_PROPOSAL.md` s.3.2's table).
static func to_bbcode(md: String, accent_hex: String = "", dim_hex: String = "", link_info: Callable = Callable(), hooks: Dictionary = {}) -> String:
	_link_ctx = link_info
	_hooks = hooks
	_embed_stack = (hooks.get("stack", []) as Array).duplicate()
	_readonly = 0
	embed_keys = []
	var out := _to_bbcode(md, accent_hex, dim_hex)
	_link_ctx = Callable()
	_hooks = {}
	_embed_stack = []
	return out


## The classifier `to_bbcode` is rendering under, set for the duration of one call so the inline
## renderer needs no extra parameter threaded through its many call sites. Single-threaded: the
## shell renders on the main thread only.
static var _link_ctx: Callable = Callable()

## The host hooks `to_bbcode` is rendering under (VE-7), set and cleared like `_link_ctx`. Keys, all
## optional: `embed` (`(inner, stack) -> Dictionary`, the crate's `vault_embed_open`), `image`
## (`(embed_result) -> {src, w, h}`, or `{}` when the picture cannot be drawn), `chronos`
## (`(block_text) -> Dictionary`, `vault_note_chronos`), `ticks` (true when a task's box may be
## clicked), `stack` (the notes being drawn, the open note first) and `open` (embed key -> expanded).
## With none of them the output is what it was before VE-7 for everything but the new block kinds
## (tables, callout colours), which need no host.
static var _hooks: Dictionary = {}
## The notes being drawn, outermost first. An embed's excerpt is rendered with its note pushed, so a
## note embedded inside itself is the crate's `cycle` answer and not an endless render.
static var _embed_stack: Array = []
## Above 0 while an embedded excerpt is rendered: an embed is a READ-ONLY view, so a task's box in it
## is not a link (ticking it would edit a note the user did not open).
static var _readonly := 0
## The keys of the note embeds the last `to_bbcode` drew, by the number carried in their `embed:<n>`
## meta; the editor reads it right after rendering to know which embed a click names.
static var embed_keys: Array = []

## Longest run of an embedded note's lines shown before it is collapsed behind a toggle (a labelled
## judgement: enough to recognise the note, short enough that three embeds still fit a screen).
const EMBED_COLLAPSED_LINES := 6
## Widest a picture is drawn, in pixels (a labelled judgement: the Preview column is about this wide
## on the narrowest Split; a wider picture would scroll the label sideways). A `|N` size is honoured up
## to this and no further.
const IMG_MAX_PX := 560
## A backstop under the crate's own cap (`cartalith_vault::embed::MAX_EMBED_DEPTH`, 3): if a host hook ever answers `note`
## for a chain this long, the renderer stops anyway. It is not the rule, only the guarantee that a wrong hook cannot
## make Preview recurse without end.
const EMBED_HARD_LIMIT := 8
## Width of the timeline's text axis, in characters (labelled judgement: fits a phone's Preview).
const CHRONOS_AXIS_WIDTH := 36


## `rrggbb` (no `#`) of a theme token held to 4.5:1 against the field ground, as the Write colouring
## does, so a callout or a notice reads on either palette.
static func _tok_hex(token: String) -> String:
	return Highlighter.legible(DccTheme.c(token), DccTheme.c("sunken"), 4.5).to_html(false)


## Obsidian's callout types, grouped (https://help.obsidian.md/callouts, the built-in list and its
## aliases), each group given one theme token and one glyph. Obsidian colours by type; the tokens
## here are the palette's nearest semantic ones (water = information, good = success and advice,
## warn = a question or a caution, block = failure or danger, accent = an example, text_secondary =
## a quote). An unknown type reads as a note, as Obsidian draws it. Pure: `_ve7_probe.gd` pins
## every row.
static func callout_style(type_name: String) -> Dictionary:
	match type_name.strip_edges().to_lower():
		"abstract", "summary", "tldr":
			return {"token": "water", "glyph": "≡"}
		"info", "todo":
			return {"token": "water", "glyph": "ⓘ"}
		"tip", "hint", "important":
			return {"token": "good", "glyph": "✦"}
		"success", "check", "done":
			return {"token": "good", "glyph": "✔"}
		"question", "help", "faq":
			return {"token": "warn", "glyph": "?"}
		"warning", "caution", "attention":
			return {"token": "warn", "glyph": "⚠"}
		"failure", "fail", "missing", "danger", "error", "bug":
			return {"token": "block", "glyph": "✖"}
		"example":
			return {"token": "accent", "glyph": "▤"}
		"quote", "cite":
			return {"token": "text_secondary", "glyph": "❝"}
	return {"token": "water", "glyph": "✎"}


## The cells of one table row: split at `|` that is not escaped as `\|`, the row's own outer pipes
## dropped, each cell trimmed and its `\|` made a `|`. `a | b` and `| a | b |` both give two cells.
static func split_table_row(line: String) -> PackedStringArray:
	var s := line.strip_edges()
	var cells := PackedStringArray()
	var cur := ""
	var i := 0
	while i < s.length():
		var c := s[i]
		if c == "\\" and i + 1 < s.length() and s[i + 1] == "|":
			cur += "|"
			i += 2
			continue
		if c == "|":
			cells.append(cur)
			cur = ""
		else:
			cur += c
		i += 1
	cells.append(cur)
	if s.begins_with("|"):
		cells.remove_at(0)
	if s.ends_with("|") and not s.ends_with("\\|") and cells.size() > 0:
		cells.remove_at(cells.size() - 1)
	for k in cells.size():
		cells[k] = cells[k].strip_edges()
	return cells


## The alignments of a table's delimiter row (`|---|:-:|--:|`): one of `l`, `c`, `r` or `""` per
## column, or `[]` when the line is not a delimiter row. Every cell must be dashes with an optional
## colon at either end; a line with no pipe needs two columns (a lone `---` is a rule, not a table).
static func table_aligns(line: String) -> Array:
	var cells := split_table_row(line)
	if cells.is_empty() or (cells.size() < 2 and not line.contains("|")):
		return []
	var rx := RegEx.create_from_string("^(:?)-+(:?)$")
	var out: Array = []
	for c in cells:
		var m := rx.search(c)
		if m == null:
			return []
		var l := m.get_string(1) == ":"
		var r := m.get_string(2) == ":"
		out.append("c" if l and r else ("l" if l else ("r" if r else "")))
	return out


## One table as BBCode: a `[table]` with a bold, washed header row, a thin border, cells padded to
## the header's column count (a short row is padded with empty cells, a long one cut), and each
## column aligned as the delimiter row said.
static func _table_bbcode(header: PackedStringArray, aligns: Array, rows: Array, accent: String, dim: String) -> String:
	var n := aligns.size()
	var out := "[table=%d]" % n
	for r in range(-1, rows.size()):
		var cells: PackedStringArray = header if r < 0 else rows[r]
		for c in n:
			var txt := inline_bbcode(String(cells[c]) if c < cells.size() else "", accent)
			if r < 0:
				txt = "[b]%s[/b]" % txt
			match String(aligns[c]):
				"c":
					txt = "[center]%s[/center]" % txt
				"r":
					txt = "[right]%s[/right]" % txt
			var bg := " bg=#%s26" % accent if r < 0 else ""
			out += "[cell border=#%s%s padding=6,3,6,3]%s[/cell]" % [dim, bg, txt]
	return out + "[/table]"


## The timeline's text axis: `lo ├──●━━●─┤ hi` over `width` cells, one `●` per event's start (and
## end), `━` along a period between them. `""` when fewer than two distinct years are in play
## (there is no axis to draw on one point). `events` are the crate's rows (`start`, optional `end`).
static func chronos_axis(events: Array, width: int = CHRONOS_AXIS_WIDTH) -> String:
	var lo := 0
	var hi := 0
	var seen := false
	for e in events:
		for k in ["start", "end"]:
			if (e as Dictionary).has(k):
				var y := int((e as Dictionary)[k])
				lo = y if not seen else mini(lo, y)
				hi = y if not seen else maxi(hi, y)
				seen = true
	if not seen or hi == lo:
		return ""
	var cells: Array = []
	for _i in width:
		cells.append("─")
	var at := func(y: int) -> int:
		return clampi(roundi(float(y - lo) * float(width - 1) / float(hi - lo)), 0, width - 1)
	for e in events:
		var d: Dictionary = e
		if d.has("end"):
			for i in range(int(at.call(int(d["start"]))), int(at.call(int(d["end"]))) + 1):
				cells[i] = "━"
	for e in events:
		var d: Dictionary = e
		cells[int(at.call(int(d["start"])))] = "●"
		if d.has("end"):
			cells[int(at.call(int(d["end"])))] = "●"
	return "%d ├%s┤ %d" % [lo, "".join(PackedStringArray(cells)), hi]


## `1200` or `1200–1245`: the years an entry covers.
static func chronos_years(e: Dictionary) -> String:
	if e.has("end") and int(e["end"]) != int(e["start"]):
		return "%d–%d" % [int(e["start"]), int(e["end"])]
	return str(int(e["start"]))


## One ` ```chronos ` block as a timeline (`VAULT_EDITOR_PROPOSAL.md` s.3.4): a heading with the entry
## count, the axis, then one row per entry in year order. A row (years, name, group, description) is a link `year:<start>`: clicking it
## emits `chronos_year_requested`, and the host moves Cartalith's year there (the note's history and the
## map's timeline stay one thing, Ruling AM). A line the block could not read is listed, never dropped. With no
## `chronos` hook, or when it cannot answer, the block is drawn as the code it is.
static func _chronos_bbcode(block_lines: PackedStringArray, accent: String, dim: String) -> String:
	var hook: Callable = _hooks.get("chronos", Callable())
	var r: Dictionary = {}
	if hook.is_valid():
		r = hook.call("```chronos\n" + "\n".join(block_lines) + "\n```")
	if not bool(r.get("ok", false)):
		return "[code]" + _esc("\n".join(block_lines)) + "[/code]"
	var events: Array = (r.get("events", []) as Array).duplicate()
	var order: Array = []
	for i in events.size():
		order.append(i)
	## Year order, ties by the order written: the sort key carries the index, so it is stable by construction.
	order.sort_custom(func(a, b) -> bool:
		var ya := int((events[a] as Dictionary)["start"])
		var yb := int((events[b] as Dictionary)["start"])
		return a < b if ya == yb else ya < yb)
	var skipped: Array = r.get("skipped", [])
	var out := PackedStringArray()
	out.append("[indent][b]Timeline[/b] [color=#%s]· %s[/color]" % [dim, "empty" if events.is_empty() else "%d %s" % [events.size(), "entry" if events.size() == 1 else "entries"]])
	var ax := chronos_axis(events)
	if ax != "":
		out.append("[color=#%s][code]%s[/code][/color]" % [dim, ax])
	var glyph := {"event": "●", "period": "▬", "point": "◆", "marker": "┃"}
	var hex_rx := RegEx.create_from_string("^[0-9a-fA-F]{6}$")
	for i in order:
		var e: Dictionary = events[i]
		var gcol := accent
		var c := String(e.get("color", ""))
		if hex_rx.search(c) != null:
			gcol = c
		var row := "[color=#%s]%s[/color] [url=year:%d][color=#%s]%s[/color]  [b]%s[/b]" % [
			gcol, glyph.get(String(e.get("kind", "event")), "●"), int(e["start"]), accent, chronos_years(e), _esc(String(e["name"]))]
		if e.has("group"):
			row += " [color=#%s]{%s}[/color]" % [dim, _esc(String(e["group"]))]
		if e.has("description"):
			row += " [color=#%s]— %s[/color]" % [dim, _esc(String(e["description"]))]
		## The whole row after its glyph is the link, so a finger has a line's height and the row's width to land on.
		out.append(row + "[/url]")
	var warn := _tok_hex("warn")
	for l in skipped:
		out.append("[color=#%s]✖ not read: %s[/color]" % [warn, _esc(String(l))])
	return "\n".join(out) + "[/indent]"


## Placeholder text for an embed that cannot be drawn: one warn-coloured line saying why, never blank, never
## an error dialog. `r` is the crate's `vault_embed_open` answer.
static func _embed_notice(r: Dictionary, inner: String) -> String:
	var name := inner.get_slice("|", 0).strip_edges()
	var msg := ""
	if not bool(r.get("ok", false)):
		msg = "Embed could not be read: %s" % String(r.get("error", "no vault"))
	else:
		match String(r.get("kind", "")):
			"missing":
				msg = "Embed not found: %s" % name
			"ambiguous":
				msg = "Embed %s could mean %d notes" % [name, (r.get("candidates", []) as Array).size()]
			"cycle":
				var names := PackedStringArray()
				for p in (r.get("chain", []) as Array):
					names.append(String(p).get_file().get_basename())
				msg = "Embed skipped, it would repeat itself: %s" % " → ".join(names)
			"toodeep":
				msg = "Embed skipped, more than %d levels deep: %s" % [int(r.get("limit", 0)), name]
			"fragment":
				msg = "%s has nothing at %s" % [String(r.get("path", name)).get_file().get_basename(), String(r.get("fragment", ""))]
			"escape":
				msg = "Embed refused, outside this vault: %s" % name
			"unsupported":
				msg = "Embed not drawn: %s" % String(r.get("why", name))
			"toolarge":
				msg = "Image too large to show (%d MB, limit %d MB): %s" % [int(r.get("bytes", 0)) / 1048576, int(r.get("limit", 0)) / 1048576, name]
			_:
				msg = "Embed not drawn: %s" % name
	return "[color=#%s][i]⚠ %s[/i][/color]" % [_tok_hex("warn"), _esc(msg)]


## An image embed as `[img]`, or a notice when the host cannot decode it. The width is the picture's own
## unless the note asked for one (`|300`, which Obsidian also takes as the width, larger or smaller), and
## never more than `IMG_MAX_PX`.
static func _embed_image(r: Dictionary, inner: String) -> String:
	var img_hook: Callable = _hooks.get("image", Callable())
	var got: Dictionary = img_hook.call(r) if img_hook.is_valid() else {}
	if String(got.get("src", "")) == "":
		return "[color=#%s][i]⚠ Image could not be read: %s[/i][/color]" % [_tok_hex("warn"), _esc(inner.get_slice("|", 0).strip_edges())]
	var w := int(got.get("w", IMG_MAX_PX))
	if r.has("width"):
		w = int(r["width"])
	return "[img width=%d]%s[/img]" % [clampi(w, 1, IMG_MAX_PX), String(got["src"])]


## A note embed as a block: a header (`▸` name, a toggle when there is more than the collapsed
## lines, and `open ›` to follow it), then the excerpt rendered read-only inside the chain with this
## note pushed. The crate already refused a cycle and a runaway chain before this is reached.
static func _embed_note_block(r: Dictionary, inner: String, accent: String, dim: String) -> String:
	var path := String(r["path"])
	var lines := String(r.get("text", "")).split("\n")
	var key := "%s>%s" % ["|".join(PackedStringArray(_embed_stack)), inner]
	embed_keys.append(key)
	var n := embed_keys.size() - 1
	var open := bool((_hooks.get("open", {}) as Dictionary).get(key, false))
	var long := lines.size() > EMBED_COLLAPSED_LINES or bool(r.get("truncated", false))
	var shown: Array = Array(lines)
	if long and not open:
		shown = shown.slice(0, EMBED_COLLAPSED_LINES)
	var title := path.get_file().get_basename()
	if r.has("heading"):
		title += " › " + String(r["heading"])
	var mark := ("▾" if open else "▸") if long else "▪"
	var head := "[color=#%s]%s [url=embed:%d]%s[/url]   [url=wiki:%s]open ›[/url][/color]" % [
		dim, mark, n, _esc(title), _esc(path.trim_suffix(".md"))]
	_embed_stack.append(path)
	_readonly += 1
	var body := _to_bbcode("\n".join(PackedStringArray(shown)), accent, dim)
	_readonly -= 1
	_embed_stack.pop_back()
	var tail := ""
	if long and not open:
		tail = "\n[color=#%s]… more in this note[/color]" % dim
	elif bool(r.get("truncated", false)):
		tail = "\n[color=#%s]… the rest is in the note[/color]" % dim
	return "%s\n[indent]%s%s[/indent]" % [head, body, tail]


## An embed standing alone on its line, with every kind drawn in full. Only called with an `embed` hook.
static func _embed_block(inner: String, accent: String, dim: String) -> String:
	var hook: Callable = _hooks["embed"]
	if _embed_stack.size() > EMBED_HARD_LIMIT:
		return _embed_notice({"ok": true, "kind": "toodeep", "limit": EMBED_HARD_LIMIT}, inner)
	var r: Dictionary = hook.call(inner, _embed_stack)
	if bool(r.get("ok", false)):
		match String(r.get("kind", "")):
			"note":
				return _embed_note_block(r, inner, accent, dim)
			"image":
				return _embed_image(r, inner)
	return _embed_notice(r, inner)


## An embed in the middle of a line: a picture stays where it is; a note shrinks to a link to it (a whole
## note cannot sit inside a sentence); anything else is its notice.
static func _embed_inline(inner: String, accent: String) -> String:
	var hook: Callable = _hooks["embed"]
	var r: Dictionary = hook.call(inner, _embed_stack)
	if bool(r.get("ok", false)):
		match String(r.get("kind", "")):
			"note":
				return "[color=#%s][u][url=wiki:%s]▪ %s[/url][/u][/color]" % [
					accent, _esc(String(r["path"]).trim_suffix(".md")), _esc(String(r["path"]).get_file().get_basename())]
			"image":
				return _embed_image(r, inner)
	return _embed_notice(r, inner)


static func _to_bbcode(md: String, accent_hex: String, dim_hex: String) -> String:
	var accent := accent_hex if accent_hex != "" else DccTheme.c("accent").to_html(false)
	var dim := dim_hex if dim_hex != "" else DccTheme.c("text_dim").to_html(false)
	var out := PackedStringArray()
	var in_code := false
	var code_lang := ""
	var in_callout := false
	var callout_col := accent
	var code_buf := PackedStringArray()
	var bullet := RegEx.create_from_string("^(\\s*)[-*+] (.*)$")
	var number := RegEx.create_from_string("^(\\s*)(\\d+)[.)] (.*)$")
	var task := RegEx.create_from_string("^(\\s*)[-*+] \\[(.)\\] (.*)$")
	var callout_head := RegEx.create_from_string(_CALLOUT_RE)
	var embed_line := RegEx.create_from_string("^\\s*!\\[\\[([^\\]]+)\\]\\]\\s*$")
	md = md.replace("\r\n", "\n")
	## A leading YAML block is data, not prose: shown dim and monospaced, as
	## written, rather than as two rules around a paragraph.
	var base := 0
	var body := strip_frontmatter(md)
	if body != md:
		var removed := md.substr(0, md.length() - body.length())
		base = removed.count("\n")
		var fm := removed.strip_edges()
		out.append("[color=#%s][code]%s[/code][/color]" % [dim, _esc(fm)])
		md = body
	var lines := md.split("\n")
	var skip_to := 0
	for i in lines.size():
		if i < skip_to:
			continue
		var line := String(lines[i])
		if line.strip_edges().begins_with("```"):
			if in_code:
				if code_lang == "chronos":
					out.append(_chronos_bbcode(code_buf, accent, dim))
				else:
					out.append("[code]" + _esc("\n".join(code_buf)) + "[/code]")
				code_buf = PackedStringArray()
			else:
				code_lang = line.strip_edges().trim_prefix("```").strip_edges().to_lower()
			in_code = not in_code
			continue
		if in_code:
			code_buf.append(line)
			continue
		## A callout: the header line names its type and optional title (the
		## `+`/`-` fold marker is dropped -- there is nothing to fold here), and
		## the `>` lines under it are its body, drawn as prose, not as a quote. The
		## type picks the colour and glyph (`callout_style`, VE-7); the bar `▎` repeats on
		## every body line so the block reads as one box.
		var ch := callout_head.search(line)
		if ch != null:
			var title := ch.get_string(3).strip_edges()
			if title == "":
				title = ch.get_string(1).strip_edges().capitalize()
			var cs := callout_style(ch.get_string(1))
			callout_col = _tok_hex(String(cs["token"]))
			out.append("[indent][color=#%s]▎ %s [b]%s[/b][/color][/indent]" % [callout_col, cs["glyph"], inline_bbcode(title, accent)])
			in_callout = true
			continue
		if not line.begins_with(">"):
			in_callout = false
		## A table: a row, then a delimiter row of the same width, then rows until a line without a pipe.
		if line.contains("|") and i + 1 < lines.size():
			var al := table_aligns(String(lines[i + 1]))
			var head_cells := split_table_row(line)
			if not al.is_empty() and head_cells.size() == al.size():
				var rows: Array = []
				var j := i + 2
				while j < lines.size() and String(lines[j]).contains("|") and String(lines[j]).strip_edges() != "":
					rows.append(split_table_row(String(lines[j])))
					j += 1
				skip_to = j
				out.append(_table_bbcode(head_cells, al, rows, accent, dim))
				continue
		## An embed alone on its line is drawn as a block; anywhere else it is handled inline.
		if _hooks.has("embed"):
			var em := embed_line.search(line)
			if em != null:
				out.append(_embed_block(em.get_string(1), accent, dim))
				continue
		var hp := _heading_prefix(line)
		if hp.x > 0:
			out.append("[font_size=%d][b]%s[/b][/font_size]" % [HEADING_PX[hp.x], inline_bbcode(line.substr(hp.y), accent)])
			continue
		if line.begins_with(">"):
			var q := line.substr(1)
			if q.begins_with(" "):
				q = q.substr(1)
			if in_callout:
				out.append("[indent][color=#%s]▎[/color] %s[/indent]" % [callout_col, inline_bbcode(q, accent)])
			else:
				out.append("[indent][color=#%s][i]%s[/i][/color][/indent]" % [dim, inline_bbcode(q, accent)])
			continue
		## Before the bullet test, which a task line also matches. A box is a link (`task:<line>`)
		## only when the host can tick it and this is the open note itself, not an embedded view of
		## another; the line number is in the WHOLE text, front matter counted (`base`).
		var mt := task.search(line)
		if mt != null:
			var tdepth := mt.get_string(1).length() / 2
			var ticking: bool = bool(_hooks.get("ticks", false)) and _readonly == 0
			var box_open := "[url=task:%d]" % (base + i) if ticking else ""
			## The link starts at the item's indent and runs through the box and the two spaces after it, so a
			## finger has a target of about 40 px, not the glyph's 14.
			var box_close := "[/url]" if ticking else ""
			if mt.get_string(2) == " ":
				out.append("%s%s  ☐  %s%s" % ["    ".repeat(tdepth), box_open, box_close, inline_bbcode(mt.get_string(3), accent)])
			else:
				out.append("%s%s  ☑  %s[color=#%s][s]%s[/s][/color]" % ["    ".repeat(tdepth), box_open, box_close, dim, inline_bbcode(mt.get_string(3), accent)])
			continue
		var mb := bullet.search(line)
		if mb != null:
			var depth := mb.get_string(1).length() / 2
			out.append("%s  •  %s" % ["    ".repeat(depth), inline_bbcode(mb.get_string(2), accent)])
			continue
		var mn := number.search(line)
		if mn != null:
			var depth2 := mn.get_string(1).length() / 2
			out.append("%s  %s.  %s" % ["    ".repeat(depth2), mn.get_string(2), inline_bbcode(mn.get_string(3), accent)])
			continue
		var st := line.strip_edges()
		if st.length() >= 3 and (st == "-".repeat(st.length()) or st == "*".repeat(st.length())):
			out.append("[color=#%s]%s[/color]" % [dim, "─".repeat(24)])
			continue
		out.append(inline_bbcode(line, accent))
	if in_code:
		if code_lang == "chronos":
			out.append(_chronos_bbcode(code_buf, accent, dim))
		else:
			out.append("[code]" + _esc("\n".join(code_buf)) + "[/code]")
	return "\n".join(out)


## One line's inline markup. Code spans are cut out first (nothing inside
## them is markup), links become placeholders before `[` is escaped, then the
## emphasis patterns run longest-marker first so `***x***` is bold-italic.
static func inline_bbcode(s: String, accent_hex: String = "") -> String:
	if accent_hex == "":
		accent_hex = DccTheme.c("accent").to_html(false)
	var parts := s.split("`")
	var out := ""
	for i in parts.size():
		var seg := String(parts[i])
		if i % 2 == 1 and i < parts.size() - 1:
			out += "[code]" + _esc(seg) + "[/code]"
			continue
		if i % 2 == 1:
			seg = "`" + seg
		out += _inline_text(seg, accent_hex)
	return out


## What a wikilink shows, as Obsidian's reading view shows it: the alias when
## there is one; else `note > heading` for `[[note#heading]]` (the heading
## alone for a same-note `[[#heading]]`); else the note's name.
static func _wiki_shown(target: String, alias: String) -> String:
	if alias != "":
		return alias
	var hash := target.find("#")
	if hash < 0:
		return target
	var note := target.substr(0, hash)
	var sub := target.substr(hash + 1).replace("#", " > ")
	return sub if note == "" else "%s > %s" % [note, sub]


## One link as BBCode. Without a classifier this is the pre-VE-1 form. With one: a Markdown link
## whose address has a URL scheme is external (`ext:` meta, accent, underlined); any other address
## is a note path (percent-decoded, `.md` dropped) and, like a wikilink, is classed and given a
## `wiki:` meta. `target` is the text between the brackets, `#heading` included; classing looks at
## the note part only.
static func _link_bbcode(target: String, shown: String, accent_hex: String, is_markdown_link: bool) -> String:
	if not _link_ctx.is_valid():
		return "[color=#%s][url=%s]%s[/url][/color]" % [accent_hex, _esc(target), _esc(shown)]
	if is_markdown_link and RegEx.create_from_string("^[A-Za-z][A-Za-z0-9+.-]*:").search(target) != null:
		return "[color=#%s][u][url=ext:%s]%s[/url][/u][/color]" % [accent_hex, _esc(target), _esc(shown)]
	var t := target.uri_decode() if is_markdown_link else target
	if is_markdown_link and t.to_lower().ends_with(".md"):
		t = t.substr(0, t.length() - 3)
	var note_part := t.get_slice("#", 0).get_slice("^", 0)
	## A same-note link (`[[#Heading]]`) has no note part: it is always this note.
	var info: Dictionary = {"kind": "note"} if note_part == "" else _link_ctx.call(note_part)
	var dim := DccTheme.c("text_dim").to_html(false)
	match String(info.get("kind", "unresolved")):
		"note":
			return "[color=#%s][u][url=wiki:%s]%s[/url][/u][/color]" % [accent_hex, _esc(t), _esc(shown)]
		"entity":
			return "[color=#%s][url=wiki:%s]\u2316 %s[/url][/color]" % [accent_hex, _esc(t), _esc(shown)]
		_:
			return "[color=#%s][i][url=wiki:%s]%s[/url][/i][/color]" % [dim, _esc(t), _esc(shown)]


## Render one line's inline marks as BBCode: embeds, wikilinks and Markdown links are lifted out
## into numbered placeholders first (so their inner brackets or marks are not re-read), the rest is
## escaped, then tags, highlight, bold-italic, bold, italic, strike and underline are substituted,
## and the links are put back. `accent_hex` colours links, tags and the highlight wash. Order is
## load-bearing: embeds before wikilinks, tags before emphasis.
static func _inline_text(s: String, accent_hex: String) -> String:
	var links: Array = []
	var embed := RegEx.create_from_string("!\\[\\[([^\\]|]+)(?:\\|([^\\]]+))?\\]\\]")
	var wiki := RegEx.create_from_string("\\[\\[([^\\]|]+)(?:\\|([^\\]]+))?\\]\\]")
	var md_link := RegEx.create_from_string("\\[([^\\]]+)\\]\\(([^)\\s]+)\\)")
	var md_image := RegEx.create_from_string("!\\[([^\\]]*)\\]\\(([^)\\s]+)(?:\\s+\"[^\"]*\")?\\)")
	var k := 0
	## `![alt](path)` (VE-7): with the host's `embed` hook it is a picture (or a note, or a notice) drawn
	## through the same crate decision as `![[...]]`, its address percent-decoded. Without the hook it
	## stays what it was, a link with a stray `!`.
	if _hooks.has("embed"):
		for m in md_image.search_all(s):
			links.append(_embed_inline(m.get_string(2).uri_decode(), accent_hex))
			s = s.replace(m.get_string(0), "\u0001%d\u0001" % k)
			k += 1
	## `![[embed]]` first -- `[[…]]` inside it would otherwise read as a link.
	## Drawn as a labelled placeholder, not the embedded note or image.
	for m in embed.search_all(s):
		if _hooks.has("embed"):
			links.append(_embed_inline(m.get_string(1) + ("|" + m.get_string(2) if m.get_string(2) != "" else ""), accent_hex))
		else:
			links.append("[color=#%s][i]embedded: %s[/i][/color]" % [accent_hex, _esc(m.get_string(1))])
		s = s.replace(m.get_string(0), "\u0001%d\u0001" % k)
		k += 1
	for m in wiki.search_all(s):
		var shown := _wiki_shown(m.get_string(1), m.get_string(2))
		links.append(_link_bbcode(m.get_string(1), shown, accent_hex, false))
		s = s.replace(m.get_string(0), "\u0001%d\u0001" % k)
		k += 1
	for m in md_link.search_all(s):
		links.append(_link_bbcode(m.get_string(2), m.get_string(1), accent_hex, true))
		s = s.replace(m.get_string(0), "\u0001%d\u0001" % k)
		k += 1
	s = _esc(s)
	## `#tag`: Obsidian's rule -- letters, digits, `_`, `-`, `/`, at least one
	## of them not a digit, and not glued to a word before it (`x#y` is text).
	## Before the emphasis pass, whose BBCode would otherwise carry a `#`.
	s = RegEx.create_from_string("(?<![\\w#&/])#([\\p{L}\\p{N}_/-]*[\\p{L}_/-][\\p{L}\\p{N}_/-]*)") \
		.sub(s, "[color=#%s]#$1[/color]" % accent_hex, true)
	for pair in [
			["==(?=\\S)(.+?)(?<=\\S)==", "[bgcolor=#%s59]$1[/bgcolor]" % accent_hex],
			["\\*\\*\\*(.+?)\\*\\*\\*", "[b][i]$1[/i][/b]"],
			["\\*\\*(.+?)\\*\\*", "[b]$1[/b]"],
			["__(.+?)__", "[b]$1[/b]"],
			["(?<![\\*\\w])\\*(?!\\s)(.+?)(?<!\\s)\\*(?![\\*\\w])", "[i]$1[/i]"],
			["(?<![_\\w])_(?![\\s_])(.+?)(?<![\\s_])_(?![_\\w])", "[i]$1[/i]"],
			["~~(.+?)~~", "[s]$1[/s]"],
			["<u>(.+?)</u>", "[u]$1[/u]"]]:
		s = RegEx.create_from_string(String(pair[0])).sub(s, String(pair[1]), true)
	for j in links.size():
		s = s.replace("\u0001%d\u0001" % j, String(links[j]))
	return s


# -- VE-3: smart keys ---------------------------------------------------------

static var _rx_enter: RegEx
static var _rx_number: RegEx


## Compiled once. Group 1 is the line's lead (indent and any block-quote markers), group 2 the list
## item marker with the spaces after it: a task box (`- [ ] `), a bullet (`- `, `* `, `+ `) or a
## number (`1. `, `1) `). Either group can be empty. The task alternative comes first, or the bullet
## alternative would swallow `- ` and leave `[ ]` as content.
static func _enter_re() -> RegEx:
	if _rx_enter == null:
		_rx_enter = RegEx.new()
		_rx_enter.compile("^([ \\t]*(?:>[ \\t]?)*)([-*+][ \\t]+\\[[ xX]\\][ \\t]+|[-*+][ \\t]+|\\d{1,9}[.)][ \\t]+)?")
		_rx_number = RegEx.new()
		_rx_number.compile("^(\\d{1,9})([.)][ \\t]+)$")
	return _rx_enter


## What Enter does at column `col` of `line`, or `{}` for the editor's own newline.
##
## - **Continue**: on a list item, task or quote line, `{"kind": "continue", "insert": "\n" + marker}`
##   -- the text to insert at the caret. A bullet repeats, a number counts up, a task comes back
##   **unticked**, a quote repeats. Text after the caret flows onto the new line, as in any editor.
## - **End**: on an item with nothing in it and the caret at the end, `{"kind": "end", "line": ...}`
##   -- the line to leave there instead, and no new line. A top-level item or quote line becomes empty
##   (the list ends); an indented item steps out one level and keeps its marker; the item inside a
##   quote loses its marker and keeps the quote.
## - **Default** (`{}`): a plain line, a caret inside the marker itself, or no marker at all.
static func smart_enter(line: String, col: int) -> Dictionary:
	var m := _enter_re().search(line)
	if m == null:
		return {}
	var lead := m.get_string(1)
	var item := m.get_string(2)
	var quoted := lead.contains(">")
	if item.is_empty() and not quoted:
		return {}
	var prefix_len := lead.length() + item.length()
	if col < prefix_len:
		return {}
	if line.substr(prefix_len).strip_edges().is_empty() and col >= line.length():
		if item.is_empty():
			return {"kind": "end", "line": ""}
		if quoted:
			return {"kind": "end", "line": lead}
		if not lead.is_empty():
			return {"kind": "end", "line": outdent_line(lead) + item}
		return {"kind": "end", "line": ""}
	var next_item := item
	var nm := _rx_number.search(item)
	if nm != null:
		next_item = str(int(nm.get_string(1)) + 1) + nm.get_string(2)
	elif item.contains("["):
		next_item = item.replace("[x]", "[ ]").replace("[X]", "[ ]")
	return {"kind": "continue", "insert": "\n" + lead + next_item}


## Whether `line` is a list item or task (a number, bullet or task marker after any indent and quote
## markers) -- the lines Tab indents.
static func is_list_line(line: String) -> bool:
	var m := _enter_re().search(line)
	return m != null and not m.get_string(2).is_empty()


## `line` one indent level shallower: a leading tab goes, otherwise up to `OUTDENT_SPACES` leading
## spaces. A line with no indent is returned as it is.
static func outdent_line(line: String) -> String:
	if line.begins_with("\t"):
		return line.substr(1)
	var n := 0
	while n < OUTDENT_SPACES and n < line.length() and line[n] == " ":
		n += 1
	return line.substr(n)


## Whether the `**` just typed (the two characters before `col`) should be completed to `**|**`. Not a
## third `*` (that is a rule or a bold-italic run), not over an existing `**`, and not when an unmatched
## `**` opener is already earlier on the line -- then this one is the closer of text being typed.
static func should_pair_bold(line: String, col: int) -> bool:
	if col < 2 or line.substr(col - 2, 2) != "**":
		return false
	if col >= 3 and line[col - 3] == "*":
		return false
	if line.substr(col, 1) == "*":
		return false
	var openers := 0
	var i := line.find("**")
	while i >= 0 and i < col - 2:
		openers += 1
		i = line.find("**", i + 2)
	return openers % 2 == 0


## Whether the `*` just typed completes `***` right before the `**` closers `should_pair_bold` added
## (`**|**` then `*`): the pair is then withdrawn so three stars stay three.
static func unpairs_third_star(line: String, col: int) -> bool:
	if col < 3 or line.substr(col - 3, 3) != "***" or line.substr(col, 2) != "**":
		return false
	return col < 4 or line[col - 4] != "*"


## Whether the `*` just typed (at `col - 1`) lands in front of an existing `*` while an unmatched `**`
## opener stands earlier on the line -- the closer of a pair `should_pair_bold` made, which is then typed
## over instead of doubled. False in plain text, so a `*` typed before an unrelated `*` stays.
static func overtypes_star(line: String, col: int) -> bool:
	if col < 1 or line.substr(col - 1, 1) != "*" or line.substr(col, 1) != "*":
		return false
	var openers := 0
	var prefix := line.substr(0, col - 1)
	var i := prefix.find("**")
	while i >= 0:
		openers += 1
		i = prefix.find("**", i + 2)
	return openers % 2 == 1


## Enter and Tab (VE-3). True when the key was used (the caller accepts the event). Enter keeps the
## editor's own newline whenever `smart_enter` answers `{}`, with a selection, with Shift (a plain line
## break, as Obsidian's Shift+Enter) and inside a fence or the front matter. Tab and Shift+Tab move a
## list line a level; on any other line, and over a selection, the editor's own Tab stays.
func _smart_key(k: InputEventKey) -> bool:
	if text_edit.has_selection():
		return false
	var cl := text_edit.get_caret_line()
	var col := text_edit.get_caret_column()
	var line := text_edit.get_line(cl)
	if _hl != null and _hl.kind_at(cl) != Highlighter.K_NORMAL:
		return false
	if k.keycode == KEY_ENTER or k.keycode == KEY_KP_ENTER:
		if k.shift_pressed:
			return false
		var r := smart_enter(line, col)
		if r.is_empty():
			return false
		text_edit.begin_complex_operation()
		if String(r["kind"]) == "continue":
			text_edit.insert_text_at_caret(String(r["insert"]))
		else:
			text_edit.remove_text(cl, 0, cl, line.length())
			text_edit.insert_text(String(r["line"]), cl, 0)
			text_edit.set_caret_line(cl)
			text_edit.set_caret_column(String(r["line"]).length())
		text_edit.end_complex_operation()
		return true
	if k.keycode == KEY_TAB and is_list_line(line):
		text_edit.begin_complex_operation()
		if k.shift_pressed:
			var out := outdent_line(line)
			var cut := line.length() - out.length()
			if cut > 0:
				text_edit.remove_text(cl, 0, cl, cut)
				text_edit.set_caret_column(maxi(col - cut, 0))
		else:
			text_edit.insert_text(INDENT, cl, 0)
			text_edit.set_caret_column(col + INDENT.length())
		text_edit.end_complex_operation()
		return true
	return false


# -- VE-3: find and replace ----------------------------------------------------

## Every non-overlapping hit of `query` in `text` as `Vector2i(from, to)` character offsets, in order.
## Empty for an empty query. `match_case` false folds ASCII case (`findn`), which keeps offsets
## identical to the text's own -- a lower-cased copy would shift them for letters whose lower form is
## longer.
static func find_matches(text: String, query: String, match_case: bool) -> Array:
	var out: Array = []
	if query.is_empty():
		return out
	var i := text.find(query) if match_case else text.findn(query)
	while i >= 0:
		out.append(Vector2i(i, i + query.length()))
		i = text.find(query, i + query.length()) if match_case else text.findn(query, i + query.length())
	return out


## The index in `matches` of the hit after `pos` (forward: first one starting at or after it) or
## before it (backward: last one ending at or before it), wrapping round; -1 when there are none.
static func pick_match(matches: Array, pos: int, forward: bool) -> int:
	if matches.is_empty():
		return -1
	if forward:
		for i in matches.size():
			if (matches[i] as Vector2i).x >= pos:
				return i
		return 0
	for i in range(matches.size() - 1, -1, -1):
		if (matches[i] as Vector2i).y <= pos:
			return i
	return matches.size() - 1


## `text` with every hit of `query` replaced by `replacement`, and how many there were:
## `{"text", "count"}`. A single left-to-right pass, so a replacement that contains the query is not
## searched again. Nothing is replaced for an empty query.
static func replace_all(text: String, query: String, replacement: String, match_case: bool) -> Dictionary:
	var hits := find_matches(text, query, match_case)
	if hits.is_empty():
		return {"text": text, "count": 0}
	var out := ""
	var at := 0
	for h in hits:
		out += text.substr(at, (h as Vector2i).x - at) + replacement
		at = (h as Vector2i).y
	return {"text": out + text.substr(at), "count": hits.size()}


## The find bar: a flow row (so it wraps on a phone) built at setup and hidden, so a rebuild's touch
## pass sees it. Find field, match-case, previous / next, the count, Replace… (reveals the replace
## row) and close; the replace row has its field, Replace and Replace all.
func _build_find() -> void:
	_find_bar = HFlowContainer.new()
	_find_bar.name = "EditorFind"
	_find_bar.add_theme_constant_override("h_separation", 6)
	_find_bar.add_theme_constant_override("v_separation", 4)
	_find_bar.visible = false
	add_child(_find_bar)
	_find_edit = LineEdit.new()
	_find_edit.name = "EditorFindField"
	_find_edit.placeholder_text = "Find"
	_find_edit.custom_minimum_size.x = 150
	DccWidgets.well(_find_edit)
	_find_edit.text_changed.connect(func(_t: String):
		_find_notice = ""
		_find_refresh(true))
	_find_edit.text_submitted.connect(func(_t: String): _find_step(true))
	_find_edit.gui_input.connect(_on_find_input)
	_find_bar.add_child(_find_edit)
	_find_case_button = DccWidgets.segment(_find_bar, "Aa", func():
		_find_case = not _find_case
		DccWidgets.set_segment_on(_find_case_button, _find_case)
		_find_refresh(true))
	_find_case_button.name = "EditorFindCase"
	_find_case_button.tooltip_text = "Match case"
	buttons["find_case"] = _find_case_button
	buttons["find_prev"] = DccWidgets.action(_find_bar, "‹ Prev", func(): _find_step(false))
	buttons["find_prev"].name = "EditorFindPrev"
	buttons["find_next"] = DccWidgets.action(_find_bar, "Next ›", func(): _find_step(true))
	buttons["find_next"].name = "EditorFindNext"
	_find_count = DccTheme.label("", "text_ghost", DccTheme.FS_SMALL)
	_find_count.name = "EditorFindCount"
	_find_bar.add_child(_find_count)
	buttons["find_replace"] = DccWidgets.action(_find_bar, "Replace…", func(): _find_replace_box.visible = not _find_replace_box.visible)
	buttons["find_replace"].name = "EditorFindReplaceToggle"
	buttons["find_close"] = DccWidgets.action(_find_bar, "Close", func(): _find_close(true))
	buttons["find_close"].name = "EditorFindClose"
	_find_replace_box = HBoxContainer.new()
	_find_replace_box.name = "EditorReplaceRow"
	_find_replace_box.add_theme_constant_override("separation", 6)
	_find_replace_box.visible = false
	_find_bar.add_child(_find_replace_box)
	_replace_edit = LineEdit.new()
	_replace_edit.name = "EditorReplaceField"
	_replace_edit.placeholder_text = "Replace with"
	_replace_edit.custom_minimum_size.x = 150
	DccWidgets.well(_replace_edit)
	_replace_edit.text_submitted.connect(func(_t: String): _replace_one())
	_replace_edit.gui_input.connect(_on_find_input)
	_find_replace_box.add_child(_replace_edit)
	buttons["replace_one"] = DccWidgets.action(_find_replace_box, "Replace", func(): _replace_one())
	buttons["replace_one"].name = "EditorReplaceOne"
	buttons["replace_all"] = DccWidgets.action(_find_replace_box, "Replace all", func(): _replace_all())
	buttons["replace_all"].name = "EditorReplaceAll"
	buttons["replace_all"].tooltip_text = "Replaces every match in the note in one step; Ctrl+Z puts them all back."


## Escape closes the bar from either field; Shift+Enter in the find field goes to the previous hit.
func _on_find_input(ev: InputEvent) -> void:
	var k := ev as InputEventKey
	if k == null or not k.pressed:
		return
	if k.keycode == KEY_ESCAPE:
		_find_close(true)
		_find_edit.accept_event()
	elif (k.keycode == KEY_ENTER or k.keycode == KEY_KP_ENTER) and k.shift_pressed and _find_edit.has_focus():
		_find_step(false)
		_find_edit.accept_event()


## Opens the bar (Write and Split only -- it searches the source). A one-line selection becomes the
## query. `with_replace` shows the replace row; opening plain keeps it as it was.
func _find_open_bar(with_replace: bool) -> void:
	if _mode == "preview" or _find_bar == null:
		return
	if text_edit.has_selection():
		var sel := text_edit.get_selected_text()
		if not sel.contains("\n") and not sel.is_empty():
			_find_edit.text = sel
	_find_bar.visible = true
	_find_open = true
	_find_notice = ""
	if with_replace:
		_find_replace_box.visible = true
	_esc_hold("find", true)
	_sg_hide()
	_find_refresh(false)
	_find_edit.grab_focus()
	_find_edit.select_all()


## Hides the bar and clears the match colouring; the text keeps focus unless `focus_text` is false.
func _find_close(focus_text: bool) -> void:
	if _find_bar == null:
		return
	var was_open := _find_open
	_find_bar.visible = false
	_find_open = false
	_esc_hold("find", false)
	if was_open and _hl != null:
		_hl.refresh()
	if focus_text and _mode != "preview":
		text_edit.grab_focus()


## Re-reads the hits for the query and the count label, and re-colours them. With `jump`, the first
## hit at or after the caret is selected -- incremental search: each letter typed keeps the same hit
## when it still matches. The caret and focus stay where they are (the field keeps typing).
func _find_refresh(jump: bool) -> void:
	if not _find_open:
		return
	_find_matches = find_matches(text_edit.text, _find_edit.text, _find_case)
	if _hl != null:
		_hl.refresh()
	if jump and not _find_matches.is_empty():
		var i := pick_match(_find_matches, selection_offsets().x, true)
		_select_match(i)
	_update_find_count()


func _update_find_count() -> void:
	if _find_notice != "":
		_find_count.text = _find_notice
		return
	if _find_edit.text.is_empty():
		_find_count.text = ""
		return
	if _find_matches.is_empty():
		_find_count.text = "No matches"
		return
	var sel := selection_offsets()
	for i in _find_matches.size():
		if _find_matches[i] == sel:
			_find_count.text = "%d of %d" % [i + 1, _find_matches.size()]
			return
	_find_count.text = "%d match%s" % [_find_matches.size(), "" if _find_matches.size() == 1 else "es"]


func _select_match(i: int) -> void:
	if i < 0 or i >= _find_matches.size():
		return
	var m: Vector2i = _find_matches[i]
	select_offsets(m.x, m.y)
	text_edit.center_viewport_to_caret()


## Next or previous hit from the selection (or the caret), wrapping round.
func _find_step(forward: bool) -> void:
	if not _find_open:
		return
	_find_notice = ""
	_find_matches = find_matches(text_edit.text, _find_edit.text, _find_case)
	var sel := selection_offsets()
	_select_match(pick_match(_find_matches, sel.y if forward else sel.x, forward))
	_update_find_count()


## Replaces the selected hit with the replacement (an undo step of its own) and moves to the next;
## when the selection is not a hit, just moves to one.
func _replace_one() -> void:
	if not _find_open:
		return
	_find_notice = ""
	var sel := selection_offsets()
	_find_matches = find_matches(text_edit.text, _find_edit.text, _find_case)
	if not _find_matches.has(sel):
		_find_step(true)
		return
	var repl := _replace_edit.text
	var a := _line_col(text_edit.text, sel.x)
	var b := _line_col(text_edit.text, sel.y)
	text_edit.begin_complex_operation()
	text_edit.remove_text(a.x, a.y, b.x, b.y)
	text_edit.insert_text(repl, a.x, a.y)
	text_edit.end_complex_operation()
	var after := sel.x + repl.length()
	select_offsets(after, after)
	_find_matches = find_matches(text_edit.text, _find_edit.text, _find_case)
	_select_match(pick_match(_find_matches, after, true))
	_update_find_count()


## Replaces every hit in ONE undo step: the new text comes from `replace_all` and is written as a single
## complex operation (as `_commit` does), so Ctrl+Z restores all of them at once. The scroll position and
## caret line are kept; the note is not saved.
func _replace_all() -> void:
	if not _find_open:
		return
	var r := replace_all(text_edit.text, _find_edit.text, _replace_edit.text, _find_case)
	var n := int(r["count"])
	if n == 0:
		_find_count.text = "No matches"
		return
	var scroll := text_edit.scroll_vertical
	var line := text_edit.get_caret_line()
	text_edit.begin_complex_operation()
	var last := text_edit.get_line_count() - 1
	text_edit.remove_text(0, 0, last, text_edit.get_line(last).length())
	text_edit.insert_text(String(r["text"]), 0, 0)
	text_edit.end_complex_operation()
	text_edit.set_caret_line(mini(line, text_edit.get_line_count() - 1))
	text_edit.set_caret_column(0)
	text_edit.scroll_vertical = scroll
	_find_matches = find_matches(text_edit.text, _find_edit.text, _find_case)
	_hl.refresh()
	_find_notice = "Replaced %d" % n
	_update_find_count()


## The find hits in one line as column ranges, for the highlighter; nothing while the bar is closed.
func _matches_in_line(line_text: String) -> Array:
	if not _find_open or _find_edit == null or _find_edit.text.is_empty():
		return []
	return find_matches(line_text, _find_edit.text, _find_case)


# -- VE-3: footer and theme ----------------------------------------------------

## "N words · M links" from the host's counters. A counter the host did not give is left out, not
## shown as zero.
func _refresh_footer() -> void:
	if _footer_label == null or text_edit == null:
		return
	var parts := PackedStringArray()
	if word_counter.is_valid():
		var w := int(word_counter.call(text_edit.text))
		if w >= 0:
			parts.append("%d word%s" % [w, "" if w == 1 else "s"])
	if link_counter.is_valid():
		var l := int(link_counter.call(text_edit.text))
		if l >= 0:
			parts.append("%d link%s" % [l, "" if l == 1 else "s"])
	_footer_label.text = "  ·  ".join(parts)


var _hl_dark := DccTheme.is_dark()


## A palette flip re-colours the source. The vault window usually rebuilds on a flip, but the colours
## are the highlighter's own and cost nothing to refresh.
func _on_theme_changed() -> void:
	if _hl == null or _hl_dark == DccTheme.is_dark():
		return
	_hl_dark = DccTheme.is_dark()
	_hl.refresh_palette()
	text_edit.add_theme_color_override("font_color", _hl.color_of(Highlighter.BODY))
