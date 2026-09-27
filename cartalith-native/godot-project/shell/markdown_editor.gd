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
## A header row (file name, Write / Preview, Cancel, Save), a toolbar, and one
## body that is either the source (`text_edit`) or its rendering (`preview`).
## It never touches the disk: Save and Cancel are signals, and the vault
## window answers them through `vault_write_file`'s hash guard. So this file
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
## "note > heading", as Obsidian's reading view shows it) and `![[embed]]`,
## which is drawn as a labelled placeholder ("embedded: name"), **not** the
## embedded note or image -- disclosed, not hidden. A callout is drawn in one
## colour whatever its type; Obsidian varies colour and icon by type.
## **Ctrl+E** flips Write / Preview, as Obsidian's reading-view toggle does.
## **Insert template** puts one of the vault's templates at the caret, filled
## and with its properties merged (`cartalith_vault::template::insert`, reached
## through the `template_inserter` the host passes to `setup`).
##
## Every transform is a `static func` over a plain `String` and two character
## offsets, returning `{"text", "from", "to"}`. That is what lets
## `_mdedit_probe.gd` pin each one with exact strings and no UI at all, and it
## is why the buttons below are thin: read the selection as offsets, call the
## transform, write the result back as ONE undoable edit, restore the
## selection.

signal save_requested
signal cancel_requested
## "Reload from disk", offered after a refused Save: discard and re-read.
signal reload_requested
## Emitted after any edit, typed or made by a toolbar button.
signal edited
## Emitted when Write / Preview flips, so a host that rebuilds can keep it.
signal mode_changed(mode: String)

const FONT_BOLD := preload("res://fonts/FiraSans-Bold.ttf")
const FONT_ITALIC := preload("res://fonts/FiraSans-Italic.ttf")

## Rendered heading sizes. Not `DccTheme` tokens -- the theme has no heading
## scale below `FS_MODAL_TITLE` (16) and `FS_HERO` (26); these sit on that
## ladder: H1 at the modal title's 16 + 4, H2 at 16, H3 at 14.
const HEADING_PX := {1: 20, 2: 16, 3: 14, 4: 13, 5: 12, 6: 12}

var text_edit: TextEdit
var preview: RichTextLabel
var status_label: Label
var save_button: Button
var cancel_button: Button

## Every toolbar control, by key -- `bold`, `italic`, `underline`, `strike`,
## `highlight`, `h0`..`h3`, `bullet`, `number`, `task`, `quote`, `callout`,
## `code`, `code_block`, `link`, `tag`, `template`, `write`, `preview`. A probe
## presses the real button through this map.
var buttons := {}

var _files_source: Callable
var _mode := "write"
var _phone := false
var _saved_text := ""
var _link_popup: PopupMenu
var _link_files: PackedStringArray = PackedStringArray()
var _reload_button: Button
var _templates_source: Callable
var _template_inserter: Callable
var _template_popup: PopupMenu
var _template_rels: PackedStringArray = PackedStringArray()


## `initial` is the working text; `saved` is what the file held when it was
## last read or written (the dirty check compares against it). `files_source`
## returns the vault's `.md` paths for the note-link picker.
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

	_build_header(file_label)
	_build_toolbar()
	_build_body(initial)

	status_label = DccTheme.label("", "text_ghost", DccTheme.FS_SMALL)
	status_label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	status_label.custom_minimum_size.x = 160
	status_label.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	var status_row := HBoxContainer.new()
	status_row.add_theme_constant_override("separation", 8)
	status_row.add_child(status_label)
	_reload_button = DccWidgets.text_button(status_row, "Reload from disk (discards these edits)", func(): reload_requested.emit())
	_reload_button.visible = false
	add_child(status_row)

	set_mode(mode)
	_refresh_heading_lights()


func is_dirty() -> bool:
	return text_edit != null and text_edit.text != _saved_text


func mark_saved(text: String) -> void:
	_saved_text = text
	set_status("Saved.", false)


## `refused` shows the reload offer beside the message: after a hash refusal
## the only way to a writable state is to re-read the file.
func set_status(msg: String, refused: bool) -> void:
	if status_label == null:
		return
	status_label.text = msg
	status_label.add_theme_color_override("font_color",
		DccTheme.c("accent") if refused else DccTheme.c("text_ghost"))
	_reload_button.visible = refused


func mode() -> String:
	return _mode


func set_mode(m: String) -> void:
	_mode = "preview" if m == "preview" else "write"
	var writing := _mode == "write"
	## Ctrl+E is read from whichever of the two has focus, so the one on screen
	## must take it -- but only when the editor already had focus (asked before
	## hiding, which drops it); a rebuild must not steal focus from the window.
	var had_focus := text_edit.has_focus() or preview.has_focus()
	text_edit.visible = writing
	preview.visible = not writing
	if not writing:
		preview.text = to_bbcode(text_edit.text)
	if had_focus:
		(text_edit if writing else preview).grab_focus.call_deferred()
	for k in ["write", "preview"]:
		if buttons.has(k):
			DccWidgets.set_segment_on(buttons[k], k == _mode)
	## Formatting acts on the source; in Preview there is no selection to act on.
	for k in buttons:
		if k in ["write", "preview"]:
			continue
		(buttons[k] as Button).disabled = not writing
	mode_changed.emit(_mode)


# -- Building ---------------------------------------------------------------

## Desktop: one row -- title, Write / Preview, Cancel, Save. Phone: the title
## on its own line, because five things in one row do not fit 1080 px at phone
## density, and an `HBoxContainer` never wraps a sibling on its own.
func _build_header(file_label: String) -> void:
	var title := DccTheme.mono_label("EDITING  %s" % file_label, "text_dim", DccTheme.FS_SMALL, 1)
	title.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	var row := HBoxContainer.new()
	row.name = "EditorHeader"
	row.add_theme_constant_override("separation", 8)
	if _phone:
		title.autowrap_mode = TextServer.AUTOWRAP_ARBITRARY
		title.custom_minimum_size.x = 120
		add_child(title)
		add_child(row)
		row.add_child(DccTheme.spacer())
	else:
		add_child(row)
		row.add_child(title)
	## Write / Preview as a lit segment pair -- the DCC vocabulary's "one of a
	## set is lit" control (`DccWidgets.segment`).
	buttons["write"] = DccWidgets.segment(row, "Write", func(): set_mode("write"))
	buttons["preview"] = DccWidgets.segment(row, "Preview", func(): set_mode("preview"))
	cancel_button = DccWidgets.action(row, "Cancel", func(): cancel_requested.emit())
	cancel_button.tooltip_text = "Discards every edit since the note was opened or last saved. The file on disk is untouched."
	save_button = DccWidgets.action(row, "Save", func(): save_requested.emit(), true)
	save_button.tooltip_text = "Writes the whole note back to the vault. Refuses, and writes nothing, if the file changed on disk since it was opened here."


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
	_tool(bar, "strike", "S", "Strikethrough -- ~~…~~", func(): apply_inline("~~", "~~"))
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
	_tool(bar, "link", "[[ ]]", "Link to another note in this vault -- pick one from the list.", func(): _open_link_picker())
	_tool(bar, "tag", "#", "Tag -- #tag. Obsidian's tags hold no spaces, so a selected phrase is hyphenated.", func(): apply_tag())
	_sep(bar)
	_tool(bar, "template", "Insert template", "Insert one of this vault's templates at the caret, as Obsidian's Insert template does: {{title}}, {{date}} and {{time}} filled, its properties merged into this note's.", func(): _open_template_picker())

	_link_popup = PopupMenu.new()
	_link_popup.name = "EditorLinkPicker"
	DccWidgets.style_popup(_link_popup)
	_link_popup.id_pressed.connect(func(id: int):
		if id >= 0 and id < _link_files.size():
			insert_note_link(_link_files[id]))
	add_child(_link_popup)

	_template_popup = PopupMenu.new()
	_template_popup.name = "EditorTemplatePicker"
	DccWidgets.style_popup(_template_popup)
	_template_popup.id_pressed.connect(func(id: int):
		if id >= 0 and id < _template_rels.size():
			insert_template(_template_rels[id]))
	add_child(_template_popup)


func _tool(bar: Control, key: String, label_text: String, tip: String, cb: Callable) -> Button:
	var b := DccWidgets.segment(bar, label_text, cb)
	b.tooltip_text = tip
	b.custom_minimum_size.x = maxf(b.custom_minimum_size.x, 28)
	buttons[key] = b
	return b


func _sep(bar: Control) -> void:
	var r := DccTheme.rule(true)
	r.custom_minimum_size = Vector2(1, 18)
	bar.add_child(r)


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
	text_edit.text_changed.connect(func(): edited.emit())
	text_edit.caret_changed.connect(_refresh_heading_lights)
	text_edit.gui_input.connect(_on_source_input)
	add_child(text_edit)

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
	add_child(preview)


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

func _offset(line: int, col: int) -> int:
	var o := 0
	for i in range(line):
		o += text_edit.get_line(i).length() + 1
	return o + col


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


func apply_inline(open: String, close: String) -> void:
	var s := selection_offsets()
	_commit(wrap_inline(text_edit.text, s.x, s.y, open, close))


func apply_heading(level: int) -> void:
	var s := selection_offsets()
	_commit(set_heading(text_edit.text, s.x, s.y, level))


func apply_lines(kind: String) -> void:
	var s := selection_offsets()
	_commit(prefix_lines(text_edit.text, s.x, s.y, kind))


func apply_code_block() -> void:
	var s := selection_offsets()
	_commit(code_block(text_edit.text, s.x, s.y))


func insert_note_link(rel: String) -> void:
	var s := selection_offsets()
	_commit(note_link(text_edit.text, s.x, s.y, rel))


func apply_callout() -> void:
	var s := selection_offsets()
	_commit(callout(text_edit.text, s.x, s.y))


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


func _open_link_picker() -> void:
	_link_files = _files_source.call() if _files_source.is_valid() else PackedStringArray()
	_link_popup.clear()
	if _link_files.is_empty():
		_link_popup.add_item("No other notes in this vault")
		_link_popup.set_item_disabled(0, true)
	for i in _link_files.size():
		_link_popup.add_item(String(_link_files[i]), i)
	DccWidgets.popup_anchored(_link_popup, anchor_rect(buttons["link"]), 320)


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


## Ctrl+E from the rendered view goes back to the source (Obsidian's
## reading-view toggle is the same key both ways).
func _on_preview_input(ev: InputEvent) -> void:
	var k := ev as InputEventKey
	if k == null or not k.pressed or k.echo:
		return
	if (k.ctrl_pressed or k.meta_pressed) and not k.alt_pressed and not k.shift_pressed and k.keycode == KEY_E:
		set_mode("write")
		preview.accept_event()


func _on_source_input(ev: InputEvent) -> void:
	var k := ev as InputEventKey
	if k == null or not k.pressed or k.echo:
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
		_:
			return
	text_edit.accept_event()


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


static func _line_start(text: String, pos: int) -> int:
	if pos <= 0:
		return 0
	var i := text.rfind("\n", pos - 1)
	return i + 1


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


const _BULLET_RE := "^(\\s*)[-*+] "
const _NUMBER_RE := "^(\\s*)\\d+[.)] "
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
static func to_bbcode(md: String, accent_hex: String = "", dim_hex: String = "") -> String:
	var accent := accent_hex if accent_hex != "" else DccTheme.c("accent").to_html(false)
	var dim := dim_hex if dim_hex != "" else DccTheme.c("text_dim").to_html(false)
	var out := PackedStringArray()
	var in_code := false
	var in_callout := false
	var code_buf := PackedStringArray()
	var bullet := RegEx.create_from_string("^(\\s*)[-*+] (.*)$")
	var number := RegEx.create_from_string("^(\\s*)(\\d+)[.)] (.*)$")
	var task := RegEx.create_from_string("^(\\s*)[-*+] \\[(.)\\] (.*)$")
	var callout_head := RegEx.create_from_string(_CALLOUT_RE)
	md = md.replace("\r\n", "\n")
	## A leading YAML block is data, not prose: shown dim and monospaced, as
	## written, rather than as two rules around a paragraph.
	var body := strip_frontmatter(md)
	if body != md:
		var fm := md.substr(0, md.length() - body.length()).strip_edges()
		out.append("[color=#%s][code]%s[/code][/color]" % [dim, _esc(fm)])
		md = body
	for raw in md.split("\n"):
		var line := String(raw)
		if line.strip_edges().begins_with("```"):
			if in_code:
				out.append("[code]" + _esc("\n".join(code_buf)) + "[/code]")
				code_buf = PackedStringArray()
			in_code = not in_code
			continue
		if in_code:
			code_buf.append(line)
			continue
		## A callout: the header line names its type and optional title (the
		## `+`/`-` fold marker is dropped -- there is nothing to fold here), and
		## the `>` lines under it are its body, drawn as prose, not as a quote.
		var ch := callout_head.search(line)
		if ch != null:
			var title := ch.get_string(3).strip_edges()
			if title == "":
				title = ch.get_string(1).strip_edges().capitalize()
			out.append("[indent][color=#%s][b]%s[/b][/color][/indent]" % [accent, inline_bbcode(title, accent)])
			in_callout = true
			continue
		if not line.begins_with(">"):
			in_callout = false
		var hp := _heading_prefix(line)
		if hp.x > 0:
			out.append("[font_size=%d][b]%s[/b][/font_size]" % [HEADING_PX[hp.x], inline_bbcode(line.substr(hp.y), accent)])
			continue
		if line.begins_with(">"):
			var q := line.substr(1)
			if q.begins_with(" "):
				q = q.substr(1)
			if in_callout:
				out.append("[indent]%s[/indent]" % inline_bbcode(q, accent))
			else:
				out.append("[indent][color=#%s][i]%s[/i][/color][/indent]" % [dim, inline_bbcode(q, accent)])
			continue
		## Before the bullet test, which a task line also matches.
		var mt := task.search(line)
		if mt != null:
			var tdepth := mt.get_string(1).length() / 2
			if mt.get_string(2) == " ":
				out.append("%s  ☐  %s" % ["    ".repeat(tdepth), inline_bbcode(mt.get_string(3), accent)])
			else:
				out.append("%s  ☑  [color=#%s][s]%s[/s][/color]" % ["    ".repeat(tdepth), dim, inline_bbcode(mt.get_string(3), accent)])
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


static func _inline_text(s: String, accent_hex: String) -> String:
	var links: Array = []
	var embed := RegEx.create_from_string("!\\[\\[([^\\]|]+)(?:\\|([^\\]]+))?\\]\\]")
	var wiki := RegEx.create_from_string("\\[\\[([^\\]|]+)(?:\\|([^\\]]+))?\\]\\]")
	var md_link := RegEx.create_from_string("\\[([^\\]]+)\\]\\(([^)\\s]+)\\)")
	var k := 0
	## `![[embed]]` first -- `[[…]]` inside it would otherwise read as a link.
	## Drawn as a labelled placeholder, not the embedded note or image.
	for m in embed.search_all(s):
		links.append("[color=#%s][i]embedded: %s[/i][/color]" % [accent_hex, _esc(m.get_string(1))])
		s = s.replace(m.get_string(0), "\u0001%d\u0001" % k)
		k += 1
	for m in wiki.search_all(s):
		var shown := _wiki_shown(m.get_string(1), m.get_string(2))
		links.append("[color=#%s][url=%s]%s[/url][/color]" % [accent_hex, _esc(m.get_string(1)), _esc(shown)])
		s = s.replace(m.get_string(0), "\u0001%d\u0001" % k)
		k += 1
	for m in md_link.search_all(s):
		links.append("[color=#%s][url=%s]%s[/url][/color]" % [accent_hex, _esc(m.get_string(2)), _esc(m.get_string(1))])
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
			["~~(.+?)~~", "[s]$1[/s]"],
			["<u>(.+?)</u>", "[u]$1[/u]"]]:
		s = RegEx.create_from_string(String(pair[0])).sub(s, String(pair[1]), true)
	for j in links.size():
		s = s.replace("\u0001%d\u0001" % j, String(links[j]))
	return s
